//! k124（kc5r124）：`@function` 体条件规则组钉测。
//!
//! WPT 判据 function-conditionals 族（css-mixins-1 §conditional-rules）：
//! 体级 @supports/@media/@container 条件组——条件满足时组内声明/`result`
//! 按源序原地展开（§4 last-wins 语义与平面声明一致），不满足整组跳过；
//! 嵌套组递归（外真内真才进）。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

#[test]
/// 「Basic @supports」：条件真组内 `result` 覆盖顶层（FAIL→PASS 翻转主证词
/// ——k124 前组被解析丢弃、顶层 FAIL 生效）。
fn conditional_supports_true_overrides_top_level() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = "@function --f() { result: FAIL; @supports (width: 100px) { result: PASS; } } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「Basic @supports (false)」：条件假组整体跳过，顶层 `result` 保持
///（k124 前组丢弃天然通过——守成钉，防条件判定误翻转）。
fn conditional_supports_false_group_skipped() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: PASS; @supports (not (width: 100px)) { result: FAIL; } } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「Inconsequential conditional」：组（FAIL）在前、顶层（PASS）在后——
/// 源序 last-wins 跨组，**后面的顶层胜过前面的条件组**。
fn conditional_source_order_last_wins() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { @supports (width: 100px) { result: FAIL; } result: PASS; } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「@supports with locals」：条件真组内局部覆盖顶层局部（`--y: 2`→`20`，
/// result `1 20 3`）。
fn conditional_locals_override_in_group() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f(--x) { --y: 2; --z: 3; @supports (width: 100px) { --y: 20; } \
               result: var(--x) var(--y) var(--z); } \
               #main { --tmp: --f(1); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("1 20 3"));
}

#[test]
/// 「Basic @media」：视口语境下真臂组生效（`set_viewport` 必需——无视口
/// 语境按 false，规范：条件不可判定 → 组不生效）。
fn conditional_media_true_with_viewport() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: FAIL; @media (width > 0px) { result: PASS; } } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「Basic @container」：容器链静态尺寸真臂组生效（100px 定宽
/// container-type:size 容器内 `(width = 100px)` 满足——链由
/// compute_styles_recursive 自根向下维护）。
fn conditional_container_true_with_size_container() {
    let (mut doc, _html, _body, div, p) = make_test_dom();
    doc.set_attribute(div, "style", "container-type:size;width:100px;height:50px;");
    let css = "@function --f() { result: FAIL; @container (width = 100px) { result: PASS; } } \
               #main p { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「Nested @media」：嵌套条件组递归——外层 `(width > 0px)` 真进内层，
/// 内层 `not (...)` 前缀否定（mediaqueries-5 括号包裹 media-condition）真臂
/// PASS 胜（后置假组 `(width: 99999999px)` 跳过；k124 首版 not 形解析失败
/// 整组被 skip＝held 根因）。
fn conditional_nested_media() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --f() { result: FAIL; \
               @media (width > 0px) { \
                 @media (not (height: 99999999px)) { result: PASS; } \
                 @media (width: 99999999px) { result: FAIL; } \
               } } \
               #main { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&div).expect("div 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}

#[test]
/// 「Nested @container」：嵌套容器组＋`not (...)` 尺寸否定——容器 100x50：
/// 外层 `(width = 100px)` 真，内层 `not (height = 75px)` 真（50≠75）→ PASS
/// 胜（后置假组 `(height: 75px)` 跳过）。
fn conditional_nested_container() {
    let (mut doc, _html, _body, div, p) = make_test_dom();
    doc.set_attribute(div, "style", "container-type:size;width:100px;height:50px;");
    let css = "@function --f() { result: FAIL; \
               @container (width = 100px) { \
                 @container (not (height = 75px)) { result: PASS; } \
                 @container (height: 75px) { result: FAIL; } \
               } } \
               #main p { --tmp: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let cp = &styles.get(&p).expect("p 应有计算样式").custom_properties;
    assert_eq!(cp.get("--tmp").map(String::as_str), Some("PASS"));
}
