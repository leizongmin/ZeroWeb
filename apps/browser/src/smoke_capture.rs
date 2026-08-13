//! 真实窗口产品 smoke 的最终帧捕获与像素统计。

use std::path::Path;

use zero_render_foundation::surface::FrameBuffer;

const SIGNATURE_GRID: usize = 8;

/// 帧缓冲中的矩形区域。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRegion {
    /// 左边界。
    pub x: u32,
    /// 上边界。
    pub y: u32,
    /// 宽度。
    pub width: u32,
    /// 高度。
    pub height: u32,
}

/// 区域颜色分布统计。
#[derive(Clone, Debug, PartialEq)]
pub struct RegionStats {
    /// 区域像素数。
    pub pixels: u64,
    /// 非透明像素数。
    pub opaque_pixels: u64,
    /// 5-bit RGB 量化后的非空颜色桶数。
    pub unique_bins: usize,
    /// 最大颜色桶占区域像素的比例。
    pub dominant_ratio: f64,
    /// 区域最低亮度。
    pub luma_min: u8,
    /// 区域最高亮度。
    pub luma_max: u8,
    /// 亮度低于 96 的深色像素数。
    pub dark_pixels: u64,
    /// 深色像素占区域像素的比例。
    pub dark_ratio: f64,
    /// 8x8 分块平均亮度签名。
    pub signature: Vec<u8>,
}

impl RegionStats {
    /// 验证区域包含足够的可见颜色变化。
    pub fn validate_visible(&self, name: &str) -> Result<(), String> {
        if self.pixels == 0 || self.opaque_pixels == 0 {
            return Err(format!("{name} region is empty"));
        }
        if self.unique_bins < 8 {
            return Err(format!("{name} region has too few colors: {}", self.unique_bins));
        }
        if self.dominant_ratio > 0.98 {
            return Err(format!(
                "{name} region is nearly solid: dominant_ratio={:.6}",
                self.dominant_ratio
            ));
        }
        if self.luma_max.saturating_sub(self.luma_min) < 12 {
            return Err(format!(
                "{name} region has insufficient contrast: luma_min={}, luma_max={}",
                self.luma_min, self.luma_max
            ));
        }
        Ok(())
    }
}

/// 分析 RGBA 帧缓冲中的指定矩形。
pub fn analyze_region(width: u32, height: u32, rgba: &[u8], region: PixelRegion) -> Result<RegionStats, String> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "framebuffer dimensions overflow".to_string())?;
    if rgba.len() != expected {
        return Err(format!(
            "invalid framebuffer length: expected {expected}, got {}",
            rgba.len()
        ));
    }

    let x1 = region.x.saturating_add(region.width).min(width);
    let y1 = region.y.saturating_add(region.height).min(height);
    if region.x >= x1 || region.y >= y1 {
        return Err("pixel region is outside the framebuffer".to_string());
    }

    let mut bins = vec![0_u32; 32 * 32 * 32];
    let mut opaque_pixels = 0_u64;
    let mut luma_min = u8::MAX;
    let mut luma_max = u8::MIN;
    let mut dark_pixels = 0_u64;
    for y in region.y..y1 {
        for x in region.x..x1 {
            let pixel = pixel_at(width, rgba, x, y);
            if pixel[3] != 0 {
                opaque_pixels += 1;
            }
            let bin = ((pixel[0] as usize >> 3) << 10) | ((pixel[1] as usize >> 3) << 5) | (pixel[2] as usize >> 3);
            bins[bin] += 1;
            let luma = luma(pixel);
            luma_min = luma_min.min(luma);
            luma_max = luma_max.max(luma);
            if luma < 96 {
                dark_pixels += 1;
            }
        }
    }

    let pixels = u64::from(x1 - region.x) * u64::from(y1 - region.y);
    let unique_bins = bins.iter().filter(|count| **count > 0).count();
    let dominant = bins.into_iter().max().unwrap_or(0) as u64;
    Ok(RegionStats {
        pixels,
        opaque_pixels,
        unique_bins,
        dominant_ratio: dominant as f64 / pixels as f64,
        luma_min,
        luma_max,
        dark_pixels,
        dark_ratio: dark_pixels as f64 / pixels as f64,
        signature: region_signature(width, rgba, region.x, region.y, x1, y1),
    })
}

/// 保存 PNG，并输出 Chrome 与页面内容区的结构化统计。
pub fn capture_presented_frame(
    path: &Path,
    framebuffer: &FrameBuffer,
    chrome_region: PixelRegion,
    page_region: PixelRegion,
    mode: &str,
    fixture: &str,
    source: &str,
) -> Result<(), String> {
    if framebuffer.width < 320 || framebuffer.height < 240 {
        return Err(format!(
            "smoke framebuffer is too small: {}x{}",
            framebuffer.width, framebuffer.height
        ));
    }
    let chrome = analyze_region(framebuffer.width, framebuffer.height, &framebuffer.data, chrome_region)?;
    let page = analyze_region(framebuffer.width, framebuffer.height, &framebuffer.data, page_region)?;
    chrome.validate_visible("chrome")?;
    page.validate_visible("page")?;
    if page.dark_ratio < 0.0002 {
        return Err(format!(
            "page region lacks dark text pixels: dark_pixels={}, dark_ratio={:.6}",
            page.dark_pixels, page.dark_ratio
        ));
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create smoke capture directory: {error}"))?;
    }
    let file = std::fs::File::create(path)
        .map_err(|error| format!("failed to create smoke capture {}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(file, framebuffer.width, framebuffer.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("failed to write smoke PNG header: {error}"))?;
    writer
        .write_image_data(&framebuffer.data)
        .map_err(|error| format!("failed to write smoke PNG pixels: {error}"))?;

    tracing::info!(
        "SMOKE_CAPTURE mode={mode} fixture={fixture} source={source} path={} width={} height={}",
        path.display(),
        framebuffer.width,
        framebuffer.height
    );
    log_region("chrome", &chrome);
    log_region("page", &page);
    Ok(())
}

/// 将帧缓冲中的指定区域保存为独立 PNG。
pub fn capture_region(path: &Path, framebuffer: &FrameBuffer, region: PixelRegion) -> Result<(), String> {
    let x1 = region.x.saturating_add(region.width).min(framebuffer.width);
    let y1 = region.y.saturating_add(region.height).min(framebuffer.height);
    if region.x >= x1 || region.y >= y1 {
        return Err("capture region is outside the framebuffer".to_string());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create smoke capture directory: {error}"))?;
    }
    let width = x1 - region.x;
    let height = y1 - region.y;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in region.y..y1 {
        let start = ((y * framebuffer.width + region.x) * 4) as usize;
        let end = start + width as usize * 4;
        pixels.extend_from_slice(&framebuffer.data[start..end]);
    }
    let file = std::fs::File::create(path)
        .map_err(|error| format!("failed to create region capture {}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&pixels))
        .map_err(|error| format!("failed to write region capture: {error}"))
}

fn log_region(name: &str, stats: &RegionStats) {
    let signature = stats.signature.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
    tracing::info!(
        "SMOKE_REGION name={name} pixels={} opaque={} unique_bins={} dominant_ratio={:.6} luma_min={} luma_max={} dark_pixels={} dark_ratio={:.6} signature={signature}",
        stats.pixels,
        stats.opaque_pixels,
        stats.unique_bins,
        stats.dominant_ratio,
        stats.luma_min,
        stats.luma_max,
        stats.dark_pixels,
        stats.dark_ratio
    );
}

fn region_signature(width: u32, rgba: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<u8> {
    let mut signature = Vec::with_capacity(SIGNATURE_GRID * SIGNATURE_GRID);
    for grid_y in 0..SIGNATURE_GRID {
        let cell_y0 = y0 + ((y1 - y0) as usize * grid_y / SIGNATURE_GRID) as u32;
        let cell_y1 = y0 + ((y1 - y0) as usize * (grid_y + 1) / SIGNATURE_GRID) as u32;
        for grid_x in 0..SIGNATURE_GRID {
            let cell_x0 = x0 + ((x1 - x0) as usize * grid_x / SIGNATURE_GRID) as u32;
            let cell_x1 = x0 + ((x1 - x0) as usize * (grid_x + 1) / SIGNATURE_GRID) as u32;
            let mut sum = 0_u64;
            let mut count = 0_u64;
            for y in cell_y0..cell_y1.max(cell_y0 + 1).min(y1) {
                for x in cell_x0..cell_x1.max(cell_x0 + 1).min(x1) {
                    sum += u64::from(luma(pixel_at(width, rgba, x, y)));
                    count += 1;
                }
            }
            signature.push(sum.checked_div(count).unwrap_or(0) as u8);
        }
    }
    signature
}

fn pixel_at(width: u32, rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * width as usize) + x as usize) * 4;
    [rgba[index], rgba[index + 1], rgba[index + 2], rgba[index + 3]]
}

fn luma(pixel: [u8; 4]) -> u8 {
    ((u16::from(pixel[0]) * 77 + u16::from(pixel[1]) * 150 + u16::from(pixel[2]) * 29) >> 8) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_region_reports_color_distribution_and_signature() {
        let mut rgba = Vec::new();
        for y in 0..8_u8 {
            for x in 0..8_u8 {
                rgba.extend_from_slice(&[x * 24, y * 24, (x + y) * 12, 255]);
            }
        }
        let stats = analyze_region(
            8,
            8,
            &rgba,
            PixelRegion {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
        )
        .unwrap();

        assert_eq!(stats.pixels, 64);
        assert_eq!(stats.opaque_pixels, 64);
        assert!(stats.unique_bins >= 32);
        assert!(stats.dominant_ratio < 0.1);
        assert!(stats.dark_pixels > 0);
        assert!(stats.dark_ratio > 0.0);
        assert_eq!(stats.signature.len(), 64);
        stats.validate_visible("test").unwrap();
    }

    #[test]
    fn solid_and_invalid_regions_are_rejected() {
        let rgba = [200, 200, 200, 255].repeat(16);
        let stats = analyze_region(
            4,
            4,
            &rgba,
            PixelRegion {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
            },
        )
        .unwrap();
        assert!(stats.validate_visible("solid").is_err());
        assert!(
            analyze_region(
                4,
                4,
                &rgba,
                PixelRegion {
                    x: 5,
                    y: 0,
                    width: 1,
                    height: 1,
                },
            )
            .is_err()
        );
        assert!(
            analyze_region(
                4,
                4,
                &rgba[..8],
                PixelRegion {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1
                }
            )
            .is_err()
        );
    }
}
