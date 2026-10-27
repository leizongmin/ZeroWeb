#!/usr/bin/env bash
# 用 rally run 持续推进 editing-compat goal（docs/goal/editing-compat.md）。
#
# 用途：execCommand 编辑命令族长线推进——先语料与基线分类（M2 能力现状图），
# 再按簇收敛；跨界失败簇按 run-rules §9 与 selection/dom-ranges 协调归口。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/33-editing-compat.sh
#   （编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-33-editing-compat.sh                # 推进 editing-compat goal
#   bash scripts/rally-goal-33-editing-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/editing-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
