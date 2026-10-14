# 媒体解码、播放与 Web Audio

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 zero-media 的容器/codec、播放时钟、音频输出/混音和最小 Web Audio，以及 WebView 注册表、HTMLMediaElement/MSE 特有桥接。媒体算法不承载 DOM 事件，页面语义不混入 codec；物理图像缓存与光栅化见 [图形](../graphics/index.md)，通用 JS 任务调度见 [脚本](../scripting/index.md)，多进程泵与帧交付见 [运行时](../runtime/index.md)。

### User Task Navigation
| 用户问题 | 当前源码入口 | 深入知识 |
| --- | --- | --- |
| WebM/MP4 可获取但无法解码或色偏 | `crates/media/src/decode.rs`、`mp4_h264.rs` | video-decode |
| play 后首帧冻结、seek/ended 异常 | `crates/media/src/player.rs`、`crates/webview/src/video_registry.rs` | playback |
| 音频游标、静音、AudioContext 合成 | `crates/media/src/audio_decode.rs`、`crates/webview/src/webaudio_registry.rs` | audio |
| append 无帧、续喂闩锁、能力表虚报 | `crates/media/src/mse_feed.rs`、`crates/engine/src/js_dom_shim/part06.js` | mse-and-streaming |

既有媒体断言在 `crates/media/src/tests.rs`、各模块测试及 `crates/webview/src/video_registry.rs`；fixture 说明在 `tests/fixtures/media/README.md`。本次只读核验，没有运行编解码、浏览器、MSE 或设备测试。

## Gotchas
- codec feature 与页面能力表要分别核对：zero-media 默认不启 AV1/H.264，WebView 转发 feature，JS 又以桥是否存在判断可用面；不能只看构造器或 Cargo 依赖宣称支持。 (`crates/media/Cargo.toml`, `crates/webview/Cargo.toml`, `crates/engine/src/js_dom_shim/part06.js`)
- registry 与 settle/Painter 共用 image_resource_key 算法但输入不完全相同：registry剥query/fragment，后两者保留；带query可能导致持续帧与首帧键分离，注册表还按归一化URL共享而非每元素独立。 (`crates/webview/src/video_registry.rs`, `crates/webview/src/async_load.rs`, `crates/engine/src/paint/painter/text.rs`)
- NullSink 是当前页面媒体/Web Audio 的可观测端点，CpalSink 是独立可选设备实现；帧数或过零断言不证明真实音频输出。 (`crates/media/src/audio.rs`, `crates/media/src/audio_cpalsink.rs`, `crates/webview/src/webaudio_registry.rs`)

## Architecture
- DecodedVideoFrame 统一毫秒 PTS 和 RGBA8，DecodedAudio 统一 f32 交错 PCM；VideoClock 只提供读侧时间，HTML 状态与事件由 engine shim/宿主消费。 (`crates/media/src/decode.rs`, `crates/media/src/audio_decode.rs`, `crates/media/src/clock.rs`, `crates/engine/src/js_dom_shim/part06.js`)
- 静态视频流末和 MSE 写入边缘在 VideoTrackDecoder::is_exhausted 分叉，VideoPlayer 共用调度但不能共用所有 None→Ended 规则。 (`crates/media/src/decode.rs`, `crates/media/src/player.rs`, `crates/media/src/mse_feed.rs`)

## Child Knowledge Nodes
- [容器、视频解码与 codec 路由](video-decode.md) — 魔数选容器、轨选择、排空、seek、颜色与可选解码依赖。
- [播放时钟、HTMLMediaElement 与帧注入](playback.md) — 真值时钟、背压、源登记、A/V 同步、首帧和持续换帧。
- [音频解码、输出、混音与 Web Audio](audio.md) — PCM 时间轴、Opus、NullSink/设备边界和最小合成模型。
- [MSE、流式输入与 feature 覆盖](mse-and-streaming.md) — 增长缓冲、读侧重建、双轨路由、能力表及未实现语义。
