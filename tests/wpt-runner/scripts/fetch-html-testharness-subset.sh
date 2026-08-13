#!/usr/bin/env bash
# Fetch the pinned upstream WPT files used by the HTML interaction harness.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

FILES=(
  "resources/testharness.js"
  "html/semantics/forms/the-output-element/output.html"
  "html/semantics/forms/the-input-element/input-whitespace.html"
  "html/interaction/focus/sequential-focus-navigation-and-the-tabindex-attribute/focus-tabindex-default-value.html"
  "uievents/constructors/inputevent-constructor.html"
)

for relative in "${FILES[@]}"; do
  target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    continue
  fi
  mkdir -p "$(dirname "${target}")"
  temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 3 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -s "${temporary}"
  mv "${temporary}" "${target}"
done

echo "HTML testharness subset ready (${#FILES[@]} files, WPT ${WPT_REV})"
