//! 渐变颜色插值的色彩空间转换数学（CSS Color 4 §12 / §13）。
//!
//! render-foundation 不依赖 css-parser（层级：渲染层不解析 CSS），故此处自带一份
//! sRGB↔Lab/LCH/OKLab/OKLCH/线性 sRGB 转换数学，与 `css-parser/src/values/color.rs`
//! （R2269/R2273 建立）同源。DRY 机会：未来可抽公共 color-math crate。
//!
//! driving: CSS Color 4 `gradient in <colorspace>` 颜色空间感知插值（R2289）。

use crate::color::Color;
use crate::primitive::{GradientColorSpace, HueMethod};

// ── sRGB 传递函数（CSS Color 4）────────────────────────────────────────

fn srgb_decode(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_encode(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn linear_srgb_to_u8(c: f64) -> u8 {
    (srgb_encode(c) * 255.0).round().clamp(0.0, 255.0) as u8
}

fn mat3_mul(m: [f64; 9], x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (
        m[0] * x + m[1] * y + m[2] * z,
        m[3] * x + m[4] * y + m[5] * z,
        m[6] * x + m[7] * y + m[8] * z,
    )
}

// XYZ-D65 → 线性 sRGB / 线性 sRGB → XYZ-D65 / Bradford 适应（CSS Color 4 标准值）
const XYZ_TO_SRGB: [f64; 9] = [
    3.2409699, -1.5373832, -0.4986108, -0.9692436, 1.8759675, 0.0415551, 0.0556300, -0.2039770, 1.0569715,
];
const SRGB_TO_XYZ: [f64; 9] = [
    0.4123908, 0.3575843, 0.1804808, 0.2126390, 0.7151687, 0.0721923, 0.0193308, 0.1191948, 0.9505322,
];
const XYZ_D50_TO_D65: [f64; 9] = [
    0.9555766, -0.0230393, 0.0631636, -0.0282895, 1.0099416, 0.0210077, 0.0122982, -0.0204830, 1.3299098,
];
const XYZ_D65_TO_D50: [f64; 9] = [
    1.0478112, 0.0228866, -0.0501270, 0.0295424, 0.9904844, -0.0170491, -0.0092345, 0.0150436, 0.7521316,
];

// ── Lab / LCH（CIE，D50 参考）─────────────────────────────────────────

fn lab_components_from_xyz_d50(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let (xn, yn, zn) = (0.96422, 1.0, 0.82521);
    let eps = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    let f = |t: f64| if t > eps { t.cbrt() } else { (kappa * t + 16.0) / 116.0 };
    let fx = f(x / xn);
    let fy = f(y / yn);
    let fz = f(z / zn);
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

fn lab_to_linear_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let t0 = 6.0 / 29.0;
    let f_inv = |t: f64| {
        if t > t0 {
            t * t * t
        } else {
            (116.0 * t - 16.0) * 27.0 / 24389.0
        }
    };
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    let (x, y, z) = (f_inv(fx) * 0.96422, f_inv(fy) * 1.0, f_inv(fz) * 0.82521);
    let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
    mat3_mul(XYZ_TO_SRGB, x, y, z)
}

fn srgb_u8_to_lab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (lr, lg, lb) = (
        srgb_decode(r as f64 / 255.0),
        srgb_decode(g as f64 / 255.0),
        srgb_decode(b as f64 / 255.0),
    );
    let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
    let (x, y, z) = mat3_mul(XYZ_D65_TO_D50, x, y, z);
    lab_components_from_xyz_d50(x, y, z)
}

fn lab_to_srgb_u8(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    let (lr, lg, lb) = lab_to_linear_srgb(l, a, b);
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

// ── OKLab / OKLCH（CSS Color 4 §10.4 线性 LMS）─────────────────────────

fn oklab_to_linear_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    (
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )
}

fn linear_srgb_to_oklab(lr: f64, lg: f64, lb: f64) -> (f64, f64, f64) {
    let l_ = (0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb).cbrt();
    let m_ = (0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb).cbrt();
    let s_ = (0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb).cbrt();
    (
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    )
}

fn srgb_u8_to_oklab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (lr, lg, lb) = (
        srgb_decode(r as f64 / 255.0),
        srgb_decode(g as f64 / 255.0),
        srgb_decode(b as f64 / 255.0),
    );
    linear_srgb_to_oklab(lr, lg, lb)
}

fn oklab_to_srgb_u8(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    let (lr, lg, lb) = oklab_to_linear_srgb(l, a, b);
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

// ── 色相插值（CSS Color 4 §13.5）──────────────────────────────────────

/// 计算两色相 h0→h1 在指定 hue method 下的插值色相（h0 + t·delta）。
/// h0/h1 单位为度 [0,360)。
fn interp_hue(h0: f64, h1: f64, t: f64, method: HueMethod) -> f64 {
    // delta 归一到所需弧段
    let delta = match method {
        HueMethod::Shorter => {
            // 短弧 → [-180, 180]
            ((h1 - h0 + 540.0) % 360.0) - 180.0
        }
        HueMethod::Longer => {
            // 长弧 → 取 |delta| ≥ 180 的方向（与短弧相反）
            let d = ((h1 - h0 + 540.0) % 360.0) - 180.0;
            if d.abs() < 1e-12 {
                360.0 // 同色相：长弧绕一整圈
            } else if d > 0.0 {
                d - 360.0
            } else {
                d + 360.0
            }
        }
        HueMethod::Increasing => {
            // 恒增 → (0, 360]
            let d = (h1 - h0) % 360.0;
            if d <= 0.0 { d + 360.0 } else { d }
        }
        HueMethod::Decreasing => {
            // 恒减 → [-360, 0)
            let d = (h1 - h0) % 360.0;
            if d >= 0.0 { d - 360.0 } else { d }
        }
    };
    (h0 + delta * t).rem_euclid(360.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// 在指定色彩空间内对两个色标颜色做分段插值，返回插值后的 sRGB 颜色。
///
/// `t` 为本段内的局部进度 [0,1]。alpha 始终在 sRGB 域线性插值（与既有行为一致；
/// 多数 fixture 为不透明色，premultiply 无差异）。
pub fn interp_pair(c0: Color, c1: Color, t: f64, space: GradientColorSpace, hue: HueMethod) -> Color {
    let a = lerp(c0.a as f64, c1.a as f64, t).round().clamp(0.0, 255.0) as u8;
    let (r, g, b) = match space {
        // sRGB（gamma 编码）：保留既有逐通道 lerp，保证 `in srgb` 字节级一致（零回归）。
        GradientColorSpace::Srgb => {
            let f = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round().clamp(0.0, 255.0) as u8;
            (f(c0.r, c1.r), f(c0.g, c1.g), f(c0.b, c1.b))
        }
        // 线性 sRGB：decode → lerp → encode
        GradientColorSpace::SrgbLinear => {
            let (lr0, lg0, lb0) = (
                srgb_decode(c0.r as f64 / 255.0),
                srgb_decode(c0.g as f64 / 255.0),
                srgb_decode(c0.b as f64 / 255.0),
            );
            let (lr1, lg1, lb1) = (
                srgb_decode(c1.r as f64 / 255.0),
                srgb_decode(c1.g as f64 / 255.0),
                srgb_decode(c1.b as f64 / 255.0),
            );
            (
                linear_srgb_to_u8(lerp(lr0, lr1, t)),
                linear_srgb_to_u8(lerp(lg0, lg1, t)),
                linear_srgb_to_u8(lerp(lb0, lb1, t)),
            )
        }
        // Lab：L/a/b 线性 lerp
        GradientColorSpace::Lab => {
            let (l0, a0, b0) = srgb_u8_to_lab(c0.r, c0.g, c0.b);
            let (l1, a1, b1) = srgb_u8_to_lab(c1.r, c1.g, c1.b);
            lab_to_srgb_u8(lerp(l0, l1, t), lerp(a0, a1, t), lerp(b0, b1, t))
        }
        // OKLab：L/a/b 线性 lerp
        GradientColorSpace::Oklab => {
            let (l0, a0, b0) = srgb_u8_to_oklab(c0.r, c0.g, c0.b);
            let (l1, a1, b1) = srgb_u8_to_oklab(c1.r, c1.g, c1.b);
            oklab_to_srgb_u8(lerp(l0, l1, t), lerp(a0, a1, t), lerp(b0, b1, t))
        }
        // LCH：L/C lerp，h 按 hue method
        GradientColorSpace::Lch => {
            let (l0, c0c, h0) = to_lch(c0);
            let (l1, c1c, h1) = to_lch(c1);
            let h = interp_hue(h0, h1, t, hue);
            lch_to_srgb_u8_via_lab(lerp(l0, l1, t), lerp(c0c, c1c, t), h)
        }
        // OKLCH：L/C lerp，h 按 hue method
        GradientColorSpace::Oklch => {
            let (l0, c0c, h0) = to_oklch(c0);
            let (l1, c1c, h1) = to_oklch(c1);
            let h = interp_hue(h0, h1, t, hue);
            oklch_to_srgb_u8_via_oklab(lerp(l0, l1, t), lerp(c0c, c1c, t), h)
        }
    };
    Color::rgba(r, g, b, a)
}

fn to_lch(c: Color) -> (f64, f64, f64) {
    let (l, a, b) = srgb_u8_to_lab(c.r, c.g, c.b);
    let chroma = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, chroma, h)
}

fn to_oklch(c: Color) -> (f64, f64, f64) {
    let (l, a, b) = srgb_u8_to_oklab(c.r, c.g, c.b);
    let chroma = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, chroma, h)
}

fn lch_to_srgb_u8_via_lab(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let h_rad = h.to_radians();
    lab_to_srgb_u8(l, c * h_rad.cos(), c * h_rad.sin())
}

fn oklch_to_srgb_u8_via_oklab(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let h_rad = h.to_radians();
    oklab_to_srgb_u8(l, c * h_rad.cos(), c * h_rad.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: u8, b: u8) -> bool {
        (a as i16 - b as i16).abs() <= 3
    }

    /// oklab 中点：red↔lime，chromium 的 `oklab-gradient` 期望中段偏暗的橄榄色。
    #[test]
    fn test_oklab_midpoint_differs_from_srgb() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let srgb_mid = interp_pair(red, lime, 0.5, GradientColorSpace::Srgb, HueMethod::Shorter);
        let oklab_mid = interp_pair(red, lime, 0.5, GradientColorSpace::Oklab, HueMethod::Shorter);
        // srgb 中点 = (128,128,0)；oklab 中点应明显不同（oklab 在红绿间更暗、更灰）
        assert_eq!(srgb_mid, Color::rgba(128, 128, 0, 255));
        assert!(
            (oklab_mid.r as i16 - srgb_mid.r as i16).abs() > 5 || (oklab_mid.g as i16 - srgb_mid.g as i16).abs() > 5,
            "oklab midpoint should differ from srgb: {:?} vs {:?}",
            oklab_mid,
            srgb_mid
        );
    }

    /// srgb-linear 中点：red↔lime，线性光中点比 gamma 编码中点亮。
    #[test]
    fn test_srgb_linear_midpoint_brighter() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let lin = interp_pair(red, lime, 0.5, GradientColorSpace::SrgbLinear, HueMethod::Shorter);
        let gamma = interp_pair(red, lime, 0.5, GradientColorSpace::Srgb, HueMethod::Shorter);
        // 线性光中点亮于 gamma（red 分量 ≈188 vs 128）
        assert!(
            lin.r > gamma.r + 20,
            "srgb-linear midpoint should be brighter: lin.r={} gamma.r={}",
            lin.r,
            gamma.r
        );
    }

    /// LCH shorter hue：red(h≈40.7)↔lime(h≈135.8) 短弧中点色相应在两色之间（偏黄橙→黄绿）。
    #[test]
    fn test_lch_shorter_hue_goes_short_way() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let shorter = interp_pair(red, lime, 0.5, GradientColorSpace::Lch, HueMethod::Shorter);
        // 短弧中点：L/C 中点 + 中间色相 → 偏黄；R 与 G 应都较高
        assert!(
            shorter.r > 100 && shorter.g > 100,
            "lch shorter hue mid should be yellowish: {:?}",
            shorter
        );
    }

    /// LCH longer hue：red↔lime 长弧绕远路（经 magenta/blue），中点应偏蓝紫。
    #[test]
    fn test_lch_longer_hue_goes_long_way() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let longer = interp_pair(red, lime, 0.5, GradientColorSpace::Lch, HueMethod::Longer);
        // 长弧中点色相 ≈ (40.7+135.8)/2 + 180 ... 经蓝紫；B 应明显高于短弧中点
        let shorter = interp_pair(red, lime, 0.5, GradientColorSpace::Lch, HueMethod::Shorter);
        assert!(
            longer.b > shorter.b + 30,
            "lch longer hue mid should be bluish: longer.b={} shorter.b={}",
            longer.b,
            shorter.b
        );
    }

    /// Lab 中点近似 identity 往返（同色 → 同色）。
    #[test]
    fn test_lab_identity() {
        let c = Color::rgba(100, 150, 200, 255);
        let out = interp_pair(c, c, 0.5, GradientColorSpace::Lab, HueMethod::Shorter);
        assert!(approx(out.r, c.r) && approx(out.g, c.g) && approx(out.b, c.b));
    }

    /// hue increasing：恒增方向。
    #[test]
    fn test_hue_increasing_decreasing_direction() {
        // h0=10, h1=30：increasing delta=20；decreasing delta=-340
        let inc = interp_hue(10.0, 30.0, 0.5, HueMethod::Increasing);
        assert!(inc > 10.0 && inc < 30.0, "increasing mid should be in (10,30): {}", inc);
        let dec = interp_hue(10.0, 30.0, 0.5, HueMethod::Decreasing);
        // dec 方向：10 + (-340)*0.5 = -160 → rem_euclid 200
        assert!((dec - 200.0).abs() < 1.0, "decreasing mid should be ~200: {}", dec);
    }
}
