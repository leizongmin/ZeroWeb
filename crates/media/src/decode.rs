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
    /// AV1 位流解码失败（feature `decode-av1` 面，M3）。
    #[error("av1 decode error: {0}")]
    Av1(String),
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

/// 视频位流解码器路由（M3 多格式面——同一 demux 循环下的 codec 分派）。
enum VideoCodec {
    /// VP9（rusty_vp9 纯 Rust——路线 C 主面）。
    Vp9(Box<rusty_vp9::Vp9Decoder>),
    /// AV1（dav1d 绑定——D-RFC-2 批复面，feature `decode-av1`）。
    #[cfg(feature = "decode-av1")]
    Av1(Box<crate::av1_decode::Av1Decoder>),
}

impl VideoCodec {
    fn push(&mut self, data: &[u8], pts_ms: i64) -> Result<(), DecodeError> {
        match self {
            VideoCodec::Vp9(d) => d.push(data, Some(pts_ms)).map_err(DecodeError::from),
            #[cfg(feature = "decode-av1")]
            VideoCodec::Av1(d) => d.push(data, pts_ms),
        }
    }
    /// 取一帧（VP9 路径：push 后必有输出——无前瞻重排）；流缓冲末返回 None。
    /// 色彩描述 `color` 由持有方（VideoDecoder）传入——VP9 平面 → RGBA 的
    /// 转换依赖容器声明面。
    fn next_frame(&mut self, color: &ColorSpace) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        match self {
            VideoCodec::Vp9(d) => match d.next_frame() {
                Ok(f) => Ok(Some(to_rgba(f, color))),
                Err(rusty_vp9::Error::Eof | rusty_vp9::Error::Again) => Ok(None),
                Err(e) => Err(e.into()),
            },
            #[cfg(feature = "decode-av1")]
            VideoCodec::Av1(d) => d.next_frame(),
        }
    }
    /// 排空态取帧（R3936）：flush 后的残余排空——`Again`（隐藏/非展示帧，位流
    /// 已解码仅不产出）须**继续拉取**，只有 `Eof`（队列真空）才是真流末。与
    /// [`Self::next_frame`] 的差别仅在 Again 的语义折叠：排空态把 Again 视为
    /// 「内部跳过、继续」，非排空态视作「缓冲空、继续 demux」。
    fn drain_frame(&mut self, color: &ColorSpace) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        loop {
            match self {
                VideoCodec::Vp9(d) => match d.next_frame() {
                    Ok(f) => return Ok(Some(to_rgba(f, color))),
                    Err(rusty_vp9::Error::Eof) => return Ok(None),
                    Err(rusty_vp9::Error::Again) => continue, // 隐藏帧——继续排空
                    Err(e) => return Err(e.into()),
                },
                #[cfg(feature = "decode-av1")]
                VideoCodec::Av1(d) => match d.next_frame()? {
                    Some(f) => return Ok(Some(f)),
                    None => return Ok(None),
                },
            }
        }
    }
    /// 冲刷解码器内部缓冲（流末残余帧）。
    fn flush(&mut self, color: &ColorSpace) {
        match self {
            VideoCodec::Vp9(d) => {
                d.flush();
                // flush 后残余帧同样走颜色转换（调用方 next_frame 会再拉取——
                // 此处仅作 flush 语义）。
                let _ = color;
            }
            #[cfg(feature = "decode-av1")]
            VideoCodec::Av1(d) => d.flush(),
        }
    }
    /// 重建（seek 参考链作废——新解码器从 keyframe 重启）。
    fn reset(&mut self) -> Result<(), DecodeError> {
        match self {
            VideoCodec::Vp9(d) => {
                **d = rusty_vp9::Vp9Decoder::new();
                Ok(())
            }
            #[cfg(feature = "decode-av1")]
            VideoCodec::Av1(d) => {
                **d = crate::av1_decode::Av1Decoder::new()?;
                Ok(())
            }
        }
    }
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
    /// 位流解码器（VP9 rusty_vp9 / AV1 dav1d——M3 codec 路由）。
    codec: VideoCodec,
    /// 视频轨号（Matroska track number）。
    video_track: u64,
    /// 时间戳缩放（ns/tick；frame.timestamp 的单位为 tick）。
    timestamp_scale: u64,
    /// 流是否已到末尾（`next_frame` 返回 `Err(Eof)` 后置位）。
    eof: bool,
    /// demux 已耗尽、解码器残余排空中（R3936——排空完成才置 `eof`，防滞留帧）。
    draining: bool,
    /// seek 前向推进时暂存的「已解出但未消费」帧（`seek_to_ms` 命中 target 时
    /// 写入；下一次 `next_frame` 先弹出——spec precise-seek 不丢帧）。
    pending: Option<DecodedVideoFrame>,
    /// 色彩描述（TrackEntry Video.Colour，开流时解析一次——M2 色度精化）。
    color: ColorSpace,
}

impl VideoDecoder {
    /// 从 webm/Matroska 字节打开 VP9 视频解码器。
    ///
    /// 校验容器含 VP9 视频轨（`V_VP9`），否则 [`DecodeError::NoVideoTrack`]。
    pub fn open_webm_vp9(data: &[u8]) -> Result<Self, DecodeError> {
        let cursor = Cursor::new(data.to_vec());
        let demuxer = matroska_demuxer::MatroskaFile::open(cursor)?;
        let track = demuxer
            .tracks()
            .iter()
            .find(|t| t.track_type() == matroska_demuxer::TrackType::Video && t.codec_id() == "V_VP9")
            .ok_or(DecodeError::NoVideoTrack)?;
        let video_track = track.track_number().get();
        let color = track.video().map(ColorSpace::from_track).unwrap_or_default();
        Ok(Self {
            timestamp_scale: demuxer.info().timestamp_scale().get(),
            demuxer,
            codec: VideoCodec::Vp9(Box::new(rusty_vp9::Vp9Decoder::new())),
            video_track,
            eof: false,
            draining: false,
            pending: None,
            color,
        })
    }

    /// 从 webm/Matroska 字节打开视频解码器（codec 自路由——M3 多格式面）。
    ///
    /// 候选序：V_VP9（纯 Rust）→ V_AV1（dav1d，feature `decode-av1`）。
    /// 编解码位流解码失败按 [`DecodeError::NoVideoTrack`] 语义回落（调用方
    /// 占位渲染——不可解码 src 零回归契约）。
    pub fn open_webm(data: &[u8]) -> Result<Self, DecodeError> {
        let cursor = Cursor::new(data.to_vec());
        let demuxer = matroska_demuxer::MatroskaFile::open(cursor)?;
        let tracks = demuxer.tracks();
        // 首个视频轨（不分 codec）——同一容器混多视频轨非本面（webm 惯例单视频轨）。
        let track = tracks
            .iter()
            .find(|t| t.track_type() == matroska_demuxer::TrackType::Video)
            .ok_or(DecodeError::NoVideoTrack)?;
        let video_track = track.track_number().get();
        let color = track.video().map(ColorSpace::from_track).unwrap_or_default();
        let codec = match track.codec_id() {
            "V_VP9" => VideoCodec::Vp9(Box::new(rusty_vp9::Vp9Decoder::new())),
            #[cfg(feature = "decode-av1")]
            "V_AV1" => VideoCodec::Av1(Box::new(crate::av1_decode::Av1Decoder::new()?)),
            #[cfg(not(feature = "decode-av1"))]
            "V_AV1" => return Err(DecodeError::NoVideoTrack),
            _ => return Err(DecodeError::NoVideoTrack),
        };
        Ok(Self {
            timestamp_scale: demuxer.info().timestamp_scale().get(),
            demuxer,
            codec,
            video_track,
            eof: false,
            draining: false,
            pending: None,
            color,
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
        self.codec.reset()?;
        self.eof = false;
        self.draining = false;
        let mut block = matroska_demuxer::Frame::default();
        if matches!(self.demuxer.next_frame(&mut block), Ok(true))
            && block.track == self.video_track
            && block.is_keyframe == Some(true)
        {
            // keyframe 落点：解码此块并暂存（下一次 next_frame 消费——不丢帧）。
            let pts_ms = block.timestamp * self.timestamp_scale / 1_000_000;
            self.codec.push(&block.data, pts_ms as i64)?;
            let frame = self.codec.next_frame(&self.color)?;
            self.pending = frame;
            return Ok(());
        }

        // ② 回退：从流首 keyframe 前向解码到 target（spec precise-seek）。
        self.demuxer.seek(0)?;
        self.codec.reset()?;
        self.eof = false;
        self.draining = false;
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
    ///
    /// EOF 语义（R3936 修复）：demux 耗尽（`Ok(false)`）只置 `draining`——
    /// 解码器内部缓冲（superframe 队列 + hidden/alt-ref 帧的输出滞后）须继续
    /// 逐帧排空，队列真空时才置 `eof` 并报流末。旧形态 demux 末 flush 后仅
    /// pull 一帧即置 eof，后续调用提前返 `Ok(None)`，滞留帧（实测 test.webm
    /// 15 帧 ≈0.5s，pts 5.5~6.0s）永不产出——播放器在 position < duration 处
    /// 提前转 Ended（fixture-mounted runner 的 track-cues-enter-exit 复评
    /// 阻塞根因：cue@4-5s 永不出 → `t.done()` 永不）。
    pub fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
        }
        if self.eof {
            return Ok(None);
        }
        if self.draining {
            // demux 已尽：排空解码器残余（drain_frame 对隐藏帧 Again 继续拉，
            // 队列真空才报流末——见 next_frame 文档注释）。
            match self.codec.drain_frame(&self.color)? {
                Some(frame) => return Ok(Some(frame)),
                None => {
                    self.eof = true;
                    return Ok(None);
                }
            }
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
                    self.codec.push(&block.data, pts_ms as i64)?;
                    // push 后必有输出（VP9 无前瞻重排；AV1 面自身缓冲 reorder，
                    // 缓冲空时 next_frame 返 None → 继续推进 demux 喂下一块）。
                    match self.codec.next_frame(&self.color)? {
                        Some(frame) => return Ok(Some(frame)),
                        None => continue,
                    }
                }
                Ok(false) => {
                    // 容器末：进入排空态（flush 只做一次；残余帧经上方 draining
                    // 分支逐帧产出——滞留帧不再被 eof 提前吞掉）。
                    self.draining = true;
                    self.codec.flush(&self.color);
                    match self.codec.drain_frame(&self.color)? {
                        Some(frame) => return Ok(Some(frame)),
                        None => {
                            self.eof = true;
                            return Ok(None);
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    /// 帧退回（R3936——播放器帧调度背压）：把刚取出的未来帧塞回队首，下一次
    /// `next_frame` 原样返回（不丢帧、不重解码）。
    ///
    /// 背景：`VideoPlayer::present_pending` 逐帧拉取直至 `pts > position`；
    /// 旧形态把该未来帧**返回给调用方**（渲染消费后丢弃），其时间槽永久丢失
    /// ——解码器在 position < duration 处提前耗尽（fixture-mounted runner
    /// 实测 video 在 3.6s wall / position 3.57s 转 Ended，流长 6.0s）。
    pub fn un_read(&mut self, frame: DecodedVideoFrame) {
        debug_assert!(self.pending.is_none(), "un_read on occupied pending slot");
        self.pending = Some(frame);
    }
}

/// YUV→RGB 转换的色彩描述（WebM Colour 元素解析结果，M2 解码精化）。
///
/// https://www.matroska.org/technical/elements.html#colour-element
/// 未声明时的缺省与 ffmpeg/浏览器通行解释一致：BT.601 矩阵 + limited range
///（SD webm 系素材的主面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorSpace {
    /// YUV→RGB 转换矩阵。
    pub matrix: ColorMatrix,
    /// 采样值域（limited [16,235] ↔ full [0,255]）。
    pub full_range: bool,
}

/// YCbCr→RGB 矩阵选择（WebM MatrixCoefficients 的转换相关子集；
/// 未列出的（FCC/SMPTE240/YCoCg/BT2020 等）按 BT.709 近似并留扩展位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMatrix {
    /// Identity（GBR 顺序全范围——纯 RGB 流，通道直传不做 YUV 数学）。
    Identity,
    /// BT.709（HD 主面）。
    Bt709,
    /// BT.601（SD 缺省面）。
    #[default]
    Bt601,
}

impl ColorSpace {
    /// 从容器 TrackEntry 的 Colour 元素解析（缺省字段按通行解释回落）。
    ///
    /// 缺省依据：Matroska spec 各字段缺省值 + ffmpeg `webmdshow`/浏览器对
    /// unspecified 的处理——matrix 缺省 BT.601（MatrixCoefficients 缺省值 1 的
    /// 实际语义为 BT.709，但 SD 素材声明缺失时业界按 BT.601 解；声明面优先）。
    fn from_track(video: &matroska_demuxer::Video) -> Self {
        use matroska_demuxer::{MatrixCoefficients as Mc, Range};
        let Some(colour) = video.colour() else {
            return Self::default();
        };
        let matrix = match colour.matrix_coefficients() {
            Some(Mc::Identity) => ColorMatrix::Identity,
            Some(Mc::Bt709) => ColorMatrix::Bt709,
            Some(Mc::Smpte170 | Mc::Bt470bg) => ColorMatrix::Bt601,
            // None（元素缺省值 1 = BT.709 声明面）/Unknown/其余矩阵：limited-range
            // YUV 语义面下 BT.709 声明语义优先（spec 缺省）——本仓 fixture
            // （libvpx 编码）声明面 None → BT.709 路径。
            None | Some(Mc::Unknown) => ColorMatrix::Bt709,
            // BT.2020 等 HDR 面留 M3；此处不做 HDR tone mapping，按 BT.709 近似。
            _ => ColorMatrix::Bt709,
        };
        // range：声明 Broadcast → limited；Full → full；None/Defined 按 limited
        //（Matroska 缺省仅 full range 的 identity 面显式声明——YUV 面通行 limited）。
        let full_range = matches!(colour.range(), Some(Range::Full));
        Self { matrix, full_range }
    }
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self {
            matrix: ColorMatrix::Bt601,
            full_range: false,
        }
    }
}

/// `rusty_vp9::DecodedFrame`（YUV 平面）→ [`DecodedVideoFrame`]（RGBA）。
fn to_rgba(frame: rusty_vp9::DecodedFrame, color: &ColorSpace) -> DecodedVideoFrame {
    planes_to_rgba(
        &frame.planes,
        &frame.strides,
        frame.width as usize,
        frame.height as usize,
        frame.bit_depth as usize,
        frame.subsampling_x as usize,
        frame.subsampling_y as usize,
        color,
        frame.pts.map(|p| p.max(0) as u64).unwrap_or(0),
    )
}

/// YUV 平面 → RGBA 通用转换（VP9/AV1 共用——M3 codec 路由下的转换面统一）。
///
/// 支持 8/10/12 bit 与 4:2:0/4:2:2/4:4:4（逐平面定点转换）；当前 fixture 与
/// 上游 webm 系素材主面为 8bit 4:2:0。`bit_depth` 为存储位宽（8 → u8 样本 /
/// >8 → LE u16 高位对齐）；10/12 bit 经移位归一 8bit。`sx/sy` 为色度次采样
///
/// （0=无，1=半分辨率）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn planes_to_rgba(
    planes: &[Vec<u8>],
    strides: &[usize],
    w: usize,
    h: usize,
    bit_depth: usize,
    sx: usize,
    sy: usize,
    color: &ColorSpace,
    pts_ms: u64,
) -> DecodedVideoFrame {
    let shift = bit_depth - 8; // 10/12 bit → 8 bit 的高位移除
    let chroma_w = w.div_ceil(1 + sx);
    let chroma_h = h.div_ceil(1 + sy);

    // 色度定点索引（65536 = 1.0）：luma 像素 → 色度平面最近邻采样位置。
    // 比例 = luma 尺寸 → chroma 尺寸（420: ×0.5）；除数必须是 luma 维——
    // 旧实现误用 chroma 维，索引坍缩进 {0,1}（4:2:0 全行/列只采样前两个
    // 色度样点——M2 精化揭示，BT.601 全范围锚点掩盖了该缺陷）。
    let fx = |x: usize, lw: usize, cw: usize| x * cw * 65536 / lw;
    let fy = |y: usize, lh: usize, ch: usize| y * ch * 65536 / lh;

    let sample = |plane: &[u8], stride: usize, x: usize, y: usize| -> u32 {
        let idx = y * stride + x * if bit_depth > 8 { 2 } else { 1 };
        if bit_depth > 8 {
            (u16::from_le_bytes([plane[idx], plane[idx + 1]]) >> shift) as u32
        } else {
            plane[idx] as u32
        }
    };

    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        let cy = fy(y, h, chroma_h) >> 16;
        for x in 0..w {
            let cx = fx(x, w, chroma_w) >> 16;
            let yy = sample(&planes[0], strides[0], x, y) as f32;
            let cb = sample(&planes[1], strides[1], cx, cy) as f32;
            let cr = sample(&planes[2], strides[2], cx, cy) as f32;

            // 色彩面（M2 精化）：identity（GBR 全范围）通道直传；YUV 面按
            // 声明矩阵 + 值域转换。
            // https://www.itu.int/rec/T-REC-BT.601 （BT.601 系数）
            // https://www.itu.int/rec/T-REC-BT.709 （BT.709 系数）
            // https://www.itu.int/rec/T-REC-BT.1650 （limited→full 值域映射）
            let (r, g, b) = if color.matrix == ColorMatrix::Identity {
                // identity 矩阵 = 平面即 GBR 顺序全范围（VP9 语义，WebM gbr 惯例）：
                // 平面 0/1/2 实为 G/B/R 直传，不做 YUV 数学。
                (cr, yy, cb)
            } else {
                // limited [16,235]：luma 归一到 [0,255]（不移零点）；色度恒以
                // 128 为零点。full range：luma 直用，色度移零点。
                // ITU-R BT.601-7 §2.5 / BT.709-6 §2.5 标准形：
                //   limited: R = 1.164·(Y−16) + kR·(Cr−128)（kR/kB 按矩阵）
                //   full:    R = Y + kR·(Cr−128)
                let (y, u, v) = if color.full_range {
                    (yy, cb - 128.0, cr - 128.0)
                } else {
                    ((yy - 16.0) * 255.0 / 219.0, cb - 128.0, cr - 128.0)
                };
                match color.matrix {
                    // BT.709: kR=1.5748, kG=(0.2126,0.7152,0.0722), kB=1.8556。
                    ColorMatrix::Bt709 => (y + 1.5748 * v, y - 0.1873 * u - 0.4681 * v, y + 1.8556 * u),
                    // BT.601: kR=1.402, kG=(0.299,0.587,0.114), kB=1.772。
                    _ => (y + 1.402 * v, y - 0.344136 * u - 0.714136 * v, y + 1.772 * u),
                }
            };

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
        pts_ms,
        rgba,
        width: w as u32,
        height: h as u32,
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
    workspace_path(&format!("tests/fixtures/media/{name}"))
}

/// workspace 相对路径（非测试编译不可达）。
#[cfg(test)]
pub(crate) fn workspace_path(rel: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // workspace root
    p.push(rel);
    p
}
