//! Canvas 2D 渲染上下文 — 公共 API 方法。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::path::{Path2D, PathCommand};

use super::offscreen::*;
use super::types::*;

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
        if self.fill_style.is_per_pixel_style() {
            // 渐变：每像素采样光栅化（真实 gradient 渲染）。primitives 合成层用 midpoint 近似单色记录
            //（GPU 合成路径的 gradient 为独立大工程，headless 像素回读路径已逐像素正确）。
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_fill(rect, approx);
            self.blit_rect_gradient(&rect, &self.fill_style.clone());
        } else {
            let color = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_fill(rect, color);
            self.blit_rect_to_pixels(&rect, color);
        }
    }

    /// 描边矩形。
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // 简化实现：用描边样式填充四个薄矩形表示描边（上/下/左/右）
        let lw = self.line_width;

        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            let rect = self.transform_rect(x, y, width, height);
            self.draw_shadow_rect(&rect);
        }

        // R3084：渐变描边逐像素光栅化（对称 fill_rect 渐变 R3079）。四边各经 blit_rect_gradient；
        // 纯色走 flat 快路径。primitives 合成层用 midpoint 近似（与 fill_rect 一致）。
        let gradient = self.stroke_style.is_per_pixel_style();
        let approx_or_color = self.apply_alpha(self.stroke_style.resolve_color());
        let style = self.stroke_style.clone();
        // 上 / 下 / 左 / 右 四边
        let sides = [
            self.transform_rect(x, y, width, lw),
            self.transform_rect(x, y + height - lw, width, lw),
            self.transform_rect(x, y, lw, height),
            self.transform_rect(x + width - lw, y, lw, height),
        ];
        for rect in sides {
            self.primitives.add_fill(rect, approx_or_color);
            if gradient {
                self.blit_rect_gradient(&rect, &style);
            } else {
                self.blit_rect_to_pixels(&rect, approx_or_color);
            }
        }
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
                    font_glyph_index: None,
                    source: None,
                    font_id: zero_render_foundation::primitive::FontId(0),
                    bitmap_width: None,
                    bitmap_height: None,
                    rotation: 0.0,
                    synthetic_italic: false,
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
                    font_glyph_index: None,
                    source: None,
                    font_id: zero_render_foundation::primitive::FontId(0),
                    bitmap_width: None,
                    bitmap_height: None,
                    rotation: 0.0,
                    synthetic_italic: false,
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

    /// 添加圆角矩形子路径（Canvas 2D `roundRect`，HTML Canvas §`dom-context-2d-api` roundRect）。
    /// 起点角经当前变换矩阵映射（与 `arc`/`rect` 同语义）；`radii` 为角半径列表（spec：单值 / [tl,tr,br,bl]
    /// / 其它长度按 [HTML §roundrect] 规则解析，本层透传 Path2D::round_rect，flattener 现 best-effort 退化
    /// 为矩形——角圆为 rendering 流域已知简化，几何/命中测试仍正确）。
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radii: Vec<f32>) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path.round_rect(tx, ty, w, h, radii);
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
        if self.fill_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), approx);
            self.blit_path_gradient(&vertices, &self.fill_style.clone());
        } else {
            let color = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), color);
            self.blit_path_to_pixels(&vertices, color);
        }
    }

    /// 描边路径。将路径命令扁平化为顶点列表，生成路径描边图元。
    pub fn stroke(&mut self) {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return;
        }
        // 绘制阴影（在形状之前）——R3241：用 stroke 足迹（thick rect + 连接点），非 centerline。
        if self.has_shadow() {
            self.draw_shadow_stroke(&vertices, self.line_width);
        }
        let closed = self
            .current_path
            .commands()
            .iter()
            .any(|c| matches!(c, PathCommand::ClosePath));
        if self.stroke_style.is_per_pixel_style() {
            // 渐变描边：逐像素光栅化（R3084，对称 fill 渐变 R3079）。primitives 用 midpoint 近似。
            let approx = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), approx, self.line_width, closed);
            self.blit_stroke_to_pixels_gradient(&vertices, &self.stroke_style.clone(), self.line_width);
        } else {
            let color = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), color, self.line_width, closed);
            self.blit_stroke_to_pixels(&vertices, color, self.line_width);
        }
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
        if self.fill_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), approx);
            self.blit_path_gradient(&vertices, &self.fill_style.clone());
        } else {
            let color = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), color);
            self.blit_path_to_pixels(&vertices, color);
        }
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
        let closed = path.commands().iter().any(|c| matches!(c, PathCommand::ClosePath));
        if self.stroke_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), approx, self.line_width, closed);
            self.blit_stroke_to_pixels_gradient(&vertices, &self.stroke_style.clone(), self.line_width);
        } else {
            let color = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), color, self.line_width, closed);
            self.blit_stroke_to_pixels(&vertices, color, self.line_width);
        }
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
        // R3238：source-over + 全透源像素为 no-op（保 drawImage 热路径性能——跳逐像素 composite_pixel）；
        // 非 source-over 透源有定义行为（source-in/destination-in/copy 须清除 dst），不跳。
        let skip_transparent_src = self.composite_operation == CompositeOperation::SourceOver;

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
                // R3238：drawImage 消费 globalCompositeOperation（与 fill/fillRect/stroke 一致经 composite_pixel）。
                // 旧实现固定 source-over 内联 alpha 混合，无视 composite_operation。
                let src = Color {
                    r,
                    g,
                    b,
                    a: (a as f32 * self.global_alpha) as u8,
                };
                if skip_transparent_src && src.a == 0 {
                    continue;
                }
                let (pr, pg, pb, pa) = self.composite_pixel(
                    src,
                    self.pixel_buffer[dst_idx],
                    self.pixel_buffer[dst_idx + 1],
                    self.pixel_buffer[dst_idx + 2],
                    self.pixel_buffer[dst_idx + 3],
                );
                self.pixel_buffer[dst_idx] = pr;
                self.pixel_buffer[dst_idx + 1] = pg;
                self.pixel_buffer[dst_idx + 2] = pb;
                self.pixel_buffer[dst_idx + 3] = pa;
            }
        }
    }

    // ── Output ──

    /// 判断当前是否启用了阴影（阴影颜色不透明且偏移或模糊非零）。
    fn has_shadow(&self) -> bool {
        self.shadow_color.a > 0
            && (self.shadow_blur > 0.0 || self.shadow_offset_x != 0.0 || self.shadow_offset_y != 0.0)
    }

    /// R3240：为矩形绘制阴影——region alpha mask（矩形覆盖）+ box blur（shadowBlur）+ 经
    /// composite_shadow_mask 合成（消费 globalCompositeOperation，与 fill/stroke 一致）。
    /// 旧实现仅画偏移硬边矩形、alpha 按 `1/(1+blur·0.1)` 衰减（无 blur）。
    fn draw_shadow_rect(&mut self, rect: &Rect) {
        let (radius, pad, passes) = super::raster::shadow_blur_geom(self.shadow_blur);
        let cw = self.width as i32;
        let ch = self.height as i32;
        let rx0 = (rect.left().floor() as i32 - pad).max(0);
        let ry0 = (rect.top().floor() as i32 - pad).max(0);
        let rx1 = (rect.right().ceil() as i32 + pad).min(cw);
        let ry1 = (rect.bottom().ceil() as i32 + pad).min(ch);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        let rw = (rx1 - rx0) as usize;
        let rh = (ry1 - ry0) as usize;
        let mut mask = vec![0u8; rw * rh];
        let (rl, rt, rr, rb) = (rect.left(), rect.top(), rect.right(), rect.bottom());
        for ly in 0..rh as i32 {
            let wy = ry0 + ly;
            for lx in 0..rw as i32 {
                let wx = rx0 + lx;
                if (wx as f32) >= rl && (wx as f32) < rr && (wy as f32) >= rt && (wy as f32) < rb {
                    mask[(ly as usize) * rw + (lx as usize)] = 255;
                }
            }
        }
        // R3242：3 遍 box blur ≈ gaussian（比单遍 triangle 衰减更平滑）。
        for _ in 0..passes {
            super::raster::box_blur_alpha(&mut mask, rw, rh, radius);
        }
        self.composite_shadow_mask(
            &mask,
            rx0,
            ry0,
            rw,
            rh,
            self.shadow_offset_x,
            self.shadow_offset_y,
            self.shadow_color,
            self.global_alpha,
        );
    }

    /// R3240：为路径绘制阴影——region alpha mask（扫描线覆盖）+ box blur + composite_shadow_mask。
    fn draw_shadow_path(&mut self, vertices: &[f32]) {
        if vertices.len() < 4 {
            return;
        }
        let (radius, pad, passes) = super::raster::shadow_blur_geom(self.shadow_blur);
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for c in vertices.chunks_exact(2) {
            min_x = min_x.min(c[0]);
            min_y = min_y.min(c[1]);
            max_x = max_x.max(c[0]);
            max_y = max_y.max(c[1]);
        }
        let cw = self.width as i32;
        let ch = self.height as i32;
        let rx0 = (min_x.floor() as i32 - pad).max(0);
        let ry0 = (min_y.floor() as i32 - pad).max(0);
        let rx1 = (max_x.ceil() as i32 + pad).min(cw);
        let ry1 = (max_y.ceil() as i32 + pad).min(ch);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        let rw = (rx1 - rx0) as usize;
        let rh = (ry1 - ry0) as usize;
        let mut mask = vec![0u8; rw * rh];
        super::raster::rasterize_path_coverage(vertices, &mut mask, rw, rh, rx0, ry0);
        // R3242：3 遍 box blur ≈ gaussian（比单遍 triangle 衰减更平滑）。
        for _ in 0..passes {
            super::raster::box_blur_alpha(&mut mask, rw, rh, radius);
        }
        self.composite_shadow_mask(
            &mask,
            rx0,
            ry0,
            rw,
            rh,
            self.shadow_offset_x,
            self.shadow_offset_y,
            self.shadow_color,
            self.global_alpha,
        );
    }

    /// R3241：为描边绘制阴影——region mask 由 stroke 足迹（每段 thick rect + 连接点方块）构成，
    /// 非 centerline（旧 stroke() 传 centerline 致粗描边阴影过细）。box blur + composite 同 R3240。
    fn draw_shadow_stroke(&mut self, vertices: &[f32], line_width: f32) {
        if vertices.len() < 4 {
            return;
        }
        let segments: Vec<[f32; 4]> = vertices.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
        if segments.is_empty() {
            return;
        }
        let half_lw = line_width / 2.0;
        let (radius, blur_pad, passes) = super::raster::shadow_blur_geom(self.shadow_blur);
        let pad = blur_pad as f32 + half_lw;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for s in &segments {
            min_x = min_x.min(s[0]).min(s[2]);
            min_y = min_y.min(s[1]).min(s[3]);
            max_x = max_x.max(s[0]).max(s[2]);
            max_y = max_y.max(s[1]).max(s[3]);
        }
        let cw = self.width as i32;
        let ch = self.height as i32;
        let rx0 = ((min_x - pad).floor() as i32).max(0);
        let ry0 = ((min_y - pad).floor() as i32).max(0);
        let rx1 = ((max_x + pad).ceil() as i32).min(cw);
        let ry1 = ((max_y + pad).ceil() as i32).min(ch);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        let rw = (rx1 - rx0) as usize;
        let rh = (ry1 - ry0) as usize;
        let mut mask = vec![0u8; rw * rh];
        // 每段 thick rect（与 blit_stroke_to_pixels 同款 line_segment_rect）。
        for s in &segments {
            let r = self.line_segment_rect(s[0], s[1], s[2], s[3], line_width);
            super::raster::fill_rect_into_mask(&mut mask, rw, rh, rx0, ry0, &r);
        }
        // 连接点方块（half_lw 偏移、line_width 边长，与 blit_stroke_to_pixels 一致）。
        for s in segments.iter().take(segments.len().saturating_sub(1)) {
            let r = Rect::new(s[2] - half_lw, s[3] - half_lw, line_width, line_width);
            super::raster::fill_rect_into_mask(&mut mask, rw, rh, rx0, ry0, &r);
        }
        // R3242：3 遍 box blur ≈ gaussian（比单遍 triangle 衰减更平滑）。
        for _ in 0..passes {
            super::raster::box_blur_alpha(&mut mask, rw, rh, radius);
        }
        self.composite_shadow_mask(
            &mask,
            rx0,
            ry0,
            rw,
            rh,
            self.shadow_offset_x,
            self.shadow_offset_y,
            self.shadow_color,
            self.global_alpha,
        );
    }

    /// 消费上下文，返回渲染图元列表。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 返回渲染图元列表的引用。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }
}
