#!/usr/bin/env bash
# run-benchmarks.sh — 兼容薄 wrapper（性能门禁体系落地后，测量统一由 bench-report.sh 负责）。
#
# 历史契约保留：
#   - 全量测量 → tests/benchmarks/results/benchmark_${DATE}.json + .txt
#   - ZERO_WEB_BENCH_QUICK=1 → --no-run 编译检查（PR CI 用）
# 变更：任一 crate 基准执行失败 → 非零退出（旧实现总是 exit 0，修复之）。
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/bench-report.sh" "$@"
