# The Macro Trend: PR = Prompt Request
## Why Ritalin Sits in the Middle of It

---

## The Genealogy (Jan 2023 → April 2026)

| Date | Who | What |
|------|-----|------|
| Jan 2023 | Karpathy | "The hottest new programming language is English" |
| Jan 19, 2026 | Steinberger | First "PROMPT REQUEST" post |
| Jan 26, 2026 | Steinberger | "/r/pull/prompt request" |
| Jan 8, 2026 | dbreunig | Releases **whenwords**: 0 lines of code, just spec + 750 YAML tests |
| Jan 2026 | Addy Osmani | "How to write a good spec for AI agents" — Specify→Plan→Tasks→Implement→Validate |
| Feb 2026 | Steinberger | Joins OpenAI; OpenClaw moves to foundation |
| Feb 2026 | Pragmatic Engineer | "I ship code I don't read" article |
| Mar 4, 2026 | dbreunig | "Spec-Driven Development Triangle" + Plumb CLI |
| Mar 21, 2026 | Karpathy | "It's not really about prompting, it's about context and spec engineering" |
| Mar 21, 2026 | Karpathy | **"Agents do not listen to my AGENTS.md instructions... I could use hooks but a shrug is easier"** |
| Mar 26, 2026 | Karpathy | "Build menugen" — entire DevOps lifecycle should become code |
| Mar 28, 2026 | Steinberger | "I see this as a prompt request :)" |
| Apr 2, 2026 | Karpathy | LLM Knowledge Base post (54K likes) — "manipulating knowledge, not code" |
| Apr 4, 2026 | Karpathy | **VIRAL** "PR = Prompt Request" post (272K views) + Idea File pattern (25K likes) |
| Apr 4-11, 2026 | Community | The reframe goes mainstream |

---

## The Convergence

Five independent threads converge on the same insight:

### Thread 1: Code Is Not the Artifact (Karpathy)
- 2023: English is the new programming language
- 2026: Idea File pattern — share the idea, not the code
- 2026: "Manipulating knowledge, not code"

### Thread 2: PRs Are Prompt Requests (Steinberger)
- "Pull requests are dead"
- "I ship code I don't read"
- "More interested in seeing the prompts than the code"
- OpenClaw: 2,000+ open PRs growing at 600/day, needs AI to triage

### Thread 3: Spec-Only Software (dbreunig)
- whenwords: 1,000+ stars, 0 lines of code, only SPEC.md + 750 YAML tests
- "Ghost library" — distributed as spec, materialized as code
- Spec-Driven Development Triangle: Specs ↔ Tests ↔ Code in feedback loop
- Plumb CLI: extracts decisions from diffs and agent traces

### Thread 4: Spec-Driven Workflow (Addy Osmani / GitHub Spec Kit)
- Specify → Plan → Tasks → Implement → Validate
- "Don't move to the next phase until current is fully validated"
- Spec must include: commands, testing, structure, code style, git workflow, boundaries
- Three-tier boundaries: Always do / Ask first / Never do

### Thread 5: Agents Don't Follow Instructions (Karpathy admits)
- AGENTS.md instructions ignored
- Hooks could fix it but nobody has built the tooling
- "A shrug is easier" — i.e. nobody is solving this yet

---

## The Gap Everyone Sees, Nobody Has Filled

**The macro insight**: code is just proof of intent. Specs/prompts are the artifact.

**The gap**: how do you trust the proof?

| Person | The Question They're Asking |
|--------|---------------------------|
| Steinberger | "How do I review 2,000 open PRs without reading code?" |
| Karpathy | "Why don't agents follow my AGENTS.md?" |
| Pragmatic Engineer reader | "Considerable gap: Tokens, Time, Testing" |
| @Anon93519648090 | "Maintainers may start requiring PRs to include what models were used" |
| @thenanyu | "PR as prompt request? generally a well crafted issue has things like acceptance criteria and mock ups" |
| @emonuxui | "PRs still serve as review and accountability boundaries" |
| Addy Osmani | "Don't move to the next phase until current is fully validated" — but who enforces this? |

**The market signal is loud and clear**: the prompt-request world needs a verification layer. Nobody has built it yet.

---

## How Ritalin Fits In

### The Old Story (Wrong)
"Ritalin prevents AI agents from being lazy."

### The New Story (Right)
**"Ritalin is the verification layer for the prompt-request era."**

Or even sharper: **"Ritalin is git's missing companion for spec-driven development."**

### The Insertion Point

```
SPEC-DRIVEN DEVELOPMENT WORKFLOW:

  Specify  →  Plan  →  Tasks  →  Implement  →  Validate
                                                    ↑
                                                RITALIN
                              (the runtime that enforces "don't move on until validated")
```

### What Ritalin Becomes

| Phase | What Ritalin Does |
|-------|-------------------|
| **Specify** | Spec linter rejects vague criteria ("works well", "handle errors") |
| **Plan** | Compiles SCOPE.json from intent + architecture |
| **Tasks** | Generates obligation graph (TDAD-style: which tests prove which criteria) |
| **Implement** | Sequential gating; one obligation at a time; .task-incomplete marker |
| **Validate** | Stop hook reads SCOPE.json + EVIDENCE.jsonl, blocks unless every critical obligation has proof |

### The Differentiation

| Tool | Layer | What Ritalin Adds |
|------|-------|-------------------|
| GitHub Spec Kit | Workflow structure | Runtime enforcement of validation |
| Plumb (dbreunig) | Decision extraction from diffs | Tamper-resistant evidence ledger |
| whenwords | Spec-only library demo | The runtime that runs the YAML tests at the right moment |
| Addy Osmani's spec template | What to write | The verifier that proves you wrote enough |
| Ralph Wiggum | Loop until done | Loop only against verifiable criteria |
| Superpowers | Workflow guidance | Runtime law that can't be ignored |
| Hookify | Generic hooks | Specialized PR=Prompt Request infrastructure |
| Claude Code Plan Mode | Read-only planning | Read-only AND tamper-resistant AND evidence-based |

---

## The 5-Year Vision (Sharpened)

**Year 1**: Ritalin CLI + Claude Code skill — solo developer use case

**Year 2**: Model-agnostic harness — Claude/Codex/Cursor/Windsurf/Copilot all hook into the same gate

**Year 3**: Standard schema for "proof of completion" — `.ritalin/proof.json` becomes a recognized artifact

**Year 4**: GitHub/GitLab integrate proof bundles into PR review — "proof bundle attached" badge required for AI-generated PRs

**Year 5**: SOC2 / ISO 27001 compliance frameworks recognize ritalin proofs as audit evidence for "AI-generated code review processes"

In the prompt-request era, **who reviews?** Not humans. Ritalin reviews. Ritalin becomes the **first-class verification layer of the new SDLC**.

---

## The Killer Demo (5 minutes)

**Setup**: Empty Next.js app, user wants notification preference toggle.

**Without ritalin**:
- "Add notification preference toggle to settings page"
- Claude builds UI button + 1 happy-path test
- Says: "Done! Notification toggle is wired up."
- Reality: No API endpoint, no DB persistence, no error path. Half-built.

**With ritalin**:
1. `ritalin init --outcome "User can save notification preference and see it persist after reload"`
2. `ritalin compile` — infers 6 obligations from intent + repo structure:
   - O-001 [critical]: UI toggle rendered (proof: pixel diff)
   - O-002 [critical]: Click handler wired (proof: e2e test)
   - O-003 [critical]: POST /api/settings exists (proof: contract test)
   - O-004 [critical]: DB row written (proof: integration test)
   - O-005 [critical]: Reload preserves state (proof: e2e test)
   - O-006 [critical]: Validation error visible (proof: e2e test)
3. Claude implements UI + 1 test, tries to stop
4. Stop hook fires `ritalin gate`
5. Ritalin output:
   ```json
   {"decision": "block", "reason": "Obligation O-003 (POST /api/settings exists) lacks evidence. Run: pnpm test api/settings.contract.test.ts"}
   ```
6. Claude writes the API route, contract test passes
7. Repeat for O-004, O-005, O-006
8. All 6 obligations have green evidence in EVIDENCE.jsonl
9. `ritalin gate` removes `.task-incomplete`, allows stop
10. Done. Actually done. With proof.

**Demo audience**: every developer who's been burned by "I'm done" lies.
