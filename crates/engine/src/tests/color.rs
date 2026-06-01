use zero_css_parser::values::ColorValue;

use crate::paint::{color_value_to_render, hsla_to_rgba};
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
