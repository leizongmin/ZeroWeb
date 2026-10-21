//! 合成器动画共享契约（M2-S1，goal：compositor-animation-continuity）。
//!
//! renderer 把可合成动画（opacity / 2D transform）的声明折叠为强类型键帧经 IPC
//! 交给 compositor；compositor 在 Get 帧时按共享单调钟采样并覆盖已存 paint。
//! 采样数学在本模块单一实现（renderer 静态路径与 compositor 采样路径共用，
//! 防 guidelines #6 双路径分叉）；engine 侧语义镜像自 `crates/engine/src/animation.rs`
//! （delay/fill、迭代方向奇偶、段起帧 timing 优先、异构离散切换）。
//!
//! RFC：`docs/goal/compositor-animation-continuity/m2-slice-compositor-sampling.md`。

use serde::{Deserialize, Serialize};

/// 2D transform 函数（采样域；单位：px / deg / 无量纲，镜像 CSS 函数参数）。
///
/// 仅收录 2D 可折叠子集；3D 函数（rotate3d/translate3d/matrix3d/perspective）
/// 不进入声明——renderer 导出时把含 3D 的动画标 `composable: false`。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransformFunctionValue {
    /// `translate(x, y)`。
    Translate(f64, f64),
    /// `translateX(x)`。
    TranslateX(f64),
    /// `translateY(y)`。
    TranslateY(f64),
    /// `rotate(angle)`。
    Rotate(f64),
    /// `scale(sx, sy)`。
    Scale(f64, f64),
    /// `scaleX(sx)`。
    ScaleX(f64),
    /// `scaleY(sy)`。
    ScaleY(f64),
    /// `skew(ax, ay)`。
    Skew(f64, f64),
    /// `matrix(a..ty)`（6 元 2D 仿射）。
    Matrix([f64; 6]),
}

/// 动画采样域的属性值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationPropertyValue {
    /// opacity（无量纲 [0,1]，未钳制——采样结果交给应用侧钳制，与 engine 一致）。
    Opacity(f64),
    /// transform：2D 函数列表（采样时按同构逐函数插值后折叠仿射）。
    Transform(Vec<TransformFunctionValue>),
}

/// timing function（镜像 engine `TimingFunctionValue` 全变体）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationEasing {
    /// `linear`。
    Linear,
    /// `ease`。
    Ease,
    /// `ease-in`。
    EaseIn,
    /// `ease-out`。
    EaseOut,
    /// `ease-in-out`。
    EaseInOut,
    /// `step-start`。
    StepStart,
    /// `step-end`。
    StepEnd,
    /// `cubic-bezier(x1, y1, x2, y2)`。
    CubicBezier(f64, f64, f64, f64),
    /// `steps(n, pos)`（pos 语义并入 n 的起止约定，S1 仅保留 n）。
    Steps(u32),
}

/// 动画方向（CSS Animations `animation-direction`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// `normal`。
    Normal,
    /// `reverse`。
    Reverse,
    /// `alternate`。
    Alternate,
    /// `alternate-reverse`。
    AlternateReverse,
}

/// 填充模式（CSS Animations `animation-fill-mode`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationFillMode {
    /// `none`。
    None,
    /// `forwards`。
    Forwards,
    /// `backwards`。
    Backwards,
    /// `both`。
    Both,
}

/// 键帧端点：offset ∈ [0,1] + 该端点值 + 段起帧可选 timing（段 timing 优先于全局，
/// 与 engine `KeyframePoint.timing` 一致——先选段再扭曲局部进度）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    /// 关键帧偏移 [0,1]（renderer 导出前已按 offset 排序、补齐 0%/100% 边界）。
    pub offset: f64,
    /// 该端点的采样值。
    pub value: AnimationPropertyValue,
    /// 段起帧 timing（None → 用全局 easing）。
    #[serde(default)]
    pub easing: Option<AnimationEasing>,
}

/// 动画计时（时间单位毫秒；engine 内部为秒，采样时换算）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationTiming {
    /// 单次迭代时长。
    pub duration_ms: f64,
    /// 启动延迟。
    pub delay_ms: f64,
    /// 迭代次数；None = infinite。
    pub iterations: Option<f64>,
    /// 方向。
    pub direction: AnimationDirection,
    /// 填充模式。
    pub fill: AnimationFillMode,
    /// 全局 timing function。
    pub easing: AnimationEasing,
}

/// 一条可合成动画的完整声明（renderer → compositor，随 `CompositorAnimationState` 发送）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositorAnimationDeclaration {
    /// renderer 分配的稳定 id；paint 作用域（`IpcAnimationScope`）以它绑定图元区间。
    pub animation_id: u64,
    /// 动画属性通道。
    pub property: AnimationPropertyKind,
    /// false = 含不可采样成分（3D transform、异构端点等），compositor 忽略（动画保持现状）。
    pub composable: bool,
    /// 动画启动时刻（与 compositor 同域的系统单调钟，ns）。
    pub start_monotonic_ns: u64,
    /// 计时。
    pub timing: AnimationTiming,
    /// 键帧端点列（已排序、已补界）。
    pub keyframes: Vec<AnimationKeyframe>,
}

/// 动画作用域的属性通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationPropertyKind {
    /// opacity（作用域区间内图元颜色 alpha 缩放）。
    Opacity,
    /// transform（作用域对应的 `IpcTransform` 矩阵就地改写）。
    Transform,
}

/// paint 动画作用域：一段 `draw_order` 区间 + 基线，compositor 据此把采样值
/// 应用到「动画元素的」图元上（paint 图元本身无元素身份，作用域是 renderer
/// painter 子树边界导出的唯一绑定通道）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpcAnimationScope {
    /// 绑定的动画声明 id。
    pub animation_id: u64,
    /// 属性通道。
    pub property: AnimationPropertyKind,
    /// `draw_order` 半开区间起点（含）。
    pub draw_begin: u32,
    /// `draw_order` 半开区间终点（不含）。
    pub draw_end: u32,
    /// 作用域矩形（transform 通道：恒等仿射 `IpcTransform` 的作用区域）。
    pub rect: [f32; 4],
    /// transform 通道：作用域对应 `transforms` Vec 的下标（恒等基线，compositor 就地改写）。
    #[serde(default)]
    pub transform_index: Option<u32>,
}

/// `CompositorAnimationState` 消息载荷（renderer → compositor）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositorAnimationStateParams {
    /// 页面 surface 的稳定标识。
    pub surface_id: u64,
    /// 声明所属导航世代。
    pub navigation_epoch: u64,
    /// 声明代际（同一 (surface, epoch) 内单调递增；最新生效，旧代际丢弃）。
    pub generation: u64,
    /// 当前活跃动画声明全集（每次全量替换，不用增量）。
    pub declarations: Vec<CompositorAnimationDeclaration>,
    /// 已结束动画 id（S2 结束事件契约位；S1 恒空）。
    #[serde(default)]
    pub finished: Vec<u64>,
}

// ── 采样数学（单一实现） ────────────────────────────────────────────────

/// 一次采样的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationSample {
    /// 当前时刻的覆盖值；None = 不产生覆盖（延迟期且无 backwards fill、
    /// 或已结束且无 forwards fill——与 engine `tick` 的分支一致）。
    pub value: Option<AnimationPropertyValue>,
    /// 动画是否已结束（有限迭代且播完；infinite 恒 false）。
    pub finished: bool,
}

/// 按共享单调钟采样一条声明（`now_monotonic_ns` 与 `start_monotonic_ns` 同域）。
pub fn sample_declaration(declaration: &CompositorAnimationDeclaration, now_monotonic_ns: u64) -> AnimationSample {
    if !declaration.composable || declaration.keyframes.is_empty() {
        return AnimationSample {
            value: None,
            finished: false,
        };
    }
    let timing = &declaration.timing;
    let start_s = declaration.start_monotonic_ns as f64 / 1e9;
    let now_s = now_monotonic_ns as f64 / 1e9;
    let elapsed = now_s - start_s;
    let active_elapsed = elapsed - timing.delay_ms / 1000.0;

    // 延迟期：backwards/both 用首帧值，否则无覆盖（engine tick :986-1000）。
    if active_elapsed < 0.0 {
        return AnimationSample {
            value: matches!(timing.fill, AnimationFillMode::Backwards | AnimationFillMode::Both)
                .then(|| declaration.keyframes[0].value.clone()),
            finished: false,
        };
    }

    let duration_s = timing.duration_ms / 1000.0;
    if duration_s <= 0.0 {
        // 时长非正（除法保护）：视为瞬时完成——engine 侧 total_iterations=∞ ≥ max_iter
        // 走结束分支，终帧方向按迭代奇偶（`final_animation_progress`）。
        let max_iter = timing.iterations.unwrap_or(1.0);
        return AnimationSample {
            value: Some(endpoint_value(
                declaration,
                final_direction_reversed(max_iter, &timing.direction),
            )),
            finished: true,
        };
    }

    let total_iterations = active_elapsed / duration_s;
    let Some(max_iter) = timing.iterations else {
        // infinite：永不结束，按迭代内进度采样。
        return AnimationSample {
            value: Some(sample_in_flight(
                declaration,
                total_iterations.fract(),
                total_iterations as u64,
            )),
            finished: false,
        };
    };

    if total_iterations >= max_iter {
        // 已结束：forwards/both 保持终帧（方向由最终迭代奇偶决定），否则无覆盖。
        return AnimationSample {
            value: matches!(timing.fill, AnimationFillMode::Forwards | AnimationFillMode::Both)
                .then(|| endpoint_value(declaration, final_direction_reversed(max_iter, &timing.direction))),
            finished: true,
        };
    }

    // 活跃中：迭代内进度；整数边界（fract==0 且 total>0）取 1.0（engine tick :1056-1062）。
    let iteration = total_iterations as u64;
    let mut iter_progress = total_iterations.fract();
    if iter_progress == 0.0 && total_iterations > 0.0 {
        iter_progress = 1.0;
    }
    AnimationSample {
        value: Some(sample_in_flight(declaration, iter_progress, iteration)),
        finished: false,
    }
}

/// 终帧方向：finite 结束时按最后一轮迭代奇偶决定取 0% 或 100%（engine `final_animation_progress`）。
fn final_direction_reversed(max_iter: f64, direction: &AnimationDirection) -> bool {
    let last_iteration = (max_iter.ceil() as u64).saturating_sub(1);
    matches!(direction, AnimationDirection::Reverse)
        || matches!(direction, AnimationDirection::Alternate) && last_iteration % 2 == 1
        || matches!(direction, AnimationDirection::AlternateReverse) && last_iteration.is_multiple_of(2)
}

/// 终点帧值（`reversed` = 取 0% 帧，否则 100% 帧；单帧关键帧直接取该帧）。
fn endpoint_value(declaration: &CompositorAnimationDeclaration, reversed: bool) -> AnimationPropertyValue {
    if declaration.keyframes.len() == 1 {
        return declaration.keyframes[0].value.clone();
    }
    if reversed {
        declaration.keyframes[0].value.clone()
    } else {
        declaration.keyframes[declaration.keyframes.len() - 1].value.clone()
    }
}

/// 活跃期采样：方向映射 → 选段 → 段 timing 扭曲局部进度 → 端点插值。
fn sample_in_flight(
    declaration: &CompositorAnimationDeclaration,
    iter_progress: f64,
    iteration: u64,
) -> AnimationPropertyValue {
    let directed = apply_direction(iter_progress, iteration, &declaration.timing.direction);
    interpolate_keyframes(&declaration.keyframes, directed, &declaration.timing.easing)
}

/// 方向映射（镜像 engine `apply_direction`；`iteration` 为当前迭代序号，0 起）。
fn apply_direction(progress: f64, iteration: u64, direction: &AnimationDirection) -> f64 {
    match direction {
        AnimationDirection::Normal => progress,
        AnimationDirection::Reverse => 1.0 - progress,
        AnimationDirection::Alternate => {
            if iteration % 2 == 1 {
                1.0 - progress
            } else {
                progress
            }
        }
        AnimationDirection::AlternateReverse => {
            if iteration.is_multiple_of(2) {
                1.0 - progress
            } else {
                progress
            }
        }
    }
}

/// 选段并插值（镜像 engine `interpolate_keyframes` + `interpolate_between_with_timing`）：
/// 取最后一个 `offset <= progress` 的端点为段起，段内归一化后按段起 timing（缺省全局）扭曲。
fn interpolate_keyframes(
    keyframes: &[AnimationKeyframe],
    progress: f64,
    global_easing: &AnimationEasing,
) -> AnimationPropertyValue {
    if keyframes.len() == 1 {
        return keyframes[0].value.clone();
    }
    let mut from_idx = 0;
    for (i, keyframe) in keyframes.iter().enumerate() {
        if keyframe.offset <= progress {
            from_idx = i;
        } else {
            break;
        }
    }
    let to_idx = (from_idx + 1).min(keyframes.len() - 1);
    let from = &keyframes[from_idx];
    let to = &keyframes[to_idx];
    let span = to.offset - from.offset;
    if span.abs() < 1e-9 {
        return to.value.clone();
    }
    let normalized = ((progress - from.offset) / span).clamp(0.0, 1.0);
    let local = apply_easing(normalized, from.easing.unwrap_or(*global_easing));
    interpolate_value(&from.value, &to.value, local)
}

/// timing function 求值（镜像 engine `apply_timing_function`，含简化 bezier 求解）。
fn apply_easing(t: f64, easing: AnimationEasing) -> f64 {
    match easing {
        AnimationEasing::Linear => t,
        AnimationEasing::Ease => cubic_bezier(t, 0.25, 0.1, 0.25, 1.0),
        AnimationEasing::EaseIn => cubic_bezier(t, 0.42, 0.0, 1.0, 1.0),
        AnimationEasing::EaseOut => cubic_bezier(t, 0.0, 0.0, 0.58, 1.0),
        AnimationEasing::EaseInOut => cubic_bezier(t, 0.42, 0.0, 0.58, 1.0),
        AnimationEasing::StepStart => {
            if t > 0.0 {
                1.0
            } else {
                0.0
            }
        }
        AnimationEasing::StepEnd => {
            if t >= 1.0 {
                1.0
            } else {
                0.0
            }
        }
        AnimationEasing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, x1, y1, x2, y2),
        AnimationEasing::Steps(n) => {
            let n = f64::from(n.max(1));
            (t * n).floor() / n
        }
    }
}

/// 三次贝塞尔求值（与 engine `cubic_bezier` 同法：二分逼近 bezier_x(t)=x 后取 y）。
fn cubic_bezier(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;
    let cy = 3.0 * y1;
    let by = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by;

    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    let mut t = x;
    for _ in 0..20 {
        let sample_x = ((ax * t + bx) * t + cx) * t;
        if (sample_x - x).abs() < 1e-6 {
            break;
        }
        if sample_x < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) / 2.0;
    }
    ((ay * t + by) * t + cy) * t
}

/// 端点值插值：opacity 线性；transform 同构逐函数线性、异构在 t>0.5 离散切换、
/// None 端按恒等展开（镜像 engine `transform_interp` 语义）。
fn interpolate_value(from: &AnimationPropertyValue, to: &AnimationPropertyValue, t: f64) -> AnimationPropertyValue {
    match (from, to) {
        (AnimationPropertyValue::Opacity(a), AnimationPropertyValue::Opacity(b)) => {
            AnimationPropertyValue::Opacity(a + (b - a) * t)
        }
        (AnimationPropertyValue::Transform(a), AnimationPropertyValue::Transform(b)) => {
            AnimationPropertyValue::Transform(lerp_transform_lists(a, b, t))
        }
        // 值域不同（opacity↔transform 声明不会发生；防御性离散切换与 engine 未知属性一致）。
        _ => {
            if t > 0.5 {
                to.clone()
            } else {
                from.clone()
            }
        }
    }
}

/// transform 函数列表插值：长度一致且逐位同变体 → 逐函数线性；
/// 否则 t>0.5 离散切换（镜像 engine `transform_interp` 异构离散语义）。
fn lerp_transform_lists(
    from: &[TransformFunctionValue],
    to: &[TransformFunctionValue],
    t: f64,
) -> Vec<TransformFunctionValue> {
    if from.len() == to.len() && from.iter().zip(to.iter()).all(|(a, b)| discriminant_matches(a, b)) {
        return from
            .iter()
            .zip(to.iter())
            .map(|(a, b)| lerp_function(a, b, t))
            .collect();
    }
    if t > 0.5 { to.to_vec() } else { from.to_vec() }
}

/// 同变体判定（变体名一致即视为同构；参数差异交逐函数插值）。
fn discriminant_matches(a: &TransformFunctionValue, b: &TransformFunctionValue) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// 单个 transform 函数的参数线性插值（参数域：px / deg / 无量纲）。
fn lerp_function(from: &TransformFunctionValue, to: &TransformFunctionValue, t: f64) -> TransformFunctionValue {
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    match (from, to) {
        (TransformFunctionValue::Translate(ax, ay), TransformFunctionValue::Translate(bx, by)) => {
            TransformFunctionValue::Translate(lerp(*ax, *bx), lerp(*ay, *by))
        }
        (TransformFunctionValue::TranslateX(a), TransformFunctionValue::TranslateX(b)) => {
            TransformFunctionValue::TranslateX(lerp(*a, *b))
        }
        (TransformFunctionValue::TranslateY(a), TransformFunctionValue::TranslateY(b)) => {
            TransformFunctionValue::TranslateY(lerp(*a, *b))
        }
        (TransformFunctionValue::Rotate(a), TransformFunctionValue::Rotate(b)) => {
            TransformFunctionValue::Rotate(lerp(*a, *b))
        }
        (TransformFunctionValue::Scale(ax, ay), TransformFunctionValue::Scale(bx, by)) => {
            TransformFunctionValue::Scale(lerp(*ax, *bx), lerp(*ay, *by))
        }
        (TransformFunctionValue::ScaleX(a), TransformFunctionValue::ScaleX(b)) => {
            TransformFunctionValue::ScaleX(lerp(*a, *b))
        }
        (TransformFunctionValue::ScaleY(a), TransformFunctionValue::ScaleY(b)) => {
            TransformFunctionValue::ScaleY(lerp(*a, *b))
        }
        (TransformFunctionValue::Skew(ax, ay), TransformFunctionValue::Skew(bx, by)) => {
            TransformFunctionValue::Skew(lerp(*ax, *bx), lerp(*ay, *by))
        }
        (TransformFunctionValue::Matrix(a), TransformFunctionValue::Matrix(b)) => {
            let mut out = [0.0_f64; 6];
            for (i, value) in out.iter_mut().enumerate() {
                *value = lerp(a[i], b[i]);
            }
            TransformFunctionValue::Matrix(out)
        }
        // 同 discriminant 下不可达；防御性保持 from。
        _ => *from,
    }
}

/// 把 2D transform 函数列表折叠为 `[a, b, c, d, tx, ty]` 仿射（列向量约定，
/// 与 engine `compute_transform_matrix` 2D 折叠同序：按列表顺序依次相乘）。
/// 遇到本模块外的成分（不应出现——3D 在导出侧已判 `composable: false`）返回 None。
pub fn fold_affine(functions: &[TransformFunctionValue]) -> Option<[f64; 6]> {
    let mut m = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    for function in functions {
        let next = match function {
            TransformFunctionValue::Translate(x, y) => [1.0, 0.0, 0.0, 1.0, *x, *y],
            TransformFunctionValue::TranslateX(x) => [1.0, 0.0, 0.0, 1.0, *x, 0.0],
            TransformFunctionValue::TranslateY(y) => [1.0, 0.0, 0.0, 1.0, 0.0, *y],
            TransformFunctionValue::Rotate(deg) => {
                let (sin, cos) = deg.to_radians().sin_cos();
                [cos, sin, -sin, cos, 0.0, 0.0]
            }
            TransformFunctionValue::Scale(x, y) => [*x, 0.0, 0.0, *y, 0.0, 0.0],
            TransformFunctionValue::ScaleX(x) => [*x, 0.0, 0.0, 1.0, 0.0, 0.0],
            TransformFunctionValue::ScaleY(y) => [1.0, 0.0, 0.0, *y, 0.0, 0.0],
            TransformFunctionValue::Skew(ax, ay) => [1.0, ay.to_radians().tan(), ax.to_radians().tan(), 1.0, 0.0, 0.0],
            TransformFunctionValue::Matrix(m2) => *m2,
        };
        m = multiply_affine(m, next);
    }
    Some(m)
}

/// 仿射相乘 `a · b`（先应用 b 再应用 a；`[a, b, c, d, tx, ty]` 列向量约定）。
#[must_use]
pub fn multiply_affine(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_opacity_decl(duration_ms: f64, from: f64, to: f64) -> CompositorAnimationDeclaration {
        CompositorAnimationDeclaration {
            animation_id: 1,
            property: AnimationPropertyKind::Opacity,
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

    #[test]
    fn opacity_linear_progress_and_fill_semantics() {
        let decl = linear_opacity_decl(1000.0, 1.0, 0.0);
        // t=0 → 起点；半程 → 0.5。
        assert_eq!(
            sample_declaration(&decl, 1_000_000_000).value,
            Some(AnimationPropertyValue::Opacity(1.0))
        );
        assert_eq!(
            sample_declaration(&decl, 1_500_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.5))
        );
        // 结束且 fill none → 无覆盖，finished=true。
        let sample = sample_declaration(&decl, 2_100_000_000);
        assert_eq!(sample.value, None);
        assert!(sample.finished);
        // fill forwards → 保持末帧。
        let mut filled = decl.clone();
        filled.timing.fill = AnimationFillMode::Forwards;
        assert_eq!(
            sample_declaration(&filled, 2_100_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.0))
        );
        // 延迟期 fill backwards → 首帧；无 fill → 无覆盖。
        let mut delayed = decl.clone();
        delayed.timing.delay_ms = 500.0;
        assert_eq!(sample_declaration(&delayed, 1_200_000_000).value, None);
        delayed.timing.fill = AnimationFillMode::Backwards;
        assert_eq!(
            sample_declaration(&delayed, 1_200_000_000).value,
            Some(AnimationPropertyValue::Opacity(1.0))
        );
    }

    #[test]
    fn alternate_direction_flips_per_iteration() {
        let mut decl = linear_opacity_decl(1000.0, 0.0, 1.0);
        decl.timing.iterations = None;
        decl.timing.direction = AnimationDirection::Alternate;
        // 第 0 轮（偶）正向：t=0.5 → 0.5；第 1 轮（奇）反向：同一相位 → 0.5（对称值不同侧）。
        assert_eq!(
            sample_declaration(&decl, 1_500_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.5))
        );
        assert_eq!(
            sample_declaration(&decl, 2_500_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.5))
        );
        // 第 1 轮相位 0.25（wall 2.25s）反向 → 0.75。
        assert_eq!(
            sample_declaration(&decl, 2_250_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.75))
        );
    }

    #[test]
    fn rotate_list_lerps_per_function() {
        let decl = CompositorAnimationDeclaration {
            animation_id: 2,
            property: AnimationPropertyKind::Transform,
            composable: true,
            start_monotonic_ns: 0,
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
                    value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Rotate(0.0)]),
                    easing: None,
                },
                AnimationKeyframe {
                    offset: 1.0,
                    value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Rotate(90.0)]),
                    easing: None,
                },
            ],
        };
        let sample = sample_declaration(&decl, 500_000_000).value;
        let Some(AnimationPropertyValue::Transform(functions)) = sample else {
            panic!("期望 transform 采样值");
        };
        assert_eq!(functions, vec![TransformFunctionValue::Rotate(45.0)]);
        // 折叠仿射：rotate 45°。
        let affine = fold_affine(&functions).unwrap();
        let expected = std::f64::consts::FRAC_1_SQRT_2;
        assert!((affine[0] - expected).abs() < 1e-9 && (affine[1] - expected).abs() < 1e-9);
    }

    #[test]
    fn non_composable_and_instant_decl_degrade() {
        let mut decl = linear_opacity_decl(1000.0, 1.0, 0.0);
        decl.composable = false;
        assert_eq!(sample_declaration(&decl, 1_500_000_000).value, None);
        // 时长为 0：瞬时完成给末帧。
        let instant = linear_opacity_decl(0.0, 1.0, 0.25);
        let sample = sample_declaration(&instant, 1_000_000_000);
        assert!(sample.finished);
        assert_eq!(sample.value, Some(AnimationPropertyValue::Opacity(0.25)));
    }

    #[test]
    fn easing_segment_overrides_global() {
        let mut decl = linear_opacity_decl(1000.0, 0.0, 1.0);
        decl.timing.easing = AnimationEasing::Linear;
        // 段起帧 step-end：半程前保持 0。
        decl.keyframes[0].easing = Some(AnimationEasing::StepEnd);
        assert_eq!(
            sample_declaration(&decl, 400_000_000 + 1_000_000_000).value,
            Some(AnimationPropertyValue::Opacity(0.0))
        );
        // 注意：start_monotonic_ns=1e9，这里 t=1.4s → 进度 0.4 → step-end → 0。
    }

    #[test]
    fn serialize_roundtrip_declaration_and_scope() {
        let declaration = linear_opacity_decl(1000.0, 1.0, 0.0);
        let params = CompositorAnimationStateParams {
            surface_id: 7,
            navigation_epoch: 2,
            generation: 9,
            declarations: vec![declaration],
            finished: vec![],
        };
        let bytes = crate::serialize::serialize(&crate::message::IpcMessage {
            id: 1,
            kind: crate::message::IpcMessageKind::CompositorAnimationState(params.clone()),
        })
        .expect("序列化失败");
        let decoded = crate::serialize::deserialize(&bytes).expect("反序列化失败");
        match decoded.kind {
            crate::message::IpcMessageKind::CompositorAnimationState(out) => assert_eq!(out, params),
            other => panic!("消息类型不符: {other:?}"),
        }
    }
}
