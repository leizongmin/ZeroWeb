# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**M2c 音频解码面落地**——symphonia 0.6 入 workspace
（mp3+ogg/vorbis feature 面；**opus 不在其编解码面**——纯 Rust 约束，oga-opus
fixture 留后续选型）；`AudioDecoder` f32 交错 PCM 逐包输出；新 fixture
`sample-ogg-vorbis.oga`（440Hz sine，生成命令入 README）；全链 e2e 双件常驻：
mp3 + vorbis → NullSink 过零率 ≈880 锚点（media-audio M1 契约）。media 30 全绿）

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
  `matroska-demuxer 0.8.1`（Zlib OR MIT OR Apache-2.0）API 干净（双许可证均兼容工作区 MIT）
- ✅ **生产侧帧注入 + 播放桥（M2a 切片 4/5）**：settle 首帧注入 ImageCache；
  `VideoPlayerRegistry` + `__zwVideoBridge` 宿主桥 + tab_worker 帧泵（切片 5a/5b）
- ⚠️ renderer 多进程路径未接桥注入（tab_worker 路径已通；renderer js_worker 镜像
  待做——生产双路径一致性）
- ⚠️ 色度元数据精化：WebM Colour 元素（range/matrix）未读，固定 BT.601 full-range——
  replaced-element-003 unmask 案揭示（M2 解码层精化项）
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

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ✅ 获批（2026-09-01）——三项决议见 RFC §5 |

## 下一步计划

1. **M2c 后续（下一项）**：播放管线接 AudioSink——`<video>`/`<audio>` play 时把
   音频轨解码帧（webm 含音频轨面 symphonia 直解 / 独立音频资源）喂 Mixer → sink，
   muted/volume 增益联动（media-elements IDL 语义 ↔ mixer 增益）；A/V 同步前置
   （audio clock 主时钟——media-audio M2 契约）。
2. **renderer 多进程路径对齐**：桥接线当前在 tab_worker 路径（test-support feature）；
   renderer js_worker 同款 SetVideoPlayers 注入（镜像）——生产双路径一致性。
3. **M2 解码精化项**（M1b 揭示）：WebM Colour 元素解析（colourRange/matrix）→
   limited-range 与 BT.709 自适应转换（replaced-element-003 unmask 面收口）。
4. **opus 解码选型注记**：symphonia 0.6 无 opus（纯 Rust 面缺位）——后续评估
   （对称面： webinar/opusic 等 pure-Rust crate 成熟度，或维持 mp3+vorbis 面）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 完成并获批（2026-09-01，路线 C） |
| M1 — 首个视频帧上屏 | ✅ 完成（2026-09-01：M1a 解码管线 + M1b 帧上屏通路 + e2e 常驻） |
| M2 — 连续播放 + 语义驱动 | 🔄 M2a + M2b 收口（2026-09-01：播放/真值/桥/帧泵 + 精确 seek + 变速桥接）；M2c 音频待续 |
| M3 — 多格式 + 稳定 + 收尾 | ⬜（含 AV1 dav1d（D-RFC-2）与 H.264 立项（D-RFC-3）） |

## 验证基线

- 测试基线：`make test` 全绿（zero-media default 23 单测 + 1 doctest；engine 2539
  含桥契约测试；webview 667 含桥 e2e + registry 4 + settle e2e 2；browser 411 under
  xvfb）；clippy 零警告；testharness-media 372P/0F/41PF 基线维持（桥 feature-detect
  回落面零回归实证）
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 上屏 e2e 锚点：帧区 RGB 均值 138-168（testsrc2 ≈153.5）+ 帧界外白底 + 不可解码
  src 占位负例；reftest-upstream 13950/16730（83.4%，唯一净 delta = replaced-element-003
  false-pass unmask，见 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
