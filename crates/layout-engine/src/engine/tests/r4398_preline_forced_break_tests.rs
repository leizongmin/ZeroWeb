//! R4398（css-text-3 §white-space-phase-1）：pre-line 强制断行 + 断点前可折叠空格剥除。
//!
//! run 级 white-space=pre-line（break_at_newline）的 ` \n` 序列：空格词入行后、
//! 强制断行标记消费时须剥除行尾可折叠空格 fragment（「collapsible spaces immediately
//! preceding a sequent break are removed」）——`XXXXXXXX<i> \n</i>XXXXXXXX` 行宽应恰
//! 8 字（200px @25px Ahem 口径），空格不得计入行宽。

use crate::inline::{InlineFormattingContext, RunWhiteSpace, TextRun};
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

#[test]
fn r4398_preline_forced_break_strips_trailing_collapsible_space() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let ws = RunWhiteSpace {
        preserve: false,
        break_at_newline: true,
        no_wrap: false,
        hang_trailing: false,
    };
    let mk = |text: &str, ws: Option<RunWhiteSpace>| TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 25.0,
        line_height: 25.0,
        vertical_align: VA::Baseline,
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
        ws_override: ws,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    };
    // together 形态：`XXXXXXXX<i> \n</i>XXXXXXXX`——单 run 携带 " \n"。
    let runs = vec![mk("XXXXXXXX", None), mk(" \n", Some(ws)), mk("XXXXXXXX", None)];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 2, "pre-line \\n 强制断行 → 2 行");
    let line1_w: f32 = ctx.lines[0].runs.iter().map(|f| f.width).sum();
    let line2_w: f32 = ctx.lines[1].runs.iter().map(|f| f.width).sum();
    assert!(
        (line1_w - 200.0).abs() < 1.0,
        "行 1 = 8×25 = 200px（断点前可折叠空格剥除），got {line1_w}"
    );
    assert!((line2_w - 200.0).abs() < 1.0, "行 2 = 200px，got {line2_w}");
    // R4399：强制断行标记零宽 fragment（携带 ws_override）入行——inline_metric_storage
    // 据此按文本节点/owner 存 RunWhiteSpace，paint Path B 重收集恢复 pre-line 断行
    // identity（Path B 的 ws_overrides 对 marker 词永不命中的缺口）。
    let marker = ctx.lines[0]
        .runs
        .iter()
        .find(|f| f.text.is_empty() && f.width == 0.0 && f.ws_override.is_some());
    assert!(marker.is_some(), "行 1 应含零宽 ws_override 标记 fragment");
}
