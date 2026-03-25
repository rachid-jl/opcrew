use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub assigned_to: Option<AgentId>,
    pub assigned_role: String,
    pub depends_on: Vec<TaskId>,
    pub priority: u8,
    pub status: TaskStatus,
    pub result: Option<String>,
}

impl Task {
    pub fn new(title: String, description: String, assigned_role: String) -> Self {
        Self {
            id: TaskId::new(),
            title,
            description,
            assigned_to: None,
            assigned_role,
            depends_on: Vec::new(),
            priority: 1,
            status: TaskStatus::Pending,
            result: None,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_depends_on(mut self, deps: Vec<TaskId>) -> Self {
        self.depends_on = deps;
        self
    }

    pub fn is_ready(&self, completed_tasks: &[TaskId]) -> bool {
        self.status == TaskStatus::Pending
            && self
                .depends_on
                .iter()
                .all(|dep| completed_tasks.contains(dep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_is_ready_when_deps_met() {
        let dep_id = TaskId::new();
        let task = Task::new("test".into(), "desc".into(), "dev".into())
            .with_depends_on(vec![dep_id.clone()]);

        assert!(!task.is_ready(&[]));
        assert!(task.is_ready(&[dep_id]));
    }

    #[test]
    fn task_not_ready_if_not_pending() {
        let mut task = Task::new("test".into(), "desc".into(), "dev".into());
        task.status = TaskStatus::InProgress;
        assert!(!task.is_ready(&[]));
    }
}
