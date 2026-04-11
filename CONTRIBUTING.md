# Contributing to ritalin

Three-step contribution process.

## 1. Open an issue first

If you're touching the gate logic, scope contract format, or evidence ledger, open an issue describing what you want to change and why. These are load-bearing — we'd rather discuss before you write code.

For docs, examples, and new patterns, skip this step. Just open the PR.

## 2. Build and test

```bash
git clone https://github.com/199-biotechnologies/ritalin
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
- **No prompt surgery.** ritalin enforces; it does not coach. The mechanism is the message.
- **Keep `agent-info` honest.** Every command listed there must be routable. Drift is a P0 bug.
- **Semantic exit codes only.** 0–4. No custom codes. The contract comes from [agent-cli-framework](https://github.com/199-biotechnologies/agent-cli-framework).
- **Two output shapes max.** JSON envelope on pipes, coloured human on TTY. Stop hook decisions are the only exception.

## Areas that need help

- **Diff compiler** — `ritalin compile` to infer obligations from `git diff` against a `patterns.yaml`
- **Pattern library** — seed `patterns.yaml` rules for common stacks (Next.js, Rails, FastAPI, etc.)
- **Browser verifier** — pixel-diff proof for UI claims via headless Chrome
- **Multi-language SKILL.md** — currently English only

## Code of conduct

Be kind. Disagree with the code, not the person. If something feels off, open an issue and we'll work it out.
