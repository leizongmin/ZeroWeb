//! R5309：`text-wrap: nowrap` layout 期换行抑制回归钉（CSS Text 4 §text-wrap）。
//!
//! 根因：run 级 `ws_override` 在 break_lines 处遮蔽容器级 IFC `no_wrap` 旗标
//!（`Some` 优先），`white-space: normal` 的 run 覆盖 `Some(default)`（no_wrap=false）
//! 把容器级 nowrap 抑制无效化——text-wrap:nowrap 仅在容器级（compute_final
//! 主配置与 measure resolver）设置时，带样式文本 run 仍照常软换行。修复：
//! collect_items `run_white_space` 改读整个 ComputedStyle，把 `text-wrap: nowrap`
//! 折入 run 级 no_wrap（text-wrap 与 white-space 均 inherited 且独立生效，任一
//! nowrap 即禁软换行）。
//!
//! 案形（text-wrap-nowrap-001 原页）：`width: 20ch` 正确解析 160px（cascade
//! resolve_computed_style ch→Px），nowrap 页单行溢出（高 18），mis-ref（同宽无
//! nowrap）3 行换行（高 54）——修复前双页同形 3 行 0/480000px 全同 → mismatch-ref
//! 误判红。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5309-text-wrap-nowrap-2026-10-28.md
//!
//! https://drafts.csswg.org/css-text-4/#text-wrap

use zero_engine::RenderPipeline;

const TEST: &str = r#"<!DOCTYPE html>
<style>
div {
  width: 20ch;
  text-wrap: nowrap;
}
</style>
<div>Pass if this is a single line of text without breaks.</div>"#;

const MIS_REF: &str = r#"<!DOCTYPE html>
<style>
div {
  width: 20ch;
}
</style>
<div>Pass if this is a single line of text without breaks.</div>"#;

/// 快照中最后一个元素盒（两页形态下 = 目标 div）的 `size=(w,h)` 行。
fn last_box_size_line(snapshot: &str) -> String {
    snapshot
        .lines()
        .rev()
        .find(|l| l.contains("NodeId("))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// text-wrap:nowrap 页：文本单行（盒高 18 = 1 行），不再按 160px 宽换行 3 行。
#[test]
fn nowrap_page_renders_single_line() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(TEST, "");
    let line = last_box_size_line(&result.layout.snapshot());
    assert!(
        line.contains("size=(160.00,18.00)"),
        "text-wrap:nowrap 须单行（盒 160×18）；实际：{line}"
    );
}

/// mis-ref 页（同宽无 nowrap）：维持 3 行换行形（盒 160×54）——mismatch 语义成立。
#[test]
fn mis_ref_page_still_wraps() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(MIS_REF, "");
    let line = last_box_size_line(&result.layout.snapshot());
    assert!(
        line.contains("size=(160.00,54.00)"),
        "无 nowrap 的 20ch 页须维持换行形（盒 160×54）；实际：{line}"
    );
}
