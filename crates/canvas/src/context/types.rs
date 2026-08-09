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

    /// 为渐变变体添加颜色停止点（spec `CanvasGradient.addColorStop`）。
    /// Color/Pattern 变体为 no-op（非渐变样式无停止点概念）。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        match self {
            CanvasStyle::LinearGradient(g) => g.add_color_stop(offset, color),
            CanvasStyle::RadialGradient(g) => g.add_color_stop(offset, color),
            CanvasStyle::ConicGradient(g) => g.add_color_stop(offset, color),
            _ => {}
        }
    }

    /// 判断是否为渐变样式（光栅化路径分流用）。
    pub fn is_gradient(&self) -> bool {
        matches!(
            self,
            CanvasStyle::LinearGradient(_) | CanvasStyle::RadialGradient(_) | CanvasStyle::ConicGradient(_)
        )
    }

    /// 在设备空间某点 (x, y) 采样样式颜色（spec canvas 渐变光栅化的核心）。
    ///
    /// - Color：直接返回。
    /// - LinearGradient：将点投影到渐变线 (x0,y0)→(x1,y1) 得参数 t∈[0,1]，再线性插值停止点。
    ///   https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-createlineargradient
    /// - RadialGradient：以内心 (x0,y0,r0) 为基准，按距离归一化到外圆 (x1,y1,r1) 得 t。
    /// - ConicGradient：以中心 (cx,cy) 计相对 start_angle 的角度，归一化到 [0,1]。
    /// - Pattern：回落黑色（图案平铺光栅化 defer）。
    ///
    /// 偏移量超出 [0,1] 由 `sample_gradient_stops` 钳制到首/末停止点颜色（spec：渐变在停止点之外延伸为端点色）。
    pub fn sample_at(&self, x: f32, y: f32) -> Color {
        match self {
            CanvasStyle::Color(c) => *c,
            CanvasStyle::LinearGradient(g) => {
                let dx = g.x1 - g.x0;
                let dy = g.y1 - g.y0;
                let len2 = dx * dx + dy * dy;
                let t = if len2 < f32::EPSILON {
                    0.0
                } else {
                    ((x - g.x0) * dx + (y - g.y0) * dy) / len2
                };
                sample_gradient_stops(&g.stops, t)
            }
            CanvasStyle::RadialGradient(g) => {
                let ddx = x - g.x0;
                let ddy = y - g.y0;
                let dist = (ddx * ddx + ddy * ddy).sqrt();
                let span = g.r1 - g.r0;
                let t = if span.abs() < f32::EPSILON {
                    0.0
                } else {
                    (dist - g.r0) / span
                };
                sample_gradient_stops(&g.stops, t)
            }
            CanvasStyle::ConicGradient(g) => {
                let mut ang = (y - g.cy).atan2(x - g.cx) - g.start_angle;
                // 归一化到 [0, 2π)
                while ang < 0.0 {
                    ang += std::f32::consts::TAU;
                }
                while ang >= std::f32::consts::TAU {
                    ang -= std::f32::consts::TAU;
                }
                sample_gradient_stops(&g.stops, ang / std::f32::consts::TAU)
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── 枚举基础测试 ──────────────────────────────────────

    #[test]
    fn test_font_weight_equality() {
        assert_eq!(FontWeight::Normal, FontWeight::Normal);
        assert_ne!(FontWeight::Normal, FontWeight::Bold);
    }

    #[test]
    fn test_font_style_equality() {
        assert_eq!(FontStyle::Normal, FontStyle::Normal);
        assert_ne!(FontStyle::Normal, FontStyle::Italic);
    }

    #[test]
    fn test_text_align_variants() {
        assert_ne!(TextAlign::Start, TextAlign::End);
        assert_ne!(TextAlign::Left, TextAlign::Right);
        assert_ne!(TextAlign::Center, TextAlign::Start);
    }

    #[test]
    fn test_text_baseline_variants() {
        assert_ne!(TextBaseline::Top, TextBaseline::Middle);
        assert_ne!(TextBaseline::Alphabetic, TextBaseline::Bottom);
    }

    #[test]
    fn test_text_direction_default() {
        assert_eq!(TextDirection::default(), TextDirection::Inherit);
    }

    #[test]
    fn test_line_join_default() {
        assert_eq!(LineJoin::default(), LineJoin::Miter);
    }

    #[test]
    fn test_line_cap_default() {
        assert_eq!(LineCap::default(), LineCap::Butt);
    }

    #[test]
    fn test_composite_operation_default() {
        assert_eq!(CompositeOperation::default(), CompositeOperation::SourceOver);
    }

    #[test]
    fn test_pattern_repetition_default() {
        assert_eq!(PatternRepetition::default(), PatternRepetition::Repeat);
    }

    // ── FontDescriptor 测试 ───────────────────────────────

    #[test]
    fn test_font_descriptor_default() {
        let desc = FontDescriptor::default();
        assert_eq!(desc.family, "sans-serif");
        assert!((desc.size - 10.0).abs() < f32::EPSILON);
        assert_eq!(desc.weight, FontWeight::Normal);
        assert_eq!(desc.style, FontStyle::Normal);
    }

    #[test]
    fn test_font_descriptor_clone() {
        let desc = FontDescriptor {
            family: "serif".into(),
            size: 14.0,
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
        };
        let cloned = desc.clone();
        assert_eq!(cloned.family, "serif");
        assert_eq!(cloned.size, 14.0);
        assert_eq!(cloned.weight, FontWeight::Bold);
        assert_eq!(cloned.style, FontStyle::Italic);
    }

    #[test]
    fn test_font_descriptor_debug() {
        let desc = FontDescriptor::default();
        let debug = format!("{:?}", desc);
        assert!(debug.contains("sans-serif"));
    }

    // ── Transform2D 测试 ──────────────────────────────────

    #[test]
    fn test_transform_identity() {
        let t = Transform2D::identity();
        assert_eq!(t.a, 1.0);
        assert_eq!(t.b, 0.0);
        assert_eq!(t.c, 0.0);
        assert_eq!(t.d, 1.0);
        assert_eq!(t.e, 0.0);
        assert_eq!(t.f, 0.0);
    }

    #[test]
    fn test_transform_default_is_identity() {
        let t = Transform2D::default();
        let id = Transform2D::identity();
        assert_eq!(t.a, id.a);
        assert_eq!(t.b, id.b);
        assert_eq!(t.c, id.c);
        assert_eq!(t.d, id.d);
        assert_eq!(t.e, id.e);
        assert_eq!(t.f, id.f);
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform2D::translate(10.0, 20.0);
        assert_eq!(t.a, 1.0);
        assert_eq!(t.d, 1.0);
        assert_eq!(t.e, 10.0);
        assert_eq!(t.f, 20.0);
        // translate 应保持点平移
        let (x, y) = t.transform_point(5.0, 5.0);
        assert!((x - 15.0).abs() < f32::EPSILON);
        assert!((y - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_scale() {
        let t = Transform2D::scale(2.0, 3.0);
        assert_eq!(t.a, 2.0);
        assert_eq!(t.d, 3.0);
        let (x, y) = t.transform_point(10.0, 10.0);
        assert!((x - 20.0).abs() < f32::EPSILON);
        assert!((y - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_rotate_90() {
        let t = Transform2D::rotate(std::f32::consts::FRAC_PI_2);
        let (x, y) = t.transform_point(1.0, 0.0);
        // 旋转 90°: (1,0) → (0,1)
        assert!(x.abs() < 0.001, "x should be ~0, got {x}");
        assert!((y - 1.0).abs() < 0.001, "y should be ~1, got {y}");
    }

    #[test]
    fn test_transform_multiply_identity() {
        let id = Transform2D::identity();
        let t = Transform2D::translate(5.0, 10.0);
        let result = id.multiply(&t);
        assert!((result.e - 5.0).abs() < f32::EPSILON);
        assert!((result.f - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_multiply_translate_scale() {
        let scale = Transform2D::scale(2.0, 3.0);
        let translate = Transform2D::translate(10.0, 20.0);
        let result = scale.multiply(&translate);
        // multiply = self * other 的矩阵乘法:
        // result.e = scale.a * translate.e + scale.c * translate.f + scale.e
        //          = 2*10 + 0*20 + 0 = 20
        // result.f = scale.b * translate.e + scale.d * translate.f + scale.f
        //          = 0*10 + 3*20 + 0 = 60
        // 对点 (1,1) 应用：result * (1,1) = (2*1+0*1+20, 0*1+3*1+60) = (22, 63)
        let (x, y) = result.transform_point(1.0, 1.0);
        assert!((x - 22.0).abs() < 0.01, "x should be 22, got {x}");
        assert!((y - 63.0).abs() < 0.01, "y should be 63, got {y}");
    }

    #[test]
    fn test_transform_clone_copy() {
        let t = Transform2D::translate(1.0, 2.0);
        let copied = t; // Copy
        assert_eq!(copied.e, 1.0);
        let cloned = t; // Clone (Copy implies Clone)
        assert_eq!(cloned.f, 2.0);
    }

    #[test]
    fn test_transform_debug() {
        let t = Transform2D::identity();
        let debug = format!("{:?}", t);
        assert!(debug.contains("Transform2D"));
    }

    // ── LinearGradient 测试 ───────────────────────────────

    #[test]
    fn test_linear_gradient_new() {
        let g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        assert_eq!(g.x0, 0.0);
        assert_eq!(g.x1, 100.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_linear_gradient_add_color_stop() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[0].offset, 0.0);
        assert_eq!(g.stops[1].offset, 1.0);
    }

    #[test]
    fn test_linear_gradient_sample_empty() {
        let g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        let c = g.sample_color(0.5);
        assert_eq!(c, Color::BLACK);
    }

    #[test]
    fn test_linear_gradient_sample_single_stop() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.5, Color::rgb(255, 0, 0));
        assert_eq!(g.sample_color(0.0), Color::rgb(255, 0, 0));
        assert_eq!(g.sample_color(1.0), Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_linear_gradient_sample_two_stops() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.g, 128);
        assert_eq!(mid.b, 128);
    }

    #[test]
    fn test_linear_gradient_sample_clamp() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        // offset < 0 → clamped to first stop
        assert_eq!(g.sample_color(-1.0), Color::rgb(255, 0, 0));
        // offset > 1 → clamped to last stop
        assert_eq!(g.sample_color(2.0), Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_linear_gradient_clone() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 100.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        let cloned = g.clone();
        assert_eq!(cloned.stops.len(), 1);
    }

    // ── RadialGradient 测试 ───────────────────────────────

    #[test]
    fn test_radial_gradient_new() {
        let g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        assert_eq!(g.x0, 0.0);
        assert_eq!(g.r1, 50.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_radial_gradient_sample() {
        let mut g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.b, 128);
    }

    // ── ConicGradient 测试 ────────────────────────────────

    #[test]
    fn test_conic_gradient_new() {
        let g = ConicGradient::new(0.0, 50.0, 50.0);
        assert_eq!(g.cx, 50.0);
        assert_eq!(g.cy, 50.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_conic_gradient_sample() {
        let mut g = ConicGradient::new(0.0, 0.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
    }

    // ── CanvasPattern 测试 ────────────────────────────────

    #[test]
    fn test_canvas_pattern_new() {
        let img = ImageData {
            width: 10,
            height: 10,
            data: vec![0u8; 400],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
        assert_eq!(pattern.repetition, PatternRepetition::Repeat);
        assert_eq!(pattern.image_data.width, 10);
    }

    #[test]
    fn test_canvas_pattern_no_repeat() {
        let img = ImageData {
            width: 5,
            height: 5,
            data: vec![0u8; 100],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::NoRepeat);
        assert_eq!(pattern.repetition, PatternRepetition::NoRepeat);
    }

    // ── CanvasStyle 测试 ──────────────────────────────────

    #[test]
    fn test_canvas_style_default_black() {
        let style = CanvasStyle::default_black();
        let color = style.resolve_color();
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn test_canvas_style_color_resolve() {
        let style = CanvasStyle::Color(Color::rgb(128, 64, 32));
        assert_eq!(style.resolve_color(), Color::rgb(128, 64, 32));
    }

    #[test]
    fn test_canvas_style_linear_gradient_resolve() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let style = CanvasStyle::LinearGradient(g);
        let c = style.resolve_color();
        // offset 0.5 → mid-gray
        assert_eq!(c.r, 128);
    }

    #[test]
    fn test_canvas_style_radial_gradient_resolve() {
        let mut g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        let style = CanvasStyle::RadialGradient(g);
        let c = style.resolve_color();
        assert!(c.r > 0);
    }

    #[test]
    fn test_canvas_style_conic_gradient_resolve() {
        let mut g = ConicGradient::new(0.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(100, 100, 100));
        let style = CanvasStyle::ConicGradient(g);
        let c = style.resolve_color();
        assert_eq!(c.r, 100);
    }

    #[test]
    fn test_canvas_style_pattern_resolve() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
        let style = CanvasStyle::Pattern(pattern);
        assert_eq!(style.resolve_color(), Color::BLACK);
    }

    #[test]
    fn test_canvas_style_clone() {
        let style = CanvasStyle::Color(Color::rgb(1, 2, 3));
        let cloned = style.clone();
        assert_eq!(cloned.resolve_color(), Color::rgb(1, 2, 3));
    }

    // ── TextMetrics 测试 ──────────────────────────────────

    #[test]
    fn test_text_metrics_fields() {
        let metrics = TextMetrics {
            width: 120.5,
            actual_bounding_box_ascent: 10.0,
            actual_bounding_box_descent: 3.0,
        };
        assert!((metrics.width - 120.5).abs() < f32::EPSILON);
        assert_eq!(metrics.actual_bounding_box_ascent, 10.0);
        assert_eq!(metrics.actual_bounding_box_descent, 3.0);
    }

    #[test]
    fn test_text_metrics_clone() {
        let m = TextMetrics {
            width: 50.0,
            actual_bounding_box_ascent: 8.0,
            actual_bounding_box_descent: 2.0,
        };
        let cloned = m.clone();
        assert_eq!(cloned.width, m.width);
    }

    // ── ImageData 测试 ────────────────────────────────────

    #[test]
    fn test_image_data_fields() {
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255; 16], // 2x2 RGBA = 16 bytes
        };
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.data.len(), 16);
    }

    #[test]
    fn test_image_data_clone() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![128, 64, 32, 255],
        };
        let cloned = img.clone();
        assert_eq!(cloned.data, img.data);
    }

    #[test]
    fn test_image_data_debug() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![0; 4],
        };
        let debug = format!("{:?}", img);
        assert!(debug.contains("ImageData"));
    }

    // ── GradientStop 测试 ─────────────────────────────────

    #[test]
    fn test_gradient_stop_fields() {
        let stop = GradientStop {
            offset: 0.5,
            color: Color::rgb(128, 128, 128),
        };
        assert!((stop.offset - 0.5).abs() < f32::EPSILON);
    }

    // ── sample_gradient_stops 间接测试（通过渐变类型）──

    #[test]
    fn test_gradient_three_stops_interpolation() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(0.5, Color::rgb(128, 128, 128));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        // at 0.25 → between stop0 and stop1
        let c = g.sample_color(0.25);
        assert_eq!(c.r, 64);
        // at 0.75 → between stop1 and stop2
        let c = g.sample_color(0.75);
        assert_eq!(c.r, 192);
    }

    #[test]
    fn test_gradient_identical_stops() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(128, 128, 128));
        g.add_color_stop(0.0, Color::rgb(128, 128, 128));
        // span ≈ 0 → should return first stop's color
        let c = g.sample_color(0.0);
        assert_eq!(c.r, 128);
    }
}
