//! getComputedStyle 计算与序列化——把 [`zero_style_system::ComputedStyle`] 的单属性序列化为
//! CSS 字符串（kebab-case 属性名）。从 `js_dom_bridge` 拆出（R2709）以控制主文件行数。
//!
//! 覆盖：display/position/visibility/opacity + 颜色族（color/background-color/border-*-color/outline-color/caret-color/accent-color）+ 长度族 + 关键字/枚举族 + font-family/复合族
//! + Transforms 全簇 + contain + filter + will-change + clip-path + content + background 簇（position/size/repeat/attachment/clip/origin）+ Box Alignment 簇（align-items/self、justify-content/items/self、align-content）+ CSS Text 换行/断词（word-break/overflow-wrap/hyphens/line-break/text-wrap/text-align-last）+ vertical-align/unicode-bidi/empty-cells/resize/appearance + box-decoration-break/scrollbar-*/touch-action + outline-offset/break-before·after·inside + grid-auto-flow/container-type·name/tab-size + border-spacing/list-style-image/font-size-adjust + border-image-source/object-position/quotes；未覆盖属性返 ''。

use std::collections::HashMap;

use zero_css_parser::values::{
    AlignmentValue, AnimationDirectionValue, AnimationFillModeValue, AnimationPlayStateValue, BackgroundEdge,
    BoxSizingValue, ClearValue, ClipPathRadius, ClipPathValue, ColorHueMethod, ColorInterpolation,
    ColorInterpolationSpace, ColorValue, ConicGradient, ContentListItem, DisplayValue, FlexDirectionValue,
    FlexWrapValue, FloatValue, FontStyleValue, FontWeightValue, GradientColorStop, GradientDirection, LengthValue,
    ListStylePositionValue, ListStyleTypeValue, OverflowValue, PolygonFillRule, PositionValue, RadialGradient,
    RadialShape, RadialSize, StepPosition, TimingFunctionValue, TransformFunction, TransformValue, VisibilityValue,
};
use zero_dom::{Document, NodeId, parse_html};
use zero_style_system::{
    AccentColorComputedValue, AlignContentValue, AppearanceComputedValue, BackfaceVisibilityValue,
    BackgroundAttachmentComputedValue, BackgroundClipComputedValue, BackgroundImageComputedValue,
    BackgroundOriginComputedValue, BackgroundPositionComputedValue, BackgroundRepeatComputedValue,
    BackgroundSizeComputedValue, BgSizeComponentComputed, BorderCollapseValue, BorderImageOutsetComputedComponent,
    BorderImageOutsetComputedValue, BorderImageRepeatComputedMode, BorderImageRepeatComputedValue,
    BorderImageSliceComputedComponent, BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    BorderImageWidthComputedComponent, BorderImageWidthComputedValue, BorderSpacingComputedValue, BorderStyleValue,
    BoxDecorationBreakValue, BoxShadowComputedValue, BreakInsideValue, BreakValue, CaptionSideValue,
    CaretColorComputedValue, ColumnCountComputedValue, ColumnFillComputedValue, ColumnRuleStyleComputedValue,
    ColumnRuleWidthComputedValue, ColumnSpanComputedValue, ColumnWidthComputedValue, ComputedStyle,
    ContainComputedValue, ContainerType, ContentComputedValue, ContentVisibilityValue, CounterActionValue, CursorValue,
    DirectionValue, EmptyCellsComputedValue, FilterComputedValue, FlexBasisValue, FontKerningValue,
    FontSizeAdjustBasis, FontSizeAdjustMetric, FontSizeAdjustValue, FontVariantAlternatesValue,
    FontVariantNumericValue, FontVariationSettingsValue, GridAutoFlowValue, GridLineValue, HyphensComputedValue,
    ImageRenderingValue, IsolationValue, JustifyItemsValue, JustifySelfValue, LineBreakValue, LineHeightValue,
    ListStyleImageComputedValue, MaskModeComputedValue, MixBlendModeComputedValue, ObjectFitComputedValue,
    OutlineStyleValue, OverflowWrapValue, PointerEventsValue, QuotesComputedValue, ResizeValue, ScrollPadding,
    ScrollbarGutterComputedValue, ScrollbarWidthComputedValue, StyleSystem, TabSizeValue, TableLayoutValue,
    TextAlignLastValue, TextAlignValue, TextDecorationLineValue, TextDecorationStyleValue,
    TextDecorationThicknessValue, TextEmphasisPositionValue, TextEmphasisStyleValue, TextOverflowValue,
    TextShadowComputedValue, TextTransformValue, TextWrapComputedValue, TouchActionValue, TransformStyleValue,
    UnicodeBidiValue, UserSelectValue, VerticalAlignValue, WhiteSpaceValue, WillChangeValue, WordBreakValue,
    WritingModeValue, ZIndexValue,
};

use super::{DomMutation, apply_remove_style, apply_style_property, find_by_selector};

/// `getComputedStyle(el).getPropertyValue(prop)` 的 host 实现：解析 html → 收集 `<style>`（+
/// color-scheme meta）→ 计算样式（UA 默认 builtin via `ua_default_display`，故 `<div>` 等的
/// display 无须外链 UA stylesheet）→ 序列化查询属性。供 `__zw_get_computed_style` 回调 → shim
/// `getComputedStyle`。
///
/// **覆盖范围**：`display`/`position`/`visibility`/`opacity`（visibility/hidden 检查 + position
/// 查询主导用例）+ 颜色族（color/background-color/border-*-color/outline-color）。其余属性返 ''。
///
/// **限制**：外链 `<link>` CSS 不在 dom_html snapshot 内（snapshot 限制，同 gBCR）。**每次调用
/// 重跑 parse+cascade**——作为无缓存的参考实现；生产回调 `__zw_get_computed_style` 用
/// [`compute_document_styles`] + [`lookup_computed_property`] 配 per-snapshot 缓存复用 (doc, styles)。
pub fn computed_style_property(html: &str, selector: &str, prop: &str) -> String {
    let (doc, styles) = compute_document_styles(html);
    lookup_computed_property(&doc, &styles, selector, prop)
}

/// 解析 html 并计算全文档样式，返回 `(Document, NodeId→ComputedStyle)`。供 getComputedStyle
/// 回调的 per-snapshot 缓存：html 未变时复用此结果，避免每次属性查询重跑 parse+cascade
/// （O(文档) + O(规则×节点)）。viewport 固定 1280×800（同 getComputedStyle 既有默认）。
pub fn compute_document_styles(html: &str) -> (Document, HashMap<NodeId, ComputedStyle>) {
    let doc = parse_html(html);
    let styles = compute_styles_for_doc(&doc);
    (doc, styles)
}

/// [`compute_document_styles`] 的共享尾部：收集 `<style>` 表并计算样式（viewport 1280×800）。
/// 供纯快照路径与 inline-style override 路径复用。
fn compute_styles_for_doc(doc: &Document) -> HashMap<NodeId, ComputedStyle> {
    let sheets = crate::pipeline::collect_stylesheets(doc, "");
    let mut sys = StyleSystem::new();
    // 设默认 viewport（length 属性 % 解析需要；首批属性 viewport 无关，但为后续 length 扩展设）。
    sys.set_viewport(1280.0, 800.0);
    sys.compute_styles(doc, &sheets)
}

/// R3030：把 `mutations` 中影响元素 inline `style` 的子集顺序应用到 parsed `doc` 的匹配节点，
/// 使 getComputedStyle 在 render apply 前（snapshot 仍 stale）即反映脚本内的 inline style 变更。
///
/// 覆盖四类 mutation（与 render 路径 `apply_dom_mutations` 语义一致）：`SetStyle`/`RemoveStyle`
///（per-property 增删）+ `SetAttr`/`RemoveAttr` 且 name 归一为 `style`（`el.style.cssText=` 整体替换）。
/// **latest-wins 由顺序 apply 自然实现**——后 apply 覆盖先 apply（merge_style_property 按 prop_key
/// 去重、SetAttr style 整体覆盖），与 render 路径同序列应用结果逐位一致。handle-based 变体
///（`SetStyleOnHandle` 等）跳过：其 handle 元素未 append 前不在 snapshot 内，`find_by_selector`
/// 不命中 → 无效果，与 gCS 仅对 live-DOM 元素（有 selector）查询的契约一致。
fn apply_inline_style_overrides(doc: &mut Document, mutations: &[DomMutation]) {
    for m in mutations {
        match m {
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => {
                if let Some(node) = find_by_selector(doc, selector) {
                    apply_style_property(doc, node, property, value);
                }
            }
            DomMutation::RemoveStyle { selector, property } => {
                if let Some(node) = find_by_selector(doc, selector) {
                    apply_remove_style(doc, node, property);
                }
            }
            DomMutation::SetAttr { selector, name, value } if name.eq_ignore_ascii_case("style") => {
                if let Some(node) = find_by_selector(doc, selector) {
                    doc.set_attribute(node, "style", value);
                }
            }
            DomMutation::RemoveAttr { selector, name } if name.eq_ignore_ascii_case("style") => {
                if let Some(node) = find_by_selector(doc, selector) {
                    doc.remove_attribute(node, "style");
                }
            }
            _ => {}
        }
    }
}

/// R3030：`compute_document_styles` 的动态 inline-style override 变体——parse snapshot 后先把
/// pending inline style mutation 顺序 apply 到 parsed doc，再 cascade。闭合 getComputedStyle
/// 读 stale snapshot 的「动态样式正确性」缺口：`el.style.color='red'; getComputedStyle(el).color`
/// 旧返快照旧值（render apply 前），现返经完整 cascade 的计算值（`rgb(255, 0, 0)`），与真实浏览器
/// getComputedStyle 返 computed-value 语义一致（length/em/rem/vw 经 style-system 解析为 px，
/// %/auto 保留为计算值——同 [`serialize_computed_property`] 既有 documented 限制）。
pub fn compute_document_styles_with_inline_overrides(
    html: &str,
    mutations: &[DomMutation],
) -> (Document, HashMap<NodeId, ComputedStyle>) {
    let mut doc = parse_html(html);
    apply_inline_style_overrides(&mut doc, mutations);
    let styles = compute_styles_for_doc(&doc);
    (doc, styles)
}

/// 在已计算的 `(doc, styles)` 上按选择器查询单个属性并序列化。缓存命中路径仅此步
/// （find_by_selector + HashMap lookup + serialize，O(节点)）。
pub fn lookup_computed_property(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    selector: &str,
    prop: &str,
) -> String {
    let Some(node) = find_by_selector(doc, selector) else {
        return String::new();
    };
    let Some(style) = styles.get(&node) else {
        return String::new();
    };
    serialize_computed_property(style, prop)
}

/// 把 [`ComputedStyle`] 的单个属性序列化为 CSS 字符串（kebab-case 属性名）。覆盖：
/// display/position/visibility/opacity + 颜色族（color/background-color/border-*-color/outline-color）
/// + 长度族（width/height/min-max/margin/padding/border-*-width/border-*-radius/outline-width/
///   font-size/top/right/bottom/left/gap/letter-spacing/word-spacing/text-indent 等，经
///   [`length_to_css`]）+ 关键字/枚举族（float/clear/box-sizing/overflow-x·y/text-align/
///   white-space/font-weight/font-style/line-height/z-index/cursor/text-transform/text-overflow/
///   direction/border-collapse/table-layout/caption-side/border-*-style/outline-style）+ 复合/列表族
///   （font-family/flex-direction/wrap/justify-content/align-items/align-self/writing-mode/object-fit/
///   isolation/mix-blend-mode/pointer-events/user-select/list-style-type·position）+ 数值族
///   （flex-grow/flex-shrink/order/flex-basis/aspect-ratio）+ Transforms（transform 函数列表 /
///   transform-origin 两长度 / transform-style / backface-visibility / perspective /
///   perspective-origin）+ contain（关键字 / 位掩码组合）+ filter（函数列表）+ will-change（列表）
///   + clip-path（basic-shape 函数）+ content（生成内容）+ background 簇（position/size/repeat 多层）。未覆盖属性返 ''。
///
/// **长度族返回计算值**（非 used 值）：`compute_styles` 已把 em/rem/vw/vh/非 % calc 解析为 Px，
/// 故 px 指定值与 real browser getComputedStyle 精确一致；百分比/auto 保留为 `N%`/`auto`
///（real browser 对 width/height/margin 等 geometric 属性返 used 值——需 layout 解析百分比，
/// 此处无 layout 故返计算值；对 font-size/border-radius/gap/letter-spacing 等非 geometric 属性
/// 与 real browser 一致）。
pub fn serialize_computed_property(style: &ComputedStyle, prop: &str) -> String {
    let p = prop.trim().to_ascii_lowercase();
    // 元素自身计算 color，作 currentColor 解析上下文（color 属性本身则 currentColor→自解析）。
    let element_color = &style.color;
    // 元素 font-size（resolve 阶段已为 Px）供长度族兜底解析残余相对单位（em/rem/vw 等）。
    let font_size_px = match &style.font_size {
        LengthValue::Px(v) => *v,
        _ => 16.0,
    };
    let length = |lv: &LengthValue| length_to_css(lv, font_size_px);
    match p.as_str() {
        "display" => display_value_str(&style.display),
        "position" => position_value_str(&style.position),
        "visibility" => visibility_value_str(&style.visibility),
        "opacity" => {
            let o = style.opacity;
            // 整数 opacity 打印无小数点（"1"/"0"），与 real browser getComputedStyle 一致。
            if o == o.trunc() {
                format!("{}", o as i32)
            } else {
                format!("{o}")
            }
        }
        "color" => color_to_css(&crate::resolve_color_current(&style.color, element_color)),
        "background-color" => color_to_css(&crate::resolve_color_current(&style.background_color, element_color)),
        // ── background 简写（R2757）── 各 longhand 早覆（image/position/size/repeat/attachment/clip/origin）；
        // 简写恒完整规范形重组（无省略），Chromium 150 oracle 锚定。单层正确（多层受 ZW 单值存储限）。
        "background" => background_shorthand_to_css(style, element_color, font_size_px),
        "border-top-color" => color_to_css(&crate::resolve_color_current(&style.border_top_color, element_color)),
        "border-right-color" => color_to_css(&crate::resolve_color_current(&style.border_right_color, element_color)),
        "border-bottom-color" => color_to_css(&crate::resolve_color_current(&style.border_bottom_color, element_color)),
        "border-left-color" => color_to_css(&crate::resolve_color_current(&style.border_left_color, element_color)),
        "outline-color" => color_to_css(&crate::resolve_color_current(&style.outline_color, element_color)),
        // ── 长度族（计算值；resolve_computed_style 已把主要相对单位解析为 Px）──
        "width" => length(&style.width),
        "height" => length(&style.height),
        "min-width" => length(&style.min_width),
        "min-height" => length(&style.min_height),
        "max-width" => max_size_to_css(&style.max_width, font_size_px),
        "max-height" => max_size_to_css(&style.max_height, font_size_px),
        "margin-top" => length(&style.margin_top),
        "margin-right" => length(&style.margin_right),
        "margin-bottom" => length(&style.margin_bottom),
        "margin-left" => length(&style.margin_left),
        "padding-top" => length(&style.padding_top),
        "padding-right" => length(&style.padding_right),
        "padding-bottom" => length(&style.padding_bottom),
        "padding-left" => length(&style.padding_left),
        // ── margin / padding 简写（R2748）── longhand 早覆（line 145-152）；复用 box_4_to_css 做
        // CSSOM 4 值最小化（同 R2738 border-radius：全等→1 值 / top==bottom&&right==left→2 值 / ...）。
        "margin" => box_4_to_css(
            &style.margin_top,
            &style.margin_right,
            &style.margin_bottom,
            &style.margin_left,
            font_size_px,
        ),
        "padding" => box_4_to_css(
            &style.padding_top,
            &style.padding_right,
            &style.padding_bottom,
            &style.padding_left,
            font_size_px,
        ),
        "border-top-width" => border_width_to_css(&style.border_top_width, &style.border_top_style, font_size_px),
        "border-right-width" => border_width_to_css(&style.border_right_width, &style.border_right_style, font_size_px),
        "border-bottom-width" => {
            border_width_to_css(&style.border_bottom_width, &style.border_bottom_style, font_size_px)
        }
        "border-left-width" => border_width_to_css(&style.border_left_width, &style.border_left_style, font_size_px),
        "border-top-left-radius" => length(&style.border_top_left_radius),
        "border-top-right-radius" => length(&style.border_top_right_radius),
        "border-bottom-right-radius" => length(&style.border_bottom_right_radius),
        "border-bottom-left-radius" => length(&style.border_bottom_left_radius),
        // outline-width：real browser getComputedStyle 返 computed 值（medium→3px），
        // 与 border-width 不同——outline-width 的 used 值不因 outline-style:none 归零
        // （outline 不绘制但宽度保留）。旧实现误套 border-width 的 none→0px 规则致 default
        // 返 "0px" 与 Chromium "3px" diverge（R2754 oracle 核实）。
        "outline-width" => length(&style.outline_width),
        "font-size" => length(&style.font_size),
        "top" => length(&style.top),
        "right" => length(&style.right),
        "bottom" => length(&style.bottom),
        "left" => length(&style.left),
        // ── inset 简写（R2760）── top/right/bottom/left longhand 早覆；简写 CSSOM 4 值最小化
        // （复用 box_4_to_css，同 margin/padding/border-radius）。Chromium 150 oracle：`inset:10px`→"10px"。
        "inset" => box_4_to_css(&style.top, &style.right, &style.bottom, &style.left, font_size_px),
        // ── gap 简写修正 ── gap 是 row-gap/column-gap 简写（CSS Box Alignment 3）。
        // 旧实现仅读 legacy `style.gap`（= shorthand 首值 = row-gap），致 `gap: 5px 10px`
        // 丢 column-gap 返 "5px"（real browser 返 "5px 10px"）。改用 longhand 字段做
        // 2 值最小化（row==col→单值，否则 "row col"），与 box_4_to_css 同 CSSOM 思路。
        "gap" => {
            let rg = length(&style.row_gap);
            let cg = length(&style.column_gap);
            if rg == cg { rg } else { format!("{rg} {cg}") }
        }
        "row-gap" => length(&style.row_gap),
        "column-gap" => length(&style.column_gap),
        // letter-spacing：Chromium 150 oracle 把 0 值（默认 / normal / 显式 0/0px）恒归一为 "normal"
        //（normal 与 0 在 layout 等价；ZW parse 已把 normal→Px(0.0)，故 Px(0.0)→"normal" 精确对齐）。
        // 非 0 长度才返 "Npx"。word-spacing 不归一（恒 "0px"）。
        "letter-spacing" => letter_spacing_to_css(&style.letter_spacing, font_size_px),
        "word-spacing" => length(&style.word_spacing),
        "text-indent" => length(&style.text_indent),
        // ── 关键字/枚举族 ──
        "float" => float_value_str(&style.float),
        "clear" => clear_value_str(&style.clear),
        "box-sizing" => box_sizing_str(&style.box_sizing),
        "overflow-x" => overflow_value_str(&style.overflow_x),
        "overflow-y" => overflow_value_str(&style.overflow_y),
        // ── overflow 简写（R2745）── overflow-x/y longhand 早覆；x==y→单值，否则 "x y"（CSS Overflow 3）。
        "overflow" => {
            let x = overflow_value_str(&style.overflow_x);
            let y = overflow_value_str(&style.overflow_y);
            if x == y { x } else { format!("{x} {y}") }
        }
        // ── scroll-margin-* / scroll-padding-* / mask-mode（R2746）── scroll-snap 边距 + 遮罩模式。
        "scroll-margin-top" => format_num(style.scroll_margin_top as f64, "px"),
        "scroll-margin-right" => format_num(style.scroll_margin_right as f64, "px"),
        "scroll-margin-bottom" => format_num(style.scroll_margin_bottom as f64, "px"),
        "scroll-margin-left" => format_num(style.scroll_margin_left as f64, "px"),
        "scroll-padding-top" => scroll_padding_to_css(&style.scroll_padding_top),
        "scroll-padding-right" => scroll_padding_to_css(&style.scroll_padding_right),
        "scroll-padding-bottom" => scroll_padding_to_css(&style.scroll_padding_bottom),
        "scroll-padding-left" => scroll_padding_to_css(&style.scroll_padding_left),
        "mask-mode" => mask_mode_str(&style.mask_mode),
        // ── background-image / mask-image（R2747）── Vec<BackgroundImageComputedValue>。
        // None/Url 逐层序列化（url("u")，同 list-style-image）；任一 Gradient 层→''（gradient 全序列化
        // 是多 helper 子工程 defer，避免混合层产生错列表）；空列表→none（初值）。
        "background-image" => image_layer_list_to_css(&style.background_image, element_color, font_size_px),
        "mask-image" => image_layer_list_to_css(&style.mask_image, element_color, font_size_px),
        "text-align" => text_align_str(&style.text_align),
        "white-space" => white_space_str(&style.white_space),
        "font-weight" => font_weight_str(&style.font_weight),
        "font-style" => font_style_str(&style.font_style),
        "font-stretch" | "font-width" => format_num(style.font_stretch as f64, "%"),
        "font-kerning" => match style.font_kerning {
            FontKerningValue::Auto => "auto".to_string(),
            FontKerningValue::Normal => "normal".to_string(),
            FontKerningValue::None => "none".to_string(),
        },
        "font-variation-settings" => match &style.font_variation_settings {
            FontVariationSettingsValue::Normal => "normal".to_string(),
            FontVariationSettingsValue::Settings(settings) => settings
                .iter()
                .map(|setting| {
                    format!(
                        "\"{}\" {}",
                        String::from_utf8_lossy(&setting.tag),
                        format_num(setting.value as f64, "")
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        },
        "font-variant-alternates" => font_variant_alternates_str(&style.font_variant_alternates),
        "line-height" => line_height_str(&style.line_height, font_size_px),
        "z-index" => z_index_str(&style.z_index),
        "cursor" => cursor_str(&style.cursor),
        "text-transform" => text_transform_str(&style.text_transform),
        "text-overflow" => text_overflow_str(&style.text_overflow),
        // ── text-decoration 簇 longhand（R2754）── 4 longhand 早有 storage（types.rs），
        // 补 getComputedStyle 序列化：line（多 flag 组合，规范序 underline overline
        // line-through，空→none）/ style（5 关键字）/ color（currentcolor 解析）/
        // thickness（auto/from-font/length）。简写 text-decoration 重组 defer（CSSOM
        // 简写规则待 oracle 核实）。
        "text-decoration-line" => text_decoration_line_to_css(&style.text_decoration_line),
        "text-decoration-style" => text_decoration_style_str(&style.text_decoration_style),
        "text-decoration-color" => color_to_css(&crate::resolve_color_current(
            &style.text_decoration_color,
            element_color,
        )),
        "text-decoration-thickness" => text_decoration_thickness_to_css(&style.text_decoration_thickness),
        // ── text-underline-offset（R2762）── CSS Text Decoration 4 §2.5，下划线偏移。Auto→auto；Length→px。
        "text-underline-offset" => text_underline_offset_to_css(&style.text_underline_offset, font_size_px),
        // ── text-emphasis 簇（R2763）── CJK 文本强调。style（char→keyword 逆映射）/ color（currentcolor→rgb）/
        // position（over/under + left，省默认 right）longhand + 简写（style + color）。Chromium 150 oracle 锚定。
        "text-emphasis-style" => text_emphasis_style_to_css(&style.text_emphasis_style),
        "text-emphasis-color" => color_to_css(&crate::resolve_color_current(&style.text_emphasis_color, element_color)),
        "text-emphasis-position" => text_emphasis_position_to_css(&style.text_emphasis_position),
        "text-emphasis" => format!(
            "{} {}",
            text_emphasis_style_to_css(&style.text_emphasis_style),
            color_to_css(&crate::resolve_color_current(&style.text_emphasis_color, element_color)),
        ),
        // ── text-decoration 简写（R2755）── 4 longhand 早覆（上方）；简写 CSSOM 重组
        // （line=none→"none"；否则 line/thickness/style/color 省初值），Chromium 150 oracle 锚定。
        "text-decoration" => text_decoration_shorthand_to_css(style, element_color),
        "direction" => direction_str(&style.direction),
        "border-collapse" => border_collapse_str(&style.border_collapse),
        "table-layout" => table_layout_str(&style.table_layout),
        "caption-side" => caption_side_str(&style.caption_side),
        "border-top-style" => border_style_str(&style.border_top_style),
        "border-right-style" => border_style_str(&style.border_right_style),
        "border-bottom-style" => border_style_str(&style.border_bottom_style),
        "border-left-style" => border_style_str(&style.border_left_style),
        "outline-style" => outline_style_str(&style.outline_style),
        // ── 复合枚举/列表族（R2710）──
        "font-family" => font_family_to_css(&style.font_family),
        // ── font 简写（R2761）── style/weight/stretch/size/line-height/family CSSOM 重组。
        "font" => font_shorthand_to_css(style, font_size_px),
        "flex-direction" => flex_direction_str(&style.flex_direction),
        "flex-wrap" => flex_wrap_str(&style.flex_wrap),
        "justify-content" => alignment_str(&style.justify_content),
        "align-items" => alignment_str(&style.align_items),
        "align-self" => alignment_str(&style.align_self),
        "writing-mode" => writing_mode_str(&style.writing_mode),
        "object-fit" => object_fit_str(&style.object_fit),
        "isolation" => isolation_str(&style.isolation),
        "mix-blend-mode" => mix_blend_mode_str(&style.mix_blend_mode),
        "pointer-events" => pointer_events_str(&style.pointer_events),
        "user-select" => user_select_str(&style.user_select),
        "list-style-type" => list_style_type_str(&style.list_style_type),
        "list-style-position" => list_style_position_str(&style.list_style_position),
        // ── list-style 简写（R2755）── position/image/type longhand 早覆；简写恒 3 段 "position image type"。
        "list-style" => list_style_shorthand_to_css(style),
        // ── 数值/special 族（R2711）──
        "flex-grow" => format_num(style.flex_grow, ""),
        "flex-shrink" => format_num(style.flex_shrink, ""),
        "order" => style.order.to_string(),
        "flex-basis" => flex_basis_str(&style.flex_basis, font_size_px),
        // ── flex / flex-flow 简写（R2754）── flex="<grow> <shrink> <basis>"（恒 3 段）；
        // flex-flow="<direction> <wrap>"（恒 2 段）。Chromium oracle 锚定（flex:1→"1 1 0%"）。
        "flex" => flex_shorthand_to_css(&style.flex_basis, font_size_px, style.flex_grow, style.flex_shrink),
        "flex-flow" => format!(
            "{} {}",
            flex_direction_str(&style.flex_direction),
            flex_wrap_str(&style.flex_wrap)
        ),
        "aspect-ratio" => aspect_ratio_str(style.aspect_ratio, style.aspect_ratio_auto),
        // ── transform（R2715）── CSS Transforms L1/L2 计算值 = 函数列表（Chromium 返 resolved matrix，diverge）。
        "transform" => transform_to_css(&style.transform),
        // ── transform-origin（R2716）── 2 LengthValue 经 length_to_css，空格连接。
        "transform-origin" => {
            transform_origin_to_css(&style.transform_origin_x, &style.transform_origin_y, font_size_px)
        }
        // ── contain（R2717）── CSS Containment L1/L2 计算值（关键字 / 位掩码组合）。
        "contain" => contain_to_css(&style.contain),
        // ── filter（R2718）── CSS Filter Effects 函数列表（空 Vec / None → none）。
        "filter" => filter_to_css(&style.filter, element_color),
        // ── backdrop-filter（R2762）── 与 filter 同 FilterComputedValue 列表，复用 filter_to_css
        // （空→none，Chromium oracle 一致）。glass/frosted 效果高频查询。
        "backdrop-filter" => filter_to_css(&style.backdrop_filter, element_color),
        // ── 3D Transforms 簇（R2719）── transform-style / backface-visibility 枚举 + perspective /
        // perspective-origin（与 transform / transform-origin 同族，完成 3D transform 簇）。
        "transform-style" => transform_style_str(&style.transform_style),
        "backface-visibility" => backface_visibility_str(&style.backface_visibility),
        "perspective" => perspective_to_css(&style.perspective, font_size_px),
        "perspective-origin" => format!(
            "{} {}",
            length_to_css(&style.perspective_origin_x, font_size_px),
            length_to_css(&style.perspective_origin_y, font_size_px)
        ),
        // ── will-change（R2720）── CSS Will Change 列表（空 Vec / Auto → auto）。
        "will-change" => will_change_to_css(&style.will_change),
        // ── clip-path（R2721）── CSS Masking basic-shape 函数（none / inset / circle / ellipse / polygon）。
        "clip-path" => clip_path_to_css(&style.clip_path, font_size_px),
        // ── content（R2722）── CSS Generated Content（::before/::after 生成内容，多 component value）。
        "content" => content_to_css(&style.content),
        // ── background-position（R2724）── CSS Backgrounds <bg-position># 多层（逗号分隔）。
        "background-position" => background_position_to_css(&style.background_position),
        // ── background-size / background-repeat（R2725）── CSS Backgrounds 多层（逗号分隔）。
        "background-size" => background_size_to_css(&style.background_size),
        "background-repeat" => background_repeat_to_css(&style.background_repeat),
        // ── background-attachment / clip / origin（R2726）── CSS Backgrounds 单值 box-model 枚举。
        "background-attachment" => background_attachment_to_css(&style.background_attachment),
        "background-clip" => background_clip_to_css(&style.background_clip),
        "background-origin" => background_origin_to_css(&style.background_origin),
        // ── align-content / justify-items / justify-self（R2727）── CSS Box Alignment 单值枚举
        // （补齐 align-items/align-self/justify-content R2710 后的 alignment 簇缺口）。
        "align-content" => align_content_to_css(&style.align_content),
        "justify-items" => justify_items_to_css(&style.justify_items),
        "justify-self" => justify_self_to_css(&style.justify_self),
        // ── place-content / place-items / place-self 简写（R2758）── align+justify longhand 早覆；
        // 简写 CSSOM 2 值最小化（align==justify→单值，否则 "align justify"），Chromium 150 oracle 锚定。
        "place-content" => place_2value_min(
            &align_content_to_css(&style.align_content),
            &alignment_str(&style.justify_content),
        ),
        "place-items" => place_2value_min(
            &alignment_str(&style.align_items),
            &justify_items_to_css(&style.justify_items),
        ),
        "place-self" => place_2value_min(
            &alignment_str(&style.align_self),
            &justify_self_to_css(&style.justify_self),
        ),
        // ── word-break / overflow-wrap / hyphens / line-break（R2728）── CSS Text 换行/断词单值枚举。
        "word-break" => word_break_to_css(&style.word_break),
        "overflow-wrap" => overflow_wrap_to_css(&style.overflow_wrap),
        "hyphens" => hyphens_to_css(&style.hyphens),
        "line-break" => line_break_to_css(&style.line_break),
        // ── vertical-align / unicode-bidi / empty-cells（R2729）── 单值关键字枚举。
        "vertical-align" => vertical_align_to_css(&style.vertical_align),
        "unicode-bidi" => unicode_bidi_to_css(&style.unicode_bidi),
        "empty-cells" => empty_cells_to_css(&style.empty_cells),
        // ── caret-color / accent-color（R2730）── CSS UI 颜色（auto | <color>，复用 R2705 颜色解析）。
        "caret-color" => caret_color_to_css(&style.caret_color, element_color),
        "accent-color" => accent_color_to_css(&style.accent_color, element_color),
        // ── text-wrap / text-align-last / resize / appearance（R2731）── 单值关键字枚举。
        "text-wrap" => text_wrap_to_css(&style.text_wrap),
        "text-align-last" => text_align_last_to_css(&style.text_align_last),
        "resize" => resize_to_css(&style.resize),
        "appearance" => appearance_to_css(&style.appearance),
        // ── box-decoration-break / scrollbar-* / touch-action（R2732）── 容器交互/UI 单值枚举。
        "box-decoration-break" => box_decoration_break_to_css(&style.box_decoration_break),
        "scrollbar-width" => scrollbar_width_to_css(&style.scrollbar_width),
        "scrollbar-gutter" => scrollbar_gutter_to_css(&style.scrollbar_gutter),
        "touch-action" => touch_action_to_css(&style.touch_action),
        // ── outline-offset / break-*（R2733）── 补齐 outline 簇 + CSS Fragmentation 簇。
        "outline-offset" => length(&style.outline_offset),
        "break-before" => break_value_to_css(&style.break_before),
        "break-after" => break_value_to_css(&style.break_after),
        "break-inside" => break_inside_to_css(&style.break_inside),
        // ── grid-auto-flow / container-type·name / tab-size（R2734）── Grid 簇起 + Containment 簇。
        "grid-auto-flow" => grid_auto_flow_to_css(&style.grid_auto_flow),
        "container-type" => container_type_to_css(&style.container_type),
        "container-name" => match &style.container_name {
            None => "none".to_string(),
            Some(n) => n.clone(),
        },
        "tab-size" => tab_size_to_css(&style.tab_size, font_size_px),
        // ── border-spacing / list-style-image / font-size-adjust（R2735）── 补齐 table/list/font 簇。
        "border-spacing" => border_spacing_to_css(&style.border_spacing),
        "list-style-image" => list_style_image_to_css(&style.list_style_image),
        "font-size-adjust" => font_size_adjust_to_css(&style.font_size_adjust),
        // ── border-image-source / object-position / quotes（R2736）── 简单枚举收尾。
        // border-image-source：None/Url，复用 R2735 list-style-image 的 url() 引号模式。
        "border-image-source" => border_image_source_to_css(&style.border_image_source, element_color, font_size_px),
        // ── border-image 简写（R2765）── source/slice/width/outset/repeat 5 子分量 CSSOM 重组。
        // Chromium 150 oracle：source==none→整值 "none"；否则恒全量 "<source> <slice> / <width> / <outset> <repeat>"。
        "border-image" => border_image_shorthand_to_css(style, element_color, font_size_px),
        // ── border-image 切片族 longhand（R2764）── slice/width/outset 4 值最小化 + repeat 2 值。
        // CSS Border Image §3，Chromium 150 oracle 锚定（slice 默认 100% 修 Percent，paint-neutral）。
        "border-image-slice" => border_image_slice_to_css(&style.border_image_slice),
        "border-image-width" => border_image_width_to_css(&style.border_image_width),
        "border-image-outset" => border_image_outset_to_css(&style.border_image_outset),
        "border-image-repeat" => border_image_repeat_to_css(&style.border_image_repeat),
        // object-position：单 <position>，复用 R2724 background-position 的逐层序列化（默认 Center→50% 50%）。
        "object-position" => bg_position_layer_to_css(&style.object_position),
        // quotes：None/Auto/Pairs（auto 初值；pairs→空格分隔双引号串，复用 css_string_to_css 转义）。
        "quotes" => quotes_to_css(&style.quotes),
        // ── CSS Multi-column 簇（R2737）── column-gap 已覆（R2707 length 族）；补 rule/count/width/fill/span。
        "column-rule-width" => {
            column_rule_width_to_css(&style.column_rule_width, &style.column_rule_style, font_size_px)
        }
        "column-rule-style" => column_rule_style_str(&style.column_rule_style),
        "column-rule-color" => color_to_css(&crate::resolve_color_current(&style.column_rule_color, element_color)),
        "column-count" => column_count_to_css(&style.column_count),
        "column-width" => column_width_to_css(&style.column_width, font_size_px),
        "column-fill" => column_fill_str(&style.column_fill),
        "column-span" => column_span_str(&style.column_span),
        // ── columns / column-rule 简写（R2755）── longhand 早覆（上方）；简写 CSSOM 重组
        // （columns 省 auto / column-rule 省 none style），Chromium 150 oracle 锚定。
        "columns" => columns_shorthand_to_css(&style.column_width, &style.column_count, font_size_px),
        "column-rule" => column_rule_shorthand_to_css(
            &style.column_rule_width,
            &style.column_rule_style,
            &style.column_rule_color,
            element_color,
            font_size_px,
        ),
        // ── font-variant-numeric / image-rendering（R2737）── 单值关键字枚举（残余纯枚举收尾）。
        "font-variant-numeric" => font_variant_numeric_str(&style.font_variant_numeric),
        "image-rendering" => image_rendering_str(&style.image_rendering),
        // ── border-radius 簇（R2738）── 4 角 longhand 早覆（line 156-159 经 length 闭包）；
        // 此处补简写：复用 box_4_to_css 的 CSSOM 4 值最小化（全等→1 值 / TL==BR&&TR==BL→2 值 / BL==TR→3 值）。
        "border-radius" => box_4_to_css(
            &style.border_top_left_radius,
            &style.border_top_right_radius,
            &style.border_bottom_right_radius,
            &style.border_bottom_left_radius,
            font_size_px,
        ),
        // ── border 简写簇（R2754）── per-side 简写 "<width> <style> <color>"（width 经
        // none/hidden→0px used 规则，复用 border_width_to_css）；全边 border 仅当 4 边 width/style/color
        // 序列化全等时返单边值，否则 ''（Chromium oracle：border-top:1px;border-bottom:2px → border=""）。
        "border-top" => border_side_shorthand(
            &style.border_top_width,
            &style.border_top_style,
            &style.border_top_color,
            element_color,
            font_size_px,
        ),
        "border-right" => border_side_shorthand(
            &style.border_right_width,
            &style.border_right_style,
            &style.border_right_color,
            element_color,
            font_size_px,
        ),
        "border-bottom" => border_side_shorthand(
            &style.border_bottom_width,
            &style.border_bottom_style,
            &style.border_bottom_color,
            element_color,
            font_size_px,
        ),
        "border-left" => border_side_shorthand(
            &style.border_left_width,
            &style.border_left_style,
            &style.border_left_color,
            element_color,
            font_size_px,
        ),
        "border" => border_shorthand(style, element_color, font_size_px),
        // ── outline 简写（R2754）── "<color> <style> <width>"（与 border 的 width-style-color
        // 顺序相反！恒 3 段含 none/0px；Chromium oracle：default "rgb(0,0,0) none 3px"）。
        "outline" => format!(
            "{} {} {}",
            color_to_css(&crate::resolve_color_current(&style.outline_color, element_color)),
            outline_style_str(&style.outline_style),
            length(&style.outline_width),
        ),
        // ── box-shadow / text-shadow（R2739）── 多阴影列表，空→none。
        // Chromium/WPT 格式：color 在前（解析 rgb/rgba）+ 全长度（box-shadow 4 长 + inset 末；text-shadow 3 长）。
        "box-shadow" => box_shadow_to_css(&style.box_shadow, element_color),
        "text-shadow" => text_shadow_to_css(&style.text_shadow, element_color),
        // ── Grid 轨道簇（R2740）── Option<String> 存原始 specified 值（apply.rs 未展开 repeat()，
        // 故 repeat() diverge Chromium 展开，pre-existing 解析限制；非 repeat 的固定/fr/minmax 一致）。
        // grid-template-* 初始 none；grid-auto-* 初始 auto（CSS Grid §6.1/§6.4）。
        "grid-template-columns" => opt_css_string(&style.grid_template_columns, "none"),
        "grid-template-rows" => opt_css_string(&style.grid_template_rows, "none"),
        "grid-template-areas" => opt_css_string(&style.grid_template_areas, "none"),
        // ── grid-template 简写（R2766）── rows/columns/areas 三 longhand（Option<String> 原始串）重组。
        // Chromium 150 oracle：全 none→"none"；areas==none→"<rows> / <cols>"；areas!=none→引号区域与行尺寸
        // 逐行交错 + " / " + cols（area 数 != 行尺寸数 → "" 空串，Chromium 同样不可序列化）。
        "grid-template" => grid_template_shorthand_to_css(style),
        "grid-auto-columns" => opt_css_string(&style.grid_auto_columns, "auto"),
        "grid-auto-rows" => opt_css_string(&style.grid_auto_rows, "auto"),
        // ── grid 线定位 longhand + 简写（R2759）── grid-column/row-start/end longhand + grid-column/row/area
        // 简写（CSSOM 最小化），Chromium 150 oracle 锚定。
        "grid-column-start" => grid_line_to_css(&style.grid_column_start),
        "grid-column-end" => grid_line_to_css(&style.grid_column_end),
        "grid-row-start" => grid_line_to_css(&style.grid_row_start),
        "grid-row-end" => grid_line_to_css(&style.grid_row_end),
        "grid-column" => grid_line_pair_to_css(&style.grid_column_start, &style.grid_column_end),
        "grid-row" => grid_line_pair_to_css(&style.grid_row_start, &style.grid_row_end),
        "grid-area" => grid_area_to_css(style),
        // ── containment 簇（R2741）── content-visibility（3 关键字，初值 visible）+
        // contain-intrinsic-width/height（Option<LengthValue>，None→none，CSS Sizing 4 初值 none）。
        "content-visibility" => content_visibility_str(&style.content_visibility),
        "contain-intrinsic-width" => opt_length_to_css(&style.contain_intrinsic_width, font_size_px),
        "contain-intrinsic-height" => opt_length_to_css(&style.contain_intrinsic_height, font_size_px),
        // ── counter-increment / counter-reset（R2742）── Vec<CounterActionValue>，空→none；
        // 否则空格分隔 `name integer` 列表（value=None 时取默认：increment=1 / reset=0）。
        "counter-increment" => counter_action_to_css(&style.counter_increment, 1),
        "counter-reset" => counter_action_to_css(&style.counter_reset, 0),
        // ── transition/animation 簇（R2743）── 列表属性，逗号分隔；空→各 CSS 初值。
        // timing-function（含 steps() 位置歧义）defer 到后续轮核实，本轮不含。
        "transition-property" => string_list_to_css(&style.transition_property, "all"),
        "transition-duration" => time_list_to_css(&style.transition_duration, "0s"),
        "transition-delay" => time_list_to_css(&style.transition_delay, "0s"),
        "animation-name" => string_list_to_css(&style.animation_name, "none"),
        "animation-duration" => time_list_to_css(&style.animation_duration, "0s"),
        "animation-delay" => time_list_to_css(&style.animation_delay, "0s"),
        "animation-iteration-count" => iter_count_list_to_css(&style.animation_iteration_count),
        "animation-direction" => enum_list_to_css(&style.animation_direction, "normal", animation_direction_str),
        "animation-fill-mode" => enum_list_to_css(&style.animation_fill_mode, "none", animation_fill_mode_str),
        "animation-play-state" => enum_list_to_css(&style.animation_play_state, "running", animation_play_state_str),
        // ── transition/animation-timing-function（R2744）── Vec<TimingFunctionValue>，空→ease（初值）。
        "transition-timing-function" => {
            enum_list_to_css(&style.transition_timing_function, "ease", timing_function_to_css)
        }
        "animation-timing-function" => {
            enum_list_to_css(&style.animation_timing_function, "ease", timing_function_to_css)
        }
        // ── transition / animation 简写（R2756）── longhand 早覆（上方列表族）；简写 CSSOM 重组
        // （逐索引 zip 等长列表，省初值，逗号连接），Chromium 150 oracle 锚定。
        "transition" => transition_shorthand_to_css(style),
        "animation" => animation_shorthand_to_css(style),
        _ => String::new(),
    }
}

/// 把解析后的 [`Color`]（render-foundation，u8 通道）序列化为 CSS 颜色串。不透明 →
/// `rgb(r, g, b)`；含透明度 → `rgba(r, g, b, a)`（a 为 0-1 小数，对齐 real browser）。
fn color_to_css(c: &zero_render_foundation::color::Color) -> String {
    if c.a == 255 {
        format!("rgb({}, {}, {})", c.r, c.g, c.b)
    } else {
        let alpha = ((c.a as f64 / 255.0) * 1000.0).round() / 1000.0;
        format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, alpha)
    }
}

/// caret-color：CSS UI 光标颜色（auto | <color>）。Auto→auto（CSS UI 4 初值，Chromium 返 auto）；
/// Color→经 [`crate::resolve_color_current`] 解析 currentcolor 后 rgb/rgba（复用 R2705 颜色路径）。
fn caret_color_to_css(c: &CaretColorComputedValue, element_color: &ColorValue) -> String {
    match c {
        CaretColorComputedValue::Auto => "auto".to_string(),
        CaretColorComputedValue::Color(col) => color_to_css(&crate::resolve_color_current(col, element_color)),
    }
}

/// accent-color：CSS UI 表单控件强调色（auto | <color>）。Auto→auto；Color→rgb/rgba（复用 R2705）。
fn accent_color_to_css(a: &AccentColorComputedValue, element_color: &ColorValue) -> String {
    match a {
        AccentColorComputedValue::Auto => "auto".to_string(),
        AccentColorComputedValue::Color(col) => color_to_css(&crate::resolve_color_current(col, element_color)),
    }
}

/// text-wrap：CSS Text 4 行换行模式单值序列化。初值 wrap。
fn text_wrap_to_css(t: &TextWrapComputedValue) -> String {
    match t {
        TextWrapComputedValue::Wrap => "wrap",
        TextWrapComputedValue::Nowrap => "nowrap",
        TextWrapComputedValue::Balance => "balance",
        TextWrapComputedValue::Pretty => "pretty",
        TextWrapComputedValue::Stable => "stable",
    }
    .to_string()
}

/// text-align-last：CSS Text 最后一行对齐单值序列化。初值 auto。
fn text_align_last_to_css(t: &TextAlignLastValue) -> String {
    match t {
        TextAlignLastValue::Auto => "auto",
        TextAlignLastValue::Start => "start",
        TextAlignLastValue::End => "end",
        TextAlignLastValue::Left => "left",
        TextAlignLastValue::Right => "right",
        TextAlignLastValue::Center => "center",
        TextAlignLastValue::Justify => "justify",
    }
    .to_string()
}

/// resize：CSS UI 可调整尺寸单值序列化。初值 none。
fn resize_to_css(r: &ResizeValue) -> String {
    match r {
        ResizeValue::None => "none",
        ResizeValue::Both => "both",
        ResizeValue::Horizontal => "horizontal",
        ResizeValue::Vertical => "vertical",
        ResizeValue::Block => "block",
        ResizeValue::Inline => "inline",
    }
    .to_string()
}

/// appearance：CSS Basic UI 控件外观单值序列化。CamelCase→kebab-case。初值 auto。
fn appearance_to_css(a: &AppearanceComputedValue) -> String {
    match a {
        AppearanceComputedValue::None => "none",
        AppearanceComputedValue::Auto => "auto",
        AppearanceComputedValue::Button => "button",
        AppearanceComputedValue::Checkbox => "checkbox",
        AppearanceComputedValue::Listbox => "listbox",
        AppearanceComputedValue::Menulist => "menulist",
        AppearanceComputedValue::Meter => "meter",
        AppearanceComputedValue::ProgressBar => "progress-bar",
        AppearanceComputedValue::PushButton => "push-button",
        AppearanceComputedValue::Radio => "radio",
        AppearanceComputedValue::Searchfield => "searchfield",
        AppearanceComputedValue::SliderHorizontal => "slider-horizontal",
        AppearanceComputedValue::SquareButton => "square-button",
        AppearanceComputedValue::Textarea => "textarea",
        AppearanceComputedValue::Textfield => "textfield",
    }
    .to_string()
}

/// box-decoration-break：CSS Box 装饰断行单值序列化。初值 slice。
fn box_decoration_break_to_css(b: &BoxDecorationBreakValue) -> String {
    match b {
        BoxDecorationBreakValue::Slice => "slice",
        BoxDecorationBreakValue::Clone => "clone",
    }
    .to_string()
}

/// scrollbar-width：CSS Scrollbars 单值序列化。初值 auto（CSS 规范；ZeroWeb 一致）。
fn scrollbar_width_to_css(s: &ScrollbarWidthComputedValue) -> String {
    match s {
        ScrollbarWidthComputedValue::Auto => "auto",
        ScrollbarWidthComputedValue::Thin => "thin",
        ScrollbarWidthComputedValue::None => "none",
    }
    .to_string()
}

/// scrollbar-gutter：CSS Overflow 4 单值序列化。初值 auto；StableBothEdges→`stable both-edges`。
fn scrollbar_gutter_to_css(s: &ScrollbarGutterComputedValue) -> String {
    match s {
        ScrollbarGutterComputedValue::Auto => "auto",
        ScrollbarGutterComputedValue::Stable => "stable",
        ScrollbarGutterComputedValue::StableBothEdges => "stable both-edges",
    }
    .to_string()
}

/// touch-action：CSS Pointer Events 单值序列化。初值 auto；PanXPanY→`pan-x pan-y`（空格分隔）。
fn touch_action_to_css(t: &TouchActionValue) -> String {
    match t {
        TouchActionValue::Auto => "auto",
        TouchActionValue::None => "none",
        TouchActionValue::PanX => "pan-x",
        TouchActionValue::PanY => "pan-y",
        TouchActionValue::PanXPanY => "pan-x pan-y",
        TouchActionValue::Manipulation => "manipulation",
    }
    .to_string()
}

/// break-before / break-after：CSS Fragmentation 单值序列化（共享 [`BreakValue`]）。
/// 初值 auto。ZeroWeb enum 仅 6 变体（CSS 规范另含 recto/verso/left/right/region 等，未存储）。
/// CamelCase→kebab：AvoidPage→avoid-page、AvoidColumn→avoid-column。
fn break_value_to_css(b: &BreakValue) -> String {
    match b {
        BreakValue::Auto => "auto",
        BreakValue::Avoid => "avoid",
        BreakValue::Column => "column",
        BreakValue::Page => "page",
        BreakValue::AvoidPage => "avoid-page",
        BreakValue::AvoidColumn => "avoid-column",
    }
    .to_string()
}

/// break-inside：CSS Fragmentation 单值序列化。初值 auto。
fn break_inside_to_css(b: &BreakInsideValue) -> String {
    match b {
        BreakInsideValue::Auto => "auto",
        BreakInsideValue::Avoid => "avoid",
        BreakInsideValue::AvoidPage => "avoid-page",
        BreakInsideValue::AvoidColumn => "avoid-column",
    }
    .to_string()
}

/// grid-auto-flow：CSS Grid 自动放置算法单值序列化。初值 row；dense 组合为多词值。
fn grid_auto_flow_to_css(g: &GridAutoFlowValue) -> String {
    match g {
        GridAutoFlowValue::Row => "row",
        GridAutoFlowValue::Column => "column",
        GridAutoFlowValue::RowDense => "row dense",
        GridAutoFlowValue::ColumnDense => "column dense",
    }
    .to_string()
}

/// container-type：CSS Containment 容器类型单值序列化。初值 normal。
fn container_type_to_css(c: &ContainerType) -> String {
    match c {
        ContainerType::Normal => "normal",
        ContainerType::Size => "size",
        ContainerType::InlineSize => "inline-size",
    }
    .to_string()
}

/// tab-size：CSS Text 制表符宽度。Number→无单位整数；Length→经 [`length_to_css`] px。
/// 初值 Number(8) → `8`（CSS 规范初值 8，Chromium getComputedStyle 返 `8`）。
fn tab_size_to_css(t: &TabSizeValue, font_size_px: f64) -> String {
    match t {
        TabSizeValue::Number(n) => format!("{n}"),
        TabSizeValue::Length(lv) => length_to_css(lv, font_size_px),
    }
}

/// border-spacing：CSS Table 单元格间距。两个 px 值；水平==垂直时 Chromium 返单值 `Xpx`，
/// 否则 `Xpx Ypx`。初值 0px。**补齐 table 簇**（table-layout/caption-side/border-collapse/empty-cells）。
fn border_spacing_to_css(s: &BorderSpacingComputedValue) -> String {
    let h = format_num(s.horizontal as f64, "px");
    if s.horizontal == s.vertical {
        h
    } else {
        format!("{} {}", h, format_num(s.vertical as f64, "px"))
    }
}

/// list-style-image：CSS List 列表标记图。None→none；Url(s)→`url("<s>")`（对齐 Chromium 引号形式）。
/// **补齐 list-style 簇**（type/position 已覆，+image）。
fn list_style_image_to_css(i: &ListStyleImageComputedValue) -> String {
    match i {
        ListStyleImageComputedValue::None => "none".to_string(),
        ListStyleImageComputedValue::Url(u) => format!("url(\"{u}\")"),
    }
}

/// `font-size-adjust` 的 computed serialization。
fn font_size_adjust_to_css(f: &FontSizeAdjustValue) -> String {
    match f {
        FontSizeAdjustValue::None => "none".to_string(),
        FontSizeAdjustValue::Adjust { metric, basis } => {
            let metric = metric.map(|metric| match metric {
                FontSizeAdjustMetric::ExHeight => "ex-height ",
                FontSizeAdjustMetric::CapHeight => "cap-height ",
                FontSizeAdjustMetric::ChWidth => "ch-width ",
                FontSizeAdjustMetric::IcWidth => "ic-width ",
                FontSizeAdjustMetric::IcHeight => "ic-height ",
            });
            let basis = match basis {
                FontSizeAdjustBasis::Number(value) => format_num(*value, ""),
                FontSizeAdjustBasis::FromFont => "from-font".to_string(),
            };
            format!("{}{basis}", metric.unwrap_or_default())
        }
    }
}

/// border-image-source：CSS Border Image 源图。None→none；Url(s)→`url("<s>")`（复用 R2735
/// list-style-image 的引号形式，对齐 Chromium）。补齐 border-image 子簇（slice/width/repeat/outset
/// 仍残余，需 track-list/数值序列化）。
fn border_image_source_to_css(
    s: &BorderImageSourceComputedValue,
    element_color: &ColorValue,
    font_size_px: f64,
) -> String {
    use zero_css_parser::values::GradientValue;
    match s {
        BorderImageSourceComputedValue::None => "none".to_string(),
        BorderImageSourceComputedValue::Url(u) => format!("url(\"{u}\")"),
        BorderImageSourceComputedValue::Gradient(GradientValue::Linear(g)) => {
            linear_gradient_to_css(g, element_color, font_size_px)
        }
        BorderImageSourceComputedValue::Gradient(GradientValue::Radial(g)) => {
            radial_gradient_to_css(g, element_color, font_size_px)
        }
        BorderImageSourceComputedValue::Gradient(GradientValue::Conic(g)) => {
            conic_gradient_to_css(g, element_color, font_size_px)
        }
    }
}

/// 4 值 CSSOM 最小化（字符串版，供 border-image slice/width/outset 复用）：
/// 全等→1 / top==bottom&&right==left→2 / right==left→3 / 否则 4。
fn box4_str_min(top: &str, right: &str, bottom: &str, left: &str) -> String {
    if top == right && right == bottom && bottom == left {
        top.to_string()
    } else if top == bottom && right == left {
        format!("{top} {right}")
    } else if right == left {
        format!("{top} {right} {bottom}")
    } else {
        format!("{top} {right} {bottom} {left}")
    }
}

/// border-image-slice：CSS Border Image §3.2。Number→`n` / Percent→`n%`；4 值最小化；fill 真→末尾 ` fill`。
/// Chromium 150 oracle：默认→`"100%"`、`10 20 30 40`→`"10 20 30 40"`、`10% fill`→`"10% fill"`。
fn border_image_slice_to_css(s: &BorderImageSliceComputedValue) -> String {
    let cmp = |c: &BorderImageSliceComputedComponent| match c {
        BorderImageSliceComputedComponent::Number(n) => format_num(*n as f64, ""),
        BorderImageSliceComputedComponent::Percent(n) => format_num(*n as f64, "%"),
    };
    let mut out = box4_str_min(&cmp(&s.top), &cmp(&s.right), &cmp(&s.bottom), &cmp(&s.left));
    if s.fill {
        out.push_str(" fill");
    }
    out
}

/// border-image-width：Auto→`auto` / Number→`n` / Length→px / Percent→`n%`；4 值最小化。
/// Chromium 150 oracle：默认→`"1"`、`10px 20px`→`"10px 20px"`、`auto`→`"auto"`。
fn border_image_width_to_css(w: &BorderImageWidthComputedValue) -> String {
    let cmp = |c: &BorderImageWidthComputedComponent| match c {
        BorderImageWidthComputedComponent::Auto => "auto".to_string(),
        BorderImageWidthComputedComponent::Number(n) => format_num(*n as f64, ""),
        BorderImageWidthComputedComponent::Length(px) => format_num(*px as f64, "px"),
        BorderImageWidthComputedComponent::Percent(n) => format_num(*n as f64, "%"),
    };
    box4_str_min(&cmp(&w.top), &cmp(&w.right), &cmp(&w.bottom), &cmp(&w.left))
}

/// border-image-outset：Number→`n` / Length→px；4 值最小化。Chromium 150 oracle：默认→`"0"`、`5px 10px`→`"5px 10px"`。
fn border_image_outset_to_css(o: &BorderImageOutsetComputedValue) -> String {
    let cmp = |c: &BorderImageOutsetComputedComponent| match c {
        BorderImageOutsetComputedComponent::Number(n) => format_num(*n as f64, ""),
        BorderImageOutsetComputedComponent::Length(px) => format_num(*px as f64, "px"),
    };
    box4_str_min(&cmp(&o.top), &cmp(&o.right), &cmp(&o.bottom), &cmp(&o.left))
}

/// border-image-repeat：水平/垂直；stretch/repeat/round/space。相等→单值，否则 `"horizontal vertical"`。
/// Chromium 150 oracle：默认→`"stretch"`、`round repeat`→`"round repeat"`。
fn border_image_repeat_mode_str(m: &BorderImageRepeatComputedMode) -> &'static str {
    match m {
        BorderImageRepeatComputedMode::Stretch => "stretch",
        BorderImageRepeatComputedMode::Repeat => "repeat",
        BorderImageRepeatComputedMode::Round => "round",
        BorderImageRepeatComputedMode::Space => "space",
    }
}

fn border_image_repeat_to_css(r: &BorderImageRepeatComputedValue) -> String {
    let h = border_image_repeat_mode_str(&r.horizontal);
    let v = border_image_repeat_mode_str(&r.vertical);
    if h == v { h.to_string() } else { format!("{h} {v}") }
}

/// `border-image` 简写：5 子分量 CSSOM 重组。Chromium 150 oracle 锚定：
/// `border-image-source==none` → 整值 `"none"`（不论其余子分量是否非初值）；
/// 否则恒全量 `"<source> <slice> / <width> / <outset> <repeat>"`（不省初值，width/outset 各独占一个 `/`）。
/// 复用 4 切片族 longhand 序列化（slice/width/outset/repeat）+ source 序列化。
fn border_image_shorthand_to_css(style: &ComputedStyle, element_color: &ColorValue, font_size_px: f64) -> String {
    if matches!(style.border_image_source, BorderImageSourceComputedValue::None) {
        return "none".to_string();
    }
    format!(
        "{} {} / {} / {} {}",
        border_image_source_to_css(&style.border_image_source, element_color, font_size_px),
        border_image_slice_to_css(&style.border_image_slice),
        border_image_width_to_css(&style.border_image_width),
        border_image_outset_to_css(&style.border_image_outset),
        border_image_repeat_to_css(&style.border_image_repeat),
    )
}

/// quotes：CSS Generated Content 引号。None→none；Auto（初值）→auto；Pairs→逐对开/闭串空格分隔
/// 双引号化（复用 [`css_string_to_css`] 转义 `\`/`"`/换行），对齐 Chromium getComputedStyle
///（`quotes: "«" "»" "‹" "›"` → `"«" "»" "‹" "›"`）。
fn quotes_to_css(q: &QuotesComputedValue) -> String {
    match q {
        QuotesComputedValue::None => "none".to_string(),
        QuotesComputedValue::Auto => "auto".to_string(),
        QuotesComputedValue::Pairs(pairs) => pairs
            .iter()
            .flat_map(|(open, close)| [css_string_to_css(open), css_string_to_css(close)])
            .collect::<Vec<_>>()
            .join(" "),
    }
}

// ── CSS Multi-column 簇（R2737）──────────────────────────────────────────
// column-gap 已由 length 族（R2707）覆盖；此处补 rule-width/style/color + count/width/fill/span。

/// column-rule-width：CSS Multi-column 分隔线宽度。Medium/Thin/Thick→对齐 Chromium used px
///（3px/1px/5px，CSS Border 的常规 UA 初始值）；Length→px（经 length_to_css）。**column-rule-width 的
/// computed 值独立于 column-rule-style**——Chromium 对 style=none/hidden 仍返 medium→3px（R2755 oracle
/// 核实），与 border-width 的 none/hidden→used 0px 语义**不同**（R2737 曾误套 border-width 规则致 diverge）。
fn column_rule_width_to_css(
    w: &ColumnRuleWidthComputedValue,
    _style: &ColumnRuleStyleComputedValue,
    font_size_px: f64,
) -> String {
    match w {
        ColumnRuleWidthComputedValue::Medium => "3px".to_string(),
        ColumnRuleWidthComputedValue::Thin => "1px".to_string(),
        ColumnRuleWidthComputedValue::Thick => "5px".to_string(),
        ColumnRuleWidthComputedValue::Length(l) => length_to_css(l, font_size_px),
    }
}

/// column-rule-style：CSS Multi-column 分隔线样式（10 关键字，同 border-style 语义但独立枚举）。
fn column_rule_style_str(s: &ColumnRuleStyleComputedValue) -> String {
    match s {
        ColumnRuleStyleComputedValue::None => "none",
        ColumnRuleStyleComputedValue::Hidden => "hidden",
        ColumnRuleStyleComputedValue::Dotted => "dotted",
        ColumnRuleStyleComputedValue::Dashed => "dashed",
        ColumnRuleStyleComputedValue::Solid => "solid",
        ColumnRuleStyleComputedValue::Double => "double",
        ColumnRuleStyleComputedValue::Groove => "groove",
        ColumnRuleStyleComputedValue::Ridge => "ridge",
        ColumnRuleStyleComputedValue::Inset => "inset",
        ColumnRuleStyleComputedValue::Outset => "outset",
    }
    .to_string()
}

/// column-count：CSS Multi-column 列数。Auto→auto；Number(n)→无单位正整数（对齐 Chromium）。
fn column_count_to_css(c: &ColumnCountComputedValue) -> String {
    match c {
        ColumnCountComputedValue::Auto => "auto".to_string(),
        ColumnCountComputedValue::Number(n) => n.to_string(),
    }
}

/// column-width：CSS Multi-column 列宽。Auto→auto；Length→px（经 length_to_css 解析残余相对单位）。
fn column_width_to_css(w: &ColumnWidthComputedValue, font_size_px: f64) -> String {
    match w {
        ColumnWidthComputedValue::Auto => "auto".to_string(),
        ColumnWidthComputedValue::Length(l) => length_to_css(l, font_size_px),
    }
}

/// column-fill：CSS Multi-column 列填充。Balance（初值）/Auto（对齐 Chromium）。
fn column_fill_str(f: &ColumnFillComputedValue) -> String {
    match f {
        ColumnFillComputedValue::Balance => "balance",
        ColumnFillComputedValue::Auto => "auto",
    }
    .to_string()
}

/// column-span：CSS Multi-column 列跨越。None（初值）/All（对齐 Chromium）。
fn column_span_str(s: &ColumnSpanComputedValue) -> String {
    match s {
        ColumnSpanComputedValue::None => "none",
        ColumnSpanComputedValue::All => "all",
    }
    .to_string()
}

// ── CSS Multi-column / Lists / Text-decoration 简写（R2755）─────────────────
// 复用上方 longhand helper 做 CSSOM 重组；oracle 锚定本地 Chromium 150（见
// docs/learnings/patterns/local-chromium-getcomputedstyle-oracle.md）。

/// `columns` 简写：`column-width || column-count`（CSS Multicol）。CSSOM 序列化省略 auto 值；
/// 全 auto→`"auto"`。Chromium oracle：`columns:200px 4`→`"200px 4"`、`columns:5`→`"5"`、
/// `columns:12em`→`"192px"`（width 解析，count auto 省）、默认→`"auto"`。
fn columns_shorthand_to_css(
    width: &ColumnWidthComputedValue,
    count: &ColumnCountComputedValue,
    font_size_px: f64,
) -> String {
    let w_auto = matches!(width, ColumnWidthComputedValue::Auto);
    let c_auto = matches!(count, ColumnCountComputedValue::Auto);
    match (w_auto, c_auto) {
        (false, false) => {
            format!(
                "{} {}",
                column_width_to_css(width, font_size_px),
                column_count_to_css(count)
            )
        }
        (false, true) => column_width_to_css(width, font_size_px),
        (true, false) => column_count_to_css(count),
        (true, true) => "auto".to_string(),
    }
}

/// `column-rule` 简写：`width || style || color`。CSSOM 序列化：**style=none 时省略**（hidden 保留），
/// width 恒显（独立于 style，见 [`column_rule_width_to_css`]），color 恒显。Chromium oracle：
/// `column-rule:thick double red`→`"5px double rgb(255, 0, 0)"`、默认→`"3px rgb(0, 0,0)"`（style none 省）。
fn column_rule_shorthand_to_css(
    width: &ColumnRuleWidthComputedValue,
    style_val: &ColumnRuleStyleComputedValue,
    color: &ColorValue,
    element_color: &ColorValue,
    font_size_px: f64,
) -> String {
    let mut parts = vec![column_rule_width_to_css(width, style_val, font_size_px)];
    if !matches!(style_val, ColumnRuleStyleComputedValue::None) {
        parts.push(column_rule_style_str(style_val));
    }
    parts.push(color_to_css(&crate::resolve_color_current(color, element_color)));
    parts.join(" ")
}

/// `list-style` 简写：`position || image || type`（CSS Lists）。CSSOM 序列化恒 3 段 `"position image type"`。
/// Chromium oracle：默认→`"outside none disc"`、`list-style:square inside`→`"inside none square"`。
fn list_style_shorthand_to_css(style: &ComputedStyle) -> String {
    format!(
        "{} {} {}",
        list_style_position_str(&style.list_style_position),
        list_style_image_to_css(&style.list_style_image),
        list_style_type_str(&style.list_style_type),
    )
}

/// `text-decoration` 简写：`line || style || color || thickness`（CSS Text Decoration）。CSSOM 序列化
/// （Chromium oracle）：line=none→整值 `"none"`；否则顺序 `"line [thickness if !auto] [style if !solid]
/// [color if !currentcolor]"`。`text-decoration:underline overline wavy green 3px`→
/// `"underline overline 3px wavy rgb(0, 128, 0)"`；color 仅当显式色（非 currentcolor 关键字）才显。
fn text_decoration_shorthand_to_css(style: &ComputedStyle, element_color: &ColorValue) -> String {
    let line = text_decoration_line_to_css(&style.text_decoration_line);
    if line == "none" {
        return "none".to_string();
    }
    let mut parts = vec![line];
    let thickness = text_decoration_thickness_to_css(&style.text_decoration_thickness);
    if thickness != "auto" {
        parts.push(thickness);
    }
    let style_str = text_decoration_style_str(&style.text_decoration_style);
    if style_str != "solid" {
        parts.push(style_str);
    }
    if !matches!(style.text_decoration_color, ColorValue::CurrentColor) {
        parts.push(color_to_css(&crate::resolve_color_current(
            &style.text_decoration_color,
            element_color,
        )));
    }
    parts.join(" ")
}

/// font-variant-numeric：CSS Fonts 数字变体（9 关键字单值，对齐 Chromium 单值场景）。
/// **已知限制**：CSS 允许空格组合多值（如 `lining-nums tabular-nums`），ZeroWeb computed 值为
/// 单 enum 仅保留一个变体，故多值输入 diverge（pre-existing 解析限制，非本序列化引入）。
fn font_variant_numeric_str(v: &FontVariantNumericValue) -> String {
    match v {
        FontVariantNumericValue::Normal => "normal",
        FontVariantNumericValue::Ordinal => "ordinal",
        FontVariantNumericValue::SlashedZero => "slashed-zero",
        FontVariantNumericValue::LiningNums => "lining-nums",
        FontVariantNumericValue::OldstyleNums => "oldstyle-nums",
        FontVariantNumericValue::ProportionalNums => "proportional-nums",
        FontVariantNumericValue::TabularNums => "tabular-nums",
        FontVariantNumericValue::DiagonalFractions => "diagonal-fractions",
        FontVariantNumericValue::StackedFractions => "stacked-fractions",
    }
    .to_string()
}

fn font_variant_alternates_str(value: &FontVariantAlternatesValue) -> String {
    let FontVariantAlternatesValue::Values(value) = value else {
        return "normal".to_string();
    };
    let mut parts = Vec::new();
    if value.historical_forms {
        parts.push("historical-forms".to_string());
    }
    if let Some(name) = &value.stylistic {
        parts.push(format!("stylistic({name})"));
    }
    if !value.styleset.is_empty() {
        parts.push(format!("styleset({})", value.styleset.join(", ")));
    }
    if let Some(name) = &value.character_variant {
        parts.push(format!("character-variant({name})"));
    }
    if let Some(name) = &value.swash {
        parts.push(format!("swash({name})"));
    }
    if let Some(name) = &value.ornaments {
        parts.push(format!("ornaments({name})"));
    }
    if let Some(name) = &value.annotation {
        parts.push(format!("annotation({name})"));
    }
    parts.join(" ")
}

/// image-rendering：CSS Images 图像缩放算法（5 关键字，对齐 Chromium auto/pixelated/crisp-edges；
/// smooth/high-quality 为非标准值，ZeroWeb computed 保留 specified 关键字）。
fn image_rendering_str(r: &ImageRenderingValue) -> String {
    match r {
        ImageRenderingValue::Auto => "auto",
        ImageRenderingValue::Smooth => "smooth",
        ImageRenderingValue::HighQuality => "high-quality",
        ImageRenderingValue::Pixelated => "pixelated",
        ImageRenderingValue::CrispEdges => "crisp-edges",
    }
    .to_string()
}

/// 把数值序列化为带后缀的 CSS 量（对齐 real browser getComputedStyle 数值串）。
/// 整数无小数点（`16px`/`50%`/`0px`），小数去尾零（`19.2px`）。四舍五入到 1e-3 抑制
/// f64 累积噪声（如 em 解析 `1.2 * 16 = 19.200000000000003` → `19.2px`）。
fn format_num(v: f64, suffix: &str) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}{}", r as i64, suffix)
    } else {
        format!("{r}{suffix}")
    }
}

/// 把已计算的 [`LengthValue`] 序列化为 CSS 字符串（getComputedStyle 长度族）。
///
/// `compute_styles` 经 `resolve_computed_style` 已把主要字段的 em/rem/vw/vh/vmin/vmax/ch/非 % calc
/// 解析为 `Px`；此处对残余相对单位（resolve 未覆盖的字段如 text-indent 的 em）用 `font_size_px`
/// 兜底解析为 px。百分比/auto/关键字保留为计算值（`N%`/`auto`/`min-content`/…）；含 % 的 calc
/// 无容器尺寸无法解析为绝对值 → ''。
/// letter-spacing：Chromium 150 oracle 把 0 值（默认 / `normal` / 显式 `0`/`0px`）恒归一为
/// `"normal"`（normal 与 0 在 layout 等价）。ZW parse 已把 `normal`→`Px(0.0)`，故此处
/// `Px(0.0)`→`"normal"` 精确对齐；非 0 长度走 [`length_to_css`]。
fn letter_spacing_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    if *lv == LengthValue::Px(0.0) {
        "normal".to_string()
    } else {
        length_to_css(lv, font_size_px)
    }
}

fn length_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    // viewport 与 compute_document_styles 一致（getComputedStyle 默认 1280×800）。
    const VW: f64 = 1280.0;
    const VH: f64 = 800.0;
    match lv {
        LengthValue::Px(v) => format_num(*v, "px"),
        LengthValue::Percentage(v) => format_num(*v, "%"),
        LengthValue::Auto => "auto".to_string(),
        LengthValue::MinContent => "min-content".to_string(),
        LengthValue::MaxContent => "max-content".to_string(),
        LengthValue::FitContent(_) => "fit-content".to_string(),
        // 含 % 的 calc 无容器尺寸无法解析（非 % calc 已被 resolve_computed_style 转 Px）→ ''。
        LengthValue::Calc(_) => String::new(),
        // em/rem/vw/vh/vmin/vmax/ch：兜底解析残余相对单位为 px。
        other => format_num(
            zero_style_system::resolve_length(other, font_size_px, Some(VW), Some(VH)),
            "px",
        ),
    }
}

/// 序列化 `border-*-width` 的计算值。real browser getComputedStyle 对 border-width 返 used 值：
/// `border-style: none/hidden` → `"0px"`（ZeroWeb computed 值保留 specified 宽度供 inherit，
/// 见 `computed.rs` csswg #2768 注；gCS 须对齐 Chromium 的 used 行为，故 none/hidden 归零）。
fn border_width_to_css(width: &LengthValue, style: &BorderStyleValue, font_size_px: f64) -> String {
    match style {
        BorderStyleValue::None | BorderStyleValue::Hidden => "0px".to_string(),
        _ => length_to_css(width, font_size_px),
    }
}

/// 序列化 `max-width`/`max-height` 的计算值。ZeroWeb 用 `Px(f64::INFINITY)` 表示 initial 值
/// `none`（见 `default_impl.rs`），real browser getComputedStyle 对 max-size:none 返 `"none"`。
fn max_size_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    match lv {
        LengthValue::Px(v) if v.is_infinite() => "none".to_string(),
        _ => length_to_css(lv, font_size_px),
    }
}

fn display_value_str(d: &DisplayValue) -> String {
    match d {
        DisplayValue::Block => "block",
        DisplayValue::Inline => "inline",
        DisplayValue::InlineBlock => "inline-block",
        DisplayValue::Flex => "flex",
        DisplayValue::InlineFlex => "inline-flex",
        DisplayValue::Grid => "grid",
        DisplayValue::InlineGrid => "inline-grid",
        DisplayValue::None => "none",
        DisplayValue::Contents => "contents",
        DisplayValue::Flow => "flow",
        DisplayValue::FlowRoot => "flow-root",
        DisplayValue::ListItem => "list-item",
        DisplayValue::Table => "table",
        DisplayValue::InlineTable => "inline-table",
        DisplayValue::TableRow => "table-row",
        DisplayValue::TableCell => "table-cell",
        DisplayValue::TableCaption => "table-caption",
        DisplayValue::TableColumn => "table-column",
        DisplayValue::TableColumnGroup => "table-column-group",
        DisplayValue::TableRowGroup => "table-row-group",
        DisplayValue::TableHeaderGroup => "table-header-group",
        DisplayValue::TableFooterGroup => "table-footer-group",
    }
    .to_string()
}

fn position_value_str(p: &PositionValue) -> String {
    match p {
        PositionValue::Static => "static",
        PositionValue::Relative => "relative",
        PositionValue::Absolute => "absolute",
        PositionValue::Fixed => "fixed",
        PositionValue::Sticky => "sticky",
    }
    .to_string()
}

fn visibility_value_str(v: &VisibilityValue) -> String {
    match v {
        VisibilityValue::Visible => "visible",
        VisibilityValue::Hidden => "hidden",
        VisibilityValue::Collapse => "collapse",
    }
    .to_string()
}

// ── getComputedStyle 关键字/枚举族序列化（R2708）── 多为 variant→kebab-case 关键字直映。

fn float_value_str(f: &FloatValue) -> String {
    match f {
        FloatValue::None => "none",
        FloatValue::Left => "left",
        FloatValue::Right => "right",
        FloatValue::InlineStart => "inline-start",
        FloatValue::InlineEnd => "inline-end",
    }
    .to_string()
}

fn clear_value_str(c: &ClearValue) -> String {
    match c {
        ClearValue::None => "none",
        ClearValue::Left => "left",
        ClearValue::Right => "right",
        ClearValue::Both => "both",
        ClearValue::InlineStart => "inline-start",
        ClearValue::InlineEnd => "inline-end",
    }
    .to_string()
}

fn overflow_value_str(o: &OverflowValue) -> String {
    match o {
        OverflowValue::Visible => "visible",
        OverflowValue::Hidden => "hidden",
        OverflowValue::Scroll => "scroll",
        OverflowValue::Auto => "auto",
        OverflowValue::Clip => "clip",
    }
    .to_string()
}

fn box_sizing_str(b: &BoxSizingValue) -> String {
    match b {
        BoxSizingValue::ContentBox => "content-box",
        BoxSizingValue::BorderBox => "border-box",
    }
    .to_string()
}

fn text_align_str(a: &TextAlignValue) -> String {
    match a {
        TextAlignValue::Left => "left",
        TextAlignValue::Right => "right",
        TextAlignValue::Center => "center",
        TextAlignValue::Justify => "justify",
        TextAlignValue::Start => "start",
        TextAlignValue::End => "end",
        TextAlignValue::MatchParent => "match-parent",
    }
    .to_string()
}

fn white_space_str(w: &WhiteSpaceValue) -> String {
    match w {
        WhiteSpaceValue::Normal => "normal",
        WhiteSpaceValue::Pre => "pre",
        WhiteSpaceValue::Nowrap => "nowrap",
        WhiteSpaceValue::PreWrap => "pre-wrap",
        WhiteSpaceValue::PreLine => "pre-line",
        WhiteSpaceValue::BreakSpaces => "break-spaces",
    }
    .to_string()
}

/// real browser getComputedStyle 把 font-weight 序列化为绝对值（normal=400、bold=700）。
/// `bolder`/`lighter` 已在 style-system computed 阶段按父链解析；对应分支仅作防御性回退。
fn font_weight_str(w: &FontWeightValue) -> String {
    match w {
        FontWeightValue::Absolute(n) => n.to_string(),
        FontWeightValue::Normal => "400".to_string(),
        FontWeightValue::Bold => "700".to_string(),
        FontWeightValue::Bolder => "bolder".to_string(),
        FontWeightValue::Lighter => "lighter".to_string(),
    }
}

fn font_style_str(s: &FontStyleValue) -> String {
    match s {
        FontStyleValue::Normal => "normal".to_string(),
        FontStyleValue::Italic => "italic".to_string(),
        FontStyleValue::Oblique(None) => "oblique".to_string(),
        FontStyleValue::Oblique(Some(deg)) => format!("oblique {deg}deg"),
    }
}

/// line-height：normal→`normal`；number→解析为 used px（`font-size × number`，对齐 Chromium
/// getComputedStyle 返 used 值，R2761 修旧返无单位数 diverge）；Length→px。
fn line_height_str(lh: &LineHeightValue, font_size_px: f64) -> String {
    match lh {
        LineHeightValue::Normal => "normal".to_string(),
        LineHeightValue::Number(n) => format_num(font_size_px * *n, "px"),
        LineHeightValue::Length(lv) => length_to_css(lv, font_size_px),
    }
}

fn z_index_str(z: &ZIndexValue) -> String {
    match z {
        ZIndexValue::Auto => "auto".to_string(),
        ZIndexValue::Integer(n) => n.to_string(),
    }
}

fn cursor_str(c: &CursorValue) -> String {
    match c {
        CursorValue::Auto => "auto",
        CursorValue::Default => "default",
        CursorValue::Pointer => "pointer",
        CursorValue::Move => "move",
        CursorValue::Text => "text",
        CursorValue::Wait => "wait",
        CursorValue::Crosshair => "crosshair",
        CursorValue::Help => "help",
        CursorValue::NotAllowed => "not-allowed",
        CursorValue::Grab => "grab",
        CursorValue::Grabbing => "grabbing",
        CursorValue::ColResize => "col-resize",
        CursorValue::RowResize => "row-resize",
        CursorValue::NsResize => "ns-resize",
        CursorValue::EwResize => "ew-resize",
        CursorValue::None => "none",
        CursorValue::Progress => "progress",
        CursorValue::Cell => "cell",
        CursorValue::Copy => "copy",
        CursorValue::Alias => "alias",
        CursorValue::AllScroll => "all-scroll",
        CursorValue::ZoomIn => "zoom-in",
        CursorValue::ZoomOut => "zoom-out",
    }
    .to_string()
}

fn text_transform_str(t: &TextTransformValue) -> String {
    match t {
        TextTransformValue::None => "none",
        TextTransformValue::Uppercase => "uppercase",
        TextTransformValue::Lowercase => "lowercase",
        TextTransformValue::Capitalize => "capitalize",
        TextTransformValue::FullWidth => "full-width",
        TextTransformValue::FullSizeKana => "full-size-kana",
    }
    .to_string()
}

/// `text-decoration-line`：多 flag 组合（CSS Text Decoration §3.1）。规范序
/// `underline overline line-through`，空（含 obsolete blink）→ `none`。对齐 Chromium
/// getComputedStyle（`text-decoration: overline underline` → `text-decoration-line` 重组为
/// `underline overline`）。
fn text_decoration_line_to_css(l: &TextDecorationLineValue) -> String {
    let mut parts = Vec::new();
    if l.underline {
        parts.push("underline");
    }
    if l.overline {
        parts.push("overline");
    }
    if l.line_through {
        parts.push("line-through");
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" ")
    }
}

/// `text-decoration-style`：5 关键字（CSS Text Decoration §3.2，初值 solid）。
fn text_decoration_style_str(s: &TextDecorationStyleValue) -> String {
    match s {
        TextDecorationStyleValue::Solid => "solid",
        TextDecorationStyleValue::Double => "double",
        TextDecorationStyleValue::Dotted => "dotted",
        TextDecorationStyleValue::Dashed => "dashed",
        TextDecorationStyleValue::Wavy => "wavy",
    }
    .to_string()
}

/// `text-decoration-thickness`（CSS Text Decoration 4 §2.3）：auto/from-font 或长度（px）。
fn text_decoration_thickness_to_css(t: &TextDecorationThicknessValue) -> String {
    match t {
        TextDecorationThicknessValue::Auto => "auto".to_string(),
        TextDecorationThicknessValue::Length(px) => format_num(*px, "px"),
    }
}

/// text-underline-offset：CSS Text Decoration 4 §2.5。Auto→`auto`；Length→px（经 length_to_css
/// 解析残余相对单位）。Chromium 150 oracle：`3px`→`"3px"`、默认→`"auto"`。
fn text_underline_offset_to_css(o: &zero_css_parser::values::TextUnderlineOffsetValue, font_size_px: f64) -> String {
    use zero_css_parser::values::TextUnderlineOffsetValue as T;
    match o {
        T::Auto => "auto".to_string(),
        T::Length(lv) => length_to_css(lv, font_size_px),
    }
}

/// text-emphasis-style：ZW 存解析后的标记 char（`TextEmphasisStyleValue::Char`），序列化时**逆映射**
/// 回 CSS keyword 形（`parse_text_emphasis_style` 用标准 CSS 字符，见 parse_misc.rs:530）。
/// filled 省略（初值），open 显；非 10 个标准字符→`<string>` 引号化。Chromium 150 oracle：
/// `dot`→`"dot"`、`open circle`→`"open circle"`、`sesame`→`"sesame"`、`"*"`→`"\"*\""`、默认→`"none"`。
fn text_emphasis_style_to_css(s: &TextEmphasisStyleValue) -> String {
    match s {
        TextEmphasisStyleValue::None => "none".to_string(),
        TextEmphasisStyleValue::Char(c) => match *c {
            '\u{2022}' => "dot".to_string(),                // • filled dot
            '\u{25E6}' => "open dot".to_string(),           // ◦
            '\u{25CF}' => "circle".to_string(),             // ● filled circle
            '\u{25CB}' => "open circle".to_string(),        // ○
            '\u{25C9}' => "double-circle".to_string(),      // ◉ filled
            '\u{25CE}' => "open double-circle".to_string(), // ◎
            '\u{25B2}' => "triangle".to_string(),           // ▲ filled
            '\u{25B3}' => "open triangle".to_string(),      // △
            '\u{FE45}' => "sesame".to_string(),             // ﹅ filled
            '\u{FE46}' => "open sesame".to_string(),        // ﹆
            other => css_string_to_css(&other.to_string()), // <string>（非标准字符）
        },
    }
}

/// text-emphasis-position：CSS Text Decoration 3 §3.2。over/under 恒显；left 显（right 初值省）。
/// Chromium 150 oracle：默认 over right→`"over"`、`under left`→`"under left"`。
fn text_emphasis_position_to_css(p: &TextEmphasisPositionValue) -> String {
    match p {
        TextEmphasisPositionValue::OverRight => "over",
        TextEmphasisPositionValue::OverLeft => "over left",
        TextEmphasisPositionValue::UnderRight => "under",
        TextEmphasisPositionValue::UnderLeft => "under left",
    }
    .to_string()
}

fn text_overflow_str(t: &TextOverflowValue) -> String {
    match t {
        TextOverflowValue::Clip => "clip".to_string(),
        TextOverflowValue::Ellipsis => "ellipsis".to_string(),
        TextOverflowValue::String(s) => s.clone(),
    }
}

fn direction_str(d: &DirectionValue) -> String {
    match d {
        DirectionValue::Ltr => "ltr",
        DirectionValue::Rtl => "rtl",
    }
    .to_string()
}

fn border_collapse_str(b: &BorderCollapseValue) -> String {
    match b {
        BorderCollapseValue::Separate => "separate",
        BorderCollapseValue::Collapse => "collapse",
    }
    .to_string()
}

fn table_layout_str(t: &TableLayoutValue) -> String {
    match t {
        TableLayoutValue::Auto => "auto",
        TableLayoutValue::Fixed => "fixed",
    }
    .to_string()
}

fn caption_side_str(c: &CaptionSideValue) -> String {
    match c {
        CaptionSideValue::Top => "top",
        CaptionSideValue::Bottom => "bottom",
    }
    .to_string()
}

fn border_style_str(s: &BorderStyleValue) -> String {
    match s {
        BorderStyleValue::None => "none",
        BorderStyleValue::Hidden => "hidden",
        BorderStyleValue::Dotted => "dotted",
        BorderStyleValue::Dashed => "dashed",
        BorderStyleValue::Solid => "solid",
        BorderStyleValue::Double => "double",
        BorderStyleValue::Groove => "groove",
        BorderStyleValue::Ridge => "ridge",
        BorderStyleValue::Inset => "inset",
        BorderStyleValue::Outset => "outset",
    }
    .to_string()
}

fn outline_style_str(s: &OutlineStyleValue) -> String {
    match s {
        OutlineStyleValue::None => "none",
        OutlineStyleValue::Dotted => "dotted",
        OutlineStyleValue::Dashed => "dashed",
        OutlineStyleValue::Solid => "solid",
        OutlineStyleValue::Double => "double",
        OutlineStyleValue::Groove => "groove",
        OutlineStyleValue::Ridge => "ridge",
        OutlineStyleValue::Inset => "inset",
        OutlineStyleValue::Outset => "outset",
        OutlineStyleValue::Auto => "auto",
    }
    .to_string()
}

// ── 复合枚举/列表族序列化（R2710）──

/// font-family：逗号分隔，每个族名按 CSS ident 规则决定是否加引号（对齐 Chromium：
/// `"Helvetica Neue", Arial, sans-serif`）。空 Vec → ''（UA 默认字体未入 computed）。
fn font_family_to_css(families: &[String]) -> String {
    families
        .iter()
        .map(|f| {
            let bare = f.trim_matches('"').trim_matches('\'');
            quote_font_family(bare)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `font` 简写：CSS Fonts。CSSOM 序列化 `<style> <variant> <weight> <stretch> <size>[/<line-height>]
/// <family>`，省初值（style normal / weight 400 / line-height normal）。Chromium 150 oracle：
/// `font:italic bold 14px/1.5 Arial`→`"italic 700 14px / 21px Arial"`、默认→`"16px \"Times New Roman\""`。
fn font_shorthand_to_css(style: &ComputedStyle, font_size_px: f64) -> String {
    let mut parts = Vec::new();
    let fs = font_style_str(&style.font_style);
    if fs != "normal" {
        parts.push(fs);
    }
    let fw = font_weight_str(&style.font_weight);
    if fw != "400" {
        parts.push(fw);
    }
    if (style.font_stretch - 100.0).abs() > f32::EPSILON {
        parts.push(format_num(style.font_stretch as f64, "%"));
    }
    // size 恒显；line-height 非 normal 显作 "size / lh"。
    let size = length_to_css(&style.font_size, font_size_px);
    let lh = line_height_str(&style.line_height, font_size_px);
    if lh != "normal" {
        parts.push(format!("{size} / {lh}"));
    } else {
        parts.push(size);
    }
    parts.push(font_family_to_css(&style.font_family));
    parts.join(" ")
}

/// 族名是合法 CSS 标识符（非空、首字符非数字/`-数字`、仅 ident 字符）→ 不引号；否则双引号
///（转义内嵌 `"`、`\`）。对齐 real browser 的 font-family 序列化。
fn quote_font_family(name: &str) -> String {
    if is_css_ident(name) {
        name.to_string()
    } else {
        let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }
}

/// 判断字符串是否为合法 CSS 标识符（font-family 不引号条件）。非空；首字符为字母/下划线/
/// 连字符（但非 `-` 后跟数字，如 `-1`）；其余字符为字母/数字/`-`/`_`。
fn is_css_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    // 首字符：字母/_；或 `-` 后非数字（CSS：`-` 须后跟 ident 字符，不能直接跟数字）。
    let first_ok = first.is_ascii_alphabetic()
        || first == '_'
        || (first == '-' && !matches!(chars.clone().next(), Some(c) if c.is_ascii_digit()));
    first_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn flex_direction_str(d: &FlexDirectionValue) -> String {
    match d {
        FlexDirectionValue::Row => "row",
        FlexDirectionValue::RowReverse => "row-reverse",
        FlexDirectionValue::Column => "column",
        FlexDirectionValue::ColumnReverse => "column-reverse",
    }
    .to_string()
}

fn flex_wrap_str(w: &FlexWrapValue) -> String {
    match w {
        FlexWrapValue::Nowrap => "nowrap",
        FlexWrapValue::Wrap => "wrap",
        FlexWrapValue::WrapReverse => "wrap-reverse",
    }
    .to_string()
}

/// justify-content / align-items / align-self 共用（CSS Box Align 3）。
fn alignment_str(a: &AlignmentValue) -> String {
    match a {
        AlignmentValue::Auto => "auto",
        AlignmentValue::Normal => "normal",
        AlignmentValue::FlexStart => "flex-start",
        AlignmentValue::FlexEnd => "flex-end",
        AlignmentValue::Center => "center",
        AlignmentValue::SpaceBetween => "space-between",
        AlignmentValue::SpaceAround => "space-around",
        AlignmentValue::SpaceEvenly => "space-evenly",
        AlignmentValue::Stretch => "stretch",
        AlignmentValue::Start => "start",
        AlignmentValue::End => "end",
        AlignmentValue::Baseline => "baseline",
    }
    .to_string()
}

fn writing_mode_str(w: &WritingModeValue) -> String {
    match w {
        WritingModeValue::HorizontalTb => "horizontal-tb",
        WritingModeValue::VerticalRl => "vertical-rl",
        WritingModeValue::VerticalLr => "vertical-lr",
    }
    .to_string()
}

fn object_fit_str(o: &ObjectFitComputedValue) -> String {
    match o {
        ObjectFitComputedValue::Fill => "fill",
        ObjectFitComputedValue::Contain => "contain",
        ObjectFitComputedValue::Cover => "cover",
        ObjectFitComputedValue::None => "none",
        ObjectFitComputedValue::ScaleDown => "scale-down",
    }
    .to_string()
}

fn isolation_str(i: &IsolationValue) -> String {
    match i {
        IsolationValue::Auto => "auto",
        IsolationValue::Isolate => "isolate",
    }
    .to_string()
}

fn mix_blend_mode_str(m: &MixBlendModeComputedValue) -> String {
    match m {
        MixBlendModeComputedValue::Normal => "normal",
        MixBlendModeComputedValue::Multiply => "multiply",
        MixBlendModeComputedValue::Screen => "screen",
        MixBlendModeComputedValue::Overlay => "overlay",
        MixBlendModeComputedValue::Darken => "darken",
        MixBlendModeComputedValue::Lighten => "lighten",
        MixBlendModeComputedValue::ColorDodge => "color-dodge",
        MixBlendModeComputedValue::ColorBurn => "color-burn",
        MixBlendModeComputedValue::HardLight => "hard-light",
        MixBlendModeComputedValue::SoftLight => "soft-light",
        MixBlendModeComputedValue::Difference => "difference",
        MixBlendModeComputedValue::Exclusion => "exclusion",
        MixBlendModeComputedValue::Hue => "hue",
        MixBlendModeComputedValue::Saturation => "saturation",
        MixBlendModeComputedValue::Color => "color",
        MixBlendModeComputedValue::Luminosity => "luminosity",
    }
    .to_string()
}

fn pointer_events_str(p: &PointerEventsValue) -> String {
    match p {
        PointerEventsValue::Auto => "auto",
        PointerEventsValue::None => "none",
        PointerEventsValue::VisiblePainted => "visiblePainted",
        PointerEventsValue::VisibleFill => "visibleFill",
        PointerEventsValue::VisibleStroke => "visibleStroke",
        PointerEventsValue::Visible => "visible",
        PointerEventsValue::Painted => "painted",
        PointerEventsValue::Fill => "fill",
        PointerEventsValue::Stroke => "stroke",
        PointerEventsValue::All => "all",
        PointerEventsValue::Inherit => "inherit",
    }
    .to_string()
}

fn user_select_str(u: &UserSelectValue) -> String {
    match u {
        UserSelectValue::Auto => "auto",
        UserSelectValue::Text => "text",
        UserSelectValue::None => "none",
        UserSelectValue::All => "all",
        UserSelectValue::Contain => "contain",
    }
    .to_string()
}

/// list-style-type：~28 builtin 关键字 + Custom(name)（`@counter-style` 名，不引号）+
/// String(s)（`<string>` 标记，双引号）。对齐 Chromium getComputedStyle 序列化。
fn list_style_type_str(t: &ListStyleTypeValue) -> String {
    match t {
        ListStyleTypeValue::Disc => "disc".into(),
        ListStyleTypeValue::Circle => "circle".into(),
        ListStyleTypeValue::Square => "square".into(),
        ListStyleTypeValue::Decimal => "decimal".into(),
        ListStyleTypeValue::DecimalLeadingZero => "decimal-leading-zero".into(),
        ListStyleTypeValue::LowerRoman => "lower-roman".into(),
        ListStyleTypeValue::UpperRoman => "upper-roman".into(),
        ListStyleTypeValue::LowerAlpha => "lower-alpha".into(),
        ListStyleTypeValue::UpperAlpha => "upper-alpha".into(),
        ListStyleTypeValue::LowerGreek => "lower-greek".into(),
        ListStyleTypeValue::Persian => "persian".into(),
        ListStyleTypeValue::Armenian => "armenian".into(),
        ListStyleTypeValue::LowerArmenian => "lower-armenian".into(),
        ListStyleTypeValue::Georgian => "georgian".into(),
        ListStyleTypeValue::Hebrew => "hebrew".into(),
        ListStyleTypeValue::ArabicIndic => "arabic-indic".into(),
        ListStyleTypeValue::Devanagari => "devanagari".into(),
        ListStyleTypeValue::Bengali => "bengali".into(),
        ListStyleTypeValue::Gujarati => "gujarati".into(),
        ListStyleTypeValue::Gurmukhi => "gurmukhi".into(),
        ListStyleTypeValue::Kannada => "kannada".into(),
        ListStyleTypeValue::Malayalam => "malayalam".into(),
        ListStyleTypeValue::Tamil => "tamil".into(),
        ListStyleTypeValue::Telugu => "telugu".into(),
        ListStyleTypeValue::Lao => "lao".into(),
        ListStyleTypeValue::Khmer => "khmer".into(),
        ListStyleTypeValue::Myanmar => "myanmar".into(),
        ListStyleTypeValue::CjkDecimal => "cjk-decimal".into(),
        ListStyleTypeValue::None => "none".into(),
        ListStyleTypeValue::Custom(name) => name.clone(),
        ListStyleTypeValue::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
    }
}

fn list_style_position_str(p: &ListStylePositionValue) -> String {
    match p {
        ListStylePositionValue::Outside => "outside",
        ListStylePositionValue::Inside => "inside",
    }
    .to_string()
}

// ── 数值/special 族序列化（R2711）──

/// flex-basis：auto/content/length（em 已在 resolve 阶段转 Px）。
fn flex_basis_str(b: &FlexBasisValue, font_size_px: f64) -> String {
    match b {
        FlexBasisValue::Auto => "auto".to_string(),
        FlexBasisValue::Content => "content".to_string(),
        FlexBasisValue::Length(lv) => length_to_css(lv, font_size_px),
    }
}

/// aspect-ratio：None/auto → `auto`；Some(r) 无 auto → 数值；Some(r) + auto → `auto <r>`。
///
/// **已知 diverge**：ZeroWeb 只存合并比值（`Option<f32>`），不保留原始 `w / h`——故
/// `aspect-ratio: 16 / 9`（Chrome 返 `"16 / 9"`）序列化为 `"1.7778"`。单数值（`2`）与 auto
/// 路径与 Chromium 一致。
fn aspect_ratio_str(ratio: Option<f32>, auto: bool) -> String {
    match (ratio, auto) {
        (None, _) => "auto".to_string(),
        (Some(r), false) => format_num(r as f64, ""),
        (Some(r), true) => format!("auto {}", format_num(r as f64, "")),
    }
}

/// transform：按 CSS Transforms L1/L2 计算值序列化为**函数列表**（none / 空格分隔函数）。
///
/// **已知 diverge**：Chromium getComputedStyle 对非 none transform 返 **resolved matrix**
///（`matrix(...)` / `matrix3d(...)`）；CSS 规范（L1/L2）+ Firefox 返函数列表（长度解析为 px、
/// 百分比保留——border-box 相对，须 layout 故保 `%`）。ZeroWeb 返函数列表（spec-correct）。
fn transform_to_css(t: &TransformValue) -> String {
    match t {
        TransformValue::None => "none".to_string(),
        TransformValue::List(fns) => fns.iter().map(transform_function_to_css).collect::<Vec<_>>().join(" "),
    }
}

/// transform-origin：按 CSS Transforms L1 计算值序列化为两个长度（空格连接）。
///
/// 存储为 `transform_origin_x`/`transform_origin_y` 两个 `LengthValue`，默认均为 `50%`
///（即关键字 `center` 的计算值）。经 [`length_to_css`]：px 指定值精确、百分比保留为 `N%`。
///
/// **已知 diverge（同 transform / width·height used-value）**：Chromium getComputedStyle 对
/// transform-origin 返 **used** 值（border-box 中心绝对 px，如 `100px 50px`）；CSS 规范 + Firefox
/// 返**计算值**（百分比/指定 px）。ZeroWeb 返计算值（spec-correct）。单值 `center`/`top`/`left` 等
/// 轴感知 `<position>` 关键字语法解析为独立 follow-up（当前 apply 仅解析长度，关键字降级为默认
/// `50% 50%`，恰好等于 `center` 计算值，故 `center` 行为正确）。
fn transform_origin_to_css(x: &LengthValue, y: &LengthValue, font_size_px: f64) -> String {
    format!("{} {}", length_to_css(x, font_size_px), length_to_css(y, font_size_px))
}

/// contain：按 CSS Containment L1/L2 计算值序列化。
///
/// `None`/`Strict`/`Content`/`Size`/`Layout`/`Style`/`Paint` → 对应关键字（`strict`/`content`
/// 保留 shorthand 不展开，与 Chromium getComputedStyle 一致）；`Custom(u8)` 位掩码按 spec 语法序
///（`size || layout || paint || style`）解码为空格分隔关键字列表。空位掩码→`none`（防御性，
/// parser 正常不产 Custom(0)）。
fn contain_to_css(c: &ContainComputedValue) -> String {
    use ContainComputedValue as C;
    match c {
        C::None => "none".to_string(),
        C::Strict => "strict".to_string(),
        C::Content => "content".to_string(),
        C::Size => "size".to_string(),
        C::Layout => "layout".to_string(),
        C::Style => "style".to_string(),
        C::Paint => "paint".to_string(),
        C::Custom(flags) => {
            let mut parts: Vec<&str> = Vec::new();
            if (flags & C::FLAG_SIZE) != 0 {
                parts.push("size");
            }
            if (flags & C::FLAG_LAYOUT) != 0 {
                parts.push("layout");
            }
            if (flags & C::FLAG_PAINT) != 0 {
                parts.push("paint");
            }
            if (flags & C::FLAG_STYLE) != 0 {
                parts.push("style");
            }
            if parts.is_empty() {
                "none".to_string()
            } else {
                parts.join(" ")
            }
        }
    }
}

/// filter：按 CSS Filter Effects 计算值序列化为**函数列表**（none / 空格分隔函数）。
///
/// 空 `Vec`（= `filter: none`，见 `parse_filter_list`）→ `none`；否则各函数空格连接。
/// 长度函数 `blur` 为 px；数值函数（brightness/contrast/...）无单位；`hue-rotate` 为 `deg`；
/// `drop-shadow` 颜色经 [`crate::resolve_color_current`] 解析（currentcolor→元素计算 color）后串行化。
fn filter_to_css(filters: &[FilterComputedValue], element_color: &ColorValue) -> String {
    if filters.is_empty() {
        return "none".to_string();
    }
    filters
        .iter()
        .map(|f| filter_function_to_css(f, element_color))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 序列化单个 [`FilterComputedValue`] 函数。
fn filter_function_to_css(f: &FilterComputedValue, element_color: &ColorValue) -> String {
    use FilterComputedValue as F;
    match f {
        F::None => "none".to_string(),
        F::Blur(n) => format!("blur({})", format_num(*n as f64, "px")),
        F::Brightness(n) => format!("brightness({})", format_num(*n as f64, "")),
        F::Contrast(n) => format!("contrast({})", format_num(*n as f64, "")),
        F::Grayscale(n) => format!("grayscale({})", format_num(*n as f64, "")),
        F::HueRotate(n) => format!("hue-rotate({}deg)", format_num(*n as f64, "")),
        F::Invert(n) => format!("invert({})", format_num(*n as f64, "")),
        F::Opacity(n) => format!("opacity({})", format_num(*n as f64, "")),
        F::Saturate(n) => format!("saturate({})", format_num(*n as f64, "")),
        F::Sepia(n) => format!("sepia({})", format_num(*n as f64, "")),
        F::DropShadow(x, y, blur, color) => format!(
            "drop-shadow({} {} {} {})",
            format_num(*x as f64, "px"),
            format_num(*y as f64, "px"),
            format_num(*blur as f64, "px"),
            color_to_css(&crate::resolve_color_current(color, element_color)),
        ),
    }
}

/// transform-style：CSS Transforms 2 计算值（flat / preserve-3d）。
fn transform_style_str(t: &TransformStyleValue) -> String {
    match t {
        TransformStyleValue::Flat => "flat",
        TransformStyleValue::Preserve3d => "preserve-3d",
    }
    .to_string()
}

/// backface-visibility：CSS Transforms 2 计算值（visible / hidden）。
fn backface_visibility_str(b: &BackfaceVisibilityValue) -> String {
    match b {
        BackfaceVisibilityValue::Visible => "visible",
        BackfaceVisibilityValue::Hidden => "hidden",
    }
    .to_string()
}

/// perspective：CSS Transforms 2 计算值。ZeroWeb 用 `Px(0.0)` 表示 initial 值 `none`
///（见 `default_impl.rs` / `apply` 的 `none→Px(0.0)`），real browser getComputedStyle 对
/// perspective:none 返 `"none"`（对齐 [`max_size_to_css`] 的 INFINITY→none 模式）。
fn perspective_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    match lv {
        LengthValue::Px(v) if *v == 0.0 => "none".to_string(),
        _ => length_to_css(lv, font_size_px),
    }
}

/// will-change：CSS Will Change 计算值。空 `Vec`（默认 + `will-change: auto`，见
/// `parse_will_change_list`）→ `auto`；否则各标识符空格连接（`scroll-position` / `contents` /
/// 自定义属性名原样）。对齐 Chromium getComputedStyle。
fn will_change_to_css(list: &[WillChangeValue]) -> String {
    if list.is_empty() {
        return "auto".to_string();
    }
    list.iter()
        .map(|v| match v {
            WillChangeValue::Auto => "auto",
            WillChangeValue::ScrollPosition => "scroll-position",
            WillChangeValue::Contents => "contents",
            WillChangeValue::Custom(s) => s.as_str(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// clip-path：按 CSS Masking 计算值序列化 basic-shape 函数。
///
/// `None`→`none`；`inset()` 四内缩按 box 简写折叠（1/2/3/4 值，同 margin 语法）+ 可选 `round`；
/// `circle()`/`ellipse()` 半径 + 可选 ` at <pos>`；`polygon()` 填充规则（仅 evenodd 输出）+ 逗号分隔顶点。
/// 长度经 [`length_to_css`]；半径关键字 closest-side/farthest-side 原样。
fn clip_path_to_css(c: &ClipPathValue, font_size_px: f64) -> String {
    use ClipPathValue as C;
    match c {
        C::None => "none".to_string(),
        C::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            let mut inner = box_4_to_css(top, right, bottom, left, font_size_px);
            if let Some(r) = round {
                inner.push_str(&format!(" round {}", clip_path_radius_to_css(r, font_size_px)));
            }
            format!("inset({inner})")
        }
        C::Circle { radius, position } => {
            let mut inner = clip_path_radius_to_css(radius, font_size_px);
            if let Some((x, y)) = position {
                inner.push_str(&format!(
                    " at {} {}",
                    length_to_css(x, font_size_px),
                    length_to_css(y, font_size_px)
                ));
            }
            format!("circle({inner})")
        }
        C::Ellipse { rx, ry, position } => {
            let mut inner = format!(
                "{} {}",
                clip_path_radius_to_css(rx, font_size_px),
                clip_path_radius_to_css(ry, font_size_px)
            );
            if let Some((x, y)) = position {
                inner.push_str(&format!(
                    " at {} {}",
                    length_to_css(x, font_size_px),
                    length_to_css(y, font_size_px)
                ));
            }
            format!("ellipse({inner})")
        }
        C::Polygon { fill_rule, points } => {
            let pts = points
                .iter()
                .map(|(x, y)| format!("{} {}", length_to_css(x, font_size_px), length_to_css(y, font_size_px)))
                .collect::<Vec<_>>()
                .join(", ");
            match fill_rule {
                PolygonFillRule::NonZero => format!("polygon({pts})"),
                PolygonFillRule::EvenOdd => format!("polygon(evenodd, {pts})"),
            }
        }
    }
}

/// 序列化 clip-path 半径（circle/ellipse）。具体长度经 [`length_to_css`]；关键字原样。
fn clip_path_radius_to_css(r: &ClipPathRadius, font_size_px: f64) -> String {
    match r {
        ClipPathRadius::Length(lv) => length_to_css(lv, font_size_px),
        ClipPathRadius::ClosestSide => "closest-side".to_string(),
        ClipPathRadius::FarthestSide => "farthest-side".to_string(),
    }
}

/// 把 4 个 box 维度（top right bottom left，如 inset 内缩 / margin）序列化为 CSS box 简写：
/// 全等→1 值；top==bottom && left==right→2 值；left==right→3 值；否则 4 值（同 margin 语法）。
/// 比较按序列化后的字符串（等价 LengthValue 必序列化等价）。
fn box_4_to_css(
    top: &LengthValue,
    right: &LengthValue,
    bottom: &LengthValue,
    left: &LengthValue,
    font_size_px: f64,
) -> String {
    let t = length_to_css(top, font_size_px);
    let r = length_to_css(right, font_size_px);
    let b = length_to_css(bottom, font_size_px);
    let l = length_to_css(left, font_size_px);
    if t == r && r == b && b == l {
        t
    } else if t == b && r == l {
        format!("{t} {r}")
    } else if r == l {
        format!("{t} {r} {b}")
    } else {
        format!("{t} {r} {b} {l}")
    }
}

/// `flex` 简写序列化：`"<grow> <shrink> <basis>"`（CSS Flexbox §7.1，恒 3 段）。
/// 对齐 Chromium getComputedStyle（`flex: 2 1 50px`→`"2 1 50px"` / `flex: 1`→`"1 1 0%"` /
/// default→`"0 1 auto"`）。basis 经 [`flex_basis_str`]（Auto/content/length/percentage）。
fn flex_shorthand_to_css(basis: &FlexBasisValue, font_size_px: f64, grow: f64, shrink: f64) -> String {
    format!(
        "{} {} {}",
        format_num(grow, ""),
        format_num(shrink, ""),
        flex_basis_str(basis, font_size_px)
    )
}

/// 单边 border 简写（border-top/right/bottom/left）：`"<width> <style> <color>"`。
/// width 经 [`border_width_to_css`]（border-style:none/hidden→used "0px"，对齐 Chromium）；
/// color 经 currentcolor 解析。Chromium oracle：`border-top: 3px dashed blue`→`"3px dashed rgb(0,0,255)"`。
fn border_side_shorthand(
    width: &LengthValue,
    style_val: &BorderStyleValue,
    color: &ColorValue,
    element_color: &ColorValue,
    font_size_px: f64,
) -> String {
    format!(
        "{} {} {}",
        border_width_to_css(width, style_val, font_size_px),
        border_style_str(style_val),
        color_to_css(&crate::resolve_color_current(color, element_color)),
    )
}

/// `border` 全边简写：仅当 4 边的 width/style/color 序列化串全等时返该串，否则 `""`
/// （CSSOM：longhand 不一致时简写不可序列化；Chromium oracle：border-top:1px + border-bottom:2px
/// → `border=""`，border:3px dashed blue → `"3px dashed rgb(0,0,255)"`）。
fn border_shorthand(style: &ComputedStyle, element_color: &ColorValue, font_size_px: f64) -> String {
    let top = border_side_shorthand(
        &style.border_top_width,
        &style.border_top_style,
        &style.border_top_color,
        element_color,
        font_size_px,
    );
    let right = border_side_shorthand(
        &style.border_right_width,
        &style.border_right_style,
        &style.border_right_color,
        element_color,
        font_size_px,
    );
    let bottom = border_side_shorthand(
        &style.border_bottom_width,
        &style.border_bottom_style,
        &style.border_bottom_color,
        element_color,
        font_size_px,
    );
    let left = border_side_shorthand(
        &style.border_left_width,
        &style.border_left_style,
        &style.border_left_color,
        element_color,
        font_size_px,
    );
    if top == right && right == bottom && bottom == left {
        top
    } else {
        String::new()
    }
}

/// transition/animation 列表族通用序列化：空列表→`default`（各 CSS 初值），否则逗号分隔。
/// 对齐 Chromium getComputedStyle（`transition-property: margin, padding` → `margin, padding`）。
fn string_list_to_css(list: &[String], default: &str) -> String {
    if list.is_empty() {
        default.to_string()
    } else {
        list.join(", ")
    }
}

/// 时间列表（transition/animation duration·delay，f64 秒）：空→`default`（`0s`），
/// 否则逗号分隔 `Ns`（整数无小数点 `2s` / 小数去尾零 `0.3s`，经 [`format_num`]）。
fn time_list_to_css(list: &[f64], default: &str) -> String {
    if list.is_empty() {
        default.to_string()
    } else {
        list.iter().map(|t| format_num(*t, "s")).collect::<Vec<_>>().join(", ")
    }
}

/// animation-iteration-count 列表（`Vec<Option<f64>>`）：空→`1`（CSS 初值）；
/// None→`infinite`，Some(n)→无后缀数（整数 `2` / 小数 `2.5`）。
fn iter_count_list_to_css(list: &[Option<f64>]) -> String {
    if list.is_empty() {
        "1".to_string()
    } else {
        list.iter()
            .map(|c| match c {
                None => "infinite".to_string(),
                Some(n) => format_num(*n, ""),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// transition/animation 枚举列表（direction/fill-mode/play-state）通用序列化：
/// 空→`default`（CSS 初值），否则逐元素经 `f` 映射关键字后逗号分隔。
fn enum_list_to_css<T>(list: &[T], default: &str, f: impl Fn(&T) -> String) -> String {
    if list.is_empty() {
        default.to_string()
    } else {
        list.iter().map(f).collect::<Vec<_>>().join(", ")
    }
}

/// `background` 简写：CSS Backgrounds 3。CSSOM 序列化恒为完整规范形（**无省略**）：
/// `"<color> <image> <repeat> <attachment> <position> / <size> <origin> <clip>"`。
/// Chromium 150 oracle：默认→`"rgba(0, 0, 0, 0) none repeat scroll 0% 0% / auto padding-box border-box"`；
/// origin/clip 即使相等也恒双显（`content-box content-box`）。复用各 longhand 序列化重组。
/// **已知限制**：ZW 对 attachment/clip/origin 存单值（非多层 Vec），故**多层** background 无法
/// 正确 round-trip（单层正确；多层 longhand 亦单值，diverge 一致）。
fn background_shorthand_to_css(style: &ComputedStyle, element_color: &ColorValue, font_size_px: f64) -> String {
    let color = color_to_css(&crate::resolve_color_current(&style.background_color, element_color));
    let image = image_layer_list_to_css(&style.background_image, element_color, font_size_px);
    // Vec 族 longhand 空 Vec 时返 ''，用 CSS 初值兜底（default_impl 实际已填充，防御极端路径）。
    let repeat = {
        let s = background_repeat_to_css(&style.background_repeat);
        if s.is_empty() { "repeat".to_string() } else { s }
    };
    let attachment = background_attachment_to_css(&style.background_attachment);
    let position = {
        let s = background_position_to_css(&style.background_position);
        if s.is_empty() { "0% 0%".to_string() } else { s }
    };
    let size = {
        let s = background_size_to_css(&style.background_size);
        if s.is_empty() { "auto".to_string() } else { s }
    };
    let origin = background_origin_to_css(&style.background_origin);
    let clip = background_clip_to_css(&style.background_clip);
    format!("{color} {image} {repeat} {attachment} {position} / {size} {origin} {clip}")
}

/// `transition` 简写：`<single-transition>#`（CSS Transitions）。CSSOM 序列化（Chromium 150 oracle）：
/// 每条目 = property / duration / timing-function / delay，各 longhand 省初值；property=all 仅在
/// 其余 longhand 全初值时显（避免空串）。`transition:margin 2s ease-in 1s`→`"margin 2s ease-in 1s"`；
/// 默认（空列表）→`"all"`；`transition:none`→`"none"`。4 longhand 列表等长（`expand_transition`
/// 保证），逐索引 zip 后逗号连接。
fn transition_shorthand_to_css(style: &ComputedStyle) -> String {
    let n = style.transition_property.len();
    if n == 0 {
        return "all".to_string();
    }
    (0..n)
        .map(|i| {
            let prop = style.transition_property[i].as_str();
            let dur = format_num(style.transition_duration.get(i).copied().unwrap_or(0.0), "s");
            let tf = style
                .transition_timing_function
                .get(i)
                .map(timing_function_to_css)
                .unwrap_or_else(|| "ease".to_string());
            let delay = format_num(style.transition_delay.get(i).copied().unwrap_or(0.0), "s");
            let all_rest_initial = dur == "0s" && tf == "ease" && delay == "0s";
            let mut parts = Vec::new();
            if prop != "all" || all_rest_initial {
                parts.push(prop.to_string());
            }
            if dur != "0s" {
                parts.push(dur);
            }
            if tf != "ease" {
                parts.push(tf);
            }
            if delay != "0s" {
                parts.push(delay);
            }
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `animation` 简写：`<single-animation>#`（CSS Animations）。CSSOM 序列化（Chromium 150 oracle）：
/// 每条目顺序 duration / timing-function / delay / iteration-count / direction / fill-mode /
/// play-state / name，各 longhand 省初值（0s/ease/0s/1/normal/none/running/none）；全初值→`"none"`。
/// `animation:bounce 2s linear infinite alternate`→`"2s linear infinite alternate bounce"`。
/// 8 longhand 列表等长（`expand_animation` 保证），逐索引 zip 后逗号连接。
/// **已知 diverge**：`animation: bounce 0s`（显式 0s duration）Chromium 返 `"0s bounce"`，但 ZW
/// computed duration=0s 与 `animation: bounce`（省略 duration）不可区分→均返 `"bounce"`（罕见病理输入，
/// 同 R2754/R2755 谱「computed 不追踪 specified-ness」，记录不阻塞）。
fn animation_shorthand_to_css(style: &ComputedStyle) -> String {
    let n = style.animation_name.len();
    if n == 0 {
        return "none".to_string();
    }
    (0..n)
        .map(|i| {
            let name = style.animation_name[i].as_str();
            let dur = format_num(style.animation_duration.get(i).copied().unwrap_or(0.0), "s");
            let tf = style
                .animation_timing_function
                .get(i)
                .map(timing_function_to_css)
                .unwrap_or_else(|| "ease".to_string());
            let delay = format_num(style.animation_delay.get(i).copied().unwrap_or(0.0), "s");
            let iter = match style.animation_iteration_count.get(i) {
                None => "1".to_string(),
                Some(None) => "infinite".to_string(),
                Some(Some(v)) => format_num(*v, ""),
            };
            let dir = style
                .animation_direction
                .get(i)
                .map(animation_direction_str)
                .unwrap_or_else(|| "normal".to_string());
            let fill = style
                .animation_fill_mode
                .get(i)
                .map(animation_fill_mode_str)
                .unwrap_or_else(|| "none".to_string());
            let play = style
                .animation_play_state
                .get(i)
                .map(animation_play_state_str)
                .unwrap_or_else(|| "running".to_string());
            let mut parts = Vec::new();
            if dur != "0s" {
                parts.push(dur);
            }
            if tf != "ease" {
                parts.push(tf);
            }
            if delay != "0s" {
                parts.push(delay);
            }
            if iter != "1" {
                parts.push(iter);
            }
            if dir != "normal" {
                parts.push(dir);
            }
            if fill != "none" {
                parts.push(fill);
            }
            if play != "running" {
                parts.push(play);
            }
            if name != "none" {
                parts.push(name.to_string());
            }
            if parts.is_empty() {
                "none".to_string()
            } else {
                parts.join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// animation-direction：CSS Animations 方向（normal/reverse/alternate/alternate-reverse）。
fn animation_direction_str(d: &AnimationDirectionValue) -> String {
    match d {
        AnimationDirectionValue::Normal => "normal",
        AnimationDirectionValue::Reverse => "reverse",
        AnimationDirectionValue::Alternate => "alternate",
        AnimationDirectionValue::AlternateReverse => "alternate-reverse",
    }
    .to_string()
}

/// animation-fill-mode：CSS Animations 填充模式（none/forwards/backwards/both）。
fn animation_fill_mode_str(m: &AnimationFillModeValue) -> String {
    match m {
        AnimationFillModeValue::None => "none",
        AnimationFillModeValue::Forwards => "forwards",
        AnimationFillModeValue::Backwards => "backwards",
        AnimationFillModeValue::Both => "both",
    }
    .to_string()
}

/// animation-play-state：CSS Animations 播放状态（running/paused）。
fn animation_play_state_str(s: &AnimationPlayStateValue) -> String {
    match s {
        AnimationPlayStateValue::Running => "running",
        AnimationPlayStateValue::Paused => "paused",
    }
    .to_string()
}

/// transition/animation-timing-function：CSS Easing 单个缓动函数序列化。
///
/// 关键字（ease/linear/ease-in/out/in-out）+ step-start/end 对齐 Chromium（保 keyword 不展开为
/// cubic-bezier）；`cubic-bezier(a,b,c,d)` 4 数逗号分隔。`steps(n, pos)` 按 CSS Easing 1 §4：
/// 默认位置 End 省略（`steps(n)`）、Start→`start`/End→`end`（legacy canonical）、Both→`jump-both`、
/// None→`jump-none`。**待 Chromium A/B 核实**（Web 核实本轮被网络阻断，steps 位置 canonical 化为
/// spec-aligned 最佳推断；若 Chromium 显式含 `end` 或用 `jump-*` 别名，后续轮按 oracle 修正）。
fn timing_function_to_css(tf: &TimingFunctionValue) -> String {
    match tf {
        TimingFunctionValue::Ease => "ease".to_string(),
        TimingFunctionValue::Linear => "linear".to_string(),
        TimingFunctionValue::EaseIn => "ease-in".to_string(),
        TimingFunctionValue::EaseOut => "ease-out".to_string(),
        TimingFunctionValue::EaseInOut => "ease-in-out".to_string(),
        TimingFunctionValue::StepStart => "step-start".to_string(),
        TimingFunctionValue::StepEnd => "step-end".to_string(),
        TimingFunctionValue::CubicBezier(a, b, c, d) => format!(
            "cubic-bezier({}, {}, {}, {})",
            format_num(*a, ""),
            format_num(*b, ""),
            format_num(*c, ""),
            format_num(*d, "")
        ),
        TimingFunctionValue::Steps(n, pos) => match pos {
            None | Some(StepPosition::End) => format!("steps({n})"),
            Some(StepPosition::Start) => format!("steps({n}, start)"),
            Some(StepPosition::Both) => format!("steps({n}, jump-both)"),
            Some(StepPosition::None) => format!("steps({n}, jump-none)"),
        },
    }
}

/// scroll-padding-top/right/bottom/left：CSS Scroll Snap padding（`ScrollPadding`）。
/// Auto→`auto`（初值）；Length(v)→`Npx`（f32→f64 经 [`format_num`]）。
fn scroll_padding_to_css(p: &ScrollPadding) -> String {
    match p {
        ScrollPadding::Auto => "auto".to_string(),
        ScrollPadding::Length(v) => format_num(*v as f64, "px"),
    }
}

/// background-image / mask-image：CSS 图层列表（`Vec<BackgroundImageComputedValue>`）。
/// 空列表→`none`（初值）；None/Url 逐层序列化（`url("u")`，同 [`list_style_image_to_css`]），
/// linear/radial/conic-gradient 层各经对应 helper 序列化。多层逗号分隔。
fn image_layer_list_to_css(
    layers: &[BackgroundImageComputedValue],
    element_color: &ColorValue,
    font_size_px: f64,
) -> String {
    use zero_css_parser::values::GradientValue;
    if layers.is_empty() {
        return "none".to_string();
    }
    layers
        .iter()
        .map(|l| match l {
            BackgroundImageComputedValue::None => "none".to_string(),
            BackgroundImageComputedValue::Url(u) => format!("url(\"{u}\")"),
            BackgroundImageComputedValue::Gradient(GradientValue::Linear(g)) => {
                linear_gradient_to_css(g, element_color, font_size_px)
            }
            BackgroundImageComputedValue::Gradient(GradientValue::Radial(g)) => {
                radial_gradient_to_css(g, element_color, font_size_px)
            }
            BackgroundImageComputedValue::Gradient(GradientValue::Conic(g)) => {
                conic_gradient_to_css(g, element_color, font_size_px)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// linear-gradient / repeating-linear-gradient：CSS Images 线性渐变序列化（对齐 Chromium）。
/// `<prefix>(<dir>, [in <space>], <stop>, ...)`：dir `ToBottom` 初值省略 / `Angle(a)`→`adeg` /
/// 角关键字 `to right` 等；插值 `Srgb` 初值省略、余 `in <space>`（极坐标 Lch/Oklch 非默认 hue
/// 附 ` <hue-method>`）；色标 `<color>[ <pos>]`（color 经 currentcolor 解析，pos 经 length_to_css）。
/// **已知限制**：无双位置/色标提示存储（ZeroWeb 模型单 pos/无 hint）；radial/conic defer。
fn linear_gradient_to_css(
    g: &zero_css_parser::values::LinearGradient,
    element_color: &ColorValue,
    font_size_px: f64,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    match g.direction {
        GradientDirection::ToBottom => {} // 初值，省略
        GradientDirection::ToTop => parts.push("to top".to_string()),
        GradientDirection::ToLeft => parts.push("to left".to_string()),
        GradientDirection::ToRight => parts.push("to right".to_string()),
        GradientDirection::ToTopLeft => parts.push("to top left".to_string()),
        GradientDirection::ToTopRight => parts.push("to top right".to_string()),
        GradientDirection::ToBottomLeft => parts.push("to bottom left".to_string()),
        GradientDirection::ToBottomRight => parts.push("to bottom right".to_string()),
        GradientDirection::Angle(a) => parts.push(format!("{}deg", format_num(a, ""))),
    }
    if g.interpolation.space != ColorInterpolationSpace::Srgb {
        parts.push(color_interpolation_to_css(&g.interpolation));
    }
    for s in &g.stops {
        parts.push(color_stop_to_css(s, element_color, font_size_px));
    }
    let prefix = if g.repeating {
        "repeating-linear-gradient"
    } else {
        "linear-gradient"
    };
    format!("{}({})", prefix, parts.join(", "))
}

/// radial-gradient / repeating-radial-gradient：CSS Images 径向渐变序列化。
/// 规则锚定 WPT `background-image-computed`/`gradient-position-computed` oracle：
/// - 默认 `<ellipse> <farthest-corner> at center` 全省略 → `radial-gradient(stops)`
/// - `circle`（默认 size）保留；非默认 size 关键字（closest-side 等）保留；`circle <kw>` 双保留
/// - 显式半径 `Length(r)` → `r`（circle 关键字省略）；ellipse 双长度 ZeroWeb 仅存单 length（解析限制）
/// - 非默认 position → `at <X> <Y>`（center/top/bottom/left/right 解析期已归一为 50%/0%/100%）
fn radial_gradient_to_css(g: &RadialGradient, element_color: &ColorValue, font_size_px: f64) -> String {
    // config 组件：shape/size + position 空格连接（同 oracle「circle at 25% 40%」）。
    let mut config_inner: Vec<String> = Vec::new();
    match (&g.shape, &g.size) {
        (RadialShape::Ellipse, RadialSize::FarthestCorner) => {} // 默认，省略
        (RadialShape::Circle, RadialSize::FarthestCorner) => config_inner.push("circle".to_string()),
        (_, RadialSize::ClosestSide) => config_inner.push(radial_size_str(&g.size, &g.shape)),
        (_, RadialSize::FarthestSide) => config_inner.push(radial_size_str(&g.size, &g.shape)),
        (_, RadialSize::ClosestCorner) => config_inner.push(radial_size_str(&g.size, &g.shape)),
        (_, RadialSize::Length(l)) => config_inner.push(length_to_css(l, font_size_px)),
    }
    if !is_center(&g.position_x, &g.position_y) {
        config_inner.push(format!(
            "at {} {}",
            length_to_css(&g.position_x, font_size_px),
            length_to_css(&g.position_y, font_size_px)
        ));
    }
    let mut parts: Vec<String> = Vec::new();
    let config = config_inner.join(" ");
    if !config.is_empty() {
        parts.push(config);
    }
    if g.interpolation.space != ColorInterpolationSpace::Srgb {
        parts.push(color_interpolation_to_css(&g.interpolation));
    }
    for s in &g.stops {
        parts.push(color_stop_to_css(s, element_color, font_size_px));
    }
    let prefix = if g.repeating {
        "repeating-radial-gradient"
    } else {
        "radial-gradient"
    };
    format!("{}({})", prefix, parts.join(", "))
}

/// radial size 关键字 + 形状组合：`circle farthest-side` / `farthest-side`（ellipse 默认形状省略）。
fn radial_size_str(size: &RadialSize, shape: &RadialShape) -> String {
    let kw = match size {
        RadialSize::ClosestSide => "closest-side",
        RadialSize::FarthestSide => "farthest-side",
        RadialSize::ClosestCorner => "closest-corner",
        _ => "",
    };
    match shape {
        RadialShape::Circle => format!("circle {kw}"),
        RadialShape::Ellipse => kw.to_string(),
    }
}

/// conic-gradient / repeating-conic-gradient：CSS Images 锥向渐变序列化（spec-aligned，无 WPT conic
/// computed oracle，待 Chromium A/B 核实）。`<prefix>([from <angle>]? [at <pos>]?, stops)`：
/// 默认 from 0deg + at center 全省略；非默认 from→`from <deg>deg`、position→`at <X> <Y>`。
fn conic_gradient_to_css(g: &ConicGradient, element_color: &ColorValue, font_size_px: f64) -> String {
    let mut config_inner: Vec<String> = Vec::new();
    if g.from_angle != 0.0 {
        config_inner.push(format!("from {}deg", format_num(g.from_angle, "")));
    }
    if !is_center(&g.position_x, &g.position_y) {
        config_inner.push(format!(
            "at {} {}",
            length_to_css(&g.position_x, font_size_px),
            length_to_css(&g.position_y, font_size_px)
        ));
    }
    let mut parts: Vec<String> = Vec::new();
    let config = config_inner.join(" ");
    if !config.is_empty() {
        parts.push(config);
    }
    if g.interpolation.space != ColorInterpolationSpace::Srgb {
        parts.push(color_interpolation_to_css(&g.interpolation));
    }
    for s in &g.stops {
        parts.push(color_stop_to_css(s, element_color, font_size_px));
    }
    let prefix = if g.repeating {
        "repeating-conic-gradient"
    } else {
        "conic-gradient"
    };
    format!("{}({})", prefix, parts.join(", "))
}

/// position 是否为默认 center（50% 50%）。解析期 center→`Percentage(50.0)`。
fn is_center(x: &LengthValue, y: &LengthValue) -> bool {
    matches!(x, LengthValue::Percentage(v) if *v == 50.0) && matches!(y, LengthValue::Percentage(v) if *v == 50.0)
}

/// 渐变色标：`<color>[ <position>]`（color 经 currentcolor 解析为 rgb/rgba；pos 经 length_to_css）。
fn color_stop_to_css(s: &GradientColorStop, element_color: &ColorValue, font_size_px: f64) -> String {
    let color = color_to_css(&crate::resolve_color_current(&s.color, element_color));
    match &s.position {
        None => color,
        Some(pos) => format!("{} {}", color, length_to_css(pos, font_size_px)),
    }
}

/// 渐变颜色插值（CSS Color 4 `in <colorspace> [<hue-method>]`）。极坐标 Lch/Oklch 且 hue 非
/// 默认 Shorter 时附 hue-method；否则仅 `in <space>`。
fn color_interpolation_to_css(ci: &ColorInterpolation) -> String {
    let space = match ci.space {
        ColorInterpolationSpace::Srgb => "srgb",
        ColorInterpolationSpace::SrgbLinear => "srgb-linear",
        ColorInterpolationSpace::Lab => "lab",
        ColorInterpolationSpace::Oklab => "oklab",
        ColorInterpolationSpace::Lch => "lch",
        ColorInterpolationSpace::Oklch => "oklch",
    };
    let polar = matches!(ci.space, ColorInterpolationSpace::Lch | ColorInterpolationSpace::Oklch);
    if polar && ci.hue != ColorHueMethod::Shorter {
        let hue = match ci.hue {
            ColorHueMethod::Shorter => "shorter hue",
            ColorHueMethod::Longer => "longer hue",
            ColorHueMethod::Increasing => "increasing hue",
            ColorHueMethod::Decreasing => "decreasing hue",
        };
        format!("in {space} {hue}")
    } else {
        format!("in {space}")
    }
}

/// mask-mode：CSS Masking 遮罩模式（alpha/luminance/match-source，初值 match-source）。
fn mask_mode_str(m: &MaskModeComputedValue) -> String {
    match m {
        MaskModeComputedValue::Alpha => "alpha",
        MaskModeComputedValue::Luminance => "luminance",
        MaskModeComputedValue::MatchSource => "match-source",
    }
    .to_string()
}

/// counter-increment / counter-reset：CSS Lists 计数器操作（`Vec<CounterActionValue>`）。
/// 空列表→`none`；否则空格分隔 `name integer` 列表（对齐 Chromium；`value=None` 取 `default`
/// —— increment 默认 1 / reset 默认 0）。多计数器空格连接（非逗号，同 CSS Lists §4.1）。
fn counter_action_to_css(actions: &[CounterActionValue], default: i64) -> String {
    if actions.is_empty() {
        return "none".to_string();
    }
    actions
        .iter()
        .map(|a| format!("{} {}", a.name, a.value.unwrap_or(default)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// content-visibility：CSS Containment 2 可见性（Visible/Hidden/Auto，初值 visible）。
fn content_visibility_str(v: &ContentVisibilityValue) -> String {
    match v {
        ContentVisibilityValue::Visible => "visible",
        ContentVisibilityValue::Hidden => "hidden",
        ContentVisibilityValue::Auto => "auto",
    }
    .to_string()
}

/// 把 `Option<LengthValue>`（contain-intrinsic-width/height 等）序列化：None→`none`（初值），
/// Some→经 [`length_to_css`] 解析为 px（含残余相对单位兜底）。
fn opt_length_to_css(lv: &Option<LengthValue>, font_size_px: f64) -> String {
    match lv {
        None => "none".to_string(),
        Some(l) => length_to_css(l, font_size_px),
    }
}

/// 把 `Option<String>`（grid-template-* / grid-auto-* 等存原始 specified 串的字段）序列化为 CSS：
/// `Some(s)`→原样返回；`None`→`default`（grid-template-*/areas 初值 `none`，grid-auto-* 初值 `auto`）。
/// **已知限制**：存的是 specified 原文，`repeat()` 不展开（Chromium getComputedStyle 展开）—— pre-existing
/// 解析层限制（apply.rs 直接 `value.to_string()`），非 repeat 的固定/fr/minmax 轨道与 Chromium 一致。
fn opt_css_string(s: &Option<String>, default: &str) -> String {
    s.clone().unwrap_or_else(|| default.to_string())
}

/// `grid-template` 简写：rows/columns/areas 三 longhand（`Option<String>` 存原始 specified 串）重组。
/// Chromium 150 oracle 锚定：
/// - 全 none → `"none"`；
/// - areas==none → `"<rows> / <cols>"`（rows/cols 各自可为 `none`，如仅设列时 `none / 1fr 1fr`）；
/// - areas!=none → 把引号区域串逐行交错进行尺寸：`"<area0> <size0> <area1> <size1> ... / <cols>"`，
///   且仅当 area 数 == 行尺寸数可重组（Chromium 对不等数同样返 `""` 空串不可序列化）。
fn grid_template_shorthand_to_css(style: &ComputedStyle) -> String {
    let rows = style.grid_template_rows.as_deref();
    let cols = style.grid_template_columns.as_deref();
    let areas = style.grid_template_areas.as_deref();
    if rows.is_none() && cols.is_none() && areas.is_none() {
        return "none".to_string();
    }
    let rows_str = rows.unwrap_or("none");
    let cols_str = cols.unwrap_or("none");
    let rows_part = match areas {
        None => rows_str.to_string(),
        Some(a) => match interleave_grid_template_areas(a, rows_str) {
            Some(s) => s,
            // area 数 != 行尺寸数：Chromium 同样不可序列化，返空串（同未实现 fallback）。
            None => return String::new(),
        },
    };
    format!("{rows_part} / {cols_str}")
}

/// 把 grid-template-areas（`"a a" "b b"`，引号串空格连接）拆为带引号的 area 串列表。
fn split_grid_area_strings(areas: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    for ch in areas.chars() {
        if ch == '"' {
            in_quote = !in_quote;
            current.push(ch);
            if !in_quote {
                tokens.push(std::mem::take(&mut current));
            }
        } else if in_quote {
            current.push(ch);
        }
    }
    tokens
}

/// 交错 grid-template-areas 与 grid-template-rows：area 数 == 行尺寸数时返
/// `"<area0> <size0> <area1> <size1> ..."`，否则 `None`（不可重组）。
fn interleave_grid_template_areas(areas: &str, rows: &str) -> Option<String> {
    let area_tokens = split_grid_area_strings(areas);
    let sizes: Vec<&str> = rows.split_whitespace().collect();
    if area_tokens.is_empty() || area_tokens.len() != sizes.len() {
        return None;
    }
    let mut parts: Vec<String> = Vec::with_capacity(area_tokens.len() * 2);
    for (a, s) in area_tokens.iter().zip(sizes.iter()) {
        parts.push(a.clone());
        parts.push((*s).to_string());
    }
    Some(parts.join(" "))
}

/// box-shadow：CSS Box Shadow 计算值序列化。空列表→`none`；否则每个阴影按 Chromium/WPT
/// 格式 `<color> <ox>px <oy>px <blur>px <spread>px [inset]`（color 在前经 currentcolor 解析，
/// 4 长度全含即使为 0，inset 在末），多阴影逗号分隔。格式锚定 WPT box-shadow-interpolation/
/// composition 的 `expect` 串（如 `rgb(100,100,100) 10px 20px 30px 40px inset`）。
fn box_shadow_to_css(shadows: &[BoxShadowComputedValue], element_color: &ColorValue) -> String {
    if shadows.is_empty() {
        return "none".to_string();
    }
    shadows
        .iter()
        .map(|s| {
            let color = color_to_css(&crate::resolve_color_current(&s.color, element_color));
            let inset = if s.inset { " inset" } else { "" };
            format!(
                "{color} {} {} {} {}{inset}",
                format_num(s.offset_x as f64, "px"),
                format_num(s.offset_y as f64, "px"),
                format_num(s.blur_radius as f64, "px"),
                format_num(s.spread_radius as f64, "px"),
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// text-shadow：CSS Text Shadow 计算值序列化。空列表→`none`；否则每个阴影按 Chromium/WPT
/// 格式 `<color> <ox>px <oy>px <blur>px`（color 在前经 currentcolor 解析，3 长度全含即使为 0，
/// 无 spread/inset——text-shadow spec 无此二者），多阴影逗号分隔。格式与 box-shadow 对齐。
fn text_shadow_to_css(shadows: &[TextShadowComputedValue], element_color: &ColorValue) -> String {
    if shadows.is_empty() {
        return "none".to_string();
    }
    shadows
        .iter()
        .map(|s| {
            let color = color_to_css(&crate::resolve_color_current(&s.color, element_color));
            format!(
                "{color} {} {} {}",
                format_num(s.offset_x as f64, "px"),
                format_num(s.offset_y as f64, "px"),
                format_num(s.blur_radius as f64, "px"),
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// content：按 CSS Generated Content 计算值序列化（::before/::after 生成内容）。
///
/// `Normal`/`None`→关键字；`String`→CSS 双引号串（[`css_string_to_css`]）；`Attr`→`attr(name)`；
/// `Counter`/`Counters`→计数器函数；`Url`→`url(...)`；`List`→多 component value 空格连接。
fn content_to_css(c: &ContentComputedValue) -> String {
    use ContentComputedValue as C;
    match c {
        C::Normal => "normal".to_string(),
        C::None => "none".to_string(),
        C::String(s) => css_string_to_css(s),
        C::Attr(name) => format!("attr({name})"),
        C::Counter { name, style } => counter_fn_to_css(name, style.as_deref()),
        C::Counters { name, separator, style } => counters_fn_to_css(name, separator, style.as_deref()),
        C::Url(u) => format!("url({u})"),
        C::List(items) => items.iter().map(content_list_item_to_css).collect::<Vec<_>>().join(" "),
    }
}

/// 序列化 content 多 component value 序列的单项 [`ContentListItem`]（Str/Counter/Counters）。
fn content_list_item_to_css(item: &ContentListItem) -> String {
    use ContentListItem as I;
    match item {
        I::Str(s) => css_string_to_css(s),
        I::Counter { name, style } => counter_fn_to_css(name, style.as_deref()),
        I::Counters { name, separator, style } => counters_fn_to_css(name, separator, style.as_deref()),
    }
}

/// `counter(name[, style])` 函数序列化。
fn counter_fn_to_css(name: &str, style: Option<&str>) -> String {
    match style {
        Some(s) => format!("counter({name}, {s})"),
        None => format!("counter({name})"),
    }
}

/// `counters(name, "sep"[, style])` 函数序列化（separator 经 CSS 引号串化）。
fn counters_fn_to_css(name: &str, separator: &str, style: Option<&str>) -> String {
    let sep = css_string_to_css(separator);
    match style {
        Some(s) => format!("counters({name}, {sep}, {s})"),
        None => format!("counters({name}, {sep})"),
    }
}

/// 把字符串字面量序列化为 CSS 双引号串：转义 `\` / `"` / 换行（CSS escape `\A `）。
fn css_string_to_css(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\A "),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// background-position：CSS Backgrounds `<bg-position>#` 多层序列化（逗号分隔层）。
///
/// 关键字解析为百分比（center 50% / left 0% / right 100% / top 0% / bottom 100%），对齐 Chromium
/// getComputedStyle（WPT background-computed.html）；单关键字/单长度按轴展开（缺省轴 = center 50%）。
fn background_position_to_css(layers: &[BackgroundPositionComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(bg_position_layer_to_css)
        .collect::<Vec<_>>()
        .join(", ")
}

/// 序列化单层 `<bg-position>`。单值按轴展开（缺省轴 center 50%）；TwoValue 两轴；EdgeOffset 边缘+偏移。
fn bg_position_layer_to_css(p: &BackgroundPositionComputedValue) -> String {
    use BackgroundPositionComputedValue as P;
    match p {
        // 单关键字：按轴展开（水平关键字→垂直 center 50%；垂直关键字→水平 center 50%）。
        P::Center => "50% 50%".to_string(),
        P::Left => "0% 50%".to_string(),
        P::Right => "100% 50%".to_string(),
        P::Top => "50% 0%".to_string(),
        P::Bottom => "50% 100%".to_string(),
        // 单长度/百分比：水平 = 值，垂直缺省 center 50%。
        P::Length(f) => format!("{} 50%", format_num(*f as f64, "px")),
        P::Percent(f) => format!("{} 50%", format_num(*f as f64, "%")),
        // % calc 无容器尺寸不可解析（非 % calc 已 resolve）→ ''。
        P::Calc(_) => String::new(),
        P::TwoValue(h, v) => format!("{} {}", bg_position_axis_to_css(h), bg_position_axis_to_css(v)),
        P::EdgeOffset(edge, offset) => {
            format!("{} {}", bg_edge_to_str(edge), bg_position_axis_to_css(offset))
        }
    }
}

/// 序列化 `<bg-position>` 单轴值（关键字→% / Length→px / Percent→%）。TwoValue/EdgeOffset 不应出现于轴位。
fn bg_position_axis_to_css(v: &BackgroundPositionComputedValue) -> String {
    use BackgroundPositionComputedValue as P;
    match v {
        P::Center => "50%".to_string(),
        P::Left => "0%".to_string(),
        P::Right => "100%".to_string(),
        P::Top => "0%".to_string(),
        P::Bottom => "100%".to_string(),
        P::Length(f) => format_num(*f as f64, "px"),
        P::Percent(f) => format_num(*f as f64, "%"),
        P::Calc(_) => String::new(),
        // 防御：轴位不应嵌套 TwoValue/EdgeOffset。
        P::TwoValue(..) | P::EdgeOffset(..) => String::new(),
    }
}

/// `<bg-position>` 边缘关键字（3/4 值 EdgeOffset 的 side）。
fn bg_edge_to_str(e: &BackgroundEdge) -> &'static str {
    match e {
        BackgroundEdge::Left => "left",
        BackgroundEdge::Right => "right",
        BackgroundEdge::Top => "top",
        BackgroundEdge::Bottom => "bottom",
    }
}

/// background-size：CSS Backgrounds `<bg-size>#` 多层序列化（逗号分隔）。
/// Auto/Cover/Contain→关键字；Length→px；Percent→%。
/// R2878：background-size 两值语法单维分量序列化（auto / <length>px / <percent>%）。
fn bg_size_component_to_css(c: &BgSizeComponentComputed) -> String {
    match c {
        BgSizeComponentComputed::Auto => "auto".to_string(),
        BgSizeComponentComputed::Length(f) => format_num(*f as f64, "px"),
        BgSizeComponentComputed::Percent(f) => format_num(*f as f64, "%"),
    }
}

fn background_size_to_css(layers: &[BackgroundSizeComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(|s| match s {
            BackgroundSizeComputedValue::Auto => "auto".to_string(),
            BackgroundSizeComputedValue::Cover => "cover".to_string(),
            BackgroundSizeComputedValue::Contain => "contain".to_string(),
            BackgroundSizeComputedValue::Length(f) => format_num(*f as f64, "px"),
            BackgroundSizeComputedValue::Percent(f) => format_num(*f as f64, "%"),
            // R2878：两值语法序列化（`<w> <h>`，auto/length/percent 每维）。
            BackgroundSizeComputedValue::TwoValue(cw, ch) => {
                format!("{} {}", bg_size_component_to_css(cw), bg_size_component_to_css(ch))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// background-repeat：CSS Backgrounds `<repeat-style>#` 多层序列化（逗号分隔）。
fn background_repeat_to_css(layers: &[BackgroundRepeatComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(|r| match r {
            BackgroundRepeatComputedValue::Repeat => "repeat",
            BackgroundRepeatComputedValue::RepeatX => "repeat-x",
            BackgroundRepeatComputedValue::RepeatY => "repeat-y",
            BackgroundRepeatComputedValue::NoRepeat => "no-repeat",
            BackgroundRepeatComputedValue::Space => "space",
            BackgroundRepeatComputedValue::Round => "round",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// background-attachment：CSS Backgrounds `<attachment>` 单值序列化。
/// Scroll/Fixed/Local → scroll/fixed/local。ZeroWeb 存单值（非多层 Vec），与解析侧一致。
fn background_attachment_to_css(a: &BackgroundAttachmentComputedValue) -> String {
    match a {
        BackgroundAttachmentComputedValue::Scroll => "scroll",
        BackgroundAttachmentComputedValue::Fixed => "fixed",
        BackgroundAttachmentComputedValue::Local => "local",
    }
    .to_string()
}

/// background-clip：CSS Backgrounds `<visual-box>` 单值序列化。
/// border-box/padding-box/content-box/text。ZeroWeb 存单值（非多层 Vec）。
fn background_clip_to_css(c: &BackgroundClipComputedValue) -> String {
    match c {
        BackgroundClipComputedValue::BorderBox => "border-box",
        BackgroundClipComputedValue::PaddingBox => "padding-box",
        BackgroundClipComputedValue::ContentBox => "content-box",
        BackgroundClipComputedValue::Text => "text",
    }
    .to_string()
}

/// background-origin：CSS Backgrounds `<geometry-box>` 单值序列化。
/// padding-box/border-box/content-box。ZeroWeb 存单值（非多层 Vec）。
fn background_origin_to_css(o: &BackgroundOriginComputedValue) -> String {
    match o {
        BackgroundOriginComputedValue::PaddingBox => "padding-box",
        BackgroundOriginComputedValue::BorderBox => "border-box",
        BackgroundOriginComputedValue::ContentBox => "content-box",
    }
    .to_string()
}

/// align-content：CSS Box Alignment `<content-distribution>` 单值序列化。
/// Chromium getComputedStyle 初值 = normal（CSS Align 3）。ZeroWeb computed 默认 Normal。
fn align_content_to_css(a: &AlignContentValue) -> String {
    match a {
        AlignContentValue::Auto => "auto",
        AlignContentValue::Normal => "normal",
        AlignContentValue::Start => "start",
        AlignContentValue::End => "end",
        AlignContentValue::Center => "center",
        AlignContentValue::Stretch => "stretch",
        AlignContentValue::Baseline => "baseline",
        AlignContentValue::SpaceBetween => "space-between",
        AlignContentValue::SpaceAround => "space-around",
        AlignContentValue::SpaceEvenly => "space-evenly",
    }
    .to_string()
}

/// justify-items：CSS Box Alignment 单值序列化。初值 = normal（CSS Align 3）。
fn justify_items_to_css(j: &JustifyItemsValue) -> String {
    match j {
        JustifyItemsValue::Auto => "auto",
        JustifyItemsValue::Normal => "normal",
        JustifyItemsValue::Start => "start",
        JustifyItemsValue::End => "end",
        JustifyItemsValue::Center => "center",
        JustifyItemsValue::Stretch => "stretch",
        JustifyItemsValue::Baseline => "baseline",
        JustifyItemsValue::Left => "left",
        JustifyItemsValue::Right => "right",
    }
    .to_string()
}

/// justify-self：CSS Box Alignment 单值序列化。初值 = auto（CSS Align 3）。
fn justify_self_to_css(j: &JustifySelfValue) -> String {
    match j {
        JustifySelfValue::Auto => "auto",
        JustifySelfValue::Normal => "normal",
        JustifySelfValue::Start => "start",
        JustifySelfValue::End => "end",
        JustifySelfValue::Center => "center",
        JustifySelfValue::Stretch => "stretch",
        JustifySelfValue::Baseline => "baseline",
        JustifySelfValue::Left => "left",
        JustifySelfValue::Right => "right",
    }
    .to_string()
}

/// `place-content` / `place-items` / `place-self` 简写 CSSOM 2 值最小化（CSS Box Alignment）：
/// align==justify→单值，否则 `"align justify"`。Chromium 150 oracle：`place-content:center start`→
/// `"center start"`、`place-self:center`→`"center"`（单值设两轴同值）。
/// **已知 diverge（pre-existing，记录不阻塞）**：place-content/items 默认值受 ZW 的 justify-content
/// 默认 FlexStart / align-items 默认 Stretch 影响（ZW 默认 vs Chromium normal）——根因为 layout-coupled
/// 默认值，非本序列化引入；显式设置的值（含单值同值）序列化正确。
fn place_2value_min(align: &str, justify: &str) -> String {
    if align == justify {
        align.to_string()
    } else {
        format!("{align} {justify}")
    }
}

/// `<grid-line>` 单值序列化（grid-column/row-start/end longhand + grid-area 分量）：
/// Auto→`auto`、Line(n)→`n`（负数 from-end）、Span(n)→`span n`、Name(s)→`s`。
/// Chromium 150 oracle：`2`/`span 2`/`main`/`auto`。
fn grid_line_to_css(line: &GridLineValue) -> String {
    match line {
        GridLineValue::Auto => "auto".to_string(),
        GridLineValue::Line(n) => n.to_string(),
        GridLineValue::Span(n) => format!("span {}", n),
        GridLineValue::Name(s) => s.clone(),
    }
}

/// grid-column / grid-row 简写（start / end 2 值最小化）。Chromium 150 oracle：
/// start==end→单值；否则 end==auto 且 start 非 custom-ident(Name)→单值 start；否则 `"start / end"`。
/// `grid-column:2`→`"2"`、`grid-column:main`→`"main / auto"`（Name 保留 auto end 避歧义）。
fn grid_line_pair_to_css(start: &GridLineValue, end: &GridLineValue) -> String {
    let s = grid_line_to_css(start);
    let e = grid_line_to_css(end);
    if s == e || (e == "auto" && !matches!(start, GridLineValue::Name(_))) {
        s
    } else {
        format!("{s} / {e}")
    }
}

/// grid-area 简写（row-start / column-start / row-end / column-end 4 槽 trailing-drop 最小化）。
/// Chromium 150 oracle 规则：① 四值全等→单值；② 否则从末尾 drop 可省槽——ce 可省 iff ce==auto 且
/// cs 非 Name，re 可省 iff（ce 已省）且 re==auto 且 rs 非 Name；rs/cs 恒留（≥2 值）。
/// `grid-area:1/1/3/3`→`"1 / 1 / 3 / 3"`、`grid-area:2/3`→`"2 / 3"`、`grid-area:header`→`"header"`、
/// `grid-column-start:main`（cs=Name）→grid-area=`"auto / main / auto / auto"`（Name 阻止 ce 省）。
fn grid_area_to_css(style: &ComputedStyle) -> String {
    let rs = grid_line_to_css(&style.grid_row_start);
    let cs = grid_line_to_css(&style.grid_column_start);
    let re = grid_line_to_css(&style.grid_row_end);
    let ce = grid_line_to_css(&style.grid_column_end);
    if rs == cs && cs == re && re == ce {
        return rs;
    }
    let mut slots = vec![rs.clone(), cs.clone(), re.clone(), ce];
    let cs_is_name = matches!(style.grid_column_start, GridLineValue::Name(_));
    let rs_is_name = matches!(style.grid_row_start, GridLineValue::Name(_));
    // ce（slot 3）可省 iff ce==auto 且 cs 非 Name。
    if slots[3] == "auto" && !cs_is_name {
        slots.truncate(3);
        // re（现末尾）可省 iff re==auto 且 rs 非 Name。
        if slots[2] == "auto" && !rs_is_name {
            slots.truncate(2);
        }
    }
    slots.join(" / ")
}

/// word-break：CSS Text `<word-break>` 单值序列化。初值 normal。
fn word_break_to_css(w: &WordBreakValue) -> String {
    match w {
        WordBreakValue::Normal => "normal",
        WordBreakValue::BreakAll => "break-all",
        WordBreakValue::KeepAll => "keep-all",
        WordBreakValue::BreakWord => "break-word",
    }
    .to_string()
}

/// overflow-wrap：CSS Text `<overflow-wrap>` 单值序列化。初值 normal。
fn overflow_wrap_to_css(w: &OverflowWrapValue) -> String {
    match w {
        OverflowWrapValue::Normal => "normal",
        OverflowWrapValue::BreakWord => "break-word",
        OverflowWrapValue::Anywhere => "anywhere",
    }
    .to_string()
}

/// hyphens：CSS Text 单值序列化。ZeroWeb 初值 None（diverge：CSS 规范初值 manual，Chromium 返 manual）。
fn hyphens_to_css(h: &HyphensComputedValue) -> String {
    match h {
        HyphensComputedValue::None => "none",
        HyphensComputedValue::Manual => "manual",
        HyphensComputedValue::Auto => "auto",
    }
    .to_string()
}

/// line-break：CSS Text 单值序列化。初值 auto。
fn line_break_to_css(l: &LineBreakValue) -> String {
    match l {
        LineBreakValue::Auto => "auto",
        LineBreakValue::Loose => "loose",
        LineBreakValue::Normal => "normal",
        LineBreakValue::Strict => "strict",
        LineBreakValue::Anywhere => "anywhere",
    }
    .to_string()
}

/// vertical-align：CSS 行内/表格单元格垂直对齐单值序列化。初值 baseline。
/// ZeroWeb enum 仅关键字（无 `<length>`/`<percentage>` 变体）——diverge：CSS 规范允许，Chromium 对
/// `5px`/`50%` 返 used 值，ZeroWeb 无 length 变体存储故无法表达（解析层丢弃，本序列化如实反映枚举）。
fn vertical_align_to_css(v: &VerticalAlignValue) -> String {
    match v {
        VerticalAlignValue::Baseline => "baseline",
        VerticalAlignValue::Top => "top",
        VerticalAlignValue::Middle => "middle",
        VerticalAlignValue::Bottom => "bottom",
        VerticalAlignValue::TextTop => "text-top",
        VerticalAlignValue::TextBottom => "text-bottom",
        VerticalAlignValue::Sub => "sub",
        VerticalAlignValue::Super => "super",
    }
    .to_string()
}

/// unicode-bidi：CSS Writing Modes 双向文本算法单值序列化。初值 normal。
fn unicode_bidi_to_css(u: &UnicodeBidiValue) -> String {
    match u {
        UnicodeBidiValue::Normal => "normal",
        UnicodeBidiValue::Embed => "embed",
        UnicodeBidiValue::Isolate => "isolate",
        UnicodeBidiValue::BidiOverride => "bidi-override",
        UnicodeBidiValue::IsolateOverride => "isolate-override",
        UnicodeBidiValue::Plaintext => "plaintext",
    }
    .to_string()
}

/// empty-cells：CSS 表格空单元格边框单值序列化。初值 show。
fn empty_cells_to_css(e: &EmptyCellsComputedValue) -> String {
    match e {
        EmptyCellsComputedValue::Show => "show",
        EmptyCellsComputedValue::Hide => "hide",
    }
    .to_string()
}

/// 序列化单个 [`TransformFunction`]。translate 类长度为 px（混合百分比分支保 `%`）；rotate/skew
/// 角度为 `deg`；scale 系数无单位；matrix 6 分量无单位。
fn transform_function_to_css(f: &TransformFunction) -> String {
    use TransformFunction as Tf;
    match f {
        Tf::Translate(tx, ty) => {
            format!("translate({}, {})", format_num(*tx, "px"), format_num(*ty, "px"))
        }
        Tf::TranslateMixed(tx, txp, ty, typ) => {
            format!("translate({}, {})", mixed_len(*tx, *txp), mixed_len(*ty, *typ))
        }
        Tf::TranslateXMixed(v, pct) => format!("translateX({})", mixed_len(*v, *pct)),
        Tf::TranslateYMixed(v, pct) => format!("translateY({})", mixed_len(*v, *pct)),
        Tf::TranslateX(v) => format!("translateX({})", format_num(*v, "px")),
        Tf::TranslateY(v) => format!("translateY({})", format_num(*v, "px")),
        Tf::Rotate(deg) => format!("rotate({}deg)", format_num(*deg, "")),
        Tf::Scale(sx, sy) => match sy {
            Some(sy) => format!("scale({}, {})", format_num(*sx, ""), format_num(*sy, "")),
            None => format!("scale({})", format_num(*sx, "")),
        },
        Tf::ScaleX(sx) => format!("scaleX({})", format_num(*sx, "")),
        Tf::ScaleY(sy) => format!("scaleY({})", format_num(*sy, "")),
        Tf::Skew(ax, ay) => match ay {
            Some(ay) => format!("skew({}deg, {}deg)", format_num(*ax, ""), format_num(*ay, "")),
            None => format!("skew({}deg)", format_num(*ax, "")),
        },
        Tf::RotateX(deg) => format!("rotateX({}deg)", format_num(*deg, "")),
        Tf::RotateY(deg) => format!("rotateY({}deg)", format_num(*deg, "")),
        Tf::RotateZ(deg) => format!("rotateZ({}deg)", format_num(*deg, "")),
        Tf::Translate3d(tx, ty, tz) => format!(
            "translate3d({}, {}, {})",
            format_num(*tx, "px"),
            format_num(*ty, "px"),
            format_num(*tz, "px")
        ),
        Tf::Scale3d(sx, sy, sz) => format!(
            "scale3d({}, {}, {})",
            format_num(*sx, ""),
            format_num(*sy, ""),
            format_num(*sz, "")
        ),
        Tf::Rotate3d(x, y, z, angle) => format!(
            "rotate3d({}, {}, {}, {}deg)",
            format_num(*x, ""),
            format_num(*y, ""),
            format_num(*z, ""),
            format_num(*angle, "")
        ),
        Tf::Perspective(len) => format!("perspective({})", format_num(*len, "px")),
        Tf::Matrix(a, b, c, d, e, f) => format!(
            "matrix({}, {}, {}, {}, {}, {})",
            format_num(*a, ""),
            format_num(*b, ""),
            format_num(*c, ""),
            format_num(*d, ""),
            format_num(*e, ""),
            format_num(*f, "")
        ),
    }
}

/// translate 类混合长度：`is_pct` → `N%`，否则 `Npx`。
fn mixed_len(v: f64, is_pct: bool) -> String {
    if is_pct {
        format_num(v, "%")
    } else {
        format_num(v, "px")
    }
}
