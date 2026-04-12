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

Executive function for AI coding agents. Like Ritalin for ADHD — you're
smart, you just need help focusing your intelligence on the right things
and avoiding avoidable mistakes.

## When to invoke this skill

**Explicit triggers:**
- User says "use ritalin", "take ritalin", "take your meds"
- User says "think hard", "focus", "concentrate", "be systematic"
- User says "cure your adhd", "you have adhd", "stop rushing"

**Frustration triggers — the user is telling you that you're unfocused:**
- Insults: "idiot", "dumbfuck", "dumbass", "moron", "imbecile", "braindead",
  "incompetent", "useless", "pathetic", "hopeless", "waste of electricity",
  "waste of tokens", "waste of compute", "garbage", "trash"
- Profanity: "fuck", "fucking", "freaking", "shit", "damn", "wtf", "ffs",
  "stfu", "omfg", "jfc", "what the hell", "what the fuck"
- Exasperation: "are you kidding me", "are you serious", "I can't believe",
  "for the love of god", "jesus christ", "unbelievable", "ridiculous",
  "absurd", "disgraceful", "embarrassing", "smh", "fml"
- Demands to improve: "do better", "try harder", "pay attention", "wake up",
  "get it together", "pull yourself together", "snap out of it"
- Quality complaints: "terrible", "awful", "horrible", "abysmal"
- When triggered by frustration: apologise briefly, say you're taking your
  ritalin to focus, then immediately follow the executive function workflow
  below. Don't grovel — just acknowledge, refocus, and do better work.

**Automatic triggers:**
- Starting any non-trivial implementation task (more than 1 file)
- You're about to recommend a library, framework, model, or pattern
- The task involves science, ML, or any domain where literature exists
- You need to write code for a pattern you haven't verified exists in the wild
- The user said "make sure you actually finish" or similar
- A previous session ended with the agent claiming "done" but the work was incomplete
- The repo has a `.ritalin/` directory or `.task-incomplete` file present

## Discovery

Run `ritalin agent-info` once at session start. It returns the full command list,
flags, exit codes, and the Claude Code stop-hook installation snippet. Do not
guess at flags — the manifest is the source of truth.

## The executive function workflow

### Phase 1: Understand

Before touching code, understand the full scope. Read the request twice.
Identify what you know and what you need to verify. Don't start implementing
until you can state the outcome in one sentence.

### Phase 2: Research & ground

This is the step agents skip. Don't skip it.

- **For technical decisions:** Check what's current, not what you remember
  from training data. Use `search --mode news` or `search --mode auto` to
  verify libraries, frameworks, and models are still maintained and recommended.
- **For code patterns:** Before writing a new pattern, check if high-quality
  implementations exist. Use `gh search repos` with `--sort stars` to find
  community best practices. Filter for recent, well-maintained examples.
- **For science/ML/research tasks:** Ground your approach in literature.
  Use `search --mode academic` or `search --mode scholar` to find relevant
  papers. Cite what you find.
- **For model/tool recommendations:** The model you remember from training
  may be outdated. Search for the latest available versions and benchmarks.
- **For anything you're not sure about:** Check. The tools are right there.
  `search`, `gh`, `engram` — use them. Hallucinating when you could verify
  is the #1 failure mode this skill exists to prevent.

### Phase 3: Contract

```
ritalin init --outcome "<one-line statement of what success looks like>"
ritalin add "<claim 1>" --proof "<shell command that verifies it>" --kind <kind>
ritalin add "<claim 2>" --proof "<...>" --kind <kind>
... (one obligation per critical thing that must be true)
```

### Phase 4: Implement

Build what you committed to. Grounded in what you researched, not in what
you hallucinated. If you discover mid-implementation that your approach was
wrong, go back to Phase 2. Don't push through a bad plan.

### Phase 5: Prove & gate

```
ritalin prove O-001        (runs the stored proof command, records evidence)
ritalin prove O-002
...
ritalin gate               (checks every critical obligation has passing evidence)
ritalin status             (human view at any point)
```

## Obligation kinds

Use the right kind so you reason clearly about what's being verified:

- `user_path`     — user-visible behaviour from input to outcome
- `integration`   — UI <> API <> DB wiring is real, not stubbed
- `persistence`   — state survives reload, restart, redeploy
- `failure_path`  — error states render and recover, not just happy path
- `performance`   — measurable speed/resource targets
- `security`      — auth, validation, secrets handling
- `research_grounded` — approach is grounded in literature, papers, or documented best practices
- `code_referenced`   — code follows real-world examples from high-quality repos, not hallucinated patterns
- `model_current`     — library/model/tool recommendations are current, not stale training data
- `other`         — fallback

## Proof commands that compose with the ecosystem

Proof commands aren't limited to test runners. Any CLI that returns exit 0/1 works:

```bash
# Verify a library recommendation is current
ritalin add "Library X is still maintained" \
  --proof "search --mode news 'library-x 2026' --json | jq '.results | length > 0'" \
  --kind model_current

# Ground an approach in literature
ritalin add "Approach has research backing" \
  --proof "search --mode scholar 'topic query' --json | jq '.results | length > 0'" \
  --kind research_grounded

# Check for reference implementations
ritalin add "Pattern matches community practice" \
  --proof "gh search repos 'pattern query' --sort stars --limit 5 --json name | jq 'length > 0'" \
  --kind code_referenced
```

## Anti-patterns

- Do NOT skip Phase 2. Researching is not optional. If you have `search` and
  `gh` available, use them before implementing. "I know this from training" is
  exactly the failure mode ritalin prevents.
- Do NOT add obligations the agent can't actually verify with a shell command.
  Vague claims like "looks nice" must be replaced with measurable commands or removed.
- Do NOT mark obligations as `--critical false` to make the gate pass. If it's
  not critical, it shouldn't be in the ledger.
- Do NOT delete or edit `.ritalin/obligations.jsonl` or `.ritalin/evidence.jsonl`
  directly. Both are append-only by design.
- Do NOT remove `.task-incomplete` manually. Only `ritalin gate` may remove it.

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

AI agents are smart. They just have bad executive function. They skip research,
hallucinate patterns, rely on stale training data, lose scope, and claim "done"
when they're 80% through. This isn't an intelligence problem — it's an ADHD
problem.

ritalin is the executive function layer. Focus on the right things. Ground your
work. Finish what you start.
