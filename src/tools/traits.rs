use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub tool_name: String,
    pub action: String,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl ToolResult {
    pub fn ok(output: String, duration_ms: u64) -> Self {
        Self {
            success: true,
            output,
            error: None,
            duration_ms,
        }
    }

    pub fn err(error: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
            duration_ms,
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn is_remote_capable(&self) -> bool;
    async fn execute(&self, params: &ToolParams, timeout: Duration) -> Result<ToolResult>;
}
