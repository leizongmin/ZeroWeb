#!/usr/bin/env bash
# 用 rally run 持续推进 event-loop-spec goal（docs/goal/event-loop-spec.md）。
#
# 用途：长期无人值守推进事件循环与异步回调 spec 化目标（IO/RO WPT 基线 + host 侧
# MutationObserver 方案 C + microtask checkpoint spec 化），agent-command 用
# claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-event-loop-spec.sh                # 推进 event-loop-spec goal
#   bash scripts/rally-event-loop-spec.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/event-loop-spec.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
