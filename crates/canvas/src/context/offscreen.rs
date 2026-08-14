//! OffscreenCanvas 与辅助几何函数。
//!
//! R34xx：OffscreenCanvas 从 API 桩真实化——持有 CanvasContext（绘制状态与像素在
//! transfer 间保留，旧桩每次 get_context 新建丢失状态）；transfer_to_image_bitmap
//! 取当前 bitmap 快照并清空（spec：transfer 后 bitmap 置空）；width/height setter
//! 重置画布尺寸与 bitmap（spec OffscreenCanvas.width/height 可写）。
//! https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas

use super::types::*;

/// 使用射线法（ray casting）判断点是否在多边形内部。
pub(crate) fn point_in_polygon(px: f32, py: f32, points: &[(f32, f32)]) -> bool {
    let n = points.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// 计算点到线段的最短距离。
pub(crate) fn point_to_segment_dist(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < f32::EPSILON {
        // 线段退化为点
        let ddx = px - x1;
        let ddy = py - y1;
        return (ddx * ddx + ddy * ddy).sqrt();
    }
    let t = (((px - x1) * dx + (py - y1) * dy) / len_sq).clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    let ddx = px - proj_x;
    let ddy = py - proj_y;
    (ddx * ddx + ddy * ddy).sqrt()
}

/// OffscreenCanvas — 提供可离屏渲染的画布（HTML OffscreenCanvas）。
///
/// 持有 [`CanvasContext`]：绘制状态与像素在多次 get_context 调用间保留。
/// `transfer_to_image_bitmap` 取当前 bitmap 快照并清空（spec transfer 语义）。
pub struct OffscreenCanvas {
    context: CanvasContext,
}

impl OffscreenCanvas {
    /// 创建指定尺寸的 OffscreenCanvas。
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            context: CanvasContext::new(width, height),
        }
    }

    /// 获取 2D 渲染上下文（可变引用——绘制状态跨调用保留）。
    pub fn get_context(&mut self) -> &mut CanvasContext {
        &mut self.context
    }

    /// 将当前画布内容转换为 ImageBitmap（R34xx 真实化：取持有上下文的像素快照，
    /// 并清空画布 bitmap——spec transferToImageBitmap 后 bitmap 置空）。
    /// 返回 ImageData 作为 ImageBitmap 的像素载体（ImageBitmap Rust 类型尚未建立）。
    pub fn transfer_to_image_bitmap(&mut self) -> ImageData {
        let (w, h) = (self.context.width(), self.context.height());
        let bitmap = self.context.get_image_data(0, 0, w as i32, h as i32);
        // spec：transfer 后 bitmap 清空（绘制状态保留——仅 bitmap 像素清零）。
        self.context.clear_bitmap();
        bitmap
    }

    /// 返回画布宽度。
    pub fn width(&self) -> u32 {
        self.context.width()
    }

    /// 返回画布高度。
    pub fn height(&self) -> u32 {
        self.context.height()
    }

    /// 设置画布宽度（spec OffscreenCanvas.width setter：重置尺寸与 bitmap）。
    pub fn set_width(&mut self, width: u32) {
        self.context.resize(width, self.context.height());
    }

    /// 设置画布高度（spec OffscreenCanvas.height setter：重置尺寸与 bitmap）。
    pub fn set_height(&mut self, height: u32) {
        self.context.resize(self.context.width(), height);
    }
}
