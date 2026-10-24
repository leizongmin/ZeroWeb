# M1 基线测量——长视频资源、帧消费链、权限表与平台能力

**日期**：2026-10-24
**状态**：PASS（DC-1 测量面成立；时钟保真与音频输出两缺口转 M3/RFC 待决）
**goal**：[media-runtime-isolation](../../master.md)

## 1. 版本与环境

| 项 | 值 |
|---|---|
| HEAD | `666101a7a3aa378bb57bba63f68309544603d048`（main，本轮改动前） |
| 平台 | Linux 6.18.33.2-microsoft-standard-WSL2，AMD Ryzen 7 9700X（16 线程），39GB RAM |
| 工具链 | rustc 1.99.1 / cargo 1.99.1，edition 2024 |
| 测量档 | release（optimized）；debug 档解码性能无基线意义，不采信 |
| feature | 生产解析面：zero-webview default → zero-media `default = []`（VP9-only，`decode-h264`/`decode-av1`/`audio-cpal` 全关） |
| 隔离与守卫 | 编译阶段无阈值；执行阶段 `./target/test-guard --time-limit 900 --per-proc-mem 4`；clippy 走 24G systemd scope（`zeroweb-dev-guard-*-clippy-targeted`） |

原始输出：[m1baseline-raw.log](m1baseline-raw.log)（`M1BASELINE key=value` 行，harness `crates/webview/tests/media_m1_baseline.rs`，5 场景 6/6 PASS）。

## 2. 长视频资源基线（新 fixture）

`tests/fixtures/media/long-webm-vp9-60s.webm`（1.2MB）与 `long-webm-vp9-vorbis-60s.webm`（1.4MB）：
仓内 2s fixture（ffmpeg 7.1.5 生成，来源清白声明见 README）经 `scripts/make-long-video-fixture.py`
簇重复 ×30 重 mux——零外部素材、零编码器依赖。生成器重建 Info Duration / SeekHead / Cues
（matroska-demuxer 0.8.1 的 `seek()` 硬依赖 SeekHead Cluster 条目；CueClusterPosition 相对
Segment data start）。

生成过程暴露并修正两处自建容器缺陷（脚本内，非产品代码）：尺寸 varint 长度标记位错位
（簇被读成 2.7GB → 流在首簇 EOF）；cue 位置漏加 Info 元素包装头 5 字节（系统性 -5 偏移）。
终态经 Python 自检（cue 落点逐簇命中 `1F43B675` 簇头）与独立 demuxer 探针（seek 0/2000/15000
全 OK）双重验证。60s 播放全程无错帧、Ended 可达（1440 帧 @24fps 全呈现）。

## 3. 测量结果（DC-1 测量面）

### 3.1 解码与泵容量（16ms 模拟节拍，registry tick_all → ImageCache 生产链）

| 场景 | 墙钟（60s 媒体） | CPU | 核 | 呈现帧 | 每 tick p50 | 帧 tick p95 |
|---|---|---|---|---|---|---|
| 纯视频 VP9 60s | 1076ms | 1070ms | 0.994 | 1440 | 400ns | ~780µs |
| A/V 双轨 60s | 1733ms | 1740ms | 1.004 | 1440 | 748µs | ~1.39ms |

- 解码容量 ≈ 34–56× 实时：单核软解 320×240@24fps 余量充足；空 tick p50=400ns 证明
  `is_any_playing` 门控零开销成立（runtime.rs:4096 同款）。
- 每 tick 消耗（p95 ~1ms）远低于 16ms 泵节拍：renderer 主循环节拍不被解码挤占。

### 3.2 时钟保真（真实节拍 15s 段，16ms sleep 泵）

**FINDING（缺陷量级钉住，修复归 M3/RFC 切片）**：A/V pair 的 media clock（audio cursor，
页面可见 currentTime 真值）速率 **1.45402**——系统性快于墙钟 45%。根因形态：
`AudioEntry/WebmAudioEntry::advance_to`（webview/src/video_registry.rs）整批解码推进，
`cursor_ms` 停在「越过 target 的首个批次末」；vorbis 批粒度（~1024 样本 ≈23ms）对 16ms 泵
每拍平均 +batch/2 超越。60s 源实测 ~41s 墙钟走完（`media_end_s=60.677 @ wall 1733ms`
为容量档；真实节拍档 rate 同款）。harness 断言收紧为 sanity 带 0.5–3.0 防回退。

### 3.3 A/V 漂移（media 层真值，VideoPlayer.sync_to_media_time）

60s 双轨：漂移（audio cursor − 呈现帧 pts）p50=11ms / p95=21ms / max=35ms / mean=11.1ms，
1440 帧全呈现。量化面正常（≤1.5 帧间隔 41.7ms；均值 ≈ 半帧间隔属 present_pending 弹出语义
的期望形态），**视频追音频主时钟不脱轨**。

### 3.4 seek 基线（cue 路径，纯视频 60s）

| 目标 | seek 耗时 | 落点 | 续播 |
|---|---|---|---|
| 15s | 300ms（冷） | 精确 15.000s | 至流末 59.960s |
| 30s | 1.1ms（热） | 精确 30.000s | 至流末 59.968s |
| 45s | 932ms | 精确 45.000s | 至流末 59.960s |

cue 路径全部精确命中（precise-seek 语义成立）。耗时方差大（冷 I/O + VP9 解码器重建 +
narrow phase 前向扫块），30s 热路径 O(log n) 达标；冷/热差与 15s/45s 慢点为后续
M3 seek 切片的优化证据，本轮不展开。

## 4. 帧复制链静态图（decode → 上屏）

生产链（runtime.rs:4122 泵 → try_republish_cached → CPU/GPU 光栅）逐节点：

| # | 节点 | 位置 | 每帧成本 |
|---|---|---|---|
| C1 | YUV→RGBA 转换写 | media/src/decode.rs `to_rgba`/`planes_to_rgba` | 1 次分配 + 全帧写（320×240×4=300KB） |
| C2 | ImageData::from_rgba | render-foundation/src/image_cache.rs:76 | move（零拷贝）+ **逐像素纯色扫描**（全帧读一遍） |
| C3 | image_cache.insert_with_key | image_cache.rs:1212 | move（零拷贝） |
| C4 | **快照全量深拷贝** | image_cache.rs:1341 `duplicate_for_snapshot`（webview.rs:1713 publish 路径） | **每次发布全缓存逐条 clone**——播放期每帧 300KB 级 |
| C5 | 光栅采样 | cpu/mod.rs:1284 `image_cache.get`（引用）+ 逐目标像素双线性 | 每目标像素 1 读 1 写（含混合） |

结构性放大点（非逐帧拷贝，但随帧更新触发）：

- **每帧全量重渲染**：帧更新 → `try_republish_cached`（runtime.rs:3217）→ `wv.render()`
  整文档 style+layout+paint 重跑（trace 行 `full render in …us`），无脏区/层化短路径。
- C2 纯色扫描对视频帧恒 false（帧间差异），属固定 O(n) 税，可按来源跳过（OPTIMIZATION 候选，归 M3）。

结论：解码后到上屏每帧至少 2 次全帧内存写（C1 + C4）+ 1 次全帧读（C2 扫描）+ 光栅采样；
CPU 路径无零拷贝上屏通道。这是 M2 RFC「平台解码/进程隔离」的基线输入：进程隔离若引入
额外帧传递，当前拷贝面上还要叠加 IPC 拷贝成本。

## 5. 权限表（现有媒体面能力边界）

| 能力 | 现状 | 位置 |
|---|---|---|
| 源获取 | registry 未命中时 `source_provider` 同步回调直联 GET（秒级上界，锁外） | webview.rs:148、video_registry.rs:1780+ |
| MSE 缓冲封顶 | `MSE_FEED_MAX_BYTES = 256MB`（页面 appendBuffer 字节，超限拒绝→QuotaExceededError 面） | video_registry.rs:477 |
| 图片缓存预算 | 256 entries / 64MB（`ImageCache::default`，LRU 淘汰） | image_cache.rs:1352 |
| 源字节驻留 | `sources`/`av_sources` HashMap 无总量封顶（单源无界；release/clear 导航面释放） | video_registry.rs:444–455 |
| 解码线程模型 | 进程内单线程同步解码（renderer 主循环泵内联），无独立解码进程/线程池 | runtime.rs:4122 |
| 音频输出 | `NullSink`（静音计数器）；真设备输出 `audio-cpal` feature 默认关，开则失败回落 NullSink | audio.rs:80、audio_cpalsink.rs:4 |
| JS 桥面 | `__zw_video_*`/`__zw_ms_*` 回调族（feature-detect：未注册回落 headless 路径） | video_registry.rs:1700+ |
| 硬解 | 无任何平台硬解路径（见 §6） | — |

权限缺口（记 M2 RFC 输入）：源字节无界驻留（对齐 MSE 封顶策略待做）；解码无内存/时长上限
（病态源在单源字节量级上受 fetch 面约束，解码期无 quota）。

## 6. 平台能力核查（Linux/WSL2）

- `/dev/dri` 不存在：VAAPI/V4L2/NVDEC 均**不可用**，本平台无硬件解码路径；DC-4「平台无硬解」
  在本机即常态，软件回退即唯一路径（现有 VP9 纯 Rust 软解即回退面，测量见 §3.1）。
- 声音设备：WSL2 无 PulseAudio/ALSA 直通（`audio-cpal` 关闭态运行）——NullSink 即生产行为。
- openh264（decode-h264）与 libdav1d（decode-av1）为系统库 + feature 门控，默认关
  （Cargo.toml:182–191）；本轮未启用、未安装新 SDK。
- 平台结论：**本平台能力查询不虚报**（canPlayType/isTypeSupported 真值随 feature 门控）；
  硬解候选（macOS VideoToolbox 等）只能跨平台核查，归 M2 RFC。

## 7. background-resource 媒体时钟三缺口盘点（M1 盘点输入）

上游移交（archive/background-resource-lifecycle：DC-2 INCONCLUSIVE 子项）锚点逐条核对，
**三缺口在当前源码全部在位**：

| # | 缺口 | 当前锚点 | 核对 |
|---|---|---|---|
| 1 | autoplay 置态绕桥——`_zwMediaAutoplay` 直置 `ms.playing=true`，不经 play() IDL 路径，桥不武装、registry 不起播 | engine/src/js_dom_shim/part06.js:10084 | 在位 |
| 2 | `_zwMediaTimeMarchesOn` 死钩子——定义后全仓零调用方（仅 part01b.js:727 注释引用） | engine/src/js_dom_shim/part06.js:16350 | 在位 |
| 3 | audio load 桥武装未命中——`bridgeOn` 仅在 play() IDL 同步/重试命中路径置位（part03.js:23558/23610），load/autoplay 路径无武装点 | engine/src/js_dom_shim/part03.js:23558/23610 | 在位 |

三缺口修复面归媒体播放语义（engine shim + registry 桥），属 M3 切片候选；本轮只盘点不修。
背景流基线 v2 回补条件（缺口修复后 bg02-media 复测）依赖此修复，保持上游记录不动。

## 8. 结论与待决

- DC-1 测量面（CPU/复制/同步报告 + 能力对照）：**本轮成立**（长视频 60s 版本化基线 + 复制链
  + 权限表 + 平台核查），DC-1 整体收口待 M2 RFC 评审后由用户确认。
- 转入待决/后续：
  1. 时钟量化缺陷（rate 1.454）→ M3 切片候选（advance_to 批内对齐）；
  2. NullSink 生产行为（无声输出）→ M2 RFC 平台音频面议题；
  3. 三缺口修复 + bg02-media 基线 v2 → M3/engine 切片；
  4. 帧拷贝链 C2/C4 优化面 → M2 RFC 成本模型输入。
- M2（RFC）：NOT_STARTED，架构门未解锁（用户决策项）。
- 清理：无残留进程/目录；/tmp/seekprobe、/tmp/cue_debug.py 为本机临时探针不入仓。

## 9. 改动面

| 类 | 文件 |
|---|---|
| 新 fixture | tests/fixtures/media/long-webm-vp9-60s.webm、long-webm-vp9-vorbis-60s.webm |
| 生成器 | scripts/make-long-video-fixture.py |
| 测量 harness | crates/webview/tests/media_m1_baseline.rs（5 场景，全 `#[ignore]` 显式运行） |
| 文档 | tests/fixtures/media/README.md、本 goal master.md 与 evidence |
