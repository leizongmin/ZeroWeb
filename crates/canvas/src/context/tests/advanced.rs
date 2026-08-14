//! Canvas 上下文测试（advanced 批次）。

use super::super::types::*;
use crate::context::*;
use crate::path::{Path2D, PathCommand};
use zero_render_foundation::color::Color;

/// 重叠区域的蓝色应覆盖红色（不透明像素的 source-over 结果）。
#[test]
fn test_composite_source_over_default() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 再绘制蓝色 (5,0)-(15,10)，默认 source-over
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 不重叠区域 (0,0)：仍为红色
    let red_only = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(
        red_only.data[0..4],
        [255, 0, 0, 255],
        "source-over: 非重叠区域应保留红色"
    );

    // 重叠区域 (7,0)：蓝色覆盖红色
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-over: 重叠区域应为蓝色");

    // 蓝色独占区域 (12,0)：蓝色
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(
        blue_only.data[0..4],
        [0, 0, 255, 255],
        "source-over: 蓝色独占区域应为蓝色"
    );
}

/// 测试 destination-over 合成：先绘制蓝色（目标），再以 destination-over 绘制红色（源），
/// 红色应出现在蓝色下方（重叠区域蓝色在上方）。
#[test]
fn test_composite_destination_over() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制蓝色 (0,0)-(10,10) 作为目标
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 destination-over 绘制红色 (5,0)-(15,10) 作为源
    ctx.set_composite_operation(CompositeOperation::DestinationOver);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 蓝色独占区域 (2,0)：蓝色不变
    let blue_only = ctx.get_image_data(2, 0, 1, 1);
    assert_eq!(
        blue_only.data[0..4],
        [0, 0, 255, 255],
        "destination-over: 蓝色独占区域不变"
    );

    // 重叠区域 (7,0)：destination-over 下蓝色（目标）在红色（源）之上
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(
        overlap.data[0..4],
        [0, 0, 255, 255],
        "destination-over: 重叠区域应显示蓝色（目标在上）"
    );

    // 红色独占区域 (12,0)：没有目标，只有源
    let red_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(
        red_only.data[0..4],
        [255, 0, 0, 255],
        "destination-over: 红色独占区域应显示红色"
    );
}

/// 测试 copy 合成：先绘制红色，再设置 copy 模式绘制蓝色，
/// copy 模式下蓝色完全替换已有内容，不受之前内容影响。
#[test]
fn test_composite_copy() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色填充整个画布
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 20.0, 10.0);
    // 使用 copy 合成绘制蓝色矩形
    ctx.set_composite_operation(CompositeOperation::Copy);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // copy 模式下蓝色区域内应为蓝色
    let inside = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(inside.data[0..4], [0, 0, 255, 255], "copy: 绘制区域内应为蓝色");

    // R34xx：copy 的未覆盖区域清除为透明（spec Porter-Duff 全局语义——
    // 2d.composite.uncovered.fill.copy；旧实现保留底）。
    let outside = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(outside.data[0..4], [0, 0, 0, 0], "copy: 未绘制区域应清除为透明");
}

/// 测试 xor 合成：先绘制红色矩形，再绘制重叠的蓝色矩形，
/// xor 模式下重叠区域应变为透明（两个不透明像素的异或结果为空）。
#[test]
fn test_composite_xor() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 xor 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::Xor);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 红色独占区域 (2,0)：sa=0,da=1 → xor 保留目标 = 红色
    let red_only = ctx.get_image_data(2, 0, 1, 1);
    assert_eq!(red_only.data[0..4], [255, 0, 0, 255], "xor: 红色独占区域应保留");

    // 重叠区域 (7,0)：两个不透明像素 xor → 透明
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(overlap.data[0..4], [0, 0, 0, 0], "xor: 重叠区域应为透明");

    // 蓝色独占区域 (12,0)：sa=1,da=0 → xor 保留源 = 蓝色
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(blue_only.data[0..4], [0, 0, 255, 255], "xor: 蓝色独占区域应为蓝色");
}

/// 测试 source-atop 合成：先绘制红色矩形，再绘制重叠的蓝色矩形，
/// source-atop 模式下蓝色只出现在已有红色内容的区域。
#[test]
fn test_composite_source_atop() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 source-atop 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::SourceAtop);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 重叠区域 (7,0)：source-atop → 源色（蓝色）出现在目标存在的区域
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-atop: 重叠区域应为蓝色");

    // 蓝色独占区域 (12,0)：没有目标像素 → source-atop 保留目标 = 透明
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(
        blue_only.data[0..4],
        [0, 0, 0, 0],
        "source-atop: 无目标的源区域应为透明"
    );
}

// ── image_smoothing_enabled 测试 ──

/// 测试 image_smoothing_enabled 默认值为 true。
#[test]
fn test_image_smoothing_enabled_default_is_true() {
    let ctx = CanvasContext::new(100, 100);
    assert!(ctx.image_smoothing_enabled(), "imageSmoothingEnabled 默认应为 true");
}

/// 测试 set/get 往返一致性。
#[test]
fn test_image_smoothing_enabled_set_get_roundtrip() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_image_smoothing_enabled(false);
    assert!(!ctx.image_smoothing_enabled(), "设置为 false 后应返回 false");
    ctx.set_image_smoothing_enabled(true);
    assert!(ctx.image_smoothing_enabled(), "设置为 true 后应返回 true");
}

/// 测试 save/restore 保存并恢复 image_smoothing_enabled 的值。
#[test]
fn test_image_smoothing_enabled_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_image_smoothing_enabled(false);
    ctx.save();
    ctx.set_image_smoothing_enabled(true);
    assert!(ctx.image_smoothing_enabled(), "save 后修改应为 true");
    ctx.restore();
    assert!(!ctx.image_smoothing_enabled(), "restore 后应恢复为 false");
}

/// 测试 save 后修改 image_smoothing_enabled 不影响已保存的状态。
#[test]
fn test_image_smoothing_enabled_modify_after_save_does_not_affect_saved() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_image_smoothing_enabled(true);
    ctx.save();
    ctx.set_image_smoothing_enabled(false);
    assert!(!ctx.image_smoothing_enabled(), "修改后当前值应为 false");
    ctx.restore();
    assert!(ctx.image_smoothing_enabled(), "restore 后应恢复为 save 时的 true");
}

// ── stroke line_cap / line_join 渲染测试 ──

/// 测试描边使用 line_cap Butt 时端点为平头（不超出线段端点）。
/// 验证描边像素仅在线段范围内，不延伸到端点之外。
#[test]
fn test_stroke_line_cap_butt_flat_endpoints() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::BLUE);
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Butt);
    ctx.begin_path();
    ctx.move_to(20.0, 50.0);
    ctx.line_to(80.0, 50.0);
    ctx.stroke();
    // 在线段起点之前不应有像素
    let before_start = ctx.get_image_data(15, 48, 1, 4);
    assert_eq!(before_start.data[0..4], [0, 0, 0, 0], "Butt cap: 线段起点前不应有像素");
    // 在线段终点之后不应有像素
    let after_end = ctx.get_image_data(85, 48, 1, 4);
    assert_eq!(after_end.data[0..4], [0, 0, 0, 0], "Butt cap: 线段终点后不应有像素");
    // 在线段中点应有像素
    let mid = ctx.get_image_data(50, 48, 1, 4);
    assert_eq!(mid.data[0..4], [0, 0, 255, 255], "Butt cap: 线段中点应为蓝色");
}

/// 测试描边使用 line_cap Round 时端点扩展（半圆形）。
/// 验证描边像素在端点处超出线段范围。
#[test]
fn test_stroke_line_cap_round_extended_endpoints() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::RED);
    ctx.set_line_width(10.0);
    ctx.set_line_cap(LineCap::Round);
    ctx.begin_path();
    ctx.move_to(30.0, 50.0);
    ctx.line_to(70.0, 50.0);
    ctx.stroke();
    // R34xx：Round cap 为真圆盘（半径 half_lw=5）——旧断言 (25,46) 距端点 6.4 > 5 在圆外，
    // 方块近似已废弃。断言圆盘内点（(30,45) 距端点恰 5、(26,49) 距 √17≈4.1）。
    let near_start = ctx.get_image_data(26, 49, 1, 1);
    assert_ne!(near_start.data[3], 0, "Round cap: 起点端圆盘内应有像素");
    let near_end = ctx.get_image_data(72, 46, 1, 1);
    assert_ne!(near_end.data[3], 0, "Round cap: 终点端圆盘内应有像素");
}

/// 测试描边使用 line_join Miter 时产生尖角连接。
/// Miter 连接的轮廓顶点应超出两条线段的简单矩形叠加范围。
#[test]
fn test_stroke_line_join_miter_sharp_corners() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_stroke_color(Color::BLACK);
    ctx.set_line_width(4.0);
    ctx.set_line_join(LineJoin::Miter);
    ctx.begin_path();
    ctx.move_to(10.0, 100.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(190.0, 100.0);
    ctx.stroke();
    // Miter join 应在连接点 (100,10) 处产生额外的填充区域
    // 检查连接点附近有像素
    let join_pixel = ctx.get_image_data(98, 8, 1, 1);
    assert_ne!(join_pixel.data[3], 0, "Miter join: 连接点附近应有像素");
}

/// 测试描边使用 line_join Round 时产生圆角连接。
/// Round 连接的轮廓顶点应包含扇形顶点。
#[test]
fn test_stroke_line_join_round_corners() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_stroke_color(Color::GREEN);
    ctx.set_line_width(6.0);
    ctx.set_line_join(LineJoin::Round);
    ctx.begin_path();
    ctx.move_to(10.0, 100.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(190.0, 100.0);
    ctx.stroke();
    // Round join 应在连接点处产生覆盖区域
    let join_pixel = ctx.get_image_data(98, 8, 1, 1);
    assert_ne!(join_pixel.data[3], 0, "Round join: 连接点附近应有像素");
}

/// 测试 line_width 影响描边宽度。
/// 更大的 line_width 应产生更宽的描边覆盖区域。
#[test]
fn test_line_width_affects_stroke_width() {
    // 细线
    let mut ctx_thin = CanvasContext::new(100, 100);
    ctx_thin.set_stroke_color(Color::RED);
    ctx_thin.set_line_width(2.0);
    ctx_thin.begin_path();
    ctx_thin.move_to(50.0, 10.0);
    ctx_thin.line_to(50.0, 90.0);
    ctx_thin.stroke();
    // 粗线
    let mut ctx_thick = CanvasContext::new(100, 100);
    ctx_thick.set_stroke_color(Color::RED);
    ctx_thick.set_line_width(10.0);
    ctx_thick.begin_path();
    ctx_thick.move_to(50.0, 10.0);
    ctx_thick.line_to(50.0, 90.0);
    ctx_thick.stroke();
    // 粗线在距中心更远的位置应有像素
    // 细线 (line_width=2) 在 x=54 应无像素
    let thin_at_54 = ctx_thin.get_image_data(54, 50, 1, 1);
    assert_eq!(thin_at_54.data[0..4], [0, 0, 0, 0], "line_width=2: x=54 不应有像素");
    // 粗线 (line_width=10) 在 x=54 应有像素
    let thick_at_54 = ctx_thick.get_image_data(54, 50, 1, 1);
    assert_eq!(thick_at_54.data[0..4], [255, 0, 0, 255], "line_width=10: x=54 应有像素");
}

/// 测试 stroke_outline_vertices 生成包含法线偏移的轮廓顶点。
/// 验证单条线段的轮廓为 8 个浮点数（4 个顶点 × 2 坐标）。
#[test]
fn test_stroke_outline_vertices_single_segment() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Butt);
    ctx.set_line_join(LineJoin::Miter);
    ctx.begin_path();
    ctx.move_to(10.0, 50.0);
    ctx.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    // 单条水平线段：4 个角点 = 8 floats
    assert_eq!(outline.len(), 8, "单条线段轮廓应有 8 个浮点数（4 个顶点）");
    // 验证上下偏移：y 坐标应为 50 ± 2（line_width/2 = 2）
    let y_values: Vec<f32> = outline.iter().skip(1).step_by(2).copied().collect();
    assert!(
        y_values.iter().any(|&y| (y - 48.0).abs() < 0.1),
        "应有 y ≈ 48 的顶点（50 - half_lw）"
    );
    assert!(
        y_values.iter().any(|&y| (y - 52.0).abs() < 0.1),
        "应有 y ≈ 52 的顶点（50 + half_lw）"
    );
}

/// 测试 stroke_outline_vertices 包含连接点顶点。
/// 两条线段路径应生成线段轮廓 + 连接点轮廓。
#[test]
fn test_stroke_outline_vertices_with_join() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Butt);
    ctx.set_line_join(LineJoin::Bevel);
    ctx.begin_path();
    ctx.move_to(10.0, 50.0);
    ctx.line_to(50.0, 10.0);
    ctx.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    // 2 条线段 × 8 floats = 16（线段轮廓）+ 连接点顶点（Bevel = 4 floats）
    // 总计应大于 16
    assert!(
        outline.len() > 16,
        "两条线段路径应有连接点额外顶点，实际 {}",
        outline.len()
    );
}

/// 测试 stroke_outline_vertices 使用 Round cap 时包含额外扇形顶点。
#[test]
fn test_stroke_outline_vertices_round_cap_extra_vertices() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Round);
    ctx.set_line_join(LineJoin::Miter);
    ctx.begin_path();
    ctx.move_to(10.0, 50.0);
    ctx.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();

    // 对比 Butt cap
    let mut ctx_butt = CanvasContext::new(200, 200);
    ctx_butt.set_line_width(4.0);
    ctx_butt.set_line_cap(LineCap::Butt);
    ctx_butt.set_line_join(LineJoin::Miter);
    ctx_butt.begin_path();
    ctx_butt.move_to(10.0, 50.0);
    ctx_butt.line_to(90.0, 50.0);
    let outline_butt = ctx_butt.stroke_outline_vertices();

    // Round cap 应比 Butt cap 多出扇形顶点
    assert!(
        outline.len() > outline_butt.len(),
        "Round cap 应比 Butt cap 多出扇形顶点: {} vs {}",
        outline.len(),
        outline_butt.len()
    );
}

/// 测试 stroke_outline_vertices 使用 Square cap 时包含延伸矩形顶点。
#[test]
fn test_stroke_outline_vertices_square_cap_extra_vertices() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Square);
    ctx.set_line_join(LineJoin::Miter);
    ctx.begin_path();
    ctx.move_to(20.0, 50.0);
    ctx.line_to(80.0, 50.0);
    let outline = ctx.stroke_outline_vertices();

    let mut ctx_butt = CanvasContext::new(200, 200);
    ctx_butt.set_line_width(4.0);
    ctx_butt.set_line_cap(LineCap::Butt);
    ctx_butt.set_line_join(LineJoin::Miter);
    ctx_butt.begin_path();
    ctx_butt.move_to(20.0, 50.0);
    ctx_butt.line_to(80.0, 50.0);
    let outline_butt = ctx_butt.stroke_outline_vertices();

    // Square cap 应比 Butt cap 多出延伸矩形顶点
    assert!(
        outline.len() > outline_butt.len(),
        "Square cap 应比 Butt cap 多出延伸矩形顶点: {} vs {}",
        outline.len(),
        outline_butt.len()
    );
}

/// 测试 stroke_outline_vertices 空路径返回空列表。
#[test]
fn test_stroke_outline_vertices_empty_path() {
    let ctx = CanvasContext::new(100, 100);
    let outline = ctx.stroke_outline_vertices();
    assert!(outline.is_empty(), "空路径应返回空顶点列表");
}

/// 测试 stroke_outline_vertices 仅 MoveTo 的路径返回空列表。
#[test]
fn test_stroke_outline_vertices_move_to_only() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(50.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    assert!(outline.is_empty(), "仅 MoveTo 应返回空顶点列表");
}

/// 测试 line_join Miter 的轮廓顶点在连接处产生额外的尖角顶点。
#[test]
fn test_stroke_outline_miter_has_join_vertices() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(10.0);
    ctx.set_line_join(LineJoin::Miter);
    ctx.set_line_cap(LineCap::Butt);
    ctx.begin_path();
    // 直角转弯：(10,90) -> (10,10) -> (90,10)
    ctx.move_to(10.0, 90.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(90.0, 10.0);
    let outline = ctx.stroke_outline_vertices();

    // 对比 Bevel join
    let mut ctx_bevel = CanvasContext::new(200, 200);
    ctx_bevel.set_line_width(10.0);
    ctx_bevel.set_line_join(LineJoin::Bevel);
    ctx_bevel.set_line_cap(LineCap::Butt);
    ctx_bevel.begin_path();
    ctx_bevel.move_to(10.0, 90.0);
    ctx_bevel.line_to(10.0, 10.0);
    ctx_bevel.line_to(90.0, 10.0);
    let outline_bevel = ctx_bevel.stroke_outline_vertices();

    // Miter join 应在连接处有额外的尖角顶点
    assert!(
        outline.len() > outline_bevel.len(),
        "Miter join ({}) 应比 Bevel join ({}) 多出尖角顶点",
        outline.len(),
        outline_bevel.len()
    );
}

/// 测试 line_join Round 的轮廓包含扇形连接顶点。
#[test]
fn test_stroke_outline_round_join_has_fan_vertices() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(10.0);
    ctx.set_line_join(LineJoin::Round);
    ctx.set_line_cap(LineCap::Butt);
    ctx.begin_path();
    ctx.move_to(10.0, 90.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(90.0, 10.0);
    let outline = ctx.stroke_outline_vertices();

    // 对比 Bevel join
    let mut ctx_bevel = CanvasContext::new(200, 200);
    ctx_bevel.set_line_width(10.0);
    ctx_bevel.set_line_join(LineJoin::Bevel);
    ctx_bevel.set_line_cap(LineCap::Butt);
    ctx_bevel.begin_path();
    ctx_bevel.move_to(10.0, 90.0);
    ctx_bevel.line_to(10.0, 10.0);
    ctx_bevel.line_to(90.0, 10.0);
    let outline_bevel = ctx_bevel.stroke_outline_vertices();

    // Round join 应比 Bevel join 多出扇形顶点
    assert!(
        outline.len() > outline_bevel.len(),
        "Round join ({}) 应比 Bevel join ({}) 多出扇形顶点",
        outline.len(),
        outline_bevel.len()
    );
}

// ── 合成操作像素级测试（剩余操作） ──

/// 测试 destination-out 合成：先绘制红色，再使用 destination-out 绘制蓝色，
/// 重叠区域的已有内容被清除（变为透明）。
#[test]
fn test_composite_destination_out() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 destination-out 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::DestinationOut);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 红色独占区域 (2,0)：不变
    let red_only = ctx.get_image_data(2, 0, 1, 1);
    assert_eq!(
        red_only.data[0..4],
        [255, 0, 0, 255],
        "destination-out: 红色独占区域不变"
    );

    // 重叠区域 (7,0)：destination-out 清除已有内容 → 透明
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(overlap.data[0..4], [0, 0, 0, 0], "destination-out: 重叠区域应为透明");

    // 蓝色独占区域 (12,0)：无已有内容 → 透明
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(
        blue_only.data[0..4],
        [0, 0, 0, 0],
        "destination-out: 无目标区域应为透明"
    );
}

/// 测试 destination-atop 合成：先绘制红色，再使用 destination-atop 绘制蓝色。
/// destination-atop 在源区域内：保留目标在源区域内的部分。
/// 注意：当前实现只修改绘制矩形内的像素，矩形外的像素保持不变。
#[test]
fn test_composite_destination_atop() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 destination-atop 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::DestinationAtop);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 重叠区域 (7,0)：destination-atop → 保留目标 + 源
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_ne!(overlap.data[3], 0, "destination-atop: 重叠区域应有内容");

    // 蓝色独占区域 (12,0)：无目标 → destination-atop 保留源
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_ne!(blue_only.data[3], 0, "destination-atop: 源独占区域应有内容");
}

/// 测试 source-in 合成：先绘制红色，再使用 source-in 绘制蓝色。
/// source-in 只保留源与目标重叠的部分。
/// 注意：当前实现只修改绘制矩形内的像素，矩形外的目标像素保持不变。
#[test]
fn test_composite_source_in() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 source-in 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::SourceIn);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 重叠区域 (7,0)：source-in → fa=da=1.0, fb=0.0 → 源色（蓝色）
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-in: 重叠区域应为蓝色");

    // 蓝色独占区域 (12,0)：无目标 → source-in → 透明
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(blue_only.data[0..4], [0, 0, 0, 0], "source-in: 无目标区域应为透明");
}

/// 测试 destination-in 合成：先绘制红色，再使用 destination-in 绘制蓝色。
/// destination-in 只保留目标与源重叠的部分。
/// 注意：当前实现只修改绘制矩形内的像素，矩形外的目标像素保持不变。
#[test]
fn test_composite_destination_in() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 destination-in 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::DestinationIn);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 重叠区域 (7,0)：destination-in → fa=0, fb=sa=1.0 → 保留目标色（红色）
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    assert_eq!(
        overlap.data[0..4],
        [255, 0, 0, 255],
        "destination-in: 重叠区域应保留红色"
    );

    // 蓝色独占区域 (12,0)：无目标 → destination-in → 透明
    let blue_only = ctx.get_image_data(12, 0, 1, 1);
    assert_eq!(blue_only.data[0..4], [0, 0, 0, 0], "destination-in: 无目标区域应为透明");
}

/// 测试 lighter 合成：两个不同颜色像素的 lighter 模式进行加法混合。
#[test]
fn test_composite_lighter() {
    let mut ctx = CanvasContext::new(20, 10);
    // 先绘制红色 (0,0)-(10,10)
    ctx.set_fill_color(Color::rgba(200, 0, 0, 255));
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 使用 lighter 绘制蓝色 (5,0)-(15,10)
    ctx.set_composite_operation(CompositeOperation::Lighter);
    ctx.set_fill_color(Color::rgba(0, 0, 200, 255));
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    // 重叠区域：lighter 模式 = fa=1, fb=1 → 加法混合
    let overlap = ctx.get_image_data(7, 0, 1, 1);
    let r = overlap.data[0];
    let _g = overlap.data[1];
    let b = overlap.data[2];
    // 红色通道应有目标的贡献（dr * da * fb / out_a）
    // 蓝色通道应有源的贡献
    assert!(r >= 100, "lighter: 红色通道应 >= 100，实际 {}", r);
    assert!(b >= 100, "lighter: 蓝色通道应 >= 100，实际 {}", b);
}

/// 测试 copy 合成覆盖已有内容。
/// copy 模式只保留源像素，重叠区域的目标完全被替换。
#[test]
fn test_composite_copy_replaces() {
    let mut ctx = CanvasContext::new(20, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 20.0, 10.0);
    ctx.set_composite_operation(CompositeOperation::Copy);
    ctx.set_fill_color(Color::GREEN);
    ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

    let inside = ctx.get_image_data(10, 0, 1, 1);
    assert_eq!(inside.data[0..4], [0, 255, 0, 255], "copy: 内部区域应为绿色");
    // R34xx：copy 的未覆盖区域清除为透明（同 test_composite_copy）。
    let outside = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(outside.data[0..4], [0, 0, 0, 0], "copy: 外部区域应清除为透明");
}

// ── OffscreenCanvas 测试 ──

/// 测试 OffscreenCanvas 创建时的尺寸正确。
#[test]
fn test_offscreen_canvas_creation_with_dimensions() {
    let oc = OffscreenCanvas::new(640, 480);
    assert_eq!(oc.width(), 640);
    assert_eq!(oc.height(), 480);
}

/// 测试 OffscreenCanvas get_context 返回正确尺寸的 CanvasContext。
#[test]
fn test_offscreen_canvas_get_context_returns_working_context() {
    let mut oc = OffscreenCanvas::new(200, 150);
    let ctx = oc.get_context();
    assert_eq!(ctx.width(), 200);
    assert_eq!(ctx.height(), 150);
}

/// 测试在 OffscreenCanvas 上下文上绘制操作后能产生像素数据。
#[test]
fn test_offscreen_canvas_drawing_produces_pixels() {
    let mut oc = OffscreenCanvas::new(100, 100);
    let ctx = oc.get_context();
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
    // 验证绘制区域内有红色像素
    let pixel = ctx.get_image_data(20, 20, 1, 1);
    assert_eq!(
        pixel.data[0..4],
        [255, 0, 0, 255],
        "OffscreenCanvas 上下文绘制后应产生像素"
    );
}

/// 测试 OffscreenCanvas 的宽高与传入参数一致（含零尺寸边界情况）。
#[test]
fn test_offscreen_canvas_dimensions_are_correct() {
    let oc = OffscreenCanvas::new(0, 0);
    assert_eq!(oc.width(), 0);
    assert_eq!(oc.height(), 0);

    let oc2 = OffscreenCanvas::new(1920, 1080);
    assert_eq!(oc2.width(), 1920);
    assert_eq!(oc2.height(), 1080);
}

/// 测试 OffscreenCanvas transfer_to_image_bitmap 返回正确尺寸的 ImageData。
#[test]
fn test_offscreen_canvas_transfer_to_image_bitmap() {
    let mut oc = OffscreenCanvas::new(50, 40);
    let bitmap = oc.transfer_to_image_bitmap();
    assert_eq!(bitmap.width, 50);
    assert_eq!(bitmap.height, 40);
    assert_eq!(bitmap.data.len(), 50 * 40 * 4);
}

// ── 错误恢复测试 ──

/// 测试 drawImage 使用空 ImageData（零尺寸）时不 panic。
/// 空图像数据不应导致像素缓冲区越界访问或 panic。
#[test]
fn test_draw_image_no_data() {
    let mut ctx = CanvasContext::new(100, 100);
    // 空的 ImageData — 零尺寸，无像素数据
    let empty_img = ImageData {
        width: 0,
        height: 0,
        data: vec![],
    };
    // 不应 panic
    ctx.draw_image(&empty_img, 0.0, 0.0);
    ctx.draw_image_with_size(&empty_img, 0.0, 0.0, 50.0, 50.0);
    ctx.draw_image_sliced(&empty_img, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0, 50.0);
    // 验证画布未被修改
    let pixel = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "空图像不应写入任何像素");

    // ImageData 有尺寸但数据向量为空 — 也不应 panic
    let img_no_data = ImageData {
        width: 10,
        height: 10,
        data: vec![],
    };
    ctx.draw_image(&img_no_data, 0.0, 0.0);
    // 不 panic 即可
}

// ── Path2D closePath 和 is_point_in_path 测试 ──

/// 测试 Path2D close_path 后形成闭合三角形，顶点包含回到起点的线段。
#[test]
fn test_path2d_close_path_triangle() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(50.0, 100.0);
    p.close_path();
    // close_path 应添加 ClosePath 命令
    assert!(matches!(p.commands().last(), Some(PathCommand::ClosePath)));
    // 扁平化后应包含从 (50,100) 回到 (0,0) 的闭合线段
    let ctx = CanvasContext::new(200, 200);
    let vertices = ctx.flatten_path_for(&p);
    // 3 条线段: (0,0)->(100,0), (100,0)->(50,100), (50,100)->(0,0)
    // 每条 4 floats = 12 floats
    assert_eq!(vertices.len(), 12);
    // 最后一条线段应回到起点
    assert!((vertices[8] - 50.0).abs() < f32::EPSILON);
    assert!((vertices[9] - 100.0).abs() < f32::EPSILON);
    assert!((vertices[10]).abs() < f32::EPSILON);
    assert!((vertices[11]).abs() < f32::EPSILON);
}

/// 测试闭合路径 fill 产生非空 path_fills。
#[test]
fn test_path2d_close_path_creates_fill() {
    let mut p = Path2D::new();
    p.move_to(10.0, 10.0);
    p.line_to(100.0, 10.0);
    p.line_to(100.0, 100.0);
    p.close_path();
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_with_path(&p);
    assert!(
        !ctx.primitives().path_fills.is_empty(),
        "闭合路径 fill 应产生 path_fill 图元"
    );
}

/// 测试 Path2D is_point_in_path 在矩形内部返回 true。
#[test]
fn test_path2d_is_point_in_path_inside() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(100.0, 100.0);
    p.line_to(0.0, 100.0);
    p.close_path();
    // 矩形内部点应返回 true
    assert!(p.is_point_in_path(50.0, 50.0));
    assert!(p.is_point_in_path(10.0, 10.0));
}

/// 测试 Path2D is_point_in_path 在矩形外部返回 false。
#[test]
fn test_path2d_is_point_in_path_outside() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(100.0, 100.0);
    p.line_to(0.0, 100.0);
    p.close_path();
    // 矩形外部点应返回 false
    assert!(!p.is_point_in_path(200.0, 200.0));
    assert!(!p.is_point_in_path(-10.0, -10.0));
    assert!(!p.is_point_in_path(150.0, 50.0));
}

/// 测试 Path2D is_point_in_path 在边界上不 panic。
#[test]
fn test_path2d_is_point_in_path_edge() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(100.0, 100.0);
    p.line_to(0.0, 100.0);
    p.close_path();
    // 边界上的点不应 panic（结果不确定，只验证不崩溃）
    let _ = p.is_point_in_path(0.0, 0.0);
    let _ = p.is_point_in_path(100.0, 100.0);
    let _ = p.is_point_in_path(50.0, 0.0);
    let _ = p.is_point_in_path(0.0, 50.0);
}

// ── 新增边界条件测试 ──

/// 测试 resize 后画布尺寸和像素缓冲区正确更新。
#[test]
fn test_canvas_create_resize() {
    let mut ctx = CanvasContext::new(100, 200);
    assert_eq!(ctx.width(), 100);
    assert_eq!(ctx.height(), 200);
    ctx.resize(400, 300);
    assert_eq!(ctx.width(), 400);
    assert_eq!(ctx.height(), 300);
    // resize 后像素缓冲区应全部为零
    let pixel = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0]);
}

/// 测试 clearRect(0,0,w,h) 将整个画布清为透明。
#[test]
fn test_canvas_clear_entire() {
    let mut ctx = CanvasContext::new(50, 50);
    // 先填充红色
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // 验证 (0,0) 为红色
    let before = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(before.data[0..4], [255, 0, 0, 255]);
    // 清除整个画布
    ctx.clear_rect(0.0, 0.0, 50.0, 50.0);
    // 验证 (0,0) 为透明
    let after = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(
        after.data[0..4],
        [0, 0, 0, 0],
        "clearRect(0,0,w,h) should make pixel transparent"
    );
}

/// 测试 lineWidth 为 0 时 stroke 不 panic。
#[test]
fn test_canvas_stroke_zero_width() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(0.0);
    ctx.stroke_rect(10.0, 10.0, 50.0, 50.0);
    // 不应 panic；零线宽生成描边图元但无像素（R34xx 旧四边 fill 断言已废弃）
    assert_eq!(ctx.primitives().path_strokes.len(), 1);
}

/// 测试负值平移后变换矩阵正确。
#[test]
fn test_canvas_negative_translate() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(-10.0, -20.0);
    let t = ctx.transform;
    assert!((t.e - (-10.0)).abs() < f32::EPSILON, "translate tx should be -10");
    assert!((t.f - (-20.0)).abs() < f32::EPSILON, "translate ty should be -20");
    assert!((t.a - 1.0).abs() < f32::EPSILON);
    assert!((t.d - 1.0).abs() < f32::EPSILON);
}

/// 测试无 save 时 restore 不 panic，状态保持默认。
#[test]
fn test_canvas_restore_without_save() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.restore(); // 不应 panic
    assert_eq!(ctx.fill_color(), Color::BLACK);
    assert!((ctx.global_alpha() - 1.0).abs() < f32::EPSILON);
    assert!((ctx.line_width() - 1.0).abs() < f32::EPSILON);
}

/// 测试 globalAlpha 超出 [0,1] 时被 clamp。
#[test]
fn test_canvas_set_global_alpha_clamp() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(2.0);
    assert!(
        (ctx.global_alpha() - 1.0).abs() < f32::EPSILON,
        "alpha > 1 should clamp to 1.0"
    );
    ctx.set_global_alpha(-0.5);
    assert!(
        (ctx.global_alpha()).abs() < f32::EPSILON,
        "alpha < 0 should clamp to 0.0"
    );
}

// ── 新增边界条件测试（6 个） ──

/// 测试创建包含 4 个颜色停止点的线性渐变，验证停止点数量、偏移量和颜色均正确。
#[test]
fn test_canvas_create_linear_gradient_multi_stop() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.33, Color::rgba(255, 255, 0, 255));
    grad.add_color_stop(0.66, Color::GREEN);
    grad.add_color_stop(1.0, Color::BLUE);
    assert_eq!(grad.stops.len(), 4);
    assert!((grad.stops[0].offset - 0.0).abs() < f32::EPSILON);
    assert_eq!(grad.stops[0].color, Color::RED);
    assert!((grad.stops[1].offset - 0.33).abs() < f32::EPSILON);
    assert_eq!(grad.stops[1].color, Color::rgba(255, 255, 0, 255));
    assert!((grad.stops[2].offset - 0.66).abs() < f32::EPSILON);
    assert_eq!(grad.stops[2].color, Color::GREEN);
    assert!((grad.stops[3].offset - 1.0).abs() < f32::EPSILON);
    assert_eq!(grad.stops[3].color, Color::BLUE);
}

/// 测试创建径向渐变，验证内外圆的圆心坐标和半径正确。
#[test]
fn test_canvas_create_radial_gradient_circle() {
    let ctx = CanvasContext::new(200, 200);
    let grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 80.0);
    // 内圆：圆心 (50,50)，半径 10
    assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r0 - 10.0).abs() < f32::EPSILON);
    // 外圆：圆心 (50,50)，半径 80
    assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r1 - 80.0).abs() < f32::EPSILON);
    // 同心圆渐变初始无停止点
    assert!(grad.stops.is_empty());
}

/// 测试当前路径填充使用奇偶规则（even-odd）。
/// 当前实现默认使用非零环绕规则，is_point_in_path 基于射线法（等效于奇偶规则）。
/// 验证嵌套矩形路径在奇偶规则下内部矩形被判断为"外部"。
#[test]
fn test_canvas_fill_rule_evenodd() {
    // 使用 CanvasContext 的 is_point_in_path（射线法 = 奇偶规则）
    // 构造嵌套矩形路径（外矩形顺时针 + 内矩形顺时针）
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    // 外矩形
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(100.0, 100.0);
    ctx.line_to(0.0, 100.0);
    ctx.close_path();
    // 内矩形（反向绕行模拟 even-odd 挖空）
    ctx.move_to(25.0, 25.0);
    ctx.line_to(25.0, 75.0);
    ctx.line_to(75.0, 75.0);
    ctx.line_to(75.0, 25.0);
    ctx.close_path();
    // 射线法（奇偶规则）：外矩形与内矩形之间的点穿过 1 条边 → 在路径内
    assert!(ctx.is_point_in_path(15.0, 15.0), "even-odd: 两矩形之间的点应在路径内");
    // 射线法（奇偶规则）：内矩形内的点穿过 2 条边 → 不在路径内
    assert!(
        !ctx.is_point_in_path(50.0, 50.0),
        "even-odd: 内矩形内的点应不在路径内（穿过偶数条边）"
    );
}

/// 测试设置线段虚线模式 [5, 10, 15]，验证 get_line_dash 返回加倍后的数组。
#[test]
fn test_canvas_set_line_dash_pattern() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_dash(vec![5.0, 10.0, 15.0]);
    // 奇数长度时 Canvas 规范要求复制拼接
    assert_eq!(ctx.get_line_dash(), &[5.0, 10.0, 15.0, 5.0, 10.0, 15.0]);
}

/// 测试 measure_text("Hello") 返回非零宽度。
#[test]
fn test_canvas_measure_text_hello() {
    let ctx = CanvasContext::new(200, 200);
    let metrics = ctx.measure_text("Hello");
    // 默认字体大小 10.0，5 字符 × 10.0 × 0.6 = 30.0
    assert!(
        metrics.width > 0.0,
        "measure_text(\"Hello\") 宽度应大于零，实际 {}",
        metrics.width
    );
    assert!(
        (metrics.width - 30.0).abs() < f32::EPSILON,
        "measure_text(\"Hello\") 宽度应约 30.0，实际 {}",
        metrics.width
    );
}

/// 测试设置 shadowBlur、shadowOffsetX、shadowOffsetY、shadowColor 后，
/// 每个 getter 返回正确的设置值。
#[test]
fn test_canvas_shadow_properties() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_blur(8.0);
    ctx.set_shadow_offset_x(4.0);
    ctx.set_shadow_offset_y(6.0);
    ctx.set_shadow_color(Color::rgba(128, 0, 128, 200));
    assert!((ctx.shadow_blur() - 8.0).abs() < f32::EPSILON, "shadowBlur 应为 8.0");
    assert!(
        (ctx.shadow_offset_x() - 4.0).abs() < f32::EPSILON,
        "shadowOffsetX 应为 4.0"
    );
    assert!(
        (ctx.shadow_offset_y() - 6.0).abs() < f32::EPSILON,
        "shadowOffsetY 应为 6.0"
    );
    let sc = ctx.shadow_color();
    assert_eq!(sc.r, 128, "shadowColor.r 应为 128");
    assert_eq!(sc.g, 0, "shadowColor.g 应为 0");
    assert_eq!(sc.b, 128, "shadowColor.b 应为 128");
    assert_eq!(sc.a, 200, "shadowColor.a 应为 200");
}

// ── 边界条件测试：putImageData/getImageData、createConicGradient、font、textAlign、textBaseline ──

/// 测试 put_image_data 后 get_image_data 返回完全一致的像素数据。
#[test]
fn test_canvas_put_image_data_and_get() {
    let mut ctx = CanvasContext::new(10, 10);
    // 构造 3x3 的彩虹色 ImageData
    let pixels: Vec<u8> = vec![
        255, 0, 0, 255, // 红
        0, 255, 0, 255, // 绿
        0, 0, 255, 255, // 蓝
        255, 255, 0, 255, // 黄
        255, 0, 255, 255, // 品红
        0, 255, 255, 255, // 青
        128, 128, 128, 255, // 灰
        255, 128, 0, 255, // 橙
        0, 128, 255, 255, // 天蓝
    ];
    let img = ImageData {
        width: 3,
        height: 3,
        data: pixels.clone(),
    };
    ctx.put_image_data(&img, 2, 3);
    // 读取写入区域并验证像素完全匹配
    let result = ctx.get_image_data(2, 3, 3, 3);
    assert_eq!(result.data, pixels, "put 后 get 的像素数据应完全一致");
    // 验证写入区域外的像素仍为零
    let outside = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(outside.data[0..4], [0, 0, 0, 0], "写入区域外应保持透明");
}

/// 测试 create_conic_gradient 指定起始角度后，渐变对象的 start_angle 与传入值精确匹配。
#[test]
fn test_canvas_create_conic_gradient() {
    let ctx = CanvasContext::new(200, 200);
    let angle = std::f32::consts::FRAC_PI_2; // 90 度
    let grad = ctx.create_conic_gradient(angle, 75.0, 125.0);
    assert!(
        (grad.start_angle - angle).abs() < f32::EPSILON,
        "start_angle 应为 {}",
        angle
    );
    assert!((grad.cx - 75.0).abs() < f32::EPSILON, "cx 应为 75.0");
    assert!((grad.cy - 125.0).abs() < f32::EPSILON, "cy 应为 125.0");
}

/// 测试 set_font 设置 "bold 16px Arial" 风格字体后，font() getter 返回正确的描述符。
#[test]
fn test_canvas_set_font_and_get() {
    let mut ctx = CanvasContext::new(100, 100);
    let font = FontDescriptor {
        family: "Arial".to_string(),
        size: 16.0,
        weight: FontWeight::Bold,
        style: FontStyle::Normal,
        small_caps: false,
        weight_value: None,
        letter_spacing: "0px".to_string(),
        word_spacing: "0px".to_string(),
        kerning_none: false,
        lang: String::new(),
    };
    ctx.set_font(font);
    let f = ctx.font();
    assert_eq!(f.family, "Arial");
    assert!((f.size - 16.0).abs() < f32::EPSILON, "字体大小应为 16.0");
    assert_eq!(f.weight, FontWeight::Bold, "字体粗细应为 Bold");
    assert_eq!(f.style, FontStyle::Normal, "字体样式应为 Normal");
}

/// 测试 set_text_align 对各种值（left, center, right）的设置和获取。
#[test]
fn test_canvas_text_align_values() {
    let mut ctx = CanvasContext::new(100, 100);
    // 默认值应为 Start
    assert_eq!(ctx.text_align(), TextAlign::Start);

    ctx.set_text_align(TextAlign::Left);
    assert_eq!(ctx.text_align(), TextAlign::Left);

    ctx.set_text_align(TextAlign::Center);
    assert_eq!(ctx.text_align(), TextAlign::Center);

    ctx.set_text_align(TextAlign::Right);
    assert_eq!(ctx.text_align(), TextAlign::Right);
}

/// 测试 set_text_baseline 对各种值（top, middle, bottom）的设置和获取。
#[test]
fn test_canvas_text_baseline_values() {
    let mut ctx = CanvasContext::new(100, 100);
    // 默认值应为 Alphabetic
    assert_eq!(ctx.text_baseline(), TextBaseline::Alphabetic);

    ctx.set_text_baseline(TextBaseline::Top);
    assert_eq!(ctx.text_baseline(), TextBaseline::Top);

    ctx.set_text_baseline(TextBaseline::Middle);
    assert_eq!(ctx.text_baseline(), TextBaseline::Middle);

    ctx.set_text_baseline(TextBaseline::Bottom);
    assert_eq!(ctx.text_baseline(), TextBaseline::Bottom);
}

// ── Path2D.addPath() 测试 ──

/// 测试 add_path 将两个包含矩形的路径合并后，命令数量正确。
#[test]
fn test_path2d_add_path() {
    let mut p1 = Path2D::new();
    p1.rect(0.0, 0.0, 10.0, 10.0); // 5 个命令
    let mut p2 = Path2D::new();
    p2.rect(20.0, 20.0, 10.0, 10.0); // 5 个命令
    p1.add_path(&p2);
    assert_eq!(p1.len(), 10, "合并后应有 10 个命令");
    assert!(!p2.is_empty(), "源路径应不受影响");
}

/// 测试 add_path 追加空路径后，原路径不变。
#[test]
fn test_path2d_add_path_empty() {
    let mut p = Path2D::new();
    p.rect(0.0, 0.0, 50.0, 50.0); // 5 个命令
    let empty = Path2D::new();
    p.add_path(&empty);
    assert_eq!(p.len(), 5, "追加空路径后命令数不变");
}

/// 测试 add_path 后源路径保持不变。
#[test]
fn test_path2d_add_path_preserves_original() {
    let mut target = Path2D::new();
    target.rect(0.0, 0.0, 10.0, 10.0);
    let mut source = Path2D::new();
    source.rect(100.0, 100.0, 20.0, 20.0);
    let source_len_before = source.len();
    target.add_path(&source);
    assert_eq!(source.len(), source_len_before, "add_path 后源路径命令数不变");
}

/// 测试闭合三角路径后 is_point_in_path 正确判断内部点。
#[test]
fn test_path2d_close_path_is_point_in_path() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(50.0, 100.0);
    p.close_path();
    // 三角形中心应在路径内
    assert!(p.is_point_in_path(50.0, 40.0), "三角形内部点应命中");
    // 远离三角形的外部点不应命中
    assert!(!p.is_point_in_path(200.0, 200.0), "外部点不应命中");
}

// ── create_image_data 测试 ──

/// 测试 create_image_data 创建指定尺寸的 ImageData，数据全为零。
#[test]
fn test_create_image_data() {
    let ctx = CanvasContext::new(100, 100);
    let img = ctx.create_image_data(10, 20);
    assert_eq!(img.width, 10);
    assert_eq!(img.height, 20);
    assert_eq!(img.data.len(), 800); // 10 * 20 * 4
    // 所有像素应为透明黑色
    for chunk in img.data.chunks_exact(4) {
        assert_eq!(chunk, &[0, 0, 0, 0]);
    }
}

/// 测试 create_image_data 零尺寸不 panic。
#[test]
fn test_create_image_data_zero_size() {
    let ctx = CanvasContext::new(100, 100);
    let img = ctx.create_image_data(0, 0);
    assert_eq!(img.width, 0);
    assert_eq!(img.height, 0);
    assert!(img.data.is_empty());
}

/// 测试 create_image_data 与 get_image_data 的区别：
/// create_image_data 返回全零，get_image_data 从画布读取实际像素。
#[test]
fn test_create_image_data_vs_get_image_data() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let created = ctx.create_image_data(5, 5);
    let fetched = ctx.get_image_data(0, 0, 5, 5);
    // created 应全为零（透明黑）
    assert_eq!(created.data[0..4], [0, 0, 0, 0]);
    // fetched 应为红色（从画布读取）
    assert_eq!(fetched.data[0..4], [255, 0, 0, 255]);
}

// ── get_transform 测试 ──

/// 测试 get_transform 返回单位矩阵（初始状态）。
#[test]
fn test_get_transform_identity() {
    let ctx = CanvasContext::new(100, 100);
    let t = ctx.get_transform();
    assert!((t.a - 1.0).abs() < f32::EPSILON);
    assert!((t.b).abs() < f32::EPSILON);
    assert!((t.c).abs() < f32::EPSILON);
    assert!((t.d - 1.0).abs() < f32::EPSILON);
    assert!((t.e).abs() < f32::EPSILON);
    assert!((t.f).abs() < f32::EPSILON);
}

/// 测试 get_transform 在 translate 后返回正确的矩阵。
#[test]
fn test_get_transform_after_translate() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(10.0, 20.0);
    let t = ctx.get_transform();
    assert!((t.a - 1.0).abs() < f32::EPSILON);
    assert!((t.d - 1.0).abs() < f32::EPSILON);
    assert!((t.e - 10.0).abs() < f32::EPSILON);
    assert!((t.f - 20.0).abs() < f32::EPSILON);
}

/// 测试 get_transform 在 set_transform 后返回设置的矩阵。
#[test]
fn test_get_transform_after_set() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_transform(2.0, 0.5, -0.5, 2.0, 10.0, 20.0);
    let t = ctx.get_transform();
    assert!((t.a - 2.0).abs() < f32::EPSILON);
    assert!((t.b - 0.5).abs() < f32::EPSILON);
    assert!((t.c - (-0.5)).abs() < f32::EPSILON);
    assert!((t.d - 2.0).abs() < f32::EPSILON);
    assert!((t.e - 10.0).abs() < f32::EPSILON);
    assert!((t.f - 20.0).abs() < f32::EPSILON);
}

// ── transform(a,b,c,d,e,f) 乘法方法测试 ──

/// 测试 transform() 方法将参数矩阵乘到当前变换上。
#[test]
fn test_transform_multiply_basic() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    let t = ctx.get_transform();
    // 单位矩阵 * scale(2,2) = scale(2,2)
    assert!((t.a - 2.0).abs() < f32::EPSILON);
    assert!((t.d - 2.0).abs() < f32::EPSILON);
}

/// 测试 transform() 后乘顺序：先 scale 后 translate。
#[test]
fn test_transform_post_multiply_order() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.scale(2.0, 1.0);
    ctx.transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
    let (x, y) = ctx.get_transform().transform_point(5.0, 5.0);
    // scale(2,1) * translate(10,20)：矩阵乘法结果 e = 2*10 = 20, f = 1*20 = 20
    // transform_point(5,5) = (2*5 + 20, 1*5 + 20) = (30, 25)
    assert!((x - 30.0).abs() < 0.01);
    assert!((y - 25.0).abs() < 0.01);
}

/// 测试 transform() 不会替换而是叠加（与 set_transform 的区别）。
#[test]
fn test_transform_accumulates_vs_set_transform_replaces() {
    let mut ctx1 = CanvasContext::new(100, 100);
    ctx1.translate(10.0, 0.0);
    ctx1.transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    let t1 = ctx1.get_transform();

    let mut ctx2 = CanvasContext::new(100, 100);
    ctx2.translate(10.0, 0.0);
    ctx2.set_transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    let t2 = ctx2.get_transform();

    // transform() 叠加：translate(10,0) * scale(2,2)
    assert!((t1.a - 2.0).abs() < f32::EPSILON, "transform should accumulate");
    assert!((t1.e - 10.0).abs() < f32::EPSILON, "translate should remain");

    // set_transform() 替换
    assert!((t2.a - 2.0).abs() < f32::EPSILON, "set_transform replaces");
    assert!((t2.e).abs() < f32::EPSILON, "set_transform clears translate");
}

/// 测试连续多次 transform() 调用累积。
#[test]
fn test_transform_multiple_calls() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.transform(2.0, 0.0, 0.0, 1.0, 0.0, 0.0); // scale x2
    ctx.transform(1.0, 0.0, 0.0, 3.0, 0.0, 0.0); // scale y3
    let (x, y) = ctx.get_transform().transform_point(5.0, 5.0);
    // identity * scaleX(2) * scaleY(3) applied to (5,5):
    // first scaleX: (10, 5), then scaleY: (10, 15)
    assert!((x - 10.0).abs() < 0.01);
    assert!((y - 15.0).abs() < 0.01);
}

// ── miter_limit 测试 ──

/// 测试 miter_limit 默认值为 10.0。
#[test]
fn test_miter_limit_default() {
    let ctx = CanvasContext::new(100, 100);
    assert!((ctx.miter_limit() - 10.0).abs() < f32::EPSILON);
}

/// 测试设置和获取 miter_limit。
#[test]
fn test_miter_limit_set_get() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_miter_limit(5.0);
    assert!((ctx.miter_limit() - 5.0).abs() < f32::EPSILON);
    ctx.set_miter_limit(20.0);
    assert!((ctx.miter_limit() - 20.0).abs() < f32::EPSILON);
}

/// 测试 miter_limit 在 save/restore 中正确保存和恢复。
#[test]
fn test_miter_limit_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_miter_limit(5.0);
    ctx.save();
    ctx.set_miter_limit(15.0);
    assert!((ctx.miter_limit() - 15.0).abs() < f32::EPSILON);
    ctx.restore();
    assert!((ctx.miter_limit() - 5.0).abs() < f32::EPSILON);
}

// ── direction 测试 ──

/// 测试 direction 默认值为 Inherit。
#[test]
fn test_direction_default() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.direction(), TextDirection::Inherit);
}

/// 测试 TextDirection 枚举 Default trait。
#[test]
fn test_text_direction_default_trait() {
    assert_eq!(TextDirection::default(), TextDirection::Inherit);
}

/// 测试设置和获取 direction。
#[test]
fn test_direction_set_get() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_direction(TextDirection::Ltr);
    assert_eq!(ctx.direction(), TextDirection::Ltr);
    ctx.set_direction(TextDirection::Rtl);
    assert_eq!(ctx.direction(), TextDirection::Rtl);
    ctx.set_direction(TextDirection::Inherit);
    assert_eq!(ctx.direction(), TextDirection::Inherit);
}

/// 测试 direction 在 save/restore 中正确保存和恢复。
#[test]
fn test_direction_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_direction(TextDirection::Rtl);
    ctx.save();
    ctx.set_direction(TextDirection::Ltr);
    assert_eq!(ctx.direction(), TextDirection::Ltr);
    ctx.restore();
    assert_eq!(ctx.direction(), TextDirection::Rtl);
}

/// 测试 TextDirection 枚举各变体互不相等。
#[test]
fn test_text_direction_variants_distinct() {
    let variants = [TextDirection::Ltr, TextDirection::Rtl, TextDirection::Inherit];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b, "TextDirection variants {} and {} should differ", i, j);
            }
        }
    }
}
