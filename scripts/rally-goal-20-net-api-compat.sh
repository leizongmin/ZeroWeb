#!/usr/bin/env bash
# 用 rally run 持续推进 net-api-compat goal（docs/goal/net-api-compat.md，二期）。
#
# 用途：一期（2026-10-04 收口归档）零回归底线上的二期——url/mimesniff/fetch
# 三族已覆盖失败簇归因收敛（~2,319 失败子测试）+ WebSocket 二期挂账重估。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/20-net-api-compat.sh
#   （一期通道沿用；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-20-net-api-compat.sh                # 推进 net-api-compat goal
#   bash scripts/rally-goal-20-net-api-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/net-api-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
