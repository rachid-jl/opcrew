use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub problem_hash: String,
    pub problem: String,
    pub outcome: Option<String>,
    pub created_at: String,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRecord {
    pub id: String,
    pub session_id: String,
    pub agent_role: String,
    pub finding: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionRecord {
    pub id: String,
    pub session_id: String,
    pub problem_hash: String,
    pub solution: String,
    pub commands: String,
    pub worked: bool,
    pub failure_reason: Option<String>,
    pub approach_summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachOutcome {
    pub problem_hash: String,
    pub approach: String,
    pub times_succeeded: i32,
    pub times_failed: i32,
}

impl ApproachOutcome {
    pub fn success_rate(&self) -> f32 {
        let total = self.times_succeeded + self.times_failed;
        if total == 0 {
            0.0
        } else {
            self.times_succeeded as f32 / total as f32
        }
    }

    pub fn total_tries(&self) -> i32 {
        self.times_succeeded + self.times_failed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisOutcome {
    pub problem_hash: String,
    pub hypothesis_category: String,
    pub times_confirmed: i32,
    pub times_denied: i32,
}

impl HypothesisOutcome {
    pub fn prior_probability(&self) -> f32 {
        let total = self.times_confirmed + self.times_denied;
        if total == 0 {
            0.5 // No data → neutral prior
        } else {
            self.times_confirmed as f32 / total as f32
        }
    }
}
