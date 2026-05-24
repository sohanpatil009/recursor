#![allow(non_snake_case)]

use agent_core::event::{AgentEvent, UserCommand};
use agent_core::types::{Context, Criterion, Priority, Task, TaskId, TaskStatus};
use agent_runtime::AgentEngine;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

fn build_engine() -> Result<(AgentEngine, broadcast::Receiver<AgentEvent>), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .map_err(|_| "GEMINI_API_KEY or OPENAI_API_KEY not set".to_string())?;

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

    let mut pipeline = agent_verifier::VerifierPipeline::new();
    pipeline.add_gate(Box::new(agent_verifier::StructuralVerifier));
    pipeline.add_gate(Box::new(agent_verifier::DeterministicVerifier));
    pipeline.add_gate(Box::new(agent_verifier::LLMVerifier::new(llm.clone())));
    pipeline.add_gate(Box::new(agent_verifier::CodeVerifier::new()));

    let tools: Arc<dyn agent_core::agent::ToolRegistry> = Arc::new(registry);
    let verifier = Arc::new(pipeline);

    let (event_tx, event_rx) = broadcast::channel(10_000);

    let reflection = Arc::new(agent_verifier::ReflectionLoop::new(llm.clone()));
    let agent = Arc::new(
        agent_runtime::GenericAgent::new(
            agent_core::types::AgentId("desktop-agent".to_string()),
            llm,
            tools,
            verifier,
        )
        .with_reflection(reflection),
    );

    let engine = AgentEngine::new(agent, event_tx);
    Ok((engine, event_rx))
}

type SharedEngine = Arc<tokio::sync::Mutex<(AgentEngine, broadcast::Receiver<AgentEvent>)>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    dioxus::launch(App);
    Ok(())
}

fn App() -> Element {
    let tasks = use_signal(Vec::<Task>::new);
    let events = use_signal(Vec::<AgentEvent>::new);
    let mut selected = use_signal(|| None::<TaskId>);
    let loading = use_signal(|| false);

    let engine: SharedEngine = use_hook(|| {
        let (engine, rx) = build_engine().unwrap_or_else(|e| {
            eprintln!("Failed to build engine: {}", e);
            let (tx, rx) = broadcast::channel(10_000);
            let agent = Arc::new(agent_runtime::GenericAgent::new(
                agent_core::types::AgentId("fallback".to_string()),
                Arc::new(agent_llm::OpenAIProvider::new("none".to_string(), "gpt-4o".to_string())),
                Arc::new(agent_tools::SimpleToolRegistry::new()),
                Arc::new(agent_verifier::VerifierPipeline::new()),
            ));
            (AgentEngine::new(agent, tx), rx)
        });
        Arc::new(Mutex::new((engine, rx)))
    });

    rsx! {
        div {
            class: "app-container",
            style: "
                font-family: system-ui, sans-serif;
                padding: 20px;
                max-width: 1200px;
                margin: 0 auto;
                background: #fff;
                color: #333;
                min-height: 100vh;
            ",
            h1 { " Agentic" }
            agent_ui::TaskInput {
                disabled: *loading.read(),
                on_submit: move |prompt: String| {
                    let engine = engine.clone();
                    let mut loading = loading;
                    let mut events = events;
                    let mut tasks = tasks;
                    async move {
                        loading.set(true);
                        let task = Task {
                            id: TaskId(uuid::Uuid::new_v4().to_string()),
                            title: prompt,
                            description: String::new(),
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
                        };

                        // Dispatch task (sync lock, no await held)
                        {
                            let mut guard = engine.lock().await;
                            guard.0.dispatch(UserCommand::SubmitTask(Box::new(task)));
                        }

                        // Process engine (this will do think→plan→execute→verify)
                        {
                            let mut guard = engine.lock().await;
                            let (ref mut eng, ref mut rx) = &mut *guard;
                            let _ = eng.process_next().await;
                            while let Ok(evt) = rx.try_recv() {
                                events.write().push(evt);
                            }
                            let all = eng.all_tasks().into_iter().cloned().collect::<Vec<_>>();
                            drop(guard);
                            tasks.write().clear();
                            tasks.write().extend(all);
                        }

                        loading.set(false);
                    }
                },
            }
            div {
                class: "main-grid",
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 20px;",
                agent_ui::TaskList {
                    tasks: tasks.read().clone(),
                    on_select: move |id: TaskId| selected.set(Some(id)),
                }
                agent_ui::TaskDetail {
                    task: {
                        let id = selected.read().clone();
                        id.and_then(|id| tasks.read().iter().find(|t| t.id == id).cloned())
                    },
                }
            }
            agent_ui::AgentLog {
                events: events.read().clone(),
                max_height: Some("400px".to_string()),
            }
        }
    }
}
