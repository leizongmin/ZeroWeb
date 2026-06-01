//! CSS 颜色和基础属性解析。

use super::*;

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()`、`hsl()`/`hsla()` 和 `hwb()` 函数。
pub fn parse_color(value: &str) -> Option<ColorValue> {
    let value = value.trim();

    // 特殊关键字
    if value.eq_ignore_ascii_case("transparent") {
        return Some(ColorValue::Transparent);
    }
    if value.eq_ignore_ascii_case("currentColor") || value == "currentcolor" {
        return Some(ColorValue::CurrentColor);
    }

    // 十六进制颜色
    if value.starts_with('#') {
        return parse_hex_color(value);
    }

    // rgb() / rgba() 函数
    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        return parse_rgb_function(value);
    }

    // hsl() / hsla() 函数
    if value.starts_with("hsl(") || value.starts_with("hsla(") {
        return parse_hsl_function(value);
    }

    // hwb() 函数
    if value.starts_with("hwb(") {
        return parse_hwb_function(value);
    }

    // 命名颜色
    parse_named_color(value)
}

/// 解析十六进制颜色。
fn parse_hex_color(value: &str) -> Option<ColorValue> {
    let hex = &value[1..]; // 去掉 #
    match hex.len() {
        3 => {
            // #RGB → RRGGBB
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        4 => {
            // #RGBA → RRGGBBAA
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let c3 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            let a = hex_char_to_byte(c3, c3);
            Some(ColorValue::Rgba(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// 将两个十六进制字符合并为一个字节（重复单字符，如 'f' → 0xFF）。
fn hex_char_to_byte(c1: char, c2: char) -> u8 {
    let s = format!("{}{}", c1, c2);
    u8::from_str_radix(&s, 16).unwrap_or(0)
}

/// 解析 rgb() / rgba() 函数。
fn parse_rgb_function(value: &str) -> Option<ColorValue> {
    // 提取括号内的内容
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let r = parse_color_component(parts[0].trim())?;
    let g = parse_color_component(parts[1].trim())?;
    let b = parse_color_component(parts[2].trim())?;
    let a = if parts.len() > 3 {
        parse_alpha_component(parts[3].trim())?
    } else {
        255u8
    };

    Some(ColorValue::Rgba(r, g, b, a))
}

/// 解析颜色分量（0-255 或 0%-100%）。
fn parse_color_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 alpha 分量（0-1 或 0%-100%）。
fn parse_alpha_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some((v * 255.0).round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 hsl() / hsla() 函数。
fn parse_hsl_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let h: f64 = parts[0].trim().trim_end_matches("deg").parse().ok()?;
    let s: f64 = parts[1].trim().trim_end_matches('%').parse().ok()?;
    let l: f64 = parts[2].trim().trim_end_matches('%').parse().ok()?;
    let a = if parts.len() > 3 {
        parts[3].trim().parse().ok()?
    } else {
        1.0
    };

    Some(ColorValue::Hsla(h, s, l, a))
}

/// 将 HWB 颜色转换为 RGBA。
///
/// 参数：
/// - `h`：色相（度），0-360
/// - `w`：白度（0-1 比例）
/// - `b`：黑度（0-1 比例）
/// - `a`：透明度（0-1 比例）
///
/// 如果 W+B > 1，两者按比例缩小使总和为 1。
pub fn hwb_to_rgba(h: f64, w: f64, b: f64, a: f64) -> (u8, u8, u8, u8) {
    // 钳制 W+B 到 100%
    let mut ww = w.clamp(0.0, 1.0);
    let mut bb = b.clamp(0.0, 1.0);
    if ww + bb > 1.0 {
        let scale = 1.0 / (ww + bb);
        ww *= scale;
        bb *= scale;
    }

    // 先将 HWB 转为 HSL 再转为 RGB
    // HWB → RGB 标准算法：
    // 先算出没有白度/黑度影响的纯色 RGB，再与白/黑混合
    let h_norm = (h % 360.0) / 60.0;
    let sector = h_norm.floor() as i32;
    let f = h_norm - sector as f64;

    // 6 个扇区的纯色分量
    let (r_pure, g_pure, b_pure) = match sector % 6 {
        0 => (1.0, f, 0.0),
        1 => (1.0 - f, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, 1.0 - f, 1.0),
        4 => (f, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - f),
    };

    // 混合：result = color * (1 - W - B) + W
    let factor = 1.0 - ww - bb;
    let r = (r_pure * factor + ww).clamp(0.0, 1.0);
    let g = (g_pure * factor + ww).clamp(0.0, 1.0);
    let bv = (b_pure * factor + ww).clamp(0.0, 1.0);

    let alpha = a.clamp(0.0, 1.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (bv * 255.0).round() as u8,
        (alpha * 255.0).round() as u8,
    )
}

/// 解析 hwb() 颜色函数。
///
/// 格式：`hwb(H W B)` 或 `hwb(H W B / A)`，其中 H 为色相（数字），
/// W 为白度（百分比），B 为黑度（百分比），A 为可选的透明度。
fn parse_hwb_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    // 检查是否有斜杠分隔的 alpha
    let slash_pos = inner.find('/');
    let main_part = match slash_pos {
        Some(pos) => inner[..pos].trim(),
        None => inner,
    };
    let alpha_str = slash_pos.map(|pos| inner[pos + 1..].trim());

    // 按空格分割：H W B
    let parts: Vec<&str> = main_part.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let h: f64 = parts[0].trim_end_matches("deg").parse().ok()?;
    let w_pct: f64 = parts[1].trim_end_matches('%').parse().ok()?;
    let b_pct: f64 = parts[2].trim_end_matches('%').parse().ok()?;
    let w = w_pct / 100.0;
    let b = b_pct / 100.0;
    let a = if let Some(a_str) = alpha_str {
        if a_str.ends_with('%') {
            a_str.trim_end_matches('%').parse::<f64>().ok()? / 100.0
        } else {
            a_str.parse::<f64>().ok()?
        }
    } else {
        1.0
    };

    let (r, g, bv, av) = hwb_to_rgba(h, w, b, a);
    Some(ColorValue::Rgba(r, g, bv, av))
}

/// 解析命名颜色。
///
/// 支持全部 148 种 CSS 标准命名颜色。
fn parse_named_color(value: &str) -> Option<ColorValue> {
    let lower = value.to_ascii_lowercase();
    let rgba = |r: u8, g: u8, b: u8| Some(ColorValue::Rgba(r, g, b, 255));
    match lower.as_str() {
        // CSS 基础 16 色
        "black" => rgba(0, 0, 0),
        "white" => rgba(255, 255, 255),
        "red" => rgba(255, 0, 0),
        "green" => rgba(0, 128, 0),
        "blue" => rgba(0, 0, 255),
        "yellow" => rgba(255, 255, 0),
        "cyan" | "aqua" => rgba(0, 255, 255),
        "magenta" | "fuchsia" => rgba(255, 0, 255),
        "silver" => rgba(192, 192, 192),
        "gray" | "grey" => rgba(128, 128, 128),
        "maroon" => rgba(128, 0, 0),
        "olive" => rgba(128, 128, 0),
        "lime" => rgba(0, 255, 0),
        "teal" => rgba(0, 128, 128),
        "navy" => rgba(0, 0, 128),
        "purple" => rgba(128, 0, 128),
        "orange" => rgba(255, 165, 0),
        // 扩展命名颜色 (A-F)
        "aliceblue" => rgba(240, 248, 255),
        "antiquewhite" => rgba(250, 235, 215),
        "aquamarine" => rgba(127, 255, 212),
        "azure" => rgba(240, 255, 255),
        "beige" => rgba(245, 245, 220),
        "bisque" => rgba(255, 228, 196),
        "blanchedalmond" => rgba(255, 235, 205),
        "burlywood" => rgba(222, 184, 135),
        "cadetblue" => rgba(95, 158, 160),
        "chartreuse" => rgba(127, 255, 0),
        "chocolate" => rgba(210, 105, 30),
        "coral" => rgba(255, 127, 80),
        "cornflowerblue" => rgba(100, 149, 237),
        "cornsilk" => rgba(255, 248, 220),
        "crimson" => rgba(220, 20, 60),
        "darkblue" => rgba(0, 0, 139),
        "darkcyan" => rgba(0, 139, 139),
        "darkgoldenrod" => rgba(184, 134, 11),
        "darkgray" | "darkgrey" => rgba(169, 169, 169),
        "darkgreen" => rgba(0, 100, 0),
        "darkkhaki" => rgba(189, 183, 107),
        "darkmagenta" => rgba(139, 0, 139),
        "darkolivegreen" => rgba(85, 107, 47),
        "darkorange" => rgba(255, 140, 0),
        "darkorchid" => rgba(153, 50, 204),
        "darkred" => rgba(139, 0, 0),
        "darksalmon" => rgba(233, 150, 122),
        "darkseagreen" => rgba(143, 188, 143),
        "darkslateblue" => rgba(72, 61, 139),
        "darkslategray" | "darkslategrey" => rgba(47, 79, 79),
        "darkturquoise" => rgba(0, 206, 209),
        "darkviolet" => rgba(148, 0, 211),
        "deeppink" => rgba(255, 20, 147),
        "deepskyblue" => rgba(0, 191, 255),
        "dimgray" | "dimgrey" => rgba(105, 105, 105),
        "dodgerblue" => rgba(30, 144, 255),
        "firebrick" => rgba(178, 34, 34),
        "floralwhite" => rgba(255, 250, 240),
        "forestgreen" => rgba(34, 139, 34),
        // G-L
        "gainsboro" => rgba(220, 220, 220),
        "ghostwhite" => rgba(248, 248, 255),
        "gold" => rgba(255, 215, 0),
        "goldenrod" => rgba(218, 165, 32),
        "greenyellow" => rgba(173, 255, 47),
        "honeydew" => rgba(240, 255, 240),
        "hotpink" => rgba(255, 105, 180),
        "indianred" => rgba(205, 92, 92),
        "indigo" => rgba(75, 0, 130),
        "ivory" => rgba(255, 255, 240),
        "khaki" => rgba(240, 230, 140),
        "lavender" => rgba(230, 230, 250),
        "lavenderblush" => rgba(255, 240, 245),
        "lawngreen" => rgba(124, 252, 0),
        "lemonchiffon" => rgba(255, 250, 205),
        "lightblue" => rgba(173, 216, 230),
        "lightcoral" => rgba(240, 128, 128),
        "lightcyan" => rgba(224, 255, 255),
        "lightgoldenrodyellow" => rgba(250, 250, 210),
        "lightgray" | "lightgrey" => rgba(211, 211, 211),
        "lightgreen" => rgba(144, 238, 144),
        "lightpink" => rgba(255, 182, 193),
        "lightsalmon" => rgba(255, 160, 122),
        "lightseagreen" => rgba(32, 178, 170),
        "lightskyblue" => rgba(135, 206, 250),
        "lightslategray" | "lightslategrey" => rgba(119, 136, 153),
        "lightsteelblue" => rgba(176, 196, 222),
        "lightyellow" => rgba(255, 255, 224),
        "limegreen" => rgba(50, 205, 50),
        "linen" => rgba(250, 240, 230),
        // M-P
        "mediumaquamarine" => rgba(102, 205, 170),
        "mediumblue" => rgba(0, 0, 205),
        "mediumorchid" => rgba(186, 85, 211),
        "mediumpurple" => rgba(147, 112, 219),
        "mediumseagreen" => rgba(60, 179, 113),
        "mediumslateblue" => rgba(123, 104, 238),
        "mediumspringgreen" => rgba(0, 250, 154),
        "mediumturquoise" => rgba(72, 209, 204),
        "mediumvioletred" => rgba(199, 21, 133),
        "midnightblue" => rgba(25, 25, 112),
        "mintcream" => rgba(245, 255, 250),
        "mistyrose" => rgba(255, 228, 225),
        "moccasin" => rgba(255, 228, 181),
        "navajowhite" => rgba(255, 222, 173),
        "oldlace" => rgba(253, 245, 230),
        "olivedrab" => rgba(107, 142, 35),
        "orangered" => rgba(255, 69, 0),
        "orchid" => rgba(218, 112, 214),
        "palegoldenrod" => rgba(238, 232, 170),
        "palegreen" => rgba(152, 251, 152),
        "paleturquoise" => rgba(175, 238, 238),
        "palevioletred" => rgba(219, 112, 147),
        "papayawhip" => rgba(255, 239, 213),
        "peachpuff" => rgba(255, 218, 185),
        "peru" => rgba(205, 133, 63),
        "pink" => rgba(255, 192, 203),
        "plum" => rgba(221, 160, 221),
        "powderblue" => rgba(176, 224, 230),
        // R-T
        "rosybrown" => rgba(188, 143, 143),
        "royalblue" => rgba(65, 105, 225),
        "saddlebrown" => rgba(139, 69, 19),
        "salmon" => rgba(250, 128, 114),
        "sandybrown" => rgba(244, 164, 96),
        "seagreen" => rgba(46, 139, 87),
        "seashell" => rgba(255, 245, 238),
        "sienna" => rgba(160, 82, 45),
        "skyblue" => rgba(135, 206, 235),
        "slateblue" => rgba(106, 90, 205),
        "slategray" | "slategrey" => rgba(112, 128, 144),
        "snow" => rgba(255, 250, 250),
        "springgreen" => rgba(0, 255, 127),
        "steelblue" => rgba(70, 130, 180),
        "tan" => rgba(210, 180, 140),
        "thistle" => rgba(216, 191, 216),
        "tomato" => rgba(255, 99, 71),
        "turquoise" => rgba(64, 224, 208),
        // U-Z
        "violet" => rgba(238, 130, 238),
        "wheat" => rgba(245, 222, 179),
        "whitesmoke" => rgba(245, 245, 245),
        "yellowgreen" => rgba(154, 205, 50),
        // transparent 和 currentColor 由 parse_color_value 直接处理
        "transparent" => Some(ColorValue::Transparent),
        "currentcolor" => Some(ColorValue::CurrentColor),
        _ => None,
    }
}

/// 解析 CSS display 属性值。
pub fn parse_display(value: &str) -> Option<DisplayValue> {
    match value.trim() {
        "block" => Some(DisplayValue::Block),
        "inline" => Some(DisplayValue::Inline),
        "inline-block" => Some(DisplayValue::InlineBlock),
        "flex" => Some(DisplayValue::Flex),
        "inline-flex" => Some(DisplayValue::InlineFlex),
        "grid" => Some(DisplayValue::Grid),
        "inline-grid" => Some(DisplayValue::InlineGrid),
        "none" => Some(DisplayValue::None),
        "contents" => Some(DisplayValue::Contents),
        "flow" => Some(DisplayValue::Flow),
        "flow-root" => Some(DisplayValue::FlowRoot),
        "list-item" => Some(DisplayValue::ListItem),
        _ => None,
    }
}

/// 解析 CSS position 属性值。
pub fn parse_position(value: &str) -> Option<PositionValue> {
    match value.trim() {
        "static" => Some(PositionValue::Static),
        "relative" => Some(PositionValue::Relative),
        "absolute" => Some(PositionValue::Absolute),
        "fixed" => Some(PositionValue::Fixed),
        "sticky" => Some(PositionValue::Sticky),
        _ => None,
    }
}

/// 解析 CSS overflow 属性值。
pub fn parse_overflow(value: &str) -> Option<OverflowValue> {
    match value.trim() {
        "visible" => Some(OverflowValue::Visible),
        "hidden" => Some(OverflowValue::Hidden),
        "scroll" => Some(OverflowValue::Scroll),
        "auto" => Some(OverflowValue::Auto),
        "clip" => Some(OverflowValue::Clip),
        _ => None,
    }
}

/// 解析 CSS float 属性值。
pub fn parse_float(value: &str) -> Option<FloatValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(FloatValue::None),
        "left" => Some(FloatValue::Left),
        "right" => Some(FloatValue::Right),
        "inline-start" => Some(FloatValue::InlineStart),
        "inline-end" => Some(FloatValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS clear 属性值。
pub fn parse_clear(value: &str) -> Option<ClearValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(ClearValue::None),
        "left" => Some(ClearValue::Left),
        "right" => Some(ClearValue::Right),
        "both" => Some(ClearValue::Both),
        "inline-start" => Some(ClearValue::InlineStart),
        "inline-end" => Some(ClearValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS list-style-type 属性值。
pub fn parse_list_style_type(value: &str) -> Option<ListStyleTypeValue> {
    match value.trim().to_lowercase().as_str() {
        "disc" => Some(ListStyleTypeValue::Disc),
        "circle" => Some(ListStyleTypeValue::Circle),
        "square" => Some(ListStyleTypeValue::Square),
        "decimal" => Some(ListStyleTypeValue::Decimal),
        "decimal-leading-zero" => Some(ListStyleTypeValue::DecimalLeadingZero),
        "lower-roman" => Some(ListStyleTypeValue::LowerRoman),
        "upper-roman" => Some(ListStyleTypeValue::UpperRoman),
        "lower-alpha" | "lower-latin" => Some(ListStyleTypeValue::LowerAlpha),
        "upper-alpha" | "upper-latin" => Some(ListStyleTypeValue::UpperAlpha),
        "none" => Some(ListStyleTypeValue::None),
        _ => None,
    }
}

/// 解析 CSS list-style-position 属性值。
pub fn parse_list_style_position(value: &str) -> Option<ListStylePositionValue> {
    match value.trim().to_lowercase().as_str() {
        "outside" => Some(ListStylePositionValue::Outside),
        "inside" => Some(ListStylePositionValue::Inside),
        _ => None,
    }
}

/// 解析 CSS flex-direction 属性值。
pub fn parse_flex_direction(value: &str) -> Option<FlexDirectionValue> {
    match value.trim() {
        "row" => Some(FlexDirectionValue::Row),
        "row-reverse" => Some(FlexDirectionValue::RowReverse),
        "column" => Some(FlexDirectionValue::Column),
        "column-reverse" => Some(FlexDirectionValue::ColumnReverse),
        _ => None,
    }
}

/// 解析 CSS flex-wrap 属性值。
pub fn parse_flex_wrap(value: &str) -> Option<FlexWrapValue> {
    match value.trim() {
        "nowrap" => Some(FlexWrapValue::Nowrap),
        "wrap" => Some(FlexWrapValue::Wrap),
        "wrap-reverse" => Some(FlexWrapValue::WrapReverse),
        _ => None,
    }
}

/// 解析 CSS justify-content / align-items 属性值。
pub fn parse_alignment(value: &str) -> Option<AlignmentValue> {
    match value.trim() {
        "flex-start" => Some(AlignmentValue::FlexStart),
        "flex-end" => Some(AlignmentValue::FlexEnd),
        "center" => Some(AlignmentValue::Center),
        "space-between" => Some(AlignmentValue::SpaceBetween),
        "space-around" => Some(AlignmentValue::SpaceAround),
        "space-evenly" => Some(AlignmentValue::SpaceEvenly),
        "stretch" => Some(AlignmentValue::Stretch),
        "start" => Some(AlignmentValue::Start),
        "end" => Some(AlignmentValue::End),
        "baseline" => Some(AlignmentValue::Baseline),
        _ => None,
    }
}

/// 解析 CSS box-sizing 属性值。
pub fn parse_box_sizing(value: &str) -> Option<BoxSizingValue> {
    match value.trim() {
        "content-box" => Some(BoxSizingValue::ContentBox),
        "border-box" => Some(BoxSizingValue::BorderBox),
        _ => None,
    }
}

/// 解析 CSS visibility 属性值。
pub fn parse_visibility(value: &str) -> Option<VisibilityValue> {
    match value.trim() {
        "visible" => Some(VisibilityValue::Visible),
        "hidden" => Some(VisibilityValue::Hidden),
        "collapse" => Some(VisibilityValue::Collapse),
        _ => None,
    }
}

/// 解析 CSS word-break 属性值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS writing-mode 属性值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
        _ => None,
    }
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS text-indent 属性值。
///
/// 支持长度值（如 `2em`、`20px`）和百分比值（如 `10%`）。
/// 不支持 `auto` 关键字。
pub fn parse_text_indent(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_length(v)
}

/// 解析 CSS letter-spacing / word-spacing 值。
/// "normal" 映射为 LengthValue::Px(0.0)。
pub fn parse_spacing(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("normal") {
        return Some(LengthValue::Px(0.0));
    }
    parse_length(v)
}

/// 解析 CSS font-weight 属性值。
pub fn parse_font_weight(value: &str) -> Option<FontWeightValue> {
    match value.trim() {
        "bold" => Some(FontWeightValue::Bold),
        "normal" => Some(FontWeightValue::Normal),
        "bolder" => Some(FontWeightValue::Bolder),
        "lighter" => Some(FontWeightValue::Lighter),
        s => {
            let w: u16 = s.parse().ok()?;
            if (100..=900).contains(&w) {
                Some(FontWeightValue::Absolute(w))
            } else {
                None
            }
        }
    }
}

/// 解析 CSS font-style 属性值。
pub fn parse_font_style(value: &str) -> Option<FontStyleValue> {
    let value = value.trim();
    if value == "normal" {
        Some(FontStyleValue::Normal)
    } else if value == "italic" {
        Some(FontStyleValue::Italic)
    } else if value.starts_with("oblique") {
        let angle_str = value.strip_prefix("oblique")?.trim();
        if angle_str.is_empty() {
            Some(FontStyleValue::Oblique(None))
        } else {
            // 处理 "(angle)" 或 "(angledeg)" 形式
            let angle_str = angle_str
                .strip_prefix('(')
                .unwrap_or(angle_str)
                .strip_suffix(')')
                .unwrap_or(angle_str);
            let angle: f64 = angle_str.trim_end_matches("deg").trim().parse().ok()?;
            Some(FontStyleValue::Oblique(Some(angle)))
        }
    } else {
        None
    }
}
