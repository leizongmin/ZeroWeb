//! 颜色转换工具 — CSS ColorValue 到渲染层 Color 的转换。

use zero_css_parser::values::ColorValue;
use zero_render_foundation::color::Color;

/// 将 ComputedStyle 的 ColorValue 转换为 render-foundation 的 Color。
///
/// 注意：`ColorValue::CurrentColor` 在此无元素上下文，回落为黑色。需要正确解析
/// currentColor 的调用点（如边框绘制）须在传入前先把 currentColor 替换为元素
/// 自身计算 `color`（CSS-Color §resolving）。
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
    // R2253：HSL 色相当作**角度**（CSS Color §6.1），归一化到 [0,360)。负值与 >360 值取模。
    // Rust `%` 是取余（对负被除数保留符号），故 `-300 % 360 = -300` → 错误扇区；
    // `((h % 360) + 360) % 360` 把 -300→60、-360→0、450→90。driving: css-color
    // t424-hsl-h-rotating-b / t425-hsla-h-rotating-b（H 值「even when outside [0,360)」）。
    let h = ((h % 360.0) + 360.0) % 360.0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::ColorValue;

    // ── color_value_to_render ───────────────────────────────────────────

    #[test]
    fn test_rgba_color() {
        let c = color_value_to_render(&ColorValue::Rgba(100, 200, 50, 255));
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 200);
        assert_eq!(c.b, 50);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_transparent_color() {
        let c = color_value_to_render(&ColorValue::Transparent);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 0);
    }

    #[test]
    fn test_current_color() {
        let c = color_value_to_render(&ColorValue::CurrentColor);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_hsla_color() {
        // hsla(0, 100, 50, 1.0) = red
        let c = color_value_to_render(&ColorValue::Hsla(0.0, 100.0, 50.0, 1.0));
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    /// R2253：HSL 色相为角度（mod 360），负值/越界值须归一化。driving: css-color
    /// t424-hsl-h-rotating-b / t425-hsla-h-rotating-b（H 值「even when outside [0,360)」）。
    fn test_hsla_hue_angle_normalization() {
        // 基准色（归一化后应等于这些）
        let red = hsla_to_rgba(0.0, 100.0, 50.0, 1.0); // hsl(0)=红
        let yellow = hsla_to_rgba(60.0, 100.0, 50.0, 1.0); // hsl(60)=黄
        let green = hsla_to_rgba(120.0, 100.0, 50.0, 1.0); // hsl(120)=绿
        let cyan = hsla_to_rgba(180.0, 100.0, 50.0, 1.0); // hsl(180)=青

        // 负值：-360≡0、-300≡60、-240≡120、-180≡180
        assert_eq!(hsla_to_rgba(-360.0, 100.0, 50.0, 1.0), red, "hsl(-360)≡hsl(0)");
        assert_eq!(hsla_to_rgba(-300.0, 100.0, 50.0, 1.0), yellow, "hsl(-300)≡hsl(60)");
        assert_eq!(hsla_to_rgba(-240.0, 100.0, 50.0, 1.0), green, "hsl(-240)≡hsl(120)");
        assert_eq!(hsla_to_rgba(-180.0, 100.0, 50.0, 1.0), cyan, "hsl(-180)≡hsl(180)");

        // 越界正值：360≡0、420≡60、480≡120
        assert_eq!(hsla_to_rgba(360.0, 100.0, 50.0, 1.0), red, "hsl(360)≡hsl(0)");
        assert_eq!(hsla_to_rgba(420.0, 100.0, 50.0, 1.0), yellow, "hsl(420)≡hsl(60)");
        assert_eq!(hsla_to_rgba(480.0, 100.0, 50.0, 1.0), green, "hsl(480)≡hsl(120)");

        // 大幅越界：720≡0
        assert_eq!(hsla_to_rgba(720.0, 100.0, 50.0, 1.0), red, "hsl(720)≡hsl(0)");
    }

    #[test]
    fn test_named_color_red() {
        let c = color_value_to_render(&ColorValue::Named("red".to_string()));
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    // ── hsla_to_rgba ────────────────────────────────────────────────────

    #[test]
    fn test_hsla_red() {
        let c = hsla_to_rgba(0.0, 100.0, 50.0, 1.0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_hsla_green() {
        let c = hsla_to_rgba(120.0, 100.0, 50.0, 1.0);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hsla_blue() {
        let c = hsla_to_rgba(240.0, 100.0, 50.0, 1.0);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_hsla_zero_saturation_is_gray() {
        let c = hsla_to_rgba(0.0, 0.0, 50.0, 1.0);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 128);
    }

    #[test]
    fn test_hsla_alpha() {
        let c = hsla_to_rgba(0.0, 100.0, 50.0, 0.5);
        assert_eq!(c.a, 128); // 0.5 * 255 ≈ 128
    }

    #[test]
    fn test_hsla_black() {
        let c = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hsla_white() {
        let c = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
    }

    // ── named_color_to_render ───────────────────────────────────────────

    #[test]
    fn test_named_black() {
        let c = named_color_to_render("black");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_named_white() {
        let c = named_color_to_render("white");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_named_case_insensitive() {
        let c = named_color_to_render("RED");
        assert_eq!(c.r, 255);
    }

    #[test]
    fn test_named_cyan_aqua_alias() {
        let c1 = named_color_to_render("cyan");
        let c2 = named_color_to_render("aqua");
        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);
    }

    #[test]
    fn test_named_unknown_returns_black() {
        let c = named_color_to_render("nonexistentcolor");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_named_orange() {
        let c = named_color_to_render("orange");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 165);
        assert_eq!(c.b, 0);
    }
}
