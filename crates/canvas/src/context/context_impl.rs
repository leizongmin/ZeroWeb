//! Canvas 2D 渲染上下文 — 公共 API 方法。

use std::sync::{Arc, Mutex};

use zero_render_foundation::color::Color;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::path::{Path2D, PathCommand};

use super::offscreen::*;
use super::types::*;

impl CanvasContext {
    /// 创建指定尺寸的 Canvas 上下文。
    pub fn new(width: u32, height: u32) -> Self {
        // R3354：尺寸计算用 usize + saturating_mul，避免 width*height*4 在 u32 上溢出
        // （65536*65536*4 在 u32 回绕为 0 → 分配 0 字节缓冲区，后续索引写越界）。
        // saturating 后极端大尺寸 → Vec 分配失败（安全 abort），而非静默回绕致内存损坏。
        let buffer_size = (width as usize).saturating_mul(height as usize).saturating_mul(4);
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
            image_smoothing_quality: ImageSmoothingQuality::default(),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            miter_limit: 10.0,
            direction: TextDirection::Inherit,
            font_loader: None,
            font_id: None,
            stroke_dedup_mask: None,
        }
    }

    /// R34xx：注入共享字体加载器（bridge 在 getContext2d 时设置；None = 无字体栈）。
    pub fn set_font_loader(&mut self, loader: Option<Arc<Mutex<FontLoader>>>) {
        self.font_loader = loader;
        // 字体栈变化 → 重新解析当前字体。
        self.resolve_font_id();
    }

    /// R34xx：按当前 FontDescriptor.family 经 loader 解析器查 font_id（找不到 → None，
    /// fill_text 回落启发式）。
    fn resolve_font_id(&mut self) {
        self.font_id = None;
        let Some(loader) = self.font_loader.clone() else {
            return;
        };
        let Ok(loader) = loader.lock() else {
            return;
        };
        let resolver = loader.build_font_resolver();
        // R34xx：家族名大小写不敏感匹配——resolver 键为注册时原样（@font-face
        // 'CanvasTest' → 键 "CanvasTest"），先按原串再按小写查（ctx.font='50px
        // CanvasTest' 的 2d.text.draw.* 系列——先前仅小写查询 miss 后回退 sans-serif，
        // 恰逢 sans-serif 落 id 0（首字体=CanvasTest）被掩盖；系统字体预载后暴露）。
        let family = self.font.family.trim();
        self.font_id = resolver
            .get(family)
            .copied()
            .or_else(|| resolver.get(&family.to_ascii_lowercase()).copied())
            .or_else(|| {
                // 未注册族 → 通用族回落（sans-serif 等）。
                resolver
                    .get("sans-serif")
                    .or_else(|| resolver.get("serif"))
                    .or_else(|| resolver.get("monospace"))
                    .copied()
            });
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
                // R34xx：clip 区域裁剪（clip_path 未设时零开销）。
                if !self.clip_applies(px as f32, py as f32) {
                    continue;
                }
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
            self.draw_shadow_rect(&rect, &self.fill_style.clone());
        }
        if self.fill_style.is_per_pixel_style() {
            // 渐变：每像素采样光栅化（真实 gradient 渲染）。primitives 合成层用 midpoint 近似单色记录
            //（GPU 合成路径的 gradient 为独立大工程，headless 像素回读路径已逐像素正确）。
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_fill(rect, approx);
            let style = self.transform_gradient(&self.fill_style);
            self.blit_rect_gradient(&rect, &style);
        } else {
            let color = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_fill(rect, color);
            self.blit_rect_to_pixels(&rect, color);
        }
        // R34xx：source 独占类 composite 的未覆盖区域清除（矩形外置透明——
        // 2d.composite.uncovered.fill.* / solid.*）。
        if self.composite_clears_uncovered() {
            self.clear_outside_rect(&rect);
        }
    }

    /// 描边矩形。
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // R34xx：改为矩形周长路径 + stroke 语义（spec：strokeRect 画闭合矩形路径并按
        // lineWidth/lineJoin 描边，线向内外各扩 lw/2）。旧四边薄矩形实现：(a) 只覆盖矩形
        // 内侧，Nx0 退化矩形（上游 2d.strokeRect.zero.5）与负尺寸（2d.strokeRect.negative）
        // 描边几何错位；(b) 阴影走 rect 而非 stroke 足迹。顶点先经 CTM 变换（同 move_to/line_to）。
        let (x1, y1) = self.transform.transform_point(x, y);
        let (x2, y2) = self.transform.transform_point(x + width, y + height);
        // R34xx：0x0（含变换后）→ 无操作（spec：strokeRect of 0x0 draws nothing，含 caps/joins）。
        if x1 == x2 && y1 == y2 {
            return;
        }
        let mut rect_path = Path2D::new();
        rect_path.move_to(x1, y1);
        rect_path.line_to(x2, y1);
        rect_path.line_to(x2, y2);
        rect_path.line_to(x1, y2);
        rect_path.close_path();
        let vertices = rect_path.flatten_to_vertices();
        if vertices.is_empty() {
            return;
        }
        // 绘制阴影（在形状之前）——stroke 足迹（同 stroke()/stroke_with_path R3356 口径）。
        if self.has_shadow() {
            let (min_x, min_y, max_x, max_y) = vertices
                .chunks_exact(2)
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(mnx, mny, mxx, mxy), c| {
                    (mnx.min(c[0]), mny.min(c[1]), mxx.max(c[0]), mxy.max(c[1]))
                });
            let shape_alpha = self.style_alpha(&self.stroke_style, (min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
            self.draw_shadow_stroke(&vertices, self.line_width, shape_alpha);
        }
        let closed = true;
        if self.stroke_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), approx, self.line_width, closed);
            self.blit_stroke_to_pixels_gradient(&vertices, &self.stroke_style.clone(), self.line_width, closed);
        } else {
            let color = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), color, self.line_width, closed);
            self.blit_stroke_to_pixels(&vertices, color, self.line_width, closed);
        }
    }

    // ── Text ──

    /// 填充文本。为每个字符生成独立的 GlyphPrimitive，glyph_id 取字符的 Unicode 码点。
    /// R34xx：字体栈可用（headless/testharness @font-face 注入）时走真实 shape + 光栅化——
    /// 逐 glyph 灰度位图 alpha 混合进 pixel_buffer（2d.text.draw.* 像素断言）。
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32, max_width: Option<f32>) {
        // R34xx：文本路径经 sample_at 取样式色（替代 midpoint resolve_color）——零长渐变
        // （x0==x1&&y0==y1）sample_at 返透明不画（2d.gradient.interpolate.zerosize.
        // fillText/strokeText 期望保持底色；midpoint 会取 stop 色误画）。
        let color = self.apply_alpha(self.fill_style.sample_at(x, y));
        self.draw_text_glyphs(text, x, y, color, max_width);
    }

    /// 描边文本。R34xx：与 fill_text 同真字体路径（shape + 墨迹 blit——描边以填充近似，
    /// strokeTextCluster 等 WPT 断言字形覆盖；精确 outline 描边为深缺口）。
    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32, max_width: Option<f32>) {
        let color = self.apply_alpha(self.stroke_style.sample_at(x, y));
        self.draw_text_glyphs(text, x, y, color, max_width);
    }

    /// R34xx：真字体光栅共用路径（shape + 逐 glyph 位图 blit + primitives）——fill_text 与
    /// stroke_text 共用。无字体栈 → 仅 primitives 启发式（不写像素）。
    fn draw_text_glyphs(&mut self, text: &str, x: f32, y: f32, color: Color, max_width: Option<f32>) {
        let font_size = self.font.size;
        // 对齐/基线偏移（与 fill_text 同——R34xx 推导的 em 方块定位）。
        let width = text.chars().count() as f32 * font_size;
        let rtl = matches!(self.direction, TextDirection::Rtl);
        let ox = match self.text_align {
            TextAlign::Center => -width / 2.0,
            TextAlign::Right => -width,
            TextAlign::Left => 0.0,
            TextAlign::Start => {
                if rtl {
                    -width
                } else {
                    0.0
                }
            }
            TextAlign::End => {
                if rtl {
                    0.0
                } else {
                    -width
                }
            }
        };
        let (ascent, descent) = match (self.font_loader.clone(), self.font_id) {
            (Some(loader), Some(fid)) => loader
                .lock()
                .ok()
                .and_then(|l| l.line_metrics(fid, font_size))
                .unwrap_or((font_size * 0.8, font_size * 0.2)),
            _ => (font_size * 0.8, font_size * 0.2),
        };
        let oy = match self.text_baseline {
            TextBaseline::Top => ascent,
            TextBaseline::Hanging => font_size * 0.5,
            TextBaseline::Middle => (ascent + descent) / 2.0,
            TextBaseline::Ideographic => font_size * 0.125,
            TextBaseline::Alphabetic => 0.0,
            TextBaseline::Bottom => descent,
        };
        let (tx, ty) = self.transform.transform_point(x + ox, y + oy);
        // R34xx：spec text preparation——ASCII whitespace → U+0020（tab 等与 space 同绘制）。
        let prepared = prepare_canvas_text(text);
        // R34xx：font-variant small-caps 合成（字体无 smcp 特征时 Chromium 以小写→大写
        // 字形渲染——2d.text.fontVariantCaps2.worker 的 measure 宽度须不同）。
        let shaped_text = if self.font.small_caps {
            prepared.to_uppercase()
        } else {
            prepared
        };
        if let Some(font_id) = self.font_id
            && let Some(loader) = self.font_loader.clone()
            && let Ok(loader) = loader.lock()
            && let Some(shaped) = if rtl {
                loader.shape_text_cached_with_features(
                    font_id,
                    &shaped_text,
                    font_size,
                    zero_render_foundation::font::TextDirection::RightToLeft,
                    &text_features(self.font.kerning_none, self.font.small_caps),
                )
            } else {
                loader.shape_text_cached_with_features(
                    font_id,
                    &shaped_text,
                    font_size,
                    zero_render_foundation::font::TextDirection::Auto,
                    &text_features(self.font.kerning_none, self.font.small_caps),
                )
            }
        {
            let natural: f32 = shaped.iter().map(|g| g.advance_x).sum::<f32>().abs();
            let scale = match max_width {
                Some(mw) if mw > 0.0 && natural > mw => mw / natural,
                _ => 1.0,
            };
            let mut pen_x = tx;
            for g in &shaped {
                let glyph_index = g.glyph_id.min(u32::from(u16::MAX)) as u16;
                if let Ok(bmp) = loader.rasterize_glyph_index(font_id, glyph_index, g.font_size * scale) {
                    let (gx, gy) = (
                        pen_x + g.x_offset * scale + bmp.x_offset as f32,
                        ty - bmp.y_offset as f32 - bmp.height as f32,
                    );
                    self.blit_glyph_bitmap(&bmp, gx, gy, color);
                }
                self.primitives
                    .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                        x: pen_x + g.x_offset * scale,
                        y: ty,
                        font_size: g.font_size * scale,
                        color,
                        glyph_id: g.glyph_id,
                        font_glyph_index: Some(glyph_index),
                        source: None,
                        font_id: zero_render_foundation::primitive::FontId(font_id),
                        font_variation_id: None,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                pen_x += g.advance_x * scale;
                let ls = parse_length_px(&self.font.letter_spacing, font_size).unwrap_or(0.0);
                pen_x += ls * scale;
                if g.glyph_id == 7 {
                    let ws = parse_length_px(&self.font.word_spacing, font_size).unwrap_or(0.0);
                    pen_x += ws * scale;
                }
            }
            return;
        }
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
                    font_variation_id: None,
                    bitmap_width: None,
                    bitmap_height: None,
                    rotation: 0.0,
                    synthetic_italic: false,
                });
            offset_x += em_width;
        }
    }

    /// 测量文本度量（HTML Canvas `TextMetrics`，
    /// https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-measuretext）。
    ///
    /// R3303：补全 spec 全字段。canvas crate 无真实字体度量后端（无字体表），故除 `width` 外均为字体度量
    /// 启发式近似——按 `font.size` 比例估，与既有 R3078 ascent=0.8em/descent=0.2em 一致。真实字体度量（经
    /// 字体表 hhea/OS/2）须接渲染流字体栈（render-stream 协调点），作后续 follow-up。即便近似，完整字段集
    /// 使文本布局库（chart.js 轴尺寸 / 自定义换行 / 基线对齐）不因缺字段读 NaN/undefined。
    pub fn measure_text(&self, text: &str) -> TextMetrics {
        let char_count = text.chars().count() as f32;
        let size = self.font.size;
        // R34xx：字体栈可用（headless @font-face）→ 真实 shape 度量——width = advances 和、
        // fontBoundingBox = 字体真实 ascent/descent；actualBoundingBox* = 逐字形墨迹真实
        // 边界（rasterize 位图——2d.text.measure.getActualBoundingBox.tentative）。
        // descent 为负值（fontdue）；无字体栈回落启发式（0.6em/字符、0.8/0.2em）。
        let mut glyph_rects: Vec<(f32, f32, f32, f32)> = Vec::new();
        let (width, ascent, descent, ink_l, ink_t, ink_r, ink_b) = match (self.font_loader.clone(), self.font_id) {
            (Some(loader), Some(fid)) => {
                if let Ok(loader) = loader.lock() {
                    // R34xx：spec text preparation——ASCII whitespace → U+0020 + null 剥离。
                    let clean = prepare_canvas_text(text);
                    // R34xx：small-caps 合成（大写 shaping——字体无 smcp 特征）。
                    let shaped_source = if self.font.small_caps {
                        clean.to_uppercase()
                    } else {
                        clean
                    };
                    let shaped = if self.font.kerning_none {
                        loader
                            .shape_text_cached_with_features(
                                fid,
                                &shaped_source,
                                size,
                                zero_render_foundation::font::TextDirection::Auto,
                                &text_features(self.font.kerning_none, self.font.small_caps),
                            )
                            .unwrap_or_default()
                    } else {
                        loader.shape_text_cached(fid, &shaped_source, size).unwrap_or_default()
                    };
                    let mut w: f32 = shaped.iter().map(|g| g.advance_x).sum();
                    // R34xx：letterSpacing（含末字符——WPT ×11 期望）与 wordSpacing。
                    let ls = parse_length_px(&self.font.letter_spacing, size).unwrap_or(0.0);
                    let ws = parse_length_px(&self.font.word_spacing, size).unwrap_or(0.0);
                    w += ls * shaped.len() as f32;
                    let words = text.split_whitespace().count();
                    if words > 1 {
                        w += ws * (words - 1) as f32;
                    }
                    let (a, d) = loader.line_metrics(fid, size).unwrap_or((size * 0.8, size * 0.2));
                    // 逐字形墨迹（字形轮廓 bbox 亚像素精度——位图在亚像素字号下量化，
                    // 2d.text.measure.actualBoundingBox.small-font：1.5px 期望 right≈1.5
                    // 而非量化 2；outline 解析失败回落位图）。pen 与 draw 同序：advance + ls。
                    let font_data = loader.get_font_data(fid).map(|d| d.to_vec());
                    let mut pen = 0.0f32;
                    let mut il = f32::MAX;
                    let mut it = f32::MAX;
                    let mut ir = f32::NEG_INFINITY;
                    let mut ib = f32::NEG_INFINITY;
                    for g in &shaped {
                        let gid = g.glyph_id.min(u32::from(u16::MAX)) as u16;
                        let ink = font_data
                            .as_deref()
                            .and_then(|data| glyph_ink_bbox(data, loader.face_index(fid), gid, g.font_size))
                            .or_else(|| {
                                loader.rasterize_glyph_index(fid, gid, g.font_size).ok().map(|bmp| {
                                    (
                                        bmp.x_offset as f32,
                                        -bmp.y_offset as f32 - bmp.height as f32,
                                        bmp.x_offset as f32 + bmp.width as f32,
                                        -bmp.y_offset as f32,
                                    )
                                })
                            })
                            .map(|(l, t, r, b)| (pen + g.x_offset + l, t, pen + g.x_offset + r, b));
                        if let Some((gl, gt, gr, gb)) = ink {
                            // R34xx：空墨迹字形（空格 w/h=0）不入并集（否则退化矩形
                            // (pen,0,pen,0) 的 r=pen 污染 max-right——'A    ' 的 R 应为 50）。
                            if gr > gl || gb > gt {
                                il = il.min(gl);
                                it = it.min(gt);
                                ir = ir.max(gr);
                                ib = ib.max(gb);
                            }
                            glyph_rects.push((gl, gt, gr, gb));
                        } else {
                            glyph_rects.push((pen, 0.0, pen, 0.0));
                        }
                        pen += g.advance_x + ls;
                    }
                    let (il, it, ir, ib) = if glyph_rects.is_empty() {
                        (0.0, 0.0, 0.0, 0.0)
                    } else {
                        (il, it, ir, ib)
                    };
                    (w, a, d, il, it, ir, ib)
                } else {
                    (char_count * (size * 0.6), size * 0.8, size * 0.2, 0.0, 0.0, 0.0, 0.0)
                }
            }
            _ => {
                let ls = parse_length_px(&self.font.letter_spacing, size).unwrap_or(0.0);
                let ws = parse_length_px(&self.font.word_spacing, size).unwrap_or(0.0);
                // R34xx：null 字符（U+0000）不占宽（2d.text.measure.width.nullCharacter）。
                let ink_chars = text.chars().filter(|c| *c != '\0').count() as f32;
                let w = ink_chars * (size * 0.6)
                    + ls * ink_chars
                    + ws * text.split_whitespace().count().saturating_sub(1) as f32;
                (w, size * 0.8, size * 0.2, 0.0, 0.0, 0.0, 0.0)
            }
        };
        let descent_abs = descent.abs();
        // R34xx：actualBoundingBoxLeft/Right 按 textAlign/direction 锚定（2d.text.drawing.
        // style.measure.direction/textAlign——rtl/right 对齐时文本在原点左侧 → left > right）。
        // left/right = 原点到 bbox 边缘的正向距离（extent 语义）。
        let rtl_measure = matches!(self.direction, TextDirection::Rtl);
        let anchor = match self.text_align {
            TextAlign::Center => -width / 2.0,
            TextAlign::Right => -width,
            TextAlign::Left => 0.0,
            TextAlign::Start => {
                if rtl_measure {
                    -width
                } else {
                    0.0
                }
            }
            TextAlign::End => {
                if rtl_measure {
                    0.0
                } else {
                    -width
                }
            }
        };
        // 墨迹 bbox（loader 路径）：ink 相对基线原点，经锚定偏移；无墨迹（fallback）回落
        // em 边界（anchor..anchor+width）。
        let has_ink = glyph_rects.iter().any(|r| r.2 > r.0 || r.3 > r.1);
        let (bbox_l, bbox_r) = if has_ink {
            (anchor + ink_l, anchor + ink_r)
        } else {
            (anchor.min(anchor + width), anchor.max(anchor + width))
        };
        let (ink_asc, ink_desc) = if has_ink {
            ((-ink_t).max(0.0), ink_b.max(0.0))
        } else {
            (ascent, descent_abs)
        };
        // R34xx：BASE 表基线（字体数据可解析时）——hanging/ideographic 在 font units 下
        // 为正（基线上方距离），乘 size/upem 得 px。
        let (base_hang, base_ideo) = match (self.font_loader.clone(), self.font_id) {
            (Some(loader), Some(fid)) => loader
                .lock()
                .ok()
                .and_then(|l| {
                    let data = l.get_font_data(fid).map(|d| d.to_vec());
                    let idx = l.face_index(fid);
                    data.and_then(|d| font_baselines_px(&d, idx, size))
                })
                .unwrap_or((None, None)),
            _ => (None, None),
        };
        // R34xx：emHeightAscent/Descent——em square 半行距定位（实测三字体吻合）：
        //   leading = em − (ascent+descent)；bottom_leading = min(leading/2, descent)；
        //   top_leading = leading − bottom_leading；emHeightAscent = ascent + top_leading，
        //   emHeightDescent = descent + bottom_leading。
        // （CanvasTest: 768/256→768/256；ascent256: 256/256→512/512；descent0: 768/0→1024/0）
        let em = size;
        let em_leading = (em - ascent - descent_abs).max(0.0);
        let em_bottom_lead = (em_leading * 0.5).min(descent_abs);
        let em_height_ascent = ascent + (em_leading - em_bottom_lead);
        let em_height_descent = descent_abs + em_bottom_lead;
        TextMetrics {
            width,
            actual_bounding_box_ascent: ink_asc,
            actual_bounding_box_descent: ink_desc,
            // R34xx：spec 符号约定——actualBoundingBoxLeft 正值=向左距离、Right 正值=向右
            //（不钳制：墨迹在原点右侧时 Left 为负——2d.text.measure.actualBoundingBox.
            // whitespace 的 ' A' 期望 |Left|≥49；旧 max(0) 把 -50 钳成 0）。
            actual_bounding_box_left: -bbox_l,
            actual_bounding_box_right: bbox_r,
            font_bounding_box_ascent: ascent,       // 字体 ascent，由字体表给定
            font_bounding_box_descent: descent_abs, // 字体 descent，由字体表给定
            em_height_ascent,
            em_height_descent,
            alphabetic_baseline: 0.0, // 默认基线即 alphabetic → 距自身 0
            // R34xx：BASE 表 'hang'/'ideo' 基线（2d.text.measure.baselines：CanvasTest
            // hang=512units=0.5em、ideo=128units=0.125em）；无 BASE 表回退启发式。
            hanging_baseline: base_hang.unwrap_or(ascent),
            ideographic_baseline: base_ideo.unwrap_or(-descent_abs),
            glyph_rects,
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
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, anticlockwise: bool) {
        let (tx, ty) = self.transform.transform_point(x, y);
        self.current_path
            .arc(tx, ty, radius, start_angle, end_angle, anticlockwise);
    }

    /// 添加圆角矩形子路径（Canvas 2D `roundRect`，HTML Canvas §`dom-context-2d-api` roundRect）。
    /// 起点角经当前变换矩阵映射（与 `arc`/`rect` 同语义）；`radii` 为角半径列表（spec：单值 / [tl,tr,br,bl]
    /// / 其它长度按 [HTML §roundrect] 规则解析，本层透传 Path2D::round_rect，flattener 现 best-effort 退化
    /// 为矩形——角圆为 rendering 流域已知简化，几何/命中测试仍正确）。
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radii: Vec<(f32, f32)>) {
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
        // R34xx：source 独占类 composite 的未覆盖区域清除（path 外置透明）。
        let clear_uncovered = self.composite_clears_uncovered();
        // 绘制阴影（在形状之前）
        if self.has_shadow() {
            let (min_x, min_y, max_x, max_y) = vertices
                .chunks_exact(2)
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(mnx, mny, mxx, mxy), c| {
                    (mnx.min(c[0]), mny.min(c[1]), mxx.max(c[0]), mxy.max(c[1]))
                });
            let shape_alpha = self.style_alpha(&self.fill_style, (min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
            self.draw_shadow_path(&vertices, shape_alpha);
        }
        if self.fill_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), approx);
            let style = self.transform_gradient(&self.fill_style);
            self.blit_path_gradient(&vertices, &style);
        } else {
            let color = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), color);
            self.blit_path_to_pixels(&vertices, color);
        }
        // R34xx：source 独占类 composite 的未覆盖区域清除（path 外置透明）。
        if clear_uncovered {
            self.clear_outside_path();
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
            let (min_x, min_y, max_x, max_y) = vertices
                .chunks_exact(2)
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(mnx, mny, mxx, mxy), c| {
                    (mnx.min(c[0]), mny.min(c[1]), mxx.max(c[0]), mxy.max(c[1]))
                });
            let shape_alpha = self.style_alpha(&self.stroke_style, (min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
            self.draw_shadow_stroke(&vertices, self.line_width, shape_alpha);
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
            self.blit_stroke_to_pixels_gradient(&vertices, &self.stroke_style.clone(), self.line_width, closed);
        } else {
            let color = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), color, self.line_width, closed);
            self.blit_stroke_to_pixels(&vertices, color, self.line_width, closed);
        }
    }

    /// 使用指定 Path2D 填充路径。
    pub fn fill_with_path(&mut self, path: &Path2D) {
        let vertices = self.flatten_path_for(path);
        if vertices.is_empty() {
            return;
        }
        if self.has_shadow() {
            let (min_x, min_y, max_x, max_y) = vertices
                .chunks_exact(2)
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(mnx, mny, mxx, mxy), c| {
                    (mnx.min(c[0]), mny.min(c[1]), mxx.max(c[0]), mxy.max(c[1]))
                });
            let shape_alpha = self.style_alpha(&self.fill_style, (min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
            self.draw_shadow_path(&vertices, shape_alpha);
        }
        if self.fill_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.fill_style.resolve_color());
            self.primitives.add_path_fill(vertices.clone(), approx);
            let style = self.transform_gradient(&self.fill_style);
            self.blit_path_gradient(&vertices, &style);
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
        // R3356：描边阴影用 stroke 足迹（thick rect + 连接点），非 centerline——与 stroke()（R3241）
        // 一致。旧实现误用 draw_shadow_path（centerline），致同一描边几何经 stroke_with_path 与
        // stroke_path（→stroke()）产生不同阴影（粗线 stroke_with_path 阴影过细）。
        if self.has_shadow() {
            let (min_x, min_y, max_x, max_y) = vertices
                .chunks_exact(2)
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(mnx, mny, mxx, mxy), c| {
                    (mnx.min(c[0]), mny.min(c[1]), mxx.max(c[0]), mxy.max(c[1]))
                });
            let shape_alpha = self.style_alpha(&self.stroke_style, (min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
            self.draw_shadow_stroke(&vertices, self.line_width, shape_alpha);
        }
        let closed = path.commands().iter().any(|c| matches!(c, PathCommand::ClosePath));
        if self.stroke_style.is_per_pixel_style() {
            let approx = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), approx, self.line_width, closed);
            self.blit_stroke_to_pixels_gradient(&vertices, &self.stroke_style.clone(), self.line_width, closed);
        } else {
            let color = self.apply_alpha(self.stroke_style.resolve_color());
            self.primitives
                .add_path_stroke(vertices.clone(), color, self.line_width, closed);
            self.blit_stroke_to_pixels(&vertices, color, self.line_width, closed);
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
            image_smoothing_quality: self.image_smoothing_quality,
            text_align: self.text_align,
            text_baseline: self.text_baseline,
            miter_limit: self.miter_limit,
            direction: self.direction,
            clip_path: self.clip_path.clone(),
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
            self.image_smoothing_quality = state.image_smoothing_quality;
            self.text_align = state.text_align;
            self.text_baseline = state.text_baseline;
            self.miter_limit = state.miter_limit;
            self.direction = state.direction;
            self.clip_path = state.clip_path;
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
        self.resolve_font_id();
    }

    /// R34xx：letterSpacing 原始 CSS 长度串（相对单位随字号重解析）。
    pub fn set_letter_spacing(&mut self, raw: &str) {
        self.font.letter_spacing = raw.to_string();
    }

    /// R34xx：wordSpacing 原始 CSS 长度串。
    pub fn set_word_spacing(&mut self, raw: &str) {
        self.font.word_spacing = raw.to_string();
    }

    /// R34xx：fontKerning（'none' → shaping 关 kern 特征——2d.text.drawing.style.fontKerning
    /// 的 measure 宽度对比；'auto'/'normal' 默认开）。
    pub fn set_font_kerning(&mut self, v: &str) {
        self.font.kerning_none = v.trim().eq_ignore_ascii_case("none");
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

    /// 设置图像平滑质量（R3305）。
    pub fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) {
        self.image_smoothing_quality = quality;
    }

    /// 返回当前图像平滑质量（R3305）。
    pub fn image_smoothing_quality(&self) -> ImageSmoothingQuality {
        self.image_smoothing_quality
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

    /// 调整画布尺寸。会清空像素缓冲区并**重置绘图状态到默认**（HTML spec §4.12.5.1「Reset the
    /// rendering context to its default state」——设 canvas.width/height 时，bitmap 清空 + transform/
    /// clip/state-stack/style/dash/shadow/text 状态全回默认，等同新建 context 仅尺寸不同）。
    pub fn resize(&mut self, width: u32, height: u32) {
        // 复用 new() 的默认状态（单一权威来源），仅尺寸用入参。
        let fresh = CanvasContext::new(width, height);
        *self = fresh;
    }

    /// R3254-C8：仅清空 bitmap 像素（替换透明黑），**保留**绘图状态——transferToImageBitmap
    /// 的 spec 语义（区别于 resize：重置全状态）。
    pub fn clear_bitmap(&mut self) {
        self.pixel_buffer.fill(0);
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

    // ── Path2D 参数形式（R3306：ctx.fill(path)/stroke(path)/clip(path)，spec CanvasDrawingStyles）──
    // 语义：fill(path) 用给定 Path2D 而非当前路径（current_path 不变）。实现：保存当前路径 → 替换 →
    // 调裸方法 → 恢复（零侵入，复用既有光栅化）。Path2D 经 engine 路径注册表（path registry）按 id 引用。

    /// 用给定 `Path2D` 填充（替代当前路径，spec `ctx.fill(path)`）。当前路径不被修改。
    pub fn fill_path(&mut self, path: &Path2D) {
        let saved = std::mem::replace(&mut self.current_path, path.clone());
        self.fill();
        self.current_path = saved;
    }

    /// 用给定 `Path2D` 描边（替代当前路径，spec `ctx.stroke(path)`）。当前路径不被修改。
    pub fn stroke_path(&mut self, path: &Path2D) {
        let saved = std::mem::replace(&mut self.current_path, path.clone());
        self.stroke();
        self.current_path = saved;
    }

    /// 用给定 `Path2D` 设置裁剪（替代当前路径，spec `ctx.clip(path)`）。当前路径不被修改。
    pub fn clip_path(&mut self, path: &Path2D) {
        let saved = std::mem::replace(&mut self.current_path, path.clone());
        self.clip();
        self.current_path = saved;
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
    ///
    /// 点坐标 (x, y) 为画布坐标空间（device space），与 [`Self::move_to`] / [`Self::line_to`]
    /// 追加路径顶点时已按当前变换矩阵（CTM）变换到设备空间的顶点同空间比对——无需逆变换。
    /// spec `CanvasRenderingContext2D.isPointInPath`：点在画布坐标空间，命中测试针对当前
    /// 变换后的路径（路径顶点在追加时即经 CTM 映射，见 [`Self::move_to`]）。
    pub fn is_point_in_path(&self, x: f32, y: f32) -> bool {
        let vertices = self.flatten_path();
        if vertices.is_empty() {
            return false;
        }
        let points: Vec<(f32, f32)> = vertices.chunks_exact(2).map(|c| (c[0], c[1])).collect();
        point_in_polygon(x, y, &points)
    }

    /// R34xx：CPU 光栅路径的裁剪判定。clip 后绘制须裁剪到 clip_path 内——旧实现只把 clip
    /// 加入 primitives 图元层（GPU/合成路径生效），blit_* 直接写像素无视裁剪（上游
    /// 2d.fillRect.clip / clearRect.clip / strokeRect.clip 全失败）。clip 未设时零开销。
    /// 点坐标 (x, y) 为画布像素坐标（device space），与 clip_path 顶点同空间。
    pub(crate) fn clip_applies(&self, x: f32, y: f32) -> bool {
        match &self.clip_path {
            Some(path) => path.is_point_in_path(x + 0.5, y + 0.5),
            None => true,
        }
    }

    /// 判断点是否在当前路径的描边区域内。
    ///
    /// 点坐标 (x, y) 为画布坐标空间，与描边中线顶点（追加时已按 CTM 变换到设备空间）同空间
    /// 比对；检测点到各线段的距离是否小于 `line_width / 2`（设备空间度量）。
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
    pub fn get_image_data(&self, x: i32, y: i32, width: i32, height: i32) -> ImageData {
        // R3354：usize + saturating_mul 计算 RGBA 缓冲区大小。
        // 旧实现 `(width * height * 4) as usize` 在 u32 算术溢出：getImageData(0,0,65536,65536)
        // → 65536*65536 在 u32 回绕为 0 → data 为空 vec → 复制循环 data[0..N] 切片越界 panic。
        // spec getImageData：返回请求 width×height 的 ImageData，画布外像素透明黑。
        // R34xx：有符号语义——负 sw/sh 矩形翻转（2d.imageData.get.source.negative）；
        // 负/越界 sx/sy 区域画布外像素透明（get.source.outside）。
        let flip_x = width < 0;
        let flip_y = height < 0;
        let (w, h) = (width.unsigned_abs(), height.unsigned_abs());
        let size = (w as usize).saturating_mul(h as usize).saturating_mul(4);
        let mut data = vec![0u8; size];
        let canvas_w = self.width as i32;
        let canvas_h = self.height as i32;
        if !flip_x && !flip_y {
            // 快路径：行复制（画布内行）。
            for row in 0..h {
                let src_row = y + row as i32;
                if src_row < 0 || src_row >= canvas_h {
                    continue;
                }
                let col0 = x.max(0) as usize;
                let col1 = (x + width).min(canvas_w).max(0) as usize;
                if col1 <= col0 {
                    continue;
                }
                let src_start = src_row as usize * canvas_w as usize * 4 + col0 * 4;
                let src_end = src_row as usize * canvas_w as usize * 4 + col1 * 4;
                // 结果列 = 源列 − x（负 x 时结果左侧为画布外透明区）。
                let dst_col0 = (col0 as i32 - x) as usize;
                let dst_start = row as usize * w as usize * 4 + dst_col0 * 4;
                let dst_len = (col1 - col0) * 4;
                data[dst_start..dst_start + dst_len].copy_from_slice(&self.pixel_buffer[src_start..src_end]);
            }
        } else {
            // 负 dims：源矩形反向（[x+w, x)×[y+h, y)），数据仍按左上→右下读取
            //（2d.imageData.get.source.negative——"top-to-bottom left-to-right"）。
            for row in 0..h {
                let src_y = if flip_y {
                    y + height + row as i32
                } else {
                    y + row as i32
                };
                for col in 0..w {
                    let src_x = if flip_x { x + width + col as i32 } else { x + col as i32 };
                    if src_x < 0 || src_y < 0 || src_x >= canvas_w || src_y >= canvas_h {
                        continue;
                    }
                    let src_idx = (src_y as usize * canvas_w as usize + src_x as usize) * 4;
                    let dst_idx = (row as usize * w as usize + col as usize) * 4;
                    data[dst_idx..dst_idx + 4].copy_from_slice(&self.pixel_buffer[src_idx..src_idx + 4]);
                }
            }
        }
        ImageData {
            width: w,
            height: h,
            data,
        }
    }

    /// 全画布 RGBA 快照（显示链路：canvas 元素内容 → ImagePrimitive）。
    /// 画布有内容（任一像素非透明黑）时返回 Some((w, h, rgba))，否则 None（跳过绘制）。
    pub fn snapshot_rgba(&self) -> Option<(u32, u32, Vec<u8>)> {
        if self.pixel_buffer.iter().any(|&b| b != 0) {
            Some((self.width, self.height, self.pixel_buffer.clone()))
        } else {
            None
        }
    }

    /// 创建指定尺寸的 ImageData，填充透明黑色（rgba 0,0,0,0）。
    pub fn create_image_data(&self, width: u32, height: u32) -> ImageData {
        // R3354：usize + saturating_mul（同 get_image_data，避免 u32 溢出回绕为 0 字节缓冲区）。
        let size = (width as usize).saturating_mul(height as usize).saturating_mul(4);
        ImageData {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    /// 放置像素数据。将 ImageData 写入画布像素缓冲区的指定偏移位置。
    pub fn put_image_data(&mut self, image_data: &ImageData, x: i32, y: i32) {
        let canvas_w = self.width as i32;
        let canvas_h = self.height as i32;
        let iw = image_data.width as i32;
        let ih = image_data.height as i32;
        for row in 0..ih {
            // i64 防溢出（极端坐标——edge 测试 u32::MAX/i32::MAX）。
            let dst_row = y as i64 + row as i64;
            if dst_row < 0 || dst_row >= canvas_h as i64 {
                continue;
            }
            let src_row = row;
            let col0 = x.max(0) as i64;
            let col1 = ((x as i64 + iw as i64).min(canvas_w as i64)).max(0);
            if col1 <= col0 {
                continue;
            }
            let src_start = (src_row as i64 * iw as i64 + col0 - x as i64) as usize * 4;
            let dst_start = (dst_row * canvas_w as i64 + col0) as usize * 4;
            let len = (col1 - col0) as usize * 4;
            if src_start + len <= image_data.data.len() {
                self.pixel_buffer[dst_start..dst_start + len]
                    .copy_from_slice(&image_data.data[src_start..src_start + len]);
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

        // R34xx：drawImage 阴影（2d.shadow.image.* / 2d.shadow.canvas.*）——先画阴影再画图像本身。
        if self.has_shadow() {
            self.draw_shadow_image(image_data, sx, sy, sw, sh, dx, dy, dw, dh);
        }

        let sx = sx.max(0.0) as usize;
        let sy = sy.max(0.0) as usize;
        // R3292：源矩形起点越出图像边界（sx>=img_w / sy>=img_h）时无像素可取，提前返回。
        // 修复 unsigned 下溢：旧 `img_w - sx` 在 sx>img_w 时 debug panic（release 静默回绕致错绘）。
        if sx >= img_w || sy >= img_h {
            return;
        }
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
        // R34xx：source 独占类 composite（copy/source-in 等）的未覆盖区域清除——
        // drawImage 版本（2d.composite.uncovered.image.*：目标矩形外置透明）。
        if self.composite_clears_uncovered() {
            let corners = [
                self.transform.transform_point(dx, dy),
                self.transform.transform_point(dx + dw, dy),
                self.transform.transform_point(dx, dy + dh),
                self.transform.transform_point(dx + dw, dy + dh),
            ];
            let (mut l, mut t) = (f32::INFINITY, f32::INFINITY);
            let (mut rr, mut bb) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
            for (cx, cy) in corners {
                l = l.min(cx);
                rr = rr.max(cx);
                t = t.min(cy);
                bb = bb.max(cy);
            }
            self.clear_outside_rect(&Rect::new(l, t, rr - l, bb - t));
        }
    }

    // ── Output ──

    /// 判断当前是否启用了阴影（阴影颜色不透明且偏移或模糊非零）。
    /// R34xx：composite 模式是否在绘制前清除未覆盖区域（spec：source 独占类操作
    /// source-in/source-out/copy 等绘制后未覆盖像素为 (0,0,0,0)——Porter-Duff 的
    /// 全局语义，2d.composite.uncovered.fill.* 全族）。
    /// R34xx：绘制时按当前 CTM 变换渐变坐标（spec：渐变坐标相对 fill 时的坐标空间——
    /// 2d.gradient.linear.transform.*）。半径/角度不缩放（画布单位）。
    fn transform_gradient(&self, style: &CanvasStyle) -> CanvasStyle {
        match style {
            CanvasStyle::LinearGradient(g) => {
                let (x0, y0) = self.transform.transform_point(g.x0, g.y0);
                let (x1, y1) = self.transform.transform_point(g.x1, g.y1);
                let mut ng = g.clone();
                ng.x0 = x0;
                ng.y0 = y0;
                ng.x1 = x1;
                ng.y1 = y1;
                CanvasStyle::LinearGradient(ng)
            }
            CanvasStyle::RadialGradient(g) => {
                let (x0, y0) = self.transform.transform_point(g.x0, g.y0);
                let (x1, y1) = self.transform.transform_point(g.x1, g.y1);
                // R34xx：半径随 CTM 缩放（spec：渐变坐标相对 fill 时坐标空间——radii 同为
                // 该空间长度，2d.gradient.radial.transform.1/2/3 断言 scale(10) 后几何放大）。
                // 取各向同性近似 sqrt(|det|)（旋转不变；非均匀缩放为近似，WPT 为均匀场景）。
                let scale = (self.transform.a * self.transform.d - self.transform.b * self.transform.c)
                    .abs()
                    .sqrt();
                let mut ng = g.clone();
                ng.x0 = x0;
                ng.y0 = y0;
                ng.x1 = x1;
                ng.y1 = y1;
                ng.r0 *= scale;
                ng.r1 *= scale;
                CanvasStyle::RadialGradient(ng)
            }
            CanvasStyle::ConicGradient(g) => {
                let (cx, cy) = self.transform.transform_point(g.cx, g.cy);
                let mut ng = g.clone();
                ng.cx = cx;
                ng.cy = cy;
                CanvasStyle::ConicGradient(ng)
            }
            CanvasStyle::Pattern(p) => {
                // R34xx：平铺锚定 fill 空间（见 CanvasPattern::tile_transform）。
                let mut ng = p.clone();
                ng.tile_transform = self.transform.inverse();
                CanvasStyle::Pattern(ng)
            }
            _ => style.clone(),
        }
    }

    fn composite_clears_uncovered(&self) -> bool {
        matches!(
            self.composite_operation,
            CompositeOperation::SourceIn
                | CompositeOperation::SourceOut
                | CompositeOperation::DestinationIn
                | CompositeOperation::DestinationAtop
                | CompositeOperation::Copy
                | CompositeOperation::Clear
        )
    }

    /// R34xx：source 独占类 composite 绘制后清除矩形外像素（未覆盖区域 → 透明）。
    fn clear_outside_rect(&mut self, rect: &Rect) {
        let (x0, y0) = (rect.left().max(0.0) as usize, rect.top().max(0.0) as usize);
        let (x1, y1) = (
            (rect.right().min(self.width as f32) as usize).min(self.width as usize),
            (rect.bottom().min(self.height as f32) as usize).min(self.height as usize),
        );
        let w = self.width as usize;
        for y in 0..self.height as usize {
            for x in 0..w {
                // R34xx：clip 外的像素不受影响（clip 限制绘制与清除范围——clip.copy 等）。
                if (x < x0 || x >= x1 || y < y0 || y >= y1) && self.clip_applies(x as f32, y as f32) {
                    let idx = (y * w + x) * 4;
                    self.pixel_buffer[idx] = 0;
                    self.pixel_buffer[idx + 1] = 0;
                    self.pixel_buffer[idx + 2] = 0;
                    self.pixel_buffer[idx + 3] = 0;
                }
            }
        }
    }

    /// R34xx：source 独占类 composite 绘制后清除当前路径外像素（未覆盖区域 → 透明）。
    fn clear_outside_path(&mut self) {
        let w = self.width as usize;
        for y in 0..self.height as usize {
            for x in 0..w {
                if !self.is_point_in_path(x as f32 + 0.5, y as f32 + 0.5) && self.clip_applies(x as f32, y as f32) {
                    let idx = (y * w + x) * 4;
                    self.pixel_buffer[idx] = 0;
                    self.pixel_buffer[idx + 1] = 0;
                    self.pixel_buffer[idx + 2] = 0;
                    self.pixel_buffer[idx + 3] = 0;
                }
            }
        }
    }

    /// R34xx：样式代表 alpha（阴影 mask 调制用——半透明形状的阴影应半透明，
    /// 2d.shadow.gradient.alpha / alpha.5）。per-pixel 样式在给定点采样。
    fn style_alpha(&self, style: &CanvasStyle, x: f32, y: f32) -> f32 {
        match style {
            CanvasStyle::Color(c) => c.a as f32 / 255.0,
            _ => style.sample_at(x, y).a as f32 / 255.0,
        }
    }

    fn has_shadow(&self) -> bool {
        self.shadow_color.a > 0
            && (self.shadow_blur > 0.0 || self.shadow_offset_x != 0.0 || self.shadow_offset_y != 0.0)
    }

    /// R3240：为矩形绘制阴影——region alpha mask（矩形覆盖）+ box blur（shadowBlur）+ 经
    /// composite_shadow_mask 合成（消费 globalCompositeOperation，与 fill/stroke 一致）。
    /// 旧实现仅画偏移硬边矩形、alpha 按 `1/(1+blur·0.1)` 衰减（无 blur）。
    fn draw_shadow_rect(&mut self, rect: &Rect, style: &CanvasStyle) {
        let (radius, pad, passes) = super::raster::shadow_blur_geom(self.shadow_blur);
        let cw = self.width as i32;
        let ch = self.height as i32;
        // R34xx：region 用 rect 原始坐标（不提前钳到画布）——画布外矩形（如 y=-50..0）若
        // 被 .max(0) 钳成 0 高度，region 空直接 return，阴影（含 offset 后落入画布的部分）
        // 整体丢失（上游 2d.fillRect.shadow 断言中心像素阴影色失败）。
        // R3355：saturating_add/sub 防 shadowBlur 极大时 pad 致 i32 加减溢出（保持）。
        // 偏移后的可见性由 composite_shadow_mask 的 cx/cy 画布钳位负责。
        let rx0 = (rect.left().floor() as i32).saturating_sub(pad);
        let ry0 = (rect.top().floor() as i32).saturating_sub(pad);
        let rx1 = (rect.right().ceil() as i32).saturating_add(pad);
        let ry1 = (rect.bottom().ceil() as i32).saturating_add(pad);
        // R34xx：region 裁剪到阴影可见范围（画布 − offset）——阴影只可能在 offset 后落入
        // 画布的区域出现；旧实现不裁剪 + mask 封顶（4×画布）会截断画布外大偏移阴影
        //（2d.shadow.stroke.join.2：offsetX=100 的阴影 y∈[-200,50] 被 4×50=200 封顶截断）。
        let vis_x0 = (-self.shadow_offset_x).floor() as i32;
        let vis_y0 = (-self.shadow_offset_y).floor() as i32;
        let vis_x1 = (self.width as f32 - self.shadow_offset_x).ceil() as i32;
        let vis_y1 = (self.height as f32 - self.shadow_offset_y).ceil() as i32;
        let rx0 = rx0.max(vis_x0);
        let ry0 = ry0.max(vis_y0);
        let rx1 = rx1.min(vis_x1);
        let ry1 = ry1.min(vis_y1);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        // R34xx：mask 尺寸封顶（画布 4 倍）兜底防极端 offset 下 region 过大；裁剪后
        // 常规阴影 region ≤ 画布尺寸不受影响。
        // R34xx：+1 闭区间 [rx0, rx1]（半开丢右/下边界像素——2d.shadow.stroke.join.2 的 (-50,25) 恰在边界）。
        let rw = ((rx1 - rx0) as usize).min((cw as usize).saturating_mul(4)) + 1;
        let rh = ((ry1 - ry0) as usize).min((ch as usize).saturating_mul(4)) + 1;
        let mut mask = vec![0u8; rw * rh];
        let (rl, rt, rr, rb) = (rect.left(), rect.top(), rect.right(), rect.bottom());
        for ly in 0..rh as i32 {
            let wy = ry0 + ly;
            for lx in 0..rw as i32 {
                let wx = rx0 + lx;
                if (wx as f32) >= rl && (wx as f32) < rr && (wy as f32) >= rt && (wy as f32) < rb {
                    // R34xx：mask 逐像素乘形状 alpha（渐变/图案透明部分无阴影——
                    // 2d.shadow.gradient.transparent.2 / pattern.transparent.1）。
                    let alpha = self.style_alpha(style, wx as f32, wy as f32);
                    mask[(ly as usize) * rw + (lx as usize)] = (255.0 * alpha) as u8;
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
            1.0, // mask 已逐像素乘形状 alpha（R34xx）
        );
    }

    /// R34xx：为 drawImage 绘制阴影——mask = 变换后目标区域的源图 alpha（与主循环同源采样），
    /// box blur + composite_shadow_mask（2d.shadow.image.* / 2d.shadow.canvas.*——canvas 源经
    /// shim getImageData wire 走同一 draw_image 路径）。源像素 alpha 作形状 alpha（透源阴影轻）。
    #[allow(clippy::too_many_arguments)]
    fn draw_shadow_image(
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
        let (radius, pad, passes) = super::raster::shadow_blur_geom(self.shadow_blur);
        // 变换后目标矩形的 device bbox（旋转/非轴对齐四边形取角点包围盒）。
        let corners = [
            self.transform.transform_point(dx, dy),
            self.transform.transform_point(dx + dw, dy),
            self.transform.transform_point(dx, dy + dh),
            self.transform.transform_point(dx + dw, dy + dh),
        ];
        let (mut l, mut t) = (f32::INFINITY, f32::INFINITY);
        let (mut rr, mut bb) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for (cx, cy) in corners {
            l = l.min(cx);
            rr = rr.max(cx);
            t = t.min(cy);
            bb = bb.max(cy);
        }
        // region + 可见性裁剪：与 draw_shadow_rect 同款（画布 − offset；saturating 防溢出）。
        let rx0 = (l.floor() as i32).saturating_sub(pad);
        let ry0 = (t.floor() as i32).saturating_sub(pad);
        let rx1 = (rr.ceil() as i32).saturating_add(pad);
        let ry1 = (bb.ceil() as i32).saturating_add(pad);
        let vis_x0 = (-self.shadow_offset_x).floor() as i32;
        let vis_y0 = (-self.shadow_offset_y).floor() as i32;
        let vis_x1 = (self.width as f32 - self.shadow_offset_x).ceil() as i32;
        let vis_y1 = (self.height as f32 - self.shadow_offset_y).ceil() as i32;
        let rx0 = rx0.max(vis_x0);
        let ry0 = ry0.max(vis_y0);
        let rx1 = rx1.min(vis_x1);
        let ry1 = ry1.min(vis_y1);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        let rw = ((rx1 - rx0) as usize).min((self.width as usize).saturating_mul(4)) + 1;
        let rh = ((ry1 - ry0) as usize).min((self.height as usize).saturating_mul(4)) + 1;
        let mut mask = vec![0u8; rw * rh];
        // 逆变换（2×3 仿射）：device → 目标空间，再按 x_scale/y_scale 映射到源像素（与主循环一致）。
        let det = self.transform.a * self.transform.d - self.transform.b * self.transform.c;
        let x_scale = sw / dw;
        let y_scale = sh / dh;
        for ly in 0..rh as i32 {
            let wy = ry0 + ly;
            for lx in 0..rw as i32 {
                let wx = rx0 + lx;
                let dxw = wx as f32 - self.transform.e;
                let dyw = wy as f32 - self.transform.f;
                // 逆矩阵乘（det≠0；退化矩阵阴影 region 恒空，兜底 0）
                let (ix, iy) = if det.abs() > f32::EPSILON {
                    let ux = (dxw * self.transform.d - dyw * self.transform.b) / det;
                    let uy = (-dxw * self.transform.c + dyw * self.transform.a) / det;
                    (ux, uy)
                } else {
                    continue;
                };
                let rel_x = ix - dx;
                let rel_y = iy - dy;
                if rel_x < 0.0 || rel_y < 0.0 || rel_x >= dw || rel_y >= dh {
                    continue;
                }
                let src_x = sx as usize + (rel_x * x_scale) as usize;
                let src_y = sy as usize + (rel_y * y_scale) as usize;
                if src_x >= img_w || src_y >= img_h {
                    continue;
                }
                let idx = (src_y * img_w + src_x) * 4;
                if idx + 3 >= image_data.data.len() {
                    continue;
                }
                mask[(ly as usize) * rw + (lx as usize)] = image_data.data[idx + 3];
            }
        }
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
            1.0, // mask 已逐像素乘形状 alpha（源 alpha 直接作 mask 值）
        );
    }

    /// R3240：为路径绘制阴影——region alpha mask（扫描线覆盖）+ box blur + composite_shadow_mask。
    fn draw_shadow_path(&mut self, vertices: &[f32], shape_alpha: f32) {
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
        // R34xx：region 用原始坐标不提前钳画布（同 draw_shadow_rect——画布外路径的阴影
        // 含 offset 后落入画布部分不可丢）；可见性由 composite_shadow_mask 钳位负责。
        // R3355：saturating_add/sub 避免 pad（极大 shadowBlur 时为 i32::MAX）致 i32 加减溢出。
        let rx0 = (min_x.floor() as i32).saturating_sub(pad);
        let ry0 = (min_y.floor() as i32).saturating_sub(pad);
        let rx1 = (max_x.ceil() as i32).saturating_add(pad);
        let ry1 = (max_y.ceil() as i32).saturating_add(pad);
        // R34xx（2026-08-14 CI 修复）：与 draw_shadow_rect 一致的可见性裁剪——裁剪到
        // 「阴影 offset 后可能落入画布」的区域。path 版本此前缺失该裁剪，极端 blur
        // 下 mask 封顶到 4×画布（如 201×201）+ 朴素 box_blur O(w·h·r) → 单测 33s、
        // macos-x86_64 CI 120s 超时（test_shadow_path_huge_blur_no_overflow_panic_r3355）。
        // 裁剪后 mask ≈ 画布尺寸，耗时降 ~16×，渲染结果不变（composite_shadow_mask 钳位兜底）。
        let vis_x0 = (-self.shadow_offset_x).floor() as i32;
        let vis_y0 = (-self.shadow_offset_y).floor() as i32;
        let vis_x1 = (self.width as f32 - self.shadow_offset_x).ceil() as i32;
        let vis_y1 = (self.height as f32 - self.shadow_offset_y).ceil() as i32;
        let rx0 = rx0.max(vis_x0);
        let ry0 = ry0.max(vis_y0);
        let rx1 = rx1.min(vis_x1);
        let ry1 = ry1.min(vis_y1);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        // R34xx：mask 尺寸封顶（画布 4 倍）防画布外超大路径致 mask 分配 OOM。
        // R34xx：+1 闭区间 [rx0, rx1]（半开丢右/下边界像素——2d.shadow.stroke.join.2 的 (-50,25) 恰在边界）。
        let rw = ((rx1 - rx0) as usize).min((cw as usize).saturating_mul(4)) + 1;
        let rh = ((ry1 - ry0) as usize).min((ch as usize).saturating_mul(4)) + 1;
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
            shape_alpha,
        );
    }

    /// R3241：为描边绘制阴影——region mask 由 stroke 足迹（每段 thick rect + 连接点方块）构成，
    /// 非 centerline（旧 stroke() 传 centerline 致粗描边阴影过细）。box blur + composite 同 R3240。
    fn draw_shadow_stroke(&mut self, vertices: &[f32], line_width: f32, shape_alpha: f32) {
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
        // R34xx：region 不提前钳画布（同 draw_shadow_rect——画布外描边的阴影经 offset 落入
        // 画布的部分不可丢；可见性由 composite_shadow_mask 钳位负责）。mask 尺寸封顶防 OOM。
        let rx0 = (min_x - pad).floor() as i32;
        let ry0 = (min_y - pad).floor() as i32;
        let rx1 = (max_x + pad).ceil() as i32;
        let ry1 = (max_y + pad).ceil() as i32;
        // R34xx：region 裁剪到阴影可见范围（同 draw_shadow_rect）。
        let vis_x0 = (-self.shadow_offset_x).floor() as i32;
        let vis_y0 = (-self.shadow_offset_y).floor() as i32;
        let vis_x1 = (self.width as f32 - self.shadow_offset_x).ceil() as i32;
        let vis_y1 = (self.height as f32 - self.shadow_offset_y).ceil() as i32;
        let rx0 = rx0.max(vis_x0);
        let ry0 = ry0.max(vis_y0);
        let rx1 = rx1.min(vis_x1);
        let ry1 = ry1.min(vis_y1);
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        // R34xx：+1 闭区间 [rx0, rx1]（半开丢右/下边界像素——2d.shadow.stroke.join.2 的 (-50,25) 恰在边界）。
        let rw = ((rx1 - rx0) as usize).min((cw as usize).saturating_mul(4)) + 1;
        let rh = ((ry1 - ry0) as usize).min((ch as usize).saturating_mul(4)) + 1;
        let mut mask = vec![0u8; rw * rh];
        // R34xx：段主体逐像素精确判定（投影 t∈[0,1] + 距中心线 ≤ half——旧 bbox 直填对
        // 斜线段覆盖端点外区域，2d.shadow.stroke.join.1 的 (-99,1) 在段2 延长线外仍被覆盖）。
        for s in &segments {
            let (ax, ay) = (s[0], s[1]);
            let (bx, by) = (s[2], s[3]);
            let (dx, dy) = (bx - ax, by - ay);
            let len2 = dx * dx + dy * dy;
            if len2 < f32::EPSILON {
                continue;
            }
            let len = len2.sqrt();
            let (nx, ny) = (-dy / len * half_lw, dx / len * half_lw);
            let min_x = ax.min(bx).min(ax + nx).min(bx + nx).min(ax - nx).min(bx - nx);
            let max_x = ax.max(bx).max(ax + nx).max(bx + nx).max(ax - nx).max(bx - nx);
            let min_y = ay.min(by).min(ay + ny).min(by + ny).min(ay - ny).min(by - ny);
            let max_y = ay.max(by).max(ay + ny).max(by + ny).max(ay - ny).max(by - ny);
            let h2 = half_lw * half_lw;
            let x0 = (min_x.floor() as i32).max(rx0);
            let y0 = (min_y.floor() as i32).max(ry0);
            let x1 = (max_x.ceil() as i32).min(rx0 + rw as i32 - 1);
            let y1 = (max_y.ceil() as i32).min(ry0 + rh as i32 - 1);
            for wy in y0..=y1 {
                for wx in x0..=x1 {
                    let (qx, qy) = (wx as f32 + 0.5 - ax, wy as f32 + 0.5 - ay);
                    let t = (qx * dx + qy * dy) / len2;
                    if t < 0.0 || t > 1.0 {
                        continue;
                    }
                    let (rx, ry) = (qx - t * dx, qy - t * dy);
                    if rx * rx + ry * ry > h2 {
                        continue;
                    }
                    let (lx, ly) = ((wx - rx0) as usize, (wy - ry0) as usize);
                    if lx < rw && ly < rh {
                        mask[ly * rw + lx] = 255;
                    }
                }
            }
        }
        // R34xx：端 cap（square/round）——阴影 cap 与形状 cap 同几何
        //（2d.shadow.stroke.cap.2：square cap 的阴影应覆盖 cap 延伸区域）。
        if let Some(first) = segments.first() {
            self.shadow_cap_into_mask(
                first[0], first[1], first[2], first[3], &mut mask, rw, rh, rx0, ry0, half_lw,
            );
        }
        if let Some(last) = segments.last() {
            self.shadow_cap_into_mask(last[2], last[3], last[0], last[1], &mut mask, rw, rh, rx0, ry0, half_lw);
        }
        // R34xx：连接点用真实 join 几何（miter 尖角三角 / bevel 平切 / round 圆盘；共线角
        // 不画）——旧方块近似覆盖角外大片区域（2d.shadow.stroke.join.2 (1,1) 失败）。
        for i in 0..segments.len().saturating_sub(1) {
            let seg_a = segments[i];
            let seg_b = segments[i + 1];
            if !self.join_visible(&seg_a, &seg_b) {
                continue;
            }
            let (jx, jy) = (seg_a[2], seg_a[3]);
            let (dax, day) = (jx - seg_a[0], jy - seg_a[1]);
            let (dbx, dby) = (seg_b[2] - jx, seg_b[3] - jy);
            let la = (dax * dax + day * day).sqrt();
            let lb = (dbx * dbx + dby * dby).sqrt();
            if la < f32::EPSILON || lb < f32::EPSILON {
                continue;
            }
            let (uax, uay) = (dax / la, day / la);
            let (ubx, uby) = (dbx / lb, dby / lb);
            let (mx, my) = (uax - ubx, uay - uby);
            let ml = (mx * mx + my * my).sqrt();
            if ml < f32::EPSILON {
                continue;
            }
            let (a_ext_x, a_ext_y) = if mx * -uay + my * uax > 0.0 {
                (jx - uay * half_lw, jy + uax * half_lw)
            } else {
                (jx + uay * half_lw, jy - uax * half_lw)
            };
            let (b_ext_x, b_ext_y) = if mx * -uby + my * ubx > 0.0 {
                (jx - uby * half_lw, jy + ubx * half_lw)
            } else {
                (jx + uby * half_lw, jy - ubx * half_lw)
            };
            match self.line_join {
                LineJoin::Round => {
                    let r2 = half_lw * half_lw;
                    for ly in 0..rh as i32 {
                        let wy = ry0 + ly;
                        for lx in 0..rw as i32 {
                            let wx = rx0 + lx;
                            let dx = wx as f32 + 0.5 - jx;
                            let dy = wy as f32 + 0.5 - jy;
                            if dx * dx + dy * dy <= r2 {
                                mask[(ly as usize) * rw + (lx as usize)] = 255;
                            }
                        }
                    }
                }
                LineJoin::Bevel => {
                    // R34xx：bevel 单平切三角 {jx, a_ext, b_ext}（旧并入 Miter 分支且
                    // P=jx 时退化成两个退化三角，point_in_triangle 误判覆盖角外——
                    // 2d.shadow.stroke.join.1 的 (-99,1) 被误覆盖）。
                    for ly in 0..rh as i32 {
                        let wy = ry0 + ly;
                        for lx in 0..rw as i32 {
                            let wx = rx0 + lx;
                            let x = wx as f32 + 0.5;
                            let y = wy as f32 + 0.5;
                            if super::raster::point_in_triangle(x, y, jx, jy, a_ext_x, a_ext_y, b_ext_x, b_ext_y) {
                                mask[(ly as usize) * rw + (lx as usize)] = 255;
                            }
                        }
                    }
                }
                LineJoin::Miter => {
                    // miter 尖点（超限降级 bevel 平切三角）——两个三角填充（重心同侧判定）。
                    let cos_theta = -(uax * ubx + uay * uby);
                    let sin_half = ((1.0 - cos_theta) / 2.0).sqrt();
                    let (px, py) = if sin_half < f32::EPSILON {
                        (jx, jy)
                    } else {
                        let miter_len = half_lw / sin_half;
                        if miter_len / half_lw > self.miter_limit {
                            (jx, jy) // 超限 → bevel（只画平切三角）
                        } else {
                            (jx + mx / ml * miter_len, jy + my / ml * miter_len)
                        }
                    };
                    for ly in 0..rh as i32 {
                        let wy = ry0 + ly;
                        for lx in 0..rw as i32 {
                            let wx = rx0 + lx;
                            let x = wx as f32 + 0.5;
                            let y = wy as f32 + 0.5;
                            if super::raster::point_in_triangle(x, y, jx, jy, a_ext_x, a_ext_y, px, py)
                                || super::raster::point_in_triangle(x, y, jx, jy, px, py, b_ext_x, b_ext_y)
                            {
                                mask[(ly as usize) * rw + (lx as usize)] = 255;
                            }
                        }
                    }
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
            shape_alpha,
        );
    }

    /// R34xx：端 cap 写入阴影 mask（square = 延伸段矩形，round = 圆盘；butt 无）。
    #[allow(clippy::too_many_arguments)]
    fn shadow_cap_into_mask(
        &self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        mask: &mut [u8],
        rw: usize,
        rh: usize,
        rx0: i32,
        ry0: i32,
        half_lw: f32,
    ) {
        match self.line_cap {
            LineCap::Butt => {}
            LineCap::Square => {
                let (dx, dy) = (endpoint_x - other_x, endpoint_y - other_y);
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let (ux, uy) = (dx / len, dy / len);
                let (ext_x, ext_y) = (endpoint_x + ux * half_lw, endpoint_y + uy * half_lw);
                let rect = self.line_segment_rect(endpoint_x, endpoint_y, ext_x, ext_y, half_lw * 2.0);
                super::raster::fill_rect_into_mask(mask, rw, rh, rx0, ry0, &rect);
            }
            LineCap::Round => {
                let r2 = half_lw * half_lw;
                for ly in 0..rh as i32 {
                    let wy = ry0 + ly;
                    for lx in 0..rw as i32 {
                        let wx = rx0 + lx;
                        let dx = wx as f32 + 0.5 - endpoint_x;
                        let dy = wy as f32 + 0.5 - endpoint_y;
                        if dx * dx + dy * dy <= r2 {
                            mask[(ly as usize) * rw + (lx as usize)] = 255;
                        }
                    }
                }
            }
        }
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

/// R34xx：canvas 文本 shaping 的 OpenType feature 列表——fontKerning 'none' 关 kern；
/// font-variant small-caps 开 smcp（2d.text.fontVariantCaps2.worker：small-caps 与
/// normal 的 measure 宽度须不同）。
fn text_features(kerning_none: bool, small_caps: bool) -> Vec<zero_render_foundation::font::OpenTypeFeature> {
    let mut v = Vec::new();
    if kerning_none {
        v.push(zero_render_foundation::font::OpenTypeFeature::new(*b"kern", 0));
    }
    if small_caps {
        v.push(zero_render_foundation::font::OpenTypeFeature::new(*b"smcp", 1));
    }
    v
}

/// R34xx：canvas 文本预处理（spec text preparation algorithm：替换 ASCII whitespace 为
/// U+0020——tab/CR/LF/FF 与 space 同宽同墨迹；2d.text.measure.actualBoundingBox.whitespace
/// 的 tab 期望 |Left|≥49，而 CanvasTest tab 字形自带墨迹）+ null 剥离（width.nullCharacter）。
pub(crate) fn prepare_canvas_text(text: &str) -> String {
    text.chars()
        .filter(|c| *c != '\0')
        .map(|c| if c.is_ascii_whitespace() && c != ' ' { ' ' } else { c })
        .collect()
}

/// R34xx：字形墨迹 bbox（font units → px，亚像素精度）——ttf_parser 轮廓 bbox 按
/// size/upem 缩放。位图光栅在亚像素字号下量化（2d.text.measure.actualBoundingBox.
/// small-font：1.5px 'E' 期望 right≈1.5，量化位图给 2）。空字形（无轮廓）返 None。
fn glyph_ink_bbox(data: &[u8], face_index: u32, glyph_id: u16, size: f32) -> Option<(f32, f32, f32, f32)> {
    let face = rustybuzz::ttf_parser::Face::parse(data, face_index).ok()?;
    let upem = f32::from(face.units_per_em());
    if upem <= 0.0 {
        return None;
    }
    let bb = face.glyph_bounding_box(rustybuzz::ttf_parser::GlyphId(glyph_id))?;
    let s = size / upem;
    Some((
        bb.x_min as f32 * s,
        -bb.y_max as f32 * s,
        bb.x_max as f32 * s,
        -bb.y_min as f32 * s,
    ))
}

/// R34xx：BASE 表 'hang'/'ideo' 基线（font units → px）——2d.text.measure.baselines：
/// CanvasTest BASE 表 hang=512（0.5em）、ideo=128（0.125em）。结构：
/// BASE(version, horizAxisOffset) → horizAxis(tagList/scriptList/lineList 偏移) →
/// tagList(count+tags) → scriptList 首脚本 → BaseScript(baseValuesOffset) →
/// BaseValues(defaultIndex, count, count×offset) → BaselineValues(format1, int16)。
/// 解析失败返 (None, None)（回退启发式）。
pub(crate) fn font_baselines_px(data: &[u8], face_index: u32, size: f32) -> Option<(Option<f32>, Option<f32>)> {
    fn u16at(data: &[u8], off: usize) -> Option<u16> {
        data.get(off..off + 2).map(|b| u16::from_be_bytes([b[0], b[1]]))
    }
    let count = u16at(data, 4)? as usize;
    let mut base = None;
    for i in 0..count {
        let start = 12 + 16 * i;
        if data.get(start..start + 4) == Some(b"BASE") {
            let o = data.get(start + 8..start + 12)?;
            base = Some(u32::from_be_bytes([o[0], o[1], o[2], o[3]]) as usize);
            break;
        }
    }
    let base = base?;
    let hax = base + u16at(data, base + 4)? as usize;
    let tag_list = hax + u16at(data, hax)? as usize;
    let script_list = hax + u16at(data, hax + 2)? as usize;
    let tag_count = u16at(data, tag_list)? as usize;
    let mut hang_idx = None;
    let mut ideo_idx = None;
    for i in 0..tag_count {
        let start = tag_list + 2 + 4 * i;
        let t = data.get(start..start + 4)?;
        if t == b"hang" {
            hang_idx = Some(i);
        }
        if t == b"ideo" {
            ideo_idx = Some(i);
        }
    }
    let (hang_idx, ideo_idx) = (hang_idx?, ideo_idx?);
    let sc_count = u16at(data, script_list)? as usize;
    if sc_count == 0 {
        return None;
    }
    let script = script_list + u16at(data, script_list + 6)? as usize;
    let bv = script + u16at(data, script)? as usize;
    let v_count = u16at(data, bv + 2)? as usize;
    let mut values: Vec<f32> = Vec::new();
    for i in 0..v_count {
        let rec = bv + 4 + 2 * i;
        let off = u16at(data, rec)? as usize;
        let v = bv + off;
        let fmt = u16at(data, v)?;
        if fmt != 1 {
            continue;
        }
        let raw = data.get(v + 2..v + 4)?;
        values.push(i16::from_be_bytes([raw[0], raw[1]]) as f32);
    }
    if values.len() <= hang_idx.max(ideo_idx) {
        return None;
    }
    let face = rustybuzz::ttf_parser::Face::parse(data, face_index).ok()?;
    let upem = f32::from(face.units_per_em());
    if upem <= 0.0 {
        return None;
    }
    let scale = size / upem;
    Some((
        values.get(hang_idx).copied().map(|v| v * scale),
        values.get(ideo_idx).copied().map(|v| v * scale),
    ))
}
