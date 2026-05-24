use agent_core::agent::ToolRegistry;
use agent_core::error::AgentError;
use agent_core::llm::{LLMProvider, LLMRequest};
use agent_core::types::{
    AgentId, AgentRole, Context, Plan, PlanId, RetryPolicy, Step, StepId, StepResult, Task, Thought, ThoughtId,
    ToolCall, ToolCallId, ToolResult, Turn, Verdict,
};
use agent_core::verifier::Verifier;
use agent_verifier::{FailureTracker, ReflectionLoop};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, warn};

pub struct GenericAgent {
    id: AgentId,
    llm: Arc<dyn LLMProvider>,
    tools: Arc<dyn ToolRegistry>,
    verifier: Arc<dyn Verifier>,
    reflection: Option<Arc<ReflectionLoop>>,
}

impl GenericAgent {
    pub fn new(
        id: AgentId,
        llm: Arc<dyn LLMProvider>,
        tools: Arc<dyn ToolRegistry>,
        verifier: Arc<dyn Verifier>,
    ) -> Self {
        Self {
            id,
            llm,
            tools,
            verifier,
            reflection: None,
        }
    }

    pub fn with_reflection(mut self, reflection: Arc<ReflectionLoop>) -> Self {
        self.reflection = Some(reflection);
        self
    }

    pub fn id(&self) -> &AgentId {
        &self.id
    }

    pub fn role(&self) -> AgentRole {
        AgentRole::Executor
    }

    pub fn llm_provider(&self) -> &Arc<dyn LLMProvider> {
        &self.llm
    }

    pub fn verifier(&self) -> &Arc<dyn Verifier> {
        &self.verifier
    }

    pub fn reflection(&self) -> Option<&Arc<ReflectionLoop>> {
        self.reflection.as_ref()
    }

    pub async fn think(&self, task: &Task) -> Result<Thought, AgentError> {
        let available_tools = self.tools.available_tools().join(", ");
        let prompt = format!(
            "Task: {}\nDescription: {}\n\nAvailable tools: {}\n\nAnalyze this task and suggest an approach. Be specific about which tools to use.",
            task.title, task.description, available_tools
        );

        let request = LLMRequest {
            system_prompt: "You are an AI agent that analyzes tasks and plans execution. Respond in JSON with: reasoning (string), plan_suggestion (string), confidence (0-1)".to_string(),
            user_prompt: prompt,
            model: None,
            temperature: Some(0.5),
            max_tokens: Some(2000),
            response_format: Some(agent_core::llm::ResponseFormat::Json { schema: None }),
        };

        #[derive(Deserialize)]
        struct ThoughtResponse {
            reasoning: String,
            plan_suggestion: String,
            confidence: f64,
        }

        let response: ThoughtResponse = agent_core::llm::generate_structured(&*self.llm, request).await?;

        Ok(Thought {
            id: ThoughtId(uuid::Uuid::new_v4().to_string()),
            agent_id: self.id.clone(),
            reasoning: response.reasoning,
            plan_suggestion: response.plan_suggestion,
            confidence: response.confidence,
        })
    }

    pub async fn create_plan(&self, thought: &Thought, task: &Task) -> Result<Plan, AgentError> {
        let tools_info: Vec<String> = self
            .tools
            .available_tools()
            .iter()
            .map(|t| match t.as_str() {
                "filesystem" => {
                    "- filesystem (name: \"filesystem\"): read(path), write(path, content), list(path)".to_string()
                }
                "shell" => "- shell (name: \"shell\"): execute(command, workdir)".to_string(),
                "http" => "- http (name: \"http\"): get(url), post(url, body)".to_string(),
                "search" => "- search (name: \"search\"): search(query)".to_string(),
                other => format!("- {} (name: \"{}\")", other, other),
            })
            .collect();

        let prompt = format!(
            "Task: {}\n\nThought/reasoning: {}\n\nGenerate a step-by-step execution plan. Each step must specify which tool to use and the exact parameters to pass.\n\nAvailable tools:\n{}\n\nRespond in JSON with: steps (array of {{description, tool_to_use, params}}), reasoning (string). The tool_to_use field must be exactly one of: filesystem, shell, http, search. The params object should contain the exact arguments the tool expects.",
            task.description, thought.reasoning,
            tools_info.join("\n")
        );

        let request = LLMRequest {
            system_prompt: "You generate execution plans with specific tool calls. Output valid JSON.".to_string(),
            user_prompt: prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(3000),
            response_format: Some(agent_core::llm::ResponseFormat::Json { schema: None }),
        };

        #[derive(Deserialize)]
        struct PlanStep {
            description: String,
            tool_to_use: String,
            params: serde_json::Value,
        }

        #[derive(Deserialize)]
        struct PlanResponse {
            steps: Vec<PlanStep>,
            reasoning: String,
        }

        let response: PlanResponse = agent_core::llm::generate_structured(&*self.llm, request).await?;

        let steps: Vec<Step> = response
            .steps
            .into_iter()
            .enumerate()
            .map(|(i, s)| Step {
                id: StepId(format!("step-{}", i)),
                index: i,
                description: s.description,
                tool_requirements: vec![agent_core::types::ToolRequirement {
                    tool_name: s.tool_to_use,
                    description: String::new(),
                }],
                tool_params: s.params,
                criteria: task.criteria.clone(),
                max_retries: task.max_retries,
                timeout_seconds: task.timeout_seconds,
                dependencies: Vec::new(),
                parallel_group: None,
            })
            .collect();

        Ok(Plan {
            id: PlanId(uuid::Uuid::new_v4().to_string()),
            task_id: task.id.clone(),
            steps,
            reasoning: response.reasoning,
        })
    }

    pub async fn execute_step(&self, step: &Step, _context: &Context) -> Result<StepResult, AgentError> {
        let start = std::time::Instant::now();

        let tool_name = step
            .tool_requirements
            .first()
            .map(|t| t.tool_name.clone())
            .unwrap_or_default();

        let params = &step.tool_params;

        let call = ToolCall {
            id: ToolCallId(uuid::Uuid::new_v4().to_string()),
            name: tool_name.clone(),
            params: params.clone(),
        };

        info!(tool = %tool_name, step = %step.id.0, "Executing step");
        let tool_output = self.tools.execute(&tool_name, &call.params).await?;

        let tool_result = ToolResult {
            tool_name: tool_name.clone(),
            call,
            duration_ms: start.elapsed().as_millis() as u64,
            output: tool_output.clone(),
        };

        Ok(StepResult {
            step_id: step.id.clone(),
            output: tool_output.stdout,
            tool_results: vec![tool_result],
            success: tool_output.success,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub async fn execute_step_with_retry(
        &self,
        step: &Step,
        context: &Context,
        task: &Task,
    ) -> Result<(StepResult, Vec<Turn>), AgentError> {
        let reflection = match self.reflection.as_ref() {
            Some(r) => r.clone(),
            None => {
                let result = self.execute_step(step, context).await?;
                let _verdict = self.verifier.verify(&result, &task.criteria).await?;
                return Ok((result, vec![]));
            }
        };

        let mut history: Vec<Turn> = Vec::new();
        let mut tracker = FailureTracker::new(task.max_retries);
        let retry_policy = RetryPolicy::default_executor();

        loop {
            let result = self.execute_step(step, context).await?;
            let verdict = self.verifier.verify(&result, &task.criteria).await?;

            if verdict.passed {
                return Ok((result, history));
            }

            if !tracker.can_retry() {
                return Ok((result, history));
            }

            let failure_type = verdict.issues.first().cloned().unwrap_or_default();
            tracker.record_failure(&failure_type);

            if tracker.escalated {
                warn!(
                    "Escalating after {} consecutive same-type failures",
                    tracker.consecutive_same_failures
                );
            }

            let reflection = reflection
                .generate_reflection(task, &result, &verdict, &history)
                .await?;

            history.push(Turn {
                step_result: result,
                verdict,
                reflection,
            });

            if tracker.can_retry() {
                let delay = retry_policy.delay_for_attempt(tracker.attempts);
                tokio::time::sleep(delay).await;
            }
        }
    }

    pub async fn verify_step(
        &self,
        result: &StepResult,
        criteria: &[agent_core::types::Criterion],
    ) -> Result<Verdict, AgentError> {
        let verdict = self.verifier.verify(result, criteria).await?;
        Ok(verdict)
    }

    pub async fn run(&self, task: &Task) -> Result<(Vec<StepResult>, Vec<Verdict>, Vec<Vec<Turn>>), AgentError> {
        info!(task = %task.id.0, "Agent starting task");

        let thought = self.think(task).await?;
        info!(confidence = %thought.confidence, "Thought generated");

        let plan = self.create_plan(&thought, task).await?;
        info!(steps = %plan.steps.len(), "Plan created");

        let mut results = Vec::new();
        let mut verdicts = Vec::new();
        let mut all_turns = Vec::new();
        let context = Context::new(".");

        for step in &plan.steps {
            info!(step = %step.description, "Executing step with retry");
            let (result, turns) = self.execute_step_with_retry(step, &context, task).await?;

            let verdict = if turns.is_empty() {
                let v = self.verify_step(&result, &task.criteria).await?;
                verdicts.push(v.clone());
                v
            } else {
                let last_verdict = turns.last().unwrap().verdict.clone();
                verdicts.push(last_verdict.clone());
                last_verdict
            };

            results.push(result);
            all_turns.push(turns);

            if !verdict.passed {
                warn!("Step failed after all retries: {}", step.description);
                return Ok((results, verdicts, all_turns));
            }
        }

        Ok((results, verdicts, all_turns))
    }
}
