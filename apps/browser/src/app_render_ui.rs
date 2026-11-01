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

#[cfg(test)]
mod ui_text_shape_cache_tests {
    use super::*;

    /// T28-P37 R2（TV2 返修）：shape 缓存命中路径输出与未命中一致（总宽 +
    /// 逐字符 (字符, 字体 ID, x 偏移)），draw/measure 两个消费方冷热一致，
    /// size 参与键隔离（同文本不同 size 不得互串）。
    #[test]
    fn ui_text_shape_cache_hits_match_misses_and_sizes_isolate_keys() {
        let app = BrowserApp::new(RenderMode::Cpu);
        let text = "打开菜单 Open 42";
        let size = 14.0;
        let cold = app.ui_text_shape(text, size);
        assert!(!cold.1.is_empty(), "有字体时必须产出 shape");
        assert!(!app.ui_text_shape_cache.borrow().is_empty(), "首跑应写入缓存");
        let warm = app.ui_text_shape(text, size);
        assert_eq!(cold, warm, "命中输出必须与未命中一致");

        let glyph_key = |glyphs: &[GlyphDraw]| -> Vec<(char, u32, u32, u32, u32)> {
            glyphs
                .iter()
                .map(|g| {
                    (
                        g.ch,
                        g.font_id,
                        g.x.to_bits(),
                        g.baseline_y.to_bits(),
                        g.font_size.to_bits(),
                    )
                })
                .collect()
        };
        let mut cold_glyphs = Vec::new();
        let mut warm_glyphs = Vec::new();
        app.draw_ui_text(text, 3.0, 5.0, size, Color::BLACK, &mut cold_glyphs);
        app.draw_ui_text(text, 3.0, 5.0, size, Color::BLACK, &mut warm_glyphs);
        assert_eq!(glyph_key(&cold_glyphs), glyph_key(&warm_glyphs), "draw 冷热一致");
        assert!(!cold_glyphs.is_empty());
        assert_eq!(
            app.ui_text_shape(text, size).0,
            app.measure_ui_text_width(text, size),
            "measure 与 shape 总宽同源"
        );

        let other_size = app.ui_text_shape(text, 20.0);
        assert_eq!(app.ui_text_shape(text, 20.0), other_size, "同 size 命中");
        assert_eq!(app.ui_text_shape(text, size), cold, "size 键隔离，旧键不变");
    }

    /// T28-P37 R2（TV2 返修）：字体 registry 变化必须清空 shape 缓存——旧字体
    /// ID 解析与 advance 全部失效（sync_page_fonts 清空点）；清空后重算可用。
    #[test]
    fn sync_page_fonts_invalidates_ui_text_shape_cache() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        let tab = app.shell.active_tab_id().unwrap();
        app.tabs.ensure_snapshot_for_test(tab);
        let _ = app.ui_text_shape("GitHub", 14.0);
        assert!(!app.ui_text_shape_cache.borrow().is_empty(), "前置：缓存已预热");
        let font = zero_protocol::IpcFontPayload {
            font_id: 421,
            face_index: 0,
            data: include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf").to_vec(),
        };
        app.tabs.snapshot_mut(tab).unwrap().font_payloads = vec![font];
        assert!(app.sync_page_fonts());
        assert!(
            app.ui_text_shape_cache.borrow().is_empty(),
            "registry 换代必须清空 shape 缓存"
        );
        let refreshed = app.ui_text_shape("GitHub", 14.0);
        assert!(!refreshed.1.is_empty(), "清空后重算可用");
        assert_eq!(refreshed, app.ui_text_shape("GitHub", 14.0), "重算后命中一致");
    }
}
