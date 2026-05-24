use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::TaskId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentError {
    LLMError(String),
    ToolError(String),
    VerificationError(String),
    MemoryError(String),
    TaskExecutionError(String),
    MaxRetriesExceeded(String),
    NeedsHumanReview(String),
    Cancelled(String),
    Timeout(String),
    Internal(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::LLMError(msg) => write!(f, "LLM error: {}", msg),
            AgentError::ToolError(msg) => write!(f, "Tool error: {}", msg),
            AgentError::VerificationError(msg) => write!(f, "Verification error: {}", msg),
            AgentError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            AgentError::TaskExecutionError(msg) => write!(f, "Task execution error: {}", msg),
            AgentError::MaxRetriesExceeded(msg) => write!(f, "Max retries exceeded: {}", msg),
            AgentError::NeedsHumanReview(msg) => write!(f, "Needs human review: {}", msg),
            AgentError::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
            AgentError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            AgentError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolError {
    NotFound(String),
    ExecutionFailed(String),
    PermissionDenied(String),
    Timeout(String),
    InvalidParams(String),
    PathNotAllowed(String),
    CommandNotAllowed(String),
    OutputTooLarge(usize),
    UnknownAction(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::NotFound(name) => write!(f, "Tool not found: {}", name),
            ToolError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ToolError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ToolError::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
            ToolError::PathNotAllowed(p) => write!(f, "Path not allowed: {}", p),
            ToolError::CommandNotAllowed(cmd) => write!(f, "Command not allowed: {}", cmd),
            ToolError::OutputTooLarge(size) => write!(f, "Output too large: {} bytes", size),
            ToolError::UnknownAction(action) => write!(f, "Unknown action: {}", action),
        }
    }
}

impl From<ToolError> for AgentError {
    fn from(e: ToolError) -> Self {
        AgentError::ToolError(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationError {
    GateFailed(String),
    MissingCriterion(String),
    VerifierError(String),
    UnsupportedLanguage(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::GateFailed(msg) => write!(f, "Gate failed: {}", msg),
            VerificationError::MissingCriterion(msg) => write!(f, "Missing criterion: {}", msg),
            VerificationError::VerifierError(msg) => write!(f, "Verifier error: {}", msg),
            VerificationError::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
        }
    }
}

impl From<VerificationError> for AgentError {
    fn from(e: VerificationError) -> Self {
        AgentError::VerificationError(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryError {
    NotFound(String),
    StoreFailed(String),
    SearchFailed(String),
    StorageError(String),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::NotFound(key) => write!(f, "Memory entry not found: {}", key),
            MemoryError::StoreFailed(msg) => write!(f, "Store failed: {}", msg),
            MemoryError::SearchFailed(msg) => write!(f, "Search failed: {}", msg),
            MemoryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl From<MemoryError> for AgentError {
    fn from(e: MemoryError) -> Self {
        AgentError::MemoryError(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerError {
    QueueFull,
    TaskNotFound(TaskId),
    TaskAlreadyCompleted(TaskId),
    Internal(String),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::QueueFull => write!(f, "Task queue is full"),
            SchedulerError::TaskNotFound(id) => write!(f, "Task not found: {}", id.0),
            SchedulerError::TaskAlreadyCompleted(id) => write!(f, "Task already completed: {}", id.0),
            SchedulerError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
