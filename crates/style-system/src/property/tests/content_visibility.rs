//! CSS content-visibility 样式管线测试（R2251，CSS Containment Module Level 2）。
//!
//! 验证 content-visibility 解析进入 ComputedStyle（apply 分发）。

use super::super::*;

#[test]
fn test_content_visibility_apply_hidden() {
    let mut style = ComputedStyle::default();
    // 初始值 = Visible
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
    assert!(apply_property_value(&mut style, "content-visibility", "hidden"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Hidden));
}

#[test]
fn test_content_visibility_apply_auto_and_visible() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content-visibility", "auto"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Auto));
    assert!(apply_property_value(&mut style, "content-visibility", "visible"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
}

#[test]
fn test_content_visibility_apply_invalid_rejected() {
    let mut style = ComputedStyle::default();
    // 非法值不应用（保持默认 Visible），返回 false
    assert!(!apply_property_value(&mut style, "content-visibility", "inherit"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
}

#[test]
/// content-visibility:hidden 的「跳过内容」效果仅对 size-containment 适用盒生效（CSS Containment 2）。
/// 无主盒（display:none/contents）与非替换 inline 盒无效。WPT driving：
/// content-visibility-on-display-contents / content-visibility-on-ruby / content-visibility-073。
fn test_content_visibility_hidden_effective_display_gate() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "content-visibility", "hidden");

    // 默认 display:inline → 非原子 inline，content-visibility:hidden 无视觉效果。
    assert!(!style.content_visibility_hidden_effective(), "inline: 无效果");

    style.display = DisplayValue::Block;
    assert!(style.content_visibility_hidden_effective(), "block: 生效");

    style.display = DisplayValue::InlineBlock;
    assert!(
        style.content_visibility_hidden_effective(),
        "inline-block（atomic inline）: 生效"
    );

    style.display = DisplayValue::Contents;
    assert!(!style.content_visibility_hidden_effective(), "contents: 无主盒无效");

    style.display = DisplayValue::None;
    assert!(!style.content_visibility_hidden_effective(), "none: 无主盒无效");

    // visible/auto 永不生效
    style.display = DisplayValue::Block;
    apply_property_value(&mut style, "content-visibility", "visible");
    assert!(!style.content_visibility_hidden_effective());
    apply_property_value(&mut style, "content-visibility", "auto");
    assert!(
        !style.content_visibility_hidden_effective(),
        "auto 静态等价 visible，不跳过"
    );
}
