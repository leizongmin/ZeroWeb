//! IPC 绘制快照 — 渲染进程向浏览器进程传递简化图元。

use serde::{Deserialize, Serialize};

/// IPC 颜色（RGBA 0–255）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IpcColor {
    /// 红。
    pub r: u8,
    /// 绿。
    pub g: u8,
    /// 蓝。
    pub b: u8,
    ///  alpha。
    pub a: u8,
}

/// IPC 矩形（CSS 逻辑像素）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IpcRect {
    /// 左上角 x。
    pub x: f32,
    /// 左上角 y。
    pub y: f32,
    /// 宽度。
    pub width: f32,
    /// 高度。
    pub height: f32,
}

/// IPC 填充矩形。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcFill {
    /// 区域。
    pub rect: IpcRect,
    /// 颜色。
    pub color: IpcColor,
}

/// IPC 文本 glyph。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcGlyph {
    /// x。
    pub x: f32,
    /// y。
    pub y: f32,
    /// 字号。
    pub font_size: f32,
    /// glyph id。
    pub glyph_id: u32,
    /// 字体 id。
    pub font_id: u32,
    /// 颜色。
    pub color: IpcColor,
    /// 旋转（弧度）。
    pub rotation: f32,
}

/// 渲染进程输出的绘制快照（fills + glyphs 子集）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintSnapshotParams {
    /// 视口宽度（CSS 逻辑像素）。
    pub viewport_width: u32,
    /// 视口高度（CSS 逻辑像素）。
    pub viewport_height: u32,
    /// 文档高度（CSS 逻辑像素）。
    pub document_height: f32,
    /// 填充图元。
    pub fills: Vec<IpcFill>,
    /// 文本图元。
    pub glyphs: Vec<IpcGlyph>,
}
