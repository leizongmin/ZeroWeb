//! `transition` / `animation` 简写展开族（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `expand_transition` / `expand_animation` + 其私有辅助（parse_single_transition /
//! parse_single_animation / split_top_level_commas / is_time_value /
//! is_timing_function_keyword / is_animation_direction / is_animation_fill_mode /
//! is_animation_play_state）。仅 `expand_transition` / `expand_animation` 对外
//!（`pub(super)`，供 `mod.rs::expand_one` 调度）；其余为族内私有。共享的
//! `split_outside_parens` 仍留在 `mod.rs`（border-image / list-style 亦用），经 `use super::` 引用。

use super::{MatchingDecl, matches_css_wide_keyword, split_outside_parens, wide_keyword_to_longhands};

/// 展开 transition 简写。
///
/// CSS `transition` 简写格式为：
/// `transition: [property] [duration] [timing-function] [delay]`
///
/// 简化实现：解析空格分隔的标记，识别类型：
/// - 时间值（带 s/ms 后缀）→ duration 或 delay
/// - timing-function 关键字 → timing-function
/// - 其他 → property
pub(super) fn expand_transition(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &[
                "transition-property",
                "transition-duration",
                "transition-timing-function",
                "transition-delay",
            ],
            important,
            specificity,
        );
    }

    if value.eq_ignore_ascii_case("none") {
        return vec![
            mk("transition-property", "none"),
            mk("transition-duration", "0s"),
            mk("transition-timing-function", "ease"),
            mk("transition-delay", "0s"),
        ];
    }

    // R2307：CSS Transitions — `transition: <single-transition>#`（逗号分隔多过渡）。
    // 顶层逗号分割（paren-aware：cubic-bezier(...)/steps(...) 内部逗号保持一体），
    // 每条单独解析，各 longhand 跨条目用 ", " 连接（longhand apply 已按逗号 split 成 Vec）。
    let mut entries = split_top_level_commas(value);
    if entries.is_empty() {
        entries.push(String::new());
    }
    let mut properties = Vec::with_capacity(entries.len());
    let mut durations = Vec::with_capacity(entries.len());
    let mut timings = Vec::with_capacity(entries.len());
    let mut delays = Vec::with_capacity(entries.len());
    for entry in &entries {
        let (p, d, ti, de) = parse_single_transition(entry);
        properties.push(p);
        durations.push(d);
        timings.push(ti);
        delays.push(de);
    }

    let properties_str = properties.join(", ");
    let durations_str = durations.join(", ");
    let timings_str = timings.join(", ");
    let delays_str = delays.join(", ");
    vec![
        mk("transition-property", &properties_str),
        mk("transition-duration", &durations_str),
        mk("transition-timing-function", &timings_str),
        mk("transition-delay", &delays_str),
    ]
}

/// 解析单个 transition 条目（不含顶层逗号）→ (property, duration, timing, delay)。
/// 空白条目返回默认值（all / 0s / ease / 0s）。
fn parse_single_transition(entry: &str) -> (String, String, String, String) {
    let tokens = split_outside_parens(entry);
    let mut property = "all".to_string();
    let mut duration = "0s".to_string();
    let mut timing = "ease".to_string();
    let mut delay = "0s".to_string();
    let mut found_duration = false;

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        // 判断是否为时间值（duration/delay）
        if is_time_value(t) {
            if !found_duration {
                duration = t.to_string();
                found_duration = true;
            } else {
                delay = t.to_string();
            }
        } else if is_timing_function_keyword(t) || t.starts_with("cubic-bezier(") || t.starts_with("steps(") {
            timing = t.to_string();
        } else {
            property = t.to_string();
        }
    }
    (property, duration, timing, delay)
}

/// 顶层逗号分割（paren-aware：括号内逗号不分割，保留 cubic-bezier()/steps() 一体）。
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() {
                    parts.push(t);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() {
        parts.push(t);
    }
    parts
}

/// 检查字符串是否为 CSS 时间值。
fn is_time_value(s: &str) -> bool {
    s.ends_with("ms")
        || (s.ends_with('s') && !s.ends_with("ease"))
            && s.trim_end_matches("ms").trim_end_matches('s').parse::<f64>().is_ok()
}

/// 检查字符串是否为 timing-function 关键字。
fn is_timing_function_keyword(s: &str) -> bool {
    matches!(
        s,
        "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
    )
}

/// 展开 animation 简写。
///
/// CSS `animation` 简写格式：
/// `animation: [name] [duration] [timing-function] [delay] [iteration-count] [direction] [fill-mode] [play-state]`
///
/// 简化实现：按空格分割，根据值的类型推断对应的子属性。
pub(super) fn expand_animation(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &[
                "animation-name",
                "animation-duration",
                "animation-timing-function",
                "animation-delay",
                "animation-iteration-count",
                "animation-direction",
                "animation-fill-mode",
                "animation-play-state",
            ],
            important,
            specificity,
        );
    }

    // 特殊值 "none" 表示无动画（R2354：关键字大小写不敏感）
    if value.eq_ignore_ascii_case("none") {
        return vec![
            mk("animation-name", "none"),
            mk("animation-duration", "0s"),
            mk("animation-timing-function", "ease"),
            mk("animation-delay", "0s"),
            mk("animation-iteration-count", "1"),
            mk("animation-direction", "normal"),
            mk("animation-fill-mode", "none"),
            mk("animation-play-state", "running"),
        ];
    }

    // R2307：CSS Animations — `animation: <single-animation>#`（逗号分隔多动画）。
    // 顶层逗号分割（paren-aware），每条单独解析，各 longhand 跨条目用 ", " 连接。
    let mut entries = split_top_level_commas(value);
    if entries.is_empty() {
        entries.push(String::new());
    }
    let mut names = Vec::with_capacity(entries.len());
    let mut durations = Vec::with_capacity(entries.len());
    let mut timings = Vec::with_capacity(entries.len());
    let mut delays = Vec::with_capacity(entries.len());
    let mut iteration_counts = Vec::with_capacity(entries.len());
    let mut directions = Vec::with_capacity(entries.len());
    let mut fill_modes = Vec::with_capacity(entries.len());
    let mut play_states = Vec::with_capacity(entries.len());
    for entry in &entries {
        let (n, d, ti, de, ic, di, fm, ps) = parse_single_animation(entry);
        names.push(n);
        durations.push(d);
        timings.push(ti);
        delays.push(de);
        iteration_counts.push(ic);
        directions.push(di);
        fill_modes.push(fm);
        play_states.push(ps);
    }

    vec![
        mk("animation-name", &names.join(", ")),
        mk("animation-duration", &durations.join(", ")),
        mk("animation-timing-function", &timings.join(", ")),
        mk("animation-delay", &delays.join(", ")),
        mk("animation-iteration-count", &iteration_counts.join(", ")),
        mk("animation-direction", &directions.join(", ")),
        mk("animation-fill-mode", &fill_modes.join(", ")),
        mk("animation-play-state", &play_states.join(", ")),
    ]
}

/// 解析单个 animation 条目（不含顶层逗号）→ 8 个 longhand 值。
/// 空白条目返回默认值。
fn parse_single_animation(entry: &str) -> (String, String, String, String, String, String, String, String) {
    let tokens = split_outside_parens(entry);
    let mut name = "none".to_string();
    let mut duration = "0s".to_string();
    let mut timing = "ease".to_string();
    let mut delay = "0s".to_string();
    let mut iteration_count = "1".to_string();
    let mut direction = "normal".to_string();
    let mut fill_mode = "none".to_string();
    let mut play_state = "running".to_string();
    let mut found_time_count = 0u32;

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }

        // 时间值（duration/delay）
        if is_time_value(t) {
            found_time_count += 1;
            if found_time_count == 1 {
                duration = t.to_string();
            } else if found_time_count == 2 {
                delay = t.to_string();
            }
        } else if is_timing_function_keyword(t) || t.starts_with("cubic-bezier(") || t.starts_with("steps(") {
            timing = t.to_string();
        } else if t == "infinite" {
            iteration_count = "infinite".to_string();
        } else if is_animation_direction(t) {
            direction = t.to_string();
        } else if is_animation_fill_mode(t) {
            fill_mode = t.to_string();
        } else if is_animation_play_state(t) {
            play_state = t.to_string();
        } else if t.parse::<f64>().is_ok() {
            // 纯数字 → iteration-count
            iteration_count = t.to_string();
        } else {
            // 其他 → animation-name
            name = t.to_string();
        }
    }
    (
        name,
        duration,
        timing,
        delay,
        iteration_count,
        direction,
        fill_mode,
        play_state,
    )
}

/// 检查字符串是否为 animation-direction 关键字。
fn is_animation_direction(s: &str) -> bool {
    matches!(s, "normal" | "reverse" | "alternate" | "alternate-reverse")
}

/// 检查字符串是否为 animation-fill-mode 关键字。
fn is_animation_fill_mode(s: &str) -> bool {
    matches!(s, "none" | "forwards" | "backwards" | "both")
}

/// 检查字符串是否为 animation-play-state 关键字。
fn is_animation_play_state(s: &str) -> bool {
    matches!(s, "running" | "paused")
}
