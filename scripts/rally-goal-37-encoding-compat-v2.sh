#!/usr/bin/env bash
# 用 rally run 持续推进 encoding-compat-v2 goal（docs/goal/encoding-compat-v2.md）。
#
# 用途：encoding 域二期——legacy-mb 三目录未导入族 + 根面 labels/API 残余 +
# streams 残余入账（一期 30/31/36 已收口：decode 100%P、form-encode 99.9999%，
# 零回归门在册）。coverage-2026-10-30 对账：encoding 缺口 573,911 全 WPT 最大。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/37-encoding-compat-v2.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-37-encoding-compat-v2.sh                # 推进 encoding-compat-v2 goal
#   bash scripts/rally-goal-37-encoding-compat-v2.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/encoding-compat-v2.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
