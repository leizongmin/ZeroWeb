//! R5191：transform 函数列表插值（css-transforms-1 §13.2 #decomposing-a-2d-matrix /
//! css-transforms-2 §14 #interpolation-of-transforms）。
//!
//! 插值策略（与 [`crate::animation::interpolate_property_value`] 的 filter 臂同型）：
//! - 同长同序同函数逐参数线性插值（仅 2D 函数面）；
//! - matrix ↔ matrix 走规范 2D unmatrix 分解（translation/scale/angle + 残差 2×2
//!   各分量 lerp 后按 R(θ)·S·M 重组），非可逆矩阵无法分解 → 离散；
//! - `none` 一侧展开为对侧函数序列的恒等对应；
//! - 3D 函数（translate3d/scaleZ/scale3d/rotateX/rotateY/rotate3d/perspective/
//!   matrix3d）不插值——ZW 无 3D 插值语义，回落离散切换（既有默认臂同语义，零回归）；
//! - 异构列表（长度/函数种类不匹配）→ 离散。

use zero_css_parser::values::{TransformFunction as Tf, TransformValue, parse_transform};

use crate::js_dom_bridge::computed_style::transform_to_css;

/// 插值 `transform` 属性两个计算值字符串，返回插值结果的序列化形态；返回 `None`
/// 表示不可插值（调用方按 `t > 0.5` 离散切换，css-transforms-2 同款回退）。
pub(crate) fn interpolate_transform_value(from: &str, to: &str, t: f64) -> Option<String> {
    let from_v = parse_transform(from)?;
    let to_v = parse_transform(to)?;
    match (from_v, to_v) {
        (TransformValue::None, TransformValue::None) => Some("none".to_string()),
        // none ↔ 列表：none 展开为对侧函数序列的恒等对应（css-transforms-2
        // #interpolation-of-transforms：none 与列表插值按恒等函数补齐）。
        (TransformValue::None, TransformValue::List(tl)) => {
            let il = identity_list(&tl)?;
            let out = lerp_lists(&il, &tl, t)?;
            Some(transform_to_css(&out))
        }
        (TransformValue::List(fl), TransformValue::None) => {
            let il = identity_list(&fl)?;
            let out = lerp_lists(&fl, &il, t)?;
            Some(transform_to_css(&out))
        }
        (TransformValue::List(fl), TransformValue::List(tl)) => {
            let out = lerp_lists(&fl, &tl, t)?;
            Some(transform_to_css(&out))
        }
    }
}

/// 对侧函数序列的恒等对应列表；含 3D 函数（无恒等插值语义）→ `None`。
fn identity_list(fns: &[Tf]) -> Option<Vec<Tf>> {
    fns.iter()
        .map(|f| {
            Some(match f {
                Tf::Translate(_, _) => Tf::Translate(0.0, 0.0),
                Tf::TranslateMixed(_, txp, _, typ) => Tf::TranslateMixed(0.0, *txp, 0.0, *typ),
                Tf::TranslateXMixed(_, pct) => Tf::TranslateXMixed(0.0, *pct),
                Tf::TranslateYMixed(_, pct) => Tf::TranslateYMixed(0.0, *pct),
                Tf::TranslateX(_) => Tf::TranslateX(0.0),
                Tf::TranslateY(_) => Tf::TranslateY(0.0),
                Tf::Rotate(_) => Tf::Rotate(0.0),
                Tf::RotateZ(_) => Tf::RotateZ(0.0),
                Tf::Scale(_, sy) => Tf::Scale(1.0, sy.map(|_| 1.0)),
                Tf::ScaleX(_) => Tf::ScaleX(1.0),
                Tf::ScaleY(_) => Tf::ScaleY(1.0),
                Tf::Skew(_, ay) => Tf::Skew(0.0, ay.map(|_| 0.0)),
                Tf::Matrix(..) => Tf::Matrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                // 3D 函数：ZW 无 3D 插值语义（css-transforms-2 域挂账）→ 离散。
                Tf::Translate3d(..)
                | Tf::ScaleZ(_)
                | Tf::Scale3d(..)
                | Tf::RotateX(_)
                | Tf::RotateY(_)
                | Tf::Rotate3d(..)
                | Tf::Perspective(_)
                | Tf::Matrix3d(_) => return None,
            })
        })
        .collect()
}

/// 同长同序同函数逐参数插值；不满足 → `None`（离散回退）。
fn lerp_lists(fl: &[Tf], tl: &[Tf], t: f64) -> Option<TransformValue> {
    if fl.len() != tl.len() {
        return None;
    }
    let out: Option<Vec<Tf>> = fl.iter().zip(tl.iter()).map(|(f, g)| lerp_function(f, g, t)).collect();
    Some(TransformValue::List(out?))
}

/// 同种函数逐参数插值；异种/3D → `None`。
fn lerp_function(f: &Tf, g: &Tf, t: f64) -> Option<Tf> {
    match (f, g) {
        (Tf::Translate(ax, ay), Tf::Translate(bx, by)) => Some(Tf::Translate(
            crate::animation::lerp(*ax, *bx, t),
            crate::animation::lerp(*ay, *by, t),
        )),
        // 混合百分比：逐分量同单位（%↔% / px↔px）数值插值（同一盒子参照下线性
        // 等价）；单位不一致 → 离散（跨单位插值须 layout 长度，动画层无盒信息）。
        (Tf::TranslateMixed(ax, ap, ay, ayp), Tf::TranslateMixed(bx, bp, by, byp)) => {
            if ap != bp || ayp != byp {
                return None;
            }
            Some(Tf::TranslateMixed(
                crate::animation::lerp(*ax, *bx, t),
                *ap,
                crate::animation::lerp(*ay, *by, t),
                *ayp,
            ))
        }
        (Tf::TranslateXMixed(av, ap), Tf::TranslateXMixed(bv, bp)) => {
            if ap != bp {
                return None;
            }
            Some(Tf::TranslateXMixed(crate::animation::lerp(*av, *bv, t), *ap))
        }
        (Tf::TranslateYMixed(av, ap), Tf::TranslateYMixed(bv, bp)) => {
            if ap != bp {
                return None;
            }
            Some(Tf::TranslateYMixed(crate::animation::lerp(*av, *bv, t), *ap))
        }
        (Tf::TranslateX(a), Tf::TranslateX(b)) => Some(Tf::TranslateX(crate::animation::lerp(*a, *b, t))),
        (Tf::TranslateY(a), Tf::TranslateY(b)) => Some(Tf::TranslateY(crate::animation::lerp(*a, *b, t))),
        (Tf::Rotate(a), Tf::Rotate(b)) => Some(Tf::Rotate(crate::animation::lerp(*a, *b, t))),
        (Tf::RotateZ(a), Tf::RotateZ(b)) => Some(Tf::RotateZ(crate::animation::lerp(*a, *b, t))),
        // scale(sx) ≡ scale(sx, sx)（css-transforms-1 #funcdef-scale），None 先归一。
        (Tf::Scale(ax, ay), Tf::Scale(bx, by)) => {
            let ay = ay.unwrap_or(*ax);
            let by = by.unwrap_or(*bx);
            let sx = crate::animation::lerp(*ax, *bx, t);
            let sy = crate::animation::lerp(ay, by, t);
            if (sx - sy).abs() < 1e-9 {
                Some(Tf::Scale(sx, None))
            } else {
                Some(Tf::Scale(sx, Some(sy)))
            }
        }
        (Tf::ScaleX(a), Tf::ScaleX(b)) => Some(Tf::ScaleX(crate::animation::lerp(*a, *b, t))),
        (Tf::ScaleY(a), Tf::ScaleY(b)) => Some(Tf::ScaleY(crate::animation::lerp(*a, *b, t))),
        // skew(ax) ≡ skew(ax, 0)（css-transforms-1 #funcdef-skew），None 归一为 0。
        (Tf::Skew(ax, ay), Tf::Skew(bx, by)) => {
            let ay = ay.unwrap_or(0.0);
            let by = by.unwrap_or(0.0);
            let rx = crate::animation::lerp(*ax, *bx, t);
            let ry = crate::animation::lerp(ay, by, t);
            if ry.abs() < 1e-9 {
                Some(Tf::Skew(rx, None))
            } else {
                Some(Tf::Skew(rx, Some(ry)))
            }
        }
        (Tf::Matrix(a1, a2, a3, a4, a5, a6), Tf::Matrix(b1, b2, b3, b4, b5, b6)) => Some(lerp_matrix(
            *a1, *a2, *a3, *a4, *a5, *a6, *b1, *b2, *b3, *b4, *b5, *b6, t,
        )?),
        // 异种/3D 函数：离散回退（零回归——既有默认臂同语义）。
        _ => None,
    }
}

/// matrix ↔ matrix 规范 2D 分解插值（css-transforms-1 §13.2：#decomposing-a-2d-matrix、
/// #interpolation-of-decomposed-2d-matrix-values、#recomposing-to-a-2d-matrix）。
/// 分解前提 = 矩阵可被 R(θ)·S·M（残差恒等）无损重构——纯 rotate/scale/translate 组合
/// 恰好残差恒等；含 shear（斜切耦合进矩阵）时规范分解对重构不可逆（端点无法还原），
/// 拒绝插值 → `None`（离散，保持端点语义）。非可逆（轴长 0）同拒。
#[allow(clippy::too_many_arguments)]
fn lerp_matrix(
    aa: f64,
    ab: f64,
    ac: f64,
    ad: f64,
    ae: f64,
    af: f64, //
    ba: f64,
    bb: f64,
    bc: f64,
    bd: f64,
    be: f64,
    bf: f64,
    t: f64,
) -> Option<Tf> {
    let (mut ar0x, mut ar0y, mut ar1x, mut ar1y) = (aa, ab, ac, ad);
    let (atx, aty) = (ae, af);
    let (mut s0a, mut s1a) = (ar0x.hypot(ar0y), ar1x.hypot(ar1y));
    // 行列式 < 0 → 镜像：翻转单位向量点积较小的轴（规范 flip 步骤）。
    if ar0x * ar1y - ar0y * ar1x < 0.0 {
        if ar0x < ar1y {
            s0a = -s0a;
        } else {
            s1a = -s1a;
        }
    }
    if s0a == 0.0 || s1a == 0.0 {
        return None;
    }
    ar0x /= s0a;
    ar0y /= s0a;
    ar1x /= s1a;
    ar1y /= s1a;
    let mut angle_a = ar0y.atan2(ar0x);
    let (am11, am12, am21, am22) = unrotate_rows(ar0x, ar0y, ar1x, ar1y, &mut angle_a);
    // 残差恒等守卫：shear 耦合使残差偏离恒等（分解不可逆）→ 离散。
    if (am11 - 1.0).abs() > 1e-9 || am12.abs() > 1e-9 || am21.abs() > 1e-9 || (am22 - 1.0).abs() > 1e-9 {
        return None;
    }

    let (mut br0x, mut br0y, mut br1x, mut br1y) = (ba, bb, bc, bd);
    let (btx, bty) = (be, bf);
    let (mut s0b, mut s1b) = (br0x.hypot(br0y), br1x.hypot(br1y));
    if br0x * br1y - br0y * br1x < 0.0 {
        if br0x < br1y {
            s0b = -s0b;
        } else {
            s1b = -s1b;
        }
    }
    if s0b == 0.0 || s1b == 0.0 {
        return None;
    }
    br0x /= s0b;
    br0y /= s0b;
    br1x /= s1b;
    br1y /= s1b;
    let mut angle_b = br0y.atan2(br0x);
    let (bm11, bm12, bm21, bm22) = unrotate_rows(br0x, br0y, br1x, br1y, &mut angle_b);
    if (bm11 - 1.0).abs() > 1e-9 || bm12.abs() > 1e-9 || bm21.abs() > 1e-9 || (bm22 - 1.0).abs() > 1e-9 {
        return None;
    }

    // 插值预处理（规范）：x/y 轴交叉翻转 → 转为未翻转旋转；0° 归一为 360° 后
    // 走最短路径。
    if (s0a < 0.0 && s1b < 0.0) || (s1a < 0.0 && s0b < 0.0) {
        s0a = -s0a;
        s1a = -s1a;
        angle_a += if angle_a < 0.0 { 180.0 } else { -180.0 };
    }
    if angle_a == 0.0 {
        angle_a = 360.0;
    }
    if angle_b == 0.0 {
        angle_b = 360.0;
    }
    if (angle_a - angle_b).abs() > 180.0 {
        if angle_a > angle_b {
            angle_a -= 360.0;
        } else {
            angle_b -= 360.0;
        }
    }

    // 各分量线性插值（残差恒等守卫下 m11..m22 恒为 (1,0,0,1)，重组退化为
    // R(angle)·diag(s0, s1)）。
    let s0 = crate::animation::lerp(s0a, s0b, t);
    let s1 = crate::animation::lerp(s1a, s1b, t);
    let angle = crate::animation::lerp(angle_a, angle_b, t);
    let tx = crate::animation::lerp(atx, btx, t);
    let ty = crate::animation::lerp(aty, bty, t);

    // 重组（#recomposing-to-a-2d-matrix）：线性部分 = M·R(angle)·S（规范 post-multiply
    // 语义；残差恒等守卫下 M = I，即 R(angle)·diag(s0, s1)），平移列 = M·t = t
    //（R/S post-multiply 不触碰平移列）。|sin|<1e-9 吸收 360°/0° 三角函数舍入尾差，
    // 避免序列化出现 1e-16 级噪声（渲染不可见，仅数值整洁）。
    let rad = angle.to_radians();
    let (cs, sn) = {
        let (c, s) = (rad.cos(), rad.sin());
        (
            if c.abs() < 1e-9 { 0.0 } else { c },
            if s.abs() < 1e-9 { 0.0 } else { s },
        )
    };
    let a = cs * s0;
    let b = sn * s0;
    let c = -sn * s1;
    let d = cs * s1;
    Some(Tf::Matrix(a, b, c, d, tx, ty))
}

/// 去旋转：按 -angle 旋转行对（规范 unrotate 步骤），返回残差 (m11, m12, m21, m22)，
/// 并把 angle 转为度。angle 为 0 时残差 = 当前行对（规范 if(angle) 跳过）。
fn unrotate_rows(r0x: f64, r0y: f64, r1x: f64, r1y: f64, angle: &mut f64) -> (f64, f64, f64, f64) {
    if *angle != 0.0 {
        let sn = -r0y;
        let cs = r0x;
        let (m11, m12, m21, m22) = (r0x, r0y, r1x, r1y);
        let n0x = cs * m11 + sn * m21;
        let n0y = cs * m12 + sn * m22;
        let n1x = -sn * m11 + cs * m21;
        let n1y = -sn * m12 + cs * m22;
        *angle = angle.to_degrees();
        (n0x, n0y, n1x, n1y)
    } else {
        (r0x, r0y, r1x, r1y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interp(from: &str, to: &str, t: f64) -> Option<String> {
        interpolate_transform_value(from, to, t)
    }

    #[test]
    fn test_matrix_scale_midpoint() {
        // transform-interpolation-matrix 驱动案形：matrix scale 1→2 中点 1.5。
        let out = interp("matrix(1,0,0,1,0,0)", "matrix(2,0,0,2,0,0)", 0.5).expect("可插值");
        assert!(out.contains("matrix(1.5, 0, 0, 1.5, 0, 0)"), "got {out}");
    }

    #[test]
    fn test_matrix_rotation_halfway_quarter_turn() {
        // identity → matrix(-1,0,0,-1)（= rotate 180°）：规范 0→360 归一后中点 270°
        //（与 -90° 同渲染），残差恒等、行列式 1。
        let out = interp("matrix(1,0,0,1,0,0)", "matrix(-1,0,0,-1,0,0)", 0.5).expect("可插值");
        let v = parse_transform(&out).unwrap();
        let TransformValue::List(fns) = v else {
            panic!("expected list, got {v:?}")
        };
        let Tf::Matrix(a, b, c, d, _, _) = fns[0] else {
            panic!("expected matrix, got {:?}", fns[0])
        };
        assert!((a.abs() - 0.0) < 1e-9 && (d.abs() - 0.0) < 1e-9, "四分之一转 {out}");
        assert!((b * c + 1.0).abs() < 1e-9, "行列式分量积 -1（旋转非镜像）{out}");
    }

    #[test]
    fn test_rotate_function_lerp() {
        let out = interp("rotate(0deg)", "rotate(360deg)", 0.5).expect("可插值");
        assert_eq!(out, "rotate(180deg)");
    }

    #[test]
    fn test_none_expands_to_identity() {
        let out = interp("none", "rotate(180deg)", 0.5).expect("可插值");
        assert_eq!(out, "rotate(90deg)");
        let out = interp("translate(100px, 40px)", "none", 0.5).expect("可插值");
        assert_eq!(out, "translate(50px, 20px)");
    }

    #[test]
    fn test_function_list_pairwise() {
        // 驱动案全形：matrix(...) rotate(...) → matrix(...) rotate(...) 同构列表。
        let out = interp(
            "matrix(1,0,0,1,0,0) rotate(0deg)",
            "matrix(2,0,0,2,0,0) rotate(360deg)",
            0.5,
        )
        .expect("可插值");
        assert!(out.contains("matrix(1.5, 0, 0, 1.5, 0, 0)"), "got {out}");
        assert!(out.contains("rotate(180deg)"), "got {out}");
    }

    #[test]
    fn test_mismatched_lists_discrete() {
        // 长度不同 / 函数异种 → None（调用方离散切换）。
        assert_eq!(interp("translate(10px)", "rotate(90deg)", 0.5), None);
        assert_eq!(interp("translate(10px)", "translate(10px) scale(2)", 0.5), None);
    }

    #[test]
    fn test_3d_functions_discrete() {
        // 3D 函数不插值（零回归）。
        assert_eq!(interp("translateZ(10px)", "translateZ(20px)", 0.5), None);
        assert_eq!(
            interp("matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)", "none", 0.5),
            None
        );
    }

    #[test]
    fn test_invalid_input_discrete() {
        assert_eq!(interp("garbage(", "rotate(90deg)", 0.5), None);
    }

    #[test]
    fn test_endpoints_round_trip() {
        // rotate(30°)·scale(1.5,2)·translate(15,25) 组合（残差恒等）：端点插值
        // 序列化后与原值同语义（format_num 三位小数舍入内）。
        let from = "matrix(1.299038105676658,0.75,-1,1.7320508075688774,-5.514428414850132,54.551270189221935)";
        let to = "matrix(2,0,0,2,0,0)";
        assert_eq!(interp(from, to, 0.0).unwrap(), normalize(from));
        assert_eq!(interp(from, to, 1.0).unwrap(), normalize(to));
        // 含 shear 的矩阵（列非正交，分解不可逆）→ 离散，端点语义不被破坏。
        assert_eq!(interp("matrix(1,0.5,0,1,0,0)", "matrix(2,0,0,2,0,0)", 0.5), None);
    }

    /// 端点经插值序列化后与原值同语义（规范化空白）。
    fn normalize(s: &str) -> String {
        let v = parse_transform(s).unwrap();
        transform_to_css(&v)
    }
}
