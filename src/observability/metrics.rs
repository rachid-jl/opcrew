use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

/// In-process metrics tracker. Zero-cost when not queried (atomics only).
pub struct Metrics {
    guardian_approvals: AtomicU32,
    guardian_blocks: AtomicU32,
    guardian_user_prompts: AtomicU32,
    total_tokens: AtomicU32,
    total_api_calls: AtomicU32,
    start_time: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            guardian_approvals: AtomicU32::new(0),
            guardian_blocks: AtomicU32::new(0),
            guardian_user_prompts: AtomicU32::new(0),
            total_tokens: AtomicU32::new(0),
            total_api_calls: AtomicU32::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn record_guardian_approval(&self) {
        self.guardian_approvals.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_guardian_block(&self) {
        self.guardian_blocks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_guardian_prompt(&self) {
        self.guardian_user_prompts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_tokens(&self, tokens: u32) {
        self.total_tokens.fetch_add(tokens, Ordering::Relaxed);
        self.total_api_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens.load(Ordering::Relaxed)
    }

    pub fn summary(&self) -> MetricsSummary {
        let elapsed = self.start_time.elapsed();
        let tokens = self.total_tokens.load(Ordering::Relaxed);
        let calls = self.total_api_calls.load(Ordering::Relaxed);

        MetricsSummary {
            guardian_approvals: self.guardian_approvals.load(Ordering::Relaxed),
            guardian_blocks: self.guardian_blocks.load(Ordering::Relaxed),
            guardian_user_prompts: self.guardian_user_prompts.load(Ordering::Relaxed),
            total_tokens: tokens,
            total_api_calls: calls,
            duration_secs: elapsed.as_secs(),
            tokens_per_minute: if elapsed.as_secs() > 0 {
                (tokens as f64 / elapsed.as_secs_f64() * 60.0) as u32
            } else {
                0
            },
        }
    }
}

#[derive(Debug)]
pub struct MetricsSummary {
    pub guardian_approvals: u32,
    pub guardian_blocks: u32,
    pub guardian_user_prompts: u32,
    pub total_tokens: u32,
    pub total_api_calls: u32,
    pub duration_secs: u64,
    pub tokens_per_minute: u32,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Duration: {}s | Tokens: {} ({}/min) | API calls: {} | Guardian: {} approved, {} blocked, {} prompted",
            self.duration_secs,
            self.total_tokens,
            self.tokens_per_minute,
            self.total_api_calls,
            self.guardian_approvals,
            self.guardian_blocks,
            self.guardian_user_prompts,
        )
    }
}
