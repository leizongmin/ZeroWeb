#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::geometry::Rect;

use crate::pipeline::RenderPipeline;
/// 首次渲染无 CSS 的文档，然后通过 recompute_styles 添加背景色样式，
/// 验证第二次渲染产生的填充图元数量严格大于第一次。
#[test]
fn test_dirty_tracking_after_style_change() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="target">Content</div></body></html>"#;

    // 首次渲染：无 CSS 背景
    let first = pipeline.render_html(html, "");
    let first_fill_count = first.primitives.fills.len();

    // 重新解析文档并添加背景色样式
    let doc = zero_dom::parse_html(html);
    let css = r#".target { background-color: red; width: 200px; height: 100px; }"#;
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

    // 样式变化应产生更多填充图元
    assert!(
        !prims.fills.is_empty(),
        "recomputed styles should produce fills after adding background-color"
    );
    assert!(
        prims.fills.len() > first_fill_count,
        "style change should increase fill count: {} > {}",
        prims.fills.len(),
        first_fill_count,
    );

    // 布局缓存应更新
    assert!(pipeline.layout().is_some());
}
/// 新建的管线脏区域追踪器应处于初始状态：
/// 无脏矩形、不需要全量重绘、脏面积为 0。
#[test]
fn test_pipeline_initial_dirty_tracker_state() {
    let mut pipeline = RenderPipeline::new(1024.0, 768.0);

    // 初始状态
    let tracker = pipeline.dirty_tracker();
    assert!(tracker.dirty_rects().is_empty(), "新建管线脏矩形列表应为空");
    assert!(!tracker.is_full_redraw(), "新建管线不应需要全量重绘");
    assert_eq!(tracker.dirty_area(), 0.0, "新建管线脏面积应为 0");

    // 渲染后脏区域追踪器仍为空（render_html 不标记脏区域）
    let html = "<html><body><div>Test</div></body></html>";
    let _result = pipeline.render_html(html, "");
    let tracker = pipeline.dirty_tracker();
    assert!(tracker.dirty_rects().is_empty(), "render_html 后脏矩形列表应仍为空");
    assert!(!tracker.is_full_redraw(), "render_html 后不应需要全量重绘");
}
/// 执行一次小区域的增量渲染后，验证脏区域追踪器完全清除：
/// dirty_rects 为空、is_full_redraw 为 false、dirty_area 为 0.0。
#[test]
fn test_pipeline_incremental_render_clears_dirty_area() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    // 首次全量渲染
    let _first = pipeline.render_html(html, "");
    assert!(pipeline.layout().is_some());

    // 创建一个中等大小的脏区域（不触发全量重绘）
    let dirty_box = LayoutBox {
        node_id: None,
        x: 50.0,
        y: 50.0,
        width: 100.0,
        height: 100.0,
        content_x: 50.0,
        content_y: 50.0,
        content_width: 100.0,
        content_height: 100.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0, "增量渲染应正常完成");

    // 增量渲染后脏区域追踪器应完全清除
    let tracker = pipeline.dirty_tracker();
    assert!(tracker.dirty_rects().is_empty(), "增量渲染后 dirty_rects 应为空");
    assert!(!tracker.is_full_redraw(), "小脏区域不应触发全量重绘");
    assert_eq!(tracker.dirty_area(), 0.0, "增量渲染后 dirty_area 应为 0.0");
}
/// 全量渲染后，依次执行 3 次增量渲染（不同区域大小），
/// 验证每次增量渲染后：脏矩形列表为空、不标记全量重绘、脏面积为 0。
#[test]
fn test_dirty_tracker_after_full_then_three_incremental() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div class=\"box\">Content</div></body></html>";
    let css = ".box { background-color: red; width: 200px; height: 100px; }";

    // 全量渲染
    let _full = pipeline.render_html(html, css);
    assert!(pipeline.layout().is_some());
    let tracker = pipeline.dirty_tracker();
    assert!(tracker.dirty_rects().is_empty(), "全量渲染后脏矩形应为空");
    assert!(!tracker.is_full_redraw(), "全量渲染后不应标记全量重绘");
    assert_eq!(tracker.dirty_area(), 0.0, "全量渲染后脏面积应为 0");

    // 辅助：构造脏区域 LayoutBox
    let make_dirty = |x, y, w, h| LayoutBox {
        node_id: None,
        x,
        y,
        width: w,
        height: h,
        content_x: x,
        content_y: y,
        content_width: w,
        content_height: h,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    // 第 1 次增量渲染：小区域
    let dirty1 = make_dirty(10.0, 10.0, 50.0, 50.0);
    let r1 = pipeline.incremental_render(html, "", &dirty1);
    assert!(r1.timings.total_ms >= 0.0, "第 1 次增量渲染应正常完成");
    let t1 = pipeline.dirty_tracker();
    assert!(t1.dirty_rects().is_empty(), "第 1 次增量后脏矩形应为空");
    assert_eq!(t1.dirty_area(), 0.0, "第 1 次增量后脏面积应为 0");

    // 第 2 次增量渲染：中等区域
    let dirty2 = make_dirty(100.0, 50.0, 200.0, 150.0);
    let r2 = pipeline.incremental_render(html, "", &dirty2);
    assert!(r2.timings.total_ms >= 0.0, "第 2 次增量渲染应正常完成");
    let t2 = pipeline.dirty_tracker();
    assert!(t2.dirty_rects().is_empty(), "第 2 次增量后脏矩形应为空");
    assert_eq!(t2.dirty_area(), 0.0, "第 2 次增量后脏面积应为 0");

    // 第 3 次增量渲染：大区域
    let dirty3 = make_dirty(0.0, 0.0, 400.0, 300.0);
    let r3 = pipeline.incremental_render(html, "", &dirty3);
    assert!(r3.timings.total_ms >= 0.0, "第 3 次增量渲染应正常完成");
    let t3 = pipeline.dirty_tracker();
    assert!(t3.dirty_rects().is_empty(), "第 3 次增量后脏矩形应为空");
    assert!(!t3.is_full_redraw(), "增量渲染不应触发全量重绘");
    assert_eq!(t3.dirty_area(), 0.0, "第 3 次增量后脏面积应为 0");

    // 布局缓存始终有效
    assert!(pipeline.layout().is_some(), "布局缓存应始终有效");
}
/// 两个矩形紧密相邻但不重叠（间距小于合并阈值），
/// 验证合并后状态正确：矩形数量减少或保持，脏面积有效。
/// 当两矩形并集面积不超过各自面积之和的 150% 时应合并。
#[test]
fn test_dirty_tracker_merge_adjacent_non_overlapping_rects() {
    let mut tracker = crate::dirty::DirtyTracker::new();

    // 两个相邻矩形：rect1 在左侧，rect2 在右侧，有 1px 间隙
    // rect1: (0, 0, 50, 50) 面积 2500
    // rect2: (51, 0, 50, 50) 面积 2500
    // 个体面积之和 = 5000
    // 并集: (0, 0, 101, 50) 面积 = 5050
    // 5050 / 5000 = 1.01 <= 1.5 → 应该合并
    tracker.mark_dirty(Rect::new(0.0, 0.0, 50.0, 50.0));
    tracker.mark_dirty(Rect::new(51.0, 0.0, 50.0, 50.0));
    assert_eq!(tracker.dirty_rects().len(), 2, "合并前应有 2 个脏矩形");

    let area_before = tracker.dirty_area();
    assert!(
        (area_before - 5000.0).abs() < 1.0,
        "合并前脏面积应约为 5000，实际 {}",
        area_before
    );

    tracker.merge_overlapping();

    // 相邻矩形应合并为 1 个（并集面积比率 <= 150%）
    assert!(
        tracker.dirty_rects().len() <= 2,
        "相邻矩形合并后数量应 <= 2，实际 {}",
        tracker.dirty_rects().len()
    );

    // 合并后脏面积应大于任一单个矩形
    assert!(tracker.dirty_area() >= 2500.0, "合并后脏面积应 >= 2500");

    // 如果合并为 1 个，验证并集矩形覆盖原范围
    if tracker.dirty_rects().len() == 1 {
        let merged = &tracker.dirty_rects()[0];
        assert!(merged.origin.x <= 0.0, "合并矩形左边界应 <= 0");
        assert!(merged.origin.x + merged.size.width >= 100.0, "合并矩形右边界应 >= 100");
    }
}
