//! TextDecoder legacy 编码 host 面（encoding-compat M2 labels 全表 + legacy 解码；
//! M3 增补空 chunk 直通 guard——encoding_rs 0.8.35 空输入调用丢驻留 lead）。
//! encode 方向：form/urlencoded 提交编码桥（form-url-encoding-compat M1，
//! [`text_encode_percent`]）。
//! 经 encoding_rs（WHATWG encoding 标准 tables 的生成实现，zero-net 同款工作区依赖）——
//! labels 标签匹配（trim ASCII whitespace + ASCII case-insensitive 全表）与解码器
//!（单字节 28 编码 / GBK / GB18030 / Big5 / Shift_JIS / EUC-JP / EUC-KR /
//! ISO-2022-JP 状态机 / UTF-16LE-BE / replacement）全表数据驱动，不散写。
//! 字节经 csv 十进制 wire（crypto 模块对称），解码输出经 JSON wire（[`super::json_str`]——
//! 文本可含任意字符）。
//!
//! **有状态 decoder 表**：spec decode 是跨 decode() 调用的有状态算法（stream:true 半截
//! 多字节 lead / iso-2022-jp ESC 模式机驻留 decoder 内部）——纯字节回传 carry 无法表达
//!（encoding_rs 消费 lead 后 read 计数含 lead，宿主侧已无该字节）。故 host 持
//! handle→Decoder 表，JS 侧 TextDecoder 持 handle；容量封顶 FIFO 逐出，逐出后 decode
//! 按同 label+ignoreBOM 重建（透明降级为丢一次跨块状态）。
//! spec：https://encoding.spec.whatwg.org/#interface-textdecoder

use super::crypto::{bytes_from_csv, bytes_to_csv};
use super::json_str;
use encoding_rs::{Decoder, DecoderResult, Encoding};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::sync::OnceLock;

/// decoder 表容量封顶——每条目数十字节（Encoding 引用 + 状态机），4096 条 ≈ 亚 MB 级；
/// 逐出透明重建（丢跨块状态，等同无状态实现的行为上界）。
const DECODER_TABLE_CAP: usize = 4096;

struct DecoderSlot {
    decoder: Decoder,
}

#[derive(Default)]
struct DecoderTable {
    slots: HashMap<u64, DecoderSlot>,
    queue: VecDeque<u64>,
    next_id: u64,
}

fn decoder_table() -> &'static Mutex<DecoderTable> {
    static TABLE: OnceLock<Mutex<DecoderTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(DecoderTable::default()))
}

/// 标签 → 规范编码名（小写，spec `TextDecoder.prototype.encoding` 属性值）。
/// 未知标签 → 空串（shim 抛 RangeError）。replacement 照表返 "replacement"——
/// TextDecoder/TextDecoderStream 构造面拒绝（spec：replacement 编码不可经 decoder
/// 构造），XHR overrideMimeType 解码面照用（final encoding = replacement → 整段单
/// U+FFFD）。空串返空串（shim 缺省 utf-8，不经本函数）。
pub fn text_encoding_of(label: &str) -> String {
    match Encoding::for_label(label.as_bytes()) {
        Some(encoding) => encoding.name().to_ascii_lowercase(),
        None => String::new(),
    }
}

/// `new TextDecoder(label, {ignoreBOM})` 的 legacy host 半步：建有状态 decoder，
/// 返 handle 十进制串（未知标签 → 空串，shim RangeError）。BOM 语义由 encoding_rs
/// decoder 承接：`new_decoder_with_bom_removal`（首 decode 剥匹配本编码的 BOM——spec
/// decode-BOM：他编码 BOM 不剥、按内容解，且永不 morph 换编码）vs ignoreBOM →
/// `without`（BOM 按内容解出 U+FEFF——spec「do not strip」）。
pub fn text_decoder_new(label: &str, ignore_bom: bool) -> String {
    let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
        return String::new();
    };
    let decoder = if ignore_bom {
        encoding.new_decoder_without_bom_handling()
    } else {
        encoding.new_decoder_with_bom_removal()
    };
    let mut table = decoder_table().lock().unwrap_or_else(|e| e.into_inner());
    table.next_id += 1;
    let id = table.next_id;
    table.queue.push_back(id);
    while table.queue.len() > DECODER_TABLE_CAP {
        if let Some(oldest) = table.queue.pop_front() {
            table.slots.remove(&oldest);
        }
    }
    table.slots.insert(id, DecoderSlot { decoder });
    id.to_string()
}

/// `TextDecoder.prototype.decode(bytes, {stream})` 的 legacy host 半步：字节 csv 喂
/// handle 对应 decoder。`last` = !stream（flush——残余不完整序列按 fatal/替换语义收尾）。
/// 返 JSON `{"text":...,"err":0|1}`：err = fatal 且 malformed（shim 抛 TypeError——spec
/// do not flush 置位时状态保留、未置位时 shim 重建 handle 等价 reset）。handle 缺失（封顶逐出后超期
/// 使用）→ 返空串（透明降级，与丢跨块状态的上界行为一致）。
pub fn text_decoder_decode(handle: u64, bytes_csv: &str, fatal: bool, last: bool) -> String {
    let data = bytes_from_csv(bytes_csv);
    // encoding_rs 0.8.35 空输入调用会丢弃驻留 lead（big5/shift_jis/euc-kr stream 空 chunk
    // 实证：hold [0x87] → 空 chunk → [0x40] 得 "@" 而非 "䏰"；flush（last=true）面不受
    // 影响）——空块非 flush 直通为无输出。
    if data.is_empty() && !last {
        return "{\"text\":\"\",\"err\":0}".to_string();
    }
    let mut table = decoder_table().lock().unwrap_or_else(|e| e.into_inner());
    // 逐出/未知 handle → 透明重建（无 slot 元数据可循时——不应发生——按 utf-8 语义空转）。
    if !table.slots.contains_key(&handle) {
        return "{\"text\":\"\",\"err\":0}".to_string();
    }
    let slot = table.slots.get_mut(&handle).expect("contains_key checked");
    let mut text = String::with_capacity((data.len() * 3) / 2 + 8);
    // 两 API 返回形状不同：fatal 面（without_replacement）返 (DecoderResult, read)——
    // 非 InputEmpty 即 malformed；非 fatal 面已按替换语义收尾（malformed → U+FFFD），无 err。
    let err = if fatal {
        let (result, _read) = slot
            .decoder
            .decode_to_string_without_replacement(&data, &mut text, last);
        result != DecoderResult::InputEmpty
    } else {
        let (_result, _read, _replaced) = slot.decoder.decode_to_string(&data, &mut text, last);
        false
    };
    format!("{{\"text\":{},\"err\":{}}}", json_str(&text), u8::from(err))
}

/// form/urlencoded 提交编码 encode 桥（form-url-encoding-compat M1）：legacy 提交
/// 编码逐 scalar fresh 编码出字节，再按 URL standard percent-encode after encoding
///（https://url.spec.whatwg.org/#string-percent-encode-after-encoding）字节级百分号
/// 编码。HTML urlencoded serializer 的编码来自「select a form submission encoding」
///（accept-charset 逐 token，首个合法者胜，回落文档编码）——shim 侧经
/// `_zwEncodingFromLabel` 归一后传规范名；UTF-8 路径不经本桥（shim 既有纯 JS
/// `_encPair` 零变化）。
///
/// **逐 scalar fresh 模型**：每码点独立编码、编码器状态不跨码点驻留、不产
/// end-of-queue flush 切换。WPT iso-2022-jp encode-form 期望结构钉死：条目分隔符
///（U+0019，ASCII）编为裸 %19、条目 = 完整 fresh 序列（ESC $ B 前缀逐条目重现）；
/// 整串流式编码会在每个 ASCII 字节前多出 ESC ( B 模式切换而判 FAIL，Chrome 全
/// variant 过（wpt.fyi c=[1000,1000]）同向实证。无状态编码（euc-jp/big5/gbk/
/// euc-kr/shift_jis/windows-1252）逐 scalar 与整串逐位相同，无行为差。
///
/// mode="form"：application/x-www-form-urlencoded percent-encode set——字面量仅
/// alnum + `*` `-` `.` `_`（与 shim `_encPair` 的 encodeURIComponent+补编同域），
/// 空格 → `+`。mode="query"：special-query set（C0 + >U+007E + 空格 `"` `#` `<` `>` `` ` `` +
/// `'`，M2 href 面预留）。unmappable → encoding_rs 出 HTML NCR `&#N;` 字节，
/// `&` `#` `;` 在 set 内自然成 `%26%23N%3B`（spec errors 面同式）。UTF-16 系与
/// replacement 经 get-an-output-encoding 以 UTF-8 出字节（URL standard 同名算法）。
/// 未知 label → 空串（shim 归一后方调用，正常不达）。
pub fn text_encode_percent(label: &str, text: &str, mode: &str) -> String {
    let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
        return String::new();
    };
    // get-an-output-encoding：replacement 与 UTF-16 系的输出编码 = UTF-8。
    let encoding = if encoding == encoding_rs::REPLACEMENT
        || encoding == encoding_rs::UTF_16LE
        || encoding == encoding_rs::UTF_16BE
    {
        encoding_rs::UTF_8
    } else {
        encoding
    };
    let query_mode = mode == "query";
    let mut out = String::with_capacity(text.len() * 3 + 8);
    if encoding == encoding_rs::UTF_8 {
        encode_percent_bytes(text.as_bytes(), query_mode, &mut out);
        return out;
    }
    let mut buf = [0u8; 4];
    for ch in text.chars() {
        let (encoded, _, _) = encoding.encode(ch.encode_utf8(&mut buf));
        let raw: &[u8] = encoded.as_ref();
        // ISO-2022-JP fresh 单 scalar 编码的 end-of-queue flush 尾巴（ESC ( B）剥除——
        // per-scalar 输出里恒为伪迹（jis0208 字节域 0x21-0x7E、katakana 0xA1-0xDF、
        // roman 0x5C/0x7E 均不含 0x1B；ASCII 面不切模式），非编码产出。
        let raw = if encoding == encoding_rs::ISO_2022_JP && raw.ends_with(&[0x1B, 0x28, 0x42]) {
            &raw[..raw.len() - 3]
        } else {
            raw
        };
        // NCR 错误路径直出：unmappable 的 HTML NCR 字节（`&#N;`）不经 set 再判定，
        // 以预编码形态 `%26%23N%3B` 直接落输出（URL standard percent-encode after
        // encoding 的 HTML error mode 面，
        // https://url.spec.whatwg.org/#string-percent-encode-after-encoding）——
        // special-query set 不含 `&` `;`，set 路径在 query mode 会漏编界定符出
        // 字面量 `&#N;`；corpus 侧 errors-href expect 硬编码 `%26%23N%3B` +
        // gb18030-encoder 硬编码期望同向钉死。form mode 经 set 展开逐字节同形
        //（等价路径，M1 form 面行为不变）。guard 前置 len/首字节，避免可映射
        // 输出逐 char 分配。
        if raw.len() > 1 && raw[0] == b'&' && raw == format!("&#{};", ch as u32).as_bytes() {
            out.push_str("%26%23");
            out.push_str(&(ch as u32).to_string());
            out.push_str("%3B");
            continue;
        }
        encode_percent_bytes(raw, query_mode, &mut out);
    }
    out
}

/// form 提交编码字节化桥（html-semantics M2 S184，text/plain + multipart
/// name/value/filename 面）：逐 scalar fresh 编码进提交编码（[`text_encode_percent`]
/// 同款模型——ISO-2022-JP ESC 伪迹剥除、encoding_rs HTML error mode 出 unmappable
/// 的 NCR `&#N;` 字节；lone surrogate 经 JS→Rust 损毁转换已落 U+FFFD，windows-1252
/// 不可映射 → `&#65533;` 同路径），字节 csv 十进制返（crypto 模块 wire，
/// shim `_zwDecodeBytesPrefix` 对称）。HTML §4.10.22.4/§4.10.22.5 text/plain 与
/// multipart/form-data encoding algorithm 的提交编码消费臂（WPT
/// forms/form-submission-0 text-plain/multipart-formdata windows-1252 钉态：
/// á→0xE1、ə→`&#601;`、💩→`&#128169;`、lone surrogate→`&#65533;`）。
/// UTF-8 臂 = `as_bytes` 直通（shim 仅非 UTF-8 提交编码调用本桥，UTF-8 路径
/// 既有纯 JS 字节组装零变化）；未知 label → 空串（shim 归一后方调用）。
pub fn text_encode_form_bytes(label: &str, text: &str) -> String {
    let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
        return String::new();
    };
    // get-an-output-encoding：replacement 与 UTF-16 系的输出编码 = UTF-8（与
    // [`text_encode_percent`] 同门判定）。
    let encoding = if encoding == encoding_rs::REPLACEMENT
        || encoding == encoding_rs::UTF_16LE
        || encoding == encoding_rs::UTF_16BE
    {
        encoding_rs::UTF_8
    } else {
        encoding
    };
    if encoding == encoding_rs::UTF_8 {
        return bytes_to_csv(text.as_bytes());
    }
    let mut bytes = Vec::with_capacity(text.len() + 8);
    let mut buf = [0u8; 4];
    for ch in text.chars() {
        let (encoded, _, _) = encoding.encode(ch.encode_utf8(&mut buf));
        let raw: &[u8] = encoded.as_ref();
        // ISO-2022-JP fresh 单 scalar 编码的 end-of-queue flush 尾巴剥除
        //（[`text_encode_percent`] 同款——per-scalar 输出恒为伪迹非编码产出）。
        let raw = if encoding == encoding_rs::ISO_2022_JP && raw.ends_with(&[0x1B, 0x28, 0x42]) {
            &raw[..raw.len() - 3]
        } else {
            raw
        };
        bytes.extend_from_slice(raw);
    }
    bytes_to_csv(&bytes)
}

/// 字节级 percent-encode set 判定 + 编码（URL standard 序列化语义：输出字节按其
/// 字节值比对该 set，https://url.spec.whatwg.org/#percent-encoded-bytes）。form
/// mode：空格 → `+`，字面量 = alnum + `*-._`；query mode：special-query set
///（query set = C0 controls + >0x7E + 空格 `"` `#` `<` `>`，special 补 `'`）+
/// `%`——engine 适配：rust-url 解析面保留裸 `%` 不归一 `%25`（Chrome 消费侧归一，
/// WPT iso2022jp-encode-href 期望依赖该归一），预编码源直接出 `%25` 对齐。
fn encode_percent_bytes(bytes: &[u8], query_mode: bool, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &b in bytes {
        if !query_mode && b == b' ' {
            out.push('+');
            continue;
        }
        let in_set = if query_mode {
            b <= 0x20 || b >= 0x7F || matches!(b, b'%' | b'"' | b'#' | b'<' | b'>' | b'`' | b'\'')
        } else {
            !(b.is_ascii_alphanumeric() || matches!(b, b'*' | b'-' | b'.' | b'_'))
        };
        if in_set {
            out.push('%');
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
        } else {
            out.push(b as char);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn decode_text(handle: &str, csv: &str, fatal: bool, last: bool) -> (String, bool) {
        let out = text_decoder_decode(handle.parse().unwrap(), csv, fatal, last);
        let text_start = out.find("\"text\":").unwrap() + 7;
        // json_str 输出带引号（内部引号已转义）——剥首尾引号取原文。
        let text_end = out.rfind(",\"err\":").unwrap();
        let err = out[text_end..].ends_with(":1}");
        let raw = &out[text_start..text_end];
        (raw[1..raw.len() - 1].to_string(), err)
    }

    #[test]
    fn test_labels_table() {
        // 大小写不敏感 + 首尾 ASCII whitespace trample（spec get-an-encoding-by-label）。
        assert_eq!(text_encoding_of("utf-8"), "utf-8");
        assert_eq!(text_encoding_of("  UTF-8\t"), "utf-8");
        assert_eq!(text_encoding_of("Unicode-1-1-UTF-8"), "utf-8");
        assert_eq!(text_encoding_of("windows-1252"), "windows-1252");
        assert_eq!(text_encoding_of("CP1252"), "windows-1252");
        assert_eq!(text_encoding_of("GBK"), "gbk");
        assert_eq!(text_encoding_of("gb2312"), "gbk");
        assert_eq!(text_encoding_of("GB18030"), "gb18030");
        assert_eq!(text_encoding_of("Shift_JIS"), "shift_jis");
        assert_eq!(text_encoding_of("EUC-KR"), "euc-kr");
        assert_eq!(text_encoding_of("Big5"), "big5");
        assert_eq!(text_encoding_of("ISO-2022-JP"), "iso-2022-jp");
        assert_eq!(text_encoding_of("utf-16"), "utf-16le");
        assert_eq!(text_encoding_of("utf-16le"), "utf-16le");
        assert_eq!(text_encoding_of("utf-16be"), "utf-16be");
        // 未知/NUL 污染 → 空串（shim RangeError）。
        assert_eq!(text_encoding_of("invalid-invalidLabel"), "");
        assert_eq!(text_encoding_of("\0unicode-1-1-utf-8"), "");
        assert_eq!(text_encoding_of(""), "");
        // replacement 照表识别（构造面拒绝归 shim）。
        assert_eq!(text_encoding_of("csiso2022kr"), "replacement");
        assert_eq!(text_encoding_of("hz-gb-2312"), "replacement");
        assert_eq!(text_encoding_of("iso-2022-cn"), "replacement");
    }

    #[test]
    fn test_unknown_label_no_decoder() {
        assert_eq!(text_decoder_new("invalid-invalidLabel", false), "");
    }

    #[test]
    fn test_single_byte_decode() {
        // windows-1252：0x41→'A'、0x80→U+20AC（€）。
        let handle = text_decoder_new("windows-1252", false);
        let (text, err) = decode_text(&handle, "65,128", false, true);
        assert!(text.contains('\u{20ac}') && text.starts_with('A'), "{text}");
        assert!(!err);
    }

    #[test]
    fn test_gbk_gb18030_decode() {
        // GBK pointer 0（0x81 0x40）→ index-gbk[0] = U+4E02。
        let handle = text_decoder_new("gbk", false);
        let (text, _) = decode_text(&handle, "129,64", false, true);
        assert!(text.contains('\u{4e02}'), "{text}");
        // GB18030 单字节 0x80 → €（U+20AC）。
        let handle = text_decoder_new("gb18030", false);
        let (text, _) = decode_text(&handle, "128", false, true);
        assert!(text.contains('\u{20ac}'), "{text}");
        // GB18030 四字节 0x81 0x30 0x81 0x30 → 线性指针 0 → U+0080。
        let handle = text_decoder_new("gb18030", false);
        let (text, _) = decode_text(&handle, "129,48,129,48", false, true);
        assert!(text.contains('\u{80}'), "{text}");
    }

    #[test]
    fn test_iso_2022_jp_state_machine() {
        // WPT iso-2022-jp-decoder 向量：ESC ( B 0x50 → "P"（回 ASCII）；ESC $ B 0x50 0x50 → 「佩」。
        let handle = text_decoder_new("iso-2022-jp", false);
        let (text, _) = decode_text(&handle, "27,40,66,80", false, true);
        assert_eq!(text, "P", "{text}");
        let handle = text_decoder_new("iso-2022-jp", false);
        let (text, _) = decode_text(&handle, "27,36,66,80,80", false, true);
        assert!(text.contains('\u{4f69}'), "{text}");
        // 模式机跨 decode() 调用驻留：首调用 ESC ( B（stream 不 flush——flush 即 finished），
        // 次调用裸 0x50 仍按 ASCII 解（状态机在 decoder 内部，不随调用边界重置）。
        let handle = text_decoder_new("iso-2022-jp", false);
        let (text, _) = decode_text(&handle, "27,40,66", false, false);
        assert_eq!(text, "", "{text}");
        let (text, _) = decode_text(&handle, "80", false, true);
        assert_eq!(text, "P", "{text}");
    }

    #[test]
    fn test_utf16_decode_and_bom() {
        // utf-16le "z¢" = 7A 00 A2 00。
        let handle = text_decoder_new("utf-16le", false);
        let (text, _) = decode_text(&handle, "122,0,162,0", false, true);
        assert!(text.contains('\u{a2}') && text.starts_with('z'), "{text}");
        // BOM 剥除：FF FE 前缀匹配 utf-16le → 剥（spec decode-BOM）。
        let handle = text_decoder_new("utf-16le", false);
        let (text, _) = decode_text(&handle, "255,254,122,0", false, true);
        assert_eq!(text, "z", "{text}");
        // ignoreBOM：BOM 保留 → 解出 U+FEFF。
        let handle = text_decoder_new("utf-16le", true);
        let (text, _) = decode_text(&handle, "255,254,122,0", false, true);
        assert_eq!(text, "\u{feff}z", "{text}");
        // utf-16be：FE FF 前缀匹配 → 剥。
        let handle = text_decoder_new("utf-16be", false);
        let (text, _) = decode_text(&handle, "254,255,0,162", false, true);
        assert_eq!(text, "\u{a2}", "{text}");
    }

    #[test]
    fn test_replacement_decode() {
        // replacement（经 decoder 表——XHR final-encoding 面 shim 直接走本 host）：
        // 非空输入 → 整段单 U+FFFD；空输入 → 空。
        let handle = text_decoder_new("csiso2022kr", false);
        let (text, _) = decode_text(&handle, "65,66,67,160", false, true);
        assert_eq!(text, "\u{fffd}", "{text}");
        let handle = text_decoder_new("csiso2022kr", false);
        let (text, _) = decode_text(&handle, "", false, true);
        assert_eq!(text, "", "{text}");
    }

    #[test]
    fn test_streaming_split_multibyte() {
        // shift_jis 0x93 0xFA → 「日」（U+65E5）：lead 跨 chunk 切开——host decoder
        // 内部驻留 lead，stream（last=false）半截不出字、不丢 lead；flush 拼上 trail 还原。
        let handle = text_decoder_new("shift_jis", false);
        let (text, _) = decode_text(&handle, "147", false, false);
        assert_eq!(text, "", "{text}");
        let (text, _) = decode_text(&handle, "250", false, true);
        assert!(text.contains('\u{65e5}'), "{text}");
    }

    #[test]
    fn test_fatal_malformed_errors() {
        // fatal：utf-16be 奇数长度（孤立尾字节）→ malformed → err=1。
        let handle = text_decoder_new("utf-16be", false);
        let (_, err) = decode_text(&handle, "0,162,122", true, true);
        assert!(err);
        // 非 fatal 同输入 → 替换、err=0。
        let handle = text_decoder_new("utf-16be", false);
        let (_, err) = decode_text(&handle, "0,162,122", false, true);
        assert!(!err);
        // fatal 完整输入 → err=0。
        let handle = text_decoder_new("utf-16be", false);
        let (_, err) = decode_text(&handle, "0,162,0,122", true, true);
        assert!(!err);
    }

    #[test]
    fn test_table_cap_eviction() {
        // 封顶逐出不 panic：灌满 +1 后最老 handle 解码仍返回（透明降级——逐出 handle
        // 无元数据重建路径，返空串）。
        let first = text_decoder_new("windows-1252", false);
        for _ in 0..DECODER_TABLE_CAP {
            text_decoder_new("utf-16le", false);
        }
        let out = text_decoder_decode(first.parse().unwrap(), "65", false, true);
        assert!(out.contains("\"err\":0"), "{out}");
    }

    #[test]
    fn test_encode_percent_utf8_form_literals() {
        // 字面量域 alnum + `*-._`（与 shim _encPair 的 encodeURIComponent+补编同域）；
        // 空格 '+'；form set 补编 !'()~ 及 path/userinfo/component 系字符。
        assert_eq!(text_encode_percent("UTF-8", "a*-._Z9", "form"), "a*-._Z9");
        assert_eq!(text_encode_percent("UTF-8", "a b", "form"), "a+b");
        assert_eq!(text_encode_percent("UTF-8", "!'()~", "form"), "%21%27%28%29%7E");
        assert_eq!(text_encode_percent("UTF-8", "a=b&c#d", "form"), "a%3Db%26c%23d");
        // 未知 label → 空串（shim 归一后方调用，正常不达）。
        assert_eq!(text_encode_percent("no-such-enc", "A", "form"), "");
    }

    #[test]
    fn test_encode_percent_windows_1252() {
        // € U+20AC → 0x80（M0 失败形态对称锚点：期望 %80 而非 UTF-8 %E2%82%AC）。
        assert_eq!(text_encode_percent("windows-1252", "\u{20ac}", "form"), "%80");
        assert_eq!(text_encode_percent("windows-1252", "A \u{20ac}", "form"), "A+%80");
    }

    #[test]
    fn test_encode_percent_euc_jp() {
        // euc-jp U+FF9A → 8E DA（M0 实测断言消息「expected %8E%DA but got
        // %EF%BE%9A」——host 桥产出即期望侧字节序列）。
        assert_eq!(text_encode_percent("EUC-JP", "\u{ff9a}", "form"), "%8E%DA");
        // 分隔符 ","（eucjp-encode-form separator）→ %2C，条目拼接形态。
        assert_eq!(
            text_encode_percent("EUC-JP", "\u{ff9a},\u{ff9a}", "form"),
            "%8E%DA%2C%8E%DA"
        );
    }

    #[test]
    fn test_encode_percent_iso_2022_jp_fresh_scalar() {
        // 佩 U+4F69 = jis0208 0x50 0x50（decode 面 test_iso_2022_jp_state_machine
        // 同表反向）——fresh 条目 = ESC $ B 前缀 + 双字节；字节级 set 下 0x42/0x50
        // alnum 字面量 → "%1B%24BPP"（百分号解码后 == ESC $ B P P；flush 尾巴
        // ESC ( B 已剥）。整串流式编码会产出 "%1B%24BPP%1B(B..." 形态，此处同时
        // 钉死 fresh 模型与字节级 set 两语义。
        assert_eq!(text_encode_percent("ISO-2022-JP", "\u{4f69}", "form"), "%1B%24BPP");
        // 分隔符 U+0019 fresh ASCII 面 → 裸 %19（无模式切换）；两条目串保持
        // split(encodedSeparator) 结构性质（WPT harness 依赖）。
        assert_eq!(text_encode_percent("ISO-2022-JP", "\u{0019}", "form"), "%19");
        assert_eq!(
            text_encode_percent("ISO-2022-JP", "\u{4f69}\u{0019}\u{4f69}", "form"),
            "%1B%24BPP%19%1B%24BPP"
        );
        // unmappable（U+4DC0 不在 jis0208）→ HTML NCR `&#19904;` → set 补编
        // %26%23N%3B（WPT encode-form errors 变体期望公式）。
        assert_eq!(text_encode_percent("ISO-2022-JP", "\u{4dc0}", "form"), "%26%2319904%3B");
    }

    #[test]
    fn test_encode_percent_query_mode_and_output_encoding() {
        // query mode（M2 href 面预留）：special-query set——空格 %20（非 '+'）、
        // `'` 补编、`!` 字面量。
        assert_eq!(text_encode_percent("UTF-8", "a b'c", "query"), "a%20b%27c");
        assert_eq!(text_encode_percent("UTF-8", "a!b", "query"), "a!b");
        // get-an-output-encoding：utf-16le / replacement 输出面 = UTF-8。
        assert_eq!(text_encode_percent("utf-16le", "\u{4f69}", "form"), "%E4%BD%A9");
        assert_eq!(text_encode_percent("csiso2022kr", "A", "form"), "A");
    }

    #[test]
    fn test_encode_percent_query_ncr_direct() {
        // M2 href 面 NCR 直出：query mode 下 unmappable → 预编码 `%26%23N%3B`
        //（special-query set 不含 `&` `;`，set 路径会出字面量 `&#N;`——
        // errors-href expect 硬编码同式）。
        assert_eq!(text_encode_percent("EUC-JP", "\u{4dc0}", "query"), "%26%2319904%3B");
        // form mode 输出不变：set 展开与直出逐字节同形（M1 行为锁）。
        assert_eq!(text_encode_percent("EUC-JP", "\u{4dc0}", "form"), "%26%2319904%3B");
        // ISO-2022-JP NCR：模式切换序列后直出判定不误伤（尾剥后仍 `&#N;` 形态）。
        assert_eq!(
            text_encode_percent("ISO-2022-JP", "\u{4dc0}", "query"),
            "%26%2319904%3B"
        );
        // mappable 输出不误入直出：euc-jp ¥（U+00A5）→ 0x5C，query set 外 → 字面量。
        assert_eq!(text_encode_percent("EUC-JP", "\u{a5}", "query"), "\\");
        // ISO-2022-JP roman fresh：¥ → ESC ( J + 0x5C（无 flush 尾）——%1B 出 set，
        // ( J \ 字面量。
        assert_eq!(text_encode_percent("ISO-2022-JP", "\u{a5}", "query"), "%1B(J\\");
        // 字面量 `&`（单字节 mappable，len 1）不触发直出判定：query set 外 → 原样。
        assert_eq!(text_encode_percent("windows-1252", "a&b", "query"), "a&b");
        // `%` 归一（rust-url 保留裸 %，Chrome 消费侧归一 %25——预编码源对齐）：
        // ISO-2022-JP Ε（U+0395）fresh = ESC $ B 0x26 0x25，字面 $ B & + %25，
        // normalizeStr 后与期望 %1B%24%42%26%25 逐位合流（WPT 实测失败向量）。
        assert_eq!(text_encode_percent("ISO-2022-JP", "\u{395}", "query"), "%1B$B&%25");
        // 字面量 `%`（UTF-8 面）query 出 %25；form 面行为不变。
        assert_eq!(text_encode_percent("UTF-8", "a%", "query"), "a%25");
        assert_eq!(text_encode_percent("UTF-8", "a%", "form"), "a%25");
    }

    #[test]
    fn test_encode_form_bytes_windows_1252_ncr() {
        // S184 WPT 钉态（forms/form-submission-0 text-plain/multipart windows-1252）：
        // mappable á U+00E1 → 0xE1 单字节（echo 往返 U+00E1 同字符）。
        assert_eq!(text_encode_form_bytes("windows-1252", "\u{e1}"), "225");
        // € U+20AC → 0x80（text_encode_percent %80 锚点同表反向）。
        assert_eq!(text_encode_form_bytes("windows-1252", "\u{20ac}"), "128");
        // unmappable ə U+0259 → HTML NCR `&#601;` 字节（expected a&#601;b 面）。
        assert_eq!(
            text_encode_form_bytes("windows-1252", "a\u{259}b"),
            "97,38,35,54,48,49,59,98"
        );
        // U+FFFD（lone surrogate 桥入形态）→ `&#65533;` 五位十进制 NCR（expected
        // `&#65533;` 面；& # 6 5 5 3 3 ; 八字节）。
        assert_eq!(
            text_encode_form_bytes("windows-1252", "\u{fffd}"),
            "38,35,54,53,53,51,51,59"
        );
        // astral 💩 U+1F4A9 → `&#128169;` 码点十进制（filename 面）。
        assert_eq!(
            text_encode_form_bytes("windows-1252", "\u{1f4a9}"),
            "38,35,49,50,56,49,54,57,59"
        );
        // ASCII 体（含 CRLF/NUL）两编码同 csv——UTF-8 路径零变化锚。
        assert_eq!(
            text_encode_form_bytes("windows-1252", "x=y\r\n"),
            text_encode_form_bytes("UTF-8", "x=y\r\n")
        );
        // UTF-8 臂 = as_bytes 直通（非 ASCII 出 UTF-8 字节序列）。
        assert_eq!(text_encode_form_bytes("UTF-8", "a\u{e9}"), "97,195,169");
        // 未知 label → 空串（shim 归一后方调用，正常不达）。
        assert_eq!(text_encode_form_bytes("no-such-enc", "A"), "");
    }
}

#[cfg(test)]
mod stream_empty_chunk_guard {
    use super::tests::decode_text;
    use super::*;

    /// encoding_rs 0.8.35 空输入调用会丢弃驻留 lead（big5 hold [0x87] → 空 chunk →
    /// [0x40] 得 "@" 而非 "䏰"）——host 对「空块非 flush」直通为无输出（flush 面不受
    /// 影响，产出替换字符）。锁定 guard 行为防回退。
    #[test]
    fn test_empty_chunk_guard_and_flush() {
        let handle = text_decoder_new("big5", false);
        let (a, _) = decode_text(&handle, "135", false, false); // lead 驻留
        assert_eq!(a, "");
        let (b, _) = decode_text(&handle, "", false, false); // 空 chunk 非 flush → 直通
        assert_eq!(b, "");
        let (c, _) = decode_text(&handle, "64", false, false); // lead 仍驻留——成对成功
        assert_eq!(c, "\u{43f0}", "空 chunk 不得丢弃驻留 lead");
        // flush（last=true）空输入：驻留半截 → 替换字符（guard 不影响 flush 语义）。
        let handle2 = text_decoder_new("big5", false);
        let (d, _) = decode_text(&handle2, "135", false, false);
        assert_eq!(d, "");
        let (e, err) = decode_text(&handle2, "", false, true);
        assert_eq!(e, "\u{fffd}");
        assert!(!err);
    }
}
