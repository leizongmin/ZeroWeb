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

# the-audio-element 反射面（M3 扩批 III：Audio 构造器 spec 语义——preload=auto +
# 无 new TypeError；兄弟目录，路径前缀同级）。
fetch_raw "html/semantics/embedded-content/the-audio-element/audio_constructor.html"
# the-video-element 反射面（M3 扩批 IV：属性不凭空出现——UA 面不加 tabindex）。
fetch_raw "html/semantics/embedded-content/the-video-element/video-tabindex.html"
# M3 扩批 VIII：空 src 容错面（about:blank/"" src → error 事件不 crash——
# M3 扩批 II 的空 src 错误码语义 + 动态 .src= 模拟覆盖面）。
fetch_raw "html/semantics/embedded-content/the-video-element/video_crash_empty_src.html"

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
  "event_volumechange.html"
  "volume_nonfinite.html"
  "controlsList.tentative.html"
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
  # M3 扩批 VII：移除文档暂停面（spec「media elements pause on removal」——
  # 同步 paused 面保持 + stable state 后 pause 事件 + 重插不续播）。
  "playing-the-media-resource/pause-remove-from-document.html"
  # M3 扩批 XIV：移除暂停两变体（appendChild 后 src 路径 + NETWORK_EMPTY 负例——
  # 无候选 play() promise pending → 移除后 AbortError）。
  "playing-the-media-resource/pause-remove-from-document-different-load.html"
  "playing-the-media-resource/pause-remove-from-document-networkState.html"
  # M3 扩批 IX：移动面（同文档移动仍 related → 不暂停）。
  # pause-move-to-other-document 不导入：跨 iframe 文档 adopt 在 shim 融合视图下
  # appendChild 静默落空（元素保持 detached）——实施需 iframe 文档模型 adopt 面
  #（深结构），当前形态导入即假失败；待 iframe adopt 面落地后重评。
  "playing-the-media-resource/pause-move-within-document.html"
  "offsets-into-the-media-resource/duration.html"
  "offsets-into-the-media-resource/currentTime.html"
  "location-of-the-media-resource/currentSrc.html"
  "interfaces/HTMLElement/HTMLMediaElement/crossOrigin.html"
  "interfaces/HTMLElement/HTMLMediaElement/addTextTrack.html"
  "interfaces/HTMLElement/HTMLMediaElement/textTracks.html"
)

# HTMLTrackElement 反射面（kind/src/srclang/label/default/track/readyState）。
fetch_dir "${BASE}/interfaces/HTMLElement/HTMLTrackElement"

# M3 扩批 X：track 子元素 ↔ video.textTracks 集合同步断言面（逐文件白名单——
# track-element 目录整体含大量 VTT 解析/cue 渲染用例，依赖真字幕加载，不整目录）。
# M3 扩批 XI：resource selection 算法 JS 可观察面（逐文件白名单——networkState
# 同步段/稳定态、invoke 面、src 移除不触发；依赖真资源失败时序的 pointer/candidate/
# source-media 族 + MSE/iframe/manual + data:, error settle 两案不导入，见 master.md）。
RS_FILES=(
  "autoplay-overrides-preload.html"
  "load-removes-queued-error-event.html"
  "resource-selection-candidate-insert-before.html"
  "resource-selection-invoke-audio-constructor-no-src.html"
  "resource-selection-invoke-audio-constructor.html"
  "resource-selection-invoke-in-sync-event.html"
  "resource-selection-invoke-insert-fragment-into-document.html"
  "resource-selection-invoke-insert-into-document.html"
  "resource-selection-invoke-insert-parent-into-document.html"
  "resource-selection-invoke-insert-source-in-div.html"
  "resource-selection-invoke-insert-source-in-namespace.html"
  "resource-selection-invoke-insert-source-not-in-document.html"
  "resource-selection-invoke-insert-source.html"
  "resource-selection-invoke-load.html"
  "resource-selection-invoke-pause.html"
  "resource-selection-invoke-play.html"
  "resource-selection-invoke-remove-from-document.html"
  "resource-selection-invoke-remove-src.html"
  "resource-selection-invoke-set-src-in-namespace.html"
  "resource-selection-invoke-set-src-networkState.html"
  "resource-selection-invoke-set-src-not-in-document.html"
  "resource-selection-invoke-set-src.html"
  "resource-selection-remove-source.html"
  "resource-selection-remove-src.html"
  "resource-selection-resumes-onload.html"
)
for relative in "${RS_FILES[@]}"; do
  fetch_raw "${BASE}/loading-the-media-resource/${relative}"
done

TRACK_ELEMENT_FILES=(
  "track/track-element/track-api-texttracks.html"
  "track/track-element/track-addtrack-kind.html"
  "track/track-element/track-texttracks.html"
  "track/track-element/track-node-add-remove.html"
  "track/track-element/track-id.html"
  "track/track-element/track-element-dom-change.html"
  # M3 扩批 XIII：VTTCue 定位选项 IDL 面（headless 仅存储不做视觉布局）+
  # data:text/vtt 加载（crossorigin 属性三态——headless 不区分 CORS 模式）+
  # addtrack 异步派发 / cue order 排序断言 / src 变更清 cue。
  # track-change-event 不导入：change 事件广播需 TextTrackList↔TextTrack 反向链 +
  # 模式变更前 list 未暴露的 pending 重放（深结构）——见 master.md 排除清单。
  "track/track-element/vtt-cue-float-precision.html"
  "track/track-element/track-data-url.html"
  "track/track-element/track-add-track.html"
  "track/track-element/track-cue-order.html"
  "track/track-element/src-clear-cues.html"
  # M3 扩批 XV：http(s) VTT 文件加载 + 解析深化（header 校验/cue id 错误恢复/
  # cue settings/实体解码——shim part06 `_zwParseVtt`）。
  # track-add-remove-cue：settings.vtt 加载 + cue 增删/排序 + VTTCue 缺省反射。
  "track/track-element/track-add-remove-cue.html"
  # cue id 行解析（含 '-->' 错误恢复——id 行含 --> 不识别）。
  "track/track-element/track-webvtt-cue-identifiers.html"
  # 空行/无分隔 cue 块解析。
  "track/track-element/track-webvtt-blank-lines.html"
  # cue settings（line/position/size/align/vertical，含 % 值与 tab 分隔）。
  "track/track-element/track-webvtt-settings.html"
  # 实体解码（&amp;/&lt;/&gt;/&lrm;/&rlm;/&nbsp;）+ settings 组合。
  "track/track-element/track-webvtt-entities.html"
  # 小时位时间戳（00:00:00.000 / 100:20:00.500）。
  "track/track-element/track-webvtt-timings-hour.html"
  # WEBVTT magic header 校验（rubbish 头拒收 → error + cues []；no-webvtt 拒收）。
  "track/track-element/track-webvtt-magic-header.html"
  # header 长度/名称校验（四变体两 load 两 error）。
  "track/track-element/track-webvtt-header-checks.html"
  # 负时间戳 cue（startTime=-5 等——headless 纯存储/排序面）。
  "track/track-element/track-cue-negative-timestamp.html"
  # src 三段变更（cues 立即清空 + same list 身份 + 同值变更不重载）。
  "track/track-element/track-element-src-change.html"
  # default 属性 readyState=LOADED 面（静态 HTML 两 track 形态）。
  "track/track-element/track-default-attribute.html"
  # src setter 触发加载（NONE → LOADED；mode hidden 先设）。
  "track/track-element/track-load-from-src-readyState.html"
  # M3 扩批 XVII（2026-09-03）：track-cues-* 播放推进族续批——随 fixture-mounted
  # 切片 2 基础设施（区间捕获 + 事件时间序 + ended 面）逐件复评导入。
  "track/track-element/track-cues-sorted-before-dispatch.html"
)
# M3 扩批 XV：上述用例引用的共享 helper（inline_local_scripts 相对路径内联）
# 与 VTT 资源文件（runner fetch handler 以 wpt-data 静态文件服务）。
TRACK_ELEMENT_SUPPORT=(
  "track/track-element/track-helpers.js"
  "track/track-element/resources/webvtt-file.vtt"
  "track/track-element/resources/webvtt-file.vtt"
  "track/track-element/resources/settings.vtt"
  "track/track-element/resources/settings-bad-separation.vtt"
  "track/track-element/resources/entities.vtt"
  "track/track-element/resources/cue-id.vtt"
  "track/track-element/resources/cue-id-error.vtt"
  "track/track-element/resources/cues.vtt"
  "track/track-element/resources/cues-overlapping.vtt"
  "track/track-element/resources/sorted-dispatch.vtt"
  "track/track-element/resources/cues-no-separation.vtt"
  "track/track-element/resources/metadata.vtt"
  "track/track-element/resources/header-empty-after.vtt"
  "track/track-element/resources/header-newlines-after.vtt"
  "track/track-element/resources/header-too-short.vtt"
  "track/track-element/resources/header-invalid-equal.vtt"
  "track/track-element/resources/webvtt-rubbish.vtt"
  "track/track-element/resources/no-webvtt.vtt"
)
for relative in "${TRACK_ELEMENT_FILES[@]}"; do
  fetch_raw "${BASE}/${relative}"
done
for relative in "${TRACK_ELEMENT_SUPPORT[@]}"; do
  fetch_raw "${BASE}/${relative}"
done

# M3 扩批 XVI 备料（2026-09-02）：真媒体文件——fixture-mounted runner 播放用例面
# （media-playback M3「上游 WPT 可执行子集导入」+ media-elements track-cues-* 播放
# 推进族的媒体源）。实测 codec 面：VP9 视频（+Opus 音频轨）——与 zero-media 解码面
# 对齐（rusty_vp9 / opus-decoder）。sound_5.mp3 = symphonia mp3 面。
# https://github.com/web-platform-tests/wpt/tree/3159769/media
MEDIA_FILES=(
  "media/movie_5.webm"
  "media/sound_5.mp3"
  "media/test.webm"
  "media/test-1s.webm"
  "media/counting.webm"
)
for relative in "${MEDIA_FILES[@]}"; do
  fetch_raw "${relative}"
done

for relative in "${TOP_FILES[@]}"; do
  fetch_raw "${BASE}/${relative}"
done

# M3 扩批 XII：TextTrack 家族接口语义面（TextTrack/TextTrackCueList/TextTrackList/
# TextTrackCue/TrackEvent——VTTCue 最小面 + addCue/removeCue/cues 排序/getCueById/
# mode 枚举归一/on* EventTarget 面）。VTT 解析依赖子测（track.src=data:text/vtt →
# parsed cue）经 settle 链 data: URL 文本面解锁（shim part06 `_zwParseVttDataUrl`）。
INTERFACES_FILES=(
  "interfaces/TextTrack/activeCues.html"
  "interfaces/TextTrack/addCue.html"
  "interfaces/TextTrack/constants.html"
  "interfaces/TextTrack/cues.html"
  "interfaces/TextTrack/kind.html"
  "interfaces/TextTrack/label.html"
  "interfaces/TextTrack/language.html"
  "interfaces/TextTrack/mode.html"
  "interfaces/TextTrack/oncuechange.html"
  "interfaces/TextTrack/removeCue.html"
  "interfaces/TextTrackCue/constructor.html"
  "interfaces/TextTrackCue/endTime.html"
  "interfaces/TextTrackCue/id.html"
  "interfaces/TextTrackCue/onenter.html"
  "interfaces/TextTrackCue/onexit.html"
  "interfaces/TextTrackCue/pauseOnExit.html"
  "interfaces/TextTrackCue/startTime.html"
  "interfaces/TextTrackCue/track.html"
  "interfaces/TextTrackCueList/getCueById.html"
  "interfaces/TextTrackCueList/getter.html"
  "interfaces/TextTrackCueList/length.html"
  "interfaces/TextTrackList/getTrackById.html"
  "interfaces/TextTrackList/getter.html"
  "interfaces/TextTrackList/length.html"
  "interfaces/TextTrackList/onaddtrack.html"
  "interfaces/TextTrackList/onremovetrack.html"
  "interfaces/TrackEvent/constructor.html"
  "interfaces/TrackEvent/createEvent.html"
)
for relative in "${INTERFACES_FILES[@]}"; do
  fetch_raw "${BASE}/${relative}"
done

echo "Media testharness subset ready (${#TOP_FILES[@]} top files + track dir, WPT ${WPT_REV})"
