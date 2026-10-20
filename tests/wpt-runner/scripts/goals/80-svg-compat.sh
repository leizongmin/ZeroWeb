#!/usr/bin/env bash
# Goal: svg-compat（编号 80——推进顺序见同目录 README.md）
# SVG 文档模型（渲染面归 rendering-compat）
# M1 资产预置：fetch svg 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/svg-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/80-svg-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="svg-compat"
DIRS=( "svg"  )
# 语料三层嵌套（`types/scripted/`、`struct/reftests/`、`animations/scripted/` 等
# ——上游 pin 全树 2224 blobs），shallow Contents API 只递归一层拉不全——走 git
# trees 深挖（lib.sh fetch_dir_deep，2 次 API 调用拿全量清单；WebCrypto/workers
# 先例）。扩展名分流天然覆盖 .svg 文档（WPT svg/ 用例页大量为 .svg 形态——渲染
# 断言面）与 .html（testharness DOM 面）；渲染断言面占比盘点在 M1 rally 轮按
# 内容分类入档（docs/goal/svg-compat/evidence/）。
# runner 可执行子集 skip 规则（src/testharness_svg.rs svg_case_skipped 同源）：
#   ① 路径含 `.https.`（secure context 前提）
#   ② .html/.htm 不含 `testharness`（reftest/ref/manual/demo——渲染断言面）
#   ③ 内容标记 `iframe`（frame 树面，navigation-compat M3 门槛后重入）
#   ④ 内容标记 `window.open`（第二文档面，同 iframe 门槛）
#   ⑤ `.svg` 文档扫描层排除（XML 文档解析路径缺位，M2 重入条件）
#   SMIL（animations/）不设目录级排除——SVGAnimationElement 接口面属 M2 DOM
#   类层级；SMIL 时序面按 goal 契约评估后挂账（M4 定稿）。
GOAL_FETCH_DEEP=1

# 共享 helper 补拉（svg/animations 182 案引用的 SMIL testharness 包装器——位于
# WPT 根 resources/，目录限定 fetch 拉不到；无它 182 案脚本加载即 ReferenceError
# 死、零声明测试。fetch_raw 幂等单文件，同 pin rev）。/common/rendering-utils.js
# 与 reftest-wait.js（reftest 支撑，渲染面）、testdriver.js（runner 无 testdriver
# 面，1 案）不拉，缺口在 M1 evidence 记账。
fetch_raw "resources/SVGAnimationTestCase-testharness.js"

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
