use agent_core::error::VerificationError;
use agent_core::types::{ConfidenceScore, Criterion, StepResult, Verdict};
use agent_core::verifier::{VerificationGate, Verifier};
use async_trait::async_trait;
use tracing::warn;

pub struct VerifierPipeline {
    gates: Vec<Box<dyn VerificationGate>>,
}

impl VerifierPipeline {
    pub fn new() -> Self {
        Self { gates: Vec::new() }
    }

    pub fn add_gate(&mut self, gate: Box<dyn VerificationGate>) {
        self.gates.push(gate);
    }
}

impl Default for VerifierPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Verifier for VerifierPipeline {
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError> {
        let mut total_confidence = 0.0_f64;
        let mut passed_count = 0_u32;
        let mut all_issues = Vec::new();

        for gate in &self.gates {
            let verdict = match gate.verify(result, criteria).await {
                Ok(v) => v,
                Err(e) => {
                    warn!("Gate '{}' returned error, skipping: {}", gate.name(), e);
                    continue;
                }
            };
            if verdict.passed {
                total_confidence += verdict.confidence.overall;
                passed_count += 1;
            } else {
                all_issues.extend(verdict.issues);
            }
        }

        if all_issues.is_empty() {
            let avg_confidence = if passed_count > 0 {
                total_confidence / passed_count as f64
            } else {
                0.5
            };
            Ok(Verdict::pass(ConfidenceScore::new(
                avg_confidence,
                avg_confidence,
                avg_confidence,
                avg_confidence,
            )))
        } else {
            Ok(Verdict::fail(ConfidenceScore::new(0.0, 0.0, 0.0, 0.0), all_issues))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::types::{StepId, StepResult, ToolCall, ToolCallId, ToolOutput, ToolResult};

    struct PassGate;
    struct FailGate;

    #[async_trait]
    impl VerificationGate for PassGate {
        fn name(&self) -> &str {
            "pass_gate"
        }
        async fn verify(&self, _: &StepResult, _: &[Criterion]) -> Result<Verdict, VerificationError> {
            Ok(Verdict::pass(ConfidenceScore::new(1.0, 1.0, 1.0, 0.0)))
        }
    }

    #[async_trait]
    impl VerificationGate for FailGate {
        fn name(&self) -> &str {
            "fail_gate"
        }
        async fn verify(&self, _: &StepResult, _: &[Criterion]) -> Result<Verdict, VerificationError> {
            Ok(Verdict::fail(ConfidenceScore::zero(), vec!["gate failed".to_string()]))
        }
    }

    struct ErrorGate;

    #[async_trait]
    impl VerificationGate for ErrorGate {
        fn name(&self) -> &str {
            "error_gate"
        }
        async fn verify(&self, _: &StepResult, _: &[Criterion]) -> Result<Verdict, VerificationError> {
            Err(VerificationError::VerifierError("something broke".to_string()))
        }
    }

    fn sample_result() -> StepResult {
        StepResult {
            step_id: StepId("s1".to_string()),
            output: "ok".to_string(),
            tool_results: vec![ToolResult {
                tool_name: "test".to_string(),
                call: ToolCall {
                    id: ToolCallId("c1".to_string()),
                    name: "test".to_string(),
                    params: serde_json::json!({}),
                },
                output: ToolOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    truncated: false,
                    data: None,
                },
                duration_ms: 0,
            }],
            success: true,
            duration_ms: 0,
        }
    }

    #[tokio::test]
    async fn test_pipeline_all_pass() {
        let mut p = VerifierPipeline::new();
        p.add_gate(Box::new(PassGate));
        p.add_gate(Box::new(PassGate));
        let v = p.verify(&sample_result(), &[]).await.unwrap();
        assert!(v.passed);
    }

    #[tokio::test]
    async fn test_pipeline_any_fail() {
        let mut p = VerifierPipeline::new();
        p.add_gate(Box::new(PassGate));
        p.add_gate(Box::new(FailGate));
        p.add_gate(Box::new(PassGate));
        let v = p.verify(&sample_result(), &[]).await.unwrap();
        assert!(!v.passed);
    }

    #[tokio::test]
    async fn test_pipeline_gate_error_skips() {
        let mut p = VerifierPipeline::new();
        p.add_gate(Box::new(PassGate));
        p.add_gate(Box::new(ErrorGate));
        p.add_gate(Box::new(PassGate));
        let v = p.verify(&sample_result(), &[]).await.unwrap();
        assert!(v.passed); // ErrorGate is skipped
    }

    #[tokio::test]
    async fn test_pipeline_empty() {
        let p = VerifierPipeline::new();
        let v = p.verify(&sample_result(), &[]).await.unwrap();
        assert!(v.passed);
    }

    #[tokio::test]
    async fn test_pipeline_all_fail() {
        let mut p = VerifierPipeline::new();
        p.add_gate(Box::new(FailGate));
        p.add_gate(Box::new(FailGate));
        let v = p.verify(&sample_result(), &[]).await.unwrap();
        assert!(!v.passed);
    }
}
