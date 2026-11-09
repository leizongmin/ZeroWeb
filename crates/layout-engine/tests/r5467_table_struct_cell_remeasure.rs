//! R5467 主判例：列压缩重测的 cell 判定结构臂——display 被作者块化的 td 仍参与
//! R4298 按最终列宽重测内容高。
//!
//! 病理（sizing-orthog-htb-in-vrl-007 ref 臂残余 3.98%，R5467 双臂 LAYOUT_DUMP +
//! chrome oracle 归因缺陷 1）：td#data `display: block` 仍是 grid 成员（build_row
//! 按行盒非 inline 元素子收集，CSS2 §17.2.1），但 R4298 remeasure 的 cell 判定只认
//! `ComputedStyle.display == TableCell` → 块化 cell 逃逸重测，内容高停留压缩前
//! taffy 值（taffy 按压缩前宽排 4 行 ch=80 vs 最终列宽 394 换 6 行真值 120；行/表高
//! 86 vs 126）。对照实证：同一页去掉 display:block 即 ch=120/h=126（ZW 与 chrome
//! oracle 同值），gate 逃逸为唯一病因。修复后 driver 3.98%→3.55%（残余 = 缺陷 2
//! htb cell 内 vertical p 测量 w=0 + 缺陷 3 UA p margin 物理/逻辑域，见证据档挂账）。

use zero_layout_engine::types::LayoutBox;
use zero_layout_engine::{LayoutEngine, LayoutResult};

fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 按 DOM id 找盒（深度优先）。
fn find_by_id<'a>(b: &'a LayoutBox, id: &str, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
    if b.node_id
        .is_some_and(|nid| doc.get_attribute(nid, "id").as_deref() == Some(id))
    {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_by_id(c, id, doc) {
            return Some(f);
        }
    }
    None
}

/// 按子 cell 数找行盒。
fn find_row_n(b: &LayoutBox, n_cells: usize) -> Option<&LayoutBox> {
    let cell_count = b.children.iter().filter(|c| c.node_id.is_some()).count();
    if cell_count == n_cells {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_row_n(c, n_cells) {
            return Some(f);
        }
    }
    None
}

#[test]
fn blocked_td_content_height_follows_final_column_width() {
    // 驱动案 ref 形态（真实 ref 页同构，p 无显式 margin 同 UA 域）：块化 td#data
    // 内容高按最终列宽 394 重排换行——6 行 × 20 = 120（+上下边框 6 = 盒高 126），
    // 行高/表高随之 126。旧版本失败证据：残留压缩前 taffy 值 ch=80/h=86，行/表高 86。
    let nums: String = (1..=100)
        .map(|i| format!("{:02} ", if i == 100 { 0 } else { i }))
        .collect();
    let html = format!(
        r#"<html><body><table id="t"><tr><td id="after"><p>Sentence&nbsp;after.</p></td><td id="data">{nums}</td><td id="before"><p>Sentence&nbsp;before.</p></td></tr></table></body></html>"#
    );
    let css = "body { font-size: 16px; line-height: 1.25; } \
         table#t { border-spacing: 0; padding: 0 84px; width: 672px; } \
         td { padding: 0; vertical-align: top; } \
         td#after, td#before { width: 52px; } \
         p { writing-mode: vertical-rl; margin: 1em 0; } \
         td#data { border: blue solid 3px; display: block; }";
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let r = engine.compute(&doc, &styles);

    let data = find_by_id(&r.root, "data", &doc).expect("data cell 在树");
    assert!(
        (data.content_height - 120.0).abs() < 0.5,
        "块化 cell 内容高应按最终列宽 394 重排 = 120，实得 {}",
        data.content_height
    );
    assert!(
        (data.height - 126.0).abs() < 0.5,
        "块化 cell 盒高 = 120 + 边框 6 = 126，实得 {}",
        data.height
    );
    let table = find_by_id(&r.root, "t", &doc).expect("表盒在树");
    assert!(
        (table.height - 126.0).abs() < 0.5,
        "表高应随块化 cell 增高至 126（chrome oracle 同值），实得 {}",
        table.height
    );
}

#[test]
fn blocked_td_wrapping_text_row_height_generic() {
    // 泛化域（非驱动案形态）：窄表强制列压缩 + 块化 td 内包裹性长文本——块化 cell
    // 内容高须按压缩后列宽换行增高（CSS2 §17.5.2.1）。旧版本失败证据：残留 taffy
    // 压缩前度量（单行 20）。
    let r = compute(
        r#"<html><body><table id="t"><tr><td id="a">aaaaaaaaaa bbbb</td><td id="b">x</td></tr></table></body></html>"#,
        "body { font-size: 16px; line-height: 1.25; } \
         table#t { border-spacing: 0; width: 120px; } \
         td { padding: 0; } \
         td#a { display: block; }",
    );
    let row = find_row_n(&r.root, 2).expect("行盒在树");
    let hs: Vec<f32> = row.children.iter().map(|c| c.height).collect();
    assert!(
        hs[0] >= 40.0,
        "块化 td 内容高应按压缩列宽换行 ≥ 2 行（≥40），实得 {hs:?}"
    );
}
