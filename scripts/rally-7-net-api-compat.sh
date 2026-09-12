#!/usr/bin/env bash
# 用 rally run 持续推进 net-api-compat goal（docs/goal/net-api-compat.md）。
#
# 用途：长期无人值守推进——主攻切片（fetch/XHR/URL/mimesniff/streams/EventSource）；WebSocket 二期挂账。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/20-net-api-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-7-net-api-compat.sh                # 推进 net-api-compat goal
#   bash scripts/rally-7-net-api-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/net-api-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
