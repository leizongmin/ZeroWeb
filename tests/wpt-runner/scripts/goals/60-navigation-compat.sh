#!/usr/bin/env bash
# Goal: navigation-compat（编号 60——推进顺序见同目录 README.md）
# M3 iframe 切片用户门控
# M1 资产预置：fetch navigation-api + 会话历史/导航遍历 + iframe 元素属性 corpus 子集；
# 基线运行（runner 通道 + Makefile target + skip 规则）= 各 goal M1 rally 轮落地，
# 契约见 docs/goal/navigation-compat.md DC-1。
#
# 目录取舍（2026-10-07 M1 实测，WPT_REV pin 下无顶层 history/——会话历史 corpus 实际
# 落在 html/browsers/history/** 四叶目录）：
# - navigation-api/  — Navigation API（navigatesuccess/currententry 等）面
# - html/browsers/history/{the-history-interface,the-location-interface,
#   the-session-history-of-browsing-contexts,joint-session-history} — history/location/
#   会话历史条目语义（DC-2 轻面标尺）
# - html/browsers/browsing-the-web/{history-traversal,navigating-across-documents,
#   scroll-to-fragid,unloading-documents} — 前进后退遍历/跨文档导航/fragid/卸载事件序
# - html/semantics/embedded-content/the-iframe-element — iframe 元素属性基线
#   （DC-1 资产面；实现侧 frame tree 仍属 M3 用户门控，不因导入解锁）
# 排除：html/browsers/{the-window-object,windows,nested-browsing-contexts 等}
# 多窗口/popup/manual 面与嵌套 browsing context 深结构面（runner 单文档无 frame 树，
# M3 门控后随实现面重评）；offline/origin/sandboxing 非本 goal 范围。
# 用法：bash tests/wpt-runner/scripts/goals/60-navigation-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="navigation-compat"
DIRS=(
  "navigation-api"
  # depth-1 递归不覆盖的嵌套叶（用例 META/src 引用的嵌套 resources/ 支持 .js——
  # 2026-10-07 M1 基线首跑缺口盘点：190 处引用 25 个缺失文件，本表补齐 goal 域内部分）
  "navigation-api/navigation-methods/return-value"
  "navigation-api/navigation-methods/resources"
  "navigation-api/navigation-history-entry/resources"
  "navigation-api/state/resources"
  "navigation-api/precommit-handler/resources"
  "navigation-api/navigate-event/resources"
  "navigation-api/navigation-activation/resources"
  "navigation-api/per-entry-events/resources"
  "html/browsers/browsing-the-web/navigating-across-documents/replace-before-load/resources"
  "html/browsers/browsing-the-web/navigating-across-documents/initial-empty-document/resources"
  "html/browsers/browsing-the-web/navigating-across-documents/multiple-globals/resources"
  "html/browsers/browsing-the-web/navigating-across-documents/resources"
  "html/browsers/history/the-history-interface"
  "html/browsers/history/the-location-interface"
  "html/browsers/history/the-session-history-of-browsing-contexts"
  "html/browsers/history/joint-session-history"
  "html/browsers/browsing-the-web/history-traversal"
  "html/browsers/browsing-the-web/navigating-across-documents"
  "html/browsers/browsing-the-web/scroll-to-fragid"
  "html/browsers/browsing-the-web/unloading-documents"
  "html/semantics/embedded-content/the-iframe-element"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
