#!/usr/bin/env bash
# Fetch the pinned WebAssembly (wasm/jsapi) testharness subset used by page-wasm.
#
# 页面 WASM goal（docs/goal/page-wasm.md）M1 / DC-1。window 环境可执行的真实上游用例，
# 标准（非 tentative）JS API 面：Module/Instance/Memory/Table/Global/compile/instantiate/
# validate/导出面/链接错误。
#
# skip 域（Support Envelope「不在范围内」/ fetch 侧不拉，与 testharness.rs WASM_CASES 双保险）：
# - esm-integration/（WASM ESM integration——goal 明确排除）
# - jspi/、js-string/（JS Promise Integration / js-string builtins 提案）
# - gc/（WASM GC 提案，Tier 3+）
# - exception/、tag/（wasm EH JS API——提案期接口面，内核 EH 后端支持不在 M1）
# - function/（WebAssembly.Function type reflection 提案，tentative）
# - functions/（incumbent realm html harness——依赖 iframe/window.open，runner 不支持）
# - memory/*.tentative（shared-EW / resizable-AB / type reflection 提案面）
# - idlharness.any.js（IDL 反射面，fs 子集同款先例）、proto-from-ctor-realm.html（realm harness）
# - wasm/spec/（引擎内核语义——内核正确性由 wasmtime/wasmi 上游保证，goal 明确 skip）

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"

FILES=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
  "wasm/jsapi/assertions.js"
  "wasm/jsapi/bad-imports.js"
  "wasm/jsapi/instanceTestFactory.js"
  "wasm/jsapi/wasm-module-builder.js"
  "wasm/jsapi/memory/assertions.js"
  "wasm/jsapi/table/assertions.js"
  "wasm/jsapi/interface.any.js"
  "wasm/jsapi/prototypes.any.js"
  "wasm/jsapi/constructor/compile.any.js"
  "wasm/jsapi/constructor/instantiate.any.js"
  "wasm/jsapi/constructor/instantiate-bad-imports.any.js"
  "wasm/jsapi/constructor/multi-value.any.js"
  "wasm/jsapi/constructor/toStringTag.any.js"
  "wasm/jsapi/constructor/validate.any.js"
  "wasm/jsapi/global/constructor.any.js"
  "wasm/jsapi/global/toString.any.js"
  "wasm/jsapi/global/value-get-set.any.js"
  "wasm/jsapi/global/valueOf.any.js"
  "wasm/jsapi/instance/constructor.any.js"
  "wasm/jsapi/instance/constructor-bad-imports.any.js"
  "wasm/jsapi/instance/constructor-caching.any.js"
  "wasm/jsapi/instance/exports.any.js"
  "wasm/jsapi/instance/toString.any.js"
  "wasm/jsapi/memory/buffer.any.js"
  "wasm/jsapi/memory/constructor.any.js"
  "wasm/jsapi/memory/grow.any.js"
  "wasm/jsapi/memory/toString.any.js"
  "wasm/jsapi/module/constructor.any.js"
  "wasm/jsapi/module/customSections.any.js"
  "wasm/jsapi/module/exports.any.js"
  "wasm/jsapi/module/imports.any.js"
  "wasm/jsapi/module/toString.any.js"
  "wasm/jsapi/table/constructor.any.js"
  "wasm/jsapi/table/get-set.any.js"
  "wasm/jsapi/table/grow.any.js"
  "wasm/jsapi/table/length.any.js"
  "wasm/jsapi/table/toString.any.js"
)

fetch_jsdelivr() {
  # jsdelivr CDN（rev 锁定）——raw.githubusercontent.com 在部分网络下吞吐极慢/
  # 超时，jsdelivr 实测秒级；失败回落 raw（权威源）。
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error --connect-timeout 8 --max-time 60 \
    "https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@${WPT_REV}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    return 1
  fi
  test -s "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
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

fetch_one() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  fetch_jsdelivr "${relative}" || fetch_raw "${relative}"
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
  echo "WebAssembly jsapi testharness subset ready (31 cases, WPT ${WPT_REV})"
  exit 0
fi

failed=0
for file in "${FILES[@]}"; do
  if ! fetch_one "${file}"; then
    failed=1
    break
  fi
done

if [[ "${failed}" == "1" ]]; then
  echo "wasm subset fetch failed (set WPT_SOURCE to a pinned local checkout to retry)" >&2
  exit 1
fi

echo "WebAssembly jsapi testharness subset ready (31 cases, WPT ${WPT_REV})"
