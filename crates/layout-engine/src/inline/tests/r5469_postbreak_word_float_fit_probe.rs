//! R5469：fit 门断行开出的新行首词放不下 float 缩短带 → 行位下移（CSS2 §9.5 rule 8）锚。
//!
//! R5240 的行首词下绕只在 loop-head 空行臂生效；fit 门断行 fall-through 开出的新行
//! 不经 loop-head 重评——新带放不下本词时原地强放溢出。案驱动：
//! css/CSS2/css1/c5525-fltmrgn-000 test3（img float 左 45×45、margin-top/left 30/30、
//! p 90px、Ahem 15px：float 带底 y=75，y=45 起每行带仅剩 15px；第 4 词 "xx"（30px）
//! 经 fit 门 fall-through 在 15px 带上强放，词 6/7 挤同行 → 6 行 h=90；
//! chromium 行位下绕到带底 y=75 满宽整行 → 5 行盒 h=105）。
//!
//! kill-switch 与 loop-head R5240 共用（`ZW_FIRST_WORD_FLOAT_FIT=0`）；反证不由单测
//! 承担：flag 走 selected 快照路径，进程内 set_var 不可靠（R3858 教训）——由 corpus
//! OFF 轮覆盖。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue;

fn ahem_text(text: &str) -> TextRun {
    let node = zero_dom::Document::new().create_text_node("");
    let mut run = TextRun::simple(text.to_string(), node, 15.0, 15.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run
}

/// c5525 test3 形态：容器 90px、float 排除带 (0,0,75,75)（margin-box 口径）、词
/// x/x/x/xx/x/x/x。L1-3 各容 1 词于 15px 右带；L4 "xx" 经 fit 门 fall-through 开行，
/// 新带 15px < 30px → 下绕到带底 y=75 满宽容纳 xx/x/x（3 词 90px 恰满），L5 容尾词。
#[test]
fn postbreak_first_word_shifted_below_float_band() {
    let mut ctx = InlineFormattingContext::new(90.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 75.0,
        width: 75.0,
        x: 0.0,
        is_left: true,
    }]);
    ctx.break_into_lines(vec![ahem_text("x x x xx x x x")]);

    assert_eq!(
        ctx.lines.len(),
        5,
        "应 5 行盒（3 行带内 + 下绕满宽 + 尾词），实际 {}",
        ctx.lines.len()
    );
    assert!(
        (ctx.lines[3].y - 75.0).abs() < 0.5,
        "第 4 行应下绕到 float 带底 y=75，实际 {}",
        ctx.lines[3].y
    );
    assert_eq!(ctx.lines[3].runs.len(), 3, "下绕行应满宽容纳 xx/x/x 三词");
    assert!(
        (ctx.lines[3].runs[0].x - 0.0).abs() < 0.5,
        "下绕行首 x 应回到带左缘 0，实际 {}",
        ctx.lines[3].runs[0].x
    );
    assert!(
        (ctx.lines[4].y - 90.0).abs() < 0.5,
        "尾词行应紧随下绕行 y=90，实际 {}",
        ctx.lines[4].y
    );
}

/// 守卫：词放得下新带（15px 词 = 15px 带）时不下绕——fit 门逐行开新行，各行停留
/// 带内原位（x=75），防「放得下也被误下移」。
#[test]
fn fitting_word_keeps_in_band_placement() {
    let mut ctx = InlineFormattingContext::new(90.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 75.0,
        width: 75.0,
        x: 0.0,
        is_left: true,
    }]);
    ctx.break_into_lines(vec![ahem_text("x x x x x x")]);

    assert_eq!(ctx.lines.len(), 6, "6 词各占 1 行（带内 5 行 + 带底后 1 行）");
    for (i, line) in ctx.lines.iter().enumerate().take(5) {
        assert!(
            (line.y - (i as f32 * 15.0)).abs() < 0.5,
            "第 {} 行应停留带内 y={}，实际 {}",
            i + 1,
            i as f32 * 15.0,
            line.y
        );
        assert!(
            (line.runs[0].x - 75.0).abs() < 0.5,
            "第 {} 行词应停留右带 x=75，实际 {}",
            i + 1,
            line.runs[0].x
        );
    }
}
