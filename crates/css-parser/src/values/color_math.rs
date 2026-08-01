//! CSS 色彩空间转换数学（sRGB ↔ Lab/LCH/OKLab/OKLCH/XYZ/预定义空间）。
//!
//! 纯数值（f64/u8）转换函数 + 3×3 矩阵常量，无 ColorValue/解析依赖。从 color.rs
//! 拆出（run-rules §5 文件大小控制，R2277「解析 vs 转换数学」flag）。pub 函数经
//! values::* re-export 供 engine resolve_color_current / mix_lch 等消费。

/// CIE Lab → 线性 sRGB（CSS Color 4 §10.3：Lab → XYZ-D50 → XYZ-D65 → 线性 sRGB）。
/// L∈[0,100]，a/b 为 a*/b*（无固定范围，常见 ±125）。
fn lab_to_linear_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    // f⁻¹（CSS Color 4：t > 6/29 → t³，否则 (116t−16)·27/24389）
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
    // XYZ-D50（D50 白点 Xn=0.96422, Yn=1.0, Zn=0.82521）
    let x = f_inv(fx) * 0.96422;
    let y = f_inv(fy) * 1.0;
    let z = f_inv(fz) * 0.82521;
    // XYZ-D50 → XYZ-D65（Bradford）→ 线性 sRGB
    let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
    mat3_mul(XYZ_TO_SRGB, x, y, z)
}

/// sRGB u8 → CIE Lab-D50（CSS Color 4）：L∈[0,100]、a/b。供 RCS lab()/lch() origin 解析 + color-mix。
pub fn srgb_u8_to_lab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    // sRGB gamma → 线性 sRGB → XYZ-D65 → XYZ-D50（Bradford）→ Lab
    let lr = srgb_decode(r as f64 / 255.0);
    let lg = srgb_decode(g as f64 / 255.0);
    let lb = srgb_decode(b as f64 / 255.0);
    let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
    let (x, y, z) = mat3_mul(XYZ_D65_TO_D50, x, y, z);
    lab_components_from_xyz_d50(x, y, z)
}

/// sRGB u8 → CIE LCH-D50（CSS Color 4）：L∈[0,100]、C、h∈[0,360)。供 color-mix(in lch) 插值。
pub fn srgb_u8_to_lch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (l, a, b) = srgb_u8_to_lab(r, g, b);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h)
}

/// CIE Lab-D50 → sRGB u8（CSS Color 4；越界色逐通道钳制）。供 RCS lab() 输出回转。
/// L 边界（L≤0→黑、L≥100→白）：L=0/100 平面退化为黑/白单点，任意 chroma 解析为黑/白
///（driving: lch-009/010）。近边界（0<L<100）逐通道钳制（byte-identical 于 R2323）。
pub fn lab_to_srgb_u8(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    let l = l.clamp(0.0, 100.0);
    if l <= 0.0 {
        return (0, 0, 0);
    }
    if l >= 100.0 {
        return (255, 255, 255);
    }
    let (lr, lg, lb) = lab_to_linear_srgb(l, a, b);
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

/// XYZ-D50 → Lab（L, a, b）（CSS Color 4；f 正函数）。
fn lab_components_from_xyz_d50(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // D50 参考白点
    let (xn, yn, zn) = (0.96422, 1.0, 0.82521);
    let eps = 216.0 / 24389.0; // (6/29)³
    let kappa = 24389.0 / 27.0; // (29/3)³
    let f = |t: f64| if t > eps { t.cbrt() } else { (kappa * t + 16.0) / 116.0 };
    let fx = f(x / xn);
    let fy = f(y / yn);
    let fz = f(z / zn);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    (l, a, b)
}

/// CIE LCH-D50 → sRGB u8（CSS Color 4；越界色逐通道钳制）。供 color-mix(in lch) 插值回转。
/// driving: css-color lch-009（lch(100% 110 60)→白）/ lch-010（lch(0% 110 60)→黑）。
pub fn lch_to_srgb_u8(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let l = l.clamp(0.0, 100.0);
    if l <= 0.0 {
        return (0, 0, 0);
    }
    if l >= 100.0 {
        return (255, 255, 255);
    }
    let h_rad = h.to_radians();
    let (lr, lg, lb) = lab_to_linear_srgb(l, c * h_rad.cos(), c * h_rad.sin());
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

/// OKLab → 线性 sRGB（CSS Color 4 §10.4，直接线性 LMS 矩阵）。
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

/// 线性 sRGB → OKLab（CSS Color 4 §10.4 逆变换：线性 sRGB → LMS → 立方根 → OKLab）。
/// 立方根保留负号（f64::cbrt），支持越界线性光。
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

/// sRGB u8 → OKLab（CSS Color 4）：L∈[0,1]、a/b。供 RCS oklab()/oklch() origin 解析。
pub fn srgb_u8_to_oklab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let lr = srgb_decode(r as f64 / 255.0);
    let lg = srgb_decode(g as f64 / 255.0);
    let lb = srgb_decode(b as f64 / 255.0);
    linear_srgb_to_oklab(lr, lg, lb)
}

/// OKLab → sRGB u8（CSS Color 4；越界色逐通道钳制）。供 RCS oklab() 输出回转。
/// oklch_to_srgb_u8 委托本函数，故 OKLCH 亦覆盖。
/// L 边界用 1e-4 容差（≈0.01%，视觉上即黑/白）：L 极接近 0/1 时与 L=0/1 渲染一致——
/// L=0/1 平面经 gamut 映射恒收敛到黑/白，故 oklab(0.0001% …)（L=1e-6）须与 oklab(0 …)
/// 同为黑（driving: oklab/oklch-l-almost-0/1）；其余逐通道钳制，byte-identical 于 R2323。
pub fn oklab_to_srgb_u8(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    const L_EPS: f64 = 1e-4;
    let l = l.clamp(0.0, 1.0);
    if l <= L_EPS {
        return (0, 0, 0);
    }
    if l >= 1.0 - L_EPS {
        return (255, 255, 255);
    }
    let (lr, lg, lb) = oklab_to_linear_srgb(l, a, b);
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

/// sRGB u8 → OKLCH（CSS Color 4）：L∈[0,1]、C、h∈[0,360)。供 RCS oklch() origin 解析。
pub fn srgb_u8_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (l, a, b) = srgb_u8_to_oklab(r, g, b);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h)
}

/// OKLCH → sRGB u8（CSS Color 4；L, C, h° → a=C·cos h, b=C·sin h → oklab 回转）。
pub fn oklch_to_srgb_u8(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let h_rad = h.to_radians();
    oklab_to_srgb_u8(l, c * h_rad.cos(), c * h_rad.sin())
}

/// sRGB 传递函数：分量 → 线性光（CSS Color 4，与 display-p3 共用）。
fn srgb_decode(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB 传递函数：线性光 → 分量（编码 gamma）。
fn srgb_encode(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// 线性 RGB 分量 → sRGB u8（编码 gamma + 钳制 + 四舍五入）。
fn linear_srgb_to_u8(c: f64) -> u8 {
    (srgb_encode(c) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// sRGB u8 → 线性光 sRGB 三通道（0..1），供 `color-mix(in srgb-linear)` 笛卡尔插值。R2377。
pub fn srgb_u8_to_srgb_linear(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    (
        srgb_decode(r as f64 / 255.0),
        srgb_decode(g as f64 / 255.0),
        srgb_decode(b as f64 / 255.0),
    )
}

/// 线性光 sRGB 三通道（0..1）→ sRGB u8，供 `color-mix(in srgb-linear)` 回转。R2377。
pub fn srgb_linear_to_srgb_u8(lr: f64, lg: f64, lb: f64) -> (u8, u8, u8) {
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

/// sRGB u8 → CIE XYZ-D65 三通道，供 `color-mix(in xyz)` 笛卡尔插值。R2378。
/// `xyz` ≡ `xyz-d65`（CSS Color 4）；`xyz-d50` 需 D65→D50 Bradford 适配，defer。
pub fn srgb_u8_to_xyz(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let lr = srgb_decode(r as f64 / 255.0);
    let lg = srgb_decode(g as f64 / 255.0);
    let lb = srgb_decode(b as f64 / 255.0);
    mat3_mul(SRGB_TO_XYZ, lr, lg, lb)
}

/// CIE XYZ-D65 三通道 → sRGB u8，供 `color-mix(in xyz)` 回转。R2378。
pub fn xyz_to_srgb_u8(x: f64, y: f64, z: f64) -> (u8, u8, u8) {
    let (lr, lg, lb) = mat3_mul(XYZ_TO_SRGB, x, y, z);
    (linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb))
}

/// 3×3 矩阵（行优先 [9]）乘列向量。
fn mat3_mul(m: [f64; 9], x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (
        m[0] * x + m[1] * y + m[2] * z,
        m[3] * x + m[4] * y + m[5] * z,
        m[6] * x + m[7] * y + m[8] * z,
    )
}

// display-p3（线性）→ XYZ-D65（CSS Color 4）
const P3_TO_XYZ: [f64; 9] = [
    0.4865709, 0.2656677, 0.1982173, 0.2289746, 0.6917385, 0.0792869, 0.0, 0.0451134, 1.0439444,
];
// a98-rgb（线性）→ XYZ-D65（CSS Color 4）
const A98_TO_XYZ: [f64; 9] = [
    0.5766690, 0.1855582, 0.1882286, 0.2973448, 0.6273624, 0.0752914, 0.0270313, 0.0706892, 0.9913375,
];
// rec2020（线性）→ XYZ-D65（CSS Color 4）
const REC2020_TO_XYZ: [f64; 9] = [
    0.6369580, 0.1446169, 0.1688809, 0.2627002, 0.6779981, 0.0593017, 0.0, 0.0280727, 1.0609851,
];
// prophoto-rgb（线性）→ XYZ-D50（CSS Color 4）
const PROPHOTO_TO_XYZ: [f64; 9] = [
    0.7977666, 0.1351813, 0.0313477, 0.2880747, 0.7118762, 0.0000853, 0.0, 0.0, 0.8251044,
];
// XYZ-D65 → 线性 sRGB（CSS Color 4）
const XYZ_TO_SRGB: [f64; 9] = [
    3.2409699, -1.5373832, -0.4986108, -0.9692436, 1.8759675, 0.0415551, 0.0556300, -0.2039770, 1.0569715,
];
// XYZ-D50 → XYZ-D65（Bradford 色度适应）
const XYZ_D50_TO_D65: [f64; 9] = [
    0.9555766, -0.0230393, 0.0631636, -0.0282895, 1.0099416, 0.0210077, 0.0122982, -0.0204830, 1.3299098,
];
// 线性 sRGB → XYZ-D65（CSS Color 4，XYZ_TO_SRGB 的逆）。driving: srgb_u8_to_lch（color-mix lch 插值）。
const SRGB_TO_XYZ: [f64; 9] = [
    0.4123908, 0.3575843, 0.1804808, 0.2126390, 0.7151687, 0.0721923, 0.0193308, 0.1191948, 0.9505322,
];
// XYZ-D65 → XYZ-D50（Bradford 适应逆，CSS Color 4）。
const XYZ_D65_TO_D50: [f64; 9] = [
    1.0478112, 0.0228866, -0.0501270, 0.0295424, 0.9904844, -0.0170491, -0.0092345, 0.0150436, 0.7521316,
];

/// 把预定义颜色空间 3 分量转换为 sRGB u8。返回 None 表示不支持的空间（rec2020/prophoto/未知）。
pub fn convert_predefined_to_srgb(space: &str, c0: f64, c1: f64, c2: f64) -> Option<(u8, u8, u8)> {
    let (lr, lg, lb) = match space {
        "srgb" => {
            // 已是 sRGB gamma 分量（0-1），直接转 u8。
            return Some((
                (c0 * 255.0).round().clamp(0.0, 255.0) as u8,
                (c1 * 255.0).round().clamp(0.0, 255.0) as u8,
                (c2 * 255.0).round().clamp(0.0, 255.0) as u8,
            ));
        }
        "srgb-linear" => (c0, c1, c2), // 已是线性 sRGB
        "display-p3" => {
            let (x, y, z) = mat3_mul(P3_TO_XYZ, srgb_decode(c0), srgb_decode(c1), srgb_decode(c2));
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        // CSS Color 4 线性光变体（分量已是线性，跳过 gamma 传递函数 decode）。
        // driving: css-color display-p3-linear-*（spec #valdef-color-display-p3-linear）。
        "display-p3-linear" => {
            let (x, y, z) = mat3_mul(P3_TO_XYZ, c0, c1, c2);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "a98-rgb" => {
            let g = 563.0 / 256.0; // a98 gamma ≈ 2.1992
            let (x, y, z) = mat3_mul(A98_TO_XYZ, safe_powf(c0, g), safe_powf(c1, g), safe_powf(c2, g));
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "a98-rgb-linear" => {
            let (x, y, z) = mat3_mul(A98_TO_XYZ, c0, c1, c2);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "rec2020" => {
            let (x, y, z) = mat3_mul(
                REC2020_TO_XYZ,
                rec2020_decode(c0),
                rec2020_decode(c1),
                rec2020_decode(c2),
            );
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "rec2020-linear" => {
            let (x, y, z) = mat3_mul(REC2020_TO_XYZ, c0, c1, c2);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "prophoto-rgb" => {
            // prophoto 矩阵到 XYZ-D50，须 Bradford 适应到 D65。
            let (x, y, z) = mat3_mul(
                PROPHOTO_TO_XYZ,
                prophoto_decode(c0),
                prophoto_decode(c1),
                prophoto_decode(c2),
            );
            let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "prophoto-rgb-linear" => {
            let (x, y, z) = mat3_mul(PROPHOTO_TO_XYZ, c0, c1, c2);
            let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, x, y, z);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        "xyz" | "xyz-d65" => mat3_mul(XYZ_TO_SRGB, c0, c1, c2),
        "xyz-d50" => {
            let (x, y, z) = mat3_mul(XYZ_D50_TO_D65, c0, c1, c2);
            mat3_mul(XYZ_TO_SRGB, x, y, z)
        }
        _ => return None,
    };
    Some((linear_srgb_to_u8(lr), linear_srgb_to_u8(lg), linear_srgb_to_u8(lb)))
}

/// 安全幂运算：负分量（越界/色域外）钳到 0（powf 对负数返回 NaN）。
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

/// prophoto-rgb 传递函数（分量 → 线性光，gamma 1.8 + 线性 toe）。
fn prophoto_decode(c: f64) -> f64 {
    if c < 0.03125 {
        (c / 16.0).max(0.0)
    } else {
        safe_powf(c, 1.8)
    }
}

/// 3×3 矩阵求逆（行列式 + 伴随矩阵法）。用于 color() RCS 的 sRGB→native 反向转换——
/// 数值化从正向矩阵推导，避免手算逆矩阵的精度/转写错误（driving: color() RCS 非 identity）。
fn mat3_inverse(m: [f64; 9]) -> [f64; 9] {
    let det =
        m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6]) + m[2] * (m[3] * m[7] - m[4] * m[6]);
    if det.abs() < 1e-15 {
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]; // 退化 → 单位阵（保守）
    }
    let inv_det = 1.0 / det;
    [
        (m[4] * m[8] - m[5] * m[7]) * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        (m[5] * m[6] - m[3] * m[8]) * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        (m[3] * m[7] - m[4] * m[6]) * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ]
}

/// a98-rgb 传递函数逆（线性光 → 分量）：正向 gamma 563/256，逆 = 256/563。
fn a98_encode(c: f64) -> f64 {
    safe_powf(c, 256.0 / 563.0)
}

/// BT.2020 传递函数逆（线性光 → 分量）；rec2020_decode 的分段逆。
fn rec2020_encode(lin: f64) -> f64 {
    const ALPHA: f64 = 1.09929682680944;
    const BETA: f64 = 0.018053968510807;
    if lin < BETA {
        lin * 4.5
    } else {
        ALPHA * safe_powf(lin, 0.45) - ALPHA + 1.0
    }
}

/// prophoto-rgb 传递函数逆（线性光 → 分量）；prophoto_decode 的分段逆。
/// toe 段边界：c=0.03125 → lin=0.03125/16=0.001953125。
fn prophoto_encode(lin: f64) -> f64 {
    if lin < 0.001953125 {
        lin * 16.0
    } else {
        safe_powf(lin, 1.0 / 1.8)
    }
}

/// sRGB u8 → 预定义色彩空间分量（color() RCS origin 解析）：返回 (c0,c1,c2)。
/// rect 空间返回 gamma/编码信号分量（0-1），xyz 空间返回 XYZ 分量。None = 未知空间。
/// 与 [`convert_predefined_to_srgb`] 互为逆运算（数值上经 `mat3_inverse` 从正向矩阵推导）。
pub fn srgb_u8_to_predefined(space: &str, r: u8, g: u8, b: u8) -> Option<(f64, f64, f64)> {
    let lr = srgb_decode(r as f64 / 255.0);
    let lg = srgb_decode(g as f64 / 255.0);
    let lb = srgb_decode(b as f64 / 255.0);
    let comps = match space {
        "srgb" => (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0),
        "srgb-linear" => (lr, lg, lb),
        "display-p3" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            let (c0, c1, c2) = mat3_mul(mat3_inverse(P3_TO_XYZ), x, y, z);
            (srgb_encode(c0), srgb_encode(c1), srgb_encode(c2))
        }
        "display-p3-linear" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            mat3_mul(mat3_inverse(P3_TO_XYZ), x, y, z)
        }
        "a98-rgb" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            let (c0, c1, c2) = mat3_mul(mat3_inverse(A98_TO_XYZ), x, y, z);
            (a98_encode(c0), a98_encode(c1), a98_encode(c2))
        }
        "a98-rgb-linear" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            mat3_mul(mat3_inverse(A98_TO_XYZ), x, y, z)
        }
        "rec2020" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            let (c0, c1, c2) = mat3_mul(mat3_inverse(REC2020_TO_XYZ), x, y, z);
            (rec2020_encode(c0), rec2020_encode(c1), rec2020_encode(c2))
        }
        "rec2020-linear" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            mat3_mul(mat3_inverse(REC2020_TO_XYZ), x, y, z)
        }
        "prophoto-rgb" => {
            // prophoto 矩阵定义于 XYZ-D50：线性 sRGB → XYZ-D65 → XYZ-D50 → prophoto 线性 → encode。
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            let (x, y, z) = mat3_mul(XYZ_D65_TO_D50, x, y, z);
            let (c0, c1, c2) = mat3_mul(mat3_inverse(PROPHOTO_TO_XYZ), x, y, z);
            (prophoto_encode(c0), prophoto_encode(c1), prophoto_encode(c2))
        }
        "prophoto-rgb-linear" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            let (x, y, z) = mat3_mul(XYZ_D65_TO_D50, x, y, z);
            mat3_mul(mat3_inverse(PROPHOTO_TO_XYZ), x, y, z)
        }
        "xyz" | "xyz-d65" => mat3_mul(SRGB_TO_XYZ, lr, lg, lb),
        "xyz-d50" => {
            let (x, y, z) = mat3_mul(SRGB_TO_XYZ, lr, lg, lb);
            mat3_mul(XYZ_D65_TO_D50, x, y, z)
        }
        _ => return None,
    };
    Some(comps)
}
