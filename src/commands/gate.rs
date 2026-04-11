use serde::Serialize;
use std::io::{IsTerminal, Read};

use crate::error::AppError;
use crate::gate_eval::{self, Verdict};
use crate::ledger::{evidence, is_initialized, marker, obligations, state_dir, workspace_hash};
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
    if hook_mode && read_stop_hook_active() {
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        if hook_mode {
            return Ok(());
        }
        return Err(AppError::NotInitialized);
    }

    let dir = state_dir(&cwd);
    let obs = obligations::read_all(&dir)?;
    let evidence_index = evidence::index_by_obligation(&dir)?;
    let current_ws_hash = workspace_hash::compute(&cwd).unwrap_or_default();

    let eval = gate_eval::evaluate(&obs, &evidence_index, &current_ws_hash);

    match eval.verdict {
        Verdict::Pass => {
            marker::remove(&dir)?;

            if hook_mode {
                return Ok(());
            }

            let result = GateResult {
                verdict: "pass",
                obligations_total: eval.obligations_total,
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
            Ok(())
        }
        Verdict::Fail => {
            let blocking = eval.open_critical[0];
            let reason = format!(
                "Obligation {} ({}) lacks passing evidence. Run: `ritalin prove {} --cmd \"{}\"` (or fix the proof and re-run).",
                blocking.id, blocking.claim, blocking.id, blocking.proof_cmd
            );

            if hook_mode {
                let decision = StopHookDecision {
                    decision: "block",
                    reason: reason.clone(),
                };
                println!("{}", output::safe_json_string(&decision));
                return Ok(());
            }

            let result = GateResult {
                verdict: "fail",
                obligations_total: eval.obligations_total,
                obligations_open_critical: eval.open_critical.len(),
                blocking_obligation: Some(blocking.id.clone()),
                blocking_reason: Some(reason.clone()),
            };

            // For JSON mode, emit one fail envelope to stdout and exit directly.
            // This avoids the double-output problem where print_success_or emits
            // a "success" envelope and then main.rs emits an "error" envelope.
            match ctx.format {
                output::Format::Json => {
                    let envelope = serde_json::json!({
                        "version": "1",
                        "status": "fail",
                        "data": result,
                    });
                    println!("{}", output::safe_json_string(&envelope));
                    std::process::exit(1);
                }
                output::Format::Human => {
                    if !ctx.quiet {
                        use owo_colors::OwoColorize;
                        println!(
                            "{} {} of {} critical obligations open",
                            "FAIL".red().bold(),
                            result.obligations_open_critical,
                            result.obligations_total
                        );
                        if let Some(reason) = &result.blocking_reason {
                            println!("  {}", reason.dimmed());
                        }
                    }
                    Err(AppError::VerificationFailed(reason))
                }
            }
        }
    }
}
