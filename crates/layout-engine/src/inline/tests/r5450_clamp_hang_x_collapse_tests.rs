//! R5450（R5297 band 域幻影带根因收口）：clamp trim 清零 run 的 x 折叠到剪除点。
//! 清零 run 若保留递进 x，`owner_line_band_extent_map`（(owner, line_top) → 祖先
//! 累计 (min_x, max_end)）仍 union 其 (x, x+0) 延伸 band 域，paint 侧 R639
//! fragment bg 臂（whole_box）按幻影带涂背景 → block-ellipsis-032 hangable span
//! 红底残带；折叠后 band 退化为零宽点。引擎级端到端钉见
//! tests/r5450_clamp_hang_x_collapse.rs；kill-switch 独立进程钉见
//! tests/r5450_clamp_hang_x_collapse_kill_switch.rs。

use crate::inline::{InlineFormattingContext, TextRun};
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

/// 单 TextRun（缺省 16px 常规字体）。
fn run(text: &str) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
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
        is_ahem_font: false,
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

/// pre-wrap + 强制断行 IFC：`"aa \nbb \ncc "` 三行，各行以保留空格片段收尾
///（强制断行使行构成与宽度估计无关，断言确定性来源）。
fn trailing_space_lines() -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true);
    ctx.break_into_lines(vec![run("aa \nbb \ncc ")]);
    ctx
}

/// 多个纯空白 run 清零后 x 全部折叠到首个纯空白 run 起点（kept end）——行 band
/// 上界 max(x+width) 不再越过剪除点（R5297 幻影带根因）。
#[test]
fn cleared_pure_runs_x_collapse_to_kept_end() {
    let mut ctx = trailing_space_lines();
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    // L2 构造双纯空白 run 尾（["bb", " ", "  "]），第二段 x 取自然递进位。
    let space_advance = ctx.advance_of(' ', None, 16.0, false);
    let first_pure_x;
    {
        let line = &mut ctx.lines[1];
        let mut extra = line.runs.last().unwrap().clone();
        extra.x += extra.width;
        extra.text.push(' ');
        extra.width += space_advance;
        first_pure_x = line.runs.last().unwrap().x;
        line.runs.push(extra);
    }
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    let runs = &ctx.lines.last().unwrap().runs;
    let cleared: Vec<_> = runs.iter().filter(|f| f.text.is_empty()).collect();
    assert_eq!(cleared.len(), 2, "双纯空白 run 同臂清零");
    assert!(cleared.iter().all(|f| f.width == 0.0), "清零片段零推进");
    assert!(
        cleared.iter().all(|f| (f.x - first_pure_x).abs() < 0.01),
        "清零 run x 全部折叠到剪除点 {first_pure_x}，实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
    let band_end = runs.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    assert!(
        (band_end - first_pure_x).abs() < 0.01,
        "行 band 上界不越过剪除点（band 域零延伸）：{band_end} vs {first_pure_x}"
    );
}

/// 混合 run 剪尾随空格后，纯空白 run x 折叠到混合 run 的新末端（trim point =
/// 混合 run x + 收缩后 width）。
#[test]
fn cleared_pure_runs_x_collapse_to_mixed_run_new_end() {
    let mut ctx = trailing_space_lines();
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    let space_advance = ctx.advance_of(' ', None, 16.0, false);
    let word_width = ctx.lines[1].runs[0].width;
    assert_eq!(ctx.lines[1].runs[0].text, "bb", "前置：L2 首片段为词");
    {
        let frag = &mut ctx.lines[1].runs[0];
        frag.text.push_str("  ");
        frag.width += 2.0 * space_advance;
    }
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    let runs = &ctx.lines.last().unwrap().runs;
    let word = runs.iter().find(|f| f.text == "bb").expect("bb 保留");
    assert!(
        (word.width - word_width).abs() < 0.01,
        "混合 run width 收回双空格 advance：{} 应回 {word_width}",
        word.width
    );
    let cleared: Vec<_> = runs.iter().filter(|f| f.text.is_empty()).collect();
    assert_eq!(cleared.len(), 1, "纯空白 run 清零");
    let trim_x = word.x + word.width;
    assert!(
        (cleared[0].x - trim_x).abs() < 0.01,
        "清零 run x 折叠到混合 run 新末端 {trim_x}，实得 {}",
        cleared[0].x
    );
}
