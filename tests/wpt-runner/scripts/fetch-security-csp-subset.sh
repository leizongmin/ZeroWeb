#!/usr/bin/env bash
# Fetch the pinned upstream security corpora subset used by the security-hardening
# goal (docs/goal/security-hardening.md, M1 / DC-1).
#
# 与 fetch-observers-subset.sh 同一 WPT_REV pin，但枚举/取数走 **sparse blob:none
# 浅克隆**而非逐文件 raw 拉：本三 corpus 首批即 400+ 文件，逐文件 raw + contents
# API 枚举两处都超时/触发 GitHub 匿名限额（实测 R1：API 枚举 10 分钟仅落 75 文件；
# sparse 克隆 checkout 11 秒落全量）。wpt-data 整体 gitignored，用例按需 fetch、
# 不入库。
#
# 覆盖三 corpus（DC-1）：
# - content-security-policy — 目标覆盖范围指令目录（default-src/script-src/style-src/
#   img-src/connect-src/frame-src/font-src/media-src/object-src/base-uri/form-action/
#   frame-ancestors）+ 违规报告面（securitypolicyviolation）+ meta 解析面（meta）+
#   generic 矩阵 + gen/top.meta 生成面 + blob/child-src。**第二批（R7 扩批）新增**：
#   inheritance/navigation/sandbox/unsafe-eval/wasm-unsafe-eval。**仍不拉**：
#   inside-worker/reporting*/embedded-enforcement/nonce-hiding/plugin-types/
#   resource-hints/svg/webrtc/xslt（worker 执行面 / HTTP server 依赖 / 范围外指令）
#   ——后续按簇扩批，master.md 记账。
# - mixed-content — 顶层可执行面（blob.https.sub.html / imageset.https.sub.html）+
#   resources/。gen/ 为 window.js/iframe 包装形态（runner 无 wrapper，照 observers
#   先例不拉取），tentative/ 后续按簇评估。
# - secure-contexts — 顶层文件 + support/（多数用例 iframe/worker 依赖，runner 侧
#   内容规则筛减）。
#
# 用例筛选不在 fetch 侧：runner（testharness.rs `security_case_skipped`）按内容规则
# 排除——*(-ref|-notref).html 参照页、source 含 <iframe / Worker( / SharedWorker( /
# navigator.serviceWorker（runner 无 iframe 文档与 worker 执行面）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"

# 幂等：首批标记目录已就位（非空）即跳过，FORCE=1 重拉。
SENTINEL="${WPT_DATA}/content-security-policy/securitypolicyviolation"
if [[ -d "${SENTINEL}" && -n "$(ls -A "${SENTINEL}" 2>/dev/null)" && "${FORCE:-0}" != "1" ]]; then
  echo "Security corpora subset already present (WPT ${WPT_REV}); FORCE=1 to refetch"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

# sparse blob:none 浅克隆：一次网络往返取三 corpus + 共享 harness 资产。
# 直接 fetch pin rev（不做 HEAD 克隆再二次 fetch——两次全量树下载，实测 10 分钟
# 超时；init + fetch 单往返约省一半）。
git init -q "${TMP}/wpt"
git -C "${TMP}/wpt" remote add origin https://github.com/web-platform-tests/wpt.git
git -C "${TMP}/wpt" sparse-checkout set --no-cone \
  "content-security-policy/securitypolicyviolation/**" \
  "content-security-policy/generic/**" \
  "content-security-policy/meta/**" \
  "content-security-policy/default-src/**" \
  "content-security-policy/script-src/**" \
  "content-security-policy/style-src/**" \
  "content-security-policy/img-src/**" \
  "content-security-policy/connect-src/**" \
  "content-security-policy/frame-src/**" \
  "content-security-policy/font-src/**" \
  "content-security-policy/media-src/**" \
  "content-security-policy/object-src/**" \
  "content-security-policy/base-uri/**" \
  "content-security-policy/form-action/**" \
  "content-security-policy/frame-ancestors/**" \
  "content-security-policy/blob/**" \
  "content-security-policy/child-src/**" \
  "content-security-policy/inheritance/**" \
  "content-security-policy/navigation/**" \
  "content-security-policy/sandbox/**" \
  "content-security-policy/unsafe-eval/**" \
  "content-security-policy/wasm-unsafe-eval/**" \
  "content-security-policy/gen/top.meta/**" \
  "content-security-policy/resources/**" \
  "content-security-policy/support/**" \
  "mixed-content/blob.https.sub.html" \
  "mixed-content/imageset.https.sub.html" \
  "mixed-content/resources/**" \
  "secure-contexts/**" \
  "resources/testharness.js" \
  "resources/testharnessreport.js"
git -C "${TMP}/wpt" fetch --filter=blob:none --depth 1 --no-tags origin "${WPT_REV}" 2>&1 | tail -1
# checkout 触发懒 blob 补取（promisor 批量）：代理抖动会单批失败中断——已落 blob 留在
# 本地，重试只补缺，不重头下载。
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
cp -R "${TMP}/wpt/content-security-policy" "${WPT_DATA}/"
cp -R "${TMP}/wpt/mixed-content" "${WPT_DATA}/"
cp -R "${TMP}/wpt/secure-contexts" "${WPT_DATA}/"
mkdir -p "${WPT_DATA}/resources"
cp "${TMP}/wpt/resources/testharness.js" "${TMP}/wpt/resources/testharnessreport.js" \
  "${WPT_DATA}/resources/"

echo "Security corpora subset ready (WPT ${WPT_REV})"
