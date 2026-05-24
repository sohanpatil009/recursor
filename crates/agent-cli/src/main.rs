use agent_core::agent::ToolRegistry;
use agent_core::error::AgentError;
use agent_core::event::AgentEvent;
use agent_core::types::*;
use agent_runtime::AgentEngine;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Parser)]
#[command(name = "agentic", about = "Agentic AI Agent Platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a task with a natural language prompt
    Run {
        /// Task description
        prompt: String,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run a task with interactive live event stream
    RunInteractive {
        /// Task description
        prompt: String,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// List all tasks
    List,
    /// Show task status
    Status {
        /// Task ID
        id: String,
    },
}

fn build_engine() -> Result<(AgentEngine, broadcast::Receiver<AgentEvent>), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .map_err(|_| {
            AgentError::LLMError("GEMINI_API_KEY or OPENAI_API_KEY not set. Add it to .env file.".to_string())
        })?;

    let use_gemini = std::env::var("GEMINI_API_KEY").is_ok();
    let llm: Arc<dyn agent_core::llm::LLMProvider> = if use_gemini {
        Arc::new(agent_llm::GeminiProvider::new(api_key, "gemini-3.5-flash".to_string()))
    } else {
        Arc::new(agent_llm::OpenAIProvider::new(api_key, "gpt-4o".to_string()))
    };

    let mut registry = agent_tools::SimpleToolRegistry::new();
    let cwd = std::env::current_dir()?;
    registry.register(Box::new(agent_tools::FilesystemTool::new(&cwd)));
    registry.register(Box::new(agent_tools::ShellTool::new(cwd.to_string_lossy().as_ref())));
    registry.register(Box::new(agent_tools::HttpTool::default()));
    registry.register(Box::new(agent_tools::SearchTool));

    // Build verification pipeline with all 4 gates
    let mut pipeline = agent_verifier::VerifierPipeline::new();
    pipeline.add_gate(Box::new(agent_verifier::StructuralVerifier));
    pipeline.add_gate(Box::new(agent_verifier::DeterministicVerifier));
    pipeline.add_gate(Box::new(agent_verifier::LLMVerifier::new(llm.clone())));
    pipeline.add_gate(Box::new(agent_verifier::CodeVerifier::new()));

    let tools: Arc<dyn ToolRegistry> = Arc::new(registry);
    let verifier = Arc::new(pipeline);

    let (event_tx, event_rx) = broadcast::channel(10_000);

    // Wire up reflection loop
    let reflection = Arc::new(agent_verifier::ReflectionLoop::new(llm.clone()));

    let agent = Arc::new(
        agent_runtime::GenericAgent::new(AgentId("agent-1".to_string()), llm, tools, verifier)
            .with_reflection(reflection),
    );

    // Wire up SQLite memory store
    let store = std::env::current_dir()?.join("agentic.db");
    let store_path = store.to_string_lossy().to_string();
    let rt = tokio::runtime::Handle::current();
    let memory_store = Arc::new(rt.block_on(async {
        agent_runtime::SqliteMemoryStore::new(&store_path)
            .await
            .map_err(|e| AgentError::LLMError(format!("Failed to init SQLite: {}", e)))
    })?);

    let engine = AgentEngine::new(agent, event_tx).with_store(memory_store);

    Ok((engine, event_rx))
}

fn print_task_result(task: &Task, verbose: bool) {
    match &task.status {
        TaskStatus::Completed(output) => {
            println!("╭──────────────────────────────────────╮");
            println!("│ ✅ Task Complete                      │");
            println!("╰──────────────────────────────────────╯");
            println!(" Attempts: {}", output.attempts);
            println!(" Confidence: {:.0}%", output.verdict.confidence.overall * 100.0);
            println!();
            println!(" Output:");
            for line in output.output.lines() {
                println!("   {}", line);
            }
            if verbose {
                println!();
                println!(" Tool calls: {}", output.tool_results.len());
                for tr in &output.tool_results {
                    println!("   • {} ({})", tr.tool_name, tr.duration_ms);
                }
            }
        }
        TaskStatus::Failed(err) => {
            println!("╭──────────────────────────────────────╮");
            println!("│ ❌ Task Failed                       │");
            println!("╰──────────────────────────────────────╯");
            println!(" Error: {}", err);
        }
        _ => {
            println!("Status: {:?}", task.status);
        }
    }
}

fn escalation_handler(request: ApprovalRequest) -> bool {
    println!();
    println!("╭──────────────────────────────────────╮");
    println!("│ 👋 Human Approval Required           │");
    println!("╰──────────────────────────────────────╯");
    println!(" Description: {}", request.description);
    println!(" Action: {}", request.proposed_action);
    if !request.risks.is_empty() {
        println!(" Risks:");
        for risk in &request.risks {
            println!("   ⚠ {}", risk);
        }
    }
    println!();
    loop {
        print!("Approve? (y/N): ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" | "" => return false,
            _ => println!("Please answer y or n."),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("agent=info".parse()?))
        .init();

    let cli = Cli::parse();
    let (mut engine, mut rx) = build_engine()?;

    let run_task = |prompt: String| -> Result<Task, Box<dyn std::error::Error>> {
        let trimmed = prompt.trim().to_string();
        if trimmed.is_empty() {
            return Err("Empty task prompt. Please provide a description of what you want the agent to do.".into());
        }
        if trimmed.len() > 10000 {
            return Err(format!("Task prompt too long ({} chars). Max is 10000.", trimmed.len()).into());
        }
        Ok(Task {
            id: TaskId(uuid::Uuid::new_v4().to_string()),
            title: trimmed.clone(),
            description: trimmed,
            priority: Priority::Normal,
            status: TaskStatus::Pending,
            max_retries: 3,
            timeout_seconds: 120,
            criteria: vec![Criterion::ToolExecuted, Criterion::ExitCode(0)],
            context: Context::new("."),
            created_at: chrono::Utc::now(),
            parent_task: None,
            subtasks: Vec::new(),
            tags: Vec::new(),
        })
    };

    match cli.command {
        Commands::Run { prompt, verbose } => {
            println!("╭──────────────────────────────────────╮");
            println!("│ 🤖 Agentic AI Agent                   │");
            println!("╰──────────────────────────────────────╯");
            println!();
            println!("📋 Task: {}", prompt);
            println!();

            let task = run_task(prompt)?;
            engine.dispatch(agent_core::event::UserCommand::SubmitTask(Box::new(task)));
            engine.process_next().await?;

            for task in engine.all_tasks() {
                if matches!(task.status, TaskStatus::AwaitingApproval(_)) {
                    println!("⏸ Task awaiting approval...");
                }
                print_task_result(task, verbose);
            }
        }
        Commands::RunInteractive { prompt, verbose } => {
            println!("╭──────────────────────────────────────╮");
            println!("│ 🤖 Agentic AI Agent (interactive)    │");
            println!("╰──────────────────────────────────────╯");
            println!();
            println!("📋 Task: {}", prompt);
            println!();

            let task = run_task(prompt)?;
            engine.dispatch(agent_core::event::UserCommand::SubmitTask(Box::new(task)));

            loop {
                tokio::select! {
                    result = async { engine.process_next().await } => {
                        if let Err(e) = result {
                            println!("\n⚠ Engine error: {}", e);
                        }
                        // Check if task is awaiting approval
                        let needs_approval = engine.all_tasks().iter().any(|t| {
                            matches!(t.status, TaskStatus::AwaitingApproval(_))
                        });
                        if needs_approval {
                            println!("\n⚠ Task needs human approval. Processing...");
                        }
                        break;
                    }
                    evt = rx.recv() => {
                        if let Ok(event) = evt {
                            match &event {
                                AgentEvent::TaskStarted(_) => println!("🔄 Task started..."),
                                AgentEvent::ThoughtComplete(_, _) => println!("💭 Thinking..."),
                                AgentEvent::PlanCreated(_, _) => println!("📝 Planning..."),
                                AgentEvent::StepStarted(_, id) => println!("⚙️ Step {} started...", id.0),
                                AgentEvent::ToolCalled(_, call) => println!("🔧 Calling {}...", call.name),
                                AgentEvent::VerificationGatePassed(_, gate) => println!("✅ Gate {} passed", gate),
                                AgentEvent::VerificationGateFailed(_, gate, reason) => println!("❌ Gate {} failed: {}", gate, reason),
                                AgentEvent::RetryScheduled(_, attempt, _) => println!("🔄 Retry #{}...", attempt),
                                AgentEvent::ReflectionGenerated(_, _) => println!("🔄 Reflecting on failure..."),
                                AgentEvent::TaskCompleted(_, output) => println!("\n✅ Task complete! Confidence: {:.0}%", output.verdict.confidence.overall * 100.0),
                                AgentEvent::TaskFailed(_, err) => println!("\n❌ Task failed: {}", err),
                                AgentEvent::HumanApprovalRequested(_, req) => {
                                    if escalation_handler(*req.clone()) {
                                        println!("✅ Approved by user");
                                    } else {
                                        println!("❌ Rejected by user");
                                    }
                                }
                                AgentEvent::EscalationTriggered(_, reason) => println!("⚠ Escalation: {}", reason),
                                _ => {}
                            }
                        }
                    }
                }
            }

            for task in engine.all_tasks() {
                print_task_result(task, verbose);
            }
        }
        Commands::List => {
            let tasks = engine.all_tasks();
            if tasks.is_empty() {
                println!("No tasks.");
                return Ok(());
            }
            for task in tasks {
                let status = match &task.status {
                    TaskStatus::Completed(_) => "✅",
                    TaskStatus::Failed(_) => "❌",
                    TaskStatus::InProgress => "🔄",
                    TaskStatus::Pending => "⏳",
                    TaskStatus::Paused(_) => "⏸",
                    TaskStatus::Queued => "📋",
                    TaskStatus::AwaitingApproval(_) => "👀",
                    TaskStatus::Cancelled(_) => "🚫",
                };
                println!("{} {} — {}", status, &task.id.0[..8], task.title);
            }
        }
        Commands::Status { id } => {
            let task_id = TaskId(id);
            if let Some(task) = engine.get_task(&task_id) {
                println!("ID:       {}", task.id.0);
                println!("Title:    {}", task.title);
                println!("Priority: {:?}", task.priority);
                println!("Status:   {:?}", task.status);
            } else {
                println!("Task not found: {}", task_id.0);
            }
        }
    }

    Ok(())
}
