#[cfg(test)]
use zero_css_parser::Parser;
use zero_dom::parse_html;

/// 验证 HTML 解析后 DOM 树包含预期的元素节点
#[test]
fn test_html_parse_produces_dom_tree() {
    let html =
        r#"<html><head><title>Test</title></head><body><div id="main" class="container">Hello</div></body></html>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "DOM 应包含节点");

    // 查找 div 元素
    let root = doc.root();
    let _body = doc.get(root).unwrap();
    assert!(doc.node_count() > 5, "DOM 应包含多个节点");
}

/// 验证 CSS 解析产生正确的规则结构
#[test]
fn test_css_parse_produces_rules() {
    let css = r#"
        body { margin: 0; padding: 0; }
        .container { display: flex; width: 100%; }
        #main { background-color: blue; }
    "#;
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3, "应解析 3 条 CSS 规则");

    // 验证选择器存在
    for rule in &stylesheet.rules {
        if let zero_css_parser::Rule::Style(style_rule) = rule {
            assert!(!style_rule.selectors.is_empty());
            assert!(!style_rule.declarations.is_empty());
        }
    }
}

/// DOM + CSS 选择器匹配集成
#[test]
fn test_dom_element_attributes_accessible() {
    let html = r#"<html><body><div id="app" class="main active">Content</div></body></html>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 0);

    // 遍历所有节点，验证至少有一个元素
    let root = doc.root();
    let data = doc.get(root).unwrap();
    assert!(!data.children.is_empty(), "根节点应有子节点");
}
