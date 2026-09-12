#!/usr/bin/env bash
# Goal: encoding-compat（编号 30——推进顺序见同目录 README.md）
# 小快赢（legacy 编码标签表）
# M1 资产预置：fetch encoding 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/encoding-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/30-encoding-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="encoding-compat"
DIRS=( "encoding"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
