#!/usr/bin/env bash
# Fetch the pinned upstream WPT files used by the HTML interaction harness.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

FILES=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
  "common/subset-tests.js"
  # legacy-mb encode-form 页的 form 提交目标（encode-form-common.js
  # `form.action = "/common/blank.html"`，onload 后读 iframe location query
  # 断言序列化）。缺失时 CI fresh checkout 导航回路不完成——form 面全部
  # async_test 挂起到单案预算杀掉（合成 Timeout「completion callback was
  # not called」），run 41037174647 form 面 550k 子测试贡献 0 的根因。
  # 本地 goal 时代已拉过该文件，从不暴露。上游该文件为 0 字节合法空文档
  # （blank.html 上游 content-length: 0）——下载守卫用 -f 存在性检查，
  # -s 会拒收空文件。
  "common/blank.html"
  "html/semantics/forms/the-output-element/output.html"
  "html/semantics/forms/the-input-element/input-whitespace.html"
  "html/interaction/focus/sequential-focus-navigation-and-the-tabindex-attribute/focus-tabindex-default-value.html"
  "html/semantics/embedded-content/media-elements/networkState_initial.html"
  "html/semantics/embedded-content/media-elements/readyState_initial.html"
  "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/readyState.html"
  "uievents/constructors/inputevent-constructor.html"
)

for relative in "${FILES[@]}"; do
  target="${WPT_DATA}/${relative}"
  if [[ -f "${target}" && "${FORCE:-0}" != "1" ]]; then
    continue
  fi
  mkdir -p "$(dirname "${target}")"
  temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 3 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -f "${temporary}"
  mv "${temporary}" "${target}"
done

echo "HTML testharness subset ready (${#FILES[@]} files, WPT ${WPT_REV})"
