//! Canvas context_impl.rs 覆盖率补充测试。
//!
//! 重点覆盖：
//! - fill/stroke 带阴影（draw_shadow_path）
//! - is_point_in_stroke 空路径和命中
//! - draw_image_sliced 缩放绘制
//! - save/restore 状态管理
//! - clip 路径裁剪
//! - 各种 getter/setter

use crate::context::*;
use crate::path::Path2D;
use zero_render_foundation::color::Color;

// ═══════════════════════════════════════════════════════════════════════
// fill/stroke 带阴影 → 覆盖 draw_shadow_path（lines 994-1014）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fill_with_shadow() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 128));
    ctx.set_shadow_blur(5.0);
    ctx.set_shadow_offset_x(3.0);
    ctx.set_shadow_offset_y(3.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(50.0, 10.0);
    ctx.line_to(50.0, 50.0);
    ctx.close_path();
    ctx.fill();
    // 不应 panic
}

#[test]
fn test_stroke_with_shadow() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 100));
    ctx.set_shadow_blur(2.0);
    ctx.set_shadow_offset_x(2.0);
    ctx.set_shadow_offset_y(2.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(80.0, 80.0);
    ctx.stroke();
}

#[test]
fn test_fill_with_shadow_zero_blur() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::rgba(255, 0, 0, 200));
    ctx.set_shadow_blur(0.0);
    ctx.set_shadow_offset_x(1.0);
    ctx.set_shadow_offset_y(1.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(60.0, 10.0);
    ctx.line_to(60.0, 60.0);
    ctx.line_to(10.0, 60.0);
    ctx.close_path();
    ctx.fill();
}

// ═══════════════════════════════════════════════════════════════════════
// is_point_in_stroke 空路径（line 748）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_point_in_stroke_empty_path() {
    let ctx = CanvasContext::new(100, 100);
    // 没有路径命令
    assert!(!ctx.is_point_in_stroke(50.0, 50.0));
}

#[test]
fn test_is_point_in_stroke_hit() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.line_to(100.0, 50.0);
    // 点在线段上
    assert!(ctx.is_point_in_stroke(50.0, 50.0));
}

#[test]
fn test_is_point_in_stroke_miss() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.line_to(100.0, 50.0);
    // 点在线段外
    assert!(!ctx.is_point_in_stroke(50.0, 0.0));
}

// ═══════════════════════════════════════════════════════════════════════
// draw_image_sliced（lines 849-957）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_draw_image_sliced_basic() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 50,
        height: 50,
        data: vec![255u8; 50 * 50 * 4],
    };
    ctx.draw_image_sliced(&img, 0.0, 0.0, 50.0, 50.0, 10.0, 10.0, 40.0, 40.0);
}

#[test]
fn test_draw_image_sliced_with_alpha() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(0.5);
    let mut img = ImageData {
        width: 20,
        height: 20,
        data: vec![128u8; 20 * 20 * 4],
    };
    // 设置一些半透明像素
    for i in 0..20 {
        for j in 0..20 {
            let idx = (i * 20 + j) * 4;
            img.data[idx] = 255; // R
            img.data[idx + 1] = 0; // G
            img.data[idx + 2] = 0; // B
            img.data[idx + 3] = 128; // A
        }
    }
    ctx.draw_image_sliced(&img, 0.0, 0.0, 20.0, 20.0, 0.0, 0.0, 20.0, 20.0);
}

#[test]
fn test_draw_image_sliced_zero_source() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![255u8; 10 * 10 * 4],
    };
    // 零宽度/高度的源 → 应提前返回不 panic
    ctx.draw_image_sliced(&img, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 10.0, 10.0);
}

// ═══════════════════════════════════════════════════════════════════════
// draw_image_with_size（lines 833-846）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_draw_image_with_size() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![255u8; 10 * 10 * 4],
    };
    ctx.draw_image_with_size(&img, 0.0, 0.0, 50.0, 50.0);
}

// ═══════════════════════════════════════════════════════════════════════
// save/restore 状态管理（lines 370-420）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_save_restore_shadow() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 255));
    ctx.set_shadow_blur(10.0);
    ctx.save();
    ctx.set_shadow_blur(20.0);
    assert_eq!(ctx.shadow_blur(), 20.0);
    ctx.restore();
    assert_eq!(ctx.shadow_blur(), 10.0);
}

// ═══════════════════════════════════════════════════════════════════════
// clip（lines 670-693）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_clip_basic() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(60.0, 10.0);
    ctx.line_to(60.0, 60.0);
    ctx.line_to(10.0, 60.0);
    ctx.close_path();
    ctx.clip();
}

#[test]
fn test_clip_with_path() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.rect(10.0, 10.0, 50.0, 50.0);
    ctx.clip_with_path(&path);
}

// ═══════════════════════════════════════════════════════════════════════
// line dash（lines 341-365）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_line_dash() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_dash(vec![5.0, 10.0]);
    assert_eq!(ctx.get_line_dash(), &[5.0, 10.0]);
    ctx.set_line_dash_offset(2.0);
    assert_eq!(ctx.get_line_dash_offset(), 2.0);
}

// ═══════════════════════════════════════════════════════════════════════
// composite operation（lines 697-702）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_composite_operation() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_composite_operation(CompositeOperation::SourceOver);
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
}

// ═══════════════════════════════════════════════════════════════════════
// get_image_data / put_image_data（lines 763-815）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_and_put_image_data() {
    let mut ctx = CanvasContext::new(50, 50);
    let img = ctx.get_image_data(0, 0, 10, 10);
    assert_eq!(img.width, 10);
    assert_eq!(img.height, 10);
    ctx.put_image_data(&img, 5, 5);
}

#[test]
fn test_create_image_data() {
    let ctx = CanvasContext::new(50, 50);
    let img = ctx.create_image_data(10, 10);
    assert_eq!(img.width, 10);
    assert_eq!(img.height, 10);
    assert!(img.data.iter().all(|&b| b == 0));
}

// ═══════════════════════════════════════════════════════════════════════
// gradients and patterns（lines 709-726）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_linear_gradient() {
    let ctx = CanvasContext::new(100, 100);
    let grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 100.0);
    // 不应 panic
    let _ = grad;
}

#[test]
fn test_create_radial_gradient() {
    let ctx = CanvasContext::new(100, 100);
    let grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 50.0);
    let _ = grad;
}

#[test]
fn test_create_conic_gradient() {
    let ctx = CanvasContext::new(100, 100);
    let grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
    let _ = grad;
}

#[test]
fn test_create_pattern() {
    let ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![255u8; 10 * 10 * 4],
    };
    let _pattern = ctx.create_pattern(img, PatternRepetition::Repeat);
}

// ═══════════════════════════════════════════════════════════════════════
// transform operations（lines 424-440）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_set_transform() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_transform(2.0, 0.0, 0.0, 2.0, 10.0, 10.0);
}

#[test]
fn test_translate() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(10.0, 20.0);
}

#[test]
fn test_scale() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.scale(2.0, 3.0);
}

// ═══════════════════════════════════════════════════════════════════════
// resize（lines 616-625）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_resize() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.resize(200, 200);
    assert_eq!(ctx.width(), 200);
    assert_eq!(ctx.height(), 200);
}

// ═══════════════════════════════════════════════════════════════════════
// is_point_in_path（lines 734-758）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_point_in_path_hit() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(60.0, 10.0);
    ctx.line_to(60.0, 60.0);
    ctx.line_to(10.0, 60.0);
    ctx.close_path();
    assert!(ctx.is_point_in_path(30.0, 30.0));
}

#[test]
fn test_is_point_in_path_miss() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(60.0, 10.0);
    ctx.line_to(60.0, 60.0);
    ctx.line_to(10.0, 60.0);
    ctx.close_path();
    assert!(!ctx.is_point_in_path(0.0, 0.0));
}

// R3359：isPointInPath 命中测试在画布坐标空间（device space）进行——路径顶点在 move_to/
// line_to 追加时已按 CTM 变换（context_impl.rs move_to/line_to），flatten_path 产出设备空间
// 顶点，故 isPointInPath(x,y) 直接在设备空间点 vs 设备空间顶点命中（spec CanvasRenderingContext2D.
// isPointInPath：点在画布坐标空间）。本组测锁定「CTM 非单位时命中测试正确」——translate 后
// 路径在设备空间偏移，设备空间点命中设备空间变换后路径。
#[test]
fn test_is_point_in_path_under_transform_r3359() {
    let mut ctx = CanvasContext::new(100, 100);
    // translate(50,0)：move_to(10,10) 经 CTM 追加为设备空间顶点 (60,10)；路径设备空间 (60,10)-(70,20)。
    ctx.translate(50.0, 0.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(20.0, 10.0);
    ctx.line_to(20.0, 20.0);
    ctx.line_to(10.0, 20.0);
    ctx.close_path();
    // 设备空间点 (65,15) 在设备空间路径 (60,10)-(70,20) 内 → 命中。
    assert!(
        ctx.is_point_in_path(65.0, 15.0),
        "isPointInPath 须在画布坐标空间命中（路径顶点已按 CTM 变换）"
    );
    // 设备空间点 (15,15) 在路径空间但不在设备空间路径内 → 未命中。
    assert!(!ctx.is_point_in_path(15.0, 15.0));
}

// R3359：scale(2,2) 下路径顶点放大——move_to(10,10) 经 scale 追加为 (20,20)，路径设备空间 (20,20)-(40,40)。
#[test]
fn test_is_point_in_path_under_scale_r3359() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.scale(2.0, 2.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(20.0, 10.0);
    ctx.line_to(20.0, 20.0);
    ctx.line_to(10.0, 20.0);
    ctx.close_path();
    // 设备空间点 (30,30) 在设备空间路径 (20,20)-(40,40) 内 → 命中。
    assert!(ctx.is_point_in_path(30.0, 30.0));
    // 设备空间点 (15,15) 在路径空间但不在放大后的设备空间路径内 → 未命中。
    assert!(!ctx.is_point_in_path(15.0, 15.0));
}

// R3359：isPointInStroke 同在画布坐标空间——描边中线顶点已按 CTM 变换，距离阈值 line_width/2
// 在设备空间度量。
#[test]
fn test_is_point_in_stroke_under_transform_r3359() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(50.0, 0.0);
    ctx.set_line_width(4.0);
    ctx.begin_path();
    ctx.move_to(10.0, 30.0); // 经 CTM 追加为设备空间 (60,30)
    ctx.line_to(20.0, 30.0); // 设备空间 (70,30)
    // 设备空间点 (65,30) 在描边中线 (60,30)-(70,30) 上 → 命中。
    assert!(ctx.is_point_in_stroke(65.0, 30.0));
    // 设备空间点 (15,30) 远离设备空间描边 → 未命中。
    assert!(!ctx.is_point_in_stroke(15.0, 30.0));
}

// ═══════════════════════════════════════════════════════════════════════
// fill_with_path / stroke_with_path（lines 286-315）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fill_with_path() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(50.0, 10.0);
    path.line_to(50.0, 50.0);
    path.close_path();
    ctx.fill_with_path(&path);
}

#[test]
fn test_stroke_with_path() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(80.0, 80.0);
    ctx.stroke_with_path(&path);
}

// ═══════════════════════════════════════════════════════════════════════
// arc / ellipse / quadratic / bezier（lines 200-247）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_arc() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI);
}

#[test]
fn test_arc_to() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.arc_to(50.0, 10.0, 50.0, 50.0, 10.0);
}

#[test]
fn test_ellipse() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI * 2.0);
}

#[test]
fn test_quadratic_curve_to() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.quadratic_curve_to(50.0, 0.0, 90.0, 10.0);
}

#[test]
fn test_bezier_curve_to() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.bezier_curve_to(30.0, 0.0, 70.0, 20.0, 90.0, 10.0);
}
