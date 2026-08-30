#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNNER_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DATA_DIR="${RUNNER_DIR}/wpt-data"
TEST_LEDGER="${RUNNER_DIR}/imported-tests.txt"
RESOURCE_LEDGER="${RUNNER_DIR}/imported-resources.txt"

missing=0

# POSIX 可移植路径归一化（解析 "."/".." 但不要求文件存在，等价 GNU `realpath -m`；
# macOS 的 BSD realpath 不支持 -m，CI macos runner 实测 "realpath: illegal option -- m"）。
normalize_web_path() {
  local path="$1" out="" segment
  local IFS='/'
  for segment in ${path}; do
    case "${segment}" in
      '' | '.') continue ;;
      '..') out="${out%/*}" ;;
      *) out="${out}/${segment}" ;;
    esac
  done
  printf '%s' "${out}"
}

inspect_font_urls() {
  local file="$1"
  local owner="$2"
  local web_path="$3"
  while IFS= read -r raw_url; do
    local url="${raw_url#url(}"
    url="${url%)}"
    url="${url#\"}"
    url="${url%\"}"
    url="${url#\'}"
    url="${url%\'}"

    local relative resolved
    case "${url}" in
      data:*|http:*|https:*) continue ;;
      /*) relative="${url#/}" ;;
      *) relative="$(normalize_web_path "/$(dirname "${web_path}")/${url}")"; relative="${relative#/}" ;;
    esac
    resolved="${DATA_DIR}/${relative}"

    case "${resolved}" in
      *.ttf|*.otf|*.woff|*.woff2)
        if ! grep -Fqx "${relative}" "${RESOURCE_LEDGER}"; then
          printf 'missing font resource: %s -> %s\n' "${owner}" "${relative}" >&2
          missing=1
        fi
        ;;
    esac
  done < <(grep -Eo 'url\([^)]*\)' "${file}" || true)
}

inspect_page() {
  local page="$1"
  local file="${DATA_DIR}/${page}"
  [[ -f "${file}" ]] || return 0

  inspect_font_urls "${file}" "${page}" "${page}"

  while IFS= read -r raw_href; do
    local href="${raw_href#href=}"
    href="${href#\"}"
    href="${href%\"}"
    href="${href#\'}"
    href="${href%\'}"

    local stylesheet_relative stylesheet
    case "${href}" in
      http:*|https:*) continue ;;
      /*) stylesheet_relative="${href#/}" ;;
      *)
        stylesheet_relative="$(normalize_web_path "/$(dirname "${page}")/${href}")"
        stylesheet_relative="${stylesheet_relative#/}"
        ;;
    esac
    stylesheet="${DATA_DIR}/${stylesheet_relative}"
    if [[ ! -f "${stylesheet}" ]]; then
      printf 'missing stylesheet resource: %s -> %s\n' "${page}" "${href}" >&2
      missing=1
      continue
    fi
    inspect_font_urls "${stylesheet}" "${page} -> ${href}" "${stylesheet_relative}"
  done < <(
    # 只取**真实 stylesheet link**（`rel="stylesheet"` 在同一 <link> 标签内）——
    # 裸 grep `href="…css…"` 会命中 <script> 内 JS 字符串字面量的 CSS 文本
    #（R381 实测：dom/nodes/processing-instruction-attributes.html 的 PI data
    # `type="text/css" href="style.css"` 是测试断言字符串而非依赖，style.css 上游
    # 不存在，假阳性使 fetch-wpt-data 整步 fatal）。
    {
      grep -Eo '<link[^>]*rel="stylesheet"[^>]*>' "${file}" | grep -Eo 'href="[^"]+\.css[^"]*"' || true
      grep -Eo "<link[^>]*rel='stylesheet'[^>]*>" "${file}" | grep -Eo "href='[^']+\\.css[^']*'" || true
      # rel 在 href 之后的标签序（<link href=… rel=stylesheet>）
      grep -Eo '<link[^>]*href="[^"]+\.css[^"]*"[^>]*rel="stylesheet"[^>]*>' "${file}" | grep -Eo 'href="[^"]+\.css[^"]*"' || true
      grep -Eo "<link[^>]*href='[^']+\\.css[^']*'[^>]*rel='stylesheet'[^>]*>" "${file}" | grep -Eo "href='[^']+\\.css[^']*'" || true
    } | sort -u
  )
}

while read -r test reference _; do
  [[ -n "${test}" && "${test}" != \#* ]] || continue
  inspect_page "${test}"
  inspect_page "${reference}"
done < "${TEST_LEDGER}"

if (( missing != 0 )); then
  exit 1
fi

echo "imported font resource closure: PASS"
