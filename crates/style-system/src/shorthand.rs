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
fn expand_one(
    property: &str,
    value: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
    };

    match property {
        // ── 4 边简写 ──
        "margin" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![mk("margin-top", t), mk("margin-right", r), mk("margin-bottom", b), mk("margin-left", l)]
        }
        "padding" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![mk("padding-top", t), mk("padding-right", r), mk("padding-bottom", b), mk("padding-left", l)]
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
        "border-top" => expand_border_side(value, "border-top-width", "border-top-style", "border-top-color", important, specificity),
        "border-right" => expand_border_side(value, "border-right-width", "border-right-style", "border-right-color", important, specificity),
        "border-bottom" => expand_border_side(value, "border-bottom-width", "border-bottom-style", "border-bottom-color", important, specificity),
        "border-left" => expand_border_side(value, "border-left-width", "border-left-style", "border-left-color", important, specificity),

        // ── border 全写 ──
        "border" => expand_border_all(value, important, specificity),

        // ── overflow ──
        "overflow" => {
            let v = value.trim();
            vec![mk("overflow-x", v), mk("overflow-y", v)]
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
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
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
    let parsed = parse_border_shorthand(value);
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
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
        "none" | "hidden" | "dotted" | "dashed" | "solid" | "double" | "groove" | "ridge"
            | "inset" | "outset"
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
        "black" | "white" | "red" | "green" | "blue" | "yellow" | "orange" | "purple"
            | "pink" | "brown" | "gray" | "grey" | "cyan" | "magenta" | "lime" | "maroon"
            | "navy" | "olive" | "teal" | "aqua" | "fuchsia" | "silver" | "gold" | "indigo"
            | "violet" | "coral" | "salmon" | "tomato" | "skyblue" | "tan" | "wheat"
            | "khaki" | "beige" | "ivory" | "snow" | "linen" | "azure" | "lavender"
            | "whitesmoke" | "gainsboro" | "lightgray" | "darkgray" | "dimgray"
            | "darkred" | "darkgreen" | "darkblue" | "lightblue" | "lightgreen" | "lightcoral"
            | "deeppink" | "hotpink" | "orangered" | "crimson" | "firebrick" | "chocolate"
            | "sienna" | "peru" | "goldenrod" | "darkgoldenrod" | "greenyellow"
            | "chartreuse" | "limegreen" | "palegreen" | "seagreen" | "forestgreen"
            | "yellowgreen" | "olivedrab" | "darkolivegreen" | "darkcyan" | "darkseagreen"
            | "lightseagreen" | "mediumseagreen" | "turquoise" | "darkturquoise"
            | "paleturquoise" | "deepskyblue" | "dodgerblue" | "cornflowerblue"
            | "royalblue" | "mediumblue" | "midnightblue" | "darkviolet" | "blueviolet"
            | "mediumpurple" | "darkorchid" | "orchid" | "plum" | "currentcolor"
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
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
    };
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
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
    };

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
        2 => vec![mk("flex-grow", parts[0]), mk("flex-shrink", parts[1]), mk("flex-basis", "0")],
        3 => vec![mk("flex-grow", parts[0]), mk("flex-shrink", parts[1]), mk("flex-basis", parts[2])],
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
    let mk = |prop: &str, val: &str| -> MatchingDecl {
        (prop.to_string(), val.to_string(), important, specificity)
    };

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
    s.ends_with("ms") || (s.ends_with('s') && !s.ends_with("ease"))
        && s.trim_end_matches("ms").trim_end_matches('s').parse::<f64>().is_ok()
}

/// 检查字符串是否为 timing-function 关键字。
fn is_timing_function_keyword(s: &str) -> bool {
    matches!(s, "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end")
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
            vec!["border-top-width", "border-right-width", "border-bottom-width", "border-left-width"]
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
}
