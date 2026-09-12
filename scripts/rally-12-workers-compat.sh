#!/usr/bin/env bash
# 用 rally run 持续推进 workers-compat goal（docs/goal/workers-compat.md）。
#
# 用途：长期无人值守推进——M2 前先与 zero-page-runtime 对契约（未对齐会 BLOCK 上报）。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/70-workers-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-12-workers-compat.sh                # 推进 workers-compat goal
#   bash scripts/rally-12-workers-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/workers-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
