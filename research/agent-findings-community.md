# Agent Findings: Claude Code Community Tricks

## Top 10 Highest-Impact Techniques

1. **Stop Hook + Test Gate** - Forces verified completion (hooks)
2. **CLAUDE.md Self-Improvement Loop** - Accumulates institutional knowledge
3. **Disable Adaptive Thinking env var** - Fixes Feb-Mar quality regression
4. **Ralph Wiggum dual-exit loop** - Autonomous long-running tasks
5. **Bug Log in CLAUDE.md** - Prevents repeated mistakes
6. **Progressive Disclosure (agent_docs/)** - Saves ~15k tokens
7. **Builder/Validator agent split** - Independent review catches bias
8. **Todo list recency injection** - Keeps goals in high-attention zone
9. **"Correctness over brevity" override** - Counteracts simplicity bias
10. **Exclusive file ownership** - Prevents agent conflicts

## Critical Environment Variables
- CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1 - Fixed thinking budgets
- CLAUDE_CODE_EFFORT_LEVEL=high - Override effort drop
- showThinkingSummaries: true - Visible reasoning

## CLAUDE.md Rules
- Keep under 100 lines / 2500 tokens
- Every line must solve an observed problem
- Maintain Bug Log section
- Self-improvement: add mistakes as they happen
- Root CLAUDE.md < 60 lines, specifics in agent_docs/
- "Before handing back, verify: lint, tests, no errors, renders"

## Ralph Wiggum
- Dual-condition exit: >=2 heuristic detections + EXIT_SIGNAL:true
- Circuit breakers: exits after 3 no-progress or 5 identical-error loops
- Real metrics: 3-month loop built complete programming language
- YC teams: 6+ repos overnight for $297

## Novel Patterns
- Marker file pattern: .task-incomplete blocks stop hook
- Permission auto-allow for read-only operations
- PostToolUse auto-research verification loop
- Three-tier multi-agent framework (subagent/worktree/cloud)
- TaskCompleted hook fires when task marked complete
