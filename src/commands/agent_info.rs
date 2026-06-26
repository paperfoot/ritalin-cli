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
                        "required": "unless --literal+--file (kind=literal_match) or --regex+--file (kind=literal_regex)",
                        "conflicts_with": ["--literal", "--regex"],
                        "description": "Shell command that proves the claim"
                    },
                    {
                        "name": "--literal",
                        "type": "string",
                        "required": false,
                        "requires": ["--file"],
                        "conflicts_with": ["--proof", "--regex"],
                        "description": "Verbatim string that must appear in --file. Pairs with --kind literal_match. Synthesised proof: `grep -F -- <literal> <file>`."
                    },
                    {
                        "name": "--regex",
                        "type": "string",
                        "required": false,
                        "requires": ["--file"],
                        "conflicts_with": ["--proof", "--literal"],
                        "description": "POSIX ERE pattern that must match in --file. Pairs with --kind literal_regex. Synthesised proof: `grep -E -- <pattern> <file>`. Use [[:space:]] not \\s, alternatives via (A|B)."
                    },
                    {
                        "name": "--file",
                        "type": "string",
                        "required": false,
                        "description": "Path searched by --literal or --regex. Resolved at prove time; missing file is a proof failure (grep exit 2), not an add error."
                    },
                    {
                        "name": "--kind",
                        "type": "string",
                        "required": false,
                        "default": "other",
                        "values": ["user_path", "integration", "persistence", "failure_path", "performance", "security", "research_grounded", "code_referenced", "model_current", "literal_match", "literal_regex", "other"],
                        "description": "Category of obligation. literal_match pairs with --literal/--file (verbatim). literal_regex pairs with --regex/--file (POSIX ERE for semantic claims)."
                    },
                    {
                        "name": "--critical",
                        "type": "bool",
                        "required": false,
                        "default": true,
                        "description": "Block stop if this obligation is open"
                    },
                    {
                        "name": "--depends-on",
                        "type": "list<string>",
                        "required": false,
                        "default": [],
                        "description": "Comma-separated repo-relative file paths (no .., no absolute). When set, this obligation's evidence freshness is checked against the SHA-256 of just these files instead of the whole workspace — so unrelated edits in a parallel session don't churn this obligation. When empty, the global workspace hash is used (v0.3 behavior)."
                    }
                ],
                "appends_to": [".ritalin/obligations.jsonl"]
            },
            "prove": {
                "description": "Run a verification command and record evidence for an obligation. With --all, batch-refreshes every obligation in order.",
                "args": [
                    {
                        "name": "id",
                        "kind": "positional",
                        "type": "string",
                        "required": "unless --all is set",
                        "description": "Obligation ID (e.g. O-001). Omit when using --all."
                    }
                ],
                "options": [
                    {
                        "name": "--cmd",
                        "type": "string",
                        "required": false,
                        "conflicts_with": ["--all"],
                        "description": "Override the stored proof command (single-id only). Diagnostic-only — won't discharge the obligation because gate recomputes proof_hash from r.command and the substituted command won't match."
                    },
                    {
                        "name": "--all",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "Re-prove every obligation in add-order. Continues on failure. Result envelope reports passed/failed/skipped counts. Exit 0 if all passed, 1 if any failed."
                    },
                    {
                        "name": "--stale-only",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "requires": ["--all"],
                        "description": "With --all, skip obligations whose evidence is already passing+fresh. The post-commit refresh idiom."
                    }
                ],
                "appends_to": [".ritalin/evidence.jsonl"],
                "output_fields": {
                    "remaining_open": "Scope-refresh snapshot recomputed after evidence is appended: { ids: [...], critical: N, advisory: N }.",
                    "workspace_mutated": "true if the proof command itself rewrote a file it depends on (formatters, codegen). Other obligations sharing those files may now be stale.",
                    "proved": "(--all only) Array of per-obligation prove results.",
                    "skipped": "(--all --stale-only) Array of skipped obligations with reason.",
                    "summary": "(--all only) { total, discharged, failed, skipped }."
                }
            },
            "gate": {
                "description": "Stop hook gate. Blocks unless every critical obligation has passing evidence (proof_hash recomputed from r.command, workspace_hash matches per-obligation scope).",
                "args": [],
                "options": [
                    {
                        "name": "--hook-mode",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "conflicts_with": ["--summary"],
                        "description": "Emit Claude Code stop hook decision JSON instead of framework envelope"
                    },
                    {
                        "name": "--summary",
                        "type": "bool",
                        "required": false,
                        "default": false,
                        "description": "One-line shell-friendly verdict. Format: `verdict=<pass|fail> critical_open=<n> advisory_open=<n> total=<n>[ blocking=O-NNN]`. Stable schema, awk-safe."
                    }
                ],
                "behavior": {
                    "hook_mode": "Reads stop_hook_active from stdin JSON. On block, prints {\"decision\":\"block\",\"reason\":\"...\"} to stdout. On pass, exits 0 with empty stdout. Removes .task-incomplete on pass.",
                    "cli_mode": "Returns framework JSON envelope or human-readable report. Exits non-zero on fail.",
                    "summary_mode": "Single line stdout, exits non-zero on fail."
                },
                "env": {
                    "RITALIN_GATE": "Hook-mode opt-out. When set to 0/off/false/no/disable/disabled (case-insensitive), `gate --hook-mode` exits 0 with empty stdout and leaves the contract untouched — so a one-shot reviewer/auditor/CI run that does NOT own the contract stops cleanly instead of being hijacked into the gate. Unset (or any other value) keeps the gate active. Only affects hook mode; manual `ritalin gate` always reports the true verdict."
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
                "description": "Install SKILL.md to ~/.claude/skills, ~/.agents/skills, $CODEX_HOME/skills (or ~/.codex/skills), and ~/.gemini/skills. Codex targets also receive agents/openai.yaml metadata.",
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
