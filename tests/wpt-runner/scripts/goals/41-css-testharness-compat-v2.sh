#!/usr/bin/env bash
# Goal: css-testharness-compat-v2（编号 41——推进顺序见同目录 README.md；
# 契约 docs/goal/css-testharness-compat-v2.md，DC-1 语料对齐 pin + missing=0）。
#
# v2 M0 批次目录语料预置：44 个未导入目录（M0 甄别批次 B1 值/序列化簇 24 +
# B2 规则/级联簇 7 + B3 混合布局簇 13——逐目录甄别表见 goal evidence
# 2026-11-04-m0-triage-and-gate.md）+ css/support 共享 helper。
#
# 与 35 号脚本的差异——sync 补差模式：v2 池 24 个目录本地已含一期 rendering
# 拉取的 reftest 口径存量页，fetch_dir_deep 的幂等快路径（已含 .html 即跳过）
# 会整体漏拉 testharness 新增面。本脚本改走 pin 树清单逐文件对账：git trees
# recursive 列全量 blob → 本地缺失即拉、blob hash 不符即定向重拉（旧文件在
# 替换成功前不删——语料不可再生不变式 #31；learnings 2026-10-13 先例）。
# 空目录（cssom/filter-effects/css-syntax 等）走同一 sync 路径（全量缺失即
# 全量首拉），不区分两种形态。
#
# .xht 不在拉取扩展内（同 fetch_dir_deep）：XML 文档执行路径缺位（runner 只收
# .html/.htm），CSS2 10.5k xht 大山为挂账面，见甄别表。
#
# 用法：bash tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh
#   SYNC_PAR=6（并行拉取 worker 数，缺省 6；CDN 友好值）
#   GITHUB_TOKEN=...（trees 列目录认证；未认证 60 req/h 不够 37 目录）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="css-testharness-compat-v2"

DIRS=(
  # B1 值/属性解析序列化簇（M0 甄别：引擎声明值/序列化强域，24 目录 23,882 子测试）
  "css/cssom"
  "css/css-images"
  "css/filter-effects"
  "css/css-ui"
  "css/css-borders"
  "css/css-logical"
  "css/css-text-decor"
  "css/css-box"
  "css/css-pseudo"
  "css/css-display"
  "css/css-mixins"
  "css/css-syntax"
  "css/fill-stroke"
  "css/css-shadow"
  "css/css-size-adjust"
  "css/css-content"
  "css/css-will-change"
  "css/css-color-adjust"
  "css/css-easing"
  "css/compositing"
  "css/css-counter-styles"
  "css/css-nesting"
  "css/css-color-hdr"
  "css/css-env"
  # B2 规则/条件/级联簇（7 目录 ~7.5k 子测试；CSS2 按 html 面计 ~300）
  "css/css-conditional"
  "css/mediaqueries"
  "css/css-cascade"
  "css/CSS2"
  "css/css-forms"
  "css/css-link-params"
  "css/css-font-loading"
  # B3 混合布局簇（13 目录 ~19.3k 子测试；布局探测面通过率低——M0 数值门按 2% 计，
  # 布局探测地址池 > 10k 触发独立 layout goal 分档，见甄别表）
  "css/css-gaps"
  "css/css-flexbox"
  "css/css-multicol"
  "css/css-position"
  "css/css-overflow"
  "css/css-tables"
  "css/css-lists"
  "css/css-inline"
  "css/css-scroll-snap"
  "css/css-viewport"
  "css/css-break"
  "css/css-contain"
  "css/css-writing-modes"
  # css 面共享 helper 目录（35 号先例：parsing/computed/interpolation-testcommon.js）
  "css/support"
)

SYNC_PAR="${SYNC_PAR:-6}"
export WPT_DATA WPT_REV RAW_ROOT

# -- sync 拉取 worker（与 lib.sh fetch_raw 同源双源回退：jsdelivr CDN 优先，
#    raw.githubusercontent 权威回落；tmp + mv 原子落盘；已存在即跳过）-----
sync_fetch_one() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error \
    --retry 2 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@${WPT_REV}/${relative}" \
    -o "${temporary}" \
    && ! curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    echo "sync fetch failed: ${relative}" >&2
    return 1
  fi
  mv "${temporary}" "${target}"
}

# -- mismatch 定向重拉（目标替换成功前旧文件保留——#31）---------------------
sync_replace_one() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  local replacement="${target}.newsync"
  mkdir -p "$(dirname "${replacement}")"
  if ! curl --fail --location --silent --show-error \
    --retry 2 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@${WPT_REV}/${relative}" \
    -o "${replacement}" \
    && ! curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${replacement}"; then
    rm -f "${replacement}"
    echo "sync replace failed（旧文件保留）: ${relative}" >&2
    return 1
  fi
  mv "${replacement}" "${target}"
}

# -- 上游清单 → 本地对账分派（单进程 python；产出缺失/存量清单文件）--------
# 入参：wpt-data 根、filtered 清单文件（逐行 "sha path"）、absent/present 输出。
sync_dispatch() {
  python3 - "$1" "$2" "$3" "$4" << 'PYEOF'
import os
import sys

wpt_data, filtered_path, absent_path, present_path = sys.argv[1:5]
absent, present = [], []
with open(filtered_path) as handle:
    for line in handle:
        parts = line.split(None, 1)
        if len(parts) < 2:
            continue
        path = parts[1].strip()
        if os.path.isfile(os.path.join(wpt_data, path)):
            present.append(path)
        else:
            absent.append(path)
with open(absent_path, "w") as handle:
    handle.write("\n".join(absent) + ("\n" if absent else ""))
with open(present_path, "w") as handle:
    handle.write("\n".join(present) + ("\n" if present else ""))
print(f"{len(absent)} {len(present)}")
PYEOF
}

# -- 批量 blob sha 对账（python 内联 git blob hash）-------------------------
# git hash-object --stdin-paths 对 stdin 相对路径按 repo 根解析（不前置 cwd
# 前缀，实测 2.53 fatal）——改用 blob sha1("blob <len>\0" + content) 内联计算，
# 单进程零子进程、无路径解析与输出对齐问题。上游 trees API 的 blob sha 同式。
# 入参：wpt-data 根、filtered 清单（逐行 "sha path"）、mismatch 输出文件；
# 输出两行："缺失数 哈希不符数"。
sync_hash_report() {
  python3 - "$1" "$2" "$3" << 'PYEOF'
import hashlib
import os
import sys

wpt_data, filtered_path, mismatch_path = sys.argv[1:4]
absent = mismatched = 0
mismatches = []
with open(filtered_path) as handle, open(mismatch_path, "w") as out:
    for line in handle:
        parts = line.split(None, 1)
        if len(parts) < 2:
            continue
        sha, path = parts[0], parts[1].strip()
        full = os.path.join(wpt_data, path)
        if not os.path.isfile(full):
            absent += 1
            continue
        with open(full, "rb") as blob:
            data = blob.read()
        digest = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if digest != sha:
            mismatched += 1
            mismatches.append(path)
    out.write("\n".join(mismatches) + ("\n" if mismatches else ""))
print(f"{absent} {mismatched}")
PYEOF
}

export -f sync_fetch_one sync_replace_one

echo "== v2 语料 sync（pin ${WPT_REV:0:9}，并行 ${SYNC_PAR}，${#DIRS[@]} 目录）=="
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT
SYNC_OK=0
SYNC_FAIL=0
for dir in "${DIRS[@]}"; do
  # 首段 SHA 从根树下钻（fetch_dir_deep 同构）；css 树逐目录复用查询。
  tree_sha=""
  query_sha="${WPT_REV}"
  ok=1
  IFS='/' read -r -a segments <<< "${dir}"
  for seg in "${segments[@]}"; do
    if ! tree_sha=$(curl --fail --location --silent --show-error --retry 3 \
      --connect-timeout 8 --max-time 30 \
      "${GH_API_AUTH[@]}" \
      "${GIT_TREES_ROOT}/${query_sha}" \
      | python3 -c 'import json,sys
want = sys.argv[1]
for entry in json.load(sys.stdin)["tree"]:
    if entry["path"] == want:
        print(entry["sha"])
        break' "${seg}"); then
      echo "  ${dir}: git trees 列「${seg}」失败（限流/网络）——跳过，稍后重跑续拉" >&2
      ok=0
      break
    fi
    if [[ -z "${tree_sha}" ]]; then
      echo "  ${dir}: 上游无「${seg}」目录——跳过" >&2
      ok=0
      break
    fi
    query_sha="${tree_sha}"
  done
  if [[ "${ok}" != "1" ]]; then
    SYNC_FAIL=$((SYNC_FAIL + 1))
    continue
  fi
  if ! curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 120 \
    "${GH_API_AUTH[@]}" \
    "${GIT_TREES_ROOT}/${query_sha}?recursive=1" \
    | python3 -c 'import json,sys
d = json.load(sys.stdin)
if d.get("truncated"):
    print("TRUNCATED", file=sys.stderr)
for entry in d["tree"]:
    if entry["type"] == "blob":
        print(entry["sha"], entry["path"])' \
    | python3 -c 'import sys
# 子树 recursive 清单的 path 相对子树根——补 dir 前缀成 wpt-data 相对路径
#（fetch_dir_deep 的 "${dir}/${name}" 拼接同语义）；漏拼会 404 且错位落盘。
prefix = sys.argv[1]
for line in sys.stdin:
    parts = line.split(None, 1)
    if len(parts) < 2:
        continue
    path = parts[1].strip()
    if path.endswith((".html", ".htm", ".css", ".headers", ".js", ".json", ".xml", ".xhtml", ".svg")):
        sys.stdout.write(f"{parts[0]} {prefix}/{path}\n")' "${dir}" \
    > "${WORK_DIR}/filtered.txt"; then
    echo "  ${dir}: 递归清单/分流失败——跳过，稍后重跑续拉" >&2
    SYNC_FAIL=$((SYNC_FAIL + 1))
    continue
  fi
  total=$(wc -l < "${WORK_DIR}/filtered.txt")
  dispatch=$(sync_dispatch "${WPT_DATA}" "${WORK_DIR}/filtered.txt" \
    "${WORK_DIR}/absent.txt" "${WORK_DIR}/present.txt")
  new_n=${dispatch%% *}
  if [[ "${new_n}" -gt 0 ]]; then
    # 拉取失败容忍（verify 步对账兜底），不中止整目录。
    xargs -P "${SYNC_PAR}" -I{} bash -c 'sync_fetch_one "$@"' _ {} \
      < "${WORK_DIR}/absent.txt" || true
  fi
  # 存量文件 blob sha 对账（python 内联 blob hash——git hash-object
  # --stdin-paths 对 stdin 相对路径按 repo 根解析的坑，见 sync_hash_report 头注）。
  report=$(sync_hash_report "${WPT_DATA}" "${WORK_DIR}/filtered.txt" \
    "${WORK_DIR}/mismatch.txt")
  mismatch_n=${report##* }
  if [[ "${mismatch_n}" -gt 0 ]]; then
    while IFS= read -r relpath; do
      sync_replace_one "${relpath}" || true
    done < "${WORK_DIR}/mismatch.txt"
  fi
  # missing=0 对账：上游扩展面应全部在本地且 hash 与 pin 一致（替换后复算）。
  # missing = 仍缺失 + 仍不符（重拉失败者留给下次重跑续拉）。
  if [[ "${mismatch_n}" -gt 0 ]]; then
    report=$(sync_hash_report "${WPT_DATA}" "${WORK_DIR}/filtered.txt" \
      "${WORK_DIR}/mismatch.final.txt")
    missing=$(( ${report%% *} + ${report##* } ))
  else
    missing=${report%% *}
  fi
  printf '  %-28s 上游 %-5d 新拉 %-5d 重拉 %-3d missing %s\n' \
    "${dir}" "${total}" "${new_n}" "${mismatch_n}" "${missing}"
  if [[ "${missing}" == "0" ]]; then
    SYNC_OK=$((SYNC_OK + 1))
  else
    SYNC_FAIL=$((SYNC_FAIL + 1))
  fi
done

fetch_raw "resources/check-layout-th.js" \
  || echo "  ⚠ resources/check-layout-th.js 拉取失败（重跑脚本幂等续拉）" >&2

echo "== sync 结果：${SYNC_OK} 目录 missing=0，${SYNC_FAIL} 目录有缺失（重跑续拉）=="
if [[ "${SYNC_FAIL}" -ne 0 ]]; then
  exit 1
fi
