#!/usr/bin/env bash
# Goal: webcrypto-compat-v3（编号 38——推进顺序见同目录 README.md）
# 三期（零遗留终局）：二期 DC-2 转正甄别定谳的新目录族收敛 + 谱系零遗留
# （10 域 109 案/18,052 子测试 + 散新案 5 案 + worker 腿，D1 用户拍板
# 2026-11-01，同日二次拍板扩围）。
# M1 资产预置：HEAD 布局全子树新增式拉取（两新族 + 散新案）；基线运行
# （runner 通道 + per-dir skip 规则）= M1 rally 轮落地，契约见
# docs/goal/webcrypto-compat-v3.md DC-1。
# 注意：lib.sh WPT_REV pin（3159769…）早于上游转正重排——本脚本锁二期
# DC-2 查证、2026-11-01 复证零漂移的子树（commit cb6cc97a…，子树
# 084495e783…），全子树逐文件新增式取数：本地已有文件（pin 语料）经
# fetch_raw 幂等跳过零变动（一期 33,109 分母锚保持），只补 HEAD 布局新增。
# 用法：bash tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

# 三期语料锁转正子树（二期 DC-2 甄别 + 2026-11-01 复证零漂移）。lib.sh 的
# 取数变量均为调用时引用，此处覆盖即生效，不改共享 lib.sh、不影响其他 goal。
WPT_REV="cb6cc97ada2772b2afe668b9fd5cb39a576a5347"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

GOAL="webcrypto-compat-v3"

# fetch_dir_deep 的幂等快路径是目录级（本地已含用例即整目录跳过）——对
# WebCryptoAPI 旧目录会漏掉 HEAD 布局新增的散在文件（散新案 5 案散在旧目录）。
# 三期改走全子树文件级新增式清点：git trees 递归列全量 blob，逐文件 fetch_raw
# （已存在即幂等跳过，仅新增落盘）；扩展名分流与 lib.sh fetch_dir_deep 同规则。
FAILED_FETCHES=0
fetch_subtree_additive() {
  local dir="$1"
  local root_sha listing
  root_sha=$(curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 30 \
    "${GH_API_AUTH[@]}" \
    "${GIT_TREES_ROOT}/${WPT_REV}" \
    | python3 -c 'import json,sys
want = sys.argv[1]
for entry in json.load(sys.stdin)["tree"]:
    if entry["path"] == want:
        print(entry["sha"])
        break' "${dir}") || {
    echo "  ${dir}: git trees 根树下钻失败（限流/网络）——跳过，稍后重跑续拉" >&2
    return 0
  }
  if [[ -z "${root_sha}" ]]; then
    echo "  ${dir}: 上游无「${dir}」目录——跳过" >&2
    return 0
  fi
  if ! listing=$(curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 60 \
    "${GH_API_AUTH[@]}" \
    "${GIT_TREES_ROOT}/${root_sha}?recursive=1" \
    | python3 -c 'import json,sys
d = json.load(sys.stdin)
if d.get("truncated"):
    print("TRUNCATED", file=sys.stderr)
for entry in d["tree"]:
    if entry["type"] == "blob":
        print(entry["path"])'); then
    echo "  ${dir}: git trees 递归清单失败（限流/网络）——跳过，稍后重跑续拉" >&2
    return 0
  fi
  local path name
  while read -r path; do
    [[ -z "${path}" ]] && continue
    name="${path##*/}"
    case "${name}" in
      # .worker.js（in-worker harness 形态）默认排除——crypto 面无 .worker.js
      # 文件（2026-11-01 子树清点），worker 腿经 .any.js worker 变体走
      # runner 接线（入口文档 M-worker），不在本脚本取数差异面。
      *.worker.js)
        if [[ "${GOAL_PULL_WORKER_JS:-0}" == "1" ]]; then
          fetch_raw "${dir}/${path}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      # .any.js 单文件含 window/worker 双变体源——必拉（GOAL_PULL_ANY_JS=1）。
      *.any.js)
        if [[ "${GOAL_PULL_ANY_JS:-0}" == "1" ]]; then
          fetch_raw "${dir}/${path}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      *.txt)
        if [[ "${GOAL_PULL_TXT:-0}" == "1" ]]; then
          fetch_raw "${dir}/${path}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      *.wasm)
        if [[ "${GOAL_PULL_WASM:-0}" == "1" ]]; then
          fetch_raw "${dir}/${path}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      *.html | *.htm | *.css | *.headers | *.js | *.json | *.xml | *.xhtml | *.svg)
        fetch_raw "${dir}/${path}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        if [[ "${GOAL_PULL_HEADERS_TARGETS:-0}" == "1" && "${name}" == *.headers ]]; then
          local header_target="${name%.headers}"
          if [[ -n "${header_target}" && "${header_target}" != *.* ]]; then
            fetch_raw "${dir}/${header_target}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
          fi
        fi
        ;;
      # META.yml / README.md / WEB_FEATURES.yml / tools/*.py 等非测试资产
      # 不匹配上述扩展名，自然排除（同 lib.sh 分流口径）。
    esac
  done <<< "${listing}"
}

GOAL_PULL_ANY_JS=1

echo "== fetch 语料（转正子树 ${WPT_REV:0:9}，全子树新增式，FORCE=${FORCE:-0}）=="
fetch_subtree_additive "WebCryptoAPI"
if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
  echo "  ⚠ ${FAILED_FETCHES} 个文件拉取失败（瞬时错误不计入中止；重跑脚本幂等续拉）" >&2
fi
# 跨域 /common 依赖照 26 号补拉（幂等：pin 轮已在库即跳过）。
fetch_raw "common/subset-tests.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "common/gc.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

# 语料盘点（全子树 .js/.html 计数 + 新族/散新案在位核对——.any.js 是本域
# 主形态，goals_inventory 的 .html 口径不适用，改定制盘点）。
echo "== 语料盘点（wpt-data 本地，WebCryptoAPI 子树）=="
total=$(find "${WPT_DATA}/WebCryptoAPI" -type f \( -name '*.js' -o -name '*.html' -o -name '*.htm' \) 2>/dev/null | wc -l)
modern=$(find "${WPT_DATA}/WebCryptoAPI/modern-algos" -type f 2>/dev/null | wc -l)
secure=$(find "${WPT_DATA}/WebCryptoAPI/secure-curves" -type f 2>/dev/null | wc -l)
printf '  WebCryptoAPI 全子树      %5d 文件\n' "${total}"
printf '  modern-algos/            %5d 文件\n' "${modern}"
printf '  secure-curves/           %5d 文件\n' "${secure}"
for f in export-key-error-order.https.any.js \
         import_export/okp_fixed_length.https.any.js \
         import_export/okp_malformed_asn1.https.any.js \
         sharedarraybuffer.https.any.js \
         wrapKey_unwrapKey/aes_kw_unwrapKey_failures.https.any.js; do
  if [[ -s "${WPT_DATA}/WebCryptoAPI/${f}" ]]; then
    echo "  散新案在位: ${f}"
  else
    echo "  散新案缺位: ${f}（重跑续拉）" >&2
  fi
done

goals_next_steps "${GOAL}"
