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

pub fn run(ctx: Ctx, outcome: Option<String>) -> Result<(), AppError> {
    let outcome = outcome.unwrap_or_else(|| {
        "TODO: replace with one-line outcome (e.g. \"User can save and reload notification settings\")"
            .to_string()
    });

    let cwd = std::env::current_dir()?;
    let dir = state_dir(&cwd);

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
        println!("  1. Add critical obligations:");
        println!(
            "     {}",
            "ritalin add \"User can submit form\" --proof \"pnpm test e2e/form.test.ts\" --kind user_path"
                .dimmed()
        );
        println!("  2. Wire the stop hook in .claude/settings.json:");
        println!(
            "     {}",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"ritalin gate --hook-mode"}]}]}}"#
                .dimmed()
        );
        println!("  3. Let the agent work. It will be blocked until evidence exists.");
    });

    Ok(())
}
