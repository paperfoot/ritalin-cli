use chrono::Utc;
use serde::Serialize;

use crate::cli::ObligationKind;
use crate::error::AppError;
use crate::ledger::{is_initialized, marker, obligations, obligations::Obligation, state_dir};
use crate::output::{self, Ctx};

#[derive(Serialize)]
struct AddResult {
    id: String,
    claim: String,
    kind: String,
    critical: bool,
    proof_cmd: String,
}

/// POSIX single-quote a string for safe embedding in a shell command.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Synthesise the proof command for a literal_match obligation:
/// `grep -F -- <quoted-literal> <quoted-file>`. `-F` treats the literal as a
/// fixed string (no regex). `--` ensures literals starting with `-` are not
/// parsed as grep flags.
fn synth_literal_match(literal: &str, file: &str) -> String {
    format!("grep -F -- {} {}", sh_quote(literal), sh_quote(file))
}

/// Normalize a list of dependency paths: trim, drop empties, sort, dedupe,
/// reject anything that escapes the repo (absolute paths or `..` segments).
/// Returns the canonical list ready for storage on the obligation.
fn normalize_depends_on(raw: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    for p in raw {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = std::path::Path::new(trimmed);
        if path.is_absolute() {
            return Err(AppError::InvalidInput(format!(
                "depends_on path must be repo-relative, got absolute: {trimmed}"
            )));
        }
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(AppError::InvalidInput(format!(
                "depends_on path must not contain `..`: {trimmed}"
            )));
        }
        out.push(trimmed.to_string());
    }
    out.sort();
    out.dedup();
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    ctx: Ctx,
    claim: String,
    proof: Option<String>,
    literal: Option<String>,
    file: Option<String>,
    kind: ObligationKind,
    critical: bool,
    depends_on: Vec<String>,
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

    // Validate dependency paths: trim whitespace, drop empties, dedupe,
    // sort, reject paths that escape the repo root via `..` or absolute
    // paths. Existence is checked at prove/gate time so the obligation can
    // be added before the file lands.
    let depends_on = normalize_depends_on(depends_on)?;

    // clap has already enforced:
    //   proof XOR (literal AND file); literal ↔ file; at least one path chosen.
    // Only kind consistency is validated here.
    let proof_cmd = match (proof, literal.as_deref(), file.as_deref()) {
        (Some(p), None, None) => {
            if matches!(kind, ObligationKind::LiteralMatch) {
                return Err(AppError::InvalidInput(
                    "--kind literal_match requires --literal and --file, not --proof".into(),
                ));
            }
            let p = p.trim().to_string();
            if p.is_empty() {
                return Err(AppError::InvalidInput(
                    "proof command cannot be empty".into(),
                ));
            }
            p
        }
        (None, Some(lit), Some(f)) => {
            if !matches!(kind, ObligationKind::LiteralMatch) {
                return Err(AppError::InvalidInput(
                    "--literal and --file require --kind literal_match".into(),
                ));
            }
            if lit.is_empty() {
                return Err(AppError::InvalidInput("--literal cannot be empty".into()));
            }
            if f.is_empty() {
                return Err(AppError::InvalidInput("--file cannot be empty".into()));
            }
            synth_literal_match(lit, f)
        }
        _ => unreachable!("clap constraints guarantee exactly one of proof or literal+file"),
    };

    let id = obligations::next_id(&dir)?;
    let ob = Obligation {
        id: id.clone(),
        claim: claim.clone(),
        kind,
        critical,
        proof_cmd: proof_cmd.clone(),
        created_at: Utc::now(),
        depends_on: depends_on.clone(),
    };
    obligations::append(&dir, &ob)?;

    if critical && !marker::exists(&dir) {
        marker::create(
            &dir,
            &format!("ritalin: reopened — obligation {id} added after gate\n"),
        )?;
    }

    let result = AddResult {
        id,
        claim,
        kind: kind.to_string(),
        critical,
        proof_cmd,
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        let crit = if r.critical {
            "[critical]".red().to_string()
        } else {
            "[advisory]".dimmed().to_string()
        };
        println!(
            "{} {} {} {}",
            "+".green().bold(),
            r.id.bold(),
            crit,
            r.claim
        );
        println!("  proof: {}", r.proof_cmd.dimmed());
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sh_quote_simple() {
        assert_eq!(sh_quote("hello"), "'hello'");
    }

    #[test]
    fn sh_quote_with_single_quote() {
        assert_eq!(sh_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn sh_quote_empty() {
        assert_eq!(sh_quote(""), "''");
    }

    #[test]
    fn synth_includes_fixed_string_flag() {
        let cmd = synth_literal_match("rgba(0,0,0,0.5)", "src/theme.css");
        assert!(cmd.starts_with("grep -F -- "));
        assert!(cmd.contains("'rgba(0,0,0,0.5)'"));
        assert!(cmd.contains("'src/theme.css'"));
    }

    #[test]
    fn synth_quotes_single_quotes_and_spaces() {
        let cmd = synth_literal_match("it's a trap", "weird path.txt");
        assert_eq!(cmd, r#"grep -F -- 'it'\''s a trap' 'weird path.txt'"#);
    }

    #[test]
    fn synth_handles_literal_starting_with_dash() {
        // The `--` guard means literals like "-webkit-" don't parse as flags.
        let cmd = synth_literal_match("-webkit-font-smoothing", "src/a.css");
        assert!(cmd.contains("-- '-webkit-font-smoothing'"));
    }
}
