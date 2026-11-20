#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批五：
# html/browsers the-window-object + origin 簇语料（summary-chrome 4b26db8978
# 通道内分母 234 键 1,441 子测试——the-window-object 自有 84 键 629（window
# properties.https 178 子测试为最大单面）+ origin 150 键 812（api
# hyperlinkelementutils.window 104 为次）。
#
# 拉取面：目录清单按 pin 31597693 git trees 子树显式枚举（fetch_dir_html 只递归
# 一层，resources/support 助资产目录逐个显式；幂等标记防 API 限流重列）。
# 不拉不计分：named-access-on-the-window-object（DOM 套件 slice28 既有记账域，
# 防跨套件重复计账）、open-close/non_automated + garbage-collection/
# non-automated（分母 0 键实证——非自动化手动面）、security-window（pin 无此
# 目录——4b26db8978 快照上游漂移，124 子测试挂账随周跑另报）。
# GOAL_PULL_ANY_JS=1：origin/api origin-from-*.any.js 是该域唯一语料形态（分母
# .any.html 生成形映射），必须拉。.worker.js 不拉——runner 本通道零消费
# （origin 分母 worker 族 7 键挂账）。.headers sidecar 全量拉取（COOP/COEP
# agent-clusters 面响应头语义，runner fetch 通道读 sidecar 先例）。
# runner 通道：HTML_SYNTAX_CORPUS_SUBDIRS 加收 15 计分目录（resources/support
# 只拉不扫——分母 0 键实证）。
# 用法：bash tests/wpt-runner/scripts/goals/58-wptfyi-html-browsers-window.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
FAILED_FETCHES=0
export GOAL_PULL_ANY_JS=1

fetch_dir_html "html/browsers/the-window-object"
fetch_dir_html "html/browsers/the-window-object/accessing-other-browsing-contexts"
fetch_dir_html "html/browsers/the-window-object/garbage-collection-and-browsing-contexts"
fetch_dir_html "html/browsers/the-window-object/open-close"
fetch_dir_html "html/browsers/the-window-object/open-close/resources"
fetch_dir_html "html/browsers/origin"
fetch_dir_html "html/browsers/origin/api"
fetch_dir_html "html/browsers/origin/api/resources"
fetch_dir_html "html/browsers/origin/cross-origin-objects"
fetch_dir_html "html/browsers/origin/cross-origin-objects/resources"
fetch_dir_html "html/browsers/origin/inheritance"
fetch_dir_html "html/browsers/origin/inheritance/resources"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/1-iframe"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/2-iframes"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/getter-special-cases"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/getter-special-cases/resources"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/iframe-navigation"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/popups"
fetch_dir_html "html/browsers/origin/origin-keyed-agent-clusters/resources"
fetch_dir_html "html/browsers/origin/relaxing-the-same-origin-restriction"
fetch_dir_html "html/browsers/origin/relaxing-the-same-origin-restriction/support"

DIRS=( "html/browsers/the-window-object" "html/browsers/origin" )
goals_inventory
goals_next_steps "${GOAL}"
