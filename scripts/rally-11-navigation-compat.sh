#!/usr/bin/env bash
# 用 rally run 持续推进 navigation-compat goal（docs/goal/navigation-compat.md）。
#
# 用途：长期无人值守推进——M2 history 轻面自主推进；M3 iframe 深结构切片须用户点名批准。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/60-navigation-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-11-navigation-compat.sh                # 推进 navigation-compat goal
#   bash scripts/rally-11-navigation-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/navigation-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
