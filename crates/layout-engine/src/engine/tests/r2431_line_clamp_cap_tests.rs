//! R2431 line-clamp slice 1：IFC 行夹到 N（CSS Overflow 4）回归测试。
//!
//! `InlineFormattingContext::layout()` 读容器 `line-clamp: Count(n)` → `apply_line_clamp_cap(n)`：
//! `break_items_into_lines` 后 truncate `self.lines` 到 N + 置 `clamped`。box 高度由下游从 lines 推。
//! 仅 horizontal-tb；kill-switch `ZW_LINE_CLAMP=0`。driving：css-overflow/line-clamp/line-clamp-001。
//! 承接 line-clamp-rfc-2026-08-02.md slice 1。

use crate::inline::InlineFormattingContext;
use crate::inline::TextRun;
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

/// 构造 N 个 TextRun（每行一个短词），窄容器强制每词一行 → lines.len() == n_runs。
fn ctx_with_lines(n_runs: usize) -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(40.0); // 窄容器：每词独占一行
    let runs: Vec<_> = (0..n_runs)
        .map(|i| TextRun {
            text: format!("word{i}"),
            node_id: NodeId::default(),
            font_size: 16.0,
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
            is_ahem_font: false,
            font_id: None,
        })
        .collect();
    ctx.break_into_lines(runs);
    ctx
}

/// 5 行内容 + apply_line_clamp_cap(4) → 4 行 + clamped=true。
#[test]
fn test_line_clamp_caps_lines() {
    let mut ctx = ctx_with_lines(5);
    assert_eq!(ctx.lines.len(), 5, "前置：5 词窄容器 → 5 行");
    ctx.apply_line_clamp_cap(4);
    assert_eq!(ctx.lines.len(), 4, "clamp 4：夹到 4 行");
    assert!(ctx.clamped, "clamped=true（第 4 行后有更多内容）");
}

/// 不调 apply_line_clamp_cap → 不夹行，clamped=false。
#[test]
fn test_no_line_clamp_no_cap() {
    let ctx = ctx_with_lines(5);
    assert_eq!(ctx.lines.len(), 5, "无 clamp：5 行全保留");
    assert!(!ctx.clamped, "clamped=false（未截断）");
}

/// apply_line_clamp_cap(10) 但仅 2 行 → 不截断（clamped=false，行数=2）。
#[test]
fn test_line_clamp_content_fewer_than_n() {
    let mut ctx = ctx_with_lines(2);
    ctx.apply_line_clamp_cap(10);
    assert_eq!(ctx.lines.len(), 2, "内容 2 行 < clamp 10：不夹");
    assert!(!ctx.clamped, "clamped=false（内容不足 N）");
}

/// apply_line_clamp_cap(0) → 不截断（n=0 守卫）。
#[test]
fn test_line_clamp_zero_no_cap() {
    let mut ctx = ctx_with_lines(5);
    ctx.apply_line_clamp_cap(0);
    assert_eq!(ctx.lines.len(), 5, "clamp 0：不夹");
    assert!(!ctx.clamped, "clamped=false（n=0）");
}
