//! RFC 4.4-S3：compositor 侧 page + UI 合成 present 帧。

#![allow(clippy::too_many_arguments)]

/// 将 `src` 不透明 blit 到 `dst` 的 `(dst_x, dst_y)`。
fn blit_opaque(dst: &mut [u8], dst_w: u32, dst_h: u32, src: &[u8], src_w: u32, src_h: u32, dst_x: u32, dst_y: u32) {
    let copy_w = src_w.min(dst_w.saturating_sub(dst_x));
    let copy_h = src_h.min(dst_h.saturating_sub(dst_y));
    if copy_w == 0 || copy_h == 0 {
        return;
    }
    let row = (dst_w * 4) as usize;
    for y in 0..copy_h {
        let sy = y as usize;
        let dy = (dst_y + y) as usize;
        let src_off = (sy * src_w as usize) * 4;
        let dst_off = dy * row + dst_x as usize * 4;
        let len = copy_w as usize * 4;
        if src_off + len <= src.len() && dst_off + len <= dst.len() {
            dst[dst_off..dst_off + len].copy_from_slice(&src[src_off..src_off + len]);
        }
    }
}

/// 将 `src` 按 alpha 合成到 `dst`（src-over）。
fn blit_src_over(dst: &mut [u8], dst_w: u32, dst_h: u32, src: &[u8], src_w: u32, src_h: u32, dst_x: u32, dst_y: u32) {
    let copy_w = src_w.min(dst_w.saturating_sub(dst_x));
    let copy_h = src_h.min(dst_h.saturating_sub(dst_y));
    if copy_w == 0 || copy_h == 0 {
        return;
    }
    let row = (dst_w * 4) as usize;
    for y in 0..copy_h {
        for x in 0..copy_w {
            let sy = y as usize;
            let sx = x as usize;
            let src_i = (sy * src_w as usize + sx) * 4;
            if src_i + 3 >= src.len() {
                continue;
            }
            let a = src[src_i + 3] as f32 / 255.0;
            if a <= 0.0 {
                continue;
            }
            let dy = (dst_y + y) as usize;
            let dx = (dst_x + x) as usize;
            let dst_i = dy * row + dx * 4;
            if dst_i + 3 >= dst.len() {
                continue;
            }
            if a >= 1.0 {
                dst[dst_i..dst_i + 4].copy_from_slice(&src[src_i..src_i + 4]);
            } else {
                let inv = 1.0 - a;
                for c in 0..3 {
                    dst[dst_i + c] = (src[src_i + c] as f32 * a + dst[dst_i + c] as f32 * inv).round() as u8;
                }
                dst[dst_i + 3] = (255.0 * (a + dst[dst_i + 3] as f32 / 255.0 * inv)).min(255.0) as u8;
            }
        }
    }
}

/// 合成 present 帧：白底 → 页面 → UI（src-over）。
pub fn composite_present_frame(
    out_w: u32,
    out_h: u32,
    page: &[u8],
    page_w: u32,
    page_h: u32,
    ui: &[u8],
    ui_w: u32,
    ui_h: u32,
) -> Vec<u8> {
    let mut out = vec![255u8; (out_w * out_h * 4) as usize];
    for px in out.chunks_mut(4) {
        px[3] = 255;
    }
    blit_opaque(&mut out, out_w, out_h, page, page_w, page_h, 0, 0);
    blit_src_over(&mut out, out_w, out_h, ui, ui_w, ui_h, 0, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_alpha_composites_over_page() {
        let page = vec![0u8, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255];
        let ui = vec![255u8, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let out = composite_present_frame(2, 2, &page, 2, 2, &ui, 2, 2);
        assert_eq!(&out[..4], &[128, 0, 127, 255]);
        assert_eq!(&out[4..8], &[0, 0, 255, 255]);
    }
}
