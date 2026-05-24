use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub String);

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThoughtId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolCallId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Planner,
    Executor,
    Verifier,
    Critic,
    Supervisor,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Queued,
    InProgress,
    Paused(String),
    AwaitingApproval(Box<ApprovalRequest>),
    Completed(Box<FinalOutput>),
    Failed(Box<crate::error::AgentError>),
    Cancelled(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub description: String,
    pub proposed_action: String,
    pub risks: Vec<String>,
    pub context: Context,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PauseReason {
    UserRequested,
    Escalated,
    ResourceExhausted,
}

impl fmt::Display for PauseReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PauseReason::UserRequested => write!(f, "user_requested"),
            PauseReason::Escalated => write!(f, "escalated"),
            PauseReason::ResourceExhausted => write!(f, "resource_exhausted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReason {
    UserRequested,
    Timeout,
    InternalError(String),
}

impl fmt::Display for CancelReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelReason::UserRequested => write!(f, "user_requested"),
            CancelReason::Timeout => write!(f, "timeout"),
            CancelReason::InternalError(e) => write!(f, "internal_error: {}", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    DependencyFailed,
    ConditionNotMet,
    ManuallySkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockReason {
    PermissionDenied,
    RateLimited,
    RequiresApproval,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: TaskStatus,
    pub max_retries: usize,
    pub timeout_seconds: u64,
    pub criteria: Vec<Criterion>,
    pub context: Context,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub parent_task: Option<TaskId>,
    pub subtasks: Vec<TaskId>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub variables: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn new(working_directory: &str) -> Self {
        Self {
            working_directory: working_directory.to_string(),
            environment: HashMap::new(),
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub index: usize,
    pub description: String,
    pub tool_requirements: Vec<ToolRequirement>,
    pub tool_params: serde_json::Value,
    pub criteria: Vec<Criterion>,
    pub max_retries: usize,
    pub timeout_seconds: u64,
    pub dependencies: Vec<StepId>,
    pub parallel_group: Option<GroupId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRequirement {
    pub tool_name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub task_id: TaskId,
    pub steps: Vec<Step>,
    pub reasoning: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thought {
    pub id: ThoughtId,
    pub agent_id: AgentId,
    pub reasoning: String,
    pub plan_suggestion: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub name: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    pub data: Option<serde_json::Value>,
}

impl ToolOutput {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            exit_code: Some(0),
            stdout: output.into(),
            stderr: String::new(),
            truncated: false,
            data: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            exit_code: Some(1),
            stdout: String::new(),
            stderr: error.into(),
            truncated: false,
            data: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: StepId,
    pub output: String,
    pub tool_results: Vec<ToolResult>,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub call: ToolCall,
    pub output: ToolOutput,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub passed: bool,
    pub confidence: ConfidenceScore,
    pub evidence: Vec<String>,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

impl Verdict {
    pub fn pass(confidence: ConfidenceScore) -> Self {
        Self {
            passed: true,
            confidence,
            evidence: Vec::new(),
            issues: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn fail(confidence: ConfidenceScore, issues: Vec<String>) -> Self {
        Self {
            passed: false,
            confidence,
            evidence: Vec::new(),
            issues,
            suggestions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub structural: f64,
    pub llm_verification: f64,
    pub tool_verification: f64,
    pub self_consistency: f64,
    pub overall: f64,
}

impl ConfidenceScore {
    pub fn new(structural: f64, llm: f64, tool: f64, consistency: f64) -> Self {
        let overall = structural * 0.3 + llm * 0.4 + tool * 0.2 + consistency * 0.1;
        Self {
            structural,
            llm_verification: llm,
            tool_verification: tool,
            self_consistency: consistency,
            overall,
        }
    }

    pub fn zero() -> Self {
        Self {
            structural: 0.0,
            llm_verification: 0.0,
            tool_verification: 0.0,
            self_consistency: 0.0,
            overall: 0.0,
        }
    }

    pub fn is_pass(&self, threshold: f64) -> bool {
        self.overall >= threshold
    }

    pub fn requires_review(&self, threshold: f64) -> bool {
        self.overall >= threshold * 0.7 && self.overall < threshold
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    pub step_result: StepResult,
    pub verdict: Verdict,
    pub reflection: Reflection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reflection {
    pub root_cause: String,
    pub changes_required: Vec<String>,
    pub keep_same: Vec<String>,
    pub next_attempt_confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalOutput {
    pub output: String,
    pub verdict: Verdict,
    pub attempts: usize,
    pub tool_results: Vec<ToolResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Criterion {
    JsonSchema(String),
    RequiredFields(Vec<String>),
    RegexPattern(String),
    ExitCode(i32),
    OutputBounds { max_length: usize },
    ToolExecuted,
    Compiled,
    TestsPassed,
    LintPassed,
    Custom(String, serde_json::Value),
}

impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Criterion::JsonSchema(_) => write!(f, "json_schema"),
            Criterion::RequiredFields(fields) => write!(f, "required_fields: {}", fields.join(", ")),
            Criterion::RegexPattern(p) => write!(f, "regex: {}", p),
            Criterion::ExitCode(code) => write!(f, "exit_code: {}", code),
            Criterion::OutputBounds { max_length } => write!(f, "output_bounds: max {} chars", max_length),
            Criterion::ToolExecuted => write!(f, "tool_executed"),
            Criterion::Compiled => write!(f, "compiled"),
            Criterion::TestsPassed => write!(f, "tests_passed"),
            Criterion::LintPassed => write!(f, "lint_passed"),
            Criterion::Custom(name, _) => write!(f, "custom: {}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub importance: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub key: String,
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointState {
    pub task_id: TaskId,
    pub step_index: usize,
    pub completed_steps: Vec<StepId>,
    pub partial_results: Vec<StepResult>,
    pub context: Context,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl RetryPolicy {
    pub fn default_executor() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }

    pub fn delay_for_attempt(&self, attempt: usize) -> std::time::Duration {
        let delay = self.base_delay_ms * 2u64.pow(attempt as u32);
        let delay = delay.min(self.max_delay_ms);
        std::time::Duration::from_millis(delay)
    }
}
