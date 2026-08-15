//! RFC 4.2-S2：compositor 侧滚动变换（默认开，`ZW_COMPOSITOR_SCROLL_TRANSFORM=0` 禁用）。
//!
//! 在 `GetCompositorFrame` 时将 scroll 烘焙进 RGBA，回读 scroll 归零。

/// 将 `scroll` 烘焙进 viewport 位图：输出像素 `(x,y)` 采样源 `(x+scroll_x, y+scroll_y)`。
pub fn bake_scroll_into_rgba(rgba: &[u8], width: u32, height: u32, scroll_x: f32, scroll_y: f32) -> Vec<u8> {
    let expected = (width as usize).saturating_mul(height as usize).saturating_mul(4);
    if rgba.len() != expected || expected == 0 {
        return rgba.to_vec();
    }
    if scroll_x == 0.0 && scroll_y == 0.0 {
        return rgba.to_vec();
    }
    let sx = scroll_x.round() as i32;
    let sy = scroll_y.round() as i32;
    let w = width as i32;
    let h = height as i32;
    let mut out = vec![0u8; expected];
    for oy in 0..h {
        for ox in 0..w {
            let ix = ox + sx;
            let iy = oy + sy;
            if ix < 0 || iy < 0 || ix >= w || iy >= h {
                continue;
            }
            let src = ((iy * w + ix) * 4) as usize;
            let dst = ((oy * w + ox) * 4) as usize;
            out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, color: [u8; 4]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            v.extend_from_slice(&color);
        }
        v
    }

    #[test]
    fn zero_scroll_is_identity() {
        let src = solid(2, 2, [1, 2, 3, 4]);
        let out = bake_scroll_into_rgba(&src, 2, 2, 0.0, 0.0);
        assert_eq!(out, src);
    }

    #[test]
    fn vertical_scroll_shifts_pixels_up() {
        let mut src = solid(2, 4, [0, 0, 0, 255]);
        // top row red
        for x in 0..2 {
            let i = (x * 4) as usize;
            src[i..i + 4].copy_from_slice(&[255, 0, 0, 255]);
        }
        let out = bake_scroll_into_rgba(&src, 2, 4, 0.0, 1.0);
        // Positive scroll moves content up: old red row 0 leaves the viewport,
        // output row 0 samples old black row 1, and the bottom row is cleared.
        assert_eq!(&out[0..4], &[0, 0, 0, 255]);
        let bottom = ((4 - 1) * 2 * 4) as usize;
        assert_eq!(&out[bottom..bottom + 4], &[0, 0, 0, 0]);
    }
}
