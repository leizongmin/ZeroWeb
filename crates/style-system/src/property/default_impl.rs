//! ComputedStyle 的 Default 实现。

use super::types::*;

impl Default for ComputedStyle {
    fn default() -> Self {
        let zero = LengthValue::Px(0.0);
        let auto_length = LengthValue::Auto;
        let initial_color = ColorValue::Rgba(0, 0, 0, 255); // black
        let transparent = ColorValue::Transparent;

        Self {
            // 盒模型
            display: DisplayValue::Inline,
            position: PositionValue::Static,
            float: zero_css_parser::values::FloatValue::None,
            clear: zero_css_parser::values::ClearValue::None,
            list_style_type: zero_css_parser::values::ListStyleTypeValue::Disc,
            list_style_position: zero_css_parser::values::ListStylePositionValue::Outside,
            list_style_image: ListStyleImageComputedValue::None,
            writing_mode: WritingModeValue::HorizontalTb,
            width: auto_length.clone(),
            height: auto_length.clone(),
            min_width: LengthValue::Px(0.0),
            min_height: LengthValue::Px(0.0),
            max_width: LengthValue::Px(f64::INFINITY),
            max_height: LengthValue::Px(f64::INFINITY),
            margin_top: LengthValue::Px(0.0),
            margin_right: LengthValue::Px(0.0),
            margin_bottom: LengthValue::Px(0.0),
            margin_left: LengthValue::Px(0.0),
            padding_top: LengthValue::Px(0.0),
            padding_right: LengthValue::Px(0.0),
            padding_bottom: LengthValue::Px(0.0),
            padding_left: LengthValue::Px(0.0),
            box_sizing: BoxSizingValue::ContentBox,

            // 边框
            border_top_width: LengthValue::Px(0.0),
            border_right_width: LengthValue::Px(0.0),
            border_bottom_width: LengthValue::Px(0.0),
            border_left_width: LengthValue::Px(0.0),
            border_top_color: initial_color.clone(),
            border_right_color: initial_color.clone(),
            border_bottom_color: initial_color.clone(),
            border_left_color: initial_color.clone(),
            border_top_style: BorderStyleValue::None,
            border_right_style: BorderStyleValue::None,
            border_bottom_style: BorderStyleValue::None,
            border_left_style: BorderStyleValue::None,
            border_top_left_radius: LengthValue::Px(0.0),
            border_top_right_radius: LengthValue::Px(0.0),
            border_bottom_right_radius: LengthValue::Px(0.0),
            border_bottom_left_radius: LengthValue::Px(0.0),

            // Outline
            outline_width: LengthValue::Px(0.0),
            outline_style: OutlineStyleValue::None,
            outline_color: initial_color.clone(),
            outline_offset: LengthValue::Px(0.0),

            // 颜色和背景
            color: initial_color.clone(),
            background_color: transparent,
            opacity: 1.0,
            visibility: VisibilityValue::Visible,

            // 字体
            font_family: vec![],
            font_size: LengthValue::Px(16.0),
            font_weight: FontWeightValue::Normal,
            font_style: FontStyleValue::Normal,
            line_height: LineHeightValue::Normal,

            // 文本
            text_align: TextAlignValue::Start,
            text_decoration: TextDecorationValue::None,
            text_decoration_line: TextDecorationLineValue::None,
            text_transform: TextTransformValue::None,
            letter_spacing: LengthValue::Px(0.0),
            word_spacing: LengthValue::Px(0.0),
            white_space: WhiteSpaceValue::Normal,
            text_overflow: TextOverflowValue::Clip,
            vertical_align: VerticalAlignValue::Baseline,
            word_break: WordBreakValue::Normal,
            text_indent: LengthValue::Px(0.0),
            resize: ResizeValue::None,

            // 表格
            table_layout: TableLayoutValue::Auto,
            caption_side: CaptionSideValue::Top,
            border_collapse: BorderCollapseValue::Separate,
            empty_cells: EmptyCellsComputedValue::Show,
            border_spacing: BorderSpacingComputedValue {
                horizontal: 0.0,
                vertical: 0.0,
            },

            // Flexbox
            flex_direction: FlexDirectionValue::Row,
            flex_wrap: FlexWrapValue::Nowrap,
            justify_content: AlignmentValue::FlexStart,
            align_items: AlignmentValue::Stretch,
            align_self: AlignmentValue::Stretch,
            justify_items: JustifyItemsValue::Normal,
            justify_self: JustifySelfValue::Auto,
            align_content: AlignContentValue::Normal,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: FlexBasisValue::Auto,
            gap: LengthValue::Px(0.0),
            row_gap: LengthValue::Px(0.0),
            column_gap: LengthValue::Px(0.0),
            order: 0,

            // Grid
            grid_template_columns: None,
            grid_template_rows: None,
            grid_auto_flow: GridAutoFlowValue::Row,
            grid_column_start: GridLineValue::Auto,
            grid_column_end: GridLineValue::Auto,
            grid_row_start: GridLineValue::Auto,
            grid_row_end: GridLineValue::Auto,
            grid_auto_rows: None,
            grid_auto_columns: None,
            grid_template_areas: None,

            // 定位
            top: zero.clone(),
            right: zero.clone(),
            bottom: zero.clone(),
            left: zero,
            z_index: ZIndexValue::Auto,

            // Overflow
            overflow_x: OverflowValue::Visible,
            overflow_y: OverflowValue::Visible,

            // Aspect Ratio
            aspect_ratio: None,

            // Cursor
            cursor: CursorValue::Auto,

            // Transforms
            transform: zero_css_parser::values::TransformValue::None,
            transform_origin_x: LengthValue::Percentage(50.0),
            transform_origin_y: LengthValue::Percentage(50.0),
            perspective: LengthValue::Px(0.0),
            perspective_origin_x: LengthValue::Percentage(50.0),
            perspective_origin_y: LengthValue::Percentage(50.0),
            transform_style: TransformStyleValue::Flat,
            backface_visibility: BackfaceVisibilityValue::Visible,

            // Transitions
            transition_property: vec![],
            transition_duration: vec![],
            transition_timing_function: vec![],
            transition_delay: vec![],

            // Animations
            animation_name: vec![],
            animation_duration: vec![],
            animation_timing_function: vec![],
            animation_delay: vec![],
            animation_iteration_count: vec![],
            animation_direction: vec![],
            animation_fill_mode: vec![],
            animation_play_state: vec![],

            // Scroll Snap
            scroll_snap_type: ScrollSnapType {
                strictness: ScrollSnapStrictness::None,
                axis: ScrollSnapAxis::Both,
            },
            scroll_snap_align: ScrollSnapAlign::None,
            scroll_snap_stop: ScrollSnapStop::Normal,
            scroll_margin_top: 0.0,
            scroll_margin_right: 0.0,
            scroll_margin_bottom: 0.0,
            scroll_margin_left: 0.0,
            scroll_padding_top: ScrollPadding::Auto,
            scroll_padding_right: ScrollPadding::Auto,
            scroll_padding_bottom: ScrollPadding::Auto,
            scroll_padding_left: ScrollPadding::Auto,

            // Container Query
            container_type: ContainerType::Normal,
            container_name: None,

            // Counters / Content / Quotes
            counter_reset: vec![],
            counter_increment: vec![],
            counter_set: vec![],
            content: ContentComputedValue::Normal,
            quotes: QuotesComputedValue::Auto,

            // Page Break
            page_break_before: PageBreakValue::Auto,
            page_break_after: PageBreakValue::Auto,
            page_break_inside: PageBreakValue::Auto,

            // 其他
            box_decoration_break: BoxDecorationBreakValue::Slice,
            image_rendering: ImageRenderingValue::Auto,
            isolation: IsolationValue::Auto,

            // Break
            break_inside: BreakInsideValue::Auto,
            break_before: BreakValue::Auto,
            break_after: BreakValue::Auto,

            // Column Rule
            column_rule_width: ColumnRuleWidthComputedValue::Medium,
            column_rule_style: ColumnRuleStyleComputedValue::None,
            column_rule_color: ColorValue::Rgba(0, 0, 0, 255), // currentColor 解析后默认黑色

            // Contain
            contain: ContainComputedValue::None,

            // Interaction / Performance Hint
            overscroll_behavior_x: OverscrollBehaviorValue::Auto,
            overscroll_behavior_y: OverscrollBehaviorValue::Auto,
            touch_action: TouchActionValue::Auto,
            user_select: UserSelectValue::Auto,
            will_change: WillChangeValue::Auto,
            pointer_events: PointerEventsValue::Auto,

            // Text (新属性)
            overflow_wrap: OverflowWrapValue::Normal,
            text_align_last: TextAlignLastValue::Auto,
            font_variant_numeric: FontVariantNumericValue::Normal,

            // Writing Direction / Tab
            direction: DirectionValue::Ltr,
            unicode_bidi: UnicodeBidiValue::Normal,
            tab_size: TabSizeValue::Number(8),

            // Columns
            column_count: ColumnCountComputedValue::Auto,
            column_width: ColumnWidthComputedValue::Auto,

            // Object Fit / Filter
            object_fit: ObjectFitComputedValue::Fill,
            filter: FilterComputedValue::None,

            // UI Appearance
            appearance: AppearanceComputedValue::Auto,
            accent_color: AccentColorComputedValue::Auto,
            caret_color: CaretColorComputedValue::Auto,

            // Compositing / Scrolling
            mix_blend_mode: MixBlendModeComputedValue::Normal,
            scrollbar_width: ScrollbarWidthComputedValue::Auto,
            scrollbar_gutter: ScrollbarGutterComputedValue::Auto,

            // Text Wrap / Hyphens / Line Clamp
            text_wrap: TextWrapComputedValue::Wrap,
            hyphens: HyphensComputedValue::None,
            line_clamp: LineClampComputedValue::None,

            // Background Image / Position / Repeat / Size / Attachment / Clip / Origin
            background_image: BackgroundImageComputedValue::None,
            background_position: BackgroundPositionComputedValue::Percent(0.0),
            background_repeat: BackgroundRepeatComputedValue::Repeat,
            background_size: BackgroundSizeComputedValue::Auto,
            background_attachment: BackgroundAttachmentComputedValue::Scroll,
            background_clip: BackgroundClipComputedValue::BorderBox,
            background_origin: BackgroundOriginComputedValue::PaddingBox,

            // Border Image (Source / Slice / Width / Repeat / Outset)
            border_image_source: BorderImageSourceComputedValue::None,
            border_image_slice: BorderImageSliceComputedValue {
                top: BorderImageSliceComputedComponent::Number(100.0),
                right: BorderImageSliceComputedComponent::Number(100.0),
                bottom: BorderImageSliceComputedComponent::Number(100.0),
                left: BorderImageSliceComputedComponent::Number(100.0),
                fill: false,
            },
            border_image_width: BorderImageWidthComputedValue {
                top: BorderImageWidthComputedComponent::Number(1.0),
                right: BorderImageWidthComputedComponent::Number(1.0),
                bottom: BorderImageWidthComputedComponent::Number(1.0),
                left: BorderImageWidthComputedComponent::Number(1.0),
            },
            border_image_repeat: BorderImageRepeatComputedValue {
                horizontal: BorderImageRepeatComputedMode::Stretch,
                vertical: BorderImageRepeatComputedMode::Stretch,
            },
            border_image_outset: BorderImageOutsetComputedValue {
                top: BorderImageOutsetComputedComponent::Number(0.0),
                right: BorderImageOutsetComputedComponent::Number(0.0),
                bottom: BorderImageOutsetComputedComponent::Number(0.0),
                left: BorderImageOutsetComputedComponent::Number(0.0),
            },

            text_shadow: TextShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
            },
            box_shadow: BoxShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                spread_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                inset: false,
            },
        }
    }
}
