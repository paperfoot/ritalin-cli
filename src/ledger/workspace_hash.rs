use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::AppError;

/// Compute a SHA-256 digest over all tracked files in the workspace.
/// Excludes `.git/`, `.ritalin/`, `.task-incomplete`, and `target/`.
pub fn compute(root: &Path) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let mut paths = Vec::new();
    collect_files(root, &mut paths)?;
    paths.sort();
    for path in &paths {
        let rel = path.strip_prefix(root).unwrap_or(path);
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update(b"\0");
        let content = std::fs::read(path)?;
        hasher.update(&content);
        hasher.update(b"\0");
    }
    Ok(hex::encode(hasher.finalize()))
}

fn collect_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), AppError> {
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
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}
