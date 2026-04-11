# ritalin v0.1.1 — Benchmarks & Test Suite Design

**Date:** 2026-04-11
**Goal:** Ship publishable benchmark results + comprehensive test coverage for the ritalin CLI.

---

## 1. Benchmark Suite

### Tool: `hyperfine` (industry-standard CLI benchmarking)

Benchmarks run against the installed `ritalin` binary, measuring real-world latency from the user/hook's perspective.

### Benchmarks to run:

#### B1: Gate latency (the critical path)
The `gate --hook-mode` command fires on EVERY Claude Code Stop event. This is the number that matters most.
```
hyperfine --warmup 3 --min-runs 100 \
  'echo "{}" | ritalin gate --hook-mode' \
  --export-json bench/gate-hookmode.json \
  --export-markdown bench/gate-hookmode.md
```
Setup: tmpdir with `ritalin init`, 1 obligation, 1 passing evidence.

#### B2: Gate latency — uninitialised repo (no-op path)
```
hyperfine --warmup 3 --min-runs 100 \
  'echo "{}" | ritalin gate --hook-mode' \
  --export-json bench/gate-noop.json \
  --export-markdown bench/gate-noop.md
```
Setup: tmpdir with NO `.ritalin/` directory.

#### B3: Full workflow latency
```
hyperfine --warmup 3 --min-runs 50 \
  'ritalin init -o test && ritalin add "x" --proof true --kind other && ritalin prove O-001 && ritalin gate' \
  --export-json bench/full-workflow.json \
  --export-markdown bench/full-workflow.md
```
Setup: fresh tmpdir per run (--prepare 'rm -rf .ritalin .task-incomplete')

#### B4: Gate scaling (10 / 100 / 1000 obligations)
How does gate perform as the obligation count grows?
```
hyperfine --warmup 3 --min-runs 50 \
  --parameter-list n 10,100,1000 \
  'ritalin gate' \
  --export-json bench/gate-scaling.json
```
Setup: script generates N obligations with matching evidence.

#### B5: Python baseline comparison
Prove the Rust choice. Write a minimal Python script that reads the same JSONL files and evaluates gate logic. Compare:
```
hyperfine --warmup 3 --min-runs 50 \
  'ritalin gate' \
  'python3 bench/python_gate.py' \
  --export-json bench/rust-vs-python.json \
  --export-markdown bench/rust-vs-python.md
```

#### B6: Binary size & memory
- Binary size: `ls -lh $(which ritalin)` → report in README
- Peak RSS: `command time -l ritalin gate 2>&1 | grep "maximum resident"` (macOS)

### Output
All benchmark results go to `bench/` directory. A `bench/RESULTS.md` summarises findings in a format ready to paste into README.

---

## 2. Test Suite

### Tier 0: Unit tests (in-module `#[cfg(test)]`)

#### evidence.rs tests:
- `is_discharged` returns false on empty slice
- `is_discharged` returns false when all exit codes non-zero
- `is_discharged` returns true when at least one exit code is 0
- `is_discharged` returns true even with mixed pass/fail records

#### obligations.rs tests:
- `next_id` returns "O-001" for empty ledger
- `next_id` increments correctly after appends
- `read_all` handles empty file
- `read_all` skips blank lines
- `find` returns error for unknown id

#### marker.rs tests:
- `create` + `exists` round-trip
- `remove` when file exists
- `remove` when file doesn't exist (no error)
- `marker_path` resolves to parent of state_dir

#### prove.rs `tail` function:
- Input shorter than TAIL_LIMIT returns unchanged
- Input exactly TAIL_LIMIT returns unchanged
- Input longer than TAIL_LIMIT is truncated with "…" prefix

### Tier 1: Integration tests (`tests/` directory, using `assert_cmd`)

#### Happy path:
1. `ritalin init --outcome "test"` → exit 0, `.ritalin/` created, `.task-incomplete` created
2. `ritalin add "must pass" --proof "true" --kind other` → exit 0, obligations.jsonl has 1 line
3. `ritalin prove O-001` → exit 0, evidence.jsonl has 1 line with exit_code 0
4. `ritalin gate` → exit 0, `.task-incomplete` removed

#### Failure path:
1. init + add with `--proof "false"`
2. prove → exit 0 (prove itself succeeds), but evidence has exit_code != 0
3. gate → exit non-zero, `.task-incomplete` still exists

#### Hook mode:
1. `echo '{}' | ritalin gate --hook-mode` with open obligation → stdout contains `{"decision":"block"}`
2. `echo '{"stop_hook_active":true}' | ritalin gate --hook-mode` → empty stdout (allows stop)
3. `echo '{}' | ritalin gate --hook-mode` after all proved → empty stdout

#### Edge cases:
1. `ritalin gate` before `ritalin init` → error
2. `ritalin add` before `ritalin init` → error
3. `ritalin add ""` → error (empty claim)
4. Advisory (non-critical) obligations don't block gate

### Tier 2: JSON contract tests
- All commands produce valid JSON when piped (not a TTY)
- JSON envelope has `status` and `data` fields
- `gate --hook-mode` produces either empty stdout or valid `{"decision":"block","reason":"..."}` JSON

---

## 3. Implementation Order

1. Write Tier 0 unit tests → `cargo test`
2. Write Tier 1 integration tests → `cargo test`
3. Write Tier 2 JSON contract tests → `cargo test`
4. Install hyperfine if needed
5. Write benchmark scripts in `bench/`
6. Run benchmarks, collect results
7. Write `bench/RESULTS.md` with publishable numbers
8. Update README with benchmark section
9. Commit everything, push
