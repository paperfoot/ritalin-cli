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

/// Returns true if the obligation has at least one passing record whose
/// `command` field hashes to the expected proof and whose workspace_hash
/// matches the current workspace.
///
/// We deliberately recompute the proof hash from `r.command` instead of
/// trusting the stored `r.proof_hash` field. Trusting the stored field
/// allows an attacker to append a forged record where `command` is anything
/// and `proof_hash` is set to the obligation's expected hash (which is
/// readable from `obligations.jsonl`). Recomputing from `r.command` forces
/// the recorded command to actually be the obligation's proof_cmd before
/// the record can discharge.
///
/// Legacy v0.1.x records that lack a `command` field, or evidence with no
/// workspace_hash, cannot discharge.
pub fn is_discharged(
    records: &[Evidence],
    expected_proof_hash: &str,
    current_workspace_hash: &str,
) -> bool {
    records.iter().any(|r| {
        r.exit_code == 0
            && !r.command.is_empty()
            && !r.workspace_hash.is_empty()
            && proof_hash(&r.command) == expected_proof_hash
            && r.workspace_hash == current_workspace_hash
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceState {
    Passed,
    Missing,
    Failed,
    ProofMismatch,
    StaleWorkspace,
    Legacy,
}

impl EvidenceState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Missing => "missing",
            Self::Failed => "failed",
            Self::ProofMismatch => "proof_mismatch",
            Self::StaleWorkspace => "stale_workspace",
            Self::Legacy => "legacy_evidence",
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            Self::Passed => "passing evidence matches the stored proof and current workspace",
            Self::Missing => "no evidence has been recorded",
            Self::Failed => "the last recorded proof command did not pass",
            Self::ProofMismatch => "passing evidence was recorded for a different proof command",
            Self::StaleWorkspace => {
                "passing evidence exists, but the workspace changed since it was recorded"
            }
            Self::Legacy => {
                "passing evidence exists, but it lacks proof/workspace hashes and cannot discharge"
            }
        }
    }
}

/// Classify why an obligation is or is not discharged. This is deliberately
/// more specific than `is_discharged` so gate/status output can tell agents
/// whether to re-run the same proof, fix a failing command, or stop using a
/// `--cmd` override that cannot satisfy the stored contract.
pub fn classify(
    records: Option<&[Evidence]>,
    expected_proof_hash: &str,
    current_workspace_hash: &str,
) -> EvidenceState {
    let Some(records) = records else {
        return EvidenceState::Missing;
    };
    if records.is_empty() {
        return EvidenceState::Missing;
    }
    if is_discharged(records, expected_proof_hash, current_workspace_hash) {
        return EvidenceState::Passed;
    }

    // Recompute the proof hash from `r.command` for the same reason
    // is_discharged does — the stored `proof_hash` field is informational
    // and not trusted for verification.
    let cmd_matches_proof = |r: &Evidence| -> bool {
        !r.command.is_empty() && proof_hash(&r.command) == expected_proof_hash
    };

    if records.iter().any(|r| {
        r.exit_code == 0
            && cmd_matches_proof(r)
            && !r.workspace_hash.is_empty()
            && r.workspace_hash != current_workspace_hash
    }) {
        return EvidenceState::StaleWorkspace;
    }

    if records
        .iter()
        .any(|r| r.exit_code == 0 && !r.command.is_empty() && !cmd_matches_proof(r))
    {
        return EvidenceState::ProofMismatch;
    }

    if records
        .iter()
        .any(|r| r.exit_code == 0 && (r.command.is_empty() || r.workspace_hash.is_empty()))
    {
        return EvidenceState::Legacy;
    }

    EvidenceState::Failed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an Evidence record for a given executed command, mirroring
    /// what `prove.rs` writes. The `proof_hash` field is computed from the
    /// command (matching production behaviour); is_discharged ignores the
    /// stored field and recomputes anyway.
    fn ev(command: &str, exit_code: i32, workspace_hash: &str) -> Evidence {
        Evidence {
            obligation_id: "O-001".into(),
            command: command.into(),
            exit_code,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            proof_hash: proof_hash(command),
            workspace_hash: workspace_hash.into(),
            recorded_at: chrono::Utc::now(),
        }
    }

    /// Variant for tests that need to exercise a forged stored proof_hash
    /// field — i.e. evidence whose `proof_hash` field LIES about its
    /// `command`. Pre-fix this would discharge; post-fix is_discharged
    /// recomputes from `command` and rejects.
    fn ev_with_forged_hash(
        command: &str,
        exit_code: i32,
        workspace_hash: &str,
        forged_proof_hash: &str,
    ) -> Evidence {
        Evidence {
            obligation_id: "O-001".into(),
            command: command.into(),
            exit_code,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            proof_hash: forged_proof_hash.into(),
            workspace_hash: workspace_hash.into(),
            recorded_at: chrono::Utc::now(),
        }
    }

    const GOOD_CMD: &str = "true";
    const GOOD_WH: &str = "def456";

    fn good_ph() -> String {
        proof_hash(GOOD_CMD)
    }

    #[test]
    fn empty_records_not_discharged() {
        assert!(!is_discharged(&[], &good_ph(), GOOD_WH));
    }

    #[test]
    fn all_failing_not_discharged() {
        let recs = vec![
            ev(GOOD_CMD, 1, GOOD_WH),
            ev(GOOD_CMD, 2, GOOD_WH),
            ev(GOOD_CMD, -1, GOOD_WH),
        ];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn one_passing_with_matching_command_discharges() {
        let recs = vec![ev(GOOD_CMD, 1, GOOD_WH), ev(GOOD_CMD, 0, GOOD_WH)];
        assert!(is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn passing_with_wrong_command_not_discharged() {
        // command="echo bypass" hashes to something other than expected.
        let recs = vec![ev("echo bypass", 0, GOOD_WH)];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn passing_with_wrong_workspace_hash_not_discharged() {
        let recs = vec![ev(GOOD_CMD, 0, "stale")];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn empty_command_not_discharged() {
        let recs = vec![ev("", 0, GOOD_WH)];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn empty_workspace_hash_not_discharged() {
        let recs = vec![ev(GOOD_CMD, 0, "")];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn legacy_v01x_evidence_not_discharged() {
        // Empty command + empty workspace hash. The shape v0.1.x wrote.
        let recs = vec![ev("", 0, "")];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    #[test]
    fn mixed_pass_fail_with_one_valid_discharges() {
        let recs = vec![
            ev(GOOD_CMD, 1, GOOD_WH),
            ev("echo bypass", 0, GOOD_WH),
            ev(GOOD_CMD, 0, "stale"),
            ev(GOOD_CMD, 0, GOOD_WH), // this one counts
        ];
        assert!(is_discharged(&recs, &good_ph(), GOOD_WH));
    }

    /// Fix A regression test: a record whose stored `proof_hash` field is
    /// forged to match the obligation's expected hash, but whose `command`
    /// hashes to something else, must NOT discharge.
    #[test]
    fn forged_stored_proof_hash_does_not_discharge() {
        let recs = vec![ev_with_forged_hash(
            "echo bypass", // command the agent actually wrote
            0,
            GOOD_WH,
            &good_ph(), // forged stored hash claiming "I ran the real proof"
        )];
        assert!(!is_discharged(&recs, &good_ph(), GOOD_WH));
        // And classify should call this out as a proof mismatch.
        assert_eq!(
            classify(Some(&recs), &good_ph(), GOOD_WH),
            EvidenceState::ProofMismatch
        );
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
