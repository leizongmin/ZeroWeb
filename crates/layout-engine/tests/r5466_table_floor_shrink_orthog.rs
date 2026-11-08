//! R5466 主判例：表布局三片——显式宽 floor 词级 min-content、超宽压缩冻结臂、
//! 正交子块轴固有测宽。
//!
//! 病理（sizing-orthog-htb-in-vrl-007 ref 臂残余 4.27%，R5465 证据 §4 三面归因）：
//! ①R364b 显式宽 cell 的列宽 floor 回落 max-content（指定宽被内容 max-content 越过，
//! c5501 族同根因）；②R4298 超宽压缩纯比例摊薄，specified/floor 列不冻结（chrome：
//! 赤字由 slack 列按比例吸收）；③htb cell 内 vertical-flow 子沿文本 advance 测宽
//!（css-writing-modes-3 §7：块轴 extent = 行高，文本 advance 进物理高）。
//! chrome oracle（/tmp/r5465-oracle 探针组 + R5466 复核）：ref 形态列 52/400/52——
//! 侧列 = max(specified 52, vertical p 块轴 ~20) = 52，data 列吸收赤字 400。
//!
//! 修复：table_types.rs 片 1（floor 语境词级 min-content，`ZW_TABLE_SPEC_FLOOR_MINC`）；
//! table.rs 片 2（col_frozen_floor 快照 + 压缩冻结臂，`ZW_TABLE_SHRINK_FREEZE`）；
//! table_types.rs 片 3（正交子块轴 IFC 测量臂，`ZW_CELL_ORTHOG_INTRINSIC`）。
//! 回退锚见 r5466_table_floor_shrink_orthog_kill_switch.rs（独立进程，R3858 教训）。

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

/// 找「行盒」：宽度 == expected_row_w 且恰有 n 个带 node_id 的 cell 子。
/// x 为相对父内容原点（LAYOUT_DUMP 口径，R5465 取证教训）。
fn find_row(b: &LayoutBox, expected_row_w: f32, n_cells: usize) -> Option<&LayoutBox> {
    let cell_count = b.children.iter().filter(|c| c.node_id.is_some()).count();
    if (b.width - expected_row_w).abs() < 0.5 && cell_count == n_cells {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_row(c, expected_row_w, n_cells) {
            return Some(f);
        }
    }
    None
}

/// 按子 cell 数找行盒（表 auto 宽域行宽随字体漂移，不宜按宽定位）。
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

fn cell_widths(row: &LayoutBox) -> Vec<f32> {
    row.children.iter().map(|c| c.width).collect()
}

fn cell_xs(row: &LayoutBox) -> Vec<f32> {
    row.children.iter().map(|c| c.x).collect()
}

#[test]
fn explicit_floor_is_word_min_content_c5501_guard() {
    // 片 1 + c5501 防护域：td 指定 10em（160）+ 可断行长文本 → 列宽恰 160
    //（floor = max(160, 词 min-content ~70) = 160；旧版本失败证据：floor 回落
    // max-content 整行 ~300+，指定列被内容撑破——c5501 族同根因）。
    let r = compute(
        r#"<html><body><table id="t"><tr><td id="test">some fairly long content words wrap here naturally</td><td id="b">x</td></tr></table></body></html>"#,
        "table#t { border-spacing: 0; width: 504px; } \
         td { padding: 0; } \
         td#test { width: 10em; } td#b { width: 344px; }",
    );
    let row = find_row(&r.root, 504.0, 2).expect("行盒在树");
    assert!(
        (cell_widths(row)[0] - 160.0).abs() < 0.5,
        "td 10em 列应恰 160（floor 词级 min-content 不越过指定宽），实得 {:?}",
        cell_widths(row)
    );
    assert!((cell_widths(row)[1] - 344.0).abs() < 0.5, "邻列 344 应维持");
}

#[test]
fn shrink_freezes_explicit_column() {
    // 片 2：列总和超表宽时显式列冻结、slack 列按 slack 比例吸收赤字。td 80 指定 +
    // 不可断长词（max-content ~300+）→ 总和 ~380 > 200：冻结臂 [80, 120]。
    // 旧版本失败证据：纯比例压缩 ratio=200/380 → [42.1, 157.9]，specified 列被摊薄。
    let r = compute(
        r#"<html><body><table id="t"><tr><td id="s">x</td><td id="w">Supercalifragilisticexpialidociousdontbreakthiswordpleaseokthen</td></tr></table></body></html>"#,
        "table#t { border-spacing: 0; width: 200px; } \
         td { padding: 0; } \
         td#s { width: 80px; }",
    );
    let row = find_row(&r.root, 200.0, 2).expect("行盒在树");
    let ws = cell_widths(row);
    assert!((ws[0] - 80.0).abs() < 0.5, "显式列 80 应冻结不被压缩，实得 {ws:?}");
    assert!(
        (ws[1] - 120.0).abs() < 0.5,
        "slack 列应吸收赤字至 120（列和 = 表宽 200），实得 {ws:?}"
    );
}

#[test]
fn ref_shape_columns_52_400_52() {
    // 片 2+3 联合（驱动案 ref 臂骨架，in-flow 变体）：侧列 vertical p（块轴 ~20 ≤ 52）
    // → floor = max(52, 20) = 52（片 3；旧版本失败证据：nbsp 句文本 advance 128.8/137.6
    // 作宽）；data 列 max-content ~1937 超表宽 → 冻结臂下侧列冻 52、data 吸收赤字恰
    // 400（片 2；旧版本失败证据：纯比例侧列 16.56 < 52，列 16.56/470.88/16.56）。
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
    let row = find_row(&r.root, 504.0, 3).expect("行盒在树");
    let ws = cell_widths(row);
    let xs = cell_xs(row);
    assert!(
        (ws[0] - 52.0).abs() < 0.5 && (ws[2] - 52.0).abs() < 0.5,
        "侧列应冻结 52（vertical p 块轴 ~20 不越过指定宽），实得 {ws:?}"
    );
    assert!(
        (ws[1] - 400.0).abs() < 0.5,
        "data 列应吸收赤字恰 400（52+400+52 = 504），实得 {ws:?}"
    );
    assert!(
        (xs[1] - 52.0).abs() < 0.5 && (xs[2] - 452.0).abs() < 0.5,
        "列起点应 0/52/452，实得 {xs:?}"
    );
}

#[test]
fn vertical_child_measured_along_block_axis() {
    // 片 3：htb 表 auto 列内 vertical-rl p——cell 固有宽 = p 块轴 extent（单列行高 20，
    // 1.25×16）而非文本 advance（旧版本失败证据：nbsp 句 124.8/128.8 文本 run 当宽）。
    let r = compute(
        r#"<html><body><table id="t"><tr><td id="v"><p>Sentence&nbsp;after.</p></td><td id="x">xx</td></tr></table></body></html>"#,
        "body { font-size: 16px; line-height: 1.25; } \
         table#t { border-spacing: 0; } \
         td { padding: 0; } \
         p { writing-mode: vertical-rl; margin: 1em 0; }",
    );
    // 表 auto 宽 shrink-to-fit：vertical cell 20 + "xx" cell ~19 → 行 ~39。
    let row = find_row_n(&r.root, 2).expect("行盒在树");
    let ws = cell_widths(row);
    assert!(
        (ws[0] - 20.0).abs() < 0.5,
        "vertical p cell 固有宽应 = 块轴行高 20，实得 {ws:?}"
    );
    assert!(ws[0] < 40.0, "不得回到文本 advance 旧径（124.8）");
}
