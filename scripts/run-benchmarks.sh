#!/usr/bin/env bash
# run-benchmarks.sh — run benchmarks for all crates with [[bench]] entries
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"

mkdir -p "$RESULTS_DIR"

DATE=$(date +%Y%m%d_%H%M%S)
REPORT="$RESULTS_DIR/benchmark_${DATE}.txt"

# All crates with [[bench]] entries and their bench file names
declare -A BENCH_MAP=(
    [zero-css-parser]="css_bench"
    [zero-dom]="dom_bench"
    [zero-style-system]="style_bench"
    [zero-layout-engine]="layout_bench"
    [zero-engine]="engine_bench"
    [zero-canvas]="canvas_bench"
    [zero-render-foundation]="render_bench"
    [zero-host-runtime]="host_runtime_bench"
    [zero-webview]="webview_bench"
    [zero-net]="net_bench"
    [zero-protocol]="protocol_bench"
    [zero-security]="security_bench"
    [zero-storage]="storage_bench"
    [zero-wasm-sandbox]="wasm_bench"
    [zero-browser-shell]="browser_shell_bench"
    [zero-script-sandbox]="script_sandbox_bench"
)

echo "=== ZeroWeb Benchmarks ===" | tee "$REPORT"
echo "Date: $(date)" | tee -a "$REPORT"
echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" | tee -a "$REPORT"
echo "Crates: ${#BENCH_MAP[@]}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

QUICK_MODE=0
if [ "${ZERO_WEB_BENCH_QUICK:-}" = "1" ]; then
    QUICK_MODE=1
    echo "Mode: quick compile check" | tee -a "$REPORT"
    echo "" | tee -a "$REPORT"
fi

PASSED=()
FAILED=()

for crate in "${!BENCH_MAP[@]}"; do
    bench_name="${BENCH_MAP[$crate]}"
    echo "--- $crate ($bench_name) ---" | tee -a "$REPORT"
    if [ "$QUICK_MODE" = "1" ]; then
        bench_cmd=(cargo bench -p "$crate" --bench "$bench_name" --no-run)
    else
        bench_cmd=(cargo bench -p "$crate" --bench "$bench_name")
    fi

    if "${bench_cmd[@]}" 2>&1 | grep -E "^(Benchmarking|$crate|time:|Found|change:|    Finished)" | tee -a "$REPORT"; then
        PASSED+=("$crate")
    else
        FAILED+=("$crate")
        echo "[WARN] $crate benchmarks failed" | tee -a "$REPORT"
    fi
    echo "" | tee -a "$REPORT"
done

echo "=== Summary ===" | tee -a "$REPORT"
echo "Passed: ${#PASSED[@]} / ${#BENCH_MAP[@]}" | tee -a "$REPORT"
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed: ${FAILED[*]}" | tee -a "$REPORT"
fi
echo "Report saved to: $REPORT" | tee -a "$REPORT"
