#!/usr/bin/env bash
# Fetch a pinned subset of the upstream WPT html/canvas tests (testharness style)
# used by the Canvas 2D goal (docs/goal/canvas-2d.md).
#
# Strategy: first batch = main-thread .html cases of the core API surface
# (canvas state / rectangles / transformations / pixel manipulation / line styles).
# Subdirectories are appended per M1 slices as the harness surface grows.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"

# 第一批：核心 API 面主线程 .html 用例（与 test_cases_canvas.rs 既有 40 smoke 面一致）。
SUBDIRS=(
  "html/canvas/element/the-canvas-state"
  "html/canvas/element/drawing-rectangles-to-the-canvas"
  "html/canvas/element/transformations"
  "html/canvas/element/pixel-manipulation"
  "html/canvas/element/line-styles"
  "html/canvas/element/shadows"
)

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -s "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_dir_html() {
  local dir="$1"
  local names
  names=$(curl --fail --location --silent --show-error --retry 3 --connect-timeout 8 --max-time 30 \
    "${API_ROOT}/${dir}" | grep -o '"name": "[^"]*"')
  while IFS= read -r line; do
    local name="${line#\"name\": \"}"
    name="${name%\"}"
    # 只取主线程 .html（排除 .worker.js / .any.js 变体与 manual/、目录）
    case "${name}" in
      *.html) fetch_raw "${dir}/${name}" ;;
    esac
  done <<< "${names}"
}

fetch_raw "resources/testharness.js"
fetch_raw "html/canvas/resources/canvas-tests.js"
fetch_raw "html/canvas/resources/canvas-tests.css"
# 用例引用的测试图片（drawImage 驱动用例）
fetch_raw "images/clear-100x50.png"
fetch_raw "images/green-100x50.png"
fetch_raw "images/red.png"

for dir in "${SUBDIRS[@]}"; do
  fetch_dir_html "${dir}"
done

echo "Canvas testharness subset ready (${#SUBDIRS[@]} dirs, WPT ${WPT_REV})"
