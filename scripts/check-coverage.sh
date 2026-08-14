#!/usr/bin/env bash
# check-coverage.sh — 一键测量并输出覆盖率摘要
# 默认跑 workspace 全量摘要（快）。加 --dom-bindings 额外输出 dom_bindings 子模块独立口径
#（js-dom goal M0 项 4：dom_bindings 是 zero-engine 子模块，--summary-only 会 fold 进 zero-engine
# 无独立数字；dom_bindings 口径经 scripts/check-dom-bindings-coverage.sh 单独测量）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUN_DOM_BINDINGS=0
for arg in "$@"; do
  case "$arg" in
    --dom-bindings) RUN_DOM_BINDINGS=1;;
  esac
done

echo "=== ZeroWeb 覆盖率检查 ==="
echo "日期: $(date)"
echo ""

cd "$PROJECT_ROOT"

# 检查工具是否可用
if command -v cargo-llvm-cov &>/dev/null; then
    echo "--- 使用 cargo-llvm-cov ---"
    # 使用 --test-threads=1 避免 render-foundation GPU 测试在并行时 SIGSEGV
    cargo llvm-cov --workspace --summary-only -- --test-threads=1 2>&1 || {
        echo "警告: 覆盖率测量失败，可能需要安装 cargo-llvm-cov"
        echo "安装: cargo install cargo-llvm-cov"
        exit 1
    }
elif command -v cargo-tarpaulin &>/dev/null; then
    echo "--- 使用 cargo-tarpaulin ---"
    cargo tarpaulin --workspace --skip-clean 2>&1 || {
        echo "警告: 覆盖率测量失败"
        exit 1
    }
else
    echo "错误: 未找到覆盖率工具"
    echo "请安装以下工具之一:"
    echo "  cargo install cargo-llvm-cov  (推荐)"
    echo "  cargo install cargo-tarpaulin"
    exit 1
fi

echo ""
echo "=== 覆盖率检查完成 ==="

# dom_bindings 子模块独立口径（js-dom goal M0 项 4）。经 --dom-bindings 开启（cov run 额外 ~15s）。
if [ "$RUN_DOM_BINDINGS" -eq 1 ]; then
    echo ""
    bash "$SCRIPT_DIR/check-dom-bindings-coverage.sh" || true
fi
