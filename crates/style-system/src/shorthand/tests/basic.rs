use super::super::*;

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
fn test_border_edge_quartet_rejects_invalid_tokens() {
    assert!(expand_one("border-width", "red", false, (0, 0, 1)).is_empty());
    assert!(expand_one("border-width", "-1px", false, (0, 0, 1)).is_empty());
    assert!(expand_one("border-style", "1px", false, (0, 0, 1)).is_empty());
    assert!(expand_one("border-color", "solid", false, (0, 0, 1)).is_empty());

    let result = expand_one("border-width", "thin medium thick 2px", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "thin");
    assert_eq!(result[3].1, "2px");

    let inherit = expand_one("border-style", "inherit", false, (0, 0, 1));
    assert_eq!(inherit.len(), 4);
    assert!(inherit.iter().all(|(_, value, _, _)| value == "inherit"));
}

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

/// border 简写的 rgba 颜色含逗号后空格（标准格式）必须保持完整 token。
///
/// 此前 `parse_border_shorthand` 用 `split_whitespace()` 把 `rgba(255, 0, 0, 0.3)`
/// 拆碎，颜色退化成碎片或 currentcolor（→黑）。修复后括号感知分割保留完整 rgba。
#[test]
fn test_border_all_rgba_with_spaces_keeps_color() {
    let result = expand_one("border", "6px solid rgba(255, 0, 0, 0.3)", false, (0, 0, 1));
    // border-top-color 应为完整 rgba（非碎片 / 非 currentcolor）
    assert_eq!(result[2].0, "border-top-color");
    assert_eq!(
        result[2].1, "rgba(255, 0, 0, 0.3)",
        "spaced rgba in border shorthand must stay intact (was fragmented → black)"
    );
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
fn test_border_radius_rejects_invalid_tokens() {
    assert!(expand_one("border-radius", "red", false, (0, 0, 1)).is_empty());
    assert!(expand_one("border-radius", "-1px", false, (0, 0, 1)).is_empty());
    let result = expand_one("border-radius", "10% 2px", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "10%");
    assert_eq!(result[1].1, "2px");

    let inherit = expand_one("border-radius", "inherit", false, (0, 0, 1));
    assert_eq!(inherit.len(), 4);
    assert!(inherit.iter().all(|(_, value, _, _)| value == "inherit"));
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
    assert_eq!(result[2].1, "0%"); // basis（R2754：省略 basis→0%，对齐 Chromium）
}

#[test]
fn test_flex_two_values() {
    let result = expand_one("flex", "2 1", false, (0, 0, 1));
    assert_eq!(result[0].1, "2"); // grow
    assert_eq!(result[1].1, "1"); // shrink
    assert_eq!(result[2].1, "0%"); // basis（R2754：省略 basis→0%）
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

// 从展开结果中取出 (grow, shrink, basis)。
fn flex_parts(value: &str) -> (String, String, String) {
    let result = expand_one("flex", value, false, (0, 0, 1));
    let get = |prop: &str| -> String {
        result
            .iter()
            .find(|(p, _, _, _)| p == prop)
            .map(|(_, v, _, _)| v.clone())
            .unwrap_or_default()
    };
    (get("flex-grow"), get("flex-shrink"), get("flex-basis"))
}

#[test]
fn test_expand_flex_single_number_is_grow() {
    // 纯数字单值 → grow（CSS §7.1.1，省略 basis→0%，R2754 对齐 Chromium）
    assert_eq!(flex_parts("1"), ("1".to_string(), "1".to_string(), "0%".to_string()));
    assert_eq!(
        flex_parts("2.5"),
        ("2.5".to_string(), "1".to_string(), "0%".to_string())
    );
}

#[test]
fn test_expand_flex_single_width_is_basis() {
    // 非数字单值 → basis（纠正旧位置式把宽度当 grow 的 bug）
    assert_eq!(flex_parts("50%"), ("0".to_string(), "1".to_string(), "50%".to_string()));
    assert_eq!(
        flex_parts("100px"),
        ("0".to_string(), "1".to_string(), "100px".to_string())
    );
    assert_eq!(
        flex_parts("10em"),
        ("0".to_string(), "1".to_string(), "10em".to_string())
    );
}

#[test]
fn test_expand_flex_two_numbers_are_grow_shrink() {
    // 双数字 → grow/shrink，省略 basis→0%（R2754 对齐 Chromium）
    assert_eq!(flex_parts("1 2"), ("1".to_string(), "2".to_string(), "0%".to_string()));
    assert_eq!(flex_parts("0 0"), ("0".to_string(), "0".to_string(), "0%".to_string()));
}

#[test]
fn test_expand_flex_number_then_width_is_grow_basis() {
    // 双值中次值非数字 → basis，shrink 默认 1（纠正旧位置式把宽度当 shrink 的 bug）
    assert_eq!(
        flex_parts("1 100px"),
        ("1".to_string(), "1".to_string(), "100px".to_string())
    );
    assert_eq!(
        flex_parts("0 auto"),
        ("0".to_string(), "1".to_string(), "auto".to_string())
    );
    assert_eq!(
        flex_parts("1 auto"),
        ("1".to_string(), "1".to_string(), "auto".to_string())
    );
}

#[test]
fn test_expand_flex_three_values() {
    assert_eq!(
        flex_parts("2 1 100px"),
        ("2".to_string(), "1".to_string(), "100px".to_string())
    );
    assert_eq!(
        flex_parts("0 0 auto"),
        ("0".to_string(), "0".to_string(), "auto".to_string())
    );
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
    assert!(looks_like_length("6ex"));
    assert!(looks_like_length("1Q"));
    assert!(looks_like_length("0"));
    assert!(looks_like_length("thin"));
    assert!(!looks_like_length("solid"));
    assert!(!looks_like_length("red"));
    assert!(!looks_like_length("begin"));
    assert!(!looks_like_length("auto"));
    assert!(!looks_like_length("min-content"));
    assert!(!looks_like_length("fit-content(10px)"));
    // % 不算 length——border-width/outline-width 不接受百分比
    assert!(!looks_like_length("50%"));
}

#[test]
fn test_looks_like_color() {
    assert!(looks_like_color("#fff"));
    assert!(looks_like_color("rgb(255,0,0)"));
    assert!(looks_like_color("red"));
    assert!(looks_like_color("transparent"));
    assert!(!looks_like_color("10px"));
    assert!(!looks_like_color("solid"));
    assert!(!looks_like_color("begin"));
    assert!(!looks_like_color("rgbfoo"));
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

#[test]
fn test_border_shorthand_rejects_ident_ending_with_length_unit() {
    let result = expand_one("border", "begin solid red", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
fn test_text_emphasis_shorthand_style_only() {
    // text-emphasis: circle → text-emphasis-style: circle
    let result = expand_one("text-emphasis", "circle", false, (0, 0, 1));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "text-emphasis-style");
    assert_eq!(result[0].1, "circle");
}

#[test]
fn test_text_emphasis_shorthand_expands_color() {
    // R2523：text-emphasis: filled circle red → color + style 两条 longhand
    let result = expand_one("text-emphasis", "filled circle red", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "text-emphasis-color");
    assert_eq!(result[0].1, "red");
    assert_eq!(result[1].0, "text-emphasis-style");
    assert_eq!(result[1].1, "filled circle");
}

#[test]
fn test_text_emphasis_shorthand_string() {
    // text-emphasis: "*" → style: "*"（自定义字符）
    let result = expand_one("text-emphasis", "\"*\"", false, (0, 0, 1));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "text-emphasis-style");
    assert_eq!(result[0].1, "\"*\"");
}

#[test]
/// R2132：简写值首尾空白守卫——值字符串首尾的空白只能来自转义（consume_declaration
/// deferred-ws 已保证无首尾空白 token），应丢弃整个简写声明（与 chromium 一致：
/// 含转义首尾空白的简写值非法）。driving：escapes-014 `background:\0020red` → `" red"`。
/// 对照：普通值无首尾空白（deferred-ws）不受影响。
fn test_shorthand_boundary_whitespace_drops_declaration() {
    // 转义产生的首部空白（`\0020red` → " red"）→ 丢弃，不剥成 "red" 误应用。
    let result = expand_one("background", " red", false, (0, 0, 1));
    assert!(result.is_empty(), "leading escape-ws should drop shorthand");

    // 转义产生的尾部空白（`red\0020` → "red "）→ 丢弃。
    let result = expand_one("background", "red ", false, (0, 0, 1));
    assert!(result.is_empty(), "trailing escape-ws should drop shorthand");

    // 对照：普通简写值无首尾空白 → 正常展开。
    let result = expand_one("background", "red", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "red");

    // 对照：内部空白（多 token）正常切分，不受守卫影响。
    let result = expand_one("margin", "10px 20px", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].1, "20px");
}

// ── R2879：background 简写 gradient+color+size 拆分（R2877 重写 + R2878 size:0 渲染器）──

#[test]
fn test_background_shorthand_gradient_color_size_split() {
    // `green linear-gradient(red,red) center / 0 0` 应拆为：
    // color=green / image=gradient / position=center / size="0 0"（旧 gradient 早返回把整串当 image）。
    let result = expand_one(
        "background",
        "green linear-gradient(red, red) center / 0 0",
        false,
        (0, 0, 1),
    );
    let map: std::collections::HashMap<&str, &str> =
        result.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert_eq!(map.get("background-color"), Some(&"green"));
    assert_eq!(map.get("background-image"), Some(&"linear-gradient(red, red)"));
    assert_eq!(map.get("background-position"), Some(&"center"));
    // bare-0 token（unitless-zero）归 size 而非 color（R2878 classify_bg_token 修正）。
    assert_eq!(map.get("background-size"), Some(&"0 0"));
}

#[test]
fn test_background_shorthand_bare_gradient_alone() {
    // 纯渐变（无 color/position/size）仍正确——image=渐变，color=transparent（默认）。
    let result = expand_one("background", "linear-gradient(green, green)", false, (0, 0, 1));
    let map: std::collections::HashMap<&str, &str> =
        result.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert_eq!(map.get("background-image"), Some(&"linear-gradient(green, green)"));
    assert_eq!(map.get("background-color"), Some(&"transparent"));
}
