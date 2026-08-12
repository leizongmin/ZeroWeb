//! CSS Scroll Snap、变量、分页属性解析。

use super::*;

// ── CSS Scroll Snap 值类型 ──────────────────────────────────────────

/// CSS scroll-snap-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapTypeValue {
    /// none。
    None,
    /// mandatory（必须吸附）。
    Mandatory,
    /// proximity（接近时吸附）。
    Proximity,
}

/// CSS scroll-snap-type 轴。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAxis {
    /// x 轴。
    X,
    /// y 轴。
    Y,
    /// 两个轴。
    Both,
}

/// CSS scroll-snap-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAlignValue {
    /// none。
    None,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
}

/// CSS scroll-snap-stop 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapStopValue {
    /// normal。
    Normal,
    /// always。
    Always,
}

/// CSS container-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerTypeValue {
    /// normal。
    Normal,
    /// size。
    Size,
    /// inline-size。
    InlineSize,
}

/// 解析 CSS scroll-snap-type 属性值。
///
/// 支持格式如 `"none"`、`"x mandatory"`、`"y proximity"`、`"both mandatory"`。
/// 返回 (strictness, axis) 元组。
pub fn parse_scroll_snap_type(value: &str) -> Option<(ScrollSnapTypeValue, Option<ScrollSnapAxis>)> {
    let value = value.trim().to_ascii_lowercase();

    if value == "none" {
        return Some((ScrollSnapTypeValue::None, None));
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut strictness = None;
    let mut axis = None;

    for part in parts {
        match part {
            "mandatory" => strictness = Some(ScrollSnapTypeValue::Mandatory),
            "proximity" => strictness = Some(ScrollSnapTypeValue::Proximity),
            "x" => axis = Some(ScrollSnapAxis::X),
            "y" => axis = Some(ScrollSnapAxis::Y),
            "both" => axis = Some(ScrollSnapAxis::Both),
            _ => return None,
        }
    }

    strictness.map(|s| (s, axis))
}

/// 解析 CSS scroll-snap-align 属性值。
pub fn parse_scroll_snap_align(value: &str) -> Option<ScrollSnapAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ScrollSnapAlignValue::None),
        "start" => Some(ScrollSnapAlignValue::Start),
        "end" => Some(ScrollSnapAlignValue::End),
        "center" => Some(ScrollSnapAlignValue::Center),
        _ => None,
    }
}

/// 解析 CSS scroll-snap-stop 属性值。
pub fn parse_scroll_snap_stop(value: &str) -> Option<ScrollSnapStopValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ScrollSnapStopValue::Normal),
        "always" => Some(ScrollSnapStopValue::Always),
        _ => None,
    }
}

/// 解析 CSS container-type 属性值。
pub fn parse_container_type(value: &str) -> Option<ContainerTypeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ContainerTypeValue::Normal),
        "size" => Some(ContainerTypeValue::Size),
        "inline-size" => Some(ContainerTypeValue::InlineSize),
        _ => None,
    }
}

/// CSS vertical-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalAlignValue {
    /// baseline（默认值）— 元素基线与父元素基线对齐。
    Baseline,
    /// top — 元素顶部与行盒顶部对齐。
    Top,
    /// middle — 元素中部与父元素基线 + 半 x-height 处对齐。
    Middle,
    /// bottom — 元素底部与行盒底部对齐。
    Bottom,
    /// text-top — 元素顶部与父元素字体的顶部对齐。
    TextTop,
    /// text-bottom — 元素底部与父元素字体的底部对齐。
    TextBottom,
    /// sub — 元素基线下移至适合下标的位置。
    Sub,
    /// super — 元素基线上移至适合上标的位置。
    Super,
}

/// CSS cursor 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CursorValue {
    /// auto。
    Auto,
    /// default。
    Default,
    /// pointer。
    Pointer,
    /// move。
    Move,
    /// text。
    Text,
    /// wait。
    Wait,
    /// crosshair。
    Crosshair,
    /// not-allowed。
    NotAllowed,
    /// grab。
    Grab,
    /// grabbing。
    Grabbing,
    /// help。
    Help,
    /// progress。
    Progress,
    /// n-resize。
    NResize,
    /// s-resize。
    SResize,
    /// e-resize。
    EResize,
    /// w-resize。
    WResize,
    /// ne-resize。
    NeResize,
    /// nw-resize。
    NwResize,
    /// se-resize。
    SeResize,
    /// sw-resize。
    SwResize,
    /// col-resize。
    ColResize,
    /// row-resize。
    RowResize,
    /// all-scroll。
    AllScroll,
    /// zoom-in。
    ZoomIn,
    /// zoom-out。
    ZoomOut,
    /// none。
    None,
}

/// 解析 CSS cursor 属性值。
pub fn parse_cursor(value: &str) -> Option<CursorValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(CursorValue::Auto),
        "default" => Some(CursorValue::Default),
        "pointer" => Some(CursorValue::Pointer),
        "move" => Some(CursorValue::Move),
        "text" => Some(CursorValue::Text),
        "wait" => Some(CursorValue::Wait),
        "crosshair" => Some(CursorValue::Crosshair),
        "not-allowed" => Some(CursorValue::NotAllowed),
        "grab" => Some(CursorValue::Grab),
        "grabbing" => Some(CursorValue::Grabbing),
        "help" => Some(CursorValue::Help),
        "progress" => Some(CursorValue::Progress),
        "n-resize" => Some(CursorValue::NResize),
        "s-resize" => Some(CursorValue::SResize),
        "e-resize" => Some(CursorValue::EResize),
        "w-resize" => Some(CursorValue::WResize),
        "ne-resize" => Some(CursorValue::NeResize),
        "nw-resize" => Some(CursorValue::NwResize),
        "se-resize" => Some(CursorValue::SeResize),
        "sw-resize" => Some(CursorValue::SwResize),
        "col-resize" => Some(CursorValue::ColResize),
        "row-resize" => Some(CursorValue::RowResize),
        "all-scroll" => Some(CursorValue::AllScroll),
        "zoom-in" => Some(CursorValue::ZoomIn),
        "zoom-out" => Some(CursorValue::ZoomOut),
        "none" => Some(CursorValue::None),
        _ => None,
    }
}

/// 解析 CSS opacity 属性值。
///
/// 支持数值（0.0-1.0）和百分比（如 `50%` → 0.5）。
/// 结果限制在 [0.0, 1.0] 范围内。
pub fn parse_opacity(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.ends_with('%') {
        let pct: f64 = value.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0).clamp(0.0, 1.0))
    } else {
        let num: f64 = value.parse().ok()?;
        Some(num.clamp(0.0, 1.0))
    }
}

/// 解析 CSS vertical-align 属性值。
pub fn parse_vertical_align(value: &str) -> Option<VerticalAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(VerticalAlignValue::Baseline),
        "top" => Some(VerticalAlignValue::Top),
        "middle" => Some(VerticalAlignValue::Middle),
        "bottom" => Some(VerticalAlignValue::Bottom),
        "text-top" => Some(VerticalAlignValue::TextTop),
        "text-bottom" => Some(VerticalAlignValue::TextBottom),
        "sub" => Some(VerticalAlignValue::Sub),
        "super" => Some(VerticalAlignValue::Super),
        _ => None,
    }
}

/// 解析 1-4 个长度值的简写属性（如 scroll-margin、scroll-padding）。
///
/// 返回 [top, right, bottom, left]（按 CSS 简写规则展开）。
pub fn parse_length_shorthand(value: &str) -> Option<[LengthValue; 4]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_length(parts[0])?;
            Some([v.clone(), v.clone(), v.clone(), v])
        }
        2 => {
            let tb = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            Some([tb.clone(), lr.clone(), tb, lr])
        }
        3 => {
            let top = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            Some([top, lr.clone(), bottom, lr])
        }
        4 => {
            let top = parse_length(parts[0])?;
            let right = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            let left = parse_length(parts[3])?;
            Some([top, right, bottom, left])
        }
        _ => None,
    }
}

/// 解析 CSS var() 函数引用。
///
/// 支持格式如 `var(--name)` 和 `var(--name, fallback)`。
pub fn parse_var(value: &str) -> Option<VarReference> {
    let value = value.trim();

    // CSS Values §4：函数名大小写不敏感（VAR ≡ Var ≡ var）。自定义属性名（--x）大小写敏感，
    // 故仅前缀大小写不敏感检查，内容（变量名/回退）按原样提取。
    if !(value.len() >= 4 && value[..4].eq_ignore_ascii_case("var(") && value.ends_with(')')) {
        return None;
    }

    // 提取括号内的内容
    let inner = value.get(4..value.len() - 1)?.trim();

    // 找到逗号（如果有）
    if let Some(comma_pos) = inner.find(',') {
        let name = inner[..comma_pos].trim().to_string();
        let fallback = inner[comma_pos + 1..].trim().to_string();
        Some(VarReference {
            name,
            fallback: Some(fallback),
        })
    } else {
        Some(VarReference {
            name: inner.to_string(),
            fallback: None,
        })
    }
}

// ── CSS Page Break 值类型 ──────────────────────────────────────────────

/// CSS page-break 属性值（page-break-before、page-break-after、page-break-inside）。
#[derive(Debug, Clone, PartialEq)]
pub enum PageBreakValue {
    /// auto。
    Auto,
    /// always。
    Always,
    /// avoid。
    Avoid,
    /// left。
    Left,
    /// right。
    Right,
}

/// 解析 CSS page-break 属性值。
pub fn parse_page_break(value: &str) -> Option<PageBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(PageBreakValue::Auto),
        "always" => Some(PageBreakValue::Always),
        "avoid" => Some(PageBreakValue::Avoid),
        "left" => Some(PageBreakValue::Left),
        "right" => Some(PageBreakValue::Right),
        _ => None,
    }
}

/// CSS box-decoration-break 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxDecorationBreakValue {
    /// slice。
    Slice,
    /// clone。
    Clone,
}

/// 解析 CSS box-decoration-break 属性值。
pub fn parse_box_decoration_break(value: &str) -> Option<BoxDecorationBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "slice" => Some(BoxDecorationBreakValue::Slice),
        "clone" => Some(BoxDecorationBreakValue::Clone),
        _ => None,
    }
}

/// CSS image-rendering 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ImageRenderingValue {
    /// auto。
    Auto,
    /// smooth。
    Smooth,
    /// high-quality。
    HighQuality,
    /// pixelated。
    Pixelated,
    /// crisp-edges。
    CrispEdges,
}

/// 解析 CSS image-rendering 属性值。
pub fn parse_image_rendering(value: &str) -> Option<ImageRenderingValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(ImageRenderingValue::Auto),
        "smooth" => Some(ImageRenderingValue::Smooth),
        "high-quality" => Some(ImageRenderingValue::HighQuality),
        "pixelated" => Some(ImageRenderingValue::Pixelated),
        "crisp-edges" => Some(ImageRenderingValue::CrispEdges),
        _ => None,
    }
}

/// CSS isolation 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationValue {
    /// auto。
    Auto,
    /// isolate。
    Isolate,
}

/// 解析 CSS isolation 属性值。
pub fn parse_isolation(value: &str) -> Option<IsolationValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(IsolationValue::Auto),
        "isolate" => Some(IsolationValue::Isolate),
        _ => None,
    }
}

/// CSS break-inside 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakInsideValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS break-before / break-after 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// column。
    Column,
    /// page。
    Page,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS column-rule-width 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleWidthValue {
    /// medium。
    Medium,
    /// thin。
    Thin,
    /// thick。
    Thick,
    /// 长度值。
    Length(LengthValue),
}

/// CSS column-rule-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleStyleValue {
    /// none。
    None,
    /// hidden。
    Hidden,
    /// dotted。
    Dotted,
    /// dashed。
    Dashed,
    /// solid。
    Solid,
    /// double。
    Double,
    /// groove。
    Groove,
    /// ridge。
    Ridge,
    /// inset。
    Inset,
    /// outset。
    Outset,
}

/// 解析 CSS break-inside 属性值。
pub fn parse_break_inside(value: &str) -> Option<BreakInsideValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakInsideValue::Auto),
        "avoid" => Some(BreakInsideValue::Avoid),
        "avoid-page" => Some(BreakInsideValue::AvoidPage),
        "avoid-column" => Some(BreakInsideValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-before 属性值。
pub fn parse_break_before(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-after 属性值。
pub fn parse_break_after(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS column-rule-width 属性值。
pub fn parse_column_rule_width(value: &str) -> Option<ColumnRuleWidthValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "medium" => Some(ColumnRuleWidthValue::Medium),
        "thin" => Some(ColumnRuleWidthValue::Thin),
        "thick" => Some(ColumnRuleWidthValue::Thick),
        _ => parse_length(&v).map(ColumnRuleWidthValue::Length),
    }
}

/// 解析 CSS column-rule-style 属性值。
pub fn parse_column_rule_style(value: &str) -> Option<ColumnRuleStyleValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ColumnRuleStyleValue::None),
        "hidden" => Some(ColumnRuleStyleValue::Hidden),
        "dotted" => Some(ColumnRuleStyleValue::Dotted),
        "dashed" => Some(ColumnRuleStyleValue::Dashed),
        "solid" => Some(ColumnRuleStyleValue::Solid),
        "double" => Some(ColumnRuleStyleValue::Double),
        "groove" => Some(ColumnRuleStyleValue::Groove),
        "ridge" => Some(ColumnRuleStyleValue::Ridge),
        "inset" => Some(ColumnRuleStyleValue::Inset),
        "outset" => Some(ColumnRuleStyleValue::Outset),
        _ => None,
    }
}

/// CSS direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum DirectionValue {
    /// ltr（默认值）— 从左到右。
    Ltr,
    /// rtl — 从右到左。
    Rtl,
}

/// 解析 CSS direction 属性值。
pub fn parse_direction(value: &str) -> Option<DirectionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ltr" => Some(DirectionValue::Ltr),
        "rtl" => Some(DirectionValue::Rtl),
        _ => None,
    }
}

/// CSS unicode-bidi 值。
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeBidiValue {
    /// normal（默认值）。
    Normal,
    /// embed。
    Embed,
    /// isolate。
    Isolate,
    /// bidi-override。
    BidiOverride,
    /// isolate-override。
    IsolateOverride,
    /// plaintext。
    Plaintext,
}

/// 解析 CSS unicode-bidi 属性值。
pub fn parse_unicode_bidi(value: &str) -> Option<UnicodeBidiValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(UnicodeBidiValue::Normal),
        "embed" => Some(UnicodeBidiValue::Embed),
        "isolate" => Some(UnicodeBidiValue::Isolate),
        "bidi-override" => Some(UnicodeBidiValue::BidiOverride),
        "isolate-override" => Some(UnicodeBidiValue::IsolateOverride),
        "plaintext" => Some(UnicodeBidiValue::Plaintext),
        _ => None,
    }
}

/// CSS tab-size 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TabSizeValue {
    /// 数字值（空格数）。
    Number(u32),
    /// 长度值（如 px、em）。
    Length(LengthValue),
}

/// 解析 CSS tab-size 属性值。
///
/// 支持整数（如 `4`）和长度值（如 `20px`、`1em`）。
pub fn parse_tab_size(value: &str) -> Option<TabSizeValue> {
    let value = value.trim();
    // 先尝试解析为整数
    if let Ok(n) = value.parse::<u32>() {
        return Some(TabSizeValue::Number(n));
    }
    // 再尝试解析为长度值
    parse_length(value).map(TabSizeValue::Length)
}

/// CSS overflow-wrap 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowWrapValue {
    /// normal。
    Normal,
    /// break-word。
    BreakWord,
    /// anywhere。
    Anywhere,
}

/// 解析 CSS overflow-wrap 属性值。
pub fn parse_overflow_wrap(value: &str) -> Option<OverflowWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(OverflowWrapValue::Normal),
        "break-word" => Some(OverflowWrapValue::BreakWord),
        "anywhere" => Some(OverflowWrapValue::Anywhere),
        _ => None,
    }
}

/// CSS text-align-last 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignLastValue {
    /// auto。
    Auto,
    /// start。
    Start,
    /// end。
    End,
    /// left。
    Left,
    /// right。
    Right,
    /// center。
    Center,
    /// justify。
    Justify,
}

/// 解析 CSS text-align-last 属性值。
pub fn parse_text_align_last(value: &str) -> Option<TextAlignLastValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TextAlignLastValue::Auto),
        "start" => Some(TextAlignLastValue::Start),
        "end" => Some(TextAlignLastValue::End),
        "left" => Some(TextAlignLastValue::Left),
        "right" => Some(TextAlignLastValue::Right),
        "center" => Some(TextAlignLastValue::Center),
        "justify" => Some(TextAlignLastValue::Justify),
        _ => None,
    }
}

/// CSS font-variant-numeric 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontVariantNumericValue {
    /// normal。
    Normal,
    /// ordinal。
    Ordinal,
    /// slashed-zero。
    SlashedZero,
    /// lining-nums。
    LiningNums,
    /// oldstyle-nums。
    OldstyleNums,
    /// proportional-nums。
    ProportionalNums,
    /// tabular-nums。
    TabularNums,
    /// diagonal-fractions。
    DiagonalFractions,
    /// stacked-fractions。
    StackedFractions,
}

/// 解析 CSS font-variant-numeric 属性值。
pub fn parse_font_variant_numeric(value: &str) -> Option<FontVariantNumericValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantNumericValue::Normal),
        "ordinal" => Some(FontVariantNumericValue::Ordinal),
        "slashed-zero" => Some(FontVariantNumericValue::SlashedZero),
        "lining-nums" => Some(FontVariantNumericValue::LiningNums),
        "oldstyle-nums" => Some(FontVariantNumericValue::OldstyleNums),
        "proportional-nums" => Some(FontVariantNumericValue::ProportionalNums),
        "tabular-nums" => Some(FontVariantNumericValue::TabularNums),
        "diagonal-fractions" => Some(FontVariantNumericValue::DiagonalFractions),
        "stacked-fractions" => Some(FontVariantNumericValue::StackedFractions),
        _ => None,
    }
}

/// CSS font-variant-caps 属性值。
/// https://drafts.csswg.org/css-fonts-4/#font-variant-caps-prop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontVariantCapsValue {
    /// normal。
    Normal,
    /// small-caps。
    SmallCaps,
    /// all-small-caps。
    AllSmallCaps,
    /// petite-caps。
    PetiteCaps,
    /// all-petite-caps。
    AllPetiteCaps,
    /// unicase。
    Unicase,
    /// titling-caps。
    TitlingCaps,
}

/// 解析 CSS font-variant-caps 属性值。
pub fn parse_font_variant_caps(value: &str) -> Option<FontVariantCapsValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantCapsValue::Normal),
        "small-caps" => Some(FontVariantCapsValue::SmallCaps),
        "all-small-caps" => Some(FontVariantCapsValue::AllSmallCaps),
        "petite-caps" => Some(FontVariantCapsValue::PetiteCaps),
        "all-petite-caps" => Some(FontVariantCapsValue::AllPetiteCaps),
        "unicase" => Some(FontVariantCapsValue::Unicase),
        "titling-caps" => Some(FontVariantCapsValue::TitlingCaps),
        _ => None,
    }
}

/// CSS font-variant-east-asian 属性值。
/// https://drafts.csswg.org/css-fonts-4/#font-variant-east-asian-prop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontVariantEastAsianValue {
    /// normal。
    Normal,
    /// jis78。
    Jis78,
    /// jis83。
    Jis83,
    /// jis90。
    Jis90,
    /// jis04。
    Jis04,
    /// simplified。
    Simplified,
    /// traditional。
    Traditional,
    /// full-width。
    FullWidth,
    /// proportional-width。
    ProportionalWidth,
    /// ruby。
    Ruby,
}

/// 解析 CSS font-variant-east-asian 属性值。
pub fn parse_font_variant_east_asian(value: &str) -> Option<FontVariantEastAsianValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantEastAsianValue::Normal),
        "jis78" => Some(FontVariantEastAsianValue::Jis78),
        "jis83" => Some(FontVariantEastAsianValue::Jis83),
        "jis90" => Some(FontVariantEastAsianValue::Jis90),
        "jis04" => Some(FontVariantEastAsianValue::Jis04),
        "simplified" => Some(FontVariantEastAsianValue::Simplified),
        "traditional" => Some(FontVariantEastAsianValue::Traditional),
        "full-width" => Some(FontVariantEastAsianValue::FullWidth),
        "proportional-width" => Some(FontVariantEastAsianValue::ProportionalWidth),
        "ruby" => Some(FontVariantEastAsianValue::Ruby),
        _ => None,
    }
}

/// CSS font-variant-position 属性值。
/// https://drafts.csswg.org/css-fonts-4/#font-variant-position-prop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontVariantPositionValue {
    /// normal。
    Normal,
    /// sub。
    Sub,
    /// super。
    Super,
}

/// 解析 CSS font-variant-position 属性值。
pub fn parse_font_variant_position(value: &str) -> Option<FontVariantPositionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantPositionValue::Normal),
        "sub" => Some(FontVariantPositionValue::Sub),
        "super" => Some(FontVariantPositionValue::Super),
        _ => None,
    }
}

/// 解析 CSS `font-stretch` / `font-width` 值为百分比（`normal` = 100）。
///
/// https://drafts.csswg.org/css-fonts-4/#font-width-prop
pub fn parse_font_stretch(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "normal" => Some(100.0),
        "ultra-condensed" => Some(50.0),
        "extra-condensed" => Some(62.5),
        "condensed" => Some(75.0),
        "semi-condensed" => Some(87.5),
        "semi-expanded" => Some(112.5),
        "expanded" => Some(125.0),
        "extra-expanded" => Some(150.0),
        "ultra-expanded" => Some(200.0),
        _ => value
            .strip_suffix('%')
            .and_then(|number| number.parse::<f32>().ok())
            .filter(|percentage| percentage.is_finite() && *percentage > 0.0),
    }
}

/// CSS `font-feature-settings` 中的单个 OpenType feature。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontFeatureSetting {
    /// 四字节 OpenType tag。
    pub tag: [u8; 4],
    /// feature 值（`on` = 1，`off` = 0，也可为非负整数）。
    pub value: u32,
}

/// CSS `font-feature-settings` 计算值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontFeatureSettingsValue {
    /// UA/字体默认设置。
    Normal,
    /// 显式 feature 列表，后出现的同 tag 项覆盖先出现项。
    Features(Vec<FontFeatureSetting>),
}

/// 解析 CSS `font-feature-settings`。
///
/// https://drafts.csswg.org/css-fonts-4/#font-feature-settings-prop
pub fn parse_font_feature_settings(value: &str) -> Option<FontFeatureSettingsValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some(FontFeatureSettingsValue::Normal);
    }
    if value.is_empty() {
        return None;
    }

    let mut features = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        let quote = item.as_bytes().first().copied()?;
        if quote != b'\'' && quote != b'"' {
            return None;
        }
        let close = item[1..].find(quote as char)? + 1;
        let tag_text = &item[1..close];
        if tag_text.len() != 4 || !tag_text.is_ascii() {
            return None;
        }
        let mut tag = [0; 4];
        tag.copy_from_slice(tag_text.as_bytes());
        let suffix = item[close + 1..].trim();
        let setting = if suffix.is_empty() || suffix.eq_ignore_ascii_case("on") {
            1
        } else if suffix.eq_ignore_ascii_case("off") {
            0
        } else {
            suffix.parse::<u32>().ok()?
        };
        features.push(FontFeatureSetting { tag, value: setting });
    }
    (!features.is_empty()).then_some(FontFeatureSettingsValue::Features(features))
}

/// CSS `font-variant-ligatures` 计算值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FontVariantLigaturesValue {
    /// `liga` / `clig`；`None` 表示 normal。
    pub common: Option<bool>,
    /// `dlig`；`None` 表示 normal。
    pub discretionary: Option<bool>,
    /// `hlig`；`None` 表示 normal。
    pub historical: Option<bool>,
    /// `calt`；`None` 表示 normal。
    pub contextual: Option<bool>,
}

/// 解析 CSS `font-variant-ligatures`。
///
/// https://drafts.csswg.org/css-fonts-4/#font-variant-ligatures-prop
pub fn parse_font_variant_ligatures(value: &str) -> Option<FontVariantLigaturesValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some(FontVariantLigaturesValue::default());
    }
    if value.eq_ignore_ascii_case("none") {
        return Some(FontVariantLigaturesValue {
            common: Some(false),
            discretionary: Some(false),
            historical: Some(false),
            contextual: Some(false),
        });
    }

    let mut result = FontVariantLigaturesValue::default();
    let mut saw_keyword = false;
    for keyword in value.split_ascii_whitespace() {
        let (slot, enabled) = match keyword.to_ascii_lowercase().as_str() {
            "common-ligatures" => (&mut result.common, true),
            "no-common-ligatures" => (&mut result.common, false),
            "discretionary-ligatures" => (&mut result.discretionary, true),
            "no-discretionary-ligatures" => (&mut result.discretionary, false),
            "historical-ligatures" => (&mut result.historical, true),
            "no-historical-ligatures" => (&mut result.historical, false),
            "contextual" => (&mut result.contextual, true),
            "no-contextual" => (&mut result.contextual, false),
            _ => return None,
        };
        if slot.replace(enabled).is_some() {
            return None;
        }
        saw_keyword = true;
    }
    saw_keyword.then_some(result)
}

/// https://drafts.csswg.org/css-fonts-4/#font-synthesis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontSynthesisValue {
    /// Synthesize bold weight.
    pub weight: bool,
    /// Synthesize italic style.
    pub style: bool,
    /// Synthesize small-caps.
    pub small_caps: bool,
    /// Synthesize sub/super position.
    pub position: bool,
}

impl Default for FontSynthesisValue {
    fn default() -> Self {
        Self {
            weight: true,
            style: true,
            small_caps: true,
            position: true,
        }
    }
}

/// https://drafts.csswg.org/css-fonts-4/#font-synthesis
pub fn parse_font_synthesis(value: &str) -> Option<FontSynthesisValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(FontSynthesisValue {
            weight: false,
            style: false,
            small_caps: false,
            position: false,
        });
    }

    let mut result = FontSynthesisValue {
        weight: false,
        style: false,
        small_caps: false,
        position: false,
    };
    let mut saw_keyword = false;
    for token in value.split_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "weight" => {
                if result.weight {
                    return None;
                }
                result.weight = true;
            }
            "style" => {
                if result.style {
                    return None;
                }
                result.style = true;
            }
            "small-caps" => {
                if result.small_caps {
                    return None;
                }
                result.small_caps = true;
            }
            "position" => {
                if result.position {
                    return None;
                }
                result.position = true;
            }
            _ => return None,
        }
        saw_keyword = true;
    }
    saw_keyword.then_some(result)
}
