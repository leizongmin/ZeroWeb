#!/usr/bin/env bash
# 用 rally run 持续推进 selection-compat goal（docs/goal/selection-compat.md，二期）。
#
# 用途：一期零回归底线上的二期——selection 语料从 20 文件扩到全量 101 文件 +
# 扩面基线与失败归因（editing 依赖簇归口 33 号）。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/23-selection-compat.sh
#   （一期通道沿用，DIRS 扩全量；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-23-selection-compat.sh                # 推进 selection-compat goal
#   bash scripts/rally-goal-23-selection-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/selection-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
