#!/usr/bin/env bash
# goals/lib.sh — goal 推进脚本共享库（WPT_REV pin + raw/API fetch + 语料盘点）。
# 与 fetch-dom-subset.sh 同构（同 pin + GitHub Contents API 列目录 + raw 拉单文件）；
# 抽成库避免 goals/NN-*.sh 逐份复制。wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# **不直接执行**——由同目录 NN-*.sh source（REPO_ROOT 由本库自行推导）。
# 子目录扩充：fetch_dir_html 只列目录 top-level（GitHub 未认证 API 60 req/h 速率
# 限制，递归易触限）；大域（fetch/ svg/ 等）子目录按各 goal M1 rally 轮照
# fetch-dom-subset.sh SUBDIRS 先例追加。
# 拉取扩展（2026-09-28，net-api-compat M1）：.json 数据依赖（mimesniff mime-types.json、
# url toascii.json 等 testharness 用例 fetch 的数据资产）与 .any.js window 变体
# opt-in（GOAL_PULL_ANY_JS=1）、GITHUB_TOKEN 认证列目录。

WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
# REPO_ROOT 由本库自行推导（lib 位于 tests/wpt-runner/scripts/goals/，4 层到仓库根），
# 不依赖 source 方传入——goals 脚本曾因少算一层把语料拉到 tests/tests/ 误入库。
LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${LIB_DIR}/../../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"
GIT_TREES_ROOT="https://api.github.com/repos/web-platform-tests/wpt/git/trees"
# GitHub API 认证（可选）：设 GITHUB_TOKEN 时列目录带 Bearer 头——未认证 60 req/h
# 对大域多目录 goal（net-api 六 corpus）一轮拉不完；认证 5000 req/h。raw 拉取
# （raw.githubusercontent.com）不走该配额，不受影响。
GH_API_AUTH=()
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
  GH_API_AUTH=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
fi

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  # jsdelivr CDN（同 pin rev 锁定）优先——raw.githubusercontent.com 在部分网络下
  # 吞吐极慢/超时（fetch-wasm-subset.sh 先例，jsdelivr 实测秒级）；失败回落 raw
  # （权威源）。
  if ! { curl --fail --location --silent --show-error \
    --retry 2 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@${WPT_REV}/${relative}" \
    -o "${temporary}" || curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; }; then
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
    "${GH_API_AUTH[@]}" \
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
        # 拉 .html 用例 + .js 依赖；.worker.js（in-worker harness 形态）默认排除，
        # goal 脚本设 GOAL_PULL_WORKER_JS=1 时拉取——runner 侧 fetch_tests_from_worker
        # 聚合管道（canvas worker 通道先例；workers-compat M2 worker harness 通道）。
        # .any.js 默认排除，goal 脚本设
        # GOAL_PULL_ANY_JS=1 时拉取——runner 侧 any_js_window_wrapper window 变体
        # 执行已支持（fs/wasm/timing 先例；streams/ 等以 .any.js 为唯一形态的域
        # 必须拉，否则语料为空）。
        case "${name}" in
          *.worker.js)
            if [[ "${GOAL_PULL_WORKER_JS:-0}" == "1" ]]; then
              fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
            fi
            ;;
          *.any.js)
            if [[ "${GOAL_PULL_ANY_JS:-0}" == "1" ]]; then
              fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
            fi
            ;;
          *.html | *.htm | *.js | *.json | *.xml | *.xhtml | *.svg | *.css)
            fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
            ;;
          # .headers sidecar（wptserve per-file 响应头——xmldecl support 的
          # *-vs-http 族断言 HTTP charset 头优先；runner fetch 通道读 sidecar
          # 覆盖 Content-Type。无害全量拉取——上游对 .html 同名 sidecar 语义）。
          # GOAL_PULL_HEADERS_TARGETS=1：sidecar 目标文件一并拉取（wptserve `foo` +
          # `foo.headers` MIME sidecar 语义——workers importScripts `1`/`null`/
          # `undefined` 无扩展名脚本面）；仅无扩展名目标生效（有扩展名目标已被
          # 扩展名分支拉取，避免双拉）。
          *.headers)
            fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
            if [[ "${GOAL_PULL_HEADERS_TARGETS:-0}" == "1" ]]; then
              local header_target="${name%.headers}"
              if [[ -n "${header_target}" && "${header_target}" != *.* ]]; then
                fetch_raw "${dir}/${header_target}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
              fi
            fi
            ;;
        esac
        ;;
    esac
  done <<< "${listing}"
}

FAILED_FETCHES=0

# 深挖拉取（goal 脚本设 GOAL_FETCH_DEEP=1 时 goals_fetch_all 改走本函数）。
# fetch_dir_html 只递归一层且逐目录打 Contents API（未认证 60 req/h）——语料埋在
# 二三层子目录的域（WebCryptoAPI/sign_verify/*.any.js 等）够不着也列不完。
# 本函数改用 git trees API：根树定位首段 + 子树 `recursive=1` 一次拿全量文件清单
#（目录 1 段 = 2 次 API 调用，与文件数无关），文件本体仍走 fetch_raw（jsdelivr
# CDN，不吃 API 配额）。扩展名分流与 fetch_dir_html 同规则。
fetch_dir_deep() {
  local dir="$1"
  local local_dir="${WPT_DATA}/${dir}"
  # 幂等快路径：目录已含 .any.js/.html 用例即视为已拉过（FORCE=1 重拉）。
  if [[ "${FORCE:-0}" != "1" && -d "${local_dir}" ]]; then
    if compgen -G "${local_dir}/*.any.js" > /dev/null || compgen -G "${local_dir}/*.html" > /dev/null; then
      echo "  ${dir}: 已含用例，跳过深挖（FORCE=1 可强制重拉）"
      return 0
    fi
  fi
  # 首段 SHA 从根树（commit sha 可直接作 tree-ish）拿；多段目录逐层下钻。
  local tree_sha segments seg query_sha
  IFS='/' read -r -a segments <<< "${dir}"
  query_sha="${WPT_REV}"
  tree_sha=""
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
      return 0
    fi
    if [[ -z "${tree_sha}" ]]; then
      echo "  ${dir}: 上游无「${seg}」目录——跳过" >&2
      return 0
    fi
    query_sha="${tree_sha}"
  done
  # 子树递归清单：逐行 "type path"，只拉 blob 且按扩展名分流（同 fetch_dir_html）。
  local listing
  if ! listing=$(curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 60 \
    "${GH_API_AUTH[@]}" \
    "${GIT_TREES_ROOT}/${query_sha}?recursive=1" \
    | python3 -c 'import json,sys
d = json.load(sys.stdin)
if d.get("truncated"):
    print("TRUNCATED", file=sys.stderr)
for entry in d["tree"]:
    print(entry["type"], entry["path"])'); then
    echo "  ${dir}: git trees 递归清单失败（限流/网络）——跳过，稍后重跑续拉" >&2
    return 0
  fi
  local type name
  while read -r type name; do
    [[ -z "${name}" ]] && continue
    [[ "${type}" != "blob" ]] && continue
    case "${name}" in
      *.worker.js)
        if [[ "${GOAL_PULL_WORKER_JS:-0}" == "1" ]]; then
          fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      *.any.js)
        if [[ "${GOAL_PULL_ANY_JS:-0}" == "1" ]]; then
          fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      # .txt 静态资源（net-api 二期 M2-S3c：fetch/api/resources top.txt /
      # cors-top.txt / empty.txt——73F 引用面单点解锁；opt-in 防其他 goal
      # 语料面意外扩张）。
      *.txt)
        if [[ "${GOAL_PULL_TXT:-0}" == "1" ]]; then
          fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        fi
        ;;
      # .htm 与 .headers 同 fetch_dir_html 扩展模式（workers corpus 三层 .htm 用例 +
      # .headers sidecar——深挖模式此前漏拉，2026-10-20 workers-compat M1 补齐）。
      # GOAL_PULL_HEADERS_TARGETS=1 的 sidecar 目标带拉语义同 fetch_dir_html（此处
      # name 为相对路径，%#.headers 还原目标；仅无扩展名目标生效防双拉）。
      *.html | *.htm | *.headers | *.js | *.json | *.xml | *.xhtml | *.svg)
        fetch_raw "${dir}/${name}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
        if [[ "${GOAL_PULL_HEADERS_TARGETS:-0}" == "1" && "${name}" == *.headers ]]; then
          local header_target="${name%.headers}"
          if [[ -n "${header_target}" && "${header_target}" != *.* ]]; then
            fetch_raw "${dir}/${header_target}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
          fi
        fi
        ;;
    esac
  done <<< "${listing}"
}

goals_fetch_all() {
  echo "== fetch 语料（pin ${WPT_REV:0:9}，FORCE=${FORCE:-0}）=="
  local dir
  for dir in "${DIRS[@]}"; do
    if [[ "${GOAL_FETCH_DEEP:-0}" == "1" ]]; then
      fetch_dir_deep "${dir}"
    else
      fetch_dir_html "${dir}"
    fi
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
