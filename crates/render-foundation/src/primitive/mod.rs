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

/// 渐变图元
#[derive(Debug, Clone)]
pub struct GradientPrimitive {
    /// 渐变区域
    pub rect: Rect,
    /// 渐变类型
    pub kind: GradientKind,
    /// 色标列表
    pub stops: Vec<GradientStop>,
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
}

/// 图片图元
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// 图片绘制区域
    pub rect: Rect,
    /// 图片缓存键
    pub image_key: ImageKey,
}

/// Glyph 图元 — 单个字符的渲染指令
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
    /// Glyph ID
    pub glyph_id: u32,
    /// 字体 ID
    pub font_id: FontId,
    /// 预缓存位图宽度（可选）
    pub bitmap_width: Option<u32>,
    /// 预缓存位图高度（可选）
    pub bitmap_height: Option<u32>,
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
}

impl RenderPrimitives {
    /// 创建空的图元列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个填充矩形
    pub fn add_fill(&mut self, rect: Rect, color: Color) {
        self.fills.push(FillPrimitive { rect, color });
    }

    /// 添加一个圆角矩形
    pub fn add_rounded_rect(&mut self, rounded: RoundedRectPrimitive) {
        self.rounded_rects.push(rounded);
    }

    /// 添加一个路径填充图元。
    pub fn add_path_fill(&mut self, vertices: Vec<f32>, color: Color) {
        self.path_fills.push(PathFillPrimitive { vertices, color });
    }

    /// 添加一个路径描边图元。
    pub fn add_path_stroke(&mut self, vertices: Vec<f32>, color: Color, line_width: f32, closed: bool) {
        self.path_strokes.push(PathStrokePrimitive {
            vertices,
            color,
            line_width,
            closed,
        });
    }

    /// 添加一个描边线段
    pub fn add_stroke(&mut self, stroke: StrokePrimitive) {
        self.strokes.push(stroke);
    }

    /// 添加一个裁剪区域
    pub fn add_clip(&mut self, rect: Rect) {
        self.clips.push(ClipPrimitive { rect });
    }

    /// 添加一个渐变
    pub fn add_gradient(&mut self, gradient: GradientPrimitive) {
        self.gradients.push(gradient);
    }

    /// 添加一个阴影
    pub fn add_shadow(&mut self, shadow: ShadowPrimitive) {
        self.shadows.push(shadow);
    }

    /// 添加一个图片图元
    pub fn add_image(&mut self, image: ImagePrimitive) {
        self.images.push(image);
    }

    /// 添加一个 Glyph
    pub fn add_glyph(&mut self, glyph: GlyphPrimitive) {
        self.glyphs.push(glyph);
    }

    /// 添加一个 Filter
    pub fn add_filter(&mut self, filter: FilterPrimitive) {
        self.filters.push(filter);
    }

    /// 添加一个混合模式
    pub fn add_blend_mode(&mut self, blend: BlendModePrimitive) {
        self.blend_modes.push(blend);
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
    }
}

#[cfg(test)]
mod tests;
