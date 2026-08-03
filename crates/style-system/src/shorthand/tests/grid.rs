use super::super::*;

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
//
// R2487：list-style 简写现展开全部 3 个 longhand（type/position/image）。旧实现仅展开
// type+position 且 is_type 硬编码枚举，漏自定义 @counter-style 名与 R2445-R2450 预定义
// 计数器样式（lower-greek/georgian/hebrew/...），并丢弃 list-style-image。

/// 从展开结果中按 longhand 名取值（便于多 longhand 断言）。
fn ls_get<'a>(result: &'a [(String, String, bool, (u32, u32, u32))], prop: &str) -> &'a str {
    result
        .iter()
        .find(|(p, _, _, _)| p == prop)
        .map(|(_, v, _, _)| v.as_str())
        .unwrap_or("")
}

#[test]
/// list-style: none → type=none, position=outside, image=none（none 同时置 type+image）
fn test_list_style_none() {
    let result = expand_one("list-style", "none", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(ls_get(&result, "list-style-type"), "none");
    assert_eq!(ls_get(&result, "list-style-position"), "outside");
    assert_eq!(ls_get(&result, "list-style-image"), "none");
}

#[test]
/// list-style: inside → type=disc(初始), position=inside, image=none
fn test_list_style_position_only() {
    let result = expand_one("list-style", "inside", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(ls_get(&result, "list-style-type"), "disc");
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
    assert_eq!(ls_get(&result, "list-style-image"), "none");
}

#[test]
/// list-style: square inside → type=square, position=inside, image=none
fn test_list_style_type_and_position() {
    let result = expand_one("list-style", "square inside", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(ls_get(&result, "list-style-type"), "square");
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
    assert_eq!(ls_get(&result, "list-style-image"), "none");
}

#[test]
/// list-style: decimal outside → type=decimal, position=outside
fn test_list_style_decimal_outside() {
    let result = expand_one("list-style", "decimal outside", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-type"), "decimal");
    assert_eq!(ls_get(&result, "list-style-position"), "outside");
}

#[test]
/// list-style: lower-roman inside → type=lower-roman, position=inside
fn test_list_style_lower_roman_inside() {
    let result = expand_one("list-style", "lower-roman inside", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-type"), "lower-roman");
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
}

#[test]
/// list-style 保留 important 标记与 specificity（3 longhand 全部）
fn test_list_style_preserves_important() {
    let result = expand_one("list-style", "square inside", true, (0, 1, 0));
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|(_, _, imp, _)| *imp));
    assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
}

// ── R2487 driving：修旧实现缺口（自定义/计数器样式 + image + none 双语义 + 大小写） ──

#[test]
/// R2487：自定义 @counter-style 名作 type（旧实现落「其他值」→ type 退回 disc）
fn test_list_style_custom_counter_style_name() {
    let result = expand_one("list-style", "Custom-Style inside", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-type"), "Custom-Style");
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
    assert_eq!(ls_get(&result, "list-style-image"), "none");
}

#[test]
/// R2487：R2445-R2450 预定义计数器样式作 type（lower-greek/georgian 旧 is_type 枚举漏）
fn test_list_style_predefined_counter_style() {
    for style in ["lower-greek", "georgian", "hebrew", "arabic-indic", "cjk-decimal"] {
        let result = expand_one("list-style", &format!("{style} inside"), false, (0, 0, 1));
        assert_eq!(ls_get(&result, "list-style-type"), style, "type for {style}");
        assert_eq!(ls_get(&result, "list-style-position"), "inside");
    }
}

#[test]
/// R2487：list-style-image（url()）现展开（旧实现完全丢弃）
fn test_list_style_image_url() {
    let result = expand_one("list-style", "disc url(support/green15x15.png)", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(ls_get(&result, "list-style-type"), "disc");
    assert_eq!(ls_get(&result, "list-style-position"), "outside");
    assert_eq!(ls_get(&result, "list-style-image"), "url(support/green15x15.png)");
}

#[test]
/// R2487：paren-aware——url() 含空格不被 split（url('a b.png') 保持一体）
fn test_list_style_image_url_with_space() {
    let result = expand_one("list-style", "inside url('support/green 15.png')", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
    assert_eq!(ls_get(&result, "list-style-image"), "url('support/green 15.png')");
    assert_eq!(ls_get(&result, "list-style-type"), "disc");
}

#[test]
/// R2487：none 双语义——`none square url(...)` → type=square, image=url, position=outside
/// （none 不冲突显式 type/image；CSS Lists 3「other values assigned to property they fit」）
fn test_list_style_none_with_type_and_image() {
    let result = expand_one(
        "list-style",
        "none square url(support/swatch-red.png)",
        false,
        (0, 0, 1),
    );
    assert_eq!(ls_get(&result, "list-style-type"), "square");
    assert_eq!(ls_get(&result, "list-style-image"), "url(support/swatch-red.png)");
    assert_eq!(ls_get(&result, "list-style-position"), "outside");
}

#[test]
/// R2487：仅 image（无 type）→ type=初始 disc, image=url, position=outside
fn test_list_style_image_only_type_initial_disc() {
    let result = expand_one("list-style", "url(star.png)", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-type"), "disc");
    assert_eq!(ls_get(&result, "list-style-image"), "url(star.png)");
    assert_eq!(ls_get(&result, "list-style-position"), "outside");
}

#[test]
/// R2487：type token 大小写保留（计数器样式名大小写敏感，交 longhand parser 匹配）
fn test_list_style_type_case_preserved() {
    let result = expand_one("list-style", "Hiragana inside", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-type"), "Hiragana");
}

#[test]
/// R2487：position 关键字大小写不敏感（归一为小写）
fn test_list_style_position_case_insensitive() {
    let result = expand_one("list-style", "disc INSIDE", false, (0, 0, 1));
    assert_eq!(ls_get(&result, "list-style-position"), "inside");
}

#[test]
/// R2487：CSS-wide keyword 透传 3 longhand（含 image，旧 wide-keyword 漏 image）
fn test_list_style_wide_keyword_includes_image() {
    let result = expand_one("list-style", "inherit", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|(_, v, _, _)| v == "inherit"));
    assert!(result.iter().any(|(p, _, _, _)| p == "list-style-image"));
}

// ── overscroll-behavior 简写测试 ──

#[test]
/// overscroll-behavior 单值：contain → x=contain, y=contain
fn test_overscroll_behavior_single_value() {
    let result = expand_one("overscroll-behavior", "contain", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "overscroll-behavior-x");
    assert_eq!(result[0].1, "contain");
    assert_eq!(result[1].0, "overscroll-behavior-y");
    assert_eq!(result[1].1, "contain");
}

#[test]
/// overscroll-behavior 双值：auto none → x=auto, y=none
fn test_overscroll_behavior_two_values() {
    let result = expand_one("overscroll-behavior", "auto none", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "overscroll-behavior-x");
    assert_eq!(result[0].1, "auto");
    assert_eq!(result[1].0, "overscroll-behavior-y");
    assert_eq!(result[1].1, "none");
}

// ═══════════════════════════════════════════════════════════════════
// 新增简写展开测试（test_shorthand_<property>_<scenario>）
// ═══════════════════════════════════════════════════════════════════

#[test]
/// background 简写展开：颜色值 → 展开为所有子属性，颜色设为 background-color
fn test_shorthand_background_color() {
    let result = expand_one("background", "red", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "red");
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "none");
}

#[test]
/// background 简写展开：url() 值 → 展开为所有子属性，image 设为 background-image
fn test_shorthand_background_image() {
    let result = expand_one("background", "url(img/bg.png)", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "transparent");
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "url(img/bg.png)");
}

#[test]
/// border-radius 单值展开：10px → 四个角均为 10px
fn test_shorthand_border_radius_single_value() {
    let result = expand_one("border-radius", "10px", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "border-top-left-radius");
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].0, "border-top-right-radius");
    assert_eq!(result[1].1, "10px");
    assert_eq!(result[2].0, "border-bottom-right-radius");
    assert_eq!(result[2].1, "10px");
    assert_eq!(result[3].0, "border-bottom-left-radius");
    assert_eq!(result[3].1, "10px");
}

#[test]
/// border-radius 双值展开：10px 20px → top-left/bottom-right=10px, top-right/bottom-left=20px
fn test_shorthand_border_radius_two_values() {
    let result = expand_one("border-radius", "10px 20px", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "10px"); // top-left
    assert_eq!(result[1].1, "20px"); // top-right
    assert_eq!(result[2].1, "10px"); // bottom-right
    assert_eq!(result[3].1, "20px"); // bottom-left
}

#[test]
/// font 简写展开：bold 16px/1.5 sans-serif → font-weight, font-size, line-height, font-family
fn test_shorthand_font_bold_size_line_family() {
    let result = expand_one("font", "bold 16px/1.5 sans-serif", false, (0, 0, 1));
    assert_eq!(result.len(), 5);
    assert_eq!(result[0].0, "font-style");
    assert_eq!(result[0].1, "normal");
    assert_eq!(result[1].0, "font-weight");
    assert_eq!(result[1].1, "bold");
    assert_eq!(result[2].0, "font-size");
    assert_eq!(result[2].1, "16px");
    assert_eq!(result[3].0, "line-height");
    assert_eq!(result[3].1, "1.5");
    assert_eq!(result[4].0, "font-family");
    assert_eq!(result[4].1, "sans-serif");
}

#[test]
/// list-style 简写展开：disc inside → list-style-type=disc, list-style-position=inside（R2487：+image=none）
fn test_shorthand_list_style_disc_inside() {
    let result = expand_one("list-style", "disc inside", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "list-style-type");
    assert_eq!(result[0].1, "disc");
    assert_eq!(result[1].0, "list-style-position");
    assert_eq!(result[1].1, "inside");
    assert_eq!(result[2].0, "list-style-image");
    assert_eq!(result[2].1, "none");
}

#[test]
/// text-decoration 简写展开：underline dotted red → line, style, color, thickness
/// （R2592：CSS Text Decoration 4 简写含第 4 个 longhand text-decoration-thickness，
/// 未显式给定则重置为 initial auto）
fn test_shorthand_text_decoration_line_style_color() {
    let result = expand_one("text-decoration", "underline dotted red", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "underline");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "dotted");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "red");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// R2592：text-decoration 简写中的 thickness token 路由到 text-decoration-thickness longhand。
/// driving: css-text-decor text-decoration-shorthands-001(auto)/002(100px)。
fn test_shorthand_text_decoration_thickness_routing() {
    // 显式长度 thickness（driving: text-decoration-shorthands-002 green underline 100px）
    let r = expand_one("text-decoration", "green underline 100px", false, (0, 0, 1));
    let thickness = r.iter().find(|(p, _, _, _)| p == "text-decoration-thickness").unwrap();
    assert_eq!(thickness.1, "100px");

    // auto 关键字（driving: text-decoration-shorthands-001 green underline auto）
    let r = expand_one("text-decoration", "green underline auto", false, (0, 0, 1));
    let thickness = r.iter().find(|(p, _, _, _)| p == "text-decoration-thickness").unwrap();
    assert_eq!(thickness.1, "auto");

    // from-font 关键字
    let r = expand_one("text-decoration", "underline from-font", false, (0, 0, 1));
    let thickness = r.iter().find(|(p, _, _, _)| p == "text-decoration-thickness").unwrap();
    assert_eq!(thickness.1, "from-font");

    // 百分比 thickness
    let r = expand_one("text-decoration", "underline 50%", false, (0, 0, 1));
    let thickness = r.iter().find(|(p, _, _, _)| p == "text-decoration-thickness").unwrap();
    assert_eq!(thickness.1, "50%");

    // 无 thickness token → 重置为 initial auto
    let r = expand_one("text-decoration", "underline", false, (0, 0, 1));
    let thickness = r.iter().find(|(p, _, _, _)| p == "text-decoration-thickness").unwrap();
    assert_eq!(thickness.1, "auto");
}

#[test]
/// 无效简写值应返回空 vec
fn test_shorthand_invalid_returns_empty() {
    let result = expand_one("margin", "1 2 3 4 5 6 7", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// overscroll-behavior 双值展开：contain none → x=contain, y=none
fn test_shorthand_overscroll_behavior_double_value() {
    let result = expand_one("overscroll-behavior", "contain none", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "overscroll-behavior-x");
    assert_eq!(result[0].1, "contain");
    assert_eq!(result[1].0, "overscroll-behavior-y");
    assert_eq!(result[1].1, "none");
}
