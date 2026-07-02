//! # zero-ui-adapter-render-foundation
//!
//! render-foundation 光栅后端适配器（spec TBD-2 闭环）。
//!
//! [`RenderFoundationBackend`] 实现 [`zero_ui_render::RenderBackend`]，把通用 UI SDK 产出的
//! 扁平 [`Scene`](zero_ui_render::Scene) 累积为 render-foundation 的
//! [`RenderPrimitives`](zero_render_foundation::primitive::RenderPrimitives)，供现有 GPU/CPU
//! 渲染管线消费。`ui/render` 因此**不直接依赖** render-foundation（spec TBD-2），二者经本适配器耦合。
//!
//! 调用：`paint_scene(&scene, &mut backend)` → `backend.into_primitives()`。
//!
//! ## 当前覆盖（DC-14 视觉迁移：几何 + 文本 + 外部表面，RenderBackend 全功能）
//! - `fill_rect`：圆角为零 → [`FillPrimitive`]；否则 → [`RoundedRectPrimitive`]（四角半径分别映射）。
//! - `stroke_rect`：矩形四角顶点 → [`PathStrokePrimitive`]（closed）。**圆角暂忽略**（render-foundation
//!   无 stroke 圆角矩形图元；TODO 跟踪）。
//! - `apply_clip`：`Some(rect)` → [`add_clip`]；`None` → 回落视口矩形（"无裁剪" = 整个视口）。
//!   render-foundation 经 `draw_order` 流式应用裁剪，与本适配器的流式 `apply_clip` 语义一致。
//! - `draw_text`（DC-11 文本，原始字符串——SDK widgets 实际走此路径）：经共享 `FontdueBackend`
//!   shape（默认 FontRequest 回落首个已加载字体）+ `rasterize_glyph` → tinted RGBA → [`ImageCache`] →
//!   [`ImagePrimitive`]（fontdue 定位：xmin 左、ymin 底）。
//! - `draw_text_blob`（DC-11 文本，预 shape）：同上但跳过 shape（TextBlob 已 shape）。
//!   **契约**：TextBlob 生产者与本后端共享同一 `Arc<FontdueBackend>`（`new_with_text`），FontId 一致。
//! - `draw_external_surface`（DC-3 phase-2）：`set_surface` 注册调用方**预变换**的表面（WebView）场景；
//!   `draw_external_surface(rect, id)` 以 `rect` 为裁剪边界，把注册场景合并进帧（`draw_order` 按桶
//!   偏移重映射，保留表面内部 z 序）。本后端不做空间变换——调用方负责 offset/scale（参考
//!   apps/browser `append_webview_primitives`）。
//!
//! ## 暂未覆盖（明确 follow-up，非阻塞当前闭环）
//! - 生产集成（DC-14 真实接线）：本后端自带 `ImageCache`；浏览器消费时须把 glyph key 解析到
//!   渲染器的 image cache（或共享）——当前无消费者，几何+文本+外部表面经测试验证。
//!
//! [`FillPrimitive`]: zero_render_foundation::primitive::FillPrimitive
//! [`RoundedRectPrimitive`]: zero_render_foundation::primitive::RoundedRectPrimitive
//! [`PathStrokePrimitive`]: zero_render_foundation::primitive::PathStrokePrimitive
//! [`add_clip`]: zero_render_foundation::primitive::RenderPrimitives::add_clip

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use zero_render_foundation::color::Color as RfColor;
use zero_render_foundation::geometry::{Point as RfPoint, Rect as RfRect, Size as RfSize};
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};
use zero_render_foundation::primitive::{DrawOp, ImagePrimitive, RenderPrimitives, RoundedRectPrimitive};
use zero_text_foundation::{
    FontId, FontRequest, FontdueBackend, GlyphBitmap, GlyphRun, ShapeInput, TextBlob, TextDirection, TextShaper,
};
use zero_ui_core::geometry::{Point, Rect, Rounding};
use zero_ui_core::theme::Color;
use zero_ui_render::RenderBackend;

/// glyph 位图缓存容量（条目数 / 字节数）。
const GLYPH_CACHE_MAX_ENTRIES: usize = 4096;
const GLYPH_CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;

/// render-foundation 后端：把 `RenderBackend` 调用累积为 [`RenderPrimitives`]。
///
/// 构造时传入 `viewport`（用于 `apply_clip(None)` 的"无裁剪"回落）。绘制完成后用
/// [`into_primitives`](Self::into_primitives) 取出累积结果交给 render-foundation 渲染。
///
/// 文本路径（DC-11）：[`Self::new_with_text`] 注入共享 `Arc<FontdueBackend>`——TextBlob 的生产者
/// 与本后端共享同一字体栈（FontId 一致），`draw_text_blob` 经 `rasterize_glyph` 光栅每个 glyph →
/// tinted RGBA → [`ImageCache`] → [`ImagePrimitive`]（DC-11 共享字体栈不变量）。
pub struct RenderFoundationBackend {
    primitives: RenderPrimitives,
    viewport: RfRect,
    text: Arc<FontdueBackend>,
    image_cache: ImageCache,
    /// 已上传 glyph 的 key 集合（避免每帧重复 raster+upload；key 稳定）。
    uploaded: HashSet<ImageKey>,
    /// 外部表面（WebView 等）的预变换场景注册表（DC-3 phase-2）。key = surface_id。
    surfaces: HashMap<u64, RenderPrimitives>,
}

impl RenderFoundationBackend {
    /// 创建后端，`viewport` 为目标帧的全区域（用于 clip=None 回落）。
    ///
    /// 不带文本后端——`draw_text_blob` 会因无字体而 no-op。需要渲染文本用
    /// [`Self::new_with_text`]。
    pub fn new(viewport: RfRect) -> Self {
        Self::new_with_text(viewport, Arc::new(FontdueBackend::new()))
    }

    /// 创建后端并共享 `text`（TextBlob 生产者须用同一 `Arc<FontdueBackend>` 实例 shape，
    /// 保证 FontId 一致——DC-11 字体栈共享契约）。
    pub fn new_with_text(viewport: RfRect, text: Arc<FontdueBackend>) -> Self {
        RenderFoundationBackend {
            primitives: RenderPrimitives::default(),
            viewport,
            text,
            image_cache: ImageCache::new(GLYPH_CACHE_MAX_ENTRIES, GLYPH_CACHE_MAX_BYTES),
            uploaded: HashSet::new(),
            surfaces: HashMap::new(),
        }
    }

    /// 同 [`Self::new_with_text`]，但视口以 ui/core [`Size`](zero_ui_core::geometry::Size) 给出
    ///（原点 (0,0)），免去调用方构造 render-foundation Rect。便于从 [`WindowMetrics`] 等逻辑尺寸直接构造。
    ///
    /// [`WindowMetrics`]: zero_ui_core::layout::WindowMetrics
    pub fn new_with_text_size(viewport_size: zero_ui_core::geometry::Size, text: Arc<FontdueBackend>) -> Self {
        Self::new_with_text(
            RfRect {
                origin: RfPoint { x: 0.0, y: 0.0 },
                size: RfSize {
                    width: viewport_size.width,
                    height: viewport_size.height,
                },
            },
            text,
        )
    }

    /// 取出累积的 [`RenderPrimitives`]（消费后端）。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 只读访问累积结果（测试/调试用）。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }

    /// 只读访问 glyph 位图缓存（draw_text_blob 产出的 ImageKey 在此解析）。
    pub fn image_cache(&self) -> &ImageCache {
        &self.image_cache
    }

    /// 取出 glyph 位图缓存（消费后端；与 [`Self::into_primitives`] 配套交给渲染器）。
    pub fn into_image_cache(self) -> ImageCache {
        self.image_cache
    }

    /// 注册外部表面（WebView 等）的本帧预变换场景（DC-3 phase-2）。
    ///
    /// **契约**：`primitives` 须已由调用方变换到帧坐标空间（offset/scale/clip，参考
    /// apps/browser 的 `append_webview_primitives`）。`draw_external_surface(rect, id)` 据此
    /// `id` 取回并以 `rect` 为裁剪边界合并进累积场景。
    pub fn set_surface(&mut self, surface_id: u64, primitives: RenderPrimitives) {
        self.surfaces.insert(surface_id, primitives);
    }

    /// 清空外部表面注册表（每帧绘制前调用，避免上一帧残留）。
    pub fn clear_surfaces(&mut self) {
        self.surfaces.clear();
    }
}

impl RenderBackend for RenderFoundationBackend {
    fn fill_rect(&mut self, rect: Rect, color: Color, rounding: Rounding) {
        let rf_rect = to_rf_rect(rect);
        let rf_color = to_rf_color(color);
        if rounding.top_left == 0.0
            && rounding.top_right == 0.0
            && rounding.bottom_right == 0.0
            && rounding.bottom_left == 0.0
        {
            self.primitives.add_fill(rf_rect, rf_color);
        } else {
            self.primitives.add_rounded_rect(RoundedRectPrimitive {
                rect: rf_rect,
                color: rf_color,
                top_left_radius: rounding.top_left,
                top_right_radius: rounding.top_right,
                bottom_right_radius: rounding.bottom_right,
                bottom_left_radius: rounding.bottom_left,
            });
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32, _rounding: Rounding) {
        // render-foundation 无 stroke 圆角矩形图元；用 4 角顶点的闭合路径描边。
        // 圆角 `_rounding` 暂忽略（TODO：圆角描边需路径曲线，跟踪）。
        let (x1, y1) = (rect.origin.x, rect.origin.y);
        let (x2, y2) = (rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);
        // 闭合矩形：左上 → 右上 → 右下 → 左下（renderer closed=true 连回左上）。
        let vertices = vec![x1, y1, x2, y1, x2, y2, x1, y2];
        self.primitives
            .add_path_stroke(vertices, to_rf_color(color), stroke_width, true);
    }

    fn draw_text_blob(&mut self, blob: &TextBlob, position: Point, color: Color) {
        // DC-11 文本路径（预 shape）：TextBlob 的 FontId 在 self.text 的字体空间
        //（调用方契约：shape 与 raster 共享同一 FontdueBackend 实例）。
        if self.text.is_empty() {
            return; // 无字体 → 无法光栅
        }
        raster_runs(self, &blob.shaped.runs, position, to_rf_color(color));
    }

    fn draw_text(&mut self, text: &str, position: Point, size_px: f32, color: Color) {
        // DC-11 文本路径（原始字符串）：SDK widgets（Button/Label/chrome 等）经 paint_ctx 走本方法。
        // 用共享 FontdueBackend shape（默认 FontRequest，回落首个已加载字体）→ 光栅。
        if self.text.is_empty() || text.is_empty() {
            return;
        }
        let shaped = match self.text.shape(&ShapeInput {
            text: text.into(),
            // 无调用方 FontRequest → 用通用族，best_match 回落到首个已加载字体。
            font_request: FontRequest::new("sans-serif"),
            size_px,
            direction: TextDirection::Ltr,
            script: None,
            scale_factor: 1.0,
        }) {
            Ok(s) => s,
            Err(_) => return, // shape 失败（如无字体）→ 安静跳过
        };
        raster_runs(self, &shaped.runs, position, to_rf_color(color));
    }

    fn draw_external_surface(&mut self, rect: Rect, surface_id: u64) {
        // DC-3 phase-2：以 rect 为裁剪边界，把已注册（调用方预变换）的表面场景合并进帧。
        // 渲染顺序：先 clip(rect) 约束，再合并表面图元（draw_order 重映射保持表面内部 z 序）。
        self.primitives.add_clip(to_rf_rect(rect));
        if let Some(src) = self.surfaces.get(&surface_id) {
            merge_primitives(&mut self.primitives, src);
        }
        // 未注册的 surface_id → 仅留 clip（占位，不阻断；调用方应先 set_surface）。
    }

    fn apply_clip(&mut self, clip: Option<Rect>) {
        // Some → 裁剪到该矩形；None → 回落视口（"无裁剪" = 整个视口）。
        // render-foundation 经 draw_order 流式应用裁剪，与本适配器语义一致。
        let rf_rect = match clip {
            Some(r) => to_rf_rect(r),
            None => self.viewport,
        };
        self.primitives.add_clip(rf_rect);
    }
}

// ── 外部表面合并（DC-3 phase-2 draw_external_surface）────────────────

/// 把 `src` 的全部图元合并进 `dst`（扩展各分桶 + 按桶偏移重映射 `draw_order`）。
///
/// **无空间变换**：调用方须预先把 `src` 的图元变换到 `dst` 的坐标空间（offset/scale/clip）。
/// 本函数只做结构合并，保留 `src` 内部的绘制顺序（draw_order 相对位置不变）。
fn merge_primitives(dst: &mut RenderPrimitives, src: &RenderPrimitives) {
    // 记录各桶当前长度（合并后 src 索引的偏移基准）。
    let o_clip = dst.clips.len();
    let o_fill = dst.fills.len();
    let o_rr = dst.rounded_rects.len();
    let o_pf = dst.path_fills.len();
    let o_ps = dst.path_strokes.len();
    let o_st = dst.strokes.len();
    let o_gr = dst.gradients.len();
    let o_sh = dst.shadows.len();
    let o_im = dst.images.len();
    let o_gl = dst.glyphs.len();
    let o_fi = dst.filters.len();
    let o_bm = dst.blend_modes.len();
    let o_tr = dst.transforms.len();

    dst.clips.extend(src.clips.iter().cloned());
    dst.fills.extend(src.fills.iter().cloned());
    dst.rounded_rects.extend(src.rounded_rects.iter().cloned());
    dst.path_fills.extend(src.path_fills.iter().cloned());
    dst.path_strokes.extend(src.path_strokes.iter().cloned());
    dst.strokes.extend(src.strokes.iter().cloned());
    dst.gradients.extend(src.gradients.iter().cloned());
    dst.shadows.extend(src.shadows.iter().cloned());
    dst.images.extend(src.images.iter().cloned());
    dst.glyphs.extend(src.glyphs.iter().cloned());
    dst.filters.extend(src.filters.iter().cloned());
    dst.blend_modes.extend(src.blend_modes.iter().cloned());
    dst.transforms.extend(src.transforms.iter().cloned());

    for op in &src.draw_order {
        dst.draw_order.push(match *op {
            DrawOp::Fill(i) => DrawOp::Fill(i + o_fill),
            DrawOp::RoundedRect(i) => DrawOp::RoundedRect(i + o_rr),
            DrawOp::PathFill(i) => DrawOp::PathFill(i + o_pf),
            DrawOp::PathStroke(i) => DrawOp::PathStroke(i + o_ps),
            DrawOp::Stroke(i) => DrawOp::Stroke(i + o_st),
            DrawOp::Gradient(i) => DrawOp::Gradient(i + o_gr),
            DrawOp::Shadow(i) => DrawOp::Shadow(i + o_sh),
            DrawOp::Image(i) => DrawOp::Image(i + o_im),
            DrawOp::Glyph(i) => DrawOp::Glyph(i + o_gl),
            DrawOp::Filter(i) => DrawOp::Filter(i + o_fi),
            DrawOp::BlendMode(i) => DrawOp::BlendMode(i + o_bm),
            DrawOp::Transform(i) => DrawOp::Transform(i + o_tr),
            DrawOp::Clip(i) => DrawOp::Clip(i + o_clip),
        });
    }
}

// ── 文本光栅化辅助（DC-11 draw_text / draw_text_blob）─────────────────

/// 把已 shape 的 glyph runs 光栅为 ImagePrimitive（draw_text / draw_text_blob 共用）。
///
/// pen 自 position.x 起逐 glyph 推进（x_advance）；空 bitmap（空格）/ 光栅失败（FontId 不匹配）
/// → 不出图，pen 仍推进（best-effort）。
fn raster_runs(backend: &mut RenderFoundationBackend, runs: &[GlyphRun], position: Point, tint: RfColor) {
    for run in runs {
        let font_id = run.font.id;
        let size_px = run.font_size_px;
        let mut pen_x = 0.0_f32;
        for g in &run.glyphs {
            let draw_x = pen_x + g.x_offset;
            if let Ok(bmp) = backend.text.rasterize_glyph(font_id, g.glyph_id, size_px)
                && bmp.width > 0
                && bmp.height > 0
            {
                let key = glyph_cache_key(font_id, g.glyph_id, size_px);
                emit_glyph_image(backend, key, &bmp, position, draw_x, tint);
            }
            pen_x += g.x_advance;
        }
    }
}

/// 把单个 glyph 位图作为 [`ImagePrimitive`] 累积（含缓存去重与屏幕定位）。
fn emit_glyph_image(
    backend: &mut RenderFoundationBackend,
    key: ImageKey,
    bmp: &GlyphBitmap,
    position: Point,
    draw_x: f32,
    tint: RfColor,
) {
    // 缓存去重：同一 (font,glyph,size) 只 raster+upload 一次（key 稳定）。
    if backend.uploaded.insert(key.clone()) {
        let rgba = tinted_rgba(bmp, tint);
        match ImageData::from_rgba(rgba, bmp.width as u32, bmp.height as u32) {
            Ok(data) => backend.image_cache.insert_with_key(key.clone(), data),
            Err(_) => {
                backend.uploaded.remove(&key); // 插入失败 → 允许后续重试
            }
        }
    }
    // 屏幕定位：fontdue xmin = 左边缘偏移；ymin = 底边偏移（y 向上）→ top = baseline − ymin − height。
    let left = position.x + draw_x + bmp.xmin as f32;
    let top = position.y - bmp.ymin as f32 - bmp.height as f32;
    backend.primitives.add_image(ImagePrimitive {
        rect: RfRect {
            origin: RfPoint { x: left, y: top },
            size: RfSize {
                width: bmp.width as f32,
                height: bmp.height as f32,
            },
        },
        image_key: key,
        clip: None,
    });
}

/// glyph → 稳定 ImageKey（font_id 高 16 位 / glyph_id 中 32 位 / size 0.25px 桶 低 16 位）。
fn glyph_cache_key(font_id: FontId, glyph_id: u32, size_px: f32) -> ImageKey {
    let size_q2 = (((size_px * 4.0).round().max(0.0)) as u64) & 0xFFFF;
    ImageKey(((font_id.0 as u64) << 48) | (((glyph_id as u64) & 0xFFFF_FFFF) << 16) | size_q2)
}

/// glyph alpha 覆盖 → tint 着色的 RGBA（RGB = 文本色，A = 覆盖）。
fn tinted_rgba(bmp: &GlyphBitmap, tint: RfColor) -> Vec<u8> {
    let mut out = Vec::with_capacity(bmp.coverage.len() * 4);
    for &a in &bmp.coverage {
        out.push(tint.r);
        out.push(tint.g);
        out.push(tint.b);
        out.push(a);
    }
    out
}

// ── 类型转换（ui/core → render-foundation）─────────────────────────────

/// ui/core Color（f32 0.0..=1.0）→ render-foundation Color（u8 0..=255）。
fn to_rf_color(c: Color) -> RfColor {
    RfColor {
        r: ch(c.r),
        g: ch(c.g),
        b: ch(c.b),
        a: ch(c.a),
    }
}

fn ch(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// ui/core Rect → render-foundation Rect。
fn to_rf_rect(r: Rect) -> RfRect {
    RfRect {
        origin: RfPoint {
            x: r.origin.x,
            y: r.origin.y,
        },
        size: RfSize {
            width: r.size.width,
            height: r.size.height,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::{Point, Rect, Rounding};
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::scene::{Scene, SceneEntry};
    use zero_ui_render::{RenderPrimitive, paint_scene};

    fn viewport() -> RfRect {
        RfRect {
            origin: RfPoint { x: 0.0, y: 0.0 },
            size: RfSize {
                width: 800.0,
                height: 600.0,
            },
        }
    }

    fn entry(clip: Option<Rect>, prim: RenderPrimitive) -> SceneEntry {
        SceneEntry {
            source: WidgetId::new("t"),
            clip,
            primitive: prim,
        }
    }

    #[test]
    fn fill_rect_plain_uses_fill_no_rounding() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.fill_rect(Rect::from_ltrb(0.0, 0.0, 10.0, 20.0), Color::WHITE, Rounding::ZERO);
        let p = b.into_primitives();
        assert_eq!(p.fills.len(), 1);
        assert!(p.rounded_rects.is_empty());
        // f32 0-1 → u8：WHITE(1,1,1,1) → (255,255,255,255)。
        assert_eq!(
            p.fills[0].color,
            RfColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(p.fills[0].rect.origin.x, 0.0);
        assert_eq!(p.fills[0].rect.size.width, 10.0);
        assert_eq!(p.fills[0].rect.size.height, 20.0);
    }

    #[test]
    fn fill_rect_rounded_maps_radii() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.fill_rect(
            Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
            Color::BLACK,
            Rounding {
                top_left: 2.0,
                top_right: 4.0,
                bottom_right: 6.0,
                bottom_left: 8.0,
            },
        );
        let p = b.into_primitives();
        assert!(p.fills.is_empty());
        assert_eq!(p.rounded_rects.len(), 1);
        let r = &p.rounded_rects[0];
        assert_eq!(r.top_left_radius, 2.0);
        assert_eq!(r.top_right_radius, 4.0);
        assert_eq!(r.bottom_right_radius, 6.0);
        assert_eq!(r.bottom_left_radius, 8.0);
    }

    #[test]
    fn stroke_rect_emits_closed_path_of_four_corners() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.stroke_rect(Rect::from_ltrb(1.0, 2.0, 11.0, 22.0), Color::WHITE, 1.5, Rounding::ZERO);
        let p = b.into_primitives();
        assert_eq!(p.path_strokes.len(), 1);
        let s = &p.path_strokes[0];
        assert!(s.closed);
        assert_eq!(s.line_width, 1.5);
        // 4 角顶点（8 个 f32）。
        assert_eq!(s.vertices, vec![1.0, 2.0, 11.0, 2.0, 11.0, 22.0, 1.0, 22.0]);
    }

    #[test]
    fn apply_clip_some_and_none() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.apply_clip(Some(Rect::from_ltrb(5.0, 5.0, 50.0, 50.0)));
        b.apply_clip(None); // 回落视口
        let p = b.primitives();
        assert_eq!(p.clips.len(), 2);
        // 第一条 = 显式裁剪；第二条 = 视口（无裁剪回落）。
        assert_eq!(p.clips[0].rect.origin.x, 5.0);
        assert_eq!(p.clips[0].rect.size.width, 45.0);
        assert_eq!(p.clips[1].rect.size.width, 800.0);
        assert_eq!(p.clips[1].rect.size.height, 600.0);
    }

    #[test]
    fn paint_scene_dispatches_geometry_into_buckets() {
        // Scene：plain fill（无裁剪）→ rounded fill（裁剪 c）→ stroke（无裁剪）。
        // paint_scene 对每条 entry 先 apply_clip 再绘制。
        let clip = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        let mut scene = Scene::new();
        scene.push(entry(
            None,
            RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
                color: Color::WHITE,
                rounding: Rounding::ZERO,
            },
        ));
        scene.push(entry(
            Some(clip),
            RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 5.0, 5.0),
                color: Color::BLACK,
                rounding: Rounding::all(3.0),
            },
        ));
        scene.push(entry(
            None,
            RenderPrimitive::StrokeRect {
                rect: Rect::from_ltrb(0.0, 0.0, 8.0, 8.0),
                color: Color::WHITE,
                stroke_width: 1.0,
                rounding: Rounding::ZERO,
            },
        ));

        let mut b = RenderFoundationBackend::new(viewport());
        paint_scene(&scene, &mut b);
        let p = b.into_primitives();

        assert_eq!(p.fills.len(), 1); // plain fill
        assert_eq!(p.rounded_rects.len(), 1); // rounded fill
        assert_eq!(p.rounded_rects[0].top_left_radius, 3.0);
        assert_eq!(p.path_strokes.len(), 1); // stroke
        // 裁剪：每条 entry 一个 apply_clip → clips.len() == 3（None→viewport, Some→clip, None→viewport）。
        assert_eq!(p.clips.len(), 3);
        assert_eq!(p.clips[1].rect.size.width, 100.0); // 第二条 = 显式 clip
        // draw_order 按插入顺序记录 clip/fill/rounded_rect/path_stroke 交错。
        assert!(p.draw_order.len() >= 6);
    }

    #[test]
    fn draw_text_noop_without_fonts() {
        // 无字体后端 → draw_text no-op（不出图元、不 panic）。
        let mut b = RenderFoundationBackend::new(viewport());
        b.draw_text("hi", Point::ZERO, 12.0, Color::WHITE);
        let p = b.into_primitives();
        assert!(p.glyphs.is_empty());
        assert!(p.images.is_empty());
        assert!(p.fills.is_empty());
    }

    #[test]
    fn draw_text_emits_glyph_images() {
        // draw_text（原始字符串——SDK widgets 实际路径）经共享 backend shape+raster → ImagePrimitive。
        let backend = ahem_backend();
        let mut b = RenderFoundationBackend::new_with_text(viewport(), backend);
        b.draw_text("Hi", Point::new(5.0, 30.0), 16.0, Color::WHITE);
        let p = b.into_primitives();
        // "Hi" → 2 glyph → 2 ImagePrimitive，自左向右排列。
        assert_eq!(p.images.len(), 2);
        assert!(p.images[1].rect.origin.x > p.images[0].rect.origin.x);
    }

    // ── draw_text_blob（DC-11 文本路径）──────────────────────────────────

    /// WPT 标准字体（每字符 1em 实心方块）。路径相对 crate 根。
    const AHEM: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

    fn ahem_backend() -> Arc<FontdueBackend> {
        let mut b = FontdueBackend::new();
        b.load_family("Ahem", AHEM).expect("Ahem parses via fontdue");
        Arc::new(b)
    }

    #[test]
    fn draw_text_blob_emits_one_image_per_glyph() {
        use zero_text_foundation::{
            FontRequest, ShapeInput, TextBlob, TextDirection, TextMeasureInput, TextMeasurer, TextShaper,
        };
        let backend = ahem_backend();
        let mut b = RenderFoundationBackend::new_with_text(viewport(), backend.clone());
        // 经共享 backend shape "Hi"（FontId 在 backend 字体空间）→ TextBlob。
        let shaped = backend
            .shape(&ShapeInput {
                text: "Hi".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                direction: TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap();
        let metrics = backend
            .measure(&TextMeasureInput {
                text: "Hi".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                max_width: None,
                direction: TextDirection::Ltr,
            })
            .unwrap();
        let blob = TextBlob::new(shaped, metrics);

        let mut scene = Scene::new();
        scene.push(entry(
            None,
            RenderPrimitive::TextBlob {
                blob,
                position: Point::new(10.0, 50.0),
                color: Color::WHITE,
            },
        ));
        paint_scene(&scene, &mut b);
        let p = b.primitives();
        // "Hi" → 2 glyph → 2 ImagePrimitive。
        assert_eq!(p.images.len(), 2);
        // glyph 自左向右排列（第二个 x > 第一个）。
        assert!(p.images[1].rect.origin.x > p.images[0].rect.origin.x);
        // 两个 image_key 不同（不同 glyph）。
        assert_ne!(&p.images[0].image_key, &p.images[1].image_key);
    }

    #[test]
    fn draw_text_blob_caches_glyphs_across_paints() {
        use zero_text_foundation::{
            FontRequest, ShapeInput, TextBlob, TextDirection, TextMeasureInput, TextMeasurer, TextShaper,
        };
        let backend = ahem_backend();
        let shaped = backend
            .shape(&ShapeInput {
                text: "A".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                direction: TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap();
        let metrics = backend
            .measure(&TextMeasureInput {
                text: "A".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                max_width: None,
                direction: TextDirection::Ltr,
            })
            .unwrap();
        let make_scene = || {
            let blob = TextBlob::new(shaped.clone(), metrics);
            let mut s = Scene::new();
            s.push(SceneEntry {
                source: WidgetId::new("t"),
                clip: None,
                primitive: RenderPrimitive::TextBlob {
                    blob,
                    position: Point::new(0.0, 20.0),
                    color: Color::BLACK,
                },
            });
            s
        };
        let mut b = RenderFoundationBackend::new_with_text(viewport(), backend.clone());
        // 同一 glyph 画两次：uploaded 去重，image_cache 只插一次（第二次命中 uploaded 跳过 raster）。
        let key = {
            paint_scene(&make_scene(), &mut b);
            b.primitives().images[0].image_key.clone()
        };
        let key2 = {
            paint_scene(&make_scene(), &mut b);
            b.primitives().images[1].image_key.clone()
        };
        assert_eq!(key, key2, "同 glyph/size 应复用同一缓存 key");
    }

    #[test]
    fn draw_text_blob_noop_without_fonts() {
        // 无字体后端 → draw_text_blob no-op（不 panic、不出图）。
        let mut b = RenderFoundationBackend::new(viewport());
        // 构造空 blob（无 runs）直接喂入也应 no-op。
        use zero_text_foundation::{ShapedText, TextBlob, TextMetrics};
        let blob = TextBlob::new(
            ShapedText {
                runs: Vec::new(),
                total_advance_x: 0.0,
                total_advance_y: 0.0,
            },
            TextMetrics {
                width: 0.0,
                height: 0.0,
                ascent: 0.0,
                descent: 0.0,
                line_count: 1,
            },
        );
        let mut scene = Scene::new();
        scene.push(entry(
            None,
            RenderPrimitive::TextBlob {
                blob,
                position: Point::ZERO,
                color: Color::WHITE,
            },
        ));
        paint_scene(&scene, &mut b);
        assert!(b.primitives().images.is_empty());
    }

    #[test]
    fn glyph_cache_key_is_stable_and_distinct() {
        let k = glyph_cache_key(FontId(1), 65, 16.0);
        // 稳定。
        assert_eq!(k, glyph_cache_key(FontId(1), 65, 16.0));
        // 不同 font / glyph / size 不同。
        assert_ne!(k, glyph_cache_key(FontId(2), 65, 16.0));
        assert_ne!(k, glyph_cache_key(FontId(1), 66, 16.0));
        assert_ne!(k, glyph_cache_key(FontId(1), 65, 32.0));
    }

    // ── draw_external_surface（DC-3 phase-2 外部表面合并）──────────────

    fn rf_rect(x: f32, y: f32, w: f32, h: f32) -> RfRect {
        RfRect {
            origin: RfPoint { x, y },
            size: RfSize { width: w, height: h },
        }
    }

    #[test]
    fn draw_external_surface_merges_registered_with_remapped_draw_order() {
        use zero_render_foundation::primitive::{LineCap, LineStyle, StrokePrimitive};
        let mut b = RenderFoundationBackend::new(viewport());
        // 桥接先有一个 fill（使后续 surface fill 合并到 index 1，验证按桶偏移重映射）。
        b.fill_rect(Rect::from_ltrb(0.0, 0.0, 1.0, 1.0), Color::WHITE, Rounding::ZERO);

        // 注册表面场景：1 fill + 1 stroke（draw_order = [Fill(0), Stroke(0)]）。
        let mut surf = RenderPrimitives::default();
        surf.add_fill(
            rf_rect(10.0, 10.0, 5.0, 5.0),
            RfColor {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        );
        surf.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 9.0,
            y2: 9.0,
            width: 1.0,
            color: RfColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        assert_eq!(surf.draw_order, vec![DrawOp::Fill(0), DrawOp::Stroke(0)]);
        b.set_surface(7, surf);

        b.draw_external_surface(Rect::from_ltrb(10.0, 10.0, 100.0, 100.0), 7);
        let p = b.into_primitives();
        // 裁剪：draw_external_surface 加 1 个（rect）。
        assert_eq!(p.clips.len(), 1);
        // fills：桥接原 1 + surface 合并 1 = 2。
        assert_eq!(p.fills.len(), 2);
        // strokes：surface 合并 1。
        assert_eq!(p.strokes.len(), 1);
        // draw_order：桥接 Fill(0) → draw_external_surface Clip(0) → 合并 surface Fill(0+1)=Fill(1), Stroke(0+0)=Stroke(0)。
        assert_eq!(
            p.draw_order,
            vec![DrawOp::Fill(0), DrawOp::Clip(0), DrawOp::Fill(1), DrawOp::Stroke(0)]
        );
    }

    #[test]
    fn draw_external_surface_unknown_id_only_clips() {
        // 未注册 surface → 仅留 clip 占位，不合并、不 panic。
        let mut b = RenderFoundationBackend::new(viewport());
        b.draw_external_surface(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), 999);
        let p = b.into_primitives();
        assert_eq!(p.clips.len(), 1);
        assert!(p.fills.is_empty());
        assert!(p.strokes.is_empty());
        assert!(p.draw_order.iter().all(|op| matches!(op, DrawOp::Clip(_))));
    }

    // ── 全链集成（DC-14 管线去险）──────────────────────────────────────
    // BrowserChromeModel → DesktopBrowserShell::build → WidgetHost reconcile+layout+paint
    // → Scene → paint_scene → RenderFoundationBackend → render-foundation RenderPrimitives。
    // 证明浏览器接线前的完整 SDK chrome 渲染管线正确（不触 apps/browser，纯测试可验证）。

    #[test]
    fn full_pipeline_chrome_scene_to_render_primitives() {
        use zero_browser_chrome::{
            BrowserChromeModel, BrowserChromeShell, BrowserTab, DesktopBrowserShell, NavigationButtons, SecurityState,
            register_chrome_factories,
        };
        use zero_ui_core::geometry::{Constraints, Insets, Size};
        use zero_ui_core::layout::WindowMetrics;
        use zero_ui_core::theme::SemanticTokens;
        use zero_ui_runtime::WidgetHost;

        // 共享字体后端（加载 Ahem 供 chrome 文本 shape+raster）。
        let mut backend = FontdueBackend::new();
        backend.load_family("Ahem", AHEM).expect("Ahem parses");
        let backend = Arc::new(backend);

        // 有数据的 chrome 模型（address/security/tab）。
        let mut model = BrowserChromeModel::new();
        model.address_text = "https://example.com".into();
        model.security = SecurityState::Secure;
        model.navigation = NavigationButtons::new(true, false, false);
        model.tabs = vec![BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "Example".into(),
            loading: false,
        }];
        model.active_tab_index = Some(0);

        let metrics = WindowMetrics {
            logical_size: Size::new(1280.0, 800.0),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
        };
        let spec = DesktopBrowserShell.build(&model, &metrics);

        // WidgetHost：注册 chrome 工厂 → 装载声明树 → layout → paint → Scene。
        let mut host = WidgetHost::new();
        register_chrome_factories(&mut host, &SemanticTokens::light());
        host.set_root(&spec);
        host.layout(Constraints::loose(metrics.logical_size));
        let scene = host.paint().clone();
        assert!(!scene.entries.is_empty(), "chrome scene 应产出图元");

        // 桥接：Scene → render-foundation RenderPrimitives。
        let viewport_rect = RfRect {
            origin: RfPoint { x: 0.0, y: 0.0 },
            size: RfSize {
                width: 1280.0,
                height: 800.0,
            },
        };
        let mut bridge = RenderFoundationBackend::new_with_text(viewport_rect, backend);
        paint_scene(&scene, &mut bridge);
        let p = bridge.primitives();

        // chrome 背景几何 → FillPrimitive 非空。
        assert!(!p.fills.is_empty(), "桥接应产出 chrome 背景 fills");
        // chrome 文本（draw_text 经 Ahem shape+raster）→ glyph ImagePrimitive 非空。
        assert!(!p.images.is_empty(), "桥接应产出 widget 文本 ImagePrimitive");
        // draw_order 有序记录（clip/fill/image 交错）。
        assert!(!p.draw_order.is_empty());
    }
}
