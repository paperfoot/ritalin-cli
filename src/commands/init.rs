use serde::Serialize;

use crate::error::AppError;
use crate::ledger::{marker, scope::Scope, state_dir};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct InitResult {
    state_dir: String,
    outcome: String,
    marker_created: bool,
}

pub fn run(ctx: Ctx, outcome: Option<String>, force: bool) -> Result<(), AppError> {
    let outcome = outcome.unwrap_or_else(|| {
        "TODO: replace with one-line outcome (e.g. \"User can save and reload notification settings\")"
            .to_string()
    });

    let cwd = std::env::current_dir()?;
    let dir = state_dir(&cwd);

    if dir.exists() && !force {
        return Err(AppError::InvalidInput(
            "contract already exists — use --force to overwrite".into(),
        ));
    }

    // When forcing, clear old ledgers so the contract starts fresh.
    if force && dir.exists() {
        let _ = std::fs::remove_file(dir.join("obligations.jsonl"));
        let _ = std::fs::remove_file(dir.join("evidence.jsonl"));
    }

    let scope = Scope::new(outcome.clone());
    scope.write(&dir)?;

    // Marker file lives next to .ritalin/, not inside it.
    let marker_msg = format!(
        "ritalin: outcome = {outcome}\n\
         This file is removed by `ritalin gate` once every critical obligation has evidence.\n"
    );
    marker::create(&dir, &marker_msg)?;

    let result = InitResult {
        state_dir: dir.display().to_string(),
        outcome,
        marker_created: true,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        println!("{} ritalin initialized", "+".green().bold());
        println!("  state:   {}", r.state_dir.dimmed());
        println!("  outcome: {}", r.outcome);
        println!();
        println!("Next steps:");
        println!("  1. Research & ground your approach before implementing");
        println!("  2. Add obligations (tests, research, references, freshness):");
        println!(
            "     {}",
            "ritalin add \"Feature works\" --proof \"pnpm test e2e/feature.test.ts\" --kind user_path"
                .dimmed()
        );
        println!(
            "     {}",
            "ritalin add \"Approach grounded\" --proof \"search --mode scholar 'topic' --json | jq '.results | length > 0'\" --kind research_grounded"
                .dimmed()
        );
        println!("  3. Wire the stop hook in .claude/settings.json:");
        println!(
            "     {}",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"ritalin gate --hook-mode"}]}]}}"#
                .dimmed()
        );
        println!("  4. Work, prove, gate. Blocked until every critical obligation has evidence.");
    });

    Ok(())
}
