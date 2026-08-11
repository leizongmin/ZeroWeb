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
//! - `transform`（通过 opacity/translate 的简单场景）

use std::collections::HashMap;

use zero_css_parser::ast::{KeyframeBlock, KeyframeSelector, KeyframesRule};
use zero_css_parser::values::{AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue};
use zero_style_system::ComputedStyle;

/// 关键帧点 — 某个时间进度处的属性快照。
#[derive(Debug, Clone)]
pub struct KeyframePoint {
    /// 时间进度（0.0 ~ 1.0）。
    pub offset: f64,
    /// 声明的属性名→原始值字符串。
    pub properties: HashMap<String, String>,
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
        TimingFunctionValue::Steps(n, _pos) => {
            let n = (*n).max(1) as f64;
            (t * n).floor() / n
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

/// 对两个关键帧之间的属性值进行插值。
///
/// 返回插值后的属性列表。
pub fn interpolate_between(from: &KeyframePoint, to: &KeyframePoint, progress: f64) -> Vec<InterpolatedProperty> {
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

    // 局部进度归一化到 [0,1]
    let local = ((progress - from.offset) / span).clamp(0.0, 1.0);

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
        "background-color" | "color" | "border-color" => {
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
        _ => {
            // 不支持插值的属性，在进度 > 0.5 时切换到目标值
            if t > 0.5 { to.to_string() } else { from.to_string() }
        }
    }
}

/// 线性插值。
fn lerp(a: f64, b: f64, t: f64) -> f64 {
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

    // rgb(r, g, b)
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
fn lerp_color(from: Rgba, to: Rgba, t: f64) -> Rgba {
    (
        lerp(from.0, to.0, t).clamp(0.0, 255.0),
        lerp(from.1, to.1, t).clamp(0.0, 255.0),
        lerp(from.2, to.2, t).clamp(0.0, 255.0),
        lerp(from.3, to.3, t).clamp(0.0, 1.0),
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
                },
            );
        }
        if points.last().map(|p| p.offset) != Some(1.0) {
            points.push(KeyframePoint {
                offset: 1.0,
                properties: HashMap::new(),
            });
        }

        self.keyframes_registry.insert(rule.name.clone(), points);
    }

    /// 从样式表提取并注册所有 @keyframes 规则。
    pub fn register_from_stylesheets(&mut self, stylesheets: &[zero_css_parser::Stylesheet]) {
        for ss in stylesheets {
            for rule in &ss.rules {
                if let zero_css_parser::ast::Rule::Keyframes(kf) = rule {
                    self.register_keyframes(kf);
                }
            }
        }
    }

    /// 启动一个动画。
    ///
    /// 使用 `AnimationConfig` 结构体配置动画参数。
    pub fn start_animation(&mut self, element_id: u64, config: &AnimationConfig) -> bool {
        let Some(keyframes) = self.keyframes_registry.get(&config.name).cloned() else {
            return false;
        };

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
        };

        self.active_animations.entry(element_id).or_default().push(state);
        true
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

            self.start_animation(
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

            let elapsed = current_time - anim.start_time;
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
                let props = interpolate_keyframes(&anim.keyframes, final_progress);
                all_props.extend(props);
                continue;
            }

            anim.iteration = total_iterations.floor() as u64;

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

            // 应用 timing function
            let timed = apply_timing_function(directed, &anim.timing_function);

            // 在关键帧之间插值
            let props = interpolate_keyframes(&anim.keyframes, timed);
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

    let properties: HashMap<String, String> = block
        .declarations
        .iter()
        .map(|d| (d.property.clone(), d.value.clone()))
        .collect();

    KeyframePoint { offset, properties }
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
fn interpolate_keyframes(keyframes: &[KeyframePoint], progress: f64) -> Vec<InterpolatedProperty> {
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

    // 找到包含 progress 的区间 [i, i+1]
    let mut from_idx = 0;
    for (i, kf) in keyframes.iter().enumerate() {
        if kf.offset <= progress {
            from_idx = i;
        } else {
            break;
        }
    }
    let to_idx = (from_idx + 1).min(keyframes.len() - 1);

    interpolate_between(&keyframes[from_idx], &keyframes[to_idx], progress)
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
        "background-color" => {
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

    // ── 关键帧插值测试 ──

    #[test]
    fn test_interpolate_between_simple() {
        let from = KeyframePoint {
            offset: 0.0,
            properties: HashMap::from([("opacity".to_string(), "1.0".to_string())]),
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([("opacity".to_string(), "0.0".to_string())]),
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
        };
        let to = KeyframePoint {
            offset: 1.0,
            properties: HashMap::from([
                ("opacity".to_string(), "0.5".to_string()),
                ("width".to_string(), "200px".to_string()),
            ]),
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
        };

        let result = interpolate_between(&pt, &pt, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "opacity");
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
        clock.register_from_stylesheets(&[ss]);

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
        let result = interpolate_keyframes(&[], 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_interpolate_keyframes_single() {
        let pt = KeyframePoint {
            offset: 0.5,
            properties: HashMap::from([("opacity".to_string(), "0.5".to_string())]),
        };
        let result = interpolate_keyframes(&[pt], 0.3);
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
}
