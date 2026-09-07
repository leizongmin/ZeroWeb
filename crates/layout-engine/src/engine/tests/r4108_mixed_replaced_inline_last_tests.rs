//! R4108（CSS2 §9.2.1.1 块容器的匿名块序列 + §10.3.2 replaced inline）：块容器
//! 「块级子 + 末位 inline replaced 子」混排时跳过 inline-only remeasure——IFC 重测以
//! 容器内容原点为行盒起点（无视块级子已占据的流高度），重跑后
//! sync_inline_child_boxes_from_ifc 把 inline 子 y 从 taffy 匿名块堆叠位覆写回 0，
//! inline svg 与块级 p 重叠（view-box 五案 2.92% 像素 y 偏 36px 根因）。
//!
//! 锚：`<p>x</p><svg width=400 height=200>` 的 svg 盒 y = p 高之后（taffy 堆叠位），
//! 不再被 IFC 同步覆写回 0；含非 replaced inline 子（span）或尾随块级子的容器
//! 保持旧 IFC 路径（block-in-inline-insert / content-visibility-025 域）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.y, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("p { margin: 0; } svg { margin: 0; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 末位 inline replaced 子（svg）的 y = 块级 p 的流高之后（taffy 匿名块堆叠位），
/// 不被 IFC 重测覆写回容器原点。
#[test]
fn r4108_trailing_inline_svg_stacks_after_block_sibling() {
    let html = r#"<html><body style="margin:0">
<p>x</p>
<svg width="400" height="200"><rect width="200" height="200" fill="green"/></svg>
</body></html>"#;
    let (doc, result) = layout(html);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (y, h) = find_box(&result.root, svg_id).expect("svg box");
    // p 高 ~19px（Ahem/默认字体行高）；svg y 应为 p 高（≥18），旧覆写为 0。
    assert!(
        y >= 18.0,
        "末位 inline svg 的 y 应在块级 p 流高之后（≥18），got {y}（IFC 覆写回 0 = R4108 缺陷回归）"
    );
    assert_eq!(h, 200.0, "svg 高应保持 attr 值 200");
}

/// 含非 replaced inline 子（span）的混排容器：保持旧 IFC 路径（block-in-inline-insert
/// 域依赖），其 inline 子几何不受本 gate 影响。
#[test]
fn r4108_mixed_with_span_keeps_legacy_path() {
    let html = r#"<html><body style="margin:0">
<p>x</p>
<span>t</span><svg width="400" height="200"><rect width="200" height="200" fill="green"/></svg>
</body></html>"#;
    let (doc, result) = layout(html);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    // span 非 replaced → gate 不触发 → svg 走旧 IFC 同步（y 可能为 0，域归后续切片）。
    // 本锚只断言不 panic 且 svg 盒存在（尺寸不被破坏）。
    let (_y, h) = find_box(&result.root, svg_id).expect("svg box");
    assert_eq!(h, 200.0, "svg 高保持 200");
}
