//! R5329（css-sizing-3 + css-flexbox-1 §4.5）：替换内容叶（img）intrinsic 测量回
//! 固有尺寸锚——measure 空叶分支对 MinContent/MaxContent 面旧回 0，taffy flexbox
//! 的 automatic minimum size fallback（§4.5 specified/content size suggestion 取小）
//! 拿到 0 → min-size:auto 塌 0，img item 在窄容器缩进（flex-minimum-width-flex-items-
//! 005/006：img width:100px + 200×200/100×100 固有，10px 容器缩到 ~10px）。
//! 替换元素的 min-content/max-content = 固有尺寸（css-sizing-3），与显式 size 的
//! 取小钳由 taffy 侧完成；非替换空叶仍回 0（flex-shrink-001 族守卫腿）。

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

fn compute_with_intrinsic(html: &str, iw: f32, ih: f32) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let imgs = doc.get_elements_by_tag_name("img");
    let mut intrinsic = std::collections::HashMap::new();
    for id in imgs {
        intrinsic.insert(id, (iw, ih));
    }
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, intrinsic, std::collections::HashMap::new());
    (doc, result)
}

/// flex-minimum-width-flex-items-005 驱动案：10px flex 容器 + img item
///（width:100px，固有 200×200）。specified size suggestion 100 < content size
/// suggestion 200 → automatic min = 100，item 保持 100px（溢出容器）。
#[test]
fn r5329_img_flex_item_min_is_specified_size_suggestion() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width: 10px;">
  <img src="200x200-green.png" style="width: 100px;">
</div>
</body></html>"#;
    let (doc, result) = compute_with_intrinsic(html, 200.0, 200.0);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let (w, _h) = find_box(&result.root, img).expect("img box");
    assert!(
        (w - 100.0).abs() < 1.0,
        "R5329: img item 应保持 specified size suggestion 100px，实际 {w}"
    );
}

/// flex-minimum-width-flex-items-006 驱动案：同形，固有 100×100——min-content =
/// 固有 100 = specified 100，item 保持 100px。
#[test]
fn r5329_img_flex_item_min_equals_intrinsic() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width: 10px;">
  <img src="100x100-green.png" style="width: 100px;">
</div>
</body></html>"#;
    let (doc, result) = compute_with_intrinsic(html, 100.0, 100.0);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let (w, _h) = find_box(&result.root, img).expect("img box");
    assert!(
        (w - 100.0).abs() < 1.0,
        "R5329: img item 应保持 100px（min(固有, 显式)），实际 {w}"
    );
}

/// 守卫腿（flex-shrink-001 族）：非替换空叶（div）在 content 测量面仍回 0——
/// 显式 width 不回填，min-size:auto 不阻收缩（R4008 守卫语义不变）。
#[test]
fn r5329_non_replaced_empty_leaf_still_shrinks() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width: 10px;">
  <div style="width: 100px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(
        &doc,
        &styles,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let (w, _h) = find_box(&result.root, item).expect("item box");
    assert!(
        (w - 10.0).abs() < 1.0,
        "R5329 守卫: 非替换空叶仍应收缩到容器 10px，实际 {w}"
    );
}
