# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-04（**M3 fixture-mounted runner 播放面切片 9 落地（扩批
XXVI）**——seekable/buffered TimeRanges headless 近似 getter 落地
（`__zwMediaSeekableRanges` 共享面：readyState>=1 后 [0,duration] 单区间；
duration 解析序 桥真值 → _mediaState → settle durationMs → headless 600；
HAVE_NOTHING 空集合 + IndexSizeError；has-trap 白名单补列）+ **currentTime
setter seek 语义补全**：clamp 到 seekable 范围（spec seek 步 5——镜像写 clamp
后值；duration 未知只 clamp 下限 0）+ seeking/timeupdate/seeked 同一排队任务
序派发（seeking 异步——后挂 onseeking 可达；seeking 翻 false 先于 timeupdate
——Chromium 可观察语义；事件序 [seeking, timeupdate, seeked]）。
media-elements 552P/0F/24PF（+9 净涨零回归——seeking/ 三件 + volume_nonfinite
导入；buffered/seekable「上游无断言用例」的旧注记失效——seeking/ 目录即断言
面，DC-3 buffered/seekable 注记项收口）。
此前 2026-09-03：**M3 fixture-mounted runner 播放面切片 8 落地（扩批
XXV）**——loop-from-ended.tentative 导入 + 四处播放面缺陷收口：
① **registry Ended→play 解码器重建**——`play` 的 player Ended 态经 sources/
av_sources 留存字节 `reset()` 重建 + 伴生轨游标/静默线归零（此前直接置 Playing
下一拍即再 Ended——解码器单向流耗尽，重头播放从未真正工作）；
② **seek 游标 clamp**——av entry seek 后游标 clamp 到 player clamp 后位置
（语义层以 headless 600 算的 seek 目标可超真实流长，audio clock 主时钟游标超界
把视频位置拉出流末）；
③ **泵时钟注入**——`install_playback_bridge_with_clock`：桥 play 的 nowMs=0
（shim 无钟）翻译为宿主泵时钟现值，播放锚与 tick 同源（原点错位使首拍
delta=泵全程、位置瞬间跳到流末）；
④ **shim ended 态 play 语义**——play() 命中 `_zwEndedDispatched` 标记时派
seeking/seeked 回最早位置（spec「ended playback」步 6.4 在 play 入口生效；
loop setter 翻 ended 后 ms.ended 不可靠，以 march 非 loop 分支标记为判定面）+
loop=true IDL setter 翻回 ended=false（looping 媒体不能是 ended）。
media-elements 543P/0F/24PF（+1 净涨零回归——loop-from-ended 导入）。
此前同日：**切片 7（扩批 XXIV）**——loop 属性真面 + played TimeRanges：registry `set_loop`（音频 entry
流末回卷——`restart()` 解码器重建 + 游标归零 + 播放态保持；伴生轨同面；
`reached_end` 标志补音频面 isEnded 驱动源——此前音频流末对桥不可见）+
`registry_key` 规范化（strip query/fragment——WPT cache-buster query 与 runner/
shim 两侧 URL 编码差异同键命中；bridge play 的 audio_guess 同面——`.oga?...`
此前恒 miss 使音频条目永不登记）+ shim loop IDL setter/getter + march Ended 面
loop 分叉（seek(0)+play + seeking/seeked 派发非 ended）+ played TimeRanges
（march 采样 `_zwPlayedRanges` → getter TimeRanges 形状；loop 尾段计入）+
duration getter settle 竞态兜底。**setLoop 桥回调参数索引修复**
（args.get(2)→get(1)——门面传 2 参，此前 on 恒 false 使 loop 真面从未生效）。
media-elements 542P/0F/24PF（+2 净涨零回归——played-loop /
audio_loop_seek_to_eos 导入）。此前同日：**M3 fixture-mounted runner 播放面切片 6 落地**——
media load invoke 重置面收口：`_zwMediaScheduleLoad` invoke 入口重置
`_resourceStates[key]` + invoke 步 6 位置重置（currentTime=0 / HAVE_NOTHING / 
`_zwMediaTimeKnown` 失效）+ invoke 重置 track 子产物 cue（addTextTrack 产物
排除）+ settle 的 media/track 元素 load/error 派发改 `_zwMediaFire`
（handle-only 元素 on\* expando handler 兜底）；track-active-cues 导入——
**B 组排除件全清**；media-elements 540P/0F/24PF。此前同日：**切片 5**——
play() 桥 src 读身份分派（handle 身份走 registry 现值——createElement 媒体元素
形态的桥失联修复）+ march 遍历面统一（addTextTrack 产物 cue 调度）+ disabled
gate + cuechange 派发；media-elements 539P/0F/24PF。此前同日：**切片 4**——
HAVE_NOTHING 期 seek 挂起语义：currentTime setter readyState 0 时挂
`_zwSeekDeferred`，`_zwMediaLoadSequence` readyState 0→1 翻转时补跑 seek 算法
（spec「default playback start position」）；track-cues-seeking 导入；media-elements
535P/0F/24PF。此前同日：**M3 fixture-mounted runner 播放面切片 3 落地**——
解码器 EOF 排空缺陷修复：`VideoDecoder::next_frame` draining 中间态（demux 尽后
排空 hidden/alt-ref 帧滞后队列才报流末）+ `VideoPlayer::present_pending` 未来帧
`un_read` 队首退回——此前 position < duration 即提前 Ended（fixture-mounted
WPT 流的最小暴露面：test.webm 30fps + 15 个 alt-ref hidden 帧）；media-elements
534P/0F/24PF（+3 净涨——track-cues-enter-exit / pause-on-exit 解除排除）。
此前同日：**M3 fixture-mounted runner 播放面切片 2 落地**——
track-cues-* 播放推进族解锁：runner 播放桥前置 + 逐 tick 动态源登记 + shim play()
latest-wins 读/退避重试/pending seek 补推 + registry 字节留存/is_ended 桥面 +
march 区间捕获/事件时间序/ended 面；media-elements 531P/0F/24PF（+2 净涨）。
此前 2026-09-02：**M3 fixture-mounted runner 播放面切片 1 落地**——webview
`install_playback_bridge` + wpt-runner 播放泵/源登记 + shim `_zwMediaTimeMarchesOn`
cue 调度钩子；webm A_OPUS 解码切片（WebmOpusAudioTrack + registry codec 泛化 +
canPlayType webm-opus 扩表）；media-elements 529P/0F 零回归。此前同日：**M3 AV1
解码切片落地 + H.264 立项 RFC 起草**——
AV1：dav1d 绑定 feature `decode-av1` + VideoCodec 自路由 + fixture 48 帧全解
（ffmpeg 参照 ±15 窗）；H.264：[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)
Proposed 态——D-RFC-3a 专利授权链 / 3b OpenH264 分发形态 / 3c AAC 随期 三决策
点**待用户批复**（D-RFC-3「单独立项」决议的立项评估文档，批准前不动源码）。
此前 2026-09-01：D2 获批（选 A：libdav1d-dev 1.5.1 在位，pkg-config 发现，
apt 清单已记入 development/linux-macos.md））

---

## 当前状态

**专项定位**：媒体方向三拆之二（门控流）。视频解码与帧渲染——「占位框 → 能播放」的一跳。
**M0 已收口**：RFC 获批（2026-09-01，路线 C「VP9/AV1 开源先行 + 进程内 crate」）。
**M3 AV1 解码切片已落地（2026-09-02，D-RFC-2）**：`crates/media` 新模块
`av1_decode`（feature `decode-av1` 门控）——dav1d 安全 Rust 绑定（系统 libdav1d
1.5.1）+ Matroska V_AV1 轨 low-overhead OBU 喂入；`decode.rs` VideoCodec enum
codec 自路由（新 `open_webm`：V_VP9 → rusty_vp9 / V_AV1 → dav1d，feature 关闭
回落 NoVideoTrack 占位面；`open_webm_vp9` 原样保留零回归）；YUV→RGBA 提为通用
`planes_to_rgba`（VP9/AV1 共用，M2 色度面单点维护）；webview `video_registry.play`
切换 `open_webm`（生产播放面 codec 无关）。fixture `sample-webm-av1.webm` 48 帧
全解、PTS 单调、首帧 RGB 均值与 ffmpeg 7.1.5 RGBA 参照（123.26）同窗对齐 ±15。
media 40 单测（default）/ 42（decode-av1）全绿、webview 678 全绿、clippy 双态零警告。
**AV1 settle 探针接通 + canPlayType 扩表（2026-09-02 补片，跨 goal 联动兑现）**：
async_load `probe_video_media_meta` 从 `open_webm_vp9` 切 `open_webm`（与播放面
同一 codec 自路由入口）——修复「play 可路由而 settle 探针 VP9-only」的分叉
（AV1 源 settle 后 duration 真值 + 首帧注入缺失）；webview `decode-av1` feature
转发 + AV1 settle e2e（`video_settle_av1_first_frame_and_truth_m3`，feature-gated）；
media-elements canPlayType 能力表扩 av1（video/webm → probably——M4g-d
「新增解码面同步扩表」注记兑现）。media-elements 面 510P 维持零回归。
**M1a 已落地**（2026-09-01）：`crates/media`（`zero-media`）解码管线全通——
`VideoDecoder::open_webm_vp9` → 逐帧 `next_frame()` → `DecodedVideoFrame`（RGBA +
pts_ms）；fixture 48 帧全解、PTS 单调、首帧与 ffmpeg 7.1.5 rawvideo 参照**逐字节一致**
（探针实测），`VideoClock` trait（M2a 对接点）已定义。
**M1b 已落地**（同日）：首帧上屏通路全链打通——painter `paint_video_element`
（src 哈希 → ImagePrimitive，仅当解码像素已注入时发图元，占位行为零回归）+
pipeline `build_img_intrinsic_all` video 段（解码尺寸 → NodeId 固有尺寸表）+
layout-engine replaced sizing 白名单 +video + wpt-runner `load_video_first_frames`
（真实 fixture 首帧解码注入 ImageCache）。e2e 双测常驻（正例：真实帧像素上屏、
RGB 均值锚点；负例：不可解码 src 保持占位）。
**M2a 已落地**（同日）：`zero-media::player::VideoPlayer`——`VideoClock` 的帧率
驱动实现（PlayerState Ready/Playing/Ended 状态机 + play(now)/pause + tick(now)
位置推进与帧调度（pts ≤ position 呈现最新帧）+ playback_rate clamp + reset 重播）；
调用方注入单调时钟（rAF event loop P1a 挂点），单测 6 件常驻。
**M2a 切片 2 已落地**（同日）：duration 真值注入链全通——webview async_load 对
video 资源 fetch 成功后经 `probe_video_media_meta`（容器时长 + 首帧固有尺寸）真值化
`ResourceElementEvent`（新增 `media_duration_ms`，webview → zero-media 依赖）→
`script_commit_resource_element_state` 增第 6 参 → shim `_zwSettleResourceKey
.durationMs` → `_zwMediaLoadSequence` 以真值（ms→spec 秒）设 `duration`；无真值
（非 webm-VP9/headless 路径）回落定值 600——**testharness-media 372P/0F/0T/41PF
维持（零回归）**，单测 `test_media_duration_truth_injection_m2a` 3 断言组常驻。
**M2a 切片 3 已落地**（同日）：`videoWidth`/`videoHeight` IDL getter（part04 get
trap，VIDEO-gated 读 `_resourceStates.width/height`——切片 2 探针真值；未 settle 恒 0
per spec 元数据未就绪）+ has 白名单 tag-gated 分支（part05，`'videoWidth' in audio`
恒 false——接口成员归属面）。单测 `test_media_video_width_height_truth_m2a` 5 断言组
常驻；testharness-media 372P/0F/0T/41PF 维持（零回归）。**M2a 切片 4 已落地**（同日）：生产侧帧注入——`probe_video_media_meta` 扩为返回
首帧 RGBA（时长/尺寸/像素三真值一体），settle 时经 `ImageData::from_rgba` 注入
`webview.image_cache`（`ImageKey(image_resource_key(abs_url))`——与 painter
`image_resource_key(src, document_url)` 解析后同键，img/canvas 同款两段式）；非 webm
负例不注入（渲染占位零回归）。e2e 双测常驻（`video_settle_injects_first_frame_and_
truth_m2a` 真实 webm fixture 驱动 + `video_settle_non_webm_stays_headless_and_
placeholder` 负例）；webview 662 全绿。**生产侧首帧出图闭环达成。
**M2a 切片 5a 已落地**（同日）：`VideoPlayerRegistry`（webview `video_registry`
新模块）——`register_source`（settle 登记）/`play`（懒建 player，源未登记 no-op）/
`pause`/`current_time`/`duration`/`is_playing` 真值查询/`tick_all(now, ImageCache)`
（渲染泵推帧 + painter 同键注入 + changed 返回）/`release`（导航/元素移除资源释放）。
WebView 持 `video_players()` Arc 句柄；async_load settle 自动登记源字节（e2e 扩断言：
settle 后 `play` 即成功）。单测 4 件常驻；webview 666 全绿。**M2a 切片 5b 已落地**（同日）：三段接线全通——
① webview `register_video_bridge_callbacks`：五回调族（play/pause/current_time/
duration/is_playing，`Fn(&[String]) -> String` 契约）+ `__zwVideoBridge` JS 门面
（shim feature-detect 单点）；
② tab_js_worker `SetVideoPlayers` 命令（SetFetchHandler 同款 late-injection 模式）
+ `TabJsWorkerHandle::set_video_players`；tab_worker WebView 构建后注入
`wv.video_players()` Arc（settle 写入与桥读取同一实例）；
③ 帧泵：tab_worker 1ms 事件循环节拍——`is_any_playing` 快速门（无播放零开销）→
`tick_all(pump_epoch.elapsed(), image_cache)` 注帧 → changed 时增量重渲染 + snapshot。
shim 侧（part03/part04）：`play()` 桥接（bridgeSrc = IDL src getter 同源解析绝对 URL，
bridgeOn 标记）/`pause()` 桥停/`currentTime`/`duration` getter 桥真值优先——无桥环境
回落 headless（零回归）。测试：webview 桥 e2e（V8 sandbox + 真实 fixture roundtrip）
+ engine shim 契约测试（JS stub 桥：play 传绝对 URL / currentTime 1.25 / duration 2 /
pause 记录）；engine 2539 / webview 667 / browser 411 全绿；testharness-media 372
基线维持。**M2a 全部切片收口**。
**M2 切片 C 色度精化已落地**（2026-09-01，commit `f05493a07`）：`zero-media` 解码层
色彩面全对齐——
① **WebM Colour 解析**：`ColorSpace`（matrix/full_range）开流时经
`TrackEntry.Video.Colour` 解析一次（matroska-demuxer `colour()` API）；identity（GBR）
通道直传、BT.709/BT.601 矩阵选择、limited [16,235] 值域归一（ITU-R BT.601-7 §2.5/
BT.709-6 §2.5 标准形）；缺省：matrix None→BT.709 声明语义、range 非 Full→limited。
② **色度采样索引坍缩根因修复**：定点索引除数误用 chroma 维（应为 luma 维）——4:2:0
全行/列只采样前两个色度样点；旧锚点 153.5 为「全范围误释 + 该缺陷」叠加伪值，修正后
RGB 均值 123.3 与 ffmpeg swscale RGBA 参照对齐。
③ **值域误释修复**：声明 limited 的源此前按 full-range 数学转换（亮度 +25% 失真）。
④ **reftest-upstream 实证**：replaced-element-003（2x2-green.webm = identity+full，
位面真值 (0,127,0) vs ref #008000 差 1 ≤ fuzzy 0-30）✓ PASS——**R3881 以来唯一净
delta 的 false-pass unmask 案收口**；13950→13981/16730 = 83.6%（+31 净涨零回归）；
product-smoke 23.37% 逐字节同值（welcome 无 video，非回归）+ struct PASS。
⑤ 测试锚点同步：zero-media 新增 2 件（identity 直传位面真值 + limited 不削顶），
138-168 窗口更正 108-138（media 单测 + M1b e2e）。

**M2c 后续切片 A+B 已落地**（同日）：播放管线接 AudioSink 的生产侧全通——
① **settle 登记接通**（核心缺口修复）：`async_load::poll_element_resources` 对
video 源 `register_source` + `register_audio_source` 双登记、audio 源音频登记——
切片 5a 提交说明声称「async_load settle 自动登记源字节」但代码中从未存在生产调用方
（`register_source` 仅测试可达），宿主桥 play 在真实浏览器路径恒返 false 回落
headless；本片补全后 settle → 桥 play → 帧泵/音频泵全链生产可达（e2e
`media_settle_registers_playback_registry_m2c` 三面断言：webm video play 即成功 /
mp3 audio 音频 play 成功 / oga-opus 不登记回落）。
② **资源生命周期（DC-4）**：`prepare_document_state` 清空注册表（导航离开释放
player/解码器/源字节）+ `register_audio_source` 留存源字节（seek 重建解码器所需）+
registry `clear()` 单测。
③ **seek 追赶区静默**：`AudioEntry.skip_until_ms` 丢弃线——音频解码器单向流，seek
重建后前向解码至 target，target 前采样不入 sink（spec precise-seek；断言见
audio chain 单测）。
④ **增益联动全接**：shim play() 起播同步 `setGain`（play 前设置的 volume/muted 生效）
+ volume/muted IDL setter 桥推（切片 A 未提交工作树内容收编）；tab_worker 音频泵
（audio_advance_all）写入不计入重渲染触发（音频不改帧）。
⑤ **renderer 多进程路径对齐**（master.md 下一步 #2 兑现）：renderer js_worker 增
`SetVideoPlayers` 命令 + `set_video_players` 句柄 + runtime WebView 初始化后注入
`webview.video_players()`——镜像 tab_js_worker 切片 5b 同款，`__zwVideoBridge`
真值面两路径一致。
测试：webview 674（+6）/ engine 2540 / make test 18636 全绿；testharness-media
372P/0F/41PF 基线维持；clippy 零警告。
**M2c 音频解码面已落地**（同日）：`zero-media::audio_decode::AudioDecoder`——
symphonia probe 自动识别容器/编码 → f32 交错 PCM 逐包输出（`copy_to_vec_interleaved`
跨格式直转，值域 [-1,1] 对齐 AudioSink 契约）；损坏包跳过（DecodeError 面继续）；
依赖 `symphonia = { 0.6, default-features = false, features = [mp3, ogg, vorbis, pcm] }`
（纯 Rust 零 C 依赖，路线 C 约束保持）。**opus 选型注记**：symphonia 0.6 无 opus
解码器（libopus 为 C 依赖）——`sample-ogg-opus.oga` 不在 M2c 面，留待后续选型评估
（新增 vorbis fixture 补齐 ogg 容器面）。全链 e2e 双件常驻：`audio_mp3_decode_to_
nullsink_zero_crossing_chain` / `audio_ogg_vorbis_decode_to_nullsink_chain`——fixture
（440Hz sine）→ 解码 → NullSink 过零率 ≈880（2×频率锚点）+ 采样数 ≈2s 容差面；
media 30 全绿。**余：A/V 同步（media-audio M2，audio clock 主时钟）+ 播放管线接
AudioSink（video play 时同步音频面——M2c 后续切片）**。
**M2 切片 D+E 已落地**（同日，A/V 同步——media-audio M2 契约兑现）：
① **切片 D（webm 双轨伴生音频面）**：`zero-media::av_decode::open_webm_audio_track`
——Matroska demux A_VORBIS 轨 + CodecPrivate 三段 Xiph 头拆解 → OGG 页重封装
（RFC 3533 纯字节操作，零 C 依赖，路线 C 保持）→ symphonia ogg/vorbis 解码 f32 PCM
（与独立音频面同契约）。granule 语义注记：symphonia ogg reader 按页 granule 对包做
端部裁剪，granule 小于包累计时长会把采样整段裁空（实测 0 样本缺陷）——本轨无 seek，
每页取 (i+1)×16384 保守高估使裁剪恒零；数据页不设 EOS 位（symphonia 以 EOF 判流末）。
fixture `sample-webm-vp9-vorbis.webm`（VP9+Vorbis 双轨 2s）。registry 增
`WebmAudioEntry` 伴生条目：video play 懒建、同锚起播、泵推进写 NullSink、增益联动、
pause 冻结；纯视频 webm 无音频轨静默。
② **切片 E（audio clock 主时钟）**：`VideoPlayer::sync_to_media_time`——视频帧调度
对齐外部主时钟游标（位置每 tick 派生自主时钟——drift 构造校正不积累墙钟差；位置只
前进不回退；帧调度核抽 `present_pending` 供 tick/sync 共用，墙钟路径零回归）。
`tick_all` 主时钟先行序（伴音频轨先推进 → A/V pair 视频 sync 追随游标 → 纯视频回落
墙钟 tick）。currentTime 组合时钟：A/V pair 优先报音频游标。seek 双轨对齐：
`av_sources` 字节留存 + 伴生轨重建（游标/静默线对齐 target）——master clock 面不脱轨。
`WebmAudioEntry` 补 `skip_until_ms` 追赶区静默（AudioEntry 同面）。
③ 测试：zero-media 36+1（av_decode 3 件：解码链 44.1kHz 2s≈88200 帧 + 440Hz 过零率
880 锚点 / 双轨下 VP9 视频轨 48 帧零回归 / 无音频轨 NoTrack feature-detect；player
sync 4 面断言）；webview 676（registry AV pair master-clock e2e + seek 对齐续进）；
engine 2542 全绿。

**M2b 已落地**（同日）：精确 seek + 变速桥接全链——
① `VideoDecoder::seek_to_ms`：两阶段精确 seek——phase ① demuxer Cues 定位 +
  **keyframe 落点验证**（`block.is_keyframe` gate：cue 点即 keyframe；非 keyframe
  落点——无 Cues 流的线性搜索——参考链断裂不可靠）→ phase ② 全量回退（seek 0 +
  解码器重建 + 前向解码至 ≥ target 首帧，帧存 `pending` 不丢——spec precise-seek）。
  实测锚点：fixture 单 keyframe（testsrc2 无 GOP 内刷新，ffprobe 实证全流唯一 K 帧
  pts=0）→ seek(1000ms) 回退路径命中 pts≥1000 首帧、续播 PTS 单调至末；seek(0)
  全流 48 帧完整重播；
② `VideoPlayer::seek_to_ms`：位置 clamp [0,duration]、播放中 seek 保持播放（时钟
  锚点重置防 Δt 跳变）、暂停中 seek 保持暂停（spec）、pending 帧先弹出；
③ registry/桥：`seek`（未 play 的已登记源自动建 player 且置暂停——spec「seek 不改
  paused」）+ `set_playback_rate`；桥回调 `__zw_video_seek`/`__zw_video_set_rate`
  + JS 门面 `seek`/`setRate`；
④ shim：`currentTime=` setter 桥推（seeking/seeked 事件序列不变——headless 断言面
  保持）+ `playbackRate=` setter 桥推（ratechange 不变）+ `play()` 起播时同步既有
  速率。
测试：zero-media seek 2 件 + player seek 2 件；registry seek 1 件 + 桥 e2e seek 断言；
engine 2539 / media 27 / webview 668 全绿；testharness-media 372P/0F/41PF 维持。

**与兄弟 goal 的边界**：
- media-elements — 语义面（状态机/事件/canPlayType）归其管；本目标产出 readyState 真实
  驱动接口（`VideoClock` trait，其 headless 近似驱动届时替换，语义层不返工——RFC §3.1）
- media-audio — 音频输出/A/V 同步归其管（其 M0 已收口，AudioSink trait 验证策略成立）；
  本目标首期静音播放（video clock 驱动），音频解码面 M2c 经其 AudioSink 接入
- js-dom — 媒体反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项 + 2026-09-01 M0/M1a/M1b 更新）

### 现有实现

- ✅ **解码管线（M1a）**：`crates/media`（`zero-media` crate）——webm/Matroska
  demux（`matroska-demuxer 0.8`，纯 Rust）+ VP9 解码（`rusty_vp9 0.1`，纯 Rust 零 C
  依赖）+ YUV→RGBA（BT.601，8/10/12bit 与 4:2:0/4:2:2/4:4:4 面宽）；单测 5 + doctest 1
  常驻（fixture 帧数/PTS 单调/像素窗口/确定性/拒收非 webm）
- ✅ **帧上屏通路（M1b）**：painter `paint_video_element`（ImagePrimitive，解码像素
  gate）→ pipeline video 固有尺寸段 → layout-engine replaced sizing 白名单 →
  wpt-runner `load_video_first_frames`（harness 侧解码注入）；e2e 双测常驻
  （`m1b_video_first_frame_renders_to_framebuffer` 正例 + undecodable 负例）——
  证据：[evidence/2026-09-01-m1b-first-frame-on-screen.md](evidence/2026-09-01-m1b-first-frame-on-screen.md)
- ✅ **播放驱动接口（M1a 定义）**：`VideoClock` trait（currentTime/duration/is_playing/
  playbackRate）——M2a player 模块实现、语义层对接点
- ✅ 架构先例：image-decoder 独立进程（D1）+ zero-protocol IPC（`ImageDecodeParams/
  Result` 字节进 RGBA 出——视频解码进程升级时同构扩展）+ compositor（C2）
- ✅ 渲染通路：canvas 像素 → 页面图元桥接（R3268）——M1b 已按同款两段式落地
- ✅ event loop 帧驱动：rAF（P1a）——播放时钟可挂
- ✅ **e2e 资产已备**（V5 闭合）：`tests/fixtures/media/` 四 fixture（h264+aac mp4 /
  vp9 webm / mp3 / opus oga，ffmpeg 生成、来源清白、生成命令入 README）
- ✅ crate 生态调研数据：symphonia 0.6（纯 Rust 容器+音频）/ dav1d 0.11（AV1 绑定）/
  openh264 0.9 / ffmpeg-next 9.0 / rav1e 0.8（crates.io 实测版本）；M1a 实测补充——
  `rusty_vp9 0.1.1`（纯 Rust VP9，Apache-2.0，MSRV 1.85）首帧与 ffmpeg 逐字节一致，
  `matroska-demuxer 0.8.1`（Zlib OR MIT OR Apache-2.0）API 干净（双许可证均兼容工作区 MIT）；
  M2c opus 面实测补充——`opus-decoder 0.1.1`（纯 Rust RFC 6716/8251 decoder，零 unsafe
  零 FFI，仅依赖 thiserror，MIT OR Apache-2.0，MSRV 1.85，conformance 测试常驻）入
  workspace 依赖
- ✅ **生产侧帧注入 + 播放桥（M2a 切片 4/5 + M2c 后续 A/B）**：settle 首帧注入
  ImageCache + 源字节登记；`VideoPlayerRegistry` + `__zwVideoBridge` 宿主桥 +
  tab_worker 帧泵（切片 5a/5b）；settle 登记生产链路补全 + renderer 多进程路径
  SetVideoPlayers 对齐（M2c 后续切片 A/B）——双路径（tabworker/renderer）一致性
- ✅ **音频播放面（M2c 后续 A/B）**：`<audio>` settle 登记 → 桥 play → 音频泵实时
  节奏解码写 NullSink + volume/muted 增益联动（IDL setter 桥推 + play 起播同步）+
  seek 追赶区静默；导航释放（DC-4）
- ✅ **色彩面（M2 切片 C）**：WebM Colour 解析 + identity/BT.709/BT.601 矩阵 +
  limited/full 值域自适应转换；色度采样索引与值域两处旧缺陷修复——与 ffmpeg
  swscale 参照对齐；replaced-element-003 unmask 收口
- ✅ **A/V 同步面（M2 切片 D+E）**：webm 双轨伴生音频（OGG 重封装 → symphonia）+
  audio clock 主时钟（视频帧调度 sync_to_media_time 对齐音频游标）+ currentTime
  组合时钟 + seek 双轨对齐（media-audio M2 契约兑现——drift 构造校正）
- ⚠️ AV1（dav1d 绑定）与 H.264 未引入——M3（D-RFC-2 / D-RFC-3 决议）

## 深结构缺口发现（2026-09-02，Web Audio 多进程接线巡检）

**renderer 路径无播放泵**（记录不修——深结构）：browser tab_worker 主循环有 1ms
帧泵/音频泵（`is_any_playing` 门 → `tick_all` + `audio_advance_all` + WebAudio
`wa.advance`）；renderer 路径 M2c 切片 ⑤ 仅对齐了**桥面**（`__zwVideoBridge` 注入
js_worker）而**从未 tick**——renderer 的 VideoPlayerRegistry Arc 注入后无消费方，
play 真值面在 renderer 播放请求可登记但帧/音频永不推进（`is_any_playing` 恒真
即自旋、无泵即永不推进——实际为「play 登记后静默」）。Web Audio 同理：本轮已补
`SetWebAudio`/`__zwWA*` 注入（桥面一致性），但 `WebAudioRegistry.advance` 无
renderer 主循环节拍驱动。**修复方向**：renderer 主循环（`runtime.run`，当前
事件驱动无固定节拍）引入播放泵节拍或迁移独立泵线程——架构决策域（进程内线程
模型 vs 事件循环节拍），待用户点名后实施；在此前 renderer 路径播放面维持
「登记但不推进」现状（与 M2c 切片 ⑤ 交付态一致，非回归）。

## DC 达成审计（2026-09-02，对照入口文档 Done Criteria 逐项核验）

**DC-1（选型 RFC 已批准并落地）✅**：主 RFC 获批（D-RFC-1/2/3 三决议，2026-09-01）；
实现与 RFC 一致（路线 C：进程内 crate + feature gate——`decode-av1` 默认关、VP9
纯 Rust 主线）；偏离处零（dav1d 为 RFC §4 明示的 M3 面）。H.264 增量按 D-RFC-3
「单独立项」决议起草独立 RFC（Proposed，待批复）——不属 DC-1 范围。

**DC-2（首个视频端到端播放）✅**：① 真实 fixture → demux → 解码 → 首帧上屏
（M1a/M1b，`load_video_first_frames` + settle e2e 双测常驻）；② 连续播放
（M2a VideoPlayer：帧率驱动/currentTime 推进/play/pause/seek/ended——单测 6 件
+ 桥 e2e）；③ 帧渲染走页面图元通路（painter `paint_video_element` →
ImagePrimitive，与 canvas R3268 同通路）。

**DC-3（语义驱动真值化）✅（buffered/seekable 注记）**：duration 真值注入链全通
（M2a 切片 2——容器时长 → settle → shim `_zwMediaLoadSequence`）；readyState 由
settle 事实驱动（headless 加载序列推进 HAVE_METADATA→HAVE_ENOUGH_DATA）；videoWidth/
videoHeight 解码器探针真值（M2a 切片 3）。**buffered/seekable TimeRanges headless
近似面已收口（2026-09-04 扩批 XXVI）**——`__zwMediaSeekableRanges`（readyState>=1
后 [0,duration] 单区间 + IndexSizeError）+ seeking/ 三件断言用例导入（此前「上游
无断言用例」注记失效——seeking/ 目录即断言面）；真值化依赖真解码流的缓冲区间
追踪（随播放面背压优化一并评估，记录为后续项）。

**DC-4（多格式 + 稳定性）🔄（余 WPT 子集导入）**：① 选型面内容器/编解码 e2e
（VP9 单轨/双轨 + AV1 decode→settle→play→canPlayType 全链 ✅）；② 资源生命周期
（`prepare_document_state` 清空注册表 + `clear()` 单测——DC-4 导航释放面 ✅）；
③ **上游 WPT 可执行子集导入未启动**（master.md 下一步 #1 尾项——runner 桥注入
可行性分析已记：WPT corpus 无 settle 真源，注入后行为零变化；待 fixture-mounted
runner 播放用例面评估，随 D-RFC-3 批复状态一并决策）。

**DC-5（测试与质量不可退让）✅**：make test 18694 全绿（2026-09-02 组合树实测）、
clippy 零警告、每切片带单测 + e2e/fixture 资产化（AV1 全流单测 + settle e2e +
桥 roundtrip）。

**结论**：DC-1/2/3/5 满足；DC-4 余 WPT 可执行子集导入一项（外部门控：D-RFC-3
批复影响其形态——H.264 批准则 mp4 面 WPT 用例可随实施导入，不批准则 webm 面
维持 headless 饱和态）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | 解码路线选型（专利/依赖/架构三维） | ✅ RFC 获批（路线 C，2026-09-01） |
| V2 | 零解码管线（demux/解码/帧转换） | ✅ M1a 落地（2026-09-01，`zero-media` crate） |
| V3 | 播放驱动（帧率时钟/seek/ended）缺失 | ✅ M2a + M2b 落地（时钟/play/pause/ended/精确 seek/playbackRate 变速桥接） |
| V4 | readyState 真值驱动接口未建 | ✅ 5b 落地：duration/videoWidth/currentTime 真值 + 宿主桥 play/pause + 帧泵（readyState 推进面仍 headless 序列——语义层契约不返工） |
| V5 | 播放 e2e 资产为零 | ✅ fixture 已落地 + M1b 帧上屏 e2e 常驻 |
| V6 | 帧上屏通路（video 元素盒 → 图元）缺失 | ✅ M1b（harness 全链）+ 切片 4（生产 settle 注入）+ 切片 5b（播放帧泵） |
| V7 | 色度元数据精化（Colour 元素/limited-range/BT.709） | ✅ M2 切片 C 落地（2026-09-01）——replaced-element-003 unmask 收口，reftest-upstream 83.6% |
| V8 | A/V 同步（audio clock 主时钟）缺失 | ✅ M2 切片 D+E 落地（2026-09-01）——webm 双轨伴生音频 + 视频帧调度对齐音频游标 + currentTime 组合时钟 + seek 双轨对齐（media-audio M2 契约） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ✅ 获批（2026-09-01）——三项决议见 RFC §5 |
| D2 | **AV1 dav1d 依赖引入方式**（M3 解锁前置）：本机实测——系统有 libdav1d7 运行时
  （.so.7）但**无 libdav1d-dev 头**（pkg-config 找不到）；`dav1d 0.11` crate 的
  dav1d-sys 走 system_deps：优先 pkg-config 系统库，缺则**从源码构建**（git clone
  videolan/dav1d + meson + ninja——本机 meson/ninja 均未装）。两条路都需系统级安装
  （`apt install libdav1d-dev` 或 `apt install meson ninja`），按 run-rules 须用户
  批准；三平台 CI 构建矩阵成本同 RFC §6 风险面 | ✅ 获批选 A（2026-09-01，
  GB-20260901 批复）——`libdav1d-dev 1.5.1-1` 已装，pkg-config 发现 dav1d 1.5.1；
  apt 清单已记入 [docs/development/linux-macos.md](../../development/linux-macos.md)（不阻塞 M3 其余面——
  WPT 子集导入可先行） |
| D3 | **H.264/AAC 增量立项批复（D-RFC-3a/3b/3c）**——立项 RFC 已起草
  （[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)，
  2026-09-02 Proposed）：推荐路线 A（Cisco OpenH264 `openh264 0.9` 安全 Rust 绑定 +
  symphonia aac feature 扩展；本机探针实证 48/48 帧解码）；三决策点：
  **3a** 专利授权链是否接受（核心门禁——MPEG-LA/Via 池面，Cisco AVC Patent Trust
  License 授权链 + 源码编译态确定性注记，本 RFC 不构成法律意见）；
  **3b** OpenH264 分发形态（① 构建期源码编译【推荐，与路线 C 轻依赖一致】/
  ② 官方预编译二进制——授权链最强但分发矩阵成本回潮）；
  **3c** AAC 是否随期（推荐随期——symphonia feature 扩展成本 ≈0）。
  为何需用户：专利/授权属 Mission 级决策（run-rules rule 11 + 主 RFC D-RFC-3
  「单独立项」决议），agent 不可代判 | ⏳ 待批复（2026-09-02 起草，飞书已征询
  msg `om_x100b664d8a6f44b0dee3398474de92b`；批准前不动源码；不批准亦请明示
  「维持不实施」以便归档） |
| D4 | **renderer 路径播放泵架构决策**（2026-09-02 深结构缺口发现，2026-09-02 巡检
  补入决策表）：browser tab_worker 主循环有 1ms 帧泵/音频泵（`is_any_playing` 门
  → `tick_all` + `audio_advance_all` + WebAudio `wa.advance`），renderer 路径桥面
  已对齐（VideoPlayerRegistry Arc + `SetWebAudio`/`__zwWA*` 注入）但**主循环无节拍
  驱动 advance**——play 登记后帧/音频永不推进（「登记但不推进」现状，非回归）。
  修复须架构决策：进程内独立泵线程 vs 事件循环节拍（renderer `runtime.run` 当前
  事件驱动无固定节拍）
  为何需用户：多进程线程模型属架构决策域（run-rules rule 11 深结构），待点名后
  实施 | ⬜ 待点名（此前仅记于「深结构缺口发现」块，决策表不可见——2026-09-02
  巡检补登；不影响 tab_worker 路径现有功能） |

## 下一步计划

1. **M3 多格式收尾**（当前首选）：~~AV1~~ ✅ 2026-09-02 落地（解码切片 +
   codec 自路由 + fixture 48 帧全解——见当前状态）；**H.264 立项 RFC 已起草
   （2026-09-02，[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)
   ——Proposed 态，D-RFC-3a（专利授权链）/3b（OpenH264 分发形态）/3c（AAC 随期）
   三决策点待用户批复，批准前不动源码）**；上游 WPT 可执行子集导入。
   **M3 预备资产已落库（2026-09-01）**——
   `sample-webm-av1.webm`（libaom-av1 生成，README 命令记录）；matroska-demuxer
   实测可枚举 V_AV1 轨（CodecPrivate 在）——demux 面就绪，解码切片
   （dav1d 绑定 + `open_webm` track 路由 V_AV1）直接以本资产验证。
   **runner 桥注入可行性分析（2026-09-01）**：wpt-runner 沙箱可注入
   `register_video_bridge_callbacks`（tab_worker 同款）+ take_probe 泵 tick——
   但 WPT corpus 无 settle 真源（play() 桥 play 返 false 回落 headless），注入后
   行为零变化、无新增可跑用例；待 D2（AV1 fixture 资产面）落地后一并评估
   fixture-mounted runner 播放用例面。
   **切片 1 落地（2026-09-02，DC-4 WPT 子集导入前置）**：① webm A_OPUS 解码
   （`WebmOpusAudioTrack`——Matroska demux + CodecPrivate=OpusHead 解析 +
   opus-decoder 逐包直解，无 OGG 重封装；movie_5.webm 实测 5.01s/pts 单调/
   48kHz）+ registry 伴生轨 codec 泛化（`WebmAudioTrackKind`，Vorbis/Opus 双形态
   Box 装箱）+ canPlayType video/webm opus 扩表（'vp9, opus' → probably——WPT
   common/media.js getVideoURI 判定串解锁，此前该判定 '' 使 URI 落不存在的
   .mp4）；② wpt-runner 播放面：webview `install_playback_bridge()`（同进程嵌入方
   桥注册入口）+ 页面脚本后 media src 提取登记（extract_media_resources →
   wpt-data 字节 → registry）+ probe 循环播放泵（tick_all + audio_advance_all，
   playback_clock_origin 单调时钟与桥 play(0) 对齐——tab_worker pump_epoch 同
   契约）+ shim `_zwMediaTimeMarchesOn` cue 调度钩子（桥真值钟推进 enter/exit/
   pauseOnExit——media-elements track-cues-* 播放推进族解锁面）。探针实证
   play→playing→promise resolve→currentTime 真值推进（0.53s@500ms）全链通；
   media-elements 529P/0F/24PF 零回归。**余**：WPT 播放推进用例导入
   （track-cues-enter-exit / missed / seeking 等——cue 时钟推进面已就绪，
   seek 语义面待切片 2）。
   **切片 2 落地（2026-09-03，track-cues-* 播放推进族解锁——media-elements
   M3 扩批 XVI 兑现）**：① runner 播放桥**前置**页面脚本（execute_script 预热
   ensure_sandbox/ensure_js_shim 后 install_playback_bridge——canplaythrough 内
   同步 play 可达桥）；② probe 循环**逐 tick 动态源登记**（JS 快照 media 现值
   src → wpt-data 字节 → registry；contains_source 幂等 + Rc 字节缓存；settle
   提交随首次登记）；③ shim play() 桥 src 改 latest-wins 读 + `_hit` 单次调用 +
   未命中退避重试（setTimeout 0 × 5000 上限）+ pending seek 补推（seek-before-
   play 落位）；④ registry play 未命中**不消费**源字节（sources.get 克隆、命中
   才 remove——旧形态一次失败即丢字节，单测
   `play_miss_retains_source_bytes_for_retry`）+ `contains_source` + `is_ended`
   桥面（`__zw_video_is_ended`/`isEnded`）；⑤ march 钩子**区间捕获 + 事件时间
   序派发**（跨 (lastMs, nowMs] 收集 enter@start/exit@end 按时间排序——1ms cue
   不再整体跳过；seek 判据改「时钟回退 ∨ seeking 标志」）+ **ended 面**（active
   cue 倒序 exit flush + timeupdate + ended）。WPT 实证：media-elements 531P/
   0F/24PF（+2 净涨——track-cues-enter-seeking + track-cues-missed 稳定全绿；
   enter-exit 因注册竞态 1/4 flake 暂排除，随泵节拍精化复评）。**余**：WPT 播放
   推进用例续批（enter-exit 复评 + seeking/sorted/pause-on-exit 等随基础设施
   增量逐件）。
   **切片 3 落地（2026-09-03 续，解码器 EOF 排空缺陷修复——播放推进族余件
   解锁）**：宿主插桩实证（console→tracing + 泵侧 Rust 快照）定位两类
   zero-media 解码层缺陷：① `VideoDecoder::next_frame` 的 demux 耗尽分支 flush
   后仅 pull 一帧即置 `eof`——rusty_vp9 的 hidden/alt-ref 帧（show_frame=0）
   解码后返 `Again` 不产出，每次消耗一个 pull 机会但其 pts 帧晚一个 demux 块
   浮现，形成 ~15 帧（≈0.5s）输出滞后；积压帧被提前置位的 eof 吞掉，
   `present_pending` 在 position < duration 处遇 `Ok(None)` 即转 Ended
   （test.webm 实测 Ended@媒体时间 3.57s / 流长 6.035s；WPT 形态 =
   track-cues-enter-exit 的 cue@4-5s 永不触发）。修复：`draining` 中间态 +
   `drain_frame`（Again=隐藏帧继续拉、Eof=队列真空才停）；seek 双分支同步
   重置。② `present_pending` 遇 `pts > position` 的未来帧把它返回给调用方
   （渲染后丢弃）——时间槽永久丢失，粗 tick 背压下逐 tick 丢未来帧使解码器
   再次提前耗尽。修复：`VideoDecoder::un_read` 队首退回（pending 槽复用）——
   spec ended「currentTime 到达流末」，帧调度不得超越时钟消费时间线。修复后
   模拟 181 帧全呈现、position=6.0 才 Ended；WPT：media-elements 534P/0F/24PF
   （+3 净涨——enter-exit / pause-on-exit 解除排除，4 连跑稳定）。单测
   `webm_sequential_decode_drains_hidden_tail_frames_r3936`。**余**：WPT 播放
   推进用例余面（track-cues-seeking——seek 事件真值化复评；其余 B 组随基础
   设施增量逐件）。
   **切片 6 落地（2026-09-03 续，media load invoke 重置面——B 组排除件全清）**：
   扩批 XXII 后 track-active-cues 复评仍 Timeout，dormant 插桩
   （`__zwPauseWatch` + `ZW_MEDIA_SEEK_DEBUG` runner 门控，验证后移除）定位双
   根因：① invoke 入口 `_resourceStates[key]` 未重置——settle 幂等门吞掉
   `src=''` 二次调度的 error 提交；② settle 的 load/error 派发传 handle=null——
   handle-only（createElement）元素 listener 键恒失配，on\* expando 兜底
   （`_zwMediaFire`）未接。修复四处（shim part06）：invoke 重置 settle 面 +
   invoke 步 6 位置重置（currentTime=0 / `_zwMediaTimeKnown` 失效）+ invoke 重置
   track 子产物 cue（addTextTrack 产物排除）+ settle 的 track/audio/video
   load|error 派发改 `_zwMediaFire`。track-active-cues 导入（540P/0F/24PF，
   +1 净涨零回归；单测 `test_media_load_invoke_reset_face_m3xxiii`）。
   **B 组排除件至此全清**（XX disabled/no-cuechange/remove-active-cue + XXIII
   track-active-cues）。**余**：WPT 播放推进用例余面（playing-the-media-resource
   的 play-in-detached-document / loop-from-ended / fragmented-mp4-end——依赖
   detached 文档播放钟与 loop 面）。
   **切片 7 落地（2026-09-03 续，扩批 XXIV——loop 真面 + played TimeRanges）**：
   registry `set_loop`（音频 entry 流末回卷：`restart()` 解码器重建 + 游标归零 +
   skip 线清零 + 播放态保持——解码器单向流 fixture 级小源可接受；`WebmAudioEntry`
   伴生轨同面；`reached_end` 标志——loop=false 流末置位 / play·seek·restart 清除，
   补齐音频面 `isEnded` 桥驱动源）+ `registry_key` 规范化（strip query/fragment，
   全查询面统一——WPT cache-buster `?...Math.random()` 与 shim/runner URL 编码
   差异此前两侧键失配；bridge play 的 `audio_guess` 同面 strip，`.oga?...` 此前恒
   miss）+ shim loop IDL setter/getter（`_mediaState.loop` 镜像 + 桥推送 +
   play 起播/retry 路径同步）+ march Ended 面 loop 分叉（seek(0)+play 重头 +
   seeking/seeked 异步派发非 ended；非 loop 分支补 `ms.ended = true` IDL 置位）+
   played TimeRanges（march 逐拍采样 `_zwPlayedRanges` 区间合并 250ms 容差 →
   getter `length/start/end` + IndexSizeError；seek 回退不合并；loop 回卷前尾段
   `[lastMs, duration]` 按桥真值计入）。**缺陷修复**：`__zw_video_set_loop` 回调
   读 `args.get(2)` 而门面 `setLoop(src, on)` 只传 2 参——`on` 恒 false，loop 真
   面从未生效（本片单测实证回卷后修正）。media-elements 542P/0F/24PF（+2 净涨
   零回归——played-loop / audio_loop_seek_to_eos 导入；loop-from-ended 暂不导入，
   动态 src settle 竞态注记于 testharness.rs）。单测
   `registry_audio_loop_restarts_at_stream_end`（回卷 ≥2 次 + 播放态保持 +
   loop=false 对照停）。
   **切片 8 落地（2026-09-03 续，扩批 XXV——loop-from-ended 导入 + Ended→play
   全链收口）**：dormant 探针链（8 轮 probe，验证后全移除）定位四处缺陷：
   ① registry `play` 的 player Ended 态直接置 Playing——解码器耗尽下一拍即再
   Ended（seek(0) 也无效）；经 sources/av_sources 留存字节 `reset()` 重建 +
   伴生轨 `cursor/skip/last_tick` 归零（current_time 优先读 av 游标——不归零则
   Ended→play 后桥钟读数恒在流末）；② registry `seek` 的 av 游标未 clamp——
   语义层以 headless duration 600 算的 seek 目标（599.5s）超真实流长（5.008s），
   audio clock 主时钟游标超界把视频位置拉出流末（ended 面 currentTime 读数
   失真）；clamp 到 player clamp 后位置；③ **泵时钟原点错位**——shim 桥 play
   恒传 nowMs=0 而泵 tick 用宿主单调钟 elapsed，首拍 delta=泵全程使位置瞬间跳
   到流末；`register_video_bridge_callbacks` 增 `clock: Option<Arc<AtomicU64>>`
   参数 + webview `install_playback_bridge_with_clock`，runner 泵时钟注入——
   play 锚与 tick 同源（tab_worker pump_epoch 语义同构，后续可同法注入）；
   ④ shim `play()` 无「ended 态 play」语义——spec「ended playback」步 6.4 在
   play 入口即生效（seek 回最早位置 + 派 seeking/seeked 非 ended），以 march
   非 loop 分支的 `_zwEndedDispatched` 标记为判定面（loop setter 已把 ms.ended
   翻回 false——spec「looping 媒体不能是 ended」，IDL 态不可靠）+ loop=true
   setter 同步翻 ended=false。loop-from-ended.tentative 导入
   （543P/0F/24PF，+1 净涨零回归）。
   **切片 9 落地（2026-09-04 续，扩批 XXVI——seekable/buffered TimeRanges +
   seek clamp）**：① `__zwMediaSeekableRanges`（part06 共享 helper，part04
   get trap 的 seekable/buffered 统一入口）：readyState>=1 后 [0,duration]
   单区间、HAVE_NOTHING 空集合、越界 IndexSizeError；duration 解析序 桥真值 →
   _mediaState.duration → settle durationMs → headless 600；has-trap 白名单补
   seekable/buffered（part05）。**DC-3 的「buffered/seekable 未实施——上游无
   断言用例」注记失效**（seeking/ 目录即 seekable 断言面）——headless 近似面
   收口，流式真值化随播放面背压优化另评。② currentTime setter seek 语义补全：
   clamp 到 seekable 范围（spec seek 步 5——镜像写 clamp 后值，duration 未知只
   clamp 下限 0）+ seeking/timeupdate/seeked 同一排队任务序派发（seeking 异步
   化——后挂 onseeking 可达；seeking 翻 false 先于 timeupdate——Chromium 可
   观察语义；事件序 [seeking, timeupdate, seeked]）。seeking/ 三件
   （seek-to-currentTime/max-value/negative-time）+ volume_nonfinite 导入
   （552P/0F/24PF，+9 净涨零回归）。**余**：playing-the-media-resource 余面
   （play-in-detached-document / fragmented-mp4-end）。
2. ~~**A/V 同步精化余项**~~ ✅ 2026-09-01 收口：ended 面回归守卫落地
   （切片 F——伴音流末 video player 走到 Ended、泵停）；音频设备面（CpalSink
   真出声）挂 media-audio M1 可选切片。
3. ~~**opus 解码选型注记**~~ ✅ 2026-09-01 落地（`opus-decoder 0.1.1` 纯 Rust 面——
   评估结论：libopus 绑定族全部违反路线 C；pure-Rust 候选对比后选 opus-decoder
  （RFC 8251 conformant + conformance 常驻 + 零依赖）；音频输出格式面收口为
   mp3 + vorbis + opus 三编解码）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 完成并获批（2026-09-01，路线 C） |
| M1 — 首个视频帧上屏 | ✅ 完成（2026-09-01：M1a 解码管线 + M1b 帧上屏通路 + e2e 常驻） |
| M2 — 连续播放 + 语义驱动 | 🔄 M2a + M2b + M2c + 切片 C/D/E/F 收口（播放/真值/桥/帧泵/seek/变速 + 音频面生产链路/增益/导航释放/renderer 对齐 + 色彩面全对齐 + A/V 同步 audio clock 主时钟 + A/V pair ended 面回归守卫）；余音频设备面（media-audio M1 CpalSink，可选） |
| M3 — 多格式 + 稳定 + 收尾 | 🔄 AV1 ✅（2026-09-02，D-RFC-2：解码/settle/播放/canPlayType 全链 + fixture e2e）；余 H.264 立项（D-RFC-3，RFC Proposed 待批复）+ WPT 可执行子集导入（外部门控：随 D-RFC-3 批复状态决策形态） |

## 验证基线

- 测试基线：`make test` 全绿（zero-media default 23 单测 + 1 doctest；engine 2539
  含桥契约测试；webview 667 含桥 e2e + registry 4 + settle e2e 2；browser 411 under
  xvfb）；clippy 零警告；testharness-media 372P/0F/41PF 基线维持（桥 feature-detect
  回落面零回归实证）
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 上屏 e2e 锚点：帧区 RGB 均值 108-138（testsrc2 ≈123.3，M2 切片 C 后与 ffmpeg
  swscale 一致）+ 帧界外白底 + 不可解码 src 占位负例；reftest-upstream
  13981/16730（**83.6%**，replaced-element-003 unmask 已收口 ✓）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## 归档

- [archive/2026-09-01_m0-m2-slices.md](archive/2026-09-01_m0-m2-slices.md) —
  M0 门控收口与 M1/M2 切片全链过程记录（只追加不修改；本控制面保留最新态）。
