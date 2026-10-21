//! M2-S1c：compositor 侧动画采样覆盖（goal：compositor-animation-continuity）。
//!
//! Get 帧时对已存 paint 的 **clone** 按共享单调钟采样声明并就地覆盖（与
//! `scroll_transform` 同构：基线 `surface.paint` 不被改写）：
//! - opacity：作用域 `draw_order` 区间内图元颜色 alpha 乘采样值。重叠作用域
//!   （父子动画嵌套）因子累积后每图元一次舍入；覆盖集合与 engine 静态烘焙
//!   `apply_opacity_to_new_primitives` 一致（fills/rounded_rects/gradients/
//!   shadows/glyphs/strokes；image 无色、path 引擎侧本就不烘焙）——镜像同集合
//!   保 t=0/无覆盖帧与静态输出逐字节等价（S1e 验收口径）。
//! - transform：`IpcAnimationScope.transform_index` 指向的恒等基线 `IpcTransform`
//!   就地改写为采样函数列表折叠仿射；origin 项按 engine
//!   `compute_transform_matrix`（transform.rs「应用 transform-origin 偏移」式）
//!   并入 tx/ty，经 protocol `multiply_affine` 单一实现组合（R3833：光栅侧消费
//!   的 tx/ty 是含 origin 的全矩阵）；rect 按 R3915 同款并集扩为
//!   「原矩形 ∪ 变换后四角 bbox」——CPU `apply_transform_post` 的 clear+反向
//!   采样仅作用于 rect 内，rect 不含变换 bbox 时溢出部分被丢弃。
//!
//! RFC：`docs/goal/compositor-animation-continuity/m2-slice-compositor-sampling.md` §2.4。

use std::collections::HashMap;

use zero_protocol::animation::{
    AnimationPropertyValue, CompositorAnimationDeclaration, IpcAnimationScope, TransformFunctionValue, fold_affine,
    multiply_affine, sample_declaration,
};
use zero_protocol::paint_snapshot::{IpcColor, IpcDrawOp, IpcRect, PaintSnapshotParams};

/// 是否仍有未完成的可合成动画（Get 节奏判定）。
///
/// 延迟期也算活跃：采样无覆盖但浏览器须保持轮询，延迟结束后采样无需新声明
/// 即可接管。非 `composable` 声明不驱动节奏——它们不产生覆盖，维持轮询只会
/// 空转。
pub fn has_active_animations(declarations: &[CompositorAnimationDeclaration], now_monotonic_ns: u64) -> bool {
    declarations
        .iter()
        .any(|declaration| declaration.composable && !sample_declaration(declaration, now_monotonic_ns).finished)
}

/// 采样并覆盖 paint（调用方传入 clone）。返回 [`has_active_animations`] 同口径的
/// 活跃标志（响应 `animations_active`；与本帧是否实际产生覆盖无关——延迟期
/// 无覆盖也活跃）。
pub fn apply(
    paint: &mut PaintSnapshotParams,
    declarations: &[CompositorAnimationDeclaration],
    now_monotonic_ns: u64,
) -> bool {
    if paint.animation_scopes.is_empty() || declarations.is_empty() {
        return false;
    }
    // 每声明采样一次；finished/延迟无覆盖 → None（无覆盖 = 基线像素，与
    // renderer 中性化基线一致，见模块文档）。
    let samples: HashMap<u64, Option<AnimationPropertyValue>> = declarations
        .iter()
        .filter(|declaration| declaration.composable)
        .map(|declaration| {
            (
                declaration.animation_id,
                sample_declaration(declaration, now_monotonic_ns).value,
            )
        })
        .collect();
    let scopes = paint.animation_scopes.clone();

    // transform 通道：恒等基线矩阵就地改写。导出规则「单通道单动画」保证槽位
    // 不被两条 transform 动画共享；None/越界槽位来自不含本特性的旧 peer，跳过
    // 即无覆盖（基线像素）。
    for scope in &scopes {
        if let Some(Some(AnimationPropertyValue::Transform(functions))) = samples.get(&scope.animation_id) {
            apply_transform_scope(paint, scope, functions);
        }
    }

    // opacity 通道：因子累积（重叠作用域乘积合成，每图元一次舍入）后统一落盘。
    let mut factors = AlphaFactors::default();
    for scope in &scopes {
        if let Some(Some(AnimationPropertyValue::Opacity(value))) = samples.get(&scope.animation_id) {
            accumulate_opacity(&paint.draw_order, scope, *value, &mut factors);
        }
    }
    factors.flush(paint);

    // 采样可能把内容移出原脏区（transform 四角 bbox / opacity 作用域整体），
    // 与 `scroll_transform::paint_for_viewport` 同款全视口脏区 → `rasterize_into_back`
    // 走 `is_full_viewport` 全量 CPU 重光栅（RFC §2.4）。
    paint.dirty_rects = vec![IpcRect {
        x: 0.0,
        y: 0.0,
        width: paint.viewport_width.max(1) as f32,
        height: paint.viewport_height.max(1) as f32,
    }];

    has_active_animations(declarations, now_monotonic_ns)
}

/// 改写一个 transform 作用域的基线矩阵（含 origin 全矩阵 + R3915 rect 并集）。
fn apply_transform_scope(
    paint: &mut PaintSnapshotParams,
    scope: &IpcAnimationScope,
    functions: &[TransformFunctionValue],
) {
    let Some(index) = scope.transform_index.map(|i| i as usize) else {
        return;
    };
    let Some(affine) = fold_affine(functions) else {
        return;
    };
    let Some(transform) = paint.transforms.get_mut(index) else {
        return;
    };
    // 含 origin 全矩阵 = T(origin) · A · T(−origin)（engine compute_transform_matrix
    // 的 origin 项同式；经 protocol multiply_affine 单一实现组合）。
    let (ox, oy) = (f64::from(transform.origin_x), f64::from(transform.origin_y));
    let full = multiply_affine(
        multiply_affine([1.0, 0.0, 0.0, 1.0, ox, oy], affine),
        [1.0, 0.0, 0.0, 1.0, -ox, -oy],
    );
    let [a, b, c, d, tx, ty] = full;
    transform.rect = union_transformed_rect(
        transform.rect,
        [a as f32, b as f32, c as f32, d as f32, tx as f32, ty as f32],
    );
    transform.a = a as f32;
    transform.b = b as f32;
    transform.c = c as f32;
    transform.d = d as f32;
    transform.tx = tx as f32;
    transform.ty = ty as f32;
}

/// R3915 同款并集：rect ∪ 矩阵映射四角 bbox（`[a, b, c, d, tx, ty]` 列向量约定；
/// 原 rect 保留在并集内，收敛映射（scale < 1）不缩采样区域）。
fn union_transformed_rect(rect: IpcRect, m: [f32; 6]) -> IpcRect {
    let [a, b, c, d, tx, ty] = m;
    let (x0, y0) = (rect.x, rect.y);
    let (x1, y1) = (rect.x + rect.width, rect.y + rect.height);
    let map_x = |x: f32, y: f32| a * x + c * y + tx;
    let map_y = |x: f32, y: f32| b * x + d * y + ty;
    let mut min_x = map_x(x0, y0).min(x0);
    let mut min_y = map_y(x0, y0).min(y0);
    let mut max_x = map_x(x1, y1).max(x1);
    let mut max_y = map_y(x1, y1).max(y1);
    for (cx, cy) in [(x1, y0), (x0, y1)] {
        min_x = min_x.min(map_x(cx, cy));
        min_y = min_y.min(map_y(cx, cy));
        max_x = max_x.max(map_x(cx, cy));
        max_y = max_y.max(map_y(cx, cy));
    }
    IpcRect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

/// 把一个 opacity 作用域的采样值累积进因子表。
fn accumulate_opacity(draw_order: &[IpcDrawOp], scope: &IpcAnimationScope, value: f64, factors: &mut AlphaFactors) {
    let begin = scope.draw_begin as usize;
    let end = (scope.draw_end as usize).min(draw_order.len());
    for op in draw_order.get(begin..end).into_iter().flatten() {
        factors.accumulate(op, value as f32);
    }
}

/// opacity 采样因子累积表：重叠作用域按乘积合成，每图元仅一次舍入；键为图元
/// 下标。作用域数远小于图元数，稀疏表即可。
#[derive(Default)]
struct AlphaFactors {
    fills: HashMap<usize, f32>,
    rounded_rects: HashMap<usize, f32>,
    gradients: HashMap<usize, f32>,
    shadows: HashMap<usize, f32>,
    glyphs: HashMap<usize, f32>,
    strokes: HashMap<usize, f32>,
}

impl AlphaFactors {
    fn accumulate(&mut self, op: &IpcDrawOp, value: f32) {
        match *op {
            IpcDrawOp::Fill(i) => *self.fills.entry(i).or_insert(1.0) *= value,
            IpcDrawOp::RoundedRect(i) => *self.rounded_rects.entry(i).or_insert(1.0) *= value,
            IpcDrawOp::Gradient(i) => *self.gradients.entry(i).or_insert(1.0) *= value,
            IpcDrawOp::Shadow(i) => *self.shadows.entry(i).or_insert(1.0) *= value,
            IpcDrawOp::Glyph(i) => *self.glyphs.entry(i).or_insert(1.0) *= value,
            IpcDrawOp::Stroke(i) => *self.strokes.entry(i).or_insert(1.0) *= value,
            // image 无色属性；path/clip/transform/filter/blend 引擎静态路径本就
            // 不做 opacity 烘焙（engine clip.rs `apply_opacity_to_new_primitives`），
            // 镜像同集合保无覆盖帧逐字节等价。
            IpcDrawOp::Image(_) | IpcDrawOp::PathFill(_) | IpcDrawOp::PathStroke(_) => {}
            IpcDrawOp::Clip(_) | IpcDrawOp::Transform(_) | IpcDrawOp::Filter(_) | IpcDrawOp::BlendMode(_) => {}
        }
    }

    fn scale_color(color: &mut IpcColor, factor: f32) {
        // 饱和 `as` 转换与 engine 静态烘焙同语义（factor ∈ [0,1] 时无饱和路径）。
        color.a = (f32::from(color.a) * factor).round() as u8;
    }

    fn flush(&self, paint: &mut PaintSnapshotParams) {
        for (index, factor) in &self.fills {
            if let Some(fill) = paint.fills.get_mut(*index) {
                Self::scale_color(&mut fill.color, *factor);
            }
        }
        for (index, factor) in &self.rounded_rects {
            if let Some(rect) = paint.rounded_rects.get_mut(*index) {
                Self::scale_color(&mut rect.color, *factor);
            }
        }
        for (index, factor) in &self.gradients {
            if let Some(gradient) = paint.gradients.get_mut(*index) {
                for stop in &mut gradient.stops {
                    Self::scale_color(&mut stop.color, *factor);
                }
            }
        }
        for (index, factor) in &self.shadows {
            if let Some(shadow) = paint.shadows.get_mut(*index) {
                Self::scale_color(&mut shadow.color, *factor);
            }
        }
        for (index, factor) in &self.glyphs {
            if let Some(glyph) = paint.glyphs.get_mut(*index) {
                Self::scale_color(&mut glyph.color, *factor);
            }
        }
        for (index, factor) in &self.strokes {
            if let Some(stroke) = paint.strokes.get_mut(*index) {
                Self::scale_color(&mut stroke.color, *factor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::animation::{
        AnimationDirection, AnimationEasing, AnimationFillMode, AnimationKeyframe, AnimationSample, AnimationTiming,
    };
    use zero_protocol::paint_snapshot::{IpcFill, IpcTransform};

    fn opacity_declaration(animation_id: u64, duration_ms: f64, from: f64, to: f64) -> CompositorAnimationDeclaration {
        CompositorAnimationDeclaration {
            animation_id,
            property: zero_protocol::animation::AnimationPropertyKind::Opacity,
            composable: true,
            start_monotonic_ns: 1_000_000_000,
            timing: AnimationTiming {
                duration_ms,
                delay_ms: 0.0,
                iterations: Some(1.0),
                direction: AnimationDirection::Normal,
                fill: AnimationFillMode::None,
                easing: AnimationEasing::Linear,
            },
            keyframes: vec![
                AnimationKeyframe {
                    offset: 0.0,
                    value: AnimationPropertyValue::Opacity(from),
                    easing: None,
                },
                AnimationKeyframe {
                    offset: 1.0,
                    value: AnimationPropertyValue::Opacity(to),
                    easing: None,
                },
            ],
        }
    }

    fn transform_declaration(animation_id: u64, from_deg: f64, to_deg: f64) -> CompositorAnimationDeclaration {
        CompositorAnimationDeclaration {
            animation_id,
            property: zero_protocol::animation::AnimationPropertyKind::Transform,
            composable: true,
            start_monotonic_ns: 1_000_000_000,
            timing: AnimationTiming {
                duration_ms: 1000.0,
                delay_ms: 0.0,
                iterations: Some(1.0),
                direction: AnimationDirection::Normal,
                fill: AnimationFillMode::None,
                easing: AnimationEasing::Linear,
            },
            keyframes: vec![
                AnimationKeyframe {
                    offset: 0.0,
                    value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Rotate(from_deg)]),
                    easing: None,
                },
                AnimationKeyframe {
                    offset: 1.0,
                    value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Rotate(to_deg)]),
                    easing: None,
                },
            ],
        }
    }

    fn scope(animation_id: u64, property: zero_protocol::animation::AnimationPropertyKind) -> IpcAnimationScope {
        IpcAnimationScope {
            animation_id,
            property,
            draw_begin: 0,
            draw_end: 1,
            rect: [100.0, 100.0, 100.0, 100.0],
            transform_index: None,
        }
    }

    fn base_paint() -> PaintSnapshotParams {
        PaintSnapshotParams {
            viewport_width: 400,
            viewport_height: 400,
            document_height: 400.0,
            fills: vec![
                IpcFill {
                    rect: IpcRect {
                        x: 100.0,
                        y: 100.0,
                        width: 100.0,
                        height: 100.0,
                    },
                    color: IpcColor {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                },
                IpcFill {
                    rect: IpcRect {
                        x: 250.0,
                        y: 0.0,
                        width: 50.0,
                        height: 50.0,
                    },
                    color: IpcColor {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 255,
                    },
                },
            ],
            draw_order: vec![IpcDrawOp::Fill(0), IpcDrawOp::Fill(1)],
            ..Default::default()
        }
    }

    /// opacity 作用域只缩放区间内图元 alpha；区间外不动；活跃标志为真。
    #[test]
    fn opacity_scope_scales_interval_alpha_only() {
        let mut paint = base_paint();
        paint.animation_scopes = vec![scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)];
        let declarations = [opacity_declaration(1, 1000.0, 0.3, 1.0)];

        let active = apply(&mut paint, &declarations, 1_500_000_000);

        assert!(active);
        // 半程采样 0.65 → 255·0.65 = 165.75 → 166。
        assert_eq!(paint.fills[0].color.a, 166);
        assert_eq!(paint.fills[1].color.a, 255, "区间外图元不动");
        // 全视口脏区（全量重光栅）。
        assert_eq!(paint.dirty_rects.len(), 1);
        assert_eq!(paint.dirty_rects[0].width, 400.0);
    }

    /// 重叠作用域（父子嵌套）按乘积合成，每图元一次舍入。
    #[test]
    fn overlapping_opacity_scopes_compose_product() {
        let mut paint = base_paint();
        paint.animation_scopes = vec![
            IpcAnimationScope {
                animation_id: 1,
                ..scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)
            },
            IpcAnimationScope {
                animation_id: 2,
                ..scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)
            },
        ];
        let declarations = [
            opacity_declaration(1, 1000.0, 0.63, 0.63),
            opacity_declaration(2, 1000.0, 0.63, 0.63),
        ];

        let active = apply(&mut paint, &declarations, 1_100_000_000);

        assert!(active);
        // 0.63·0.63 = 0.3969 → 255·0.3969 = 101.2 → 101（单次舍入）。
        assert_eq!(paint.fills[0].color.a, 101);
    }

    /// transform 作用域：恒等基线改写为采样折叠仿射（origin 项入 tx/ty），rect
    /// 扩为旋转 bbox 并集。
    #[test]
    fn transform_scope_rewrites_matrix_around_origin_and_expands_rect() {
        let mut paint = base_paint();
        paint.transforms = vec![IpcTransform {
            rect: IpcRect {
                x: 100.0,
                y: 100.0,
                width: 100.0,
                height: 100.0,
            },
            origin_x: 150.0,
            origin_y: 150.0,
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }];
        let mut transform_scope = scope(1, zero_protocol::animation::AnimationPropertyKind::Transform);
        transform_scope.transform_index = Some(0);
        paint.animation_scopes = vec![transform_scope];
        let declarations = [transform_declaration(1, 0.0, 90.0)];

        let active = apply(&mut paint, &declarations, 1_500_000_000);

        assert!(active);
        let transform = &paint.transforms[0];
        // 半程 45°：a=d=cos45，b=−c=sin45。
        let cos45 = std::f64::consts::FRAC_1_SQRT_2 as f32;
        assert!((transform.a - cos45).abs() < 1e-5);
        assert!((transform.b - cos45).abs() < 1e-5);
        assert!((transform.c + cos45).abs() < 1e-5);
        assert!((transform.d - cos45).abs() < 1e-5);
        // origin (150,150) 项：tx = ox(1−a) − oy·c；ty = −ox·b + oy(1−d)。
        let expected_tx = 150.0 * (1.0 - cos45) - 150.0 * (-cos45);
        let expected_ty = -150.0 * cos45 + 150.0 * (1.0 - cos45);
        assert!((transform.tx - expected_tx).abs() < 1e-3);
        assert!((transform.ty - expected_ty).abs() < 1e-3);
        // rect 扩为旋转 bbox（对角 100√2 ≈ 141.4）∪ 原 rect。
        assert!(transform.rect.width > 140.0 && transform.rect.width < 142.0);
        assert!(transform.rect.x < 100.0, "rot90 bbox 越出原 rect 左界");
    }

    /// 已结束（fill none）与非 composable 声明不产生覆盖且不活跃。
    #[test]
    fn finished_and_non_composable_skip_and_report_inactive() {
        let mut paint = base_paint();
        paint.animation_scopes = vec![scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)];
        let mut declaration = opacity_declaration(1, 1000.0, 0.3, 1.0);

        let active = apply(&mut paint, &[declaration.clone()], 2_500_000_000);
        assert!(!active);
        assert_eq!(paint.fills[0].color.a, 255, "已结束无 fill → 基线");

        declaration.composable = false;
        let active = apply(&mut paint, std::slice::from_ref(&declaration), 1_500_000_000);
        assert!(!active);
        assert_eq!(paint.fills[0].color.a, 255, "非 composable 不覆盖");

        assert!(!has_active_animations(&[declaration], 1_000_000_000));
    }

    /// 延迟期无 backwards fill：无覆盖但活跃（浏览器保持轮询，延迟结束后采样接管）。
    #[test]
    fn delay_without_fill_keeps_active_with_no_override() {
        let mut declaration = opacity_declaration(1, 1000.0, 0.3, 1.0);
        declaration.timing.delay_ms = 500.0;
        let mut paint = base_paint();
        paint.animation_scopes = vec![scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)];

        let active = apply(&mut paint, std::slice::from_ref(&declaration), 1_200_000_000);

        assert!(active, "延迟期仍活跃");
        assert_eq!(paint.fills[0].color.a, 255, "延迟期无覆盖 → 基线");
        // 采样语义对齐：延迟期值 None。
        let AnimationSample { value, finished } = sample_declaration(&declaration, 1_200_000_000);
        assert!(value.is_none() && !finished);
    }

    /// 无作用域或无声明：恒等不覆盖、不活跃（flag 关回到既有 Get 行为）。
    #[test]
    fn empty_scopes_or_declarations_noop() {
        let mut paint = base_paint();
        let declarations = [opacity_declaration(1, 1000.0, 0.3, 1.0)];
        assert!(!apply(&mut paint, &declarations, 1_500_000_000));

        paint.animation_scopes = vec![scope(1, zero_protocol::animation::AnimationPropertyKind::Opacity)];
        assert!(!apply(&mut paint, &[], 1_500_000_000));
    }
}
