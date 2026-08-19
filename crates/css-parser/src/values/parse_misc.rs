//! 杂项 CSS 属性值解析（display/position/flex/text/font 等枚举解析器）。
//!
//! 这些解析器历史上积累在 color.rs 中（与颜色无关），按 run-rules §5 文件大小控制
//! 拆分到此独立模块。仅依赖 types.rs 的枚举类型，自包含。

use super::*;

/// 解析 CSS display 属性值。
pub fn parse_display(value: &str) -> Option<DisplayValue> {
    let value = value.trim().to_ascii_lowercase();

    // CSS Display 3 §2.4 两值语法 `<display-outside> || <display-inside>`（`||` 合取子，
    // 顺序无关），如 `inline flex`→InlineFlex、`block flow-root`→FlowRoot、`block table`→Table。
    // 映射到既有 legacy 单 keyword 变体（无新布局基建）；单 keyword（含 inline-flex /
    // flow-root 等连字符变体）走下方既有 match。
    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.len() == 2 {
        // 顺序无关：尝试 (outside, inside) 与反转两种排列
        return display_two_value(tokens[0], tokens[1]).or_else(|| display_two_value(tokens[1], tokens[0]));
    }
    if tokens.len() > 2 {
        return None;
    }

    match value.as_str() {
        "block" => Some(DisplayValue::Block),
        "inline" => Some(DisplayValue::Inline),
        "inline-block" => Some(DisplayValue::InlineBlock),
        "flex" => Some(DisplayValue::Flex),
        "inline-flex" => Some(DisplayValue::InlineFlex),
        "grid" => Some(DisplayValue::Grid),
        "inline-grid" => Some(DisplayValue::InlineGrid),
        "none" => Some(DisplayValue::None),
        "contents" => Some(DisplayValue::Contents),
        "flow" => Some(DisplayValue::Flow),
        "flow-root" => Some(DisplayValue::FlowRoot),
        "list-item" => Some(DisplayValue::ListItem),
        "table" => Some(DisplayValue::Table),
        "inline-table" => Some(DisplayValue::InlineTable),
        "table-row" => Some(DisplayValue::TableRow),
        "table-cell" => Some(DisplayValue::TableCell),
        "table-caption" => Some(DisplayValue::TableCaption),
        "table-column" => Some(DisplayValue::TableColumn),
        "table-column-group" => Some(DisplayValue::TableColumnGroup),
        "table-row-group" => Some(DisplayValue::TableRowGroup),
        "table-header-group" => Some(DisplayValue::TableHeaderGroup),
        "table-footer-group" => Some(DisplayValue::TableFooterGroup),
        _ => None,
    }
}

/// CSS Display 3 §2.4 两值 display 的 (display-outside, display-inside) → 既有 legacy
/// DisplayValue 映射。outside ∈ {block, inline}，inside ∈ {flow, flow-root, table, flex,
/// grid}。调用方对两 token 尝试两种排列以支持 `||`（顺序无关）。输入须已小写化。
fn display_two_value(outside: &str, inside: &str) -> Option<DisplayValue> {
    let block_level = outside == "block";
    if !block_level && outside != "inline" {
        return None;
    }
    Some(match inside {
        "flow" => {
            if block_level {
                DisplayValue::Block
            } else {
                DisplayValue::Inline
            }
        }
        "flow-root" => {
            if block_level {
                DisplayValue::FlowRoot
            } else {
                DisplayValue::InlineBlock
            }
        }
        "flex" => {
            if block_level {
                DisplayValue::Flex
            } else {
                DisplayValue::InlineFlex
            }
        }
        "grid" => {
            if block_level {
                DisplayValue::Grid
            } else {
                DisplayValue::InlineGrid
            }
        }
        "table" => {
            if block_level {
                DisplayValue::Table
            } else {
                DisplayValue::InlineTable
            }
        }
        // CSS Display 3 §2.4 <display-listitem>：`<display-outside>? list-item`。
        // `block list-item`/`inline list-item` 均映射 ListItem（ZW 单变体不区分 block/inline
        // level list-item——inline-level list item 系建模缺口，此处按 block-level ListItem
        // 近似，严格优于声明被丢；单 keyword `list-item` 已走 fast-path）。
        "list-item" => DisplayValue::ListItem,
        _ => return None,
    })
}

/// 解析 CSS position 属性值。
pub fn parse_position(value: &str) -> Option<PositionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "static" => Some(PositionValue::Static),
        "relative" => Some(PositionValue::Relative),
        "absolute" => Some(PositionValue::Absolute),
        "fixed" => Some(PositionValue::Fixed),
        "sticky" => Some(PositionValue::Sticky),
        _ => None,
    }
}

/// 解析 CSS overflow 属性值。
pub fn parse_overflow(value: &str) -> Option<OverflowValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(OverflowValue::Visible),
        "hidden" => Some(OverflowValue::Hidden),
        "scroll" => Some(OverflowValue::Scroll),
        "auto" => Some(OverflowValue::Auto),
        "clip" => Some(OverflowValue::Clip),
        _ => None,
    }
}

/// 解析 CSS overflow-clip-margin 值（CSS Overflow 3 §3）。
///
/// 文法 `<visual-box> || <length>`——box ∈ {content-box, padding-box, border-box}
///（缺省 padding-box）+ length（缺省 0px）。`||` = 二者任意顺序、各至多一次。
/// 非法值（>2 token / 重复 box / 重复 length / 未知 token / length 不可解析）→ None
///（整条声明按解析错误丢）。driving: css-overflow/overflow-clip-margin-*。
pub fn parse_overflow_clip_margin(value: &str) -> Option<OverflowClipMarginValue> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    if parts.len() > 2 {
        return None;
    }
    let mut box_kind: Option<OverflowClipMarginBox> = None;
    let mut length: Option<LengthValue> = None;
    for p in parts {
        match p.trim() {
            "content-box" if box_kind.is_none() => box_kind = Some(OverflowClipMarginBox::ContentBox),
            "padding-box" if box_kind.is_none() => box_kind = Some(OverflowClipMarginBox::PaddingBox),
            "border-box" if box_kind.is_none() => box_kind = Some(OverflowClipMarginBox::BorderBox),
            _ => {
                if length.is_some() {
                    return None;
                }
                length = Some(parse_length(p)?);
            }
        }
    }
    Some(OverflowClipMarginValue {
        box_kind: box_kind.unwrap_or(OverflowClipMarginBox::PaddingBox),
        length: length.unwrap_or(LengthValue::Px(0.0)),
    })
}

/// 解析 CSS float 属性值。
pub fn parse_float(value: &str) -> Option<FloatValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(FloatValue::None),
        "left" => Some(FloatValue::Left),
        "right" => Some(FloatValue::Right),
        "inline-start" => Some(FloatValue::InlineStart),
        "inline-end" => Some(FloatValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS clear 属性值。
pub fn parse_clear(value: &str) -> Option<ClearValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(ClearValue::None),
        "left" => Some(ClearValue::Left),
        "right" => Some(ClearValue::Right),
        "both" => Some(ClearValue::Both),
        "inline-start" => Some(ClearValue::InlineStart),
        "inline-end" => Some(ClearValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS list-style-type 属性值。
pub fn parse_list_style_type(value: &str) -> Option<ListStyleTypeValue> {
    // list-style-type: <string>（CSS Lists 3）：引号字符串作为固定标记文本（每个 li 同值）。
    // 须在 keyword match 之前处理（引号起始的值非合法 ident，会落入 _ 被 is_custom_counter_name
    // 拒绝丢失）。支持 "..." 与 '...'，空串合法（无标记）。
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        return Some(ListStyleTypeValue::String(trimmed[1..trimmed.len() - 1].to_string()));
    }
    match value.trim().to_lowercase().as_str() {
        "disc" => Some(ListStyleTypeValue::Disc),
        "circle" => Some(ListStyleTypeValue::Circle),
        "square" => Some(ListStyleTypeValue::Square),
        "decimal" => Some(ListStyleTypeValue::Decimal),
        "decimal-leading-zero" => Some(ListStyleTypeValue::DecimalLeadingZero),
        "lower-roman" => Some(ListStyleTypeValue::LowerRoman),
        "upper-roman" => Some(ListStyleTypeValue::UpperRoman),
        "lower-alpha" | "lower-latin" => Some(ListStyleTypeValue::LowerAlpha),
        "upper-alpha" | "upper-latin" => Some(ListStyleTypeValue::UpperAlpha),
        "lower-greek" => Some(ListStyleTypeValue::LowerGreek),
        "persian" => Some(ListStyleTypeValue::Persian),
        // CSS Counter Styles 3 §6.1：armenian 是 upper-armenian 的别名（大写亚美尼亚数字）。
        "armenian" | "upper-armenian" => Some(ListStyleTypeValue::Armenian),
        "lower-armenian" => Some(ListStyleTypeValue::LowerArmenian),
        "georgian" => Some(ListStyleTypeValue::Georgian),
        "hebrew" => Some(ListStyleTypeValue::Hebrew),
        "arabic-indic" => Some(ListStyleTypeValue::ArabicIndic),
        // R2471：CSS Counter Styles 3 §6.1 预定义 numeric system（十进制位数字替换）。
        "devanagari" => Some(ListStyleTypeValue::Devanagari),
        "bengali" => Some(ListStyleTypeValue::Bengali),
        "gujarati" => Some(ListStyleTypeValue::Gujarati),
        "gurmukhi" => Some(ListStyleTypeValue::Gurmukhi),
        "kannada" => Some(ListStyleTypeValue::Kannada),
        "malayalam" => Some(ListStyleTypeValue::Malayalam),
        "tamil" => Some(ListStyleTypeValue::Tamil),
        "telugu" => Some(ListStyleTypeValue::Telugu),
        "lao" => Some(ListStyleTypeValue::Lao),
        "khmer" => Some(ListStyleTypeValue::Khmer),
        "myanmar" => Some(ListStyleTypeValue::Myanmar),
        // R2472：cjk-decimal（CJK ideographic digits，非连续 lookup）。cambodian ≡ khmer 别名。
        "cjk-decimal" => Some(ListStyleTypeValue::CjkDecimal),
        "cambodian" => Some(ListStyleTypeValue::Khmer),
        "none" => Some(ListStyleTypeValue::None),
        _ => {
            // R2392：非 builtin 的 `<custom-ident>` 视为自定义计数器样式名（@counter-style）。
            // 渲染时查 CounterStyleRegistry，未命中走 fallback（默认 decimal）。
            // CSS-wide 关键字（inherit/initial/unset/revert/revert-layer）不应到此（cascade
            // 已解析），但防御性排除；非法 token（数字起始/空）→ None（声明丢弃）。
            // kill-switch `ZW_COUNTER_STYLE=0`（default-on）：关闭则不识别自定义名 → None
            // → 声明丢弃 → 回退 builtin decimal（旧行为，零回归）。
            if std::env::var("ZW_COUNTER_STYLE").as_deref() == Ok("0") {
                return None;
            }
            let name = value.trim();
            if is_custom_counter_name(name) {
                Some(ListStyleTypeValue::Custom(name.to_string()))
            } else {
                None
            }
        }
    }
}

/// 判定字符串是否为合法的自定义计数器样式名（CSS `<custom-ident>`，CSS Counter Styles 3）。
/// driving: R2392。允许字母/连字符起始 + 字母数字连字符；排除 CSS-wide 关键字与 `none`。
fn is_custom_counter_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // 排除 CSS-wide 关键字（防御；正常经 cascade 已不至此）。
    if matches!(
        s.to_ascii_lowercase().as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "none" | "default"
    ) {
        return false;
    }
    // 首字符须为字母或连字符（CSS ident-start）；余下为 ident 字符。
    let first = s.chars().next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '-' || first == '_') {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// 解析 CSS list-style-position 属性值。
pub fn parse_list_style_position(value: &str) -> Option<ListStylePositionValue> {
    match value.trim().to_lowercase().as_str() {
        "outside" => Some(ListStylePositionValue::Outside),
        "inside" => Some(ListStylePositionValue::Inside),
        _ => None,
    }
}

/// 解析 CSS flex-direction 属性值。
pub fn parse_flex_direction(value: &str) -> Option<FlexDirectionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "row" => Some(FlexDirectionValue::Row),
        "row-reverse" => Some(FlexDirectionValue::RowReverse),
        "column" => Some(FlexDirectionValue::Column),
        "column-reverse" => Some(FlexDirectionValue::ColumnReverse),
        _ => None,
    }
}

/// 解析 CSS flex-wrap 属性值。
pub fn parse_flex_wrap(value: &str) -> Option<FlexWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "nowrap" => Some(FlexWrapValue::Nowrap),
        "wrap" => Some(FlexWrapValue::Wrap),
        "wrap-reverse" => Some(FlexWrapValue::WrapReverse),
        _ => None,
    }
}

/// 解析 CSS justify-content / align-items / align-self 属性值。
pub fn parse_alignment(value: &str) -> Option<AlignmentValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(AlignmentValue::Auto),
        // R2383：CSS Box Align 3 normal（justify-content/align-items/align-self 初始值）。
        "normal" => Some(AlignmentValue::Normal),
        "flex-start" => Some(AlignmentValue::FlexStart),
        "flex-end" => Some(AlignmentValue::FlexEnd),
        "center" => Some(AlignmentValue::Center),
        "space-between" => Some(AlignmentValue::SpaceBetween),
        "space-around" => Some(AlignmentValue::SpaceAround),
        "space-evenly" => Some(AlignmentValue::SpaceEvenly),
        "stretch" => Some(AlignmentValue::Stretch),
        "start" => Some(AlignmentValue::Start),
        "end" => Some(AlignmentValue::End),
        "baseline" => Some(AlignmentValue::Baseline),
        _ => None,
    }
}

/// 解析 CSS box-sizing 属性值。
pub fn parse_box_sizing(value: &str) -> Option<BoxSizingValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "content-box" => Some(BoxSizingValue::ContentBox),
        "border-box" => Some(BoxSizingValue::BorderBox),
        _ => None,
    }
}

/// 解析 CSS visibility 属性值。
pub fn parse_visibility(value: &str) -> Option<VisibilityValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(VisibilityValue::Visible),
        "hidden" => Some(VisibilityValue::Hidden),
        "collapse" => Some(VisibilityValue::Collapse),
        _ => None,
    }
}

/// 解析 CSS content-visibility 属性值（CSS Containment Module Level 2）。
///
/// 大小写不敏感（与 `parse_visibility` 同色，实际匹配走 `to_ascii_lowercase`）。
pub fn parse_content_visibility(value: &str) -> Option<ContentVisibilityValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(ContentVisibilityValue::Visible),
        "hidden" => Some(ContentVisibilityValue::Hidden),
        "auto" => Some(ContentVisibilityValue::Auto),
        _ => None,
    }
}

/// 解析 CSS word-break 属性值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS line-break 属性值（CSS Text 3 §5.3）。
pub fn parse_line_break(value: &str) -> Option<LineBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(LineBreakValue::Auto),
        "loose" => Some(LineBreakValue::Loose),
        "normal" => Some(LineBreakValue::Normal),
        "strict" => Some(LineBreakValue::Strict),
        "anywhere" => Some(LineBreakValue::Anywhere),
        _ => None,
    }
}

/// 解析 CSS writing-mode 属性值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        // R1785：sideways-rl/lr 的 block-flow 方向等价 vertical-rl/lr（仅 glyph rotation 不同），
        // 在 parse 时规范化为对应 vertical 值——使 sideways 走已验证的 vertical 块流路径
        //（解 block-flow-direction-slr/srl 86% diff）。字形旋转（line-box-direction）是
        // paint-side 独立关注，未实现，留 future。
        "sideways-rl" => Some(WritingModeValue::VerticalRl),
        "sideways-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
        _ => None,
    }
}

/// 解析 CSS text-decoration-style 值。
pub fn parse_text_decoration_style(value: &str) -> Option<TextDecorationStyleValue> {
    match value.to_ascii_lowercase().as_str() {
        "solid" => Some(TextDecorationStyleValue::Solid),
        "double" => Some(TextDecorationStyleValue::Double),
        "dotted" => Some(TextDecorationStyleValue::Dotted),
        "dashed" => Some(TextDecorationStyleValue::Dashed),
        "wavy" => Some(TextDecorationStyleValue::Wavy),
        _ => None,
    }
}

/// 解析 CSS text-decoration-thickness 值（CSS Text Decoration 4 §2.3）。R1402。
///
/// 支持 `auto` / `from-font` 关键字与 `<length>`（如 `2px`）。em/rem/% 须 computed 层
/// 字号上下文，指定值层仅认 Px（driver test text-decoration-thickness-length-rounding 用 px）。
pub fn parse_text_decoration_thickness(value: &str) -> Option<TextDecorationThicknessValue> {
    let v = value.trim();
    match v.to_ascii_lowercase().as_str() {
        "auto" => Some(TextDecorationThicknessValue::Auto),
        "from-font" => Some(TextDecorationThicknessValue::FromFont),
        _ => match parse_length(v) {
            Some(LengthValue::Px(n)) => Some(TextDecorationThicknessValue::Length(n)),
            _ => None,
        },
    }
}

/// 解析 CSS text-decoration-inset 值（CSS Text Decoration 4 §2.4）。R1607。
///
/// 语法：`<length>{1,2}`。1 个值 → start=end；2 个值 → (start, end)。
/// 仅接受真 `<length>`（px/em/rem/ch/v*/%），拒绝 auto/min-content/max-content
/// 等关键字（inset 无关键字值）。负值=向外延伸。
pub fn parse_text_decoration_inset(value: &str) -> Option<TextDecorationInsetValue> {
    // 仅认数值长度，过滤 auto/min-content/max-content 等关键字变体。
    let parse_one = |s: &str| -> Option<LengthValue> {
        match parse_length(s)? {
            v @ (LengthValue::Px(_)
            | LengthValue::Em(_)
            | LengthValue::Rem(_)
            | LengthValue::Cap(_)
            | LengthValue::Rcap(_)
            | LengthValue::Ch(_)
            | LengthValue::Ic(_)
            | LengthValue::Ric(_)
            | LengthValue::Percentage(_)
            | LengthValue::Vh(_)
            | LengthValue::Vw(_)
            | LengthValue::Vmin(_)
            | LengthValue::Vmax(_)) => Some(v),
            // auto/min-content/max-content/calc/fit-content 等非纯长度 → 拒绝。
            _ => None,
        }
    };
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_one(parts[0])?;
            Some(TextDecorationInsetValue {
                start: v.clone(),
                end: v,
            })
        }
        2 => Some(TextDecorationInsetValue {
            start: parse_one(parts[0])?,
            end: parse_one(parts[1])?,
        }),
        _ => None,
    }
}

/// 解析 CSS text-underline-offset 值（CSS Text Decoration 4 §2.5）。
///
/// 支持 `auto` 与 `<length-percentage>`（如 `11px` / `0.5em` / `50%`）。负值=上抬。
/// 仅认数值长度/百分比，拒绝非法关键字（from-font 等非该属性合法值）。
pub fn parse_text_underline_offset(value: &str) -> Option<TextUnderlineOffsetValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(TextUnderlineOffsetValue::Auto);
    }
    match parse_length(v)? {
        v @ (LengthValue::Px(_)
        | LengthValue::Em(_)
        | LengthValue::Rem(_)
        | LengthValue::Cap(_)
        | LengthValue::Rcap(_)
        | LengthValue::Ch(_)
        | LengthValue::Ic(_)
        | LengthValue::Ric(_)
        | LengthValue::Percentage(_)
        | LengthValue::Vh(_)
        | LengthValue::Vw(_)
        | LengthValue::Vmin(_)
        | LengthValue::Vmax(_)) => Some(TextUnderlineOffsetValue::Length(v)),
        // calc/fit-content 等非纯长度 → 拒绝。
        _ => None,
    }
}

/// 解析 CSS text-emphasis-style 值（CSS Text Decoration 3 §3.1）。
/// `none` | [ [ filled | open ] || [ dot | circle | double-circle | triangle | sesame ] ] | <string>
/// 关键字组合解析为对应标记字符（filled dot → '•' 等）；<string> 取首字符。
pub fn parse_text_emphasis_style(value: &str) -> Option<TextEmphasisStyleValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(TextEmphasisStyleValue::None);
    }
    // <string>："..." 取首字符
    if v.len() >= 3 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        return Some(TextEmphasisStyleValue::Char(inner.chars().next().unwrap_or('\u{2022}')));
    }
    // 关键字：[filled|open] [shape] 任意顺序，各可缺省（filled 默认 dot，shape 缺省 filled）
    let mut filled: Option<bool> = None;
    let mut shape: Option<&str> = None;
    for tok in v.split_whitespace() {
        match tok.to_ascii_lowercase().as_str() {
            "filled" => filled = Some(true),
            "open" => filled = Some(false),
            "dot" | "circle" | "double-circle" | "triangle" | "sesame" => {
                shape = Some(match tok {
                    "dot" => "dot",
                    "circle" => "circle",
                    "double-circle" => "double-circle",
                    "triangle" => "triangle",
                    _ => "sesame",
                })
            }
            _ => return None,
        }
    }
    let filled = filled.unwrap_or(true);
    let shape = shape.unwrap_or("dot");
    let ch = match (filled, shape) {
        (true, "dot") => '\u{2022}',            // •
        (false, "dot") => '\u{25E6}',           // ◦
        (true, "circle") => '\u{25CF}',         // ●
        (false, "circle") => '\u{25CB}',        // ○
        (true, "double-circle") => '\u{25C9}',  // ◉
        (false, "double-circle") => '\u{25CE}', // ◎
        (true, "triangle") => '\u{25B2}',       // ▲
        (false, "triangle") => '\u{25B3}',      // △
        (true, "sesame") => '\u{FE45}',         // ﹅
        (false, "sesame") => '\u{FE46}',        // ﹆
        _ => '\u{2022}',
    };
    Some(TextEmphasisStyleValue::Char(ch))
}

/// 解析 CSS text-emphasis-position 值（CSS Text Decoration 3 §3.2）。
/// `[ over | under ] && [ right | left ]`，各可缺省（默认 over right）。
pub fn parse_text_emphasis_position(value: &str) -> Option<TextEmphasisPositionValue> {
    let mut over: Option<bool> = None;
    let mut right: Option<bool> = None;
    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    for tok in tokens {
        match tok.to_ascii_lowercase().as_str() {
            "over" if over.is_none() => over = Some(true),
            "under" if over.is_none() => over = Some(false),
            "right" if right.is_none() => right = Some(true),
            "left" if right.is_none() => right = Some(false),
            _ => return None,
        }
    }
    let over = over.unwrap_or(true);
    use TextEmphasisPositionValue::*;
    Some(match (over, right) {
        (true, Some(false)) => OverLeft,
        (true, _) => OverRight,
        (false, Some(false)) => UnderLeft,
        (false, _) => UnderRight,
    })
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS text-indent 属性值。
///
/// 支持长度值（如 `2em`、`20px`）和百分比值（如 `10%`）。
/// 不支持 `auto` 关键字。
pub fn parse_text_indent(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_length(v)
}

/// 解析 CSS letter-spacing / word-spacing 值。
/// "normal" 映射为 LengthValue::Px(0.0)。
pub fn parse_spacing(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("normal") {
        return Some(LengthValue::Px(0.0));
    }
    parse_length(v)
}

/// 解析 CSS font-weight 属性值。
pub fn parse_font_weight(value: &str) -> Option<FontWeightValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bold" => Some(FontWeightValue::Bold),
        "normal" => Some(FontWeightValue::Normal),
        "bolder" => Some(FontWeightValue::Bolder),
        "lighter" => Some(FontWeightValue::Lighter),
        s => {
            let w: u16 = s.parse().ok()?;
            if (100..=900).contains(&w) {
                Some(FontWeightValue::Absolute(w))
            } else {
                None
            }
        }
    }
}

/// 解析 CSS font-style 属性值。
pub fn parse_font_style(value: &str) -> Option<FontStyleValue> {
    // CSS 关键字大小写不敏感（NORMAL/Italic/OBLIQUE ≡ normal/italic/oblique）。归一化小写后匹配。
    let value = value.trim().to_ascii_lowercase();
    if value == "normal" {
        Some(FontStyleValue::Normal)
    } else if value == "italic" {
        Some(FontStyleValue::Italic)
    } else if starts_with_oblique_token(&value) {
        let angle_str = value.strip_prefix("oblique")?.trim();
        if angle_str.is_empty() {
            Some(FontStyleValue::Oblique(None))
        } else {
            // 处理 "(angle)" 或 "(angledeg)" 形式
            let angle_str = angle_str
                .strip_prefix('(')
                .unwrap_or(angle_str)
                .strip_suffix(')')
                .unwrap_or(angle_str);
            let angle: f64 = angle_str.trim_end_matches("deg").trim().parse().ok()?;
            Some(FontStyleValue::Oblique(Some(angle)))
        }
    } else {
        None
    }
}

fn starts_with_oblique_token(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("oblique") else {
        return false;
    };
    rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with('(')
}
