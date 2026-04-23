use chrono::Utc;
use serde::Serialize;
use std::process::Command;

use crate::error::AppError;
use crate::gate_eval;
use crate::ledger::{
    evidence, evidence::Evidence, is_initialized, obligations, state_dir, workspace_hash,
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
    discharged: bool,
    stdout_tail: String,
    stderr_tail: String,
    remaining_open: RemainingOpen,
}

pub fn run(ctx: Ctx, id: String, cmd: Option<String>) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    let ob = obligations::find(&dir, &id)?;
    let command = cmd.unwrap_or_else(|| ob.proof_cmd.clone());

    // Run via shell so users can pass pipes, redirects, env vars, etc.
    let output_res = Command::new("sh").arg("-c").arg(&command).output()?;

    let exit_code = output_res.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output_res.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output_res.stderr).to_string();

    let proof_hash = evidence::proof_hash(&command);
    let project_root = dir.parent().unwrap_or(&cwd);
    let ws_hash = workspace_hash::compute(project_root).unwrap_or_default();

    let ev = Evidence {
        obligation_id: id.clone(),
        command: command.clone(),
        exit_code,
        stdout_tail: tail(&stdout),
        stderr_tail: tail(&stderr),
        proof_hash,
        workspace_hash: ws_hash.clone(),
        recorded_at: Utc::now(),
    };
    evidence::append(&dir, &ev)?;

    // Compute the scope-refresh *after* appending evidence, so this run's
    // result (and any `--cmd` hash mismatch) is reflected. This is the
    // recency-injection line the agent sees in its high-attention zone.
    let all_obs = obligations::read_all(&dir)?;
    let evidence_index = evidence::index_by_obligation(&dir)?;
    let eval = gate_eval::evaluate(&all_obs, &evidence_index, &ws_hash);
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

    let discharged = exit_code == 0;
    let result = ProveResult {
        obligation_id: id,
        command,
        exit_code,
        discharged,
        stdout_tail: ev.stdout_tail.clone(),
        stderr_tail: ev.stderr_tail.clone(),
        remaining_open,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        let badge = if r.discharged {
            "PASS".green().bold().to_string()
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
        // Scope refresh — recency-zone anchor for the remaining contract.
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

    if !discharged {
        return Err(AppError::VerificationFailed(format!(
            "proof command exited {} for {}",
            exit_code, result.obligation_id
        )));
    }

    Ok(())
}
