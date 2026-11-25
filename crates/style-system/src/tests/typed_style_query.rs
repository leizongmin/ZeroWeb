//! k122（kc5r122）：style() 冒号等值类型感知比较钉测。
//!
//! WPT 判据 function-parameter-types 族（css-mixins-1 §2.1 typed 参数＝
//! 局部注册语义）：型别表（@function 参数∪@property 注册语法）下 style()
//! 冒号等值双侧解析为可比值再比较（"red"≡"#f00"）；type(*)/未知型/解析
//! 失败回退裸文本 text_eq（观测恒同，构造性零 P2F）。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

#[test]
/// 「A parameter retains its type」：typed 参数查询 #f00 ≡ red → PASS。
fn typed_param_retains_type() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c <color>) { result: if(style(--c:red): PASS; else: FAIL); } \
               #main { --tmp: --f(#f00); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「A parameter type acts as a local registration」：局部声明继承参数
/// 型别语义，#00f ≡ blue → PASS。
fn typed_param_local_registration() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c <color>) { --c: #00f; result: if(style(--c:blue): PASS; else: FAIL); } \
               #main { --tmp: --f(#000); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「A parameter retains its type (parent stack frame)」：父栈帧参数
/// --c 经调用方作用域穿线可见，#f00 ≡ red → PASS。
fn typed_param_parent_stack_frame() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: if(style(--c:red): PASS; else: FAIL); } \
               @function --g(--c <color>) { result: --f(); } \
               #main { --tmp: --g(#f00); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「A parameter type acts as a local registration (parent stack frame)」：
/// 父栈帧局部注册 #00f ≡ blue → PASS。
fn typed_local_registration_parent_stack_frame() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: if(style(--c:blue): PASS; else: FAIL); } \
               @function --g(--c <color>) { --c: #00f; result: --f(); } \
               #main { --tmp: --g(#f00); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「if() within @function can query registered custom property」：
/// @property 注册语法回落型别表，red ≡ #f00 → PASS。
fn typed_query_registered_custom_property() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@property --c { syntax: '<color>'; inherits: false; initial-value: black; } \
               @function --f() { result: if(style(--c:#f00): PASS; else: FAIL); } \
               #main { --c: red; --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// type(*) 万能型豁免：型别规约不识 → 裸文本比较不变（green ≠ #0f0 文本
/// → else 支路 FAIL——「Universally typed」族现行为零变）。
fn universal_type_star_stays_text_eq() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c type(*)) { result: if(style(--c:green): PASS; else: FAIL); } \
               #main { --tmp: --f(#0f0); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("FAIL"));
}

#[test]
/// untyped 参数不变：#f00 vs red 文本不等 → FAIL（k112 裸文本通道零变）。
fn untyped_param_text_compare_unchanged() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c) { result: if(style(--c:red): PASS; else: FAIL); } \
               #main { --tmp: --f(#f00); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("FAIL"));
}

#[test]
/// untyped 参数等值文本照旧 PASS（text_eq 正常面不受刀扰）。
fn untyped_param_equal_text_still_passes() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c) { result: if(style(--c:red): PASS; else: FAIL); } \
               #main { --tmp: --f(red); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 右侧解析失败回退：typed <color> 查询右操作数不可解析 → None 回退
/// 裸文本 text_eq（"#f00"≠"oops" → FAIL——观测与刀前恒同）。
fn typed_right_parse_failure_falls_back() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--c <color>) { result: if(style(--c:oops): PASS; else: FAIL); } \
               #main { --tmp: --f(#f00); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("FAIL"));
}

#[test]
/// 左侧解析失败回退：@property 注册型别＋元素基值不可解析 → None 回退
/// 裸文本 text_eq（"oops"=="oops" → PASS——若误硬判 False 此钉红）。
fn typed_left_parse_failure_falls_back() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@property --c { syntax: '<color>'; inherits: false; initial-value: black; } \
               @function --f() { result: if(style(--c:oops): PASS; else: FAIL); } \
               #main { --c: oops; --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// `<length>` px canonical 翻转：1em（γ1 基准 16px）≡ 16px → PASS
///（裸文本刀前 FAIL——数值域解析等值正向证词）。
fn typed_length_canonical_px_compare() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--l <length>) { result: if(style(--l:16px): PASS; else: FAIL); } \
               #main { --tmp: --f(1em); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// k123a 帧归属锚：type(*) 参数跨帧精确归属——嵌套形
/// --g(--c <color>)→--f(--c type(*)) 帧表 --f 帧内 type(*) 胜（万能型
/// 豁免→text_eq：lime≠#0f0 文本不等）。k122 全局 params 联合迭代序不定
/// （"lime"≡"#0f0" 同色解析碰撞——Universally typed 键 P2F）修正锚；
/// 20 实例循环断言（每实例独立 HashMap 种子——确定性证词）。
fn universal_type_star_shadow_frame_deterministic() {
    let css = "@function --f(--c type(*)) { result: if(style(--c:red): FAIL1; style(--c:#f00): FAIL2; \
               style(--c:lime): FAIL3; style(--c:#0f0): PASS; else: FAIL4); } \
               @function --g(--c <color>) { result: --f(#0f0); } \
               #main { --tmp: --g(#f00); }";
    for _ in 0..20 {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
        let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
        assert_eq!(
            cp.get("--tmp").map(String::as_str),
            Some("PASS"),
            "帧表归属不定＝k123a 修正回归"
        );
    }
}
