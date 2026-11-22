#!/usr/bin/env bash
# Goal: html-syntax-compat（编号 40——推进顺序见同目录 README.md）
# html-compat（fixture 制）的 WPT 化续篇
# M1 资产预置：fetch html/syntax html/dom 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/archive/html-syntax-compat.md DC-1。
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

# wptfyi-html goal DC-2（docs/goal/wptfyi-html.md）：html/dom idlharness.https.html
# 解锁面——idl_test 运行期 fetch('/interfaces/html.idl')，page fetch 静态文件回退
# 路径供给（webgl idlharness 先例，webgl1/webgl2/dom.idl 同目录在册）；.idl 后缀
# 不在 fetch_dir_html 拉取模式内，显式单文件。页面 dependencies 列 12 个依赖 IDL
# （include/implements 解析用，不直接出测试），除 dom.idl 外逐个补拉。
fetch_raw "interfaces/html.idl"
fetch_raw "interfaces/wai-aria.idl"
fetch_raw "interfaces/SVG.idl"
fetch_raw "interfaces/cssom.idl"
fetch_raw "interfaces/touch-events.idl"
fetch_raw "interfaces/pointerevents.idl"
fetch_raw "interfaces/uievents.idl"
fetch_raw "interfaces/xhr.idl"
fetch_raw "interfaces/FileAPI.idl"
fetch_raw "interfaces/mediacapture-streams.idl"
fetch_raw "interfaces/performance-timeline.idl"
fetch_raw "interfaces/trusted-types.idl"
# dom.idl 是 idl_test(['html'], ['dom']) 的依赖基座（webgl M2 先例：webref 提取生成物，
# 上游 pin 在册）。批十六「除 dom.idl 外逐个补拉」的前提是同目录在册——本地由
# 99-webgl-compat.sh 带入、CI syntax-dom 分片只跑本 goal fetch（分片语料隔离律）恒缺
# → idl_test setup 三查询键各 1 Fail（批十七 run 45008050513 实证
# Error fetching /interfaces/dom.idl，5,822 子测试零注册形的真实根因）。
fetch_raw "interfaces/dom.idl"

# dom-expansion goal M1（docs/goal/archive/dom-expansion.md DC-1/DC-2）：html/dom 深层叶子
# 目录补拉——34 号 deep 语料（git trees recursive）扩进来、runner 通道
#（HTML_SYNTAX_CORPUS_SUBDIRS）同步扫描的面；fetch_dir_html depth<1 不达 depth-2，
# 按上方 support/ 先例显式列目录。目录清单与 runner 扫描面一一对应。
fetch_dir_html "html/dom/documents/dom-tree-accessors"
fetch_dir_html "html/dom/documents/dom-tree-accessors/document.getElementsByName"
fetch_dir_html "html/dom/documents/resource-metadata-management"
fetch_dir_html "html/dom/partial-updates/tentative"
fetch_dir_html "html/dom/partial-updates/tentative/fragment"
fetch_dir_html "html/dom/elements/elements-in-the-dom"
fetch_dir_html "html/dom/elements/global-attributes"
fetch_dir_html "html/dom/elements/images"
fetch_dir_html "html/dom/elements/requirements-relating-to-bidirectional-algorithm-formatting-characters"
fetch_dir_html "html/dom/elements/the-innertext-and-outertext-properties"

# wptfyi-html goal DC-2 批十一（docs/goal/wptfyi-html.md）：html/syntax/parsing/template
# 子树补拉——depth-2 测试叶子（批八分母对账「无扫描链」面主项，summary-chrome
# 4b26db8978 计 22 键 ~323 子测试，创建于 template 解析语义域）；目录清单与
# runner 扫描面（HTML_SYNTAX_CORPUS_SUBDIRS template 块）一一对应。子目录内无
# support/resources 助资产（pin 目录列表实证）；parsing/crashtests 记账面不拉。
fetch_dir_html "html/syntax/parsing/template/additions-to-foster-parenting"
fetch_dir_html "html/syntax/parsing/template/additions-to-the-in-body-insertion-mode"
fetch_dir_html "html/syntax/parsing/template/additions-to-the-in-frameset-insertion-mode"
fetch_dir_html "html/syntax/parsing/template/additions-to-the-in-head-insertion-mode"
fetch_dir_html "html/syntax/parsing/template/additions-to-the-in-table-insertion-mode"
fetch_dir_html "html/syntax/parsing/template/appending-to-a-template"
fetch_dir_html "html/syntax/parsing/template/clearing-the-stack-back-to-a-given-context"
fetch_dir_html "html/syntax/parsing/template/creating-an-element-for-the-token"

goals_inventory
goals_next_steps "${GOAL}"
