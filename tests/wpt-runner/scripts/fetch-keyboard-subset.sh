#!/usr/bin/env bash
# Fetch a pinned subset of the upstream WPT keyboard interaction tests
# (testharness style) used by the keyboard-default-actions goal
# (docs/goal/keyboard-default-actions.md, M1 / DC-1).
#
# Strategy: first batch = uievents/keyboard event faces (keydown/keyup/
# input ordering, KeyboardEvent composed/legacy, modifier keys) + forms
# implicit-submission face. These exercise the keydown default-action
# dispatch layer against the runner's testdriver send_keys adapter (R142).
# 与 fetch-selection-subset.sh 同构（同 WPT_REV pin + raw 拉单文件）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  echo "fetch ${relative}"
  curl -fsSL --retry 3 --max-time 60 "${RAW_ROOT}/${relative}" -o "${target}"
}

# M1 首批（2026-09-07）：uievents/keyboard 全主线程用例（manual 两案排除——
# 需真物理键盘布局）+ forms implicit-submission（Enter 提交规则 K3 驱动面）。
# 排除项记录于 evidence 导入清单，非静默丢弃。
CASES=(
  "uievents/keyboard/keydown-input-events.html"
  "uievents/keyboard/keyboardevent-composed.html"
  "uievents/keyboard/keyboardevent-legacy.html"
  "uievents/keyboard/keypress-not-fired-for-modifier-shortcuts.html"
  "uievents/keyboard/modifier-keys.html"
  "html/semantics/forms/form-submission-0/implicit-submission.optional.html"
)

# 用例依赖的 helper（targetted-form.js——implicit-submission 断言用）。
HELPERS=(
  "html/semantics/forms/form-submission-0/resources/targetted-form.js"
)

for rel in "${CASES[@]}" "${HELPERS[@]}"; do
  fetch_raw "$rel"
done

echo "keyboard subset ready: ${#CASES[@]} cases + ${#HELPERS[@]} helpers @ ${WPT_REV:0:12}"
