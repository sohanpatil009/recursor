use agent_core::types::*;

#[test]
fn test_task_creation() {
    let task = Task {
        id: TaskId("test-1".to_string()),
        title: "Test task".to_string(),
        description: "Do something".to_string(),
        priority: Priority::Normal,
        status: TaskStatus::Pending,
        max_retries: 3,
        timeout_seconds: 60,
        criteria: vec![Criterion::ToolExecuted],
        context: Context::new("."),
        created_at: chrono::Utc::now(),
        parent_task: None,
        subtasks: Vec::new(),
        tags: Vec::new(),
    };
    assert_eq!(task.id.0, "test-1");
    assert!(matches!(task.status, TaskStatus::Pending));
}

#[test]
fn test_confidence_score() {
    let score = ConfidenceScore::new(1.0, 0.8, 1.0, 0.0);
    assert!(score.is_pass(0.5));
    assert!(!score.is_pass(0.95));
    assert!(score.overall > 0.0);
}

#[test]
fn test_verdict() {
    let score = ConfidenceScore::new(1.0, 1.0, 1.0, 1.0);
    let pass = Verdict::pass(score.clone());
    assert!(pass.passed);

    let fail = Verdict::fail(score, vec!["something went wrong".to_string()]);
    assert!(!fail.passed);
    assert_eq!(fail.issues.len(), 1);
}

#[test]
fn test_retry_policy() {
    let policy = RetryPolicy::default_executor();
    assert_eq!(policy.max_retries, 3);
    let d1 = policy.delay_for_attempt(0);
    let d2 = policy.delay_for_attempt(1);
    assert!(d2 > d1); // exponential backoff
}

#[test]
fn test_serialization_roundtrip() {
    let task = Task {
        id: TaskId("roundtrip".to_string()),
        title: "Serialize me".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: TaskStatus::InProgress,
        max_retries: 2,
        timeout_seconds: 30,
        criteria: vec![],
        context: Context::new("/tmp"),
        created_at: chrono::Utc::now(),
        parent_task: None,
        subtasks: Vec::new(),
        tags: vec!["test".to_string()],
    };

    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id.0, "roundtrip");
    assert!(matches!(deserialized.priority, Priority::High));
}
