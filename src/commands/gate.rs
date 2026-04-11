//! The completion gate. This is the load-bearing piece.
//!
//! When called as a Claude Code Stop hook with `--hook-mode`, it reads
//! `stop_hook_active` from stdin to prevent infinite loops, then either
//! emits a `{"decision": "block", "reason": "..."}` JSON to stdout
//! (which Claude Code interprets as "keep working") or exits cleanly
//! to allow the stop.
//!
//! When called from a terminal without `--hook-mode`, it returns the
//! framework JSON envelope or a coloured human report.

use serde::Serialize;
use std::io::{IsTerminal, Read};

use crate::error::AppError;
use crate::ledger::{evidence, is_initialized, marker, obligations, state_dir};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct StopHookDecision {
    decision: &'static str,
    reason: String,
}

#[derive(Serialize)]
struct GateResult {
    verdict: &'static str,
    obligations_total: usize,
    obligations_open_critical: usize,
    blocking_obligation: Option<String>,
    blocking_reason: Option<String>,
}

fn read_stop_hook_active() -> bool {
    if std::io::stdin().is_terminal() {
        return false;
    }
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return false;
    }
    if buf.trim().is_empty() {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(&buf)
        .ok()
        .and_then(|v| v.get("stop_hook_active").and_then(|x| x.as_bool()))
        .unwrap_or(false)
}

pub fn run(ctx: Ctx, hook_mode: bool) -> Result<(), AppError> {
    // Critical: respect stop_hook_active to break out of forced-continuation cycles.
    if hook_mode && read_stop_hook_active() {
        // Allow stop. Empty stdout = no decision = Claude Code stops normally.
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        // Not initialized = nothing to enforce. Allow stop.
        if hook_mode {
            return Ok(());
        }
        return Err(AppError::NotInitialized);
    }

    let dir = state_dir(&cwd);
    let obs = obligations::read_all(&dir)?;
    let evidence_index = evidence::index_by_obligation(&dir)?;

    let mut open_critical = Vec::new();
    for ob in &obs {
        if !ob.critical {
            continue;
        }
        let discharged = evidence_index
            .get(&ob.id)
            .map(|recs| evidence::is_discharged(recs))
            .unwrap_or(false);
        if !discharged {
            open_critical.push(ob);
        }
    }

    if open_critical.is_empty() {
        // PASS: every critical obligation has at least one passing proof.
        marker::remove(&dir)?;

        if hook_mode {
            // Claude Code: empty stdout = allow stop.
            return Ok(());
        }

        let result = GateResult {
            verdict: "pass",
            obligations_total: obs.len(),
            obligations_open_critical: 0,
            blocking_obligation: None,
            blocking_reason: None,
        };
        output::print_success_or(ctx, &result, |r| {
            use owo_colors::OwoColorize;
            println!(
                "{} all {} critical obligations discharged",
                "PASS".green().bold(),
                r.obligations_total
            );
            println!("  {} removed", ".task-incomplete".dimmed());
        });
        return Ok(());
    }

    // FAIL: at least one critical obligation lacks evidence.
    let blocking = open_critical[0];
    let reason = format!(
        "Obligation {} ({}) lacks passing evidence. Run: `ritalin prove {} --cmd \"{}\"` (or fix the proof and re-run).",
        blocking.id, blocking.claim, blocking.id, blocking.proof_cmd
    );

    if hook_mode {
        // Emit Claude Code stop hook JSON. Stdout JSON is what triggers the block.
        let decision = StopHookDecision {
            decision: "block",
            reason: reason.clone(),
        };
        // Hook output goes through stdout regardless of TTY detection — Claude Code reads it.
        println!("{}", output::safe_json_string(&decision));
        return Ok(());
    }

    let result = GateResult {
        verdict: "fail",
        obligations_total: obs.len(),
        obligations_open_critical: open_critical.len(),
        blocking_obligation: Some(blocking.id.clone()),
        blocking_reason: Some(reason.clone()),
    };
    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        println!(
            "{} {} of {} critical obligations open",
            "FAIL".red().bold(),
            r.obligations_open_critical,
            r.obligations_total
        );
        if let Some(reason) = &r.blocking_reason {
            println!("  {}", reason.dimmed());
        }
    });

    // Non-zero exit when called from CLI so users see it failed.
    Err(AppError::VerificationFailed(reason))
}
