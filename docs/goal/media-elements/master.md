# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-02（**M3 扩批 XII**——TextTrack 家族接口语义面：VTTCue 构造器 +
addCue/removeCue + cues 排序 + getCueById + TrackEvent + on* EventTarget 面 + data:text/vtt
解析，28 用例导入。**496P/0F/0T/25PF（496/521 = 95.2%）**）

---

## 当前状态

**专项定位**：媒体方向三拆之一（可立即启动）。HTMLMediaElement 非解码语义面（状态机/
事件序列/API 行为），WPT media-elements 真实用例驱动。**不被解码选型 RFC 阻塞**——
headless 近似驱动先行，兄弟目标建成后替换驱动源。

**M1 已完成（2026-08-31）**：`make testharness-media` 基础设施落地（fetch-media-subset.sh
+ runner `testharness-media` 子命令 + Makefile 目标）；首批 30 用例 / 245 subtest 基线
**46.5%**（114P/77F/13T/41PF）；失败聚类 F1~F6 成文（见 evidence/2026-08-31-media-baseline.md）。

**M1 切片 3 已落地（同日，F2/F6）**：媒体元数据 IDL 面（currentTime/duration NaN/
playbackRate/volume clamp/seeking/paused/preload/crossOrigin 枚举反射 + has-trap 白名单）
+ HTMLTrackElement 反射（kind/label/srclang/default/src + setter + R122 实例层同步）
+ `<a href="">` 空串解析修复。**73.1%**（179P/12F/13T/41PF，+26.6pp）；余账 12 Fail
全部为 TextTrack 家族（M3 域）。单测 `test_media_metadata_idl_face_r388`；make test
65 套件全绿、fmt/clippy 干净。

**M2 已落地（同日，F4 事件序列）**：`_zwSettleResourceKey` handle/sel 双身份泛化 + headless
加载序列（loadstart→progress→durationchange+loadedmetadata→loadeddata→canplay→
canplaythrough，networkState LOADING→IDLE）+ 动态 `.src=` setTimeout 模拟（精确空串仅
loadstart）+ play/pause 事件面 + seeking/volumechange/ratechange + on* 兜底派发。
**79.6%**（214P/12F/2T/41PF）。单测 `test_media_load_event_sequence_r389`；FR-009 集成
测试契约更新（error 路径断言零改动通过 = A/B 佐证）。

**M3 已落地（同日，TextTrack 家族）**：TextTrack/TextTrackCueList/TextTrackList/TextTrackCue
接口声明（Illegal constructor + 工厂原型链）+ `addTextTrack`（枚举精确校验 + WebIDL
DOMString 转换）+ `textTracks`（same-object + 增量同步）+ `track.track`（身份缓存 +
default→showing）。**84.0%**（226P/0F/2T/41PF）——**Fail 清零**。单测
`test_text_track_family_r390`。余账 2 Timeout 均深域：error 错误码语义（随资源选择/解码层）、
currentSrc 的 source-child 插入触发（mutation 面）。

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

**里程碑归档（2026-09-01）**：M1~M3 与六轮扩批的过程记录、排除用例决策清单已归档至
[archive/2026-08-31_m1-m3-and-2026-09-01_batches.md](archive/2026-08-31_m1-m3-and-2026-09-01_batches.md)
（只追加不修改；本控制面保留最新态与缺口清单）。

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

**DC 达成审计（2026-09-01）**：DC-1~4 实质满足——① 60 用例导入 + 8 份 evidence JSON
（基线演进 46.5→90.1 全程可追溯）；② 状态机/事件序列 WPT 断言面全绿（headless 近似
驱动逐项记录）；③ API 语义面全对齐（canPlayType 空表 + M4g-d 显式记录为跨 goal 依赖项；
play-pause 异常/元数据 setter/track 面全绿）；④ make test 65 套件全绿 + clippy 零警告 +
每修复带单测与用例资产化。
**治理注记**：入口文档 DC-1 第三条「经 `make import-wpt` 记入 imported-tests.txt」字面
指 reftest 通道；media 用例的资产化通道实际为 testharness-media pin（fetch 白名单 +
MEDIA_TEST_FILES + evidence JSON 序列）——两通道并行为 CLAUDE.md 测试资产化规则的
等价实现，差异在此记录，不改入口文档。**完成度**：语义面在当前 headless 驱动形态下
已达饱和（可导入面吃尽）；进一步提升依赖兄弟目标解锁（media-playback 解码层真值化
→ videoWidth/buffered/真 seek 面重评导入）。

**与兄弟 goal 的边界**：
- media-playback — 解码/帧渲染归其管（RFC 门控）；本目标的 readyState 真实驱动源由其
  供给（接口契约记录于两流 master.md）
- media-audio — 音频输出归其管；volume/muted 本目标只做语义（真增益归其接线）
- js-dom — 媒体反射段（part01.js R3040）共享；该流已归档（2026-08-31 核对），媒体段
  近 14 天无并发编辑，可安全作业

## 实测基线（2026-08-31 首批跑通后）

### 现有实现（修正后的真实面）

- ✅ 属性反射：R3040 autoplay/controls/loop/muted/playsInline + 布局占位渲染
- ✅ FR-009 资源 settle：audio/video/source/track 的 fetch 状态提交 + load/error 事件
  （error/readyState/networkState 初值、src 反射、historical 面均绿——**修正 2026-08-17
  立项时低估**：初值面与状态常量并非全缺）
- ✅ R2835 四方法：play()/pause()/load()/canPlayType()（canPlayType 能力表 2026-09-01
  真值化——webm/ogg 容器 maybe + vp9/vorbis/mp3 probably，域外 ''）
- ⚠️ 媒体专有 IDL 属性面缺失：currentTime/duration(NaN)/playbackRate/preload/
  crossOrigin 归一/seeking/paused 等（F2）
- ⚠️ `<track>` 反射 + TextTrack/TextTrackList/addTextTrack/textTracks 全缺（F1/F3）
- ⚠️ load 算法事件序列（loadstart/canplay/loadedmetadata/...）未派发——FR-009 只有
  load/error 两事件（F4）
- ⚠️ track.src 未按 URL 属性解析（同 R2838 a.href 模式未接）（F6）

## 缺口清单（含失败聚类映射）

| # | 缺口 | 状态 | 失败聚类 |
|---|------|------|----------|
| M1g | WPT media-elements 用例覆盖 | ✅ 119 用例已导入（含 event_* 族 25 + volume/Audio 构造器 + controlsList + track-element 6 + resource-selection 族 25 + interfaces/TextTrack 家族 28），**95.2%** | — |
| M2g | load 算法 + 状态机（事件序列派发） | ✅ M2 落地（13T→**0T**） | F4 闭合 |
| M3g | 事件序列 headless 近似驱动 | ✅（同 M2g；source-child 触发已落地） | F4 闭合 |
| M4g-a | 媒体元数据 IDL 反射（初值面） | ✅ 切片 3 落地 | F2 闭合（-9 Fail） |
| M4g-b | `<track>` 反射 + TextTrack 最小面 | ✅ M3 落地（TextTrack 接口/addTextTrack/textTracks/track.track 全绿） | F1/F3 闭合 |
| M4g-c | track.src URL 解析 + \0 剥离 | ✅ 切片 3 落地（含 `<a href="">` 修复） | F6 闭合（-6 Fail） |
| M4g-d | canPlayType 能力表（空表→选型面更新） | ✅ 2026-09-01 落地（media-playback 流代行——跨 goal 联动兑现）：能力表由解码面真值驱动（webm/ogg 容器 maybe + vp9/vorbis/mp3 probably，VP8/Opus/Theora/H.264/AAC 域外诚实 ''）；单测 `test_media_can_play_type_capability_table_m4gd`（18 断言面） | F5（41→27 PF，in-face 全转 Pass） |
| M4g-e | play()/pause() 生命周期语义（queued task + 移除暂停） | ✅ M3 扩批 VII 落地（2026-09-01）：play/playing/timeupdate 改 queued task 派发（play() 后注册的 handler 仍收到）；pause-on-removal 两段 defer（tick1 paused=true → tick2 pause 事件，幂等）；导入 pause-remove-from-document.html（387/414 = 93.5%）；单测 `test_media_pause_on_removal_m3b7` | — |
| M4g-f | resource selection 算法（load 算法正题） | ✅ M3 扩批 XI 落地（2026-09-02）：networkState 同步段 NO_SOURCE(3)/稳定态 EMPTY(0) microtask 续段 + invoke 面（play/pause/load/setAttr-src/insert-source）+ load() 重跑（重置/重调度/epoch）+ 候选失效中断 + source 子 error 面；单测 `test_media_resource_selection_m3xi` | — |
| M4g-g | TextTrack cue 面（VTTCue/addCue/排序/getCueById/TrackEvent） | ✅ M3 扩批 XII 落地（2026-09-02）：VTTCue 构造器 + 非有限 TypeError + cues 动态重排 + 索引 own 镜像 + TrackEvent 惰性链 + gate 非对称（cues readiness / activeCues mode）；单测 `test_media_text_track_cue_face_m3xii` | F1 尾账闭合 |

## 下一步计划

1. **扩大导入面（余面收口）**：playing-the-media-resource 剩余
  （play-in-detached-document——需 detached 文档播放时钟推进，依赖兄弟目标
   media-playback 播放钟接语义层；loop-from-ended.tentative / fragmented-mp4-end
   同域）；the-video-element 反射余面（video-loading-* preload 语义族——视
   lazy-loading 支撑面）。**headless 可导入面已吃尽（95.2%）**——后续增量依赖
   兄弟目标解锁（media-playback 解码/时钟真值化 → 真播放推进面用例）。
2. ~~**M4g-d**：canPlayType 能力表联动更新~~ ✅ 2026-09-01 兑现（能力表真值化——
   后续新增解码面（AV1/H.264，media-playback M3）时同步扩表）。
3. ~~**M4g-f**：resource selection 算法面~~ ✅ 2026-09-02 兑现（M3 扩批 XI——
   上一轮「吃尽」判断漏了 loading-the-media-resource 族，本轮 30 案全绿）。
4. ~~**M4g-g**：TextTrack 家族接口语义面~~ ✅ 2026-09-02 兑现（M3 扩批 XII——
   限额前 WIP 续接收口，interfaces/ 28 案全绿）。track/track-element 目录余下
   VTT 解析/渲染用例（cue 视觉呈现、渲染面）仍排除——字幕渲染归渲染域远期，
   与入口文档排除清单一致。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 摸底 | ✅ 完成（2026-08-31，46.5% 基线 + 切片 3 → 73.1%） |
| M2 — 状态机与事件序列 | ✅ 完成（2026-08-31，13T→2T） |
| M3 — API 语义 + track 面 + 播放层衔接 | ✅ 完成（2026-08-31，12F→0F，84.0%；2026-09-01 扩批 event_* 族 → 88.3%） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | canPlayType 能力表（依赖 media-playback M0 解码选型——选型决定支持面） | ✅ 2026-09-01 兑现（能力表真值化，见 M4g-d）——后续 AV1/H.264 解码面扩表时同步 |

## 验证基线

- 基线时点：2026-08-31，WPT rev `3159769338`；基线 **114/245 = 46.5%** → 切片 3
  **179/245 = 73.1%** → M2 **214/245 = 79.6%** → M3 **226/245 = 84.0%**（Fail 0 /
  Timeout 2 / PF 41）→ M3 扩批（2026-09-01，event_* 族）**324/367 = 88.3%**
  → 扩批第二批（同日，source-child + error 码）**334/375 = 89.1%**
  → 扩批 III（同日，volume/muted + Audio 构造器）**369/410 = 90.0%**
  → 扩批 IV（同日，controlsList + the-video-element）**372/413 = 90.1%**
  → 扩批 V/VI（同日，playbackRate TypeError + preload setter 补缺）**90.1% 维持**
  （Fail 0 / **Timeout 0** / PF 41）
  → 扩批 VII~IX + M4g-d 扩表（同日，pause-on-removal 族 + about: src + canPlayType
  opus）**392/417 = 94.0%**（PF 41→25）
  → 扩批 X（同日，track 子 ↔ textTracks 集合同步）**400/425 = 94.1%**
  （+8 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
  → 扩批 XI（2026-09-02，resource selection 算法族）**430/455 = 94.5%**
  （+30 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
  → 扩批 XII（2026-09-02，TextTrack 家族接口语义面）**496/521 = 95.2%**
  （+66 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
- 入口：`make testharness-media`（FILTER 透传，`--json` 捕获 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：`evidence/2026-08-31-media-baseline.md`（+ 同名 .json 机读版）、
  `evidence/2026-09-01-media-event-family.json`、`evidence/2026-09-01-media-source-child.json`、
  `evidence/2026-09-01-media-volume-audio.json`、`evidence/2026-09-01-media-controlslist.json`、
  `evidence/2026-09-01-media-playbackrate-typeerror.json`、`evidence/2026-09-01-media-preload-setter.json`、
  `evidence/2026-09-01-media-track-sync.json`、`evidence/2026-09-02-media-resource-selection.json`、
  `evidence/2026-09-02-media-texttrack-family.json`
