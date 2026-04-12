/// On-disk state for ritalin contracts.
///
/// Layout (relative to current working directory):
///   .ritalin/scope.yaml         — human-edited contract (outcome + metadata)
///   .ritalin/obligations.jsonl  — append-only obligation ledger
///   .ritalin/evidence.jsonl     — append-only verification evidence ledger
///   .task-incomplete            — marker file; presence = "agent must keep working"
///
/// Why JSONL for ledgers? Append-only writes are atomic line-by-line on POSIX,
/// so we never corrupt the ledger even on crash. No locking needed for the
/// single-builder case; multi-writer scenarios should serialize through `gate`.
///
/// Why YAML for scope? Humans (and agents) read and write it directly;
/// JSON's lack of comments makes it hostile to in-line acceptance criteria.
pub mod evidence;
pub mod marker;
pub mod obligations;
pub mod scope;
pub mod workspace_hash;

use std::path::{Path, PathBuf};

/// Find the ritalin state directory by walking up from cwd.
/// Returns `.ritalin/` next to the first ancestor that contains it,
/// or `<cwd>/.ritalin/` if no ancestor has one (used by `init`).
pub fn state_dir(cwd: &Path) -> PathBuf {
    let mut current = Some(cwd);
    while let Some(dir) = current {
        if dir.join(".ritalin").exists() {
            return dir.join(".ritalin");
        }
        current = dir.parent();
    }
    cwd.join(".ritalin")
}

/// Returns true if `.ritalin/` exists somewhere up the cwd ancestry.
pub fn is_initialized(cwd: &Path) -> bool {
    let mut current = Some(cwd);
    while let Some(dir) = current {
        if dir.join(".ritalin").exists() {
            return true;
        }
        current = dir.parent();
    }
    false
}
