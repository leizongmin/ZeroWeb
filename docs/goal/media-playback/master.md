# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**M2c opus 面落地**——`opus-decoder 0.1.1` 纯 Rust
（RFC 6716/8251，零 unsafe 零 FFI，MIT OR Apache-2.0）补齐 symphonia 0.6 缺位：
`opus_decode::open_ogg_opus`（symphonia ogg reader 容器 demux + OpusHead extra_data
解析 + pre-skip 丢弃）→ registry 双面回落登记（symphonia → opus）。`sample-ogg-opus
.oga` 从「不登记回落」转正为可播面。media 38+1 / webview 677 / engine 2546 全绿）

---

## 当前状态

**专项定位**：媒体方向三拆之二（门控流）。视频解码与帧渲染——「占位框 → 能播放」的一跳。
**M0 已收口**：RFC 获批（2026-09-01，路线 C「VP9/AV1 开源先行 + 进程内 crate」）。
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
  批准；三平台 CI 构建矩阵成本同 RFC §6 风险面 | ⬜ 待批（不阻塞 M3 其余面——
  WPT 子集导入可先行） |

## 下一步计划

1. **M3 多格式收尾**（当前首选）：AV1（dav1d 绑定，D-RFC-2）与 H.264 立项
   （D-RFC-3）；上游 WPT 可执行子集导入。
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
| M3 — 多格式 + 稳定 + 收尾 | ⬜（含 AV1 dav1d（D-RFC-2）与 H.264 立项（D-RFC-3）） |

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
