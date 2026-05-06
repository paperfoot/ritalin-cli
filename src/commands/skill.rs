use serde::Serialize;
use std::path::PathBuf;

use crate::error::AppError;
use crate::output::{self, Ctx};

/// SKILL.md content. Embedded in the binary so updates ship with the binary.
const SKILL_MD: &str = include_str!("../skill/SKILL.md");
const OPENAI_YAML: &str = r#"interface:
  display_name: "ritalin"
  short_description: "Evidence gate for AI coding agents"
  default_prompt: "Use ritalin to create, prove, and gate a verifiable task contract."
policy:
  allow_implicit_invocation: true
"#;

struct SkillTarget {
    name: &'static str,
    path: PathBuf,
    openai_metadata: bool,
}

fn home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn skill_targets() -> Vec<SkillTarget> {
    let h = home();
    let codex_home = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| h.join(".codex"));
    vec![
        SkillTarget {
            name: "Claude Code",
            path: h.join(".claude/skills/ritalin"),
            openai_metadata: false,
        },
        SkillTarget {
            name: "Codex user",
            path: h.join(".agents/skills/ritalin"),
            openai_metadata: true,
        },
        SkillTarget {
            name: "Codex legacy",
            path: codex_home.join("skills/ritalin"),
            openai_metadata: true,
        },
        SkillTarget {
            name: "Gemini CLI",
            path: h.join(".gemini/skills/ritalin"),
            openai_metadata: false,
        },
    ]
}

#[derive(Serialize)]
struct InstallResult {
    platform: String,
    path: String,
    metadata_path: Option<String>,
    status: String,
}

pub fn install(ctx: Ctx) -> Result<(), AppError> {
    let mut results: Vec<InstallResult> = Vec::new();
    for target in &skill_targets() {
        let skill_path = target.path.join("SKILL.md");
        let metadata_path = target.path.join("agents/openai.yaml");
        let skill_current = skill_path.exists()
            && std::fs::read_to_string(&skill_path).is_ok_and(|c| c == SKILL_MD);
        let metadata_current = !target.openai_metadata
            || (metadata_path.exists()
                && std::fs::read_to_string(&metadata_path).is_ok_and(|c| c == OPENAI_YAML));
        if skill_current && metadata_current {
            results.push(InstallResult {
                platform: target.name.into(),
                path: skill_path.display().to_string(),
                metadata_path: target
                    .openai_metadata
                    .then(|| metadata_path.display().to_string()),
                status: "already_current".into(),
            });
            continue;
        }
        std::fs::create_dir_all(&target.path)?;
        std::fs::write(&skill_path, SKILL_MD)?;
        if target.openai_metadata {
            if let Some(parent) = metadata_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&metadata_path, OPENAI_YAML)?;
        }
        results.push(InstallResult {
            platform: target.name.into(),
            path: skill_path.display().to_string(),
            metadata_path: target
                .openai_metadata
                .then(|| metadata_path.display().to_string()),
            status: "installed".into(),
        });
    }

    output::print_success_or(ctx, &results, |r| {
        use owo_colors::OwoColorize;
        for item in r {
            let marker = if item.status == "installed" { "+" } else { "=" };
            println!(
                " {} {} -> {}",
                marker.green(),
                item.platform.bold(),
                item.path.dimmed()
            );
            if let Some(metadata_path) = &item.metadata_path {
                println!("   metadata -> {}", metadata_path.dimmed());
            }
        }
    });

    Ok(())
}

#[derive(Serialize)]
struct SkillStatus {
    platform: String,
    installed: bool,
    current: bool,
    metadata_current: Option<bool>,
}

pub fn status(ctx: Ctx) -> Result<(), AppError> {
    let mut results: Vec<SkillStatus> = Vec::new();
    for target in &skill_targets() {
        let skill_path = target.path.join("SKILL.md");
        let metadata_path = target.path.join("agents/openai.yaml");
        let (installed, current) = if skill_path.exists() {
            let current = std::fs::read_to_string(&skill_path).is_ok_and(|c| c == SKILL_MD);
            (true, current)
        } else {
            (false, false)
        };
        let metadata_current = target.openai_metadata.then(|| {
            metadata_path.exists()
                && std::fs::read_to_string(&metadata_path).is_ok_and(|c| c == OPENAI_YAML)
        });
        results.push(SkillStatus {
            platform: target.name.into(),
            installed,
            current: current && metadata_current.unwrap_or(true),
            metadata_current,
        });
    }

    output::print_success_or(ctx, &results, |r| {
        use owo_colors::OwoColorize;
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["Platform", "Installed", "Current", "OpenAI metadata"]);
        for item in r {
            table.add_row(vec![
                item.platform.clone(),
                if item.installed {
                    "Yes".green().to_string()
                } else {
                    "No".red().to_string()
                },
                if item.current {
                    "Yes".green().to_string()
                } else {
                    "No".dimmed().to_string()
                },
                match item.metadata_current {
                    Some(true) => "Yes".green().to_string(),
                    Some(false) => "No".red().to_string(),
                    None => "-".dimmed().to_string(),
                },
            ]);
        }
        println!("{table}");
    });

    Ok(())
}
