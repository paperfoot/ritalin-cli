use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn ritalin() -> Command {
    Command::cargo_bin("ritalin").unwrap()
}

fn init_in(dir: &Path) {
    ritalin()
        .args(["init", "--outcome", "test outcome"])
        .current_dir(dir)
        .assert()
        .success();
}

fn add_in(dir: &Path, claim: &str, proof: &str) {
    ritalin()
        .args(["add", claim, "--proof", proof, "--kind", "other"])
        .current_dir(dir)
        .assert()
        .success();
}

// ─── Happy path ─────────────────────────────────────────────

#[test]
fn happy_path_init_add_prove_gate() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // init
    init_in(dir);
    assert!(dir.join(".ritalin").exists());
    assert!(dir.join(".task-incomplete").exists());

    // add
    add_in(dir, "must pass", "true");
    assert!(dir.join(".ritalin/obligations.jsonl").exists());

    // prove
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
    assert!(dir.join(".ritalin/evidence.jsonl").exists());

    // gate
    ritalin().args(["gate"]).current_dir(dir).assert().success();
    assert!(!dir.join(".task-incomplete").exists());
}

// ─── Failure path ───────────────────────────────────────────

#[test]
fn failing_proof_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must fail", "false");

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure();

    // Evidence is still recorded even on failure
    assert!(dir.join(".ritalin/evidence.jsonl").exists());
}

#[test]
fn gate_blocks_with_open_obligations() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must pass", "false");

    // prove fails
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure();

    // gate blocks
    ritalin().args(["gate"]).current_dir(dir).assert().failure();

    assert!(dir.join(".task-incomplete").exists());
}

// ─── Hook mode ──────────────────────────────────────────────

#[test]
fn hook_mode_blocks_with_json() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "undone", "false");

    // prove to get evidence (fails)
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure();

    // hook-mode should emit block decision
    ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin("{}")
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"block\""));
}

#[test]
fn hook_mode_respects_stop_hook_active() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "undone", "false");

    ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin(r#"{"stop_hook_active":true}"#)
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn hook_mode_allows_stop_when_all_proved() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must pass", "true");

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin("{}")
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn hook_mode_uninitialised_allows_stop() {
    let tmp = TempDir::new().unwrap();

    ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin("{}")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ─── Edge cases ─────────────────────────────────────────────

#[test]
fn gate_before_init_errors() {
    let tmp = TempDir::new().unwrap();

    ritalin()
        .args(["gate"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn add_before_init_errors() {
    let tmp = TempDir::new().unwrap();

    ritalin()
        .args(["add", "claim", "--proof", "true"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn add_empty_claim_errors() {
    let tmp = TempDir::new().unwrap();
    init_in(tmp.path());

    ritalin()
        .args(["add", "", "--proof", "true"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn advisory_obligations_dont_block_gate() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);

    // Add advisory (non-critical) obligation with failing proof
    ritalin()
        .args([
            "add",
            "advisory thing",
            "--proof",
            "echo advisory",
            "--kind",
            "other",
            "--critical=false",
        ])
        .current_dir(dir)
        .assert()
        .success();

    // gate should pass (no critical obligations)
    ritalin().args(["gate"]).current_dir(dir).assert().success();
}

#[test]
fn init_refuses_overwrite_without_force() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);

    ritalin()
        .args(["init", "--outcome", "second"])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn init_allows_overwrite_with_force() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);

    ritalin()
        .args(["init", "--outcome", "second", "--force"])
        .current_dir(dir)
        .assert()
        .success();
}

// ─── JSON contract ──────────────────────────────────────────

#[test]
fn json_output_has_envelope() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);

    let output = ritalin()
        .args(["status", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["version"], "1");
    assert_eq!(json["status"], "success");
    assert!(json["data"].is_object());
}

#[test]
fn gate_json_fail_emits_single_envelope() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "undone", "false");

    // prove (fails, records evidence)
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure();

    let output = ritalin()
        .args(["gate", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    // stdout should have exactly one JSON document with status "fail"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(json["version"], "1");
    assert_eq!(json["status"], "fail");
    assert_eq!(json["data"]["verdict"], "fail");
    assert!(json["data"]["obligations_open_critical"].as_u64().unwrap() > 0);

    // stderr should be empty (no duplicate error envelope)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty but got: {stderr}"
    );
}

#[test]
fn gate_json_pass_emits_success_envelope() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must pass", "true");

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    let output = ritalin()
        .args(["gate", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(json["version"], "1");
    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["verdict"], "pass");
}
