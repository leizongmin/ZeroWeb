//! R4270（filter-effects-1 §15 Privacy Considerations）：SVG filter tainting 规则的
//! 序列化源级静态分析。
//!
//! usvg/resvg 不实现 taint 规则，对受限原语 feDisplacementMap 一律执行位移；spec 要求
//! 位移映射（in2）为 tainted 输入时该原语作 **pass through filter**（输出 = 主输入 in，
//! 位移不生效）。ZW 的 SVG 绘制走「序列化 DOM 子树 → resvg::render」路径，本模块在
//! 序列化文本上做 filter 链静态分析：按文档序走查原语、建模 taint 传播、把命中的
//! feDisplacementMap 改写为恒等 feOffset（`<feOffset in="{in}" dx="0" dy="0"/>`——
//! resvg 对 feOffset 的实现 = in 原样平移 0，视觉恒等 pass through；result 属性保留
//! 使下游引用不断链；in 未指定时改写体同样省略，由 §9.2 默认规则承载「输出 = 主输入」）。
//!
//! 输入默认规则（§9.2 公共属性）：`in`/`in2` 未指定时，首个原语用 SourceGraphic、
//! 后续原语用**前一原语的结果**——二者各自独立解析该默认，`in2` 未指定 **不是** 取
//! `in` 的值（R4271 修正：首版误判致 no-taint 位移被恒改写为 pass through）。
//!
//! taint 源（§15.1）：①feFlood/feDropShadow 的 flood-color 计算为 currentColor；
//! ②feDiffuseLighting/feSpecularLighting 的 lighting-color 计算为 currentColor；
//! ③feImage（url 引用元素或 No-CORS——源级无法判定 CORS，保守全 tainted）；④标准输入
//! SourceGraphic/SourceAlpha/BackgroundImage/BackgroundAlpha/FillPaint/StrokePaint 恒
//! tainted。传播按**像素数据流**：输出携带输入像素的原语，任一输入 tainted → 结果
//! tainted；feFlood/feImage 输出与输入无关，其结果 taint 仅由自身规则决定（§15.1
//! 「有 tainted 输入即 tainted」按字面套在默认 SourceGraphic 输入上会与 stripe ref
//! 矛盾，R4271 精化）。
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
            // 输入 taint 判定。显式引用：标准输入恒 tainted，或已传播的 tainted 命名
            // 结果。未指定（§9.2 公共属性默认规则）：首个原语默认 SourceGraphic（标准
            // 输入，恒 tainted）；后续原语默认**前一原语的结果**，其 taint 由隐式链布尔
            // 携带——in 与 in2 各自独立取该默认，in2 未指定 **不是** 取 in 的值（首版
            // 误把 in2 默认为 in，使 `in="SourceGraphic"` 无 in2 的位移恒按标准输入
            // tainted 走 pass through，17 案 no-taint 变体被误改写，R4271 修正）。
            let input_tainted = |attr: Option<&String>| match attr {
                Some(n) => is_standard_input(n) || tainted.contains(n),
                None => first_primitive || last_implicit_tainted,
            };
            // 自 taint 源（§15.1 规则 1-5）。
            let self_tainted = match name {
                "feImage" => true,
                "feFlood" | "feDropShadow" => color_prop_is_currentcolor(&attrs, "flood-color"),
                "feDiffuseLighting" | "feSpecularLighting" => color_prop_is_currentcolor(&attrs, "lighting-color"),
                _ => false,
            };
            let in_tainted = input_tainted(in_attr.as_ref());
            let in2_tainted = input_tainted(in2_attr.as_ref());

            // §15.2：位移映射（in2）tainted → pass through（输出 = in）。
            if name == "feDisplacementMap" && in2_tainted {
                // 改写体省略 in 时，feOffset 同样按 §9.2 默认规则取前一原语结果——
                // 与被改写原语的默认主输入精确一致（未命名结果无名字可写）。
                let mut replacement = String::from("<feOffset");
                if let Some(n) = &in_attr {
                    replacement.push_str(&format!(" in=\"{n}\""));
                }
                replacement.push_str(" dx=\"0\" dy=\"0\"");
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

            // 一般 taint 传播：任一输入 tainted（含隐式默认输入）或自身 taint 源 →
            // 结果 tainted。例外：feFlood/feImage 的输出与输入像素无关（纯色 / 外部
            // 图像），其结果 taint 仅由自身规则决定——若按 §15.1 字面「有 tainted 输入
            // 即 tainted」把默认 SourceGraphic 输入传入 feFlood，则 no-taint 链（如
            // feFlood 常规色 → feDisplacementMap）会被恒判 tainted，与 stripe ref
            // 矛盾（tainting-fe*-001 族实证，R4271）。
            let result_tainted = if matches!(name, "feFlood" | "feImage") {
                self_tainted
            } else {
                in_tainted || in2_tainted || self_tainted
            };
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

fn is_standard_input(n: &str) -> bool {
    STANDARD_INPUTS.contains(&n)
}

/// flood-color/lighting-color 有效值是否「computes to currentColor」（§15.1）：值文本
/// 任意位置含 currentcolor 即判 tainted——`color-mix(in srgb, currentcolor …)` /
/// `contrast-color(currentcolor)` 等函数内嵌 currentcolor 的计算值同样依赖 color
/// 属性（保守超集，tainting-feflood-003/004 实证）。级联取值：内联 style 声明优先于
/// presentation 属性——JS 动态变更（`style.floodColor = …`）的序列化落点即 style
/// 声明（tainting-feflood-dynamic-001/002 实证）。
fn color_prop_is_currentcolor(attrs: &std::collections::HashMap<String, String>, prop: &str) -> bool {
    if let Some(style) = attrs.get("style") {
        let ci = style.to_ascii_lowercase();
        let pat = format!("{prop}:");
        if let Some(rel) = ci.find(&pat) {
            let vs = rel + pat.len();
            let ve = ci[vs..].find(';').map_or(ci.len(), |e| vs + e);
            return ci[vs..ve].contains("currentcolor");
        }
    }
    attrs
        .get(prop)
        .is_some_and(|v| v.to_ascii_lowercase().contains("currentcolor"))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// R4271（§9.2 公共属性默认规则）：`in2` 未指定 = 前一原语的结果，**不是** `in`
    /// 的值。feFlood 常规色非 taint 源 → 隐式链 untainted → 位移应照常执行，不得改写
    /// （tainting-fe*-001 族 17 案的驱动形态，ref = 位移后的 stripe）。
    #[test]
    fn in2_defaults_to_previous_result_not_in_value() {
        let src = r#"<filter id="f" color-interpolation-filters="sRGB"><feFlood flood-color="rgb(0%, 100%, 50%)"/><feDisplacementMap in="SourceGraphic" xChannelSelector="G" yChannelSelector="B" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(out.contains("feDisplacementMap"), "no-taint 位移不应被改写: {out}");
        assert!(!out.contains("feOffset"), "不应出现改写体: {out}");
    }

    /// R4270 回归守护：currentcolor feFlood 为 taint 源 → feOffset 隐式链传播 →
    /// feDisplacementMap 的 in2（隐式 = 前一结果）tainted → 改写为恒等 feOffset，
    /// 显式 in 与 result 引用保留。
    #[test]
    fn currentcolor_flood_taints_implicit_chain_into_pass_through() {
        let src = r#"<filter id="f"><feFlood flood-color="currentcolor"/><feOffset/><feDisplacementMap in="SourceGraphic" result="out" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "tainted 位移应被改写: {out}");
        assert!(out.contains("in=\"SourceGraphic\""), "主输入须保留: {out}");
        assert!(out.contains("result=\"out\""), "result 须保留使下游引用不断链: {out}");
        assert!(out.contains("dx=\"0\" dy=\"0\""), "{out}");
    }

    /// 首个原语位移且无 in/in2：两者默认 SourceGraphic（标准输入恒 tainted）→ 改写。
    #[test]
    fn first_primitive_displacement_defaults_to_tainted_sourcegraphic() {
        let src = r#"<filter id="f"><feDisplacementMap scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "标准输入位移应被改写: {out}");
        assert!(out.contains("<feOffset dx=\"0\" dy=\"0\"/>"), "{out}");
    }

    /// 隐式主输入 + tainted 隐式 in2：改写体省略 in，由 §9.2 默认规则承载
    /// 「pass through 输出 = 主输入」（未命名结果无名字可写）。
    #[test]
    fn tainted_implicit_chain_rewrite_omits_in() {
        let src = r#"<filter id="f"><feFlood flood-color="currentcolor"/><feDisplacementMap scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "{out}");
        assert!(out.contains("<feOffset dx=\"0\" dy=\"0\"/>"), "{out}");
        assert!(!out.contains("in=\"\""), "不得产出空引用: {out}");
    }

    /// in2 显式引用 untainted 命名结果（即使 in = SourceGraphic 恒 tainted）→
    /// §15.2 仅约束 in2，位移照常执行，不得改写。
    #[test]
    fn untainted_named_in2_keeps_displacement() {
        let src = r#"<filter id="f"><feFlood flood-color="green" result="fl"/><feDisplacementMap in="SourceGraphic" in2="fl" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(out.contains("feDisplacementMap"), "in2 untainted 不应改写: {out}");
    }

    /// in2 显式引用 tainted 命名结果（feImage 恒 taint 源）→ 改写；pass through 后
    /// 该 result 的 taint 随主输入（此处 in 亦 tainted，命名结果保持 tainted）。
    #[test]
    fn explicit_tainted_in2_rewrites_and_propagates_result_taint() {
        let src = r##"<filter id="f"><feImage href="#a" result="img"/><feDisplacementMap in="img" in2="img" result="disp" scale="100"/><feComposite in="disp"/></filter>"##;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "{out}");
        assert!(out.contains("in=\"img\""), "{out}");
    }

    /// R4271（§15.1「computes to currentColor」）：color-mix()/contrast-color() 内嵌
    /// currentcolor 的 flood-color/lighting-color 同为 taint 源（首版仅匹配裸关键字
    /// 漏检，tainting-feflood-003/004、tainting-fespecularlighting-004 形态）。
    #[test]
    fn currentcolor_embedded_in_color_function_taints() {
        let src = r#"<filter id="f"><feFlood flood-color="color-mix(in srgb, currentcolor 99.9%, black)"/><feDisplacementMap in="SourceGraphic" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "{out}");
        let src = r#"<filter id="f"><feSpecularLighting lighting-color="contrast-color(currentcolor)"><feDistantLight elevation="90"/></feSpecularLighting><feDisplacementMap in="SourceGraphic" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "{out}");
    }

    /// R4271：内联 style 声明优先于 presentation 属性（级联）——JS 动态变更
    /// `style.floodColor = 'currentcolor'` 的序列化落点是 style 声明
    /// （tainting-feflood-dynamic-001/002 形态）；style 未声明该属性时回落属性值，
    /// 无关属性（color）含 currentcolor 不影响判定。
    #[test]
    fn inline_style_declaration_overrides_presentation_attribute() {
        let src = r#"<filter id="f"><feFlood flood-color="rgb(0%, 100%, 50%)" style="color: rgb(0%, 100%, 50%); flood-color: currentcolor"/><feDisplacementMap in="SourceGraphic" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(!out.contains("feDisplacementMap"), "{out}");
        let src = r#"<filter id="f"><feFlood flood-color="rgb(0%, 100%, 50%)" style="color: currentcolor; flood-color: rgb(0%, 100%, 50%)"/><feDisplacementMap in="SourceGraphic" scale="100"/></filter>"#;
        let out = apply_svg_filter_taint_rules(src);
        assert!(out.contains("feDisplacementMap"), "常规 style 值不应判 tainted: {out}");
    }
}
