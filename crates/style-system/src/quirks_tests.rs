use super::*;
use zero_css_parser::values::LengthValue;

/// 测试 quirks mode 下百分比高度回退为 auto
///
/// 当父元素 height 为 auto 时，子元素的 height: <percentage> 应回退为 auto。
/// R2016：quirks mode 下百分比高度**保留**（不再 compute-to-auto）——layout 的
/// `apply_indefinite_percent_height_to_auto` quirks 分支按 ICB（viewport）解析。
/// （原规则反向：把 standards 的 compute-to-auto 误安到 quirks gate 上，已移除。）
#[test]
fn test_quirks_mode_percentage_height_fallback() {
    let mut child_style = ComputedStyle::default();
    child_style.height = LengthValue::Percentage(50.0);

    let parent_style = ComputedStyle::default();
    // parent height is Auto by default

    apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

    assert_eq!(
        child_style.height,
        LengthValue::Percentage(50.0),
        "Quirks mode must KEEP percentage height (resolved against viewport in layout), not convert to auto"
    );
}

/// 测试 quirks mode 下父元素有明确高度时百分比高度不变
#[test]
fn test_quirks_mode_percentage_height_kept_with_explicit_parent() {
    let mut child_style = ComputedStyle::default();
    child_style.height = LengthValue::Percentage(50.0);

    let mut parent_style = ComputedStyle::default();
    parent_style.height = LengthValue::Px(200.0);

    apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

    assert_eq!(
        child_style.height,
        LengthValue::Percentage(50.0),
        "Percentage height should be kept when parent has explicit height"
    );
}

/// 测试 quirks mode 下非百分比高度不受影响
#[test]
fn test_quirks_mode_px_height_unaffected() {
    let mut child_style = ComputedStyle::default();
    child_style.height = LengthValue::Px(100.0);

    let parent_style = ComputedStyle::default();

    apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

    assert_eq!(
        child_style.height,
        LengthValue::Px(100.0),
        "Px height should not be affected by quirks mode"
    );
}

/// 测试 quirks mode 下无父元素时百分比高度不变
#[test]
fn test_quirks_mode_percentage_height_no_parent() {
    let mut child_style = ComputedStyle::default();
    child_style.height = LengthValue::Percentage(50.0);

    apply_quirks_mode_adjustments(&mut child_style, None, None);

    assert_eq!(
        child_style.height,
        LengthValue::Percentage(50.0),
        "Percentage height should be kept when no parent style"
    );
}

/// 测试 quirks mode 下 table 元素的 height 转为 min-height
#[test]
fn test_quirks_mode_table_height_as_min_height() {
    let mut table_style = ComputedStyle::default();
    table_style.height = LengthValue::Px(300.0);

    let mut parent_style = ComputedStyle::default();
    parent_style.height = LengthValue::Px(600.0);

    apply_quirks_mode_adjustments(&mut table_style, Some(&parent_style), Some("table"));

    assert_eq!(
        table_style.height,
        LengthValue::Auto,
        "Table height should be set to auto in quirks mode"
    );
    assert_eq!(
        table_style.min_height,
        LengthValue::Px(300.0),
        "Table height value should be moved to min-height in quirks mode"
    );
}

/// 测试 quirks mode 下非 table 元素的 height 不受影响
#[test]
fn test_quirks_mode_non_table_height_unaffected() {
    let mut div_style = ComputedStyle::default();
    div_style.height = LengthValue::Px(300.0);

    let mut parent_style = ComputedStyle::default();
    parent_style.height = LengthValue::Px(600.0);

    apply_quirks_mode_adjustments(&mut div_style, Some(&parent_style), Some("div"));

    assert_eq!(
        div_style.height,
        LengthValue::Px(300.0),
        "Non-table element height should not be affected by table quirk"
    );
}

/// 测试 quirks mode 下 table 元素 auto height 不受影响
#[test]
fn test_quirks_mode_table_auto_height_unaffected() {
    let mut table_style = ComputedStyle::default();
    // height is Auto by default

    let parent_style = ComputedStyle::default();

    apply_quirks_mode_adjustments(&mut table_style, Some(&parent_style), Some("table"));

    assert_eq!(
        table_style.height,
        LengthValue::Auto,
        "Table with auto height should remain auto"
    );
}
