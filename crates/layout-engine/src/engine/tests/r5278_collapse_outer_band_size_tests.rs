//! R5278 回归：collapse cell 胜出缘外半带与表自身 border 带同条不双计（CSS2 §17.6.2.1）。
//!
//! collapsing-border-model-001/004（✗1.36）：表 `border: 25px`、cell `border: 50px`
//! 胜出四缘时，表盒宽 = content（边框中心距 170）+ 自身边框 25×2，胜出带外半展进
//! [盒缘, content 缘] 同一窄条，不再加 outer_half → 220 = chromium（#div1 参照宽）。
//! 旧实现 outer_half 叠加在自身 border 之上双计同条 → 270。
//!
//! R5241 原锚（表 border 0，外半整计入）由半式 `outer/2 − 自身边框` 镜像保持：
//! border 0 时补整外半，量不变。

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

/// collapsing-border-model-001 形态：collapse 表自身 border 25，cell border 50 胜出
/// 四缘，td width 100 + padding 10 → 边框中心距 170，表盒应 220（胜出带 [0,50]/[170,220]
/// 以格线为中心，内容 [50,170]）。
#[test]
fn r5278_cell_won_outer_band_not_double_counted_over_table_border() {
    let html = r#"<html><head><style>
table { border: 25px solid red; border-collapse: collapse; }
td { background: blue; border: 50px solid blue; padding: 10px; width: 100px; height: 50px; }
</style></head><body>
<table id="t"><tr><td></td></tr></table>
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
        (tb.width - 220.0).abs() < 0.5,
        "cell 胜出缘外半带与表自身 border 同条，表盒应 220（chromium 实证），实际 {}（旧缺陷双计 270）",
        tb.width
    );
    assert!(
        (tb.height - 170.0).abs() < 0.5,
        "行高含整 cell border（70+50=120）+ 自身边框带 25×2 = 170（§17.6.2 边框中心距），实际 {}",
        tb.height
    );
}

/// R5241 原锚形态：表无自身 border，cell border 50 胜出四缘 → 外半整计入
/// （表盒 = 中心距 170 + 外半 25×2 = 220，与表有 border 时同值不同账）。
#[test]
fn r5278_outer_half_intact_when_table_border_zero() {
    let html = r#"<html><head><style>
table { border-collapse: collapse; }
td { border: 50px solid blue; padding: 10px; width: 100px; height: 50px; }
</style></head><body>
<table id="t"><tr><td></td></tr></table>
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
        (tb.width - 220.0).abs() < 0.5,
        "表 border 0 时外半整计入（R5241 原锚），表盒应 220，实际 {}",
        tb.width
    );
    assert!(
        (tb.height - 170.0).abs() < 0.5,
        "表 border 0 时高侧同式：行 120 + 外半 25×2 = 170，实际 {}",
        tb.height
    );
}
