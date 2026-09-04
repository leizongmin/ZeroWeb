# M3 扩批过程记录归档（event_* 族 ~ 第 XVIII 批）（只追加不修改）

**入口文档**: [../media-elements.md](../media-elements.md) | **控制面**: [../master.md](../master.md)
**归档日期**: 2026-09-04（治理切片——master.md 当前状态块的批次明细移入本档：2026-09-01 扩批族（event_*/第二批/III~X）+ 2026-09-02 批（XI~XV）+ 2026-09-03 批（XVI~XVIII）。第 XIX~XXXIX 批明细在 master.md 头链（最新态）；每批证据 JSON 在 evidence/ 不动；累计口径 603P/0F/24PF = 96.17%）

---

## 2026-09-01 扩批族（event_* 族 + 第二批 + III~X，原文）

**M3 扩批已落地（2026-09-01，event_* 族）**：下游计划 #1 兑现——25 个 event_* 用例
（autoplay + noautoplay + order_* 五件）接入 MEDIA_TEST_FILES / fetch-media-subset.sh。
支撑语义补齐（part03.js / part06.js）：
- `play()` 返 **pending** Promise（spec：resolve 于播放真正推进）——setTimeout(0) 后仍
  playing 则 resolve；`pause()` 先行 → reject **AbortError**（spec「pause() rejects all
  pending play Promises」，event_play_noautoplay 断言面）；already playing → resolved。
- `_zwMediaLoadSequence` 增 tag 参数：video 派 **resize**（durationchange 后、
  loadedmetadata 前，event_order_durationchange_resize_loadedmetadata 断言面；audio 无）。
- autoplay 与 play() 路径均补派 **timeupdate**（event_timeupdate* 断言面）。
单测 `test_audio_constructor_and_media_methods_r2835` 扩展 pending→pause→reject 链 +
already-playing resolved 断言。**88.3%**（324P/0F/2T/41PF，+80 subtest 全绿）。evidence：
`evidence/2026-09-01-media-event-family.json`。make test 65 套件 18555 全绿、fmt/clippy 干净。

**M3 扩批第二批已落地（2026-09-01，余账 2T 清零）**：
- **source-child 资源选择触发**（appendChild 钩子，part04.js）：source 子插入 audio/video
  父（handle 父、无自身 src、资源未 settle）→ `_zwMediaScheduleLoad` 以 source 的 src
  调度加载序列——父元素 loadstart + currentSrc 真值化（WPT currentSrc「adding source
  element」族 8 subtest）。
- **空 src 错误码语义**（`_zwSettleResourceKey` 增 errorCode 参数）：空 src 资源选择失败
  → error 事件 + `error.code = MEDIA_ERR_SRC_NOT_SUPPORTED(4)` + MediaError 实例 +
  currentSrc 恒空（WPT error-codes「empty string」族 4 subtest）。
- **source.src URL 绝对化**（get trap 增 SOURCE 分支）：URL 属性反射 + base 解析 +
  C0/space 剥离（同 track.src / a.href 模式；WPT currentSrc 断言 `e.currentSrc === s.src` 面）。
单测 `test_media_source_child_and_error_code_r391`。**89.1%**（334P/0F/**0T**/41PF，
+10 subtest 全绿，Timeout 清零）。evidence：`evidence/2026-09-01-media-source-child.json`。

**M3 扩批 III 已落地（2026-09-01，volume/muted + Audio 构造器）**：
- **volume IDL setter spec 语义**（part05）：非有限数值 → TypeError（spec dom-media-volume
  步 2，volume_nonfinite 断言面）；同值短路不派事件；volumechange 改 **queued** 派发
  （media element task source——同 turn 内后注册 handler 也收到，event_volumechange
  「repeatedly fires」断言面）；`_mediaState[key].pendingVc` 标记 + **load() 清除**
  （spec dom-media-load「pending events 丢弃」——「before load will not fire」断言面）。
- **volume getter 缺省守卫**（part04）：`_mediaState` entry 可被 muted setter 先建
  （volume 字段未写）→ `!= null` 守卫防 undefined 漏出（1-volume=NaN→TypeError）。
- **muted IDL setter + getter**（part04/part05）：setter 增分支（此前落 expando 吞）——
  现值读法与 getter 同源（dirty 镜像优先/attr presence 回落，attr 已设时 `e.muted=true`
  值未变不派）；值变 → attr 反射同步 + queued volumechange；has-trap 白名单补 'muted'。
- **Audio 构造器 spec 面**（part02/part01b/part03）：设 preload='auto'（spec dom-audio）+
  无 new 调用抛 TypeError（WebIDL constructor 语义）；HTMLAudioElement 接口构造器
  Illegal constructor + prototype 链补接（audio_constructor 断言面 11 subtest 全绿）。
- R2835 旧断言「Audio() 无 new 亦返 proxy」与 spec 冲突——随用例导入一并修正。
- 导入 volume_nonfinite.html / event_volumechange.html / the-audio-element/
  audio_constructor.html（fetch 脚本 + MEDIA_TEST_FILES 同步）。
- 单测 `test_media_volume_muted_semantics_r392`（6 断言组）。**90.0%**
  （369P/0F/0T/41PF，+35 新通过 0 回归）。evidence：
  `evidence/2026-09-01-media-volume-audio.json`。

**M3 扩批 IV 已落地（2026-09-01，controlsList + the-video-element 起步）**：
- **controlsList IDL**（part03/part04/part05）：`_classListProxy` 增 supportedTokens
  参数——`supports(token)` 精确匹配四值表（nodownload/nofullscreen/noplaybackrate/
  noremoteplayback，大小写敏感；未传表恒 false，spec 其它 DOMTokenList 无 supported
  tokens 定义）；get trap 增 controlsList 分支（gate：HTML ns 的 audio/video，R374
  gate-miss → undefined 同款）；has-trap 白名单补列。
- 导入 controlsList.tentative.html（2 subtest）+ the-video-element/video-tabindex.html
  （1 subtest，UA 面不凭空加 tabindex）。
- 单测 `test_media_controls_list_r393`（6 断言组）。**90.1%**（372P/0F/0T/41PF，
  +3 新通过 0 回归）。evidence：`evidence/2026-09-01-media-controlslist.json`。
- **决策记录**：audio_volume_check/video_volume_check **不导入**——越界值期望
  IndexSizeError 是 Intel 旧 spec 时代断言，与现行 spec（clamp 不抛）及已导入的
  volume_nonfinite.html 冲突，导入即制造假失败面。

**M3 扩批 V 已落地（2026-09-01，playbackRate 非有限 TypeError）**：
- `playbackRate`/`defaultPlaybackRate` IDL setter：非有限数值 → TypeError
  （spec dom-media-playbackrate / dom-media-defaultplaybackrate 步 2；旧静默回落 1
  与 volume TypeError 修复同款缺口）。合法值照常设置 + ratechange 派发不变。
- 上游 playbackRate.html 断言面不含非有限值（7P 全绿零回归）——纯 spec 对齐修复，
  无新用例可导入（WPT 无对应断言用例）。单测
  `test_media_playback_rate_non_finite_r394`（3 断言组）。evidence：
  `evidence/2026-09-01-media-playbackrate-typeerror.json`（90.1% 维持）。

**M3 扩批 VII 已落地（2026-09-01，pause-on-removal + play() queued task 化）**：
- **pause-on-removal**（part04 removeChild sel 路径）：播放中 media 元素移除文档 →
  两段 defer 对齐上游观察序——tick1 置 `paused=true`（afterStableState 的 volumechange
  回调内「paused after stable state」断言可观测）、tick2 派 pause 事件（回调内挂的
  onpause 仍收到——「paused in pause event」断言面）；`removedPauseFired` 幂等防重复；
  重插文档不自动续播。
- **play() 事件 queued task 化**（part03）：play/playing/timeupdate 从同步派发改
  setTimeout(0)（spec dom-media-play「queue a media element task」）——play() 返回后
  注册的 onplaying/listener 仍收到；paused 状态翻转保持同步（spec dom-media-paused）；
  无 setTimeout 环境回落同步（零回归面）。
- 导入 playing-the-media-resource/pause-remove-from-document.html；单测
  `test_media_pause_on_removal_m3b7`（时序/幂等/重插 5 断言面）。**93.5%**
  （387P/0F/0T/27PF，387/414；+1P 0 回归）。

**M3 扩批 VIII 已落地（2026-09-01，about: src 失败路径 + play promise 竞态修复）**：
- **about: src 资源选择失败**（part06 `_zwMediaScheduleLoad`）：非空 about:（about:blank
  等）src → 资源获取不产出可播媒体资源 → error 事件 + code 4（同空 src 面）。
  导入 the-video-element/video_crash_empty_src.html（2 subtest 全绿）。
- **play() promise 单 task settle**（part03）：play Promise 的 settle 与事件派发合并
  同一个 queued task——宿主 `__zw_setTimeout` 每定时器独立线程投递，两个 0ms 定时器
  顺序不保证，竞态下 promise-check 先跑会误 resolve、丢 AbortError 契约
  （event_play_noautoplay 全量 2 Fail、FILTER 单跑不可复现的调度非确定性根因）。
  task 内序：先派事件（listener pause() 同步 reject）→ 后查 playPromise 身份 → resolve。
  **教训**：全量跑与 FILTER 跑不一致 = 定时器竞态信号，不是用例间状态泄漏。
- **389P/0F/0T/27PF（389/416 = 93.5%）四连跑稳定**；engine 2544 全绿。

**M3 扩批 IX 已落地（2026-09-01，adopt/move 守卫）**：
- pause-on-removal hook tick1 增 `_zwIsRemovedNode` 检查——被 appendChild 移动
  （adopt）的元素仍 related → 不置停不派 pause（spec「pause on removal」限定
  移除文档面）。导入 pause-move-within-document.html（Pass）。
- **决策注记**：pause-move-to-other-document 不导入——跨 iframe 文档 adopt 在
  shim 融合视图下 appendChild 静默落空（元素保持 detached → 暂停语义正确但
  用例期望不暂停），实施需 iframe 文档 adopt 面（深结构，待用户点名方向）。
- **390P/0F/0T/27PF（390/417 = 93.5%）**；engine 2546 全绿。

**M3 扩批 VI 已落地（2026-09-01，preload setter 补缺 + sweep 巡检收口）**：
- `preload` IDL setter：enumerated 反射（写 preload 内容属性原样值——invalid 原样写、
  getter 归一 'metadata' 分离面；DOMString 非 nullable，null→'null' 串）。旧无 setter
  分支 → 落 expando 吞、attr 不写 → set→get round-trip 断。
- **全 IDL setter sweep 巡检**（探针实证）：controls/loop/autoplay/playsInline/
  crossOrigin/defaultMuted/muted/currentTime/volume/track.kind/label/srclang/src/
  media.src 全部 round-trip 正确——语义面 setter 缺口清零，巡检收口。
- 单测 `test_media_preload_setter_roundtrip_r395`（2 断言组）。evidence：
  `evidence/2026-09-01-media-preload-setter.json`（90.1% 维持 0 回归）。

**M3 扩批 X 已落地（2026-09-01，track 子 ↔ textTracks 集合同步）**：
- **共享工厂 `_zwTextTrackForElement`**（part01）：由 track 元素构造/取回关联
  TextTrack（身份缓存 `_elementTextTrack` 同表）——`track.track` get trap 与集合同步
  共用；新增 `id` 反射（readonly accessor——赋值被吞，track-id 断言面）。
- **集合同步 `_zwSyncTextTracksFromChildren`**（part01）：全量重建 track 子段
  （树序）+ `entry.manual` 保 addTextTrack 产物（添加序保尾）——spec
  text-tracks-in-media-elements 的列表模型；list 对象身份不变（same object）。
  子视图：handle 父读 `_handleChildren` registry（detached createElement 形态），
  sel 父走 `_childNodeList` 融合视图（静态 HTML 形态）。
- **同步钩子三路**（part04）：appendChild（SOURCE hook 相邻）、removeChild
  （handle 父 + sel 父双分支）、innerHTML 整体替换（`_handleChildren` 本地子视图
  更新后、`_mo_notify` 前）。
- **textTracks getter 接通**（part03）：返回前先同步（首读含既有 track 子）。
- 导入 track/track-element 6 用例（track-api-texttracks / track-addtrack-kind /
  track-texttracks / track-node-add-remove / track-id / track-element-dom-change，
  fetch 脚本逐文件白名单——目录整体含 VTT 解析面不整目录导入）。
- 单测 `test_media_track_texttracks_sync_m3x`（append/顺序/remove 身份/getTrackById+
  id readonly/innerHTML 清空 5 断言组）。**400P/0F/0T/25PF（400/425 = 94.1%）**
  （+8 subtest 全绿 0 回归）。evidence：`evidence/2026-09-01-media-track-sync.json`。
  make test 66 套件 18662 全绿、fmt/clippy 干净。

## 2026-09-02 批（XI ~ XV，原文）

**M3 扩批 XI 已落地（2026-09-02，resource selection 算法面——Mission 覆盖范围第 1 条
「load 算法（resource selection algorithm）」的正题收口）**：
- **背景**：上一轮 session 被限额打断前抓取的 6 个 resource-selection 探针揭示
  「可导入面吃尽」判断漏了 loading-the-media-resource 资源选择族（47 文件）；本轮
  拉全族分类——27 案 headless 可行 / 20 案排除（真资源失败时序 / MSE / iframe /
  manual / data:, 契约冲突）。
- **同步段语义**（`_zwMediaResourceSelect`，part06）：invoke（资源选择启动）后阻塞
  期间 networkState = NETWORK_NO_SOURCE(3)；稳定态续段（queueMicrotask——V8 execute
  末 checkpoint 排空，等价「当前 task 末」）重查候选，无 src 且无 source 子 →
  NETWORK_EMPTY(0)，有候选 → LOADING(2)。
- **加载模拟改 microtask**：`_zwMediaScheduleLoad` 的 settle 续段从 host 真实线程
  setTimeout 改 queueMicrotask（同检查点排空）——消除「loadstart 先于 window load」
  与后续脚本执行的竞态假失败（**教训**：host __zw_setTimeout 真实线程投递 ~0ms 与
  脚本执行竞态，全量跑与 FILTER 单跑不一致的又一形态）。
- **settle 触发时重验候选**（spec 同步段「await a stable state」续段语义）：src 属性
  移除/改写或 source 子被移除 → 加载中断（不派 loadstart/不 settle）；
  resource-selection-remove-src/-remove-source/-resumes-onload 断言面。
- **source 子候选失败语义**：error 派在 source 元素上（onerror），父级 error 仅全
  候选耗尽后；父级不落 _resourceStates（等待下一候选）。
- **media load invoke 播放中止**（spec dom-media-load 同步段）：paused 同步置 true +
  pending play promise reject AbortError + timeupdate/pause 派发；
  invoke-set-src-networkState 断言面。
- **load() 重跑算法**：`_resourceStates` 重置 + 现行 src（含空串失败面与
  `lastSourceChild` 重试）重调度；`loadEpoch` 纪元使 loadstart handler 内 load() 作废
  本续段余下事件（[loadstart,loadstart,error] 序断言面）。
- **加载序列尾部补 suspend**（「once the entire media resource has been fetched」
  ——headless 全量已取语义；load-removes-queued-error-event 断言面）。
- **setAttribute('src') HTML-ns 钩子**（part04）：内容属性路径 invoke media load
  （IDL src= setter 同语义；setAttributeNS 非 HTML ns 不触发）。
- **appendChild source 钩子补强**：仅 HTML ns source 是候选（createElementNS('bogus',
  'source') 非候选）；source 子自身无 src 时仍触发父资源选择（loadstart 面）。
- **单测**：`test_media_resource_selection_m3xi`（invoke 面 / load 中止 / AbortError /
  load 重置 4 断言组）；r389 / m2a 既有单测同步 microtask 模型（`__zw_timers` 泵不再
  参与 headless 加载面）。
- 导入 loading-the-media-resource 25 文件（fetch 脚本逐文件白名单 + MEDIA_TEST_FILES
  同步）。**430P/0F/0T/25PF（430/455 = 94.5%）**（+30 subtest 全绿 0 回归）。evidence：
  `evidence/2026-09-02-media-resource-selection.json`。make test 66 套件 18665 全绿、
  fmt/clippy 干净、reftest 687/687。

**M3 扩批 XII 已落地（2026-09-02，TextTrack 家族接口语义面——上一轮 session 被限额
打断后的续接收口）**：
- **背景**：限额前 WIP（shim 六 part + MEDIA_TEST_FILES 28 用例）处于未提交态且
  9 Fail；本轮续接完成——TrackEvent 缺失 / cues-activity gate 非对称 / 列表索引
  own-property 不可见 / prototype 链序 四类根因修复。
- **VTTCue 构造器**（part01b）：脚本创建 cue 唯一入口（与 TextTrackCue 分离接口）；
  startTime setter NaN/±Inf → TypeError、endTime NaN/-Inf → TypeError 而 **+Inf 合法**
  （无末尾 cue——dom-vttcue-endtime 断言面）；id DOMString null→'null'；pauseOnExit；
  track readonly；onenter/onexit 初值 null（赋 undefined → null）；EventTarget 最小派发面
  （per-cue 内表 `_zwEnsureEventTarget`）。
- **TextTrack addCue/removeCue/cues**：cue 已属它 track 先移除；removeCue 未列入 →
  NotFoundError；cues 按 startTime 升序动态重排（tie：end 大者在前→添加序——「changing
  order」断言面，setter 即时重排经 `_zwInvalidateCues`）；getCueById 空串恒 null；mode
  枚举归一 setter（invalid 保留旧值）；disabled → cues/activeCues null。
- **gate 非对称修正（本轮根因 #1）**：cues=readiness gate（track 资源未 settle 恒 null
  ——cues「default attribute」断言面），activeCues=**仅 mode gate**（spec
  dom-texttrack-activecues 只关 disabled——detached track 未 settle 亦非 null，此前
  readiness 同 gate 致 activeCues 6 断言全断）。
- **track 检索触发不限 connected（根因 #2）**：part04 appendChild 钩子移除 `_elConnected`
  门——spec「further handling of the track element」随 parent media element 的 relevant
  mutation（track 插入）queue，detached video 亦触发（cues.html「default attribute」经
  onplay 时 settle 完成断言非 null）。
- **TextTrackList/TextTrackCueList 索引 own-property 镜像（根因 #3）**：列表经索引只读
  Proxy（set trap 恒 false 保「no indexed set/create (strict)」TypeError 面），但
  assert_array_equals 走 hasOwnProperty → get trap 不可见；补 `_zwSyncListHolder` 在
  **target** 上镜像索引 own props（页面经 proxy 写仍被拦——traps 与 own props 正交）。
- **TrackEvent 构造器（根因 #4）**：type + init dict {track}；track readonly accessor
  （赋值被吞）；prototype **惰性链接** Event（本 part 装载早于 part05 的 Event 定义——
  装载期链接落 Object.prototype 致 instanceof Event 断败，首次构造时补链）；createEvent
  ('TrackEvent') 不入 part06 legacy map → NotSupportedError（non-createable 断言面）。
- **data:text/vtt 解析**（part06 `_zwParseVttDataUrl`）：BOM/WEBVTT 头 + cue 块（id 行+
  时间行+文本）→ VTTCue 填入关联 TextTrack + settle（`_zwTrackScheduleLoad` microtask
  幂等）；track.src IDL setter / setAttribute('src') 双路触发。
- **导入**：interfaces/ 28 文件（TextTrack 10 + TextTrackCue 8 + TextTrackCueList 3 +
  TextTrackList 5 + TrackEvent 2；fetch 脚本 INTERFACES_FILES 白名单 + MEDIA_TEST_FILES
  同步）。**496P/0F/0T/25PF（496/521 = 95.2%）**（+66 subtest 全绿 0 回归，PF 25 为
  canPlayType optional 组既存面）。evidence：
  `evidence/2026-09-02-media-texttrack-family.json`。engine 2556 全绿、fmt/clippy 干净。
- 单测 `test_media_text_track_cue_face_m3xii`（VTTCue TypeError 面 / 排序+镜像+strict
  TypeError+getCueById / removeCue NotFoundError+mode gate / gate 非对称 / TrackEvent+
  on* 派发 / data:text/vtt settle 六断言组）。

**M3 扩批 XIII 已落地（2026-09-02，TextTrack cue 选项 + 列表增量事件面）**：
- **VTTCue 定位选项 IDL**（part01b）：vertical（''/rl/lr 枚举）、snapToLines、
  line/position（double or 'auto'）、size（clamp [0,100]）、align 五值枚举——headless
  仅存储不做视觉布局（vtt-cue-float-precision / track-add-remove-cue 断言面）。
- **data:text/vtt 加载**（track-data-url 3 subtest 全绿）：onload → cue 值断言
  （startTime/endTime/id/text），crossorigin 三态不区分（headless 无 CORS 模式面）。
- **readyState settle 面修正**：旧「src 有值 → LOADING(1)」同步分支断
  track-add-track 断言面（`track.src=` setter 同步后立即读期望 NONE）——改只由
  settle 事实驱动（NONE 0 → LOADED 2 / ERROR 3；LOADING 态对脚本不可观察即被终态
  替代）。
- **addtrack 异步派发**（part01 `_zwSyncTextTracksFromChildren` + part03
  addTextTrack 增量段）：以 **list holder 现内容** 为增量基线（appendChild 同步钩子
  已先行记账——entry.tracks 基线首读恒 0 增量），`_zwFireTracksAdded` 经 queueMicrotask
  派 TrackEvent('addtrack', {track})；spec「list changes」queued task。
- **TrackEvent type 面修复**：构造基座从 `_makeEvent`（part03 IIFE 私有——本 part
  作用域不可见，静默回落 {} 丢 type）改 `new globalThis.Event`；dispatchEvent 补
  spec concept-event-dispatch 的 target/currentTarget/eventPhase 设置与复原——
  TextTrackList 事件目标取 **exposed proxy**（holder.self——索引只读 Proxy，
  track-add-track 断言 event.target === video.textTracks 身份面）。
- **src 变更清 cue + 重调度**（part06 `_zwTrackScheduleLoad` 增 srcChange 语义，
  part04/part05 setter 传标记）：`_zwClearCues` 清既有 cue + 重置 settle 面按新 URL
  重跑（spec「track URL 变更」——src-clear-cues 3 subtest 全绿；**首调度前变更**
  同样清——detached track 形态）。
- **cues gate 收窄**（part01b `_cuesGate`）：仅 track 子仍挂 media 父下时 readiness
  gate 生效——detached track（无 media 父）cue list 即可用（spec 轨道不参与 media
  加载循环即「启动完成」）。
- **导入**：track/track-element 5 文件（track-data-url / vtt-cue-float-precision /
  track-add-track / track-cue-order / src-clear-cues）。**510P/0F/0T/25PF
  （510/535 = 95.3%）**（+14 subtest 全绿 0 回归）。evidence：
  `evidence/2026-09-02-media-cue-options.json`。engine 2559 全绿、fmt/clippy 干净。
- **排除注记**：track-change-event 不导入——change 事件广播需 TextTrackList↔
  TextTrack 反向链 + 模式变更前 list 未暴露的 pending 重放（深结构）；no-cuechange-
  before-play / track-cues-* 播放推进族依赖真播放钟（兄弟目标 media-playback）；
  track-webvtt-* 布局/渲染族归渲染域远期。

**M3 扩批 XV 已落地（2026-09-02，http VTT 文件加载 + WebVTT 解析深化——上一批
「可导入面吃尽」判断的第二次修正：track/track-element 目录的 **http VTT 加载族**
此前因「无真字幕抓取通路」整体排除，本批接通同步 fetch 后 12 案全绿）**：
- **http(s) VTT 加载**（part06 `_zwTrackScheduleLoad`）：同步 `__zw_fetch`（R115
  iframe 同款契约）取回 VTT 文本 → 新 `_zwParseVtt` 解析填 cue；`__zwfr:` wire 解析
  取 body；fetch 失败/非 WEBVTT 头 → error settle（track onerror + readyState ERROR
  ——track-webvtt-magic-header no-webvtt 断言面）；无 fetch 宿主（浏览器异步路径）
  回落 headless load 恒派（零回归）。
- **`_zwParseVtt` 解析深化**（part06，data: 版复用 `_zwParseVttDataUrl` 不动）：
  ① header 校验（BOM 剥离后 'WEBVTT' 开头）；② cue id 行**含 '-->' 不识别**（该行
  作坏 timings 丢弃、后续行重新起块——cue-id-error 3 cue 恢复面）；③ cue settings
  （line/position/size/align/vertical：% 剥离 + name 大小写敏感 + `<align:end>` 尖
  括号段容错 + settings 含字面 '-->' 只按前两个 --> 分割）；④ timings 小时位 +
  **mm/ss ∈ [00,59] 范围校验**（timings-hour-error '00:120:00.500' 拒收）；⑤ cue 文本
  tag/annotation 截断（原始 '>' 终点；无原始 '>'（含 '&gt;' 实体）→ '<' 起全吞——
  entities-wrong 断言面）；⑥ **cue.text 保持 parser 原文**（实体不解码——spec；
  track-element-src-change '&amp;' 字面断言），解码发生在 getCueAsHTML()（DOM 面）。
- **getCueAsHTML 最小面**（part01b VTTCue.prototype）：DocumentFragment + 单 text
  node + 实体解码（entities 断言族以 `getCueAsHTML().textContent` 消费；headless 无
  cue 标记树——`<Tag>` 结构解析归渲染域远期）。
- **静态 HTML track 子调度触发面**（parser 创建无 appendChild 钩子——三个入口补跑
  `_zwScheduleChildTrackLoads`）：document `querySelector(All)('track')`（part06 doc
  对象口）+ `track.track` 访问（part04，`_zwScheduleParentTrackLoad` 父向）+
  video.textTracks 首读（part03）。
- **default 属性 mode gate**（`_zwScheduleChildTrackLoads`）：无 default 属性的 track
  子 TextTrack mode 缺省 disabled → **不自动加载**（track-default-attribute 断言
  「只有 default track 派 onload」）；**TextTrack mode disabled→hidden/showing setter
  触发 track URL 处理**（part01b mode setter——enableAllTextTracks 形态，
  timings-hour/magic-header/header-checks 断言面）。
- **src 同值变更不重载**（`_zwTrackScheduleLoad` srcChange 分支前置比较）：新旧绝对
  URL 相等且已 settle → no-op（track-element-src-change stage3「不派 onload、cues
  保持」）；srcChange 重置 trackScheduled 幂等标记（三段变更各派 onload）。
- **window.event（legacy）**（part03 `_dispatchWithBubble`）：dispatch 期间暴露
  `globalThis.event` + 嵌套派发栈式恢复（spec legacy current event；track-default-
  attribute handler 内裸 `event.target` 断言面）。
- **导入**：track/track-element 12 文件（track-add-remove-cue / track-webvtt-cue-
  identifiers / blank-lines / settings / entities / timings-hour / magic-header /
  header-checks / track-cue-negative-timestamp / track-element-src-change /
  track-default-attribute / track-load-from-src-readyState）+ 16 个 VTT 资源 +
  track-helpers.js（fetch 白名单 TRACK_ELEMENT_SUPPORT + MEDIA_TEST_FILES 同步）。
  **528P/0F/0T/25PF（528/553 = 95.5%）**（+16 subtest 全绿 0 回归）。evidence：
  `evidence/2026-09-02-media-http-vtt.json`。make test 18733 全绿、fmt/clippy 干净。
- **单测**：`test_media_http_vtt_loading_m3xv`（http 加载 settle + cue 解析面 /
  非 WEBVTT 头 error settle + src 变更重调度 / mode 触发加载 / cue.text 原文 +
  getCueAsHTML 解码两面分离，4 断言组）。
- **排除注记**（B 组——真播放钟推进，随 media-playback 泵接语义层复评）：track-cues-*
  全族（enter/exit/cuechange/missed/seeking/pause-on-exit/sorted-before-dispatch/
  add-new-track）、track-active-cues、no-cuechange-before-play、track-remove-active-cue、
  track-disabled（timeupdate 推进面）、track-mode-triggers-loading（canplaythrough
  后触发时序）、track-element-src-aborted-load/-src-change-error（abort/error 时序）、
  track-remove-quickly/-by-setting-innerHTML（移除竞态）。C 组（渲染域远期）：
  track-cue-rendering-*、track-css-cue-pseudo-class、track-webvtt-*positioning/layout、
  track-cue-inline。D 组（深结构）：track-change-event（既有记录）。


## 2026-09-03 批（XVI ~ XVIII，原文）

**M3 扩批 XVI 已落地（2026-09-03，track-cues-* 播放推进族——B 组排除清单的
第一次实质解锁，兄弟目标 media-playback fixture-mounted 切片 2 兑现）**：
- **背景**：限额中断前 WIP（M3 扩批 XVI 初版：cue iterator + seek sync + kind
  gate + 延后一拍加载序列）处于未提交态且 1 Fail/2 Timeout；本轮续接收口，
  逐一定位并修复 **5 类根因**，2 用例稳定全绿 + 1 件（enter-exit）竞态 flake
  暂排除。
- **根因 1 — 播放桥后装**：runner 的 `install_playback_bridge()` 在
  `run_page_scripts` **之后**执行——页面脚本 turn 内（headless settle 链）
  canplaythrough handler 同步调 video.play() 时桥不存在 → headless 分支、
  bridgeOn 永不置位。修复：execute_script 空转预热（ensure_sandbox +
  ensure_js_shim）后**先装桥再跑页面脚本**。
- **根因 2 — 动态 .src= 不登记**：runner 只在脚本前 extract 静态标记登记源字节
  ——脚本赋值的 `.src=` 永不登记（bridge.play 恒 false）。修复：runner probe
  循环逐 tick 动态登记（JS 快照 audio/video 的 currentSrc/src → wpt-data 读
  字节 → registry；contains_source 幂等 + Rc 字节缓存免重复 IO；settle 提交随
  首次登记发出）。
- **根因 3 — 桥 src 读纯快照**：shim play() 经 `__zw_get_attr`（纯快照）读
  src——IDL `.src=` setter 的写是 pending mutation（apply 前不入快照）→
  bridgeSrc 空串 → 桥失联。修复：改 `__zw_get_attr_lw`（latest-wins，R2995
  同源语义）。
- **根因 4 — 桥未命中即字节丢失 + play 双调**：registry `play()` 的
  `sources.remove` 前置于 open_webm——解码器构建失败一次即丢字节（重试恒
  no-op）；且 XVI 初版在条件中各调一次桥 play → 桥双调（engine 桥契约测试
  `test_media_bridge_playpath_m2a_5b` 捕获）。修复：registry 改 `sources.get`
  克隆、命中才 remove（单测 `play_miss_retains_source_bytes_for_retry`）；shim
  改 `_hit` 单次调用 + 未命中**退避重试**（setTimeout 0 × 5000 上限，playing 态
  保持，spec play() promise pending 于播放真正推进）。
- **根因 5 — march 采样粗粒度丢 cue**：runner 泵 tick 粒度 ~0.5-1s（execute
  脚本开销），1ms cue（missed-cues.vtt）整体跳过；且前进大跳被 >250ms 启发式
  误判为 seek → 清 active 跳过整窗。修复：**区间捕获 + 事件时间序派发**——跨
  (lastMs, nowMs] 收集 enter@start / exit@end 事件按时间排序派发（上游
  missed-cues 期望 enter,exit 交错对）；seek 判据改「时钟回退 ∨ seeking 标志」
  （前进跳变是桥真值钟 tick 合并的常态）。
- **ended 面**：桥真值钟走到流末（registry player Ended 态，新 `is_ended` 桥
  面 `__zw_video_is_ended`/`isEnded`）→ active cue 逐个派 exit（倒序 flush）+
  timeupdate + ended（`ms._zwSel/_zwHandle` 身份留档派发；track-cues-missed 的
  onended 断言面）。
- **pending seek 补推**：seek-before-play 时序（currentTime= 早于桥接通）在
  `_zwSeekPendingMs` 留档，桥命中后 seek 落位（spec 播放启动位置 = 请求 seek
  目标；missed 用例 currentTime=5.0 → play 形态）。
- **回退**：XVI 初版的 track 自动加载 kind gate（metadata 排除）回退为 XV 的
  default gate——track-cues-* 各用例 track 子均带 `default` 属性，kind gate 非
  必需且致 track-default-attribute「onload 只派 default track」回归。
- **导入**：track-cues-enter-seeking（+1P）/ track-cues-missed（+1P，含
  cues-chrono-order / missed-cues VTT 资源已在白名单）/ **track-cues-sorted-
  before-dispatch（+1P，XVII 续批复评——区间捕获 + 事件时间序派发直接覆盖其
  「events sorted by time」断言面，连续 3 次全绿；cues-overlapping /
  sorted-dispatch VTT 资源入白名单）**。**532P/0F/24PF（532/556 = 95.7%）**
  （+3 净涨零回归）。evidence：
  `evidence/2026-09-03-media-cues-playback.json`。make test 66 套件全绿
  （engine 2571 / webview 682 / integration 781）、fmt/clippy 干净。
- **续批评估实录（2026-09-03 同日）**：track-cues-seeking / track-cues-
  pause-on-exit 评估后**暂不导入**——前者依赖「逐次 seek 后 activeCues 数量
  重建」语义（当前 seekSync 仅重建 active 集合、无 per-seek 计数窗口）；后者
  的 onexit 内 `assert_true(video.paused)` 依赖 pauseOnExit 中断播放的即时可
  观察时序（当前泵粒度下 exit 与 paused 翻转存在同 tick 交错）。两者根因已
  记录，随泵节拍精化 + seek 面深化复评。

**M3 扩批 XXIII 已落地（2026-09-03 续，media load invoke 重置面——B 组末件
track-active-cues 解除排除）**：
- **根因定位**（dormant 插桩 `__zwPauseWatch` + `ZW_MEDIA_SEEK_DEBUG` runner 门控
  实证，验证后移除）：扩批 XXII 后复评仍 Timeout——`eventCount==3` 置 `src=''`
  后见 LOAD-INVOKE-STOP 而 settle-dispatch video error 缺席。两个独立根因相互
  掩蔽：**A** `_zwMediaScheduleLoad` 二次调度未重置 `_resourceStates[key]`——
  settle 幂等门「每资源只 settle 一次」吞掉失败候选的 error 提交（`delete` 只在
  IDL load() 与 src 移除分支存在）；**B** settle 的 load/error 派发传
  `_dispatchWithBubble(key, sel, null, ev)`——handle-only（createElement）元素
  listener 键恒失配 0 命中，`_zwMediaFire` 的 on\* expando 兜底才是唯一可达通路
  而 settle 未走它。
- **实施**（shim part06）：① invoke 入口 `delete _resourceStates[key]`（spec
  资源选择 invoke 步「await a stable state」前资源状态归零；IDL load() 已先行
  清——统一到调度入口，src= setter / setAttribute 路径同语义）；② invoke 步 6
  位置重置——`readyState>=1` 时 `currentTime=0` + `_zwMediaTimeKnown=false`
  （spec「set the current playback position to 0 ... HAVE_NOTHING」；activeCues
  headless gate 复位）；③ invoke 重置 track 子产物 cue（`_textTracksCache[key]`
  中 `_zwOwnerEl` 在位者 `_zwClearCues()`——cue@0-5 在位置 0 仍合法 active，须清
  cue 才空；**addTextTrack 产物排除**——无 URL 面不随 media load 重置，
  TextTrack/activeCues「video playing」断言面零回归约束）；④ settle 的
  track/audio/video load|error 派发改 `_zwMediaFire`（img/source 等其余 tag 保持
  原路径——source 的 error 派在 source 元素上由 sourceChild 分支自理）。
- **WIP 清理**：上一 session 限额中断前试推的 track settle 续段
  microtask→macrotask 改动经归因与本三案无关（根因 A/B 均在 media 面），已回退
  原 queueMicrotask 模型。
- **导入**：track-active-cues（+1P；FILTER 单跑 3 连跑稳定）。testharness-media
  **540P/0F/24PF（540/564 = 95.7%）**（+1 净涨零回归）。
- 单测 `test_media_load_invoke_reset_face_m3xxiii`（invoke 重置复 settle error
  面 + 位置归零/activeCues 清空面 + audio onerror expando 兜底面，3 断言组）。
  make test 66 套件 18806/0。evidence：
  `evidence/2026-09-03-media-load-invoke-reset-r39xx.md`（+同名 .json）。
- **收口补丁**：invoke 步 6 位置重置改**无条件**（spec 不以 readyState 为前提——
  readyState>=1 门使 error settle 面（readyState 0）的 currentTime 停留
  undefined，IDL 读法违约；sandbox probe 实证后修正）。
- **评估注记（load-events-networkState）**：load() 的 abort/emptied/timeupdate
  队列派发评估后**不导入**——四 subtest 中 NETWORK_NO_SOURCE 依赖 data:, src 的
  「fetch 成功但解码探测失败」两段 settle（既有 data:, 排除项，与已导入
  location currentSrc.html 的 data: loaded settle 断言冲突，两案同夹具互斥）；
  其余三 subtest 虽可绿但整文件 Timeout 不合「已知失败不导入」纪律；
  abort/emptied 机制不落地（无既有消费者——避免无消费者投机面），随两段 settle
  （解码层真失败判定）一并复评。

**M3 扩批 XXI 已落地（2026-09-03 续，TextTrackList change 广播——D 组首个收口）**：
- **实施**：① 反向链回填三处（`_zwSyncTextTracksFromChildren` holder 同步段 /
  textTracks getter list 首建处 / addTextTrack）——`track._zwOwnerList = list`；
  ② addTextTrack **即时建 list**（spec track 创建即属于列表；上游时序
  addTextTrack → mode='showing' → textTracks 首读中 setter 先于首读——惰性建
  list 使 change 广播失联，首版 Timeout 根因）；③ mode setter 有效值变更 →
  `_zwFireTracksChanged(list)` 异步派基础 Event('change')（无 track 属性——
  上游 hasOwnProperty('track')===false 断言面；target=list exposed proxy；
  同值/invalid 不派）。
- **导入**：track-change-event（+1P）。testharness-media **536P/0F/24PF
  （536/560 = 95.7%）**；track 族 52 用例 2 连跑稳定。
- 单测 `test_media_texttrack_list_change_broadcast_m3xxi`（派发面 + 事件形状 +
  同值/invalid 不派 + 再变更再派）。make test 66 套件 18804/0。evidence：
  `evidence/2026-09-03-media-change-event-r39xx.md`（+同名 .json）。

**M3 扩批 XX 已落地（2026-09-03 续，HAVE_NOTHING 期 seek 挂起语义）**：
- **根因**：track-cues-seeking（onseeked 计数链 + activeCues 递增断言）Timeout
  ——track.onload 内 `video.src=` → `currentTime=0.5` 立即执行，动态 src 的
  settle 尚未跑（readyState 0），currentTime setter 的 seek 门（`>=1`）关闭 →
  seeking/seeked 永不派发 → onseeked 永不 → done 永不。旧实现静默丢弃。
- **spec 语义**（seek 步 1）：HAVE_NOTHING 期 seek 不立即跑，但「set the
  default playback start position」——元数据就绪后从该位置起播；Chromium
  可观察面：seeking/seeked 在 loadedmetadata 后照常派发。
- **修复**：part05 setter readyState 0 时挂 `_zwSeekDeferred`（值已写入
  `_mediaState.currentTime`——起播位置即该值）；part06 `_zwMediaLoadSequence`
  的 `readyState = 1` 翻转处消费标记补跑 seek 算法（seeking + seeked 异步
  回落 + seekSync cue active 面），幂等单次。
- **导入**：track-cues-seeking（+1P）。testharness-media **535P/0F/24PF
  （535/559 = 95.7%）**；track-cues 6 用例 3 连跑稳定；关联面回归确认
  （event_* 116P / resource-selection 23P / networkState 10P / currentTime 3P；
  engine 2572 全绿）。evidence：
  `evidence/2026-09-03-media-deferred-seek-r3937.md`（+同名 .json）。

**M3 扩批 XIX 已落地（2026-09-03 续，解码器 EOF 排空缺陷修复——enter-exit
解除排除的正题）**：
- **背景**：XVIII 的 enter-exit 维持排除（宿主观察：march 正常推进但 cue1 enter
  缺席、ended 在媒体时间 3.5s 提前触发——流长 6.035s）。本轮以休眠插桩
  （`__zwPauseWatch`/`__zwMarchDebugHook` 经 console→tracing 通道 + runner 泵
  Rust 侧 `debug_snapshot` eprintln）宿主实证，定位 **4 类根因**（前两类为
  zero-media 解码层缺陷，与 shim 无关）：
- **根因 1 — `VideoDecoder::next_frame` EOF 提前滞留（decode.rs）**：rusty_vp9
  的 hidden/alt-ref 帧（show_frame=0）解码后返 `Again` 不产出——每次消耗一个
  pull 机会但其 pts 帧晚一个 demux 块浮现，形成 ~15 帧（≈0.5s）**输出流水线
  滞后**。demux 耗尽（`Ok(false)`）分支 flush 后仅 pull 一帧即置 `eof=true`，
  后续调用提前返 `Ok(None)`——积压的 14 帧（pts 5558~5990）永不产出（探针
  实证：顺序解码 167 帧/5525 止；pull-all 181 全出；pull-one 终止后
  pre-flush drain +6 / post-flush +9——帧不丢、被 eof 挡住）。**修复**：
  `draining` 中间态——demux 尽只置 draining，残余经 `drain_frame`（Again=
  隐藏帧继续拉、Eof=队列真空才停）逐帧产出，真空才置 eof；seek 双分支同步
  重置 draining。
- **根因 2 — `present_pending` 未来帧消费丢失（player.rs）**：根因 1 修复后
  真实泵节拍（~15ms tick）模拟仍在 position≈3.8s 提前 Ended——循环拉取遇
  `pts > position` 的未来帧时 `get_or_insert(frame)` 把它**返回调用方**
  （渲染后丢弃），时间槽永久丢失；粗 tick 背压下逐 tick 累积使解码器提前
  耗尽。**修复**：`VideoDecoder::un_read(frame)` 队首退回（pending 槽复用）——
  spec ended「currentTime 到达媒体资源末尾」，帧调度不得超越时钟消费时间线。
  修复后模拟 181 帧全呈现、position=6.0 才 Ended。
- **根因 3 — march pauseOnExit 暂停后置（part06）**：上游 pause-on-exit 在
  onexit handler 内同步断言 `assert_true(video.paused)`（spec time-marches-on
  的暂停须 handler 内可观察）；旧实现先派 exit 后置 playing=false。**修复**：
  暂停（含桥 pause）先于 exit 派发，handler 内 `video.play()` 照常续播。
- **根因 4 — pending seek 补推缺 seekSync（part03）**：seek-before-play 时序
  （currentTime=4.0 早于桥接通）在 play() 桥命中后只记 `_zwLastMarchMs` 不跑
  `_zwMediaSeekSync`——起点恰在 seek 目标上的 cue0@4.0-4.5 永不 enter
  （start > lastMs 恒假）。**修复**：同步命中 + 退避重试命中两路均补
  `_zwMediaSeekSync(_pKey)`。
- **导入**：track-cues-enter-exit（+2P）/ track-cues-pause-on-exit（+1P）；
  **track-cues-seeking 评估后维持排除**（video.onseeked 回调内
  `currentTime === seekedCount * 0.5` 逐次 seek 链 + activeCues.length
  计数断言——依赖 seek 事件真值化，随 seek 面深化复评）。
- 单测 `webm_sequential_decode_drains_hidden_tail_frames_r3936`（顺序解码末帧
  pts 贴近容器时长 + eof 幂等面；wpt-data 缺席时跳过的渐进形态）。
- **534P/0F/24PF（534/558 = 95.7%）**（+3 净涨零回归；track-cues 5 用例
  4 连跑稳定全绿）。evidence：
  `evidence/2026-09-03-media-eof-drain-r3936.md`（+同名 .json 机读版——含
  宿主观察归因全链）。

**M3 扩批 XVIII 已落地（2026-09-03 续，注册竞态消除 + march 区间基线修正）**：
- **MediaSourceProvider（webview/runner）**：宿主桥 play 未命中（源未登记）时
  **同步**回调嵌入方取字节补登记后重评一次——消除「重试等下一 probe tick」的
  时序依赖（全套件并行负载下 tick 延迟放大是 enter-exit flake 的放大器）。
  WebView 增 `set_media_source_provider` + `MediaSourceProvider` 类型别名；
  `register_video_bridge_callbacks` 增第三参（tab_worker/renderer 生产路径传
  None 零回归）；runner 注入 wpt-data 字节供给方。
- **march 首拍区间基线**：`lastMs` 未记账时取 **0（播放起点）**而非首个采样
  时刻——旧初始化把首拍捕获区间置空，采样粒度 ~1s 时起点落采样边界的 cue
  （enter-exit 的 cue1@1.0s）被永久跳过。
- **pending seek 的 march 记账**：seek-before-play（pending seek 补推）路径
  同步置 `_zwLastMarchMs = seek 目标`——该路径无 seeked 回调，基线须 = 目标
  （spec seeked missed-cue 语义；修复 sorted-before-dispatch 在源供给方落位后
  暴露的「cue0（4.0-4.5，目标前）误入捕获区间」16≠14 回归，恢复确定性全绿）。
- **enter-exit 维持排除（宿主观察实证）**：march 正常推进（bo=true/last 前进
  至 3.4s）但 cue1 enter 缺席，且 p=false@3.49s 出现未知暂停源；加 dormant
  `setTimeout(7000)` 即确定性全绿——execute_script 预 drain 的 20ms 等待与
  pending_timer 计数的调度耦合是主嫌疑（双通道定时器：宿主线程 rx + probe
  recorded-timer）。随 runner 泵节拍精化复评。
- 532P/0F/24PF 维持（sorted 恢复确定性）；make test 66 套件全绿、fmt/clippy
  零警告。
- **排除注记（本片暂排除）**：track-cues-enter-exit——单跑 1/4 概率 Timeout
  （桥 play 重试命中与 runner 源登记竞态：播放钟推进偶发晚于 case 10s 预算；
  全套件并行负载下复现率上升），随 runner 泵节拍精化（march 采样粒度收敛）
  复评。其余 B 组（track-cues-seeking activeCues 断言面 / sorted-before-
  dispatch / pause-on-exit / add-new-track / no-cuechange-before-play /
  track-disabled / track-remove-*）维持排除，随本片基础设施增量逐件复评。


## 扩批 XL — loading-the-media-resource 尾件清点（2026-09-04）

- 背景：fetch 全量 vs MEDIA_TEST_FILES diff（comm -23）发现 34 件「已 fetch 未导入
  未注记」残留——扩批 XI 排除注记未覆盖全部 fetched 文件（fetch 脚本与导入清单的
  漂移累积）。逐件定性 + 上游核查（wpt.fyi api/search，2026-09-04 master run，
  edge=Chromium 内核逐件比对 chrome/firefox/safari 无数据时以 edge 为准）。
- 三件解除排除导入（+6 净涨，609P/0F/24PF = 96.21%）：
  1. resource-selection-candidate-remove-no-listener——上游 1/1 绿，本地零改动即绿。
  2. resource-selection-invoke-pause-networkState——data: 媒体候选**两段 settle**：
     一段 loaded（loadstart/currentSrc 面），二段（再一 queueMicrotask 续段）
     「failed with media resource」：error 派发 + code 4 + networkState NO_SOURCE；
     不重置 currentSrc（与 failed with attribute 异——location currentSrc data:,
     断言面零回归）；_zwMediaLoadSequence 入口 gate（error 覆盖后 setTimeout
     过期加载序列作废）。
  3. load-events-networkState——load() invoke 步 5 abort/emptied/timeupdate
     **排队**派发：queueMicrotask 续段（load() 同步返回时未派——「events should be
     fired in queued tasks」断言面；先于新加载序列的 setTimeout loadstart）；
     LOADING/IDLE → abort、非 EMPTY → emptied、旧位置非零 → timeupdate（判定先于
     位置归零）；epoch 门（load() 重跑丢弃旧排队任务）。
- 上游亦红维持排除（**坏用例不导入**——Chromium oracle 口径下导入即恒假失败；
  与 XXIV video_loop_base「结构互斥」不同类的定性）：resource-selection-pointer-*
  全 7 件（control/insert-source/insert-br/insert-text/remove-source/
  remove-source-after/remove-text）+ candidate-moved + candidate-remove-onerror
  ——edge 全红/Timeout（crbug 593289「await a stable state」族：无 src source
  是否派 error 的指针语义 Chromium 自身未实现，各引擎行为分歧）；candidate-remove-
  addEventListener——上游无数据 + 本地 Timeout。
- 其余定性（维持排除，注记已在 fetch 脚本/testharness.rs）：currentSrc（MSE/Blob/
  MediaStream 断言面）；source-media-env-change（iframe + promise 编排）；
  stable-state-print/dialogs/beforeunload-manual（print()/对话框交互面，非自动化）。
  media="not all" 静态面（source-media.html 上游 1/1 绿）——本地 media 属性匹配器
  未实施（网络眉题外），随媒体查询域复评。
- 收口：**loading-the-media-resource 目录 47 文件全数定性**（导入 28 / 上游红或
  深结构排除 19），fetch 与导入清单的漂移归零（新增 3 件已同步 RS_FILES 白名单）。
- shim 面共 3 处小改（part06.js：data: 二段 settle + 加载序列 gate + invoke 步 5
  排队）；make test 全绿、clippy/fmt 干净。evidence：
  evidence/2026-09-04-media-loading-xl.json。

## 扩批 XLI — 上游核查第二轮：track-element/playing 余件（2026-09-04）

- 背景：XL 轮 diff 出的 34 件残留中 track-element 8 件 + playing 2 件逐件上游核查
  （wpt.fyi api/search，edge=Chromium 内核 master run）后试导/定性。
- 解除排除导入（+7 净涨，616P/0F/24PF = 96.25%）：
  - playing-the-media-resource/playbackRate——第 35 件漂移件（文件早已 fetch，从未
    入 MEDIA_TEST_FILES 也未注记）；上游 edge 7/7 绿；本地零改动 7 子测全绿
    （playbackRate setter ratechange 派发面 M2 既有）。
- 试导回退三件（定性升级，均为「上游绿 + 本地缺口」可回访件）：
  1. pause-move-to-other-document——本地「paused after stable state got true」：
     shim 融合视图下 iframe contentDocument.body.appendChild 触发 removal-pause
     两段 defer；spec related 文档判定含 iframe 文档（移入 related 文档不暂停）。
     归「pause-on-removal related-document 判定精化」切片。
  2. track-remove-insert-ready-state——本地 canplaythrough 时 track.readyState
     got 0（期望 ERROR 3）：video 加载序列与 track settle 双通道时序未收敛，与
     video_size_preserved_after_ended 同族。
  3. track-mode——本地 Timeout：mode 切换 no-event 断言依赖 cuechange 计数
     done 链（4 次 enter/exit），真播放推进 + cue 时序收敛依赖。
- markup 结构族定性升级：voice/class-markup/cue-recovery/markup/timestamp/
  unsupported-markup 6 件上游 edge 全绿——排除归域从「渲染域远期」修正为
  「WebVTT cue text parser 切片」（spec webvtt-cue-text-parsing-rules：
  i/u/b/ruby/rt/v/c span 树构建 + 恢复规则 + getCueAsHTML DOM 面对拍；中等
  深结构，上游全绿证明可回访）。
- evidence：evidence/2026-09-04-media-upstream-audit-xli.json。

## 扩批 XLII — WebVTT cue text parser 树解析（2026-09-04）

- 背景：XLI 轮将 markup 结构族 6 件（voice/class-markup/markup/cue-recovery/
  unsupported-markup/timestamp）归域升级为「WebVTT cue text parser 切片」（上游
  edge 全绿可回访）。本轮实施。
- **getCueAsHTML 升级为 markup 树解析**（`_zwCueTextToFragment`，part01b，spec
  webvtt-cue-text-parsing-rules）：
  - b/i/u/ruby/rt → 同名 HTML 元素；c/v → span；classes 空格连接 → className
    （所有支持标签均可带 class——i.larger → <i class="larger">）；仅 v 的
    annotation → title。
  - class 字符集 = 非空白非 '>' 非 '.'（'>' 终止 tag；'.' 分隔下一段 class——
    探针实证初版两处缺陷：'>' 被吞进 class、'.' 未分段致 'red.uppercase' 连写）。
  - 无效起始标签（'< v Speaker>' '<v&…>' '<v-Speaker>'）→ 整体吞到 '>'（文本不
    保留）；'</ b>' 无效闭合 → 吞到 '>'；'<'+空白 → annotation 态吞到**原始 '>'**
    （实体形态 '&gt;' 不终止——entities-wrong「textContent 只剩 '<' 前文本」）；
    '<00:00:05.000>' 数字 name → timestamp 锚点吞到 '>' 无产物。
  - 未知标签（h1/a/ul/li/img/video 等）→ 忽略标签保留内容；裸 rt（无 ruby 祖先）
    → 忽略标签保留内容；闭合标签栈内匹配 → 收拢其上全部，无匹配 → 忽略；cue 末
    未闭合 → auto-close。
  - 空 cue → 单空 Text 节点（扩批 XXXVIII track-cue-empty 断言面保持——初版回归
    即刻修复）。
- **cue.text 保留 parser 输入原文**：_zwParseVtt 移除 _stripMarkup 剥离层（XXX 批
  近似）——unsupported-markup 断言 text 含 '<h1>' 原文；markup/annotation 吞并
  全部移至 getCueAsHTML DOM 面（cue.text 与 DOM 面分离为 spec 语义）。
- **header 恢复**：cue-recovery-header（WEBVTT 行后无空行直接 cue 块）——元数据区
  的 timings 行终止 header（spec 恢复语义；此前整段跳过致首 cue 丢失；修复时
  _tParse 引用时序勘误——跳过循环移至 _tParse 定义后）。
- 6 件 12 子测全绿（628P/0F/24PF = 96.32%，+12 净涨零回归）；make test 66 套件
  全绿、clippy/fmt 干净。evidence：evidence/2026-09-04-webvtt-cue-text-parser-xlii.json。

## 扩批 XLIII — pause-on-removal related 判定精化（2026-09-04）

- 背景：XLI 轮试导回退件 pause-move-to-other-document 的可回访断言面闭环。
- **根因定位**（探针实证）：跨 iframe 移动（`iframe.contentDocument.body.appendChild(v)`）
  走 part03 R112 通用树分支（sel-only 子；handle 子另有 R369 串行合并分支）——该分支
  不清 `_zwRemovedSels` 移除标记：removeChild（隐式 adopt）时置位的 'video' 标记残留
  → removal-pause 两段 defer 的 tick1 `_zwIsRemovedNode` 判「仍 removed」→ playing 置
  false + pause 事件 → 「paused after stable state」假真（上游期望 false）。
- **修复**：通用树分支（`ensureTree(); r=_tree.appendChild(c)` 后）对 sel-only 子补
  `_zwUnmarkRemoved(c.__zwSelector)`——元素获新父（含跨文档 move）即脱离 removed 态
  （spec「pause on removal」限定「removed from a document」，move 到 related 文档不暂停）。
  handle 子路径（part04 appendChild 4149 unmark）与 removeChild 双分支的 pause 语义
  零变化。
- pause-move-to-other-document 解除排除导入（629P/0F/24PF = 96.32%，+1 净涨零回归）；
  media 全量 629P 零回归（iframe body appendChild 为 MutationObserver/R369 既有件
  共享面——无回归）；make test 66 套件全绿、clippy/fmt 干净。evidence：
  evidence/2026-09-04-media-pause-move-related-xliii.json。

## 扩批 XLIV — track 重加载 readyState hold（2026-09-04）

- 背景：XLI 回退件 track-remove-insert-ready-state 复评。
- **可回访面收敛确证**（探针）：canplaythrough 时 track.readyState=3（ERROR）已由
  XLII/XLIII 改动顺带收敛——「双通道 settle 时序未收敛」的旧定性部分失效。
- **readyState hold 落地**（spec「track 重加载是 queued task——新加载落定前同步读
  保持旧值」）：_zwTrackScheduleLoad srcChange 重调度时快照旧 outcome 为
  `_zwTrackReadyStateHold`（ERROR=3/LOADED=2）+ 删 resourceState（幂等门重 settle
  需要）；part04 readyState getter 在 resourceState 缺失时回读 hold；deferCont 续段
  入口（新加载判定开始）撤除。
- **定性再升级 + 维持排除**：余缺口是「主文档静态 video removeChild→重插」链在
  runner 沙箱 mutation 应用层报「set_attr: no match for video」——移除-重插静态元素
  的 host 通道缺口（非语义面，与 video_size_preserved_after_ended 同族），随
  runner/shim 事件通道统一后复评。
- 629P/0F/24PF = 96.32% 维持零回归（played-loop 单轮全量偶发经复跑排除）；clippy/fmt
  干净。

## 扩批 XLV — 静态元素移除-重插 host 通道归因修复（2026-09-04）

- 背景：XLIV 轮「set_attr: no match for video」缺口归因。探针定位出**三处真实缺陷**
  并修复（纯 DOM 重插探针全绿：parent=BODY / boxKids=1 / bodyContains=true）：
  1. R334 sel 子 appendChild 分支不清 `_zwRemovedSels`——removeChild→appendChild 重插
     后标记残留，parentNode getter（_zwIsRemoved 短路）恒 null。修复：分支内补
     `_zwUnmarkRemoved`。
  2. R334 旧父在 wire（reparent）之后读——读到的恒 null/新父，removed record 归旧父
     语义丢失。修复：wire 前读。
  3. parentNode getter 不消费 `_zwSelPendingParent` 槽——host mutation 异步 apply 期
     `_parentNodeFor` 读 host 视图返旧父。修复：getter 优先读槽（expando 表 + this
     dual 读），后回落 _parentNodeFor。
- **track-remove-insert-ready-state 维持排除（定性再升级）**：三修复 + XLIV hold 后
  用例全链路暴露**静态 video（src 已加载）removeChild 与 media load 管线死循环**（90s
  看门狗截断）。归因注记：非 XLV 引入——XLIII/XLV 修复使 removeChild 走得更深而暴露
  （此前被 set_attr 报错提前截断遮蔽）。归 media load 移除竞态独立切片。
- 诊断注记：二分期间一次「body.removeChild 卡死」误判系**并行负载下的看门狗偶发**+
  顶层/track-element 两个同名探针文件的路径混淆——复核后排除。全量 629P/0F/24PF
  零回归；clippy/fmt 干净。
