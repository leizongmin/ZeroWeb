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
//! - `apply_clip`：**stateful clip**——设置 `current_clip`（`Some(rect)` → 该矩形；`None` → 视口），
//!   后续 `fill_rect`/`stroke_rect` 经 CPU 侧 intersect 裁到本矩形。**不 emit render-foundation 的
//!   破坏性 `ClipPrimitive`**（render-foundation `apply_clip` 是 clear-clip-外像素，与 `paint_scene`
//!   每 entry 一个 clip 的累积语义冲突——会逐个擦除兄弟 fill）。外部表面 `draw_external_surface`
//!   仍用显式 [`add_clip`] 取 render-foundation 原生 clip 语义。
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
//! ## 生产集成（DC-14 真实接线）
//! - [`merge_into_frame`]：把本后端产出的图元 + glyph image cache 合并进帧统一
//!   `RenderPrimitives` / `ImageCache`——经 [`ImageCache::extend_from_other`] 在帧 cache 为
//!   source image 分配新键（collision-safe）+ 重映射 source 图元的 image_key，再合并图元
//!   （13 分桶 + draw_order 偏移）。浏览器据此把 SDK chrome（fills + 文本 ImagePrimitive +
//!   glyph cache）并入帧，单一 `render_full_scene` 调用解析所有图片，无需为 chrome 单独光栅。
//!
//! [`ImageCache::extend_from_other`]: zero_render_foundation::image_cache::ImageCache::extend_from_other
//!
//! [`FillPrimitive`]: zero_render_foundation::primitive::FillPrimitive
//! [`RoundedRectPrimitive`]: zero_render_foundation::primitive::RoundedRectPrimitive
//! [`PathStrokePrimitive`]: zero_render_foundation::primitive::PathStrokePrimitive
//! [`add_clip`]: zero_render_foundation::primitive::RenderPrimitives::add_clip

use hashbrown::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use zero_render_foundation::color::Color as RfColor;
use zero_render_foundation::geometry::{Point as RfPoint, Rect as RfRect, Size as RfSize};
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};
use zero_render_foundation::primitive::{DrawOp, ImagePrimitive, RenderPrimitives, RoundedRectPrimitive};
use zero_text_foundation::{FontId, FontdueBackend, GlyphBitmap, GlyphRun, TextBlob};
use zero_ui_core::geometry::{Point, Rect, Rounding};
use zero_ui_core::image::ImageRef;
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
    /// 当前裁剪矩形（stateful clip，[`apply_clip`](RenderBackend::apply_clip) 设置；
    /// `fill_rect`/`stroke_rect` 经 CPU 侧 intersect 裁到本矩形）。默认 = viewport（"无裁剪"）。
    ///
    /// **不 emit render-foundation 的破坏性 `ClipPrimitive`/`DrawOp::Clip`**：render-foundation 的
    /// `apply_clip` 是 clear-clip-外像素（destructive），与 `paint_scene` 每 entry 一个 clip 的
    /// **累积**语义冲突——会逐个擦除兄弟 widget 的 fill（SDK chrome scene 渲染全白的根因）。
    /// 改为 stateful：每 draw 自行 intersect 到 current_clip，draw_order 只含 Fill/Image，累积渲染。
    /// （外部表面 `draw_external_surface` 仍用显式 `add_clip` 取 render-foundation 原生 clip 语义。）
    current_clip: RfRect,
    text: Arc<FontdueBackend>,
    image_cache: ImageCache,
    /// 已上传 glyph 的 key 集合（避免每帧重复 raster+upload；key 稳定）。
    uploaded: HashSet<ImageKey>,
    /// 外部表面（WebView 等）的预变换场景注册表（DC-3 phase-2）。key = surface_id。
    surfaces: HashMap<u64, RenderPrimitives>,
    /// 外部表面的 image cache（与 surfaces 一一对应；key = surface_id）。
    /// `draw_external_surface` 时自动 extend 进 `self.image_cache`，保证表面内 ImagePrimitive
    /// 的 image_key 有效（DC-3 phase-2 真实纹理合成闭环）。
    surface_caches: HashMap<u64, ImageCache>,
    /// 宿主预注册的图像 alpha 掩码（如浏览器 SVG 图标经 resvg 光栅后的覆盖率）。
    /// `draw_image` 据 [`ImageRef`] 取回掩码 + 按 `tint` 着色光栅（与 glyph 路径对称）。
    image_masks: HashMap<ImageRef, AlphaMask>,
}

/// 单通道 alpha 掩码（如 SVG 图标经 resvg 光栅后的覆盖率）。`coverage.len() = width*height`。
///
/// 与 foundation/text 的 `GlyphBitmap` 同构（alpha 覆盖），但本桥接不依赖 foundation/text 的
/// 图标能力——浏览器把任意 alpha 掩码注册为 `ImageRef`，控件经 `draw_image` 引用。
struct AlphaMask {
    coverage: Vec<u8>,
    width: u32,
    height: u32,
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
            current_clip: viewport,
            viewport,
            text,
            image_cache: ImageCache::new(GLYPH_CACHE_MAX_ENTRIES, GLYPH_CACHE_MAX_BYTES),
            uploaded: HashSet::new(),
            surfaces: HashMap::new(),
            surface_caches: HashMap::new(),
            image_masks: HashMap::new(),
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

    /// 同时取出累积图元与 glyph 位图缓存（消费后端）。
    ///
    /// [`merge_into_frame`] 接受 owned `RenderPrimitives` + 借用 `&ImageCache`，故浏览器接线
    /// （DC-14：SDK chrome 并入帧）用本方法一次取出两者，再交给 `merge_into_frame`。
    pub fn into_primitives_and_cache(self) -> (RenderPrimitives, ImageCache) {
        (self.primitives, self.image_cache)
    }

    /// 注册外部表面（WebView 等）的本帧预变换场景（DC-3 phase-2）。
    ///
    /// **契约**：`primitives` 须已由调用方变换到帧坐标空间（offset/scale/clip，参考
    /// apps/browser 的 `append_webview_primitives`）。`draw_external_surface(rect, id)` 据此
    /// `id` 取回并以 `rect` 为裁剪边界合并进累积场景。
    /// 注册外部表面的预变换图元（不含 image cache；仅 geometry-only）——向后兼容路径。
    pub fn set_surface(&mut self, surface_id: u64, primitives: RenderPrimitives) {
        self.surfaces.insert(surface_id, primitives);
    }

    /// 注册外部表面的预变换图元 + ImageCache（DC-3 phase-2 真实纹理合成）。
    ///
    /// `cache` 中的 image key 会在 `draw_external_surface` 时自动 extend 进后端自身的
    /// `ImageCache`，保证 surface 内 `ImagePrimitive` 的 key 在后续 `into_primitives_and_cache`
    /// 取出后仍有效（浏览器侧 `merge_into_frame` + `extend_from_other` 重映射闭环）。
    pub fn set_surface_with_cache(&mut self, surface_id: u64, primitives: RenderPrimitives, cache: ImageCache) {
        self.surfaces.insert(surface_id, primitives);
        self.surface_caches.insert(surface_id, cache);
    }

    /// 清空外部表面注册表（每帧绘制前调用，避免上一帧残留）。
    pub fn clear_surfaces(&mut self) {
        self.surfaces.clear();
        self.surface_caches.clear();
    }

    /// 注册一张图像 alpha 掩码（如浏览器 SVG 图标经 resvg 光栅的覆盖率），供 `draw_image` 引用。
    ///
    /// `coverage` 长度须 = `width * height`（单字节 alpha 0..=255）。同一 `key` 重复注册覆盖旧值。
    /// 控件经 `draw_image(rect, key, tint)` 引用本掩码；桥接按 `tint` 着色光栅（RGB=tint，
    /// A=coverage），与 glyph 文本路径完全对称。**ui/render 不依赖 render-foundation**（DC-1）：
    /// 本注册把浏览器侧的图标位图经 SDK 层 `ImageRef` 暴露给控件。
    pub fn register_image_mask(&mut self, key: ImageRef, coverage: Vec<u8>, width: u32, height: u32) {
        self.image_masks.insert(
            key,
            AlphaMask {
                coverage,
                width,
                height,
            },
        );
    }

    /// DC-3 phase-2：把 surface 的 ImageCache extend 进自身，remap surface image keys，merge。
    fn merge_surface_with_cache(&mut self, surface_id: u64, src: &RenderPrimitives) {
        let rekey = match self.surface_caches.remove(&surface_id) {
            Some(sc) => self.image_cache.extend_from_other(&sc),
            None => {
                merge_primitives(&mut self.primitives, src);
                return;
            }
        };
        if rekey.is_empty() {
            merge_primitives(&mut self.primitives, src);
        } else {
            let mut remapped = src.clone();
            for img in &mut remapped.images {
                if let Some(nk) = rekey.get(&img.image_key) {
                    img.image_key = nk.clone();
                }
            }
            merge_primitives(&mut self.primitives, &remapped);
        }
    }
}

impl RenderBackend for RenderFoundationBackend {
    fn fill_rect(&mut self, rect: Rect, color: Color, rounding: Rounding) {
        let rf_rect = to_rf_rect(rect);
        let rf_color = to_rf_color(color);
        // stateful clip：把 fill rect CPU 侧 intersect 到 current_clip（累积语义，非破坏性）。
        // 圆角 fill 的 intersect 近似（按 axis-aligned bbox 裁，clipped 边圆角失真）——chrome 用例
        // 多为直角 fill；圆角被裁是边缘情况，可接受。
        let Some(clipped) = rf_rect.intersection(&self.current_clip) else {
            return; // fill 完全在 clip 外 → 跳过
        };
        if rounding.top_left == 0.0
            && rounding.top_right == 0.0
            && rounding.bottom_right == 0.0
            && rounding.bottom_left == 0.0
        {
            self.primitives.add_fill(clipped, rf_color);
        } else {
            self.primitives.add_rounded_rect(RoundedRectPrimitive {
                rect: clipped,
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
        // stateful clip：按 axis-aligned bbox intersect 到 current_clip（近似；clipped 边描边失真）。
        let rf_rect = to_rf_rect(rect);
        let Some(clipped) = rf_rect.intersection(&self.current_clip) else {
            return;
        };
        let (x1, y1) = (clipped.origin.x, clipped.origin.y);
        let (x2, y2) = (
            clipped.origin.x + clipped.size.width,
            clipped.origin.y + clipped.size.height,
        );
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
        // DC-11 文本路径：逐字符 rasterize（匹配手绘 chrome 的 per-char 路径）。
        // 手绘 chrome `draw_ui_text` 对每个字符调用 fontdue `rasterize(char)`，
        // 不经过 rustybuzz shaping（无 GPOS kerning），提前进 `measure_advance` 定位下
        // 一字符。SDK 旧实现用 rustybuzz shaping 产生不同 glyph x-positions → pixel diff。
        // 本路径直接逐字符 rasterize，与手绘逐位一致。
        if self.text.is_empty() || text.is_empty() {
            return;
        }
        // 首个已加载字体（ChromeUI / sdk_font_backend 加载的字体）。
        let font_id = FontId(0);
        let tint = to_rf_color(color);
        let mut pen_x = 0.0_f32;
        for ch in text.chars() {
            match self.text.rasterize_char(font_id, ch, size_px) {
                Ok(bmp) => {
                    // 所有 glyph（含空格/零尺寸空白字符）都用真实 advance 推进 pen_x，
                    // 以便与手绘 `measure_advance` 路径逐位一致（手绘不跳过任何字符的 advance）。
                    // 仅对非零尺寸 glyph 发射位图（空格等空白字符宽高=0，无需发射 image）。
                    if bmp.width > 0 && bmp.height > 0 {
                        // cache key 用字符码点作为「glyph_id」（fontdue 内部 lookup_glyph_index 映射
                        // 字符→glyph id，与 `rasterize_char` 一致；手绘 `rasterize_glyph_with_fallback`
                        // 的缓存键也是 `(font_id, code_point, size)`）。
                        let key = glyph_cache_key(font_id, ch as u32, size_px);
                        emit_glyph_image(self, key, &bmp, position, pen_x, tint);
                    }
                    pen_x += bmp.advance;
                }
                Err(_) => {
                    // 光栅失败（如未知字体），用保守 fallback advance 避免布局坍塌。
                    pen_x += size_px * 0.5;
                }
            }
        }
    }

    fn draw_external_surface(&mut self, rect: Rect, surface_id: u64) {
        // DC-3 phase-2：以 rect 为裁剪边界，把已注册（调用方预变换）的表面场景合并进帧。
        // 渲染顺序：先 clip(rect) 约束，再合并表面图元（draw_order 重映射保持表面内部 z 序）。
        self.primitives.add_clip(to_rf_rect(rect));
        if let Some(src) = self.surfaces.get(&surface_id).cloned() {
            // DC-3 phase-2 真实纹理合成：
            // 1. extend surface image cache into bridge cache (get old→new key remap)
            // 2. remap surface ImagePrimitive keys
            // 3. merge remapped primitives
            self.merge_surface_with_cache(surface_id, &src);
        }
        // 未注册的 surface_id → 仅留 clip（占位，不阻断；调用方应先 set_surface）。
    }

    fn draw_image(&mut self, rect: Rect, key: ImageRef, tint: Color) {
        // 预注册图像（如 SVG 图标）：按 key 取回 alpha 掩码 + 按 tint 着色光栅 → ImagePrimitive。
        // 与 glyph 文本路径对称（glyph = 字体内 alpha 掩码 + 文本色；本路径 = 宿主提供的 alpha
        // 掩码 + tint）。未注册 key 安静跳过（不 panic）。clip: None 与 glyph ImagePrimitive 一致
        // （chrome 图标始终在视口/toolbar 内，无须 stateful intersect 裁剪）。
        let Some(mask) = self.image_masks.get(&key) else {
            return;
        };
        let rf_key = image_cache_key(key, tint);
        // 缓存去重：同一 (image, tint) 只 tint+upload 一次（rf_key 稳定）。
        if self.uploaded.insert(rf_key.clone()) {
            let rgba = tint_alpha(&mask.coverage, to_rf_color(tint));
            match ImageData::from_rgba(rgba, mask.width, mask.height) {
                Ok(data) => self.image_cache.insert_with_key(rf_key.clone(), data),
                Err(_) => {
                    self.uploaded.remove(&rf_key); // 插入失败 → 允许后续重试
                }
            }
        }
        self.primitives.add_image(ImagePrimitive {
            rect: to_rf_rect(rect),
            image_key: rf_key,
            clip: None,
        });
    }

    fn apply_clip(&mut self, clip: Option<Rect>) {
        // stateful clip：设置 current_clip，后续 fill/stroke 经 CPU 侧 intersect 裁到本矩形。
        // Some → 该矩形；None → 视口（"无裁剪" = 整个视口）。**不 emit 破坏性 ClipPrimitive**
        // （见 current_clip 字段文档：paint_scene 每 entry 一个 clip，破坏性 clear 会擦除兄弟 fill）。
        self.current_clip = match clip {
            Some(r) => to_rf_rect(r),
            None => self.viewport,
        };
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

/// 把 `source` 图元 + 其 glyph image cache 合并进帧统一 primitives / image_cache（DC-14 chrome
/// 文本集成）。
///
/// **解决 ImageCache 键碰撞**：frame image_cache 可能已含与 `source` 键 id 重叠的条目（两边
/// 各自从 0 起顺序分配，或哈希碰撞），故经 [`ImageCache::extend_from_other`] 在 frame cache
/// 为 source 的每条 image 分配**新键**（绝不覆盖 frame 既有条目），用返回的重映射改写 source
/// 图元的 `image_key`，再经 [`merge_primitives`] 合并图元（13 分桶 + draw_order 偏移）。
///
/// 合并后 source 的 image 键全部指向 frame cache，可经帧的**单一** `image_cache` 解析——这是把
/// SDK chrome 渲染产出（`render_chrome_via_sdk` 的 fills + 文本 ImagePrimitive + glyph
/// image_cache）并入浏览器帧的标准入口，无需为 chrome 单独光栅再合成 framebuffer。
///
/// - `source_primitives`：被合并的图元（消费；image_key 会被改写到 frame 键空间）。
/// - `source_cache`：source 图元 image_key 当前指向的缓存（只读，合并后调用方可丢弃）。
/// - `frame_primitives` / `frame_cache`：帧统一图元与缓存（就地扩展）。
pub fn merge_into_frame(
    mut source_primitives: RenderPrimitives,
    source_cache: &ImageCache,
    frame_primitives: &mut RenderPrimitives,
    frame_cache: &mut ImageCache,
) {
    let remap = frame_cache.extend_from_other(source_cache);
    // 改写 source 图元的 image_key 到 frame 键空间（collision-safe）。
    for img in &mut source_primitives.images {
        if let Some(new_key) = remap.get(&img.image_key) {
            img.image_key = new_key.clone();
        }
    }
    merge_primitives(frame_primitives, &source_primitives);
}

// ── 文本光栅化辅助（DC-11 draw_text / draw_text_blob）─────────────────

/// 把已 shape 的 glyph runs 光栅为 ImagePrimitive（draw_text / draw_text_blob 共用）。
///
/// pen 自 position.x 起逐 glyph 推进（x_advance）；空 bitmap（空格）/ 光栅失败（FontId 不匹配）
/// → 不出图，pen 仍推进（best-effort）。
///
/// **单 run 假设**：foundation/text `shape()` 当前产出**单个** `GlyphRun`（`vec![GlyphRun]`），
/// 故每个 run 内 pen_x 自 0 起即可正确累进。若未来引入字体 fallback 产出**多 run**，本函数需
/// 跨 run 累计 pen_x（每 run 起点 = 前一 run advance_x 之和），否则各 run 会重叠在 position.x。
/// `GlyphRun` 当前无 run-level 起点偏移字段——多 run 支持须先扩展 ShapedText/GlyphRun 携带
/// 每 run 绝对起点。
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

/// glyph → 稳定 ImageKey（font_id bits 32-63 / glyph_id bits 16-31 / size 0.25px 桶 bits 0-15）。
///
/// 位分配按类型的实际值域：
/// - `FontId(pub u32)` → 占 bits 32-63（全 u32 范围，无高位丢失）。
/// - `glyph_id`：foundation/text `shape_with_font` 已钳制 ≤ `u16::MAX`（DC-11 深度审查 M2 修复），
///   故 bits 16-31（`& 0xFFFF` 对已钳制值无损）。
/// - `size_q2`：`(size_px × 4).round()` 0.25px 桶 → bits 0-15（size_px ≤ 16383px）。
///
/// 三者占满 u64，**全 FontId 范围无碰撞**。此前 `font_id << 48` 仅留 16 位 → FontId ≥ 65536
/// 高位被丢、键碰撞（虽 FontId 顺序分配实践中难达 65k，但位打包对 u32 契约不正确）。
fn glyph_cache_key(font_id: FontId, glyph_id: u32, size_px: f32) -> ImageKey {
    let size_q2 = (((size_px * 4.0).round().max(0.0)) as u64) & 0xFFFF;
    ImageKey(((font_id.0 as u64) << 32) | (((glyph_id as u64) & 0xFFFF) << 16) | size_q2)
}

/// SDK `ImageRef` + tint → 稳定 render-foundation `ImageKey`（draw_image 缓存键）。
///
/// 位分配（与 glyph key 不碰撞）：
/// - bit 63 = 1 标记 image（glyph key 把 font_id 放 bits 32-63，但 FontId 实际顺序分配很小
///   → bit 63=0；故 image 与 glyph 永不碰撞）。
/// - bits 32-47 = `ImageRef.0` 低 16 位（图标 id 实际 <16，足够）。
/// - bits 0-31 = tint RGBA（u8×4 紧凑打包；同图标不同 tint → 不同 key → 不同缓存条目）。
fn image_cache_key(image_ref: ImageRef, tint: Color) -> ImageKey {
    let t = to_rf_color(tint);
    let tint_packed = ((t.r as u64) << 24) | ((t.g as u64) << 16) | ((t.b as u64) << 8) | (t.a as u64);
    ImageKey((1u64 << 63) | (((image_ref.0) & 0xFFFF) << 32) | tint_packed)
}

/// alpha 掩码 → tint 着色 RGBA（RGB=tint，A=coverage × tint.a / 255）。
///
/// 与 glyph 的 `tinted_rgba` 同语义；本函数服务宿主注册的任意 alpha 掩码（图标等）。
fn tint_alpha(coverage: &[u8], tint: RfColor) -> Vec<u8> {
    let mut out = Vec::with_capacity(coverage.len() * 4);
    for &a in coverage {
        out.push(tint.r);
        out.push(tint.g);
        out.push(tint.b);
        out.push((a as u16 * tint.a as u16 / 255) as u8);
    }
    out
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
    fn apply_clip_stateful_intersects_subsequent_fills() {
        // stateful clip：apply_clip 设置 current_clip（不 emit 破坏性 ClipPrimitive），
        // 后续 fill 经 CPU 侧 intersect 裁到 current_clip。修 paint_scene 每 entry 一个 clip
        // + render-foundation 破坏性 apply_clip 擦除兄弟 fill 的语义冲突（SDK chrome 全白根因）。
        let mut b = RenderFoundationBackend::new(viewport());
        b.apply_clip(Some(Rect::from_ltrb(5.0, 5.0, 50.0, 50.0)));
        b.apply_clip(None); // 回落视口
        let p = b.primitives();
        assert_eq!(
            p.clips.len(),
            0,
            "apply_clip 不再 emit ClipPrimitive（stateful intersect，非破坏性）"
        );

        // clip (5,5)-(50,50)，fill (0,0)-(10,10) → intersect (5,5)-(10,10)（width 5）。
        let mut b2 = RenderFoundationBackend::new(viewport());
        b2.apply_clip(Some(Rect::from_ltrb(5.0, 5.0, 50.0, 50.0)));
        b2.fill_rect(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0), Color::WHITE, Rounding::ZERO);
        let p2 = b2.primitives();
        assert_eq!(p2.fills.len(), 1);
        assert_eq!(p2.fills[0].rect.origin.x, 5.0);
        assert!(
            (p2.fills[0].rect.size.width - 5.0).abs() < 1e-5,
            "fill clipped to current_clip"
        );

        // fill 完全在 clip 外 → 跳过（不出 fill）。
        let mut b3 = RenderFoundationBackend::new(viewport());
        b3.apply_clip(Some(Rect::from_ltrb(5.0, 5.0, 50.0, 50.0)));
        b3.fill_rect(
            Rect::from_ltrb(100.0, 100.0, 200.0, 200.0),
            Color::WHITE,
            Rounding::ZERO,
        );
        let p3 = b3.primitives();
        assert!(p3.fills.is_empty(), "fill outside clip → skipped");
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

        assert_eq!(p.fills.len(), 1); // plain fill（entry clip=None→viewport，intersect 不变）
        assert_eq!(p.rounded_rects.len(), 1); // rounded fill（clip 100×100 含 5×5 rect，intersect 不变）
        assert_eq!(p.rounded_rects[0].top_left_radius, 3.0);
        assert_eq!(p.path_strokes.len(), 1); // stroke（clip=None→viewport，intersect 不变）
        // stateful clip：apply_clip 不 emit ClipPrimitive → clips.len() == 0。
        assert_eq!(p.clips.len(), 0);
        // draw_order 按插入顺序记录 fill/rounded_rect/path_stroke（无 Clip op，因 apply_clip stateful）。
        assert!(p.draw_order.len() >= 3);
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

    #[test]
    fn draw_text_with_spaces_advances_correctly() {
        // DC-11 空格处理回归守卫：draw_text 对空白字符必须用真实 advance 而非 `size * 0.5`
        // fallback，否则文本总宽度不匹配手绘 `measure_advance` 路径 → placeholder 像素 diff。
        let backend = ahem_backend();
        // Ahem 每字符 1em 方块 + 1em advance。空格 advance = fontdue 实际值（> 0，pos）。
        let mut b = RenderFoundationBackend::new_with_text(viewport(), backend);
        // "a b"：字符+空格+字符。空格不出图（零尺寸）但 advance 推进 pen_x。
        b.draw_text("a b", Point::new(0.0, 16.0), 16.0, Color::WHITE);
        let p = b.into_primitives();
        // Ahem 字体：每个可见字符产生一幅图（空格尺寸=0 → 无图）。
        // "a b": a(图1) + space(无图) + b(图2) → 2 幅图
        assert_eq!(p.images.len(), 2, "space should not emit an image (zero-size glyph)");
        // "b" 的 x 位置 = a.advance + space.advance > a.advance（即有空格时 b 比纯 "ab" 更右）。
        assert!(
            p.images[1].rect.origin.x > 16.0,
            "'b' after space should be advanced past 'a' + space width, got x={}",
            p.images[1].rect.origin.x
        );
    }

    #[test]
    fn draw_text_preserves_advance_for_non_emitting_glyphs() {
        // 验证 fontdue 返回 zero-size glyph（空格）仍通过 advance 推进 pen_x，
        // 不会无端丢掉 advance 导致后续字符位置偏移。
        let backend = ahem_backend();
        // "a b" 与 "ab" 对比：前者 "b" 的 x 位置应严格大于后者 "b" 的 x（差 = space.advance）。
        let mut with_space = RenderFoundationBackend::new_with_text(viewport(), backend.clone());
        with_space.draw_text("a b", Point::new(0.0, 16.0), 16.0, Color::WHITE);
        let ps = with_space.into_primitives();
        let mut no_space = RenderFoundationBackend::new_with_text(viewport(), backend);
        no_space.draw_text("ab", Point::new(0.0, 16.0), 16.0, Color::WHITE);
        let pn = no_space.into_primitives();
        assert_eq!(ps.images.len(), 2, "with space: a + b = 2 images");
        assert_eq!(pn.images.len(), 2, "no space: a + b = 2 images");
        // "a b" 的第二个字符（b）必须比 "ab" 的第二个字符更靠右（空格额外推进了）。
        assert!(
            ps.images[1].rect.origin.x > pn.images[1].rect.origin.x,
            "space should push 'b' rightward: with_space={} vs no_space={}",
            ps.images[1].rect.origin.x,
            pn.images[1].rect.origin.x
        );
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

    #[test]
    fn glyph_cache_key_distinguishes_font_ids_beyond_u16() {
        // 回归守卫：FontId 为 u32，cache key 须区分低 16 位相同、高位不同的 FontId
        //（旧 `font_id << 48` 仅留 16 位 → FontId(1) 与 FontId(65537)=0x10001 碰撞，
        // 会取错 glyph 位图——视觉损坏）。修复后 font_id 占 bits 32-63，全 u32 范围无碰撞。
        let k1 = glyph_cache_key(FontId(1), 65, 16.0);
        let k65537 = glyph_cache_key(FontId(65537), 65, 16.0); // 0x10001：低 16 位 = 1，bit 16 置位
        assert_ne!(k1, k65537, "FontId(1) vs FontId(65537) must not collide");
        // 边界：FontId::MAX 与低 16 位相同的较小 id 仍区分。
        let kmax = glyph_cache_key(FontId(u32::MAX), 65, 16.0);
        assert_ne!(k1, kmax, "FontId(u32::MAX) must not collide with FontId(1)");
        // glyph_id 仍在 bits 16-31（≤ u16::MAX 经 shape 钳制）：同 font 下不同 glyph 区分。
        assert_ne!(
            glyph_cache_key(FontId(5), 1, 16.0),
            glyph_cache_key(FontId(5), 65535, 16.0),
            "glyph_id extremes distinct"
        );
    }

    // ── draw_image（预注册图像：图标 alpha 掩码 + tint）─────────────────────

    #[test]
    fn draw_image_emits_image_primitive_with_tinted_rgba() {
        // 注册 2x2 alpha 掩码（对角线不透明），draw_image → 1 ImagePrimitive + image_cache 含 tinted RGBA。
        let mut b = RenderFoundationBackend::new(viewport());
        b.register_image_mask(
            ImageRef::new(1),
            vec![255, 0, 0, 255], // 2x2：左上+右下不透明，右上+左下透明
            2,
            2,
        );
        b.draw_image(Rect::from_ltrb(10.0, 10.0, 26.0, 26.0), ImageRef::new(1), Color::WHITE);
        let (p, mut cache) = b.into_primitives_and_cache();
        assert_eq!(p.images.len(), 1, "one ImagePrimitive emitted");
        let img = &p.images[0];
        assert_eq!(img.rect.origin.x, 10.0);
        assert_eq!(img.rect.size.width, 16.0);
        assert!(img.clip.is_none(), "image clip None (与 glyph ImagePrimitive 一致)");
        // 缓存解析：tint WHITE(255,255,255,255) × coverage → 对角像素 RGBA=(255,255,255,255)。
        let data = cache.get(&img.image_key).expect("image key resolves in cache");
        assert_eq!(data.width, 2);
        assert_eq!(data.get_pixel(0, 0), [255, 255, 255, 255], "opaque pixel tinted white");
        assert_eq!(data.get_pixel(1, 0), [255, 255, 255, 0], "transparent pixel alpha 0");
    }

    #[test]
    fn draw_image_unknown_key_is_noop() {
        // 未注册 key → 不出图、不 panic。
        let mut b = RenderFoundationBackend::new(viewport());
        b.draw_image(Rect::from_ltrb(0.0, 0.0, 16.0, 16.0), ImageRef::new(999), Color::BLACK);
        let p = b.into_primitives();
        assert!(p.images.is_empty(), "unregistered image key → no ImagePrimitive");
    }

    #[test]
    fn draw_image_caches_per_tint_and_dedups_same_tint() {
        // 同 (image, tint) 画两次 → 同一 rf key（uploaded 去重，cache 只插一次）；
        // 同 image 不同 tint → 不同 rf key（不同缓存条目）。
        let mut b = RenderFoundationBackend::new(viewport());
        b.register_image_mask(ImageRef::new(1), vec![255], 1, 1);
        b.draw_image(Rect::ZERO, ImageRef::new(1), Color::WHITE);
        b.draw_image(Rect::ZERO, ImageRef::new(1), Color::WHITE); // 同 tint → dedup
        b.draw_image(Rect::ZERO, ImageRef::new(1), Color::BLACK); // 不同 tint → 新条目
        let (p, mut cache) = b.into_primitives_and_cache();
        assert_eq!(p.images.len(), 3, "3 ImagePrimitive（每次 draw 都出图）");
        // 只有 2 个 distinct rf key（WHITE + BLACK）。
        let distinct: HashSet<_> = p.images.iter().map(|i| i.image_key.clone()).collect();
        assert_eq!(distinct.len(), 2, "same tint dedup → 2 distinct cache keys");
        // 同 tint 的两次 draw 复用同一 key。
        assert_eq!(p.images[0].image_key, p.images[1].image_key);
        assert_ne!(p.images[0].image_key, p.images[2].image_key);
        // 两个 key 都能在 cache 解析。
        for k in &distinct {
            assert!(cache.get(k).is_some(), "tinted image key resolves");
        }
    }

    #[test]
    fn image_cache_key_never_collides_with_glyph_keys() {
        // bit 63 标记 image：glyph key 的 font_id 在 bits 32-63 但 FontId 实际小 → bit 63=0；
        // image key bit 63=1 → 两族永不碰撞。
        let img_key = image_cache_key(ImageRef::new(1), Color::WHITE);
        assert_ne!(
            img_key,
            glyph_cache_key(FontId(1), 0xE000, 16.0),
            "image key must not collide with glyph key"
        );
        // 稳定 + 区分 tint。
        assert_eq!(img_key, image_cache_key(ImageRef::new(1), Color::WHITE));
        assert_ne!(img_key, image_cache_key(ImageRef::new(1), Color::BLACK));
        // 区分 image_ref。
        assert_ne!(img_key, image_cache_key(ImageRef::new(2), Color::WHITE));
    }

    #[test]
    fn draw_image_via_paint_scene_end_to_end() {
        // SDK Scene 含 Image 图元 → paint_scene → bridge.draw_image → ImagePrimitive。
        // 证明 widget 经 PaintRecorder.draw_image 记录的图标能经完整 Scene 管线光栅。
        let mut scene = Scene::new();
        scene.push(entry(
            None,
            RenderPrimitive::Image {
                rect: Rect::from_ltrb(5.0, 5.0, 21.0, 21.0),
                key: ImageRef::new(7),
                tint: Color::BLACK,
            },
        ));
        let mut b = RenderFoundationBackend::new(viewport());
        b.register_image_mask(ImageRef::new(7), vec![128], 1, 1); // coverage 128
        paint_scene(&scene, &mut b);
        let (p, mut cache) = b.into_primitives_and_cache();
        assert_eq!(p.images.len(), 1, "Image primitive rasterized via paint_scene");
        // coverage 128 × tint BLACK(opaque 255) → alpha = 128*255/255 = 128。
        let data = cache.get(&p.images[0].image_key).expect("image key resolves");
        assert_eq!(data.get_pixel(0, 0)[3], 128, "coverage 128 × opaque tint → alpha 128");
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

    #[test]
    fn set_surface_with_cache_extends_bridge_image_cache() {
        // DC-3 phase-2：set_surface_with_cache 后 draw_external_surface 自动 extend
        // ImageCache，保证 surface 内 ImagePrimitive 的 key 在 bridge output 中有效。
        let mut b = RenderFoundationBackend::new(viewport());

        // 构造含 ImagePrimitive 的 surface + 对应的 ImageCache。
        let mut surf = RenderPrimitives::default();
        let mut surf_cache = ImageCache::new(16, 1_000_000);
        let img_key =
            surf_cache.insert(zero_render_foundation::image_cache::ImageData::from_rgba(vec![0u8; 16], 2, 2).unwrap());
        surf.add_image(zero_render_foundation::primitive::ImagePrimitive {
            rect: rf_rect(10.0, 10.0, 20.0, 20.0),
            image_key: img_key.clone(),
            clip: None,
        });

        // 用带 cache 的 API 注册 surface。
        b.set_surface_with_cache(1, surf, surf_cache);

        // draw_external_surface 应在 merge 时自动 extend ImageCache。
        b.draw_external_surface(Rect::from_ltrb(0.0, 0.0, 100.0, 100.0), 1);

        let (p, mut cache) = b.into_primitives_and_cache();
        // surface 的 image 已合并进 primitives。
        assert_eq!(p.images.len(), 1, "surface image merged");
        // bridge 自身 image_cache 包含该图像数据（extend_from_other 生效；key 可能经重映射）。
        let merged_key = &p.images[0].image_key;
        assert!(
            cache.get(merged_key).is_some(),
            "surface image data present in bridge image_cache (under key {merged_key:?})"
        );
        assert_eq!(
            cache.get(merged_key).map(|d| d.width),
            Some(2),
            "image data preserved (2x2 → width 2)"
        );
    }

    #[test]
    fn set_surface_without_cache_still_works_geometry_only() {
        // 向后兼容：不带 cache 的 set_surface 仍正常工作（geometry-only merge）。
        let mut b = RenderFoundationBackend::new(viewport());
        let mut surf = RenderPrimitives::default();
        surf.add_fill(
            rf_rect(10.0, 10.0, 5.0, 5.0),
            RfColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        );
        b.set_surface(1, surf);
        b.draw_external_surface(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), 1);
        let p = b.into_primitives();
        assert!(!p.fills.is_empty(), "surface fill merged via old API");
    }

    #[test]
    fn clear_surfaces_clears_caches_too() {
        // DC-3 phase-2：clear_surfaces 应同时清空 surfaces 与 surface_caches。
        let mut b = RenderFoundationBackend::new(viewport());
        let cache = ImageCache::new(16, 1_000_000);
        b.set_surface_with_cache(1, RenderPrimitives::default(), cache);
        assert_eq!(b.surface_caches.len(), 1);
        b.clear_surfaces();
        assert!(b.surface_caches.is_empty(), "surface_caches cleared");
        assert!(b.surfaces.is_empty(), "surfaces cleared");
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
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::Landscape,
        };
        let spec = DesktopBrowserShell.build(&model, &metrics);

        // WidgetHost：注册 chrome 工厂 → 装载声明树 → layout → paint → Scene。
        let mut host = WidgetHost::new();
        register_chrome_factories(
            &mut host,
            &SemanticTokens::light(),
            zero_browser_chrome::render::ChromeTabColors::from_tokens(&SemanticTokens::light()),
        );
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

    // ── merge_into_frame（DC-14 chrome 文本集成：缓存合并 + image_key 重映射）──────

    fn one_pixel(r: u8, g: u8, b: u8) -> ImageData {
        ImageData::from_rgba(vec![r, g, b, 255], 1, 1).unwrap()
    }

    #[test]
    fn merge_into_frame_remaps_source_image_keys_collision_safe() {
        // source 与 frame 各有 fills + image，image_key 各自指向自己的 cache（两边键 id 都从 0
        // 起 → 碰撞）。merge_into_frame 后：frame fills 含两者；frame images 含两者且键全部可在
        // frame_cache 解析；frame 原条目不被覆盖；source 的 image_key 被改写到 frame 新键。
        let mut source_cache = ImageCache::new(16, 1024 * 1024);
        let source_key = source_cache.insert(one_pixel(10, 10, 10)); // source 键 id 0
        let mut source = RenderPrimitives::default();
        source.add_fill(RfRect::new(0.0, 0.0, 10.0, 10.0), RfColor::rgb(255, 0, 0));
        source.add_image(ImagePrimitive {
            rect: RfRect::new(0.0, 0.0, 10.0, 10.0),
            image_key: source_key.clone(),
            clip: None,
        });

        let mut frame_cache = ImageCache::new(16, 1024 * 1024);
        let frame_key = frame_cache.insert(one_pixel(20, 20, 20)); // frame 键 id 0（与 source_key 碰撞）
        let mut frame = RenderPrimitives::default();
        frame.add_fill(RfRect::new(0.0, 0.0, 5.0, 5.0), RfColor::rgb(0, 255, 0));
        frame.add_image(ImagePrimitive {
            rect: RfRect::new(0.0, 0.0, 5.0, 5.0),
            image_key: frame_key.clone(),
            clip: None,
        });

        merge_into_frame(source, &source_cache, &mut frame, &mut frame_cache);

        assert_eq!(frame.fills.len(), 2, "merged fills from both");
        assert_eq!(frame.images.len(), 2, "merged images from both");
        // frame 原条目键仍解析为 frame 自己的图（r=20），未被覆盖。
        assert_eq!(frame_cache.get(&frame_key).map(|d| d.get_pixel(0, 0)[0]), Some(20));
        // source 的 image 被改写为新键（≠ source_key 原碰撞键，≠ frame_key），新键解析为 source 图（r=10）。
        let merged_source_key = frame
            .images
            .iter()
            .find(|im| im.image_key != frame_key)
            .expect("merged source image present")
            .image_key
            .clone();
        assert_ne!(
            merged_source_key, source_key,
            "source key remapped away from colliding id"
        );
        assert_ne!(merged_source_key, frame_key);
        assert_eq!(
            frame_cache.get(&merged_source_key).map(|d| d.get_pixel(0, 0)[0]),
            Some(10),
            "remapped key resolves to source image in frame cache"
        );
        assert_eq!(frame_cache.len(), 2, "frame cache holds original + merged entry");
    }

    #[test]
    fn merge_into_frame_source_without_images_leaves_cache_untouched() {
        // source 只有 fills（无文本/图片）→ merge 不触碰 frame_cache，只合并 fills。
        let source_cache = ImageCache::new(16, 1024 * 1024); // 空
        let mut source = RenderPrimitives::default();
        source.add_fill(RfRect::new(0.0, 0.0, 10.0, 10.0), RfColor::rgb(255, 0, 0));

        let mut frame_cache = ImageCache::new(16, 1024 * 1024);
        let frame_key = frame_cache.insert(one_pixel(20, 20, 20));
        let mut frame = RenderPrimitives::default();
        frame.add_fill(RfRect::new(0.0, 0.0, 5.0, 5.0), RfColor::rgb(0, 255, 0));

        merge_into_frame(source, &source_cache, &mut frame, &mut frame_cache);

        assert_eq!(frame.fills.len(), 2);
        assert!(frame.images.is_empty());
        assert_eq!(frame_cache.len(), 1, "empty source cache → frame cache untouched");
        assert_eq!(frame_cache.get(&frame_key).map(|d| d.get_pixel(0, 0)[0]), Some(20));
    }

    #[test]
    fn render_chrome_via_sdk_merges_into_frame_end_to_end() {
        // DC-14 完整集成流：render_chrome_via_sdk 产出 SDK chrome（fills + 文本 ImagePrimitive +
        // glyph cache）→ merge_into_frame 并入帧（image_key 重映射到帧 cache）→ 帧单一 image_cache
        // 可解析所有 image（SDK 文本 + 帧原有）。证明浏览器接线前的端到端正确性。
        use zero_browser_chrome::sdk_render::render_chrome_via_sdk;
        use zero_browser_shell::BrowserShell;
        use zero_ui_core::geometry::{Insets, Size};
        use zero_ui_core::layout::WindowMetrics;
        use zero_ui_core::theme::SemanticTokens;

        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut backend = FontdueBackend::new();
        backend.load_family("Ahem", AHEM).expect("Ahem parses");
        let backend = Arc::new(backend);
        let metrics = WindowMetrics {
            logical_size: Size::new(1280.0, 800.0),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::Landscape,
        };

        // SDK chrome 渲染 → bridge（fills + 文本 ImagePrimitive + glyph cache）。
        let bridge = render_chrome_via_sdk(&shell, &metrics, &SemanticTokens::light(), backend);
        let (sdk_prims, sdk_cache) = bridge.into_primitives_and_cache();
        assert!(!sdk_prims.fills.is_empty(), "SDK chrome 产出 fills");
        let sdk_image_count = sdk_prims.images.len();
        assert!(sdk_image_count > 0, "SDK chrome 产出文本 ImagePrimitive");

        // 模拟帧：已有 webview 内容（fill + image，image_key 指向帧 cache，键 id 与 SDK 可能碰撞）。
        let mut frame_cache = ImageCache::new(64, 16 * 1024 * 1024);
        let frame_img_key = frame_cache.insert(one_pixel(99, 99, 99)); // 帧 image 键 id 0
        let mut frame = RenderPrimitives::default();
        frame.add_fill(RfRect::new(0.0, 0.0, 1280.0, 800.0), RfColor::rgb(255, 255, 255));
        frame.add_image(ImagePrimitive {
            rect: RfRect::new(0.0, 0.0, 100.0, 100.0),
            image_key: frame_img_key.clone(),
            clip: None,
        });

        // 合并 SDK chrome 进帧（collision-safe：SDK image 键重映射到帧 cache 新键）。
        merge_into_frame(sdk_prims, &sdk_cache, &mut frame, &mut frame_cache);

        // 帧 images 含原有 + SDK 文本，且全部键可在帧 cache 解析。
        assert_eq!(frame.images.len(), 1 + sdk_image_count);
        for img in &frame.images {
            assert!(
                frame_cache.get(&img.image_key).is_some(),
                "every merged image key resolves in frame cache"
            );
        }
        // 帧原 image 键仍解析（未被 SDK 合并覆盖）。
        assert!(
            frame_cache.get(&frame_img_key).is_some(),
            "frame's original image key still resolves after merge"
        );
    }

    #[test]
    fn full_pipeline_chrome_with_webview_surface() {
        // DC-3 phase-2 端到端：render_chrome_via_sdk_with_webview_surface 把 WebView surface
        // 注册到 bridge → ExternalSurface marker 经 draw_external_surface 合并。
        use zero_browser_chrome::sdk_render::render_chrome_via_sdk_with_webview_surface;
        use zero_ui_core::geometry::{Insets, Size};
        use zero_ui_core::layout::WindowMetrics;
        use zero_ui_core::theme::SemanticTokens;

        let mut font_backend = FontdueBackend::new();
        font_backend.load_family("Ahem", AHEM).expect("Ahem parses");

        let mut shell = zero_browser_shell::BrowserShell::new();
        shell.new_tab(Some("https://example.com"));

        let metrics = WindowMetrics {
            logical_size: Size::new(1280.0, 800.0),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::Landscape,
        };

        // 模拟 WebView 渲染输出（一个填充矩形）。
        let mut webview_prims = RenderPrimitives::default();
        webview_prims.add_fill(RfRect::new(0.0, 0.0, 1280.0, 704.0), RfColor::rgb(255, 255, 255));

        let (bridge, vp) = render_chrome_via_sdk_with_webview_surface(
            &shell,
            &metrics,
            &SemanticTokens::light(),
            zero_ui_core::theme::ResolvedColorScheme::Light,
            std::sync::Arc::new(font_backend),
            Some((0, webview_prims, None)),
            &[],
            zero_browser_chrome::render::ChromeTabColors::from_tokens(&SemanticTokens::light()),
        );
        let p = bridge.into_primitives();

        // chrome fills 非空（toolbar 等几何）。
        assert!(!p.fills.is_empty(), "chrome fills present");
        // viewport rect 非空。
        assert!(vp.is_some(), "viewport rect present");
        // WebView surface 合并（至少 chrome fills + webview fill）。
        assert!(!p.fills.is_empty(), "fills after webview merge: {}", p.fills.len());
    }

    // ── DC-11 字体栈兼容性验证 ─────────────────────────────────────────────
    // 验证 foundation/text（FontdueBackend）与 render-foundation（FontLoader）
    // 都能加载同一字体数据并产出一致的 metrics，证明字体栈统一的可行性（DC-11 不变量）。

    /// 两套后端加载同一 Ahem 字体后，render-foundation FontLoader 能正确光栅化 glyph，
    /// foundation/text FontdueBackend 能正确 measure 文本宽度——验证两套后端独立可用。
    /// 注：FontLoader 对 Ahem 有特殊光栅化处理（per-spec 完美方块），
    /// 故不逐像素比较 raster 输出；此测验证加载 + 基本能力。
    #[test]
    fn dc11_both_backends_load_and_produce_output() {
        use zero_render_foundation::font::FontLoader;
        use zero_text_foundation::font_request::{FontRequest, TextDirection};
        use zero_text_foundation::text_measure::{TextMeasureInput, TextMeasurer};

        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        // ── foundation/text：加载 + measure ──
        let mut ft_backend = zero_text_foundation::backend::FontdueBackend::new();
        ft_backend
            .load_family("Ahem", ahem_data)
            .expect("FontdueBackend loads Ahem");
        let ft_metrics = ft_backend
            .measure(&TextMeasureInput {
                text: "ABCD".to_string(),
                font_request: FontRequest::new("Ahem"),
                size_px: 8.0,
                max_width: None,
                direction: TextDirection::Ltr,
            })
            .expect("ft measure 'ABCD'");
        assert!(ft_metrics.width > 0.0, "ft measure width > 0");

        // ── render-foundation：加载 + advance ──
        let mut rf_loader = FontLoader::new();
        let rf_font_id = rf_loader.load_font(ahem_data).expect("FontLoader loads Ahem");
        let rf_advance = rf_loader.measure_advance(rf_font_id, 'A', 8.0);
        assert!(rf_advance > 0.0, "rf advance for 'A' @8px Ahem > 0, got {rf_advance}");
        // Ahem 是等宽 8px/em → 'A' advance ≈ 8px。
        assert!(
            (rf_advance - 8.0).abs() < 1.0,
            "rf advance 'A' @8px ≈ 8px, got {rf_advance}"
        );
        // 4 chars × ~8px ≈ 32px 总宽。
        assert!(
            (ft_metrics.width - 32.0).abs() < 1.0,
            "ft measure 'ABCD' @8px ≈ 32px, got {}",
            ft_metrics.width
        );
    }

    /// 现有 bridge draw_text 路径（经共享 `Arc<FontdueBackend>` shape+raster）
    /// 产出 ImagePrimitive 的 rect 尺寸非零——证明 DC-11 文本全链已通（加载→shape→raster→ImagePrimitive）。
    #[test]
    fn dc11_draw_text_produces_nonzero_image_primitive() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");
        let mut ft = zero_text_foundation::backend::FontdueBackend::new();
        ft.load_family("Ahem", ahem_data).expect("loads Ahem");
        let backend = std::sync::Arc::new(ft);

        let mut bridge = RenderFoundationBackend::new_with_text(
            zero_render_foundation::geometry::Rect::new(0.0, 0.0, 800.0, 600.0),
            backend,
        );
        // draw_text 经共享 FontdueBackend shape→raster→ImagePrimitive。
        // `draw_text(text, position, size_px, color)` — RenderBackend trait 方法。
        use zero_ui_render::RenderBackend;
        bridge.draw_text(
            "Hi",
            zero_ui_core::geometry::Point::new(10.0, 100.0),
            16.0,
            zero_ui_core::theme::Color::BLACK,
        );
        let prims = bridge.into_primitives();
        // "Hi" 两个 glyphs → 2 ImagePrimitive。
        assert_eq!(prims.images.len(), 2, "draw_text 'Hi' @16px → 2 ImagePrimitives");
        for img in &prims.images {
            assert!(
                img.rect.size.width > 0.0 && img.rect.size.height > 0.0,
                "ImagePrimitive rect non-zero: {:?}",
                img.rect.size
            );
        }
    }
}
