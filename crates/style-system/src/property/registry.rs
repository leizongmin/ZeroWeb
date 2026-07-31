//! 属性注册表，提供初始值和继承性查询。

use super::types::*;

/// 属性注册表，提供初始值和继承性查询。
///
/// 使用 match 语句实现属性名到初始值和继承性的映射。
pub struct PropertyRegistry;

impl PropertyRegistry {
    /// 获取指定属性的初始值。
    ///
    /// 返回 `None` 表示未知属性。
    pub fn initial_value(property: &str) -> Option<PropertyValue> {
        use PropertyValue::*;
        match property {
            // 盒模型
            "display" => Some(Display(DisplayValue::Inline)),
            "position" => Some(Position(PositionValue::Static)),
            "float" => Some(Float(zero_css_parser::values::FloatValue::None)),
            "clear" => Some(Clear(zero_css_parser::values::ClearValue::None)),
            "list-style-type" => Some(ListStyleType(zero_css_parser::values::ListStyleTypeValue::Disc)),
            "list-style-position" => Some(ListStylePosition(
                zero_css_parser::values::ListStylePositionValue::Outside,
            )),
            "list-style-image" => Some(ListStyleImage(ListStyleImageComputedValue::None)),
            "width" | "height" => Some(Length(LengthValue::Px(0.0))),
            "min-width" | "min-height" => Some(Length(LengthValue::Px(0.0))),
            "max-width" | "max-height" => Some(Length(LengthValue::Px(f64::INFINITY))),
            "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => Some(Length(LengthValue::Px(0.0))),
            "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => Some(Length(LengthValue::Px(0.0))),
            "box-sizing" => Some(BoxSizing(BoxSizingValue::ContentBox)),

            // 边框 — border-width 初始值 = medium（CSS §8.5.1），ZeroWeb 取 3px。
            "border-top-width" | "border-right-width" | "border-bottom-width" | "border-left-width" => {
                Some(Length(LengthValue::Px(3.0)))
            }
            // border-color 初始值 = currentColor（CSS §8.5.1），paint 时解析为元素计算 color。
            "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
                Some(Color(ColorValue::CurrentColor))
            }
            "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
                Some(BorderStyle(BorderStyleValue::None))
            }
            "border-top-left-radius"
            | "border-top-right-radius"
            | "border-bottom-right-radius"
            | "border-bottom-left-radius" => Some(Length(LengthValue::Px(0.0))),

            // Outline — outline-width 初始 = medium(3px)（CSS UI，与 border-width 同）；
            // outline-offset 初始 = 0。outline-style 初始 none 抑制绘制。
            "outline-width" => Some(Length(LengthValue::Px(3.0))),
            "outline-offset" => Some(Length(LengthValue::Px(0.0))),
            "outline-style" => Some(OutlineStyle(OutlineStyleValue::None)),
            "outline-color" => Some(Color(ColorValue::Rgba(0, 0, 0, 255))),

            // 颜色和背景
            "color" => Some(Color(ColorValue::Rgba(0, 0, 0, 255))),
            "background-color" => Some(Color(ColorValue::Transparent)),
            // color-scheme 初始 = normal → light（dark=false）。CSS Color Adjust。
            "color-scheme" => Some(ColorScheme(false)),
            "opacity" => Some(Number(1.0)),
            "visibility" => Some(Visibility(VisibilityValue::Visible)),
            "content-visibility" => Some(ContentVisibility(ContentVisibilityValue::Visible)),

            // 字体
            "font-family" => Some(StringList(vec![])),
            "font-size" => Some(Length(LengthValue::Px(16.0))),
            "font-weight" => Some(FontWeight(FontWeightValue::Normal)),
            "font-style" => Some(FontStyle(FontStyleValue::Normal)),
            "line-height" => Some(LineHeight(LineHeightValue::Normal)),
            "font-size-adjust" => Some(FontSizeAdjust(FontSizeAdjustValue::None)),

            // 文本
            "text-align" => Some(TextAlign(TextAlignValue::Start)),
            "text-decoration" => Some(TextDecoration(TextDecorationValue::None)),
            "text-decoration-line" => Some(TextDecorationLine(TextDecorationLineValue::None)),
            "text-decoration-color" => Some(TextDecorationColor(ColorValue::CurrentColor)),
            "text-decoration-style" => Some(TextDecorationStyle(TextDecorationStyleValue::Solid)),
            "text-decoration-thickness" => Some(TextDecorationThickness(TextDecorationThicknessValue::Auto)),
            "text-decoration-inset" => Some(TextDecorationInset(zero_css_parser::values::TextDecorationInsetValue {
                start: LengthValue::Px(0.0),
                end: LengthValue::Px(0.0),
            })),
            // CSS Text Decoration 3 §3.1/§3.2：emphasis-style 与 position 均继承。
            "text-emphasis-style" => Some(TextEmphasisStyle(TextEmphasisStyleValue::None)),
            "text-emphasis-position" => Some(TextEmphasisPosition(TextEmphasisPositionValue::OverRight)),
            // text-emphasis-color 未单独实现（用 text_decoration_color 近似）；shorthand 待补
            "text-transform" => Some(TextTransform(TextTransformValue::None)),
            "letter-spacing" | "word-spacing" => Some(Length(LengthValue::Px(0.0))),
            "white-space" => Some(WhiteSpace(WhiteSpaceValue::Normal)),
            "text-overflow" => Some(TextOverflow(TextOverflowValue::Clip)),
            "vertical-align" => Some(VerticalAlign(VerticalAlignValue::Baseline)),
            "word-break" => Some(WordBreak(WordBreakValue::Normal)),
            "text-autospace" => Some(TextAutospace(TextAutospaceValue::NoAutospace)),
            "text-indent" => Some(TextIndent(LengthValue::Px(0.0))),
            "table-layout" => Some(TableLayout(TableLayoutValue::Auto)),
            "caption-side" => Some(CaptionSide(CaptionSideValue::Top)),
            "border-collapse" => Some(BorderCollapse(BorderCollapseValue::Separate)),
            "empty-cells" => Some(EmptyCells(EmptyCellsComputedValue::Show)),
            "border-spacing" => Some(BorderSpacing(BorderSpacingComputedValue {
                horizontal: 0.0,
                vertical: 0.0,
            })),
            "resize" => Some(Resize(ResizeValue::None)),

            // Writing Mode
            "writing-mode" => Some(WritingMode(WritingModeValue::HorizontalTb)),

            // Flexbox
            "flex-direction" => Some(FlexDirection(FlexDirectionValue::Row)),
            "flex-wrap" => Some(FlexWrap(FlexWrapValue::Nowrap)),
            "justify-content" => Some(Alignment(AlignmentValue::FlexStart)),
            "align-items" => Some(Alignment(AlignmentValue::Stretch)),
            "align-self" => Some(Alignment(AlignmentValue::Auto)),
            "justify-items" => Some(JustifyItems(JustifyItemsValue::Normal)),
            "justify-self" => Some(JustifySelf(JustifySelfValue::Auto)),
            "align-content" => Some(AlignContent(AlignContentValue::Normal)),
            "flex-grow" => Some(Number(0.0)),
            "flex-shrink" => Some(Number(1.0)),
            "flex-basis" => Some(FlexBasis(FlexBasisValue::Auto)),
            "gap" => Some(Length(LengthValue::Px(0.0))),
            "column-gap" => Some(Length(LengthValue::Px(0.0))),
            "row-gap" => Some(Length(LengthValue::Px(0.0))),
            "order" => Some(Integer(0)),

            // 定位
            // CSS 2.1 §9.3.2: top/right/bottom/left 的初始值为 auto，不是 0px
            "top" | "right" | "bottom" | "left" => Some(Length(LengthValue::Auto)),
            "z-index" => Some(ZIndex(ZIndexValue::Auto)),

            // Overflow
            "overflow-x" | "overflow-y" => Some(Overflow(OverflowValue::Visible)),

            // Aspect Ratio
            "aspect-ratio" => Some(Number(f64::NAN)), // NaN 表示 auto

            // Cursor
            "cursor" => Some(Cursor(CursorValue::Auto)),

            // Transitions
            "transition-property" => Some(StringList(vec![])),
            "transition-duration" | "transition-delay" => Some(Number(0.0)),
            "transition-timing-function" => Some(TimingFunctionList(vec![])),

            // Animations
            "animation-name" => Some(StringList(vec![])),
            "animation-duration" | "animation-delay" => Some(Number(0.0)),
            "animation-timing-function" => Some(TimingFunctionList(vec![])),
            "animation-iteration-count" => Some(OptionalNumberList(vec![])),
            "animation-direction" => Some(AnimationDirectionList(vec![])),
            "animation-fill-mode" => Some(AnimationFillModeList(vec![])),
            "animation-play-state" => Some(AnimationPlayStateList(vec![])),

            // Grid
            "grid-template-columns" | "grid-template-rows" => Some(OptionalString(None)),
            "grid-auto-flow" => Some(GridAutoFlow(GridAutoFlowValue::Row)),
            "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
                Some(GridLine(GridLineValue::Auto))
            }
            "grid-auto-rows" | "grid-auto-columns" => Some(OptionalString(None)),
            "grid-template-areas" => Some(OptionalString(None)),

            // Transform
            "transform" => Some(Transform(zero_css_parser::values::TransformValue::None)),
            "transform-origin" => Some(Length(LengthValue::Percentage(50.0))),
            "perspective" => Some(Length(LengthValue::Px(0.0))),
            "perspective-origin" => Some(Length(LengthValue::Percentage(50.0))),
            "transform-style" => Some(TransformStyle(TransformStyleValue::Flat)),
            "backface-visibility" => Some(BackfaceVisibility(BackfaceVisibilityValue::Visible)),

            // Scroll Snap
            "scroll-snap-type" => {
                let default_sst = crate::property::ScrollSnapType {
                    strictness: crate::property::ScrollSnapStrictness::None,
                    axis: zero_css_parser::values::ScrollSnapAxis::Both,
                };
                Some(ScrollSnapType(default_sst))
            }
            "scroll-snap-align" => Some(ScrollSnapAlign(crate::property::ScrollSnapAlign::None)),
            "scroll-snap-stop" => Some(ScrollSnapStop(crate::property::ScrollSnapStop::Normal)),
            "scroll-margin-top" | "scroll-margin-right" | "scroll-margin-bottom" | "scroll-margin-left" => {
                Some(Number(0.0))
            }
            "scroll-padding-top" | "scroll-padding-right" | "scroll-padding-bottom" | "scroll-padding-left" => {
                Some(ScrollPadding(crate::property::ScrollPadding::Auto))
            }

            // Container Query
            "container-type" => Some(ContainerType(crate::property::ContainerType::Normal)),
            "container-name" => Some(ContainerName(None)),

            // Counters / Content / Quotes
            "counter-reset" => Some(CounterReset(vec![])),
            "counter-increment" => Some(CounterIncrement(vec![])),
            "counter-set" => Some(CounterSet(vec![])),
            "content" => Some(Content(ContentComputedValue::Normal)),
            "quotes" => Some(Quotes(QuotesComputedValue::Auto)),

            // Page Break
            "page-break-before" | "page-break-after" | "page-break-inside" => Some(PageBreak(PageBreakValue::Auto)),

            // 其他
            "box-decoration-break" => Some(BoxDecorationBreak(BoxDecorationBreakValue::Slice)),
            "image-rendering" => Some(ImageRendering(ImageRenderingValue::Auto)),
            "isolation" => Some(Isolation(IsolationValue::Auto)),

            // Break
            "break-inside" => Some(BreakInside(BreakInsideValue::Auto)),
            "break-before" => Some(BreakBefore(BreakValue::Auto)),
            "break-after" => Some(BreakAfter(BreakValue::Auto)),

            // Column Rule
            "column-rule-width" => Some(ColumnRuleWidth(ColumnRuleWidthComputedValue::Medium)),
            "column-rule-style" => Some(ColumnRuleStyle(ColumnRuleStyleComputedValue::None)),

            // Interaction / Performance Hint
            "overscroll-behavior-x" | "overscroll-behavior-y" => {
                Some(OverscrollBehavior(OverscrollBehaviorValue::Auto))
            }
            "touch-action" => Some(TouchAction(TouchActionValue::Auto)),
            "user-select" => Some(UserSelect(UserSelectValue::Auto)),
            "will-change" => Some(WillChange(Vec::new())),
            "pointer-events" => Some(PointerEvents(PointerEventsValue::Auto)),

            // Text (新属性)
            "overflow-wrap" => Some(OverflowWrap(OverflowWrapValue::Normal)),
            "text-align-last" => Some(TextAlignLast(TextAlignLastValue::Auto)),
            "font-variant-numeric" => Some(FontVariantNumeric(FontVariantNumericValue::Normal)),

            // Writing Direction / Tab
            "direction" => Some(Direction(DirectionValue::Ltr)),
            "unicode-bidi" => Some(UnicodeBidi(UnicodeBidiValue::Normal)),
            "tab-size" => Some(TabSize(TabSizeValue::Number(8))),

            // Columns
            "columns" => Some(ColumnCount(ColumnCountComputedValue::Auto)),
            "column-count" => Some(ColumnCount(ColumnCountComputedValue::Auto)),
            "column-width" => Some(ColumnWidth(ColumnWidthComputedValue::Auto)),
            "column-fill" => Some(ColumnFill(ColumnFillComputedValue::Balance)),
            "column-span" => Some(ColumnSpan(ColumnSpanComputedValue::None)),

            // Object Fit / Filter
            "object-fit" => Some(ObjectFit(ObjectFitComputedValue::Fill)),
            // object-position 默认 50% 50%（Center），CSS Images §3。
            "object-position" => Some(ObjectPosition(BackgroundPositionComputedValue::Center)),
            "filter" => Some(Filter(Vec::new())),

            // Column Rule Color
            "column-rule-color" => Some(ColumnRuleColor(ColorValue::Rgba(0, 0, 0, 255))),

            // Contain
            "contain" => Some(Contain(ContainComputedValue::None)),
            "contain-intrinsic-size" | "contain-intrinsic-width" | "contain-intrinsic-height" => {
                Some(ContainIntrinsicSize(None, None))
            }

            // UI Appearance
            "appearance" => Some(Appearance(AppearanceComputedValue::Auto)),
            "accent-color" => Some(AccentColor(AccentColorComputedValue::Auto)),
            "caret-color" => Some(CaretColor(CaretColorComputedValue::Auto)),

            // Compositing / Scrolling
            "mix-blend-mode" => Some(MixBlendMode(MixBlendModeComputedValue::Normal)),
            "scrollbar-width" => Some(ScrollbarWidth(ScrollbarWidthComputedValue::Auto)),
            "scrollbar-gutter" => Some(ScrollbarGutter(ScrollbarGutterComputedValue::Auto)),

            // Text Wrap / Hyphens / Line Clamp
            "text-wrap" => Some(TextWrap(TextWrapComputedValue::Wrap)),
            "hyphens" => Some(Hyphens(HyphensComputedValue::None)),
            "line-clamp" => Some(LineClamp(LineClampComputedValue::None)),

            // Background Image / Position / Repeat / Size / Attachment / Clip / Origin
            "background-image" => Some(BackgroundImage(BackgroundImageComputedValue::None)),
            "background-position" => Some(BackgroundPosition(vec![BackgroundPositionComputedValue::TwoValue(
                Box::new(BackgroundPositionComputedValue::Percent(0.0)),
                Box::new(BackgroundPositionComputedValue::Percent(0.0)),
            )])),
            "background-repeat" => Some(BackgroundRepeat(vec![BackgroundRepeatComputedValue::Repeat])),
            "background-size" => Some(BackgroundSize(vec![BackgroundSizeComputedValue::Auto])),
            "background-attachment" => Some(BackgroundAttachment(BackgroundAttachmentComputedValue::Scroll)),
            "background-clip" => Some(BackgroundClip(BackgroundClipComputedValue::BorderBox)),
            "background-origin" => Some(BackgroundOrigin(BackgroundOriginComputedValue::PaddingBox)),

            // Border Image
            "border-image-source" => Some(BorderImageSource(BorderImageSourceComputedValue::None)),
            "border-image-slice" => Some(BorderImageSlice(BorderImageSliceComputedValue {
                top: BorderImageSliceComputedComponent::Number(100.0),
                right: BorderImageSliceComputedComponent::Number(100.0),
                bottom: BorderImageSliceComputedComponent::Number(100.0),
                left: BorderImageSliceComputedComponent::Number(100.0),
                fill: false,
            })),
            "border-image-width" => Some(BorderImageWidth(BorderImageWidthComputedValue {
                top: BorderImageWidthComputedComponent::Number(1.0),
                right: BorderImageWidthComputedComponent::Number(1.0),
                bottom: BorderImageWidthComputedComponent::Number(1.0),
                left: BorderImageWidthComputedComponent::Number(1.0),
            })),
            "border-image-repeat" => Some(BorderImageRepeat(BorderImageRepeatComputedValue {
                horizontal: BorderImageRepeatComputedMode::Stretch,
                vertical: BorderImageRepeatComputedMode::Stretch,
            })),
            "border-image-outset" => Some(BorderImageOutset(BorderImageOutsetComputedValue {
                top: BorderImageOutsetComputedComponent::Number(0.0),
                right: BorderImageOutsetComputedComponent::Number(0.0),
                bottom: BorderImageOutsetComputedComponent::Number(0.0),
                left: BorderImageOutsetComputedComponent::Number(0.0),
            })),
            "text-shadow" => Some(TextShadow(Vec::new())),
            "box-shadow" => Some(BoxShadow(Vec::new())),

            // Clip Path
            "clip-path" => Some(ClipPath(ClipPathComputedValue::None)),

            // Clip (deprecated CSS2)
            "clip" => Some(Clip(ClipRectComputedValue::Auto)),

            _ => None,
        }
    }

    /// 查询指定属性是否为继承属性。
    ///
    /// 继承属性在没有显式值时会从父元素继承。
    pub fn is_inherited(property: &str) -> bool {
        matches!(
            property,
            "color"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "font-size-adjust"
                | "text-align"
                | "text-transform"
                | "letter-spacing"
                | "word-spacing"
                | "white-space"
                | "word-break"
                | "text-autospace"
                | "visibility"
                | "cursor"
                | "text-indent"
                | "caption-side"
                | "border-collapse"
                | "quotes"
                | "pointer-events"
                | "overflow-wrap"
                | "text-align-last"
                | "font-variant-numeric"
                | "direction"
                | "tab-size"
                | "accent-color"
                | "caret-color"
                | "text-wrap"
                | "hyphens"
                | "text-shadow"
                | "list-style-image"
                | "list-style-type"
                | "list-style-position"
                | "empty-cells"
                | "border-spacing"
                | "writing-mode"
                | "color-scheme"
        )
    }

    /// 获取所有已知属性名的列表。
    pub fn known_properties() -> &'static [&'static str] {
        &[
            "display",
            "position",
            "float",
            "clear",
            "list-style-type",
            "list-style-position",
            "list-style-image",
            "width",
            "height",
            "min-width",
            "min-height",
            "max-width",
            "max-height",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
            "box-sizing",
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
            "color",
            "background-color",
            "color-scheme",
            "opacity",
            "visibility",
            "content-visibility",
            "font-family",
            "font-size",
            "font-weight",
            "font-style",
            "line-height",
            "font-size-adjust",
            "text-align",
            "text-decoration",
            "text-decoration-line",
            "text-decoration-color",
            "text-decoration-style",
            "text-decoration-thickness",
            "text-decoration-inset",
            "text-emphasis-style",
            "text-emphasis-position",
            "text-transform",
            "letter-spacing",
            "word-spacing",
            "white-space",
            "text-overflow",
            "vertical-align",
            "word-break",
            "text-autospace",
            "flex-direction",
            "flex-wrap",
            "justify-content",
            "align-items",
            "align-self",
            "justify-items",
            "justify-self",
            "align-content",
            "flex-grow",
            "flex-shrink",
            "flex-basis",
            "gap",
            "column-gap",
            "order",
            "top",
            "right",
            "bottom",
            "left",
            "z-index",
            "overflow-x",
            "overflow-y",
            "aspect-ratio",
            "cursor",
            "transition-property",
            "transition-duration",
            "transition-timing-function",
            "transition-delay",
            "animation-name",
            "animation-duration",
            "animation-timing-function",
            "animation-delay",
            "animation-iteration-count",
            "animation-direction",
            "animation-fill-mode",
            "animation-play-state",
            "grid-column-start",
            "grid-column-end",
            "grid-row-start",
            "grid-row-end",
            "grid-template-columns",
            "grid-template-rows",
            "grid-template-areas",
            "grid-auto-flow",
            "row-gap",
            "grid-auto-rows",
            "grid-auto-columns",
            "outline-width",
            "outline-style",
            "outline-color",
            "outline-offset",
            "transform",
            "scroll-snap-type",
            "scroll-snap-align",
            "scroll-snap-stop",
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
            "container-type",
            "container-name",
            "writing-mode",
            "text-indent",
            "table-layout",
            "caption-side",
            "border-collapse",
            "empty-cells",
            "border-spacing",
            "resize",
            "transform-origin",
            "perspective",
            "perspective-origin",
            "transform-style",
            "backface-visibility",
            "counter-reset",
            "counter-increment",
            "counter-set",
            "content",
            "quotes",
            "page-break-before",
            "page-break-after",
            "page-break-inside",
            "box-decoration-break",
            "image-rendering",
            "isolation",
            "break-inside",
            "break-before",
            "break-after",
            "column-rule-width",
            "column-rule-style",
            "overscroll-behavior-x",
            "overscroll-behavior-y",
            "touch-action",
            "user-select",
            "will-change",
            "pointer-events",
            "overflow-wrap",
            "text-align-last",
            "font-variant-numeric",
            "direction",
            "unicode-bidi",
            "tab-size",
            "columns",
            "column-count",
            "column-width",
            "column-fill",
            "column-span",
            "object-fit",
            "object-position",
            "filter",
            "column-rule-color",
            "contain",
            "contain-intrinsic-size",
            "contain-intrinsic-width",
            "contain-intrinsic-height",
            "appearance",
            "accent-color",
            "caret-color",
            "mix-blend-mode",
            "scrollbar-width",
            "scrollbar-gutter",
            "text-wrap",
            "hyphens",
            "line-clamp",
            "background-image",
            "background-position",
            "background-repeat",
            "background-size",
            "background-attachment",
            "background-clip",
            "background-origin",
            "border-image-source",
            "border-image-slice",
            "border-image-width",
            "border-image-repeat",
            "border-image-outset",
            "text-shadow",
            "box-shadow",
            "clip-path",
            "clip",
        ]
    }
}
