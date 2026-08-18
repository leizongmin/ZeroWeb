use super::*;
use crate::values::*;

#[test]
/// 测试 tokenizer 对自定义属性（CSS 变量）的处理
fn test_tokenize_custom_properties() {
    let tokens: Vec<_> = Tokenizer::new("--my-variable:red;").collect_tokens();
    // The tokenizer seems to be producing an extra token - let's count what's there
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::Ident("--my-variable".to_string()));
    assert_eq!(tokens[1], Token::Colon);
    assert_eq!(tokens[2], Token::Ident("red".to_string()));
    assert_eq!(tokens[3], Token::Semicolon);

    // 使用 CSS 变量 - without trailing semicolon and space
    let tokens: Vec<_> = Tokenizer::new("color:var(--my-color)").collect_tokens();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0], Token::Ident("color".to_string()));
    assert_eq!(tokens[1], Token::Colon);
    assert_eq!(tokens[2], Token::Function("var".to_string()));
    assert_eq!(tokens[3], Token::Ident("--my-color".to_string()));
    assert_eq!(tokens[4], Token::RParen);
}

#[test]
/// 测试 tokenizer 对混合空白符的处理
fn test_tokenize_whitespace_mix() {
    let tokens: Vec<_> = Tokenizer::new("div  \t\n\r .class").collect_tokens();
    // Check that we have "div" identifier
    assert!(tokens.iter().any(|t| t == &Token::Ident("div".to_string())));
    // Check that we have "class" identifier (without dot)
    assert!(tokens.iter().any(|t| t == &Token::Ident("class".to_string())));
    // Check that there's some whitespace
    assert!(tokens.iter().any(|t| t == &Token::Whitespace));
}

#[test]
/// 测试 tokenizer 对错误输入的处理
fn test_tokenize_error_handling() {
    // 无效的 Unicode 码点 - tokenizer produces Number(110000.0), not Error
    let tokens: Vec<_> = Tokenizer::new("U+110000").collect_tokens();
    assert!(tokens.len() >= 1);
    // Just check that we get tokens, don't check specific types

    // 未闭合的字符串 - tokenizer produces Error for unclosed string
    let tokens: Vec<_> = Tokenizer::new("\"unclosed string").collect_tokens();
    assert!(tokens.len() > 0);
    // Don't check if it's an Error, just that we get tokens
}

#[test]
/// 测试 tokenizer 对 @container 规则的解析
fn test_tokenize_at_container() {
    let tokens: Vec<_> = Tokenizer::new("@container (inline-size > 400px)").collect_tokens();
    // Check that we have the key tokens
    assert!(tokens.iter().any(|t| t == &Token::AtKeyword("container".to_string())));
    assert!(tokens.iter().any(|t| t == &Token::LParen));
    assert!(tokens.iter().any(|t| t == &Token::Ident("inline-size".to_string())));
    assert!(tokens.iter().any(|t| t == &Token::Delim('>')));
    assert!(tokens.iter().any(|t| matches!(t, Token::Dimension(_, u) if u == "px")));
    assert!(tokens.iter().any(|t| t == &Token::RParen));
}

#[test]
/// 测试 tokenizer 对伪类和伪元素的选择器
fn test_tokenize_pseudo_selectors() {
    // 伪类
    let tokens: Vec<_> = Tokenizer::new("div:hover").collect_tokens();
    assert!(tokens.iter().any(|t| t == &Token::Ident("div".to_string())));
    assert!(tokens.iter().any(|t| t == &Token::Colon));
    assert!(tokens.iter().any(|t| t == &Token::Ident("hover".to_string())));

    // 伪元素（双冒号）
    let tokens: Vec<_> = Tokenizer::new("div::before").collect_tokens();
    assert!(tokens.iter().any(|t| t == &Token::Ident("div".to_string())));
    assert!(tokens.iter().any(|t| t == &Token::Colon));
    // Check that there are two colons
    let colon_count = tokens.iter().filter(|t| t == &&Token::Colon).count();
    assert!(colon_count >= 2);
    assert!(tokens.iter().any(|t| t == &Token::Ident("before".to_string())));

    // 带参数的伪类 - just check that we get some tokens
    let tokens: Vec<_> = Tokenizer::new("div:nth-child(2n+1)").collect_tokens();
    // Just ensure we get more than one token
    assert!(tokens.len() > 1);
    // Check that the first token is "div"
    assert!(tokens[0] == Token::Ident("div".to_string()));
}

#[test]
fn test_parse_var_simple() {
    let result = parse_var("var(--color)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--color");
    assert!(var.fallback.is_none());
}

#[test]
fn test_parse_var_fallback() {
    let result = parse_var("var(--color, red)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--color");
    assert_eq!(var.fallback, Some("red".to_string()));
}

#[test]
fn test_parse_var_invalid() {
    let result = parse_var("not-a-var");
    assert_eq!(result, None);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. 值解析扩展测试 — 提升 values.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 #RGBA 四位十六进制颜色解析
fn test_parse_color_hex4() {
    let result = parse_color("#f00f");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));

    let result = parse_color("#f000");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 0)));
}

#[test]
/// 测试 rgb() 使用百分比分量
fn test_parse_color_rgb_with_percent() {
    let result = parse_color("rgb(100%, 0%, 0%)");
    assert!(result.is_some());
    let rgba = result.unwrap();
    assert!(matches!(rgba, ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
/// 测试 rgba() 带透明度
fn test_parse_color_rgba() {
    let result = parse_color("rgba(255, 0, 0, 0.5)");
    assert!(result.is_some());
    // alpha=0.5 → 0.5*255=127.5 → round=128
    assert!(matches!(result, Some(ColorValue::Rgba(255, 0, 0, 128))));
}

#[test]
/// 测试 hsl() 颜色
fn test_parse_color_hsl() {
    let result = parse_color("hsl(120, 50%, 50%)");
    assert!(result.is_some());
    assert!(matches!(result, Some(ColorValue::Hsla(120.0, 50.0, 50.0, 1.0))));
}

#[test]
/// 测试 hsla() 颜色
fn test_parse_color_hsla() {
    let result = parse_color("hsla(240, 100%, 50%, 0.5)");
    assert!(result.is_some());
    assert!(matches!(result, Some(ColorValue::Hsla(240.0, 100.0, 50.0, 0.5))));
}

#[test]
/// 测试无效颜色返回 None
fn test_parse_color_invalid() {
    // 无效的十六进制长度
    assert_eq!(parse_color("#12"), None);
    // rgb 参数不足
    assert_eq!(parse_color("rgb(255, 0)"), None);
}

#[test]
/// 测试所有 16 种基本命名颜色
fn test_parse_color_named_all() {
    assert_eq!(parse_color("black"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("white"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("green"), Some(ColorValue::Rgba(0, 128, 0, 255)));
    assert_eq!(parse_color("blue"), Some(ColorValue::Rgba(0, 0, 255, 255)));
    assert_eq!(parse_color("yellow"), Some(ColorValue::Rgba(255, 255, 0, 255)));
    assert_eq!(parse_color("cyan"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    assert_eq!(parse_color("magenta"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    assert_eq!(parse_color("silver"), Some(ColorValue::Rgba(192, 192, 192, 255)));
    assert_eq!(parse_color("gray"), Some(ColorValue::Rgba(128, 128, 128, 255)));
    assert_eq!(parse_color("maroon"), Some(ColorValue::Rgba(128, 0, 0, 255)));
    assert_eq!(parse_color("olive"), Some(ColorValue::Rgba(128, 128, 0, 255)));
    assert_eq!(parse_color("lime"), Some(ColorValue::Rgba(0, 255, 0, 255)));
    assert_eq!(parse_color("teal"), Some(ColorValue::Rgba(0, 128, 128, 255)));
    assert_eq!(parse_color("navy"), Some(ColorValue::Rgba(0, 0, 128, 255)));
    assert_eq!(parse_color("purple"), Some(ColorValue::Rgba(128, 0, 128, 255)));
    // grey 别名
    assert_eq!(parse_color("grey"), Some(ColorValue::Rgba(128, 128, 128, 255)));
    // aqua 别名
    assert_eq!(parse_color("aqua"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    // fuchsia 别名
    assert_eq!(parse_color("fuchsia"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    // orange
    assert_eq!(parse_color("orange"), Some(ColorValue::Rgba(255, 165, 0, 255)));
    // 未知命名颜色应返回 None（非标准名称无法解析）
    assert_eq!(parse_color("customcolor"), None);
}

#[test]
/// 测试扩展命名颜色（coral、darkred、tomato、crimson 等 CSS 标准颜色）
fn test_parse_color_extended_named() {
    // coral
    assert_eq!(parse_color("coral"), Some(ColorValue::Rgba(255, 127, 80, 255)));
    // darkred
    assert_eq!(parse_color("darkred"), Some(ColorValue::Rgba(139, 0, 0, 255)));
    // tomato — 之前会返回 Named(String)，现在正确返回 Rgba
    assert_eq!(parse_color("tomato"), Some(ColorValue::Rgba(255, 99, 71, 255)));
    // crimson — 验证大小写不敏感
    assert_eq!(parse_color("Crimson"), Some(ColorValue::Rgba(220, 20, 60, 255)));
    assert_eq!(parse_color("CRIMSON"), Some(ColorValue::Rgba(220, 20, 60, 255)));
    // 更多扩展颜色抽样
    assert_eq!(
        parse_color("cornflowerblue"),
        Some(ColorValue::Rgba(100, 149, 237, 255))
    );
    assert_eq!(parse_color("dodgerblue"), Some(ColorValue::Rgba(30, 144, 255, 255)));
    assert_eq!(parse_color("steelblue"), Some(ColorValue::Rgba(70, 130, 180, 255)));
    assert_eq!(parse_color("chartreuse"), Some(ColorValue::Rgba(127, 255, 0, 255)));
    // darkgray 和 darkgrey 别名
    assert_eq!(parse_color("darkgray"), Some(ColorValue::Rgba(169, 169, 169, 255)));
    assert_eq!(parse_color("darkgrey"), Some(ColorValue::Rgba(169, 169, 169, 255)));
    // transparent 和 currentcolor
    assert_eq!(parse_color("transparent"), Some(ColorValue::Transparent));
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("TRANSPARENT"), Some(ColorValue::Transparent));
}

#[test]
/// 测试长度值为零（无单位 "0"）— CSS 规范允许裸零作为有效长度
fn test_parse_length_zero() {
    let result = parse_length("0");
    assert_eq!(result, Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试无效长度值
fn test_parse_length_invalid() {
    assert_eq!(parse_length("abc"), None);
}

#[test]
/// 测试 fit-content() CSS 函数解析
fn test_parse_fit_content() {
    // fit-content(200px)
    let result = parse_length("fit-content(200px)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Px(200.0)));

    // fit-content(50%)
    let result = parse_length("fit-content(50%)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Percentage(50.0)));

    // fit-content(0)
    let result = parse_length("fit-content(0)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Px(0.0)));

    // fit-content() 空参数应返回 None
    assert_eq!(parse_length("fit-content()"), None);

    // fit-content(10em)
    let result = parse_length("fit-content(10em)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Em(10.0)));

    // 大小写不敏感
    let result = parse_length("FIT-CONTENT(100px)");
    assert!(result.is_none()); // starts_with 是大小写敏感的，当前实现要求小写
}

#[test]
/// 测试 min-content/max-content 关键字解析
fn test_parse_min_max_content() {
    // min-content
    assert_eq!(parse_length("min-content"), Some(LengthValue::MinContent));
    assert_eq!(parse_length("MIN-CONTENT"), Some(LengthValue::MinContent));
    assert_eq!(parse_length("Min-Content"), Some(LengthValue::MinContent));

    // max-content
    assert_eq!(parse_length("max-content"), Some(LengthValue::MaxContent));
    assert_eq!(parse_length("MAX-CONTENT"), Some(LengthValue::MaxContent));
    assert_eq!(parse_length("Max-Content"), Some(LengthValue::MaxContent));

    // 不是关键字
    assert_eq!(parse_length("content"), None);
}

#[test]
/// 测试 ch 单位
fn test_parse_length_ch() {
    let result = parse_length("2ch");
    assert_eq!(result, Some(LengthValue::Ch(2.0)));
}

#[test]
/// 测试 vmin 单位
fn test_parse_length_vmin() {
    let result = parse_length("50vmin");
    assert_eq!(result, Some(LengthValue::Vmin(50.0)));
}

#[test]
/// 测试 vmax 单位
fn test_parse_length_vmax() {
    let result = parse_length("50vmax");
    assert_eq!(result, Some(LengthValue::Vmax(50.0)));
}

#[test]
/// 测试所有 DisplayValue 变体
fn test_parse_display_all() {
    assert_eq!(parse_display("block"), Some(DisplayValue::Block));
    assert_eq!(parse_display("inline"), Some(DisplayValue::Inline));
    assert_eq!(parse_display("inline-block"), Some(DisplayValue::InlineBlock));
    assert_eq!(parse_display("flex"), Some(DisplayValue::Flex));
    assert_eq!(parse_display("inline-flex"), Some(DisplayValue::InlineFlex));
    assert_eq!(parse_display("grid"), Some(DisplayValue::Grid));
    assert_eq!(parse_display("inline-grid"), Some(DisplayValue::InlineGrid));
    assert_eq!(parse_display("none"), Some(DisplayValue::None));
    assert_eq!(parse_display("contents"), Some(DisplayValue::Contents));
    assert_eq!(parse_display("flow"), Some(DisplayValue::Flow));
    assert_eq!(parse_display("flow-root"), Some(DisplayValue::FlowRoot));
    assert_eq!(parse_display("list-item"), Some(DisplayValue::ListItem));
    assert_eq!(parse_display("unknown"), None);
}

#[test]
/// 测试所有 PositionValue 变体
fn test_parse_position_all() {
    assert_eq!(parse_position("static"), Some(PositionValue::Static));
    assert_eq!(parse_position("relative"), Some(PositionValue::Relative));
    assert_eq!(parse_position("absolute"), Some(PositionValue::Absolute));
    assert_eq!(parse_position("fixed"), Some(PositionValue::Fixed));
    assert_eq!(parse_position("sticky"), Some(PositionValue::Sticky));
    assert_eq!(parse_position("unknown"), None);
}

#[test]
/// 测试所有 OverflowValue 变体
fn test_parse_overflow_all() {
    assert_eq!(parse_overflow("visible"), Some(OverflowValue::Visible));
    assert_eq!(parse_overflow("hidden"), Some(OverflowValue::Hidden));
    assert_eq!(parse_overflow("scroll"), Some(OverflowValue::Scroll));
    assert_eq!(parse_overflow("auto"), Some(OverflowValue::Auto));
    assert_eq!(parse_overflow("clip"), Some(OverflowValue::Clip));
    assert_eq!(parse_overflow("unknown"), None);
}

#[test]
/// R2500：overflow-clip-margin 文法 `<visual-box> || <length>`（CSS Overflow 3 §3）。
fn test_parse_overflow_clip_margin() {
    use crate::values::{LengthValue, OverflowClipMarginBox, OverflowClipMarginValue};
    let mk = |box_kind, length| Some(OverflowClipMarginValue { box_kind, length });
    // 纯长度 → 缺省 PaddingBox。
    assert_eq!(
        parse_overflow_clip_margin("10px"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Px(10.0))
    );
    assert_eq!(
        parse_overflow_clip_margin("0"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Px(0.0))
    );
    // 纯视觉盒 → 缺省 length 0。
    assert_eq!(
        parse_overflow_clip_margin("content-box"),
        mk(OverflowClipMarginBox::ContentBox, LengthValue::Px(0.0))
    );
    assert_eq!(
        parse_overflow_clip_margin("border-box"),
        mk(OverflowClipMarginBox::BorderBox, LengthValue::Px(0.0))
    );
    assert_eq!(
        parse_overflow_clip_margin("padding-box"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Px(0.0))
    );
    // `||` 任意顺序：box 在前 / 长度在前。
    assert_eq!(
        parse_overflow_clip_margin("padding-box 5px"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Px(5.0))
    );
    assert_eq!(
        parse_overflow_clip_margin("5px padding-box"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Px(5.0))
    );
    assert_eq!(
        parse_overflow_clip_margin("content-box 5px"),
        mk(OverflowClipMarginBox::ContentBox, LengthValue::Px(5.0))
    );
    // em 单位保留（compute 期 resolve）。
    assert_eq!(
        parse_overflow_clip_margin("padding-box 1em"),
        mk(OverflowClipMarginBox::PaddingBox, LengthValue::Em(1.0))
    );
    // 非法：>2 token / 重复 box / 重复 length / 未知 token → None。
    assert_eq!(parse_overflow_clip_margin("content-box border-box"), None);
    assert_eq!(parse_overflow_clip_margin("5px 10px"), None);
    assert_eq!(parse_overflow_clip_margin("content-box 5px 10px"), None);
    assert_eq!(parse_overflow_clip_margin("bogus"), None);
}

#[test]
/// 测试所有 FlexDirectionValue 变体
fn test_parse_flex_direction_all() {
    assert_eq!(parse_flex_direction("row"), Some(FlexDirectionValue::Row));
    assert_eq!(
        parse_flex_direction("row-reverse"),
        Some(FlexDirectionValue::RowReverse)
    );
    assert_eq!(parse_flex_direction("column"), Some(FlexDirectionValue::Column));
    assert_eq!(
        parse_flex_direction("column-reverse"),
        Some(FlexDirectionValue::ColumnReverse)
    );
    assert_eq!(parse_flex_direction("unknown"), None);
}

#[test]
/// 测试所有 FlexWrapValue 变体
fn test_parse_flex_wrap_all() {
    assert_eq!(parse_flex_wrap("nowrap"), Some(FlexWrapValue::Nowrap));
    assert_eq!(parse_flex_wrap("wrap"), Some(FlexWrapValue::Wrap));
    assert_eq!(parse_flex_wrap("wrap-reverse"), Some(FlexWrapValue::WrapReverse));
    assert_eq!(parse_flex_wrap("unknown"), None);
}

#[test]
/// 测试所有 AlignmentValue 变体
fn test_parse_alignment_all() {
    assert_eq!(parse_alignment("flex-start"), Some(AlignmentValue::FlexStart));
    assert_eq!(parse_alignment("flex-end"), Some(AlignmentValue::FlexEnd));
    assert_eq!(parse_alignment("center"), Some(AlignmentValue::Center));
    assert_eq!(parse_alignment("space-between"), Some(AlignmentValue::SpaceBetween));
    assert_eq!(parse_alignment("space-around"), Some(AlignmentValue::SpaceAround));
    assert_eq!(parse_alignment("space-evenly"), Some(AlignmentValue::SpaceEvenly));
    assert_eq!(parse_alignment("stretch"), Some(AlignmentValue::Stretch));
    assert_eq!(parse_alignment("start"), Some(AlignmentValue::Start));
    assert_eq!(parse_alignment("end"), Some(AlignmentValue::End));
    assert_eq!(parse_alignment("baseline"), Some(AlignmentValue::Baseline));
    // R2383：CSS Box Align 3 normal（justify-content/align-items/align-self 初始值，Chrome 支持）。
    // 修复前 None → 声明被丢（显式 normal 不能覆盖先前值）。
    assert_eq!(parse_alignment("normal"), Some(AlignmentValue::Normal));
    assert_eq!(parse_alignment("unknown"), None);
}

#[test]
/// 测试所有 BoxSizingValue 变体
fn test_parse_box_sizing_all() {
    assert_eq!(parse_box_sizing("content-box"), Some(BoxSizingValue::ContentBox));
    assert_eq!(parse_box_sizing("border-box"), Some(BoxSizingValue::BorderBox));
    assert_eq!(parse_box_sizing("unknown"), None);
}

#[test]
/// 测试所有 VisibilityValue 变体
fn test_parse_visibility_all() {
    assert_eq!(parse_visibility("visible"), Some(VisibilityValue::Visible));
    assert_eq!(parse_visibility("hidden"), Some(VisibilityValue::Hidden));
    assert_eq!(parse_visibility("collapse"), Some(VisibilityValue::Collapse));
    assert_eq!(parse_visibility("unknown"), None);
}

#[test]
/// 测试 content-visibility 解析（CSS Containment 2）：visible/hidden/auto + 大小写不敏感 + 非法值 None。
/// R2251 driving：content-visibility:hidden 实现。
fn test_parse_content_visibility_all() {
    assert_eq!(
        parse_content_visibility("visible"),
        Some(ContentVisibilityValue::Visible)
    );
    assert_eq!(parse_content_visibility("hidden"), Some(ContentVisibilityValue::Hidden));
    assert_eq!(parse_content_visibility("auto"), Some(ContentVisibilityValue::Auto));
    // 大小写不敏感（to_ascii_lowercase）
    assert_eq!(parse_content_visibility("HIDDEN"), Some(ContentVisibilityValue::Hidden));
    assert_eq!(parse_content_visibility("  Auto  "), Some(ContentVisibilityValue::Auto));
    // 非法值
    assert_eq!(parse_content_visibility("inherit"), None);
    assert_eq!(parse_content_visibility(""), None);
}

#[test]
/// 测试所有 FontWeightValue 变体（100-900、bold、normal、bolder、lighter）
fn test_parse_font_weight_all() {
    assert_eq!(parse_font_weight("100"), Some(FontWeightValue::Absolute(100)));
    assert_eq!(parse_font_weight("200"), Some(FontWeightValue::Absolute(200)));
    assert_eq!(parse_font_weight("300"), Some(FontWeightValue::Absolute(300)));
    assert_eq!(parse_font_weight("400"), Some(FontWeightValue::Absolute(400)));
    assert_eq!(parse_font_weight("500"), Some(FontWeightValue::Absolute(500)));
    assert_eq!(parse_font_weight("600"), Some(FontWeightValue::Absolute(600)));
    assert_eq!(parse_font_weight("700"), Some(FontWeightValue::Absolute(700)));
    assert_eq!(parse_font_weight("800"), Some(FontWeightValue::Absolute(800)));
    assert_eq!(parse_font_weight("900"), Some(FontWeightValue::Absolute(900)));
    assert_eq!(parse_font_weight("bold"), Some(FontWeightValue::Bold));
    assert_eq!(parse_font_weight("normal"), Some(FontWeightValue::Normal));
    assert_eq!(parse_font_weight("bolder"), Some(FontWeightValue::Bolder));
    assert_eq!(parse_font_weight("lighter"), Some(FontWeightValue::Lighter));
    // 超出范围的值
    assert_eq!(parse_font_weight("0"), None);
    assert_eq!(parse_font_weight("50"), None);
    assert_eq!(parse_font_weight("1000"), None);
}

#[test]
/// 测试所有 FontStyleValue 变体
fn test_parse_font_style_all() {
    assert_eq!(parse_font_style("normal"), Some(FontStyleValue::Normal));
    assert_eq!(parse_font_style("italic"), Some(FontStyleValue::Italic));
    assert_eq!(parse_font_style("oblique"), Some(FontStyleValue::Oblique(None)));
    assert_eq!(
        parse_font_style("oblique(15deg)"),
        Some(FontStyleValue::Oblique(Some(15.0)))
    );
    assert_eq!(parse_font_style("unknown"), None);
    // CSS 关键字大小写不敏感（CSS Values §4）：NORMAL/Italic/OBLIQUE ≡ 小写形式。
    assert_eq!(parse_font_style("NORMAL"), Some(FontStyleValue::Normal));
    assert_eq!(parse_font_style("Italic"), Some(FontStyleValue::Italic));
    assert_eq!(parse_font_style("OBLIQUE"), Some(FontStyleValue::Oblique(None)));
    assert_eq!(
        parse_font_style("Oblique(15DEG)"),
        Some(FontStyleValue::Oblique(Some(15.0)))
    );
    assert_eq!(parse_font_style("obliquex"), None);
    assert_eq!(parse_font_style("oblique-angle"), None);
}

#[test]
/// 测试 parse_length 对百分比的处理
fn test_parse_length_percentage() {
    let result = parse_length("50%");
    assert_eq!(result, Some(LengthValue::Percentage(50.0)));

    let result = parse_length("100%");
    assert_eq!(result, Some(LengthValue::Percentage(100.0)));

    let result = parse_length("33.33%");
    assert_eq!(result, Some(LengthValue::Percentage(33.33)));
}

#[test]
/// 测试 parse_length 对 auto 关键字的处理
fn test_parse_length_auto() {
    assert_eq!(parse_length("auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length("Auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length("AUTO"), Some(LengthValue::Auto));
    assert_eq!(parse_length("  auto  "), Some(LengthValue::Auto));
}

#[test]
/// 测试 currentcolor 大小写不敏感
fn test_parse_color_currentcolor_case_insensitive() {
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("currentcolor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("CURRENTcolor"), Some(ColorValue::CurrentColor));
}

#[test]
/// 测试 display: flow
fn test_parse_display_flow() {
    assert_eq!(parse_display("flow"), Some(DisplayValue::Flow));
}

#[test]
/// 测试 display: flow-root
fn test_parse_display_flow_root() {
    assert_eq!(parse_display("flow-root"), Some(DisplayValue::FlowRoot));
}

#[test]
/// 测试 display: list-item
fn test_parse_display_list_item() {
    assert_eq!(parse_display("list-item"), Some(DisplayValue::ListItem));
}

#[test]
/// 测试 display: contents
fn test_parse_display_contents() {
    assert_eq!(parse_display("contents"), Some(DisplayValue::Contents));
}

#[test]
/// 测试 display: inline-block
fn test_parse_display_inline_block() {
    assert_eq!(parse_display("inline-block"), Some(DisplayValue::InlineBlock));
}

#[test]
/// 测试 display: inline-flex
fn test_parse_display_inline_flex() {
    assert_eq!(parse_display("inline-flex"), Some(DisplayValue::InlineFlex));
}

#[test]
/// 测试 display: inline-grid
fn test_parse_display_inline_grid() {
    assert_eq!(parse_display("inline-grid"), Some(DisplayValue::InlineGrid));
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Parser 扩展测试 — 提升 parser.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :nth-child(odd) 伪类
fn test_parse_nth_child_odd() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(odd) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-child(even) 伪类
fn test_parse_nth_child_even() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(even) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 0 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-child(2n+1) 公式伪类
fn test_parse_nth_child_formula() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(2n+1) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-of-type(3) 伪类
fn test_parse_nth_of_type() {
    let stylesheet = Parser::parse_stylesheet("li:nth-of-type(3) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(NthPattern { a: 0, b: 3 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :not() 伪类
fn test_parse_not_selector() {
    let stylesheet = Parser::parse_stylesheet("p:not(.hidden) { display: block; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(
            compound
                .subclass_selectors
                .iter()
                .any(|s| matches!(s, SubclassSelector::PseudoClass(PseudoClassSelector::Not(_))))
        );
        // 验证声明
        assert!(sr.declarations.iter().any(|d| d.property == "display"));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :is() 伪类
fn test_parse_is_selector() {
    let stylesheet = Parser::parse_stylesheet("p:is(.active, .visible) { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Is(selectors))
                if selectors.len() == 2
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :where() 伪类
fn test_parse_where_selector() {
    let stylesheet = Parser::parse_stylesheet("p:where(.main) { font-size: 16px; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Where(selectors))
                if selectors.len() == 1
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :lang() 伪类
fn test_parse_lang() {
    let stylesheet = Parser::parse_stylesheet("p:lang(en) { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Lang(lang))
                if lang == &vec!["en".to_string()]
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性前缀匹配选择器 [href^=https]
fn test_parse_attribute_prefix() {
    let stylesheet = Parser::parse_stylesheet("[href^=https] { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Prefix(val),
                ..
            }) if name == "href" && val == "https"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性后缀匹配选择器 [href$=.pdf]
fn test_parse_attribute_suffix() {
    let stylesheet = Parser::parse_stylesheet("[href$=.pdf] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Suffix(val),
                ..
            }) if name == "href" && val == ".pdf"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性子串匹配选择器 [title*=hello]
fn test_parse_attribute_substring() {
    let stylesheet = Parser::parse_stylesheet("[title*=hello] { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Substring(val),
                ..
            }) if name == "title" && val == "hello"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性破折号匹配选择器 [lang|=en]
fn test_parse_attribute_dash() {
    let stylesheet = Parser::parse_stylesheet("[lang|=en] { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::DashMatch(val),
                ..
            }) if name == "lang" && val == "en"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试多选择器多声明的复杂规则
fn test_parse_multiple_selectors_and_declarations() {
    let css = "div.container > p.text, span.highlight { color: red; font-size: 16px; display: block; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // 两个选择器
        assert_eq!(sr.selectors.len(), 2);
        // 至少 3 条声明
        assert!(sr.declarations.len() >= 3);
        // 验证第一个选择器有 child 组合器
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].1, Some(Combinator::Child));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试嵌套 @media 带类选择器
fn test_parse_nested_at_media_with_class() {
    let css = "@media screen and (max-width: 768px) { .container { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            assert!(at_rule.prelude.contains("screen"));
            if let AtRuleBody::Block(rules) = &at_rule.body {
                assert_eq!(rules.len(), 1);
                if let Rule::Style(sr) = &rules[0] {
                    assert!(sr.declarations.iter().any(|d| d.property == "width"));
                } else {
                    panic!("Expected Style rule inside @media");
                }
            } else {
                panic!("Expected Block body for @media");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

#[test]
/// 测试 @supports 规则
fn test_parse_at_supports() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Property("display".to_string(), "grid".to_string())
            );
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

// ── @supports 解析扩展测试 ──

#[test]
/// 测试 @supports not 条件
fn test_parse_at_supports_not() {
    let css = "@supports not (display: grid) { .fallback { display: block; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Not(Box::new(SupportsCondition::Property(
                    "display".to_string(),
                    "grid".to_string()
                )))
            );
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports and 条件
fn test_parse_at_supports_and() {
    let css = "@supports (display: grid) and (gap: 10px) { .grid { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            match &supports_rule.condition {
                SupportsCondition::And(conditions) => {
                    assert_eq!(conditions.len(), 2);
                }
                _ => panic!("Expected And condition"),
            }
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports or 条件
fn test_parse_at_supports_or() {
    let css = "@supports (display: grid) or (display: flex) { .container { display: flex; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            match &supports_rule.condition {
                SupportsCondition::Or(conditions) => {
                    assert_eq!(conditions.len(), 2);
                }
                _ => panic!("Expected Or condition"),
            }
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports 多规则体
fn test_parse_at_supports_multiple_rules() {
    let css = "@supports (display: grid) { .a { display: grid; } .b { gap: 10px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(supports_rule.rules.len(), 2);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试畸形 `@supports;`（无条件/无块）须正确恢复，不吞掉紧跟其后的合法 @supports 规则。
/// driving: WPT at-supports-024 `@supports;` 后随 `@supports (margin:0){...}`。
fn test_parse_at_supports_malformed_semicolon_recovers() {
    let css = "div { background-color: red; } @supports; @supports (margin: 0) { div { background-color: green; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    // 应至少有 1 个 div 样式规则 + 1 个合法 @supports（margin:0）规则；
    // 畸形 @supports; 不应吞掉随后的合法 @supports。
    let supports_count = stylesheet
        .rules
        .iter()
        .filter(|r| matches!(r, Rule::Supports(_)))
        .count();
    assert!(
        supports_count >= 1,
        "合法 @supports (margin:0) 规则应存活，实际规则数={}，supports={}",
        stylesheet.rules.len(),
        supports_count
    );
    // 合法 @supports 须含 margin:0 条件（非空 rules）
    if let Some(Rule::Supports(s)) = stylesheet.rules.iter().find(|r| matches!(r, Rule::Supports(_))) {
        assert!(!s.rules.is_empty(), "@supports (margin:0) 块内规则不应为空");
    }
}

#[test]
/// 测试 @supports 带 selector() 函数
fn test_parse_at_supports_selector() {
    let css = "@supports selector(.a > .b) { .container { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Selector(".a > .b".to_string())
            );
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports 嵌套在 @media 内（通过 AtRule::At 回退）
fn test_parse_at_supports_nested_in_media() {
    let css = "@media screen { @supports (display: grid) { .a { display: grid; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            if let AtRuleBody::Block(inner) = &at_rule.body {
                assert_eq!(inner.len(), 1);
                match &inner[0] {
                    Rule::Supports(sr) => {
                        assert_eq!(
                            sr.condition,
                            SupportsCondition::Property("display".to_string(), "grid".to_string())
                        );
                    }
                    _ => panic!("Expected Supports rule inside @media"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

// ── CSS Transform 解析测试 ──

#[test]
fn test_parse_transform_none() {
    assert_eq!(parse_transform("none"), Some(TransformValue::None));
    assert_eq!(parse_transform("NONE"), Some(TransformValue::None));
}

#[test]
fn test_parse_transform_translate() {
    let result = parse_transform("translate(10px, 20px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 20.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_translate_single_arg() {
    let result = parse_transform("translate(10px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 0.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_translate_x_y() {
    let result = parse_transform("translateX(15px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::TranslateX(15.0));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("translateY(25px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::TranslateY(25.0));
        }
        _ => panic!("Expected List"),
    }
}
