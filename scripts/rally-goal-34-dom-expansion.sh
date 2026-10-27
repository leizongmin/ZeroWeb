#!/usr/bin/env bash
# 用 rally run 持续推进 dom-expansion goal（docs/goal/dom-expansion.md）。
#
# 用途：dom/ranges + html/dom 语料扩面与接口完备性收敛；40 号已收口面
# （parser 语义通道）零回归是底线。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/34-dom-expansion.sh
#   （编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-34-dom-expansion.sh                # 推进 dom-expansion goal
#   bash scripts/rally-goal-34-dom-expansion.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/dom-expansion.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
