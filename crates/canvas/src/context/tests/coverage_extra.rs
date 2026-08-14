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

// R34xx：drawImage 阴影（WPT 2d.shadow.image.* / 2d.shadow.canvas.*）——源图 alpha 构
// shadow mask + offset/blur 合成，再画图像本身。CPU 像素路径（GPU 路径经像素缓冲上传）。
#[test]
fn test_draw_image_shadow_offset() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(255, 0, 0)));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    let img = ImageData {
        width: 100,
        height: 50,
        data: vec![255u8; 100 * 50 * 4],
    };
    ctx.set_shadow_color(Color::rgb(0, 255, 0));
    ctx.set_shadow_offset_y(50.0);
    // 源图绘于 (0,-50)：阴影（绿）落于 (0,0)..(100,50)。
    ctx.draw_image(&img, 0.0, -50.0);
    let p = ctx.get_image_data(50, 25, 1, 1);
    assert_eq!(
        (p.data[0], p.data[1], p.data[2]),
        (0, 255, 0),
        "shadow pixel should be green"
    );
}

#[test]
fn test_draw_image_shadow_respects_source_alpha() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(255, 0, 0)));
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // 半透明源（alpha=128）→ 阴影 alpha 减半：合成后绿通道 ~128（WPT shadow.image.alpha）。
    let mut img = ImageData {
        width: 10,
        height: 10,
        data: vec![0u8; 10 * 10 * 4],
    };
    for px in img.data.chunks_mut(4) {
        px[0] = 255;
        px[3] = 128;
    }
    ctx.set_shadow_color(Color::rgb(0, 255, 0));
    ctx.set_shadow_offset_y(50.0);
    ctx.draw_image(&img, 0.0, -50.0);
    let p = ctx.get_image_data(5, 5, 1, 1);
    // 源 alpha 128/255 → 阴影绿贡献 ≈ (0, 128, 0) 混合红底 → g ∈ (0, 255) 之间（非全强度）。
    assert!(
        p.data[1] > 0 && p.data[1] < 255,
        "shadow should be semi-transparent green, got g={}",
        p.data[1]
    );
}

#[test]
fn test_draw_image_shadow_transparent_source_no_shadow() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(255, 0, 0)));
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // 全透明源 → 无阴影（WPT 2d.shadow.image.transparent.2：透明部分不画阴影）。
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![0u8; 10 * 10 * 4],
    };
    ctx.set_shadow_color(Color::rgb(0, 255, 0));
    ctx.set_shadow_offset_y(50.0);
    ctx.draw_image(&img, 0.0, -50.0);
    let p = ctx.get_image_data(5, 25, 1, 1);
    assert_eq!(
        (p.data[0], p.data[1], p.data[2]),
        (255, 0, 0),
        "no shadow for transparent source"
    );
}

#[test]
fn test_draw_image_shadow_scaled_sliced() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(255, 0, 0)));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    let img = ImageData {
        width: 50,
        height: 25,
        data: vec![255u8; 50 * 25 * 4],
    };
    ctx.set_shadow_color(Color::rgb(0, 255, 0));
    ctx.set_shadow_offset_y(50.0);
    // 9 参重载：源切片 (0,0,50,25) → 目标 (-10,-50,240,50)（WPT 2d.shadow.image.scale）。
    ctx.draw_image_sliced(&img, 0.0, 0.0, 50.0, 25.0, -10.0, -50.0, 240.0, 50.0);
    let p = ctx.get_image_data(50, 25, 1, 1);
    assert_eq!((p.data[0], p.data[1], p.data[2]), (0, 255, 0), "scaled shadow pixel");
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
    ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI, false);
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

// R34xx：真字体光栅路径（headless @font-face 注入的 FontLoader）——fill_text 像素落盘 +
// measure_text 真度量。CanvasTest.ttf 资产位于 tests/wpt-runner/wpt-data/fonts/。
#[test]
fn test_fill_text_real_font_rasterization() {
    use std::sync::{Arc, Mutex};
    let bytes = std::fs::read("/lzcapp/document/work/ZeroWeb-2/tests/wpt-runner/wpt-data/fonts/CanvasTest.ttf")
        .unwrap_or_else(|_| Vec::new());
    if bytes.is_empty() {
        // 资产缺失（非 wpt-runner 环境）→ 跳过（不 panic）。
        return;
    }
    let mut loader = zero_render_foundation::font::loader::FontLoader::new();
    let fid = loader.load_font(&bytes).unwrap();
    loader.register_family_alias("CanvasTest", fid);
    let loader = Arc::new(Mutex::new(loader));
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_font_loader(Some(loader));
    ctx.set_font(FontDescriptor {
        family: "CanvasTest".to_string(),
        size: 50.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        small_caps: false,
        weight_value: None,
        letter_spacing: "0px".to_string(),
        word_spacing: "0px".to_string(),
        kerning_none: false,
    });
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(0, 255, 0)));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(255, 0, 0)));
    ctx.fill_text("CC", 0.0, 37.5, None);
    // CanvasTest 'C' 为竖条字形：墨水覆盖 (5,5)..(95,45)（WPT baseline.alphabetic 同断言）。
    for (x, y) in [(5, 5), (25, 25), (5, 45), (75, 25), (95, 45)] {
        let p = ctx.get_image_data(x, y, 1, 1);
        assert_eq!(
            (p.data[0], p.data[1]),
            (255, 0),
            "text ink at ({x},{y}) should be red, got r={} g={}",
            p.data[0],
            p.data[1]
        );
    }
    // measureText 真度量：CanvasTest advance = 1em/字符。
    let m = ctx.measure_text("CC");
    assert!((m.width - 100.0).abs() < 1.0, "advance 2em, got {}", m.width);
}

// R34xx：letterSpacing 相对单位随字号重解析（change.font 语义）。
#[test]
fn test_letter_spacing_em_reresolves_on_font_change() {
    let mut ctx = CanvasContext::new(200, 50);
    ctx.set_font(FontDescriptor {
        family: "sans-serif".to_string(),
        size: 10.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        small_caps: false,
        weight_value: None,
        letter_spacing: "1em".to_string(),
        word_spacing: "0px".to_string(),
        kerning_none: false,
    });
    let m10 = ctx.measure_text("hello");
    assert!((m10.width - (5.0 * 6.0 + 10.0 * 5.0)).abs() < 0.01, "10px: 1em=10px");
    ctx.set_font(FontDescriptor {
        family: "sans-serif".to_string(),
        size: 20.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        small_caps: false,
        weight_value: None,
        letter_spacing: "1em".to_string(),
        word_spacing: "0px".to_string(),
        kerning_none: false,
    });
    let m20 = ctx.measure_text("hello");
    assert!((m20.width - (5.0 * 12.0 + 20.0 * 5.0)).abs() < 0.01, "20px: 1em=20px");
}

// R34xx：fontKerning 'none' 关 kern 特征（2d.text.drawing.style.fontKerning 驱动——
// 'TAWATAVA' 在 kerned 字体（Lato-Medium，GPOS kern）下 normal 宽度 < none）。
// Lato 资产位于 tests/wpt-runner/fonts/。
#[test]
fn test_font_kerning_none_widens_measure() {
    use std::sync::{Arc, Mutex};
    let manifest = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{manifest}/../../tests/wpt-runner/fonts/Lato-Medium.ttf"))
        .unwrap_or_else(|_| Vec::new());
    if bytes.is_empty() {
        return; // 资产缺失（非 wpt-runner 环境）→ 跳过。
    }
    let mut loader = zero_render_foundation::font::loader::FontLoader::new();
    let fid = loader.load_font(&bytes).unwrap();
    loader.register_family_alias("Lato", fid);
    let loader = Arc::new(Mutex::new(loader));
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_font_loader(Some(loader));
    ctx.set_font(FontDescriptor {
        family: "Lato".to_string(),
        size: 20.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        small_caps: false,
        weight_value: None,
        letter_spacing: "0px".to_string(),
        word_spacing: "0px".to_string(),
        kerning_none: false,
    });
    let kerned = ctx.measure_text("TAWATAVA").width;
    ctx.set_font_kerning("none");
    let none = ctx.measure_text("TAWATAVA").width;
    assert!(
        kerned < none,
        "fontKerning normal ({kerned}) should be narrower than none ({none})"
    );
    // 'none' 状态不污染后续：恢复 normal 后宽度回到 kerned。
    ctx.set_font_kerning("normal");
    let back = ctx.measure_text("TAWATAVA").width;
    assert_eq!(back, kerned);
}

// R34xx：fontKerning 状态往返（set_font_kerning → font() 描述符；'none' 置位，
// 其他值清位——reset.fontKerning.none 的跨字体保持由 engine 桥 setFont op 继承实现）。
#[test]
fn test_font_kerning_state_roundtrip() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_font(FontDescriptor::default());
    assert!(!ctx.font().kerning_none, "默认 kern 开");
    ctx.set_font_kerning("none");
    assert!(ctx.font().kerning_none, "'none' 置位");
    ctx.set_font_kerning("normal");
    assert!(!ctx.font().kerning_none, "'normal' 清位");
    ctx.set_font_kerning("auto");
    assert!(!ctx.font().kerning_none, "'auto' 清位");
}

// R34xx：零长渐变 + fillText 不绘制（2d.gradient.interpolate.zerosize.fillText 驱动——
// x0==x1&&y0==y1 的 linear gradient 在文本路径 sample_at → 透明；midpoint resolve 会取 stop 色）。
#[test]
fn test_fill_text_zero_size_gradient_paints_nothing() {
    use crate::context::types::{CanvasStyle, LinearGradient};
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgb(0, 255, 0)));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    let mut g = LinearGradient::new(50.0, 25.0, 50.0, 25.0); // 零长（undefined direction）
    g.add_color_stop(0.0, Color::rgb(255, 0, 0));
    g.add_color_stop(1.0, Color::rgb(255, 0, 0));
    ctx.set_fill_style(CanvasStyle::LinearGradient(g));
    ctx.fill_text("AA", 0.0, 50.0, None);
    let p = ctx.get_image_data(25, 25, 1, 1);
    assert_eq!((p.data[0], p.data[1]), (0, 255), "零长渐变不画文本（保持绿底）");
}

// R34xx：ASCII whitespace → U+0020 预处理（2d.text.measure.actualBoundingBox.whitespace
// 驱动——'\t' 在 CanvasTest 有自带墨迹字形，转换后与 space 同墨迹；tab 期望 |Left|≥49）。
#[test]
fn test_prepare_canvas_text_whitespace_conversion() {
    use crate::context::context_impl::prepare_canvas_text;
    assert_eq!(prepare_canvas_text("a\tb"), "a b");
    assert_eq!(prepare_canvas_text("a\nb\rc\x0Cd"), "a b c d");
    assert_eq!(prepare_canvas_text("a b"), "a b"); // U+0020 不变
    assert_eq!(prepare_canvas_text("a\u{3000}b"), "a\u{3000}b"); // 非 ASCII whitespace 不变
    assert_eq!(prepare_canvas_text("a\u{0}b"), "ab"); // null 剥离
}

// R34xx：actualBoundingBoxLeft/Right 符号约定（spec：Left 正值=向左、Right 正值=向右，
// 不钳制）——墨迹在原点右侧时 Left 为负（' A' 期望 |Left|≥49）。
#[test]
fn test_measure_bbox_left_negative_when_ink_right_of_origin() {
    let bytes = std::fs::read(format!(
        "{}/../../tests/wpt-runner/wpt-data/fonts/CanvasTest.ttf",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|_| Vec::new());
    if bytes.is_empty() {
        return;
    }
    use std::sync::{Arc, Mutex};
    let mut loader = zero_render_foundation::font::loader::FontLoader::new();
    let fid = loader.load_font(&bytes).unwrap();
    loader.register_family_alias("CanvasTest", fid);
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_font_loader(Some(Arc::new(Mutex::new(loader))));
    ctx.set_font(FontDescriptor {
        family: "CanvasTest".to_string(),
        size: 50.0,
        ..FontDescriptor::default()
    });
    let m = ctx.measure_text(" A");
    assert!(
        m.actual_bounding_box_left.abs() >= 49.0,
        "' A' 墨迹左缘应在原点右侧 1em 附近，got {}",
        m.actual_bounding_box_left
    );
    // 首字即墨迹（'A'）：left ≈ 0。
    let m2 = ctx.measure_text("A");
    assert!(m2.actual_bounding_box_left.abs() <= 1.0);
}

// R34xx：font-variant small-caps 合成（字体无 smcp 特征 → 大写 shaping——
// 2d.text.fontVariantCaps2.worker 驱动：small-caps 与 normal 的 measure 宽度不同）。
#[test]
fn test_small_caps_changes_measure_width() {
    let paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
    ];
    let mut loader = zero_render_foundation::font::loader::FontLoader::new();
    let mut loaded = false;
    for p in paths {
        if let Ok(bytes) = std::fs::read(p)
            && let Ok(id) = loader.load_font(&bytes)
        {
            loader.register_family_alias("serif", id);
            loaded = true;
            break;
        }
    }
    if !loaded {
        return; // 无系统字体环境跳过。
    }
    use std::sync::{Arc, Mutex};
    let mut ctx = CanvasContext::new(200, 50);
    ctx.set_font_loader(Some(Arc::new(Mutex::new(loader))));
    ctx.set_font(FontDescriptor {
        family: "serif".to_string(),
        size: 32.0,
        small_caps: true,
        ..FontDescriptor::default()
    });
    let sc = ctx.measure_text("Hello World").width;
    ctx.set_font(FontDescriptor {
        family: "serif".to_string(),
        size: 32.0,
        small_caps: false,
        ..FontDescriptor::default()
    });
    let n = ctx.measure_text("Hello World").width;
    assert_ne!(sc, n, "small-caps 与 normal 宽度须不同");
}
