# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-04（**M3 扩批 XXVIII 落地**——currentTime-move-within-document
导入（offsets-into-the-media-resource 末件——同文档移动不重置播放，headless 时钟
推进面现成零改动导入；fixture 增 movie_300.webm）+ track-mode-triggers-loading
导入（metadata track disabled 不加载，mode 改 hidden 触发——扩批 XV mode 触发面
直接覆盖）+ track-remove-quickly / -by-setting-innerHTML 导入（track 移除不
crash smoke 面——innerHTML 注入 + seeked 计数链中 innerHTML 清空后再 seek）。
**560P/0F/24PF，560/584 = 95.9%**（+4 净涨零回归）。**track 空 src 语义
两处补全**（src=''空串 error settle code 4 + removeAttribute('src') 重调度——
与 setAttribute 对称）；track-element-src-change-error / -src-aborted-load
维持排除（in-flight loading 中断时序 / trickle pipe——headless 不可复现，
实证注记）。此前同日：**M3 扩批 XXVII 落地**——media fragment #t= 起点解析
（settle 加载序列内 currentTime 初始化：npt:/HH:MM:SS/ms/percent-encode 五形态）
+ headless 播放时钟推进（march 内墙钟差 × playbackRate——autoplay 驱动的播放
currentTime 不再恒 0）+ 周期 timeupdate（250ms 节流——播放推进期页面可收
timeupdate）。media_fragment_seek + autoplay-with-broken-track 导入。
**556P/0F/24PF，556/580 = 95.9%**（+4 净涨零回归）。此前同日：**M3 扩批 XXVI 落地**——seekable/buffered TimeRanges
headless 近似 getter（`__zwMediaSeekableRanges` 共享面：readyState>=1 后
[0,duration] 单区间 + IndexSizeError + has-trap 补列）+ currentTime setter
seek 语义补全（clamp 到 seekable 范围镜像写回 + seeking/timeupdate/seeked
同一排队任务序派发——seeking 异步化后挂 handler 可达；seeking 翻 false 先于
timeupdate）。seeking/ 三件 + volume_nonfinite 导入。**552P/0F/24PF，
552/576 = 95.8%**（+9 净涨零回归）。此前 2026-09-03：**M3 扩批 XXV 落地**——loop-from-ended.tentative 导入
（playing-the-media-resource/——ended 后设 loop 再 play 须回卷 seeked，Chromium
crbug 364442 断言面）。配套播放面四处缺陷收口（见 media-playback master.md 切片
8）：registry Ended→play 解码器重建 + 伴生轨游标归零 / seek 游标 clamp / 泵时钟
注入（桥 play 锚与 tick 同源）/ shim ended 态 play 派 seeking/seeked + loop
setter 翻回 ended=false。**543P/0F/24PF，543/567 = 95.8%**（+1 净涨零回归）。
此前同日：**M3 扩批 XXIV 落地**——loop 属性真面 + played
TimeRanges：registry `set_loop`（音频 entry 流末回卷——解码器重建 + 游标归零 +
播放态保持；伴生轨同面；`reached_end` 标志补音频面 isEnded 驱动源）+ shim loop
IDL setter/getter（`_mediaState` 镜像 + 桥推送 + play 起播同步，retry 路径同面）+
march Ended 面 loop 分叉（seek(0)+play + seeking/seeked 派发非 ended；ended IDL
属性翻转补置）+ played TimeRanges（march 采样 `_zwPlayedRanges` 250ms 容差合并 →
getter TimeRanges 形状；loop 回卷前尾段计入）+ duration getter settle 竞态兜底
（动态 src 真值落位晚于加载序列时读 settle durationMs；已开始无真值回落 600；
未加载 NaN）。**setLoop 桥回调参数索引修复（args.get(2)→get(1)——门面传 2 参，
此前 on 恒 false）**。played-loop / audio_loop_seek_to_eos 导入；loop-from-ended
暂不导入（动态 src settle 竞态注记，随基础设施复评）。**542P/0F/24PF，
542/566 = 95.8%**（+2 净涨零回归）。此前同日：**M3 扩批 XXIII 落地**——media load invoke 重置面收口：
`_zwMediaScheduleLoad` invoke 入口重置 `_resourceStates[key]`（spec 资源选择 invoke
步——二次调度失败候选须重新 settle，幂等门不再误吞）+ invoke 步 6 位置重置
（`readyState>=1` 时 currentTime=0 + `_zwMediaTimeKnown` 失效——spec「set the
current playback position to 0 ... HAVE_NOTHING」）+ invoke 重置 track 子产物 cue
（addTextTrack 产物排除——无 URL 面不随 media load 重置）+ settle 的 media/track
元素 load/error 派发改 `_zwMediaFire`（handle-only 元素 on\* expando handler 兜底——
此前 `_dispatchWithBubble` 传 handle=null 致 listener 键恒失配）。track-active-cues
导入（B 组末件解除排除）。**540P/0F/24PF，540/564 = 95.7%**。此前同日：
**M3 扩批 XXII**——B 组排除件复评三案收口：march disabled gate（disabled track
跳过 cue 调度 + active 清空——spec time-marches-on 步 2）+ march 遍历面统一
（addTextTrack 产物纳入——此前 cue 永不 enter/exit）+ cuechange 派发（per-track
单次异步 + track 元素转发）+ play() 桥 src 读身份分派（handle 身份走 registry
现值）。track-disabled / no-cuechange-before-play / track-remove-active-cue 导入。
**539P/0F/24PF，539/563 = 95.7%**。此前同日：
**M3 扩批 XXI**——深结构项 D 组首个收口：TextTrackList change 事件广播——
TextTrack↔TextTrackList 反向链
（`_zwOwnerList` 三处回填）+ addTextTrack 即时建 list（spec：track 创建即属于
media 元素 track 列表；上游用例 mode setter 先于 textTracks 首读的时序依赖）+
mode 有效值变更异步派基础 Event('change')（无 track 属性/target=list/同值不派）。
track-change-event 导入（536P/0F/24PF）。此前同日：
**M3 扩批 XX**——HAVE_NOTHING 期 seek 挂起
语义：currentTime setter readyState 0 时挂 `_zwSeekDeferred`，
`_zwMediaLoadSequence` readyState 0→1 翻转时补跑 seek 算法（seeking + seeked
异步回落 + cue active 面同步）——spec「default playback start position」；
track-cues-seeking 导入。**535P/0F/24PF，535/559 = 95.7%**（+1 净涨零回归）。
此前同日：**M3 扩批 XIX**——解码器 EOF 排空缺陷修复 +
track-cues-enter-exit / pause-on-exit 导入：zero-media `next_frame` draining
中间态（demux 尽后排空隐藏帧滞后队列才报流末）+ player `present_pending`
未来帧 `un_read` 退回 + march pauseOnExit 暂停先于 exit 派发 + pending seek
补推路径补 seekSync（534P/0F/24PF；track-cues 5 用例 4 连跑稳定）。此前同日：**M3 扩批 XVI+XVII+XVIII**——
track-cues-* 播放推进族解锁：fixture-mounted runner 播放桥前置 + 逐 tick
动态源登记 + MediaSourceProvider 按需补登记 + time-marches-on 区间捕获/
事件时间序/ended 面 + play() 桥 latest-wins 读/退避重试/pending seek 补推 +
march 区间基线修正 + registry play 未命中不消费源字节 + is_ended 桥面
（532P/0F/24PF）。此前 2026-09-02：**M3 扩批 XV**——http VTT 文件加载 +
WebVTT 解析深化 + 静态 track 调度触发面 + window.event，12 用例导入）

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
| M1g | WPT media-elements 用例覆盖 | ✅ 154 用例已导入（136 + 扩批 XVI~XXVIII：播放推进族 6 件 + track-change-event + track-active-cues + played-loop + audio_loop_seek_to_eos + loop-from-ended.tentative + seeking/ 三件 + volume_nonfinite + media_fragment_seek + autoplay-with-broken-track + currentTime-move-within-document + track-mode-triggers-loading + track-remove-quickly + track-remove-by-setting-innerHTML），**95.9%**（560/584） | — |
| M2g | load 算法 + 状态机（事件序列派发） | ✅ M2 落地（13T→**0T**） | F4 闭合 |
| M3g | 事件序列 headless 近似驱动 | ✅（同 M2g；source-child 触发已落地） | F4 闭合 |
| M4g-a | 媒体元数据 IDL 反射（初值面） | ✅ 切片 3 落地 | F2 闭合（-9 Fail） |
| M4g-b | `<track>` 反射 + TextTrack 最小面 | ✅ M3 落地（TextTrack 接口/addTextTrack/textTracks/track.track 全绿） | F1/F3 闭合 |
| M4g-c | track.src URL 解析 + \0 剥离 | ✅ 切片 3 落地（含 `<a href="">` 修复） | F6 闭合（-6 Fail） |
| M4g-d | canPlayType 能力表（空表→选型面更新） | ✅ 2026-09-01 落地（media-playback 流代行——跨 goal 联动兑现）：能力表由解码面真值驱动（webm/ogg 容器 maybe + vp9/vorbis/mp3 probably，VP8/Opus/Theora/H.264/AAC 域外诚实 ''）；单测 `test_media_can_play_type_capability_table_m4gd`（18 断言面）。**2026-09-02 av1 扩表**（media-playback M3 AV1 切片落地联动）：video/webm; codecs="av1" → probably（单测 +2 断言面） | F5（41→27 PF，in-face 全转 Pass） |
| M4g-e | play()/pause() 生命周期语义（queued task + 移除暂停） | ✅ M3 扩批 VII 落地（2026-09-01）：play/playing/timeupdate 改 queued task 派发（play() 后注册的 handler 仍收到）；pause-on-removal 两段 defer（tick1 paused=true → tick2 pause 事件，幂等）；导入 pause-remove-from-document.html（387/414 = 93.5%）；单测 `test_media_pause_on_removal_m3b7` | — |
| M4g-f | resource selection 算法（load 算法正题） | ✅ M3 扩批 XI 落地（2026-09-02）：networkState 同步段 NO_SOURCE(3)/稳定态 EMPTY(0) microtask 续段 + invoke 面（play/pause/load/setAttr-src/insert-source）+ load() 重跑（重置/重调度/epoch）+ 候选失效中断 + source 子 error 面；单测 `test_media_resource_selection_m3xi` | — |
| M4g-g | TextTrack cue 面（VTTCue/addCue/排序/getCueById/TrackEvent） | ✅ M3 扩批 XII 落地（2026-09-02）：VTTCue 构造器 + 非有限 TypeError + cues 动态重排 + 索引 own 镜像 + TrackEvent 惰性链 + gate 非对称（cues readiness / activeCues mode）；单测 `test_media_text_track_cue_face_m3xii` | F1 尾账闭合 |
| M4g-h | TextTrack cue 选项 + 列表增量事件（VTTCue 定位/addtrack 派发/src 清 cue） | ✅ M3 扩批 XIII 落地（2026-09-02）：VTTCue line/position/size/align/vertical/snapToLines IDL + data:text/vtt 加载 + addtrack 异步派发（holder 基线）+ TrackEvent target/类型修复 + src 变更重调度清 cue + readyState settle 面 | — |
| M4g-i | http VTT 加载 + WebVTT 解析深化（header/id/settings/实体/tag 截断） | ✅ M3 扩批 XV 落地（2026-09-02）：同步 `__zw_fetch` 通路 + `_zwParseVtt` + getCueAsHTML + 静态 track 调度触发面（querySelector/track.track/textTracks 三入口）+ default mode gate + mode setter 触发 + window.event；单测 `test_media_http_vtt_loading_m3xv` | — |

## 下一步计划

1. **扩大导入面（余面收口）**：track-cues-* 播放推进族全件已导入（扩批 XX
   seeking 兑现——HAVE_NOTHING 挂起 seek 语义）+ B 组排除件全清（扩批 XXII
   disabled/no-cuechange/remove-active-cue 三案 + 扩批 XXIII track-active-cues
   末件——load invoke 重置面收口）+ loop 真面三件导入（扩批 XXIV played-loop /
   audio_loop_seek_to_eos + 扩批 XXV loop-from-ended.tentative——ended 后设
   loop 再 play 回卷 seeked，配套 registry Ended→play 解码器重建 + 泵时钟注入
   收口）+ seekable/buffered TimeRanges 面四件导入（扩批 XXVI seeking/ 三件 +
   volume_nonfinite——seek clamp + seek 事件序排队任务化收口）+ media fragment
   与 autoplay 面三件导入（扩批 XXVII media_fragment_seek +
   autoplay-with-broken-track——headless 播放时钟推进 + 周期 timeupdate 收口；
   扩批 XXVIII currentTime-move-within-document——同文档移动不重置播放 +
   track-mode-triggers-loading——metadata track mode 触发加载（D 组排除件
   首次解锁）+ track-remove-quickly/-by-setting-innerHTML——移除不 crash smoke
   面（D 组「移除竞态」注记收口）；
   audio/video_volume_check 不导入：越界断言 e.code==1 为旧 spec 语义——现行
   spec 越界 clamp 不抛，导入即恒假失败）。
   playing-the-media-resource
   剩余（play-in-detached-document——需 detached 文档播放时钟推进，依赖兄弟目标
   media-playback 播放钟接语义层；fragmented-mp4-end——MSE 面，归远期）；
   no-autoplay-audio-history-back（iframe+history+postMessage 导航深结构，
   pause-move-to-other-document 同域排除）；the-video-element 反射余面
   （video-loading-*
   preload 语义族——视 lazy-loading 支撑面）。**headless 可导入面已在 95.9% 重饱和
   （M3 扩批 XXVIII 后第十一次修正；track-mode-triggers-loading 的 metadata
   加载时序由扩批 XV mode 触发面直接覆盖——D 组排除注记失效）**——余下增量依赖兄弟目标解锁（真播放钟 →
   time-marches-on 余面）+ 深结构项（~~TextTrackList change 事件广播反向链~~ ✅
   扩批 XXI 兑现、cue 标记树解析——归渲染域远期）。
   **下一轮候选**：ready-states/autoplay.html（autoplaying flag 与 play()/pause()/
   load() 交互 + 事件严格序断言——需 autoplay 时机面评估：canplay/canplaythrough
   同步派发与 play/playing 异步派发的序保证，audio+video 各 5 子测）；
   audio/video_volume_check 维持排除（越界断言 e.code==1 为旧 spec 语义）。
2. ~~**M4g-d**：canPlayType 能力表联动更新~~ ✅ 2026-09-01 兑现（能力表真值化——
   后续新增解码面（AV1/H.264，media-playback M3）时同步扩表）。
3. ~~**M4g-f**：resource selection 算法面~~ ✅ 2026-09-02 兑现（M3 扩批 XI——
   上一轮「吃尽」判断漏了 loading-the-media-resource 族，本轮 30 案全绿）。
4. ~~**M4g-g**：TextTrack 家族接口语义面~~ ✅ 2026-09-02 兑现（M3 扩批 XII——
   限额前 WIP 续接收口，interfaces/ 28 案全绿）+ 扩批 XIII（同日，cue 选项 +
   列表增量事件面，track/track-element 余下 5 案全绿）。track/track-element 目录
   余下 VTT 渲染/布局用例（cue 视觉呈现）仍排除——字幕渲染归渲染域远期，
   与入口文档排除清单一致。**headless 可导入面在 95.3% 重饱和**——余下增量依赖
   兄弟目标解锁（真播放钟 → track-cues-* 播放推进族）+ 深结构项（TextTrackList
   change 事件广播反向链）。

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
  → 扩批 XIII（2026-09-02，TextTrack cue 选项 + 列表增量事件面）**510/535 = 95.3%**
  （+14 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
  → 扩批 XIV（2026-09-02，pause-on-removal 两变体 + play() 无候选 pending）**512/537 = 95.3%**
  （+2 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
  → 扩批 XV（2026-09-02，http VTT 加载 + WebVTT 解析深化）**528/553 = 95.5%**
  （+16 subtest 全绿；Fail 0 / Timeout 0 / PF 25）
  → 扩批 XV 尾（2026-09-02，fixture-mounted 切片 1 canPlayType webm-opus 扩表）
  **529/553**（+1P）
  → 扩批 XVI（2026-09-03，track-cues-* 播放推进族——enter-seeking + missed 导入；
  enter-exit 暂排除）**531/555 = 95.7%**（+2 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XVII/XVIII（2026-09-03，sorted-before-dispatch 复评导入 + MediaSourceProvider
  按需补登记 + march 区间基线修正）**532/556 = 95.7%**
  → 扩批 XIX（2026-09-03，解码器 EOF 排空缺陷修复——enter-exit + pause-on-exit
  导入）**534/558 = 95.7%**（+3 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XX（2026-09-03，HAVE_NOTHING 期 seek 挂起语义——seeking 导入）
  **535/559 = 95.7%**（+1 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XXI（2026-09-03，TextTrackList change 广播——track-change-event 导入）
  **536/560 = 95.7%**（+1 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XXII（2026-09-03，disabled gate + march 遍历统一 + cuechange——三案导入）
  **539/563 = 95.7%**（+3 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XXIII（2026-09-03，media load invoke 重置面——track-active-cues 导入）
  **540/564 = 95.7%**（+1 净涨零回归；Fail 0 / Timeout 0 / PF 24）
- 入口：`make testharness-media`（FILTER 透传，`--json` 捕获 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：`evidence/2026-08-31-media-baseline.md`（+ 同名 .json 机读版）、
  `evidence/2026-09-01-media-event-family.json`、`evidence/2026-09-01-media-source-child.json`、
  `evidence/2026-09-01-media-volume-audio.json`、`evidence/2026-09-01-media-controlslist.json`、
  `evidence/2026-09-01-media-playbackrate-typeerror.json`、`evidence/2026-09-01-media-preload-setter.json`、
  `evidence/2026-09-01-media-track-sync.json`、`evidence/2026-09-02-media-resource-selection.json`、
  `evidence/2026-09-02-media-texttrack-family.json`、`evidence/2026-09-02-media-cue-options.json`、
  `evidence/2026-09-02-media-pause-removal-variants.json`、`evidence/2026-09-02-media-http-vtt.json`、
  `evidence/2026-09-03-media-cues-playback.json`、
  `evidence/2026-09-03-media-eof-drain-r3936.md`（+同名 .json）、
  `evidence/2026-09-03-media-deferred-seek-r3937.md`（+同名 .json）、
  `evidence/2026-09-03-media-change-event-r39xx.md`（+同名 .json）、
  `evidence/2026-09-03-media-b-group-revisit-r39xx.md`（+同名 .json）、
  `evidence/2026-09-03-media-load-invoke-reset-r39xx.md`（+同名 .json）
