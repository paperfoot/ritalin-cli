#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BENCH_DIR="$SCRIPT_DIR"
RITALIN="$(which ritalin)"

echo "=== ritalin benchmark suite ==="
echo "Binary: $RITALIN"
echo "Version: $($RITALIN --version 2>/dev/null | head -1 || echo 'unknown')"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ─── Helper: set up a temp workspace ─────────────────────────
setup_workspace() {
    local dir="$1"
    local n_obligations="${2:-1}"
    cd "$dir"
    $RITALIN init --outcome "benchmark" --force >/dev/null 2>&1
    for i in $(seq 1 "$n_obligations"); do
        $RITALIN add "obligation $i" --proof "true" --kind other >/dev/null 2>&1
    done
}

prove_all() {
    local dir="$1"
    local n_obligations="${2:-1}"
    cd "$dir"
    for i in $(seq 1 "$n_obligations"); do
        $RITALIN prove "O-$(printf '%03d' "$i")" >/dev/null 2>&1
    done
}

# ─── B1: Gate hook-mode latency (the critical path) ─────────
echo ">>> B1: Gate hook-mode latency (1 obligation, all proved)"
WORK1=$(mktemp -d)
setup_workspace "$WORK1" 1
prove_all "$WORK1" 1
cd "$WORK1"
hyperfine --warmup 5 --min-runs 200 \
    'echo "{}" | ritalin gate --hook-mode' \
    --export-json "$BENCH_DIR/gate-hookmode.json" \
    --export-markdown "$BENCH_DIR/gate-hookmode.md"
rm -rf "$WORK1"
echo ""

# ─── B2: Gate no-op (uninitialised repo) ─────────────────────
echo ">>> B2: Gate hook-mode no-op (uninitialised)"
WORK2=$(mktemp -d)
cd "$WORK2"
hyperfine --warmup 5 --min-runs 200 \
    'echo "{}" | ritalin gate --hook-mode' \
    --export-json "$BENCH_DIR/gate-noop.json" \
    --export-markdown "$BENCH_DIR/gate-noop.md"
rm -rf "$WORK2"
echo ""

# ─── B3: Full workflow ───────────────────────────────────────
echo ">>> B3: Full workflow (init + add + prove + gate)"
WORK3=$(mktemp -d)
cd "$WORK3"
hyperfine --warmup 3 --min-runs 50 \
    --prepare 'rm -rf .ritalin .task-incomplete' \
    'ritalin init -o bench && ritalin add "x" --proof true --kind other && ritalin prove O-001 && ritalin gate' \
    --export-json "$BENCH_DIR/full-workflow.json" \
    --export-markdown "$BENCH_DIR/full-workflow.md"
rm -rf "$WORK3"
echo ""

# ─── B4: Gate scaling ────────────────────────────────────────
echo ">>> B4: Gate scaling (10 / 100 / 1000 obligations)"
for N in 10 100 1000; do
    echo "  > $N obligations"
    WORKN=$(mktemp -d)
    setup_workspace "$WORKN" "$N"
    prove_all "$WORKN" "$N"
    cd "$WORKN"
    hyperfine --warmup 3 --min-runs 50 \
        'ritalin gate' \
        --export-json "$BENCH_DIR/gate-scaling-$N.json" \
        --export-markdown "$BENCH_DIR/gate-scaling-$N.md"
    rm -rf "$WORKN"
done
echo ""

# ─── B5: Rust vs Python ─────────────────────────────────────
echo ">>> B5: Rust vs Python gate comparison"
WORK5=$(mktemp -d)
setup_workspace "$WORK5" 10
prove_all "$WORK5" 10
cd "$WORK5"
hyperfine --warmup 3 --min-runs 100 \
    'echo "{}" | ritalin gate --hook-mode' \
    "python3 $BENCH_DIR/python_gate.py" \
    --export-json "$BENCH_DIR/rust-vs-python.json" \
    --export-markdown "$BENCH_DIR/rust-vs-python.md"
rm -rf "$WORK5"
echo ""

# ─── B6: Binary size + memory ────────────────────────────────
echo ">>> B6: Binary size & peak memory"
BINARY_SIZE=$(ls -lh "$RITALIN" | awk '{print $5}')
echo "Binary size: $BINARY_SIZE"
echo "Binary path: $RITALIN"

WORK6=$(mktemp -d)
setup_workspace "$WORK6" 1
prove_all "$WORK6" 1
cd "$WORK6"
PEAK_MEM=$( { command time -l ritalin gate 2>&1 1>/dev/null; } | grep "maximum resident" | awk '{print $1}')
echo "Peak RSS (gate): ${PEAK_MEM} bytes"
rm -rf "$WORK6"

# Write size/memory to file
cat > "$BENCH_DIR/binary-stats.md" <<STATS
## Binary Statistics

| Metric | Value |
|--------|-------|
| Binary size | $BINARY_SIZE |
| Peak RSS (gate) | ${PEAK_MEM} bytes |
| Path | $RITALIN |
STATS

echo ""
echo "=== All benchmarks complete ==="
echo "Results in: $BENCH_DIR/"
