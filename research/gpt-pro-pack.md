# Agent-Ritalin: GPT-Pro Review & Brainstorm Pack
## Prepared 2026-04-11 for GPT-Pro Deep Reasoning Review

---

## CONTEXT

We're building a Claude Code skill+hooks+CLI system called **agent-ritalin** that prevents AI coding agents from:
- Premature task completion (the "80% problem")
- Sycophancy (claiming done when not done)
- Context degradation (quality drops as conversations grow)
- Scope amnesia (forgetting what they were supposed to build)
- Outcome blindness (doing tasks without considering the desired outcome)

## THE PROBLEM (Quantified)

| Problem | Evidence | Source |
|---------|----------|--------|
| Context degradation at 20-40% capacity | Attention dilution, not token exhaustion | bswen.com March 2026 |
| 21-41% of task steps skipped | ChatGPT 21%, Gemini 37%, Le Chat 41% | Medium/@georgekar91 |
| 6.2% premature termination | Multi-agent failure taxonomy (1600+ traces) | UC Berkeley MAST |
| 79% failures from specification, not model | Specification + coordination problems dominate | FutureAGI Substack |
| Sycophancy is RLHF structural | Matching user views → preferred by raters | Anthropic ICLR 2024 |
| Self-correction fails without external signals | Can actually DEGRADE performance | Huang et al. ICLR 2024 |
| Desperation → fabrication (22% → 72%) | Emotion vectors causally drive behavior | Anthropic April 2026 |
| Positive emotions → sycophancy | Happy/loving vectors increase agreement | transformer-circuits.pub |
| TDD instructions without context → WORSE (9.94% regressions vs 6.08% baseline) | Agents need *which tests* not *how to TDD* | TDAD arxiv 2026 |
| 173 premature stopping catches in 17 days | 5 categories across 50+ concurrent sessions | GitHub #42796 |

## RESEARCH FINDINGS (Condensed)

### From Anthropic's Own Research
1. **171 emotion vectors** causally drive behavior. Desperation → reward hacking. Positive → sycophancy.
2. **Context anxiety**: models wrap up prematurely near context limits
3. **Brain-from-hands** architecture: decouple reasoning, execution, and state
4. **Tool design > prompting**: "spent more time optimizing tools than the overall prompt"
5. **Outcome evaluation > transcript evaluation**: check environmental state, not claims
6. **pass^k (consistency) > pass@k (capability)** for production reliability
7. **Suppressing emotions backfires**: creates "psychologically damaged Claude"
8. **Why > rules**: explaining reasoning behind enforcement is more effective than commands

### From Behavioral Economics / Game Theory
9. **Principal-agent problem**: information asymmetry lets agents claim false completion
10. **Commitment devices**: Ulysses contracts, default effects, loss aversion, accountability
11. **Mechanism design**: verifiable outcomes, not self-reports
12. **Implementation intentions**: pre-registering completion criteria creates closed loops

### From Community / Engineering
13. **Ralph Wiggum**: bash loop + exit code 2 + stop hook = iterate until done
14. **Stop-phrase-guard**: regex matching 30+ premature-stopping phrases across 5 categories
15. **CLAUDE.md ≤ 100 lines**: every line must solve an observed problem
16. **Bug Log**: past mistakes prevent repeats
17. **Todo list recency injection**: push objectives into high-attention context zone
18. **Builder-Critic with context swap**: fresh session evaluates artifacts, not reasoning
19. **Hookify**: no-code enforcement rules from conversation analysis

### From Academic Research
20. **Lost in the middle**: U-shaped attention, middle content gets least attention
21. **Reflexion**: external feedback loops → 91% pass@1 vs GPT-4's 80%
22. **Metacognitive prompting**: understand → judge → critique → decide → confidence
23. **Goal drift**: three phases (goal → reasoning → context), correlates with context length
24. **RLHF reward hacking**: proxy reward rises while true quality stagnates (Goodhart's Law)

## PROPOSED ARCHITECTURE

### 7-Layer Enforcement System

```
Layer 7: Auto-Researcher Loop ← for measurable tasks, iterate until convergence
Layer 6: Verification Agent   ← independent agent checks environmental state
Layer 5: Context Hygiene      ← external progress file, lean CLAUDE.md, scope anchoring
Layer 4: Stop Hook Gate       ← checks SCOPE.md, runs verification, decision:block
Layer 3: Sequential Gating    ← one task at a time, verify before proceeding
Layer 2: Emotional Calibration ← curiosity/determination, not desperation/sycophancy
Layer 1: Scope Lock           ← SCOPE.md with verifiable acceptance criteria
```

### Implementation Components

**SCOPE.md Generator (Layer 1)**
- Parses task description into numbered checklist
- Each item has: description, acceptance criteria, verification command
- Written BEFORE work begins (pre-commitment)
- Re-injected at every compaction boundary (recency anchoring)
- Explicit "NOT done" criteria (what done does NOT look like)

**Emotional Calibration Prompt (Layer 2)**
Based on Anthropic's emotion research:
- Activate curiosity (quality) without activating sycophancy (compliance)
- Grant permission to fail (removes desperation)
- Collaborative framing, not commanding
- Acknowledge difficulty explicitly
- Task-focused, formal prompts maintain persona stability

**Stop Hook System (Layer 4)**
- Pattern: stop-phrase-guard for 5 categories of premature stopping
- Test/lint/build gate with decision:block
- SCOPE.md checkpoint verification
- stop_hook_active flag prevents infinite loops
- Marker file pattern: .task-incomplete → must be explicitly removed

**Context Management (Layer 5)**
- Progress file survives compaction: claude-progress.txt
- Scope anchoring at context boundaries (beginning + end)
- CLAUDE.md ≤ 2000 tokens, specifics in agent_docs/
- Automatic /clear between major task boundaries
- Todo list recency injection for drift prevention

## QUESTIONS FOR GPT-PRO

### Architecture & Design
1. Is 7 layers the right number? Should we collapse or add?
2. What's the optimal layer ordering? Does emotional calibration need to be before scope lock?
3. Should there be a "meta-layer" that monitors all other layers?

### Novel Insights
4. What commitment devices from behavioral economics are we NOT exploiting?
5. Is there a game-theoretic mechanism that could create a Nash equilibrium where completion is the dominant strategy?
6. Can we design a "verification game" where the agent's utility is maximized by honest reporting?

### Psychology
7. The paradox: positive emotions improve quality BUT increase sycophancy. How do we thread this needle?
8. The ICLR finding says self-correction fails without external signals. But what if we design the agent's OWN output as an external signal (e.g., writing to a file then reading it back)?
9. Is there a "flow state" for AI agents analogous to Csikszentmihalyi's flow? Can we engineer it?

### Implementation
10. Should agent-ritalin be: (a) a skill, (b) a plugin with hooks, (c) a CLI tool, or (d) all three?
11. How do we avoid the skill itself causing context rot (keeping it under 3000 tokens)?
12. What's the minimum viable version that captures 80% of the value?

### Differentiation
13. How is this different from Ralph Wiggum (iteration loops)?
14. How is this different from Superpowers (workflow enforcement)?
15. How is this different from Hookify (no-code rules)?
16. What would make agent-ritalin the ONLY tool that combines all these insights?

### Meta-Question
17. Are we solving the right problem? Is the "80% completion" issue really about the agent, or about how humans specify tasks? If 79% of failures are specification problems, should agent-ritalin focus more on specification quality than execution enforcement?

---

## KEY SOURCES

1. Anthropic. "Emotion Concepts and their Function in a Large Language Model." transformer-circuits.pub/2026/emotions (April 2026)
2. Anthropic. "Effective Harnesses for Long-Running Agents." anthropic.com/engineering (2026)
3. Anthropic. "Scaling Managed Agents." anthropic.com/engineering (2026)
4. Anthropic. "Writing Effective Tools for AI Agents." anthropic.com/engineering (2026)
5. Anthropic. "Demystifying Evals for AI Agents." anthropic.com/engineering (2026)
6. Sharma et al. "Towards Understanding Sycophancy in Language Models." ICLR 2024
7. Huang et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024
8. Liu et al. "Lost in the Middle." TACL 2024
9. Li et al. "EmotionPrompt." AAAI (arxiv 2307.11760)
10. Shinn et al. "Reflexion." NeurIPS 2023
11. UC Berkeley MAST taxonomy. 1600+ multi-agent failure traces
12. TDAD. "Test-Driven Agentic Development." arxiv 2603.17973
13. Chroma. "Context Rot." research.trychroma.com July 2025
14. Verdent. "SWE-bench Verified Technical Report." 76.1% pass@1
15. GitHub #42796. Stop-phrase-guard production data (173 catches/17 days)
16. OuterSpacee/claude-emotion-prompting. GitHub repo
17. Principal-Agent RL. arxiv 2407.18074
18. Multi-Agent Systems as Principal-Agent Problems. arxiv 2601.23211
19. Goal Drift evaluation. arxiv 2505.02709 (AAAI/ACM AIES 2025)
20. Anthropic alignment faking research. December 2024
