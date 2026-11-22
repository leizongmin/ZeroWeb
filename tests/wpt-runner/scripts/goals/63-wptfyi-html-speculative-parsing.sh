#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批十二：
# html/syntax/speculative-parsing 族语料——预加载扫描器（preload scanner）执行面
# 127 键：generated/page-load/*.tentative.html 63（iframe 载 framed 页：
# slow.py 脚本 stall + document.write('<plaintext>') 杀解析 + 尾随 img 资源，
# 扫描器应在 stall 窗口内预取）+ generated/document-write/*.tentative.sub.html 63
# （页内 document.write 同构）+ expect-no-linked-resources/
# no-speculative-fetch.tentative.optional.html 1（window.open popup 门控——runner
# 无弹窗管理，skip 记账）。
#
# 机制（pin 逐文件实读）：两端各打
# `resources/stash.py?action=put&uuid={{GET[uuid]}}&encodingcheck=&Gbreve;`——
# put 记 `param-encodingcheck: <&encodingcheck= 之后原文>` + 请求 Origin/Accept/
# Referer 头行；顶页 onload 后 `stash.py?action=take`（take 即清）先取
# speculative 结果，再 iframe `../resources/<title>-nonspeculative.sub.html`
# （可解析 img 直打 put）后二次 take，断言两次头序列逐行相等（Referer 经
# `/generated/…` 归一）。stash.py 端点不在语料拉取扩展名分流（.py 不拉）——
# runner 侧 py-route 仿真（testharness_py_routes misc.rs route_speculative_stash）
# + `/common/slow.py` 路由臂（runner 无真实网络时序，delay 即返）。
#
# 执行通道：testharness-html-speculative（testharness.rs
# HTML_SPECULATIVE_CORPUS_SUBDIRS 3 个平铺目录——page-load/document-write/
# expect-no-linked-resources；generated/resources/ 62 个 nonspec 助页与
# page-load/resources/ 63 个 framed 助页静态伺服不进扫描）。分母（summary-chrome
# @4b26db8978）127 键 / 127 子测试 / chrome 过 118（9 键 chrome 亦未过，负案
# chrome 本身过——批十二 leg-2 扫描器刀兑现上限 +118）。
#
# 实测形态（leg-1 首跑 + A/B 复跑，2026-11-22）：无扫描器，但 runner iframe/
# 嵌套 write 管道 document.write('<plaintext>') kill 不生效 + template 内容
# 取数不惰性 → 126 执行键 **50 Pass / 76 Timeout**、墙钟 ~15 min（推翻预估
# 0 Pass / 63 min；pass 面根因与逐类归属见批十二证据）。pass 面依赖引擎
# loader 怪癖而非扫描器语义，leg-2 engine 修（扫描器刀 + loader 惰性/kill
# 语义）落地后此面会重排；A/B 零漂移为入账前提。
# 拉取面：fetch_dir_deep 子树整拉（4 次 API 列目录 + jsdelivr CDN 逐文件，
# 不吃 API 配额）；tools/ 生成器与 WEB_FEATURES.yml 随扩展名分流不入盘。
# 用法：bash tests/wpt-runner/scripts/goals/63-wptfyi-html-speculative-parsing.sh
#（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
GOAL_FETCH_DEEP=1
FAILED_FETCHES=0

DIRS=(
  html/syntax/speculative-parsing
)
goals_fetch_all

# 批十五（2026-11-22）支撑文件补档：/common/utils.js（定义 token()）在拉取子树外，
# 本地由其他 goal 通道带入、CI 分片作业只拉本树——缺档形＝套件 126 键全灭
# 「token is not defined」（ReferenceError 先于任何断言，run 44972755575
# catch-all-4 实测 126 F / 0 P / 0 TO，本地同码 43 P / 83 TO）。同批十四 62 脚本
# 根因类（META script= 传递闭包在 fetch 目录外的支撑文件）。
fetch_raw "common/utils.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

echo "== speculative-parsing 拉取完成（失败 ${FAILED_FETCHES}，pin ${WPT_REV:0:9}）=="
printf "  入盘文件: %s\n" \
  "$(find "${WPT_DATA}/html/syntax/speculative-parsing" -type f ! -name '.goal-fetched-*' 2>/dev/null | wc -l)"
printf "  page-load 顶页: %s / document-write 顶页: %s / nonspec 助页: %s\n" \
  "$(find "${WPT_DATA}/html/syntax/speculative-parsing/generated/page-load" -maxdepth 1 -name '*.tentative.html' 2>/dev/null | wc -l)" \
  "$(find "${WPT_DATA}/html/syntax/speculative-parsing/generated/document-write" -maxdepth 1 -name '*.tentative.sub.html' 2>/dev/null | wc -l)" \
  "$(find "${WPT_DATA}/html/syntax/speculative-parsing/generated/resources" -name '*-nonspeculative.sub.html' 2>/dev/null | wc -l)"
