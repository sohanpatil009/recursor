use agent_core::error::ToolError;
use agent_core::tool::Tool;
use agent_core::types::ToolOutput;
use async_trait::async_trait;
use regex::Regex;
use std::time::Duration;

const BLOCKED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "format ",
    "shutdown",
    "reboot",
    "del /F /S",
    "rd /S /Q",
    "del /f /s",
    "rd /s /q",
    "dd if=",
    "mkfs.",
    "> /dev/sda",
    "| shutdown",
];

pub struct ShellTool {
    blocked_patterns: Vec<Regex>,
    max_execution_time: Duration,
    max_output_size: usize,
    _allowed_workdirs: Vec<String>,
}

impl ShellTool {
    pub fn new(working_directory: &str) -> Self {
        let mut blocked = Vec::new();
        for cmd in BLOCKED_COMMANDS {
            let escaped = regex::escape(cmd);
            if let Ok(re) = Regex::new(&format!("(?i){}", escaped)) {
                blocked.push(re);
            }
        }

        Self {
            blocked_patterns: blocked,
            max_execution_time: Duration::from_secs(30),
            max_output_size: 1024 * 1024, // 1MB
            _allowed_workdirs: vec![working_directory.to_string()],
        }
    }

    fn is_command_blocked(&self, command: &str) -> bool {
        self.blocked_patterns.iter().any(|re| re.is_match(command))
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute shell commands with restrictions"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "workdir": { "type": "string" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: &serde_json::Value) -> Result<ToolOutput, ToolError> {
        let command = params["command"]
            .as_str()
            .ok_or(ToolError::InvalidParams("missing command".into()))?;

        if self.is_command_blocked(command) {
            return Err(ToolError::CommandNotAllowed(command.to_string()));
        }

        let workdir = params["workdir"]
            .as_str()
            .and_then(|d| if d.is_empty() { None } else { Some(d) })
            .unwrap_or(".");

        #[cfg(not(target_family = "wasm"))]
        {
            let output = tokio::time::timeout(
                self.max_execution_time,
                tokio::process::Command::new(if cfg!(target_os = "windows") { "cmd" } else { "sh" })
                    .arg(if cfg!(target_os = "windows") { "/C" } else { "-c" })
                    .arg(command)
                    .current_dir(workdir)
                    .output(),
            )
            .await
            .map_err(|_| ToolError::Timeout(command.to_string()))?
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let truncated = stdout.len() > self.max_output_size;

            Ok(ToolOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: if truncated {
                    stdout[..self.max_output_size].to_string()
                } else {
                    stdout
                },
                stderr,
                truncated,
                data: None,
            })
        }

        #[cfg(target_family = "wasm")]
        {
            let _ = workdir;
            Err(ToolError::ExecutionFailed("Shell tool not available in web".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_commands() {
        let tool = ShellTool::new(".");
        assert!(tool.is_command_blocked("rm -rf /"));
        assert!(tool.is_command_blocked("rm -rf /*"));
        assert!(tool.is_command_blocked("shutdown -s"));
        assert!(tool.is_command_blocked("format c:"));
        assert!(!tool.is_command_blocked("echo hello"));
        assert!(!tool.is_command_blocked("dir"));
        assert!(!tool.is_command_blocked("cargo build"));
    }

    #[test]
    fn test_blocked_case_insensitive() {
        let tool = ShellTool::new(".");
        assert!(tool.is_command_blocked("RM -RF /"));
        assert!(tool.is_command_blocked("Shutdown -r"));
    }

    #[tokio::test]
    async fn test_simple_command() {
        let tool = ShellTool::new(".");
        let params = serde_json::json!({"command": "echo hello"});
        let result = tool.execute(&params).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_blocked_command_rejected() {
        let tool = ShellTool::new(".");
        let params = serde_json::json!({"command": "rm -rf /"});
        let result = tool.execute(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_command() {
        let tool = ShellTool::new(".");
        let params = serde_json::json!({});
        let result = tool.execute(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_command() {
        let tool = ShellTool::new(".");
        let params = serde_json::json!({"command": ""});
        let result = tool.execute(&params).await;
        // Empty command should run and succeed (exit code 0)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unicode_command() {
        let tool = ShellTool::new(".");
        let params = serde_json::json!({"command": "echo こんにちは"});
        let result = tool.execute(&params).await;
        assert!(result.is_ok());
    }
}
