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

    /// 辅助函数：解析 HTML 并运行完整样式+布局管线。
    fn render(html: &str, css: &str) -> (Document, zero_layout_engine::LayoutResult) {
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![Parser::parse_stylesheet(css)];
        let mut style_system = StyleSystem::new();
        style_system.set_viewport(800.0, 600.0);
        let styles = style_system.compute_styles(&doc, &stylesheets);
        let mut layout_engine = LayoutEngine::new(800.0, 600.0);
        let layout = layout_engine.compute(&doc, &styles);
        (doc, layout)
    }

    // ── 基础命中测试 ──

    /// 测试点击链接元素返回 href。
    #[test]
    fn hit_test_finds_anchor_href() {
        let html = r#"<html><body>
            <a href="https://example.com" style="display: block; width: 200px; height: 40px; padding: 10px;">
                Example
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "a { background-color: #eeeeee; }");
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("https://example.com"));
    }

    /// 测试点击视口外返回 None。
    #[test]
    fn hit_test_outside_viewport() {
        let html = r#"<html><body>
            <a href="https://example.com" style="display: block; width: 200px; height: 40px;">
                Link
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 900.0, 20.0).is_none());
    }

    /// 测试点击非链接元素返回 None。
    #[test]
    fn hit_test_non_link_element() {
        let html = r#"<html><body>
            <div style="display: block; width: 200px; height: 40px;">Not a link</div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    // ── 嵌套链接测试 ──

    /// 测试点击嵌套在 div 内的链接能正确找到 href。
    #[test]
    fn hit_test_nested_link_in_div() {
        let html = r#"<html><body>
            <div style="display: block; width: 300px; height: 100px; padding: 20px;">
                <a href="/page" style="display: block; width: 100px; height: 30px;">Link</a>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 30.0, 30.0);
        assert_eq!(href.as_deref(), Some("/page"));
    }

    /// 测试深层嵌套链接（div > p > a）能正确命中。
    #[test]
    fn hit_test_deeply_nested_link() {
        let html = r#"<html><body>
            <div style="width: 400px; height: 200px;">
                <p style="width: 300px; height: 100px;">
                    <a href="https://deep.example.com" style="display: block; width: 200px; height: 40px;">Deep Link</a>
                </p>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 20.0, 20.0);
        assert!(href.is_some(), "深层嵌套链接应能被命中");
    }

    // ── 多链接测试 ──

    /// 测试页面中有多个链接时点击不同位置命中不同链接。
    #[test]
    fn hit_test_multiple_links() {
        let html = r#"<html><body>
            <a href="/first" style="display: block; width: 200px; height: 30px;">First</a>
            <a href="/second" style="display: block; width: 200px; height: 30px;">Second</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        let href1 = hit_test_link(&doc, &layout.root, 50.0, 10.0);
        assert_eq!(href1.as_deref(), Some("/first"));

        let href2 = hit_test_link(&doc, &layout.root, 50.0, 40.0);
        assert_eq!(href2.as_deref(), Some("/second"));
    }

    // ── 边界条件 ──

    /// 测试空 href 的链接不应被返回。
    #[test]
    fn hit_test_empty_href_ignored() {
        let html = r#"<html><body>
            <a href="" style="display: block; width: 200px; height: 40px;">Empty</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    /// 测试 href="#" 的链接不应被返回。
    #[test]
    fn hit_test_hash_href_ignored() {
        let html = r##"<html><body>
            <a href="#" style="display: block; width: 200px; height: 40px;">Hash</a>
        </body></html>"##;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    /// 测试 href 只含空格的链接不应被返回。
    #[test]
    fn hit_test_whitespace_only_href_ignored() {
        let html = r#"<html><body>
            <a href="  " style="display: block; width: 200px; height: 40px;">Whitespace</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    /// 测试点击元素边界（恰好包含）和边界外（恰好不包含）。
    /// 注意：body 有 UA 默认 margin:8px，因此 <a> 元素从约 (8,8) 开始。
    #[test]
    fn hit_test_exact_boundary() {
        let html = r#"<html><body>
            <a href="/edge" style="display: block; width: 100px; height: 50px;">Edge</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        // 元素内部（包含左上角，含 body 8px margin 偏移）
        assert!(hit_test_link(&doc, &layout.root, 8.0, 8.0).is_some());

        // 元素内部（接近右下角但不超出）
        let near_edge = hit_test_link(&doc, &layout.root, 107.0, 57.0);
        assert!(near_edge.is_some());

        // 元素外部（body margin 区域，不应命中链接）
        assert!(hit_test_link(&doc, &layout.root, 0.0, 0.0).is_none());
    }

    /// 测试链接文本包含子元素（如 span）时命中测试仍正确。
    #[test]
    fn hit_test_link_with_inline_children() {
        let html = r#"<html><body>
            <a href="/with-span" style="display: block; width: 200px; height: 40px;">
                <span>Link Text</span>
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("/with-span"));
    }

    /// 测试绝对定位元素的命中测试。
    #[test]
    fn hit_test_absolute_positioned_link() {
        let html = r#"<html><body style="margin: 0;">
            <div style="position: relative; width: 400px; height: 300px;">
                <a href="/abs" style="position: absolute; top: 50px; left: 100px; width: 150px; height: 30px;">Abs</a>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 120.0, 60.0);
        assert_eq!(href.as_deref(), Some("/abs"));
    }

    /// 测试点击空白区域（无任何元素）返回 None。
    #[test]
    fn hit_test_empty_body() {
        let html = "<html><body></body></html>";
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 100.0, 100.0).is_none());
    }

    // ── deepest_node_at 直接测试 ──

    /// 测试 deepest_node_at 选择更深的节点。
    #[test]
    fn test_deepest_node_prefers_deeper() {
        let html = r#"<html><body>
            <div style="width: 200px; height: 100px;">
                <div style="width: 100px; height: 50px;">
                    <span style="display: block; width: 50px; height: 20px;">Inner</span>
                </div>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        // 点击内部 span 的位置
        let mut best = (0usize, doc.root());
        deepest_node_at(&layout.root, 0.0, 0.0, 10.0, 10.0, 0, &mut best);
        // 应该找到一个节点（不一定是 span，取决于布局结果，但深度 > 0）
        assert!(best.0 > 0, "应命中嵌套元素，深度 > 0");
    }

    /// 测试负坐标不命中任何元素。
    #[test]
    fn test_negative_coordinates_miss() {
        let html = r#"<html><body>
            <a href="/test" style="display: block; width: 200px; height: 40px;">Link</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, -10.0, -10.0).is_none());
    }

    /// 测试链接带有查询参数和片段标识符。
    #[test]
    fn hit_test_link_with_query_and_fragment() {
        let html = r#"<html><body>
            <a href="/page?foo=bar#section" style="display: block; width: 200px; height: 40px;">Link</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("/page?foo=bar#section"));
    }
}
