#!/usr/bin/env bash
# Fetch the pinned upstream corpus subset used by the gap-batch1 goal
# (docs/goal/gap-batch1.md，覆盖率缺口小面批量补齐 batch1——十二小面通道).
#
# 与 fetch-security-csp-subset.sh 同一 WPT_REV pin + sparse blob:none 浅克隆
# （先例：本仓 11+ corpus 首批即百+ 文件，逐文件 raw + contents API 两处都
# 超时/触发 GitHub 限额；sparse 克隆单往返落全量）。wpt-data 整体 gitignored，
# 用例按需 fetch、不入库。
#
# 覆盖十二 corpus（对账表见 goal 文档）：
# 【CONFIRMED 六面——实现已在、只差通道（M1 五件套）】
# - quirks — 全树（reference/ crashtests/ 一并落盘；runner 侧 skip 规则只扫
#   testharness 形态 .html，reftest 面按 skips 排除）。
# - domparsing — 全树含 tentative/；.xhtml 由 runner 扩展名过滤排除（XML 变体
#   面记 runner 管道限制，goal evidence 对账）。
# - domxpath — 全树（顶层 35 .html + resources/）。
# - subresource-integrity — 全树（顶层 9 .html + integrity-policy/ signatures/
#   unencoded-digest/；.any.js 变体面 fetch 不拉 wrapper（timing 先例），
#   后续按簇扩批）。
# - cors — 全树含 .htm 主面（run_corpus_subdirs 扩展名变体消费）与 .py server
#   handler（runner 无动态 header 端点，相关面按内容规则排除）。
# - import-maps — 全树（data-driven/ not-overridden/ multiple-import-maps/
#   acquiring/ csp/ bare/ 各子目录 + resources/）。
# 【mixed-content 扩面（M2，DC-3）】顶层 *.html/*.headers/*.js + tentative/
# + generic/ + resources/——gen/ 为 window.js/iframe 包装形态不拉取（goal 划界
# 同域：gen/ 512 文件挂账 runner 基建，不进本 goal DC）。
# 【LIKELY 五面——M0 甄别语料基座 + M3 逐面入账】referrer-policy（generic/
# + css-integration/ + 4K* 变体 generic/ 镜像；gen/ 1,185 包装面不拉）、
# input-events、preload、wai-aria（manual/ 234 页一并落盘仅磁盘开销，runner
# 子目录清单不含 manual/）、storage。
#
# 用例筛选不在 fetch 侧：runner（testharness.rs 各面 case_skipped 谓词）按
# 内容规则排除——ref 页 / manual / 非 testharness 形态 / iframe / worker 面
# （fetch 清单与 runner 子目录清单同域双保险，security-csp 先例）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"

# 幂等：首批标记目录已就位（非空）即跳过，FORCE=1 重拉。
SENTINEL="${WPT_DATA}/domparsing/DOMParser-parseFromString-html.html"
if [[ -f "${SENTINEL}" && "${FORCE:-0}" != "1" ]]; then
  echo "gap-batch1 corpus subset already present (WPT ${WPT_REV}); FORCE=1 to refetch"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

# sparse blob:none 浅克隆：一次网络往返取十二 corpus + 共享 harness 资产
# （init + fetch pin rev 单往返，security-csp 先例）。checkout 触发懒 blob 补取
# （promisor 批量）：代理抖动会单批失败中断——重试只补缺，不重头下载。
git init -q "${TMP}/wpt"
git -C "${TMP}/wpt" remote add origin https://github.com/web-platform-tests/wpt.git
git -C "${TMP}/wpt" sparse-checkout set --no-cone \
  "quirks/**" \
  "domparsing/**" \
  "domxpath/**" \
  "subresource-integrity/**" \
  "cors/**" \
  "import-maps/**" \
  "mixed-content/*.html" \
  "mixed-content/*.headers" \
  "mixed-content/*.js" \
  "mixed-content/*.yml" \
  "mixed-content/*.md" \
  "mixed-content/generic/**" \
  "mixed-content/tentative/**" \
  "mixed-content/resources/**" \
  "referrer-policy/generic/**" \
  "referrer-policy/css-integration/**" \
  "referrer-policy/4K/generic/**" \
  "referrer-policy/4K+1/generic/**" \
  "referrer-policy/4K-1/generic/**" \
  "referrer-policy/*.json" \
  "referrer-policy/*.yml" \
  "input-events/**" \
  "preload/**" \
  "wai-aria/**" \
  "storage/**" \
  "resources/testharness.js" \
  "resources/testharnessreport.js" \
  "resources/sriharness.js" \
  "resources/check-layout-th.js" \
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
for corpus in quirks domparsing domxpath subresource-integrity cors import-maps \
              mixed-content referrer-policy input-events preload wai-aria storage; do
  if [[ -d "${TMP}/wpt/${corpus}" ]]; then
    cp -R "${TMP}/wpt/${corpus}" "${WPT_DATA}/"
  else
    echo "WARN: corpus ${corpus} missing from checkout" >&2
  fi
done
mkdir -p "${WPT_DATA}/resources"
cp "${TMP}/wpt/resources/testharness.js" "${TMP}/wpt/resources/testharnessreport.js" \
  "${TMP}/wpt/resources/sriharness.js" "${TMP}/wpt/resources/check-layout-th.js" \
  "${WPT_DATA}/resources/"
# SRI signatures/tentative 面的跨窗 helper（wpt-data common/ 既有 utils.js 等不覆盖）。
mkdir -p "${WPT_DATA}/common/dispatcher"
cp -R "${TMP}/wpt/common/dispatcher/." "${WPT_DATA}/common/dispatcher/"
# referrer-policy generic/css-integration 判据 helper（common.sub.js 42 引用）。
mkdir -p "${WPT_DATA}/common/security-features"
cp -R "${TMP}/wpt/common/security-features/." "${WPT_DATA}/common/security-features/"

echo "gap-batch1 corpus subset ready (WPT ${WPT_REV})"
