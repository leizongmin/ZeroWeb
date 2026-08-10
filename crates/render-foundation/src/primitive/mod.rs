//! 渲染图元 — 填充矩形、圆角矩形、路径填充、路径描边、裁剪区域、渐变、阴影、图片、Glyph 图元等

mod ops;

use crate::color::Color;
use crate::geometry::Rect;
use crate::image_cache::ImageKey;

/// 填充图元 — 纯色矩形
#[derive(Debug, Clone)]
pub struct FillPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
}

/// 圆角矩形图元 — 支持 border-radius 的填充矩形
#[derive(Debug, Clone)]
pub struct RoundedRectPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
    /// 左上角圆角半径
    pub top_left_radius: f32,
    /// 右上角圆角半径
    pub top_right_radius: f32,
    /// 右下角圆角半径
    pub bottom_right_radius: f32,
    /// 左下角圆角半径
    pub bottom_left_radius: f32,
}

impl RoundedRectPrimitive {
    /// 创建四个圆角相同的圆角矩形
    pub fn uniform(rect: Rect, color: Color, radius: f32) -> Self {
        Self {
            rect,
            color,
            top_left_radius: radius,
            top_right_radius: radius,
            bottom_right_radius: radius,
            bottom_left_radius: radius,
        }
    }
}

/// 路径填充图元 — 使用路径命令填充任意形状。
#[derive(Debug, Clone)]
pub struct PathFillPrimitive {
    /// 路径命令列表（扁平化的线段序列）。
    /// 每对 f32 表示一个顶点 (x, y)，构成闭合多边形。
    pub vertices: Vec<f32>,
    /// 填充颜色。
    pub color: Color,
}

/// 线段端点样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineCap {
    /// 平头
    Butt,
    /// 圆头
    Round,
    /// 方头
    Square,
}

/// 描边线型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    /// 实线
    Solid,
    /// 虚线（线段和间隔交替）
    Dashed,
    /// 点线
    Dotted,
}

/// 路径描边图元 — 使用路径命令描边任意形状。
#[derive(Debug, Clone)]
pub struct PathStrokePrimitive {
    /// 路径顶点列表
    pub vertices: Vec<f32>,
    /// 描边颜色
    pub color: Color,
    /// 线宽
    pub line_width: f32,
    /// 是否闭合路径
    pub closed: bool,
}

/// 描边图元 — 两点之间的线段
#[derive(Debug, Clone)]
pub struct StrokePrimitive {
    /// 起点 x
    pub x1: f32,
    /// 起点 y
    pub y1: f32,
    /// 终点 x
    pub x2: f32,
    /// 终点 y
    pub y2: f32,
    /// 线宽
    pub width: f32,
    /// 颜色
    pub color: Color,
    /// 线型
    pub style: LineStyle,
    /// 端点样式
    pub cap: LineCap,
}

/// 裁剪区域图元
#[derive(Debug, Clone)]
pub struct ClipPrimitive {
    /// 裁剪矩形
    pub rect: Rect,
}

/// 渐变色标
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// 偏移量 [0, 1]
    pub offset: f32,
    /// 颜色
    pub color: Color,
}

/// 渐变类型
#[derive(Debug, Clone)]
pub enum GradientKind {
    /// 线性渐变
    Linear {
        /// 起点 x
        x0: f32,
        /// 起点 y
        y0: f32,
        /// 终点 x
        x1: f32,
        /// 终点 y
        y1: f32,
    },
    /// 径向渐变
    Radial {
        /// 圆心 x
        cx: f32,
        /// 圆心 y
        cy: f32,
        /// 内圆半径
        inner_radius: f32,
        /// 外圆半径
        outer_radius: f32,
    },
    /// 锥形渐变
    Conic {
        /// 圆心 x
        cx: f32,
        /// 圆心 y
        cy: f32,
        /// 起始角度（弧度）
        start_angle: f32,
    },
}

/// CSS Color 4 渐变颜色插值色彩空间（`gradient in <colorspace>`）。
///
/// driving: R2289 gradient colorspace render-math。Srgb 为默认，保留既有 gamma 编码
/// 逐通道插值（零回归）。wide-gamut（display-p3/xyz/rec2020/...）无色彩管理管线，
/// 由 parser 端归一为 Srgb 优雅回退。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientColorSpace {
    /// gamma 编码 sRGB（CSS 默认 `in srgb`）。
    #[default]
    Srgb,
    /// 线性光 sRGB（`in srgb-linear`）。
    SrgbLinear,
    /// CIE Lab（`in lab`）。
    Lab,
    /// OKLab（`in oklab`）。
    Oklab,
    /// CIE LCH（`in lch`，极坐标，需色相插值法）。
    Lch,
    /// OKLCH（`in oklch`，极坐标，需色相插值法）。
    Oklch,
}

/// 极坐标色彩空间（LCH/OKLCH）的色相插值法（CSS Color 4 §13.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HueMethod {
    /// `shorter hue`（默认，短弧）。
    #[default]
    Shorter,
    /// `longer hue`（长弧）。
    Longer,
    /// `increasing hue`（恒增）。
    Increasing,
    /// `decreasing hue`（恒减）。
    Decreasing,
}

/// CSS Color 4 渐变颜色插值配置：色彩空间 + （极坐标时）色相插值法。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GradientInterpolation {
    /// 插值色彩空间。
    pub space: GradientColorSpace,
    /// 色相插值法（仅 Lch/Oklch 有意义；其余忽略）。
    pub hue: HueMethod,
}

/// 渐变图元
#[derive(Debug, Clone)]
pub struct GradientPrimitive {
    /// 渐变区域
    pub rect: Rect,
    /// 渐变类型
    pub kind: GradientKind,
    /// 色标列表
    pub stops: Vec<GradientStop>,
    /// 是否为重复渐变（repeating-*-gradient）
    pub repeating: bool,
    /// 颜色插值配置（CSS Color 4 `in <colorspace>`）。默认 Srgb = 既有行为。
    pub interpolation: GradientInterpolation,
}

/// 阴影图元
#[derive(Debug, Clone)]
pub struct ShadowPrimitive {
    /// 阴影参考矩形
    pub rect: Rect,
    /// 阴影颜色
    pub color: Color,
    /// 水平偏移
    pub offset_x: f32,
    /// 垂直偏移
    pub offset_y: f32,
    /// 模糊半径
    pub blur_radius: f32,
    /// 扩展半径
    pub spread_radius: f32,
    /// R2476：是否内阴影（inset）。outset（false）= 阴影在盒外向外模糊；inset（true）=
    /// 阴影在盒内（box 减 offset+spread 收缩的洞），向内模糊，裁切到盒。
    pub inset: bool,
}

/// 图片图元
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// 图片绘制区域（原始、未裁剪；source 始终映射到整个 rect）
    pub rect: Rect,
    /// 图片缓存键
    pub image_key: ImageKey,
    /// 可选裁剪窗口（CSS clip:rect / overflow:hidden / clip-path inset）。
    ///
    /// 裁剪语义 = **裁剪（crop）非重缩放**：仅绘制 rect 与此窗口的交集区域，
    /// 但 source 仍按完整 rect 映射（保持原始分辨率，不因裁剪而缩放）。
    /// None = 无裁剪，绘制整个 rect。
    pub clip: Option<Rect>,
}

/// Glyph 图元 — 单个字符或整形后字形的渲染指令
#[derive(Debug, Clone)]
pub struct GlyphPrimitive {
    /// 绘制位置 x
    pub x: f32,
    /// 绘制位置 y
    pub y: f32,
    /// 字号
    pub font_size: f32,
    /// 颜色
    pub color: Color,
    /// 源 Unicode 码点；整形后仍保留，用于选择、命中测试与文本恢复。
    pub glyph_id: u32,
    /// 当前字体内部的 OpenType glyph index；`None` 表示按 `glyph_id` 查字符。
    pub font_glyph_index: Option<u16>,
    /// 字体 ID
    pub font_id: FontId,
    /// 预缓存位图宽度（可选）
    pub bitmap_width: Option<u32>,
    /// 预缓存位图高度（可选）
    pub bitmap_height: Option<u32>,
    /// 旋转角度（弧度）— 用于 writing-mode: vertical-rl/vertical-lr 时旋转字符。
    /// 0.0 = 无旋转，std::f32::consts::FRAC_PI_2 = 顺时针 90°。
    pub rotation: f32,
    /// 合成斜体标记（R2497 synthetic italic）— true 时 CPU blit 对该 glyph 应用 ~14°
    /// 水平 shear（italic/oblique 在系统字体无 italic face 时的合成斜体，对齐 chromium）。
    /// 由 painter 据 `font_style:italic/oblique && resolved face 非 italic` 置位，
    /// 避免 double-shear（真 italic face 已斜，不再合成）。
    pub synthetic_italic: bool,
}

impl GlyphPrimitive {
    /// 返回字体内部 glyph index；普通 Unicode 图元返回 `None`。
    pub const fn font_glyph_index(&self) -> Option<u16> {
        self.font_glyph_index
    }

    /// 返回源 Unicode 码点；不可见标记返回 `None`。
    pub fn code_point(&self) -> Option<char> {
        char::from_u32(self.glyph_id).filter(|ch| *ch != '\0')
    }
}

/// CSS filter 函数类型。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterKind {
    /// blur(px) — 高斯模糊。
    Blur(f32),
    /// brightness(number) — 亮度调节。
    Brightness(f32),
    /// contrast(number) — 对比度调节。
    Contrast(f32),
    /// grayscale(number) — 灰度。
    Grayscale(f32),
    /// hue-rotate(deg) — 色相旋转。
    HueRotate(f32),
    /// invert(number) — 反色。
    Invert(f32),
    /// opacity(number) — 透明度。
    Opacity(f32),
    /// saturate(number) — 饱和度调节。
    Saturate(f32),
    /// sepia(number) — 棕褐色调。
    Sepia(f32),
    /// drop-shadow(x, y, blur, color) — 投影阴影。
    DropShadow(f32, f32, f32, Color),
}

/// CSS filter 图元 — 对指定区域内的所有图元应用滤镜效果。
#[derive(Debug, Clone)]
pub struct FilterPrimitive {
    /// 滤镜应用区域（元素的内容+padding 盒）
    pub rect: Rect,
    /// 滤镜函数列表（按顺序依次应用）
    pub filters: Vec<FilterKind>,
}

/// CSS transform 图元 — 对指定区域内的所有图元应用 2D 仿射变换。
///
/// 变换以 3x3 仿射矩阵表示（最后一行 [0, 0, 1] 省略）：
/// ```text
/// | a  c  tx |
/// | b  d  ty |
/// | 0  0   1 |
/// ```
#[derive(Debug, Clone)]
pub struct TransformPrimitive {
    /// 变换应用区域（元素的盒模型区域）
    pub rect: Rect,
    /// 变换原点（相对于视口的绝对坐标）
    pub origin_x: f32,
    /// 变换原点 Y
    pub origin_y: f32,
    /// 仿射矩阵 a 分量（水平缩放/旋转）
    pub a: f32,
    /// 仿射矩阵 b 分量（垂直旋转/倾斜）
    pub b: f32,
    /// 仿射矩阵 c 分量（水平倾斜/旋转）
    pub c: f32,
    /// 仿射矩阵 d 分量（垂直缩放/旋转）
    pub d: f32,
    /// 仿射矩阵 tx 分量（水平平移）
    pub tx: f32,
    /// 仿射矩阵 ty 分量（垂直平移）
    pub ty: f32,
}

/// CSS mix-blend-mode 混合模式枚举。
/// 定义元素与下层内容混合的方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    /// normal — 默认值，不混合
    Normal,
    /// multiply — 正片叠底
    Multiply,
    /// screen — 滤色
    Screen,
    /// overlay — 叠加
    Overlay,
    /// darken — 变暗
    Darken,
    /// lighten — 变亮
    Lighten,
    /// color-dodge — 颜色减淡
    ColorDodge,
    /// color-burn — 颜色加深
    ColorBurn,
    /// hard-light — 强光
    HardLight,
    /// soft-light — 柔光
    SoftLight,
    /// difference — 差值
    Difference,
    /// exclusion — 排除
    Exclusion,
    /// hue — 色相
    Hue,
    /// saturation — 饱和度
    Saturation,
    /// color — 颜色
    Color,
    /// luminosity — 亮度
    Luminosity,
}

/// 混合模式图元 — 标记区域内图元需要与下层内容混合。
#[derive(Debug, Clone)]
pub struct BlendModePrimitive {
    /// 混合模式应用区域（元素盒模型区域）
    pub rect: Rect,
    /// 混合模式类型
    pub mode: BlendMode,
}

/// 字体 ID 标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(pub u32);

/// 渲染统计 — 追踪图元数量、估算 draw call 数量和批处理效率。
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// 填充矩形数量
    pub fill_count: usize,
    /// 圆角矩形数量
    pub rounded_rect_count: usize,
    /// 路径填充数量
    pub path_fill_count: usize,
    /// 路径描边数量
    pub path_stroke_count: usize,
    /// 描边线段数量
    pub stroke_count: usize,
    /// 渐变数量
    pub gradient_count: usize,
    /// 阴影数量
    pub shadow_count: usize,
    /// 图片数量
    pub image_count: usize,
    /// Glyph 数量
    pub glyph_count: usize,
    /// Filter 数量
    pub filter_count: usize,
    /// 裁剪区域数量
    pub clip_count: usize,
    /// 估算的 draw call 数量（基于颜色/材质去重）
    pub estimated_draw_calls: usize,
    /// 被视口剔除的图元数量
    pub culled_count: usize,
    /// 本帧需要重绘的脏区域（S3 增量重绘契约：`(x, y, w, h)`，视口坐标）。
    /// 当前全量渲染 = 全视口脏；增量光栅化（RFC S3）消费本字段只重绘变化区域。
    pub dirty_rects: Vec<(f32, f32, f32, f32)>,
}

impl RenderStats {
    /// 图元总数
    pub fn total_primitives(&self) -> usize {
        self.fill_count
            + self.rounded_rect_count
            + self.path_fill_count
            + self.path_stroke_count
            + self.stroke_count
            + self.gradient_count
            + self.shadow_count
            + self.image_count
            + self.glyph_count
            + self.clip_count
    }
}

/// 渲染图元集合 — 包含一帧的所有渲染指令。
#[derive(Debug, Clone, Default)]
pub struct RenderPrimitives {
    /// 裁剪区域列表（绘制其他图元前应应用裁剪）
    pub clips: Vec<ClipPrimitive>,
    /// 填充矩形列表
    pub fills: Vec<FillPrimitive>,
    /// 圆角矩形列表
    pub rounded_rects: Vec<RoundedRectPrimitive>,
    /// 路径填充列表
    pub path_fills: Vec<PathFillPrimitive>,
    /// 路径描边列表
    pub path_strokes: Vec<PathStrokePrimitive>,
    /// 描边线段列表
    pub strokes: Vec<StrokePrimitive>,
    /// 渐变列表
    pub gradients: Vec<GradientPrimitive>,
    /// 阴影列表
    pub shadows: Vec<ShadowPrimitive>,
    /// 图片列表
    pub images: Vec<ImagePrimitive>,
    /// Glyph 列表
    pub glyphs: Vec<GlyphPrimitive>,
    /// Filter 列表
    pub filters: Vec<FilterPrimitive>,
    /// Blend mode 列表（混合模式应用区域）
    pub blend_modes: Vec<BlendModePrimitive>,
    /// Transform 列表（2D 仿射变换）
    pub transforms: Vec<TransformPrimitive>,
    /// 绘制顺序记录 — 按图元被 `add_*` 的真实插入顺序。
    ///
    /// 默认渲染（`render_full_scene`）按类型分桶渲染（所有 images 画在所有 fills
    /// 之后），违反 CSS painting order（父背景图应画在子内容**之下**）。
    /// 本字段记录插入顺序，供 env-gated `ZERO_DRAW_ORDER` 路径按真实 z 序渲染，
    /// 修复 DC-10 类型分桶绘制顺序缺陷。默认行为保持字节不变（零回归）。
    pub draw_order: Vec<DrawOp>,
}

/// 绘制顺序条目 — 指向某个 typed Vec 中的图元索引。
///
/// 与 `RenderPrimitives` 的 typed Vec 并存：clip/opacity 后处理仍读 typed Vec
/// （`PrimitiveCounts` 快照），`draw_order` 仅用于渲染时的顺序遍历。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawOp {
    /// `fills` 中的索引
    Fill(usize),
    /// `rounded_rects` 中的索引
    RoundedRect(usize),
    /// `path_fills` 中的索引
    PathFill(usize),
    /// `path_strokes` 中的索引
    PathStroke(usize),
    /// `strokes` 中的索引
    Stroke(usize),
    /// `gradients` 中的索引
    Gradient(usize),
    /// `shadows` 中的索引
    Shadow(usize),
    /// `images` 中的索引
    Image(usize),
    /// `glyphs` 中的索引
    Glyph(usize),
    /// `filters` 中的索引
    Filter(usize),
    /// `blend_modes` 中的索引
    BlendMode(usize),
    /// `transforms` 中的索引
    Transform(usize),
    /// `clips` 中的索引
    Clip(usize),
}

impl RenderPrimitives {
    /// 创建空的图元列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个填充矩形
    pub fn add_fill(&mut self, rect: Rect, color: Color) {
        let idx = self.fills.len();
        self.fills.push(FillPrimitive { rect, color });
        self.draw_order.push(DrawOp::Fill(idx));
    }

    /// 添加一个圆角矩形
    pub fn add_rounded_rect(&mut self, rounded: RoundedRectPrimitive) {
        let idx = self.rounded_rects.len();
        self.rounded_rects.push(rounded);
        self.draw_order.push(DrawOp::RoundedRect(idx));
    }

    /// 添加一个路径填充图元。
    pub fn add_path_fill(&mut self, vertices: Vec<f32>, color: Color) {
        let idx = self.path_fills.len();
        self.path_fills.push(PathFillPrimitive { vertices, color });
        self.draw_order.push(DrawOp::PathFill(idx));
    }

    /// 添加一个路径描边图元。
    pub fn add_path_stroke(&mut self, vertices: Vec<f32>, color: Color, line_width: f32, closed: bool) {
        let idx = self.path_strokes.len();
        self.path_strokes.push(PathStrokePrimitive {
            vertices,
            color,
            line_width,
            closed,
        });
        self.draw_order.push(DrawOp::PathStroke(idx));
    }

    /// 添加一个描边线段
    pub fn add_stroke(&mut self, stroke: StrokePrimitive) {
        let idx = self.strokes.len();
        self.strokes.push(stroke);
        self.draw_order.push(DrawOp::Stroke(idx));
    }

    /// 添加一个裁剪区域
    pub fn add_clip(&mut self, rect: Rect) {
        let idx = self.clips.len();
        self.clips.push(ClipPrimitive { rect });
        self.draw_order.push(DrawOp::Clip(idx));
    }

    /// 添加一个渐变
    pub fn add_gradient(&mut self, gradient: GradientPrimitive) {
        let idx = self.gradients.len();
        self.gradients.push(gradient);
        self.draw_order.push(DrawOp::Gradient(idx));
    }

    /// 添加一个阴影
    pub fn add_shadow(&mut self, shadow: ShadowPrimitive) {
        let idx = self.shadows.len();
        self.shadows.push(shadow);
        self.draw_order.push(DrawOp::Shadow(idx));
    }

    /// 添加一个图片图元
    pub fn add_image(&mut self, image: ImagePrimitive) {
        let idx = self.images.len();
        self.images.push(image);
        self.draw_order.push(DrawOp::Image(idx));
    }

    /// 添加一个 Glyph
    pub fn add_glyph(&mut self, glyph: GlyphPrimitive) {
        let idx = self.glyphs.len();
        self.glyphs.push(glyph);
        self.draw_order.push(DrawOp::Glyph(idx));
    }

    /// 添加一个 Filter
    pub fn add_filter(&mut self, filter: FilterPrimitive) {
        let idx = self.filters.len();
        self.filters.push(filter);
        self.draw_order.push(DrawOp::Filter(idx));
    }

    /// 添加一个混合模式
    pub fn add_blend_mode(&mut self, blend: BlendModePrimitive) {
        let idx = self.blend_modes.len();
        self.blend_modes.push(blend);
        self.draw_order.push(DrawOp::BlendMode(idx));
    }

    /// 图元总数
    pub fn len(&self) -> usize {
        self.clips.len()
            + self.fills.len()
            + self.rounded_rects.len()
            + self.path_fills.len()
            + self.path_strokes.len()
            + self.strokes.len()
            + self.gradients.len()
            + self.shadows.len()
            + self.images.len()
            + self.glyphs.len()
            + self.filters.len()
            + self.blend_modes.len()
            + self.transforms.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
            && self.fills.is_empty()
            && self.rounded_rects.is_empty()
            && self.path_fills.is_empty()
            && self.path_strokes.is_empty()
            && self.strokes.is_empty()
            && self.gradients.is_empty()
            && self.shadows.is_empty()
            && self.images.is_empty()
            && self.glyphs.is_empty()
            && self.filters.is_empty()
            && self.blend_modes.is_empty()
            && self.transforms.is_empty()
    }

    /// 添加变换图元。
    pub fn add_transform(&mut self, transform: TransformPrimitive) {
        let idx = self.transforms.len();
        self.transforms.push(transform);
        self.draw_order.push(DrawOp::Transform(idx));
    }
}

#[cfg(test)]
mod tests;
