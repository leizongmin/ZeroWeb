//! R2428：替换元素 `<img>` 两侧 auto + padding 时 taffy aspect_ratio 覆盖显式 height 回归测试。
//!
//! `apply_replaced_element_sizing`（tree.rs）对无显式尺寸的 `<img>`（两侧 auto）把
//! size.width/height 都设为固有值，但旧代码同时设 aspect_ratio（条件 `width_auto || height_auto`）。
//! taffy 把 aspect_ratio 作用到 **border-box** 宽（含 padding），覆盖显式 height：固有 60×60 +
//! `padding-right:40` 渲染成 border-box 100×100（应 100×60）。fix：条件改 XOR（恰好一侧 auto），
//! 两侧均 auto（等同两侧显式）不设 aspect_ratio。driving：css/css-box/margin-trim/block-container-
//! replaced-*（img 不渲染绿→假 fail；R2427 让 `/css/support/...` 图片加载后暴露此 sizing bug）。
use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout_img(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut sizes = std::collections::HashMap::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        sizes.insert(img_id, (60.0_f32, 60.0_f32));
    }
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute_with_img_intrinsic(
        &doc,
        &styles,
        sizes,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    (doc, Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone()))
}

fn find_img<'a>(r: &'a LayoutBox, d: &zero_dom::Document) -> Option<&'a LayoutBox> {
    if r.node_id.is_some_and(|nid| {
        d.get(nid)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "img"))
    }) {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_img(c, d) {
            return Some(b);
        }
    }
    None
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// 两侧 auto + padding-right:40 + 父 min-content：border-box 应 100×60（内容 60 + padding 40 宽，
/// 高 60），不因 aspect_ratio 被拉到 100×100。driving: css-box/margin-trim/block-container-replaced-*。
#[test]
fn test_img_both_auto_padding_not_stretched_by_aspect_ratio() {
    let html = r#"<html><body style="margin:0">
<div style="width:min-content">
  <img id="i" style="display:block; padding-right:40px" src="x.png">
</div>
</body></html>"#;
    let (doc, root) = layout_img(html);
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 100.0),
        "border-box width 60+40padding=100; got {}",
        img.width
    );
    assert!(
        approx(img.height, 60.0),
        "height stays intrinsic 60 (no vert padding); got {}",
        img.height
    );
    assert!(
        approx(img.content_width, 60.0),
        "content width 60; got {}",
        img.content_width
    );
}

/// 对照：无 padding → 60×60（fix 不回归既有行为）。
#[test]
fn test_img_both_auto_no_padding_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="width:min-content">
  <img id="i" style="display:block" src="x.png">
</div>
</body></html>"#;
    let (doc, root) = layout_img(html);
    let img = find_img(&root, &doc).expect("img");
    assert!(approx(img.width, 60.0), "no padding: 60 wide; got {}", img.width);
    assert!(approx(img.height, 60.0), "no padding: 60 tall; got {}", img.height);
}

/// 对照：definite 宽容器 + padding → 仍 100×60（min-content 非诱因，padding+aspect_ratio 才是）。
#[test]
fn test_img_both_auto_padding_definite_container() {
    let html = r#"<html><body style="margin:0">
<div style="width:200px">
  <img id="i" style="display:block; padding-right:40px" src="x.png">
</div>
</body></html>"#;
    let (doc, root) = layout_img(html);
    let img = find_img(&root, &doc).expect("img");
    assert!(approx(img.width, 100.0), "border-box 100; got {}", img.width);
    assert!(approx(img.height, 60.0), "height 60; got {}", img.height);
}
