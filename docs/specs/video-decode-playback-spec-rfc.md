# Spec RFC：视频解码与帧播放（media-playback M0 解码器选型）

**版本**：v1.1
**日期**：2026-09-01（v1.0 草案；同日获批，见 §5 决议记录）
**状态**：Approved（2026-09-01 用户批复 D-RFC-1=批准路线 C，D-RFC-2=M3 再含，D-RFC-3=单独立项）
**关联**：[media-playback goal](../goal/media-playback.md)（M0 门控项）·
[media-elements goal](../goal/media-elements.md)（语义层消费方）·
[media-audio goal](../goal/media-audio/master.md)（AudioSink 输出面，与选型解耦）

---

## 0. 执行摘要

- **目标**：让 `<video>` 从布局占位变为能播放真实视频文件——帧解码 → 帧率驱动 →
  帧图元渲染 → currentTime 真值推进。本文档解决前置的 Mission 级选型：
  **解码器技术路线 + 进程边界**。
- **推荐**：**路线 C（限定开源编解码先行）+ 进程内 crate（复用 image-decoder 进程
  模式为可选加固）**——VP9/AV1 视频起步（纯 Rust 生态可编译、零专利费、fixture 已备），
  H.264/AAC 待专利评估后另行增量决策。
- **不做**：EME/DRM、MSE、WebCodecs、直播流（HLS/DASH）——均为后续目标，本 RFC
  只锁定解码与播放骨架。
- **验证资产**：`tests/fixtures/media/` 四个 ffmpeg 生成 fixture（h264+aac mp4 /
  vp9 webm / mp3 / opus oga）已入仓，来源清白（见该目录 README）。

## 1. 背景与现状

### 1.1 现状（2026-09-01 实测）

- **语义层已闭环**：media-elements goal 把 HTMLMediaElement 的非解码语义面推进到
  WPT 90.0%（状态机/事件序列/play-pause Promise/元数据 IDL/track 反射），
  全部以 headless 近似驱动（`_mediaState` + setTimeout 模拟）。**语义层代码
  不因本 RFC 选型返工**——驱动源替换是接口对接，不是重写。
- **解码层为零**：全工作区无任何视频解码依赖（Cargo.toml 无 symphonia/ffmpeg/
  dav1d/openh264 等）；`<video>` 渲染为布局占位框。
- **可复用先例**：
  - `apps/image-decoder`（D1）——独立解码进程 + `zero-protocol` IPC
    （`ImageDecodeRequest/Result`，字节进 RGBA 出）——多进程解码边界已在产；
  - canvas 像素 → 页面图元桥接（R3268）——「解码位图 → 渲染管线」通路已验证；
  - rAF 帧驱动（P1a）——播放时钟可挂的 event loop 底座。

### 1.2 验证标准

- **本地 e2e**：真实视频文件（`tests/fixtures/media/`）→ 帧上屏 + currentTime 推进
  + play/pause/ended 事件——常驻 CI（NullSink 式可观测断言，不需真 GPU 出图）。
- **上游子集**：WPT the-video-element 语义子集随解码真值化逐批解锁
  （video_size_preserved_after_ended 等依赖 videoWidth/videoHeight 真值的用例）。

## 2. 三路线对比（切片 2 调研产出）

| 维度 | 路线 A：ffmpeg 外部进程 | 路线 B：Rust crate 组合（进程内） | 路线 C：限定开源编解码先行 |
|---|---|---|---|
| **格式覆盖** | 最广（H.264/HEVC/VP9/AV1 全支持） | 取决于 crate 组合（见下） | VP9/AV1（+Vorbis/Opus 音频）；H.264 缺位 |
| **专利风险** | ⚠️ H.264/HEVC 编解码分发涉 MPEG-LA 专利池（ffmpeg 二进制分发即触发） | 同左（若含 H.264） | ✅ VP9/AV1 免专利费；无 AVC 专利暴露 |
| **依赖体量** | ❌ 重：ffmpeg C 库 ~30MB+，全平台构建矩阵（Win/macOS/Linux 三平台 CI 均需预编译分发）；`ffmpeg-next 9.0` 绑 FFmpeg 4 API | 中：纯 Rust 可 cargo 直依赖；涉及绑定的（dav1d/openh264）需 C 工具链 | ✅ 轻：`symphonia 0.6`（纯 Rust 容器+音频解码）+ `dav1d 0.11` 绑定（AV1）或 `rav1e`（编码不适用于解码）；VP9 解码器 `libvpx` 绑定或纯 Rust（`webm`/`vp9` crate 成熟度有限） |
| **工程量** | 大：进程边界 + 协议扩展 + 三平台分发 + 版本升级跟踪 | 中：进程内集成 + 错误边界；绑定类仍需 C 工具链 | 小→中：与 B 同构但面更窄（先 VP9/AV1 两条编解码） |
| **架构一致性** | ✅ image-decoder 同款进程边界（漏洞隔离最彻底） | ⚠️ 解码代码进 renderer 进程（崩溃域扩大）；可用独立进程包裹恢复隔离 | 同 B |
| **回滚性** | 差：分发面变更回滚成本高 | 好：feature gate + 依赖删除即回滚 | ✅ 最好：单格式增量、逐格式开闸 |
| **e2e 可验证性（本环境）** | ❌ 本环境无预编译 ffmpeg 分发链 | 部分可验证（纯 Rust 部分立即可编译） | ✅ fixture 中 `sample-webm-vp9.webm` 即测即用 |

**交叉结论**：A 的分发与专利成本在「首个视频帧上屏」阶段（M1）不成比例；B 若直接
含 H.264 绑定（openh264）会提前触发专利评估；**C 以 VP9/AV1 起步，覆盖 fixture 与
WPT 上游主流测试素材格式（webm 系），专利零暴露、依赖最轻、回滚最干净**——
H.264/HEVC 作为路线 C 后续增量（届时按 §5 决策点单独立项），不进本期。

## 3. 推荐方案：路线 C + 进程内 crate（细节设计）

### 3.1 分层架构（解码与播放解耦）

```
┌─ engine（现有，不改语义层）────────────────────────┐
│ HTMLMediaElement 语义面（_mediaState/事件/IDL）      │
│   ▲ 驱动源接口：VideoClock trait（M2 对接点）        │
└──────┼──────────────────────────────────────────┘
       │ readyState/duration/currentTime 真值化
┌──────┴──────────────┐   ┌──────────────────────┐
│ player（新模块）      │   │ decode（新模块）       │
│ 帧率时钟/play/seek/  │──▶│ demux(webm) →        │
│ ended/playbackRate   │   │ vpx/av1 decode →     │
└─────────────────────┘   │ YUV→RGBA 转换         │
                          └──────────────────────┘
                                   │ RGBA 帧位图
                          ┌────────▼─────────────┐
                          │ 渲染通路（R3268 同款） │
                          │ video 元素盒 → 图元    │
                          └──────────────────────┘
```

- **decode 模块**：demux（webm/Matroska 容器解析）+ VP9/AV1 解码 + YUV→RGBA。
  纯 Rust 优先；AV1 用 `dav1d` 绑定（C 但单库、构建简单、mesa 项目维护）作为
  可选 feature（`decode-av1`），VP9 起步不依赖任何 C。
- **player 模块**：帧率时钟挂 rAF event loop（P1a 底座）；play/pause/seek（关键帧
  粒度起步）/ended/playbackRate；对上暴露 `VideoClock` 接口喂 media-elements 语义层
  （readyState/duration/currentTime 真值化——**驱动源替换点**）。
- **进程边界**：M1 先进程内（最小可工作）；若解码器漏洞隔离需求上升，按
  image-decoder 模式追加独立进程（`VideoDecodeRequest/Result` 协议消息形似
  `ImageDecodeParams`，player 模块与进程边界解耦，升级不返工）。

### 3.2 音频输出（与 media-audio 对接）

- 视频 fixture 含 AAC/Opus 音轨：首期**静音播放**（media-playback goal 关键约束——
  currentTime 由视频时钟驱动）；音频解码面（symphonia）在 M1 后期接入，
  输出经 media-audio 的 `AudioSink` trait（NullSink 可观测断言进 CI）。
- A/V 同步（audio clock 主时钟）归 media-audio M2，本 RFC 只预留时钟接口。

### 3.3 测试资产化（CLAUDE.md 规则）

- M1 验收 e2e：`sample-webm-vp9.webm` → 首帧 RGBA 哈希断言 + currentTime 推进断言，
  入 `tests/integration` 常驻。
- 逐修复附带 WPT 用例导入（media-elements 流同款 `make import-wpt` 资产化路径）。

## 4. 分阶段里程碑（RFC 批准后）

| 里程碑 | 内容 | 验证 |
|---|---|---|
| M1a | webm 容器 demux + VP9 解码 + YUV→RGBA | 解码单测（fixture 帧哈希） |
| M1b | 首帧上屏（video 元素盒渲染通路） | e2e 帧上屏断言常驻 |
| M2a | 帧率时钟 + play/pause/ended + currentTime 真值化 | VideoClock 接口对接 media-elements（headless 驱动源替换） |
| M2b | seek（关键帧粒度）+ playbackRate | e2e 断言扩展 |
| M2c | 音频解码（symphonia）+ AudioSink 接入（静音→有声） | NullSink 可观测断言 |
| M3 | AV1（decode-av1 feature）+ H.264 专利决策增量 | 单独立项 |

## 5. 待用户决策点

| # | 决策 | 选项 | 推荐 |
|---|---|---|---|
| D-RFC-1 | **本 RFC 是否批准**（解锁 M1a 起源码实施） | 批准路线 C / 改选 A / 改选 B / 暂缓 | 批准路线 C |
| D-RFC-2 | AV1 的 `dav1d` C 绑定是否接受（M1 可不含 AV1——纯 Rust VP9 先行） | M1 含 AV1 / M3 再含 | M3 再含（M1 零 C 依赖） |
| D-RFC-3 | H.264/HEVC 后续增量（专利池评估）是否单独立项 | 单独立项 / 放弃 | 单独立项（media-playback M3 时点） |

### 决议记录（2026-09-01）

| # | 决议 | 备注 |
|---|---|---|
| D-RFC-1 | ✅ **批准路线 C** | 解锁 M1a 起源码实施 |
| D-RFC-2 | ✅ **M3 再含 AV1** | M1 保持零 C 依赖；`decode-av1` feature 默认关 |
| D-RFC-3 | ✅ **单独立项** | media-playback M3 时点按本节决策点评估 H.264/HEVC |

## 6. 风险与回滚

| 风险 | 缓解 |
|---|---|
| 纯 Rust VP9 解码器成熟度不足 | 退化路径：`libvpx` C 绑定（构建复杂度 +1 但单库）；再退化：路线 A |
| dav1d 绑定的 C 工具链在三平台 CI 的构建成本 | feature-gate（`decode-av1` 默认关）；VP9 主线不受影响 |
| 进程内解码崩溃域扩大 | player/decode 模块边界清晰 → 升级独立进程（image-decoder 模式）零接口返工 |
| 语义层与真解码对接时序偏差 | VideoClock 接口在 M1a 期先行定义（纯 trait 文件，无实现），media-elements 侧 mock 对接测试先行 |
