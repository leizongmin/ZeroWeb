//! ComputedStyle 的 Default 实现。

use super::computed_style::ComputedStyle;
use super::types::*;

impl Default for ComputedStyle {
    fn default() -> Self {
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
            // CSS 规范：min-width/min-height 的 initial value 是 `auto`（非 0）。
            // 对 flex/grid item，`auto` 表示「基于内容的自动最小尺寸」(§4.5/§6.6)，
            // 经 converter→Dimension::Auto 让 taffy 计算 min-content floor。
            // 旧默认 `Px(0.0)`→`Some(0.0)` 会短路 taffy 的内容下限，使 flex item 可缩至 0
            // (R428-R437 验证：css-flexbox +14/496、css-grid +1/48、css-multicol 不变、0 净回归)。
            // 对 block 元素，taffy 把 min-size:Auto 视作 0，行为不变。
            min_width: LengthValue::Auto,
            min_height: LengthValue::Auto,
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

            // 边框 — border-width 初始值 = medium（CSS §8.5.1），ZeroWeb 取 3px。
            // 默认无边框：border-style 初始 = none，converter 在 style=none/hidden 时
            // 把 border-width 抑制为 0（不进布局盒），故默认元素无布局边框。
            border_top_width: LengthValue::Px(3.0),
            border_right_width: LengthValue::Px(3.0),
            border_bottom_width: LengthValue::Px(3.0),
            border_left_width: LengthValue::Px(3.0),
            // border-color 初始值 = currentColor（CSS §8.5.1）。currentColor 作为关键字
            // 经层叠/继承保留（CSS-Color §resolving），paint 时解析为元素自身计算 `color`。
            // 默认元素无边框（border-style=none），故不影响默认渲染。
            border_top_color: ColorValue::CurrentColor,
            border_right_color: ColorValue::CurrentColor,
            border_bottom_color: ColorValue::CurrentColor,
            border_left_color: ColorValue::CurrentColor,
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
            font_size_adjust: FontSizeAdjustValue::None,

            // 文本
            text_align: TextAlignValue::Start,
            text_decoration: TextDecorationValue::None,
            text_decoration_line: TextDecorationLineValue::None,
            text_decoration_color: ColorValue::CurrentColor,
            text_decoration_style: TextDecorationStyleValue::Solid,
            text_decoration_thickness: TextDecorationThicknessValue::Auto,
            text_decoration_inset: zero_css_parser::values::TextDecorationInsetValue {
                start: LengthValue::Px(0.0),
                end: LengthValue::Px(0.0),
            },
            text_emphasis_style: TextEmphasisStyleValue::None,
            text_emphasis_position: TextEmphasisPositionValue::OverRight,
            text_transform: TextTransformValue::None,
            letter_spacing: LengthValue::Px(0.0),
            word_spacing: LengthValue::Px(0.0),
            white_space: WhiteSpaceValue::Normal,
            text_overflow: TextOverflowValue::Clip,
            vertical_align: VerticalAlignValue::Baseline,
            word_break: WordBreakValue::Normal,
            text_autospace: TextAutospaceValue::NoAutospace,
            line_break: LineBreakValue::Auto,
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
            align_self: AlignmentValue::Auto,
            justify_items: JustifyItemsValue::Normal,
            justify_self: JustifySelfValue::Auto,
            align_content: AlignContentValue::Normal,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: FlexBasisValue::Auto,
            gap: LengthValue::Px(0.0),
            row_gap: LengthValue::Px(0.0),
            column_gap: LengthValue::Auto, // R1040: column-gap 初始值 normal（multicol=1em, flex/grid=0）
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
            // CSS 2.1 §9.3.2: top/right/bottom/left 初始值为 auto
            top: auto_length.clone(),
            right: auto_length.clone(),
            bottom: auto_length.clone(),
            left: auto_length,
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
            column_fill: ColumnFillComputedValue::Balance,
            column_span: ColumnSpanComputedValue::None,

            // Object Fit / Filter
            object_fit: ObjectFitComputedValue::Fill,
            filter: FilterComputedValue::None,
            backdrop_filter: FilterComputedValue::None,

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
            background_image: vec![],
            background_position: BackgroundPositionComputedValue::TwoValue(
                Box::new(BackgroundPositionComputedValue::Percent(0.0)),
                Box::new(BackgroundPositionComputedValue::Percent(0.0)),
            ),
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
            clip_path: ClipPathComputedValue::None,
            clip: ClipRectComputedValue::Auto,
            mask_image: vec![],
            mask_mode: MaskModeComputedValue::MatchSource,
            before_pseudo: None,
            after_pseudo: None,
        }
    }
}
