//! 探针：baidu 热榜 clearfix 同款形状——多重 class 复合 + `::after`，
//! `content:"";display:block;clear:both` 应产出 after_pseudo（清除浮动盒化前提）。

use super::super::*;

fn find_id(doc: &Document, root: NodeId, id: &str) -> Option<NodeId> {
    if doc.get_attribute(root, "id").is_some_and(|v| v == id) {
        return Some(root);
    }
    for child in doc.child_nodes(root) {
        if let Some(found) = find_id(doc, child, id) {
            return Some(found);
        }
    }
    None
}

/// baidu 原型：`.s-hotsearch-wrapper.s-hotsearch-wrapper-new-hot.s-hotsearch-wrapper.s-hotsearch-wrapper-new-hot::after`
fn probe_baidu_compound_after() {
    let css = ".s-hotsearch-wrapper.s-hotsearch-wrapper-new-hot.s-hotsearch-wrapper.s-hotsearch-wrapper-new-hot::after{content:\"\";display:block;clear:both}";
    let doc = zero_dom::parse_html(
        "<html><body><div id=\"w\" class=\"s-isindex-wrap s-hotsearch-wrapper s-hotsearch-wrapper-no-login s-hotsearch-wrapper-new-hot\"><ul><li>a</li><li>b</li></ul></div></body></html>",
    );
    let w = find_id(&doc, doc.root(), "w").expect("div#w");
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let st = styles.get(&w).expect("div#w 应有计算样式");
    let after = st
        .after_pseudo
        .as_ref()
        .expect("baidu 同款 ::after clearfix 应产出 after_pseudo");
    assert!(
        matches!(&after.content, property::types::ContentComputedValue::String(s) if s.is_empty()),
        "content 须为空串，got {:?}",
        after.content
    );
    assert_eq!(after.display, DisplayValue::Block, "display 须为 block");
}

/// 简化对照：单 class + `::after` 同声明（隔离「多重复合」变量）。
fn probe_single_class_after() {
    let css = ".box::after{content:\"\";display:block;clear:both}";
    let doc = zero_dom::parse_html("<html><body><div id=\"w\" class=\"box\">x</div></body></html>");
    let w = find_id(&doc, doc.root(), "w").expect("div#w");
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let st = styles.get(&w).expect("div#w 应有计算样式");
    assert!(st.after_pseudo.is_some(), "单 class ::after 也应产出 after_pseudo");
}

#[test]
fn baidu_compound_class_after_clearfix() {
    probe_baidu_compound_after();
}

#[test]
fn single_class_after_clearfix() {
    probe_single_class_after();
}
