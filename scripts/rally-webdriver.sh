#!/usr/bin/env bash
# 用 rally run 持续推进 webdriver goal（docs/goal/webdriver.md）。
#
# 用途：长期无人值守推进 WebDriver 服务完善目标（W3C endpoint 补齐 + CI 接线为
# 自动化验证基建），agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-webdriver.sh                # 推进 webdriver goal
#   bash scripts/rally-webdriver.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webdriver.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
