//! 颜色转换工具 — CSS ColorValue 到渲染层 Color 的转换。

use zero_css_parser::values::{
    ColorHueMethod, ColorMixSpace, ColorValue, RcsAlpha, RcsChannel, RelativeColorFunc, RelativeColorSpec,
    convert_predefined_to_srgb, lab_to_srgb_u8, lch_to_srgb_u8, oklab_to_srgb_u8, oklch_to_srgb_u8,
    srgb_linear_to_srgb_u8, srgb_u8_to_lab, srgb_u8_to_lch, srgb_u8_to_oklab, srgb_u8_to_oklch, srgb_u8_to_predefined,
    srgb_u8_to_srgb_linear, srgb_u8_to_xyz, xyz_to_srgb_u8,
};
use zero_render_foundation::color::Color;
use zero_render_foundation::color_space::interp_hue;
use zero_render_foundation::primitive::HueMethod;

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
            match spec.space {
                ColorMixSpace::Srgb => mix_srgb(c1, spec.c1.percentage, c2, spec.c2.percentage),
                ColorMixSpace::SrgbLinear => mix_cartesian(
                    c1,
                    spec.c1.percentage,
                    c2,
                    spec.c2.percentage,
                    srgb_u8_to_srgb_linear,
                    srgb_linear_to_srgb_u8,
                ),
                ColorMixSpace::Lch => mix_lch(c1, spec.c1.percentage, c2, spec.c2.percentage, spec.hue),
                ColorMixSpace::Lab => mix_cartesian(
                    c1,
                    spec.c1.percentage,
                    c2,
                    spec.c2.percentage,
                    srgb_u8_to_lab,
                    lab_to_srgb_u8,
                ),
                ColorMixSpace::OkLab => mix_cartesian(
                    c1,
                    spec.c1.percentage,
                    c2,
                    spec.c2.percentage,
                    srgb_u8_to_oklab,
                    oklab_to_srgb_u8,
                ),
                ColorMixSpace::OkLch => mix_oklch(c1, spec.c1.percentage, c2, spec.c2.percentage, spec.hue),
                ColorMixSpace::Xyz => mix_cartesian(
                    c1,
                    spec.c1.percentage,
                    c2,
                    spec.c2.percentage,
                    srgb_u8_to_xyz,
                    xyz_to_srgb_u8,
                ),
            }
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

/// `color-mix(in lch [<method> hue], c1 [p1], c2 [p2])` 的 LCH 极坐标插值（CSS Color 4）。
///
/// 两色转 CIE LCH-D50，L/C 线性插值、h 色相按 method 插值（默认短弧），再回转 sRGB。
/// 百分比归一化同 srgb；alpha 独立线性插值。driving: css-color color-mix-percents-01/02。
fn mix_lch(c1: Color, p1: Option<f64>, c2: Color, p2: Option<f64>, hue: ColorHueMethod) -> Color {
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, 100.0 - a),
        (None, Some(b)) => (100.0 - b, b),
        (None, None) => (50.0, 50.0),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return Color::rgba(0, 0, 0, 0);
    }
    let alpha_mult = (sum / 100.0).min(1.0);
    let w2 = p2 / sum; // w1 = 1 - w2
    let (l1, ch1, h1) = srgb_u8_to_lch(c1.r, c1.g, c1.b);
    let (l2, ch2, h2) = srgb_u8_to_lch(c2.r, c2.g, c2.b);
    let l = l1 + (l2 - l1) * w2;
    let c = ch1 + (ch2 - ch1) * w2;
    // 色相按 method 插值（复用 gradient 的 interp_hue，R2381）。
    let h = interp_hue(h1, h2, w2, map_hue_method(hue));
    let (r, g, b) = lch_to_srgb_u8(l, c, h);
    // alpha 独立线性插值（premultiplied 权重同 srgb；不透明时 alpha_mult=1）。
    let a1 = c1.a as f64 / 255.0;
    let a2 = c2.a as f64 / 255.0;
    let pa = a1 * (1.0 - w2) + a2 * w2;
    let final_a = (pa * alpha_mult).clamp(0.0, 1.0);
    Color::rgba(r, g, b, (final_a * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// 笛卡尔色彩空间（`in lab` / `in oklab`）color-mix 插值（R2376，CSS Color 4 §12）。
///
/// L/a/b 三通道独立线性插值，回转 sRGB；百分比归一化 + alpha 独立线性插值（同 mix_lch
/// 极坐标版）。`to_space`/`from_space` 注入转换闭包，lab/oklab 共用本函数体。
fn mix_cartesian(
    c1: Color,
    p1: Option<f64>,
    c2: Color,
    p2: Option<f64>,
    to_space: impl Fn(u8, u8, u8) -> (f64, f64, f64),
    from_space: impl Fn(f64, f64, f64) -> (u8, u8, u8),
) -> Color {
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, 100.0 - a),
        (None, Some(b)) => (100.0 - b, b),
        (None, None) => (50.0, 50.0),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return Color::rgba(0, 0, 0, 0);
    }
    let alpha_mult = (sum / 100.0).min(1.0);
    let w2 = p2 / sum; // w1 = 1 - w2
    let (l1, a1, b1) = to_space(c1.r, c1.g, c1.b);
    let (l2, a2, b2) = to_space(c2.r, c2.g, c2.b);
    let l = l1 + (l2 - l1) * w2;
    let a = a1 + (a2 - a1) * w2;
    let b = b1 + (b2 - b1) * w2;
    let (r, g, b) = from_space(l, a, b);
    // alpha 独立线性插值（premultiplied 权重同 srgb；不透明时 alpha_mult=1）。
    let a1 = c1.a as f64 / 255.0;
    let a2 = c2.a as f64 / 255.0;
    let pa = a1 * (1.0 - w2) + a2 * w2;
    let final_a = (pa * alpha_mult).clamp(0.0, 1.0);
    Color::rgba(r, g, b, (final_a * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// `color-mix(in oklch [<method> hue], c1 [p1], c2 [p2])` 的 OKLCH 极坐标插值（CSS Color 4 §12）。
///
/// 与 `mix_lch` 同构（L/C 线性、h 色相按 method 插值），仅换用 OKLab 系转换。
fn mix_oklch(c1: Color, p1: Option<f64>, c2: Color, p2: Option<f64>, hue: ColorHueMethod) -> Color {
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, 100.0 - a),
        (None, Some(b)) => (100.0 - b, b),
        (None, None) => (50.0, 50.0),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return Color::rgba(0, 0, 0, 0);
    }
    let alpha_mult = (sum / 100.0).min(1.0);
    let w2 = p2 / sum; // w1 = 1 - w2
    let (l1, ch1, h1) = srgb_u8_to_oklch(c1.r, c1.g, c1.b);
    let (l2, ch2, h2) = srgb_u8_to_oklch(c2.r, c2.g, c2.b);
    let l = l1 + (l2 - l1) * w2;
    let c = ch1 + (ch2 - ch1) * w2;
    // 色相按 method 插值（复用 gradient 的 interp_hue，R2381）。
    let h = interp_hue(h1, h2, w2, map_hue_method(hue));
    let (r, g, b) = oklch_to_srgb_u8(l, c, h);
    let a1 = c1.a as f64 / 255.0;
    let a2 = c2.a as f64 / 255.0;
    let pa = a1 * (1.0 - w2) + a2 * w2;
    let final_a = (pa * alpha_mult).clamp(0.0, 1.0);
    Color::rgba(r, g, b, (final_a * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// CSS parser `ColorHueMethod` → render-foundation `HueMethod`（color-mix 复用 gradient 的
/// `interp_hue` 数学）。与 helpers.rs gradient 映射一致。R2381。
fn map_hue_method(h: ColorHueMethod) -> HueMethod {
    match h {
        ColorHueMethod::Shorter => HueMethod::Shorter,
        ColorHueMethod::Longer => HueMethod::Longer,
        ColorHueMethod::Increasing => HueMethod::Increasing,
        ColorHueMethod::Decreasing => HueMethod::Decreasing,
    }
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
        // wide-gamut（lab/lch/oklab/oklch）：origin sRGB → 目标空间分量 → 通道覆盖 → 回 sRGB。
        RelativeColorFunc::Lab => {
            let c = rcs_pick(&spec.channels, srgb_u8_to_lab(origin.r, origin.g, origin.b));
            let (r, g, b) = lab_to_srgb_u8(c.0, c.1, c.2);
            Color::rgba(r, g, b, resolve_rcs_alpha(spec.alpha, origin.a))
        }
        RelativeColorFunc::Lch => {
            let c = rcs_pick(&spec.channels, srgb_u8_to_lch(origin.r, origin.g, origin.b));
            let (r, g, b) = lch_to_srgb_u8(c.0, c.1, c.2);
            Color::rgba(r, g, b, resolve_rcs_alpha(spec.alpha, origin.a))
        }
        RelativeColorFunc::Oklab => {
            let c = rcs_pick(&spec.channels, srgb_u8_to_oklab(origin.r, origin.g, origin.b));
            let (r, g, b) = oklab_to_srgb_u8(c.0, c.1, c.2);
            Color::rgba(r, g, b, resolve_rcs_alpha(spec.alpha, origin.a))
        }
        RelativeColorFunc::Oklch => {
            let c = rcs_pick(&spec.channels, srgb_u8_to_oklch(origin.r, origin.g, origin.b));
            let (r, g, b) = oklch_to_srgb_u8(c.0, c.1, c.2);
            Color::rgba(r, g, b, resolve_rcs_alpha(spec.alpha, origin.a))
        }
        // color(from <origin> <space> <channels>)：origin sRGB → 预定义空间分量 → 通道覆盖 → 回 sRGB。
        RelativeColorFunc::Color => {
            let space = spec.space.as_deref().unwrap_or("srgb");
            match srgb_u8_to_predefined(space, origin.r, origin.g, origin.b) {
                Some(comps) => {
                    let c = rcs_pick(&spec.channels, comps);
                    let (r, g, b) = convert_predefined_to_srgb(space, c.0, c.1, c.2).unwrap_or((0, 0, 0));
                    Color::rgba(r, g, b, resolve_rcs_alpha(spec.alpha, origin.a))
                }
                None => Color::rgba(0, 0, 0, resolve_rcs_alpha(spec.alpha, origin.a)),
            }
        }
    }
}

/// 按 RCS channels 从 origin 目标空间分量 (c0,c1,c2) 选取输出分量：
/// Ref → origin 对应通道（支持置换）、Num → 字面量覆盖、None → 0。
fn rcs_pick(channels: &[RcsChannel; 3], comps: (f64, f64, f64)) -> (f64, f64, f64) {
    let arr = [comps.0, comps.1, comps.2];
    let pick = |i: usize| match channels[i] {
        RcsChannel::Ref(r) => arr[r as usize],
        RcsChannel::Num(v) => v,
        RcsChannel::None => 0.0,
    };
    (pick(0), pick(1), pick(2))
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
    // R34xx：饱和度 clamp 到 [0, 100]（负饱和度 → 0——2d.fillStyle.parse
    // hsl-clamp-negative-saturation 期望 hsl(120,-200%,49.9%) = 灰）。
    let s = s.clamp(0.0, 100.0) / 100.0;
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
    use zero_css_parser::values::{ColorHueMethod, ColorMixComponent, ColorMixSpace, ColorMixSpec};
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
        space: ColorMixSpace::Srgb,
        hue: ColorHueMethod::Shorter,
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
        space: ColorMixSpace::Srgb,
        hue: ColorHueMethod::Shorter,
    }));
    let r = resolve_color_current(&mix_rg, &ColorValue::CurrentColor);
    assert_eq!((r.r, r.g, r.b), (128, 64, 0), "mix(red 50%, green 50%) = rgb(128,64,0)");
}

#[test]
/// R2376/R2377：color-mix srgb-linear/lab/oklab/oklch 空间解析（engine 侧）。此前这些空间
/// parse 返回 None → 颜色回退；现 resolve_color_current 产出真实插值色（介于两端、非回退黑、
/// 同色=identity）。
fn test_resolve_color_mix_lab_oklab_oklch() {
    use zero_css_parser::values::{ColorHueMethod, ColorMixComponent, ColorMixSpace, ColorMixSpec};
    let black_elem = ColorValue::Rgba(0, 0, 0, 255);
    // red 50% ↔ blue 50% → 介于两端、非黑、非红、非蓝
    let mk = |space| {
        ColorValue::Mix(Box::new(ColorMixSpec {
            c1: ColorMixComponent {
                color: ColorValue::Rgba(255, 0, 0, 255),
                percentage: Some(50.0),
            },
            c2: ColorMixComponent {
                color: ColorValue::Rgba(0, 0, 255, 255),
                percentage: Some(50.0),
            },
            space,
            hue: ColorHueMethod::Shorter,
        }))
    };
    for space in [
        ColorMixSpace::SrgbLinear,
        ColorMixSpace::Lab,
        ColorMixSpace::OkLab,
        ColorMixSpace::OkLch,
        ColorMixSpace::Xyz,
    ] {
        let r = resolve_color_current(&mk(space), &black_elem);
        assert_ne!((r.r, r.g, r.b), (0, 0, 0), "{space:?} 应解析非黑（非回退）");
        assert_ne!((r.r, r.g, r.b), (255, 0, 0), "{space:?} 不应恒等于 red");
        assert_ne!((r.r, r.g, r.b), (0, 0, 255), "{space:?} 不应恒等于 blue");
    }
    // 同色 mix = 该色（identity，四空间均成立）
    let mk_same = |space| {
        ColorValue::Mix(Box::new(ColorMixSpec {
            c1: ColorMixComponent {
                color: ColorValue::Rgba(0, 128, 0, 255),
                percentage: Some(50.0),
            },
            c2: ColorMixComponent {
                color: ColorValue::Rgba(0, 128, 0, 255),
                percentage: Some(50.0),
            },
            space,
            hue: ColorHueMethod::Shorter,
        }))
    };
    for space in [
        ColorMixSpace::SrgbLinear,
        ColorMixSpace::Lab,
        ColorMixSpace::OkLab,
        ColorMixSpace::OkLch,
        ColorMixSpace::Xyz,
    ] {
        let r = resolve_color_current(&mk_same(space), &black_elem);
        assert_eq!((r.r, r.g, r.b), (0, 128, 0), "{space:?} 同色 mix 应回该色");
    }
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
        space: None,
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
        space: None,
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
/// R2274：wide-gamut RCS resolve——lab/lch/oklab/oklch origin→目标空间→通道覆盖→回 sRGB。
/// 验证 sRGB↔各空间转换精度（全 Ref 往返≈origin）+ 数字覆盖语义（L/hue 覆盖）+ alpha 覆盖。
fn test_resolve_relative_color_wide_gamut() {
    use zero_css_parser::values::{RcsAlpha, RcsChannel, RelativeColorFunc, RelativeColorSpec};
    let mk = |func: RelativeColorFunc, channels: [RcsChannel; 3], alpha: RcsAlpha, origin: ColorValue| {
        ColorValue::RelativeColor(Box::new(RelativeColorSpec {
            func,
            origin,
            channels,
            alpha,
            space: None,
        }))
    };
    let white = ColorValue::Rgba(255, 255, 255, 255);

    // lab 覆盖 L=50 on white（a=b=0）→ 中灰 rgb(119,119,119)。
    let lab_gray = mk(
        RelativeColorFunc::Lab,
        [RcsChannel::Num(50.0), RcsChannel::Ref(1), RcsChannel::Ref(2)],
        RcsAlpha::Origin,
        white.clone(),
    );
    let r = resolve_color_current(&lab_gray, &white);
    assert_eq!(r.r, r.g, "lab(50 0 0) 应为灰（r==g）");
    assert_eq!(r.g, r.b, "lab(50 0 0) 应为灰（g==b）");
    assert!(r.r > 110 && r.r < 130, "lab(50 0 0) ≈ rgb(119,119,119)，实际 {}", r.r);

    // oklab 全 Ref 往返 red：应回到 red（±2/通道，验证 sRGB↔OKLab 转换精度）。
    let oklab_rt = mk(
        RelativeColorFunc::Oklab,
        [RcsChannel::Ref(0), RcsChannel::Ref(1), RcsChannel::Ref(2)],
        RcsAlpha::Origin,
        ColorValue::Rgba(255, 0, 0, 255),
    );
    let r = resolve_color_current(&oklab_rt, &ColorValue::Rgba(255, 0, 0, 255));
    assert!(
        (r.r as i32 - 255).abs() <= 2 && r.g <= 2 && r.b <= 2,
        "oklab 往返 red 应≈red，实际 {:?}",
        (r.r, r.g, r.b)
    );

    // oklch 色相覆盖（red h≈29°，覆盖为 180° → 互补青绿，r 跌、g 涨）。
    let oklch_hue = mk(
        RelativeColorFunc::Oklch,
        [RcsChannel::Ref(0), RcsChannel::Ref(1), RcsChannel::Num(180.0)],
        RcsAlpha::Origin,
        ColorValue::Rgba(255, 0, 0, 255),
    );
    let r = resolve_color_current(&oklch_hue, &ColorValue::Rgba(255, 0, 0, 255));
    assert!(
        r.r < 100 && r.g > 100,
        "oklch(red l c 180) hue 翻转→青绿，实际 {:?}",
        (r.r, r.g, r.b)
    );

    // alpha 覆盖：lab(from white l a b / 0.5) → white alpha 0.5。
    let lab_alpha = mk(
        RelativeColorFunc::Lab,
        [RcsChannel::Ref(0), RcsChannel::Ref(1), RcsChannel::Ref(2)],
        RcsAlpha::Num(0.5),
        white.clone(),
    );
    let r = resolve_color_current(&lab_alpha, &white);
    assert_eq!(r.a, 128, "alpha 0.5 → 128");
    assert_eq!((r.r, r.g, r.b), (255, 255, 255), "全 Ref → origin white 往返");
}

#[test]
/// R2277：color() RCS 非 identity resolve——origin sRGB → 预定义空间 → 通道覆盖 → 回 sRGB。
/// 关键验证：全 Ref（保留 origin 各通道）应在所有空间往返 ≈ origin（±3/通道）—— 这是 0 reftest
/// footprint 下的正确性 backstop（inverse 经 mat3_inverse 数值推导，encode/decode 互逆，仅 u8 量化损失）。
fn test_resolve_relative_color_color_function() {
    use zero_css_parser::values::{RcsAlpha, RcsChannel, RelativeColorFunc, RelativeColorSpec};
    let mk = |space: &str, channels: [RcsChannel; 3], origin: ColorValue| {
        ColorValue::RelativeColor(Box::new(RelativeColorSpec {
            func: RelativeColorFunc::Color,
            origin,
            channels,
            alpha: RcsAlpha::Origin,
            space: Some(space.to_string()),
        }))
    };
    let all_ref = [RcsChannel::Ref(0), RcsChannel::Ref(1), RcsChannel::Ref(2)];

    // 全 Ref 往返：多个 in-gamut 色 × 多个空间（覆盖 trivial / mat3_inverse×4 / XYZ 路径）。
    let spaces = [
        "srgb",
        "srgb-linear",
        "display-p3",
        "display-p3-linear",
        "a98-rgb",
        "a98-rgb-linear",
        "rec2020",
        "rec2020-linear",
        "prophoto-rgb",
        "prophoto-rgb-linear",
        "xyz",
        "xyz-d50",
        "xyz-d65",
    ];
    let colors = [(255u8, 0, 0), (0, 128, 0), (0, 0, 255), (128, 128, 128), (200, 100, 50)];
    for &(r0, g0, b0) in &colors {
        let ov = ColorValue::Rgba(r0, g0, b0, 255);
        for &space in &spaces {
            let r = resolve_color_current(&mk(space, all_ref, ov.clone()), &ov);
            assert!(
                (r.r as i32 - r0 as i32).abs() <= 3
                    && (r.g as i32 - g0 as i32).abs() <= 3
                    && (r.b as i32 - b0 as i32).abs() <= 3,
                "color(from {:?} {} r g b) 往返应≈origin {:?}，实际 {:?}",
                space,
                space,
                (r0, g0, b0),
                (r.r, r.g, r.b)
            );
        }
    }

    // 通道覆盖：color(from red srgb 0.5 g b) → r=0.5*255≈128, g/b 保留 origin red 的 0 → rgb(128,0,0)。
    let red = ColorValue::Rgba(255, 0, 0, 255);
    let r = resolve_color_current(
        &mk(
            "srgb",
            [RcsChannel::Num(0.5), RcsChannel::Ref(1), RcsChannel::Ref(2)],
            red.clone(),
        ),
        &red,
    );
    assert!(
        (r.r as i32 - 128).abs() <= 2 && r.g <= 2 && r.b <= 2,
        "color(from red srgb 0.5 g b) ≈ rgb(128,0,0)，实际 {:?}",
        (r.r, r.g, r.b)
    );

    // alpha 覆盖：color(from red srgb r g b / 0.5) → red alpha 128。
    let r = resolve_color_current(
        &ColorValue::RelativeColor(Box::new(RelativeColorSpec {
            func: RelativeColorFunc::Color,
            origin: red.clone(),
            channels: all_ref,
            alpha: RcsAlpha::Num(0.5),
            space: Some("srgb".to_string()),
        })),
        &red,
    );
    assert_eq!(r.a, 128, "alpha 0.5 → 128");
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

#[test]
/// R2273：color-mix(in lch) 极坐标插值——purple/plum 50-50 应接近 WPT ref rgb(175,92,174)
///（color-mix-percents-01 ref = rgb(68.4898% 36.015% 68.3102%)）。driving: color-mix-percents-01/02。
fn test_mix_lch_purple_plum() {
    use zero_css_parser::values::ColorHueMethod;
    let purple = Color::rgb(128, 0, 128);
    let plum = Color::rgb(221, 160, 221);
    // 50%/50%（双省略也归一为 50/50）
    let r = mix_lch(purple, Some(50.0), plum, Some(50.0), ColorHueMethod::Shorter);
    // WPT ref：≈ (175, 92, 174)。矩阵精度容差 ±4/通道。
    assert!(
        (r.r as i32 - 175).abs() <= 4 && (r.g as i32 - 92).abs() <= 4 && (r.b as i32 - 174).abs() <= 4,
        "mix(in lch, purple, plum) 应≈rgb(175,92,174)，实际 ({},{},{})",
        r.r,
        r.g,
        r.b
    );
    // 百分比省略归一化（purple, plum → 50/50）应等价
    let r2 = mix_lch(purple, None, plum, None, ColorHueMethod::Shorter);
    assert_eq!((r2.r, r2.g, r2.b), (r.r, r.g, r.b), "省略百分比应等价 50/50");
    assert_eq!(r.a, 255, "两端不透明 → alpha 255");
    // 色相短弧：red↔yellow（0°↔60°）应走短弧 30°→橙色，非长弧走 210°
    let red = Color::rgb(255, 0, 0);
    let yellow = Color::rgb(255, 255, 0);
    let ry = mix_lch(red, Some(50.0), yellow, Some(50.0), ColorHueMethod::Shorter);
    assert!(
        (ry.r as i32 - 255).abs() <= 5 && ry.g > 80,
        "red↔yellow 短弧→橙色，实际 ({},{},{})",
        ry.r,
        ry.g,
        ry.b
    );
}

#[test]
/// R2381：color-mix hue method（CSS Color 4 §12.3）—— longer hue 应与 shorter hue 产生不同
/// 中点色（red↔lime 短弧经黄、长弧经洋红/蓝）。验证 mix_lch 接受 hue 参数且 longer≠shorter。
fn test_mix_lch_hue_method_longer_vs_shorter() {
    use zero_css_parser::values::ColorHueMethod;
    let red = Color::rgb(255, 0, 0);
    let lime = Color::rgb(0, 255, 0);
    let shorter = mix_lch(red, Some(50.0), lime, Some(50.0), ColorHueMethod::Shorter);
    let longer = mix_lch(red, Some(50.0), lime, Some(50.0), ColorHueMethod::Longer);
    // 短弧 red↔lime 中点偏黄/橙（r、g 都高）；长弧绕远路经洋红/蓝（b 明显高于短弧）。
    assert_ne!(
        (shorter.r, shorter.g, shorter.b),
        (longer.r, longer.g, longer.b),
        "longer hue 应与 shorter 产生不同中点色"
    );
    assert!(
        longer.b > shorter.b + 20,
        "长弧中点应更蓝（绕洋红/蓝），shorter={:?} longer={:?}",
        shorter,
        longer
    );
}
