#!/usr/bin/env bash
# goals/lib.sh — goal 推进脚本共享库（WPT_REV pin + raw/API fetch + 语料盘点）。
# 与 fetch-dom-subset.sh 同构（同 pin + GitHub Contents API 列目录 + raw 拉单文件）；
# 抽成库避免 goals/NN-*.sh 逐份复制。wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# **不直接执行**——由同目录 NN-*.sh source（REPO_ROOT 由本库自行推导）。
# 子目录扩充：fetch_dir_html 只列目录 top-level（GitHub 未认证 API 60 req/h 速率
# 限制，递归易触限）；大域（fetch/ svg/ 等）子目录按各 goal M1 rally 轮照
# fetch-dom-subset.sh SUBDIRS 先例追加。

WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
# REPO_ROOT 由本库自行推导（lib 位于 tests/wpt-runner/scripts/goals/，4 层到仓库根），
# 不依赖 source 方传入——goals 脚本曾因少算一层把语料拉到 tests/tests/ 误入库。
LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${LIB_DIR}/../../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    echo "fetch failed: ${relative}" >&2
    return 1
  fi
  # 合法空文件放行（设计为 0 字节的测试形态；curl --fail 已保证非 2xx 报错）。
  test -e "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_dir_html() {
  local dir="$1" depth="${2:-0}"
  local local_dir="${WPT_DATA}/${dir}"
  # 幂等快路径：WPT_REV 固定 → 文件集稳定；已拉过且未 FORCE=1 则跳过 API 列目录
  # （避免未认证 60 req/h 限流 403 阻断）。注意：仅当目录已含 .html 才跳过——
  # 用例全在子目录的域（如 web-animations/）每次仍会重列，属预期。
  if [[ "${FORCE:-0}" != "1" && -d "${local_dir}" ]]; then
    if compgen -G "${local_dir}/*.html" > /dev/null; then
      echo "  ${dir}: 已含 .html 用例，跳过 API 列目录（FORCE=1 可强制重列）"
      return 0
    fi
  fi
  local listing
  # ?ref=<pin>：列目录与 raw 拉取同 rev——不加 ref 会列默认分支 HEAD，
  # HEAD 与 pin 漂移时列出 pin 中不存在的文件 → 404。
  # python3 解析拿 type：子目录递归一层（depth<1），防 API 限流不深挖。
  # 列目录失败（403 限流/网络）优雅降级：跳过该目录不中止，重跑幂等续拉。
  if ! listing=$(curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 30 \
    "${API_ROOT}/${dir}?ref=${WPT_REV}" \
    | python3 -c 'import json,sys
for entry in json.load(sys.stdin):
    print(entry["type"], entry["name"])' 2>/dev/null); then
    echo "  ${dir}: API 列目录失败（限流/网络）——跳过，稍后重跑续拉" >&2
    return 0
  fi
  local type name
  while read -r type name; do
    [[ -z "${name}" ]] && continue
    case "${type}" in
      dir)
        if [[ "${depth}" -lt 1 ]]; then
          fetch_dir_html "${dir}/${name}" $((depth + 1))
        fi
        ;;
      file)
        # 拉 .html 用例 + .js 依赖；排除 .worker.js / .any.js 变体（需 dedicated
        # worker / wrapper harness，runner 形态支持由各 goal M1 rally 轮评估）。
        case "${name}" in
          *.worker.js | *.any.js) ;;
          *.html | *.js | *.xml | *.xhtml | *.svg)
            fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
            ;;
        esac
        ;;
    esac
  done <<< "${listing}"
}

FAILED_FETCHES=0

goals_fetch_all() {
  echo "== fetch 语料（pin ${WPT_REV:0:9}，FORCE=${FORCE:-0}）=="
  local dir
  for dir in "${DIRS[@]}"; do
    fetch_dir_html "${dir}"
  done
  if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
    echo "  ⚠ ${FAILED_FETCHES} 个文件拉取失败（瞬时错误不计入中止；重跑脚本幂等续拉）" >&2
  fi
}

goals_inventory() {
  echo "== 语料盘点（wpt-data 本地 .html，含子目录）=="
  local dir n
  for dir in "${DIRS[@]}"; do
    n=0
    if [[ -d "${WPT_DATA}/${dir}" ]]; then
      n=$(find "${WPT_DATA}/${dir}" -name '*.html' 2>/dev/null | wc -l)
    fi
    printf '  %-28s %5d 案\n' "${dir}" "${n}"
  done
}

goals_next_steps() {
  local goal="$1"
  cat <<EOF
== M1 下一步（rally 轮，契约见 docs/goal/${goal}.md DC-1）==
  1. runner 通道：zero-wpt-runner 新增 testharness 子命令 + per-dir skip 规则
  2. Makefile：fetch-wpt / testharness target（test-guard 包裹，照 testharness-fs 先例）
  3. 基线：分类通过率（文本 + JSON）落 docs/goal/${goal}/evidence/
  4. 回填：docs/compat/trends/wpt-suites.csv planned 行转数据行（evidence 列给来源）
EOF
}
