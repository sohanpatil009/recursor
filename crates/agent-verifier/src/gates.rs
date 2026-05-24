use agent_core::error::VerificationError;
use agent_core::types::{ConfidenceScore, Criterion, StepResult, Verdict};
use agent_core::verifier::VerificationGate;
use async_trait::async_trait;

/// Gate 1: Structural validation — JSON schema, required fields, regex
pub struct StructuralVerifier;

#[async_trait]
impl VerificationGate for StructuralVerifier {
    fn name(&self) -> &str {
        "structural"
    }

    #[allow(clippy::collapsible_match)]
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError> {
        let mut issues = Vec::new();

        for criterion in criteria {
            match criterion {
                Criterion::RequiredFields(fields) => {
                    for field in fields {
                        if !result.output.contains(field) {
                            issues.push(format!("Missing required field: {}", field));
                        }
                    }
                }
                Criterion::OutputBounds { max_length } => {
                    if result.output.len() > *max_length {
                        issues.push(format!(
                            "Output too long: {} > {} chars",
                            result.output.len(),
                            max_length
                        ));
                    }
                }
                Criterion::RegexPattern(pattern) => {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if !re.is_match(&result.output) {
                            issues.push(format!("Output does not match pattern: {}", pattern));
                        }
                    }
                }
                _ => {}
            }
        }

        if issues.is_empty() {
            Ok(Verdict::pass(ConfidenceScore::new(1.0, 0.0, 0.0, 0.0)))
        } else {
            Ok(Verdict::fail(ConfidenceScore::new(0.0, 0.0, 0.0, 0.0), issues))
        }
    }
}

/// Gate 2: Deterministic checks — exit code, tool executed, output bounds
pub struct DeterministicVerifier;

#[async_trait]
impl VerificationGate for DeterministicVerifier {
    fn name(&self) -> &str {
        "deterministic"
    }

    #[allow(clippy::collapsible_match)]
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError> {
        let mut issues = Vec::new();

        for criterion in criteria {
            match criterion {
                Criterion::ExitCode(expected) => {
                    for tr in &result.tool_results {
                        if let Some(code) = tr.output.exit_code {
                            if code != *expected {
                                issues.push(format!(
                                    "Tool '{}' exit code {} != expected {}",
                                    tr.tool_name, code, expected
                                ));
                            }
                        }
                    }
                }
                Criterion::ToolExecuted => {
                    if result.tool_results.is_empty() {
                        issues.push("No tools were executed".to_string());
                    }
                }
                _ => {}
            }
        }

        if issues.is_empty() {
            Ok(Verdict::pass(ConfidenceScore::new(0.0, 0.0, 1.0, 0.0)))
        } else {
            Ok(Verdict::fail(ConfidenceScore::new(0.0, 0.0, 0.0, 0.0), issues))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::types::{StepId, StepResult};

    fn make_result(output: &str, success: bool, exit_code: Option<i32>) -> StepResult {
        StepResult {
            step_id: StepId("test".to_string()),
            output: output.to_string(),
            tool_results: vec![agent_core::types::ToolResult {
                tool_name: "test".to_string(),
                call: agent_core::types::ToolCall {
                    id: agent_core::types::ToolCallId("call-1".to_string()),
                    name: "test".to_string(),
                    params: serde_json::json!({}),
                },
                output: agent_core::types::ToolOutput {
                    success,
                    exit_code,
                    stdout: output.to_string(),
                    stderr: String::new(),
                    truncated: false,
                    data: None,
                },
                duration_ms: 0,
            }],
            success,
            duration_ms: 0,
        }
    }

    #[tokio::test]
    async fn test_structural_required_fields() {
        let verifier = StructuralVerifier;
        let result = make_result("name: John\nage: 30", true, Some(0));
        let criteria = vec![Criterion::RequiredFields(vec!["name".to_string()])];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_structural_missing_field() {
        let verifier = StructuralVerifier;
        let result = make_result("name: John", true, Some(0));
        let criteria = vec![Criterion::RequiredFields(vec!["email".to_string()])];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(!verdict.passed);
    }

    #[tokio::test]
    async fn test_structural_output_bounds() {
        let verifier = StructuralVerifier;
        let result = make_result("short", true, Some(0));
        let criteria = vec![Criterion::OutputBounds { max_length: 100 }];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_structural_output_too_long() {
        let verifier = StructuralVerifier;
        let long = "a".repeat(200);
        let result = make_result(&long, true, Some(0));
        let criteria = vec![Criterion::OutputBounds { max_length: 50 }];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(!verdict.passed);
    }

    #[tokio::test]
    async fn test_structural_regex_match() {
        let verifier = StructuralVerifier;
        let result = make_result("hello@example.com", true, Some(0));
        let criteria = vec![Criterion::RegexPattern(r"^[\w.+-]+@[\w-]+\.[\w.]+$".to_string())];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_structural_no_relevant_criteria() {
        let verifier = StructuralVerifier;
        let result = make_result("any output", true, Some(0));
        // Empty criteria or non-matching criteria should pass
        let criteria: Vec<Criterion> = vec![];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_deterministic_exit_code() {
        let verifier = DeterministicVerifier;
        let result = make_result("ok", true, Some(0));
        let criteria = vec![Criterion::ExitCode(0)];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_deterministic_bad_exit_code() {
        let verifier = DeterministicVerifier;
        let result = make_result("error", false, Some(1));
        let criteria = vec![Criterion::ExitCode(0)];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(!verdict.passed);
    }

    #[tokio::test]
    async fn test_deterministic_tool_executed() {
        let verifier = DeterministicVerifier;
        let result = make_result("done", true, Some(0));
        let criteria = vec![Criterion::ToolExecuted];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(verdict.passed);
    }

    #[tokio::test]
    async fn test_deterministic_no_tool_executed() {
        let verifier = DeterministicVerifier;
        let result = StepResult {
            step_id: StepId("empty".to_string()),
            output: String::new(),
            tool_results: vec![],
            success: false,
            duration_ms: 0,
        };
        let criteria = vec![Criterion::ToolExecuted];
        let verdict = verifier.verify(&result, &criteria).await.unwrap();
        assert!(!verdict.passed);
    }
}
