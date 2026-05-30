#!/usr/bin/env bash
# run-benchmarks.sh — run benchmarks for all crates with [[bench]] entries
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"

mkdir -p "$RESULTS_DIR"

DATE=$(date +%Y%m%d_%H%M%S)
REPORT="$RESULTS_DIR/benchmark_${DATE}.txt"

# All crates with [[bench]] entries (order: pipeline stages first)
BENCH_CRATES=(
    zero-css-parser
    zero-dom
    zero-style-system
    zero-layout-engine
    zero-engine-core
    zero-canvas
    zero-render-foundation
    zero-host-runtime
    zero-webview-api
    zero-net
    zero-protocol
    zero-security
    zero-storage
    zero-wasm-sandbox
)

echo "=== ZeroBrowser Benchmarks ===" | tee "$REPORT"
echo "Date: $(date)" | tee -a "$REPORT"
echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" | tee -a "$REPORT"
echo "Crates: ${#BENCH_CRATES[@]}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

PASSED=()
FAILED=()

for crate in "${BENCH_CRATES[@]}"; do
    echo "--- $crate ---" | tee -a "$REPORT"
    if cargo bench -p "$crate" 2>&1 | tee -a "$REPORT"; then
        PASSED+=("$crate")
    else
        FAILED+=("$crate")
        echo "[WARN] $crate benchmarks failed" | tee -a "$REPORT"
    fi
    echo "" | tee -a "$REPORT"
done

echo "=== Summary ===" | tee -a "$REPORT"
echo "Passed: ${#PASSED[@]} / ${#BENCH_CRATES[@]}" | tee -a "$REPORT"
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed: ${FAILED[*]}" | tee -a "$REPORT"
fi
echo "Report saved to: $REPORT" | tee -a "$REPORT"
