# ARC-AGI-3 Benchmark: Ritalin vs Baseline

**Date:** 2026-04-12
**Game:** vc33 (click-based, 7 levels)
**Model:** Claude Opus 4.6 (both agents)
**Benchmark:** ARC-AGI-3 — interactive reasoning, no instructions, agent must explore and learn

---

## Results

| Metric | Baseline (no ritalin) | Ritalin Agent | Delta |
|---|---|---|---|
| **Levels completed** | 1/7 | 3/7 | **+200%** |
| **Total actions** | 3 | 83 | +80 |
| **Strategy iterations** | 1 (BFS approach) | 7 revisions (v7) | +6 |
| **Strategy size** | 303 lines | 189 lines | -38% |
| **Approach** | Single-shot algorithmic (BFS) | Iterative: explore, hardcode, generalize | - |

### Key Finding: 3x more levels with structured reasoning

The ritalin agent completed **3x more levels** than the baseline on the same game using the same model. The ritalin agent used more actions (83 vs 3) but solved harder levels that the baseline couldn't reach.

---

## How Each Agent Approached the Problem

### Baseline Agent (no ritalin)

- Read the game source code to understand mechanics
- Built a 303-line strategy using BFS (breadth-first search) state space exploration
- Modeled the game formally: track sprites, map button-to-pipe relationships, search for optimal click sequences
- Solved level 1 in 3 clicks (very efficient)
- Got stuck — the BFS approach didn't scale to harder levels with larger state spaces
- **One-shot approach**: built a general solver, ran it once

### Ritalin Agent (with ritalin)

- Initialized a ritalin contract with outcome and 4 typed obligations
- Systematically explored the game mechanics by reading source code
- Wrote a strategy, ran it, observed what worked and what didn't
- Iterated 7 times (v1 through v7), hardcoding what worked per level
- Level 1: solved (5 actions), Level 2: solved (19 actions), Level 3: solved
- Hit GAME_OVER on level 4 — game mechanics change across levels
- **Iterative approach**: explore, learn, refine, repeat

---

## The ADHD Finding

The most interesting finding isn't the score — it's the **executive function failure pattern** the ritalin agent exhibited.

The ritalin agent:
1. **Started strong** — initialized contract, added obligations, structured its approach
2. **Got hyperfocused** on the interesting part (playing the game, iterating strategies)
3. **Forgot the boring-but-important part** — never wrote exploration docs, never ran `ritalin prove`, never recorded evidence

This is the classic ADHD pattern: strong initiation, hyperfocus on the stimulating task, dropped follow-through on administrative obligations.

**Evidence:**
- `vc33-exploration.md` — obligation O-001 requires this file. **Never created.**
- `vc33-log.md` — obligations O-002/O-003 require `Hypothesis:` and `Evidence:` sections. **Never created.**
- `ritalin prove` — never ran on any obligation. **Zero evidence recorded.**
- `.task-incomplete` — still present. **Gate would block completion claim.**

The contract enforcement is working as designed — `.task-incomplete` correctly prevents the agent from claiming "done" without evidence. But the agent needs a stronger mechanism to remind it to prove obligations *during* work, not just at the end.

### Implication for ritalin v0.2

This finding points directly at the need for a **cadence governor** (`ritalin orient`) — a periodic checkpoint that re-anchors the agent to its obligations. The current system is gate-only (checks at stop time). Adding periodic re-orientation would catch the "hyperfocus drift" pattern earlier.

---

## Methodology

### Setup
- Both agents are Claude Code subagents (Opus 4.6)
- Both received equivalent prompts describing the task
- Both had access to the `arc-agi` Python toolkit and the game source code
- The baseline agent had no ritalin skill installed
- The ritalin agent had the ritalin skill and was instructed to use it

### Game: vc33
- Type: click-based interactive puzzle
- 7 levels of increasing difficulty
- Mechanics: track-switching puzzle with markers (HQB), targets (fZK), buttons (ZGd), and pipes (rDn)
- Clicking buttons moves markers along pipes toward targets
- Human baseline: ~6 actions for level 1

### Limitations
- Single game (vc33) — not yet tested across multiple games
- Single run per agent — no statistical significance
- The agents had access to game source code (not just the visual grid)
- The ritalin agent's prompt explicitly told it to use ritalin; a real-world test would rely on the skill triggering naturally
- Codex (GPT-5.4) comparison not yet run

---

## Next Steps

1. **Run Codex (GPT-5.4 xhigh)** baseline and ritalin variants for 4-way comparison
2. **Test on more games** — expand to 6 games across click, keyboard, and mixed types
3. **Add cadence governor** — `ritalin orient` to catch hyperfocus drift mid-task
4. **Test without source code access** — force agents to explore purely through interaction
5. **Multiple runs** — statistical significance requires 3+ runs per configuration

---

## Raw Data

- Baseline strategy: `bench/arc-agi-3/results/baseline/vc33-strategy.py`
- Baseline score: `bench/arc-agi-3/results/baseline/vc33-score.json`
- Ritalin strategy: `bench/arc-agi-3/results/ritalin/vc33-strategy.py`
- Ritalin score: `bench/arc-agi-3/results/ritalin/vc33-score.json`
- Ritalin contract: `.ritalin/scope.yaml`, `.ritalin/obligations.jsonl`
