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
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
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
        Self {
            image_data,
            repetition,
        }
    }
}

/// Canvas 状态（用于 save/restore）。
#[derive(Debug, Clone)]
struct CanvasState {
    fill_color: Color,
    stroke_color: Color,
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
}

/// Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。
pub struct CanvasContext {
    /// 画布宽度。
    width: u32,
    /// 画布高度。
    height: u32,
    /// 当前填充颜色。
    fill_color: Color,
    /// 当前描边颜色。
    stroke_color: Color,
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
            fill_color: Color::BLACK,
            stroke_color: Color::BLACK,
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
        }
    }

    // ── Rectangle drawing ──

    /// 清除矩形区域（设为透明）。
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 添加一个透明色填充来表示清除操作
        let rect = self.transform_rect(x, y, width, height);
        self.primitives.add_fill(rect, Color::TRANSPARENT);
        self.blit_rect_to_pixels(&rect, Color::TRANSPARENT);
    }

    /// 填充矩形。
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = self.transform_rect(x, y, width, height);
        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            self.draw_shadow_rect(&rect);
        }
        let color = self.apply_alpha(self.fill_color);
        self.primitives.add_fill(rect, color);
        self.blit_rect_to_pixels(&rect, color);
    }

    /// 描边矩形。
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 简化实现：用描边颜色填充一个薄矩形表示描边
        let lw = self.line_width;
        let color = self.apply_alpha(self.stroke_color);

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
        let color = self.apply_alpha(self.fill_color);
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
        let color = self.apply_alpha(self.stroke_color);
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
        self.current_path
            .arc(tx, ty, radius, start_angle, end_angle);
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
            .push(PathCommand::BezierCurveTo(
                tcp1x, tcp1y, tcp2x, tcp2y, tx, ty,
            ));
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
        let color = self.apply_alpha(self.fill_color);
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
        let color = self.apply_alpha(self.stroke_color);
        let closed = self.current_path.commands().iter().any(|c| matches!(c, PathCommand::ClosePath));
        self.primitives.add_path_stroke(vertices.clone(), color, self.line_width, closed);
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
        let color = self.apply_alpha(self.fill_color);
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
        let color = self.apply_alpha(self.stroke_color);
        let closed = path.commands().iter().any(|c| matches!(c, PathCommand::ClosePath));
        self.primitives.add_path_stroke(vertices.clone(), color, self.line_width, closed);
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
            fill_color: self.fill_color,
            stroke_color: self.stroke_color,
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
        });
    }

    /// 从栈恢复状态。
    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.fill_color = state.fill_color;
            self.stroke_color = state.stroke_color;
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

    // ── Properties ──

    /// 设置填充颜色。
    pub fn set_fill_color(&mut self, color: Color) {
        self.fill_color = color;
    }

    /// 设置描边颜色。
    pub fn set_stroke_color(&mut self, color: Color) {
        self.stroke_color = color;
    }

    /// 设置线宽。
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }

    /// 设置字体。
    pub fn set_font(&mut self, font: FontDescriptor) {
        self.font = font;
    }

    /// 设置全局透明度。
    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.global_alpha = alpha.clamp(0.0, 1.0);
    }

    /// 返回当前填充颜色。
    pub fn fill_color(&self) -> &Color {
        &self.fill_color
    }

    /// 返回当前描边颜色。
    pub fn stroke_color(&self) -> &Color {
        &self.stroke_color
    }

    /// 返回当前线宽。
    pub fn line_width(&self) -> f32 {
        self.line_width
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
    pub fn create_radial_gradient(
        &self,
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
    ) -> RadialGradient {
        RadialGradient::new(x0, y0, r0, x1, y1, r1)
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
        let points: Vec<(f32, f32)> = vertices
            .chunks_exact(2)
            .map(|c| (c[0], c[1]))
            .collect();
        point_in_polygon(x, y, &points)
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
                data[dst_start..dst_start + copy_len]
                    .copy_from_slice(&self.pixel_buffer[src_start..src_end]);
            }
        }
        ImageData {
            width,
            height,
            data,
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
            0.0, 0.0,
            image_data.width as f32, image_data.height as f32,
            dx, dy,
            image_data.width as f32, image_data.height as f32,
        );
    }

    /// 将图像绘制到画布的指定位置，缩放到目标尺寸。应用当前变换。
    pub fn draw_image_with_size(
        &mut self,
        image_data: &ImageData,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
    ) {
        self.draw_image_sized(
            image_data,
            0.0, 0.0,
            image_data.width as f32, image_data.height as f32,
            dx, dy, dw, dh,
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
                        self.pixel_buffer[dst_idx] = ((r as f32 * src_a + self.pixel_buffer[dst_idx] as f32 * dst_a * (1.0 - src_a)) * factor) as u8;
                        self.pixel_buffer[dst_idx + 1] = ((g as f32 * src_a + self.pixel_buffer[dst_idx + 1] as f32 * dst_a * (1.0 - src_a)) * factor) as u8;
                        self.pixel_buffer[dst_idx + 2] = ((b as f32 * src_a + self.pixel_buffer[dst_idx + 2] as f32 * dst_a * (1.0 - src_a)) * factor) as u8;
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
        let shadow_alpha = ((self.shadow_color.a as f32 * self.global_alpha * blur_factor) as u8).min(self.shadow_color.a);
        let color = Color::rgba(self.shadow_color.r, self.shadow_color.g, self.shadow_color.b, shadow_alpha);
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
        let shadow_alpha = ((self.shadow_color.a as f32 * self.global_alpha * blur_factor) as u8).min(self.shadow_color.a);
        let color = Color::rgba(self.shadow_color.r, self.shadow_color.g, self.shadow_color.b, shadow_alpha);
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

    /// 将当前路径命令扁平化为顶点列表（x, y 交替）。
    /// 对于圆弧，使用线性近似（固定 16 段细分）。
    fn flatten_path(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        const ARC_SEGMENTS: usize = 16;

        for cmd in self.current_path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
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
                    // 简化实现：忽略圆角，退化为矩形子路径
                    // 使用与 Path2D::rect 相同的矩形线段
                    let _ = radii;
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
                    // 闭合回起点
                    vertices.push(corners[3].0);
                    vertices.push(corners[3].1);
                    vertices.push(corners[0].0);
                    vertices.push(corners[0].1);
                    current_x = corners[0].0;
                    current_y = corners[0].1;
                }
                PathCommand::ClosePath => {
                    // ClosePath 不产生额外线段（路径自动闭合由渲染器处理）
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
        const ARC_SEGMENTS: usize = 16;

        for cmd in path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
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
                    let _ = radii;
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
                    current_x = corners[0].0;
                    current_y = corners[0].1;
                }
                PathCommand::ClosePath => {}
            }
        }
        vertices
    }

    /// 将矩形区域的颜色写入像素缓冲区（光栅化填充）。
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
                self.pixel_buffer[idx] = color.r;
                self.pixel_buffer[idx + 1] = color.g;
                self.pixel_buffer[idx + 2] = color.b;
                self.pixel_buffer[idx + 3] = color.a;
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

    /// 将路径描边写入像素缓冲区（简化：使用 Bresenham 线段光栅化）。
    fn blit_stroke_to_pixels(&mut self, vertices: &[f32], color: Color, line_width: f32) {
        if vertices.len() < 4 {
            return;
        }
        // 为每条线段画一个宽度为 line_width 的矩形
        for chunk in vertices.chunks_exact(4) {
            let x1 = chunk[0];
            let y1 = chunk[1];
            let x2 = chunk[2];
            let y2 = chunk[3];
            let rect = self.line_segment_rect(x1, y1, x2, y2, line_width);
            self.blit_rect_to_pixels(&rect, color);
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
        assert_eq!(*ctx.fill_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(*ctx.fill_color(), Color::RED);
    }

    #[test]
    fn test_canvas_set_fill_color() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.set_fill_color(Color::GREEN);
        assert_eq!(*ctx.fill_color(), Color::GREEN);
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
        assert_eq!(*ctx.fill_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(*ctx.fill_color(), Color::GREEN);
        ctx.restore();
        assert_eq!(*ctx.fill_color(), Color::RED);
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
        assert_eq!(*ctx.stroke_color(), Color::BLUE);
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
        assert_eq!(*ctx.fill_color(), Color::BLACK);
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
        assert_eq!(*ctx.stroke_color(), Color::BLUE);
        ctx.restore();
        assert_eq!(*ctx.stroke_color(), Color::RED);
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
}
