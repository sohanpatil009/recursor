use agent_core::agent::ToolRegistry;
use agent_core::error::ToolError;
use agent_core::tool::ToolBox;
use agent_core::types::ToolOutput;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct SimpleToolRegistry {
    tools: HashMap<String, ToolBox>,
}

impl SimpleToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: ToolBox) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }
}

#[async_trait]
impl ToolRegistry for SimpleToolRegistry {
    async fn execute(&self, tool_name: &str, params: &serde_json::Value) -> Result<ToolOutput, ToolError> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| ToolError::NotFound(tool_name.to_string()))?;

        tool.execute(params).await
    }

    fn available_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for SimpleToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
