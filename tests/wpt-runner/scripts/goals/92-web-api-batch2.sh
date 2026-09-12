#!/usr/bin/env bash
# Goal: web-api-batch2（编号 92——推进顺序见同目录 README.md）
# 已立项 goal（M12 余面）
# M1 资产预置：fetch clipboard-apis fullscreen 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/web-api-batch2.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/92-web-api-batch2.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="web-api-batch2"
DIRS=( "clipboard-apis" "fullscreen"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
