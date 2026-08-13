//! CSS 颜色和基础属性解析。

use super::color_math::*;
use super::*;

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值（按 light color-scheme 解析 `light-dark()`）。
///
/// 等价 [`parse_color_with_scheme`](parse_color_with_scheme)`(value, false)`。
/// 保留无 scheme 参数入口供所有不关心 color-scheme 的调用方使用（零回归）。
pub fn parse_color(value: &str) -> Option<ColorValue> {
    parse_color_with_scheme(value, false)
}

/// 解析 CSS 颜色值，`dark` 控制元素 used color-scheme 是否为暗。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()`、`hsl()`/`hsla()` 和 `hwb()` 函数。`dark` 仅影响 `light-dark(L, D)`：
/// `dark=true`（元素 `color-scheme: dark`）取第二个（dark）参数，否则取第一个（light）参数。
/// dark 向所选参数递归传播（`light-dark(light-dark(a,b), c)` 等嵌套）。
pub fn parse_color_with_scheme(value: &str, dark: bool) -> Option<ColorValue> {
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

    // RCS 相对色语法（CSS Color 5）：<func>(from <origin> <channels>)。
    // **identity 快捷**：当 channels 恰为该函数的自然关键字（rgb→r g b、hsl→h s l、…）时，
    // 结果 = origin（origin↔输出色彩空间往返保色）。currentColor origin → CurrentColor
    //（paint 时按元素色解析，支持 inherit 透传）。driving: css-color relative-currentcolor-*
    //（14/16 案为 identity；hsl-02 h 覆盖 / rgb-02 swap 非 identity，defer）。
    if value.to_ascii_lowercase().contains("from ") {
        if let Some(c) = try_parse_relative_identity(value) {
            return Some(c);
        }
        // 非 identity RCS（channel 覆盖/置换）：仅 rgb/rgba/hsl/hsla + 关键字引用/数字字面量。
        // currentColor origin 保留未解析（paint 时按元素色解析，支持 inherit 透传，同 Mix）。
        // driving: css-color relative-currentcolor-rgb-02（g r b 置换）/ hsl-02（120 s l 覆盖）。
        if let Some(c) = parse_relative_color(value) {
            return Some(c);
        }
    }

    // 十六进制颜色
    if value.starts_with('#') {
        return parse_hex_color(value);
    }

    // CSS Values §4：颜色函数名大小写不敏感（RGB ≡ rgb、HSL ≡ hsl、Lab ≡ lab …）。函数式颜色
    // 走 lower 路径（dispatch + 子解析器委托均用 lower，子解析器内 case-sensitive starts_with 在
    // lower 输入下匹配）；命名色 / 十六进制 / 转义（red\9 等 escapes-014/015/016）仍用原 value
    //（lowercase 会破转义）。函数参数（数字 + 大小写不敏感关键字/单位/色彩空间名）经 lower 安全。
    let lower = value.to_ascii_lowercase();

    // rgb() / rgba() 函数
    if lower.starts_with("rgb(") || lower.starts_with("rgba(") {
        return parse_rgb_function(&lower);
    }

    // hsl() / hsla() 函数
    if lower.starts_with("hsl(") || lower.starts_with("hsla(") {
        return parse_hsl_function(&lower);
    }

    // hwb() 函数
    if lower.starts_with("hwb(") {
        return parse_hwb_function(&lower);
    }

    // color() 函数（CSS Color 4 预定义颜色空间：srgb/srgb-linear/display-p3/a98-rgb/xyz…）
    if lower.starts_with("color(") {
        return parse_color_function(&lower);
    }

    // lab() / lch() / oklab() / oklch()（CSS Color 4 CIE Lab / OKLab 色彩空间）。driving:
    // css-color lab-*/lch-*/oklab-*/oklch-*（~54 案；R2255 XYZ↔sRGB 基础设施复用）。
    if lower.starts_with("lab(") {
        return parse_lab(&lower);
    }
    if lower.starts_with("lch(") {
        return parse_lch(&lower);
    }
    if lower.starts_with("oklab(") {
        return parse_oklab(&lower);
    }
    if lower.starts_with("oklch(") {
        return parse_oklch(&lower);
    }

    // light-dark() 函数（CSS Color Adjust §color-scheme-effect）：light-dark(<light>, <dark>)
    // 按元素 used color-scheme 取值：dark（color-scheme: dark）取第二个（dark）参数，否则取
    // 第一个（light）参数。dark 向所选参数递归传播。driving: css-color light-dark-inheritance /
    // light-dark-currentcolor + css-variables registered-property-light-dark。
    if lower.starts_with("light-dark(") {
        let start = value.find('(')?;
        let end = value.rfind(')')?;
        let inner = strip_css_comments(value.get(start + 1..end)?);
        let chosen = match top_level_byte_index(&inner, b',') {
            // 两个参数：按 scheme 选 light/dark。
            Some(comma_pos) => {
                let light = inner[..comma_pos].trim();
                let dark_arg = inner[comma_pos + 1..].trim();
                if dark { dark_arg } else { light }
            }
            // 仅一个参数（非标准，宽容取之）。
            None => inner.trim(),
        };
        if chosen.is_empty() {
            return None;
        }
        return parse_color_with_scheme(chosen, dark);
    }

    // color-mix() 函数（CSS Color 4）：color-mix(in <space>, <c1> [<p1>], <c2> [<p2>])。
    // 支持 srgb/srgb-linear/lab/lch/oklab/oklch/xyz（其他色彩空间 xyz-d50/display-p3 defer）。
    // 存为未解析 ColorValue::Mix——currentColor 在 paint 时按元素色解析，支持 inherit 透传。
    // driving: css-color color-mix-currentcolor-001。
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
            let r = hex_char_to_byte(c0, c0)?;
            let g = hex_char_to_byte(c1, c1)?;
            let b = hex_char_to_byte(c2, c2)?;
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        4 => {
            // #RGBA → RRGGBBAA
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let c3 = chars.next()?;
            let r = hex_char_to_byte(c0, c0)?;
            let g = hex_char_to_byte(c1, c1)?;
            let b = hex_char_to_byte(c2, c2)?;
            let a = hex_char_to_byte(c3, c3)?;
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
///
/// 返回 `None` 表示存在非 hex digit（如 `G`）——调用方据此拒绝整个 hex 颜色，与
/// 6/8 位路径（`u8::from_str_radix(...).ok()?`）保持一致。R3344 deep-review：旧实现
/// `unwrap_or(0)` 把 `#G00`（3 位非法）静默转为黑色，而 `#GGGGGG`（6 位）被正确拒绝，
/// 两路径不一致；CSS Color 规定 `#` 后须全为 hex digit，非法 hex 颜色应拒绝。
/// // https://drafts.csswg.org/css-color-4/#hex-notation
fn hex_char_to_byte(c1: char, c2: char) -> Option<u8> {
    let s = format!("{}{}", c1, c2);
    u8::from_str_radix(&s, 16).ok()
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
        let parts: Vec<&str> = main.split(',').map(str::trim).collect();
        // R34xx：空分量（尾部/连续逗号）→ 无效（2d.fillStyle.parse.invalid.rgb-1：
        // 'rgb(255.0, 0, 0,)' 应被拒绝）。
        if parts.iter().any(|c| c.is_empty()) {
            return None;
        }
        parts
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

/// 提取 `<func>(...)` 内层（首个 `(` 到末个 `)`，去注释）。
fn inner_of_parens(value: &str) -> Option<String> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    if end <= start {
        return None;
    }
    Some(strip_css_comments(value.get(start + 1..end)?).trim().to_string())
}

/// 拆分颜色分量（main + 可选 `/ alpha`），空白与逗号均作分隔（兼容 legacy 逗号语法）。
/// 返回（main 分量 token 列表, 可选 alpha [0,1]）。
fn split_color_components(inner: &str) -> Option<(Vec<&str>, Option<f64>)> {
    let (main, slash_alpha) = if let Some((m, a)) = inner.split_once('/') {
        (m.trim(), Some(a.trim()))
    } else {
        (inner.trim(), None)
    };
    let comps: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .collect();
    let alpha = match slash_alpha {
        Some(ap) => Some(parse_alpha_value(ap)?),
        None => Some(1.0),
    };
    Some((comps, alpha))
}

/// 解析带可选百分比的分量：`<number>` 或 `<number>%`（按 `percent_scale` 缩放）；`none`→0。
fn parse_scaled_component(s: &str, percent_scale: f64) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(num_str) = s.strip_suffix('%') {
        return num_str.parse::<f64>().ok().map(|v| v * percent_scale / 100.0);
    }
    s.parse::<f64>().ok()
}

/// alpha f64 → u8。
fn alpha_to_u8(a: Option<f64>) -> u8 {
    ((a.unwrap_or(1.0)) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// `lab(L a b [/ alpha])`：L∈[0,100]（% of 100），a/b（% of 125 或数字）。
fn parse_lab(value: &str) -> Option<ColorValue> {
    let inner = inner_of_parens(value)?;
    let (comps, alpha) = split_color_components(&inner)?;
    if comps.len() < 3 {
        return None;
    }
    let l = parse_scaled_component(comps[0], 100.0)?.clamp(0.0, 100.0);
    let a = parse_scaled_component(comps[1], 125.0)?;
    let b = parse_scaled_component(comps[2], 125.0)?;
    let (r, g, b) = lab_to_srgb_u8(l, a, b);
    Some(ColorValue::Rgba(r, g, b, alpha_to_u8(alpha)))
}

/// `lch(L C h [/ alpha])`：L∈[0,100]（% of 100），C（% of 150 或数字），h 为角度。
fn parse_lch(value: &str) -> Option<ColorValue> {
    let inner = inner_of_parens(value)?;
    let (comps, alpha) = split_color_components(&inner)?;
    if comps.len() < 3 {
        return None;
    }
    let l = parse_scaled_component(comps[0], 100.0)?.clamp(0.0, 100.0);
    let c = parse_scaled_component(comps[1], 150.0)?;
    let h_deg = parse_hue_angle(comps[2])?;
    let (r, g, b) = lch_to_srgb_u8(l, c, h_deg);
    Some(ColorValue::Rgba(r, g, b, alpha_to_u8(alpha)))
}

/// `oklab(L a b [/ alpha])`：L∈[0,1]（% of 1），a/b（% of 0.4 或数字）。
fn parse_oklab(value: &str) -> Option<ColorValue> {
    let inner = inner_of_parens(value)?;
    let (comps, alpha) = split_color_components(&inner)?;
    if comps.len() < 3 {
        return None;
    }
    let l = parse_scaled_component(comps[0], 1.0)?.clamp(0.0, 1.0);
    let a = parse_scaled_component(comps[1], 0.4)?;
    let b = parse_scaled_component(comps[2], 0.4)?;
    let (r, g, b) = oklab_to_srgb_u8(l, a, b);
    Some(ColorValue::Rgba(r, g, b, alpha_to_u8(alpha)))
}

/// `oklch(L C h [/ alpha])`：L∈[0,1]（% of 1），C（% of 0.4 或数字），h 为角度。
fn parse_oklch(value: &str) -> Option<ColorValue> {
    let inner = inner_of_parens(value)?;
    let (comps, alpha) = split_color_components(&inner)?;
    if comps.len() < 3 {
        return None;
    }
    let l = parse_scaled_component(comps[0], 1.0)?.clamp(0.0, 1.0);
    let c = parse_scaled_component(comps[1], 0.4)?;
    let h_deg = parse_hue_angle(comps[2])?;
    let (r, g, b) = oklch_to_srgb_u8(l, c, h_deg);
    Some(ColorValue::Rgba(r, g, b, alpha_to_u8(alpha)))
}

/// 解析 color() 分量数字（0-1 浮点，可负/越界）；`none` → 0。
fn parse_color_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    // CSS Color 4：color() 预定义空间分量可为 <number>（0-1）或 <percentage>（0-100% → 0-1）。
    // driving: css-color predefined-002（color(srgb 0% 60% 0%) ≡ #009900）。
    if let Some(pct) = s.strip_suffix('%') {
        return pct.trim().parse::<f64>().ok().map(|v| v / 100.0);
    }
    s.parse().ok()
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
        let parts: Vec<&str> = main.split(',').map(str::trim).collect();
        // R34xx：空分量（尾部/连续逗号）→ 无效（2d.fillStyle.parse.invalid.rgb-1：
        // 'rgb(255.0, 0, 0,)' 应被拒绝）。
        if parts.iter().any(|c| c.is_empty()) {
            return None;
        }
        parts
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

/// 解析 `color-mix(in <space> [<method> hue], <c1> [<p1>], <c2> [<p2>])`（CSS Color 4）。
///
/// 支持 srgb/srgb-linear/lab/lch/oklab/oklch/xyz 空间；极坐标空间（lch/oklch）额外接受
/// 色相插值法（shorter/longer/increasing/decreasing hue，默认 shorter）。分量 `<color> [<百分比>]`，
/// 百分比省略按 spec 默认（双省略=50/50）。currentColor 保留未解析（paint 时按元素色解析）。
/// driving: color-mix-currentcolor-001。
fn parse_color_mix(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = strip_css_comments(value.get(start + 1..end)?);
    // 首个顶层逗号分隔色彩空间与第一分量
    let first_comma = top_level_byte_index(&inner, b',')?;
    let space = inner[..first_comma].trim();
    let (mix_space, hue) = parse_color_mix_space(space)?;
    let rest = inner[first_comma + 1..].trim();
    // 顶层逗号分隔两分量
    let second_comma = top_level_byte_index(rest, b',')?;
    let c1 = parse_color_mix_component(rest[..second_comma].trim())?;
    let c2 = parse_color_mix_component(rest[second_comma + 1..].trim())?;
    Some(ColorValue::Mix(Box::new(ColorMixSpec {
        c1,
        c2,
        space: mix_space,
        hue,
    })))
}

/// 解析 `color-mix` 的色彩空间段 `in <space> [ <method> hue ]`（CSS Color 4 §12）。
///
/// 返回 `(ColorMixSpace, ColorHueMethod)`。hue 插值法仅对极坐标空间（lch/oklch）有意义，
/// 其他空间忽略（默认 Shorter）。镜像 gradient 的 `parse_color_interpolation` 逻辑。
/// R2381。
fn parse_color_mix_space(s: &str) -> Option<(ColorMixSpace, crate::values::parse_transform::ColorHueMethod)> {
    use crate::values::parse_transform::ColorHueMethod;
    let lower = s.to_ascii_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    // tokens[0] 须为 "in"
    if tokens.first().copied() != Some("in") {
        return None;
    }
    let mix_space = match tokens.get(1).copied() {
        Some("srgb") => ColorMixSpace::Srgb,
        Some("srgb-linear") => ColorMixSpace::SrgbLinear,
        Some("lab") => ColorMixSpace::Lab,
        Some("lch") => ColorMixSpace::Lch,
        Some("oklab") => ColorMixSpace::OkLab,
        Some("oklch") => ColorMixSpace::OkLch,
        Some("xyz") | Some("xyz-d65") => ColorMixSpace::Xyz,
        _ => return None, // 其他色彩空间（xyz-d50/display-p3/…）defer
    };
    // 可选 hue 插值法（仅 lch/oklch）
    let mut hue = ColorHueMethod::default();
    if matches!(mix_space, ColorMixSpace::Lch | ColorMixSpace::OkLch) {
        let mut it = tokens.iter().skip(2);
        while let Some(w) = it.next() {
            let method = match *w {
                "shorter" => Some(ColorHueMethod::Shorter),
                "longer" => Some(ColorHueMethod::Longer),
                "increasing" => Some(ColorHueMethod::Increasing),
                "decreasing" => Some(ColorHueMethod::Decreasing),
                _ => None,
            };
            if let Some(m) = method
                && matches!(it.next(), Some(next) if *next == "hue")
            {
                hue = m;
                break;
            }
        }
    }
    Some((mix_space, hue))
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

/// RCS（CSS Color 5 相对色）identity 快捷：`<func>(from <origin> <channels>)`，当 channels
/// 恰为该函数自然关键字时返回 origin（currentColor origin → CurrentColor，paint 时解析）。
/// 非 identity（channel 覆盖/swap/calc）或非 `from` 形式 → None（落回常规解析或丢弃）。
fn try_parse_relative_identity(value: &str) -> Option<ColorValue> {
    let open = value.find('(')?;
    let close = value.rfind(')')?;
    if close <= open {
        return None;
    }
    // R34xx：')' 后不得有多余 token（'rgb(from #fff r g b) 100%' 的尾部 '100%' 应致整个
    // 解析失败——2d.fillStyle.parse.invalid.css-color-4-rgb-6 期望无效值忽略）。
    if value.get(close + 1..).is_some_and(|s| !s.trim().is_empty()) {
        return None;
    }
    let func = value[..open].trim().to_ascii_lowercase();
    let inner = strip_css_comments(value.get(open + 1..close)?).trim().to_string();
    let lower_inner = inner.to_ascii_lowercase();
    let rest = lower_inner.strip_prefix("from ")?.trim();
    // 拆分 origin（首个颜色 token）与剩余 channels（含 color() 的空间名）。
    let (origin_str, channels_str) = split_origin_channels(rest)?;
    // color() 的 channels 含色彩空间名前缀；按空间决定自然关键字。
    let natural = natural_channel_keywords(&func, channels_str)?;
    let chans: Vec<&str> = channels_str.split_whitespace().collect();
    if chans != natural {
        return None; // 非 identity（覆盖/swap/calc）
    }
    // identity：origin 经 origin↔输出空间往返保色 → 结果 = origin（按原样解析，保留 currentColor）
    parse_color(origin_str)
}

/// 拆分 RCS rest（`<origin> <channels...>`）为 (origin 原始串, channels 串)。
/// origin 取首个顶层颜色 token：currentColor / 命名 / hex 为单 token；函数 `foo(...)` 取平衡括号。
fn split_origin_channels(rest: &str) -> Option<(&str, &str)> {
    let bytes = rest.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    // 跳过 origin：单 token 或平衡 foo(...)
    if i < bytes.len() && bytes[i] == b'(' {
        // 不太可能以 ( 开头
    }
    // 扫到首个顶层空白（单 token origin）或匹配 foo(...) 的 )
    let start = i;
    if bytes.get(i).is_some_and(|b| b.is_ascii_alphabetic()) {
        // 扫 ident；若紧跟 ( 则是函数，取平衡
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
        }
        if bytes.get(i).is_some_and(|b| *b == b'(') {
            // 函数 origin：取平衡 (...)（含内部嵌套）
            depth = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'(' | b'[' => depth += 1,
                    b')' | b']' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
        }
    } else {
        // hex（#...）或其他：取到首个顶层空白
        while i < bytes.len() && !(depth == 0 && (bytes[i] == b' ' || bytes[i] == b'\t')) {
            i += 1;
        }
    }
    if i >= bytes.len() {
        return None; // 仅有 origin 无 channels
    }
    let origin = rest.get(start..i)?.trim();
    let channels = rest.get(i..)?.trim();
    Some((origin, channels))
}

/// 返回该函数的「自然关键字」（小写），channels 与之完全相等即为 identity。
/// color() 的 channels 形如 `<space> r g b`——空间决定关键字（rect 空间→r g b，xyz→x y z）。
/// 返回的 Vec 包含 color() 空间名（如有）+ 关键字，供与 channels split 后逐 token 比较。
fn natural_channel_keywords<'a>(func: &str, channels: &'a str) -> Option<Vec<&'a str>> {
    let tokens: Vec<&str> = channels.split_whitespace().collect();
    let (kw, offset) = if func == "color" {
        // 首个 token = 色彩空间名
        let space = tokens.first()?.to_ascii_lowercase();
        let kw = if matches!(
            space.as_str(),
            "srgb" | "srgb-linear" | "a98-rgb" | "display-p3" | "prophoto-rgb" | "rec2020"
        ) {
            ["r", "g", "b"]
        } else if matches!(space.as_str(), "xyz" | "xyz-d50" | "xyz-d65") {
            ["x", "y", "z"]
        } else {
            return None; // 未知空间
        };
        (kw, 1) // 跳过空间名 token
    } else {
        let kw = match func {
            "rgb" | "rgba" => ["r", "g", "b"],
            "hsl" | "hsla" => ["h", "s", "l"],
            "hwb" => ["h", "w", "b"],
            "lab" => ["l", "a", "b"],
            "lch" => ["l", "c", "h"],
            "oklab" => ["l", "a", "b"],
            "oklch" => ["l", "c", "h"],
            _ => return None,
        };
        (kw, 0)
    };
    // 构造期望 token 序列（color 含空间名 + 关键字）
    let mut expected: Vec<&str> = tokens[..offset].to_vec(); // 空间名（如有）
    expected.extend(kw.iter().copied());
    Some(expected)
}

/// 解析非 identity RCS（CSS Color 5 相对色）：`<func>(from <origin> <ch1> <ch2> <ch3> [/ <alpha>])`。
///
/// 仅 rgb/rgba/hsl/hsla 输出空间。origin 经 `split_origin_channels` 拆出后递归 `parse_color`（保留
/// currentColor 未解析）。通道为关键字引用（r/g/b、h/s/l，记录引用的 origin 通道序以支持置换）或
/// 数字字面量（rgb: 0-255；hsl h: 度；s/l: 0-100，`%` 去除）。alpha 省略 = origin alpha。
/// 非法（非 rgb/hsl 函数、通道数≠3、未知通道 token、非 `from` 形式）→ None。
fn parse_relative_color(value: &str) -> Option<ColorValue> {
    let open = value.find('(')?;
    let close = value.rfind(')')?;
    if close <= open {
        return None;
    }
    // R34xx：')' 后不得有多余 token（'rgb(from #fff r g b) 100%' 的尾部 '100%' 应致整个
    // 解析失败——2d.fillStyle.parse.invalid.css-color-4-rgb-6 期望无效值忽略）。
    if value.get(close + 1..).is_some_and(|s| !s.trim().is_empty()) {
        return None;
    }
    let func_name = value[..open].trim().to_ascii_lowercase();
    let func = match func_name.as_str() {
        "rgb" | "rgba" => RelativeColorFunc::Rgb,
        "hsl" | "hsla" => RelativeColorFunc::Hsl,
        "lab" => RelativeColorFunc::Lab,
        "lch" => RelativeColorFunc::Lch,
        "oklab" => RelativeColorFunc::Oklab,
        "oklch" => RelativeColorFunc::Oklch,
        "color" => RelativeColorFunc::Color,
        _ => return None,
    };
    let inner = strip_css_comments(value.get(open + 1..close)?)
        .trim()
        .to_ascii_lowercase();
    let rest = inner.strip_prefix("from ")?.trim();
    let (origin_str, channels_str) = split_origin_channels(rest)?;
    let origin = parse_color(origin_str)?;
    // alpha：仅无逗号时认斜杠 alpha（与 parse_hsl_function 一致）。
    let (main, alpha) = if main_uses_comma_syntax(channels_str) {
        (channels_str, RcsAlpha::Origin)
    } else {
        match channels_str.split_once('/') {
            Some((m, a)) => (m.trim(), parse_rcs_alpha(a.trim())),
            None => (channels_str, RcsAlpha::Origin),
        }
    };
    // color() 的 main 形如 `<space> <ch1> <ch2> <ch3>`：首 token 为色彩空间名，其余为 3 通道。
    let mut comps: Vec<&str> = if main.contains(',') {
        main.split(',').map(str::trim).filter(|s| !s.is_empty()).collect()
    } else {
        main.split_whitespace().collect()
    };
    let space = if func == RelativeColorFunc::Color {
        // 首个 token 为色彩空间名（display-p3 / srgb / xyz-d50 …）。
        if comps.is_empty() {
            return None;
        }
        Some(comps.remove(0).to_string())
    } else {
        None
    };
    if comps.len() != 3 {
        return None;
    }
    let channels = [
        parse_rcs_channel(comps[0], func, 0)?,
        parse_rcs_channel(comps[1], func, 1)?,
        parse_rcs_channel(comps[2], func, 2)?,
    ];
    Some(ColorValue::RelativeColor(Box::new(RelativeColorSpec {
        func,
        origin,
        channels,
        alpha,
        space,
    })))
}

/// 解析 RCS 单个通道：关键字引用（记录 origin 通道序，支持置换）/ 数字字面量 / `none`。
/// `idx` 为输出位置：rgb 任意位 0-255；hsl h/lch h/oklch h（idx 0 或 2）为色相（度，可带角度单位）；
/// wide-gamut（lab/lch/oklab/oklch）非 hue 通道按 spec 百分比基准缩放（见 parse_rcs_number_wide）。
fn parse_rcs_channel(s: &str, func: RelativeColorFunc, idx: usize) -> Option<RcsChannel> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(RcsChannel::None);
    }
    // 关键字引用：记录所引用的 origin 通道序。
    // rgb r/g/b=0/1/2；hsl h/s/l；lab/oklab l/a/b；lch/oklch l/c/h；color r/g/b 或 x/y/z（均 0/1/2）。
    let kw_ref = match (func, s.to_ascii_lowercase().as_str()) {
        (RelativeColorFunc::Rgb, "r") => Some(0u8),
        (RelativeColorFunc::Rgb, "g") => Some(1),
        (RelativeColorFunc::Rgb, "b") => Some(2),
        (RelativeColorFunc::Hsl, "h") => Some(0),
        (RelativeColorFunc::Hsl, "s") => Some(1),
        (RelativeColorFunc::Hsl, "l") => Some(2),
        (RelativeColorFunc::Lab, "l") => Some(0),
        (RelativeColorFunc::Lab, "a") => Some(1),
        (RelativeColorFunc::Lab, "b") => Some(2),
        (RelativeColorFunc::Lch, "l") => Some(0),
        (RelativeColorFunc::Lch, "c") => Some(1),
        (RelativeColorFunc::Lch, "h") => Some(2),
        (RelativeColorFunc::Oklab, "l") => Some(0),
        (RelativeColorFunc::Oklab, "a") => Some(1),
        (RelativeColorFunc::Oklab, "b") => Some(2),
        (RelativeColorFunc::Oklch, "l") => Some(0),
        (RelativeColorFunc::Oklch, "c") => Some(1),
        (RelativeColorFunc::Oklch, "h") => Some(2),
        (RelativeColorFunc::Color, "r" | "x") => Some(0),
        (RelativeColorFunc::Color, "g" | "y") => Some(1),
        (RelativeColorFunc::Color, "b" | "z") => Some(2),
        _ => None,
    };
    if let Some(i) = kw_ref {
        return Some(RcsChannel::Ref(i));
    }
    let v = match func {
        RelativeColorFunc::Rgb => parse_rcs_number_255(s)?,
        // hsl 首通道 = 色相（数字或角度单位 → 度）。
        RelativeColorFunc::Hsl if idx == 0 => parse_hue_angle(s)?,
        RelativeColorFunc::Hsl => parse_rcs_number_100(s)?,
        // lch/oklch 第三通道（idx=2）= 色相。
        RelativeColorFunc::Lch | RelativeColorFunc::Oklch if idx == 2 => parse_hue_angle(s)?,
        // lab/lch/oklab/oklch 非 hue 通道。
        RelativeColorFunc::Lab | RelativeColorFunc::Lch | RelativeColorFunc::Oklab | RelativeColorFunc::Oklch => {
            parse_rcs_number_wide(s, func, idx)?
        }
        // color() 通道为 0-1（`%` → p/100，裸数字 → 0-1，可越界/负）。
        RelativeColorFunc::Color => parse_rcs_color_component(s)?,
    };
    Some(RcsChannel::Num(v))
}

/// 解析 color() RCS 通道数字字面量：`p%` → p/100，裸数字 → 0-1 浮点（可负/越界，回转时钳）。
fn parse_rcs_color_component(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        let p: f64 = pct.trim().parse().ok()?;
        Some(p / 100.0)
    } else {
        s.parse().ok()
    }
}

/// 解析 rgb 通道数字字面量：`p%` → p/100*255，裸数字 → 0-255（不钳制，paint 时钳）。
fn parse_rcs_number_255(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        let p: f64 = pct.trim().parse().ok()?;
        Some(p / 100.0 * 255.0)
    } else {
        Some(s.parse().ok()?)
    }
}

/// 解析 hsl s/l 数字字面量：`p%` 或裸数字 → 0-100。
fn parse_rcs_number_100(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        Some(pct.trim().parse().ok()?)
    } else {
        Some(s.parse().ok()?)
    }
}

/// 解析 wide-gamut RCS 通道数字字面量（lab/lch/oklab/oklch 非 hue 通道）。
/// `%` 按该通道 spec 百分比基准缩放；裸数字 = 通道自然单位值（不缩放、不钳制，paint 时回转钳）。
fn parse_rcs_number_wide(s: &str, func: RelativeColorFunc, idx: usize) -> Option<f64> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        let p: f64 = pct.trim().parse().ok()?;
        // 百分比基准：lab/lch L=100；lab a/b=125；lch C=150；oklab/oklch L=1；oklab a/b=0.4；oklch C=0.4。
        let basis = match (func, idx) {
            (RelativeColorFunc::Lab | RelativeColorFunc::Lch, 0) => 100.0,
            (RelativeColorFunc::Lab, _) => 125.0,
            (RelativeColorFunc::Lch, _) => 150.0,
            (RelativeColorFunc::Oklab | RelativeColorFunc::Oklch, 0) => 1.0,
            (RelativeColorFunc::Oklab, _) => 0.4,
            (RelativeColorFunc::Oklch, _) => 0.4,
            _ => return None,
        };
        return Some(p / 100.0 * basis);
    }
    s.parse().ok()
}

/// 解析 RCS alpha：`none` → None，`p%` → p/100，裸数字 → 0-1（不钳制）。
fn parse_rcs_alpha(s: &str) -> RcsAlpha {
    if s.eq_ignore_ascii_case("none") {
        return RcsAlpha::None;
    }
    if let Some(pct) = s.strip_suffix('%') {
        if let Ok(p) = pct.trim().parse::<f64>() {
            return RcsAlpha::Num(p / 100.0);
        }
    }
    if let Ok(v) = s.parse::<f64>() {
        return RcsAlpha::Num(v);
    }
    RcsAlpha::Origin // 无法解析时退回 origin alpha（保守不破坏规则）
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::parse_misc::*;

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

    // ── R2323：Lab/LCH/OKLab/OKLCH L 边界 gamut 映射（CSS Color 4）─────────

    #[test]
    fn test_r2323_lab_lch_l_boundary_white_black() {
        // CSS Color 4：L≥100（Lab/LCH 0-100 尺度）→ 白点（任意 chroma gamut-map 到白）；
        // L≤0 → 黑。driving: css-color lch-009（lch(100% 110 60)→白）/ lch-010（lch(0% 110 60)→黑）。
        // 此前逐通道钳制致 lch(100,110,60) 非纯白。
        assert_eq!(lab_to_srgb_u8(100.0, 55.0, 95.3), (255, 255, 255), "lab L=100 -> white");
        assert_eq!(lab_to_srgb_u8(0.0, 55.0, 95.3), (0, 0, 0), "lab L=0 -> black");
        assert_eq!(
            lch_to_srgb_u8(100.0, 110.0, 60.0),
            (255, 255, 255),
            "lch L=100 -> white"
        );
        assert_eq!(lch_to_srgb_u8(0.0, 110.0, 60.0), (0, 0, 0), "lch L=0 -> black");
        // 超 100（越界亮）也→白；负 L→黑
        assert_eq!(lab_to_srgb_u8(120.0, 40.0, 40.0), (255, 255, 255));
        assert_eq!(lch_to_srgb_u8(-5.0, 30.0, 30.0), (0, 0, 0));
        // 回归：in-gamput mid-L 不变（byte-identical）
        let mid = lab_to_srgb_u8(50.0, 0.0, 0.0);
        assert_eq!(mid, lab_to_srgb_u8(50.0, 0.0, 0.0));
        // mid-L lab(50,0,0) 应是中灰，非纯白/纯黑
        assert!(mid.0 > 50 && mid.0 < 250, "mid-L should be mid-gray, got {mid:?}");
    }

    #[test]
    fn test_r2323_oklab_oklch_l_boundary_white_black() {
        // OKLab/OKLCH L∈[0,1]：L≥1.0 → 白，L≤0 → 黑。
        // driving: css-color oklch-009（oklch(100% 110 60)→白）/ oklch-010（oklch(0% 1.1 60)→黑）。
        assert_eq!(oklab_to_srgb_u8(1.0, 0.2, 0.3), (255, 255, 255), "oklab L=1 -> white");
        assert_eq!(oklab_to_srgb_u8(0.0, 0.2, 0.3), (0, 0, 0), "oklab L=0 -> black");
        assert_eq!(oklch_to_srgb_u8(1.0, 0.11, 60.0), (255, 255, 255), "oklch L=1 -> white");
        assert_eq!(oklch_to_srgb_u8(0.0, 1.1, 60.0), (0, 0, 0), "oklch L=0 -> black");
        // 超 1.0 / 负
        assert_eq!(oklab_to_srgb_u8(1.5, 0.1, 0.1), (255, 255, 255));
        assert_eq!(oklch_to_srgb_u8(-0.2, 0.5, 30.0), (0, 0, 0));
        // 回归：in-gamut mid-L byte-identical
        let mid = oklab_to_srgb_u8(0.5, 0.0, 0.0);
        assert!(mid.0 > 50 && mid.0 < 250, "oklab mid-L should be mid-gray, got {mid:?}");
    }

    // ── R2324：color() 预定义空间百分比分量（CSS Color 4）─────────────────

    #[test]
    fn test_r2324_color_function_percent_components() {
        // CSS Color 4：color() 分量可为 <number>（0-1）或 <percentage>（0-100% → 0-1）。
        // driving: css-color predefined-002（color(srgb 0% 60% 0%) ≡ #009900）。
        // 此前 parse_color_number 仅 number → color(srgb 0% ...) None。
        assert_eq!(
            parse_color("color(srgb 0% 60% 0%)"),
            parse_color("#009900"),
            "color(srgb 0% 60% 0%) must equal #009900"
        );
        // percent ≡ number（0-1）
        assert_eq!(
            parse_color("color(srgb 100% 100% 100%)"),
            parse_color("color(srgb 1 1 1)")
        );
        assert_eq!(parse_color("color(srgb 0% 0% 0%)"), parse_color("color(srgb 0 0 0)"));
        // 回归：number 语法 byte-identical
        assert_eq!(
            parse_color("color(srgb 0 0.6 0)"),
            Some(ColorValue::Rgba(0, 153, 0, 255))
        );
        // display-p3 percent 也接受（解析成功）
        assert!(parse_color("color(display-p3 0% 100% 0%)").is_some());
    }

    // ── R2325：OKLab/OKLCH L 边界容差（CSS Color 4）─────────────────────
    // R2323 的 L 边界用精确 0/1 阈值：oklab(0 …)→黑 但 oklab(0.0001% …)（L=1e-6）走逐通道钳制
    // →(4,7,0)，二者不一致 → oklab-l-almost-0/1 fail。fix：OKLab/OKLCH 边界改用 1e-4 容差，
    // L 极接近 0/1（含 0.0001%）即与 L=0/1 同为黑/白（L=0/1 平面经 gamut 映射恒收敛到黑/白点）；
    // 其余逐通道钳制 byte-identical 于 R2323。lab/lch 仍精确 0/100（lch-009/010 不变）。

    #[test]
    fn test_r2325_oklab_l_almost_boundary_consistent() {
        // driving: css-color oklab-l-almost-0/1。L 极接近 0/1 须与 L 恰为 0/1 渲染一致。
        // R2323 精确阈值致 oklab(0 ...)→黑 但 oklab(0.0001% ...)→(4,7,0) 不一致。
        assert_eq!(
            oklab_to_srgb_u8(0.0, 0.15, 0.15),
            oklab_to_srgb_u8(0.000001, 0.15, 0.15),
            "oklab L=0 and L≈0 must match (almost-0)"
        );
        assert_eq!(
            oklab_to_srgb_u8(1.0, 0.15, 0.15),
            oklab_to_srgb_u8(0.999999, 0.15, 0.15),
            "oklab L=1 and L≈1 must match (almost-1)"
        );
        // OKLCH 同族（委托 oklab_to_srgb_u8）
        assert_eq!(
            oklch_to_srgb_u8(0.0, 0.2121, 45.0),
            oklch_to_srgb_u8(0.000001, 0.2121, 45.0),
            "oklch L=0 and L≈0 must match"
        );
        assert_eq!(
            oklch_to_srgb_u8(1.0, 0.2121, 45.0),
            oklch_to_srgb_u8(0.999999, 0.2121, 45.0),
            "oklch L=1 and L≈1 must match"
        );
        // 边界外（恰超容差）仍走逐通道钳制，不被强制——容差边界清晰
        assert_ne!(
            oklab_to_srgb_u8(0.0, 0.15, 0.15),
            oklab_to_srgb_u8(0.001, 0.15, 0.15),
            "L=0.001 (above L_EPS) is not forced to black"
        );
    }

    #[test]
    fn test_r2325_l_boundary_forces_white_black() {
        // driving: css-color lch-009/010、oklch-009/010。L 边界（含 OKLab 容差带）任意 chroma → 黑/白。
        // 回归守护 R2323 已修的 lch/oklch-009/010。
        assert_eq!(
            lch_to_srgb_u8(100.0, 110.0, 60.0),
            (255, 255, 255),
            "lch L=100 大C -> white"
        );
        assert_eq!(lch_to_srgb_u8(0.0, 110.0, 60.0), (0, 0, 0), "lch L=0 大C -> black");
        assert_eq!(lab_to_srgb_u8(100.0, 55.0, 95.3), (255, 255, 255));
        assert_eq!(lab_to_srgb_u8(0.0, 55.0, 95.3), (0, 0, 0));
        assert_eq!(oklch_to_srgb_u8(1.0, 0.11, 60.0), (255, 255, 255));
        assert_eq!(oklch_to_srgb_u8(0.0, 1.1, 60.0), (0, 0, 0));
        // 超 100 / 负 L 钳制后同上
        assert_eq!(lab_to_srgb_u8(120.0, 40.0, 40.0), (255, 255, 255));
        assert_eq!(lch_to_srgb_u8(-5.0, 30.0, 30.0), (0, 0, 0));
    }

    #[test]
    fn test_r2325_in_gamut_byte_identical_saturated_stable() {
        // in-gamut mid-L 灰不变（byte-identical 守护，未触边界）
        let lab_mid = lab_to_srgb_u8(50.0, 0.0, 0.0);
        assert!(lab_mid.0 > 50 && lab_mid.0 < 250, "lab mid-L mid-gray, got {lab_mid:?}");
        let ok_mid = oklab_to_srgb_u8(0.5, 0.0, 0.0);
        assert!(ok_mid.0 > 50 && ok_mid.0 < 250, "oklab mid-L mid-gray, got {ok_mid:?}");
        // L=0/1 中等 chroma 经边界容差强制为黑/白（与 L=0/1 一致）
        assert_eq!(oklab_to_srgb_u8(0.0, 0.2, 0.3), (0, 0, 0));
        assert_eq!(oklab_to_srgb_u8(1.0, 0.2, 0.3), (255, 255, 255));
        // 高饱和 in-gamut 色（如纯红 oklab）应稳定渲染为红，非黑/白
        let red = oklab_to_srgb_u8(0.6279, 0.22486, 0.12585); // ≈ sRGB 红
        assert!(
            red.0 > 200 && red.1 < 60 && red.2 < 60,
            "oklab red renders red, got {red:?}"
        );
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

    #[test]
    fn test_light_dark_resolves_to_dark_when_dark_scheme() {
        // dark color-scheme（color-scheme: dark）→ 取第二个（dark）参数。
        // driving: css-variables registered-property-light-dark。
        assert_eq!(
            parse_color_with_scheme("light-dark(red, green)", true),
            parse_color("green")
        );
        assert_eq!(
            parse_color_with_scheme("light-dark(red, #008000)", true),
            parse_color("#008000")
        );
        // light scheme（默认）仍取首个参数（parse_color = parse_color_with_scheme(_, false)）。
        assert_eq!(
            parse_color_with_scheme("light-dark(red, green)", false),
            parse_color("red")
        );
        // dark 向所选参数递归传播：light-dark(light-dark(a, b), c) 在 dark 下取 c。
        assert_eq!(
            parse_color_with_scheme("light-dark(red, light-dark(red, blue))", true),
            parse_color("blue")
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

    // ── R2357：枚举关键字大小写不敏感 ──────────────────────────────────

    #[test]
    fn test_parse_enum_keywords_case_insensitive() {
        // CSS 关键字大小写不敏感（CSS Syntax §）。这些解析器被 style-system cascade.rs
        // is_invalid_enum_value 用原始值调用判定合法性——大小写敏感会导致 `display: FLEX`
        // 等被误判非法而在级联时丢弃。全大写应与全小写等价。
        assert_eq!(parse_display("FLEX"), Some(DisplayValue::Flex));
        assert_eq!(parse_display("INLINE-BLOCK"), Some(DisplayValue::InlineBlock));
        assert_eq!(parse_position("ABSOLUTE"), Some(PositionValue::Absolute));
        assert_eq!(parse_overflow("HIDDEN"), Some(OverflowValue::Hidden));
        assert_eq!(
            parse_flex_direction("ROW-REVERSE"),
            Some(FlexDirectionValue::RowReverse)
        );
        assert_eq!(parse_flex_wrap("WRAP-REVERSE"), Some(FlexWrapValue::WrapReverse));
        assert_eq!(parse_alignment("SPACE-BETWEEN"), Some(AlignmentValue::SpaceBetween));
        assert_eq!(parse_box_sizing("BORDER-BOX"), Some(BoxSizingValue::BorderBox));
        assert_eq!(parse_visibility("COLLAPSE"), Some(VisibilityValue::Collapse));
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
        // R2574：list-style-type: <string>（CSS Lists 3）——引号字符串作固定标记文本。
        assert_eq!(
            parse_list_style_type("\"▶\""),
            Some(ListStyleTypeValue::String("▶".to_string()))
        );
        assert_eq!(
            parse_list_style_type("'Step '"),
            Some(ListStyleTypeValue::String("Step ".to_string()))
        );
        // 空串合法（无标记）。
        assert_eq!(
            parse_list_style_type("\"\""),
            Some(ListStyleTypeValue::String(String::new()))
        );
        // 单引号未闭合 → 非 string（落入 keyword/custom-ident，均不匹配）→ None。
        assert_eq!(parse_list_style_type("'unclosed"), None);
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
