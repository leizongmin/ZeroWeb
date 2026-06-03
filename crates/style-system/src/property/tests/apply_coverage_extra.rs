// Property apply module - additional coverage tests
use super::*;
use crate::property::ComputedStyle;

#[test]
/// 测试无效的 display 值
fn test_apply_property_value_invalid_display() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "display", "invalid-value"));
    assert_eq!(style.display, DisplayValue::Inline); // 保持默认值
}

#[test]
/// 测试无效的 position 值
fn test_apply_property_value_invalid_position() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "position", "invalid-value"));
    assert_eq!(style.position, PositionValue::Static);
}

#[test]
/// 测试无效的 overflow 值
fn test_apply_property_value_invalid_overflow() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "overflow-x", "invalid-value"));
    assert!(!apply_property_value(&mut style, "overflow-y", "invalid-value"));
    assert_eq!(style.overflow_x, OverflowValue::Visible);
    assert_eq!(style.overflow_y, OverflowValue::Visible);
}

#[test]
/// 测试 max-width: none
fn test_apply_property_value_max_width_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "max-width", "none"));
    assert_eq!(style.max_width, LengthValue::Px(f64::INFINITY));
}

#[test]
/// 测试 max-height: none
fn test_apply_property_value_max_height_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "max-height", "none"));
    assert_eq!(style.max_height, LengthValue::Px(f64::INFINITY));
}

#[test]
/// 测试无效的 border-style 值
fn test_apply_property_value_invalid_border_style() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "border-top-style", "invalid-style"));
    assert_eq!(style.border_top_style, BorderStyleValue::None);
}

#[test]
/// 测试无效的颜色值
fn test_apply_property_value_invalid_color() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "color", "invalid-color"));
    assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255)); // 保持默认黑色
}

#[test]
/// 测试无效的 font-weight 值
fn test_apply_property_value_invalid_font_weight() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "font-weight", "invalid-weight"));
    assert_eq!(style.font_weight, FontWeightValue::Normal);
}

#[test]
/// 测试 font-weight: bold
fn test_apply_property_value_font_weight_bold() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-weight", "bold"));
    assert_eq!(style.font_weight, FontWeightValue::Bold);
}

#[test]
/// 测试 font-weight: 700
fn test_apply_property_value_font_weight_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-weight", "700"));
    assert_eq!(style.font_weight, FontWeightValue::Bold);
}

#[test]
/// 测试无效的 font-style 值
fn test_apply_property_value_invalid_font_style() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "font-style", "invalid-style"));
    assert_eq!(style.font_style, FontStyleValue::Normal);
}

#[test]
/// 测试 font-style: italic
fn test_apply_property_value_font_style_italic() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-style", "italic"));
    assert_eq!(style.font_style, FontStyleValue::Italic);
}

#[test]
/// 测试无效的 text-align 值
fn test_apply_property_value_invalid_text_align() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "text-align", "invalid-align"));
    assert_eq!(style.text_align, TextAlignValue::Left);
}

#[test]
/// 测试 text-align: center
fn test_apply_property_value_text_align_center() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-align", "center"));
    assert_eq!(style.text_align, TextAlignValue::Center);
}

#[test]
/// 测试无效的 visibility 值
fn test_apply_property_value_invalid_visibility() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "visibility", "invalid-visibility"));
    assert_eq!(style.visibility, VisibilityValue::Visible);
}

#[test]
/// 测试 visibility: hidden
fn test_apply_property_value_visibility_hidden() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "visibility", "hidden"));
    assert_eq!(style.visibility, VisibilityValue::Hidden);
}

#[test]
/// 测试 visibility: collapse
fn test_apply_property_value_visibility_collapse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "visibility", "collapse"));
    assert_eq!(style.visibility, VisibilityValue::Collapse);
}

#[test]
/// 测试无效的 box-sizing 值
fn test_apply_property_value_invalid_box_sizing() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "box-sizing", "invalid-box-sizing"));
    assert_eq!(style.box_sizing, BoxSizingValue::ContentBox);
}

#[test]
/// 测试 box-sizing: border-box
fn test_apply_property_value_box_sizing_border_box() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "box-sizing", "border-box"));
    assert_eq!(style.box_sizing, BoxSizingValue::BorderBox);
}

#[test]
/// 测试无效的 flex-direction 值
fn test_apply_property_value_invalid_flex_direction() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "flex-direction", "invalid-direction"));
    assert_eq!(style.flex_direction, FlexDirectionValue::Row);
}

#[test]
/// 测试 flex-direction: column
fn test_apply_property_value_flex_direction_column() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-direction", "column"));
    assert_eq!(style.flex_direction, FlexDirectionValue::Column);
}

#[test]
/// 测试 flex-direction: row-reverse
fn test_apply_property_value_flex_direction_row_reverse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-direction", "row-reverse"));
    assert_eq!(style.flex_direction, FlexDirectionValue::RowReverse);
}

#[test]
/// 测试 flex-direction: column-reverse
fn test_apply_property_value_flex_direction_column_reverse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-direction", "column-reverse"));
    assert_eq!(style.flex_direction, FlexDirectionValue::ColumnReverse);
}

#[test]
/// 测试无效的 flex-wrap 值
fn test_apply_property_value_invalid_flex_wrap() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "flex-wrap", "invalid-wrap"));
    assert_eq!(style.flex_wrap, FlexWrapValue::NoWrap);
}

#[test]
/// 测试 flex-wrap: wrap
fn test_apply_property_value_flex_wrap_wrap() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-wrap", "wrap"));
    assert_eq!(style.flex_wrap, FlexWrapValue::Wrap);
}

#[test]
/// 测试 flex-wrap: wrap-reverse
fn test_apply_property_value_flex_wrap_wrap_reverse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-wrap", "wrap-reverse"));
    assert_eq!(style.flex_wrap, FlexWrapValue::WrapReverse);
}

#[test]
/// 测试无效的 justify-content 值
fn test_apply_property_value_invalid_justify_content() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "justify-content", "invalid-content"));
    assert_eq!(style.justify_content, JustifyContentValue::FlexStart);
}

#[test]
/// 测试 justify-content: space-between
fn test_apply_property_value_justify_content_space_between() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "justify-content", "space-between"));
    assert_eq!(style.justify_content, JustifyContentValue::SpaceBetween);
}

#[test]
/// 测试 justify-content: space-around
fn test_apply_property_value_justify_content_space_around() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "justify-content", "space-around"));
    assert_eq!(style.justify_content, JustifyContentValue::SpaceAround);
}

#[test]
/// 测试 justify-content: space-evenly
fn test_apply_property_value_justify_content_space_evenly() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "justify-content", "space-evenly"));
    assert_eq!(style.justify_content, JustifyContentValue::SpaceEvenly);
}

#[test]
/// 测试 flex-grow: 2.5
fn test_apply_property_value_flex_grow_decimal() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-grow", "2.5"));
    assert_eq!(style.flex_grow, 2.5);
}

#[test]
/// 测试 flex-grow: 0
fn test_apply_property_value_flex_grow_zero() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-grow", "0"));
    assert_eq!(style.flex_grow, 0.0);
}

#[test]
/// 测试 flex-shrink: 2.5
fn test_apply_property_value_flex_shrink_decimal() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-shrink", "2.5"));
    assert_eq!(style.flex_shrink, 2.5);
}

#[test]
/// 测试 flex-basis: auto
fn test_apply_property_value_flex_basis_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-basis", "auto"));
    assert_eq!(style.flex_basis, FlexBasisValue::Auto);
}

#[test]
/// 测试 flex-basis: 200px
fn test_apply_property_value_flex_basis_length() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-basis", "200px"));
    assert_eq!(style.flex_basis, FlexBasisValue::Length(LengthValue::Px(200.0)));
}

#[test]
/// 测试 flex-basis: content
fn test_apply_property_value_flex_basis_content() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-basis", "content"));
    assert_eq!(style.flex_basis, FlexBasisValue::Content);
}

#[test]
/// 测试 aspect-ratio: auto
fn test_apply_property_value_aspect_ratio_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "aspect-ratio", "auto"));
    assert!(style.aspect_ratio.is_none());
}

#[test]
/// 测试 aspect-ratio: 16/9
fn test_apply_property_value_aspect_ratio_fraction() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "aspect-ratio", "16 / 9"));
    assert_eq!(style.aspect_ratio, Some(16.0 / 9.0));
}

#[test]
/// 测试 aspect-ratio: 1.777
fn test_apply_property_value_aspect_ratio_decimal() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "aspect-ratio", "1.777"));
    assert_eq!(style.aspect_ratio, Some(1.777));
}

#[test]
/// 测试无效的 cursor 值
fn test_apply_property_value_invalid_cursor() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "cursor", "invalid-cursor"));
    assert_eq!(style.cursor, CursorValue::Auto);
}

#[test]
/// 测试 cursor: pointer
fn test_apply_property_value_cursor_pointer() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "cursor", "pointer"));
    assert_eq!(style.cursor, CursorValue::Pointer);
}

#[test]
/// 测试 cursor: not-allowed
fn test_apply_property_value_cursor_not_allowed() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "cursor", "not-allowed"));
    assert_eq!(style.cursor, CursorValue::NotAllowed);
}

#[test]
/// 测试 cursor: zoom-in
fn test_apply_property_value_cursor_zoom_in() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "cursor", "zoom-in"));
    assert_eq!(style.cursor, CursorValue::ZoomIn);
}

#[test]
/// 测试 overflow-wrap: break-word
fn test_apply_property_value_overflow_wrap_break_word() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "overflow-wrap", "break-word"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::BreakWord);
}

#[test]
/// 测试 overflow-wrap: anywhere
fn test_apply_property_value_overflow_wrap_anywhere() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "overflow-wrap", "anywhere"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::Anywhere);
}

#[test]
/// 测试 text-align-last: center
fn test_apply_property_value_text_align_last_center() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-align-last", "center"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Center);
}

#[test]
/// 测试 text-align-last: justify
fn test_apply_property_value_text_align_last_justify() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-align-last", "justify"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Justify);
}

#[test]
/// 测试 font-variant-numeric: tabular-nums
fn test_apply_property_value_font_variant_numeric_tabular_nums() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-variant-numeric", "tabular-nums"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::TabularNums);
}

#[test]
/// 测试 font-variant-numeric: slashed-zero
fn test_apply_property_value_font_variant_numeric_slashed_zero() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-variant-numeric", "slashed-zero"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::SlashedZero);
}

#[test]
/// 测试 direction: rtl
fn test_apply_property_value_direction_rtl() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "direction", "rtl"));
    assert_eq!(style.direction, DirectionValue::Rtl);
}

#[test]
/// 测试 unicode-bidi: embed
fn test_apply_property_value_unicode_bidi_embed() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "unicode-bidi", "embed"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Embed);
}

#[test]
/// 测试 unicode-bidi: plaintext
fn test_apply_property_value_unicode_bidi_plaintext() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "unicode-bidi", "plaintext"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Plaintext);
}

#[test]
/// 测试 tab-size: 4
fn test_apply_property_value_tab_size_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "tab-size", "4"));
    assert_eq!(style.tab_size, TabSizeValue::Number(4.0));
}

#[test]
/// 测试 tab-size: 2em
fn test_apply_property_value_tab_size_length() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "tab-size", "2em"));
    assert_eq!(style.tab_size, TabSizeValue::Length(LengthValue::Em(2.0)));
}

#[test]
/// 测试 object-fit: cover
fn test_apply_property_value_object_fit_cover() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "object-fit", "cover"));
    assert_eq!(style.object_fit, ObjectFitComputedValue::Cover);
}

#[test]
/// 测试 object-fit: scale-down
fn test_apply_property_value_object_fit_scale_down() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "object-fit", "scale-down"));
    assert_eq!(style.object_fit, ObjectFitComputedValue::ScaleDown);
}

#[test]
/// 测试 filter: blur(5px)
fn test_apply_property_value_filter_blur() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "filter", "blur(5px)"));
    assert_eq!(style.filter, FilterComputedValue::Blur(5.0));
}

#[test]
/// 测试 filter: brightness(1.5)
fn test_apply_property_value_filter_brightness() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "filter", "brightness(1.5)"));
    assert_eq!(style.filter, FilterComputedValue::Brightness(1.5));
}

#[test]
/// 测试 filter: drop-shadow(2px 2px 4px rgba(0,0,0,0.5))
fn test_apply_property_value_filter_drop_shadow() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "filter", "drop-shadow(2px 2px 4px rgba(0,0,0,0.5))"));
    match style.filter {
        FilterComputedValue::DropShadow(x, y, blur, color) => {
            assert_eq!(x, 2.0);
            assert_eq!(y, 2.0);
            assert_eq!(blur, 4.0);
            assert_eq!(color, ColorValue::Rgba(0, 0, 0, 128));
        }
        _ => panic!("Expected drop-shadow filter"),
    }
}

#[test]
/// 测试 mix-blend-mode: multiply
fn test_apply_property_value_mix_blend_mode_multiply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "multiply"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Multiply);
}

#[test]
/// 测试 mix-blend-mode: screen
fn test_apply_property_value_mix_blend_mode_screen() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "screen"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Screen);
}

#[test]
/// 测试 mix-blend-mode: overlay
fn test_apply_property_value_mix_blend_mode_overlay() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "overlay"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Overlay);
}

#[test]
/// 测试 mix-blend-mode: color
fn test_apply_property_value_mix_blend_mode_color() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "color"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Color);
}

#[test]
/// 测试 text-wrap: balance
fn test_apply_property_value_text_wrap_balance() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "balance"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Balance);
}

#[test]
/// 测试 text-wrap: pretty
fn test_apply_property_value_text_wrap_pretty() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "pretty"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Pretty);
}

#[test]
/// 测试 hyphens: auto
fn test_apply_property_value_hyphens_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "hyphens", "auto"));
    assert_eq!(style.hyphens, HyphensComputedValue::Auto);
}

#[test]
/// 测试 hyphens: manual
fn test_apply_property_value_hyphens_manual() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "hyphens", "manual"));
    assert_eq!(style.hyphens, HyphensComputedValue::Manual);
}

#[test]
/// 测试 line-clamp: none
fn test_apply_property_value_line_clamp_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-clamp", "none"));
    assert_eq!(style.line_clamp, LineClampComputedValue::None);
}

#[test]
/// 测试 line-clamp: 3
fn test_apply_property_value_line_clamp_count() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-clamp", "3"));
    assert_eq!(style.line_clamp, LineClampComputedValue::Count(3.0));
}

#[test]
/// 测试 background-image: url('image.jpg')
fn test_apply_property_value_background_image_url() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "url('image.jpg')"));
    assert_eq!(style.background_image, BackgroundImageComputedValue::Url("image.jpg".to_string()));
}

#[test]
/// 测试 background-image: linear-gradient(red, blue)
fn test_apply_property_value_background_image_gradient() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "linear-gradient(red, blue)"));
    assert!(matches!(style.background_image, BackgroundImageComputedValue::Gradient(_)));
}

#[test]
/// 测试 background-position: center
fn test_apply_property_value_background_position_center() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "center"));
    assert_eq!(style.background_position, BackgroundPositionComputedValue::Center);
}

#[test]
/// 测试 background-position: 25% 75%
fn test_apply_property_value_background_position_percent() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "25% 75%"));
    assert_eq!(style.background_position, BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Percent(25.0)),
        Box::new(BackgroundPositionComputedValue::Percent(75.0))
    ));
}

#[test]
/// 测试 background-repeat: round
fn test_apply_property_value_background_repeat_round() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "round"));
    assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::Round);
}

#[test]
/// 测试 background-size: cover
fn test_apply_property_value_background_size_cover() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "cover"));
    assert_eq!(style.background_size, BackgroundSizeComputedValue::Cover);
}

#[test]
/// 测试 background-attachment: fixed
fn test_apply_property_value_background_attachment_fixed() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-attachment", "fixed"));
    assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Fixed);
}

#[test]
/// 测试 background-clip: text
fn test_apply_property_value_background_clip_text() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-clip", "text"));
    assert_eq!(style.background_clip, BackgroundClipComputedValue::Text);
}

#[test]
/// 测试 border-image-source: url('border.png')
fn test_apply_property_value_border_image_source_url() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-source", "url('border.png')"));
    assert_eq!(style.border_image_source, BorderImageSourceComputedValue::Url("border.png".to_string()));
}

#[test]
/// 测试 border-image-source: none
fn test_apply_property_value_border_image_source_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-image-source", "none"));
    assert_eq!(style.border_image_source, BorderImageSourceComputedValue::None);
}

#[test]
/// 测试 text-shadow: 1px 1px 2px rgba(0,0,0,0.5)
fn test_apply_property_value_text_shadow() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-shadow", "1px 1px 2px rgba(0,0,0,0.5)"));
    assert_eq!(style.text_shadow.offset_x, 1.0);
    assert_eq!(style.text_shadow.offset_y, 1.0);
    assert_eq!(style.text_shadow.blur_radius, 2.0);
    assert_eq!(style.text_shadow.color, ColorValue::Rgba(0, 0, 0, 128));
}

#[test]
/// 测试 box-shadow: inset 2px 2px 4px rgba(0,0,0,0.3)
fn test_apply_property_value_box_shadow_inset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "box-shadow", "inset 2px 2px 4px rgba(0,0,0,0.3)"));
    assert_eq!(style.box_shadow.offset_x, 2.0);
    assert_eq!(style.box_shadow.offset_y, 2.0);
    assert_eq!(style.box_shadow.blur_radius, 4.0);
    assert_eq!(style.box_shadow.spread_radius, 0.0);
    assert_eq!(style.box_shadow.color, ColorValue::Rgba(0, 0, 0, 77));
    assert!(style.box_shadow.inset);
}

#[test]
/// 测试 empty-cells: hide
fn test_apply_property_value_empty_cells_hide() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "empty-cells", "hide"));
    assert_eq!(style.empty_cells, EmptyCellsComputedValue::Hide);
}

#[test]
/// 测试 border-spacing: 10px 20px
fn test_apply_property_value_border_spacing() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-spacing", "10px 20px"));
    assert_eq!(style.border_spacing.horizontal, 10.0);
    assert_eq!(style.border_spacing.vertical, 20.0);
}

#[test]
/// 测试未知属性（应该保持原样，但不应用）
fn test_apply_property_value_unknown_property() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "unknown-property", "value"));
    // 检查默认值没有被改变
    assert_eq!(style.display, DisplayValue::Inline);
}

// 辅助函数：创建 ComputedStyle
fn create_test_style() -> ComputedStyle {
    let mut style = ComputedStyle::default();
    // 设置一些非默认值以便测试
    style.display = DisplayValue::Block;
    style.position = PositionValue::Relative;
    style.width = LengthValue::Px(100.0);
    style.height = LengthValue::Px(50.0);
    style
}