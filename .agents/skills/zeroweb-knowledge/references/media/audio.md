# 音频解码、输出、混音与 Web Audio

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位音频解码无采样、静音/音量不生效、A/V 游标停滞和 AudioContext 合成边界。AudioDecoder/OpusAudioTrack 产出 f32 交错 PCM，AudioSink 消费输出，Mixer 合并多源；页面媒体和 Web Audio 分别由 video_registry 与 webaudio_registry 管理，不是所有页面音频都自动经过 Mixer 或 CpalSink。

### User Task Navigation
- 解码与音轨：`crates/media/src/audio_decode.rs`、`opus_decode.rs`、`av_decode.rs`；输出：`audio.rs`、`audio_cpalsink.rs`。
- 合成与页面接线：`crates/media/src/mixer.rs`、`webaudio.rs`、`crates/webview/src/webaudio_registry.rs`。
- 解码、混音、振荡器和 NullSink 断言分布在这些模块及 `crates/media/src/tests.rs`；本轮未跑测试，也未验证设备出声。

## Gotchas
- AudioDecoder 由 symphonia 的编译期 codec 注册表选择首音轨，损坏 DecodeError 包跳过，UnexpectedEof 按结束处理；Opus 需要独立 opus-decoder 路径，不能从 Ogg 容器探测成功推断该 decoder 可用。 (`crates/media/src/audio_decode.rs`, `crates/media/src/opus_decode.rs`)
- AudioDecoder 的 PTS 由已输出帧数推导，不直接透传容器包 PTS；损坏包被跳过后该时钟仍按成功输出采样计数，不能作为原始容器时间戳的等价物。 (`crates/media/src/audio_decode.rs`)
- Opus 固定输出 48kHz，按 OpusHead pre-skip 的每声道采样数裁首段后才累计输出 PTS；不能把 pre-skip 当交错总长度直接裁一次。 (`crates/media/src/opus_decode.rs`)
- NullSink 帧数等于交错采样数除声道数，过零只观察首声道且跨 write 保连续；这些统计证明喂入采样，不证明声卡消费或真实听感。 (`crates/media/src/audio.rs`)
- CpalSink 的 samples_consumed 实际返回队列积压长度，不是累计消费量；设备回调缺数补静音并计 underrun，格式变化必须重建 sink，不能照函数名做吞吐结论。 (`crates/media/src/audio_cpalsink.rs`)
- Mixer 按最长输入补零并逐采样相加后钳幅，已卸载或 muted 源跳过；不做采样率/声道转换，调用方须先保证块格式一致。 (`crates/media/src/mixer.rs`)
- WebAudioRegistry 当前固定 NullSink，每次 advance 合成 48 帧而非按两次 now 的差值补帧；换泵节拍不能据墙钟前进推断合成时长相同。 (`crates/webview/src/webaudio_registry.rs`)
- WebAudioRegistry 的 start/stop 给传入 when_ms 再加调用时 now，实际是相对当前时刻的延后量，不是原样采用 AudioContext 的绝对时间；不能从 when 参数命名宣称完整调度语义。 (`crates/webview/src/webaudio_registry.rs`)

## Architecture
- 页面独立音频与视频伴音条目各自持 NullSink、解码游标和静默 seek 追赶线，A/V 同步由 registry 游标驱动；AudioSink trait 本身没有设备消费时钟接口。 (`crates/webview/src/video_registry.rs`, `crates/media/src/audio.rs`)
- WebAudioContext 是振荡器列表、源增益和 destination 增益的最小合成模型，波形相位跨批连续，不是完整拉取式节点图、AudioWorklet、OfflineAudioContext 或效果处理器。 (`crates/media/src/webaudio.rs`)
- Web Audio 桥用字符串句柄和参数，返回 "1" 不能独立证明句柄有效或采样已输出；registry 的 set/start/stop 先查 nodes 映射，可观测输出要看帧统计与合成结果。 (`crates/webview/src/webaudio_registry.rs`)

## Decisions
- 纯 Rust Opus 解码补 symphonia 的位流缺位，但仍复用其 Ogg demux；旧 audio_decode 头部“Opus 留待选型”的描述不是当前全媒体能力边界。 (`crates/media/src/audio_decode.rs`, `crates/media/src/opus_decode.rs`)
- audio-cpal 只是 zero-media 的可选设备实现，WebView registry 当前直接构造 NullSink；启用 feature 不等于页面音频已自动接入声卡。 (`crates/media/Cargo.toml`, `crates/webview/src/video_registry.rs`, `crates/webview/src/webaudio_registry.rs`)
