#!/usr/bin/env bash
# 用 rally run 持续推进 shadow-dom-expansion goal（docs/goal/shadow-dom-expansion.md）。
#
# 用途：shadow-dom 语料滚动扩面（4.05%，缺口 14,729——web-components 收口
# 后上游滚动 + 排除域未重审）+ custom-elements 同通道零回归门。渲染级
# composed tree（:host/::slotted 进样式/布局/绘制）维持收口排除条款，
# 不进本 goal。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/42-shadow-dom-expansion.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-42-shadow-dom-expansion.sh                # 推进 shadow-dom-expansion goal
#   bash scripts/rally-goal-42-shadow-dom-expansion.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/shadow-dom-expansion.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
