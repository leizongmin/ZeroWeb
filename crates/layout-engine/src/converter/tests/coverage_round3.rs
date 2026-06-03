//! 覆盖率补全第三轮：converter 内部函数覆盖

use super::super::*;
use zero_css_parser::values::*;
use zero_style_system::ComputedStyle;

#[test]
fn test_convert_length_ch_unit() {
    let style = ComputedStyle {
        width: LengthValue::Ch(2.5),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.size.width, taffy::style::Dimension::Length(2.5));
}

#[test]
fn test_convert_length_to_lp_vmax_and_ch() {
    let style = ComputedStyle {
        padding_top: LengthValue::Vmax(5.0),
        padding_left: LengthValue::Ch(1.5),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::Length(5.0));
    assert_eq!(result.padding.left, taffy::style::LengthPercentage::Length(1.5));
}

#[test]
fn test_convert_length_to_lpa_rem_and_vh() {
    let style = ComputedStyle {
        left: LengthValue::Rem(1.5),
        right: LengthValue::Vh(10.0),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.inset.left, taffy::style::LengthPercentageAuto::Length(1.5));
    assert_eq!(result.inset.right, taffy::style::LengthPercentageAuto::Length(10.0));
}

#[test]
fn test_convert_length_to_lpa_vmin_vmax_ch() {
    let style = ComputedStyle {
        left: LengthValue::Vmin(3.0),
        top: LengthValue::Vmax(7.0),
        right: LengthValue::Ch(0.5),
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert_eq!(result.inset.left, taffy::style::LengthPercentageAuto::Length(3.0));
    assert_eq!(result.inset.top, taffy::style::LengthPercentageAuto::Length(7.0));
    assert_eq!(result.inset.right, taffy::style::LengthPercentageAuto::Length(0.5));
}

#[test]
fn test_parse_grid_tracks_invalid_repeat_count() {
    let track_def = "repeat(abc, 100px)";
    let tracks = parse_grid_tracks(&Some(track_def.to_string()));
    assert!(!tracks.is_empty());
}

#[test]
fn test_parse_grid_tracks_mixed_units() {
    let track_def = "100px 1fr auto minmax(50px, 2fr)";
    let tracks = parse_grid_tracks(&Some(track_def.to_string()));
    assert_eq!(tracks.len(), 4);
}

#[test]
fn test_parse_grid_tracks_min_max_content() {
    let track_def = "min-content max-content";
    let tracks = parse_grid_tracks(&Some(track_def.to_string()));
    assert_eq!(tracks.len(), 2);
}

#[test]
fn test_convert_alignment_baseline_for_align_self() {
    // Baseline is valid for align_self but not align_content
    let style = ComputedStyle {
        align_self: AlignmentValue::Baseline,
        ..ComputedStyle::default()
    };
    let result = computed_style_to_taffy(&style, None);
    assert!(result.align_self.is_some());
}

#[test]
fn test_resolve_named_area_all_directions() {
    let mut areas = std::collections::HashMap::new();
    areas.insert("header".to_string(), (1, 2, 1, 4));
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "row-start"),
        GridLineValue::Line(1)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "row-end"),
        GridLineValue::Line(2)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "col-start"),
        GridLineValue::Line(1)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "col-end"),
        GridLineValue::Line(4)
    );
}

#[test]
fn test_resolve_named_area_no_map() {
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test".to_string()), None, "row-start"),
        GridLineValue::Auto
    );
}

#[test]
fn test_parse_single_track_min_max_content() {
    let track = parse_single_track("min-content");
    assert!(matches!(track, taffy::style::TrackSizingFunction::Single(_)));
    let track = parse_single_track("max-content");
    assert!(matches!(track, taffy::style::TrackSizingFunction::Single(_)));
}

#[test]
fn test_find_top_level_comma_nested() {
    let input = "minmax(100px, 1fr), auto";
    let pos = find_top_level_comma(input);
    assert_eq!(pos, Some(18));
}

#[test]
fn test_parse_repeat_auto_fill() {
    let track_def = "repeat(auto-fill, 200px)";
    let tracks = parse_grid_tracks(&Some(track_def.to_string()));
    assert_eq!(tracks.len(), 1);
    if let taffy::style::TrackSizingFunction::Repeat(repetition, _) = &tracks[0] {
        assert_eq!(repetition, &taffy::style::GridTrackRepetition::AutoFill);
    } else {
        panic!("Expected Repeat");
    }
}

#[test]
fn test_parse_repeat_auto_fit() {
    let track_def = "repeat(auto-fit, 1fr minmax(100px, 1fr))";
    let tracks = parse_grid_tracks(&Some(track_def.to_string()));
    assert_eq!(tracks.len(), 1);
    if let taffy::style::TrackSizingFunction::Repeat(repetition, _) = &tracks[0] {
        assert_eq!(repetition, &taffy::style::GridTrackRepetition::AutoFit);
    } else {
        panic!("Expected Repeat");
    }
}

#[test]
fn test_minmax_parsing_complex() {
    let track = parse_single_track("minmax(min-content, max-content)");
    if let taffy::style::TrackSizingFunction::Single(taffy::geometry::MinMax { min, max }) = track {
        assert!(matches!(min, taffy::style::MinTrackSizingFunction::Auto));
        assert!(matches!(max, taffy::style::MaxTrackSizingFunction::Auto));
    } else {
        panic!("Expected TrackSizingFunction::Single");
    }
}
