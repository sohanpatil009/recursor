use agent_core::error::AgentError;
use agent_core::llm::{LLMProvider, LLMRequest};
use agent_core::types::{Reflection, StepResult, Task, Turn, Verdict};
use std::sync::Arc;

pub struct ReflectionLoop {
    llm: Arc<dyn LLMProvider>,
    max_cycles: usize,
    confidence_threshold: f64,
}

impl ReflectionLoop {
    pub fn new(llm: Arc<dyn LLMProvider>) -> Self {
        Self {
            llm,
            max_cycles: 3,
            confidence_threshold: 0.7,
        }
    }

    pub async fn generate_reflection(
        &self,
        task: &Task,
        output: &StepResult,
        verdict: &Verdict,
        history: &[Turn],
    ) -> Result<Reflection, AgentError> {
        let previous: Vec<String> = history.iter().map(|t| t.reflection.root_cause.clone()).collect();

        let prompt = format!(
            "You attempted a task and it failed verification.\n\n## Task\n{}\n\n## Output\n{}\n\n## Failure Reason\n{}\n\n## Previous Attempts\n{}\n\nAnalyze why and provide: root cause, changes needed, what to keep, and confidence next attempt will succeed (0-1). Respond in JSON with keys: root_cause, changes_required (array), keep_same (array), next_attempt_confidence (number).",
            task.description,
            output.output,
            verdict.issues.join("\n"),
            previous.join("\n---\n")
        );

        let request = LLMRequest {
            system_prompt: "You are a debugger analyzing agent failures. Be specific.".to_string(),
            user_prompt: prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(1000),
            response_format: Some(agent_core::llm::ResponseFormat::Json { schema: None }),
        };

        let reflection: Reflection = agent_core::llm::generate_structured(&*self.llm, request).await?;
        Ok(reflection)
    }

    pub fn is_pass(&self, verdict: &Verdict) -> bool {
        verdict.passed && verdict.confidence.overall >= self.confidence_threshold
    }

    pub fn max_cycles(&self) -> usize {
        self.max_cycles
    }

    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }
}

#[derive(Debug, Clone)]
pub struct FailureTracker {
    pub max_retries: usize,
    pub attempts: usize,
    pub last_failure_type: Option<String>,
    pub consecutive_same_failures: usize,
    pub escalated: bool,
}

impl FailureTracker {
    pub fn new(max_retries: usize) -> Self {
        Self {
            max_retries,
            attempts: 0,
            last_failure_type: None,
            consecutive_same_failures: 0,
            escalated: false,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_retries && !self.escalated
    }

    pub fn record_failure(&mut self, failure_type: &str) -> bool {
        self.attempts += 1;
        if self.last_failure_type.as_deref() == Some(failure_type) {
            self.consecutive_same_failures += 1;
        } else {
            self.consecutive_same_failures = 1;
        }
        self.last_failure_type = Some(failure_type.to_string());

        if self.consecutive_same_failures >= 2 {
            self.escalated = true;
        }

        self.attempts >= self.max_retries
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
        self.last_failure_type = None;
        self.consecutive_same_failures = 0;
        self.escalated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_tracker_can_retry() {
        let mut ft = FailureTracker::new(3);
        assert!(ft.can_retry());
        ft.record_failure("timeout");
        assert!(ft.can_retry());
        ft.record_failure("timeout");
        assert!(!ft.can_retry()); // escalated after 2 same-type
    }

    #[test]
    fn test_failure_tracker_max_attempts() {
        let mut ft = FailureTracker::new(2);
        assert!(ft.can_retry());
        ft.record_failure("err1");
        assert!(ft.can_retry());
        ft.record_failure("err2");
        assert!(!ft.can_retry()); // max retries reached
    }

    #[test]
    fn test_failure_tracker_reset() {
        let mut ft = FailureTracker::new(2);
        ft.record_failure("err");
        ft.reset();
        assert!(ft.can_retry());
        assert_eq!(ft.attempts, 0);
    }

    #[test]
    fn test_failure_tracker_different_failures() {
        let mut ft = FailureTracker::new(5);
        ft.record_failure("err1");
        assert!(!ft.escalated);
        ft.record_failure("err2"); // different type, not escalated
        assert!(!ft.escalated);
        ft.record_failure("err2"); // 2nd same type, now escalated
        assert!(ft.escalated);
    }

    #[test]
    fn test_failure_tracker_zero_retries() {
        let ft = FailureTracker::new(0);
        assert!(!ft.can_retry());
    }
}
