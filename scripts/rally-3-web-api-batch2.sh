#!/usr/bin/env bash
# 用 rally run 持续推进 web-api-batch2 goal（docs/goal/web-api-batch2.md）。
#
# 用途：长期无人值守推进 Web API 第二批目标（Clipboard + Fullscreen，WPT 两 corpus
# 验收；DnD 挂账不在范围）。
#
# 用法：
#   bash scripts/rally-3-web-api-batch2.sh                # 推进 web-api-batch2 goal
#   bash scripts/rally-3-web-api-batch2.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/web-api-batch2.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
