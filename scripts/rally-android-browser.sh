#!/usr/bin/env bash
# 用 rally run 持续推进 android-browser goal（docs/goal/android-browser.md）。
#
# 用途：长期无人值守推进 Android 浏览器可用化目标（CI 构建门禁 + 回归保护 + 冒烟验收；
# 功能中期 M2 级、治理缺位），agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-android-browser.sh                # 推进 android-browser goal
#   bash scripts/rally-android-browser.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/android-browser.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
