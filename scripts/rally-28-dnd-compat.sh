#!/usr/bin/env bash
# 用 rally run 持续推进 dnd-compat goal（docs/goal/dnd-compat.md）。
#
# 用途：长期无人值守推进。**门控 goal**——M1 盘点 + M2 构造器/DataTransfer 面自主；拖放交互管线须用户点名批准。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/28-dnd-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/28-dnd-compat.sh                # 推进 dnd-compat goal
#   bash scripts/28-dnd-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/dnd-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
