#!/usr/bin/env bash
# Fetch a pinned subset of the upstream WPT html/semantics/embedded-content media
# tests (testharness style) used by the media-elements goal
# (docs/goal/media-elements.md, M1 / DC-1).
#
# Strategy: first batch = media-elements/ top-level testharness cases that assert
# JS-observable semantics (canPlayType / readyState/networkState constants /
# reflection / historical), plus the HTMLTrackElement reflection interface dir.
# Decode/playback-dependent cases (event_* with real media URIs, ready-states/
# autoplay, seeking/) stay out until the headless semantics layer lands or the
# decode gate (media-playback M0) resolves — they are a later slice, not skips
# baked into the runner.
#
# 与 fetch-dom-subset.sh 同构（同 WPT_REV pin + GitHub API 列目录 + raw 拉单文件）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"
API_ROOT="https://api.github.com/repos/web-platform-tests/wpt/contents"

BASE="html/semantics/embedded-content/media-elements"

# 首批：语义面（非真解码）目录。主目录顶层逐文件白名单（目录内另有大量
# event_* / autoplay / manual 用例依赖真实媒体 URI 解码——不进首批）。
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

fetch_dir() {
  local dir="$1"
  local local_dir="${WPT_DATA}/${dir}"
  # 幂等快路径：WPT_REV 固定 → 目录文件集稳定（同 fetch-dom-subset.sh）。
  if [[ "${FORCE:-0}" != "1" && -d "${local_dir}" ]]; then
    if compgen -G "${local_dir}/*" > /dev/null; then
      echo "  ${dir}: 已含用例，跳过 API 列目录（FORCE=1 可强制重列）"
      return 0
    fi
  fi
  local names
  names=$(curl --fail --location --silent --show-error --retry 5 --retry-delay 3 \
    --connect-timeout 8 --max-time 30 \
    "${API_ROOT}/${dir}" | grep -o '"name": "[^"]*"')
  while IFS= read -r line; do
    local name="${line#\"name\": \"}"
    name="${name%\"}"
    case "${name}" in
      *.html | *.js) fetch_raw "${dir}/${name}" ;;
    esac
  done <<< "${names}"
}

# 共享 helper：media 用例经 /common/media.js 取 URI（canPlayType 空表时回落 .mp4/.mp3
# 扩展名——headless 语义层不真解码，URL 仅作字符串断言面）。
fetch_raw "common/media.js"

# 主目录顶层 testharness 白名单（语义面非解码面）。
# media-elements M3 扩批（event_* 族——M2 headless 加载序列落地后事件断言可跑；
# 上游用例断言 loadstart/progress/loadedmetadata/loadeddata/canplay/canplaythrough/
# play/pause/playing/durationchange/resize 时序与状态）。
TOP_FILES=(
  "event_canplay.html"
  "event_canplay_noautoplay.html"
  "event_canplaythrough.html"
  "event_canplaythrough_noautoplay.html"
  "event_loadeddata.html"
  "event_loadeddata_noautoplay.html"
  "event_loadedmetadata.html"
  "event_loadedmetadata_noautoplay.html"
  "event_loadstart.html"
  "event_loadstart_noautoplay.html"
  "event_order_canplay_canplaythrough.html"
  "event_order_canplay_playing.html"
  "event_order_durationchange_resize_loadedmetadata.html"
  "event_order_loadedmetadata_loadeddata.html"
  "event_order_loadstart_progress.html"
  "event_pause.html"
  "event_pause_noautoplay.html"
  "event_play.html"
  "event_play_noautoplay.html"
  "event_playing.html"
  "event_playing_noautoplay.html"
  "event_timeupdate.html"
  "event_timeupdate_noautoplay.html"
  "event_progress.html"
  "event_progress_noautoplay.html"
  "mime-types/canPlayType.html"
  "error-codes/error.html"
  "src_reflects_attribute_not_source_elements.html"
  "historical.html"
  "networkState_initial.html"
  "networkState_during_loadstart.html"
  "networkState_during_progress.html"
  "readyState_initial.html"
  "readyState_during_canplay.html"
  "readyState_during_canplaythrough.html"
  "readyState_during_loadeddata.html"
  "readyState_during_loadedmetadata.html"
  "readyState_during_playing.html"
  "paused_false_during_play.html"
  "paused_true_during_pause.html"
  "preload_reflects_none_autoplay.html"
  "playing-the-media-resource/playbackRate.html"
  "offsets-into-the-media-resource/duration.html"
  "offsets-into-the-media-resource/currentTime.html"
  "location-of-the-media-resource/currentSrc.html"
  "interfaces/HTMLElement/HTMLMediaElement/crossOrigin.html"
  "interfaces/HTMLElement/HTMLMediaElement/addTextTrack.html"
  "interfaces/HTMLElement/HTMLMediaElement/textTracks.html"
)

# HTMLTrackElement 反射面（kind/src/srclang/label/default/track/readyState）。
fetch_dir "${BASE}/interfaces/HTMLElement/HTMLTrackElement"

for relative in "${TOP_FILES[@]}"; do
  fetch_raw "${BASE}/${relative}"
done

echo "Media testharness subset ready (${#TOP_FILES[@]} top files + track dir, WPT ${WPT_REV})"
