//! R1975 诊断探针：vertical IFC `break_into_lines` 在不同 container_width 下的行为。
//!
//! **背景（TBD2，[`docs/goal/rendering-compat/r109-vertical-native-layout-design.md`] v0.1.3）**：
//! R109 vertical 核心疑虑 = IFC circular-dependency（measure extent 须 container_width，
//! container_width = content_height = extent）。two-phase measure（pass1 unbounded→extent,
//! pass2 extent→container_width）是否可行，取决于 IFC 在 unbounded container_width 下
//! 是否 hang 或产出 inf。
//!
//! **本轮探针发现（break_into_lines 层）**：
//! 1. break_into_lines 对 vertical 文本**任何 container_width（50 / 100000）都产 1 行**（不 wrap）。
//! 2. **不 hang**（R1895 measurement-loop 担忧不适用于 IFC break_into_lines 本身）。
//! 3. LineBox.height = line-height（20），**非文本 inline extent**（10 char×20=200）。
//! 4. vertical 的 container_width-based 换行在 `break_items_into_columns`（由 `layout()` 调用），
//!    **非 break_into_lines**——故本探针不触达它（preliminary，非 definitive TBD2 答案）。
//!
//! **IFC vertical 两字段（mod.rs:95-98）**：`container_width`=inline 深度（content_height，
//! line-break max_depth）；`block_extent`=block 方向宽（content_width，列排布）。两字段分离。
//!
//! definitive TBD2 须 `layout()` 层探针（构造 doc/styles 调 layout + dump break_items_into_
//! columns 的 max_depth 行为）。本文件记录 break_into_lines 基线行为（durable 数据）。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue as VA;

fn ahem_run(text: &str) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 20.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
    }
}

/// vertical IFC unbounded（100000）container_width → 1 line，不 hang，height=line-height（非 extent）。
#[test]
fn r1975_vertical_ifc_unbounded_single_line_no_hang() {
    let run = ahem_run("AAAAAAAAAA"); // 10 chars × 20px = 200px extent
    let mut ctx = InlineFormattingContext::new(100_000.0).with_vertical(true);
    ctx.break_into_lines(vec![run]);
    // 到此 = break_into_lines 终止（不 hang）。R1975 关键发现 1+2。
    eprintln!(
        "R1975 vertical unbounded(100000): {} lines, line[0].height={}",
        ctx.lines.len(),
        ctx.lines.first().map(|l| l.height).unwrap_or(-1.0)
    );
    assert!(!ctx.lines.is_empty(), "应产出至少一行");
    assert_eq!(
        ctx.lines.len(),
        1,
        "break_into_lines 不按 container_width wrap vertical（见 break_items_into_columns）"
    );
    // height = line-height（20），非 extent（200）——extent 在 fragment 内（非 LineBox.height）。
    let h = ctx.lines[0].height;
    assert!((h - 20.0).abs() < 0.5, "LineBox.height 应=line-height=20，实 {}", h);
}

/// vertical IFC narrow（50）container_width → 仍 1 line（break_into_lines 不 wrap）。
/// 对照：证明 break_into_lines 不用 container_width 做 vertical wrap（max_depth 逻辑在别处）。
#[test]
fn r1975_vertical_ifc_narrow_also_single_line() {
    let run = ahem_run("AAAAAAAAAA");
    let mut ctx = InlineFormattingContext::new(50.0).with_vertical(true);
    ctx.break_into_lines(vec![run]);
    eprintln!("R1975 vertical narrow(50): {} lines", ctx.lines.len());
    assert_eq!(
        ctx.lines.len(),
        1,
        "narrow 也 1 line = break_into_lines 不 wrap vertical"
    );
}

/// 对照：horizontal IFC unbounded → 1 line（确认 break_into_lines 基本 single-line 行为）。
#[test]
fn r1975_horizontal_ifc_unbounded_single_line() {
    let run = ahem_run("AAAAAAAAAA");
    let mut ctx = InlineFormattingContext::new(100_000.0);
    ctx.break_into_lines(vec![run]);
    assert_eq!(ctx.lines.len(), 1);
}
