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
GOAL_FETCH_DEEP=1

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
