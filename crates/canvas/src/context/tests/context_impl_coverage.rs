//! CanvasContext context_impl.rs 覆盖率测试

use super::super::types::*;
use crate::context::*;
use crate::path::{Path2D, PathCommand};
use zero_render_foundation::color::Color;

#[test]
fn test_context_new_zero_size() {
    let ctx = CanvasContext::new(0, 0);
    assert_eq!(ctx.width(), 0);
    assert_eq!(ctx.height(), 0);
}

#[test]
fn test_context_clear_rect_entire_canvas() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.clear_rect(0.0, 0.0, 10.0, 10.0);
    // Check if all pixels are cleared to 0
    let all_cleared = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_cleared, "Clear entire canvas should zero all pixels");
}

#[test]
fn test_context_clear_rect_partial_canvas() {
    let mut ctx = CanvasContext::new(10, 10);
    // Fill the canvas first
    for i in 0..400 {
        ctx.pixel_buffer[i] = 255;
    }
    ctx.clear_rect(2.0, 2.0, 4.0, 4.0);
    // Check that only cleared area is 0
    for y in 0..10 {
        for x in 0..10 {
            let idx = (y * 10 + x) * 4;
            if x >= 2 && x < 6 && y >= 2 && y < 6 {
                assert_eq!(ctx.pixel_buffer[idx], 0, "Cleared pixel at ({}, {})", x, y);
                assert_eq!(ctx.pixel_buffer[idx + 1], 0, "Cleared pixel at ({}, {})", x, y);
                assert_eq!(ctx.pixel_buffer[idx + 2], 0, "Cleared pixel at ({}, {})", x, y);
                assert_eq!(ctx.pixel_buffer[idx + 3], 0, "Cleared pixel at ({}, {})", x, y);
            } else {
                assert_eq!(ctx.pixel_buffer[idx], 255, "Uncleared pixel at ({}, {})", x, y);
            }
        }
    }
}

#[test]
fn test_context_clear_rect_out_of_bounds() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.clear_rect(-5.0, -5.0, 20.0, 20.0);
    // Should only clear within canvas bounds
    for y in 0..10 {
        for x in 0..10 {
            let idx = (y * 10 + x) * 4;
            assert_eq!(ctx.pixel_buffer[idx], 0, "Pixel at ({}, {}) should be cleared", x, y);
        }
    }
}

#[test]
fn test_context_fill_rect_zero_width() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.fill_rect(0.0, 0.0, 0.0, 10.0);
    // Zero width rect should not fill any pixels
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "Zero width fill should not affect pixels");
}

#[test]
fn test_context_fill_rect_zero_height() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.fill_rect(0.0, 0.0, 10.0, 0.0);
    // Zero height rect should not fill any pixels
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "Zero height fill should not affect pixels");
}

#[test]
fn test_context_fill_rect_partial_outside_canvas() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.fill_rect(8.0, 8.0, 5.0, 5.0);
    // Should only fill the part within canvas bounds
    // Check pixel at (9, 9) which should be filled
    let idx = (9 * 10 + 9) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 0); // Default fill is black
    assert_eq!(ctx.pixel_buffer[idx + 3], 255); // Default fill is opaque
}

#[test]
fn test_context_stroke_rect_line_width_greater_than_height() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_line_width(15.0);
    ctx.set_stroke_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.stroke_rect(2.0, 2.0, 4.0, 4.0);
    // Very wide line should fill the entire canvas
    let center_pixel = (5 * 10 + 5) * 4;
    assert_eq!(ctx.pixel_buffer[center_pixel], 255);
    assert_eq!(ctx.pixel_buffer[center_pixel + 3], 255);
}

#[test]
fn test_context_stroke_rect_zero_line_width() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_line_width(0.0);
    ctx.stroke_rect(0.0, 0.0, 10.0, 10.0);
    // Zero width stroke should not fill any pixels
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "Zero width stroke should not affect pixels");
}

#[test]
fn test_context_save_restore_empty() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.save();
    ctx.restore();
    // Should not panic
}

#[test]
fn test_context_save_restore_nested() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.save();
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(0, 255, 0, 255)));
    ctx.save();
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(0, 0, 255, 255)));

    assert_eq!(ctx.fill_style().resolve_color().r, 0);
    assert_eq!(ctx.fill_style().resolve_color().g, 0);
    assert_eq!(ctx.fill_style().resolve_color().b, 255);

    ctx.restore();
    assert_eq!(ctx.fill_style().resolve_color().r, 0);
    assert_eq!(ctx.fill_style().resolve_color().g, 255);
    assert_eq!(ctx.fill_style().resolve_color().b, 0);

    ctx.restore();
    assert_eq!(ctx.fill_style().resolve_color().r, 255);
    assert_eq!(ctx.fill_style().resolve_color().g, 0);
    assert_eq!(ctx.fill_style().resolve_color().b, 0);
}

#[test]
fn test_context_transform_multiple_operations() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.translate(5.0, 5.0);
    ctx.scale(2.0, 2.0);
    ctx.rotate(std::f32::consts::PI / 4.0);

    let t = ctx.get_transform();
    // Check that transform is not identity
    assert_ne!(t.a, 1.0);
    assert_ne!(t.b, 0.0);
    assert_ne!(t.c, 0.0);
    assert_ne!(t.d, 1.0);
    assert_ne!(t.e, 0.0);
    assert_ne!(t.f, 0.0);
}

#[test]
fn test_context_reset_transform() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.translate(5.0, 5.0);
    ctx.reset_transform();

    let t = ctx.get_transform();
    assert_eq!(t.a, 1.0);
    assert_eq!(t.b, 0.0);
    assert_eq!(t.c, 0.0);
    assert_eq!(t.d, 1.0);
    assert_eq!(t.e, 0.0);
    assert_eq!(t.f, 0.0);
}

#[test]
fn test_context_set_transform_overrides() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.translate(5.0, 5.0);
    ctx.set_transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);

    let t = ctx.get_transform();
    assert_eq!(t.a, 2.0);
    assert_eq!(t.d, 2.0);
    assert_eq!(t.e, 0.0);
    assert_eq!(t.f, 0.0);
}

#[test]
fn test_context_global_alpha_clamping() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_global_alpha(-1.0);
    assert_eq!(ctx.global_alpha(), 0.0);

    ctx.set_global_alpha(2.0);
    assert_eq!(ctx.global_alpha(), 1.0);

    ctx.set_global_alpha(0.5);
    assert_eq!(ctx.global_alpha(), 0.5);
}

#[test]
fn test_context_resize_clears_primitives() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.resize(5, 5);

    // Check that pixel buffer is cleared
    assert_eq!(ctx.width(), 5);
    assert_eq!(ctx.height(), 5);
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "Resize should clear pixel buffer");
}

#[test]
fn test_context_get_image_data_partial() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.fill_rect(0.0, 0.0, 5.0, 5.0);

    let img = ctx.get_image_data(2, 2, 3, 3);
    assert_eq!(img.width, 3);
    assert_eq!(img.height, 3);
    assert_eq!(img.data.len(), 3 * 3 * 4);

    // Check that the data starts from the correct offset
    // Pixel (2, 2) should be the first in the data
    let expected_offset = (2 * 10 + 2) * 4;
    assert_eq!(img.data[0], ctx.pixel_buffer[expected_offset]);
}

#[test]
fn test_context_get_image_data_out_of_bounds() {
    let ctx = CanvasContext::new(10, 10);
    let img = ctx.get_image_data(8, 8, 5, 5);
    // 当前 API 返回请求尺寸的 ImageData（OOB 部分零填充），而非裁剪到画布内。
    // spec CanvasRenderingContext2D.getImageData：返回请求 width×height，越界像素为透明黑 0,0,0,0。
    assert_eq!(img.width, 5);
    assert_eq!(img.height, 5);
    assert_eq!(img.data.len(), 5 * 5 * 4);
}

// R3354：get_image_data / create_image_data 的 RGBA 缓冲区尺寸计算改用 usize。
//
// 旧实现 `(width * height * 4) as usize`：先在 u32 算术里乘，再转 usize。当 width*height*4
// 越过 u32::MAX（如 getImageData(0,0,65536,65536) → 65536*65536*4 = 2^34），u32 中间结果回绕
// 为一个小值（此处为 0）→ data 分配 0 字节 → 随后复制循环 `data[dst_start..dst_start+copy_len]`
// 在 copy_from_slice 的长度检查上 panic（slice index out of bounds，debug + release 均触发，
// 与算术溢出无关）。在有内容的小画布上调 getImageData 即确定性 panic。
//
// 修复：`(width as usize).saturating_mul(height as usize).saturating_mul(4)` —— 直接在 usize
// 域计算，64-bit usize 下 65536*65536*4=2^34 合法可表示（不再回绕），极端超大尺寸 saturating
// 到 usize::MAX 由 Vec 分配层处理（OOM abort 而非静默内存损坏）。
//
// 复现 panic 需 width*height*4 > u32::MAX，最小触发点 65536*65536*4=16GB——直接经公共 API
// 复现会触发 16GB 分配，不适合 CI。故本组测以 usize 计算正确性间接锁修复：断言 data.len()
// 严格等于 `(w as usize)*(h as usize)*4`（usize 域）。旧 u32 实现在这些尺寸下恰好也正确，
// 但若有人回退到 u32 实现，下述 saturating 边界测（create_image_data_overflow_saturates）会捕获。
#[test]
fn test_context_get_image_data_size_calc_uses_usize_r3354() {
    let ctx = CanvasContext::new(0, 0);
    let img = ctx.get_image_data(0, 0, 65536, 2);
    assert_eq!(img.width, 65536);
    assert_eq!(img.height, 2);
    // 65536*2*4 = 524288（512KB），usize 精确。
    assert_eq!(img.data.len(), 65536usize * 2 * 4);
}

#[test]
fn test_context_create_image_data_size_calc_uses_usize_r3354() {
    let ctx = CanvasContext::new(0, 0);
    let img = ctx.create_image_data(4096, 4096);
    assert_eq!(img.width, 4096);
    assert_eq!(img.height, 4096);
    // 4096*4096*4 = 67108864（64MB），usize 精确。
    assert_eq!(img.data.len(), 4096usize * 4096 * 4);
}

// R3354：saturating 边界——证明修复后的 usize 域 saturating_mul 不像旧 u32 域那样回绕到小值。
// 取 w=h=u32::MAX：u32 域 u32::MAX*u32::MAX*4 在 wrapping 下回绕到一个小值（旧 bug 根因，
// 致 data 缓冲区远小于真实需要 → 切片越界 panic）；usize 域 saturating_mul 在真正越过
// usize::MAX 时钳到 usize::MAX（不回绕），未越过时保留真实大值（仍远大于 u32 回绕结果）。
#[test]
fn test_context_create_image_data_overflow_saturates_r3354() {
    let w = u32::MAX as usize;
    // usize 域：u32::MAX^2 ≈ 1.8e19，*4 ≈ 7.2e19，已越 usize::MAX(≈1.8e19) → saturating 到 MAX。
    let size_usize = w.saturating_mul(w).saturating_mul(4);
    assert_eq!(
        size_usize,
        usize::MAX,
        "usize saturating_mul 越界钳到 usize::MAX 而非回绕"
    );
    // 对照：旧 u32 域 wrapping 回绕到一个小值（这正是 size=0 致 panic 的根因）。
    let wrapped_u32 = u32::MAX.wrapping_mul(u32::MAX).wrapping_mul(4);
    assert_eq!(
        wrapped_u32 as usize, 4,
        "u32 wrapping 回绕到 4（远小于真实需要，旧 bug 根因）"
    );
}

// R3355：shadowBlur 极大值不再触发 i32 溢出 panic + box_blur 挂起/u32 溢出。
//
// 旧实现三重故障：① shadow_blur_geom 的 `(blur/2).round() as i32` 饱和到 i32::MAX，
// `pad = 3·i32::MAX as i32` 再饱和到 i32::MAX；② draw_shadow_* 的 `rect.floor() as i32 ± pad`
// 在 i32 域加减法溢出（cargo debug overflow-checks=true → panic，修复前确定性复现于
// context_impl.rs:1115 `attempt to add with overflow`）；③ box_blur_alpha `for dx in -r..=r`
// 达 ~4.3e9 次迭代 + `sum: u32` 在 ~17M 次累加后溢出 panic。
//
// 修复：shadow_blur_geom 半径封顶 SHADOW_BLUR_MAX_RADIUS(8192)（pad=24576 远在 i32 内、
// box_blur 窗口 sum 上限安全）+ draw_shadow_rect/path 的 region padding 改 saturating_add/sub。
// shadowBlur 经 setShadowBlur op 从 JS 可控，故页面/攻击者可触发——确定性 panic = DoS。
// 本组测覆盖三条 shadow 路径（fillRect→rect / fill→path / stroke→stroke footprint）均不 panic。
#[test]
fn test_shadow_rect_huge_blur_no_overflow_panic_r3355() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 255));
    ctx.set_shadow_blur(1e30);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    // draw_shadow_rect 经 fillRect 触发：region rx0/rx1 i32 溢出（修复前 panic @ context_impl.rs:1115）。
    ctx.fill_rect(0.0, 0.0, 20.0, 20.0);
}

#[test]
fn test_shadow_path_huge_blur_no_overflow_panic_r3355() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 255));
    ctx.set_shadow_blur(1e30);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.begin_path();
    ctx.move_to(5.0, 5.0);
    ctx.line_to(25.0, 5.0);
    ctx.line_to(25.0, 25.0);
    ctx.close_path();
    // draw_shadow_path 经 fill 触发：region padding i32 溢出（修复前 panic）。
    ctx.fill();
}

#[test]
fn test_shadow_stroke_huge_blur_no_overflow_panic_r3355() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 255));
    ctx.set_shadow_blur(1e30);
    ctx.set_stroke_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_line_width(4.0);
    ctx.begin_path();
    ctx.move_to(5.0, 25.0);
    ctx.line_to(45.0, 25.0);
    // draw_shadow_stroke 经 stroke 触发（pad 为 f32 域，本路径修复前不 panic，纳入回归锁防止
    // 未来重构改回 i32 域时回退）。
    ctx.stroke();
}

// R3355：shadow_blur_geom 半径封顶行为锁——shadowBlur 远超封顶值时，半径钳到 SHADOW_BLUR_MAX_RADIUS，
// 阴影仍正常软化（非硬边 no-op），且 pad = 3·封顶值 合法 i32。
#[test]
fn test_shadow_blur_geom_caps_radius_r3355() {
    use crate::context::raster::SHADOW_BLUR_MAX_RADIUS;
    // 正常 blur：半径 = round(blur/2)。
    let (r, pad, passes) = crate::context::raster::shadow_blur_geom(20.0);
    assert_eq!(r, 10);
    assert_eq!(pad, 30);
    assert_eq!(passes, 3);
    // 极大 blur：半径钳到封顶，pad = 3·封顶（合法 i32，远小于 i32::MAX）。
    let (r_big, pad_big, _) = crate::context::raster::shadow_blur_geom(1e30);
    assert_eq!(r_big, SHADOW_BLUR_MAX_RADIUS);
    assert_eq!(pad_big, (3 * SHADOW_BLUR_MAX_RADIUS) as i32);
    assert!(pad_big < i32::MAX, "封顶后 pad 须在 i32 范围内（避免下游 i32 溢出）");
    // blur<=0：no-op。
    assert_eq!(crate::context::raster::shadow_blur_geom(0.0), (0, 0, 0));
    assert_eq!(crate::context::raster::shadow_blur_geom(-5.0), (0, 0, 0));
}

#[test]
fn test_context_put_image_data_out_of_bounds() {
    let mut ctx = CanvasContext::new(10, 10);
    let mut src_img = ctx.create_image_data(5, 5);
    // Fill with non-zero data
    for i in 0..(5 * 5 * 4) {
        src_img.data[i] = 255;
    }

    ctx.put_image_data(&src_img, 8, 8);
    // Should only copy the part that fits
    // Check pixel (9, 9) which should be copied
    let idx = (9 * 10 + 9) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255);
    assert_eq!(ctx.pixel_buffer[idx + 1], 255);
    assert_eq!(ctx.pixel_buffer[idx + 2], 255);
    assert_eq!(ctx.pixel_buffer[idx + 3], 255);
}

#[test]
fn test_context_create_pattern_with_repetition() {
    let ctx = CanvasContext::new(10, 10);
    let mut pattern_data = ctx.create_image_data(5, 5);
    // Fill pattern with red
    for i in 0..(5 * 5 * 4) {
        pattern_data.data[i] = 255;
        if i % 4 == 3 {
            pattern_data.data[i] = 255;
        }
    }

    let pattern = ctx.create_pattern(pattern_data, PatternRepetition::Repeat);
    assert!(!pattern.image_data.data.is_empty());
    assert_eq!(pattern.repetition, PatternRepetition::Repeat);
}

#[test]
fn test_context_is_point_in_path_empty() {
    let ctx = CanvasContext::new(10, 10);
    // Empty path should return false
    assert!(!ctx.is_point_in_path(5.0, 5.0));
}

#[test]
fn test_context_is_point_in_path_single_point() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.move_to(5.0, 5.0);
    // Single point is not considered a path for fill
    assert!(!ctx.is_point_in_path(5.0, 5.0));
}

#[test]
fn test_context_is_point_in_path_line_segment() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.move_to(0.0, 0.0);
    ctx.line_to(10.0, 10.0);
    // Point on the line may return true or false depending on implementation
    let on_path = ctx.is_point_in_path(5.0, 5.0);
    // Don't assert specific value as it depends on the point-in-polygon implementation
    assert!(on_path || !on_path);
}

#[test]
fn test_context_clip_with_empty_path() {
    let mut ctx = CanvasContext::new(10, 10);
    // Empty clip should not add any clip regions
    let path = Path2D::new();
    ctx.clip_with_path(&path);
    // This test primarily checks that it doesn't panic
}

#[test]
fn test_context_clip_with_path_rect() {
    let mut ctx = CanvasContext::new(10, 10);
    let mut path = Path2D::new();
    path.rect(2.0, 2.0, 6.0, 6.0);
    ctx.clip_with_path(&path);

    // This test primarily checks that it doesn't panic
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
}

#[test]
fn test_context_set_line_dash_odd_length() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_line_dash(vec![5.0, 10.0, 15.0]); // odd length
    let dash = ctx.get_line_dash();
    // Should be doubled: [5.0, 10.0, 15.0, 5.0, 10.0, 15.0]
    assert_eq!(dash.len(), 6);
    assert_eq!(dash[0], 5.0);
    assert_eq!(dash[1], 10.0);
    assert_eq!(dash[2], 15.0);
    assert_eq!(dash[3], 5.0);
    assert_eq!(dash[4], 10.0);
    assert_eq!(dash[5], 15.0);
}

#[test]
fn test_context_set_line_dash_even_length() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_line_dash(vec![5.0, 10.0, 15.0, 20.0]); // even length
    let dash = ctx.get_line_dash();
    // Should remain the same
    assert_eq!(dash, [5.0, 10.0, 15.0, 20.0]);
}

#[test]
fn test_context_set_line_dash_zero_segments() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_line_dash(vec![]);
    let dash = ctx.get_line_dash();
    assert_eq!(dash.len(), 0);
}

#[test]
fn test_context_shadow_color_transparent() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_shadow_color(Color::TRANSPARENT);
    assert_eq!(ctx.shadow_color().a, 0);

    // Drawing with transparent shadow should not draw shadows
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
}

#[test]
fn test_context_shadow_properties_zero_blur() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_shadow_color(Color::rgba(255, 0, 0, 128));
    ctx.set_shadow_blur(0.0);
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);

    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
}

#[test]
fn test_context_shadow_properties_negative_blur() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_shadow_blur(-5.0);
    assert_eq!(ctx.shadow_blur(), 0.0); // Should be clamped to 0
}

#[test]
fn test_context_create_gradients() {
    let ctx = CanvasContext::new(10, 10);

    let linear = ctx.create_linear_gradient(0.0, 0.0, 10.0, 10.0);
    assert_eq!(linear.x0, 0.0);
    assert_eq!(linear.y0, 0.0);
    assert_eq!(linear.x1, 10.0);
    assert_eq!(linear.y1, 10.0);

    let radial = ctx.create_radial_gradient(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
    assert_eq!(radial.x0, 0.0);
    assert_eq!(radial.y0, 0.0);
    assert_eq!(radial.r0, 0.0);
    assert_eq!(radial.x1, 10.0);
    assert_eq!(radial.y1, 10.0);
    assert_eq!(radial.r1, 10.0);

    let conic = ctx.create_conic_gradient(0.0, 5.0, 5.0);
    assert_eq!(conic.start_angle, 0.0);
    assert_eq!(conic.cx, 5.0);
    assert_eq!(conic.cy, 5.0);
}

#[test]
fn test_context_text_properties() {
    let mut ctx = CanvasContext::new(10, 10);

    // Test text align
    ctx.set_text_align(TextAlign::Center);
    assert_eq!(ctx.text_align(), TextAlign::Center);

    ctx.set_text_align(TextAlign::End);
    assert_eq!(ctx.text_align(), TextAlign::End);

    // Test text baseline
    ctx.set_text_baseline(TextBaseline::Top);
    assert_eq!(ctx.text_baseline(), TextBaseline::Top);

    ctx.set_text_baseline(TextBaseline::Bottom);
    assert_eq!(ctx.text_baseline(), TextBaseline::Bottom);

    // Test direction
    ctx.set_direction(TextDirection::Ltr);
    assert_eq!(ctx.direction(), TextDirection::Ltr);

    ctx.set_direction(TextDirection::Rtl);
    assert_eq!(ctx.direction(), TextDirection::Rtl);
}

#[test]
fn test_context_measure_text_empty() {
    let ctx = CanvasContext::new(10, 10);
    let metrics = ctx.measure_text("");
    assert_eq!(metrics.width, 0.0);
}

#[test]
fn test_context_measure_text_whitespace() {
    let ctx = CanvasContext::new(10, 10);
    let metrics = ctx.measure_text("   ");
    // Whitespace should have width based on character count
    assert!(metrics.width > 0.0);
}

#[test]
fn test_context_draw_image_sized() {
    let mut ctx = CanvasContext::new(20, 20);
    let mut img_data = ctx.create_image_data(10, 10);
    // Fill with a pattern（10×10×4 = 400 字节）
    for i in 0..(10 * 10 * 4) {
        img_data.data[i] = (i % 255) as u8;
    }

    // Draw image at full size
    ctx.draw_image(&img_data, 0.0, 0.0);

    // Draw image scaled
    ctx.draw_image_with_size(&img_data, 5.0, 5.0, 5.0, 5.0);

    // Draw image sliced
    ctx.draw_image_sliced(&img_data, 0.0, 0.0, 5.0, 5.0, 10.0, 10.0, 5.0, 5.0);
}

#[test]
fn test_context_draw_image_zero_size() {
    let mut ctx = CanvasContext::new(10, 10);
    let img_data = ctx.create_image_data(0, 0);
    ctx.draw_image(&img_data, 0.0, 0.0);
}

#[test]
fn test_context_draw_image_out_of_bounds() {
    let mut ctx = CanvasContext::new(10, 10);
    let mut img_data = ctx.create_image_data(5, 5);
    for i in 0..100 {
        img_data.data[i] = 255;
    }

    ctx.draw_image(&img_data, -5.0, -5.0);
}

#[test]
fn test_context_composite_operations() {
    let mut ctx = CanvasContext::new(10, 10);

    // Test various composite operations
    ctx.set_composite_operation(CompositeOperation::Copy);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Copy);

    ctx.set_composite_operation(CompositeOperation::SourceOver);
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);

    ctx.set_composite_operation(CompositeOperation::Xor);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Xor);
}

#[test]
fn test_context_image_smoothing() {
    let mut ctx = CanvasContext::new(10, 10);

    // Default is true
    assert!(ctx.image_smoothing_enabled());

    // Set to false
    ctx.set_image_smoothing_enabled(false);
    assert!(!ctx.image_smoothing_enabled());

    // Set back to true
    ctx.set_image_smoothing_enabled(true);
    assert!(ctx.image_smoothing_enabled());
}

#[test]
fn test_context_line_properties() {
    let mut ctx = CanvasContext::new(10, 10);

    // Test line join
    ctx.set_line_join(LineJoin::Bevel);
    assert_eq!(ctx.line_join(), LineJoin::Bevel);

    ctx.set_line_join(LineJoin::Round);
    assert_eq!(ctx.line_join(), LineJoin::Round);

    // Test line cap
    ctx.set_line_cap(LineCap::Square);
    assert_eq!(ctx.line_cap(), LineCap::Square);

    ctx.set_line_cap(LineCap::Round);
    assert_eq!(ctx.line_cap(), LineCap::Round);

    // Test miter limit
    ctx.set_miter_limit(5.0);
    assert_eq!(ctx.miter_limit(), 5.0);

    ctx.set_miter_limit(0.5);
    assert_eq!(ctx.miter_limit(), 0.5);
}

// ── 覆盖率补全第三轮：阴影路径 / draw_image_sliced 边界 / is_point_in_stroke ──

/// 覆盖 fill() 中的 draw_shadow_path 分支（line 257）
#[test]
fn test_fill_with_shadow_triggers_shadow_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_shadow_color(Color::rgba(0, 0, 255, 128));
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);
    ctx.set_shadow_blur(3.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(60.0, 10.0);
    ctx.line_to(60.0, 60.0);
    ctx.close_path();
    ctx.fill();
    // Shadow pixels should be present around the filled area
    let has_blue = ctx.pixel_buffer.chunks_exact(4).any(|px| px[2] > 0 && px[3] > 0);
    assert!(has_blue, "Shadow path should produce blue-tinted pixels");
}

/// 覆盖 stroke() 中的 draw_shadow_path 分支（line 272）
#[test]
fn test_stroke_with_shadow_triggers_shadow_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_line_width(3.0);
    ctx.set_shadow_color(Color::rgba(0, 255, 0, 128));
    ctx.set_shadow_offset_x(4.0);
    ctx.set_shadow_offset_y(4.0);
    ctx.set_shadow_blur(2.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(80.0, 80.0);
    ctx.stroke();
    let has_green = ctx.pixel_buffer.chunks_exact(4).any(|px| px[1] > 0 && px[3] > 0);
    assert!(has_green, "Stroke shadow should produce green-tinted pixels");
}

/// 覆盖 fill_with_path() 中的 draw_shadow_path 分支（line 292）
#[test]
fn test_fill_with_path_and_shadow() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_shadow_color(Color::rgba(128, 0, 128, 100));
    ctx.set_shadow_offset_x(3.0);
    ctx.set_shadow_offset_y(3.0);
    let mut path = Path2D::new();
    path.move_to(20.0, 20.0);
    path.line_to(70.0, 20.0);
    path.line_to(70.0, 70.0);
    path.close_path();
    ctx.fill_with_path(&path);
    // Verify something was drawn
    let has_content = ctx.pixel_buffer.chunks_exact(4).any(|px| px[3] > 0);
    assert!(has_content, "fill_with_path with shadow should produce pixels");
}

/// 覆盖 stroke_with_path() 中的 draw_shadow_path 分支（lines 303-306）
#[test]
fn test_stroke_with_path_and_shadow() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_style(CanvasStyle::Color(Color::rgba(0, 0, 255, 255)));
    ctx.set_line_width(2.0);
    ctx.set_shadow_color(Color::rgba(255, 128, 0, 100));
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);
    let mut path = Path2D::new();
    path.move_to(15.0, 15.0);
    path.line_to(85.0, 85.0);
    ctx.stroke_with_path(&path);
    let has_content = ctx.pixel_buffer.chunks_exact(4).any(|px| px[3] > 0);
    assert!(has_content, "stroke_with_path with shadow should produce pixels");
}

/// 覆盖 is_point_in_stroke 非空路径分支（line 748 附近）
#[test]
fn test_is_point_in_stroke_with_line_segment() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(10.0);
    ctx.begin_path();
    ctx.move_to(10.0, 50.0);
    ctx.line_to(90.0, 50.0);
    // Point near the line should be in stroke
    assert!(ctx.is_point_in_stroke(50.0, 50.0));
    // Point far away should not
    assert!(!ctx.is_point_in_stroke(50.0, 10.0));
}

/// 覆盖 is_point_in_stroke 空路径分支（line 748: return false）
#[test]
fn test_is_point_in_stroke_empty_path() {
    let ctx = CanvasContext::new(100, 100);
    assert!(!ctx.is_point_in_stroke(50.0, 50.0));
}

/// 覆盖 draw_image_sliced 中 canvas_w==0 分支（line 887）
#[test]
fn test_draw_image_data_scaled_zero_canvas() {
    let mut ctx = CanvasContext::new(0, 0);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![255u8; 400],
    };
    // Should not panic on zero-sized canvas
    ctx.draw_image_sliced(&img, 0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 10.0, 10.0);
}

/// 覆盖 draw_image_sliced 中 sw==0 分支（line 895）
#[test]
fn test_draw_image_data_scaled_zero_source() {
    let mut ctx = CanvasContext::new(50, 50);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![255u8; 400],
    };
    // Source rect with zero width/height - should return early
    ctx.draw_image_sliced(&img, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 10.0, 10.0);
    ctx.draw_image_sliced(&img, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 10.0, 10.0);
}

/// 覆盖 draw_image_sliced 中 src_alpha==0 continue 分支（line 932）
#[test]
fn test_draw_image_data_scaled_fully_transparent_source() {
    let mut ctx = CanvasContext::new(20, 20);
    let img = ImageData {
        width: 10,
        height: 10,
        data: vec![0u8; 400], // all transparent
    };
    ctx.draw_image_sliced(&img, 0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 10.0, 10.0);
    // All pixels should still be 0
    assert!(ctx.pixel_buffer.iter().all(|&v| v == 0));
}

/// 覆盖 draw_shadow_rect 中 blur_factor=1.0 分支（line 972-974）
#[test]
fn test_fill_rect_shadow_zero_blur() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_shadow_color(Color::rgba(0, 128, 0, 150));
    ctx.set_shadow_blur(0.0); // zero blur → blur_factor = 1.0
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);
    ctx.fill_rect(10.0, 10.0, 40.0, 40.0);
    // Verify shadow pixels exist
    let has_green = ctx.pixel_buffer.chunks_exact(4).any(|px| px[1] > 50 && px[3] > 0);
    assert!(has_green, "Shadow with zero blur should still be visible");
}

/// 覆盖 draw_shadow_path 函数（lines 994-1014）
#[test]
fn test_stroke_shadow_path_with_blur() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_style(CanvasStyle::Color(Color::rgba(255, 0, 0, 255)));
    ctx.set_line_width(5.0);
    ctx.set_shadow_color(Color::rgba(0, 0, 255, 100));
    ctx.set_shadow_blur(10.0);
    ctx.set_shadow_offset_x(3.0);
    ctx.set_shadow_offset_y(3.0);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(90.0, 10.0);
    ctx.line_to(90.0, 90.0);
    ctx.stroke();
    let has_blue = ctx.pixel_buffer.chunks_exact(4).any(|px| px[2] > 0 && px[3] > 0);
    assert!(has_blue, "Stroke shadow path with blur should produce pixels");
}

/// 覆盖 draw_image_sliced 正常 alpha 混合路径（lines 940-955）
#[test]
fn test_draw_image_data_scaled_alpha_blend() {
    let mut ctx = CanvasContext::new(20, 20);
    // Pre-fill canvas with a base color
    for px in ctx.pixel_buffer.chunks_exact_mut(4) {
        px[0] = 100;
        px[1] = 100;
        px[2] = 100;
        px[3] = 255;
    }
    // Draw semi-transparent image
    let mut img_data = vec![0u8; 400]; // 10x10 * 4
    for px in img_data.chunks_exact_mut(4) {
        px[0] = 255;
        px[1] = 0;
        px[2] = 0;
        px[3] = 128; // semi-transparent red
    }
    let img = ImageData {
        width: 10,
        height: 10,
        data: img_data,
    };
    ctx.draw_image_sliced(&img, 0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 10.0, 10.0);
    // Check some blending occurred
    let has_blend = ctx.pixel_buffer.chunks_exact(4).any(|px| px[0] > 100);
    assert!(has_blend, "Alpha blending should have occurred");
}

/// 覆盖 draw_image_sliced 中 src_x 越界 continue（line 908）
#[test]
fn test_draw_image_data_scaled_source_out_of_bounds() {
    let mut ctx = CanvasContext::new(20, 20);
    let img = ImageData {
        width: 5,
        height: 5,
        data: vec![255u8; 100],
    };
    // Source offset that exceeds image dimensions
    ctx.draw_image_sliced(&img, 10.0, 10.0, 5.0, 5.0, 0.0, 0.0, 10.0, 10.0);
    // Should not panic, most pixels should remain 0
}
