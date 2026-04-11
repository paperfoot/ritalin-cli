use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::AppError;

/// One evidence record. Stored append-only in `.ritalin/evidence.jsonl`.
///
/// Each `prove` invocation appends one record. An obligation is considered
/// discharged when at least one evidence record exists for its id with
/// `exit_code == 0`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub obligation_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub recorded_at: DateTime<Utc>,
}

pub fn ledger_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join("evidence.jsonl")
}

pub fn append(state_dir: &Path, ev: &Evidence) -> Result<(), AppError> {
    std::fs::create_dir_all(state_dir)?;
    let path = ledger_path(state_dir);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(ev)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_all(state_dir: &Path) -> Result<Vec<Evidence>, AppError> {
    let path = ledger_path(state_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let ev: Evidence = serde_json::from_str(&line)?;
        out.push(ev);
    }
    Ok(out)
}

/// Index evidence by obligation id, returning all records per obligation.
pub fn index_by_obligation(state_dir: &Path) -> Result<HashMap<String, Vec<Evidence>>, AppError> {
    let mut map: HashMap<String, Vec<Evidence>> = HashMap::new();
    for ev in read_all(state_dir)? {
        map.entry(ev.obligation_id.clone()).or_default().push(ev);
    }
    Ok(map)
}

/// Returns true if the obligation has at least one passing (exit 0) evidence record.
pub fn is_discharged(records: &[Evidence]) -> bool {
    records.iter().any(|r| r.exit_code == 0)
}
