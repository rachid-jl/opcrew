use std::collections::HashMap;
use std::sync::Arc;

use super::traits::Tool;
use crate::error::{AgentError, Result};

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Result<&Arc<dyn Tool>> {
        self.tools.get(name).ok_or_else(|| {
            let available = self
                .tools
                .keys()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            AgentError::ToolNotFound {
                name: name.to_string(),
                available,
            }
        })
    }

    pub fn available_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    pub fn tool_descriptions(&self) -> Vec<(String, String)> {
        self.tools
            .values()
            .map(|t| (t.name().to_string(), t.description().to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::shell::ShellTool;
    use crate::tools::target::TargetHost;

    #[test]
    fn register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(ShellTool::new(TargetHost::Local)));

        assert!(registry.get("shell").is_ok());
        assert_eq!(registry.available_tools().len(), 1);
    }

    #[test]
    fn tool_not_found_lists_available() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(ShellTool::new(TargetHost::Local)));

        let result = registry.get("nonexistent");
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("nonexistent"));
        assert!(msg.contains("shell"));
    }
}
