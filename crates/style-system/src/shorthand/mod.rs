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

        // ── border 逻辑属性简写（CSS Logical Properties §3）──
        // 展开为 logical longhand（border-{axis}-{side}-{width,style,color}），由
        // apply_advanced 按元素 computed writing-mode 映射到物理边。简写层不感知元素
        // 上下文，故只拆分组件、不做 logical→物理映射。
        "border-inline-start" => expand_border_side(
            value,
            "border-inline-start-width",
            "border-inline-start-style",
            "border-inline-start-color",
            important,
            specificity,
        ),
        "border-inline-end" => expand_border_side(
            value,
            "border-inline-end-width",
            "border-inline-end-style",
            "border-inline-end-color",
            important,
            specificity,
        ),
        "border-block-start" => expand_border_side(
            value,
            "border-block-start-width",
            "border-block-start-style",
            "border-block-start-color",
            important,
            specificity,
        ),
        "border-block-end" => expand_border_side(
            value,
            "border-block-end-width",
            "border-block-end-style",
            "border-block-end-color",
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

        // ── overscroll-behavior ──
        // 单值：同时应用于 overscroll-behavior-x 和 overscroll-behavior-y
        // 双值：第一个为 x，第二个为 y
        "overscroll-behavior" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            match parts.len() {
                1 => vec![
                    mk("overscroll-behavior-x", parts[0]),
                    mk("overscroll-behavior-y", parts[0]),
                ],
                2 => vec![
                    mk("overscroll-behavior-x", parts[0]),
                    mk("overscroll-behavior-y", parts[1]),
                ],
                _ => vec![
                    mk("overscroll-behavior-x", value.trim()),
                    mk("overscroll-behavior-y", value.trim()),
                ],
            }
        }

        // ── border-radius ──
        "border-radius" => expand_border_radius(value, important, specificity),

        // ── flex ──
        "flex" => expand_flex(value, important, specificity),

        // ── flex-flow: <flex-direction> || <flex-wrap> ──
        // CSS Flexbox §5.1: 当 flex-flow 省略一个组件时，缺失组件应设为初始值
        // (flex-direction: row, flex-wrap: nowrap)。始终同时输出两个子属性，
        // 确保简写正确覆盖之前的长写属性值。
        "flex-flow" => {
            let mut direction = None;
            let mut wrap = None;
            for token in value.split_whitespace() {
                if direction.is_none() && matches!(token, "row" | "row-reverse" | "column" | "column-reverse") {
                    direction = Some(token);
                } else if wrap.is_none() && matches!(token, "nowrap" | "wrap" | "wrap-reverse") {
                    wrap = Some(token);
                } else {
                    // 无法识别的 token，忽略
                }
            }
            vec![
                mk("flex-direction", direction.unwrap_or("row")),
                mk("flex-wrap", wrap.unwrap_or("nowrap")),
            ]
        }

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

        // ── background 简写 ──
        "background" => expand_background(value, important, specificity),

        // ── font 简写 ──
        "font" => expand_font(value, important, specificity),

        // ── text-decoration 简写 ──
        "text-decoration" => expand_text_decoration(value, important, specificity),

        // ── text-emphasis 简写（CSS Text Decoration 3 §3.1）──
        // text-emphasis: <text-emphasis-style> || <text-emphasis-color>
        // text-emphasis-color 暂未在 ComputedStyle 存储，故仅展开 style（剥离 color token）。
        "text-emphasis" => expand_text_emphasis(value, important, specificity),

        // ── list-style 简写 ──
        "list-style" => expand_list_style(value, important, specificity),

        // ── outline 简写 ──
        "outline" => expand_outline(value, important, specificity),

        // ── columns 简写 ──
        // columns: <column-count> <column-width>
        // 若值为纯整数 → column-count，其余设为 auto
        // 若值为长度 → column-width，其余设为 auto
        // 双值：依次为 column-count 和 column-width
        "columns" => expand_columns(value, important, specificity),

        // ── gap 简写 ──
        // gap: <row-gap> <column-gap>
        // 单值同时应用于 gap、row-gap 和 column-gap
        "gap" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            match parts.len() {
                1 => vec![mk("gap", parts[0]), mk("row-gap", parts[0]), mk("column-gap", parts[0])],
                2 => vec![mk("gap", parts[0]), mk("row-gap", parts[0]), mk("column-gap", parts[1])],
                _ => vec![],
            }
        }

        // ── column-rule 简写 ──
        // column-rule: [width] [style] [color]
        // 与 outline 类似，各部分顺序无关
        "column-rule" => expand_column_rule(value, important, specificity),

        // ── border-image 简写 ──
        "border-image" => expand_border_image(value, important, specificity),

        // ── 非简写，原样返回 ──
        _ => vec![mk(property, value)],
    }
}

/// 展开 border-image 简写。
///
/// CSS `border-image` 简写格式为：
/// `border-image: source slice / width / outset repeat`
///
/// 简化实现：
/// - 若值为 "none" → border-image-source: none
/// - 若值含 url(...) → 提取为 source
/// - 解析 slice、width（/ 后第一组）、outset（/ 后第二组）、repeat 关键字
fn expand_border_image(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 特殊值 "none" → 仅设置 source
    if value == "none" {
        return vec![mk("border-image-source", "none")];
    }

    let tokens = split_outside_parens(value);

    // 提取 source（url(...) 或 none）
    let mut source: Option<String> = None;
    let mut remaining: Vec<String> = Vec::new();

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if source.is_none() && (t.starts_with("url(") || t == "none") {
            source = Some(t.to_string());
        } else {
            remaining.push(t.to_string());
        }
    }

    // 处理斜杠分隔：在 remaining 中将 "/ " 转为独立 token
    // 按 "/" 分割 remaining 为最多 3 组：slice_part / width_part / outset_part
    let mut slash_groups: Vec<Vec<String>> = vec![Vec::new()];
    for token in &remaining {
        if token == "/" {
            slash_groups.push(Vec::new());
        } else {
            slash_groups.last_mut().unwrap().push(token.clone());
        }
    }

    let mut source_val = "none".to_string();
    let mut slice_val = String::new();
    let mut width_val = String::new();
    let mut outset_val = String::new();
    let mut repeat_val = String::new();

    if let Some(s) = source {
        source_val = s;
    }

    // repeat 关键字
    let is_repeat = |s: &str| matches!(s, "stretch" | "repeat" | "round" | "space");

    // 从 slash_groups 中提取值
    // 第一组（slice）：可能包含数字和 fill 关键字
    // 第二组（width）：数字/长度
    // 第三组（outset）：数字/长度
    for (gi, group) in slash_groups.iter().enumerate() {
        // 收集非 repeat 的 token 为组值，repeat 关键字单独记录
        let mut group_tokens: Vec<String> = Vec::new();
        for token in group {
            if is_repeat(token) {
                if repeat_val.is_empty() {
                    repeat_val = token.clone();
                }
            } else {
                group_tokens.push(token.clone());
            }
        }
        let group_str = group_tokens.join(" ");

        match gi {
            0 => slice_val = group_str,
            1 => width_val = group_str,
            2 => outset_val = group_str,
            _ => {}
        }
    }

    let mut result = Vec::new();
    result.push(mk("border-image-source", &source_val));
    if !slice_val.is_empty() {
        result.push(mk("border-image-slice", &slice_val));
    }
    if !width_val.is_empty() {
        result.push(mk("border-image-width", &width_val));
    }
    if !outset_val.is_empty() {
        result.push(mk("border-image-outset", &outset_val));
    }
    if !repeat_val.is_empty() {
        result.push(mk("border-image-repeat", &repeat_val));
    }
    result
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
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性
    if value == "inherit" || value == "initial" || value == "unset" {
        let mut result = Vec::with_capacity(12);
        for side in &["top", "right", "bottom", "left"] {
            result.push(mk(&format!("border-{side}-width"), value));
            result.push(mk(&format!("border-{side}-style"), value));
            result.push(mk(&format!("border-{side}-color"), value));
        }
        return result;
    }

    let parsed = parse_border_shorthand(value);
    let Some(parsed) = parsed else {
        return vec![];
    };

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
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性
    if value == "inherit" || value == "initial" || value == "unset" {
        return vec![mk(width_prop, value), mk(style_prop, value), mk(color_prop, value)];
    }

    let parsed = parse_border_shorthand(value);
    let Some(parsed) = parsed else {
        return vec![];
    };
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
fn parse_border_shorthand(value: &str) -> Option<BorderShorthand> {
    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut width = "medium".to_string();
    let mut style = "none".to_string();
    let mut color = "currentcolor".to_string();
    // CSS 规范：border 简写的 width/style/color 各至多一个；重复组件值（如
    // `border: red solid 16px red` 的两个 color）使整个声明非法，必须忽略。
    let mut seen_width = false;
    let mut seen_style = false;
    let mut seen_color = false;

    for part in parts {
        if is_border_style_keyword(part) {
            if seen_style {
                return None;
            }
            seen_style = true;
            style = part.to_string();
        } else if looks_like_length(part) {
            if seen_width {
                return None;
            }
            seen_width = true;
            width = part.to_string();
        } else if looks_like_color(part) {
            if seen_color {
                return None;
            }
            seen_color = true;
            color = part.to_string();
        }
    }

    Some(BorderShorthand { width, style, color })
}

/// 检查字符串是否为 border-style 关键字。
fn is_border_style_keyword(s: &str) -> bool {
    matches!(
        s,
        "none" | "hidden" | "dotted" | "dashed" | "solid" | "double" | "groove" | "ridge" | "inset" | "outset"
    )
}

/// 检查字符串是否看起来像长度值。
///
/// 单位后缀集须与 live `parse_length`（css-parser values/types.rs）保持一致，
/// 否则 border/outline 简写中的长度 token 会落回 medium/0 默认值
/// （R1231：缺 `ex` 致 `border-bottom: 6ex solid black` 丢 6ex → medium 3px，
/// top-091/092/bottom-091/092 簇 15-16% diff）。`%` 不列入——border-width/outline-width
/// 不接受百分比，`border: 5%` 应判非法落回默认（与 parse_length 接受 % 但 border-width
/// 拒绝无关）。`Q` 单位大小写敏感（CSS 规范 `Q`）。
fn looks_like_length(s: &str) -> bool {
    s.ends_with("px")
        || s.ends_with("em")
        || s.ends_with("ex")
        || s.ends_with("rem")
        || s.ends_with("pt")
        || s.ends_with("pc")
        || s.ends_with("cm")
        || s.ends_with("mm")
        || s.ends_with("Q")
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

/// 展开 flex 简写（CSS Flexbox §7.1）。
///
/// 语法：`none | auto | initial | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]`
///
/// 解析是 **type-based**（非位置式）：裸 `<number>` 计入 grow/shrink，任何
/// `<width>`/百分比/关键字（auto/content 等）计入 basis。因此：
/// - `none` → grow: 0, shrink: 0, basis: auto
/// - `auto` → grow: 1, shrink: 1, basis: auto
/// - `initial` → grow: 0, shrink: 1, basis: auto
/// - 单值 `<number>`（如 `flex:1`）→ grow, shrink: 1, basis: 0
/// - 单值 `<width>`（如 `flex:50%`）→ grow: 0, shrink: 1, basis: <width>
/// - 双值（如 `flex:1 2`）→ grow, shrink, basis: 0
/// - 双值（如 `flex:1 100px` / `flex:0 auto`）→ grow, shrink: 1, basis: <width>
/// - 三值 → grow, shrink, basis
fn expand_flex(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 判定一个 token 是否为裸 `<number>`（可作 flex-grow/shrink）。
    // `f64::parse` 拒绝带单位的长度/百分比（"100px"/"50%"），但接受 "inf"/"nan"，
    // 故用 `is_finite()` 排除这两个非有效 CSS 数字的边界。
    let is_number = |s: &str| s.parse::<f64>().map(|n| n.is_finite()).unwrap_or(false);

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
        // 单值：<number> → grow；否则（<width>/关键字）→ basis
        1 => {
            if is_number(parts[0]) {
                vec![mk("flex-grow", parts[0]), mk("flex-shrink", "1"), mk("flex-basis", "0")]
            } else {
                vec![mk("flex-grow", "0"), mk("flex-shrink", "1"), mk("flex-basis", parts[0])]
            }
        }
        // 双值：首值=grow；次值 <number> → shrink，否则 → basis（shrink 默认 1）
        2 => {
            let (grow, second) = (parts[0], parts[1]);
            if is_number(second) {
                vec![mk("flex-grow", grow), mk("flex-shrink", second), mk("flex-basis", "0")]
            } else {
                vec![mk("flex-grow", grow), mk("flex-shrink", "1"), mk("flex-basis", second)]
            }
        }
        // 三值：grow, shrink, basis
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

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
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

/// 展开 columns 简写。
///
/// CSS `columns` 简写格式为：
/// `columns: [column-count] [column-width]`
///
/// - 单个整数（如 `3`）→ column-count: 3, column-width: auto
/// - 单个长度值（如 `200px`）→ column-count: auto, column-width: 200px
/// - 双值（如 `3 200px`）→ column-count: 3, column-width: 200px
fn expand_columns(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    /// 检查值是否为有效的 column-count 值（正整数或 auto）。
    /// CSS Multicol §3.2：column-count 须为正整数；0 非法（zero-column-width-layout：
    /// `columns: 0` 的 0 不可归 column-count，须归 column-width）。
    fn is_valid_column_count(s: &str) -> bool {
        s == "auto" || s.parse::<u32>().is_ok_and(|n| n >= 1)
    }

    /// 检查值是否为有效的 column-width 值（长度或 auto）
    fn is_valid_column_width(s: &str) -> bool {
        s == "auto" || looks_like_length(s)
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            // 单值：判断是正整数（column-count）还是长度（column-width）。
            // 0 非正整数 → 归 column-width（CSS Multicol §3.2：column-count 须 ≥1）。
            let is_positive_int = parts[0].parse::<u32>().is_ok_and(|n| n >= 1);
            if is_positive_int || parts[0] == "auto" {
                vec![mk("column-count", parts[0]), mk("column-width", "auto")]
            } else if is_valid_column_width(parts[0]) {
                vec![mk("column-count", "auto"), mk("column-width", parts[0])]
            } else {
                // 无效值 — 整个声明无效
                vec![]
            }
        }
        2 => {
            // 双值：CSS Multicol §3.4 `columns: <column-width> || <column-count>`，顺序无关。
            // 消歧规则：正整数 → column-count；长度 → column-width；auto 填余下槽位。
            // R1425 修复：旧逻辑 `parts[0]=="auto"` 把 auto 当 count 指示，致 `auto 6` 误解析为
            // column-count:auto + column-width:6（应 column-count:6 + column-width:auto），使
            // `columns: auto N` 案（如 multicol-columns-007）列数变 auto → 退回 column-width 驱动。
            let p0_int = parts[0].parse::<u32>().is_ok_and(|n| n >= 1);
            let p1_int = parts[1].parse::<u32>().is_ok_and(|n| n >= 1);
            let (count_val, width_val) = if p0_int {
                (parts[0], parts[1])
            } else if p1_int {
                (parts[1], parts[0])
            } else if looks_like_length(parts[0]) {
                // parts[0] 是长度 → width，parts[1]（auto 或长度）→ count
                (parts[1], parts[0])
            } else {
                // parts[0] 非 int/length（即 auto）→ count，parts[1] → width
                (parts[0], parts[1])
            };
            // 验证两个值都有效：count 必须是整数/auto，width 必须是长度/auto
            if is_valid_column_count(count_val) && is_valid_column_width(width_val) {
                vec![mk("column-count", count_val), mk("column-width", width_val)]
            } else {
                // 无效值（如 "8 normal"）— 整个声明无效，不覆盖先前值
                vec![]
            }
        }
        _ => vec![],
    }
}

/// 展开 column-rule 简写。
///
/// CSS `column-rule` 简写格式为：
/// `column-rule: [width] [style] [color]`
///
/// 各部分顺序无关，未指定的部分使用初始值。
fn expand_column_rule(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
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

    vec![
        mk("column-rule-width", &width),
        mk("column-rule-style", &style),
        mk("column-rule-color", &color),
    ]
}

/// 展开 background 简写。
///
/// CSS 规范要求 `background` 简写必须展开为所有子属性。
/// 此实现解析每个 token 并分类到对应的子属性：
/// - 颜色值 → background-color
/// - url() / 渐变 → background-image
/// - repeat-x/repeat-y/repeat/no-repeat → background-repeat
/// - scroll/fixed/local → background-attachment
/// - 位置关键字/长度/百分比 → background-position
/// - border-box/padding-box/content-box → background-origin / background-clip
fn expand_background(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性
    if value == "inherit" || value == "initial" || value == "unset" {
        let subprops = [
            "background-color",
            "background-image",
            "background-repeat",
            "background-position",
            "background-size",
            "background-attachment",
            "background-clip",
            "background-origin",
        ];
        return subprops.iter().map(|p| mk(p, value)).collect();
    }

    let mut bg_color = String::new();
    let mut bg_image = String::new();
    let mut bg_repeat = String::new();
    let mut bg_attachment = String::new();
    let mut bg_position = String::new();

    // 渐变函数检查（优先于 var() 检查，因为渐变可能包含 var() 引用）
    let gradient_funcs = [
        "linear-gradient(",
        "repeating-linear-gradient(",
        "radial-gradient(",
        "repeating-radial-gradient(",
        "conic-gradient(",
        "repeating-conic-gradient(",
    ];
    for func in &gradient_funcs {
        if value.contains(func) {
            bg_image = value.to_string();
            return vec![
                mk("background-color", "transparent"),
                mk("background-image", &bg_image),
                mk("background-repeat", "repeat"),
                mk("background-position", "0% 0%"),
                mk("background-attachment", "scroll"),
                mk("background-clip", "border-box"),
                mk("background-origin", "padding-box"),
                mk("background-size", "auto"),
            ];
        }
    }

    // 如果包含 var() 或颜色函数 rgb()/rgba()/hsl()/hsla()，整体作为 background-color
    // 这些值包含逗号和空格，不能通过简单的 split_whitespace 解析
    if value.contains("var(")
        || value.contains("rgb(")
        || value.contains("rgba(")
        || value.contains("hsl(")
        || value.contains("hsla(")
    {
        bg_color = value.to_string();
        return vec![
            mk("background-color", &bg_color),
            mk("background-image", "none"),
            mk("background-repeat", "repeat"),
            mk("background-position", "0% 0%"),
            mk("background-attachment", "scroll"),
            mk("background-clip", "border-box"),
            mk("background-origin", "padding-box"),
            mk("background-size", "auto"),
        ];
    }

    // 如果包含 url()，提取 url() 部分作为 image，剩余 tokens 继续解析
    if value.contains("url(") {
        if let Some(start) = value.find("url(") {
            let mut depth = 0u32;
            let mut found_open = false;
            let mut end = start;
            for (i, c) in value[start..].char_indices() {
                if c == '(' {
                    depth += 1;
                    found_open = true;
                }
                if c == ')' && depth > 0 {
                    depth -= 1;
                }
                if found_open && depth == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            bg_image = value[start..end].to_string();
        }
        // 解析剩余部分（url() 之外的 tokens）
        let remaining = value.replace(&bg_image, "");
        for token in remaining.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            classify_bg_token_owned(
                token,
                &mut bg_color,
                &mut bg_repeat,
                &mut bg_attachment,
                &mut bg_position,
            );
        }
    } else {
        // 没有 url()，逐 token 解析
        for token in value.split_whitespace() {
            classify_bg_token_owned(
                token,
                &mut bg_color,
                &mut bg_repeat,
                &mut bg_attachment,
                &mut bg_position,
            );
        }
    }

    vec![
        mk(
            "background-color",
            if bg_color.is_empty() { "transparent" } else { &bg_color },
        ),
        mk("background-image", if bg_image.is_empty() { "none" } else { &bg_image }),
        mk(
            "background-repeat",
            if bg_repeat.is_empty() { "repeat" } else { &bg_repeat },
        ),
        mk(
            "background-position",
            if bg_position.is_empty() { "0% 0%" } else { &bg_position },
        ),
        mk(
            "background-attachment",
            if bg_attachment.is_empty() {
                "scroll"
            } else {
                &bg_attachment
            },
        ),
        mk("background-clip", "border-box"),
        mk("background-origin", "padding-box"),
        mk("background-size", "auto"),
    ]
}

/// 将 background 简写中的 token 分类到对应的子属性（owned String 版本）。
fn classify_bg_token_owned(
    token: &str,
    bg_color: &mut String,
    bg_repeat: &mut String,
    bg_attachment: &mut String,
    bg_position: &mut String,
) {
    // repeat 值
    match token {
        "repeat-x" | "repeat-y" | "repeat" | "no-repeat" | "space" | "round" => {
            *bg_repeat = token.to_string();
            return;
        }
        _ => {}
    }

    // attachment 值
    match token {
        "scroll" | "fixed" | "local" => {
            *bg_attachment = token.to_string();
            return;
        }
        _ => {}
    }

    // position 关键字
    match token {
        "top" | "center" | "bottom" | "left" | "right" => {
            if bg_position.is_empty() {
                *bg_position = token.to_string();
            } else {
                bg_position.push(' ');
                bg_position.push_str(token);
            }
            return;
        }
        _ => {}
    }

    // 如果 token 看起来像长度/百分比，归为 position
    if token.ends_with("px")
        || token.ends_with('%')
        || token.ends_with("em")
        || token.ends_with("rem")
        || token.ends_with("in")
        || token.ends_with("pt")
        || token.ends_with("pc")
        || token.ends_with("cm")
        || token.ends_with("mm")
        || token.ends_with("ch")
        || token.ends_with("vh")
        || token.ends_with("vw")
    {
        if bg_position.is_empty() {
            *bg_position = token.to_string();
        } else {
            // CSS 允许双值 background-position（如 "0.5in 0.5in"）
            // 累积第二个位置值
            bg_position.push(' ');
            bg_position.push_str(token);
        }
        return;
    }

    // box 值（origin/clip）— 简化处理，跳过
    match token {
        "border-box" | "padding-box" | "content-box" => {
            return;
        }
        _ => {}
    }

    // 默认：作为 background-color（颜色值）
    *bg_color = token.to_string();
}

/// 展开 font 简写。
///
/// 简化实现：`font: [style] [weight] <size>[/<line-height>] <family>`
/// 识别 font-weight 关键字、font-size 和 line-height（通过 `/` 分隔）以及 font-family。
fn expand_font(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    let mut weight = "normal".to_string();
    let mut style = "normal".to_string();
    let mut size = "medium".to_string();
    let mut line_height = "normal".to_string();
    let mut family = String::new();

    let is_weight = |s: &str| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "normal"
                | "bold"
                | "bolder"
                | "lighter"
                | "100"
                | "200"
                | "300"
                | "400"
                | "500"
                | "600"
                | "700"
                | "800"
                | "900"
        )
    };

    let is_style = |s: &str| matches!(s.to_ascii_lowercase().as_str(), "normal" | "italic" | "oblique");

    // 找到 size 部分：包含数字或者带 / 的部分
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut size_found = false;
    let mut family_parts: Vec<&str> = Vec::new();

    for part in &parts {
        if size_found {
            family_parts.push(part);
        } else if part.contains('/') {
            // size/line-height 格式
            let sub: Vec<&str> = part.splitn(2, '/').collect();
            size = sub[0].to_string();
            if sub.len() > 1 {
                line_height = sub[1].to_string();
            }
            size_found = true;
        } else if !size_found && (looks_like_length(part) || part.parse::<f64>().is_ok()) {
            size = part.to_string();
            size_found = true;
        } else if !size_found && is_weight(part) {
            weight = part.to_string();
        } else if !size_found && is_style(part) {
            style = part.to_string();
        }
    }

    if !family_parts.is_empty() {
        family = family_parts.join(" ");
    }

    // CSS 规范：font 简写必须至少包含 font-size 和 font-family
    // 如果没有找到 size 部分，声明无效（除非是系统字体关键字）
    if !size_found {
        // 检查是否为系统字体关键字
        let system_fonts = ["caption", "icon", "menu", "message-box", "small-caption", "status-bar"];
        if system_fonts.contains(&value.to_ascii_lowercase().as_str()) {
            // 系统字体关键字：设置所有子属性为 normal
            return vec![
                mk("font-style", "normal"),
                mk("font-weight", "normal"),
                mk("font-size", "medium"),
                mk("line-height", "normal"),
                mk("font-family", value),
            ];
        }
        // 无效的 font 简写声明
        return vec![];
    }

    // CSS 规范：font 简写中 line-height 为负值时，整个声明无效
    // CSS Fonts §3.7: "Values have the same meanings as for their non-shorthand equivalents.
    // Negative <line-height> values are illegal."
    if line_height.starts_with('-') {
        return vec![];
    }

    vec![
        mk("font-style", &style),
        mk("font-weight", &weight),
        mk("font-size", &size),
        mk("line-height", &line_height),
        mk("font-family", &family),
    ]
}

/// 展开 text-decoration 简写。
///
/// `text-decoration: [line] [style] [color]`
/// line: underline, overline, line-through, none
/// style: solid, double, dotted, dashed, wavy
/// color: 颜色值
fn expand_text_decoration(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if value == "none" {
        return vec![
            mk("text-decoration-line", "none"),
            mk("text-decoration-style", "solid"),
            mk("text-decoration-color", "currentcolor"),
        ];
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut line = "none".to_string();
    let mut dec_style = "solid".to_string();
    let mut color = "currentcolor".to_string();

    let is_line = |s: &str| matches!(s, "underline" | "overline" | "line-through" | "blink" | "none");

    let is_dec_style = |s: &str| matches!(s, "solid" | "double" | "dotted" | "dashed" | "wavy");

    for part in &parts {
        if is_line(part) {
            line = part.to_string();
        } else if is_dec_style(part) {
            dec_style = part.to_string();
        } else if looks_like_color(part) {
            color = part.to_string();
        }
    }

    vec![
        mk("text-decoration-line", &line),
        mk("text-decoration-style", &dec_style),
        mk("text-decoration-color", &color),
    ]
}

/// 展开 text-emphasis 简写（CSS Text Decoration 3 §3.1）。
///
/// `text-emphasis: <text-emphasis-style> || <text-emphasis-color>`
/// text-emphasis-color 暂未在 ComputedStyle 存储，故仅展开 style（剥离 color token，
/// 剩余 token 拼回作为 style 值，支持 `circle`、`filled circle`、`"*"` 等形式）。
fn expand_text_emphasis(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords：透传到 style longhand
    if value == "inherit" || value == "initial" || value == "unset" {
        return vec![mk("text-emphasis-style", value)];
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let mut style_parts: Vec<String> = Vec::new();
    for tok in &toks {
        if looks_like_color(tok) {
            // text-emphasis-color（ZW 暂未存储，剥离）
            continue;
        }
        style_parts.push(tok.clone());
    }
    if style_parts.is_empty() {
        return vec![];
    }
    vec![mk("text-emphasis-style", &style_parts.join(" "))]
}

#[cfg(test)]
mod tests;
