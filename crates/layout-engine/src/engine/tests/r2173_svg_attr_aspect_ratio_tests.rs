//! R2173：SVG data URI unitless width/height attr 经 `extract_svg_data_uri_size` 命中
//! `apply_replaced_element_sizing` 的 attr 分支（463/474）时，须补设 aspect_ratio（从
//! decoded intrinsic 比），否则替换元素 cross 维 = 0。
//! driving：css-flexbox/flex-aspect-ratio-img-row-010（200×0 应 ~100×100；几何修对，
//! diff 2.12→1.05%，残余=font-wall）+ img-row-011（0×100，width 未推）。
//!
//! 根因：`extract_attr_float` 解析 unitless '200' → Some(200)（命中 attr 分支 463/474，
//! 仅设 size 不设 aspect_ratio），而 '50px' → None（落 img_intrinsic_sizes 分支，设
//! aspect_ratio）。R2173 在 attr 分支补设 aspect_ratio 从 decoded img_intrinsic_sizes。
//! kill-switch `ZW_SVG_ATTR_AR=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// R2173 主驱：SVG data URI unitless width attr（width='200'）+ decoded intrinsic (200,200)
/// 须设 aspect_ratio，使 flex item cross（height）推导（非 0）。driving img-row-010。
#[test]
fn r2173_svg_datauri_unitless_width_sets_aspect_ratio() {
    let html = r#"<html><body>
<div style="display:flex;">
  <img id="img" src="data:image/svg+xml,%3Csvg viewBox='0 0 100 100' width='200'%3E%3Crect width='100%25' height='100%25' fill='green'/%3E%3C/svg%3E" style="max-height:100px; flex:0 0 auto;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_element_by_id("img").expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (200.0f32, 200.0f32)); // decoded ComputedIntrinsic
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, HashMap::new());
    let img = find(&result.root, img_id).expect("img box");
    // 修复前：attr 分支不设 aspect_ratio → height=0。修复后：设 aspect_ratio=1.0 →
    // height 从 width/max-height 推导（~100，非 0）。
    assert!(
        img.height > 50.0,
        "SVG data URI unitless width attr must set aspect_ratio so height derives (not 0); got h={}",
        img.height
    );
}

/// R2173 对称：SVG data URI unitless height attr（height='200'）+ decoded intrinsic 须设
/// aspect_ratio，使 width 推导（非 0）。driving img-row-011。
#[test]
fn r2173_svg_datauri_unitless_height_sets_aspect_ratio() {
    let html = r#"<html><body>
<div style="display:flex;">
  <img id="img" src="data:image/svg+xml,%3Csvg viewBox='0 0 100 100' height='200'%3E%3Crect width='100%25' height='100%25' fill='green'/%3E%3C/svg%3E" style="max-height:100px; flex:0 0 auto;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_element_by_id("img").expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (200.0f32, 200.0f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, HashMap::new());
    let img = find(&result.root, img_id).expect("img box");
    // 修复前：width=0。修复后：设 aspect_ratio → width 推导（~100，非 0）。
    assert!(
        img.width > 50.0,
        "SVG data URI unitless height attr must set aspect_ratio so width derives (not 0); got w={}",
        img.width
    );
}
