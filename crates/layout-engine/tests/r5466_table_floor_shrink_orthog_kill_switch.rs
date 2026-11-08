//! R5466 回退锚：三片 kill-switch 全 OFF 回退旧径（独立测试二进制 = 独立进程，
//! R3858 教训——LazyLock 进程级单读，与主判例同进程会互翻）。
//!
//! 三开关 `ZW_TABLE_SPEC_FLOOR_MINC=0`（片 1 floor 回落 max-content 旧径）、
//! `ZW_TABLE_SHRINK_FREEZE=0`（片 2 回纯比例压缩）与
//! `ZW_CELL_ORTHOG_INTRINSIC=0`（片 3 回水平文本 advance 旧径）全 OFF 时，
//! 驱动案 ref 形态侧列被压缩摊薄 < 52（R5466 前实症：列 21.7/457.9/24.4 →
//! 片 1+2 后中间态 16.56/470.88/16.56，specified 列均不保）。

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
fn kill_switch_reverts_to_legacy_squash() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；LazyLock 首读 env 前生效
    //（独立进程首读即 OFF，R3858 教训）。
    unsafe { std::env::set_var("ZW_TABLE_SPEC_FLOOR_MINC", "0") };
    unsafe { std::env::set_var("ZW_TABLE_SHRINK_FREEZE", "0") };
    unsafe { std::env::set_var("ZW_CELL_ORTHOG_INTRINSIC", "0") };

    let nums: String = (1..=100)
        .map(|i| format!("{:02} ", if i == 100 { 0 } else { i }))
        .collect();
    let r = compute(
        &format!(
            r#"<html><body><table id="t"><tr><td id="after"><p>Sentence&nbsp;after.</p></td><td id="data">{nums}</td><td id="before"><p>Sentence&nbsp;before.</p></td></tr></table></body></html>"#
        ),
        "body { font-size: 16px; line-height: 1.25; } \
         table#t { border-spacing: 0; padding: 0 84px; width: 672px; } \
         td { padding: 0; vertical-align: top; } \
         td#after, td#before { width: 52px; } \
         p { writing-mode: vertical-rl; margin: 1em 0; } \
         td#data { border: blue solid 3px; display: block; }",
    );
    let row = find_row_n(&r.root, 3).expect("行盒在树");
    let ws: Vec<f32> = row.children.iter().map(|c| c.width).collect();
    assert!(
        ws[0] < 52.0 && ws[2] < 52.0,
        "旧径侧列应被压缩摊薄到 52 以下（旧版本失败证据：21.7/457.9/24.4），实得 {ws:?}"
    );
    assert!(ws[1] > 400.0, "旧径 data 列应保持内容主导 > 400，实得 {ws:?}");
}
