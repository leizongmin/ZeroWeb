#!/usr/bin/env bash
# Goal: dom-expansion（编号 34——推进顺序见同目录 README.md）
# dom/ranges + html/dom 扩面（coverage-2026-10-27 对账：dom/ranges 44,720 子测试
# 0/73 文件；html/dom 72,282 子测试 0/394 文件——40 号 html-syntax-compat 收口时
# 只预置了 40 文件子集）。
#
# 两面性质不同：dom/ranges 是 Range API 完备性（73 文件中小域，引擎+语料双活）；
# html/dom 是 HTML 元素/Document DOM 接口面（394 文件，idlharness.any.js 为
# build 期生成资产上游不存在，拉取会记 1 个失败属预期）。语料补全后归既有
# runner 通道扩面：ranges → testharness-dom、html/dom → testharness-html-syntax
# （M1 rally 轮落地，契约见 docs/goal/dom-expansion.md DC-1）。
# 用法：bash tests/wpt-runner/scripts/goals/34-dom-expansion.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="dom-expansion"
GOAL_FETCH_DEEP=1
GOAL_PULL_ANY_JS=1
DIRS=(
  "dom/ranges"
  "html/dom"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
