# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-04（**M3 扩批 XXXIX 落地**——audio_loop_base 解除排除导入
（603P/0F/24PF = 96.17%，+1 净涨零回归；3 连跑稳定）：**registry loop 回卷可观测面**
——dormant 探针实证推翻 XXIV 排除注记：Timeout 根因非「fixture 短于泵采样粒度」，
而是 loop=true 的音频 entry 流末回卷为静默 restart，语义层 march ended/loop 分叉的
唯一驱动源 isEnded 恒 false → seeking/seeked 派发不可达。修复：AudioEntry 增
wrap_pending（回卷置位 / is_ended 读取 / audio_play 消费复位——零新增桥面）。
video_loop_base 维持排除（结构互斥：fixture 2x2-green.webm 实为 VP8+Opus，解码面
外——canPlayType 判定串 'vp9, opus' → .webm 变体与可解码 codec 不符，bridge play
失败回落 headless 无流末界）。单测
`registry_audio_loop_wrap_observable_via_is_ended_m3xxxix`（三面）；make test
18872/0。evidence：`evidence/2026-09-04-media-audio-loop-base-xxxix.json`。
此前 2026-09-04：**M3 扩批 XXXVIII 落地**——track-cue-empty 解除排除导入
（602P/0F/24PF = 96.17%，+1 净涨零回归）：**Text.prototype.constructor 自引修复**
（WebIDL §4.6.1 interface prototype object——探针实证 XXX 排除注记「constructor.name
原生 class 断言 shim 工厂面差异」的真缺口为 Text.prototype 缺 constructor 自引，
文本节点 R179 原型链后 constructor 解析到 Node.prototype.constructor → 名 'Node'；
shim part03 Text.prototype 构建处补 defineProperty(value=Text, enumerable false)）。
getCueAsHTML 空 cue 面（DocumentFragment instanceof + 单空 Text 节点 + 
constructor.name === Text.name + length/data）。此前 2026-09-04：**M3 扩批 XXXVII 落地**——the-video-element 清点补遗：
intrinsic_sizes.htm 定性（XXXV 未显式定性尾件）+ video 固有尺寸 getComputedStyle
面落地（spec 对齐修复，无整文件可导入面）——shim 生产回调路径
compute_document_styles_with_inline_overrides 增 video 分支（canvas R34xx 同形）：
width/height 属性 auto 侧覆盖 + 仍 auto 维落 default object size 300×150
（css-images-3 §5.1 + HTML §4.8.11）+ 显式 CSS 尺寸不覆盖。子测 4/5（src 移除
rAF 回流回落 + poster 解码固有尺寸）依赖解码真值进 computed-style（跨面扩展）
+ rAF 回流面——维持排除注记。单测
`test_get_computed_style_video_intrinsic_sizes_m3xxxvii`（9 断言面）。
601P/0F/24PF = 96.16% 维持零回归；make test 18868/0。evidence：
`evidence/2026-09-04-media-video-intrinsic-sizes-xxxvii.json`。此前 2026-09-04：
**M3 扩批 XXXVI 落地**——the-audio-element 目录清点：
audio-loading-eager 导入（601P/0F/24PF = 96.16%，+1 净涨零回归）——loading=eager
立即加载面 audio 形态（与 XXXV video-loading-eager 同构；audio.loading IDL 反射面
R115 _REFLECTED_STRING_FLAT 已含 loading；media/sine440.mp3 资产入档）。
**余件逐件定性**：audio_constructor 已导入（扩批 III）；audio-loading-lazy-* 8 件
与 autoplay/load/preload-deferred 系 4 件维持排除（lazy 断言「视口外不加载」与
eager-by-default 实现互斥——同 XXXV 定性）；audio_001/002 + audio_content-ref 为
reftest 渠道（MS 面「audio 元素内容不呈现」，testharness-media 通道不适用）；
audio-*-inactive-document-crash 2 件（iframe contentDocument.cloneNode 深结构）/
audio-with-replaced-after-pseudo-crash（::after replaced content 渲染域）不导入
——**the-audio-element 目录清点收束**。evidence：
`evidence/2026-09-04-media-audio-loading-eager-xxxvi.json`。此前 2026-09-04：**M3 扩批 XXXV 落地**——the-video-element 目录清点：
video-loading-eager 导入（600P/0F/24PF = 96.15%，+1 净涨零回归）——loading=eager
立即加载面（loadeddata 到达；headless settle 无视口 gate，eager 语义即本实现形态；
video.loading IDL setter 反射 + media/A4.webm 资产入档）。**余件逐件定性**：
video-tabindex/video_crash_empty_src 已导入（扩批 IV/VIII）；video-loading-lazy-*
与 load/autoplay/poster-deferred 系维持排除（lazy 断言「视口外不加载」与
eager-by-default 实现互斥——lazy-loading 视口交叉观测深结构项）；preload
deferred 系同域；video_initially_paused/poster/transparent-controls 渲染呈现或
reftest 面；video_timeupdate_on_seek WPT CGI 慢资源端点；resize-during-playback
渲染事件面——**the-video-element 目录清点收束**。evidence：
`evidence/2026-09-04-media-video-loading-eager-xxxv.json`。此前 2026-09-04：**M3 扩批 XXXIV 落地**——play-in-detached-document 解除
排除导入（599P/0F/24PF = 96.15%，+1 净涨零回归）：**detached 文档媒体方法面**——
根因勘误：此前排除注记「需 detached 文档播放时钟推进」定性不准（扩批 XXVII
headless 时钟 + 周期 timeupdate 已就绪）；真缺口是主文档 media 方法装在 part03
get trap（R2835）而 _zwMEl（createHTMLDocument 产物，plain object 无 trap）
`v.play` 恒 undefined。修复：_zwMEl AUDIO/VIDEO 分支补 play/pause/load/
canPlayType + paused/currentTime/duration/src IDL（状态入 _mediaState synthetic
key '#dmN'，_zwEl 直指节点）+ detached settle 近似（src setter 触发 readyState
1→4 加载事件序 + autoplay 续派）+ march 周期 timeupdate 的 detached 分支
（_zwEl.dispatchEvent 直接派发）。单测
`test_media_detached_document_play_face_m3xxxiv`；make test 18867/0。evidence：
`evidence/2026-09-04-media-play-in-detached-xxxiv.json`。此前 2026-09-04：**M3 扩批 XXX 落地**——WebVTT 解析面批量导入：
track-helpers.js 断言辅助（assert_cues_equal/check_cues_from_track/
as_textcontent 对拍）+ 27 件 vtt 资源 + 14 用例（BOM/UTF8 编码面（iconv 拒收）/
header 注释/空 cue/timings 变体（no-hours/whitespace/large-timestamp/
negative-duration）/interspersed-non-cue/newlines/退化形态）。
**584P/0F/24PF，584/608 = 96.05%**（+14 净涨零回归）。**XXX 续**：mode/
cuechange 播放推进面 3 件导入（track-mode-disabled / track-cues-cuechange /
track-cues-add-new-track——B 组基建现成试导全绿；track-mode 维持排除——mode
数值 setter 回落 + cue 计数 done 链独立切片）。
**587P/0F/24PF，587/611 = 96.07%**（累计 +17 净涨零回归）。**XXXI**：
removetrack 派发（TextTrackList 增量同步以 holder 为基线，消失 track 逐个派
TrackEvent('removetrack',{track}) + dispatch 期 window.event——addtrack 同构
补面）+ TextTrackList @@iterator（for...of 断言面）+ selection 算法 kind-aware
初始 mode（metadata+default → hidden）——track-selection-metadata /
track-remove-track / track-cues-missed-no-immediate-events 导入。
**590P/0F/24PF，590/614 = 96.09%**（累计 +20 净涨零回归）。**XXXII**：
VTTCue line/position setter 关键字校验（非数字且非关键字字符串 → TypeError）+
position/size 范围校验（[0,100] 外 → IndexSizeError——此前 clamp 面改 spec
范围校验）；readyState/稳态面 7 件导入（load-error/load-from-element/
src-empty-string 的 readyState 断言 + cuechange-dynamically-created/
disabled-addcue / insert-after-load / cue-mutable 属性全链）。
**597P/0F/24PF，597/621 = 96.14%**（累计 +27 净涨零回归）。**XXXIII**：
track-text-track-cue-list 导入（length/[]/getCueById 零改动）。**598P/0F/24PF，
598/622 = 96.14%**（累计 +28 净涨零回归）。**track-element 缺失清单全数清点**
（导入/排除均有归域注记）。维持排除（实证注记）：
markup 结构族 6 件 + track-cue-mutable-fragment（assert_cue_fragment isEqualNode
对拍 getCueAsHTML 的 span 节点树——cue 标记树解析深结构项归渲染域远期）+
~~track-cue-empty~~（✅ 扩批 XXXVIII 解除排除——Text.prototype.constructor 自引修复）+
track-mode-not-changed-by-new-track（textTracks 身份对拍切片）+
track-remove-insert-ready-state（re-attach 播放推进链切片）+
track-selection-task-order（selection 宏任务序切片）。**全目录清点收束（2026-09-04
勘察）**：主目录 24 件缺失全定性（manual×8 / permissions-policy https×6 /
~~loop_base~~×2（✅ XXXIX audio_loop_base 解除排除导入；video_loop_base 结构互斥注记）/ 已注记排除×5——error-sequence MSE-util / no-autoplay iframe /
preserves-pitch / src_object_blob / volume_check 旧语义）；resize-during-playback
（双分辨率 mid-stream resize fixture + resize 二次派发——排除注记）；the-video-
element 余 26 件 / the-audio-element 余 16 件（✅ 扩批 XXXVI/XXXVII 清点收束——
lazy/deferred 系互斥排除、reftest 渠道面、intrinsic_sizes 补遗、crash 族不导入）；user-interface muted 面 2 件（UI 域）；
crashtests GC 面 1 件（no-blocking-loads-gc-crash——2026-09-04 定性：无 test() 的
crash 回归 smoke + /common/gc.js garbageCollect 驱动，runner 对零测试文件结构性
不适用；真断言「GC 期不 crash」归宿主健壮性面非 WPT 语义面）；ready-states 余 2 件已注记。**缺口面穷尽定性——余下
增量依赖深结构或新批复解锁**。证据快照：
[evidence/2026-09-04-media-598p-snapshot.json](evidence/2026-09-04-media-598p-snapshot.json)
（598/622 = 96.14%，PF 全为 canPlayType optional 中性面）。此前同日：**M3 扩批 XXIX 落地**——ready-states/autoplay 导入
（audio+video 各 5 子测：autoplaying flag 与 play()/pause()/load() 交互 +
事件严格序 canplay→canplaythrough→play→playing / play 先行形态
play→canplay→playing→canplaythrough）。配套 HAVE_NOTHING 期 play() 挂起
语义（spec dom-media-play 步 6——play 事件 readyState 变化点派、playing 于
canplay 后；engine 契约测试同步序断言更新）。**570P/0F/24PF，
570/594 = 95.96%**（+10 净涨零回归）。此前同日：**M3 扩批 XXVIII 落地**——currentTime-move-within-document
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

（**M3 扩批 event_* 族 + 第二批 + III~X（2026-09-01）/ XI~XV（2026-09-02）/
  XVI~XVIII（2026-09-03）过程记录**——每批 shim 面/runner 面明细与排除注记——
  已归档至 [archive/2026-09-04_m3-batches-vii-to-xviii.md](archive/2026-09-04_m3-batches-vii-to-xviii.md)，
  证据 JSON 序列见验证基线；累计口径 603P/0F/24PF = 96.17%。第 XIX~XXXIX 批
  明细见头链（最新态）。）

**里程碑归档（2026-09-01）**：M1~M3 与六轮扩批的过程记录、排除用例决策清单已归档至
[archive/2026-08-31_m1-m3-and-2026-09-01_batches.md](archive/2026-08-31_m1-m3-and-2026-09-01_batches.md)
（只追加不修改）。**2026-09-04 续**：扩批 event_* 族~第 XVIII 批过程记录再归档至
[archive/2026-09-04_m3-batches-vii-to-xviii.md](archive/2026-09-04_m3-batches-vii-to-xviii.md)。

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
| M1g | WPT media-elements 用例覆盖 | ✅ 171 用例已导入（136 + 扩批 XVI~XXX：播放推进族 6 件 + track-change-event + track-active-cues + played-loop + audio_loop_seek_to_eos + loop-from-ended.tentative + seeking/ 三件 + volume_nonfinite + media_fragment_seek + autoplay-with-broken-track + currentTime-move-within-document + track-mode-triggers-loading + track-remove-quickly + track-remove-by-setting-innerHTML + ready-states/autoplay + WebVTT 解析面 14 件 + mode/cuechange 播放推进面 3 件 + XXXI 三件 + XXXII 七件 + XXXIII 一件 + XXXIV play-in-detached-document + XXXV video-loading-eager + XXXVI audio-loading-eager + XXXVIII track-cue-empty + XXXIX audio_loop_base），**96.17%**（603/627） | — |
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
   剩余（~~play-in-detached-document~~ ✅ 扩批 XXXIV 兑现——detached 文档媒体
   方法面落地后解除排除；fragmented-mp4-end——MSE 面，归远期）；
   no-autoplay-audio-history-back（iframe+history+postMessage 导航深结构，
   pause-move-to-other-document 同域排除）；~~the-video-element 反射余面~~
   （~~video-loading-* preload 语义族~~ ✅ 扩批 XXXV 清点——video-loading-eager
   导入，lazy/preload-deferred 系与 eager-by-default 实现互斥维持排除，
   **the-video-element 目录清点收束**）。**headless 可导入面已在 95.9% 重饱和
   （M3 扩批 XXXIX 后第二十次修正：96.17%）**——余下增量依赖兄弟目标解锁（真播放钟 →
   time-marches-on 余面）+ 深结构项（~~TextTrackList change 事件广播反向链~~ ✅
   扩批 XXI 兑现、cue 标记树解析——归渲染域远期）。
   ~~ready-states/autoplay.html~~ ✅ 扩批 XXIX 兑现（autoplaying flag 交互 +
   事件严格序——HAVE_NOTHING 期 play() 挂起语义落地）；ready-states 余两件
   维持排除（autoplay-with-slow-text-tracks——trickle pipe + readyState 与
   track 加载耦合面；autoplay-hidden.optional——hidden 节能语义 optional）；
   audio/video_volume_check 维持排除（越界断言 e.code==1 为旧 spec 语义）。
   ~~the-video-element intrinsic_sizes.htm~~ ✅ 扩批 XXXVII 清点补遗——静态属性面
   落地（video getComputedStyle 固有尺寸 + default object size 300×150），动态
   子测（解码真值 computed-style + rAF 回流）维持排除注记，单测资产化。
   ~~the-audio-element 余 16 件~~ ✅ 扩批 XXXVI 清点——audio-loading-eager 导入
   （同 XXXV 面 audio 形态），lazy/deferred 系与 eager-by-default 互斥维持排除、
   audio_001/002 reftest 渠道、crash/渲染域不导入，**目录清点收束**。
   video_size_preserved_after_ended 维持排除（2026-09-04 实证：静态 <source>
   形态 loadedmetadata 与 promise_test EventWatcher 时序 headless 双通道
   settle 下不稳定）——但其调试过程产出三项基础设施资产（静态 <source>
   settle/事件链 + 候选可达性判定 + registry probe_dimensions 真值链，
   engine/webview 回归面零回归落地）。
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
  → 扩批 XXIV~XXXV（2026-09-03 ~ 2026-09-04，loop/seekable/fragment/headless 时钟/
  detached 方法面/ready-states/WebVTT/track 清点/video 清点）**543→600P**（详见
  头部记录与 evidence 序列）
  → 扩批 XXXVI（2026-09-04，the-audio-element 目录清点——audio-loading-eager 导入）
  **601/625 = 96.16%**（+1 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XXXVII（2026-09-04，video 固有尺寸 getComputedStyle 面——spec 对齐修复，
  无新增导入）**601/625 维持零回归**
  → 扩批 XXXVIII（2026-09-04，Text.prototype.constructor 自引——track-cue-empty
  解除排除）**602/626 = 96.17%**（+1 净涨零回归；Fail 0 / Timeout 0 / PF 24）
  → 扩批 XXXIX（2026-09-04，registry wrap_pending 可观测面——audio_loop_base
  解除排除；video_loop_base 结构互斥注记）**603/627 = 96.17%**（+1 净涨零回归；
  Fail 0 / Timeout 0 / PF 24）
- 复核（2026-09-04 治理整固后终审）：fresh 跑 603P/0F/24PF（198 文件）与
  本档记录逐位一致；验证基线 evidence 链 26 文件全在盘；make test 18877/0。
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
  `evidence/2026-09-03-media-load-invoke-reset-r39xx.md`（+同名 .json）、
  `evidence/2026-09-04-media-598p-snapshot.json`、
  `evidence/2026-09-04-media-play-in-detached-xxxiv.json`、
  `evidence/2026-09-04-media-video-loading-eager-xxxv.json`、
  `evidence/2026-09-04-media-audio-loading-eager-xxxvi.json`、
  `evidence/2026-09-04-media-video-intrinsic-sizes-xxxvii.json`、
  `evidence/2026-09-04-media-track-cue-empty-xxxviii.json`、
  `evidence/2026-09-04-media-audio-loop-base-xxxix.json`

## 归档

- [archive/2026-08-31_m1-m3-and-2026-09-01_batches.md](archive/2026-08-31_m1-m3-and-2026-09-01_batches.md) —
  M1~M3 与六轮扩批（2026-08-31 ~ 2026-09-01）过程记录（只追加不修改）。
- [archive/2026-09-04_m3-batches-vii-to-xviii.md](archive/2026-09-04_m3-batches-vii-to-xviii.md) —
  扩批 event_* 族 + 第二批 + III~X / XI~XV / XVI~XVIII 过程记录归档
  （2026-09-04 治理切片；本控制面保留第 XIX~XXXIX 批头链明细与累计口径
  603P/0F/24PF = 96.17%）。
