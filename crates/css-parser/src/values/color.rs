//! CSS 颜色和基础属性解析。

use super::*;

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()`、`hsl()`/`hsla()` 和 `hwb()` 函数。
pub fn parse_color(value: &str) -> Option<ColorValue> {
    // 不在此 trim：声明值经 consume_declaration deferred-whitespace 已无首尾空白 token，
    // 此处 trim 会误剥**转义产生的**空白（如 `red\9` → `red\t`，应 ≠ 关键字 `red` 判无效，
    // apply 拒绝→cascade R2126 丢弃）。调用方传入均为已 trim 值；quirks 入口自行 trim。
    // driving：escapes-014/015/016。

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

    // color() 函数（CSS Color 4 预定义颜色空间：srgb/srgb-linear/display-p3/a98-rgb/xyz…）
    if value.starts_with("color(") {
        return parse_color_function(value);
    }

    // light-dark() 函数（CSS Color Adjust §color-scheme-effect）：light-dark(<light>, <dark>)
    // 按元素的 color-scheme 取值。ZW 默认 color-scheme = light（normal→light），故取第一个
    //（light）参数。driving: css-color light-dark-inheritance / light-dark-currentcolor。
    if value.starts_with("light-dark(") {
        let start = value.find('(')?;
        let end = value.rfind(')')?;
        let inner = strip_css_comments(value.get(start + 1..end)?);
        let light = first_top_level_comma_arg(&inner);
        if light.is_empty() {
            return None;
        }
        return parse_color(light);
    }

    // color-mix() 函数（CSS Color 5）：color-mix(in <space>, <c1> [<p1>], <c2> [<p2>])。
    // 仅 `in srgb` 支持（其他色彩空间 defer）。存为未解析 ColorValue::Mix——currentColor 在
    // paint 时按元素色解析，支持 inherit 透传。driving: css-color color-mix-currentcolor-001。
    if value.len() >= 10 && value[..10].eq_ignore_ascii_case("color-mix(") {
        return parse_color_mix(value);
    }

    // 命名颜色
    parse_named_color(value)
}

/// Quirks mode 颜色解析。
///
/// 先尝试标准 `parse_color`，如果失败，则尝试 quirks mode 特有的解析规则：
/// - 不带 `#` 前缀的十六进制字符串（如 `"FF0000"` → 红色）
/// - 纯数字字符串（如 `"0"` → 黑色，`"16711680"` → 红色）
///
/// 这是浏览器在 quirks mode 下对颜色属性值的宽容行为。
pub fn parse_color_quirks(value: &str) -> Option<ColorValue> {
    let value = value.trim();

    // 先尝试标准解析
    if let Some(c) = parse_color(value) {
        return Some(c);
    }

    // Quirks: 尝试作为不带 # 的十六进制解析（3位或6位）
    if (value.len() == 3 || value.len() == 6) && value.chars().all(|c| c.is_ascii_hexdigit()) {
        return parse_hex_color(&format!("#{}", value));
    }

    // Quirks: 尝试作为纯数字解析（转为 24-bit RGB）
    if let Ok(num) = value.parse::<u32>() {
        // CSS-06: 钳制到 0xFFFFFF，高位截断不符合浏览器行为
        let num = num.min(0xFFFFFF);
        let r = ((num >> 16) & 0xFF) as u8;
        let g = ((num >> 8) & 0xFF) as u8;
        let b = (num & 0xFF) as u8;
        return Some(ColorValue::Rgba(r, g, b, 255));
    }

    None
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
///
/// 支持两种语法（CSS Color 4）：
/// - 遗留逗号：`rgb(R, G, B)` / `rgba(R, G, B, A)`。
/// - 现代空白：`rgb(R G B)` / `rgb(R G B / A)`（分量以空白分隔，alpha 以斜杠分隔）。
///
/// 分量为 0-255 或 0%-100%；alpha 为 0-1 或百分比；`none` → 0。分量间允许 `/* 注释 */`。
/// driving: css-color rgb-001..006 / background-color-rgb-001..002。
fn parse_rgb_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner_str = strip_css_comments(value.get(start + 1..end)?);
    let inner = inner_str.trim();

    // 现代斜杠 alpha（仅无逗号时）；遗留逗号 alpha 在第 4 分量。
    let (main, slash_alpha) = if inner.contains(',') {
        (inner, None)
    } else {
        match inner.split_once('/') {
            Some((m, a)) => (m.trim(), Some(a.trim())),
            None => (inner, None),
        }
    };

    let comps: Vec<&str> = if main.contains(',') {
        main.split(',').map(str::trim).filter(|s| !s.is_empty()).collect()
    } else {
        main.split_whitespace().collect()
    };
    if !(3..=4).contains(&comps.len()) {
        return None;
    }

    let r = parse_rgb_component(comps[0])?;
    let g = parse_rgb_component(comps[1])?;
    let b = parse_rgb_component(comps[2])?;
    let a = if let Some(ap) = slash_alpha {
        parse_rgb_alpha(ap)?
    } else if comps.len() == 4 {
        parse_rgb_alpha(comps[3])?
    } else {
        255u8
    };

    Some(ColorValue::Rgba(r, g, b, a))
}

/// rgb 分量（0-255 或 0%-100%）；CSS Color 4 `none` → 0。
fn parse_rgb_component(s: &str) -> Option<u8> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0);
    }
    parse_color_component(s)
}

/// rgb alpha（0-1 或 0%-100%）；`none` → 0。
fn parse_rgb_alpha(s: &str) -> Option<u8> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0);
    }
    parse_alpha_component(s)
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

/// 解析 CSS Color 4 `color()` 函数（预定义颜色空间）。
///
/// `color(<space> c1 c2 c3 [ / <alpha>])`，space ∈ srgb / srgb-linear / display-p3 /
/// a98-rgb / rec2020 / prophoto-rgb / xyz / xyz-d50 / xyz-d65（CSS Color 4 全部预定义空间）。
/// 分量 0-1（可越界/负）；`none` → 0。wide-gamut 经传递函数 + XYZ-D65 矩阵转换到 sRGB 渲染。
/// driving: css-color a98rgb-001..004 / xyz-* / display-p3-* / rec2020-* / prophoto-* / background-color-color-*。
fn parse_color_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner_str = strip_css_comments(value.get(start + 1..end)?);
    let inner = inner_str.trim();

    let (main, slash_alpha) = if let Some((m, a)) = inner.split_once('/') {
        (m.trim(), Some(a.trim()))
    } else {
        (inner, None)
    };

    let mut parts = main.split_whitespace();
    let space = parts.next()?.to_ascii_lowercase();
    let comps: Vec<&str> = parts.collect();
    if comps.len() != 3 {
        return None;
    }
    let c0 = parse_color_number(comps[0])?;
    let c1 = parse_color_number(comps[1])?;
    let c2 = parse_color_number(comps[2])?;
    let a = if let Some(ap) = slash_alpha {
        parse_alpha_value(ap)?
    } else {
        1.0
    };

    let (r, g, b) = convert_predefined_to_srgb(&space, c0, c1, c2)?;
    Some(ColorValue::Rgba(r, g, b, (a * 255.0).round().clamp(0.0, 255.0) as u8))
}

/// 解析 color() 分量数字（0-1 浮点，可负/越界）；`none` → 0。
fn parse_color_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    s.parse().ok()
}

/// sRGB 传递函数：分量 → 线性光（CSS Color 4，与 display-p3 共用）。
fn srgb_decode(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB 传递函数：线性光 → 分量（编码 gamma）。
fn srgb_encode(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// 线性 RGB 分量 → sRGB u8（编码 gamma + 钳制 + 四舍五入）。
fn linear_srgb_to_u8(c: f64) -> u8 {
    (srgb_encode(c) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// 3×3 矩阵（行优先 [9]）乘列向量。
fn mat3_mul(m: [f64; 9], x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (
        m[0] * x + m[1] * y + m[2] * z,
        m[3] * x + m[4] * y + m[5] * z,
        m[6] * x + m[7] * y + m[8] * z,
    )
}

// display-p3（线性）→ XYZ-D65（CSS Color 4）
const P3_TO_XYZ: [f64; 9] = [
    0.4865709, 0.2656677, 0.1982173, 0.2289746, 0.6917385, 0.0792869, 0.0, 0.0451134, 1.0439444,
];
// a98-rgb（线性）→ XYZ-D65（CSS Color 4）
const A98_TO_XYZ: [f64; 9] = [
    0.5766690, 0.1855582, 0.1882286, 0.2973448, 0.6273624, 0.0752914, 0.0270313, 0.0706892, 0.9913375,
];
// rec2020（线性）→ XYZ-D65（CSS Color 4）
const REC2020_TO_XYZ: [f64; 9] = [
    0.6369580, 0.1446169, 0.1688809, 0.2627002, 0.6779981, 0.0593017, 0.0, 0.0280727, 1.0609851,
];
// prophoto-rgb（线性）→ XYZ-D50（CSS Color 4）
const PROPHOTO_TO_XYZ: [f64; 9] = [
    0.7977666, 0.1351813, 0.0313477, 0.2880747, 0.7118762, 0.0000853, 0.0, 0.0, 0.8251044,
];
// XYZ-D65 → 线性 sRGB（CSS Color 4）
const XYZ_TO_SRGB: [f64; 9] = [
    3.2409699, -1.5373832, -0.4986108, -0.9692436, 1.8759675, 0.0415551, 0.0556300, -0.2039770, 1.0569715,
];
// XYZ-D50 → XYZ-D65（Bradford 色度适应）
const XYZ_D50_TO_D65: [f64; 9] = [
    0.9555766, -0.0230393, 0.0631636, -0.0282895, 1.0099416, 0.0210077, 0.0122982, -0.0204830, 1.3299098,
];

/// 把预定义颜色空间 3 分量转换为 sRGB u8。返回 None 表示不支持的空间（rec2020/prophoto/未知）。
fn convert_predefined_to_srgb(space: &str, c0: f64, c1: f64, c2: f64) -> Option<(u8, u8, u8)> {
    let (lr, lg, lb) = match space {
        "srgb" => {
            // 已是 sRGB gamma 分量（0-1），直接转 u8。
            return Some((
                (c0 * 255.0).round().clamp(0.0, 255.0) as u8,
                (c1 * 255.0).round().clamp(0.0, 255.0) as u8,
                (c2 * 255.0).round().clamp(0.0, 255.0) as u8,
            ));
        }
        "srgb-linear" => (c0, c1, c2), // 已是线性 sRGB
        "display-p3" => {
            let (x, y, z) = mat3_mul(P3_TO_XYZ, srgb_decode(c0), srgb_decode(c1), srgb_decode(c2));
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "a98-rgb" => {
            let g = 563.0 / 256.0; // a98 gamma ≈ 2.1992
            let (x, y, z) = mat3_mul(A98_TO_XYZ, safe_powf(c0, g), safe_powf(c1, g), safe_powf(c2, g));
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "rec2020" => {
            let (x, y, z) = mat3_mul(
                REC2020_TO_XYZ,
                rec2020_decode(c0),
                rec2020_decode(c1),
                rec2020_decode(c2),
            );
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "prophoto-rgb" => {
            // prophoto 矩阵到 XYZ-D50，须 Bradford 适应到 D65。
            let (x, y, z) = mat3_mul(
                PROPHOTO_TO_XYZ,
                prophoto_decode(c0),
                prophoto_decode(c1),
                prophoto_decode(c2),
            );
            let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "xyz" | "xyz-d65" => mat3_mul(XYZ_TO_SRGB, c0, c1, c2),
        "xyz-d50" => {
            let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, c0, c1, c2);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        _ => return None,
    };
    Some((linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb)))
}

/// 安全幂运算：负分量（越界/色域外）钳到 0（powf 对负数返回 NaN）。
fn safe_powf(c: f64, g: f64) -> f64 {
    c.max(0.0).powf(g)
}

/// BT.2020 传递函数（分量 → 线性光）。α/β 为 BT.2020 常数。
fn rec2020_decode(c: f64) -> f64 {
    const ALPHA: f64 = 1.09929682680944;
    const BETA: f64 = 0.018053968510807;
    if c < BETA * 4.5 {
        c / 4.5
    } else {
        safe_powf((c + ALPHA - 1.0) / ALPHA, 1.0 / 0.45)
    }
}

/// prophoto-rgb 传递函数（分量 → 线性光，gamma 1.8 + 线性 toe）。
fn prophoto_decode(c: f64) -> f64 {
    if c < 0.03125 {
        (c / 16.0).max(0.0)
    } else {
        safe_powf(c, 1.8)
    }
}

/// 解析 hsl() / hsla() 函数。
///
/// 支持两种语法（CSS Color 4）：
/// - 遗留逗号：`hsl(H, S, L)` / `hsla(H, S, L, A)`。
/// - 现代空白：`hsl(H S L)` / `hsl(H S L / A)`（分量以空白分隔，alpha 以斜杠分隔）。
///
/// 色相 H 为角度（数字 + 可选 `deg`/`grad`/`rad`/`turn` 单位）；S/L 为百分比；alpha 为 0-1 或百分比。
/// CSS Color 4 `none` 关键字 → 该分量取 0。分量间允许 `/* 注释 */`。负值/越界 H 留待
/// `hsla_to_rgba` 渲染时归一化（R2253）。driving: css-color hsl-001..008 / hsla-001..008 /
/// background-color-hsl-001..003 / t424 / t425。
fn parse_hsl_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner_str = strip_css_comments(value.get(start + 1..end)?);
    let inner = inner_str.trim();

    // 斜杠分隔的现代 alpha（hsl(H S L / A)）；遗留逗号语法的 alpha 在分量列表第 4 位。
    // CSS Color 4 禁止逗号与斜杠 alpha 混用：仅当无逗号时才认斜杠。
    let (main, slash_alpha) = if main_uses_comma_syntax(inner) {
        (inner, None)
    } else {
        match inner.split_once('/') {
            Some((m, a)) => (m.trim(), Some(a.trim())),
            None => (inner, None),
        }
    };

    // 分量分隔：逗号（遗留）或空白（Color 4）。
    let comps: Vec<&str> = if main.contains(',') {
        main.split(',').map(str::trim).filter(|s| !s.is_empty()).collect()
    } else {
        main.split_whitespace().collect()
    };
    if !(3..=4).contains(&comps.len()) {
        return None;
    }

    let h = parse_hue_angle(comps[0])?;
    let s = parse_percent_component(comps[1])?;
    let l = parse_percent_component(comps[2])?;
    let a = if let Some(ap) = slash_alpha {
        parse_alpha_value(ap)?
    } else if comps.len() == 4 {
        parse_alpha_value(comps[3])?
    } else {
        1.0
    };

    Some(ColorValue::Hsla(h, s, l, a))
}

/// 判定 hsl/hwb 内部是否使用遗留逗号语法（含逗号即按逗号语义；逗号语法禁斜杠 alpha）。
fn main_uses_comma_syntax(inner: &str) -> bool {
    inner.contains(',')
}

/// 移除 CSS 注释 `/* ... */`（CSS Color 4 允许颜色分量间穿插注释，如
/// `hsl(120/* c */75%/* c */50%)`）。注释是**词法分隔符**，故以空格替换（而非删除），
/// 避免把 `120/* c */75%` 拼成 `12075%`。未闭合的 `/*` 丢弃其后剩余内容。
fn strip_css_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        out.push(' ');
        rest = match rest[start..].find("*/") {
            Some(end_rel) => &rest[start + end_rel + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// 取首个**顶层**（括号深度 0）逗号前的参数，trim 返回。无顶层逗号则返回整体 trim。
/// 用于 light-dark() 等多参数颜色函数取首个参数（参数内可能含嵌套逗号，如 color-mix()）。
fn first_top_level_comma_arg(s: &str) -> &str {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return s[..i].trim(),
            _ => {}
        }
    }
    s.trim()
}

/// 返回**顶层**（括号深度 0）首个目标字符的字节位置，无则 None。
fn top_level_byte_index(s: &str, target: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = (depth - 1).max(0),
            _ if depth == 0 && b == target => return Some(i),
            _ => {}
        }
    }
    None
}

/// 解析 `color-mix(in <space>, <c1> [<p1>], <c2> [<p2>])`（CSS Color 5）。
///
/// 仅 `in srgb` 支持；分量 `<color> [<百分比>]`，百分比省略按 spec 默认（双省略=50/50）。
/// currentColor 保留未解析（paint 时按元素色解析）。driving: color-mix-currentcolor-001。
fn parse_color_mix(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = strip_css_comments(value.get(start + 1..end)?);
    // 首个顶层逗号分隔色彩空间与第一分量
    let first_comma = top_level_byte_index(&inner, b',')?;
    let space = inner[..first_comma].trim();
    if !space.eq_ignore_ascii_case("in srgb") {
        return None; // 其他色彩空间（srgb-linear/lch/oklch/…）defer
    }
    let rest = inner[first_comma + 1..].trim();
    // 顶层逗号分隔两分量
    let second_comma = top_level_byte_index(rest, b',')?;
    let c1 = parse_color_mix_component(rest[..second_comma].trim())?;
    let c2 = parse_color_mix_component(rest[second_comma + 1..].trim())?;
    Some(ColorValue::Mix(Box::new(ColorMixSpec { c1, c2 })))
}

/// 解析 color-mix 单分量：`<color>` 或 `<color> <百分比>`（CSS Color 4 空白语法）。
fn parse_color_mix_component(s: &str) -> Option<ColorMixComponent> {
    let s = s.trim();
    // 末尾百分比：最后一个**顶层**空白之后是 `<num>%`。
    if let Some(ws) = trailing_percentage_pos(s) {
        let color_str = s[..ws].trim();
        let pct_str = s[ws..].trim();
        let pct = pct_str.trim_end_matches('%').parse::<f64>().ok()?;
        let color = parse_color(color_str)?;
        return Some(ColorMixComponent {
            color,
            percentage: Some(pct),
        });
    }
    let color = parse_color(s)?;
    Some(ColorMixComponent {
        color,
        percentage: None,
    })
}

/// 返回末尾百分比在 `s` 中的起始字节位置（最后一个顶层空白处），无则 None。
fn trailing_percentage_pos(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut last_ws = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = (depth - 1).max(0),
            b' ' | b'\t' if depth == 0 => last_ws = Some(i),
            _ => {}
        }
    }
    let ws = last_ws?;
    let after = s[ws..].trim();
    if after.ends_with('%') && after[..after.len() - 1].parse::<f64>().is_ok() {
        Some(ws)
    } else {
        None
    }
}

/// 解析色相角度（CSS Color 4）：数字 + 可选角度单位（`deg`/`grad`/`rad`/`turn`），
/// 归一化为度。`none` → 0。单位大小写不敏感。
fn parse_hue_angle(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    let lower = s.to_ascii_lowercase();
    let (num_str, scale) = if let Some(n) = lower.strip_suffix("deg") {
        (n, 1.0)
    } else if let Some(n) = lower.strip_suffix("grad") {
        (n, 360.0 / 400.0)
    } else if let Some(n) = lower.strip_suffix("turn") {
        (n, 360.0)
    } else if let Some(n) = lower.strip_suffix("rad") {
        (n, 180.0 / std::f64::consts::PI)
    } else {
        (lower.as_str(), 1.0) // 裸数字 = deg
    };
    let v: f64 = num_str.trim().parse().ok()?;
    Some(v * scale)
}

/// 解析百分比分量（S/L）：去掉 `%` 返回数值（0-100+）。`none` → 0。
fn parse_percent_component(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    s.trim_end_matches('%').parse().ok()
}

/// 解析 alpha 分量（0-1 或 0%-100%），钳制到 [0,1]。`none` → 0。
fn parse_alpha_value(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(pct) = s.strip_suffix('%') {
        let v: f64 = pct.parse().ok()?;
        Some((v / 100.0).clamp(0.0, 1.0))
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.clamp(0.0, 1.0))
    }
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
    // R2253：色相为角度，归一化到 [0,360)（负值/越界值取模，Rust `%` 取余保留符号故须 `+360` 再 `%`）。
    let h_mod = ((h % 360.0) + 360.0) % 360.0;
    let h_norm = h_mod / 60.0;
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
        // ── CSS 系统颜色（CSS Color 4 §system-colors + §deprecated-system-colors）──
        // 现代系统颜色取 light color-scheme 合理默认值。deprecated-sameas 测试为**相对**
        // 匹配（test 的 deprecated 色 == ref 的现代色），故具体值非关键，仅需每个现代色
        // 自洽。`@supports (color: X)` 经 is_property_supported→parse_color 自动求值。
        // driving: css-color deprecated-sameas-001..023（deprecated 系统色 ≡ 其现代等价）。
        "canvas" => rgba(255, 255, 255),
        "canvastext" => rgba(0, 0, 0),
        "buttonface" => rgba(240, 240, 240),
        "buttonborder" => rgba(128, 128, 128),
        "graytext" | "greytext" => rgba(128, 128, 128),
        // deprecated 系统颜色 → 现代等价（CSS Color 4 §deprecated-system-colors 映射表）
        "activeborder" | "inactiveborder" | "threeddarkshadow" | "threedhighlight" | "threedlightshadow"
        | "threedshadow" | "windowframe" => rgba(128, 128, 128), /* → ButtonBorder */
        "activecaption" | "appworkspace" | "background" | "inactivecaption" | "infobackground" | "menu"
        | "scrollbar" | "window" => rgba(255, 255, 255), /* → Canvas */
        "buttonhighlight" | "buttonshadow" | "threedface" => rgba(240, 240, 240), /* → ButtonFace */
        "captiontext" | "infotext" | "menutext" | "windowtext" => rgba(0, 0, 0),  /* → CanvasText */
        "inactivecaptiontext" => rgba(128, 128, 128),                             /* → GrayText */
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
        "table" => Some(DisplayValue::Table),
        "inline-table" => Some(DisplayValue::InlineTable),
        "table-row" => Some(DisplayValue::TableRow),
        "table-cell" => Some(DisplayValue::TableCell),
        "table-caption" => Some(DisplayValue::TableCaption),
        "table-column" => Some(DisplayValue::TableColumn),
        "table-column-group" => Some(DisplayValue::TableColumnGroup),
        "table-row-group" => Some(DisplayValue::TableRowGroup),
        "table-header-group" => Some(DisplayValue::TableHeaderGroup),
        "table-footer-group" => Some(DisplayValue::TableFooterGroup),
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

/// 解析 CSS content-visibility 属性值（CSS Containment Module Level 2）。
///
/// 大小写不敏感（与 `parse_visibility` 同色，实际匹配走 `to_ascii_lowercase`）。
pub fn parse_content_visibility(value: &str) -> Option<ContentVisibilityValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(ContentVisibilityValue::Visible),
        "hidden" => Some(ContentVisibilityValue::Hidden),
        "auto" => Some(ContentVisibilityValue::Auto),
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

/// 解析 CSS line-break 属性值（CSS Text 3 §5.3）。
pub fn parse_line_break(value: &str) -> Option<LineBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(LineBreakValue::Auto),
        "loose" => Some(LineBreakValue::Loose),
        "normal" => Some(LineBreakValue::Normal),
        "strict" => Some(LineBreakValue::Strict),
        "anywhere" => Some(LineBreakValue::Anywhere),
        _ => None,
    }
}

/// 解析 CSS writing-mode 属性值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        // R1785：sideways-rl/lr 的 block-flow 方向等价 vertical-rl/lr（仅 glyph rotation 不同），
        // 在 parse 时规范化为对应 vertical 值——使 sideways 走已验证的 vertical 块流路径
        //（解 block-flow-direction-slr/srl 86% diff）。字形旋转（line-box-direction）是
        // paint-side 独立关注，未实现，留 future。
        "sideways-rl" => Some(WritingModeValue::VerticalRl),
        "sideways-lr" => Some(WritingModeValue::VerticalLr),
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

/// 解析 CSS text-decoration-style 值。
pub fn parse_text_decoration_style(value: &str) -> Option<TextDecorationStyleValue> {
    match value.to_ascii_lowercase().as_str() {
        "solid" => Some(TextDecorationStyleValue::Solid),
        "double" => Some(TextDecorationStyleValue::Double),
        "dotted" => Some(TextDecorationStyleValue::Dotted),
        "dashed" => Some(TextDecorationStyleValue::Dashed),
        "wavy" => Some(TextDecorationStyleValue::Wavy),
        _ => None,
    }
}

/// 解析 CSS text-decoration-thickness 值（CSS Text Decoration 4 §2.3）。R1402。
///
/// 支持 `auto` / `from-font` 关键字与 `<length>`（如 `2px`）。em/rem/% 须 computed 层
/// 字号上下文，指定值层仅认 Px（driver test text-decoration-thickness-length-rounding 用 px）。
pub fn parse_text_decoration_thickness(value: &str) -> Option<TextDecorationThicknessValue> {
    let v = value.trim();
    match v.to_ascii_lowercase().as_str() {
        "auto" => Some(TextDecorationThicknessValue::Auto),
        "from-font" => Some(TextDecorationThicknessValue::FromFont),
        _ => match parse_length(v) {
            Some(LengthValue::Px(n)) => Some(TextDecorationThicknessValue::Length(n)),
            _ => None,
        },
    }
}

/// 解析 CSS text-decoration-inset 值（CSS Text Decoration 4 §2.4）。R1607。
///
/// 语法：`<length>{1,2}`。1 个值 → start=end；2 个值 → (start, end)。
/// 仅接受真 `<length>`（px/em/rem/ch/v*/%），拒绝 auto/min-content/max-content
/// 等关键字（inset 无关键字值）。负值=向外延伸。
pub fn parse_text_decoration_inset(value: &str) -> Option<TextDecorationInsetValue> {
    // 仅认数值长度，过滤 auto/min-content/max-content 等关键字变体。
    let parse_one = |s: &str| -> Option<LengthValue> {
        match parse_length(s)? {
            v @ (LengthValue::Px(_)
            | LengthValue::Em(_)
            | LengthValue::Rem(_)
            | LengthValue::Ch(_)
            | LengthValue::Percentage(_)
            | LengthValue::Vh(_)
            | LengthValue::Vw(_)
            | LengthValue::Vmin(_)
            | LengthValue::Vmax(_)) => Some(v),
            // auto/min-content/max-content/calc/fit-content 等非纯长度 → 拒绝。
            _ => None,
        }
    };
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_one(parts[0])?;
            Some(TextDecorationInsetValue {
                start: v.clone(),
                end: v,
            })
        }
        2 => Some(TextDecorationInsetValue {
            start: parse_one(parts[0])?,
            end: parse_one(parts[1])?,
        }),
        _ => None,
    }
}

/// 解析 CSS text-emphasis-style 值（CSS Text Decoration 3 §3.1）。
/// `none` | [ [ filled | open ] || [ dot | circle | double-circle | triangle | sesame ] ] | <string>
/// 关键字组合解析为对应标记字符（filled dot → '•' 等）；<string> 取首字符。
pub fn parse_text_emphasis_style(value: &str) -> Option<TextEmphasisStyleValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(TextEmphasisStyleValue::None);
    }
    // <string>："..." 取首字符
    if v.len() >= 3 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        return Some(TextEmphasisStyleValue::Char(inner.chars().next().unwrap_or('\u{2022}')));
    }
    // 关键字：[filled|open] [shape] 任意顺序，各可缺省（filled 默认 dot，shape 缺省 filled）
    let mut filled: Option<bool> = None;
    let mut shape: Option<&str> = None;
    for tok in v.split_whitespace() {
        match tok.to_ascii_lowercase().as_str() {
            "filled" => filled = Some(true),
            "open" => filled = Some(false),
            "dot" | "circle" | "double-circle" | "triangle" | "sesame" => {
                shape = Some(match tok {
                    "dot" => "dot",
                    "circle" => "circle",
                    "double-circle" => "double-circle",
                    "triangle" => "triangle",
                    _ => "sesame",
                })
            }
            _ => return None,
        }
    }
    let filled = filled.unwrap_or(true);
    let shape = shape.unwrap_or("dot");
    let ch = match (filled, shape) {
        (true, "dot") => '\u{2022}',            // •
        (false, "dot") => '\u{25E6}',           // ◦
        (true, "circle") => '\u{25CF}',         // ●
        (false, "circle") => '\u{25CB}',        // ○
        (true, "double-circle") => '\u{25C9}',  // ◉
        (false, "double-circle") => '\u{25CE}', // ◎
        (true, "triangle") => '\u{25B2}',       // ▲
        (false, "triangle") => '\u{25B3}',      // △
        (true, "sesame") => '\u{FE45}',         // ﹅
        (false, "sesame") => '\u{FE46}',        // ﹆
        _ => '\u{2022}',
    };
    Some(TextEmphasisStyleValue::Char(ch))
}

/// 解析 CSS text-emphasis-position 值（CSS Text Decoration 3 §3.2）。
/// `[ over | under ] && [ right | left ]`，各可缺省（默认 over right）。
pub fn parse_text_emphasis_position(value: &str) -> Option<TextEmphasisPositionValue> {
    let mut over: Option<bool> = None;
    let mut right: Option<bool> = None;
    for tok in value.split_whitespace() {
        match tok.to_ascii_lowercase().as_str() {
            "over" => over = Some(true),
            "under" => over = Some(false),
            "right" => right = Some(true),
            "left" => right = Some(false),
            _ => return None,
        }
    }
    let over = over.unwrap_or(true);
    use TextEmphasisPositionValue::*;
    Some(match (over, right) {
        (true, Some(false)) => OverLeft,
        (true, _) => OverRight,
        (false, Some(false)) => UnderLeft,
        (false, _) => UnderRight,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_hsl_function 现代/遗留语法（R2253，CSS Color 4）─────────────

    #[test]
    fn test_hsl_modern_whitespace_no_alpha() {
        // hsl(120 100% 25%) — 现代空白语法，无 alpha（hsl-001 driving）
        assert_eq!(
            parse_color("hsl(120 100% 25%)"),
            Some(ColorValue::Hsla(120.0, 100.0, 25.0, 1.0))
        );
    }

    #[test]
    fn test_hsl_modern_slash_alpha() {
        // hsl(120deg 100% 25% / 1) 与 / 0.5
        assert_eq!(
            parse_color("hsl(120deg 100% 25% / 1)"),
            Some(ColorValue::Hsla(120.0, 100.0, 25.0, 1.0))
        );
        assert_eq!(
            parse_color("hsl(120 100% 25% / 0.5)"),
            Some(ColorValue::Hsla(120.0, 100.0, 25.0, 0.5))
        );
        // alpha 百分比
        assert_eq!(
            parse_color("hsla(120 75% 50% / 60%)"),
            Some(ColorValue::Hsla(120.0, 75.0, 50.0, 0.6))
        );
        assert_eq!(
            parse_color("hsla(120.0 75% 50% / 1.0)"),
            Some(ColorValue::Hsla(120.0, 75.0, 50.0, 1.0))
        );
    }

    #[test]
    fn test_hsl_legacy_comma_percent_alpha() {
        // 遗留逗号 + 百分比 alpha：旧实现 "20%".parse::<f64>() 失败致丢色（background-color-hsl-001 #p1）
        assert_eq!(
            parse_color("hsla(120.0, 75%, 50%, 20%)"),
            Some(ColorValue::Hsla(120.0, 75.0, 50.0, 0.2))
        );
        assert_eq!(
            parse_color("hsla(120, 75%, 50%, 0.4)"),
            Some(ColorValue::Hsla(120.0, 75.0, 50.0, 0.4))
        );
    }

    #[test]
    fn test_hsl_comments_between_components() {
        // CSS Color 4 允许分量间 /* 注释 */（background-color-hsl-001 #p5 driving）
        assert_eq!(
            parse_color("hsla(120/* c */75%/* c */50%/1.0)"),
            Some(ColorValue::Hsla(120.0, 75.0, 50.0, 1.0))
        );
    }

    #[test]
    fn test_hsl_angle_units_and_none() {
        // 角度单位：turn/grad/rad 归一化为度；none → 0
        assert_eq!(
            parse_color("hsl(0.5turn 100% 50%)"),
            Some(ColorValue::Hsla(180.0, 100.0, 50.0, 1.0))
        );
        assert_eq!(
            parse_color("hsl(none 100% 50%)"),
            Some(ColorValue::Hsla(0.0, 100.0, 50.0, 1.0))
        );
        assert_eq!(
            parse_color("hsl(120 100% none)"),
            Some(ColorValue::Hsla(120.0, 100.0, 0.0, 1.0))
        );
    }

    #[test]
    fn test_hsl_negative_hue_stored_raw() {
        // 负值 H 原样存储，归一化在 hsla_to_rgba 渲染时（R2253 hue-angle 修复）
        assert_eq!(
            parse_color("hsl(-300 100% 50%)"),
            Some(ColorValue::Hsla(-300.0, 100.0, 50.0, 1.0))
        );
    }

    // ── parse_rgb_function 现代/遗留语法（R2253，CSS Color 4）─────────────

    #[test]
    fn test_rgb_modern_whitespace_and_slash() {
        // 现代空白语法（rgb-001 driving）+ 斜杠 alpha + none
        assert_eq!(parse_color("rgb(0% 50% 0%)"), Some(ColorValue::Rgba(0, 128, 0, 255)));
        assert_eq!(parse_color("rgb(0 0 0 / 0.5)"), Some(ColorValue::Rgba(0, 0, 0, 128)));
        assert_eq!(parse_color("rgb(0 0 0 / 50%)"), Some(ColorValue::Rgba(0, 0, 0, 128)));
        assert_eq!(
            parse_color("rgb(none 255 none)"),
            Some(ColorValue::Rgba(0, 255, 0, 255))
        );
    }

    #[test]
    fn test_rgb_legacy_comma_regression() {
        // 遗留逗号语法不回归
        assert_eq!(parse_color("rgb(0, 128, 0)"), Some(ColorValue::Rgba(0, 128, 0, 255)));
        assert_eq!(parse_color("rgba(0, 0, 0, 0.5)"), Some(ColorValue::Rgba(0, 0, 0, 128)));
    }

    // ── CSS 系统颜色（R2254：deprecated ≡ 现代等价）─────────────────────

    #[test]
    fn test_deprecated_system_colors_equal_modern() {
        // deprecated-sameas 测试为相对匹配：deprecated 色须 == 其现代等价。
        let modern = |name: &str| parse_color(name);
        // ActiveBorder == ButtonBorder
        assert_eq!(parse_color("ActiveBorder"), modern("ButtonBorder"));
        assert_eq!(parse_color("ThreeDShadow"), modern("ButtonBorder"));
        assert_eq!(parse_color("WindowFrame"), modern("ButtonBorder"));
        // CaptionText == CanvasText
        assert_eq!(parse_color("CaptionText"), modern("CanvasText"));
        assert_eq!(parse_color("WindowText"), modern("CanvasText"));
        // Window == Canvas
        assert_eq!(parse_color("Window"), modern("Canvas"));
        assert_eq!(parse_color("Scrollbar"), modern("Canvas"));
        // ButtonHighlight == ButtonFace
        assert_eq!(parse_color("ButtonHighlight"), modern("ButtonFace"));
        assert_eq!(parse_color("ThreeDFace"), modern("ButtonFace"));
        // InactiveCaptionText == GrayText
        assert_eq!(parse_color("InactiveCaptionText"), modern("GrayText"));
        // 现代色大小写不敏感
        assert_eq!(parse_color("canvas"), parse_color("Canvas"));
        assert_eq!(parse_color("graytext"), parse_color("GrayText"));
    }

    // ── CSS Color 4 color() 预定义颜色空间（R2255）──────────────────────

    #[test]
    fn test_color_function_srgb() {
        // srgb 空间 = 普通 sRGB 分量（0-1）
        assert_eq!(parse_color("color(srgb 0 0 0)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
        assert_eq!(
            parse_color("color(srgb 1 1 1)"),
            Some(ColorValue::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(srgb 0 0.5 0)"),
            Some(ColorValue::Rgba(0, 128, 0, 255))
        );
        // slash alpha
        assert_eq!(
            parse_color("color(srgb 1 0 0 / 0.5)"),
            Some(ColorValue::Rgba(255, 0, 0, 128))
        );
    }

    #[test]
    fn test_color_function_wide_gamut_round_trip() {
        // a98rgb-001 driving：sRGB green #008000 转 a98-rgb 坐标，转回应得 green。
        let c = parse_color("color(a98-rgb 0.281363 0.498012 0.116746)").unwrap();
        assert_eq!(c, ColorValue::Rgba(0, 128, 0, 255));
        // display-p3 red (1,0,0) → sRGB red（钳制后 ≈ 255,0,0）
        let c = parse_color("color(display-p3 1 0 0)").unwrap();
        assert_eq!(c, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_color_function_unsupported_space() {
        // 未知空间 → None（color 声明被拒绝，cascade 丢弃）。rec2020/prophoto 已支持。
        assert_eq!(parse_color("color(unknown-space 0 0 0)"), None);
        // rec2020/prophoto-rgb 黑色 round-trip → sRGB 黑
        assert_eq!(
            parse_color("color(rec2020 0 0 0)"),
            Some(ColorValue::Rgba(0, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(prophoto-rgb 0 0 0)"),
            Some(ColorValue::Rgba(0, 0, 0, 255))
        );
    }

    // ── light-dark()（CSS Color Adjust，R2259）───────────────────────────

    #[test]
    fn test_light_dark_resolves_to_light() {
        // 默认 color-scheme = light → 取首个（light）参数。driving: light-dark-inheritance。
        assert_eq!(parse_color("light-dark(green, red)"), parse_color("green"));
        assert_eq!(parse_color("light-dark(#008000, red)"), parse_color("#008000"));
        // 嵌套 color() / hsl 作 light 参数
        assert_eq!(
            parse_color("light-dark(rgb(0,128,0), white)"),
            parse_color("rgb(0,128,0)")
        );
        // 参数内含逗号（color-mix）——顶层逗号切分，首个参数完整取出
        assert_eq!(
            parse_color("light-dark(color(srgb 0 0.5 0), red)"),
            parse_color("color(srgb 0 0.5 0)")
        );
        // currentColor 作 light 参数（透传，paint 时解析）
        assert_eq!(
            parse_color("light-dark(currentColor, red)"),
            parse_color("currentColor")
        );
    }

    // ── hwb_to_rgba ─────────────────────────────────────────────────────

    #[test]
    fn test_hwb_red() {
        // hwb(0, 0, 0) = pure red
        let (r, g, b, _a) = hwb_to_rgba(0.0, 0.0, 0.0, 1.0);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hwb_green() {
        // hwb(120, 0, 0) = pure green
        let (r, g, b, _a) = hwb_to_rgba(120.0, 0.0, 0.0, 1.0);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hwb_blue() {
        // hwb(240, 0, 0) = pure blue
        let (r, g, b, _a) = hwb_to_rgba(240.0, 0.0, 0.0, 1.0);
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_hwb_white() {
        // hwb(0, 1, 0) = pure white
        let (r, g, b, _a) = hwb_to_rgba(0.0, 1.0, 0.0, 1.0);
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_hwb_black() {
        // hwb(0, 0, 1) = pure black
        let (r, g, b, _a) = hwb_to_rgba(0.0, 0.0, 1.0, 1.0);
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hwb_alpha() {
        let (_r, _g, _b, a) = hwb_to_rgba(0.0, 0.0, 0.0, 0.5);
        assert_eq!(a, 128);
    }

    #[test]
    fn test_hwb_w_plus_b_exceeds_1() {
        // w+b > 1 should be scaled down
        let (r, g, b, _a) = hwb_to_rgba(0.0, 0.8, 0.8, 1.0);
        // After scaling: w=0.5, b=0.5, factor=0
        // result = pure_color * 0 + 0.5 = (128, 128, 128) for all sectors
        assert_eq!(r, g); // gray
        assert_eq!(g, b);
    }

    #[test]
    fn test_hwb_yellow() {
        // hwb(60, 0, 0) = yellow
        let (r, g, b, _a) = hwb_to_rgba(60.0, 0.0, 0.0, 1.0);
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    // ── parse_display ───────────────────────────────────────────────────

    #[test]
    fn test_parse_display_all() {
        assert_eq!(parse_display("block"), Some(DisplayValue::Block));
        assert_eq!(parse_display("inline"), Some(DisplayValue::Inline));
        assert_eq!(parse_display("inline-block"), Some(DisplayValue::InlineBlock));
        assert_eq!(parse_display("flex"), Some(DisplayValue::Flex));
        assert_eq!(parse_display("inline-flex"), Some(DisplayValue::InlineFlex));
        assert_eq!(parse_display("grid"), Some(DisplayValue::Grid));
        assert_eq!(parse_display("inline-grid"), Some(DisplayValue::InlineGrid));
        assert_eq!(parse_display("none"), Some(DisplayValue::None));
        assert_eq!(parse_display("contents"), Some(DisplayValue::Contents));
        assert_eq!(parse_display("flow"), Some(DisplayValue::Flow));
        assert_eq!(parse_display("flow-root"), Some(DisplayValue::FlowRoot));
        assert_eq!(parse_display("list-item"), Some(DisplayValue::ListItem));
        assert_eq!(parse_display("unknown"), None);
    }

    // ── parse_position ──────────────────────────────────────────────────

    #[test]
    fn test_parse_position_all() {
        assert_eq!(parse_position("static"), Some(PositionValue::Static));
        assert_eq!(parse_position("relative"), Some(PositionValue::Relative));
        assert_eq!(parse_position("absolute"), Some(PositionValue::Absolute));
        assert_eq!(parse_position("fixed"), Some(PositionValue::Fixed));
        assert_eq!(parse_position("sticky"), Some(PositionValue::Sticky));
        assert_eq!(parse_position("other"), None);
    }

    // ── parse_overflow ──────────────────────────────────────────────────

    #[test]
    fn test_parse_overflow_all() {
        assert_eq!(parse_overflow("visible"), Some(OverflowValue::Visible));
        assert_eq!(parse_overflow("hidden"), Some(OverflowValue::Hidden));
        assert_eq!(parse_overflow("scroll"), Some(OverflowValue::Scroll));
        assert_eq!(parse_overflow("auto"), Some(OverflowValue::Auto));
        assert_eq!(parse_overflow("clip"), Some(OverflowValue::Clip));
        assert_eq!(parse_overflow("inherit"), None);
    }

    // ── parse_float / parse_clear ───────────────────────────────────────

    #[test]
    fn test_parse_float_all() {
        assert_eq!(parse_float("none"), Some(FloatValue::None));
        assert_eq!(parse_float("left"), Some(FloatValue::Left));
        assert_eq!(parse_float("right"), Some(FloatValue::Right));
        assert_eq!(parse_float("inline-start"), Some(FloatValue::InlineStart));
        assert_eq!(parse_float("inline-end"), Some(FloatValue::InlineEnd));
    }

    #[test]
    fn test_parse_float_case_insensitive() {
        assert_eq!(parse_float("LEFT"), Some(FloatValue::Left));
        assert_eq!(parse_float("Right"), Some(FloatValue::Right));
    }

    #[test]
    fn test_parse_clear_all() {
        assert_eq!(parse_clear("none"), Some(ClearValue::None));
        assert_eq!(parse_clear("left"), Some(ClearValue::Left));
        assert_eq!(parse_clear("right"), Some(ClearValue::Right));
        assert_eq!(parse_clear("both"), Some(ClearValue::Both));
        assert_eq!(parse_clear("inline-start"), Some(ClearValue::InlineStart));
        assert_eq!(parse_clear("inline-end"), Some(ClearValue::InlineEnd));
    }

    // ── parse_flex_direction / parse_flex_wrap ──────────────────────────

    #[test]
    fn test_parse_flex_direction_all() {
        assert_eq!(parse_flex_direction("row"), Some(FlexDirectionValue::Row));
        assert_eq!(
            parse_flex_direction("row-reverse"),
            Some(FlexDirectionValue::RowReverse)
        );
        assert_eq!(parse_flex_direction("column"), Some(FlexDirectionValue::Column));
        assert_eq!(
            parse_flex_direction("column-reverse"),
            Some(FlexDirectionValue::ColumnReverse)
        );
    }

    #[test]
    fn test_parse_flex_wrap_all() {
        assert_eq!(parse_flex_wrap("nowrap"), Some(FlexWrapValue::Nowrap));
        assert_eq!(parse_flex_wrap("wrap"), Some(FlexWrapValue::Wrap));
        assert_eq!(parse_flex_wrap("wrap-reverse"), Some(FlexWrapValue::WrapReverse));
    }

    // ── parse_alignment ─────────────────────────────────────────────────

    #[test]
    fn test_parse_alignment_all() {
        assert_eq!(parse_alignment("flex-start"), Some(AlignmentValue::FlexStart));
        assert_eq!(parse_alignment("flex-end"), Some(AlignmentValue::FlexEnd));
        assert_eq!(parse_alignment("center"), Some(AlignmentValue::Center));
        assert_eq!(parse_alignment("space-between"), Some(AlignmentValue::SpaceBetween));
        assert_eq!(parse_alignment("space-around"), Some(AlignmentValue::SpaceAround));
        assert_eq!(parse_alignment("space-evenly"), Some(AlignmentValue::SpaceEvenly));
        assert_eq!(parse_alignment("stretch"), Some(AlignmentValue::Stretch));
        assert_eq!(parse_alignment("start"), Some(AlignmentValue::Start));
        assert_eq!(parse_alignment("end"), Some(AlignmentValue::End));
        assert_eq!(parse_alignment("baseline"), Some(AlignmentValue::Baseline));
        assert_eq!(parse_alignment("invalid"), None);
    }

    // ── parse_box_sizing / parse_visibility ─────────────────────────────

    #[test]
    fn test_parse_box_sizing() {
        assert_eq!(parse_box_sizing("content-box"), Some(BoxSizingValue::ContentBox));
        assert_eq!(parse_box_sizing("border-box"), Some(BoxSizingValue::BorderBox));
        assert_eq!(parse_box_sizing("auto"), None);
    }

    #[test]
    fn test_parse_visibility() {
        assert_eq!(parse_visibility("visible"), Some(VisibilityValue::Visible));
        assert_eq!(parse_visibility("hidden"), Some(VisibilityValue::Hidden));
        assert_eq!(parse_visibility("collapse"), Some(VisibilityValue::Collapse));
    }

    // ── parse_word_break / parse_writing_mode ───────────────────────────

    #[test]
    fn test_parse_word_break() {
        assert_eq!(parse_word_break("normal"), Some(WordBreakValue::Normal));
        assert_eq!(parse_word_break("break-all"), Some(WordBreakValue::BreakAll));
        assert_eq!(parse_word_break("keep-all"), Some(WordBreakValue::KeepAll));
        assert_eq!(parse_word_break("break-word"), Some(WordBreakValue::BreakWord));
    }

    #[test]
    fn test_parse_writing_mode() {
        assert_eq!(
            parse_writing_mode("horizontal-tb"),
            Some(WritingModeValue::HorizontalTb)
        );
        assert_eq!(parse_writing_mode("vertical-rl"), Some(WritingModeValue::VerticalRl));
        assert_eq!(parse_writing_mode("vertical-lr"), Some(WritingModeValue::VerticalLr));
    }

    // ── parse_text_decoration_line / parse_text_transform ───────────────

    #[test]
    fn test_parse_text_decoration_line() {
        assert_eq!(parse_text_decoration_line("none"), Some(TextDecorationLineValue::None));
        assert_eq!(
            parse_text_decoration_line("underline"),
            Some(TextDecorationLineValue::Underline)
        );
        assert_eq!(
            parse_text_decoration_line("overline"),
            Some(TextDecorationLineValue::Overline)
        );
        assert_eq!(
            parse_text_decoration_line("line-through"),
            Some(TextDecorationLineValue::LineThrough)
        );
    }

    #[test]
    fn test_parse_text_transform() {
        assert_eq!(parse_text_transform("none"), Some(TextTransformValue::None));
        assert_eq!(parse_text_transform("uppercase"), Some(TextTransformValue::Uppercase));
        assert_eq!(parse_text_transform("lowercase"), Some(TextTransformValue::Lowercase));
        assert_eq!(parse_text_transform("capitalize"), Some(TextTransformValue::Capitalize));
    }

    // ── parse_text_emphasis_style / parse_text_emphasis_position ───────

    #[test]
    fn test_parse_text_emphasis_style_none() {
        assert_eq!(parse_text_emphasis_style("none"), Some(TextEmphasisStyleValue::None));
        assert_eq!(parse_text_emphasis_style("NONE"), Some(TextEmphasisStyleValue::None));
    }

    #[test]
    fn test_parse_text_emphasis_style_keywords() {
        // 默认 filled → filled dot (•)
        assert_eq!(
            parse_text_emphasis_style("filled"),
            Some(TextEmphasisStyleValue::Char('\u{2022}'))
        );
        assert_eq!(
            parse_text_emphasis_style("dot"),
            Some(TextEmphasisStyleValue::Char('\u{2022}'))
        );
        // filled/open × shape 任意顺序
        assert_eq!(
            parse_text_emphasis_style("filled circle"),
            Some(TextEmphasisStyleValue::Char('\u{25CF}'))
        );
        assert_eq!(
            parse_text_emphasis_style("circle open"),
            Some(TextEmphasisStyleValue::Char('\u{25CB}'))
        );
        assert_eq!(
            parse_text_emphasis_style("open sesame"),
            Some(TextEmphasisStyleValue::Char('\u{FE46}'))
        );
        assert_eq!(
            parse_text_emphasis_style("filled triangle"),
            Some(TextEmphasisStyleValue::Char('\u{25B2}'))
        );
    }

    #[test]
    fn test_parse_text_emphasis_style_string() {
        assert_eq!(
            parse_text_emphasis_style("\"X\""),
            Some(TextEmphasisStyleValue::Char('X'))
        );
        // 空字符串视为无效（不产生标记）
        assert_eq!(parse_text_emphasis_style("\"\""), None);
        // 多字符取首字符
        assert_eq!(
            parse_text_emphasis_style("\"foo\""),
            Some(TextEmphasisStyleValue::Char('f'))
        );
    }

    #[test]
    fn test_parse_text_emphasis_style_invalid() {
        assert!(parse_text_emphasis_style("not-a-keyword").is_none());
        assert!(parse_text_emphasis_style("filled unknown").is_none());
    }

    #[test]
    fn test_parse_text_emphasis_position() {
        use TextEmphasisPositionValue::*;
        assert_eq!(parse_text_emphasis_position("over right"), Some(OverRight));
        assert_eq!(parse_text_emphasis_position("over left"), Some(OverLeft));
        assert_eq!(parse_text_emphasis_position("under right"), Some(UnderRight));
        assert_eq!(parse_text_emphasis_position("under left"), Some(UnderLeft));
        // 缺省 right/left → right
        assert_eq!(parse_text_emphasis_position("over"), Some(OverRight));
        assert_eq!(parse_text_emphasis_position("under"), Some(UnderRight));
        assert!(parse_text_emphasis_position("beside").is_none());
    }

    // ── parse_font_weight ───────────────────────────────────────────────

    #[test]
    fn test_parse_font_weight_keywords() {
        assert_eq!(parse_font_weight("normal"), Some(FontWeightValue::Normal));
        assert_eq!(parse_font_weight("bold"), Some(FontWeightValue::Bold));
        assert_eq!(parse_font_weight("bolder"), Some(FontWeightValue::Bolder));
        assert_eq!(parse_font_weight("lighter"), Some(FontWeightValue::Lighter));
    }

    #[test]
    fn test_parse_font_weight_numeric() {
        assert_eq!(parse_font_weight("400"), Some(FontWeightValue::Absolute(400)));
        assert_eq!(parse_font_weight("700"), Some(FontWeightValue::Absolute(700)));
        assert_eq!(parse_font_weight("100"), Some(FontWeightValue::Absolute(100)));
        assert_eq!(parse_font_weight("900"), Some(FontWeightValue::Absolute(900)));
        assert_eq!(parse_font_weight("50"), None); // out of range
        assert_eq!(parse_font_weight("950"), None); // out of range
    }

    // ── parse_font_style ────────────────────────────────────────────────

    #[test]
    fn test_parse_font_style_normal() {
        assert_eq!(parse_font_style("normal"), Some(FontStyleValue::Normal));
    }

    #[test]
    fn test_parse_font_style_italic() {
        assert_eq!(parse_font_style("italic"), Some(FontStyleValue::Italic));
    }

    #[test]
    fn test_parse_font_style_oblique_no_angle() {
        assert_eq!(parse_font_style("oblique"), Some(FontStyleValue::Oblique(None)));
    }

    #[test]
    fn test_parse_font_style_oblique_with_angle() {
        assert_eq!(
            parse_font_style("oblique 14deg"),
            Some(FontStyleValue::Oblique(Some(14.0)))
        );
    }

    #[test]
    fn test_parse_font_style_unknown() {
        assert_eq!(parse_font_style("unknown"), None);
    }

    // ── parse_list_style_type / parse_list_style_position ───────────────

    #[test]
    fn test_parse_list_style_type() {
        assert_eq!(parse_list_style_type("disc"), Some(ListStyleTypeValue::Disc));
        assert_eq!(parse_list_style_type("circle"), Some(ListStyleTypeValue::Circle));
        assert_eq!(parse_list_style_type("square"), Some(ListStyleTypeValue::Square));
        assert_eq!(parse_list_style_type("decimal"), Some(ListStyleTypeValue::Decimal));
        assert_eq!(parse_list_style_type("none"), Some(ListStyleTypeValue::None));
        assert_eq!(
            parse_list_style_type("lower-alpha"),
            Some(ListStyleTypeValue::LowerAlpha)
        );
        assert_eq!(
            parse_list_style_type("upper-latin"),
            Some(ListStyleTypeValue::UpperAlpha)
        );
    }

    #[test]
    fn test_parse_list_style_position() {
        assert_eq!(
            parse_list_style_position("outside"),
            Some(ListStylePositionValue::Outside)
        );
        assert_eq!(
            parse_list_style_position("inside"),
            Some(ListStylePositionValue::Inside)
        );
        assert_eq!(parse_list_style_position("center"), None);
    }

    // ── parse_spacing ───────────────────────────────────────────────────

    #[test]
    fn test_parse_spacing_normal() {
        assert_eq!(parse_spacing("normal"), Some(LengthValue::Px(0.0)));
    }

    #[test]
    fn test_parse_spacing_px() {
        assert_eq!(parse_spacing("5px"), Some(LengthValue::Px(5.0)));
    }

    // ── parse_color_quirks ──────────────────────────────────────────────

    #[test]
    fn test_quirks_color_standard_still_works() {
        // 标准格式在 quirks mode 下仍然正常解析
        assert!(parse_color_quirks("red").is_some());
        assert!(parse_color_quirks("#FF0000").is_some());
        assert!(parse_color_quirks("rgb(255, 0, 0)").is_some());
    }

    #[test]
    fn test_quirks_color_hashless_hex() {
        // 不带 # 的十六进制
        let c = parse_color_quirks("FF0000").unwrap();
        assert_eq!(c, ColorValue::Rgba(255, 0, 0, 255));

        let c = parse_color_quirks("f00").unwrap();
        assert_eq!(c, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_quirks_color_numeric() {
        // 纯数字 → 24-bit RGB
        let c = parse_color_quirks("0").unwrap();
        assert_eq!(c, ColorValue::Rgba(0, 0, 0, 255));

        let c = parse_color_quirks("16711680").unwrap(); // 0xFF0000
        assert_eq!(c, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_quirks_color_invalid_still_none() {
        assert!(parse_color_quirks("not-a-color").is_none());
        assert!(parse_color_quirks("").is_none());
    }
}
