//! R5294 结构锚（CSS2 §9.4.2 × §16.3.2）：动态构建的 DOM（createElement/
//! createTextNode/appendChild）与静态同形标记布局须逐盒一致——行内格式化上下文
//! 不因 DOM 构建路径差异改变文本折叠（行中空格保留仅在行缘吞格）。
//! dir-style-04 的实际丢格根因在 engine 侧 innerHTML fragment 解析 trim
//!（js_dom_bridge_tests::test_apply_set_inner_html_trailing_ws_preserved_r5294），
//! 本锚证明布局面无路径分歧（修前即绿，排查中排除布局嫌疑的证据固化）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
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

/// 同一 DOM 形状（div{文本 "start. " + span x{"mmm. nnn. " + span y}}）静态解析
/// 与动态构建两条路径布局须一致：span x 盒宽含两处尾随空格。
#[test]
fn r5294_dyn_append_trailing_space_matches_static() {
    // 静态参照。
    let html = r#"<html><body style="margin:0"><div id="d">start. <span id="xs">mmm. nnn. <span id="ys"></span></span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let xs_id = doc.get_element_by_id("xs").expect("xs");
    let ys_id = doc.get_element_by_id("ys").expect("ys");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let xs_static = find(&result.root, xs_id).unwrap();
    let ys_static = find(&result.root, ys_id).unwrap();

    // 动态构建：createElement + 独立文本节点 + append（同 dir-style-04 appendChildren）。
    let mut doc = zero_dom::parse_html(r#"<html><body style="margin:0"><div id="d">start. </div></body></html>"#);
    let xs = doc.create_element("span");
    let text = doc.create_text_node("mmm. nnn. ");
    doc.append_child(xs, text).expect("append text");
    let ys = doc.create_element("span");
    doc.append_child(xs, ys).expect("append ys");
    let d = doc.get_element_by_id("d").expect("d");
    doc.append_child(d, xs).expect("append xs");

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let xs_id = xs;
    let ys_id = ys;
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let xs_dyn = find(&result.root, xs_id).unwrap();
    let ys_dyn = find(&result.root, ys_id).unwrap();

    assert!(
        (xs_dyn.width - xs_static.width).abs() < 0.5,
        "span x 盒宽静态 {} vs 动态 {}（动态丢尾随空格则变窄）",
        xs_static.width,
        xs_dyn.width
    );
    assert!(
        (ys_dyn.x - ys_static.x).abs() < 0.5,
        "span y 起点：静态 {} vs 动态 {}（丢格则左移一个空格宽）",
        ys_static.x,
        ys_dyn.x
    );
}
