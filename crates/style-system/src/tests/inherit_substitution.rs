//! k123（kc5r123）：会话内 inherit() 记法父栈帧替换钉测。
//!
//! WPT 判据 local-inherit-substitution 族（css-values-5 §inherit-notation
//! × css-mixins-1 栈帧语义）：inherit(--name[, fallback]) 取**父栈帧**
//! （调用方已解析作用域）值——局部声明不遮蔽父帧读取；父帧 guaranteed-
//! invalid（var 环/缺席→遮蔽标记）走 fallback 臂；typed 参数绑定经类型
//! 强制 canonical 化（42.0px≡42px）。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

#[test]
/// 「inherit() refers to parent stack frame (element)」：元素父帧值
/// PASS 胜过本帧局部 FAIL2。
fn inherit_element_parent_frame() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { --x: FAIL2; result: inherit(--x); } \
               #main { --x: PASS; --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「inherit() refers to parent stack frame (other function call)」：
/// 嵌套调用父帧＝调用方栈帧局部值（--f 的 --x: PASS 胜过 --g 本帧 FAIL3
/// 与元素 FAIL2）。
fn inherit_caller_function_frame() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { --x: PASS; result: --g(); } \
               @function --g() { --x: FAIL3; result: inherit(--x); } \
               #main { --x: FAIL2; --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「inherit() referring to guaranteed-invalid in parent frame」：父帧
/// var 缺席 guaranteed-invalid（遮蔽标记）→ fallback 臂 PASS。
fn inherit_guaranteed_invalid_parent_takes_fallback() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { --x: var(--noexist); result: --g(); } \
               @function --g() { --x: FAIL3; result: inherit(--x, PASS); } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「inherit() referring to cycle in parent frame」：父帧 var 自环
/// guaranteed-invalid → fallback 臂 PASS。
fn inherit_cycle_parent_takes_fallback() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { --x: var(--x); result: --g(); } \
               @function --g() { --x: FAIL3; result: inherit(--x, PASS); } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「inherit() referring to typed value in parent frame」：typed 参数
/// 绑定 canonical 化（42.0px→42px）经父帧读取透传。
fn inherit_typed_param_parent_frame() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--x <length>) { result: --g(); } \
               @function --g() { --x: 14px; result: inherit(--x); } \
               #main { --tmp: --f(42.0px); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("42px"));
}

#[test]
/// 父帧缺席且无 fallback → guaranteed-invalid 链：result 站 Err → 调用
/// invalid（消费声明 IACVT——--tmp 表移除）。
fn inherit_absent_without_fallback_is_invalid() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: inherit(--noexist); } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert!(cp.get("--tmp").is_none(), "--tmp 应缺席（调用 invalid）");
}
