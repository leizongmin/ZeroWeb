//! R4023（CSS2 §10.3.3 + §10.3.9）：inline-block shrink-to-fit 后块级子宽同步。
//!
//! taffy 第一趟按收缩前可用宽把 inline-block 的块级子拉到满宽（784）；step 5.6 收缩
//! inline-block 自身后无重排，子保持拉伸值——子带背景时溢出收缩后的父盒可见
//! （inline-block-zorder-005：黄条 791px 应 ~96px）。本 pass 在收缩实际发生时把
//! 「CSS width:auto 的 in-flow 块级子」宽同步到父新内容宽（收缩方向，单向不放大）。

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

/// inline-block 含块级子（div，文本 10 字符）时：父 shrink-to-fit 后子宽应同步到
/// 父内容宽（~文本宽），而非保持 taffy 拉伸的满宽（zorder-005 域）。
#[test]
fn r4023_block_child_width_syncs_after_inline_block_shrink() {
    let html = r#"<html><body style="margin:0">
<span style="display: inline-block;"><div style="background: yellow">xxxxxxxxxx</div></span>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let did = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, did).expect("target box");
    // 10 字符 ~80px（16px 字号），远小于满宽 784；旧实现子保持 784。
    assert!(
        w < 200.0,
        "R4023: inline-block 收缩后块级子宽应同步到内容宽（<200），实际 {w}"
    );
}

/// 对照锚（行为不回退）：inline-block 含 inline 文本子时父收缩语义本就正确
///（R180/R1017 域），同步 pass 不得破坏。
#[test]
fn r4023_text_only_inline_block_shrink_unchanged() {
    let html = r#"<html><body style="margin:0">
<span style="display: inline-block; background: green">xxxxxxxxxx</span>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let sid = *spans.last().expect("target span");
    let (w, _h) = find_box(&result.root, sid).expect("target box");
    assert!(
        w < 200.0,
        "R4023: 纯文本 inline-block shrink-to-fit 语义不变（<200），实际 {w}"
    );
}
