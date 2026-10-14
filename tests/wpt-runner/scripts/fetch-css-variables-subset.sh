#!/usr/bin/env bash
# R5114（site-optimizer P9-9 切片 2）：css-variables testharness 语料的 harness 资源。
# 用例本体由 fetch-wpt-data 全量拉取（wpt-data/css/css-variables）——本脚本只补
# testharness 运行面所需的公共资源（数据仓为 reftest 口径，不含 resources/）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

FILES=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
)

for relative in "${FILES[@]}"; do
  target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    continue
  fi
  mkdir -p "$(dirname "${target}")"
  temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 3 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -s "${temporary}"
  mv "${temporary}" "${target}"
done

echo "css-variables harness resources ready (${#FILES[@]} files, WPT ${WPT_REV})"
