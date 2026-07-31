//! 计算样式结构体定义。
//!
//! `ComputedStyle` 包含所有 Tier 1 CSS 属性的 typed 字段。

use super::types::*;

/// 计算样式结构体，包含所有 Tier 1 CSS 属性。
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    // ── 盒模型 ──
    /// display 属性。
    pub display: DisplayValue,
    /// position 属性。
    pub position: PositionValue,
    /// float 属性。
    pub float: zero_css_parser::values::FloatValue,
    /// clear 属性。
    pub clear: zero_css_parser::values::ClearValue,
    /// list-style-type 属性。
    pub list_style_type: zero_css_parser::values::ListStyleTypeValue,
    /// list-style-position 属性。
    pub list_style_position: zero_css_parser::values::ListStylePositionValue,
    /// list-style-image 属性。
    pub list_style_image: ListStyleImageComputedValue,
    /// writing-mode 属性。
    pub writing_mode: WritingModeValue,
    /// width 属性。
    pub width: LengthValue,
    /// height 属性。
    pub height: LengthValue,
    /// min-width 属性。
    pub min_width: LengthValue,
    /// min-height 属性。
    pub min_height: LengthValue,
    /// max-width 属性。
    pub max_width: LengthValue,
    /// max-height 属性。
    pub max_height: LengthValue,
    /// margin-top 属性。
    pub margin_top: LengthValue,
    /// margin-right 属性。
    pub margin_right: LengthValue,
    /// margin-bottom 属性。
    pub margin_bottom: LengthValue,
    /// margin-left 属性。
    pub margin_left: LengthValue,
    /// padding-top 属性。
    pub padding_top: LengthValue,
    /// padding-right 属性。
    pub padding_right: LengthValue,
    /// padding-bottom 属性。
    pub padding_bottom: LengthValue,
    /// padding-left 属性。
    pub padding_left: LengthValue,
    /// box-sizing 属性。
    pub box_sizing: BoxSizingValue,

    // ── 边框 ──
    /// border-top-width 属性。
    pub border_top_width: LengthValue,
    /// border-right-width 属性。
    pub border_right_width: LengthValue,
    /// border-bottom-width 属性。
    pub border_bottom_width: LengthValue,
    /// border-left-width 属性。
    pub border_left_width: LengthValue,
    /// border-top-color 属性。
    pub border_top_color: ColorValue,
    /// border-right-color 属性。
    pub border_right_color: ColorValue,
    /// border-bottom-color 属性。
    pub border_bottom_color: ColorValue,
    /// border-left-color 属性。
    pub border_left_color: ColorValue,
    /// border-top-style 属性。
    pub border_top_style: BorderStyleValue,
    /// border-right-style 属性。
    pub border_right_style: BorderStyleValue,
    /// border-bottom-style 属性。
    pub border_bottom_style: BorderStyleValue,
    /// border-left-style 属性。
    pub border_left_style: BorderStyleValue,
    /// border-top-left-radius 属性。
    pub border_top_left_radius: LengthValue,
    /// border-top-right-radius 属性。
    pub border_top_right_radius: LengthValue,
    /// border-bottom-right-radius 属性。
    pub border_bottom_right_radius: LengthValue,
    /// border-bottom-left-radius 属性。
    pub border_bottom_left_radius: LengthValue,

    // ── Outline ──
    /// outline-width 属性。
    pub outline_width: LengthValue,
    /// outline-style 属性。
    pub outline_style: OutlineStyleValue,
    /// outline-color 属性。
    pub outline_color: ColorValue,
    /// outline-offset 属性。
    pub outline_offset: LengthValue,
    /// outline-offset: inset（CSS-UI-4 §4.4）关键字标记——为真时 outline 绘制在
    /// border-box 内侧，等价于负 outline-width 的偏移（painter 计算 offset=-outline_width）。
    pub outline_offset_inset: bool,

    // ── 颜色和背景 ──
    /// color 属性（前景色）。
    pub color: ColorValue,
    /// background-color 属性。
    pub background_color: ColorValue,
    /// color-scheme 属性的暗 scheme 标志（`color-scheme: dark` → true）。
    /// 影响 `light-dark(L, D)` 解析：dark 取第二参。继承属性。
    pub color_scheme_dark: bool,
    /// opacity 属性。
    pub opacity: f64,
    /// visibility 属性。
    pub visibility: VisibilityValue,
    /// content-visibility 属性（CSS Containment 2）。非继承。
    pub content_visibility: ContentVisibilityValue,

    // ── 字体 ──
    /// font-family 属性。
    pub font_family: Vec<String>,
    /// font-size 属性。
    pub font_size: LengthValue,
    /// font-weight 属性。
    pub font_weight: FontWeightValue,
    /// font-style 属性。
    pub font_style: FontStyleValue,
    /// line-height 属性。
    pub line_height: LineHeightValue,
    /// font-size-adjust 属性（Slice 1 R1191：parse+store+inherit dormant，未 apply）。
    pub font_size_adjust: FontSizeAdjustValue,

    // ── 文本 ──
    /// text-align 属性。
    pub text_align: TextAlignValue,
    /// text-decoration 属性。
    pub text_decoration: TextDecorationValue,
    /// text-decoration-line 属性。
    pub text_decoration_line: TextDecorationLineValue,
    /// text-decoration-color 属性（不继承）。
    pub text_decoration_color: ColorValue,
    /// text-decoration-style 属性（不继承）。
    pub text_decoration_style: TextDecorationStyleValue,
    /// text-decoration-thickness 属性（CSS Text Decoration 4 §2.3，不继承）。R1402。
    pub text_decoration_thickness: TextDecorationThicknessValue,
    /// text-decoration-inset 属性（CSS Text Decoration 4 §2.4，不继承）。R1607。
    /// 装饰线 inline 轴内缩（负值=延伸）；em 在 paint 期按 font_size 解析。
    pub text_decoration_inset: zero_css_parser::values::TextDecorationInsetValue,
    /// text-emphasis-style 属性（CSS Text Decoration 3 §3.1，继承）。
    pub text_emphasis_style: TextEmphasisStyleValue,
    /// text-emphasis-position 属性（§3.2，继承）。默认 OverRight。
    pub text_emphasis_position: TextEmphasisPositionValue,
    /// text-transform 属性。
    pub text_transform: TextTransformValue,
    /// letter-spacing 属性。
    pub letter_spacing: LengthValue,
    /// word-spacing 属性。
    pub word_spacing: LengthValue,
    /// white-space 属性。
    pub white_space: WhiteSpaceValue,
    /// text-overflow 属性。
    pub text_overflow: TextOverflowValue,
    /// vertical-align 属性。
    pub vertical_align: VerticalAlignValue,
    /// word-break 属性。
    pub word_break: WordBreakValue,
    /// text-autospace 属性（CSS Text 4 §8，表意文字与字母/数字间 0.125em 间距）。
    pub text_autospace: TextAutospaceValue,
    /// line-break 属性（CSS Text 3 §5.3，CJK 换行严格度）。
    pub line_break: LineBreakValue,
    /// text-indent 属性。
    pub text_indent: LengthValue,
    /// resize 属性。
    pub resize: ResizeValue,
    /// margin-trim 属性（css-box-4 §margin-trim）。
    pub margin_trim: MarginTrimValue,

    // ── 表格 ──
    /// table-layout 属性。
    pub table_layout: TableLayoutValue,
    /// caption-side 属性。
    pub caption_side: CaptionSideValue,
    /// border-collapse 属性。
    pub border_collapse: BorderCollapseValue,
    /// empty-cells 属性。
    pub empty_cells: EmptyCellsComputedValue,
    /// border-spacing 属性。
    pub border_spacing: BorderSpacingComputedValue,

    // ── Flexbox ──
    /// flex-direction 属性。
    pub flex_direction: FlexDirectionValue,
    /// flex-wrap 属性。
    pub flex_wrap: FlexWrapValue,
    /// justify-content 属性。
    pub justify_content: AlignmentValue,
    /// align-items 属性。
    pub align_items: AlignmentValue,
    /// align-self 属性。
    pub align_self: AlignmentValue,
    /// justify-items 属性。
    pub justify_items: JustifyItemsValue,
    /// justify-self 属性。
    pub justify_self: JustifySelfValue,
    /// align-content 属性。
    pub align_content: AlignContentValue,
    /// flex-grow 属性。
    pub flex_grow: f64,
    /// flex-shrink 属性。
    pub flex_shrink: f64,
    /// flex-basis 属性。
    pub flex_basis: FlexBasisValue,
    /// gap 属性。
    pub gap: LengthValue,
    /// row-gap 属性。
    pub row_gap: LengthValue,
    /// column-gap 属性。
    pub column_gap: LengthValue,
    /// order 属性。
    pub order: i32,

    // ── Grid ──
    /// grid-template-columns 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_template_columns: Option<String>,
    /// grid-template-rows 属性。
    pub grid_template_rows: Option<String>,
    /// grid-auto-flow 属性。
    pub grid_auto_flow: GridAutoFlowValue,
    /// grid-column-start 属性。
    pub grid_column_start: GridLineValue,
    /// grid-column-end 属性。
    pub grid_column_end: GridLineValue,
    /// grid-row-start 属性。
    pub grid_row_start: GridLineValue,
    /// grid-row-end 属性。
    pub grid_row_end: GridLineValue,
    /// grid-auto-rows 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_auto_rows: Option<String>,
    /// grid-auto-columns 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_auto_columns: Option<String>,
    /// grid-template-areas 属性。
    /// 存储 CSS 原始值字符串（如 '"header header" "sidebar main"'），
    /// 在布局转换时解析为区域映射。
    pub grid_template_areas: Option<String>,

    // ── 定位 ──
    /// top 属性。
    pub top: LengthValue,
    /// right 属性。
    pub right: LengthValue,
    /// bottom 属性。
    pub bottom: LengthValue,
    /// left 属性。
    pub left: LengthValue,
    /// z-index 属性。
    pub z_index: ZIndexValue,

    // ── Overflow ──
    /// overflow-x 属性。
    pub overflow_x: OverflowValue,
    /// overflow-y 属性。
    pub overflow_y: OverflowValue,

    // ── Aspect Ratio ──
    /// aspect-ratio 属性（width / height 比值），None 表示 auto。
    pub aspect_ratio: Option<f32>,

    // ── Cursor ──
    /// cursor 属性。
    pub cursor: CursorValue,

    // ── Transforms ──
    /// transform 属性。
    pub transform: zero_css_parser::values::TransformValue,
    /// transform-origin X 分量。
    pub transform_origin_x: LengthValue,
    /// transform-origin Y 分量。
    pub transform_origin_y: LengthValue,
    /// perspective 属性。
    pub perspective: LengthValue,
    /// perspective-origin X 分量。
    pub perspective_origin_x: LengthValue,
    /// perspective-origin Y 分量。
    pub perspective_origin_y: LengthValue,
    /// transform-style 属性。
    pub transform_style: TransformStyleValue,
    /// backface-visibility 属性。
    pub backface_visibility: BackfaceVisibilityValue,

    // ── Transitions ──
    /// transition-property 属性（逗号分隔的属性名列表）。
    pub transition_property: Vec<String>,
    /// transition-duration 属性（逗号分隔的秒数列表）。
    pub transition_duration: Vec<f64>,
    /// transition-timing-function 属性（逗号分隔的时间函数列表）。
    pub transition_timing_function: Vec<zero_css_parser::values::TimingFunctionValue>,
    /// transition-delay 属性（逗号分隔的秒数列表）。
    pub transition_delay: Vec<f64>,

    // ── Animations ──
    /// animation-name 属性（逗号分隔的动画名列表）。
    pub animation_name: Vec<String>,
    /// animation-duration 属性（逗号分隔的秒数列表）。
    pub animation_duration: Vec<f64>,
    /// animation-timing-function 属性（逗号分隔的时间函数列表）。
    pub animation_timing_function: Vec<zero_css_parser::values::TimingFunctionValue>,
    /// animation-delay 属性（逗号分隔的秒数列表）。
    pub animation_delay: Vec<f64>,
    /// animation-iteration-count 属性（逗号分隔的迭代次数列表）。
    /// None 表示 infinite。
    pub animation_iteration_count: Vec<Option<f64>>,
    /// animation-direction 属性（逗号分隔的方向列表）。
    pub animation_direction: Vec<zero_css_parser::values::AnimationDirectionValue>,
    /// animation-fill-mode 属性（逗号分隔的填充模式列表）。
    pub animation_fill_mode: Vec<zero_css_parser::values::AnimationFillModeValue>,
    /// animation-play-state 属性（逗号分隔的播放状态列表）。
    pub animation_play_state: Vec<zero_css_parser::values::AnimationPlayStateValue>,

    // ── Scroll Snap ──
    /// scroll-snap-type 属性。
    pub scroll_snap_type: ScrollSnapType,
    /// scroll-snap-align 属性。
    pub scroll_snap_align: ScrollSnapAlign,
    /// scroll-snap-stop 属性。
    pub scroll_snap_stop: ScrollSnapStop,
    /// scroll-margin-top 属性（px）。
    pub scroll_margin_top: f32,
    /// scroll-margin-right 属性（px）。
    pub scroll_margin_right: f32,
    /// scroll-margin-bottom 属性（px）。
    pub scroll_margin_bottom: f32,
    /// scroll-margin-left 属性（px）。
    pub scroll_margin_left: f32,
    /// scroll-padding-top 属性。
    pub scroll_padding_top: ScrollPadding,
    /// scroll-padding-right 属性。
    pub scroll_padding_right: ScrollPadding,
    /// scroll-padding-bottom 属性。
    pub scroll_padding_bottom: ScrollPadding,
    /// scroll-padding-left 属性。
    pub scroll_padding_left: ScrollPadding,

    // ── Container Query ──
    /// container-type 属性。
    pub container_type: ContainerType,
    /// container-name 属性。
    pub container_name: Option<String>,

    // ── Counters / Content / Quotes ──
    /// counter-reset 属性。
    pub counter_reset: Vec<CounterActionValue>,
    /// counter-increment 属性。
    pub counter_increment: Vec<CounterActionValue>,
    /// counter-set 属性。
    pub counter_set: Vec<CounterActionValue>,
    /// content 属性。
    pub content: ContentComputedValue,
    /// quotes 属性。
    pub quotes: QuotesComputedValue,

    // ── Page Break ──
    /// page-break-before 属性。
    pub page_break_before: PageBreakValue,
    /// page-break-after 属性。
    pub page_break_after: PageBreakValue,
    /// page-break-inside 属性。
    pub page_break_inside: PageBreakValue,

    // ── 其他 ──
    /// box-decoration-break 属性。
    pub box_decoration_break: BoxDecorationBreakValue,
    /// image-rendering 属性。
    pub image_rendering: ImageRenderingValue,
    /// isolation 属性。
    pub isolation: IsolationValue,

    // ── Break ──
    /// break-inside 属性。
    pub break_inside: BreakInsideValue,
    /// break-before 属性。
    pub break_before: BreakValue,
    /// break-after 属性。
    pub break_after: BreakValue,

    // ── Column Rule ──
    /// column-rule-width 属性。
    pub column_rule_width: ColumnRuleWidthComputedValue,
    /// column-rule-style 属性。
    pub column_rule_style: ColumnRuleStyleComputedValue,
    /// column-rule-color 属性。
    pub column_rule_color: ColorValue,

    // ── Contain ──
    /// contain 属性。
    pub contain: ContainComputedValue,
    /// contain-intrinsic-size 宽度分量（CSS Sizing 4）。仅对 size-containment 元素生效
    ///（contain:size / content-visibility:hidden）。None = 不覆盖（取 size containment 的 0）。
    pub contain_intrinsic_width: Option<LengthValue>,
    /// contain-intrinsic-size 高度分量。语义同 contain_intrinsic_width。
    pub contain_intrinsic_height: Option<LengthValue>,

    // ── Interaction / Performance Hint ──
    /// overscroll-behavior-x 属性。
    pub overscroll_behavior_x: OverscrollBehaviorValue,
    /// overscroll-behavior-y 属性。
    pub overscroll_behavior_y: OverscrollBehaviorValue,
    /// touch-action 属性。
    pub touch_action: TouchActionValue,
    /// user-select 属性。
    pub user_select: UserSelectValue,
    /// will-change 属性。
    pub will_change: Vec<WillChangeValue>,
    /// pointer-events 属性。
    pub pointer_events: PointerEventsValue,

    // ── Text (新属性) ──
    /// overflow-wrap 属性。
    pub overflow_wrap: OverflowWrapValue,
    /// text-align-last 属性。
    pub text_align_last: TextAlignLastValue,
    /// font-variant-numeric 属性。
    pub font_variant_numeric: FontVariantNumericValue,

    // ── Writing Direction / Tab ──
    /// direction 属性。
    pub direction: DirectionValue,
    /// unicode-bidi 属性。
    pub unicode_bidi: UnicodeBidiValue,
    /// tab-size 属性。
    pub tab_size: TabSizeValue,

    // ── Columns ──
    /// column-count 属性。
    pub column_count: ColumnCountComputedValue,
    /// column-width 属性。
    pub column_width: ColumnWidthComputedValue,
    /// column-fill 属性（balance 或 auto）。
    pub column_fill: ColumnFillComputedValue,
    /// column-span 属性（none 或 all）。`all` 使元素成为 spanner 跨越全宽。
    pub column_span: ColumnSpanComputedValue,

    // ── Object Fit / Filter ──
    /// object-fit 属性。
    pub object_fit: ObjectFitComputedValue,
    /// object-position 属性（CSS Images §3）：替换元素内容在盒内的对齐位置（默认 Center=50% 50%）。
    pub object_position: BackgroundPositionComputedValue,
    /// filter 属性（CSS Filter Effects：`none | <filter-function>+`，多函数列表；空 Vec = none）。
    pub filter: Vec<FilterComputedValue>,
    /// backdrop-filter 属性（对元素背后内容应用滤镜；多函数列表；空 Vec = none）。
    pub backdrop_filter: Vec<FilterComputedValue>,

    // ── UI Appearance ──
    /// appearance 属性。
    pub appearance: AppearanceComputedValue,
    /// accent-color 属性。
    pub accent_color: AccentColorComputedValue,
    /// caret-color 属性。
    pub caret_color: CaretColorComputedValue,
    /// mix-blend-mode 属性。
    pub mix_blend_mode: MixBlendModeComputedValue,
    /// scrollbar-width 属性。
    pub scrollbar_width: ScrollbarWidthComputedValue,
    /// scrollbar-gutter 属性。
    pub scrollbar_gutter: ScrollbarGutterComputedValue,

    // ── Text Wrap / Hyphens / Line Clamp ──
    /// text-wrap 属性。
    pub text_wrap: TextWrapComputedValue,
    /// hyphens 属性。
    pub hyphens: HyphensComputedValue,
    /// line-clamp 属性。
    pub line_clamp: LineClampComputedValue,

    // ── Background Image / Position / Repeat / Size / Attachment ──
    /// background-image 属性（支持多图层，CSS 规范渲染顺序为逆序）。
    pub background_image: Vec<BackgroundImageComputedValue>,
    /// background-position 属性。
    pub background_position: BackgroundPositionComputedValue,
    /// background-repeat 属性。
    pub background_repeat: BackgroundRepeatComputedValue,
    /// background-size 属性。
    pub background_size: BackgroundSizeComputedValue,
    /// background-attachment 属性。
    pub background_attachment: BackgroundAttachmentComputedValue,
    /// background-clip 属性。
    pub background_clip: BackgroundClipComputedValue,
    /// background-origin 属性。
    pub background_origin: BackgroundOriginComputedValue,
    /// border-image-source 属性。
    pub border_image_source: BorderImageSourceComputedValue,
    /// border-image-slice 属性。
    pub border_image_slice: BorderImageSliceComputedValue,
    /// border-image-width 属性。
    pub border_image_width: BorderImageWidthComputedValue,
    /// border-image-repeat 属性。
    pub border_image_repeat: BorderImageRepeatComputedValue,
    /// border-image-outset 属性。
    pub border_image_outset: BorderImageOutsetComputedValue,
    /// text-shadow 属性（CSS Text Decoration §3：`none | <shadow>#`，多阴影列表；空 Vec = none）。
    pub text_shadow: Vec<TextShadowComputedValue>,
    /// box-shadow 属性（CSS Backgrounds §7.2：<shadow>#，多阴影列表；空 Vec = none）。
    pub box_shadow: Vec<BoxShadowComputedValue>,
    /// clip-path 属性。
    pub clip_path: ClipPathComputedValue,
    /// clip 属性（已弃用的 CSS2 裁剪属性，仅对绝对定位元素生效）。
    pub clip: ClipRectComputedValue,
    /// mask-image 属性（支持多图层，格式与 background-image 相同）。
    pub mask_image: Vec<BackgroundImageComputedValue>,
    /// mask-mode 属性（alpha/luminance/match-source）。
    pub mask_mode: MaskModeComputedValue,
    /// `::before` 伪元素的计算样式（无匹配规则时为 None；由 style-system 计算，
    /// layout 据此在元素内容前合成生成盒）。伪元素不参与继承传播（继承构造全新 default）。
    pub before_pseudo: Option<Box<ComputedStyle>>,
    /// `::after` 伪元素的计算样式（语义同 `before_pseudo`，合成在元素内容后）。
    pub after_pseudo: Option<Box<ComputedStyle>>,
}

impl ComputedStyle {
    /// R2251：`content-visibility: hidden` 是否产生「跳过内容」视觉效果（CSS Containment 2）。
    ///
    /// `content-visibility: hidden` 通过隐式 `contain: size layout paint` 起效，而 size
    /// containment **不适用于**无主盒（`display: none` / `display: contents`）与非替换 inline
    /// 盒（"non-atomic inline"）。在这些情况下 `content-visibility: hidden` 无视觉效果——
    /// 内容正常渲染。WPT 量证：`content-visibility-on-display-contents`（display:contents +
    /// CV:hidden → 绿块可见）、`content-visibility-on-ruby`（`<ruby>` inline + CV:hidden →
    /// base/annotation 可见）、`content-visibility-073`（meta assert「no effect on non-atomic
    /// inlines」）。ZW 无 ruby display（`<ruby>` 作 inline 处理），内部 table 盒边角未在语料
    /// 触发，此处不排除。
    ///
    /// **kill-switch 由调用方检查**（env `ZW_CONTENT_VISIBILITY`，default-on）。
    pub fn content_visibility_hidden_effective(&self) -> bool {
        matches!(self.content_visibility, ContentVisibilityValue::Hidden)
            && !matches!(
                self.display,
                DisplayValue::None | DisplayValue::Contents | DisplayValue::Inline
            )
    }
}
