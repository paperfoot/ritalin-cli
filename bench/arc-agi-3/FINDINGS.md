# ARC-AGI-3 Benchmark: Ritalin vs Baseline

**Date:** 2026-04-12
**Game:** vc33 (click-based, 7 levels)
**Model:** Claude Opus 4.6 (both agents)
**Benchmark:** ARC-AGI-3 — interactive reasoning, no instructions, agent must explore and learn

---

## Results

| Metric | Baseline (no ritalin) | Ritalin Agent | Winner |
|---|---|---|---|
| **Levels completed** | 3/7 | 3/7 | Tie |
| **In-game actions** | **33** | 83 | Baseline |
| **Clock time** | 66 min | **57 min** | Ritalin |
| **Token cost** | **231K** | 242K | Baseline |
| **Tool uses** | **96** | 130 | Baseline |
| **Strategy iterations** | 1 (BFS approach) | 7 revisions (v7) | - |
| **Documentation produced** | Score + strategy | Exploration notes + hypothesis log + evidence + strategy + score | **Ritalin** |
| **Obligations proved** | N/A | 4/4 (all exit=0) | Ritalin |
| **Blocked on** | L4 zHk teleport | L4 zHk teleport | Same wall |

### Key Finding: Same score, different value

Both agents solved the same 3 levels and hit the same wall (level 4's zHk teleport mechanic). The baseline was more action-efficient (33 vs 83 actions). The ritalin agent was faster clock-time (57 vs 66 min) and produced dramatically better documentation.

**The real difference isn't the score — it's the evidence trail.** The ritalin agent left behind exploration notes, a hypothesis log with evidence sections, and 4 proved obligations. If you need to hand this work off to another session, debug what went wrong on level 4, or build on the findings, the ritalin agent's output is immediately useful. The baseline left a strategy file and a number.

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

**Evidence of the pattern:**
- The agent initialized the contract and added obligations at the start of its session
- It then spent ~50 minutes writing and iterating game strategies (7 versions)
- Only AFTER solving 3 levels did it circle back to create `vc33-exploration.md` and `vc33-log.md`
- It then ran `ritalin prove` on all 4 obligations — all passed (exit=0)
- The obligations eventually got proved, but the documentation was written *after* the work, not *during*

**What the gate enforced:** `.task-incomplete` stayed present throughout, correctly blocking the agent from claiming "done" without evidence. When the agent finally proved all obligations, the gate could pass. The contract worked — it just didn't prevent the mid-task drift.

### Implication for ritalin v0.2

This finding points directly at the need for a **cadence governor** (`ritalin orient`) — a periodic checkpoint that re-anchors the agent to its obligations mid-task, not just at stop time. The current system catches "I'm done without evidence" but not "I'm working without documenting." Adding periodic re-orientation would catch the hyperfocus drift pattern earlier and produce better documentation in real-time.

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
