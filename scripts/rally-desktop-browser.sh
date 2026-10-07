#!/usr/bin/env bash
# 用 rally run 持续推进 desktop-browser goal（docs/goal/desktop-browser.md）。
#
# 用途：长期无人值守推进桌面浏览器产品化目标（真窗口主链路 + 导航/标签 + 内容工具 +
# 数据面，逐功能演示流验收）。注意与 android-browser / devtools 的 apps/browser 碰撞边界。
#
# 用法：
#   bash scripts/rally-desktop-browser.sh                # 推进 desktop-browser goal
#   bash scripts/rally-desktop-browser.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/desktop-browser.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
