#!/usr/bin/env bash
# 用 rally run 持续推进 html-syntax-compat goal（docs/goal/html-syntax-compat.md）。
#
# 用途：长期无人值守推进——html-compat（fixture 制）的 WPT 化续篇；无门控。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-9-html-syntax-compat.sh                # 推进 html-syntax-compat goal
#   bash scripts/rally-9-html-syntax-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/html-syntax-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
