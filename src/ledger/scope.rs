use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::AppError;

/// The scope contract. Lives at `.ritalin/scope.yaml`.
///
/// This is the user/agent-authored description of the outcome and any
/// machine-checkable acceptance criteria. Obligations are kept in a
/// separate JSONL file because they grow append-only and benefit from
/// line-atomic writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Schema version
    pub version: u32,

    /// One-line statement of the desired outcome
    pub outcome: String,

    /// When this scope was created
    pub created_at: DateTime<Utc>,

    /// Optional notes (free-form, agent or human edited)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

impl Scope {
    pub fn new(outcome: String) -> Self {
        Self {
            version: 1,
            outcome,
            created_at: Utc::now(),
            notes: String::new(),
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<(), AppError> {
        std::fs::create_dir_all(state_dir)?;
        let path = state_dir.join("scope.yaml");
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    pub fn read(state_dir: &Path) -> Result<Self, AppError> {
        let path = state_dir.join("scope.yaml");
        if !path.exists() {
            return Err(AppError::ScopeMissing);
        }
        let yaml = std::fs::read_to_string(path)?;
        let scope: Scope = serde_yaml::from_str(&yaml)?;
        Ok(scope)
    }
}
