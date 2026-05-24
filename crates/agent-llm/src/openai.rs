use agent_core::error::AgentError;
use agent_core::llm::{LLMProvider, LLMRequest, LLMResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormatObj>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormatObj {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    total_tokens: usize,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, AgentError> {
        let mut messages = Vec::new();

        if !request.system_prompt.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: request.system_prompt,
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: request.user_prompt,
        });

        let req_body = ChatCompletionRequest {
            model: request.model.unwrap_or_else(|| self.model.clone()),
            messages,
            temperature: request.temperature.unwrap_or(0.7),
            max_tokens: request.max_tokens.unwrap_or(4096),
            response_format: match &request.response_format {
                Some(agent_core::llm::ResponseFormat::Json { .. }) => Some(ResponseFormatObj {
                    format_type: "json_object".to_string(),
                }),
                _ => None,
            },
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req_body)
            .send()
            .await
            .map_err(|e| AgentError::LLMError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AgentError::LLMError(format!("Read body failed: {}", e)))?;

        if !status.is_success() {
            return Err(AgentError::LLMError(format!("API error ({}): {}", status, text)));
        }

        let body: ChatCompletionResponse = serde_json::from_str(&text)
            .map_err(|e| AgentError::LLMError(format!("Parse failed: {} — body: {}", e, text)))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            tokens_used: body.usage.total_tokens,
            model: req_body.model,
        })
    }
}
