#!/usr/bin/env bash
# Fetch the pinned Fullscreen API testharness subset used by the web-api-batch2 goal
# (docs/goal/web-api-batch2.md, M1 / DC-1).
#
# 与 fetch-observers-subset.sh 同构（同 WPT_REV pin + raw 拉单文件，显式清单）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# 拉取域 = fullscreen/api/ + fullscreen/model/ + fullscreen/crashtests/ 全量 .html +
# 两处 helper（trusted-click.js、api/resources/ 页）。rendering/ 不拉取——`:fullscreen`
# 伪类与全屏 UA 渲染样式面属 rendering-compat 流域（goal 排除 + 跨域记账）。
# **用例筛选不在 fetch 侧**（双保险照 observers 先例）：runner（testharness.rs
# `fullscreen_case_skipped`）按内容规则排除——
# - *-manual（无——fullscreen 无 manual 案，规则同域防新增）
# - source 含 <iframe（runner 无 iframe 文档/几何管道——fullscreen 的 nested/
#   allowfullscreen/cross-origin 深依赖 iframe 语义，跨域记账，重入条件 = iframe 管道）
# - fullscreen/rendering/、fullscreen/api/resources/（helper 资产/参照页，不按案跑）
# - api/*.window.js（keyboard-lock 4 案——.window.js 包装形态不拉取，Keyboard API
#   依赖面，与 observers 先例一致）
# 基线预期：requestFullscreen/exitFullscreen 面未实现，绝大多数案 Fail——基线只记通过率。

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

HELPERS=(
  "fullscreen/trusted-click.js"
  "fullscreen/api/resources/attempt-fullscreen.html"
  "fullscreen/api/resources/echo-fullscreenEnabled.html"
  "fullscreen/api/resources/keyboard-lock-inner.sub.html"
  "fullscreen/api/resources/navigate.sub.html"
  "fullscreen/api/resources/recursive-iframe-fullscreen.html"
  "fullscreen/api/resources/report-fullscreen-enabled.html"
)

API_CASES=(
  "fullscreen/api/document-exit-fullscreen-active-document.html"
  "fullscreen/api/document-exit-fullscreen-nested-in-direct-fullscreen-iframe.html"
  "fullscreen/api/document-exit-fullscreen-nested-in-iframe.html"
  "fullscreen/api/document-exit-fullscreen-nested-shadow-dom.html"
  "fullscreen/api/document-exit-fullscreen-nested.html"
  "fullscreen/api/document-exit-fullscreen-timing.html"
  "fullscreen/api/document-exit-fullscreen-twice.html"
  "fullscreen/api/document-exit-fullscreen.html"
  "fullscreen/api/document-fullscreen-element.html"
  "fullscreen/api/document-fullscreen-enabled-active-document.html"
  "fullscreen/api/document-fullscreen-enabled-cross-origin.sub.html"
  "fullscreen/api/document-fullscreen-enabled-removing-allowfullscreen.sub.html"
  "fullscreen/api/document-fullscreen-enabled-setting-allowfullscreen-timing.sub.html"
  "fullscreen/api/document-fullscreen-enabled-setting-allowfullscreen.sub.html"
  "fullscreen/api/document-fullscreen-enabled.html"
  "fullscreen/api/document-onfullscreenchange.html"
  "fullscreen/api/document-onfullscreenerror.html"
  "fullscreen/api/element-ready-allowed.html"
  "fullscreen/api/element-ready-check-allowed-cross-origin.sub.html"
  "fullscreen/api/element-ready-check-containing-iframe.html"
  "fullscreen/api/element-ready-check-fullscreen-element-sibling.html"
  "fullscreen/api/element-ready-check-fullscreen-iframe-child.html"
  "fullscreen/api/element-ready-check-iframe-child.html"
  "fullscreen/api/element-ready-check-not-allowed-cross-origin.sub.html"
  "fullscreen/api/element-ready-check-not-in-document.html"
  "fullscreen/api/element-request-fullscreen-active-document.html"
  "fullscreen/api/element-request-fullscreen-after-error.html"
  "fullscreen/api/element-request-fullscreen-and-exit-iframe.html"
  "fullscreen/api/element-request-fullscreen-and-move-to-iframe.html"
  "fullscreen/api/element-request-fullscreen-and-move.html"
  "fullscreen/api/element-request-fullscreen-and-remove-iframe.html"
  "fullscreen/api/element-request-fullscreen-and-remove.html"
  "fullscreen/api/element-request-fullscreen-consume-user-activation.html"
  "fullscreen/api/element-request-fullscreen-cross-origin.sub.html"
  "fullscreen/api/element-request-fullscreen-dialog.html"
  "fullscreen/api/element-request-fullscreen-namespaces.html"
  "fullscreen/api/element-request-fullscreen-non-top.html"
  "fullscreen/api/element-request-fullscreen-not-allowed.html"
  "fullscreen/api/element-request-fullscreen-options.html"
  "fullscreen/api/element-request-fullscreen-options.tentative.https.html"
  "fullscreen/api/element-request-fullscreen-same-element.html"
  "fullscreen/api/element-request-fullscreen-screen-size.https.html"
  "fullscreen/api/element-request-fullscreen-svg-rect.html"
  "fullscreen/api/element-request-fullscreen-svg-svg.html"
  "fullscreen/api/element-request-fullscreen-svg-text.html"
  "fullscreen/api/element-request-fullscreen-timing.html"
  "fullscreen/api/element-request-fullscreen-top.html"
  "fullscreen/api/element-request-fullscreen-twice.html"
  "fullscreen/api/element-request-fullscreen-two-elements.html"
  "fullscreen/api/element-request-fullscreen-without-user-activation.tentative.https.html"
  "fullscreen/api/element-request-fullscreen.html"
  "fullscreen/api/fullscreen-display-contents.html"
  "fullscreen/api/fullscreen-reordering.html"
  "fullscreen/api/historical.html"
  "fullscreen/api/keyboard-lock-cross-origin-iframe.sub.html"
  "fullscreen/api/navigate-iframe.sub.html"
  "fullscreen/api/permission.tentative.https.html"
  "fullscreen/api/promises-reject.html"
  "fullscreen/api/promises-resolve.html"
  "fullscreen/api/shadowroot-fullscreen-element.html"
)

MODEL_CASES=(
  "fullscreen/model/move-fullscreen-element.html"
  "fullscreen/model/move-to-fullscreen-iframe.html"
  "fullscreen/model/move-to-iframe.html"
  "fullscreen/model/move-to-inactive-document.html"
  "fullscreen/model/remove-child.html"
  "fullscreen/model/remove-first-sibling.html"
  "fullscreen/model/remove-first.html"
  "fullscreen/model/remove-last-sibling.html"
  "fullscreen/model/remove-last.html"
  "fullscreen/model/remove-parent.html"
  "fullscreen/model/remove-single.html"
)

CRASH_CASES=(
  "fullscreen/crashtests/backdrop-list-item.html"
  "fullscreen/crashtests/chrome-1312699.html"
  "fullscreen/crashtests/content-visibility-2-crash.html"
  "fullscreen/crashtests/content-visibility-crash.html"
  "fullscreen/crashtests/frameset-crash.html"
)

for rel in "${SHARED[@]}" "${HELPERS[@]}" "${API_CASES[@]}" "${MODEL_CASES[@]}" "${CRASH_CASES[@]}"; do
  fetch_raw "$rel"
done

echo "fullscreen subset ready: ${#API_CASES[@]} api + ${#MODEL_CASES[@]} model + ${#CRASH_CASES[@]} crashtests cases + ${#HELPERS[@]} helpers @ ${WPT_REV:0:12}"
