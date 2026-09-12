#!/usr/bin/env bash
# 用 rally run 持续推进 uievents-compat goal（docs/goal/uievents-compat.md）。
#
# 用途：长期无人值守推进——UI/指针事件语义；touch/pointerlock/IME 挂账。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/50-uievents-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-10-uievents-compat.sh                # 推进 uievents-compat goal
#   bash scripts/rally-10-uievents-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/uievents-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
