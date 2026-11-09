//! R5467 回退锚：`ZW_TABLE_STRUCT_CELL_REMEASURE=0` 回退旧径（独立测试二进制 =
//! 独立进程，R3858 教训——LazyLock 进程级单读，与主判例同进程会互翻）。
//!
//! 开关 OFF 时块化 td 逃逸列压缩重测（仅 `ComputedStyle.display == TableCell` 判定）：
//! 驱动案 ref 形态 td#data 内容高停留压缩前 taffy 值（旧版本失败证据：ch=80/h=86，
//! 表高 86）。

use zero_layout_engine::LayoutEngine;

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

use zero_layout_engine::types::LayoutBox;

#[test]
fn kill_switch_reverts_blocked_td_to_stale_height() {
    // Rust 2024：set_var 为 unsafe；LazyLock 首读 env 前生效（独立进程首读即 OFF）。
    unsafe { std::env::set_var("ZW_TABLE_STRUCT_CELL_REMEASURE", "0") };

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
        (data.content_height - 80.0).abs() < 0.5,
        "旧径块化 cell 内容高应停留压缩前 taffy 值 80，实得 {}",
        data.content_height
    );
    assert!(
        (data.height - 86.0).abs() < 0.5,
        "旧径块化 cell 盒高应停留 86（80 + 边框 6），实得 {}",
        data.height
    );
}
