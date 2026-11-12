//! sibling-index()/sibling-count() 元素兄弟序位置 + 树计数页端到端（CSS Values 5
//! §tree-counting，g19）。
//!
//! [`crate::sibling_position`]：父元素子**元素**列表 1 基序数 + 总数；根元素
//! (1, 1)（document 父非元素路径自然成立）；非元素子（文本/注释）不计入。
//! 端到端：`z-index: calc(sibling-index() * 2)` 树计数页断言形态（同键兄弟
//! style_cache 禁用后各元素独立求值）。
//!
//! driving: wptfyi-css DC-2 g19（tree-counting calc-sibling-function）。

use crate::sibling_position;
use zero_dom::parse_html;

#[test]
fn sibling_position_ordinal_and_count() {
    let html = r#"<html><body><div><span></span>text<!--c--><p id="a"></p><span></span><p id="b"></p><ul></ul></div></body></html>"#;
    let doc = parse_html(html);
    let a = doc.query_selector(doc.root(), "#a").expect("#a");
    let b = doc.query_selector(doc.root(), "#b").expect("#b");
    // 元素子：span, p#a, span, p#b, ul——文本/注释不计入（§tree-counting 元素子序）。
    assert_eq!(sibling_position(&doc, a), (2, 5));
    assert_eq!(sibling_position(&doc, b), (4, 5));
}

#[test]
fn sibling_position_root_is_one_one() {
    let html = r#"<html><head></head><body></body></html>"#;
    let doc = parse_html(html);
    let html_el = doc.query_selector(doc.root(), "html").expect("html");
    // 根元素：父为 document（非元素）→ (1, 1)。
    assert_eq!(sibling_position(&doc, html_el), (1, 1));
}

#[test]
fn tree_counting_page_end_to_end_z_index() {
    // tree-counting 主页断言形态：#test 为父 div 第 2 个元素子（共 5 个），
    // `z-index: calc(sibling-index() * 2)` → 4、`order: calc(sibling-count() * 2)` → 10。
    let html = r#"<html><head><style>
        #test { z-index: calc(sibling-index() * 2); order: calc(sibling-count() * 2); }
    </style></head><body><div><div></div><div id="test"></div><div></div><div></div><ul></ul></div></body></html>"#;
    let css = r#"#test { z-index: calc(sibling-index() * 2); order: calc(sibling-count() * 2); }"#;
    let doc = parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let mut sys = crate::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let test = doc.query_selector(doc.root(), "#test").expect("#test");
    let s = styles.get(&test).expect("style");
    assert_eq!(s.z_index, crate::property::types::ZIndexValue::Integer(4));
    assert_eq!(s.order, 10);
}
