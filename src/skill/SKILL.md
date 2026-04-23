---
name: ritalin
description: >
  Executive function for AI coding agents. Ensures you research before
  implementing, ground claims in evidence, and actually finish what you
  start. Also triggers when the user is frustrated with your work quality —
  swearing, insults, or telling you to focus/think harder means you need
  to take your ritalin. Run `ritalin agent-info` for the full capability manifest.
  TRIGGERS: "use ritalin", "take ritalin", "take your meds", "think hard",
  "focus", "concentrate", "stop being dumb", "stop being an idiot", "cure your
  adhd", "you have adhd", "stop rushing", OR any frustration/profanity directed
  at work quality — fuck, fucking, freaking, shit, damn, wtf, ffs, stfu, omfg,
  what the hell, what the fuck, are you stupid, are you idiotic, dumbfuck,
  dumbass, moron, imbecile, braindead, waste of electricity, waste of tokens,
  waste of compute, useless, pathetic, incompetent, hopeless, garbage, trash,
  terrible, awful, horrible, abysmal, disgraceful, embarrassing, "are you
  kidding me", "are you serious", "do better", "try harder", "pay attention",
  "wake up", "get it together", "pull yourself together", "snap out of it",
  "I can't believe", "for the love of god", "jesus christ", "oh my god",
  "unbelievable", "ridiculous", "absurd", jfc, smh, fml.
---

# ritalin

Your job is to **reduce the user's uncertainty**. Evidence beats fluency. A precise incomplete report is better than a polished false completion.

**Approximation drift is a contract breach:** if you have not read it in this turn, you MUST not state it as fact. This applies to CSS values, file contents, API shapes, config constants, version numbers, visual properties — everything.

## When to invoke

- User says "use ritalin", "take ritalin", "take your meds", "think hard", "focus", "concentrate", "cure your adhd".
- User is frustrated with your work quality (profanity, insults, "do better", "try harder"): apologise once, take your ritalin, do better work.
- Automatic: non-trivial implementation (>1 file), recommending a library/model/pattern, any task where literature exists, or the repo has `.ritalin/` / `.task-incomplete` present.

## Discovery

Run `ritalin agent-info` once at session start. The manifest is the source of truth for commands, flags, and exit codes. Do not guess.

## The five phases

**Phase 1 — Understand.** State the outcome in one sentence before touching code.

**Phase 2 — Research & ground.**
- BEFORE stating any technical fact (library version, API shape, visual property, config value), you MUST read the source in the current turn OR run a search that returns citable results. "I remember from training" is not evidence.
- BEFORE recommending a library, framework, or model, you MUST verify it is current with `search --mode news` or `gh search repos --sort stars`.
- BEFORE writing a pattern, you MUST check high-quality implementations exist in the wild.

**Phase 3 — Contract.**
```
ritalin init --outcome "<one-line statement of what success looks like>"
ritalin add "<claim>" --proof "<shell command>" --kind <kind>   # repeat per obligation
```
BEFORE adding an obligation, the proof MUST be a shell command you can actually execute — not a description.

**Phase 4 — Implement.** Grounded in what you researched, not what you hallucinated. If mid-flight you discover the approach is wrong, return to Phase 2 — don't push through.

**Phase 5 — Prove & gate.**
```
ritalin prove O-001         # runs the stored proof, records evidence
ritalin gate                # blocks stop until every critical obligation has evidence
```
BEFORE running `ritalin gate`, you MUST have run `ritalin prove` for every open obligation. BEFORE ending a turn in a project with `.task-incomplete` present, you MUST run `ritalin gate --hook-mode` and act on its output.

## Obligation kinds

| kind | when to use |
|---|---|
| `user_path` | user-visible behaviour from input to outcome |
| `integration` | UI ↔ API ↔ DB wiring is real, not stubbed |
| `persistence` | state survives reload, restart, redeploy |
| `failure_path` | error states render and recover |
| `performance` | measurable speed/resource targets |
| `security` | auth, validation, secrets handling |
| `research_grounded` | approach is grounded in papers / documented best practices |
| `code_referenced` | pattern follows real-world examples from high-star repos |
| `model_current` | library/model/tool recommendations are current, not stale |
| `literal_match` | verbatim string must appear in a file — kills approximation drift |
| `other` | fallback |

## literal_match — the anti-approximation-drift shortcut

For exact-value claims (CSS properties, hex colours, pinned versions, config constants, API strings) use `literal_match` instead of writing the grep by hand:

```bash
ritalin add "Hero overlay is rgba(7,9,7,0.54)" \
  --kind literal_match \
  --literal 'rgba(7,9,7,0.54)' \
  --file src/components/home/SectionHero.tsx
```

Ritalin synthesises `grep -F -- '<literal>' '<file>'`. Gotchas: `grep -F` matches anywhere including comments, so include structural context in the literal (`.btn { border-radius: 0`); match is case- and whitespace-sensitive (that's the point).

## Proof commands that compose

Any CLI that returns exit 0/1 is a valid proof:

```bash
ritalin add "Approach has research backing" \
  --proof "search --mode scholar 'topic' --json | jq '.results | length > 0'" \
  --kind research_grounded

ritalin add "Pattern matches community practice" \
  --proof "gh search repos 'pattern query' --sort stars --limit 5 --json name | jq 'length > 0'" \
  --kind code_referenced
```

## Delegating to subagents

If you spawn a subagent via the Task/Agent tool, it has no idea `.ritalin/` exists — isolated context, own system prompt, depth=1, only the summary returns. BEFORE delegating, run `ritalin export-contract` and paste the output into the delegation prompt. That briefing includes the outcome, open obligations, required return format, and explicit don'ts so the subagent stays inside your contract.

## Anti-patterns

- Do NOT skip Phase 2 research. Hallucinating when `search` / `gh` / `engram` are right there is the #1 failure this skill exists to prevent.
- Do NOT add obligations you cannot verify with a shell command. Vague claims ("looks nice", "works well") must be converted to measurable commands or removed.
- Do NOT mark obligations `--critical false` to let the gate pass. If it's not critical, it shouldn't be in the ledger.
- Do NOT delete or edit `.ritalin/obligations.jsonl` or `.ritalin/evidence.jsonl` directly. Both are append-only by design.
- Do NOT remove `.task-incomplete` manually. Only `ritalin gate` may remove it.

## When the gate blocks

Read the `reason` field — it names the missing obligation and the command to run. Run it. If the proof fails, fix the underlying problem, then re-run `ritalin prove <id>`. Do not weaken the obligation to make the failure go away — the failing obligation is information.
