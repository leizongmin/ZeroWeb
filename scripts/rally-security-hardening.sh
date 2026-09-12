#!/usr/bin/env bash
# 用 rally run 持续推进 security-hardening goal（docs/goal/security-hardening.md）。
#
# 用途：长期无人值守推进安全加固目标（CSP 完整实现 + Mixed Content + HSTS + 权限模型，
# WPT 三 corpus 验收）。行为变更走 kill-switch + A/B 门禁。
#
# 用法：
#   bash scripts/rally-security-hardening.sh                # 推进 security-hardening goal
#   bash scripts/rally-security-hardening.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/security-hardening.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
