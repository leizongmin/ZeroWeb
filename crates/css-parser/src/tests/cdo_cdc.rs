//! CDO（`<!--`）/ CDC（`-->`）token 化与 stylesheet 顶层忽略（CSS Syntax §4.1.1）。
//!
//! 背景：legacy HTML 3.2/4 静态页常用 `<style><!-- ... --></style>` 包裹样式块。
//! 修复前 tokenizer 不识别 CDO/CDC，`<!--` 落 `Error('<')`，顶层 `consume_rule` 把它当
//! 选择器解析失败后 `skip_malformed_qualified_rule` 一路消耗到 `{...}` 块，**吞掉紧跟其
//! 后的真实规则**。修复：tokenizer 把 `<!--`/`-->` 识别为 `Token::Comment`，复用既有
//! `skip_whitespace` 跳过 Comment 的 ignorable 通道，顶层被忽略（与 chromium 一致）。

use super::*;

/// 单条规则被 `<!-- ... -->` 包裹时不被吞，正常解析。
#[test]
fn test_cdo_cdc_wrap_does_not_swallow_rule() {
    let css = "<!--\nbody { color: red; }\n-->";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(
        stylesheet.rules.len(),
        1,
        "CDO/CDC 包裹不应吞掉规则，应解析出 1 条 Style 规则"
    );
    if let Rule::Style(style) = &stylesheet.rules[0] {
        assert_eq!(style.declarations.len(), 1);
        assert_eq!(style.declarations[0].property, "color");
    } else {
        panic!("应为 Style 规则");
    }
}

/// 多条规则被包裹时全部保留。
#[test]
fn test_cdo_cdc_wrap_multiple_rules() {
    let css = "<!--\np { color: red; }\ndiv { color: blue; }\n-->";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2, "两条规则都应保留");
}

/// 只有起始 CDO、无结尾 CDC（legacy 常见），后续规则仍正常解析。
#[test]
fn test_cdo_only_without_closing_cdc() {
    let css = "<!--\nbody { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "无结尾 CDC 时规则也应解析");
}

/// 顶层裸 CDC（无前置 CDO）被忽略，不产生幻影规则。
#[test]
fn test_top_level_cdc_alone_ignored() {
    let css = "body { color: red; }\n-->";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "尾部裸 CDC 应被忽略，仅 1 条规则");
}

/// 真实 `<style>` 块内容（多声明 + 多规则 + 换行）整体保留。
#[test]
fn test_cdo_cdc_realistic_style_block() {
    let css = "<!--\nbody { background: #fff; color: #000; }\nh1 { font-size: 200%; }\n-->";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2, "真实 style 块两条规则都应保留");
    // 第一条 body 规则有两条声明
    if let Rule::Style(style) = &stylesheet.rules[0] {
        assert_eq!(style.declarations.len(), 2);
    }
}

/// `<` 非紧跟 `!--` 时退回 Delim('<')（不误吞 CDO）。
#[test]
fn test_lt_not_cdo_falls_back_to_delim() {
    // `<` 单独出现于值上下文：声明值解析应不崩溃，规则仍解析为 1 条。
    let css = "x { content: \"<\"; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}
