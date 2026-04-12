use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ritalin",
    version,
    about = "Executive function for AI coding agents",
    long_about = "ritalin is executive function for AI coding agents.\n\
                  Like Ritalin for ADHD — agents are smart, they just need help focusing their\n\
                  intelligence on the right things and avoiding avoidable mistakes.\n\n\
                  It ensures agents research before implementing, ground claims in evidence, reference\n\
                  real code instead of hallucinating patterns, and actually finish what they start.\n\n\
                  Workflow:\n  \
                  1. ritalin init --outcome \"...\"\n  \
                  2. ritalin add \"claim\" --proof \"shell command\"  (repeat per obligation)\n  \
                  3. Hook ritalin gate --hook-mode into Claude Code's Stop event\n  \
                  4. Agent works, runs ritalin prove <id> as it discharges obligations\n  \
                  5. Stop is blocked until every critical obligation has evidence"
)]
pub struct Cli {
    /// Force JSON output even in a terminal
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress informational human output (errors still print)
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum ObligationKind {
    UserPath,
    Integration,
    Persistence,
    FailurePath,
    Performance,
    Security,
    ResearchGrounded,
    CodeReferenced,
    ModelCurrent,
    Other,
}

impl std::fmt::Display for ObligationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserPath => write!(f, "user_path"),
            Self::Integration => write!(f, "integration"),
            Self::Persistence => write!(f, "persistence"),
            Self::FailurePath => write!(f, "failure_path"),
            Self::Performance => write!(f, "performance"),
            Self::Security => write!(f, "security"),
            Self::ResearchGrounded => write!(f, "research_grounded"),
            Self::CodeReferenced => write!(f, "code_referenced"),
            Self::ModelCurrent => write!(f, "model_current"),
            Self::Other => write!(f, "other"),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a ritalin scope contract in the current directory
    Init {
        /// One-line outcome statement (the user-facing thing being built)
        #[arg(long, short)]
        outcome: Option<String>,
        /// Overwrite an existing contract
        #[arg(long)]
        force: bool,
    },

    /// Add a new obligation to the ledger
    Add {
        /// What must be true for this obligation to be discharged
        claim: String,
        /// Shell command that proves it (e.g. "pnpm test settings.contract.test.ts")
        #[arg(long)]
        proof: String,
        /// Category of obligation
        #[arg(long, value_enum, default_value = "other")]
        kind: ObligationKind,
        /// Mark as critical (gate blocks stop if open). Default true.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        critical: bool,
    },

    /// Run a verification command and record evidence for an obligation
    Prove {
        /// Obligation ID (e.g. O-001)
        id: String,
        /// Override the proof command (optional; default uses obligation's stored proof)
        #[arg(long)]
        cmd: Option<String>,
    },

    /// Stop hook gate. Blocks unless every critical obligation has evidence.
    Gate {
        /// Emit Claude Code stop hook decision JSON instead of framework envelope
        #[arg(long)]
        hook_mode: bool,
    },

    /// Seed a contract from a TOML/YAML manifest file
    Seed {
        /// Path to the manifest file (TOML or YAML)
        manifest: String,
        /// Overwrite an existing contract
        #[arg(long)]
        force: bool,
    },

    /// Show current scope, obligations, and evidence
    Status,

    /// Machine-readable capability manifest
    #[command(visible_alias = "info")]
    AgentInfo,

    /// Install skill file to AI agent platforms
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// Self-update from GitHub Releases
    Update {
        /// Check only, don't install
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// Write SKILL.md to ~/.claude/skills, ~/.codex/skills, ~/.gemini/skills
    Install,
    /// Check which platforms have the skill installed
    Status,
}
