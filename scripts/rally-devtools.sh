#!/usr/bin/env bash
# 用 rally run 持续推进 devtools goal（docs/goal/devtools.md）。
#
# 用途：长期无人值守推进 DevTools 调试面目标（复用 Chrome DevTools frontend，四面板
# 演示流验收）。注意启动门控：cdp-protocol M3 前仅推进 M0 自主面（bundle/serve/判据）。
#
# 用法：
#   bash scripts/rally-devtools.sh                # 推进 devtools goal
#   bash scripts/rally-devtools.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/devtools.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
