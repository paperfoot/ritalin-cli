use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::cli::ObligationKind;
use crate::error::AppError;

/// A single obligation in the contract.
///
/// Stored append-only in `.ritalin/obligations.jsonl`. Each line is one JSON
/// object. The ledger is never edited in place — adding a new obligation
/// always appends a new line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub id: String,
    pub claim: String,
    pub kind: ObligationKind,
    pub critical: bool,
    pub proof_cmd: String,
    pub created_at: DateTime<Utc>,
}

pub fn ledger_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join("obligations.jsonl")
}

/// Append a new obligation to the ledger. Atomic line-write on POSIX.
pub fn append(state_dir: &Path, ob: &Obligation) -> Result<(), AppError> {
    std::fs::create_dir_all(state_dir)?;
    let path = ledger_path(state_dir);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(ob)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Read all obligations from the ledger. Empty file = empty vector.
pub fn read_all(state_dir: &Path) -> Result<Vec<Obligation>, AppError> {
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
        let ob: Obligation = serde_json::from_str(&line)?;
        out.push(ob);
    }
    Ok(out)
}

/// Compute the next obligation id (O-001, O-002, ...).
pub fn next_id(state_dir: &Path) -> Result<String, AppError> {
    let count = read_all(state_dir)?.len();
    Ok(format!("O-{:03}", count + 1))
}

/// Find an obligation by id.
pub fn find(state_dir: &Path, id: &str) -> Result<Obligation, AppError> {
    read_all(state_dir)?
        .into_iter()
        .find(|o| o.id == id)
        .ok_or_else(|| AppError::UnknownObligation(id.into()))
}
