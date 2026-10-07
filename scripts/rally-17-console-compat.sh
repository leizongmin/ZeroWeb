#!/usr/bin/env bash
# 用 rally run 持续推进 console-compat goal（docs/goal/console-compat.md）。
#
# 用途：长期无人值守推进——M1 盘点/基线与 M2 console 语义收敛自主推进；
# console 捕获通道（runner 侧）在 M1 甄别后如需 engine/page-runtime 面改动，
# 先与 devtools 流按 run-rules §9 协调工作面。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/21-console-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-17-console-compat.sh                # 推进 console-compat goal
#   bash scripts/rally-17-console-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/console-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
