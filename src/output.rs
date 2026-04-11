/// Output format detection and JSON envelope helpers.
///
/// Adapted from agent-cli-framework. Key behaviours:
///   - Terminal (TTY): coloured human output
///   - Piped/redirected: JSON envelope on stdout
///   - --json flag: force JSON even in terminal
///   - --quiet flag: suppress human informational output (errors always print)
///   - All JSON serialization is panic-safe via safe_json_string()
use serde::Serialize;
use std::io::IsTerminal;

use crate::error::AppError;

#[derive(Clone, Copy)]
pub enum Format {
    Json,
    Human,
}

impl Format {
    pub fn detect(json_flag: bool) -> Self {
        if json_flag || !std::io::stdout().is_terminal() {
            Format::Json
        } else {
            Format::Human
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ctx {
    pub format: Format,
    pub quiet: bool,
}

impl Ctx {
    pub fn new(json_flag: bool, quiet: bool) -> Self {
        Self {
            format: Format::detect(json_flag),
            quiet,
        }
    }
}

/// Serialize to pretty JSON. Never panics — falls back to a hand-rolled
/// error envelope if serde itself fails.
pub fn safe_json_string<T: Serialize>(value: &T) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(s) => s,
        Err(e) => {
            let fallback = serde_json::json!({
                "version": "1",
                "status": "error",
                "error": {
                    "code": "serialize",
                    "message": e.to_string(),
                    "suggestion": "Retry the command",
                },
            });
            serde_json::to_string_pretty(&fallback).unwrap_or_else(|_| {
                r#"{"version":"1","status":"error","error":{"code":"serialize","message":"serialization failed","suggestion":"Retry the command"}}"#.to_string()
            })
        }
    }
}

/// Print success envelope (JSON) or call the human closure (terminal).
/// Quiet + Human suppresses output. JSON always emits.
pub fn print_success_or<T: Serialize, F: FnOnce(&T)>(ctx: Ctx, data: &T, human: F) {
    match ctx.format {
        Format::Json => {
            let envelope = serde_json::json!({
                "version": "1",
                "status": "success",
                "data": data,
            });
            println!("{}", safe_json_string(&envelope));
        }
        Format::Human if !ctx.quiet => human(data),
        Format::Human => {}
    }
}

/// Print error to stderr in the appropriate format. Errors are never suppressed.
pub fn print_error(format: Format, err: &AppError) {
    let envelope = serde_json::json!({
        "version": "1",
        "status": "error",
        "error": {
            "code": err.error_code(),
            "message": err.to_string(),
            "suggestion": err.suggestion(),
        },
    });
    match format {
        Format::Json => eprintln!("{}", safe_json_string(&envelope)),
        Format::Human => {
            use owo_colors::OwoColorize;
            eprintln!("{} {}", "error:".red().bold(), err);
            let s = err.suggestion();
            if !s.is_empty() {
                eprintln!("  {}", s.dimmed());
            }
        }
    }
}

/// Wrap --help / --version output in a success JSON envelope.
pub fn print_help_json(err: clap::Error) {
    let envelope = serde_json::json!({
        "version": "1",
        "status": "success",
        "data": { "usage": err.to_string().trim_end() },
    });
    println!("{}", safe_json_string(&envelope));
}

/// Wrap clap parse errors. JSON to stderr; human renders directly.
pub fn print_clap_error(format: Format, err: &clap::Error) {
    match format {
        Format::Json => {
            let envelope = serde_json::json!({
                "version": "1",
                "status": "error",
                "error": {
                    "code": "invalid_input",
                    "message": err.to_string(),
                    "suggestion": "Run `ritalin --help` to see valid arguments",
                },
            });
            eprintln!("{}", safe_json_string(&envelope));
        }
        Format::Human => {
            eprint!("{err}");
        }
    }
}
