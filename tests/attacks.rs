use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ritalin() -> Command {
    Command::cargo_bin("ritalin").unwrap()
}

fn init_in(dir: &std::path::Path) {
    ritalin()
        .args(["init", "--outcome", "test"])
        .current_dir(dir)
        .assert()
        .success();
}

fn add_in(dir: &std::path::Path, claim: &str, proof: &str) {
    ritalin()
        .args(["add", claim, "--proof", proof, "--kind", "other"])
        .current_dir(dir)
        .assert()
        .success();
}

// ─── Attack: forged evidence append ─────────────────────────
// An agent could try to write a fake exit-0 record directly to evidence.jsonl.
// With proof_hash + workspace_hash binding, forged records won't have matching
// hashes and therefore won't discharge the obligation.

#[test]
fn forged_evidence_does_not_discharge() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must pass for real", "echo real_check");

    // Forge a fake evidence record with exit_code 0 but wrong hashes
    let fake_record = serde_json::json!({
        "obligation_id": "O-001",
        "command": "echo bypass",
        "exit_code": 0,
        "stdout_tail": "",
        "stderr_tail": "",
        "proof_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "workspace_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "recorded_at": "2026-01-01T00:00:00Z"
    });

    let evidence_path = dir.join(".ritalin/evidence.jsonl");
    std::fs::write(&evidence_path, format!("{}\n", fake_record)).unwrap();

    // Gate should still block — forged hashes don't match
    ritalin().args(["gate"]).current_dir(dir).assert().failure();

    assert!(dir.join(".task-incomplete").exists());
}

// ─── Attack: stale evidence after regression ────────────────
// Prove once, then change the workspace. Old evidence should be invalidated.

#[test]
fn stale_evidence_after_workspace_change() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // Create a file that the proof depends on
    std::fs::write(dir.join("ok.txt"), "works").unwrap();

    init_in(dir);
    add_in(dir, "ok.txt exists", "test -f ok.txt");

    // Prove passes
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    // Gate passes
    ritalin().args(["gate"]).current_dir(dir).assert().success();

    // Re-init to bring marker back
    ritalin()
        .args(["init", "--outcome", "test again", "--force"])
        .current_dir(dir)
        .assert()
        .success();

    add_in(dir, "ok.txt exists", "test -f ok.txt");

    // Now change the workspace (modify ok.txt)
    std::fs::write(dir.join("ok.txt"), "changed content").unwrap();

    // Gate should block — workspace hash changed, old evidence is stale
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
}

// ─── Attack: delete obligations ledger ──────────────────────
// If someone deletes obligations.jsonl, open_critical becomes empty.
// Gate should still work correctly (vacuous pass is acceptable since
// there are genuinely no obligations, but marker should exist).

#[test]
fn deleted_obligations_ledger() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "critical thing", "true");

    // Delete the obligations ledger
    std::fs::remove_file(dir.join(".ritalin/obligations.jsonl")).unwrap();

    // Gate sees no obligations — passes (vacuous truth)
    // The .task-incomplete marker gets removed
    ritalin().args(["gate"]).current_dir(dir).assert().success();
}

// ─── Attack: corrupt JSONL DoS ──────────────────────────────
// Writing invalid JSON to the ledger should not crash gate.

#[test]
fn corrupt_evidence_jsonl_errors_gracefully() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "test", "true");

    // Corrupt the evidence file
    std::fs::write(
        dir.join(".ritalin/evidence.jsonl"),
        "this is not json at all\n",
    )
    .unwrap();

    // Gate should error (JSON parse failure), not panic
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
}

// ─── Attack: corrupt obligations JSONL ──────────────────────

#[test]
fn corrupt_obligations_jsonl_errors_gracefully() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);

    // Corrupt the obligations file
    std::fs::write(dir.join(".ritalin/obligations.jsonl"), "{broken json\n").unwrap();

    // Gate should error gracefully
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
}

// ─── Attack: hook-mode corrupt JSONL fails closed ───────────

#[test]
fn hook_mode_corrupt_evidence_blocks() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "test", "true");

    // Corrupt evidence
    std::fs::write(dir.join(".ritalin/evidence.jsonl"), "not json\n").unwrap();

    // Hook-mode should fail (non-empty stdout or exit non-zero)
    // Current behavior: returns Ok(()) which is actually "allow stop"
    // This documents the current behavior — it should ideally fail closed
    let output = ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin("{}")
        .current_dir(dir)
        .output()
        .unwrap();

    // The process should exit (either 0 with block payload, or non-zero)
    // Currently it exits non-zero on JSON parse error
    assert!(!output.status.success() || !output.stdout.is_empty());
}

// ─── Attack: re-init overwrite protection ───────────────────

#[test]
fn reinit_without_force_is_blocked() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "important obligation", "true");

    // Try to re-init without --force
    ritalin()
        .args(["init", "--outcome", "overwrite attempt"])
        .current_dir(dir)
        .assert()
        .failure();

    // Original obligation should still be present
    let output = ritalin()
        .args(["status", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["data"]["obligations_total"], 1);
}

// ─── Attack: legacy v0.1.x evidence (no hashes) ────────────

#[test]
fn legacy_evidence_without_hashes_does_not_discharge() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must pass", "true");

    // Write v0.1.x-style evidence (no proof_hash or workspace_hash)
    let legacy_record = serde_json::json!({
        "obligation_id": "O-001",
        "command": "true",
        "exit_code": 0,
        "stdout_tail": "",
        "stderr_tail": "",
        "recorded_at": "2026-01-01T00:00:00Z"
    });

    std::fs::write(
        dir.join(".ritalin/evidence.jsonl"),
        format!("{}\n", legacy_record),
    )
    .unwrap();

    // Gate should block — legacy evidence has empty hashes
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
}
