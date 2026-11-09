#!/usr/bin/env bash
# Goal: svg-compat-v2（编号 44——推进顺序见同目录 README.md；契约
# docs/goal/archive/svg-compat-v2.md（2026-11-08 模式 A 归档；2026-11-07
# 收口），DC-1 语料续拉对齐 pin + 差集清单 + missing=0）。
#
# 与 80 号脚本的差异——sync 补差模式：一期收口（2026-10-22）后 svg/ 语料经
# 10-27 局部重拉后残缺（本地 1,444 vs 一期入账 2,144，缺 ~700 文件），且
# fetch_dir_deep 的幂等快路径（已含 .html 即跳过）对「目录在但文件缺」形态
# 整体漏拉。本脚本照 41 号先例改走 pin 树清单逐文件对账：git trees recursive
# 列全量 blob → 本地缺失即拉、blob hash 不符即定向重拉（旧文件在替换成功前
# 不删——语料不可再生不变式 #31；learnings 2026-10-13 先例）。
#
# 用法：bash tests/wpt-runner/scripts/goals/44-svg-compat-v2.sh
#   SYNC_PAR=6（并行拉取 worker 数，缺省 6；CDN 友好值）
#   GITHUB_TOKEN=...（trees 列目录认证；未认证 60 req/h 易限流）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

# 滚动 pin（svg-compat-v2 M3，2026-11-07）：本 goal 域内 pin 前滚至 upstream
# master 快照。lib.sh 共享 pin 未动——22 脚本共用、一次前滚即全 goal 域语料
# 事件，越出本 goal 范围；其余 goal 后续各自按先例滚动。RAW_ROOT 与 WPT_REV
# 同步重推（lib.sh source 时嵌入 rev）。
WPT_REV="28c152b74f8679534e558745fe83312c8dcca8c8"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

GOAL="svg-compat-v2"

DIRS=(
  # svg 全树（一期 80 号同面——testharness DOM 面 + 渲染断言面语料共用，
  # skip 规则两侧同源在 runner 侧 src/testharness_svg.rs）
  "svg"
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

# -- 批量 blob sha 对账（python 内联 git blob hash，41 号同源）---------------
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

echo "== svg-v2 语料 sync（pin ${WPT_REV:0:9}，并行 ${SYNC_PAR}，${#DIRS[@]} 目录）=="
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT
SYNC_OK=0
SYNC_FAIL=0
for dir in "${DIRS[@]}"; do
  # 首段 SHA 从根树下钻（fetch_dir_deep 同构）。
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
#（fetch_dir_deep 的 "${dir}/${name}" 拼接同语义）。
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
  # 存量文件 blob sha 对账（python 内联 blob hash，41 号先例头注）。
  report=$(sync_hash_report "${WPT_DATA}" "${WORK_DIR}/filtered.txt" \
    "${WORK_DIR}/mismatch.txt")
  mismatch_n=${report##* }
  if [[ "${mismatch_n}" -gt 0 ]]; then
    while IFS= read -r relpath; do
      sync_replace_one "${relpath}" || true
    done < "${WORK_DIR}/mismatch.txt"
  fi
  # missing=0 对账：上游扩展面应全部在本地且 hash 与 pin 一致（替换后复算）。
  if [[ "${mismatch_n}" -gt 0 ]]; then
    report=$(sync_hash_report "${WPT_DATA}" "${WORK_DIR}/filtered.txt" \
      "${WORK_DIR}/mismatch.final.txt")
    missing=$(( ${report%% *} + ${report##* } ))
  else
    missing=${report%% *}
  fi
  # 差集清单落 goal evidence 目录（missing/re-mismatch 两清单逐行 wpt-data 相对路径）。
  if [[ -n "${GOAL_DIFF_DIR:-}" && -d "${GOAL_DIFF_DIR}" ]]; then
    cp "${WORK_DIR}/absent.txt" "${GOAL_DIFF_DIR}/m0-missing-pre.txt" 2>/dev/null || true
    cp "${WORK_DIR}/mismatch.txt" "${GOAL_DIFF_DIR}/m0-hash-mismatch-pre.txt" 2>/dev/null || true
  fi
  printf '  %-28s 上游 %-5d 新拉 %-5d 重拉 %-3d missing %s\n' \
    "${dir}" "${total}" "${new_n}" "${mismatch_n}" "${missing}"
  if [[ "${missing}" == "0" ]]; then
    SYNC_OK=$((SYNC_OK + 1))
  else
    SYNC_FAIL=$((SYNC_FAIL + 1))
  fi
done

# SMIL testharness 包装器（svg/animations 182 案引用，80 号先例；sync 模式
# 目录限定拉不到 wpt-data 根外资源，fetch_raw 幂等单文件补）。
fetch_raw "resources/SVGAnimationTestCase-testharness.js" \
  || echo "  ⚠ resources/SVGAnimationTestCase-testharness.js 拉取失败（重跑脚本幂等续拉）" >&2
# /common/rendering-utils.js——svg-v2 M0 补拉（一期「渲染域不补」决策在本 goal
# 解除）：waitForAtLeastOneFrame 是 SMIL 采样断言案（smil-values-* 等）的
# 等帧前置，缺失即 ReferenceError 假失败；本体属 SMIL 时基面语料依赖。
fetch_raw "common/rendering-utils.js" \
  || echo "  ⚠ common/rendering-utils.js 拉取失败（重跑脚本幂等续拉）" >&2

echo "== sync 结果：${SYNC_OK} 目录 missing=0，${SYNC_FAIL} 目录有缺失（重跑续拉）=="
if [[ "${SYNC_FAIL}" -ne 0 ]]; then
  exit 1
fi
