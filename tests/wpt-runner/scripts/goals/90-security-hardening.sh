#!/usr/bin/env bash
# Goal: security-hardening（编号 90——推进顺序见同目录 README.md）
# 已立项 goal（策略执行面）
# M1 资产预置：fetch content-security-policy mixed-content secure-contexts 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/security-hardening.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/90-security-hardening.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="security-hardening"
DIRS=( "content-security-policy" "mixed-content" "secure-contexts"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
