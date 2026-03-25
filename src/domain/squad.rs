use std::sync::Arc;

use super::agent::{AgentBehavior, AgentId};
use super::task::Task;

pub struct Squad {
    pub agents: Vec<Arc<dyn AgentBehavior>>,
    pub tasks: Vec<Task>,
}

impl Squad {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn add_agent(&mut self, agent: Arc<dyn AgentBehavior>) {
        self.agents.push(agent);
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn agent_for_role(&self, role: &str) -> Option<&Arc<dyn AgentBehavior>> {
        self.agents.iter().find(|a| a.role() == role)
    }

    pub fn agent_by_id(&self, id: &AgentId) -> Option<&Arc<dyn AgentBehavior>> {
        self.agents.iter().find(|a| a.id() == id)
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_squad() {
        let squad = Squad::new();
        assert_eq!(squad.agent_count(), 0);
        assert_eq!(squad.task_count(), 0);
        assert!(squad.agent_for_role("dev").is_none());
    }
}
