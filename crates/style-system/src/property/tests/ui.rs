// Auto-generated test file — split from property.rs
use super::super::*;

#[test]
fn test_apply_property_background_size_length() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "100px"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Length(100.0)]);
}

#[test]
fn test_apply_property_background_size_percent() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "50%"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Percent(50.0)]);
}

#[test]
fn test_apply_property_background_size_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-size", "invalid"));
}

#[test]
fn test_background_size_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-size"));
}

#[test]
fn test_background_size_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-size"));
}

#[test]
fn test_background_size_initial_value() {
    assert!(PropertyRegistry::initial_value("background-size").is_some());
    let mut style = ComputedStyle::default();
    style.background_size = vec![BackgroundSizeComputedValue::Cover];
    assert!(apply_initial_value(&mut style, "background-size"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Auto]);
}

// ── background-attachment 属性测试 ──

#[test]
fn test_apply_property_background_attachment_scroll() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-attachment", "scroll"));
    assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Scroll);
}

#[test]
fn test_apply_property_background_attachment_fixed() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-attachment", "fixed"));
    assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Fixed);
}

#[test]
fn test_apply_property_background_attachment_local() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-attachment", "local"));
    assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Local);
}

#[test]
fn test_apply_property_background_attachment_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-attachment", "invalid"));
}

#[test]
fn test_background_attachment_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-attachment"));
}

#[test]
fn test_background_attachment_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-attachment"));
}

#[test]
fn test_background_attachment_initial_value() {
    assert!(PropertyRegistry::initial_value("background-attachment").is_some());
    let mut style = ComputedStyle::default();
    style.background_attachment = BackgroundAttachmentComputedValue::Fixed;
    assert!(apply_initial_value(&mut style, "background-attachment"));
    assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Scroll);
}

// ── background-clip ──

#[test]
fn test_apply_property_background_clip_border_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-clip", "border-box"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::BorderBox);
}

#[test]
fn test_apply_property_background_clip_padding_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-clip", "padding-box"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::PaddingBox);
}

#[test]
fn test_apply_property_background_clip_content_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-clip", "content-box"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::ContentBox);
}

#[test]
fn test_apply_property_background_clip_text() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-clip", "text"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::Text);
}

#[test]
fn test_apply_property_background_clip_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-clip", "invalid"));
}

#[test]
fn test_background_clip_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-clip"));
}

#[test]
fn test_background_clip_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-clip"));
}

#[test]
fn test_background_clip_initial_value() {
    assert!(PropertyRegistry::initial_value("background-clip").is_some());
    let mut style = ComputedStyle::default();
    style.background_clip = BackgroundClipComputedValue::Text;
    assert!(apply_initial_value(&mut style, "background-clip"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::BorderBox);
}

// ── background-origin ──

#[test]
fn test_apply_property_background_origin_padding_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-origin", "padding-box"));
    assert_eq!(style.background_origin, BackgroundOriginComputedValue::PaddingBox);
}

#[test]
fn test_apply_property_background_origin_border_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-origin", "border-box"));
    assert_eq!(style.background_origin, BackgroundOriginComputedValue::BorderBox);
}

#[test]
fn test_apply_property_background_origin_content_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-origin", "content-box"));
    assert_eq!(style.background_origin, BackgroundOriginComputedValue::ContentBox);
}

#[test]
fn test_apply_property_background_origin_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-origin", "invalid"));
    // text 不是有效的 background-origin 值
    assert!(!apply_property_value(&mut style, "background-origin", "text"));
}

#[test]
fn test_background_origin_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-origin"));
}

#[test]
fn test_background_origin_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-origin"));
}

#[test]
fn test_background_origin_initial_value() {
    assert!(PropertyRegistry::initial_value("background-origin").is_some());
    let mut style = ComputedStyle::default();
    style.background_origin = BackgroundOriginComputedValue::ContentBox;
    assert!(apply_initial_value(&mut style, "background-origin"));
    assert_eq!(style.background_origin, BackgroundOriginComputedValue::PaddingBox);
}

// ── border-image-source ──

#[test]
fn test_apply_property_border_image_source_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-source", "none"));
    assert_eq!(style.border_image_source, BorderImageSourceComputedValue::None);
}

#[test]
fn test_apply_property_border_image_source_url() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "border-image-source",
        "url(border.png)"
    ));
    assert_eq!(
        style.border_image_source,
        BorderImageSourceComputedValue::Url("border.png".to_string())
    );
}

#[test]
fn test_apply_property_border_image_source_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-image-source", "invalid"));
}

#[test]
fn test_border_image_source_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("border-image-source"));
}

#[test]
fn test_border_image_source_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"border-image-source"));
}

#[test]
fn test_border_image_source_initial_value() {
    assert!(PropertyRegistry::initial_value("border-image-source").is_some());
    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("test.png".to_string());
    assert!(apply_initial_value(&mut style, "border-image-source"));
    assert_eq!(style.border_image_source, BorderImageSourceComputedValue::None);
}

// ── border-image-slice ──

#[test]
fn test_apply_property_border_image_slice_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-slice", "50"));
    assert_eq!(
        style.border_image_slice.top,
        BorderImageSliceComputedComponent::Number(50.0)
    );
    assert_eq!(
        style.border_image_slice.right,
        BorderImageSliceComputedComponent::Number(50.0)
    );
}

#[test]
fn test_apply_property_border_image_slice_percent() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-slice", "30%"));
    assert_eq!(
        style.border_image_slice.top,
        BorderImageSliceComputedComponent::Percent(30.0)
    );
}

#[test]
fn test_apply_property_border_image_slice_fill() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-slice", "25 fill"));
    assert!(style.border_image_slice.fill);
    assert_eq!(
        style.border_image_slice.top,
        BorderImageSliceComputedComponent::Number(25.0)
    );
}

#[test]
fn test_apply_property_border_image_slice_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-image-slice", "invalid"));
}

#[test]
fn test_border_image_slice_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("border-image-slice"));
}

#[test]
fn test_border_image_slice_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"border-image-slice"));
}

#[test]
fn test_border_image_slice_initial_value() {
    assert!(PropertyRegistry::initial_value("border-image-slice").is_some());
    let mut style = ComputedStyle::default();
    style.border_image_slice.fill = true;
    assert!(apply_initial_value(&mut style, "border-image-slice"));
    assert!(!style.border_image_slice.fill);
}

// ── border-image-width ──

#[test]
fn test_apply_property_border_image_width_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-width", "auto"));
    assert_eq!(style.border_image_width.top, BorderImageWidthComputedComponent::Auto);
}

#[test]
fn test_apply_property_border_image_width_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-width", "3"));
    assert_eq!(
        style.border_image_width.top,
        BorderImageWidthComputedComponent::Number(3.0)
    );
}

#[test]
fn test_apply_property_border_image_width_px() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-width", "10px"));
    assert_eq!(
        style.border_image_width.top,
        BorderImageWidthComputedComponent::Length(10.0)
    );
}

#[test]
fn test_apply_property_border_image_width_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-image-width", "invalid"));
}

#[test]
fn test_border_image_width_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("border-image-width"));
}

#[test]
fn test_border_image_width_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"border-image-width"));
}

#[test]
fn test_border_image_width_initial_value() {
    assert!(PropertyRegistry::initial_value("border-image-width").is_some());
    let mut style = ComputedStyle::default();
    style.border_image_width.top = BorderImageWidthComputedComponent::Auto;
    assert!(apply_initial_value(&mut style, "border-image-width"));
    assert_eq!(
        style.border_image_width.top,
        BorderImageWidthComputedComponent::Number(1.0)
    );
}

// ── border-image-repeat ──

#[test]
fn test_apply_property_border_image_repeat_stretch() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-repeat", "stretch"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Stretch
    );
}

#[test]
fn test_apply_property_border_image_repeat_repeat() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-repeat", "repeat"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Repeat
    );
}

#[test]
fn test_apply_property_border_image_repeat_round() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-repeat", "round"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Round
    );
}

#[test]
fn test_apply_property_border_image_repeat_space() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-repeat", "space"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Space
    );
}

#[test]
fn test_apply_property_border_image_repeat_two_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-repeat", "repeat round"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Repeat
    );
    assert_eq!(style.border_image_repeat.vertical, BorderImageRepeatComputedMode::Round);
}

#[test]
fn test_apply_property_border_image_repeat_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-image-repeat", "invalid"));
}

#[test]
fn test_border_image_repeat_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("border-image-repeat"));
}

#[test]
fn test_border_image_repeat_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"border-image-repeat"));
}

#[test]
fn test_border_image_repeat_initial_value() {
    assert!(PropertyRegistry::initial_value("border-image-repeat").is_some());
    let mut style = ComputedStyle::default();
    style.border_image_repeat.horizontal = BorderImageRepeatComputedMode::Repeat;
    assert!(apply_initial_value(&mut style, "border-image-repeat"));
    assert_eq!(
        style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Stretch
    );
}

// ── border-image-outset ──

#[test]
fn test_apply_property_border_image_outset_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-outset", "2"));
    assert_eq!(
        style.border_image_outset.top,
        BorderImageOutsetComputedComponent::Number(2.0)
    );
}

#[test]
fn test_apply_property_border_image_outset_px() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-outset", "10px"));
    assert_eq!(
        style.border_image_outset.top,
        BorderImageOutsetComputedComponent::Length(10.0)
    );
}

#[test]
fn test_apply_property_border_image_outset_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-image-outset", "invalid"));
}

#[test]
fn test_border_image_outset_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("border-image-outset"));
}

#[test]
fn test_border_image_outset_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"border-image-outset"));
}

#[test]
fn test_border_image_outset_initial_value() {
    assert!(PropertyRegistry::initial_value("border-image-outset").is_some());
    let mut style = ComputedStyle::default();
    style.border_image_outset.top = BorderImageOutsetComputedComponent::Number(10.0);
    assert!(apply_initial_value(&mut style, "border-image-outset"));
    assert_eq!(
        style.border_image_outset.top,
        BorderImageOutsetComputedComponent::Number(0.0)
    );
}

// ── text-shadow ──

#[test]
fn test_apply_text_shadow_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-shadow", "none"));
    assert!(style.text_shadow.is_empty(), "none → 空阴影列表");
}

#[test]
fn test_apply_text_shadow_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-shadow", "2px 3px 4px red"));
    assert_eq!(style.text_shadow[0].offset_x, 2.0);
    assert_eq!(style.text_shadow[0].offset_y, 3.0);
    assert_eq!(style.text_shadow[0].blur_radius, 4.0);
    assert_eq!(
        style.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255)
    );
}

#[test]
fn test_text_shadow_is_inherited() {
    assert!(PropertyRegistry::is_inherited("text-shadow"));
}

/// R2305：多 text-shadow 列表（CSS Text Decoration §3：`none | <shadow>#`）应用到 ComputedStyle。
#[test]
fn test_apply_text_shadow_multiple_list() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "text-shadow",
        "1px 2px red, 3px 4px blue"
    ));
    assert_eq!(style.text_shadow.len(), 2, "应解析为 2 个阴影");

    let first = &style.text_shadow[0];
    assert_eq!(first.offset_x, 1.0);
    assert_eq!(first.offset_y, 2.0);
    assert_eq!(first.color, zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255));

    let second = &style.text_shadow[1];
    assert_eq!(second.offset_x, 3.0);
    assert_eq!(second.offset_y, 4.0);
    assert_eq!(second.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_text_shadow_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"text-shadow"));
}

#[test]
fn test_text_shadow_initial_value() {
    assert!(PropertyRegistry::initial_value("text-shadow").is_some());
}

#[test]
fn test_text_shadow_apply_initial() {
    let mut style = ComputedStyle::default();
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 10.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
    }];
    assert!(apply_initial_value(&mut style, "text-shadow"));
    assert!(style.text_shadow.is_empty(), "initial → 空阴影列表");
}

// ── box-shadow ──

#[test]
fn test_apply_box_shadow_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "box-shadow", "none"));
    assert!(style.box_shadow.is_empty(), "none → 空阴影列表");
}

#[test]
fn test_apply_box_shadow_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "box-shadow",
        "10px 20px 30px 5px blue"
    ));
    let s = &style.box_shadow[0];
    assert_eq!(s.offset_x, 10.0);
    assert_eq!(s.offset_y, 20.0);
    assert_eq!(s.blur_radius, 30.0);
    assert_eq!(s.spread_radius, 5.0);
    assert_eq!(s.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255));
    assert!(!s.inset);
}

#[test]
fn test_apply_box_shadow_inset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "box-shadow", "inset 5px 10px"));
    let s = &style.box_shadow[0];
    assert!(s.inset);
    assert_eq!(s.offset_x, 5.0);
    assert_eq!(s.offset_y, 10.0);
}

#[test]
fn test_box_shadow_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("box-shadow"));
}

/// R2304：多 box-shadow 列表（CSS Backgrounds §7.2：<shadow>#）应用到 ComputedStyle。
#[test]
fn test_apply_box_shadow_multiple_list() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "box-shadow",
        "1px 2px red, inset 3px 4px blue"
    ));
    assert_eq!(style.box_shadow.len(), 2, "应解析为 2 个阴影");

    let first = &style.box_shadow[0];
    assert_eq!(first.offset_x, 1.0);
    assert_eq!(first.offset_y, 2.0);
    assert!(!first.inset);
    assert_eq!(first.color, zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255));

    let second = &style.box_shadow[1];
    assert_eq!(second.offset_x, 3.0);
    assert_eq!(second.offset_y, 4.0);
    assert!(second.inset, "第二个应为 inset");
    assert_eq!(second.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_box_shadow_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"box-shadow"));
}

#[test]
fn test_box_shadow_initial_value() {
    assert!(PropertyRegistry::initial_value("box-shadow").is_some());
}

#[test]
fn test_box_shadow_apply_initial() {
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 99.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    }];
    assert!(apply_initial_value(&mut style, "box-shadow"));
    assert!(style.box_shadow.is_empty(), "initial → 空阴影列表");
}

#[test]
fn test_box_shadow_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "box-shadow", "invalid"));
}

#[test]
fn test_text_shadow_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "text-shadow", "invalid"));
}

// ── 边界测试：text-shadow 通过 DOM 树继承 ──

/// 验证 text-shadow 作为可继承属性，通过 inherit_property 从父元素传递到子元素。
/// 父元素设置 text-shadow: 3px 5px 2px blue，子元素应完整继承该值。
#[test]
fn test_text_shadow_inheritance_through_dom_tree() {
    // 构造父元素样式：设置 text-shadow
    let mut parent = ComputedStyle::default();
    assert!(apply_property_value(&mut parent, "text-shadow", "3px 5px 2px blue"));
    assert_eq!(parent.text_shadow[0].offset_x, 3.0);
    assert_eq!(parent.text_shadow[0].offset_y, 5.0);
    assert_eq!(parent.text_shadow[0].blur_radius, 2.0);
    assert_eq!(
        parent.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255)
    );

    // 构造子元素样式：从父元素继承 text-shadow
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "text-shadow"));

    // 子元素应获得与父元素完全相同的 text-shadow 值
    assert_eq!(child.text_shadow, parent.text_shadow);
}

// ── 边界测试：box-shadow inset 与 normal 正确区分 ──

/// 验证 box-shadow 的 inset 标志与普通（outset）阴影正确区分。
/// 同一偏移量下，inset 版本的 inset 字段应为 true，普通版本应为 false。
#[test]
fn test_box_shadow_inset_vs_normal_applied_correctly() {
    // 普通 box-shadow（无 inset）
    let mut normal_style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut normal_style,
        "box-shadow",
        "4px 8px 6px 2px green"
    ));
    let n = &normal_style.box_shadow[0];
    assert!(!n.inset, "普通 box-shadow 的 inset 应为 false");
    assert_eq!(n.offset_x, 4.0);
    assert_eq!(n.offset_y, 8.0);
    assert_eq!(n.blur_radius, 6.0);
    assert_eq!(n.spread_radius, 2.0);
    assert_eq!(n.color, zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255));

    // inset box-shadow
    let mut inset_style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut inset_style,
        "box-shadow",
        "inset 4px 8px 6px 2px green"
    ));
    let i = &inset_style.box_shadow[0];
    assert!(i.inset, "inset box-shadow 的 inset 应为 true");
    assert_eq!(i.offset_x, 4.0);
    assert_eq!(i.offset_y, 8.0);
    assert_eq!(i.blur_radius, 6.0);
    assert_eq!(i.spread_radius, 2.0);
    assert_eq!(i.color, zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255));
}

// ── 边界测试：outline 简写属性通过 expand_shorthands 展开 ──

/// 验证 outline 简写属性通过 expand_shorthands 正确展开为
/// outline-width、outline-style、outline-color 三个长属性，
/// 且 important 标志和特异性正确保留。
#[test]
fn test_outline_shorthand_expansion_via_expand_shorthands() {
    use crate::shorthand::expand_shorthands;

    // outline: 3px dashed red, important=true, specificity=(0,1,0)
    let decls: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("outline".to_string(), "3px dashed red".to_string(), true, (0, 1, 0))];
    let expanded = expand_shorthands(&decls);

    // 展开后应得到 3 个长属性声明
    assert_eq!(expanded.len(), 3);

    // 验证各长属性名称和值
    let props: Vec<(&str, &str)> = expanded.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert!(props.contains(&("outline-width", "3px")));
    assert!(props.contains(&("outline-style", "dashed")));
    assert!(props.contains(&("outline-color", "red")));

    // 验证 important 和特异性在展开中保留
    for (_, _, imp, spec) in &expanded {
        assert!(imp, "important 标志应被保留");
        assert_eq!(*spec, (0, 1, 0), "特异性应被保留");
    }
}

// ── 边界测试：border-image-slice 带 fill 关键字通过 apply_property_value ──

/// 验证 border-image-slice 的 fill 关键字在 apply_property_value 中正确解析，
/// fill=true 时四个分量值也正确设置。
#[test]
fn test_border_image_slice_with_fill_keyword() {
    let mut style = ComputedStyle::default();

    // 默认 fill 应为 false
    assert!(!style.border_image_slice.fill);

    // 设置 border-image-slice: fill 10 20% 30 40%
    assert!(apply_property_value(
        &mut style,
        "border-image-slice",
        "fill 10 20% 30 40%"
    ));

    // fill 应为 true
    assert!(style.border_image_slice.fill, "fill 关键字应使 fill=true");

    // 验证四个分量的值
    assert_eq!(
        style.border_image_slice.top,
        BorderImageSliceComputedComponent::Number(10.0)
    );
    assert_eq!(
        style.border_image_slice.right,
        BorderImageSliceComputedComponent::Percent(20.0)
    );
    assert_eq!(
        style.border_image_slice.bottom,
        BorderImageSliceComputedComponent::Number(30.0)
    );
    assert_eq!(
        style.border_image_slice.left,
        BorderImageSliceComputedComponent::Percent(40.0)
    );
}

// ── 边界测试：text_shadow 和 box_shadow 计算样式的默认值（无阴影） ──

/// 验证 ComputedStyle 默认构造时，text_shadow 和 box_shadow 均表示"无阴影"状态：
/// text-shadow 全偏移/半径为 0；box-shadow 为空阴影列表（R2304：none = 空 Vec）。
#[test]
fn test_computed_style_default_no_shadow() {
    let style = ComputedStyle::default();

    // text-shadow 默认值：空阴影列表 = 无阴影（R2305：none = 空 Vec）
    assert!(style.text_shadow.is_empty(), "默认 text-shadow 应为空列表");

    // box-shadow 默认值：空阴影列表 = 无阴影
    assert!(style.box_shadow.is_empty(), "默认 box-shadow 应为空列表");
}

// ── list-style-image ──

#[test]
fn test_apply_list_style_image_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "list-style-image", "none"));
    assert_eq!(style.list_style_image, ListStyleImageComputedValue::None);
}

#[test]
fn test_apply_list_style_image_url() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "list-style-image", "url(star.png)"));
    assert_eq!(
        style.list_style_image,
        ListStyleImageComputedValue::Url("star.png".to_string())
    );
}

#[test]
fn test_apply_list_style_image_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "list-style-image", "invalid"));
}

#[test]
fn test_list_style_image_is_inherited() {
    assert!(PropertyRegistry::is_inherited("list-style-image"));
}

#[test]
fn test_list_style_image_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"list-style-image"));
}

#[test]
fn test_list_style_image_initial_value() {
    assert!(PropertyRegistry::initial_value("list-style-image").is_some());
    let mut style = ComputedStyle::default();
    style.list_style_image = ListStyleImageComputedValue::Url("test.png".to_string());
    assert!(apply_initial_value(&mut style, "list-style-image"));
    assert_eq!(style.list_style_image, ListStyleImageComputedValue::None);
}

// ── column-gap ──

#[test]
fn test_apply_column_gap_px() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "column-gap", "20px"));
    assert_eq!(style.column_gap, LengthValue::Px(20.0));
}

#[test]
fn test_apply_column_gap_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "column-gap", "invalid"));
}

#[test]
fn test_column_gap_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("column-gap"));
}

#[test]
fn test_column_gap_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"column-gap"));
}

#[test]
fn test_transform_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"transform"));
}

#[test]
fn test_grid_template_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"grid-template-columns"));
    assert!(props.contains(&"grid-template-rows"));
    assert!(props.contains(&"grid-template-areas"));
    assert!(props.contains(&"grid-auto-flow"));
    assert!(props.contains(&"row-gap"));
}

// ═══════════════════════════════════════════════════════════════════
// 边界测试 — list-style-image 继承 / column-gap 百分比 /
// transform 多函数 / grid-auto-flow dense / row-gap em
// ═══════════════════════════════════════════════════════════════════

/// 验证 list-style-image 作为可继承属性，通过 inherit_property 从父元素传递到子元素。
/// 父元素设置 list-style-image: url(bullet.png)，子元素应完整继承该 URL 值。
#[test]
fn test_list_style_image_inheritance_through_inherit_property() {
    // 构造父元素样式：设置 list-style-image
    let mut parent = ComputedStyle::default();
    assert!(apply_property_value(&mut parent, "list-style-image", "url(bullet.png)"));
    assert_eq!(
        parent.list_style_image,
        ListStyleImageComputedValue::Url("bullet.png".to_string())
    );

    // 构造子元素样式：从父元素继承 list-style-image
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "list-style-image"));

    // 子元素应获得与父元素完全相同的 list-style-image 值
    assert_eq!(child.list_style_image, parent.list_style_image);
}

/// 验证 column-gap 接受百分比值。
/// 百分比在布局阶段相对于容器宽度计算，此处验证解析和存储正确性。
#[test]
fn test_column_gap_with_percentage_value() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "column-gap", "25%"));
    assert_eq!(style.column_gap, LengthValue::Percentage(25.0));
}

/// 验证 transform 属性支持多个变换函数组合。
/// "translate(10px) rotate(45deg)" 应解析为包含两个 TransformFunction 的列表。
#[test]
fn test_transform_with_multiple_functions() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "transform",
        "translate(10px) rotate(45deg)"
    ));
    match &style.transform {
        zero_css_parser::values::TransformValue::List(fns) => {
            assert_eq!(fns.len(), 2, "应包含两个变换函数");
            // 第一个函数：translate(10px) → Translate(10.0, 0.0)
            assert_eq!(fns[0], zero_css_parser::values::TransformFunction::Translate(10.0, 0.0));
            // 第二个函数：rotate(45deg) → Rotate(45.0)
            assert_eq!(fns[1], zero_css_parser::values::TransformFunction::Rotate(45.0));
        }
        other => panic!("transform 应为 List 变体，实际为: {other:?}"),
    }
}

/// 验证 grid-auto-flow 仅使用 "dense" 关键字时，
/// 解析为 RowDense（等效于 "row dense"）。
#[test]
fn test_grid_auto_flow_dense_keyword() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-auto-flow", "dense"));
    assert_eq!(style.grid_auto_flow, GridAutoFlowValue::RowDense);
}

/// 验证 row-gap 接受 em 单位值。
/// em 值在计算样式阶段相对于当前 font-size 解析，此处验证原始值正确存储。
#[test]
fn test_row_gap_with_em_value() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "row-gap", "1.5em"));
    assert_eq!(style.row_gap, LengthValue::Em(1.5));
}

// ═══════════════════════════════════════════════════════════════════
// justify-items / justify-self / align-content / empty-cells / border-spacing
// ═══════════════════════════════════════════════════════════════════

/// 验证 justify-items 的 apply_property_value 正确解析所有关键字值。
#[test]
fn test_apply_justify_items_keywords() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "justify-items", "center"));
    assert_eq!(style.justify_items, JustifyItemsValue::Center);
    assert!(apply_property_value(&mut style, "justify-items", "start"));
    assert_eq!(style.justify_items, JustifyItemsValue::Start);
    assert!(apply_property_value(&mut style, "justify-items", "normal"));
    assert_eq!(style.justify_items, JustifyItemsValue::Normal);
    assert!(apply_property_value(&mut style, "justify-items", "stretch"));
    assert_eq!(style.justify_items, JustifyItemsValue::Stretch);
    // R2382：CSS Box Align 3 left/right（物理位置关键字，Chrome 支持）。修复前 None → 被丢。
    assert!(apply_property_value(&mut style, "justify-items", "left"));
    assert_eq!(style.justify_items, JustifyItemsValue::Left);
    assert!(apply_property_value(&mut style, "justify-items", "right"));
    assert_eq!(style.justify_items, JustifyItemsValue::Right);
}

/// 验证 justify-items 对无效值返回 false。
#[test]
fn test_apply_justify_items_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "justify-items", "invalid"));
}

/// 验证 justify-self 的 apply_property_value 正确解析所有关键字值。
#[test]
fn test_apply_justify_self_keywords() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "justify-self", "auto"));
    assert_eq!(style.justify_self, JustifySelfValue::Auto);
    assert!(apply_property_value(&mut style, "justify-self", "end"));
    assert_eq!(style.justify_self, JustifySelfValue::End);
    assert!(apply_property_value(&mut style, "justify-self", "baseline"));
    assert_eq!(style.justify_self, JustifySelfValue::Baseline);
    // R2382：left/right（CSS Box Align 3）。
    assert!(apply_property_value(&mut style, "justify-self", "left"));
    assert_eq!(style.justify_self, JustifySelfValue::Left);
    assert!(apply_property_value(&mut style, "justify-self", "right"));
    assert_eq!(style.justify_self, JustifySelfValue::Right);
}

/// 验证 align-content 的 apply_property_value 正确解析所有关键字值。
#[test]
fn test_apply_align_content_keywords() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "align-content", "space-between"));
    assert_eq!(style.align_content, AlignContentValue::SpaceBetween);
    assert!(apply_property_value(&mut style, "align-content", "space-around"));
    assert_eq!(style.align_content, AlignContentValue::SpaceAround);
    assert!(apply_property_value(&mut style, "align-content", "space-evenly"));
    assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);
    assert!(apply_property_value(&mut style, "align-content", "center"));
    assert_eq!(style.align_content, AlignContentValue::Center);
    // R1412：flex-start/flex-end 此前未解析（fall through → 默认 Normal），现映射
    // Start/End（horizontal-tb 下 flex-start=end of block axis 等价 start/end）。
    assert!(apply_property_value(&mut style, "align-content", "flex-start"));
    assert_eq!(style.align_content, AlignContentValue::Start);
    assert!(apply_property_value(&mut style, "align-content", "flex-end"));
    assert_eq!(style.align_content, AlignContentValue::End);
}

/// 验证 align-content 对无效值返回 false。
#[test]
fn test_apply_align_content_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "align-content", "bogus-value"));
}

/// 验证 empty-cells 的 apply_property_value 正确解析 show/hide。
#[test]
fn test_apply_empty_cells() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "empty-cells", "hide"));
    assert_eq!(style.empty_cells, EmptyCellsComputedValue::Hide);
    assert!(apply_property_value(&mut style, "empty-cells", "show"));
    assert_eq!(style.empty_cells, EmptyCellsComputedValue::Show);
}

/// 验证 empty-cells 对无效值返回 false。
#[test]
fn test_apply_empty_cells_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "empty-cells", "visible"));
}

/// 验证 border-spacing 的 apply_property_value 正确解析单值和双值。
#[test]
fn test_apply_border_spacing() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-spacing", "5px"));
    assert_eq!(style.border_spacing.horizontal, 5.0);
    assert_eq!(style.border_spacing.vertical, 5.0);
    assert!(apply_property_value(&mut style, "border-spacing", "2px 4px"));
    assert_eq!(style.border_spacing.horizontal, 2.0);
    assert_eq!(style.border_spacing.vertical, 4.0);
}

/// 验证 border-spacing 对无效值返回 false。
#[test]
fn test_apply_border_spacing_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-spacing", "invalid"));
}

/// 验证 empty-cells 和 border-spacing 是继承属性。
#[test]
fn test_inheritance_empty_cells_and_border_spacing() {
    assert!(PropertyRegistry::is_inherited("empty-cells"));
    assert!(PropertyRegistry::is_inherited("border-spacing"));
    // justify-items / justify-self / align-content 不继承
    assert!(!PropertyRegistry::is_inherited("justify-items"));
    assert!(!PropertyRegistry::is_inherited("justify-self"));
    assert!(!PropertyRegistry::is_inherited("align-content"));
}

/// 验证 5 个属性都在 known_properties 中注册。
#[test]
fn test_known_properties_new_five() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"justify-items"));
    assert!(props.contains(&"justify-self"));
    assert!(props.contains(&"align-content"));
    assert!(props.contains(&"empty-cells"));
    assert!(props.contains(&"border-spacing"));
}

/// 验证 5 个属性的 initial_value 均可获取。
#[test]
fn test_initial_value_new_five() {
    assert!(PropertyRegistry::initial_value("justify-items").is_some());
    assert!(PropertyRegistry::initial_value("justify-self").is_some());
    assert!(PropertyRegistry::initial_value("align-content").is_some());
    assert!(PropertyRegistry::initial_value("empty-cells").is_some());
    assert!(PropertyRegistry::initial_value("border-spacing").is_some());
}

/// 验证 apply_initial_value 对 5 个新属性能正确重置为默认值。
#[test]
fn test_apply_initial_value_new_five() {
    let mut style = ComputedStyle::default();
    // 先设置非默认值
    apply_property_value(&mut style, "justify-items", "center");
    apply_property_value(&mut style, "justify-self", "end");
    apply_property_value(&mut style, "align-content", "space-between");
    apply_property_value(&mut style, "empty-cells", "hide");
    apply_property_value(&mut style, "border-spacing", "10px");

    // 重置
    assert!(apply_initial_value(&mut style, "justify-items"));
    assert_eq!(style.justify_items, JustifyItemsValue::Normal);
    assert!(apply_initial_value(&mut style, "justify-self"));
    assert_eq!(style.justify_self, JustifySelfValue::Auto);
    assert!(apply_initial_value(&mut style, "align-content"));
    assert_eq!(style.align_content, AlignContentValue::Normal);
    assert!(apply_initial_value(&mut style, "empty-cells"));
    assert_eq!(style.empty_cells, EmptyCellsComputedValue::Show);
    assert!(apply_initial_value(&mut style, "border-spacing"));
    assert_eq!(style.border_spacing.horizontal, 0.0);
    assert_eq!(style.border_spacing.vertical, 0.0);
}

/// 验证 empty-cells 和 border-spacing 的继承正确工作。
#[test]
fn test_inherit_property_empty_cells_and_border_spacing() {
    let mut parent = ComputedStyle::default();
    apply_property_value(&mut parent, "empty-cells", "hide");
    apply_property_value(&mut parent, "border-spacing", "3px 7px");

    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "empty-cells"));
    assert_eq!(child.empty_cells, EmptyCellsComputedValue::Hide);
    assert!(inherit_property(&parent, &mut child, "border-spacing"));
    assert_eq!(child.border_spacing.horizontal, 3.0);
    assert_eq!(child.border_spacing.vertical, 7.0);
}

// ═══════════════════════════════════════════════════════════════════
// 边界条件测试 — justify-items 全值 / align-content space-between /
//   empty-cells 继承 / border-spacing 继承 / gap 简写展开
// ═══════════════════════════════════════════════════════════════════

/// 测试 justify-items 所有枚举值通过 apply_property_value 正确应用到 ComputedStyle。
#[test]
fn test_justify_items_all_values_via_apply() {
    let mut style = ComputedStyle::default();

    // 默认值为 Normal
    assert_eq!(style.justify_items, JustifyItemsValue::Normal);

    // 逐一验证所有 7 个枚举值
    assert!(apply_property_value(&mut style, "justify-items", "auto"));
    assert_eq!(style.justify_items, JustifyItemsValue::Auto);

    assert!(apply_property_value(&mut style, "justify-items", "normal"));
    assert_eq!(style.justify_items, JustifyItemsValue::Normal);

    assert!(apply_property_value(&mut style, "justify-items", "start"));
    assert_eq!(style.justify_items, JustifyItemsValue::Start);

    assert!(apply_property_value(&mut style, "justify-items", "end"));
    assert_eq!(style.justify_items, JustifyItemsValue::End);

    assert!(apply_property_value(&mut style, "justify-items", "center"));
    assert_eq!(style.justify_items, JustifyItemsValue::Center);

    assert!(apply_property_value(&mut style, "justify-items", "stretch"));
    assert_eq!(style.justify_items, JustifyItemsValue::Stretch);

    assert!(apply_property_value(&mut style, "justify-items", "baseline"));
    assert_eq!(style.justify_items, JustifyItemsValue::Baseline);

    // 无效值应返回 false 且不改变当前值
    assert!(!apply_property_value(&mut style, "justify-items", "invalid"));
    assert_eq!(style.justify_items, JustifyItemsValue::Baseline);
}

/// 测试 align-content: space-between 通过 apply_property_value 正确应用。
#[test]
fn test_align_content_space_between() {
    let mut style = ComputedStyle::default();

    // 默认值为 Normal
    assert_eq!(style.align_content, AlignContentValue::Normal);

    // space-between 是 Box Alignment 规范中的关键值
    assert!(apply_property_value(&mut style, "align-content", "space-between"));
    assert_eq!(style.align_content, AlignContentValue::SpaceBetween);

    // 同系列值也应工作
    assert!(apply_property_value(&mut style, "align-content", "space-around"));
    assert_eq!(style.align_content, AlignContentValue::SpaceAround);

    assert!(apply_property_value(&mut style, "align-content", "space-evenly"));
    assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "align-content", "space-invalid"));
    assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);
}

/// 测试 empty-cells 通过 inherit_property 正确从父元素继承到子元素。
#[test]
fn test_empty_cells_inheritance_via_inherit_property() {
    // empty-cells 是继承属性，父元素设置 hide 后子元素应继承
    let mut parent = ComputedStyle::default();
    parent.empty_cells = EmptyCellsComputedValue::Hide;

    let mut child = ComputedStyle::default();
    // 子元素默认为 Show
    assert_eq!(child.empty_cells, EmptyCellsComputedValue::Show);

    // 继承成功
    assert!(inherit_property(&parent, &mut child, "empty-cells"));
    assert_eq!(child.empty_cells, EmptyCellsComputedValue::Hide);

    // 子元素显式设置后覆盖继承值
    assert!(apply_property_value(&mut child, "empty-cells", "show"));
    assert_eq!(child.empty_cells, EmptyCellsComputedValue::Show);

    // 反向：父元素 Show → 子元素继承 Show
    let parent2 = ComputedStyle::default();
    let mut child2 = ComputedStyle::default();
    child2.empty_cells = EmptyCellsComputedValue::Hide;
    assert!(inherit_property(&parent2, &mut child2, "empty-cells"));
    assert_eq!(child2.empty_cells, EmptyCellsComputedValue::Show);
}

/// 测试 border-spacing 通过 inherit_property 正确从父元素继承到子元素，
/// 包括水平/垂直分量独立验证。
#[test]
fn test_border_spacing_inheritance_via_inherit_property() {
    // border-spacing 是继承属性
    let mut parent = ComputedStyle::default();
    parent.border_spacing.horizontal = 12.0;
    parent.border_spacing.vertical = 24.0;

    let mut child = ComputedStyle::default();
    // 子元素默认为 0 0
    assert_eq!(child.border_spacing.horizontal, 0.0);
    assert_eq!(child.border_spacing.vertical, 0.0);

    // 继承成功，水平/垂直分量分别复制
    assert!(inherit_property(&parent, &mut child, "border-spacing"));
    assert_eq!(child.border_spacing.horizontal, 12.0);
    assert_eq!(child.border_spacing.vertical, 24.0);

    // 子元素显式设置后覆盖继承值（只设水平，垂直仍由简写决定）
    assert!(apply_property_value(&mut child, "border-spacing", "5px"));
    assert_eq!(child.border_spacing.horizontal, 5.0);
    assert_eq!(child.border_spacing.vertical, 5.0);

    // 两值形式继承：水平和垂直不同
    let mut parent3 = ComputedStyle::default();
    parent3.border_spacing.horizontal = 8.0;
    parent3.border_spacing.vertical = 16.0;

    let mut child3 = ComputedStyle::default();
    assert!(inherit_property(&parent3, &mut child3, "border-spacing"));
    assert_eq!(child3.border_spacing.horizontal, 8.0);
    assert_eq!(child3.border_spacing.vertical, 16.0);
}

/// 测试 gap 简写属性通过 expand_shorthands 正确展开为
/// gap、row-gap、column-gap 三个长属性，
/// 覆盖单值和双值两种形式。
#[test]
fn test_gap_shorthand_expansion_via_expand_shorthands() {
    use crate::shorthand::expand_shorthands;

    // ── 单值形式：gap: 10px → row-gap: 10px, column-gap: 10px ──
    let decls: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("gap".to_string(), "10px".to_string(), false, (0, 0, 1))];
    let expanded = expand_shorthands(&decls);

    // 展开后应得到 3 个声明：gap + row-gap + column-gap
    assert_eq!(expanded.len(), 3);

    let props: Vec<(&str, &str)> = expanded.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert!(props.contains(&("gap", "10px")));
    assert!(props.contains(&("row-gap", "10px")));
    assert!(props.contains(&("column-gap", "10px")));

    // important 和特异性应保留
    for (_, _, imp, spec) in &expanded {
        assert!(!imp);
        assert_eq!(*spec, (0, 0, 1));
    }

    // ── 双值形式：gap: 10px 20px → row-gap: 10px, column-gap: 20px ──
    let decls2: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("gap".to_string(), "10px 20px".to_string(), true, (0, 1, 0))];
    let expanded2 = expand_shorthands(&decls2);

    assert_eq!(expanded2.len(), 3);

    let props2: Vec<(&str, &str)> = expanded2.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert!(props2.contains(&("gap", "10px")));
    assert!(props2.contains(&("row-gap", "10px")));
    assert!(props2.contains(&("column-gap", "20px")));

    // important 和特异性保留
    for (_, _, imp, spec) in &expanded2 {
        assert!(imp, "important 标志应被保留");
        assert_eq!(*spec, (0, 1, 0), "特异性应被保留");
    }
}

// ═══════════════════════════════════════════════════════════════════
// counter-set 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_apply_counter_set_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "counter-set", "none"));
}

#[test]
fn test_apply_counter_set_value() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "counter-set", "mycounter 3"));
}

#[test]
fn test_counter_set_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("counter-set"));
}

#[test]
fn test_counter_set_in_known_properties() {
    assert!(PropertyRegistry::known_properties().contains(&"counter-set"));
}

#[test]
fn test_counter_set_initial_value() {
    assert!(PropertyRegistry::initial_value("counter-set").is_some());
}

// ── 边界测试：box-shadow / text-shadow / background-image ──

/// 测试 box-shadow 计算值默认值（空阴影列表 = 无阴影）。
#[test]
fn test_edge_box_shadow_default_all_zero() {
    let style = ComputedStyle::default();
    assert!(style.box_shadow.is_empty(), "默认 box-shadow 应为空列表");
}

/// 测试 box-shadow 解析 "4px 4px 8px 0px rgba(0,0,0,0.5)"。
#[test]
fn test_edge_box_shadow_rgba_parse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "box-shadow",
        "4px 4px 8px 0px rgba(0,0,0,0.5)"
    ));
    let s = &style.box_shadow[0];
    assert_eq!(s.offset_x, 4.0);
    assert_eq!(s.offset_y, 4.0);
    assert_eq!(s.blur_radius, 8.0);
    assert_eq!(s.spread_radius, 0.0);
    // rgba(0,0,0,0.5) -> alpha=128 (0.5*255 rounded)
    assert_eq!(s.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 128));
    assert!(!s.inset);
}

/// 测试 text-shadow 计算值默认值（空阴影列表 = 无阴影）。
#[test]
fn test_edge_text_shadow_default() {
    let style = ComputedStyle::default();
    assert!(style.text_shadow.is_empty(), "默认 text-shadow 应为空列表");
}

/// 测试 text-shadow 解析 "2px 2px 4px red"。
#[test]
fn test_edge_text_shadow_red_parse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-shadow", "2px 2px 4px red"));
    assert_eq!(style.text_shadow[0].offset_x, 2.0);
    assert_eq!(style.text_shadow[0].offset_y, 2.0);
    assert_eq!(style.text_shadow[0].blur_radius, 4.0);
    assert_eq!(
        style.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255)
    );
}

/// 测试 background-image 计算值默认值为 None。
#[test]
fn test_edge_background_image_default_none() {
    let style = ComputedStyle::default();
    assert!(style.background_image.is_empty());
}

/// 测试 background-image 解析 "url(hero.png)"。
#[test]
fn test_edge_background_image_url_hero() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "url(hero.png)"));
    assert_eq!(
        style.background_image,
        vec![BackgroundImageComputedValue::Url("hero.png".to_string())]
    );
}

/// 测试 box-shadow 负值解析。
#[test]
fn test_edge_box_shadow_negative_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "box-shadow",
        "-3px -5px -2px -1px red"
    ));
    let s = &style.box_shadow[0];
    assert_eq!(s.offset_x, -3.0);
    assert_eq!(s.offset_y, -5.0);
    assert_eq!(s.blur_radius, -2.0);
    assert_eq!(s.spread_radius, -1.0);
    assert_eq!(s.color, zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255));
    assert!(!s.inset);
}

/// 测试 text-shadow 继承到子元素。
#[test]
fn test_edge_text_shadow_inherit_to_child() {
    let mut parent = ComputedStyle::default();
    assert!(apply_property_value(&mut parent, "text-shadow", "2px 2px 4px red"));
    let mut child = ComputedStyle::default();
    // text-shadow 是继承属性，inherit_property 应成功
    assert!(inherit_property(&parent, &mut child, "text-shadow"));
    assert_eq!(child.text_shadow[0].offset_x, 2.0);
    assert_eq!(child.text_shadow[0].offset_y, 2.0);
    assert_eq!(child.text_shadow[0].blur_radius, 4.0);
    assert_eq!(
        child.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255)
    );
}

// ── background-image 渐变边界测试 ──

/// 测试 background-image 渐变值解析。
#[test]
fn test_edge_background_image_gradient() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "background-image",
        "linear-gradient(red, blue)"
    ));
    assert!(matches!(
        &style.background_image[..],
        [BackgroundImageComputedValue::Gradient(..)]
    ));
}

/// 测试 background-image radial-gradient 解析。
#[test]
fn test_edge_background_image_radial_gradient() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "background-image",
        "radial-gradient(circle, red, blue)"
    ));
    match &style.background_image[0] {
        BackgroundImageComputedValue::Gradient(g) => {
            assert!(matches!(g, zero_css_parser::values::GradientValue::Radial(..)));
        }
        other => panic!("expected Gradient(Radial(..)), got {:?}", other),
    }
}

/// 测试 background-image 默认值仍为 None。
#[test]
fn test_edge_background_image_default_still_none() {
    let style = ComputedStyle::default();
    assert!(style.background_image.is_empty());
}

/// 测试 background-image 初始值重置。
#[test]
fn test_edge_background_image_initial_reset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "background-image",
        "linear-gradient(red, blue)"
    ));
    assert!(matches!(
        &style.background_image[..],
        [BackgroundImageComputedValue::Gradient(..)]
    ));
    assert!(apply_initial_value(&mut style, "background-image"));
    assert!(style.background_image.is_empty());
}

/// 测试 background 简写展开渐变。
#[test]
fn test_edge_background_shorthand_gradient() {
    use crate::shorthand::expand_shorthands;

    let decls: Vec<(String, String, bool, (u32, u32, u32))> = vec![(
        "background".to_string(),
        "linear-gradient(red, blue)".to_string(),
        false,
        (0, 0, 1),
    )];
    let expanded = expand_shorthands(&decls);
    let props: Vec<(&str, &str)> = expanded.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
    assert!(props.contains(&("background-image", "linear-gradient(red, blue)")));
}

/// 测试 background-image conic-gradient 解析。
#[test]
fn test_edge_background_image_conic_gradient() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "background-image",
        "conic-gradient(from 45deg, red, blue)"
    ));
    match &style.background_image[0] {
        BackgroundImageComputedValue::Gradient(g) => {
            assert!(matches!(g, zero_css_parser::values::GradientValue::Conic(..)));
        }
        other => panic!("expected Gradient(Conic(..)), got {:?}", other),
    }
}

// ── 新增边界测试 ──

/// 测试 background-color: transparent 管线。
#[test]
fn test_background_color_transparent_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-color", "transparent"));
    assert_eq!(style.background_color, ColorValue::Transparent);
}

/// 测试 color: currentColor 管线。
#[test]
fn test_color_current_color_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "color", "currentColor"));
    assert_eq!(style.color, ColorValue::CurrentColor);
}

/// 测试 display: inline-block 管线。
#[test]
fn test_display_inline_block_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "display", "inline-block"));
    assert_eq!(style.display, DisplayValue::InlineBlock);
}

/// 测试 display: flex 管线。
#[test]
fn test_display_flex_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "display", "flex"));
    assert_eq!(style.display, DisplayValue::Flex);
}

/// 测试 position: fixed 管线。
#[test]
fn test_position_fixed_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "position", "fixed"));
    assert_eq!(style.position, PositionValue::Fixed);
}

/// 测试 overflow-x/y 管线。
#[test]
fn test_overflow_xy_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "overflow-x", "scroll"));
    assert!(apply_property_value(&mut style, "overflow-y", "hidden"));
    assert_eq!(style.overflow_x, OverflowValue::Scroll);
    assert_eq!(style.overflow_y, OverflowValue::Hidden);
}

/// 测试 z-index: auto 管线。
#[test]
fn test_z_index_auto_pipeline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "z-index", "auto"));
    assert_eq!(style.z_index, ZIndexValue::Auto);
}

/// 测试多个 font-weight 值管线。
#[test]
fn test_font_weight_values_pipeline() {
    use zero_css_parser::values::FontWeightValue;
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-weight", "bold"));
    assert_eq!(style.font_weight, FontWeightValue::Bold);
    assert!(apply_property_value(&mut style, "font-weight", "300"));
    assert_eq!(style.font_weight, FontWeightValue::Absolute(300));
    assert!(apply_property_value(&mut style, "font-weight", "normal"));
    assert_eq!(style.font_weight, FontWeightValue::Normal);
}
