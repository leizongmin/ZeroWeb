#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批六：
# html/semantics 补簇整批接入——html-semantics 套件 9 扫描根（HTML_SEMANTICS_TEST_SUBDIRS：
# forms/embedded-content/document-metadata/grouping-content/text-level-semantics/
# tabular-data/interactive-elements/scripting-1 八元素域 + links）之外的 10 簇
# （summary-chrome 通道内分母 428 键 6,444 子测试 chrome 全过 6,041——popovers
# 127 键 3,918 为最大单簇（chrome 过 3,902）、the-button-element 812、selectors
# 413、permission-element 382 次之）。防跨套件重复计账：补簇 = 与
# HTML_SEMANTICS_TEST_SUBDIRS 互补（该套件不扫这 10 簇，goal 文档零认领实证），
# 周跑聚合金各记一次。links 簇不拉不扫——html-semantics 套件第 9 扫描根已认领
# （fetch 25 五叶 + 该 goal 账本 links 119P/20F/4T 在账），本批收之则聚合金重复
# 计账（首版误收 11 簇，2026-11-21 复核剔除）。
#
# 拉取面：目录清单按 pin 31597693 git trees 子树显式枚举（fetch_dir_html 只递归
# 一层，depth-2+ 子目录与 resources/support 助资产目录逐个显式，30 目录；
# 幂等标记防 API 限流重列）。
# 不拉不计分：resources/support 助资产目录只拉不扫（分母 0 键实证）；
# obsolete/crashtests 分母 2 键 0 子测试，随簇扫入诚实记 0。
# GOAL_PULL_ANY_JS=1 预设——簇内若存在 .any.js 唯一形态语料不漏拉（空集无害）。
# .headers sidecar 全量拉取（默认语义）。
# runner 通道：HTML_SYNTAX_CORPUS_SUBDIRS 加收 23 计分目录（9 顶层 + 14 计分
# 子目录——selectors 顶层无独立计分键故通道不列，resources/support 只拉不扫）
# + `.https` 命名面簇内解锁
# （permission-element 16 键 + menu tentative 1 键——后者落 idlharness 命名门
# 仍排除，挂账 1 键）。the-h1-h2-h3-h4-h5-and-h6-elements 子目录分母 0 键只拉
# 不扫（headingoffset 计分在 sections 顶层 3 键 107 子测试）。
# 用法：bash tests/wpt-runner/scripts/goals/59-wptfyi-html-semantics-complement.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
FAILED_FETCHES=0
export GOAL_PULL_ANY_JS=1

# 10 顶层簇（计分，通道扫描根；links 归 html-semantics 套件不在此列）
fetch_dir_html "html/semantics/disabled-elements"
fetch_dir_html "html/semantics/edits"
fetch_dir_html "html/semantics/interestfor"
fetch_dir_html "html/semantics/menu"
fetch_dir_html "html/semantics/obsolete"
fetch_dir_html "html/semantics/permission-element"
fetch_dir_html "html/semantics/popovers"
fetch_dir_html "html/semantics/sections"
fetch_dir_html "html/semantics/selectors"
fetch_dir_html "html/semantics/the-button-element"

# 子目录显式枚举（fetch_dir_html 只递归一层；逐目录独立完成标记防限流重列）
fetch_dir_html "html/semantics/edits/the-del-element"
fetch_dir_html "html/semantics/edits/the-ins-element"
fetch_dir_html "html/semantics/interestfor/resources"
fetch_dir_html "html/semantics/menu/tentative"
fetch_dir_html "html/semantics/menu/tentative/resources"
fetch_dir_html "html/semantics/obsolete/crashtests"
fetch_dir_html "html/semantics/permission-element/camera"
fetch_dir_html "html/semantics/permission-element/geolocation-element"
fetch_dir_html "html/semantics/permission-element/install"
fetch_dir_html "html/semantics/permission-element/microphone"
fetch_dir_html "html/semantics/permission-element/permission-icon"
fetch_dir_html "html/semantics/permission-element/usermedia"
fetch_dir_html "html/semantics/permission-element/usermedia/legacy-mode"
fetch_dir_html "html/semantics/permission-element/usermedia/resources"
fetch_dir_html "html/semantics/popovers/resources"
fetch_dir_html "html/semantics/sections/the-h1-h2-h3-h4-h5-and-h6-elements"
fetch_dir_html "html/semantics/selectors/case-sensitivity"
fetch_dir_html "html/semantics/selectors/pseudo-classes"
fetch_dir_html "html/semantics/the-button-element/command-and-commandfor"
fetch_dir_html "html/semantics/the-button-element/command-and-commandfor/resources"

DIRS=( "html/semantics" )

# 批十五（2026-11-22）支撑文件补档：拉取簇外的共享支撑文件，本地由其他通道带入、
# CI 分片作业只拉本树——同批十四 62 脚本根因类（批十五定谳补齐，页面清单为
# HTML_SYNTAX_CORPUS_SUBDIRS 补簇计分域实跑页引用闭包扫描产物）：
# - resources/WebIDLParser.js＋resources/idlharness.js——interestfor idlharness 页；
# - resources/WebIDLParser.js（webappapis event-handler 形 8 页借道同款）与
#   cors/support.js 由 57 脚本补档（同分片各自自足）；
# - common/top-layer.js＋close-watcher/resources/helpers.js——popovers
#   close-request/top-layer-combinations 7 页（25 脚本同款文件，本分片另一作业
#   不拉 25——各自自足补档）；
# - resources/check-layout-th.js——popovers popover-anchor-multicol-display 页；
# - fonts/ahem.css＋images/blue.png——interestfor 字体/事件页（25 脚本 green 系
#   同款模式，blue 系本簇专用）；
# - media/movie_5.mp4＋media/sound_5.mp3——command-and-commandfor 音视频行为页
#   （媒体通道资产，本簇 3 页引用）。
# 批十七（2026-11-22）勘误：resources/WebIDLParser.js 上游不是独立 Git blob
# （wptserve 对 resources/webidl2/lib/webidl2.js 的生成路由，双源 404）——拉真身
# 落别名（57 脚本同款）：
if fetch_raw "resources/webidl2/lib/webidl2.js"; then
  cp "${WPT_DATA}/resources/webidl2/lib/webidl2.js" "${WPT_DATA}/resources/WebIDLParser.js"
else
  FAILED_FETCHES=$((FAILED_FETCHES + 1))
fi
fetch_raw "resources/idlharness.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "resources/check-layout-th.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "common/top-layer.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "close-watcher/resources/helpers.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "fonts/ahem.css" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "images/blue.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "media/movie_5.mp4" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "media/sound_5.mp3" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

# 批十八（2026-11-22）：html/semantics/scripting-1 模块脚本 .any.js window 形补档
# ——子树 27 源中 META: global= 含 window 的 14 源显式单文件拉取（worker-only 12
# 源不拉：本通道无 worker 形键，runner html_syntax_case_skipped META 门同形不生成
# window 键、分母亦无 .any.html——worker 通道域挂账）。.html 形不拉：scripting-1
# .html 已由 25 脚本（html-sem 分片）带入、html-semantics 套件计分，本分片收之则
# 聚合金重复计账（本脚本头注释防重同款）。扫描域 = HTML_SYNTAX_CORPUS_SUBDIRS 批十八
# 5 平铺叶子。支撑闭包 = 14 源静态引用逐文件枚举；/common/utils.js 由 58 脚本带入
# （同分片各自自足）；serve-custom-response.py 与 module-fetch-tester.py 走 py-route
# （testharness_py_routes/site.rs s267/s346 臂）不入盘，serve-js-then-text.py/
# serve-text-then-js.py 无 py-route 臂未拉（text-module/repeated-imports MIME 缓存
# 面预期 Fail 记账，行为批议）。
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/charset-bom.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/invalid-content-type.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/non-object.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/repeated-imports.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/blob-url.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/concurrent-failed-imports.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/repeated-imports.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/microtasks/basic.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/microtasks/with-import-assertions.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/charset-bom.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/ignore-content-type.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/ignore-extension.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/repeated-imports.any.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/bom-utf-16be.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/bom-utf-16le.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/bom-utf-8.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/array.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/module.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
# non-object.any.js 静态 import 的 4 个支撑 json（批二十一 CI 缺档：Failed to
# fetch module './null.json' 等 4 形 → 该键 1P/4F，本地 5P/0F；分片语料隔离律
# 第 6 例——支撑闭包枚举漏源文件自身 import 闭包）
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/null.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/true.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/false.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/json-module/string.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/microtasks/ticker.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/microtasks/empty-module.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__microtask__parent.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__microtask__check-get-sync.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__microtask__check-set-async.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__task__parent.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__task__check-get-sync.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/module/top-level-await/sibling-imports-not-blocked__task__check-set-async.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/bom-utf-16be.txt" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/bom-utf-16le.txt" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/bom-utf-8.txt" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/file" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/file.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/file.json" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/file.txt" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
goals_inventory
goals_next_steps "${GOAL}"
