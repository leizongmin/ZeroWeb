# Spec RFC：H.264/AAC 增量立项（media-playback M3，D-RFC-3）

**版本**：v1.0
**日期**：2026-09-02
**状态**：Approved（2026-09-05 用户批复 D-RFC-3a=有条件批准实施【分发前须法务复核】、
3b=构建期源码编译、3c=AAC 随期——见 §5 决议记录。原 Proposed 态注记：media-playback
M3 门控项，D-RFC-3 决议「单独立项」的本立项文档）
**关联**：[media-playback goal](../goal/archive/media-playback.md)（M3 多格式收尾；goal 已完成，2026-09-05 归档）·
[video-decode-playback-spec-rfc.md](video-decode-playback-spec-rfc.md)（路线 C 主 RFC，
D-RFC-1/2 已批：VP9 纯 Rust + AV1 dav1d 已落地）·
[web-audio-audiocontext-minimal-face-spec-rfc.md](web-audio-audiocontext-minimal-face-spec-rfc.md)
（Mission 级 RFC 先例）

---

## 0. 执行摘要

- **一句话目标**：评估 H.264（视频）+ AAC（音频）解码增量在现有 zero-media 解码
  管线上的可行性与工程量，形成是否实施的决策依据（D-RFC-3 决议「单独立项」的
  立项评估文档）。
- **结论**：**可行，工程量与 AV1 切片同量级**。解码源选 **Cisco OpenH264**
  （`openh264 0.9` 安全 Rust 绑定 + 构建期源码编译，零外部下载）；AAC 复用
  **symphonia 既有 `aac` feature**（纯 Rust，仅扩 feature 清单）；mp4 demux 用
  **symphonia `format-isomp4`**（纯 Rust）。预估 **2~3 个切片**（≈500~700 行
  Rust + fixture 验证 + WPT/产品面），与 AV1 切片（699205910，479 行）同构。
- **专利面（核心风险项，诚实评估）**：
  - H.264 解码**分发**涉 MPEG-LA AVC 专利池——但 **OpenH264 本体是 Cisco 以
    AVC Patent Trust License 2.0 免版税分发**的开源实现（二进制含解码能力且由
    Cisco 承担专利授权）。**ZeroWeb 分发 OpenH264 二进制/源码编译产物的专利
    授权链 = Cisco → 终端用户**，ZeroWeb 自身按 Cisco 的 FAQ 可免版税再分发
    （附 license/NOTICE）；若改为**自行从源码编译**（openh264-sys2 `source`
    feature 默认行为），专利授权链依赖 Cisco 开源声明，**法务确定性弱于分发
    Cisco 官方预编译二进制**——两条子路线的取舍见 §3 决策点。
  - AAC 解码（symphonia bundled aac）涉 Via Licensing AAC 专利池——与 H.264
    同一决策面（解码器实现方持有授权与否）；symphonia 为纯开源实现（MIT），
    **授权链同样不经实现库转授**，与 H.264 面同风险等级。
  - **本 RFC 不构成法律意见**；专利面结论以「实现方授权链 + 分发形态」两维
    呈现，是否接受该风险面为用户决策（§5 D-RFC-3a）。
- **不做**（§6.2）：H.264 **编码**（openh264 encoder 面不用）、HEVC/AVC 4:2:2
  以上 profile、HLS/DASH 流媒体、DRM、硬件解码（VideoToolbox/MediaCodec）。
- **推荐**：**批准实施**（产品兼容性主缺口——H.264 mp4 是 web 视频存量最大面；
  工程量小、架构面已有 decode-av1 先例），或**维持不实施**（WPT/产品面当前无
  强需求驱动；不批准不影响 media-playback goal 的 M3 其余面）。

---

## 1. 背景与现状

### 1.1 规范面与产品面

| 面 | 内容 | 现状 |
|-----|------|------|
| 产品面 | web 视频存量以 H.264 mp4 为主（Chromium 內建） | `<video src=*.mp4>` 当前不可解码 → 占位渲染 |
| WPT 面 | `html/semantics/embedded-content/media-elements` 多数 ready-states/事件时序用例以 h264 mp4 为素材 | headless 近似面已覆盖（media-elements 95.3%）；真解码面归本立项 |
| fixture | `tests/fixtures/media/sample-mp4-h264.mp4`（M0 切片 3 已备，h264 baseline + AAC LC） | ✅ 在库，生成命令可重现 |

### 1.2 ZeroWeb 现状（2026-09-02 实测）

- **解码管线**（`crates/media`，M1a~M3 已落地）：
  - `VideoDecoder::open_webm_vp9`（rusty_vp9 纯 Rust）+ `open_webm` codec 自路由
    （V_VP9/V_AV1——dav1d feature-gated，699205910）；
  - 通用 `planes_to_rgba` 转换面（M2 色度精化——BT.709/601/identity + limited/full）；
  - `WebmAudioTrack`（vorbis/opus）+ `AudioDecoder`（symphonia mp3/vorbis）+
    `opus-decoder`（opus 纯 Rust）——音频输出面 `AudioSink` 双实现齐备。
- **缺**：mp4 容器 demux、H.264 位流解码、AAC 解码（symphonia feature 未启用）。
- **架构先例**：AV1 切片的 codec 路由模式（`VideoCodec` enum + `open_webm` 自路由
  + feature gate + fixture 全流验证）——H.264 增量走同款骨架，**接口层零返工**。

---

## 2. 方案对比

| 维度 | A. OpenH264（Cisco）+ symphonia aac | B. FFmpeg（libavcodec 全家桶） | C. 纯 Rust（rusty_h264 等） | D. 不实施 |
|------|------|------|------|------|
| 解码能力 | H.264 baseline/main/high（Cisco 实现覆盖主流 profile）；AAC 由 symphonia | H.264/HEVC/全格式最广 | rusty_h264 0.12（baseline+部分 high，声称 bit-exact openh264）成熟度**未经验证** | — |
| 依赖体量 | `openh264 0.9`（构建期源码编译，cc 驱动，零外部下载）+ symphonia 两个 feature 扩展 | ffmpeg C 库 ~30MB + 三平台预编译分发（主 RFC §4 路线 A 已否决的形态） | 纯 Rust 零 C | 0 |
| 专利/授权链 | **Cisco AVC Patent Trust License 2.0**（实现方授权；源码编译态的确定性弱于官方二进制，见 §0） | LGPL + MPEG-LA 双重暴露 | 纯 Rust 实现**不改变专利池义务**（专利面随编解码技术本身，不随实现）——同 A | 无新增 |
| 工程量 | 2~3 切片（mp4 demux + h264 解码 + aac/播放接线 + fixture/WPT 面） | 大（分发矩阵 + 进程边界——与已否决路线 A 同构） | 中（成熟度风险高——衰退路径仍是 A） | 0 |
| 风险 | 低-中（专利面待拍板；源码编译确定性注记） | 高（重复主 RFC 已否决面） | 高（生态未验证） | 产品 H.264 面持续空缺 |

**决策点**（§5）：A 为主推荐；C 作为 A 的**编译环境兜底路径**记录（不单独立项）。

---

## 3. 最小面技术设计（路线 A）

### 3.1 架构

```
JS（既有，零改动）                Rust（crates/media 增量）
┌──────────────────┐            ┌────────────────────────────────┐
│ <video src=*.mp4> │ ── settle ─→ VideoDecoder::open_webm/open_mp4 │
│  语义层/桥/泵     │             │   codec: Vp9 / Av1 / H264 ← 新  │
│ （media-elements  │             │   （VideoCodec enum 增第 3 变体）│
│  95.3% 面不返工） │             └────────────┬───────────────────┘
└──────────────────┘                          │ mp4: symphonia isomp4
        A/AAC 音频 → AudioDecoder（symphonia aac feature）→ AudioSink
        H.264 位流 → openh264 Decoder → YUV 平面 → planes_to_rgba（共用）
```

- **mp4 demux**：symphonia `format-isomp4`（probe/mp4 面）——或复用 `mp4 0.14`
  纯 Rust 容器读（sample 面评估后取一，实施切片定案）；产出 H.264 Annex-B
  位流 + AAC raw 帧 + 时序。
- **H.264 解码**：`openh264::decoder::Decoder`（**已实证**：本机探针
  `sample-mp4-h264.mp4` 提取 Annex-B → 48/48 帧全解、320x240、luma 均值 122.14
  对 ffmpeg 参照 123.3 同窗）——`VideoCodec::H264` 第 3 变体，接口与
  Av1Decoder 同形（push/next_frame/flush）。
- **AAC**：symphonia workspace 依赖扩 feature（`aac` + `format-isomp4`）——
  `AudioDecoder` 增 mp4/aac 容器支持；播放接线复用 `AudioEntry`/`WebmAudioEntry`
  同款（`AudioStreamDecoder` enum 增变体或直接命中 Symphonia 路径）。
- **codec 自路由**：`open_webm` 泛化为 `open_media`（按容器嗅探 webm/mp4）或
  平行 `open_mp4`——实施切片按调用方（webview settle 面）最小改动原则定案。
- **feature gate**：`decode-h264`（默认关，同 `decode-av1`）——openh264 源码
  编译入 build.rs；默认构建矩阵零新增 C 工具链面（CI 主线不受影响）。

### 3.2 时序与语义锚点（headless 简化记录）

- 首帧验证：fixture 48 帧全解 + PTS 单调 + 首帧 luma/RGB 均值与 ffmpeg 参照
  同窗（AV1 切片同款 ±15 窗——探针实测 122.14 vs 123.26，天然达标）。
- H.264 时序：mp4 stts/ctts 时序 → pts_ms（与 matroska timestamp 同款归一）；
  B 帧重排由 openh264 输出面承担（display 顺序）。
- 音频面：AAC LC 44.1kHz mono fixture → symphonia aac → NullSink 过零率 ≈880
  锚点（M1 契约同款）。

### 3.3 切片划分

1. **切片 1（zero-media）**：mp4 demux + `VideoCodec::H264`（openh264）+
   fixture 全流单测（`decode-h264` feature）——与 AV1 切片同构验证面。
2. **切片 2（音频 + 接线）**：symphonia aac/isomp4 feature 扩展 + AAC fixture
   e2e（NullSink 锚点）+ webview registry `open_mp4` 路由 + settle 面。
3. **切片 3（可选）**：WPT 真解码面用例评估导入（ready-states 真值面——依赖
   settle 时序真实化）+ 产品 smoke 观察面。

---

## 4. 需求（FR 摘选——完整 BDD 面随批准后实施细化）

### FR-001：mp4/H.264 首帧解码
- 描述：当 `<video src=*.mp4>`（H.264 baseline yuv420p）settle 时，系统必须
  产出与 ffmpeg 参照同窗的首帧 RGBA。
- 验收：fixture 全流 48 帧 / PTS 单调 / 首帧均值窗 ±15（探针已预验证）。

### FR-002：AAC 音频链
- 描述：当 mp4 含 AAC LC 音轨时，系统必须经 symphonia 解码写入 AudioSink
  （NullSink 过零率锚点）。

### FR-003：feature 隔离
- 描述：`decode-h264` 默认关时，全部既有构建/测试零变化（`decode-av1` 同款
  门控纪律）。

### FR-004：路由与占位
- 描述：feature 关闭时 mp4 源回落占位渲染（不可解码 src 零回归契约——AV1
  NoVideoTrack 回落同款语义）。

## 5. 待用户决策点

| # | 决策 | 选项 | 推荐 |
|---|---|---|---|
| D-RFC-3a | **H.264/AAC 专利授权链是否接受**（核心门禁——本 RFC 不构成法律意见） | 接受（批准实施）/ 拒绝（维持不实施）/ 暂缓待法务评估 | 接受并批准实施（Cisco 授权链 + 开源实现；源码编译态确定性注记已诚实记录） |
| D-RFC-3b | OpenH264 分发形态 | ① 构建期源码编译（openh264-sys2 `source` 默认；build 面简单但授权确定性弱）② 分发 Cisco 官方预编译二进制（授权链最强但三平台分发矩阵成本回潮——主 RFC 已否决的形态） | ①（与路线 C 轻依赖原则一致；确定性差距以 NOTICE/版本锁定缓解） |
| D-RFC-3c | AAC 是否随本期一并实施 | 随本期 / 分离立项（音频面独立） | 随本期（symphonia feature 扩展成本 ≈0，分离无收益） |

### 决议记录

**✅ 2026-09-05 用户批复（GB-20260904 待决策征询跟进，session 对话）**：

| # | 决议 | 注记 |
|---|---|---|
| D-RFC-3a | ✅ **有条件批准实施** | 接受 Cisco 授权链（源码编译态确定性弱，注记在案）+ Via AAC 池风险面；**前置条件：任何二进制分发/发布前须完成法务复核**。实施与分发解耦，代码工作可回退 |
| D-RFC-3b | ✅ **① 构建期源码编译**（openh264-sys2 `source` 默认） | 与路线 C 轻依赖原则一致；确定性差距以 NOTICE/版本锁定缓解 |
| D-RFC-3c | ✅ **AAC 随本期** | symphonia feature 扩展成本 ≈0，分离无收益 |

- RFC 状态：Proposed → **Approved**（批准前不动源码的冻结解除，解锁切片 1 起实施）。
- 征询凭据：msg `om_x100b664d8a6f44b0dee3398474de92b`（2026-09-02）+ msg
  `om_x100b669923cd64a4c3e335615ed3d9f`（GB-20260904 巡检合并跟进）。

---

## 6. 约束与边界

### 6.1 必须（Must）
- `decode-h264` feature gate（默认关）；主线构建零新增 C 依赖面。
- 既有 VP9/AV1 面、media-elements 语义面（95.3%）、播放管线零回归。

### 6.2 禁止（Must Not）
- H.264 **编码**面（openh264 encoder 不引入）。
- HEVC / 4:2:2+ profile / 10bit（fixture 与产品面均非本期）。
- HLS/DASH/DRM/硬件解码（goal 排除清单一致）。

### 6.3 已定决策
- 路线 C 主架构（进程内 crate + feature gate）——D-RFC-1/2 已批。
- 色彩转换复用 `planes_to_rgba` 单点（M2 色度面）。

### 6.4 技术约束
- openh264-sys2 源码编译需 C 编译器（cc 驱动——与 dav1d 同为系统前提；
  本机已验证可编译，CI 矩阵成本在 feature 关闭态为零）。
- symphonia 0.6 的 AAC 解码面为 `symphonia-codec-aac`（纯 Rust bundled）。

### 6.5 假设
- Cisco OpenH264 的再分发授权链覆盖源码编译产物 — 状态：**待验证**
  （本 RFC 以 §5 D-RFC-3a 呈现风险面，不作为已闭合事实）。
- `mp4 0.14` 与 symphonia isomp4 二选一的 demux 成本相当 — 状态：待验证
  （切片 1 定案，两者均为纯 Rust）。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| H.264 位流解码 | 新增依赖（feature-gated） | `openh264 0.9`（openh264-sys2 源码编译，Cisco OpenH264） | 本机探针已验证 48/48 帧解码 |
| AAC 解码 | 既有依赖扩 feature | `symphonia 0.6`（`aac` feature，纯 Rust） | workspace 依赖已在 |
| mp4 容器 demux | 新增依赖（二选一） | `mp4 0.14` 或 symphonia `format-isomp4` | 均纯 Rust，切片 1 定案 |
| YUV→RGBA | 复用现有模块 | `crates/media/src/decode.rs::planes_to_rgba` | M2 色度面单点 |
| 播放接线 | 复用现有模块 | `VideoCodec` enum / `AudioEntry` / settle 面 | AV1 切片同构 |

### 6.6 代码变更边界（批准后实施期）
- **允许修改**：`crates/media/**`、`crates/webview/src/video_registry.rs`（mp4
  settle 登记/路由）、`Cargo.toml`（workspace 依赖 + feature）、
  `tests/fixtures/media/**`（如需补 fixture）、`docs/goal/media-playback/**`
  （注：goal 已完成归档，现位于 `docs/goal/archive/media-playback/**`）。
- **禁止修改**：`crates/engine/src/js_dom_shim/**`（语义面零改动——95.3% 面
  不返工是本立项的架构前提）、渲染域 crate。

---

## 7. 里程碑建议

| 里程碑 | 范围 | 预估 |
|---|---|---|
| M-H1 | 切片 1（mp4 demux + H264 解码 + fixture 验证） | 1 轮 |
| M-H2 | 切片 2（AAC + 播放接线 + settle 面） | 1 轮 |
| M-H3 | 切片 3（WPT/产品面，可选） | 0.5 轮 |

---

*本 RFC 为 D-RFC-3「单独立项」决议的立项评估文档；**批准与否均不影响
media-playback goal 的 M3 其余面**（AV1 已落地、WPT 子集导入独立推进）。
批复请直接回复 D-RFC-3a/3b/3c 三项决策。*
