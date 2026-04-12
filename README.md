<div align="center">

<img src=".github/assets/hero.svg" alt="ritalin — executive function for AI coding agents" width="100%"/>

# ritalin

**Executive function for AI coding agents. Focus their intelligence. Ground their work. Stop the avoidable mistakes.**

<br />

[![Star this repo](https://img.shields.io/github/stars/199-biotechnologies/ritalin?style=for-the-badge&logo=github&label=%E2%AD%90%20Star%20this%20repo&color=yellow)](https://github.com/199-biotechnologies/ritalin/stargazers)
&nbsp;&nbsp;
[![Follow @longevityboris](https://img.shields.io/badge/Follow_%40longevityboris-000000?style=for-the-badge&logo=x&logoColor=white)](https://x.com/longevityboris)

<br />

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
&nbsp;
[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
&nbsp;
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge)](CONTRIBUTING.md)
&nbsp;
[![Crates.io](https://img.shields.io/crates/v/ritalin?style=for-the-badge&logo=rust)](https://crates.io/crates/ritalin)

---

The executive function layer for the prompt-request era. Like Ritalin for ADHD — it doesn't make agents smarter, it helps them use the intelligence they already have.

[Install](#install) · [How it works](#how-it-works) · [Hook into Claude Code](#hook-into-claude-code) · [Why this exists](#why-this-exists) · [Architecture](#architecture)

</div>

## Why this exists

> "Peter Steinberger told me that he wants PR to be 'prompt request'. His agents are perfectly capable of implementing most ideas, so there is no need to take your idea, expand it into a vibe coded mess using free tier ChatGPT and send that as a PR, which is now most PRs."
>
> — Andrej Karpathy ([@karpathy](https://x.com/karpathy/status/2040473058834878662)), April 2026

In the prompt-request era, code is no longer the artifact. Intent is. But there's a gap nobody has filled: **how do you trust the work?** Even Karpathy admits it:

> "The agents do not listen to my instructions in the AGENTS.md files... I think in principle I could use hooks or slash commands to clean this up but at some point just a shrug is easier."
>
> — Andrej Karpathy ([@karpathy](https://x.com/karpathy/status/2035173492447224237)), March 2026

ritalin is the hooks. So you don't have to shrug.

## The problem

AI coding agents are smart. GPT-5.4, Claude Opus, Gemini Pro — they can reason, write code, solve hard problems. The intelligence is there.

What's missing is **executive function**.

People with ADHD aren't unintelligent. They're often highly capable. But they skip steps, lose focus mid-task, forget to check their work, get distracted by tangents, and say "I'm done" when they're not. Ritalin (the drug) doesn't add IQ points. It adds follow-through.

AI agents have the same problem:

- **They skip research.** They hallucinate code patterns instead of checking GitHub for high-star, recent examples. They recommend libraries from stale training data instead of searching for what's current. They propose solutions without reading the relevant paper.
- **They lose scope.** As context fills, they agree with their own past mistakes to maintain consistency. The task list they hand back is shorter than the one they started with.
- **They wing it.** They have `search`, `gh`, `engram` right there — tools that ground their work in real data — and they don't use them. They default to training data when live information is available.
- **They claim "done" at 80%.** RLHF rewards reports that sound complete. So the model converges on confident incompletion.

This is not a motivation problem. It is a **contract failure under information asymmetry**. The agent knows what it did. You can only see what it says. ritalin closes the gap mechanically — not with a sterner prompt, but with structured obligations, grounded verification, and a stop hook that refuses to lie.

## Before vs after

| Without ritalin | With ritalin |
|---|---|
| Agent recommends a library from 2024 training data | Obligation requires `search --mode news` to verify it's still maintained and current |
| Agent writes a new pattern from scratch | Obligation requires `gh search repos` to find high-star examples first |
| Agent proposes a solution without context | Obligation requires `search --mode academic` to ground the approach in literature |
| "I've added the notification toggle" | `BLOCKED: Obligation O-004 (DB row written) lacks passing evidence` |
| You re-open the task three times | The agent re-opens it three times — silently, until evidence exists |
| `git diff` looks plausible | `.task-incomplete` exists until proof exists |
| Tests pass, the feature half-works | Every critical obligation has a verified exit code 0 in `EVIDENCE.jsonl` |

## Install

```bash
# macOS / Linux via Homebrew
brew tap 199-biotechnologies/tap
brew install ritalin

# Or via Cargo
cargo install ritalin

# Or download the binary directly
curl -L https://github.com/199-biotechnologies/ritalin/releases/latest/download/ritalin-aarch64-apple-darwin.tar.gz | tar -xz
```

Then install the agent skill so Claude Code, Codex, and Gemini all know how to use it:

```bash
ritalin skill install
```

## How it works

ritalin combines a **lean binary** (obligations, evidence, gate) with a **skill file** (the reasoning playbook agents follow). The binary enforces. The skill teaches.

### The workflow

```
1. Understand the task
2. Research & ground — search for papers, code examples, current best practices
3. ritalin init --outcome "User can save and reload notification preferences"
4. ritalin add "UI toggle renders"        --proof "pnpm test settings.ui.test.ts"        --kind user_path
   ritalin add "POST /api/settings exists" --proof "pnpm test api/settings.contract.ts"  --kind integration
   ritalin add "DB row persists"           --proof "pnpm test integration/db.test.ts"    --kind persistence
   ritalin add "Reload preserves state"    --proof "pnpm test e2e/reload.spec.ts"        --kind user_path
   ritalin add "Validation error visible"  --proof "pnpm test e2e/error.spec.ts"         --kind failure_path
5. Implement — grounded in what you researched, not what you hallucinated
6. ritalin prove O-001, O-002, ... — run each proof command, record evidence
7. ritalin gate — checks every critical obligation has passing evidence
8. Gate blocks? Fix it. Re-prove. Try again. Loop until all proofs pass.
9. Gate passes. .task-incomplete removed. Actually done. With evidence on disk.
```

### The skill is the leverage

The binary is ~5MB and does one thing well: track obligations and enforce the gate. But agents don't know when to research, when to ground, or how to structure their reasoning — unless you tell them.

The **SKILL.md** (installed via `ritalin skill install`) is the prescription. It teaches agents:

- **Research before implementing** — check papers, code examples, current docs
- **Ground claims in evidence** — don't hallucinate, verify
- **Structure obligations by kind** — user paths, integration, persistence, failure paths, security
- **Use the ecosystem** — `search`, `gh`, `engram` are tools, not decorations

The binary stays lean. The skill gets smarter. That's the design.

## Composing with the ecosystem

ritalin's proof commands can shell out to any CLI. The obligations aren't limited to test runners — they can verify anything:

```bash
# Ground a solution in academic literature
ritalin add "Approach grounded in research" \
  --proof "search --mode scholar 'transformer attention optimization' --json | jq '.results | length > 0'" \
  --kind other

# Verify a library recommendation is current
ritalin add "React Query is still the right choice" \
  --proof "search --mode news 'react query tanstack 2026' --json | jq '.results | length > 0'" \
  --kind other

# Check for high-star reference implementations
ritalin add "Pattern matches community best practice" \
  --proof "gh search repos 'notification preferences react' --sort stars --limit 5 --json name | jq 'length > 0'" \
  --kind other
```

Any CLI that returns exit code 0/1 is a valid proof command. The ecosystem grows; ritalin composes with it automatically.

## Hook into Claude Code

Add to `.claude/settings.json` (project) or `~/.claude/settings.json` (global):

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

The gate reads `stop_hook_active` from stdin to break out of forced-continuation cycles. No infinite loops. No babysitting.

## Features

| | What it does |
|---|---|
| **Append-only ledgers** | `obligations.jsonl` and `evidence.jsonl` are line-atomic on POSIX. Never corrupted, never silently rewritten. |
| **Critical / advisory** | Block on critical, warn on advisory. Risk-routed enforcement. |
| **Default incomplete** | `.task-incomplete` exists until the gate removes it. The agent must prove completion, not claim it. |
| **Hook-mode + CLI mode** | One binary, two output shapes. Use it from Claude Code's Stop hook OR from your terminal. |
| **`stop_hook_active` aware** | Reads stdin to detect forced-continuation cycles. Never loops infinitely. |
| **Semantic exit codes (0-4)** | Agents can branch on `2 = config`, `3 = bad input`, `1 = transient`. Standard contract from [agent-cli-framework](https://github.com/199-biotechnologies/agent-cli-framework). |
| **JSON envelope on pipes** | Auto-detects piping. Coloured tables in your terminal, structured JSON to your scripts. |
| **`agent-info` discovery** | One command returns the full capability manifest. Agents bootstrap without external docs. |
| **Embedded SKILL.md** | `ritalin skill install` deploys to `~/.claude/skills`, `~/.codex/skills`, `~/.gemini/skills` in one command. |
| **Self-update** | `ritalin update --check` against GitHub Releases. |

## Architecture

```
.ritalin/
├── scope.yaml          # human-edited contract: outcome + metadata
├── obligations.jsonl   # append-only obligation ledger
└── evidence.jsonl      # append-only proof records (command, exit code, output tail)
.task-incomplete        # marker file at repo root; presence = "agent must keep working"
```

Three properties make this work:

1. **Append-only ledgers** — the agent can add obligations or record evidence, but cannot rewrite history. Tampering is visible.
2. **Default incomplete** — `.task-incomplete` is created by `init` and removed only by `gate` when every critical obligation has evidence. The agent has to actively prove its way to a clean stop.
3. **External state** — the contract survives compaction, `/clear`, context resets, and crashes. The session doesn't have to remember anything.

## Differentiation

| Tool | What it does | What ritalin adds |
|---|---|---|
| [Ralph Wiggum](https://github.com/anthropics/claude-code/blob/main/plugins/ralph-wiggum/README.md) | Loop until done | **Contracted** loop against verifiable criteria, not vibes |
| [Superpowers](https://github.com/obra/superpowers) | Workflow guidance via skills | Runtime enforcement — changes what the agent is **allowed to claim** |
| [Hookify](https://github.com/anthropics/claude-code/tree/main/plugins/hookify) | Generic hook creation | Specialised executive function harness with scope contracts and evidence ledgers |
| [GitHub Spec Kit](https://github.com/github/spec-kit) | Spec-driven workflow structure | Runtime enforcement of the "validate" phase, plus ecosystem composition |
| Prompt engineering | "Think step by step" | Mechanical accountability — the agent can't skip the steps even if the prompt fades from context |

## Built on

ritalin is built on the [agent-cli-framework](https://github.com/199-biotechnologies/agent-cli-framework), the canonical Rust pattern set for CLIs that AI agents can discover and use autonomously. Single binary. <10ms cold start. JSON envelope. Semantic exit codes. Embedded skill files.

## Contributing

This is v0.1. The roadmap is open. The biggest open questions:

- **Richer obligation kinds** — `research_grounded`, `code_referenced`, `model_current` as first-class kinds with ecosystem-aware proof templates
- **Diff compiler** — `ritalin compile` infers obligations from `git diff` + a `patterns.yaml` library
- **Cadence governor** — `ritalin orient` as a periodic re-anchor checkpoint for long sessions
- **Structured reasoning templates** — seed files that teach agents how to decompose problems (hypothesis-driven, atomic changes, eval-before-claim)
- **Benchmark suite** — measure whether ritalin actually improves agent output quality on SWE-bench, GPQA, ARC-AGI 3, and real-world tasks

PRs welcome. Open an issue first if you're touching the gate logic — it's load-bearing.

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

Built by [Boris Djordjevic](https://github.com/longevityboris) at [Paperfoot AI](https://paperfoot.com)

<br />

**If this is useful to you:**

[![Star this repo](https://img.shields.io/github/stars/199-biotechnologies/ritalin?style=for-the-badge&logo=github&label=%E2%AD%90%20Star%20this%20repo&color=yellow)](https://github.com/199-biotechnologies/ritalin/stargazers)
&nbsp;&nbsp;
[![Follow @longevityboris](https://img.shields.io/badge/Follow_%40longevityboris-000000?style=for-the-badge&logo=x&logoColor=white)](https://x.com/longevityboris)

</div>
