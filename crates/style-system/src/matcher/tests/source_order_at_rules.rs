//! R4024（CSS2 §6.4.1 源码顺序 + CSS Cascading 5 §6.4）：声明收集须按真实源码顺序。
//!
//! 旧实现先收集全部顶层 Style 规则、再单独遍历 At 规则——@media/@supports 块内规则
//! 的级联位置被推到所有顶层规则之后。同特异性下「后出现者胜」被违反：
//! `@media all { .t { red } }` 在前、`.t { green }` 在后，后者应胜出。

use crate::matcher::{build_stylesheet_index, collect_matching_declarations_with_media};
use zero_css_parser::Parser;
use zero_css_parser::media_query::MediaContext;
use zero_dom::parse_html;

fn background_of(decls: &[(String, String, bool, (u32, u32, u32), Option<usize>)]) -> String {
    // 模拟级联 tie-break：同特异性按收集顺序后者胜（列表序 = 源码序）
    decls
        .iter()
        .rev()
        .find(|(p, _, _, _, _)| p == "background")
        .map(|(_, v, _, _, _)| v.clone())
        .unwrap_or_default()
}

fn eval(css: &str, html: &str, target: &str) -> String {
    let sheet = Parser::parse_stylesheet(css);
    let doc = parse_html(html);
    let divs = doc.get_elements_by_tag_name(target);
    let el = *divs.last().expect("target element");
    let index = build_stylesheet_index(std::slice::from_ref(&sheet));
    let media_ctx = MediaContext::new(800.0, 600.0);
    let decls = collect_matching_declarations_with_media(
        &doc,
        el,
        std::slice::from_ref(&sheet),
        &index,
        Some(&media_ctx),
        None,
    );
    background_of(&decls)
}

/// 顶层规则须覆盖源码位置更早的 @media 内同特异性规则。
#[test]
fn r4024_top_level_rule_overrides_earlier_media_block() {
    let css = "@media all { .t { background: red; } } .t { background: green; }";
    let bg = eval(css, r#"<div class="t"></div>"#, "div");
    assert_eq!(bg, "green", "R4024: 源码靠后的顶层规则应胜出，实际 {bg}");
}

/// 反向：@media 块源码靠后时，块内规则同样靠后胜出。
#[test]
fn r4024_later_media_block_overrides_top_level_rule() {
    let css = ".t { background: red; } @media all { .t { background: green; } }";
    let bg = eval(css, r#"<div class="t"></div>"#, "div");
    assert_eq!(bg, "green", "R4024: 源码靠后的 @media 内规则应胜出，实际 {bg}");
}

/// @supports 同语义：块内规则参与全局源码序，不再整体后置。
#[test]
fn r4024_supports_block_participates_in_source_order() {
    let css = "@supports (display: block) { .t { background: red; } } .t { background: green; }";
    let bg = eval(css, r#"<div class="t"></div>"#, "div");
    assert_eq!(bg, "green", "R4024: @supports 块内规则不得后置，实际 {bg}");
}
