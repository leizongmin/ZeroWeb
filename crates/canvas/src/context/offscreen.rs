//! OffscreenCanvas 和辅助几何函数。

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

/// OffscreenCanvas — 提供可离屏渲染的画布（桩实现，不包含 Web Worker 集成）。
///
/// 可用于在后台线程中执行绘制操作，然后将结果传回主线程。
/// 当前为 API 桩，仅支持创建和获取 2D 上下文。
pub struct OffscreenCanvas {
    /// 画布宽度。
    width: u32,
    /// 画布高度。
    height: u32,
}

impl OffscreenCanvas {
    /// 创建指定尺寸的 OffscreenCanvas。
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 获取 2D 渲染上下文。返回一个与 OffscreenCanvas 尺寸相同的 CanvasContext。
    pub fn get_context(&self) -> CanvasContext {
        CanvasContext::new(self.width, self.height)
    }

    /// 将当前画布内容转换为 ImageData（桩实现）。
    ///
    /// 在完整实现中，此方法应返回 ImageBitmap，此处返回 ImageData 作为桩。
    /// 返回的 ImageData 包含画布全部像素的快照。
    pub fn transfer_to_image_bitmap(&self) -> ImageData {
        let ctx = CanvasContext::new(self.width, self.height);
        ctx.get_image_data(0, 0, self.width, self.height)
    }

    /// 返回画布宽度。
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 返回画布高度。
    pub fn height(&self) -> u32 {
        self.height
    }
}
