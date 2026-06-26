use serde::Serialize;
use std::io::{IsTerminal, Read};

use crate::error::AppError;
use crate::gate_eval::{self, Verdict};
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
    obligations_open_advisory: usize,
    blocking_obligation: Option<String>,
    blocking_reason: Option<String>,
}

fn print_hook_block(reason: String) {
    let decision = StopHookDecision {
        decision: "block",
        reason,
    };
    println!("{}", output::safe_json_string(&decision));
}

/// Returns true if `RITALIN_GATE` is explicitly set to a disable value
/// (`0`, `off`, `false`, `no`, `disable`, `disabled` — case-insensitive).
///
/// This lets a session opt OUT of the Stop-hook gate. One-shot reviewers,
/// auditors, and CI runs that do not own the contract set `RITALIN_GATE=0` so
/// the gate stops them cleanly instead of hijacking their termination into
/// `ritalin prove` bookkeeping. Unset leaves the gate active, so existing Stop
/// hooks keep working unchanged.
fn hook_disabled_by_env() -> bool {
    match std::env::var("RITALIN_GATE") {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no" | "disable" | "disabled"
        ),
        Err(_) => false,
    }
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

pub fn run(ctx: Ctx, hook_mode: bool, summary: bool) -> Result<(), AppError> {
    // `RITALIN_GATE=0` (or off/false/no/disable/disabled) opts this session out of the
    // gate: a non-owning reviewer/auditor/CI run stops cleanly instead of being
    // hijacked. Checked before reading stdin so it short-circuits regardless of
    // the hook payload. Only affects hook mode — a manual `ritalin gate` always
    // reports the true verdict.
    if hook_mode && (hook_disabled_by_env() || read_stop_hook_active()) {
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
    let loaded = (|| {
        let obs = obligations::read_all(&dir)?;
        let evidence_index = evidence::index_by_obligation(&dir)?;
        let project_root = dir.parent().unwrap_or(&cwd);
        let scope_hashes = gate_eval::compute_scope_hashes(&obs, project_root)?;
        Ok::<_, AppError>((obs, evidence_index, scope_hashes))
    })();

    let (obs, evidence_index, scope_hashes) = match loaded {
        Ok(loaded) => loaded,
        Err(err) if hook_mode => {
            print_hook_block(format!(
                "ritalin gate could not verify the contract: {err}. {}",
                err.suggestion()
            ));
            let _ = marker::create(
                &dir,
                &format!("ritalin: gate blocked — could not verify contract: {err}\n"),
            );
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    let eval = gate_eval::evaluate(&obs, &evidence_index, &scope_hashes);

    match eval.verdict {
        Verdict::Pass => {
            marker::remove(&dir)?;

            if hook_mode {
                return Ok(());
            }

            if summary {
                println!(
                    "verdict=pass critical_open=0 advisory_open={} total={}",
                    eval.open_advisory.len(),
                    eval.obligations_total
                );
                return Ok(());
            }

            let advisory_open = eval.open_advisory.len();
            let result = GateResult {
                verdict: "pass",
                obligations_total: eval.obligations_total,
                obligations_open_critical: 0,
                obligations_open_advisory: advisory_open,
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
                if r.obligations_open_advisory > 0 {
                    println!(
                        "  {} {} advisory obligations lack evidence",
                        "WARN".yellow().bold(),
                        r.obligations_open_advisory
                    );
                }
                println!("  {} removed", ".task-incomplete".dimmed());
            });
            Ok(())
        }
        Verdict::Empty => {
            let reason =
                "No obligations defined. Add obligations with `ritalin add` before gating."
                    .to_string();

            if hook_mode {
                let _ = marker::create(&dir, &format!("ritalin: gate blocked — {reason}\n"));
                print_hook_block(reason.clone());
                return Ok(());
            }
            marker::create(&dir, &format!("ritalin: gate blocked — {reason}\n"))?;

            if summary {
                println!("verdict=fail critical_open=0 advisory_open=0 total=0 blocking=empty");
                return Err(AppError::VerificationFailed(reason));
            }

            let result = GateResult {
                verdict: "fail",
                obligations_total: 0,
                obligations_open_critical: 0,
                obligations_open_advisory: 0,
                blocking_obligation: None,
                blocking_reason: Some(reason.clone()),
            };

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
                        println!("{} {}", "FAIL".red().bold(), reason);
                    }
                    Err(AppError::VerificationFailed(reason))
                }
            }
        }
        Verdict::Fail => {
            let blocking = eval.open_critical[0];
            let expected_ph = evidence::proof_hash(&blocking.proof_cmd);
            let blocking_scope = scope_hashes
                .get(&blocking.id)
                .map(String::as_str)
                .unwrap_or("");
            let evidence_state = evidence::classify(
                evidence_index.get(&blocking.id).map(Vec::as_slice),
                &expected_ph,
                blocking_scope,
            );
            let reason = format!(
                "Obligation {} ({}) lacks passing evidence: {}. Run: `ritalin prove {}` (or fix the proof and re-run).",
                blocking.id,
                blocking.claim,
                evidence_state.explanation(),
                blocking.id
            );

            if hook_mode {
                let _ = marker::create(&dir, &format!("ritalin: gate blocked — {reason}\n"));
                print_hook_block(reason.clone());
                return Ok(());
            }
            marker::create(&dir, &format!("ritalin: gate blocked — {reason}\n"))?;

            if summary {
                println!(
                    "verdict=fail critical_open={} advisory_open={} total={} blocking={}",
                    eval.open_critical.len(),
                    eval.open_advisory.len(),
                    eval.obligations_total,
                    blocking.id
                );
                return Err(AppError::VerificationFailed(reason));
            }

            let result = GateResult {
                verdict: "fail",
                obligations_total: eval.obligations_total,
                obligations_open_critical: eval.open_critical.len(),
                obligations_open_advisory: eval.open_advisory.len(),
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
