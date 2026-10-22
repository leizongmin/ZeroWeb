#!/usr/bin/env bash
# Goal: webgl-compat（编号 99——推进顺序见同目录 README.md）
# 远期门控（M2+ 全门控）
# M1 资产预置：fetch webgl 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/webgl-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/99-webgl-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webgl-compat"
DIRS=( "webgl"  )

# pin rev 处 webgl/ 为 12 文件平铺目录（8 .html 回归用例 + idlharness.any.js +
# common.js + 2 元数据，无子目录）；.any.js 为 WebGL 接口面（idlharness）唯一
# 形态，按 lib.sh opt-in 拉取。
GOAL_PULL_ANY_JS=1

goals_fetch_all

# M2 idlharness 资产补拉（2026-10-22 用户裁决③）：idlharness.any.js 的
# `idl_test(['webgl1','webgl2'], ['dom'])` 经 fetch('/interfaces/<id>.idl') 取 IDL、
# META script 内联 /resources/{WebIDLParser,idlharness}.js——四类资产按 pin rev 补齐，
# runner 侧不再按 idlharness.* skip 记账排除（clipboard/compression 等通道先例是
# 资产缺席的退路，本 goal 有裁决补拉不走该退路）。
# - interfaces/{webgl1,webgl2,dom}.idl：webref 提取生成物，repo 内存在；
#   dom.idl 是 ['dom'] 依赖（WebGLContextEvent : Event 继承解析面）。
# - resources/idlharness.js：repo 内存在。
# - resources/WebIDLParser.js：repo 内**不存在**（wpt build 期生成物）；上游部署
#   形态 = resources/webidl2/lib/webidl2.js 的 webpack UMD dist（同字节数 125228，
#   暴露 globalThis.WebIDL2）。按 pin rev 拉 lib/webidl2.js 落到部署路径名，
#   runner 侧 wpt_meta_scripts 无需特判。
# fetch_raw 幂等（已存在且非 FORCE 跳过），失败记账不中止（goals_fetch_all 同语义）。
fetch_raw "resources/idlharness.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "resources/webidl2/lib/webidl2.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
cp "${WPT_DATA}/resources/webidl2/lib/webidl2.js" "${WPT_DATA}/resources/WebIDLParser.js" 2>/dev/null \
  || fetch_raw "resources/WebIDLParser.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
for idl in webgl1 webgl2 dom; do
  fetch_raw "interfaces/${idl}.idl" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
done

goals_inventory
goals_next_steps "${GOAL}"
