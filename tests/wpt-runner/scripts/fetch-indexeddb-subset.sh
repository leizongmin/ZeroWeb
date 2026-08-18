#!/usr/bin/env bash
# Fetch the pinned first IndexedDB testharness slice used by storage-indexeddb M1.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

FILES=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
  "IndexedDB/resources/support.js"
  "IndexedDB/resources/support-promises.js"
  "IndexedDB/resources/nested-cloning-common.js"
  "IndexedDB/resources/support-get-all.js"
  "IndexedDB/globalscope-indexedDB-SameObject.any.js"
  "IndexedDB/idbfactory_cmp.any.js"
  "IndexedDB/idbfactory_deleteDatabase.any.js"
  "IndexedDB/idbfactory-deleteDatabase-request-success.any.js"
  "IndexedDB/idbfactory_open.any.js"
  "IndexedDB/idbfactory-open-error-properties.any.js"
  "IndexedDB/idbfactory-open-request-error.any.js"
  "IndexedDB/idbfactory-open-request-success.any.js"
  "IndexedDB/idbversionchangeevent.any.js"
  "IndexedDB/idbobjectstore_add.any.js"
  "IndexedDB/idbobjectstore_put.any.js"
  "IndexedDB/idbobjectstore_get.any.js"
  "IndexedDB/idbobjectstore_delete.any.js"
  "IndexedDB/idbobjectstore_clear.any.js"
  "IndexedDB/idbobjectstore_count.any.js"
  "IndexedDB/idbobjectstore_getAll.any.js"
  "IndexedDB/idbobjectstore_getAllKeys.any.js"
  "IndexedDB/idbindex_get.any.js"
  "IndexedDB/idbindex_getKey.any.js"
  "IndexedDB/idbindex_count.any.js"
  "IndexedDB/idbcursor-continue.any.js"
  "IndexedDB/idbcursor-advance.any.js"
  "IndexedDB/idbcursor-advance-invalid.any.js"
  "IndexedDB/idbcursor-advance-continue-async.any.js"
  "IndexedDB/idbcursor-continuePrimaryKey.any.js"
  "IndexedDB/idbcursor-continuePrimaryKey-exceptions.any.js"
  "IndexedDB/idbcursor-continuePrimaryKey-exception-order.any.js"
  "IndexedDB/idbrequest_result.any.js"
  "IndexedDB/idbrequest_error.any.js"
  "IndexedDB/idbtransaction-objectStore-finished.any.js"
  "IndexedDB/idbtransaction_abort.any.js"
  "IndexedDB/request_bubble-and-capture.any.js"
  "IndexedDB/transaction-abort-request-error.any.js"
  "IndexedDB/error-attributes.any.js"
  "IndexedDB/idbtransaction-oncomplete.any.js"
  "IndexedDB/transaction-deactivation-timing.any.js"
  "IndexedDB/event-dispatch-active-flag.any.js"
  "IndexedDB/transaction-lifetime-empty.any.js"
  "IndexedDB/transaction-scheduling-across-connections.any.js"
  "IndexedDB/transaction-scheduling-across-databases.any.js"
  "IndexedDB/transaction-scheduling-mixed-scopes.any.js"
  "IndexedDB/transaction-scheduling-ordering.any.js"
  "IndexedDB/transaction-scheduling-ro-waits-for-rw.any.js"
  "IndexedDB/transaction-scheduling-rw-scopes.any.js"
  "IndexedDB/transaction-scheduling-within-database.any.js"
)

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error --retry 1 \
    --connect-timeout 8 --max-time 20 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    return 1
  fi
  test -s "${temporary}"
  mv "${temporary}" "${target}"
}

fetch_from_git() {
  local checkout
  checkout="$(mktemp -d "${TMPDIR:-/tmp}/zeroweb-wpt-indexeddb.XXXXXX")"
  trap "rm -rf -- '${checkout}'" EXIT
  git -C "${checkout}" init --quiet
  git -C "${checkout}" remote add origin https://github.com/web-platform-tests/wpt.git
  git -C "${checkout}" config core.sparseCheckout true
  printf '%s\n' "${FILES[@]}" > "${checkout}/.git/info/sparse-checkout"
  local attempt
  for attempt in 1 2 3; do
    if git -C "${checkout}" fetch --quiet --depth=1 --filter=blob:none origin "${WPT_REV}"; then
      break
    fi
    if [[ "${attempt}" == "3" ]]; then
      return 1
    fi
    sleep 2
  done
  git -C "${checkout}" checkout --quiet FETCH_HEAD
  for relative in "${FILES[@]}"; do
    local target="${WPT_DATA}/${relative}"
    mkdir -p "$(dirname "${target}")"
    cp "${checkout}/${relative}" "${target}"
  done
}

fetch_from_checkout() {
  local checkout="${WPT_SOURCE:?WPT_SOURCE is required}"
  local revision
  revision="$(git -C "${checkout}" rev-parse HEAD)"
  if [[ "${revision}" != "${WPT_REV}" ]]; then
    echo "WPT_SOURCE revision ${revision} does not match pinned ${WPT_REV}" >&2
    return 1
  fi
  for relative in "${FILES[@]}"; do
    local source="${checkout}/${relative}"
    local target="${WPT_DATA}/${relative}"
    test -s "${source}"
    mkdir -p "$(dirname "${target}")"
    cp "${source}" "${target}"
  done
}

if [[ -n "${WPT_SOURCE:-}" ]]; then
  fetch_from_checkout
  echo "IndexedDB testharness subset ready (45 cases, WPT ${WPT_REV})"
  exit 0
fi

raw_failed=0
for file in "${FILES[@]}"; do
  if ! fetch_raw "${file}"; then
    raw_failed=1
    break
  fi
done

if [[ "${raw_failed}" == "1" ]]; then
  echo "Raw WPT download unavailable; using pinned sparse Git fetch"
  fetch_from_git
fi

echo "IndexedDB testharness subset ready (45 cases, WPT ${WPT_REV})"
