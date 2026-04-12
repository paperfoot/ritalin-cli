use assert_cmd::Command;
use tempfile::TempDir;

fn ritalin() -> Command {
    Command::cargo_bin("ritalin").unwrap()
}

#[test]
fn seed_from_toml_manifest() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    let manifest = dir.join("task.toml");
    std::fs::write(
        &manifest,
        r#"
outcome = "User can toggle notifications"

[[obligations]]
claim = "Settings page renders toggle"
proof = "true"
kind = "user_path"
critical = true

[[obligations]]
claim = "POST persists value"
proof = "true"
kind = "integration"
critical = true

[[obligations]]
claim = "Nice-to-have docs"
proof = "echo docs"
kind = "other"
critical = false
"#,
    )
    .unwrap();

    ritalin()
        .args(["seed", manifest.to_str().unwrap()])
        .current_dir(dir)
        .assert()
        .success();

    assert!(dir.join(".ritalin").exists());
    assert!(dir.join(".task-incomplete").exists());

    // Verify obligations were seeded
    let output = ritalin()
        .args(["status", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["data"]["obligations_total"], 3);
    assert_eq!(json["data"]["obligations_critical"], 2);
}

#[test]
fn seed_refuses_overwrite_without_force() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    let manifest = dir.join("task.toml");
    std::fs::write(&manifest, "outcome = \"first\"").unwrap();

    ritalin()
        .args(["seed", manifest.to_str().unwrap()])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["seed", manifest.to_str().unwrap()])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn seed_allows_overwrite_with_force() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    let manifest = dir.join("task.toml");
    std::fs::write(&manifest, "outcome = \"first\"").unwrap();

    ritalin()
        .args(["seed", manifest.to_str().unwrap()])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["seed", manifest.to_str().unwrap(), "--force"])
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn seed_invalid_manifest_errors() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    let manifest = dir.join("bad.toml");
    std::fs::write(&manifest, "this is not valid TOML {{{").unwrap();

    ritalin()
        .args(["seed", manifest.to_str().unwrap()])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn seed_missing_manifest_errors() {
    let tmp = TempDir::new().unwrap();

    ritalin()
        .args(["seed", "/nonexistent/task.toml"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}
