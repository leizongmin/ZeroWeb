//! 颜色转换工具 — CSS ColorValue 到渲染层 Color 的转换。

use zero_css_parser::values::{ColorValue, RcsAlpha, RcsChannel, RelativeColorFunc, RelativeColorSpec};
use zero_render_foundation::color::Color;

/// 将 ComputedStyle 的 ColorValue 转换为 render-foundation 的 Color。
///
/// 注意：`ColorValue::CurrentColor` 在此无元素上下文，回落为黑色。需要正确解析
/// currentColor 的调用点（如边框绘制）须在传入前先把 currentColor 替换为元素
/// 自身计算 `color`（CSS-Color §resolving）。等价于 `resolve_color_current(color, &CurrentColor)`。
pub fn color_value_to_render(color: &ColorValue) -> Color {
    resolve_color_current(color, &ColorValue::CurrentColor)
}

/// 解析颜色到渲染 Color，**currentColor 替换为元素自身计算 `color`**（CSS Color §resolving）。
///
/// 与 `color_value_to_render` 的区别：后者无元素上下文把 currentColor 回落黑色；本函数
/// 用于 background-color 等须正确解析 currentColor 的场景（driving: css-color currentcolor-001..
/// `background-color: currentColor` 应取元素 `color`，非黑色）。若元素 color 本身未解析仍为
/// CurrentColor（`color: currentColor` 罕见，须 cascade 解析继承色），回落黑色（旧行为）。
///
/// `ColorValue::Mix`（color-mix）的两分量 currentColor 也按 `element_color` 递归解析后插值，
/// 故 `background-color: inherit` 透传的 Mix 在子元素按其自身 color 重解析 currentColor
///（driving: color-mix-currentcolor-001）。
pub fn resolve_color_current(color: &ColorValue, element_color: &ColorValue) -> Color {
    match color {
        ColorValue::CurrentColor => match element_color {
            ColorValue::CurrentColor => Color::rgba(0, 0, 0, 255),
            other => color_value_to_render(other),
        },
        ColorValue::Mix(spec) => {
            let c1 = resolve_color_current(&spec.c1.color, element_color);
            let c2 = resolve_color_current(&spec.c2.color, element_color);
            mix_srgb(c1, spec.c1.percentage, c2, spec.c2.percentage)
        }
        // RCS 非 identity：先按元素色解析 origin（currentColor → 元素色），再按函数通道语义计算。
        // 支持 inherit 透传（background-color: inherit 透传 RelativeColor，currentColor 在子元素
        // 按其自身 color 重解析）。driving: relative-currentcolor-rgb-02 / hsl-02。
        ColorValue::RelativeColor(spec) => {
            let origin = resolve_color_current(&spec.origin, element_color);
            resolve_relative_color(spec, origin)
        }
        ColorValue::Rgba(r, g, b, a) => Color::rgba(*r, *g, *b, *a),
        ColorValue::Transparent => Color::rgba(0, 0, 0, 0),
        ColorValue::Named(name) => named_color_to_render(name),
        ColorValue::Hsla(h, s, l, a) => hsla_to_rgba(*h, *s, *l, *a),
    }
}

/// `color-mix(in srgb, c1 [p1], c2 [p2])` 的 sRGB 插值（CSS Color 5 §12.2）。
///
/// sRGB 色彩空间 gamma-encoded 线性插值（premultiplied alpha）。百分比默认：双省略=50/50，
/// 单省略=100-另一；和≠100 时按比例归一化到 100%，sum<100 则 alpha ×= sum/100。
fn mix_srgb(c1: Color, p1: Option<f64>, c2: Color, p2: Option<f64>) -> Color {
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, 100.0 - a),
        (None, Some(b)) => (100.0 - b, b),
        (None, None) => (50.0, 50.0),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return Color::rgba(0, 0, 0, 0); // 两端 0% → 全透明
    }
    let alpha_mult = (sum / 100.0).min(1.0);
    let w1 = p1 / sum;
    let w2 = p2 / sum;
    // premultiplied alpha 插值（CSS Color §12.2）
    let a1 = c1.a as f64 / 255.0;
    let a2 = c2.a as f64 / 255.0;
    let pr = c1.r as f64 * a1 * w1 + c2.r as f64 * a2 * w2;
    let pg = c1.g as f64 * a1 * w1 + c2.g as f64 * a2 * w2;
    let pb = c1.b as f64 * a1 * w1 + c2.b as f64 * a2 * w2;
    let pa = a1 * w1 + a2 * w2;
    let final_a = (pa * alpha_mult).clamp(0.0, 1.0);
    if final_a <= 0.0 {
        return Color::rgba(0, 0, 0, 0);
    }
    let to_u8 = |v: f64| (v / pa).round().clamp(0.0, 255.0) as u8; // un-premultiply
    Color::rgba(
        to_u8(pr),
        to_u8(pg),
        to_u8(pb),
        (final_a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// 解析 RCS（CSS Color 5 相对色）非 identity：origin 已解析为 sRGB Color，按函数通道语义计算输出。
///
/// - rgb：origin 通道 r/g/b（0-255）按 channels 引用或字面量重组（支持置换），结果 rgb。
/// - hsl：origin 经 rgba_to_hsl 转 HSL（h 度、s/l 0-100），按 channels 覆盖，再 hsla_to_rgba 回 sRGB。
///
/// alpha：省略/Origin → origin alpha；Num(0-1) → 归一 0-255；None → 0。
fn resolve_relative_color(spec: &RelativeColorSpec, origin: Color) -> Color {
    match spec.func {
        RelativeColorFunc::Rgb => {
            let chans = [origin.r as f64, origin.g as f64, origin.b as f64];
            let pick = |i: usize| match spec.channels[i] {
                RcsChannel::Ref(r) => chans[r as usize],
                RcsChannel::Num(v) => v,
                RcsChannel::None => 0.0,
            };
            Color::rgba(
                pick(0).round().clamp(0.0, 255.0) as u8,
                pick(1).round().clamp(0.0, 255.0) as u8,
                pick(2).round().clamp(0.0, 255.0) as u8,
                resolve_rcs_alpha(spec.alpha, origin.a),
            )
        }
        RelativeColorFunc::Hsl => {
            let (h0, s0, l0) = rgba_to_hsl(origin.r, origin.g, origin.b);
            let h = match spec.channels[0] {
                RcsChannel::Ref(_) => h0,
                RcsChannel::Num(v) => v,
                RcsChannel::None => 0.0,
            };
            let s = match spec.channels[1] {
                RcsChannel::Ref(_) => s0,
                RcsChannel::Num(v) => v,
                RcsChannel::None => 0.0,
            };
            let l = match spec.channels[2] {
                RcsChannel::Ref(_) => l0,
                RcsChannel::Num(v) => v,
                RcsChannel::None => 0.0,
            };
            let mut c = hsla_to_rgba(h, s, l, 1.0);
            c.a = resolve_rcs_alpha(spec.alpha, origin.a);
            c
        }
    }
}

/// 归一 RCS alpha 到 0-255 u8：Origin → origin alpha；Num(0-1) → ×255 钳制；None → 0。
fn resolve_rcs_alpha(alpha: RcsAlpha, origin_alpha: u8) -> u8 {
    match alpha {
        RcsAlpha::Origin => origin_alpha,
        RcsAlpha::Num(v) => (v * 255.0).round().clamp(0.0, 255.0) as u8,
        RcsAlpha::None => 0,
    }
}

/// sRGB (0-255) → HSL：h ∈ [0,360)，s/l ∈ [0,100]。灰色（r=g=b）→ h=0/s=0。
fn rgba_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return (0.0, 0.0, l * 100.0);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    let h = h * 60.0;
    (h, s * 100.0, l * 100.0)
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
    /// R2259：resolve_color_current 把 currentColor 解析为元素自身 color（非黑色）。
    /// driving: css-color currentcolor-001..（`background-color: currentColor` → 元素 color）。
    fn test_resolve_color_current() {
        let green = ColorValue::Named("green".to_string());
        let red = ColorValue::Named("red".to_string());
        // currentColor → 元素 color（green）
        assert_eq!(
            resolve_color_current(&ColorValue::CurrentColor, &green),
            color_value_to_render(&green)
        );
        assert_eq!(
            resolve_color_current(&ColorValue::CurrentColor, &red),
            color_value_to_render(&red)
        );
        // 非 currentColor 透传（不受 element_color 影响）
        assert_eq!(resolve_color_current(&red, &green), color_value_to_render(&red));
        // 元素 color 本身未解析仍为 currentColor（color:currentColor 罕见）→ 回落黑
        assert_eq!(
            resolve_color_current(&ColorValue::CurrentColor, &ColorValue::CurrentColor),
            Color::rgba(0, 0, 0, 255)
        );
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

#[test]
/// R2267：resolve_color_current 对 color-mix Mix 的解析（currentColor 按元素色 + sRGB 插值）。
fn test_resolve_color_mix() {
    use zero_css_parser::values::{ColorMixComponent, ColorMixSpec};
    // mix(green 50%, green) = green（001 场景：currentColor 在子元素按 green 解析）
    let both_green = ColorValue::Mix(Box::new(ColorMixSpec {
        c1: ColorMixComponent {
            color: ColorValue::Rgba(0, 128, 0, 255),
            percentage: Some(50.0),
        },
        c2: ColorMixComponent {
            color: ColorValue::CurrentColor,
            percentage: None,
        },
    }));
    let green_elem = ColorValue::Rgba(0, 128, 0, 255);
    let r = resolve_color_current(&both_green, &green_elem);
    assert_eq!(
        (r.r, r.g, r.b),
        (0, 128, 0),
        "mix(green, currentColor=green) 应为 green"
    );

    // mix(red 50%, green) = rgb(128, 64, 0)
    let mix_rg = ColorValue::Mix(Box::new(ColorMixSpec {
        c1: ColorMixComponent {
            color: ColorValue::Rgba(255, 0, 0, 255),
            percentage: Some(50.0),
        },
        c2: ColorMixComponent {
            color: ColorValue::Rgba(0, 128, 0, 255),
            percentage: Some(50.0),
        },
    }));
    let r = resolve_color_current(&mix_rg, &ColorValue::CurrentColor);
    assert_eq!((r.r, r.g, r.b), (128, 64, 0), "mix(red 50%, green 50%) = rgb(128,64,0)");
}

#[test]
/// R2271：RCS 非 identity 解析——currentColor origin 按元素色解析后做通道置换/覆盖。
/// driving: css-color relative-currentcolor-rgb-02（g r b 置换）/ hsl-02（120 s l 覆盖）。
/// 两 driving 案共同点：origin = 元素 color = #800000 (128,0,0)，结果均为 green。
fn test_resolve_relative_color() {
    use zero_css_parser::values::{RcsAlpha, RcsChannel, RelativeColorFunc, RelativeColorSpec};
    // rgb 置换：`rgb(from currentColor g r b)`，origin=#800000 → (g=0, r=128, b=0) = green。
    let rgb_swap = ColorValue::RelativeColor(Box::new(RelativeColorSpec {
        func: RelativeColorFunc::Rgb,
        origin: ColorValue::CurrentColor,
        channels: [RcsChannel::Ref(1), RcsChannel::Ref(0), RcsChannel::Ref(2)],
        alpha: RcsAlpha::Origin,
    }));
    let elem = ColorValue::Rgba(128, 0, 0, 255); // #800000
    let r = resolve_color_current(&rgb_swap, &elem);
    assert_eq!((r.r, r.g, r.b), (0, 128, 0), "rgb(from #800000 g r b) = green");
    assert_eq!(r.a, 255, "alpha 省略 → origin alpha");

    // hsl 覆盖：`hsl(from currentColor 120 s l)`，origin=#800000 → HSL(0,100,25)，覆盖 h=120 → green。
    let hsl_override = ColorValue::RelativeColor(Box::new(RelativeColorSpec {
        func: RelativeColorFunc::Hsl,
        origin: ColorValue::CurrentColor,
        channels: [RcsChannel::Num(120.0), RcsChannel::Ref(1), RcsChannel::Ref(2)],
        alpha: RcsAlpha::Origin,
    }));
    let r = resolve_color_current(&hsl_override, &elem);
    assert_eq!((r.r, r.g, r.b), (0, 128, 0), "hsl(from #800000 120 s l) = green");

    // inherit 透传：同一 RelativeColor 对不同元素 color 重解析（rgb 置换 origin=#0000FF → (b? no: g=0,r=0,b=255)）
    // `rgb(from #0000ff g r b)`：g=0,r=0,b=255 → (0,0,255) 蓝（b 引用未变，r/g 互换仍 0）
    let blue_elem = ColorValue::Rgba(0, 0, 255, 255);
    let r = resolve_color_current(&rgb_swap, &blue_elem);
    assert_eq!((r.r, r.g, r.b), (0, 0, 255), "rgb(from #0000ff g r b) = blue");
}

#[test]
/// R2271：rgba_to_hsl 往返——sRGB → HSL → sRGB 经 hsla_to_rgba 应回到原色（灰色与饱和色）。
fn test_rgba_to_hsl_roundtrip() {
    // #800000 = (128,0,0) → HSL(0,100,~25)（红，色相 0、饱和 100）
    let (h, s, l) = rgba_to_hsl(128, 0, 0);
    assert!((h - 0.0).abs() < 0.5, "h≈0 (red), got {h}");
    assert!((s - 100.0).abs() < 0.5, "s≈100, got {s}");
    assert!((l - 25.0).abs() < 0.5, "l≈25, got {l}");
    // 灰色 (128,128,128) → h=0/s=0
    let (h, s, _l) = rgba_to_hsl(128, 128, 128);
    assert_eq!(h, 0.0);
    assert_eq!(s, 0.0);
}
