#!/usr/bin/env bash
# Fetch the pinned File System (OPFS) testharness subset used by storage-opfs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"

# window 面可执行 OPFS 用例（spec https://fs.spec.whatwg.org/）：
# - 核心句柄面（M1 覆盖范围 1-2）：getDirectoryHandle/getFileHandle/iteration/removeEntry/
#   resolve/getFile/isSameEntry + writable 流全语义（write/close/abort/pipe）
# - 排除（Support Envelope「不在范围内」）：postMessage*（transferable 结构化克隆）、
#   create-sync-access-handle*（Worker 专用，M3 评估）、move（文件改名，页面级语义未实现）、
#   FileSystemObserver*（tentative 观察者 API）、IndexedDB/buckets（序列化/存储桶）、
#   opaque-origin（沙箱 iframe）、bfcache（页面往返）、idlharness（IDL 反射面）
FILES=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
  "common/gc.js"
  "fs/resources/test-helpers.js"
  "fs/resources/sandboxed-fs-test-helpers.js"
  "streams/resources/recording-streams.js"
  "fs/script-tests/FileSystemBaseHandle-isSameEntry.js"
  "fs/script-tests/FileSystemBaseHandle-getUniqueId.js"
  "fs/script-tests/FileSystemBaseHandle-remove.js"
  "fs/script-tests/FileSystemDirectoryHandle-getDirectoryHandle.js"
  "fs/script-tests/FileSystemDirectoryHandle-getFileHandle.js"
  "fs/script-tests/FileSystemDirectoryHandle-iteration.js"
  "fs/script-tests/FileSystemDirectoryHandle-removeEntry.js"
  "fs/script-tests/FileSystemDirectoryHandle-resolve.js"
  "fs/script-tests/FileSystemFileHandle-getFile.js"
  "fs/script-tests/FileSystemWritableFileStream.js"
  "fs/script-tests/FileSystemWritableFileStream-write.js"
  "fs/script-tests/FileSystemWritableFileStream-piped.js"
  "fs/root-name.https.any.js"
  "fs/FileSystemBaseHandle-isSameEntry.https.any.js"
  "fs/FileSystemBaseHandle-getUniqueId.https.any.js"
  "fs/FileSystemBaseHandle-remove.https.any.js"
  "fs/FileSystemDirectoryHandle-getDirectoryHandle.https.any.js"
  "fs/FileSystemDirectoryHandle-getFileHandle.https.any.js"
  "fs/FileSystemDirectoryHandle-iteration.https.any.js"
  "fs/FileSystemDirectoryHandle-removeEntry.https.any.js"
  "fs/FileSystemDirectoryHandle-resolve.https.any.js"
  "fs/FileSystemFileHandle-getFile.https.any.js"
  "fs/FileSystemWritableFileStream.https.any.js"
  "fs/FileSystemWritableFileStream-write.https.any.js"
  "fs/FileSystemWritableFileStream-piped.https.any.js"
)

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error --retry 2 --retry-all-errors \
    --continue-at - \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT:-https://raw.githubusercontent.com/web-platform-tests/wpt/}${WPT_REV}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    return 1
  fi
  test -s "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_from_checkout() {
  local checkout="${WPT_SOURCE:?WPT_SOURCE is required}"
  local revision
  revision="$(git -C "${checkout}" rev-parse HEAD)"
  if [[ "${revision}" != "${WPT_REV}" ]]; then
    echo "WPT_SOURCE revision ${revision} does not match pinned ${WPT_REV}" >&2
    return 1
  fi
  for relative in "${FILES[@]}"; do
    local source="${checkout}/${relative}"
    local target="${WPT_DATA}/${relative}"
    test -s "${source}"
    mkdir -p "$(dirname "${target}")"
    cp "${source}" "${target}"
  done
}

if [[ -n "${WPT_SOURCE:-}" ]]; then
  fetch_from_checkout
  echo "File System (OPFS) testharness subset ready (13 cases, WPT ${WPT_REV})"
  exit 0
fi

failed=0
for file in "${FILES[@]}"; do
  if ! fetch_raw "${file}"; then
    failed=1
    break
  fi
done

if [[ "${failed}" == "1" ]]; then
  echo "fs subset fetch failed (set WPT_SOURCE to a pinned local checkout to retry)" >&2
  exit 1
fi

echo "File System (OPFS) testharness subset ready (13 cases, WPT ${WPT_REV})"
