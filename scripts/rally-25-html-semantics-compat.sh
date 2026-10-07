#!/usr/bin/env bash
# 用 rally run 持续推进 html-semantics-compat goal（docs/goal/html-semantics-compat.md）。
#
# 用途：长期无人值守推进——M1 盘点/基线与 M2 语义收敛自主推进；甄别挂账项见
# 契约文档。大域分域切片推进；渲染差异面归 rendering-compat。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/25-html-semantics-compat.sh                # 推进 html-semantics-compat goal
#   bash scripts/25-html-semantics-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/html-semantics-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
