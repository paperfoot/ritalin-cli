use std::path::Path;

use crate::error::AppError;

// The .task-incomplete marker file.
//
// Default state: present (created by `ritalin init`). Removed only by `gate`
// after every critical obligation has passing evidence. Hooks and humans can
// inspect this file as a quick "is the agent done?" check.

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn state_dir_in(tmp: &TempDir) -> std::path::PathBuf {
        let dir = tmp.path().join(".ritalin");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_and_exists_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let dir = state_dir_in(&tmp);
        assert!(!exists(&dir));
        create(&dir, "test").unwrap();
        assert!(exists(&dir));
    }

    #[test]
    fn remove_existing() {
        let tmp = TempDir::new().unwrap();
        let dir = state_dir_in(&tmp);
        create(&dir, "test").unwrap();
        assert!(exists(&dir));
        remove(&dir).unwrap();
        assert!(!exists(&dir));
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let tmp = TempDir::new().unwrap();
        let dir = state_dir_in(&tmp);
        assert!(remove(&dir).is_ok());
    }

    #[test]
    fn marker_path_is_parent_of_state_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = state_dir_in(&tmp);
        let path = marker_path(&dir);
        assert_eq!(path.parent().unwrap(), tmp.path());
        assert_eq!(path.file_name().unwrap(), ".task-incomplete");
    }
}
