use chrono::Utc;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;
use crate::gate_eval;
use crate::ledger::{
    evidence, evidence::Evidence, is_initialized, obligations, obligations::Obligation, state_dir,
    workspace_hash,
};
use crate::output::{self, Ctx};

const TAIL_LIMIT: usize = 2000;

fn tail(s: &str) -> String {
    if s.len() <= TAIL_LIMIT {
        s.to_string()
    } else {
        // Find a valid UTF-8 char boundary near the desired start position.
        let desired_start = s.len() - TAIL_LIMIT;
        let start = s
            .char_indices()
            .map(|(i, _)| i)
            .find(|&i| i >= desired_start)
            .unwrap_or(desired_start);
        format!("…{}", &s[start..])
    }
}

/// Scope-refresh: which obligations are still open after this `prove` call.
///
/// Recomputed against the freshly-appended evidence ledger, so `--cmd` overrides
/// (hash mismatch) and failed proofs correctly keep their obligations in the
/// remaining list. Critical and advisory are split so agents can distinguish
/// "gate would block" from "gate would pass but advisories are open".
#[derive(Serialize)]
struct RemainingOpen {
    ids: Vec<String>,
    critical: usize,
    advisory: usize,
}

#[derive(Serialize)]
struct ProveResult {
    obligation_id: String,
    command: String,
    exit_code: i32,
    command_passed: bool,
    discharged: bool,
    evidence_status: String,
    stdout_tail: String,
    stderr_tail: String,
    workspace_mutated: bool,
    remaining_open: RemainingOpen,
}

/// Result for a single skipped obligation in `--all --stale-only` mode.
#[derive(Serialize)]
struct SkippedResult {
    obligation_id: String,
    reason: &'static str,
}

#[derive(Serialize)]
struct ProveAllSummary {
    total: usize,
    discharged: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Serialize)]
struct ProveAllResult {
    proved: Vec<ProveResult>,
    skipped: Vec<SkippedResult>,
    summary: ProveAllSummary,
}

pub fn run(
    ctx: Ctx,
    id: Option<String>,
    cmd: Option<String>,
    all: bool,
    stale_only: bool,
) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    if all {
        run_all(ctx, &dir, &cwd, stale_only)
    } else {
        let id = id.expect("clap requires id when --all is not set");
        let ob = obligations::find(&dir, &id)?;
        let project_root = dir.parent().unwrap_or(&cwd).to_path_buf();
        let result = prove_one(&dir, &project_root, &ob, cmd)?;
        let command_passed = result.command_passed;
        emit_one(ctx, &result);
        if !command_passed {
            return Err(AppError::VerificationFailed(format!(
                "proof command exited {} for {}",
                result.exit_code, result.obligation_id
            )));
        }
        Ok(())
    }
}

/// Run a single obligation's proof, append evidence, and return the result.
/// Does not print or exit; callers handle output and exit semantics.
fn prove_one(
    dir: &Path,
    project_root: &Path,
    ob: &Obligation,
    cmd_override: Option<String>,
) -> Result<ProveResult, AppError> {
    let command = cmd_override.unwrap_or_else(|| ob.proof_cmd.clone());

    // Capture pre-execution scope hash so we can detect proofs that mutate
    // their own dependency files (formatters, codegen) — those cascade
    // into stale evidence for everything else.
    let pre_ws_hash = workspace_hash::compute_for(project_root, &ob.depends_on)?;

    // Run via shell so users can pass pipes, redirects, env vars, etc.
    let output_res = Command::new("sh").arg("-c").arg(&command).output()?;

    let exit_code = output_res.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output_res.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output_res.stderr).to_string();

    let proof_hash = evidence::proof_hash(&command);
    let post_ws_hash = workspace_hash::compute_for(project_root, &ob.depends_on)?;
    let workspace_mutated = pre_ws_hash != post_ws_hash;

    let ev = Evidence {
        obligation_id: ob.id.clone(),
        command: command.clone(),
        exit_code,
        stdout_tail: tail(&stdout),
        stderr_tail: tail(&stderr),
        proof_hash,
        // Record against the post-execution hash — that's the workspace
        // state the evidence is actually bound to. If the proof mutated
        // its dependencies, downstream gate calls compare against the new
        // post-state, so the record stays fresh as long as nothing else
        // changes.
        workspace_hash: post_ws_hash.clone(),
        recorded_at: Utc::now(),
    };
    evidence::append(dir, &ev)?;

    // Scope-refresh: rebuild evaluation against the just-appended ledger.
    let all_obs = obligations::read_all(dir)?;
    let evidence_index = evidence::index_by_obligation(dir)?;
    let scope_hashes = gate_eval::compute_scope_hashes(&all_obs, project_root)?;
    let eval = gate_eval::evaluate(&all_obs, &evidence_index, &scope_hashes);
    let stored_proof_hash = evidence::proof_hash(&ob.proof_cmd);
    let records = evidence_index.get(&ob.id).map(Vec::as_slice);
    let evidence_status = evidence::classify(records, &stored_proof_hash, &post_ws_hash);
    let remaining_open = RemainingOpen {
        ids: eval
            .open_critical
            .iter()
            .chain(eval.open_advisory.iter())
            .map(|o| o.id.clone())
            .collect(),
        critical: eval.open_critical.len(),
        advisory: eval.open_advisory.len(),
    };

    let command_passed = exit_code == 0;
    let discharged = matches!(evidence_status, evidence::EvidenceState::Passed);
    Ok(ProveResult {
        obligation_id: ob.id.clone(),
        command,
        exit_code,
        command_passed,
        discharged,
        evidence_status: evidence_status.as_str().to_string(),
        stdout_tail: ev.stdout_tail,
        stderr_tail: ev.stderr_tail,
        workspace_mutated,
        remaining_open,
    })
}

/// Print a single ProveResult in human or JSON form.
fn emit_one(ctx: Ctx, result: &ProveResult) {
    output::print_success_or(ctx, result, |r| {
        use owo_colors::OwoColorize;
        let badge = if r.discharged {
            "PASS".green().bold().to_string()
        } else if r.command_passed {
            "OPEN".yellow().bold().to_string()
        } else {
            "FAIL".red().bold().to_string()
        };
        println!(
            "{} {} (exit {})",
            badge,
            r.obligation_id.bold(),
            r.exit_code
        );
        println!("  cmd: {}", r.command.dimmed());
        if !r.stderr_tail.is_empty() {
            println!("  stderr: {}", r.stderr_tail.dimmed());
        }
        if !r.discharged {
            println!("  evidence: {}", r.evidence_status.dimmed());
        }
        if r.workspace_mutated {
            // The proof rewrote a file it depends on — formatters, codegen,
            // etc. Other obligations sharing those files may now be stale.
            println!(
                "  {} proof mutated workspace; other obligations may be stale",
                "WARN".yellow().bold()
            );
        }
        let refresh = if r.remaining_open.ids.is_empty() {
            format!(
                "  remaining: none ({} critical, {} advisory — gate ready)",
                r.remaining_open.critical, r.remaining_open.advisory
            )
        } else {
            format!(
                "  remaining: {} ({} critical, {} advisory)",
                r.remaining_open.ids.join(", "),
                r.remaining_open.critical,
                r.remaining_open.advisory,
            )
        };
        println!("{}", refresh.dimmed());
    });
}

/// Re-prove every obligation in add-order. With `stale_only`, skip
/// obligations whose evidence is already passing+fresh.
fn run_all(ctx: Ctx, dir: &Path, cwd: &Path, stale_only: bool) -> Result<(), AppError> {
    let project_root = dir.parent().unwrap_or(cwd).to_path_buf();
    let obs = obligations::read_all(dir)?;

    // Pre-snapshot scope hashes for the stale-only filter. Use today's
    // ledger view so we skip obligations that the user has already proven
    // since the last commit.
    let evidence_index = evidence::index_by_obligation(dir)?;
    let scope_hashes = gate_eval::compute_scope_hashes(&obs, &project_root)?;

    let mut proved: Vec<ProveResult> = Vec::new();
    let mut skipped: Vec<SkippedResult> = Vec::new();
    let mut failed: usize = 0;

    for ob in &obs {
        if stale_only {
            let scope = scope_hashes.get(&ob.id).map(String::as_str).unwrap_or("");
            let expected = evidence::proof_hash(&ob.proof_cmd);
            let recs = evidence_index.get(&ob.id).map(Vec::as_slice);
            let state = evidence::classify(recs, &expected, scope);
            if matches!(state, evidence::EvidenceState::Passed) {
                skipped.push(SkippedResult {
                    obligation_id: ob.id.clone(),
                    reason: "evidence already passing and fresh",
                });
                continue;
            }
        }
        match prove_one(dir, &project_root, ob, None) {
            Ok(r) => {
                if !r.command_passed {
                    failed += 1;
                }
                proved.push(r);
            }
            Err(e) => {
                // Surface IO / config failures as a synthesized "failed"
                // record so the summary stays consistent.
                failed += 1;
                proved.push(ProveResult {
                    obligation_id: ob.id.clone(),
                    command: ob.proof_cmd.clone(),
                    exit_code: -1,
                    command_passed: false,
                    discharged: false,
                    evidence_status: format!("error:{}", e.error_code()),
                    stdout_tail: String::new(),
                    stderr_tail: e.to_string(),
                    workspace_mutated: false,
                    remaining_open: RemainingOpen {
                        ids: Vec::new(),
                        critical: 0,
                        advisory: 0,
                    },
                });
            }
        }
    }

    let discharged = proved.iter().filter(|r| r.discharged).count();
    let summary = ProveAllSummary {
        total: obs.len(),
        discharged,
        failed,
        skipped: skipped.len(),
    };
    let result = ProveAllResult {
        proved,
        skipped,
        summary,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        for p in &r.proved {
            let badge = if p.discharged {
                "PASS".green().bold().to_string()
            } else if p.command_passed {
                "OPEN".yellow().bold().to_string()
            } else {
                "FAIL".red().bold().to_string()
            };
            println!("{} {}", badge, p.obligation_id.bold());
            if !p.discharged && !p.stderr_tail.is_empty() {
                println!("  stderr: {}", p.stderr_tail.dimmed());
            }
            if p.workspace_mutated {
                println!("  {} proof mutated workspace", "WARN".yellow().bold());
            }
        }
        for s in &r.skipped {
            println!(
                "{} {}  ({})",
                "SKIP".dimmed(),
                s.obligation_id,
                s.reason.dimmed()
            );
        }
        println!();
        println!(
            "summary: {} discharged, {} failed, {} skipped of {} total",
            r.summary.discharged.to_string().green(),
            r.summary.failed.to_string().red(),
            r.summary.skipped.to_string().dimmed(),
            r.summary.total
        );
    });

    if failed > 0 {
        return Err(AppError::VerificationFailed(format!(
            "{} of {} obligations failed",
            failed,
            obs.len()
        )));
    }
    Ok(())
}
