#!/usr/bin/env bash
# 用 rally run 持续推进 net-api-compat-v3 goal（docs/goal/net-api-compat-v3.md）。
#
# 用途：网络面三期——fetch 族未导入目录（http-cache/nosniff/range/
# multipart 等，缺口 11,380）+ websockets 全量（4.27%，P3 scheme 翻转
# 重估）+ xhr 残余（23.75%）。一期/二期通道零回归底线在册
# （v2 终账 11,040 + S 系列增量）；url/mimesniff 不重开。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/43-net-api-compat-v3.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-43-net-api-compat-v3.sh                # 推进 net-api-compat-v3 goal
#   bash scripts/rally-goal-43-net-api-compat-v3.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/net-api-compat-v3.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
