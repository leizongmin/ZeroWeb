#!/usr/bin/env bash
# 用 rally run 持续推进 css-testharness-compat goal（docs/goal/css-testharness-compat.md）。
#
# 用途：css/* 目录 testharness 面从 0 入账——M0 试点（css-values + selectors
# 通道与基线）→ M1 runner 内存治理（分片执行 + coverage 分片管道）→
# M2 css-variables 解禁 → M3 top15 扩面 → M4 coverage 复测收口。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh
#   （编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-35-css-testharness-compat.sh                # 推进 css-testharness-compat goal
#   bash scripts/rally-goal-35-css-testharness-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/css-testharness-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
