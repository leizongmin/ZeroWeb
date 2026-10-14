//! CompressionStream / DecompressionStream host 实现（gzip/deflate/deflate-raw）。
//! R2986 初版为 buffer-then-process；compression-compat M2（2026-10-15）重写为
//! **流式状态机 + 手工帧校验**——WPT corpus（rev 31597693）证明两处硬缺口：
//! ① `decompression-correct-input` 等 3 案要求 write 后未 close 即可读出解压输出
//! （buffer-then-flush 下 read() 永挂）；② `decompression-corrupt-input` 面要求
//! trailer 级校验（flate2/miniz 读路径对截断 zlib trailer、trailing junk 静默成功）
//! 且错误面全量 TypeError（shim 侧抛）。
//!
//! **有状态流表**（text_encoding.rs TextDecoder 先例）：host 持 handle→slot 表
//!（容量封顶 FIFO 逐出），JS 侧构造时取 handle，update/finish 逐步驱动；逐出后
//! 超期使用按错误返回（压缩/解压状态无法透明重建，静默续跑会产出截断流——防数据
//! 丢失不做降级）。
//!
//! **解压走裸 deflate + 手工帧**（`Decompress::new(false)` 只吃 deflate body）：
//! - deflate（zlib wrapper）：CMF/FLG 头校验（CM=8、CINFO≤7、FCHECK %31==0、
//!   FDICT=1 → 错——preset dictionary 不支持）+ adler32 尾验（内联实现）+ 帧外
//!   剩余输入报错；
//! - gzip：头解析（ID/CM/FLG 保留位 + FEXTRA/FNAME/FCOMMENT/FHCRC 增量消费）+
//!   成员尾 CRC32（[`flate2::Crc`]，crc32fast 背书）+ ISIZE 校验 + 多成员串联
//!   （RFC 1952 concat 语义）；
//! - deflate-raw：裸 deflate，StreamEnd 后剩余输入报错，无 trailer。
//!
//! 所有解压错误 → err 串（shim 抛 TypeError——spec DecompressionStream 错误面，
//! corpus 以 `e instanceof TypeError` 判定）。
//!
//! 压缩走 flate2 write 编码器（Gz/Zlib/Deflate）增量 `write_all`，产出尾部增量
//! 回传（spec transform per-chunk enqueue；编码器内部缓冲的产出节奏不影响正确性）。
//! 字节经逗号分隔十进制串 wire（复用 crypto [`bytes_from_csv`]/[`bytes_to_csv`]），
//! 结果经 JSON wire `{"out":..,"err":..}`（err 非空 = 终态错误；out/err 可同时
//! 出现——trailing junk 面输出先于错误可见，spec enqueue-then-throw 语义）。
//! https://wicg.github.io/compression/  https://www.rfc-editor.org/rfc/rfc1950
//! https://www.rfc-editor.org/rfc/rfc1951  https://www.rfc-editor.org/rfc/rfc1952

use super::crypto::{bytes_from_csv, bytes_to_csv};
use super::json_str;
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::{Compression, Decompress, FlushDecompress, Status};
use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// 流表容量封顶——解压 slot 持 miniz inflate 状态（~数十 KB），64 条 ≈ MB 级；
/// 逐出后 update/finish 返错误（shim 抛 TypeError，流出错）——不做透明重建
///（压缩状态丢失会产出静默截断流，防数据丢失优先）。
const STREAM_TABLE_CAP: usize = 64;

/// 单流解压输出上限（防压缩炸弹病态输入把 headless 进程拖爆；spec 无上限语义，
/// 真实浏览器同面有实现级上限）。
const DECOMPRESS_OUTPUT_CAP: u64 = 1 << 30;

/// gzip 头累计上限（FEXTRA/FNAME/FCOMMENT 合法头远小于此；超限按流错误拒绝）。
const GZ_HEADER_CAP: usize = 1 << 16;

/// 单次 update 喂入裸 inflate 的输出预留步长。
const INFLATE_OUT_STEP: usize = 64 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fmt {
    Deflate,
    Gzip,
    DeflateRaw,
}

fn parse_format(format: &str) -> Option<Fmt> {
    match format {
        "deflate" => Some(Fmt::Deflate),
        "gzip" => Some(Fmt::Gzip),
        "deflate-raw" => Some(Fmt::DeflateRaw),
        _ => None,
    }
}

// ── 压缩侧：flate2 write 编码器增量封装 ──────────────────────────────────────

enum Encoder {
    Gz(GzEncoder<Vec<u8>>),
    Zlib(ZlibEncoder<Vec<u8>>),
    Raw(DeflateEncoder<Vec<u8>>),
}

struct CompressSlot {
    enc: Encoder,
    /// 已回传 JS 的产出偏移（产出增量 = inner[emitted..]）。
    emitted: usize,
}

#[derive(Default)]
struct CompressTable {
    slots: HashMap<u64, CompressSlot>,
    queue: VecDeque<u64>,
    next_id: u64,
}

fn compress_table() -> &'static Mutex<CompressTable> {
    static TABLE: OnceLock<Mutex<CompressTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(CompressTable::default()))
}

/// `new CompressionStream(format)` host 半步：返 handle 十进制串（不支持 format →
/// 空串；shim 侧构造时已按 WebIDL enum 校验并抛 TypeError，此处兜底）。
pub fn compress_new(format: &str) -> String {
    let Some(fmt) = parse_format(format) else {
        return String::new();
    };
    let enc = match fmt {
        Fmt::Deflate => Encoder::Zlib(ZlibEncoder::new(Vec::new(), Compression::default())),
        Fmt::Gzip => Encoder::Gz(GzEncoder::new(Vec::new(), Compression::default())),
        Fmt::DeflateRaw => Encoder::Raw(DeflateEncoder::new(Vec::new(), Compression::default())),
    };
    let mut table = compress_table().lock().unwrap_or_else(|e| e.into_inner());
    table.next_id += 1;
    let id = table.next_id;
    table.queue.push_back(id);
    while table.queue.len() > STREAM_TABLE_CAP {
        if let Some(oldest) = table.queue.pop_front() {
            table.slots.remove(&oldest);
        }
    }
    table.slots.insert(id, CompressSlot { enc, emitted: 0 });
    id.to_string()
}

/// `CompressionStream.transform` host 半步：喂入字节，增量压缩，返产出尾部
///（{"out":..,"err":..}）。handle 缺失（逐出）→ 错误（见 [`STREAM_TABLE_CAP`]）。
pub fn compress_update(handle: u64, bytes_csv: &str) -> String {
    let data = bytes_from_csv(bytes_csv);
    let mut table = compress_table().lock().unwrap_or_else(|e| e.into_inner());
    let Some(slot) = table.slots.get_mut(&handle) else {
        return wire(&[], "compression state evicted");
    };
    // Vec sink 的 write_all 实际不可失败；io::Error 仍按错误 wire 上抛（不吞）。
    let write_err = match &mut slot.enc {
        Encoder::Gz(enc) => enc.write_all(&data).err(),
        Encoder::Zlib(enc) => enc.write_all(&data).err(),
        Encoder::Raw(enc) => enc.write_all(&data).err(),
    };
    if let Some(err) = write_err {
        table.slots.remove(&handle);
        return wire(&[], &format!("compression failed: {err}"));
    }
    let (total, tail) = match &slot.enc {
        Encoder::Gz(enc) => (enc.get_ref().len(), enc.get_ref()[slot.emitted..].to_vec()),
        Encoder::Zlib(enc) => (enc.get_ref().len(), enc.get_ref()[slot.emitted..].to_vec()),
        Encoder::Raw(enc) => (enc.get_ref().len(), enc.get_ref()[slot.emitted..].to_vec()),
    };
    slot.emitted = total;
    wire(&tail, "")
}

/// `CompressionStream.flush` host 半步：收尾帧（gzip trailer/zlib adler），返剩余
/// 产出并释放 slot。
pub fn compress_finish(handle: u64) -> String {
    let mut table = compress_table().lock().unwrap_or_else(|e| e.into_inner());
    let Some(slot) = table.slots.remove(&handle) else {
        return wire(&[], "compression state evicted");
    };
    let emitted = slot.emitted;
    let finished = match slot.enc {
        Encoder::Gz(enc) => enc.finish(),
        Encoder::Zlib(enc) => enc.finish(),
        Encoder::Raw(enc) => enc.finish(),
    };
    match finished {
        Ok(buf) => wire(&buf[emitted..], ""),
        Err(err) => wire(&[], &format!("compression failed: {err}")),
    }
}

// ── 解压侧：裸 inflate core + 三格式手工帧状态机 ─────────────────────────────

/// adler32（RFC 1950 §8）滚动增量——zlib wrapper 尾验用（flate2 只公开 CRC32 的
/// [`flate2::Crc`]，adler 内联；per-byte 形态对 headless 流量足够——OPTIMIZATION：
/// 批量延迟取模可再快一个量级，语义不变，待有实测需要再做）。
struct Adler {
    a: u32,
    b: u32,
}

impl Adler {
    fn new() -> Self {
        Adler { a: 1, b: 0 }
    }
    fn update(&mut self, data: &[u8]) {
        const MOD: u32 = 65521;
        for &byte in data {
            self.a = (self.a + byte as u32) % MOD;
            self.b = (self.b + self.a) % MOD;
        }
    }
    fn sum(&self) -> u32 {
        (self.b << 16) | self.a
    }
}

/// gzip body 校验累计（CRC32 + ISIZE；flate2::Crc = crc32fast Hasher 背书）。
struct CrcState {
    crc: flate2::Crc,
    isize: u64,
}

impl CrcState {
    fn new() -> Self {
        CrcState {
            crc: flate2::Crc::new(),
            isize: 0,
        }
    }
    fn update(&mut self, data: &[u8]) {
        self.crc.update(data);
        self.isize += data.len() as u64;
    }
}

/// 裸 inflate core：吃输入、产出追加 `out`，返 (是否 StreamEnd, 消费字节数)。
/// 未到 StreamEnd 时消费恒等于 input.len()（输出容量由 reserve 步长保证推进）。
struct InflateCore {
    dz: Decompress,
}

impl InflateCore {
    fn new() -> Self {
        // 裸 deflate——zlib/gzip 帧校验由本模块手工承接（miniz wrapper 校验不完整）。
        InflateCore {
            dz: Decompress::new(false),
        }
    }

    fn feed(&mut self, input: &[u8], out: &mut Vec<u8>) -> Result<(bool, usize), String> {
        let mut consumed = 0usize;
        while consumed < input.len() {
            if self.dz.total_out() > DECOMPRESS_OUTPUT_CAP {
                return Err("compressed stream output exceeds limit".to_string());
            }
            if out.capacity() == out.len() {
                out.reserve(INFLATE_OUT_STEP);
            }
            let before_in = self.dz.total_in();
            let before_out = self.dz.total_out();
            match self.dz.decompress_vec(&input[consumed..], out, FlushDecompress::None) {
                Ok(Status::StreamEnd) => {
                    consumed += (self.dz.total_in() - before_in) as usize;
                    return Ok((true, consumed));
                }
                Ok(Status::Ok | Status::BufError) => {
                    consumed += (self.dz.total_in() - before_in) as usize;
                    let wrote = self.dz.total_out() - before_out;
                    if consumed < input.len() && wrote == 0 {
                        // 非空输入 + 有容量却不推进——防御性报错（不可静默续跑，
                        // 否则调用方循环无法前进）。
                        return Err("corrupt deflate stream: decoder stalled".to_string());
                    }
                }
                Err(err) => return Err(format!("corrupt deflate stream: {err}")),
            }
        }
        Ok((false, consumed))
    }
}

/// deflate（zlib wrapper）状态机：2 字节头 → body（adler 累计）→ 4 字节 adler 尾
/// → Done（Done 后余输入报错）。feed 每轮 `mem::replace` 取出 stage 再回写，
/// 规避「match 借 stage + 调 core」的借用冲突。
enum DeflateStage {
    Head([u8; 2], u8),
    Body(Adler),
    Adler(Adler, [u8; 4], u8),
    Done,
}

struct DeflateDec {
    core: InflateCore,
    stage: DeflateStage,
}

impl DeflateDec {
    fn new() -> Self {
        DeflateDec {
            core: InflateCore::new(),
            stage: DeflateStage::Head([0; 2], 0),
        }
    }

    fn feed(&mut self, mut data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
        while !data.is_empty() {
            match std::mem::replace(&mut self.stage, DeflateStage::Done) {
                DeflateStage::Done => {
                    return Err("unexpected data after end of compressed stream".to_string());
                }
                DeflateStage::Head(mut buf, mut got) => {
                    buf[got as usize] = data[0];
                    got += 1;
                    data = &data[1..];
                    if got == 2 {
                        check_zlib_header(buf[0], buf[1])?;
                        self.stage = DeflateStage::Body(Adler::new());
                    } else {
                        self.stage = DeflateStage::Head(buf, got);
                    }
                }
                DeflateStage::Body(mut adler) => {
                    let before_out = self.core.dz.total_out();
                    let (ended, used) = self.core.feed(data, out)?;
                    adler.update(&out[out.len() - (self.core.dz.total_out() - before_out) as usize..]);
                    data = &data[used..];
                    self.stage = if ended {
                        DeflateStage::Adler(adler, [0; 4], 0)
                    } else {
                        DeflateStage::Body(adler)
                    };
                }
                DeflateStage::Adler(adler, mut buf, mut got) => {
                    let take = (4 - got as usize).min(data.len());
                    buf[got as usize..got as usize + take].copy_from_slice(&data[..take]);
                    got += take as u8;
                    data = &data[take..];
                    if got == 4 {
                        if adler.sum() != u32::from_be_bytes(buf) {
                            return Err("deflate stream checksum mismatch".to_string());
                        }
                        self.stage = DeflateStage::Done;
                    } else {
                        self.stage = DeflateStage::Adler(adler, buf, got);
                    }
                }
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        match self.stage {
            DeflateStage::Done => Ok(()),
            _ => Err("unexpected end of deflate stream".to_string()),
        }
    }
}

/// gzip 状态机（RFC 1952）：头（逐字节增量解析——完成字节即最后头字节，跨界
/// 消费零尾差）→ body（CRC32+ISIZE 累计）→ 8 字节尾；流级 `None` = 成员边界
///（可接下个成员头或 clean finish）。
enum GzStage {
    Header(Vec<u8>),
    Body(CrcState),
    Trailer(CrcState, [u8; 8], u8),
}

struct GzipDec {
    core: InflateCore,
    stage: Option<GzStage>,
}

impl GzipDec {
    fn new() -> Self {
        GzipDec {
            core: InflateCore::new(),
            stage: Some(GzStage::Header(Vec::new())),
        }
    }

    fn feed(&mut self, mut data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
        while !data.is_empty() {
            match self.stage.take() {
                None => {
                    // 成员边界又进输入 → 按下一成员头解析（垃圾 → magic 校验失败报错，
                    // RFC 1952 concat 语义）。inflate core 同步重建——Decompress 在
                    // StreamEnd 后不得续用（会立即再报 StreamEnd，输出/消费全零）。
                    self.core = InflateCore::new();
                    self.stage = Some(GzStage::Header(Vec::new()));
                }
                Some(GzStage::Header(mut buf)) => {
                    buf.push(data[0]);
                    data = &data[1..];
                    if buf.len() > GZ_HEADER_CAP {
                        return Err("gzip header too large".to_string());
                    }
                    if gz_header_complete(&buf)?.is_some() {
                        self.stage = Some(GzStage::Body(CrcState::new()));
                    } else {
                        self.stage = Some(GzStage::Header(buf));
                    }
                }
                Some(GzStage::Body(mut crc)) => {
                    let before_out = self.core.dz.total_out();
                    let (ended, used) = self.core.feed(data, out)?;
                    crc.update(&out[out.len() - (self.core.dz.total_out() - before_out) as usize..]);
                    data = &data[used..];
                    self.stage = Some(if ended {
                        GzStage::Trailer(crc, [0; 8], 0)
                    } else {
                        GzStage::Body(crc)
                    });
                }
                Some(GzStage::Trailer(crc, mut buf, mut got)) => {
                    let take = (8 - got as usize).min(data.len());
                    buf[got as usize..got as usize + take].copy_from_slice(&data[..take]);
                    got += take as u8;
                    data = &data[take..];
                    if got == 8 {
                        if crc.crc.sum() != u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) {
                            return Err("gzip stream CRC32 mismatch".to_string());
                        }
                        if crc.isize != u64::from(u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])) {
                            return Err("gzip stream ISIZE mismatch".to_string());
                        }
                        self.stage = None; // 成员完；余输入接下一成员
                    } else {
                        self.stage = Some(GzStage::Trailer(crc, buf, got));
                    }
                }
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        match self.stage {
            None => Ok(()),
            _ => Err("unexpected end of gzip stream".to_string()),
        }
    }
}

/// deflate-raw 状态机：body → StreamEnd 即终态（无 trailer；终态后余输入报错）。
struct RawDec {
    core: InflateCore,
    done: bool,
}

impl RawDec {
    fn new() -> Self {
        RawDec {
            core: InflateCore::new(),
            done: false,
        }
    }

    fn feed(&mut self, mut data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
        while !data.is_empty() {
            if self.done {
                return Err("unexpected data after end of compressed stream".to_string());
            }
            let (ended, used) = self.core.feed(data, out)?;
            data = &data[used..];
            if ended {
                self.done = true;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        if self.done {
            Ok(())
        } else {
            Err("unexpected end of deflate-raw stream".to_string())
        }
    }
}

enum Decompressor {
    Deflate(DeflateDec),
    Gzip(GzipDec),
    Raw(RawDec),
}

impl Decompressor {
    fn new(fmt: Fmt) -> Self {
        match fmt {
            Fmt::Deflate => Decompressor::Deflate(DeflateDec::new()),
            Fmt::Gzip => Decompressor::Gzip(GzipDec::new()),
            Fmt::DeflateRaw => Decompressor::Raw(RawDec::new()),
        }
    }

    fn feed(&mut self, data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
        match self {
            Decompressor::Deflate(d) => d.feed(data, out),
            Decompressor::Gzip(d) => d.feed(data, out),
            Decompressor::Raw(d) => d.feed(data, out),
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        match self {
            Decompressor::Deflate(d) => d.finish(),
            Decompressor::Gzip(d) => d.finish(),
            Decompressor::Raw(d) => d.finish(),
        }
    }
}

struct DecompressSlot {
    decomp: Decompressor,
}

#[derive(Default)]
struct DecompressTable {
    slots: HashMap<u64, DecompressSlot>,
    queue: VecDeque<u64>,
    next_id: u64,
}

fn decompress_table() -> &'static Mutex<DecompressTable> {
    static TABLE: OnceLock<Mutex<DecompressTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(DecompressTable::default()))
}

/// `new DecompressionStream(format)` host 半步：同 [`compress_new`]。
pub fn decompress_new(format: &str) -> String {
    let Some(fmt) = parse_format(format) else {
        return String::new();
    };
    let mut table = decompress_table().lock().unwrap_or_else(|e| e.into_inner());
    table.next_id += 1;
    let id = table.next_id;
    table.queue.push_back(id);
    while table.queue.len() > STREAM_TABLE_CAP {
        if let Some(oldest) = table.queue.pop_front() {
            table.slots.remove(&oldest);
        }
    }
    table.slots.insert(
        id,
        DecompressSlot {
            decomp: Decompressor::new(fmt),
        },
    );
    id.to_string()
}

/// `DecompressionStream.transform` host 半步：喂入字节，增量解压，返产出
///（{"out":..,"err":..}；err 非空 = 终态错误，slot 已释放）。
pub fn decompress_update(handle: u64, bytes_csv: &str) -> String {
    let data = bytes_from_csv(bytes_csv);
    let mut table = decompress_table().lock().unwrap_or_else(|e| e.into_inner());
    let Some(slot) = table.slots.get_mut(&handle) else {
        return wire(&[], "decompression state evicted");
    };
    let mut out = Vec::new();
    match slot.decomp.feed(&data, &mut out) {
        Ok(()) => wire(&out, ""),
        Err(err) => {
            table.slots.remove(&handle);
            // 错误前已产出部分随 wire 可见（enqueue-then-throw 语义）。
            wire(&out, &err)
        }
    }
}

/// `DecompressionStream.flush` host 半步：流必须到达已验证终态，否则按截断错误；
/// 释放 slot。
pub fn decompress_finish(handle: u64) -> String {
    let mut table = decompress_table().lock().unwrap_or_else(|e| e.into_inner());
    let Some(slot) = table.slots.get_mut(&handle) else {
        return wire(&[], "decompression state evicted");
    };
    let result = slot.decomp.finish();
    table.slots.remove(&handle);
    match result {
        Ok(()) => wire(&[], ""),
        Err(err) => wire(&[], &err),
    }
}

/// JSON 结果 wire：`{"out":csv,"err":msg}`——err 空串 = 成功；out/err 可并存
///（trailing junk 面：错误前已产出部分仍可见，spec enqueue-then-throw）。
fn wire(out: &[u8], err: &str) -> String {
    format!("{{\"out\":{},\"err\":{}}}", json_str(&bytes_to_csv(out)), json_str(err))
}

/// zlib 头两字节校验（RFC 1950 §2.2）：CM=8、CINFO≤7、FCHECK (CMF*256+FLG)%31==0、
/// FDICT=1 → 拒绝（preset dictionary 不支持——corpus dictionary 面要求报错）。
fn check_zlib_header(cmf: u8, flg: u8) -> Result<(), String> {
    let cm = cmf & 0x0f;
    let cinfo = cmf >> 4;
    if cm != 8 {
        return Err(format!("deflate stream unsupported compression method {cm}"));
    }
    if cinfo > 7 {
        return Err(format!("deflate stream invalid window size {cinfo}"));
    }
    if (u16::from(cmf) * 256 + u16::from(flg)) % 31 != 0 {
        return Err("deflate stream header check bits invalid".to_string());
    }
    if flg & 0x20 != 0 {
        return Err("deflate stream preset dictionary not supported".to_string());
    }
    Ok(())
}

/// gzip 头完整判定（RFC 1952 §2.3）：返 Some(len) = 头长（含 FEXTRA/FNAME/
/// FCOMMENT/FHCRC），None = 需更多字节，Err = 头非法。调用方逐字节 append 后
/// 重入——完成时 len == buf.len()（完成字节即最后头字节）。
fn gz_header_complete(buf: &[u8]) -> Result<Option<usize>, String> {
    if buf.len() >= 1 && buf[0] != 0x1f {
        return Err("gzip stream bad magic".to_string());
    }
    if buf.len() >= 2 && buf[1] != 0x8b {
        return Err("gzip stream bad magic".to_string());
    }
    if buf.len() >= 3 && buf[2] != 8 {
        return Err(format!("gzip stream unsupported compression method {}", buf[2]));
    }
    if buf.len() >= 4 && buf[3] & 0xE0 != 0 {
        return Err("gzip stream reserved FLG bits set".to_string());
    }
    if buf.len() < 10 {
        return Ok(None);
    }
    let flg = buf[3];
    let mut p = 10usize;
    if flg & 0x04 != 0 {
        // FEXTRA：2 字节 LE 长度 + 数据
        if buf.len() < p + 2 {
            return Ok(None);
        }
        let xlen = u16::from_le_bytes([buf[p], buf[p + 1]]) as usize;
        p += 2;
        if buf.len() < p + xlen {
            return Ok(None);
        }
        p += xlen;
    }
    for flag in [0x08u8, 0x10u8] {
        // FNAME / FCOMMENT：零终止串
        if flg & flag != 0 {
            let Some(zero) = buf[p..].iter().position(|&b| b == 0) else {
                return Ok(None);
            };
            p += zero + 1;
        }
    }
    if flg & 0x02 != 0 {
        // FHCRC：头 CRC16（CRC32 低 16 位）
        if buf.len() < p + 2 {
            return Ok(None);
        }
        let mut crc = flate2::Crc::new();
        crc.update(&buf[..p]);
        let expect = u16::from_le_bytes([buf[p], buf[p + 1]]);
        if (crc.sum() & 0xffff) as u16 != expect {
            return Err("gzip stream header CRC16 mismatch".to_string());
        }
        p += 2;
    }
    Ok(Some(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 跨 feed/finish 的流式驱动 helper：按 `chunk_size` 切片喂入，聚合输出。
    fn drive(decomp: &mut Decompressor, input: &[u8], chunk_size: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        for piece in input.chunks(chunk_size.max(1)) {
            decomp.feed(piece, &mut out)?;
        }
        decomp.finish()?;
        Ok(out)
    }

    /// 取 wire JSON 的字符串字段（out/err 值均为无转义纯 ASCII，字符串切片足够，
    /// 不为此引 JSON 解析依赖）。
    fn field_of(wire_json: &str, key: &str) -> String {
        let pat = format!("\"{key}\":\"");
        let start = wire_json.find(&pat).map(|i| i + pat.len()).unwrap_or(0);
        let rest = &wire_json[start..];
        rest[..rest.find('"').unwrap_or(rest.len())].to_string()
    }

    /// 取 wire JSON 的 out 字段 csv。
    fn out_csv(wire_json: &str) -> String {
        field_of(wire_json, "out")
    }

    /// 取 wire JSON 的 err 字段。
    fn err_of(wire_json: &str) -> String {
        field_of(wire_json, "err")
    }

    const PAYLOAD: &[u8] = b"expected output";

    /// corpus decompression-input.js 的 deflate-raw 基准帧体（压缩
    /// 'expected output'；corpus deflateChunkValue / deflateRawChunkValue 同体）。
    const RAW_BODY: &[u8] = &[
        0x4b, 0xad, 0x28, 0x48, 0x4d, 0x2e, 0x49, 0x4d, 0x51, 0xc8, 0x2f, 0x2d, 0x29, 0x28, 0x2d, 0x01, 0x00,
    ];

    /// zlib 帧（corpus deflateChunkValue 逐字节：0x78 0x9c + body + adler32）。
    fn zlib_frame() -> Vec<u8> {
        let mut f = vec![0x78, 0x9c];
        f.extend_from_slice(RAW_BODY);
        f.extend_from_slice(&[0x30, 0xad, 0x06, 0x24]);
        f
    }

    /// gzip 帧 = 10B 头 + raw body + CRC32(payload) LE + ISIZE LE（trailer 程序内
    /// 现算——与手写向量等价且自洽）。
    fn gzip_frame() -> Vec<u8> {
        let mut crc = flate2::Crc::new();
        crc.update(PAYLOAD);
        let mut f = vec![0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 0x03];
        f.extend_from_slice(RAW_BODY);
        f.extend_from_slice(&crc.sum().to_le_bytes());
        f.extend_from_slice(&(PAYLOAD.len() as u32).to_le_bytes());
        f
    }

    /// 压缩 helper：handle 化增量 write → finish 聚合。
    fn compress(fmt: &str, input: &[u8], chunk_size: usize) -> Vec<u8> {
        let handle: u64 = compress_new(fmt).parse().unwrap();
        let mut out = Vec::new();
        for piece in input.chunks(chunk_size.max(1)) {
            out.extend(bytes_from_csv(&out_csv(&compress_update(handle, &bytes_to_csv(piece)))));
        }
        out.extend(bytes_from_csv(&out_csv(&compress_finish(handle))));
        out
    }

    #[test]
    fn test_roundtrip_streaming_all_formats() {
        // 大输入（超单步输出预留）按 1B/7B/大块三种切片全往返——流式一致性。
        let payload: Vec<u8> = (0..200_000usize).map(|i| (i % 251) as u8).collect();
        for fmt in ["gzip", "deflate", "deflate-raw"] {
            let compressed = compress(fmt, &payload, 7777);
            assert!(!compressed.is_empty(), "{fmt} 压缩产出非空");
            for chunk in [1usize, 7, 1 << 20] {
                let mut d = Decompressor::new(parse_format(fmt).unwrap());
                let restored = drive(&mut d, &compressed, chunk).unwrap_or_else(|e| panic!("{fmt} x{chunk}: {e}"));
                assert_eq!(restored, payload, "{fmt} chunk={chunk} 往返还原");
            }
        }
    }

    #[test]
    fn test_decompress_output_before_finish() {
        // corpus decompression-correct-input 语义：write 后未 finish 即可读出产出。
        for (fmt, frame) in [
            ("gzip", gzip_frame()),
            ("deflate", zlib_frame()),
            ("deflate-raw", RAW_BODY.to_vec()),
        ] {
            let mut d = Decompressor::new(parse_format(fmt).unwrap());
            let mut out = Vec::new();
            d.feed(&frame, &mut out).unwrap_or_else(|e| panic!("{fmt}: {e}"));
            assert_eq!(out, PAYLOAD, "{fmt} 未 finish 即产出全量");
        }
    }

    #[test]
    fn test_empty_streams_done_not_error() {
        // corpus decompression-empty-input：合法空流 → done 无输出无错误。
        let cases: &[(&str, Vec<u8>)] = &[
            // gzip 空成员（头 + stored 空 raw 块 + CRC=0 + ISIZE=0）
            (
                "gzip",
                vec![
                    0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 0x03, 0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            ),
            ("deflate", vec![0x78, 0x9c, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01]),
            ("deflate-raw", vec![0x01, 0x00, 0x00, 0xff, 0xff]),
        ];
        for (fmt, frame) in cases {
            let mut d = Decompressor::new(parse_format(fmt).unwrap());
            let out = drive(&mut d, frame, 3).unwrap_or_else(|e| panic!("{fmt}: {e}"));
            assert!(out.is_empty(), "{fmt} 空流产出应为空");
        }
    }

    #[test]
    fn test_truncated_and_trailing_junk() {
        for (fmt, frame) in [
            ("gzip", gzip_frame()),
            ("deflate", zlib_frame()),
            ("deflate-raw", RAW_BODY.to_vec()),
        ] {
            // 截断（去尾 1 字节）→ finish 报错。
            let mut d = Decompressor::new(parse_format(fmt).unwrap());
            let err = drive(&mut d, &frame[..frame.len() - 1], 4).unwrap_err();
            assert!(!err.is_empty(), "{fmt} 截断 → 错误");
            // trailing junk → 错误（产出仍先于错误可见）。
            let mut extended = frame.clone();
            extended.push(0);
            let mut d = Decompressor::new(parse_format(fmt).unwrap());
            let mut out = Vec::new();
            match d.feed(&extended, &mut out) {
                Ok(()) => {
                    // 部分格式在 feed 中即报错；未报的（gzip 成员完、垃圾未进）经
                    // finish 后下一字节才报——此处 finish 必须仍是未终态。
                    assert!(d.finish().is_err(), "{fmt} trailing junk → 错误");
                }
                Err(err) => {
                    assert!(!err.is_empty());
                    assert_eq!(out, PAYLOAD, "{fmt} trailing junk：错误前产出可见");
                }
            }
        }
    }

    #[test]
    fn test_zlib_header_validation() {
        let frame = zlib_frame();
        // CM=0 → 错。
        let mut bad = frame.clone();
        bad[0] = 0x00;
        let mut d = Decompressor::new(Fmt::Deflate);
        assert!(drive(&mut d, &bad, 3).is_err(), "CM=0 → 错");
        // FCHECK 坏 → 错。
        let mut bad = frame.clone();
        bad[1] = 157; // (0x78*256+157) % 31 != 0
        let mut d = Decompressor::new(Fmt::Deflate);
        assert!(drive(&mut d, &bad, 3).is_err(), "FCHECK 坏 → 错");
        // FDICT 位 → 错（corpus dictionary 向量：preset dictionary 不支持）。
        let dict_frame: [u8; 15] = [
            0x78, 0xbb, 0x74, 0xee, 0x09, 0x59, 0x2b, 0xc1, 0x2e, 0x0c, 0x00, 0x74, 0xee, 0x09, 0x59,
        ];
        let mut d = Decompressor::new(Fmt::Deflate);
        assert!(drive(&mut d, &dict_frame, 3).is_err(), "FDICT → 错");
        // adler 篡改 → 错。
        let mut bad = frame.clone();
        let last = bad.len() - 1;
        bad[last] ^= 0xff;
        let mut d = Decompressor::new(Fmt::Deflate);
        assert!(drive(&mut d, &bad, 3).is_err(), "adler 篡改 → 错");
        // 头分字节喂入（1B 切片跨 Head 阶段）。
        let mut d = Decompressor::new(Fmt::Deflate);
        assert!(drive(&mut d, &frame, 1).is_ok(), "合法 zlib 1B 切片通过");
    }

    #[test]
    fn test_gzip_header_and_trailer_validation() {
        let frame = gzip_frame();
        // ID 破坏 → 错。
        let mut bad = frame.clone();
        bad[0] = 0xff;
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &bad, 2).is_err());
        // CM 破坏 → 错。
        let mut bad = frame.clone();
        bad[2] = 0;
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &bad, 2).is_err());
        // FLG 保留位 → 错。
        let mut bad = frame.clone();
        bad[3] = 0x20;
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &bad, 2).is_err());
        // CRC 破坏 → 错。
        let mut bad = frame.clone();
        let n = bad.len();
        bad[n - 8] ^= 0xff;
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &bad, 2).is_err(), "CRC32 篡改 → 错");
        // ISIZE 破坏 → 错。
        let mut bad = frame.clone();
        let n = bad.len();
        bad[n - 4] ^= 0xff;
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &bad, 2).is_err(), "ISIZE 篡改 → 错");
    }

    #[test]
    fn test_gzip_extended_header_flags() {
        // FNAME + FHCRC 扩展头：手拼头（FLG = FNAME|FHCRC）+ corpus raw body +
        // 标准尾；FHCRC = CRC32(头) 低 16 位（RFC 1952 §2.3.1.2）。
        let mut head = vec![0x1f, 0x8b, 0x08, 0x0a, 0, 0, 0, 0, 0, 0x03];
        head.extend_from_slice(b"name\0");
        let mut hcrc = flate2::Crc::new();
        hcrc.update(&head);
        head.extend_from_slice(&(hcrc.sum() as u16).to_le_bytes());
        let mut frame = head;
        frame.extend_from_slice(RAW_BODY);
        let mut tcrc = flate2::Crc::new();
        tcrc.update(PAYLOAD);
        frame.extend_from_slice(&tcrc.sum().to_le_bytes());
        frame.extend_from_slice(&(PAYLOAD.len() as u32).to_le_bytes());
        let mut d = Decompressor::new(Fmt::Gzip);
        let out = drive(&mut d, &frame, 3).unwrap();
        assert_eq!(out, PAYLOAD, "FNAME+FHCRC 扩展头成员解出原文");
    }

    #[test]
    fn test_gzip_multi_member() {
        // RFC 1952 concat：两成员串联 → 输出串联。
        let frame = gzip_frame();
        let mut multi = frame.clone();
        multi.extend_from_slice(&frame);
        let mut d = Decompressor::new(Fmt::Gzip);
        let out = drive(&mut d, &multi, 5).unwrap();
        let mut expected = PAYLOAD.to_vec();
        expected.extend_from_slice(PAYLOAD);
        assert_eq!(out, expected, "双成员串联输出");
        // 成员后垃圾 → 错。
        let mut junk = frame.clone();
        junk.push(0x00);
        let mut d = Decompressor::new(Fmt::Gzip);
        assert!(drive(&mut d, &junk, 3).is_err(), "成员后垃圾字节 → 错");
    }

    #[test]
    fn test_output_cap_not_hit_by_normal_streams() {
        // 正常流远离 1GiB 上限（smoke：~200KB 输入全量解出）。
        let payload: Vec<u8> = b"cap smoke payload ".repeat(12_000);
        let compressed = compress("gzip", &payload, 64 * 1024);
        let mut d = Decompressor::new(Fmt::Gzip);
        let out = drive(&mut d, &compressed, 64 * 1024).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn test_host_handle_api_surface() {
        // handle 化 API 面：支持格式返非空 handle；不支持返空串（shim 抛 TypeError）。
        assert!(!compress_new("gzip").is_empty());
        assert!(compress_new("brotli").is_empty());
        assert!(!decompress_new("deflate-raw").is_empty());
        assert!(decompress_new("brotli").is_empty());
        // 未知 handle → 错误 wire（slot 逐出后使用不静默续跑——防数据丢失）。
        assert!(
            !err_of(&decompress_update(u64::MAX, "")).is_empty(),
            "未知 handle → err"
        );
        assert!(
            !err_of(&decompress_finish(u64::MAX)).is_empty(),
            "未知 handle finish → err"
        );
        assert!(!err_of(&compress_update(u64::MAX, "")).is_empty(), "未知 handle → err");
        assert!(
            !err_of(&compress_finish(u64::MAX)).is_empty(),
            "未知 handle finish → err"
        );
    }
}
