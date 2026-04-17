# Contributing to ritalin

Three-step contribution process.

## 1. Open an issue first

If you're touching the gate logic, scope contract format, or evidence ledger, open an issue describing what you want to change and why. These are load-bearing — we'd rather discuss before you write code.

For docs, examples, and new patterns, skip this step. Just open the PR.

## 2. Build and test

```bash
git clone https://github.com/paperfoot/ritalin-cli
cd ritalin
cargo build
cargo test
```

Smoke test the binary against the canonical workflow:

```bash
mkdir /tmp/ritalin-test && cd /tmp/ritalin-test
~/Projects/ritalin/target/debug/ritalin init --outcome "test"
~/Projects/ritalin/target/debug/ritalin add "must pass" --proof "true"
~/Projects/ritalin/target/debug/ritalin prove O-001
~/Projects/ritalin/target/debug/ritalin gate
# Should print PASS, exit 0, and remove .task-incomplete
```

## 3. Open the PR

- Keep PRs focused. One change per PR.
- Match the existing code style. Run `cargo fmt`.
- If you add a command, update `agent_info.rs` AND `SKILL.md`. The capability manifest is a contract.
- Sign-off optional, license is MIT.

## Design rules

These exist to keep ritalin small and load-bearing:

- **Append-only ledgers stay append-only.** No "edit" or "delete" commands. Mistakes are amended with new entries.
- **The binary stays lean.** ritalin composes with the ecosystem (`search`, `gh`, `engram`, etc.) via proof commands. Don't build network capabilities, memory, or search into the binary. Shell out.
- **The skill is the leverage.** The SKILL.md teaches agents how to reason. The binary enforces the contract. Most improvements to agent quality come from improving the skill, not adding commands.
- **Keep `agent-info` honest.** Every command listed there must be routable. Drift is a P0 bug.
- **Semantic exit codes only.** 0-4. No custom codes. The contract comes from [agent-cli-framework](https://github.com/paperfoot/agent-cli-framework).
- **Two output shapes max.** JSON envelope on pipes, coloured human on TTY. Stop hook decisions are the only exception.

## Areas that need help

- **Proof templates** — pre-built proof commands for the `research_grounded`, `code_referenced`, and `model_current` obligation kinds that compose with ecosystem CLIs
- **Diff compiler** — `ritalin compile` to infer obligations from `git diff` against a `patterns.yaml`
- **Structured reasoning templates** — seed files for common reasoning patterns (hypothesis-driven, decomposition, grounding-first)
- **Benchmark suite** — measure whether ritalin actually improves agent output quality on SWE-bench, GPQA, ARC-AGI 3, and real-world tasks
- **Pattern library** — seed `patterns.yaml` rules for common stacks (Next.js, Rails, FastAPI, etc.)
- **Cadence governor** — `ritalin orient` as a periodic re-anchor for long sessions

## Code of conduct

Be kind. Disagree with the code, not the person. If something feels off, open an issue and we'll work it out.
