#!/usr/bin/env bash
# Fetch the pinned upstream corpus subset for the wptfyi-dom goal DC-2
# sanitizer/trusted-types 面（docs/goal/wptfyi-dom/master.md 优先序第三簇）.
#
# 与 fetch-gap-batch1-subset.sh 同一 WPT_REV pin + sparse blob:none 浅克隆
# （逐文件 raw + contents API 会在百文件量级超时/触发 GitHub 限额——batch1
# 先例）。wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# 覆盖两 corpus（DC-1 缺口三分法：sanitizer-api 3,834st / trusted-types
# 3,764st，三缺＝语料+通道+API）：
# - sanitizer-api — 全树（含 support/）。
# - trusted-types — 全树（dedicated-tests/ support/ 等一并落盘；runner 侧
#   子目录清单与 skip 谓词按内容规则排除 iframe/worker/非 testharness 形态，
#   fetch 清单与 runner 清单同域双保险，security-csp 先例）。
#
# resources/ 与 common/ 共享 harness 资产由 gap-batch1/security-csp 先批落盘，
# 本脚本幂等哨兵不依赖它们；FORCE=1 重拉。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"

# 幂等：两 corpus 首批标记（各取一个稳定文件）已就位且非空即跳过。
S_SENTINEL="${WPT_DATA}/sanitizer-api/sanitizer-basic-filtering.html"
T_SENTINEL="${WPT_DATA}/trusted-types/DOMParser-parseFromString.html"
if [[ -f "${S_SENTINEL}" && -f "${T_SENTINEL}" && "${FORCE:-0}" != "1" ]]; then
  echo "sanitizer/tt corpus subset already present (WPT ${WPT_REV}); FORCE=1 to refetch"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

# sparse blob:none 浅克隆：单网络往返取两 corpus + 共享 harness 资产。
git init -q "${TMP}/wpt"
git -C "${TMP}/wpt" remote add origin https://github.com/web-platform-tests/wpt.git
git -C "${TMP}/wpt" sparse-checkout set --no-cone \
  "sanitizer-api/**" \
  "trusted-types/**" \
  "resources/testharness.js" \
  "resources/testharnessreport.js" \
  "common/dispatcher/**" \
  "common/security-features/**"
git -C "${TMP}/wpt" fetch --filter=blob:none --depth 1 --no-tags origin "${WPT_REV}" 2>&1 | tail -1
checkout_ok=0
for attempt in 1 2 3 4 5; do
  if git -C "${TMP}/wpt" checkout --detach FETCH_HEAD 2>&1 | tail -1; then
    checkout_ok=1
    break
  fi
  echo "checkout attempt ${attempt} failed; retrying" >&2
  sleep 5
done
if [[ "${checkout_ok}" != "1" ]]; then
  echo "checkout failed after retries" >&2
  exit 1
fi

mkdir -p "${WPT_DATA}"
for corpus in sanitizer-api trusted-types; do
  if [[ -d "${TMP}/wpt/${corpus}" ]]; then
    rm -rf "${WPT_DATA:?}/${corpus}"
    cp -R "${TMP}/wpt/${corpus}" "${WPT_DATA}/"
    echo "copied ${corpus}"
  else
    echo "MISSING corpus ${corpus} at ${WPT_REV}" >&2
    exit 1
  fi
done
# 共享资产补齐（已存在则等值覆盖，同 pin 无漂移）。
for asset in resources/testharness.js resources/testharnessreport.js; do
  if [[ -f "${TMP}/wpt/${asset}" ]]; then
    mkdir -p "${WPT_DATA}/$(dirname "${asset}")"
    cp "${TMP}/wpt/${asset}" "${WPT_DATA}/${asset}"
  fi
done

echo "sanitizer/tt corpus subset fetched (WPT ${WPT_REV})"
