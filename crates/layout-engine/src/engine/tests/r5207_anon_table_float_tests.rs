//! R5207（CSS2 §9.5 + §17.2.1）：孤立 table-internal 元素经匿名 table 包装
//! 形成的匿名表根与真表同为块级 BFC，参与 §9.5 float 推开（table_float_fix
//! is_table_box 识别）；同时本 pass 对匿名表根**只推位不重排**——初始布局
//! 已按 shrink-to-fit 定宽，layout_table 重排按 fill-available 度量、C 步
//! beside 填宽臂都会把已 shrink 的表根撑宽（001 案 96→480，cell 背景横跨
//! 整行）。driving: css/CSS2/floats-clear/clear-applies-to-005/006（修前
//! 2.56% 红）+ 001/002/003/007/015（修中回归、gate 后复原 0.00%）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(body: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let html = format!("<html><body style=\"margin:0\">{body}</body></html>");
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
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

/// 005 锚点：float p（320px）旁的孤立 table-row 兄弟群经匿名表根包装后，
/// 应被 §9.5 推到 float 右侧（x=320），且保持 shrink-to-fit 宽（2 cell
/// 48px = 96，非 beside 填宽 480）。修前表根压在 float 下（x=0）。
#[test]
fn r5207_orphan_row_run_pushed_beside_float_keeps_shrink_width() {
    let (doc, result) = layout(
        "<p style=\"float: left; margin: 16px 0; width: 320px;\">float text</p>\
         <div id=\"row1\" style=\"display: table-row;\">\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">a</div>\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">b</div>\
         </div>",
    );
    let node = doc.get_element_by_id("row1").expect("row 元素");
    let wrapper = find_box(&result.root, node).expect("匿名表根盒");
    assert!(wrapper.is_anon_table_root, "孤立 row 应形成匿名表根");
    assert!(
        (wrapper.x - 320.0).abs() < 0.5,
        "匿名表根应被推到 float 右侧 x=320，实得 {}",
        wrapper.x
    );
    assert!(
        (wrapper.width - 96.0).abs() < 0.5,
        "匿名表根应保 shrink-to-fit 宽 96（beside 填宽则 480），实得 {}",
        wrapper.width
    );
}

/// 001 判例回归：float p 旁的 row-group 子树（匿名表根）推位后**宽度不被
/// 重排/填宽撑大**——修中 layout_table 重排与 C 步 beside 填宽臂曾把 96 撑到
/// 480（cell 蓝底横跨整行），gate 后与通用 step5 布局恒等。
#[test]
fn r5207_row_group_wrapper_keeps_shrink_width_beside_float() {
    let (doc, result) = layout(
        "<p style=\"float: left; margin: 16px 0; width: 320px;\">float text</p>\
         <div id=\"tg\" style=\"display: table-row-group;\">\
         <div style=\"display: table-row;\">\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">a</div>\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">b</div>\
         </div>\
         <div style=\"display: table-row;\">\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">c</div>\
         <div style=\"display: table-cell; background: blue; height: 48px; width: 48px;\">d</div>\
         </div>\
         </div>",
    );
    let node = doc.get_element_by_id("tg").expect("row-group 元素");
    let wrapper = find_box(&result.root, node).expect("匿名表根盒");
    assert!(
        (wrapper.x - 320.0).abs() < 0.5,
        "row-group 匿名表根应在 float 右侧 x=320，实得 {}",
        wrapper.x
    );
    assert!(
        (wrapper.width - 96.0).abs() < 0.5,
        "row-group 匿名表根应保 shrink 宽 96（重排/填宽则 480），实得 {}",
        wrapper.width
    );
    assert!(
        (wrapper.height - 96.0).abs() < 0.5,
        "row-group 匿名表根高 96 = 2 行×48，实得 {}",
        wrapper.height
    );
}
