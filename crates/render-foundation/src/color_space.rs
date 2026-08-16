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
/// h0/h1 单位为度 [0,360)。pub 供 engine color-mix 复用（R2381）。
pub fn interp_hue(h0: f64, h1: f64, t: f64, method: HueMethod) -> f64 {
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
/// `t` 为本段内的局部进度 [0,1]。**CSS Color 4 §12.3 premultiplied alpha**：颜色分量在
/// 插值空间内先乘以源 alpha（premultiply），插值后再除以结果 alpha（un-premultiply）。
/// 不透明色（alpha=255）premultiply = identity → 与既有行为字节级一致（零回归）；半透明色
/// 才发散（spec-correct）。alpha 始终独立线性插值。极坐标空间（LCH/OKLCH）的色相不参与
/// premultiply（角度无量纲）。
pub fn interp_pair(c0: Color, c1: Color, t: f64, space: GradientColorSpace, hue: HueMethod) -> Color {
    let a0 = c0.a as f64 / 255.0;
    let a1 = c1.a as f64 / 255.0;
    let a_f = lerp(a0, a1, t); // 结果 alpha（0..1）
    let a_u8 = (a_f * 255.0).round().clamp(0.0, 255.0) as u8;
    // un-premultiply 分量到 u8（输入 v 为 0..255 尺度；结果 alpha 为 0 时颜色无意义 → 0）。
    let unmulp = |v: f64| -> u8 {
        if a_f > 1e-10 {
            (v / a_f).round().clamp(0.0, 255.0) as u8
        } else {
            0
        }
    };
    let (r, g, b) = match space {
        // sRGB（gamma 编码）：premultiply channel×alpha（0..255 尺度）→ lerp → un-premultiply。
        // 不透明（alpha=1）：×1/÷1 精确 → 与既有逐通道 lerp 字节级一致（零回归）。
        GradientColorSpace::Srgb => {
            let (r0, g0, b0) = (c0.r as f64 * a0, c0.g as f64 * a0, c0.b as f64 * a0);
            let (r1, g1, b1) = (c1.r as f64 * a1, c1.g as f64 * a1, c1.b as f64 * a1);
            (
                unmulp(lerp(r0, r1, t)),
                unmulp(lerp(g0, g1, t)),
                unmulp(lerp(b0, b1, t)),
            )
        }
        // 线性 sRGB：decode → premultiply ×alpha → lerp → un-premultiply → encode
        GradientColorSpace::SrgbLinear => {
            let decode_premul = |c: &Color, a: f64| {
                (
                    srgb_decode(c.r as f64 / 255.0) * a,
                    srgb_decode(c.g as f64 / 255.0) * a,
                    srgb_decode(c.b as f64 / 255.0) * a,
                )
            };
            let (lr0, lg0, lb0) = decode_premul(&c0, a0);
            let (lr1, lg1, lb1) = decode_premul(&c1, a1);
            (
                unmulp_lin(lerp(lr0, lr1, t), a_f),
                unmulp_lin(lerp(lg0, lg1, t), a_f),
                unmulp_lin(lerp(lb0, lb1, t), a_f),
            )
        }
        // Lab：L/a/b premultiply ×alpha → lerp → un-premultiply
        GradientColorSpace::Lab => {
            let (l0, la0, b0) = srgb_u8_to_lab(c0.r, c0.g, c0.b);
            let (l1, la1, b1) = srgb_u8_to_lab(c1.r, c1.g, c1.b);
            let (l, la, b) = unmulp3(
                lerp(l0 * a0, l1 * a1, t),
                lerp(la0 * a0, la1 * a1, t),
                lerp(b0 * a0, b1 * a1, t),
                a_f,
            );
            lab_to_srgb_u8(l, la, b)
        }
        // OKLab：L/a/b premultiply ×alpha → lerp → un-premultiply
        GradientColorSpace::Oklab => {
            let (l0, la0, b0) = srgb_u8_to_oklab(c0.r, c0.g, c0.b);
            let (l1, la1, b1) = srgb_u8_to_oklab(c1.r, c1.g, c1.b);
            let (l, la, b) = unmulp3(
                lerp(l0 * a0, l1 * a1, t),
                lerp(la0 * a0, la1 * a1, t),
                lerp(b0 * a0, b1 * a1, t),
                a_f,
            );
            oklab_to_srgb_u8(l, la, b)
        }
        // LCH：L/C premultiply ×alpha（hue 不 premultiply，按 hue method 插值）→ un-premultiply
        GradientColorSpace::Lch => {
            let (l0, c0c, h0) = to_lch(c0);
            let (l1, c1c, h1) = to_lch(c1);
            let h = interp_hue(h0, h1, t, hue);
            let (l, c) = unmulp2(lerp(l0 * a0, l1 * a1, t), lerp(c0c * a0, c1c * a1, t), a_f);
            lch_to_srgb_u8_via_lab(l, c, h)
        }
        // OKLCH：L/C premultiply ×alpha → un-premultiply
        GradientColorSpace::Oklch => {
            let (l0, c0c, h0) = to_oklch(c0);
            let (l1, c1c, h1) = to_oklch(c1);
            let h = interp_hue(h0, h1, t, hue);
            let (l, c) = unmulp2(lerp(l0 * a0, l1 * a1, t), lerp(c0c * a0, c1c * a1, t), a_f);
            oklch_to_srgb_u8_via_oklab(l, c, h)
        }
        // HSL：H 不 premultiply（hue method），L/S premultiply ×alpha。
        GradientColorSpace::Hsl => {
            let (h0, s0, l0) = srgb_u8_to_hsl(c0.r, c0.g, c0.b);
            let (h1, s1, l1) = srgb_u8_to_hsl(c1.r, c1.g, c1.b);
            let h = interp_hue(h0, h1, t, hue);
            let (s, l) = unmulp2(lerp(s0 * a0, s1 * a1, t), lerp(l0 * a0, l1 * a1, t), a_f);
            hsl_to_srgb_u8(h, s, l)
        }
        // HWB：H 不 premultiply，W/B premultiply ×alpha。
        GradientColorSpace::Hwb => {
            let (h0, w0, b0) = srgb_u8_to_hwb(c0.r, c0.g, c0.b);
            let (h1, w1, b1) = srgb_u8_to_hwb(c1.r, c1.g, c1.b);
            let h = interp_hue(h0, h1, t, hue);
            let (w, b) = unmulp2(lerp(w0 * a0, w1 * a1, t), lerp(b0 * a0, b1 * a1, t), a_f);
            hwb_to_srgb_u8(h, w, b)
        }
        // XYZ（D65）：线性 sRGB → XYZ premultiply ×alpha → lerp → 逆变换。
        GradientColorSpace::Xyz => {
            let (x0, y0, z0) = srgb_u8_to_xyz_d65(c0.r, c0.g, c0.b);
            let (x1, y1, z1) = srgb_u8_to_xyz_d65(c1.r, c1.g, c1.b);
            let (x, y, z) = unmulp3(
                lerp(x0 * a0, x1 * a1, t),
                lerp(y0 * a0, y1 * a1, t),
                lerp(z0 * a0, z1 * a1, t),
                a_f,
            );
            xyz_d65_to_srgb_u8(x, y, z)
        }
        // ProPhoto RGB（D50）：sRGB → XYZ(D65) → Bradford D50 → prophoto EOTF。
        GradientColorSpace::ProphotoRgb => {
            let (p0, p1, p2) = srgb_u8_to_prophoto(c0.r, c0.g, c0.b);
            let (q0, q1, q2) = srgb_u8_to_prophoto(c1.r, c1.g, c1.b);
            let (x, y, z) = unmulp3(
                lerp(p0 * a0, q0 * a1, t),
                lerp(p1 * a0, q1 * a1, t),
                lerp(p2 * a0, q2 * a1, t),
                a_f,
            );
            prophoto_to_srgb_u8(x, y, z)
        }
        // R57：Display-P3（gamma 编码，EOTF 同 sRGB）。
        GradientColorSpace::DisplayP3 => {
            let (p0, p1, p2) = srgb_u8_to_p3(c0.r, c0.g, c0.b);
            let (q0, q1, q2) = srgb_u8_to_p3(c1.r, c1.g, c1.b);
            let (x, y, z) = unmulp3(
                lerp(p0 * a0, q0 * a1, t),
                lerp(p1 * a0, q1 * a1, t),
                lerp(p2 * a0, q2 * a1, t),
                a_f,
            );
            p3_to_srgb_u8(x, y, z)
        }
        // Display-P3 线性光（无 EOTF，直用线性分量）。
        GradientColorSpace::DisplayP3Linear => {
            let (p0, p1, p2) = srgb_u8_to_xyz_d65(c0.r, c0.g, c0.b);
            let (q0, q1, q2) = srgb_u8_to_xyz_d65(c1.r, c1.g, c1.b);
            let (x, y, z) = mat3_mul(XYZ_TO_P3, p0, p1, p2);
            let (x2, y2, z2) = mat3_mul(XYZ_TO_P3, q0, q1, q2);
            let (i, j, k) = unmulp3(
                lerp(x * a0, x2 * a1, t),
                lerp(y * a0, y2 * a1, t),
                lerp(z * a0, z2 * a1, t),
                a_f,
            );
            p3_to_srgb_u8(i, j, k)
        }
        // Adobe RGB (1998)（gamma 563/256）。
        GradientColorSpace::A98Rgb => {
            let (a0c, a1c, a2c) = srgb_u8_to_a98(c0.r, c0.g, c0.b);
            let (b0c, b1c, b2c) = srgb_u8_to_a98(c1.r, c1.g, c1.b);
            let (x, y, z) = unmulp3(
                lerp(a0c * a0, b0c * a1, t),
                lerp(a1c * a0, b1c * a1, t),
                lerp(a2c * a0, b2c * a1, t),
                a_f,
            );
            a98_to_srgb_u8(x, y, z)
        }
        // Rec.2020（BT.2020 EOTF）。
        GradientColorSpace::Rec2020 => {
            let (r0c, r1c, r2c) = srgb_u8_to_rec2020(c0.r, c0.g, c0.b);
            let (s0c, s1c, s2c) = srgb_u8_to_rec2020(c1.r, c1.g, c1.b);
            let (x, y, z) = unmulp3(
                lerp(r0c * a0, s0c * a1, t),
                lerp(r1c * a0, s1c * a1, t),
                lerp(r2c * a0, s2c * a1, t),
                a_f,
            );
            rec2020_to_srgb_u8(x, y, z)
        }
        // XYZ-D50：sRGB → XYZ(D65) → Bradford D50 → lerp → D65 → sRGB。
        GradientColorSpace::XyzD50 => {
            let (x0, y0, z0) = srgb_u8_to_xyz_d65(c0.r, c0.g, c0.b);
            let (x1, y1, z1) = srgb_u8_to_xyz_d65(c1.r, c1.g, c1.b);
            let (x50a, y50a, z50a) = mat3_mul(XYZ_D65_TO_D50, x0, y0, z0);
            let (x50b, y50b, z50b) = mat3_mul(XYZ_D65_TO_D50, x1, y1, z1);
            let (x, y, z) = unmulp3(
                lerp(x50a * a0, x50b * a1, t),
                lerp(y50a * a0, y50b * a1, t),
                lerp(z50a * a0, z50b * a1, t),
                a_f,
            );
            let (x65, y65, z65) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
            xyz_d65_to_srgb_u8(x65, y65, z65)
        }
    };
    Color::rgba(r, g, b, a_u8)
}

/// 线性光分量 un-premultiply → sRGB u8（encode gamma + 钳制）。
fn unmulp_lin(v: f64, a_f: f64) -> u8 {
    let lin = if a_f > 1e-10 { v / a_f } else { 0.0 };
    linear_srgb_to_u8(lin)
}

/// 3 个插值空间分量 un-premultiply（结果 alpha 为 0 → 全 0）。
fn unmulp3(x: f64, y: f64, z: f64, a_f: f64) -> (f64, f64, f64) {
    if a_f > 1e-10 {
        (x / a_f, y / a_f, z / a_f)
    } else {
        (0.0, 0.0, 0.0)
    }
}

/// 2 个分量（L/C）un-premultiply。
fn unmulp2(x: f64, y: f64, a_f: f64) -> (f64, f64) {
    if a_f > 1e-10 { (x / a_f, y / a_f) } else { (0.0, 0.0) }
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

// ── HSL / HWB（sRGB 圆柱坐标，CSS Color 4）──────────────────────────

/// sRGB u8 → HSL（h 0..360 度；s/l 0..1）。
fn srgb_u8_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    let s = if d > 0.0 {
        d / (1.0 - (2.0 * l - 1.0).abs())
    } else {
        0.0
    };
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, s, l)
}

fn hsl_to_srgb_u8(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h.rem_euclid(360.0) / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as i64 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        (255.0 * (r1 + m)).round().clamp(0.0, 255.0) as u8,
        (255.0 * (g1 + m)).round().clamp(0.0, 255.0) as u8,
        (255.0 * (b1 + m)).round().clamp(0.0, 255.0) as u8,
    )
}

/// sRGB u8 → HWB（h 0..360 度；w/b 0..1）。
fn srgb_u8_to_hwb(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, min, 1.0 - max)
}

fn hwb_to_srgb_u8(h: f64, w: f64, b: f64) -> (u8, u8, u8) {
    let (w, b) = (w.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
    if w + b >= 1.0 {
        let v = w / (w + b);
        let x = (255.0 * v).round().clamp(0.0, 255.0) as u8;
        return (x, x, x);
    }
    let (r1, g1, b1) = hsl_to_srgb_components(h.rem_euclid(360.0) / 60.0);
    let scale = 1.0 - w - b;
    (
        (255.0 * (r1 * scale + w)).round().clamp(0.0, 255.0) as u8,
        (255.0 * (g1 * scale + w)).round().clamp(0.0, 255.0) as u8,
        (255.0 * (b1 * scale + w)).round().clamp(0.0, 255.0) as u8,
    )
}

/// HSL 色相段分量（无 m 偏移——HWB 用）。
fn hsl_to_srgb_components(hp: f64) -> (f64, f64, f64) {
    let c = 1.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    match hp as i64 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    }
}

// ── XYZ（D65）/ ProPhoto RGB（D50）──────────────────────────────────

/// 线性 sRGB u8 → XYZ（D65）。
fn srgb_u8_to_xyz_d65(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    mat3_mul(
        SRGB_TO_XYZ,
        srgb_decode(r as f64 / 255.0),
        srgb_decode(g as f64 / 255.0),
        srgb_decode(b as f64 / 255.0),
    )
}

/// XYZ（D65）→ 线性 sRGB u8（clamp——插值可能越出 sRGB 色域）。
fn xyz_d65_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (r, g, b) = mat3_mul(XYZ_TO_SRGB, x, y, z);
    (linear_srgb_to_u8(r), linear_srgb_to_u8(g), linear_srgb_to_u8(b))
}

/// ProPhoto RGB（D50）矩阵（CSS Color 4 §8.2）：prophoto 线性 → XYZ(D50)。
const PROPHOTO_TO_XYZ_D50: [f64; 9] = [
    0.797_760_489_672_302_7,
    0.135_185_837_175_740_3,
    0.031_349_349_581_435_8,
    0.288_071_128_229_293_4,
    0.711_843_217_810_101_4,
    0.000_085_653_960_605_259_2,
    0.0,
    0.0,
    0.825_104_602_510_460_1,
];
/// XYZ(D50) → ProPhoto RGB 线性。
const XYZ_D50_TO_PROPHOTO: [f64; 9] = [
    1.345_798_973_102_906_6,
    -0.255_580_100_079_975_3,
    -0.051_106_285_686_021_8,
    -0.544_622_493_449_452_8,
    1.508_232_741_313_938_6,
    0.020_536_032_391_437_5,
    0.0,
    0.0,
    1.211_967_545_638_945_4,
];

/// sRGB u8 → ProPhoto 线性分量（sRGB 线性 → XYZ(D65) → Bradford D50 → prophoto 矩阵）。
fn srgb_u8_to_prophoto(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (x65, y65, z65) = srgb_u8_to_xyz_d65(r, g, b);
    let (x50, y50, z50) = mat3_mul(XYZ_D65_TO_D50, x65, y65, z65);
    mat3_mul(XYZ_D50_TO_PROPHOTO, x50, y50, z50)
}

/// ProPhoto 线性分量 → sRGB u8（逆 EOTF + XYZ(D50)→D65 → 线性 sRGB）。
fn prophoto_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (r50, g50, b50) = mat3_mul(PROPHOTO_TO_XYZ_D50, x, y, z);
    let (r65, g65, b65) = mat3_mul(XYZ_D50_TO_D65, r50, g50, b50);
    xyz_d65_to_srgb_u8(r65, g65, b65)
}

// ── Display-P3 / Adobe RGB (1998) / Rec.2020（R57：canvas colorInterpolationMethod
//    完整 CSS Color 4 空间；数学与 css-parser color_math.rs 同源，层规则不跨依赖）────

/// Display-P3 线性 → XYZ(D65)。
const P3_TO_XYZ: [f64; 9] = [
    0.48657095, 0.26566769, 0.19821729, 0.22897456, 0.69173852, 0.07928691, 0.0, 0.04511338, 1.04394437,
];
/// XYZ(D65) → Display-P3 线性。
const XYZ_TO_P3: [f64; 9] = [
    2.49349691,
    -0.93138362,
    -0.40271078,
    -0.82948897,
    1.76266406,
    0.02362469,
    0.03584583,
    -0.07617239,
    0.95688452,
];
/// Adobe RGB (1998) 线性 → XYZ(D65)。
const A98_TO_XYZ: [f64; 9] = [
    0.57666904, 0.18555837, 0.18822865, 0.29734498, 0.62736357, 0.07529146, 0.02703136, 0.07068885, 0.99133754,
];
/// XYZ(D65) → Adobe RGB (1998) 线性。
const XYZ_TO_A98: [f64; 9] = [
    2.0415879,
    -0.56500697,
    -0.34473135,
    -0.96924326,
    1.87596752,
    0.04155506,
    0.01344428,
    -0.11836239,
    1.01517499,
];
/// Rec.2020 线性 → XYZ(D65)。
const REC2020_TO_XYZ: [f64; 9] = [
    0.63695805, 0.1446169, 0.16888097, 0.26270021, 0.67799807, 0.05930172, 0.0, 0.02807269, 1.06098506,
];
/// XYZ(D65) → Rec.2020 线性。
const XYZ_TO_REC2020: [f64; 9] = [
    1.71665119,
    -0.35567078,
    -0.25336628,
    -0.66668435,
    1.61648124,
    0.01576855,
    0.01763986,
    -0.04277061,
    0.94210312,
];

/// 安全幂（负分量 → 0，powf 对负数 NaN）。
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

/// BT.2020 传递函数逆（线性光 → 分量）。
fn rec2020_encode(lin: f64) -> f64 {
    const ALPHA: f64 = 1.09929682680944;
    const BETA: f64 = 0.018053968510807;
    if lin < BETA {
        lin * 4.5
    } else {
        ALPHA * safe_powf(lin, 0.45) - ALPHA + 1.0
    }
}

/// a98-rgb 传递函数逆（线性光 → 分量）：正向 gamma 563/256，逆 = 256/563。
fn a98_encode(c: f64) -> f64 {
    safe_powf(c, 256.0 / 563.0)
}

/// a98-rgb 传递函数（分量 → 线性光）。
fn a98_decode(c: f64) -> f64 {
    safe_powf(c, 563.0 / 256.0)
}

/// sRGB u8 → Display-P3 gamma 分量（sRGB EOTF → XYZ(D65) → 逆 p3 矩阵 → p3 EOTF）。
fn srgb_u8_to_p3(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (x, y, z) = srgb_u8_to_xyz_d65(r, g, b);
    let (p0, p1, p2) = mat3_mul(XYZ_TO_P3, x, y, z);
    (srgb_encode(p0), srgb_encode(p1), srgb_encode(p2))
}

/// Display-P3 gamma 分量 → sRGB u8。
fn p3_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (p0, p1, p2) = mat3_mul(P3_TO_XYZ, srgb_decode(x), srgb_decode(y), srgb_decode(z));
    xyz_d65_to_srgb_u8(p0, p1, p2)
}

/// sRGB u8 → Adobe RGB (1998) 分量。
fn srgb_u8_to_a98(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (x, y, z) = srgb_u8_to_xyz_d65(r, g, b);
    let (a0, a1, a2) = mat3_mul(XYZ_TO_A98, x, y, z);
    (a98_encode(a0), a98_encode(a1), a98_encode(a2))
}

/// Adobe RGB (1998) 分量 → sRGB u8。
fn a98_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (a0, a1, a2) = mat3_mul(A98_TO_XYZ, a98_decode(x), a98_decode(y), a98_decode(z));
    xyz_d65_to_srgb_u8(a0, a1, a2)
}

/// sRGB u8 → Rec.2020 分量。
fn srgb_u8_to_rec2020(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (x, y, z) = srgb_u8_to_xyz_d65(r, g, b);
    let (q0, q1, q2) = mat3_mul(XYZ_TO_REC2020, x, y, z);
    (rec2020_encode(q0), rec2020_encode(q1), rec2020_encode(q2))
}

/// Rec.2020 分量 → sRGB u8。
fn rec2020_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (q0, q1, q2) = mat3_mul(REC2020_TO_XYZ, rec2020_decode(x), rec2020_decode(y), rec2020_decode(z));
    xyz_d65_to_srgb_u8(q0, q1, q2)
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

    /// CSS Color 4 §12.3 premultiplied alpha：半透明色插值应 premultiply。
    /// rgba(255,0,0,128) [red 50%] ↔ rgba(0,0,255,255) [opaque blue] @ t=0.5：
    /// 非 premultiply 会给 rgb(128,0,128)；premultiply 给 rgb(85,0,170)（不透明端主导）。
    #[test]
    fn test_premultiplied_alpha_semi_transparent() {
        let red_half = Color::rgba(255, 0, 0, 128);
        let blue = Color::rgba(0, 0, 255, 255);
        let mid = interp_pair(red_half, blue, 0.5, GradientColorSpace::Srgb, HueMethod::Shorter);
        // 结果 alpha = lerp(128/255, 1.0, 0.5) ≈ 0.751 → 191
        assert!(
            (mid.a as i16 - 191).abs() <= 1,
            "result alpha should be ~191, got {}",
            mid.a
        );
        // premultiply：r 由不透明端少贡献（≈85），b 由不透明端主导（≈170）。
        // 关键：与「非 premultiply 的 rgb(128,0,128)」明显不同——b 应远大于 r。
        assert!(
            mid.b > mid.r + 60,
            "premultiplied: blue should dominate red (b={}, r={})",
            mid.b,
            mid.r
        );
        assert!(
            approx(mid.r, 85) && approx(mid.b, 170),
            "mid ≈ rgb(85,0,170), got {:?}",
            mid
        );
    }

    /// premultiply 对不透明色 = identity（零回归守护）：opaque srgb 中点不变。
    #[test]
    fn test_premultiplied_identity_for_opaque() {
        let red = Color::rgba(255, 0, 0, 255);
        let blue = Color::rgba(0, 0, 255, 255);
        let mid = interp_pair(red, blue, 0.5, GradientColorSpace::Srgb, HueMethod::Shorter);
        // 不透明：与既有逐通道 lerp 一致 = rgb(128,0,128) alpha 255
        assert_eq!(mid, Color::rgba(128, 0, 128, 255));
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

#[cfg(test)]
mod r56h_tests {
    use super::*;

    fn approx(a: u8, b: u8) -> bool {
        (a as i16 - b as i16).abs() <= 3
    }

    /// HSL 中点：red↔lime 的 H 插值（shorter）→ 黄。
    #[test]
    fn test_hsl_midpoint() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let c = interp_pair(red, lime, 0.5, GradientColorSpace::Hsl, HueMethod::Shorter);
        // H=60°（黄）——sRGB 逐通道中点则是 128,128,0 的暗黄。
        assert!(c.r > 200 && c.g > 200, "HSL 中点应偏黄: {c:?}");
        assert!(c.b < 50, "B 通道低: {c:?}");
    }

    /// HWB 往返：red 的 H=0, W=0, B=0 → 还原红。
    #[test]
    fn test_hwb_roundtrip_red() {
        let (h, w, b) = srgb_u8_to_hwb(255, 0, 0);
        assert!(h.abs() < 1e-6 && w.abs() < 1e-6 && b.abs() < 1e-6, "{h} {w} {b}");
        let (r, g, b2) = hwb_to_srgb_u8(h, w, b);
        assert!(approx(r, 255) && approx(g, 0) && approx(b2, 0), "{r} {g} {b2}");
    }

    /// XYZ 中点：red↔lime → 线性空间的暗黄（与 sRGB 逐通道不同）。
    #[test]
    fn test_xyz_midpoint_differs_from_srgb() {
        let red = Color::rgba(255, 0, 0, 255);
        let lime = Color::rgba(0, 255, 0, 255);
        let xyz = interp_pair(red, lime, 0.5, GradientColorSpace::Xyz, HueMethod::Shorter);
        let srgb = interp_pair(red, lime, 0.5, GradientColorSpace::Srgb, HueMethod::Shorter);
        assert!(xyz.r != srgb.r || xyz.g != srgb.g, "XYZ 与 sRGB 中点应不同");
    }

    /// ProPhoto 往返：红在 prophoto 空间插值自身。
    #[test]
    fn test_prophoto_roundtrip_red() {
        let (x, y, z) = srgb_u8_to_prophoto(255, 0, 0);
        let (r, g, b) = prophoto_to_srgb_u8(x, y, z);
        assert!(approx(r, 255) && approx(g, 0) && approx(b, 0), "{r} {g} {b}");
    }
}
