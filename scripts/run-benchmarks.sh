#!/usr/bin/env bash
# run-benchmarks.sh — 一键运行所有基准测试并输出报告
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"

mkdir -p "$RESULTS_DIR"

DATE=$(date +%Y%m%d_%H%M%S)
REPORT="$RESULTS_DIR/benchmark_${DATE}.txt"

echo "=== ZeroBrowser 基准测试 ==="
echo "日期: $(date)"
echo "提交: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
echo ""

# 运行 render-foundation 基准
echo "--- render-foundation 基准 ---"
cd "$PROJECT_ROOT"
cargo bench --manifest-path tests/benchmarks/benches/Cargo.toml 2>&1 | tee "$REPORT"

echo ""
echo "=== 基准测试完成 ==="
echo "报告已保存到: $REPORT"
