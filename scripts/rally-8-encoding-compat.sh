#!/usr/bin/env bash
# 用 rally run 持续推进 encoding-compat goal（docs/goal/encoding-compat.md）。
#
# 用途：长期无人值守推进——小快赢（编码标签表 + legacy 解码）；无门控。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/30-encoding-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-8-encoding-compat.sh                # 推进 encoding-compat goal
#   bash scripts/rally-8-encoding-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/encoding-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
