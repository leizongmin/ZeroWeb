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
# M3 扩批 XXXVI：the-audio-element 目录清点——audio-loading-eager 导入（loading=eager
# 立即加载面，与 XXXV video-loading-eager 同构）。余件逐件定性：loading-lazy-* 与
# load/autoplay/poster(preload)-deferred 系维持排除（lazy 断言「视口外不加载」与
# eager-by-default 实现互斥——lazy-loading 视口交叉观测深结构项）；audio_001/002 +
# audio_content-ref 为 reftest 渠道（audio 元素内容不呈现面，testharness-media 不
# 适用）；audio-*-inactive-document-crash（iframe contentDocument cloneNode 深结构）
# / audio-with-replaced-after-pseudo-crash（::after replaced content 渲染域）不导入。
fetch_raw "html/semantics/embedded-content/the-audio-element/audio-loading-eager.html"
# M3 扩批 XXXVIII：track-cue-empty 解除排除导入（getCueAsHTML 空 cue 面——
# DocumentFragment + 单空 Text 节点 + constructor.name 断言；配套 shim Text.prototype
# constructor 自引修复（WebIDL §4.6.1），XXX 排除注记「shim 工厂面差异」消除）。
fetch_raw "html/semantics/embedded-content/media-elements/track/track-element/track-cue-empty.html"
# M3 扩批 XXXIX：audio_loop_base 试导（loop=true → seeking 事件二次派发——扩批
# XXIV registry set_loop 回卷 + XXXIX wrap_pending 可观测面落地后解除排除）。
# 不导入 video_loop_base：fixture 2x2-green.webm 为 VP8+Opus（解码面外——canPlayType
# 判定串 'vp9, opus' → .webm，上游 .webm 变体实为 vp8；bridge play 失败回落 headless
# 无流末界，结构互斥），随 VP8/解码面扩展复评。
fetch_raw "html/semantics/embedded-content/media-elements/audio_loop_base.html"
# the-video-element 反射面（M3 扩批 IV：属性不凭空出现——UA 面不加 tabindex）。
fetch_raw "html/semantics/embedded-content/the-video-element/video-tabindex.html"
# M3 扩批 XXXV：loading=eager 立即加载面（loadeddata 到达——headless settle 无视口
# gate，加载即数据可用；lazy 系 defer 断言用例与 eager-by-default 实现互斥，维持
# 不导入）。
fetch_raw "html/semantics/embedded-content/the-video-element/video-loading-eager.html"
# M3 扩批 VIII：空 src 容错面（about:blank/"" src → error 事件不 crash——
# M3 扩批 II 的空 src 错误码语义 + 动态 .src= 模拟覆盖面）。
fetch_raw "html/semantics/embedded-content/the-video-element/video_crash_empty_src.html"
# 不导入 video_size_preserved_after_ended（2026-09-04 实证）：静态 <source> 形态
# loadedmetadata 与 promise_test EventWatcher 时序 headless 双通道 settle 下不稳定。
# 不导入 video_timeupdate_on_seek（WPT CGI src）/ video_initially_paused（reftest）。

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
  # M3 扩批 XXIV（2026-09-03）：loop 属性真面（spec「ended playback」步 6.4——
  # loop 元素不进入 ended playback，位置回卷重播 + seeked 非 ended）。配套：
  # registry set_loop（音频 entry 流末回卷重建解码器 + 伴生轨同面）+ shim loop IDL
  # setter/getter（_mediaState 镜像 + 桥推送 + play 起播同步）+ march Ended 面
  # loop 分叉（seek(0)+play + seeking/seeked 派发）。fixture 增 media/sound_0.mp3。
  # audio_loop_base 勘误（M3 扩批 XXXIX 解除排除导入）：静默回卷对语义层不可观测
  # 的根因经 registry wrap_pending 可观测面修复（march Ended/loop 分叉 isEnded 驱动
  # 恢复可达）——「looped 标志依赖二次 seeking 回调时序」注记失效。video_loop_base
  # 维持不导入（2x2-green.webm 为 VP8——解码面域外，见上）。played-loop 已导入。
  "played-loop.html"
  "audio_loop_seek_to_eos.html"
  # M3 扩批 XXV（2026-09-03）：loop-from-ended.tentative——ended 后设 loop 再 play
  # 回卷 seeked（crbug 364442 断言面）。duration getter settle 竞态兜底（扩批 XXIV）
  # 解除其 seek 目标真值前置。
  "playing-the-media-resource/loop-from-ended.tentative.html"
  # M3 扩批 XXXIV（2026-09-04）：play-in-detached-document——detached 文档（
  # createHTMLDocument）video play() 推进面（headless 时钟 + 周期 timeupdate；
  # 此前「依赖真播放钟」排除注记随扩批 XXVII 基建失效）。
  "playing-the-media-resource/play-in-detached-document.html"
  # M3 扩批 XXVI（2026-09-04）：seeking/ 三件——seekable TimeRanges 面（seek to
  # currentTime/max/negative：clamp 边界 + seeking/timeupdate/seeked 事件序）。
  # volume_nonfinite——volume IDL setter 非有限 TypeError（headless 纯 IDL 面）。
  "seeking/seek-to-currentTime.html"
  "seeking/seek-to-max-value.htm"
  "seeking/seek-to-negative-time.htm"
  "volume_nonfinite.html"
  # M3 扩批 XXVII（2026-09-04）：media_fragment_seek——#t= 媒体片段起点解析
  # （npt:/HH:MM:SS/ms/percent-encode 面 + src 反射保 fragment）。fragment 起点
  # 在 settle 加载序列内初始化 currentTime（settle url 携带 hash）。
  # autoplay-with-broken-track——broken track（invalid://url/404/空）不阻塞
  # autoplay 推进（track error settle 面已有，video 照常 settle → 泵 → timeupdate）。
  # 不导入 no-autoplay-audio-history-back-does-not-play（iframe+history+postMessage
  # 导航深结构——pause-move-to-other-document 同域排除）。
  # M3 扩批 XXIX（2026-09-04）：ready-states/autoplay——autoplaying flag 与
  # play()/pause()/load() 交互 + 事件严格序（canplay→canplaythrough→play→playing；
  # play 先行形态 play→canplay→playing→canplaythrough 的 pending promise 面）。
  "ready-states/autoplay.html"

  "media_fragment_seek.html"
  "autoplay-with-broken-track.html"
  # M3 扩批 XXVIII（2026-09-04）：同文档移动不重置播放（currentTime>=10 保持 +
  # paused=false——pause-move-within-document 同域的 currentTime 面）。fixture
  # 增 movie_300.webm（VP9 300s）。
  # 不导入 audio/video_loop_base（短 fixture 回卷时序不可观测 + VP8 域外——
  # XXIV 注记）；preserves-pitch/src_object_blob（testdriver 音高检测/blob URL 面）。
  "offsets-into-the-media-resource/currentTime-move-within-document.html"
  # M3 扩批 XXVIII 续：track-mode-triggers-loading——metadata track 初始 disabled
  # 不加载，mode 改 hidden 触发（扩批 XV mode 触发面 + VTT 解析面既有；cues 12 条
  # + cue[11].startTime==22 断言）。
  "track/track-element/track-mode-triggers-loading.html"
  # M3 扩批 XXXI（2026-09-04）：removetrack 派发落地后试导——selection metadata
  # mode 初始态面 / removetrack TrackEvent 面 / 播放中 add cue 面。资源
  # webvtt-file.vtt 补入。
  "track/track-element/track-selection-metadata.html"
  "track/track-element/track-remove-track.html"
  "track/track-element/track-cues-missed-no-immediate-events.html"
  # M3 扩批 XXXII（2026-09-04）：readyState/cue-mutable/mode 稳态面批量试导
  #（readyState ERROR/LOADED 断言、VTTCue 属性可变面、src='' 空 URL error 面）。
  # track-mode-not-changed-by-new-track 维持排除——textTracks 增量同步的 track
  # 身份对拍（getElementById(x.track.track) same-object 面，身份表切片）；
  # track-remove-insert-ready-state 维持排除——remove→re-attach 后的播放推进链
  #（期望 timeupdate 计数 3 got 0，removal 面与推进泵耦合切片）。
  "track/track-element/track-cues-cuechange-dynamically-created-track-element.html"
  "track/track-element/track-disabled-addcue.html"
  "track/track-element/track-insert-after-load.html"
  "track/track-element/track-load-error-readyState.html"
  "track/track-element/track-load-from-element-readyState.html"
  "track/track-element/track-cue-mutable.html"
  "track/track-element/src-empty-string.html"
  # M3 扩批 XXXIII（2026-09-04）：TextTrackCueList 功能面（length/[]/getCueById）。
  # track-cue-mutable-fragment 维持排除（captions-html.vtt cue 标记树 isEqualNode
  # 对拍——深结构项）；track-selection-task-order 维持排除（selection 宏任务序）。
  "track/track-element/track-text-track-cue-list.html"
  # M3 扩批 XXX 续：mode/cuechange 播放推进面（B 组——headless 时钟 + march 基建）。
  # 资源 captions-fast.vtt/cues-chrono-order.vtt/captions-gaps.vtt 已在清单。
  # counting.webm/test.webm 已在 MEDIA_FILES。track-mode 维持排除——mode 数值
  # setter 回落 + cue 计数 done 链（独立语义切片，随 mode setter 精化复评）。
  "track/track-element/track-mode-disabled.html"
  "track/track-element/track-cues-cuechange.html"
  "track/track-element/track-cues-add-new-track.html"
  # M3 扩批 XXX（2026-09-04）：WebVTT 解析面批量导入——track-helpers.js 断言辅助
  #（getCueAsHTML textContent 对拍）+ BOM/UTF8（iconv 编码拒收）/header 注释/
  # 恢复（cue-recovery 三形态）/空 cue/timings 变体（no-hours/whitespace/large/
  # negative-duration）/markup（voice/class/unsupported/timestamp 标签）/退化形态。
  # positioning/layout 渲染件维持 C 组排除；markup 结构族（voice/class/markup/
  # unsupported-markup/timestamp/cue-recovery-header）维持排除——assert_cue_fragment
  # isEqualNode 对拍 getCueAsHTML 的 span 节点树（<v>/<c> → className/title）——
  # cue 标记树解析深结构项（归渲染域远期，与既有注记一致）；
  # track-cue-empty 维持排除——fragment.childNodes[0].constructor.name === 'Text'
  # 原生 class 语义断言（shim 工厂 constructor.name='Node'，非行为缺陷）。
  "track/track-element/track-webvtt-bom.html"
  "track/track-element/track-webvtt-utf8.html"
  "track/track-element/track-webvtt-header-comment.html"
  "track/track-element/track-webvtt-interspersed-non-cue.html"
  "track/track-element/track-webvtt-no-timings.html"
  "track/track-element/track-webvtt-cue-no-id.html"
  "track/track-element/track-webvtt-degenerate-cues.html"
  "track/track-element/track-webvtt-empty-cue.html"
  "track/track-element/track-webvtt-newlines.html"
  "track/track-element/track-webvtt-timings-no-hours.html"
  "track/track-element/track-webvtt-timings-whitespace.html"
  "track/track-element/track-cue-negative-duration.html"
  "track/track-element/track-large-timestamp.html"
  # M3 扩批 XXVIII 续二：track-remove-quickly / -by-setting-innerHTML——track 移除
  # 不 crash smoke 面（innerHTML 注入 + seeked 计数链 + innerHTML 清空后再 seek）。
  "track/track-element/track-remove-quickly.html"
  "track/track-element/track-remove-by-setting-innerHTML.html"
  # 不导入 track-element-src-change-error：stage3→4 依赖「加载中移除 src」的
  # in-flight 中断时序——headless settle 同步完成（microtask）无 in-flight 窗口，
  # settings.vtt 的 onload 恒先于 removeAttribute 到达（onload case4 unreached）。
  # 不导入 track-element-src-aborted-load：WPT 服务器 trickle(d3600) pipe 机制
  #（pending 加载模拟），runner 无 HTTP 服务器不可复现。
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
  # M3 扩批 XL（2026-09-04，排除注记复核 + 上游核查）：candidate-remove-no-listener
  #（上游 edge 1/1 绿——「移除候选不 crash」smoke 面）+ invoke-pause-networkState
  #（上游 1/1 绿——data:, 两段 settle 落地后「pause() 不重触发资源选择」断言面）+
  # load-events-networkState（上游 4/4 绿——load() invoke 步 5 abort/emptied/timeupdate
  # 排队派发落地）。pointer 全 7 件 / candidate-moved / candidate-remove-onerror /
  # candidate-remove-addEventListener 维持不导入——上游核查（wpt.fyi 2026-09-04 master
  # run，edge=Chromium 内核）全红/Timeout（crbug 593289「await a stable state」族——
  # 无 src source 的指针 error 语义 Chromium 自身未实现），坏用例不导入。
  "resource-selection-candidate-remove-no-listener.html"
  "resource-selection-invoke-pause-networkState.html"
  "load-events-networkState.html"
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
  # track-change-event（M3 扩批 XXI 已实施反向链 + change 广播——已导入白名单）。
  # 此前不导入：change 事件广播需 TextTrackList↔TextTrack 反向链 +
  # 模式变更前 list 未暴露的 pending 重放（深结构）——2026-09-03 扩批 XXI 收口。
  "track/track-element/vtt-cue-float-precision.html"
  "track/track-element/track-change-event.html"
  "track/track-element/track-disabled.html"
  "track/track-element/no-cuechange-before-play.html"
  "track/track-element/track-active-cues.html"
  "track/track-element/track-remove-active-cue.html"
  "track/track-element/resources/captions-fast.vtt"
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
  # M3 扩批 XXX 解析面波次：track-helpers.js + WebVTT 解析深化用例资源集
  #（BOM/UTF8/注释/恢复/空 cue/timings 变体/markup/voice 等 27 件）。
  "track/track-element/track-helpers.js"
  "track/track-element/resources/bom.vtt"
  "track/track-element/resources/class-bad.vtt"
  "track/track-element/resources/class.vtt"
  "track/track-element/resources/cue-no-id-error.vtt"
  "track/track-element/resources/cue-no-id.vtt"
  "track/track-element/resources/cue-recovery-cuetext.vtt"
  "track/track-element/resources/cue-recovery-header.vtt"
  "track/track-element/resources/cue-recovery-note.vtt"
  "track/track-element/resources/degenerate-cues.vtt"
  "track/track-element/resources/empty-cue.vtt"
  "track/track-element/resources/interspersed-non-cue.vtt"
  "track/track-element/resources/iso2022jp3.vtt"
  "track/track-element/resources/large-timestamp.vtt"
  "track/track-element/resources/markup-bad.vtt"
  "track/track-element/resources/markup.vtt"
  "track/track-element/resources/metadata-area.vtt"
  "track/track-element/resources/no-newline-at-eof.vtt"
  "track/track-element/resources/no-timings.vtt"
  "track/track-element/resources/timestamp-bad.vtt"
  "track/track-element/resources/timestamp.vtt"
  "track/track-element/resources/timings-no-hour-errors.vtt"
  "track/track-element/resources/timings-no-hour.vtt"
  "track/track-element/resources/timings-whitespace.vtt"
  "track/track-element/resources/unsupported-markup.vtt"
  "track/track-element/resources/utf8.vtt"
  "track/track-element/resources/voice-bad.vtt"
  "track/track-element/resources/voice.vtt"
  "track/track-element/resources/simple-captions.vtt"
  "track/track-element/resources/captions-gaps.vtt"
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
  # M3 扩批 XXVIII：currentTime-move-within-document 媒体源（VP9 300s 长流——
  # seek(10) 后持续推进面）。
  "media/movie_300.webm"
  "media/sound_5.mp3"
  "media/test.webm"
  "media/test-1s.webm"
  "media/counting.webm"
  # M3 扩批 XXIV：audio_loop_seek_to_eos 媒体源（getAudioURI 判定 audio/ogg → .oga；
  # vorbis → symphonia 面；sound_0.mp3 为对照/复用源）。
  "media/sound_0.mp3"
  "media/sound_5.oga"
  # M3 扩批 XXXV：video-loading-eager 媒体源（loading=eager 立即加载 loadeddata
  # 到达断言面；trickle(d2) pipe 参数 runner 无服务端 pipes，源字节直读即时 settle
  # = eager 语义即本实现形态）。
  "media/A4.webm"
  # M3 扩批 XXXVI：audio-loading-eager 媒体源（同 XXXV 面——audio eager 立即加载
  # loadeddata 到达）。
  "media/sine440.mp3"
  # M3 扩批 XXXIX：audio_loop_base 媒体源（getAudioURI 判定 audio/ogg → .oga；
  # 0.05s 短流——loop 回卷 seeking 二次派发面）。
  "media/sound_0.oga"
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
