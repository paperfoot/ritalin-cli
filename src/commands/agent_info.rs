// Machine-readable capability manifest.
//
// Always JSON — agents call this to bootstrap. Uses its own schema (not the
// envelope) because this IS the schema definition.

pub fn run() {
    let info = serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "purpose": "Executive function for AI coding agents. Ensures agents research before implementing, ground claims in evidence, and actually finish what they start.",
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
                    },
                    {
                        "name": "--force",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "Overwrite an existing contract"
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
                        "required": "unless --literal and --file are supplied (kind=literal_match)",
                        "conflicts_with": ["--literal", "--file"],
                        "description": "Shell command that proves the claim"
                    },
                    {
                        "name": "--literal",
                        "type": "string",
                        "required": false,
                        "requires": ["--file"],
                        "conflicts_with": ["--proof"],
                        "description": "Verbatim string that must appear in --file. Use with --kind literal_match. Proof is auto-synthesised as `grep -F -- <literal> <file>`. Note: -F matches anywhere (including comments) — include structural context in the literal."
                    },
                    {
                        "name": "--file",
                        "type": "string",
                        "required": false,
                        "requires": ["--literal"],
                        "conflicts_with": ["--proof"],
                        "description": "Path searched by --literal. Resolved at prove time; missing file is a proof failure (grep exit 2), not an add error."
                    },
                    {
                        "name": "--kind",
                        "type": "string",
                        "required": false,
                        "default": "other",
                        "values": ["user_path", "integration", "persistence", "failure_path", "performance", "security", "research_grounded", "code_referenced", "model_current", "literal_match", "other"],
                        "description": "Category of obligation. literal_match pairs with --literal/--file for verbatim-string checks (kills approximation drift on CSS values, config constants, hex colors, API strings, etc.)."
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
                "appends_to": [".ritalin/evidence.jsonl"],
                "output_fields": {
                    "remaining_open": "Scope-refresh snapshot (recomputed after this evidence is appended): { ids: [...], critical: N, advisory: N }. Lists obligations still open — includes this one if its proof failed or --cmd override changed the proof hash."
                }
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
            "seed": {
                "description": "Seed a contract from a TOML/YAML manifest file",
                "args": [
                    {
                        "name": "manifest",
                        "kind": "positional",
                        "type": "string",
                        "required": true,
                        "description": "Path to the manifest file (TOML or YAML)"
                    }
                ],
                "options": [
                    {
                        "name": "--force",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "Overwrite an existing contract"
                    }
                ],
                "creates": [".ritalin/scope.yaml", ".ritalin/obligations.jsonl", ".task-incomplete"]
            },
            "status": {
                "description": "Show current scope, obligations, and evidence",
                "args": [],
                "options": []
            },
            "export-contract": {
                "description": "Emit a subagent-ready briefing for Task/Agent delegation prompts. Read-only, zero-arg. Human mode prints the raw briefing for copy/paste; --json wraps it in the envelope with structured open_obligations.",
                "args": [],
                "options": [],
                "output_fields": {
                    "outcome": "Current contract outcome from scope.yaml",
                    "obligations_total": "Total obligations in the ledger",
                    "remaining_open": "{ ids: [...], critical: N, advisory: N }",
                    "open_obligations": "Array of { id, claim, kind, critical, proof_cmd, last_exit_code }",
                    "briefing": "Ready-to-paste subagent prompt text"
                }
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
            "1": "Verification failed or transient IO error — fix the underlying issue or retry",
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
            ".ritalin/evidence.jsonl": "append-only verification evidence ledger",
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
