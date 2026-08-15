// WebView 图元消费层（DC-10）—— 把 WebView 输出的基础图元追加/变换到浏览器场景，
// 并按视口矩形 / 圆角矩形裁剪。
//
// 从 app_render.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs 同模式（self 字段、伴生函数、外部 crate 符号均直接可达）。

/// 从渲染图元估算文档高度（CSS 逻辑像素，fills + glyphs 下界）。
pub fn primitives_content_height(primitives: &RenderPrimitives) -> f32 {
    crate::page_scroll::primitives_content_height(primitives)
}

/// 将 compositor 页面位图转换为现有页面 image primitive，并应用视口变换与裁剪。
///
/// `allow_gpu_direct_shadow`：gpu_direct 帧（dmabuf 导入路径）在窗口渲染中由
/// compositor_import 纹理呈现（本函数返回空，避免双绘）；headless GPU 捕获
/// 渲染器没有导入纹理，须回退绘制采纳时保留的 RGBA 影子。
pub(crate) fn compositor_frame_primitives(
    frame: &crate::tab_snapshot::CompositorFrame,
    x_offset: f32,
    y_offset: f32,
    scale: f32,
    clip_viewport: ViewportClip,
    _allow_gpu_direct_shadow: bool,
) -> RenderPrimitives {
    #[cfg(target_os = "linux")]
    if frame.gpu_direct && !_allow_gpu_direct_shadow {
        return RenderPrimitives::new();
    }
    let mut source = RenderPrimitives::new();
    source.images.push(ImagePrimitive {
        // compositor 帧已按设备像素光栅化；先还原为 CSS 尺寸，再由下方
        // 统一坐标变换映射回物理像素，保持纹理采样一对一。
        rect: Rect::new(
            0.0,
            0.0,
            frame.width as f32 / scale.max(f32::EPSILON),
            frame.height as f32 / scale.max(f32::EPSILON),
        ),
        image_key: frame.image_key.clone(),
        clip: None,
    });
    transform_webview_primitives_extra(&source, x_offset, y_offset, scale, Some(clip_viewport))
}

/// 将 WebView 输出的基础图元追加到浏览器场景。
///
/// `clip_y` 为物理像素坐标 `(top, bottom)`，fill 与该区间求交后绘制，glyph 完全落在区间外则跳过。
/// `clip_rounded` 为 `(x, y, w, h, radius)`，将内容裁剪到圆角矩形内。
#[allow(clippy::too_many_arguments)]
pub fn append_webview_primitives(
    primitives: &RenderPrimitives,
    fills: &mut Vec<FillPrimitive>,
    glyphs: &mut Vec<GlyphDraw>,
    x_offset: f32,
    y_offset: f32,
    fallback_font_id: u32,
    s: f32,
    clip_y: Option<(f32, f32)>,
    clip_rounded: Option<(f32, f32, f32, f32, f32)>,
) -> bool {
    let fill_start = fills.len();
    let glyph_start = glyphs.len();

    for fill in &primitives.fills {
        let x = fill.rect.origin.x * s + x_offset;
        let mut y = fill.rect.origin.y * s + y_offset;
        let w = fill.rect.size.width * s;
        let mut h = fill.rect.size.height * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let bottom = y + h;
            if bottom <= clip_top || y >= clip_bottom {
                continue;
            }
            if y < clip_top {
                h -= clip_top - y;
                y = clip_top;
            }
            let bottom = y + h;
            if bottom > clip_bottom {
                h -= bottom - clip_bottom;
            }
            if h <= 0.0 {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            push_fill_clipped_to_rounded_rect(fills, x, y, w, h, fill.color, rx, ry, rw, rh, radius);
        } else {
            let mut translated = fill.clone();
            translated.rect.origin.x = x;
            translated.rect.origin.y = y;
            translated.rect.size.width = w;
            translated.rect.size.height = h;
            fills.push(translated);
        }
    }

    for glyph in &primitives.glyphs {
        let ch = glyph.code_point().unwrap_or('\0');
        if ch == '\0' && glyph.font_glyph_index().is_none() {
            continue;
        }
        let x = glyph.x * s + x_offset;
        let baseline_y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            if bottom <= clip_top || top >= clip_bottom || top < clip_top {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            let width = font_size * 0.6;
            if !axis_rect_intersects_rounded_rect(x, top, width, bottom - top, rx, ry, rw, rh, radius) {
                continue;
            }
        }
        glyphs.push(GlyphDraw {
            ch,
            font_glyph_index: glyph.font_glyph_index(),
            x,
            baseline_y,
            color: glyph.color,
            font_id: if glyph.font_id.0 == 0 {
                fallback_font_id
            } else {
                glyph.font_id.0
            },
            font_variations: primitives.shared_font_variations(glyph.font_variation_id),
            font_size,
            rotation: 0.0,
        });
    }

    fills.len() > fill_start || glyphs.len() > glyph_start
}

/// 页面视口裁剪区（物理像素）。
#[derive(Debug, Clone, Copy)]
pub struct ViewportClip {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl ViewportClip {
    /// 由 `(x, y, w, h)` 构造视口裁剪矩形。
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + w,
            bottom: y + h,
        }
    }

    fn excludes(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w <= self.left || x >= self.right || y + h <= self.top || y >= self.bottom
    }
}

fn clip_axis_aligned_rect(x: f32, y: f32, w: f32, h: f32, clip: ViewportClip) -> Option<(f32, f32, f32, f32)> {
    if clip.excludes(x, y, w, h) {
        return None;
    }
    let left = x.max(clip.left);
    let top = y.max(clip.top);
    let right = (x + w).min(clip.right);
    let bottom = (y + h).min(clip.bottom);
    let w = right - left;
    let h = bottom - top;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some((left, top, w, h))
}

fn clamp_rounded_rect_radii(rr: &mut zero_render_foundation::primitive::RoundedRectPrimitive) {
    let max_r = rr.rect.size.width.min(rr.rect.size.height) * 0.5;
    rr.top_left_radius = rr.top_left_radius.min(max_r);
    rr.top_right_radius = rr.top_right_radius.min(max_r);
    rr.bottom_right_radius = rr.bottom_right_radius.min(max_r);
    rr.bottom_left_radius = rr.bottom_left_radius.min(max_r);
}

fn clip_rect_field(rect: &mut Rect, clip: ViewportClip) -> bool {
    let Some((x, y, w, h)) =
        clip_axis_aligned_rect(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height, clip)
    else {
        return false;
    };
    rect.origin.x = x;
    rect.origin.y = y;
    rect.size.width = w;
    rect.size.height = h;
    true
}

fn path_vertices_bbox(vertices: &[f32]) -> Option<(f32, f32, f32, f32)> {
    if vertices.len() < 2 {
        return None;
    }
    let mut min_x = vertices[0];
    let mut max_x = vertices[0];
    let mut min_y = vertices[1];
    let mut max_y = vertices[1];
    for chunk in vertices.chunks(2).skip(1) {
        if chunk.len() < 2 {
            continue;
        }
        min_x = min_x.min(chunk[0]);
        max_x = max_x.max(chunk[0]);
        min_y = min_y.min(chunk[1]);
        max_y = max_y.max(chunk[1]);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

/// 将 WebView 的所有 13 种图元类型转换为浏览器坐标（应用 scale、offset、clip）。
///
/// 返回的 `RenderPrimitives` 中的图元坐标为物理像素，
/// 已经应用了 `scale_factor`、`offset` 和视口裁剪。
pub fn transform_webview_primitives(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_viewport: Option<ViewportClip>,
) -> RenderPrimitives {
    transform_webview_primitives_impl(primitives, x_offset, y_offset, s, clip_viewport, true)
}

/// 仅变换非 fills/glyphs 的图元类型（性能门禁优化 S2，2026-08-08）。
///
/// 浏览器每帧调用本函数生成「extra 图元」层——fills/glyphs 已由
/// [`append_webview_primitives`] 以相同 offset 数学处理，旧实现先变换再在调用方
/// 丢弃（`app_render.rs` 的 `.fills.clear()`/`.glyphs.clear()`），4400 元素页每帧
/// 白白克隆 ~11k fills + ~22k glyphs。跳过两段后 extra 层只含其余 11 类图元。
pub fn transform_webview_primitives_extra(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_viewport: Option<ViewportClip>,
) -> RenderPrimitives {
    transform_webview_primitives_impl(primitives, x_offset, y_offset, s, clip_viewport, false)
}

fn transform_webview_primitives_impl(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_viewport: Option<ViewportClip>,
    include_fills_glyphs: bool,
) -> RenderPrimitives {
    let mut out = RenderPrimitives::new();
    if include_fills_glyphs {
        out.font_variations = primitives.font_variations.clone();
    }

    // 1. 阴影
    for shadow in &primitives.shadows {
        let mut s_clone = shadow.clone();
        s_clone.rect.origin.x = s_clone.rect.origin.x * s + x_offset;
        s_clone.rect.origin.y = s_clone.rect.origin.y * s + y_offset;
        s_clone.rect.size.width *= s;
        s_clone.rect.size.height *= s;
        s_clone.offset_x *= s;
        s_clone.offset_y *= s;
        s_clone.blur_radius *= s;
        s_clone.spread_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut s_clone.rect, clip)
        {
            continue;
        }
        out.shadows.push(s_clone);
    }

    // 2. 填充矩形（include_fills_glyphs=false 时跳过——调用方已由 append_webview_primitives 处理）
    if include_fills_glyphs {
        for fill in &primitives.fills {
            let x = fill.rect.origin.x * s + x_offset;
            let y = fill.rect.origin.y * s + y_offset;
            let w = fill.rect.size.width * s;
            let h = fill.rect.size.height * s;
            let Some((x, y, w, h)) = clip_viewport
                .and_then(|clip| clip_axis_aligned_rect(x, y, w, h, clip))
                .or_else(|| {
                    if clip_viewport.is_some() {
                        None
                    } else {
                        Some((x, y, w, h))
                    }
                })
            else {
                continue;
            };
            out.fills.push(FillPrimitive {
                rect: Rect::new(x, y, w, h),
                color: fill.color,
            });
        }
    }

    // 3. 圆角矩形
    for rr in &primitives.rounded_rects {
        let mut r_clone = rr.clone();
        r_clone.rect.origin.x = r_clone.rect.origin.x * s + x_offset;
        r_clone.rect.origin.y = r_clone.rect.origin.y * s + y_offset;
        r_clone.rect.size.width *= s;
        r_clone.rect.size.height *= s;
        r_clone.top_left_radius *= s;
        r_clone.top_right_radius *= s;
        r_clone.bottom_right_radius *= s;
        r_clone.bottom_left_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut r_clone.rect, clip)
        {
            continue;
        }
        clamp_rounded_rect_radii(&mut r_clone);
        out.rounded_rects.push(r_clone);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        let mut g_clone = gradient.clone();
        g_clone.rect.origin.x = g_clone.rect.origin.x * s + x_offset;
        g_clone.rect.origin.y = g_clone.rect.origin.y * s + y_offset;
        g_clone.rect.size.width *= s;
        g_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut g_clone.rect, clip)
        {
            continue;
        }
        g_clone.kind = match g_clone.kind {
            GradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear {
                x0: x0 * s + x_offset,
                y0: y0 * s + y_offset,
                x1: x1 * s + x_offset,
                y1: y1 * s + y_offset,
            },
            GradientKind::Radial {
                cx,
                cy,
                inner_radius,
                outer_radius,
            } => GradientKind::Radial {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                inner_radius: inner_radius * s,
                outer_radius: outer_radius * s,
            },
            GradientKind::Conic { cx, cy, start_angle } => GradientKind::Conic {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                start_angle,
            },
        };
        out.gradients.push(g_clone);
    }

    // 5. 图片（裁剪须用 `clip` 字段，不可缩小 `rect`，否则会拉伸纹理）
    for image in &primitives.images {
        let x = image.rect.origin.x * s + x_offset;
        let y = image.rect.origin.y * s + y_offset;
        let w = image.rect.size.width * s;
        let h = image.rect.size.height * s;
        let full_rect = Rect::new(x, y, w, h);

        if let Some(clip) = clip_viewport
            && clip.excludes(
                full_rect.origin.x,
                full_rect.origin.y,
                full_rect.size.width,
                full_rect.size.height,
            )
        {
            continue;
        }

        let mut i_clone = image.clone();
        i_clone.rect = full_rect;
        if let Some(clip) = &image.clip {
            i_clone.clip = Some(Rect::new(
                clip.origin.x * s + x_offset,
                clip.origin.y * s + y_offset,
                clip.size.width * s,
                clip.size.height * s,
            ));
        } else {
            i_clone.clip = None;
        }

        if let Some(clip) = clip_viewport {
            let window = Rect::new(clip.left, clip.top, clip.right - clip.left, clip.bottom - clip.top);
            i_clone.clip = match i_clone.clip {
                Some(existing) => existing.intersection(&window),
                None => Some(window),
            };
            if i_clone.clip.is_none() {
                continue;
            }
        }

        out.images.push(i_clone);
    }

    // 6. 线段
    for stroke in &primitives.strokes {
        let mut st = stroke.clone();
        st.x1 = st.x1 * s + x_offset;
        st.y1 = st.y1 * s + y_offset;
        st.x2 = st.x2 * s + x_offset;
        st.y2 = st.y2 * s + y_offset;
        st.width *= s;
        if let Some(clip) = clip_viewport {
            let pad = st.width * 0.5;
            let min_x = st.x1.min(st.x2) - pad;
            let min_y = st.y1.min(st.y2) - pad;
            let max_x = st.x1.max(st.x2) + pad;
            let max_y = st.y1.max(st.y2) + pad;
            if clip.excludes(min_x, min_y, max_x - min_x, max_y - min_y) {
                continue;
            }
        }
        out.strokes.push(st);
    }

    // 7. 路径填充
    for pf in &primitives.path_fills {
        let mut p_clone = pf.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
            && clip.excludes(x, y, w, h)
        {
            continue;
        }
        out.path_fills.push(p_clone);
    }

    // 8. 路径描边
    for ps in &primitives.path_strokes {
        let mut p_clone = ps.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        p_clone.line_width *= s;
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
        {
            let pad = p_clone.line_width * 0.5;
            if clip.excludes(x - pad, y - pad, w + pad * 2.0, h + pad * 2.0) {
                continue;
            }
        }
        out.path_strokes.push(p_clone);
    }

    // 9. 文字（include_fills_glyphs=false 时跳过——调用方已由 append_webview_primitives 处理）
    if include_fills_glyphs {
        for glyph in &primitives.glyphs {
            let x = glyph.x * s + x_offset;
            let y = glyph.y * s + y_offset;
            let font_size = glyph.font_size * s;
            if let Some(clip) = clip_viewport {
                let top = y - font_size;
                let bottom = y + font_size * 0.25;
                let width = font_size * 0.6;
                if clip.excludes(x, top, width, bottom - top) {
                    continue;
                }
            }
            out.glyphs.push(GlyphPrimitive {
                x,
                y,
                font_size,
                color: glyph.color,
                glyph_id: glyph.glyph_id,
                font_glyph_index: None,
                source: None,
                font_id: glyph.font_id,
                font_variation_id: glyph.font_variation_id,
                bitmap_width: glyph.bitmap_width,
                bitmap_height: glyph.bitmap_height,
                rotation: glyph.rotation,
                synthetic_italic: glyph.synthetic_italic,
            });
        }
    }

    // 10. 裁剪
    for clip in &primitives.clips {
        let mut c_clone = clip.clone();
        c_clone.rect.origin.x = c_clone.rect.origin.x * s + x_offset;
        c_clone.rect.origin.y = c_clone.rect.origin.y * s + y_offset;
        c_clone.rect.size.width *= s;
        c_clone.rect.size.height *= s;
        if let Some(viewport) = clip_viewport
            && !clip_rect_field(&mut c_clone.rect, viewport)
        {
            continue;
        }
        out.clips.push(c_clone);
    }

    // 11. 变换
    for transform in &primitives.transforms {
        let mut t_clone = transform.clone();
        t_clone.rect.origin.x = t_clone.rect.origin.x * s + x_offset;
        t_clone.rect.origin.y = t_clone.rect.origin.y * s + y_offset;
        t_clone.rect.size.width *= s;
        t_clone.rect.size.height *= s;
        t_clone.origin_x = t_clone.origin_x * s + x_offset;
        t_clone.origin_y = t_clone.origin_y * s + y_offset;
        t_clone.tx *= s;
        t_clone.ty *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut t_clone.rect, clip)
        {
            continue;
        }
        out.transforms.push(t_clone);
    }

    // 12. 滤镜
    for filter in &primitives.filters {
        let mut f_clone = filter.clone();
        f_clone.rect.origin.x = f_clone.rect.origin.x * s + x_offset;
        f_clone.rect.origin.y = f_clone.rect.origin.y * s + y_offset;
        f_clone.rect.size.width *= s;
        f_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut f_clone.rect, clip)
        {
            continue;
        }
        out.filters.push(f_clone);
    }

    // 13. 混合模式
    for blend in &primitives.blend_modes {
        let mut b_clone = blend.clone();
        b_clone.rect.origin.x = b_clone.rect.origin.x * s + x_offset;
        b_clone.rect.origin.y = b_clone.rect.origin.y * s + y_offset;
        b_clone.rect.size.width *= s;
        b_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut b_clone.rect, clip)
        {
            continue;
        }
        out.blend_modes.push(b_clone);
    }

    out
}

#[cfg(test)]
mod compositor_frame_tests {
    use super::*;
    use crate::tab_snapshot::CompositorFrame;
    use zero_render_foundation::image_cache::ImageKey;

    #[test]
    fn compositor_image_applies_scroll_scale_and_viewport_clip() {
        let frame = CompositorFrame {
            surface_id: 7,
            navigation_epoch: 2,
            frame_id: 9,
            width: 100,
            height: 80,
            image_key: ImageKey::new(3),
            #[cfg(target_os = "linux")]
            gpu_direct: false,
        };
        let viewport = ViewportClip::new(25.0, 30.0, 100.0, 90.0);

        let primitives = compositor_frame_primitives(
            &frame,
            20.0,  // viewport x - horizontal scroll
            -10.0, // viewport y - vertical scroll
            2.0,
            viewport,
            false,
        );

        assert_eq!(primitives.images.len(), 1);
        let image = &primitives.images[0];
        assert_eq!(image.rect, Rect::new(20.0, -10.0, 100.0, 80.0));
        assert_eq!(image.clip, Some(Rect::new(25.0, 30.0, 100.0, 90.0)));
        assert_eq!(image.image_key, ImageKey::new(3));
    }
}
