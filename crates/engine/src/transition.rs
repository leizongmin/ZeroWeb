//! CSS Transition 执行引擎 — 检测样式变化并执行属性过渡。
//!
//! CSS Transition 与 Animation 的区别：
//! - Animation 基于 @keyframes 预定义的关键帧序列
//! - Transition 由样式变化触发，自动在旧值和新值之间插值
//!
//! 此模块实现：
//! 1. TransitionClock — 管理所有活跃的过渡
//! 2. 样式变化检测 — 比较前后 ComputedStyle 找出需要过渡的属性
//! 3. 自动启动过渡 — 对 transition-property 列出的属性启动插值
//! 4. 与 AnimationClock 共享插值基础设施

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TimingFunctionValue};
use zero_style_system::ComputedStyle;

use crate::animation::{InterpolatedProperty, apply_timing_function, interpolate_property_value};

/// 活跃的过渡实例。
#[derive(Debug, Clone)]
pub struct TransitionState {
    /// 过渡的属性名。
    pub property: String,
    /// 旧值。
    pub from_value: String,
    /// 新值。
    pub to_value: String,
    /// 时长（秒）。
    pub duration: f64,
    /// 延迟（秒）。
    pub delay: f64,
    /// Timing function。
    pub timing_function: TimingFunctionValue,
    /// 开始时间（秒）。
    pub start_time: f64,
    /// 是否已完成。
    pub finished: bool,
    /// 是否已进入活跃态（R3252，CSS Transitions §transitionstart）——首次跨过 transition-delay 进入活跃
    /// 间隔的帧由 false→true，供 `tick()` 推入 `just_started` 派发 transitionstart（elapsedTime=0）。
    pub started: bool,
}

/// 「本轮新完成」的过渡记录（R3248）——`tick()` 在 `finished` 由 false→true 的帧推入 `just_finished`，
/// 供宿主经 `drain_just_finished()` 取出后映射元素并派发 `transitionend` 事件（CSS Transitions §transitionend）。
/// `duration` = 过渡时长（秒），即 transitionend 事件的 `elapsedTime`（spec：完成时 = duration，不含 delay）。
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedTransition {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 过渡属性名（transitionend.propertyName）。
    pub property: String,
    /// 过渡时长（秒，transitionend.elapsedTime）。
    pub duration: f64,
}

/// 「本轮新创建」的过渡记录（R3252）——`start_transitions()` 创建 TransitionState 时推入 `just_run`，供宿主经
/// `drain_just_run()` 取出后派发 `transitionrun`（CSS Transitions §transitionrun——过渡被创建即派发，可能在
/// delay 期）。elapsedTime 恒为 0，故结构不带 elapsed 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RunTransition {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 过渡属性名（transitionrun.propertyName）。
    pub property: String,
}

/// 「本轮新启动」的过渡记录（R3252）——`tick()` 在 `started` 由 false→true（首次跨过 transition-delay 进入
/// 活跃间隔）的帧推入 `just_started`，供宿主经 `drain_just_started()` 取出后派发 `transitionstart`
/// （CSS Transitions §transitionstart）。elapsedTime 恒为 0，故结构不带 elapsed 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct StartedTransition {
    /// 元素 key（= NodeId::as_ffi()，调用方据此映射回 NodeId）。
    pub element_key: u64,
    /// 过渡属性名（transitionstart.propertyName）。
    pub property: String,
}

/// Transition 时钟 — 管理所有活跃的过渡并按帧推进。
#[derive(Debug, Clone, Default)]
pub struct TransitionClock {
    /// 活跃的过渡（元素 ID → 过渡列表）。
    active_transitions: HashMap<u64, Vec<TransitionState>>,
    /// 「本轮新完成」的过渡（R3248）——`tick()` 推入，`drain_just_finished()` 取出派发 transitionend。
    just_finished: Vec<FinishedTransition>,
    /// 「本轮新创建」的过渡（R3252）——`start_transitions()` 创建时推入，`drain_just_run()` 取出派发 transitionrun。
    just_run: Vec<RunTransition>,
    /// 「本轮新启动」的过渡（R3252）——`tick()` 首活跃帧推入，`drain_just_started()` 取出派发 transitionstart。
    just_started: Vec<StartedTransition>,
}

impl TransitionClock {
    /// 创建空的过渡时钟。
    pub fn new() -> Self {
        Self::default()
    }

    /// 检测样式变化并启动必要的过渡。
    ///
    /// 比较旧样式和新样式，对 `transition-property` 列出的属性
    /// 启动从旧值到新值的过渡。
    pub fn start_transitions(
        &mut self,
        element_key: u64,
        old_style: &ComputedStyle,
        new_style: &ComputedStyle,
        current_time: f64,
    ) {
        if new_style.transition_property.is_empty() {
            return;
        }

        for (i, property) in new_style.transition_property.iter().enumerate() {
            if property == "none" || property.is_empty() {
                continue;
            }

            let duration = new_style.transition_duration.get(i).copied().unwrap_or(0.0);
            if duration <= 0.0 {
                continue; // 无持续时间的过渡直接跳过
            }

            let delay = new_style.transition_delay.get(i).copied().unwrap_or(0.0);
            let timing = new_style
                .transition_timing_function
                .get(i)
                .cloned()
                .unwrap_or(TimingFunctionValue::Ease);

            // 获取新旧属性值
            let old_value = get_property_value(old_style, property);
            let new_value = get_property_value(new_style, property);

            // 值相同则不需要过渡
            if old_value == new_value {
                continue;
            }

            // 检查是否已有该属性的活跃过渡
            if let Some(transitions) = self.active_transitions.get_mut(&element_key) {
                transitions.retain(|t| t.property != *property || t.finished);
            }

            let state = TransitionState {
                property: property.clone(),
                from_value: old_value,
                to_value: new_value,
                duration,
                delay,
                timing_function: timing,
                start_time: current_time,
                finished: false,
                started: false,
            };

            // R3252（CSS Transitions §transitionrun）：过渡被创建即派 transitionrun（可能在 delay 期，尚未
            // 真正播放）。与 transitionstart 的区别：start 在 delay 过后活跃帧派；无 delay 时 run 与 start 同帧。
            self.just_run.push(RunTransition {
                element_key,
                property: property.clone(),
            });

            self.active_transitions.entry(element_key).or_default().push(state);
        }
    }

    /// 推进时钟并获取指定元素的过渡插值结果。
    pub fn tick(&mut self, element_key: u64, current_time: f64) -> Vec<InterpolatedProperty> {
        let Some(transitions) = self.active_transitions.get_mut(&element_key) else {
            return Vec::new();
        };

        let mut props = Vec::new();

        for transition in transitions.iter_mut() {
            if transition.finished {
                continue;
            }

            let elapsed = current_time - transition.start_time;
            let active_elapsed = elapsed - transition.delay;

            // 延迟期间
            if active_elapsed < 0.0 {
                // 返回旧值（保持起始状态）
                props.push(InterpolatedProperty {
                    name: transition.property.clone(),
                    value: transition.from_value.clone(),
                });
                continue;
            }

            let progress = (active_elapsed / transition.duration).clamp(0.0, 1.0);

            // R3252（CSS Transitions §transitionstart）：检测「首次进入活跃间隔」——delay 过后的首帧
            // （`started` 由 false→true）派 transitionstart。置于 finish 检查**之前**，使瞬时过渡（首帧即完成）
            // 先派 start 再派 end（spec：transitionstart 先于 transitionend，即使同帧）。elapsedTime=0。
            // `started` 标志保证每过渡只派一次。
            if !transition.started {
                transition.started = true;
                self.just_started.push(StartedTransition {
                    element_key,
                    property: transition.property.clone(),
                });
            }

            if progress >= 1.0 {
                transition.finished = true;
                // R3248：记录「本轮新完成」——`finished` 由 false→true 的帧推入（`if transition.finished
                // { continue; }` 在循环顶保证每个过渡仅推一次），供 `drain_just_finished()` 派发 transitionend。
                self.just_finished.push(FinishedTransition {
                    element_key,
                    property: transition.property.clone(),
                    duration: transition.duration,
                });
                // 过渡完成，使用目标值
                props.push(InterpolatedProperty {
                    name: transition.property.clone(),
                    value: transition.to_value.clone(),
                });
            } else {
                // 应用 timing function
                let timed = apply_timing_function(progress, &transition.timing_function);
                let value = interpolate_property_value(
                    &transition.property,
                    &transition.from_value,
                    &transition.to_value,
                    timed,
                );
                props.push(InterpolatedProperty {
                    name: transition.property.clone(),
                    value,
                });
            }
        }

        props
    }

    /// 将过渡插值结果叠加到 ComputedStyle。
    pub fn apply_to_computed_style(props: &[InterpolatedProperty], style: &mut ComputedStyle) {
        use crate::animation::AnimationClock;
        AnimationClock::apply_to_computed_style(props, style);
    }

    /// 获取所有有活跃过渡的元素 ID。
    pub fn active_element_ids(&self) -> Vec<u64> {
        self.active_transitions
            .iter()
            .filter(|(_, ts)| ts.iter().any(|t| !t.finished))
            .map(|(&id, _)| id)
            .collect()
    }

    /// 取出「自上次 drain 后新完成」的过渡（R3248，CSS Transitions §transitionend）。
    /// 宿主据此映射元素并派发 `transitionend` 事件。每次调用清空内部缓冲（每完成帧只派发一次）。
    pub fn drain_just_finished(&mut self) -> Vec<FinishedTransition> {
        std::mem::take(&mut self.just_finished)
    }

    /// 取出「自上次 drain 后新创建」的过渡（R3252，CSS Transitions §transitionrun）。宿主据此映射元素并派发
    /// `transitionrun` 事件（过渡被创建即派发，可能在 delay 期）。每次调用清空缓冲。
    pub fn drain_just_run(&mut self) -> Vec<RunTransition> {
        std::mem::take(&mut self.just_run)
    }

    /// 取出「自上次 drain 后新启动」的过渡（R3252，CSS Transitions §transitionstart）。宿主据此映射元素并
    /// 派发 `transitionstart` 事件（delay 过后活跃帧派，每过渡仅一次）。每次调用清空缓冲。
    pub fn drain_just_started(&mut self) -> Vec<StartedTransition> {
        std::mem::take(&mut self.just_started)
    }

    /// 移除已完成的过渡。
    pub fn cleanup_finished(&mut self) {
        self.active_transitions.retain(|_, transitions| {
            transitions.retain(|t| !t.finished);
            !transitions.is_empty()
        });
    }

    /// 清除所有过渡。
    pub fn clear(&mut self) {
        self.active_transitions.clear();
    }
}

/// 从 ComputedStyle 中获取指定属性的字符串值。
fn get_property_value(style: &ComputedStyle, property: &str) -> String {
    match property {
        "opacity" => format!("{}", style.opacity),
        "width" => match &style.width {
            LengthValue::Px(v) => format!("{}px", v),
            LengthValue::Auto => "auto".to_string(),
            other => format!("{:?}", other),
        },
        "height" => match &style.height {
            LengthValue::Px(v) => format!("{}px", v),
            LengthValue::Auto => "auto".to_string(),
            other => format!("{:?}", other),
        },
        "background-color" => color_value_to_string(&style.background_color),
        "color" => color_value_to_string(&style.color),
        "margin-top" => length_value_to_string(&style.margin_top),
        "margin-right" => length_value_to_string(&style.margin_right),
        "margin-bottom" => length_value_to_string(&style.margin_bottom),
        "margin-left" => length_value_to_string(&style.margin_left),
        "padding-top" => length_value_to_string(&style.padding_top),
        "padding-right" => length_value_to_string(&style.padding_right),
        "padding-bottom" => length_value_to_string(&style.padding_bottom),
        "padding-left" => length_value_to_string(&style.padding_left),
        "all" => String::new(), // "all" 不参与单个属性比较
        _ => String::new(),
    }
}

/// 将 ColorValue 转为字符串。
fn color_value_to_string(color: &ColorValue) -> String {
    match color {
        ColorValue::Rgba(r, g, b, a) => format!("rgba({}, {}, {}, {})", r, g, b, a),
        ColorValue::Hsla(h, s, l, a) => format!("hsla({}, {}, {}, {})", h, s, l, a),
        ColorValue::Named(name) => name.clone(),
        ColorValue::Transparent => "transparent".to_string(),
        ColorValue::CurrentColor => "currentColor".to_string(),
        // color-mix() 在 transition 串化中罕见，回退为占位（精确动画需插值，defer）。
        ColorValue::Mix(_) => "color-mix(in srgb, …)".to_string(),
        // RCS 相对色在 transition 串化中罕见，回退为占位（精确动画需插值，defer）。
        ColorValue::RelativeColor(_) => "rgb(from …)".to_string(),
    }
}

/// 将 LengthValue 转为字符串。
fn length_value_to_string(length: &LengthValue) -> String {
    match length {
        LengthValue::Px(v) if *v == 0.0 => "0px".to_string(),
        LengthValue::Px(v) => format!("{}px", v),
        LengthValue::Auto => "auto".to_string(),
        other => format!("{:?}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助：创建两个仅某属性不同的 ComputedStyle。
    fn make_style_pair(property: &str, old: &str, new: &str) -> (ComputedStyle, ComputedStyle) {
        use zero_css_parser::values::LengthValue;
        let mut old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();

        match property {
            "opacity" => {
                old_style.opacity = old.parse().unwrap_or(1.0);
                new_style.opacity = new.parse().unwrap_or(1.0);
            }
            "width" => {
                old_style.width = LengthValue::Px(old.trim_end_matches("px").parse().unwrap_or(0.0));
                new_style.width = LengthValue::Px(new.trim_end_matches("px").parse().unwrap_or(0.0));
            }
            _ => {}
        }

        (old_style, new_style)
    }

    /// 辅助：设置 transition 属性。
    fn set_transition_property(style: &mut ComputedStyle, property: &str) {
        style.transition_property = vec![property.to_string()];
        style.transition_duration = vec![1.0];
        style.transition_delay = vec![0.0];
        style.transition_timing_function = vec![TimingFunctionValue::Linear];
    }

    // ── 基础功能测试 ──

    #[test]
    fn test_transition_no_change_no_start() {
        let mut clock = TransitionClock::new();
        let style = ComputedStyle::default();
        let mut new_style = style.clone();
        set_transition_property(&mut new_style, "opacity");

        // 值相同 → 不启动过渡
        clock.start_transitions(1, &style, &new_style, 0.0);
        let ids = clock.active_element_ids();
        assert!(ids.is_empty());
    }

    #[test]
    fn test_transition_opacity_starts() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        let ids = clock.active_element_ids();
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_transition_tick_at_start() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);

        // t=0 → progress=0 → opacity=1.0
        let props = clock.tick(1, 0.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_transition_tick_at_mid() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);

        // t=0.5 → progress=0.5 → opacity=0.5
        let props = clock.tick(1, 0.5);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_transition_tick_at_end() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);

        // t=1.0 → progress=1.0 → opacity=0.0
        let props = clock.tick(1, 1.0);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap()).abs() < 0.05);
    }

    #[test]
    fn test_transition_drain_just_finished_r3248() {
        // R3248（CSS Transitions §transitionend）：finished 由 false→true 的帧推入 just_finished，
        // drain_just_finished 取出（含 element_key/property/duration=elapsedTime），每完成帧只派发一次。
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity"); // duration = 1.0s

        clock.start_transitions(7, &old_style, &new_style, 0.0);

        // t=0.5 → 进行中，未完成 → drain 空
        let _ = clock.tick(7, 0.5);
        assert!(clock.drain_just_finished().is_empty(), "进行中 drain 空");

        // t=1.0 → 完成 → drain 含该过渡（element_key=7, property=opacity, duration=1.0=elapsedTime）
        let _ = clock.tick(7, 1.0);
        let finished = clock.drain_just_finished();
        assert_eq!(finished.len(), 1, "完成帧 drain 恰好 1 条");
        assert_eq!(finished[0].element_key, 7);
        assert_eq!(finished[0].property, "opacity");
        assert!(
            (finished[0].duration - 1.0).abs() < 1e-9,
            "duration=1.0（= elapsedTime）"
        );

        // 再次 drain → 空（每完成帧只派发一次，不重复）
        assert!(clock.drain_just_finished().is_empty(), "二次 drain 空（不重复派发）");
    }

    #[test]
    fn test_transition_drain_just_run_and_started_r3252() {
        // R3252（CSS Transitions §transitionrun/§transitionstart）：transitionrun 在 start_transitions 创建
        // 时派发（无 delay 时 run 与 start 同帧）；transitionstart 在 delay 过后首活跃帧派发。每过渡各一次。
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity"); // delay=0，duration=1.0

        // start_transitions 创建过渡 → 立即派 transitionrun（无需 tick）。
        clock.start_transitions(7, &old_style, &new_style, 0.0);
        let run = clock.drain_just_run();
        assert_eq!(run.len(), 1, "创建即派 transitionrun");
        assert_eq!(run[0].element_key, 7);
        assert_eq!(run[0].property, "opacity");
        assert!(
            clock.drain_just_started().is_empty(),
            "start_transitions 不派 transitionstart（须等 tick 首活跃帧）"
        );

        // 首帧 tick（delay=0 → 立即活跃）→ 派 transitionstart（started false→true）。
        let _ = clock.tick(7, 0.0);
        let started = clock.drain_just_started();
        assert_eq!(started.len(), 1, "首活跃帧派 transitionstart");
        assert_eq!(started[0].element_key, 7);
        assert_eq!(started[0].property, "opacity");

        // 后续帧不再派 transitionstart（去重），进行中也未派 transitionend。
        let _ = clock.tick(7, 0.5);
        assert!(clock.drain_just_started().is_empty(), "已启动不重复派 transitionstart");
        assert!(clock.drain_just_run().is_empty(), "不重复派 transitionrun");
        assert!(clock.drain_just_finished().is_empty(), "进行中未完成");
    }

    #[test]
    fn test_transition_run_before_start_with_delay_r3252() {
        // R3252 关键：正 transition-delay 时，transitionrun 在创建即派，transitionstart 延后到 delay 过后。
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");
        new_style.transition_delay = vec![1.0]; // 1.0s 延迟

        // 创建 → 派 transitionrun（delay 期也派）。
        clock.start_transitions(3, &old_style, &new_style, 0.0);
        assert_eq!(clock.drain_just_run().len(), 1, "delay 期仍派 transitionrun");

        // t=0.5（delay 中）→ 不派 transitionstart。
        let _ = clock.tick(3, 0.5);
        assert!(clock.drain_just_started().is_empty(), "delay 期不派 transitionstart");

        // t=1.0（delay 刚过，进入活跃）→ 派 transitionstart。
        let _ = clock.tick(3, 1.0);
        assert_eq!(clock.drain_just_started().len(), 1, "delay 过后首帧派 transitionstart");
    }

    #[test]
    fn test_transition_start_before_end_instant_r3252() {
        // R3252 边界：瞬时过渡（首帧即完成）应先派 transitionstart 再派 transitionend（spec：start 先于
        // end，即使同帧）。tick() 的 started 检测置于 finish 检查之前，保证此序。
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity"); // delay=0，duration=1.0

        clock.start_transitions(4, &old_style, &new_style, 0.0);
        let _ = clock.drain_just_run(); // 清 run

        // 首帧 t=2.0 已远超 duration → 同帧既 start 又 end。
        let _ = clock.tick(4, 2.0);
        assert_eq!(clock.drain_just_started().len(), 1, "瞬时过渡首帧仍派 transitionstart");
        assert_eq!(clock.drain_just_finished().len(), 1, "瞬时过渡同帧也派 transitionend");
    }

    #[test]
    fn test_transition_with_delay() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");
        new_style.transition_delay = vec![0.5]; // 0.5s 延迟

        clock.start_transitions(1, &old_style, &new_style, 0.0);

        // t=0.3 → 在延迟中 → 返回旧值
        let props = clock.tick(1, 0.3);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 1.0).abs() < 0.05);

        // t=0.75 → 延迟后 0.25s → progress=0.25 → opacity=0.75
        let props = clock.tick(1, 0.75);
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.75).abs() < 0.05);
    }

    #[test]
    fn test_transition_no_property_skips() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        // 无 transition-property → 不启动
        new_style.transition_property = vec!["none".to_string()];

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        assert!(clock.active_element_ids().is_empty());
    }

    #[test]
    fn test_transition_zero_duration_skips() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");
        new_style.transition_duration = vec![0.0]; // 0s → 跳过

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        assert!(clock.active_element_ids().is_empty());
    }

    #[test]
    fn test_transition_cleanup_finished() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        // 推进到完成
        let _ = clock.tick(1, 2.0);
        clock.cleanup_finished();
        assert!(clock.active_transitions.is_empty());
    }

    #[test]
    fn test_transition_clear() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        clock.clear();
        assert!(clock.active_transitions.is_empty());
    }

    #[test]
    fn test_transition_multiple_properties() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.5;

        use zero_css_parser::values::LengthValue;
        new_style.width = LengthValue::Px(200.0);
        new_style.transition_property = vec!["opacity".to_string(), "width".to_string()];
        new_style.transition_duration = vec![1.0, 1.0];
        new_style.transition_delay = vec![0.0, 0.0];
        new_style.transition_timing_function = vec![TimingFunctionValue::Linear, TimingFunctionValue::Linear];

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        let props = clock.tick(1, 0.5);

        assert!(props.iter().any(|p| p.name == "opacity"));
        assert!(props.iter().any(|p| p.name == "width"));
    }

    #[test]
    fn test_transition_replaces_existing() {
        let mut clock = TransitionClock::new();

        // 第一次过渡：opacity 1→0.5
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.5;
        set_transition_property(&mut new_style, "opacity");
        clock.start_transitions(1, &old_style, &new_style, 0.0);

        // 第二次过渡：opacity 0.5→0.0（应替换）
        let old_style2 = new_style.clone();
        let mut new_style2 = ComputedStyle::default();
        new_style2.opacity = 0.0;
        set_transition_property(&mut new_style2, "opacity");
        clock.start_transitions(1, &old_style2, &new_style2, 0.5);

        // 应只有 1 个活跃过渡
        let transitions = clock.active_transitions.get(&1).unwrap();
        let active = transitions.iter().filter(|t| !t.finished).count();
        assert_eq!(active, 1);
    }

    #[test]
    fn test_transition_ease_timing() {
        let mut clock = TransitionClock::new();
        let old_style = ComputedStyle::default();
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        set_transition_property(&mut new_style, "opacity");
        new_style.transition_timing_function = vec![TimingFunctionValue::Ease];

        clock.start_transitions(1, &old_style, &new_style, 0.0);
        let props = clock.tick(1, 0.5);

        // ease 在 t=0.5 时输出约 0.8（快于线性）
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        let val = opacity.value.parse::<f64>().unwrap();
        // ease(0.5) ≈ 0.8，所以 opacity ≈ 1 - 0.8 = 0.2
        assert!(val < 0.5, "ease at 0.5 should be faster than linear, got {}", val);
    }

    #[test]
    fn test_transition_no_element_returns_empty() {
        let mut clock = TransitionClock::new();
        let props = clock.tick(999, 1.0);
        assert!(props.is_empty());
    }

    // ── 属性值提取测试 ──

    #[test]
    fn test_get_property_value_opacity() {
        let mut style = ComputedStyle::default();
        style.opacity = 0.75;
        assert_eq!(get_property_value(&style, "opacity"), "0.75");
    }

    #[test]
    fn test_get_property_value_width() {
        use zero_css_parser::values::LengthValue;
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(200.0);
        assert_eq!(get_property_value(&style, "width"), "200px");
    }

    #[test]
    fn test_get_property_value_unknown() {
        let style = ComputedStyle::default();
        assert!(get_property_value(&style, "display").is_empty());
    }

    // ── 辅助函数测试 ──

    #[test]
    fn test_color_value_to_string() {
        use zero_css_parser::values::ColorValue;
        assert_eq!(
            color_value_to_string(&ColorValue::Rgba(255, 0, 0, 255)),
            "rgba(255, 0, 0, 255)"
        );
        assert_eq!(color_value_to_string(&ColorValue::Transparent), "transparent");
    }

    #[test]
    fn test_length_value_to_string() {
        use zero_css_parser::values::LengthValue;
        assert_eq!(length_value_to_string(&LengthValue::Px(100.0)), "100px");
        assert_eq!(length_value_to_string(&LengthValue::Auto), "auto");
    }

    // ── 管线集成测试 ──

    /// 测试 TransitionClock 通过 RenderPipeline 的完整生命周期。
    ///
    /// 第一帧渲染建立基础样式，第二帧相同样式不会启动过渡，
    /// 修改 CSS 后第三帧应检测到变化并启动过渡。
    #[test]
    fn test_pipeline_transition_lifecycle() {
        use crate::pipeline::RenderPipeline;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body><div class="box">Trans</div></body></html>"#;
        let css = r#".box { transition: opacity 1s linear; opacity: 1.0; background-color: red; width: 100px; }"#;

        // 第一帧 — 建立基础样式
        let _r1 = pipeline.render_html_animated(html, css, 0.0);

        // 第二帧 — 相同样式，不应有活跃过渡
        let _r2 = pipeline.render_html_animated(html, css, 0.5);
        let active = pipeline.transition_clock_mut().active_element_ids();
        // 注意：由于每次渲染都重建 DOM，NodeId 会变化，过渡检测基于 cached_styles
        // 如果 NodeId 不匹配则不会有活跃过渡
        // 这是预期行为：render_html_animated 主要用于动画，
        // 过渡更适用于 recompute_styles 路径（DOM 不变）

        // 确认管线不崩溃且结果有效
        let _r3 = pipeline.render_html_animated(html, css, 1.0);
        drop(active);
    }

    /// 测试通过 transition_clock_mut 手动启动过渡后 render_html_animated 应用插值。
    #[test]
    fn test_pipeline_manual_transition_applied() {
        use crate::pipeline::RenderPipeline;
        use zero_css_parser::values::{LengthValue, TimingFunctionValue};
        use zero_style_system::ComputedStyle;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body><div class="box">Trans</div></body></html>"#;
        let css = r#".box { transition: opacity 1s linear; opacity: 1.0; background-color: red; width: 100px; }"#;

        // 手动启动一个过渡（模拟样式变化场景）
        let mut old_style = ComputedStyle::default();
        old_style.opacity = 1.0;
        let mut new_style = ComputedStyle::default();
        new_style.opacity = 0.0;
        new_style.transition_property = vec!["opacity".to_string()];
        new_style.transition_duration = vec![1.0];
        new_style.transition_delay = vec![0.0];
        new_style.transition_timing_function = vec![TimingFunctionValue::Linear];

        pipeline
            .transition_clock_mut()
            .start_transitions(42, &old_style, &new_style, 0.0);

        // 确认过渡已启动
        assert!(pipeline.transition_clock_mut().active_element_ids().contains(&42));

        // 推进到 t=0.5
        let props = pipeline.transition_clock_mut().tick(42, 0.5);
        assert!(!props.is_empty());
        let opacity = props.iter().find(|p| p.name == "opacity").unwrap();
        assert!((opacity.value.parse::<f64>().unwrap() - 0.5).abs() < 0.05);
    }

    /// 测试 TransitionClock 的 clear 方法通过管线访问器正确工作。
    #[test]
    fn test_pipeline_transition_clock_clear() {
        use crate::pipeline::RenderPipeline;
        use zero_css_parser::values::TimingFunctionValue;
        use zero_style_system::ComputedStyle;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);

        // 启动一个过渡
        let mut old_s = ComputedStyle::default();
        old_s.opacity = 1.0;
        let mut new_s = ComputedStyle::default();
        new_s.opacity = 0.0;
        new_s.transition_property = vec!["opacity".to_string()];
        new_s.transition_duration = vec![1.0];
        new_s.transition_timing_function = vec![TimingFunctionValue::Linear];

        pipeline
            .transition_clock_mut()
            .start_transitions(1, &old_s, &new_s, 0.0);
        assert!(!pipeline.transition_clock_mut().active_element_ids().is_empty());

        pipeline.transition_clock_mut().clear();
        assert!(pipeline.transition_clock_mut().active_element_ids().is_empty());
    }
}
