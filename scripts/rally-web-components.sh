#!/usr/bin/env bash
# 用 rally run 持续推进 web-components goal（docs/goal/web-components.md）。
#
# 用途：长期无人值守推进 Web Components 目标（Custom Elements 收口 + template 真实化
# + slot 全链路；Shadow DOM 渲染级排除、等用户点名专项），agent-command 用 claude-glm
#（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-web-components.sh                # 推进 web-components goal
#   bash scripts/rally-web-components.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/web-components.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
