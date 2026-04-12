use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

use crate::error::AppError;

/// Compute a SHA-256 digest over tracked workspace files.
///
/// Uses `git ls-files` when inside a git repo (respects `.gitignore`,
/// deterministic regardless of cwd). Falls back to recursive walk for
/// non-git repos, excluding `.git/`, `.ritalin/`, `.task-incomplete`,
/// and `target/`.
pub fn compute(root: &Path) -> Result<String, AppError> {
    let paths = git_tracked_files(root).unwrap_or_else(|_| walk_files(root));
    hash_files(root, &paths)
}

fn hash_files(root: &Path, paths: &[std::path::PathBuf]) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let mut sorted = paths.to_vec();
    sorted.sort();
    for path in &sorted {
        let full = root.join(path);
        if !full.is_file() {
            continue;
        }
        // Hash the relative path so result is location-independent
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(b"\0");
        let content = std::fs::read(&full)?;
        hasher.update(&content);
        hasher.update(b"\0");
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Use `git ls-files` to get tracked files (respects .gitignore).
/// Returns relative paths from repo root.
fn git_tracked_files(root: &Path) -> Result<Vec<std::path::PathBuf>, AppError> {
    let output = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(root)
        .output()
        .map_err(|e| AppError::InvalidInput(format!("git ls-files failed: {e}")))?;

    if !output.status.success() {
        return Err(AppError::InvalidInput("not a git repository".into()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<std::path::PathBuf> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with(".ritalin/") && *l != ".task-incomplete")
        .map(std::path::PathBuf::from)
        .collect();
    Ok(paths)
}

/// Fallback for non-git repos: recursive walk excluding build dirs.
fn walk_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    collect_files(root, root, &mut out).ok();
    out
}

fn collect_files(
    base: &Path,
    dir: &Path,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<(), AppError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git"
            || name_str == ".ritalin"
            || name_str == ".task-incomplete"
            || name_str == "target"
        {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, out)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_path_buf());
            }
        }
    }
    Ok(())
}
