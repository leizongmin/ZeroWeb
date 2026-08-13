#!/usr/bin/env bash
# Fetch a pinned subset of the upstream WPT dom/ tests (testharness style)
# used by the JS/DOM nativization goal (docs/goal/js-dom.md, M4 / DC-3).
#
# Strategy: first batch = main-thread .html cases of dom/nodes/ (the core
# Node/Element/Document surface — directly exercises native DOM bindings vs
# polyfill). Subdirectories (events/, collections/, lists/, ranges/,
# traversal/) are appended per M4 slices as the baseline grows.
#
# 与 fetch-canvas-subset.sh 同构（同 WPT_REV pin + GitHub API 列目录 + raw 拉单文件）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"

# 第一批：dom/nodes/ 核心 Node/Element/Document 主线程 .html 用例。
# 后续 M4 切片按需追加 "dom/events" / "dom/collections" / "dom/lists" /
# "dom/ranges" / "dom/traversal"。
SUBDIRS=(
  "dom/nodes"
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
    # 拉主线程 .html + .js 依赖（用例常引用同目录 .js 测试体，如 Document-createElementNS.js）。
    # 排除 .worker.js / .any.js 变体（需 dedicated worker / 不同 harness）。
    case "${name}" in
      *.html | *.js) fetch_raw "${dir}/${name}" ;;
    esac
  done <<< "${names}"
}

# testharness.js / testharnessreport.js 为所有 dom 用例共享（canvas fetch 已拉 testharness.js 则跳过）。
# testharnessreport.js：runner 内联替换为空，但仍需文件存在（prepare_harness_html 替换 src）。
fetch_raw "resources/testharness.js"
fetch_raw "resources/testharnessreport.js"
# dom 根共享 JS（dom/nodes 用例引用 ../constants.js / ../common.js）。
fetch_raw "dom/constants.js"
fetch_raw "dom/common.js"

for dir in "${SUBDIRS[@]}"; do
  fetch_dir_html "${dir}"
done

echo "DOM testharness subset ready (${#SUBDIRS[@]} dirs, WPT ${WPT_REV})"
