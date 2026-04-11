# Agent Findings: Novel Enforcement Techniques

## Most Impactful Finding: Stop-Phrase-Guard (Production Evidence)

From GitHub issue #42796 — user running 50+ concurrent agent sessions:

**5 categories of premature stopping (173 catches in 17 days):**
| Category | Triggers | Example phrases |
|---|---|---|
| Ownership dodging | 73 | "not caused by my changes", "existing issue" |
| Permission-seeking | 40 | "should I continue?", "want me to keep going?" |
| Premature stopping | 18 | "good stopping point", "natural checkpoint" |
| Known-limitation labeling | 14 | "known limitation", "future work" |
| Session-length excuses | 4 | "continue in a new session", "getting long" |

Hook fired 0 times before model regression, 173 times after = machine-readable canary signal.

## Hook Patterns

### Hookify (Anthropic Official Plugin)
- No-code rules via YAML frontmatter markdown files
- event types: bash, file, stop, prompt, all
- actions: warn, block
- `/hookify` command analyzes conversation to auto-detect fixable behaviors
- Rules activate immediately without restart

### PostToolUse Validators
- Chained validators: ruff linting + type checking in .claude/hooks/validators/
- Exit code 2 signals policy enforcement
- Multiple hooks run in parallel; any block = model continues

## Agentic Loop Patterns

### Verdent (76.1% SWE-bench Verified)
- Todo list anchoring: explicit structured checklist continuously read/updated
- Intelligent code review subagent: inspects diffs, flags risks, triggers repair
- Multi-model switching: GPT for review, Claude for coding

### Devin (67% PR merge rate)
- Checkpoint-based: Plan → Implement chunk → Test → Fix → Review → Next chunk
- Knowledge persistence: codify testing procedures and architectural patterns
- Fresh-start philosophy: comprehensive upfront instructions > correcting corrupted context

### TDAD (Test-Driven Agentic Development)
- 70% regression reduction (6.08% → 1.82%)
- Builds code-test dependency graph using AST parsing
- CRITICAL: Adding TDD *instructions* without graph context INCREASED regressions to 9.94%
- Agents need *which tests to check*, not *how to do TDD*
- Simplified 107-line instructions to 20 lines + targeted test mappings = 4x resolution

### OODA Loop Framework
- Pre-built .claude/agents/ configurations
- Observe → Orient → Decide → Act
- Enforced phasing prevents oversight

## Multi-Agent Verification

### Adversarial Code Review (Builder-Critic)
- Context swap required: close Builder session, start fresh Critic
- Critic evaluates only artifacts (spec + diff), not reasoning process
- Advanced: parallel Critic Lanes (Architect, SecOps, QA) + Moderator synthesis
- Caught silent architecture violation that passed all tests

### Agentic Rubrics (Scale Labs)
- Agent explores repo → produces structured rubric → patches scored against rubric
- Separate judge, no test execution required

### Chain of Verification (CoV)
- Separate verification from baseline draft
- Intermediate milestones output must pass

## Progressive Disclosure

### Working Memory Research
- Context drift causes ~65% of enterprise AI agent failures
- Agent Cognitive Compressor (ACC): bio-inspired bounded internal state
- HiAgent: hierarchical working memory management
- Persistent memory graphs: critical decisions as first-class entities

### Specification Engineering
- Verifiable: "done" defined in checks someone else can evaluate
- Constrained: explicit must/must-not/preferences
- Decomposable: independently executable chunks
- "You don't get to babysit the session — encode oversight up front"

## CLI Tools

### Superpowers Plugin (obra/superpowers)
- verification-before-completion: evidence before assertions
- test-driven-development: RED-GREEN-REFACTOR enforced
- requesting-code-review: reviews against plan, blocks on critical
- systematic-debugging: must be used before proposing any fix

### GitHub Spec Kit
- `specify` CLI: Specify → Tasks → Implement → Validate
- Don't move forward until current phase validated

### AgentSys
- 19 plugins, 47 agents, 40 skills
- Orchestration runtime: structured pipelines, gated phases
