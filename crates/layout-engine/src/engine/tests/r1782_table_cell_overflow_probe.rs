//! R1782 回归守卫：table-cell `height` 是 minimum（cell 应长到容纳内容），非 max/fixed。
//!
//! 背景（WPT `table-cell-overflow-explicit-height-001/002`，css-tables #height-distribution）：
//! `<td style="height:20px;overflow:hidden">` 含 300px tall div + 文本。规范：table-cell 的 `height`
//! 是**最小高度**（cell 应长到容纳 in-flow 内容），故 td 应长到 ~300px+ 而非固定 20px；test(有
//! height+overflow) == ref(无) 两者都显示完整内容。R1782 实证 ZW td height=304（正确 grow 到容纳
//! 300px div + border）；残余 3.94% diff = border 细节 + 文本 font-wall，**非** height-as-minimum
//! 缺口。本测试锁定 height-as-minimum 语义，防未来 table-cell height 改动误把 20px 当 max/fixed
//! （会 break 大量 table reftest）。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

fn find_first<'a>(
    box_node: &'a crate::types::LayoutBox,
    tag: &str,
    doc: &zero_dom::Document,
) -> Option<&'a crate::types::LayoutBox> {
    if let Some(nid) = box_node.node_id {
        if let Some(n) = doc.get(nid) {
            if matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case(tag)) {
                return Some(box_node);
            }
        }
    }
    for c in &box_node.children {
        if let Some(b) = find_first(c, tag, doc) {
            return Some(b);
        }
    }
    None
}

/// table-cell `height:20px` 是 minimum：含 300px 内容时应长到 ~300px+，而非固定 20px。
#[test]
fn test_table_cell_height_is_minimum_grows_to_fit_content() {
    let html = r#"<html><body style="margin:0">
<table border><td style="height:20px;border:2px solid cyan;overflow:hidden">
<div style="height:300px;background:blue">tall</div>
text
</td></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let td = find_first(&result.root, "td", &doc).expect("td LayoutBox should exist");
    // td 应长到容纳 300px div（+border）≈ 300-304px，远大于声明的 height:20px。
    assert!(
        td.height >= 300.0,
        "table-cell height:20px 是 minimum，应 grow 到容纳 300px 内容（>=300），实际 {}；若 <20 说明被误当 max/fixed（css-tables #height-distribution 回归）",
        td.height
    );
}
