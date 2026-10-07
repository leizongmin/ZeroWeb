#!/usr/bin/env bash
# 用 rally run 持续推进 webmessaging-compat goal（docs/goal/webmessaging-compat.md）。
#
# 用途：长期无人值守推进——M1 盘点/基线与 M2 语义收敛自主推进；甄别挂账项见
# 契约文档。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/17-webmessaging-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/17-webmessaging-compat.sh                # 推进 webmessaging-compat goal
#   bash scripts/17-webmessaging-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webmessaging-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
