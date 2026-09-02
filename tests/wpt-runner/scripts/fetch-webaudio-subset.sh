#!/usr/bin/env bash
# Fetch the pinned subset of upstream WPT webaudio the-audio-api interface tests
# used by the media-audio goal M3 Web Audio minimal face
# (docs/goal/media-audio/master.md, D1 批复切片 2).
#
# Strategy: constructor/interface-semantic cases only — the shim's AudioContext
# face (shim part06) covers construction + node interface + connect semantics.
# Rendering-dependent cases (startRendering / worklet / OfflineAudioContext
# rendering) stay out — RFC §0 exclusion list.
#
# 与 fetch-media-subset.sh 同构（同 WPT_REV pin + raw 拉单文件）；wpt-data
# gitignored，用例按需 fetch、不入库。

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
  local temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 5 --retry-delay 3 \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -e "${temporary}"
  mv "${temporary}" "${target}"
}

WA_FILES=(
  "webaudio/the-audio-api/the-audionode-interface/audionode-connect-return-value.html"
  "webaudio/the-audio-api/the-destinationnode-interface/destination.html"
  # OscillatorNode 构造器面（audit.js 框架——runner 内联 webaudio/resources/*.js）。
  "webaudio/the-audio-api/the-oscillatornode-interface/ctor-oscillator.html"
)

# audit.js 框架（runner inline_extras 内联——用例以绝对路径引用）。
for f in audit.js audit-util.js audionodeoptions.js; do
  fetch_raw "webaudio/resources/${f}"
done

for relative in "${WA_FILES[@]}"; do
  fetch_raw "${relative}"
done

echo "Web Audio testharness subset ready (${#WA_FILES[@]} files + audit framework, WPT ${WPT_REV})"
