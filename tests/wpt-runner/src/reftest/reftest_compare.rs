//! Reftest 像素对比与 PNG/PPM I/O —— 帧缓冲间像素差异计算 + 失败诊断落盘。
//!
//! `compare_pixels` / `compare_pixels_labeled` 是 reftest 的核心比对算子；`save_fb_as_png`
//! 与 `save_framebuffer_png` 把帧缓冲落盘供失败诊断与 chromium Oracle 对账使用。

use std::path::Path;

use zero_render_foundation::surface::FrameBuffer;

/// DC-14 非平凡性检查——帧是否「接近纯色」（退化/空白渲染）。
///
/// 每 16 像素采样 1 个，主色占比 > 99.9% 判为近纯色。空帧视为退化。
/// 用于排除退化假绿：test==ref 且都近纯色（如 parsing/animation/print/crashtest
/// headless 空白页）的 reftest 会自源「通过」但无意义，须标可疑单独审计。
pub fn frame_is_near_solid(fb: &FrameBuffer) -> bool {
    let n_px = (fb.width as usize) * (fb.height as usize);
    if n_px == 0 {
        return true; // 空帧视为退化
    }
    let stride = 16; // 每 16 个像素采样 1 个（800×600→30000 样本）
    let mut counts: std::collections::HashMap<[u8; 4], u32> = std::collections::HashMap::new();
    let mut samples = 0u32;
    let mut i = 0usize;
    while i < fb.data.len() {
        let px = [fb.data[i], fb.data[i + 1], fb.data[i + 2], fb.data[i + 3]];
        *counts.entry(px).or_insert(0) += 1;
        samples += 1;
        i += 4 * stride;
    }
    if samples == 0 {
        return true;
    }
    let dominant = counts.values().copied().max().unwrap_or(0) as f64;
    dominant / samples as f64 > 0.999
}

/// 将 FrameBuffer 保存为 PNG 文件（用于失败诊断）。
pub fn save_fb_as_png(fb: &FrameBuffer, path: &Path) {
    use std::io::BufWriter;
    let Ok(file) = std::fs::File::create(path) else {
        return;
    };
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, fb.width, fb.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let Ok(mut writer) = encoder.write_header() else {
        return;
    };
    // FrameBuffer data is RGBA
    let _ = writer.write_image_data(&fb.data);
    let _ = writer.finish();
}

/// 比较两个帧缓冲的像素。
///
/// 返回 (不同像素数, 最大单通道色差)。
pub fn compare_pixels(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8) -> (usize, u8) {
    let (d, m, _subpixel) = compare_pixels_labeled(fb1, fb2, threshold, "");
    (d, m)
}

/// 带标签的像素对比 —— 标签会附加到 REFTEST_BBOX 诊断行，便于定位差异归属。
///
/// 返回 `(diff_pixels, max_channel_diff, subpixel_diff)`：
/// `subpixel_diff` 是通道差**恰好为 1** 的差异像素数（D2 审计建议的诊断维度）——
/// 这类差异通常来自 f32 布局亚像素坐标漂移/AA 抖动（见
/// docs/goal/rendering-compat/evidence/f32-layout-precision-audit-2026-08-07.md），
/// **不参与通过判定**，仅用于量化浮点噪声对 oracle 一致率的影响面。
pub fn compare_pixels_labeled(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8, label: &str) -> (usize, u8, usize) {
    let mut diff_pixels = 0usize;
    let mut subpixel_diff = 0usize;
    let mut max_diff = 0u8;
    // 调试工具：设置 REFTEST_BBOX 环境变量时，打印差异像素的包围盒，
    // 帮助定位失败用例的差异区域（图像分析工具不可靠时的精确替代）。
    let track_bbox = std::env::var("REFTEST_BBOX").is_ok();
    let fw = fb1.width as usize;
    let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
    let (mut max_x, mut max_y) = (0usize, 0usize);

    for i in (0..fb1.data.len()).step_by(4) {
        let r1 = fb1.data[i];
        let g1 = fb1.data[i + 1];
        let b1 = fb1.data[i + 2];
        let a1 = fb1.data[i + 3];

        let r2 = fb2.data.get(i).copied().unwrap_or(0);
        let g2 = fb2.data.get(i + 1).copied().unwrap_or(0);
        let b2 = fb2.data.get(i + 2).copied().unwrap_or(0);
        let a2 = fb2.data.get(i + 3).copied().unwrap_or(0);

        let dr = (r1 as i16 - r2 as i16).unsigned_abs() as u8;
        let dg = (g1 as i16 - g2 as i16).unsigned_abs() as u8;
        let db = (b1 as i16 - b2 as i16).unsigned_abs() as u8;
        let da = (a1 as i16 - a2 as i16).unsigned_abs() as u8;

        let channel_max = dr.max(dg).max(db).max(da);
        max_diff = max_diff.max(channel_max);

        if channel_max > threshold {
            diff_pixels += 1;
            if channel_max == 1 {
                subpixel_diff += 1;
            }
            if track_bbox {
                let px = (i / 4) % fw;
                let py = (i / 4) / fw;
                if px < min_x {
                    min_x = px;
                }
                if py < min_y {
                    min_y = py;
                }
                if px > max_x {
                    max_x = px;
                }
                if py > max_y {
                    max_y = py;
                }
            }
        }
    }

    if track_bbox && diff_pixels > 0 {
        eprintln!("[REFTEST_BBOX] {label} x=[{min_x},{max_x}] y=[{min_y},{max_y}] fb_w={fw}");
    }

    (diff_pixels, max_diff, subpixel_diff)
}

/// 将帧缓冲保存为 PNG 文件。
pub fn save_framebuffer_png(fb: &FrameBuffer, path: &std::path::Path) -> Result<(), String> {
    // 简单的 BMP 保存（避免引入 PNG 编码依赖）
    // 使用 PPM 格式（最简单的无损图像格式）
    let ppm_path = path.with_extension("ppm");
    let mut content = format!("P6\n{} {}\n255\n", fb.width, fb.height);
    for i in (0..fb.data.len()).step_by(4) {
        content.push(fb.data[i] as char);
        content.push(fb.data[i + 1] as char);
        content.push(fb.data[i + 2] as char);
    }
    std::fs::write(&ppm_path, content.as_bytes()).map_err(|e| format!("Failed to save framebuffer: {e}"))?;
    Ok(())
}
