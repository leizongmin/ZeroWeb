//! R2429：`contain: size` 替换元素固有尺寸忽略回归测试。
//!
//! CSS Containment 1：`contain: size` 使元素按「无内容」sized，替换元素固有尺寸须忽略
//!（intrinsic → 0）。`apply_replaced_element_sizing`（tree.rs）旧实现无条件用固有尺寸覆盖
//! converter 的 contain:size→0，破坏 size containment。fix：contain:size 时早返回，让
//! converter（mod.rs:123 `contain.has_size()`，含 contain-intrinsic-size 覆盖）生效。
//! driving：css-contain/contain-size-013（`<img contain:size padding:50>` 固有 60×60 应
//! padding-only=100×100，非 160×160）。承接 R2427（让 `/css/...` 图片加载暴露此 bug）+
//! R2428（aspect_ratio sizing）。
use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout_with_img_intrinsic(html: &str, intrinsic: (f32, f32)) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut sizes = std::collections::HashMap::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        sizes.insert(img_id, intrinsic);
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

/// `contain:size` + padding:50 + 固有 60×60 → box 100×100（padding-only，固有忽略）。
/// driving: css-contain/contain-size-013。
#[test]
fn test_contain_size_img_ignores_intrinsic() {
    let html = r#"<html><body style="margin:0">
<img style="contain: size; padding: 50px; background: green" src="x.png">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 100.0),
        "contain:size: box = padding 50×2 = 100 wide (intrinsic ignored); got {}",
        img.width
    );
    assert!(
        approx(img.height, 100.0),
        "contain:size: box = padding 50×2 = 100 tall; got {}",
        img.height
    );
}

/// 对照：无 contain:size → 固有 60×60 + padding 50×2 = 160×160（fix 不破坏既有替换元素 sizing）。
#[test]
fn test_no_contain_size_img_uses_intrinsic() {
    let html = r#"<html><body style="margin:0">
<img style="padding: 50px; background: green" src="x.png">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 160.0),
        "no contain:size: 60 intrinsic + 50×2 padding = 160; got {}",
        img.width
    );
    assert!(
        approx(img.height, 160.0),
        "no contain:size: 160 tall; got {}",
        img.height
    );
}
