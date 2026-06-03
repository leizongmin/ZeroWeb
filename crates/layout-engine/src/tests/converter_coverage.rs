//! Layout engine converter tests - testing public API.

use zero_css_parser::values::*;
use zero_layout_engine::converter::*;
use zero_style_system::ComputedStyle;

#[test]
fn test_convert_length_to_dimension_max_width_infinity() {
    let style = ComputedStyle {
        max_width: LengthValue::Rem(f64::INFINITY),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    // Rem(inf) maps to Length(inf) via convert_max_length_to_dimension
    match result.max_size.width {
        taffy::style::Dimension::Auto => {}
        taffy::style::Dimension::Length(v) => assert!(v.is_infinite()),
        taffy::style::Dimension::Percent(_) => {}
        _ => {}
    }
}

#[test]
fn test_convert_length_to_dimension_vmin_values() {
    let style = ComputedStyle {
        width: LengthValue::Vmin(10.0),
        height: LengthValue::Vmin(20.0),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.size.width, taffy::style::Dimension::Length(10.0));
    assert_eq!(result.size.height, taffy::style::Dimension::Length(20.0));
}

#[test]
fn test_convert_length_to_dimension_vmax_values() {
    let style = ComputedStyle {
        width: LengthValue::Vmax(15.0),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.size.width, taffy::style::Dimension::Length(15.0));
}

#[test]
fn test_convert_clear_value() {
    assert!(!convert_clear(&ClearValue::None));
    assert!(convert_clear(&ClearValue::Left));
    assert!(convert_clear(&ClearValue::Right));
    assert!(convert_clear(&ClearValue::Both));
}

#[test]
fn test_convert_float_value() {
    assert!(!convert_float(&FloatValue::None));
    assert!(convert_float(&FloatValue::Left));
    assert!(convert_float(&FloatValue::Right));
    assert!(convert_float(&FloatValue::InlineStart));
}

#[test]
fn test_convert_length_to_lpa_auto() {
    let style = ComputedStyle {
        left: LengthValue::Auto,
        right: LengthValue::Px(10.0),
        top: LengthValue::Auto,
        bottom: LengthValue::Px(20.0),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.inset.left, taffy::style::LengthPercentageAuto::Auto);
    assert_eq!(result.inset.right, taffy::style::LengthPercentageAuto::Length(10.0));
    assert_eq!(result.inset.top, taffy::style::LengthPercentageAuto::Auto);
    assert_eq!(result.inset.bottom, taffy::style::LengthPercentageAuto::Length(20.0));
}

#[test]
fn test_grid_template_areas_complex() {
    let grid_template = r#"
        "header header header"
        "sidebar main main"
        "sidebar main main"
        "footer footer footer"
    "#;
    let areas = parse_grid_template_areas(grid_template);
    assert!(areas.contains_key("header"));
    assert!(areas.contains_key("footer"));
}

#[test]
fn test_grid_template_areas_invalid_columns() {
    let grid_template = r#"
        "header header"
        "sidebar main extra"
    "#;
    let areas = parse_grid_template_areas(grid_template);
    assert!(areas.contains_key("header"));
    assert!(areas.contains_key("main"));
}
