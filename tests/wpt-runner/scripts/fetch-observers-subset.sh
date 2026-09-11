#!/usr/bin/env bash
# Fetch the pinned IntersectionObserver / ResizeObserver testharness subset used
# by the event-loop-spec goal (docs/goal/event-loop-spec.md, M1 / DC-1).
#
# 与 fetch-indexeddb-subset.sh 同构（同 WPT_REV pin + raw 拉单文件，显式清单）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# 拉取域 = 两目录 top-level 全量 .html + resources/ 全目录。**用例筛选不在 fetch
# 侧**：runner（testharness.rs `observers_case_skipped`）按内容规则排除——名字
# 规则会漏判（document-scrolling-element-root.html 名字无 iframe 但内容用 iframe）：
# - *-ref.html / *-notref.html（reftest 参照页）
# - source 含 <iframe（runner 无 iframe 文档/几何管道）
# - intersection-observer/v2/（IO v2 trackVisibility 范围外，不拉取——双保险同样
#   写进 runner skip 规则）
# idlharness.window.js / .window.js 包装形态不拉取（runner 无 wrapper，与
# fetch-web-components-subset.sh 先例一致）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    echo "fetch failed: ${relative}" >&2
    return 1
  fi
  test -e "${temporary}"
  mv "${temporary}" "${target}"
}

SHARED=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
)

IO_CASES=(
  "intersection-observer/animating.html"
  "intersection-observer/bounding-box.html"
  "intersection-observer/callback-cross-realm-report-exception.html"
  "intersection-observer/client-rect.html"
  "intersection-observer/clip-path-animation.html"
  "intersection-observer/clip-path.html"
  "intersection-observer/containing-block.html"
  "intersection-observer/cross-document-root.html"
  "intersection-observer/cross-origin-iframe.sub.html"
  "intersection-observer/cross-origin-tall-iframe-rendering-ref.html"
  "intersection-observer/cross-origin-tall-iframe-rendering.sub.html"
  "intersection-observer/cross-origin-tall-iframe.sub.html"
  "intersection-observer/disconnect.html"
  "intersection-observer/display-none.html"
  "intersection-observer/document-scrolling-element-root.html"
  "intersection-observer/edge-inclusive-intersection.html"
  "intersection-observer/empty-root-margin.html"
  "intersection-observer/explicit-root-different-document.html"
  "intersection-observer/fixed-position-child-scroll.html"
  "intersection-observer/fixed-position-iframe-scroll.html"
  "intersection-observer/fixed-position-scroll.html"
  "intersection-observer/grow-height-and-scrolled.html"
  "intersection-observer/iframe-no-root-with-wrapping-scroller.html"
  "intersection-observer/iframe-no-root.html"
  "intersection-observer/iframe-root-with-overflow-propagation.html"
  "intersection-observer/initial-observation-with-threshold.html"
  "intersection-observer/inline-client-rect.html"
  "intersection-observer/inline-with-block-child-client-rect.html"
  "intersection-observer/intersection-ratio-ib-split.html"
  "intersection-observer/intersection-ratio-with-fractional-bounds-2.html"
  "intersection-observer/intersection-ratio-with-fractional-bounds-in-iframe.html"
  "intersection-observer/intersection-ratio-with-fractional-bounds.html"
  "intersection-observer/isIntersecting-change-events.html"
  "intersection-observer/isIntersecting-threshold.html"
  "intersection-observer/multicol-2.html"
  "intersection-observer/multicol.html"
  "intersection-observer/multiple-targets.html"
  "intersection-observer/multiple-thresholds.html"
  "intersection-observer/nested-cross-origin-iframe.sub.html"
  "intersection-observer/not-in-containing-block-chain.html"
  "intersection-observer/observer-attributes.html"
  "intersection-observer/observer-callback-arguments.html"
  "intersection-observer/observer-exceptions.html"
  "intersection-observer/observer-in-iframe.html"
  "intersection-observer/observer-without-js-reference.html"
  "intersection-observer/padding-clip.html"
  "intersection-observer/reinsert-element.html"
  "intersection-observer/remove-element.html"
  "intersection-observer/root-is-table-with-overflow-scroll.html"
  "intersection-observer/root-margin-in-same-origin-iframe.html"
  "intersection-observer/root-margin-root-element.html"
  "intersection-observer/root-margin-rounding.html"
  "intersection-observer/root-margin.html"
  "intersection-observer/root-vertical-rl.html"
  "intersection-observer/rtl-clipped-root.html"
  "intersection-observer/same-document-no-root.html"
  "intersection-observer/same-document-root.html"
  "intersection-observer/same-document-with-document-root.html"
  "intersection-observer/same-document-zero-size-target.html"
  "intersection-observer/same-origin-grand-child-iframe.sub.html"
  "intersection-observer/scroll-and-root-margin.html"
  "intersection-observer/scroll-margin-4-val.html"
  "intersection-observer/scroll-margin-clip-path.html"
  "intersection-observer/scroll-margin-dynamic.html"
  "intersection-observer/scroll-margin-horizontal.html"
  "intersection-observer/scroll-margin-iframe.html"
  "intersection-observer/scroll-margin-nested-2.html"
  "intersection-observer/scroll-margin-nested-3.html"
  "intersection-observer/scroll-margin-nested.html"
  "intersection-observer/scroll-margin-no-intersect.html"
  "intersection-observer/scroll-margin-non-scrolling-root.html"
  "intersection-observer/scroll-margin-not-contained.html"
  "intersection-observer/scroll-margin-percent.html"
  "intersection-observer/scroll-margin-propagation.html"
  "intersection-observer/scroll-margin-with-border-outline.html"
  "intersection-observer/scroll-margin-zero.html"
  "intersection-observer/scroll-margin.html"
  "intersection-observer/shadow-content.html"
  "intersection-observer/svg-clipped-rect-target.html"
  "intersection-observer/svg-container-element.html"
  "intersection-observer/svg-group-target.html"
  "intersection-observer/svg-image.html"
  "intersection-observer/svg-intersection-with-fractional-bounds-2.html"
  "intersection-observer/svg-intersection-with-fractional-bounds.html"
  "intersection-observer/svg-rect-target.html"
  "intersection-observer/svg-stroke-change.html"
  "intersection-observer/svg-target-changes-position.html"
  "intersection-observer/svg-transformed-rect-target.html"
  "intersection-observer/svg-viewbox.html"
  "intersection-observer/target-in-detached-document.html"
  "intersection-observer/target-in-different-window.html"
  "intersection-observer/target-is-root.html"
  "intersection-observer/text-target.html"
  "intersection-observer/timestamp.html"
  "intersection-observer/transform-animation.html"
  "intersection-observer/transformed-iframe-001-cross-origin.html"
  "intersection-observer/transformed-iframe-001-same-origin.html"
  "intersection-observer/transformed-iframe-002-cross-origin.html"
  "intersection-observer/transformed-iframe-002-same-origin.html"
  "intersection-observer/transformed-iframe-003-cross-origin.html"
  "intersection-observer/transformed-iframe-003-same-origin.html"
  "intersection-observer/transformed-iframe-004-cross-origin.html"
  "intersection-observer/transformed-iframe-004-same-origin.html"
  "intersection-observer/transformed-non-invertible-iframe-cross-origin-crash.sub.html"
  "intersection-observer/transformed-non-invertible-iframe-same-origin-crash.html"
  "intersection-observer/unclipped-root.html"
  "intersection-observer/visibility-hidden.html"
  "intersection-observer/zero-area-element-hidden.html"
  "intersection-observer/zero-area-element-visible.html"
  "intersection-observer/zoom-scaled-target.html"
)

IO_RESOURCES=(
  "intersection-observer/resources/cross-origin-child-iframe.sub.html"
  "intersection-observer/resources/cross-origin-subframe.html"
  "intersection-observer/resources/cross-origin-tall-subframe.sub.html"
  "intersection-observer/resources/iframe-body-with-overflow-propagation.html"
  "intersection-observer/resources/iframe-no-root-subframe.html"
  "intersection-observer/resources/intersection-observer-test-utils.js"
  "intersection-observer/resources/intersection-ratio-with-fractional-bounds-in-iframe-content.html"
  "intersection-observer/resources/nested-cross-origin-child-iframe.sub.html"
  "intersection-observer/resources/nested-cross-origin-grand-child-iframe.html"
  "intersection-observer/resources/observer-in-iframe-subframe.html"
  "intersection-observer/resources/root-margin-in-same-origin-iframe-subframe.html"
  "intersection-observer/resources/same-origin-grand-child-iframe.html"
  "intersection-observer/resources/scaled-target-subframe.html"
  "intersection-observer/resources/scroll-margin-propagation-iframe-1.html"
  "intersection-observer/resources/scroll-margin-propagation-iframe-2.html"
  "intersection-observer/resources/scroll-margin-propagation-iframe-3.html"
  "intersection-observer/resources/timestamp-subframe.html"
  "intersection-observer/resources/transformed-iframe-subframe.html"
  "intersection-observer/resources/v2-midframe.sub.html"
  "intersection-observer/resources/v2-subframe.html"
)

RO_CASES=(
  "resize-observer/calculate-depth-for-node.html"
  "resize-observer/callback-cross-realm-report-exception.html"
  "resize-observer/change-layout-in-error.html"
  "resize-observer/devicepixel-ref.html"
  "resize-observer/devicepixel.html"
  "resize-observer/devicepixel2-ref.html"
  "resize-observer/devicepixel2.html"
  "resize-observer/eventloop.html"
  "resize-observer/fragments.html"
  "resize-observer/iframe-same-origin-ref.html"
  "resize-observer/iframe-same-origin.html"
  "resize-observer/multiple-observers-with-mutation-crash.html"
  "resize-observer/notify.html"
  "resize-observer/observe-001.html"
  "resize-observer/observe-002.html"
  "resize-observer/observe-003.html"
  "resize-observer/observe-004.html"
  "resize-observer/observe-005.html"
  "resize-observer/observe-006.html"
  "resize-observer/observe-007.html"
  "resize-observer/observe-008.html"
  "resize-observer/observe-009.html"
  "resize-observer/observe-010.html"
  "resize-observer/observe-011.html"
  "resize-observer/observe-012.html"
  "resize-observer/observe-013.html"
  "resize-observer/observe-014.html"
  "resize-observer/observe-015.html"
  "resize-observer/observe-016.html"
  "resize-observer/observe-017.html"
  "resize-observer/observe-018.html"
  "resize-observer/observe-019.html"
  "resize-observer/observe-020.html"
  "resize-observer/observer-in-cross-origin-frame.sub.html"
  "resize-observer/ordering.html"
  "resize-observer/scrollbars-2.html"
  "resize-observer/scrollbars.html"
  "resize-observer/svg-with-css-box-001.html"
  "resize-observer/svg.html"
  "resize-observer/zoom.html"
)

RO_RESOURCES=(
  "resize-observer/resources/cross-origin-subframe.html"
  "resize-observer/resources/iframe.html"
  "resize-observer/resources/image.png"
  "resize-observer/resources/resizeTestHelper.js"
)

for rel in "${SHARED[@]}" "${IO_CASES[@]}" "${IO_RESOURCES[@]}" "${RO_CASES[@]}" "${RO_RESOURCES[@]}"; do
  fetch_raw "$rel"
done

echo "observers subset ready: ${#IO_CASES[@]} IO + ${#RO_CASES[@]} RO cases + ${#IO_RESOURCES[@]}+${#RO_RESOURCES[@]} resources @ ${WPT_REV:0:12}"
