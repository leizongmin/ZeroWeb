//! 自定义属性存储共享测试（P9-9 切片 2 / R5114 DF-1 返修）。
//!
//! 验证 `ComputedStyle::custom_properties`（`Arc<HashMap<..>>`）在无自身 `--` 声明的
//! 元素间共享父链 Arc（O(1) 克隆），仅在自身声明或注册属性导致值差时 fork——
//! 旧实现每元素全量深拷贝，github.com 3000+ 节点量级内存放大 90–180MB。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

#[test]
/// 无任何 `--` 声明：全文档共享 EMPTY 单例 Arc（旧实现每元素独立深拷贝）。
fn test_custom_properties_empty_shares_singleton() {
    let (doc, _html, _body, _div, _p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet("body { color: red; }")];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let empty = crate::property::ComputedStyle::empty_custom_properties();
    for node in [_html, _body, _div, _p] {
        let cp = &styles.get(&node).expect("应有计算样式").custom_properties;
        assert!(
            std::sync::Arc::ptr_eq(cp, &empty),
            "无 `--` 声明的元素必须共享 EMPTY 单例"
        );
    }
}

#[test]
/// 仅 :root（html）声明：后代全部共享 html 的 Arc，不逐元素拷贝。
fn test_custom_properties_descendants_share_root_arc() {
    let (doc, html, body, div, p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet("html { --theme-a: red; }")];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let html_cp = &styles.get(&html).expect("html 应有计算样式").custom_properties;
    let empty = crate::property::ComputedStyle::empty_custom_properties();
    assert!(
        !std::sync::Arc::ptr_eq(html_cp, &empty),
        "声明元素自身 fork（≠ EMPTY 单例）"
    );
    for node in [body, div, p] {
        let cp = &styles.get(&node).expect("应有计算样式").custom_properties;
        assert!(std::sync::Arc::ptr_eq(cp, html_cp), "后代共享根声明 Arc");
    }
}

#[test]
/// 中间元素自有声明：自身 fork，其下后代共享 fork 后的 Arc，兄弟链不受影响。
fn test_custom_properties_forks_at_own_declaration() {
    let (doc, html, body, div, p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(
        "html { --theme-a: red; } #main { --theme-b: blue; }",
    )];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let get = |n| &styles.get(&n).expect("应有计算样式").custom_properties;
    let (html_cp, body_cp, div_cp, p_cp) = (get(html), get(body), get(div), get(p));
    assert!(!std::sync::Arc::ptr_eq(div_cp, body_cp), "div 自有声明 → fork");
    assert!(std::sync::Arc::ptr_eq(p_cp, div_cp), "p 无声明 → 共享 div 的 Arc");
    assert!(std::sync::Arc::ptr_eq(body_cp, html_cp), "body 链不受 div fork 影响");
    // 值正确性：共享的 map 内容齐全（div/p 可见两个主题变量）。
    assert_eq!(div_cp.get("--theme-a").map(String::as_str), Some("red"));
    assert_eq!(div_cp.get("--theme-b").map(String::as_str), Some("blue"));
    assert_eq!(body_cp.get("--theme-b"), None, "fork 的兄弟链不泄漏后声明");
}
