use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Claude API error: {message} (status: {status_code})")]
    ApiError { status_code: u16, message: String },

    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Planning error: {0}")]
    PlanningError(String),

    #[error("Agent execution error: {agent_role} failed: {message}")]
    ExecutionError { agent_role: String, message: String },

    #[error("Synthesis error: {0}")]
    SynthesisError(String),

    #[error("Token budget exceeded for agent {agent_role}: limit {limit}")]
    BudgetExceeded { agent_role: String, limit: u32 },

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Remote connection failed to {host}: {reason}")]
    RemoteConnectionFailed { host: String, reason: String },

    #[error("Tool not found: {name}. Available: {available}")]
    ToolNotFound { name: String, available: String },

    #[error("Tool execution error: {tool} — {message}")]
    ToolExecutionError { tool: String, message: String },

    #[error("Tool timeout: {tool} exceeded {timeout_secs}s")]
    ToolTimeout { tool: String, timeout_secs: u64 },

    #[error("Guardian blocked: {reason}")]
    GuardianBlocked { reason: String },

    #[error("Path denied: {path} is in the system denylist")]
    PathDenied { path: String },

    #[error("Shell composition detected: {command}")]
    ShellComposition { command: String },

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Audit log error: {0}")]
    AuditError(String),

    #[error("Circuit breaker open: {service}")]
    CircuitBreakerOpen { service: String },

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Verification error: {0}")]
    VerificationError(String),

    #[error("Infrastructure error: {0}")]
    InfraError(String),

    #[error("Watch error: {0}")]
    WatchError(String),
}

pub type Result<T> = std::result::Result<T, AgentError>;
