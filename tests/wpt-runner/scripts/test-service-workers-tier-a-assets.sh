#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FETCH_SCRIPT="${SCRIPT_DIR}/fetch-service-workers-tier-a.sh"
SOURCE_ROOT="${WPT_SERVICE_WORKER_SOURCE:?set WPT_SERVICE_WORKER_SOURCE to a verified Tier A corpus}"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/zeroweb-sw-tier-a.XXXXXX")"
trap 'rm -rf -- "${TMP_ROOT}"' EXIT

run_fetch() {
  WPT_SOURCE="${SOURCE_ROOT}" \
    WPT_SERVICE_WORKER_DATA="${TMP_ROOT}" \
    "${FETCH_SCRIPT}" "$@"
}

run_fetch
run_fetch --verify-only

asset="service-workers/service-worker/resources/empty-worker.js"
printf '\ntampered\n' >> "${TMP_ROOT}/${asset}"
if run_fetch --verify-only >/dev/null 2>&1; then
  echo "verify-only unexpectedly accepted a tampered asset" >&2
  exit 1
fi
run_fetch
run_fetch --verify-only

rm "${TMP_ROOT}/${asset}"
if run_fetch --verify-only >/dev/null 2>&1; then
  echo "verify-only unexpectedly accepted a missing asset" >&2
  exit 1
fi

echo "Service Worker Tier A asset regression: PASS"
