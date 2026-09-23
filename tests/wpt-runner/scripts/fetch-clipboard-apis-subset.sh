#!/usr/bin/env bash
# Fetch the pinned Clipboard API testharness subset used by the web-api-batch2 goal
# (docs/goal/web-api-batch2.md, M1 / DC-1).
#
# 与 fetch-observers-subset.sh 同构（同 WPT_REV pin + raw 拉单文件，显式清单）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# 拉取域 = clipboard-apis top-level window 用例 + events/permissions/text-write-read
# 子目录 + resources/ 资产。**用例筛选不在 fetch 侧**（iframe 形态双保险照 observers
# 先例）：runner（testharness.rs `clipboard_apis_case_skipped`）按内容规则排除——
# - *-manual.html（clipboard-file-manual / cut-event-manual / paste-event-manual，
#   需真实用户手势/物理剪贴板交互）
# - source 含 <iframe（runner 无 iframe 文档管道；detached-iframe/ 6 案同此）
# - detached-iframe/ / permissions-policy/（runner 不拉取——前者 iframe 面、后者需
#   Permissions-Policy HTTP 头/infra）
# - drag-multiple-urls.html（DnD 挂账面，goal 排除）
# - idlharness.https.window.js（.window.js 包装形态不拉取，与 observers 先例一致）
# 基线预期：navigator.clipboard 面未实现，绝大多数案 Fail——基线只记通过率。

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

TOP_CASES=(
  "clipboard-apis/async-clipboard-cached-getType-reject.tentative.https.html"
  "clipboard-apis/async-clipboard-read-unsanitized-null.https.html"
  "clipboard-apis/async-clipboard-stale-getType-reject.tentative.https.html"
  "clipboard-apis/async-custom-formats-write-fail.tentative.https.html"
  "clipboard-apis/async-custom-formats-write-read-web-prefix.tentative.https.html"
  "clipboard-apis/async-custom-formats-write-read-without-web-prefix.tentative.https.html"
  "clipboard-apis/async-custom-formats-write-read.tentative.https.html"
  "clipboard-apis/async-html-script-removal.https.html"
  "clipboard-apis/async-navigator-clipboard-basics.https.html"
  "clipboard-apis/async-navigator-clipboard-change-event.tentative.https.html"
  "clipboard-apis/async-navigator-clipboard-read-resource-load.https.html"
  "clipboard-apis/async-navigator-clipboard-read-sanitize.https.html"
  "clipboard-apis/async-navigator-clipboard-write-domstring.https.html"
  "clipboard-apis/async-navigator-clipboard-write-multiple.tentative.https.sub.html"
  "clipboard-apis/async-promise-write-blobs-read-blobs.https.html"
  "clipboard-apis/async-svg-read-write.tentative.https.html"
  "clipboard-apis/async-unsanitized-html-formats-write-read.tentative.https.html"
  "clipboard-apis/async-unsanitized-plaintext-formats-write-read.tentative.https.html"
  "clipboard-apis/async-unsanitized-standard-html-read-fail.tentative.https.html"
  "clipboard-apis/async-write-blobs-read-blobs.https.html"
  "clipboard-apis/async-write-html-read-html.https.html"
  "clipboard-apis/async-write-image-read-image.https.html"
  "clipboard-apis/clipboard-copy-selection-line-break.https.html"
  "clipboard-apis/clipboard-events-synthetic.html"
  "clipboard-apis/data-transfer-file-list-change-reference-updates.html"
  "clipboard-apis/dataTransfer-clearData.html"
)

SUBDIR_CASES=(
  "clipboard-apis/events/copy-event.html"
  "clipboard-apis/permissions/readText-denied.https.html"
  "clipboard-apis/permissions/readText-granted.https.html"
  "clipboard-apis/permissions/writeText-denied.https.html"
  "clipboard-apis/permissions/writeText-granted.https.html"
  "clipboard-apis/text-write-read/async-write-read.https.html"
  "clipboard-apis/text-write-read/async-write-readText.https.html"
  "clipboard-apis/text-write-read/async-writeText-read.https.html"
  "clipboard-apis/text-write-read/async-writeText-readText.https.html"
)

RESOURCES=(
  "clipboard-apis/resources/copied-file.txt"
  "clipboard-apis/resources/greenbox.png"
  "clipboard-apis/resources/page.html"
  "clipboard-apis/resources/user-activation.js"
)

for rel in "${SHARED[@]}" "${TOP_CASES[@]}" "${SUBDIR_CASES[@]}" "${RESOURCES[@]}"; do
  fetch_raw "$rel"
done

echo "clipboard-apis subset ready: ${#TOP_CASES[@]} top + ${#SUBDIR_CASES[@]} subdir cases + ${#RESOURCES[@]} resources @ ${WPT_REV:0:12}"
