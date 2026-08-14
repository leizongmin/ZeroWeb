#!/usr/bin/env bash
# 测试 flakiness 检查（调研 P4/A3；Ladybird check-test-flakiness.py 简化版）
#
# 对指定 crate 的测试（可选 filter）重跑 RUNS 轮，任一轮失败即判 flaky
# （退出 1）——门禁语义：CI 上不稳定的测试必须先修稳定，不能带 flaky 合入。
#
# 用法：
#   bash scripts/check-test-flakiness.sh <crate> [test-filter]
#   RUNS=5 bash scripts/check-test-flakiness.sh zero-wpt-runner   # 自定义轮数
#
# 说明：
# - 经 test-guard 包裹（OOM 防护，run-rules.md 入口约定）
# - 默认 1 线程（--test-threads=1）保证每轮结果可复现
# - base ref 对比（Ladybird 原版按 base 重跑）暂未实现——同 checkout
#   多轮重跑已能捕获「偶发失败」类 flaky；跨 commit 对比留待需要时加

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RUNS="${RUNS:-3}"

CRATE="${1:-}"
FILTER="${2:-}"
if [[ -z "$CRATE" ]]; then
  echo "Usage: $0 <crate> [test-filter]"
  echo "  RUNS=5 $0 zero-wpt-runner   # 自定义轮数（默认 3）"
  exit 1
fi

# test-guard 就绪（每次强制重编译：CI cargo 缓存可能恢复旧二进制——
# 2026-08-15 smoke 曾因缓存的旧 test-guard 缺 --compile-first 参数而误判 FLAKY）
mkdir -p "${REPO_ROOT}/target"
rustc -O "${REPO_ROOT}/scripts/test-guard.rs" -o "${REPO_ROOT}/target/test-guard"

echo "Flakiness check: cargo test -p ${CRATE} ${FILTER:-（全部）} × ${RUNS} 轮（test-threads=1）"
echo "════════════════════════════════════════════════"

for i in $(seq 1 "$RUNS"); do
  echo "── 第 ${i}/${RUNS} 轮 ──"
  if ! (cd "${REPO_ROOT}" && "${REPO_ROOT}/target/test-guard" --compile-first -- cargo test -p "$CRATE" $FILTER -- --test-threads=1); then
    echo ""
    echo "✗ FLAKY DETECTED：第 ${i} 轮失败（共 ${RUNS} 轮）——请先修复该测试的稳定性再合入"
    exit 1
  fi
done

echo ""
echo "✓ ${RUNS} 轮全部通过（无 flaky）"
