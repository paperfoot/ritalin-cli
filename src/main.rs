//! ritalin -- executive function for AI coding agents.
//!
//! Like Ritalin for ADHD: agents are smart but unfocused. They skip research,
//! hallucinate patterns, use stale training data, lose scope, and claim "done"
//! at 80%. This CLI helps them focus their intelligence on the right things.
//!
//! Built on the agent-cli-framework patterns:
//!   - JSON envelope on stdout, coloured table on TTY
//!   - Semantic exit codes (0-4)
//!   - `agent-info` for machine-readable capability discovery
//!   - `skill install` to register with Claude Code, Codex, Gemini
//!
//! The binary stays lean. The SKILL.md teaches reasoning. The gate enforces:
//!   - `init` creates a scope contract
//!   - `add` records obligations (research, reference, freshness, tests, etc.)
//!   - `prove` runs verification commands and records evidence
//!   - `gate` is the Stop hook — blocks until every critical obligation is discharged

mod cli;
mod commands;
mod error;
mod gate_eval;
mod ledger;
mod output;

use clap::Parser;

use cli::{Cli, Commands, SkillAction};
use output::{Ctx, Format};

/// Pre-scan argv for --json before clap parses. Honors --json on help/version
/// and parse-error paths where clap hasn't populated the Cli struct yet.
fn has_json_flag() -> bool {
    std::env::args_os().any(|a| a == "--json")
}

fn main() {
    let json_flag = has_json_flag();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                let format = Format::detect(json_flag);
                match format {
                    Format::Json => {
                        output::print_help_json(e);
                        std::process::exit(0);
                    }
                    Format::Human => e.exit(),
                }
            }
            let format = Format::detect(json_flag);
            output::print_clap_error(format, &e);
            std::process::exit(3);
        }
    };

    let ctx = Ctx::new(cli.json, cli.quiet);

    let result = match cli.command {
        Commands::Init { outcome, force } => commands::init::run(ctx, outcome, force),
        Commands::Add {
            claim,
            proof,
            literal,
            file,
            kind,
            critical,
        } => commands::add::run(ctx, claim, proof, literal, file, kind, critical),
        Commands::Prove { id, cmd } => commands::prove::run(ctx, id, cmd),
        Commands::Gate { hook_mode } => commands::gate::run(ctx, hook_mode),
        Commands::Seed { manifest, force } => commands::seed::run(ctx, manifest, force),
        Commands::Status => commands::status::run(ctx),
        Commands::AgentInfo => {
            commands::agent_info::run();
            Ok(())
        }
        Commands::Skill { action } => match action {
            SkillAction::Install => commands::skill::install(ctx),
            SkillAction::Status => commands::skill::status(ctx),
        },
        Commands::Update { check } => commands::update::run(ctx, check),
    };

    if let Err(e) = result {
        output::print_error(ctx.format, &e);
        std::process::exit(e.exit_code());
    }
}
