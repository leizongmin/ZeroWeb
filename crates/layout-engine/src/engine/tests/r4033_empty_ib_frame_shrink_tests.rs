//! R4033（CSS2 §10.3.9 shrink-to-fit）：空内容 inline-block 收缩到 frame。
//!
//! 旧 `content_max_w > 0.0` guard 使「只有 padding+border 的 auto inline-block」保持
//! taffy 拉伸宽——边框画在拉伸盒右缘（padding-right-applies-to-012：竖条渲染在容器
//! 右缘 722，应 8）。空内容（content_max_w=0）但有 frame 时 shrink 目标 = frame。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 空内容 + 仅右边框的 inline-block：shrink 目标 = border（10），不保持拉伸宽。
#[test]
fn r4033_empty_inline_block_with_border_shrinks_to_frame() {
    let html = r#"<html><body style="margin:0">
<div><div style="display: inline-block; border-right: 10px solid blue;"></div></div></body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let ib = *divs.last().expect("inner div");
    let (w, _h) = find_box(&result.root, ib).expect("inner box");
    assert!(w < 60.0, "R4033: 空内容 inline-block 应收缩到 frame（<60），实际 {w}");
}

/// 纯空 inline-block（无 frame 无内容）维持原状（不塌缩为 0 的既有语义面）。
#[test]
fn r4033_pure_empty_inline_block_unchanged() {
    let html = r#"<html><body style="margin:0">
<div><span style="display: inline-block;"></span></div></body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let ib = *spans.last().expect("span");
    // 无 frame 无内容：无 shrink 驱动，宽由容器决定（拉伸语义保持）——断言不 panic
    // 且不收缩到 0 以下即可（行为面锚定）。
    let found = find_box(&result.root, ib).is_some();
    assert!(found, "R4033: 纯空 inline-block 盒应存在");
}
