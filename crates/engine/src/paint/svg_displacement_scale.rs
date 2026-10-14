//! R5123（filter-effects-1 §feDisplacementMap）：resvg 0.47 位移 scale **平方**
//! 缺陷的序列化补偿——`<feDisplacementMap scale="N">` 改写为 `scale="√N"`。
//!
//! usvg/resvg 路径：usvg 解析 `scale` attr × primitiveUnits 变量（缺省
//! userSpaceOnUse → ×1）存入 `DisplacementMap.scale`；resvg 渲染时
//! `scale_coordinates()` 先算 `sx = fe.scale × ts_scale`，`displacement_map::apply`
//! 再乘 `dx × sx × fe.scale()`——`fe.scale` 被乘了**两次**，恒等画布变换下有效位移
//! = `(channel − 0.5) × scale²`（spec：`× scale`，filter-effects-1
//! #feDisplacementMapElementAttribute）。实证：tainting-feflood-001（scale=100）
//! resvg 单独栅格化全透明（5000px 位移出界）；scale=3/5/8 量得色标边界位移 5/13/32px
//! = 0.5×scale² 精确吻合。上游 resvg 0.47 未修（usvg 同文件 TODO 自认
//! "resvg doesn't support displacement map anyway"）。
//!
//! 补偿数学：序列化 `√N` 后有效位移 = `(ch − 0.5) × (√N)² × ts_scale = (ch − 0.5)
//! × N × ts_scale`——与 spec 的 user 单位位移 × 画布缩放精确一致（含非恒等 ts）。
//! scale=0/1 平方不变（零扰动面）。
//!
//! 保守边界：
//! - **负 scale 不改写**（FIXME）：平方丢符号，resvg 对负 scale 位移方向恒正，
//!   与 spec（负 = 左/上）方向相反，attr 侧补偿不可达——保持原样（既有红，
//!   effect-reference-displacement-negative-scale-001）。
//! - attr 缺省 = 0（resvg 语义）不改写；非有限数 / 解析失败不改写。
//! - `primitiveUnits="objectBoundingBox"` 页（usvg 再乘 bbox 均值，平方后 ×bbox²）
//!   补偿不精确——ZW 页面域未见使用，FIXME 留档。
//!
//! 与 taint 改写（`svg_filter_taint`）的顺序：调用点在 taint 之后——被 taint 改写
//! 为 feOffset 的原语不再含 feDisplacementMap 标签，天然跳过。
//!
//! kill-switch：env `ZW_SVG_DISP_SCALE=0`（default-on，与 R4270/R4274 同族）。

/// 对序列化 SVG 源应用位移 scale 平方补偿（√N 改写）。
pub(crate) fn compensate_displacement_scale(source: &str) -> String {
    if std::env::var("ZW_SVG_DISP_SCALE").as_deref() == Ok("0") {
        return source.to_string();
    }
    // 快速 bail：无位移映射原语则无可改写（与 svg_filter_taint 同款）。
    if !source.contains("feDisplacementMap") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let name_start = i + 1;
            let mut name_end = name_start;
            while name_end < bytes.len() && bytes[name_end].is_ascii_alphanumeric() {
                name_end += 1;
            }
            let name = &source[name_start..name_end];
            // 标签结束 '>'（引号内含 '>' 极罕见，接受与 svg_shape_disable 同一近似）。
            let Some(gt_rel) = source[name_end..].find('>') else {
                break;
            };
            let gt = name_end + gt_rel;
            let tag_text = &source[i..=gt];
            if name == "feDisplacementMap"
                && let Some(compensated) = compensate_tag_scale(tag_text)
            {
                out.push_str(&compensated);
                i = gt + 1;
                continue;
            }
            out.push_str(tag_text);
            i = gt + 1;
        } else {
            let ch_len = utf8_char_len(bytes[i]);
            out.push_str(&source[i..i + ch_len]);
            i += ch_len;
        }
    }
    out
}

/// 改写单个 feDisplacementMap 开标签的 scale attr：`√N`（仅正有限 N）。
/// 无 scale attr / 解析失败 / 非正数 → `None`（原样保留）。
fn compensate_tag_scale(tag: &str) -> Option<String> {
    let attr_pos = find_attr_value_span(tag, "scale")?;
    let raw = &tag[attr_pos.0..attr_pos.1];
    let n: f64 = raw.trim().parse().ok()?;
    // 负 scale：平方丢符号，attr 侧不可补偿（模块注 FIXME）；0/1 平方不变。
    if !n.is_finite() || n <= 0.0 || (n - 1.0).abs() < f64::EPSILON {
        return None;
    }
    let sqrt = n.sqrt();
    // 6 位小数：平方还原误差 ~1e-6 量级，远低于像素分辨率。
    let mut new_attr = format!("{sqrt:.6}");
    if new_attr.ends_with('0') {
        new_attr = new_attr.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    Some(format!("{}{}{}", &tag[..attr_pos.0], new_attr, &tag[attr_pos.1..]))
}

/// 定位标签文本中 `name="value"` 的 value span（`[start, end)`，不含引号）；
/// 仅双引号（DOM 序列化形态），且 attr 名前须为空白（防值内误配）。找不到返回 `None`。
/// R5124 起供 svg_filter_functions 复用。
pub(crate) fn find_attr_value_span(tag: &str, attr: &str) -> Option<(usize, usize)> {
    let needle = format!("{attr}=\"");
    let mut from = 0usize;
    while let Some(rel) = tag[from..].find(&needle) {
        let hit = from + rel;
        if tag[..hit].ends_with(|c: char| c.is_ascii_whitespace()) {
            let vstart = hit + needle.len();
            let vend_rel = tag[vstart..].find('"')?;
            return Some((vstart, vstart + vend_rel));
        }
        from = hit + needle.len();
    }
    None
}

fn utf8_char_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_scale_gets_sqrt() {
        let src = r#"<feDisplacementMap in="SourceGraphic" scale="100"></feDisplacementMap>"#;
        let out = compensate_displacement_scale(src);
        assert!(out.contains(r#"scale="10""#), "{out}");
        // 平方还原 = 原值。
        let parsed: f64 = out
            .split("scale=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!((parsed * parsed - 100.0).abs() < 1e-4);
    }

    #[test]
    fn fractional_scale() {
        let out = compensate_displacement_scale(r#"<feDisplacementMap scale="0.5"/>"#);
        let parsed: f64 = out
            .split("scale=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!((parsed - 0.5_f64.sqrt()).abs() < 1e-4, "{out}");
    }

    #[test]
    fn zero_one_negative_and_missing_untouched() {
        for scale in ["0", "1", "-4"] {
            let src = format!(r#"<feDisplacementMap scale="{scale}"/>"#);
            assert_eq!(compensate_displacement_scale(&src), src, "scale={scale}");
        }
        let src = r#"<feDisplacementMap in="SourceGraphic"/>"#;
        assert_eq!(compensate_displacement_scale(src), src);
    }

    #[test]
    fn other_elements_untouched() {
        let src = r#"<filter><feFlood scale="100"/><feDisplacementMap scale="9"/></filter>"#;
        let out = compensate_displacement_scale(src);
        assert!(out.contains(r#"<feFlood scale="100"/>"#), "{out}");
        assert!(out.contains(r#"scale="3""#), "{out}");
    }

    #[test]
    fn no_displacement_bails_fast() {
        let src = r#"<filter><feFlood flood-color="red"/></filter>"#;
        assert_eq!(compensate_displacement_scale(src), src);
    }
}
