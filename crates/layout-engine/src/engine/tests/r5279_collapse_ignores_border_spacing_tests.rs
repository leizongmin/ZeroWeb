//! R5279 回归：collapse 表 border-spacing 整体无效（CSS2 §17.6.1）。
//!
//! collapsing-border-model-010a（✗9.87）：collapse 表声明 `border-spacing:50px`
//! 必须被忽略。旧实现三处漏门控——layout_table_inner 源头（下游列间/行间 gap 与
//! 列背景推进）、apply_table_size_constraints 的 spacing_total_x（表 100 被撑到
//! 208）、update_row_group_positions 的行间推进（行组盒与 cell 错位、tbody 行带
//! 露表底红色）。同案 010b（无 spacing）修前已 ✓ 佐证唯一 delta 即 spacing。
//!
//! separated 模式不受影响：周界 + 列间 spacing 语义由
//! visibility-collapse-border-spacing-002 / separated-border-model-003b 锚定。

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in b.children.iter() {
            stack.push(c);
        }
    }
    None
}

/// collapse 表 + border-spacing:50px：列间与周界 spacing 均不计——两列 100 表盒
/// 应 200（旧缺陷 250=含列间 / 300=含周界），第二列 cell x 应 100（旧 150）。
#[test]
fn r5279_collapse_table_ignores_border_spacing() {
    let html = r#"<html><head><style>
#t { border-collapse: collapse; border-spacing: 50px; }
td { width: 100px; }
</style></head><body>
<table id="t"><tr><td id="c1"></td><td id="c2"></td></tr></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table = doc.get_element_by_id("t").expect("table 应存在");
    let tb = find_box(&result.root, table).expect("table 盒应存在");
    assert!(
        (tb.width - 200.0).abs() < 0.5,
        "collapse 表 border-spacing 无效：表盒应 200（列宽和），实际 {}（旧缺陷 spacing_total_x 撑到 250+）",
        tb.width
    );
    let c2 = doc.get_element_by_id("c2").expect("c2 应存在");
    let c2b = find_box(&result.root, c2).expect("c2 盒应存在");
    let c1 = doc.get_element_by_id("c1").expect("c1 应存在");
    let c1b = find_box(&result.root, c1).expect("c1 盒应存在");
    let second_x = c2b.x - c1b.x;
    assert!(
        (second_x - 100.0).abs() < 0.5,
        "collapse 表列间无 gap：第二列相对第一列应 100，实际 {}（旧缺陷 150 = 多一个 spacing）",
        second_x
    );
}

/// 对照组：同形态 separated 表 spacing 照常生效——表盒 = 200 + 列间 50 + 周界 100
/// = 350，第二列 x = 150（§17.6.1 周界+列间语义，R4243 口径）。
#[test]
fn r5279_separate_table_keeps_border_spacing() {
    let html = r#"<html><head><style>
#t { border-spacing: 50px; }
td { width: 100px; }
</style></head><body>
<table id="t"><tr><td id="c1"></td><td id="c2"></td></tr></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table = doc.get_element_by_id("t").expect("table 应存在");
    let tb = find_box(&result.root, table).expect("table 盒应存在");
    assert!(
        (tb.width - 350.0).abs() < 0.5,
        "separated 表 spacing 照常：表盒 = 200 + 列间 50 + 周界 100 = 350，实际 {}",
        tb.width
    );
    let c2 = doc.get_element_by_id("c2").expect("c2 应存在");
    let c2b = find_box(&result.root, c2).expect("c2 盒应存在");
    let c1 = doc.get_element_by_id("c1").expect("c1 应存在");
    let c1b = find_box(&result.root, c1).expect("c1 盒应存在");
    let second_x = c2b.x - c1b.x;
    assert!(
        (second_x - 150.0).abs() < 0.5,
        "separated 表列间 gap 50：第二列相对第一列应 150，实际 {}",
        second_x
    );
}
