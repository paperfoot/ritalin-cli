use std::path::Path;

use crate::error::AppError;

/// The .task-incomplete marker file.
///
/// Default state: present (created by `ritalin init`). Removed only by `gate`
/// after every critical obligation has passing evidence. Hooks and humans can
/// inspect this file as a quick "is the agent done?" check.

pub fn marker_path(state_dir: &Path) -> std::path::PathBuf {
    // The marker lives at the *parent* of the .ritalin/ dir so it shows up
    // in repo listings (next to .gitignore etc.) — not buried inside .ritalin/.
    state_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".task-incomplete")
}

pub fn create(state_dir: &Path, content: &str) -> Result<(), AppError> {
    std::fs::write(marker_path(state_dir), content)?;
    Ok(())
}

pub fn remove(state_dir: &Path) -> Result<(), AppError> {
    let path = marker_path(state_dir);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn exists(state_dir: &Path) -> bool {
    marker_path(state_dir).exists()
}
