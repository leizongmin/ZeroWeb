# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**M2a 切片 3 落地**——videoWidth/videoHeight IDL 真值面：
get trap VIDEO-gated 读 `_resourceStates.width/height`（切片 2 探针真值）+ has 白名单
tag-gated（audio 无此接口成员）；未 settle 恒 0（spec 元数据未就绪）。单测 5 断言组
常驻；testharness-media 372P/0F/0T/41PF 维持零回归）

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
常驻；testharness-media 372P/0F/0T/41PF 维持（零回归）。**下一切片：生产侧帧注入
（ImageCache 接线）+ currentTime 真值推进**。

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
- ⚠️ 生产侧渲染线程未接解码注入（harness 通路已通，webview/async_load 的媒体字节
  → 解码 → ImageCache 为 M2a player 实施项）
- ⚠️ 色度元数据精化：WebM Colour 元素（range/matrix）未读，固定 BT.601 full-range——
  replaced-element-003 unmask 案揭示（M2 解码层精化项）
- ⚠️ AV1（dav1d 绑定）与 H.264 未引入——M3（D-RFC-2 / D-RFC-3 决议）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | 解码路线选型（专利/依赖/架构三维） | ✅ RFC 获批（路线 C，2026-09-01） |
| V2 | 零解码管线（demux/解码/帧转换） | ✅ M1a 落地（2026-09-01，`zero-media` crate） |
| V3 | 播放驱动（帧率时钟/seek/ended）缺失 | 🔄 M2a 落地（时钟/play/pause/ended）；seek 归 M2b |
| V4 | readyState 真值驱动接口未建 | 🔄 `VideoPlayer` 实现 + duration 真值链已通；currentTime 推进接线待切片 4 |
| V5 | 播放 e2e 资产为零 | ✅ fixture 已落地 + M1b 帧上屏 e2e 常驻 |
| V6 | 帧上屏通路（video 元素盒 → 图元）缺失 | ✅ M1b 落地（harness 侧全链；生产侧注入归 M2a） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ✅ 获批（2026-09-01）——三项决议见 RFC §5 |

## 下一步计划

1. **M2a 切片 4（下一项）**：生产侧帧注入——webview/async_load 媒体字节 → 解码 →
   生产渲染线程 ImageCache（harness 通路 M1b 已通；生产侧接线使 `<video>` 实际出图）。
2. **M2a 切片 5**：currentTime 真值推进——`VideoPlayer` 挂 rAF event loop，语义层
   `_mediaState.currentTime` 从 player 读真值（RFC §3.1 驱动源替换完成面）。
3. **M2b**：seek（关键帧粒度）+ playbackRate 变速语义；**M2c**：音频解码（symphonia）+
   AudioSink/Mixer 接入（media-audio 输出面三切片已备）。
4. **M2 解码精化项**（M1b 揭示）：WebM Colour 元素解析（colourRange/matrix）→
   limited-range 与 BT.709 自适应转换（replaced-element-003 unmask 面收口）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 完成并获批（2026-09-01，路线 C） |
| M1 — 首个视频帧上屏 | ✅ 完成（2026-09-01：M1a 解码管线 + M1b 帧上屏通路 + e2e 常驻） |
| M2 — 连续播放 + 语义驱动 | 🔄 M2a 播放驱动落地（2026-09-01，VideoPlayer + 单测 6）；语义层替换 + 生产注入 + seek/音频待续 |
| M3 — 多格式 + 稳定 + 收尾 | ⬜（含 AV1 dav1d（D-RFC-2）与 H.264 立项（D-RFC-3）） |

## 验证基线

- 测试基线：`make test` 全绿（zero-media default 23 单测 + 1 doctest：decode 5 +
  NullSink 5 + mixer 7 + player 6；`audio-cpal` feature 另增 CpalSink 冒烟 1 件）+
  engine `video_frame_display_tests` 4 件 + wpt-runner M1b e2e 2 件；clippy 零警告
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 上屏 e2e 锚点：帧区 RGB 均值 138-168（testsrc2 ≈153.5）+ 帧界外白底 + 不可解码
  src 占位负例；reftest-upstream 13950/16730（83.4%，唯一净 delta = replaced-element-003
  false-pass unmask，见 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
