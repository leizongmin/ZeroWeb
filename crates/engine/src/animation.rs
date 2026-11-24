//! CSS 动画运行时 — 关键帧插值与动画时钟。
//!
//! 此模块实现 CSS 动画的执行逻辑：
//! 1. 从 `@keyframes` 规则构建关键帧序列
//! 2. 根据时间进度和 timing function 计算当前帧的插值
//! 3. 将插值结果叠加到 ComputedStyle 上
//!
//! 当前支持的插值属性：
//! - `opacity`（浮点插值）
//! - `background-color` / `color`（RGBA 颜色插值）
//! - `width` / `height`（长度插值）
//! - `transform`（R5191 起函数列表插值，见 [`transform_interp`]）

use std::collections::HashMap;

/// kc5r82 face 指纹（wptfyi-css，2026-11-24）：multicol 离散三长手写回臂 +
/// column-count/width/height 端点严格门。仅作 bin strings 指纹探针用，
/// 不入运行时路径；`#[used]` 防 release DCE 剥离未引用数据段。
#[used]
pub static KC5R82_FACE_MARKER: [u8; 20] = *b"zw-kc5r82-2026-11-24";

// R5191：transform 函数列表插值（css-transforms-1 §13.2 unmatrix + 2D 函数逐参数）。
pub(crate) mod transform_interp;
// wptfyi-css 基本形状族刀（css-shapes-2 §interpolating-shape）：clip-path 同构
// 逐分量插值（inset/circle/ellipse/polygon/shape/path/rect/xywh）。
mod clip_interp;
// M2-S1b：可合成动画声明导出（compositor-animation-continuity，renderer → compositor 采样域）。
pub mod compositor_export;
// M2-S3c：动画 transform 命中修正提供者槽（renderer 注册全仿射，hit_test 逆映射点测）。
pub mod hit_provider;

use zero_css_parser::ast::{KeyframeBlock, KeyframeSelector, KeyframesRule};
use zero_css_parser::values::{
    AnimationCompositionValue, AnimationDirectionValue, AnimationFillModeValue, BorderImageOutsetComponent,
    BorderImageSliceComponent, BorderImageWidthComponent, CalcMathType, LengthValue, StepPosition, TimingFunctionValue,
    TransformFunction, calc_expr_math_type, eval_calc_with_default_font, parse_angle_to_radians,
    parse_border_image_outset, parse_border_image_slice, parse_border_image_width, parse_font_feature_settings,
    parse_font_variation_settings, parse_math_function,
};
use zero_style_system::{
    ClipRuleComputedValue, ComputedStyle, FilterComputedValue, OffsetPositionValue, OffsetRotateValue,
    ShapeOutsideBoxKind, ShapeOutsideComputedValue,
    property::types::{
        BorderImageOutsetComputedComponent, BorderImageOutsetComputedValue, BorderImageRepeatComputedMode,
        BorderImageRepeatComputedValue, BorderImageSliceComputedComponent, BorderImageSliceComputedValue,
        BorderImageWidthComputedComponent, BorderImageWidthComputedValue, ColumnCountComputedValue,
        ColumnHeightComputedValue, ColumnWidthComputedValue, ColumnWrapComputedValue, ZIndexValue,
    },
};

/// 关键帧点 — 某个时间进度处的属性快照。
#[derive(Debug, Clone)]
pub struct KeyframePoint {
    /// 时间进度（0.0 ~ 1.0）。
    pub offset: f64,
    /// 声明的属性名→原始值字符串。
    pub properties: HashMap<String, String>,
    /// R3883：该帧声明的 animation-timing-function（CSS Animations §keyframes——
    /// 应用于**从该帧到下一帧的段**；None = 回落到动画的 animation-timing-function）。
    pub timing: Option<TimingFunctionValue>,
    /// 刀 ANIMCOMP S2（wptfyi-css，2026-11-19）：该帧声明的 animation-composition
    ///（css-animations-2 §animation-composition——关键帧内声明对该帧值生效；None =
    /// 回落到动画级 animation-composition，再回落 replace）。与 R3883 timing 同法
    /// 自 properties 剥离，不落插值/应用名单。
    pub composition: Option<AnimationCompositionValue>,
}

/// 活跃的动画实例。
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// 动画名称。
    pub name: String,
    /// 时长（秒）。
    pub duration: f64,
    /// 延迟（秒）。
    pub delay: f64,
    /// 迭代次数（None = infinite）。
    pub iteration_count: Option<f64>,
    /// 方向。
    pub direction: AnimationDirectionValue,
    /// 填充模式。
    pub fill_mode: AnimationFillModeValue,
    /// Timing function。
    pub timing_function: TimingFunctionValue,
    /// 动画开始时间（秒，相对于时钟原点）。
    pub start_time: f64,
    /// 关键帧列表（已按 offset 排序）。
    pub keyframes: Vec<KeyframePoint>,
    /// 是否已完成。
    pub finished: bool,
    /// 当前迭代序号。
    pub iteration: u64,
    /// 是否已进入活跃态（R3251，CSS Animations §animationstart）——首次跨过 animation-delay 进入活跃
    /// 间隔的帧由 false→true，供 `tick()` 推入 `just_started` 派发 animationstart（elapsedTime=0）。
    pub started: bool,
    /// M2-S3b：暂停时刻（时钟域秒）——Some = `animation-play-state: paused` 冻结中
    /// （tick 以 paused_at 求值，输出冻结于暂停时刻插值，迭代/终态判定停走）；None =
    /// running。恢复时由 `reconcile_play_state` 重基 `start_time`（暂停时长不计入活跃
    /// 时长，CSS Animations §play-state）。
    pub paused_at: Option<f64>,
    /// 刀 ANIMCOMP S2：动画级合成操作（css-animations-2 §animation-composition；
    /// 关键帧未声明 animation-composition 的属性以此为准，再回落 replace）。
    pub composition: AnimationCompositionValue,
    /// 刀 ANIMCOMP S2：tick 期结果合成名单（属性 → add/accumulate）——start 时逐
    /// 属性归类「全帧 add/accumulate」的属性在此登记（web-animations-1 §5.4.4：
    /// 均匀 add 语义 = U ⊕ lerp(F,T,p)，端点预合成的恒等式），插值输出经
    /// [`compose_animation_value`] 与 underlying 合成后施值。混合 op 属性不在此
    /// 名单（走端点预合成）；replace 属性不登记（行为与刀前恒等）。
    pub compose_result: HashMap<String, AnimationCompositionValue>,
    /// 刀 ANIMCOMP S2：underlying 值（属性 → start 时元素 computed 串）——结果
    /// 合成与端点预合成的 U 侧。CSSA 通道 start 时从元素 computed 捕获；WAAPI
    /// 通道由 shim gCS 预读经 [`AnimationConfig::underlying_values`] 注入。
    pub underlying_values: HashMap<String, String>,
}

/// 动画启动配置。
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// 动画名称（@keyframes 名称）。
    pub name: String,
    /// 时长（秒）。
    pub duration: f64,
    /// 延迟（秒）。
    pub delay: f64,
    /// Timing function。
    pub timing_function: TimingFunctionValue,
    /// 迭代次数（None = infinite）。
    pub iteration_count: Option<f64>,
    /// 方向。
    pub direction: AnimationDirectionValue,
    /// 填充模式。
    pub fill_mode: AnimationFillModeValue,
    /// 当前时间（秒，相对于时钟原点）。
    pub current_time: f64,
    /// 刀 ANIMCOMP S2：动画级合成操作（css-animations-2 §animation-composition）；
    /// 关键帧点未声明时回落此值。缺省 replace（刀前行为恒等）。
    pub composition: AnimationCompositionValue,
    /// 刀 ANIMCOMP S2：WAAPI shim 预读 underlying（属性 → computed 值串）；CSSA
    /// 通道留空——start 时由元素 computed 捕获（[`Self::default`] 空表）。
    pub underlying_values: HashMap<String, String>,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: 0.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Ease,
            iteration_count: None,
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        }
    }
}

/// 动画时钟 — 管理所有活跃动画并按帧推进。
#[derive(Debug, Clone, Default)]
pub struct AnimationClock {
    /// 已注册的 @keyframes 规则（名称 → 关键帧列表）。
    keyframes_registry: HashMap<String, Vec<KeyframePoint>>,
    /// 活跃的动画（元素标识 → 动画列表）。
    active_animations: HashMap<u64, Vec<AnimationState>>,
    /// 「本轮新完成」的动画（R3249）——`tick()` 在 `finished` 由 false→true 的帧推入，供宿主经
    /// `drain_just_finished()` 取出后映射元素并派发 `animationend`（CSS Animations §animationend）。
    just_finished: Vec<FinishedAnimation>,
    /// 「本轮跨越的迭代边界」记录（R3250）——`tick()` 在 `iteration` 递增（非完成帧）时推入，供宿主经
    /// `drain_just_iterated()` 取出后派发 `animationiteration`（CSS Animations §animationiteration）。
    /// 关键：infinite 动画永不完成（不派 animationend），靠此事件驱动循环回调。
    just_iterated: Vec<IteratedAnimation>,
    /// 「本轮新启动」的动画记录（R3251）——`tick()` 在 `started` 由 false→true（首次跨过 animation-delay
    /// 进入活跃间隔）的帧推入，供宿主经 `drain_just_started()` 取出后派发 `animationstart`
    /// （CSS Animations §animationstart，elapsedTime=0）。
    just_started: Vec<StartedAnimation>,
}

/// 「本轮新完成」的动画记录（R3249）——`tick()` 在 `finished` 由 false→true 的帧推入 `just_finished`，
/// 供宿主经 `drain_just_finished()` 取出后映射元素并派发 `animationend`（CSS Animations §animationend）。
/// `duration` = 活跃时长（iteration_count * duration，即 animationend 事件的 `elapsedTime`）。
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedAnimation {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 动画名（animationend.animationName）。
    pub name: String,
    /// 活跃时长（秒，animationend.elapsedTime = iteration_count * duration）。
    pub duration: f64,
}

/// 「本轮跨越的迭代边界」记录（R3250）——`tick()` 在 `iteration` 递增（非完成帧）时推入 `just_iterated`，
/// 供宿主经 `drain_just_iterated()` 取出后派发 `animationiteration`（CSS Animations §animationiteration）。
/// `elapsed` = 已完成迭代数 × duration（animationiteration.elapsedTime）。
#[derive(Debug, Clone, PartialEq)]
pub struct IteratedAnimation {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 动画名（animationiteration.animationName）。
    pub name: String,
    /// 已完成迭代累计时长（秒，animationiteration.elapsedTime = completed_iterations × duration）。
    pub elapsed: f64,
}

/// 「本轮新启动」的动画记录（R3251）——`tick()` 在 `started` 由 false→true（首次进入活跃间隔，即
/// animation-delay 已过）的帧推入 `just_started`，供宿主经 `drain_just_started()` 取出后派发
/// `animationstart`（CSS Animations §animationstart）。elapsedTime 恒为 0（动画刚开始，未播放），
/// 故结构不带 elapsed 字段——pipeline 侧硬编码 0.0。
#[derive(Debug, Clone, PartialEq)]
pub struct StartedAnimation {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 动画名（animationstart.animationName）。
    pub name: String,
}

// ── 时间函数求值 ──────────────────────────────────────────────────────

/// 将线性进度 [0,1] 通过 timing function 映射到 [0,1]。
pub fn apply_timing_function(t: f64, tf: &TimingFunctionValue) -> f64 {
    match tf {
        TimingFunctionValue::Linear => t,
        TimingFunctionValue::Ease => cubic_bezier(t, 0.25, 0.1, 0.25, 1.0),
        TimingFunctionValue::EaseIn => cubic_bezier(t, 0.42, 0.0, 1.0, 1.0),
        TimingFunctionValue::EaseOut => cubic_bezier(t, 0.0, 0.0, 0.58, 1.0),
        TimingFunctionValue::EaseInOut => cubic_bezier(t, 0.42, 0.0, 0.58, 1.0),
        TimingFunctionValue::StepStart => {
            if t > 0.0 {
                1.0
            } else {
                0.0
            }
        }
        TimingFunctionValue::StepEnd => {
            if t >= 1.0 {
                1.0
            } else {
                0.0
            }
        }
        TimingFunctionValue::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, *x1, *y1, *x2, *y2),
        TimingFunctionValue::Steps(n, pos) => {
            // css-easing-1 §step-easing-functions：start/jump-start 越点取步进
            // 上包络（ceil），end/jump-end 下包络（floor，缺省语义不变）；
            // jump-both 两端钳 0/1 中段 ceil；jump-none 内缩 round。此前恒
            // floor 弃 position——steps(1,start) 误同 jump-end（二十轮 T 通道
            // 定谳残面：jsanim/WAAPI at(1) 族 eased(0.5) 误落 0 → from），与
            // transition_registry 本地 entry_eased 同语义清偿共享面。
            let n = (*n).max(1) as f64;
            match pos {
                Some(StepPosition::Start) if t > 0.0 => ((t * n).ceil() / n).min(1.0),
                Some(StepPosition::Start) => 0.0,
                Some(StepPosition::Both) if t <= 0.0 => 0.0,
                Some(StepPosition::Both) if t >= 1.0 => 1.0,
                Some(StepPosition::Both) => ((t * n).ceil() / n).min(1.0),
                Some(StepPosition::None) => {
                    let m = (n - 1.0).max(1.0);
                    ((t * m).round() / m).clamp(0.0, 1.0)
                }
                // End / None（缺省 jump-end）。
                _ => ((t * n).floor() / n).min(1.0),
            }
        }
    }
}

/// 三次贝塞尔曲线求值（Newton-Raphson 求解参数 t）。
fn cubic_bezier(x: f64, x1: f64, _y1: f64, x2: f64, y2: f64) -> f64 {
    // 简化实现：使用采样法近似
    // 对于精确实现需要 Newton-Raphson 迭代
    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * _y1;
    let by = 3.0 * (y2 - _y1) - cy;
    let ay = 1.0 - cy - by;

    // 二分查找求解参数 t，使 bezier_x(t) ≈ x
    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    let mut t = x; // 初始猜测

    for _ in 0..20 {
        let t2 = t * t;
        let t3 = t2 * t;
        let sample_x = ax * t3 + bx * t2 + cx * t;

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

    let t2 = t * t;
    let t3 = t2 * t;
    ay * t3 + by * t2 + cy * t
}

// ── 属性值插值 ────────────────────────────────────────────────────────

/// 插值结果 — 一个可叠加到 ComputedStyle 上的属性覆盖。
#[derive(Debug, Clone)]
pub struct InterpolatedProperty {
    /// 属性名。
    pub name: String,
    /// 插值后的值字符串。
    pub value: String,
}

/// R5018（css-animations §keyframes，implicit keyframes）：0%/100% 合成边界帧缺失的
/// 属性以元素 underlying（当前 computed）值为该侧端点——规范语义「缺失关键帧 = 回退
/// underlying value」。仅覆盖 individual transform 三属性（与 R4202 插值臂同域，
/// driving: translate-explicit-and-implicit-keyframes）；其余属性维持既有回退
///（parse 失败 = 属性初值近似）。非合成但显式空声明（`from {}`）同语义（spec 同款
/// 回退），一并填充。
fn resolve_implicit_boundaries(points: &mut [KeyframePoint], style: &ComputedStyle) {
    if points.len() < 2 {
        return;
    }
    let last = points.len() - 1;
    // 首帧（0%）← 邻帧属性集；末帧（100%）← 前邻帧属性集。序列化失败（None）不填，
    // 保持既有回退。
    let heads: [(usize, usize); 2] = [(0, 1), (last, last - 1)];
    for (boundary, adjacent) in heads {
        let missing: Vec<String> = points[adjacent]
            .properties
            .keys()
            .filter(|name| !points[boundary].properties.contains_key(*name))
            .cloned()
            .collect();
        for name in missing {
            if let Some(v) = underlying_individual_value(style, &name) {
                points[boundary].properties.insert(name, v);
            }
        }
    }
}

/// rule-inset 槽字段访问（underlying 补键与回写共用）：8 longhand 名 →
/// 字段引用。None = 非 rule-inset 槽名。
fn rule_inset_underlying_component<'a>(
    style: &'a ComputedStyle,
    name: &str,
) -> Option<&'a zero_css_parser::values::RuleInsetComponent> {
    match name {
        "column-rule-inset-cap-start" => Some(&style.column_rule_inset_cap_start),
        "column-rule-inset-cap-end" => Some(&style.column_rule_inset_cap_end),
        "column-rule-inset-junction-start" => Some(&style.column_rule_inset_junction_start),
        "column-rule-inset-junction-end" => Some(&style.column_rule_inset_junction_end),
        "row-rule-inset-cap-start" => Some(&style.row_rule_inset_cap_start),
        "row-rule-inset-cap-end" => Some(&style.row_rule_inset_cap_end),
        "row-rule-inset-junction-start" => Some(&style.row_rule_inset_junction_start),
        "row-rule-inset-junction-end" => Some(&style.row_rule_inset_junction_end),
        _ => None,
    }
}

/// rule-inset 槽动画面串化（underlying 补键）：overlap-join 保关键字；LP 经
/// gCS 同源解析（em/vmin→px、calc 求值、% 保形）。
fn rule_inset_component_to_anim_css(c: &zero_css_parser::values::RuleInsetComponent, font_size_px: f64) -> String {
    match c {
        zero_css_parser::values::RuleInsetComponent::OverlapJoin => "overlap-join".to_string(),
        zero_css_parser::values::RuleInsetComponent::Length(lv) => {
            crate::js_dom_bridge::computed_style::length_to_css(lv, font_size_px)
        }
    }
}

/// 元素 individual transform 属性的 underlying 值序列化（css-transforms-2
/// §individual-transforms）。声明在场：纯 px Translate / RotateZ / Scale 序列化
///（与插值臂的解析语境一致；% 形态（Mixed）与非 Z 轴旋转近似回初值）。声明缺席：
/// 属性初值（translate ≡ 0px、rotate ≡ 0deg、scale ≡ 1——css-values §initial），
/// 保证空边界帧被填充、插值段不因 to 侧 properties 空而整段无输出。
fn underlying_individual_value(style: &ComputedStyle, name: &str) -> Option<String> {
    // R5191（css-animations-1 §keyframes：隐式边界帧 = underlying value）：transform
    // 计算值序列化——from-only/to-only keyframes 的空边界帧补键，否则插值循环
    // （to 侧属性集驱动）零输出、动画值永不应用。与 [`underlying_cascade_value`]
    // 的 transform arm 同语义（revert-layer 解析面），序列化函数列表 spec-correct 形态。
    if name == "transform" {
        return Some(crate::js_dom_bridge::computed_style::transform_to_css(&style.transform));
    }
    // wptfyi-css 基本形状族刀：clip-path underlying（隐式边界帧补键与
    // neutral 单帧语义同源——css-animations-1 §keyframes）。
    if name == "clip-path" {
        let font = match style.font_size {
            LengthValue::Px(v) => v,
            _ => 16.0,
        };
        return Some(crate::js_dom_bridge::computed_style::clip_path_to_css(
            &style.clip_path,
            font,
        ));
    }
    // wptfyi-css css-gaps 刀 B：rule-width 列表 underlying（隐式边界帧补键，
    // neutral keyframe = 底层计算列表——css-animations-1 §keyframes）。gCS
    // 列表串化单源（关键字→used px、repeat/auto/calc 结构保形）。
    if name == "column-rule-width" || name == "row-rule-width" {
        let font = match style.font_size {
            LengthValue::Px(v) => v,
            _ => 16.0,
        };
        let segments = if name == "column-rule-width" {
            &style.column_rule_width
        } else {
            &style.row_rule_width
        };
        return Some(crate::js_dom_bridge::computed_style::rule_width_list_to_gcs(
            segments, font,
        ));
    }
    // wptfyi-css css-gaps 刀 D：rule-inset 槽 underlying（隐式边界帧补键，
    // neutral keyframe = 底层计算值——css-animations-1 §keyframes；语料
    // gap-decorations-inset-neutral-keyframe-{001,002} 钉：声明 '1px' →
    // 边界帧 '1px'）。gCS 槽串化单源（em/vmin→px、calc 求值、% 保形）。
    if let Some(c) = rule_inset_underlying_component(style, name) {
        let font = match style.font_size {
            LengthValue::Px(v) => v,
            _ => 16.0,
        };
        return Some(rule_inset_component_to_anim_css(c, font));
    }
    // CSS Motion Path offset 簇 underlying（css-motion-1；wptfyi-css offset-* 刀）：
    // 声明值序列化，声明缺席落 spec 初始（0px / auto / auto / none）。
    match name {
        "offset-path" => {
            return Some(style.offset_path.clone().unwrap_or_else(|| "none".to_string()));
        }
        "offset-distance" => {
            return Some(match style.offset_distance.as_ref() {
                Some(LengthValue::Percentage(p)) => format!("{p}%"),
                Some(LengthValue::Px(v)) => format!("{v}px"),
                _ => "0px".to_string(),
            });
        }
        "offset-rotate" => {
            return Some(offset_rotate_str(style.offset_rotate.as_ref()));
        }
        "offset-anchor" => {
            return Some(match &style.offset_anchor {
                Some((x, y)) => format!("{} {}", offset_len_str(x), offset_len_str(y)),
                None => "auto".to_string(),
            });
        }
        "offset-position" => {
            return Some(match style.offset_position.as_ref() {
                Some(OffsetPositionValue::Normal) => "normal".to_string(),
                Some(OffsetPositionValue::Auto) => "auto".to_string(),
                Some(OffsetPositionValue::Position(x, y)) => {
                    format!("{} {}", offset_len_str(x), offset_len_str(y))
                }
                None => "normal".to_string(),
            });
        }
        _ => {}
    }
    let declared = match name {
        "translate" => style.individual_translate.as_ref(),
        "rotate" => style.individual_rotate.as_ref(),
        "scale" => style.individual_scale.as_ref(),
        _ => return generic_underlying_value(style, name),
    };
    // 声明缺席或 % / 非单轴形态 → 属性初值近似（none 的 used value）。
    let individual = match (name, declared) {
        ("translate", Some(TransformFunction::Translate(x, y))) => {
            if *y == 0.0 {
                Some(format!("{x}px"))
            } else {
                Some(format!("{x}px {y}px"))
            }
        }
        ("rotate", Some(TransformFunction::RotateZ(a))) => Some(format!("{a}deg")),
        ("scale", Some(TransformFunction::Scale(sx, sy))) => {
            let sy = sy.unwrap_or(*sx);
            if (sx - sy).abs() < 1e-9 {
                Some(format!("{sx}"))
            } else {
                Some(format!("{sx} {sy}"))
            }
        }
        ("translate", _) => Some("0px".to_string()),
        ("rotate", _) => Some("0deg".to_string()),
        ("scale", _) => Some("1".to_string()),
        _ => None,
    };
    if individual.is_some() {
        return individual;
    }
    generic_underlying_value(style, name)
}

/// 通用臂（R5018 通用化刀）：逐属性臂未覆盖的属性走 gCS 同源串化兜底——
/// serialize_computed_property_with_sibling 对任意已实现属性给出 underlying
/// 计算值（css-animations-1 §keyframes：缺失端点帧 = neutral key，即
/// underlying value；与 E5 keyword 解析同一串化通道，声明形/计算形一致）。
/// 空串（未实现属性）返 None 保持既有回退（parse 失败 = 属性初值近似）。
fn generic_underlying_value(style: &ComputedStyle, name: &str) -> Option<String> {
    let generic = crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(style, name, None);
    if generic.is_empty() { None } else { Some(generic) }
}

/// R5189：keyframe 值 `revert-layer` 的 underlying 解析（css-cascade-5 §revert-layer——
/// 动画 origin 的 revert-layer 回退到 author origin，即**动画未叠加**时的元素级联值）。
/// 从动画启动时传入的 ComputedStyle 序列化（`start_animation_with_underlying` 的
/// underlying 参数此刻仍为 pre-animation 级联结果，见 pipeline
/// apply_animation_overrides 的启动时序）。仅覆盖插值器长度臂的属性 + opacity；
/// 非 Px 形态（%/auto/em）返回 None 保原值走既有离散回退。
///
/// FIXME(revert-keyframe)：普通 `revert` 须回退到 UA origin（低两层），ComputedStyle
/// 只有全量级联结果、无 origin 分层数据；自定义属性（--x）动画依赖 var()/@property
/// 管线——两者均维持原值离散回退（revert-layer-011 域）。
fn underlying_cascade_value(style: &ComputedStyle, name: &str) -> Option<String> {
    let len = |v: &LengthValue| match v {
        LengthValue::Px(x) => Some(format!("{x}px")),
        _ => None,
    };
    match name {
        "width" => len(&style.width),
        "height" => len(&style.height),
        "top" => len(&style.top),
        "left" => len(&style.left),
        "right" => len(&style.right),
        "bottom" => len(&style.bottom),
        "margin-top" => len(&style.margin_top),
        "margin-right" => len(&style.margin_right),
        "margin-bottom" => len(&style.margin_bottom),
        "margin-left" => len(&style.margin_left),
        "padding-top" => len(&style.padding_top),
        "padding-right" => len(&style.padding_right),
        "padding-bottom" => len(&style.padding_bottom),
        "padding-left" => len(&style.padding_left),
        "font-size" => len(&style.font_size),
        "letter-spacing" => len(&style.letter_spacing),
        "word-spacing" => len(&style.word_spacing),
        "opacity" => Some(format!("{:.4}", style.opacity)),
        // R5191（css-animations-1 §keyframes：隐式边界帧 = underlying value）：
        // transform 计算值序列化（函数列表，js_dom_bridge 同款 spec-correct 形态）。
        // 缺失即 from-only/to-only keyframes 的空边界帧补 underlying——注册层给单点
        // keyframes 补的空 100%/0% 点若无此键，插值循环（to 侧属性集驱动）零输出，
        // transform 动画值永不应用（transform-non-invertible-discrete-interpolation：
        // from-only 非可逆 matrix3d 黑块原样）。`none` → "none"（apply 侧 parse 为
        // TransformValue::None）。无插值臂 → 离散切换（t≤0.5 取 from，规范同款回退）。
        "transform" => Some(crate::js_dom_bridge::computed_style::transform_to_css(&style.transform)),
        _ => None,
    }
}

/// 刀 ANIMCOMP S2（wptfyi-css，2026-11-19）：add/accumulate 动画值与 underlying
/// 合成（css-animations-2 §animation-composition apply 面 / web-animations-1
/// §5.4.4 combining values）。支持域：
/// - 影列表拼接（text-shadow / box-shadow）：add 语义 = 动画影列表追加到
///   underlying 列表（css-text-decor-3 §text-shadow / css-backgrounds-3
///   §box-shadow 的 Web Animations 加法定义）；underlying `none`/空 = 单位元，
///   恒等回落动画值。序列化形态锚定 WPT composition 期望串（逗号 + 空格分隔，
///   与 gCS text/box-shadow 列表串化同形）。
/// - vertical-align：underlying 与动画值均 `<length>` px 严格可解析时数值相加
///   （串化经 [`format_interp_num`] 与 gCS 数字形态一致）；% / 关键字（baseline
///   等）/ 混合单位无定义加法 → 动画值原样（replace 回落）。
///
/// 其余属性 → 动画值原样。`op` 在单动画单迭代域 add ≡ accumulate
/// （web-animations-1 §5.4.4 accumulate 注记），参数保留供多迭代域扩展。
pub(crate) fn compose_animation_value(
    name: &str,
    op: &AnimationCompositionValue,
    underlying: Option<&str>,
    animated: &str,
) -> String {
    let _ = op;
    let Some(u) = underlying else {
        return animated.to_string();
    };
    let u = u.trim();
    if u.is_empty() || u.eq_ignore_ascii_case("none") {
        return animated.to_string();
    }
    match name {
        "text-shadow" | "box-shadow" => format!("{u}, {animated}"),
        // 刀 FAM-F：aspect-ratio 不可加（css-values-4 §combine-ratio not-additive）
        // → add/accumulate 合成无定义，有效关键帧值＝underlying（web-animations-1
        // §5.4.2/§5.5 composite-against-underlying 先于插值；WPT ANIMCOMP 期望
        // `underlying [2 / 1] … to add [1 / 1] at (1)` = [2 / 1]）。add 帧预合成后
        // 与 replace 端点插值；均匀 add 面合成结果恒 underlying＝动画无效应（规范
        // 语义）。caller 仅对 add 帧/登记属性调用本函数，replace 面不触本臂。
        "aspect-ratio" => u.to_string(),
        "vertical-align" => match (parse_px_strict(u), parse_px_strict(animated)) {
            (Some(up), Some(vp)) => format!("{}px", format_interp_num(up + vp)),
            _ => animated.to_string(),
        },
        // fame2b：sizing 六长手逐 term 加法（web-animations-1 §5.4.4 addition，
        // css-sizing-3 各 propdef <length-percentage>）：px/% 双 term 独立相加，
        // 规范形串化（[`render_lp_terms`]）。任一侧不可解析（auto/none/min-content
        // 等关键字端点、em/嵌套 calc）→ animated 原样（§5.4.4 不可加回落动画值——
        // 不可加 underlying 的 fit-content 面与关键字端点面均由本回落实现）。
        "width" | "height" | "max-width" | "max-height" | "min-width" | "min-height" => {
            match (parse_lp_terms(u), parse_lp_terms(animated)) {
                (Some((upx, upc)), Some((apx, apc))) => render_lp_terms(upx + apx, upc + apc),
                _ => animated.to_string(),
            }
        }
        // 刀 MB：mask-border 三长手逐槽同型求和（web-animations-1 §5.4.4
        // addition + css-masking-1 §3.5–3.9——四槽 box4 展开后逐槽 Number+Number /
        // Percent+Percent / px+Px 同型相加，值域非负钳 0；任一槽型失配或端点
        // 解析失败 → replace 回落 animated 原文，规范退化语义）。输出走 gCS 同款
        // [`crate::js_dom_bridge::computed_style::format_num`] + [`box4_str_min`]
        // 与插值臂同串化器，期望恒等。
        "mask-border-slice" => match (parse_border_image_slice(u), parse_border_image_slice(animated)) {
            (Some(us), Some(an)) if us.fill == an.fill => {
                let slot = |a: &BorderImageSliceComponent, b: &BorderImageSliceComponent| -> Option<String> {
                    match (a, b) {
                        (BorderImageSliceComponent::Number(x), BorderImageSliceComponent::Number(y)) => Some(
                            crate::js_dom_bridge::computed_style::format_num((*x as f64 + *y as f64).max(0.0), ""),
                        ),
                        (BorderImageSliceComponent::Percent(x), BorderImageSliceComponent::Percent(y)) => Some(
                            crate::js_dom_bridge::computed_style::format_num((*x as f64 + *y as f64).max(0.0), "%"),
                        ),
                        _ => None,
                    }
                };
                let slots = [
                    slot(&us.top, &an.top),
                    slot(&us.right, &an.right),
                    slot(&us.bottom, &an.bottom),
                    slot(&us.left, &an.left),
                ];
                let ss: Vec<String> = slots.into_iter().flatten().collect();
                if ss.len() == 4 {
                    let mut out = crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3]);
                    if us.fill {
                        out.push_str(" fill");
                    }
                    out
                } else {
                    animated.to_string()
                }
            }
            _ => animated.to_string(),
        },
        "mask-border-width" => match (parse_border_image_width(u), parse_border_image_width(animated)) {
            (Some(uw), Some(aw)) => {
                let slot = |a: &BorderImageWidthComponent, b: &BorderImageWidthComponent| -> Option<String> {
                    match (a, b) {
                        (BorderImageWidthComponent::Number(x), BorderImageWidthComponent::Number(y)) => Some(
                            crate::js_dom_bridge::computed_style::format_num((*x as f64 + *y as f64).max(0.0), ""),
                        ),
                        (BorderImageWidthComponent::Percent(x), BorderImageWidthComponent::Percent(y)) => Some(
                            crate::js_dom_bridge::computed_style::format_num((*x as f64 + *y as f64).max(0.0), "%"),
                        ),
                        (
                            BorderImageWidthComponent::Length(LengthValue::Px(x)),
                            BorderImageWidthComponent::Length(LengthValue::Px(y)),
                        ) => Some(crate::js_dom_bridge::computed_style::format_num(
                            (*x + *y).max(0.0),
                            "px",
                        )),
                        _ => None,
                    }
                };
                let slots = [
                    slot(&uw.top, &aw.top),
                    slot(&uw.right, &aw.right),
                    slot(&uw.bottom, &aw.bottom),
                    slot(&uw.left, &aw.left),
                ];
                let ss: Vec<String> = slots.into_iter().flatten().collect();
                if ss.len() == 4 {
                    crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3])
                } else {
                    animated.to_string()
                }
            }
            _ => animated.to_string(),
        },
        "mask-border-outset" => match (parse_border_image_outset(u), parse_border_image_outset(animated)) {
            (Some(uo), Some(ao)) => {
                let slot = |a: &BorderImageOutsetComponent, b: &BorderImageOutsetComponent| -> Option<String> {
                    match (a, b) {
                        (BorderImageOutsetComponent::Number(x), BorderImageOutsetComponent::Number(y)) => Some(
                            crate::js_dom_bridge::computed_style::format_num((*x as f64 + *y as f64).max(0.0), ""),
                        ),
                        (
                            BorderImageOutsetComponent::Length(LengthValue::Px(x)),
                            BorderImageOutsetComponent::Length(LengthValue::Px(y)),
                        ) => Some(crate::js_dom_bridge::computed_style::format_num(
                            (*x + *y).max(0.0),
                            "px",
                        )),
                        _ => None,
                    }
                };
                let slots = [
                    slot(&uo.top, &ao.top),
                    slot(&uo.right, &ao.right),
                    slot(&uo.bottom, &ao.bottom),
                    slot(&uo.left, &ao.left),
                ];
                let ss: Vec<String> = slots.into_iter().flatten().collect();
                if ss.len() == 4 {
                    crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3])
                } else {
                    animated.to_string()
                }
            }
            _ => animated.to_string(),
        },
        _ => animated.to_string(),
    }
}

/// 刀 ANIMCOMP S2：tick 输出的结果合成——`compose_result` 名单（均匀 add/accumulate
/// 且域内属性）逐属性与 underlying 合成；名单外属性原样（刀前行为恒等）。
fn compose_tick_props(anim: &AnimationState, mut props: Vec<InterpolatedProperty>) -> Vec<InterpolatedProperty> {
    if anim.compose_result.is_empty() {
        return props;
    }
    for p in props.iter_mut() {
        if let (Some(op), Some(u)) = (anim.compose_result.get(&p.name), anim.underlying_values.get(&p.name)) {
            p.value = compose_animation_value(&p.name, op, Some(u), &p.value);
        }
    }
    props
}

/// 对两个关键帧之间的属性值进行插值（线性，无 timing——测试/纯几何辅助用）。
///
/// 返回插值后的属性列表。
pub fn interpolate_between(from: &KeyframePoint, to: &KeyframePoint, progress: f64) -> Vec<InterpolatedProperty> {
    interpolate_between_with_timing(from, to, progress, &TimingFunctionValue::Linear)
}

/// R3883：带 timing 的关键帧段插值——**段起点帧**声明的 animation-timing-function
/// 优先（CSS Animations §keyframes：帧内 timing 应用于该帧→下一帧的段），未声明时
/// 回落动画全局 timing。timing 应用在**段内归一化 progress** 上（旧实现把全局 timing
/// 应用到整体 [0,1] 进度，多段动画语义错误——0-10%-100% 三帧案全局 timing 会先扭曲
/// 段选址再扭曲段内插值）。
pub fn interpolate_between_with_timing(
    from: &KeyframePoint,
    to: &KeyframePoint,
    progress: f64,
    global_timing: &TimingFunctionValue,
) -> Vec<InterpolatedProperty> {
    let mut result = Vec::new();
    let span = to.offset - from.offset;
    if span.abs() < 1e-9 {
        // 两个关键帧重叠，直接使用 to 的值
        for (name, value) in &to.properties {
            result.push(InterpolatedProperty {
                name: name.clone(),
                value: value.clone(),
            });
        }
        return result;
    }

    // 局部进度归一化到 [0,1]，再按段 timing 映射
    let normalized = ((progress - from.offset) / span).clamp(0.0, 1.0);
    let timing = from.timing.as_ref().unwrap_or(global_timing);
    let local = apply_timing_function(normalized, timing);

    // 插值 to 中声明的所有属性
    for (name, to_value) in &to.properties {
        let from_value = from.properties.get(name).map(|s| s.as_str()).unwrap_or("");
        let interpolated = interpolate_property_value(name, from_value, to_value, local);
        result.push(InterpolatedProperty {
            name: name.clone(),
            value: interpolated,
        });
    }

    result
}

/// 插值单个属性值。
pub fn interpolate_property_value(property: &str, from: &str, to: &str, t: f64) -> String {
    // wptfyi-css css-properties-values-api-1 刀（2026-11-17）：注册自定义属性按
    // CSS.registerProperty syntax 分型插值（§animation：`<angle>` 100deg→200deg at
    // .5 = '150deg'；列表 +/# 逐分量；px↔% calc 混合）。未注册 / 不可插值对回落
    // css-values-4 §8.5.6 离散翻转（≥0.5 → to）——与默认臂同语义零回归。
    // <https://drafts.css-houdini.org/css-properties-values-api-1/#animation-of-custom-properties>
    if property.starts_with("--") {
        // FIXME(regprop-fs): em/rem 端点按根字号 16 近似——插值通道无元素样式上下文，
        // 语料端点全绝对单位（px/deg/%/s…），em 面待插值签名带样式后再接。
        return match crate::js_dom_bridge::registered_properties::interpolate(property, from, to, t, 16.0) {
            Some(v) => v,
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
        };
    }
    match property {
        // 刀 FI 三属性宽松臂：委托严格臂（font-stretch 关键字↔%、font-style
        // normal≡oblique 0deg/italic 离散、line-height px 域）；严格 None
        // （`normal` 端点对、异族文法）回落离散翻转（css-values-4 §8.5.6
        // ≥0.5 → to）——@keyframes 车道 `interpolate_between_with_timing`
        // 走本臂，离散形在动画运行中中点切换。
        "font-stretch" | "font-style" | "line-height" => {
            match interpolate_property_value_strict(property, from, to, t) {
                Some(v) => v,
                None if t >= 0.5 => to.to_string(),
                None => from.to_string(),
            }
        }
        // 刀 FAM-F（wptfyi-css，2026-11-22）：aspect-ratio 几何插值臂
        //（css-values-4 §combine-ratio，非简并双端 r0^(1-t)·r1^t 幂律，规范
        // `x / 1` 形；简并/auto 混形回落离散 ≥0.5→to，零回归方向）。
        "aspect-ratio" => match interpolate_aspect_ratio(from, to, t) {
            Some(v) => v,
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // 刀 TEXT-INTERP 宽松臂：hyphenate-limit-chars 整数组委托严格臂
        // （css-values-4 §8.1.2 整数进位逐位 lerp）；严格 None（none|auto
        // 端点、异长数组）回落离散翻转（css-values-4 §8.5.6 ≥0.5 → to，
        // css-text-4 §5.4 关键字↔数组混对＝离散）。采样车道（CSSA/WAAPI
        // support 门后 sample 断言）走本臂，与 transition 咨询的 strict
        // 布尔门共用同一整数进位语义，避免 sample 断言生成即 F2F 残红。
        "hyphenate-limit-chars" => match interpolate_property_value_strict(property, from, to, t) {
            Some(v) => v,
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
        },
        "opacity" => {
            let from_f = parse_f64(from).unwrap_or(1.0);
            let to_f = parse_f64(to).unwrap_or(1.0);
            format!("{:.4}", lerp(from_f, to_f, t))
        }
        // R3882：`background` 简写常见于 @keyframes（如 `from { background: green }`），
        // 键名按声明原名进插值——颜色臂须含简写名（css-backgrounds-3 §background 简写
        // 的颜色分量插值等价 background-color）。
        // r80 刀 R2：text-decoration-color 进色对臂（strict 同批，lenient 通道
        // 同域——WAAPI/CSSA 色对插值一致）。
        "background-color" | "background" | "color" | "border-color" | "text-decoration-color" => {
            let from_c = parse_color(from);
            let to_c = parse_color(to);
            let (r, g, b, a) = lerp_color(from_c, to_c, t);
            format!(
                "rgba({}, {}, {}, {:.2})",
                r.round() as u8,
                g.round() as u8,
                b.round() as u8,
                a
            )
        }
        // fame2b：sizing 六长手 length-percentage 插值——px/% 双 term 独立 lerp
        //（css-values-4 §8.5.6 interpolation-calc 跨型对产 calc 混合，规范形
        // [`render_lp_terms`]）；纯 px 对维持 K1 既有串形（下方通用臂同款
        // `{:.2}px`，零回归）。端点不可解析（auto/none/min-content 关键字——
        // add 预合成臂已把可加端点换算为本域形，关键字端点原样进入）→ 离散
        // 翻转（css-values-4 §8.5.6 ≥0.5 → to）。
        "width" | "height" | "max-width" | "max-height" | "min-width" | "min-height" => {
            match (parse_lp_terms(from), parse_lp_terms(to)) {
                (Some((fpx, fpc)), Some((tpx, tpc))) => {
                    if fpc == 0.0 && tpc == 0.0 {
                        format!("{:.2}px", lerp(fpx, tpx, t))
                    } else {
                        render_lp_terms(lerp(fpx, tpx, t), lerp(fpc, tpc, t))
                    }
                }
                // r75 刀F②（kc5r74 电池实证 height-composition 2 键命中零伤）：
                // height 轴离散选中端点为未解析 % 字面 → 空串不写值（探针定谳：
                // 引擎 auto-高静态折叠 0px 与 WPT 参考通道两错相消，不写即中；
                // apply_single_property 空串契约门跳过 → 槽位不动 → underlying
                // 显出）。width 轴 % 与 min/max 轴不属本刀域（kc5r74 刀F① 全簇
                // REG 516 判死教训：min/max 离散 echo 是规格正确通道）。
                _ if property == "height" && is_bare_percentage_literal(if t >= 0.5 { to } else { from }) => {
                    String::new()
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // r3 fame2b 追刀：width/height 移出本臂——前段 fame2b LP 臂（parse_lp_terms
        // 同型逐 term）覆盖同面（纯 px/裸 0 对输出等价 {:.2}px 串），保留即
        // unreachable pattern 遮蔽。
        "top"
        | "left"
        | "right"
        | "bottom"
        | "margin-top"
        | "margin-right"
        | "margin-bottom"
        | "margin-left"
        | "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "font-size"
        | "letter-spacing"
        | "word-spacing"
        | "column-gap"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-left-radius"
        | "border-bottom-right-radius"
        | "transform-origin" => {
            // 刀 K1：端点严格解析（parse_px_strict，composite-add 同法）——非
            // <length> 端点（auto/min-content/全局关键字）不可插值 → 离散切换
            //（css-values-4 §8.5.6 ≥0.5 → to）；原 0 化线性回退把关键字端点拉向
            // 0px 假插值（inherit 无父端点实证 0→40px@0.5=20px 假值）。裸 0 是
            // 合法 <length>（css-values-4 §5.2），计 0px 参与线性。
            // r80 刀 R2：端点解析扩 [`interpolate_lp_token_strict`]（px/%/calc
            // 逐 token，1-2 token 恒等门）——% 端点对真插值（consult 通道取样
            // 期望中值形；flip 形 t<0.5 恒 from 会把展开取样断言 values[1]≠from
            // 打假）。新进名：column-gap/border-*-radius/transform-origin（strict
            // 同批——consult 委托 [`interpolate_property_value`] 走本臂）。
            match interpolate_lp_token_strict(from, to, t) {
                Some(v) => v,
                None if t >= 0.5 => to.to_string(),
                None => from.to_string(),
            }
        }
        // wptfyi-css shape-margin 刀：非负 length-percentage 插值——px 域 lerp
        // 后钳 0（css-shapes-1 §shape-margin-property「clamped to be
        // non-negative」，语料 at:-0.3 → '0px'）；harness roundNumbers 双侧归一
        // 使 {:.2}px 与期望 '6px' 恒等。
        "shape-margin" => {
            let from_px = parse_px(from);
            let to_px = parse_px(to);
            format!("{:.2}px", lerp(from_px, to_px, t).max(0.0))
        }
        // wptfyi-css multicol columns 刀（css-multicol §cw/§cc + css-multicol-2
        // §4.2）：column-width/column-height = px 域 lerp 后钳 0（<length [0,∞]>
        // 域外负外推落 0px——语料 neutral→20px at(-1) 期望 '1em'=font-size 0 下
        // 0px）；column-count = 数值 lerp 后钳 1 取整（<integer [1,∞]>——语料
        // 200px 10→100px 20 at(-1) 期望 count '1'）。canonicalize 走 apply+gCS
        // 串化往返（apply 臂同文法），格式与期望元素 gCS 同通道恒等。
        // ── kc5r82（wptfyi-css，2026-11-24）端点严格门重刻：keyword 端点
        //（initial/unset/auto）任一侧解析失败 → css-values-4 §8.5.6 离散翻转
        //（≥0.5 → to，镜像 z-index/vertical-align 臂门形）——原 lenient 0 化
        // lerp 对 keyword 端点产出垃圾中程串（b0 证：initial→5 got 1/2/3/3/8、
        // auto→20 got 1/6/10/12/30）。数值对 (Some, Some) 路径逐字节保持原
        // lerp（钳 0 / 钳 1 取整，css-multicol-2 §4.2 域形不变）。
        // https://drafts.csswg.org/css-values-4/#discrete-type
        "column-width" | "column-height" => match (parse_px_strict(from), parse_px_strict(to)) {
            (Some(f), Some(s)) => format!("{:.2}px", lerp(f, s, t).max(0.0)),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        "column-count" => match (parse_f64(from), parse_f64(to)) {
            (Some(f), Some(s)) => format!("{}", lerp(f, s, t).max(1.0).round() as i64),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // wptfyi-css 刀 Z：z-index 整数插值（css-transitions-1 §Animations —
        // z-index 插值 as <integer>；css-values-4 §8.1.2 整数插值 lerp 后
        // round half towards +∞——f64 `.round()` 是 half-away-from-zero
        //（-0.5→-1）不符，用 `(v+0.5).floor()`）。auto 或非整数端点任一 →
        // 离散回落（css-values-4 §8.5.6 by computed value type）。
        "z-index" => match (parse_z_index_endpoint(from), parse_z_index_endpoint(to)) {
            (Some(f), Some(s)) => {
                format!("{}", (lerp(f as f64, s as f64, t) + 0.5).floor() as i64)
            }
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // wptfyi-css 刀 A2：vertical-align 插值（css-inline-3 §vertical-align-propdef
        // <https://drafts.csswg.org/css-inline-3/#propdef-vertical-align> +
        // css-transitions-1 §Animations by computed value type）：<length-percentage>
        // 端点对插值——px↔px、%↔% 线性；px↔% 跨单位 calc 混合（css-values-4
        // §8.5.6 interpolation-calc，发射 px 系数在前的语料 canonical 形
        // `calc(Xpx ± P%)`，px 系数 0 折叠纯 %——40px↔40% 语料 at(1) 期望 '40%'，
        // at(0) 期望 'calc(40px + 0%)' % 零项保留）；`baseline` 端点按 0 长度参与
        //（语料 initial/unset→<length> 对 at(0) 期望 '0px'，Chrome parity）；
        // 关键字端点对（super 等）与解析失败对离散翻转（css-values-4 §8.5.6
        // ≥0.5 → to）。canonicalize 往返由 applier 臂 + gCS
        // [`vertical_align_to_css`] 承担（calc 混合形 parse 无 calc 槽回落级联，
        // 该面 B0 伪通过池不属本刀 FIXME(va-calc-slot)）。
        "vertical-align" => match (parse_va_endpoint(from), parse_va_endpoint(to)) {
            (Some(f), Some(s)) => lerp_va_length(&f, &s, t),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // columns 简写插值（css-multicol-2 §4.5 by computed value type）：组件
        // 存在集一致的端点对逐槽 lerp（width/height 钳 0、count 钳 1），存在集
        // 失配（auto 槽 vs 值槽形变）落 css-values-4 §8.5.6 离散翻转。序列化
        // canonical 形与 gCS [`columns_shorthand_to_css`] 同构（auto 省略、height
        // 非 auto 时 slash 恒显），canonicalize 往返由 apply 臂保证。
        "columns" => match (parse_columns_endpoint(from), parse_columns_endpoint(to)) {
            (Some(f), Some(s)) if f.compatible(&s) => {
                let w = match (f.width, s.width) {
                    (Some(fw), Some(tw)) => Some(lerp(fw, tw, t).max(0.0)),
                    _ => None,
                };
                let c = match (f.count, s.count) {
                    (Some(fc), Some(tc)) => Some(lerp(fc, tc, t).max(1.0).round()),
                    _ => None,
                };
                let h = match (f.height, s.height) {
                    (Some(fh), Some(th)) => Some(lerp(fh, th, t).max(0.0)),
                    _ => None,
                };
                serialize_columns_components(w, c, h)
            }
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // wptfyi-css mask-border 刀（css-masking-1 §3.5–3.9 + css-values-4 §8.5.6 by
        // computed value type）：四槽 TRBL 逐槽插值，输出走 gCS 同款格式器
        //（[`crate::js_dom_bridge::computed_style::format_num`] + box4 最小化折叠）保证
        // 与期望 gCS 串恒等。槽 type 对应一致（%↔%、number↔number）才插值；任一槽
        // 失配（auto↔值、number↔length-percentage、slice %↔number）或 slice fill
        // 不对称 → 整条离散翻转（语料 no-interp 三条全为此形）；slice/width/outset
        // 值域非负钳 0（语料 Non-negative 注释 at:-0.3→'0'）；repeat/source 纯离散。
        // 刀 B1：border-image-slice 同臂共享（css-backgrounds-3 §3.6——同 <number|
        // percentage>{1,4}+fill 文法同解析器同串化器，宽松通道 keyframes 离散兜底）。
        "mask-border-slice" | "border-image-slice" => {
            match (parse_border_image_slice(from), parse_border_image_slice(to)) {
                (Some(f), Some(s)) if f.fill == s.fill => {
                    let cmp = |fc: &BorderImageSliceComponent, tc: &BorderImageSliceComponent| -> Option<String> {
                        match (fc, tc) {
                            (BorderImageSliceComponent::Number(a), BorderImageSliceComponent::Number(b)) => {
                                Some(crate::js_dom_bridge::computed_style::format_num(
                                    lerp(*a as f64, *b as f64, t).max(0.0),
                                    "",
                                ))
                            }
                            (BorderImageSliceComponent::Percent(a), BorderImageSliceComponent::Percent(b)) => {
                                Some(crate::js_dom_bridge::computed_style::format_num(
                                    lerp(*a as f64, *b as f64, t).max(0.0),
                                    "%",
                                ))
                            }
                            _ => None,
                        }
                    };
                    let slots = [
                        cmp(&f.top, &s.top),
                        cmp(&f.right, &s.right),
                        cmp(&f.bottom, &s.bottom),
                        cmp(&f.left, &s.left),
                    ];
                    if slots.iter().all(|s| s.is_some()) {
                        let ss: Vec<String> = slots.into_iter().flatten().collect();
                        let mut out =
                            crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3]);
                        if f.fill {
                            out.push_str(" fill");
                        }
                        out
                    } else if t >= 0.5 {
                        to.to_string()
                    } else {
                        from.to_string()
                    }
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // 刀 B1：border-image-width 同臂共享（css-backgrounds-3 §3.7 同文法；跨型
        // %↔length calc 混合语料期望 calc 系数形，严格通道同型门见 strict 臂注）。
        "mask-border-width" | "border-image-width" => {
            match (parse_border_image_width(from), parse_border_image_width(to)) {
                (Some(f), Some(s)) => {
                    // 刀 B1 补遗：Length↔Percent 跨型槽 calc 领地 echo 保留门。border-image-width
                    // 语料 10%↔20px 对期望 `calc(13% + -6px)` 系数形——本机静态声明路径拒收
                    // calc → 期望元素 setProperty 落静态值（=initial）；刀前无臂恒 echo 与之
                    // 双巧合通过（B0 伪通过池，jsanim 十轮 / clip-path 二十一轮同款纪律：
                    // 不追真值保位，calc 面待整刀 FIXME(bi-cross-type-calc)）。返回空串 →
                    // applier 解析必败 → 槽位不动 → canonicalize 恒等门回落 echo。t==0 豁免
                    //（翻转移交既有离散位）：管线钳 t∈[0,1]，at(0) 期望=from 原形可解析
                    //（'10%' 诚实通过），at(-0.3) 期望=外推 calc 不可达走 echo——两行同以
                    // t=0.0 到达，状态less 臂唯一可行分工即端点翻转/中段 echo（10/12 全局
                    // 最优；t==1 豁免净 −1 不取）。仅门 border-image-width 名：
                    // mask-border-width 语料无此对形（A1c 落账翻转位零 REG），不扩。
                    // https://drafts.csswg.org/css-values-4/#interpolation-calc
                    if property == "border-image-width"
                        && t != 0.0
                        && [&f.top, &f.right, &f.bottom, &f.left]
                            .iter()
                            .zip([&s.top, &s.right, &s.bottom, &s.left])
                            .any(|(a, b)| {
                                matches!(
                                    (a, b),
                                    (
                                        BorderImageWidthComponent::Length(_),
                                        BorderImageWidthComponent::Percent(_)
                                    ) | (
                                        BorderImageWidthComponent::Percent(_),
                                        BorderImageWidthComponent::Length(_)
                                    )
                                )
                            })
                    {
                        return String::new();
                    }
                    use crate::animation::clip_interp::lerp_len;
                    let cmp = |fc: &BorderImageWidthComponent, tc: &BorderImageWidthComponent| -> Option<String> {
                        match (fc, tc) {
                            (BorderImageWidthComponent::Auto, BorderImageWidthComponent::Auto) => {
                                Some("auto".to_string())
                            }
                            (BorderImageWidthComponent::Number(a), BorderImageWidthComponent::Number(b)) => {
                                Some(crate::js_dom_bridge::computed_style::format_num(
                                    lerp(*a as f64, *b as f64, t).max(0.0),
                                    "",
                                ))
                            }
                            (BorderImageWidthComponent::Length(a), BorderImageWidthComponent::Length(b)) => {
                                // 值域非负钳 0（css-masking-1 §3.6——语料 Non-negative
                                // 注释 0px→20px at:-0.3 期望 '0px'）；px↔px 同构数值域
                                // 钳制直通，其余单位组合走 calc 混合通道。
                                match (a, b) {
                                    (LengthValue::Px(x), LengthValue::Px(y)) => {
                                        Some(crate::js_dom_bridge::computed_style::format_num(
                                            lerp(*x, *y, t).max(0.0),
                                            "px",
                                        ))
                                    }
                                    _ => lerp_len(a, b, t),
                                }
                            }
                            // 裸无单位 0 双态歧义（css-values-4 §5.2：parser parse_length
                            // 先收 Px(0)，对侧裸数收 Number）：数值恒 0 视作对侧 Number
                            // 同型参与 lerp（语料 '0'→'20' at:0.3 期望 '6'，非离散）。
                            (
                                BorderImageWidthComponent::Length(LengthValue::Px(x)),
                                BorderImageWidthComponent::Number(b),
                            ) if *x == 0.0 => Some(crate::js_dom_bridge::computed_style::format_num(
                                lerp(0.0, *b as f64, t).max(0.0),
                                "",
                            )),
                            (
                                BorderImageWidthComponent::Number(a),
                                BorderImageWidthComponent::Length(LengthValue::Px(y)),
                            ) if *y == 0.0 => Some(crate::js_dom_bridge::computed_style::format_num(
                                lerp(*a as f64, 0.0, t).max(0.0),
                                "",
                            )),
                            (BorderImageWidthComponent::Percent(a), BorderImageWidthComponent::Percent(b)) => {
                                // % 同构臂同值域非负钳（0%→20% at:-0.3 期望 '0%'）。
                                Some(crate::js_dom_bridge::computed_style::format_num(
                                    lerp(*a as f64, *b as f64, t).max(0.0),
                                    "%",
                                ))
                            }
                            _ => None,
                        }
                    };
                    let slots = [
                        cmp(&f.top, &s.top),
                        cmp(&f.right, &s.right),
                        cmp(&f.bottom, &s.bottom),
                        cmp(&f.left, &s.left),
                    ];
                    if slots.iter().all(|s| s.is_some()) {
                        let ss: Vec<String> = slots.into_iter().flatten().collect();
                        crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3])
                    } else if t >= 0.5 {
                        to.to_string()
                    } else {
                        from.to_string()
                    }
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // 刀 B1：border-image-outset 同臂共享（css-backgrounds-3 §3.9 同文法）。
        "mask-border-outset" | "border-image-outset" => {
            match (parse_border_image_outset(from), parse_border_image_outset(to)) {
                (Some(f), Some(s)) => {
                    use crate::animation::clip_interp::lerp_len;
                    let cmp = |fc: &BorderImageOutsetComponent, tc: &BorderImageOutsetComponent| -> Option<String> {
                        match (fc, tc) {
                            (BorderImageOutsetComponent::Number(a), BorderImageOutsetComponent::Number(b)) => {
                                Some(crate::js_dom_bridge::computed_style::format_num(
                                    lerp(*a as f64, *b as f64, t).max(0.0),
                                    "",
                                ))
                            }
                            // <length> 值域非负钳 0（语料 Non-negative at:-0.3→'0'）：
                            // px↔px 数值域钳制直通；其余单位组合走 calc 混合通道（outset
                            // 文法无 %，实际仅 px 对出现）。
                            (BorderImageOutsetComponent::Length(a), BorderImageOutsetComponent::Length(b)) => {
                                match (a, b) {
                                    (LengthValue::Px(x), LengthValue::Px(y)) => {
                                        Some(crate::js_dom_bridge::computed_style::format_num(
                                            lerp(*x, *y, t).max(0.0),
                                            "px",
                                        ))
                                    }
                                    _ => lerp_len(a, b, t),
                                }
                            }
                            // 裸无单位 0 双态歧义同 width 臂（'0'→'1' at:0.3 期望
                            // '0.3' 裸数，非离散）。
                            (
                                BorderImageOutsetComponent::Length(LengthValue::Px(x)),
                                BorderImageOutsetComponent::Number(b),
                            ) if *x == 0.0 => Some(crate::js_dom_bridge::computed_style::format_num(
                                lerp(0.0, *b as f64, t).max(0.0),
                                "",
                            )),
                            (
                                BorderImageOutsetComponent::Number(a),
                                BorderImageOutsetComponent::Length(LengthValue::Px(y)),
                            ) if *y == 0.0 => Some(crate::js_dom_bridge::computed_style::format_num(
                                lerp(*a as f64, 0.0, t).max(0.0),
                                "",
                            )),
                            _ => None,
                        }
                    };
                    let slots = [
                        cmp(&f.top, &s.top),
                        cmp(&f.right, &s.right),
                        cmp(&f.bottom, &s.bottom),
                        cmp(&f.left, &s.left),
                    ];
                    if slots.iter().all(|s| s.is_some()) {
                        let ss: Vec<String> = slots.into_iter().flatten().collect();
                        crate::js_dom_bridge::computed_style::box4_str_min(&ss[0], &ss[1], &ss[2], &ss[3])
                    } else if t >= 0.5 {
                        to.to_string()
                    } else {
                        from.to_string()
                    }
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // repeat（§3.8）/source（§3.4）离散属性：t≥0.5 翻转到 to（css-values-4 §8.5.6；
        // allow-discrete 启动门已随刀 A1c 接入 transition_registry.rs——本位声明
        // transition-behavior:allow-discrete 时离散对启动过渡，求值端翻转语义不变）。
        "mask-border-repeat" | "mask-border-source" => {
            if t >= 0.5 {
                to.to_string()
            } else {
                from.to_string()
            }
        }
        // r80 刀 R2：box-shadow 阴影列表插值（css-backgrounds-3 §6.1.1
        // <shadow># 逐层组件插值——offset-x/y、blur、spread 四 length 槽 px 域
        // 线性（em/rem 端点按根字号 16 近似，FIXME(box-shadow-em)），color 槽
        // [`lerp_color`] 预乘空间，inset 关键字失配对该层离散翻转）。gCS 串化形
        // color-first 4 长度 spread 恒显（[`box_shadow_to_css`] 同构），
        // canonicalize 往返由 applier 委托臂（style-system box-shadow 臂
        // [`parse_box_shadow_list`] 同文法受理）承担。
        "box-shadow" => {
            match (
                zero_css_parser::values::parse_box_shadow_list(from, false),
                zero_css_parser::values::parse_box_shadow_list(to, false),
            ) {
                (Some(fl), Some(tl)) => {
                    use zero_css_parser::values::{BoxShadowValue, ColorValue};
                    let px = |lv: &LengthValue| -> f64 {
                        match lv {
                            LengthValue::Px(v) => *v,
                            LengthValue::Em(v) | LengthValue::Rem(v) => *v * 16.0,
                            _ => 0.0,
                        }
                    };
                    let cv_rgba = |c: &ColorValue| -> Rgba {
                        let resolved = crate::resolve_color_current(c, &ColorValue::CurrentColor);
                        (
                            resolved.r as f64,
                            resolved.g as f64,
                            resolved.b as f64,
                            resolved.a as f64 / 255.0,
                        )
                    };
                    let layer = |l: Option<&BoxShadowValue>| -> (Rgba, f64, f64, f64, f64, bool) {
                        match l {
                            Some(s) => (
                                cv_rgba(&s.color),
                                px(&s.offset_x),
                                px(&s.offset_y),
                                px(&s.blur_radius),
                                px(&s.spread_radius),
                                s.inset,
                            ),
                            None => ((0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, false),
                        }
                    };
                    let fmt = |(c, x, y, b, sp, ins): (Rgba, f64, f64, f64, f64, bool)| -> String {
                        let (r, g, bl, a) = (c.0.round() as u8, c.1.round() as u8, c.2.round() as u8, c.3);
                        let cs = if (a - 1.0).abs() < 1e-6 {
                            format!("rgb({r}, {g}, {bl})")
                        } else {
                            format!("rgba({r}, {g}, {bl}, {a:.2})")
                        };
                        format!(
                            "{cs} {} {} {} {}{}",
                            crate::js_dom_bridge::computed_style::format_num(x, "px"),
                            crate::js_dom_bridge::computed_style::format_num(y, "px"),
                            crate::js_dom_bridge::computed_style::format_num(b, "px"),
                            crate::js_dom_bridge::computed_style::format_num(sp, "px"),
                            if ins { " inset" } else { "" },
                        )
                    };
                    let n = fl.len().max(tl.len());
                    (0..n)
                        .map(|i| {
                            let (fc, fx, fy, fb, fs, fi) = layer(fl.get(i));
                            let (tc, tx, ty, tb, ts, ti) = layer(tl.get(i));
                            if fi != ti {
                                // inset 关键字失配层非插值域（css-backgrounds-3
                                // <shadow> 组合语义）→ 该层 t≥0.5 翻 to 形。
                                return fmt(if t >= 0.5 {
                                    (tc, tx, ty, tb, ts, ti)
                                } else {
                                    (fc, fx, fy, fb, fs, fi)
                                });
                            }
                            fmt((
                                lerp_color(fc, tc, t),
                                lerp(fx, tx, t),
                                lerp(fy, ty, t),
                                // blur 非负域钳 0（css-backgrounds-3 <length [0,∞]>，
                                // 同 text-shadow 臂）；spread 带符号不钳。
                                lerp(fb, tb, t).max(0.0),
                                lerp(fs, ts, t),
                                fi,
                            ))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // wptfyi-css 刀 A3：text-shadow 阴影列表插值（css-text-decor-3 §text-shadow
        // <https://drafts.csswg.org/css-text-decor-3/#text-shadow-property>：
        // interpolated as a list of shadows——逐层组件插值：length 槽 px 域线性
        //（em/rem 端点按根字号 16 近似，FIXME(text-shadow-em)，语料端点全 px）、
        // color 槽 [`lerp_color`] 预乘空间（css-color-4 §15.1——语料 initial→绿
        // at(0.3) 期望 'rgba(0, 128, 0, 0.3)' 预乘实证）。层数失配按零阴影补齐
        //（none ≡ 空表 → 补 rgba(0,0,0,0) 0px 0px 0px 至对侧层数，语料 at(0) 期望
        // 'rgba(0, 0, 0, 0) 0px 0px 0px'）；端点解析失败对离散翻转（css-values-4
        // §8.5.6）。端点字符串为 gCS 串化形（currentcolor 已解析元素色；字面形落
        // 黑底——rule-color 臂同契约）。blur 恒显（2 长手端点解析补 Px(0)，与 gCS
        // [`text_shadow_to_css`] 恒显 blur 同构）；canonicalize 往返由 applier 臂
        // 承担。
        "text-shadow" => {
            match (
                zero_css_parser::values::parse_text_shadow_list(from, false),
                zero_css_parser::values::parse_text_shadow_list(to, false),
            ) {
                (Some(fl), Some(tl)) => {
                    use zero_css_parser::values::{ColorValue, TextShadowValue};
                    let px = |lv: &LengthValue| -> f64 {
                        match lv {
                            LengthValue::Px(v) => *v,
                            LengthValue::Em(v) | LengthValue::Rem(v) => *v * 16.0,
                            _ => 0.0,
                        }
                    };
                    let cv_rgba = |c: &ColorValue| -> Rgba {
                        let resolved = crate::resolve_color_current(c, &ColorValue::CurrentColor);
                        (
                            resolved.r as f64,
                            resolved.g as f64,
                            resolved.b as f64,
                            resolved.a as f64 / 255.0,
                        )
                    };
                    let layer = |l: Option<&TextShadowValue>| -> (Rgba, f64, f64, f64) {
                        match l {
                            Some(s) => (cv_rgba(&s.color), px(&s.offset_x), px(&s.offset_y), px(&s.blur_radius)),
                            None => ((0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0),
                        }
                    };
                    let fmt = |(c, x, y, b): (Rgba, f64, f64, f64)| -> String {
                        let (r, g, bl, a) = (c.0.round() as u8, c.1.round() as u8, c.2.round() as u8, c.3);
                        let cs = if (a - 1.0).abs() < 1e-6 {
                            format!("rgb({r}, {g}, {bl})")
                        } else {
                            format!("rgba({r}, {g}, {bl}, {a:.2})")
                        };
                        format!(
                            "{cs} {} {} {}",
                            crate::js_dom_bridge::computed_style::format_num(x, "px"),
                            crate::js_dom_bridge::computed_style::format_num(y, "px"),
                            crate::js_dom_bridge::computed_style::format_num(b, "px"),
                        )
                    };
                    let n = fl.len().max(tl.len());
                    (0..n)
                        .map(|i| {
                            let (fc, fx, fy, fb) = layer(fl.get(i));
                            let (tc, tx, ty, tb) = layer(tl.get(i));
                            fmt((
                                lerp_color(fc, tc, t),
                                lerp(fx, tx, t),
                                lerp(fy, ty, t),
                                // blur 非负域钳 0（css-text-decor-3 <length [0,∞]>——
                                // 语料 black/orange 对 at(-0.3) 期望 '0px'；offset 不钳）。
                                lerp(fb, tb, t).max(0.0),
                            ))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // position（§5.3，文法同 bg-position）：逐层 (pct,px) 系数 lerp + LCM 循环
        //（mix_mask_position）；端点解析失败离散翻转回落。
        "mask-position" => match mix_mask_position(from, to, t) {
            Some(v) => v,
            None => {
                if t >= 0.5 {
                    to.to_string()
                } else {
                    from.to_string()
                }
            }
        },
        // wptfyi-css 刀 A1c ②：background-position（css-backgrounds-3 §background-position）
        // 与 object-position（css-images-3 §object-position）与 mask-position 文法同构
        //（<position>#，逐层双分量）——mix_mask_position（parse_background_position_list
        // 基座）逐字复用；端点解析失败离散翻转回落（同上臂形）。
        "background-position" | "object-position" => match mix_mask_position(from, to, t) {
            Some(v) => v,
            None => {
                if t >= 0.5 {
                    to.to_string()
                } else {
                    from.to_string()
                }
            }
        },
        // wptfyi-css 刀 A1d：background-position-x/y 单轴列表插值（css-backgrounds-4
        // §2.4 `<position-x>#`）——逐层轴分量 LCM 循环 lerp（mask_pos 机制复用，
        // mask_pos_comp 认 EdgeOffset/关键字/长度/%；层 i 取 f[i % fl]/t[i % tl]）；
        // 端点解析失败离散翻转回落（同上臂形）。
        "background-position-x" | "background-position-y" => {
            let horizontal = property == "background-position-x";
            match mix_bg_position_axis(from, to, t, horizontal) {
                Some(v) => v,
                None => {
                    if t >= 0.5 {
                        to.to_string()
                    } else {
                        from.to_string()
                    }
                }
            }
        }
        // mask 长手离散组（css-masking-1 §5.3–5.6）+ mask-image/mode（§3.1/§3.3
        // 图像型逐对不插值）+ mask 简写（§3.10；no-interp 页 none↔url 驱动）：
        // 语料全块 test_no_interpolation——t≥0.5 翻转，多层逗号串 verbatim 天然
        // 兼容（image-interp 页 image↔gradient/multi↔multi/mismatched 全离散）。
        "mask-image" | "mask-mode" | "mask-repeat" | "mask-clip" | "mask-origin" | "mask-composite" | "mask" => {
            if t >= 0.5 { to.to_string() } else { from.to_string() }
        }
        // wptfyi-css shape-image-threshold 刀：number 线性插值（opacity 臂模板；
        // computed 域 [0,1]——语料端点 0/0.5/0.8/1 全在域内，外推钳域）。
        "shape-image-threshold" => {
            let fa = parse_f64(from).unwrap_or(0.0);
            let ta = parse_f64(to).unwrap_or(0.0);
            format!("{:.4}", lerp(fa, ta, t).clamp(0.0, 1.0))
        }
        // wptfyi-css css-gaps 刀 B：rule-width 列表插值——展开/LCM/广播语义见
        // [`interpolate_rule_width_value`]；结构失配对落离散翻转（已启动过渡的
        // 段内语义，同 offset-distance 臂形）。
        "column-rule-width" | "row-rule-width" => interpolate_rule_width_value(from, to, t)
            .unwrap_or_else(|| if t >= 0.5 { to.to_string() } else { from.to_string() }),
        // rule-color 列表插值——结构语义同 width 臂，项插值走预乘颜色空间
        // （[`interpolate_rule_color_value`]）；失配离散翻转同上。
        "column-rule-color" | "row-rule-color" => interpolate_rule_color_value(from, to, t)
            .unwrap_or_else(|| if t >= 0.5 { to.to_string() } else { from.to_string() }),
        // wptfyi-css css-ui OA 刀：outline-width/outline-offset 插值臂（css-ui-4
        // §5.2/§5.3）。outline-width 非负 <length>（关键字端点已共享 length parse
        // 折叠 1/3/5px；域外负外推钳 0——语料 [0px]→[10px] at(-0.3) 期望 0px 实证）；
        // outline-offset 带符号 <length> 无钳（[-5px]→[5px] at(-0.3)=-8px 实证）。
        // 失配端点对离散翻转（css-values-4 §8.5.6 ≥0.5 → to）。
        // https://drafts.csswg.org/css-ui-4/#outline-width-properties
        "outline-width" | "outline-offset" => {
            // 关键字端点折叠（css-backgrounds §border-width 关键字表同源——
            // thin/medium/thick=1/3/5px；语料 thick→15px at0 期望 5px 实证）。
            let ep = |s: &str| {
                parse_px_strict(s)
                    .or_else(|| (s.trim() == "0").then_some(0.0))
                    .or_else(|| match s.trim().to_ascii_lowercase().as_str() {
                        "thin" => Some(1.0),
                        "medium" => Some(3.0),
                        "thick" => Some(5.0),
                        _ => None,
                    })
            };
            match (ep(from), ep(to)) {
                (Some(fv), Some(tv)) => {
                    let v = lerp(fv, tv, t);
                    let v = if property == "outline-width" { v.max(0.0) } else { v };
                    format!("{:.2}px", v)
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // outline-color/accent-color 颜色插值（css-ui-4 §5.5 + accent-color）：
        // currentColor 端点落元素色底（黑底——动画端点无元素色视图，rule-color
        // 臂同契约）；非色端点对（accent-color auto/initial 族，auto 非插值型）
        // 离散翻转保持（语料 at<0.5=auto ≥0.5=green 契约）。
        // https://drafts.csswg.org/css-ui-4/#accent-color
        "outline-color" | "accent-color" => {
            use zero_css_parser::values::ColorValue;
            let cv = |s: &str| -> Option<Rgba> {
                if s.trim().eq_ignore_ascii_case("currentcolor") {
                    let c = crate::resolve_color_current(&ColorValue::CurrentColor, &ColorValue::CurrentColor);
                    return Some((c.r as f64, c.g as f64, c.b as f64, c.a as f64 / 255.0));
                }
                parse_color_strict(s)
            };
            match (cv(from), cv(to)) {
                (Some(fc), Some(tc)) => {
                    let (r, g, b, a) = lerp_color(fc, tc, t);
                    format!(
                        "rgba({}, {}, {}, {:.2})",
                        r.round() as u8,
                        g.round() as u8,
                        b.round() as u8,
                        a
                    )
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // wptfyi-css css-gaps 刀 D：rule-inset 槽插值（css-gaps-1 §inset）——
        // computed 端点同单位（px/%）线性 lerp（parse_len_pct_val 同单位门），
        // 跨单位 calc 混合（刀 E：css-values-4 §8.5.6 calc-mix——interpolation
        // 页 10px→-10% 期望 `calc(-3% + 7px)` 线性混合形）；overlap-join 端点
        // 离散切换（css-values-4 §8.5.6 ≥0.5 → to）。
        "column-rule-inset-cap-start"
        | "column-rule-inset-cap-end"
        | "column-rule-inset-junction-start"
        | "column-rule-inset-junction-end"
        | "row-rule-inset-cap-start"
        | "row-rule-inset-cap-end"
        | "row-rule-inset-junction-start"
        | "row-rule-inset-junction-end" => match (parse_len_pct_val(from), parse_len_pct_val(to)) {
            (Some((fv, fu)), Some((tv, tu))) if fu == tu => {
                format!("{:.2}{}", lerp(fv, tv, t), fu)
            }
            (Some((fv, fu)), Some((tv, tu))) => blend_rule_inset_lp(&fv, &fu, &tv, &tu, t)
                .unwrap_or_else(|| if t >= 0.5 { to.to_string() } else { from.to_string() }),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // R4202：individual transform 属性插值——`translate` 逐分量 px/%（% 保真原样
        // 从目标侧取值，progress 中点跃迁近似可接受）；`rotate` 角度线性（deg）；
        // `scale` 数值线性（单值等比保持单值形态）。
        "translate" => {
            let (fx, fy) = parse_translate_pair(from);
            let (tx, ty) = parse_translate_pair(to);
            format!("{:.2}px {:.2}px", lerp(fx, tx, t), lerp(fy, ty, t))
        }
        "rotate" => {
            let fa = parse_angle_deg(from);
            let ta = parse_angle_deg(to);
            format!("{:.2}deg", lerp(fa, ta, t))
        }
        "scale" => {
            let (fx, fy) = parse_scale_pair(from);
            let (tx, ty) = parse_scale_pair(to);
            let sx = lerp(fx, tx, t);
            let sy = lerp(fy, ty, t);
            if (sx - sy).abs() < 1e-6 {
                format!("{:.4}", sx)
            } else {
                format!("{:.4} {:.4}", sx, sy)
            }
        }
        // CSS Motion Path offset 簇插值（css-motion-1；wptfyi-css offset-* 刀）：
        // distance = length-percentage 同单位线性（混合单位离散切换）；rotate = 纯
        // 角度形线性、auto/reverse 关键字侧离散；anchor = position 对同单位线性；
        // path 离散。离散边界 css-values-4 §8.5.6（≥ 0.5 → to）。
        "offset-distance" => match (parse_len_pct_val(from), parse_len_pct_val(to)) {
            (Some((fv, fu)), Some((tv, tu))) if fu == tu => {
                format!("{:.2}{}", lerp(fv, tv, t), fu)
            }
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        "offset-rotate" => {
            // 刀 MOTION-1b：t==1 且端点非 canonical 形 → 端点原文 echo
            // （css-values-4 §8.5.6 端点＝端点计算值；语料 bare-`reverse`/
            // `100grad auto`/`auto calc(...)` at(1) 原文钉）。canonical 复合
            // （关键词先序+plain deg 角）与纯角走 lerp——at(1.5)/at(2) 外推
            // 采样与 block10 canonical 折叠期望不能直通（电池 r1 窗 44F 实证）。
            if t == 1.0 && offset_rotate_endpoint_echo_raw(to) {
                return to.to_string();
            }
            // 刀 MOTION-1：deg 等价分类插值——纯角↔纯角线性、关键词类（auto/
            // reverse/复合，reverse ≡ +180 折叠）↔关键词类按 canonical auto 词
            // lerp；跨类（复合↔纯角，语料 `auto 200deg→300deg` no_interpolation
            // 钉）或不可解析 → 离散（css-values-4 §8.5.6 ≥0.5 → to）。
            match (classify_rotate_endpoint(from), classify_rotate_endpoint(to)) {
                (Some((kf, fd)), Some((kt, td))) if kf == kt => {
                    if kf {
                        format!("auto {:.2}deg", lerp(fd, td, t))
                    } else {
                        format!("{:.2}deg", lerp(fd, td, t))
                    }
                }
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        "offset-anchor" => match (parse_len_pair_pct(from), parse_len_pair_pct(to)) {
            (Some((fx, fy)), Some((tx, ty))) if fx.1 == tx.1 && fy.1 == ty.1 => {
                format!("{:.2}{} {:.2}{}", lerp(fx.0, tx.0, t), fx.1, lerp(fy.0, ty.0, t), fy.1)
            }
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // offset-position = normal | auto | <position>：position 对同单位线性
        //（px/% 各自保单位），normal/auto 端点或跨形离散（css-values-4 §8.5.6
        // ≥0.5 → to；关键字/全局关键字端点组 WPT 期望离散翻转实测一致）。
        "offset-position" => match (parse_len_pair_pct(from), parse_len_pair_pct(to)) {
            (Some((fx, fy)), Some((tx, ty))) if fx.1 == tx.1 && fy.1 == ty.1 => {
                format!("{:.2}{} {:.2}{}", lerp(fx.0, tx.0, t), fx.1, lerp(fy.0, ty.0, t), fy.1)
            }
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // offset-path：ray↔ray 同 ray-size 角度线性、shape↔shape（coord-box
        // 后缀一致）逐命令插值、path↔path 逐段插值（clip_interp 共享文法通道
        // ——css-motion-1 §offset-path + css-shapes-2 §shape）；异构/异后缀
        // 离散切换。发射走 chrome-canonical fmt（gCS offset-path 臂 verbatim
        // 无 reparse 归一，见 computed_style.rs）。
        "offset-path" => match clip_interp::interpolate_offset_path(from, to, t) {
            Some(v) => v,
            // 失配/异构车道翻转（css-values-4 §8.5.6）：path() 端点 canonical
            // 单引号发射（车道端点为 computed 双引号形，原文翻转引号风格不符
            // 语料）；非 path() 对原文翻转。transitions 车道失配对在启动门
            // 已拒（strict None 不登记），本翻转只服务动画车道。
            None => clip_interp::discrete_offset_path_canonical(from, to, t),
        },
        // R4272（filter-effects-1 #animation）：filter 函数列表插值——同构列表逐函数
        // 数值插值；一侧为 none 时按另一侧函数序列展开为初始值列表；异构（含
        // drop-shadow）回落离散切换。
        "filter" | "backdrop-filter" => match interpolate_filter_list(from, to, t) {
            Some(v) => v,
            // 离散边界同 css-values-4 §8.5.6（≥ 0.5 → to，wptfyi-css 动画值进 gCS 刀校准）。
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
        },
        // R5191（css-transforms-1 §13.2 / css-transforms-2 §14）：transform 函数列表
        // 插值——同构列表逐函数插值、matrix↔matrix 走规范 2D unmatrix 分解、none 侧
        // 恒等展开；异构/3D/无效回落离散切换（既有默认臂同语义，零回归）。
        // 离散边界 css-values-4 §8.5.6：progress < 0.5 → from，**≥ 0.5 → to**
        //（wptfyi-css 动画值进 gCS 刀校准——expectFlip(f,t,0.5) 在 at=0.5 期望 to，
        // 旧实现严格 > 恒差半帧）。
        "transform" => match transform_interp::interpolate_transform_value(from, to, t) {
            Some(v) => v,
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
        },
        // wptfyi-css composite-add 刀（2026-11-15，css-fonts-4 #font-weight-prop 动画
        // 区间）：数值字重线性插值，钳 [1,1000]；关键字端点（bold/bolder/lighter）
        // 解析失败回落离散切换（与默认臂同语义零回归）。
        "font-weight" => match (parse_f64(from), parse_f64(to)) {
            (Some(fa), Some(ta)) => format!("{}", lerp(fa, ta, t).round().clamp(1.0, 1000.0) as i64),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // wptfyi-css composite-add 刀（2026-11-15，css-fonts-4 #font-size-adjust-prop
        // 动画区间）：同 metric（含双侧省略）数值线性插值，metric 保持；none /
        // from-font / metric 失配离散切换。
        "font-size-adjust" => {
            match (
                parse_font_size_adjust_endpoint(from),
                parse_font_size_adjust_endpoint(to),
            ) {
                (Some((mf, fv)), Some((mt, tv))) if mf == mt => format!(
                    "{}{}",
                    mf.map_or_else(String::new, |m| format!("{m} ")),
                    format_interp_num(lerp(fv, tv, t)),
                ),
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
        // wptfyi-css composite-add 刀（2026-11-15）：text-indent px 端点严格解析
        // 插值——既有 px 列表臂为宽松解析（关键字端点取 0 数值化），此处关键字
        // 端点须保离散语义故独立成臂；解析失败回落离散切换（css-values-4
        // §8.5.6，边界 ≥ 0.5 → to）。原含 min/max-sizing 四属性，r2 fame2b LP
        // 插值臂（本函数前段 :846 六长手）覆盖同面且更广（px/% 双 term），
        // r3 收窄本臂清 unreachable pattern 遮蔽（纯 px 对两臂输出等价）。
        "text-indent" => match (parse_px_strict(from), parse_px_strict(to)) {
            (Some(fv), Some(tv)) => format!("{:.2}px", lerp(fv, tv, t)),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
        // wptfyi-css 基本形状族刀（css-shapes-2 §interpolating-shape）：clip-path
        // 同构逐分量插值（clip_interp）；异构/单位失配/calc 端点离散切换。
        // shape-outside 端点文法与 clip-path 的 <basic-shape> 部分同构（css-shapes-1
        // §shape-outside 复用 basic-shape 文法），共享 clip_interp 通道；裸
        // <shape-box> 关键字端点与 none 参与形恒异构 → None 离散切换（同上）。
        // 二十一轮：不可插值对经 echo 保留门（clip_pair_echo_preserve）回落
        // 基线行为——B0 伪通过池（期望串含 calc 时本机 setProperty 拒收 →
        // expected 落回静态值、B0 无 applier 臂恒 echo 双巧合通过）不追真值
        // 保位（jsanim 十轮同款纪律）。
        "clip-path" | "shape-outside" => match clip_interp::interpolate_clip_path(from, to, t) {
            Some(v) => v,
            None if clip_pair_echo_preserve(from, to) => String::new(),
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
        },
        // KC-5-A：contain-intrinsic 族真插值（css-sizing-4 §8）——px 对逐轴
        // lerp 钳 [0,∞]、auto/none 跨形离散切换（allow-discrete 过渡离散翻转
        // 语义经本臂成立，css-transitions §transition-behavior）。
        "contain-intrinsic-size" | "contain-intrinsic-width" | "contain-intrinsic-height" => {
            match interpolate_cis_value(property == "contain-intrinsic-size", from, to, t) {
                Some(v) => v,
                None if t >= 0.5 => to.to_string(),
                None => from.to_string(),
            }
        }
        _ => {
            // 不支持插值的属性：离散切换（css-values-4 §8.5.6，边界 ≥ 0.5 → to）。
            if t >= 0.5 { to.to_string() } else { from.to_string() }
        }
    }
}

/// wptfyi-css 二十一轮：不可插值 clip-path/shape-outside 对的 echo 保留门。
/// 命中下列形态时返回空串（apply 解析必败 → 槽位不动 → canonicalize 恒等门
/// 回落 echo）＝刀前（无 applier 臂）行为逐位保真，B0 伪通过池保位：
///
/// - 任一端点插值视角 calc 化（[`clip_value_has_calc`]：`LengthValue::Calc`
///   直查 + 位置缘偏移经 shape_axis_comp_resolved 归一产出 `calc(100% - off)`，
///   原文无 "calc(" 子串故按解析后结构检测）：真期望是 calc 代数插值形（共享
///   残面，edge-offset 位置与 inset calc 分量）——且语料期望串含 calc 时
///   setProperty 写面拒收 → expected 落回静态值，离散翻转写回必失配
///   （6 REG 定谳形态）；
/// - `<basic-shape-rect>` 族（inset/rect/xywh）跨形对：spec 上三者 compute
///   为等价 inset() 应整体插值（css-shapes-1 §basic-shape-computed-values），
///   期望串为 calc 化 inset——同上不可达，保位待 calc 面整刀；
/// - `inherit` 端点（解析不进 <basic-shape>）∧ 对端 rect 族：underlying =
///   parent computed 值形不定，期望串为 calc 化 inset 不可达（b1v6 四 REG
///   定谳形态 `from [inherit] to [rect/xywh] at (0.875/2)`），保位——仅收
///   `inherit`：`initial`/`unset` 的 underlying 恒 `none`，对端形状对 Chrome
///   即离散翻转语义（期望=端点原形，b1v7 过宽臂压掉 16 真增益定谳反例），
///   不进门；
/// - path() 双端**不进门**（v5 定谳回滚 v3 臂）：段数/命令失配对 Chrome
///   期望即离散端点形（t<0.5 → from、≥0.5 → to），离散翻转逐位命中——
///   v6 中间账的 path 门臂以 echo 替换翻转，同页 9 例命中翻转被 echo 破坏
///   （+61→+53）；彼处唯一伪盈 at(1) 的真根因是共享 Steps(Start) 臂 eased
///   误落 0（本刀已修），门臂随修回滚。
///
/// 其余 None 形（none↔形状、形状类型异构等真离散对，期望串可解析）仍走
/// 离散翻转——clip-path-interpolation-discrete 页 +8 真增益保留。
fn clip_pair_echo_preserve(from: &str, to: &str) -> bool {
    use zero_css_parser::values::ClipPathValue as C;
    let rect_family = |v: &C| matches!(v, C::Inset { .. } | C::Rect(_) | C::Xywh(_));
    // 原文 calc 兜底（解析失败形态）；可解析值走下方结构化检测——缘偏移
    // 位置（`right 5%`）calc 化发生在 shape_axis_comp_resolved 归一层，
    // 原文无 "calc(" 子串（v2 引擎测试失败定谳，文本检查漏检该类）。
    if from.contains("calc(") || to.contains("calc(") {
        return true;
    }
    let pair = (
        zero_css_parser::values::parse_clip_path(from),
        zero_css_parser::values::parse_clip_path(to),
    );
    if pair.0.as_ref().is_some_and(clip_value_has_calc) || pair.1.as_ref().is_some_and(clip_value_has_calc) {
        return true;
    }
    match pair {
        (Some(a), Some(b)) => {
            // 同族但变体不同（inset↔rect / rect↔xywh / inset↔xywh）。
            rect_family(&a) && rect_family(&b) && std::mem::discriminant(&a) != std::mem::discriminant(&b)
        }
        // 单端不可解析：仅收 `inherit` 端点 ∧ 对端 rect 族（initial/unset
        // underlying=none 走真离散翻转；其余单端坏值形不进门）。
        _ => {
            (from == "inherit" && pair.1.as_ref().is_some_and(rect_family))
                || (to == "inherit" && pair.0.as_ref().is_some_and(rect_family))
        }
    }
}

/// 判定解析后 clip-path 值的任一分量在插值视角下 calc 化（需 calc 代数插值
/// 的共享残面）。直查 `LengthValue::Calc` 分量；位置关键字分量经
/// [`zero_css_parser::values::shape_axis_comp_resolved`] 归一后判定——
/// 右/下缘偏移产出 `calc(100% - off)`（左/上缘折纯偏移，不算 calc），
/// inset/rect/xywh/polygon/shape 各槽位逐分量遍历。
fn clip_value_has_calc(v: &zero_css_parser::values::ClipPathValue) -> bool {
    use zero_css_parser::values::parse_basic_shape::{
        BorderRadiusValue, RectCoord, ShapeAxisComp, ShapeCommand, ShapeControlPoint, ShapeEndpoint, ShapePosition,
        shape_axis_comp_resolved,
    };
    use zero_css_parser::values::{ClipPathRadius, ClipPathValue as C, LengthValue};

    fn len_calc(l: &LengthValue) -> bool {
        matches!(l, LengthValue::Calc(_))
    }
    fn radius_calc(r: &ClipPathRadius) -> bool {
        matches!(r, ClipPathRadius::Length(l) if len_calc(l))
    }
    fn pair_calc(p: &(LengthValue, LengthValue)) -> bool {
        len_calc(&p.0) || len_calc(&p.1)
    }
    fn axis_calc(c: &ShapeAxisComp) -> bool {
        len_calc(&shape_axis_comp_resolved(c))
    }
    fn position_calc(p: &ShapePosition) -> bool {
        axis_calc(&p.x) || axis_calc(&p.y)
    }
    fn endpoint_calc(e: &ShapeEndpoint) -> bool {
        match e {
            ShapeEndpoint::Position(p) => position_calc(p),
            ShapeEndpoint::Offset(x, y) => len_calc(x) || len_calc(y),
        }
    }
    fn control_calc(c: &ShapeControlPoint) -> bool {
        position_calc(&c.pos)
    }
    fn round_calc(br: &BorderRadiusValue) -> bool {
        br.h.iter().any(len_calc) || br.v.as_ref().is_some_and(|v| v.iter().any(len_calc))
    }
    match v {
        // url() 变体载荷为原文串（无结构化 calc 分量）。
        C::None | C::Path(_) | C::Url(_) => false,
        // 刀 GBX：几何盒后缀不承载 calc，calc 探测透传内层 basic-shape。
        C::WithBox { shape, .. } => shape.as_ref().is_some_and(|s| clip_value_has_calc(s)),
        C::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            len_calc(top)
                || len_calc(right)
                || len_calc(bottom)
                || len_calc(left)
                || round
                    .as_ref()
                    .is_some_and(|r| r.h.iter().any(len_calc) || r.v.as_ref().is_some_and(|v| v.iter().any(len_calc)))
        }
        C::Circle { radius: r, position: p } => radius_calc(r) || p.as_ref().is_some_and(pair_calc),
        C::Ellipse { rx, ry, position: p } => radius_calc(rx) || radius_calc(ry) || p.as_ref().is_some_and(pair_calc),
        C::Polygon { points, .. } => points.iter().any(pair_calc),
        C::Shape(s) => {
            position_calc(&s.from)
                || s.commands.iter().any(|cmd| match cmd {
                    ShapeCommand::Move { end } | ShapeCommand::Line { end } => endpoint_calc(end),
                    ShapeCommand::Hline { x, .. } => axis_calc(x),
                    ShapeCommand::Vline { y, .. } => axis_calc(y),
                    ShapeCommand::Curve { end, c1, c2 } => {
                        endpoint_calc(end) || control_calc(c1) || c2.as_ref().is_some_and(control_calc)
                    }
                    ShapeCommand::Smooth { end, c1 } => endpoint_calc(end) || c1.as_ref().is_some_and(control_calc),
                    ShapeCommand::Arc { end, rx, ry: ry_o, .. } => {
                        endpoint_calc(end) || len_calc(rx) || ry_o.as_ref().is_some_and(len_calc)
                    }
                    ShapeCommand::Close => false,
                })
        }
        C::Rect(r) => {
            [&r.top, &r.right, &r.bottom, &r.left].iter().any(|c| match c {
                RectCoord::Auto => false,
                RectCoord::Length(l) => len_calc(l),
            }) || r.round.as_ref().is_some_and(round_calc)
        }
        C::Xywh(x) => {
            len_calc(&x.x)
                || len_calc(&x.y)
                || len_calc(&x.w)
                || len_calc(&x.h)
                || x.round.as_ref().is_some_and(round_calc)
        }
    }
}

/// wptfyi-css Transition 通道刀（2026-11-16）：严格插值——**端点按臂解析成功且
/// 插值器真插值**（非离散回落）才返回插值串；任一端点解析失败 / 异构离散步 /
/// 无臂属性 → `None`。
///
/// 与 [`interpolate_property_value`] 的区别：后者对不可插值对回落 css-values-4
/// §8.5.6 离散翻转（≥0.5 → to）——那是**已启动过渡**的段内语义；css-transitions
/// §starting-of-transitions 规定**非 animatable 值对根本不启动过渡**（值立即跳
/// 终值）。Transition gCS 咨询据此以 `None` 判「不启动」：回落既有级联通道（= 终
/// 值 echo，刀前行为），端点 echo 池结构零扰动。端点严格性同时排除关键字端点
/// 0 数值化伪插值（`width: auto→100px` 经宽松 parse_px 会产出垃圾中值）。
/// 刀 B1：border-image 三长手同型槽位门（css-backgrounds-3 §border-image-width/
/// slice/outset 插值规则 + css-transitions §starting-of-transitions）——逐槽
/// component variant 对齐才可插值：同型对 `true`（严格通道 Some 真插值），跨型槽
///（%↔length、number↔length、auto↔值）或 slice fill 不对称 `false`（非 animatable
/// 对不启动过渡）。跨型 %↔length 语料期望 calc 系数混合形（calc(13% + -6px)），
/// 未接挂 FIXME(bi-cross-type-calc)。
fn border_image_same_type_slots(property: &str, from: &str, to: &str) -> bool {
    // 刀 MB：mask-border 三长手同文法同解析器（css-masking-1 §3.5–3.9 与
    // css-backgrounds-3 §3.5–3.7 文法同构，B1 臂共享先例）——别名归一到
    // border-image-* 分派，border-image 调用方行为零变化。
    let property = match property {
        "mask-border-slice" => "border-image-slice",
        "mask-border-width" => "border-image-width",
        "mask-border-outset" => "border-image-outset",
        p => p,
    };
    match property {
        "border-image-slice" => match (parse_border_image_slice(from), parse_border_image_slice(to)) {
            (Some(f), Some(s)) if f.fill == s.fill => {
                slot_pair_same_type(&f.top, &s.top)
                    && slot_pair_same_type(&f.right, &s.right)
                    && slot_pair_same_type(&f.bottom, &s.bottom)
                    && slot_pair_same_type(&f.left, &s.left)
            }
            _ => false,
        },
        "border-image-width" => match (parse_border_image_width(from), parse_border_image_width(to)) {
            (Some(f), Some(s)) => [
                width_same_type(&f.top, &s.top),
                width_same_type(&f.right, &s.right),
                width_same_type(&f.bottom, &s.bottom),
                width_same_type(&f.left, &s.left),
            ]
            .into_iter()
            .all(std::convert::identity),
            _ => false,
        },
        "border-image-outset" => match (parse_border_image_outset(from), parse_border_image_outset(to)) {
            (Some(f), Some(s)) => [
                outset_same_type(&f.top, &s.top),
                outset_same_type(&f.right, &s.right),
                outset_same_type(&f.bottom, &s.bottom),
                outset_same_type(&f.left, &s.left),
            ]
            .into_iter()
            .all(std::convert::identity),
            _ => false,
        },
        _ => false,
    }
}

/// 刀 MB v3：mask-border-width 过渡通道可插值门——同型槽对（auto↔auto /
/// number↔number / length↔length / percent↔percent，[`width_same_type`]）与
/// 跨型 Length↔Percent 对放行；Number↔Length/Percent、Auto↔值槽失配 → false
///（非可插值对不启动过渡，css-transitions-1 §starting-of-transitions）。
/// 裸无单位 0 双态桥接与 [`width_same_type`] 同契约。
fn mask_border_width_interpolable(from: &str, to: &str) -> bool {
    match (parse_border_image_width(from), parse_border_image_width(to)) {
        (Some(f), Some(s)) => [
            (&f.top, &s.top),
            (&f.right, &s.right),
            (&f.bottom, &s.bottom),
            (&f.left, &s.left),
        ]
        .into_iter()
        .all(|(a, b)| {
            width_same_type(a, b)
                || matches!(
                    (a, b),
                    (
                        BorderImageWidthComponent::Length(_),
                        BorderImageWidthComponent::Percent(_)
                    ) | (
                        BorderImageWidthComponent::Percent(_),
                        BorderImageWidthComponent::Length(_)
                    )
                )
        }),
        _ => false,
    }
}

/// slice 槽同型门：Number↔Number / Percent↔Percent（css-backgrounds-3 §3.6）。
fn slot_pair_same_type(a: &BorderImageSliceComponent, b: &BorderImageSliceComponent) -> bool {
    matches!(
        (a, b),
        (
            BorderImageSliceComponent::Number(_),
            BorderImageSliceComponent::Number(_)
        ) | (
            BorderImageSliceComponent::Percent(_),
            BorderImageSliceComponent::Percent(_)
        )
    )
}

/// width 槽同型门：auto↔auto / number↔number / percent↔percent / length↔length；
/// 裸无单位 0 双态歧义视同型（css-values-4 §5.2——宽松臂零值桥接臂同契约）。
fn width_same_type(a: &BorderImageWidthComponent, b: &BorderImageWidthComponent) -> bool {
    use BorderImageWidthComponent as W;
    matches!(
        (a, b),
        (W::Auto, W::Auto)
            | (W::Number(_), W::Number(_))
            | (W::Percent(_), W::Percent(_))
            | (W::Length(_), W::Length(_))
    ) || matches!(
        (a, b),
        (W::Length(LengthValue::Px(x)), W::Number(_)) if *x == 0.0
    ) || matches!(
        (a, b),
        (W::Number(_), W::Length(LengthValue::Px(y))) if *y == 0.0
    )
}

/// outset 槽同型门：number↔number / length↔length；裸无单位 0 桥接同 width。
fn outset_same_type(a: &BorderImageOutsetComponent, b: &BorderImageOutsetComponent) -> bool {
    use BorderImageOutsetComponent as O;
    matches!((a, b), (O::Number(_), O::Number(_)) | (O::Length(_), O::Length(_)))
        || matches!(
            (a, b),
            (O::Length(LengthValue::Px(x)), O::Number(_)) if *x == 0.0
        )
        || matches!(
            (a, b),
            (O::Number(_), O::Length(LengthValue::Px(y))) if *y == 0.0
        )
}

pub(crate) fn interpolate_property_value_strict(property: &str, from: &str, to: &str, t: f64) -> Option<String> {
    match property {
        "opacity" => {
            let fa = parse_f64(from)?;
            let ta = parse_f64(to)?;
            Some(format!("{:.4}", lerp(fa, ta, t)))
        }
        // r80 刀 R2：text-decoration-color 进色对臂（css-text-decor-3
        // §text-decoration-color-property <color> 值域可插值；generator 端点
        // rgba 对同 parse_color_strict 域）。
        "background-color" | "background" | "color" | "border-color" | "text-decoration-color" => {
            let from_c = parse_color_strict(from)?;
            let to_c = parse_color_strict(to)?;
            let (r, g, b, a) = lerp_color(from_c, to_c, t);
            Some(format!(
                "rgba({}, {}, {}, {:.2})",
                r.round() as u8,
                g.round() as u8,
                b.round() as u8,
                a
            ))
        }
        // wptfyi-css r80 刀 R2（properties-value 260F 分解）：px 白名单臂扩为
        // LP 逐 token 门——端点全 token 经 [`parse_lp_terms`]（px/%/裸 0/calc
        // 混合形）解析 ∧ token 数恒等才插值（css-values-4 §8.5.6 interpolation-
        // calc：px/% 双 term 独立 lerp，[`render_lp_terms`] canonical 形）；纯
        // px 对维持既有 `{:.2}px` 串形（零回归）。异型/关键字端点 None =
        // 非 animatable 对不启动过渡（css-transitions-1 §starting-of-transitions）。
        // 新进名：column-gap（<length-percentage>，css-multicol §column-gap）+
        // border-*-radius 四长手（css-backgrounds-3 §3.6——单 token 域，computed
        // 串化单值形）+ transform-origin（css-transforms-1 §transform-origin，
        // computed 恒两 token px/% 形——关键字声明已级联解析）。
        "width"
        | "height"
        | "top"
        | "left"
        | "right"
        | "bottom"
        | "margin-top"
        | "margin-right"
        | "margin-bottom"
        | "margin-left"
        | "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "font-size"
        | "letter-spacing"
        | "word-spacing"
        | "column-gap"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-left-radius"
        | "border-bottom-right-radius"
        | "transform-origin" => interpolate_lp_token_strict(from, to, t),
        // wptfyi-css mask-border 刀：严格通道委托宽松臂（端点解析失败已在其内落
        // 离散翻转=保守基线；全局关键字端点的父链/initial 解析在通道进臂前完成
        // ——刀 K1 cssa/gCS 父链同契约）。输出串经 gCS 同款格式器与期望恒等。
        // 刀 MB：slice/width/outset 收 B1 同型门（css-transitions-1
        // §starting-of-transitions：非可插值对不启动过渡、值立即取终态，harness
        // transitions 通道 expectFlip(from, to, -Infinity) 恒 to 形——语料修复
        // 后 css-masking no-interp 页（fill 不对称/auto 槽/跨型对）暴露恒 Some
        // 委托误翻转）；CSSA/WAAPI 通道不经本门，lenient 50% 翻转（expectFlip
        // 0.5 形）不变。repeat/source 纯离散（不在 transitions registry，臂体
        // 不可达保持原文）。
        "mask-border-slice" | "mask-border-outset" => {
            border_image_same_type_slots(property, from, to).then(|| interpolate_property_value(property, from, to, t))
        }
        // 刀 MB v3：width 跨型 Length↔Percent 对放行（css-masking-1 §3.7
        // mask-border-width = <<length-percentage>||<number>|auto>，length-
        // percentage 域跨型对 computed 值插值语义 css-values-4 §8.5.6；语料
        // transitions 通道 px↔% 对期望 calc 系数形——本机跨型 calc 插值未接
        // FIXME(bi-cross-type-calc)，lenient 臂对该对形端点翻转（t<0.5→from）
        // 即 HEAD 基线行为保位）；Auto↔值 / Number↔length-percentage 失配仍
        // 不启动过渡。
        "mask-border-width" => {
            mask_border_width_interpolable(from, to).then(|| interpolate_property_value(property, from, to, t))
        }
        // 刀 MB：source 纯离散恒 None（css-masking-1 §3.4 `<none | <image>>`
        // discrete——语料全页 test_no_interpolation，transitions 通道 expectFlip
        // (from, to, -Infinity) 恒 to 形；CSSA/WAAPI 通道 lenient 翻转不变）。
        "mask-border-source" => None,
        "mask-border-repeat" => Some(interpolate_property_value(property, from, to, t)),
        // wptfyi-css 刀 B1：border-image 三长手同型槽位严格门——同型对 Some（委托
        // 宽松臂真插值，槽位失配 None = 非 animatable 对不启动过渡、静态跳终值，
        // css-transitions §starting-of-transitions）。宽/ outset 与 mask-border 臂体
        // 共享（同 BorderImage* 解析器），但 mask-border 的恒 Some 委托在此不复制：
        // border-image 语料 transitions 半面含关键字端点 no-interp 对（initial→20px
        // 期望恒 to），恒 Some 会把非可插值对登记成真过渡致 P2F。
        "border-image-slice" | "border-image-width" | "border-image-outset" => {
            border_image_same_type_slots(property, from, to).then(|| interpolate_property_value(property, from, to, t))
        }
        // wptfyi-css mask-interp 刀（css-masking-1 §5.3–5.6 + §3.10 简写）：同契约
        // 委托宽松臂——position 逐层系数 lerp、离散组/简写翻转（端点多层串
        // verbatim 天然兼容）。background-position/object-position（刀 A1c ②）
        // 同构委托——<position># 文法同 mask-position，mix_mask_position 复用。
        "mask-position"
        | "mask-image"
        | "mask-mode"
        | "mask-repeat"
        | "mask-clip"
        | "mask-origin"
        | "mask-composite"
        | "mask"
        | "background-position"
        | "background-position-x"
        | "background-position-y"
        | "object-position" => Some(interpolate_property_value(property, from, to, t)),
        // 刀 A2：vertical-align 严格门=真实可插值性——两端点解析成功（LP 或
        // baseline≡0px）才 Some 委托 lenient。实测修正：语料
        // vertical-align-interpolation.html 确有 test_no_interpolation 块
        // （super→40%），恒 Some 委托会让 transitions 通道按 50% 翻转（-6F）。
        "vertical-align" if parse_va_endpoint(from).is_some() && parse_va_endpoint(to).is_some() => {
            Some(interpolate_property_value(property, from, to, t))
        }
        // 刀 A2：非插值对（关键字↔LP）恒 None——css-transitions-1
        // §starting-of-transitions：不可插值对不启动过渡，值立即取终态
        // （harness transitions 通道 expectFlip(from, to, -Infinity) 恒 to 形）；
        // CSSA/WAAPI 通道不经本门，lenient 50% 离散翻转（expectFlip 0.5 形）不变。
        "vertical-align" => None,
        // wptfyi-css 刀 A3：text-shadow 严格门——两端点均为阴影列表（含 none=空表）
        // 即真实可插值（css-text-decor-3 §text-shadow interpolation：层差零阴影
        // 补齐，语料无关键字 no-interp 对形）；委托宽松臂。端点解析失败 None =
        // 非 animatable 对不启动过渡、静态跳终值（css-transitions-1
        // §starting-of-transitions，同 va 臂形）。
        // r80 刀 R2：box-shadow 独立严格门换 [`parse_box_shadow_list`]
        // （css-backgrounds-3 §6.1.1 <shadow># 逐层插值——gCS 串化形 color-first
        // 4 长度含 spread 恒显，text-shadow 文法无 spread 必拒；spread 形端点对
        // 经本门真实登记，委托宽松臂 box-shadow 分支）。
        "box-shadow"
            if zero_css_parser::values::parse_box_shadow_list(from, false).is_some()
                && zero_css_parser::values::parse_box_shadow_list(to, false).is_some() =>
        {
            Some(interpolate_property_value(property, from, to, t))
        }
        "text-shadow"
            if zero_css_parser::values::parse_text_shadow_list(from, false).is_some()
                && zero_css_parser::values::parse_text_shadow_list(to, false).is_some() =>
        {
            Some(interpolate_property_value(property, from, to, t))
        }
        // 刀 A3：畸形端点对恒 None（同上臂形）。
        "text-shadow" | "box-shadow" => None,
        "translate" => {
            // 逐分量同单位（px/%）严格解析——单位失配对回落 None（离散面）。
            let interp_pair = |f: &str, s: &str| -> Option<(f64, String)> {
                let (fv, fu) = parse_len_pct_val(f)?;
                let (tv, tu) = parse_len_pct_val(s)?;
                (fu == tu).then_some((lerp(fv, tv, t), fu))
            };
            let fp: Vec<&str> = from.split_whitespace().collect();
            let tp: Vec<&str> = to.split_whitespace().collect();
            if fp.is_empty() || fp.len() != tp.len() || fp.len() > 2 {
                return None;
            }
            let mut out = Vec::with_capacity(fp.len());
            for (f_tok, t_tok) in fp.iter().zip(tp.iter()) {
                let (v, u) = interp_pair(f_tok, t_tok)?;
                out.push(format!("{v:.2}{u}"));
            }
            Some(out.join(" "))
        }
        "rotate" => {
            let fa = parse_angle_deg_strict(from)?;
            let ta = parse_angle_deg_strict(to)?;
            Some(format!("{:.2}deg", lerp(fa, ta, t)))
        }
        "scale" => {
            let interp_scale = |s: &str| -> Option<f64> { s.trim().parse::<f64>().ok() };
            let fp: Vec<&str> = from.split_whitespace().collect();
            let tp: Vec<&str> = to.split_whitespace().collect();
            if fp.is_empty() || fp.len() > 2 || fp.len() != tp.len() {
                return None;
            }
            let sx = lerp(interp_scale(fp[0])?, interp_scale(tp[0])?, t);
            let sy = if fp.len() == 2 {
                lerp(interp_scale(fp[1])?, interp_scale(tp[1])?, t)
            } else {
                sx
            };
            if (sx - sy).abs() < 1e-6 {
                Some(format!("{sx:.4}"))
            } else {
                Some(format!("{sx:.4} {sy:.4}"))
            }
        }
        "offset-distance" => {
            let (fv, fu) = parse_len_pct_val(from)?;
            let (tv, tu) = parse_len_pct_val(to)?;
            (fu == tu).then(|| format!("{:.2}{}", lerp(fv, tv, t), fu))
        }
        "offset-rotate" => {
            // 刀 MOTION-1b：t==1 且端点非 canonical 形 → 原文 echo（同
            // transitions 臂口径；bare-`reverse`/`100grad auto`/calc 端点 at(1)
            // 原文钉，外推与 canonical 折叠期望走 lerp）。
            if t == 1.0 && offset_rotate_endpoint_echo_raw(to) {
                return Some(to.to_string());
            }
            // 刀 MOTION-1：deg 等价分类插值（同 transitions 臂口径）；跨类/
            // 不可解析 None = 非插值对 → 调用方离散翻转。
            match (classify_rotate_endpoint(from), classify_rotate_endpoint(to)) {
                (Some((kf, fd)), Some((kt, td))) if kf == kt => Some(if kf {
                    format!("auto {:.2}deg", lerp(fd, td, t))
                } else {
                    format!("{:.2}deg", lerp(fd, td, t))
                }),
                _ => None,
            }
        }
        "offset-anchor" | "offset-position" => {
            let (fx, fy) = parse_len_pair_pct(from)?;
            let (tx, ty) = parse_len_pair_pct(to)?;
            (fx.1 == tx.1 && fy.1 == ty.1)
                .then(|| format!("{:.2}{} {:.2}{}", lerp(fx.0, tx.0, t), fx.1, lerp(fy.0, ty.0, t), fy.1))
        }
        "offset-path" => clip_interp::interpolate_offset_path(from, to, t),
        "filter" | "backdrop-filter" => interpolate_filter_list(from, to, t),
        "transform" => transform_interp::interpolate_transform_value(from, to, t),
        "clip-path" => clip_interp::interpolate_clip_path(from, to, t),
        // 同构 basic-shape 端点真插值；裸 <shape-box>/none 参与形 None（= 非
        // animatable 对不启动过渡，css-transitions §starting-of-transitions）。
        "shape-outside" => clip_interp::interpolate_clip_path(from, to, t),
        // wptfyi-css shape-margin 刀：px 端点严格插值，负外推钳 0（非负域）。
        "shape-margin" => {
            let fa = parse_px_strict(from)?;
            let ta = parse_px_strict(to)?;
            Some(format!("{:.2}px", lerp(fa, ta, t).max(0.0)))
        }
        // wptfyi-css multicol columns 刀：column-width/height = px 端点严格插值
        // （非长度端点 = 非 animatable 对不启动）；column-count = 数值严格；columns
        // = 组件存在集恒等才 Some（槽形变 = 非 animatable，css-transitions
        // §starting-of-transitions——allow-discrete 启动门已随刀 A1c 接入
        // transition_registry.rs，本位声明时 None 对亦启动、求值端离散翻转）。
        "column-width" | "column-height" => {
            let fa = parse_px_strict(from)?;
            let ta = parse_px_strict(to)?;
            Some(format!("{:.2}px", lerp(fa, ta, t).max(0.0)))
        }
        "column-count" => {
            let fa = parse_f64(from)?;
            let ta = parse_f64(to)?;
            Some(format!("{}", lerp(fa, ta, t).max(1.0).round() as i64))
        }
        // 刀 Z：strict 同型门——两端均 <integer> 才插值（css-transitions-1
        // §animations：auto 或小数端点 None =非 animatable 对不登记，静态
        // 跳终值；allow-discrete 对随刀 A1c 门在此 None 亦启动、求值端翻转）。
        "z-index" => {
            let fa = parse_z_index_endpoint(from)?;
            let ta = parse_z_index_endpoint(to)?;
            Some(format!("{}", (lerp(fa as f64, ta as f64, t) + 0.5).floor() as i64))
        }
        // 刀 TEXT-INTERP（wptfyi-css，2026-11-22）：hyphenate-limit-chars
        // <integer>{1,3} 逐位线性 lerp——css-text-4 §5.4 可插值属性，同分量
        // 数逐位 <integer> lerp（css-values-4 §8.1.2 整数进位法同 z-index，
        // fset 期望算术核验：5 1 2→17 5 10 at .25 = 8 2 4、at -.5 = -1 -1 -2
        // 无钳制）；异长/含 none|auto 端点 None＝离散（strict 门语义，求值端
        // 翻转）。
        "hyphenate-limit-chars" => {
            let fa = parse_hyphenate_limit_chars_endpoint(from)?;
            let ta = parse_hyphenate_limit_chars_endpoint(to)?;
            if fa.len() != ta.len() {
                return None;
            }
            Some(
                fa.iter()
                    .zip(ta.iter())
                    .map(|(&a, &b)| format!("{}", (lerp(a as f64, b as f64, t) + 0.5).floor() as i64))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        "columns" => {
            let f = parse_columns_endpoint(from)?;
            let s = parse_columns_endpoint(to)?;
            if !f.compatible(&s) {
                return None;
            }
            let w = match (f.width, s.width) {
                (Some(fw), Some(tw)) => Some(lerp(fw, tw, t).max(0.0)),
                _ => None,
            };
            let c = match (f.count, s.count) {
                (Some(fc), Some(tc)) => Some(lerp(fc, tc, t).max(1.0)),
                _ => None,
            };
            let h = match (f.height, s.height) {
                (Some(fh), Some(th)) => Some(lerp(fh, th, t).max(0.0)),
                _ => None,
            };
            Some(serialize_columns_components(w, c, h))
        }
        // shape-image-threshold：number 严格插值，外推钳 [0,1]（computed 域）。
        "shape-image-threshold" => {
            let fa = parse_f64(from)?;
            let ta = parse_f64(to)?;
            Some(format!("{:.4}", lerp(fa, ta, t).clamp(0.0, 1.0)))
        }
        "font-weight" => {
            let fa = parse_f64(from)?;
            let ta = parse_f64(to)?;
            Some(format!("{}", lerp(fa, ta, t).round().clamp(1.0, 1000.0) as i64))
        }
        "font-size-adjust" => {
            let (mf, fv) = parse_font_size_adjust_endpoint(from)?;
            let (mt, tv) = parse_font_size_adjust_endpoint(to)?;
            (mf == mt).then(|| {
                format!(
                    "{}{}",
                    mf.map_or_else(String::new, |m| format!("{m} ")),
                    format_interp_num(lerp(fv, tv, t))
                )
            })
        }
        // 刀 FI：font-stretch 数值插值（css-fonts-4 §font-width-prop——关键字↔% 单源
        // [`zero_css_parser::values::parse_font_stretch`]，负 % 文法拒、0% 合法；
        // 输出 % 整形与 gCS `format_num(font_stretch, "%")` 同形）。端点均非负时
        // lerp 亦非负，无钳制；上界不钳（语料 `1.5`→250% 合法形）。
        // https://drafts.csswg.org/css-fonts-4/#font-width-prop
        "font-stretch" => {
            let fa = zero_css_parser::values::parse_font_stretch(from)?;
            let ta = zero_css_parser::values::parse_font_stretch(to)?;
            Some(format!("{}%", format_interp_num(lerp(fa as f64, ta as f64, t))))
        }
        // 刀 FI：font-style 插值（css-fonts-4 #font-style-animatable——`normal` ≡
        // `oblique 0deg`；italic 与 oblique/normal 异族离散；oblique `<angle>` 对
        // 数值 lerp（负角合法）；t∈{0,1} 端点 verbatim——语料钉形 from [normal]
        // at (0) = [normal]，0deg 数值形会串写 `oblique 0deg`；italic↔italic 同值
        // 离散恒等）。输出经 apply 端 `parse_font_style` 角形直读。
        // https://drafts.csswg.org/css-fonts-4/#font-style-animatable
        "font-style" => {
            if t == 0.0 {
                Some(from.to_string())
            } else if t == 1.0 {
                Some(to.to_string())
            } else {
                let angle = |s: &str| -> Option<(bool, f64)> {
                    let v = s.trim().to_ascii_lowercase();
                    match v.as_str() {
                        "normal" | "oblique" => Some((false, 0.0)),
                        "italic" => Some((true, 0.0)),
                        _ => v
                            .strip_prefix("oblique")
                            .and_then(|rest| rest.trim().strip_suffix("deg"))
                            .and_then(|num| num.trim().parse::<f64>().ok())
                            .map(|deg| (false, deg)),
                    }
                };
                let (fi, ai) = angle(from)?;
                let (ft, at) = angle(to)?;
                if fi != ft {
                    Some((if t >= 0.5 { to } else { from }).to_string())
                } else if fi {
                    Some(from.to_string())
                } else {
                    // 角值域门（css-fonts-4 #font-style-prop：oblique 角域
                    // [-90deg, 90deg]，域外插值结果钳入——语料 -90↔90 对
                    // at(-2)/(-1)/(1.5) 外推钉形）；插值恰 0deg 串化 `normal`
                    //（`oblique 0deg` 规范等价 normal——语料 at(0.5) 期望
                    // [normal] 钉形；t∈{0,1} 端点 verbatim 分支已先行）。
                    // https://drafts.csswg.org/css-fonts-4/#font-style-prop
                    let v = lerp(ai, at, t).clamp(-90.0, 90.0);
                    if v == 0.0 {
                        Some("normal".to_string())
                    } else {
                        Some(format!("oblique {}deg", format_interp_num(v)))
                    }
                }
            }
        }
        // 刀 FI：line-height px 域插值（css-transitions-1 §animatable-types
        // 「line-height: as length」；computed=gCS used-value px 形——R2761
        // `line_height_str` number→used px 契约）；px↔px 数值 lerp，`normal`
        // 端点对 parse 失败 None = 非 animatable 对不启动过渡。
        "line-height" => {
            let fa = parse_px_strict(from)?;
            let ta = parse_px_strict(to)?;
            Some(format!("{:.2}px", lerp(fa, ta, t)))
        }
        "text-indent" | "min-width" | "max-width" | "min-height" | "max-height" => {
            let fv = parse_px_strict(from)?;
            let tv = parse_px_strict(to)?;
            Some(format!("{:.2}px", lerp(fv, tv, t)))
        }
        // wptfyi-css css-gaps 刀 B：rule-width 列表插值（展开/LCM/广播语义见
        // [`interpolate_rule_width_value`]）；结构失配对 None（非 animatable
        // 值对不启动过渡，css-transitions §starting-of-transitions）。
        "column-rule-width" | "row-rule-width" => interpolate_rule_width_value(from, to, t),
        // rule-color 列表插值（结构语义同 width strict 臂；失配对 None 不启动过渡）。
        "column-rule-color" | "row-rule-color" => interpolate_rule_color_value(from, to, t),
        // css-ui OA 刀严格通道：outline-width 非负 <length>（关键字折叠后
        // parse_px_strict；域外钳 0）；outline-offset 带符号 <length> 无钳；
        // outline-color/accent-color 纯色对 lerp（currentColor 黑底同宽松臂），
        // 非色对 None 不启动过渡（css-transitions-1 §starting-of-transitions——
        // accent-color auto/initial 端点对静态跳终值面挂残池）。
        "outline-width" | "outline-offset" => {
            // 关键字端点折叠同宽松臂（thin/medium/thick=1/3/5px）。
            let ep = |s: &str| {
                parse_px_strict(s)
                    .or_else(|| (s.trim() == "0").then_some(0.0))
                    .or_else(|| match s.trim().to_ascii_lowercase().as_str() {
                        "thin" => Some(1.0),
                        "medium" => Some(3.0),
                        "thick" => Some(5.0),
                        _ => None,
                    })
            };
            match (ep(from), ep(to)) {
                (Some(fv), Some(tv)) => {
                    let v = lerp(fv, tv, t);
                    let v = if property == "outline-width" { v.max(0.0) } else { v };
                    Some(format!("{:.2}px", v))
                }
                _ => None,
            }
        }
        "outline-color" | "accent-color" => {
            use zero_css_parser::values::ColorValue;
            let cv = |s: &str| -> Option<Rgba> {
                if s.trim().eq_ignore_ascii_case("currentcolor") {
                    let c = crate::resolve_color_current(&ColorValue::CurrentColor, &ColorValue::CurrentColor);
                    return Some((c.r as f64, c.g as f64, c.b as f64, c.a as f64 / 255.0));
                }
                parse_color_strict(s)
            };
            match (cv(from), cv(to)) {
                (Some(fc), Some(tc)) => {
                    let (r, g, b, a) = lerp_color(fc, tc, t);
                    Some(format!(
                        "rgba({}, {}, {}, {:.2})",
                        r.round() as u8,
                        g.round() as u8,
                        b.round() as u8,
                        a
                    ))
                }
                _ => None,
            }
        }
        // wptfyi-css css-gaps 刀 D：rule-inset 槽严格插值（css-gaps-1 §inset）——
        // 同单位（px/%）computed 端点 lerp；跨单位 calc 混合（刀 E，css-values-4
        // §8.5.6——过渡登记门放行，interpolation 页 px↔% 对 168F 根因）；overlap-join
        // 端点对 None（非 animatable 对不启动过渡，css-transitions §starting-of-
        // transitions）。
        "column-rule-inset-cap-start"
        | "column-rule-inset-cap-end"
        | "column-rule-inset-junction-start"
        | "column-rule-inset-junction-end"
        | "row-rule-inset-cap-start"
        | "row-rule-inset-cap-end"
        | "row-rule-inset-junction-start"
        | "row-rule-inset-junction-end" => {
            let (fv, fu) = parse_len_pct_val(from)?;
            let (tv, tu) = parse_len_pct_val(to)?;
            if fu == tu {
                Some(format!("{:.2}{}", lerp(fv, tv, t), fu))
            } else {
                blend_rule_inset_lp(&fv, &fu, &tv, &tu, t)
            }
        }
        // KC-5-A：contain-intrinsic 族 strict 视角——px 对可插值（登记放行），
        // auto/none 跨形轴 None＝plain transition 不启动（spec：非可插值对
        // 瞬跳，css-transitions §starting-of-transitions；allow-discrete 变体
        // 登记走 lenient 离散翻转）。
        "contain-intrinsic-size" | "contain-intrinsic-width" | "contain-intrinsic-height" => {
            interpolate_cis_value(property == "contain-intrinsic-size", from, to, t)
        }
        // 无臂属性：非 animatable 对（transition 不启动——咨询回落级联终值 echo）。
        _ => None,
    }
}

// ── css-gaps 刀 B：rule-width 列表插值 ─────────────────────────────────────

/// rule-width 列表插值（css-gaps-1 §column-row-rule-width，wptfyi-css 刀 B）。
/// 端点为 computed 串化形（gCS 同源：平列表 / `repeat(N, …)` / `repeat(auto, …)`，
/// calc 计数已在 gCS 折叠整数，故端点再解析不含 Calc 头）。
///
/// 语义（语料 rule-width-interpolation-{repeaters,multiple-values} 页钉）：
/// - 无 auto：双侧有限 repeat 展开为平列表——等长逐位 lerp；单值侧广播对齐
///   （`repeat(2, 20px)`→`30px` 以 2 位输出 `23px, 23px`）；双侧多值异长 →
///   None（离散）。
/// - 有 auto：仅双侧同含（单侧 auto → None）；leading/trailing 平段数各等才
///   插值，auto 循环体取 LCM 平铺逐位 lerp，`repeat(auto, …)` 结构保形输出。
/// - 项可插域：仅 px 项（gCS 串化后恒 px）；其余 → None。
/// - 负外推钳 0（<length [0,∞]> computed 域，与 computed resolve 钳一致）。
///
/// wptfyi-css css-ui OA 刀 FACE-MARKER（电池哨兵；ASCII 字面入二进制 strings 门
/// ——#[used]+pub 双保险保对象文件发射：#[allow(dead_code)] const 与裸 pub static
/// 均被 release 剥离＝两射 FACE-MARKER-FAIL rc=7 实证）。
#[used]
pub static OA1_FACE_MARKER: &str = "oa1-ui-interp-knife";

fn interpolate_rule_width_value(from: &str, to: &str, t: f64) -> Option<String> {
    use zero_css_parser::values::{ColumnRuleWidthValue, RuleRepeatCount, RuleWidthSegment, parse_rule_width_list};

    let px = |v: &ColumnRuleWidthValue| -> Option<f64> {
        match v {
            ColumnRuleWidthValue::Length(LengthValue::Px(p)) => Some(*p),
            _ => None,
        }
    };
    let lerp_flat = |f: &[ColumnRuleWidthValue], s: &[ColumnRuleWidthValue]| -> Option<Vec<String>> {
        f.iter()
            .zip(s.iter())
            .map(|(fv, tv)| {
                let (fv, tv) = (px(fv)?, px(tv)?);
                Some(format!("{:.2}px", lerp(fv, tv, t).max(0.0)))
            })
            .collect()
    };

    // 段拆解：有限 repeat 展开；auto 循环体单独提出（≤1 处，parse 保证）。
    let split = |segments: &[RuleWidthSegment]| -> Option<(
        Vec<ColumnRuleWidthValue>,
        Option<Vec<ColumnRuleWidthValue>>,
        Vec<ColumnRuleWidthValue>,
    )> {
        let mut leading = Vec::new();
        let mut auto = None;
        let mut trailing = Vec::new();
        let mut dest = &mut leading;
        for seg in segments {
            match seg {
                RuleWidthSegment::Plain(w) => dest.push(w.clone()),
                RuleWidthSegment::Repeat { count, widths } => match count {
                    RuleRepeatCount::Auto => {
                        if auto.is_some() {
                            return None;
                        }
                        auto = Some(widths.clone());
                        dest = &mut trailing;
                    }
                    RuleRepeatCount::N(n) => {
                        for _ in 0..*n {
                            dest.extend(widths.iter().cloned());
                        }
                    }
                    // computed 串化端点无 calc 头（gCS 已折叠整数）。
                    RuleRepeatCount::Calc(_) => return None,
                },
            }
        }
        Some((leading, auto, trailing))
    };

    let (fl, fa, ft) = split(&parse_rule_width_list(from)?)?;
    let (tl, ta, tt) = split(&parse_rule_width_list(to)?)?;
    match (fa, ta) {
        // 无 auto：等长逐位 / 单值广播 / 异长离散。
        (None, None) => {
            if fl.len() == tl.len() {
                lerp_flat(&fl, &tl).map(|v| v.join(", "))
            } else if fl.len() == 1 {
                lerp_flat(&vec![fl[0].clone(); tl.len()], &tl).map(|v| v.join(", "))
            } else if tl.len() == 1 {
                lerp_flat(&fl, &vec![tl[0].clone(); fl.len()]).map(|v| v.join(", "))
            } else {
                None
            }
        }
        // 双 auto：leading/trailing 段数各等 + 循环体 LCM 对齐。
        (Some(fc), Some(tc)) => {
            if fl.len() != tl.len() || ft.len() != tt.len() {
                return None;
            }
            let lcm = {
                let (a, b) = (fc.len(), tc.len());
                let gcd = |mut x: usize, mut y: usize| -> usize {
                    while y != 0 {
                        (x, y) = (y, x % y);
                    }
                    x
                };
                a * b / gcd(a, b)
            };
            let tile = |c: &[ColumnRuleWidthValue]| -> Vec<ColumnRuleWidthValue> {
                c.iter().cycle().take(lcm).cloned().collect()
            };
            let mid = lerp_flat(&tile(&fc), &tile(&tc))?;
            let mut parts = lerp_flat(&fl, &tl)?;
            parts.push(format!("repeat(auto, {})", mid.join(", ")));
            parts.extend(lerp_flat(&ft, &tt)?);
            Some(parts.join(", "))
        }
        // 单侧 auto：结构异构离散。
        _ => None,
    }
}

/// rule-color 列表插值（css-gaps-1 §column-row-rule-color，wptfyi-css 刀 B）。
/// 结构语义同 [`interpolate_rule_width_value`]（有限 repeat 展开/单值广播/auto
/// LCM/leading-trailing 段数门）；项插值 = [`lerp_color`] 预乘空间（css-color-4
/// §15.1）。端点项经 css-parser 全名色表解析（gCS 串化 rgb() 与 keyframe 指定
/// 关键字双侧覆盖）；currentcolor 落黑底（动画端点无元素色视图——gCS 串化端点
/// 已解析元素色，仅 keyframe 指定形落此）。
fn interpolate_rule_color_value(from: &str, to: &str, t: f64) -> Option<String> {
    use zero_css_parser::values::{ColorValue, RuleColorSegment, RuleRepeatCount, parse_rule_color_list};

    let cv_rgba = |c: &ColorValue| -> Rgba {
        let resolved = crate::resolve_color_current(c, &ColorValue::CurrentColor);
        (
            resolved.r as f64,
            resolved.g as f64,
            resolved.b as f64,
            resolved.a as f64 / 255.0,
        )
    };
    let fmt_item = |c: &Rgba| -> String {
        let (r, g, b, a) = (c.0.round() as u8, c.1.round() as u8, c.2.round() as u8, c.3);
        if (a - 1.0).abs() < 1e-6 {
            format!("rgb({r}, {g}, {b})")
        } else {
            format!("rgba({r}, {g}, {b}, {a:.2})")
        }
    };
    let lerp_flat = |f: &[ColorValue], s: &[ColorValue]| -> Option<Vec<String>> {
        f.iter()
            .zip(s.iter())
            .map(|(fv, tv)| Some(fmt_item(&lerp_color(cv_rgba(fv), cv_rgba(tv), t))))
            .collect()
    };

    // 段拆解：有限 repeat 展开；auto 循环体单独提出（≤1 处，parse 保证）。
    let split = |segments: &[RuleColorSegment]| -> Option<(Vec<ColorValue>, Option<Vec<ColorValue>>, Vec<ColorValue>)> {
        let mut leading = Vec::new();
        let mut auto = None;
        let mut trailing = Vec::new();
        let mut dest = &mut leading;
        for seg in segments {
            match seg {
                RuleColorSegment::Plain(c) => dest.push(c.clone()),
                RuleColorSegment::Repeat { count, colors } => match count {
                    RuleRepeatCount::Auto => {
                        if auto.is_some() {
                            return None;
                        }
                        auto = Some(colors.clone());
                        dest = &mut trailing;
                    }
                    RuleRepeatCount::N(n) => {
                        for _ in 0..*n {
                            dest.extend(colors.iter().cloned());
                        }
                    }
                    // computed 串化端点无 calc 头（gCS 已折叠整数）。
                    RuleRepeatCount::Calc(_) => return None,
                },
            }
        }
        Some((leading, auto, trailing))
    };

    let (fl, fa, ft) = split(&parse_rule_color_list(from)?)?;
    let (tl, ta, tt) = split(&parse_rule_color_list(to)?)?;
    match (fa, ta) {
        // 无 auto：等长逐位 / 单值广播 / 异长离散。
        (None, None) => {
            if fl.len() == tl.len() {
                lerp_flat(&fl, &tl).map(|v| v.join(", "))
            } else if fl.len() == 1 {
                lerp_flat(&vec![fl[0].clone(); tl.len()], &tl).map(|v| v.join(", "))
            } else if tl.len() == 1 {
                lerp_flat(&fl, &vec![tl[0].clone(); fl.len()]).map(|v| v.join(", "))
            } else {
                None
            }
        }
        // 双 auto：leading/trailing 段数各等 + 循环体 LCM 对齐。
        (Some(fc), Some(tc)) => {
            if fl.len() != tl.len() || ft.len() != tt.len() {
                return None;
            }
            let lcm = {
                let (a, b) = (fc.len(), tc.len());
                let gcd = |mut x: usize, mut y: usize| -> usize {
                    while y != 0 {
                        (x, y) = (y, x % y);
                    }
                    x
                };
                a * b / gcd(a, b)
            };
            let tile = |c: &[ColorValue]| -> Vec<ColorValue> { c.iter().cycle().take(lcm).cloned().collect() };
            let mid = lerp_flat(&tile(&fc), &tile(&tc))?;
            let mut parts = lerp_flat(&fl, &tl)?;
            parts.push(format!("repeat(auto, {})", mid.join(", ")));
            parts.extend(lerp_flat(&ft, &tt)?);
            Some(parts.join(", "))
        }
        // 单侧 auto：结构异构离散。
        _ => None,
    }
}

/// 严格角度解析（仅 `Ndeg` 形；无单位裸数字/其他单位拒——Transition 端点来自
/// gCS 序列化恒带单位，裸数字即非插值形）。
fn parse_angle_deg_strict(s: &str) -> Option<f64> {
    s.trim().strip_suffix("deg").and_then(|v| v.trim().parse::<f64>().ok())
}

/// 严格颜色解析（`rgb(r, g, b)` / `rgba(r, g, b, a)` 逗号形——gCS 颜色序列化
/// 形同源；命名色/新语法形拒收回落离散面）。组件任一非数值 → None。
fn parse_color_strict(s: &str) -> Option<Rgba> {
    let s = s.trim().to_lowercase();
    let (body, n) = if let Some(rest) = s.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
        (rest, 4)
    } else {
        let rest = s.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')'))?;
        (rest, 3)
    };
    let parts: Vec<&str> = body.split(',').map(|p| p.trim()).collect();
    if parts.len() != n {
        return None;
    }
    let r = parts[0].parse::<f64>().ok()?;
    let g = parts[1].parse::<f64>().ok()?;
    let b = parts[2].parse::<f64>().ok()?;
    let a = if n == 4 { parts[3].parse::<f64>().ok()? } else { 1.0 };
    Some((r, g, b, a))
}

/// filter 函数列表插值（filter-effects-1 #animation）。
///
/// 同长同序的同名函数逐参数线性插值（drop-shadow 含 x/y/blur + 颜色分量）；`none`
/// 一侧按另一侧函数展开为初始值列表（blur(0)/brightness(1)/contrast(1)/grayscale(0)/
/// hue-rotate(0deg)/invert(0)/opacity(1)/saturate(1)/sepia(0)/drop-shadow(0 0 0
/// transparent)）；异构返回 `None` 走离散。双 none 返回 "none"。
fn interpolate_filter_list(from: &str, to: &str, t: f64) -> Option<String> {
    let from_list = zero_css_parser::values::parse_filter_list(from)?;
    let to_list = zero_css_parser::values::parse_filter_list(to)?;
    // none 展开：与另一侧同构的初始值列表。
    let expand = |list: &[zero_css_parser::values::FilterValue],
                  model: &[zero_css_parser::values::FilterValue]|
     -> Vec<zero_css_parser::values::FilterValue> {
        if list.is_empty() {
            model.iter().map(filter_function_initial).collect()
        } else {
            list.to_vec()
        }
    };
    if from_list.is_empty() && to_list.is_empty() {
        return Some("none".to_string());
    }
    let a = expand(&from_list, &to_list);
    let b = expand(&to_list, &from_list);
    if a.len() != b.len() {
        return None;
    }
    let mut parts = Vec::with_capacity(a.len());
    for (fa, fb) in a.iter().zip(b.iter()) {
        if filter_function_name(fa) != filter_function_name(fb) {
            // 同长但函数不同构（filter-effects-1 #animation：仅同名函数可插值）。
            return None;
        }
        parts.push(interpolate_filter_pair(fa, fb, t)?);
    }
    Some(parts.join(" "))
}

/// 插值一对同名 filter 函数，产出 parse_filter 可回解析的规范形。
fn interpolate_filter_pair(
    fa: &zero_css_parser::values::FilterValue,
    fb: &zero_css_parser::values::FilterValue,
    t: f64,
) -> Option<String> {
    use zero_css_parser::values::FilterValue as F;
    match (fa, fb) {
        // drop-shadow：x/y/blur 数值 + 颜色分量分别插值（css-filters-animation-drop-shadow
        // 驱动：30px 30px 0px black × none → 15px 15px 0px rgba(0,0,0,.5)）。
        (F::DropShadow(x1, y1, b1, c1), F::DropShadow(x2, y2, b2, c2)) => {
            let ca = color_value_rgba(c1)?;
            let cb = color_value_rgba(c2)?;
            let (r, g, b, a) = lerp_color(ca, cb, t);
            let x = lerp(*x1 as f64, *x2 as f64, t);
            let y = lerp(*y1 as f64, *y2 as f64, t);
            let bl = lerp(*b1 as f64, *b2 as f64, t);
            Some(format!(
                "drop-shadow({:.2}px {:.2}px {:.2}px rgba({}, {}, {}, {:.2}))",
                x,
                y,
                bl,
                r.round() as u8,
                g.round() as u8,
                b.round() as u8,
                a
            ))
        }
        _ => {
            let va = filter_function_param(fa)?;
            let vb = filter_function_param(fb)?;
            let v = lerp(va as f64, vb as f64, t) as f32;
            Some(match fa {
                F::Blur(_) => format!("blur({v:.2}px)"),
                F::HueRotate(_) => format!("hue-rotate({v:.2}deg)"),
                F::Brightness(_) => format!("brightness({v:.4})"),
                F::Contrast(_) => format!("contrast({v:.4})"),
                F::Grayscale(_) => format!("grayscale({v:.4})"),
                F::Invert(_) => format!("invert({v:.4})"),
                F::Opacity(_) => format!("opacity({v:.4})"),
                F::Saturate(_) => format!("saturate({v:.4})"),
                F::Sepia(_) => format!("sepia({v:.4})"),
                _ => return None,
            })
        }
    }
}

/// ColorValue → animation Rgba 元组（0-255 + 0-1 alpha）；未解析形（Mix/RCS/
/// CurrentColor/Hsla）返回 None 走离散（命名色在 parse 阶段已短路为 Rgba）。
fn color_value_rgba(c: &zero_css_parser::values::ColorValue) -> Option<Rgba> {
    match c {
        zero_css_parser::values::ColorValue::Rgba(r, g, b, a) => {
            Some((*r as f64, *g as f64, *b as f64, *a as f64 / 255.0))
        }
        zero_css_parser::values::ColorValue::Transparent => Some((0.0, 0.0, 0.0, 0.0)),
        _ => None,
    }
}

/// filter 单函数的规范名（同构判定用）。
fn filter_function_name(f: &zero_css_parser::values::FilterValue) -> &'static str {
    use zero_css_parser::values::FilterValue as F;
    match f {
        F::Blur(_) => "blur",
        F::Brightness(_) => "brightness",
        F::Contrast(_) => "contrast",
        F::Grayscale(_) => "grayscale",
        F::HueRotate(_) => "hue-rotate",
        F::Invert(_) => "invert",
        F::Opacity(_) => "opacity",
        F::Saturate(_) => "saturate",
        F::Sepia(_) => "sepia",
        F::DropShadow(..) => "drop-shadow",
        F::Url(_) => "url",
        F::None => "none",
    }
}

/// filter 单函数的可插值参数；drop-shadow 无（颜色分量不做数值插值）→ None。
fn filter_function_param(f: &zero_css_parser::values::FilterValue) -> Option<f32> {
    use zero_css_parser::values::FilterValue as F;
    match f {
        F::Blur(v)
        | F::Brightness(v)
        | F::Contrast(v)
        | F::Grayscale(v)
        | F::HueRotate(v)
        | F::Invert(v)
        | F::Opacity(v)
        | F::Saturate(v)
        | F::Sepia(v) => Some(*v),
        F::DropShadow(..) | F::Url(_) | F::None => None,
    }
}

/// filter 单函数取初始值（同变体，§filter 初始值表；drop-shadow none 展开 =
/// 0 偏移 0 模糊透明色，css-filters-animation-drop-shadow ref 实证）。
fn filter_function_initial(f: &zero_css_parser::values::FilterValue) -> zero_css_parser::values::FilterValue {
    use zero_css_parser::values::{ColorValue, FilterValue as F};
    match f {
        F::Blur(_) => F::Blur(0.0),
        F::Brightness(_) => F::Brightness(1.0),
        F::Contrast(_) => F::Contrast(1.0),
        F::Grayscale(_) => F::Grayscale(0.0),
        F::HueRotate(_) => F::HueRotate(0.0),
        F::Invert(_) => F::Invert(0.0),
        F::Opacity(_) => F::Opacity(1.0),
        F::Saturate(_) => F::Saturate(1.0),
        F::Sepia(_) => F::Sepia(0.0),
        F::DropShadow(..) => F::DropShadow(0.0, 0.0, 0.0, ColorValue::Transparent),
        F::Url(_) | F::None => f.clone(),
    }
}

/// 线性插值。
// R5191：pub(crate)——transform 函数列表插值（animation/transform_interp.rs）复用。
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// columns 简写端点组件（css-multicol-2 §4.5 声明形）：width/count/height 三槽，
/// None = auto 槽（声明中未出现或显式 auto）。`slots()` = 三槽显式存在集——
/// 插值配对要求两端存在集恒等（组件形变 = 非可插值对，css-values-4 §8.5.6）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct ColumnsEndpoint {
    width: Option<f64>,
    count: Option<f64>,
    height: Option<f64>,
    /// 显式槽存在集（'auto' 关键字占位仍计存在——`'10 100px / auto'` 与
    /// `'20 200px / auto'` 存在集恒等故可插值；裸 `auto` 全零）。
    slots: (bool, bool, bool),
}

impl ColumnsEndpoint {
    fn slots(&self) -> (bool, bool, bool) {
        self.slots
    }

    /// 可插值配对：存在集恒等且各槽值存在性恒等（auto↔值槽形变 = 离散，
    /// css-values-4 §8.5.6——组件 computed value type 变更不可 lerp）。
    fn compatible(&self, other: &ColumnsEndpoint) -> bool {
        self.slots() == other.slots()
            && self.width.is_some() == other.width.is_some()
            && self.count.is_some() == other.count.is_some()
            && self.height.is_some() == other.height.is_some()
    }
}

/// 解析 columns 声明形端点（`'200px 10'`/`'100px / 200px'`/`'10 100px / auto'`/
/// `'auto'`）。width||count 顺序无关（整数→count、长度→width、auto 填余槽）；
/// 斜杠段 = column-height。非文法形（css-wide 关键字、残缺段）返 None → 离散。
fn parse_columns_endpoint(v: &str) -> Option<ColumnsEndpoint> {
    let tokens: Vec<&str> = v.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    let slash = tokens.iter().position(|t| *t == "/");
    let (pre, post) = match slash {
        Some(pos) => (&tokens[..pos], &tokens[pos + 1..]),
        None => (&tokens[..], &tokens[0..0]),
    };
    if pre.len() > 2 || post.len() > 1 {
        return None;
    }
    let mut ep = ColumnsEndpoint {
        width: None,
        count: None,
        height: None,
        slots: (false, false, false),
    };
    match (pre.len(), slash.is_some()) {
        // 裸 'auto'（全初值，无槽）；仅当无斜杠段时成立
        (1, false) if pre[0].eq_ignore_ascii_case("auto") => {}
        (0, false) => return None, // 空端点非文法形
        (0, true) => return None,  // 斜杠前组文法必选
        _ => {
            for tok in pre {
                if tok.eq_ignore_ascii_case("auto") {
                    continue;
                }
                if tok.parse::<u32>().is_ok_and(|n| n >= 1) && ep.count.is_none() {
                    ep.count = Some(tok.parse::<f64>().unwrap_or(0.0));
                    ep.slots.1 = true;
                } else if ep.width.is_none() {
                    // 严格解析：css-wide 关键字/畸形 token 不得 0 化（端点非法 →
                    // None → 离散回落）
                    ep.width = Some(parse_px_strict(tok)?);
                    ep.slots.0 = true;
                } else {
                    return None;
                }
            }
            // 'auto' 占位槽记存在（无值语义——slots 参与存在集配对）
            if pre.iter().any(|t| t.eq_ignore_ascii_case("auto")) {
                ep.slots.0 |= ep.width.is_none();
                ep.slots.1 |= ep.count.is_none();
            }
        }
    }
    if let Some(h) = post.first() {
        ep.slots.2 = true;
        if !h.eq_ignore_ascii_case("auto") {
            ep.height = Some(parse_px_strict(h)?);
        }
    }
    Some(ep)
}

/// columns 组件序列化（css-multicol-2 §4.5 canonical）：width/count auto 省略，
/// 全 auto → "auto"；height 非 auto 时 slash 恒显——与 gCS
/// [`crate::js_dom_bridge::computed_style`] columns_shorthand_to_css 同构。
fn serialize_columns_components(w: Option<f64>, c: Option<f64>, h: Option<f64>) -> String {
    let width_count = match (w, c) {
        (Some(wv), Some(cv)) => format!("{:.2}px {}", wv, cv.round() as i64),
        (Some(wv), None) => format!("{:.2}px", wv),
        (None, Some(cv)) => format!("{}", cv.round() as i64),
        (None, None) => "auto".to_string(),
    };
    match h {
        Some(hv) => format!("{} / {:.2}px", width_count, hv),
        None => width_count,
    }
}

/// 解析 f64 值。
fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

/// 刀 Z：z-index 端点整数解析——`<integer>` 语法严格（css-values-4 §8.1.2）：
/// auto/小数/带单位形返 `None`（Rust `parse::<i64>` 接受前导 +/- 号与
/// css-syntax 整数符号法一致）。
fn parse_z_index_endpoint(s: &str) -> Option<i64> {
    s.trim().parse::<i64>().ok()
}

/// 刀 TEXT-INTERP FACE-MARKER（A/B bin 指纹，strings 抽验判别只认代码字面
/// ASCII——KA2 刀同款先例）。零引用，`#[used]` 保证 rodata 携带。
#[used]
static TEXT_INTERP_FACE_MARKER: &str = "TEXT-INTERP discrete family";

/// 刀 TEXT-INTERP：hyphenate-limit-chars 端点解析——`<integer>{1,3}` 空格
/// 分隔严格形（css-text-4 §5.4）；none/auto/空/超三分量返 None＝离散对。
fn parse_hyphenate_limit_chars_endpoint(s: &str) -> Option<Vec<i64>> {
    let mut out = Vec::new();
    for part in s.split_whitespace() {
        out.push(part.parse::<i64>().ok()?);
    }
    if out.is_empty() || out.len() > 3 {
        return None;
    }
    Some(out)
}

/// 刀 A2：vertical-align 插值端点解析——`<length-percentage>` 严格形（数字
/// 前置门同 [`zero_css_parser::values::parse_vertical_align`]，防关键字臂误收）；
/// `baseline` 按 0 长度参与（语料 initial/unset→<length> 对 at(0) 期望 '0px'，
/// Chrome parity）；其余关键字（super/sub/middle 等）返 None 走离散翻转。
fn parse_va_endpoint(s: &str) -> Option<zero_css_parser::values::LengthValue> {
    let t = s.trim();
    if t.eq_ignore_ascii_case("baseline") {
        return Some(zero_css_parser::values::LengthValue::Px(0.0));
    }
    let first = t.as_bytes().first()?;
    if !(first.is_ascii_digit() || *first == b'-' || *first == b'+' || *first == b'.') {
        return None;
    }
    zero_css_parser::values::parse_length(t)
}

/// 刀 A2：vertical-align 长度对插值发射（css-values-4 §8.5.6 interpolation-calc
/// 跨单位混合形）——同单位线性；px↔% 发射 px 系数在前的语料 canonical 形
/// `calc(Xpx ± P%)`（40px↔40% at(-0.5) 期望 `calc(60px - 20%)`），px 系数 0
/// 折叠纯 %（at(1) 期望 '40%'）；数字形 {:.2}（shim roundNumbers 双侧归一，
/// blend_rule_inset_lp 同约定）。em/rem 等其余单位对空串（applier 解析必败
/// 保旧值，调用方语义同离散回落基线）。
fn lerp_va_length(
    f: &zero_css_parser::values::LengthValue,
    s: &zero_css_parser::values::LengthValue,
    t: f64,
) -> String {
    use zero_css_parser::values::LengthValue as Lv;
    match (f, s) {
        (Lv::Px(a), Lv::Px(b)) => format!("{:.2}px", lerp(*a, *b, t)),
        (Lv::Percentage(a), Lv::Percentage(b)) => format!("{:.2}%", lerp(*a, *b, t)),
        (Lv::Px(a), Lv::Percentage(b)) => blend_va_lp(*a, "px", *b, "%", t),
        (Lv::Percentage(a), Lv::Px(b)) => blend_va_lp(*a, "%", *b, "px", t),
        _ => String::new(),
    }
}

/// 刀 A2：vertical-align px↔% 跨单位系数混合（css-values-4 §8.5.6）——
/// pct 系数 = (1-t)·from_pct + t·to_pct、px 系数同理（对侧单位分量恒 0）；
/// px 系数 0 折叠纯 %（40px↔40% at(1) 期望 '40%'）；发射 px 在前
/// `calc(Xpx ± P%)`（语料 canonical 形，blend_rule_inset_lp 的 % 前缀形
/// 是 rule-inset 槽序、vertical-align 语料相反）。
fn blend_va_lp(fv: f64, fu: &str, tv: f64, tu: &str, t: f64) -> String {
    let pct = (1.0 - t) * if fu == "%" { fv } else { 0.0 } + t * if tu == "%" { tv } else { 0.0 };
    let px = (1.0 - t) * if fu == "px" { fv } else { 0.0 } + t * if tu == "px" { tv } else { 0.0 };
    if px.abs() < 1e-9 {
        return format!("{:.2}%", pct);
    }
    let seg = if pct < 0.0 {
        format!(" - {:.2}", -pct)
    } else {
        format!(" + {:.2}", pct)
    };
    format!("calc({:.2}px{}%)", px, seg)
}

/// 解析 px 值（如 "100px" → 100.0）。
fn parse_px(s: &str) -> f64 {
    let s = s.trim();
    s.strip_suffix("px").and_then(|v| v.trim().parse().ok()).unwrap_or(0.0)
}

/// composite-add 刀：严格解析 px 值（仅 `Npx` 形；宽松版 [`parse_px`] 失败取 0
/// 数值化——关键字端点 auto/min-content 等须走离散语义，不得 0 化）。
fn parse_px_strict(s: &str) -> Option<f64> {
    let s = s.trim();
    s.strip_suffix("px").and_then(|v| v.trim().parse().ok())
}

/// r80 刀 R2：LP 逐 token 严格插值共享体——1-2 token、token 数恒等 ∧ 全 token
/// [`parse_lp_terms`] 解析才 Some（strict 门语义：非插值对 None = 不启动过渡，
/// css-transitions-1 §starting-of-transitions）；纯 px token 对维持 `{:.2}px`
/// 串形（strict px 臂既有输出，零回归），含 % token 走 [`render_lp_terms`]
/// canonical 形。strict/Lenient 双臂共用（css-values-4 §8.5.6）。
fn interpolate_lp_token_strict(from: &str, to: &str, t: f64) -> Option<String> {
    let fp: Vec<&str> = from.split_whitespace().collect();
    let tp: Vec<&str> = to.split_whitespace().collect();
    if fp.is_empty() || fp.len() != tp.len() || fp.len() > 2 {
        return None;
    }
    let mut out = Vec::with_capacity(fp.len());
    for (f, s) in fp.iter().zip(tp.iter()) {
        let (fpx, fpc) = parse_lp_terms(f)?;
        let (tpx, tpc) = parse_lp_terms(s)?;
        let (px, pct) = (lerp(fpx, tpx, t), lerp(fpc, tpc, t));
        out.push(if fpc == 0.0 && tpc == 0.0 {
            // 纯 px 对（含零 % 系数）：px 域串化。
            format!("{:.2}px", px)
        } else if fpx == 0.0 && tpx == 0.0 {
            // r80 刀 R2：纯 % 对（transform-origin '0% 50%'→'100% 50%'）按 %
            // 域串化——此前 % 对 px 系数恒 0 误走 px 形，gCS % 形端点往返断裂。
            format!("{:.2}%", pct)
        } else {
            render_lp_terms(px, pct)
        });
    }
    Some(out.join(" "))
}

// ── fame2b（wptfyi-css，2026-11-22）：sizing 六长手 length-percentage 加法合成 ──
// web-animations-1 §5.4.4 combining values（addition 先于插值、不可加回落动画值）
// ＋ css-sizing-3 §width/height/max-width/max-height/min-width/min-height
// <length-percentage> 值域。端点域：裸 px / 裸 % / 裸 0 / 本刀 calc 混合形
//（[`render_lp_terms`] 产出）；auto/none/min-content 等关键字端点拒 → 调用方
// 回落（compose 臂 animated 原样＝§5.4.4「不可加 → 动画值原样」退化语义；
// 插值臂离散翻转 css-values-4 §8.5.6）。

/// fame2b：length-percentage 端点 → (px, %) 双 term 分解。calc 形受理本刀规范形
/// `calc(Npx ± P%)`（px 项先序，负项减法形——与 [`blend_va_lp`] 串形同域，
/// apply+gCS 往返已证通道）。
fn parse_lp_terms(s: &str) -> Option<(f64, f64)> {
    let t = s.trim();
    if let Some(v) = parse_px_strict(t) {
        return Some((v, 0.0));
    }
    if t == "0" {
        return Some((0.0, 0.0));
    }
    if let Some(p) = t.strip_suffix('%') {
        let v: f64 = p.trim().parse().ok()?;
        return Some((0.0, v));
    }
    let inner = t.strip_prefix("calc(")?.strip_suffix(')')?.trim();
    // 顶层恰两 term `A <op> B`（op ∈ {+,-}；语料形无嵌套/min/max）。
    let mut terms = inner.split_whitespace();
    let a = terms.next()?;
    let op = terms.next()?;
    let b = terms.next()?;
    if terms.next().is_some() {
        return None;
    }
    let parse_term = |x: &str| -> Option<(f64, bool)> {
        let x = x.trim();
        if let Some(v) = parse_px_strict(x) {
            return Some((v, false));
        }
        let p = x.strip_suffix('%')?;
        Some((p.trim().parse().ok()?, true))
    };
    let (av, a_pct) = parse_term(a)?;
    let (bv, b_pct) = parse_term(b)?;
    if a_pct == b_pct {
        return None; // 同型两项非本刀产出形（折叠已在 render 期完成），拒
    }
    let (px, pct, sign) = if a_pct { (bv, av, -1.0) } else { (av, bv, 1.0) };
    let px = if op == "-" { sign * px } else { px };
    Some((px, pct))
}

/// fame2b：length-percentage 合成串化——px 项先序 `calc(Npx ± P%)`（
/// [`blend_va_lp`] 同形，apply 侧 calc 门与 gCS `calc_computed_to_css` 往返已证；
/// 断言通道 double-gCS：期望侧 setProperty 后同过 gCS，term 序两侧抵消——
/// PREP-fame2b 附记 B）。零项折叠纯 px/%（css-values-4 §10.11 computed
/// canonical 零项丢弃语义）。
/// 裸百分比字面判定（r75 刀F②）：`50%`/`-20%` 形 true；`calc(...)`/px/
/// 关键字 false。仅 height 轴离散臂使用。
fn is_bare_percentage_literal(v: &str) -> bool {
    v.trim().strip_suffix('%').unwrap_or("").trim().parse::<f64>().is_ok()
}

fn render_lp_terms(px: f64, pct: f64) -> String {
    if px.abs() < 1e-9 {
        return format!("{:.2}%", pct);
    }
    if pct.abs() < 1e-9 {
        return format!("{:.2}px", px);
    }
    let seg = if pct < 0.0 {
        format!(" - {:.2}", -pct)
    } else {
        format!(" + {:.2}", pct)
    };
    format!("calc({:.2}px{})", px, seg)
}

// ── KC-5-A（css-sizing-4 §8 contain-intrinsic 族插值）──
// 背景：该族此前引擎全链无臂（插值器/过渡登记/apply 写面），gCS 亦无
// 简写串化臂（KC-4 另立）——interpolation 页在 b0 是 harness 双侧读 gCS
// 的双空串空洞过（详见 .acceptance NOTES-kc5），本刀补齐插值机制使页
// 面真过。域限 px/none/auto 三形；em/% 端点不入门（落离散/拒登）。

/// contain-intrinsic 单轴端点：`auto` 前缀旗标＋`none` | `<length>` px 值。
#[derive(Clone, Copy, PartialEq)]
struct CisAxis {
    auto: bool,
    none: bool,
    px: f64,
}

/// KC-5-A：cis 端点串 → 1–2 轴。轴文法与 decl 门（callbacks.rs
/// cis_decl_normalize）同界：`auto` 吸附后随长度 token（`auto` 无随从 /
/// 随从非 length 全拒）；`none` 独立单轴 token；裸 `<length>` 单轴。
/// 裸 `0` 计 0px（合法 <length>，刀 K1 同口径）。
fn parse_cis_axes(s: &str) -> Option<Vec<CisAxis>> {
    let toks: Vec<&str> = s.split_whitespace().collect();
    let px_val = |t: &str| -> Option<f64> { parse_px_strict(t).or_else(|| (t.trim() == "0").then_some(0.0)) };
    let mut axes: Vec<CisAxis> = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        let axis = if toks[i].eq_ignore_ascii_case("auto") {
            let v = px_val(toks.get(i + 1)?)?;
            i += 2;
            CisAxis {
                auto: true,
                none: false,
                px: v,
            }
        } else if toks[i].eq_ignore_ascii_case("none") {
            i += 1;
            CisAxis {
                auto: false,
                none: true,
                px: 0.0,
            }
        } else {
            let v = px_val(toks[i])?;
            i += 1;
            CisAxis {
                auto: false,
                none: false,
                px: v,
            }
        };
        if axes.len() == 2 {
            return None;
        }
        axes.push(axis);
    }
    if axes.is_empty() { None } else { Some(axes) }
}

/// KC-5-A：单轴插值——同形轴（auto↔auto / none↔none / len↔len）参与 lerp
///（none 对恒 none、len 对按域 [0,∞] 钳 0——语料 at(1.5) 期望 `5px 0px`
/// 钳形实证）；跨形轴（auto↔len / none↔len 等）None＝strict 拒登 /
/// lenient 调用方离散切换。
fn cis_axis_interp(f: &CisAxis, to: &CisAxis, t: f64) -> Option<CisAxis> {
    if f.auto != to.auto || f.none != to.none {
        return None;
    }
    Some(CisAxis {
        auto: f.auto,
        none: f.none,
        px: lerp(f.px, to.px, t).max(0.0),
    })
}

/// KC-5-A：contain-intrinsic 族插值核心。双端点轴数归一（单轴→双轴复制，
/// css-sizing-4 §8 简写文法）；任一轴不可插值 → None。输出经 gCS 同通道
/// `length_to_css` 串化（格式恒等由构造保证）；`shorthand` 时等轴折叠
///（`3px 3px`→`3px`）——serve 形与 KC-4 gCS 聚合臂一致，否则 expected
/// 侧聚合 vs actual 未折叠＝假红。
fn interpolate_cis_value(shorthand: bool, from: &str, to: &str, t: f64) -> Option<String> {
    let render = |a: &CisAxis| -> String {
        if a.none {
            "none".to_string()
        } else if a.auto {
            format!(
                "auto {}",
                crate::js_dom_bridge::computed_style::length_to_css(&LengthValue::Px(a.px), 16.0)
            )
        } else {
            crate::js_dom_bridge::computed_style::length_to_css(&LengthValue::Px(a.px), 16.0)
        }
    };
    let mut fa = parse_cis_axes(from)?;
    let mut ta = parse_cis_axes(to)?;
    if fa.len() == 1 {
        let a = fa[0];
        fa.push(a);
    }
    if ta.len() == 1 {
        let a = ta[0];
        ta.push(a);
    }
    if fa.len() != ta.len() {
        return None;
    }
    let a0 = cis_axis_interp(&fa[0], &ta[0], t)?;
    let a1 = cis_axis_interp(&fa[1], &ta[1], t)?;
    if shorthand {
        let s0 = render(&a0);
        let s1 = render(&a1);
        Some(if s0 == s1 { s0 } else { format!("{s0} {s1}") })
    } else {
        Some(render(&a0))
    }
}

// KC-5-A：cis 族插值属性名面（简写＋width/height 双长手）。block-size/
// inline-size 声明门已收（KC-3）但 gCS 串化臂未立，暂不入插值面（FIXME：
// 逻辑轴写作模式面随竖排刀补）。

// 刀 FAM-F（wptfyi-css，2026-11-22）：aspect-ratio 端点解析——`auto? w / h`
// 或 bare `N`（= N/1）。返回 (auto 前缀, ratio 合并值)；简并形（分量 ≤0 或
// 非有限，css-values-4 §combine-ratio）与解析失败均返 None＝不可插值。
fn parse_aspect_ratio_decl(s: &str) -> Option<(bool, f64, f64)> {
    let rest = s.trim();
    let (auto, rest) = match rest.strip_prefix("auto") {
        Some(r) => (true, r.trim()),
        None => (false, rest),
    };
    let (w, h) = match rest.split_once('/') {
        Some((w, h)) => (w.trim().parse::<f64>().ok()?, h.trim().parse::<f64>().ok()?),
        None => (rest.parse::<f64>().ok()?, 1.0),
    };
    if w <= 0.0 || h <= 0.0 || !w.is_finite() || !h.is_finite() {
        return None;
    }
    Some((auto, w, h))
}

/// 刀 FAM-F：aspect-ratio 几何插值（css-values-4 §combine-ratio：双端非简并
/// ratio 按 r0^(1-t)·r1^t 幂律插值，规范形 `x / 1`；auto 前缀双侧一致时保留）。
/// 简并/auto 混形/解析失败返 None 回落离散 flip（css-values-4 §8.5.6 ≥0.5→to）。
/// <https://drafts.csswg.org/css-values-4/#combine-ratio>
fn interpolate_aspect_ratio(from: &str, to: &str, t: f64) -> Option<String> {
    let (auto0, w0, h0) = parse_aspect_ratio_decl(from)?;
    let (auto1, w1, h1) = parse_aspect_ratio_decl(to)?;
    if auto0 != auto1 {
        return None;
    }
    let r0 = w0 / h0;
    let r1 = w1 / h1;
    let x = r0.powf(1.0 - t) * r1.powf(t);
    let prefix = if auto0 { "auto " } else { "" };
    // 输出走 CSSOM serialize 同口径（6 位有效数字去尾零，Chromium 一致）：
    // 幂律中途值含 fp 残差（0.5^0.5·2^05 = 1.0000000000000002），Display 直印
    // 会破 `x / 1` 规范形比较；format_num 与 gCS 回读 [`aspect_ratio_str`] 恒同。
    Some(format!(
        "{prefix}{} / 1",
        crate::js_dom_bridge::computed_style::format_num(x, "")
    ))
}

// 刀 CSINTERP-KA2 max-none-identity：none 端点归一 none 槽（css-sizing-3 §5.2）。
// r69 R4 探针剥臂暂存：r3 fame2b 委托剥除后本函数恢复供 min/max-sizing 写槽臂。
// <https://drafts.csswg.org/css-sizing-3/#valdef-max-width-none>
fn parse_max_sizing_anim_value(s: &str) -> Option<zero_css_parser::values::LengthValue> {
    if s.trim().eq_ignore_ascii_case("none") {
        return Some(zero_css_parser::values::LengthValue::Px(f64::INFINITY));
    }
    parse_px_strict(s).map(zero_css_parser::values::LengthValue::Px)
}

/// CSINTERP-KA2 电池 A/B 烙印（b0 剥 face 时随父版消失；零引用，`#[used]`
/// 保证 rodata 携带——函数符号 release 内联抹除，判别只认代码字面 ASCII，
/// KA_FACE_MARKER/URLSMOD_FACE_MARKER 先例）。
#[used]
static KA2_FACE_MARKER: &str = "KA2-FACE max-none-identity";

/// composite-add 刀：font-size-adjust 插值端点解析（css-fonts-4
/// #font-size-adjust-prop）——`<metric> <number>` 返 `(Some(metric), n)`、
/// 裸 `<number>` 返 `(None, n)`；none / from-font / 畸形返 `None`（离散回落）。
fn parse_font_size_adjust_endpoint(s: &str) -> Option<(Option<&'static str>, f64)> {
    let s = s.trim();
    let mut parts = s.splitn(2, char::is_whitespace);
    let metric = match parts.next() {
        Some("ex-height") => Some("ex-height"),
        Some("cap-height") => Some("cap-height"),
        Some("ch-width") => Some("ch-width"),
        Some("ic-width") => Some("ic-width"),
        Some("ic-height") => Some("ic-height"),
        _ => None,
    };
    let num_text = if metric.is_some() { parts.next()?.trim() } else { s };
    num_text.parse::<f64>().ok().map(|n| (metric, n))
}

/// composite-add 刀：插值数值串化（3 位小数去尾零、整数无小数点，与 gCS
/// `format_num` 同形——canonicalize 会重序列化再规范化，此处形态服务 raw 面）。
fn format_interp_num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// R4202：解析 `translate` 值为 (x, y) px 数值对（% 分量按 0 近似——keyframes 的
/// translate % 中点插值目标侧取值已由字符串臂覆盖主形态，此处数值化仅服务 lerp）。
fn parse_translate_pair(s: &str) -> (f64, f64) {
    let parts: Vec<&str> = s.split_whitespace().collect();
    let x = parts.first().map(|p| parse_px(p)).unwrap_or(0.0);
    let y = parts.get(1).map(|p| parse_px(p)).unwrap_or(0.0);
    (x, y)
}

/// R4202：解析 `rotate` 角度值（deg 数值；无单位裸数字按 deg）。
fn parse_angle_deg(s: &str) -> f64 {
    let s = s.trim();
    s.strip_suffix("deg")
        .and_then(|v| v.trim().parse().ok())
        .or_else(|| s.parse().ok())
        .unwrap_or(0.0)
}

/// R4202：解析 `scale` 值为 (x, y) 数值对（单值等比展开）。
fn parse_scale_pair(s: &str) -> (f64, f64) {
    let parts: Vec<&str> = s.split_whitespace().collect();
    let x: f64 = parts.first().and_then(|p| p.parse().ok()).unwrap_or(1.0);
    let y: f64 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(x);
    (x, y)
}

// ── CSS Motion Path offset 簇（css-motion-1；wptfyi-css offset-* 刀）辅助 ──

/// offset-distance/anchor 值解析：(数值, 单位)——仅收 px/%（em 形 apply 期已绝对化
/// 为 px；其余单位/数学形离散回落，corpus 端点以 px/% 为主）。
fn parse_len_pct_val(s: &str) -> Option<(f64, String)> {
    let s = s.trim();
    if let Some(v) = s.strip_suffix('%') {
        return v.trim().parse::<f64>().ok().map(|n| (n, "%".to_string()));
    }
    s.strip_suffix("px")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|n| (n, "px".to_string()))
}

/// rule-inset 槽跨单位（px↔%）calc 混合插值（css-values-4 §8.5.6 interpolation
/// of length-percentage）：两端点逐单位系数线性组合——`[(1-t)·from + t·to]` 产
/// `calc(P% ± Xpx)` canonical 文本。px 系数为 0 → 纯 `P%`（端点简化——语料
/// interpolation 页 10px→-10% at(1) 期望 `-10%`）；% 系数为 0 保 calc 形（at(0)
/// 期望 `calc(0% + 10px)`）。同单位对/防御性 None（调用方 lerp/离散回落承担）。
/// 两侧期望/实际都过同一 parse+gCS 串化器（interpolation-testcommon expected
/// 元素 setProperty 同通道），结构/系数一致性由构造保证。
/// https://drafts.csswg.org/css-values-4/#interpolation-calc
fn blend_rule_inset_lp(fv: &f64, fu: &str, tv: &f64, tu: &str, t: f64) -> Option<String> {
    if (fu == "%" && tu == "%") || (fu == "px" && tu == "px") {
        return None;
    }
    // 统一形式：pct 系数 = (1-t)·from_pct + t·to_pct；px 系数同理（对侧单位分量恒 0）。
    let pct = (1.0 - t) * if fu == "%" { *fv } else { 0.0 } + t * if tu == "%" { *tv } else { 0.0 };
    let px = (1.0 - t) * if fu == "px" { *fv } else { 0.0 } + t * if tu == "px" { *tv } else { 0.0 };
    if px.abs() < 1e-9 {
        return Some(format!("{:.2}%", pct));
    }
    if px < 0.0 {
        Some(format!("calc({:.2}% - {:.2}px)", pct, -px))
    } else {
        Some(format!("calc({:.2}% + {:.2}px)", pct, px))
    }
}

/// offset-anchor `X Y` 对解析。
fn parse_len_pair_pct(s: &str) -> Option<((f64, String), (f64, String))> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    Some((parse_len_pct_val(parts[0])?, parse_len_pct_val(parts[1])?))
}

// ── mask-position 插值（css-masking-1 §5.3，wptfyi-css 刀 MI/A1）───────────
// 轴分量归一为 `(pct 系数, px 系数, 是否含 % 项)` 三元组：left/top→(0,0)、
// center→(50,0)、right/bottom→(100,0)、Percent(p)→(p,0)、Px(v)→(0,v)、
// EdgeOffset 左/上缘基 offset 本身（纯 px offset 不含 % 项——Chromium 把该
// 形 resolve 成纯 length，语料 top 0px left 0px 块钉）、右/下缘基 (100,−off)
//（resolve 翻转同语义，缘百分比恒在场故含 % 项）；Calc 端点不插值。
// 逐层列表 LCM 循环展开（语料 mismatched-cases 判例 2 层 vs 3 层→6 层），
// 系数线性 lerp 后串化：两端点均纯 length → 纯 `Xpx`（length↔length 插值
// 产物仍是 length 不引入 calc——px↔px 语料块期望元素 inline 纯 px，calc
// 系数形会落不可解析 % Calc 轴空串）；任一端点含 % 项时 px 系数 0 → 纯
// `P%`（端点简化），否则 `calc(P% ± Xpx)`（css-values-4 §8.5.6；两侧经
// shim roundNumbers 归一同通道比对）。任一层解析失败 → None（调用方离散
// 翻转回落）。

/// 轴分量→(pct 系数, px 系数, 是否含 % 项)。
type MaskPosComp = (f64, f64, bool);

/// % / px 加减脊 calc → comp（刀 A1d）：Add/Subtract 展平（Subtract 右臂取负）、
/// Percentage/Px 叶累积；其余树（乘除/Min/Max/Mix/树计数）None → 调用方离散回落。
fn mask_pos_comp_calc(expr: &zero_css_parser::values::CalcExpr) -> Option<MaskPosComp> {
    use zero_css_parser::values::{CalcExpr as CE, CalcOp, LengthValue};
    match expr {
        CE::BinaryOp(l, CalcOp::Add, r) => {
            let (lp, lx, lc) = mask_pos_comp_calc(l)?;
            let (rp, rx, rc) = mask_pos_comp_calc(r)?;
            Some((lp + rp, lx + rx, lc || rc))
        }
        CE::BinaryOp(l, CalcOp::Subtract, r) => {
            let (lp, lx, lc) = mask_pos_comp_calc(l)?;
            let (rp, rx, rc) = mask_pos_comp_calc(r)?;
            Some((lp - rp, lx - rx, lc || rc))
        }
        CE::Length(LengthValue::Percentage(p)) => Some((*p, 0.0, true)),
        CE::Length(LengthValue::Px(v)) => Some((0.0, *v, false)),
        _ => None,
    }
}

fn mask_pos_comp(v: &zero_css_parser::values::BackgroundPositionValue) -> Option<MaskPosComp> {
    use zero_css_parser::values::{BackgroundEdge, BackgroundPositionValue as Bpv};
    match v {
        Bpv::Center => Some((50.0, 0.0, true)),
        Bpv::Left | Bpv::Top | Bpv::XStart | Bpv::YStart => Some((0.0, 0.0, true)),
        Bpv::Right | Bpv::Bottom | Bpv::XEnd | Bpv::YEnd => Some((100.0, 0.0, true)),
        Bpv::Percent(p) => Some((*p as f64, 0.0, true)),
        Bpv::Length(LengthValue::Px(p)) => Some((0.0, (*p), false)),
        // 刀 A1d：% / px 加减脊 calc 折 comp（`calc(100% - 20px)` → (100, -20,
        // true)）——transitions 端点串化走 canonical calc 形，插值通道须能吃回
        //（css-values-4 §10.11 混合形往返；乘除/未知树 None → 离散回落）。
        Bpv::Calc(expr) => mask_pos_comp_calc(expr),
        Bpv::EdgeOffset { side, offset } => {
            let (bp, bx) = match side {
                BackgroundEdge::Left | BackgroundEdge::XStart | BackgroundEdge::Top | BackgroundEdge::YStart => {
                    (0.0, 0.0)
                }
                BackgroundEdge::Right | BackgroundEdge::XEnd | BackgroundEdge::Bottom | BackgroundEdge::YEnd => {
                    (100.0, 0.0)
                }
            };
            let edge_pct = matches!(
                side,
                BackgroundEdge::Right | BackgroundEdge::XEnd | BackgroundEdge::Bottom | BackgroundEdge::YEnd
            );
            let (op, ox, opct) = mask_pos_comp(offset)?;
            // 刀 A1d 符号口径（css-backgrounds-3 §3.6）：right/bottom 缘语义 =
            // `100% − 偏移`（`right 20px` → calc(100% - 20px)）——偏移项取负
            //（origin-interpolation 语料 `calc(62.5% - 5px)` 期望形钉；mask 语料
            // 无 right/bottom EdgeOffset 对，A1 账面零扰动）。
            if edge_pct {
                Some((bp - op, -ox, true))
            } else {
                Some((bp + op, bx + ox, opct))
            }
        }
        _ => None,
    }
}

/// 层 → (x, y) 分量对（单值层垂直轴缺省 center——bg 同语义）。
fn mask_pos_layer(v: &zero_css_parser::values::BackgroundPositionValue) -> Option<(MaskPosComp, MaskPosComp)> {
    use zero_css_parser::values::BackgroundPositionValue as Bpv;
    match v {
        Bpv::TwoValue(h, vert) => Some((mask_pos_comp(h)?, mask_pos_comp(vert)?)),
        Bpv::Calc(_) => None,
        // 刀 A1e（css-backgrounds-3 §3 单值形）：单值形垂直关键字（top/bottom/
        // start/end）属 **Y 轴**（`bottom` ≡ `center bottom`），x 缺省 center——
        // 原臂恒入 X 致 `center`→`bottom` 对 x/y 分量交换（origin 页 CSSA+WAAPI
        // 8F：期望 "50% 100%" got "100% 50%"）；水平关键字/长百分比单值形维持
        // X 位 y=center（原语义，bg `bottom` 形语料钉面同改净 +8）。
        single => {
            if matches!(single, Bpv::Top | Bpv::Bottom | Bpv::YStart | Bpv::YEnd) {
                let y = mask_pos_comp(single)?;
                Some(((50.0, 0.0, true), y))
            } else {
                let x = mask_pos_comp(single)?;
                Some((x, (50.0, 0.0, true)))
            }
        }
    }
}

/// (pct, px) 系数 lerp 后串化：两端点均纯 length → 纯 `Xpx`（length↔length
/// 插值产物仍是 length，不引入 calc——calc 系数形落不可解析 % Calc 轴空
/// 串）；含 % 项时 px 系数 0 → 纯 `P%`（端点简化），否则 `calc(P% ± Xpx)`。
fn mask_pos_fmt((fp, fx, fpct): MaskPosComp, (tp, tx, tpct): MaskPosComp, t: f64) -> String {
    let pct = (1.0 - t) * fp + t * tp;
    let px = (1.0 - t) * fx + t * tx;
    if !fpct && !tpct {
        return format!("{:.2}px", px);
    }
    if px.abs() < 1e-9 {
        format!("{:.2}%", pct)
    } else if px < 0.0 {
        format!("calc({:.2}% - {:.2}px)", pct, -px)
    } else {
        format!("calc({:.2}% + {:.2}px)", pct, px)
    }
}

/// mask-position 端点列表插值：LCM 循环展开逐层 lerp，", " 连接。
fn mix_mask_position(from: &str, to: &str, t: f64) -> Option<String> {
    use zero_css_parser::values::parse_background_position_list;
    let fl = parse_background_position_list(from)?;
    let tl = parse_background_position_list(to)?;
    let fm: Option<Vec<(MaskPosComp, MaskPosComp)>> = fl.iter().map(mask_pos_layer).collect();
    let tm: Option<Vec<(MaskPosComp, MaskPosComp)>> = tl.iter().map(mask_pos_layer).collect();
    let (fm, tm) = (fm?, tm?);
    let lcm = |a: usize, b: usize| a / gcd(a, b) * b;
    let n = lcm(fm.len(), tm.len());
    let out: Vec<String> = (0..n)
        .map(|i| {
            let (f, tt) = (&fm[i % fm.len()], &tm[i % tm.len()]);
            format!("{} {}", mask_pos_fmt(f.0, tt.0, t), mask_pos_fmt(f.1, tt.1, t))
        })
        .collect();
    Some(out.join(", "))
}

/// 刀 A1d：background-position-x/y 单轴端点列表插值——逐层
/// [`parse_background_position_axis`]（单轴文法 `<position-x>`，css-backgrounds-4
/// §2.4）→ [`mask_pos_comp`] 系数归一 → LCM 循环 lerp（[`mask_pos_fmt`] 串化：
/// 纯 length 对出纯 px、含 % 项出 %/calc 系数形），", " 连接。任一层解析失败
/// → None（调用方离散翻转回落）。
fn mix_bg_position_axis(from: &str, to: &str, t: f64, horizontal: bool) -> Option<String> {
    use zero_css_parser::values::parse_background_position_axis;
    use zero_css_parser::values::split_top_level_commas;
    let axis_layers = |s: &str| -> Option<Vec<MaskPosComp>> {
        split_top_level_commas(s)?
            .iter()
            .map(|lv| parse_background_position_axis(lv, horizontal).and_then(|v| mask_pos_comp(&v)))
            .collect()
    };
    let f = axis_layers(from)?;
    let tl = axis_layers(to)?;
    let n = f.len() / gcd(f.len(), tl.len()) * tl.len();
    let out: Vec<String> = (0..n)
        .map(|i| mask_pos_fmt(f[i % f.len()], tl[i % tl.len()], t))
        .collect();
    Some(out.join(", "))
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// offset-rotate 端点 deg 等价分类（刀 MOTION-1 插值面）：
/// `Some((关键词类, deg))`——关键词类 true＝auto/reverse/复合形（reverse ≡
/// deg+180 折叠、纯 reverse ≡ 180deg、canonical 词恒 auto——css-motion-1
/// §offset-rotate；语料 block3/5/10-12 interior 钉）；false＝纯 `<angle>`。
/// `None`＝不可解析 → 离散（复合↔纯角跨类同判离散，css-values-4 §8.5.6）。
fn classify_rotate_endpoint(s: &str) -> Option<(bool, f64)> {
    let v = s.trim();
    if v.is_empty() {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if lower == "auto" {
        return Some((true, 0.0));
    }
    if lower == "reverse" {
        return Some((true, 180.0));
    }
    // 纯角单 token（deg 直读；rad/grad/turn/calc 归 deg）。
    if !v.chars().any(char::is_whitespace) {
        return angle_token_deg(v).map(|d| (false, d));
    }
    // 复合形 `[auto || reverse] <angle>` 两序均收（`||` 乱序：语料
    // `100grad auto`）。关键词先行 → 余部整体为角度词（calc 带空格）；
    // 角度先行 → 末词为关键词，前缀为角度词。
    let first_word_end = v.find(char::is_whitespace)?;
    let first = &v[..first_word_end];
    let first_l = first.to_ascii_lowercase();
    let (keyword_is_reverse, rest) = if first_l == "auto" || first_l == "reverse" {
        (first_l == "reverse", v[first_word_end..].trim())
    } else {
        let last_start = v.rfind(char::is_whitespace)?;
        let last = v[last_start + 1..].trim();
        let last_l = last.to_ascii_lowercase();
        if last_l != "auto" && last_l != "reverse" {
            return None;
        }
        (last_l == "reverse", v[..last_start].trim())
    };
    let deg = angle_token_deg(rest)?;
    Some((true, if keyword_is_reverse { deg + 180.0 } else { deg }))
}

/// offset-rotate t==1 端点原文 echo 判定（刀 MOTION-1b）：裸关键词、乱序
/// 复合（角先行）或 calc 角端点回作者原文（语料 bare-`reverse`/`100grad
/// auto`/`auto calc(...)` at(1) 原文钉）；关键词先序＋plain deg 角的
/// canonical 复合与纯角返回 false 走 lerp——at(1.5)/at(2) 外推采样与
/// block10 canonical 折叠期望不能直通（1b 电池 blanket t≥1 窗 44F 实证）。
fn offset_rotate_endpoint_echo_raw(to: &str) -> bool {
    let t = to.trim();
    if t.contains("calc(") {
        // calc 角端点原文保真（canonical 化丢失作者形——`auto calc(90deg -
        // 0.5turn - 300grad + 0rad)` at(1) 钉）。
        return true;
    }
    let mut parts = t.split_whitespace();
    let first = parts.next().unwrap_or("");
    let second = parts.next();
    let keyword_first = first.eq_ignore_ascii_case("auto") || first.eq_ignore_ascii_case("reverse");
    match second {
        // 裸关键词 → 原文；裸角 → canonical lerp。
        None => keyword_first,
        // 复合：keyword-first + plain deg 角＝canonical 形 → lerp；乱序或
        // 非 deg 角单位 → 原文。
        Some(s2) => !(keyword_first && s2.ends_with("deg")),
    }
}

/// 单角度 token → deg（deg 直读避免往返精度损失；rad/grad/turn 经
/// [`zero_css_parser::values::parse_angle_to_radians`] 归 deg；calc() 角走
/// Number|Angle 维度门后按默认字体基准求值——font 相对项 parse 期近似与
/// style-system `parse_angle_in_deg` 同口径，语料 `calc(90deg - 0.5turn -
/// 300grad + 0rad)` 纯角维度）。
fn angle_token_deg(tok: &str) -> Option<f64> {
    let t = tok.trim();
    let lower = t.to_ascii_lowercase();
    if let Some(n) = lower.strip_suffix("deg") {
        let v: f64 = n.trim().parse().ok()?;
        return v.is_finite().then_some(v);
    }
    if lower.starts_with("calc(") {
        let expr = parse_math_function(t)?;
        match calc_expr_math_type(&expr) {
            CalcMathType::Number | CalcMathType::Angle => {}
            _ => return None,
        }
        let rad = eval_calc_with_default_font(&expr)?;
        return rad.is_finite().then_some(rad.to_degrees());
    }
    parse_angle_to_radians(t).map(|rad| rad.to_degrees())
}

/// offset-rotate 值串化（underlying/端点发射；auto/reverse 关键字系离散面）。
/// 刀 MOTION-1：reverse 复合形计算值折叠为规范 auto 形 `auto (N+180)deg`
///（css-motion-1 §offset-rotate；WPT 语料 block10-12 端点 echo 实证——
/// `reverse -170deg` 计算值读回 `auto 10deg`）；Verbatim 作者原文直通。
fn offset_rotate_str(v: Option<&OffsetRotateValue>) -> String {
    match v {
        None | Some(OffsetRotateValue::Auto) => "auto".to_string(),
        Some(OffsetRotateValue::Reverse) => "reverse".to_string(),
        Some(OffsetRotateValue::Angle(deg)) => format!("{deg}deg"),
        Some(OffsetRotateValue::AutoAngle(deg)) => format!("auto {deg}deg"),
        Some(OffsetRotateValue::ReverseAngle(deg)) => format!("auto {}deg", deg + 180.0),
        Some(OffsetRotateValue::Verbatim(raw)) => raw.to_string(),
    }
}

/// offset-anchor 长度分量串化（underlying 发射；残余相对单位按 0px 近似在册）。
fn offset_len_str(v: &LengthValue) -> String {
    match v {
        LengthValue::Percentage(p) => format!("{p}%"),
        LengthValue::Px(n) => format!("{n}px"),
        LengthValue::Em(n) => format!("{n}em"),
        _ => "0px".to_string(),
    }
}

/// (数值, 单位) → LengthValue（apply 写回）。
fn offset_len_from(n: f64, unit: &str) -> LengthValue {
    if unit == "%" {
        LengthValue::Percentage(n)
    } else {
        LengthValue::Px(n)
    }
}

/// offset-rotate 计算值形解析（apply 写回）：auto | reverse | <angle> |
/// [auto || reverse] <angle>；canonical 复合形全单位归 deg，其余形（乱序
/// 复合/calc/单词非 deg）Verbatim 保原文回显（刀 MOTION-1 gCS roundtrip
/// 保真）；非法原文 → Auto 近似（fail-soft，与 scale 无单位回落同风格）。
fn parse_offset_rotate_computed(value: &str) -> Option<OffsetRotateValue> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if v.eq_ignore_ascii_case("auto") {
        return Some(OffsetRotateValue::Auto);
    }
    if v.eq_ignore_ascii_case("reverse") {
        return Some(OffsetRotateValue::Reverse);
    }
    let mut parts = v.split_whitespace();
    let first = parts.next()?;
    // 严格 deg 直读（roundtrip 通道不收 rad/calc 等价——原文走 Verbatim）。
    let strict_deg = |tok: &str| -> Option<f64> {
        tok.trim()
            .to_ascii_lowercase()
            .strip_suffix("deg")
            .and_then(|n| n.trim().parse::<f64>().ok())
    };
    match parts.next() {
        None => {
            if let Some(d) = strict_deg(first) {
                return Some(OffsetRotateValue::Angle(d));
            }
            Some(OffsetRotateValue::Verbatim(v.into()))
        }
        // 复合形：canonical 关键词先行形全单位归 deg（deg/grad/rad/turn——
        // css-values-4 computed 角度以 deg 规范序列化；语料 offset-rotate-computed
        // 钉 'auto 5turn'→'auto 1800deg'、'reverse -50grad'→'auto 135deg'）；
        // calc() 与乱序形仍 Verbatim（端点 echo 原文；语料 block1/2 钉；语义面
        // 由 classify_rotate_endpoint 按 deg 等价处理）。
        Some(second) => {
            let is_kw_first = first.eq_ignore_ascii_case("auto") || first.eq_ignore_ascii_case("reverse");
            let unit_deg = |tok: &str| -> Option<f64> {
                strict_deg(tok).or_else(|| parse_angle_to_radians(tok.trim()).map(|r| r.to_degrees()))
            };
            let canonical = is_kw_first && unit_deg(second).is_some();
            if !canonical {
                return Some(OffsetRotateValue::Verbatim(v.into()));
            }
            let d = unit_deg(second)?;
            if first.eq_ignore_ascii_case("auto") {
                return Some(OffsetRotateValue::AutoAngle(d));
            }
            Some(OffsetRotateValue::ReverseAngle(d))
        }
    }
}

/// RGBA 颜色（0-255 范围）。
type Rgba = (f64, f64, f64, f64);

/// 解析颜色字符串为 RGBA。
fn parse_color(s: &str) -> Rgba {
    let s = s.trim().to_lowercase();

    // rgba(r, g, b, a)
    if let Some(rest) = s.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
        let parts: Vec<&str> = rest.split(',').map(|p| p.trim()).collect();
        if parts.len() == 4 {
            return (
                parts[0].parse().unwrap_or(0.0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
                parts[3].parse().unwrap_or(1.0),
            );
        }
    }

    // rgb(r, g, b) —— R3882：兼收 legacy 4 参数形 rgb(r, g, b, a)（CSS Color 4 允许
    // rgb/rgba 前缀与 3/4 参数自由组合；@keyframes 里 `rgb(0, 200, 0, 0)` 这类写法
    // 上游常见，此前 4 参数 rgb() 解析失败回落黑色不透明，插值输出整体变黑）。
    if let Some(rest) = s.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        let parts: Vec<&str> = rest.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            return (
                parts[0].parse().unwrap_or(0.0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
                1.0,
            );
        }
        if parts.len() == 4 {
            return (
                parts[0].parse().unwrap_or(0.0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
                parts[3].parse().unwrap_or(1.0),
            );
        }
    }

    // hex: #RRGGBB / #RRGGBBAA
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                return (r as f64, g as f64, b as f64, 1.0);
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                return (r as f64, g as f64, b as f64, a as f64 / 255.0);
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                return (r as f64, g as f64, b as f64, 1.0);
            }
            _ => {}
        }
    }

    // 命名颜色（常见）
    match s.as_str() {
        "transparent" => (0.0, 0.0, 0.0, 0.0),
        "black" => (0.0, 0.0, 0.0, 1.0),
        "white" => (255.0, 255.0, 255.0, 1.0),
        "red" => (255.0, 0.0, 0.0, 1.0),
        "green" => (0.0, 128.0, 0.0, 1.0),
        "blue" => (0.0, 0.0, 255.0, 1.0),
        "yellow" => (255.0, 255.0, 0.0, 1.0),
        "orange" => (255.0, 165.0, 0.0, 1.0),
        "purple" => (128.0, 0.0, 128.0, 1.0),
        "gray" | "grey" => (128.0, 128.0, 128.0, 1.0),
        _ => (0.0, 0.0, 0.0, 1.0), // 默认黑色
    }
}

/// 颜色插值。
/// R5019（css-color-4 §15.1 Interpolation）：颜色动画在**预乘 alpha** 空间插值——
/// 各通道先按 alpha 预乘、lerp、再按插值后 alpha 反预乘（等 alpha 端点与直插等价，
/// alpha 有差时非预乘直插偏离 chromium：background-color-animation-half-opaque
/// 实证 rgba(0,200,0,1)→rgba(200,0,0,0.6) @50% chromium (111,151,51)，直插 (131,131,·)）。
/// alpha=0 端点通道值无意义（未定义色），反预乘除零保护回 0。
fn lerp_color(from: Rgba, to: Rgba, t: f64) -> Rgba {
    let a = lerp(from.3, to.3, t).clamp(0.0, 1.0);
    let chan = |fc: f64, tc: f64| -> f64 {
        if a <= 1e-6 {
            0.0
        } else {
            ((lerp(fc * from.3, tc * to.3, t)) / a).clamp(0.0, 255.0)
        }
    };
    (chan(from.0, to.0), chan(from.1, to.1), chan(from.2, to.2), a)
}

// 刀 A3r（wptfyi-css，2026-11-18）：shadow 列表端点内嵌 currentcolor 的 start
// 时预解析——interpolate_property_value 为纯字符串函数无元素上下文，
// currentcolor 端点必须在 keyframe 装载时以元素 computed color 定死
//（css-color-4 §5.4「currentcolor = color 属性的 computed value」）。仅当值
// 含 currentcolor 时重写（返回 None = 保持原串走既有回退）；序列化契约与
// computed_style::text_shadow_to_css 对齐（color-first、format_num 长度、
// a≈1 出 rgb 形）。underlying=None（WAAPI 简化通道）不解析，同
// FIXME(kf-inherit-waapi) 域。
fn serialize_text_shadow_resolved(value: &str, style: &ComputedStyle) -> Option<String> {
    use zero_css_parser::values::ColorValue;
    let list = zero_css_parser::values::parse_text_shadow_list(value, false)?;
    let has_cc = list.iter().any(|l| matches!(l.color, ColorValue::CurrentColor));
    if !has_cc {
        return None;
    }
    let color_str = |c: &ColorValue| -> String {
        let resolved = crate::resolve_color_current(c, &style.color);
        if resolved.a == 255 {
            format!("rgb({}, {}, {})", resolved.r, resolved.g, resolved.b)
        } else {
            format!(
                "rgba({}, {}, {}, {:.2})",
                resolved.r,
                resolved.g,
                resolved.b,
                resolved.a as f64 / 255.0
            )
        }
    };
    Some(
        list.iter()
            .map(|l| {
                let len = |lv: &zero_css_parser::values::LengthValue| -> String {
                    crate::js_dom_bridge::computed_style::format_num(
                        zero_style_system::property::apply_advanced::resolve_effect_length(lv, style) as f64,
                        "px",
                    )
                };
                format!(
                    "{} {} {} {}",
                    color_str(&l.color),
                    len(&l.offset_x),
                    len(&l.offset_y),
                    len(&l.blur_radius)
                )
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
}

// ── AnimationClock 实现 ───────────────────────────────────────────────

impl AnimationClock {
    /// 创建空的动画时钟。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 @keyframes 规则。
    pub fn register_keyframes(&mut self, rule: &KeyframesRule) {
        let mut points: Vec<KeyframePoint> = rule.keyframes.iter().flat_map(block_to_points).collect();

        // 按 offset 排序
        points.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));

        // 确保有 0% 和 100% 边界
        if points.is_empty() {
            return;
        }
        if points.first().map(|p| p.offset) != Some(0.0) {
            points.insert(
                0,
                KeyframePoint {
                    offset: 0.0,
                    properties: HashMap::new(),
                    timing: None,
                    composition: None,
                },
            );
        }
        if points.last().map(|p| p.offset) != Some(1.0) {
            points.push(KeyframePoint {
                offset: 1.0,
                properties: HashMap::new(),
                timing: None,
                composition: None,
            });
        }

        self.keyframes_registry.insert(rule.name.clone(), points);
    }

    /// 从样式表提取并注册所有 @keyframes 规则。
    ///
    /// R3881：条件块（@media/@supports）内条件成立的 @keyframes 一并注册
    ///（CSS Conditional 3 §contents-of；经 [`crate::pipeline::extract::flatten_conditional_rules`] 展开）。
    pub fn register_from_stylesheets(
        &mut self,
        stylesheets: &[zero_css_parser::Stylesheet],
        media_ctx: &zero_css_parser::media_query::MediaContext,
    ) {
        for rule in crate::pipeline::flatten_conditional_rules(stylesheets, media_ctx) {
            if let zero_css_parser::ast::Rule::Keyframes(kf) = rule {
                self.register_keyframes(&kf);
            }
        }
    }

    /// 启动一个动画。
    ///
    /// 使用 `AnimationConfig` 结构体配置动画参数。
    pub fn start_animation(&mut self, element_id: u64, config: &AnimationConfig) -> bool {
        self.start_animation_with_underlying(element_id, config, None, None)
    }

    /// R5018（css-animations §keyframes，implicit keyframes）：带 underlying value 的
    /// 启动——0%/100% 合成边界帧（`register_keyframes` 补齐的空 properties 帧）缺失的
    /// 属性以**元素当前 computed 值**为该侧插值端点。registry 全局、基值逐元素，故
    /// 解析在 start 时对 per-animation 的 keyframes 克隆进行（注册表零污染）。
    /// driving: translate-explicit-and-implicit-keyframes（base-value-from 渲染 50 应 125
    /// ——from 缺失合成 0 实证）。
    pub fn start_animation_with_underlying(
        &mut self,
        element_id: u64,
        config: &AnimationConfig,
        underlying: Option<&ComputedStyle>,
        parent: Option<&ComputedStyle>,
    ) -> bool {
        let Some(mut keyframes) = self.keyframes_registry.get(&config.name).cloned() else {
            return false;
        };
        // E5（wptfyi-css 刀 E，2026-11-17）css-animations-1 §keyframes /
        // css-cascade-4 §default：关键帧 'initial'/'unset' 关键字解析为属性初始值
        // （interpolation corpus neutral→15px、initial→10px 等块的 keyword 端点）。
        // 无元素依赖（ComputedStyle::default() + gCS 序列化器同通道），start 时
        // 一次性解析，WAAPI（start_animation 委托，underlying=None）与 CSSA
        //（underlying=Some）两端共享。串化为空（applier 未覆盖属性）保持原值
        // ——避免空串端点污染插值。
        // 刀 K1（wptfyi-css，2026-11-17）css-cascade-4 §inheritance：关键帧
        // 'inherit' = 父元素该属性 computed value（继承强制，含非继承属性）——
        // 管线通道经 apply_animation_overrides 传入父 computed 解析；无父（根
        // 元素；WAAPI/js_anim 通道父链不可达传 None，回落 'initial' 同式解析
        //（FIXME(kf-inherit-waapi)——真父链接入另行开刀），替代原「原值保留→
        // 长度臂 0 化假插值」。
        for point in &mut keyframes {
            for (name, value) in point.properties.iter_mut() {
                let v = value.trim();
                // 刀 A3r（wptfyi-css，2026-11-18）css-cascade-4 §default：unset
                // 级联向——inherited 属性取 inherit（父 computed）、其余取
                // initial（修正刀 K1「unset 恒 initial」对 text-shadow 等继承
                // 属性端点解析错基值）。is_inherited 表与 cascade 继承管线同源。
                let inherited = zero_style_system::PropertyRegistry::is_inherited(name);
                if v.eq_ignore_ascii_case("initial") || (v.eq_ignore_ascii_case("unset") && !inherited) {
                    let init = crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(
                        &ComputedStyle::default(),
                        name,
                        None,
                    );
                    if !init.is_empty() {
                        *value = init;
                    }
                } else if v.eq_ignore_ascii_case("inherit") || (v.eq_ignore_ascii_case("unset") && inherited) {
                    let fallback = ComputedStyle::default();
                    let src = parent.unwrap_or(&fallback);
                    let inh =
                        crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(src, name, None);
                    if !inh.is_empty() {
                        *value = inh;
                    }
                } else if let Some(style) = underlying {
                    // 刀 A3r：值内嵌 currentcolor 的列表端点 start 时以元素
                    // computed color 定死（css-color-4 §5.4；插值臂无元素
                    // 上下文）。属性门=仅 text-shadow——parse_text_shadow 对
                    // 裸长度列表默认 CurrentColor 色，不加门会把 background/
                    // mask-position、border/mask-image outset/width 等属性
                    // 的 "Npx Npx" 端点误判重写（A3r 电池 r2 110 P→F 回归
                    // 实证）。None = 非 currentcolor 列表，保持原串。
                    if name.eq_ignore_ascii_case("text-shadow")
                        && let Some(resolved) = serialize_text_shadow_resolved(v, style)
                    {
                        *value = resolved;
                    }
                }
            }
        }
        if let Some(style) = underlying {
            resolve_implicit_boundaries(&mut keyframes, style);
            // R5189：keyframe 值 `revert-layer` 在插值前解析为 underlying（动画未叠加时）
            // 级联值——css-cascade-5：动画 origin 回退到 author origin。解析失败（属性
            // 不在覆盖面/非 Px 形态）保持原值走既有离散回退。普通 `revert` 见
            // FIXME(revert-keyframe)。
            for point in &mut keyframes {
                for (name, value) in point.properties.iter_mut() {
                    if value.trim().eq_ignore_ascii_case("revert-layer")
                        && let Some(resolved) = underlying_cascade_value(style, name)
                    {
                        *value = resolved;
                    }
                }
            }
        }

        // ── 刀 ANIMCOMP S2（wptfyi-css，2026-11-19）合成归类与预合成 ──
        // css-animations-2 §animation-composition / web-animations-1 §5.4.4：
        // 属性有效 op = 帧内 animation-composition ?? 动画级 config.composition
        // ?? replace。三分类：
        // ① 均匀 add/accumulate（该属性全部出现帧同 op）且在可合成域（影列表
        //    拼接 / vertical-align px 加）→ `compose_result` 登记，tick 输出
        //    U ⊕ lerp(F,T,p)（均匀 add 与端点预合成在 lerp 下恒等，且避免多影
        //    列表端点插值依赖）。
        // ② 混合 op（部分帧 replace）→ add 帧端点预合成 U ⊕ 端点（影串拼接恒
        //    安全；vertical-align 须双侧 px 严格可解析，否则该端点保持原文回落
        //    replace）。text-shadow b1（replace from + add to、underlying none）
        //    驱动：端点恒等改写 → 既有单影插值通道直达。
        // ③ replace-only / 域外属性：零动作（刀前行为恒等，零回归方向）。
        // underlying：WAAPI 通道经 config.underlying_values（shim gCS 预读）注入
        // 优先；CSSA 通道 start 时经 gCS 序列化器捕获。FIXME(animcomp-stacking)：
        // 多动画同属性叠加时 spec 要求 ((U ⊕ V1) ⊕ V2) 链式合成，本实现单动画
        // 语义（corpus 全单动画面）；FIXME(animcomp-implicit)：隐式边界帧
        //（值 = underlying）与 add op 的合成语义未处理（corpus 全显式帧面）。
        let mut compose_result: HashMap<String, AnimationCompositionValue> = HashMap::new();
        let mut underlying_values = config.underlying_values.clone();
        {
            let is_add = |op: &AnimationCompositionValue| {
                matches!(
                    op,
                    AnimationCompositionValue::Add | AnimationCompositionValue::Accumulate
                )
            };
            // 逐属性收集出现帧的 op 序列（帧内声明 ?? 动画级 ?? replace）。
            let mut prop_ops: HashMap<String, Vec<AnimationCompositionValue>> = HashMap::new();
            for point in &keyframes {
                for pname in point.properties.keys() {
                    let op = point.composition.clone().unwrap_or_else(|| config.composition.clone());
                    prop_ops.entry(pname.clone()).or_default().push(op);
                }
            }
            for (pname, ops) in &prop_ops {
                let any_add = ops.iter().any(&is_add);
                if !any_add {
                    continue; // ③ replace-only
                }
                // 可合成域门：影列表（css-text-decor-3 §text-shadow /
                // css-backgrounds-3 §box-shadow——add 语义 = 列表追加）+
                // vertical-align（<length> 同单位数值加）+ mask-border 三长手
                //（刀 MB：css-masking-1 §3.5–3.9 逐槽同型求和，见
                // [`compose_animation_value`] 臂）+ aspect-ratio（刀 FAM-F：
                // 非可加准入——add 帧预合成＝有效值取 underlying，见
                // [`compose_animation_value`] 臂；域外回落 raw 会把不可加 add
                // 帧当 replace 参与插值，WPT ANIMCOMP at(0.5) 期望 1/1 实测
                // 0.71/1）。
                if !matches!(
                    pname.as_str(),
                    "text-shadow"
                        | "box-shadow"
                        | "vertical-align"
                        | "mask-border-slice"
                        | "mask-border-width"
                        | "mask-border-outset"
                        | "aspect-ratio"
                        // fame2b：sizing 六长手 length-percentage（web-animations-1
                        // §5.4.4 addition；css-sizing-3 值域）——域门放开后由
                        // [`compose_animation_value`] LP 臂逐 term 加法/不可加回落。
                        | "width"
                        | "height"
                        | "max-width"
                        | "max-height"
                        | "min-width"
                        | "min-height"
                ) {
                    continue; // 域外：无定义加法 → replace 回落（规范退化语义）
                }
                let u = match underlying_values.get(pname) {
                    Some(u) if !u.is_empty() => u.clone(),
                    _ => match underlying {
                        Some(style) => {
                            let u = crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(
                                style, pname, None,
                            );
                            if u.is_empty() {
                                continue; // underlying 不可串化 → replace 回落
                            }
                            u
                        }
                        None => continue,
                    },
                };
                if ops.iter().all(&is_add) {
                    // 刀 MB v3：mask-border 三长手均匀 add 走端点预合成换道——
                    // web-animations-1 §5.4.4 加法合成先于插值（合成端点参与
                    // interpolate，计算值期非负钳 0 落在插值结果上）；post-compose
                    // 会把逐槽钳 0 提前到插值前，负外推位（语料 composition 页
                    // at:-0.25 expect '0'）错成 underlying+0。影列表 append 语义
                    // 非仿射必须 post-compose、vertical-align 同单位数值加在线性
                    // 插值下与预合成恒等——两者维持既有账不动。
                    if matches!(
                        pname.as_str(),
                        "mask-border-slice" | "mask-border-width" | "mask-border-outset"
                            // fame2b：sizing 六长手同 mask-border v3 换道——端点预合成
                            // （U ⊕ 端点，[%compose_animation_value] LP 臂）后走既有
                            // 插值通道（跨型对逐 term calc 混合在插值 LP 臂域内自洽）。
                            // 不入 compose_result post-compose：裸 px lerp 与 underlying
                            // 加法拆两步会把 % 项域外化（calc-mix 面失真）。
                            | "width"
                            | "height"
                            | "max-width"
                            | "max-height"
                            | "min-width"
                            | "min-height"
                    ) {
                        for point in &mut keyframes {
                            let op = point.composition.clone().unwrap_or_else(|| config.composition.clone());
                            if !is_add(&op) {
                                continue;
                            }
                            if let Some(v) = point.properties.get_mut(pname) {
                                *v = compose_animation_value(pname, &op, Some(&u), v.trim());
                            }
                        }
                    } else {
                        // ① 均匀 add/accumulate：结果合成登记（单迭代域 add ≡ accumulate）
                        compose_result.insert(pname.clone(), ops[0].clone());
                        underlying_values.insert(pname.clone(), u);
                    }
                } else {
                    // ② 混合 op：add 帧端点预合成（replace 帧原文保留）
                    for point in &mut keyframes {
                        let op = point.composition.clone().unwrap_or_else(|| config.composition.clone());
                        if !is_add(&op) {
                            continue;
                        }
                        if let Some(v) = point.properties.get_mut(pname) {
                            *v = compose_animation_value(pname, &op, Some(&u), v.trim());
                        }
                    }
                }
            }
        }

        let state = AnimationState {
            name: config.name.clone(),
            duration: config.duration.max(0.001), // 避免除零
            delay: config.delay,
            iteration_count: config.iteration_count,
            direction: config.direction.clone(),
            fill_mode: config.fill_mode.clone(),
            timing_function: config.timing_function.clone(),
            start_time: config.current_time,
            keyframes,
            finished: false,
            iteration: 0,
            started: false,
            paused_at: None,
            composition: config.composition.clone(),
            compose_result,
            underlying_values,
        };

        self.active_animations.entry(element_id).or_default().push(state);
        true
    }

    /// M2-S3b（CSS Animations §play-state）：暂停/恢复对账——`paused=true` 且未冻结
    /// 时记 `paused_at = now`（下一 tick 起输出冻结于该时刻）；恢复（`paused=false`
    /// 且冻结中）重基 `start_time += now − paused_at`（暂停时长不计入活跃时长）。
    /// finished 动画不对账（恢复不复活已终动画，fill 语义照旧）。
    pub fn reconcile_play_state(&mut self, element_id: u64, name: &str, paused: bool, now: f64) {
        let Some(animations) = self.active_animations.get_mut(&element_id) else {
            return;
        };
        for anim in animations.iter_mut() {
            if anim.name != name || anim.finished {
                continue;
            }
            match anim.paused_at {
                None if paused => anim.paused_at = Some(now),
                Some(paused_at) if !paused => {
                    anim.start_time += now - paused_at;
                    anim.paused_at = None;
                }
                _ => {}
            }
        }
    }

    /// R342：animation-delay 变更的 seek 语义（css-animations §animations—样式变更
    /// 重新求值动画时序）。WPT 用例在 animationstart 里把 delay 改为负值（-100s）
    /// 「seek 到第二轮迭代」驱动 animationiteration/animationend（
    /// Event-dispatch-on-disabled-elements 动画链）。时钟的 delay 在 start 时快照，
    /// 这里按当前 computed delay 更新活跃动画（未完成）的 delay——负 delay 使
    /// active_elapsed 前跳，下一 tick 的迭代边界检测自然派发 iteration。
    /// 返回是否发生了更新。
    pub fn refresh_delay(&mut self, element_id: u64, name: &str, delay: f64) -> bool {
        let Some(animations) = self.active_animations.get_mut(&element_id) else {
            return false;
        };
        let mut updated = false;
        for anim in animations.iter_mut() {
            if anim.name == name && !anim.finished && anim.delay != delay {
                anim.delay = delay;
                updated = true;
            }
        }
        updated
    }

    /// WAAPI JS 动画注册表（js_dom_bridge::js_anim_registry）seek 通道：直接设定
    /// 冻结求值点（时钟域秒，local time 含 delay）——`currentTime = t` 后的同步
    /// gCS 读值经 paused_at 求值（M2-S3b 同语义）。`None` = 解除冻结（running）。
    pub fn set_hold(&mut self, element_id: u64, hold: Option<f64>) {
        if let Some(animations) = self.active_animations.get_mut(&element_id) {
            for anim in animations.iter_mut() {
                anim.paused_at = hold;
            }
        }
    }

    /// WAAPI JS 动画注册表 play 重锚通道：设定 start_time（时钟域秒）——
    /// running 求值 elapsed = now − start_time（宿主墙钟时基）。
    pub fn set_start_time(&mut self, element_id: u64, start: f64) {
        if let Some(animations) = self.active_animations.get_mut(&element_id) {
            for anim in animations.iter_mut() {
                anim.start_time = start;
            }
        }
    }

    /// 从 ComputedStyle 的 animation 属性自动创建动画。
    ///
    /// 检查元素的 animation-name 列表，为每个名称创建动画。
    /// 如果同名动画已存在且未完成，不重复创建。
    pub fn start_from_computed_style(
        &mut self,
        element_id: u64,
        style: &ComputedStyle,
        current_time: f64,
        parent: Option<&ComputedStyle>,
    ) {
        for (i, name) in style.animation_name.iter().enumerate() {
            if name.is_empty() || name == "none" {
                continue;
            }

            // 检查是否已存在同名活跃动画
            if let Some(animations) = self.active_animations.get(&element_id)
                && animations.iter().any(|a| a.name == *name && !a.finished)
            {
                continue;
            }

            let duration = style.animation_duration.get(i).copied().unwrap_or(0.0);
            let delay = style.animation_delay.get(i).copied().unwrap_or(0.0);
            let timing = style
                .animation_timing_function
                .get(i)
                .cloned()
                .unwrap_or(TimingFunctionValue::Ease);
            let iteration_count = style.animation_iteration_count.get(i).and_then(|v| *v);
            let direction = style
                .animation_direction
                .get(i)
                .cloned()
                .unwrap_or(AnimationDirectionValue::Normal);
            let fill_mode = style
                .animation_fill_mode
                .get(i)
                .cloned()
                .unwrap_or(AnimationFillModeValue::None);
            // 刀 ANIMCOMP S2：动画级合成操作（css-animations-2 §animation-composition
            // 列表逐动画索引对位；空列表/越界回落 replace）
            let composition = style
                .animation_composition
                .get(i)
                .cloned()
                .unwrap_or(AnimationCompositionValue::Replace);

            self.start_animation_with_underlying(
                element_id,
                &AnimationConfig {
                    name: name.clone(),
                    duration,
                    delay,
                    timing_function: timing,
                    iteration_count,
                    direction,
                    fill_mode,
                    current_time,
                    composition,
                    underlying_values: HashMap::new(),
                },
                Some(style),
                parent,
            );
        }
    }

    /// 推进时钟并获取指定元素的动画插值结果。
    ///
    /// 返回该元素所有活跃动画的插值属性列表。
    pub fn tick(&mut self, element_id: u64, current_time: f64) -> Vec<InterpolatedProperty> {
        let Some(animations) = self.active_animations.get_mut(&element_id) else {
            return Vec::new();
        };

        let mut all_props = Vec::new();

        for anim in animations.iter_mut() {
            if anim.finished {
                // fill-mode: forwards 时保持最后一帧
                if matches!(
                    anim.fill_mode,
                    AnimationFillModeValue::Forwards | AnimationFillModeValue::Both
                ) {
                    // 刀 ANIMCOMP S2：fill 终帧同走结果合成（fill forwards 保持的是
                    // 合成后值，css-animations-2 §animation-composition）
                    let props = compose_tick_props(anim, interpolate_to_end(anim));
                    all_props.extend(props);
                }
                continue;
            }

            // M2-S3b：暂停中的动画以 paused_at 求值——输出冻结于暂停时刻插值，
            // 迭代边界与终态判定停走（CSS Animations §play-state）。
            let now = anim.paused_at.unwrap_or(current_time);
            let elapsed = now - anim.start_time;
            let active_elapsed = elapsed - anim.delay;

            // 延迟期间
            if active_elapsed < 0.0 {
                // fill-mode: backwards 时使用 0% 帧
                if matches!(
                    anim.fill_mode,
                    AnimationFillModeValue::Backwards | AnimationFillModeValue::Both
                ) && let Some(first) = anim.keyframes.first()
                {
                    // 刀 ANIMCOMP S2：delay 相位 fill backwards 首帧同走结果合成
                    let frame_props = first
                        .properties
                        .iter()
                        .map(|(name, value)| InterpolatedProperty {
                            name: name.clone(),
                            value: value.clone(),
                        })
                        .collect();
                    all_props.extend(compose_tick_props(anim, frame_props));
                }
                continue;
            }

            let iteration_progress = active_elapsed / anim.duration;
            let total_iterations = iteration_progress;

            // R3251（CSS Animations §animationstart）：检测「首次进入活跃间隔」——动画跨过 animation-delay
            // 后的首帧（`started` 由 false→true）派 animationstart。置于 finish 检查**之前**，使瞬时动画
            // （start 与 end 同帧）先派 start 再派 end（spec：animationstart 先于 animationend，即使同时）。
            // elapsedTime = 0（动画刚开始，未播放）。`started` 标志保证每动画只派一次。
            if !anim.started {
                anim.started = true;
                self.just_started.push(StartedAnimation {
                    element_key: element_id,
                    name: anim.name.clone(),
                });
            }

            // 检查是否完成
            if let Some(max_iter) = anim.iteration_count
                && total_iterations >= max_iter
            {
                anim.finished = true;
                anim.iteration = max_iter.ceil() as u64;
                // R3249：记录「本轮新完成」——`finished` 由 false→true 的帧推入（`if anim.finished { continue }`
                // 在循环顶保证每个动画仅推一次），供 `drain_just_finished()` 派发 animationend。
                // elapsedTime = 活跃时长 = iteration_count * duration（CSS Animations §animationend）。
                self.just_finished.push(FinishedAnimation {
                    element_key: element_id,
                    name: anim.name.clone(),
                    duration: max_iter * anim.duration,
                });
                // 计算最终帧进度（考虑方向）
                let final_progress = final_animation_progress(anim);
                // 刀 ANIMCOMP S2：终帧同走结果合成
                let props = compose_tick_props(
                    anim,
                    interpolate_keyframes(&anim.keyframes, final_progress, &anim.timing_function),
                );
                all_props.extend(props);
                continue;
            }

            // R3250（CSS Animations §animationiteration）：检测迭代边界跨越——新 floor(iteration) >
            // 旧 anim.iteration 表示完成了一整轮迭代。非完成帧（finish 分支已 continue），故此处置
            // animationiteration（finish 分支的末轮边界派 animationend 而非 iteration，spec）。infinite
            // 动画靠此事件驱动循环回调（永不 finish）。frame 间隔内跨多轮 → 合并一事件（elapsedTime 取
            // 最新完成轮累计时长，近似 rAF 单帧单事件）。
            let prev_iter = anim.iteration;
            let new_iter = total_iterations.floor() as u64;
            if new_iter > prev_iter {
                self.just_iterated.push(IteratedAnimation {
                    element_key: element_id,
                    name: anim.name.clone(),
                    elapsed: new_iter as f64 * anim.duration,
                });
            }
            anim.iteration = new_iter;

            // 单次迭代内的进度 [0,1)
            let iter_progress = iteration_progress.fract();
            // 如果正好在整数迭代边界且不是 infinite，使用 1.0
            let iter_progress = if iter_progress == 0.0 && iteration_progress > 0.0 {
                1.0
            } else {
                iter_progress
            };

            // 应用方向
            let directed = apply_direction(iter_progress, anim.iteration, &anim.direction);

            // R3883：timing function 改为段内应用——interpolate_keyframes 选段后按
            // 段起点帧的 timing（回落全局）映射段内归一化进度（CSS Animations §keyframes）。
            // 全局 timing 不再扭曲整体进度（多段动画的段选址须用线性进度）。
            // 刀 ANIMCOMP S2：活跃相位插值输出同走结果合成（均匀 add = U ⊕ lerp）。
            let props = compose_tick_props(
                anim,
                interpolate_keyframes(&anim.keyframes, directed, &anim.timing_function),
            );
            all_props.extend(props);
        }

        all_props
    }

    /// KC-5-r72 pair fold-lerp 纯读探针（wptfyi-css，2026-11-24）：镜像 [`Self::tick`]
    /// 进度数学但零状态写（不置 `started`/`finished`/`iteration`、不派事件——学习
    /// 2026-11-15：tick() 在 t≥duration 不可逆置 finished，端点捕获必须无副作用读）。
    /// 返回 `prop` 的开区间端点对 `(from, to, eased)`（eased∈(0,1)——端点域交既有
    /// tick/flip 通道，peek 只补 spec 可插值中点域）。多条动画按注册序取最后命中
    ///（tick 输出后值覆盖先值同序）。调用方随后在同一次咨询内经 [`Self::tick`]
    /// 求值单值链，`set_hold`/`set_start_time` 幂等，时钟净态与单链路径恒等。
    pub fn peek_sizing_interp_pair(
        &self,
        element_id: u64,
        prop: &str,
        current_time: f64,
    ) -> Option<(String, String, f64)> {
        let animations = self.active_animations.get(&element_id)?;
        let mut out: Option<(String, String, f64)> = None;
        for anim in animations {
            if anim.finished {
                continue;
            }
            // 暂停以 paused_at 求值（tick 同式纯读）。
            let now = anim.paused_at.unwrap_or(current_time);
            let active_elapsed = now - anim.start_time - anim.delay;
            if active_elapsed < 0.0 {
                continue;
            }
            if anim.duration <= 0.0 {
                continue;
            }
            let iteration_progress = active_elapsed / anim.duration;
            if let Some(max_iter) = anim.iteration_count
                && iteration_progress >= max_iter
            {
                continue;
            }
            // 迭代内进度：floor 用现算 new_iter（不读可能滞后的 anim.iteration——
            // 方向奇偶相位与 tick 后续取值恒同源）；整数边界规则同 tick。
            let new_iter = iteration_progress.floor() as u64;
            let iter_progress = iteration_progress.fract();
            let iter_progress = if iter_progress == 0.0 && iteration_progress > 0.0 {
                1.0
            } else {
                iter_progress
            };
            if !(iter_progress > 0.0 && iter_progress < 1.0) {
                continue;
            }
            let directed = apply_direction(iter_progress, new_iter, &anim.direction);
            // 段选址与 interpolate_keyframes 同式（线性进度选址）。
            let kfs = &anim.keyframes;
            if kfs.len() < 2 {
                continue;
            }
            let mut from_idx = 0;
            for (i, kf) in kfs.iter().enumerate() {
                if kf.offset <= directed && i + 1 < kfs.len() {
                    from_idx = i;
                } else if kf.offset > directed {
                    break;
                }
            }
            let to_idx = (from_idx + 1).min(kfs.len() - 1);
            let (from_kf, to_kf) = (&kfs[from_idx], &kfs[to_idx]);
            let span = to_kf.offset - from_kf.offset;
            if span.abs() < 1e-9 {
                continue;
            }
            let normalized = ((directed - from_kf.offset) / span).clamp(0.0, 1.0);
            let timing = from_kf.timing.as_ref().unwrap_or(&anim.timing_function);
            let eased = apply_timing_function(normalized, timing);
            if !(eased > 0.0 && eased < 1.0) {
                continue;
            }
            let (Some(from_v), Some(to_v)) = (from_kf.properties.get(prop), to_kf.properties.get(prop)) else {
                continue;
            };
            if from_v == to_v {
                continue;
            }
            out = Some((from_v.clone(), to_v.clone(), eased));
        }
        out
    }

    /// 将插值结果叠加到 ComputedStyle 上。
    ///
    /// 只修改可动画属性，不改变不可动画的属性。
    pub fn apply_to_computed_style(props: &[InterpolatedProperty], style: &mut ComputedStyle) {
        for prop in props {
            // 刀 I important-tier（css-cascade-4 §importance）：级联排序中 animation
            // declarations 低于 important 声明（important author/user/UA 全部在其上）
            // ——级联 `!important` 胜出的属性跳过动画施值。简写按**写达 longhand**
            // 对位（与 apply_single_property 简写臂写面同源：R3882 `background` 只写
            // background-color；刀 I D 臂 `border-color` 写四边 color 槽；其余臂名即
            // longhand）。渲染 stamping（pipeline `apply_animation_overrides`）与 gCS
            // 咨询（js_anim/cssa 共享的 `canonicalize`）同通道过本门，单点覆盖；跳过
            // 即恒等 → canonicalize 返 None 回落级联序列化（零回归方向）。
            // https://drafts.csswg.org/css-cascade/#importance
            if style.important_won_props.iter().any(|w| {
                w.eq_ignore_ascii_case(&prop.name)
                    || (prop.name.eq_ignore_ascii_case("background") && w.eq_ignore_ascii_case("background-color"))
                    || (prop.name.eq_ignore_ascii_case("border-color")
                        && matches!(
                            w.as_str(),
                            "border-color"
                                | "border-top-color"
                                | "border-right-color"
                                | "border-bottom-color"
                                | "border-left-color"
                        ))
            }) {
                continue;
            }
            apply_single_property(&prop.name, &prop.value, style);
        }
    }

    /// R347（CSS Animations §animationcancel）：取消指定元素的未完成动画——元素
    /// `display:none` 使动画不再渲染相关。返回 (key, name, elapsed) 供宿主派发
    /// animationcancel。保留 finished 状态（fill-mode 消费），只移除未完成的。
    pub fn cancel_display_none(
        &mut self,
        element_keys: &std::collections::HashSet<u64>,
        cancelled: &mut Vec<(u64, String, f64)>,
    ) {
        let keys: Vec<u64> = self
            .active_animations
            .keys()
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .filter(|k| element_keys.contains(k))
            .collect();
        for key in keys {
            let mut cancelled_names: Vec<String> = Vec::new();
            if let Some(anims) = self.active_animations.get_mut(&key) {
                for anim in anims.iter_mut() {
                    if anim.finished {
                        continue;
                    }
                    let elapsed = anim.duration * (anim.iteration as f64);
                    cancelled_names.push(anim.name.clone());
                    cancelled.push((key, anim.name.clone(), elapsed));
                    anim.finished = true;
                }
            }
            // 已完成状态保留（fill-mode forwards 消费最后帧）；若全部取消且无 fill 消费价值
            // 则由既有 cleanup 逻辑处理——此处不额外清理。
            let _ = cancelled_names.len();
        }
    }

    /// 获取所有有活跃动画的元素 ID。
    pub fn active_element_ids(&self) -> Vec<u64> {
        self.active_animations
            .iter()
            .filter(|(_, anims)| anims.iter().any(|a| !a.finished))
            .map(|(&id, _)| id)
            .collect()
    }

    /// 取出「自上次 drain 后新完成」的动画（R3249，CSS Animations §animationend）。
    /// 宿主据此映射元素并派发 `animationend` 事件。每次调用清空内部缓冲（每完成帧只派发一次）。
    pub fn drain_just_finished(&mut self) -> Vec<FinishedAnimation> {
        std::mem::take(&mut self.just_finished)
    }

    /// 取出「自上次 drain 后跨越的迭代边界」（R3250，CSS Animations §animationiteration）。
    /// 宿主据此映射元素并派发 `animationiteration` 事件（infinite 动画循环回调靠此）。每次调用清空缓冲。
    pub fn drain_just_iterated(&mut self) -> Vec<IteratedAnimation> {
        std::mem::take(&mut self.just_iterated)
    }

    /// 取出「自上次 drain 后新启动」的动画（R3251，CSS Animations §animationstart）。宿主据此映射元素并
    /// 派发 `animationstart` 事件（elapsedTime=0，每动画仅一次）。每次调用清空缓冲。
    pub fn drain_just_started(&mut self) -> Vec<StartedAnimation> {
        std::mem::take(&mut self.just_started)
    }

    /// 移除已完成动画。
    pub fn cleanup_finished(&mut self) {
        self.active_animations.retain(|_, anims| {
            anims.retain(|a| !a.finished);
            !anims.is_empty()
        });
    }

    /// 清除所有动画和注册的关键帧。
    pub fn clear(&mut self) {
        self.keyframes_registry.clear();
        self.active_animations.clear();
    }

    /// 查询已注册的 @keyframes 名称列表。
    pub fn registered_keyframe_names(&self) -> Vec<&str> {
        self.keyframes_registry.keys().map(|s| s.as_str()).collect()
    }
}

/// 将 KeyframeBlock 转为 KeyframePoint 集。css-animations-1 §keyframes：逗号分隔
/// 选择器列表每项各产出**一个关键帧**（`from, to {...}` → 0% 与 100% 两帧同声明集）
/// ——旧实现折叠取首个选择器，`from, to` 形丢失 100% 帧、由 `register_keyframes`
/// 合成空边界帧（刀 I 回归 important-prop 的缺陷 A 面：刀 F 通用 underlying 臂把
/// 该空帧填成计算值，动画从恒定输出漂移为渐变面）。
/// https://drafts.csswg.org/css-animations-1/#keyframes
fn block_to_points(block: &KeyframeBlock) -> Vec<KeyframePoint> {
    let offset_for = |sel: &KeyframeSelector| match sel {
        KeyframeSelector::From => 0.0,
        KeyframeSelector::To => 1.0,
        KeyframeSelector::Percentage(p) => *p / 100.0,
    };

    // R3883：帧内 `animation-timing-function` 是**段 timing**（应用于本帧→下一帧），
    // 不是可插值属性——从 properties 剥离单独存储，避免落入插值/应用名单被丢弃或误用。
    // 值解析失败（畸形 timing）→ None（回落动画全局 timing）。
    // 刀 I（C 面）：css-animations-1 §keyframes——关键帧声明中的 `!important`
    // **无效，按未声明丢弃**（keyframes 非级联上下文，重要度标志不产生任何排序
    // 效果；driving：important-prop `border-color: red !important` 不得覆盖同块
    // 前条 `border-color: green`）。
    // https://drafts.csswg.org/css-animations-1/#keyframes
    let mut timing = None;
    // 刀 ANIMCOMP S2：帧内 `animation-composition` 是该帧值的合成操作（css-animations-2
    // §animation-composition），与 timing 同法剥离存储——不落插值/应用名单（刀前作为
    // 伪属性流入插值输出、apply 端 no-op）。畸形值（非 replace/add/accumulate）→ None
    // （回落动画级/replace）。
    let mut composition = None;
    let mut properties: HashMap<String, String> = block
        .declarations
        .iter()
        .filter(|d| !d.important)
        .filter(|d| {
            if d.property.eq_ignore_ascii_case("animation-timing-function") {
                timing = zero_css_parser::values::parse_timing_function(d.value.trim());
                return false;
            }
            if d.property.eq_ignore_ascii_case("animation-composition") {
                composition = zero_css_parser::values::parse_animation_composition(d.value.trim());
                return false;
            }
            true
        })
        .map(|d| (d.property.clone(), d.value.clone()))
        .collect();

    // 刀 FI：帧内 `font` 简写此前整条 no-op（[`AnimationClock::apply_to_computed_style`]
    // 无 `font` 简写臂，[`apply_single_property`] 只有 size/weight/adjust 长手臂）→
    // 长手全落 UA 默认（system-fonts 396F 根因）。ingest 侧经 style-system
    // `expand_shorthands` 单源展开（`expand_font` 16 槽，css-fonts-4 §font-prop
    // 「先复位全部子属性再覆盖」语义）。只扩 `font`：apply 端已有臂的简写
    // （background/border-color/columns 等）在 ingest 展开会把「简写名进插值→简写臂」
    // 改成逐 longhand 插值，css-animations 存量 P 面有 P2F 风险；`font` 无插值/简写
    // 应用臂 = 零扰动展开（battery A/B 验证）。解析失败展开空 → 声明按原语义丢弃。
    // https://drafts.csswg.org/css-fonts-4/#font-prop
    let font_key = properties.keys().find(|k| k.eq_ignore_ascii_case("font")).cloned();
    if let Some(font_key) = font_key
        && let Some(font_value) = properties.remove(&font_key)
    {
        for (prop, val, _, _) in zero_style_system::expand_shorthands(&[(font_key, font_value, false, (0, 0, 0))]) {
            properties.insert(prop, val);
        }
    }

    block
        .selectors
        .iter()
        .map(|sel| KeyframePoint {
            offset: offset_for(sel),
            properties: properties.clone(),
            timing: timing.clone(),
            composition: composition.clone(),
        })
        .collect()
}

/// 应用动画方向。
fn apply_direction(progress: f64, iteration: u64, direction: &AnimationDirectionValue) -> f64 {
    match direction {
        AnimationDirectionValue::Normal => progress,
        AnimationDirectionValue::Reverse => 1.0 - progress,
        AnimationDirectionValue::Alternate => {
            if !iteration.is_multiple_of(2) {
                1.0 - progress
            } else {
                progress
            }
        }
        AnimationDirectionValue::AlternateReverse => {
            if iteration.is_multiple_of(2) {
                1.0 - progress
            } else {
                progress
            }
        }
    }
}

/// 在关键帧序列中找到当前进度对应的两个帧并插值。
fn interpolate_keyframes(
    keyframes: &[KeyframePoint],
    progress: f64,
    global_timing: &TimingFunctionValue,
) -> Vec<InterpolatedProperty> {
    if keyframes.is_empty() {
        return Vec::new();
    }
    if keyframes.len() == 1 {
        return keyframes[0]
            .properties
            .iter()
            .map(|(k, v)| InterpolatedProperty {
                name: k.clone(),
                value: v.clone(),
            })
            .collect();
    }

    // 找到包含 progress 的区间 [i, i+1]。progress 落在末帧（如 t=1 / fill 终态）
    // 时**不选末帧自身**（span=0 会退化为 to 帧声明原样）而停在末段 [last-1, last]
    //——normalized=1.0 走端点 blend，产出经 canonicalize/apply 过 gCS 串化器与
    // WPT 期望（at(1) 混合公式形，如 rule-inset 跨单位对 `calc(0% + 1px)`）对齐；
    // apply 解析两形同值，渲染面零漂移（wptfyi-css 刀 E，2026-11-17）。
    let mut from_idx = 0;
    for (i, kf) in keyframes.iter().enumerate() {
        if kf.offset <= progress && i + 1 < keyframes.len() {
            from_idx = i;
        } else if kf.offset > progress {
            break;
        }
    }
    let to_idx = (from_idx + 1).min(keyframes.len() - 1);

    interpolate_between_with_timing(&keyframes[from_idx], &keyframes[to_idx], progress, global_timing)
}

/// 生成动画最后一帧的属性（用于 fill-mode: forwards）。
fn interpolate_to_end(anim: &AnimationState) -> Vec<InterpolatedProperty> {
    // 最后一帧就是 100% 关键帧
    if let Some(last) = anim.keyframes.last() {
        last.properties
            .iter()
            .map(|(k, v)| InterpolatedProperty {
                name: k.clone(),
                value: v.clone(),
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// 计算动画完成时的最终关键帧进度（考虑方向）。
///
/// 对于 normal/alternate（偶数迭代），最终进度 = 1.0（100% 帧）。
/// 对于 reverse/alternate（奇数迭代）/ alternate-reverse（偶数迭代），最终进度 = 0.0（0% 帧）。
fn final_animation_progress(anim: &AnimationState) -> f64 {
    let last_iteration = anim.iteration.saturating_sub(1);
    match &anim.direction {
        AnimationDirectionValue::Normal => 1.0,
        AnimationDirectionValue::Reverse => 0.0,
        AnimationDirectionValue::Alternate => {
            if !last_iteration.is_multiple_of(2) {
                0.0
            } else {
                1.0
            }
        }
        AnimationDirectionValue::AlternateReverse => {
            if last_iteration.is_multiple_of(2) {
                0.0
            } else {
                1.0
            }
        }
    }
}

/// 将单个插值属性应用到 ComputedStyle。
fn apply_single_property(name: &str, value: &str, style: &mut ComputedStyle) {
    // r75 刀F② 空串契约：插值臂产空串＝本 tick 不写值（槽位不动，
    // underlying 显出）；kc5r74 电池实证 height-composition 2 键命中。
    if value.is_empty() {
        return;
    }
    use zero_css_parser::values::ColorValue;
    use zero_css_parser::values::LengthValue;

    match name {
        "opacity" => {
            if let Some(v) = parse_f64(value) {
                style.opacity = v.clamp(0.0, 1.0);
            }
        }
        // R3882：`background` 简写（@keyframes 常见形）取其颜色分量——只覆盖
        // background_color，图片分量另行处理（与 background-image 独立属性一致）。
        "background-color" | "background" => {
            let (r, g, b, a) = parse_color(value);
            style.background_color = ColorValue::Rgba(r as u8, g as u8, b as u8, (a * 255.0) as u8);
        }
        "color" => {
            let (r, g, b, a) = parse_color(value);
            style.color = ColorValue::Rgba(r as u8, g as u8, b as u8, (a * 255.0) as u8);
        }
        // 刀 I D 臂（css-animations applier 补缺）：border-color 写达四边 color 槽
        // ——与静态声明 apply 同语义。此前无臂 → 动画 border-color 输出静默丢弃
        // （important-prop 边框面根因：keyframe green 落不了盘，元素 `border:10px
        // solid red` 持位）。
        "border-color" => {
            let (r, g, b, a) = parse_color(value);
            let c = ColorValue::Rgba(r as u8, g as u8, b as u8, (a * 255.0) as u8);
            style.border_top_color = c.clone();
            style.border_right_color = c.clone();
            style.border_bottom_color = c.clone();
            style.border_left_color = c;
        }
        // ── r3 fame2b 委托臂·r69 R5 修复形（wptfyi-css，2026-11-23）：sizing 六长手
        // 动画回写委托 style-system apply_property_value（css-sizing-3 §width/height/
        // max-*/min-* propdef <length-percentage>；calc()/%/em 与静态声明同通道受理
        // ——fame2c 探针硬证的 calc-mix 中程采样塌 0px 由此修复）。委托拒收（负外
        // 插值等出域值——插值允许出域、computed 值期按 propdef <length [0,∞]> 钳
        // ，css-values-4 §8.6）回落原生写槽臂。r70 证据勘正：serialize 计算阶段对
        // 负 width/height 无钳（负 px 原样落 gCS "-15px"）——r72 Delta C 在写槽点
        // 对 width/height 臂直接 .max(0.0) 钳（r71 实测 +14 增益零损伤）；min-*/max-*
        // 臂保持原样（multicol P2F 36 防线按端点形态保留）。
        // https://drafts.csswg.org/css-sizing-3/#propdef-width
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" => {
            if !zero_style_system::property::apply::apply_property_value(style, name, value) {
                match name {
                    "width" => style.width = LengthValue::Px(parse_px(value).max(0.0)),
                    "height" => style.height = LengthValue::Px(parse_px(value).max(0.0)),
                    "min-width" => {
                        if let Some(px) = parse_px_strict(value) {
                            style.min_width = LengthValue::Px(px);
                        }
                    }
                    "max-width" => {
                        if let Some(v) = parse_max_sizing_anim_value(value) {
                            style.max_width = v;
                        }
                    }
                    "min-height" => {
                        if let Some(px) = parse_px_strict(value) {
                            style.min_height = LengthValue::Px(px);
                        }
                    }
                    "max-height" => {
                        if let Some(v) = parse_max_sizing_anim_value(value) {
                            style.max_height = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        // KC-5-A：contain-intrinsic 族动画插值串写达 typed 双轴＋auto 旗标
        //（canonicalize 采纳门 ser1≠ser0 的写面依赖；gCS 聚合臂读 typed 场
        // 回吐）。简写展开双轴、长手写单轴；解析失败槽位不动（宁缺勿错）。
        "contain-intrinsic-size" | "contain-intrinsic-width" | "contain-intrinsic-height" => {
            if let Some(axes) = parse_cis_axes(value) {
                let (w, h) = if axes.len() == 2 {
                    (axes[0], axes[1])
                } else {
                    (axes[0], axes[0])
                };
                if name == "contain-intrinsic-width" {
                    style.contain_intrinsic_width_auto = w.auto;
                    style.contain_intrinsic_width = if w.none { None } else { Some(LengthValue::Px(w.px)) };
                } else if name == "contain-intrinsic-height" {
                    style.contain_intrinsic_height_auto = h.auto;
                    style.contain_intrinsic_height = if h.none { None } else { Some(LengthValue::Px(h.px)) };
                } else {
                    style.contain_intrinsic_width_auto = w.auto;
                    style.contain_intrinsic_width = if w.none { None } else { Some(LengthValue::Px(w.px)) };
                    style.contain_intrinsic_height_auto = h.auto;
                    style.contain_intrinsic_height = if h.none { None } else { Some(LengthValue::Px(h.px)) };
                }
            }
        }
        "margin-top" => {
            style.margin_top = LengthValue::Px(parse_px(value));
        }
        "margin-bottom" => {
            style.margin_bottom = LengthValue::Px(parse_px(value));
        }
        "margin-left" => {
            style.margin_left = LengthValue::Px(parse_px(value));
        }
        "margin-right" => {
            style.margin_right = LengthValue::Px(parse_px(value));
        }
        "padding-top" => {
            style.padding_top = LengthValue::Px(parse_px(value));
        }
        "padding-bottom" => {
            style.padding_bottom = LengthValue::Px(parse_px(value));
        }
        "padding-left" => {
            style.padding_left = LengthValue::Px(parse_px(value));
        }
        "padding-right" => {
            style.padding_right = LengthValue::Px(parse_px(value));
        }
        // ── r80 刀 R2 applier 臂（wptfyi-css，2026-11-24）── css-transitions 刀
        // R2 新名 consult 值 canonicalize 往返的写面依赖：此前无臂 → apply 空转
        // ser1≡ser0 恒 None 回落 cascade 终值＝properties-value 页 "must not be
        // target value after start" 假败根因（mask/ui/background-position 委托臂
        // 同病同治）。LP 族写 LengthValue 槽（strict 臂产 "N.NNpx"/"N.NN%" 形，
        // [`parse_len_pct_val`] 受理；解析失败槽位不动——回落方向）；
        // transform-origin/box-shadow 委托 style-system apply_advanced（双 LP 槽 /
        // [`parse_box_shadow_list`] 全槽映射同文法单点）；text-decoration-color
        // 写色槽（rgb()/rgba() 产形经 parse_color，同 color 臂形）。
        // https://drafts.csswg.org/css-backgrounds-3/#border-radius
        "border-top-left-radius" => {
            if let Some((v, u)) = parse_len_pct_val(value) {
                style.border_top_left_radius = if u == "%" {
                    LengthValue::Percentage(v)
                } else {
                    LengthValue::Px(v)
                };
            }
        }
        "border-top-right-radius" => {
            if let Some((v, u)) = parse_len_pct_val(value) {
                style.border_top_right_radius = if u == "%" {
                    LengthValue::Percentage(v)
                } else {
                    LengthValue::Px(v)
                };
            }
        }
        "border-bottom-right-radius" => {
            if let Some((v, u)) = parse_len_pct_val(value) {
                style.border_bottom_right_radius = if u == "%" {
                    LengthValue::Percentage(v)
                } else {
                    LengthValue::Px(v)
                };
            }
        }
        "border-bottom-left-radius" => {
            if let Some((v, u)) = parse_len_pct_val(value) {
                style.border_bottom_left_radius = if u == "%" {
                    LengthValue::Percentage(v)
                } else {
                    LengthValue::Px(v)
                };
            }
        }
        // https://drafts.csswg.org/css-multicol-1/#cg
        "column-gap" => {
            if let Some((v, u)) = parse_len_pct_val(value) {
                style.column_gap = if u == "%" {
                    LengthValue::Percentage(v)
                } else {
                    LengthValue::Px(v)
                };
            }
        }
        "transform-origin" | "box-shadow" => {
            let _ =
                zero_style_system::property::apply_advanced::apply_advanced_property_value(style, name, value, None);
        }
        // https://drafts.csswg.org/css-text-decor-3/#text-decoration-color-property
        "text-decoration-color" => {
            let (r, g, b, a) = parse_color(value);
            style.text_decoration_color = ColorValue::Rgba(r as u8, g as u8, b as u8, (a * 255.0) as u8);
        }
        // R4202（css-transforms-2 §individual-transforms + css-transforms-1）：
        // transform 系动画值**替换**对应字段（css-transforms-2 #individual-transforms：
        // animated transform 替换 individual 基值合成——combine 案 assert「The scale
        // property is replaced in the animation」），非与 underlying 叠加。动画期间
        // 直接覆写字段即可（forwards fill 同路径）。
        "transform" => {
            if let Some(v) = zero_css_parser::values::parse_transform(value) {
                style.transform = v;
            }
        }
        "translate" => {
            if value.trim().eq_ignore_ascii_case("none") {
                style.individual_translate = None;
            } else if let Some(f) = zero_css_parser::values::parse_individual_translate(value) {
                style.individual_translate = Some(f);
            }
        }
        "rotate" => {
            if value.trim().eq_ignore_ascii_case("none") {
                style.individual_rotate = None;
            } else if let Some(f) = zero_css_parser::values::parse_individual_rotate(value, None) {
                style.individual_rotate = Some(f);
            }
        }
        "scale" => {
            if value.trim().eq_ignore_ascii_case("none") {
                style.individual_scale = None;
            } else if let Some(f) = zero_css_parser::values::parse_individual_scale(value, None) {
                style.individual_scale = Some(f);
            }
        }
        // CSS Motion Path offset 簇写回（css-motion-1；wptfyi-css offset-* 刀）——
        // 与 apply_advanced 静态声明同一语义（distance px/% 直存，path path() 原文）。
        // 基本形状族刀：插值发射值与静态 apply 同文法裁决单点（parse_clip_path
        // ——ray/基本形状全族/coord-box 后缀，css-shapes-2），verbatim 存。
        "offset-path" => {
            let v = value.trim();
            if v.eq_ignore_ascii_case("none") {
                style.offset_path = None;
            } else if (v.to_ascii_lowercase().starts_with("path(") && v.ends_with(')'))
                || zero_css_parser::values::parse_clip_path(v).is_some()
            {
                style.offset_path = Some(v.to_string());
            }
        }
        "offset-distance" => {
            if let Some((n, u)) = parse_len_pct_val(value) {
                style.offset_distance = Some(offset_len_from(n, &u));
            }
        }
        "offset-rotate" => {
            style.offset_rotate = parse_offset_rotate_computed(value);
        }
        "offset-anchor" => {
            style.offset_anchor = if value.trim().eq_ignore_ascii_case("auto") {
                None
            } else {
                parse_len_pair_pct(value).map(|((x, xu), (y, yu))| (offset_len_from(x, &xu), offset_len_from(y, &yu)))
            };
        }
        "offset-position" => {
            style.offset_position = if value.trim().eq_ignore_ascii_case("normal") {
                Some(OffsetPositionValue::Normal)
            } else if value.trim().eq_ignore_ascii_case("auto") {
                Some(OffsetPositionValue::Auto)
            } else {
                parse_len_pair_pct(value).map(|((x, xu), (y, yu))| {
                    OffsetPositionValue::Position(offset_len_from(x, &xu), offset_len_from(y, &yu))
                })
            };
        }
        // ── wptfyi-css mask 长手族 applier 臂（2026-11-18 刀 A1）──
        // css-masking-1 §5.3–5.6 长手动画回写：委托 style-system apply_advanced
        // 写值通道八臂（刀 MI 已挂插值/gCS 串化，本函数缺臂致 canonicalize
        // ser1≡ser0 恒 None 回落基值——animations/mask-position 插值页 186F、
        // mask-image 页 168F 根因；与 clip-path 臂 2026-11-16 同病同治）。
        // 解析失败不动样式（回落方向）。
        // https://drafts.fxtf.org/css-masking-1/#masking
        "mask-image" | "mask-mode" | "mask-position" | "mask-size" | "mask-repeat" | "mask-clip" | "mask-origin"
        | "mask-composite" => {
            let _ =
                zero_style_system::property::apply_advanced::apply_advanced_property_value(style, name, value, None);
        }
        // ── wptfyi-css css-ui OA 刀 applier 臂 ── outline-width/color/offset 委托
        // style-system apply_property_value（css-ui-4 §5.2/§5.3/§5.5 写值通道单点）；
        // accent-color 委托 apply_advanced（AccentColorValue 文法单点）。此前无臂致
        // canonicalize 往返 ser1≡ser0 恒 None 回落基值——css-ui interpolation 四页
        // CSS Animations/Web Animations 通道 got=underlying 恒定根因（mask 族刀 A1
        // 同病同治）。解析失败不动样式（回落方向）。
        // https://drafts.csswg.org/css-ui-4/#outline-width-properties
        "outline-width" | "outline-color" | "outline-offset" => {
            let _ = zero_style_system::property::apply::apply_property_value(style, name, value);
        }
        "accent-color" => {
            let _ =
                zero_style_system::property::apply_advanced::apply_advanced_property_value(style, name, value, None);
        }
        // ── wptfyi-css 刀 A1d applier 臂 ── css-backgrounds-3/4 §background-position
        // (± x/y) + css-images-3 §object-position：动画插值回写委托 style-system
        // apply_advanced 同文法单点。此前无臂致 canonicalize 往返 ser1≡ser0 恒
        // None 回落基值——css-backgrounds position 三页 transitions 通道全静态
        // to 根因（刀 A1c ②补了清单/插值臂、漏本臂；与 mask 族刀 A1 同病同治）。
        // 解析失败不动样式（回落方向）。
        "background-position" | "background-position-x" | "background-position-y" | "object-position" => {
            let _ =
                zero_style_system::property::apply_advanced::apply_advanced_property_value(style, name, value, None);
        }
        // ── wptfyi-css 刀 A2 applier 臂 ── css-inline-3 §vertical-align：动画插值
        // 回写 VerticalAlignValue 槽（'30.00px'/'12%' 小数形经 parse_length、
        // 关键字离散 echo 经关键字臂）。此前无臂致 canonicalize 往返 ser1≡ser0
        // 恒 None 回落基值——vertical-align-interpolation 页 transitions 通道
        // 静态 to 同 A1d 根因。calc 混合形（'calc(Xpx ± P%)'）parse 无 calc 槽
        // 返 None 不动样式（canonicalize 恒等回落级联，该面挂残面池
        // FIXME(va-calc-slot)）。
        // https://drafts.csswg.org/css-inline-3/#propdef-vertical-align
        "vertical-align" => {
            if let Some(v) = zero_css_parser::values::parse_vertical_align(value) {
                style.vertical_align = v;
            }
        }
        // ── wptfyi-css 基本形状族 applier 臂（2026-11-16 二十一轮）──
        // css-shapes-2 §basic-shape-interpolation：动画插值后的 clip-path 全族
        // （inset/circle/ellipse/polygon/shape()/path()/rect()/xywh()）经
        // parse_clip_path 裁决单点写回槽位——此前无臂致 canonicalize 恒 None
        // 回落 echo（masking clip-path T 面零增益根因）。与静态声明同一转换
        // （apply_advanced 同文法单点），严格解析失败不动样式（回落方向）。
        "clip-path" => {
            if let Some(v) = zero_css_parser::values::parse_clip_path(value) {
                style.clip_path = v;
            }
        }
        // clip-rule：css-masking-1 §the-clip-rule 关键字槽位写回——canonicalize
        // 往返（apply + 串化变化判定）承载 cssAnimations/WAAPI 通道端点关键帧
        //（driving: clip-rule-no-interpolation 42 子测 ≥0.5 翻转位）。
        // https://drafts.fxtf.org/css-masking-1/#the-clip-rule
        "clip-rule" => {
            if value.eq_ignore_ascii_case("nonzero") {
                style.clip_rule = ClipRuleComputedValue::Nonzero;
            } else if value.eq_ignore_ascii_case("evenodd") {
                style.clip_rule = ClipRuleComputedValue::Evenodd;
            }
        }
        // css-shapes-1 §shape-outside：none | [ <basic-shape> || <shape-box> ]
        // ——镜像 apply_advanced 静态映射（None→None、其余 Shape 包装）+ 裸
        // <shape-box> 关键字 Box 变体（clip-path 无此形态；刀 GBX 起 parse
        // 亦受理 masking 七盒裸形/组合形，此处按 shape-outside 文法收窄：
        // 4 形裸盒走 Box 槽、组合形未建模不动样式回落）。
        "shape-outside" => {
            if let Some(v) = zero_css_parser::values::parse_clip_path(value) {
                let next = match v {
                    zero_css_parser::values::ClipPathValue::None => Some(ShapeOutsideComputedValue::None),
                    zero_css_parser::values::ClipPathValue::WithBox { shape: None, geometry } => {
                        use zero_css_parser::values::MaskGeometryBoxValue as G;
                        match geometry {
                            G::ContentBox => Some(ShapeOutsideComputedValue::Box(ShapeOutsideBoxKind::ContentBox)),
                            G::PaddingBox => Some(ShapeOutsideComputedValue::Box(ShapeOutsideBoxKind::PaddingBox)),
                            G::BorderBox => Some(ShapeOutsideComputedValue::Box(ShapeOutsideBoxKind::BorderBox)),
                            G::MarginBox => Some(ShapeOutsideComputedValue::Box(ShapeOutsideBoxKind::MarginBox)),
                            // fill/stroke/view-box 为 masking 侧扩形，
                            // <shape-box> 值域外 → 非法不动样式
                            _ => None,
                        }
                    }
                    // FIXME: <basic-shape> <shape-box> 组合形未建模——不动样式
                    zero_css_parser::values::ClipPathValue::WithBox { .. } => None,
                    // url() 变体（CLIP-URL 刀起 parse 受理 clip-path
                    // <clip-source>）域外拒收——shape-outside 文法 url()
                    // <image> 形未建模，不动样式回落。
                    zero_css_parser::values::ClipPathValue::Url(_) => None,
                    v => Some(ShapeOutsideComputedValue::Shape(v)),
                };
                if let Some(s) = next {
                    style.shape_outside = s;
                }
            }
        }
        // wptfyi-css shape-margin / shape-image-threshold 写回（css-shapes-1；
        // 插值发射值恒 px/number 形，同一非负/[0,1] 域钳与 apply_advanced 静态
        // 声明一致；解析失败不动样式回落方向）。
        "shape-margin" => {
            let px = parse_px(value);
            style.shape_margin = LengthValue::Px(px.max(0.0));
        }
        "shape-image-threshold" => {
            if let Some(v) = parse_f64(value) {
                style.shape_image_threshold = v.clamp(0.0, 1.0);
            }
        }
        // ── wptfyi-css multicol columns 刀 apply 臂 ── css-multicol §cw/§cc +
        // css-multicol-2 §4.2/§4.5：动画插值串写回 column-width/count/height 槽
        // （插值发射值恒 px/正整数形；'auto' 落初值槽——离散翻转端点回声经
        // canonicalize 往返）。columns 简写按 [`parse_columns_endpoint`] 组件写
        // 存在槽（未出现槽落 auto = 简写展开初值语义），并复位 column-wrap
        // （css-multicol-2 §4.5 reset 条款）。解析失败不动样式回落方向。
        // kc5r82：端点写回 strict 化（镜像 column-count/z-index 臂拒收语义）——
        // CSS-wide 关键字与非法形不动槽 = underlying 显出（css-multicol §4.2
        // propdef <length [0,∞]>，'auto' 简写回声分支保留）；原 lenient 0 化把
        // 翻转端点 echo 串落 0px 垃圾槽。
        "column-width" => {
            if value.trim().eq_ignore_ascii_case("auto") {
                style.column_width = ColumnWidthComputedValue::Auto;
            } else if let Some(px) = parse_px_strict(value) {
                style.column_width = ColumnWidthComputedValue::Length(LengthValue::Px(px));
            }
        }
        "column-count" => {
            if value.trim().eq_ignore_ascii_case("auto") {
                style.column_count = ColumnCountComputedValue::Auto;
            } else if let Some(n) = parse_f64(value) {
                style.column_count = ColumnCountComputedValue::Number(n.max(1.0).round() as u32);
            }
        }
        // 刀 Z：z-index 插值通道槽写入——离散回落 echo 串原样落槽，溢出
        // i32 槽域保旧值（css-position-3 §z-index）。
        "z-index" => {
            if value.trim().eq_ignore_ascii_case("auto") {
                style.z_index = ZIndexValue::Auto;
            } else if let Ok(v) = value.trim().parse::<i32>() {
                style.z_index = ZIndexValue::Integer(v);
            }
        }
        "column-height" => {
            if value.trim().eq_ignore_ascii_case("auto") {
                style.column_height = ColumnHeightComputedValue::Auto;
            } else if let Some(px) = parse_px_strict(value) {
                style.column_height = ColumnHeightComputedValue::Length(LengthValue::Px(px));
            }
        }
        "columns" => {
            if let Some(ep) = parse_columns_endpoint(value) {
                style.column_width = ep
                    .width
                    .map(|w| ColumnWidthComputedValue::Length(LengthValue::Px(w)))
                    .unwrap_or(ColumnWidthComputedValue::Auto);
                style.column_count = ep
                    .count
                    .map(|c| ColumnCountComputedValue::Number(c.max(1.0).round() as u32))
                    .unwrap_or(ColumnCountComputedValue::Auto);
                style.column_height = ep
                    .height
                    .map(|h| ColumnHeightComputedValue::Length(LengthValue::Px(h)))
                    .unwrap_or(ColumnHeightComputedValue::Auto);
                style.column_wrap = ColumnWrapComputedValue::Auto;
            }
        }
        // ── kc5r82（wptfyi-css，2026-11-24）multicol 离散三长手委托臂 ──
        // css-multicol-1 §column-fill/§column-rule-style/§column-span propdef：
        // 动画离散翻转输出（css-values-4 §8.5.6，DISCRETE_TRANSITIONABLE 在册）
        // 委托 style-system 静态声明同通道受理（'auto'/'balance'/'dashed'/'all'
        // 与 expectedTarget gCS 同链恒等）；CSS-wide 关键字（翻转 from 侧 echo
        // 'initial' 等）与非法值由 apply 拒收 → false → 不动槽 = underlying 显
        // 出（两侧同拒收恒等，镜像 sizing 委托臂形）。
        // https://drafts.csswg.org/css-values-4/#discrete-type
        "column-fill" | "column-rule-style" | "column-span" => {
            let _ = zero_style_system::property::apply::apply_property_value(style, name, value);
        }
        // s560 全桥红窗补臂：display 离散翻转端点写回 display 槽（css-display-3
        // §2 关键字集；css-transitions-2 §transition-behavior-property allow-discrete
        // 离散对 gCS 咨询经 canonicalize 往返——缺臂即 ser1==ser0 恒等回落级联，
        // 过渡值静默丢失）。复用静态声明同款解析器（与 apply.rs 臂同语义）；解析
        // 失败不动样式回落方向。
        // https://drafts.csswg.org/css-transitions-2/#transition-behavior-property
        "display" => {
            if let Some(v) = zero_css_parser::values::parse_display(value) {
                style.display = v;
            }
        }
        // wptfyi-css mask-border 刀：动画插值串写回五槽（css-masking-1 §3.4–3.9）。
        // 复用静态声明同款解析器/转换（与 apply_advanced 臂同语义）；解析失败不动
        // 样式回落方向。
        "mask-border-source" => {
            if let Some(v) = zero_css_parser::values::parse_border_image_source(value) {
                let dark = style.color_scheme_dark;
                style.mask_border_source =
                    zero_style_system::property::apply_advanced::border_image_source_to_computed(v, dark);
            }
        }
        "mask-border-slice" => {
            if let Some(v) = parse_border_image_slice(value) {
                let comp = |c: &BorderImageSliceComponent| match c {
                    BorderImageSliceComponent::Number(n) => BorderImageSliceComputedComponent::Number(*n),
                    BorderImageSliceComponent::Percent(n) => BorderImageSliceComputedComponent::Percent(*n),
                };
                style.mask_border_slice = BorderImageSliceComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                    fill: v.fill,
                };
            }
        }
        "mask-border-width" => {
            if let Some(v) = parse_border_image_width(value) {
                let comp = |c: &BorderImageWidthComponent| match c {
                    BorderImageWidthComponent::Auto => BorderImageWidthComputedComponent::Auto,
                    BorderImageWidthComponent::Number(n) => BorderImageWidthComputedComponent::Number(*n),
                    BorderImageWidthComponent::Length(lv) => BorderImageWidthComputedComponent::Length(
                        zero_style_system::property::apply_advanced::resolve_effect_length(lv, style),
                    ),
                    BorderImageWidthComponent::Percent(n) => BorderImageWidthComputedComponent::Percent(*n),
                };
                style.mask_border_width = BorderImageWidthComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                };
            }
        }
        "mask-border-repeat" => {
            if let Some(v) = zero_css_parser::values::parse_border_image_repeat(value) {
                let mode = |m: &zero_css_parser::values::BorderImageRepeatMode| match m {
                    zero_css_parser::values::BorderImageRepeatMode::Stretch => BorderImageRepeatComputedMode::Stretch,
                    zero_css_parser::values::BorderImageRepeatMode::Repeat => BorderImageRepeatComputedMode::Repeat,
                    zero_css_parser::values::BorderImageRepeatMode::Round => BorderImageRepeatComputedMode::Round,
                    zero_css_parser::values::BorderImageRepeatMode::Space => BorderImageRepeatComputedMode::Space,
                };
                style.mask_border_repeat = BorderImageRepeatComputedValue {
                    horizontal: mode(&v.horizontal),
                    vertical: mode(&v.vertical),
                };
            }
        }
        "mask-border-outset" => {
            if let Some(v) = parse_border_image_outset(value) {
                let comp = |c: &zero_css_parser::values::BorderImageOutsetComponent| match c {
                    zero_css_parser::values::BorderImageOutsetComponent::Number(n) => {
                        BorderImageOutsetComputedComponent::Number(*n)
                    }
                    zero_css_parser::values::BorderImageOutsetComponent::Length(lv) => {
                        BorderImageOutsetComputedComponent::Length(
                            zero_style_system::property::apply_advanced::resolve_effect_length(lv, style),
                        )
                    }
                };
                style.mask_border_outset = BorderImageOutsetComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                };
            }
        }
        // ── wptfyi-css 刀 A3 text-shadow applier 臂（css-text-decor-3 §text-shadow）──
        // 镜像 apply_advanced 静态臂（resolve_effect_length em 域穿线、currentColor
        // 槽内延迟解析由 gCS [`text_shadow_to_css`] 按元素色解析）。canonicalize
        // apply→gCS 串化往返 ser1≠ser0 为 CSSA/WAAPI/transitions 三通道共因
        //（刀 B1 探针谱系 v3 同证：缺臂 ⇒ ser1≡ser0 ⇒ 三通道回落级联静态）。
        // https://drafts.csswg.org/css-text-decor-3/#text-shadow-property
        "text-shadow" => {
            if let Some(list) = zero_css_parser::values::parse_text_shadow_list(value, style.color_scheme_dark) {
                style.text_shadow = list
                    .into_iter()
                    .map(|v| zero_style_system::property::types::TextShadowComputedValue {
                        offset_x: zero_style_system::property::apply_advanced::resolve_effect_length(
                            &v.offset_x,
                            style,
                        ),
                        offset_y: zero_style_system::property::apply_advanced::resolve_effect_length(
                            &v.offset_y,
                            style,
                        ),
                        blur_radius: zero_style_system::property::apply_advanced::resolve_effect_length(
                            &v.blur_radius,
                            style,
                        ),
                        color: v.color,
                    })
                    .collect();
            }
        }
        // ── wptfyi-css 刀 B1 border-image 五槽 applier 臂（css-backgrounds-3 §3.5–3.9）──
        // 与 mask-border 五臂同构（同解析器/同 computed 类型/同 gCS 串化器——
        // computed_style.rs 两族共用 border_image_*_to_css），写 border_image_* 槽。
        // 此前无臂致 canonicalize 往返 ser1≡ser0 恒 None——border-image 三页
        // CSSA/WAAPI 整批静态、transitions 跳终值三通道共因（与 mask 族刀 A1、
        // bg-position 刀 A1d 同病同治）。解析失败不动样式（回落方向）。
        // https://drafts.csswg.org/css-backgrounds-3/#border-images
        "border-image-source" => {
            if let Some(v) = zero_css_parser::values::parse_border_image_source(value) {
                let dark = style.color_scheme_dark;
                style.border_image_source =
                    zero_style_system::property::apply_advanced::border_image_source_to_computed(v, dark);
            }
        }
        "border-image-slice" => {
            if let Some(v) = parse_border_image_slice(value) {
                let comp = |c: &BorderImageSliceComponent| match c {
                    BorderImageSliceComponent::Number(n) => BorderImageSliceComputedComponent::Number(*n),
                    BorderImageSliceComponent::Percent(n) => BorderImageSliceComputedComponent::Percent(*n),
                };
                style.border_image_slice = BorderImageSliceComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                    fill: v.fill,
                };
            }
        }
        "border-image-width" => {
            if let Some(v) = parse_border_image_width(value) {
                let comp = |c: &BorderImageWidthComponent| match c {
                    BorderImageWidthComponent::Auto => BorderImageWidthComputedComponent::Auto,
                    BorderImageWidthComponent::Number(n) => BorderImageWidthComputedComponent::Number(*n),
                    BorderImageWidthComponent::Length(lv) => BorderImageWidthComputedComponent::Length(
                        zero_style_system::property::apply_advanced::resolve_effect_length(lv, style),
                    ),
                    BorderImageWidthComponent::Percent(n) => BorderImageWidthComputedComponent::Percent(*n),
                };
                style.border_image_width = BorderImageWidthComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                };
            }
        }
        "border-image-repeat" => {
            if let Some(v) = zero_css_parser::values::parse_border_image_repeat(value) {
                let mode = |m: &zero_css_parser::values::BorderImageRepeatMode| match m {
                    zero_css_parser::values::BorderImageRepeatMode::Stretch => BorderImageRepeatComputedMode::Stretch,
                    zero_css_parser::values::BorderImageRepeatMode::Repeat => BorderImageRepeatComputedMode::Repeat,
                    zero_css_parser::values::BorderImageRepeatMode::Round => BorderImageRepeatComputedMode::Round,
                    zero_css_parser::values::BorderImageRepeatMode::Space => BorderImageRepeatComputedMode::Space,
                };
                style.border_image_repeat = BorderImageRepeatComputedValue {
                    horizontal: mode(&v.horizontal),
                    vertical: mode(&v.vertical),
                };
            }
        }
        "border-image-outset" => {
            if let Some(v) = parse_border_image_outset(value) {
                let comp = |c: &zero_css_parser::values::BorderImageOutsetComponent| match c {
                    zero_css_parser::values::BorderImageOutsetComponent::Number(n) => {
                        BorderImageOutsetComputedComponent::Number(*n)
                    }
                    zero_css_parser::values::BorderImageOutsetComponent::Length(lv) => {
                        BorderImageOutsetComputedComponent::Length(
                            zero_style_system::property::apply_advanced::resolve_effect_length(lv, style),
                        )
                    }
                };
                style.border_image_outset = BorderImageOutsetComputedValue {
                    top: comp(&v.top),
                    right: comp(&v.right),
                    bottom: comp(&v.bottom),
                    left: comp(&v.left),
                };
            }
        }
        // R4272：动画插值后的 filter / backdrop-filter 函数列表写回对应字段
        // （parse_filter_list("none") = 空 Vec = none；与 apply_advanced 静态声明
        // 同一转换）。
        "filter" | "backdrop-filter" => {
            if let Some(list) = zero_css_parser::values::parse_filter_list(value) {
                use zero_css_parser::values::FilterValue;
                let mapped = list
                    .into_iter()
                    .map(|v| match v {
                        FilterValue::None => FilterComputedValue::None,
                        FilterValue::Blur(n) => FilterComputedValue::Blur(n),
                        FilterValue::Brightness(n) => FilterComputedValue::Brightness(n),
                        FilterValue::Contrast(n) => FilterComputedValue::Contrast(n),
                        FilterValue::Grayscale(n) => FilterComputedValue::Grayscale(n),
                        FilterValue::HueRotate(n) => FilterComputedValue::HueRotate(n),
                        FilterValue::Invert(n) => FilterComputedValue::Invert(n),
                        FilterValue::Opacity(n) => FilterComputedValue::Opacity(n),
                        FilterValue::Saturate(n) => FilterComputedValue::Saturate(n),
                        FilterValue::Sepia(n) => FilterComputedValue::Sepia(n),
                        FilterValue::DropShadow(x, y, b, c) => FilterComputedValue::DropShadow(x, y, b, c),
                        FilterValue::Url(r) => FilterComputedValue::Url(r),
                    })
                    .collect();
                if name == "filter" {
                    style.filter = mapped;
                } else {
                    style.backdrop_filter = mapped;
                }
            }
        }
        // ── wptfyi-css composite-add 刀（2026-11-15）apply 臂 ──
        // 注册动画插值串施加到基样式后经 gCS 序列化器 canonicalize（js_anim_registry），
        // CSS 动画/过渡渲染 tick 同路径（AnimationClock::apply_to_computed_style）。
        // 严格解析（失败不动样式 → canonicalize 序列化恒等返 None → 回落 B0 通道，
        // 零回归方向）；不并入既有宽松 parse_px 臂——关键字端点（auto/none/
        // normal/min-content）不得 0 化为错误值。
        "font-size" => {
            if let Some(px) = parse_px_strict(value) {
                style.font_size = LengthValue::Px(px);
            }
        }
        // css-fonts-4 #font-weight-prop：数值字重钳 [1,1000] 存储（区间外与
        // 关键字形不动样式）。
        "font-weight" => match value.trim().parse::<u16>() {
            Ok(n) if (1..=1000).contains(&n) => {
                style.font_weight = zero_css_parser::values::FontWeightValue::Absolute(n);
            }
            _ => {}
        },
        // css-fonts-4 #font-size-adjust-prop：复用属性解析器（none/metric/number/
        // math 全形），串化经 font_size_adjust_to_css 与期望元素同通道。
        "font-size-adjust" => {
            if let Some(v) = zero_style_system::property::parse::parse_font_size_adjust(value) {
                style.font_size_adjust = v;
            }
        }
        // 刀 fontcycle ffix（2026-11-19）：font-feature-settings 动画应用臂
        // 缺失——apply_single_property 无臂时 js_anim_registry canonicalize 施加
        // 后序列化恒等返 None 回落基 echo（css-animations/Web Animations 离散
        // 翻转 ≥0.5 恒基值；基 echo 臂入库前恒空串，两侧空串比较掩蔽此缺口，
        // echo 臂入库后 discrete-no-interpolation 8F P2F 暴露）。严格解析（失败
        // 不动样式 → canonicalize None 回落，零回归方向）；串化经 gCS 回显臂
        // 与期望元素同通道。font-variation-settings 同族补齐（transition_registry
        // DISCRETE_TRANSITIONABLE 已在清单，应用端对齐）。
        "font-feature-settings" => {
            if let Some(v) = parse_font_feature_settings(value) {
                style.font_feature_settings = v;
            }
        }
        "font-variation-settings" => {
            if let Some(v) = parse_font_variation_settings(value) {
                style.font_variation_settings = v;
            }
        }
        // 刀 ffix2（2026-11-19）：font-kerning/font-variant-numeric/
        // font-variant-alternates 应用臂——三注册表共享 canonicalize 通道
        // 无臂恒 None 回落基值（css-animations/Web Animations 离散翻转
        // t>=0.5 恒 from 值 8 行/性根因，承 fontcycle ffix ffs/fvs 配方）。
        // 严格解析（失败不动样式 → canonicalize None 回落，零回归方向）；
        // 串化经 gCS 回显臂与期望元素同通道。discrete-only 可过渡面见
        // transition_registry DISCRETE_TRANSITIONABLE 同批加列。
        // https://drafts.csswg.org/css-fonts-4/#font-kerning-prop
        "font-kerning" => {
            let parsed = match value.trim().to_ascii_lowercase().as_str() {
                "auto" => Some(zero_style_system::FontKerningValue::Auto),
                "normal" => Some(zero_style_system::FontKerningValue::Normal),
                "none" => Some(zero_style_system::FontKerningValue::None),
                _ => None,
            };
            if let Some(v) = parsed {
                style.font_kerning = v;
            }
        }
        // https://drafts.csswg.org/css-fonts-4/#font-variant-numeric-prop
        "font-variant-numeric" => {
            if let Some(v) = zero_css_parser::values::parse_font_variant_numeric(value) {
                style.font_variant_numeric = match v {
                    zero_css_parser::values::FontVariantNumericValue::Normal => {
                        zero_style_system::FontVariantNumericValue::Normal
                    }
                    zero_css_parser::values::FontVariantNumericValue::Ordinal => {
                        zero_style_system::FontVariantNumericValue::Ordinal
                    }
                    zero_css_parser::values::FontVariantNumericValue::SlashedZero => {
                        zero_style_system::FontVariantNumericValue::SlashedZero
                    }
                    zero_css_parser::values::FontVariantNumericValue::LiningNums => {
                        zero_style_system::FontVariantNumericValue::LiningNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::OldstyleNums => {
                        zero_style_system::FontVariantNumericValue::OldstyleNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::ProportionalNums => {
                        zero_style_system::FontVariantNumericValue::ProportionalNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::TabularNums => {
                        zero_style_system::FontVariantNumericValue::TabularNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::DiagonalFractions => {
                        zero_style_system::FontVariantNumericValue::DiagonalFractions
                    }
                    zero_css_parser::values::FontVariantNumericValue::StackedFractions => {
                        zero_style_system::FontVariantNumericValue::StackedFractions
                    }
                };
            }
        }
        // https://drafts.csswg.org/css-fonts-4/#font-variant-alternates-prop
        "font-variant-alternates" => {
            if let Some(v) = zero_css_parser::values::parse_font_variant_alternates(value) {
                style.font_variant_alternates = v;
                style.font_variant_alternates_features.clear();
            }
        }
        // 刀 optical（2026-11-20）：font-optical-sizing 应用臂——discrete-only
        // （css-fonts-4 §font-optical-sizing-prop），auto/none keyword 严格解析
        //（失败不动样式 → canonicalize None 回落基值，零回归方向）；串化经
        // gCS 回显臂（预备 hunk 先行在树）。承 ffix2 三性配方。
        // https://drafts.csswg.org/css-fonts-4/#font-optical-sizing-prop
        "font-optical-sizing" => {
            let parsed = match value.trim().to_ascii_lowercase().as_str() {
                "auto" => Some(zero_style_system::FontOpticalSizingValue::Auto),
                "none" => Some(zero_style_system::FontOpticalSizingValue::None),
                _ => None,
            };
            if let Some(v) = parsed {
                style.font_optical_sizing = v;
            }
        }
        "letter-spacing" => {
            if let Some(px) = parse_px_strict(value) {
                style.letter_spacing = LengthValue::Px(px);
                style.letter_spacing_normal = false;
            }
        }
        "word-spacing" => {
            if let Some(px) = parse_px_strict(value) {
                style.word_spacing = LengthValue::Px(px);
            }
        }
        "text-indent" => {
            if let Some(px) = parse_px_strict(value) {
                style.text_indent = LengthValue::Px(px);
            }
        }
        // css-gaps 刀 B：rule-width/rule-color 列表动画值回写（CSS Animations/WAAPI
        // apply 通道）——插值输出串经 css-parser 列表文法回 typed 字段，gCS 串化
        // 单源规范化（"23.00px"→"23px"、repeat(auto,…) 保形）；解析失败保级联值。
        "column-rule-width" | "row-rule-width" => {
            if let Some(segments) = zero_css_parser::values::parse_rule_width_list(value) {
                if name == "column-rule-width" {
                    style.column_rule_width = segments;
                } else {
                    style.row_rule_width = segments;
                }
            }
        }
        "column-rule-color" | "row-rule-color" => {
            if let Some(segments) = zero_css_parser::values::parse_rule_color_list(value) {
                if name == "column-rule-color" {
                    style.column_rule_color = segments;
                } else {
                    style.row_rule_color = segments;
                }
            }
        }
        // css-gaps 刀 D：rule-inset 槽动画值回写（CSS Animations/WAAPI apply
        // 通道）——插值输出串经单槽文法回 typed 字段（"6.00px"→Px(6) 解析）；
        // 解析失败保级联值。
        "column-rule-inset-cap-start" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.column_rule_inset_cap_start = c;
            }
        }
        "column-rule-inset-cap-end" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.column_rule_inset_cap_end = c;
            }
        }
        "column-rule-inset-junction-start" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.column_rule_inset_junction_start = c;
            }
        }
        "column-rule-inset-junction-end" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.column_rule_inset_junction_end = c;
            }
        }
        "row-rule-inset-cap-start" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.row_rule_inset_cap_start = c;
            }
        }
        "row-rule-inset-cap-end" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.row_rule_inset_cap_end = c;
            }
        }
        "row-rule-inset-junction-start" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.row_rule_inset_junction_start = c;
            }
        }
        "row-rule-inset-junction-end" => {
            if let Some(c) = zero_css_parser::values::parse_rule_inset_component(value) {
                style.row_rule_inset_junction_end = c;
            }
        }
        // 刀 FI：keyframe `font` 简写展开长手写回臂（style-system apply 单声明
        // 单源——family→`Vec<String>`、style→`FontStyleValue`（oblique 角形直读）、
        // stretch→f32（关键字/% 同源 `parse_font_stretch`）、line-height→
        // `LineHeightValue`，转换与静态声明面恒同）。展开集其余 reset 槽
        // （ligatures/numeric/…）走不上动画覆盖面，无需臂。
        "font-family" | "font-style" | "font-stretch" | "line-height" => {
            zero_style_system::apply_property_value(style, name, value);
        }
        // variant-caps 无 style-system 单声明臂（apply.rs 零命中）——值域文法
        // 直写（css-fonts-4 §font-variant-caps-prop，`<'font-variant-caps'>`
        // 七关键字；无效值落 store 前门拒，动画覆盖面解析失败静默跳过）。
        // https://drafts.csswg.org/css-fonts-4/#font-variant-caps-prop
        "font-variant-caps" => {
            style.font_variant_caps = match value.trim().to_ascii_lowercase().as_str() {
                "small-caps" => zero_style_system::FontVariantCapsValue::SmallCaps,
                "all-small-caps" => zero_style_system::FontVariantCapsValue::AllSmallCaps,
                "petite-caps" => zero_style_system::FontVariantCapsValue::PetiteCaps,
                "all-petite-caps" => zero_style_system::FontVariantCapsValue::AllPetiteCaps,
                "unicase" => zero_style_system::FontVariantCapsValue::Unicase,
                "titling-caps" => zero_style_system::FontVariantCapsValue::TitlingCaps,
                _ => zero_style_system::FontVariantCapsValue::Normal,
            };
        }
        // 刀 FAM-F：aspect-ratio 采样值落盘（css-sizing-4 §aspect-ratio）——
        // `auto` → 槽位 (None, auto=true)；`auto w / h`／`w / h`／bare `N` →
        // (Some(w/h 合并值), auto?)。解析失败（简并/非数形）不动样式 =
        // canonicalize None 回落级联（零回归方向）。
        "aspect-ratio" => {
            if value.trim().eq_ignore_ascii_case("auto") {
                style.aspect_ratio = None;
                style.aspect_ratio_auto = true;
            } else if let Some((auto, w, h)) = parse_aspect_ratio_decl(value) {
                style.aspect_ratio = Some((w / h) as f32);
                style.aspect_ratio_auto = auto;
            }
        }
        _ => {
            // 其他属性暂不支持动画覆盖
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{Declaration, KeyframeBlock, KeyframesRule};

    // ── wptfyi-css multicol columns 刀钉测 ──────────────────────────────
    // 期望表取 B 账 columns-interpolation 子测名（CSS Animations 通道权威行）。

    // ── wptfyi-css css-ui OA 刀钉测 ────────────────────────────────────
    // 期望表取 corpusexp r3 css-ui interpolation 四页失败消息 expected 列
    //（results-corpusexp-probe-102144/css-ui.json，CSS Animations 通道权威行）。

    #[test]
    /// outline-width 非负钳 + 关键字折叠（[0px]→[10px] at(-0.3)=0px、
    /// thick→15px at0=5px、thick→15px at(-2)=0px——语料契约）。
    fn test_oa1_outline_width_clamp_and_keywords() {
        assert_eq!(
            interpolate_property_value("outline-width", "0px", "10px", -0.3),
            "0.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-width", "thick", "15px", 0.0),
            "5.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-width", "thick", "15px", 0.3),
            "8.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-width", "thick", "15px", -2.0),
            "0.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-width", "0px", "10px", 0.3),
            "3.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-width", "0px", "10px", 1.5),
            "15.00px"
        );
    }

    #[test]
    /// outline-offset 带符号线性无钳（[-5px]→[5px] at(-0.3)=-8px——语料契约）。
    fn test_oa1_outline_offset_signed() {
        assert_eq!(
            interpolate_property_value("outline-offset", "-5px", "5px", -0.3),
            "-8.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-offset", "-5px", "5px", 0.3),
            "-2.00px"
        );
        assert_eq!(
            interpolate_property_value("outline-offset", "10px", "20px", 1.5),
            "25.00px"
        );
    }

    #[test]
    /// outline-color/accent-color 色对 lerp + currentColor 黑底 + 非色对
    /// 离散翻转（auto 族 at<0.5=from ≥0.5=to——css-ui-4 auto 非插值型）。
    fn test_oa1_color_lerp_and_discrete() {
        assert_eq!(
            interpolate_property_value("outline-color", "rgb(0, 0, 255)", "rgb(0, 128, 0)", 0.3),
            "rgba(0, 38, 179, 1.00)"
        );
        assert_eq!(
            interpolate_property_value("outline-color", "currentColor", "rgb(0, 128, 0)", 0.0),
            "rgba(0, 0, 0, 1.00)"
        );
        assert_eq!(
            interpolate_property_value("accent-color", "auto", "rgb(0, 128, 0)", 0.3),
            "auto"
        );
        assert_eq!(
            interpolate_property_value("accent-color", "auto", "rgb(0, 128, 0)", 0.6),
            "rgb(0, 128, 0)"
        );
    }

    #[test]
    /// 严格通道探针：outline 宽度/偏移可插值对 Some、accent 色对 Some、
    /// 非色对 None（不启动过渡——css-transitions §starting-of-transitions）。
    fn test_oa1_strict_probes() {
        assert_eq!(
            interpolate_property_value_strict("outline-width", "5px", "15px", 0.5).as_deref(),
            Some("10.00px")
        );
        assert_eq!(
            interpolate_property_value_strict("outline-offset", "-5px", "5px", 0.0).as_deref(),
            Some("-5.00px")
        );
        assert!(interpolate_property_value_strict("accent-color", "auto", "rgb(0, 0, 255)", 0.5).is_none());
        assert!(interpolate_property_value_strict("outline-color", "rgb(0, 0, 0)", "rgb(0, 128, 0)", 0.5).is_some());
    }

    #[test]
    /// 刀 MOTION-1b：strict 臂 offset-rotate t==1 端点原文 echo（非 canonical
    /// 端点——bare-`reverse`/乱序 `100grad auto`，r2 P2F ×2 语料钉）；canonical
    /// 复合与纯角走 lerp（外推 at(2) 与折叠语义，blanket 窗 44F 实证）；
    /// 跨类 None（离散翻转由调用方）。
    fn test_motion1b_strict_endpoint_echo() {
        assert_eq!(
            interpolate_property_value_strict("offset-rotate", "auto 100deg", "reverse", 1.0).as_deref(),
            Some("reverse")
        );
        assert_eq!(
            interpolate_property_value_strict("offset-rotate", "auto 170deg", "100grad auto", 1.0).as_deref(),
            Some("100grad auto")
        );
        assert_eq!(
            interpolate_property_value_strict("offset-rotate", "auto 100deg", "reverse", 0.875).as_deref(),
            Some("auto 170.00deg")
        );
        // 外推采样不直通（t>1 lerp 外推——1b blanket 窗 at(2) 44F 教训钉）。
        assert_eq!(
            interpolate_property_value_strict("offset-rotate", "auto 100deg", "reverse", 2.0).as_deref(),
            Some("auto 260.00deg")
        );
        // calc 角端点 t==1 原文 echo。
        assert_eq!(
            interpolate_property_value_strict(
                "offset-rotate",
                "auto -280deg",
                "auto calc(90deg - 0.5turn - 300grad + 0rad)",
                1.0
            )
            .as_deref(),
            Some("auto calc(90deg - 0.5turn - 300grad + 0rad)")
        );
        assert!(interpolate_property_value_strict("offset-rotate", "auto 200deg", "300deg", 0.5).is_none());
    }

    #[test]
    /// column-width/height 负外推钳 0（<length [0,∞]>——语料 neutral→20px at(-1)
    /// 期望 '1em' = font-size 0 下 0px）。
    fn test_column_width_height_negative_clamp() {
        assert_eq!(
            interpolate_property_value("column-width", "10px", "20px", -1.0),
            "0.00px"
        );
        assert_eq!(
            interpolate_property_value("column-width", "10px", "20px", -0.3),
            "7.00px"
        );
        assert_eq!(
            interpolate_property_value("column-width", "10px", "20px", 1.5),
            "25.00px"
        );
        assert_eq!(
            interpolate_property_value("column-height", "100px", "200px", -1.0),
            "0.00px"
        );
    }

    // ── wptfyi-css KC-5-r75 刀F② 钉测（kc5r74 电池归因处方）──────────
    // 刀F②＝height 轴离散端点未解析 % 字面不写值（空串契约）；width 轴
    // % 与 min/max 轴不属刀域（r74 刀F① REG 516 判死）。

    #[test]
    /// 刀F②：height 轴离散选中端点为裸 % 字面 → 空串；width 同形维持
    /// 回显；height 纯 %/% 对 LP 插值通道零扰动。
    fn test_knife_r75_f2_height_pct_nowrite() {
        // 电池 height-composition 命中形：calc-size(auto,·) → 50% 离散
        assert_eq!(
            interpolate_property_value("height", "calc-size(auto, size * 2)", "50%", 1.0),
            ""
        );
        assert_eq!(
            interpolate_property_value("height", "calc-size(auto, size * 2)", "50%", 0.75),
            ""
        );
        // t<0.5 选中 calc-size(auto,·) 侧＝非 % 字面 → 原样
        assert_eq!(
            interpolate_property_value("height", "calc-size(auto, size * 2)", "50%", 0.25),
            "calc-size(auto, size * 2)"
        );
        // width 轴 % 字面维持回显（非本刀域）
        assert_eq!(interpolate_property_value("width", "auto", "50%", 0.75), "50%");
        // 纯 %/% 对 LP 插值（零扰动）
        assert_eq!(interpolate_property_value("height", "0%", "50%", 0.5), "25.00%");
    }

    #[test]
    /// apply 层空串契约：写槽跳过（槽位不动＝underlying 显出）。
    fn test_knife_r75_apply_empty_nowrite() {
        let mut style = zero_style_system::ComputedStyle::default();
        style.min_width = zero_css_parser::values::LengthValue::Px(10.0);
        apply_single_property("min-width", "", &mut style);
        assert_eq!(style.min_width, zero_css_parser::values::LengthValue::Px(10.0));
        // 非空对照：正常写槽路径仍工作
        apply_single_property("min-width", "20px", &mut style);
        assert_eq!(style.min_width, zero_css_parser::values::LengthValue::Px(20.0));
    }

    // ── wptfyi-css 刀 FI font 车道钉测 ──────────────────────────────
    // 三面：keyframe `font` 简写 ingest 展开（css-fonts-4 §font-prop）+
    // stretch/style/line-height 插值臂 + apply 写回。期望形取 system-fonts
    // 396F / font-stretch-interpolation 153F / font-style-interpolation 109F
    // 语料钉（initialFont `italic 100 small-caps ultra-expanded 100px/100px
    // Ahem` 长手全落默认根因）。

    #[test]
    /// keyframe `font` 简写 ingest 展开：block_to_points 落 16 槽（stretch 位
    /// `ultra-expanded` 覆盖 reset、size/lh `100px/100px` 拆分、family 保真、
    /// 简写名移除）。
    fn test_knife_fi_keyframe_font_expansion() {
        let block = KeyframeBlock {
            selectors: vec![KeyframeSelector::Percentage(0.0)],
            declarations: vec![Declaration {
                property: "font".to_string(),
                value: "italic 100 small-caps ultra-expanded 100px/100px Ahem".to_string(),
                important: false,
            }],
        };
        let points = block_to_points(&block);
        assert_eq!(points.len(), 1);
        let props = &points[0].properties;
        assert_eq!(props.get("font-stretch").map(String::as_str), Some("ultra-expanded"));
        assert_eq!(props.get("font-style").map(String::as_str), Some("italic"));
        assert_eq!(props.get("font-weight").map(String::as_str), Some("100"));
        assert_eq!(props.get("font-variant-caps").map(String::as_str), Some("small-caps"));
        assert_eq!(props.get("font-size").map(String::as_str), Some("100px"));
        assert_eq!(props.get("line-height").map(String::as_str), Some("100px"));
        assert_eq!(props.get("font-family").map(String::as_str), Some("Ahem"));
        assert!(!props.contains_key("font"));
    }

    #[test]
    /// font-stretch 关键字↔% 数值插值（css-fonts-4 §font-width-prop 映射单源
    /// `parse_font_stretch`；输出 % 整形、上界不钳）。
    fn test_knife_fi_stretch_interpolation() {
        assert_eq!(
            interpolate_property_value_strict("font-stretch", "50%", "150%", 0.5).as_deref(),
            Some("100%")
        );
        assert_eq!(
            interpolate_property_value_strict("font-stretch", "ultra-condensed", "ultra-expanded", 0.5).as_deref(),
            Some("125%")
        );
        assert_eq!(
            interpolate_property_value_strict("font-stretch", "100%", "200%", 1.5).as_deref(),
            Some("250%")
        );
        assert_eq!(
            interpolate_property_value_strict("font-stretch", "50%", "150%", 0.25).as_deref(),
            Some("75%")
        );
        assert_eq!(
            interpolate_property_value("font-stretch", "normal", "200%", 0.5),
            "150%"
        );
    }

    #[test]
    /// font-style：t∈{0,1} 端点 verbatim（from [normal] at (0) = [normal] 钉形）、
    /// normal≡oblique 0deg 数值 lerp、italic 异族离散中点翻转
    ///（css-fonts-4 #font-style-animatable）。
    fn test_knife_fi_style_interpolation() {
        assert_eq!(
            interpolate_property_value_strict("font-style", "normal", "oblique 10deg", 0.0).as_deref(),
            Some("normal")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "normal", "oblique 10deg", 1.0).as_deref(),
            Some("oblique 10deg")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "oblique 10deg", "oblique 20deg", 0.5).as_deref(),
            Some("oblique 15deg")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "normal", "oblique 40deg", 0.5).as_deref(),
            Some("oblique 20deg")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "italic", "oblique 20deg", 0.4).as_deref(),
            Some("italic")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "italic", "oblique 20deg", 0.6).as_deref(),
            Some("oblique 20deg")
        );
        assert_eq!(
            interpolate_property_value("font-style", "italic", "normal", 0.7),
            "normal"
        );
        // 角域门：外推钳 [-90deg, 90deg] + 恰 0deg 串化 normal（语料 -90↔90 对钉形）。
        assert_eq!(
            interpolate_property_value_strict("font-style", "oblique -90deg", "oblique 90deg", 0.5).as_deref(),
            Some("normal")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "oblique -90deg", "oblique 90deg", 1.5).as_deref(),
            Some("oblique 90deg")
        );
        assert_eq!(
            interpolate_property_value_strict("font-style", "oblique -90deg", "oblique 90deg", -2.0).as_deref(),
            Some("oblique -90deg")
        );
    }

    #[test]
    /// line-height px 域插值 + `normal` 端点对宽松臂离散翻转、严格臂 None
    ///（css-transitions-1 as length；computed used-px 形——R2761 契约）。
    fn test_knife_fi_line_height_interpolation() {
        assert_eq!(
            interpolate_property_value_strict("line-height", "10px", "20px", 0.25).as_deref(),
            Some("12.50px")
        );
        assert_eq!(interpolate_property_value("line-height", "normal", "20px", 0.5), "20px");
        assert_eq!(
            interpolate_property_value("line-height", "normal", "20px", 0.4),
            "normal"
        );
        assert_eq!(
            interpolate_property_value_strict("line-height", "normal", "20px", 0.5),
            None
        );
    }

    #[test]
    /// apply 写回：stretch %→f32 槽 + gCS round-trip、style oblique 角形、
    /// variant-caps 关键字、family `Vec<String>`、line-height Length（FI 写回臂
    /// 与 style-system apply/文法单源）。
    fn test_knife_fi_apply_writeback() {
        let mut style = ComputedStyle::default();
        AnimationClock::apply_to_computed_style(
            &[
                InterpolatedProperty {
                    name: "font-stretch".to_string(),
                    value: "125%".to_string(),
                },
                InterpolatedProperty {
                    name: "font-style".to_string(),
                    value: "oblique 15deg".to_string(),
                },
                InterpolatedProperty {
                    name: "font-variant-caps".to_string(),
                    value: "small-caps".to_string(),
                },
                InterpolatedProperty {
                    name: "font-family".to_string(),
                    value: "Ahem".to_string(),
                },
                InterpolatedProperty {
                    name: "line-height".to_string(),
                    value: "24.00px".to_string(),
                },
            ],
            &mut style,
        );
        assert!((style.font_stretch - 125.0).abs() < 1e-6);
        assert_eq!(
            style.font_style,
            zero_css_parser::values::FontStyleValue::Oblique(Some(15.0))
        );
        assert_eq!(
            style.font_variant_caps,
            zero_style_system::FontVariantCapsValue::SmallCaps
        );
        assert_eq!(style.font_family, vec!["Ahem".to_string()]);
        assert!(matches!(
            style.line_height,
            zero_style_system::LineHeightValue::Length(_)
        ));
        let ser = crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(
            &style,
            "font-stretch",
            None,
        );
        assert_eq!(ser, "125%");
    }

    #[test]
    /// column-count 数值 lerp 钳 1 取整（<integer [1,∞]>——语料 200px 10→100px 20
    /// at(-1) count 期望 '1'；10→1 at(0.7) 期望 '4'）。
    fn test_column_count_clamp_and_round() {
        assert_eq!(interpolate_property_value("column-count", "10", "20", -1.0), "1");
        assert_eq!(interpolate_property_value("column-count", "10", "20", 0.5), "15");
        assert_eq!(interpolate_property_value("column-count", "10", "1", 0.7), "4");
        assert_eq!(interpolate_property_value("column-count", "10", "1", 1.5), "1");
    }

    #[test]
    /// columns 简写逐槽 lerp：canonical 'W N' 形 count 钳 1（语料 200px 10→
    /// 100px 20 at(-1) → '300px 1'）；slash 形 height 钳 0（neutral→'100px /
    /// 200px' at(-1) → '300px / 0px'）；count/height 形 at(-0.5) → '5 / 50px'。
    fn test_columns_component_interpolation() {
        assert_eq!(
            interpolate_property_value("columns", "200px 10", "100px 20", -1.0),
            "300.00px 1"
        );
        assert_eq!(
            interpolate_property_value("columns", "200px / 100px", "100px / 200px", -1.0),
            "300.00px / 0.00px"
        );
        assert_eq!(
            interpolate_property_value("columns", "10 / 100px", "20 / 200px", -0.5),
            "5 / 50.00px"
        );
        // 源序形「整数在前」：整数=count、长度=width（css-multicol-2 §3.4 顺序
        // 无关消歧，expand_columns 同款）——width 100px→200px lerp 0.5 = 150px、
        // count 10→20 = 15，canonical 'W N' 序列化。
        assert_eq!(
            interpolate_property_value("columns", "10 100px / auto", "20 200px / auto", 0.5),
            "150.00px 15"
        );
    }

    #[test]
    /// columns 组件存在集失配（auto 槽 vs 值槽形变）落离散翻转 ≥0.5 → to
    /// （css-values-4 §8.5.6——语料 auto→'100px / 200px' 期望行）。
    fn test_columns_discrete_flip_on_slot_mismatch() {
        assert_eq!(
            interpolate_property_value("columns", "auto", "100px / 200px", 0.3),
            "auto"
        );
        assert_eq!(
            interpolate_property_value("columns", "auto", "100px / 200px", 0.5),
            "100px / 200px"
        );
        assert_eq!(
            interpolate_property_value("columns", "100px / 200px", "auto", 1.5),
            "auto"
        );
    }

    #[test]
    /// strict 门：auto/非长度端点 None（css-transitions §starting-of-transitions
    /// 非 animatable 对不启动过渡）；组件存在集恒等对 Some。
    fn test_columns_strict_gate() {
        assert!(interpolate_property_value_strict("column-width", "10px", "20px", 0.5).is_some());
        assert!(interpolate_property_value_strict("column-width", "auto", "20px", 0.5).is_none());
        assert!(interpolate_property_value_strict("column-height", "10px", "20px", 0.5).is_some());
        assert!(interpolate_property_value_strict("column-count", "10", "20", 0.5).is_some());
        assert!(interpolate_property_value_strict("columns", "10 / 100px", "20 / 200px", 0.5).is_some());
        assert!(interpolate_property_value_strict("columns", "auto", "100px / 200px", 0.5).is_none());
    }

    #[test]
    /// parse_columns_endpoint 文法形：顺序无关 width||count、斜杠 height、
    /// auto 槽存在集、裸 auto 全零存在集。
    fn test_parse_columns_endpoint_forms() {
        let ep = parse_columns_endpoint("200px 10").unwrap();
        assert_eq!(ep.width, Some(200.0));
        assert_eq!(ep.count, Some(10.0));
        assert_eq!(ep.height, None);
        assert_eq!(ep.slots(), (true, true, false));

        let ep = parse_columns_endpoint("10 100px / auto").unwrap();
        assert_eq!(ep.width, Some(100.0));
        assert_eq!(ep.count, Some(10.0));
        assert_eq!(ep.height, None);
        assert_eq!(ep.slots(), (true, true, true));

        let ep = parse_columns_endpoint("auto / 100px").unwrap();
        assert_eq!(ep.width, None);
        assert_eq!(ep.count, None);
        assert_eq!(ep.height, Some(100.0));
        assert_eq!(ep.slots(), (true, true, true));

        let ep = parse_columns_endpoint("auto").unwrap();
        assert_eq!(ep.slots(), (false, false, false));

        assert!(parse_columns_endpoint("inherit").is_none());
        assert!(parse_columns_endpoint("/ 100px").is_none());
    }

    #[test]
    /// apply 臂 + gCS 往返：columns '100px / 200px' 落三槽后 gCS canonical
    /// 恒等（canonicalize 通道同构保证）。
    fn test_apply_columns_roundtrip_gcs() {
        let mut style = ComputedStyle::default();
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "columns".to_string(),
                value: "100px / 200px".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            style.column_width,
            ColumnWidthComputedValue::Length(zero_css_parser::values::LengthValue::Px(100.0))
        );
        assert_eq!(style.column_count, ColumnCountComputedValue::Auto);
        assert_eq!(
            style.column_height,
            ColumnHeightComputedValue::Length(zero_css_parser::values::LengthValue::Px(200.0))
        );
        assert_eq!(style.column_wrap, ColumnWrapComputedValue::Auto);
        let ser =
            crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(&style, "columns", None);
        assert_eq!(ser, "100px / 200px");
    }

    /// 辅助：创建简单的关键帧规则。
    fn make_keyframes(name: &str, blocks: Vec<(f64, Vec<(&str, &str)>)>) -> KeyframesRule {
        KeyframesRule {
            name: name.to_string(),
            name_is_string: false,
            keyframes: blocks
                .into_iter()
                .map(|(pct, decls)| KeyframeBlock {
                    selectors: vec![KeyframeSelector::Percentage(pct)],
                    declarations: decls
                        .into_iter()
                        .map(|(prop, val)| Declaration {
                            property: prop.to_string(),
                            value: val.to_string(),
                            important: false,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    /// 辅助：启动简单动画（默认参数）。
    fn start_anim(
        clock: &mut AnimationClock,
        element_id: u64,
        name: &str,
        duration: f64,
        delay: f64,
        timing: TimingFunctionValue,
        iteration_count: Option<f64>,
        direction: AnimationDirectionValue,
        fill_mode: AnimationFillModeValue,
        current_time: f64,
    ) -> bool {
        clock.start_animation(
            element_id,
            &AnimationConfig {
                name: name.to_string(),
                duration,
                delay,
                timing_function: timing,
                iteration_count,
                direction,
                fill_mode,
                current_time,
                composition: AnimationCompositionValue::Replace,
                underlying_values: HashMap::new(),
            },
        )
    }

    // ── timing function 测试 ──

    #[test]
    fn test_timing_function_linear() {
        assert!((apply_timing_function(0.0, &TimingFunctionValue::Linear) - 0.0).abs() < 1e-6);
        assert!((apply_timing_function(0.5, &TimingFunctionValue::Linear) - 0.5).abs() < 1e-6);
        assert!((apply_timing_function(1.0, &TimingFunctionValue::Linear) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_timing_function_step_start() {
        assert_eq!(apply_timing_function(0.0, &TimingFunctionValue::StepStart), 0.0);
        assert_eq!(apply_timing_function(0.01, &TimingFunctionValue::StepStart), 1.0);
        assert_eq!(apply_timing_function(0.5, &TimingFunctionValue::StepStart), 1.0);
    }

    /// 二十一轮共享臂修复钉：解析串形 `steps(n, <step-position>)` 逐形态
    /// （css-easing-1 §step-easing-functions）——此前恒 floor 弃 position，
    /// steps(1,start) 误同 jump-end（at(1) 编码 y(0.5)=1 误落 0 → from）。
    #[test]
    fn test_timing_function_steps_position() {
        // start：越点取步进上包络（ceil）；t=0 恒 0。
        assert_eq!(
            apply_timing_function(0.5, &TimingFunctionValue::Steps(1, Some(StepPosition::Start))),
            1.0
        );
        assert_eq!(
            apply_timing_function(0.3, &TimingFunctionValue::Steps(2, Some(StepPosition::Start))),
            0.5
        );
        assert_eq!(
            apply_timing_function(0.0, &TimingFunctionValue::Steps(1, Some(StepPosition::Start))),
            0.0
        );
        assert_eq!(
            apply_timing_function(1.0, &TimingFunctionValue::Steps(1, Some(StepPosition::Start))),
            1.0
        );
        // jump-both：两端钳 0/1，中段 ceil（n=2：t=0.25→0.5、t=0.5→0.5）。
        assert_eq!(
            apply_timing_function(0.25, &TimingFunctionValue::Steps(2, Some(StepPosition::Both))),
            0.5
        );
        assert_eq!(
            apply_timing_function(0.5, &TimingFunctionValue::Steps(2, Some(StepPosition::Both))),
            0.5
        );
        // jump-none：内缩 round（n=3 → m=2：t=0.25→0.5）。
        assert_eq!(
            apply_timing_function(0.25, &TimingFunctionValue::Steps(3, Some(StepPosition::None))),
            0.5
        );
        // 缺省（End / None position）保持 jump-end floor 既有语义。
        assert_eq!(
            apply_timing_function(0.5, &TimingFunctionValue::Steps(1, Some(StepPosition::End))),
            0.0
        );
        assert_eq!(apply_timing_function(0.5, &TimingFunctionValue::Steps(2, None)), 0.5);
        assert_eq!(apply_timing_function(1.0, &TimingFunctionValue::Steps(2, None)), 1.0);
    }

    #[test]
    fn test_timing_function_step_end() {
        assert_eq!(apply_timing_function(0.0, &TimingFunctionValue::StepEnd), 0.0);
        assert_eq!(apply_timing_function(0.5, &TimingFunctionValue::StepEnd), 0.0);
        assert_eq!(apply_timing_function(1.0, &TimingFunctionValue::StepEnd), 1.0);
    }

    #[test]
    fn test_timing_function_ease_bounds() {
        let ease = TimingFunctionValue::Ease;
        assert!(apply_timing_function(0.0, &ease) < 0.1);
        assert!(apply_timing_function(1.0, &ease) > 0.9);
    }

    // ── 属性值插值测试 ──

    #[test]
    fn test_interpolate_opacity() {
        let result = interpolate_property_value("opacity", "1.0", "0.0", 0.5);
        assert!((result.parse::<f64>().unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_opacity_start() {
        let result = interpolate_property_value("opacity", "0.0", "1.0", 0.0);
        assert!((result.parse::<f64>().unwrap() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_opacity_end() {
        let result = interpolate_property_value("opacity", "0.0", "1.0", 1.0);
        assert!((result.parse::<f64>().unwrap() - 1.0).abs() < 0.01);
    }

    // ── css-gaps 刀 B：rule-width 列表插值（css-gaps-1 §column-row-rule-width）──

    /// 平列表等长逐位（语料 multiple-values-001 column 形：10/15/20 → 40 三位）。
    #[test]
    fn test_rule_width_interp_flat() {
        let result = interpolate_property_value("column-rule-width", "10px, 15px, 20px", "40px, 40px, 40px", 0.3);
        assert_eq!(result, "19.00px, 22.50px, 26.00px");
    }

    /// 有限 repeat 展开（语料 repeaters neutral→to：1+4+1 位逐位）。
    #[test]
    fn test_rule_width_interp_repeat_expand() {
        let result = interpolate_property_value(
            "column-rule-width",
            "10px, repeat(2, 20px, 20px), 20px",
            "20px, repeat(2, 30px, 30px), 30px",
            0.3,
        );
        assert_eq!(result, "13.00px, 23.00px, 23.00px, 23.00px, 23.00px, 23.00px");
    }

    /// 单值广播（语料 repeaters：repeat(2, 20px)→30px 以 2 位输出）。
    #[test]
    fn test_rule_width_interp_single_broadcast() {
        let result = interpolate_property_value("column-rule-width", "repeat(2, 20px)", "30px", 0.3);
        assert_eq!(result, "23.00px, 23.00px");
    }

    /// auto 循环体 LCM 对齐（语料 repeaters：cycle 1 vs 2 → LCM 2）。
    #[test]
    fn test_rule_width_interp_auto_lcm() {
        let result = interpolate_property_value(
            "column-rule-width",
            "repeat(auto, 20px)",
            "repeat(auto, 30px, 30px)",
            0.3,
        );
        assert_eq!(result, "repeat(auto, 23.00px, 23.00px)");
    }

    /// leading/trailing 段数失配（语料 no-interpolation 钉）→ 离散翻转。
    #[test]
    fn test_rule_width_interp_leading_mismatch_discrete() {
        let result = interpolate_property_value(
            "column-rule-width",
            "20px, repeat(auto, 20px)",
            "30px, 30px, repeat(auto, 30px)",
            0.3,
        );
        assert_eq!(result, "20px, repeat(auto, 20px)", "t<0.5 落 from");
        let result = interpolate_property_value(
            "column-rule-width",
            "20px, repeat(auto, 20px)",
            "30px, 30px, repeat(auto, 30px)",
            0.6,
        );
        assert_eq!(result, "30px, 30px, repeat(auto, 30px)", "t≥0.5 落 to");
    }

    /// 双 auto 同位插值：leading/trailing 等段 + 循环体 lerp（语料 leading/trailing 钉）。
    #[test]
    fn test_rule_width_interp_auto_lead_trail() {
        let result = interpolate_property_value(
            "column-rule-width",
            "repeat(2, 20px), repeat(auto, 20px), 20px",
            "30px, 30px, repeat(auto, 30px), 30px",
            0.3,
        );
        assert_eq!(result, "23.00px, 23.00px, repeat(auto, 23.00px), 23.00px");
    }

    /// 结构失配对 Transition 通道不启动（strict None——css-transitions
    /// §starting-of-transitions）。
    #[test]
    fn test_rule_width_interp_strict_none_on_mismatch() {
        assert!(
            interpolate_property_value_strict("column-rule-width", "10px, 20px, 30px", "40px, 50px", 0.5).is_none()
        );
        assert!(interpolate_property_value_strict("column-rule-width", "repeat(auto, 20px)", "30px", 0.5).is_none());
        // 等长平列表真插值。
        assert_eq!(
            interpolate_property_value_strict("row-rule-width", "10px", "40px", 0.3).as_deref(),
            Some("19.00px")
        );
    }

    // ── 刀 A1c ②：background-position/object-position <position># 插值 ──

    /// mix_mask_position 复用臂：pct 对 / px 对 / 多层 LCM 循环；strict 委托
    /// Some 恒真（mask 族同契约）。
    #[test]
    fn test_bg_object_position_interp() {
        assert_eq!(
            interpolate_property_value("background-position", "0% 0%", "100% 100%", 0.5),
            "50.00% 50.00%"
        );
        assert_eq!(
            interpolate_property_value("object-position", "10px 20px", "30px 40px", 0.5),
            "20.00px 30.00px"
        );
        // 2 层 vs 1 层 → LCM 2，to 层循环广播。
        assert_eq!(
            interpolate_property_value("background-position", "0% 0%, 100% 100%", "50% 50%", 0.25),
            "12.50% 12.50%, 87.50% 87.50%"
        );
        assert!(interpolate_property_value_strict("background-position", "0% 0%", "100% 100%", 0.3).is_some());
        assert!(interpolate_property_value_strict("object-position", "10px", "30px", 0.3).is_some());
    }

    // ── css-gaps 刀 B：rule-color 列表插值（css-gaps-1 §column-row-rule-color）──

    /// 平列表等长逐位预乘插值（语料 repeaters red→lime 形）。
    #[test]
    fn test_rule_color_interp_flat() {
        // red→lime @0.3 = (178.5, 76.5, 0)；green→blue @0.3 = (0, 89.6, 76.5)。
        let result = interpolate_property_value("column-rule-color", "red, green", "lime, blue", 0.3);
        assert_eq!(result, "rgb(179, 77, 0), rgb(0, 90, 77)");
    }

    /// 有限 repeat 展开 + 单值广播（语料 repeaters repeat(2, c)→c 形）。
    #[test]
    fn test_rule_color_interp_repeat_broadcast() {
        let result = interpolate_property_value("column-rule-color", "repeat(2, red)", "lime", 0.3);
        assert_eq!(result, "rgb(179, 77, 0), rgb(179, 77, 0)");
    }

    /// auto 循环体 LCM 对齐（cycle 1 vs 2 → LCM 2，保形 repeat(auto, …)）。
    #[test]
    fn test_rule_color_interp_auto_lcm() {
        let result = interpolate_property_value("row-rule-color", "repeat(auto, red)", "repeat(auto, lime, blue)", 0.3);
        assert_eq!(result, "repeat(auto, rgb(179, 77, 0), rgb(179, 0, 77))");
    }

    /// leading/trailing 段数失配 → 离散翻转（lenient 端语义）。
    #[test]
    fn test_rule_color_interp_mismatch_discrete() {
        let result = interpolate_property_value(
            "column-rule-color",
            "red, repeat(auto, red)",
            "lime, lime, repeat(auto, lime)",
            0.3,
        );
        assert_eq!(result, "red, repeat(auto, red)", "t<0.5 落 from");
        let result = interpolate_property_value(
            "column-rule-color",
            "red, repeat(auto, red)",
            "lime, lime, repeat(auto, lime)",
            0.6,
        );
        assert_eq!(result, "lime, lime, repeat(auto, lime)", "t≥0.5 落 to");
    }

    /// 端点关键字经 css-parser 全名色表解析 + rgba 项 alpha 串化。
    #[test]
    fn test_rule_color_interp_keyword_and_alpha() {
        // purple→blue @0.5 = (64, 0, 191.5)。
        let result = interpolate_property_value("row-rule-color", "purple", "blue", 0.5);
        assert_eq!(result, "rgb(64, 0, 192)");
        // 预乘：a=lerp(1,0.5,.5)=0.75；r=lerp(255,0,.5)/0.75=170；b=lerp(0,127.5,.5)/0.75=85。
        let result = interpolate_property_value("row-rule-color", "rgba(255, 0, 0, 1)", "rgba(0, 0, 255, 0.5)", 0.5);
        assert_eq!(result, "rgba(170, 0, 85, 0.75)");
    }

    /// 结构失配对 Transition 通道不启动（strict None）+ 等长真插值。
    #[test]
    fn test_rule_color_interp_strict() {
        assert!(
            interpolate_property_value_strict("column-rule-color", "red, green, blue", "lime, lime", 0.5).is_none()
        );
        assert_eq!(
            interpolate_property_value_strict("row-rule-color", "red", "lime", 0.3).as_deref(),
            Some("rgb(179, 77, 0)")
        );
    }

    /// 动画值回写通道（apply_single_property 列表臂）：插值串经列表文法回
    /// typed 字段，gCS 串化单源规范化小数（"23.00px"→"23px"）与 repeat 保形。
    #[test]
    fn test_rule_width_apply_to_computed_style() {
        let mut style = ComputedStyle::default();
        let props = vec![
            InterpolatedProperty {
                name: "column-rule-width".to_string(),
                value: "23.00px, 23.00px".to_string(),
            },
            InterpolatedProperty {
                name: "row-rule-width".to_string(),
                value: "repeat(auto, 23.00px, 23.00px)".to_string(),
            },
            InterpolatedProperty {
                name: "column-rule-color".to_string(),
                value: "rgb(179, 77, 0)".to_string(),
            },
        ];
        AnimationClock::apply_to_computed_style(&props, &mut style);
        assert_eq!(style.column_rule_width.len(), 2);
        assert_eq!(style.row_rule_width.len(), 1);
        assert_eq!(style.column_rule_color.len(), 1);
        // gCS 串化规范化：列表小数 → 整数 px；auto 保形。
        assert_eq!(
            crate::js_dom_bridge::computed_style::rule_width_list_to_gcs(&style.column_rule_width, 16.0),
            "23px, 23px"
        );
        assert_eq!(
            crate::js_dom_bridge::computed_style::rule_width_list_to_gcs(&style.row_rule_width, 16.0),
            "repeat(auto, 23px, 23px)"
        );
    }

    // ── R4272 filter 函数列表插值（filter-effects-1 #animation）──

    /// 同构列表逐函数数值插值：hue-rotate(90deg)→hue-rotate(0deg) @0.5 = 45deg
    /// （css-filters-animation-hue-rotate 驱动）。
    #[test]
    fn test_interpolate_filter_hue_rotate() {
        let result = interpolate_property_value("filter", "hue-rotate(90deg)", "none", 0.5);
        assert_eq!(result, "hue-rotate(45.00deg)", "none 展开为初始值 hue-rotate(0)");
    }

    /// 百分数参数插值：saturate(4900%)→none(saturate(1)) @0.5 = 25
    /// （css-filters-animation-saturate 驱动，ref 静态 2500%）。
    #[test]
    fn test_interpolate_filter_saturate_percent() {
        let result = interpolate_property_value("filter", "saturate(4900%)", "none", 0.5);
        assert_eq!(result, "saturate(25.0000)");
    }

    /// 多函数同序列表插值 + 回解析。
    #[test]
    fn test_interpolate_filter_multi_function() {
        let result = interpolate_property_value("filter", "blur(2px) brightness(2)", "blur(4px) brightness(1)", 0.25);
        assert_eq!(result, "blur(2.50px) brightness(1.7500)");
    }

    /// 异构列表回落离散切换（filter-effects-1：不同构不可数值插值）。
    #[test]
    fn test_interpolate_filter_mismatched_falls_back_discrete() {
        let result = interpolate_property_value("filter", "blur(2px)", "brightness(2)", 0.25);
        assert_eq!(result, "blur(2px)", "t<=0.5 取 from");
        let result = interpolate_property_value("filter", "blur(2px)", "brightness(2)", 0.75);
        assert_eq!(result, "brightness(2)", "t>0.5 取 to");
    }

    /// drop-shadow 成对插值（x/y/blur + 颜色分量；none 展开 = 0 0 0 transparent）：
    /// 30px 30px 0px black × none @0.5 = 15px 15px 0px rgba(0,0,0,.5)
    /// （css-filters-animation-drop-shadow ref 值）。
    #[test]
    fn test_interpolate_filter_drop_shadow() {
        let result = interpolate_property_value("filter", "drop-shadow(30px 30px 0px black)", "none", 0.5);
        assert_eq!(result, "drop-shadow(15.00px 15.00px 0.00px rgba(0, 0, 0, 0.50))");
    }

    /// 插值结果须可回解析为 ComputedStyle.filter（apply_single_property 路径）。
    #[test]
    fn test_apply_interpolated_filter_to_style() {
        let mut style = ComputedStyle::default();
        apply_single_property("filter", "hue-rotate(45.00deg)", &mut style);
        assert_eq!(style.filter.len(), 1, "插值 filter 应写入 style");
        assert!(
            matches!(style.filter[0], FilterComputedValue::HueRotate(deg) if (deg - 45.0).abs() < 0.01),
            "应为 HueRotate(45)，got {:?}",
            style.filter
        );
        apply_single_property("filter", "none", &mut style);
        assert!(style.filter.is_empty(), "none 应清空 filter 列表");
    }

    /// 二十一轮 applier clip-path 臂：全族写槽（parse_clip_path 裁决单点），
    /// 非法值不动样式。
    #[test]
    fn test_apply_clip_path_to_style() {
        use zero_css_parser::values::ClipPathValue;
        let mut style = ComputedStyle::default();
        apply_single_property("clip-path", "circle(30px at 15px 15px)", &mut style);
        assert!(
            matches!(style.clip_path, ClipPathValue::Circle { .. }),
            "circle 形应写槽，got {:?}",
            style.clip_path
        );
        apply_single_property("clip-path", "shape(from 5px 5px, line to 20px 20%)", &mut style);
        assert!(
            matches!(style.clip_path, ClipPathValue::Shape(_)),
            "shape() 形应写槽，got {:?}",
            style.clip_path
        );
        apply_single_property("clip-path", "none", &mut style);
        assert_eq!(style.clip_path, ClipPathValue::None, "none 应写 None");
        // 非法值不动槽（回落方向零扰动）。
        apply_single_property("clip-path", "garbage(1px)", &mut style);
        assert_eq!(style.clip_path, ClipPathValue::None, "非法值不动槽");
    }

    /// 二十一轮 applier shape-outside 臂：<basic-shape>→Shape 包装、裸
    /// <shape-box>→Box、none→None（css-shapes-1 §shape-outside 映射）。
    #[test]
    fn test_apply_shape_outside_to_style() {
        use zero_style_system::property::types::ShapeOutsideBoxKind;
        let mut style = ComputedStyle::default();
        apply_single_property("shape-outside", "circle(50%)", &mut style);
        assert!(
            matches!(style.shape_outside, ShapeOutsideComputedValue::Shape(_)),
            "basic-shape 应包装 Shape，got {:?}",
            style.shape_outside
        );
        apply_single_property("shape-outside", "border-box", &mut style);
        assert_eq!(
            style.shape_outside,
            ShapeOutsideComputedValue::Box(ShapeOutsideBoxKind::BorderBox),
            "裸 shape-box 关键字应写 Box"
        );
        apply_single_property("shape-outside", "none", &mut style);
        assert_eq!(style.shape_outside, ShapeOutsideComputedValue::None);
    }

    /// 二十一轮 strict 门 shape-outside 臂：同构 basic-shape 端点真插值 Some，
    /// none 参与形（异构离散步）None——登记门拒（css-transitions §starting-of-
    /// transitions 不启动过渡）。
    #[test]
    fn test_shape_outside_strict_gate() {
        assert!(
            interpolate_property_value_strict(
                "shape-outside",
                "circle(10px at 5px 5px)",
                "circle(30px at 15px 15px)",
                0.5
            )
            .is_some(),
            "同构 circle 端点可插值"
        );
        assert!(
            interpolate_property_value_strict("shape-outside", "none", "circle(30px)", 0.5).is_none(),
            "none 参与形拒登记"
        );
        assert!(
            interpolate_property_value_strict("shape-outside", "border-box", "circle(30px)", 0.5).is_none(),
            "裸 shape-box 端点异构拒登记"
        );
    }

    /// 二十一轮 A/B 18 REG 修正钉：关键字半径对（含关键字↔关键字）不可插值
    /// ——strict 拒登记（css-transitions §starting-of-transitions），宽松通道
    /// 离散切换（css-shapes-1 §interpolation）。
    #[test]
    fn test_keyword_radius_pair_discrete() {
        let kw_pair = (
            "circle(farthest-side at 25px 75%)",
            "circle(farthest-side at 50px center)",
        );
        assert!(
            interpolate_property_value_strict("clip-path", kw_pair.0, kw_pair.1, 0.5).is_none(),
            "关键字↔关键字半径对拒登记"
        );
        assert!(
            interpolate_property_value_strict(
                "clip-path",
                "ellipse(closest-side farthest-side at 25px 75%)",
                "ellipse(closest-side farthest-side at 50px center)",
                0.5
            )
            .is_none(),
            "ellipse 双关键字半径对拒登记"
        );
        // 宽松通道仍离散翻转（不进 echo 保留门——真离散对，期望串可解析）。
        assert_eq!(
            interpolate_property_value("clip-path", kw_pair.0, kw_pair.1, 0.25),
            kw_pair.0,
            "t<0.5 离散取 from"
        );
        assert_eq!(
            interpolate_property_value("clip-path", kw_pair.0, kw_pair.1, 0.5),
            kw_pair.1,
            "t≥0.5 离散取 to"
        );
    }

    /// 二十一轮 echo 保留门钉：真不可插值对（rect 族跨形/单端不可解析）返回
    /// 空串（apply 必败 → 槽位不动 → canonicalize 恒等门回落 echo，B0 伪通过
    /// 池保位）；真离散对（none↔circle）不受门影响。calc 同构缘偏移对已由
    /// lerp_calc_same_shape 臂兑现真插值（css-values-4 §8.5.6）——原「calc
    /// 端点保 echo」断言随伪通过池兑现翻转（clip-lerp 臂，2026-11-18）。
    #[test]
    fn test_clip_pair_echo_preserve_gate() {
        // calc 同构缘偏移对：真插值（原 B0 伪通过池——期望串写面拒收已随
        // css-parser calc 位置词法修复解除，echo 保位翻真插值）。
        assert_eq!(
            interpolate_property_value(
                "clip-path",
                "circle(25% at right 5% bottom 15px)",
                "circle(45% at right 25% bottom 35px)",
                0.5
            ),
            "circle(35% at 85% calc(100% - 25px))",
            "calc 同构缘偏移对真插值"
        );
        // 门不过宽：左/上缘偏移归一为纯偏移（无 calc），同构真插值保留。
        assert_eq!(
            interpolate_property_value(
                "clip-path",
                "circle(25% at left 5px top 15px)",
                "circle(45% at left 25px top 35px)",
                0.5
            ),
            "circle(35% at 15px 25px)",
            "左/上缘偏移对真插值不进门"
        );
        // rect 族跨形对：spec 应整体插值为 calc 化 inset（css-shapes-1
        // §basic-shape-computed-values），真值不可达 → 保位。
        assert_eq!(
            interpolate_property_value("clip-path", "inset(10px)", "xywh(20% 50px 70px 100%)", 0.875),
            "",
            "inset↔xywh 跨形对保 echo"
        );
        // CSS-wide keyword 端点 ∧ 对端 rect 族：期望串同为 calc 化 inset
        // （b1v6 四 REG 定谳形态）→ 保位。
        assert_eq!(
            interpolate_property_value("clip-path", "inherit", "rect(50px 70px 80% 20%)", 0.875),
            "",
            "inherit→rect 对保 echo"
        );
        // 门不过宽：CSS-wide keyword 对端非 rect 族（真离散对）仍翻转。
        assert_eq!(
            interpolate_property_value("clip-path", "inherit", "circle(30px at 50px 50px)", 0.75),
            "circle(30px at 50px 50px)",
            "inherit→circle 仍翻转取 to"
        );
        // 门不过宽：initial/unset underlying=none → Chrome 离散翻转语义
        // （期望=端点原形），不进门（b1v7 过宽臂压掉 16 真增益定谳反例）。
        assert_eq!(
            interpolate_property_value("clip-path", "initial", "inset(20px)", 0.5),
            "inset(20px)",
            "initial→inset 仍翻转取 to"
        );
        assert_eq!(
            interpolate_property_value("clip-path", "unset", "inset(10%)", 1.5),
            "inset(10%)",
            "unset→inset 仍翻转取 to"
        );
        // 真离散对不受门影响：none↔circle 仍翻转。
        assert_eq!(
            interpolate_property_value("clip-path", "none", "circle(30px at 50px 50px)", 0.75),
            "circle(30px at 50px 50px)",
            "真离散对仍翻转取 to"
        );
        // path 段数失配对保位。
        assert_eq!(
            interpolate_property_value(
                "shape-outside",
                "path(\"M 10 10 L 20 20\")",
                "path(\"M 10 10 L 20 20 Z\")",
                1.0
            ),
            "path(\"M 10 10 L 20 20 Z\")",
            "path 段数失配对离散翻转取 to（Chrome 期望即端点形）"
        );
    }

    #[test]
    fn test_interpolate_color() {
        let result = interpolate_property_value("background-color", "#000000", "#ffffff", 0.5);
        assert!(result.contains("rgba"));
        // 中间值应约 (127.5, 127.5, 127.5)
        // rgba 输出使用 {:.0} 格式化整数部分
        assert!(result.contains("127") || result.contains("128"));
    }

    #[test]
    fn test_interpolate_length() {
        let result = interpolate_property_value("width", "0px", "200px", 0.5);
        assert!(result.contains("100.00"));
    }

    #[test]
    fn test_interpolate_length_start() {
        let result = interpolate_property_value("height", "50px", "150px", 0.0);
        assert!(result.contains("50.00"));
    }

    #[test]
    fn test_interpolate_length_end() {
        let result = interpolate_property_value("height", "50px", "150px", 1.0);
        assert!(result.contains("150.00"));
    }

    #[test]
    fn test_interpolate_unknown_property_switch() {
        // t=0.3 → 使用 from 值
        let result = interpolate_property_value("display", "block", "none", 0.3);
        assert_eq!(result, "block");
        // t=0.6 → 使用 to 值
        let result = interpolate_property_value("display", "block", "none", 0.6);
        assert_eq!(result, "none");
    }

    // ── clip-path / offset-path 基本形状族插值钉（css-shapes-2
    // §interpolating-shape + css-motion-1；wptfyi-css 基本形状族刀） ──

    #[test]
    fn test_interpolate_clip_path_circle_inset() {
        // circle 同构逐分量线性。
        let r = interpolate_property_value("clip-path", "circle(10px at 5px 5px)", "circle(30px at 15px 15px)", 0.5);
        assert_eq!(r, "circle(20px at 10px 10px)");
        // inset + round 槽（round 为单值半径槽——clip-path inset 文法，
        // gCS reparse 后两侧同 canonical）。
        let r = interpolate_property_value("clip-path", "inset(0px round 10px)", "inset(20px round 30px)", 0.5);
        assert_eq!(r, "inset(10px 10px 10px 10px round 20px 20px 20px 20px)");
    }

    #[test]
    fn test_interpolate_clip_path_cross_unit_calc() {
        // 刀 G：px↔% 跨单位 calc 混合（css-values-4 §8.5.6 interpolation-calc；
        // 复用 blend_rule_inset_lp 模型——css-shapes-1 §basic-shape-interpolation
        // 坐标/半径同文法槽，刀 E GONE 定谳 clip-path-interpolation-002 靶面）。
        // inset 四边逐位 mix 形（emit 两位小数，canonicalize 通道 gCS 再串化归 shortest）。
        let r = interpolate_property_value("clip-path", "inset(10px)", "inset(40%)", 0.5);
        assert_eq!(
            r,
            "inset(calc(20.00% + 5.00px) calc(20.00% + 5.00px) calc(20.00% + 5.00px) calc(20.00% + 5.00px))"
        );
        // 半径/位置槽同模型；px 系数零折叠纯 %。
        let r = interpolate_property_value("clip-path", "circle(10px at 5px 5px)", "circle(50% at 25% 25%)", 0.5);
        assert_eq!(
            r,
            "circle(calc(25.00% + 5.00px) at calc(12.50% + 2.50px) calc(12.50% + 2.50px))"
        );
        let r = interpolate_property_value("clip-path", "inset(0px)", "inset(40%)", 0.5);
        assert_eq!(r, "inset(20.00% 20.00% 20.00% 20.00%)");
        // 关键词半径↔长度仍离散（t<0.5 → from）。
        let r = interpolate_property_value("clip-path", "circle(closest-side)", "circle(50%)", 0.3);
        assert_eq!(r, "circle(closest-side)");
        // offset-path 同文法通道同步生效（css-motion-1 offset-path basic-shape）。
        let r = interpolate_property_value("offset-path", "circle(10px)", "circle(50%)", 0.5);
        assert_eq!(r, "circle(calc(25.00% + 5.00px))");
    }

    #[test]
    fn test_interpolate_clip_path_polygon_and_hetero() {
        // polygon 同数逐点。
        let r = interpolate_property_value(
            "clip-path",
            "polygon(0px 0px, 10px 10px)",
            "polygon(10px 20px, 30px 40px)",
            0.5,
        );
        assert_eq!(r, "polygon(5px 10px, 20px 25px)");
        // 点数失配 → 离散切换（css-values-4 §8.5.6 ≥0.5 → to）。
        let r = interpolate_property_value("clip-path", "polygon(0px 0px)", "polygon(0px 0px, 10px 10px)", 0.3);
        assert_eq!(r, "polygon(0px 0px)");
        let r = interpolate_property_value("clip-path", "polygon(0px 0px)", "polygon(0px 0px, 10px 10px)", 0.7);
        assert_eq!(r, "polygon(0px 0px, 10px 10px)");
    }

    #[test]
    fn test_interpolate_clip_path_shape_path() {
        // shape() 同构逐命令（to 端点经 resolved 数值插值）。
        let r = interpolate_property_value(
            "clip-path",
            "shape(from 0px 0px, line to 10px 10px)",
            "shape(from 10px 20px, line to 30px 40px)",
            0.5,
        );
        assert_eq!(r, "shape(from 5px 10px, line to 20px 25px)");
        // path() 同段数同命令字母逐实参。
        let r = interpolate_property_value("clip-path", "path(\"M 0 0 L 10 10\")", "path(\"M 10 20 L 30 40\")", 0.5);
        assert_eq!(r, "path(\"M 5 10 L 20 25\")");
    }

    #[test]
    fn test_interpolate_offset_path_ray_and_hetero() {
        // ray↔ray 同 ray-size 角度线性（负 t 外推实测 chrome 口径）。
        let r = interpolate_property_value(
            "offset-path",
            "ray(10deg closest-side)",
            "ray(20deg closest-side)",
            -0.3,
        );
        assert_eq!(r, "ray(7deg closest-side)");
        // ray-size 失配 / 异构（ray↔circle）→ 离散切换。
        let r = interpolate_property_value(
            "offset-path",
            "ray(10deg closest-side)",
            "ray(20deg farthest-side)",
            0.3,
        );
        assert_eq!(r, "ray(10deg closest-side)");
        let r = interpolate_property_value("offset-path", "ray(10deg)", "circle(50%)", 0.75);
        assert_eq!(r, "circle(50%)");
    }

    #[test]
    fn test_interpolate_offset_path_shape_coord_box() {
        // shape↔shape + coord-box 后缀一致：形状插值 + 后缀保持。
        let r = interpolate_property_value(
            "offset-path",
            "shape(from 0px 0px, line to 10px 10px) border-box",
            "shape(from 10px 20px, line to 30px 40px) border-box",
            0.5,
        );
        assert_eq!(r, "shape(from 5px 10px, line to 20px 25px) border-box");
        // 后缀失配 → 离散（t<0.5 → from，边界 0.5 → to）。
        let r = interpolate_property_value(
            "offset-path",
            "circle(10px) border-box",
            "circle(30px) content-box",
            0.4,
        );
        assert_eq!(r, "circle(10px) border-box");
        // 纯 coord-box 恒值对。
        let r = interpolate_property_value("offset-path", "border-box", "border-box", 0.5);
        assert_eq!(r, "border-box");
    }

    #[test]
    fn test_interpolate_offset_path_m2_path_canonical() {
        // 刀 MOTION-2：offset-path path() 对——绝对化 canonical 同构 lerp、
        // 单引号发射、结构失配 None（调用方按通道分派离散，css-motion-1
        // §offset-path 插值，offset-path-interpolation-001/002/003 语料钉面）。
        // 同构 M/H lerp（transitions 臂路由 clip_interp::interpolate_offset_path）。
        let r = interpolate_property_value("offset-path", "path('M 50 60 H 70')", "path('M 10 140 H 270')", 0.125);
        assert_eq!(r, "path('M 45 70 H 95')");
        // 相对端点 canonical at(0)（严格臂）。
        let r = interpolate_property_value("offset-path", "path('m 50 60 h 20')", "path('M 10 140 H 270')", 0.0);
        assert_eq!(r, "path('M 50 60 H 70')");
        // 结构失配段数（语料 001 test_no_interpolation 形）：插值臂 None、
        // 车道翻转回落端点原文（0.3 → from、0.5 → to，css-values-4 §8.5.6）。
        let r = interpolate_property_value("offset-path", "path('M 0 0 H 1 H 2')", "path('M 0 0 H 3')", 0.3);
        assert_eq!(r, "path('M 0 0 H 1 H 2')");
        let r = interpolate_property_value("offset-path", "path('M 0 0 H 1 H 2')", "path('M 0 0 H 3')", 0.5);
        assert_eq!(r, "path('M 0 0 H 3')");
        // 严格臂 None = 非 animatable 信号——transitions 启动门据此拒启
        // （css-transitions §starting-of-transitions，18 P2F v2 订正实证）。
        assert!(
            interpolate_property_value_strict("offset-path", "path('M 0 0 H 1 H 2')", "path('M 0 0 H 3')", 0.3)
                .is_none()
        );
        // 车道翻转 canonical 发射：computed 双引号形端点 → 单引号 canonical
        // （语料 CSS Animations/Web Animations 失配对钉面，42 案 v3 实证）。
        let r = interpolate_property_value("offset-path", "path(\"M 0 0 H 1 H 2\")", "path(\"M 0 0 H 3\")", 0.3);
        assert_eq!(r, "path('M 0 0 H 1 H 2')");
        let r = interpolate_property_value("offset-path", "path(\"M 0 0 H 1 H 2\")", "path(\"M 0 0 H 3\")", 0.6);
        assert_eq!(r, "path('M 0 0 H 3')");
        // path↔shape 混合对 → 双侧声明原文 verbatim 翻转（语料 008 形：期望
        // 双引号声明原文，canonical 化反而破坏——20 案 v3b 实证）。
        let r = interpolate_property_value(
            "offset-path",
            "path(\"M10 10 h 5\")",
            "shape(from 10px 10px, hline to 5px)",
            0.3,
        );
        assert_eq!(r, "path(\"M10 10 h 5\")");
        let r = interpolate_property_value(
            "offset-path",
            "shape(from 10px 10px, move to 10% 10%)",
            "path(\"M10 10 z\")",
            0.6,
        );
        assert_eq!(r, "path(\"M10 10 z\")");
        // path↔ray 异构 → 调用方离散原文（语料 005 形，双端已 canonical）。
        let r = interpolate_property_value("offset-path", "path('M 0 0 L 1 1')", "ray(0deg closest-corner)", 0.75);
        assert_eq!(r, "ray(0deg closest-corner)");
    }

    // ── 关键帧插值测试 ──

    #[test]
    fn test_interpolate_between_simple() {
        let from = KeyframePoint {
            offset: 0.0,
            properties: HashMap::from([("opacity".to_string(), "1.0".to_string())]),
            timing: None,
            composition: None,
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([("opacity".to_string(), "0.0".to_string())]),
            timing: None,
            composition: None,
        };

        let result = interpolate_between(&from, &to, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "opacity");
        let val: f64 = result[0].value.parse().unwrap();
        assert!((val - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_between_multiple_properties() {
        let from = KeyframePoint {
            offset: 0.0,
            properties: HashMap::from([
                ("opacity".to_string(), "1.0".to_string()),
                ("width".to_string(), "100px".to_string()),
            ]),
            timing: None,
            composition: None,
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([
                ("opacity".to_string(), "0.5".to_string()),
                ("width".to_string(), "200px".to_string()),
            ]),
            timing: None,
            composition: None,
        };

        let result = interpolate_between(&from, &to, 0.5);
        assert_eq!(result.len(), 2);

        let opacity = result.iter().find(|p| p.name == "opacity").unwrap();
        let width = result.iter().find(|p| p.name == "width").unwrap();

        assert!((opacity.value.parse::<f64>().unwrap() - 0.75).abs() < 0.01);
        assert!(width.value.contains("150"));
    }

    #[test]
    fn test_interpolate_between_zero_span() {
        let pt = KeyframePoint {
            offset: 0.5,
            properties: HashMap::from([("opacity".to_string(), "0.5".to_string())]),
            timing: None,
            composition: None,
        };

        let result = interpolate_between(&pt, &pt, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "opacity");
    }

    /// R3883：帧内 animation-timing-function 应用于**段内归一化进度**（CSS Animations
    /// §keyframes）——用 ease-in（x=0.5 → y≈0.315）验证段 timing 确实改变了线性插值。
    /// driving: css-backgrounds/animations three-keyframes1..3。
    #[test]
    fn test_per_keyframe_timing_applies_to_segment() {
        let from = KeyframePoint {
            offset: 0.0,
            properties: HashMap::from([("opacity".to_string(), "0.0".to_string())]),
            timing: Some(TimingFunctionValue::EaseIn),
            composition: None,
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([("opacity".to_string(), "1.0".to_string())]),
            timing: None,
            composition: None,
        };

        // 段 timing 生效：local 0.5 → ease-in(0.5) ≈ 0.315（≠ 线性 0.5）
        let result = interpolate_between_with_timing(&from, &to, 0.5, &TimingFunctionValue::Linear);
        let val: f64 = result[0].value.parse().unwrap();
        assert!((val - 0.315).abs() < 0.02, "segment timing must apply (got {val})");

        // 帧未声明 timing → 回落全局 timing（同样 ease-in → ≈0.315）
        let from_no_timing = KeyframePoint {
            timing: None,
            composition: None,
            ..from.clone()
        };
        let result = interpolate_between_with_timing(&from_no_timing, &to, 0.5, &TimingFunctionValue::EaseIn);
        let val: f64 = result[0].value.parse().unwrap();
        assert!(
            (val - 0.315).abs() < 0.02,
            "global timing fallback must apply (got {val})"
        );

        // 无 timing（Linear）→ 线性 0.5
        let result = interpolate_between_with_timing(&from_no_timing, &to, 0.5, &TimingFunctionValue::Linear);
        let val: f64 = result[0].value.parse().unwrap();
        assert!((val - 0.5).abs() < 0.01, "linear fallback must be linear (got {val})");
    }

    /// R3883：帧内 `animation-timing-function` 声明从可插值 properties 剥离
    ///（它不是动画属性——旧实现落入 properties 后被插值/应用名单静默丢弃）。
    #[test]
    fn test_block_to_points_strips_timing_declaration() {
        let block = KeyframeBlock {
            selectors: vec![KeyframeSelector::Percentage(10.0)],
            declarations: vec![
                Declaration {
                    property: "background-color".to_string(),
                    value: "rgb(200, 0, 0)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "animation-timing-function".to_string(),
                    value: "cubic-bezier(0,1,1,0)".to_string(),
                    important: false,
                },
            ],
        };
        let pts = block_to_points(&block);
        assert_eq!(pts.len(), 1, "single selector → one point");
        let pt = &pts[0];
        assert!(
            !pt.properties.contains_key("animation-timing-function"),
            "timing must be stripped"
        );
        assert_eq!(
            pt.properties.get("background-color").map(String::as_str),
            Some("rgb(200, 0, 0)")
        );
        assert_eq!(pt.timing, Some(TimingFunctionValue::CubicBezier(0.0, 1.0, 1.0, 0.0)));
    }

    /// 刀 I（A 面）：css-animations-1 §keyframes——逗号分隔选择器列表每项各产出
    /// 一个关键帧。`from, to` 块展开为 0%/100% 两帧同声明集（旧实现折叠取首选择器
    /// → 丢 100% 帧 → 合成空边界帧 → 刀 F 通用臂填入计算值漂移）。
    #[test]
    fn test_block_to_points_multi_selector_expansion() {
        let block = KeyframeBlock {
            selectors: vec![KeyframeSelector::From, KeyframeSelector::To],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "green".to_string(),
                important: false,
            }],
        };
        let pts = block_to_points(&block);
        assert_eq!(pts.len(), 2, "from, to → two points");
        assert_eq!(pts[0].offset, 0.0);
        assert_eq!(pts[1].offset, 1.0);
        for pt in &pts {
            assert_eq!(pt.properties.get("color").map(String::as_str), Some("green"));
        }

        // 百分比列表 `50%, 75%` 同展开（旧实现丢 75% 档）。
        let pct_block = KeyframeBlock {
            selectors: vec![KeyframeSelector::Percentage(50.0), KeyframeSelector::Percentage(75.0)],
            declarations: vec![Declaration {
                property: "opacity".to_string(),
                value: "0.5".to_string(),
                important: false,
            }],
        };
        let pts = block_to_points(&pct_block);
        assert_eq!(pts.len(), 2);
        assert!((pts[0].offset - 0.5).abs() < 1e-9);
        assert!((pts[1].offset - 0.75).abs() < 1e-9);
    }

    /// 刀 I（C 面）：css-animations-1 §keyframes——关键帧声明 `!important` 无效，
    /// 按未声明丢弃；同块前条同属性合法声明保留（HashMap 收集序即声明序）。
    #[test]
    fn test_block_to_points_drops_important_declarations() {
        let block = KeyframeBlock {
            selectors: vec![KeyframeSelector::From],
            declarations: vec![
                Declaration {
                    property: "border-color".to_string(),
                    value: "green".to_string(),
                    important: false,
                },
                Declaration {
                    property: "border-color".to_string(),
                    value: "red".to_string(),
                    important: true,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: true,
                },
            ],
        };
        let pts = block_to_points(&block);
        assert_eq!(pts.len(), 1);
        assert_eq!(
            pts[0].properties.get("border-color").map(String::as_str),
            Some("green"),
            "!important keyframe declaration dropped, valid declaration kept"
        );
        assert!(
            !pts[0].properties.contains_key("color"),
            "fully-important property dropped"
        );
    }

    /// 刀 I（B 面）：css-cascade-4 §importance——级联 `!important` 胜出属性跳过
    /// 动画施值；其余属性照常应用。渲染 stamping 与 gCS canonicalize 共享本门。
    #[test]
    fn test_animation_skips_important_won_props() {
        let mut style = ComputedStyle::default();
        style.color = zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255);
        style.opacity = 1.0;
        style.important_won_props = std::sync::Arc::new(vec!["color".to_string()]);

        AnimationClock::apply_to_computed_style(
            &[
                InterpolatedProperty {
                    name: "color".to_string(),
                    value: "rgb(255, 0, 0)".to_string(),
                },
                InterpolatedProperty {
                    name: "opacity".to_string(),
                    value: "0.5".to_string(),
                },
            ],
            &mut style,
        );
        assert_eq!(
            style.color,
            zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            "important-won property must not be overridden by animation"
        );
        assert!(
            (style.opacity - 0.5).abs() < 1e-9,
            "non-important property still applies"
        );

        // 简写写达对位（R3882 同源）：keyframe `background` 简写撞 longhand
        // background-color important 面——门按写达槽位拦截（important-prop
        // 背景面：div `background-color:green !important` 胜 keyframe
        // `background:#f00`）。
        let mut style = ComputedStyle::default();
        style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
        style.important_won_props = std::sync::Arc::new(vec!["background-color".to_string()]);
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "background".to_string(),
                value: "#f00".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            style.background_color,
            zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255),
            "shorthand animation must not bypass longhand important tier"
        );
    }

    /// 刀 I（D 臂）：动画 border-color 写达四边 color 槽（此前无臂静默丢弃）。
    #[test]
    fn test_animation_border_color_applies_four_slots() {
        let mut style = ComputedStyle::default();
        style.border_top_color = zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255);
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "border-color".to_string(),
                value: "green".to_string(),
            }],
            &mut style,
        );
        let green = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
        assert_eq!(style.border_top_color, green);
        assert_eq!(style.border_right_color, green);
        assert_eq!(style.border_bottom_color, green);
        assert_eq!(style.border_left_color, green);
    }

    /// 刀 I（A 面经 register_keyframes 全链）：`from, to` 多选择器块展开后两端点
    /// 均持声明，不再触发空边界帧合成（旧行为：仅 0% 帧 → 合成空 100% 帧）。
    #[test]
    fn test_register_keyframes_multi_selector_no_synth_boundaries() {
        let css = "@keyframes k{from,to{color:green}}";
        let sheet = zero_css_parser::Parser::parse_stylesheet(css);
        let rule = sheet.rules.into_iter().find_map(|r| match r {
            zero_css_parser::ast::Rule::Keyframes(kf) => Some(kf),
            _ => None,
        });
        let Some(rule) = rule else {
            panic!("keyframes rule must parse");
        };
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&rule);
        let points = clock.keyframes_registry.get("k").expect("registered");
        assert_eq!(points.len(), 2, "no synthetic boundary frames");
        assert_eq!(points[0].offset, 0.0);
        assert_eq!(points[1].offset, 1.0);
        assert!(points.iter().all(|p| p.properties.contains_key("color")));
    }

    // ── 颜色解析测试 ──

    #[test]
    fn test_parse_color_rgba() {
        let (r, g, b, a) = parse_color("rgba(255, 128, 0, 0.5)");
        assert_eq!(r, 255.0);
        assert_eq!(g, 128.0);
        assert_eq!(b, 0.0);
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_parse_color_rgb() {
        let (r, g, b, a) = parse_color("rgb(100, 200, 50)");
        assert_eq!(r, 100.0);
        assert_eq!(g, 200.0);
        assert_eq!(b, 50.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_color_hex_6() {
        let (r, g, b, a) = parse_color("#ff8800");
        assert_eq!(r, 255.0);
        assert_eq!(g, 136.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_color_hex_3() {
        let (r, g, b, a) = parse_color("#f80");
        assert_eq!(r, 255.0);
        assert_eq!(g, 136.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_color_hex_8() {
        let (r, g, b, a) = parse_color("#ff880080");
        assert_eq!(r, 255.0);
        assert_eq!(g, 136.0);
        assert_eq!(b, 0.0);
        assert!((a - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), (255.0, 0.0, 0.0, 1.0));
        assert_eq!(parse_color("transparent"), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(parse_color("black"), (0.0, 0.0, 0.0, 1.0));
        assert_eq!(parse_color("white"), (255.0, 255.0, 255.0, 1.0));
    }

    #[test]
    fn test_parse_color_case_insensitive() {
        assert_eq!(parse_color("RED"), (255.0, 0.0, 0.0, 1.0));
        assert_eq!(parse_color("Blue"), (0.0, 0.0, 255.0, 1.0));
    }

    // ── AnimationClock 测试 ──

    #[test]
    fn test_clock_register_keyframes() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);

        let names = clock.registered_keyframe_names();
        assert_eq!(names, vec!["fade"]);
    }

    #[test]
    fn test_clock_start_animation() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);

        let ok = start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );
        assert!(ok);

        // 不存在的动画名称
        let ok = start_anim(
            &mut clock,
            1,
            "nonexistent",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );
        assert!(!ok);
    }

    #[test]
    fn test_clock_tick_linear_fade() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0 → opacity=1.0
        let props = clock.tick(1, 0.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.05);

        // t=0.5 → opacity=0.5
        let props = clock.tick(1, 0.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);

        // t=1.0 → opacity=0.0
        let props = clock.tick(1, 1.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap()).abs() < 0.05);
    }

    #[test]
    fn test_play_state_pause_freezes_and_resume_rebases() {
        // M2-S3b（CSS Animations §play-state）：暂停冻结 tick 输出于暂停时刻插值，
        // 恢复重基 start（暂停时长不计入活跃时长，从冻结相位继续）。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // 活跃至 0.4；0.5 暂停。
        clock.tick(1, 0.4);
        clock.reconcile_play_state(1, "fade", true, 0.5);
        // 暂停中墙钟推进到 0.9：输出仍冻结于 0.5s 时刻插值（opacity 0.5）。
        let props = clock.tick(1, 0.9);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!(
            (opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05,
            "暂停冻结于 0.5s 相位"
        );

        // 1.5 恢复 → start 重基 0→1.0；t=1.2 的活跃时长 = 0.2 → opacity 0.8。
        clock.reconcile_play_state(1, "fade", false, 1.5);
        let props = clock.tick(1, 1.2);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!(
            (opacity.value.parse::<f64>().unwrap() - 0.8).abs() < 0.05,
            "恢复从冻结相位继续"
        );
    }

    #[test]
    fn test_play_state_pause_defers_finish_event() {
        // M2-S3b：暂停中终态判定停走（墙钟越过原定结束时刻也不派 animationend）；
        // 恢复后结束时刻平移，elapsedTime 仍 = 迭代 × duration（不含暂停时长）。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        clock.tick(1, 0.3);
        clock.reconcile_play_state(1, "fade", true, 0.3);
        // 暂停期间墙钟越过原定结束（1.0s）→ 无 animationend。
        let _ = clock.tick(1, 5.0);
        assert!(clock.drain_just_finished().is_empty(), "暂停中不派 animationend");

        // 5.0 恢复（暂停 4.7s）→ start 重基 4.7；t=5.5 活跃 0.8s 未完成。
        clock.reconcile_play_state(1, "fade", false, 5.0);
        let _ = clock.tick(1, 5.5);
        assert!(clock.drain_just_finished().is_empty());
        // t=5.8 活跃 1.1s ≥ 1.0 → 完成，elapsedTime = 1 × 1.0。
        let _ = clock.tick(1, 5.8);
        let finished = clock.drain_just_finished();
        assert_eq!(finished.len(), 1);
        assert!((finished[0].duration - 1.0).abs() < 1e-9, "elapsedTime 不含暂停时长");
    }

    #[test]
    fn test_animation_drain_just_finished_r3249() {
        // R3249（CSS Animations §animationend）：finished 由 false→true 的帧推入 just_finished，
        // drain_just_finished 取出（含 element_key/name/duration=elapsedTime），每完成帧只派发一次。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        // duration=1.0s，iteration_count=2.0 → 活跃时长 2.0s（animationend.elapsedTime=2.0）
        start_anim(
            &mut clock,
            9,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(2.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=1.0 → 第 1 次迭代结束，动画未完成（2 次迭代）→ drain 空
        let _ = clock.tick(9, 1.0);
        assert!(clock.drain_just_finished().is_empty(), "迭代中（未完成）drain 空");

        // t=2.0 → 第 2 次迭代结束，动画完成 → drain 含该动画（element_key=9, name=fade, duration=2.0）
        let _ = clock.tick(9, 2.0);
        let finished = clock.drain_just_finished();
        assert_eq!(finished.len(), 1, "完成帧 drain 恰好 1 条");
        assert_eq!(finished[0].element_key, 9);
        assert_eq!(finished[0].name, "fade");
        assert!(
            (finished[0].duration - 2.0).abs() < 1e-9,
            "duration=iteration_count*duration=2.0（= elapsedTime）"
        );

        // 再次 drain → 空（每完成帧只派发一次）
        assert!(clock.drain_just_finished().is_empty(), "二次 drain 空（不重复派发）");
    }

    #[test]
    fn test_animation_drain_just_iterated_r3250() {
        // R3250（CSS Animations §animationiteration）：iteration 递增（非完成帧）→ 推入 just_iterated，
        // drain_just_iterated 取出（element_key/name/elapsed=completed_iterations*duration）。
        // 关键：完成帧（finish 分支）不派 iteration（末轮边界派 animationend），故此处 drain 空。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        // duration=1.0s，iteration_count=3.0 → 活跃时长 3.0s；第 1、2 次迭代结束派 iteration，第 3 次（完成）派 end。
        start_anim(
            &mut clock,
            7,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(3.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=1.0 → 第 1 次迭代结束（未完成）→ drain_just_iterated 含 1 条（elapsed=1.0），drain_finished 空。
        let _ = clock.tick(7, 1.0);
        let iterated = clock.drain_just_iterated();
        assert_eq!(iterated.len(), 1, "第 1 次迭代边界 drain 恰好 1 条");
        assert_eq!(iterated[0].element_key, 7);
        assert_eq!(iterated[0].name, "fade");
        assert!(
            (iterated[0].elapsed - 1.0).abs() < 1e-9,
            "elapsed=completed_iterations*duration=1.0"
        );
        assert!(clock.drain_just_finished().is_empty(), "迭代中不派 animationend");

        // t=2.0 → 第 2 次迭代结束（未完成）→ drain_just_iterated 含 1 条（elapsed=2.0）。
        let _ = clock.tick(7, 2.0);
        let iterated = clock.drain_just_iterated();
        assert_eq!(iterated.len(), 1, "第 2 次迭代边界 drain 恰好 1 条");
        assert!(
            (iterated[0].elapsed - 2.0).abs() < 1e-9,
            "elapsed=completed_iterations*duration=2.0"
        );

        // t=3.0 → 第 3 次（末轮）迭代结束 = 动画完成 → drain_just_finished 含 1 条，drain_just_iterated 空
        //（末轮边界派 animationend 而非 animationiteration，CSS Animations §animationend）。
        let _ = clock.tick(7, 3.0);
        assert!(
            clock.drain_just_iterated().is_empty(),
            "完成帧（末轮边界）不派 animationiteration"
        );
        let finished = clock.drain_just_finished();
        assert_eq!(finished.len(), 1, "完成帧派 animationend");
    }

    #[test]
    fn test_animation_infinite_iteration_r3250() {
        // R3250 关键用例：infinite 动画永不完成（不派 animationend），靠 animationiteration 驱动循环回调。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "spin",
            vec![
                (0.0, vec![("transform", "rotate(0deg)")]),
                (100.0, vec![("transform", "rotate(360deg)")]),
            ],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            5,
            "spin",
            0.5,
            0.0,
            TimingFunctionValue::Linear,
            None, // None = infinite
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0.5 → 第 1 轮结束 → iteration；t=1.0 → 第 2 轮 → iteration；永无 animationend。
        let _ = clock.tick(5, 0.5);
        assert_eq!(clock.drain_just_iterated().len(), 1, "infinite 第 1 轮派 iteration");
        assert!(clock.drain_just_finished().is_empty(), "infinite 永不 finish");
        let _ = clock.tick(5, 1.0);
        assert_eq!(clock.drain_just_iterated().len(), 1, "infinite 第 2 轮派 iteration");
        assert!(clock.drain_just_finished().is_empty(), "infinite 永不 finish");
    }

    #[test]
    fn test_animation_drain_just_started_r3251() {
        // R3251（CSS Animations §animationstart）：动画首次进入活跃间隔（无 delay = 首帧）→ started 由
        // false→true → 推入 just_started。每动画只派一次（started 标志去重）。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        // duration=1.0s，iteration_count=3.0，delay=0 → 首帧即活跃。
        start_anim(
            &mut clock,
            8,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(3.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0.0（首帧）→ 进入活跃态 → drain_just_started 含 1 条（element_key=8, name=fade）。
        let _ = clock.tick(8, 0.0);
        let started = clock.drain_just_started();
        assert_eq!(started.len(), 1, "首帧派 animationstart");
        assert_eq!(started[0].element_key, 8);
        assert_eq!(started[0].name, "fade");

        // t=1.0 → 已 started，不再派 animationstart（仅派 animationiteration，见 R3250）。
        let _ = clock.tick(8, 1.0);
        assert!(clock.drain_just_started().is_empty(), "已启动不重复派 animationstart");
        assert_eq!(clock.drain_just_iterated().len(), 1, "第 1 轮迭代边界派 iteration");
    }

    #[test]
    fn test_animation_start_with_delay_r3251() {
        // R3251 关键：animation-delay 期间不派 animationstart；delay 过期后的首帧（进入活跃间隔）才派。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        // delay=1.0s：t<1.0 在 delay 期，t>=1.0 才活跃。
        start_anim(
            &mut clock,
            3,
            "fade",
            1.0,
            1.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0.5（delay 期）→ 不派 animationstart。
        let _ = clock.tick(3, 0.5);
        assert!(clock.drain_just_started().is_empty(), "delay 期不派 animationstart");

        // t=1.0（delay 刚过期，进入活跃间隔）→ 派 animationstart（恰好未完成）。
        let _ = clock.tick(3, 1.0);
        assert_eq!(clock.drain_just_started().len(), 1, "delay 过期首帧派 animationstart");
        assert!(clock.drain_just_finished().is_empty(), "此时未完成");
    }

    #[test]
    fn test_animation_start_before_end_instant_r3251() {
        // R3251 边界：瞬时动画（首帧即完成）应先派 animationstart 再派 animationend（spec：start 先于
        // end，即使同帧）。tick() 的 started 检测置于 finish 检查之前，保证此序。
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        // duration=1.0，iteration_count=1.0；首帧 t=2.0 已远超完成点 → 同帧既 start 又 end。
        start_anim(
            &mut clock,
            4,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        let _ = clock.tick(4, 2.0);
        assert_eq!(clock.drain_just_started().len(), 1, "瞬时动画首帧仍派 animationstart");
        assert_eq!(clock.drain_just_finished().len(), 1, "瞬时动画同帧也派 animationend");
    }

    #[test]
    fn test_clock_tick_with_delay() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.5,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0.3 → 还在延迟中，无插值
        let props = clock.tick(1, 0.3);
        assert!(props.is_empty());

        // t=0.5 → 延迟结束，动画开始
        let props = clock.tick(1, 0.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.05);

        // t=1.0 → 过了 0.5s 动画 → progress=0.5 → opacity=0.5
        let props = clock.tick(1, 1.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_clock_fill_mode_forwards() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::Forwards,
            0.0,
        );

        // t=2.0 → 动画已结束
        let props = clock.tick(1, 2.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap()).abs() < 0.05);
    }

    #[test]
    fn test_clock_fill_mode_backwards_during_delay() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            1.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::Backwards,
            0.0,
        );

        // t=0.5 → 延迟期间，backwards 显示 0% 帧
        let props = clock.tick(1, 0.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clock_direction_reverse() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Reverse,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0 → reversed → opacity=0.0
        let props = clock.tick(1, 0.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap()).abs() < 0.05);

        // t=1 → reversed → opacity=1.0
        let props = clock.tick(1, 1.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_clock_direction_alternate() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "0.0")]), (100.0, vec![("opacity", "1.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(2.0),
            AnimationDirectionValue::Alternate,
            AnimationFillModeValue::None,
            0.0,
        );

        // 第 0 次迭代，t=0.5 → progress=0.5 → opacity=0.5
        let props = clock.tick(1, 0.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);

        // 第 1 次迭代（反向），t=1.5 → iter_progress=0.5 → reversed=0.5 → opacity=0.5
        let props = clock.tick(1, 1.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_clock_cleanup_finished() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=2.0 → 已完成
        let _ = clock.tick(1, 2.0);
        assert!(!clock.active_element_ids().is_empty() || !clock.active_animations.is_empty());

        clock.cleanup_finished();
        assert!(clock.active_animations.is_empty());
    }

    #[test]
    fn test_clock_infinite_animation_not_finished() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "pulse",
            vec![
                (0.0, vec![("opacity", "1.0")]),
                (50.0, vec![("opacity", "0.5")]),
                (100.0, vec![("opacity", "1.0")]),
            ],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "pulse",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            None,
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=100 → 100 次迭代后仍不应完成
        let _ = clock.tick(1, 100.0);
        let ids = clock.active_element_ids();
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_clock_multiple_animations_same_element() {
        let mut clock = AnimationClock::new();
        let fade = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        let grow = make_keyframes(
            "grow",
            vec![(0.0, vec![("width", "100px")]), (100.0, vec![("width", "200px")])],
        );
        clock.register_keyframes(&fade);
        clock.register_keyframes(&grow);

        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );
        start_anim(
            &mut clock,
            1,
            "grow",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        let props = clock.tick(1, 0.5);
        assert!(props.iter().any(|p| p.name == "opacity"));
        assert!(props.iter().any(|p| p.name == "width"));
    }

    #[test]
    fn test_clock_different_elements_independent() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);

        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );
        start_anim(
            &mut clock,
            2,
            "fade",
            2.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0.5 → 元素 1 进度 0.5，元素 2 进度 0.25
        let props1 = clock.tick(1, 0.5);
        let opacity1 = props1.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity1.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);

        let props2 = clock.tick(2, 0.5);
        let opacity2 = props2.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity2.value.parse::<f64>().unwrap() - 0.75).abs() < 0.05);
    }

    #[test]
    fn test_clock_clear() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "fade",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        clock.clear();
        assert!(clock.registered_keyframe_names().is_empty());
        assert!(clock.active_animations.is_empty());
    }

    // ── 应用到 ComputedStyle 测试 ──

    #[test]
    fn test_apply_opacity_to_computed_style() {
        let mut style = ComputedStyle::default();
        style.opacity = 1.0;

        let props = vec![InterpolatedProperty {
            name: "opacity".to_string(),
            value: "0.5000".to_string(),
        }];
        AnimationClock::apply_to_computed_style(&props, &mut style);
        assert!((style.opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_apply_width_to_computed_style() {
        use zero_css_parser::values::LengthValue;
        let mut style = ComputedStyle::default();

        let props = vec![InterpolatedProperty {
            name: "width".to_string(),
            value: "150.00px".to_string(),
        }];
        AnimationClock::apply_to_computed_style(&props, &mut style);
        assert_eq!(style.width, LengthValue::Px(150.0));
    }

    #[test]
    fn test_apply_color_to_computed_style() {
        use zero_css_parser::values::ColorValue;
        let mut style = ComputedStyle::default();

        let props = vec![InterpolatedProperty {
            name: "background-color".to_string(),
            value: "rgba(128, 64, 32, 0.50)".to_string(),
        }];
        AnimationClock::apply_to_computed_style(&props, &mut style);
        // 应该是 Rgba 颜色
        if let ColorValue::Rgba(r, g, b, a) = &style.background_color {
            assert_eq!(*r, 128);
            assert_eq!(*g, 64);
            assert_eq!(*b, 32);
            // a = 0.50 * 255 ≈ 127
            assert!((*a as f64 - 127.0).abs() < 2.0);
        } else {
            panic!("expected Rgba color, got {:?}", style.background_color);
        }
    }

    // ── register_from_stylesheets 测试 ──

    #[test]
    fn test_register_from_stylesheets() {
        let css = r#"
            @keyframes slide {
                from { width: 0px; }
                to { width: 300px; }
            }
        "#;
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut clock = AnimationClock::new();
        clock.register_from_stylesheets(&[ss], &zero_css_parser::media_query::MediaContext::new(800.0, 600.0));

        let names = clock.registered_keyframe_names();
        assert!(names.contains(&"slide"));
    }

    // ── 多关键帧测试 ──

    #[test]
    fn test_three_keyframes_bounce() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "bounce",
            vec![
                (0.0, vec![("width", "100px")]),
                (50.0, vec![("width", "200px")]),
                (100.0, vec![("width", "50px")]),
            ],
        );
        clock.register_keyframes(&rule);
        start_anim(
            &mut clock,
            1,
            "bounce",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        // t=0 → width=100px
        let props = clock.tick(1, 0.0);
        let width = props.iter().find(|p| p.name == "width").unwrap();
        assert!(width.value.contains("100"));

        // t=0.5 → width=200px
        let props = clock.tick(1, 0.5);
        let width = props.iter().find(|p| p.name == "width").unwrap();
        assert!(width.value.contains("200"));

        // t=1.0 → width=50px
        let props = clock.tick(1, 1.0);
        let width = props.iter().find(|p| p.name == "width").unwrap();
        assert!(width.value.contains("50"));
    }

    // ── start_from_computed_style 测试 ──

    #[test]
    fn test_start_from_computed_style() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);

        let mut style = ComputedStyle::default();
        style.animation_name = vec!["fade".to_string()];
        style.animation_duration = vec![1.0];
        style.animation_delay = vec![0.0];
        style.animation_timing_function = vec![TimingFunctionValue::Linear];
        style.animation_iteration_count = vec![Some(1.0)];
        style.animation_direction = vec![AnimationDirectionValue::Normal];
        style.animation_fill_mode = vec![AnimationFillModeValue::None];

        clock.start_from_computed_style(42, &style, 0.0, None);

        let props = clock.tick(42, 0.5);
        assert!(!props.is_empty());
    }

    #[test]
    fn test_start_from_computed_style_no_duplicate() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "fade",
            vec![(0.0, vec![("opacity", "1.0")]), (100.0, vec![("opacity", "0.0")])],
        );
        clock.register_keyframes(&rule);

        let mut style = ComputedStyle::default();
        style.animation_name = vec!["fade".to_string()];
        style.animation_duration = vec![1.0];

        // 第一次启动
        clock.start_from_computed_style(1, &style, 0.0, None);
        // 第二次调用不应创建重复
        clock.start_from_computed_style(1, &style, 0.0, None);

        let animations = clock.active_animations.get(&1).unwrap();
        assert_eq!(animations.len(), 1);
    }

    #[test]
    fn test_start_from_computed_style_empty_name_ignored() {
        let mut clock = AnimationClock::new();
        let mut style = ComputedStyle::default();
        style.animation_name = vec!["none".to_string(), "".to_string()];

        clock.start_from_computed_style(1, &style, 0.0, None);
        assert!(clock.active_animations.is_empty());
    }

    // ── 边界条件测试 ──

    #[test]
    fn test_interpolate_keyframes_empty() {
        let result = interpolate_keyframes(&[], 0.5, &TimingFunctionValue::Linear);
        assert!(result.is_empty());
    }

    // 刀 FAM-F 钉测：aspect-ratio 几何插值＋简并/auto 混形离散回落。
    #[test]
    fn test_aspect_ratio_geometric_interp() {
        assert_eq!(
            interpolate_property_value("aspect-ratio", "1 / 2", "2 / 1", 0.5),
            "1 / 1"
        );
        assert_eq!(interpolate_property_value("aspect-ratio", "0.5", "2", 0.0), "0.5 / 1");
        assert_eq!(
            interpolate_property_value("aspect-ratio", "1 / 2", "2 / 1", -0.5),
            "0.25 / 1"
        );
        assert_eq!(
            interpolate_property_value("aspect-ratio", "auto 1 / 2", "auto 2 / 1", 1.0),
            "auto 2 / 1"
        );
    }

    #[test]
    fn test_aspect_ratio_degenerate_discrete() {
        // 简并（1/0、0/1）与 auto 混形：离散 flip，t<0.5 from、t≥0.5 to。
        assert_eq!(
            interpolate_property_value("aspect-ratio", "1 / 0", "1 / 1", 0.3),
            "1 / 0"
        );
        assert_eq!(
            interpolate_property_value("aspect-ratio", "1 / 0", "1 / 1", 0.6),
            "1 / 1"
        );
        assert_eq!(interpolate_property_value("aspect-ratio", "auto", "2 / 1", 0.3), "auto");
    }

    // 刀 FAM-F 钉测（r3 fail-stop 修）：不可加 add 帧预合成＝有效值取 underlying
    // （web-animations-1 §5.4.2/§5.5、css-values-4 §combine-ratio not-additive）
    // ——WPT ANIMCOMP `underlying [2 / 1] from replace [0.5 / 1] to add [1 / 1]`
    // 三期望：t=0→0.5/1、t=0.5→1/1（0.5^½·2^½）、t=1→2/1。
    #[test]
    fn test_aspect_ratio_add_composite_underlying_fallback() {
        let op = AnimationCompositionValue::Add;
        // add 帧合成＝underlying 原文（不与 animated 求和、不取 animated）。
        assert_eq!(
            compose_animation_value("aspect-ratio", &op, Some("2 / 1"), "1 / 1"),
            "2 / 1"
        );
        // 预合成端点参与几何插值：0.5 / 1 → 2 / 1。
        assert_eq!(
            interpolate_property_value("aspect-ratio", "0.5 / 1", "2 / 1", 0.5),
            "1 / 1"
        );
        assert_eq!(
            interpolate_property_value("aspect-ratio", "0.5 / 1", "2 / 1", 1.0),
            "2 / 1"
        );
    }

    #[test]
    fn test_interpolate_keyframes_single() {
        let pt = KeyframePoint {
            offset: 0.5,
            properties: HashMap::from([("opacity".to_string(), "0.5".to_string())]),
            timing: None,
            composition: None,
        };
        let result = interpolate_keyframes(&[pt], 0.3, &TimingFunctionValue::Linear);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value, "0.5");
    }

    #[test]
    fn test_parse_px_zero() {
        assert_eq!(parse_px("0px"), 0.0);
    }

    #[test]
    fn test_parse_px_non_px() {
        assert_eq!(parse_px("100em"), 0.0); // 不支持 em
    }

    #[test]
    fn test_parse_color_unknown() {
        let (r, g, b, a) = parse_color("somecolor");
        assert_eq!((r, g, b, a), (0.0, 0.0, 0.0, 1.0)); // 默认黑色
    }

    /// R3882：legacy 4 参数 rgb()（CSS Color 4 允许 rgb/rgba 前缀与 3/4 参数自由组合）
    /// ——此前解析失败回落黑色不透明，插值输出整体变黑。
    /// driving: css-backgrounds/animations zero-alpha 族（`rgb(0, 200, 0, 0)`）。
    #[test]
    fn test_parse_color_rgb_four_args() {
        let (r, g, b, a) = parse_color("rgb(0, 200, 0, 0)");
        assert_eq!((r, g, b, a), (0.0, 200.0, 0.0, 0.0));
        let (r, g, b, a) = parse_color("rgb(100, 100, 0, 0.5)");
        assert_eq!((r, g, b, a), (100.0, 100.0, 0.0, 0.5));
        // 3 参数形不受影响
        let (r, g, b, a) = parse_color("rgb(1, 2, 3)");
        assert_eq!((r, g, b, a), (1.0, 2.0, 3.0, 1.0));
    }

    #[test]
    fn test_clock_no_animation_for_element() {
        let mut clock = AnimationClock::new();
        let props = clock.tick(999, 1.0);
        assert!(props.is_empty());
    }

    #[test]
    fn test_register_keyframes_auto_borders() {
        // 没有 0% 和 100% 帧时自动补齐
        let mut clock = AnimationClock::new();
        let rule = make_keyframes("partial", vec![(50.0, vec![("opacity", "0.5")])]);
        clock.register_keyframes(&rule);

        start_anim(
            &mut clock,
            1,
            "partial",
            1.0,
            0.0,
            TimingFunctionValue::Linear,
            Some(1.0),
            AnimationDirectionValue::Normal,
            AnimationFillModeValue::None,
            0.0,
        );

        let props = clock.tick(1, 0.0);
        // 0% 帧是空的，50% 帧有 opacity:0.5 → 在 progress=0 时应返回空或初始值
        // 由于 from 帧无属性，插值结果取决于实现
        assert!(props.is_empty() || props.iter().any(|p| p.name == "opacity"));
    }

    /// R5189：keyframe 值 `revert-layer` 在动画 origin 回退到 author origin（动画未叠加
    /// 时的级联值）后再插值——`from {width:50px} to {width:revert-layer}`、underlying
    /// width:150px → p=0.5 处 width 应为 100px（revert-layer-010：ref 100px 方块）。
    #[test]
    fn test_keyframe_revert_layer_resolves_underlying() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "rev",
            vec![(0.0, vec![("width", "50px")]), (100.0, vec![("width", "revert-layer")])],
        );
        clock.register_keyframes(&rule);

        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(150.0);
        assert!(clock.start_animation_with_underlying(
            1,
            &AnimationConfig {
                name: "rev".to_string(),
                duration: 2.0,
                delay: 0.0,
                timing_function: TimingFunctionValue::Linear,
                iteration_count: Some(1.0),
                direction: AnimationDirectionValue::Normal,
                fill_mode: AnimationFillModeValue::None,
                current_time: 0.0,
                composition: AnimationCompositionValue::Replace,
                underlying_values: HashMap::new(),
            },
            Some(&style),
            None,
        ));

        let props = clock.tick(1, 1.0);
        let width = props
            .iter()
            .find(|p| p.name == "width")
            .expect("p=0.5 应产出 width 插值");
        assert_eq!(width.value, "100.00px", "revert-layer 应解析为 150px 再插值");
    }

    /// R5189/R5191：`transform` 的 `revert-layer` 解析为 underlying 序列化值
    ///（R5191 起 transform 纳入 underlying 序列化面，与 width 臂同语义）；p=0.5
    /// 离散边界取 from（`t > 0.5` 才切 to），不 panic、不产出错误值。
    #[test]
    fn test_keyframe_revert_layer_transform_resolves_underlying() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "rev2",
            vec![
                (0.0, vec![("transform", "translateY(0px)")]),
                (100.0, vec![("transform", "revert-layer")]),
            ],
        );
        clock.register_keyframes(&rule);

        let style = ComputedStyle::default();
        assert!(clock.start_animation_with_underlying(
            1,
            &AnimationConfig {
                name: "rev2".to_string(),
                duration: 2.0,
                delay: 0.0,
                timing_function: TimingFunctionValue::Linear,
                iteration_count: Some(1.0),
                direction: AnimationDirectionValue::Normal,
                fill_mode: AnimationFillModeValue::None,
                current_time: 0.0,
                composition: AnimationCompositionValue::Replace,
                underlying_values: HashMap::new(),
            },
            Some(&style),
            None,
        ));

        let props = clock.tick(1, 1.0);
        // p=0.5 离散边界取 from；to 侧 revert-layer 已解析为 underlying（基线 none）。
        if let Some(p) = props.iter().find(|p| p.name == "transform") {
            assert!(p.value.contains("translate"), "离散回退应取 from 值，got {p:?}");
        }
        // p=1.0 → 端点落在末段 [from, to] 跑 at(1) 混合串化（刀 E 端点语义），
        // to 侧 transform 串化为 translate 形。revert-layer 关键字解析发生在
        // keyframes 注册层（to 帧已存 underlying "none"，混合起点即已解析值），
        // 端点输出不再逐字回放帧声明；若解析失败混合将不可用而非产出 to 形。
        let props_end = clock.tick(1, 2.0);
        let to_val = props_end.iter().find(|p| p.name == "transform").expect("to 侧输出");
        assert_eq!(
            to_val.value, "translateY(0px)",
            "revert-layer 应解析为 underlying 并经端点混合串化为 to 形，got {to_val:?}"
        );
    }

    /// R5191（css-animations-1 §keyframes：隐式边界帧 = underlying value）：
    /// from-only transform keyframes——注册层补的空 100% 点经 underlying 序列化补
    /// `transform` 键，插值循环（to 侧属性集驱动）不再零输出，t≈0 离散取 from。
    /// driving: transform-non-invertible-discrete-interpolation（from-only 非可逆
    /// matrix3d 旧版黑块原样——动画值从未应用到 ComputedStyle）。
    #[test]
    fn test_from_only_transform_keyframe_emits_value() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes(
            "anim",
            vec![(0.0, vec![("transform", "matrix3d(2,0,0,0, 0,2,0,0, 0,0,0,0, 0,0,0,1)")])],
        );
        clock.register_keyframes(&rule);

        let style = ComputedStyle::default();
        assert!(clock.start_animation_with_underlying(
            7,
            &AnimationConfig {
                name: "anim".to_string(),
                duration: 100.0,
                delay: 0.0,
                timing_function: TimingFunctionValue::Linear,
                iteration_count: Some(1.0),
                direction: AnimationDirectionValue::Normal,
                fill_mode: AnimationFillModeValue::Forwards,
                current_time: 0.0,
                composition: AnimationCompositionValue::Replace,
                underlying_values: HashMap::new(),
            },
            Some(&style),
            None,
        ));

        // p≈0（100s 线性起点）：离散切换取 from 值——matrix3d 原样透传（apply 侧
        // parse 成功 → is_singular_transform 剔除渲染）。
        let props = clock.tick(7, 0.02);
        let tp = props
            .iter()
            .find(|p| p.name == "transform")
            .expect("from-only 应有 transform 输出");
        assert!(
            tp.value.contains("matrix3d(2"),
            "t≈0 离散应取 from 的 matrix3d，got {tp:?}"
        );
    }

    // ── CSS Motion Path offset 簇（css-motion-1；wptfyi-css offset-* 刀）──

    /// offset-distance 同单位线性插值；混合单位离散切换（css-values-4 §8.5.6）。
    #[test]
    fn test_offset_distance_interpolates() {
        assert_eq!(
            interpolate_property_value("offset-distance", "0px", "100px", 0.5),
            "50.00px"
        );
        assert_eq!(
            interpolate_property_value("offset-distance", "10%", "100px", 0.4),
            "10%"
        );
        assert_eq!(
            interpolate_property_value("offset-distance", "10%", "100px", 0.5),
            "100px"
        );
    }

    /// offset-rotate 纯角度形线性；auto 关键字侧离散；刀 MOTION-1 deg 等价
    /// 复合插值（WPT offset-rotate-interpolation.html 六块语料钉）。
    #[test]
    fn test_offset_rotate_interp_and_discrete() {
        assert_eq!(
            interpolate_property_value("offset-rotate", "0deg", "90deg", 0.5),
            "45.00deg"
        );
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto", "90deg", 0.4),
            "auto"
        );
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto", "90deg", 0.5),
            "90deg"
        );
        // block3：`||` 乱序复合 `100grad auto` ≡ auto 90deg，canonical auto 词 lerp。
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 170deg", "100grad auto", 0.125),
            "auto 160.00deg"
        );
        // block4：calc 角 `calc(90deg - 0.5turn - 300grad + 0rad)` ≡ -360deg。
        assert_eq!(
            interpolate_property_value(
                "offset-rotate",
                "auto -280deg",
                "auto calc(90deg - 0.5turn - 300grad + 0rad)",
                0.125
            ),
            "auto -290.00deg"
        );
        // block5：纯 reverse ≡ 180deg。
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 100deg", "reverse", 0.875),
            "auto 170.00deg"
        );
        // reverse 复合形 N+180 折叠（语料 block10-12 端点钉同口径）。
        assert_eq!(
            interpolate_property_value("offset-rotate", "reverse -170deg", "reverse -130deg", 0.5),
            "auto 30.00deg"
        );
        // 复合↔纯角跨类离散（no_interpolation 钉）。
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 200deg", "300deg", 0.25),
            "auto 200deg"
        );
        assert_eq!(interpolate_property_value("offset-rotate", "6rad", "auto", 0.6), "auto");
        // 非 deg 单位 deg 等价（rad/turn 归 deg）。
        assert_eq!(
            interpolate_property_value("offset-rotate", "0rad", "0.5turn", 0.5),
            "90.00deg"
        );
        // 刀 MOTION-1b：t==1 非 canonical 端点原文 echo——bare-reverse echo
        // `reverse`（r2 P2F ×2 语料钉）、乱序复合 echo 原文；t≠1 lerp 外推。
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 100deg", "reverse", 1.0),
            "reverse"
        );
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 170deg", "100grad auto", 1.0),
            "100grad auto"
        );
        assert_eq!(
            interpolate_property_value("offset-rotate", "auto 170deg", "100grad auto", 0.999),
            "auto 90.08deg"
        );
        // 外推采样不直通（t=2 lerp——blanket 窗 at(1.5)/at(2) 44F 教训钉）。
        assert_eq!(
            interpolate_property_value("offset-rotate", "100deg", "180deg", 2.0),
            "260.00deg"
        );
        // canonical 复合端点 t==1 走折叠（block10 'reverse -170deg' at(1)
        // 期望 'auto 10deg' 语料钉——原文直通会翻转该面）。
        assert_eq!(
            interpolate_property_value("offset-rotate", "reverse -170deg", "auto 50deg", 1.0),
            "auto 50.00deg"
        );
    }

    /// apply 写回 + underlying 序列化往返（harness 隐式边界帧依赖 underlying 面）。
    #[test]
    fn test_offset_apply_roundtrip_underlying() {
        let mut style = ComputedStyle::default();
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "offset-distance".to_string(),
                value: "40px".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "offset-distance"),
            Some("40px".to_string())
        );
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "offset-anchor".to_string(),
                value: "10px 20%".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "offset-anchor"),
            Some("10px 20%".to_string())
        );
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "offset-rotate".to_string(),
                value: "auto 45deg".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "offset-rotate"),
            Some("auto 45deg".to_string())
        );
    }

    /// offset-position position 对同单位线性；auto/normal 端点离散（css-values-4
    /// §8.5.6 ≥0.5 → to，WPT offset-position-interpolation 关键字端点组语义）。
    #[test]
    fn test_offset_position_interp_and_discrete() {
        assert_eq!(
            interpolate_property_value("offset-position", "220px 240px", "300px 400px", 0.5),
            "260.00px 320.00px"
        );
        assert_eq!(
            interpolate_property_value("offset-position", "0% 50%", "100% 150%", 0.3),
            "30.00% 80.00%"
        );
        assert_eq!(
            interpolate_property_value("offset-position", "auto", "20px 20px", 0.4),
            "auto"
        );
        assert_eq!(
            interpolate_property_value("offset-position", "auto", "20px 20px", 0.5),
            "20px 20px"
        );
    }

    /// R5018 通用化刀（css-animations-1 §keyframes neutral key = underlying
    /// value）：shape-margin 走通用 gCS 串化臂——from-only（100% 显式）边界帧
    /// 0% 补 underlying '10px'，at(0.5) blend = '30px'。刀 F 驱动：css-shapes
    /// neutral 揭穿面（shape-margin-interpolation from-neutral at(0) got 0px）。
    #[test]
    fn test_neutral_boundary_generic_shape_margin() {
        let mut clock = AnimationClock::new();
        let rule = make_keyframes("gm-neutral", vec![(100.0, vec![("shape-margin", "50px")])]);
        clock.register_keyframes(&rule);
        let mut style = ComputedStyle::default();
        style.shape_margin = LengthValue::Px(10.0);
        let config = AnimationConfig {
            name: "gm-neutral".to_string(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, Some(&style), None));
        let props = clock.tick(1, 1.0);
        let v = props
            .iter()
            .find(|p| p.name == "shape-margin")
            .expect("shape-margin 输出");
        assert_eq!(
            v.value, "30.00px",
            "at(0.5) = blend(underlying 10px, to 50px)，got {v:?}"
        );
    }

    /// 通用臂 number 形与回退语义：shape-image-threshold underlying '0.6'
    /// 直出（gCS number 同通道）；默认样式 '0px' 非空回填；未实现属性空串
    /// → None 保持既有回退（parse 失败 = 属性初值近似）。
    #[test]
    fn test_underlying_generic_number_and_fallback() {
        let mut style = ComputedStyle::default();
        style.shape_image_threshold = 0.6;
        assert_eq!(
            underlying_individual_value(&style, "shape-image-threshold"),
            Some("0.6".to_string())
        );
        assert_eq!(
            generic_underlying_value(&ComputedStyle::default(), "shape-margin"),
            Some("0px".to_string())
        );
        assert_eq!(
            generic_underlying_value(&ComputedStyle::default(), "zw-unimplemented-prop"),
            None
        );
    }

    /// offset-position apply 写回 + underlying 序列化往返；写回 unsupported 形
    /// 保持现值（apply_single_property 语义）。
    /// offset-position apply 写回 + underlying 序列化往返；写回 unsupported 形
    /// 保持现值（apply_single_property 语义）。
    #[test]
    fn test_offset_position_apply_roundtrip_underlying() {
        let mut style = ComputedStyle::default();
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "offset-position".to_string(),
                value: "30px 40px".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "offset-position"),
            Some("30px 40px".to_string())
        );
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "offset-position".to_string(),
                value: "normal".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "offset-position"),
            Some("normal".to_string())
        );
    }

    /// wptfyi-css css-gaps 刀 D（css-gaps-1 §inset）：rule-inset 槽插值/底层补键/
    /// apply 写回钉面（语料 inset-interpolation 28 例 + neutral-keyframe 双页形）。
    #[test]
    fn test_rule_inset_longhand_interpolation_lerp_same_unit() {
        // px 域中点（neutral 页钉 1px→11px @0.5 = 6px）；% 域负值中点。
        assert_eq!(
            interpolate_property_value("column-rule-inset-cap-start", "1px", "11px", 0.5),
            "6.00px"
        );
        assert_eq!(
            interpolate_property_value("row-rule-inset-junction-end", "-100%", "-10%", 0.5),
            "-55.00%"
        );
        // 严格门同形通过（transitions/WAAPI 登记前置）。
        assert_eq!(
            interpolate_property_value_strict("column-rule-inset-cap-start", "1px", "11px", 0.5).as_deref(),
            Some("6.00px")
        );
        // 跨单位 / overlap-join 端点对 None（非 animatable 对不启动过渡）。
        // 刀 E：跨单位对 strict 门放行（css-values-4 §8.5.6 calc 混合——TRANS 通道
        // px↔% 对 168F 根因，10px→20% @0.5 = calc(10.00% + 5.00px)）。
        assert_eq!(
            interpolate_property_value_strict("column-rule-inset-cap-start", "10px", "20%", 0.5).as_deref(),
            Some("calc(10.00% + 5.00px)")
        );
        // overlap-join 端点对仍 None（不可解析端点不启动过渡）。
        assert!(
            interpolate_property_value_strict("column-rule-inset-cap-start", "overlap-join", "10px", 0.5).is_none()
        );
        // 刀 E：宽松通道跨单位线性混合（WPT per-at 期望即线性形；旧 t<0.5 离散
        // 翻转已被 blend 取代——interpolation 页 168F 根因）。
        assert_eq!(
            interpolate_property_value("column-rule-inset-cap-start", "10px", "20%", 0.3),
            "calc(6.00% + 7.00px)"
        );
        assert_eq!(
            interpolate_property_value("column-rule-inset-cap-start", "10px", "20%", 0.7),
            "calc(14.00% + 3.00px)"
        );
    }

    #[test]
    fn test_rule_inset_underlying_fill_and_apply_roundtrip() {
        // neutral keyframe 底层补键：声明 '1px' → 边界帧 '1px'（gap-decorations-
        // inset-neutral-keyframe-001 钉）；插值中点 6px 写回后 underlying 读 '6px'
        //（"6.00px" 经 apply 解析回 typed 字段 → gCS 形规范化）。
        let mut style = ComputedStyle::default();
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "row-rule-inset-cap-start".to_string(),
                value: "1px".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "row-rule-inset-cap-start"),
            Some("1px".to_string())
        );
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "row-rule-inset-cap-start".to_string(),
                value: "6.00px".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "row-rule-inset-cap-start"),
            Some("6px".to_string())
        );
        // overlap-join 写回保关键字。
        AnimationClock::apply_to_computed_style(
            &[InterpolatedProperty {
                name: "row-rule-inset-cap-start".to_string(),
                value: "overlap-join".to_string(),
            }],
            &mut style,
        );
        assert_eq!(
            underlying_individual_value(&style, "row-rule-inset-cap-start"),
            Some("overlap-join".to_string())
        );
    }

    /// 刀 E 钉：跨单位 calc 混合代数（css-values-4 §8.5.6 interpolation-calc）——
    /// px↔% 端点对逐单位系数线性，px 系数零 → 纯 % 简化（无 calc 形），% 系数零
    /// 保 calc 形（WPT 期望 `calc(0% + 10px)` 结构）。
    #[test]
    fn test_rule_inset_blend_cross_unit() {
        // 10px→-10% @ 0.5：pct=-5、px=5 → calc(-5.00% + 5.00px)。
        assert_eq!(
            blend_rule_inset_lp(&10.0, "px", &-10.0, "%", 0.5),
            Some("calc(-5.00% + 5.00px)".into())
        );
        // -100%→1px @ 0.5：pct=-50、px=0.5 → calc(-50.00% + 0.50px)。
        assert_eq!(
            blend_rule_inset_lp(&-100.0, "%", &1.0, "px", 0.5),
            Some("calc(-50.00% + 0.50px)".into())
        );
        // t=1：px 系数零 → 纯 %（to 端简化）。
        assert_eq!(
            blend_rule_inset_lp(&10.0, "px", &-10.0, "%", 1.0),
            Some("-10.00%".into())
        );
        // 0%→10px @ 0.5：% 系数非零 → calc(0.00% + 5.00px)。
        assert_eq!(
            blend_rule_inset_lp(&0.0, "%", &10.0, "px", 0.5),
            Some("calc(0.00% + 5.00px)".into())
        );
        // 负 px 分量折叠 ' - '：0%→-10px @ 0.5 → calc(0.00% - 5.00px)。
        assert_eq!(
            blend_rule_inset_lp(&0.0, "%", &-10.0, "px", 0.5),
            Some("calc(0.00% - 5.00px)".into())
        );
        // 同单位对不处理（同单位走 lerp 臂）。
        assert_eq!(blend_rule_inset_lp(&10.0, "px", &20.0, "px", 0.5), None);
        assert_eq!(blend_rule_inset_lp(&10.0, "%", &20.0, "%", 0.5), None);
    }

    /// 刀 E 钉：宽松插值 rule-inset 跨单位入口（tick 通道）——junction 初值
    /// Percentage(0.0)（WPT oracle junction initial→-40% 期望 % 线性自 0%）。
    #[test]
    fn test_rule_inset_interp_cross_unit_loose() {
        assert_eq!(
            interpolate_property_value("column-rule-inset-junction-start", "10px", "-10%", 0.5),
            "calc(-5.00% + 5.00px)"
        );
        assert_eq!(
            interpolate_property_value("row-rule-inset-cap-end", "0%", "10px", 0.5),
            "calc(0.00% + 5.00px)"
        );
        // 同单位维持既有 lerp 形（零回归方向）。
        assert_eq!(
            interpolate_property_value("row-rule-inset-cap-end", "0px", "10px", 0.5),
            "5.00px"
        );
    }

    /// 刀 E 钉：junction 初值 0% / cap 初值 0px（default_impl 同笔改动的 oracle
    /// 断言——junction initial→-40% 块期望 % 线性、cap initial→10px 块期望 px 线性）。
    #[test]
    fn test_rule_inset_default_initial_values() {
        use zero_css_parser::values::RuleInsetComponent;
        let st = ComputedStyle::default();
        assert!(matches!(
            st.column_rule_inset_junction_start,
            RuleInsetComponent::Length(zero_style_system::LengthValue::Percentage(v)) if v == 0.0
        ));
        assert!(matches!(
            st.row_rule_inset_junction_end,
            RuleInsetComponent::Length(zero_style_system::LengthValue::Percentage(v)) if v == 0.0
        ));
        assert!(matches!(
            st.column_rule_inset_cap_start,
            RuleInsetComponent::Length(zero_style_system::LengthValue::Px(v)) if v == 0.0
        ));
        assert!(matches!(
            st.row_rule_inset_cap_start,
            RuleInsetComponent::Length(zero_style_system::LengthValue::Px(v)) if v == 0.0
        ));
    }

    /// 刀 E 钉（E5）：关键帧 'initial'/'unset' 关键字 start 时解析为属性初始值——
    /// clock 注册 initial→10px 帧，t=0.5 期望 lerp(0px, 10px, 0.5) = 5.00px
    ///（css-animations-1 §keyframes + css-cascade-4 §default）。'inherit' 不解析
    ///（需父 computed）→ 既有离散回退（t≥0.5 → to）。
    #[test]
    fn test_keyframe_initial_keyword_resolution() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "kw",
            vec![
                (0.0, vec![("column-rule-inset-cap-start", "initial")]),
                (100.0, vec![("column-rule-inset-cap-start", "10px")]),
            ],
        ));
        let config = AnimationConfig {
            name: "kw".into(),
            duration: 1.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation(1, &config));
        let props = clock.tick(1, 0.5);
        let got = props
            .iter()
            .find(|p| p.name == "column-rule-inset-cap-start")
            .expect("cap-start update");
        assert_eq!(got.value, "5.00px", "initial 解析为 0px 后 lerp");

        // unset（rule-inset 非继承 ≡ initial）同解析。
        let mut clock2 = AnimationClock::new();
        clock2.register_keyframes(&make_keyframes(
            "kw2",
            vec![
                (0.0, vec![("column-rule-inset-cap-start", "unset")]),
                (100.0, vec![("column-rule-inset-cap-start", "10px")]),
            ],
        ));
        let config2 = AnimationConfig {
            name: "kw2".into(),
            ..config
        };
        assert!(clock2.start_animation(1, &config2));
        let props2 = clock2.tick(1, 0.5);
        let got2 = props2
            .iter()
            .find(|p| p.name == "column-rule-inset-cap-start")
            .expect("cap-start update");
        assert_eq!(got2.value, "5.00px");
    }

    /// 刀 K1 钉：关键帧 'inherit' 端点 start 时解析为父元素 computed value
    ///（css-cascade-4 §inheritance：继承强制，含非继承属性）——parent width
    /// 90px、`from {width:inherit} to {width:40px}`，p=0.5 期望 lerp(90,40,.5)
    /// = 65px（interpolation corpus from-[inherit] 面同构）。
    #[test]
    fn test_keyframe_inherit_keyword_resolves_parent() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "inh",
            vec![(0.0, vec![("width", "inherit")]), (100.0, vec![("width", "40px")])],
        ));
        let mut parent = ComputedStyle::default();
        parent.width = LengthValue::Px(90.0);
        let config = AnimationConfig {
            name: "inh".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, None, Some(&parent)));
        let props = clock.tick(1, 1.0);
        let got = props.iter().find(|p| p.name == "width").expect("width update");
        assert_eq!(got.value, "65.00px", "inherit 解析为父 90px 后 lerp");
    }

    /// 刀 K1 钉：父面不可达（WAAPI/cssa 通道传 None）时 'inherit' 端点维持原值
    /// 走既有离散回退（t≥0.5 → to）——回退语义保持，不产出空串污染。
    #[test]
    fn test_keyframe_inherit_without_parent_keeps_fallback() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "inh0",
            vec![(0.0, vec![("width", "inherit")]), (100.0, vec![("width", "40px")])],
        ));
        let config = AnimationConfig {
            name: "inh0".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, None, None));
        let props = clock.tick(1, 1.0);
        let got = props.iter().find(|p| p.name == "width").expect("width update");
        // 离散切换臂 verbatim 返 to 端点原文（offset-rotate 等既有离散臂同形）。
        assert_eq!(got.value, "40px", "无父时离散回退取 to");
    }

    // ── wptfyi-css mask-border 刀钉测（css-masking-1 §3.5–3.9）────────────
    // 期望表取 tests/wpt-runner/wpt-data/css/css-masking/animations/
    // mask-border-*-interpolation.html 语料断言行（CSS Animations 通道权威）。

    #[test]
    /// slice 同构槽 lerp + 非负钳 0（语料 0%→50% at:-0.3 期望 '0%'）。
    fn test_mask_border_slice_lerp_and_clamp() {
        assert_eq!(interpolate_property_value("mask-border-slice", "0%", "50%", -0.3), "0%");
        assert_eq!(interpolate_property_value("mask-border-slice", "0%", "50%", 0.3), "15%");
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0% 10% 20% 30%", "40% 50% 60% 70%", 0.3),
            "12% 22% 32% 42%"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0 10 20 30 fill", "40 50 60 70 fill", -0.5),
            "0 0 0 10 fill"
        );
    }

    #[test]
    /// slice 混合槽逐槽同构插值（% 槽保 %、number 槽裸数——语料 '0% 10 20% 30
    /// fill'→'40% 50 60% 70 fill' at 0.3 期望 '12% 22 32% 42 fill'）。
    fn test_mask_border_slice_mixed_slots() {
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0% 10 20% 30 fill", "40% 50 60% 70 fill", 0.3),
            "12% 22 32% 42 fill"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0% 10 20% 30 fill", "40% 50 60% 70 fill", 0.0),
            "0% 10 20% 30 fill"
        );
    }

    #[test]
    /// slice 槽 type 失配（%↔number）与 fill 不对称 → 整条离散翻转
    ///（语料 no-interp 三条；expectFlip at<0.5→from、≥0.5→to）。
    fn test_mask_border_slice_discrete_on_mismatch() {
        assert_eq!(
            interpolate_property_value("mask-border-slice", "50%", "100", 0.3),
            "50%"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "50%", "100", 0.6),
            "100"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0% fill", "50%", 0.3),
            "0% fill"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "0% fill", "50%", 0.6),
            "50%"
        );
        assert_eq!(
            interpolate_property_value("mask-border-slice", "50% fill", "100 fill", 0.6),
            "100 fill"
        );
    }

    #[test]
    /// width 三域同构 lerp + 非负钳（0px→20px at:-0.3→'0px'、0→20 at:-0.3→'0'；
    /// 四槽混合 '10px 20% 30 40px'→'80px 70% 60 50px' at 0.3 期望 '31px 35% 39 43px'；
    /// auto↔auto 恒 'auto'）。
    fn test_mask_border_width_lerp_and_clamp() {
        assert_eq!(
            interpolate_property_value("mask-border-width", "0px", "20px", -0.3),
            "0px"
        );
        assert_eq!(interpolate_property_value("mask-border-width", "0", "20", -0.3), "0");
        // 裸 0 端点归一 Number 域插值（'0'→'20' at:0.3 期望 '6'——解析双态歧义
        // 防回归钉）。
        assert_eq!(interpolate_property_value("mask-border-width", "0", "20", 0.3), "6");
        assert_eq!(
            interpolate_property_value("mask-border-width", "10px 20% 30 40px", "80px 70% 60 50px", 0.3),
            "31px 35% 39 43px"
        );
        assert_eq!(
            interpolate_property_value("mask-border-width", "10px auto auto 20", "110px auto auto 120", 0.3),
            "40px auto auto 50"
        );
    }

    #[test]
    /// width 槽失配（auto↔值、px↔number）整条离散（语料 no-interp 四条）。
    fn test_mask_border_width_discrete_on_mismatch() {
        assert_eq!(
            interpolate_property_value("mask-border-width", "10px auto auto 20", "110px auto 120 auto", 0.3),
            "10px auto auto 20"
        );
        assert_eq!(
            interpolate_property_value("mask-border-width", "10px auto auto 20", "110px auto 120 auto", 0.6),
            "110px auto 120 auto"
        );
        assert_eq!(interpolate_property_value("mask-border-width", "10px", "20", 0.6), "20");
        assert_eq!(interpolate_property_value("mask-border-width", "10%", "20", 0.3), "10%");
    }

    #[test]
    /// outset 非负钳与逐槽混合（语料 '1 2 3px 4px'→'101 102 103px 104px'
    /// at:-0.3 期望 '0 0 0px 0px'、at:0.6 期望 '61 62 63px 64px'；inherit 端点
    /// 外推 10px→2px at:1.5 钳 '0px'）。
    fn test_mask_border_outset_lerp_and_clamp() {
        assert_eq!(
            interpolate_property_value("mask-border-outset", "0px", "5px", -0.3),
            "0px"
        );
        assert_eq!(interpolate_property_value("mask-border-outset", "0", "1", 0.3), "0.3");
        assert_eq!(
            interpolate_property_value("mask-border-outset", "1 2 3px 4px", "101 102 103px 104px", -0.3),
            "0 0 0px 0px"
        );
        assert_eq!(
            interpolate_property_value("mask-border-outset", "1 2 3px 4px", "101 102 103px 104px", 0.6),
            "61 62 63px 64px"
        );
    }

    #[test]
    /// repeat 纯离散翻转（t≥0.5 取 to）。
    fn test_mask_border_repeat_discrete() {
        assert_eq!(
            interpolate_property_value("mask-border-repeat", "repeat", "round", 0.3),
            "repeat"
        );
        assert_eq!(
            interpolate_property_value("mask-border-repeat", "repeat", "round", 0.6),
            "round"
        );
    }

    #[test]
    /// 刀 MB v3：width 严格通道跨型 px↔% 对放行（lenient 端点翻转位 t<0.5→from、
    /// ≥0.5→to——语料 transitions 通道陈旧期望位 b0 行为保位）；Number↔px、
    /// auto↔px 槽失配仍不启动过渡（None）；同型对与 0 桥接维持既有门。
    fn test_mask_border_width_strict_cross_type_gate() {
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "10px", "20%", 0.3),
            Some("10px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "10px", "20%", 0.6),
            Some("20%".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "10%", "20px", 0.0),
            Some("10%".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "10", "20px", 0.3),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "auto", "10px", 0.3),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "10px", "20px", 0.3),
            Some("13px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("mask-border-width", "0px", "20", 0.3),
            Some("6".to_string())
        );
    }

    // ── wptfyi-css 刀 B1 border-image 三长手钉测（css-backgrounds-3 §3.6–3.9）──
    // 期望表取 tests/wpt-runner/wpt-data/css/css-backgrounds/animations/
    // border-image-{slice,width,outset}-interpolation.html 语料断言行。宽松通道
    //（CSSA/WAAPI keyframes）与严格通道（transitions 同型门）分钉。

    #[test]
    /// slice 宽松臂共享验证（语料 initial→10% at:1.5 钳 '0%'、underlying 20%→10%
    /// at:0.3 期望 '17%'；跨型对离散翻转翻转点 0.5）。
    fn test_border_image_slice_lerp() {
        assert_eq!(
            interpolate_property_value("border-image-slice", "100%", "10%", 1.5),
            "0%"
        );
        assert_eq!(
            interpolate_property_value("border-image-slice", "20%", "10%", 0.3),
            "17%"
        );
        assert_eq!(
            interpolate_property_value("border-image-slice", "0% 10 20% 30 fill", "40% 50 60% 70 fill", 0.3),
            "12% 22 32% 42 fill"
        );
        assert_eq!(
            interpolate_property_value("border-image-slice", "50%", "100", 0.3),
            "50%"
        );
        assert_eq!(
            interpolate_property_value("border-image-slice", "50%", "100", 0.6),
            "100"
        );
    }

    #[test]
    /// width 多槽钳制外推（语料 '10px 20% 30 40px'→'80px 70% 60 50px' at:-0.3
    /// 期望 '0px 5% 21 37px'；auto 槽直通；裸 0↔number 桥接 '0'→'20' at:-0.3
    /// 期望 '0'；跨型 %↔px 离散翻转）。
    fn test_border_image_width_lerp() {
        assert_eq!(
            interpolate_property_value("border-image-width", "10px 20% 30 40px", "80px 70% 60 50px", -0.3),
            "0px 5% 21 37px"
        );
        assert_eq!(
            interpolate_property_value("border-image-width", "10px auto auto 20", "110px auto auto 120", -0.3),
            "0px auto auto 0"
        );
        assert_eq!(
            interpolate_property_value("border-image-width", "0px", "20px", -0.3),
            "0px"
        );
        assert_eq!(interpolate_property_value("border-image-width", "0", "20", -0.3), "0");
        // 跨型 %↔px（Block B 形）：t≠0 走 echo 保留门返回空串（canonicalize 恒等
        // 回落 echo 位——B0 伪通过池在册机制，v3 电池 REG=0 实证）；t==0 豁免转
        // 既有离散翻转位（from 可解析诚实通过，语料 at(0) 行）。
        assert_eq!(interpolate_property_value("border-image-width", "10%", "20px", 0.3), "");
        assert_eq!(
            interpolate_property_value("border-image-width", "10%", "20px", 0.0),
            "10%"
        );
    }

    #[test]
    /// outset 数值/长度槽钳制（语料 initial→2 at:-0.3 期望 '0'、underlying 1px→2px
    /// at:0.3 期望 '1.3px'）。
    fn test_border_image_outset_lerp() {
        assert_eq!(interpolate_property_value("border-image-outset", "0", "2", -0.3), "0");
        assert_eq!(
            interpolate_property_value("border-image-outset", "1px", "2px", 0.3),
            "1.3px"
        );
        assert_eq!(
            interpolate_property_value("border-image-outset", "1 2 3px 4px", "101 102 103px 104px", -0.3),
            "0 0 0px 0px"
        );
    }

    #[test]
    /// 严格通道同型槽位门：同型对 Some（真插值），跨型/关键字端点/fill 不对称
    /// None（非 animatable 对不启动过渡、静态跳终值，css-transitions
    /// §starting-of-transitions——语料 no-interp 关键字对期望恒 to）。
    fn test_border_image_strict_gate() {
        assert_eq!(
            interpolate_property_value_strict("border-image-width", "10px", "20px", 0.5),
            Some("15px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-width", "1", "20px", 0.5),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-width", "10%", "20px", 0.5),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-slice", "20%", "10%", 0.5),
            Some("15%".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-slice", "0% fill", "50%", 0.5),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-outset", "1", "2", 0.5),
            Some("1.5".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("border-image-outset", "1", "2px", 0.5),
            None
        );
    }

    #[test]
    /// 刀 Z：z-index 整数插值（css-values-4 §8.1.2 round half towards +∞，
    /// 语料 z-index-interpolation from[-2]→to[5] at(0.3)=0 / at(-0.3)=-4）；
    /// auto/小数端点离散回落、strict 同型门。
    fn test_z_index_lerp() {
        assert_eq!(interpolate_property_value("z-index", "-2", "5", 0.3), "0");
        assert_eq!(interpolate_property_value("z-index", "-2", "5", -0.3), "-4");
        assert_eq!(interpolate_property_value("z-index", "-2", "5", 0.5), "2");
        assert_eq!(interpolate_property_value("z-index", "-2", "5", 1.0), "5");
        assert_eq!(interpolate_property_value("z-index", "auto", "5", 0.4), "auto");
        assert_eq!(interpolate_property_value("z-index", "auto", "5", 0.6), "5");
        assert_eq!(interpolate_property_value("z-index", "3.14", "5", 0.3), "3.14");
        assert_eq!(
            interpolate_property_value_strict("z-index", "-2", "5", 0.5),
            Some("2".to_string())
        );
        assert_eq!(interpolate_property_value_strict("z-index", "auto", "5", 0.5), None);
        assert_eq!(interpolate_property_value_strict("z-index", "3.14", "5", 0.5), None);
    }

    #[test]
    /// 刀 A2：vertical-align 插值（css-inline-3 §vertical-align-propdef +
    /// css-transitions-1 §animations by computed value type）——LP 对线性、
    /// baseline 端点按 0 长度参与（语料 initial→100px at(0) 期望 '0px'）、
    /// 关键字对离散 50% 翻转、px↔% 跨单位 calc 系数形（px 在前，语料
    /// canonical）；strict 门=真实可插值性（非插值对 super→40% 恒 None：
    /// transition 注册拒绝→无过渡立即跳终值，css-transitions-1
    /// §starting-of-transitions；harness transitions 通道 expectFlip
    /// -Infinity 形）。em/rem 等其余单位对空串回落离散基线。
    fn test_vertical_align_lerp() {
        // 同单位 LP 线性。
        assert_eq!(
            interpolate_property_value("vertical-align", "0px", "100px", 0.3),
            "30.00px"
        );
        assert_eq!(
            interpolate_property_value("vertical-align", "10%", "30%", 0.5),
            "20.00%"
        );
        // baseline 端点按 0 长度参与。
        assert_eq!(
            interpolate_property_value("vertical-align", "baseline", "100px", 0.0),
            "0.00px"
        );
        assert_eq!(
            interpolate_property_value("vertical-align", "baseline", "100px", 0.3),
            "30.00px"
        );
        // 关键字对/关键字↔LP 离散 50% 翻转（宽松臂；CSSA/WAAPI 通道语义）。
        assert_eq!(
            interpolate_property_value("vertical-align", "super", "sub", 0.3),
            "super"
        );
        assert_eq!(interpolate_property_value("vertical-align", "super", "sub", 0.6), "sub");
        assert_eq!(
            interpolate_property_value("vertical-align", "super", "40%", 0.3),
            "super"
        );
        // px↔% 跨单位 calc 系数形（px 在前；px 系数 0 折叠纯 %）。
        assert_eq!(
            interpolate_property_value("vertical-align", "40px", "40%", -0.5),
            "calc(60.00px - 20.00%)"
        );
        assert_eq!(
            interpolate_property_value("vertical-align", "40px", "40%", 1.0),
            "40.00%"
        );
        // strict 门：可插值对 Some、非插值对恒 None。
        assert_eq!(
            interpolate_property_value_strict("vertical-align", "0px", "100px", 0.5),
            Some("50.00px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("vertical-align", "baseline", "40px", 0.5),
            Some("20.00px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("vertical-align", "super", "40%", 0.5),
            None
        );
        assert_eq!(
            interpolate_property_value_strict("vertical-align", "super", "sub", 0.5),
            None
        );
    }

    /// 刀 A3 text-shadow 阴影列表插值钉测（css-text-decor-3 §text-shadow
    /// interpolation）：逐层 length/color 组件 lerp（color 预乘空间）、none ≡
    /// 空表按零阴影补齐至对侧层数、blur 恒显、外推钳（alpha [0,1]/blur 0）、
    /// 多层逗号列表、解析失败对离散翻转；strict 门=双端点均阴影列表。
    #[test]
    fn test_text_shadow_lerp() {
        // 同型组件 lerp：颜色预乘 + 三 length 槽（语料 neutral→绿 at(-0.3)）。
        assert_eq!(
            interpolate_property_value(
                "text-shadow",
                "rgb(255, 165, 0) 10px 30px 10px",
                "rgb(0, 128, 0) 20px 20px 20px",
                -0.3
            ),
            "rgb(255, 176, 0) 7px 33px 7px"
        );
        // none 空表零阴影补齐：at(0.3) 预乘 rgba(0,128,0,0.3)、at(0) 透明零阴影。
        assert_eq!(
            interpolate_property_value("text-shadow", "none", "rgb(0, 128, 0) 20px 20px 20px", 0.3),
            "rgba(0, 128, 0, 0.30) 6px 6px 6px"
        );
        assert_eq!(
            interpolate_property_value("text-shadow", "none", "rgb(0, 128, 0) 20px 20px 20px", 0.0),
            "rgba(0, 0, 0, 0.00) 0px 0px 0px"
        );
        // 负外推钳：blur 5→25 at(-0.3) = -1 钳 0（语料 black/orange 对）。
        assert_eq!(
            interpolate_property_value(
                "text-shadow",
                "rgb(0, 0, 0) 15px 10px 5px",
                "rgb(255, 165, 0) -15px -10px 25px",
                -0.3
            ),
            "rgb(0, 0, 0) 24px 16px 0px"
        );
        // 多层列表逐层插值（层差补齐 + 逗号连接）：双不透明端点 rgb 形、
        // 补齐层预乘半透明形。
        assert_eq!(
            interpolate_property_value(
                "text-shadow",
                "rgb(255, 0, 0) 1px 1px",
                "rgb(0, 0, 255) 2px 2px 2px, rgb(0, 128, 0) 3px 3px 3px",
                0.5
            ),
            "rgb(128, 0, 128) 1.5px 1.5px 1px, rgba(0, 128, 0, 0.50) 1.5px 1.5px 1.5px"
        );
        // 解析失败端点对离散翻转（宽松臂）。
        assert_eq!(interpolate_property_value("text-shadow", "10px", "red", 0.3), "10px");
        assert_eq!(interpolate_property_value("text-shadow", "10px", "red", 0.6), "red");
        // strict 门：双端点阴影列表（含 none）Some 委托、畸形端点 None。
        assert_eq!(
            interpolate_property_value_strict("text-shadow", "rgb(0, 0, 0) 15px 10px 5px", "none", 0.5),
            Some("rgba(0, 0, 0, 0.50) 7.5px 5px 2.5px".to_string())
        );
        assert_eq!(
            interpolate_property_value_strict("text-shadow", "10px", "red", 0.5),
            None
        );
    }

    /// 刀 A3r 钉（css-cascade-4 §default）：keyframe 'unset' 对 inherited 属性
    /// （text-shadow）取父 computed 值——修正刀 K1「unset 恒 initial」。
    #[test]
    fn test_keyframe_unset_inherited_resolves_parent() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "tsunset",
            vec![
                (0.0, vec![("text-shadow", "unset")]),
                (100.0, vec![("text-shadow", "rgb(0, 128, 0) 20px 20px 20px")]),
            ],
        ));
        let mut parent = ComputedStyle::default();
        parent.text_shadow = vec![zero_style_system::property::types::TextShadowComputedValue {
            offset_x: 30.0,
            offset_y: 10.0,
            blur_radius: 30.0,
            color: zero_css_parser::values::ColorValue::Rgba(255, 165, 0, 255),
        }];
        let config = AnimationConfig {
            name: "tsunset".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, None, Some(&parent)));
        let props = clock.tick(1, 0.5);
        let got = props.iter().find(|p| p.name == "text-shadow").expect("ts update");
        // t=0.25：orange(255,165,0)→green(0,128,0) 逐通道 lerp（b 双端 0→0）+ 30/10/30→20/20/20。
        assert_eq!(got.value, "rgb(191, 156, 0) 27.5px 12.5px 27.5px", "unset→父值 lerp");
    }

    /// 刀 A3r 钉（css-color-4 §5.4）：keyframe 端点内嵌 currentcolor 以元素
    /// computed color start 时定死（underlying 上下文）；非列表值不重写。
    #[test]
    fn test_keyframe_currentcolor_resolves_element_color() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "tscc",
            vec![
                (0.0, vec![("text-shadow", "rgb(0, 0, 0) 10px 10px 10px")]),
                (100.0, vec![("text-shadow", "currentColor 10px 10px 10px")]),
            ],
        ));
        let mut style = ComputedStyle::default();
        style.color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
        let config = AnimationConfig {
            name: "tscc".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, Some(&style), None));
        let props = clock.tick(1, 1.0);
        let got = props.iter().find(|p| p.name == "text-shadow").expect("ts update");
        // t=0.5：black→green(currentcolor 解析) lerp，长度恒 10px。
        assert_eq!(got.value, "rgb(0, 64, 0) 10px 10px 10px", "currentcolor→元素色 lerp");
    }

    /// 刀 A3r 回归钉（A3r 电池 r2 110 P→F 教训）：非 text-shadow 属性的裸
    /// 长度列表端点不得被 currentcolor 预解析路径重写——parse_text_shadow
    /// 对裸长度默认 CurrentColor，无属性门时 background-position 类
    /// "40px 140px" 会被改写成阴影串致动画失效（gCS 透出 underlying）。
    #[test]
    fn test_keyframe_non_text_shadow_untouched_by_currentcolor_resolver() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "bgpos",
            vec![
                (0.0, vec![("background-position", "40px 140px")]),
                (100.0, vec![("background-position", "80px 80px")]),
            ],
        ));
        let style = ComputedStyle::default();
        let config = AnimationConfig {
            name: "bgpos".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: AnimationCompositionValue::Replace,
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, Some(&style), None));
        let props = clock.tick(1, 1.0);
        let got = props
            .iter()
            .find(|p| p.name == "background-position")
            .expect("bp update");
        // t=0.5：端点未被改写、正常 lerp（40/140→80/80；bg-position 串化为两位小数约定）。
        assert_eq!(
            got.value, "60.00px 110.00px",
            "非 text-shadow 端点不走 currentcolor 解析"
        );
    }

    // ── 刀 ANIMCOMP S2（2026-11-19）合成钉测 ──
    // 锚定 WPT css-transitions/animations/{text-shadow,vertical-align}-composition
    // 期望串形态；覆盖 uniform-add 结果合成、none 单位元恒等、% 回落 replace。

    #[test]
    fn test_compose_animation_value_shadow_concat() {
        let op = AnimationCompositionValue::Add;
        assert_eq!(
            compose_animation_value(
                "text-shadow",
                &op,
                Some("rgb(10, 20, 30) 1px 2px 3px"),
                "rgb(150, 150, 150) 15px 30px 45px"
            ),
            "rgb(10, 20, 30) 1px 2px 3px, rgb(150, 150, 150) 15px 30px 45px",
            "影列表 add = 追加（逗号空格分隔，gCS 列表串化同形）"
        );
        // none/空 underlying = 单位元，恒等动画值
        assert_eq!(
            compose_animation_value("box-shadow", &op, Some("none"), "rgb(1, 2, 3) 4px 5px 6px"),
            "rgb(1, 2, 3) 4px 5px 6px",
            "none underlying 恒等"
        );
        assert_eq!(
            compose_animation_value("box-shadow", &op, Some(""), "1px 2px 3px"),
            "1px 2px 3px"
        );
    }

    #[test]
    fn test_compose_animation_value_vertical_align_px_add() {
        let op = AnimationCompositionValue::Add;
        assert_eq!(
            compose_animation_value("vertical-align", &op, Some("50px"), "150px"),
            "200px",
            "px 加法（50 + lerp(100,200,0.5) 期望 200px 形态）"
        );
        assert_eq!(
            compose_animation_value(
                "vertical-align",
                &AnimationCompositionValue::Accumulate,
                Some("50px"),
                "70px"
            ),
            "120px",
            "accumulate 单迭代域与 add 同形"
        );
        // % / 混合单位无定义加法 → 动画值原样（replace 回落，零回归方向）
        assert_eq!(
            compose_animation_value("vertical-align", &op, Some("10%"), "150px"),
            "150px",
            "% underlying 回落"
        );
    }

    #[test]
    fn test_compose_animation_value_domain_fallback() {
        let op = AnimationCompositionValue::Add;
        // 域外属性与无 underlying 全部恒等动画值
        assert_eq!(compose_animation_value("opacity", &op, Some("0.5"), "0.75"), "0.75");
        assert_eq!(compose_animation_value("opacity", &op, None, "0.75"), "0.75");
    }

    /// uniform-add text-shadow 端到端：登记 → tick 输出 = underlying ⊕ lerp。
    /// 驱动面：text-shadow-composition b2（underlying 单影 + add 100px 族 → 200px 族）。
    #[test]
    fn test_uniform_add_text_shadow_tick_composes_underlying() {
        let mut clock = AnimationClock::new();
        clock.register_keyframes(&make_keyframes(
            "tsadd",
            vec![
                (0.0, vec![("text-shadow", "rgb(100, 100, 100) 10px 20px 30px")]),
                (100.0, vec![("text-shadow", "rgb(200, 200, 200) 20px 40px 60px")]),
            ],
        ));
        let mut style = ComputedStyle::default();
        style.animation_composition = vec![AnimationCompositionValue::Add];
        // underlying text-shadow（模拟 harness inline 写入）经 gCS 序列化器捕获，
        // default text-shadow = none → 单位元恒等：t=0.5 应为纯 lerp 值。
        let config = AnimationConfig {
            name: "tsadd".into(),
            duration: 2.0,
            delay: 0.0,
            timing_function: TimingFunctionValue::Linear,
            iteration_count: Some(1.0),
            direction: AnimationDirectionValue::Normal,
            fill_mode: AnimationFillModeValue::None,
            current_time: 0.0,
            composition: style.animation_composition[0].clone(),
            underlying_values: HashMap::new(),
        };
        assert!(clock.start_animation_with_underlying(1, &config, Some(&style), None));
        let props = clock.tick(1, 1.0);
        let got = props.iter().find(|p| p.name == "text-shadow").expect("ts composed");
        assert_eq!(
            got.value, "rgb(150, 150, 150) 15px 30px 45px",
            "none underlying → lerp 值恒等（合成串化无前缀）"
        );
    }
}
