#!/usr/bin/env bash
# 用 rally run 持续推进 css-testharness-compat-v2 goal（docs/goal/css-testharness-compat-v2.md）。
#
# 用途：css 二期——testharness 面全目录扩账（一期 35 号只入十六目录，
# 缺口 177,800 全 WPT 最大可做池）+ 一期失败簇承接（anchor-size() 参数序
# ~2,500、grid 探测簇 3,280、color 计算值面 7,199 归口）。
# 渲染面 reftest 归 rendering-compat；layout 引擎大簇挂账归口不就地修。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-41-css-testharness-compat-v2.sh                # 推进 css-testharness-compat-v2 goal
#   bash scripts/rally-goal-41-css-testharness-compat-v2.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/css-testharness-compat-v2.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
