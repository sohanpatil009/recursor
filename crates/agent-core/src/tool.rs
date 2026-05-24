use crate::error::ToolError;
use crate::types::ToolOutput;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, params: &serde_json::Value) -> Result<ToolOutput, ToolError>;
    fn requires_approval(&self) -> bool {
        false
    }
}

pub type ToolBox = Box<dyn Tool>;
