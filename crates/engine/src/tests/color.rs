use zero_css_parser::values::ColorValue;
use zero_render_foundation::color::Color;

use crate::paint::{color_value_to_render, hsla_to_rgba, length_to_f32, named_color_to_render, simple_hash};
/// 测试 crimson 色值 Rgba(220,20,60,255) 正确转换为渲染 Color。
#[test]
fn test_named_color_crimson_render() {
    let crimson = ColorValue::Rgba(220, 20, 60, 255);
    let color = color_value_to_render(&crimson);
    assert_eq!(color.r, 220);
    assert_eq!(color.g, 20);
    assert_eq!(color.b, 60);
    assert_eq!(color.a, 255);
}
/// 测试 hsla_to_rgba(0, 100, 50, 1.0) 生成正确的纯红 RGBA 值。
#[test]
fn test_hsla_to_rgba_pure_red() {
    let color = hsla_to_rgba(0.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 255, "pure red R should be 255");
    assert_eq!(color.g, 0, "pure red G should be 0");
    assert_eq!(color.b, 0, "pure red B should be 0");
    assert_eq!(color.a, 255, "alpha=1.0 should map to 255");
}
/// CSS 解析器将命名颜色在解析时转换为 Rgba 值，
/// 验证通过 color_value_to_render 正确传播到渲染 Color。
#[test]
fn test_named_color_render_conversion() {
    // coral → Rgba(255, 127, 80, 255)
    let coral = ColorValue::Rgba(255, 127, 80, 255);
    let color = color_value_to_render(&coral);
    assert_eq!(color.r, 255, "coral R should be 255");
    assert_eq!(color.g, 127, "coral G should be 127");
    assert_eq!(color.b, 80, "coral B should be 80");
    assert_eq!(color.a, 255, "coral A should be 255");

    // tomato → Rgba(255, 99, 71, 255)
    let tomato = ColorValue::Rgba(255, 99, 71, 255);
    let color = color_value_to_render(&tomato);
    assert_eq!(color.r, 255, "tomato R should be 255");
    assert_eq!(color.g, 99, "tomato G should be 99");
    assert_eq!(color.b, 71, "tomato B should be 71");
    assert_eq!(color.a, 255, "tomato A should be 255");

    // steelblue → Rgba(70, 130, 180, 255)
    let steelblue = ColorValue::Rgba(70, 130, 180, 255);
    let color = color_value_to_render(&steelblue);
    assert_eq!(color.r, 70, "steelblue R should be 70");
    assert_eq!(color.g, 130, "steelblue G should be 130");
    assert_eq!(color.b, 180, "steelblue B should be 180");
    assert_eq!(color.a, 255, "steelblue A should be 255");
}
/// 验证亮度为 0 和 100 时 HSL 转换结果正确。
#[test]
fn test_hsla_to_rgba_black_and_white() {
    // 黑色：亮度 0%
    let black = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(black.r, 0, "HSL black R should be 0");
    assert_eq!(black.g, 0, "HSL black G should be 0");
    assert_eq!(black.b, 0, "HSL black B should be 0");
    assert_eq!(black.a, 255, "HSL black A should be 255");

    // 白色：亮度 100%
    let white = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
    assert_eq!(white.r, 255, "HSL white R should be 255");
    assert_eq!(white.g, 255, "HSL white G should be 255");
    assert_eq!(white.b, 255, "HSL white B should be 255");
    assert_eq!(white.a, 255, "HSL white A should be 255");
}
/// 验证 ColorValue::Transparent 的 alpha 通道为 0。
#[test]
fn test_color_value_transparent_conversion() {
    let color = color_value_to_render(&ColorValue::Transparent);
    assert_eq!(color.r, 0, "transparent R should be 0");
    assert_eq!(color.g, 0, "transparent G should be 0");
    assert_eq!(color.b, 0, "transparent B should be 0");
    assert_eq!(color.a, 0, "transparent A should be 0");
}
/// CurrentColor 在无上下文时应回退为默认的黑色（alpha=255），
/// 这与 Transparent（alpha=0）形成对比。
#[test]
fn test_color_value_current_color_render() {
    let color = color_value_to_render(&ColorValue::CurrentColor);
    assert_eq!(color.r, 0, "CurrentColor R should be 0");
    assert_eq!(color.g, 0, "CurrentColor G should be 0");
    assert_eq!(color.b, 0, "CurrentColor B should be 0");
    assert_eq!(color.a, 255, "CurrentColor A should be 255 (fully opaque)");
}

// ── 新增边界条件测试：named_color / simple_hash / length_to_f32 ──

/// 测试 named_color_to_render 对更多命名颜色返回正确 RGB 值。
///
/// 覆盖 yellow, cyan(aqua), magenta(fuchsia), gray(grey),
/// silver, maroon, olive, lime, purple, teal, navy, orange, pink, brown。
#[test]
fn test_named_color_extended_colors() {
    assert_eq!(named_color_to_render("yellow"), Color::rgb(255, 255, 0));
    assert_eq!(named_color_to_render("cyan"), Color::rgb(0, 255, 255));
    assert_eq!(named_color_to_render("aqua"), Color::rgb(0, 255, 255));
    assert_eq!(named_color_to_render("magenta"), Color::rgb(255, 0, 255));
    assert_eq!(named_color_to_render("fuchsia"), Color::rgb(255, 0, 255));
    assert_eq!(named_color_to_render("gray"), Color::rgb(128, 128, 128));
    assert_eq!(named_color_to_render("grey"), Color::rgb(128, 128, 128));
    assert_eq!(named_color_to_render("silver"), Color::rgb(192, 192, 192));
    assert_eq!(named_color_to_render("maroon"), Color::rgb(128, 0, 0));
    assert_eq!(named_color_to_render("olive"), Color::rgb(128, 128, 0));
    assert_eq!(named_color_to_render("lime"), Color::rgb(0, 255, 0));
    assert_eq!(named_color_to_render("purple"), Color::rgb(128, 0, 128));
    assert_eq!(named_color_to_render("teal"), Color::rgb(0, 128, 128));
    assert_eq!(named_color_to_render("navy"), Color::rgb(0, 0, 128));
    assert_eq!(named_color_to_render("orange"), Color::rgb(255, 165, 0));
    assert_eq!(named_color_to_render("pink"), Color::rgb(255, 192, 203));
    assert_eq!(named_color_to_render("brown"), Color::rgb(165, 42, 42));
}

/// 测试 named_color_to_render 对未知颜色名回退为黑色。
#[test]
fn test_named_color_unknown_fallback() {
    assert_eq!(named_color_to_render("chartreuse"), Color::rgb(0, 0, 0));
    assert_eq!(named_color_to_render(""), Color::rgb(0, 0, 0));
    assert_eq!(named_color_to_render("NotAColor"), Color::rgb(0, 0, 0));
}

/// 测试 named_color_to_render 大小写不敏感。
#[test]
fn test_named_color_case_insensitive() {
    assert_eq!(named_color_to_render("ORANGE"), Color::rgb(255, 165, 0));
    assert_eq!(named_color_to_render("Pink"), Color::rgb(255, 192, 203));
    assert_eq!(named_color_to_render("BROWN"), Color::rgb(165, 42, 42));
    assert_eq!(named_color_to_render("Silver"), Color::rgb(192, 192, 192));
}

/// 测试 simple_hash 空字符串返回初始种子值 5381。
#[test]
fn test_simple_hash_empty_string() {
    let hash = simple_hash("");
    assert_eq!(hash, 5381, "empty string hash should be initial seed 5381");
}

/// 测试 simple_hash 对相同输入返回相同结果（确定性）。
#[test]
fn test_simple_hash_deterministic() {
    let h1 = simple_hash("https://example.com/image.png");
    let h2 = simple_hash("https://example.com/image.png");
    assert_eq!(h1, h2, "same input should produce same hash");
}

/// 测试 simple_hash 对不同输入返回不同结果。
#[test]
fn test_simple_hash_different_inputs() {
    let h1 = simple_hash("url_a");
    let h2 = simple_hash("url_b");
    assert_ne!(h1, h2, "different inputs should produce different hashes");
}

/// 测试 simple_hash 对长字符串不溢出（wrapping_mul 正确处理）。
#[test]
fn test_simple_hash_long_string_no_overflow() {
    let long = "a".repeat(10000);
    let hash = simple_hash(&long);
    // 只需验证不 panic 且为有限值
    assert_ne!(hash, 0, "long string hash should be non-zero");
}

/// 测试 simple_hash 对包含 Unicode 字符的字符串正常工作。
#[test]
fn test_simple_hash_unicode_string() {
    let hash = simple_hash("画像_URL_日本語");
    assert_ne!(hash, 0, "unicode string hash should be non-zero");
}

/// 测试 length_to_f32 对 Px 返回正确的 f32 值。
#[test]
fn test_length_to_f32_px() {
    assert_eq!(length_to_f32(&zero_css_parser::values::LengthValue::Px(42.0)), 42.0);
    assert_eq!(length_to_f32(&zero_css_parser::values::LengthValue::Px(0.0)), 0.0);
    assert_eq!(length_to_f32(&zero_css_parser::values::LengthValue::Px(-10.0)), -10.0);
}

/// 测试 length_to_f32 对非 Px 单位（Em、Rem、Vh、Vw、Percentage）返回 0.0。
#[test]
fn test_length_to_f32_non_px_units_return_zero() {
    use zero_css_parser::values::LengthValue;
    assert_eq!(length_to_f32(&LengthValue::Em(16.0)), 0.0, "Em should return 0.0");
    assert_eq!(length_to_f32(&LengthValue::Rem(16.0)), 0.0, "Rem should return 0.0");
    assert_eq!(length_to_f32(&LengthValue::Vh(50.0)), 0.0, "Vh should return 0.0");
    assert_eq!(length_to_f32(&LengthValue::Vw(50.0)), 0.0, "Vw should return 0.0");
    assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0, "Percentage should return 0.0");
}

/// 测试 length_to_f32 对 Px(0.0) 精确返回 0.0。
#[test]
fn test_length_to_f32_px_zero_precise() {
    let val = length_to_f32(&zero_css_parser::values::LengthValue::Px(0.0));
    assert_eq!(val, 0.0);
    assert!(val.is_sign_positive(), "zero should be positive zero");
}

/// 测试 hsla_to_rgba 对 hue=60（黄色区域）产生正确结果。
#[test]
fn test_hsla_to_rgba_hue_60_yellow() {
    let color = hsla_to_rgba(60.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 255, "hue=60 R should be 255");
    assert_eq!(color.g, 255, "hue=60 G should be 255");
    assert_eq!(color.b, 0, "hue=60 B should be 0");
    assert_eq!(color.a, 255);
}

/// 测试 hsla_to_rgba 对 hue=300（品红区域）产生正确结果。
#[test]
fn test_hsla_to_rgba_hue_300_magenta() {
    let color = hsla_to_rgba(300.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 255, "hue=300 R should be 255");
    assert_eq!(color.g, 0, "hue=300 G should be 0");
    assert_eq!(color.b, 255, "hue=300 B should be 255");
    assert_eq!(color.a, 255);
}

/// 测试 hsla_to_rgba 对 alpha=0（完全透明）产生 alpha=0。
#[test]
fn test_hsla_to_rgba_zero_alpha() {
    let color = hsla_to_rgba(0.0, 100.0, 50.0, 0.0);
    assert_eq!(color.a, 0, "alpha=0.0 should map to 0");
    assert_eq!(color.r, 255, "R channel should still be computed");
}

/// 测试 hsla_to_rgba 对 alpha > 1.0 的情况 clamp 到 255。
#[test]
fn test_hsla_to_rgba_alpha_above_one_clamped() {
    let color = hsla_to_rgba(0.0, 100.0, 50.0, 2.0);
    assert_eq!(color.a, 255, "alpha > 1.0 should be clamped to 255");
}
