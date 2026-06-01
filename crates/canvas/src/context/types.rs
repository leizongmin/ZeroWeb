//! Canvas 2D 类型定义 — 枚举、结构体、辅助函数。

use zero_render_foundation::color::Color;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::path::Path2D;

/// 字体粗细。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    /// 正常。
    Normal,
    /// 粗体。
    Bold,
}

/// 字体样式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    /// 正常。
    Normal,
    /// 斜体。
    Italic,
}

/// 字体描述符。
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    /// 字体族。
    pub family: String,
    /// 字体大小。
    pub size: f32,
    /// 字体粗细。
    pub weight: FontWeight,
    /// 字体样式。
    pub style: FontStyle,
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            family: "sans-serif".to_string(),
            size: 10.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }
}

/// 2D 仿射变换矩阵。
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    /// 矩阵元素 a (scale X / cos rotate)。
    pub a: f32,
    /// 矩阵元素 b (skew Y / sin rotate)。
    pub b: f32,
    /// 矩阵元素 c (skew X / -sin rotate)。
    pub c: f32,
    /// 矩阵元素 d (scale Y / cos rotate)。
    pub d: f32,
    /// 平移 X。
    pub e: f32,
    /// 平移 Y。
    pub f: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

impl Transform2D {
    /// 单位矩阵。
    pub fn identity() -> Self {
        Self::default()
    }

    /// 平移变换。
    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            e: tx,
            f: ty,
            ..Self::default()
        }
    }

    /// 缩放变换。
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            d: sy,
            ..Self::default()
        }
    }

    /// 旋转变换（弧度）。
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// 矩阵乘法：self * other。
    pub fn multiply(&self, other: &Transform2D) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// 变换点。
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }
}

/// 文本对齐。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    /// 起始对齐。
    Start,
    /// 末尾对齐。
    End,
    /// 左对齐。
    Left,
    /// 右对齐。
    Right,
    /// 居中对齐。
    Center,
}

/// 文本基线。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBaseline {
    /// 顶部。
    Top,
    /// 中部。
    Middle,
    /// 字母基线。
    Alphabetic,
    /// 底部。
    Bottom,
}

/// 文本方向。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDirection {
    /// 从左到右。
    Ltr,
    /// 从右到左。
    Rtl,
    /// 继承（默认）。
    #[default]
    Inherit,
}

/// 线段连接样式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineJoin {
    /// 默认：尖角连接。
    #[default]
    Miter,
    /// 圆角连接。
    Round,
    /// 斜角连接。
    Bevel,
}

/// 线段端点样式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineCap {
    /// 默认：平头端点。
    #[default]
    Butt,
    /// 圆头端点。
    Round,
    /// 方头端点（延伸半个线宽）。
    Square,
}

/// 合成操作模式 — 控制 Canvas 绘制时新图元与已有内容的混合方式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CompositeOperation {
    /// 默认：新图元绘制在已有内容之上。
    #[default]
    SourceOver,
    /// 新图元只绘制在透明区域。
    DestinationOver,
    /// 清除新图元与已有内容重叠的区域。
    DestinationOut,
    /// 新图元与已有内容重叠的部分保留已有内容。
    DestinationAtop,
    /// 新图元与已有内容的重叠区域显示已有内容。
    DestinationIn,
    /// 新图元与已有内容重叠区域显示新图元，其余清除。
    SourceIn,
    /// 新图元与已有内容重叠区域显示新图元。
    SourceAtop,
    /// 新图元和已有内容取较亮值。
    Lighter,
    /// 新图元复制到输出，忽略已有内容。
    Copy,
    /// 新图元和已有内容取异或。
    Xor,
    /// 新图元乘以已有内容（变暗）。
    Multiply,
    /// 新图元与已有内容取屏幕混合（变亮）。
    Screen,
    /// 新图元与已有内容叠加混合。
    Overlay,
    /// 新图层变暗模式。
    Darken,
    /// 新图层变亮模式。
    Lighten,
    /// 新图层颜色减淡。
    ColorDodge,
    /// 新图层颜色加深。
    ColorBurn,
    /// 新图层强光模式。
    HardLight,
    /// 新图层柔光模式。
    SoftLight,
    /// 新图层差值模式。
    Difference,
    /// 新图层排除模式。
    Exclusion,
    /// 新图层色相模式。
    Hue,
    /// 新图层饱和度模式。
    Saturation,
    /// 新图层颜色模式。
    Color,
    /// 新图层亮度模式。
    Luminosity,
}

/// 渐变停止点。
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// 偏移量 [0.0, 1.0]。
    pub offset: f32,
    /// 颜色。
    pub color: Color,
}

/// 线性渐变 — 从起点到终点的颜色过渡。
#[derive(Debug, Clone)]
pub struct LinearGradient {
    /// 起点 X。
    pub x0: f32,
    /// 起点 Y。
    pub y0: f32,
    /// 终点 X。
    pub x1: f32,
    /// 终点 Y。
    pub y1: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// 创建线性渐变。
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 径向渐变 — 从内圆到外圆的颜色过渡。
#[derive(Debug, Clone)]
pub struct RadialGradient {
    /// 内圆圆心 X。
    pub x0: f32,
    /// 内圆圆心 Y。
    pub y0: f32,
    /// 内圆半径。
    pub r0: f32,
    /// 外圆圆心 X。
    pub x1: f32,
    /// 外圆圆心 Y。
    pub y1: f32,
    /// 外圆半径。
    pub r1: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// 创建径向渐变。
    pub fn new(x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> Self {
        Self {
            x0,
            y0,
            r0,
            x1,
            y1,
            r1,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 锥形渐变 — 围绕中心点按角度过渡颜色。
#[derive(Debug, Clone)]
pub struct ConicGradient {
    /// 起始角度（弧度）。
    pub start_angle: f32,
    /// 中心 X 坐标。
    pub cx: f32,
    /// 中心 Y 坐标。
    pub cy: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl ConicGradient {
    /// 创建锥形渐变。
    pub fn new(start_angle: f32, cx: f32, cy: f32) -> Self {
        Self {
            start_angle,
            cx,
            cy,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 图案重复模式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PatternRepetition {
    /// 水平和垂直方向都重复。
    #[default]
    Repeat,
    /// 只在水平方向重复。
    RepeatX,
    /// 只在垂直方向重复。
    RepeatY,
    /// 不重复。
    NoRepeat,
}

/// 图案 — 从 ImageData 创建的平铺图案。
#[derive(Debug, Clone)]
pub struct CanvasPattern {
    /// 图案源图像数据。
    pub image_data: ImageData,
    /// 重复模式。
    pub repetition: PatternRepetition,
}

impl CanvasPattern {
    /// 创建图案。
    pub fn new(image_data: ImageData, repetition: PatternRepetition) -> Self {
        Self { image_data, repetition }
    }
}

/// Canvas 填充/描边样式。
#[derive(Debug, Clone)]
pub enum CanvasStyle {
    /// 纯色。
    Color(Color),
    /// 线性渐变。
    LinearGradient(LinearGradient),
    /// 径向渐变。
    RadialGradient(RadialGradient),
    /// 锥形渐变。
    ConicGradient(ConicGradient),
    /// 图案。
    Pattern(CanvasPattern),
}

impl CanvasStyle {
    /// 默认样式：不透明黑色。
    pub fn default_black() -> Self {
        CanvasStyle::Color(Color::BLACK)
    }

    /// 解析为有效颜色。
    ///
    /// 对于 Color 变体直接使用；
    /// 对于渐变变体在指定偏移量处采样近似颜色；
    /// 对于 Pattern 返回黑色作为回退。
    pub fn resolve_color(&self) -> Color {
        match self {
            CanvasStyle::Color(c) => *c,
            CanvasStyle::LinearGradient(g) => g.sample_color(0.5),
            CanvasStyle::RadialGradient(g) => g.sample_color(0.5),
            CanvasStyle::ConicGradient(g) => g.sample_color(0.0),
            CanvasStyle::Pattern(_) => Color::BLACK,
        }
    }
}

/// 渐变停止点颜色采样辅助函数。
///
/// 将偏移量限制在 [0.0, 1.0]，找到包围偏移量的两个停止点并线性插值。
fn sample_gradient_stops(stops: &[GradientStop], offset: f32) -> Color {
    if stops.is_empty() {
        return Color::BLACK;
    }
    if stops.len() == 1 {
        return stops[0].color;
    }
    let t = offset.clamp(0.0, 1.0);
    // 偏移量在第一个停止点之前
    if t <= stops[0].offset {
        return stops[0].color;
    }
    // 偏移量在最后一个停止点之后
    if t >= stops[stops.len() - 1].offset {
        return stops[stops.len() - 1].color;
    }
    // 找到包围 t 的两个停止点
    for i in 0..stops.len() - 1 {
        if t >= stops[i].offset && t <= stops[i + 1].offset {
            let span = stops[i + 1].offset - stops[i].offset;
            if span < f32::EPSILON {
                return stops[i].color;
            }
            let frac = (t - stops[i].offset) / span;
            let c0 = stops[i].color;
            let c1 = stops[i + 1].color;
            return Color::rgba(
                lerp_u8(c0.r, c1.r, frac),
                lerp_u8(c0.g, c1.g, frac),
                lerp_u8(c0.b, c1.b, frac),
                lerp_u8(c0.a, c1.a, frac),
            );
        }
    }
    stops[stops.len() - 1].color
}

/// 线性插值两个 u8 值。
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

/// Canvas 状态（用于 save/restore）。
#[derive(Debug, Clone)]
pub(crate) struct CanvasState {
    pub(crate) fill_style: CanvasStyle,
    pub(crate) stroke_style: CanvasStyle,
    pub(crate) line_width: f32,
    pub(crate) font: FontDescriptor,
    pub(crate) global_alpha: f32,
    pub(crate) transform: Transform2D,
    pub(crate) composite_operation: CompositeOperation,
    pub(crate) shadow_color: Color,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_offset_x: f32,
    pub(crate) shadow_offset_y: f32,
    pub(crate) line_dash: Vec<f32>,
    pub(crate) line_dash_offset: f32,
    pub(crate) line_join: LineJoin,
    pub(crate) line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    pub(crate) image_smoothing_enabled: bool,
    /// 文本对齐。
    pub(crate) text_align: TextAlign,
    /// 文本基线。
    pub(crate) text_baseline: TextBaseline,
    /// 斜接限制。
    pub(crate) miter_limit: f32,
    /// 文本方向。
    pub(crate) direction: TextDirection,
}

/// Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。
pub struct CanvasContext {
    /// 画布宽度。
    pub(crate) width: u32,
    /// 画布高度。
    pub(crate) height: u32,
    /// 当前填充样式。
    pub(crate) fill_style: CanvasStyle,
    /// 当前描边样式。
    pub(crate) stroke_style: CanvasStyle,
    /// 当前线宽。
    pub(crate) line_width: f32,
    /// 当前字体。
    pub(crate) font: FontDescriptor,
    /// 全局透明度。
    pub(crate) global_alpha: f32,
    /// 变换矩阵。
    pub(crate) transform: Transform2D,
    /// 渲染图元列表。
    pub(crate) primitives: RenderPrimitives,
    /// 状态栈（用于 save/restore）。
    pub(crate) state_stack: Vec<CanvasState>,
    /// 当前路径。
    pub(crate) current_path: Path2D,
    /// 像素缓冲区（RGBA，宽度 × 高度 × 4 字节）。
    pub(crate) pixel_buffer: Vec<u8>,
    /// 当前合成操作模式。
    pub(crate) composite_operation: CompositeOperation,
    /// 当前裁剪路径（如果有）。
    pub(crate) clip_path: Option<Path2D>,
    /// 阴影颜色。
    pub(crate) shadow_color: Color,
    /// 阴影模糊半径。
    pub(crate) shadow_blur: f32,
    /// 阴影水平偏移。
    pub(crate) shadow_offset_x: f32,
    /// 阴影垂直偏移。
    pub(crate) shadow_offset_y: f32,
    /// 线段虚线模式。
    pub(crate) line_dash: Vec<f32>,
    /// 线段虚线偏移。
    pub(crate) line_dash_offset: f32,
    /// 线段连接样式。
    pub(crate) line_join: LineJoin,
    /// 线段端点样式。
    pub(crate) line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    pub(crate) image_smoothing_enabled: bool,
    /// 文本对齐。
    pub(crate) text_align: TextAlign,
    /// 文本基线。
    pub(crate) text_baseline: TextBaseline,
    /// 斜接限制。
    pub(crate) miter_limit: f32,
    /// 文本方向。
    pub(crate) direction: TextDirection,
}

/// 文本度量。
#[derive(Debug, Clone)]
pub struct TextMetrics {
    /// 文本宽度。
    pub width: f32,
    /// 实际边界框上方。
    pub actual_bounding_box_ascent: f32,
    /// 实际边界框下方。
    pub actual_bounding_box_descent: f32,
}

/// 图像数据。
#[derive(Debug, Clone)]
pub struct ImageData {
    /// 宽度。
    pub width: u32,
    /// 高度。
    pub height: u32,
    /// RGBA 像素数据。
    pub data: Vec<u8>,
}
