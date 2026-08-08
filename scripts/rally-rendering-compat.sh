#!/usr/bin/env bash
# 用 rally run 持续推进 rendering-compat goal（docs/goal/rendering-compat.md）。
#
# 用途：长期无人值守推进渲染兼容性目标（WPT reftest 驱动的渲染正确性），
# agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-rendering-compat.sh         # 推进 rendering-compat goal
#   bash scripts/rally-rendering-compat.sh --dry-run
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/rendering-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
