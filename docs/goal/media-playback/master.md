# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**M1a 落地**——`zero-media` crate 新建：webm demux +
VP9 解码 + YUV→RGBA + VideoClock trait 定义；首帧与 ffmpeg 参照逐字节一致；
依赖 matroska-demuxer 0.8 + rusty_vp9 0.1 入 workspace）

---

## 当前状态

**专项定位**：媒体方向三拆之二（门控流）。视频解码与帧渲染——「占位框 → 能播放」的一跳。
**M0 已收口**：RFC 获批（2026-09-01，路线 C「VP9/AV1 开源先行 + 进程内 crate」）。
**M1a 已落地**（2026-09-01）：`crates/media`（`zero-media`）解码管线全通——
`VideoDecoder::open_webm_vp9` → 逐帧 `next_frame()` → `DecodedVideoFrame`（RGBA +
pts_ms）；fixture 48 帧全解、PTS 单调、首帧与 ffmpeg 7.1.5 rawvideo 参照**逐字节一致**
（探针实测），`VideoClock` trait（M2a 对接点）已定义。**M1b（首帧上屏）待实施**。

**与兄弟 goal 的边界**：
- media-elements — 语义面（状态机/事件/canPlayType）归其管；本目标产出 readyState 真实
  驱动接口（`VideoClock` trait，其 headless 近似驱动届时替换，语义层不返工——RFC §3.1）
- media-audio — 音频输出/A/V 同步归其管（其 M0 已收口，AudioSink trait 验证策略成立）；
  本目标首期静音播放（video clock 驱动），音频解码面 M2c 经其 AudioSink 接入
- js-dom — 媒体反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项 + 2026-09-01 M0/M1a 更新）

### 现有实现

- ✅ **解码管线（M1a）**：`crates/media`（`zero-media` crate）——webm/Matroska
  demux（`matroska-demuxer 0.8`，纯 Rust）+ VP9 解码（`rusty_vp9 0.1`，纯 Rust 零 C
  依赖）+ YUV→RGBA（BT.601，8/10/12bit 与 4:2:0/4:2:2/4:4:4 面宽）；单测 5 + doctest 1
  常驻（fixture 帧数/PTS 单调/像素窗口/确定性/拒收非 webm）
- ✅ **播放驱动接口（M1a 定义）**：`VideoClock` trait（currentTime/duration/is_playing/
  playbackRate）——M2a player 模块实现、语义层对接点
- ✅ 架构先例：image-decoder 独立进程（D1）+ zero-protocol IPC（`ImageDecodeParams/
  Result` 字节进 RGBA 出——视频解码进程升级时同构扩展）+ compositor（C2）
- ✅ 渲染通路：canvas 像素 → 页面图元桥接（R3268）——`DecodedVideoFrame.rgba` 与
  `ImageData.pixels` 同构，M1b 帧上屏直接进 `ImageCache` + `ImagePrimitive`
- ✅ event loop 帧驱动：rAF（P1a）——播放时钟可挂
- ✅ **e2e 资产已备**（V5 闭合）：`tests/fixtures/media/` 四 fixture（h264+aac mp4 /
  vp9 webm / mp3 / opus oga，ffmpeg 生成、来源清白、生成命令入 README）
- ✅ crate 生态调研数据：symphonia 0.6（纯 Rust 容器+音频）/ dav1d 0.11（AV1 绑定）/
  openh264 0.9 / ffmpeg-next 9.0 / rav1e 0.8（crates.io 实测版本）；M1a 实测补充——
  `rusty_vp9 0.1.1`（纯 Rust VP9，Apache-2.0，MSRV 1.85）首帧与 ffmpeg 逐字节一致，
  `matroska-demuxer 0.8.1`（Zlib OR MIT OR Apache-2.0）API 干净（双许可证均兼容工作区 MIT）
- ⚠️ 帧上屏通路未接（video 元素盒 → ImagePrimitive）——M1b 实施项
- ⚠️ AV1（dav1d 绑定）与 H.264 未引入——M3（D-RFC-2 / D-RFC-3 决议）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | 解码路线选型（专利/依赖/架构三维） | ✅ RFC 获批（路线 C，2026-09-01） |
| V2 | 零解码管线（demux/解码/帧转换） | ✅ M1a 落地（2026-09-01，`zero-media` crate） |
| V3 | 播放驱动（帧率时钟/seek/ended）缺失 | ⬜ M2 |
| V4 | readyState 真值驱动接口未建 | 🔄 trait 已定义（M1a）；M2a player 实现 + 语义层对接 |
| V5 | 播放 e2e 资产为零 | ✅ fixture 已落地（2026-09-01） |
| V6 | 帧上屏通路（video 元素盒 → 图元）缺失 | ⬜ M1b（下一项） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ✅ 获批（2026-09-01）——三项决议见 RFC §5 |

## 下一步计划

1. **M1b**：首帧上屏——video 元素盒渲染通路（`DecodedVideoFrame.rgba` →
   `ImageCache`/`ImagePrimitive`，R3268 canvas 同款）+ e2e 帧上屏断言常驻。
2. **M2a**：帧率时钟 + play/pause/ended + currentTime 真值化（`VideoClock`
   实现 + media-elements 语义层 headless 驱动源替换）。
3. **M2b/M2c**：seek（关键帧粒度）+ playbackRate；音频解码（symphonia）+
   AudioSink 接入。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 完成并获批（2026-09-01，路线 C） |
| M1 — 首个视频帧上屏 | 🔄 M1a 完成（解码管线）；M1b 待实施（上屏通路） |
| M2 — 连续播放 + 语义驱动 | ⬜（VideoClock trait 已就位） |
| M3 — 多格式 + 稳定 + 收尾 | ⬜（含 AV1 dav1d（D-RFC-2）与 H.264 立项（D-RFC-3）） |

## 验证基线

- 测试基线：`make test` 全绿（66 套件；含 zero-media 5 单测 + 1 doctest）；clippy 零警告
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 播放 e2e 面：fixture 四件已入仓（`tests/fixtures/media/`）；e2e 断言形态见 RFC §3.3
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
