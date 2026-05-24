use agent_core::event::{AgentEvent, UserCommand};
use agent_core::types::*;

#[test]
fn test_agent_event_serialization() {
    let task = Task {
        id: TaskId("task-1".to_string()),
        title: "test".to_string(),
        description: "desc".to_string(),
        priority: Priority::Normal,
        status: TaskStatus::Pending,
        max_retries: 3,
        timeout_seconds: 60,
        criteria: vec![],
        context: Context::new("."),
        created_at: chrono::Utc::now(),
        parent_task: None,
        subtasks: Vec::new(),
        tags: Vec::new(),
    };
    let event = AgentEvent::TaskCreated(TaskId("task-1".to_string()), Box::new(task));
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("TaskCreated"));
}

#[test]
fn test_user_command_dispatch() {
    let task = Task {
        id: TaskId("cmd-test".to_string()),
        title: "cmd".to_string(),
        description: "test".to_string(),
        priority: Priority::Normal,
        status: TaskStatus::Pending,
        max_retries: 3,
        timeout_seconds: 60,
        criteria: vec![],
        context: Context::new("."),
        created_at: chrono::Utc::now(),
        parent_task: None,
        subtasks: Vec::new(),
        tags: Vec::new(),
    };
    let cmd = UserCommand::SubmitTask(Box::new(task));
    match cmd {
        UserCommand::SubmitTask(t) => assert_eq!(t.id.0, "cmd-test"),
        _ => panic!("Wrong variant"),
    }
}
