#!/usr/bin/env bash
# css-variables testharness 语料 fetch（css-testharness-compat M2 / DC-3 扩展；
# R5114 起点）：
# 1. harness 公共资源（数据仓为 reftest 口径，不含 resources/）；
# 2. css/css-variables 用例树深挖补拉（goals/lib.sh fetch_dir_deep，同 pin）——
#    精选仓数据近全（2026-10-30 实测仅缺 2 用例页 + 5 helper 资产），上游滚动
#    增量与 CI 全新环境靠这里补齐。目录幂等：已含 .html 跳过 API 列目录
#    （FORCE=1 强制重列重拉；GITHUB_TOKEN 认证列目录）。

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

# 用例树深挖补拉（35 号 goal 脚本同款机制：git trees recursive 清单一次拿全量，
# 文件本体走 jsdelivr/raw 不吃 API 配额）。skip 规则两侧同源：runner 谓词
# css_testharness_case_skipped 见 testharness_css_corpus.rs。
source "${SCRIPT_DIR}/goals/lib.sh"
GOAL_FETCH_DEEP=1
fetch_dir_deep css/css-variables
if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
  echo "css-variables tree fetch incomplete: ${FAILED_FETCHES} files failed" >&2
  exit 1
fi
