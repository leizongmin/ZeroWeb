//! CSS 计算值生成。
//!
//! 将级联值转换为计算值，处理相对单位到绝对单位的转换。

use std::collections::HashMap;

use zero_css_parser::values::{parse_var, LengthValue};

use crate::property::ComputedStyle;

/// 默认根字体大小（px）。
const ROOT_FONT_SIZE: f64 = 16.0;

/// 将相对长度转换为绝对像素值。
///
/// 支持 em、rem、vh、vw 等相对单位的转换。
pub fn resolve_length(
    length: &LengthValue,
    font_size: f64,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
) -> f64 {
    match length {
        LengthValue::Px(v) => *v,
        LengthValue::Em(v) => v * font_size,
        LengthValue::Rem(v) => v * ROOT_FONT_SIZE,
        LengthValue::Vh(v) => {
            let vh = viewport_height.unwrap_or(900.0);
            v * vh / 100.0
        }
        LengthValue::Vw(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            v * vw / 100.0
        }
        LengthValue::Vmin(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            let vh = viewport_height.unwrap_or(900.0);
            v * vw.min(vh) / 100.0
        }
        LengthValue::Vmax(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            let vh = viewport_height.unwrap_or(900.0);
            v * vw.max(vh) / 100.0
        }
        LengthValue::Ch(v) => {
            // 近似：1ch ≈ 0.5em
            v * font_size * 0.5
        }
    }
}

/// 解析 var() 引用并解析自定义属性。
///
/// 如果值包含 var()，从 custom_properties 中查找并替换。
/// 支持嵌套的 var() 调用和回退值。
pub fn resolve_var(value: &str, custom_properties: &HashMap<String, String>) -> String {
    // 检查整个值是否就是一个 var() 调用
    if let Some(var_ref) = parse_var(value) {
        if let Some(resolved) = custom_properties.get(&var_ref.name) {
            return resolved.clone();
        }
        if let Some(fallback) = &var_ref.fallback {
            return resolve_var(fallback, custom_properties);
        }
        // 无法解析，返回原值
        return value.to_string();
    }

    // 处理值中嵌入的 var() 调用
    let mut result = value.to_string();
    let mut max_iterations = 10; // 防止无限递归

    while max_iterations > 0 {
        max_iterations -= 1;

        // 查找 var( 的位置
        let Some(start) = result.find("var(") else {
            break;
        };

        // 找到匹配的右括号
        let Some(end) = find_matching_paren(&result, start + 4) else {
            break;
        };

        let inner = &result[start + 4..end];
        let replacement = resolve_var_inner(inner, custom_properties);

        result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
    }

    result
}

/// 在字符串中查找匹配的右括号。
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if depth == 0 {
        Some(i - 1)
    } else {
        None
    }
}

/// 解析 var() 内部内容。
fn resolve_var_inner(inner: &str, custom_properties: &HashMap<String, String>) -> String {
    // 查找顶层逗号（可能有嵌套 var()）
    let comma_pos = find_top_level_comma(inner);
    if let Some(pos) = comma_pos {
        let name = inner[..pos].trim();
        let fallback = inner[pos + 1..].trim();
        if let Some(resolved) = custom_properties.get(name) {
            return resolved.clone();
        }
        return resolve_var(fallback, custom_properties);
    }

    // 没有回退值
    let name = inner.trim();
    if let Some(resolved) = custom_properties.get(name) {
        return resolved.clone();
    }

    // 无法解析
    String::new()
}

/// 查找字符串中顶层的逗号（不在括号内的）。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// 将计算样式中的相对长度解析为绝对值。
///
/// 返回一个新的 ComputedStyle，其中所有相对长度都被转换为 px。
pub fn resolve_computed_style(
    style: &ComputedStyle,
    _custom_properties: &HashMap<String, String>,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
) -> ComputedStyle {
    let font_size_px = resolve_length(
        &style.font_size,
        ROOT_FONT_SIZE, // 根元素的 font-size 使用 root font-size
        viewport_width,
        viewport_height,
    );

    let mut resolved = style.clone();

    // 解析所有长度属性
    resolve_length_field(&mut resolved.width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.height, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.min_width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.min_height, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.max_width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.max_height, font_size_px, viewport_width, viewport_height);

    resolve_length_field(&mut resolved.margin_top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(
        &mut resolved.margin_right,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.margin_bottom,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(&mut resolved.margin_left, font_size_px, viewport_width, viewport_height);

    resolve_length_field(&mut resolved.padding_top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(
        &mut resolved.padding_right,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.padding_bottom,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.padding_left,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolve_length_field(
        &mut resolved.border_top_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_right_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_left_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolve_length_field(
        &mut resolved.border_top_left_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_top_right_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_right_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_left_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolve_length_field(&mut resolved.top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.right, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.bottom, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.left, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.gap, font_size_px, viewport_width, viewport_height);
    resolve_length_field(
        &mut resolved.letter_spacing,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.word_spacing,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolved
}

/// 将单个长度字段解析为绝对 px。
fn resolve_length_field(
    field: &mut LengthValue,
    font_size: f64,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
) {
    if !matches!(field, LengthValue::Px(_)) {
        let px = resolve_length(field, font_size, viewport_width, viewport_height);
        *field = LengthValue::Px(px);
    }
}

/// 解析可能包含 var() 的属性值。
///
/// 先解析 var() 引用，再尝试解析为长度值。
pub fn compute_value(
    value: &str,
    custom_properties: &HashMap<String, String>,
    _font_size: f64,
) -> Option<String> {
    let resolved = resolve_var(value, custom_properties);
    if resolved != value {
        Some(resolved)
    } else {
        None
    }
}

/// 从自定义属性中收集所有 -- 开头的属性。
pub fn collect_custom_properties(
    cascaded: &HashMap<String, String>,
) -> HashMap<String, String> {
    cascaded
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::ComputedStyle;
    use zero_css_parser::values::LengthValue;

    #[test]
    fn test_resolve_px() {
        let px = LengthValue::Px(10.0);
        assert_eq!(resolve_length(&px, 16.0, None, None), 10.0);
    }

    #[test]
    fn test_resolve_em() {
        let em = LengthValue::Em(2.0);
        assert_eq!(resolve_length(&em, 16.0, None, None), 32.0);
    }

    #[test]
    fn test_resolve_rem() {
        let rem = LengthValue::Rem(1.5);
        assert_eq!(resolve_length(&rem, 20.0, None, None), 24.0); // rem 用 root font size
    }

    #[test]
    fn test_resolve_vh() {
        let vh = LengthValue::Vh(50.0);
        let result = resolve_length(&vh, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 450.0);
    }

    #[test]
    fn test_resolve_vw() {
        let vw = LengthValue::Vw(25.0);
        let result = resolve_length(&vw, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 360.0);
    }

    #[test]
    fn test_resolve_vmin() {
        let vmin = LengthValue::Vmin(10.0);
        let result = resolve_length(&vmin, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 90.0); // min(1440, 900) * 0.1
    }

    #[test]
    fn test_resolve_vmax() {
        let vmax = LengthValue::Vmax(10.0);
        let result = resolve_length(&vmax, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 144.0); // max(1440, 900) * 0.1
    }

    #[test]
    fn test_resolve_ch() {
        let ch = LengthValue::Ch(2.0);
        let result = resolve_length(&ch, 16.0, None, None);
        assert_eq!(result, 16.0); // 2 * 16 * 0.5
    }

    #[test]
    fn test_resolve_var_simple() {
        let mut custom = HashMap::new();
        custom.insert("--main-color".to_string(), "red".to_string());

        let result = resolve_var("var(--main-color)", &custom);
        assert_eq!(result, "red");
    }

    #[test]
    fn test_resolve_var_with_fallback() {
        let custom = HashMap::new();
        let result = resolve_var("var(--undefined, blue)", &custom);
        assert_eq!(result, "blue");
    }

    #[test]
    fn test_resolve_var_undefined_no_fallback() {
        let custom = HashMap::new();
        let result = resolve_var("var(--undefined)", &custom);
        assert_eq!(result, "var(--undefined)"); // 返回原值
    }

    #[test]
    fn test_resolve_var_embedded() {
        let mut custom = HashMap::new();
        custom.insert("--size".to_string(), "10px".to_string());

        let result = resolve_var("margin: var(--size);", &custom);
        assert_eq!(result, "margin: 10px;");
    }

    #[test]
    fn test_resolve_computed_style() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.margin_top = LengthValue::Em(2.0);
        style.padding_left = LengthValue::Rem(1.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None);

        assert_eq!(resolved.margin_top, LengthValue::Px(40.0)); // 2em * 20px
        assert_eq!(resolved.padding_left, LengthValue::Px(16.0)); // 1rem * 16px
    }

    #[test]
    fn test_compute_value_with_var() {
        let mut custom = HashMap::new();
        custom.insert("--width".to_string(), "100px".to_string());

        let result = compute_value("var(--width)", &custom, 16.0);
        assert_eq!(result, Some("100px".to_string()));
    }

    #[test]
    fn test_compute_value_no_var() {
        let custom = HashMap::new();
        let result = compute_value("100px", &custom, 16.0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_collect_custom_properties() {
        let mut cascaded = HashMap::new();
        cascaded.insert("--color".to_string(), "red".to_string());
        cascaded.insert("--size".to_string(), "10px".to_string());
        cascaded.insert("display".to_string(), "flex".to_string());

        let custom = collect_custom_properties(&cascaded);
        assert_eq!(custom.len(), 2);
        assert_eq!(custom.get("--color"), Some(&"red".to_string()));
        assert_eq!(custom.get("--size"), Some(&"10px".to_string()));
    }

    #[test]
    fn test_resolve_var_chained() {
        let mut custom = HashMap::new();
        custom.insert("--a".to_string(), "var(--b)".to_string());
        custom.insert("--b".to_string(), "resolved".to_string());

        let result = resolve_var("var(--a)", &custom);
        assert_eq!(result, "var(--b)"); // 只展开一层
    }
}
