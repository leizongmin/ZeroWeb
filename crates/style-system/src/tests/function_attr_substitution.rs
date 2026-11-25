//! k126（kc5r126）：函数求值端 attr() 替换＋attr-security taint 钉测。
//!
//! WPT 判据 function-attr 族（css-values-5 §attr-notations × §attr-security
//! × css-mixins-1 §3 替换序 var→attr）：函数体三域（参数缺省/局部声明/
//! result）attr() 文本级替换；产物携带 url → taint 随 DashOutcome 通道
//! 逐层合并上传，顶层消费站对取 URL 属性裁决 IACVT（继承回落）。

use super::super::*;
use super::helpers::{LengthValue, make_test_dom};
use zero_css_parser::Parser as CssParser;

#[test]
/// 「attr() in result」（untyped × `type(<length>)`）：result 直形替换——
/// `result: attr(data-42 type(<length>))` 产物 42px 落地 width（k125 前
/// 函数求值端 attr() 缺位产物原文 → apply 拒 → 槽默认）。
fn fnattr_result_untyped_length() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f() { result: attr(data-42 type(<length>)); } #actual { width: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").width, LengthValue::Px(42.0));
}

#[test]
/// 「attr() in result」（typed `returns <length>` × `type(<length>)`）：
/// typed 返回强制与 attr 型别一致——替换产物经 eval_typed_value 强制落地。
fn fnattr_result_typed_returns_length() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f() returns <length> { result: attr(data-42 type(<length>)); } \
               #actual { height: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").height, LengthValue::Px(42.0));
}

#[test]
/// 「attr() in result」（typed × `type(*)` 通配型别）：通配型别产物原文
///（42px）经 typed 返回强制落地。
fn fnattr_result_typed_returns_universal() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f() returns <length> { result: attr(data-42 type(*)); } \
               #actual { height: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").height, LengthValue::Px(42.0));
}

#[test]
/// 「attr() in default parameter value」：参数缺省域替换——实参缺席走
/// 缺省，attr() 产物经 var(--a) 引用落地。
fn fnattr_default_parameter() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f(--a: attr(data-42 type(*))) { result: var(--a); } \
               #actual { width: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").width, LengthValue::Px(42.0));
}

#[test]
/// 「attr() in local variable」：局部声明域替换——`--l: attr(...)` 经
/// var(--l) 引用落地。
fn fnattr_local_variable() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f() { --l: attr(data-42 type(*)); result: var(--l); } \
               #actual { width: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").width, LengthValue::Px(42.0));
}

#[test]
/// 「Returned url() is attr-tainted」：attr() 产物 url(cat.png) → taint
/// 上传 → 取 URL 属性（list-style-image）消费裁决 IACVT → 级联移除 →
/// 继承父计算值 url(parent)（css-values-5 §attr-security——与 4.75 站
/// 同口径；k125 观测＝产物原文残 attr 词残臂回落，k126 换真替换＋taint
/// 拦截，观测同向）。
fn fnattr_url_tainted_iacvt_inherit() {
    let (mut doc, _html, _body, div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-cat", "url(cat.png)");
    let css = "@function --f() { result: attr(data-cat type(*)); } \
               #main { list-style-image: url(parent); } \
               #actual { list-style-image: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(
        styles.get(&p).expect("p 应有计算样式").list_style_image,
        ListStyleImageComputedValue::Url("parent".into())
    );
}

#[test]
/// 嵌套调用实参形（result 文本级覆盖）：`--f()` result 内 `--g(attr(...))`
/// ——attr() 站文本级替换先于外层扫描循环产物重扫，实参具体化后嵌套求值。
fn fnattr_nested_call_argument() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --g(--a) { result: var(--a); } \
               @function --f() { result: --g(attr(data-42 type(*))); } \
               #actual { width: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").width, LengthValue::Px(42.0));
}

#[test]
/// 守成：实参直形（`--f(attr(data-42 type(*)))`）——顶层 4.75 attr 重裁
/// 站已在函数站之前替换实参（既有通路径），本刀不得翻转。
fn fnattr_direct_argument_baseline() {
    let (mut doc, _html, _body, _div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-42", "42px");
    let css = "@function --f(--a) { result: var(--a); } #actual { width: --f(attr(data-42 type(*))); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").width, LengthValue::Px(42.0));
}
