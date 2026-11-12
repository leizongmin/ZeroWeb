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
    ScrollSnapTypeValue, TableLayoutValue, TextDecorationInsetValue, TextDecorationLineValue,
    TextDecorationSkipSpacesValue, TextOverflowValue, TextTransformValue, TextUnderlineOffsetValue, TimeUnit,
    TransformFunction, TransformValue, WritingModeValue, eval_calc, eval_calc_with_context, parse_animation_direction,
    parse_animation_duration, parse_animation_fill_mode, parse_animation_iteration_count, parse_animation_name,
    parse_animation_play_state, parse_background_image, parse_border_collapse, parse_box_shadow, parse_calc,
    parse_caption_side, parse_clear, parse_color, parse_column_count, parse_column_width, parse_container_type,
    parse_cursor, parse_float, parse_gradient, parse_length, parse_length_shorthand, parse_list_style_position,
    parse_list_style_type, parse_margin_trim, parse_object_fit, parse_opacity, parse_resize, parse_scroll_snap_align,
    parse_scroll_snap_stop, parse_scroll_snap_type, parse_spacing, parse_table_layout, parse_text_decoration_inset,
    parse_text_decoration_line, parse_text_decoration_skip_spaces, parse_text_indent, parse_text_overflow,
    parse_text_shadow, parse_text_transform, parse_text_underline_offset, parse_transform, parse_var,
    parse_writing_mode,
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
mod host_selector;
mod lang_list;
mod nth_invalid;
mod nth_of_selector;
mod null_handling;
mod parser_coverage;
mod parser_coverage_extra;
mod parser_coverage_final;
mod parser_coverage_simple;
/// `@position-try` at-rule 解析契约测试。
mod position_try_at_rule;
mod property_at_rule;
/// R2280 + CSS Values 4 §10.5：individual rotate/scale 数学函数文法位测试。
mod r2280_math_function_slots_tests;
/// R2320 + CSS Values 4 §10.10/§10.11：数学函数 specified 值折叠序列化 +
/// transform 列表 rotate 臂 math 接线测试。
mod r2320_math_specified_serialize_tests;
/// R5076 scale 族百分比参数解析测试。
mod r5076_scale_percent_tests;
mod r5400_qualified_rule_recovery_tests;
mod r5401_safe_overflow_position_tests;
mod review_r3344;
/// sibling-index()/sibling-count() 求值 + 写面 canonical 串化钉面（g19）。
mod sibling_tree_counting_tests;
/// 45 号 M2：`::slotted(<compound-selector>)` 解析契约与特异性。
mod slotted_selector;
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
/// CSS Values 4 §10.5 型别表 + §10.9 IEEE sign/除零 + §10.8 hypot 变参 +
/// §10.11 scale 族折叠（WPT signs-abs / hypot-pow-sqrt 桶驱动）。
mod values_math_type_table_tests;
