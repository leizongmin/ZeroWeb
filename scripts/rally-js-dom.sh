#!/usr/bin/env bash
# 用 rally run 持续推进 js-dom goal（docs/goal/js-dom.md）。
#
# 用途：长期无人值守推进 JS/DOM 原生化目标（P1b V8 原生绑定生产路径收口——
# polyfill 字符串桥 → native 绑定、default-on + 删 kill-switch、真实 SPA/WC
# 端到端验收、WPT dom 上游通过率基线），agent-command 用 claude-glm
#（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-js-dom.sh                # 推进 js-dom goal
#   bash scripts/rally-js-dom.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/js-dom.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
