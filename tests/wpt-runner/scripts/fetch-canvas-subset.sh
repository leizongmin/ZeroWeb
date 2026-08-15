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
# R34xx（G6）：OffscreenCanvas worker 变体目录面（html/canvas/offscreen/*）
# R34xx（2026-08-15 第二批导入）：补全范围内子目录——drawing-images/path-objects/
# reset/conformance-requirements/canvas-context/canvas-host/color-type/filters/
# layers/global-hdr-headroom/wide-gamut-canvas（manual 交互面与 video 媒体面不在
# 目标范围）。顶层文件（2d.conformance.requirements.*、2d.putImageData、
# 2d.text-outside-of-the-flat-tree 等）单独 fetch_raw。
OFFSCREEN_SUBDIRS=(
  "html/canvas/offscreen/the-canvas-state"
  "html/canvas/offscreen/drawing-rectangles-to-the-canvas"
  "html/canvas/offscreen/transformations"
  "html/canvas/offscreen/pixel-manipulation"
  "html/canvas/offscreen/line-styles"
  "html/canvas/offscreen/shadows"
  "html/canvas/offscreen/compositing"
  "html/canvas/offscreen/fill-and-stroke-styles"
  "html/canvas/offscreen/text"
  "html/canvas/offscreen/conformance-requirements"
  "html/canvas/offscreen/drawing-images-to-the-canvas"
  "html/canvas/offscreen/path-objects"
  "html/canvas/offscreen/reset"
  "html/canvas/offscreen/canvas-context"
  "html/canvas/offscreen/canvas-host"
  "html/canvas/offscreen/color-type"
  "html/canvas/offscreen/filters"
  "html/canvas/offscreen/layers"
  "html/canvas/offscreen/wide-gamut-canvas"
)
SUBDIRS=(
  "html/canvas/element/the-canvas-state"
  "html/canvas/element/drawing-rectangles-to-the-canvas"
  "html/canvas/element/transformations"
  "html/canvas/element/pixel-manipulation"
  "html/canvas/element/line-styles"
  "html/canvas/element/shadows"
  "html/canvas/element/compositing"
  "html/canvas/element/fill-and-stroke-styles"
  "html/canvas/element/text"
  "html/canvas/element/conformance-requirements"
  "html/canvas/element/drawing-images-to-the-canvas"
  "html/canvas/element/path-objects"
  "html/canvas/element/reset"
  "html/canvas/element/canvas-context"
  "html/canvas/element/canvas-host"
  "html/canvas/element/color-type"
  "html/canvas/element/filters"
  "html/canvas/element/layers"
  "html/canvas/element/global-hdr-headroom"
  "html/canvas/element/wide-gamut-canvas"
)
# R34xx（2026-08-15）：element 顶层 testharness 用例（conformance-requirements
# 顶层文件 + putImageData + text-outside-of-the-flat-tree——后者为 reftest 对，
# 一并抓取供 reftest/oracle 面）。
CANVAS_TOP_FILES=(
  "html/canvas/element/2d.conformance.requirements.basics.html"
  "html/canvas/element/2d.conformance.requirements.delete.html"
  "html/canvas/element/2d.conformance.requirements.drawings.html"
  "html/canvas/element/2d.conformance.requirements.missingargs.html"
  "html/canvas/element/2d.putImageData.html"
  "html/canvas/element/2d.putImageData-ref.html"
  "html/canvas/element/2d.text-outside-of-the-flat-tree.html"
  "html/canvas/element/2d.text-outside-of-the-flat-tree-ref.html"
  "html/canvas/offscreen/2d.conformance.requirements.basics.html"
  "html/canvas/offscreen/2d.conformance.requirements.basics.worker.js"
  "html/canvas/offscreen/2d.conformance.requirements.missingargs.html"
  "html/canvas/offscreen/2d.conformance.requirements.missingargs.worker.js"
  "html/canvas/offscreen/OffscreenCanvas-ctx-font-sibling-index-invalid.tentative.html"
  "html/canvas/offscreen/set-proprietary-font-names-001-crash.html"
)

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  # 404/网络失败容忍（部分引用资源上游不存在——如 CanvasTest-low-ascent.ttf）：
  # 失败不阻断（set -e 下 return 0 继续后续抓取）。
  if ! curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT}/${relative}" -o "${temporary}" 2>/dev/null; then
    rm -f "${temporary}"
    return 0
  fi
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
    # 主线程 .html + OffscreenCanvas .worker.js 变体（G6——fetch_tests_from_worker 聚合）
    case "${name}" in
      *.html|*.worker.js) fetch_raw "${dir}/${name}" ;;
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
fetch_raw "images/yellow.png"
fetch_raw "images/yellow75.png"

# text 目录像素用例的 @font-face 测试字体（2d.text.draw.*——canvas 文本像素光栅）
fetch_raw "fonts/CanvasTest.ttf"
fetch_raw "fonts/CanvasTest-ascent256.ttf"
fetch_raw "fonts/CanvasTest-descent0.ttf"

# R34xx：variationSelectors 的 @font-face 变体字体（css/css-fonts/resources/vs/——
# 外链 variation-sequences.css 引用；色/单色 emoji + cmap14 对照族）。
fetch_raw "css/css-fonts/resources/vs/NotoEmoji-Regular_subset.ttf"
fetch_raw "css/css-fonts/resources/vs/NotoColorEmoji-Regular_subset.ttf"
fetch_raw "css/css-fonts/resources/vs/NotoEmoji-Regular_without-cmap14-subset.ttf"
fetch_raw "css/css-fonts/resources/vs/NotoSansJP-Regular_with-cmap14-subset.ttf"
fetch_raw "css/css-fonts/resources/vs/MPLUS1-Regular_without-cmap14-subset.ttf"
fetch_raw "css/css-fonts/resources/vs/STIXTwoMath-Regular_with-cmap14-subset.ttf"
fetch_raw "css/css-fonts/resources/vs/NotoSansMath-Regular_without-cmap14-subset.ttf"

for dir in "${SUBDIRS[@]}"; do
  fetch_dir_html "${dir}"
done

# R34xx（G6）：offscreen worker 变体（.worker.js——fetch_tests_from_worker 聚合执行）
for dir in "${OFFSCREEN_SUBDIRS[@]}"; do
  fetch_dir_html "${dir}"
done

# R34xx（2026-08-15）：element/offscreen 顶层 testharness 用例。
for f in "${CANVAS_TOP_FILES[@]}"; do
  fetch_raw "${f}"
done

echo "Canvas testharness subset ready (${#SUBDIRS[@]} dirs, WPT ${WPT_REV})"
