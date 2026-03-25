use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use crate::error::{AgentError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker that transitions:
/// Closed → Open (after `failure_threshold` consecutive failures)
/// Open → HalfOpen (after `open_duration` elapses)
/// HalfOpen → Closed (on success) or Open (on failure)
pub struct CircuitBreaker {
    service_name: String,
    failure_threshold: u32,
    open_duration: Duration,
    consecutive_failures: AtomicU32,
    /// Epoch millis when the breaker opened. 0 means not open.
    opened_at_ms: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(service_name: &str, failure_threshold: u32, open_duration: Duration) -> Self {
        Self {
            service_name: service_name.to_string(),
            failure_threshold,
            open_duration,
            consecutive_failures: AtomicU32::new(0),
            opened_at_ms: AtomicU64::new(0),
        }
    }

    pub fn with_defaults(service_name: &str) -> Self {
        Self::new(service_name, 5, Duration::from_secs(30))
    }

    pub fn state(&self) -> BreakerState {
        let opened_at = self.opened_at_ms.load(Ordering::SeqCst);
        if opened_at == 0 {
            return BreakerState::Closed;
        }

        let elapsed = epoch_ms().saturating_sub(opened_at);
        if elapsed >= self.open_duration.as_millis() as u64 {
            BreakerState::HalfOpen
        } else {
            BreakerState::Open
        }
    }

    /// Check if a request is allowed through the breaker.
    pub fn check(&self) -> Result<()> {
        match self.state() {
            BreakerState::Closed | BreakerState::HalfOpen => Ok(()),
            BreakerState::Open => Err(AgentError::CircuitBreakerOpen {
                service: self.service_name.clone(),
            }),
        }
    }

    /// Record a successful call — resets the breaker to Closed.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        self.opened_at_ms.store(0, Ordering::SeqCst);
    }

    /// Record a failed call — may trip the breaker to Open.
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_threshold {
            self.opened_at_ms.store(epoch_ms(), Ordering::SeqCst);
            tracing::warn!(
                service = %self.service_name,
                failures,
                "Circuit breaker opened"
            );
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::with_defaults("test");
        assert_eq!(cb.state(), BreakerState::Closed);
        assert!(cb.check().is_ok());
    }

    #[test]
    fn opens_after_threshold() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), BreakerState::Closed);

        cb.record_failure(); // 3rd failure = threshold
        assert_eq!(cb.state(), BreakerState::Open);
        assert!(cb.check().is_err());
    }

    #[test]
    fn resets_on_success() {
        let cb = CircuitBreaker::new("test", 2, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), BreakerState::Open);

        cb.record_success();
        assert_eq!(cb.state(), BreakerState::Closed);
        assert!(cb.check().is_ok());
    }

    #[test]
    fn transitions_to_half_open() {
        let cb = CircuitBreaker::new("test", 1, Duration::from_millis(10));
        cb.record_failure();
        assert_eq!(cb.state(), BreakerState::Open);

        std::thread::sleep(Duration::from_millis(15));
        assert_eq!(cb.state(), BreakerState::HalfOpen);
        assert!(cb.check().is_ok()); // HalfOpen allows probe requests
    }
}
