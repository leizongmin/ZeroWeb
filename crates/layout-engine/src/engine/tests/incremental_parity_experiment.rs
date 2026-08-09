//! 实验：compute_incremental（仅样式/属性变更）输出与全量 compute 的一致性。
//!
//! M3-S9 增量渲染前置验证——决定 render_with_dom_mutations 能否对属性类
//! mutation 走增量布局（compute_incremental），还是必须退化为全量。
//!
//! 结论（2026-08-09）：
//! - **文本/内容变更可增量**：`text_change_incremental_matches_full` 证明增量 == 全量
//!   （measure 回调消费新 styles，经 mark_dirty 重算）。
//! - **样式属性变更不可增量**：`compute_incremental` 的 taffy 树样式是树构建时快照的
//!   （mark_dirty 只重算布局，不更新 taffy style）——增量保留旧值
//!   （`style_change_incremental_keeps_old_taffy_style_boundary` 文档化该边界）。
//!   taffy style 单节点更新为 M3-S9 后续专项。

use zero_dom::parse_html;
use zero_style_system::StyleSystem;

use crate::dirty::LayoutDirtyTracker;
use crate::engine::LayoutEngine;

/// 样式变更（改 #a height）后的已知边界：增量布局**保留 taffy 快照旧值**（与全量
/// 不一致）。这决定 M3-S9 增量布局的适用边界——样式属性变更须全量（taffy style
/// 单节点更新为后续专项）。本测试文档化该边界（断言旧值保留，不断言一致性）。
#[test]
fn style_change_incremental_keeps_old_taffy_style_boundary() {
    let html = r#"<html><body><div id="a" style="width:100px;height:50px;background:red">A</div><div id="b" style="width:200px;height:100px;background:blue">B</div><div id="c">C</div></body></html>"#;
    let mut doc = parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles_v1 = sys.compute_styles(&doc, &[]);

    // 1. 建立缓存（v1 结构 + v1 样式）
    let mut incr_engine = LayoutEngine::new(800.0, 600.0);
    incr_engine.compute(&doc, &styles_v1);

    // 2. 改 #a 的 height（仅属性变更，结构不变）
    let a = doc.query_selector(doc.root(), "#a").expect("#a");
    doc.set_attribute(a, "style", "width:100px;height:120px;background:red");
    let styles_v2 = sys.compute_styles(&doc, &[]);

    // 3. 增量布局
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(a);
    let (incr_result, stats) =
        incr_engine.compute_incremental(&doc, &styles_v2, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc, "should be incremental");
    // 样式已更新（120）但增量布局保留 taffy 快照旧值（50）——已知边界。
    assert_eq!(
        styles_v2.get(&a).map(|s| s.height.clone()),
        Some(zero_css_parser::values::LengthValue::Px(120.0))
    );
    assert!(
        incr_result.snapshot().contains("size=(100.00,50.00)"),
        "incremental keeps old taffy style (boundary): {}",
        incr_result.snapshot()
    );
}

/// 文本内容变更（SetText 语义）后：增量 vs 全量一致性。
#[test]
fn text_change_incremental_matches_full() {
    let html = r#"<html><body><div id="a" style="width:100px">short</div><div id="b" style="width:50px">B</div></body></html>"#;
    let mut doc = parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles_v1 = sys.compute_styles(&doc, &[]);

    let mut incr_engine = LayoutEngine::new(800.0, 600.0);
    incr_engine.compute(&doc, &styles_v1);

    let a = doc.query_selector(doc.root(), "#a").expect("#a");
    doc.set_text_content(a, "a much longer text that should wrap differently");
    let styles_v2 = sys.compute_styles(&doc, &[]);

    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(a);
    let (incr_result, stats) =
        incr_engine.compute_incremental(&doc, &styles_v2, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc, "should be incremental");

    let mut full_engine = LayoutEngine::new(800.0, 600.0);
    let full_result = full_engine.compute(&doc, &styles_v2);

    assert_eq!(
        full_result.snapshot(),
        incr_result.snapshot(),
        "incremental layout must match full layout after text change"
    );
}
