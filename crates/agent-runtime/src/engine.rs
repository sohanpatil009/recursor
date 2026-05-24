use agent_core::error::AgentError;
use agent_core::event::{AgentEvent, UserCommand};
use agent_core::types::{ApprovalRequest, ConfidenceScore, FinalOutput, Task, TaskId, Verdict};
use tokio::sync::broadcast;

use crate::agent_executor::GenericAgent;
use crate::scheduler::TaskScheduler;
use crate::SqliteMemoryStore;
use std::sync::Arc;

pub struct AgentEngine {
    scheduler: TaskScheduler,
    agent: Arc<GenericAgent>,
    store: Option<Arc<SqliteMemoryStore>>,
    event_bus: broadcast::Sender<AgentEvent>,
    escalation_handler: Option<Box<dyn Fn(ApprovalRequest) -> bool + Send + Sync>>,
}

impl AgentEngine {
    pub fn new(agent: Arc<GenericAgent>, event_bus: broadcast::Sender<AgentEvent>) -> Self {
        let scheduler = TaskScheduler::new(event_bus.clone());
        Self {
            scheduler,
            agent,
            store: None,
            event_bus,
            escalation_handler: None,
        }
    }

    pub fn with_store(mut self, store: Arc<SqliteMemoryStore>) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_escalation_handler(mut self, handler: Box<dyn Fn(ApprovalRequest) -> bool + Send + Sync>) -> Self {
        self.escalation_handler = Some(handler);
        self
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_bus.subscribe()
    }

    pub fn dispatch(&mut self, command: UserCommand) {
        match command {
            UserCommand::SubmitTask(task) => {
                if let Some(ref store) = self.store {
                    let task_clone = (*task).clone();
                    let store = store.clone();
                    tokio::spawn(async move {
                        let _ = store.save_task(&task_clone).await;
                    });
                }
                let _ = self.scheduler.submit(*task);
            }
            UserCommand::CancelTask(id) => {
                self.scheduler.handle_command(UserCommand::CancelTask(id));
            }
            UserCommand::PauseTask(id) => {
                self.scheduler.handle_command(UserCommand::PauseTask(id));
            }
            _ => {}
        }
    }

    pub async fn process_next(&mut self) -> Result<(), AgentError> {
        if let Some(task) = self.scheduler.pop_next() {
            let id = task.id.clone();
            let _ = self.event_bus.send(AgentEvent::TaskStarted(id.clone()));

            let result = self.agent.run(&task).await;

            match result {
                Ok((results, verdicts, all_turns)) => {
                    let all_passed = verdicts.iter().all(|v| v.passed);
                    let confidence = if verdicts.is_empty() {
                        ConfidenceScore::new(1.0, 1.0, 1.0, 1.0)
                    } else {
                        let avg = verdicts.iter().map(|v| v.confidence.overall).sum::<f64>() / verdicts.len() as f64;
                        ConfidenceScore::new(avg, avg, avg, avg)
                    };

                    let total_attempts: usize = all_turns.iter().map(|t| t.len() + 1).sum();
                    let tool_results: Vec<_> = results.iter().flat_map(|r| r.tool_results.clone()).collect();

                    let output = FinalOutput {
                        output: results.iter().map(|r| r.output.clone()).collect::<Vec<_>>().join("\n"),
                        verdict: if all_passed {
                            Verdict::pass(confidence)
                        } else {
                            let issues: Vec<String> = verdicts
                                .iter()
                                .filter(|v| !v.passed)
                                .flat_map(|v| v.issues.clone())
                                .collect();
                            Verdict::fail(ConfidenceScore::zero(), issues)
                        },
                        attempts: total_attempts,
                        tool_results,
                    };

                    if all_passed {
                        let _ = self
                            .event_bus
                            .send(AgentEvent::TaskCompleted(id.clone(), Box::new(output.clone())));
                        self.scheduler.complete(&id, output);
                    } else {
                        let _ = self.event_bus.send(AgentEvent::TaskFailed(
                            id.clone(),
                            AgentError::VerificationError("Step failed verification after all retries".to_string()),
                        ));
                        self.scheduler.complete(&id, output);
                    }

                    if let Some(ref store) = self.store {
                        let store = store.clone();
                        let task_clone = task.clone();
                        tokio::spawn(async move {
                            let _ = store.save_task(&task_clone).await;
                        });
                    }
                }
                Err(err) => {
                    let _ = self.event_bus.send(AgentEvent::TaskFailed(id.clone(), err.clone()));
                    self.scheduler.fail(&id, err);
                    if let Some(ref store) = self.store {
                        let store = store.clone();
                        let task_clone = task.clone();
                        tokio::spawn(async move {
                            let _ = store.save_task(&task_clone).await;
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.scheduler.get_task(id)
    }

    pub fn all_tasks(&self) -> Vec<&Task> {
        self.scheduler.all_tasks()
    }
}
