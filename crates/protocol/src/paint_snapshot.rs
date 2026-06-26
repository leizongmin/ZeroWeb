//! IPC 绘制快照 — 渲染进程向浏览器进程传递图元与图片像素。

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

/// IPC 圆角矩形。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRoundedRect {
    /// 区域。
    pub rect: IpcRect,
    /// 颜色。
    pub color: IpcColor,
    /// 左上角半径。
    pub top_left_radius: f32,
    /// 右上角半径。
    pub top_right_radius: f32,
    /// 右下角半径。
    pub bottom_right_radius: f32,
    /// 左下角半径。
    pub bottom_left_radius: f32,
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

/// IPC 图片图元（不含像素，像素见 [`IpcImagePayload`]）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcImage {
    /// 绘制区域。
    pub rect: IpcRect,
    /// 图片缓存键（与 engine `image_resource_key` 一致）。
    pub image_key: u64,
    /// 可选裁剪窗口。
    pub clip: Option<IpcRect>,
}

/// IPC 解码后的图片像素。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcImagePayload {
    /// 与 [`IpcImage::image_key`] 对应。
    pub image_key: u64,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
    /// RGBA 行优先像素。
    pub rgba: Vec<u8>,
}

/// IPC 渐变色标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcGradientStop {
    /// 偏移 [0, 1]。
    pub offset: f32,
    /// 颜色。
    pub color: IpcColor,
}

/// IPC 渐变类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcGradientKind {
    /// 线性渐变。
    Linear {
        /// 起点 x。
        x0: f32,
        /// 起点 y。
        y0: f32,
        /// 终点 x。
        x1: f32,
        /// 终点 y。
        y1: f32,
    },
    /// 径向渐变。
    Radial {
        /// 圆心 x。
        cx: f32,
        /// 圆心 y。
        cy: f32,
        /// 内圆半径。
        inner_radius: f32,
        /// 外圆半径。
        outer_radius: f32,
    },
    /// 锥形渐变。
    Conic {
        /// 圆心 x。
        cx: f32,
        /// 圆心 y。
        cy: f32,
        /// 起始角（弧度）。
        start_angle: f32,
    },
}

/// IPC 渐变图元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcGradient {
    /// 渐变区域。
    pub rect: IpcRect,
    /// 渐变类型。
    pub kind: IpcGradientKind,
    /// 色标。
    pub stops: Vec<IpcGradientStop>,
    /// 是否 repeating。
    pub repeating: bool,
}

/// IPC 线段端点样式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcLineCap {
    /// 平头。
    Butt,
    /// 圆头。
    Round,
    /// 方头。
    Square,
}

/// IPC 线段线型。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcLineStyle {
    /// 实线。
    Solid,
    /// 虚线。
    Dashed,
    /// 点线。
    Dotted,
}

/// IPC 描边线段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStroke {
    /// 起点 x。
    pub x1: f32,
    /// 起点 y。
    pub y1: f32,
    /// 终点 x。
    pub x2: f32,
    /// 终点 y。
    pub y2: f32,
    /// 线宽。
    pub width: f32,
    /// 颜色。
    pub color: IpcColor,
    /// 线型。
    pub style: IpcLineStyle,
    /// 端点。
    pub cap: IpcLineCap,
}

/// IPC 阴影图元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcShadow {
    /// 参考矩形。
    pub rect: IpcRect,
    /// 颜色。
    pub color: IpcColor,
    /// 水平偏移。
    pub offset_x: f32,
    /// 垂直偏移。
    pub offset_y: f32,
    /// 模糊半径。
    pub blur_radius: f32,
    /// 扩展半径。
    pub spread_radius: f32,
}

/// IPC 绘制顺序条目。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcDrawOp {
    /// `fills` 索引。
    Fill(usize),
    /// `rounded_rects` 索引。
    RoundedRect(usize),
    /// `gradients` 索引。
    Gradient(usize),
    /// `shadows` 索引。
    Shadow(usize),
    /// `images` 索引。
    Image(usize),
    /// `strokes` 索引。
    Stroke(usize),
    /// `glyphs` 索引。
    Glyph(usize),
}

/// 渲染进程输出的绘制快照。
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
    /// 圆角矩形。
    pub rounded_rects: Vec<IpcRoundedRect>,
    /// 渐变。
    pub gradients: Vec<IpcGradient>,
    /// 阴影。
    pub shadows: Vec<IpcShadow>,
    /// 图片图元。
    pub images: Vec<IpcImage>,
    /// 解码后的图片像素（填充 browser 侧 ImageCache）。
    pub image_payloads: Vec<IpcImagePayload>,
    /// 描边线段。
    pub strokes: Vec<IpcStroke>,
    /// 文本图元。
    pub glyphs: Vec<IpcGlyph>,
    /// 绘制顺序（与 engine `DrawOp` 子集对应）。
    pub draw_order: Vec<IpcDrawOp>,
}

impl Default for PaintSnapshotParams {
    fn default() -> Self {
        Self {
            viewport_width: 0,
            viewport_height: 0,
            document_height: 0.0,
            fills: Vec::new(),
            rounded_rects: Vec::new(),
            gradients: Vec::new(),
            shadows: Vec::new(),
            images: Vec::new(),
            image_payloads: Vec::new(),
            strokes: Vec::new(),
            glyphs: Vec::new(),
            draw_order: Vec::new(),
        }
    }
}
