//! R5408：替换盒 min/max-height 钳后宽度回传（CSS2 §10.4 constraint-violation 表）。
//!
//! R2428 both-auto 注入域（父非 flex/grid）把 img size.width/height 写成固有值
//! definite、不设 aspect_ratio（R325：taffy 强制比例会覆盖显式值）——min/max-height
//! 钳住 used height 后 taffy 不按钳后高回传宽（intrinsic-percent-replaced-017：
//! img 固有 200×200 + max-height:100% 渲 200×100，chromium 100×100）。
//! kill-switch `ZW_REPLACED_CLAMP_TRANSFER=0`。
use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout_html(html: &str, intrinsic: f32) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut sizes = std::collections::HashMap::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        sizes.insert(img_id, (intrinsic, intrinsic));
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

fn layout_html_with_signals(
    html: &str,
    sizes_map: std::collections::HashMap<zero_dom::NodeId, (f32, f32)>,
    no_ratio_map: std::collections::HashMap<zero_dom::NodeId, (Option<f32>, Option<f32>)>,
) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute_with_img_intrinsic(&doc, &styles, sizes_map, std::collections::HashMap::new(), no_ratio_map);
    (doc, Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone()))
}

fn find_by_id<'a>(r: &'a LayoutBox, d: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    if r.node_id.is_some_and(|nid| {
        d.get(nid).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.get_attribute("id").as_deref() == Some(id),
            _ => false,
        })
    }) {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_by_id(c, d, id) {
            return Some(b);
        }
    }
    None
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// ★load-bearing（017 根因锚）：flex column 100×100 + align-items:center，inner
/// max-height:100%，img 固有 200×200 + max-height:100%——used 高被钳 100 后宽应
/// 回传 100×100（钳前 200×100 假 fail）。
#[test]
fn r5408_max_height_clamp_transfers_width() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="width:100px;height:100px;display:flex;flex-direction:column;align-items:center">
  <div id="inner" style="max-height:100%">
    <img id="i" style="display:block;max-height:100%" src="x.png">
  </div>
</div>
</body></html>"#;
    let (doc, root) = layout_html(html, 200.0);
    let img = find_by_id(&root, &doc, "i").expect("img");
    assert!(
        approx(img.width, 100.0),
        "clamped used height 100 × ratio 1 → width 100; got {}",
        img.width
    );
    assert!(
        approx(img.height, 100.0),
        "used height stays clamp 100; got {}",
        img.height
    );
    let inner = find_by_id(&root, &doc, "inner").expect("inner");
    assert!(
        approx(inner.width, 100.0),
        "flex item cross follows transferred img 100; got {}",
        inner.width
    );
}

/// 门：无 min/max-height 声明（无钳）不触发——both-auto 保持固有 200×200。
#[test]
fn r5408_no_clamp_stays_intrinsic() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="width:100px;height:100px">
  <img id="i" style="display:block" src="x.png">
</div>
</body></html>"#;
    let (doc, root) = layout_html(html, 200.0);
    let img = find_by_id(&root, &doc, "i").expect("img");
    assert!(approx(img.width, 200.0), "no clamp → intrinsic 200; got {}", img.width);
    assert!(
        approx(img.height, 200.0),
        "no clamp → intrinsic 200; got {}",
        img.height
    );
}

/// 门：CSS width definite（非 R2428 both-auto 注入域）本臂不触碰——无钳时 definite
/// 宽语义由既有路径处理，宽保持 150。注：definite 宽 **+ max-height 钳** 域存在
/// 既有独立覆盖（taffy ratio 强制，未回传前的 pre-existing，chromium 应保持 150×100
/// per CSS2 §10.4），与本臂无关、另案挂账，此处不纠缠该域。
#[test]
fn r5408_definite_width_not_overridden() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="width:100px;height:100px">
  <img id="i" style="display:block;width:150px" src="x.png">
</div>
</body></html>"#;
    let (doc, root) = layout_html(html, 200.0);
    let img = find_by_id(&root, &doc, "i").expect("img");
    assert!(approx(img.width, 150.0), "definite width stays 150; got {}", img.width);
    assert!(
        approx(img.height, 150.0),
        "auto height transfers from definite width (ratio 1); got {}",
        img.height
    );
}

/// 回归锚（visudet replaced-elements-min/max-* 六案）：no-ratio SVG 的 bogus 解码
/// sizes entry（img 路径无 ratios/no_ratio 互斥）+ no_ratio 信号共存时，臂必须跳过
/// ——无固有比者 CSS2 §10.4 不得传（min-height 钳高后宽保持注入值，不按比放大）。
#[test]
fn r5408_no_ratio_signal_suppresses_transfer() {
    let html = r#"<html><body style="margin:0">
<div style="width:200px">
  <img id="i" style="display:block;min-height:300px" src="x.svg">
</div>
</body></html>"#;
    let doc_parsed = zero_dom::parse_html(html);
    let mut sizes = std::collections::HashMap::new();
    let mut no_ratio = std::collections::HashMap::new();
    for img_id in doc_parsed.get_elements_by_tag_name("img") {
        sizes.insert(img_id, (150.0_f32, 150.0_f32));
        no_ratio.insert(img_id, (Some(150.0_f32), None));
    }
    let (doc, root) = layout_html_with_signals(html, sizes, no_ratio);
    let img = find_by_id(&root, &doc, "i").expect("img");
    assert!(
        approx(img.height, 300.0),
        "min-height clamps used height 300; got {}",
        img.height
    );
    assert!(
        approx(img.width, 150.0),
        "no intrinsic ratio → no transfer (regression would give 300); got {}",
        img.width
    );
}

/// 回归锚（css-sizing aspect-ratio replaced-element-045）：max-height:none 内部表示
/// Px(INFINITY)，门漏判曾把**全部** both-auto 替换盒当有 max-height 而按 laid_h×attr 比
/// 错传（corpus 045 canvas 100×100 被写 400×400，3×3 定谳）。本形在单元环境无 R3994
/// coercion（corpus 语义由其全量窗口验证），broken 门下会写 laid_h×(40/10)=280——
/// 门修后臂惰性，保持 padding 内含 border-box 40+60+60=160。
#[test]
fn r5408_max_height_none_is_not_a_constraint() {
    let html = r#"<html><body style="margin:0">
<canvas id="c" width="40" height="10" style="aspect-ratio: 1; background: green; padding-right: 60px; padding-bottom: 60px; box-sizing: border-box;"></canvas>
</body></html>"#;
    let doc_parsed = zero_dom::parse_html(html);
    let mut sizes = std::collections::HashMap::new();
    for c_id in doc_parsed.get_elements_by_tag_name("canvas") {
        sizes.insert(c_id, (40.0_f32, 10.0_f32));
    }
    let (doc, root) = layout_html_with_signals(html, sizes, std::collections::HashMap::new());
    let canvas = find_by_id(&root, &doc, "c").expect("canvas");
    assert!(
        approx(canvas.width, 160.0),
        "no real min/max-height → arm inert (broken gate would give 280); got {}",
        canvas.width
    );
    assert!(
        approx(canvas.height, 160.0),
        "arm inert: same as kill-switch OFF env (160); got {}",
        canvas.height
    );
}
