#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNNER_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DATA_DIR="${RUNNER_DIR}/wpt-data"
TEST_LEDGER="${RUNNER_DIR}/imported-tests.txt"
RESOURCE_LEDGER="${RUNNER_DIR}/imported-resources.txt"

missing=0

inspect_page() {
  local page="$1"
  local file="${DATA_DIR}/${page}"
  [[ -f "${file}" ]] || return 0

  while IFS= read -r raw_url; do
    local url="${raw_url#url(}"
    url="${url%)}"
    url="${url#\"}"
    url="${url%\"}"
    url="${url#\'}"
    url="${url%\'}"

    case "${url}" in
      data:*|http:*|https:*) continue ;;
      /*) resolved="${DATA_DIR}/${url#/}" ;;
      *) resolved="$(dirname "${file}")/${url}" ;;
    esac

    case "${resolved}" in
      *.ttf|*.otf|*.woff|*.woff2)
        local relative
        relative="$(realpath -m --relative-to="${DATA_DIR}" "${resolved}")"
        if ! grep -Fqx "${relative}" "${RESOURCE_LEDGER}"; then
          printf 'missing font resource: %s -> %s\n' "${page}" "${relative}" >&2
          missing=1
        fi
        ;;
    esac
  done < <(grep -Eo 'url\([^)]*\)' "${file}" || true)
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
