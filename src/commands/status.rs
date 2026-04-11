use serde::Serialize;

use crate::error::AppError;
use crate::ledger::{evidence, is_initialized, marker, obligations, scope::Scope, state_dir};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct ObligationStatus {
    id: String,
    claim: String,
    kind: String,
    critical: bool,
    discharged: bool,
    proof_cmd: String,
    last_exit_code: Option<i32>,
}

#[derive(Serialize)]
struct StatusResult {
    outcome: String,
    marker_present: bool,
    obligations_total: usize,
    obligations_critical: usize,
    obligations_open_critical: usize,
    obligations: Vec<ObligationStatus>,
}

pub fn run(ctx: Ctx) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    let scope = Scope::read(&dir)?;
    let obs = obligations::read_all(&dir)?;
    let evidence_index = evidence::index_by_obligation(&dir)?;

    let mut entries: Vec<ObligationStatus> = Vec::with_capacity(obs.len());
    let mut critical_total = 0;
    let mut open_critical = 0;
    for ob in &obs {
        let recs = evidence_index.get(&ob.id);
        let discharged = recs.map(|r| evidence::is_discharged(r)).unwrap_or(false);
        let last_exit = recs.and_then(|r| r.last().map(|e| e.exit_code));

        if ob.critical {
            critical_total += 1;
            if !discharged {
                open_critical += 1;
            }
        }
        entries.push(ObligationStatus {
            id: ob.id.clone(),
            claim: ob.claim.clone(),
            kind: ob.kind.to_string(),
            critical: ob.critical,
            discharged,
            proof_cmd: ob.proof_cmd.clone(),
            last_exit_code: last_exit,
        });
    }

    let result = StatusResult {
        outcome: scope.outcome,
        marker_present: marker::exists(&dir),
        obligations_total: obs.len(),
        obligations_critical: critical_total,
        obligations_open_critical: open_critical,
        obligations: entries,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        println!("{} {}", "outcome:".bold(), r.outcome);
        let marker_label = if r.marker_present {
            ".task-incomplete present".red().to_string()
        } else {
            ".task-incomplete absent".green().to_string()
        };
        println!("{}", marker_label);
        println!(
            "obligations: {} total, {} critical, {} open critical\n",
            r.obligations_total, r.obligations_critical, r.obligations_open_critical
        );

        if r.obligations.is_empty() {
            println!(
                "  {} no obligations yet — add one with `ritalin add`",
                "·".dimmed()
            );
            return;
        }

        let mut table = comfy_table::Table::new();
        table.set_header(vec!["id", "kind", "critical", "discharged", "claim"]);
        for o in &r.obligations {
            table.add_row(vec![
                o.id.clone(),
                o.kind.clone(),
                if o.critical {
                    "yes".red().to_string()
                } else {
                    "no".dimmed().to_string()
                },
                if o.discharged {
                    "yes".green().to_string()
                } else {
                    "no".red().to_string()
                },
                o.claim.clone(),
            ]);
        }
        println!("{table}");
    });

    Ok(())
}
