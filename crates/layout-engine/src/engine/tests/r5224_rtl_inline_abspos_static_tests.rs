//! R5224（CSS2 §10.3.7 × css-position-3 §4.1）：rtl 容器行首 inline abspos 静态位镜像。
//! 行内锚是行构建期 LTR 物理光标（行内镜像/对齐前推进值）；rtl 行首的假想盒
//! inline-start 贴**行右缘**、向 inline-end 延伸（chromium 探针：盒 margin-box 右缘
//! = 行右缘 = CB 内容右缘）。行中锚 / LTR 行为不变（锚 = 盒 margin-box 左缘）。
//! abspos-block-level-001（rtl div1 span.abs 3.46%→1.46% 后余红为 ref 空行 strut
//! 堆叠域，另行挂账）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// rtl 行首 inline abspos：盒右缘贴容器内容右缘（margin 0），y = 行顶。
#[test]
fn r5224_rtl_line_start_inline_abspos_right_edge_at_line_start() {
    let html = r#"<html><body style="margin:0">
<div id=c style="direction: rtl"><span id=a style="position: absolute">ABS</span><b>text</b></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let anode = doc.get_element_by_id("a").expect("span#a 节点");
    let cnode = doc.get_element_by_id("c").expect("div#c 节点");
    let abox = find_box(&result.root, anode).expect("span#a 盒应存在");
    let cbox = find_box(&result.root, cnode).expect("div#c 盒应存在");
    assert!(
        (abox.x + abox.width - cbox.content_width).abs() <= 0.5,
        "rtl 行首 inline abspos 盒右缘应贴内容右缘（x + w = content_width），实际 x={} w={} content_w={}",
        abox.x,
        abox.width,
        cbox.content_width
    );
    assert_eq!(abox.y, 0.0, "应锚在第 1 行行顶，实际 y={}", abox.y);
}

/// LTR 行首 inline abspos：锚 = 盒 margin-box 左缘（行左缘），镜像分支不得误伤。
#[test]
fn r5224_ltr_line_start_inline_abspos_stays_at_line_left() {
    let html = r#"<html><body style="margin:0">
<div id=c><span id=a style="position: absolute">ABS</span><b>text</b></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let anode = doc.get_element_by_id("a").expect("span#a 节点");
    let abox = find_box(&result.root, anode).expect("span#a 盒应存在");
    assert!(
        abox.x.abs() <= 0.5,
        "LTR 行首 inline abspos 盒左缘应贴行左缘，实际 x={}",
        abox.x
    );
}
