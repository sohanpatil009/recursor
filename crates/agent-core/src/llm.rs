use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    Text,
    Json { schema: Option<serde_json::Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tokens_used: usize,
    pub model: String,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, AgentError>;

    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }
}

/// Helper function to generate structured output from an LLM provider.
/// Not part of the trait to keep it dyn-compatible.
pub async fn generate_structured<T: serde::de::DeserializeOwned>(
    provider: &dyn LLMProvider,
    request: LLMRequest,
) -> Result<T, AgentError> {
    let req = LLMRequest {
        response_format: Some(ResponseFormat::Json { schema: None }),
        ..request
    };
    let response = provider.generate(req).await?;
    serde_json::from_str(&response.content)
        .map_err(|e| AgentError::LLMError(format!("Failed to parse structured output: {}", e)))
}
