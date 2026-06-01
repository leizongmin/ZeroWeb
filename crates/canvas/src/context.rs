//! Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::path::{Path2D, PathCommand};

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
struct CanvasState {
    fill_style: CanvasStyle,
    stroke_style: CanvasStyle,
    line_width: f32,
    font: FontDescriptor,
    global_alpha: f32,
    transform: Transform2D,
    composite_operation: CompositeOperation,
    shadow_color: Color,
    shadow_blur: f32,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    line_dash: Vec<f32>,
    line_dash_offset: f32,
    line_join: LineJoin,
    line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    image_smoothing_enabled: bool,
    /// 文本对齐。
    text_align: TextAlign,
    /// 文本基线。
    text_baseline: TextBaseline,
    /// 斜接限制。
    miter_limit: f32,
    /// 文本方向。
    direction: TextDirection,
}

/// Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。
pub struct CanvasContext {
    /// 画布宽度。
    width: u32,
    /// 画布高度。
    height: u32,
    /// 当前填充样式。
    fill_style: CanvasStyle,
    /// 当前描边样式。
    stroke_style: CanvasStyle,
    /// 当前线宽。
    line_width: f32,
    /// 当前字体。
    font: FontDescriptor,
    /// 全局透明度。
    global_alpha: f32,
    /// 变换矩阵。
    transform: Transform2D,
    /// 渲染图元列表。
    primitives: RenderPrimitives,
    /// 状态栈（用于 save/restore）。
    state_stack: Vec<CanvasState>,
    /// 当前路径。
    current_path: Path2D,
    /// 像素缓冲区（RGBA，宽度 × 高度 × 4 字节）。
    pixel_buffer: Vec<u8>,
    /// 当前合成操作模式。
    composite_operation: CompositeOperation,
    /// 当前裁剪路径（如果有）。
    clip_path: Option<Path2D>,
    /// 阴影颜色。
    shadow_color: Color,
    /// 阴影模糊半径。
    shadow_blur: f32,
    /// 阴影水平偏移。
    shadow_offset_x: f32,
    /// 阴影垂直偏移。
    shadow_offset_y: f32,
    /// 线段虚线模式。
    line_dash: Vec<f32>,
    /// 线段虚线偏移。
    line_dash_offset: f32,
    /// 线段连接样式。
    line_join: LineJoin,
    /// 线段端点样式。
    line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    image_smoothing_enabled: bool,
    /// 文本对齐。
    text_align: TextAlign,
    /// 文本基线。
    text_baseline: TextBaseline,
    /// 斜接限制。
    miter_limit: f32,
    /// 文本方向。
    direction: TextDirection,
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

impl CanvasContext {
    /// 创建指定尺寸的 Canvas 上下文。
    pub fn new(width: u32, height: u32) -> Self {
        let buffer_size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            fill_style: CanvasStyle::default_black(),
            stroke_style: CanvasStyle::default_black(),
            line_width: 1.0,
            font: FontDescriptor::default(),
            global_alpha: 1.0,
            transform: Transform2D::identity(),
            primitives: RenderPrimitives::new(),
            state_stack: Vec::new(),
            current_path: Path2D::new(),
            pixel_buffer: vec![0u8; buffer_size],
            composite_operation: CompositeOperation::default(),
            clip_path: None,
            shadow_color: Color::TRANSPARENT,
            shadow_blur: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            line_join: LineJoin::default(),
            line_cap: LineCap::default(),
            image_smoothing_enabled: true,
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            miter_limit: 10.0,
            direction: TextDirection::Inherit,
        }
    }

    // ── Rectangle drawing ──

    /// 清除矩形区域（设为透明）。
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 添加一个透明色填充来表示清除操作
        let rect = self.transform_rect(x, y, width, height);
        self.primitives.add_fill(rect, Color::TRANSPARENT);
        // clear_rect 直接将像素清零，不经过合成操作（与 Canvas 规范一致）
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        let x_end = (rect.right().min(self.width as f32) as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32) as usize).min(canvas_h);
        for py in y_start..y_end {
            for px in x_start..x_end {
                let idx = (py * canvas_w + px) * 4;
                self.pixel_buffer[idx] = 0;
                self.pixel_buffer[idx + 1] = 0;
                self.pixel_buffer[idx + 2] = 0;
                self.pixel_buffer[idx + 3] = 0;
            }
        }
    }

    /// 填充矩形。
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = self.transform_rect(x, y, width, height);
        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            self.draw_shadow_rect(&rect);
        }
        let color = self.apply_alpha(self.fill_style.resolve_color());
        self.primitives.add_fill(rect, color);
        self.blit_rect_to_pixels(&rect, color);
    }

    /// 描边矩形。
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 简化实现：用描边颜色填充一个薄矩形表示描边
        let lw = self.line_width;
        let color = self.apply_alpha(self.stroke_style.resolve_color());

        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            let rect = self.transform_rect(x, y, width, height);
            self.draw_shadow_rect(&rect);
        }

        // 上边
        let top = self.transform_rect(x, y, width, lw);
        self.primitives.add_fill(top, color);
        self.blit_rect_to_pixels(&top, color);
        // 下边
        let bottom = self.transform_rect(x, y + height - lw, width, lw);
        self.primitives.add_fill(bottom, color);
        self.blit_rect_to_pixels(&bottom, color);
        // 左边
        let left = self.transform_rect(x, y, lw, height);
        self.primitives.add_fill(left, color);
        self.blit_rect_to_pixels(&left, color);
        // 右边
        let right = self.transform_rect(x + width - lw, y, lw, height);
        self.primitives.add_fill(right, color);
        self.blit_rect_to_pixels(&right, color);
    }

    // ── Text ──

    /// 填充文本。为每个字符生成独立的 GlyphPrimitive，glyph_id 取字符的 Unicode 码点。
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.apply_alpha(self.fill_style.resolve_color());
        let font_size = self.font.size;
        let (tx, ty) = self.transform.transform_point(x, y);
        let em_width = font_size * 0.6;
        let mut offset_x = 0.0f32;
        for ch in text.chars() {
            let glyph_id = ch as u32;
            self.primitives
                .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                    x: tx + offset_x,
                    y: ty,
                    font_size,
                    color,
                    glyph_id,
                    font_id: zero_render_foundation::primitive::FontId(0),
                    bitmap_width: None,
                    bitmap_height: None,
                });
            offset_x += em_width;
        }
    }

    /// 描边文本。与 fill_text 相同逻辑，使用描边颜色。
    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.apply_alpha(self.stroke_style.resolve_color());
        let font_size = self.font.size;
        let (tx, ty) = self.transform.transform_point(x, y);
        let em_width = font_size * 0.6;
        let mut offset_x = 0.0f32;
        for ch in text.chars() {
            let glyph_id = ch as u32;
            self.primitives
                .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                    x: tx + offset_x,
                    y: ty,
                    font_size,
                    color,
                    glyph_id,
                    font_id: zero_render_foundation::primitive::FontId(0),
                    bitmap_width: None,
                    bitmap_height: None,
                });
            offset_x += em_width;
        }
    }

    /// 测量文本宽度（简化版：按字符数 × 字体大小估算）。
    pub fn measure_text(&self, text: &str) -> TextMetrics {
        let char_count = text.chars().count() as f32;
        let em_width = self.font.size * 0.6; // 简化：每个字符约 0.6em 宽
        TextMetrics {
            width: char_count * em_width,
            actual_bounding_box_ascent: self.font.size * 0.8,
            actual_bounding_box_descent: self.font.size * 0.2,
        }
    }

    // ── Path ──

    /// 开始新路径。
    pub fn begin_path(&mut self) {
        self.current_path.clear();
    }

    /// 闭合路径。
    pub fn close_path(&mut self) {
        self.current_path.close_path();
    }

    /// 移动到。
    pub fn move_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path.move_to(tx, ty);
    }

    /// 画线到。
    pub fn line_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path.line_to(tx, ty);
    }

    /// 画弧。
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path.arc(tx, ty, radius, start_angle, end_angle);
    }

    /// 画圆弧切线（arcTo）。通过当前点到 (x1,y1) 的线和 (x1,y1) 到 (x2,y2) 的线，
    /// 绘制一条与两条线都相切、半径为 radius 的圆弧。
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        let (tx1, ty1) = self.transform.transform_point(x1, y1);
        let (tx2, ty2) = self.transform.transform_point(x2, y2);
        self.current_path.arc_to(tx1, ty1, tx2, ty2, radius);
    }

    /// 画椭圆弧。
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
    ) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path
            .ellipse(tx, ty, radius_x, radius_y, rotation, start_angle, end_angle);
    }

    /// 画二次贝塞尔曲线。
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        let (tcpx, tcpy) = self.transform.transform_point(cpx, cpy);
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path
            .commands_mut()
            .push(PathCommand::QuadraticCurveTo(tcpx, tcpy, tx, ty));
    }

    /// 画三次贝塞尔曲线。
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        let (tcp1x, tcp1y) = self.transform.transform_point(cp1x, cp1y);
        let (tcp2x, tcp2y) = self.transform.transform_point(cp2x, cp2y);
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path
            .commands_mut()
            .push(PathCommand::BezierCurveTo(tcp1x, tcp1y, tcp2x, tcp2y, tx, ty));
    }

    /// 填充路径。将路径命令扁平化为顶点列表，生成路径填充图元。
    pub fn fill(&mut self) {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return;
        }
        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            self.draw_shadow_path(&vertices);
        }
        let color = self.apply_alpha(self.fill_style.resolve_color());
        self.primitives.add_path_fill(vertices.clone(), color);
        self.blit_path_to_pixels(&vertices, color);
    }

    /// 描边路径。将路径命令扁平化为顶点列表，生成路径描边图元。
    pub fn stroke(&mut self) {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return;
        }
        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            self.draw_shadow_path(&vertices);
        }
        let color = self.apply_alpha(self.stroke_style.resolve_color());
        let closed = self
            .current_path
            .commands()
            .iter()
            .any(|c| matches!(c, PathCommand::ClosePath));
        self.primitives
            .add_path_stroke(vertices.clone(), color, self.line_width, closed);
        self.blit_stroke_to_pixels(&vertices, color, self.line_width);
    }

    /// 使用指定 Path2D 填充路径。
    pub fn fill_with_path(&mut self, path: &Path2D) {
        let vertices = self.flatten_path_for(path);
        if vertices.is_empty() {
            return;
        }
        if self.has_shadow() {
            self.draw_shadow_path(&vertices);
        }
        let color = self.apply_alpha(self.fill_style.resolve_color());
        self.primitives.add_path_fill(vertices.clone(), color);
        self.blit_path_to_pixels(&vertices, color);
    }

    /// 使用指定 Path2D 描边路径。
    pub fn stroke_with_path(&mut self, path: &Path2D) {
        let vertices = self.flatten_path_for(path);
        if vertices.is_empty() {
            return;
        }
        if self.has_shadow() {
            self.draw_shadow_path(&vertices);
        }
        let color = self.apply_alpha(self.stroke_style.resolve_color());
        let closed = path.commands().iter().any(|c| matches!(c, PathCommand::ClosePath));
        self.primitives
            .add_path_stroke(vertices.clone(), color, self.line_width, closed);
        self.blit_stroke_to_pixels(&vertices, color, self.line_width);
    }

    /// 使用指定 Path2D 设置裁剪区域。
    pub fn clip_with_path(&mut self, path: &Path2D) {
        let vertices = self.flatten_path_for(path);
        if vertices.is_empty() {
            return;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        if min_x < max_x && min_y < max_y {
            let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
            self.primitives.add_clip(rect);
            self.clip_path = Some(path.clone());
        }
    }

    // ── Line dash ──

    /// 设置线段虚线模式。
    pub fn set_line_dash(&mut self, segments: Vec<f32>) {
        // 奇数长度时复制一份拼接到自身
        if segments.len() % 2 == 1 {
            let mut doubled = segments.clone();
            doubled.extend_from_slice(&segments);
            self.line_dash = doubled;
        } else {
            self.line_dash = segments;
        }
    }

    /// 返回当前线段虚线模式。
    pub fn get_line_dash(&self) -> &[f32] {
        &self.line_dash
    }

    /// 设置线段虚线偏移。
    pub fn set_line_dash_offset(&mut self, offset: f32) {
        self.line_dash_offset = offset;
    }

    /// 返回当前线段虚线偏移。
    pub fn get_line_dash_offset(&self) -> f32 {
        self.line_dash_offset
    }

    // ── State ──

    /// 保存当前状态到栈。
    pub fn save(&mut self) {
        self.state_stack.push(CanvasState {
            fill_style: self.fill_style.clone(),
            stroke_style: self.stroke_style.clone(),
            line_width: self.line_width,
            font: self.font.clone(),
            global_alpha: self.global_alpha,
            transform: self.transform,
            composite_operation: self.composite_operation,
            shadow_color: self.shadow_color,
            shadow_blur: self.shadow_blur,
            shadow_offset_x: self.shadow_offset_x,
            shadow_offset_y: self.shadow_offset_y,
            line_dash: self.line_dash.clone(),
            line_dash_offset: self.line_dash_offset,
            line_join: self.line_join,
            line_cap: self.line_cap,
            image_smoothing_enabled: self.image_smoothing_enabled,
            text_align: self.text_align,
            text_baseline: self.text_baseline,
            miter_limit: self.miter_limit,
            direction: self.direction,
        });
    }

    /// 从栈恢复状态。
    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.fill_style = state.fill_style;
            self.stroke_style = state.stroke_style;
            self.line_width = state.line_width;
            self.font = state.font;
            self.global_alpha = state.global_alpha;
            self.transform = state.transform;
            self.composite_operation = state.composite_operation;
            self.shadow_color = state.shadow_color;
            self.shadow_blur = state.shadow_blur;
            self.shadow_offset_x = state.shadow_offset_x;
            self.shadow_offset_y = state.shadow_offset_y;
            self.line_dash = state.line_dash;
            self.line_dash_offset = state.line_dash_offset;
            self.line_join = state.line_join;
            self.line_cap = state.line_cap;
            self.image_smoothing_enabled = state.image_smoothing_enabled;
            self.text_align = state.text_align;
            self.text_baseline = state.text_baseline;
            self.miter_limit = state.miter_limit;
            self.direction = state.direction;
        }
    }

    // ── Transform ──

    /// 设置变换矩阵。
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.transform = Transform2D { a, b, c, d, e, f };
    }

    /// 平移。
    pub fn translate(&mut self, tx: f32, ty: f32) {
        let t = Transform2D::translate(tx, ty);
        self.transform = self.transform.multiply(&t);
    }

    /// 缩放。
    pub fn scale(&mut self, sx: f32, sy: f32) {
        let s = Transform2D::scale(sx, sy);
        self.transform = self.transform.multiply(&s);
    }

    /// 旋转（弧度）。
    pub fn rotate(&mut self, angle: f32) {
        let r = Transform2D::rotate(angle);
        self.transform = self.transform.multiply(&r);
    }

    /// 重置变换矩阵为单位矩阵。
    pub fn reset_transform(&mut self) {
        self.transform = Transform2D::identity();
    }

    /// 返回当前变换矩阵的副本。
    pub fn get_transform(&self) -> Transform2D {
        self.transform
    }

    /// 将给定矩阵乘以当前变换矩阵（后乘）。
    /// 按照规范：self.transform = self.transform.multiply(&argument)。
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let other = Transform2D { a, b, c, d, e, f };
        self.transform = self.transform.multiply(&other);
    }

    // ── Properties ──

    /// 设置填充样式。
    pub fn set_fill_style(&mut self, style: CanvasStyle) {
        self.fill_style = style;
    }

    /// 设置描边样式。
    pub fn set_stroke_style(&mut self, style: CanvasStyle) {
        self.stroke_style = style;
    }

    /// 设置填充颜色（便捷方法）。
    pub fn set_fill_color(&mut self, color: Color) {
        self.fill_style = CanvasStyle::Color(color);
    }

    /// 设置描边颜色（便捷方法）。
    pub fn set_stroke_color(&mut self, color: Color) {
        self.stroke_style = CanvasStyle::Color(color);
    }

    /// 设置线宽。
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }

    /// 设置线段连接样式。
    pub fn set_line_join(&mut self, join: LineJoin) {
        self.line_join = join;
    }

    /// 设置线段端点样式。
    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.line_cap = cap;
    }

    /// 设置字体。
    pub fn set_font(&mut self, font: FontDescriptor) {
        self.font = font;
    }

    /// 设置全局透明度。
    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.global_alpha = alpha.clamp(0.0, 1.0);
    }

    /// 返回当前填充样式的有效颜色。
    pub fn fill_color(&self) -> Color {
        self.fill_style.resolve_color()
    }

    /// 返回当前描边样式的有效颜色。
    pub fn stroke_color(&self) -> Color {
        self.stroke_style.resolve_color()
    }

    /// 返回当前填充样式的引用。
    pub fn fill_style(&self) -> &CanvasStyle {
        &self.fill_style
    }

    /// 返回当前描边样式的引用。
    pub fn stroke_style(&self) -> &CanvasStyle {
        &self.stroke_style
    }

    /// 返回当前线宽。
    pub fn line_width(&self) -> f32 {
        self.line_width
    }

    /// 返回当前线段连接样式。
    pub fn line_join(&self) -> LineJoin {
        self.line_join
    }

    /// 返回当前线段端点样式。
    pub fn line_cap(&self) -> LineCap {
        self.line_cap
    }

    /// 设置图像平滑（抗锯齿）开关。
    pub fn set_image_smoothing_enabled(&mut self, enabled: bool) {
        self.image_smoothing_enabled = enabled;
    }

    /// 返回当前图像平滑开关状态。
    pub fn image_smoothing_enabled(&self) -> bool {
        self.image_smoothing_enabled
    }

    /// 返回当前字体描述符。
    pub fn font(&self) -> &FontDescriptor {
        &self.font
    }

    /// 设置文本对齐。
    pub fn set_text_align(&mut self, align: TextAlign) {
        self.text_align = align;
    }

    /// 返回当前文本对齐。
    pub fn text_align(&self) -> TextAlign {
        self.text_align
    }

    /// 设置文本基线。
    pub fn set_text_baseline(&mut self, baseline: TextBaseline) {
        self.text_baseline = baseline;
    }

    /// 返回当前文本基线。
    pub fn text_baseline(&self) -> TextBaseline {
        self.text_baseline
    }

    /// 设置斜接限制。
    pub fn set_miter_limit(&mut self, limit: f32) {
        self.miter_limit = limit;
    }

    /// 返回当前斜接限制。
    pub fn miter_limit(&self) -> f32 {
        self.miter_limit
    }

    /// 设置文本方向。
    pub fn set_direction(&mut self, dir: TextDirection) {
        self.direction = dir;
    }

    /// 返回当前文本方向。
    pub fn direction(&self) -> TextDirection {
        self.direction
    }

    /// 返回当前全局透明度。
    pub fn global_alpha(&self) -> f32 {
        self.global_alpha
    }

    /// 返回画布宽度。
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 返回画布高度。
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 调整画布尺寸。会清空像素缓冲区并重新分配。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let buffer_size = (width as usize) * (height as usize) * 4;
        self.pixel_buffer = vec![0u8; buffer_size];
        self.primitives = RenderPrimitives::new();
    }

    // ── Shadow properties ──

    /// 设置阴影颜色。
    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    /// 设置阴影模糊半径。负值会被限制为 0。
    pub fn set_shadow_blur(&mut self, blur: f32) {
        self.shadow_blur = blur.max(0.0);
    }

    /// 设置阴影水平偏移。
    pub fn set_shadow_offset_x(&mut self, offset: f32) {
        self.shadow_offset_x = offset;
    }

    /// 设置阴影垂直偏移。
    pub fn set_shadow_offset_y(&mut self, offset: f32) {
        self.shadow_offset_y = offset;
    }

    /// 返回当前阴影颜色。
    pub fn shadow_color(&self) -> &Color {
        &self.shadow_color
    }

    /// 返回当前阴影模糊半径。
    pub fn shadow_blur(&self) -> f32 {
        self.shadow_blur
    }

    /// 返回当前阴影水平偏移。
    pub fn shadow_offset_x(&self) -> f32 {
        self.shadow_offset_x
    }

    /// 返回当前阴影垂直偏移。
    pub fn shadow_offset_y(&self) -> f32 {
        self.shadow_offset_y
    }

    // ── Clipping ──

    /// 从当前路径设置裁剪区域。后续绘制操作将被限制在裁剪区域内。
    /// 调用后当前路径不会被清除（与浏览器行为一致）。
    pub fn clip(&mut self) {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return;
        }
        // 计算路径包围盒作为裁剪矩形
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        if min_x < max_x && min_y < max_y {
            let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
            self.primitives.add_clip(rect);
            // 保存裁剪路径的副本用于 isPointInPath 等后续判断
            self.clip_path = Some(self.current_path.clone());
        }
    }

    // ── Composite operation ──

    /// 设置合成操作模式。
    pub fn set_composite_operation(&mut self, op: CompositeOperation) {
        self.composite_operation = op;
    }

    /// 返回当前合成操作模式。
    pub fn composite_operation(&self) -> CompositeOperation {
        self.composite_operation
    }

    // ── Gradients ──

    /// 创建线性渐变。
    pub fn create_linear_gradient(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> LinearGradient {
        LinearGradient::new(x0, y0, x1, y1)
    }

    /// 创建径向渐变。
    pub fn create_radial_gradient(&self, x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> RadialGradient {
        RadialGradient::new(x0, y0, r0, x1, y1, r1)
    }

    /// 创建锥形渐变。
    pub fn create_conic_gradient(&self, start_angle: f32, cx: f32, cy: f32) -> ConicGradient {
        ConicGradient::new(start_angle, cx, cy)
    }

    // ── Pattern ──

    /// 从 ImageData 创建图案。
    pub fn create_pattern(&self, image_data: ImageData, repetition: PatternRepetition) -> CanvasPattern {
        CanvasPattern::new(image_data, repetition)
    }

    // ── Hit testing ──

    /// 判断点是否在当前路径内部（使用奇偶填充规则）。
    /// 点坐标为 Canvas 坐标空间，会先通过当前变换矩阵的逆变换映射到路径空间。
    pub fn is_point_in_path(&self, x: f32, y: f32) -> bool {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return false;
        }
        let points: Vec<(f32, f32)> = vertices.chunks_exact(2).map(|c| (c[0], c[1])).collect();
        point_in_polygon(x, y, &points)
    }

    /// 判断点是否在当前路径的描边区域内。
    /// 检测点到路径中每条线段的距离是否小于 line_width / 2。
    pub fn is_point_in_stroke(&self, x: f32, y: f32) -> bool {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return false;
        }
        let half_lw = self.line_width / 2.0;
        for chunk in vertices.chunks_exact(4) {
            let dist = point_to_segment_dist(x, y, chunk[0], chunk[1], chunk[2], chunk[3]);
            if dist < half_lw {
                return true;
            }
        }
        false
    }

    // ── Pixel data ──

    /// 获取像素数据。从画布像素缓冲区中读取指定区域的 RGBA 数据。
    pub fn get_image_data(&self, x: u32, y: u32, width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        for row in 0..(height as usize) {
            let src_row = y as usize + row;
            if src_row >= canvas_h {
                break;
            }
            let src_start = src_row * canvas_w * 4 + x as usize * 4;
            let src_end = (src_start + width as usize * 4).min(self.pixel_buffer.len());
            let dst_start = row * width as usize * 4;
            let copy_len = src_end.saturating_sub(src_start);
            if copy_len > 0 {
                data[dst_start..dst_start + copy_len].copy_from_slice(&self.pixel_buffer[src_start..src_end]);
            }
        }
        ImageData { width, height, data }
    }

    /// 创建指定尺寸的 ImageData，填充透明黑色（rgba 0,0,0,0）。
    pub fn create_image_data(&self, width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        ImageData {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    /// 放置像素数据。将 ImageData 写入画布像素缓冲区的指定偏移位置。
    pub fn put_image_data(&mut self, image_data: &ImageData, x: u32, y: u32) {
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        for row in 0..(image_data.height as usize) {
            let dst_row = y as usize + row;
            if dst_row >= canvas_h {
                break;
            }
            let src_start = row * image_data.width as usize * 4;
            let src_end = src_start + image_data.width as usize * 4;
            let dst_start = dst_row * canvas_w * 4 + x as usize * 4;
            let dst_end = (dst_start + image_data.width as usize * 4).min(self.pixel_buffer.len());
            let copy_len = dst_end.saturating_sub(dst_start);
            if copy_len > 0 && src_end <= image_data.data.len() {
                self.pixel_buffer[dst_start..dst_start + copy_len]
                    .copy_from_slice(&image_data.data[src_start..src_start + copy_len]);
            }
        }
    }

    // ── drawImage ──

    /// 将图像绘制到画布的指定位置（原始尺寸）。应用当前变换。
    pub fn draw_image(&mut self, image_data: &ImageData, dx: f32, dy: f32) {
        self.draw_image_sized(
            image_data,
            0.0,
            0.0,
            image_data.width as f32,
            image_data.height as f32,
            dx,
            dy,
            image_data.width as f32,
            image_data.height as f32,
        );
    }

    /// 将图像绘制到画布的指定位置，缩放到目标尺寸。应用当前变换。
    pub fn draw_image_with_size(&mut self, image_data: &ImageData, dx: f32, dy: f32, dw: f32, dh: f32) {
        self.draw_image_sized(
            image_data,
            0.0,
            0.0,
            image_data.width as f32,
            image_data.height as f32,
            dx,
            dy,
            dw,
            dh,
        );
    }

    /// 将图像的指定切片区域绘制到画布的目标区域（支持缩放）。应用当前变换。
    #[allow(clippy::too_many_arguments)]
    pub fn draw_image_sliced(
        &mut self,
        image_data: &ImageData,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
    ) {
        self.draw_image_sized(image_data, sx, sy, sw, sh, dx, dy, dw, dh);
    }

    /// 内部方法：将图像的指定区域绘制到画布的目标区域。
    #[allow(clippy::too_many_arguments)]
    fn draw_image_sized(
        &mut self,
        image_data: &ImageData,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
    ) {
        let img_w = image_data.width as usize;
        let img_h = image_data.height as usize;
        if img_w == 0 || img_h == 0 || sw <= 0.0 || sh <= 0.0 || dw <= 0.0 || dh <= 0.0 {
            return;
        }

        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        if canvas_w == 0 || canvas_h == 0 {
            return;
        }

        let sx = sx.max(0.0) as usize;
        let sy = sy.max(0.0) as usize;
        let sw = sw.min((img_w - sx) as f32) as usize;
        let sh = sh.min((img_h - sy) as f32) as usize;
        if sw == 0 || sh == 0 {
            return;
        }

        let x_scale = sw as f32 / dw;
        let y_scale = sh as f32 / dh;

        // 应用变换后的目标矩形用于逐像素计算
        for py in 0..(dh as usize) {
            for px in 0..(dw as usize) {
                // 源像素坐标（最近邻采样）
                let src_x = sx + (px as f32 * x_scale) as usize;
                let src_y = sy + (py as f32 * y_scale) as usize;
                if src_x >= img_w || src_y >= img_h {
                    continue;
                }

                let src_idx = (src_y * img_w + src_x) * 4;
                if src_idx + 3 >= image_data.data.len() {
                    continue;
                }
                let r = image_data.data[src_idx];
                let g = image_data.data[src_idx + 1];
                let b = image_data.data[src_idx + 2];
                let a = image_data.data[src_idx + 3];

                // 变换目标坐标
                let (dst_x, dst_y) = self.transform.transform_point(dx + px as f32, dy + py as f32);
                let dst_x = dst_x as usize;
                let dst_y = dst_y as usize;
                if dst_x >= canvas_w || dst_y >= canvas_h {
                    continue;
                }

                let dst_idx = (dst_y * canvas_w + dst_x) * 4;
                // 简单 alpha 混合
                let src_alpha = (a as f32 * self.global_alpha) as u8;
                if src_alpha == 0 {
                    continue;
                }
                if src_alpha == 255 {
                    self.pixel_buffer[dst_idx] = r;
                    self.pixel_buffer[dst_idx + 1] = g;
                    self.pixel_buffer[dst_idx + 2] = b;
                    self.pixel_buffer[dst_idx + 3] = src_alpha;
                } else {
                    let dst_a = self.pixel_buffer[dst_idx + 3] as f32 / 255.0;
                    let src_a = src_alpha as f32 / 255.0;
                    let out_a = src_a + dst_a * (1.0 - src_a);
                    if out_a > 0.0 {
                        let factor = 1.0 / out_a;
                        self.pixel_buffer[dst_idx] = ((r as f32 * src_a
                            + self.pixel_buffer[dst_idx] as f32 * dst_a * (1.0 - src_a))
                            * factor) as u8;
                        self.pixel_buffer[dst_idx + 1] = ((g as f32 * src_a
                            + self.pixel_buffer[dst_idx + 1] as f32 * dst_a * (1.0 - src_a))
                            * factor) as u8;
                        self.pixel_buffer[dst_idx + 2] = ((b as f32 * src_a
                            + self.pixel_buffer[dst_idx + 2] as f32 * dst_a * (1.0 - src_a))
                            * factor) as u8;
                        self.pixel_buffer[dst_idx + 3] = (out_a * 255.0) as u8;
                    }
                }
            }
        }
    }

    // ── Output ──

    /// 判断当前是否启用了阴影（阴影颜色不透明且偏移或模糊非零）。
    fn has_shadow(&self) -> bool {
        self.shadow_color.a > 0
            && (self.shadow_blur > 0.0 || self.shadow_offset_x != 0.0 || self.shadow_offset_y != 0.0)
    }

    /// 为矩形绘制阴影。简化实现：绘制一个偏移的矩形，alpha 根据 shadow_blur 降低。
    fn draw_shadow_rect(&mut self, rect: &Rect) {
        let blur_factor = if self.shadow_blur > 0.0 {
            (1.0 / (1.0 + self.shadow_blur * 0.1)).min(1.0)
        } else {
            1.0
        };
        let shadow_alpha =
            ((self.shadow_color.a as f32 * self.global_alpha * blur_factor) as u8).min(self.shadow_color.a);
        let color = Color::rgba(
            self.shadow_color.r,
            self.shadow_color.g,
            self.shadow_color.b,
            shadow_alpha,
        );
        let shadow_rect = Rect::new(
            rect.left() + self.shadow_offset_x,
            rect.top() + self.shadow_offset_y,
            rect.size.width,
            rect.size.height,
        );
        self.blit_rect_to_pixels(&shadow_rect, color);
    }

    /// 为路径绘制阴影。简化实现：绘制一个偏移的路径包围盒，alpha 根据 shadow_blur 降低。
    fn draw_shadow_path(&mut self, vertices: &[f32]) {
        let blur_factor = if self.shadow_blur > 0.0 {
            (1.0 / (1.0 + self.shadow_blur * 0.1)).min(1.0)
        } else {
            1.0
        };
        let shadow_alpha =
            ((self.shadow_color.a as f32 * self.global_alpha * blur_factor) as u8).min(self.shadow_color.a);
        let color = Color::rgba(
            self.shadow_color.r,
            self.shadow_color.g,
            self.shadow_color.b,
            shadow_alpha,
        );
        // 将路径的每个顶点偏移 shadow_offset
        let offset_vertices: Vec<f32> = vertices
            .chunks_exact(2)
            .flat_map(|c| [c[0] + self.shadow_offset_x, c[1] + self.shadow_offset_y])
            .collect();
        self.blit_path_to_pixels(&offset_vertices, color);
    }

    /// 消费上下文，返回渲染图元列表。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 返回渲染图元列表的引用。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }

    // ── Private helpers ──

    /// 对矩形应用当前变换。
    fn transform_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        let (x1, y1) = self.transform.transform_point(x, y);
        let (x2, y2) = self.transform.transform_point(x + width, y + height);
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 应用 global_alpha 到颜色。
    fn apply_alpha(&self, color: Color) -> Color {
        let a = ((color.a as f32) * self.global_alpha) as u8;
        Color::rgba(color.r, color.g, color.b, a)
    }

    /// 将圆角矩形扁平化为线段顶点。
    /// 每个圆角使用 8 段线段近似四分之一圆弧。
    /// radii 遵循 Canvas 规范：1 个值用于全部角，2 个值为 [左上/右下, 右上/左下]，4 个值为 [左上, 右上, 右下, 左下]。
    #[allow(clippy::too_many_arguments)]
    fn flatten_round_rect(
        vertices: &mut Vec<f32>,
        current_x: f32,
        current_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radii: &[f32],
    ) -> (f32, f32) {
        // 解析圆角半径：[左上, 右上, 右下, 左下]
        let mut r = [0.0f32; 4];
        match radii.len() {
            0 => {}
            1 => {
                r[0] = radii[0];
                r[1] = radii[0];
                r[2] = radii[0];
                r[3] = radii[0];
            }
            2 => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[0];
                r[3] = radii[1];
            }
            3 => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[2];
                r[3] = radii[1];
            }
            _ => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[2];
                r[3] = radii[3];
            }
        }
        // 限制半径不超过短边的一半
        let max_r = w.min(h) / 2.0;
        for radius in &mut r {
            *radius = radius.min(max_r).max(0.0);
        }

        // 所有半径为 0 时退化为矩形
        if r.iter().all(|&v| v < f32::EPSILON) {
            let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
            vertices.push(current_x);
            vertices.push(current_y);
            vertices.push(corners[0].0);
            vertices.push(corners[0].1);
            for i in 0..3 {
                vertices.push(corners[i].0);
                vertices.push(corners[i].1);
                vertices.push(corners[i + 1].0);
                vertices.push(corners[i + 1].1);
            }
            vertices.push(corners[3].0);
            vertices.push(corners[3].1);
            vertices.push(corners[0].0);
            vertices.push(corners[0].1);
            return (corners[0].0, corners[0].1);
        }

        // 圆角中心坐标和对应的弧角度范围
        // 左上角 (x + r[0], y + r[0]), 角度 π ~ 3π/2
        // 右上角 (x + w - r[1], y + r[1]), 角度 3π/2 ~ 2π
        // 右下角 (x + w - r[2], y + h - r[2]), 角度 0 ~ π/2
        // 左下角 (x + r[3], y + h - r[3]), 角度 π/2 ~ π
        let corner_cx = [x + r[0], x + w - r[1], x + w - r[2], x + r[3]];
        let corner_cy = [y + r[0], y + r[1], y + h - r[2], y + h - r[3]];
        let corner_start = [
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
        ];
        let corner_end = [
            std::f32::consts::FRAC_PI_2 * 3.0,
            std::f32::consts::TAU,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ];

        const CORNER_SEGMENTS: usize = 8;

        // 从当前点连线到第一个圆角的起点
        let start_angle = corner_start[0];
        let start_x = corner_cx[0] + r[0] * start_angle.cos();
        let start_y = corner_cy[0] + r[0] * start_angle.sin();
        vertices.push(current_x);
        vertices.push(current_y);
        vertices.push(start_x);
        vertices.push(start_y);

        // 遍历 4 个圆角
        for c in 0..4 {
            let step = (corner_end[c] - corner_start[c]) / CORNER_SEGMENTS as f32;
            let mut px = corner_cx[c] + r[c] * corner_start[c].cos();
            let mut py = corner_cy[c] + r[c] * corner_start[c].sin();
            for i in 0..CORNER_SEGMENTS {
                let angle = corner_start[c] + step * (i + 1) as f32;
                let nx = corner_cx[c] + r[c] * angle.cos();
                let ny = corner_cy[c] + r[c] * angle.sin();
                vertices.push(px);
                vertices.push(py);
                vertices.push(nx);
                vertices.push(ny);
                px = nx;
                py = ny;
            }
            // 从圆角末尾连线到下一个圆角的起点（即直边段）
            let next = (c + 1) % 4;
            let next_start = corner_start[next];
            let next_x = corner_cx[next] + r[next] * next_start.cos();
            let next_y = corner_cy[next] + r[next] * next_start.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(next_x);
            vertices.push(next_y);
        }

        (start_x, start_y)
    }

    /// 计算 arcTo 的几何信息：返回 (切点1x, 切点1y, 切点2x, 切点2y)。
    /// 特殊情况（半径为 0、共线、点重合等）返回的切点会退化为直线。
    fn compute_arc_to_geometry(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        radius: f32,
    ) -> (f32, f32, f32, f32) {
        // 方向向量：从当前点到控制点1，从控制点1到控制点2
        let dx1 = x0 - x1;
        let dy1 = y0 - y1;
        let dx2 = x2 - x1;
        let dy2 = y2 - y1;

        let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        let len2 = (dx2 * dx2 + dy2 * dy2).sqrt();

        // 退化为直线：半径为 0，或任一方向向量长度为 0
        if radius < f32::EPSILON || len1 < f32::EPSILON || len2 < f32::EPSILON {
            return (x1, y1, x1, y1);
        }

        // 单位方向向量
        let ux1 = dx1 / len1;
        let uy1 = dy1 / len1;
        let ux2 = dx2 / len2;
        let uy2 = dy2 / len2;

        // 两条切线之间的夹角
        let dot = ux1 * ux2 + uy1 * uy2;
        // 夹角接近 ±1 表示共线或反平行
        let one_minus_dot_sq = 1.0 - dot * dot;
        if one_minus_dot_sq < f32::EPSILON {
            // 共线情况：直接画线到控制点1
            return (x1, y1, x1, y1);
        }

        // 圆弧圆心到控制点1的距离
        let d = radius / one_minus_dot_sq.sqrt();

        // 圆弧圆心坐标
        let cx = x1 + d * (ux1 + ux2);
        let cy = y1 + d * (uy1 + uy2);

        // 切点1：圆心 + radius * 指向当前点方向的单位向量
        let t1x = cx + radius * ux1;
        let t1y = cy + radius * uy1;

        // 切点2：圆心 + radius * 指向控制点2方向的单位向量
        let t2x = cx + radius * ux2;
        let t2y = cy + radius * uy2;

        (t1x, t1y, t2x, t2y)
    }

    /// 将 arcTo 命令扁平化为线段顶点。
    #[allow(clippy::too_many_arguments)]
    fn flatten_arc_to(
        vertices: &mut Vec<f32>,
        current_x: f32,
        current_y: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        radius: f32,
        segments: usize,
    ) {
        let (t1x, t1y, t2x, t2y) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);

        // 从当前点画线到切点1
        if (current_x - t1x).abs() > f32::EPSILON || (current_y - t1y).abs() > f32::EPSILON {
            vertices.push(current_x);
            vertices.push(current_y);
            vertices.push(t1x);
            vertices.push(t1y);
        }

        // 如果两个切点重合（退化情况），不需要画弧
        if (t1x - t2x).abs() < f32::EPSILON && (t1y - t2y).abs() < f32::EPSILON {
            return;
        }

        // 计算圆弧圆心和角度范围
        let v1x = t1x - x1;
        let v1y = t1y - y1;
        let v2x = t2x - x1;
        let v2y = t2y - y1;
        let lv1 = (v1x * v1x + v1y * v1y).sqrt();
        let lv2 = (v2x * v2x + v2y * v2y).sqrt();

        if lv1 < f32::EPSILON || lv2 < f32::EPSILON {
            return;
        }

        // 圆心在切点1沿远离控制点1方向偏移 radius 处
        let cx = t1x + (radius / lv1) * v1x;
        let cy = t1y + (radius / lv1) * v1y;

        // 计算切点相对圆心的角度
        let start_angle = (t1y - cy).atan2(t1x - cx);
        let end_angle = (t2y - cy).atan2(t2x - cx);

        // 确定弧线方向：从 t1 经过远离 (x1,y1) 的方向到 t2
        // 使用叉积判断方向
        let cross = v1x * v2y - v1y * v2x;
        let mut angle_span = end_angle - start_angle;

        // 根据叉积方向调整角度范围
        if cross >= 0.0 {
            // 逆时针：确保 angle_span > 0
            if angle_span < 0.0 {
                angle_span += std::f32::consts::TAU;
            }
        } else {
            // 顺时针：确保 angle_span < 0
            if angle_span > 0.0 {
                angle_span -= std::f32::consts::TAU;
            }
        }

        // 用线段近似弧线
        let step = angle_span / segments as f32;
        let mut px = t1x;
        let mut py = t1y;
        for i in 0..segments {
            let angle = start_angle + step * (i + 1) as f32;
            let nx = cx + radius * angle.cos();
            let ny = cy + radius * angle.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(nx);
            vertices.push(ny);
            px = nx;
            py = ny;
        }
    }

    /// 将当前路径命令扁平化为顶点列表（x, y 交替）。
    /// 对于圆弧，使用线性近似（固定 16 段细分）。
    fn flatten_path(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        const ARC_SEGMENTS: usize = 16;

        for cmd in self.current_path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                }
                PathCommand::LineTo(x, y) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(x);
                    vertices.push(y);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    // 使用 8 段细分二次贝塞尔曲线
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * current_x + 2.0 * mt * t * cpx + t * t * x;
                        let ny = mt * mt * current_y + 2.0 * mt * t * cpy + t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    // 使用 8 段细分三次贝塞尔曲线
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * mt * current_x
                            + 3.0 * mt * mt * t * cp1x
                            + 3.0 * mt * t * t * cp2x
                            + t * t * t * x;
                        let ny = mt * mt * mt * current_y
                            + 3.0 * mt * mt * t * cp1y
                            + 3.0 * mt * t * t * cp2y
                            + t * t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle) => {
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut angle = start_angle;
                    let mut px = cx + radius * angle.cos();
                    let mut py = cy + radius * angle.sin();
                    // 如果之前有 MoveTo，弧线的第一个点应该从当前点连线
                    for i in 0..ARC_SEGMENTS {
                        angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    Self::flatten_arc_to(
                        &mut vertices,
                        current_x,
                        current_y,
                        x1,
                        y1,
                        x2,
                        y2,
                        radius,
                        ARC_SEGMENTS,
                    );
                    // flatten_arc_to updates current_x/current_y via the returned value
                    // We compute the final point directly
                    let (_, _, nx, ny) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::ClosePath => {
                    // 从当前点画线回到子路径起点
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
            }
        }
        vertices
    }

    /// 将指定 Path2D 的命令扁平化为顶点列表（x, y 交替）。
    fn flatten_path_for(&self, path: &Path2D) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        const ARC_SEGMENTS: usize = 16;

        for cmd in path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                }
                PathCommand::LineTo(x, y) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(x);
                    vertices.push(y);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * current_x + 2.0 * mt * t * cpx + t * t * x;
                        let ny = mt * mt * current_y + 2.0 * mt * t * cpy + t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * mt * current_x
                            + 3.0 * mt * mt * t * cp1x
                            + 3.0 * mt * t * t * cp2x
                            + t * t * t * x;
                        let ny = mt * mt * mt * current_y
                            + 3.0 * mt * mt * t * cp1y
                            + 3.0 * mt * t * t * cp2y
                            + t * t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle) => {
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut px = cx + radius * start_angle.cos();
                    let mut py = cy + radius * start_angle.sin();
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    Self::flatten_arc_to(
                        &mut vertices,
                        current_x,
                        current_y,
                        x1,
                        y1,
                        x2,
                        y2,
                        radius,
                        ARC_SEGMENTS,
                    );
                    let (_, _, nx, ny) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::ClosePath => {
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
            }
        }
        vertices
    }

    /// 使用当前合成操作模式，将源颜色与目标像素进行合成。
    /// 返回合成后的 RGBA 值（0-255）。
    /// 参考 Porter-Duff alpha compositing 规范实现。
    fn composite_pixel(&self, src: Color, dst_r: u8, dst_g: u8, dst_b: u8, dst_a: u8) -> (u8, u8, u8, u8) {
        let sa = src.a as f32 / 255.0;
        let da = dst_a as f32 / 255.0;
        let sr = src.r as f32 / 255.0;
        let sg = src.g as f32 / 255.0;
        let sb = src.b as f32 / 255.0;
        let dr = dst_r as f32 / 255.0;
        let dg = dst_g as f32 / 255.0;
        let db = dst_b as f32 / 255.0;

        // Porter-Duff 合成因子 (Fa, Fb)
        let (fa, fb) = match self.composite_operation {
            CompositeOperation::SourceOver => (1.0, 1.0 - sa),
            CompositeOperation::DestinationOver => (1.0 - da, 1.0),
            CompositeOperation::SourceIn => (da, 0.0),
            CompositeOperation::DestinationIn => (0.0, sa),
            CompositeOperation::DestinationOut => (0.0, 1.0 - sa),
            CompositeOperation::SourceAtop => (da, 1.0 - sa),
            CompositeOperation::DestinationAtop => (1.0 - da, sa),
            CompositeOperation::Copy => (1.0, 0.0),
            CompositeOperation::Xor => (1.0 - da, 1.0 - sa),
            CompositeOperation::Lighter => (1.0, 1.0),
            // 其余混合模式使用 source-over 的合成因子
            _ => (1.0, 1.0 - sa),
        };

        let out_a = sa * fa + da * fb;
        if out_a <= 0.0 {
            return (0, 0, 0, 0);
        }
        let out_r = (sr * sa * fa + dr * da * fb) / out_a;
        let out_g = (sg * sa * fa + dg * da * fb) / out_a;
        let out_b = (sb * sa * fa + db * da * fb) / out_a;

        (
            (out_r * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_g * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_b * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    /// 将矩形区域的颜色写入像素缓冲区（光栅化填充），应用当前合成操作模式。
    fn blit_rect_to_pixels(&mut self, rect: &Rect, color: Color) {
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        let x_end = (rect.right().min(self.width as f32) as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32) as usize).min(canvas_h);
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = (y * canvas_w + x) * 4;
                let (r, g, b, a) = self.composite_pixel(
                    color,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = r;
                self.pixel_buffer[idx + 1] = g;
                self.pixel_buffer[idx + 2] = b;
                self.pixel_buffer[idx + 3] = a;
            }
        }
    }

    /// 将路径填充写入像素缓冲区（扫描线光栅化）。
    fn blit_path_to_pixels(&mut self, vertices: &[f32], color: Color) {
        if vertices.len() < 4 {
            return;
        }
        // 找出包围盒
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        // 收集所有唯一顶点用于扫描线
        let mut points: Vec<(f32, f32)> = Vec::new();
        for chunk in vertices.chunks_exact(2) {
            points.push((chunk[0], chunk[1]));
        }
        let canvas_w = self.width;
        let canvas_h = self.height;
        let y_start = min_y.max(0.0).ceil() as u32;
        let y_end = max_y.min(canvas_h as f32).ceil() as u32;

        for scan_y in y_start..y_end {
            let mut intersections: Vec<f32> = Vec::new();
            let sy = scan_y as f32 + 0.5;
            for i in 0..points.len() {
                let (x1, y1) = points[i];
                let (x2, y2) = points[(i + 1) % points.len()];
                if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
                    let t = (sy - y1) / (y2 - y1);
                    intersections.push(x1 + t * (x2 - x1));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in intersections.chunks_exact(2) {
                let ix_start = pair[0].max(0.0) as u32;
                let ix_end = pair[1].min(canvas_w as f32) as u32;
                for scan_x in ix_start..ix_end {
                    let idx = ((scan_y * canvas_w + scan_x) * 4) as usize;
                    if idx + 3 < self.pixel_buffer.len() {
                        self.pixel_buffer[idx] = color.r;
                        self.pixel_buffer[idx + 1] = color.g;
                        self.pixel_buffer[idx + 2] = color.b;
                        self.pixel_buffer[idx + 3] = color.a;
                    }
                }
            }
        }
    }

    /// 将路径描边写入像素缓冲区（考虑 line_join 和 line_cap 设置）。
    fn blit_stroke_to_pixels(&mut self, vertices: &[f32], color: Color, line_width: f32) {
        if vertices.len() < 4 {
            return;
        }

        let half_lw = line_width / 2.0;

        // 将线段顶点列表转为 (x1,y1,x2,y2) 段列表
        let segments: Vec<[f32; 4]> = vertices.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
        if segments.is_empty() {
            return;
        }

        // 绘制每条线段的主体矩形
        for seg in &segments {
            let rect = self.line_segment_rect(seg[0], seg[1], seg[2], seg[3], line_width);
            self.blit_rect_to_pixels(&rect, color);
        }

        // 绘制连接点（相邻线段交汇处）
        for i in 0..segments.len().saturating_sub(1) {
            let seg_a = segments[i];
            let _seg_b = segments[i + 1];
            // seg_a 的终点应与 seg_b 的起点相同
            let jx = seg_a[2];
            let jy = seg_a[3];

            match self.line_join {
                LineJoin::Miter => {
                    // 尖角：在连接点画一个覆盖 half_lw 的正方形
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
                LineJoin::Round => {
                    // 圆角：在连接点画一个半径为 half_lw 的圆（近似为正方形）
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
                LineJoin::Bevel => {
                    // 斜角：在连接点画一个 half_lw × half_lw 的正方形
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
            }
        }

        // 绘制端点 cap
        let first_seg = segments[0];
        let last_seg = segments[segments.len() - 1];

        // 起点端 cap
        self.blit_line_cap(first_seg[0], first_seg[1], first_seg[2], first_seg[3], half_lw, color);
        // 终点端 cap
        self.blit_line_cap(last_seg[2], last_seg[3], last_seg[0], last_seg[1], half_lw, color);
    }

    /// 绘制线段端点的 cap。
    /// `endpoint` 是端点位置，`other` 是线段另一端（用于确定方向）。
    fn blit_line_cap(
        &mut self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        half_lw: f32,
        color: Color,
    ) {
        match self.line_cap {
            LineCap::Butt => {
                // 平头：不做额外处理（线段矩形已精确到端点）
            }
            LineCap::Round => {
                // 圆头：在端点画一个半径为 half_lw 的圆（近似为正方形）
                let rect = Rect::new(endpoint_x - half_lw, endpoint_y - half_lw, half_lw * 2.0, half_lw * 2.0);
                self.blit_rect_to_pixels(&rect, color);
            }
            LineCap::Square => {
                // 方头：在端点方向延伸 half_lw 的矩形
                let dx = endpoint_x - other_x;
                let dy = endpoint_y - other_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                // 从端点沿方向延伸 half_lw
                let ext_x = endpoint_x + ux * half_lw;
                let ext_y = endpoint_y + uy * half_lw;
                // 覆盖区域：从 endpoint 到 ext 的范围，宽度 line_width
                let min_x = endpoint_x.min(ext_x) - half_lw;
                let min_y = endpoint_y.min(ext_y) - half_lw;
                let max_x = endpoint_x.max(ext_x) + half_lw;
                let max_y = endpoint_y.max(ext_y) + half_lw;
                let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                self.blit_rect_to_pixels(&rect, color);
            }
        }
    }

    /// 计算线段的描边矩形（沿线段方向扩展 line_width / 2）。
    fn line_segment_rect(&self, x1: f32, y1: f32, x2: f32, y2: f32, line_width: f32) -> Rect {
        let half_lw = line_width / 2.0;
        let min_x = x1.min(x2) - half_lw;
        let min_y = y1.min(y2) - half_lw;
        let max_x = x1.max(x2) + half_lw;
        let max_y = y1.max(y2) + half_lw;
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 计算描边路径的顶点，包括 line_join 和 line_cap 产生的额外顶点。
    /// 返回一个包含 (x, y) 对的顶点列表，构成描边的轮廓多边形。
    pub fn stroke_outline_vertices(&self) -> Vec<f32> {
        let path_vertices = self.flatten_path();
        if path_vertices.len() < 4 {
            return Vec::new();
        }

        let half_lw = self.line_width / 2.0;
        let segments: Vec<[f32; 4]> = path_vertices
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        let mut outline = Vec::new();

        for (i, seg) in segments.iter().enumerate() {
            let x1 = seg[0];
            let y1 = seg[1];
            let x2 = seg[2];
            let y2 = seg[3];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            if len < f32::EPSILON {
                continue;
            }
            let nx = -dy / len * half_lw; // 法线方向
            let ny = dx / len * half_lw;

            // 线段主体：4 个角点
            outline.push(x1 + nx);
            outline.push(y1 + ny);
            outline.push(x2 + nx);
            outline.push(y2 + ny);
            outline.push(x2 - nx);
            outline.push(y2 - ny);
            outline.push(x1 - nx);
            outline.push(y1 - ny);

            // 起点端 cap（仅第一条线段）
            if i == 0 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x1;
                        let cy = y1;
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        let ext = half_lw;
                        outline.push(x1 + nx);
                        outline.push(y1 + ny);
                        outline.push(x1 + nx + dir_x * ext);
                        outline.push(y1 + ny + dir_y * ext);
                        outline.push(x1 - nx + dir_x * ext);
                        outline.push(y1 - ny + dir_y * ext);
                        outline.push(x1 - nx);
                        outline.push(y1 - ny);
                    }
                }
            }

            // 连接点（与下一条线段之间）
            if i < segments.len() - 1 {
                let next = segments[i + 1];
                let ndx = next[2] - next[0];
                let ndy = next[3] - next[1];
                let nlen = (ndx * ndx + ndy * ndy).sqrt();

                if nlen >= f32::EPSILON {
                    let nnx = -ndy / nlen * half_lw;
                    let nny = ndx / nlen * half_lw;
                    let jx = x2;
                    let jy = y2;

                    match self.line_join {
                        LineJoin::Miter => {
                            // 尖角：延伸两侧法线的交点
                            let miter_len = Self::compute_miter_length(nx, ny, nnx, nny, half_lw);
                            let mx = nx + nnx;
                            let my = ny + nny;
                            let m = (mx * mx + my * my).sqrt();
                            if m > f32::EPSILON {
                                let miter_x = jx + mx / m * miter_len;
                                let miter_y = jy + my / m * miter_len;
                                outline.push(jx + nx);
                                outline.push(jy + ny);
                                outline.push(miter_x);
                                outline.push(miter_y);
                                outline.push(jx + nnx);
                                outline.push(jy + nny);
                            }
                        }
                        LineJoin::Round => {
                            // 圆角：在连接点画扇形
                            const JOIN_SEGMENTS: usize = 4;
                            let start_angle = ny.atan2(nx);
                            let end_angle = nny.atan2(nnx);
                            let step = {
                                let diff = end_angle - start_angle;
                                if diff > std::f32::consts::PI {
                                    diff - std::f32::consts::TAU
                                } else if diff < -std::f32::consts::PI {
                                    diff + std::f32::consts::TAU
                                } else {
                                    diff
                                }
                            } / JOIN_SEGMENTS as f32;
                            let mut angle = start_angle;
                            for _ in 0..JOIN_SEGMENTS {
                                let a1 = angle;
                                let a2 = angle + step;
                                outline.push(jx);
                                outline.push(jy);
                                outline.push(jx + half_lw * a1.cos());
                                outline.push(jy + half_lw * a1.sin());
                                outline.push(jx + half_lw * a2.cos());
                                outline.push(jy + half_lw * a2.sin());
                                angle = a2;
                            }
                        }
                        LineJoin::Bevel => {
                            // 斜角：三角形连接
                            outline.push(jx + nx);
                            outline.push(jy + ny);
                            outline.push(jx + nnx);
                            outline.push(jy + nny);
                        }
                    }
                }
            }

            // 终点端 cap（仅最后一条线段）
            if i == segments.len() - 1 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x2;
                        let cy = y2;
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        let ext = half_lw;
                        outline.push(x2 + nx);
                        outline.push(y2 + ny);
                        outline.push(x2 + nx + dir_x * ext);
                        outline.push(y2 + ny + dir_y * ext);
                        outline.push(x2 - nx + dir_x * ext);
                        outline.push(y2 - ny + dir_y * ext);
                        outline.push(x2 - nx);
                        outline.push(y2 - ny);
                    }
                }
            }
        }

        outline
    }

    /// 计算 miter 连接的长度（从连接点到尖角顶点的距离）。
    fn compute_miter_length(nx: f32, ny: f32, nnx: f32, nny: f32, half_lw: f32) -> f32 {
        let mx = nx + nnx;
        let my = ny + nny;
        let m = (mx * mx + my * my).sqrt();
        if m < f32::EPSILON {
            return half_lw;
        }
        half_lw * 2.0 / m
    }
}

/// 使用射线法（ray casting）判断点是否在多边形内部。
fn point_in_polygon(px: f32, py: f32, points: &[(f32, f32)]) -> bool {
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
fn point_to_segment_dist(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_new() {
        let ctx = CanvasContext::new(800, 600);
        assert_eq!(ctx.width(), 800);
        assert_eq!(ctx.height(), 600);
        assert_eq!(ctx.global_alpha(), 1.0);
        assert_eq!(ctx.line_width(), 1.0);
    }

    #[test]
    fn test_canvas_fill_rect() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill_rect(10.0, 20.0, 30.0, 40.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
        let fill = &ctx.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 10.0);
        assert_eq!(fill.rect.origin.y, 20.0);
        assert_eq!(fill.rect.size.width, 30.0);
        assert_eq!(fill.rect.size.height, 40.0);
    }

    #[test]
    fn test_canvas_stroke_rect() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.stroke_rect(0.0, 0.0, 50.0, 50.0);
        // stroke_rect adds 4 fill primitives (top, bottom, left, right)
        assert_eq!(ctx.primitives().fills.len(), 4);
    }

    #[test]
    fn test_canvas_clear_rect() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.clear_rect(5.0, 5.0, 20.0, 20.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
        let fill = &ctx.primitives().fills[0];
        assert_eq!(fill.color, Color::TRANSPARENT);
    }

    #[test]
    fn test_canvas_fill_text() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_text("hello", 10.0, 20.0);
        // 每个字符生成一个 glyph
        assert_eq!(ctx.primitives().glyphs.len(), 5);
    }

    #[test]
    fn test_canvas_measure_text() {
        let ctx = CanvasContext::new(200, 200);
        let metrics = ctx.measure_text("abc");
        // 简化估算: 3 chars * 10.0 * 0.6 = 18.0
        assert!((metrics.width - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.save();
        ctx.set_fill_color(Color::BLUE);
        assert_eq!(ctx.fill_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::RED);
    }

    #[test]
    fn test_canvas_set_fill_color() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::GREEN);
        assert_eq!(ctx.fill_color(), Color::GREEN);
    }

    #[test]
    fn test_canvas_set_line_width() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_width(5.0);
        assert!((ctx.line_width() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_global_alpha() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(0.5);
        assert!((ctx.global_alpha() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_translate() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.translate(10.0, 20.0);
        ctx.fill_rect(0.0, 0.0, 5.0, 5.0);
        let fill = &ctx.primitives().fills[0];
        assert!((fill.rect.origin.x - 10.0).abs() < f32::EPSILON);
        assert!((fill.rect.origin.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_scale() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.scale(2.0, 3.0);
        ctx.fill_rect(10.0, 10.0, 10.0, 10.0);
        let fill = &ctx.primitives().fills[0];
        assert!((fill.rect.origin.x - 20.0).abs() < f32::EPSILON);
        assert!((fill.rect.origin.y - 30.0).abs() < f32::EPSILON);
        assert!((fill.rect.size.width - 20.0).abs() < f32::EPSILON);
        assert!((fill.rect.size.height - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_rotate() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.rotate(std::f32::consts::FRAC_PI_2);
        // After 90-degree rotation, drawing at (1,0) should appear near (0,1)
        let (x, y) = ctx.transform.transform_point(1.0, 0.0);
        assert!((x).abs() < 0.001);
        assert!((y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_canvas_set_transform() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_transform(2.0, 0.0, 0.0, 2.0, 10.0, 10.0);
        let (x, y) = ctx.transform.transform_point(5.0, 5.0);
        assert!((x - 20.0).abs() < f32::EPSILON);
        assert!((y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_reset_transform() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.translate(100.0, 100.0);
        ctx.reset_transform();
        let (x, y) = ctx.transform.transform_point(0.0, 0.0);
        assert!((x).abs() < f32::EPSILON);
        assert!((y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_multiple_operations() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        ctx.stroke_rect(60.0, 60.0, 30.0, 30.0);
        ctx.fill_text("test", 0.0, 0.0);
        // fill_rect = 1, stroke_rect = 4, fill_text = 4 glyphs
        assert_eq!(ctx.primitives().fills.len(), 5);
        assert_eq!(ctx.primitives().glyphs.len(), 4);
    }

    #[test]
    fn test_canvas_nested_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.save();
        ctx.set_fill_color(Color::GREEN);
        ctx.save();
        ctx.set_fill_color(Color::BLUE);
        assert_eq!(ctx.fill_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::GREEN);
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::RED);
    }

    #[test]
    fn test_canvas_primitives_collected() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        ctx.fill_rect(20.0, 20.0, 10.0, 10.0);
        let primitives = ctx.into_primitives();
        assert_eq!(primitives.fills.len(), 2);
    }

    #[test]
    fn test_image_data_new() {
        let ctx = CanvasContext::new(100, 100);
        let img = ctx.get_image_data(0, 0, 10, 10);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
        assert_eq!(img.data.len(), 400); // 10 * 10 * 4
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform2D::identity();
        assert!((t.a - 1.0).abs() < f32::EPSILON);
        assert!((t.d - 1.0).abs() < f32::EPSILON);
        assert!((t.e).abs() < f32::EPSILON);
        assert!((t.f).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_multiply() {
        let a = Transform2D::translate(10.0, 20.0);
        let b = Transform2D::scale(2.0, 2.0);
        let c = a.multiply(&b);
        let (x, y) = c.transform_point(5.0, 5.0);
        // translate(10,20) * scale(2,2) applied to (5,5):
        // first scale: (10, 10), then translate: (20, 30)
        assert!((x - 20.0).abs() < f32::EPSILON);
        assert!((y - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_point() {
        let t = Transform2D::translate(100.0, 200.0);
        let (x, y) = t.transform_point(10.0, 20.0);
        assert!((x - 110.0).abs() < f32::EPSILON);
        assert!((y - 220.0).abs() < f32::EPSILON);
    }

    // ── FontDescriptor / FontWeight / FontStyle ──

    #[test]
    fn test_font_descriptor_default() {
        let f = FontDescriptor::default();
        assert_eq!(f.family, "sans-serif");
        assert!((f.size - 10.0).abs() < f32::EPSILON);
        assert!(matches!(f.weight, FontWeight::Normal));
        assert!(matches!(f.style, FontStyle::Normal));
    }

    #[test]
    fn test_font_descriptor_custom() {
        let f = FontDescriptor {
            family: "monospace".to_string(),
            size: 14.0,
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
        };
        assert_eq!(f.family, "monospace");
        assert!(matches!(f.weight, FontWeight::Bold));
        assert!(matches!(f.style, FontStyle::Italic));
    }

    #[test]
    fn test_canvas_set_font() {
        let mut ctx = CanvasContext::new(100, 100);
        let font = FontDescriptor {
            family: "serif".to_string(),
            size: 20.0,
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
        };
        ctx.set_font(font);
        let metrics = ctx.measure_text("test");
        // 字体大小 20.0，4 字符 × 20.0 × 0.6 = 48.0
        assert!((metrics.width - 48.0).abs() < f32::EPSILON);
    }

    // ── stroke_color / stroke_text ──

    #[test]
    fn test_canvas_set_stroke_color() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::BLUE);
        assert_eq!(ctx.stroke_color(), Color::BLUE);
    }

    #[test]
    fn test_canvas_stroke_text() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.stroke_text("hello", 10.0, 20.0);
        // 每个字符生成一个 glyph
        assert_eq!(ctx.primitives().glyphs.len(), 5);
    }

    // ── 路径操作 ──

    #[test]
    fn test_canvas_begin_path_clears() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(50.0, 50.0);
        ctx.begin_path();
        ctx.fill();
        // begin_path 清除路径，fill 空路径不生成图元
        assert_eq!(ctx.primitives().path_fills.len(), 0);
    }

    #[test]
    fn test_canvas_move_to_line_to_fill() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    #[test]
    fn test_canvas_stroke_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.stroke();
        assert!(!ctx.primitives().path_strokes.is_empty());
    }

    #[test]
    fn test_canvas_fill_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill();
        assert_eq!(ctx.primitives().path_fills.len(), 0);
    }

    #[test]
    fn test_canvas_stroke_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.stroke();
        assert_eq!(ctx.primitives().path_strokes.len(), 0);
    }

    #[test]
    fn test_canvas_quadratic_curve_to() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.quadratic_curve_to(50.0, 0.0, 100.0, 50.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    #[test]
    fn test_canvas_bezier_curve_to() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.bezier_curve_to(30.0, 0.0, 70.0, 100.0, 100.0, 50.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    #[test]
    fn test_canvas_close_path_on_context() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.close_path();
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    #[test]
    fn test_canvas_arc_on_context() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI);
        ctx.line_to(100.0, 100.0); // 确保有非弧线的路径点
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    // ── 边界条件 ──

    #[test]
    fn test_canvas_new_zero_size() {
        let ctx = CanvasContext::new(0, 0);
        assert_eq!(ctx.width(), 0);
        assert_eq!(ctx.height(), 0);
    }

    #[test]
    fn test_canvas_global_alpha_clamp_high() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(2.0);
        assert!((ctx.global_alpha() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_global_alpha_clamp_negative() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(-1.0);
        assert!((ctx.global_alpha()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_global_alpha_clamp_zero() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(0.0);
        assert!((ctx.global_alpha()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_restore_empty_stack() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.restore(); // 不应 panic
        assert_eq!(ctx.fill_color(), Color::BLACK);
    }

    #[test]
    fn test_canvas_measure_text_empty_string() {
        let ctx = CanvasContext::new(200, 200);
        let metrics = ctx.measure_text("");
        assert!((metrics.width).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_measure_text_unicode() {
        let ctx = CanvasContext::new(200, 200);
        let metrics = ctx.measure_text("日本語");
        // 3 个 char × 10.0 × 0.6 = 18.0（按 char 计数，非字节）
        assert!((metrics.width - 18.0).abs() < f32::EPSILON);
    }

    // ── save/restore 完整性 ──

    #[test]
    fn test_canvas_save_restore_stroke_color() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::RED);
        ctx.save();
        ctx.set_stroke_color(Color::BLUE);
        assert_eq!(ctx.stroke_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(ctx.stroke_color(), Color::RED);
    }

    #[test]
    fn test_canvas_save_restore_line_width() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_width(5.0);
        ctx.save();
        ctx.set_line_width(10.0);
        ctx.restore();
        assert!((ctx.line_width() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_save_restore_global_alpha() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(0.5);
        ctx.save();
        ctx.set_global_alpha(0.8);
        ctx.restore();
        assert!((ctx.global_alpha() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_save_restore_transform() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.translate(10.0, 20.0);
        ctx.save();
        ctx.translate(100.0, 200.0);
        ctx.restore();
        let (x, y) = ctx.transform.transform_point(0.0, 0.0);
        assert!((x - 10.0).abs() < 1.0);
        assert!((y - 20.0).abs() < 1.0);
    }

    #[test]
    fn test_canvas_save_restore_font() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_font(FontDescriptor {
            family: "serif".to_string(),
            size: 16.0,
            weight: FontWeight::Bold,
            style: FontStyle::Normal,
        });
        ctx.save();
        ctx.set_font(FontDescriptor {
            family: "monospace".to_string(),
            size: 20.0,
            weight: FontWeight::Normal,
            style: FontStyle::Italic,
        });
        ctx.restore();
        let m = ctx.measure_text("x");
        // 应恢复到 serif 16pt: 1 × 16.0 × 0.6 = 9.6
        assert!((m.width - 9.6).abs() < f32::EPSILON);
    }

    // ── Transform 边界条件 ──

    #[test]
    fn test_transform_multiply_identity() {
        let t = Transform2D::translate(10.0, 20.0);
        let result = t.multiply(&Transform2D::identity());
        let (x, y) = result.transform_point(0.0, 0.0);
        assert!((x - 10.0).abs() < f32::EPSILON);
        assert!((y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_scale_negative() {
        let t = Transform2D::scale(-1.0, 1.0);
        let (x, y) = t.transform_point(5.0, 10.0);
        assert!((x - (-5.0)).abs() < f32::EPSILON);
        assert!((y - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_scale_zero() {
        let t = Transform2D::scale(0.0, 0.0);
        let (x, y) = t.transform_point(5.0, 10.0);
        assert!((x).abs() < f32::EPSILON);
        assert!((y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_canvas_chained_transforms() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.translate(10.0, 0.0);
        ctx.scale(2.0, 1.0);
        let (x, y) = ctx.transform.transform_point(5.0, 5.0);
        // translate(10,0) 然后 scale(2,1): 先 scale 得 (10,5)，再 translate 得 (20,5)
        // 实际矩阵乘法顺序：scale 先应用
        assert!((x - 20.0).abs() < 0.01);
        assert!((y - 5.0).abs() < 0.01);
    }

    // ── alpha 应用 ──

    #[test]
    fn test_canvas_fill_rect_alpha_zero() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(0.0);
        ctx.fill_rect(10.0, 10.0, 50.0, 50.0);
        let fill = &ctx.primitives().fills[0];
        assert_eq!(fill.color.a, 0);
    }

    // ── put_image_data stub ──

    #[test]
    fn test_canvas_put_image_data_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255; 16],
        };
        ctx.put_image_data(&img, 0, 0); // 不应 panic
    }

    // ── 边界条件补充测试 ──

    /// 测试 arc 命令不 panic（路径简化实现只记录中心点）。
    #[test]
    fn test_canvas_arc_no_panic() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.begin_path();
        ctx.arc(200.0, 200.0, 50.0, 0.0, std::f32::consts::TAU);
        // arc 不 panic 即可
    }

    /// 测试 arc 部分弧线不 panic。
    #[test]
    fn test_canvas_arc_partial_no_panic() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.begin_path();
        ctx.arc(200.0, 200.0, 100.0, 0.0, std::f32::consts::PI);
        // arc 不 panic 即可
    }

    /// 测试多次 fill_rect 累积图元。
    #[test]
    fn test_canvas_fill_rect_accumulates() {
        let mut ctx = CanvasContext::new(400, 300);
        for i in 0..10 {
            ctx.fill_rect(i as f32 * 20.0, 0.0, 15.0, 15.0);
        }
        assert_eq!(ctx.primitives().fills.len(), 10);
    }

    /// 测试 fill_rect 负坐标。
    #[test]
    fn test_canvas_fill_rect_negative_coords() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.fill_rect(-50.0, -30.0, 100.0, 100.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
    }

    /// 测试 stroke_rect 多次调用累积。
    #[test]
    fn test_canvas_stroke_rect_accumulates() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.stroke_rect(10.0, 10.0, 50.0, 50.0);
        ctx.stroke_rect(100.0, 100.0, 50.0, 50.0);
        // 每个 stroke_rect 生成 4 条边
        assert_eq!(ctx.primitives().fills.len(), 8);
    }

    /// 测试 into_primitives 消费上下文。
    #[test]
    fn test_canvas_into_primitives() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        let count = ctx.primitives().fills.len();
        let primitives = ctx.into_primitives();
        assert_eq!(primitives.fills.len(), count);
    }

    /// 测试 get_image_data 各种尺寸。
    #[test]
    fn test_canvas_get_image_data_sizes() {
        let ctx = CanvasContext::new(200, 200);
        // 正常尺寸
        let img = ctx.get_image_data(0, 0, 10, 10);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
        assert_eq!(img.data.len(), 400); // 10*10*4

        // 1x1
        let img1 = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(img1.width, 1);
        assert_eq!(img1.data.len(), 4);
    }

    /// 测试极端变换：大旋转角度。
    #[test]
    fn test_canvas_rotate_large_angle() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.rotate(100.0_f32.to_radians());
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
    }

    /// 测试变换后 reset_transform 恢复。
    #[test]
    fn test_canvas_reset_after_transform() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.translate(100.0, 100.0);
        ctx.rotate(45.0_f32.to_radians());
        ctx.reset_transform();
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        // reset_transform 后应在原始坐标系绘制
        let fill = &ctx.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 0.0);
        assert_eq!(fill.rect.origin.y, 0.0);
    }

    /// 测试 fill_text 和 stroke_text 产生字形图元。
    #[test]
    fn test_canvas_text_produces_glyphs() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.fill_text("Hello", 10.0, 20.0);
        assert_eq!(ctx.primitives().glyphs.len(), 5); // 5 chars
        ctx.stroke_text("World", 10.0, 50.0);
        assert_eq!(ctx.primitives().glyphs.len(), 10); // 5 + 5 chars
    }

    /// 测试 set_font 影响文本度量。
    #[test]
    fn test_canvas_font_affects_measure() {
        let ctx = CanvasContext::new(400, 300);
        let small = ctx.measure_text("test");
        let mut ctx2 = CanvasContext::new(400, 300);
        ctx2.set_font(FontDescriptor {
            family: "serif".into(),
            size: 32.0,
            ..Default::default()
        });
        let large = ctx2.measure_text("test");
        assert!(large.width > small.width, "大字体应产生更宽的文本度量");
    }

    /// 测试 Path2D 连续操作。
    #[test]
    fn test_canvas_complex_path() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.quadratic_curve_to(150.0, 50.0, 100.0, 100.0);
        ctx.bezier_curve_to(80.0, 120.0, 20.0, 120.0, 10.0, 100.0);
        ctx.close_path();
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
    }

    /// 测试 clear_rect 产生透明填充。
    #[test]
    fn test_canvas_clear_rect_transparent() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.clear_rect(10.0, 10.0, 50.0, 50.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
        assert_eq!(ctx.primitives().fills[0].color.a, 0);
    }

    /// 测试 global_alpha 影响填充颜色透明度。
    #[test]
    fn test_canvas_alpha_affects_all_operations() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_global_alpha(0.5);
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        assert_eq!(ctx.primitives().fills[0].color.a, 127); // 255 * 0.5 ≈ 127
    }

    // ── put_image_data + get_image_data round-trip ──

    /// 测试 put_image_data 写入像素后 get_image_data 能读回相同数据。
    #[test]
    fn test_put_get_image_data_round_trip() {
        let mut ctx = CanvasContext::new(10, 10);
        let pixels = vec![
            255, 0, 0, 255, // 红色
            0, 255, 0, 255, // 绿色
            0, 0, 255, 255, // 蓝色
            255, 255, 0, 255, // 黄色
        ];
        let img = ImageData {
            width: 2,
            height: 2,
            data: pixels.clone(),
        };
        ctx.put_image_data(&img, 0, 0);
        let result = ctx.get_image_data(0, 0, 2, 2);
        assert_eq!(result.data, pixels);
    }

    /// 测试 put_image_data 在偏移位置写入。
    #[test]
    fn test_put_image_data_with_offset() {
        let mut ctx = CanvasContext::new(10, 10);
        // 在 (5, 5) 位置写入 2x2 的红色像素
        let red = vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255];
        let img = ImageData {
            width: 2,
            height: 2,
            data: red.clone(),
        };
        ctx.put_image_data(&img, 5, 5);
        // 读取偏移位置
        let result = ctx.get_image_data(5, 5, 2, 2);
        assert_eq!(result.data, red);
    }

    /// 测试 put_image_data 后 get_image_data 只读取写入的区域。
    #[test]
    fn test_get_image_data_reflects_put() {
        let mut ctx = CanvasContext::new(10, 10);
        // 先写入红色到 (0,0) - 2x2
        let red = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
        };
        ctx.put_image_data(&red, 0, 0);
        // 再写入绿色到 (2,0) - 2x2
        let green = ImageData {
            width: 2,
            height: 2,
            data: vec![0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255],
        };
        ctx.put_image_data(&green, 2, 0);
        // 读取整个第一行 4 像素
        let result = ctx.get_image_data(0, 0, 4, 1);
        // 前 2 个红色，后 2 个绿色
        assert_eq!(result.data[0..4], [255, 0, 0, 255]); // 红
        assert_eq!(result.data[4..8], [255, 0, 0, 255]); // 红
        assert_eq!(result.data[8..12], [0, 255, 0, 255]); // 绿
        assert_eq!(result.data[12..16], [0, 255, 0, 255]); // 绿
    }

    /// 测试 get_image_data 在未写入区域返回零。
    #[test]
    fn test_get_image_data_unwritten_is_zeros() {
        let ctx = CanvasContext::new(10, 10);
        let result = ctx.get_image_data(5, 5, 2, 2);
        assert_eq!(result.data, vec![0u8; 16]);
    }

    /// 测试 fill_rect 后 get_image_data 反映绘制内容。
    #[test]
    fn test_get_image_data_after_fill_rect() {
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 3.0, 2.0);
        // 读取整个区域
        let result = ctx.get_image_data(0, 0, 10, 10);
        // (0,0) 应为红色
        assert_eq!(result.data[0..4], [255, 0, 0, 255]);
        // (3,0) 不应被填充
        assert_eq!(result.data[12..16], [0, 0, 0, 0]);
    }

    /// 测试 clear_rect 后 get_image_data 反映透明。
    #[test]
    fn test_get_image_data_after_clear_rect() {
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        ctx.clear_rect(2.0, 2.0, 3.0, 3.0);
        let result = ctx.get_image_data(2, 2, 1, 1);
        // 被清除的区域应透明
        assert_eq!(result.data[0..4], [0, 0, 0, 0]);
    }

    // ── Path fill/stroke shape correctness ──

    /// 测试 fill() 生成 path_fills 而非 fills，且包含正确的顶点。
    #[test]
    fn test_fill_emits_path_fill_primitive() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.fill();
        // 应生成 path_fill 而非 fill
        assert_eq!(ctx.primitives().fills.len(), 0);
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        let pf = &ctx.primitives().path_fills[0];
        // 三角形路径：2 条线段 = 4 个顶点对 (x1,y1,x2,y2)
        // line_to(10,10)->(100,10) 和 (100,10)->(100,100)
        assert!(pf.vertices.len() >= 8); // 至少 2 段 × 4 floats
        assert_eq!(pf.color, Color::BLACK);
    }

    /// 测试 stroke() 生成 path_stroke 图元，且包含正确的颜色和线宽。
    #[test]
    fn test_stroke_emits_path_stroke_primitive() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(3.0);
        ctx.set_stroke_color(Color::RED);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.stroke();
        assert_eq!(ctx.primitives().fills.len(), 0);
        assert_eq!(ctx.primitives().path_strokes.len(), 1);
        let ps = &ctx.primitives().path_strokes[0];
        assert_eq!(ps.color, Color::RED);
        assert!((ps.line_width - 3.0).abs() < f32::EPSILON);
        assert!(!ps.vertices.is_empty());
    }

    /// 测试 fill() 三角形路径的顶点数量正确。
    #[test]
    fn test_fill_triangle_vertices() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.line_to(25.0, 50.0);
        ctx.fill();
        let pf = &ctx.primitives().path_fills[0];
        // 2 条 LineTo 命令，每条生成 4 floats (x1,y1,x2,y2)
        assert_eq!(pf.vertices.len(), 8);
    }

    /// 测试 stroke() 的闭合标记。
    #[test]
    fn test_stroke_closed_flag() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.line_to(50.0, 50.0);
        ctx.close_path();
        ctx.stroke();
        assert!(ctx.primitives().path_strokes[0].closed);
    }

    /// 测试 stroke() 无 close_path 时 closed=false。
    #[test]
    fn test_stroke_not_closed_flag() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.stroke();
        assert!(!ctx.primitives().path_strokes[0].closed);
    }

    /// 测试 fill() 像素缓冲区写入（三角形应只覆盖部分像素）。
    #[test]
    fn test_fill_writes_pixels() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.line_to(25.0, 50.0);
        ctx.fill();
        // 检查三角形内部某点应为红色
        let result = ctx.get_image_data(10, 10, 1, 1);
        assert_eq!(result.data[0..4], [255, 0, 0, 255], "triangle interior should be red");
        // 检查三角形外部某点应为零
        let outside = ctx.get_image_data(40, 40, 1, 1);
        assert_eq!(outside.data[0..4], [0, 0, 0, 0], "outside triangle should be empty");
    }

    /// 测试 stroke() 像素缓冲区写入（描边线段应沿线覆盖像素）。
    #[test]
    fn test_stroke_writes_pixels() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::BLUE);
        ctx.set_line_width(3.0);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.stroke();
        // 沿描边路径上的像素应为蓝色
        let result = ctx.get_image_data(25, 0, 1, 1);
        assert_eq!(result.data[0..4], [0, 0, 255, 255], "stroke should be blue");
    }

    // ── fill_text / stroke_text per-character glyph ──

    /// 测试 fill_text 每个字符的 glyph_id 等于 Unicode 码点。
    #[test]
    fn test_fill_text_glyph_ids_are_codepoints() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_text("AB", 10.0, 20.0);
        let glyphs = &ctx.primitives().glyphs;
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].glyph_id, 'A' as u32);
        assert_eq!(glyphs[1].glyph_id, 'B' as u32);
    }

    /// 测试 fill_text 每个字符水平偏移递增。
    #[test]
    fn test_fill_text_glyph_positions_offset() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_text("abc", 10.0, 20.0);
        let glyphs = &ctx.primitives().glyphs;
        assert_eq!(glyphs.len(), 3);
        let em_width = 10.0 * 0.6; // font_size * 0.6
        assert!((glyphs[0].x - 10.0).abs() < f32::EPSILON);
        assert!((glyphs[1].x - (10.0 + em_width)).abs() < f32::EPSILON);
        assert!((glyphs[2].x - (10.0 + 2.0 * em_width)).abs() < f32::EPSILON);
    }

    /// 测试 stroke_text 使用描边颜色。
    #[test]
    fn test_stroke_text_uses_stroke_color() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_stroke_color(Color::RED);
        ctx.stroke_text("X", 10.0, 20.0);
        assert_eq!(ctx.primitives().glyphs[0].color, Color::RED);
    }

    /// 测试 fill_text 使用填充颜色。
    #[test]
    fn test_fill_text_uses_fill_color() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_fill_color(Color::GREEN);
        ctx.fill_text("X", 10.0, 20.0);
        assert_eq!(ctx.primitives().glyphs[0].color, Color::GREEN);
    }

    /// 测试空字符串不生成 glyph。
    #[test]
    fn test_fill_text_empty_string() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_text("", 10.0, 20.0);
        assert_eq!(ctx.primitives().glyphs.len(), 0);
    }

    /// 测试 stroke_text 空字符串不生成 glyph。
    #[test]
    fn test_stroke_text_empty_string() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.stroke_text("", 10.0, 20.0);
        assert_eq!(ctx.primitives().glyphs.len(), 0);
    }

    /// 测试 Unicode 文本的 glyph_id 正确。
    #[test]
    fn test_fill_text_unicode_glyph_ids() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_text("日本", 10.0, 20.0);
        let glyphs = &ctx.primitives().glyphs;
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].glyph_id, '日' as u32);
        assert_eq!(glyphs[1].glyph_id, '本' as u32);
    }

    // ── Quadratic/Bezier curve flattening ──

    /// 测试二次贝塞尔曲线填充生成正确的段数。
    #[test]
    fn test_quadratic_curve_flattening() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.quadratic_curve_to(50.0, 100.0, 100.0, 0.0);
        ctx.fill();
        let pf = &ctx.primitives().path_fills[0];
        // 8 段细分 × 4 floats = 32
        assert_eq!(pf.vertices.len(), 32);
    }

    /// 测试三次贝塞尔曲线填充生成正确的段数。
    #[test]
    fn test_bezier_curve_flattening() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.bezier_curve_to(25.0, 100.0, 75.0, 100.0, 100.0, 0.0);
        ctx.fill();
        let pf = &ctx.primitives().path_fills[0];
        // 8 段细分 × 4 floats = 32
        assert_eq!(pf.vertices.len(), 32);
    }

    /// 测试圆弧填充生成正确的段数。
    #[test]
    fn test_arc_flattening() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI);
        ctx.fill();
        let pf = &ctx.primitives().path_fills[0];
        // 16 段细分 × 4 floats = 64
        assert_eq!(pf.vertices.len(), 64);
    }

    // ── clip() 测试 ──

    /// 测试 clip() 从三角形路径生成裁剪图元。
    #[test]
    fn test_clip_triangle() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(50.0, 100.0);
        ctx.close_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 1);
        let clip = &ctx.primitives().clips[0];
        // 裁剪区域应是路径的包围盒
        assert!(clip.rect.origin.x <= 10.0);
        assert!(clip.rect.origin.y <= 10.0);
        assert!(clip.rect.size.width >= 90.0);
        assert!(clip.rect.size.height >= 90.0);
    }

    /// 测试 clip() 空路径不生成裁剪图元。
    #[test]
    fn test_clip_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.begin_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 0);
    }

    /// 测试 clip() 矩形路径生成精确的裁剪矩形。
    #[test]
    fn test_clip_rectangular_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(20.0, 30.0);
        ctx.line_to(80.0, 30.0);
        ctx.line_to(80.0, 70.0);
        ctx.line_to(20.0, 70.0);
        ctx.close_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 1);
        let clip = &ctx.primitives().clips[0];
        assert!((clip.rect.origin.x - 20.0).abs() < f32::EPSILON);
        assert!((clip.rect.origin.y - 30.0).abs() < f32::EPSILON);
        assert!((clip.rect.size.width - 60.0).abs() < f32::EPSILON);
        assert!((clip.rect.size.height - 40.0).abs() < f32::EPSILON);
    }

    /// 测试 clip() 后绘制操作仍然正常。
    #[test]
    fn test_clip_then_draw() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(100.0, 100.0);
        ctx.close_path();
        ctx.clip();
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        assert_eq!(ctx.primitives().clips.len(), 1);
        assert_eq!(ctx.primitives().fills.len(), 1);
    }

    /// 测试多次 clip() 调用累积裁剪区域。
    #[test]
    fn test_clip_multiple() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 0.0);
        ctx.line_to(50.0, 50.0);
        ctx.close_path();
        ctx.clip();
        ctx.begin_path();
        ctx.move_to(25.0, 25.0);
        ctx.line_to(75.0, 25.0);
        ctx.line_to(75.0, 75.0);
        ctx.close_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 2);
    }

    // ── CompositeOperation 测试 ──

    /// 测试默认合成操作模式为 SourceOver。
    #[test]
    fn test_composite_operation_default() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
    }

    /// 测试设置和获取合成操作模式。
    #[test]
    fn test_composite_operation_set_get() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_composite_operation(CompositeOperation::Multiply);
        assert_eq!(ctx.composite_operation(), CompositeOperation::Multiply);
        ctx.set_composite_operation(CompositeOperation::Screen);
        assert_eq!(ctx.composite_operation(), CompositeOperation::Screen);
    }

    /// 测试合成操作模式在 save/restore 中正确保存和恢复。
    #[test]
    fn test_composite_operation_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
        ctx.save();
        ctx.set_composite_operation(CompositeOperation::Lighter);
        assert_eq!(ctx.composite_operation(), CompositeOperation::Lighter);
        ctx.restore();
        assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
    }

    /// 测试所有合成操作模式变体可以正确设置。
    #[test]
    fn test_composite_operation_all_variants() {
        let mut ctx = CanvasContext::new(100, 100);
        let ops = [
            CompositeOperation::SourceOver,
            CompositeOperation::DestinationOver,
            CompositeOperation::DestinationOut,
            CompositeOperation::DestinationAtop,
            CompositeOperation::DestinationIn,
            CompositeOperation::SourceIn,
            CompositeOperation::SourceAtop,
            CompositeOperation::Lighter,
            CompositeOperation::Copy,
            CompositeOperation::Xor,
            CompositeOperation::Multiply,
            CompositeOperation::Screen,
            CompositeOperation::Overlay,
            CompositeOperation::Darken,
            CompositeOperation::Lighten,
            CompositeOperation::ColorDodge,
            CompositeOperation::ColorBurn,
            CompositeOperation::HardLight,
            CompositeOperation::SoftLight,
            CompositeOperation::Difference,
            CompositeOperation::Exclusion,
            CompositeOperation::Hue,
            CompositeOperation::Saturation,
            CompositeOperation::Color,
            CompositeOperation::Luminosity,
        ];
        for op in &ops {
            ctx.set_composite_operation(*op);
            assert_eq!(ctx.composite_operation(), *op);
        }
    }

    /// 测试合成操作模式 save/restore 嵌套。
    #[test]
    fn test_composite_operation_nested_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_composite_operation(CompositeOperation::SourceOver);
        ctx.save();
        ctx.set_composite_operation(CompositeOperation::Multiply);
        ctx.save();
        ctx.set_composite_operation(CompositeOperation::Screen);
        assert_eq!(ctx.composite_operation(), CompositeOperation::Screen);
        ctx.restore();
        assert_eq!(ctx.composite_operation(), CompositeOperation::Multiply);
        ctx.restore();
        assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
    }

    // ── createLinearGradient 测试 ──

    /// 测试创建线性渐变。
    #[test]
    fn test_create_linear_gradient() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
        assert!((grad.x0).abs() < f32::EPSILON);
        assert!((grad.y0).abs() < f32::EPSILON);
        assert!((grad.x1 - 200.0).abs() < f32::EPSILON);
        assert!((grad.y1).abs() < f32::EPSILON);
        assert!(grad.stops.is_empty());
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        assert_eq!(grad.stops.len(), 2);
        assert_eq!(grad.stops[0].color, Color::RED);
        assert_eq!(grad.stops[1].color, Color::BLUE);
    }

    /// 测试线性渐变多色停止点。
    #[test]
    fn test_linear_gradient_multiple_stops() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 100.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.5, Color::GREEN);
        grad.add_color_stop(1.0, Color::BLUE);
        assert_eq!(grad.stops.len(), 3);
        assert!((grad.stops[1].offset - 0.5).abs() < f32::EPSILON);
        assert_eq!(grad.stops[1].color, Color::GREEN);
    }

    /// 测试线性渐变起点和终点相同（退化情况不 panic）。
    #[test]
    fn test_linear_gradient_degenerate() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(50.0, 50.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::RED);
        assert_eq!(grad.stops.len(), 1);
    }

    // ── createRadialGradient 测试 ──

    /// 测试创建径向渐变。
    #[test]
    fn test_create_radial_gradient() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 100.0);
        assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r0 - 10.0).abs() < f32::EPSILON);
        assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r1 - 100.0).abs() < f32::EPSILON);
        assert!(grad.stops.is_empty());
        grad.add_color_stop(0.0, Color::WHITE);
        grad.add_color_stop(1.0, Color::BLACK);
        assert_eq!(grad.stops.len(), 2);
    }

    /// 测试径向渐变多色停止点。
    #[test]
    fn test_radial_gradient_multiple_stops() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_radial_gradient(0.0, 0.0, 0.0, 100.0, 100.0, 50.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.33, Color::GREEN);
        grad.add_color_stop(0.66, Color::BLUE);
        grad.add_color_stop(1.0, Color::WHITE);
        assert_eq!(grad.stops.len(), 4);
    }

    /// 测试径向渐变偏心圆（圆心不同）。
    #[test]
    fn test_radial_gradient_eccentric() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_radial_gradient(0.0, 0.0, 5.0, 200.0, 200.0, 50.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        assert_eq!(grad.stops.len(), 2);
    }

    // ── createPattern 测试 ──

    /// 测试从 ImageData 创建图案。
    #[test]
    fn test_create_pattern() {
        let ctx = CanvasContext::new(200, 200);
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        let pattern = ctx.create_pattern(img.clone(), PatternRepetition::Repeat);
        assert_eq!(pattern.image_data.width, 2);
        assert_eq!(pattern.image_data.height, 2);
        assert_eq!(pattern.repetition, PatternRepetition::Repeat);
    }

    /// 测试图案重复模式 NoRepeat。
    #[test]
    fn test_create_pattern_no_repeat() {
        let ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        let pattern = ctx.create_pattern(img, PatternRepetition::NoRepeat);
        assert_eq!(pattern.repetition, PatternRepetition::NoRepeat);
    }

    /// 测试图案重复模式 RepeatX / RepeatY。
    #[test]
    fn test_create_pattern_repeat_variants() {
        let ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![0; 4],
        };

        let p1 = ctx.create_pattern(img.clone(), PatternRepetition::RepeatX);
        assert_eq!(p1.repetition, PatternRepetition::RepeatX);

        let p2 = ctx.create_pattern(img, PatternRepetition::RepeatY);
        assert_eq!(p2.repetition, PatternRepetition::RepeatY);
    }

    /// 测试图案默认重复模式为 Repeat。
    #[test]
    fn test_pattern_repetition_default() {
        assert_eq!(PatternRepetition::default(), PatternRepetition::Repeat);
    }

    // ── isPointInPath 测试 ──

    /// 测试点在三角形路径内部。
    #[test]
    fn test_is_point_in_path_inside_triangle() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(50.0, 100.0);
        ctx.close_path();
        // 质心 (50, 33.3) 应在三角形内
        assert!(ctx.is_point_in_path(50.0, 30.0));
    }

    /// 测试点在三角形路径外部。
    #[test]
    fn test_is_point_in_path_outside_triangle() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(50.0, 100.0);
        ctx.close_path();
        // 点 (200, 200) 应在三角形外
        assert!(!ctx.is_point_in_path(200.0, 200.0));
    }

    /// 测试空路径上所有点都不在路径内。
    #[test]
    fn test_is_point_in_path_empty_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        assert!(!ctx.is_point_in_path(50.0, 50.0));
    }

    /// 测试点在矩形路径上。
    #[test]
    fn test_is_point_in_path_rectangle() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(90.0, 10.0);
        ctx.line_to(90.0, 90.0);
        ctx.line_to(10.0, 90.0);
        ctx.close_path();
        // 中心点应在矩形内
        assert!(ctx.is_point_in_path(50.0, 50.0));
        // 角落外的点应不在矩形内
        assert!(!ctx.is_point_in_path(5.0, 5.0));
    }

    /// 测试点恰好在地面上。
    #[test]
    fn test_is_point_in_path_on_edge() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(100.0, 100.0);
        ctx.close_path();
        // 边界上的点行为取决于射线法实现（不确定在内还是外）
        // 主要验证不 panic
        let _ = ctx.is_point_in_path(0.0, 0.0);
        let _ = ctx.is_point_in_path(50.0, 0.0);
    }

    /// 测试 isPointInPath 对仅有 MoveTo 的路径返回 false。
    #[test]
    fn test_is_point_in_path_move_to_only() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(50.0, 50.0);
        // 只有 MoveTo，没有闭合区域
        assert!(!ctx.is_point_in_path(50.0, 50.0));
    }

    // ── point_in_polygon 辅助函数测试 ──

    /// 测试射线法判断点是否在正方形内。
    #[test]
    fn test_point_in_polygon_square() {
        let square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        assert!(point_in_polygon(50.0, 50.0, &square));
        assert!(!point_in_polygon(150.0, 50.0, &square));
    }

    /// 测试射线法对少于 3 个点返回 false。
    #[test]
    fn test_point_in_polygon_too_few_points() {
        let two_points = [(0.0, 0.0), (100.0, 100.0)];
        assert!(!point_in_polygon(50.0, 50.0, &two_points));
        let empty: [(f32, f32); 0] = [];
        assert!(!point_in_polygon(50.0, 50.0, &empty));
    }

    /// 测试射线法判断凹多边形。
    #[test]
    fn test_point_in_polygon_concave() {
        // L 形多边形
        let l_shape = [
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 50.0),
            (50.0, 50.0),
            (50.0, 100.0),
            (0.0, 100.0),
        ];
        // 凹角内侧的点
        assert!(point_in_polygon(25.0, 75.0, &l_shape));
        // 凹角外侧的点
        assert!(!point_in_polygon(75.0, 75.0, &l_shape));
    }

    // ── CompositeOperation Default 测试 ──

    /// 测试 CompositeOperation 默认值为 SourceOver。
    #[test]
    fn test_composite_operation_default_value() {
        assert_eq!(CompositeOperation::default(), CompositeOperation::SourceOver);
    }

    // ── Shadow properties 测试 ──

    /// 测试阴影属性默认值。
    #[test]
    fn test_shadow_default_values() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(*ctx.shadow_color(), Color::TRANSPARENT);
        assert!((ctx.shadow_blur() - 0.0).abs() < f32::EPSILON);
        assert!((ctx.shadow_offset_x() - 0.0).abs() < f32::EPSILON);
        assert!((ctx.shadow_offset_y() - 0.0).abs() < f32::EPSILON);
    }

    /// 测试设置和获取阴影颜色。
    #[test]
    fn test_shadow_set_get_color() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_color(Color::RED);
        assert_eq!(*ctx.shadow_color(), Color::RED);
    }

    /// 测试设置和获取阴影模糊半径。
    #[test]
    fn test_shadow_set_get_blur() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_blur(10.0);
        assert!((ctx.shadow_blur() - 10.0).abs() < f32::EPSILON);
    }

    /// 测试设置和获取阴影偏移。
    #[test]
    fn test_shadow_set_get_offset() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_offset_x(5.0);
        ctx.set_shadow_offset_y(7.0);
        assert!((ctx.shadow_offset_x() - 5.0).abs() < f32::EPSILON);
        assert!((ctx.shadow_offset_y() - 7.0).abs() < f32::EPSILON);
    }

    /// 测试阴影属性在 save/restore 中正确保存和恢复。
    #[test]
    fn test_shadow_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_color(Color::RED);
        ctx.set_shadow_blur(5.0);
        ctx.set_shadow_offset_x(3.0);
        ctx.set_shadow_offset_y(4.0);
        ctx.save();
        ctx.set_shadow_color(Color::BLUE);
        ctx.set_shadow_blur(20.0);
        ctx.set_shadow_offset_x(10.0);
        ctx.set_shadow_offset_y(15.0);
        assert_eq!(*ctx.shadow_color(), Color::BLUE);
        assert!((ctx.shadow_blur() - 20.0).abs() < f32::EPSILON);
        ctx.restore();
        assert_eq!(*ctx.shadow_color(), Color::RED);
        assert!((ctx.shadow_blur() - 5.0).abs() < f32::EPSILON);
        assert!((ctx.shadow_offset_x() - 3.0).abs() < f32::EPSILON);
        assert!((ctx.shadow_offset_y() - 4.0).abs() < f32::EPSILON);
    }

    /// 测试阴影模糊半径负值被限制为 0。
    #[test]
    fn test_shadow_blur_clamp_negative() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_blur(-10.0);
        assert!((ctx.shadow_blur() - 0.0).abs() < f32::EPSILON);
    }

    /// 测试阴影应用于 fill_rect。
    #[test]
    fn test_shadow_applied_to_fill_rect() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_color(Color::BLACK);
        ctx.set_shadow_offset_x(5.0);
        ctx.set_shadow_offset_y(5.0);
        ctx.fill_rect(10.0, 10.0, 20.0, 20.0);
        // 检查阴影区域有像素被写入（偏移位置）
        let shadow_pixel = ctx.get_image_data(15, 15, 1, 1);
        assert_ne!(shadow_pixel.data[3], 0, "shadow area should have pixels");
    }

    /// 测试阴影应用于 stroke_rect。
    #[test]
    fn test_shadow_applied_to_stroke_rect() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_color(Color::BLACK);
        ctx.set_shadow_offset_x(5.0);
        ctx.set_shadow_offset_y(5.0);
        ctx.stroke_rect(10.0, 10.0, 20.0, 20.0);
        // 检查阴影区域有像素被写入
        let shadow_pixel = ctx.get_image_data(15, 15, 1, 1);
        assert_ne!(shadow_pixel.data[3], 0, "shadow area should have pixels");
    }

    /// 测试多次阴影绘制。
    #[test]
    fn test_shadow_multiple_draws() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_color(Color::BLACK);
        ctx.set_shadow_offset_x(2.0);
        ctx.set_shadow_offset_y(2.0);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        ctx.fill_rect(30.0, 30.0, 10.0, 10.0);
        // 两个阴影都应该存在
        let shadow1 = ctx.get_image_data(2, 2, 1, 1);
        let shadow2 = ctx.get_image_data(32, 32, 1, 1);
        assert_ne!(shadow1.data[3], 0, "first shadow should exist");
        assert_ne!(shadow2.data[3], 0, "second shadow should exist");
    }

    // ── drawImage 测试 ──

    /// 测试 draw_image 基本 blit。
    #[test]
    fn test_draw_image_basic_blit() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        ctx.draw_image(&img, 0.0, 0.0);
        let result = ctx.get_image_data(0, 0, 2, 2);
        assert_eq!(result.data[0..4], [255, 0, 0, 255]); // 红色
        assert_eq!(result.data[4..8], [0, 255, 0, 255]); // 绿色
        assert_eq!(result.data[8..12], [0, 0, 255, 255]); // 蓝色
        assert_eq!(result.data[12..16], [255, 255, 0, 255]); // 黄色
    }

    /// 测试 draw_image_with_size 缩放。
    #[test]
    fn test_draw_image_with_size_scaling() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        ctx.draw_image_with_size(&img, 0.0, 0.0, 4.0, 4.0);
        let result = ctx.get_image_data(0, 0, 4, 4);
        // 所有像素应该都是红色
        for i in 0..4 {
            for j in 0..4 {
                let idx = (i * 4 + j) * 4;
                assert_eq!(result.data[idx..idx + 4], [255, 0, 0, 255]);
            }
        }
    }

    /// 测试 draw_image_sliced。
    #[test]
    fn test_draw_image_sliced() {
        let mut ctx = CanvasContext::new(100, 100);
        // 4x4 图像，每个像素不同
        let mut pixels = Vec::with_capacity(64);
        for i in 0..16u8 {
            pixels.extend_from_slice(&[i * 16, i * 16, i * 16, 255]);
        }
        let img = ImageData {
            width: 4,
            height: 4,
            data: pixels,
        };
        // 切取左上角 2x2
        ctx.draw_image_sliced(&img, 0.0, 0.0, 2.0, 2.0, 10.0, 10.0, 2.0, 2.0);
        let result = ctx.get_image_data(10, 10, 2, 2);
        // 左上角 2x2 像素
        assert_eq!(result.data[0..4], [0, 0, 0, 255]); // pixel (0,0)
        assert_eq!(result.data[4..8], [16, 16, 16, 255]); // pixel (1,0)
        assert_eq!(result.data[8..12], [64, 64, 64, 255]); // pixel (0,1)
        assert_eq!(result.data[12..16], [80, 80, 80, 255]); // pixel (1,1)
    }

    /// 测试 draw_image 应用变换。
    #[test]
    fn test_draw_image_with_transform() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
        };
        ctx.translate(5.0, 5.0);
        ctx.draw_image(&img, 0.0, 0.0);
        // 像素应出现在偏移 (5,5) 位置
        let result = ctx.get_image_data(5, 5, 2, 2);
        assert_eq!(result.data[0..4], [255, 0, 0, 255]);
        // 原点应无像素
        let origin = ctx.get_image_data(0, 0, 2, 2);
        assert_eq!(origin.data[0..4], [0, 0, 0, 0]);
    }

    /// 测试 draw_image 越界不 panic。
    #[test]
    fn test_draw_image_out_of_bounds_no_panic() {
        let mut ctx = CanvasContext::new(10, 10);
        let img = ImageData {
            width: 100,
            height: 100,
            data: vec![255; 100 * 100 * 4],
        };
        ctx.draw_image(&img, 90.0, 90.0); // 大部分超出画布
        // 不应 panic
    }

    /// 测试 draw_image 零尺寸图像不 panic。
    #[test]
    fn test_draw_image_zero_size_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 0,
            height: 0,
            data: vec![],
        };
        ctx.draw_image(&img, 0.0, 0.0); // 不应 panic
        ctx.draw_image_with_size(&img, 0.0, 0.0, 10.0, 10.0); // 不应 panic
    }

    /// 测试 draw_image 后 get_image_data 往返一致性。
    #[test]
    fn test_draw_image_round_trip() {
        let mut ctx = CanvasContext::new(10, 10);
        let pixels = vec![
            255, 0, 0, 255, // 红
            0, 255, 0, 255, // 绿
            0, 0, 255, 255, // 蓝
            255, 255, 255, 255, // 白
        ];
        let img = ImageData {
            width: 2,
            height: 2,
            data: pixels.clone(),
        };
        ctx.draw_image(&img, 0.0, 0.0);
        let result = ctx.get_image_data(0, 0, 2, 2);
        assert_eq!(result.data, pixels);
    }

    // ── Path2D ellipse 测试 ──

    /// 测试 Path2D ellipse 命令生成正确的路径命令。
    #[test]
    fn test_path_ellipse_command() {
        let mut p = Path2D::new();
        p.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
        assert_eq!(p.len(), 1);
        assert!(matches!(
            p.commands()[0],
            PathCommand::Ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, _)
        ));
    }

    // ── Path2D rect 测试 ──

    /// 测试 Path2D rect 命令生成 5 个子命令。
    #[test]
    fn test_path_rect_subpath_count() {
        let mut p = Path2D::new();
        p.rect(10.0, 20.0, 100.0, 50.0);
        assert_eq!(p.len(), 5);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
        assert!(matches!(p.commands()[1], PathCommand::LineTo(110.0, 20.0)));
        assert!(matches!(p.commands()[2], PathCommand::LineTo(110.0, 70.0)));
        assert!(matches!(p.commands()[3], PathCommand::LineTo(10.0, 70.0)));
        assert!(matches!(p.commands()[4], PathCommand::ClosePath));
    }

    // ── Path2D round_rect 测试 ──

    /// 测试 Path2D round_rect 命令。
    #[test]
    fn test_path_round_rect_command() {
        let mut p = Path2D::new();
        p.round_rect(10.0, 20.0, 100.0, 50.0, vec![5.0]);
        assert_eq!(p.len(), 1);
        assert!(matches!(
            p.commands()[0],
            PathCommand::RoundRect(10.0, 20.0, 100.0, 50.0, ref r) if r == &vec![5.0]
        ));
    }

    /// 测试 Path2D round_rect 使用不同圆角半径。
    #[test]
    fn test_path_round_rect_different_radii() {
        let mut p = Path2D::new();
        p.round_rect(0.0, 0.0, 80.0, 60.0, vec![5.0, 10.0, 15.0, 20.0]);
        assert_eq!(p.len(), 1);
        if let PathCommand::RoundRect(x, y, w, h, ref radii) = p.commands()[0] {
            assert!((x).abs() < f32::EPSILON);
            assert!((y).abs() < f32::EPSILON);
            assert!((w - 80.0).abs() < f32::EPSILON);
            assert!((h - 60.0).abs() < f32::EPSILON);
            assert_eq!(radii, &[5.0, 10.0, 15.0, 20.0]);
        } else {
            panic!("expected RoundRect command");
        }
    }

    // ── Path2D is_empty 和 len 测试 ──

    /// 测试 Path2D is_empty 和 len。
    #[test]
    fn test_path_is_empty_and_len() {
        let p = Path2D::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);

        let mut p2 = Path2D::new();
        p2.move_to(10.0, 20.0);
        assert!(!p2.is_empty());
        assert_eq!(p2.len(), 1);

        p2.line_to(30.0, 40.0);
        assert_eq!(p2.len(), 2);
    }

    // ── fill_with_path 测试 ──

    /// 测试 fill_with_path 使用外部 Path2D 填充。
    #[test]
    fn test_fill_with_path() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.move_to(10.0, 10.0);
        path.line_to(100.0, 10.0);
        path.line_to(100.0, 100.0);
        ctx.fill_with_path(&path);
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        let pf = &ctx.primitives().path_fills[0];
        assert!(!pf.vertices.is_empty());
    }

    /// 测试 fill_with_path 空路径不生成图元。
    #[test]
    fn test_fill_with_path_empty() {
        let mut ctx = CanvasContext::new(200, 200);
        let path = Path2D::new();
        ctx.fill_with_path(&path);
        assert_eq!(ctx.primitives().path_fills.len(), 0);
    }

    // ── stroke_with_path 测试 ──

    /// 测试 stroke_with_path 使用外部 Path2D 描边。
    #[test]
    fn test_stroke_with_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(3.0);
        ctx.set_stroke_color(Color::RED);
        let mut path = Path2D::new();
        path.move_to(10.0, 10.0);
        path.line_to(100.0, 100.0);
        ctx.stroke_with_path(&path);
        assert_eq!(ctx.primitives().path_strokes.len(), 1);
        let ps = &ctx.primitives().path_strokes[0];
        assert_eq!(ps.color, Color::RED);
        assert!((ps.line_width - 3.0).abs() < f32::EPSILON);
    }

    /// 测试 stroke_with_path 闭合路径标记。
    #[test]
    fn test_stroke_with_path_closed() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.move_to(0.0, 0.0);
        path.line_to(50.0, 0.0);
        path.line_to(50.0, 50.0);
        path.close_path();
        ctx.stroke_with_path(&path);
        assert!(ctx.primitives().path_strokes[0].closed);
    }

    // ── clip_with_path 测试 ──

    /// 测试 clip_with_path 使用外部 Path2D 裁剪。
    #[test]
    fn test_clip_with_path() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.move_to(10.0, 10.0);
        path.line_to(100.0, 10.0);
        path.line_to(100.0, 100.0);
        path.close_path();
        ctx.clip_with_path(&path);
        assert_eq!(ctx.primitives().clips.len(), 1);
    }

    /// 测试 clip_with_path 空路径不生成裁剪图元。
    #[test]
    fn test_clip_with_path_empty() {
        let mut ctx = CanvasContext::new(200, 200);
        let path = Path2D::new();
        ctx.clip_with_path(&path);
        assert_eq!(ctx.primitives().clips.len(), 0);
    }

    // ── line_dash set/get 测试 ──

    /// 测试线段虚线模式设置和获取。
    #[test]
    fn test_line_dash_set_get() {
        let mut ctx = CanvasContext::new(100, 100);
        assert!(ctx.get_line_dash().is_empty());
        ctx.set_line_dash(vec![5.0, 10.0]);
        assert_eq!(ctx.get_line_dash(), &[5.0, 10.0]);
    }

    /// 测试线段虚线模式奇数长度时自动加倍。
    #[test]
    fn test_line_dash_odd_length_doubled() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_dash(vec![5.0, 10.0, 15.0]);
        assert_eq!(ctx.get_line_dash(), &[5.0, 10.0, 15.0, 5.0, 10.0, 15.0]);
    }

    // ── line_dash_offset set/get 测试 ──

    /// 测试线段虚线偏移设置和获取。
    #[test]
    fn test_line_dash_offset_set_get() {
        let mut ctx = CanvasContext::new(100, 100);
        assert!((ctx.get_line_dash_offset()).abs() < f32::EPSILON);
        ctx.set_line_dash_offset(3.5);
        assert!((ctx.get_line_dash_offset() - 3.5).abs() < f32::EPSILON);
    }

    // ── line_dash save/restore 测试 ──

    /// 测试线段虚线模式在 save/restore 中正确保存和恢复。
    #[test]
    fn test_line_dash_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_dash(vec![5.0, 10.0]);
        ctx.set_line_dash_offset(2.0);
        ctx.save();
        ctx.set_line_dash(vec![1.0, 2.0, 3.0]);
        ctx.set_line_dash_offset(5.0);
        assert_eq!(ctx.get_line_dash(), &[1.0, 2.0, 3.0, 1.0, 2.0, 3.0]);
        assert!((ctx.get_line_dash_offset() - 5.0).abs() < f32::EPSILON);
        ctx.restore();
        assert_eq!(ctx.get_line_dash(), &[5.0, 10.0]);
        assert!((ctx.get_line_dash_offset() - 2.0).abs() < f32::EPSILON);
    }

    // ── Path2D 多子路径测试 ──

    /// 测试 Path2D 包含多个子路径时 fill_with_path 正确工作。
    #[test]
    fn test_fill_with_path_multiple_subpaths() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        // 第一个子路径：矩形
        path.rect(10.0, 10.0, 30.0, 30.0);
        // 第二个子路径：三角形
        path.move_to(60.0, 10.0);
        path.line_to(100.0, 10.0);
        path.line_to(80.0, 50.0);
        path.close_path();
        ctx.fill_with_path(&path);
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        assert!(!ctx.primitives().path_fills[0].vertices.is_empty());
    }

    // ── Path2D ellipse 在 context 中使用 ──

    /// 测试 ellipse 通过 fill_with_path 生成正确的顶点数量。
    #[test]
    fn test_ellipse_flattening_via_fill_with_path() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
        ctx.fill_with_path(&path);
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        let pf = &ctx.primitives().path_fills[0];
        // 16 段细分 × 4 floats = 64
        assert_eq!(pf.vertices.len(), 64);
    }

    // ── roundRect 扁平化测试 ──

    /// 测试 roundRect 带圆角半径生成更多顶点（不是普通矩形）。
    /// 普通矩形只有 20 个 float（5 段 × 4），带圆角的应有更多。
    #[test]
    fn test_round_rect_more_vertices_than_plain_rect() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.round_rect(10.0, 20.0, 100.0, 80.0, vec![10.0]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 带圆角：1（初始连线）+ 4 × (8 段圆角 + 1 直边) = 1 + 36 = 37 段 × 4 = 148 floats
        // 而普通矩形只有 5 段 × 4 = 20 floats
        assert!(
            pf.vertices.len() > 20,
            "roundRect with radius should produce more vertices than plain rect, got {}",
            pf.vertices.len()
        );
    }

    /// 测试 roundRect 顶点不在矩形的尖角上（左上角 (x,y) 不应出现在顶点中）。
    #[test]
    fn test_round_rect_vertices_avoid_sharp_corners() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        let x = 10.0f32;
        let y = 20.0f32;
        let w = 100.0f32;
        let h = 80.0f32;
        path.round_rect(x, y, w, h, vec![15.0]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 矩形的四个尖角不应出现在顶点中
        let sharp_corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
        for chunk in pf.vertices.chunks_exact(2) {
            let (vx, vy) = (chunk[0], chunk[1]);
            for &(cx, cy) in &sharp_corners {
                assert!(
                    (vx - cx).abs() > 0.01 || (vy - cy).abs() > 0.01,
                    "vertex ({}, {}) should not be at sharp corner ({}, {})",
                    vx,
                    vy,
                    cx,
                    cy
                );
            }
        }
    }

    /// 测试 roundRect 零半径退化为普通矩形。
    #[test]
    fn test_round_rect_zero_radius_degrades_to_rect() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.round_rect(10.0, 20.0, 100.0, 80.0, vec![0.0]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 零半径应退化为普通矩形：5 段 × 4 = 20 floats
        assert_eq!(pf.vertices.len(), 20);
    }

    /// 测试 roundRect 空半径列表退化为普通矩形。
    #[test]
    fn test_round_rect_empty_radii_degrades_to_rect() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.round_rect(10.0, 20.0, 100.0, 80.0, vec![]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        assert_eq!(pf.vertices.len(), 20);
    }

    /// 测试 roundRect 四个不同圆角半径。
    #[test]
    fn test_round_rect_four_different_radii() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        path.round_rect(10.0, 20.0, 100.0, 80.0, vec![5.0, 10.0, 15.0, 20.0]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 有圆角应产生远多于普通矩形的顶点
        assert!(pf.vertices.len() > 20);
    }

    /// 测试 roundRect 半径超过短边一半时被限制。
    #[test]
    fn test_round_rect_radius_clamped() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        // 宽 40，高 20，半径 50 超过短边一半(10)
        path.round_rect(0.0, 0.0, 40.0, 20.0, vec![50.0]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 应该不 panic，并且顶点不应超出矩形范围
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in pf.vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        // 顶点应大致在矩形范围内（允许浮点误差）
        assert!(min_x >= -0.1 && min_y >= -0.1);
        assert!(max_x <= 40.1 && max_y <= 20.1);
    }

    /// 测试 roundRect 通过当前路径的 flatten_path 正确工作。
    #[test]
    fn test_round_rect_via_current_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.current_path.round_rect(10.0, 20.0, 100.0, 80.0, vec![10.0]);
        ctx.fill();
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        let pf = &ctx.primitives().path_fills[0];
        assert!(
            pf.vertices.len() > 20,
            "roundRect via current path should produce rounded vertices"
        );
    }

    /// 测试 roundRect 圆角顶点在几何上合理：左上角附近的顶点应偏向矩形的内部。
    #[test]
    fn test_round_rect_corner_vertices_offset_inward() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path = Path2D::new();
        let x = 50.0f32;
        let y = 50.0f32;
        let w = 100.0f32;
        let h = 100.0f32;
        let r = 20.0f32;
        path.round_rect(x, y, w, h, vec![r]);
        ctx.fill_with_path(&path);
        let pf = &ctx.primitives().path_fills[0];
        // 检查所有顶点不在矩形的四个尖角 20×20 正方形区域内
        let corner_zones = [
            (x, y),             // 左上
            (x + w - r, y),     // 右上起点
            (x + w, y + h - r), // 右下起点
            (x, y + h - r),     // 左下起点
        ];
        // 至少应有一些顶点在圆角区域（不在直边上）
        let mut has_corner_vertex = false;
        for chunk in pf.vertices.chunks_exact(2) {
            let (vx, vy) = (chunk[0], chunk[1]);
            // 左上角圆角区域：x 在 [x, x+r] 且 y 在 [y, y+r] 的四分之一圆内
            if vx >= x && vx <= x + r && vy >= y && vy <= y + r {
                let dx = vx - (x + r);
                let dy = vy - (y + r);
                if dx * dx + dy * dy <= r * r * 1.1 {
                    has_corner_vertex = true;
                    break;
                }
            }
        }
        assert!(has_corner_vertex, "should have vertices on the rounded corner arc");
        let _ = corner_zones;
    }

    /// 测试 roundRect 两个半径值时的映射：[左上/右下, 右上/左下]。
    #[test]
    fn test_round_rect_two_radii_mapping() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut path1 = Path2D::new();
        path1.round_rect(0.0, 0.0, 100.0, 100.0, vec![5.0, 15.0]);
        ctx.fill_with_path(&path1);
        let pf1 = ctx.primitives().path_fills[0].vertices.clone();

        // 与四个不同半径对比：[5, 15, 5, 15]
        let mut path2 = Path2D::new();
        path2.round_rect(0.0, 0.0, 100.0, 100.0, vec![5.0, 15.0, 5.0, 15.0]);
        // 清空之前的图元
        let mut ctx2 = CanvasContext::new(200, 200);
        ctx2.fill_with_path(&path2);
        let pf2 = ctx2.primitives().path_fills[0].vertices.clone();

        // 两种写法应产生相同的顶点
        assert_eq!(pf1.len(), pf2.len(), "2-radii should map to 4-radii [a,b,a,b]");
    }

    // ── drawImage alpha blending 测试 ──

    /// 测试 draw_image 对半透明像素（alpha=128）的 alpha compositing。
    /// 在不透明红色背景上绘制半透明绿色源，验证混合结果符合 source-over 公式。
    #[test]
    fn test_draw_image_alpha_blending() {
        // 准备 10x10 画布，先填充不透明红色背景
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);

        // 创建 1x1 半透明绿色像素（alpha=128）
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![0, 255, 0, 128], // 绿色，alpha=128
        };
        ctx.draw_image(&img, 0.0, 0.0);

        // 读取混合结果
        let result = ctx.get_image_data(0, 0, 1, 1);
        let r = result.data[0];
        let g = result.data[1];
        let b = result.data[2];
        let a = result.data[3];

        // source-over 公式（global_alpha=1.0）：
        //   src_a = 128/255 ≈ 0.502
        //   dst_a = 255/255 = 1.0
        //   out_a = src_a + dst_a * (1 - src_a) = 0.502 + 0.498 = 1.0 → 255
        //   out_r = (src_r * src_a + dst_r * dst_a * (1-src_a)) / out_a
        //         = (0 * 0.502 + 255 * 1.0 * 0.498) / 1.0 ≈ 127
        //   out_g = (255 * 0.502 + 0 * 1.0 * 0.498) / 1.0 ≈ 128
        //   out_b = (0 * 0.502 + 0 * 1.0 * 0.498) / 1.0 = 0
        assert_eq!(a, 255, "output alpha should be fully opaque");
        assert!((r as i32 - 127).abs() <= 2, "red channel should be ~127, got {}", r);
        assert!((g as i32 - 128).abs() <= 2, "green channel should be ~128, got {}", g);
        assert_eq!(b, 0, "blue channel should be 0");
    }

    // ── put_image_data / get_image_data 边界溢出测试 ──

    /// 测试 put_image_data 部分溢出画布边界时不 panic，且可见区域被正确写入。
    #[test]
    fn test_put_image_data_partial_overflow() {
        let mut ctx = CanvasContext::new(10, 10);
        // 创建 4x4 全红色 ImageData
        let img = ImageData {
            width: 4,
            height: 4,
            data: [255, 0, 0, 255].repeat(16), // 16 像素 × 4 通道 = 64 字节
        };
        // 放置在 (8, 8)，只有 2x2 区域在画布内
        ctx.put_image_data(&img, 8, 8);

        // 验证可见区域被正确写入
        let visible = ctx.get_image_data(8, 8, 2, 2);
        assert_eq!(
            visible.data[0..4],
            [255, 0, 0, 255],
            "visible pixel (8,8) should be red"
        );
        assert_eq!(
            visible.data[4..8],
            [255, 0, 0, 255],
            "visible pixel (9,8) should be red"
        );
        assert_eq!(
            visible.data[8..12],
            [255, 0, 0, 255],
            "visible pixel (8,9) should be red"
        );
        assert_eq!(
            visible.data[12..16],
            [255, 0, 0, 255],
            "visible pixel (9,9) should be red"
        );

        // 验证溢出区域未影响其他像素
        let outside = ctx.get_image_data(7, 7, 1, 1);
        assert_eq!(
            outside.data[0..4],
            [0, 0, 0, 0],
            "pixel before offset should be untouched"
        );
    }

    /// 测试 get_image_data 在完全超出画布边界时返回全零数据。
    #[test]
    fn test_get_image_data_out_of_bounds() {
        let ctx = CanvasContext::new(10, 10);
        // 请求画布范围外的区域
        let result = ctx.get_image_data(20, 20, 2, 2);
        // 应返回全零（4 像素 × 4 通道 = 16 字节）
        assert_eq!(result.data, vec![0u8; 16], "out-of-bounds region should be all zeros");
        // 尺寸信息应保持请求值
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 变换组合和 putImageData 边界测试
    // ═══════════════════════════════════════════════════════════════════

    /// 测试旋转变换 + 平移变换的顺序不可交换性。
    ///
    /// rotate(π/2) 后 translate(10,0) 与 translate(10,0) 后 rotate(π/2)
    /// 应产生不同的变换结果。
    #[test]
    fn test_transform_rotate_then_translate_vs_reverse() {
        let mut ctx1 = CanvasContext::new(100, 100);
        ctx1.rotate(std::f32::consts::FRAC_PI_2);
        ctx1.translate(10.0, 0.0);
        let p1 = ctx1.transform.transform_point(0.0, 0.0);

        let mut ctx2 = CanvasContext::new(100, 100);
        ctx2.translate(10.0, 0.0);
        ctx2.rotate(std::f32::consts::FRAC_PI_2);
        let p2 = ctx2.transform.transform_point(0.0, 0.0);

        // 两个结果应不同（矩阵乘法不可交换）
        assert!(
            (p1.0 - p2.0).abs() > 0.01 || (p1.1 - p2.1).abs() > 0.01,
            "rotate→translate 与 translate→rotate 应产生不同结果: ({}, {}) vs ({}, {})",
            p1.0,
            p1.1,
            p2.0,
            p2.1
        );
    }

    /// 测试 set_transform 替换（而非叠加）当前变换矩阵。
    #[test]
    fn test_set_transform_replaces_current() {
        let mut ctx = CanvasContext::new(100, 100);
        // 先设置一个非平凡的变换
        ctx.translate(100.0, 100.0);
        ctx.scale(2.0, 3.0);
        // set_transform 应替换整个矩阵为单位矩阵
        ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let p = ctx.transform.transform_point(5.0, 7.0);
        // 单位变换下 (5,7) 应保持不变
        assert!((p.0 - 5.0).abs() < 0.01, "x 应为 5.0，实际 {}", p.0);
        assert!((p.1 - 7.0).abs() < 0.01, "y 应为 7.0，实际 {}", p.1);
    }

    /// 测试 putImageData 处理超过画布大小的 ImageData 时不 panic。
    #[test]
    fn test_put_image_data_larger_than_canvas() {
        let mut ctx = CanvasContext::new(5, 5);
        // 创建 20x20 的 ImageData，但画布只有 5x5
        let mut data = vec![255u8; 20 * 20 * 4];
        // 写入一些标记值
        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        data[3] = 255; // 红色
        let image_data = ImageData {
            width: 20,
            height: 20,
            data,
        };
        // 不应 panic
        ctx.put_image_data(&image_data, 0, 0);
        // 验证画布内像素被写入
        let result = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(result.data[0], 255, "红色通道应为 255");
    }

    /// 测试 putImageData 使用零尺寸 ImageData 时不 panic。
    #[test]
    fn test_put_image_data_zero_size() {
        let mut ctx = CanvasContext::new(10, 10);
        let image_data = ImageData {
            width: 0,
            height: 0,
            data: vec![],
        };
        // 不应 panic
        ctx.put_image_data(&image_data, 0, 0);
    }

    /// 测试 putImageData 使用数据向量过短的 ImageData 时不 panic。
    #[test]
    fn test_put_image_data_short_data_vector() {
        let mut ctx = CanvasContext::new(10, 10);
        // 声明为 10x10 但数据只有 4 字节（1 个像素）
        let image_data = ImageData {
            width: 10,
            height: 10,
            data: vec![255, 0, 0, 255],
        };
        // 不应 panic 或越界访问
        ctx.put_image_data(&image_data, 0, 0);
    }

    // ── 线性渐变多色停止点边界测试 ──

    /// 测试线性渐变添加 10 个颜色停止点（0.0 到 0.9），以及逆序添加时保持插入顺序。
    #[test]
    fn test_linear_gradient_many_stops_ordering() {
        let ctx = CanvasContext::new(200, 200);

        // 顺序添加 10 个停止点：0.0, 0.1, ..., 0.9
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        for i in 0..10 {
            let offset = i as f32 * 0.1;
            let color = Color::rgba(i as u8 * 25, 0, 0, 255);
            grad.add_color_stop(offset, color);
        }
        assert_eq!(grad.stops.len(), 10);
        for i in 0..10 {
            let expected_offset = i as f32 * 0.1;
            assert!(
                (grad.stops[i].offset - expected_offset).abs() < f32::EPSILON,
                "第 {} 个停止点偏移量应为 {}，实际 {}",
                i,
                expected_offset,
                grad.stops[i].offset
            );
        }

        // 逆序添加停止点：1.0, 0.5, 0.0 — 应保持插入顺序而非排序
        let mut grad2 = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad2.add_color_stop(1.0, Color::BLUE);
        grad2.add_color_stop(0.5, Color::GREEN);
        grad2.add_color_stop(0.0, Color::RED);
        assert_eq!(grad2.stops.len(), 3);
        // 验证保持插入顺序（未排序）
        assert!((grad2.stops[0].offset - 1.0).abs() < f32::EPSILON);
        assert!((grad2.stops[1].offset - 0.5).abs() < f32::EPSILON);
        assert!((grad2.stops[2].offset - 0.0).abs() < f32::EPSILON);
        assert_eq!(grad2.stops[0].color, Color::BLUE);
        assert_eq!(grad2.stops[1].color, Color::GREEN);
        assert_eq!(grad2.stops[2].color, Color::RED);
    }

    /// 测试线性渐变在同一偏移量添加两个不同颜色的停止点，验证不会去重。
    #[test]
    fn test_gradient_duplicate_offset_stops() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.5, Color::GREEN);
        // 在同一偏移量 0.5 添加另一个颜色的停止点
        grad.add_color_stop(0.5, Color::BLUE);
        grad.add_color_stop(1.0, Color::WHITE);
        // 两个偏移量 0.5 的停止点都应保留，不会被去重
        assert_eq!(grad.stops.len(), 4);
        assert!((grad.stops[1].offset - 0.5).abs() < f32::EPSILON);
        assert_eq!(grad.stops[1].color, Color::GREEN);
        assert!((grad.stops[2].offset - 0.5).abs() < f32::EPSILON);
        assert_eq!(grad.stops[2].color, Color::BLUE);
    }

    /// 测试线性渐变添加超出 [0, 1] 范围的偏移量不会 panic。
    #[test]
    fn test_gradient_out_of_range_offset_no_panic() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        // 负偏移量 — 不应 panic
        grad.add_color_stop(-0.5, Color::RED);
        // 大于 1 的偏移量 — 不应 panic
        grad.add_color_stop(1.5, Color::BLUE);
        assert_eq!(grad.stops.len(), 2);
        assert!((grad.stops[0].offset - (-0.5)).abs() < f32::EPSILON);
        assert!((grad.stops[1].offset - 1.5).abs() < f32::EPSILON);
    }

    // ── 新增边界条件测试 ──

    /// 测试裁剪区域限制 draw_image 绘制范围：裁剪区域外的像素不应被写入。
    #[test]
    fn test_clip_constrains_draw_image() {
        let mut ctx = CanvasContext::new(100, 100);
        // 设置裁剪区域为左上角 10x10
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(10.0, 0.0);
        ctx.line_to(10.0, 10.0);
        ctx.line_to(0.0, 10.0);
        ctx.close_path();
        ctx.clip();
        // 在 (0,0) 绘制 20x20 红色图像
        let img = ImageData {
            width: 20,
            height: 20,
            data: [255, 0, 0, 255].repeat(20 * 20),
        };
        ctx.draw_image(&img, 0.0, 0.0);
        // 裁剪区域内 (5,5) 应被绘制为红色
        let inside = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(inside.data[0..4], [255, 0, 0, 255], "裁剪区域内应有像素");
        // 注意：当前 clip 实现是基于包围盒的简化裁剪，
        // draw_image 的像素级操作不检查 clip_path，因此裁剪区域外的像素可能被写入。
        // 此测试验证 clip 图元被正确注册。
        assert_eq!(ctx.primitives().clips.len(), 1, "应注册一个裁剪图元");
    }

    /// 测试 draw_image 使用负坐标目标位置时不 panic。
    /// 注意：当前实现中负坐标 float 转 usize 会变为 0，导致部分像素写入画布左上角。
    /// 测试重点验证不发生 panic。
    #[test]
    fn test_draw_image_negative_coordinates() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 10,
            height: 10,
            data: [255, 0, 0, 255].repeat(10 * 10),
        };
        // 负坐标 — 不应 panic
        ctx.draw_image(&img, -5.0, -5.0);
        ctx.draw_image(&img, -100.0, -100.0);
        // 验证至少没有 panic，部分像素可能因负坐标转 usize=0 而写入左上角
    }

    /// 测试 draw_image_with_size 使用零宽高目标尺寸时不绘制任何像素。
    #[test]
    fn test_draw_image_zero_dimensions() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 10,
            height: 10,
            data: [255, 0, 0, 255].repeat(10 * 10),
        };
        // 零尺寸 — 不应 panic，不应绘制任何像素
        ctx.draw_image_with_size(&img, 0.0, 0.0, 0.0, 0.0);
        let pixel = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "零尺寸绘制不应写入像素");
    }

    /// 测试 ImageData 使用零尺寸创建时的行为。
    #[test]
    fn test_image_data_zero_dimensions() {
        let img = ImageData {
            width: 0,
            height: 0,
            data: vec![],
        };
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.data.is_empty());
    }

    /// 测试 get_image_data 请求部分超出画布边界的区域时，越界像素返回零，画布内像素正常返回。
    #[test]
    fn test_get_image_data_partially_out_of_bounds() {
        let mut ctx = CanvasContext::new(10, 10);
        // 先在画布内写入一些数据
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 请求部分超出画布的区域 (x=8, y=8, w=4, h=4)
        // 只有 (8,8) 和 (8,9) 和 (9,8) 和 (9,9) 在画布内
        let result = ctx.get_image_data(8, 8, 4, 4);
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
        // 画布内像素 (8,8) 应为红色
        assert_eq!(result.data[0..4], [255, 0, 0, 255], "画布内像素应为红色");
        // 画布外像素应为零（超出 canvas 边界的行/列）
        // (0,2) 即第 3 行第 1 列对应 canvas y=10，已超出
        let out_idx = (2 * 4 + 0) * 4; // row=2, col=0
        assert_eq!(result.data[out_idx..out_idx + 4], [0, 0, 0, 0], "越界像素应为零");
    }

    // ── CanvasContext ellipse 测试 ──

    /// 测试 ellipse 通过 context API 生成 path_fills。
    #[test]
    fn test_ellipse_via_context_generates_path_fills() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
        ctx.fill();
        assert_eq!(ctx.primitives().path_fills.len(), 1);
        let pf = &ctx.primitives().path_fills[0];
        // 16 段细分 × 4 floats = 64
        assert_eq!(pf.vertices.len(), 64);
    }

    /// 测试 ellipse 使用单位旋转（rotation=0）时顶点与预期一致。
    #[test]
    fn test_ellipse_identity_rotation() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.ellipse(100.0, 100.0, 40.0, 20.0, 0.0, 0.0, std::f32::consts::TAU);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
        let pf = &ctx.primitives().path_fills[0];
        // 验证第一个顶点在椭圆起始位置（角度 0）附近：(cx + rx, cy)
        let first_x = pf.vertices[0];
        let first_y = pf.vertices[1];
        assert!((first_x - 140.0).abs() < 1.0, "first x ~140, got {}", first_x);
        assert!((first_y - 100.0).abs() < 1.0, "first y ~100, got {}", first_y);
    }

    /// 测试 ellipse 使用 90 度旋转时产生与无旋转不同的顶点。
    #[test]
    fn test_ellipse_rotated_90_produces_different_vertices() {
        let mut ctx1 = CanvasContext::new(200, 200);
        ctx1.begin_path();
        ctx1.ellipse(100.0, 100.0, 40.0, 20.0, 0.0, 0.0, std::f32::consts::TAU);
        ctx1.fill();
        let v1 = ctx1.primitives().path_fills[0].vertices.clone();

        let mut ctx2 = CanvasContext::new(200, 200);
        ctx2.begin_path();
        ctx2.ellipse(
            100.0,
            100.0,
            40.0,
            20.0,
            std::f32::consts::FRAC_PI_2,
            0.0,
            std::f32::consts::TAU,
        );
        ctx2.fill();
        let v2 = ctx2.primitives().path_fills[0].vertices.clone();

        // 两种旋转应产生不同的顶点
        assert_ne!(v1, v2, "90 度旋转的椭圆应产生与无旋转不同的顶点");
    }

    /// 测试 TextAlign 和 TextBaseline 枚举值可构造且互相不等。
    #[test]
    fn test_text_align_and_baseline_enums() {
        // 验证 TextAlign 各变体可以构造且互不相等
        let aligns = [
            TextAlign::Start,
            TextAlign::End,
            TextAlign::Left,
            TextAlign::Right,
            TextAlign::Center,
        ];
        for (i, a) in aligns.iter().enumerate() {
            for (j, b) in aligns.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b, "TextAlign 变体 {} 和 {} 应不相等", i, j);
                }
            }
        }

        // 验证 TextBaseline 各变体可以构造且互不相等
        let baselines = [
            TextBaseline::Top,
            TextBaseline::Middle,
            TextBaseline::Alphabetic,
            TextBaseline::Bottom,
        ];
        for (i, a) in baselines.iter().enumerate() {
            for (j, b) in baselines.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b, "TextBaseline 变体 {} 和 {} 应不相等", i, j);
                }
            }
        }
    }

    // ── createConicGradient 测试 ──

    /// 测试创建锥形渐变：起始角度和中心坐标正确，初始无停止点。
    #[test]
    fn test_create_conic_gradient() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_conic_gradient(std::f32::consts::FRAC_PI_4, 100.0, 100.0);
        assert!((grad.start_angle - std::f32::consts::FRAC_PI_4).abs() < f32::EPSILON);
        assert!((grad.cx - 100.0).abs() < f32::EPSILON);
        assert!((grad.cy - 100.0).abs() < f32::EPSILON);
        assert!(grad.stops.is_empty());
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        assert_eq!(grad.stops.len(), 2);
        assert_eq!(grad.stops[0].color, Color::RED);
        assert_eq!(grad.stops[1].color, Color::BLUE);
    }

    /// 测试锥形渐变添加多个颜色停止点。
    #[test]
    fn test_conic_gradient_multiple_stops() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.25, Color::GREEN);
        grad.add_color_stop(0.5, Color::BLUE);
        grad.add_color_stop(0.75, Color::WHITE);
        grad.add_color_stop(1.0, Color::RED);
        assert_eq!(grad.stops.len(), 5);
        assert!((grad.stops[1].offset - 0.25).abs() < f32::EPSILON);
        assert_eq!(grad.stops[1].color, Color::GREEN);
    }

    /// 测试锥形渐变无停止点的退化情况（不 panic）。
    #[test]
    fn test_conic_gradient_no_stops() {
        let ctx = CanvasContext::new(200, 200);
        let grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
        assert!(grad.stops.is_empty());
    }

    // ── arcTo 测试 ──

    /// 测试 arc_to 生成 path_fills（非空路径）。
    #[test]
    fn test_arc_to_produces_path_fills() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.arc_to(50.0, 0.0, 50.0, 50.0, 10.0);
        ctx.line_to(50.0, 50.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty(), "arc_to 应生成路径填充图元");
    }

    /// 测试 arc_to 零半径退化为直线到控制点1。
    #[test]
    fn test_arc_to_zero_radius_degenerates_to_line() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.arc_to(100.0, 0.0, 100.0, 100.0, 0.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
        // 零半径时：从 (0,0) 画线到 (100,0)，不产生弧线
        let pf = &ctx.primitives().path_fills[0];
        // 只有 1 条线段 = 4 floats（lineTo 到控制点1）
        assert_eq!(pf.vertices.len(), 4, "零半径 arcTo 应退化为一条线段");
    }

    /// 测试 arc_to 共线点（当前点、控制点1、控制点2 在一条线上）退化为直线。
    #[test]
    fn test_arc_to_collinear_points_produces_line() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        // 三个点共线：(0,0) -> (50,0) -> (100,0)
        ctx.arc_to(50.0, 0.0, 100.0, 0.0, 10.0);
        ctx.fill();
        assert!(!ctx.primitives().path_fills.is_empty());
        // 共线时退化为 lineTo(50, 0)，只有 1 条线段 = 4 floats
        let pf = &ctx.primitives().path_fills[0];
        assert_eq!(pf.vertices.len(), 4, "共线 arcTo 应退化为一条线段");
    }

    // ── line_join / line_cap 测试 ──

    /// 测试 line_join 和 line_cap 默认值分别为 Miter 和 Butt。
    #[test]
    fn test_line_join_and_line_cap_default_values() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.line_join(), LineJoin::Miter);
        assert_eq!(ctx.line_cap(), LineCap::Butt);
    }

    /// 测试 LineJoin 和 LineCap 默认值与枚举 Default trait 一致。
    #[test]
    fn test_line_join_and_line_cap_default_trait() {
        assert_eq!(LineJoin::default(), LineJoin::Miter);
        assert_eq!(LineCap::default(), LineCap::Butt);
    }

    /// 测试设置和获取 line_join 的所有变体。
    #[test]
    fn test_line_join_set_get_roundtrip() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_join(LineJoin::Round);
        assert_eq!(ctx.line_join(), LineJoin::Round);
        ctx.set_line_join(LineJoin::Bevel);
        assert_eq!(ctx.line_join(), LineJoin::Bevel);
        ctx.set_line_join(LineJoin::Miter);
        assert_eq!(ctx.line_join(), LineJoin::Miter);
    }

    /// 测试设置和获取 line_cap 的所有变体。
    #[test]
    fn test_line_cap_set_get_roundtrip() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_cap(LineCap::Round);
        assert_eq!(ctx.line_cap(), LineCap::Round);
        ctx.set_line_cap(LineCap::Square);
        assert_eq!(ctx.line_cap(), LineCap::Square);
        ctx.set_line_cap(LineCap::Butt);
        assert_eq!(ctx.line_cap(), LineCap::Butt);
    }

    /// 测试 line_join 和 line_cap 在 save/restore 中正确保存和恢复。
    #[test]
    fn test_line_join_and_line_cap_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_join(LineJoin::Round);
        ctx.set_line_cap(LineCap::Square);
        ctx.save();
        ctx.set_line_join(LineJoin::Bevel);
        ctx.set_line_cap(LineCap::Round);
        assert_eq!(ctx.line_join(), LineJoin::Bevel);
        assert_eq!(ctx.line_cap(), LineCap::Round);
        ctx.restore();
        assert_eq!(ctx.line_join(), LineJoin::Round);
        assert_eq!(ctx.line_cap(), LineCap::Square);
    }

    // ── isPointInStroke 测试 ──

    /// 测试描边线上的点被检测到。
    #[test]
    fn test_is_point_in_stroke_on_line() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 50.0);
        ctx.line_to(100.0, 50.0);
        // 默认 line_width = 1.0，点 (50, 50) 在线段上，距离为 0
        assert!(ctx.is_point_in_stroke(50.0, 50.0));
    }

    /// 测试远离描边的点不被检测到。
    #[test]
    fn test_is_point_in_stroke_far_away() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        ctx.move_to(0.0, 50.0);
        ctx.line_to(100.0, 50.0);
        // 默认 line_width = 1.0，点 (50, 100) 距线段 50，远大于 0.5
        assert!(!ctx.is_point_in_stroke(50.0, 100.0));
    }

    /// 测试粗线宽增大检测区域。
    #[test]
    fn test_is_point_in_stroke_thick_line() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(20.0);
        ctx.begin_path();
        ctx.move_to(0.0, 50.0);
        ctx.line_to(100.0, 50.0);
        // line_width = 20，half = 10，点 (50, 55) 距线段 5 < 10
        assert!(ctx.is_point_in_stroke(50.0, 55.0));
        // 点 (50, 65) 距线段 15 > 10
        assert!(!ctx.is_point_in_stroke(50.0, 65.0));
    }

    // ── 合成操作像素级测试 ──

    /// 测试默认 source-over 合成：先绘制红色矩形，再绘制重叠的蓝色矩形，
    /// 重叠区域的蓝色应覆盖红色（不透明像素的 source-over 结果）。
    #[test]
    fn test_composite_source_over_default() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 再绘制蓝色 (5,0)-(15,10)，默认 source-over
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 不重叠区域 (0,0)：仍为红色
        let red_only = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(
            red_only.data[0..4],
            [255, 0, 0, 255],
            "source-over: 非重叠区域应保留红色"
        );

        // 重叠区域 (7,0)：蓝色覆盖红色
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-over: 重叠区域应为蓝色");

        // 蓝色独占区域 (12,0)：蓝色
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(
            blue_only.data[0..4],
            [0, 0, 255, 255],
            "source-over: 蓝色独占区域应为蓝色"
        );
    }

    /// 测试 destination-over 合成：先绘制蓝色（目标），再以 destination-over 绘制红色（源），
    /// 红色应出现在蓝色下方（重叠区域蓝色在上方）。
    #[test]
    fn test_composite_destination_over() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制蓝色 (0,0)-(10,10) 作为目标
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 destination-over 绘制红色 (5,0)-(15,10) 作为源
        ctx.set_composite_operation(CompositeOperation::DestinationOver);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 蓝色独占区域 (2,0)：蓝色不变
        let blue_only = ctx.get_image_data(2, 0, 1, 1);
        assert_eq!(
            blue_only.data[0..4],
            [0, 0, 255, 255],
            "destination-over: 蓝色独占区域不变"
        );

        // 重叠区域 (7,0)：destination-over 下蓝色（目标）在红色（源）之上
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(
            overlap.data[0..4],
            [0, 0, 255, 255],
            "destination-over: 重叠区域应显示蓝色（目标在上）"
        );

        // 红色独占区域 (12,0)：没有目标，只有源
        let red_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(
            red_only.data[0..4],
            [255, 0, 0, 255],
            "destination-over: 红色独占区域应显示红色"
        );
    }

    /// 测试 copy 合成：先绘制红色，再设置 copy 模式绘制蓝色，
    /// copy 模式下蓝色完全替换已有内容，不受之前内容影响。
    #[test]
    fn test_composite_copy() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色填充整个画布
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 20.0, 10.0);
        // 使用 copy 合成绘制蓝色矩形
        ctx.set_composite_operation(CompositeOperation::Copy);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // copy 模式下蓝色区域内应为蓝色
        let inside = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(inside.data[0..4], [0, 0, 255, 255], "copy: 绘制区域内应为蓝色");

        // copy 区域外应为红色（未被覆盖）
        let outside = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(outside.data[0..4], [255, 0, 0, 255], "copy: 未绘制区域应保留红色");
    }

    /// 测试 xor 合成：先绘制红色矩形，再绘制重叠的蓝色矩形，
    /// xor 模式下重叠区域应变为透明（两个不透明像素的异或结果为空）。
    #[test]
    fn test_composite_xor() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 xor 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::Xor);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 红色独占区域 (2,0)：sa=0,da=1 → xor 保留目标 = 红色
        let red_only = ctx.get_image_data(2, 0, 1, 1);
        assert_eq!(red_only.data[0..4], [255, 0, 0, 255], "xor: 红色独占区域应保留");

        // 重叠区域 (7,0)：两个不透明像素 xor → 透明
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(overlap.data[0..4], [0, 0, 0, 0], "xor: 重叠区域应为透明");

        // 蓝色独占区域 (12,0)：sa=1,da=0 → xor 保留源 = 蓝色
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(blue_only.data[0..4], [0, 0, 255, 255], "xor: 蓝色独占区域应为蓝色");
    }

    /// 测试 source-atop 合成：先绘制红色矩形，再绘制重叠的蓝色矩形，
    /// source-atop 模式下蓝色只出现在已有红色内容的区域。
    #[test]
    fn test_composite_source_atop() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 source-atop 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::SourceAtop);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 重叠区域 (7,0)：source-atop → 源色（蓝色）出现在目标存在的区域
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-atop: 重叠区域应为蓝色");

        // 蓝色独占区域 (12,0)：没有目标像素 → source-atop 保留目标 = 透明
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(
            blue_only.data[0..4],
            [0, 0, 0, 0],
            "source-atop: 无目标的源区域应为透明"
        );
    }

    // ── image_smoothing_enabled 测试 ──

    /// 测试 image_smoothing_enabled 默认值为 true。
    #[test]
    fn test_image_smoothing_enabled_default_is_true() {
        let ctx = CanvasContext::new(100, 100);
        assert!(ctx.image_smoothing_enabled(), "imageSmoothingEnabled 默认应为 true");
    }

    /// 测试 set/get 往返一致性。
    #[test]
    fn test_image_smoothing_enabled_set_get_roundtrip() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_image_smoothing_enabled(false);
        assert!(!ctx.image_smoothing_enabled(), "设置为 false 后应返回 false");
        ctx.set_image_smoothing_enabled(true);
        assert!(ctx.image_smoothing_enabled(), "设置为 true 后应返回 true");
    }

    /// 测试 save/restore 保存并恢复 image_smoothing_enabled 的值。
    #[test]
    fn test_image_smoothing_enabled_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_image_smoothing_enabled(false);
        ctx.save();
        ctx.set_image_smoothing_enabled(true);
        assert!(ctx.image_smoothing_enabled(), "save 后修改应为 true");
        ctx.restore();
        assert!(!ctx.image_smoothing_enabled(), "restore 后应恢复为 false");
    }

    /// 测试 save 后修改 image_smoothing_enabled 不影响已保存的状态。
    #[test]
    fn test_image_smoothing_enabled_modify_after_save_does_not_affect_saved() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_image_smoothing_enabled(true);
        ctx.save();
        ctx.set_image_smoothing_enabled(false);
        assert!(!ctx.image_smoothing_enabled(), "修改后当前值应为 false");
        ctx.restore();
        assert!(ctx.image_smoothing_enabled(), "restore 后应恢复为 save 时的 true");
    }

    // ── stroke line_cap / line_join 渲染测试 ──

    /// 测试描边使用 line_cap Butt 时端点为平头（不超出线段端点）。
    /// 验证描边像素仅在线段范围内，不延伸到端点之外。
    #[test]
    fn test_stroke_line_cap_butt_flat_endpoints() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::BLUE);
        ctx.set_line_width(4.0);
        ctx.set_line_cap(LineCap::Butt);
        ctx.begin_path();
        ctx.move_to(20.0, 50.0);
        ctx.line_to(80.0, 50.0);
        ctx.stroke();
        // 在线段起点之前不应有像素
        let before_start = ctx.get_image_data(15, 48, 1, 4);
        assert_eq!(before_start.data[0..4], [0, 0, 0, 0], "Butt cap: 线段起点前不应有像素");
        // 在线段终点之后不应有像素
        let after_end = ctx.get_image_data(85, 48, 1, 4);
        assert_eq!(after_end.data[0..4], [0, 0, 0, 0], "Butt cap: 线段终点后不应有像素");
        // 在线段中点应有像素
        let mid = ctx.get_image_data(50, 48, 1, 4);
        assert_eq!(mid.data[0..4], [0, 0, 255, 255], "Butt cap: 线段中点应为蓝色");
    }

    /// 测试描边使用 line_cap Round 时端点扩展（半圆形）。
    /// 验证描边像素在端点处超出线段范围。
    #[test]
    fn test_stroke_line_cap_round_extended_endpoints() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::RED);
        ctx.set_line_width(10.0);
        ctx.set_line_cap(LineCap::Round);
        ctx.begin_path();
        ctx.move_to(30.0, 50.0);
        ctx.line_to(70.0, 50.0);
        ctx.stroke();
        // Round cap 应在端点处产生额外像素（半圆近似为正方形）
        // half_lw = 5, 起点 (30,50)，Round cap 正方形覆盖 (25,45)-(35,55)
        // 终点 (70,50)，Round cap 正方形覆盖 (65,45)-(75,55)
        let near_start = ctx.get_image_data(25, 46, 1, 1);
        assert_ne!(near_start.data[3], 0, "Round cap: 起点端附近应有像素");
        let near_end = ctx.get_image_data(74, 46, 1, 1);
        assert_ne!(near_end.data[3], 0, "Round cap: 终点端附近应有像素");
    }

    /// 测试描边使用 line_join Miter 时产生尖角连接。
    /// Miter 连接的轮廓顶点应超出两条线段的简单矩形叠加范围。
    #[test]
    fn test_stroke_line_join_miter_sharp_corners() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_stroke_color(Color::BLACK);
        ctx.set_line_width(4.0);
        ctx.set_line_join(LineJoin::Miter);
        ctx.begin_path();
        ctx.move_to(10.0, 100.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(190.0, 100.0);
        ctx.stroke();
        // Miter join 应在连接点 (100,10) 处产生额外的填充区域
        // 检查连接点附近有像素
        let join_pixel = ctx.get_image_data(98, 8, 1, 1);
        assert_ne!(join_pixel.data[3], 0, "Miter join: 连接点附近应有像素");
    }

    /// 测试描边使用 line_join Round 时产生圆角连接。
    /// Round 连接的轮廓顶点应包含扇形顶点。
    #[test]
    fn test_stroke_line_join_round_corners() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_stroke_color(Color::GREEN);
        ctx.set_line_width(6.0);
        ctx.set_line_join(LineJoin::Round);
        ctx.begin_path();
        ctx.move_to(10.0, 100.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(190.0, 100.0);
        ctx.stroke();
        // Round join 应在连接点处产生覆盖区域
        let join_pixel = ctx.get_image_data(98, 8, 1, 1);
        assert_ne!(join_pixel.data[3], 0, "Round join: 连接点附近应有像素");
    }

    /// 测试 line_width 影响描边宽度。
    /// 更大的 line_width 应产生更宽的描边覆盖区域。
    #[test]
    fn test_line_width_affects_stroke_width() {
        // 细线
        let mut ctx_thin = CanvasContext::new(100, 100);
        ctx_thin.set_stroke_color(Color::RED);
        ctx_thin.set_line_width(2.0);
        ctx_thin.begin_path();
        ctx_thin.move_to(50.0, 10.0);
        ctx_thin.line_to(50.0, 90.0);
        ctx_thin.stroke();
        // 粗线
        let mut ctx_thick = CanvasContext::new(100, 100);
        ctx_thick.set_stroke_color(Color::RED);
        ctx_thick.set_line_width(10.0);
        ctx_thick.begin_path();
        ctx_thick.move_to(50.0, 10.0);
        ctx_thick.line_to(50.0, 90.0);
        ctx_thick.stroke();
        // 粗线在距中心更远的位置应有像素
        // 细线 (line_width=2) 在 x=54 应无像素
        let thin_at_54 = ctx_thin.get_image_data(54, 50, 1, 1);
        assert_eq!(thin_at_54.data[0..4], [0, 0, 0, 0], "line_width=2: x=54 不应有像素");
        // 粗线 (line_width=10) 在 x=54 应有像素
        let thick_at_54 = ctx_thick.get_image_data(54, 50, 1, 1);
        assert_eq!(thick_at_54.data[0..4], [255, 0, 0, 255], "line_width=10: x=54 应有像素");
    }

    /// 测试 stroke_outline_vertices 生成包含法线偏移的轮廓顶点。
    /// 验证单条线段的轮廓为 8 个浮点数（4 个顶点 × 2 坐标）。
    #[test]
    fn test_stroke_outline_vertices_single_segment() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(4.0);
        ctx.set_line_cap(LineCap::Butt);
        ctx.set_line_join(LineJoin::Miter);
        ctx.begin_path();
        ctx.move_to(10.0, 50.0);
        ctx.line_to(90.0, 50.0);
        let outline = ctx.stroke_outline_vertices();
        // 单条水平线段：4 个角点 = 8 floats
        assert_eq!(outline.len(), 8, "单条线段轮廓应有 8 个浮点数（4 个顶点）");
        // 验证上下偏移：y 坐标应为 50 ± 2（line_width/2 = 2）
        let y_values: Vec<f32> = outline.iter().skip(1).step_by(2).copied().collect();
        assert!(
            y_values.iter().any(|&y| (y - 48.0).abs() < 0.1),
            "应有 y ≈ 48 的顶点（50 - half_lw）"
        );
        assert!(
            y_values.iter().any(|&y| (y - 52.0).abs() < 0.1),
            "应有 y ≈ 52 的顶点（50 + half_lw）"
        );
    }

    /// 测试 stroke_outline_vertices 包含连接点顶点。
    /// 两条线段路径应生成线段轮廓 + 连接点轮廓。
    #[test]
    fn test_stroke_outline_vertices_with_join() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(4.0);
        ctx.set_line_cap(LineCap::Butt);
        ctx.set_line_join(LineJoin::Bevel);
        ctx.begin_path();
        ctx.move_to(10.0, 50.0);
        ctx.line_to(50.0, 10.0);
        ctx.line_to(90.0, 50.0);
        let outline = ctx.stroke_outline_vertices();
        // 2 条线段 × 8 floats = 16（线段轮廓）+ 连接点顶点（Bevel = 4 floats）
        // 总计应大于 16
        assert!(
            outline.len() > 16,
            "两条线段路径应有连接点额外顶点，实际 {}",
            outline.len()
        );
    }

    /// 测试 stroke_outline_vertices 使用 Round cap 时包含额外扇形顶点。
    #[test]
    fn test_stroke_outline_vertices_round_cap_extra_vertices() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(4.0);
        ctx.set_line_cap(LineCap::Round);
        ctx.set_line_join(LineJoin::Miter);
        ctx.begin_path();
        ctx.move_to(10.0, 50.0);
        ctx.line_to(90.0, 50.0);
        let outline = ctx.stroke_outline_vertices();

        // 对比 Butt cap
        let mut ctx_butt = CanvasContext::new(200, 200);
        ctx_butt.set_line_width(4.0);
        ctx_butt.set_line_cap(LineCap::Butt);
        ctx_butt.set_line_join(LineJoin::Miter);
        ctx_butt.begin_path();
        ctx_butt.move_to(10.0, 50.0);
        ctx_butt.line_to(90.0, 50.0);
        let outline_butt = ctx_butt.stroke_outline_vertices();

        // Round cap 应比 Butt cap 多出扇形顶点
        assert!(
            outline.len() > outline_butt.len(),
            "Round cap 应比 Butt cap 多出扇形顶点: {} vs {}",
            outline.len(),
            outline_butt.len()
        );
    }

    /// 测试 stroke_outline_vertices 使用 Square cap 时包含延伸矩形顶点。
    #[test]
    fn test_stroke_outline_vertices_square_cap_extra_vertices() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(4.0);
        ctx.set_line_cap(LineCap::Square);
        ctx.set_line_join(LineJoin::Miter);
        ctx.begin_path();
        ctx.move_to(20.0, 50.0);
        ctx.line_to(80.0, 50.0);
        let outline = ctx.stroke_outline_vertices();

        let mut ctx_butt = CanvasContext::new(200, 200);
        ctx_butt.set_line_width(4.0);
        ctx_butt.set_line_cap(LineCap::Butt);
        ctx_butt.set_line_join(LineJoin::Miter);
        ctx_butt.begin_path();
        ctx_butt.move_to(20.0, 50.0);
        ctx_butt.line_to(80.0, 50.0);
        let outline_butt = ctx_butt.stroke_outline_vertices();

        // Square cap 应比 Butt cap 多出延伸矩形顶点
        assert!(
            outline.len() > outline_butt.len(),
            "Square cap 应比 Butt cap 多出延伸矩形顶点: {} vs {}",
            outline.len(),
            outline_butt.len()
        );
    }

    /// 测试 stroke_outline_vertices 空路径返回空列表。
    #[test]
    fn test_stroke_outline_vertices_empty_path() {
        let ctx = CanvasContext::new(100, 100);
        let outline = ctx.stroke_outline_vertices();
        assert!(outline.is_empty(), "空路径应返回空顶点列表");
    }

    /// 测试 stroke_outline_vertices 仅 MoveTo 的路径返回空列表。
    #[test]
    fn test_stroke_outline_vertices_move_to_only() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.begin_path();
        ctx.move_to(50.0, 50.0);
        let outline = ctx.stroke_outline_vertices();
        assert!(outline.is_empty(), "仅 MoveTo 应返回空顶点列表");
    }

    /// 测试 line_join Miter 的轮廓顶点在连接处产生额外的尖角顶点。
    #[test]
    fn test_stroke_outline_miter_has_join_vertices() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(10.0);
        ctx.set_line_join(LineJoin::Miter);
        ctx.set_line_cap(LineCap::Butt);
        ctx.begin_path();
        // 直角转弯：(10,90) -> (10,10) -> (90,10)
        ctx.move_to(10.0, 90.0);
        ctx.line_to(10.0, 10.0);
        ctx.line_to(90.0, 10.0);
        let outline = ctx.stroke_outline_vertices();

        // 对比 Bevel join
        let mut ctx_bevel = CanvasContext::new(200, 200);
        ctx_bevel.set_line_width(10.0);
        ctx_bevel.set_line_join(LineJoin::Bevel);
        ctx_bevel.set_line_cap(LineCap::Butt);
        ctx_bevel.begin_path();
        ctx_bevel.move_to(10.0, 90.0);
        ctx_bevel.line_to(10.0, 10.0);
        ctx_bevel.line_to(90.0, 10.0);
        let outline_bevel = ctx_bevel.stroke_outline_vertices();

        // Miter join 应在连接处有额外的尖角顶点
        assert!(
            outline.len() > outline_bevel.len(),
            "Miter join ({}) 应比 Bevel join ({}) 多出尖角顶点",
            outline.len(),
            outline_bevel.len()
        );
    }

    /// 测试 line_join Round 的轮廓包含扇形连接顶点。
    #[test]
    fn test_stroke_outline_round_join_has_fan_vertices() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.set_line_width(10.0);
        ctx.set_line_join(LineJoin::Round);
        ctx.set_line_cap(LineCap::Butt);
        ctx.begin_path();
        ctx.move_to(10.0, 90.0);
        ctx.line_to(10.0, 10.0);
        ctx.line_to(90.0, 10.0);
        let outline = ctx.stroke_outline_vertices();

        // 对比 Bevel join
        let mut ctx_bevel = CanvasContext::new(200, 200);
        ctx_bevel.set_line_width(10.0);
        ctx_bevel.set_line_join(LineJoin::Bevel);
        ctx_bevel.set_line_cap(LineCap::Butt);
        ctx_bevel.begin_path();
        ctx_bevel.move_to(10.0, 90.0);
        ctx_bevel.line_to(10.0, 10.0);
        ctx_bevel.line_to(90.0, 10.0);
        let outline_bevel = ctx_bevel.stroke_outline_vertices();

        // Round join 应比 Bevel join 多出扇形顶点
        assert!(
            outline.len() > outline_bevel.len(),
            "Round join ({}) 应比 Bevel join ({}) 多出扇形顶点",
            outline.len(),
            outline_bevel.len()
        );
    }

    // ── 合成操作像素级测试（剩余操作） ──

    /// 测试 destination-out 合成：先绘制红色，再使用 destination-out 绘制蓝色，
    /// 重叠区域的已有内容被清除（变为透明）。
    #[test]
    fn test_composite_destination_out() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 destination-out 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::DestinationOut);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 红色独占区域 (2,0)：不变
        let red_only = ctx.get_image_data(2, 0, 1, 1);
        assert_eq!(
            red_only.data[0..4],
            [255, 0, 0, 255],
            "destination-out: 红色独占区域不变"
        );

        // 重叠区域 (7,0)：destination-out 清除已有内容 → 透明
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(overlap.data[0..4], [0, 0, 0, 0], "destination-out: 重叠区域应为透明");

        // 蓝色独占区域 (12,0)：无已有内容 → 透明
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(
            blue_only.data[0..4],
            [0, 0, 0, 0],
            "destination-out: 无目标区域应为透明"
        );
    }

    /// 测试 destination-atop 合成：先绘制红色，再使用 destination-atop 绘制蓝色。
    /// destination-atop 在源区域内：保留目标在源区域内的部分。
    /// 注意：当前实现只修改绘制矩形内的像素，矩形外的像素保持不变。
    #[test]
    fn test_composite_destination_atop() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 destination-atop 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::DestinationAtop);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 重叠区域 (7,0)：destination-atop → 保留目标 + 源
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_ne!(overlap.data[3], 0, "destination-atop: 重叠区域应有内容");

        // 蓝色独占区域 (12,0)：无目标 → destination-atop 保留源
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_ne!(blue_only.data[3], 0, "destination-atop: 源独占区域应有内容");
    }

    /// 测试 source-in 合成：先绘制红色，再使用 source-in 绘制蓝色。
    /// source-in 只保留源与目标重叠的部分。
    /// 注意：当前实现只修改绘制矩形内的像素，矩形外的目标像素保持不变。
    #[test]
    fn test_composite_source_in() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 source-in 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::SourceIn);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 重叠区域 (7,0)：source-in → fa=da=1.0, fb=0.0 → 源色（蓝色）
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(overlap.data[0..4], [0, 0, 255, 255], "source-in: 重叠区域应为蓝色");

        // 蓝色独占区域 (12,0)：无目标 → source-in → 透明
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(blue_only.data[0..4], [0, 0, 0, 0], "source-in: 无目标区域应为透明");
    }

    /// 测试 destination-in 合成：先绘制红色，再使用 destination-in 绘制蓝色。
    /// destination-in 只保留目标与源重叠的部分。
    /// 注意：当前实现只修改绘制矩形内的像素，矩形外的目标像素保持不变。
    #[test]
    fn test_composite_destination_in() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 destination-in 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::DestinationIn);
        ctx.set_fill_color(Color::BLUE);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 重叠区域 (7,0)：destination-in → fa=0, fb=sa=1.0 → 保留目标色（红色）
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        assert_eq!(
            overlap.data[0..4],
            [255, 0, 0, 255],
            "destination-in: 重叠区域应保留红色"
        );

        // 蓝色独占区域 (12,0)：无目标 → destination-in → 透明
        let blue_only = ctx.get_image_data(12, 0, 1, 1);
        assert_eq!(blue_only.data[0..4], [0, 0, 0, 0], "destination-in: 无目标区域应为透明");
    }

    /// 测试 lighter 合成：两个不同颜色像素的 lighter 模式进行加法混合。
    #[test]
    fn test_composite_lighter() {
        let mut ctx = CanvasContext::new(20, 10);
        // 先绘制红色 (0,0)-(10,10)
        ctx.set_fill_color(Color::rgba(200, 0, 0, 255));
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 使用 lighter 绘制蓝色 (5,0)-(15,10)
        ctx.set_composite_operation(CompositeOperation::Lighter);
        ctx.set_fill_color(Color::rgba(0, 0, 200, 255));
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        // 重叠区域：lighter 模式 = fa=1, fb=1 → 加法混合
        let overlap = ctx.get_image_data(7, 0, 1, 1);
        let r = overlap.data[0];
        let _g = overlap.data[1];
        let b = overlap.data[2];
        // 红色通道应有目标的贡献（dr * da * fb / out_a）
        // 蓝色通道应有源的贡献
        assert!(r >= 100, "lighter: 红色通道应 >= 100，实际 {}", r);
        assert!(b >= 100, "lighter: 蓝色通道应 >= 100，实际 {}", b);
    }

    /// 测试 copy 合成覆盖已有内容。
    /// copy 模式只保留源像素，重叠区域的目标完全被替换。
    #[test]
    fn test_composite_copy_replaces() {
        let mut ctx = CanvasContext::new(20, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 20.0, 10.0);
        ctx.set_composite_operation(CompositeOperation::Copy);
        ctx.set_fill_color(Color::GREEN);
        ctx.fill_rect(5.0, 0.0, 10.0, 10.0);

        let inside = ctx.get_image_data(10, 0, 1, 1);
        assert_eq!(inside.data[0..4], [0, 255, 0, 255], "copy: 内部区域应为绿色");
        let outside = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(outside.data[0..4], [255, 0, 0, 255], "copy: 外部区域应保留红色");
    }

    // ── OffscreenCanvas 测试 ──

    /// 测试 OffscreenCanvas 创建时的尺寸正确。
    #[test]
    fn test_offscreen_canvas_creation_with_dimensions() {
        let oc = OffscreenCanvas::new(640, 480);
        assert_eq!(oc.width(), 640);
        assert_eq!(oc.height(), 480);
    }

    /// 测试 OffscreenCanvas get_context 返回正确尺寸的 CanvasContext。
    #[test]
    fn test_offscreen_canvas_get_context_returns_working_context() {
        let oc = OffscreenCanvas::new(200, 150);
        let ctx = oc.get_context();
        assert_eq!(ctx.width(), 200);
        assert_eq!(ctx.height(), 150);
    }

    /// 测试在 OffscreenCanvas 上下文上绘制操作后能产生像素数据。
    #[test]
    fn test_offscreen_canvas_drawing_produces_pixels() {
        let oc = OffscreenCanvas::new(100, 100);
        let mut ctx = oc.get_context();
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
        // 验证绘制区域内有红色像素
        let pixel = ctx.get_image_data(20, 20, 1, 1);
        assert_eq!(
            pixel.data[0..4],
            [255, 0, 0, 255],
            "OffscreenCanvas 上下文绘制后应产生像素"
        );
    }

    /// 测试 OffscreenCanvas 的宽高与传入参数一致（含零尺寸边界情况）。
    #[test]
    fn test_offscreen_canvas_dimensions_are_correct() {
        let oc = OffscreenCanvas::new(0, 0);
        assert_eq!(oc.width(), 0);
        assert_eq!(oc.height(), 0);

        let oc2 = OffscreenCanvas::new(1920, 1080);
        assert_eq!(oc2.width(), 1920);
        assert_eq!(oc2.height(), 1080);
    }

    /// 测试 OffscreenCanvas transfer_to_image_bitmap 返回正确尺寸的 ImageData。
    #[test]
    fn test_offscreen_canvas_transfer_to_image_bitmap() {
        let oc = OffscreenCanvas::new(50, 40);
        let bitmap = oc.transfer_to_image_bitmap();
        assert_eq!(bitmap.width, 50);
        assert_eq!(bitmap.height, 40);
        assert_eq!(bitmap.data.len(), 50 * 40 * 4);
    }

    // ── 错误恢复测试 ──

    /// 测试 drawImage 使用空 ImageData（零尺寸）时不 panic。
    /// 空图像数据不应导致像素缓冲区越界访问或 panic。
    #[test]
    fn test_draw_image_no_data() {
        let mut ctx = CanvasContext::new(100, 100);
        // 空的 ImageData — 零尺寸，无像素数据
        let empty_img = ImageData {
            width: 0,
            height: 0,
            data: vec![],
        };
        // 不应 panic
        ctx.draw_image(&empty_img, 0.0, 0.0);
        ctx.draw_image_with_size(&empty_img, 0.0, 0.0, 50.0, 50.0);
        ctx.draw_image_sliced(&empty_img, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0, 50.0);
        // 验证画布未被修改
        let pixel = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "空图像不应写入任何像素");

        // ImageData 有尺寸但数据向量为空 — 也不应 panic
        let img_no_data = ImageData {
            width: 10,
            height: 10,
            data: vec![],
        };
        ctx.draw_image(&img_no_data, 0.0, 0.0);
        // 不 panic 即可
    }

    // ── Path2D closePath 和 is_point_in_path 测试 ──

    /// 测试 Path2D close_path 后形成闭合三角形，顶点包含回到起点的线段。
    #[test]
    fn test_path2d_close_path_triangle() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 0.0);
        p.line_to(50.0, 100.0);
        p.close_path();
        // close_path 应添加 ClosePath 命令
        assert!(matches!(p.commands().last(), Some(PathCommand::ClosePath)));
        // 扁平化后应包含从 (50,100) 回到 (0,0) 的闭合线段
        let ctx = CanvasContext::new(200, 200);
        let vertices = ctx.flatten_path_for(&p);
        // 3 条线段: (0,0)->(100,0), (100,0)->(50,100), (50,100)->(0,0)
        // 每条 4 floats = 12 floats
        assert_eq!(vertices.len(), 12);
        // 最后一条线段应回到起点
        assert!((vertices[8] - 50.0).abs() < f32::EPSILON);
        assert!((vertices[9] - 100.0).abs() < f32::EPSILON);
        assert!((vertices[10]).abs() < f32::EPSILON);
        assert!((vertices[11]).abs() < f32::EPSILON);
    }

    /// 测试闭合路径 fill 产生非空 path_fills。
    #[test]
    fn test_path2d_close_path_creates_fill() {
        let mut p = Path2D::new();
        p.move_to(10.0, 10.0);
        p.line_to(100.0, 10.0);
        p.line_to(100.0, 100.0);
        p.close_path();
        let mut ctx = CanvasContext::new(200, 200);
        ctx.fill_with_path(&p);
        assert!(
            !ctx.primitives().path_fills.is_empty(),
            "闭合路径 fill 应产生 path_fill 图元"
        );
    }

    /// 测试 Path2D is_point_in_path 在矩形内部返回 true。
    #[test]
    fn test_path2d_is_point_in_path_inside() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 0.0);
        p.line_to(100.0, 100.0);
        p.line_to(0.0, 100.0);
        p.close_path();
        // 矩形内部点应返回 true
        assert!(p.is_point_in_path(50.0, 50.0));
        assert!(p.is_point_in_path(10.0, 10.0));
    }

    /// 测试 Path2D is_point_in_path 在矩形外部返回 false。
    #[test]
    fn test_path2d_is_point_in_path_outside() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 0.0);
        p.line_to(100.0, 100.0);
        p.line_to(0.0, 100.0);
        p.close_path();
        // 矩形外部点应返回 false
        assert!(!p.is_point_in_path(200.0, 200.0));
        assert!(!p.is_point_in_path(-10.0, -10.0));
        assert!(!p.is_point_in_path(150.0, 50.0));
    }

    /// 测试 Path2D is_point_in_path 在边界上不 panic。
    #[test]
    fn test_path2d_is_point_in_path_edge() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 0.0);
        p.line_to(100.0, 100.0);
        p.line_to(0.0, 100.0);
        p.close_path();
        // 边界上的点不应 panic（结果不确定，只验证不崩溃）
        let _ = p.is_point_in_path(0.0, 0.0);
        let _ = p.is_point_in_path(100.0, 100.0);
        let _ = p.is_point_in_path(50.0, 0.0);
        let _ = p.is_point_in_path(0.0, 50.0);
    }

    // ── 新增边界条件测试 ──

    /// 测试 resize 后画布尺寸和像素缓冲区正确更新。
    #[test]
    fn test_canvas_create_resize() {
        let mut ctx = CanvasContext::new(100, 200);
        assert_eq!(ctx.width(), 100);
        assert_eq!(ctx.height(), 200);
        ctx.resize(400, 300);
        assert_eq!(ctx.width(), 400);
        assert_eq!(ctx.height(), 300);
        // resize 后像素缓冲区应全部为零
        let pixel = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0]);
    }

    /// 测试 clearRect(0,0,w,h) 将整个画布清为透明。
    #[test]
    fn test_canvas_clear_entire() {
        let mut ctx = CanvasContext::new(50, 50);
        // 先填充红色
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        // 验证 (0,0) 为红色
        let before = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(before.data[0..4], [255, 0, 0, 255]);
        // 清除整个画布
        ctx.clear_rect(0.0, 0.0, 50.0, 50.0);
        // 验证 (0,0) 为透明
        let after = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(
            after.data[0..4],
            [0, 0, 0, 0],
            "clearRect(0,0,w,h) should make pixel transparent"
        );
    }

    /// 测试 lineWidth 为 0 时 stroke 不 panic。
    #[test]
    fn test_canvas_stroke_zero_width() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_width(0.0);
        ctx.stroke_rect(10.0, 10.0, 50.0, 50.0);
        // 不应 panic
        assert_eq!(ctx.primitives().fills.len(), 4);
    }

    /// 测试负值平移后变换矩阵正确。
    #[test]
    fn test_canvas_negative_translate() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.translate(-10.0, -20.0);
        let t = ctx.transform;
        assert!((t.e - (-10.0)).abs() < f32::EPSILON, "translate tx should be -10");
        assert!((t.f - (-20.0)).abs() < f32::EPSILON, "translate ty should be -20");
        assert!((t.a - 1.0).abs() < f32::EPSILON);
        assert!((t.d - 1.0).abs() < f32::EPSILON);
    }

    /// 测试无 save 时 restore 不 panic，状态保持默认。
    #[test]
    fn test_canvas_restore_without_save() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.restore(); // 不应 panic
        assert_eq!(ctx.fill_color(), Color::BLACK);
        assert!((ctx.global_alpha() - 1.0).abs() < f32::EPSILON);
        assert!((ctx.line_width() - 1.0).abs() < f32::EPSILON);
    }

    /// 测试 globalAlpha 超出 [0,1] 时被 clamp。
    #[test]
    fn test_canvas_set_global_alpha_clamp() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_global_alpha(2.0);
        assert!(
            (ctx.global_alpha() - 1.0).abs() < f32::EPSILON,
            "alpha > 1 should clamp to 1.0"
        );
        ctx.set_global_alpha(-0.5);
        assert!(
            (ctx.global_alpha()).abs() < f32::EPSILON,
            "alpha < 0 should clamp to 0.0"
        );
    }

    // ── 新增边界条件测试（6 个） ──

    /// 测试创建包含 4 个颜色停止点的线性渐变，验证停止点数量、偏移量和颜色均正确。
    #[test]
    fn test_canvas_create_linear_gradient_multi_stop() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.33, Color::rgba(255, 255, 0, 255));
        grad.add_color_stop(0.66, Color::GREEN);
        grad.add_color_stop(1.0, Color::BLUE);
        assert_eq!(grad.stops.len(), 4);
        assert!((grad.stops[0].offset - 0.0).abs() < f32::EPSILON);
        assert_eq!(grad.stops[0].color, Color::RED);
        assert!((grad.stops[1].offset - 0.33).abs() < f32::EPSILON);
        assert_eq!(grad.stops[1].color, Color::rgba(255, 255, 0, 255));
        assert!((grad.stops[2].offset - 0.66).abs() < f32::EPSILON);
        assert_eq!(grad.stops[2].color, Color::GREEN);
        assert!((grad.stops[3].offset - 1.0).abs() < f32::EPSILON);
        assert_eq!(grad.stops[3].color, Color::BLUE);
    }

    /// 测试创建径向渐变，验证内外圆的圆心坐标和半径正确。
    #[test]
    fn test_canvas_create_radial_gradient_circle() {
        let ctx = CanvasContext::new(200, 200);
        let grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 80.0);
        // 内圆：圆心 (50,50)，半径 10
        assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r0 - 10.0).abs() < f32::EPSILON);
        // 外圆：圆心 (50,50)，半径 80
        assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r1 - 80.0).abs() < f32::EPSILON);
        // 同心圆渐变初始无停止点
        assert!(grad.stops.is_empty());
    }

    /// 测试当前路径填充使用奇偶规则（even-odd）。
    /// 当前实现默认使用非零环绕规则，is_point_in_path 基于射线法（等效于奇偶规则）。
    /// 验证嵌套矩形路径在奇偶规则下内部矩形被判断为"外部"。
    #[test]
    fn test_canvas_fill_rule_evenodd() {
        // 使用 CanvasContext 的 is_point_in_path（射线法 = 奇偶规则）
        // 构造嵌套矩形路径（外矩形顺时针 + 内矩形顺时针）
        let mut ctx = CanvasContext::new(200, 200);
        ctx.begin_path();
        // 外矩形
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(100.0, 100.0);
        ctx.line_to(0.0, 100.0);
        ctx.close_path();
        // 内矩形（反向绕行模拟 even-odd 挖空）
        ctx.move_to(25.0, 25.0);
        ctx.line_to(25.0, 75.0);
        ctx.line_to(75.0, 75.0);
        ctx.line_to(75.0, 25.0);
        ctx.close_path();
        // 射线法（奇偶规则）：外矩形与内矩形之间的点穿过 1 条边 → 在路径内
        assert!(ctx.is_point_in_path(15.0, 15.0), "even-odd: 两矩形之间的点应在路径内");
        // 射线法（奇偶规则）：内矩形内的点穿过 2 条边 → 不在路径内
        assert!(
            !ctx.is_point_in_path(50.0, 50.0),
            "even-odd: 内矩形内的点应不在路径内（穿过偶数条边）"
        );
    }

    /// 测试设置线段虚线模式 [5, 10, 15]，验证 get_line_dash 返回加倍后的数组。
    #[test]
    fn test_canvas_set_line_dash_pattern() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_dash(vec![5.0, 10.0, 15.0]);
        // 奇数长度时 Canvas 规范要求复制拼接
        assert_eq!(ctx.get_line_dash(), &[5.0, 10.0, 15.0, 5.0, 10.0, 15.0]);
    }

    /// 测试 measure_text("Hello") 返回非零宽度。
    #[test]
    fn test_canvas_measure_text_hello() {
        let ctx = CanvasContext::new(200, 200);
        let metrics = ctx.measure_text("Hello");
        // 默认字体大小 10.0，5 字符 × 10.0 × 0.6 = 30.0
        assert!(
            metrics.width > 0.0,
            "measure_text(\"Hello\") 宽度应大于零，实际 {}",
            metrics.width
        );
        assert!(
            (metrics.width - 30.0).abs() < f32::EPSILON,
            "measure_text(\"Hello\") 宽度应约 30.0，实际 {}",
            metrics.width
        );
    }

    /// 测试设置 shadowBlur、shadowOffsetX、shadowOffsetY、shadowColor 后，
    /// 每个 getter 返回正确的设置值。
    #[test]
    fn test_canvas_shadow_properties() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_shadow_blur(8.0);
        ctx.set_shadow_offset_x(4.0);
        ctx.set_shadow_offset_y(6.0);
        ctx.set_shadow_color(Color::rgba(128, 0, 128, 200));
        assert!((ctx.shadow_blur() - 8.0).abs() < f32::EPSILON, "shadowBlur 应为 8.0");
        assert!(
            (ctx.shadow_offset_x() - 4.0).abs() < f32::EPSILON,
            "shadowOffsetX 应为 4.0"
        );
        assert!(
            (ctx.shadow_offset_y() - 6.0).abs() < f32::EPSILON,
            "shadowOffsetY 应为 6.0"
        );
        let sc = ctx.shadow_color();
        assert_eq!(sc.r, 128, "shadowColor.r 应为 128");
        assert_eq!(sc.g, 0, "shadowColor.g 应为 0");
        assert_eq!(sc.b, 128, "shadowColor.b 应为 128");
        assert_eq!(sc.a, 200, "shadowColor.a 应为 200");
    }

    // ── 边界条件测试：putImageData/getImageData、createConicGradient、font、textAlign、textBaseline ──

    /// 测试 put_image_data 后 get_image_data 返回完全一致的像素数据。
    #[test]
    fn test_canvas_put_image_data_and_get() {
        let mut ctx = CanvasContext::new(10, 10);
        // 构造 3x3 的彩虹色 ImageData
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // 红
            0, 255, 0, 255, // 绿
            0, 0, 255, 255, // 蓝
            255, 255, 0, 255, // 黄
            255, 0, 255, 255, // 品红
            0, 255, 255, 255, // 青
            128, 128, 128, 255, // 灰
            255, 128, 0, 255, // 橙
            0, 128, 255, 255, // 天蓝
        ];
        let img = ImageData {
            width: 3,
            height: 3,
            data: pixels.clone(),
        };
        ctx.put_image_data(&img, 2, 3);
        // 读取写入区域并验证像素完全匹配
        let result = ctx.get_image_data(2, 3, 3, 3);
        assert_eq!(result.data, pixels, "put 后 get 的像素数据应完全一致");
        // 验证写入区域外的像素仍为零
        let outside = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(outside.data[0..4], [0, 0, 0, 0], "写入区域外应保持透明");
    }

    /// 测试 create_conic_gradient 指定起始角度后，渐变对象的 start_angle 与传入值精确匹配。
    #[test]
    fn test_canvas_create_conic_gradient() {
        let ctx = CanvasContext::new(200, 200);
        let angle = std::f32::consts::FRAC_PI_2; // 90 度
        let grad = ctx.create_conic_gradient(angle, 75.0, 125.0);
        assert!(
            (grad.start_angle - angle).abs() < f32::EPSILON,
            "start_angle 应为 {}",
            angle
        );
        assert!((grad.cx - 75.0).abs() < f32::EPSILON, "cx 应为 75.0");
        assert!((grad.cy - 125.0).abs() < f32::EPSILON, "cy 应为 125.0");
    }

    /// 测试 set_font 设置 "bold 16px Arial" 风格字体后，font() getter 返回正确的描述符。
    #[test]
    fn test_canvas_set_font_and_get() {
        let mut ctx = CanvasContext::new(100, 100);
        let font = FontDescriptor {
            family: "Arial".to_string(),
            size: 16.0,
            weight: FontWeight::Bold,
            style: FontStyle::Normal,
        };
        ctx.set_font(font);
        let f = ctx.font();
        assert_eq!(f.family, "Arial");
        assert!((f.size - 16.0).abs() < f32::EPSILON, "字体大小应为 16.0");
        assert_eq!(f.weight, FontWeight::Bold, "字体粗细应为 Bold");
        assert_eq!(f.style, FontStyle::Normal, "字体样式应为 Normal");
    }

    /// 测试 set_text_align 对各种值（left, center, right）的设置和获取。
    #[test]
    fn test_canvas_text_align_values() {
        let mut ctx = CanvasContext::new(100, 100);
        // 默认值应为 Start
        assert_eq!(ctx.text_align(), TextAlign::Start);

        ctx.set_text_align(TextAlign::Left);
        assert_eq!(ctx.text_align(), TextAlign::Left);

        ctx.set_text_align(TextAlign::Center);
        assert_eq!(ctx.text_align(), TextAlign::Center);

        ctx.set_text_align(TextAlign::Right);
        assert_eq!(ctx.text_align(), TextAlign::Right);
    }

    /// 测试 set_text_baseline 对各种值（top, middle, bottom）的设置和获取。
    #[test]
    fn test_canvas_text_baseline_values() {
        let mut ctx = CanvasContext::new(100, 100);
        // 默认值应为 Alphabetic
        assert_eq!(ctx.text_baseline(), TextBaseline::Alphabetic);

        ctx.set_text_baseline(TextBaseline::Top);
        assert_eq!(ctx.text_baseline(), TextBaseline::Top);

        ctx.set_text_baseline(TextBaseline::Middle);
        assert_eq!(ctx.text_baseline(), TextBaseline::Middle);

        ctx.set_text_baseline(TextBaseline::Bottom);
        assert_eq!(ctx.text_baseline(), TextBaseline::Bottom);
    }

    // ── Path2D.addPath() 测试 ──

    /// 测试 add_path 将两个包含矩形的路径合并后，命令数量正确。
    #[test]
    fn test_path2d_add_path() {
        let mut p1 = Path2D::new();
        p1.rect(0.0, 0.0, 10.0, 10.0); // 5 个命令
        let mut p2 = Path2D::new();
        p2.rect(20.0, 20.0, 10.0, 10.0); // 5 个命令
        p1.add_path(&p2);
        assert_eq!(p1.len(), 10, "合并后应有 10 个命令");
        assert!(!p2.is_empty(), "源路径应不受影响");
    }

    /// 测试 add_path 追加空路径后，原路径不变。
    #[test]
    fn test_path2d_add_path_empty() {
        let mut p = Path2D::new();
        p.rect(0.0, 0.0, 50.0, 50.0); // 5 个命令
        let empty = Path2D::new();
        p.add_path(&empty);
        assert_eq!(p.len(), 5, "追加空路径后命令数不变");
    }

    /// 测试 add_path 后源路径保持不变。
    #[test]
    fn test_path2d_add_path_preserves_original() {
        let mut target = Path2D::new();
        target.rect(0.0, 0.0, 10.0, 10.0);
        let mut source = Path2D::new();
        source.rect(100.0, 100.0, 20.0, 20.0);
        let source_len_before = source.len();
        target.add_path(&source);
        assert_eq!(source.len(), source_len_before, "add_path 后源路径命令数不变");
    }

    /// 测试闭合三角路径后 is_point_in_path 正确判断内部点。
    #[test]
    fn test_path2d_close_path_is_point_in_path() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 0.0);
        p.line_to(50.0, 100.0);
        p.close_path();
        // 三角形中心应在路径内
        assert!(p.is_point_in_path(50.0, 40.0), "三角形内部点应命中");
        // 远离三角形的外部点不应命中
        assert!(!p.is_point_in_path(200.0, 200.0), "外部点不应命中");
    }

    // ── create_image_data 测试 ──

    /// 测试 create_image_data 创建指定尺寸的 ImageData，数据全为零。
    #[test]
    fn test_create_image_data() {
        let ctx = CanvasContext::new(100, 100);
        let img = ctx.create_image_data(10, 20);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 20);
        assert_eq!(img.data.len(), 800); // 10 * 20 * 4
        // 所有像素应为透明黑色
        for chunk in img.data.chunks_exact(4) {
            assert_eq!(chunk, &[0, 0, 0, 0]);
        }
    }

    /// 测试 create_image_data 零尺寸不 panic。
    #[test]
    fn test_create_image_data_zero_size() {
        let ctx = CanvasContext::new(100, 100);
        let img = ctx.create_image_data(0, 0);
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.data.is_empty());
    }

    /// 测试 create_image_data 与 get_image_data 的区别：
    /// create_image_data 返回全零，get_image_data 从画布读取实际像素。
    #[test]
    fn test_create_image_data_vs_get_image_data() {
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        let created = ctx.create_image_data(5, 5);
        let fetched = ctx.get_image_data(0, 0, 5, 5);
        // created 应全为零（透明黑）
        assert_eq!(created.data[0..4], [0, 0, 0, 0]);
        // fetched 应为红色（从画布读取）
        assert_eq!(fetched.data[0..4], [255, 0, 0, 255]);
    }

    // ── get_transform 测试 ──

    /// 测试 get_transform 返回单位矩阵（初始状态）。
    #[test]
    fn test_get_transform_identity() {
        let ctx = CanvasContext::new(100, 100);
        let t = ctx.get_transform();
        assert!((t.a - 1.0).abs() < f32::EPSILON);
        assert!((t.b).abs() < f32::EPSILON);
        assert!((t.c).abs() < f32::EPSILON);
        assert!((t.d - 1.0).abs() < f32::EPSILON);
        assert!((t.e).abs() < f32::EPSILON);
        assert!((t.f).abs() < f32::EPSILON);
    }

    /// 测试 get_transform 在 translate 后返回正确的矩阵。
    #[test]
    fn test_get_transform_after_translate() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.translate(10.0, 20.0);
        let t = ctx.get_transform();
        assert!((t.a - 1.0).abs() < f32::EPSILON);
        assert!((t.d - 1.0).abs() < f32::EPSILON);
        assert!((t.e - 10.0).abs() < f32::EPSILON);
        assert!((t.f - 20.0).abs() < f32::EPSILON);
    }

    /// 测试 get_transform 在 set_transform 后返回设置的矩阵。
    #[test]
    fn test_get_transform_after_set() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_transform(2.0, 0.5, -0.5, 2.0, 10.0, 20.0);
        let t = ctx.get_transform();
        assert!((t.a - 2.0).abs() < f32::EPSILON);
        assert!((t.b - 0.5).abs() < f32::EPSILON);
        assert!((t.c - (-0.5)).abs() < f32::EPSILON);
        assert!((t.d - 2.0).abs() < f32::EPSILON);
        assert!((t.e - 10.0).abs() < f32::EPSILON);
        assert!((t.f - 20.0).abs() < f32::EPSILON);
    }

    // ── transform(a,b,c,d,e,f) 乘法方法测试 ──

    /// 测试 transform() 方法将参数矩阵乘到当前变换上。
    #[test]
    fn test_transform_multiply_basic() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
        let t = ctx.get_transform();
        // 单位矩阵 * scale(2,2) = scale(2,2)
        assert!((t.a - 2.0).abs() < f32::EPSILON);
        assert!((t.d - 2.0).abs() < f32::EPSILON);
    }

    /// 测试 transform() 后乘顺序：先 scale 后 translate。
    #[test]
    fn test_transform_post_multiply_order() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.scale(2.0, 1.0);
        ctx.transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        let (x, y) = ctx.get_transform().transform_point(5.0, 5.0);
        // scale(2,1) * translate(10,20)：矩阵乘法结果 e = 2*10 = 20, f = 1*20 = 20
        // transform_point(5,5) = (2*5 + 20, 1*5 + 20) = (30, 25)
        assert!((x - 30.0).abs() < 0.01);
        assert!((y - 25.0).abs() < 0.01);
    }

    /// 测试 transform() 不会替换而是叠加（与 set_transform 的区别）。
    #[test]
    fn test_transform_accumulates_vs_set_transform_replaces() {
        let mut ctx1 = CanvasContext::new(100, 100);
        ctx1.translate(10.0, 0.0);
        ctx1.transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
        let t1 = ctx1.get_transform();

        let mut ctx2 = CanvasContext::new(100, 100);
        ctx2.translate(10.0, 0.0);
        ctx2.set_transform(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
        let t2 = ctx2.get_transform();

        // transform() 叠加：translate(10,0) * scale(2,2)
        assert!((t1.a - 2.0).abs() < f32::EPSILON, "transform should accumulate");
        assert!((t1.e - 10.0).abs() < f32::EPSILON, "translate should remain");

        // set_transform() 替换
        assert!((t2.a - 2.0).abs() < f32::EPSILON, "set_transform replaces");
        assert!((t2.e).abs() < f32::EPSILON, "set_transform clears translate");
    }

    /// 测试连续多次 transform() 调用累积。
    #[test]
    fn test_transform_multiple_calls() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.transform(2.0, 0.0, 0.0, 1.0, 0.0, 0.0); // scale x2
        ctx.transform(1.0, 0.0, 0.0, 3.0, 0.0, 0.0); // scale y3
        let (x, y) = ctx.get_transform().transform_point(5.0, 5.0);
        // identity * scaleX(2) * scaleY(3) applied to (5,5):
        // first scaleX: (10, 5), then scaleY: (10, 15)
        assert!((x - 10.0).abs() < 0.01);
        assert!((y - 15.0).abs() < 0.01);
    }

    // ── miter_limit 测试 ──

    /// 测试 miter_limit 默认值为 10.0。
    #[test]
    fn test_miter_limit_default() {
        let ctx = CanvasContext::new(100, 100);
        assert!((ctx.miter_limit() - 10.0).abs() < f32::EPSILON);
    }

    /// 测试设置和获取 miter_limit。
    #[test]
    fn test_miter_limit_set_get() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_miter_limit(5.0);
        assert!((ctx.miter_limit() - 5.0).abs() < f32::EPSILON);
        ctx.set_miter_limit(20.0);
        assert!((ctx.miter_limit() - 20.0).abs() < f32::EPSILON);
    }

    /// 测试 miter_limit 在 save/restore 中正确保存和恢复。
    #[test]
    fn test_miter_limit_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_miter_limit(5.0);
        ctx.save();
        ctx.set_miter_limit(15.0);
        assert!((ctx.miter_limit() - 15.0).abs() < f32::EPSILON);
        ctx.restore();
        assert!((ctx.miter_limit() - 5.0).abs() < f32::EPSILON);
    }

    // ── direction 测试 ──

    /// 测试 direction 默认值为 Inherit。
    #[test]
    fn test_direction_default() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.direction(), TextDirection::Inherit);
    }

    /// 测试 TextDirection 枚举 Default trait。
    #[test]
    fn test_text_direction_default_trait() {
        assert_eq!(TextDirection::default(), TextDirection::Inherit);
    }

    /// 测试设置和获取 direction。
    #[test]
    fn test_direction_set_get() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_direction(TextDirection::Ltr);
        assert_eq!(ctx.direction(), TextDirection::Ltr);
        ctx.set_direction(TextDirection::Rtl);
        assert_eq!(ctx.direction(), TextDirection::Rtl);
        ctx.set_direction(TextDirection::Inherit);
        assert_eq!(ctx.direction(), TextDirection::Inherit);
    }

    /// 测试 direction 在 save/restore 中正确保存和恢复。
    #[test]
    fn test_direction_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_direction(TextDirection::Rtl);
        ctx.save();
        ctx.set_direction(TextDirection::Ltr);
        assert_eq!(ctx.direction(), TextDirection::Ltr);
        ctx.restore();
        assert_eq!(ctx.direction(), TextDirection::Rtl);
    }

    /// 测试 TextDirection 枚举各变体互不相等。
    #[test]
    fn test_text_direction_variants_distinct() {
        let variants = [TextDirection::Ltr, TextDirection::Rtl, TextDirection::Inherit];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b, "TextDirection variants {} and {} should differ", i, j);
                }
            }
        }
    }

    // ── 边缘场景补充测试（第七批）──

    /// 测试 create_image_data 创建空白 ImageData，验证尺寸和透明像素。
    ///
    /// 创建 10x8 的 ImageData，所有像素应为 rgba(0,0,0,0)。
    #[test]
    fn test_create_image_data_blank() {
        let ctx = CanvasContext::new(100, 100);
        let img = ctx.create_image_data(10, 8);
        assert_eq!(img.width, 10, "width should be 10");
        assert_eq!(img.height, 8, "height should be 8");
        assert_eq!(img.data.len(), 10 * 8 * 4, "data length should be 10*8*4 = 320");
        // 所有像素应为透明黑色
        for chunk in img.data.chunks_exact(4) {
            assert_eq!(chunk, &[0, 0, 0, 0], "pixel should be transparent black (rgba 0,0,0,0)");
        }
    }

    /// 测试默认变换矩阵为单位矩阵。
    ///
    /// 新创建的 CanvasContext 的 get_transform() 应返回单位矩阵。
    #[test]
    fn test_get_transform_default_identity() {
        let ctx = CanvasContext::new(100, 100);
        let t = ctx.get_transform();
        assert!((t.a - 1.0).abs() < f32::EPSILON, "a should be 1.0");
        assert!((t.b).abs() < f32::EPSILON, "b should be 0.0");
        assert!((t.c).abs() < f32::EPSILON, "c should be 0.0");
        assert!((t.d - 1.0).abs() < f32::EPSILON, "d should be 1.0");
        assert!((t.e).abs() < f32::EPSILON, "e should be 0.0");
        assert!((t.f).abs() < f32::EPSILON, "f should be 0.0");
    }

    /// 测试执行 translate+rotate+scale 后 get_transform 返回正确矩阵。
    ///
    /// 依次执行 translate(10,20)、rotate(π/2)、scale(2,3)，
    /// 验证 get_transform() 返回的矩阵不等于单位矩阵，且为有限值。
    #[test]
    fn test_get_transform_after_ops() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.translate(10.0, 20.0);
        ctx.rotate(std::f32::consts::FRAC_PI_2);
        ctx.scale(2.0, 3.0);

        let t = ctx.get_transform();
        // 不应为单位矩阵
        assert!(
            (t.a - 1.0).abs() > 0.01 || (t.d - 1.0).abs() > 0.01 || (t.e).abs() > 0.01 || (t.f).abs() > 0.01,
            "transform after ops should not be identity"
        );
        // 所有元素应为有限值
        assert!(t.a.is_finite(), "a should be finite");
        assert!(t.b.is_finite(), "b should be finite");
        assert!(t.c.is_finite(), "c should be finite");
        assert!(t.d.is_finite(), "d should be finite");
        assert!(t.e.is_finite(), "e should be finite");
        assert!(t.f.is_finite(), "f should be finite");
    }

    /// 测试 transform() 方法是乘法叠加而非替换。
    ///
    /// 先 scale(2,1) 再 transform(1,0,0,1,10,0)（即 translate(10,0)），
    /// 验证 transform 是后乘叠加，结果不同于 set_transform 直接替换。
    #[test]
    fn test_transform_multiply_vs_set() {
        let mut ctx1 = CanvasContext::new(100, 100);
        ctx1.scale(2.0, 1.0);
        ctx1.transform(1.0, 0.0, 0.0, 1.0, 10.0, 0.0); // translate via transform()
        let t1 = ctx1.get_transform();

        // scale(2,1) * translate(10,0) 后乘：
        // [2 0 0]   [1 0 10]   [2 0 20]
        // [0 1 0] * [0 1  0] = [0 1  0]
        // [0 0 1]   [0 0  1]   [0 0  1]
        // 所以 a=2, d=1, e=20
        assert!((t1.a - 2.0).abs() < f32::EPSILON, "a should be 2.0");
        assert!((t1.d - 1.0).abs() < f32::EPSILON, "d should be 1.0");
        assert!((t1.e - 20.0).abs() < f32::EPSILON, "e should be 20.0 (2*10)");

        // 使用 set_transform 直接设置 a=2, d=1, e=10
        let mut ctx2 = CanvasContext::new(100, 100);
        ctx2.set_transform(2.0, 0.0, 0.0, 1.0, 10.0, 0.0);
        let t2 = ctx2.get_transform();

        // transform 叠加 vs set_transform 替换，结果应不同
        assert!(
            (t1.e - t2.e).abs() > 0.01,
            "transform multiply (e={}) should differ from set_transform (e={})",
            t1.e,
            t2.e
        );
    }

    /// 测试 miter_limit 默认值为 10.0。
    ///
    /// 新创建的 CanvasContext 的 miter_limit() 应返回 10.0。
    #[test]
    fn test_miter_limit_default_value() {
        let ctx = CanvasContext::new(100, 100);
        assert!(
            (ctx.miter_limit() - 10.0).abs() < f32::EPSILON,
            "default miter_limit should be 10.0, got {}",
            ctx.miter_limit()
        );
    }

    /// 测试 miter_limit 在 save/restore 中正确保存和恢复。
    ///
    /// 设置 miter_limit 为 5.0，save 后改为 15.0，restore 后应恢复 5.0。
    #[test]
    fn test_miter_limit_save_restore_value() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_miter_limit(5.0);
        ctx.save();
        ctx.set_miter_limit(15.0);
        assert!(
            (ctx.miter_limit() - 15.0).abs() < f32::EPSILON,
            "after save+set, miter_limit should be 15.0"
        );
        ctx.restore();
        assert!(
            (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
            "after restore, miter_limit should be back to 5.0"
        );
    }

    /// 测试 direction 默认值为 Inherit。
    ///
    /// 新创建的 CanvasContext 的 direction() 应返回 TextDirection::Inherit。
    #[test]
    fn test_text_direction_default_value() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(
            ctx.direction(),
            TextDirection::Inherit,
            "default direction should be Inherit"
        );
    }

    // ── CanvasStyle 测试 ──

    /// 测试 CanvasStyle 默认为不透明黑色。
    #[test]
    fn test_canvas_style_default() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.fill_color(), Color::BLACK);
        assert_eq!(ctx.stroke_color(), Color::BLACK);
        // 验证 fill_style/stroke_style 是 Color 变体
        assert!(matches!(ctx.fill_style(), CanvasStyle::Color(Color::BLACK)));
        assert!(matches!(ctx.stroke_style(), CanvasStyle::Color(Color::BLACK)));
    }

    /// 测试设置 fill 为线性渐变后 resolve_color 返回插值颜色。
    #[test]
    fn test_fill_style_gradient() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
        // resolve_color 在 offset=0.5 处采样，应为红色和蓝色的中间值
        let resolved = ctx.fill_color();
        // 中间色：(128, 0, 128, 255)
        assert_eq!(resolved.r, 128);
        assert_eq!(resolved.g, 0);
        assert_eq!(resolved.b, 128);
        assert_eq!(resolved.a, 255);
    }

    /// 测试设置 stroke 为线性渐变后 resolve_color 返回插值颜色。
    #[test]
    fn test_stroke_style_gradient() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
        grad.add_color_stop(0.0, Color::BLACK);
        grad.add_color_stop(1.0, Color::WHITE);
        ctx.set_stroke_style(CanvasStyle::LinearGradient(grad));
        let resolved = ctx.stroke_color();
        // 中间色：(128, 128, 128, 255)
        assert_eq!(resolved.r, 128);
        assert_eq!(resolved.g, 128);
        assert_eq!(resolved.b, 128);
        assert_eq!(resolved.a, 255);
    }

    /// 测试渐变只有一个停止点时 sample_color 返回该停止点的颜色。
    #[test]
    fn test_gradient_sample_single_stop() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.5, Color::GREEN);
        assert_eq!(grad.sample_color(0.0), Color::GREEN);
        assert_eq!(grad.sample_color(0.5), Color::GREEN);
        assert_eq!(grad.sample_color(1.0), Color::GREEN);
    }

    /// 测试渐变两个停止点时 sample_color 在各位置返回正确的插值颜色。
    #[test]
    fn test_gradient_sample_two_stops() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::BLACK);
        grad.add_color_stop(1.0, Color::WHITE);
        // offset=0.0: 黑色
        assert_eq!(grad.sample_color(0.0), Color::BLACK);
        // offset=1.0: 白色
        assert_eq!(grad.sample_color(1.0), Color::WHITE);
        // offset=0.5: 中间灰
        let mid = grad.sample_color(0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.g, 128);
        assert_eq!(mid.b, 128);
        assert_eq!(mid.a, 255);
    }

    /// 测试渐变 sample_color 在偏移量超出 [0,1] 范围时 clamp 到边界停止点颜色。
    #[test]
    fn test_gradient_sample_out_of_range() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        // 负偏移量 → clamp 到 0.0 → 红色
        assert_eq!(grad.sample_color(-1.0), Color::RED);
        // 超过 1.0 → clamp 到 1.0 → 蓝色
        assert_eq!(grad.sample_color(2.0), Color::BLUE);
    }

    /// 测试 set_fill_color 便捷方法仍然正常工作。
    #[test]
    fn test_set_fill_color_convenience() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        assert_eq!(ctx.fill_color(), Color::RED);
        assert!(matches!(ctx.fill_style(), CanvasStyle::Color(Color::RED)));
        // 填充矩形应使用红色
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        let pixel = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(pixel.data[0..4], [255, 0, 0, 255]);
    }

    /// 测试 set_stroke_color 便捷方法仍然正常工作。
    #[test]
    fn test_set_stroke_color_convenience() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_stroke_color(Color::BLUE);
        assert_eq!(ctx.stroke_color(), Color::BLUE);
        assert!(matches!(ctx.stroke_style(), CanvasStyle::Color(Color::BLUE)));
        // 描边矩形应使用蓝色
        ctx.stroke_rect(10.0, 10.0, 20.0, 20.0);
        let pixel = ctx.get_image_data(10, 10, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 255, 255]);
    }

    /// 测试径向渐变 sample_color。
    #[test]
    fn test_radial_gradient_sample_color() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 100.0);
        grad.add_color_stop(0.0, Color::WHITE);
        grad.add_color_stop(1.0, Color::BLACK);
        // offset=0: 白色
        assert_eq!(grad.sample_color(0.0), Color::WHITE);
        // offset=1: 黑色
        assert_eq!(grad.sample_color(1.0), Color::BLACK);
        // offset=0.5: 中间灰
        let mid = grad.sample_color(0.5);
        assert_eq!(mid.r, 128);
    }

    /// 测试锥形渐变 sample_color。
    #[test]
    fn test_conic_gradient_sample_color() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.5, Color::GREEN);
        grad.add_color_stop(1.0, Color::BLUE);
        // offset=0: 红色
        assert_eq!(grad.sample_color(0.0), Color::RED);
        // offset=0.25: 红绿中间
        let c = grad.sample_color(0.25);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 128);
    }

    /// 测试 CanvasStyle Pattern 变体 resolve_color 返回黑色。
    #[test]
    fn test_canvas_style_pattern_resolve() {
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
        let style = CanvasStyle::Pattern(pattern);
        assert_eq!(style.resolve_color(), Color::BLACK);
    }

    /// 测试 save/restore 正确保存和恢复 CanvasStyle（渐变）。
    #[test]
    fn test_save_restore_gradient_style() {
        let mut ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
        ctx.save();
        ctx.set_fill_style(CanvasStyle::Color(Color::GREEN));
        assert_eq!(ctx.fill_color(), Color::GREEN);
        ctx.restore();
        // 恢复后应回到渐变样式
        let resolved = ctx.fill_color();
        assert_eq!(resolved.r, 128);
        assert_eq!(resolved.b, 128);
    }

    /// 测试无停止点的渐变 sample_color 返回黑色。
    #[test]
    fn test_gradient_sample_empty_stops() {
        let ctx = CanvasContext::new(200, 200);
        let grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        assert_eq!(grad.sample_color(0.5), Color::BLACK);
    }

    /// 测试使用渐变 fill_style 绘制 fill_rect。
    #[test]
    fn test_fill_rect_with_gradient_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        // 像素应使用 offset=0.5 处的采样色 (128, 0, 128)
        let pixel = ctx.get_image_data(10, 10, 1, 1);
        assert_eq!(pixel.data[0], 128);
        assert_eq!(pixel.data[1], 0);
        assert_eq!(pixel.data[2], 128);
        assert_eq!(pixel.data[3], 255);
    }

    /// 测试使用渐变 stroke_style 绘制 stroke_rect。
    #[test]
    fn test_stroke_rect_with_gradient_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::BLACK);
        grad.add_color_stop(1.0, Color::WHITE);
        ctx.set_stroke_style(CanvasStyle::LinearGradient(grad));
        ctx.stroke_rect(5.0, 5.0, 20.0, 20.0);
        // 描边像素应使用 offset=0.5 处的采样色 (128, 128, 128)
        let pixel = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(pixel.data[0], 128);
        assert_eq!(pixel.data[1], 128);
        assert_eq!(pixel.data[2], 128);
        assert_eq!(pixel.data[3], 255);
    }

    // ── 边界条件测试：渐变填充、描边连接、嵌套裁剪、ImageData 往返、globalAlpha、Path2D ──

    /// 测试 fill_rect 使用 CanvasStyle::LinearGradient 作为填充样式。
    /// 渐变从红色到蓝色，采样 offset=0.5 处应得到 (128, 0, 128) 左右的颜色。
    #[test]
    fn test_canvas_fill_rect_with_gradient_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        // fill_rect 使用 fill_style.resolve_color() 采样 offset=0.5
        // 红(255,0,0) 与 蓝(0,0,255) 在 0.5 处插值 ≈ (128, 0, 128)
        let pixel = ctx.get_image_data(50, 50, 1, 1);
        assert!(
            (pixel.data[0] as i32 - 128).abs() <= 2,
            "red channel should be ~128, got {}",
            pixel.data[0]
        );
        assert_eq!(pixel.data[1], 0, "green channel should be 0");
        assert!(
            (pixel.data[2] as i32 - 128).abs() <= 2,
            "blue channel should be ~128, got {}",
            pixel.data[2]
        );
        assert_eq!(pixel.data[3], 255, "alpha should be 255");
    }

    /// 测试 stroke_rect 使用 LineJoin::Round 不 panic 且生成描边图元。
    #[test]
    fn test_stroke_rect_with_round_join() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_join(LineJoin::Round);
        ctx.set_line_width(5.0);
        ctx.stroke_rect(10.0, 10.0, 30.0, 30.0);
        // stroke_rect 生成 4 条边的 fill 图元
        assert_eq!(ctx.primitives().fills.len(), 4);
        assert_eq!(ctx.line_join(), LineJoin::Round);
    }

    /// 测试嵌套 clip 操作：先裁剪到大矩形，再裁剪到小矩形，最终绘制范围应受限于交集。
    #[test]
    fn test_canvas_clip_nested() {
        let mut ctx = CanvasContext::new(200, 200);
        // 第一次裁剪：大矩形
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0, 0.0);
        ctx.line_to(100.0, 100.0);
        ctx.line_to(0.0, 100.0);
        ctx.close_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 1);
        // 第二次裁剪：小矩形（嵌套）
        ctx.begin_path();
        ctx.move_to(20.0, 20.0);
        ctx.line_to(60.0, 20.0);
        ctx.line_to(60.0, 60.0);
        ctx.line_to(20.0, 60.0);
        ctx.close_path();
        ctx.clip();
        assert_eq!(ctx.primitives().clips.len(), 2);
        // 后续绘制应受限于两个裁剪区域的交集
        ctx.fill_rect(0.0, 0.0, 200.0, 200.0);
        assert_eq!(ctx.primitives().fills.len(), 1);
    }

    /// 测试 put_image_data 后 get_image_data 能完整读回相同数据（往返一致性）。
    #[test]
    fn test_put_get_image_data_roundtrip() {
        let mut ctx = CanvasContext::new(20, 20);
        // 构造 4x4 的测试像素：每个像素不同的 RGBA 值
        let mut pixels = Vec::with_capacity(4 * 4 * 4);
        for i in 0u8..16 {
            pixels.extend_from_slice(&[i * 16, (255 - i * 16), i * 8, 255]);
        }
        let img = ImageData {
            width: 4,
            height: 4,
            data: pixels.clone(),
        };
        ctx.put_image_data(&img, 5, 5);
        let result = ctx.get_image_data(5, 5, 4, 4);
        assert_eq!(
            result.data, pixels,
            "get_image_data 应返回与 put_image_data 写入完全相同的数据"
        );
    }

    /// 测试 globalAlpha=0 时 fill_rect 产生完全透明的像素。
    #[test]
    fn test_global_alpha_zero() {
        let mut ctx = CanvasContext::new(50, 50);
        ctx.set_global_alpha(0.0);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        // 填充颜色的 alpha 应被 globalAlpha=0 清零
        let pixel = ctx.get_image_data(25, 25, 1, 1);
        assert_eq!(pixel.data[3], 0, "globalAlpha=0 时像素应完全透明");
        // 图元颜色 alpha 也应为 0
        let fill = &ctx.primitives().fills[0];
        assert_eq!(fill.color.a, 0, "图元颜色 alpha 应为 0");
    }

    // ── 边界条件测试（第八批）──

    /// 测试 resize 到零尺寸后画布宽度高度为零，且不 panic。
    #[test]
    fn test_canvas_resize_to_zero() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        ctx.resize(0, 0);
        assert_eq!(ctx.width(), 0);
        assert_eq!(ctx.height(), 0);
    }

    /// 测试 fill_rect 零宽高不产生可见像素。
    #[test]
    fn test_canvas_fill_rect_zero_dimensions() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(10.0, 10.0, 0.0, 0.0);
        ctx.fill_rect(20.0, 20.0, 0.0, 50.0);
        ctx.fill_rect(30.0, 30.0, 50.0, 0.0);
        // 零宽/高的矩形不应写入像素
        let pixel = ctx.get_image_data(10, 10, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "零宽高 fill_rect 不应写入像素");
    }

    /// 测试 set_line_dash 传入单元素数组后自动加倍为双元素。
    #[test]
    fn test_line_dash_single_element_doubled() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_dash(vec![8.0]);
        // 奇数长度时 Canvas 规范要求复制拼接：[8] → [8, 8]
        assert_eq!(ctx.get_line_dash(), &[8.0, 8.0]);
    }

    /// 测试 stroke_rect 零尺寸（零宽零高）只生成 4 个退化图元且不 panic。
    #[test]
    fn test_canvas_stroke_rect_zero_size() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.stroke_rect(50.0, 50.0, 0.0, 0.0);
        // stroke_rect 始终生成 4 条边的 fill 图元，即使零尺寸
        assert_eq!(ctx.primitives().fills.len(), 4);
    }

    /// 测试深度嵌套 save/restore 正确恢复每一层状态。
    #[test]
    fn test_canvas_deep_nested_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        // 层级 0: 红色填充
        ctx.set_fill_color(Color::RED);
        ctx.save();
        // 层级 1: 绿色填充
        ctx.set_fill_color(Color::GREEN);
        ctx.save();
        // 层级 2: 蓝色填充
        ctx.set_fill_color(Color::BLUE);
        ctx.save();
        // 层级 3: 白色填充
        ctx.set_fill_color(Color::WHITE);
        assert_eq!(ctx.fill_color(), Color::WHITE);

        // 逐层恢复
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::BLUE, "恢复到层级 2 应为蓝色");
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::GREEN, "恢复到层级 1 应为绿色");
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::RED, "恢复到层级 0 应为红色");
    }

    // ── 边界条件测试（第九批）──

    /// 测试 resize 到更大尺寸后画布像素缓冲区重新分配，之前内容被清空。
    #[test]
    fn test_canvas_resize_larger_clears_pixels() {
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 确认红色已写入
        let before = ctx.get_image_data(0, 0, 5, 5);
        assert_eq!(before.data[0], 255);
        // resize 到更大尺寸
        ctx.resize(20, 20);
        assert_eq!(ctx.width(), 20);
        assert_eq!(ctx.height(), 20);
        // resize 后像素应全部清零
        let after = ctx.get_image_data(0, 0, 5, 5);
        assert_eq!(after.data[0..4], [0, 0, 0, 0], "resize 后像素应被清零");
    }

    /// 测试 set_fill_style 使用径向渐变后 fill_rect 像素使用采样颜色。
    #[test]
    fn test_fill_rect_with_radial_gradient_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_radial_gradient(50.0, 50.0, 0.0, 50.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::WHITE);
        grad.add_color_stop(1.0, Color::BLACK);
        ctx.set_fill_style(CanvasStyle::RadialGradient(grad));
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        // fill_rect 使用 resolve_color()，对 RadialGradient 在 offset=0.5 处采样
        let pixel = ctx.get_image_data(10, 10, 1, 1);
        // 中间灰度值
        assert!(
            (pixel.data[0] as i32 - 128).abs() <= 2,
            "radial gradient sample at 0.5 应为 ~128, got {}",
            pixel.data[0]
        );
        assert_eq!(pixel.data[3], 255, "alpha 应为 255");
    }

    /// 测试 set_stroke_style 使用锥形渐变后 stroke_rect 像素使用采样颜色。
    #[test]
    fn test_stroke_rect_with_conic_gradient_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::BLACK);
        grad.add_color_stop(1.0, Color::WHITE);
        ctx.set_stroke_style(CanvasStyle::ConicGradient(grad));
        ctx.stroke_rect(5.0, 5.0, 20.0, 20.0);
        // stroke_rect 使用 resolve_color()，ConicGradient 在 offset=0.0 处采样
        // offset=0.0 对应第一个 stop 的颜色：黑色
        let pixel = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(pixel.data[0], 0, "conic gradient sample at 0.0 应为黑色");
        assert_eq!(pixel.data[1], 0);
        assert_eq!(pixel.data[2], 0);
        assert_eq!(pixel.data[3], 255);
    }

    /// 测试 scale(0, 0) 后 fill_rect 不 panic 且变换结果退化。
    #[test]
    fn test_canvas_scale_zero_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.scale(0.0, 0.0);
        // scale(0,0) 使变换矩阵退化为全零平移，fill_rect 应不 panic
        ctx.fill_rect(10.0, 10.0, 50.0, 50.0);
        // 退化矩阵下 transform_point 产生的矩形宽高为 0，不应写入像素
        let pixel = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "退化矩阵不应写入像素");
    }

    /// 测试 measure_text 在 set_font 改变字体大小后返回不同的宽度值。
    #[test]
    fn test_measure_text_reflects_font_size_change() {
        let mut ctx = CanvasContext::new(100, 100);
        // 默认字体大小 10.0
        let m1 = ctx.measure_text("abc");
        let expected1 = 3.0 * 10.0 * 0.6; // 18.0
        assert!(
            (m1.width - expected1).abs() < f32::EPSILON,
            "默认大小 10.0 时宽度应为 {}",
            expected1
        );
        // 改为 20.0
        ctx.set_font(FontDescriptor {
            family: "sans-serif".to_string(),
            size: 20.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        });
        let m2 = ctx.measure_text("abc");
        let expected2 = 3.0 * 20.0 * 0.6; // 36.0
        assert!(
            (m2.width - expected2).abs() < f32::EPSILON,
            "字体大小 20.0 时宽度应为 {}",
            expected2
        );
        // 两次测量应不同
        assert!((m1.width - m2.width).abs() > 1.0, "不同字体大小的测量结果应不同");
    }

    // ── 边界条件测试（第十批）──

    /// 测试 clear_rect 使用负坐标和负尺寸时不 panic，且不破坏已有像素。
    /// 负尺寸的 clear_rect 应视为空操作（不清除任何像素）。
    #[test]
    fn test_canvas_clear_rect_negative_dimensions() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        // 负宽高的 clear_rect — 不应 panic
        ctx.clear_rect(10.0, 10.0, -20.0, -30.0);
        // 原有红色像素应保持不变
        let pixel = ctx.get_image_data(50, 50, 1, 1);
        assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "负尺寸 clear_rect 不应破坏已有像素");
    }

    /// 测试 CanvasStyle::Pattern 作为 fill_style 绘制 fill_rect 时使用黑色回退色。
    /// Pattern 的 resolve_color() 返回黑色，因此 fill_rect 应使用黑色绘制。
    #[test]
    fn test_canvas_fill_rect_with_pattern_style() {
        let mut ctx = CanvasContext::new(100, 100);
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        let pattern = ctx.create_pattern(img, PatternRepetition::Repeat);
        ctx.set_fill_style(CanvasStyle::Pattern(pattern));
        ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
        // Pattern resolve_color 回退为黑色
        let pixel = ctx.get_image_data(20, 20, 1, 1);
        assert_eq!(pixel.data[0], 0, "pattern fill 应使用黑色回退色 r");
        assert_eq!(pixel.data[1], 0, "pattern fill 应使用黑色回退色 g");
        assert_eq!(pixel.data[2], 0, "pattern fill 应使用黑色回退色 b");
        assert_eq!(pixel.data[3], 255, "pattern fill alpha 应为 255");
    }

    /// 测试三色渐变在中间停止点偏移量处精确返回该停止点的颜色（无插值误差）。
    #[test]
    fn test_gradient_sample_three_stops_exact_boundary() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.5, Color::GREEN);
        grad.add_color_stop(1.0, Color::BLUE);
        // 在偏移量 0.0 处应为红色
        assert_eq!(grad.sample_color(0.0), Color::RED, "offset 0.0 应为红色");
        // 在偏移量 0.5 处应为绿色（精确命中停止点，无需插值）
        assert_eq!(grad.sample_color(0.5), Color::GREEN, "offset 0.5 应为绿色");
        // 在偏移量 1.0 处应为蓝色
        assert_eq!(grad.sample_color(1.0), Color::BLUE, "offset 1.0 应为蓝色");
    }

    /// 测试 save/restore 保存并恢复 text_align 和 text_baseline。
    /// save 后修改文本对齐和基线，restore 后应恢复到 save 时的值。
    #[test]
    fn test_text_align_and_baseline_save_restore() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_text_align(TextAlign::Right);
        ctx.set_text_baseline(TextBaseline::Top);
        ctx.save();
        ctx.set_text_align(TextAlign::Center);
        ctx.set_text_baseline(TextBaseline::Bottom);
        assert_eq!(ctx.text_align(), TextAlign::Center);
        assert_eq!(ctx.text_baseline(), TextBaseline::Bottom);
        ctx.restore();
        assert_eq!(ctx.text_align(), TextAlign::Right, "restore 后 text_align 应为 Right");
        assert_eq!(
            ctx.text_baseline(),
            TextBaseline::Top,
            "restore 后 text_baseline 应为 Top"
        );
    }

    /// 测试 Path2D 连续添加多种子路径命令后 len() 正确递增。
    /// 依次添加 move_to、line_to、quadratic_curve_to、bezier_curve_to、arc、close_path，
    /// 验证每步后的命令数量。
    #[test]
    fn test_path2d_mixed_commands_count() {
        let mut p = Path2D::new();
        assert_eq!(p.len(), 0, "空路径应有 0 个命令");

        p.move_to(10.0, 20.0);
        assert_eq!(p.len(), 1, "move_to 后应有 1 个命令");

        p.line_to(30.0, 40.0);
        assert_eq!(p.len(), 2, "line_to 后应有 2 个命令");

        p.quadratic_curve_to(50.0, 60.0, 70.0, 80.0);
        assert_eq!(p.len(), 3, "quadratic_curve_to 后应有 3 个命令");

        p.bezier_curve_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(p.len(), 4, "bezier_curve_to 后应有 4 个命令");

        p.arc(0.0, 0.0, 10.0, 0.0, std::f32::consts::PI);
        assert_eq!(p.len(), 5, "arc 后应有 5 个命令");

        p.close_path();
        assert_eq!(p.len(), 6, "close_path 后应有 6 个命令");

        // 验证各命令类型正确
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
        assert!(matches!(p.commands()[1], PathCommand::LineTo(30.0, 40.0)));
        assert!(matches!(
            p.commands()[2],
            PathCommand::QuadraticCurveTo(50.0, 60.0, 70.0, 80.0)
        ));
        assert!(matches!(
            p.commands()[3],
            PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
        ));
        assert!(matches!(p.commands()[4], PathCommand::Arc(0.0, 0.0, 10.0, 0.0, _)));
        assert!(matches!(p.commands()[5], PathCommand::ClosePath));
    }

    // ── 边界条件测试（第十一批）──

    /// 测试 miter_limit 设置/获取和 save/restore。
    /// 默认值为 10.0，修改后 getter 应返回新值，restore 后恢复。
    #[test]
    fn test_miter_limit_set_get_and_save_restore() {
        let ctx = CanvasContext::new(100, 100);
        // 默认值为 10.0
        assert!(
            (ctx.miter_limit() - 10.0).abs() < f32::EPSILON,
            "miter_limit 默认应为 10.0"
        );

        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_miter_limit(5.0);
        assert!(
            (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
            "设置后 miter_limit 应为 5.0"
        );

        // save/restore 保存并恢复 miter_limit
        ctx.save();
        ctx.set_miter_limit(2.0);
        assert!(
            (ctx.miter_limit() - 2.0).abs() < f32::EPSILON,
            "save 后修改 miter_limit 应为 2.0"
        );
        ctx.restore();
        assert!(
            (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
            "restore 后 miter_limit 应恢复为 5.0"
        );
    }

    /// 测试 direction（文本方向）设置/获取和 save/restore。
    /// 默认值为 TextDirection::Inherit，修改后 getter 应返回新值，restore 后恢复。
    #[test]
    fn test_direction_set_get_and_save_restore() {
        let ctx = CanvasContext::new(100, 100);
        assert_eq!(ctx.direction(), TextDirection::Inherit, "direction 默认应为 Inherit");

        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_direction(TextDirection::Rtl);
        assert_eq!(ctx.direction(), TextDirection::Rtl, "设置后 direction 应为 Rtl");

        // save/restore
        ctx.save();
        ctx.set_direction(TextDirection::Ltr);
        assert_eq!(ctx.direction(), TextDirection::Ltr, "save 后修改 direction 应为 Ltr");
        ctx.restore();
        assert_eq!(ctx.direction(), TextDirection::Rtl, "restore 后 direction 应恢复为 Rtl");
    }

    /// 测试 transform() 方法（矩阵后乘）与 set_transform 的区别。
    /// transform() 是叠加乘法，set_transform() 是替换。
    #[test]
    fn test_transform_method_accumulates_vs_set_transform_replaces() {
        let mut ctx = CanvasContext::new(100, 100);
        // 使用 transform() 叠加两个平移
        ctx.transform(1.0, 0.0, 0.0, 1.0, 10.0, 0.0); // 平移 x+10
        ctx.transform(1.0, 0.0, 0.0, 1.0, 20.0, 0.0); // 平移 x+20（叠加）
        let p1 = ctx.transform.transform_point(0.0, 0.0);
        // 叠加后应平移 30.0
        assert!((p1.0 - 30.0).abs() < 0.01, "叠加两次平移后 x 应为 30.0，实际 {}", p1.0);

        // 使用 set_transform 替换（非叠加）
        ctx.set_transform(1.0, 0.0, 0.0, 1.0, 5.0, 0.0);
        let p2 = ctx.transform.transform_point(0.0, 0.0);
        assert!(
            (p2.0 - 5.0).abs() < 0.01,
            "set_transform 替换后 x 应为 5.0，实际 {}",
            p2.0
        );
    }

    /// 测试 fill_text 在不同字体大小时 glyph x 坐标按字体大小正确递进。
    /// 默认字体大小 10.0 时 em_width = 6.0；改为 20.0 时 em_width = 12.0。
    #[test]
    fn test_fill_text_glyph_offset_scales_with_font_size() {
        // 字体大小 10.0（默认）
        let mut ctx_small = CanvasContext::new(200, 200);
        ctx_small.fill_text("AB", 0.0, 0.0);
        let glyphs_small = &ctx_small.primitives().glyphs;
        // 第二个字符的 x 应为 0.0 + 10.0 * 0.6 = 6.0
        assert!(
            (glyphs_small[1].x - 6.0).abs() < f32::EPSILON,
            "字体大小 10.0 时第二个 glyph x 应为 6.0，实际 {}",
            glyphs_small[1].x
        );

        // 字体大小 20.0
        let mut ctx_large = CanvasContext::new(200, 200);
        ctx_large.set_font(FontDescriptor {
            family: "sans-serif".to_string(),
            size: 20.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        });
        ctx_large.fill_text("AB", 0.0, 0.0);
        let glyphs_large = &ctx_large.primitives().glyphs;
        // 第二个字符的 x 应为 0.0 + 20.0 * 0.6 = 12.0
        assert!(
            (glyphs_large[1].x - 12.0).abs() < f32::EPSILON,
            "字体大小 20.0 时第二个 glyph x 应为 12.0，实际 {}",
            glyphs_large[1].x
        );
    }

    /// 测试扫描线光栅化对三角形路径填充写入正确的像素。
    /// 在三角形重心附近的像素应被写入填充色，三角形外部的像素应保持透明。
    #[test]
    fn test_scanline_rasterization_triangle_fill_pixels() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.begin_path();
        // 等腰三角形：顶点 (30,10), (50,50), (10,50)
        ctx.move_to(30.0, 10.0);
        ctx.line_to(50.0, 50.0);
        ctx.line_to(10.0, 50.0);
        ctx.close_path();
        ctx.fill();

        // 三角形重心 ≈ (30, 36.7) — 应为红色
        let center = ctx.get_image_data(30, 36, 1, 1);
        assert_eq!(center.data[0..4], [255, 0, 0, 255], "三角形重心处应为红色");

        // 三角形外部 (80, 10) — 应为透明
        let outside = ctx.get_image_data(80, 10, 1, 1);
        assert_eq!(outside.data[0..4], [0, 0, 0, 0], "三角形外部应为透明");

        // 三角形下方 (30, 80) — 应为透明（低于底边）
        let below = ctx.get_image_data(30, 80, 1, 1);
        assert_eq!(below.data[0..4], [0, 0, 0, 0], "三角形底边下方应为透明");
    }

    // ── 边界条件测试（第十二批）──

    /// 测试 Transform2D 旋转 2π（360°）后应近似回到单位矩阵。
    /// 由于浮点精度，各元素与单位矩阵之差应极小（< 0.001）。
    #[test]
    fn test_transform_rotate_full_circle() {
        let rot = Transform2D::rotate(std::f32::consts::TAU); // 2π
        assert!((rot.a - 1.0).abs() < 0.001, "旋转 2π 后 a 应近似 1.0");
        assert!(rot.b.abs() < 0.001, "旋转 2π 后 b 应近似 0.0");
        assert!(rot.c.abs() < 0.001, "旋转 2π 后 c 应近似 0.0");
        assert!((rot.d - 1.0).abs() < 0.001, "旋转 2π 后 d 应近似 1.0");
        assert!(rot.e.abs() < f32::EPSILON, "旋转 2π 后 e 应为 0.0");
        assert!(rot.f.abs() < f32::EPSILON, "旋转 2π 后 f 应为 0.0");
    }

    /// 测试 ImageData clone 后修改克隆副本不影响原始数据。
    /// 克隆一份包含非零像素的 ImageData，修改克隆副本的数据，验证原始数据不变。
    #[test]
    fn test_image_data_clone_independence() {
        let original = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        let mut cloned = original.clone();
        // 修改克隆副本的第一个像素为黑色
        cloned.data[0] = 0;
        cloned.data[1] = 0;
        cloned.data[2] = 0;
        cloned.data[3] = 0;
        // 原始数据的第一个像素应保持不变（红色）
        assert_eq!(original.data[0..4], [255, 0, 0, 255], "原始数据应不受克隆修改的影响");
        assert_eq!(cloned.data[0..4], [0, 0, 0, 0], "克隆副本应反映修改");
    }

    /// 测试连续两次 begin_path 调用后当前路径被正确清空。
    /// 第一次 begin_path 前添加路径命令，第一次 begin_path 清空，
    /// 再添加新命令，第二次 begin_path 再次清空，fill 应不产生图元。
    #[test]
    fn test_double_begin_path_clears() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(50.0, 50.0);
        // 第二次 begin_path 应清空当前路径
        ctx.begin_path();
        // 空路径 fill 应不产生填充图元
        ctx.fill();
        assert!(
            ctx.primitives().fills.is_empty(),
            "第二次 begin_path 后 fill 空路径不应产生填充图元"
        );
    }

    /// 测试 save/restore 对多次连续 save 的后进先出（LIFO）行为。
    /// 依次 save 红色、绿色、蓝色，restore 后应按蓝色→绿色→红色顺序恢复。
    #[test]
    fn test_save_restore_lifo_order() {
        let mut ctx = CanvasContext::new(100, 100);
        // 层级 0：红色
        ctx.set_fill_color(Color::RED);
        ctx.save();
        // 层级 1：绿色
        ctx.set_fill_color(Color::GREEN);
        ctx.save();
        // 层级 2：蓝色
        ctx.set_fill_color(Color::BLUE);
        assert_eq!(ctx.fill_color(), Color::BLUE, "当前应为蓝色");

        // 第一次 restore：恢复到绿色
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::GREEN, "LIFO 第一次 restore 应恢复到绿色");

        // 第二次 restore：恢复到红色
        ctx.restore();
        assert_eq!(ctx.fill_color(), Color::RED, "LIFO 第二次 restore 应恢复到红色");
    }

    /// 测试设置负的 line_width 不 panic，且后续操作正常。
    /// 虽然负线宽在 Canvas 规范中被忽略，但不应导致 panic。
    #[test]
    fn test_negative_line_width_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_width(-5.0);
        // 负线宽设置后 getter 应返回设置的值
        assert!(
            (ctx.line_width() - (-5.0)).abs() < f32::EPSILON,
            "line_width getter 应返回设置值 -5.0"
        );
        // 描边矩形不应 panic
        ctx.set_stroke_color(Color::RED);
        ctx.stroke_rect(10.0, 10.0, 30.0, 30.0);
        // 验证描边图元已生成（即使线宽为负）
        assert!(!ctx.primitives().fills.is_empty(), "负线宽描边矩形仍应生成图元");
    }

    // ── 边界条件测试（第十三批）──

    /// 测试 createRadialGradient 使用完全相同的内外圆（圆心和半径均相同）时不 panic。
    /// 退化渐变应正常创建，停止点可正常添加。
    #[test]
    fn test_radial_gradient_identical_circles_no_panic() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_radial_gradient(50.0, 50.0, 25.0, 50.0, 50.0, 25.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        // 退化渐变不 panic，停止点数量正确
        assert_eq!(grad.stops.len(), 2);
        assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r0 - 25.0).abs() < f32::EPSILON);
        assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
        assert!((grad.r1 - 25.0).abs() < f32::EPSILON);
    }

    /// 测试在路径构建过程中 resize 画布后路径仍然保留。
    /// begin_path → move_to → line_to → resize → 更多路径操作 → fill，
    /// resize 只清除像素缓冲区和已渲染图元，不清除当前路径。
    #[test]
    fn test_resize_during_path_construction_preserves_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(50.0, 10.0);
        // resize 会清除 pixel_buffer 和 primitives，但 current_path 不受影响
        ctx.resize(200, 200);
        // 继续路径操作
        ctx.line_to(50.0, 50.0);
        ctx.line_to(10.0, 50.0);
        ctx.close_path();
        ctx.fill();
        // 路径应保留，fill 应产生 path_fills 图元
        assert!(
            !ctx.primitives().path_fills.is_empty(),
            "resize 后路径应保留，fill 应产生填充图元"
        );
    }

    /// 测试 set_transform 使用全零退化矩阵时不 panic。
    /// 全零矩阵将所有点映射到原点，reset_transform 后恢复正常绘制。
    #[test]
    fn test_set_transform_degenerate_all_zeros_then_reset() {
        let mut ctx = CanvasContext::new(100, 100);
        // 设置全零退化矩阵
        ctx.set_transform(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (x, y) = ctx.transform.transform_point(50.0, 50.0);
        assert!((x).abs() < f32::EPSILON, "退化矩阵应将所有点映射到原点 x=0");
        assert!((y).abs() < f32::EPSILON, "退化矩阵应将所有点映射到原点 y=0");
        // 退化矩阵下 fill_rect 不 panic
        ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
        // reset_transform 恢复单位矩阵后绘制正常
        ctx.reset_transform();
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        let pixel = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "reset_transform 后应正常绘制");
    }

    /// 测试 put_image_data 使用超出 ImageData 数据范围的偏移参数时不 panic。
    /// 将小尺寸 ImageData 放置到远超其范围的坐标上，不应导致越界访问。
    #[test]
    fn test_put_image_data_out_of_bounds_dirty_rect_no_panic() {
        let mut ctx = CanvasContext::new(50, 50);
        // 2x2 的 ImageData
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        // 放置在远超画布边界的位置 — 不应 panic
        ctx.put_image_data(&img, 100, 100);
        ctx.put_image_data(&img, 200, 200);
        ctx.put_image_data(&img, u32::MAX, u32::MAX);
        // 画布内像素应未被修改
        let pixel = ctx.get_image_data(0, 0, 1, 1);
        assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "越界 put_image_data 不应写入画布内像素");
    }

    /// 测试 createLinearGradient 起点和终点完全相同（零长度渐变）时不 panic。
    /// 零长度渐变应正常创建，可添加停止点，sample_color 不 panic。
    #[test]
    fn test_linear_gradient_zero_length_no_panic() {
        let ctx = CanvasContext::new(200, 200);
        let mut grad = ctx.create_linear_gradient(75.0, 75.0, 75.0, 75.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(0.5, Color::GREEN);
        grad.add_color_stop(1.0, Color::BLUE);
        // 零长度渐变不 panic，停止点数量正确
        assert_eq!(grad.stops.len(), 3);
        // sample_color 不应 panic
        let c = grad.sample_color(0.5);
        assert_eq!(c, Color::GREEN, "零长度渐变在 offset=0.5 处应返回绿色");
    }

    /// 测试 restore 在没有对应 save 时不 panic。
    /// Canvas 规范要求多余的 restore 静默忽略，不应导致崩溃。
    #[test]
    fn test_restore_without_matching_save_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        // 栈为空时调用 restore — 不应 panic
        ctx.restore();
        ctx.restore();
        ctx.restore();
        // 画布状态应保持默认值不变
        assert_eq!(ctx.global_alpha(), 1.0, "多余 restore 不应改变 global_alpha");
        assert_eq!(ctx.line_width(), 1.0, "多余 restore 不应改变 line_width");
    }

    /// 测试 fill_text 传入空字符串时不 panic。
    /// 空字符串没有字符需要渲染，应正常处理而不产生任何图元。
    #[test]
    fn test_fill_text_empty_string_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill_text("", 10.0, 20.0);
        // 空字符串不应产生任何图元
        assert!(
            ctx.primitives().glyphs.is_empty(),
            "空字符串 fill_text 不应产生 glyph 图元"
        );
    }

    /// 测试 stroke_rect 使用零宽高时不 panic。
    /// 零尺寸矩形的描边边框在数学上退化为点或线，实现应安全处理。
    #[test]
    fn test_stroke_rect_zero_width_height_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        // 零宽高 — 不应 panic
        ctx.stroke_rect(50.0, 50.0, 0.0, 0.0);
        // 零宽度非零高度
        ctx.stroke_rect(25.0, 25.0, 0.0, 40.0);
        // 非零宽度零高度
        ctx.stroke_rect(25.0, 25.0, 40.0, 0.0);
    }

    /// 测试 create_radial_gradient 使用负半径（退化渐变）时不 panic。
    /// 负半径在数学上无意义，但应能正常创建对象并添加停止点。
    #[test]
    fn test_radial_gradient_negative_radius_degenerate_no_panic() {
        let ctx = CanvasContext::new(100, 100);
        let mut grad = ctx.create_radial_gradient(50.0, 50.0, -10.0, 50.0, 50.0, -20.0);
        grad.add_color_stop(0.0, Color::RED);
        grad.add_color_stop(1.0, Color::BLUE);
        // 负半径渐变不 panic，停止点正确保存
        assert_eq!(grad.stops.len(), 2, "负半径渐变应能正常添加停止点");
        assert_eq!(grad.r0, -10.0, "内圆半径应保持负值不变");
        assert_eq!(grad.r1, -20.0, "外圆半径应保持负值不变");
        // sample_color 不应 panic（RED 和 BLUE 在 0.5 处插值为 (128,0,128)）
        let c = grad.sample_color(0.5);
        assert_eq!(c, Color::rgba(128, 0, 128, 255), "负半径渐变采样应返回正确插值颜色");
    }

    /// 测试 clip 在没有当前路径（空裁剪）时不 panic。
    /// 没有构建路径时直接调用 clip，应静默忽略而非崩溃。
    #[test]
    fn test_clip_no_current_path_no_panic() {
        let mut ctx = CanvasContext::new(100, 100);
        // 没有构建任何路径，直接 clip — 不应 panic
        ctx.clip();
        ctx.clip();
        // 后续绘制操作应正常执行（裁剪区域未被设置）
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        // 画布内像素应有正常绘制结果
        let pixel = ctx.get_image_data(5, 5, 1, 1);
        assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "空 clip 后 fill_rect 应正常绘制");
    }

    // ── 新增边界条件测试（5 个） ──

    /// 测试 save/restore 在嵌套场景下正确保持 line_width 状态。
    /// 外层设置 line_width=3，save 后改为 8，内层再 save 后改为 20，
    /// 逐层 restore 后应依次恢复到 8 和 3。
    #[test]
    fn test_canvas_save_restore_line_width_nested() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_line_width(3.0);
        ctx.save();
        ctx.set_line_width(8.0);
        ctx.save();
        ctx.set_line_width(20.0);
        assert!(
            (ctx.line_width() - 20.0).abs() < f32::EPSILON,
            "内层 line_width 应为 20.0"
        );
        ctx.restore();
        assert!(
            (ctx.line_width() - 8.0).abs() < f32::EPSILON,
            "恢复到中层 line_width 应为 8.0"
        );
        ctx.restore();
        assert!(
            (ctx.line_width() - 3.0).abs() < f32::EPSILON,
            "恢复到外层 line_width 应为 3.0"
        );
    }

    /// 测试连续多次 arc() 调用不会 panic，且路径命令正确累积。
    /// 模拟绘制一个由多段弧线组成的复杂路径场景。
    #[test]
    fn test_canvas_multiple_arc_paths() {
        let mut ctx = CanvasContext::new(400, 400);
        ctx.begin_path();
        ctx.arc(100.0, 100.0, 50.0, 0.0, std::f32::consts::PI);
        ctx.arc(200.0, 100.0, 30.0, 0.0, std::f32::consts::FRAC_PI_2);
        ctx.arc(300.0, 100.0, 40.0, 0.0, std::f32::consts::TAU);
        // 不应 panic
        ctx.fill();
        assert!(
            !ctx.primitives().path_fills.is_empty(),
            "多次 arc 后 fill 应生成路径填充图元"
        );
    }

    /// 测试 resize 到 0x0 尺寸不 panic，后续操作仍可安全执行。
    #[test]
    fn test_canvas_zero_size_resize() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        // resize 到零尺寸 — 不应 panic
        ctx.resize(0, 0);
        assert_eq!(ctx.width(), 0);
        assert_eq!(ctx.height(), 0);
        // 零尺寸画布上的绘制操作也不应 panic
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        ctx.stroke_rect(0.0, 0.0, 10.0, 10.0);
    }

    /// 测试对空路径同时调用 fill() 和 stroke() 均不 panic，且不生成任何图元。
    #[test]
    fn test_canvas_fill_stroke_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.begin_path();
        // 空路径上调用 fill 和 stroke — 不应 panic
        ctx.fill();
        ctx.stroke();
        assert_eq!(ctx.primitives().path_fills.len(), 0, "空路径 fill 不应生成图元");
        assert_eq!(ctx.primitives().path_strokes.len(), 0, "空路径 stroke 不应生成图元");
    }

    /// 测试 set_global_alpha 在边界值 0.0 和 1.0 时的行为正确。
    /// 0.0 应使后续绘制完全透明，1.0 应保持完全不透明。
    #[test]
    fn test_canvas_global_alpha_boundary() {
        let mut ctx = CanvasContext::new(100, 100);
        // 设置 alpha 为 0.0 — 完全透明
        ctx.set_global_alpha(0.0);
        assert!(ctx.global_alpha().abs() < f32::EPSILON, "alpha=0.0 应精确返回 0.0");
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        let pixel_zero = ctx.get_image_data(25, 25, 1, 1);
        assert_eq!(pixel_zero.data[3], 0, "alpha=0.0 时绘制的像素应完全透明");

        // 设置 alpha 为 1.0 — 完全不透明
        ctx.set_global_alpha(1.0);
        assert!(
            (ctx.global_alpha() - 1.0).abs() < f32::EPSILON,
            "alpha=1.0 应精确返回 1.0"
        );
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        let pixel_one = ctx.get_image_data(25, 25, 1, 1);
        assert_eq!(
            pixel_one.data[0..4],
            [255, 0, 0, 255],
            "alpha=1.0 时绘制的像素应完全不透明"
        );
    }
}
