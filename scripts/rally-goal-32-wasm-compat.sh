#!/usr/bin/env bash
# 用 rally run 持续推进 wasm-compat goal（docs/goal/wasm-compat.md）。
#
# 用途：wasm/core + webapi 语料扩面与 proposal 缺口归簇；jsapi 75.7% 基线
# 是回归底线（只增不降）。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/32-wasm-compat.sh
#   （编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-32-wasm-compat.sh                # 推进 wasm-compat goal
#   bash scripts/rally-goal-32-wasm-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/wasm-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
