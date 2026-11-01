#!/usr/bin/env bash
# 用 rally run 持续推进 webcrypto-compat-v3 goal（docs/goal/webcrypto-compat-v3.md，
# 三期，2026-11-01 二期 D1 转正域用户拍板立项——modern-algos + secure-curves
# 两族 10 域 109 案/18,052 子测试按域收敛）。
#
# 语料脚本：tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh（两新族
# 新增拉取，rev 锁二期 DC-2 查证的转正 rev；既有 pin 语料零变动）。
#
# 用法：
#   bash scripts/rally-goal-38-webcrypto-compat-v3.sh                # 推进 webcrypto-compat-v3 goal
#   bash scripts/rally-goal-38-webcrypto-compat-v3.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webcrypto-compat-v3.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
