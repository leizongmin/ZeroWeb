//! R5048：替换元素 intrinsic 关键字 × ratio × box-sizing 口径（css-sizing-4 §4.1-4.2）。
//! - replaced-element-034：height:min-content 在 border-box + 内联 padding 下 = 内联
//!   min-content（自然宽 + frame）经比传送，旧 content 口径传送塌 50×50（应 100×100）。
//! - replaced-element-047：min-height:max-content 传送值地板——converter 映射 length(0)
//!   地板全丢，height:0 + min-height:max-content + ar 1 应 100×100（旧 100×0 红透）。

use super::*;
use zero_style_system::StyleSystem;

fn layout_html(html: &str, no_ratio: HashMap<NodeId, (Option<f32>, Option<f32>)>) -> std::sync::Arc<LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = crate::LayoutEngine::new(800.0, 600.0);
    let r = eng.compute_with_img_intrinsic(&doc, &styles, HashMap::new(), HashMap::new(), no_ratio);
    r.root
}

fn find_by_tag<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, tag: &str) -> Option<&'a LayoutBox> {
    let hit = b
        .node_id
        .and_then(|id| doc.get(id))
        .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == tag));
    if hit {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_by_tag(c, doc, tag))
}

#[test]
/// replaced-element-034：height:min-content border-box 口径传送——100×100。
fn r5048_img_min_content_border_box_transfer() {
    let html = r#"<!DOCTYPE html><html><body style="margin:0">
<img src="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='50px'></svg>"
 style="box-sizing:border-box; aspect-ratio:1/1; padding-left:50px; height:min-content; background:green">
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut no_ratio = HashMap::new();
    for id in doc.get_elements_by_tag_name("img") {
        no_ratio.insert(id, (Some(50.0), None));
    }
    let root = layout_html(html, no_ratio);
    let b = find_by_tag(&root, &doc, "img").expect("img box").clone();
    assert_eq!(
        (b.width, b.height),
        (100.0, 100.0),
        "min-content height must transfer through border-box (natural 50 + padding 50); got {:?}",
        (b.width, b.height)
    );
}

#[test]
/// replaced-element-047：min-height:max-content 传送值地板——height:0 抬到 100，100×100。
fn r5048_canvas_min_height_max_content_floor() {
    let html = r#"<!DOCTYPE html><html><body style="margin:0">
<canvas width="100" height="50"
 style="display:block; width:max-content; height:0px; min-height:max-content; aspect-ratio:1; background:green">
</canvas>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let root = layout_html(html, HashMap::new());
    let b = find_by_tag(&root, &doc, "canvas").expect("canvas box").clone();
    assert_eq!(
        (b.width, b.height),
        (100.0, 100.0),
        "min-height:max-content must floor definite height via ratio transfer; got {:?}",
        (b.width, b.height)
    );
}
