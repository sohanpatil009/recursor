use agent_core::event::AgentEvent;
use agent_core::types::{ApprovalRequest, Task, TaskId, TaskStatus, Verdict};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TaskInputProps {
    pub on_submit: EventHandler<String>,
    pub disabled: bool,
}

pub fn TaskInput(props: TaskInputProps) -> Element {
    let mut text = use_signal(String::new);

    rsx! {
        div { class: "task-input",
            textarea {
                placeholder: "Describe what you want the agent to do...",
                value: "{text}",
                disabled: props.disabled,
                oninput: move |e| text.set(e.value()),
                rows: 3,
            }
            button {
                disabled: props.disabled || text.read().is_empty(),
                onclick: move |_| {
                    let t = text.read().clone();
                    if !t.is_empty() {
                        props.on_submit.call(t);
                        text.set(String::new());
                    }
                },
                "Submit Task"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TaskListProps {
    pub tasks: Vec<Task>,
    pub on_select: EventHandler<TaskId>,
}

pub fn TaskList(props: TaskListProps) -> Element {
    let tasks = &props.tasks;
    let list_content = if tasks.is_empty() {
        rsx! { div { class: "empty-state", "No tasks yet. Submit your first task above." } }
    } else {
        let rows: Vec<Element> = tasks
            .iter()
            .map(|task| {
                let id = task.id.clone();
                let title = task.title.clone();
                let priority = format!("{:?}", task.priority);
                let time_str = task.created_at.format("%H:%M").to_string();
                let status = task.status.clone();
                let on_select = props.on_select;
                rsx! {
                    tr {
                        onclick: move |_| { on_select.call(id.clone()); },
                        td { TaskStatusBadge { status } }
                        td { "{title}" }
                        td { "{priority}" }
                        td { "{time_str}" }
                    }
                }
            })
            .collect();

        rsx! {
            table {
                thead {
                    tr {
                        th { "Status" }
                        th { "Title" }
                        th { "Priority" }
                        th { "Created" }
                    }
                }
                tbody {
                    {rows.into_iter()}
                }
            }
        }
    };

    rsx! {
        div { class: "task-list",
            h2 { "Tasks" }
            {list_content}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TaskStatusBadgeProps {
    pub status: TaskStatus,
}

pub fn TaskStatusBadge(props: TaskStatusBadgeProps) -> Element {
    let (label, class) = match &props.status {
        TaskStatus::Pending => ("Pending", "badge-pending"),
        TaskStatus::Queued => ("Queued", "badge-queued"),
        TaskStatus::InProgress => ("Running", "badge-running"),
        TaskStatus::Paused(_) => ("Paused", "badge-paused"),
        TaskStatus::AwaitingApproval(_) => ("Approval", "badge-approval"),
        TaskStatus::Completed(_) => ("Done", "badge-done"),
        TaskStatus::Failed(_) => ("Failed", "badge-failed"),
        TaskStatus::Cancelled(_) => ("Cancelled", "badge-cancelled"),
    };

    rsx! {
        span { class: "badge {class}", "{label}" }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AgentLogProps {
    pub events: Vec<AgentEvent>,
    pub max_height: Option<String>,
}

fn render_event(evt: &AgentEvent) -> Element {
    match evt {
        AgentEvent::TaskCreated(_, _) => rsx! { span { class: "log-info", "📋 Task created" } },
        AgentEvent::TaskStarted(_) => rsx! { span { class: "log-info", "🔄 Task started" } },
        AgentEvent::TaskCompleted(_, _) => rsx! { span { class: "log-success", "✅ Task completed" } },
        AgentEvent::TaskFailed(_, _) => rsx! { span { class: "log-error", "❌ Task failed" } },
        AgentEvent::ThoughtComplete(_, _) => rsx! { span { class: "log-info", "💭 Thought generated" } },
        AgentEvent::PlanCreated(_, _) => rsx! { span { class: "log-info", "📝 Plan created" } },
        AgentEvent::StepStarted(_, _) => rsx! { span { class: "log-info", "⚙️ Step started" } },
        AgentEvent::StepCompleted(_, _, _) => rsx! { span { class: "log-success", "✅ Step completed" } },
        AgentEvent::StepFailed(_, _, _) => rsx! { span { class: "log-error", "❌ Step failed" } },
        AgentEvent::ToolCalled(_, _) => rsx! { span { class: "log-info", "🔧 Tool called" } },
        AgentEvent::ToolResult(_, _, _) => rsx! { span { class: "log-info", "🔧 Tool result" } },
        AgentEvent::VerdictReached(_, _) => rsx! { span { class: "log-info", "⚖️ Verdict reached" } },
        AgentEvent::ReflectionGenerated(_, _) => rsx! { span { class: "log-info", "🔄 Reflection generated" } },
        AgentEvent::TaskCancelled(_, _) => rsx! { span { class: "log-error", "🚫 Task cancelled" } },
        AgentEvent::TaskPaused(_, _) => rsx! { span { class: "log-warn", "⏸ Task paused" } },
        _ => rsx! { span { class: "log-info", "ℹ️ Event" } },
    }
}

pub fn AgentLog(props: AgentLogProps) -> Element {
    let style_str = props
        .max_height
        .as_ref()
        .map(|h| format!("max-height: {h}; overflow-y: auto"))
        .unwrap_or_default();
    let inner = if props.events.is_empty() {
        rsx! { div { class: "empty-state", "Waiting for agent activity..." } }
    } else {
        rsx! {
            for evt in props.events.iter().rev() {
                div { class: "log-entry", {render_event(evt)} }
            }
        }
    };

    rsx! {
        div { class: "agent-log", style: "{style_str}",
            h3 { "Agent Log" }
            {inner}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TaskDetailProps {
    pub task: Option<Task>,
}

fn render_task_detail(task: &Task) -> Element {
    let created = task.created_at.format("%Y-%m-%d %H:%M").to_string();
    let output_section = match &task.status {
        TaskStatus::Completed(ref output) => {
            rsx! {
                div { class: "output-section",
                    h3 { "Output" }
                    pre { "{output.output}" }
                    VerdictDisplay { verdict: output.verdict.clone() }
                }
            }
        }
        _ => rsx! { div {} },
    };

    rsx! {
        div { class: "task-detail",
            h2 { "{task.title}" }
            div { class: "detail-grid",
                div { class: "detail-item",
                    label { "Status" }
                    TaskStatusBadge { status: task.status.clone() }
                }
                div { class: "detail-item",
                    label { "Priority" }
                    span { "{task.priority:?}" }
                }
                div { class: "detail-item",
                    label { "Created" }
                    span { "{created}" }
                }
                div { class: "detail-item",
                    label { "Max Retries" }
                    span { "{task.max_retries}" }
                }
                div { class: "detail-item",
                    label { "Timeout" }
                    span { "{task.timeout_seconds}s" }
                }
            }
            {output_section}
        }
    }
}

pub fn TaskDetail(props: TaskDetailProps) -> Element {
    match &props.task {
        Some(task) => render_task_detail(task),
        None => rsx! {
            div { class: "task-detail empty",
                p { "Select a task to view details" }
            }
        },
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct VerdictDisplayProps {
    pub verdict: Verdict,
}

pub fn VerdictDisplay(props: VerdictDisplayProps) -> Element {
    let passed_class = if props.verdict.passed {
        "verdict-pass"
    } else {
        "verdict-fail"
    };
    let _confidence_pct = (props.verdict.confidence.overall * 100.0) as u32;

    rsx! {
        div { class: "verdict-display {passed_class}",
            h3 {
                if props.verdict.passed { "✅ Passed" } else { "❌ Failed" }
            }
            ConfidenceGauge { value: props.verdict.confidence.overall, label: "Overall" }
            if !props.verdict.issues.is_empty() {
                div { class: "issues",
                    h4 { "Issues" }
                    ul {
                        for issue in &props.verdict.issues {
                            li { "{issue}" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ConfidenceGaugeProps {
    pub value: f64,
    pub label: String,
}

pub fn ConfidenceGauge(props: ConfidenceGaugeProps) -> Element {
    let pct = (props.value * 100.0) as u32;
    let color = if props.value >= 0.7 {
        "green"
    } else if props.value >= 0.4 {
        "orange"
    } else {
        "red"
    };

    rsx! {
        div { class: "confidence-gauge",
            div { class: "gauge-label", "{props.label}: {pct}%" }
            div { class: "gauge-track",
                div {
                    class: "gauge-fill {color}",
                    style: "width: {pct}%",
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ApprovalDialogProps {
    pub request: Option<ApprovalRequest>,
    pub on_approve: EventHandler<()>,
    pub on_reject: EventHandler<()>,
    pub visible: bool,
}

pub fn ApprovalDialog(props: ApprovalDialogProps) -> Element {
    if !props.visible || props.request.is_none() {
        return rsx! { div {} };
    }

    let req = props.request.as_ref().unwrap();
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal",
                h2 { "Approval Required" }
                p { "{req.description}" }
                div { class: "modal-actions",
                    button {
                        class: "btn-approve",
                        onclick: move |_| props.on_approve.call(()),
                        "Approve"
                    }
                    button {
                        class: "btn-reject",
                        onclick: move |_| props.on_reject.call(()),
                        "Reject"
                    }
                }
            }
        }
    }
}

pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "loading-spinner",
            div { class: "spinner" }
            p { "Processing..." }
        }
    }
}

pub fn ErrorDisplay(error: &Option<String>) -> Element {
    match error {
        Some(msg) => rsx! {
            div { class: "error-toast",
                span { "❌ {msg}" }
            }
        },
        None => rsx! { div {} },
    }
}

pub fn EmptyState(message: &str) -> Element {
    rsx! {
        div { class: "empty-state",
            p { "{message}" }
        }
    }
}
