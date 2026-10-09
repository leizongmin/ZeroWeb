//! MSE（MediaSource Extensions）流式源：增长缓冲 → symphonia isomp4 流式 demux
//! → H.264 解码（`decode-h264` feature）。
//!
//! t8n 切片：`appendBuffer` 面的字节落 [`MseFeed`] 共享缓冲（`Arc<Mutex>` 增长
//! 面——shim 桥 append 与解码 tick 跨线程共享），[`MseVideoFeed`] 惰性 probe
//! （ftyp+moov 可解析即建 reader），`next_frame` 在流写入边缘区分「等待更多
//! 数据」与「真 EOF」（`ended` 旗标——`endOfStream()` 后边缘才是流末）。
//! fragmented MP4（moof+mdat 段序）由 symphonia isomp4 原生承担（0.6 demuxer
//! 流式迭代 mvex/trex/sidx），本模块不自行解析 box 语义。
//!
//! https://www.w3.org/TR/media-source-2/
//! https://html.spec.whatwg.org/multipage/media.html#mediasource-urls

use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::video::well_known::CODEC_ID_H264;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::TimeBase;

use crate::decode::{ColorSpace, DecodeError, DecodedVideoFrame};

/// append 与解码共享的缓冲（单一写入面：append 互斥；解码只读）。
struct FeedShared {
    data: Vec<u8>,
    /// `endOfStream()` 语义：置位后流写入边缘才是真 EOF。
    ended: bool,
}

/// append 侧句柄（shim 桥持有多份 clone——registry 面 append/end 共享同一缓冲）。
#[derive(Clone)]
pub struct MseFeedHandle {
    shared: Arc<Mutex<FeedShared>>,
}

impl MseFeedHandle {
    /// 新建空缓冲句柄（registry `mse_create` 面：先持句柄，play 懒建解码器挂接）。
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(FeedShared {
                data: Vec::new(),
                ended: false,
            })),
        }
    }

    /// 追加字节（appendBuffer 面；bytes 须整段拷入——调用方缓冲随宿主 op 释放）。
    /// `end()` 后再 append = 流末续喂（spec append buffer 算法步 6 的 feed 侧——
    /// readyState 回 'open'，写入边缘不再是 EOF）→ 清除 ended 闩锁；已呈 Ended 的
    /// player 复活归既有 play/reset 链（单向流模型），此处只保写入口不闩死。
    pub fn append(&self, bytes: &[u8]) {
        let mut sh = self.shared.lock().unwrap();
        sh.ended = false;
        sh.data.extend_from_slice(bytes);
    }

    /// endOfStream()：置流末旗标（之后写入边缘 = 真 EOF）。
    pub fn end(&self) {
        self.shared.lock().unwrap().ended = true;
    }

    /// 当前缓冲字节长度。
    pub fn byte_len(&self) -> usize {
        self.shared.lock().unwrap().data.len()
    }

    /// 流末旗标是否已置。
    pub fn is_ended(&self) -> bool {
        self.shared.lock().unwrap().ended
    }
}

impl Default for MseFeedHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// symphonia `MediaSource` 视图：独立读游标 + 尾缘 clean-EOF（读越已写边缘返
/// Ok(0)——demuxer 视作流末；`ended` 旗标在 feed 层消解双义）。
struct FeedSource {
    shared: Arc<Mutex<FeedShared>>,
    pos: usize,
}

impl Read for FeedSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let sh = self.shared.lock().unwrap();
        if self.pos >= sh.data.len() || buf.is_empty() {
            return Ok(0);
        }
        let n = buf.len().min(sh.data.len() - self.pos);
        buf[..n].copy_from_slice(&sh.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl Seek for FeedSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let sh = self.shared.lock().unwrap();
        let len = sh.data.len() as i128;
        let target = match pos {
            SeekFrom::Start(o) => o as i128,
            SeekFrom::End(o) => len + i128::from(o),
            SeekFrom::Current(o) => self.pos as i128 + i128::from(o),
        };
        // 越界 seek 不报错（读时 clean-EOF 兜底）——负值 clamp 0。
        self.pos = target.clamp(0, len) as usize;
        Ok(self.pos as u64)
    }
}

impl symphonia::core::io::MediaSource for FeedSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.shared.lock().unwrap().data.len() as u64)
    }
}

/// MSE 流式视频轨：共享缓冲 + 惰性 symphonia reader + openh264 H.264 解码。
///
/// 解码形态镜像 [`crate::mp4_h264::Mp4H264Decoder`]（avcC→Annex-B、长度前缀
/// NALU 转换、B 帧前瞻 pending 槽），差异仅在源：静态字节 → 共享增长缓冲。
pub struct MseVideoFeed {
    handle: MseFeedHandle,
    /// probe 成功后的 reader（`None` = 初始段未到齐，下次 next_frame 重试）。
    reader: Option<Box<dyn FormatReader>>,
    video_track_id: u32,
    time_base: TimeBase,
    /// avcC 的 SPS/PPS Annex-B 前缀（首包前注入一次）。
    parameter_set_prefix: Vec<u8>,
    decoder: openh264::decoder::Decoder,
    pending: Option<DecodedVideoFrame>,
    /// 真流末（仅 `ended` 且 demux 耗尽时置位——next_frame 快速短路用）。
    eof: bool,
    /// demux 已读到写入边缘（clean-EOF/UnexpectedEof 观测）——`is_exhausted`
    /// 按它 + 当下 `ended` 实时判定（end() 可发生在最后一次读边缘之后）。
    at_edge: bool,
    ps_injected: bool,
    /// probe 成功时轨声明的时长（毫秒；fragmented 常为 0/None——registry 面
    /// 以显式 `mediaSource.duration` 优先）。
    container_duration_ms: Option<u64>,
    /// probe（或最近一次 reader 重建）时的缓冲长度——symphonia 在建 reader 时
    /// 固化 total_len，之后 append 的增长它看不到；clean-EOF 且缓冲已增长时
    /// 据此判断需要重建 reader。
    len_at_probe: usize,
    /// 已交付帧的最大 pts（毫秒）——重建后跳过重放包的基准。
    last_delivered_pts_ms: Option<u64>,
    /// 重建后的跳过相：丢弃 pts ≤ [`Self::last_delivered_pts_ms`] 的重放包。
    skip_until_new: bool,
}

impl MseVideoFeed {
    /// 新建空 feed（probe 在首包数据 append 后的首次消费时尝试）。
    pub fn new() -> (Self, MseFeedHandle) {
        let shared = Arc::new(Mutex::new(FeedShared {
            data: Vec::new(),
            ended: false,
        }));
        let handle = MseFeedHandle {
            shared: Arc::clone(&shared),
        };
        (
            Self {
                handle: MseFeedHandle {
                    shared: Arc::clone(&shared),
                },
                reader: None,
                video_track_id: 0,
                // probe 前占位（1ms/tick）；probe 成功即被轨 time_base 覆盖。
                time_base: TimeBase::try_new(1, 1_000).expect("1/1000 non-zero"),
                parameter_set_prefix: Vec::new(),
                decoder: openh264::decoder::Decoder::new().expect("openh264 decoder"),
                pending: None,
                eof: false,
                at_edge: false,
                ps_injected: false,
                container_duration_ms: None,
                len_at_probe: 0,
                last_delivered_pts_ms: None,
                skip_until_new: false,
            },
            handle,
        )
    }

    /// 从既有 append 句柄构造（registry MSE 面：`mse_create` 先建句柄，
    /// appendBuffer 跨线程写入，play 懒建解码器时经此挂接）。
    pub fn from_handle(handle: MseFeedHandle) -> Self {
        Self {
            handle,
            reader: None,
            video_track_id: 0,
            time_base: TimeBase::try_new(1, 1_000).expect("1/1000 non-zero"),
            parameter_set_prefix: Vec::new(),
            decoder: openh264::decoder::Decoder::new().expect("openh264 decoder"),
            pending: None,
            eof: false,
            at_edge: false,
            ps_injected: false,
            container_duration_ms: None,
            len_at_probe: 0,
            last_delivered_pts_ms: None,
            skip_until_new: false,
        }
    }

    /// 尝试 probe（ftyp+moov 可解析即建 reader）；数据不足 → `Ok(false)` 留待重试。
    fn try_probe(&mut self) -> Result<bool, DecodeError> {
        if self.reader.is_some() {
            return Ok(true);
        }
        let src = FeedSource {
            shared: Arc::clone(&self.handle.shared),
            pos: 0,
        };
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("mp4");
        // 数据不足时 probe 读到 clean EOF → IoError（UnexpectedEof/EndOfFile）——
        // 留待下次 append 重试，不作错误上抛。
        let probed = match symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        ) {
            Ok(p) => p,
            // 部分数据下 probe 失败呈多形态（IoError 边缘 / Unsupported(missing
            // moov) 等）——统一按「init 段未到齐」重试；feed ended 仍失败才作
            // 硬错误（appendBuffer 输入受页面控制，语义简化如实标注）。
            Err(e) => {
                if self.handle.is_ended() {
                    return Err(DecodeError::Container(e.to_string()));
                }
                return Ok(false);
            }
        };
        for track in probed.tracks() {
            if let Some(CodecParameters::Video(v)) = &track.codec_params
                && v.codec == CODEC_ID_H264
            {
                self.video_track_id = track.id;
                if let Some(tb) = track.time_base {
                    self.time_base = tb;
                }
                // avcC：fragmented 轨的 stsd 同样携带（init segment 必备面）。
                if let Some(ed) = v.extra_data.first() {
                    self.parameter_set_prefix = crate::mp4_h264::avcc_record_to_annex_b(&ed.data);
                }
                if let (Some(tb), Some(dur)) = (track.time_base, track.duration) {
                    // t8n 返修 N6：容器可控值换算按 u64 饱和（`as` 低位回绕会让
                    // 构造性极端时长在 duration/skip_until_new 比较中失真）。
                    let ms128 = dur.get() as u128 * 1_000 * u128::from(tb.numer.get()) / u128::from(tb.denom.get());
                    self.container_duration_ms = Some(u64::try_from(ms128).unwrap_or(u64::MAX));
                }
                break;
            }
        }
        if self.video_track_id == 0 {
            return Err(DecodeError::NoVideoTrack);
        }
        self.reader = Some(probed);
        self.len_at_probe = self.handle.byte_len();
        Ok(true)
    }

    /// 解码并返回下一帧；等待更多数据或流末返回 `Ok(None)`（区分经
    /// [`Self::is_exhausted`]——`ended` 且耗尽才算真流末）。
    pub fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
        }
        if self.eof {
            return Ok(None);
        }
        if !self.try_probe()? {
            return Ok(None);
        }
        loop {
            // 读边缘归一：clean-EOF 与 UnexpectedEof 同作「等待/重建」处理。
            let next = match self.reader.as_mut().expect("probed").next_packet() {
                Ok(x) => Ok(x),
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    Ok(None)
                }
                Err(e) => Err(e),
            };
            match next {
                Ok(Some(packet)) => {
                    if packet.track_id != self.video_track_id {
                        continue;
                    }
                    // 同 N6：pts 换算 u64 饱和（容器可控极端值不回绕）。
                    let pts128 = packet.pts.get() as u128 * 1_000 * u128::from(self.time_base.numer.get())
                        / u128::from(self.time_base.denom.get());
                    let pts_ms = u64::try_from(pts128).unwrap_or(u64::MAX);
                    // 重建后的跳过相：丢弃 pts ≤ 已交付的（stale total_len 下
                    // 重读的）包，不对解码器重放。
                    if self.skip_until_new && self.last_delivered_pts_ms.is_some_and(|last| pts_ms <= last) {
                        continue;
                    }
                    self.skip_until_new = false;
                    self.at_edge = false;
                    let mut annex_b = if !self.ps_injected && !self.parameter_set_prefix.is_empty() {
                        self.ps_injected = true;
                        self.parameter_set_prefix.clone()
                    } else {
                        Vec::new()
                    };
                    annex_b.extend(crate::mp4_h264::length_prefixed_to_annex_b(&packet.data));
                    match self.decoder.decode(&annex_b) {
                        Ok(Some(yuv)) => {
                            use openh264::formats::YUVSource;
                            let (w, h) = yuv.dimensions();
                            let frame = crate::decode::planes_to_rgba(
                                &[yuv.y().to_vec(), yuv.u().to_vec(), yuv.v().to_vec()],
                                &[yuv.strides().0, yuv.strides().1, yuv.strides().2],
                                w,
                                h,
                                8,
                                1,
                                1,
                                &ColorSpace::default(),
                                pts_ms,
                            );
                            self.last_delivered_pts_ms = Some(pts_ms);
                            return Ok(Some(frame));
                        }
                        Ok(None) => continue,
                        Err(e) => return Err(DecodeError::H264(e.to_string())),
                    }
                }
                Ok(None) => {
                    // 读边缘：`ended` 后为真流末；未 ended 时若缓冲已增长，
                    // 重建 reader 续读（symphonia 建 reader 时固化 total_len，
                    // 后续 append 对它不可见——fragmented 追加段的承载点）。
                    self.at_edge = true;
                    self.eof = self.handle.is_ended();
                    if self.eof {
                        return Ok(None);
                    }
                    if self.handle.byte_len() > self.len_at_probe {
                        self.reader = None;
                        self.video_track_id = 0;
                        self.ps_injected = false;
                        self.skip_until_new = self.last_delivered_pts_ms.is_some();
                        if self.try_probe()? {
                            continue;
                        }
                    }
                    return Ok(None);
                }
                Err(e) => return Err(DecodeError::Container(e.to_string())),
            }
        }
    }

    /// 真流末（`ended` 且 demux 耗尽）——player Ended 判定的 MSE 面。
    /// 实时按边缘观测 + 当下 `ended` 判定：`end()` 可发生在最后一次读边缘之后，
    /// 不依赖此后再来一次 next_frame 才翻真。
    pub fn is_exhausted(&self) -> bool {
        self.at_edge && self.handle.is_ended()
    }

    /// 一次性维度探针（shim settle 真值链）：读缓冲内 moov 的视频轨宽高，不建
    /// reader、不消费帧、不触碰播放面状态。moov 未到齐（progressive mdat-先布局
    /// 的 append 中途）→ `None`，由 shim 侧留待后续 append 重试。
    pub fn probe_dims_from(handle: MseFeedHandle) -> Option<(u32, u32)> {
        let src = FeedSource {
            shared: Arc::clone(&handle.shared),
            pos: 0,
        };
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("mp4");
        let probed = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
            .ok()?;
        for track in probed.tracks() {
            if let Some(CodecParameters::Video(v)) = &track.codec_params
                && v.codec == CODEC_ID_H264
                && let (Some(w), Some(h)) = (v.width, v.height)
            {
                return Some((u32::from(w), u32::from(h)));
            }
        }
        None
    }

    /// 容器声明时长（毫秒；fragmented 轨常缺——调用方以显式 duration 优先）。
    pub fn duration_ms(&self) -> Option<u64> {
        self.container_duration_ms
    }

    /// seek（毫秒）——流首重建 reader 前向解码至 ≥ target（与
    /// [`crate::mp4_h264::Mp4H264Decoder::seek_to_ms`] 同构；feed 缓冲可随机读）。
    pub fn seek_to_ms(&mut self, target_ms: u64) -> Result<(), DecodeError> {
        self.reader = None;
        self.video_track_id = 0;
        self.ps_injected = false;
        self.pending = None;
        self.eof = false;
        if !self.try_probe()? {
            return Err(DecodeError::Container("mse feed lost probe on seek".into()));
        }
        loop {
            match self.next_frame() {
                Ok(Some(frame)) => {
                    if frame.pts_ms >= target_ms {
                        self.pending = Some(frame);
                        return Ok(());
                    }
                }
                Ok(None) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// 流末冲刷（openh264 前瞻缓冲残余帧）。
    pub fn flush(&mut self) {
        let _ = self.decoder.flush_remaining();
    }

    /// 帧退回（R3936 播放器背压契约——与 mp4_h264 面同语义）。
    pub fn un_read(&mut self, frame: DecodedVideoFrame) {
        debug_assert!(self.pending.is_none(), "un_read on occupied pending slot");
        self.pending = Some(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bytes() -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("tests/fixtures/media/sample-mp4-h264.mp4");
        std::fs::read(p).expect("mp4 fixture present")
    }

    /// 顶层 box 游标：(fourcc, payload 区间)。size==0（box 延伸到文件尾）按文件尾处理。
    fn top_boxes(data: &[u8]) -> Vec<([u8; 4], usize, usize)> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i + 8 <= data.len() {
            let size = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
            let mut fourcc = [0u8; 4];
            fourcc.copy_from_slice(&data[i + 4..i + 8]);
            let end = if size == 0 { data.len() } else { i + size };
            if end > data.len() {
                break;
            }
            out.push((fourcc, i + 8, end));
            i = end;
        }
        out
    }

    /// box 内查子 box（仅容器层用——不解析全量语义）。
    fn find_box<'a>(data: &'a [u8], fourcc: &[u8; 4]) -> Option<(&'a [u8], usize, usize)> {
        for (f, s, e) in top_boxes(data) {
            if &f == fourcc {
                return Some((&data[s..e], s, e));
            }
        }
        None
    }

    /// 提取 progressive fixture 的 avcC record 与 mdat 内长度前缀 NALU 序列
    /// （仅 VCL：nal_type 1/5——SPS/PPS 由 avcC 面承担）。
    /// 提取 progressive fixture 的 avcC record 与视频轨 AU（access unit）样本：
    /// 双轨交错 mdat（avc1+mp4a）——按 stco/stsc/stsz 样本表定位视频 chunk，
    /// stss 标关键帧。AU 字节（长度前缀 NALU 序列）原样进 trun sample。
    fn fixture_material() -> (Vec<u8>, Vec<(Vec<u8>, bool)>) {
        let data = fixture_bytes();
        let (moov, _, _) = find_box(&data, b"moov").expect("moov");
        let (trak, _, _) = find_box(moov, b"trak").expect("trak");
        let (mdia, _, _) = find_box(trak, b"mdia").expect("mdia");
        let (minf, _, _) = find_box(mdia, b"minf").expect("minf");
        let (stbl, _, _) = find_box(minf, b"stbl").expect("stbl");
        let (stsd, _, _) = find_box(stbl, b"stsd").expect("stsd");
        // stsd：4B version/flags + 4B entry_count，随后 sample entry。
        let entry = &stsd[8..];
        let entry_len = u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]) as usize;
        let avc1 = &entry[8..entry_len]; // 跳过 size+fourcc，进 entry body
        // VisualSampleEntry 固定段 78B（reserved/data_ref/pre/w/h/...）后为子 box。
        let children = &avc1[78..];
        let (avcc, _, _) = find_box(children, b"avcC").expect("avcC");
        let avcc = avcc.to_vec();
        let u32_at = |buf: &[u8], off: usize| u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        // stco：ver/flags(4)+count(4)+offsets(4×n)。
        let (stco, _, _) = find_box(stbl, b"stco").expect("stco");
        let chunk_count = u32_at(stco, 4) as usize;
        let chunk_offsets: Vec<usize> = (0..chunk_count).map(|k| u32_at(stco, 8 + 4 * k) as usize).collect();
        // stsc：ver/flags(4)+count(4)+entries 12B（first_chunk,spc,sdi）。
        let (stsc, _, _) = find_box(stbl, b"stsc").expect("stsc");
        let spc = u32_at(stsc, 8 + 4) as usize; // 首条 entry 的 samples_per_chunk
        // stsz：ver/flags(4)+sample_size(4)+sample_count(4)+sizes。
        let (stsz, _, _) = find_box(stbl, b"stsz").expect("stsz");
        let sample_count = u32_at(stsz, 8) as usize;
        let sizes: Vec<usize> = (0..sample_count).map(|k| u32_at(stsz, 12 + 4 * k) as usize).collect();
        // stss：ver/flags(4)+count(4)+sample_numbers（1-based）。
        let (stss, _, _) = find_box(stbl, b"stss").expect("stss");
        let key_count = u32_at(stss, 4) as usize;
        let keys: Vec<usize> = (0..key_count).map(|k| u32_at(stss, 8 + 4 * k) as usize).collect();
        // chunk → 样本展开（stsc 单 entry = 全程 spc 恒定）。
        let mut samples = Vec::new();
        let mut idx = 0usize;
        for off in chunk_offsets {
            for j in 0..spc {
                if idx >= sample_count {
                    break;
                }
                let sample_off = off + sizes[idx..idx + j].iter().sum::<usize>();
                let is_key = keys.contains(&(idx + 1));
                samples.push((data[sample_off..sample_off + sizes[idx + j]].to_vec(), is_key));
                idx += 1;
            }
        }
        assert!(!samples.is_empty(), "fixture video AUs extracted");
        (avcc, samples)
    }

    fn be32(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    /// ISO/IEC 14496-12 unity matrix（6×16.16 + 2×2.30）。
    const UNITY_MATRIX: [u32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000];

    fn box_header(fourcc: &[u8; 4], payload_len: usize) -> Vec<u8> {
        let mut b = be32((payload_len + 8) as u32).to_vec();
        b.extend_from_slice(fourcc);
        b
    }

    fn full_box(fourcc: &[u8; 4], version: u8, flags: u32, payload: Vec<u8>) -> Vec<u8> {
        let mut b = box_header(fourcc, payload.len() + 4);
        b.extend_from_slice(&version.to_be_bytes());
        b.extend_from_slice(&flags.to_be_bytes()[1..]); // version(1B)+flags(3B) = 4B
        b.extend_from_slice(&payload);
        b
    }

    /// 手造最小 fragmented fMP4（t8n 判据① fragmented 面 fixture）：
    /// ftyp + moov(mvhd/trak/mdia/minf/stbl/stsd(avc1+avcC)/mvex(trex)) +
    /// moof(mfhd/traf(tfhd/tfdt/trun)) + mdat——样本表提取的 AU 逐 sample
    /// （40ms/帧；AU 字节 = 长度前缀 NALU 序列，与 MP4 sample 布局同构）。
    fn build_fragmented(avcc: &[u8], samples: &[(Vec<u8>, bool)]) -> (Vec<u8>, Vec<u8>) {
        let timescale: u32 = 1000;
        let frame_dur: u32 = 40;
        // stsd avc1 sample entry：6B reserved + 2B data_ref_index + 16B 预留 +
        // 2B width + 2B height + ... + avcC 子 box（最小可解析面）。
        let mut avc1_body = Vec::new();
        avc1_body.extend_from_slice(&[0u8; 6]); // reserved
        avc1_body.extend_from_slice(&be32(1)[2..]); // data_reference_index = 1
        avc1_body.extend_from_slice(&[0u8; 16]); // pre_defined/reserved
        avc1_body.extend_from_slice(&be32(640)[2..]); // width
        avc1_body.extend_from_slice(&be32(360)[2..]); // height
        avc1_body.extend_from_slice(&[0u8; 12]); // hres/vres + reserved（VisualSampleEntry 固定段至此 78B）
        avc1_body.extend_from_slice(&be32(1)[2..]); // frame_count
        avc1_body.extend_from_slice(&[0u8; 32]); // compressorname
        avc1_body.extend_from_slice(&be32(0x0018)[2..]); // depth
        avc1_body.extend_from_slice(&be32(0xffff)[..2]); // pre_defined = -1
        avc1_body.extend_from_slice(&box_header(b"avcC", avcc.len()));
        avc1_body.extend_from_slice(avcc);
        let avc1 = box_header(b"avc1", avc1_body.len())
            .into_iter()
            .chain(avc1_body)
            .collect::<Vec<u8>>();
        let mut stsd_payload = be32(1).to_vec(); // entry_count = 1（avc1）
        stsd_payload.extend_from_slice(&avc1);
        let stsd = full_box(b"stsd", 0, 0, stsd_payload);
        // 空表族（duration 0 声明面——demuxer 对 fragmented 轨走 mvex 语义）。
        let stts = full_box(b"stts", 0, 0, be32(0).to_vec());
        let stsc = full_box(b"stsc", 0, 0, be32(0).to_vec());
        let mut stsz_payload = be32(0).to_vec();
        stsz_payload.extend_from_slice(&be32(0));
        let stsz = full_box(b"stsz", 0, 0, stsz_payload);
        let stco = full_box(b"stco", 0, 0, be32(0).to_vec());
        let stbl_body = [stsd, stts, stsc, stsz, stco].concat();
        let stbl = box_header(b"stbl", stbl_body.len())
            .into_iter()
            .chain(stbl_body)
            .collect::<Vec<u8>>();
        let vmhd = full_box(b"vmhd", 0, 1, vec![0u8; 8]);
        let dref = full_box(b"dref", 0, 0, {
            let mut p = be32(1).to_vec();
            p.extend_from_slice(&full_box(b"url ", 0, 1, Vec::new()));
            p
        });
        let dinf = box_header(b"dinf", dref.len())
            .into_iter()
            .chain(dref)
            .collect::<Vec<u8>>();
        let minf_body = [vmhd, dinf, stbl].concat();
        let minf = box_header(b"minf", minf_body.len())
            .into_iter()
            .chain(minf_body)
            .collect::<Vec<u8>>();
        let hdlr = full_box(b"hdlr", 0, 0, {
            let mut p = be32(0).to_vec();
            p.extend_from_slice(b"vide");
            p.extend_from_slice(&[0u8; 12]);
            p.extend_from_slice(b"VideoHandler\0");
            p
        });
        let mdhd = full_box(b"mdhd", 0, 0, {
            let mut p = Vec::new();
            p.extend_from_slice(&be32(0)); // creation
            p.extend_from_slice(&be32(0)); // modification
            p.extend_from_slice(&be32(timescale));
            p.extend_from_slice(&be32(0)); // duration（fragmented：0）
            p.extend_from_slice(&be32(0x55C00000)); // lang und
            p
        });
        let mdia_body = [mdhd, hdlr, minf].concat();
        let mdia = box_header(b"mdia", mdia_body.len())
            .into_iter()
            .chain(mdia_body)
            .collect::<Vec<u8>>();
        let tkhd = full_box(b"tkhd", 0, 7, {
            let mut p = Vec::new();
            p.extend_from_slice(&be32(0)); // creation
            p.extend_from_slice(&be32(0)); // modification
            p.extend_from_slice(&be32(1)); // track_id
            p.extend_from_slice(&be32(0)); // reserved
            p.extend_from_slice(&be32(0)); // duration
            p.extend_from_slice(&[0u8; 8]); // reserved[2]
            p.extend_from_slice(&be32(0)[2..]); // layer
            p.extend_from_slice(&be32(0)[2..]); // alternate_group
            p.extend_from_slice(&be32(0)[2..]); // volume（视频轨 0）
            p.extend_from_slice(&be32(0)[2..]); // reserved
            p.extend_from_slice(&UNITY_MATRIX.iter().flat_map(|v| v.to_be_bytes()).collect::<Vec<u8>>()); // matrix
            p.extend_from_slice(&be32(640 << 16)); // width 16.16
            p.extend_from_slice(&be32(360 << 16)); // height 16.16
            p
        });
        let trak_body = [tkhd, mdia].concat();
        let trak = box_header(b"trak", trak_body.len())
            .into_iter()
            .chain(trak_body)
            .collect::<Vec<u8>>();
        let mvhd = full_box(b"mvhd", 0, 0, {
            let mut p = Vec::new();
            p.extend_from_slice(&be32(0)); // creation
            p.extend_from_slice(&be32(0)); // modification
            p.extend_from_slice(&be32(timescale));
            p.extend_from_slice(&be32(0)); // duration
            p.extend_from_slice(&be32(0x00010000)); // rate 16.16
            p.extend_from_slice(&be32(0x01000000)[..2]); // volume 8.8
            p.extend_from_slice(&[0u8; 2]); // reserved
            p.extend_from_slice(&[0u8; 8]); // reserved[2]
            p.extend_from_slice(&UNITY_MATRIX.iter().flat_map(|v| v.to_be_bytes()).collect::<Vec<u8>>()); // matrix
            p.extend_from_slice(&[0u8; 24]); // pre_defined[6]
            p.extend_from_slice(&be32(2)); // next_track_id
            p
        });
        let trex = full_box(b"trex", 0, 0, {
            let mut p = be32(1).to_vec(); // track_id
            p.extend_from_slice(&be32(1)); // default_sample_description_index
            p.extend_from_slice(&be32(frame_dur)); // default_sample_duration
            p.extend_from_slice(&be32(0)); // default_sample_size
            p.extend_from_slice(&be32(0x01010000)); // default_sample_flags（非关键帧）
            p
        });
        let mvex = box_header(b"mvex", trex.len())
            .into_iter()
            .chain(trex)
            .collect::<Vec<u8>>();
        let moov_body = [mvhd, trak, mvex].concat();
        let moov = box_header(b"moov", moov_body.len())
            .into_iter()
            .chain(moov_body)
            .collect::<Vec<u8>>();
        // ftyp payload：major_brand(4) + minor_version(4) + compatible(4) = 12B。
        let ftyp = box_header(b"ftyp", 12)
            .into_iter()
            .chain(b"iso5".to_vec())
            .chain(be32(0))
            .chain(b"iso5".to_vec())
            .collect::<Vec<u8>>();
        let init = [ftyp, moov].concat();

        // moof + mdat（单 fragment：全部 sample；关键帧旗标取自 stss 集）。
        let mut trun_payload = be32(samples.len() as u32).to_vec(); // sample_count
        trun_payload.extend_from_slice(&be32(0)); // data_offset 占位（mdat 定位后回填）
        for (_, is_key) in samples {
            let flags = if *is_key { 0x0200_0000 } else { 0x0101_0000 };
            trun_payload.extend_from_slice(&be32(frame_dur));
            trun_payload.extend_from_slice(&be32(0)); // size 占位（下方回填）
            trun_payload.extend_from_slice(&be32(flags));
        }
        let trun = full_box(b"trun", 0, 0x000701, trun_payload); // data-offset+dur+size+flags
        let tfhd = full_box(b"tfhd", 0, 0x020000, be32(1).to_vec()); // default-base-is-moof
        let tfdt = full_box(b"tfdt", 1, 0, be32(0).to_vec()); // v1：baseMediaDecodeTime=0
        let mfhd = full_box(b"mfhd", 0, 0, be32(1).to_vec()); // sequence_number=1
        let trun_len = trun.len();
        let traf_body = [tfhd, tfdt, trun].concat();
        let traf = box_header(b"traf", traf_body.len())
            .into_iter()
            .chain(traf_body)
            .collect::<Vec<u8>>();
        let moof_body = [mfhd, traf].concat();
        let moof = box_header(b"moof", moof_body.len())
            .into_iter()
            .chain(moof_body)
            .collect::<Vec<u8>>();
        let mut mdat_payload = Vec::new();
        for (au, _) in samples {
            mdat_payload.extend_from_slice(au);
        }
        let mut moofmdat = moof.clone();
        let mdat_start = moofmdat.len();
        moofmdat.extend_from_slice(&box_header(b"mdat", mdat_payload.len()));
        moofmdat.extend_from_slice(&mdat_payload);
        // 回填 trun data_offset（相对 base = moof 起点）与逐 sample size。
        // trun 头 = size(4)+fourcc(4)+version(1)+flags(3) = 12B，
        // payload 首字段 sample_count(4)，随后 data_offset(4)。
        let trun_data_start = moof.len() - trun_len + 12 + 4;
        moofmdat[trun_data_start..trun_data_start + 4].copy_from_slice(&be32((mdat_start + 8) as u32));
        let mut off = trun_data_start + 4; // 首个 sample entry（duration+size+flags 各 4B）
        for (au, _) in samples {
            moofmdat[off + 4..off + 8].copy_from_slice(&be32(au.len() as u32)); // size 字段
            off += 12;
        }
        (init, moofmdat)
    }

    /// 钉①（渐进式流喂）：同一 progressive mp4 分两块 append——probe 惰性建
    /// reader，帧解码与整文件 open 等价；未 end() 前 None ≠ 真流末。
    #[test]
    fn mse_feed_progressive_streaming_two_chunks() {
        let data = fixture_bytes();
        // 该 fixture box 序为 ftyp/free/mdat/moov（mdat 在 moov 前——stco 绝对
        // 偏移引用 mdat，分块必须保持文件序合法）。第一块 = 文件前半（mdat
        // 中段截断，moov 未到——probe 不成）。
        let cut = data.len() / 2;
        let (mut feed, handle) = MseVideoFeed::new();
        handle.append(&data[..cut]);
        // 未 append 余下数据：None 且**非**真流末。
        assert_eq!(feed.next_frame().unwrap(), None, "数据边缘等待面");
        assert!(!feed.is_exhausted(), "未 end() 的边缘不是流末");
        // 第二块：余下全部（mdat 尾 + moov——probe 成，stco 偏移在完整流内合法）。
        handle.append(&data[cut..]);
        let mut frames = 0;
        let mut last_pts = 0u64;
        while let Some(f) = feed.next_frame().unwrap() {
            frames += 1;
            assert!(f.pts_ms >= last_pts);
            last_pts = f.pts_ms;
        }
        assert_eq!(frames, fixture_material().1.len(), "整流帧数与视频轨样本表等价");
        assert!(!feed.is_exhausted(), "未 end() 前 demux 耗尽仍非真流末");
        handle.end();
        assert_eq!(feed.next_frame().unwrap(), None);
        assert!(feed.is_exhausted(), "end() 后耗尽即真流末");
    }

    /// 钉②（fragmented 面）：手造 fMP4 init+media 段逐段 append——moof/mvex
    /// 语义经 symphonia 流式承担，帧序与 pts 单调。
    #[test]
    fn mse_feed_fragmented_fmp4_init_and_media_segments() {
        let (avcc, samples) = fixture_material();
        let (init, media) = build_fragmented(&avcc, &samples);
        let (mut feed, handle) = MseVideoFeed::new();
        handle.append(&init);
        // 仅 init 段：probe 可成（ftyp+moov），demux 等 moof。
        assert_eq!(feed.next_frame().unwrap(), None, "init 段无帧（等 media 段）");
        assert!(!feed.is_exhausted());
        handle.append(&media);
        let mut frames = 0;
        let mut last_pts = 0u64;
        while let Some(f) = feed.next_frame().unwrap() {
            frames += 1;
            assert!(f.pts_ms >= last_pts, "pts 单调（tfdt/trun 时序）");
            last_pts = f.pts_ms;
        }
        assert_eq!(frames, samples.len(), "逐 sample 解码帧数=N");
        handle.end();
        assert!(feed.is_exhausted(), "end() 后真流末");
    }

    /// 钉③（t8n 返修 F3/N4①）：end() 后续喂（append-after-endOfStream）清除 ended
    /// 闩锁——写入边缘回「等待面」，流末旗标须重新 end() 才再置（spec append
    /// buffer 算法步 6 的 feed 侧；shim 状态机回转由 engine part09 阶段⑤钉）。
    /// 已呈 Ended 的 player 复活归 play/reset 链，此处只保写入口不闩死。
    #[test]
    fn mse_feed_append_after_end_clears_eof_latch_t8n() {
        let (avcc, samples) = fixture_material();
        let (init, media) = build_fragmented(&avcc, &samples);
        let (mut feed, handle) = MseVideoFeed::new();
        handle.append(&init);
        handle.append(&media);
        handle.end();
        while feed.next_frame().unwrap().is_some() {}
        assert!(feed.is_exhausted(), "end() 后耗尽即真流末");
        // 流末续喂：闩锁清除 → 真流末撤销（同段重喂不解码，只验旗标面）。
        handle.append(&media);
        assert!(!handle.is_ended(), "append 清除 ended 闩锁");
        assert!(!feed.is_exhausted(), "续喂后写入边缘回等待面");
        // 重新 end()：闩锁再置，恢复真流末。
        handle.end();
        assert!(feed.is_exhausted(), "重新 end() 恢复真流末");
    }
}
