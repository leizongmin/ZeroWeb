//! R5124（filter-effects-1 §7.1/§7.2）：CSS filter **函数形式** `filter` 属性的
//! 序列化改写——`filter="drop-shadow(50px 0 green)"` → 等价 `<filter>` 元素 +
//! `url(#zwcssfN)` 引用。
//!
//! 根因（resvg 0.47 函数形式两个缺陷，standalone probe 实证）：
//! ① region 硬编码 bbox×(-50%..200%)（usvg parser/filter.rs `create_base_filter_func`
//!    自注 "fairly large... far from ideal" + TODO）——偏移超 bbox 50% 的阴影被裁剪：
//!    svg-mutation-group-position-changed dx=-50px、bbox 宽 50 → 阴影 [0,50) 仅剩
//!    [25,50)，[0,25) 保持红色（reftest 5000px diff 实证）。
//! ② feDropShadow sRGB 通道终点二次转换：`apply_drop_shadow` 尾部
//!    `match cs { SRGB => pixmap.into_srgb() }` 对已是 sRGB 的数据再按 linear→sRGB
//!    提亮——CSS green #008000 阴影渲染为 (0,188,0)。元素形式 + linearRGB 则正确
//!    ((0,128,0))——flood 色未线性化与终点转换两次误差相消（同为上游缺陷）。
//!
//! 改写形态：`<filter x="-200%" y="-200%" width="500%" height="500%"
//! color-interpolation-filters="sRGB">`（函数形式按 CSS 语义跑 sRGB）；drop-shadow
//! 手工展开为 feGaussianBlur(SourceAlpha)+feOffset+feFlood+feComposite(in)+feMerge
//! （filter-effects-1 #funcdef-filter-drop-shadow 等价形，绕开 ② 的 feDropShadow
//! 缺陷）；blur/hue-rotate 直接映射。混合列表中 url 项（如 `url(#hue) drop-shadow(...)`
//! ，single-to-multiple-002）：被引用 `<filter>` 的原语内联到链首。
//!
//! 保守边界（FIXME 留档）：
//! - currentcolor/color-mix()/relative color 阴影色不改写（需 CSS 计算域，attr 串
//!   不可达——drop-shadow-currentcolor-dynamic-002/003 另行挂账）。
//! - 偏移超 bbox 200% 的阴影仍会被 region 裁剪（region 为 bbox 相对量）。
//! - 零面积 bbox（objectBoundingBox region 无效）→ usvg 跳过 filter，与改写前
//!   函数形式行为一致。
//! - 出现未支持函数（brightness/contrast/grayscale/invert/opacity/saturate/sepia）
//!   即整属性不动（保留现行为）。
//! - url 项仅支持列表首位、子原语全部自闭合且无 result（corpus 形态）；其余不动。
//!
//! 与 taint（svg_filter_taint）/形状禁用（svg_shape_disable）的顺序：调用点在两者
//! 之后——内联 url 项取到的是 taint 改写后的原语，禁用形状已被移除。
//!
//! kill-switch：env `ZW_SVG_FILTER_FN=0`（default-on，R4270/R5123 同族）。

use zero_css_parser::values::{ColorValue, FilterValue, parse_filter_list};

/// 对序列化 SVG 源应用 CSS filter 函数形式改写。
pub(crate) fn rewrite_css_function_filters(source: &str) -> String {
    if std::env::var("ZW_SVG_FILTER_FN").as_deref() == Ok("0") {
        return source.to_string();
    }
    // 快速 bail：无 filter 属性或含保留 id 前缀（防与页面自定义 id 撞车）则不改写。
    if !source.contains(r#"filter=""#) || source.contains("zwcssf") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut body = String::with_capacity(source.len());
    let mut defs = String::new();
    let mut counter = 0u32;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let name_start = i + 1;
            let mut name_end = name_start;
            while name_end < bytes.len() && bytes[name_end].is_ascii_alphanumeric() {
                name_end += 1;
            }
            // 标签结束 '>'（引号内含 '>' 极罕见，接受与 R5123 同一近似）。
            let Some(gt_rel) = source[name_end..].find('>') else {
                break;
            };
            let gt = name_end + gt_rel;
            let tag_text = &source[i..=gt];
            if let Some(rewritten) = rewrite_tag_filter_attr(tag_text, source, &mut defs, &mut counter) {
                body.push_str(&rewritten);
            } else {
                body.push_str(tag_text);
            }
            i = gt + 1;
        } else {
            let ch_len = utf8_char_len(bytes[i]);
            body.push_str(&source[i..i + ch_len]);
            i += ch_len;
        }
    }
    if defs.is_empty() {
        return source.to_string();
    }
    // defs 注入根 `<svg>` 开标签之后（painter 序列化源以 `<svg` 开头）。
    let Some(gt) = source.find('>') else {
        return source.to_string();
    };
    let mut out = String::with_capacity(body.len() + defs.len() + 16);
    out.push_str(&body[..=gt]);
    out.push_str("<defs>");
    out.push_str(&defs);
    out.push_str("</defs>");
    out.push_str(&body[gt + 1..]);
    out
}

/// 改写单个标签的 `filter` 属性：含函数项 → `url(#zwcssfN)`，`<filter>` 定义追加进
/// `defs`。无需改写返回 `None`（原样保留）。
fn rewrite_tag_filter_attr(tag: &str, source: &str, defs: &mut String, counter: &mut u32) -> Option<String> {
    let attr_pos = crate::paint::svg_displacement_scale::find_attr_value_span(tag, "filter")?;
    let value = &tag[attr_pos.0..attr_pos.1];
    let items = parse_filter_list(value)?;
    // 纯 url / none 列表现行为正确（url-to-function-001 实证），不动。
    if items.iter().all(|it| matches!(it, FilterValue::Url(_))) {
        return None;
    }
    *counter += 1;
    let id = format!("zwcssf{counter}");
    let chain = build_chain(&items, source, &id)?;
    defs.push_str(&format!(
        r#"<filter id="{id}" x="-200%" y="-200%" width="500%" height="500%" color-interpolation-filters="sRGB">{chain}</filter>"#
    ));
    Some(format!("{}url(#{id}){}", &tag[..attr_pos.0], &tag[attr_pos.1..]))
}

/// 构建原语链（声明序交错应用，filter-effects-1 §7.1）。返回原语 XML；
/// result 命名以 `id` 限定（各 `<filter>` 元素独立作用域，跨元素不冲突）。
/// 不可表达的列表返回 `None`。
fn build_chain(items: &[FilterValue], source: &str, id: &str) -> Option<String> {
    let mut xml = String::new();
    // 链上游结果名：首项为 SourceGraphic；后续项消费前项 result。
    let mut prev = "SourceGraphic".to_string();
    for (idx, item) in items.iter().enumerate() {
        let n = idx + 1;
        match item {
            FilterValue::Url(fragment) => {
                // url 项仅支持链首（后续项需把引用原语的输出接入 prev 链，FIXME）。
                if prev != "SourceGraphic" {
                    return None;
                }
                let (inlined, handoff) = inline_url_filter(source, fragment, id, n)?;
                xml.push_str(&inlined);
                prev = handoff;
            }
            FilterValue::DropShadow(dx, dy, sigma, color) => {
                xml.push_str(&drop_shadow_chain(*dx, *dy, *sigma, color, &prev, id, n));
                prev = format!("{id}m{n}");
            }
            FilterValue::Blur(px) => {
                xml.push_str(&format!(
                    r#"<feGaussianBlur in="{prev}" stdDeviation="{} {}" result="{id}b{n}"/>"#,
                    fmt_num(*px),
                    fmt_num(*px)
                ));
                prev = format!("{id}b{n}");
            }
            FilterValue::HueRotate(deg) => {
                xml.push_str(&format!(
                    r#"<feColorMatrix in="{prev}" type="hueRotate" values="{}" result="{id}h{n}"/>"#,
                    fmt_num(*deg)
                ));
                prev = format!("{id}h{n}");
            }
            // 未支持函数：整属性不动（模块注保守边界）。
            _ => return None,
        }
    }
    if xml.is_empty() {
        return None;
    }
    Some(xml)
}

/// drop-shadow 展开链：blur(alpha) → offset → flood 染色 → in 复合 → 与上游合成。
/// 非首项时上游 alpha 需先经 feColorMatrix 抽取（SourceAlpha 恒取原始画布，
/// filter-effects-1 #funcdef-filter-drop-shadow 语义 = 前序结果的影子）。
fn drop_shadow_chain(dx: f32, dy: f32, sigma: f32, color: &ColorValue, prev: &str, id: &str, n: usize) -> String {
    let flood = match flood_color_attrs(color) {
        Some(f) => f,
        None => return String::new(),
    };
    let mut out = String::new();
    let alpha_in = if prev == "SourceGraphic" {
        "SourceAlpha".to_string()
    } else {
        out.push_str(&format!(
            r#"<feColorMatrix in="{prev}" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0" result="{id}a{n}"/>"#
        ));
        format!("{id}a{n}")
    };
    let (ox, oy) = (fmt_num(dx), fmt_num(dy));
    let sd = fmt_num(sigma);
    out.push_str(&format!(
        r#"<feGaussianBlur in="{alpha_in}" stdDeviation="{sd} {sd}" result="{id}b{n}"/>"#
    ));
    out.push_str(&format!(
        r#"<feOffset in="{id}b{n}" dx="{ox}" dy="{oy}" result="{id}o{n}"/>"#
    ));
    out.push_str(&format!(r#"<feFlood {flood} result="{id}f{n}"/>"#));
    out.push_str(&format!(
        r#"<feComposite in="{id}f{n}" in2="{id}o{n}" operator="in" result="{id}s{n}"/>"#
    ));
    out.push_str(&format!(
        r#"<feMerge result="{id}m{n}"><feMergeNode in="{id}s{n}"/><feMergeNode in="{prev}"/></feMerge>"#
    ));
    out
}

/// 内联 `url(#fragment)` 引用的 `<filter>` 子原语（无 result、无元素子节点——
/// filter 原语均为空内容元素；DOM 序列化为显式闭标签形态）；最后一个原语追加
/// result 作为链交接名。返回（内联 XML, 交接名）。
fn inline_url_filter(source: &str, fragment: &str, id: &str, n: usize) -> Option<(String, String)> {
    let frag = fragment.strip_prefix('#').unwrap_or(fragment);
    if frag.is_empty() {
        return None;
    }
    // 定位 `<filter ... id="{frag}" ...>` 开标签（filter 元素不嵌套，spec）。
    let mut from = 0usize;
    while let Some(rel) = source[from..].find("<filter") {
        let start = from + rel;
        let rest = &source[start..];
        if !rest[7..].starts_with(|c: char| c.is_ascii_whitespace()) {
            from = start + 7;
            continue;
        }
        let gt = rest.find('>')?;
        let open_tag = &rest[..=gt];
        let is_target = crate::paint::svg_displacement_scale::find_attr_value_span(open_tag, "id")
            .is_some_and(|(s, e)| &open_tag[s..e] == frag);
        if !is_target {
            from = start + 7;
            continue;
        }
        let close_rel = rest[gt..].find("</filter>")?;
        let inner = rest[gt + 1..gt + close_rel].trim();
        if inner.contains("result=") {
            return None;
        }
        let handoff = format!("{id}i{n}");
        let inlined = append_result_attr(inner, &handoff)?;
        return Some((inlined, handoff));
    }
    None
}

/// 把 `result="{name}"` 插入 inner 最后一个子原语开标签的属性区（闭标签形态由
/// 逐子元素扫描定位；子原语含元素子节点则 bail——FIXME 留档）。
fn append_result_attr(inner: &str, name: &str) -> Option<String> {
    let bytes = inner.as_bytes();
    let mut last_open_gt: Option<usize> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let name_start = i + 1;
            let mut name_end = name_start;
            while name_end < bytes.len() && bytes[name_end].is_ascii_alphanumeric() {
                name_end += 1;
            }
            let tag_name = &inner[name_start..name_end];
            let gt_rel = inner[name_end..].find('>')?;
            let gt = name_end + gt_rel;
            if inner[..=gt].ends_with("/>") {
                // 自闭合：属性区即开标签本身。
                last_open_gt = Some(gt);
                i = gt + 1;
            } else {
                // 显式闭标签：内容含 '<' = 元素子节点，bail。
                let close = format!("</{tag_name}>");
                let close_rel = inner[gt..].find(&close)?;
                let content = &inner[gt + 1..gt + close_rel];
                if content.contains('<') {
                    return None;
                }
                last_open_gt = Some(gt);
                i = gt + close_rel + close.len();
            }
        } else {
            // 子原语间空白/文本。
            let ch_len = utf8_char_len(bytes[i]);
            i += ch_len;
        }
    }
    let gt = last_open_gt?;
    Some(format!("{} result=\"{name}\"{}", &inner[..gt], &inner[gt..]))
}

/// ColorValue → feFlood 的 flood-color(/flood-opacity) 属性串。
/// currentcolor 系（CurrentColor/Mix/RelativeColor/ContrastColor）返回 `None`
/// （需 CSS 计算域解析，attr 串不可达——模块注 FIXME）。
fn flood_color_attrs(color: &ColorValue) -> Option<String> {
    let (r, g, b, a) = match color {
        ColorValue::Named(name) => return Some(format!(r#"flood-color="{name}""#)),
        ColorValue::Transparent => (0, 0, 0, 0u8),
        ColorValue::Rgba(r, g, b, a) => (*r, *g, *b, *a),
        ColorValue::Hsla(h, s, l, a) => {
            let c = crate::paint::color::hsla_to_rgba(*h, *s, *l, *a);
            (c.r, c.g, c.b, c.a)
        }
        ColorValue::CurrentColor | ColorValue::Mix(_) | ColorValue::RelativeColor(_) | ColorValue::ContrastColor(_) => {
            return None;
        }
    };
    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    if a == 255 {
        Some(format!(r#"flood-color="{hex}""#))
    } else {
        Some(format!(
            r#"flood-color="{hex}" flood-opacity="{}""#,
            fmt_num(a as f32 / 255.0)
        ))
    }
}

/// f32 → 最短十进制串（整数去 `.0`；小数原样）。
/// 数值 → 紧凑文本（整数去小数尾零）。R5125 起供 painter 计算值 filter 注入复用。
pub(crate) fn fmt_num(v: f32) -> String {
    if v.is_finite() && (v - v.round()).abs() < f32::EPSILON {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
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

    const NS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">"#;

    #[test]
    fn drop_shadow_attr_rewritten() {
        let src = format!(
            r#"{NS}<rect width="100" height="100" fill="red"/><g filter="drop-shadow(-50px 0 green)"><rect width="50" height="100" fill="green"/></g></svg>"#
        );
        let out = rewrite_css_function_filters(&src);
        assert!(out.contains(r#"filter="url(#zwcssf1)""#), "{out}");
        assert!(out.contains(r#"<defs><filter id="zwcssf1" x="-200%""#), "{out}");
        assert!(out.contains(r#"color-interpolation-filters="sRGB""#), "{out}");
        assert!(out.contains(r##"feFlood flood-color="#008000""##), "{out}");
        assert!(out.contains(r#"<feGaussianBlur in="SourceAlpha""#), "{out}");
        assert!(out.contains(r#"<feMergeNode in="SourceGraphic"/>"#), "{out}");
        // defs 注入在根 svg 开标签后。
        let defs_at = out.find("<defs>").unwrap();
        let svg_gt = out.find('>').unwrap();
        assert!(defs_at > svg_gt, "{out}");
    }

    #[test]
    fn pure_url_and_none_untouched() {
        let src = format!(r#"{NS}<rect filter="url(#f)"/><rect filter="none"/></svg>"#);
        assert_eq!(rewrite_css_function_filters(&src), src);
    }

    #[test]
    fn hue_rotate_then_drop_shadow_chains() {
        let src = format!(
            r#"{NS}<rect width="50" height="100" fill="rgb(71.79%, 28.82%, 0%)" filter="hue-rotate(90deg) drop-shadow(50px 0 green)"/></svg>"#
        );
        let out = rewrite_css_function_filters(&src);
        assert!(
            out.contains(r#"type="hueRotate" values="90" result="zwcssf1h1""#),
            "{out}"
        );
        // 非首项 drop-shadow：alpha 抽取自前序 result，merge 上游节点接前序。
        assert!(out.contains(r#"in="zwcssf1h1" type="matrix""#), "{out}");
        assert!(out.contains(r#"<feMergeNode in="zwcssf1h1"/>"#), "{out}");
    }

    #[test]
    fn mixed_url_first_inlines_referenced_primitives() {
        // DOM 序列化形态：子原语为显式闭标签（outer_html 不产出自闭合）。
        let src = format!(
            r#"{NS}<filter id="hue" color-interpolation-filters="sRGB"><feColorMatrix type="hueRotate" values="90"></feColorMatrix></filter><rect width="50" height="100" filter="url(#hue) drop-shadow(50px 0 green)"></rect></svg>"#
        );
        let out = rewrite_css_function_filters(&src);
        assert!(out.contains(r#"url(#zwcssf1)"#), "{out}");
        // 引用原语内联且获得链交接 result（插入闭标签形态的开标签属性区）。
        assert!(
            out.contains(r#"<feColorMatrix type="hueRotate" values="90" result="zwcssf1i1"></feColorMatrix>"#),
            "{out}"
        );
        // url 项在链首 → blur 直接用 SourceAlpha（上游=内联结果，alpha 等价）。
        assert!(out.contains(r#"stdDeviation="0 0" result="zwcssf1b2""#), "{out}");
        assert!(out.contains(r#"<feMergeNode in="zwcssf1i1"/>"#), "{out}");
    }

    #[test]
    fn currentcolor_and_unsupported_bail() {
        let cc = format!(r#"{NS}<rect filter="drop-shadow(50px 0 currentcolor)"/></svg>"#);
        assert_eq!(rewrite_css_function_filters(&cc), cc);
        let unsup = format!(r#"{NS}<rect filter="blur(2px) sepia(0.5)"/></svg>"#);
        assert_eq!(rewrite_css_function_filters(&unsup), unsup);
    }

    // kill-switch（ZW_SVG_FILTER_FN=0）不做单测：测试进程内 set/remove env 与并行
    // 测试的 env 读取存在竞态（R4270/R5123 同族均不做 kill-switch 单测）。
}
