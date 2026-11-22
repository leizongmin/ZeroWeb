#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批四：
# html/webappapis 簇语料（11-20 快照缺口 3,672；summary-chrome 4b26db8978 分母
# 370 键 3,566 子测试 chrome 全过 3,665——通道内 html（含 .https 复合形）267 键
# 2,138 / window.html 48 键 184 / any.html 18 键 571＝333 键 2,893 子测试，
# worker 族 31 键 654 与 xhtml/svg 形通道外挂账）。
#
# 拉取面：目录清单按 pin 31597693 git trees html/webappapis 子树全量枚举
# （46 目录显式列出——fetch_dir_html 只递归一层，depth-3+ 资产目录逐个显式；
# 幂等标记防 API 限流重列）。GOAL_PULL_ANY_JS=1：.any.js 是 microtask-queuing/
# timers/atob 等域唯一语料形态（18 源，分母 .any.html 生成形映射），必须拉。
# .worker.js（4 源）不拉——runner 本通道零消费（分母 worker.html 562 子测试
# 挂账，worker 通道域外）。.py（wptserve 服务端）/.yml（元数据）不在拉取模式。
# runner 通道：HTML_SYNTAX_CORPUS_SUBDIRS 加收 25 计分目录（resources/support
# 助资产目录只拉不扫——分母 0 键实证）。
# 用法：bash tests/wpt-runner/scripts/goals/57-wptfyi-html-webappapis.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
FAILED_FETCHES=0
export GOAL_PULL_ANY_JS=1

fetch_dir_html "html/webappapis/animation-frames"
fetch_dir_html "html/webappapis/atob"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/closing-the-input-stream"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/document-write"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/document-write/resources"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/document-writeln"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/html-unsafe-methods"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/html-unsafe-methods/resources"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/opening-the-input-stream"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/opening-the-input-stream/resources"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/the-innerhtml-property"
fetch_dir_html "html/webappapis/dynamic-markup-insertion/the-outerhtml-property"
fetch_dir_html "html/webappapis/microtask-queuing"
fetch_dir_html "html/webappapis/scripting"
fetch_dir_html "html/webappapis/scripting/event-loops"
fetch_dir_html "html/webappapis/scripting/event-loops/resources"
fetch_dir_html "html/webappapis/scripting/events"
fetch_dir_html "html/webappapis/scripting/events/event-handler-processing-algorithm-error"
fetch_dir_html "html/webappapis/scripting/events/event-handler-processing-algorithm-error/resources"
fetch_dir_html "html/webappapis/scripting/events/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-agent-formalism"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-agent-formalism/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/current"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/current/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/function"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/function/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/relevant"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/relevant/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/integration-with-the-javascript-job-queue/resources/resources"
fetch_dir_html "html/webappapis/scripting/processing-model-2/support"
fetch_dir_html "html/webappapis/scripting/processing-model-2/unhandled-promise-rejections"
fetch_dir_html "html/webappapis/scripting/processing-model-2/unhandled-promise-rejections/support"
fetch_dir_html "html/webappapis/structured-clone"
fetch_dir_html "html/webappapis/system-state-and-capabilities/the-navigator-object"
fetch_dir_html "html/webappapis/system-state-and-capabilities/the-navigator-object/resources"
fetch_dir_html "html/webappapis/the-windoworworkerglobalscope-mixin"
fetch_dir_html "html/webappapis/the-windoworworkerglobalscope-mixin/support"
fetch_dir_html "html/webappapis/timers"
fetch_dir_html "html/webappapis/update-rendering"
fetch_dir_html "html/webappapis/user-prompts"
fetch_dir_html "html/webappapis/user-prompts/cannot-show-simple-dialogs"
fetch_dir_html "html/webappapis/user-prompts/cannot-show-simple-dialogs/support"
fetch_dir_html "html/webappapis/user-prompts/resources"

DIRS=( "html/webappapis" )

# 批十五（2026-11-22）支撑文件补档：拉取子树外的共享支撑文件，本地由其他通道
# 带入、CI 分片作业只拉本树——同批十四 62 脚本根因类（批十五定谳补齐，页面清单
# 为 HTML_SYNTAX_CORPUS_SUBDIRS 计分域实跑页引用闭包扫描产物）：
# - cors/support.js——unhandled-promise-rejections allow/disallow-crossorigin 2 页；
# - resources/WebIDLParser.js——scripting/events event-handler idlharness 形 8 页
#  （interestfor idlharness 第 9 页由 59 脚本补档，各自自足）。
fetch_raw "cors/support.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "resources/WebIDLParser.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

goals_inventory
goals_next_steps "${GOAL}"
