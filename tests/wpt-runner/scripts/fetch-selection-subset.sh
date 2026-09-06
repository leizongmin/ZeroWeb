#!/usr/bin/env bash
# Fetch a pinned subset of the upstream WPT selection/ tests (testharness style)
# used by the editing/contenteditable goal (docs/goal/editing-contenteditable.md, M1 / DC-1).
#
# Strategy: first batch = main-thread .html cases of selection/ root (Selection
# API observable face — window/document.getSelection, addRange/collapse/extend,
# getRangeAt/removeRange/removeAllRanges, type/isCollapsed/toString,
# setBaseAndExtent, deleteFromDocument, selectAllChildren). These directly
# exercise the Selection singleton + Range interplay against native bindings.
# Subdirectories (contenteditable/, textcontrols/, caret/) are appended per
# later slices as the baseline grows.
#
# 与 fetch-dom-subset.sh 同构（同 WPT_REV pin + GitHub API 列目录 + raw 拉单文件）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"

# 第一批：selection/ 根目录主线程 .html 用例（Selection API 可观察面）。
# 排除 manual（dir-manual.html 需真拖拽交互）、repaint/ref 类（渲染域断言）与
# .tentative.html 中依赖未定规范的 modify-*（selection.modify 无 shim 面）——
# 排除项记录于 evidence 导入清单，非静默丢弃。
SUBDIRS=(
  "selection"
)

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  echo "fetch ${relative}"
  curl -fsSL --retry 3 --max-time 60 "${RAW_ROOT}/${relative}" -o "${target}"
}

# 显式导入清单（selection/ 根 .html 主线程用例；2026-09-07 M1 切片 1）。
# 每行一个相对路径。新增时保持与 evidence 导入清单同步。
CASES=(
  "selection/getSelection.html"
  "selection/addRange.htm"
  "selection/collapse.htm"
  "selection/collapseToStartEnd.html"
  "selection/getRangeAt.html"
  "selection/isCollapsed.html"
  "selection/removeAllRanges.html"
  "selection/removeRange.html"
  "selection/selectAllChildren.html"
  "selection/setBaseAndExtent.html"
  "selection/deleteFromDocument.html"
  "selection/deleteFromDocument-HTMLDetails.html"
  "selection/type.html"
  "selection/anchor-removal.html"
  "selection/script-and-style-elements.html"
  "selection/toString-ff-bug-001.html"
  "selection/extend-exception.html"
  "selection/Document-open.html"
  "selection/onselectionchange-on-document.html"
  "selection/onselectionchange-on-distinct-text-controls.html"
)

# 共享 helper（用例依赖的 .js，一并拉取）。
HELPERS=(
  "selection/common.js"
)

for rel in "${CASES[@]}" "${HELPERS[@]}"; do
  fetch_raw "$rel"
done

echo "selection subset ready: ${#CASES[@]} cases + ${#HELPERS[@]} helpers @ ${WPT_REV:0:12}"
