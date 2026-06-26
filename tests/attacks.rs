use assert_cmd::Command;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn ritalin() -> Command {
    let mut cmd = Command::cargo_bin("ritalin").unwrap();
    // Hermetic by default: never inherit an ambient RITALIN_GATE opt-out, which
    // would disable hook-mode gating and defeat these fail-closed attack tests.
    cmd.env_remove("RITALIN_GATE");
    cmd
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

fn proof_hash(cmd: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cmd.trim().as_bytes());
    hex::encode(hasher.finalize())
}

// ─── Attack: forged evidence append (zero-hash, trivial) ─────
// Baseline: anyone can append a record. With wrong hashes the gate
// rejects it.

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

// ─── Attack: forged evidence with hash matching the obligation ─
// The real attack class observed in production (codex agent rewriting
// evidence.jsonl). The attacker reads the obligation's stored proof_cmd,
// computes its sha256, writes an evidence record with `command: <garbage>`
// but `proof_hash: <hash of obligation.proof_cmd>`. Pre-fix, gate trusted
// the stored proof_hash field and accepted the record. Post-fix, gate
// recomputes proof_hash from `r.command` and rejects.

#[test]
fn forged_evidence_with_matching_proof_hash_does_not_discharge() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "must run real proof", "echo real_check");

    // Run prove honestly so we capture the current workspace_hash.
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
    let evidence_path = dir.join(".ritalin/evidence.jsonl");
    let real_line = std::fs::read_to_string(&evidence_path)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let real_record: serde_json::Value = serde_json::from_str(&real_line).unwrap();
    let ws_hash = real_record["workspace_hash"].as_str().unwrap().to_string();

    // Wipe the genuine evidence and replace with a forged record whose
    // command is something the agent NEVER ran but whose stored proof_hash
    // matches the obligation's proof_cmd. This is the "Codex synthesised
    // evidence" attack from session 2026-05-13.
    let forged_proof_hash = proof_hash("echo real_check");
    let forged = serde_json::json!({
        "obligation_id": "O-001",
        "command": "echo bypass",
        "exit_code": 0,
        "stdout_tail": "",
        "stderr_tail": "",
        "proof_hash": forged_proof_hash,
        "workspace_hash": ws_hash,
        "recorded_at": "2026-05-13T12:00:00Z"
    });
    std::fs::write(&evidence_path, format!("{}\n", forged)).unwrap();

    // With Fix A: gate recomputes proof_hash from r.command = "echo bypass"
    // = sha256("echo bypass") which does NOT match sha256("echo real_check").
    // Forgery rejected.
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
    assert!(dir.join(".task-incomplete").exists());

    // Status should classify the forgery as proof_mismatch (informational).
    let status = ritalin()
        .args(["status", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        json["data"]["obligations"][0]["evidence_status"],
        "proof_mismatch"
    );
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
// Gate must FAIL — empty contracts cannot pass. An agent that deletes
// the ledger should not be able to bypass .task-incomplete.

#[test]
fn deleted_obligations_ledger() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "critical thing", "true");

    // Delete the obligations ledger
    std::fs::remove_file(dir.join(".ritalin/obligations.jsonl")).unwrap();

    // Gate sees no obligations — must fail (empty contract bypass blocked)
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
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

    let output = ritalin()
        .args(["gate", "--hook-mode"])
        .write_stdin("{}")
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["decision"], "block");
    assert!(
        json["reason"]
            .as_str()
            .unwrap()
            .contains("could not verify the contract")
    );
    assert!(dir.join(".task-incomplete").exists());
}

#[test]
fn hook_mode_env_disable_short_circuits_before_state_read() {
    // The opt-out returns before any contract/evidence read, so even an
    // unreadable contract that would otherwise fail-closed (block) stops
    // cleanly under RITALIN_GATE=0 — without mutating the marker.
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "test", "true");
    std::fs::write(dir.join(".ritalin/evidence.jsonl"), "not json\n").unwrap();

    let output = ritalin()
        .args(["gate", "--hook-mode"])
        .env("RITALIN_GATE", "0")
        .write_stdin("{}")
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(dir.join(".task-incomplete").exists());
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
