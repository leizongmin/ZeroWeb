//! 解码管线 — webm/Matroska demux + VP9 解码 + YUV→RGBA 帧转换。
//!
//! 依赖：`matroska-demuxer`（纯 Rust 容器解析）+ `rusty_vp9`（纯 Rust VP9 解码，
//! 零 C 依赖——RFC 路线 C 的 M1 约束，D-RFC-2）。首帧解码与 ffmpeg 参照逐字节
//! 一致（2026-09-01 探针实测，fixture `sample-webm-vp9.webm` 48 帧）。

use std::io::Cursor;

/// 解码管线错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// 容器解析失败（非 webm/Matroska、EBML 结构损坏等）。
    #[error("container error: {0}")]
    Container(String),
    /// 容器中不含 VP9 视频轨。
    #[error("no VP9 video track in container")]
    NoVideoTrack,
    /// VP9 位流解码失败（malformed 或 unsupported feature）。
    #[error("vp9 decode error: {0}")]
    Vp9(String),
}

impl From<matroska_demuxer::DemuxError> for DecodeError {
    fn from(e: matroska_demuxer::DemuxError) -> Self {
        Self::Container(e.to_string())
    }
}

impl From<rusty_vp9::Error> for DecodeError {
    fn from(e: rusty_vp9::Error) -> Self {
        Self::Vp9(e.to_string())
    }
}

/// 解码出的视频帧——RGBA 像素 + 展示时序。
///
/// RGBA 面（行优先 RGBA8）与 `render-foundation::ImageData` 同构，M1b 帧上屏
/// 直接进 `ImageCache` + `ImagePrimitive` 通路（R3268 canvas 同款）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedVideoFrame {
    /// 展示时间戳（毫秒，容器 timebase 归一）。
    pub pts_ms: u64,
    /// RGBA 像素（行优先，4 字节/像素）。
    pub rgba: Vec<u8>,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
}

/// 逐帧读取的 webm/VP9 视频解码器。
///
/// 用法（push/pull 迭代）：
///
/// ```no_run
/// # use zero_media::VideoDecoder;
/// # let bytes: Vec<u8> = Vec::new();
/// let mut dec = VideoDecoder::open_webm_vp9(&bytes)?;
/// while let Some(frame) = dec.next_frame()? {
///     // frame.rgba / frame.pts_ms ...
/// }
/// # Ok::<(), zero_media::DecodeError>(())
/// ```
pub struct VideoDecoder {
    demuxer: matroska_demuxer::MatroskaFile<Cursor<Vec<u8>>>,
    vp9: rusty_vp9::Vp9Decoder,
    /// 视频轨号（Matroska track number）。
    video_track: u64,
    /// 时间戳缩放（ns/tick；frame.timestamp 的单位为 tick）。
    timestamp_scale: u64,
    /// 流是否已到末尾（`next_frame` 返回 `Err(Eof)` 后置位）。
    eof: bool,
    /// seek 前向推进时暂存的「已解出但未消费」帧（`seek_to_ms` 命中 target 时
    /// 写入；下一次 `next_frame` 先弹出——spec precise-seek 不丢帧）。
    pending: Option<DecodedVideoFrame>,
}

impl VideoDecoder {
    /// 从 webm/Matroska 字节打开 VP9 视频解码器。
    ///
    /// 校验容器含 VP9 视频轨（`V_VP9`），否则 [`DecodeError::NoVideoTrack`]。
    pub fn open_webm_vp9(data: &[u8]) -> Result<Self, DecodeError> {
        let cursor = Cursor::new(data.to_vec());
        let demuxer = matroska_demuxer::MatroskaFile::open(cursor)?;
        let video_track = demuxer
            .tracks()
            .iter()
            .find(|t| t.track_type() == matroska_demuxer::TrackType::Video && t.codec_id() == "V_VP9")
            .map(|t| t.track_number().get())
            .ok_or(DecodeError::NoVideoTrack)?;
        Ok(Self {
            timestamp_scale: demuxer.info().timestamp_scale().get(),
            demuxer,
            vp9: rusty_vp9::Vp9Decoder::new(),
            video_track,
            eof: false,
            pending: None,
        })
    }

    /// 容器声明的时长（毫秒）；容器未声明时 `None`。
    pub fn duration_ms(&self) -> Option<u64> {
        // https://www.matroska.org/technical/basics.html#timestampscale
        // Duration 以 timebase tick 计（scale ns/tick），tick × scale / 1e9 秒 → 毫秒。
        self.demuxer
            .info()
            .duration()
            .map(|d| (d * self.timestamp_scale as f64 / 1e6).round() as u64)
    }

    /// seek 到目标位置（毫秒，媒体时间轴）——M2b 精确 seek（spec「seek」算法面）。
    ///
    /// 两阶段：
    /// ① demuxer 经 Cues（cue 二分）定位 ≤ target 的最近块并重建 VP9 解码器
    ///   （cue 点即 keyframe；旧参考链作废）。落点帧可解 → 完成（cue 齐全的流
    ///   O(log n) 到位）。
    /// ② 回退：无 Cues 或落点为非 keyframe（VP9 参考链断裂不可解——如 testsrc2
    ///   单 keyframe 流）→ 全量回退（demuxer seek 0 + 解码器重建），**前向解码**
    ///   至 ≥ target 的首帧后定位完成（spec precise-seek：从最近 keyframe 解码到
    ///   精确呈现点）。
    ///
    /// 语义契约：完成后的「下一次 [`Self::next_frame`]」返回 pts ≥ target 的首帧
    ///（② 路径精确命中；① 路径为 cue 点帧，调用方 VideoPlayer 把播放位置对齐到
    /// 实际帧 pts）。
    /// https://www.matroska.org/technical/cues.html
    /// https://html.spec.whatwg.org/multipage/media.html#seek
    pub fn seek_to_ms(&mut self, target_ms: u64) -> Result<(), DecodeError> {
        // 容器 timebase：block.timestamp 为 tick（scale ns/tick）；demuxer seek
        // 参数同 timebase（默认 1e6 ns = 毫秒 tick）。ms→tick 与 next_frame 的
        // tick→ms 互逆。
        // https://www.matroska.org/technical/basics.html#timestampscale
        let target_tick = (target_ms as u128 * 1_000_000 / self.timestamp_scale as u128) as u64;
        let target_ns = target_ms as u128 * 1_000_000;

        // ① cue 路径：定位 + keyframe 验证（cue 点即 keyframe；非 keyframe 落点
        //   ——无 Cues 的线性搜索——不可解后续参考链 → 回退）。
        self.demuxer.seek(target_tick)?;
        self.vp9 = rusty_vp9::Vp9Decoder::new();
        self.eof = false;
        let mut block = matroska_demuxer::Frame::default();
        if matches!(self.demuxer.next_frame(&mut block), Ok(true))
            && block.track == self.video_track
            && block.is_keyframe == Some(true)
        {
            // keyframe 落点：解码此块并暂存（下一次 next_frame 消费——不丢帧）。
            let pts_ms = block.timestamp * self.timestamp_scale / 1_000_000;
            self.vp9.push(&block.data, Some(pts_ms as i64))?;
            let frame = self.vp9.next_frame()?;
            self.pending = Some(to_rgba(frame));
            return Ok(());
        }

        // ② 回退：从流首 keyframe 前向解码到 target（spec precise-seek）。
        self.demuxer.seek(0)?;
        self.vp9 = rusty_vp9::Vp9Decoder::new();
        self.eof = false;
        loop {
            // EOF 前必有 ≥ target 的帧（target ≤ duration 由调用方保证；越界时
            // EOF 返回 None 亦为合法终态——调用方按 ended 处理）。
            match self.try_decode_next_frame() {
                Ok(Some(frame)) => {
                    if frame.pts_ms as u128 * 1_000_000 >= target_ns {
                        self.pending = Some(frame);
                        return Ok(());
                    }
                }
                Ok(None) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// 试解下一帧（seek 定位探测/前向推进共用）；EOF → Ok(None)。
    fn try_decode_next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        match self.next_frame() {
            Ok(frame) => Ok(frame),
            // seek 后参考链断裂的坏帧：VP9 解码错误 → 交由调用方回退判定。
            Err(e @ DecodeError::Vp9(_)) => Err(e),
            Err(e) => Err(e),
        }
    }

    /// 解码并返回下一帧；流结束返回 `Ok(None)`。
    ///
    /// 内部维持 demux→decode 推进，直到产出一帧可展示帧或流末。
    pub fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
        }
        if self.eof {
            return Ok(None);
        }
        let mut block = matroska_demuxer::Frame::default();
        loop {
            match self.demuxer.next_frame(&mut block) {
                Ok(true) => {
                    if block.track != self.video_track {
                        continue;
                    }
                    // https://www.matroska.org/technical/basics.html#timestampscale
                    // block timestamp 为 timebase tick（scale ns/tick）→ 毫秒。
                    let pts_ms = block.timestamp * self.timestamp_scale / 1_000_000;
                    self.vp9.push(&block.data, Some(pts_ms as i64))?;
                    // push 后必有输出（rusty_vp9 无前瞻重排——VP9 位流本身
                    // 按 display 顺序编码 show_existing_frame 重发）。
                    let frame = self.vp9.next_frame()?;
                    return Ok(Some(to_rgba(frame)));
                }
                Ok(false) => {
                    // 容器末：冲刷解码器残余帧（若有）。
                    self.eof = true;
                    self.vp9.flush();
                    match self.vp9.next_frame() {
                        Ok(frame) => return Ok(Some(to_rgba(frame))),
                        // Eof/Again 均为无残余——正常收尾。
                        Err(rusty_vp9::Error::Eof | rusty_vp9::Error::Again) => return Ok(None),
                        Err(e) => return Err(e.into()),
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// `rusty_vp9::DecodedFrame`（YUV 平面）→ [`DecodedVideoFrame`]（RGBA）。
///
/// 支持 8/10/12 bit 与 4:2:0/4:2:2/4:4:4（逐平面定点转换）；当前 fixture 与
/// 上游 webm 系素材主面为 8bit 4:2:0。
fn to_rgba(frame: rusty_vp9::DecodedFrame) -> DecodedVideoFrame {
    let w = frame.width as usize;
    let h = frame.height as usize;
    let shift = frame.bit_depth - 8; // 10/12 bit → 8 bit 的高位移除
    let chroma_w = w.div_ceil(1 + frame.subsampling_x as usize);
    let chroma_h = h.div_ceil(1 + frame.subsampling_y as usize);

    // 色度定点索引（65536 = 1.0）：luma 像素 → 色度平面最近邻采样位置。
    let fx = |x: usize, sw: usize| x * 65536 / sw;
    let fy = |y: usize, sh: usize| y * 65536 / sh;

    let sample = |plane: &[u8], stride: usize, x: usize, y: usize| -> u32 {
        let idx = y * stride + x * if frame.bit_depth > 8 { 2 } else { 1 };
        if frame.bit_depth > 8 {
            (u16::from_le_bytes([plane[idx], plane[idx + 1]]) >> shift) as u32
        } else {
            plane[idx] as u32
        }
    };

    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        let cy = fy(y, chroma_h) >> 16;
        for x in 0..w {
            let cx = fx(x, chroma_w) >> 16;
            let yy = sample(&frame.planes[0], frame.strides[0], x, y) as f32;
            let cb = sample(&frame.planes[1], frame.strides[1], cx, cy) as f32;
            let cr = sample(&frame.planes[2], frame.strides[2], cx, cy) as f32;

            // https://www.itu.int/rec/T-REC-BT.601 （业界通行的 YCbCr→RGB 转换矩阵）
            let u = cb - 128.0;
            let v = cr - 128.0;
            let r = yy + 1.402 * v;
            let g = yy - 0.344136 * u - 0.714136 * v;
            let b = yy + 1.772 * u;

            // 色度最近邻采样；双线性插值属 M2 平滑优化（OPTIMIZATION：省两次
            // 平面采样与两次乘加——单采样在 320x240@24 无可观测差异）。
            let base = (y * w + x) * 4;
            rgba[base] = clamp_u8(r);
            rgba[base + 1] = clamp_u8(g);
            rgba[base + 2] = clamp_u8(b);
            rgba[base + 3] = 255;
        }
    }

    DecodedVideoFrame {
        pts_ms: frame.pts.map(|p| p.max(0) as u64).unwrap_or(0),
        rgba,
        width: frame.width,
        height: frame.height,
    }
}

fn clamp_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// 供测试与 M1b 校验的辅助——帧的 luma 摘要（RGBA 面的均值）。
#[cfg(test)]
pub(crate) fn rgba_mean(rgba: &[u8]) -> f64 {
    let pixels = rgba.as_chunks::<4>().0;
    let mut sum = 0u64;
    for px in pixels {
        sum += u64::from(px[0]) + u64::from(px[1]) + u64::from(px[2]);
    }
    sum as f64 / (pixels.len() as f64 * 3.0)
}

/// 测试 fixture 路径（workspace 相对）；非测试编译不可达。
#[cfg(test)]
pub(crate) fn fixture_path(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // workspace root
    p.push("tests/fixtures/media");
    p.push(name);
    p
}
