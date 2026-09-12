#!/usr/bin/env bash
# 用 rally run 持续推进 timing-animation-compat goal（docs/goal/timing-animation-compat.md）。
#
# 用途：长期无人值守推进——轻量热身切片（hr-time/performance-timeline/user-timing/WAAPI）；无门控。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-6-timing-animation-compat.sh                # 推进 timing-animation-compat goal
#   bash scripts/rally-6-timing-animation-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/timing-animation-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
