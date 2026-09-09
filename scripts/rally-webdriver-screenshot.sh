#!/usr/bin/env bash
# 用 rally run 持续推进 webdriver-screenshot goal（docs/goal/webdriver-screenshot.md）。
#
# 用途：长期无人值守推进 WebDriver Screenshot 目标（公共截图转换层 crate + GET
# /session/{id}/screenshot；方案①用户拍板 2026-09-09），agent-command 用 claude-glm
#（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-webdriver-screenshot.sh                # 推进 webdriver-screenshot goal
#   bash scripts/rally-webdriver-screenshot.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webdriver-screenshot.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
