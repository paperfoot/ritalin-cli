---
name: ritalin
description: >
  Proof-carrying completion for AI coding agents. Use this skill whenever you
  start a non-trivial task and want hard guarantees that you actually finish
  it. Run `ritalin agent-info` for the full machine-readable capability
  manifest, exit codes, and JSON envelope structure.
---

# ritalin

Proof-carrying completion. The verification layer for the prompt-request era.

## When to invoke this skill

- Starting any non-trivial implementation task (more than 1 file)
- The user said "make sure you actually finish" or similar
- A previous session ended with the agent claiming "done" but the work was incomplete
- The repo has a `.ritalin/` directory or `.task-incomplete` file present

## Discovery

Run `ritalin agent-info` once at session start. It returns the full command list,
flags, exit codes, and the Claude Code stop-hook installation snippet. Do not
guess at flags — the manifest is the source of truth.

## Workflow

```
1. ritalin init --outcome "<one-line statement of what success looks like>"
2. ritalin add "<claim 1>" --proof "<shell command that verifies it>" --kind <kind>
   ritalin add "<claim 2>" --proof "<...>" --kind <kind>
   ... (one obligation per critical thing that must be true)
3. Implement.
4. ritalin prove O-001        (runs the stored proof command, records evidence)
   ritalin prove O-002
   ...
5. ritalin gate               (checks every critical obligation has passing evidence)
   ritalin status             (human view at any point)
```

## Obligation kinds

Use the right kind so the agent reasons clearly:

- `user_path`     — user-visible behaviour from input to outcome
- `integration`   — UI ↔ API ↔ DB wiring is real, not stubbed
- `persistence`   — state survives reload, restart, redeploy
- `failure_path`  — error states render and recover, not just happy path
- `performance`   — measurable speed/resource targets
- `security`      — auth, validation, secrets handling
- `other`         — fallback

## Anti-patterns

- Do NOT add obligations the agent can't actually verify with a shell command. Vague claims like "looks nice" must be replaced with `pixel-diff` commands or removed.
- Do NOT mark obligations as `--critical false` to make the gate pass. If it's not critical, it shouldn't be in the ledger.
- Do NOT delete or edit `.ritalin/obligations.jsonl` or `.ritalin/evidence.jsonl` directly. Both are append-only by design. Edit `scope.yaml` for the outcome statement only.
- Do NOT remove `.task-incomplete` manually. Only `ritalin gate` may remove it, and only when every critical obligation has passing evidence.

## Claude Code hook installation

Add this to `.claude/settings.json` (project-local) or `~/.claude/settings.json` (global):

```json
{
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
```

The gate checks `stop_hook_active` from stdin to prevent infinite loops.

## When the gate blocks you

Read the `reason` field. It tells you exactly which obligation is missing
evidence and which command to run. Run it. If it fails, fix the underlying
problem, then re-run `ritalin prove <id>`. Do not amend the scope to make
the failure go away — the failing obligation is information.

## Why this exists

Andrej Karpathy: *"The agents do not listen to my instructions in the
AGENTS.md files... I think in principle I could use hooks or slash commands
to clean this up but at some point just a shrug is easier."*

ritalin is the hooks. So you don't have to shrug.
