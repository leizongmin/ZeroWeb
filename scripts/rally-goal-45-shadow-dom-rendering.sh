#!/usr/bin/env bash
# 用 rally run 持续推进 shadow-dom-rendering goal（docs/goal/shadow-dom-rendering.md）。
#
# 用途：Shadow DOM 渲染级专项（用户 2026-11-04 点名立项）——shadow 树进
# 样式/布局/绘制 + :host/::slotted 从「恒不匹配」翻真命中。JS 面 slot 语义
# 已收口（web-components DC-4），本 goal 只动 Rust 渲染管线；JS 语义面归
# 42 号 shadow-dom-expansion，reftest 全量账归 rendering-compat 零回归门。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/45-shadow-dom-rendering.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-45-shadow-dom-rendering.sh                # 推进 shadow-dom-rendering goal
#   bash scripts/rally-goal-45-shadow-dom-rendering.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/shadow-dom-rendering.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
