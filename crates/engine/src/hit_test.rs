//! 布局树命中测试 — 用于链接点击等交互。

use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;

/// 在布局树中查找点击位置对应的最深 DOM 节点。
fn deepest_node_at(
    layout: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    point_x: f32,
    point_y: f32,
    depth: usize,
    best: &mut (usize, NodeId),
) {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;

    if point_x < box_x || point_y < box_y || point_x >= box_x + layout.width || point_y >= box_y + layout.height {
        return;
    }

    if let Some(node_id) = layout.node_id
        && depth >= best.0
    {
        *best = (depth, node_id);
    }

    for child in &layout.children {
        deepest_node_at(child, box_x, box_y, point_x, point_y, depth + 1, best);
    }
}

/// 从节点向上查找最近的 `<a href="...">`。
fn find_link_href(doc: &Document, mut node: NodeId) -> Option<String> {
    loop {
        let is_anchor = doc.get(node).is_some_and(
            |data| matches!(&data.kind, NodeKind::Element(elem) if elem.local_name().eq_ignore_ascii_case("a")),
        );
        if is_anchor && let Some(href) = doc.get_attribute(node, "href") {
            let href = href.trim();
            if !href.is_empty() && href != "#" {
                return Some(href.to_string());
            }
        }
        node = doc.parent_node(node)?;
    }
}

/// 在文档布局中命中测试链接，返回 `href`（若存在）。
pub fn hit_test_link(doc: &Document, layout: &LayoutBox, x: f32, y: f32) -> Option<String> {
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, x, y, 0, &mut best);
    find_link_href(doc, best.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::Parser;
    use zero_layout_engine::LayoutEngine;
    use zero_style_system::StyleSystem;

    #[test]
    fn hit_test_finds_anchor_href() {
        let html = r#"<html><body>
            <a href="https://example.com" style="display: block; width: 200px; height: 40px; padding: 10px;">
                Example
            </a>
        </body></html>"#;
        let css = "a { background-color: #eeeeee; }";
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![Parser::parse_stylesheet(css)];
        let mut style_system = StyleSystem::new();
        style_system.set_viewport(800.0, 600.0);
        let styles = style_system.compute_styles(&doc, &stylesheets);
        let mut layout_engine = LayoutEngine::new(800.0, 600.0);
        let layout = layout_engine.compute(&doc, &styles);

        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("https://example.com"));

        assert!(hit_test_link(&doc, &layout.root, 900.0, 20.0).is_none());
    }
}
