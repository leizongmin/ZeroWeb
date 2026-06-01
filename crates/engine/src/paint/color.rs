//! 颜色转换工具 — CSS ColorValue 到渲染层 Color 的转换。

use zero_css_parser::values::ColorValue;
use zero_render_foundation::color::Color;

/// 将 ComputedStyle 的 ColorValue 转换为 render-foundation 的 Color。
pub fn color_value_to_render(color: &ColorValue) -> Color {
    match color {
        ColorValue::Rgba(r, g, b, a) => Color::rgba(*r, *g, *b, *a),
        ColorValue::Transparent => Color::rgba(0, 0, 0, 0),
        ColorValue::Named(name) => named_color_to_render(name),
        ColorValue::CurrentColor => Color::rgba(0, 0, 0, 255),
        ColorValue::Hsla(h, s, l, a) => hsla_to_rgba(*h, *s, *l, *a),
    }
}

/// 将 HSL(Hue, Saturation, Lightness, Alpha) 转换为 RGBA。
///
/// - `h` 色相角度 [0, 360)
/// - `s` 饱和度 [0, 100]
/// - `l` 亮度 [0, 100]
/// - `a` 不透明度 [0, 1]
pub fn hsla_to_rgba(h: f64, s: f64, l: f64, a: f64) -> Color {
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h_prime {
        hp if hp < 1.0 => (c, x, 0.0),
        hp if hp < 2.0 => (x, c, 0.0),
        hp if hp < 3.0 => (0.0, c, x),
        hp if hp < 4.0 => (0.0, x, c),
        hp if hp < 5.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let to_u8 = |v: f64| -> u8 { (v * 255.0).round().clamp(0.0, 255.0) as u8 };
    Color::rgba(
        to_u8(r1 + m),
        to_u8(g1 + m),
        to_u8(b1 + m),
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// 将命名颜色转换为渲染颜色。
pub fn named_color_to_render(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "red" => Color::rgb(255, 0, 0),
        "green" => Color::rgb(0, 128, 0),
        "blue" => Color::rgb(0, 0, 255),
        "black" => Color::rgb(0, 0, 0),
        "white" => Color::rgb(255, 255, 255),
        "yellow" => Color::rgb(255, 255, 0),
        "cyan" | "aqua" => Color::rgb(0, 255, 255),
        "magenta" | "fuchsia" => Color::rgb(255, 0, 255),
        "gray" | "grey" => Color::rgb(128, 128, 128),
        "silver" => Color::rgb(192, 192, 192),
        "maroon" => Color::rgb(128, 0, 0),
        "olive" => Color::rgb(128, 128, 0),
        "lime" => Color::rgb(0, 255, 0),
        "purple" => Color::rgb(128, 0, 128),
        "teal" => Color::rgb(0, 128, 128),
        "navy" => Color::rgb(0, 0, 128),
        "orange" => Color::rgb(255, 165, 0),
        "pink" => Color::rgb(255, 192, 203),
        "brown" => Color::rgb(165, 42, 42),
        _ => Color::rgb(0, 0, 0),
    }
}
