use agent_core::error::AgentError;
use agent_core::llm::{LLMProvider, LLMRequest, LLMResponse, ResponseFormat};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    system_instruction: Option<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    temperature: f64,
    max_output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(default)]
    prompt_feedback: Option<PromptFeedback>,
}

#[derive(Deserialize)]
struct Candidate {
    content: GeminiResponseContent,
    #[serde(default)]
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct PromptFeedback {
    #[serde(default)]
    block_reason: Option<String>,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model: if model.is_empty() {
                "gemini-3.5-flash".to_string()
            } else {
                model
            },
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, AgentError> {
        let use_json = matches!(request.response_format, Some(ResponseFormat::Json { .. }));

        let contents = vec![GeminiContent {
            parts: vec![GeminiPart {
                text: request.user_prompt,
            }],
        }];

        let system_instruction = if request.system_prompt.is_empty() {
            None
        } else {
            Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: request.system_prompt,
                }],
            })
        };

        let model = request.model.unwrap_or_else(|| self.model.clone());

        let req_body = GeminiRequest {
            contents,
            system_instruction,
            generation_config: GeminiGenerationConfig {
                temperature: request.temperature.unwrap_or(0.7),
                max_output_tokens: request.max_tokens.unwrap_or(4096),
                response_mime_type: if use_json {
                    Some("application/json".to_string())
                } else {
                    None
                },
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| AgentError::LLMError(format!("Gemini request failed: {}", e)))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AgentError::LLMError(format!("Gemini read body failed: {}", e)))?;

        if !status.is_success() {
            return Err(AgentError::LLMError(format!("Gemini API error ({}): {}", status, text)));
        }

        let body: GeminiResponse = serde_json::from_str(&text)
            .map_err(|e| AgentError::LLMError(format!("Gemini parse failed: {} — body: {}", e, text)))?;

        if let Some(feedback) = &body.prompt_feedback {
            if let Some(reason) = &feedback.block_reason {
                return Err(AgentError::LLMError(format!("Gemini request blocked: {}", reason)));
            }
        }

        let content = body
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            tokens_used: 0,
            model,
        })
    }
}
