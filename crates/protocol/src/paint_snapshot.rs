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

/// IPC 绘制快照内共享的 glyph 源文本 run。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcGlyphTextRun {
    /// 当前绘制快照内唯一的文本 run 标识。
    pub run_id: u64,
    /// 文本 fragment 的完整源文本。
    pub text: String,
}

/// IPC glyph 的源文本 cluster 引用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcGlyphSource {
    /// 当前绘制快照内的文本 run 标识。
    pub run_id: u64,
    /// UTF-8 起始字节偏移。
    pub start: u32,
    /// UTF-8 结束字节偏移（exclusive）。
    pub end: u32,
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
    /// 当前字体内部的 OpenType glyph index；`None` 表示按 `glyph_id` 查字符。
    #[serde(default)]
    pub font_glyph_index: Option<u16>,
    /// 可选源文本 cluster。
    #[serde(default)]
    pub source: Option<IpcGlyphSource>,
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
    /// 颜色插值配置（CSS Color 4 `in <colorspace>`，R2289）。serde default 保旧消息兼容。
    #[serde(default)]
    pub interpolation: IpcGradientInterpolation,
}

/// IPC 渐变颜色插值色彩空间（镜像 render-foundation GradientColorSpace）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IpcGradientColorSpace {
    /// gamma 编码 sRGB。
    #[default]
    Srgb,
    /// 线性光 sRGB。
    SrgbLinear,
    /// CIE Lab。
    Lab,
    /// OKLab。
    Oklab,
    /// CIE LCH（极坐标）。
    Lch,
    /// OKLCH（极坐标）。
    Oklch,
}

/// IPC 极坐标色相插值法（镜像 render-foundation HueMethod）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IpcHueMethod {
    /// 短弧（默认）。
    #[default]
    Shorter,
    /// 长弧。
    Longer,
    /// 恒增。
    Increasing,
    /// 恒减。
    Decreasing,
}

/// IPC 渐变颜色插值配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IpcGradientInterpolation {
    /// 插值色彩空间。
    pub space: IpcGradientColorSpace,
    /// 色相插值法（仅极坐标空间有意义）。
    #[serde(default)]
    pub hue: IpcHueMethod,
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

/// IPC 路径填充。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcPathFill {
    /// 顶点序列 (x, y, x, y, ...)。
    pub vertices: Vec<f32>,
    /// 填充颜色。
    pub color: IpcColor,
}

/// IPC 路径描边。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcPathStroke {
    /// 顶点序列。
    pub vertices: Vec<f32>,
    /// 描边颜色。
    pub color: IpcColor,
    /// 线宽。
    pub line_width: f32,
    /// 是否闭合。
    pub closed: bool,
}

/// IPC 裁剪区域。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcClip {
    /// 裁剪矩形。
    pub rect: IpcRect,
}

/// IPC 2D 仿射变换。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcTransform {
    /// 变换应用区域。
    pub rect: IpcRect,
    /// 变换原点 x。
    pub origin_x: f32,
    /// 变换原点 y。
    pub origin_y: f32,
    /// 矩阵 a。
    pub a: f32,
    /// 矩阵 b。
    pub b: f32,
    /// 矩阵 c。
    pub c: f32,
    /// 矩阵 d。
    pub d: f32,
    /// 矩阵 tx。
    pub tx: f32,
    /// 矩阵 ty。
    pub ty: f32,
}

/// IPC CSS filter 函数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcFilterKind {
    /// blur(px)。
    Blur(f32),
    /// brightness(number)。
    Brightness(f32),
    /// contrast(number)。
    Contrast(f32),
    /// grayscale(number)。
    Grayscale(f32),
    /// hue-rotate(deg)。
    HueRotate(f32),
    /// invert(number)。
    Invert(f32),
    /// opacity(number)。
    Opacity(f32),
    /// saturate(number)。
    Saturate(f32),
    /// sepia(number)。
    Sepia(f32),
    /// drop-shadow。
    DropShadow {
        /// 水平偏移。
        offset_x: f32,
        /// 垂直偏移。
        offset_y: f32,
        /// 模糊半径。
        blur: f32,
        /// 颜色。
        color: IpcColor,
    },
}

/// IPC CSS filter 图元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcFilter {
    /// 滤镜应用区域。
    pub rect: IpcRect,
    /// 滤镜链。
    pub filters: Vec<IpcFilterKind>,
}

/// IPC CSS mix-blend-mode。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcBlendMode {
    /// normal。
    Normal,
    /// multiply。
    Multiply,
    /// screen。
    Screen,
    /// overlay。
    Overlay,
    /// darken。
    Darken,
    /// lighten。
    Lighten,
    /// color-dodge。
    ColorDodge,
    /// color-burn。
    ColorBurn,
    /// hard-light。
    HardLight,
    /// soft-light。
    SoftLight,
    /// difference。
    Difference,
    /// exclusion。
    Exclusion,
    /// hue。
    Hue,
    /// saturation。
    Saturation,
    /// color。
    Color,
    /// luminosity。
    Luminosity,
}

/// IPC 混合模式图元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcBlendModePrimitive {
    /// 混合区域。
    pub rect: IpcRect,
    /// 混合模式。
    pub mode: IpcBlendMode,
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
    /// `path_fills` 索引。
    PathFill(usize),
    /// `path_strokes` 索引。
    PathStroke(usize),
    /// `clips` 索引。
    Clip(usize),
    /// `transforms` 索引。
    Transform(usize),
    /// `filters` 索引。
    Filter(usize),
    /// `blend_modes` 索引。
    BlendMode(usize),
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
    /// 路径填充。
    pub path_fills: Vec<IpcPathFill>,
    /// 路径描边。
    pub path_strokes: Vec<IpcPathStroke>,
    /// 裁剪区域。
    pub clips: Vec<IpcClip>,
    /// 仿射变换。
    pub transforms: Vec<IpcTransform>,
    /// CSS filter。
    pub filters: Vec<IpcFilter>,
    /// mix-blend-mode。
    pub blend_modes: Vec<IpcBlendModePrimitive>,
    /// glyph 共享的源文本 run 表。
    #[serde(default)]
    pub glyph_text_runs: Vec<IpcGlyphTextRun>,
    /// 文本图元。
    pub glyphs: Vec<IpcGlyph>,
    /// 绘制顺序（与 engine `DrawOp` 子集对应）。
    pub draw_order: Vec<IpcDrawOp>,
    /// 本帧脏区域（S3 增量重绘；空 = 全量）。
    #[serde(default)]
    pub dirty_rects: Vec<IpcRect>,
    /// 主线程命中测试快照（与绘制同帧）。
    pub hit_test: Option<IpcHitTestCache>,
    /// 与浏览器 `TabSnapshot.navigation_epoch` 对齐；不匹配则丢弃 stale 帧。
    #[serde(default)]
    pub navigation_epoch: u64,
}

/// IPC 命中测试布局节点（仅几何 + node id）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcHitTestLayoutNode {
    /// 关联 DOM 节点 id（slotmap ffi）。
    pub node_id: Option<u64>,
    /// 相对父内容区 x。
    pub x: f32,
    /// 相对父内容区 y。
    pub y: f32,
    /// 盒宽。
    pub width: f32,
    /// 盒高。
    pub height: f32,
    /// 子盒。
    pub children: Vec<IpcHitTestLayoutNode>,
}

/// IPC 命中测试节点元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcHitTestNodeMeta {
    /// 标签名（小写）。
    pub tag_name: String,
    /// `id` 属性。
    pub id: Option<String>,
    /// `class` 属性。
    pub class_name: Option<String>,
    /// 链接 `href`。
    pub href: Option<String>,
    /// 图片 `src`（仅 `img` 元素，绝对化后）。
    #[serde(default)]
    pub src: Option<String>,
}

/// IPC 命中测试缓存。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcHitTestCache {
    /// 文档根节点 id。
    pub doc_root: u64,
    /// 布局树根。
    pub layout_root: IpcHitTestLayoutNode,
    /// 元素元数据 `(node_id, meta)`。
    pub nodes: Vec<(u64, IpcHitTestNodeMeta)>,
    /// 父节点 `(child, parent)`。
    pub parents: Vec<(u64, u64)>,
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
            path_fills: Vec::new(),
            path_strokes: Vec::new(),
            clips: Vec::new(),
            transforms: Vec::new(),
            filters: Vec::new(),
            blend_modes: Vec::new(),
            glyph_text_runs: Vec::new(),
            glyphs: Vec::new(),
            draw_order: Vec::new(),
            dirty_rects: Vec::new(),
            hit_test: None,
            navigation_epoch: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_snapshot_roundtrip_preserves_glyph_source_run_and_font_index() {
        let glyph = IpcGlyph {
            x: 1.0,
            y: 2.0,
            font_size: 16.0,
            glyph_id: 'A' as u32,
            font_glyph_index: Some(42),
            source: Some(IpcGlyphSource {
                run_id: 9,
                start: 0,
                end: 3,
            }),
            font_id: 7,
            color: IpcColor {
                r: 1,
                g: 2,
                b: 3,
                a: 255,
            },
            rotation: 0.0,
        };
        let snapshot = PaintSnapshotParams {
            glyph_text_runs: vec![IpcGlyphTextRun {
                run_id: 9,
                text: "A\u{301}".to_string(),
            }],
            glyphs: vec![glyph],
            ..Default::default()
        };

        let bytes = bincode::serialize(&snapshot).expect("serialize PaintSnapshotParams");
        let decoded: PaintSnapshotParams = bincode::deserialize(&bytes).expect("deserialize PaintSnapshotParams");
        let glyph = &decoded.glyphs[0];

        assert_eq!(glyph.glyph_id, 'A' as u32);
        assert_eq!(glyph.font_glyph_index, Some(42));
        let source = glyph.source.as_ref().expect("source cluster");
        assert_eq!(source.run_id, 9);
        assert_eq!((source.start, source.end), (0, 3));
        assert_eq!(decoded.glyph_text_runs[0].run_id, 9);
        assert_eq!(decoded.glyph_text_runs[0].text, "A\u{301}");
    }
}
