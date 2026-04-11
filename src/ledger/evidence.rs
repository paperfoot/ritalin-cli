use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::AppError;

/// One evidence record. Stored append-only in `.ritalin/evidence.jsonl`.
///
/// Each `prove` invocation appends one record. An obligation is considered
/// discharged when at least one evidence record exists for its id with
/// `exit_code == 0`, matching proof_hash, and matching workspace_hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub obligation_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout_tail: String,
    pub stderr_tail: String,
    #[serde(default)]
    pub proof_hash: String,
    #[serde(default)]
    pub workspace_hash: String,
    pub recorded_at: DateTime<Utc>,
}

/// SHA-256 hash of the normalised proof command (trimmed whitespace).
pub fn proof_hash(cmd: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cmd.trim().as_bytes());
    hex::encode(hasher.finalize())
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

/// Returns true if the obligation has at least one passing record with
/// matching proof_hash and workspace_hash. Empty hashes (v0.1.x records)
/// are treated as non-matching so legacy evidence cannot discharge.
pub fn is_discharged(
    records: &[Evidence],
    expected_proof_hash: &str,
    current_workspace_hash: &str,
) -> bool {
    records.iter().any(|r| {
        r.exit_code == 0
            && !r.proof_hash.is_empty()
            && !r.workspace_hash.is_empty()
            && r.proof_hash == expected_proof_hash
            && r.workspace_hash == current_workspace_hash
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_evidence(exit_code: i32, proof_hash: &str, workspace_hash: &str) -> Evidence {
        Evidence {
            obligation_id: "O-001".into(),
            command: "true".into(),
            exit_code,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            proof_hash: proof_hash.into(),
            workspace_hash: workspace_hash.into(),
            recorded_at: chrono::Utc::now(),
        }
    }

    const GOOD_PH: &str = "abc123";
    const GOOD_WH: &str = "def456";

    #[test]
    fn empty_records_not_discharged() {
        assert!(!is_discharged(&[], GOOD_PH, GOOD_WH));
    }

    #[test]
    fn all_failing_not_discharged() {
        let recs = vec![
            make_evidence(1, GOOD_PH, GOOD_WH),
            make_evidence(2, GOOD_PH, GOOD_WH),
            make_evidence(-1, GOOD_PH, GOOD_WH),
        ];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn one_passing_with_matching_hashes_discharges() {
        let recs = vec![
            make_evidence(1, GOOD_PH, GOOD_WH),
            make_evidence(0, GOOD_PH, GOOD_WH),
        ];
        assert!(is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn passing_with_wrong_proof_hash_not_discharged() {
        let recs = vec![make_evidence(0, "wrong", GOOD_WH)];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn passing_with_wrong_workspace_hash_not_discharged() {
        let recs = vec![make_evidence(0, GOOD_PH, "stale")];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn empty_proof_hash_not_discharged() {
        let recs = vec![make_evidence(0, "", GOOD_WH)];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn empty_workspace_hash_not_discharged() {
        let recs = vec![make_evidence(0, GOOD_PH, "")];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn legacy_v01x_evidence_not_discharged() {
        let recs = vec![make_evidence(0, "", "")];
        assert!(!is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn mixed_pass_fail_with_one_valid_discharges() {
        let recs = vec![
            make_evidence(1, GOOD_PH, GOOD_WH),
            make_evidence(0, "wrong", GOOD_WH),
            make_evidence(0, GOOD_PH, "stale"),
            make_evidence(0, GOOD_PH, GOOD_WH), // this one counts
        ];
        assert!(is_discharged(&recs, GOOD_PH, GOOD_WH));
    }

    #[test]
    fn proof_hash_is_deterministic() {
        let h1 = proof_hash("echo hello");
        let h2 = proof_hash("echo hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex
    }

    #[test]
    fn proof_hash_trims_whitespace() {
        let h1 = proof_hash("echo hello");
        let h2 = proof_hash("  echo hello  ");
        assert_eq!(h1, h2);
    }

    #[test]
    fn proof_hash_differs_for_different_commands() {
        assert_ne!(proof_hash("true"), proof_hash("false"));
    }
}
