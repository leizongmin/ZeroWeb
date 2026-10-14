# 容器、视频解码与 codec 路由

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位容器能解析但视频轨打不开、末帧提前丢失、seek 后帧序异常和颜色偏差。`VideoTrackDecoder` 统一 WebM、MP4/H.264 与 MSE 的消费面；各 codec 只输出毫秒 PTS 和行优先 RGBA8，不派 HTML 媒体事件。伴音轨的数据封装在 av_decode/opus_decode，播放同步见 [播放](playback.md)。

### User Task Navigation
- 路由与颜色：`crates/media/src/decode.rs`；AV1：`av1_decode.rs`；MP4：`mp4_h264.rs`。
- 常驻解码断言：`crates/media/src/tests.rs` 和 codec 模块测试；输入说明：`tests/fixtures/media/README.md`。本轮只读，不声称 ffmpeg 对比或解码测试已实跑。

## Gotchas
- open_media 按 EBML 前四字节或 offset 4 的 ftyp 嗅探容器，不按 URL/MIME 分派；MP4 在 decode-h264 关闭时返回 NoVideoTrack，不能由容器解析依赖存在推断 H.264 可用。 (`crates/media/src/decode.rs`, `crates/media/Cargo.toml`)
- open_webm 只选首个视频轨再判断 VP9/AV1，首轨不支持不会继续找后续可用轨；专用 open_webm_vp9 则直接搜索 VP9 轨，两入口的多轨行为不同。 (`crates/media/src/decode.rs`)
- demux 耗尽只进入 draining，VP9 排空时 Again 表示隐藏帧需继续拉取，Eof 才结束；flush 后只取一帧便置 eof 会丢尾部帧并让播放器提前 Ended。 (`crates/media/src/decode.rs`, `git:3ad150f3dd069c09baf8238c1135c18c01addce0`)
- seek 会重建参考链；cue 命中可直接暂存落点关键帧，不保证该帧 PTS 大于等于 target，只有回退从流首前向解码的路径寻找首个不早于目标的帧。 (`crates/media/src/decode.rs`)
- MP4 的 avcC SPS/PPS 与长度前缀 NALU 要转 Annex-B 后喂 OpenH264，参数集在首包注入一次；不能把 MP4 包原样交给 decoder。 (`crates/media/src/mp4_h264.rs`)
- 未出现 WebM Colour 元素时默认 BT.601 limited，有 Colour 但矩阵缺省/未知时走 BT.709；AV1 从位流 Picture 读色彩信息，H.264 当前用默认色彩，不是所有 codec 共用一套元数据来源。 (`crates/media/src/decode.rs`, `crates/media/src/av1_decode.rs`, `crates/media/src/mp4_h264.rs`)
- planes_to_rgba 会归一高位深到 RGBA8并用最近邻色度采样，其他矩阵近似 BT.709，没有 HDR tone mapping；共用转换函数不证明全部像素格式都已验收。 (`crates/media/src/decode.rs`)

## Architecture
- decoder 的 pending 单槽同时承接 seek 命中和 un_read 背压；播放器必须把尚未到展示时间的帧退回，不可当作当前帧返回后丢弃。 (`crates/media/src/decode.rs`, `crates/media/src/player.rs`)
- WebM Vorbis 伴音复用 Ogg 封装和 symphonia 解码，Opus 伴音另走纯 Rust opus-decoder；视频 codec 路由不承担音轨同步。 (`crates/media/src/av_decode.rs`, `crates/media/src/opus_decode.rs`, `crates/webview/src/video_registry.rs`)

## Dependencies
- zero-media 默认 feature 为空，VP9 不引入 C 解码依赖；AV1 的 decode-av1 链接系统 dav1d，H.264 的 decode-h264 使用源码构建 OpenH264，二者是明确的可选依赖边界。 (`crates/media/Cargo.toml`, `crates/media/src/av1_decode.rs`)
- WebView 显式转发 decode-av1/decode-h264，settle 探针与播放共用 open_media；fixture 的 baseline/yuv420p、短单轨条件不能证明 High profile、复杂重排或多轨完整支持。 (`crates/webview/Cargo.toml`, `crates/webview/src/async_load.rs`, `tests/fixtures/media/README.md`)
