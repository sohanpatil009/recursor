use agent_core::error::AgentError;
use agent_core::event::{AgentEvent, UserCommand};
use agent_core::types::{Task, TaskId, TaskStatus};
use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;

pub struct TaskScheduler {
    tasks: HashMap<TaskId, Task>,
    queue: VecDeque<TaskId>,
    event_bus: broadcast::Sender<AgentEvent>,
}

impl TaskScheduler {
    pub fn new(event_bus: broadcast::Sender<AgentEvent>) -> Self {
        Self {
            tasks: HashMap::new(),
            queue: VecDeque::new(),
            event_bus,
        }
    }

    pub fn submit(&mut self, task: Task) -> Result<(), AgentError> {
        let id = task.id.clone();
        let _ = self
            .event_bus
            .send(AgentEvent::TaskCreated(id.clone(), Box::new(task.clone())));
        self.tasks.insert(id.clone(), task);
        self.queue.push_back(id);
        Ok(())
    }

    pub fn pop_next(&mut self) -> Option<Task> {
        while let Some(id) = self.queue.pop_front() {
            if let Some(task) = self.tasks.get_mut(&id) {
                task.status = TaskStatus::InProgress;
                let _ = self.event_bus.send(AgentEvent::TaskStarted(id.clone()));
                return Some(task.clone());
            }
        }
        None
    }

    pub fn complete(&mut self, id: &TaskId, output: agent_core::types::FinalOutput) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.status = TaskStatus::Completed(Box::new(output.clone()));
            let _ = self
                .event_bus
                .send(AgentEvent::TaskCompleted(id.clone(), Box::new(output)));
        }
    }

    pub fn fail(&mut self, id: &TaskId, error: AgentError) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.status = TaskStatus::Failed(Box::new(error.clone()));
            let _ = self.event_bus.send(AgentEvent::TaskFailed(id.clone(), error));
        }
    }

    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    pub fn handle_command(&mut self, command: UserCommand) {
        match command {
            UserCommand::CancelTask(id) => {
                self.tasks.remove(&id);
                let _ = self
                    .event_bus
                    .send(AgentEvent::TaskCancelled(id, "user requested".to_string()));
            }
            UserCommand::PauseTask(id) => {
                if let Some(task) = self.tasks.get_mut(&id) {
                    task.status = TaskStatus::Paused("user requested".to_string());
                    let _ = self
                        .event_bus
                        .send(AgentEvent::TaskPaused(id, "user requested".to_string()));
                }
            }
            _ => {}
        }
    }
}
