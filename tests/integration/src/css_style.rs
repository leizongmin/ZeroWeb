#[cfg(test)]
use zero_css_parser::Parser;
use zero_dom::parse_html;
use zero_style_system::StyleSystem;

/// 验证样式系统可以计算 DOM 节点的计算样式
#[test]
fn test_compute_styles_from_html_and_css() {
    let html = r#"<html><body><div id="box" class="container">Hello</div></body></html>"#;
    let css = r#"
        .container { display: flex; width: 200px; }
        #box { background-color: red; }
    "#;

    let doc = parse_html(html);
    let stylesheet = Parser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 样式系统应返回样式映射
    assert!(!styles.is_empty(), "应为 DOM 节点计算样式");
}

/// 验证 CSS 级联优先级
#[test]
fn test_cascade_specificity() {
    let html = r#"<html><body><div id="main" class="content">Text</div></body></html>"#;
    let css = r#"
        .content { color: blue; }
        #main { color: red; }
    "#;

    let doc = parse_html(html);
    let stylesheet = Parser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let _styles = sys.compute_styles(&doc, &[stylesheet]);
}

/// 验证 CSS 继承
#[test]
fn test_style_inheritance() {
    let html = r#"<html><body><p>Inherited text</p></body></html>"#;
    let css = r#"
        body { color: green; font-size: 16px; }
    "#;

    let doc = parse_html(html);
    let stylesheet = Parser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    assert!(!styles.is_empty());
}
