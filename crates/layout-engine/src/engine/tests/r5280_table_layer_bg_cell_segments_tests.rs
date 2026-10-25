//! R5280 回归：separated 表层背景不覆盖 border-spacing gap（CSS2 §17.5.1）。
//!
//! table-backgrounds-bs-colgroup-001 / bs-column-001（✗10.04 / ✗3.24）：
//! col/colgroup 层背景按列跨整条连续画，spacing gap 被列背景覆盖（应露表底层）；
//! 且 image 平铺相位锚在 content box 顶（应锚 cell 并集包围盒顶 = 首行 cell 顶，
//! 含纵向 perimeter spacing）。bs-rowgroup-001（✗1.45）：row-group 层背景同病。
//! 机制：列条目带 cell 并集包围盒定位区 + 逐 cell 分段裁剪；行组/行层背景写
//! `bg_cell_segments`（相对盒自身 border-box，paint 期裁剪）。

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

/// col 层背景：定位区 = cell 并集包围盒（§17.5.1 列层区域），分段 = 逐 cell
/// border-box（spacing gap 不在段内）。geometry：spacing 10×20、cell 100×30、
/// 周界 spacing 全侧 → col1 跨 x 10..110；行带 y 20..50 / 70..100。
#[test]
fn r5280_col_bg_origin_is_cell_union_bbox_and_segments_cut_at_gaps() {
    let html = r#"<html><head><style>
#t { border-spacing: 10px 20px; }
td { width: 100px; height: 30px; }
col#k { background-color: red; }
</style></head><body>
<table id="t"><colgroup><col id="k" /></colgroup>
<tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table = doc.get_element_by_id("t").expect("table 应存在");
    let tb = find_box(&result.root, table).expect("table 盒应存在");
    let col = doc.get_element_by_id("k").expect("col 应存在");
    let entries = &tb.table_col_backgrounds;
    assert_eq!(entries.len(), 1, "仅 col#k 一条列背景条目");
    let e = &entries[0];
    assert_eq!(e.node_id, col, "条目指向 col#k");
    assert!(
        (e.origin_x - 10.0).abs() < 0.5 && (e.origin_w - 100.0).abs() < 0.5,
        "定位区 x = 列跨 10..110（含横向周界 spacing），实际 x={} w={}",
        e.origin_x,
        e.origin_w
    );
    assert!(
        (e.origin_y - 20.0).abs() < 0.5 && (e.origin_h - 80.0).abs() < 0.5,
        "定位区 y = cell 并集包围盒 20..100（含纵向周界 spacing，image 相位锚此），实际 y={} h={}",
        e.origin_y,
        e.origin_h
    );
    let segs = &e.segments;
    assert_eq!(segs.len(), 2, "两行各一段（行间 gap 不在段内）");
    let expect = [(10.0f32, 20.0, 100.0, 30.0), (10.0, 70.0, 100.0, 30.0)];
    for (i, (ex, ey, ew, eh)) in expect.iter().enumerate() {
        let (sx, sy, sw, sh) = segs[i];
        assert!(
            (sx - ex).abs() < 0.5 && (sy - ey).abs() < 0.5 && (sw - ew).abs() < 0.5 && (sh - eh).abs() < 0.5,
            "段 {i} 应 ({ex},{ey},{ew},{eh})，实际 ({sx},{sy},{sw},{sh})",
        );
    }
}

/// row-group 层背景：`bg_cell_segments` = 该组各 cell border-box（相对行组盒
/// 自身 border-box 原点），paint 期只填 cell 区域（行/列 gap 露表底层）。
#[test]
fn r5280_rowgroup_bg_segments_are_cell_rects_in_group_frame() {
    let html = r#"<html><head><style>
#t { border-spacing: 10px 20px; }
td { width: 100px; height: 30px; }
tbody#g { background-color: blue; }
</style></head><body>
<table id="t"><tbody id="g"><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let g = doc.get_element_by_id("g").expect("tbody 应存在");
    let rgb = find_box(&result.root, g).expect("tbody 盒应存在");
    let segs = &rgb.bg_cell_segments;
    assert_eq!(segs.len(), 4, "2 行 × 2 cell 各一段");
    let mut got = segs.clone();
    got.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0.partial_cmp(&b.0).unwrap()));
    // cell 在行帧内 x 从 0 起（周界 spacing 记在行组盒 x 上，组帧不再含），
    // 列间 gap 10 记在 cell x；行盒在组内 y = 0 / 50（30 高 + 20 gap）
    let expect = [
        (0.0f32, 0.0, 100.0, 30.0),
        (110.0, 0.0, 100.0, 30.0),
        (0.0, 50.0, 100.0, 30.0),
        (110.0, 50.0, 100.0, 30.0),
    ];
    for (i, (ex, ey, ew, eh)) in expect.iter().enumerate() {
        let (sx, sy, sw, sh) = got[i];
        assert!(
            (sx - ex).abs() < 0.5 && (sy - ey).abs() < 0.5 && (sw - ew).abs() < 0.5 && (sh - eh).abs() < 0.5,
            "段 {i} 应 ({ex},{ey},{ew},{eh})，实际 ({sx},{sy},{sw},{sh})",
        );
    }
}
