//! r2s1：`<input type=hidden>` 不渲染端到端钉（parse → style → layout → 盒树）。
//!
//! 背景（baidu-storm2 r2s1 定位）：hidden input 此前按 tag 默认 inline-block 生成盒并
//! 占位（活体 baidu 首页 15 个 hidden input computed cs=inline-block、兄弟被推移；
//! Chrome 154 实测恒 display:none、gBCR 0×0、不占布局空间）。UA 侧修复在 style-system
//!（UA !important display:none，见 ua_display_tests::input_type_hidden_not_rendered_and_author_cannot_override），
//! 本套件钉住下游消费面：布局树无 hidden input 盒（gBCR/命中面/offsetParent 的共同
//! 真值源缺席 → 零 rect 回落）；「插入 hidden input 前后」页面对同 id 元素几何逐值
//! 一致（对照页口径，同一坐标系内比较，不跨盒路径假设）。
//! https://html.spec.whatwg.org/multipage/input.html#hidden-state-(type=hidden)
//! https://html.spec.whatwg.org/multipage/rendering.html#hidden-elements

use crate::pipeline::RenderPipeline;
use zero_dom::NodeId;

/// 沿布局树查找指定 node 的盒是否存在。
fn has_layout_box(node: &zero_layout_engine::LayoutBox, id: NodeId) -> bool {
    if node.node_id == Some(id) {
        return true;
    }
    node.children.iter().any(|c| has_layout_box(c, id))
}

/// 沿布局树找 node_id 盒的 (x, y, width, height)（相对坐标，与盒树同帧）。
fn find_box_rect(node: &zero_layout_engine::LayoutBox, id: NodeId) -> Option<(f32, f32, f32, f32)> {
    if node.node_id == Some(id) {
        return Some((node.x, node.y, node.width, node.height));
    }
    for child in &node.children {
        if let Some(found) = find_box_rect(child, id) {
            return Some(found);
        }
    }
    None
}

fn render(html: &str) -> std::sync::Arc<zero_layout_engine::LayoutBox> {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(html, "");
    let layout = pipeline.layout().expect("layout");
    std::sync::Arc::clone(&layout.root)
}

/// `<input type=hidden>` 不生成布局盒、不占布局空间：同页插入 hidden input 前后，
/// 两个 text input 的盒几何逐值一致（对照页口径）。gBCR 经 rect 桥消费盒树
///（fill_layout_rect_snapshot 只收有盒节点）→ 无盒 = 零 rect 回落，与 Chrome 一致。
#[test]
fn hidden_input_generates_no_layout_box_and_takes_no_space() {
    let without = r#"<html><body><input type="text" id="a" style="width:100px;height:20px"><input type="text" id="b" style="width:100px;height:20px"></body></html>"#;
    let with = r#"<html><body><input type="text" id="a" style="width:100px;height:20px"><input type="hidden" id="h"><input type="text" id="b" style="width:100px;height:20px"></body></html>"#;

    let doc = zero_dom::parse_html(with);
    let h_id = doc.get_element_by_id("h").expect("hidden input");

    let layout_with = render(with);
    let layout_without = render(without);

    // 树身份一致（两页 DOM 除 hidden input 外全同 → 同 id NodeId 确定性成立）。
    assert!(
        !has_layout_box(&layout_with, h_id),
        "input[type=hidden] 不得生成布局盒（gBCR/命中/offsetParent 共同真值源）"
    );

    let doc_wo = zero_dom::parse_html(without);
    for id in ["a", "b"] {
        let node_with = doc.get_element_by_id(id).expect(id);
        let node_wo = doc_wo.get_element_by_id(id).expect(id);
        let rect_with =
            find_box_rect(&layout_with, node_with).unwrap_or_else(|| panic!("{id} box missing (with page)"));
        let rect_wo =
            find_box_rect(&layout_without, node_wo).unwrap_or_else(|| panic!("{id} box missing (without page)"));
        assert_eq!(
            rect_with, rect_wo,
            "插入 input[type=hidden] 前后 #{id} 盒几何必须逐值一致（不占布局空间）"
        );
    }
}

/// 作者 `display:block`（Chrome 154 实测不可覆盖 hidden input 的不渲染）在布局侧同样
/// 无效：对照页几何仍逐值一致、仍无盒。
#[test]
fn hidden_input_author_display_block_cannot_force_box() {
    let without = r#"<html><body><input type="text" id="a" style="width:100px;height:20px"><input type="text" id="b" style="width:100px;height:20px"></body></html>"#;
    let with = r#"<html><body><input type="text" id="a" style="width:100px;height:20px"><input type="hidden" id="h" style="display:block;width:200px;height:40px"><input type="text" id="b" style="width:100px;height:20px"></body></html>"#;

    let doc = zero_dom::parse_html(with);
    let h_id = doc.get_element_by_id("h").expect("authored hidden input");

    let layout_with = render(with);
    let layout_without = render(without);

    assert!(
        !has_layout_box(&layout_with, h_id),
        "作者 display:block 不可令 input[type=hidden] 生成盒（UA !important）"
    );

    let doc_wo = zero_dom::parse_html(without);
    for id in ["a", "b"] {
        let node_with = doc.get_element_by_id(id).expect(id);
        let node_wo = doc_wo.get_element_by_id(id).expect(id);
        let rect_with =
            find_box_rect(&layout_with, node_with).unwrap_or_else(|| panic!("{id} box missing (with page)"));
        let rect_wo =
            find_box_rect(&layout_without, node_wo).unwrap_or_else(|| panic!("{id} box missing (without page)"));
        assert_eq!(rect_with, rect_wo, "作者覆盖尝试下 #{id} 盒几何仍须一致");
    }
}
