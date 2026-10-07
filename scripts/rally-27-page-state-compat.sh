#!/usr/bin/env bash
# 用 rally run 持续推进 page-state-compat goal（docs/goal/page-state-compat.md）。
#
# 用途：长期无人值守推进。M1 盘点/基线自主；sendBeacon 实现与可见性/online 事件面自主推进。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/27-page-state-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/27-page-state-compat.sh                # 推进 page-state-compat goal
#   bash scripts/27-page-state-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/page-state-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
