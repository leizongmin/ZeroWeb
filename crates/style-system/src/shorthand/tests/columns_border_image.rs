use super::super::*;

#[test]
/// columns 双值展开：3 200px → column-count=3, column-width=200px
fn test_columns_both_values() {
    let result = expand_one("columns", "3 200px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "3");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "200px");
}

#[test]
/// columns 单个整数展开：3 → column-count=3, column-width=auto
fn test_columns_count_only() {
    let result = expand_one("columns", "3", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "3");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "auto");
}

#[test]
/// columns 单个长度值展开：200px → column-count=auto, column-width=200px
fn test_columns_width_only() {
    let result = expand_one("columns", "200px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "auto");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "200px");
}

// ── R1425：columns 双值 auto/integer 消歧（CSS Multicol §3.4）──

#[test]
/// R1425：`auto 6` → column-count=6, column-width=auto（整数=count, auto=width）。
/// 旧实现误解析为 column-count:auto + column-width:6（parts[0]=="auto" 被当 count 指示），
/// 致 multicol-columns-007 列数变 auto 退回 column-width 驱动，paint multicol 路径不命中。
fn r1425_columns_auto_then_integer() {
    let result = expand_one("columns", "auto 6", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "6", "auto 6: 整数 6 应为 column-count");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "auto", "auto 6: auto 应为 column-width");
}

#[test]
/// R1425：`6 auto` → column-count=6, column-width=auto（顺序无关，整数总为 count）。
fn r1425_columns_integer_then_auto() {
    let result = expand_one("columns", "6 auto", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "6");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "auto");
}

#[test]
/// R1425：`100px auto` → column-count=auto, column-width=100px（长度=width, auto=count）。
fn r1425_columns_length_then_auto() {
    let result = expand_one("columns", "100px auto", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "auto", "100px auto: auto 应为 column-count");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "100px", "100px auto: 100px 应为 column-width");
}

#[test]
/// R1425：`auto 100px` → column-count=auto, column-width=100px（与上一测试互逆，顺序无关）。
fn r1425_columns_auto_then_length() {
    let result = expand_one("columns", "auto 100px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "auto");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "100px");
}

// ── column-rule 简写测试 ──

#[test]
/// column-rule 三值展开：2px solid blue → width, style, color
fn test_column_rule_three_values() {
    let result = expand_one("column-rule", "2px solid blue", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "column-rule-width");
    assert_eq!(result[0].1, "2px");
    assert_eq!(result[1].0, "column-rule-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "column-rule-color");
    assert_eq!(result[2].1, "blue");
}

#[test]
/// column-rule 单值展开：dotted → style=dotted，其余为默认值
fn test_column_rule_style_only() {
    let result = expand_one("column-rule", "dotted", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].1, "medium"); // 默认 width
    assert_eq!(result[1].1, "dotted"); // style
    assert_eq!(result[2].1, "currentcolor"); // 默认 color
}

#[test]
/// column-rule 双值展开：3px dashed → width=3px, style=dashed
fn test_column_rule_width_and_style() {
    let result = expand_one("column-rule", "3px dashed", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].1, "3px");
    assert_eq!(result[1].1, "dashed");
    assert_eq!(result[2].1, "currentcolor"); // 默认 color
}

// ── gap 简写测试 ──

#[test]
/// gap 简写单值：10px 同时应用于 gap、row-gap 和 column-gap
fn test_gap_shorthand_single_value() {
    let result = expand_one("gap", "10px", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "gap");
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].0, "row-gap");
    assert_eq!(result[1].1, "10px");
    assert_eq!(result[2].0, "column-gap");
    assert_eq!(result[2].1, "10px");
}

#[test]
/// gap 简写双值：10px 20px 分别应用于 gap、row-gap 和 column-gap
fn test_gap_shorthand_two_values() {
    let result = expand_one("gap", "10px 20px", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "gap");
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].0, "row-gap");
    assert_eq!(result[1].1, "10px");
    assert_eq!(result[2].0, "column-gap");
    assert_eq!(result[2].1, "20px");
}

#[test]
/// gap 简写三值及以上应为空
fn test_gap_shorthand_too_many_values() {
    let result = expand_one("gap", "10px 20px 30px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// gap 简写保留 important 和 specificity
fn test_gap_shorthand_preserves_important() {
    let result = expand_one("gap", "5px", true, (0, 1, 0));
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|(_, _, imp, _)| *imp));
    assert!(result.iter().all(|(_, _, _, spec)| *spec == (0, 1, 0)));
}

// ── border-image 简写测试 ──

#[test]
fn test_border_image_shorthand_source_only() {
    let result = expand_one("border-image", "url(test.png)", false, (0, 0, 1));
    assert!(
        result
            .iter()
            .any(|d| d.0 == "border-image-source" && d.1 == "url(test.png)")
    );
}

#[test]
fn test_border_image_shorthand_slice() {
    let result = expand_one("border-image", "25", false, (0, 0, 1));
    assert!(result.iter().any(|d| d.0 == "border-image-slice" && d.1 == "25"));
}

#[test]
fn test_border_image_shorthand_none() {
    let result = expand_one("border-image", "none", false, (0, 0, 1));
    assert!(result.iter().any(|d| d.0 == "border-image-source" && d.1 == "none"));
}

#[test]
fn test_border_image_shorthand_repeat() {
    let result = expand_one("border-image", "25 round", false, (0, 0, 1));
    assert!(result.iter().any(|d| d.0 == "border-image-slice" && d.1 == "25"));
    assert!(result.iter().any(|d| d.0 == "border-image-repeat" && d.1 == "round"));
}

#[test]
fn test_border_image_shorthand_with_slash() {
    let result = expand_one("border-image", "25 / 2", false, (0, 0, 1));
    assert!(result.iter().any(|d| d.0 == "border-image-slice" && d.1 == "25"));
    assert!(result.iter().any(|d| d.0 == "border-image-width" && d.1 == "2"));
}
