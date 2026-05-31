//! CSS 简写属性展开。
//!
//! 将 CSS 简写属性（如 `margin: 10px`）展开为对应的长属性列表
//! （如 `margin-top: 10px; margin-right: 10px; ...`）。
//!
//! 展开在级联之前，确保简写属性与长属性的特异性竞争正确处理。

/// 匹配声明类型：(属性名, 属性值, 是否 important, 特异性)
type MatchingDecl = (String, String, bool, (u32, u32, u32));

/// 展开简写属性声明。
///
/// 遍历所有匹配声明，将简写属性展开为长属性列表。
/// 非简写属性原样保留。
///
/// # 参数
///
/// - `declarations` — 原始匹配声明列表
///
/// # 返回值
///
/// 展开后的声明列表（所有属性均为长属性）
pub fn expand_shorthands(declarations: &[MatchingDecl]) -> Vec<MatchingDecl> {
    let mut result = Vec::with_capacity(declarations.len() * 2);

    for (property, value, important, specificity) in declarations {
        let expanded = expand_one(property, value, *important, *specificity);
        result.extend(expanded);
    }

    result
}

/// 展开单个声明。
fn expand_one(property: &str, value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    match property {
        // ── 4 边简写 ──
        "margin" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![
                mk("margin-top", t),
                mk("margin-right", r),
                mk("margin-bottom", b),
                mk("margin-left", l),
            ]
        }
        "padding" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![
                mk("padding-top", t),
                mk("padding-right", r),
                mk("padding-bottom", b),
                mk("padding-left", l),
            ]
        }

        // ── border 边简写 ──
        "border-width" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![
                mk("border-top-width", t),
                mk("border-right-width", r),
                mk("border-bottom-width", b),
                mk("border-left-width", l),
            ]
        }
        "border-style" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![
                mk("border-top-style", t),
                mk("border-right-style", r),
                mk("border-bottom-style", b),
                mk("border-left-style", l),
            ]
        }
        "border-color" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![
                mk("border-top-color", t),
                mk("border-right-color", r),
                mk("border-bottom-color", b),
                mk("border-left-color", l),
            ]
        }

        // ── 单边 border 简写 ──
        "border-top" => expand_border_side(
            value,
            "border-top-width",
            "border-top-style",
            "border-top-color",
            important,
            specificity,
        ),
        "border-right" => expand_border_side(
            value,
            "border-right-width",
            "border-right-style",
            "border-right-color",
            important,
            specificity,
        ),
        "border-bottom" => expand_border_side(
            value,
            "border-bottom-width",
            "border-bottom-style",
            "border-bottom-color",
            important,
            specificity,
        ),
        "border-left" => expand_border_side(
            value,
            "border-left-width",
            "border-left-style",
            "border-left-color",
            important,
            specificity,
        ),

        // ── border 全写 ──
        "border" => expand_border_all(value, important, specificity),

        // ── overflow ──
        // 单值：同时应用于 overflow-x 和 overflow-y
        // 双值：第一个为 overflow-x，第二个为 overflow-y
        "overflow" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            match parts.len() {
                1 => vec![mk("overflow-x", parts[0]), mk("overflow-y", parts[0])],
                2 => vec![mk("overflow-x", parts[0]), mk("overflow-y", parts[1])],
                _ => vec![mk("overflow-x", value.trim()), mk("overflow-y", value.trim())],
            }
        }

        // ── border-radius ──
        "border-radius" => expand_border_radius(value, important, specificity),

        // ── flex ──
        "flex" => expand_flex(value, important, specificity),

        // ── inset ──
        "inset" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![mk("top", t), mk("right", r), mk("bottom", b), mk("left", l)]
        }

        // ── transition 简写 ──
        // transition: <property> <duration> <timing-function> <delay>
        // 简化实现：单组值
        "transition" => expand_transition(value, important, specificity),

        // ── 逻辑属性简写 ──
        "margin-block" => expand_axis_logical(value, "margin-block-start", "margin-block-end", important, specificity),
        "margin-inline" => expand_axis_logical(
            value,
            "margin-inline-start",
            "margin-inline-end",
            important,
            specificity,
        ),
        "padding-block" => expand_axis_logical(
            value,
            "padding-block-start",
            "padding-block-end",
            important,
            specificity,
        ),
        "padding-inline" => expand_axis_logical(
            value,
            "padding-inline-start",
            "padding-inline-end",
            important,
            specificity,
        ),
        "inset-block" => expand_axis_logical(value, "inset-block-start", "inset-block-end", important, specificity),
        "inset-inline" => expand_axis_logical(value, "inset-inline-start", "inset-inline-end", important, specificity),

        // ── animation 简写 ──
        // animation: name duration timing-function delay iteration-count direction fill-mode play-state
        "animation" => expand_animation(value, important, specificity),

        // ── Grid placement 简写 ──
        "grid-column" => expand_grid_axis(value, "grid-column-start", "grid-column-end", important, specificity),
        "grid-row" => expand_grid_axis(value, "grid-row-start", "grid-row-end", important, specificity),
        "grid-area" => expand_grid_area(value, important, specificity),

        // ── Grid 对齐简写 ──
        "place-items" => expand_place(value, "align-items", "justify-items", important, specificity),
        "place-content" => expand_place(value, "align-content", "justify-content", important, specificity),
        "place-self" => expand_place(value, "align-self", "justify-self", important, specificity),

        // ── Grid template 简写 ──
        "grid-template" => expand_grid_template(value, important, specificity),

        // ── list-style 简写 ──
        "list-style" => expand_list_style(value, important, specificity),

        // ── outline 简写 ──
        "outline" => expand_outline(value, important, specificity),

        // ── 非简写，原样返回 ──
        _ => vec![mk(property, value)],
    }
}

/// 解析 4 边简写的值部分。
///
/// 返回 (top, right, bottom, left) 值字符串。
///
/// 模式：
/// - 1 值：全部相同
/// - 2 值：上下, 左右
/// - 3 值：上, 左右, 下
/// - 4 值：上, 右, 下, 左
fn parse_rect_values(value: &str) -> Option<(&str, &str, &str, &str)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => Some((parts[0], parts[0], parts[0], parts[0])),
        2 => Some((parts[0], parts[1], parts[0], parts[1])),
        3 => Some((parts[0], parts[1], parts[2], parts[1])),
        4 => Some((parts[0], parts[1], parts[2], parts[3])),
        _ => None,
    }
}

/// 展开 border 全写（如 `border: 1px solid red`）。
///
/// 将 `border` 展开为 12 个长属性（4 边 × width/style/color）。
fn expand_border_all(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let parsed = parse_border_shorthand(value);
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    let mut result = Vec::with_capacity(12);
    for side in &["top", "right", "bottom", "left"] {
        result.push(mk(&format!("border-{side}-width"), &parsed.width));
        result.push(mk(&format!("border-{side}-style"), &parsed.style));
        result.push(mk(&format!("border-{side}-color"), &parsed.color));
    }
    result
}

/// 展开 border 单边简写（如 `border-top: 1px solid red`）。
fn expand_border_side(
    value: &str,
    width_prop: &str,
    style_prop: &str,
    color_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let parsed = parse_border_shorthand(value);
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    vec![
        mk(width_prop, &parsed.width),
        mk(style_prop, &parsed.style),
        mk(color_prop, &parsed.color),
    ]
}

/// border 简写解析结果。
struct BorderShorthand {
    width: String,
    style: String,
    color: String,
}

/// 解析 border 简写值（如 `1px solid red`）。
///
/// 识别 width（长度值）、style（关键字）和 color 部分，
/// 未指定的部分使用初始值。
fn parse_border_shorthand(value: &str) -> BorderShorthand {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut width = "medium".to_string();
    let mut style = "none".to_string();
    let mut color = "currentcolor".to_string();

    for part in parts {
        if is_border_style_keyword(part) {
            style = part.to_string();
        } else if looks_like_length(part) {
            width = part.to_string();
        } else if looks_like_color(part) {
            color = part.to_string();
        }
    }

    BorderShorthand { width, style, color }
}

/// 检查字符串是否为 border-style 关键字。
fn is_border_style_keyword(s: &str) -> bool {
    matches!(
        s,
        "none" | "hidden" | "dotted" | "dashed" | "solid" | "double" | "groove" | "ridge" | "inset" | "outset"
    )
}

/// 检查字符串是否看起来像长度值。
fn looks_like_length(s: &str) -> bool {
    s.ends_with("px")
        || s.ends_with("em")
        || s.ends_with("rem")
        || s.ends_with("pt")
        || s.ends_with("pc")
        || s.ends_with("cm")
        || s.ends_with("mm")
        || s.ends_with("in")
        || s.ends_with("vw")
        || s.ends_with("vh")
        || s.ends_with("vmin")
        || s.ends_with("vmax")
        || s.ends_with("ch")
        || s == "0"
        || s == "thin"
        || s == "medium"
        || s == "thick"
}

/// 检查字符串是否看起来像颜色值。
fn looks_like_color(s: &str) -> bool {
    if s.starts_with('#') {
        return true;
    }
    if s.starts_with("rgb") || s.starts_with("hsl") {
        return true;
    }
    // CSS 命名颜色（常见子集）
    matches!(
        s,
        "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "brown"
            | "gray"
            | "grey"
            | "cyan"
            | "magenta"
            | "lime"
            | "maroon"
            | "navy"
            | "olive"
            | "teal"
            | "aqua"
            | "fuchsia"
            | "silver"
            | "gold"
            | "indigo"
            | "violet"
            | "coral"
            | "salmon"
            | "tomato"
            | "skyblue"
            | "tan"
            | "wheat"
            | "khaki"
            | "beige"
            | "ivory"
            | "snow"
            | "linen"
            | "azure"
            | "lavender"
            | "whitesmoke"
            | "gainsboro"
            | "lightgray"
            | "darkgray"
            | "dimgray"
            | "darkred"
            | "darkgreen"
            | "darkblue"
            | "lightblue"
            | "lightgreen"
            | "lightcoral"
            | "deeppink"
            | "hotpink"
            | "orangered"
            | "crimson"
            | "firebrick"
            | "chocolate"
            | "sienna"
            | "peru"
            | "goldenrod"
            | "darkgoldenrod"
            | "greenyellow"
            | "chartreuse"
            | "limegreen"
            | "palegreen"
            | "seagreen"
            | "forestgreen"
            | "yellowgreen"
            | "olivedrab"
            | "darkolivegreen"
            | "darkcyan"
            | "darkseagreen"
            | "lightseagreen"
            | "mediumseagreen"
            | "turquoise"
            | "darkturquoise"
            | "paleturquoise"
            | "deepskyblue"
            | "dodgerblue"
            | "cornflowerblue"
            | "royalblue"
            | "mediumblue"
            | "midnightblue"
            | "darkviolet"
            | "blueviolet"
            | "mediumpurple"
            | "darkorchid"
            | "orchid"
            | "plum"
            | "currentcolor"
            | "transparent"
    )
}

/// 展开 border-radius 简写。
///
/// 支持 1-4 值模式，与 4 边简写相同。
fn expand_border_radius(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let Some((tl, tr, br, bl)) = parse_rect_values(value) else {
        return vec![];
    };
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    vec![
        mk("border-top-left-radius", tl),
        mk("border-top-right-radius", tr),
        mk("border-bottom-right-radius", br),
        mk("border-bottom-left-radius", bl),
    ]
}

/// 展开 flex 简写。
///
/// - `none` → grow: 0, shrink: 0, basis: auto
/// - `auto` → grow: 1, shrink: 1, basis: auto
/// - 单值：grow
/// - 双值：grow, shrink
/// - 三值：grow, shrink, basis
fn expand_flex(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if value == "none" {
        return vec![mk("flex-grow", "0"), mk("flex-shrink", "0"), mk("flex-basis", "auto")];
    }
    if value == "auto" {
        return vec![mk("flex-grow", "1"), mk("flex-shrink", "1"), mk("flex-basis", "auto")];
    }
    if value == "initial" {
        return vec![mk("flex-grow", "0"), mk("flex-shrink", "1"), mk("flex-basis", "auto")];
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => vec![mk("flex-grow", parts[0]), mk("flex-shrink", "1"), mk("flex-basis", "0")],
        2 => vec![
            mk("flex-grow", parts[0]),
            mk("flex-shrink", parts[1]),
            mk("flex-basis", "0"),
        ],
        3 => vec![
            mk("flex-grow", parts[0]),
            mk("flex-shrink", parts[1]),
            mk("flex-basis", parts[2]),
        ],
        _ => vec![],
    }
}

/// 展开 transition 简写。
///
/// CSS `transition` 简写格式为：
/// `transition: [property] [duration] [timing-function] [delay]`
///
/// 简化实现：解析空格分隔的标记，识别类型：
/// - 时间值（带 s/ms 后缀）→ duration 或 delay
/// - timing-function 关键字 → timing-function
/// - 其他 → property
fn expand_transition(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if value == "none" {
        return vec![
            mk("transition-property", "none"),
            mk("transition-duration", "0s"),
            mk("transition-timing-function", "ease"),
            mk("transition-delay", "0s"),
        ];
    }

    // 解析空格分隔的标记，但保留括号内的内容
    let tokens = split_outside_parens(value);
    let mut property = "all";
    let mut duration = "0s";
    let mut timing = "ease";
    let mut delay = "0s";
    let mut found_duration = false;

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        // 判断是否为时间值（duration/delay）
        if is_time_value(t) {
            if !found_duration {
                duration = t;
                found_duration = true;
            } else {
                delay = t;
            }
        } else if is_timing_function_keyword(t) || t.starts_with("cubic-bezier(") || t.starts_with("steps(") {
            timing = t;
        } else {
            property = t;
        }
    }

    vec![
        mk("transition-property", property),
        mk("transition-duration", duration),
        mk("transition-timing-function", timing),
        mk("transition-delay", delay),
    ]
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

/// 按空格分割字符串，但保留括号内的内容不分割。
fn split_outside_parens(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;

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
            ' ' | '\t' if depth == 0 => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// 展开 animation 简写。
///
/// CSS `animation` 简写格式：
/// `animation: [name] [duration] [timing-function] [delay] [iteration-count] [direction] [fill-mode] [play-state]`
///
/// 简化实现：按空格分割，根据值的类型推断对应的子属性。
fn expand_animation(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 特殊值 "none" 表示无动画
    if value == "none" {
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

    let tokens = split_outside_parens(value);
    let mut name = "none";
    let mut duration = "0s";
    let mut timing = "ease";
    let mut delay = "0s";
    let mut iteration_count = "1";
    let mut direction = "normal";
    let mut fill_mode = "none";
    let mut play_state = "running";
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
                duration = t;
            } else if found_time_count == 2 {
                delay = t;
            }
        } else if is_timing_function_keyword(t) || t.starts_with("cubic-bezier(") || t.starts_with("steps(") {
            timing = t;
        } else if t == "infinite" {
            iteration_count = "infinite";
        } else if is_animation_direction(t) {
            direction = t;
        } else if is_animation_fill_mode(t) {
            fill_mode = t;
        } else if is_animation_play_state(t) {
            play_state = t;
        } else if t.parse::<f64>().is_ok() {
            // 纯数字 → iteration-count
            iteration_count = t;
        } else {
            // 其他 → animation-name
            name = t;
        }
    }

    vec![
        mk("animation-name", name),
        mk("animation-duration", duration),
        mk("animation-timing-function", timing),
        mk("animation-delay", delay),
        mk("animation-iteration-count", iteration_count),
        mk("animation-direction", direction),
        mk("animation-fill-mode", fill_mode),
        mk("animation-play-state", play_state),
    ]
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

/// 展开轴方向逻辑属性简写。
///
/// `margin-block: 10px` → `margin-block-start: 10px; margin-block-end: 10px`
/// `margin-block: 10px 20px` → `margin-block-start: 10px; margin-block-end: 20px`
fn expand_axis_logical(
    value: &str,
    start_prop: &str,
    end_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    match parts.len() {
        1 => vec![mk(start_prop, parts[0]), mk(end_prop, parts[0])],
        2 => vec![mk(start_prop, parts[0]), mk(end_prop, parts[1])],
        _ => vec![],
    }
}

/// 展开 grid-column / grid-row 简写。
///
/// `grid-column: 1` → `grid-column-start: 1; grid-column-end: auto`
/// `grid-column: 1 / 3` → `grid-column-start: 1; grid-column-end: 3`
/// `grid-column: span 2` → `grid-column-start: span 2; grid-column-end: auto`
fn expand_grid_axis(
    value: &str,
    start_prop: &str,
    end_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    if let Some(slash_pos) = value.find('/') {
        let start = value[..slash_pos].trim();
        let end = value[slash_pos + 1..].trim();
        vec![mk(start_prop, start), mk(end_prop, end)]
    } else {
        vec![mk(start_prop, value.trim()), mk(end_prop, "auto")]
    }
}

/// 展开 grid-area 简写。
///
/// `grid-area: 1` → row-start: 1
/// `grid-area: 1 / 2` → row-start: 1, col-start: 2
/// `grid-area: 1 / 2 / 3` → row-start: 1, col-start: 2, row-end: 3
/// `grid-area: 1 / 2 / 3 / 4` → row-start: 1, col-start: 2, row-end: 3, col-end: 4
fn expand_grid_area(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    // 用 `/` 分割，但 span 内部可能有空格
    let parts: Vec<&str> = value.split('/').map(|s| s.trim()).collect();
    match parts.len() {
        1 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", "auto"),
            mk("grid-column-start", "auto"),
            mk("grid-column-end", "auto"),
        ],
        2 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", "auto"),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", "auto"),
        ],
        3 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", parts[2]),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", "auto"),
        ],
        4 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", parts[2]),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", parts[3]),
        ],
        _ => vec![],
    }
}

/// 展开 place-items / place-content / place-self 简写。
///
/// `place-items: center` → align-items: center; justify-items: center
/// `place-items: start end` → align-items: start; justify-items: end
///
/// 单值时两个子属性获得相同值，双值时分别对应 align 和 justify。
fn expand_place(
    value: &str,
    align_prop: &str,
    justify_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => vec![mk(align_prop, parts[0]), mk(justify_prop, parts[0])],
        2 => vec![mk(align_prop, parts[0]), mk(justify_prop, parts[1])],
        _ => vec![],
    }
}

/// 展开 grid-template 简写。
///
/// 简单形式：`grid-template: <rows> / <columns>`
/// - 按 `/` 分割，第一部分为 grid-template-rows，第二部分为 grid-template-columns
/// - 若 rows 部分含引号字符串，提取为 grid-template-areas
///
/// 示例：
/// - `grid-template: 100px 200px / 1fr 1fr 1fr`
///   → grid-template-rows: 100px 200px; grid-template-columns: 1fr 1fr 1fr
/// - `grid-template: "header header" 50px "main main" 1fr / 1fr 1fr`
///   → grid-template-areas + grid-template-rows + grid-template-columns
fn expand_grid_template(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let value = value.trim();

    // 按 `/` 分割为 rows 部分和 columns 部分
    if let Some(slash_pos) = value.find('/') {
        let rows_part = value[..slash_pos].trim();
        let cols_part = value[slash_pos + 1..].trim();

        // 从 rows 部分提取引号字符串作为 grid-template-areas
        let (areas_str, rows_only) = extract_quoted_areas(rows_part);

        let mut result = Vec::with_capacity(3);
        if !areas_str.is_empty() {
            result.push(mk("grid-template-areas", &areas_str));
        }
        if !rows_only.is_empty() {
            result.push(mk("grid-template-rows", rows_only.trim()));
        }
        result.push(mk("grid-template-columns", cols_part));
        result
    } else {
        // 无 `/`：整个值作为 rows
        let (areas_str, rows_only) = extract_quoted_areas(value);
        let mut result = Vec::with_capacity(2);
        if !areas_str.is_empty() {
            result.push(mk("grid-template-areas", &areas_str));
        }
        if !rows_only.is_empty() {
            result.push(mk("grid-template-rows", rows_only.trim()));
        }
        result
    }
}

/// 从 grid-template 的 rows 部分提取引号区域字符串和纯行尺寸。
///
/// `"header header" 50px "main main" 1fr` →
///   areas: `"header header" "main main"`, rows_only: `50px 1fr`
fn extract_quoted_areas(rows_part: &str) -> (String, String) {
    let mut areas = String::new();
    let mut rows_tokens: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut current_area = String::new();
    let mut current_row_token = String::new();

    let flush_row_token = |token: &mut String, tokens: &mut Vec<String>| {
        let trimmed = token.trim().to_string();
        if !trimmed.is_empty() {
            tokens.push(trimmed);
        }
        token.clear();
    };

    for ch in rows_part.chars() {
        if ch == '"' {
            if in_quotes {
                // 闭引号
                current_area.push(ch);
                if !areas.is_empty() {
                    areas.push(' ');
                }
                areas.push_str(&current_area);
                current_area.clear();
                in_quotes = false;
            } else {
                // 开引号 — 先 flush 任何正在累积的 row token
                flush_row_token(&mut current_row_token, &mut rows_tokens);
                current_area.clear();
                current_area.push(ch);
                in_quotes = true;
            }
        } else if in_quotes {
            current_area.push(ch);
        } else {
            // 不在引号内，属于行尺寸
            current_row_token.push(ch);
        }
    }

    // flush 最后一个 row token
    flush_row_token(&mut current_row_token, &mut rows_tokens);

    (areas, rows_tokens.join(" "))
}

/// 展开 list-style 简写。
///
/// CSS `list-style` 简写格式为：
/// `list-style: [type] [position] [image]`
///
/// 识别每个部分：type 是关键字，position 是 inside/outside，其余视为 image。
/// 未指定的部分保持初始值。
fn expand_list_style(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 特殊值 "none" 同时设置 type 和 image
    if value.eq_ignore_ascii_case("none") {
        return vec![mk("list-style-type", "none"), mk("list-style-position", "outside")];
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut list_type = "disc"; // 默认
    let mut position = "outside"; // 默认

    let is_position = |s: &str| s.eq_ignore_ascii_case("inside") || s.eq_ignore_ascii_case("outside");

    let is_type = |s: &str| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "disc"
                | "circle"
                | "square"
                | "decimal"
                | "decimal-leading-zero"
                | "lower-roman"
                | "upper-roman"
                | "lower-alpha"
                | "lower-latin"
                | "upper-alpha"
                | "upper-latin"
                | "none"
        )
    };

    for part in &parts {
        if is_position(part) {
            position = *part;
        } else if is_type(part) {
            list_type = *part;
        }
        // 其他值（如 url(...)）为 image，暂不处理
    }

    vec![mk("list-style-type", list_type), mk("list-style-position", position)]
}

/// 展开 outline 简写。
///
/// CSS `outline` 简写格式为：
/// `outline: [width] [style] [color]`
///
/// 各部分顺序无关，未指定的部分使用初始值。
fn expand_outline(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // "none" 或 "0" → 全部重置
    if value == "none" {
        return vec![
            mk("outline-width", "0px"),
            mk("outline-style", "none"),
            mk("outline-color", "currentcolor"),
        ];
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut width = "0px";
    let mut style = "none";
    let mut color = "currentcolor";

    for part in parts {
        if is_border_style_keyword(part) {
            style = part;
        } else if looks_like_length(part) {
            width = part;
        } else if looks_like_color(part) {
            color = part;
        }
    }

    vec![
        mk("outline-width", width),
        mk("outline-style", style),
        mk("outline-color", color),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 辅助函数 ──

    fn decl(property: &str, value: &str) -> MatchingDecl {
        (property.to_string(), value.to_string(), false, (0, 0, 1))
    }

    fn decl_important(property: &str, value: &str) -> MatchingDecl {
        (property.to_string(), value.to_string(), true, (0, 0, 1))
    }

    // ── margin 简写测试 ──

    #[test]
    fn test_margin_1_value() {
        let result = expand_one("margin", "10px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "margin-top");
        assert_eq!(result[0].1, "10px");
        assert_eq!(result[1].0, "margin-right");
        assert_eq!(result[1].1, "10px");
        assert_eq!(result[2].0, "margin-bottom");
        assert_eq!(result[3].0, "margin-left");
    }

    #[test]
    fn test_margin_2_values() {
        let result = expand_one("margin", "10px 20px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], ("margin-top".into(), "10px".into(), false, (0, 0, 1)));
        assert_eq!(result[1], ("margin-right".into(), "20px".into(), false, (0, 0, 1)));
        assert_eq!(result[2], ("margin-bottom".into(), "10px".into(), false, (0, 0, 1)));
        assert_eq!(result[3], ("margin-left".into(), "20px".into(), false, (0, 0, 1)));
    }

    #[test]
    fn test_margin_3_values() {
        let result = expand_one("margin", "10px 20px 30px", false, (0, 0, 1));
        assert_eq!(result[0], ("margin-top".into(), "10px".into(), false, (0, 0, 1)));
        assert_eq!(result[1], ("margin-right".into(), "20px".into(), false, (0, 0, 1)));
        assert_eq!(result[2], ("margin-bottom".into(), "30px".into(), false, (0, 0, 1)));
        assert_eq!(result[3], ("margin-left".into(), "20px".into(), false, (0, 0, 1)));
    }

    #[test]
    fn test_margin_4_values() {
        let result = expand_one("margin", "10px 20px 30px 40px", false, (0, 0, 1));
        assert_eq!(result[0], ("margin-top".into(), "10px".into(), false, (0, 0, 1)));
        assert_eq!(result[1], ("margin-right".into(), "20px".into(), false, (0, 0, 1)));
        assert_eq!(result[2], ("margin-bottom".into(), "30px".into(), false, (0, 0, 1)));
        assert_eq!(result[3], ("margin-left".into(), "40px".into(), false, (0, 0, 1)));
    }

    // ── padding 简写测试 ──

    #[test]
    fn test_padding_1_value() {
        let result = expand_one("padding", "5px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert!(result.iter().all(|(_, v, _, _)| v == "5px"));
    }

    #[test]
    fn test_padding_2_values() {
        let result = expand_one("padding", "5px 10px", false, (0, 0, 1));
        assert_eq!(result[0].1, "5px"); // top
        assert_eq!(result[1].1, "10px"); // right
        assert_eq!(result[2].1, "5px"); // bottom
        assert_eq!(result[3].1, "10px"); // left
    }

    // ── border-width/style/color 简写测试 ──

    #[test]
    fn test_border_width_shorthand() {
        let result = expand_one("border-width", "1px 2px 3px 4px", false, (0, 0, 1));
        let props: Vec<&str> = result.iter().map(|(p, _, _, _)| p.as_str()).collect();
        assert_eq!(
            props,
            vec![
                "border-top-width",
                "border-right-width",
                "border-bottom-width",
                "border-left-width"
            ]
        );
        assert_eq!(result[0].1, "1px");
        assert_eq!(result[1].1, "2px");
    }

    #[test]
    fn test_border_style_shorthand() {
        let result = expand_one("border-style", "solid dashed", false, (0, 0, 1));
        assert_eq!(result[0].1, "solid"); // top
        assert_eq!(result[1].1, "dashed"); // right
        assert_eq!(result[2].1, "solid"); // bottom
        assert_eq!(result[3].1, "dashed"); // left
    }

    #[test]
    fn test_border_color_shorthand() {
        let result = expand_one("border-color", "red green blue yellow", false, (0, 0, 1));
        assert_eq!(result[0].1, "red");
        assert_eq!(result[1].1, "green");
        assert_eq!(result[2].1, "blue");
        assert_eq!(result[3].1, "yellow");
    }

    // ── border 全写测试 ──

    #[test]
    fn test_border_all() {
        let result = expand_one("border", "1px solid red", false, (0, 0, 1));
        assert_eq!(result.len(), 12); // 4 sides × 3 props

        // 验证 top 侧
        assert_eq!(result[0].0, "border-top-width");
        assert_eq!(result[0].1, "1px");
        assert_eq!(result[1].0, "border-top-style");
        assert_eq!(result[1].1, "solid");
        assert_eq!(result[2].0, "border-top-color");
        assert_eq!(result[2].1, "red");

        // 所有侧的值应相同
        for side_start in [0, 3, 6, 9] {
            assert_eq!(result[side_start].1, "1px");
            assert_eq!(result[side_start + 1].1, "solid");
            assert_eq!(result[side_start + 2].1, "red");
        }
    }

    #[test]
    fn test_border_all_only_width() {
        let result = expand_one("border", "2px", false, (0, 0, 1));
        assert_eq!(result.len(), 12);
        assert_eq!(result[0].1, "2px"); // top-width
        assert_eq!(result[1].1, "none"); // top-style (default)
    }

    #[test]
    fn test_border_all_only_style() {
        let result = expand_one("border", "solid", false, (0, 0, 1));
        assert_eq!(result[0].1, "medium"); // top-width (default)
        assert_eq!(result[1].1, "solid"); // top-style
    }

    // ── 单边 border 简写测试 ──

    #[test]
    fn test_border_top_shorthand() {
        let result = expand_one("border-top", "2px dashed blue", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "border-top-width");
        assert_eq!(result[0].1, "2px");
        assert_eq!(result[1].0, "border-top-style");
        assert_eq!(result[1].1, "dashed");
        assert_eq!(result[2].0, "border-top-color");
        assert_eq!(result[2].1, "blue");
    }

    #[test]
    fn test_border_right_only_style() {
        let result = expand_one("border-right", "dotted", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "border-right-width");
        assert_eq!(result[0].1, "medium"); // 默认宽度
        assert_eq!(result[1].0, "border-right-style");
        assert_eq!(result[1].1, "dotted");
    }

    #[test]
    fn test_border_left_color_and_width() {
        let result = expand_one("border-left", "3px green", false, (0, 0, 1));
        assert_eq!(result[0].1, "3px"); // width
        assert_eq!(result[1].1, "none"); // style (default)
        assert_eq!(result[2].1, "green"); // color
    }

    // ── overflow 简写测试 ──

    #[test]
    fn test_overflow_shorthand() {
        let result = expand_one("overflow", "hidden", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "overflow-x");
        assert_eq!(result[0].1, "hidden");
        assert_eq!(result[1].0, "overflow-y");
        assert_eq!(result[1].1, "hidden");
    }

    #[test]
    fn test_overflow_scroll() {
        let result = expand_one("overflow", "scroll", false, (0, 0, 1));
        assert!(result.iter().all(|(_, v, _, _)| v == "scroll"));
    }

    // ── border-radius 简写测试 ──

    #[test]
    fn test_border_radius_1_value() {
        let result = expand_one("border-radius", "5px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert!(result.iter().all(|(_, v, _, _)| v == "5px"));
    }

    #[test]
    fn test_border_radius_2_values() {
        let result = expand_one("border-radius", "5px 10px", false, (0, 0, 1));
        assert_eq!(result[0].1, "5px"); // top-left
        assert_eq!(result[1].1, "10px"); // top-right
        assert_eq!(result[2].1, "5px"); // bottom-right
        assert_eq!(result[3].1, "10px"); // bottom-left
    }

    #[test]
    fn test_border_radius_4_values() {
        let result = expand_one("border-radius", "1px 2px 3px 4px", false, (0, 0, 1));
        assert_eq!(result[0].1, "1px"); // top-left
        assert_eq!(result[1].1, "2px"); // top-right
        assert_eq!(result[2].1, "3px"); // bottom-right
        assert_eq!(result[3].1, "4px"); // bottom-left
    }

    // ── flex 简写测试 ──

    #[test]
    fn test_flex_none() {
        let result = expand_one("flex", "none", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("flex-grow".into(), "0".into(), false, (0, 0, 1)));
        assert_eq!(result[1], ("flex-shrink".into(), "0".into(), false, (0, 0, 1)));
        assert_eq!(result[2], ("flex-basis".into(), "auto".into(), false, (0, 0, 1)));
    }

    #[test]
    fn test_flex_auto() {
        let result = expand_one("flex", "auto", false, (0, 0, 1));
        assert_eq!(result[0].1, "1"); // grow
        assert_eq!(result[1].1, "1"); // shrink
        assert_eq!(result[2].1, "auto"); // basis
    }

    #[test]
    fn test_flex_single_value() {
        let result = expand_one("flex", "2", false, (0, 0, 1));
        assert_eq!(result[0].1, "2"); // grow
        assert_eq!(result[1].1, "1"); // shrink (default)
        assert_eq!(result[2].1, "0"); // basis (default 0)
    }

    #[test]
    fn test_flex_two_values() {
        let result = expand_one("flex", "2 1", false, (0, 0, 1));
        assert_eq!(result[0].1, "2"); // grow
        assert_eq!(result[1].1, "1"); // shrink
        assert_eq!(result[2].1, "0"); // basis
    }

    #[test]
    fn test_flex_three_values() {
        let result = expand_one("flex", "2 1 100px", false, (0, 0, 1));
        assert_eq!(result[0].1, "2"); // grow
        assert_eq!(result[1].1, "1"); // shrink
        assert_eq!(result[2].1, "100px"); // basis
    }

    // ── inset 简写测试 ──

    #[test]
    fn test_inset_1_value() {
        let result = expand_one("inset", "10px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert!(result.iter().all(|(_, v, _, _)| v == "10px"));
    }

    #[test]
    fn test_inset_4_values() {
        let result = expand_one("inset", "1px 2px 3px 4px", false, (0, 0, 1));
        assert_eq!(result[0].0, "top");
        assert_eq!(result[0].1, "1px");
        assert_eq!(result[1].0, "right");
        assert_eq!(result[1].1, "2px");
        assert_eq!(result[2].0, "bottom");
        assert_eq!(result[2].1, "3px");
        assert_eq!(result[3].0, "left");
        assert_eq!(result[3].1, "4px");
    }

    // ── 非简写属性测试 ──

    #[test]
    fn test_longhand_passthrough() {
        let result = expand_one("color", "red", false, (0, 0, 1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "color");
        assert_eq!(result[0].1, "red");
    }

    #[test]
    fn test_display_passthrough() {
        let result = expand_one("display", "flex", false, (0, 0, 1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "display");
    }

    // ── expand_shorthands 集成测试 ──

    #[test]
    fn test_expand_shorthands_mixed() {
        let decls = vec![
            decl("margin", "10px"),
            decl("color", "red"),
            decl("padding", "5px 10px"),
        ];
        let result = expand_shorthands(&decls);
        // margin → 4, color → 1, padding → 4
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn test_expand_preserves_important() {
        let decls = vec![decl_important("margin", "10px")];
        let result = expand_shorthands(&decls);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
    }

    #[test]
    fn test_expand_preserves_specificity() {
        let decls = vec![("margin".to_string(), "10px".to_string(), false, (0, 1, 0))];
        let result = expand_shorthands(&decls);
        assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
    }

    #[test]
    fn test_expand_border_preserves_important() {
        let decls = vec![decl_important("border", "1px solid red")];
        let result = expand_shorthands(&decls);
        assert_eq!(result.len(), 12);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
    }

    #[test]
    fn test_expand_flex_preserves_important() {
        let decls = vec![decl_important("flex", "auto")];
        let result = expand_shorthands(&decls);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
    }

    // ── 辅助函数测试 ──

    #[test]
    fn test_parse_rect_values() {
        let (t, r, b, l) = parse_rect_values("10px").unwrap();
        assert_eq!(t, "10px");
        assert_eq!(r, "10px");
        assert_eq!(b, "10px");
        assert_eq!(l, "10px");

        let (t, r, _b, _l) = parse_rect_values("10px 20px").unwrap();
        assert_eq!(t, "10px");
        assert_eq!(r, "20px");

        let (t, r, b, l) = parse_rect_values("1 2 3 4").unwrap();
        assert_eq!(t, "1");
        assert_eq!(r, "2");
        assert_eq!(b, "3");
        assert_eq!(l, "4");
    }

    #[test]
    fn test_parse_rect_values_invalid() {
        assert!(parse_rect_values("1 2 3 4 5").is_none());
        assert!(parse_rect_values("").is_none());
    }

    #[test]
    fn test_is_border_style_keyword() {
        assert!(is_border_style_keyword("solid"));
        assert!(is_border_style_keyword("dashed"));
        assert!(is_border_style_keyword("none"));
        assert!(!is_border_style_keyword("red"));
        assert!(!is_border_style_keyword("10px"));
    }

    #[test]
    fn test_looks_like_length() {
        assert!(looks_like_length("10px"));
        assert!(looks_like_length("1.5em"));
        assert!(looks_like_length("0"));
        assert!(looks_like_length("thin"));
        assert!(!looks_like_length("solid"));
        assert!(!looks_like_length("red"));
    }

    #[test]
    fn test_looks_like_color() {
        assert!(looks_like_color("#fff"));
        assert!(looks_like_color("rgb(255,0,0)"));
        assert!(looks_like_color("red"));
        assert!(looks_like_color("transparent"));
        assert!(!looks_like_color("10px"));
        assert!(!looks_like_color("solid"));
    }

    // ── 边界条件测试 ──

    #[test]
    fn test_empty_value() {
        let result = expand_one("margin", "", false, (0, 0, 1));
        assert!(result.is_empty()); // 0 parts → None → empty vec
    }

    #[test]
    fn test_too_many_values() {
        let result = expand_one("margin", "1 2 3 4 5 6", false, (0, 0, 1));
        assert!(result.is_empty()); // 6 parts → None → empty vec
    }

    #[test]
    fn test_border_shorthand_order_independent() {
        // color before width
        let result = expand_one("border", "red 1px solid", false, (0, 0, 1));
        assert_eq!(result[0].1, "1px"); // width
        assert_eq!(result[1].1, "solid"); // style
        assert_eq!(result[2].1, "red"); // color
    }

    // ── transition 简写测试 ──

    #[test]
    fn test_transition_shorthand_none() {
        let result = expand_one("transition", "none", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "transition-property");
        assert_eq!(result[0].1, "none");
        assert_eq!(result[1].0, "transition-duration");
        assert_eq!(result[1].1, "0s");
        assert_eq!(result[2].0, "transition-timing-function");
        assert_eq!(result[2].1, "ease");
        assert_eq!(result[3].0, "transition-delay");
        assert_eq!(result[3].1, "0s");
    }

    #[test]
    fn test_transition_shorthand_property_duration() {
        let result = expand_one("transition", "opacity 0.3s", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "transition-property");
        assert_eq!(result[0].1, "opacity");
        assert_eq!(result[1].0, "transition-duration");
        assert_eq!(result[1].1, "0.3s");
    }

    #[test]
    fn test_transition_shorthand_full() {
        let result = expand_one("transition", "opacity 0.3s ease-in 0.1s", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "opacity");
        assert_eq!(result[1].1, "0.3s");
        assert_eq!(result[2].1, "ease-in");
        assert_eq!(result[3].1, "0.1s");
    }

    #[test]
    fn test_transition_shorthand_with_cubic_bezier() {
        let result = expand_one(
            "transition",
            "transform 0.5s cubic-bezier(0.25, 0.1, 0.25, 1.0)",
            false,
            (0, 0, 1),
        );
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "transform");
        assert_eq!(result[1].1, "0.5s");
        assert_eq!(result[2].1, "cubic-bezier(0.25, 0.1, 0.25, 1.0)");
    }

    #[test]
    fn test_transition_shorthand_duration_only() {
        let result = expand_one("transition", "0.5s", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "all"); // default property
        assert_eq!(result[1].1, "0.5s");
    }

    // ── 逻辑属性简写测试 ──

    #[test]
    fn test_margin_block_shorthand_single() {
        let result = expand_one("margin-block", "10px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "margin-block-start");
        assert_eq!(result[0].1, "10px");
        assert_eq!(result[1].0, "margin-block-end");
        assert_eq!(result[1].1, "10px");
    }

    #[test]
    fn test_margin_block_shorthand_two_values() {
        let result = expand_one("margin-block", "10px 20px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "margin-block-start");
        assert_eq!(result[0].1, "10px");
        assert_eq!(result[1].0, "margin-block-end");
        assert_eq!(result[1].1, "20px");
    }

    #[test]
    fn test_margin_inline_shorthand() {
        let result = expand_one("margin-inline", "5px 15px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "margin-inline-start");
        assert_eq!(result[0].1, "5px");
        assert_eq!(result[1].0, "margin-inline-end");
        assert_eq!(result[1].1, "15px");
    }

    #[test]
    fn test_padding_block_shorthand() {
        let result = expand_one("padding-block", "8px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "padding-block-start");
        assert_eq!(result[1].0, "padding-block-end");
        assert_eq!(result[0].1, "8px");
        assert_eq!(result[1].1, "8px");
    }

    #[test]
    fn test_padding_inline_shorthand() {
        let result = expand_one("padding-inline", "3px 7px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "padding-inline-start");
        assert_eq!(result[0].1, "3px");
        assert_eq!(result[1].0, "padding-inline-end");
        assert_eq!(result[1].1, "7px");
    }

    #[test]
    fn test_inset_block_shorthand() {
        let result = expand_one("inset-block", "100px 200px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "inset-block-start");
        assert_eq!(result[0].1, "100px");
        assert_eq!(result[1].0, "inset-block-end");
        assert_eq!(result[1].1, "200px");
    }

    #[test]
    fn test_inset_inline_shorthand() {
        let result = expand_one("inset-inline", "50px", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "inset-inline-start");
        assert_eq!(result[1].0, "inset-inline-end");
        assert_eq!(result[0].1, "50px");
        assert_eq!(result[1].1, "50px");
    }

    // ── animation 简写测试 ──

    #[test]
    fn test_animation_shorthand_none() {
        let result = expand_one("animation", "none", false, (0, 0, 1));
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].0, "animation-name");
        assert_eq!(result[0].1, "none");
    }

    #[test]
    fn test_animation_shorthand_name_duration() {
        let result = expand_one("animation", "fadeIn 0.5s", false, (0, 0, 1));
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "fadeIn");
        assert_eq!(result[1].1, "0.5s");
    }

    #[test]
    fn test_animation_shorthand_full() {
        let result = expand_one(
            "animation",
            "slideIn 0.3s ease-in 0.1s 3 alternate forwards",
            false,
            (0, 0, 1),
        );
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "slideIn"); // name
        assert_eq!(result[1].1, "0.3s"); // duration
        assert_eq!(result[2].1, "ease-in"); // timing
        assert_eq!(result[3].1, "0.1s"); // delay
        assert_eq!(result[4].1, "3"); // iteration-count
        assert_eq!(result[5].1, "alternate"); // direction
        assert_eq!(result[6].1, "forwards"); // fill-mode
    }

    #[test]
    fn test_animation_shorthand_infinite() {
        let result = expand_one("animation", "bounce 1s linear infinite", false, (0, 0, 1));
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "bounce");
        assert_eq!(result[1].1, "1s");
        assert_eq!(result[2].1, "linear");
        assert_eq!(result[4].1, "infinite");
    }

    #[test]
    fn test_animation_shorthand_paused() {
        let result = expand_one("animation", "spin 2s paused", false, (0, 0, 1));
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "spin");
        assert_eq!(result[7].1, "paused");
    }

    // ── grid placement 简写测试 ──

    #[test]
    fn test_grid_column_shorthand_single() {
        let result = expand_one("grid-column", "1", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "grid-column-start");
        assert_eq!(result[0].1, "1");
        assert_eq!(result[1].0, "grid-column-end");
        assert_eq!(result[1].1, "auto");
    }

    #[test]
    fn test_grid_column_shorthand_slash() {
        let result = expand_one("grid-column", "1 / 3", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "grid-column-start");
        assert_eq!(result[0].1, "1");
        assert_eq!(result[1].0, "grid-column-end");
        assert_eq!(result[1].1, "3");
    }

    #[test]
    fn test_grid_column_shorthand_span() {
        let result = expand_one("grid-column", "span 2 / 5", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "span 2");
        assert_eq!(result[1].1, "5");
    }

    #[test]
    fn test_grid_row_shorthand() {
        let result = expand_one("grid-row", "2 / 4", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "grid-row-start");
        assert_eq!(result[0].1, "2");
        assert_eq!(result[1].0, "grid-row-end");
        assert_eq!(result[1].1, "4");
    }

    #[test]
    fn test_grid_area_shorthand_1_value() {
        let result = expand_one("grid-area", "1", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "grid-row-start");
        assert_eq!(result[0].1, "1");
        assert_eq!(result[1].0, "grid-row-end");
        assert_eq!(result[1].1, "auto");
        assert_eq!(result[2].0, "grid-column-start");
        assert_eq!(result[2].1, "auto");
    }

    #[test]
    fn test_grid_area_shorthand_2_values() {
        let result = expand_one("grid-area", "1 / 3", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "1"); // row-start
        assert_eq!(result[2].1, "3"); // col-start
    }

    #[test]
    fn test_grid_area_shorthand_4_values() {
        let result = expand_one("grid-area", "1 / 2 / 3 / 4", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "grid-row-start");
        assert_eq!(result[0].1, "1");
        assert_eq!(result[1].0, "grid-row-end");
        assert_eq!(result[1].1, "3");
        assert_eq!(result[2].0, "grid-column-start");
        assert_eq!(result[2].1, "2");
        assert_eq!(result[3].0, "grid-column-end");
        assert_eq!(result[3].1, "4");
    }

    #[test]
    fn test_grid_area_shorthand_span() {
        let result = expand_one("grid-area", "1 / span 2 / 4 / span 3", false, (0, 0, 1));
        assert_eq!(result[0].1, "1");
        assert_eq!(result[1].1, "4");
        assert_eq!(result[2].1, "span 2");
        assert_eq!(result[3].1, "span 3");
    }

    // ── outline 简写测试 ──

    #[test]
    fn test_outline_shorthand_full() {
        let result = expand_one("outline", "2px solid red", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "outline-width");
        assert_eq!(result[0].1, "2px");
        assert_eq!(result[1].0, "outline-style");
        assert_eq!(result[1].1, "solid");
        assert_eq!(result[2].0, "outline-color");
        assert_eq!(result[2].1, "red");
    }

    #[test]
    fn test_outline_shorthand_none() {
        let result = expand_one("outline", "none", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "outline-width");
        assert_eq!(result[0].1, "0px");
        assert_eq!(result[1].0, "outline-style");
        assert_eq!(result[1].1, "none");
        assert_eq!(result[2].0, "outline-color");
        assert_eq!(result[2].1, "currentcolor");
    }

    #[test]
    fn test_outline_shorthand_only_style() {
        let result = expand_one("outline", "dashed", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, "0px"); // default width
        assert_eq!(result[1].1, "dashed"); // style
        assert_eq!(result[2].1, "currentcolor"); // default color
    }

    #[test]
    fn test_outline_shorthand_width_and_style() {
        let result = expand_one("outline", "3px dotted", false, (0, 0, 1));
        assert_eq!(result[0].1, "3px");
        assert_eq!(result[1].1, "dotted");
    }

    #[test]
    fn test_outline_shorthand_color_and_width() {
        let result = expand_one("outline", "#ff0000 1px", false, (0, 0, 1));
        assert_eq!(result[0].1, "1px");
        assert_eq!(result[1].1, "none"); // default style
        assert_eq!(result[2].1, "#ff0000");
    }

    #[test]
    fn test_outline_shorthand_order_independent() {
        let result = expand_one("outline", "blue solid 2px", false, (0, 0, 1));
        assert_eq!(result[0].1, "2px");
        assert_eq!(result[1].1, "solid");
        assert_eq!(result[2].1, "blue");
    }

    #[test]
    fn test_outline_shorthand_preserves_important() {
        let result = expand_one("outline", "1px solid red", true, (0, 1, 0));
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
        assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增简写边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// border 简写只含 width 和 color（无 style）
    fn test_border_shorthand_width_and_color_no_style() {
        let result = expand_one("border", "2px green", false, (0, 0, 1));
        assert_eq!(result.len(), 12);
        assert_eq!(result[0].1, "2px"); // top-width
        assert_eq!(result[1].1, "none"); // top-style default
        assert_eq!(result[2].1, "green"); // top-color
    }

    #[test]
    /// border-radius 4 个不同值
    fn test_border_radius_4_different_values() {
        let result = expand_one("border-radius", "1px 2px 3px 4px", false, (0, 0, 1));
        assert_eq!(result[0].1, "1px"); // top-left
        assert_eq!(result[1].1, "2px"); // top-right
        assert_eq!(result[2].1, "3px"); // bottom-right
        assert_eq!(result[3].1, "4px"); // bottom-left
    }

    #[test]
    /// border-radius 3 值：top-left top-right/bottom-left bottom-right
    fn test_border_radius_3_values() {
        let result = expand_one("border-radius", "5px 10px 15px", false, (0, 0, 1));
        assert_eq!(result[0].1, "5px"); // top-left
        assert_eq!(result[1].1, "10px"); // top-right
        assert_eq!(result[2].1, "15px"); // bottom-right
        assert_eq!(result[3].1, "10px"); // bottom-left = top-right
    }

    #[test]
    /// flex: initial 关键字展开
    fn test_flex_initial_keyword() {
        let result = expand_one("flex", "initial", false, (0, 0, 1));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, "0"); // grow
        assert_eq!(result[1].1, "1"); // shrink
        assert_eq!(result[2].1, "auto"); // basis
    }

    #[test]
    /// animation 简写含全部 8 个子属性
    fn test_animation_shorthand_all_8_sub_properties() {
        let result = expand_one(
            "animation",
            "fadeIn 1s ease-in 0.5s 3 reverse forwards paused",
            false,
            (0, 0, 1),
        );
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "fadeIn"); // name
        assert_eq!(result[1].1, "1s"); // duration
        assert_eq!(result[2].1, "ease-in"); // timing
        assert_eq!(result[3].1, "0.5s"); // delay
        assert_eq!(result[4].1, "3"); // iteration-count
        assert_eq!(result[5].1, "reverse"); // direction
        assert_eq!(result[6].1, "forwards"); // fill-mode
        assert_eq!(result[7].1, "paused"); // play-state
    }

    #[test]
    /// animation 简写含 steps() timing function
    fn test_animation_shorthand_with_steps() {
        let result = expand_one("animation", "bounce 0.5s steps(4) infinite", false, (0, 0, 1));
        assert_eq!(result.len(), 8);
        assert_eq!(result[0].1, "bounce");
        assert_eq!(result[2].1, "steps(4)");
        assert_eq!(result[4].1, "infinite");
    }

    #[test]
    /// inset 简写 2 值展开
    fn test_inset_2_values() {
        let result = expand_one("inset", "10px 20px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "10px"); // top
        assert_eq!(result[1].1, "20px"); // right
        assert_eq!(result[2].1, "10px"); // bottom = top
        assert_eq!(result[3].1, "20px"); // left = right
    }

    #[test]
    /// inset 简写 3 值展开
    fn test_inset_3_values() {
        let result = expand_one("inset", "1px 2px 3px", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "1px"); // top
        assert_eq!(result[1].1, "2px"); // right
        assert_eq!(result[2].1, "3px"); // bottom
        assert_eq!(result[3].1, "2px"); // left = right
    }

    #[test]
    /// grid-area 3 值展开
    fn test_grid_area_shorthand_3_values() {
        let result = expand_one("grid-area", "1 / 2 / 3", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "1"); // row-start
        assert_eq!(result[1].1, "3"); // row-end
        assert_eq!(result[2].1, "2"); // col-start
        assert_eq!(result[3].1, "auto"); // col-end
    }

    #[test]
    /// transition 简写含 ease-in-out
    fn test_transition_shorthand_ease_in_out() {
        let result = expand_one("transition", "all 0.5s ease-in-out 0.2s", false, (0, 0, 1));
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].1, "all");
        assert_eq!(result[1].1, "0.5s");
        assert_eq!(result[2].1, "ease-in-out");
        assert_eq!(result[3].1, "0.2s");
    }

    #[test]
    /// border 简写仅含 style
    fn test_border_shorthand_only_style_dotted() {
        let result = expand_one("border", "dotted", false, (0, 0, 1));
        assert_eq!(result.len(), 12);
        assert_eq!(result[0].1, "medium"); // top-width default
        assert_eq!(result[1].1, "dotted"); // top-style
        assert_eq!(result[2].1, "currentcolor"); // top-color default
    }

    #[test]
    /// overflow 简写 visible
    fn test_overflow_shorthand_visible() {
        let result = expand_one("overflow", "visible", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "visible");
        assert_eq!(result[1].1, "visible");
    }

    #[test]
    /// overflow 简写单值：hidden → overflow-x=hidden, overflow-y=hidden
    fn test_overflow_shorthand_single_value() {
        let result = expand_one("overflow", "hidden", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "overflow-x");
        assert_eq!(result[0].1, "hidden");
        assert_eq!(result[1].0, "overflow-y");
        assert_eq!(result[1].1, "hidden");
    }

    #[test]
    /// overflow 简写双值：hidden scroll → overflow-x=hidden, overflow-y=scroll
    fn test_overflow_shorthand_two_values() {
        let result = expand_one("overflow", "hidden scroll", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "overflow-x");
        assert_eq!(result[0].1, "hidden");
        assert_eq!(result[1].0, "overflow-y");
        assert_eq!(result[1].1, "scroll");
    }

    #[test]
    /// overflow 简写双值：visible hidden → overflow-x=visible, overflow-y=hidden
    fn test_overflow_shorthand_visible_hidden() {
        let result = expand_one("overflow", "visible hidden", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "overflow-x");
        assert_eq!(result[0].1, "visible");
        assert_eq!(result[1].0, "overflow-y");
        assert_eq!(result[1].1, "hidden");
    }

    // ── place-items / place-content / place-self 简写测试 ──

    #[test]
    /// place-items 单值：两个子属性相同
    fn test_place_items_single_value() {
        let result = expand_one("place-items", "center", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-items");
        assert_eq!(result[0].1, "center");
        assert_eq!(result[1].0, "justify-items");
        assert_eq!(result[1].1, "center");
    }

    #[test]
    /// place-items 双值：align 和 justify 分别对应
    fn test_place_items_two_values() {
        let result = expand_one("place-items", "start end", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-items");
        assert_eq!(result[0].1, "start");
        assert_eq!(result[1].0, "justify-items");
        assert_eq!(result[1].1, "end");
    }

    #[test]
    /// place-content 单值：space-between
    fn test_place_content_single_value() {
        let result = expand_one("place-content", "space-between", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-content");
        assert_eq!(result[0].1, "space-between");
        assert_eq!(result[1].0, "justify-content");
        assert_eq!(result[1].1, "space-between");
    }

    #[test]
    /// place-content 双值：space-around 和 space-evenly
    fn test_place_content_two_values() {
        let result = expand_one("place-content", "space-around space-evenly", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-content");
        assert_eq!(result[0].1, "space-around");
        assert_eq!(result[1].0, "justify-content");
        assert_eq!(result[1].1, "space-evenly");
    }

    #[test]
    /// place-self 单值：stretch
    fn test_place_self_single_value() {
        let result = expand_one("place-self", "stretch", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-self");
        assert_eq!(result[0].1, "stretch");
        assert_eq!(result[1].0, "justify-self");
        assert_eq!(result[1].1, "stretch");
    }

    #[test]
    /// place-self 双值：auto 和 center
    fn test_place_self_two_values() {
        let result = expand_one("place-self", "auto center", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "align-self");
        assert_eq!(result[0].1, "auto");
        assert_eq!(result[1].0, "justify-self");
        assert_eq!(result[1].1, "center");
    }

    // ── grid-template 简写测试 ──

    #[test]
    /// grid-template 简单形式：rows / columns
    fn test_grid_template_simple() {
        let result = expand_one("grid-template", "100px 200px / 1fr 1fr 1fr", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "grid-template-rows");
        assert_eq!(result[0].1, "100px 200px");
        assert_eq!(result[1].0, "grid-template-columns");
        assert_eq!(result[1].1, "1fr 1fr 1fr");
    }

    #[test]
    /// grid-template 含引号区域字符串
    fn test_grid_template_with_areas() {
        let result = expand_one(
            "grid-template",
            "\"header header\" 50px \"main main\" 1fr / 1fr 1fr",
            false,
            (0, 0, 1),
        );
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "grid-template-areas");
        assert_eq!(result[0].1, "\"header header\" \"main main\"");
        assert_eq!(result[1].0, "grid-template-rows");
        assert_eq!(result[1].1, "50px 1fr");
        assert_eq!(result[2].0, "grid-template-columns");
        assert_eq!(result[2].1, "1fr 1fr");
    }

    #[test]
    /// grid-template 无斜杠：只有 rows
    fn test_grid_template_no_slash() {
        let result = expand_one("grid-template", "100px 200px", false, (0, 0, 1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "grid-template-rows");
        assert_eq!(result[0].1, "100px 200px");
    }

    #[test]
    /// place-items 保留 important 标记
    fn test_place_items_preserves_important() {
        let result = expand_one("place-items", "center", true, (0, 1, 0));
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
        assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
    }

    #[test]
    /// place-* 超过两个值返回空
    fn test_place_too_many_values() {
        let result = expand_one("place-items", "start end center", false, (0, 0, 1));
        assert!(result.is_empty());
    }

    // ── list-style 简写测试 ──

    #[test]
    /// list-style: none → type=none, position=outside
    fn test_list_style_none() {
        let result = expand_one("list-style", "none", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "list-style-type");
        assert_eq!(result[0].1, "none");
        assert_eq!(result[1].0, "list-style-position");
        assert_eq!(result[1].1, "outside");
    }

    #[test]
    /// list-style: inside → type=disc(默认), position=inside
    fn test_list_style_position_only() {
        let result = expand_one("list-style", "inside", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "list-style-type");
        assert_eq!(result[0].1, "disc");
        assert_eq!(result[1].0, "list-style-position");
        assert_eq!(result[1].1, "inside");
    }

    #[test]
    /// list-style: square inside → type=square, position=inside
    fn test_list_style_type_and_position() {
        let result = expand_one("list-style", "square inside", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "list-style-type");
        assert_eq!(result[0].1, "square");
        assert_eq!(result[1].0, "list-style-position");
        assert_eq!(result[1].1, "inside");
    }

    #[test]
    /// list-style: decimal outside → type=decimal, position=outside
    fn test_list_style_decimal_outside() {
        let result = expand_one("list-style", "decimal outside", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "decimal");
        assert_eq!(result[1].1, "outside");
    }

    #[test]
    /// list-style: lower-roman inside → type=lower-roman, position=inside
    fn test_list_style_lower_roman_inside() {
        let result = expand_one("list-style", "lower-roman inside", false, (0, 0, 1));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "lower-roman");
        assert_eq!(result[1].1, "inside");
    }

    #[test]
    /// list-style 保留 important 标记
    fn test_list_style_preserves_important() {
        let result = expand_one("list-style", "square inside", true, (0, 1, 0));
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, _, imp, _)| *imp));
        assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
    }
}
