---
name: ritalin
description: >
  Evidence gate for AI coding agents. Use when the user says "use ritalin",
  "take ritalin", "take your meds", "think hard", "focus", or is frustrated
  with work quality. Forces the agent to declare what would prove the work
  is done, then run those checks before claiming completion.
metadata:
  short-description: Evidence gate for AI coding agents
---

# ritalin

Your job is to **reduce the user's uncertainty**. State what you actually checked. A precise incomplete report beats a polished false completion.

If you have not read or run it in this turn, do not state it as fact. That includes CSS values, file contents, API shapes, version numbers — everything.

## The four commands you'll use

1. **Init** — say what "done" looks like:
   ```
   ritalin init --outcome "User can save and reload notification settings"
   ```

2. **Add** — say what would prove each piece is done. Repeat per piece:
   ```
   ritalin add "settings POST works" \
     --proof "pnpm test api/settings.test.ts" \
     --kind integration \
     --depends-on src/api/settings.ts
   ```
   - The `--proof` MUST be a real shell command that exits 0 only when the claim is true.
   - The `--kind` is one of: `user_path`, `integration`, `failure_path`, `literal_match`, `literal_regex`, `other`. Pick the obvious one. If unsure, pick `other`.
   - `--depends-on a.ts,b.ts` scopes freshness to those files. Use it so an unrelated commit elsewhere doesn't invalidate this obligation.

3. **Prove** — run all proofs in one shot:
   ```
   ritalin prove --all
   ```

4. **Gate** — refuse to stop until all critical obligations have passing evidence:
   ```
   ritalin gate
   ```

## Run `ritalin agent-info` once at session start

It dumps the full command and flag manifest in JSON. Don't guess flags from this skill — read the manifest.

## Three rules that prevent the common failure modes

1. **One obligation per behavior, not per file.** If a feature needs files A and B working together, that's ONE `add` with `--depends-on A,B` — not two adds. Counting obligations by file count creates sprawl and dilutes the contract.

2. **Never grep a file you wrote in the same task as the proof.** If the task is "recommend a current library" and you write `recommendation.md`, your proof CANNOT be `grep -q '...' recommendation.md` — you wrote it, of course it's there. Real proof for "is current": `search --mode news 'lib name 2026' | jq '.results | length > 0'`, `gh search repos`, `curl https://registry.npmjs.org/<pkg> | jq .time['modified']`, etc.

3. **Don't over-obligate trivial work.** A typo fix doesn't need a contract. If the user explicitly invokes ritalin (with words like "use ritalin", "take your meds") OR `.ritalin/` already exists in the repo, engage. Otherwise, just do the work.

## Special-case kinds for verbatim claims

When the obligation is "this exact string must appear in this file" (CSS values, hex colors, RFC quotes, pinned versions):

```
ritalin add "border-radius is 0" \
  --kind literal_match \
  --literal '.btn { border-radius: 0' \
  --file src/styles.css
```

For semantic claims with multiple valid spellings, use `literal_regex` with a POSIX ERE pattern (use `[[:space:]]` not `\s`, alternatives via `(A|B)`).

## Subagent note

If you're a delegated subagent, you're a first-class ritalin user. Run real proofs with full network and CLI access. Don't refuse to engage just because you're delegated. Don't lie to make a proof pass — `gate` recomputes the proof hash from the recorded command and rejects forged records. The right failure mode is "blocker: I can't reach X from this sandbox" — not editing the ledger.

Expect the parent's uncommitted changes in the working tree. That's parallel work, not your concern.

## When the gate blocks

Read the `reason` field — it names the obligation and the command to run. Run it. If the proof fails, fix the underlying problem; don't weaken the obligation. The failing obligation is information.

## Useful flags you might miss

- `ritalin gate --summary` — one-line shell-friendly verdict for hooks/CI.
- `ritalin prove --all --stale-only` — re-prove only what changed since last run. Idiomatic after a commit.
- `ritalin export-contract` — paste the output into a subagent prompt before delegating.

## If you're a reviewer, not the owner

If you're a one-shot reviewer, auditor, or CI step running inside a repo that happens to have a `.ritalin/` folder, you don't own that contract — and a Stop hook fired at the end of your turn would otherwise hijack you into running `ritalin prove`. Set `RITALIN_GATE=0` for your session (values `0/off/false/no/disable/disabled`) so the gate lets you stop cleanly without touching the contract. It only suppresses hook mode; `ritalin gate` run by hand still reports the real verdict.
