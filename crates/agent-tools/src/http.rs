use agent_core::error::ToolError;
use agent_core::tool::Tool;
use agent_core::types::ToolOutput;
use async_trait::async_trait;
use std::time::Duration;

pub struct HttpTool {
    client: reqwest::Client,
    max_response_size: usize,
    timeout: Duration,
}

impl Default for HttpTool {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
            max_response_size: 5 * 1024 * 1024, // 5MB
            timeout: Duration::from_secs(15),
        }
    }
}

fn is_private_ip(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("localhost")
        || lower.contains("127.0.0.1")
        || lower.contains("10.")
        || lower.contains("172.16.")
        || lower.contains("192.168.")
        || lower.contains("[::1]")
        || lower.contains("169.254.")
}

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http"
    }

    fn description(&self) -> &str {
        "Make HTTP GET and POST requests"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "method": { "type": "string", "enum": ["GET", "POST"] },
                "url": { "type": "string" },
                "body": { "type": "object" },
                "headers": { "type": "object" }
            },
            "required": ["method", "url"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, params: &serde_json::Value) -> Result<ToolOutput, ToolError> {
        let method = params["method"].as_str().unwrap_or("GET");
        let url = params["url"]
            .as_str()
            .ok_or(ToolError::InvalidParams("missing url".into()))?;

        if is_private_ip(url) {
            return Err(ToolError::PermissionDenied("Private IP requests not allowed".into()));
        }

        let mut req = match method {
            "GET" => self.client.get(url),
            "POST" => {
                let body = params.get("body").cloned().unwrap_or(serde_json::Value::Null);
                self.client.post(url).json(&body)
            }
            _ => return Err(ToolError::UnknownAction(method.to_string())),
        };

        if let Some(headers) = params["headers"].as_object() {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    req = req.header(key, val);
                }
            }
        }

        let response = tokio::time::timeout(self.timeout, req.send())
            .await
            .map_err(|_| ToolError::Timeout(url.to_string()))?
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let truncated = body.len() > self.max_response_size;
        let body = if truncated {
            body[..self.max_response_size].to_string()
        } else {
            body
        };

        Ok(ToolOutput {
            success: status < 500,
            exit_code: Some(if status < 500 { 0 } else { 1 }),
            stdout: body,
            stderr: String::new(),
            truncated,
            data: None,
        })
    }
}
