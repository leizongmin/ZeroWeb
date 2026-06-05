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
}

/// Transition 时钟 — 管理所有活跃的过渡并按帧推进。
#[derive(Debug, Clone, Default)]
pub struct TransitionClock {
    /// 活跃的过渡（元素 ID → 过渡列表）。
    active_transitions: HashMap<u64, Vec<TransitionState>>,
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
            };

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

            if progress >= 1.0 {
                transition.finished = true;
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
}
