**最后更新**: 2026-09-05（**H.264 切片 2 落地——AAC 音频链 + mp4 伴生轨 + precise-seek**：
① **AAC 解码链 e2e**：mp4 容器 AAC LC 轨 → symphonia aac 解码 f32 → NullSink 过零率
≈880 契约锚点（mp3/vorbis 链同款断言面；aac+isomp4 feature 切片 1 已入 workspace）；
② **registry 伴生 AAC 轨**：`WebmAudioTrackKind::Mp4Aac(Box<AudioDecoder>)` 第三形态
——play 懒建（webm 双形态失败后试 mp4 probe）+ seek 重建同面；mp4 settle→play→伴生
泵推进 e2e（feature-gated `media_settle_mp4_h264_play_companion_aac_advances`）；
③ **mp4 precise-seek**：`Mp4H264Decoder::seek_to_ms` 前向回退形态（源字节自持 +
reader/decoder 重建 + 前向解码至 ≥ target 写 pending——webm ② 回退同构；
seek(1000)→首帧 pts ∈ [1000,1042] + 后续 24 帧推到流末，单测常驻）；
stss/sync-sample 索引加速随切片 3 评估。media 49/52 双配置 + webview 691/692 双配置
全绿、clippy/fmt 干净。）
**（注：下方「最后更新」切片 1/D3/D4 块为前轮记录，保留作历史）**
**最后更新**: 2026-09-05（**D3 切片 1 + D4 双落地**——获批当日实施：
① **H.264 切片 1（zero-media）**：新增 `mp4_h264` 模块——symphonia `isomp4` demux
（H264 轨枚举/timescale/avcC extradata/轨时长）+ `openh264` 位流解码（avcC→SPS/PPS
Annex-B 前缀注入 + 长度前缀 NALU→Annex-B 转换）；`VideoTrackDecoder::open_media`
容器嗅探路由（Matroska 魔数/ftyp）——player/registry/settle 探针/probe_dimensions
消费面统一入口；`decode-h264` feature 门控（openh264 0.9 source 源码编译——D-RFC-3b
决议；无 nasm 自动退纯 C 路径）。fixture 全流单测：48 帧全解、PTS 单调 0~1958ms、
320x240、首帧 RGB 均值 122（ffmpeg 参照 123.3 同窗 ±15）、容器时长 2000ms 真值；
mp4 settle e2e（feature-gated）+ 默认面占位负例保持。canPlayType 能力表扩
video/mp4 + audio/mp4（M4g-d 纪律）。media 48/50 双配置 + webview 691 双配置全绿；
testharness-media 629P/0F/24PF 保零回归；bench-gate 定向跑 webview_load_html_simple
单指标超阈值——隔离复测 78.9µs 回预算内（ZRG 噪声签名三层判据，ZRG-2026-09-03
learning 同法），不动基线。**D-RFC-3a 前置条件注记：任何二进制分发/发布前须完成
法务复核（实施与分发解耦）。**
② **D4 renderer 播放泵（事件循环节拍——否决独立泵线程的决议落地）**：runtime.run
主循环 16ms 节拍上挂 is_any_playing 门控泵（tick_all + audio_advance_all +
webaudio advance；帧更新 → try_republish_cached 上屏）；pump_epoch/pump_clock 入
RendererRuntime，SetVideoPlayers 扩 pump_clock → register_video_bridge_callbacks
clock 注入（桥 play 锚与泵 tick 同源——扩批 XXV 原点错位缺陷的 renderer 路径消除；
「登记但不推进」深结构缺口收口）。renderer 153 lib 测试全绿、clippy 零警告。）
**（注：D3/D4 批复块头链已归档至
[archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md)
（2026-09-05 治理切片）；本控制面保留最新两块——切片 2 落地与切片 1+D4 双落地。）**

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
（**M1a/M1b/M2a 切片 2~5b/M2b/M2c/M2 切片 C/D/E 历史过程段**——解码管线/帧上屏/
  播放驱动/真值注入/registry/桥接线/色彩面/音频面/A-V 同步各片明细——已归档至
  [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md)；
  实测基线节保留各片交付面的现状清单。）

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

**DC-4（多格式 + 稳定性）✅（2026-09-05 收口）**：① 选型面内容器/编解码 e2e
（VP9 单轨/双轨 + AV1 全链 + **H.264 mp4：解码/settle/播放/伴生 AAC/seek 全链
（切片 1+2）**✅）；② 资源生命周期（`prepare_document_state` 清空注册表 +
`clear()` 单测——导航释放面 ✅）；③ 上游 WPT 可执行子集：**评估收束**——runner
桥注入面（fixture-mounted）已落地并覆盖 webm 播放用例；WPT pinned rev（3159769338）
media 素材全为 webm/mp3/oga，**无 mp4 用例** → 真解码面无可导入增量（诚实注记）；
产品 fixture 无 video 面 → product-smoke 观察面无增量。

**DC-5（测试与质量不可退让）✅**：make test 18694 全绿（2026-09-02 组合树实测）、
clippy 零警告、每切片带单测 + e2e/fixture 资产化（AV1 全流单测 + settle e2e +
桥 roundtrip）。

**结论（2026-09-05 复核）**：DC-1~5 全部满足——DC-4 的 H.264 面随切片 1+2 落地
补齐（mp4 解码/settle/播放/伴生 AAC/seek 全链 e2e），WPT 可执行子集经评估收束
（pinned rev 无 mp4 素材用例——无可导入增量，诚实注记）。**M3 收尾：全切片完成，
goal Done Criteria 全满足。**

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
  「单独立项」决议），agent 不可代判 | ✅ **获批（2026-09-05）**——D-RFC-3a=**有条件
  批准实施**（接受 Cisco 授权链 + Via AAC 池风险面；**前置条件：任何二进制分发/发布
  前须完成法务复核**，实施与分发解耦可回退）；3b=**构建期源码编译**；3c=**AAC 随本期**。
  RFC 已转 Approved、源码冻结解除（见
  [h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)
  §5 决议记录）。征询凭据：msg `om_x100b664d8a6f44b0dee3398474de92b` +
  `om_x100b669923cd64a4c3e335615ed3d9f`；批复来源：session 对话（GB-20260904
  待决策征询跟进）。**2026-09-05 切片 1 落地**：mp4_h264 模块（symphonia isomp4
  demux + openh264 解码 + avcC→Annex-B 转换）+ `VideoTrackDecoder::open_media`
  容器嗅探路由（player/registry/settle 探针/probe_dimensions 统一入口）+
  `decode-h264` feature 门控 + fixture 全流单测（48 帧/PTS 单调/首帧均值 122 同窗
  ±15/时长真值）+ mp4 settle e2e（feature-gated）+ canPlayType 扩 video/mp4 &
  audio/mp4 |
| D4 | **renderer 路径播放泵架构决策**（2026-09-02 深结构缺口发现，2026-09-02 巡检
  补入决策表）：browser tab_worker 主循环有 1ms 帧泵/音频泵（`is_any_playing` 门
  → `tick_all` + `audio_advance_all` + WebAudio `wa.advance`），renderer 路径桥面
  已对齐（VideoPlayerRegistry Arc + `SetWebAudio`/`__zwWA*` 注入）但**主循环无节拍
  驱动 advance**——play 登记后帧/音频永不推进（「登记但不推进」现状，非回归）。
  修复须架构决策：进程内独立泵线程 vs 事件循环节拍（renderer `runtime.run` 当前
  事件驱动无固定节拍）
  为何需用户：多进程线程模型属架构决策域（run-rules rule 11 深结构），待点名后
  实施 | ✅ **获点名（2026-09-05）**——选**事件循环节拍**：renderer `runtime.run`
  主循环加 `is_any_playing` 门控 tick（镜像 tab_worker 已验证模式；单线程状态变更、
  无新并发域、diff 最小）。**否决独立泵线程**（待 cpal 音频主时钟落地后泵角色自然
  弱化，不值得引入第二个线程模型）。征询凭据：msg `om_x100b669923cd64a4c3e335615ed3d9f`；
  批复来源：session 对话（GB-20260904 待决策征询跟进）。**2026-09-05 实施落地**：
  runtime.run 主循环 16ms 节拍 is_any_playing 门控泵（tick_all + audio_advance_all +
  webaudio advance + 帧更新 try_republish_cached 上屏）+ pump_epoch/pump_clock 入
  RendererRuntime + SetVideoPlayers 扩 pump_clock（桥 play 锚与泵 tick 同源——
  「登记但不推进」缺口收口） |

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
   **切片 1 ~ 11 落地明细（2026-09-02 ~ 2026-09-04）**——runner 播放桥前置/
   动态源登记/play() 退避重试/EOF 排空/loop 真面/Ended→play 收口/seekable 面/
   media fragment/headless 时钟/同文档移动等各片明细——已归档至
   [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md)
   （累计口径：media-elements 603P/0F/24PF；每片 evidence/单测见各 commit）。
   fragmented-mp4-end）。
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
| M2 — 连续播放 + 语义驱动 | ✅ M2a + M2b + M2c + 切片 C/D/E/F 收口（播放/真值/桥/帧泵/seek/变速 + 音频面生产链路/增益/导航释放/renderer 对齐 + 色彩面全对齐 + A/V 同步 audio clock 主时钟 + A/V pair ended 面回归守卫 + **D4 renderer 播放泵事件循环节拍（2026-09-05）**）；音频设备面归 media-audio goal（CpalSink 真出声为该 goal 可选切片） |
| M3 — 多格式 + 稳定 + 收尾 | 🔄 AV1 ✅（2026-09-02，D-RFC-2）+ **H.264 全切片 ✅**（2026-09-05，D-RFC-3 获批：切片 1 mp4 demux + openh264 解码 + open_media 路由 + fixture e2e + canPlayType 扩表；切片 2 AAC 链 e2e + Mp4Aac 伴生轨 + precise-seek 前向回退；切片 3 评估收束——WPT pinned rev 无 mp4 素材用例 + 产品 fixture 无 video 面，真解码面无可导入/观察增量，诚实注记）；余 DC-4 WPT 可执行子集（外部门控已解除——随批复落地，现状无 mp4 面）|

## 验证基线

- 测试基线：`make test` 全绿（zero-media default 23 单测 + 1 doctest；engine 2539
  含桥契约测试；webview 667 含桥 e2e + registry 4 + settle e2e 2；browser 411 under
  xvfb）；clippy 零警告；testharness-media 372P/0F/41PF 基线维持（桥 feature-detect
  回落面零回归实证）
- 巡检复验（2026-09-05，渲染流 d2fd8e173 组合态新基线——纯验证轮零代码变更）：
  make test 全量 **18921 全绿零失败**（组合态 SW import-order 测试 flake——
  e55038a2a 无轮询漏改——已补片修复，commit 763cfd996）；webview/engine/media
  套件全绿零回归。DC-1~5 审计结论（Done）不受影响。
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 上屏 e2e 锚点：帧区 RGB 均值 108-138（testsrc2 ≈123.3，M2 切片 C 后与 ffmpeg
  swscale 一致）+ 帧界外白底 + 不可解码 src 占位负例；reftest-upstream
  13981/16730（**83.6%**，replaced-element-003 unmask 已收口 ✓）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## 归档

- [archive/2026-09-01_m0-m2-slices.md](archive/2026-09-01_m0-m2-slices.md) —
  M0 门控收口与 M1/M2 切片全链过程记录（只追加不修改；本控制面保留最新态）。
- [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md) —
  M1a~M2b 历史过程段 + fixture-mounted 切片 1~11 落地明细归档
  （2026-09-04 治理切片；本控制面保留头链切片 5~12 摘要与 DC 审计/决策表）。
