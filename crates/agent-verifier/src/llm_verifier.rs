use agent_core::error::VerificationError;
use agent_core::llm::{LLMProvider, LLMRequest};
use agent_core::types::{ConfidenceScore, Criterion, StepResult, Verdict};
use agent_core::verifier::VerificationGate;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

/// Gate 3: LLM-Based Verification — uses a critic LLM to evaluate output quality.
pub struct LLMVerifier {
    llm: Arc<dyn LLMProvider>,
}

#[derive(Deserialize)]
struct CriticEvaluation {
    passed: bool,
    confidence: f64,
    evidence: Vec<String>,
    issues: Vec<String>,
    suggestions: Vec<String>,
}

impl LLMVerifier {
    pub fn new(llm: Arc<dyn LLMProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl VerificationGate for LLMVerifier {
    fn name(&self) -> &str {
        "llm_critic"
    }

    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError> {
        let criteria_str: Vec<String> = criteria.iter().map(|c| c.to_string()).collect();

        let prompt = format!(
            r#"You are a critic evaluating an AI agent's work.

## Execution Output
{}

## Evaluation Criteria
{}

Evaluate whether the result satisfies ALL criteria.
Provide:
1. A pass/fail decision
2. A confidence score (0.0-1.0) for your evaluation
3. Specific evidence from the result that supports your decision
4. Any issues found
5. Suggestions for improvement if failed

Respond in JSON with keys: passed (bool), confidence (f64), evidence (array of strings), issues (array of strings), suggestions (array of strings)."#,
            result.output,
            criteria_str.join("\n")
        );

        let request = LLMRequest {
            system_prompt: "You are a strict critic evaluating AI agent outputs. Be thorough and specific.".to_string(),
            user_prompt: prompt,
            model: None,
            temperature: Some(0.2),
            max_tokens: Some(1000),
            response_format: Some(agent_core::llm::ResponseFormat::Json { schema: None }),
        };

        let evaluation: CriticEvaluation = agent_core::llm::generate_structured(&*self.llm, request)
            .await
            .map_err(|e| VerificationError::VerifierError(e.to_string()))?;

        let confidence = if evaluation.passed {
            ConfidenceScore::new(1.0, evaluation.confidence, 1.0, 0.0)
        } else {
            ConfidenceScore::new(1.0, evaluation.confidence, 0.0, 0.0)
        };

        Ok(Verdict {
            passed: evaluation.passed,
            confidence,
            evidence: evaluation.evidence,
            issues: evaluation.issues,
            suggestions: evaluation.suggestions,
        })
    }
}
