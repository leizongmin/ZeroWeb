#!/usr/bin/env bash
# Fetch the pinned upstream WPT html/semantics/forms/constraints testharness cases
# used by the Form Validation goal (docs/goal/form-validation.md).
#
# 用例面：constraint-validation（required/pattern/min/max/step/length/type 约束位 +
# checkValidity/reportValidity/validationMessage/willValidate API 语义）。
# R2825 的 permissive 基础已存在（part01/04.js）；本脚本补 WPT 真实用例驱动约束计算。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
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
  # 404/网络失败容忍（失败不阻断——set -e 下 return 0 继续后续抓取）。
  if ! curl --fail --location --silent --show-error --retry 3 \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT}/${relative}" -o "${temporary}" 2>/dev/null; then
    rm -f "${temporary}"
    return 0
  fi
  test -s "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_dir_files() {
  local dir="$1"
  local names
  names=$(curl --fail --location --silent --show-error --retry 3 --connect-timeout 8 --max-time 30 \
    "${API_ROOT}/${dir}?ref=${WPT_REV}" | grep -o '"name": "[^"]*"' | sed 's/"name": "//; s/"$//')
  while IFS= read -r name; do
    [[ -z "${name}" ]] && continue
    case "${name}" in
      *.html|*.js|*.png|*.css|*.py) fetch_raw "${dir}/${name}" ;;
    esac
  done <<< "${names}"
}

# constraints 顶层用例 + support 资源
fetch_dir_files "html/semantics/forms/constraints"
fetch_dir_files "html/semantics/forms/constraints/support"
# FV M2/M3：interactive validation 的 forms 用例（requestSubmit/checkValidity）
fetch_raw "html/semantics/forms/the-form-element/form-requestsubmit.html"
fetch_raw "html/semantics/forms/the-form-element/form-checkvalidity.html"
