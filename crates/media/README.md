# ZeroWeb Media (`zero-media`)

> 媒体解码管线 — webm/Matroska demux、VP9/AV1 解码、音频解码、播放驱动与音频输出面

## 概述

`ZeroWeb Media` (`zero-media`) 实现 ZeroWeb 的媒体解码与播放管线，是 media-playback / media-audio goal 的进程内 crate 产物（RFC 路线 C：VP9/AV1 开源先行，解码与播放解耦，见 `docs/specs/video-decode-playback-spec-rfc.md`）。视频面：webm/Matroska 容器解析 → VP9（纯 Rust `rusty_vp9`）或 AV1（`decode-av1` feature，系统 libdav1d）解码 → YUV（I420）→ RGBA 帧转换，输出与 `zero-render-foundation` 的 `ImageData`（行优先 RGBA8）同构的 `DecodedVideoFrame`，M1b 帧上屏走 canvas 同款 ImagePrimitive 通路。音频面：symphonia 容器/编码探测（mp3 + ogg/vorbis）、Ogg Opus（纯 Rust `opus-decoder`）、webm 内嵌 vorbis/opus 轨的同源解码，以及 `AudioSink` 输出面（headless `NullSink` 可观测 / `audio-cpal` feature 真设备 `CpalSink`）、多源混音总线与 Web Audio 振荡器合成最小面。

## 主要功能

- **容器解析与视频解码** — `VideoDecoder::open_webm` / `open_webm_vp9` 打开 webm（V_VP9 → `rusty_vp9`、V_AV1 → `decode-av1` feature 的 `Av1Decoder` codec 自路由），`next_frame()` 逐帧产出 `DecodedVideoFrame`（pts_ms + RGBA），支持 `duration_ms()` 与 `seek_to_ms()`
- **YUV→RGBA 转换** — I420 平面转换（VP9/AV1 共用单点实现，`ColorSpace` / `ColorMatrix` BT.601/709 矩阵选择）
- **播放驱动** — `VideoPlayer`（实现 `VideoClock` trait）：play/pause/ended + currentTime 真值推进，`tick(now_ms)` / `sync_to_media_time` 按单调时钟产出应展示帧（落后多帧时快进到最新可展示帧）；调用方注入时钟保证可测试性，生产侧挂 rAF event loop
- **音频解码** — `AudioDecoder`（symphonia：mp3 + ogg/vorbis → f32 交错 PCM）、`open_ogg_opus`（RFC 7845 容器 + 纯 Rust RFC 6716/8251 位流解码）、`open_webm_audio_track` / `open_webm_opus_audio_track`（webm 音轨重封装 OGG 页流后同源解码）
- **音频输出面** — `AudioSink` trait（`start` / `write` 交错 f32 PCM）：`NullSink`（headless/CI 默认，帧数/过零率可观测断言）与 `CpalSink`（`audio-cpal` feature，真实设备 ALSA 输出）
- **混音总线** — `Mixer` 多源 f32 帧叠加 + per-source `volume`/`muted` 增益（软削幅 clamp [-1,1]），`SourceHandle` 独立挂载/卸载
- **Web Audio 最小面** — `OscillatorState`（四型波形纯函数合成，相位累积防 alias）+ `WebAudioContext`（源列表 → per-source 增益 → 下游 sink 的 `advance` 推进）
- **feature gate** — `audio-cpal`（真实设备输出，编译需 ALSA dev 头）与 `decode-av1`（AV1 解码，链接系统 libdav1d）默认关闭，headless/CI 走默认面

## 使用示例

```rust
use zero_media::{VideoDecoder, VideoPlayer, AudioDecoder, Mixer, NullSink};

// 视频解码：打开 webm（VP9/AV1 自路由），逐帧产出 RGBA
let mut decoder = VideoDecoder::open_webm(&webm_bytes)?;
if let Some(duration) = decoder.duration_ms() {
    println!("时长 {} ms", duration);
}
while let Some(frame) = decoder.next_frame()? {
    // frame.pts_ms / frame.width / frame.height / frame.rgba
}

// 播放驱动：调用方注入单调毫秒时钟（生产侧挂 rAF event loop）
let mut player = VideoPlayer::new(VideoDecoder::open_webm(&webm_bytes)?);
player.play(now_ms);
if let Some(frame) = player.tick(now_ms + 16)? {
    // 到达展示时间的帧
}

// 音频解码：f32 交错 PCM 逐包输出
let mut audio = AudioDecoder::open(&audio_bytes)?;
while let Some(batch) = audio.next_batch()? {
    // batch.samples / sample_rate() / channels()
}

// 混音总线：多源 f32 块叠加写入下游 sink
let mut mixer = Mixer::new();
let src = mixer.attach();
mixer.set_volume(src, 0.8);
let mut sink = NullSink::new();
sink.start(zero_media::AudioFormat { sample_rate: 48000, channels: 2 })?;
mixer.mix_into(&[(src, source_block)], &mut sink)?;
```
