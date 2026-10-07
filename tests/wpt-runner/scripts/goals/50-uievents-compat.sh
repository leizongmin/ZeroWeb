#!/usr/bin/env bash
# Goal: uievents-compat（编号 50——推进顺序见同目录 README.md）
# 合成事件语义先行
# M1 资产预置：fetch uievents pointerevents 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/uievents-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/50-uievents-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="uievents-compat"
DIRS=( "uievents" "pointerevents"  )

goals_fetch_all
# 二级子目录显式追加（lib 递归仅一层 depth<1——fetch-dom-subset SUBDIRS 先例）：
# order-of-events/{mouse-events,focus-events} 是 M2 鼠标事件序的核心语料
#（click-order/mouseover-out/mousemove-between 等），首轮 fetch 因深度限制缺失。
fetch_dir_html "uievents/order-of-events/mouse-events"
fetch_dir_html "uievents/order-of-events/focus-events"
# 尾簇 39（2026-10-07）：子目录资源文件——attributes / cancel-mousedown-in-subframe /
# mousemove_prevent_default_action 三案 import `resources/utils.js`（目录列举只收
# .html，资源文件漏拉 → 三案 page-threw「script fetch failed」，各折 1 条文件级
# Fail）。同 tail-30 textInput support/ 人工补拉先例，此处脚本化（幂等 + pin）。
fetch_raw "uievents/mouse/resources/utils.js"
goals_inventory
goals_next_steps "${GOAL}"
