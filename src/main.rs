//! ritalin -- proof-carrying completion for AI coding agents.
//!
//! Built on the agent-cli-framework patterns:
//!   - JSON envelope on stdout, coloured table on TTY
//!   - Semantic exit codes (0-4)
//!   - `--quiet` to suppress informational output
//!   - `agent-info` for machine-readable capability discovery
//!   - `skill install` to register with Claude Code, Codex, Gemini
//!   - `update` for self-update via GitHub Releases
//!
//! Plus the ritalin-specific contract enforcement layer:
//!   - `init` creates a verifiable scope contract
//!   - `add` records new obligations
//!   - `prove` runs verification commands and records evidence
//!   - `gate` is the Claude Code Stop hook — blocks on missing evidence

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
            kind,
            critical,
        } => commands::add::run(ctx, claim, proof, kind, critical),
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
