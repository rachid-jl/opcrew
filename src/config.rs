use crate::error::{AgentError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub base_url: String,
    pub session_token_budget: u32,
    pub per_agent_token_budget: u32,
    pub per_agent_conversation_cap: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AgentError::ConfigError("ANTHROPIC_API_KEY is required".into()))?;

        if api_key.is_empty() {
            return Err(AgentError::ConfigError(
                "ANTHROPIC_API_KEY must not be empty".into(),
            ));
        }

        Ok(Self {
            api_key,
            model: env_or("CLAUDE_MODEL", "claude-sonnet-4-20250514"),
            max_tokens: env_parse("MAX_TOKENS", 4096),
            base_url: env_or("API_BASE_URL", "https://api.anthropic.com"),
            session_token_budget: env_parse("SESSION_TOKEN_BUDGET", 500_000),
            per_agent_token_budget: env_parse("PER_AGENT_TOKEN_BUDGET", 100_000),
            per_agent_conversation_cap: env_parse("PER_AGENT_CONVERSATION_CAP", 30),
            log_level: env_or("LOG_LEVEL", "info"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_api_key_returns_error() {
        // SAFETY: test-only, single-threaded access to env vars
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn empty_api_key_returns_error() {
        unsafe { std::env::set_var("ANTHROPIC_API_KEY", "") };
        let result = Config::from_env();
        assert!(result.is_err());
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    }

    #[test]
    fn env_parse_defaults() {
        // Test the helper functions directly instead of relying on env var state
        assert_eq!(env_parse::<u32>("NONEXISTENT_VAR_12345", 4096), 4096);
        assert_eq!(env_or("NONEXISTENT_VAR_12345", "default"), "default");
    }
}
