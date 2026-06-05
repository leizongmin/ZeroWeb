//! 效果绘制 — box-shadow、背景图片、CSS filter、mix-blend-mode、resize 手柄。
//!
//! 包含 paint_box_shadow、paint_background_image、apply_filter、apply_blend_mode、
//! paint_resize_handle、add_rounded_rect_metadata、paint_text_decoration，
//! 以及 background-position/size 辅助函数。

use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, FilterKind, FilterPrimitive, LineCap, LineStyle, ShadowPrimitive, StrokePrimitive,
};
use zero_style_system::{
    BackgroundImageComputedValue, BackgroundOriginComputedValue, BackgroundPositionComputedValue,
    BackgroundSizeComputedValue, ComputedStyle, FilterComputedValue, MixBlendModeComputedValue, ResizeValue,
    TextDecorationLineValue,
};

use super::super::color::color_value_to_render;
use super::super::helpers::{BorderRadiusSpec, gradient_to_primitive, simple_hash};

impl super::Painter {
    /// 添加圆角矩形元数据图元。
    ///
    /// 在当前渲染架构下，使用额外的 0-尺寸填充图元记录圆角参数。
    pub(super) fn add_rounded_rect_metadata(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _radii: &BorderRadiusSpec) {
        // 圆角信息通过 CornerFill 图元存储。
        // 在完整实现中会生成圆角裁剪蒙版或扇形填充。
        // 当前阶段记录圆角存在，待后续渲染后端支持。
    }

    /// 绘制 box-shadow（盒阴影效果）。
    pub(super) fn paint_box_shadow(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let shadow = &style.box_shadow;

        if shadow.offset_x == 0.0 && shadow.offset_y == 0.0 && shadow.blur_radius == 0.0 && shadow.spread_radius == 0.0
        {
            return;
        }

        let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
        let color = color_value_to_render(&shadow.color);

        self.primitives.add_shadow(ShadowPrimitive {
            rect,
            color,
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur_radius: shadow.blur_radius,
            spread_radius: shadow.spread_radius,
        });
    }

    /// 绘制背景图片 / 渐变。
    pub(super) fn paint_background_image(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_render_foundation::image_cache::ImageKey;
        use zero_render_foundation::primitive::ImagePrimitive;

        // 计算 background-origin 定位区域
        let (origin_x, origin_y, origin_w, origin_h) = match style.background_origin {
            BackgroundOriginComputedValue::BorderBox => (abs_x, abs_y, box_node.width, box_node.height),
            BackgroundOriginComputedValue::PaddingBox => (
                abs_x + box_node.border_left,
                abs_y + box_node.border_top,
                box_node.width - box_node.border_left - box_node.border_right,
                box_node.height - box_node.border_top - box_node.border_bottom,
            ),
            BackgroundOriginComputedValue::ContentBox => (
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            ),
        };

        let img_w = origin_w;
        let img_h = origin_h;

        let (sized_w, sized_h) = resolve_background_size(&style.background_size, origin_w, origin_h, img_w, img_h);
        let (offset_x, offset_y) =
            resolve_background_position(&style.background_position, origin_w, origin_h, sized_w, sized_h);

        let positioned_x = origin_x + offset_x;
        let positioned_y = origin_y + offset_y;

        match &style.background_image {
            BackgroundImageComputedValue::None => {}
            BackgroundImageComputedValue::Url(url) => {
                let key = simple_hash(url);
                let rect = Rect::new(positioned_x, positioned_y, sized_w, sized_h);
                self.primitives.add_image(ImagePrimitive {
                    rect,
                    image_key: ImageKey::new(key),
                });
            }
            BackgroundImageComputedValue::Gradient(gradient) => {
                let rect = Rect::new(positioned_x, positioned_y, sized_w, sized_h);
                if let Some(prim) = gradient_to_primitive(gradient, &rect) {
                    self.primitives.add_gradient(prim);
                }
            }
        }
    }

    /// 绘制文本装饰线（underline / overline / line-through）。
    pub(crate) fn paint_text_decoration(
        &mut self,
        base_x: f32,
        baseline_y: f32,
        font_size: f32,
        total_width: f32,
        color: Color,
        decoration: &TextDecorationLineValue,
    ) {
        if total_width <= 0.0 {
            return;
        }
        let line_width = (font_size * 0.06).max(1.0);

        match decoration {
            TextDecorationLineValue::None => {}
            TextDecorationLineValue::Underline => {
                let y = baseline_y + font_size * 0.15;
                self.primitives
                    .add_fill(Rect::new(base_x, y, total_width, line_width), color);
            }
            TextDecorationLineValue::Overline => {
                let y = baseline_y - font_size;
                self.primitives
                    .add_fill(Rect::new(base_x, y, total_width, line_width), color);
            }
            TextDecorationLineValue::LineThrough => {
                let y = baseline_y - font_size * 0.35;
                self.primitives
                    .add_fill(Rect::new(base_x, y, total_width, line_width), color);
            }
            TextDecorationLineValue::Blink => {}
        }
    }

    /// 应用 CSS filter。
    pub(super) fn apply_filter(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let filters = match &style.filter {
            FilterComputedValue::None => return,
            f => vec![filter_computed_to_kind(f)],
        };

        if filters.is_empty() {
            return;
        }

        let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
        self.primitives.add_filter(FilterPrimitive { rect, filters });
    }

    /// 应用 CSS mix-blend-mode。
    pub(super) fn apply_blend_mode(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let mode = match style.mix_blend_mode {
            MixBlendModeComputedValue::Normal => return,
            MixBlendModeComputedValue::Multiply => BlendMode::Multiply,
            MixBlendModeComputedValue::Screen => BlendMode::Screen,
            MixBlendModeComputedValue::Overlay => BlendMode::Overlay,
            MixBlendModeComputedValue::Darken => BlendMode::Darken,
            MixBlendModeComputedValue::Lighten => BlendMode::Lighten,
            MixBlendModeComputedValue::ColorDodge => BlendMode::ColorDodge,
            MixBlendModeComputedValue::ColorBurn => BlendMode::ColorBurn,
            MixBlendModeComputedValue::HardLight => BlendMode::HardLight,
            MixBlendModeComputedValue::SoftLight => BlendMode::SoftLight,
            MixBlendModeComputedValue::Difference => BlendMode::Difference,
            MixBlendModeComputedValue::Exclusion => BlendMode::Exclusion,
            MixBlendModeComputedValue::Hue => BlendMode::Hue,
            MixBlendModeComputedValue::Saturation => BlendMode::Saturation,
            MixBlendModeComputedValue::Color => BlendMode::Color,
            MixBlendModeComputedValue::Luminosity => BlendMode::Luminosity,
        };
        let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
        self.primitives.add_blend_mode(BlendModePrimitive { rect, mode });
    }

    /// 绘制 resize 手柄指示器。
    pub(super) fn paint_resize_handle(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let handle_size = 8.0;
        let corner_x = abs_x + box_node.width - handle_size;
        let corner_y = abs_y + box_node.height - handle_size;

        let color = Color {
            r: 128,
            g: 128,
            b: 128,
            a: 180,
        };

        match style.resize {
            ResizeValue::None => {}
            ResizeValue::Both | ResizeValue::Block => {
                for i in 0..3 {
                    let offset = 2.0 + i as f32 * 2.5;
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: corner_x + handle_size,
                        y1: corner_y + offset,
                        x2: corner_x + offset,
                        y2: corner_y + handle_size,
                        width: 1.0,
                        color,
                        style: LineStyle::Solid,
                        cap: LineCap::Butt,
                    });
                }
            }
            ResizeValue::Horizontal | ResizeValue::Inline => {
                for i in 0..2 {
                    let y = corner_y + 2.0 + i as f32 * 3.0;
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: corner_x + 2.0,
                        y1: y,
                        x2: corner_x + handle_size,
                        y2: y,
                        width: 1.0,
                        color,
                        style: LineStyle::Solid,
                        cap: LineCap::Butt,
                    });
                }
            }
            ResizeValue::Vertical => {
                for i in 0..2 {
                    let x = corner_x + 2.0 + i as f32 * 3.0;
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: x,
                        y1: corner_y + 2.0,
                        x2: x,
                        y2: corner_y + handle_size,
                        width: 1.0,
                        color,
                        style: LineStyle::Solid,
                        cap: LineCap::Butt,
                    });
                }
            }
        }
    }
}

/// 将 ComputedStyle 中的 filter 值转换为渲染层 FilterKind。
fn filter_computed_to_kind(value: &FilterComputedValue) -> FilterKind {
    match value {
        FilterComputedValue::None => FilterKind::Blur(0.0),
        FilterComputedValue::Blur(px) => FilterKind::Blur(*px),
        FilterComputedValue::Brightness(n) => FilterKind::Brightness(*n),
        FilterComputedValue::Contrast(n) => FilterKind::Contrast(*n),
        FilterComputedValue::Grayscale(n) => FilterKind::Grayscale(*n),
        FilterComputedValue::HueRotate(deg) => FilterKind::HueRotate(*deg),
        FilterComputedValue::Invert(n) => FilterKind::Invert(*n),
        FilterComputedValue::Opacity(n) => FilterKind::Opacity(*n),
        FilterComputedValue::Saturate(n) => FilterKind::Saturate(*n),
        FilterComputedValue::Sepia(n) => FilterKind::Sepia(*n),
        FilterComputedValue::DropShadow(x, y, blur, color) => {
            FilterKind::DropShadow(*x, *y, *blur, super::super::color::color_value_to_render(color))
        }
    }
}

// ── background-position / background-size 辅助函数 ─────────────────────────

/// 计算 background-size 后的图片尺寸。
fn resolve_background_size(
    size: &BackgroundSizeComputedValue,
    container_w: f32,
    container_h: f32,
    img_w: f32,
    img_h: f32,
) -> (f32, f32) {
    match size {
        BackgroundSizeComputedValue::Auto => (img_w, img_h),
        BackgroundSizeComputedValue::Cover => {
            if img_w <= 0.0 || img_h <= 0.0 || container_w <= 0.0 || container_h <= 0.0 {
                return (container_w, container_h);
            }
            let scale = (container_w / img_w).max(container_h / img_h);
            (img_w * scale, img_h * scale)
        }
        BackgroundSizeComputedValue::Contain => {
            if img_w <= 0.0 || img_h <= 0.0 || container_w <= 0.0 || container_h <= 0.0 {
                return (container_w, container_h);
            }
            let scale = (container_w / img_w).min(container_h / img_h);
            (img_w * scale, img_h * scale)
        }
        BackgroundSizeComputedValue::Length(px) => {
            let w = *px;
            let h = if img_w > 0.0 { w * img_h / img_w } else { container_h };
            (w, h)
        }
        BackgroundSizeComputedValue::Percent(pct) => {
            let w = container_w * pct / 100.0;
            let h = if img_w > 0.0 { w * img_h / img_w } else { container_h };
            (w, h)
        }
    }
}

/// 将 background-position 单个分量解析为像素偏移。
fn resolve_position_component(pos: &BackgroundPositionComputedValue, container_size: f32, image_size: f32) -> f32 {
    match pos {
        BackgroundPositionComputedValue::Left | BackgroundPositionComputedValue::Top => 0.0,
        BackgroundPositionComputedValue::Center => (container_size - image_size) / 2.0,
        BackgroundPositionComputedValue::Right | BackgroundPositionComputedValue::Bottom => {
            (container_size - image_size).max(0.0)
        }
        BackgroundPositionComputedValue::Length(px) => *px,
        BackgroundPositionComputedValue::Percent(pct) => (container_size - image_size) * pct / 100.0,
        BackgroundPositionComputedValue::TwoValue(_, _) => 0.0,
    }
}

/// 计算 background-position 的 (x, y) 像素偏移。
fn resolve_background_position(
    pos: &BackgroundPositionComputedValue,
    container_w: f32,
    container_h: f32,
    img_w: f32,
    img_h: f32,
) -> (f32, f32) {
    match pos {
        BackgroundPositionComputedValue::TwoValue(x_pos, y_pos) => (
            resolve_position_component(x_pos, container_w, img_w),
            resolve_position_component(y_pos, container_h, img_h),
        ),
        single => (
            resolve_position_component(single, container_w, img_w),
            resolve_position_component(&BackgroundPositionComputedValue::Center, container_h, img_h),
        ),
    }
}
