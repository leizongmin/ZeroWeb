#!/usr/bin/env bash
# 用 rally run 持续推进 cookies-compat goal（docs/goal/cookies-compat.md）。
#
# 用途：长期无人值守推进——M1 盘点/基线与 M2 document.cookie 语义收敛自主推进；
# HTTP 头驱动用例的可执行性甄别在 M1 内完成（见 goals/18 脚本头注释）。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/18-cookies-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-18-cookies-compat.sh                # 推进 cookies-compat goal
#   bash scripts/rally-18-cookies-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/cookies-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
