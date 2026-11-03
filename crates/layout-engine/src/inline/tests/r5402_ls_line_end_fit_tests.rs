//! R5402（css-text-3 §letter-spacing "Letter spacing is not applied at the start/end
//! of a line"）：fit 门对词尾 letter-spacing 的行宽原谅。
//!
//! 驱动案 css-text letter-spacing-200（✗5.79% → ✓0.00%）：容器 5em（100px）Ahem 20px
//! `letter-spacing: 1em`，行 2「12三456七89」期望三行各恰好 100px（"12三"/"456"/"七89"）。
//! 旧 fit 门 `current_x + lead_gap + word_width > avail` 把词内 count 语义累计的尾随
//! ls（R4231：ls 应用于每个字符单元之后）计入词宽 → 「456」词宽 120 > 100 整词早断/
//! 溢出；修复 fit 门减去词尾一次 ls（`- run.letter_spacing`，词中多字词词尾 ls 不占
//! 行宽——行尾 R4231 后处理本就裁掉该 ls，两半合拢行尾语义）→ 100 ≤ 100 恰好放下。
//! 词中相邻字母间 ls 仍照常推进（R1450 针覆盖），行非末词不受影响。
//!
//! kill-switch `ZW_LS_LINE_END_FIT`（default-on；进程内 set_var 不可靠，R3858 教训
//! ——OFF 轮由 corpus 覆盖，不设单测；OFF 态 letter-spacing-200 回 ✗5.79%）。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue as VA;

/// ★load-bearing：5em 容器「12三456七89」三行各 100px，行 2「456」词尾 ls 原谅后
/// 恰好放下（旧实现 120 溢出）。驱动案 letter-spacing-200 行 2。
#[test]
fn r5402_ls_line_end_fit_forgives_word_trailing_spacing() {
    let mut ctx = InlineFormattingContext::new(100.0);
    let mut run = TextRun::simple("12三456七89".to_string(), NodeId::default(), 20.0, 20.0, VA::Baseline);
    run.is_ahem_font = true;
    run.letter_spacing = 20.0; // 1em
    ctx.break_into_lines(vec![run]);

    assert_eq!(
        ctx.lines.len(),
        3,
        "应恰好三行（12三/456/七89），got {}",
        ctx.lines.len()
    );

    // 行 1：「12」(w=80) + 「三」@80（w=20，行尾裁剪后裸字形）→ 视觉 100px。
    let l0 = &ctx.lines[0];
    assert_eq!(l0.runs.len(), 2, "行 1 应两片段");
    assert!((l0.runs[0].x - 0.0).abs() < 1.0 && l0.runs[0].text == "12");
    assert!((l0.runs[1].x - 80.0).abs() < 1.0 && l0.runs[1].text == "三");

    // 行 2：「456」整词单片段 w=100（=3 字形 + 2 词内 ls，词尾 ls 原谅后 fit 门放行、
    // R4231 行尾后处理裁剪）；旧实现词宽 120 溢出。可用宽 100 内。
    let l1 = &ctx.lines[1];
    assert_eq!(l1.runs.len(), 1, "行 2 应单片段「456」");
    assert_eq!(l1.runs[0].text, "456");
    assert!(
        (l1.runs[0].width - 100.0).abs() < 1.0,
        "「456」应恰好 100px（词尾 ls 不占行宽），实际 {:.1}",
        l1.runs[0].width
    );

    // 行 3：「七」@0（w=40）+「89」@40（w=60）→ 视觉 100px。
    let l2 = &ctx.lines[2];
    assert_eq!(l2.runs.len(), 2, "行 3 应两片段");
    assert!((l2.runs[0].x - 0.0).abs() < 1.0 && l2.runs[0].text == "七");
    assert!((l2.runs[1].x - 40.0).abs() < 1.0 && l2.runs[1].text == "89");

    // 三行依次排布（line-height 1 → 步进 20px）。
    assert!((l0.y - 0.0).abs() < 1.0 && (l1.y - 20.0).abs() < 1.0 && (l2.y - 40.0).abs() < 1.0);
}

/// 对照锚：行首单词 fit 语义不变——「12」(w=80) 在 100px 内照常放下；ls=0 时
/// 原谅项为 0（`-0.0` no-op），无 ls 文档流不受 R5402 影响。
#[test]
fn r5402_ls_zero_and_short_word_unaffected() {
    // ls=0：同容器无 ls 间距，字符 20px 连排——5 字/行。
    let mut ctx = InlineFormattingContext::new(100.0);
    let run = TextRun::simple(
        "一二三四五六七八".to_string(),
        NodeId::default(),
        20.0,
        20.0,
        VA::Baseline,
    );
    ctx.break_into_lines(vec![run]);
    assert_eq!(ctx.lines.len(), 2, "ls=0 应两行（5+3 字），got {}", ctx.lines.len());
    let l0_width: f32 = ctx.lines[0].runs.iter().map(|r| r.width).sum();
    assert!(
        (l0_width - 100.0).abs() < 1.0,
        "行 1 五字共 100px，实际 {:.1}",
        l0_width
    );
}
