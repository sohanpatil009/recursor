use serde::{Deserialize, Serialize};

use crate::error::AgentError;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    // Task lifecycle
    TaskCreated(TaskId, Box<Task>),
    TaskStarted(TaskId),
    TaskPaused(TaskId, String),
    TaskCancelled(TaskId, String),
    TaskCompleted(TaskId, Box<FinalOutput>),
    TaskFailed(TaskId, AgentError),

    // Agent lifecycle
    AgentSpawned(AgentId, AgentRole, TaskId),
    AgentBusy(AgentId),
    AgentIdle(AgentId),

    // Agent reasoning
    ThoughtComplete(AgentId, Box<Thought>),
    PlanCreated(AgentId, Box<Plan>),

    // Step execution
    StepStarted(AgentId, StepId),
    StepCompleted(AgentId, StepId, Box<StepResult>),
    StepFailed(AgentId, StepId, AgentError),

    // Tool execution
    ToolCalled(AgentId, ToolCall),
    ToolResult(AgentId, ToolCallId, ToolOutput),

    // Verification
    VerificationGatePassed(AgentId, String),
    VerificationGateFailed(AgentId, String, String),
    VerdictReached(AgentId, Verdict),

    // Retry / reflection
    RetryScheduled(AgentId, usize, RetryPolicy),
    ReflectionGenerated(AgentId, Box<Reflection>),
    EscalationTriggered(AgentId, String),

    // Human oversight
    HumanApprovalRequested(AgentId, Box<ApprovalRequest>),
    HumanApprovalGranted(AgentId, String),
    HumanApprovalDenied(AgentId, String, String),

    // System
    SystemStart,
    SystemShutdown,
    SystemError(AgentError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserCommand {
    SubmitTask(Box<Task>),
    CancelTask(TaskId),
    PauseTask(TaskId),
    ResumeTask(TaskId),
    ApproveAction(String),
    RejectAction(String, String),
    ProvideFeedback(TaskId, String),
    Shutdown,
}
