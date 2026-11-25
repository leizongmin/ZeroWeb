//! k121（kc5r121）：dashed-function 产物元素级 CSS 宽关键字落地钉测。
//!
//! WPT 判据「left unresolved on result descriptor」族（dashed-function-eval）：
//! 求值端对 result 产物关键字字面透传，消费元素按 custom property 语义落地。
//! 四臂：initial/revert＝guaranteed-invalid（表移除兑现 var 回退）；
//! inherit/unset＝父计算值代入；revert-layer/revert-rule 需级联回溯架构持
//! 字面。整值才认关键字；大小写不敏感；落表保原文零漂移。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

#[test]
/// result: initial → 产物整值关键字 → 表移除（guaranteed-invalid）。
fn keyword_initial_product_becomes_absent() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: initial; } #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert!(cp.get("--tmp").is_none(), "--tmp 应缺席（guaranteed-invalid）");
}

#[test]
/// result: revert → 回退 author origin 声明后按继承性质取父计算值
///（探针一代定谳：custom property 无 author 声明时值＝inherited 非
/// guaranteed-invalid）。
fn keyword_revert_product_takes_parent_value() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let css = "@function --f() { result: revert; } #main { --tmp: PASS; } p.text { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// result: revert → 缺席父值（无继承源）→ guaranteed-invalid（absent）。
fn keyword_revert_without_parent_is_absent() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: revert; } #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert!(cp.get("--tmp").is_none(), "--tmp 应缺席（guaranteed-invalid）");
}

#[test]
/// result: inherit → 父元素计算值代入（custom property 继承传递）。
fn keyword_inherit_product_takes_parent_value() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = "@function --f() { result: inherit; } #main { --tmp: PASS; } p.text { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
    let _ = div;
}

#[test]
/// result: unset → custom property 继承传递 → 同 inherit（父值代入）。
fn keyword_unset_product_takes_parent_value() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let css = "@function --f() { result: unset; } #main { --tmp: PASS; } p.text { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// inherit 臂父值缺席 → 同 absent（inherit guaranteed-invalid 链）。
fn keyword_inherit_with_absent_parent_is_absent() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let css = "@function --f() { result: inherit; } p.text { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert!(cp.get("--tmp").is_none(), "--tmp 应缺席（父值 absent 链）");
}

#[test]
/// 关键字 ASCII 大小写不敏感（css-syntax-3 关键字规则）。
fn keyword_match_is_ascii_case_insensitive() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let css = "@function --f() { result: INHERIT; } #main { --tmp: PASS; } p.text { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// revert-layer/revert-rule 需级联层/规则次优声明回溯架构——保持字面透传
///（held，不误伤为 absent）。
fn keyword_revert_layer_rule_stay_literal() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = "@function --f() { result: revert-layer; } #main { --tmp: --f(); } \
               @function --g() { result: revert-rule; } p.text { --x: --g(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("revert-layer"));
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--x").map(String::as_str), Some("revert-rule"));
}

#[test]
/// 部分值关键字不认：产物含前后缀时整值非关键字 → 保原文。
fn partial_value_keyword_stays_literal() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: initial; } #main { --tmp: pre --f() post; }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("pre initial post"));
}

#[test]
/// 非产物路径不触：作者直书 `--x: initial`（无函数替换）保持字面——
/// 本刀只作用于替换产物（套件域窄刀，页外零扩散）。
fn authored_keyword_without_substitution_stays_literal() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "#main { --tmp: initial; }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("initial"));
}
