# ritalin Benchmark Results

**Date:** 2026-04-11  
**Binary:** ritalin v0.1.1 (release, LTO + strip)  
**Platform:** macOS Darwin 25.2.0, Apple Silicon  
**Tool:** [hyperfine](https://github.com/sharkdp/hyperfine) (industry-standard CLI benchmarking)

---

## Gate Latency — The Critical Path

`ritalin gate --hook-mode` fires on **every** Claude Code Stop event. This number determines whether the agent experiences any perceptible delay.

| Benchmark | Mean | σ | Min | Max | Runs |
|-----------|------|---|-----|-----|------|
| Gate hook-mode (1 obligation, proved) | **3.2 ms** | 0.7 ms | 2.3 ms | 6.5 ms | 517 |
| Gate hook-mode (uninitialised, no-op) | **3.6 ms** | 0.6 ms | 2.7 ms | 7.8 ms | 497 |

**Verdict:** Sub-5ms. Imperceptible to the agent. Zero tax on the completion loop.

## Gate Scaling

How does gate perform as obligation count grows?

| Obligations | Mean | σ | Min | Max |
|-------------|------|---|-----|-----|
| 10 | 2.8 ms | 0.9 ms | 2.0 ms | 15.6 ms |
| 100 | 3.0 ms | 0.8 ms | 2.1 ms | 13.2 ms |
| 1,000 | 4.0 ms | 0.7 ms | 3.0 ms | 10.0 ms |

**Verdict:** Sub-linear scaling. Even with 1,000 obligations (far beyond any realistic task), gate stays under 5ms.

## Full Workflow

Complete `init → add → prove → gate` cycle:

| Benchmark | Mean | σ | Min | Max |
|-----------|------|---|-----|-----|
| Full workflow (4 commands) | **15.8 ms** | 1.1 ms | 13.9 ms | 20.7 ms |

**Verdict:** ~4ms per command average. The entire ritalin ceremony completes faster than a single Python import.

## Rust vs Python

Same gate logic, same JSONL files, same workspace hash — Rust binary vs equivalent Python script:

| Implementation | Mean | Speedup |
|----------------|------|---------|
| **ritalin (Rust)** | **3.7 ms** | **1.0x** (baseline) |
| Python equivalent | 19.1 ms | 0.19x |

**ritalin is 5.2x faster than a Python equivalent.** The gap widens under load because Python's startup overhead (~15ms) is fixed cost on every invocation.

## Binary Statistics

| Metric | Value |
|--------|-------|
| Binary size | **4.7 MB** |
| Peak RSS (gate) | **6.8 MB** |
| Cold start | < 3 ms |
| Dependencies | 0 (single static binary) |

---

## Methodology

- All benchmarks use `hyperfine` with warmup runs to eliminate cold-cache effects
- Gate benchmarks use `--min-runs 200` for statistical significance
- Each benchmark runs in an isolated tmpdir
- Python comparison uses identical file formats and hash algorithms
- Full JSON results available in `bench/*.json`

## Reproducing

```bash
cargo install --path .
bash bench/run_benchmarks.sh
```

Requires: `hyperfine` (`brew install hyperfine`), Python 3.
