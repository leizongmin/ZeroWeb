#!/usr/bin/env bash
# check-coverage.sh — 一键测量并输出覆盖率摘要
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

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
