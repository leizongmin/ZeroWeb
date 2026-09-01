//! # zero-media
//!
//! 媒体解码管线 — webm/Matroska demux → VP9 解码 → YUV→RGBA 帧转换。
//!
//! media-playback goal M1a 产物（RFC 路线 C：VP9/AV1 开源先行 + 进程内 crate），
//! 见 [docs/specs/video-decode-playback-spec-rfc.md]。
//!
//! ## 分层（RFC §3.1：解码与播放解耦）
//!
//! - [`decode`]：容器解析（webm/Matroska）+ VP9 纯 Rust 解码（`rusty_vp9`）+
//!   YUV（I420）→ RGBA 转换。输出 [`DecodedVideoFrame`] 的 RGBA 面与
//!   `render-foundation` 的 `ImageData`（行优先 RGBA8）同构，M1b 帧上屏走
//!   R3268 canvas 同款 ImagePrimitive 通路。
//! - [`clock`]：[`VideoClock`] trait——播放驱动（帧率时钟/play/seek/currentTime）
//!   对 HTMLMediaElement 语义层的真值化接口。
//! - [`player`]：[`VideoPlayer`]——`VideoClock` 的帧率驱动实现（M2a）：play/pause/
//!   ended + currentTime 真值推进，调用方注入单调时钟（rAF event loop 挂点）；
//!   media-elements 语义层 headless 近似驱动按此接口替换，语义层不返工。
//!
//! ## 格式范围（路线 C 首期）
//!
//! VP9 视频起步（免专利费、纯 Rust 零 C 依赖）；AV1（`dav1d` 绑定）为 M3
//! `decode-av1` feature（D-RFC-2）；H.264/HEVC 单独立项（D-RFC-3）。
//!
//! [docs/specs/video-decode-playback-spec-rfc.md]: ../../../docs/specs/video-decode-playback-spec-rfc.md

mod audio;
#[cfg(feature = "audio-cpal")]
mod audio_cpalsink;
mod audio_decode;
mod av_decode;
mod clock;
mod decode;
mod mixer;
mod player;

#[cfg(test)]
mod tests;

pub use audio::*;
#[cfg(feature = "audio-cpal")]
pub use audio_cpalsink::*;
pub use audio_decode::*;
pub use av_decode::*;
pub use clock::*;
pub use decode::*;
pub use mixer::*;
pub use player::*;
