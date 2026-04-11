use chrono::Utc;
use serde::Serialize;

use crate::cli::ObligationKind;
use crate::error::AppError;
use crate::ledger::{is_initialized, obligations, obligations::Obligation, state_dir};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct AddResult {
    id: String,
    claim: String,
    kind: String,
    critical: bool,
    proof_cmd: String,
}

pub fn run(
    ctx: Ctx,
    claim: String,
    proof: String,
    kind: ObligationKind,
    critical: bool,
) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    let claim = claim.trim().to_string();
    if claim.is_empty() {
        return Err(AppError::InvalidInput("claim cannot be empty".into()));
    }
    let proof = proof.trim().to_string();
    if proof.is_empty() {
        return Err(AppError::InvalidInput("proof command cannot be empty".into()));
    }

    let id = obligations::next_id(&dir)?;
    let ob = Obligation {
        id: id.clone(),
        claim: claim.clone(),
        kind,
        critical,
        proof_cmd: proof.clone(),
        created_at: Utc::now(),
    };
    obligations::append(&dir, &ob)?;

    let result = AddResult {
        id,
        claim,
        kind: kind.to_string(),
        critical,
        proof_cmd: proof,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        let crit = if r.critical {
            "[critical]".red().to_string()
        } else {
            "[advisory]".dimmed().to_string()
        };
        println!("{} {} {} {}", "+".green().bold(), r.id.bold(), crit, r.claim);
        println!("  proof: {}", r.proof_cmd.dimmed());
    });

    Ok(())
}
