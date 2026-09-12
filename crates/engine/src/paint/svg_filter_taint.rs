//! R4270（filter-effects-1 §15 Privacy Considerations）：SVG filter tainting 规则的
//! 序列化源级静态分析。
//!
//! usvg/resvg 不实现 taint 规则，对受限原语 feDisplacementMap 一律执行位移；spec 要求
//! 位移映射（in2）为 tainted 输入时该原语作 **pass through filter**（输出 = 主输入 in，
//! 位移不生效）。ZW 的 SVG 绘制走「序列化 DOM 子树 → resvg::render」路径，本模块在
//! 序列化文本上做 filter 链静态分析：按文档序走查原语、建模 taint 传播、把命中的
//! feDisplacementMap 改写为恒等 feOffset（`<feOffset in="{in}" dx="0" dy="0"/>`——
//! resvg 对 feOffset 的实现 = in 原样平移 0，视觉恒等 pass through；result 属性保留
//! 使下游引用不断链）。
//!
//! taint 源（§15.1）：①feFlood/feDropShadow 的 flood-color 计算为 currentColor；
//! ②feDiffuseLighting/feSpecularLighting 的 lighting-color 计算为 currentColor；
//! ③feImage（url 引用元素或 No-CORS——源级无法判定 CORS，保守全 tainted）；④标准输入
//! SourceGraphic/SourceAlpha/BackgroundImage/BackgroundAlpha/FillPaint/StrokePaint 恒
//! tainted。传播：任何以 tainted 原语结果为输入的原语亦 tainted。
//!
//! kill-switch：env `ZW_SVG_TAINT=0`（default-on）。

use std::collections::HashSet;

/// 标准输入关键字（§15.1 规则 6：恒 tainted）。
const STANDARD_INPUTS: [&str; 6] = [
    "SourceGraphic",
    "SourceAlpha",
    "BackgroundImage",
    "BackgroundAlpha",
    "FillPaint",
    "StrokePaint",
];

/// 对序列化 SVG 源应用 tainting 规则（仅 feDisplacementMap 受限原语改写）。
pub(crate) fn apply_svg_filter_taint_rules(source: &str) -> String {
    if std::env::var("ZW_SVG_TAINT").as_deref() == Ok("0") {
        return source.to_string();
    }
    // 快速 bail：无位移映射原语则规则不可命中（taint 传播只影响 feDisplacementMap 行为）。
    if !source.contains("feDisplacementMap") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while let Some(fs) = source[cursor..].find("<filter") {
        let filter_start = cursor + fs;
        // `<filter` 后须非标签名字符（排除 `<filters` 等——SVG 无此元素，保守按前缀）。
        let Some(close_rel) = source[filter_start..].find("</filter>") else {
            break;
        };
        let block_end = filter_start + close_rel + "</filter>".len();
        // 块前原文照抄。
        out.push_str(&source[cursor..filter_start]);
        let block = &source[filter_start..block_end];
        out.push_str(&rewrite_filter_block(block));
        cursor = block_end;
    }
    out.push_str(&source[cursor..]);
    let _ = bytes;
    out
}

/// 走查单个 `<filter>…</filter>` 块，把 in2 tainted 的 feDisplacementMap 改写为恒等
/// feOffset。
fn rewrite_filter_block(block: &str) -> String {
    let mut tainted: HashSet<String> = HashSet::new();
    // 隐式链输入：前一个原语的结果名（无 result 属性的原语其输出仍作为下一原语的
    // 隐式输入，但没有可引用名——用哨兵名跟踪其 taint 传播）。
    let mut last_implicit_tainted = false;
    let mut first_primitive = true;

    let bytes = block.as_bytes();
    let mut out = String::with_capacity(block.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && block[i..].len() > 3 && block[i + 1..].starts_with("fe") {
            // 原语标签起点：找标签名（字母直到空白/'/'/'>'）。
            let name_start = i + 1;
            let mut name_end = name_start;
            while name_end < bytes.len() && (bytes[name_end].is_ascii_alphanumeric()) {
                name_end += 1;
            }
            let name = &block[name_start..name_end];
            // 找标签结束 '>'（引号内不含标签闭合——SVG 属性值含 '>' 极罕见，接受近似）。
            let Some(gt_rel) = block[name_end..].find('>') else {
                break;
            };
            let gt = name_end + gt_rel;
            let tag_text = &block[i..=gt];
            let self_closing = tag_text.ends_with("/>");

            // feMergeNode/feFunc*/fe*Light 非原语子元素跳过（feDistantLight 等以 'fe'
            // 开头但非原语；fePointLight/feSpotLight 同）。原语名单白名单判定。
            if !is_primitive(name) {
                // 照抄整个标签（非自闭合的 feMerge 等逐字复制开标签即可）。
                out.push_str(tag_text);
                i = gt + 1;
                continue;
            }

            let attrs = parse_attrs(tag_text);
            let result = attrs.get("result").map(|s| s.to_string());
            let in_attr = attrs.get("in").map(|s| s.to_string());
            let in2_attr = attrs.get("in2").map(|s| s.to_string());
            // 隐式输入：首个原语默认 SourceGraphic，后续默认前一原语输出。
            let implicit_input = if first_primitive {
                "SourceGraphic".to_string()
            } else {
                last_implicit_name(i)
            };
            let in_name = in_attr.clone().unwrap_or_else(|| implicit_input.clone());
            let in2_name = in2_attr.clone().unwrap_or_else(|| in_name.clone());

            let input_tainted = |n: &str| is_standard_input(n) || tainted.contains(n);
            // 自 taint 源（§15.1 规则 1-5）。
            let self_tainted = match name {
                "feImage" => true,
                "feFlood" | "feDropShadow" => attrs
                    .get("flood-color")
                    .is_some_and(|v| v.eq_ignore_ascii_case("currentcolor")),
                "feDiffuseLighting" | "feSpecularLighting" => attrs
                    .get("lighting-color")
                    .is_some_and(|v| v.eq_ignore_ascii_case("currentcolor")),
                _ => false,
            };
            let in_tainted = input_tainted(&in_name);
            let in2_tainted = input_tainted(&in2_name);

            // §15.2：位移映射（in2）tainted → pass through（输出 = in）。
            if name == "feDisplacementMap" && in2_tainted {
                let mut replacement = format!("<feOffset in=\"{}\" dx=\"0\" dy=\"0\"", in_name);
                if let Some(r) = &result {
                    replacement.push_str(&format!(" result=\"{}\"", r));
                }
                replacement.push_str("/>");
                out.push_str(&replacement);
                // 非自闭合原语（DOM 序列化产出开+闭标签对）：连带消耗闭标签，防悬空
                // `</feDisplacementMap>` 残留（首版实证残 tag 致 XML 失效整 SVG 不渲染）。
                let mut skip_to = gt + 1;
                if !self_closing && let Some(crel) = block[gt + 1..].find("</feDisplacementMap>") {
                    skip_to = gt + 1 + crel + "</feDisplacementMap>".len();
                }
                // pass through 输出 = in：result 的 taint 随 in。
                if let Some(r) = &result
                    && in_tainted
                {
                    tainted.insert(r.clone());
                }
                last_implicit_tainted = in_tainted;
                first_primitive = false;
                i = skip_to;
                continue;
            }

            // 一般 taint 传播：任一输入 tainted 或自身 taint 源 → 结果 tainted。
            let result_tainted =
                in_tainted || in2_tainted || self_tainted || last_implicit_tainted && in_attr.is_none();
            if let Some(r) = &result
                && result_tainted
            {
                tainted.insert(r.clone());
            }
            // 隐式链：未命名结果的输出成为下一原语隐式输入。
            last_implicit_tainted = if result.is_some() { false } else { result_tainted };
            first_primitive = false;

            out.push_str(tag_text);
            i = gt + 1;
        } else {
            // 非原语区域：逐字符照抄（含 </filter>、文本、其他元素）。
            let ch_len = utf8_char_len(bytes[i]);
            out.push_str(&block[i..i + ch_len]);
            i += ch_len;
        }
    }
    out
}

/// 隐式链输入名（引用 taint 集合用哨兵名——未命名结果不可被显式引用，taint 经
/// last_implicit_tainted 布尔传播，此名仅占位）。
fn last_implicit_name(_idx: usize) -> String {
    String::new()
}

fn is_standard_input(n: &str) -> bool {
    STANDARD_INPUTS.contains(&n)
}

/// 原语白名单（filter primitive，§4 Terminology；feMergeNode/feFunc*/光源子元素非原语）。
fn is_primitive(name: &str) -> bool {
    matches!(
        name,
        "feBlend"
            | "feColorMatrix"
            | "feComponentTransfer"
            | "feComposite"
            | "feConvolveMatrix"
            | "feDiffuseLighting"
            | "feDisplacementMap"
            | "feDropShadow"
            | "feFlood"
            | "feGaussianBlur"
            | "feImage"
            | "feMerge"
            | "feMorphology"
            | "feOffset"
            | "feSpecularLighting"
            | "feTile"
            | "feTurbulence"
    )
}

/// 从标签文本提取属性（name="value" / name='value'；style 属性整体保留待判定）。
fn parse_attrs(tag: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let bytes = tag.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'=' && i > 0 {
            // 回溯属性名。
            let mut name_start = i;
            while name_start > 0 && !bytes[name_start - 1].is_ascii_whitespace() && bytes[name_start - 1] != b'<' {
                name_start -= 1;
            }
            let attr = tag[name_start..i].trim();
            // 属性值。
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
