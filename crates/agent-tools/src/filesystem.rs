use agent_core::error::ToolError;
use agent_core::tool::Tool;
use agent_core::types::ToolOutput;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct FilesystemTool {
    allowed_paths: Vec<PathBuf>,
    max_file_size: usize,
}

impl FilesystemTool {
    pub fn new(working_directory: &Path) -> Self {
        let cwd = dunce::canonicalize(working_directory).unwrap_or_else(|_| working_directory.to_path_buf());
        Self {
            allowed_paths: vec![cwd],
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        let p = Path::new(path);
        let cwd = &self.allowed_paths[0];
        let resolved = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };

        let canonical = if resolved.exists() {
            dunce::canonicalize(&resolved).unwrap_or(resolved.clone())
        } else if let Some(parent) = resolved.parent() {
            let canonical_parent = dunce::canonicalize(parent)
                .map_err(|_| ToolError::PathNotAllowed(format!("parent not found: {}", parent.display())))?;
            canonical_parent.join(resolved.file_name().unwrap())
        } else {
            return Err(ToolError::PathNotAllowed(format!("cannot resolve: {}", path)));
        };

        if !self.allowed_paths.iter().any(|p| canonical.starts_with(p)) {
            return Err(ToolError::PathNotAllowed(canonical.display().to_string()));
        }
        Ok(canonical)
    }
}

#[async_trait]
impl Tool for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Read, write, and list files and directories"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["read", "write", "list"] },
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["action", "path"]
        })
    }

    async fn execute(&self, params: &serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = params["path"]
            .as_str()
            .ok_or(ToolError::InvalidParams("missing path".into()))?;
        let resolved = self.resolve_path(path)?;
        let action = params["action"].as_str().unwrap_or_else(|| {
            if params.get("content").is_some() {
                "write"
            } else {
                "read"
            }
        });

        match action {
            "read" => {
                let content = tokio::fs::read_to_string(&resolved)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                Ok(ToolOutput::success(content))
            }
            "write" => {
                let content = params["content"]
                    .as_str()
                    .ok_or(ToolError::InvalidParams("missing content".into()))?;
                if content.len() > self.max_file_size {
                    return Err(ToolError::OutputTooLarge(content.len()));
                }
                if let Some(parent) = resolved.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                }
                tokio::fs::write(&resolved, content)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                Ok(ToolOutput::success(format!("Written {} bytes", content.len())))
            }
            "list" => {
                let mut entries = tokio::fs::read_dir(&resolved)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let mut items = Vec::new();
                while let Some(entry) = entries
                    .next_entry()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
                {
                    items.push(entry.file_name().to_string_lossy().to_string());
                }
                Ok(ToolOutput::success(serde_json::to_string(&items).unwrap_or_default()))
            }
            _ => Err(ToolError::UnknownAction(action.to_string())),
        }
    }
}
