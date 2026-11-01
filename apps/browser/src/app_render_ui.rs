// UI 文本 helper 渲染（行 metrics / 垂直居中 / 绘制 / 测量 / 截断）。

// 从 app_render.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs / app_render_address.rs 同模式；方法保留在 `impl BrowserApp { }` 内，
// self 字段（font_id / font_loader）与 Color / GlyphDraw 等类型直接可达。

impl BrowserApp {
    /// fontdue 行 metrics；无字体时回退为 `(font_size, 0)`。
    fn ui_line_metrics(&self, font_size: f32) -> (f32, f32) {
        let Some(primary) = self.font_id else {
            return (font_size, 0.0);
        };
        self.font_loader
            .line_metrics(primary, font_size)
            .unwrap_or((font_size, 0.0))
    }

    /// 在给定高度内垂直居中 UI 文本，返回 `(text_top, baseline_y)`。
    fn ui_text_centered_in_height(&self, height: f32, font_size: f32) -> (f32, f32) {
        let (text_top, ascent) = self.ui_text_top_in_box(0.0, height, font_size);
        (text_top, text_top + ascent)
    }

    /// 在给定矩形高度内垂直居中 UI 文本，返回 `(text_top, ascent)`。
    fn ui_text_top_in_box(&self, box_y: f32, box_h: f32, font_size: f32) -> (f32, f32) {
        let (ascent, descent) = self.ui_line_metrics(font_size);
        let line_h = ascent - descent;
        let text_top = box_y + (box_h - line_h) / 2.0;
        (text_top, ascent)
    }

    /// UI 文本整串 shape（总宽 + 逐字符 `(字符, 字体 ID, x 偏移)`），命中
    /// [`Self::ui_text_shape_cache`] 零解析。字体回退解析每字符一次
    /// （rasterize_glyph_with_fallback + measure_advance），菜单/工具栏标签不变而
    /// 逐帧重跑是渐入帧 scene 成本主项（T28-P37）。
    fn ui_text_shape(&self, text: &str, font_size: f32) -> UiTextShape {
        const UI_TEXT_SHAPE_CACHE_CAP: usize = 512;
        let Some(primary) = self.font_id else {
            return (0.0, Vec::new());
        };
        let key = (text.to_string(), font_size.to_bits());
        if let Some(hit) = self.ui_text_shape_cache.borrow().get(&key) {
            return hit.clone();
        }
        let mut shape = Vec::new();
        let mut x = 0.0f32;
        for ch in text.chars() {
            let font_id = self
                .font_loader
                .rasterize_glyph_with_fallback(primary, ch, font_size)
                .map(|(id, _)| id)
                .unwrap_or(primary);
            shape.push((ch, font_id, x));
            x += self.font_loader.measure_advance(primary, ch, font_size);
        }
        {
            let mut cache = self.ui_text_shape_cache.borrow_mut();
            if cache.len() >= UI_TEXT_SHAPE_CACHE_CAP {
                cache.clear();
            }
            cache.insert(key, (x, shape.clone()));
        }
        (x, shape)
    }

    /// 绘制 UI 文本（使用字体回退链和真实 advance 宽度）
    fn draw_ui_text(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        font_size: f32,
        color: Color,
        glyphs: &mut Vec<GlyphDraw>,
    ) {
        if self.font_id.is_none() {
            return;
        }
        let (ascent, _) = self.ui_line_metrics(font_size);
        let baseline_y = start_y + ascent;
        let (_, shape) = self.ui_text_shape(text, font_size);
        for (ch, font_id, dx) in shape {
            glyphs.push(GlyphDraw {
                ch,
                font_glyph_index: None,
                x: start_x + dx,
                baseline_y,
                color,
                font_id,
                font_variations: None,
                font_size,
                rotation: 0.0,
            });
        }
    }

    /// 测量 UI 文本总宽度
    fn measure_ui_text_width(&self, text: &str, font_size: f32) -> f32 {
        self.ui_text_shape(text, font_size).0
    }

    /// 按像素宽度截断 UI 文本
    fn truncate_ui_text(&self, text: &str, max_width: f32, font_size: f32) -> String {
        let Some(primary) = self.font_id else {
            return text.to_string();
        };
        let mut result = String::new();
        let mut width = 0.0;
        let ellipsis_advance = self.font_loader.measure_advance(primary, '…', font_size);
        for ch in text.chars() {
            let advance = self.font_loader.measure_advance(primary, ch, font_size);
            if width + advance + ellipsis_advance > max_width && !result.is_empty() {
                result.push('…');
                break;
            }
            result.push(ch);
            width += advance;
        }
        result
    }
}
