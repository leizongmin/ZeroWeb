//! R5324（CSS2 §9.5 + bugzilla 488725）：行中 float 锚遇**由 nowrap 内容溢出**的
//! 当前行（剩余带宽 < 0 ∧ 行片段 no_wrap）不与该行共行——锚 = 尾随推行后的下一行
//! 槽顶。
//!
//! 修复前：行中锚无条件 = 当前行盒顶（(current_y − line.height).max(0)）——前文
//! nowrap 内容溢出行带宽时 float 仍落在满溢行上
//!（float-nowrap-1：290px nowrap 文本于 80px 容器后接 float，test ≡ notref 同像，
//! mismatch 恒红；chromium 中 float 降行 → test ≠ notref）。
//!
//! 修复后：nowrap ∧ 溢出合取门命中时锚 = `current_y + est`（与本臂尾随推行的
//! est_height 同式）；非 nowrap 行溢出（stack-floats-003：advance 含既有折叠偏差）、
//! 剩余 ≥ 0（floats-006 / nowrap 有余量）均维持共行锚。

use super::super::*;

/// 铸 n 个互异 node id。
fn ids(n: usize) -> Vec<NodeId> {
    let mut doc = zero_dom::Document::new();
    (0..n).map(|_| doc.create_element("span")).collect()
}

fn nowrap() -> Option<RunWhiteSpace> {
    Some(RunWhiteSpace {
        preserve: false,
        break_at_newline: false,
        no_wrap: true,
        hang_trailing: false,
    })
}

/// Ahem 50px 精确字宽 run（1 字/空格 advance 恰 50，行高 32）。
fn ahem_run(text: &str, node_id: NodeId) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id,
        font_size: 50.0,
        line_height: 32.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        is_isolated_bidi: false,
        ws_override: None,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

/// 主判例（float-nowrap-1 镜像）：nowrap 长词流吃满 80px 行带后接 FloatAnchor——
/// 锚 = 下一行槽顶（= 首行高 32），非满溢行顶 0。
#[test]
fn r5324_float_descends_after_saturated_nowrap_line() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(80.0);
    ctx.break_items_into_lines(vec![
        InlineItem::Text(TextRun {
            ws_override: nowrap(),
            ..TextRun::simple(
                "Some text that overflows my parent.".to_string(),
                id[0],
                16.0,
                32.0,
                VerticalAlignValue::Baseline,
            )
        }),
        InlineItem::FloatAnchor(id[1]),
    ]);
    let anchor = ctx.float_anchor_ys.get(&id[1]).copied().expect("float 锚 y 应已记录");
    assert!(
        (anchor - 32.0).abs() < 1e-3,
        "R5324: 满溢行后的 float 锚应降行 = 32（首行高），实际 {anchor}"
    );
    // nowrap 长词流未被断行（单行盒），排除「断行导致伪降行」的假阳性。
    assert_eq!(ctx.lines.len(), 1, "nowrap 前文应单行盒");
}

// 免疫腿（stack-floats-003 镜像）：**非 nowrap** 行 advance 超带宽（该案 120 = 100
// 内容 + 20 既有空白折叠偏差）→ 不得降行，锚 = 行顶 0——降行门为 nowrap ∧ 溢出
// 合取，对此类非 nowrap 溢出免疫。Ahem 50px：2 字 = 100px 恰等于带宽。
#[test]
fn r5324_float_stays_when_non_nowrap_line_fills_band() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(100.0);
    ctx.break_items_into_lines(vec![
        InlineItem::Text(ahem_run("xx", id[0])),
        InlineItem::FloatAnchor(id[1]),
    ]);
    let anchor = ctx.float_anchor_ys.get(&id[1]).copied().expect("float 锚 y 应已记录");
    assert!(
        anchor.abs() < 1e-3,
        "R5324: 非 nowrap 行满带宽时 float 锚应维持共行 = 0，实际 {anchor}"
    );
}

// 合取门另一腿：nowrap 行但**有余量**（800px 容器单字 X）→ 不降行，锚 = 行顶 0。
#[test]
fn r5324_nowrap_line_with_room_keeps_anchor() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_items_into_lines(vec![
        InlineItem::Text(TextRun {
            ws_override: nowrap(),
            ..ahem_run("X", id[0])
        }),
        InlineItem::FloatAnchor(id[1]),
    ]);
    let anchor = ctx.float_anchor_ys.get(&id[1]).copied().expect("float 锚 y 应已记录");
    assert!(
        anchor.abs() < 1e-3,
        "R5324: nowrap 行有余量时 float 锚应维持共行 = 0，实际 {anchor}"
    );
}

// 控制腿（floats-006 镜像）：前文短词留有剩余带宽——float 与该行共行，锚 = 行顶 0。
#[test]
fn r5324_float_stays_on_line_when_room_remains() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_items_into_lines(vec![
        InlineItem::Text(ahem_run("X", id[0])),
        InlineItem::FloatAnchor(id[1]),
    ]);
    let anchor = ctx.float_anchor_ys.get(&id[1]).copied().expect("float 锚 y 应已记录");
    assert!(
        anchor.abs() < 1e-3,
        "R5324: 有剩余带宽时 float 锚应维持共行 = 0，实际 {anchor}"
    );
}

// 控制腿（float-nowrap-1-notref 镜像）：float 先行（行首空行）——锚 = current_y = 0，
// 空行分支不受降行判据影响。
#[test]
fn r5324_leading_float_keeps_line_top_anchor() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(80.0);
    ctx.break_items_into_lines(vec![
        InlineItem::FloatAnchor(id[1]),
        InlineItem::Text(TextRun {
            ws_override: nowrap(),
            ..TextRun::simple(
                "Some text that overflows my parent.".to_string(),
                id[0],
                16.0,
                32.0,
                VerticalAlignValue::Baseline,
            )
        }),
    ]);
    let anchor = ctx.float_anchor_ys.get(&id[1]).copied().expect("float 锚 y 应已记录");
    assert!(anchor.abs() < 1e-3, "R5324: 行首 float 锚应 = 行顶 0，实际 {anchor}");
}
