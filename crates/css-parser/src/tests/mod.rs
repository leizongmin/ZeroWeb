// CSS 解析器综合测试。

use crate::ast::*;
use crate::parser::Parser;
use crate::selector;
use crate::tokenizer::{Spanned, Token, Tokenizer, line_column_from_offset};
use crate::values::{
    AnimationDurationValue, AnimationIterationCountValue, AnimationNameValue, BackgroundImageValue,
    BorderCollapseValue, CalcContext, CaptionSideValue, ClearValue, ColorHueMethod, ColorInterpolationSpace,
    ColorValue, ColumnCountValue, ColumnWidthValue, ContainerTypeValue, CursorValue, FloatValue, GradientDirection,
    GradientValue, LengthValue, ListStylePositionValue, ListStyleTypeValue, MarginTrimValue, ObjectFitValue,
    RadialShape, RadialSize, ResizeValue, ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStopValue,
    ScrollSnapTypeValue, TableLayoutValue, TextDecorationInsetValue, TextDecorationLineValue, TextOverflowValue,
    TextTransformValue, TextUnderlineOffsetValue, TimeUnit, TransformFunction, TransformValue, WritingModeValue,
    eval_calc, eval_calc_with_context, parse_animation_direction, parse_animation_duration, parse_animation_fill_mode,
    parse_animation_iteration_count, parse_animation_name, parse_animation_play_state, parse_background_image,
    parse_border_collapse, parse_box_shadow, parse_calc, parse_caption_side, parse_clear, parse_color,
    parse_column_count, parse_column_width, parse_container_type, parse_cursor, parse_float, parse_gradient,
    parse_length, parse_length_shorthand, parse_list_style_position, parse_list_style_type, parse_margin_trim,
    parse_object_fit, parse_opacity, parse_resize, parse_scroll_snap_align, parse_scroll_snap_stop,
    parse_scroll_snap_type, parse_spacing, parse_table_layout, parse_text_decoration_inset, parse_text_decoration_line,
    parse_text_indent, parse_text_overflow, parse_text_shadow, parse_text_transform, parse_text_underline_offset,
    parse_transform, parse_var, parse_writing_mode,
};

/// Helper: 创建标签选择器。
pub(super) fn tag_sel(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}

/// Helper: 创建 ID 选择器。
pub(super) fn id_sel(id: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                },
                None,
            )],
        },
    }
}

/// Helper: 创建类选择器。
pub(super) fn class_sel(cls: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                },
                None,
            )],
        },
    }
}

mod attribute_case_flag;
mod bom_handling;
mod cdo_cdc;
mod counter_style_at_rule;
mod coverage_round10;
mod coverage_round11;
mod coverage_round12;
mod coverage_round3;
mod coverage_round4;
mod coverage_round5;
mod coverage_round6;
mod coverage_round7;
mod coverage_round8;
mod coverage_round9;
mod css_nesting;
mod dir_pseudo;
mod font_feature_values;
mod forgiving_selector_list;
mod lang_list;
mod nth_invalid;
mod nth_of_selector;
mod null_handling;
mod parser_coverage;
mod parser_coverage_extra;
mod parser_coverage_final;
mod parser_coverage_simple;
mod property_at_rule;
mod tests_1;
mod tests_10;
mod tests_11;
mod tests_12;
mod tests_1b;
mod tests_2;
mod tests_3;
mod tests_4;
mod tests_4b;
mod tests_5;
mod tests_9;
mod tokenizer_coverage;
mod tokenizer_coverage2;
mod transform_coverage;
mod types_coverage;
mod types_coverage2;
mod values_coverage;
