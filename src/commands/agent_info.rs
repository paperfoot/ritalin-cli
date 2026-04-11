// Machine-readable capability manifest.
//
// Always JSON — agents call this to bootstrap. Uses its own schema (not the
// envelope) because this IS the schema definition.

pub fn run() {
    let info = serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "purpose": "Proof-carrying completion enforcement for AI coding agents. The verification layer for the prompt-request era.",
        "commands": {
            "init": {
                "description": "Initialize a ritalin scope contract in the current directory",
                "args": [],
                "options": [
                    {
                        "name": "--outcome",
                        "type": "string",
                        "required": false,
                        "description": "One-line outcome statement"
                    }
                ],
                "creates": [".ritalin/scope.yaml", ".task-incomplete"]
            },
            "add": {
                "description": "Add a new obligation to the ledger",
                "args": [
                    {
                        "name": "claim",
                        "kind": "positional",
                        "type": "string",
                        "required": true,
                        "description": "What must be true for this obligation to be discharged"
                    }
                ],
                "options": [
                    {
                        "name": "--proof",
                        "type": "string",
                        "required": true,
                        "description": "Shell command that proves the claim"
                    },
                    {
                        "name": "--kind",
                        "type": "string",
                        "required": false,
                        "default": "other",
                        "values": ["user_path", "integration", "persistence", "failure_path", "performance", "security", "other"],
                        "description": "Category of obligation"
                    },
                    {
                        "name": "--critical",
                        "type": "bool",
                        "required": false,
                        "default": true,
                        "description": "Block stop if this obligation is open"
                    }
                ],
                "appends_to": [".ritalin/obligations.jsonl"]
            },
            "prove": {
                "description": "Run a verification command and record evidence for an obligation",
                "args": [
                    {
                        "name": "id",
                        "kind": "positional",
                        "type": "string",
                        "required": true,
                        "description": "Obligation ID (e.g. O-001)"
                    }
                ],
                "options": [
                    {
                        "name": "--cmd",
                        "type": "string",
                        "required": false,
                        "description": "Override the obligation's stored proof command"
                    }
                ],
                "appends_to": [".ritalin/evidence.jsonl"]
            },
            "gate": {
                "description": "Stop hook gate. Blocks unless every critical obligation has passing evidence.",
                "args": [],
                "options": [
                    {
                        "name": "--hook-mode",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "Emit Claude Code stop hook decision JSON instead of framework envelope"
                    }
                ],
                "behavior": {
                    "hook_mode": "Reads stop_hook_active from stdin JSON. On block, prints {\"decision\":\"block\",\"reason\":\"...\"} to stdout. On pass, exits 0 with empty stdout. Removes .task-incomplete on pass.",
                    "cli_mode": "Returns framework JSON envelope or human-readable report. Exits non-zero on fail."
                }
            },
            "status": {
                "description": "Show current scope, obligations, and evidence",
                "args": [],
                "options": []
            },
            "agent-info": {
                "description": "This manifest",
                "aliases": ["info"],
                "args": [],
                "options": []
            },
            "skill install": {
                "description": "Install SKILL.md to ~/.claude/skills, ~/.codex/skills, ~/.gemini/skills",
                "args": [],
                "options": []
            },
            "skill status": {
                "description": "Check which platforms have the skill installed",
                "args": [],
                "options": []
            },
            "update": {
                "description": "Self-update binary from GitHub Releases",
                "args": [],
                "options": [
                    {
                        "name": "--check",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "Check only, don't install"
                    }
                ]
            }
        },
        "global_flags": {
            "--json": {
                "description": "Force JSON output (auto-enabled when piped)",
                "type": "bool",
                "default": false
            },
            "--quiet": {
                "description": "Suppress informational output",
                "type": "bool",
                "default": false
            }
        },
        "exit_codes": {
            "0": "Success",
            "1": "Transient error (IO, verification failed) — retry",
            "2": "Not initialized — run `ritalin init`",
            "3": "Bad input (unknown obligation, invalid arg) — fix arguments",
            "4": "Rate limited (unused in v0.1) — wait and retry"
        },
        "envelope": {
            "version": "1",
            "success": "{ version, status, data }",
            "error": "{ version, status, error: { code, message, suggestion } }"
        },
        "state_files": {
            ".ritalin/scope.yaml": "human-edited contract: outcome + metadata",
            ".ritalin/obligations.jsonl": "append-only obligation ledger",
            ".ritalin/evidence.jsonl": "append-only evidence ledger (proof of work)",
            ".task-incomplete": "marker file; presence = agent must keep working"
        },
        "claude_code_hook_install": {
            "settings_path": ".claude/settings.json",
            "snippet": {
                "hooks": {
                    "Stop": [
                        {
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "ritalin gate --hook-mode"
                                }
                            ]
                        }
                    ]
                }
            }
        },
        "auto_json_when_piped": true
    });
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}
