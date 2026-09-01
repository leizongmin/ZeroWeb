//! AV1 视频解码（media-playback M3 / D-RFC-2 获批，feature `decode-av1`）。
//!
//! dav1d 绑定（安全 Rust 面，底层链接系统 libdav1d）+ Matroska V_AV1 轨的
//! low-overhead OBU 位流喂入。与 [`crate::decode`] 的 VP9 路径共用 demux 循环
//! 与 YUV→RGBA 转换（平面几何经 [`av_data::pixel`] 抽象对齐）。
//!
//! 路线 C 修订注记：VP9 面保持纯 Rust（rusty_vp9）；AV1 面按 D-RFC-2 批复
//! 引入 C 依赖 dav1d（参考解码器，Apache-2.0）——AV1 无成熟纯 Rust 解码器
//!（rav1d 未完工），批复允许的系统依赖面（libdav1d-dev）。

use crate::decode::{ColorMatrix, ColorSpace, DecodeError, DecodedVideoFrame};

/// 逐块推进的 AV1 位流解码器（push/pull——与 VP9 路径同形接口）。
pub struct Av1Decoder {
    decoder: dav1d::Decoder,
    /// send/get 交替泵的暂存帧队列（AV1 解码有 reorder 前瞻——send 后可能无
    /// 立即输出，flush 后逐帧取尽）。
    pending: Vec<DecodedVideoFrame>,
    /// 位流是否已终结（flush 后取尽 pending 置位）。
    flushed: bool,
}

impl Av1Decoder {
    pub fn new() -> Result<Self, DecodeError> {
        Ok(Self {
            decoder: dav1d::Decoder::new().map_err(av1_err)?,
            pending: Vec::new(),
            flushed: false,
        })
    }

    /// 喂入一个 Matroska block 的位流（V_AV1 轨数据 = low-overhead bitstream
    /// format 的 OBU 序列，无须再封装——Matroska spec BlockAdditions 约定）。
    /// pts_ms 透传 dav1d timestamp（其内部按 timebase 1ms 单位原样返回）。
    pub fn push(&mut self, data: &[u8], pts_ms: i64) -> Result<(), DecodeError> {
        self.decoder
            .send_data(data.to_vec(), None, Some(pts_ms), None)
            .map_err(av1_err)?;
        self.drain_pictures();
        Ok(())
    }

    /// 取下一帧（display 顺序——dav1d 内部已重排）；无输出返回 None。
    pub fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        if self.pending.is_empty() {
            self.drain_pictures();
        }
        if self.pending.is_empty() {
            return Ok(None);
        }
        // FIFO：drain 按序 push（dav1d get_picture 返回顺序即 display 顺序）。
        Ok(Some(self.pending.remove(0)))
    }

    /// 流末冲刷：取尽解码器内部缓冲的所有帧。
    pub fn flush(&mut self) {
        if self.flushed {
            return;
        }
        self.decoder.flush();
        self.drain_pictures();
        self.flushed = true;
    }

    /// 泵取当前可得的全部帧（send 后/flush 后调用）。
    fn drain_pictures(&mut self) {
        loop {
            match self.decoder.get_picture() {
                Ok(pic) => {
                    self.pending.push(av1_picture_to_rgba(&pic));
                }
                Err(dav1d::Error::Again) => break,
                Err(_) => break, // 坏帧容错：与 VP9 面一致不中断整流。
            }
        }
    }
}

fn av1_err(e: dav1d::Error) -> DecodeError {
    DecodeError::Av1(e.to_string())
}

/// dav1d Picture → RGBA（复用 decode.rs 的色彩面——平面数据经 AsRef<[u8]> 读）。
fn av1_picture_to_rgba(pic: &dav1d::Picture) -> DecodedVideoFrame {
    let w = pic.width();
    let h = pic.height();
    let pts_ms = pic.timestamp().map(|t| t.max(0) as u64).unwrap_or(0);
    // AV1 无 VP9 的显式 subsampling 布尔——PixelLayout 即几何。
    let (sx, sy) = match pic.pixel_layout() {
        dav1d::PixelLayout::I400 => (1, 1),
        dav1d::PixelLayout::I420 => (1, 1),
        dav1d::PixelLayout::I422 => (1, 0),
        dav1d::PixelLayout::I444 => (0, 0),
    };
    // 色彩面：dav1d 从位流 seq header 解析声明值（MatrixCoefficients/Range），
    // 与 ColorSpace 的 Matroska Colour 语义同表（未声明回落 BT.709 + limited
    // ——av1 主面）。
    let matrix = match pic.matrix_coefficients() {
        dav1d::pixel::MatrixCoefficients::Identity => ColorMatrix::Identity,
        dav1d::pixel::MatrixCoefficients::BT709 => ColorMatrix::Bt709,
        // BT.601 族：BT.470BG（625）与 ST170M（525/SMPTE170M 同值语义）。
        dav1d::pixel::MatrixCoefficients::BT470BG | dav1d::pixel::MatrixCoefficients::ST170M => ColorMatrix::Bt601,
        _ => ColorMatrix::Bt709,
    };
    let full_range = pic.color_range() == dav1d::pixel::YUVRange::Full;
    let color = ColorSpace { matrix, full_range };

    // 平面统一为 (bytes, stride) 视图（bit_depth>8 时 LE 16bit 样本）。
    let bit_depth = pic.bit_depth(); // 8 or 16（存储位宽）
    let planes: [Vec<u8>; 3] = [
        pic.plane(dav1d::PlanarImageComponent::Y).as_ref().to_vec(),
        pic.plane(dav1d::PlanarImageComponent::U).as_ref().to_vec(),
        pic.plane(dav1d::PlanarImageComponent::V).as_ref().to_vec(),
    ];
    let strides: [usize; 3] = [
        pic.stride(dav1d::PlanarImageComponent::Y) as usize,
        pic.stride(dav1d::PlanarImageComponent::U) as usize,
        pic.stride(dav1d::PlanarImageComponent::V) as usize,
    ];

    crate::decode::planes_to_rgba(
        &planes, &strides, w as usize, h as usize, bit_depth, sx, sy, &color, pts_ms,
    )
}
