#!/usr/bin/env bash
# Goal: html-syntax-compat（编号 40——推进顺序见同目录 README.md）
# html-compat（fixture 制）的 WPT 化续篇
# M1 资产预置：fetch html/syntax html/dom 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/html-syntax-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="html-syntax-compat"
DIRS=( "html/syntax" "html/dom"  )

goals_fetch_all

# depth-2 support/ 资产补拉（fetch_dir_html 深度<1 不入子子目录）：运行面 RUN 案引用的
# helper——render-blocking support 脚本（utils.js / test-render-blocking.js / dummy-1.*）
# 与 parsing support 脚本（DOMContentLoaded-defer.js / svg-script-self-closing.js）。
# lib.sh 拉取模式不含 .mjs/.css（此前 corpus 无此形态）——dummy-1.mjs / target-red.css
# 显式补拉（/common/sab.js 显式补拉先例）。
fetch_dir_html "html/dom/render-blocking/support"
fetch_dir_html "html/syntax/parsing/support"
# depth-2 resources/ 补拉（html5lib 树构造语料——html5lib_write/url/write_single 三案
# 引 resources/common.js + tests*.dat，M2 首轮跑 script fetch failed 记账后补）。
fetch_dir_html "html/syntax/parsing/resources"
# depth-2 references/ 补拉（P5 落地面——charset/ 11 案 rel=match 参考页；runner
# reftest-upstream 通道经 R5003 字节级嗅探解码后按参考页像素对比运行）。
fetch_dir_html "html/syntax/charset/references"
# depth-2 support/ 补拉（P5 通道外余量收编——xmldecl/ 3 案 iframe 子文档面：40 枚
# bogo-XML-declaration/meta/编码窗 fixture（.htm）+ test_support.js 断言表；
# lib.sh fetch_dir_html 增 .htm/.headers 后此目录一次拉全）。
fetch_dir_html "html/syntax/xmldecl/support"
# 单文件依赖补拉：html/resources/common.js（HTML5_ELEMENTS 表——outerHTML 双域 +
# name-content-attribute 三案引 ../../resources/common.js；DIRS 深度递归不达
# html/resources 域）与实体表数据 named-character-references-data.js
# （named-character-references 案 sibling 引用；.js 依赖 fetch_dir_html 幂等快路径
# 跳过后不保新）。2026-10-19 复测发现两资产与 6 个 parsing 页从 wpt-data 脱落
# （gitignored 目录无删除路径，成因不明）——断补后通道总分塌缩 2595 子测，记账。
# html/dom/render-blocking/dummy.html 在 pin 中不存在（上游后增资产）——034/035/037
# 案该引用 404 为 pin 期资产缺口，不补拉。
fetch_raw "html/resources/common.js"
fetch_raw "html/syntax/parsing/named-character-references-data.js"
fetch_raw "html/dom/render-blocking/support/dummy-1.mjs"
fetch_raw "html/dom/render-blocking/support/target-red.css"

goals_inventory
goals_next_steps "${GOAL}"
