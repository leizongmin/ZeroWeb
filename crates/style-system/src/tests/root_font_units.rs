use std::collections::HashMap;

use super::super::{StyleSystem, computed::FontRelativeMetrics};
use zero_css_parser::{Parser as CssParser, values::LengthValue};
use zero_dom::Document;

#[test]
fn rex_uses_root_font_x_height_and_font_size_ex_uses_parent_metrics() {
    let mut doc = Document::new();
    let html = doc.create_element("html");
    doc.append_child(doc.root(), html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let outer = doc.create_element("div");
    doc.set_attribute(outer, "class", "outer");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("span");
    doc.set_attribute(inner, "class", "inner");
    doc.append_child(outer, inner).unwrap();

    let css = r#"
        html { font-family: RootFont; font-size: 20px; }
        .outer { font-size: 1ex; }
        .inner { font-family: monospace; font-size: 1rex; width: calc(2rex); }
    "#;
    let mut metrics = HashMap::new();
    metrics.insert(
        "RootFont".to_string(),
        FontRelativeMetrics {
            ex_height: 0.25,
            ch_width: 0.5,
            size_adjust: 1.0,
        },
    );
    metrics.insert(
        "monospace".to_string(),
        FontRelativeMetrics {
            ex_height: 0.75,
            ch_width: 0.5,
            size_adjust: 1.0,
        },
    );

    let mut system = StyleSystem::new();
    system.set_font_metric_map(&metrics);
    let styles = system.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);

    assert_eq!(styles[&outer].font_size, LengthValue::Px(5.0));
    assert_eq!(styles[&inner].font_size, LengthValue::Px(5.0));
    assert_eq!(styles[&inner].width, LengthValue::Px(10.0));
}
