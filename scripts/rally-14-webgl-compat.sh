#!/usr/bin/env bash
# 用 rally run 持续推进 webgl-compat goal（docs/goal/webgl-compat.md）。
#
# 用途：长期无人值守推进——远期门控：仅 M1 语料盘点自主推进，M2+ 每切片须用户点名。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/99-webgl-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-14-webgl-compat.sh                # 推进 webgl-compat goal
#   bash scripts/rally-14-webgl-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webgl-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
