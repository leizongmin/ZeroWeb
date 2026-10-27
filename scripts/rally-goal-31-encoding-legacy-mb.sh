#!/usr/bin/env bash
# 用 rally run 持续推进 encoding-legacy-mb goal（docs/goal/encoding-legacy-mb.md）。
#
# 用途：解锁 legacy-mb 文档解码面（全量分母 50% 所在）——M0 可行性切片是第一关，
# 未过不进入后续里程碑；不可行按契约挂账重估。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/31-encoding-legacy-mb.sh
#   （补全缺口需 FORCE=1；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-31-encoding-legacy-mb.sh                # 推进 encoding-legacy-mb goal
#   bash scripts/rally-goal-31-encoding-legacy-mb.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/encoding-legacy-mb.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
