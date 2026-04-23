//! `ritalin export-contract` — emit a subagent-ready briefing.
//!
//! Subagents spawned via Claude Code's Task/Agent tool run in isolated context
//! (own system prompt, own context window, depth=1, only the summary returns).
//! They cannot see `.ritalin/` state unless you tell them. This command produces
//! a ready-to-paste text block the parent agent includes in the delegation
//! prompt so the subagent stays inside the parent's contract.
//!
//! Zero-arg. Read-only. Does not touch the ledger, marker, or scope.

use serde::Serialize;

use crate::error::AppError;
use crate::gate_eval;
use crate::ledger::{
    evidence, is_initialized, obligations, obligations::Obligation, scope::Scope, state_dir,
    workspace_hash,
};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct OpenObligation {
    id: String,
    claim: String,
    kind: String,
    critical: bool,
    proof_cmd: String,
    last_exit_code: Option<i32>,
}

#[derive(Serialize)]
struct RemainingOpen {
    ids: Vec<String>,
    critical: usize,
    advisory: usize,
}

#[derive(Serialize)]
struct ExportContractResult {
    outcome: String,
    obligations_total: usize,
    remaining_open: RemainingOpen,
    open_obligations: Vec<OpenObligation>,
    briefing: String,
}

pub fn run(ctx: Ctx) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    let scope = Scope::read(&dir)?;
    let all_obs = obligations::read_all(&dir)?;
    let evidence_index = evidence::index_by_obligation(&dir)?;
    let project_root = dir.parent().unwrap_or(&cwd);
    let ws_hash = workspace_hash::compute(project_root).unwrap_or_default();
    let eval = gate_eval::evaluate(&all_obs, &evidence_index, &ws_hash);

    // Merge open critical + open advisory, preserving the add-order of obligations.
    let open_refs: Vec<&Obligation> = eval
        .open_critical
        .iter()
        .copied()
        .chain(eval.open_advisory.iter().copied())
        .collect();

    let open_obligations: Vec<OpenObligation> = open_refs
        .iter()
        .map(|ob| {
            let last_exit = evidence_index
                .get(&ob.id)
                .and_then(|recs| recs.last().map(|e| e.exit_code));
            OpenObligation {
                id: ob.id.clone(),
                claim: ob.claim.clone(),
                kind: ob.kind.to_string(),
                critical: ob.critical,
                proof_cmd: ob.proof_cmd.clone(),
                last_exit_code: last_exit,
            }
        })
        .collect();

    let remaining_open = RemainingOpen {
        ids: open_obligations.iter().map(|o| o.id.clone()).collect(),
        critical: eval.open_critical.len(),
        advisory: eval.open_advisory.len(),
    };

    let briefing = render_briefing(
        &scope.outcome,
        all_obs.len(),
        &remaining_open,
        &open_obligations,
    );

    let result = ExportContractResult {
        outcome: scope.outcome,
        obligations_total: all_obs.len(),
        remaining_open,
        open_obligations,
        briefing,
    };

    output::print_success_or(ctx, &result, |r| {
        // Human mode: print the briefing raw so it can be copied straight
        // into a Task/Agent delegation prompt. No colour, no adornment.
        println!("{}", r.briefing);
    });

    Ok(())
}

/// Render the subagent briefing text.
///
/// Three distinct states are flagged explicitly so the subagent is not
/// misled:
///   • no obligations added yet          → still forbid claiming completion
///   • obligations exist, all discharged → still forbid claiming gate passed
///   • open obligations remain           → list them with claims + proofs
fn render_briefing(
    outcome: &str,
    obligations_total: usize,
    remaining: &RemainingOpen,
    open: &[OpenObligation],
) -> String {
    let summary_line = if obligations_total == 0 {
        "Open obligations: none yet (the parent has not added any obligations to the ledger)."
            .to_string()
    } else if remaining.ids.is_empty() {
        "Open obligations: none — every obligation in the ledger has passing evidence. Do not claim the ritalin gate passed; let the parent run `ritalin gate`.".to_string()
    } else {
        format!(
            "Open obligations: {} ({} critical, {} advisory).",
            remaining.ids.join(", "),
            remaining.critical,
            remaining.advisory
        )
    };

    let list_block = if open.is_empty() {
        String::new()
    } else {
        let mut s = String::from("\n");
        for o in open {
            let crit = if o.critical { "critical" } else { "advisory" };
            s.push_str(&format!(
                "- {id} [{kind}, {crit}]: {claim}\n",
                id = o.id,
                kind = o.kind,
                crit = crit,
                claim = o.claim
            ));
        }
        s
    };

    let proofs_block = if open.is_empty() {
        "(no open proofs to run)".to_string()
    } else {
        let mut s = String::new();
        for o in open {
            s.push_str(&format!("- {}: `{}`\n", o.id, o.proof_cmd));
        }
        s.trim_end().to_string()
    };

    format!(
        "You are a delegated implementation subagent working under a parent agent. Treat this briefing as the current contract snapshot for your task.\n\
         \n\
         Your job is to reduce the parent's uncertainty by making the codebase more true with respect to this outcome, using source-backed reasoning and edits that help the parent satisfy the existing contract:\n\
         \n\
         OUTCOME: {outcome}\n\
         \n\
         **Anti-drift rule:** if you have not read it in this subagent turn, do not state it as a fact. Approximation drift is a contract breach — this applies to visual properties, file contents, API shapes, config values, and version numbers.\n\
         \n\
         {summary_line}{list_block}\n\
         How the parent will later verify them:\n\
         {proofs_block}\n\
         \n\
         Return exactly this format:\n\
         1. Changes made (files, functions, lines).\n\
         2. Facts you checked from source in this turn (with file:line cites).\n\
         3. Which open obligations your changes help satisfy (by ID).\n\
         4. Remaining blockers or uncertainties.\n\
         5. Suggested proof runs for the parent agent.\n\
         \n\
         Do not:\n\
         - edit `.ritalin/` or `.task-incomplete`\n\
         - claim the ritalin gate passed\n\
         - add, delete, reorder, or rewrite obligations\n\
         - mark the task done unless your changes actually satisfy the listed obligations\n\
         - invent tests, commands, or technical facts you did not inspect\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(id: &str, claim: &str, critical: bool, proof: &str) -> OpenObligation {
        OpenObligation {
            id: id.into(),
            claim: claim.into(),
            kind: "other".into(),
            critical,
            proof_cmd: proof.into(),
            last_exit_code: None,
        }
    }

    #[test]
    fn briefing_lists_open_obligations_with_proofs() {
        let remaining = RemainingOpen {
            ids: vec!["O-001".into(), "O-002".into()],
            critical: 1,
            advisory: 1,
        };
        let open = vec![
            open(
                "O-001",
                "Hero overlay exact colour",
                true,
                "grep -F -- 'rgba' theme.css",
            ),
            open(
                "O-002",
                "Docs up-to-date",
                false,
                "test -f docs/changelog.md",
            ),
        ];
        let out = render_briefing("Ship v0.3.1", 2, &remaining, &open);
        assert!(out.contains("OUTCOME: Ship v0.3.1"));
        assert!(out.contains("Open obligations: O-001, O-002"));
        assert!(out.contains("O-001 [other, critical]: Hero overlay exact colour"));
        assert!(out.contains("O-002 [other, advisory]: Docs up-to-date"));
        assert!(out.contains("grep -F -- 'rgba' theme.css"));
        assert!(out.contains("Anti-drift rule"));
        assert!(out.contains("Do not:"));
        assert!(out.contains("claim the ritalin gate passed"));
    }

    #[test]
    fn briefing_handles_no_obligations() {
        let remaining = RemainingOpen {
            ids: Vec::new(),
            critical: 0,
            advisory: 0,
        };
        let out = render_briefing("Fresh start", 0, &remaining, &[]);
        assert!(out.contains("Open obligations: none yet"));
        assert!(out.contains("(no open proofs to run)"));
    }

    #[test]
    fn briefing_handles_all_proved() {
        let remaining = RemainingOpen {
            ids: Vec::new(),
            critical: 0,
            advisory: 0,
        };
        let out = render_briefing("Done work", 5, &remaining, &[]);
        assert!(out.contains("every obligation in the ledger has passing evidence"));
        assert!(out.contains("Do not claim the ritalin gate passed"));
    }
}
