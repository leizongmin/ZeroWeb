#!/usr/bin/env bash
# Restore the pinned Service Worker Tier A testharness corpus.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_REV="04067ce9c7c2165e71ad7d0dde10a4c5cb394a83"
ASSET_MANIFEST="${WPT_ASSET_MANIFEST:-${REPO_ROOT}/docs/goal/service-workers/evidence/2026-08-19-m1-tier-a-assets.tsv}"
DATA_ROOT="${WPT_SERVICE_WORKER_DATA:-${REPO_ROOT}/tests/wpt-runner/wpt-data/.service-workers-tier-a-root}"
REMOTE_ROOT="${WPT_REMOTE_ROOT:-https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}}"
FALLBACK_ROOT="${WPT_FALLBACK_ROOT:-https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@${WPT_REV}}"
MODE="restore"

if [[ "${1:-}" == "--verify-only" ]]; then
  MODE="verify"
  shift
fi
if [[ "$#" -ne 0 ]]; then
  echo "Usage: $0 [--verify-only]" >&2
  exit 2
fi

blob_sha() {
  git hash-object -- "$1"
}

validate_entry() {
  local relative="$1"
  local expected_bytes="$2"
  local expected_sha="$3"

  case "${relative}" in
    "" | /* | ../* | */../* | */..) return 1 ;;
  esac
  [[ "${expected_bytes}" =~ ^[0-9]+$ ]]
  [[ "${expected_sha}" =~ ^[0-9a-f]{40}$ ]]
}

matches_manifest() {
  local file="$1"
  local expected_bytes="$2"
  local expected_sha="$3"

  [[ -f "${file}" ]] || return 1
  [[ "$(wc -c < "${file}")" -eq "${expected_bytes}" ]] || return 1
  [[ "$(blob_sha "${file}")" == "${expected_sha}" ]]
}

fetch_remote() {
  local root="$1"
  local relative="$2"
  local output="$3"

  curl --fail --location --silent --show-error --retry 2 --retry-all-errors \
    --continue-at - \
    --connect-timeout 10 --max-time 30 \
    "${root}/${relative}" -o "${output}"
}

restore_asset() {
  local relative="$1"
  local expected_bytes="$2"
  local expected_sha="$3"
  local target="${DATA_ROOT}/${relative}"
  local temporary="${target}.tmp"

  if matches_manifest "${target}" "${expected_bytes}" "${expected_sha}"; then
    return 0
  fi

  mkdir -p "$(dirname "${target}")"
  if [[ -n "${WPT_SOURCE:-}" ]]; then
    cp "${WPT_SOURCE}/${relative}" "${temporary}"
  else
    if ! fetch_remote "${REMOTE_ROOT}" "${relative}" "${temporary}"; then
      if [[ -z "${FALLBACK_ROOT}" || "${FALLBACK_ROOT}" == "${REMOTE_ROOT}" ]] ||
        ! fetch_remote "${FALLBACK_ROOT}" "${relative}" "${temporary}"; then
        echo "Tier A WPT download incomplete; resumable file kept: ${relative}" >&2
        return 1
      fi
    fi
  fi

  if ! matches_manifest "${temporary}" "${expected_bytes}" "${expected_sha}"; then
    echo "Tier A WPT asset failed manifest verification: ${relative}" >&2
    rm -f "${temporary}"
    return 1
  fi
  mv "${temporary}" "${target}"
}

[[ -f "${ASSET_MANIFEST}" ]] || {
  echo "Tier A WPT asset manifest not found: ${ASSET_MANIFEST}" >&2
  exit 1
}

count=0
while IFS=$'\t' read -r relative _manifest_type _roles _referenced_by expected_bytes _templated expected_sha; do
  if [[ "${relative}" == "path" ]]; then
    continue
  fi
  if ! validate_entry "${relative}" "${expected_bytes}" "${expected_sha}"; then
    echo "Invalid Tier A WPT manifest entry: ${relative}" >&2
    exit 1
  fi
  if [[ "${MODE}" == "verify" ]]; then
    if ! matches_manifest "${DATA_ROOT}/${relative}" "${expected_bytes}" "${expected_sha}"; then
      echo "Tier A WPT asset does not match manifest: ${relative}" >&2
      exit 1
    fi
  else
    restore_asset "${relative}" "${expected_bytes}" "${expected_sha}"
  fi
  count=$((count + 1))
done < "${ASSET_MANIFEST}"

if [[ "${count}" -ne 18 ]]; then
  echo "Tier A WPT asset count mismatch: expected 18, found ${count}" >&2
  exit 1
fi

echo "Service Worker Tier A corpus ${MODE} complete (${count} assets, WPT ${WPT_REV})"
