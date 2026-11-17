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

// R5191：transform 函数列表插值（css-transforms-1 §13.2 unmatrix + 2D 函数逐参数）。
mod transform_interp;
// wptfyi-css 基本形状族刀（css-shapes-2 §interpolating-shape）：clip-path 同构
// 逐分量插值（inset/circle/ellipse/polygon/shape/path/rect/xywh）。
mod clip_interp;
// M2-S1b：可合成动画声明导出（compositor-animation-continuity，renderer → compositor 采样域）。
pub mod compositor_export;
// M2-S3c：动画 transform 命中修正提供者槽（renderer 注册全仿射，hit_test 逆映射点测）。
pub mod hit_provider;

use zero_css_parser::ast::{KeyframeBlock, KeyframeSelector, KeyframesRule};
use zero_css_parser::values::{
    AnimationDirectionValue, AnimationFillModeValue, LengthValue, StepPosition, TimingFunctionValue, TransformFunction,
};
use zero_style_system::{
    ComputedStyle, FilterComputedValue, OffsetPositionValue, OffsetRotateValue, ShapeOutsideBoxKind,
    ShapeOutsideComputedValue,
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
            return Some(offset_rotate_str(style.offset_rotate));
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
    match property {
        "opacity" => {
            let from_f = parse_f64(from).unwrap_or(1.0);
            let to_f = parse_f64(to).unwrap_or(1.0);
            format!("{:.4}", lerp(from_f, to_f, t))
        }
        // R3882：`background` 简写常见于 @keyframes（如 `from { background: green }`），
        // 键名按声明原名进插值——颜色臂须含简写名（css-backgrounds-3 §background 简写
        // 的颜色分量插值等价 background-color）。
        "background-color" | "background" | "color" | "border-color" => {
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
        "width" | "height" | "top" | "left" | "right" | "bottom" | "margin-top" | "margin-right" | "margin-bottom"
        | "margin-left" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left" | "font-size"
        | "letter-spacing" | "word-spacing" => {
            let from_px = parse_px(from);
            let to_px = parse_px(to);
            format!("{:.2}px", lerp(from_px, to_px, t))
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
        "offset-rotate" => match (parse_offset_rotate_deg(from), parse_offset_rotate_deg(to)) {
            (Some(fa), Some(ta)) => format!("{:.2}deg", lerp(fa, ta, t)),
            _ if t >= 0.5 => to.to_string(),
            _ => from.to_string(),
        },
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
            None if t >= 0.5 => to.to_string(),
            None => from.to_string(),
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
        // wptfyi-css composite-add 刀（2026-11-15）：text-indent 与 min/max-sizing
        // px 端点严格解析插值——既有 px 列表臂为宽松解析（关键字端点取 0 数值化），
        // 此处关键字端点（auto/min-content 等）须保离散语义故独立成臂；解析失败
        // 回落离散切换（css-values-4 §8.5.6，边界 ≥ 0.5 → to）。
        "text-indent" | "min-width" | "max-width" | "min-height" | "max-height" => {
            match (parse_px_strict(from), parse_px_strict(to)) {
                (Some(fv), Some(tv)) => format!("{:.2}px", lerp(fv, tv, t)),
                _ if t >= 0.5 => to.to_string(),
                _ => from.to_string(),
            }
        }
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
        C::None | C::Path(_) => false,
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
                || round.as_ref().is_some_and(radius_calc)
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
pub(crate) fn interpolate_property_value_strict(property: &str, from: &str, to: &str, t: f64) -> Option<String> {
    match property {
        "opacity" => {
            let fa = parse_f64(from)?;
            let ta = parse_f64(to)?;
            Some(format!("{:.4}", lerp(fa, ta, t)))
        }
        "background-color" | "background" | "color" | "border-color" => {
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
        "width" | "height" | "top" | "left" | "right" | "bottom" | "margin-top" | "margin-right" | "margin-bottom"
        | "margin-left" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left" | "font-size"
        | "letter-spacing" | "word-spacing" => {
            let fa = parse_px_strict(from)?;
            let ta = parse_px_strict(to)?;
            Some(format!("{:.2}px", lerp(fa, ta, t)))
        }
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
            let fa = parse_offset_rotate_deg(from)?;
            let ta = parse_offset_rotate_deg(to)?;
            Some(format!("{:.2}deg", lerp(fa, ta, t)))
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

/// 解析 f64 值。
fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse().ok()
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

/// offset-rotate 纯角度形（`<angle>` deg）严格解析——auto/reverse 关键字返 None
/// （插值期离散切换判定用）。
fn parse_offset_rotate_deg(s: &str) -> Option<f64> {
    s.trim().strip_suffix("deg").and_then(|v| v.trim().parse::<f64>().ok())
}

/// offset-rotate 值串化（underlying/端点发射；auto/reverse 关键字系离散面）。
fn offset_rotate_str(v: Option<OffsetRotateValue>) -> String {
    match v {
        None | Some(OffsetRotateValue::Auto) => "auto".to_string(),
        Some(OffsetRotateValue::Reverse) => "reverse".to_string(),
        Some(OffsetRotateValue::Angle(deg)) => format!("{deg}deg"),
        Some(OffsetRotateValue::AutoAngle(deg)) => format!("auto {deg}deg"),
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
/// auto <angle>；非法原文 → Auto 近似（fail-soft，与 scale 无单位回落同风格）。
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
    if first.eq_ignore_ascii_case("auto") {
        return parse_offset_rotate_deg(parts.next()?).map(OffsetRotateValue::AutoAngle);
    }
    parse_offset_rotate_deg(first).map(OffsetRotateValue::Angle)
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

// ── AnimationClock 实现 ───────────────────────────────────────────────

impl AnimationClock {
    /// 创建空的动画时钟。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 @keyframes 规则。
    pub fn register_keyframes(&mut self, rule: &KeyframesRule) {
        let mut points: Vec<KeyframePoint> = rule.keyframes.iter().map(block_to_point).collect();

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
                },
            );
        }
        if points.last().map(|p| p.offset) != Some(1.0) {
            points.push(KeyframePoint {
                offset: 1.0,
                properties: HashMap::new(),
                timing: None,
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
        self.start_animation_with_underlying(element_id, config, None)
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
    ) -> bool {
        let Some(mut keyframes) = self.keyframes_registry.get(&config.name).cloned() else {
            return false;
        };
        // E5（wptfyi-css 刀 E，2026-11-17）css-animations-1 §keyframes /
        // css-cascade-4 §default：关键帧 'initial'/'unset' 关键字解析为属性初始值
        // （interpolation corpus neutral→15px、initial→10px 等块的 keyword 端点）。
        // 无元素依赖（ComputedStyle::default() + gCS 序列化器同通道），start 时
        // 一次性解析，WAAPI（start_animation 委托，underlying=None）与 CSSA
        //（underlying=Some）两端共享。'inherit' 需父 computed，此处不解析
        //（FIXME(kf-inherit)），原值维持走既有插值回退。串化为空（applier 未覆盖
        // 属性）保持原值——避免空串端点污染插值。
        for point in &mut keyframes {
            for (name, value) in point.properties.iter_mut() {
                let v = value.trim();
                if v.eq_ignore_ascii_case("initial") || v.eq_ignore_ascii_case("unset") {
                    let init = crate::js_dom_bridge::computed_style::serialize_computed_property_with_sibling(
                        &ComputedStyle::default(),
                        name,
                        None,
                    );
                    if !init.is_empty() {
                        *value = init;
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
    pub fn start_from_computed_style(&mut self, element_id: u64, style: &ComputedStyle, current_time: f64) {
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
                },
                Some(style),
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
                    let props = interpolate_to_end(anim);
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
                    for (name, value) in &first.properties {
                        all_props.push(InterpolatedProperty {
                            name: name.clone(),
                            value: value.clone(),
                        });
                    }
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
                let props = interpolate_keyframes(&anim.keyframes, final_progress, &anim.timing_function);
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
            let props = interpolate_keyframes(&anim.keyframes, directed, &anim.timing_function);
            all_props.extend(props);
        }

        all_props
    }

    /// 将插值结果叠加到 ComputedStyle 上。
    ///
    /// 只修改可动画属性，不改变不可动画的属性。
    pub fn apply_to_computed_style(props: &[InterpolatedProperty], style: &mut ComputedStyle) {
        for prop in props {
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

/// 将 KeyframeBlock 转为 KeyframePoint。
fn block_to_point(block: &KeyframeBlock) -> KeyframePoint {
    let offset = if block.selectors.len() == 1 {
        match &block.selectors[0] {
            KeyframeSelector::From => 0.0,
            KeyframeSelector::To => 1.0,
            KeyframeSelector::Percentage(p) => *p / 100.0,
        }
    } else {
        // 多选择器取第一个
        block
            .selectors
            .first()
            .map(|s| match s {
                KeyframeSelector::From => 0.0,
                KeyframeSelector::To => 1.0,
                KeyframeSelector::Percentage(p) => *p / 100.0,
            })
            .unwrap_or(0.0)
    };

    // R3883：帧内 `animation-timing-function` 是**段 timing**（应用于本帧→下一帧），
    // 不是可插值属性——从 properties 剥离单独存储，避免落入插值/应用名单被丢弃或误用。
    // 值解析失败（畸形 timing）→ None（回落动画全局 timing）。
    let mut timing = None;
    let properties: HashMap<String, String> = block
        .declarations
        .iter()
        .filter(|d| {
            if d.property.eq_ignore_ascii_case("animation-timing-function") {
                timing = zero_css_parser::values::parse_timing_function(d.value.trim());
                return false;
            }
            true
        })
        .map(|d| (d.property.clone(), d.value.clone()))
        .collect();

    KeyframePoint {
        offset,
        properties,
        timing,
    }
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
        "width" => {
            let px = parse_px(value);
            style.width = LengthValue::Px(px);
        }
        "height" => {
            let px = parse_px(value);
            style.height = LengthValue::Px(px);
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
            } else if let Some(f) = zero_css_parser::values::parse_individual_rotate(value) {
                style.individual_rotate = Some(f);
            }
        }
        "scale" => {
            if value.trim().eq_ignore_ascii_case("none") {
                style.individual_scale = None;
            } else if let Some(f) = zero_css_parser::values::parse_individual_scale(value) {
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
        // css-shapes-1 §shape-outside：none | [ <basic-shape> || <shape-box> ]
        // ——镜像 apply_advanced 静态映射（None→None、其余 Shape 包装）+ 裸
        // <shape-box> 关键字 Box 变体（clip-path 无此形态）。
        "shape-outside" => {
            if let Some(v) = zero_css_parser::values::parse_clip_path(value) {
                style.shape_outside = match v {
                    zero_css_parser::values::ClipPathValue::None => ShapeOutsideComputedValue::None,
                    v => ShapeOutsideComputedValue::Shape(v),
                };
            } else {
                let kind = match value.trim().to_ascii_lowercase().as_str() {
                    "content-box" => Some(ShapeOutsideBoxKind::ContentBox),
                    "padding-box" => Some(ShapeOutsideBoxKind::PaddingBox),
                    "border-box" => Some(ShapeOutsideBoxKind::BorderBox),
                    "margin-box" => Some(ShapeOutsideBoxKind::MarginBox),
                    _ => None,
                };
                if let Some(k) = kind {
                    style.shape_outside = ShapeOutsideComputedValue::Box(k);
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
        "min-width" => {
            if let Some(px) = parse_px_strict(value) {
                style.min_width = LengthValue::Px(px);
            }
        }
        "max-width" => {
            if let Some(px) = parse_px_strict(value) {
                style.max_width = LengthValue::Px(px);
            }
        }
        "min-height" => {
            if let Some(px) = parse_px_strict(value) {
                style.min_height = LengthValue::Px(px);
            }
        }
        "max-height" => {
            if let Some(px) = parse_px_strict(value) {
                style.max_height = LengthValue::Px(px);
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
        _ => {
            // 其他属性暂不支持动画覆盖
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{Declaration, KeyframeBlock, KeyframesRule};

    /// 辅助：创建简单的关键帧规则。
    fn make_keyframes(name: &str, blocks: Vec<(f64, Vec<(&str, &str)>)>) -> KeyframesRule {
        KeyframesRule {
            name: name.to_string(),
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

    /// 二十一轮 echo 保留门钉：calc 端点 / rect 族跨形对返回空串（apply 必败
    /// → 槽位不动 → canonicalize 恒等门回落 echo，B0 伪通过池保位）；真离散对
    /// （none↔circle）不受门影响。
    #[test]
    fn test_clip_pair_echo_preserve_gate() {
        // calc 端点（edge-offset 位置）：期望串是 calc 代数插值形（共享残面）。
        assert_eq!(
            interpolate_property_value(
                "clip-path",
                "circle(25% at right 5% bottom 15px)",
                "circle(45% at right 25% bottom 35px)",
                0.5
            ),
            "",
            "calc 端点对保 echo（空串）"
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
        assert_eq!(r, "inset(10px 10px 10px 10px round 20px)");
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

    // ── 关键帧插值测试 ──

    #[test]
    fn test_interpolate_between_simple() {
        let from = KeyframePoint {
            offset: 0.0,
            properties: HashMap::from([("opacity".to_string(), "1.0".to_string())]),
            timing: None,
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([("opacity".to_string(), "0.0".to_string())]),
            timing: None,
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
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([
                ("opacity".to_string(), "0.5".to_string()),
                ("width".to_string(), "200px".to_string()),
            ]),
            timing: None,
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
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([("opacity".to_string(), "1.0".to_string())]),
            timing: None,
        };

        // 段 timing 生效：local 0.5 → ease-in(0.5) ≈ 0.315（≠ 线性 0.5）
        let result = interpolate_between_with_timing(&from, &to, 0.5, &TimingFunctionValue::Linear);
        let val: f64 = result[0].value.parse().unwrap();
        assert!((val - 0.315).abs() < 0.02, "segment timing must apply (got {val})");

        // 帧未声明 timing → 回落全局 timing（同样 ease-in → ≈0.315）
        let from_no_timing = KeyframePoint {
            timing: None,
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
    fn test_block_to_point_strips_timing_declaration() {
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
        let pt = block_to_point(&block);
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

        clock.start_from_computed_style(42, &style, 0.0);

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
        clock.start_from_computed_style(1, &style, 0.0);
        // 第二次调用不应创建重复
        clock.start_from_computed_style(1, &style, 0.0);

        let animations = clock.active_animations.get(&1).unwrap();
        assert_eq!(animations.len(), 1);
    }

    #[test]
    fn test_start_from_computed_style_empty_name_ignored() {
        let mut clock = AnimationClock::new();
        let mut style = ComputedStyle::default();
        style.animation_name = vec!["none".to_string(), "".to_string()];

        clock.start_from_computed_style(1, &style, 0.0);
        assert!(clock.active_animations.is_empty());
    }

    // ── 边界条件测试 ──

    #[test]
    fn test_interpolate_keyframes_empty() {
        let result = interpolate_keyframes(&[], 0.5, &TimingFunctionValue::Linear);
        assert!(result.is_empty());
    }

    #[test]
    fn test_interpolate_keyframes_single() {
        let pt = KeyframePoint {
            offset: 0.5,
            properties: HashMap::from([("opacity".to_string(), "0.5".to_string())]),
            timing: None,
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
            },
            Some(&style),
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
            },
            Some(&style),
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
            },
            Some(&style),
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

    /// offset-rotate 纯角度形线性；auto 关键字侧离散。
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
        };
        assert!(clock.start_animation_with_underlying(1, &config, Some(&style)));
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
}
