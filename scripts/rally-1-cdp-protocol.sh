#!/usr/bin/env bash
# 用 rally run 持续推进 cdp-protocol goal（docs/goal/cdp-protocol.md）。
#
# 用途：长期无人值守推进 CDP 协议兼容目标（Playwright connectOverCDP 全核心流 +
# 命令矩阵账本 + E2E 全绿收口）。
#
# 用法：
#   bash scripts/rally-1-cdp-protocol.sh                # 推进 cdp-protocol goal
#   bash scripts/rally-1-cdp-protocol.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/cdp-protocol.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
