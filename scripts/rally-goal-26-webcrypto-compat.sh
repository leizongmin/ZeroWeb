#!/usr/bin/env bash
# 用 rally run 持续推进 webcrypto-compat goal（docs/goal/webcrypto-compat.md，二期）。
#
# 用途：一期（2026-10-20 归档，corpus 77.89%）零回归底线上的二期——扩面
# derive_bits_keys 等未拉语料 + `.tentative.` 挂账域转正甄别（ML-KEM/ML-DSA
# 等对 NIST FIPS 定稿状态），不是修引擎失败（失败面已全部挂账）。
# 语料脚本：tests/wpt-runner/scripts/goals/26-webcrypto-compat.sh（一期通道，
# FORCE=1 续拉补全）。
#
# 用法：
#   bash scripts/rally-goal-26-webcrypto-compat.sh                # 推进 webcrypto-compat goal
#   bash scripts/rally-goal-26-webcrypto-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webcrypto-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
