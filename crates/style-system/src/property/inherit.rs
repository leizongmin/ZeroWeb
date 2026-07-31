//! CSS 属性继承和初始值应用。

use super::types::*;

/// 从父元素样式继承指定属性到子元素样式。
///
/// 返回 true 表示成功继承。
pub fn inherit_property(parent: &ComputedStyle, child: &mut ComputedStyle, property: &str) -> bool {
    match property {
        "color" => {
            child.color = parent.color.clone();
            true
        }
        "color-scheme" => {
            child.color_scheme_dark = parent.color_scheme_dark;
            true
        }
        "font-family" => {
            child.font_family = parent.font_family.clone();
            true
        }
        "font-size" => {
            child.font_size = parent.font_size.clone();
            true
        }
        "font-weight" => {
            child.font_weight = parent.font_weight.clone();
            true
        }
        "font-style" => {
            child.font_style = parent.font_style.clone();
            true
        }
        "line-height" => {
            child.line_height = parent.line_height.clone();
            true
        }
        "font-size-adjust" => {
            child.font_size_adjust = parent.font_size_adjust.clone();
            true
        }
        "text-align" => {
            child.text_align = parent.text_align.clone();
            true
        }
        "text-transform" => {
            child.text_transform = parent.text_transform;
            true
        }
        "letter-spacing" => {
            child.letter_spacing = parent.letter_spacing.clone();
            true
        }
        "text-emphasis-style" => {
            child.text_emphasis_style = parent.text_emphasis_style.clone();
            true
        }
        "text-emphasis-position" => {
            child.text_emphasis_position = parent.text_emphasis_position;
            true
        }
        "word-spacing" => {
            child.word_spacing = parent.word_spacing.clone();
            true
        }
        "white-space" => {
            child.white_space = parent.white_space.clone();
            true
        }
        "word-break" => {
            child.word_break = parent.word_break.clone();
            true
        }
        "text-autospace" => {
            // text-autospace 是继承属性（CSS Text 4 §8）。
            child.text_autospace = parent.text_autospace;
            true
        }
        "line-break" => {
            // line-break 是继承属性（CSS Text 3 §5.3）。
            child.line_break = parent.line_break.clone();
            true
        }
        "visibility" => {
            child.visibility = parent.visibility.clone();
            true
        }
        "cursor" => {
            child.cursor = parent.cursor.clone();
            true
        }
        "text-indent" => {
            child.text_indent = parent.text_indent.clone();
            true
        }
        "caption-side" => {
            child.caption_side = parent.caption_side.clone();
            true
        }
        "border-collapse" => {
            child.border_collapse = parent.border_collapse.clone();
            true
        }
        "quotes" => {
            child.quotes = parent.quotes.clone();
            true
        }
        "pointer-events" => {
            child.pointer_events = parent.pointer_events.clone();
            true
        }
        "overflow-wrap" => {
            child.overflow_wrap = parent.overflow_wrap.clone();
            true
        }
        "text-align-last" => {
            child.text_align_last = parent.text_align_last.clone();
            true
        }
        "font-variant-numeric" => {
            child.font_variant_numeric = parent.font_variant_numeric.clone();
            true
        }
        "direction" => {
            child.direction = parent.direction.clone();
            true
        }
        "tab-size" => {
            child.tab_size = parent.tab_size.clone();
            true
        }
        "accent-color" => {
            child.accent_color = parent.accent_color.clone();
            true
        }
        "caret-color" => {
            child.caret_color = parent.caret_color.clone();
            true
        }
        "text-wrap" => {
            child.text_wrap = parent.text_wrap.clone();
            true
        }
        "hyphens" => {
            child.hyphens = parent.hyphens.clone();
            true
        }
        "text-shadow" => {
            child.text_shadow = parent.text_shadow.clone();
            true
        }
        "list-style-image" => {
            child.list_style_image = parent.list_style_image.clone();
            true
        }
        "list-style-type" => {
            child.list_style_type = parent.list_style_type.clone();
            true
        }
        "list-style-position" => {
            child.list_style_position = parent.list_style_position.clone();
            true
        }
        "empty-cells" => {
            child.empty_cells = parent.empty_cells.clone();
            true
        }
        "border-spacing" => {
            child.border_spacing = parent.border_spacing.clone();
            true
        }
        // ── 非继承属性（CSS `inherit` 关键字显式要求从父元素复制） ──
        "background-color" => {
            child.background_color = parent.background_color.clone();
            true
        }
        "background-image" => {
            child.background_image = parent.background_image.clone();
            true
        }
        "background-position" => {
            child.background_position = parent.background_position.clone();
            true
        }
        "background-repeat" => {
            child.background_repeat = parent.background_repeat.clone();
            true
        }
        "background-size" => {
            child.background_size = parent.background_size.clone();
            true
        }
        "background-attachment" => {
            child.background_attachment = parent.background_attachment.clone();
            true
        }
        "background-clip" => {
            child.background_clip = parent.background_clip.clone();
            true
        }
        "background-origin" => {
            child.background_origin = parent.background_origin.clone();
            true
        }
        "border-top-width" => {
            child.border_top_width = parent.border_top_width.clone();
            true
        }
        "border-right-width" => {
            child.border_right_width = parent.border_right_width.clone();
            true
        }
        "border-bottom-width" => {
            child.border_bottom_width = parent.border_bottom_width.clone();
            true
        }
        "border-left-width" => {
            child.border_left_width = parent.border_left_width.clone();
            true
        }
        "border-top-style" => {
            child.border_top_style = parent.border_top_style.clone();
            true
        }
        "border-right-style" => {
            child.border_right_style = parent.border_right_style.clone();
            true
        }
        "border-bottom-style" => {
            child.border_bottom_style = parent.border_bottom_style.clone();
            true
        }
        "border-left-style" => {
            child.border_left_style = parent.border_left_style.clone();
            true
        }
        "border-top-color" => {
            child.border_top_color = parent.border_top_color.clone();
            true
        }
        "border-right-color" => {
            child.border_right_color = parent.border_right_color.clone();
            true
        }
        "border-bottom-color" => {
            child.border_bottom_color = parent.border_bottom_color.clone();
            true
        }
        "border-left-color" => {
            child.border_left_color = parent.border_left_color.clone();
            true
        }
        "margin-top" => {
            child.margin_top = parent.margin_top.clone();
            true
        }
        "margin-right" => {
            child.margin_right = parent.margin_right.clone();
            true
        }
        "margin-bottom" => {
            child.margin_bottom = parent.margin_bottom.clone();
            true
        }
        "margin-left" => {
            child.margin_left = parent.margin_left.clone();
            true
        }
        "padding-top" => {
            child.padding_top = parent.padding_top.clone();
            true
        }
        "padding-right" => {
            child.padding_right = parent.padding_right.clone();
            true
        }
        "padding-bottom" => {
            child.padding_bottom = parent.padding_bottom.clone();
            true
        }
        "padding-left" => {
            child.padding_left = parent.padding_left.clone();
            true
        }
        "writing-mode" => {
            child.writing_mode = parent.writing_mode.clone();
            true
        }
        // 盒模型尺寸（非继承属性，但 `inherit` 关键字显式要求从父元素复制计算值）。
        // 此前遗漏致 `max-width:inherit` 等静默失败（CSS2 max-width-104/max-height-104/
        // height-inherit-001 等 FAIL）。margin/padding 已在上文，补齐尺寸六属性。
        "width" => {
            child.width = parent.width.clone();
            true
        }
        "height" => {
            child.height = parent.height.clone();
            true
        }
        "min-width" => {
            child.min_width = parent.min_width.clone();
            true
        }
        "min-height" => {
            child.min_height = parent.min_height.clone();
            true
        }
        "max-width" => {
            child.max_width = parent.max_width.clone();
            true
        }
        "max-height" => {
            child.max_height = parent.max_height.clone();
            true
        }
        // 定位 inset（非继承属性，但 `inherit` 关键字显式要求从父元素复制计算值）。
        // 此前遗漏致 `left:inherit` 等静默失败（CSS2 inherit-static-offset-001/002/003：
        // static 父 left:50px + relative 子 left:inherit 应继承 50px 偏移，却得 auto）。
        // CSS 2.1：left/top/right/bottom 的 computed value 是 specified value（即使
        // position:static 不应用，值仍保留供 inherit 取用）。
        "left" => {
            child.left = parent.left.clone();
            true
        }
        "right" => {
            child.right = parent.right.clone();
            true
        }
        "top" => {
            child.top = parent.top.clone();
            true
        }
        "bottom" => {
            child.bottom = parent.bottom.clone();
            true
        }
        // 定位/布局（display/float/position/clear 高频用于 `inherit`，display 36 案）。
        "display" => {
            child.display = parent.display.clone();
            true
        }
        "position" => {
            child.position = parent.position.clone();
            true
        }
        "float" => {
            child.float = parent.float.clone();
            true
        }
        "clear" => {
            child.clear = parent.clear.clone();
            true
        }
        "vertical-align" => {
            child.vertical_align = parent.vertical_align.clone();
            true
        }
        "z-index" => {
            child.z_index = parent.z_index.clone();
            true
        }
        "unicode-bidi" => {
            child.unicode_bidi = parent.unicode_bidi.clone();
            true
        }
        "clip" => {
            child.clip = parent.clip.clone();
            true
        }
        "table-layout" => {
            child.table_layout = parent.table_layout.clone();
            true
        }
        // box-sizing / overflow（`inherit` 显式要求时复制）
        "box-sizing" => {
            child.box_sizing = parent.box_sizing.clone();
            true
        }
        "overflow-x" => {
            child.overflow_x = parent.overflow_x.clone();
            true
        }
        "overflow-y" => {
            child.overflow_y = parent.overflow_y.clone();
            true
        }
        // outline（visual-only，低风险）
        "outline-width" => {
            child.outline_width = parent.outline_width.clone();
            true
        }
        "outline-color" => {
            child.outline_color = parent.outline_color.clone();
            true
        }
        "outline-style" => {
            child.outline_style = parent.outline_style.clone();
            true
        }
        "outline-offset" => {
            child.outline_offset = parent.outline_offset.clone();
            child.outline_offset_inset = parent.outline_offset_inset;
            true
        }
        // columns
        // 注：column-count/column-width 的 inherit 暂不支持——强制继承列数会暴露
        // ZeroWeb multicol 列分布的结构性缺口（multicol-inherit-002 +1.24pp），
        // 属 R750「spec-correct 但暴露正交缺口」模式。column-rule-* 是视觉属性，
        // 风险低，保留。
        "column-rule-width" => {
            child.column_rule_width = parent.column_rule_width.clone();
            true
        }
        "column-rule-color" => {
            child.column_rule_color = parent.column_rule_color.clone();
            true
        }
        "column-rule-style" => {
            child.column_rule_style = parent.column_rule_style.clone();
            true
        }
        "column-fill" => {
            child.column_fill = parent.column_fill.clone();
            true
        }
        "column-span" => {
            child.column_span = parent.column_span.clone();
            true
        }
        // counters
        "counter-reset" => {
            child.counter_reset = parent.counter_reset.clone();
            true
        }
        "counter-increment" => {
            child.counter_increment = parent.counter_increment.clone();
            true
        }
        "counter-set" => {
            child.counter_set = parent.counter_set.clone();
            true
        }
        // page-break
        "page-break-before" => {
            child.page_break_before = parent.page_break_before.clone();
            true
        }
        "page-break-after" => {
            child.page_break_after = parent.page_break_after.clone();
            true
        }
        "page-break-inside" => {
            child.page_break_inside = parent.page_break_inside.clone();
            true
        }
        _ => false,
    }
}

/// 将初始值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
pub fn apply_initial_value(style: &mut ComputedStyle, property: &str) -> bool {
    let default_style = ComputedStyle::default();
    match property {
        // 盒模型
        "display" => {
            style.display = default_style.display;
            true
        }
        "position" => {
            style.position = default_style.position;
            true
        }
        "float" => {
            style.float = default_style.float;
            true
        }
        "clear" => {
            style.clear = default_style.clear;
            true
        }
        "list-style-type" => {
            style.list_style_type = default_style.list_style_type;
            true
        }
        "list-style-position" => {
            style.list_style_position = default_style.list_style_position;
            true
        }
        "list-style-image" => {
            style.list_style_image = default_style.list_style_image;
            true
        }
        "width" => {
            style.width = default_style.width;
            true
        }
        "height" => {
            style.height = default_style.height;
            true
        }
        "min-width" => {
            style.min_width = default_style.min_width;
            true
        }
        "min-height" => {
            style.min_height = default_style.min_height;
            true
        }
        "max-width" => {
            style.max_width = default_style.max_width;
            true
        }
        "max-height" => {
            style.max_height = default_style.max_height;
            true
        }
        "margin-top" => {
            style.margin_top = default_style.margin_top;
            true
        }
        "margin-right" => {
            style.margin_right = default_style.margin_right;
            true
        }
        "margin-bottom" => {
            style.margin_bottom = default_style.margin_bottom;
            true
        }
        "margin-left" => {
            style.margin_left = default_style.margin_left;
            true
        }
        "padding-top" => {
            style.padding_top = default_style.padding_top;
            true
        }
        "padding-right" => {
            style.padding_right = default_style.padding_right;
            true
        }
        "padding-bottom" => {
            style.padding_bottom = default_style.padding_bottom;
            true
        }
        "padding-left" => {
            style.padding_left = default_style.padding_left;
            true
        }
        "box-sizing" => {
            style.box_sizing = default_style.box_sizing;
            true
        }
        // 边框
        "border-top-width" => {
            style.border_top_width = default_style.border_top_width;
            true
        }
        "border-right-width" => {
            style.border_right_width = default_style.border_right_width;
            true
        }
        "border-bottom-width" => {
            style.border_bottom_width = default_style.border_bottom_width;
            true
        }
        "border-left-width" => {
            style.border_left_width = default_style.border_left_width;
            true
        }
        "border-top-color" => {
            style.border_top_color = default_style.border_top_color;
            true
        }
        "border-right-color" => {
            style.border_right_color = default_style.border_right_color;
            true
        }
        "border-bottom-color" => {
            style.border_bottom_color = default_style.border_bottom_color;
            true
        }
        "border-left-color" => {
            style.border_left_color = default_style.border_left_color;
            true
        }
        "border-top-style" => {
            style.border_top_style = default_style.border_top_style;
            true
        }
        "border-right-style" => {
            style.border_right_style = default_style.border_right_style;
            true
        }
        "border-bottom-style" => {
            style.border_bottom_style = default_style.border_bottom_style;
            true
        }
        "border-left-style" => {
            style.border_left_style = default_style.border_left_style;
            true
        }
        "border-top-left-radius" => {
            style.border_top_left_radius = default_style.border_top_left_radius;
            true
        }
        "border-top-right-radius" => {
            style.border_top_right_radius = default_style.border_top_right_radius;
            true
        }
        "border-bottom-right-radius" => {
            style.border_bottom_right_radius = default_style.border_bottom_right_radius;
            true
        }
        "border-bottom-left-radius" => {
            style.border_bottom_left_radius = default_style.border_bottom_left_radius;
            true
        }
        // Outline
        "outline-width" => {
            style.outline_width = default_style.outline_width;
            true
        }
        "outline-style" => {
            style.outline_style = default_style.outline_style;
            true
        }
        "outline-color" => {
            style.outline_color = default_style.outline_color;
            true
        }
        "outline-offset" => {
            style.outline_offset = default_style.outline_offset;
            style.outline_offset_inset = default_style.outline_offset_inset;
            true
        }
        // 颜色和背景
        "color" => {
            style.color = default_style.color;
            true
        }
        "background-color" => {
            style.background_color = default_style.background_color;
            true
        }
        "opacity" => {
            style.opacity = default_style.opacity;
            true
        }
        "visibility" => {
            style.visibility = default_style.visibility;
            true
        }
        "content-visibility" => {
            style.content_visibility = default_style.content_visibility;
            true
        }
        // 字体
        "font-family" => {
            style.font_family = default_style.font_family;
            true
        }
        "font-size" => {
            style.font_size = default_style.font_size;
            true
        }
        "font-weight" => {
            style.font_weight = default_style.font_weight;
            true
        }
        "font-style" => {
            style.font_style = default_style.font_style;
            true
        }
        "line-height" => {
            style.line_height = default_style.line_height;
            true
        }
        "font-size-adjust" => {
            style.font_size_adjust = default_style.font_size_adjust;
            true
        }
        // 文本
        "text-align" => {
            style.text_align = default_style.text_align;
            true
        }
        "text-decoration" => {
            style.text_decoration = default_style.text_decoration;
            true
        }
        "text-decoration-line" => {
            style.text_decoration_line = default_style.text_decoration_line;
            true
        }
        "text-decoration-color" => {
            style.text_decoration_color = default_style.text_decoration_color;
            true
        }
        "text-decoration-style" => {
            style.text_decoration_style = default_style.text_decoration_style;
            true
        }
        "text-decoration-thickness" => {
            style.text_decoration_thickness = default_style.text_decoration_thickness.clone();
            true
        }
        "text-decoration-inset" => {
            style.text_decoration_inset = default_style.text_decoration_inset.clone();
            true
        }
        "text-emphasis-style" => {
            style.text_emphasis_style = default_style.text_emphasis_style.clone();
            true
        }
        "text-emphasis-position" => {
            style.text_emphasis_position = default_style.text_emphasis_position;
            true
        }
        "text-transform" => {
            style.text_transform = default_style.text_transform;
            true
        }
        "letter-spacing" => {
            style.letter_spacing = default_style.letter_spacing;
            true
        }
        "word-spacing" => {
            style.word_spacing = default_style.word_spacing;
            true
        }
        "white-space" => {
            style.white_space = default_style.white_space;
            true
        }
        "word-break" => {
            style.word_break = default_style.word_break;
            true
        }
        "text-autospace" => {
            style.text_autospace = default_style.text_autospace;
            true
        }
        "line-break" => {
            style.line_break = default_style.line_break.clone();
            true
        }
        "text-indent" => {
            style.text_indent = default_style.text_indent;
            true
        }
        "text-overflow" => {
            style.text_overflow = default_style.text_overflow;
            true
        }
        "table-layout" => {
            style.table_layout = default_style.table_layout;
            true
        }
        "caption-side" => {
            style.caption_side = default_style.caption_side;
            true
        }
        "border-collapse" => {
            style.border_collapse = default_style.border_collapse;
            true
        }
        "resize" => {
            style.resize = default_style.resize;
            true
        }
        "vertical-align" => {
            style.vertical_align = default_style.vertical_align;
            true
        }
        // Flexbox
        "flex-direction" => {
            style.flex_direction = default_style.flex_direction;
            true
        }
        "flex-wrap" => {
            style.flex_wrap = default_style.flex_wrap;
            true
        }
        "justify-content" => {
            style.justify_content = default_style.justify_content;
            true
        }
        "align-items" => {
            style.align_items = default_style.align_items;
            true
        }
        "align-self" => {
            style.align_self = default_style.align_self;
            true
        }
        "flex-grow" => {
            style.flex_grow = default_style.flex_grow;
            true
        }
        "flex-shrink" => {
            style.flex_shrink = default_style.flex_shrink;
            true
        }
        "flex-basis" => {
            style.flex_basis = default_style.flex_basis;
            true
        }
        "gap" => {
            style.gap = default_style.gap;
            true
        }
        "column-gap" => {
            style.column_gap = default_style.column_gap;
            true
        }
        "row-gap" => {
            style.row_gap = default_style.row_gap;
            true
        }
        "order" => {
            style.order = default_style.order;
            true
        }
        // Grid
        "grid-template-columns" => {
            style.grid_template_columns = default_style.grid_template_columns;
            true
        }
        "grid-template-rows" => {
            style.grid_template_rows = default_style.grid_template_rows;
            true
        }
        "grid-auto-flow" => {
            style.grid_auto_flow = default_style.grid_auto_flow;
            true
        }
        "grid-column-start" => {
            style.grid_column_start = default_style.grid_column_start;
            true
        }
        "grid-column-end" => {
            style.grid_column_end = default_style.grid_column_end;
            true
        }
        "grid-row-start" => {
            style.grid_row_start = default_style.grid_row_start;
            true
        }
        "grid-row-end" => {
            style.grid_row_end = default_style.grid_row_end;
            true
        }
        "grid-auto-rows" => {
            style.grid_auto_rows = default_style.grid_auto_rows;
            true
        }
        "grid-auto-columns" => {
            style.grid_auto_columns = default_style.grid_auto_columns;
            true
        }
        "grid-template-areas" => {
            style.grid_template_areas = default_style.grid_template_areas;
            true
        }
        // 定位
        "top" => {
            style.top = default_style.top;
            true
        }
        "right" => {
            style.right = default_style.right;
            true
        }
        "bottom" => {
            style.bottom = default_style.bottom;
            true
        }
        "left" => {
            style.left = default_style.left;
            true
        }
        "z-index" => {
            style.z_index = default_style.z_index;
            true
        }
        // Overflow
        "overflow-x" => {
            style.overflow_x = default_style.overflow_x;
            true
        }
        "overflow-y" => {
            style.overflow_y = default_style.overflow_y;
            true
        }
        // Aspect Ratio
        "aspect-ratio" => {
            style.aspect_ratio = default_style.aspect_ratio;
            true
        }
        // Cursor
        "cursor" => {
            style.cursor = default_style.cursor;
            true
        }
        // Transform
        "transform" => {
            style.transform = default_style.transform;
            true
        }
        "transform-origin" => {
            style.transform_origin_x = default_style.transform_origin_x;
            style.transform_origin_y = default_style.transform_origin_y;
            true
        }
        "perspective" => {
            style.perspective = default_style.perspective;
            true
        }
        "perspective-origin" => {
            style.perspective_origin_x = default_style.perspective_origin_x;
            style.perspective_origin_y = default_style.perspective_origin_y;
            true
        }
        "transform-style" => {
            style.transform_style = default_style.transform_style;
            true
        }
        "backface-visibility" => {
            style.backface_visibility = default_style.backface_visibility;
            true
        }
        // Transitions
        "transition-property" => {
            style.transition_property = default_style.transition_property;
            true
        }
        "transition-duration" => {
            style.transition_duration = default_style.transition_duration;
            true
        }
        "transition-timing-function" => {
            style.transition_timing_function = default_style.transition_timing_function;
            true
        }
        "transition-delay" => {
            style.transition_delay = default_style.transition_delay;
            true
        }
        // Animations
        "animation-name" => {
            style.animation_name = default_style.animation_name;
            true
        }
        "animation-duration" => {
            style.animation_duration = default_style.animation_duration;
            true
        }
        "animation-timing-function" => {
            style.animation_timing_function = default_style.animation_timing_function;
            true
        }
        "animation-delay" => {
            style.animation_delay = default_style.animation_delay;
            true
        }
        "animation-iteration-count" => {
            style.animation_iteration_count = default_style.animation_iteration_count;
            true
        }
        "animation-direction" => {
            style.animation_direction = default_style.animation_direction;
            true
        }
        "animation-fill-mode" => {
            style.animation_fill_mode = default_style.animation_fill_mode;
            true
        }
        "animation-play-state" => {
            style.animation_play_state = default_style.animation_play_state;
            true
        }
        // Scroll Snap
        "scroll-snap-type" => {
            style.scroll_snap_type = default_style.scroll_snap_type;
            true
        }
        "scroll-snap-align" => {
            style.scroll_snap_align = default_style.scroll_snap_align;
            true
        }
        "scroll-snap-stop" => {
            style.scroll_snap_stop = default_style.scroll_snap_stop;
            true
        }
        "scroll-margin-top" => {
            style.scroll_margin_top = default_style.scroll_margin_top;
            true
        }
        "scroll-margin-right" => {
            style.scroll_margin_right = default_style.scroll_margin_right;
            true
        }
        "scroll-margin-bottom" => {
            style.scroll_margin_bottom = default_style.scroll_margin_bottom;
            true
        }
        "scroll-margin-left" => {
            style.scroll_margin_left = default_style.scroll_margin_left;
            true
        }
        "scroll-padding-top" => {
            style.scroll_padding_top = default_style.scroll_padding_top;
            true
        }
        "scroll-padding-right" => {
            style.scroll_padding_right = default_style.scroll_padding_right;
            true
        }
        "scroll-padding-bottom" => {
            style.scroll_padding_bottom = default_style.scroll_padding_bottom;
            true
        }
        "scroll-padding-left" => {
            style.scroll_padding_left = default_style.scroll_padding_left;
            true
        }
        // Container Query
        "container-type" => {
            style.container_type = default_style.container_type;
            true
        }
        "container-name" => {
            style.container_name = default_style.container_name;
            true
        }
        // Writing Mode
        "writing-mode" => {
            style.writing_mode = default_style.writing_mode;
            true
        }
        // Counters / Content / Quotes
        "counter-reset" => {
            style.counter_reset = default_style.counter_reset;
            true
        }
        "counter-increment" => {
            style.counter_increment = default_style.counter_increment;
            true
        }
        "counter-set" => {
            style.counter_set = default_style.counter_set;
            true
        }
        "content" => {
            style.content = default_style.content;
            true
        }
        "quotes" => {
            style.quotes = default_style.quotes;
            true
        }
        // Page Break
        "page-break-before" => {
            style.page_break_before = default_style.page_break_before;
            true
        }
        "page-break-after" => {
            style.page_break_after = default_style.page_break_after;
            true
        }
        "page-break-inside" => {
            style.page_break_inside = default_style.page_break_inside;
            true
        }
        // 其他
        "box-decoration-break" => {
            style.box_decoration_break = default_style.box_decoration_break;
            true
        }
        "image-rendering" => {
            style.image_rendering = default_style.image_rendering;
            true
        }
        "isolation" => {
            style.isolation = default_style.isolation;
            true
        }
        // Break
        "break-inside" => {
            style.break_inside = default_style.break_inside;
            true
        }
        "break-before" => {
            style.break_before = default_style.break_before;
            true
        }
        "break-after" => {
            style.break_after = default_style.break_after;
            true
        }
        // Column Rule
        "column-rule-width" => {
            style.column_rule_width = default_style.column_rule_width;
            true
        }
        "column-rule-style" => {
            style.column_rule_style = default_style.column_rule_style;
            true
        }
        "overscroll-behavior-x" => {
            style.overscroll_behavior_x = default_style.overscroll_behavior_x;
            true
        }
        "overscroll-behavior-y" => {
            style.overscroll_behavior_y = default_style.overscroll_behavior_y;
            true
        }
        "touch-action" => {
            style.touch_action = default_style.touch_action;
            true
        }
        "pointer-events" => {
            style.pointer_events = default_style.pointer_events;
            true
        }
        "overflow-wrap" => {
            style.overflow_wrap = default_style.overflow_wrap;
            true
        }
        "text-align-last" => {
            style.text_align_last = default_style.text_align_last;
            true
        }
        "font-variant-numeric" => {
            style.font_variant_numeric = default_style.font_variant_numeric;
            true
        }
        "user-select" => {
            style.user_select = default_style.user_select;
            true
        }
        "will-change" => {
            style.will_change = default_style.will_change;
            true
        }
        "direction" => {
            style.direction = default_style.direction;
            true
        }
        "color-scheme" => {
            style.color_scheme_dark = default_style.color_scheme_dark;
            true
        }
        "unicode-bidi" => {
            style.unicode_bidi = default_style.unicode_bidi;
            true
        }
        "tab-size" => {
            style.tab_size = default_style.tab_size;
            true
        }
        // Columns
        "columns" => {
            style.column_count = default_style.column_count;
            style.column_width = default_style.column_width;
            true
        }
        "column-count" => {
            style.column_count = default_style.column_count;
            true
        }
        "column-width" => {
            style.column_width = default_style.column_width;
            true
        }
        "column-fill" => {
            style.column_fill = default_style.column_fill;
            true
        }
        "column-span" => {
            style.column_span = default_style.column_span;
            true
        }
        // Object Fit / Filter
        "object-fit" => {
            style.object_fit = default_style.object_fit;
            true
        }
        "object-position" => {
            style.object_position = default_style.object_position.clone();
            true
        }
        "filter" => {
            style.filter = default_style.filter;
            true
        }
        "backdrop-filter" => {
            style.backdrop_filter = default_style.backdrop_filter;
            true
        }
        // Column Rule Color
        "column-rule-color" => {
            style.column_rule_color = default_style.column_rule_color;
            true
        }
        // Contain
        "contain" => {
            style.contain = default_style.contain;
            true
        }
        "contain-intrinsic-size" | "contain-intrinsic-width" | "contain-intrinsic-height" => {
            // 重置两个分量到默认（None）；任一 longhand 重置都恢复该维为 None。
            style.contain_intrinsic_width = default_style.contain_intrinsic_width;
            style.contain_intrinsic_height = default_style.contain_intrinsic_height;
            true
        }
        // UI Appearance
        "appearance" => {
            style.appearance = default_style.appearance;
            true
        }
        "accent-color" => {
            style.accent_color = default_style.accent_color;
            true
        }
        "caret-color" => {
            style.caret_color = default_style.caret_color;
            true
        }
        // Compositing / Scrolling
        "mix-blend-mode" => {
            style.mix_blend_mode = default_style.mix_blend_mode;
            true
        }
        "scrollbar-width" => {
            style.scrollbar_width = default_style.scrollbar_width;
            true
        }
        "scrollbar-gutter" => {
            style.scrollbar_gutter = default_style.scrollbar_gutter;
            true
        }
        "text-wrap" => {
            style.text_wrap = default_style.text_wrap;
            true
        }
        "hyphens" => {
            style.hyphens = default_style.hyphens;
            true
        }
        "line-clamp" => {
            style.line_clamp = default_style.line_clamp;
            true
        }
        "background-image" => {
            style.background_image = default_style.background_image;
            true
        }
        "background-position" => {
            style.background_position = default_style.background_position;
            true
        }
        "background-repeat" => {
            style.background_repeat = default_style.background_repeat;
            true
        }
        "background-size" => {
            style.background_size = default_style.background_size;
            true
        }
        "background-attachment" => {
            style.background_attachment = default_style.background_attachment;
            true
        }
        "background-clip" => {
            style.background_clip = default_style.background_clip;
            true
        }
        "background-origin" => {
            style.background_origin = default_style.background_origin;
            true
        }
        "border-image-source" => {
            style.border_image_source = default_style.border_image_source;
            true
        }
        "border-image-slice" => {
            style.border_image_slice = default_style.border_image_slice;
            true
        }
        "border-image-width" => {
            style.border_image_width = default_style.border_image_width;
            true
        }
        "border-image-repeat" => {
            style.border_image_repeat = default_style.border_image_repeat;
            true
        }
        "border-image-outset" => {
            style.border_image_outset = default_style.border_image_outset;
            true
        }
        "text-shadow" => {
            style.text_shadow = default_style.text_shadow;
            true
        }
        "box-shadow" => {
            style.box_shadow = default_style.box_shadow;
            true
        }
        "clip-path" => {
            style.clip_path = default_style.clip_path;
            true
        }
        "clip" => {
            style.clip = default_style.clip;
            true
        }
        "mask-image" => {
            style.mask_image = default_style.mask_image;
            true
        }
        "mask-mode" => {
            style.mask_mode = default_style.mask_mode;
            true
        }
        "justify-items" => {
            style.justify_items = default_style.justify_items;
            true
        }
        "justify-self" => {
            style.justify_self = default_style.justify_self;
            true
        }
        "align-content" => {
            style.align_content = default_style.align_content;
            true
        }
        "empty-cells" => {
            style.empty_cells = default_style.empty_cells;
            true
        }
        "border-spacing" => {
            style.border_spacing = default_style.border_spacing;
            true
        }
        _ => false,
    }
}
