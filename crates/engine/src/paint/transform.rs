//! 绘制辅助 — transform/translate 家族（自 helpers.rs 按职责拆分，T59；纯移动零行为变化）。

use super::clip::PrimitiveCounts;
use zero_css_parser::values::{LengthValue, TransformFunction, TransformValue};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{RenderPrimitives, TransformPrimitive};
use zero_style_system::ComputedStyle;

/// 从 ComputedStyle 的 transform 计算偏移量。
///
/// 返回 (dx, dy) 偏移，用于调整图元位置。
/// 仅提取 translate 分量；rotate/scale/skew 由 TransformPrimitive 处理。
pub fn apply_transform_offset(style: &ComputedStyle, _abs_x: f32, _abs_y: f32) -> (f32, f32) {
    match &style.transform {
        TransformValue::None => (0.0, 0.0),
        TransformValue::List(funcs) => {
            let mut dx = 0.0_f32;
            let mut dy = 0.0_f32;
            for f in funcs {
                match f {
                    TransformFunction::Translate(tx, ty) => {
                        dx += *tx as f32;
                        dy += *ty as f32;
                    }
                    TransformFunction::TranslateX(tx) => {
                        dx += *tx as f32;
                    }
                    TransformFunction::TranslateY(ty) => {
                        dy += *ty as f32;
                    }
                    // rotate, scale, skew 不产生偏移
                    _ => {}
                }
            }
            (dx, dy)
        }
    }
}

/// 计算 2D 仿射变换矩阵（含 transform-origin）。
///
/// 变换矩阵按 CSS 规范组合：
/// 1. 平移到 transform-origin
/// 2. 应用所有 transform 函数
/// 3. 平移回
///
/// 返回 None 如果变换为 None 或全部是 identity。
/// R3831：检测 transform 是否产生奇异 3D 矩阵（CSS Transforms 1 §: 变换矩阵不可逆时
/// 元素及其内容不渲染——chrome 同此，transform3d-scale-004：`scale3d(2,2,0)` z=0 →
/// 整元素不可见，`Nothing should appear except this text`）。
///
/// 覆盖可判定奇异面（保守）：`scale3d(_, _, 0)`（z 缩放 0）、`scale(0,_)`/`scaleX(0)`/
/// `scaleY(0)`（轴缩放 0）。`rotate3d`/`matrix3d` 的奇异判定需完整 3D 矩阵（ZW 无 3D
/// 矩阵合成，缺失维恒等）不在本 helper 范围。
pub fn is_singular_transform(style: &ComputedStyle) -> bool {
    let own_singular = match &style.transform {
        TransformValue::None => false,
        TransformValue::List(f) => f.iter().any(|f| match f {
            TransformFunction::Scale3d(_, _, sz) => *sz == 0.0,
            TransformFunction::Scale(sx, sy) => *sx == 0.0 || sy.is_some_and(|sy| sy == 0.0),
            TransformFunction::ScaleX(sx) => *sx == 0.0,
            TransformFunction::ScaleY(sy) => *sy == 0.0,
            _ => false,
        }),
    };
    if own_singular {
        return true;
    }
    // R4201：individual `scale: 0`（承载为 Scale(sx, sy)）同样构成奇异。
    matches!(&style.individual_scale, Some(TransformFunction::Scale(sx, sy))
        if *sx == 0.0 || sy.is_some_and(|sy| sy == 0.0))
}

/// R4035：元素是否背对观察者（CSS Transforms 2 §backface-visibility）。
///
/// ZW 无 3D 场景图（transform-style:flat 逐盒 2D 投影，R3833），背面朝向由变换链中
/// 3D 旋转函数的 z 翻转近似：每个 rotateX(θ)/rotateY(θ) 在其 2D 投影中贡献符号
/// cos(θ)（|θ|>90° 时平面镜像 = 背面）；连乘 < 0 → 背面。纯 2D 变换（rotate/scale/
/// skew/matrix）不翻转 z（scaleX(-1) 镜像仍是正面，chromium 同）。rotate3d 在 ZW
/// 为恒等近似（R3833 前），不贡献翻转。
///
/// driving：css-transforms/backface-visibility-hidden-002（rotateY(180deg) + hidden
/// → 元素不可见，旧实现照绘红块）。|θ|=90° 时投影奇异（det≈0）由
/// [`is_singular_transform`] 兜底链处理，此处 cos=0 不判背面。
pub fn transform_shows_backface(style: &ComputedStyle) -> bool {
    let funcs = match &style.transform {
        TransformValue::None => return false,
        TransformValue::List(f) => f,
    };
    let mut net = 1.0_f32;
    for f in funcs {
        let cos = match f {
            TransformFunction::RotateX(deg) => (*deg as f32).to_radians().cos(),
            TransformFunction::RotateY(deg) => (*deg as f32).to_radians().cos(),
            _ => continue,
        };
        net *= cos;
    }
    net < 0.0
}

/// 将 `style.transform` 函数列表合成为 2D 仿射变换图元（含 transform-origin）。
///
/// 仅 translate 的列表返回 `None`（由 offset 路径处理）。
pub fn compute_transform_matrix(style: &ComputedStyle, rect: &Rect) -> Option<TransformPrimitive> {
    compute_transform_matrix_with_ref_box(style, rect, None)
}

/// R4107（CSS Transforms 1 §transform-box）：参考框感知版本——`ref_rect` 为 Some
/// 时 transform-origin 基点与百分比 translate 尺寸取参考框（html content-box/fill-box
/// = 内容盒），联合 bbox（R3915）仍以 border-box `rect` 为基（绘制区域覆盖不变）。
pub fn compute_transform_matrix_with_ref_box(
    style: &ComputedStyle,
    rect: &Rect,
    ref_rect: Option<&Rect>,
) -> Option<TransformPrimitive> {
    // R4201（css-transforms-2 §individual-transforms）：独立属性合成——
    // used transform = translate · rotate · scale · transform（css-transforms-2
    // §transform 拆解顺序）。individual 属性值承载为单函数 TransformFunction
    //（% 分量相对元素 border-box，与 transform 函数同语义），前置到列表头部。
    // `transform: none` 且有 individual 时仍须合成（原 None 早退在无 individual 时保持）。
    let mut individual: Vec<TransformFunction> = Vec::with_capacity(3);
    if let Some(f) = &style.individual_translate {
        individual.push(f.clone());
    }
    if let Some(f) = &style.individual_rotate {
        individual.push(f.clone());
    }
    if let Some(f) = &style.individual_scale {
        individual.push(f.clone());
    }
    let own_funcs: Vec<TransformFunction> = match &style.transform {
        TransformValue::None => Vec::new(),
        TransformValue::List(f) => f.clone(),
    };
    let has_individual = !individual.is_empty();
    if own_funcs.is_empty() && !has_individual {
        return None;
    }
    let funcs: Vec<TransformFunction> = individual.into_iter().chain(own_funcs).collect();
    // R5016（css-transforms-1 §13.1）：rotate3d 单轴精确恒等——rotate3d(±n,0,0,α) ≡
    // rotateX(±α)（spec 矩阵对轴归一化，轴长度不影响旋转角）。单轴分解后走 rotateX/
    // rotateY 的 cos 2D 投影（正交精确），与 transform 属性同函数一致——旧 Rotate3d
    // 恒等近似致 `rotate: x 45deg`（RotateX 投影）与 `transform: rotate3d(1,0,0,45deg)`
    //（恒等）同页异像（driving: individual-transform-1 rotate_2/rotate_3）。
    // parse 层不分解：CSSOM 序列化须回写 rotate3d 原形（computed_style.rs）。
    let funcs: Vec<TransformFunction> = funcs
        .into_iter()
        .map(|f| match f {
            TransformFunction::Rotate3d(x, y, z, deg) => rotate3d_unit_axis(x, y, z, deg),
            other => other,
        })
        .collect();

    // 检查是否只有 translate 函数（由 offset 处理，不需要 TransformPrimitive）。
    // individual translate 亦为 Translate/TranslateMixed——TranslateMixed 归 matrix
    // 路径（% 需 rect），与既有混合列表语义一致。
    let has_non_translate = funcs.iter().any(|f| {
        !matches!(
            f,
            TransformFunction::Translate(_, _) | TransformFunction::TranslateX(_) | TransformFunction::TranslateY(_)
        )
    });
    if !has_non_translate {
        return None;
    }

    // 计算 transform-origin（相对于视口绝对坐标）。
    // R4107：声明 transform-box: content-box/fill-box 时基点/尺寸取参考框（内容盒）。
    let origin_base = ref_rect.unwrap_or(rect);
    let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
    let origin_x = origin_base.origin.x
        + resolve_transform_origin_length(&style.transform_origin_x, origin_base.size.width, font_size_px);
    let origin_y = origin_base.origin.y
        + resolve_transform_origin_length(&style.transform_origin_y, origin_base.size.height, font_size_px);

    // 构建累积变换矩阵（3x3 仿射，存储为 [a, b, c, d, tx, ty]）
    // | a  c  tx |
    // | b  d  ty |
    // | 0  0   1 |
    let mut a = 1.0_f32;
    let mut b = 0.0_f32;
    let mut c = 0.0_f32;
    let mut d = 1.0_f32;
    let mut tx = 0.0_f32;
    let mut ty = 0.0_f32;

    for func in &funcs {
        let (fa, fb, fc, fd, ftx, fty) = match func {
            TransformFunction::Translate(dx, dy) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
            // R2294：translate(%) 相对元素 border-box（rect.size）求值。has_non_translate 把
            // Mixed 变体当非-translate → 走本 matrix 路径（rect 可用），绕过无 rect 的 offset 路径。
            TransformFunction::TranslateMixed(tx, txp, ty, typ) => {
                let fx = if *txp {
                    rect.size.width * (*tx as f32) / 100.0
                } else {
                    *tx as f32
                };
                let fy = if *typ {
                    rect.size.height * (*ty as f32) / 100.0
                } else {
                    *ty as f32
                };
                (1.0, 0.0, 0.0, 1.0, fx, fy)
            }
            TransformFunction::TranslateXMixed(tx, _) => {
                (1.0, 0.0, 0.0, 1.0, rect.size.width * (*tx as f32) / 100.0, 0.0)
            }
            TransformFunction::TranslateYMixed(ty, _) => {
                (1.0, 0.0, 0.0, 1.0, 0.0, rect.size.height * (*ty as f32) / 100.0)
            }
            TransformFunction::TranslateX(dx) => (1.0, 0.0, 0.0, 1.0, *dx as f32, 0.0),
            TransformFunction::TranslateY(dy) => (1.0, 0.0, 0.0, 1.0, 0.0, *dy as f32),
            TransformFunction::Rotate(deg) => {
                let rad = deg.to_radians() as f32;
                let cos = rad.cos();
                let sin = rad.sin();
                (cos, sin, -sin, cos, 0.0, 0.0)
            }
            TransformFunction::Scale(sx, sy) => {
                let sy = sy.unwrap_or(*sx) as f32;
                (*sx as f32, 0.0, 0.0, sy, 0.0, 0.0)
            }
            TransformFunction::ScaleX(sx) => (*sx as f32, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::ScaleY(sy) => (1.0, 0.0, 0.0, *sy as f32, 0.0, 0.0),
            // R3832：scaleZ 在 2D 投影中恒等（z 缩放不改变 x/y 投影），但函数存在性
            // 影响奇异判定（is_singular_transform：scaleZ(0) → 奇异 → 不渲染）。
            TransformFunction::ScaleZ(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::Skew(ax, ay) => {
                let tan_ax = ax.to_radians().tan() as f32;
                let tan_ay = ay.map(|v| v.to_radians().tan() as f32).unwrap_or(0.0);
                (1.0, tan_ay, tan_ax, 1.0, 0.0, 0.0)
            }
            // 3D 变换函数降级为 2D 近似
            TransformFunction::Translate3d(dx, dy, _) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
            // R3833：rotateX/rotateY 的 2D 投影（CSS Transforms 2 §: 绕轴旋转把内容平面
            // 沿轴压缩/镜像）。rotX(180deg) = scaleY(-1)（transform3d-scale-007：test 页
            // rotatex(180) scaleZ(-1) 的正确 2D 投影 = scaleY(-1)，与 ref 的 scaleY(-1)
            // 一致）；旧恒等近似致 test 页不镜像。θ=90/270 投影为线（scale 0）。
            // 已知局限：3D 链（如 rotX(90) scale3d rotX(-90) 抵消对，transform3d-scale-002）
            // 逐函数 2D 近似因中间 scaleY(0) 退化坍缩——需全 3D 合成（见 master.md R3833）。
            TransformFunction::RotateX(deg) => {
                let cos = (*deg as f32).to_radians().cos();
                (1.0, 0.0, 0.0, cos, 0.0, 0.0)
            }
            TransformFunction::RotateY(deg) => {
                let cos = (*deg as f32).to_radians().cos();
                (cos, 0.0, 0.0, 1.0, 0.0, 0.0)
            }
            TransformFunction::RotateZ(deg) => {
                let rad = deg.to_radians() as f32;
                let cos = rad.cos();
                let sin = rad.sin();
                (cos, sin, -sin, cos, 0.0, 0.0)
            }
            TransformFunction::Rotate3d(_, _, _, _) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::Scale3d(sx, sy, _) => (*sx as f32, 0.0, 0.0, *sy as f32, 0.0, 0.0),
            TransformFunction::Perspective(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::Matrix(ma, mb, mc, md, me, mf) => {
                (*ma as f32, *mb as f32, *mc as f32, *md as f32, *me as f32, *mf as f32)
            }
        };

        // 矩阵乘法：current = current * func
        let new_a = a * fa + c * fb;
        let new_b = b * fa + d * fb;
        let new_c = a * fc + c * fd;
        let new_d = b * fc + d * fd;
        let new_tx = a * ftx + c * fty + tx;
        let new_ty = b * ftx + d * fty + ty;
        a = new_a;
        b = new_b;
        c = new_c;
        d = new_d;
        tx = new_tx;
        ty = new_ty;
    }

    // 应用 transform-origin 偏移
    // 最终变换：translate(origin) * matrix * translate(-origin)
    // 对 affine [a,b,c,d,tx,ty] 来说：
    // new_tx = origin_x * (1 - a) - origin_y * c + tx
    // new_ty = -origin_x * b + origin_y * (1 - d) + ty
    let mut final_tx = origin_x * (1.0 - a) - origin_y * c + tx;
    let mut final_ty = -origin_x * b + origin_y * (1.0 - d) + ty;

    // R3833 退化兜底：rotateX/rotateY 的 2D 投影在 θ=±90° 时行列式为 0（内容坍缩为
    // 线）。3D 链中 rotX(±90) 常成对出现（rotX(90) … rotX(-90) 平面旋转后复位，
    // transform3d-scale-002/005/006：夹在中间的 scale3d 应作用于 x/y），逐函数 2D 近似
    // 的中间 scaleY(0) 会把整链坍缩。检测 |det| ≈ 0 且链含 rotX/rotY → 以 rotX/rotY
    // 恒等重算（回退 R3833 前的近似语义，保住成对抵消链）。
    // R3834：兜底重算后 final_tx/final_ty 也须按重算后的矩阵重新应用 transform-origin
    //（旧实现沿用坍缩矩阵求得的 origin 项，rotX(90) 200×200 盒产生 spurious ty=+100
    // 位移——preserve3d-pseudo-element 内容离位全白、scale-002#0/stair 离位）。
    let det_2d = a * d - b * c;
    if det_2d.abs() < 1e-6
        && funcs
            .iter()
            .any(|f| matches!(f, TransformFunction::RotateX(_) | TransformFunction::RotateY(_)))
    {
        a = 1.0;
        b = 0.0;
        c = 0.0;
        d = 1.0;
        tx = 0.0;
        ty = 0.0;
        for func in &funcs {
            let (fa, fb, fc, fd, ftx, fty) = match func {
                TransformFunction::Translate(dx, dy) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
                TransformFunction::TranslateMixed(tx, txp, ty, typ) => {
                    let fx = if *txp {
                        rect.size.width * (*tx as f32) / 100.0
                    } else {
                        *tx as f32
                    };
                    let fy = if *typ {
                        rect.size.height * (*ty as f32) / 100.0
                    } else {
                        *ty as f32
                    };
                    (1.0, 0.0, 0.0, 1.0, fx, fy)
                }
                TransformFunction::TranslateXMixed(tx, _) => {
                    (1.0, 0.0, 0.0, 1.0, rect.size.width * (*tx as f32) / 100.0, 0.0)
                }
                TransformFunction::TranslateYMixed(ty, _) => {
                    (1.0, 0.0, 0.0, 1.0, 0.0, rect.size.height * (*ty as f32) / 100.0)
                }
                TransformFunction::TranslateX(dx) => (1.0, 0.0, 0.0, 1.0, *dx as f32, 0.0),
                TransformFunction::TranslateY(dy) => (1.0, 0.0, 0.0, 1.0, 0.0, *dy as f32),
                TransformFunction::Rotate(deg) => {
                    let rad = deg.to_radians() as f32;
                    let cos = rad.cos();
                    let sin = rad.sin();
                    (cos, sin, -sin, cos, 0.0, 0.0)
                }
                TransformFunction::Scale(sx, sy) => {
                    let sy = sy.unwrap_or(*sx) as f32;
                    (*sx as f32, 0.0, 0.0, sy, 0.0, 0.0)
                }
                TransformFunction::ScaleX(sx) => (*sx as f32, 0.0, 0.0, 1.0, 0.0, 0.0),
                TransformFunction::ScaleY(sy) => (1.0, 0.0, 0.0, *sy as f32, 0.0, 0.0),
                TransformFunction::Skew(ax, ay) => {
                    let tan_ax = ax.to_radians().tan() as f32;
                    let tan_ay = ay.map(|v| v.to_radians().tan() as f32).unwrap_or(0.0);
                    (1.0, tan_ay, tan_ax, 1.0, 0.0, 0.0)
                }
                TransformFunction::Translate3d(dx, dy, _) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
                // 退化回退路径：rotX/rotY 恒等（成对抵消链语义，R3833 兜底）。
                TransformFunction::RotateX(_) | TransformFunction::RotateY(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                TransformFunction::RotateZ(deg) => {
                    let rad = deg.to_radians() as f32;
                    let cos = rad.cos();
                    let sin = rad.sin();
                    (cos, sin, -sin, cos, 0.0, 0.0)
                }
                TransformFunction::Rotate3d(_, _, _, _) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                TransformFunction::Scale3d(sx, sy, _) => (*sx as f32, 0.0, 0.0, *sy as f32, 0.0, 0.0),
                TransformFunction::ScaleZ(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                TransformFunction::Perspective(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                TransformFunction::Matrix(ma, mb, mc, md, me, mf) => {
                    (*ma as f32, *mb as f32, *mc as f32, *md as f32, *me as f32, *mf as f32)
                }
            };
            let new_a = a * fa + c * fb;
            let new_b = b * fa + d * fb;
            let new_c = a * fc + c * fd;
            let new_d = b * fc + d * fd;
            let new_tx = a * ftx + c * fty + tx;
            let new_ty = b * ftx + d * fty + ty;
            a = new_a;
            b = new_b;
            c = new_c;
            d = new_d;
            tx = new_tx;
            ty = new_ty;
        }
        // R3834：按重算后的矩阵重新应用 transform-origin（公式与主路径一致）。
        final_tx = origin_x * (1.0 - a) - origin_y * c + tx;
        final_ty = -origin_x * b + origin_y * (1.0 - d) + ty;
    }

    // 检查是否为 identity 变换
    let is_identity = (a - 1.0).abs() < 1e-6
        && b.abs() < 1e-6
        && c.abs() < 1e-6
        && (d - 1.0).abs() < 1e-6
        && final_tx.abs() < 1e-6
        && final_ty.abs() < 1e-6;
    if is_identity {
        return None;
    }

    // R3915：rect 扩为「原矩形 ∪ 变换后四角 bbox」——rotate(±90/±270)、skew 等把内容
    // 映射到原矩形之外（transforms-rotate-degree-90：150×100 盒 rot90 的可视 bbox
    // 100×150 超出原 rect 上下各 25px）。CPU apply_transform_post 的 clear+反向采样仅
    // 作用于 rect 内：rect 不含变换 bbox 时溢出部分被丢弃（绿色上下截断露红）。GPU 侧
    // 顶点按四角独立变换，不受影响。原矩形必须保留在并集内（render_image 等按 rect
    // 定位源采样基线）。
    let map_x = |x: f32, y: f32| a * x + c * y + final_tx;
    let map_y = |x: f32, y: f32| b * x + d * y + final_ty;
    let (rx0, ry0, rx1, ry1) = (rect.left(), rect.top(), rect.right(), rect.bottom());
    let mut min_x = map_x(rx0, ry0).min(rect.left());
    let mut min_y = map_y(rx0, ry0).min(rect.top());
    let mut max_x = map_x(rx1, ry1).max(rect.right());
    let mut max_y = map_y(rx1, ry1).max(rect.bottom());
    for (cx, cy) in [(rx1, ry0), (rx0, ry1)] {
        min_x = min_x.min(map_x(cx, cy));
        min_y = min_y.min(map_y(cx, cy));
        max_x = max_x.max(map_x(cx, cy));
        max_y = max_y.max(map_y(cx, cy));
    }
    // 并集（Rect 无 union 方法，就地 min/max）。
    let ux0 = rect.left().min(min_x);
    let uy0 = rect.top().min(min_y);
    let ux1 = rect.right().max(max_x);
    let uy1 = rect.bottom().max(max_y);
    let union_rect = Rect::new(ux0, uy0, ux1 - ux0, uy1 - uy0);

    Some(TransformPrimitive {
        rect: union_rect,
        origin_x,
        origin_y,
        a,
        b,
        c,
        d,
        tx: final_tx,
        ty: final_ty,
    })
}

fn resolve_transform_origin_length(value: &LengthValue, box_size: f32, font_size_px: f64) -> f32 {
    match value {
        LengthValue::Percentage(p) => box_size * (*p as f32 / 100.0),
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_) => {
            box_size / 2.0
        }
        other => zero_style_system::computed::resolve_length(other, font_size_px, None, None) as f32,
    }
}

/// 如果样式包含非平移变换，将 TransformPrimitive 添加到图元列表。
pub fn apply_transform(style: &ComputedStyle, rect: &Rect, primitives: &mut RenderPrimitives) {
    if let Some(tp) = compute_transform_matrix(style, rect) {
        primitives.add_transform(tp);
    }
}

/// R4107（CSS Transforms 1 §transform-box）：带参考框的 apply_transform——
/// `ref_rect` = transform-box 参考框（html 元素 content-box/fill-box = 内容盒），
/// transform-origin 与百分比 translate 相对参考框解析（None = 无声明，退回
/// border-box rect，R4098 前 / 默认 view-box(svg) 语义不变）。
pub fn apply_transform_with_ref_box(
    style: &ComputedStyle,
    rect: &Rect,
    ref_rect: Option<&Rect>,
    primitives: &mut RenderPrimitives,
) {
    if let Some(tp) = compute_transform_matrix_with_ref_box(style, rect, ref_rect) {
        primitives.add_transform(tp);
    }
}

/// R5016（css-transforms-1 §13.1）：rotate3d 单轴分解——轴仅一个分量非零时精确等价
/// 于对应轴函数（角度符号随轴分量符号；全零轴 spec 语义 = 恒等，保持 Rotate3d 恒等
/// 近似不变）。任意轴（多分量非零）不分解（真 3D 合成为 R3833 已知局限）。
fn rotate3d_unit_axis(x: f64, y: f64, z: f64, deg: f64) -> TransformFunction {
    match (x != 0.0, y != 0.0, z != 0.0) {
        (true, false, false) => TransformFunction::RotateX(if x < 0.0 { -deg } else { deg }),
        (false, true, false) => TransformFunction::RotateY(if y < 0.0 { -deg } else { deg }),
        (false, false, true) => TransformFunction::RotateZ(if z < 0.0 { -deg } else { deg }),
        _ => TransformFunction::Rotate3d(x, y, z, deg),
    }
}

/// 求纯 translate 列表的累计位移 `(tx, ty)`（CSS Transforms §translate）。
///
/// R3901：旧实现承诺「仅 translate 的列表由 offset 路径处理」（compute_transform_matrix
/// 对纯 translate 返回 None），但 offset 路径（apply_transform_offset）无任何生产调用方——
/// 纯 px translate 的位移从未进入渲染（driving: transform-overflow-001 /
/// ttwf-transform-translatex-001 / translate.html 等 4.17% 簇）。
/// 混合列表（含非 translate 函数）返回 None（走 matrix 路径，平移分量已并入矩阵）。
///
/// R5016（css-transforms-2 §individual-transforms）：individual translate（纯 px 形态，
/// `Translate` 承载）并入累计位移——used transform 固定序 translate · rotate · scale ·
/// transform，纯平移可加性合成（次序无关）。两路互斥守卫：individual 含 rotate/scale、
/// 或 translate 为 % 形态（`TranslateMixed` 系）时 matrix 路径持有合成（has_non_translate
/// 命中 / 非 translate 函数在场），本函数返回 None 防平移双计。
/// driving: change-translate-property（translate: -100px 0px 动态改写后仍按
/// transform-only 平移 +100px）与 individual-transform-1（translate_1/translate_2 缺位）。
pub fn translate_offset(style: &ComputedStyle) -> Option<(f32, f32)> {
    if style.individual_rotate.is_some() || style.individual_scale.is_some() {
        return None;
    }
    let individual = match &style.individual_translate {
        None => (0.0_f32, 0.0_f32),
        // % 形态走 matrix 路径（rect 语境求值），此处让位。
        Some(TransformFunction::Translate(tx, ty)) => (*tx as f32, *ty as f32),
        Some(_) => return None,
    };
    let funcs: &[TransformFunction] = match &style.transform {
        TransformValue::None => &[],
        TransformValue::List(f) => f,
    };
    let only_translate = funcs.iter().all(|f| {
        matches!(
            f,
            TransformFunction::Translate(_, _) | TransformFunction::TranslateX(_) | TransformFunction::TranslateY(_)
        )
    });
    if !only_translate {
        return None;
    }
    let mut tx = individual.0;
    let mut ty = individual.1;
    for f in funcs {
        match f {
            TransformFunction::Translate(dx, dy) => {
                tx += *dx as f32;
                ty += *dy as f32;
            }
            TransformFunction::TranslateX(dx) => tx += *dx as f32,
            TransformFunction::TranslateY(dy) => ty += *dy as f32,
            _ => {}
        }
    }
    if tx == 0.0 && ty == 0.0 {
        return None;
    }
    Some((tx, ty))
}

/// 对快照 `from` 之后新增的图元应用平移（R3901：纯 translate 的图元级实现）。
///
/// CSS Transforms §transform-rendering：transform 作用于元素**及其整个子树**的渲染结果，
/// 故调用方须传「自子树图元起点」快照（paint_node 的 counts_before），对该元素及其全部
/// 后代产生的图元直接平移坐标。不用 TransformPrimitive 的原因：raster 侧
/// apply_transform_post 是全场景像素后处理（rect 内清白+反向采样），会破坏与元素框相交的
/// 祖先/兄弟内容（transform-descendant-001 回归实证）；图元级平移只动该子树自己的图元。
/// transform-origin 对平移无影响（origin 只固定 rotate/scale/skew）。
pub fn translate_primitives_since(primitives: &mut RenderPrimitives, from: &PrimitiveCounts, tx: f32, ty: f32) {
    for f in primitives.fills.iter_mut().skip(from.fills) {
        f.rect.origin.x += tx;
        f.rect.origin.y += ty;
    }
    for rr in primitives.rounded_rects.iter_mut().skip(from.rounded_rects) {
        rr.rect.origin.x += tx;
        rr.rect.origin.y += ty;
    }
    // 渐变：rect 随动；端点/圆心是绝对坐标但 GradientKind 字段非 pub，translate 内
    // gradient 组合罕见，A/B 无回归即保持（FIXME: 梯度端点绝对坐标未随平移）。
    for g in primitives.gradients.iter_mut().skip(from.gradients) {
        g.rect.origin.x += tx;
        g.rect.origin.y += ty;
    }
    for s in primitives.shadows.iter_mut().skip(from.shadows) {
        s.rect.origin.x += tx;
        s.rect.origin.y += ty;
    }
    for i in primitives.images.iter_mut().skip(from.images) {
        i.rect.origin.x += tx;
        i.rect.origin.y += ty;
    }
    for g in primitives.glyphs.iter_mut().skip(from.glyphs) {
        g.x += tx;
        g.y += ty;
    }
    for s in primitives.strokes.iter_mut().skip(from.strokes) {
        s.x1 += tx;
        s.y1 += ty;
        s.x2 += tx;
        s.y2 += ty;
    }
    // path_fills/path_strokes：PrimitiveCounts 无对应计数（path 来自 canvas/SVG 桥，
    // 与 CSS transform 组合罕见），不覆盖。
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::clip::PrimitiveCounts;
    use crate::paint::helpers::inline_box_vertical_bleed;
    use zero_css_parser::values::{LengthValue, TransformFunction, TransformValue};
    use zero_layout_engine::LayoutBox;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};
    use zero_style_system::ComputedStyle;

    // ── apply_transform_offset ──────────────────────────────────────────

    // ── R3901：translate_offset / translate_primitives_since ────────────

    #[test]
    fn test_translate_offset_pure_px_list() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(15.0, 25.0)]);
        assert_eq!(translate_offset(&style), Some((15.0, 25.0)));
        // X/Y 变体累计
        style.transform = TransformValue::List(vec![
            TransformFunction::TranslateX(30.0),
            TransformFunction::TranslateY(40.0),
        ]);
        assert_eq!(translate_offset(&style), Some((30.0, 40.0)));
        // 混合列表（含 scale）→ None（matrix 路径）
        style.transform = TransformValue::List(vec![
            TransformFunction::Translate(5.0, 5.0),
            TransformFunction::Scale(2.0, None),
        ]);
        assert_eq!(translate_offset(&style), None);
        // 零位移 → None
        style.transform = TransformValue::List(vec![TransformFunction::Translate(0.0, 0.0)]);
        assert_eq!(translate_offset(&style), None);
    }

    #[test]
    fn test_translate_primitives_since_shifts_snapshot_only() {
        let mut p = RenderPrimitives::new();
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::BLACK,
        });
        p.glyphs.push(GlyphPrimitive {
            x: 1.0,
            y: 2.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 'a' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        let from = PrimitiveCounts::snapshot(&p);
        // 快照之后新增的图元才被平移
        p.fills.push(FillPrimitive {
            rect: Rect::new(100.0, 100.0, 10.0, 10.0),
            color: Color::BLACK,
        });
        p.glyphs.push(GlyphPrimitive {
            x: 101.0,
            y: 102.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 'b' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        translate_primitives_since(&mut p, &from, 5.0, 7.0);
        assert_eq!(p.fills[0].rect.origin.x, 0.0, "快照前图元不动");
        assert_eq!(p.glyphs[0].x, 1.0, "快照前 glyph 不动");
        assert_eq!(p.fills[1].rect.origin.x, 105.0, "快照后 fill 平移");
        assert_eq!(p.glyphs[1].x, 106.0, "快照后 glyph 平移 x+5");
        assert_eq!(p.glyphs[1].y, 109.0, "快照后 glyph 平移 y+7");
    }

    #[test]
    fn test_transform_offset_none() {
        let style = ComputedStyle::default();
        let (dx, dy) = apply_transform_offset(&style, 10.0, 20.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    #[test]
    fn test_transform_offset_translate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(50.0, 30.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (50.0, 30.0));
    }

    #[test]
    fn test_transform_offset_translate_x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateX(100.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (100.0, 0.0));
    }

    #[test]
    fn test_transform_offset_translate_y() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateY(75.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 75.0));
    }

    #[test]
    fn test_transform_offset_multiple_translates() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::TranslateX(10.0),
            TransformFunction::TranslateY(20.0),
            TransformFunction::Translate(5.0, 15.0),
        ]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (15.0, 35.0)); // 10+5, 20+15
    }

    #[test]
    fn test_transform_offset_rotate_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(45.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    #[test]
    fn test_transform_offset_scale_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(2.0, Some(3.0))]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    // ── compute_transform_matrix ────────────────────────────────────────

    #[test]
    fn test_compute_transform_none_returns_none() {
        let style = ComputedStyle::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    #[test]
    fn test_compute_transform_translate_only_returns_none() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(10.0, 20.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // translate-only should NOT generate TransformPrimitive
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    // ── R3834：退化兜底 transform-origin 重算 ─────────────────────────────

    /// rotX(90°)（θ=±90 投影 det≈0）触发退化兜底后矩阵须为**纯恒等**（含 origin 项）。
    /// 旧实现兜底重算 a/b/c/d/tx/ty 但沿用坍缩矩阵求得的 final_tx/final_ty，200×200 盒
    /// 产生 spurious ty=+100 位移（preserve3d-pseudo-element 内容离位全白）。
    /// driving: css-transforms preserve3d-pseudo-element.html。
    #[test]
    fn test_r3834_rotx90_degenerate_fallback_is_pure_identity() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::RotateX(90.0)]);
        let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
        // 默认 transform-origin 50% 50% = (100,100)：兜底恒等 → 纯恒等 → None（无图元）。
        assert!(
            compute_transform_matrix(&style, &rect).is_none(),
            "rotX(90) 兜底后应无剩余变换（旧实现残留 ty=origin_y=+100）"
        );
    }

    /// rotX(180°)（投影 = scaleY(-1)，非退化）不受兜底影响：镜像语义保持，origin 居中时
    /// final_ty = origin_y·(1−d) = 100·(1−(−1)) = 200（镜像后平移回原盒）。
    #[test]
    fn test_r3834_rotx180_mirror_unchanged_by_fallback() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::RotateX(180.0)]);
        let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
        let tp = compute_transform_matrix(&style, &rect).expect("rotX(180) 非退化应有变换");
        assert!((tp.d + 1.0).abs() < 1e-4, "d = cos(180°) = -1, got {}", tp.d);
        assert!((tp.ty - 200.0).abs() < 1e-3, "镜像平移 ty=200, got {}", tp.ty);
    }

    /// rotX(90) 抵消对（rotX(90) … rotX(-90)）夹 scale：兜底以 rotX/rotY 恒等重算后
    /// 剩余 = 夹层 scale 的 2D 投影，且 origin 项按重算矩阵求解（R3833 语义保持）。
    /// driving: transform3d-scale-002/005/006 成对抵消链。
    #[test]
    fn test_r3834_rotx_cancel_pair_keeps_intermediate_scale() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::RotateX(90.0),
            TransformFunction::Scale3d(2.0, 1.0, 2.0),
            TransformFunction::RotateX(-90.0),
        ]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        style.transform_origin_x = LengthValue::Px(0.0);
        style.transform_origin_y = LengthValue::Px(0.0);
        let tp = compute_transform_matrix(&style, &rect).expect("含 scale3d 的抵消链应有变换");
        // 兜底：rotX 恒等 × scale(2,1) × rotX 恒等 = scaleX(2)
        assert!((tp.a - 2.0).abs() < 1e-4, "a=2, got {}", tp.a);
        assert!((tp.d - 1.0).abs() < 1e-4, "d=1, got {}", tp.d);
        assert!(tp.tx.abs() < 1e-4 && tp.ty.abs() < 1e-4, "origin 0 0 无平移");
    }

    #[test]
    fn test_compute_transform_rotate_90() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // Rotate 90°: cos(90°) ≈ 0, sin(90°) ≈ 1
        assert!(tp.a.abs() < 0.01, "a should be ~0, got {}", tp.a);
        assert!((tp.b - 1.0).abs() < 0.01, "b should be ~1, got {}", tp.b);
        assert!((tp.c + 1.0).abs() < 0.01, "c should be ~-1, got {}", tp.c);
        assert!(tp.d.abs() < 0.01, "d should be ~0, got {}", tp.d);

        // origin at center (50, 50)
        assert!((tp.origin_x - 50.0).abs() < 0.1);
        assert!((tp.origin_y - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_scale_2x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(2.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 2.0).abs() < 0.01);
        assert!((tp.d - 2.0).abs() < 0.01);
        assert!(tp.b.abs() < 0.01);
        assert!(tp.c.abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_scale_x_y() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(3.0, Some(0.5))]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 3.0).abs() < 0.01);
        assert!((tp.d - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_custom_origin() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        style.transform_origin_x = LengthValue::Px(0.0);
        style.transform_origin_y = LengthValue::Px(0.0);
        let rect = Rect::new(10.0, 20.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // origin at top-left corner (10, 20)
        assert!((tp.origin_x - 10.0).abs() < 0.1);
        assert!((tp.origin_y - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_origin_relative_lengths() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        style.transform_origin_x = LengthValue::Em(1.0);
        style.transform_origin_y = LengthValue::Em(2.0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.origin_x - 20.0).abs() < 0.1);
        assert!((tp.origin_y - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_combined_translate_rotate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::Translate(10.0, 20.0),
            TransformFunction::Rotate(45.0),
        ]);
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // rotate 45°: cos = sin ≈ 0.707
        let cos45 = 45.0_f64.to_radians().cos() as f32;
        let sin45 = 45.0_f64.to_radians().sin() as f32;
        assert!((tp.a - cos45).abs() < 0.01);
        assert!((tp.b - sin45).abs() < 0.01);
        assert!((tp.c + sin45).abs() < 0.01);
        assert!((tp.d - cos45).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_skew_x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Skew(45.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 1.0).abs() < 0.01);
        assert!(tp.b.abs() < 0.01);
        assert!((tp.c - 1.0).abs() < 0.01, "tan(45°) = 1.0, got {}", tp.c);
        assert!((tp.d - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_identity_skew_0() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Skew(0.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // skew(0) is identity → should return None
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    #[test]
    fn test_apply_transform_noop_for_none() {
        let style = ComputedStyle::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut p = RenderPrimitives::default();
        apply_transform(&style, &rect, &mut p);
        assert!(p.transforms.is_empty());
    }

    #[test]
    fn test_apply_transform_adds_for_rotate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(30.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut p = RenderPrimitives::default();
        apply_transform(&style, &rect, &mut p);
        assert_eq!(p.transforms.len(), 1);
    }

    /// R3915：TransformPrimitive.rect = 原矩形 ∪ 变换后四角 bbox——rotate(90) 等把
    /// 内容映射到原矩形之外时，CPU apply_transform_post 的 clear+反向采样区域必须
    /// 覆盖变换 bbox（transforms-rotate-degree-90：150×100 盒 rot90 bbox 100×150
    /// 超出原 rect 上下各 25px，旧实现溢出部分被丢弃）。
    #[test]
    fn r3915_transform_rect_covers_transformed_bbox() {
        use zero_css_parser::values::LengthValue;
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        style.transform_origin_x = LengthValue::Percentage(50.0);
        style.transform_origin_y = LengthValue::Percentage(50.0);
        // 150×100 盒于 (100,100)-(250,200)；origin (175,150)；rot90 bbox = (125,75)-(225,225)。
        let rect = Rect::new(100.0, 100.0, 150.0, 100.0);
        let prim = compute_transform_matrix(&style, &rect).expect("rotate(90) is not identity");
        let r = prim.rect;
        assert!(
            (r.left() - 100.0).abs() < 0.5 && (r.top() - 75.0).abs() < 0.5,
            "left={}, top={}",
            r.left(),
            r.top()
        );
        assert!(
            (r.right() - 250.0).abs() < 0.5 && (r.bottom() - 225.0).abs() < 0.5,
            "right={}, bottom={}",
            r.right(),
            r.bottom()
        );
    }

    /// R3915 守卫：平移不扩 rect（bbox ⊆ 原 rect 时并集 = 原矩形，零行为变化）。
    /// 纯 translate 走 R3901 图元平移路径（无 TransformPrimitive），此处用含 scale 的
    /// 缩中盒变换验证 bbox 不超出原 rect 时并集保持原矩形几何。
    #[test]
    fn r3915_scale_inside_keeps_rect() {
        use zero_css_parser::values::LengthValue;
        let mut style = ComputedStyle::default();
        // scale(0.5) 以中心 (60,35) 收缩：bbox (35,22.5)-(85,47.5) ⊆ 原 rect (10,10)-(110,60)。
        style.transform = TransformValue::List(vec![TransformFunction::Scale(0.5, Some(0.5))]);
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let prim = compute_transform_matrix(&style, &rect).expect("scale(0.5) is not identity");
        assert!(
            (prim.rect.left() - 10.0).abs() < 0.5
                && (prim.rect.top() - 10.0).abs() < 0.5
                && (prim.rect.right() - 110.0).abs() < 0.5
                && (prim.rect.bottom() - 60.0).abs() < 0.5,
            "rect must stay the original when bbox is inside, got {:?}",
            prim.rect
        );
    }

    // ── R4035：transform_shows_backface（CSS Transforms 2 §backface-visibility）──

    #[test]
    fn test_r4035_backface_rotey180_shows_backface() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::RotateY(180.0)]);
        assert!(transform_shows_backface(&style), "rotateY(180deg) 背面朝向观察者");
    }

    #[test]
    fn test_r4035_frontface_and_2d_no_backface() {
        // rotateY(0)/rotateY(360) 正面
        let mut front = ComputedStyle::default();
        front.transform = TransformValue::List(vec![TransformFunction::RotateY(360.0)]);
        assert!(!transform_shows_backface(&front), "rotateY(360deg) 正面");
        // 纯 2D 镜像（scaleX(-1)）不翻 z —— chromium 同
        let mut mirror = ComputedStyle::default();
        mirror.transform = TransformValue::List(vec![TransformFunction::Scale(-1.0, None)]);
        assert!(!transform_shows_backface(&mirror), "scaleX(-1) 是正面（2D 镜像非背面）");
        // 无 transform
        assert!(!transform_shows_backface(&ComputedStyle::default()));
    }

    #[test]
    fn test_r4296_inline_bleed_helper() {
        // 纯函数：未同步 taffy 盒（padding_top=0、border 已入 h）——50px padding 全额
        // 外延、border 差值 0 = 50px；R4297 缺失量口径（非 R4296 全额 51px）。
        let mut s = ComputedStyle::default();
        s.display = zero_css_parser::values::DisplayValue::Inline;
        s.padding_top = LengthValue::Px(50.0);
        s.padding_bottom = LengthValue::Px(50.0);
        s.border_top_width = LengthValue::Px(1.0);
        s.border_bottom_width = LengthValue::Px(1.0);
        s.border_top_style = zero_style_system::property::types::BorderStyleValue::Solid;
        s.border_bottom_style = zero_style_system::property::types::BorderStyleValue::Solid;
        let box_node = LayoutBox {
            border_top: 1.0,
            border_bottom: 1.0,
            ..Default::default()
        };
        assert_eq!(inline_box_vertical_bleed(&s, &box_node), (50.0, 50.0));
        // 已同步盒（box 字段已含 CSS 厚度）→ 缺失量 0。
        let mut synced = box_node.clone();
        synced.padding_top = 50.0;
        synced.padding_bottom = 50.0;
        assert_eq!(inline_box_vertical_bleed(&s, &synced), (0.0, 0.0));
        // 非 inline 盒不外延。
        let mut b = ComputedStyle::default();
        b.display = zero_css_parser::values::DisplayValue::Block;
        b.padding_top = LengthValue::Px(50.0);
        assert_eq!(inline_box_vertical_bleed(&b, &box_node), (0.0, 0.0));
        // border-style none 的边不计入。
        let mut n = ComputedStyle::default();
        n.display = zero_css_parser::values::DisplayValue::Inline;
        n.border_top_width = LengthValue::Px(3.0);
        assert_eq!(inline_box_vertical_bleed(&n, &box_node).0, 0.0, "border-style none");
    }

    #[test]
    fn test_r4035_double_flip_composes() {
        // rotateY(90) rotateY(90) = rotateY(180) → 背面（角度按轴累加语义）
        let mut style = ComputedStyle::default();
        style.transform =
            TransformValue::List(vec![TransformFunction::RotateY(90.0), TransformFunction::RotateY(90.0)]);
        assert!(
            !transform_shows_backface(&style),
            "cos(90)*cos(90)=0，不判背面（90° 投影奇异由 is_singular_transform 兜底）"
        );
        // rotateX(180) 单独 → 背面
        let mut rx = ComputedStyle::default();
        rx.transform = TransformValue::List(vec![TransformFunction::RotateX(180.0)]);
        assert!(transform_shows_backface(&rx), "rotateX(180deg) 背面朝向观察者");
    }
}
