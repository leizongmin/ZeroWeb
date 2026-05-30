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

/// Canvas 状态（用于 save/restore）。
#[derive(Debug, Clone)]
struct CanvasState {
    fill_color: Color,
    stroke_color: Color,
    line_width: f32,
    font: FontDescriptor,
    global_alpha: f32,
    transform: Transform2D,
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
        }
    }

    // ── Rectangle drawing ──

    /// 清除矩形区域（设为透明）。
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 添加一个透明色填充来表示清除操作
        let rect = self.transform_rect(x, y, width, height);
        self.primitives.add_fill(rect, Color::TRANSPARENT);
    }

    /// 填充矩形。
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = self.transform_rect(x, y, width, height);
        let color = self.apply_alpha(self.fill_color);
        self.primitives.add_fill(rect, color);
    }

    /// 描边矩形。
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 简化实现：用描边颜色填充一个薄矩形表示描边
        let lw = self.line_width;
        let color = self.apply_alpha(self.stroke_color);

        // 上边
        self.primitives
            .add_fill(self.transform_rect(x, y, width, lw), color);
        // 下边
        self.primitives
            .add_fill(self.transform_rect(x, y + height - lw, width, lw), color);
        // 左边
        self.primitives
            .add_fill(self.transform_rect(x, y, lw, height), color);
        // 右边
        self.primitives
            .add_fill(self.transform_rect(x + width - lw, y, lw, height), color);
    }

    // ── Text ──

    /// 填充文本。
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.apply_alpha(self.fill_color);
        let font_size = self.font.size;
        let (tx, ty) = self.transform.transform_point(x, y);
        self.primitives
            .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                x: tx,
                y: ty,
                font_size,
                color,
                glyph_id: 0,
                font_id: zero_render_foundation::primitive::FontId(0),
                bitmap_width: None,
                bitmap_height: None,
            });
        // 存储文本长度作为额外 glyph（简化：一个 glyph 代表整段文本）
        let _ = text;
    }

    /// 描边文本（简化：与 fill_text 相同，用描边颜色）。
    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.apply_alpha(self.stroke_color);
        let font_size = self.font.size;
        let (tx, ty) = self.transform.transform_point(x, y);
        self.primitives
            .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                x: tx,
                y: ty,
                font_size,
                color,
                glyph_id: 0,
                font_id: zero_render_foundation::primitive::FontId(0),
                bitmap_width: None,
                bitmap_height: None,
            });
        let _ = text;
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

    /// 填充路径（简化：遍历路径命令，提取矩形近似）。
    pub fn fill(&mut self) {
        let color = self.apply_alpha(self.fill_color);
        let bbox = self.path_bounding_box();
        if let Some(rect) = bbox {
            self.primitives.add_fill(rect, color);
        }
    }

    /// 描边路径（简化）。
    pub fn stroke(&mut self) {
        let color = self.apply_alpha(self.stroke_color);
        let bbox = self.path_bounding_box();
        if let Some(rect) = bbox {
            // 简化：用描边颜色填充包围盒
            self.primitives.add_fill(rect, color);
        }
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

    // ── Pixel data ──

    /// 获取像素数据（简化版：返回全零数据）。
    pub fn get_image_data(&self, _x: u32, _y: u32, width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        ImageData {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    /// 放置像素数据（简化版：记录为填充图元）。
    pub fn put_image_data(&mut self, _image_data: &ImageData, _x: u32, _y: u32) {
        // 简化：暂不实现像素级操作
    }

    // ── Output ──

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

    /// 计算当前路径的包围盒。
    fn path_bounding_box(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut has_points = false;

        for cmd in self.current_path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y)
                | PathCommand::LineTo(x, y)
                | PathCommand::Arc(x, y, ..) => {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                    has_points = true;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    min_x = min_x.min(cpx).min(x);
                    min_y = min_y.min(cpy).min(y);
                    max_x = max_x.max(cpx).max(x);
                    max_y = max_y.max(cpy).max(y);
                    has_points = true;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    min_x = min_x.min(cp1x).min(cp2x).min(x);
                    min_y = min_y.min(cp1y).min(cp2y).min(y);
                    max_x = max_x.max(cp1x).max(cp2x).max(x);
                    max_y = max_y.max(cp1y).max(cp2y).max(y);
                    has_points = true;
                }
                PathCommand::ClosePath => {}
            }
        }

        if has_points && min_x < max_x && min_y < max_y {
            Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
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
        assert_eq!(ctx.primitives().glyphs.len(), 1);
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
        // fill_rect = 1, stroke_rect = 4, fill_text = 1 glyph
        assert_eq!(ctx.primitives().fills.len(), 5);
        assert_eq!(ctx.primitives().glyphs.len(), 1);
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
        assert_eq!(ctx.primitives().glyphs.len(), 1);
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
        assert_eq!(ctx.primitives().fills.len(), 0);
    }

    #[test]
    fn test_canvas_move_to_line_to_fill() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.fill();
        assert!(!ctx.primitives().fills.is_empty());
    }

    #[test]
    fn test_canvas_stroke_path() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.stroke();
        assert!(!ctx.primitives().fills.is_empty());
    }

    #[test]
    fn test_canvas_fill_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.fill();
        assert_eq!(ctx.primitives().fills.len(), 0);
    }

    #[test]
    fn test_canvas_stroke_empty_path() {
        let mut ctx = CanvasContext::new(100, 100);
        ctx.stroke();
        assert_eq!(ctx.primitives().fills.len(), 0);
    }

    #[test]
    fn test_canvas_quadratic_curve_to() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.quadratic_curve_to(50.0, 0.0, 100.0, 50.0);
        ctx.fill();
        assert!(!ctx.primitives().fills.is_empty());
    }

    #[test]
    fn test_canvas_bezier_curve_to() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.bezier_curve_to(30.0, 0.0, 70.0, 100.0, 100.0, 50.0);
        ctx.fill();
        assert!(!ctx.primitives().fills.is_empty());
    }

    #[test]
    fn test_canvas_close_path_on_context() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.close_path();
        ctx.fill();
        assert!(!ctx.primitives().fills.is_empty());
    }

    #[test]
    fn test_canvas_arc_on_context() {
        let mut ctx = CanvasContext::new(200, 200);
        ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI);
        ctx.line_to(100.0, 100.0); // 确保包围盒有面积（arc 仅记录中心点）
        ctx.fill();
        assert!(!ctx.primitives().fills.is_empty());
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
        assert_eq!(ctx.primitives().glyphs.len(), 1);
        ctx.stroke_text("World", 10.0, 50.0);
        assert_eq!(ctx.primitives().glyphs.len(), 2);
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
        assert!(!ctx.primitives().fills.is_empty());
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
}
