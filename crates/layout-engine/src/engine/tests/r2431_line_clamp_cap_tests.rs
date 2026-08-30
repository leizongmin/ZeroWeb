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
            ws_override: None,
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

/// R3766b：apply_line_clamp_cap(0) → 截断全部（`line-clamp: auto` + `max-height: 0`/
/// <1lh → 0 行可见，css-overflow-4 auto-011/037）。旧 R2431 n=0 守卫随 Auto 语义订正
///（Count(0) 不可解析，n=0 仅来自 Auto 的块尺寸约束路径）。
#[test]
fn test_line_clamp_zero_truncates_all() {
    let mut ctx = ctx_with_lines(5);
    ctx.apply_line_clamp_cap(0);
    assert_eq!(ctx.lines.len(), 0, "clamp 0：截断全部行");
    assert!(ctx.clamped, "clamped=true（n=0 且有内容被截）");
}

/// R3776：line-clamp 仅作用于 block 容器——display:Inline 的容器不裁行
///（css-overflow-4，line-clamp-014：「only affects block containers, not inline boxes」，
/// inline span.clamp 的 line-clamp:4 被旧实现照裁 5→4 行）。
#[test]
fn r3776_inline_container_not_clamped() {
    let html = r#"<html><head><style>
.block { font: 16px / 32px serif; }
.clamp { line-clamp: 4; }
</style></head><body>
<div class="block"><span class="clamp">Line 1<br>Line 2<br>Line 3<br>Line 4<br>Line 5</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    // 找 inline span 容器，驱动 IFC layout：旧行为裁到 4 行，新行为 5 行全保留。
    let mut span_id = None;
    for (id, s) in styles.iter() {
        let is_span = doc.get(*id).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.local_name() == "span",
            _ => false,
        });
        if is_span && matches!(s.display, zero_css_parser::values::DisplayValue::Inline) {
            span_id = Some(*id);
        }
    }
    let span_id = span_id.expect("inline span container");
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, span_id, &styles);
    assert_eq!(ctx.lines.len(), 5, "inline 容器的 line-clamp 不适用：5 行全保留");
    assert!(!ctx.clamped, "inline 容器不置 clamped");
}

/// R3778：inline 包裹层声明的 white-space 经继承作用于其内文本——IFC 逐 run 消费
///（旧实现只读容器样式 → span 上的 pre 丢失，5 行折叠 1 行）。
/// driving: line-clamp-014（.block normal > span.clamp pre > 5 行）。
#[test]
fn r3778_inline_wrapper_white_space_honored() {
    let html = r#"<html><body>
<div style="font: 16px / 32px serif;"><span style="white-space: pre;">Line 1
Line 2
Line 3
Line 4
Line 5</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut span_id = None;
    for id in styles.keys() {
        let is_span = doc.get(*id).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.local_name() == "span",
            _ => false,
        });
        if is_span {
            span_id = Some(*id);
        }
    }
    let span_id = span_id.expect("inline span");
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, span_id, &styles);
    assert_eq!(ctx.lines.len(), 5, "span 的 pre 生效：5 行（非折叠 1 行）");
}
