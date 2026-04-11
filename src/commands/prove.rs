use chrono::Utc;
use serde::Serialize;
use std::process::Command;

use crate::error::AppError;
use crate::ledger::{evidence, evidence::Evidence, is_initialized, obligations, state_dir};
use crate::output::{self, Ctx};

const TAIL_LIMIT: usize = 2000;

fn tail(s: &str) -> String {
    if s.len() <= TAIL_LIMIT {
        s.to_string()
    } else {
        let start = s.len() - TAIL_LIMIT;
        format!("…{}", &s[start..])
    }
}

#[derive(Serialize)]
struct ProveResult {
    obligation_id: String,
    command: String,
    exit_code: i32,
    discharged: bool,
    stdout_tail: String,
    stderr_tail: String,
}

pub fn run(ctx: Ctx, id: String, cmd: Option<String>) -> Result<(), AppError> {
    let cwd = std::env::current_dir()?;
    if !is_initialized(&cwd) {
        return Err(AppError::NotInitialized);
    }
    let dir = state_dir(&cwd);

    let ob = obligations::find(&dir, &id)?;
    let command = cmd.unwrap_or_else(|| ob.proof_cmd.clone());

    // Run via shell so users can pass pipes, redirects, env vars, etc.
    let output_res = Command::new("sh").arg("-c").arg(&command).output()?;

    let exit_code = output_res.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output_res.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output_res.stderr).to_string();

    let ev = Evidence {
        obligation_id: id.clone(),
        command: command.clone(),
        exit_code,
        stdout_tail: tail(&stdout),
        stderr_tail: tail(&stderr),
        recorded_at: Utc::now(),
    };
    evidence::append(&dir, &ev)?;

    let result = ProveResult {
        obligation_id: id,
        command,
        exit_code,
        discharged: exit_code == 0,
        stdout_tail: ev.stdout_tail.clone(),
        stderr_tail: ev.stderr_tail.clone(),
    };

    output::print_success_or(ctx, &result, |r| {
        use owo_colors::OwoColorize;
        let badge = if r.discharged {
            "PASS".green().bold().to_string()
        } else {
            "FAIL".red().bold().to_string()
        };
        println!(
            "{} {} (exit {})",
            badge,
            r.obligation_id.bold(),
            r.exit_code
        );
        println!("  cmd: {}", r.command.dimmed());
        if !r.stderr_tail.is_empty() {
            println!("  stderr: {}", r.stderr_tail.dimmed());
        }
    });

    Ok(())
}
