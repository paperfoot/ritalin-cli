/// Error types with semantic exit codes (0-4 contract from agent-cli-framework).
///
/// Exit code mapping:
///   0: Success
///   1: Transient (IO/git) — retry
///   2: Config error (not initialized) — fix setup
///   3: Bad input (unknown obligation, invalid arg) — fix arguments
///   4: Rate limited (unused here) — wait and retry

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("ritalin not initialized — run `ritalin init` first")]
    NotInitialized,

    #[error("scope.yaml is missing — run `ritalin init`")]
    ScopeMissing,

    #[error("obligation {0} not found in ledger")]
    UnknownObligation(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("verification command failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("update failed: {0}")]
    Update(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotInitialized | Self::ScopeMissing => 2,
            Self::UnknownObligation(_) | Self::InvalidInput(_) => 3,
            Self::VerificationFailed(_) | Self::Io(_) | Self::Yaml(_) | Self::Json(_) | Self::Update(_) => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotInitialized => "not_initialized",
            Self::ScopeMissing => "scope_missing",
            Self::UnknownObligation(_) => "unknown_obligation",
            Self::InvalidInput(_) => "invalid_input",
            Self::VerificationFailed(_) => "verification_failed",
            Self::Io(_) => "io_error",
            Self::Yaml(_) => "yaml_error",
            Self::Json(_) => "json_error",
            Self::Update(_) => "update_error",
        }
    }

    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::NotInitialized | Self::ScopeMissing => {
                "Run `ritalin init --outcome \"what should be true when done\"`"
            }
            Self::UnknownObligation(_) => "Run `ritalin status` to list current obligations",
            Self::InvalidInput(_) => "Run `ritalin --help` to see valid arguments",
            Self::VerificationFailed(_) => {
                "Fix the failing command, then re-run `ritalin prove <id>`"
            }
            Self::Io(_) | Self::Yaml(_) | Self::Json(_) => "Retry the command",
            Self::Update(_) => "Retry later, or install via `cargo install ritalin`",
        }
    }
}
