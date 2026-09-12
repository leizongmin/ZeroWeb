#!/usr/bin/env bash
# 用 rally run 持续推进 svg-compat goal（docs/goal/svg-compat.md）。
#
# 用途：长期无人值守推进——SVG 文档模型（渲染面归 rendering-compat）；SMIL 评估挂账。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/80-svg-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-13-svg-compat.sh                # 推进 svg-compat goal
#   bash scripts/rally-13-svg-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/svg-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
