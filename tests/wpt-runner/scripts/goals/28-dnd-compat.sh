#!/usr/bin/env bash
# Goal: dnd-compat（编号 28——推进顺序见同目录 README.md；**门控 goal**，学 99-webgl）
# HTML5 拖放面（DataTransfer/DragEvent 构造器为纯 JS 面；真实拖放交互依赖 host
# drag-input 管道——web-api-batch2 挂账记录）。
# 分工：M1 盘点 + M2 构造器/DataTransfer 面 = 自主切片；拖放交互管线 = **用户点名
# 门控**（M1 盘点为唯一自主切片的规则见 README.md 99 行先例）。
# 用法：bash tests/wpt-runner/scripts/goals/28-dnd-compat.sh（FORCE=1 强制重拉；
# GITHUB_TOKEN 环境变量可选——深挖模式仅 4 次 API 调用，未认证 60 req/h 通常够）。
#
# 语料路径注记（2026-10-18 M1 实测）：上游**无顶层 dnd/ 目录**（Contents API 404
# 实证），拖放语料在 html/editing/dnd/（957 文件；selection 193 / platform 90 /
# overlay 77 / svg 77 / events 68 / datastore 66 ...）。可执行 testharness 面集中
# 在 dom/ synthetic/ datastore/*.html the-draggable-attribute/ the-dropzone-attribute/
# 与三 interface 目录（全 -manual）；其余目录为真实拖拽交互面。GOAL_FETCH_DEEP=1
# 走 git trees 全量拉取（census 甄别用；wpt-data gitignored）——runner 扫描面见
# testharness_dnd.rs DND_TEST_SUBDIRS。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="dnd-compat"
GOAL_FETCH_DEEP=1
DIRS=( "html/editing/dnd" )

goals_fetch_all

# 扩展名过滤器外的小面依赖（lib.sh 只拉 html/js/json/xml/xhtml/svg）：
# - dom/draggable.html 引用 ../resources/1x1-transparent.gif（img 默认 draggable 面）
# - the-draggable-attribute/draggable_attribute.html 引用 /html/semantics/interfaces.js
#   （元素清单 helper——runner 侧经 inline_extras 内联，见 testharness_dnd.rs）
fetch_raw "html/editing/dnd/resources/1x1-transparent.gif" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/semantics/interfaces.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
  echo "  ⚠ ${FAILED_FETCHES} 个文件拉取失败（瞬时错误不计入中止；重跑脚本幂等续拉）" >&2
fi

goals_inventory
goals_next_steps "${GOAL}"
