#!/usr/bin/env bash
# 用 rally run 持续推进 svg-compat-v2 goal（docs/goal/svg-compat-v2.md）。
#
# 用途：svg 二期——SMIL 时钟模型专项承接（一期 80 号挂账#1：199 子 Fail +
# 29 案 Timeout，分类账为底稿）+ 语料滚动扩面（7.70%，缺口 14,161）+
# 挂账 #2/#3 归口重估。渲染面 reftest 归 rendering-compat；动画泵边界
# （SMIL 文档级时间线 ≠ compositor 动画泵）照一期划界。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/44-svg-compat-v2.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-44-svg-compat-v2.sh                # 推进 svg-compat-v2 goal
#   bash scripts/rally-goal-44-svg-compat-v2.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/svg-compat-v2.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
