//! WOFF / WOFF2 web font container decoding.
//!
//! WOFF 是带 zlib 压缩的 sfnt 字体容器（`.woff`）。fontdue 仅加载裸 sfnt
//! （`.ttf`/`.otf`），不识别 woff 容器；本模块把 WOFF 字节解码为 sfnt 字节序列，
//! 使 `FontLoader::load_font` 能消费 `@font-face` 声明的 `.woff` 字体。
//!
//! 背景：WPT reftest 大量使用 `.woff` `@font-face`（如 text-transform / font-family
//! 簇）。此前 `load_font_faces_into` 对 `.woff` 静默跳过（fontdue 解析失败），
//! 致测试字体回退、diff 主导。本解码器补齐 W3C WOFF 1.0 解压路径。
//!
//! 规格：https://www.w3.org/TR/WOFF/

use flate2::read::ZlibDecoder;
use std::io::Read;

/// WOFF 文件魔数（`"wOFF"`，4 字节，offset 0）。
const WOFF_MAGIC: [u8; 4] = *b"wOFF";
/// WOFF2 file magic (`"wOF2"`).
const WOFF2_MAGIC: [u8; 4] = *b"wOF2";
/// WOFF 头部固定长度（44 字节）。
const WOFF_HEADER_LEN: usize = 44;
/// WOFF 表目录每条记录长度（20 字节）。
const WOFF_TABLE_ENTRY_LEN: usize = 20;
/// sfnt 偏移表（offset table）长度（12 字节）。
const SFNT_OFFSET_TABLE_LEN: usize = 12;
/// sfnt 表目录每条记录长度（16 字节）。
const SFNT_TABLE_ENTRY_LEN: usize = 16;

/// 检测字节数据是否为 WOFF 1.0 容器（`"wOFF"` 魔数）。
///
/// 注意：WOFF2（`"wOF2"`）是不同格式（brotli 压缩），本解码器不处理。
pub fn is_woff(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == WOFF_MAGIC
}

/// Detect a WOFF2 container.
pub fn is_woff2(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == WOFF2_MAGIC
}

/// Decode WOFF2 bytes into a raw sfnt font.
///
/// https://www.w3.org/TR/WOFF2/
pub fn decode_woff2(data: &[u8]) -> Option<Vec<u8>> {
    is_woff2(data).then(|| wuff::decompress_woff2(data).ok()).flatten()
}

/// 把 WOFF 1.0 字节解码为 sfnt（`.ttf`/`.otf`）字节序列。
///
/// 解析 WOFF 头 + 表目录，zlib 解压每个压缩表（compLength < origLength 时），
/// 重建 sfnt 偏移表 + 表目录（按 tag 排序）+ 表数据（4 字节对齐）。
///
/// 返回 `None`：数据残缺、魔数不符、或解压/校验失败。
pub fn decode_woff(data: &[u8]) -> Option<Vec<u8>> {
    if !is_woff(data) || data.len() < WOFF_HEADER_LEN {
        return None;
    }
    // WOFF 头：flavor（sfnt 版本，offset 4）+ numTables（offset 12）
    let flavor = read_u32(data, 4)?;
    let num_tables = read_u16(data, 12)? as usize;

    // 解析表目录：每条 (tag, offset, comp_len, orig_len, checksum)
    let dir_start = WOFF_HEADER_LEN;
    if data.len() < dir_start + num_tables * WOFF_TABLE_ENTRY_LEN {
        return None;
    }
    let mut entries: Vec<TableEntry> = Vec::with_capacity(num_tables);
    for i in 0..num_tables {
        let e = dir_start + i * WOFF_TABLE_ENTRY_LEN;
        entries.push(TableEntry {
            tag: read_u32(data, e)?,
            offset: read_u32(data, e + 4)? as usize,
            comp_len: read_u32(data, e + 8)? as usize,
            orig_len: read_u32(data, e + 12)? as usize,
            checksum: read_u32(data, e + 16)?,
        });
    }

    // 解压每个表
    let mut tables: Vec<Vec<u8>> = Vec::with_capacity(num_tables);
    for e in &entries {
        if data.len() < e.offset + e.comp_len {
            return None;
        }
        let raw = &data[e.offset..e.offset + e.comp_len];
        let body: Vec<u8> = if e.comp_len < e.orig_len {
            // zlib（RFC 1950）压缩——WOFF spec 规定
            let mut d = ZlibDecoder::new(raw);
            let mut out = Vec::with_capacity(e.orig_len);
            d.read_to_end(&mut out).ok()?;
            out
        } else {
            raw.to_vec()
        };
        if body.len() != e.orig_len {
            return None; // 解压大小与 origLength 不符
        }
        tables.push(body);
    }

    // 重建 sfnt：偏移表（12）+ 表目录（numTables × 16）+ 表数据（4 字节对齐）
    let table_data_start = SFNT_OFFSET_TABLE_LEN + num_tables * SFNT_TABLE_ENTRY_LEN;
    // 计算每表在 sfnt 中的偏移（按目录顺序，原始未排序）
    let mut offsets: Vec<u32> = Vec::with_capacity(num_tables);
    let mut cur = table_data_start;
    for body in &tables {
        offsets.push(cur as u32);
        cur += body.len();
        cur += pad4(body.len());
    }

    let total_len = cur;
    let mut sfnt: Vec<u8> = Vec::with_capacity(total_len);

    // 偏移表头：sfnt 版本（flavor）+ numTables + searchRange + entrySelector + rangeShift
    let (search_range, entry_selector, range_shift) = compute_sfnt_search(num_tables);
    sfnt.extend_from_slice(&flavor.to_be_bytes());
    sfnt.extend_from_slice(&(num_tables as u16).to_be_bytes());
    sfnt.extend_from_slice(&search_range.to_be_bytes());
    sfnt.extend_from_slice(&entry_selector.to_be_bytes());
    sfnt.extend_from_slice(&range_shift.to_be_bytes());

    // sfnt 表目录必须按 tag 升序排列
    let mut order: Vec<usize> = (0..num_tables).collect();
    order.sort_by_key(|&i| entries[i].tag);
    for &i in &order {
        let e = &entries[i];
        sfnt.extend_from_slice(&e.tag.to_be_bytes());
        sfnt.extend_from_slice(&e.checksum.to_be_bytes());
        sfnt.extend_from_slice(&offsets[i].to_be_bytes());
        sfnt.extend_from_slice(&(e.orig_len as u32).to_be_bytes());
    }

    // 表数据（按原始目录顺序写入，偏移与 offsets 一致）
    for body in &tables {
        sfnt.extend_from_slice(body);
        let pad = pad4(body.len());
        sfnt.extend(std::iter::repeat_n(0, pad));
    }

    Some(sfnt)
}

/// 单个 WOFF 表目录记录（解析中间结构）。
struct TableEntry {
    tag: u32,
    offset: usize,
    comp_len: usize,
    orig_len: usize,
    checksum: u32,
}

/// 4 字节对齐补齐字节数。
fn pad4(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

/// 计算 sfnt 偏移表的 searchRange / entrySelector / rangeShift。
///
/// - pow2 = 不超过 numTables 的最大 2 的幂
/// - searchRange = pow2 × 16
/// - entrySelector = log2(pow2)
/// - rangeShift = numTables × 16 − searchRange
fn compute_sfnt_search(num_tables: usize) -> (u16, u16, u16) {
    if num_tables == 0 {
        return (0, 0, 0);
    }
    let mut pow2: u32 = 1;
    let mut entry_selector: u32 = 0;
    while pow2 * 2 <= num_tables as u32 {
        pow2 *= 2;
        entry_selector += 1;
    }
    let search_range = (pow2 * 16) as u16;
    let range_shift = (num_tables as u32 * 16 - pow2 * 16) as u16;
    (search_range, entry_selector as u16, range_shift)
}

/// 大端读取 u32（边界检查）。
fn read_u32(data: &[u8], off: usize) -> Option<u32> {
    if data.len() < off + 4 {
        return None;
    }
    Some(u32::from_be_bytes([
        data[off],
        data[off + 1],
        data[off + 2],
        data[off + 3],
    ]))
}

/// 大端读取 u16（边界检查）。
fn read_u16(data: &[u8], off: usize) -> Option<u16> {
    if data.len() < off + 2 {
        return None;
    }
    Some(u16::from_be_bytes([data[off], data[off + 1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实 WPT 使用的 WOFF 字体：fonts/Revalia.woff。
    /// 解码后应为合法 sfnt（开头为 0x00010000 TrueType 或 'OTTO' CFF），
    /// 字节数 > 原 WOFF 的 totalSfntSize 预期（含对齐填充）。
    #[test]
    fn decode_real_woff_revalia() {
        let path = "tests/wpt-runner/wpt-data/fonts/Revalia.woff";
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("skip: Revalia.woff not present");
                return;
            }
        };
        assert!(is_woff(&data), "Revalia.woff 应为 WOFF 容器");
        let sfnt = decode_woff(&data).expect("WOFF 解码应成功");
        // 合法 sfnt 版本：0x00010000（TrueType）或 0x4F54544F（'OTTO' CFF）或 0x74727565（'true'）
        let ver = u32::from_be_bytes([sfnt[0], sfnt[1], sfnt[2], sfnt[3]]);
        assert!(
            ver == 0x00010000 || ver == 0x4F54544F || ver == 0x74727565,
            "sfnt 版本字应为 TrueType/OTTO/true，实际 0x{ver:08X}"
        );
        // 解码后字节数应显著大于 0（至少含偏移表 + 目录 + 若干表）
        assert!(sfnt.len() > 100, "sfnt 过短：{}", sfnt.len());
        // fontdue 应能加载解码后的 sfnt
        let f = fontdue::Font::from_bytes(sfnt.as_slice(), fontdue::FontSettings::default());
        assert!(f.is_ok(), "fontdue 应能加载解码后的 sfnt：{:?}", f.err());
    }

    /// 非 WOFF 数据（裸 ttf 魔数 0x00010000）不应被识别为 WOFF。
    #[test]
    fn is_woff_rejects_non_woff() {
        assert!(!is_woff(&[0x00, 0x01, 0x00, 0x00]));
        assert!(!is_woff(b"wOF2..."));
        assert!(!is_woff(&[]));
    }

    #[test]
    fn decode_real_woff2_ic_test_font() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wpt-runner/wpt-data/css/css-values/resources/IcTestFullWidth.woff2");
        let data = match std::fs::read(&path) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("skip: IcTestFullWidth.woff2 not present");
                return;
            }
        };
        assert!(is_woff2(&data));
        let sfnt = decode_woff2(&data).expect("WOFF2 decode should succeed");
        assert!(
            fontdue::Font::from_bytes(sfnt.as_slice(), fontdue::FontSettings::default()).is_ok(),
            "fontdue should load decoded WOFF2"
        );
        let mut loader = crate::font::loader::FontLoader::new();
        let font_id = loader.load_font(&data).expect("FontLoader should accept raw WOFF2");
        assert!(loader.measure_advance(font_id, '\u{6c34}', 20.0) > 0.0);
    }

    /// 残缺数据（魔数正确但长度不足）应返回 None，不 panic。
    #[test]
    fn decode_truncated_returns_none() {
        let mut truncated = vec![b'w', b'O', b'F', b'F'];
        truncated.extend_from_slice(&[0u8; 10]); // 不足 44 字节
        assert!(decode_woff(&truncated).is_none());
    }
}

#[cfg(test)]
mod r1007_tests {
    use super::*;
    #[test]
    fn decode_bundled_wpt_fonts() {
        let fonts = [
            "tests/wpt-runner/wpt-data/fonts/mplus-1p-regular.woff",
            "tests/wpt-runner/wpt-data/fonts/Scheherazade-Regular.woff",
            "tests/wpt-runner/wpt-data/fonts/sileot-webfont.woff",
            "tests/wpt-runner/wpt-data/fonts/noto/noto-sans-v8-latin-regular.woff",
            "tests/wpt-runner/wpt-data/fonts/math/mathvariant-italic.woff",
        ];
        for f in &fonts {
            let data = match std::fs::read(f) {
                Ok(d) => d,
                Err(_) => continue,
            };
            assert!(is_woff(&data), "{f} 应为 WOFF");
            let sfnt = decode_woff(&data).unwrap_or_else(|| panic!("decode 失败: {f}"));
            assert!(
                fontdue::Font::from_bytes(sfnt.as_slice(), fontdue::FontSettings::default()).is_ok(),
                "{f} fontdue 加载失败"
            );
        }
    }
}
