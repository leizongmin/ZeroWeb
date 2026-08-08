#!/usr/bin/env bash
# 用 rally run 持续推进 zero-web goal（docs/goal/zero-web.md）。
#
# 用途：长期无人值守推进父目标（P1a DOM/JS Bridge 原生化等），
# agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-zero-web.sh                # 推进 zero-web goal
#   bash scripts/rally-zero-web.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/zero-web.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
