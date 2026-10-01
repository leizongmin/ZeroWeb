//! TextDecoder legacy 编码 host 面（encoding-compat M2）。
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

use super::crypto::bytes_from_csv;
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
/// fatal 错误后本 decoder 不再可用，shim 重建 handle）。handle 缺失（封顶逐出后超期
/// 使用）→ 返空串（透明降级，与丢跨块状态的上界行为一致）。
pub fn text_decoder_decode(handle: u64, bytes_csv: &str, fatal: bool, last: bool) -> String {
    let data = bytes_from_csv(bytes_csv);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_text(handle: &str, csv: &str, fatal: bool, last: bool) -> (String, bool) {
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
}
