use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::cli::ObligationKind;
use crate::error::AppError;
use crate::ledger::{marker, obligations, obligations::Obligation, scope::Scope, state_dir};
use crate::output::{self, Ctx};

#[derive(Debug, Deserialize)]
struct Manifest {
    outcome: String,
    #[serde(default)]
    obligations: Vec<ManifestObligation>,
}

#[derive(Debug, Deserialize)]
struct ManifestObligation {
    claim: String,
    proof: String,
    #[serde(default = "default_kind")]
    kind: ObligationKind,
    #[serde(default = "default_critical")]
    critical: bool,
}

fn default_kind() -> ObligationKind {
    ObligationKind::Other
}

fn default_critical() -> bool {
    true
}

#[derive(Serialize)]
struct SeedResult {
    outcome: String,
    obligations_seeded: usize,
    state_dir: String,
}

pub fn run(ctx: Ctx, manifest_path: String, force: bool) -> Result<(), AppError> {
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| AppError::InvalidInput(format!("cannot read {manifest_path}: {e}")))?;

    let manifest: Manifest = if manifest_path.ends_with(".toml") {
        toml::from_str(&content)
            .map_err(|e| AppError::InvalidInput(format!("invalid TOML in {manifest_path}: {e}")))?
    } else if manifest_path.ends_with(".yaml") || manifest_path.ends_with(".yml") {
        serde_yaml::from_str(&content)?
    } else {
        // Try TOML first, fall back to YAML
        toml::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content).map_err(|e| e.into()))
            .map_err(|e: AppError| {
                AppError::InvalidInput(format!("cannot parse {manifest_path}: {e}"))
            })?
    };

    let cwd = std::env::current_dir()?;
    let dir = state_dir(&cwd);

    if dir.exists() && !force {
        return Err(AppError::InvalidInput(
            "contract already exists — use --force to overwrite".into(),
        ));
    }

    // Write scope
    let scope = Scope::new(manifest.outcome.clone());
    scope.write(&dir)?;

    // Create marker
    let marker_msg = format!(
        "ritalin: outcome = {}\nSeeded from: {}\n",
        manifest.outcome, manifest_path
    );
    marker::create(&dir, &marker_msg)?;

    // Seed obligations
    for (i, mob) in manifest.obligations.iter().enumerate() {
        let id = format!("O-{:03}", i + 1);
        let ob = Obligation {
            id,
            claim: mob.claim.clone(),
            kind: mob.kind,
            critical: mob.critical,
            proof_cmd: mob.proof.clone(),
            created_at: Utc::now(),
        };
        obligations::append(&dir, &ob)?;
    }

    let result = SeedResult {
        outcome: manifest.outcome,
        obligations_seeded: manifest.obligations.len(),
        state_dir: dir.display().to_string(),
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        println!(
            "{} seeded {} obligations from manifest",
            "+".green().bold(),
            r.obligations_seeded
        );
        println!("  outcome: {}", r.outcome);
        println!("  state:   {}", r.state_dir.dimmed());
    });

    Ok(())
}
