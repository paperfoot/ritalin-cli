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

// ─── Fix: --cmd override does not discharge ────────────────

#[test]
fn cmd_override_does_not_discharge_original() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    // Obligation requires "false" (which would fail)
    add_in(dir, "must fail proof", "false");

    // Prove with --cmd override that succeeds
    ritalin()
        .args(["prove", "O-001", "--cmd", "true"])
        .current_dir(dir)
        .assert()
        .success(); // prove itself succeeds (exit 0)

    // But gate should FAIL — the override hash doesn't match the stored proof
    ritalin().args(["gate"]).current_dir(dir).assert().failure();
    assert!(dir.join(".task-incomplete").exists());
}

// ─── Fix: --force clears old ledgers ───────────────────────

#[test]
fn init_force_clears_obligations_and_evidence() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // First contract
    init_in(dir);
    add_in(dir, "old obligation", "true");
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    // Force re-init
    ritalin()
        .args(["init", "--outcome", "fresh start", "--force"])
        .current_dir(dir)
        .assert()
        .success();

    // Old obligations should be gone — gate should fail with "empty"
    ritalin().args(["gate"]).current_dir(dir).assert().failure();

    // Verify the ledger files are actually gone
    assert!(!dir.join(".ritalin/obligations.jsonl").exists());
    assert!(!dir.join(".ritalin/evidence.jsonl").exists());
}

#[test]
fn seed_force_clears_old_obligations() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    // First contract with an obligation
    init_in(dir);
    add_in(dir, "old obligation", "echo old");

    // Create a manifest
    let manifest = dir.join("contract.toml");
    std::fs::write(
        &manifest,
        r#"outcome = "seeded fresh"
[[obligations]]
claim = "new obligation"
proof = "true"
"#,
    )
    .unwrap();

    // Seed with --force
    ritalin()
        .args(["seed", manifest.to_str().unwrap(), "--force"])
        .current_dir(dir)
        .assert()
        .success();

    // Should have exactly 1 obligation (the seeded one), not 2
    let output = ritalin()
        .args(["status", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["data"]["obligations_total"], 1);
}

// ─── Fix: add restores marker after gate ───────────────────

#[test]
fn add_after_gate_restores_marker() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "first", "true");

    // Prove and gate — marker removed
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
    ritalin().args(["gate"]).current_dir(dir).assert().success();
    assert!(!dir.join(".task-incomplete").exists());

    // Add a new critical obligation — marker should be restored
    add_in(dir, "second", "true");
    assert!(
        dir.join(".task-incomplete").exists(),
        ".task-incomplete should be restored after adding critical obligation post-gate"
    );
}

#[test]
fn add_advisory_after_gate_does_not_restore_marker() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    add_in(dir, "first", "true");

    // Prove and gate — marker removed
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
    ritalin().args(["gate"]).current_dir(dir).assert().success();
    assert!(!dir.join(".task-incomplete").exists());

    // Add a non-critical obligation — marker should stay gone
    ritalin()
        .args([
            "add",
            "advisory thing",
            "--proof",
            "echo ok",
            "--critical=false",
        ])
        .current_dir(dir)
        .assert()
        .success();
    assert!(
        !dir.join(".task-incomplete").exists(),
        ".task-incomplete should NOT be restored for advisory obligations"
    );
}

// ─── Fix: advisory warnings in gate output ─────────────────

// ─── prove scope-refresh ───────────────────────────────────

#[test]
fn prove_scope_refresh_lists_remaining_ids() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "first", "true");
    add_in(dir, "second", "true");

    // Prove the first; the second should appear in the refresh.
    let out = ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let remaining = &json["data"]["remaining_open"];
    assert_eq!(remaining["ids"][0], "O-002");
    assert_eq!(remaining["critical"], 1);
    assert_eq!(remaining["advisory"], 0);
}

#[test]
fn prove_json_includes_remaining_open() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "first", "true");
    add_in(dir, "second", "true");

    let out = ritalin()
        .args(["prove", "O-001", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let remaining = &json["data"]["remaining_open"];
    assert_eq!(remaining["critical"], 1);
    assert_eq!(remaining["advisory"], 0);
    assert_eq!(remaining["ids"][0], "O-002");
}

#[test]
fn prove_scope_refresh_empty_when_all_done() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "only", "true");

    let out = ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let remaining = &json["data"]["remaining_open"];
    assert_eq!(remaining["critical"], 0);
    assert_eq!(remaining["advisory"], 0);
    assert!(remaining["ids"].as_array().unwrap().is_empty());
}

#[test]
fn prove_cmd_override_keeps_obligation_in_remaining() {
    // `--cmd` override passes, but the hash won't match the stored proof,
    // so the obligation stays open in the scope-refresh.
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "need real proof", "false");

    let out = ritalin()
        .args(["prove", "O-001", "--cmd", "true", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let remaining = &json["data"]["remaining_open"];
    assert_eq!(remaining["critical"], 1);
    assert_eq!(remaining["ids"][0], "O-001");
}

// ─── export-contract ───────────────────────────────────────

#[test]
fn export_contract_human_contains_role_contract_return_format_and_donts() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    ritalin()
        .args(["init", "--outcome", "Ship notification toggle"])
        .current_dir(dir)
        .assert()
        .success();
    add_in(dir, "UI toggle renders", "true");
    add_in(dir, "POST /api/settings exists", "false");

    let out = ritalin()
        .args(["export-contract"])
        .current_dir(dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("delegated implementation subagent"));
    assert!(stdout.contains("OUTCOME: Ship notification toggle"));
    assert!(stdout.contains("Anti-drift rule"));
    assert!(stdout.contains("O-001"));
    assert!(stdout.contains("O-002"));
    assert!(stdout.contains("Return exactly this format"));
    assert!(stdout.contains("Do not:"));
    assert!(stdout.contains("claim the ritalin gate passed"));
    // Proof commands should be included in the "parent will verify with" block.
    assert!(stdout.contains("How the parent will later verify them:"));
}

#[test]
fn export_contract_json_has_briefing_and_structured_open() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "a critical obligation", "true");

    let out = ritalin()
        .args(["export-contract", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["version"], "1");
    assert_eq!(json["status"], "success");
    let data = &json["data"];
    assert_eq!(data["outcome"], "test outcome");
    assert_eq!(data["obligations_total"], 1);
    assert_eq!(data["remaining_open"]["critical"], 1);
    assert_eq!(data["open_obligations"][0]["id"], "O-001");
    assert_eq!(
        data["open_obligations"][0]["claim"],
        "a critical obligation"
    );
    assert!(data["briefing"].is_string());
    assert!(
        data["briefing"]
            .as_str()
            .unwrap()
            .contains("delegated implementation subagent")
    );
}

#[test]
fn export_contract_no_obligations_emits_none_yet() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    let out = ritalin()
        .args(["export-contract"])
        .current_dir(dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("none yet"));
    assert!(stdout.contains("no open proofs to run"));
}

#[test]
fn export_contract_all_proved_forbids_claiming_gate_passed() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);
    add_in(dir, "trivial", "true");
    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    let out = ritalin()
        .args(["export-contract"])
        .current_dir(dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("every obligation in the ledger has passing evidence"));
    assert!(stdout.contains("Do not claim the ritalin gate passed"));
}

#[test]
fn export_contract_uninitialized_errors() {
    let tmp = TempDir::new().unwrap();
    ritalin()
        .args(["export-contract"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

// ─── Skill file contract (embedded SKILL.md) ───────────────
//
// Verifies directly against the embedded file via include_str!, so we don't
// need to install into a real HOME to test its shape.

#[test]
fn embedded_skill_md_is_under_budget_and_has_directives() {
    let skill = include_str!("../src/skill/SKILL.md");
    let line_count = skill.lines().count();
    assert!(
        line_count <= 130,
        "SKILL.md is {line_count} lines; budget is 130 (Anthropic skill-length research)"
    );
    assert!(
        skill.contains("Approximation drift is a contract breach"),
        "SKILL.md must carry the anti-drift warning in the primacy zone"
    );
    assert!(
        skill.contains("BEFORE"),
        "SKILL.md must use BEFORE/MUST imperative directives"
    );
    assert!(
        !skill.contains("## Why this exists"),
        "SKILL.md's 'Why this exists' rationale moved to README"
    );
}

// ─── literal_match kind ────────────────────────────────────

#[test]
fn literal_match_proves_when_literal_present() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    std::fs::write(
        dir.join("theme.css"),
        ".hero { background: rgba(7,9,7,0.54); }\n",
    )
    .unwrap();

    ritalin()
        .args([
            "add",
            "Hero overlay is verbatim rgba(7,9,7,0.54)",
            "--kind",
            "literal_match",
            "--literal",
            "rgba(7,9,7,0.54)",
            "--file",
            "theme.css",
        ])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();

    ritalin().args(["gate"]).current_dir(dir).assert().success();
}

#[test]
fn literal_match_fails_when_literal_absent() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    // File exists but does not contain the literal.
    std::fs::write(dir.join("theme.css"), ".hero { background: #000; }\n").unwrap();

    ritalin()
        .args([
            "add",
            "Hero overlay is verbatim rgba(7,9,7,0.54)",
            "--kind",
            "literal_match",
            "--literal",
            "rgba(7,9,7,0.54)",
            "--file",
            "theme.css",
        ])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure();

    ritalin().args(["gate"]).current_dir(dir).assert().failure();
}

#[test]
fn literal_match_fails_when_file_missing() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    ritalin()
        .args([
            "add",
            "Value present in not-yet-existing file",
            "--kind",
            "literal_match",
            "--literal",
            "xyz",
            "--file",
            "does-not-exist.css",
        ])
        .current_dir(dir)
        .assert()
        .success(); // add succeeds — ritalin does not require the file to exist yet

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .failure(); // prove fails because grep exits 2 (no such file)
}

#[test]
fn literal_match_handles_literal_with_single_quotes() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    std::fs::write(dir.join("snippet.js"), "const msg = 'it\\'s here';\n").unwrap();

    ritalin()
        .args([
            "add",
            "Literal with apostrophes is safely quoted",
            "--kind",
            "literal_match",
            "--literal",
            "'it\\'s here'",
            "--file",
            "snippet.js",
        ])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn literal_match_handles_literal_starting_with_dash() {
    // Regression test for the `--` guard — literals starting with `-`
    // must not be parsed as grep flags.
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    std::fs::write(
        dir.join("theme.css"),
        ".body { -webkit-font-smoothing: antialiased; }\n",
    )
    .unwrap();

    ritalin()
        .args([
            "add",
            "Webkit smoothing is present",
            "--kind",
            "literal_match",
            "--literal",
            "-webkit-font-smoothing",
            "--file",
            "theme.css",
        ])
        .current_dir(dir)
        .assert()
        .success();

    ritalin()
        .args(["prove", "O-001"])
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn literal_match_without_kind_is_rejected() {
    // Supplying --literal + --file with the default kind (other) should fail:
    // user must be explicit about the kind.
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    ritalin()
        .args([
            "add",
            "something",
            "--literal",
            "x",
            "--file",
            "y",
            // no --kind literal_match
        ])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn literal_match_kind_with_proof_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    ritalin()
        .args([
            "add",
            "something",
            "--kind",
            "literal_match",
            "--proof",
            "true",
        ])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn literal_without_file_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    init_in(dir);

    ritalin()
        .args([
            "add",
            "something",
            "--kind",
            "literal_match",
            "--literal",
            "x",
        ])
        .current_dir(dir)
        .assert()
        .failure();
}

#[test]
fn gate_json_includes_advisory_open_count() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    init_in(dir);
    // Add advisory obligation (not proved)
    ritalin()
        .args([
            "add",
            "advisory check",
            "--proof",
            "echo advisory",
            "--critical=false",
        ])
        .current_dir(dir)
        .assert()
        .success();

    // Gate passes (no critical obligations)
    let output = ritalin()
        .args(["gate", "--json"])
        .current_dir(dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(json["data"]["verdict"], "pass");
    assert_eq!(json["data"]["obligations_open_advisory"], 1);
}
