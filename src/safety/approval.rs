use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::Mutex;

/// User approval flow with rate limiting and approve-all-similar support.
pub struct ApprovalManager {
    max_prompts: u16,
    prompts_used: Mutex<u16>,
    approved_binaries: Mutex<HashSet<String>>,
    blocked_binaries: Mutex<HashSet<String>>,
    agent_request_counts: Mutex<std::collections::HashMap<String, u16>>,
    auto_approve: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalResult {
    Approved,
    Denied,
    ApproveAllSimilar(String),
    BlockAllSimilar(String),
    AgentLoopDetected(String),
    PromptLimitReached,
}

impl ApprovalManager {
    pub fn new(max_prompts: u16, auto_approve: bool) -> Self {
        Self {
            max_prompts,
            prompts_used: Mutex::new(0),
            approved_binaries: Mutex::new(HashSet::new()),
            blocked_binaries: Mutex::new(HashSet::new()),
            agent_request_counts: Mutex::new(std::collections::HashMap::new()),
            auto_approve,
        }
    }

    /// Check if a command is pre-approved or pre-blocked.
    pub fn check_cached(&self, binary: &str) -> Option<bool> {
        if self.approved_binaries.lock().unwrap().contains(binary) {
            return Some(true);
        }
        if self.blocked_binaries.lock().unwrap().contains(binary) {
            return Some(false);
        }
        None
    }

    /// Track agent requests for loop detection.
    /// Returns true if the agent appears stuck (3+ identical requests).
    pub fn track_request(&self, agent_id: &str, command_pattern: &str) -> bool {
        let key = format!("{agent_id}:{command_pattern}");
        let mut counts = self.agent_request_counts.lock().unwrap();
        let count = counts.entry(key).or_insert(0);
        *count += 1;
        *count >= 3
    }

    /// Prompt the user for approval.
    pub fn prompt_user(
        &self,
        agent_role: &str,
        command: &str,
        binary: &str,
        reason: &str,
        risk_level: &str,
    ) -> ApprovalResult {
        // Check prompt limit
        {
            let mut used = self.prompts_used.lock().unwrap();
            if *used >= self.max_prompts {
                return ApprovalResult::PromptLimitReached;
            }
            *used += 1;
        }

        // Auto-approve mode
        if self.auto_approve {
            return ApprovalResult::Approved;
        }

        // Display prompt
        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  APPROVAL REQUIRED");
        eprintln!("{}", "=".repeat(60));
        eprintln!("  Agent:   {agent_role}");
        eprintln!("  Command: {command}");
        eprintln!("  Risk:    {risk_level}");
        eprintln!("  Reason:  {reason}");
        eprintln!("{}", "-".repeat(60));
        eprintln!("  [y]es / [n]o / [a]pprove-all-{binary} / [b]lock-all-{binary}");
        eprint!("  > ");
        io::stderr().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return ApprovalResult::Denied;
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => ApprovalResult::Approved,
            "n" | "no" => ApprovalResult::Denied,
            "a" => {
                self.approved_binaries
                    .lock()
                    .unwrap()
                    .insert(binary.to_string());
                ApprovalResult::ApproveAllSimilar(binary.to_string())
            }
            "b" => {
                self.blocked_binaries
                    .lock()
                    .unwrap()
                    .insert(binary.to_string());
                ApprovalResult::BlockAllSimilar(binary.to_string())
            }
            _ => ApprovalResult::Denied,
        }
    }

    pub fn prompts_remaining(&self) -> u16 {
        self.max_prompts - *self.prompts_used.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_approval() {
        let mgr = ApprovalManager::new(20, false);
        assert!(mgr.check_cached("systemctl").is_none());

        mgr.approved_binaries
            .lock()
            .unwrap()
            .insert("systemctl".into());
        assert_eq!(mgr.check_cached("systemctl"), Some(true));
    }

    #[test]
    fn loop_detection() {
        let mgr = ApprovalManager::new(20, false);
        assert!(!mgr.track_request("agent1", "systemctl restart nginx"));
        assert!(!mgr.track_request("agent1", "systemctl restart nginx"));
        assert!(mgr.track_request("agent1", "systemctl restart nginx")); // 3rd time
    }

    #[test]
    fn prompt_limit() {
        let mgr = ApprovalManager::new(1, true); // auto_approve to avoid stdin
        assert_eq!(
            mgr.prompt_user("dev", "cmd", "bin", "reason", "risky"),
            ApprovalResult::Approved
        );
        // Second prompt hits limit
        assert_eq!(
            mgr.prompt_user("dev", "cmd2", "bin2", "reason", "risky"),
            ApprovalResult::PromptLimitReached
        );
    }
}
