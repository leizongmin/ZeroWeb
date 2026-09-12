//! R4274（SVG2 shapes/paths 渲染禁用规则）：零尺寸 / 无几何的 SVG 形状元素**整体
//! 禁用渲染**——含其 filter 输出（`<rect width="0">` 等「A computed value of zero
//! for either dimension disables rendering of the element」，SVG2 §shapes；
//! path 空 d / polygon·polyline 空 points 同理 §paths#PathDataBNF）。
//!
//! usvg/resvg 不实现该禁用规则：对挂 filter 的禁用形状仍执行 filter 链（feFlood
//! 在 filter region 内产出纯色块），违背禁用语义（svg-empty-element-with-filter-001
//! ref = 全白实证）。ZW 的 SVG 绘制走「序列化 DOM 子树 → resvg::render」路径，本
//! 模块在序列化文本上做静态走查：显式零尺寸 attr / 空几何 attr 的形状元素整元素
//! 移除（含非自闭合的闭标签），使 resvg 无从渲染。
//!
//! 保守边界：仅匹配 **attr 显式** 给出的零值 / 空串（CSS 几何属性 ZW SVG 路径本就
//! 不支持，attr 缺省场景 resvg 自身按零处理，无需移除也不会产出 filter 输出差异——
//! 故 rect/image 缺省 width/height、circle 缺省 r 不在移除之列，防误伤）。
//!
//! kill-switch：env `ZW_SVG_SHAPE_DISABLE=0`（default-on）。

/// 需走查的形状元素（SVG2 渲染禁用语义覆盖面）。
const SHAPES: [&str; 7] = ["rect", "circle", "ellipse", "path", "polygon", "polyline", "image"];

/// 对序列化 SVG 源应用形状禁用规则（移除禁用元素）。
pub(crate) fn disable_empty_shapes(source: &str) -> String {
    if std::env::var("ZW_SVG_SHAPE_DISABLE").as_deref() == Ok("0") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // 标签名：字母直到空白/'/'/'>'。
            let name_start = i + 1;
            let mut name_end = name_start;
            while name_end < bytes.len() && bytes[name_end].is_ascii_alphanumeric() {
                name_end += 1;
            }
            let name = &source[name_start..name_end];
            // 标签结束 '>'（引号内含 '>' 极罕见，接受与 svg_filter_taint 同一近似）。
            let Some(gt_rel) = source[name_end..].find('>') else {
                break;
            };
            let gt = name_end + gt_rel;
            let tag_text = &source[i..=gt];
            if SHAPES.contains(&name) && shape_disabled(name, tag_text) {
                // 整元素移除：跳过开标签；非自闭合连跳闭标签（DOM 序列化产开+闭对）。
                let mut next = gt + 1;
                if !tag_text.ends_with("/>") {
                    let close = format!("</{name}>");
                    if let Some(crel) = source[next..].find(close.as_str()) {
                        next += crel + close.len();
                    }
                }
                i = next;
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

/// 形状是否被禁用（SVG2：零尺寸 / 空几何 attr）。仅看**显式** attr——
/// attr 缺省（几何属性本就缺省 0）不在移除之列，见模块注。
fn shape_disabled(name: &str, tag_text: &str) -> bool {
    let attrs = parse_attrs(tag_text);
    // 长度 attr 显式 0（"0" / "0px" / "0%" 均视为零）。
    let zero = |attr: &str| -> bool {
        attrs
            .get(attr)
            .and_then(|v| {
                v.trim()
                    .trim_end_matches("px")
                    .trim_end_matches('%')
                    .parse::<f32>()
                    .ok()
            })
            .is_some_and(|n| n == 0.0)
    };
    match name {
        // rect/image：width 或 height 显式 0（SVG2 §rect/§image rendering disable）。
        "rect" | "image" => zero("width") || zero("height"),
        "circle" => zero("r"),
        "ellipse" => zero("rx") || zero("ry"),
        // path：d 缺省/空/none → 无段（SVG2 §paths）。
        "path" => match attrs.get("d") {
            Some(d) => d.trim().is_empty() || d.eq_ignore_ascii_case("none"),
            None => true,
        },
        // polygon/polyline：points 缺省/空 → 空几何。
        "polygon" | "polyline" => match attrs.get("points") {
            Some(p) => p.trim().is_empty(),
            None => true,
        },
        _ => false,
    }
}

/// 从标签文本提取属性（name="value" / name='value'；与 svg_filter_taint 同一实现）。
fn parse_attrs(tag: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let bytes = tag.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'=' && i > 0 {
            let mut name_start = i;
            while name_start > 0 && !bytes[name_start - 1].is_ascii_whitespace() && bytes[name_start - 1] != b'<' {
                name_start -= 1;
            }
            let attr = tag[name_start..i].trim();
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                let quote = bytes[j];
                let Some(end_rel) = tag[j + 1..].find(quote as char) else {
                    break;
                };
                let value = &tag[j + 1..j + 1 + end_rel];
                if !attr.is_empty() && attr.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':') {
                    map.insert(attr.to_ascii_lowercase(), value.to_string());
                }
                i = j + 1 + end_rel + 1;
                continue;
            }
        }
        i += 1;
    }
    map
}

fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R4274：零尺寸形状（rect/circle/ellipse/image）与空几何（path/polygon/polyline）
    /// 整元素移除（含闭标签）；常规形状与非形状元素不动。
    #[test]
    fn empty_shapes_removed_and_normal_untouched() {
        let src = r#"<svg><rect x="10" y="10" width="0" height="0" fill="red" filter="url(#f)"></rect>
            <circle cx="40" cy="40" r="0" fill="red"/>
            <ellipse cx="50" cy="50" rx="0" ry="20" fill="red"/>
            <path d="" fill="red"/>
            <path fill="red"></path>
            <polygon points="" fill="red"/>
            <polyline points="" fill="none" stroke="red" filter="url(#f)"/>
            <image x="10" y="10" width="10" height="0" filter="url(#f)"/>
            <rect x="1" y="1" width="10" height="10" fill="blue"/>
            <path d="M0 0 L10 10" stroke="red"/>
            <g filter="url(#f)"></g>
        </svg>"#;
        let out = disable_empty_shapes(src);
        assert!(!out.contains(r#"width="0""#), "零宽 rect 应移除: {out}");
        assert!(!out.contains("circle"), "r=0 circle 应移除: {out}");
        assert!(!out.contains(r#"rx="0""#), "rx=0 ellipse 应移除: {out}");
        assert_eq!(
            out.matches("<path").count(),
            1,
            "空 d/缺省 d path 应移除，仅留常规 path: {out}"
        );
        assert!(!out.contains("polygon"), "{out}");
        assert!(!out.contains("polyline"), "{out}");
        assert!(!out.contains("<image"), "{out}");
        assert!(out.contains(r#"width="10" height="10""#), "常规 rect 应保留: {out}");
        assert!(out.contains(r#"d="M0 0 L10 10""#), "常规 path 应保留: {out}");
        assert!(out.contains("<g "), "非形状元素应保留: {out}");
    }

    /// 非零值 / 缺省 attr 不误伤（保守边界：仅显式零/空移除）。
    #[test]
    fn nonzero_and_missing_attrs_kept() {
        let src = r#"<svg><rect width="10" height="0.5"/><rect/><circle r="1"/><path d="none"/></svg>"#;
        let out = disable_empty_shapes(src);
        assert!(out.contains(r#"width="10""#), "{out}");
        assert!(
            out.contains("<rect/>"),
            "缺省 width/height 的 rect 不移除（保守）: {out}"
        );
        assert!(out.contains(r#"r="1""#), "{out}");
        assert!(!out.contains(r#"d="none""#), "d=none path 应移除: {out}");
    }
}
