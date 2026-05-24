use async_trait::async_trait;

use crate::error::{AgentError, ToolError};
use crate::types::*;

#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> &AgentId;
    fn role(&self) -> AgentRole;

    async fn think(&self, task: &Task, context: &Context) -> Result<Thought, AgentError>;

    async fn plan(&self, thought: &Thought) -> Result<Plan, AgentError>;

    async fn execute(&self, step: &Step, tools: &dyn ToolRegistry) -> Result<StepResult, AgentError>;
}

#[async_trait]
pub trait ToolRegistry: Send + Sync {
    async fn execute(&self, tool_name: &str, params: &serde_json::Value) -> Result<ToolOutput, ToolError>;
    fn available_tools(&self) -> Vec<String>;
}
