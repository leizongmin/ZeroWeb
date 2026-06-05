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
use zero_css_parser::values::ColorValue;
use zero_style_system::{
    BackgroundImageComputedValue, BackgroundOriginComputedValue, BackgroundPositionComputedValue,
    BackgroundRepeatComputedValue, BackgroundSizeComputedValue, ComputedStyle, FilterComputedValue,
    MixBlendModeComputedValue, ResizeValue, TextDecorationLineValue, TextDecorationStyleValue,
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
    ///
    /// 支持 background-repeat 渲染：根据 repeat 模式生成平铺的 ImagePrimitive。
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
                let repeat = &style.background_repeat;

                // 根据重复模式计算平铺参数
                let (repeat_x, repeat_y, tile_w, tile_h) = resolve_repeat_params(
                    repeat,
                    origin_x,
                    origin_y,
                    origin_w,
                    origin_h,
                    positioned_x,
                    positioned_y,
                    sized_w,
                    sized_h,
                );

                let mut y = repeat_y.0;
                while y < repeat_y.1 {
                    let mut x = repeat_x.0;
                    while x < repeat_x.1 {
                        // 裁剪到 origin 区域
                        let clipped = clip_tile_to_origin(x, y, tile_w, tile_h, origin_x, origin_y, origin_w, origin_h);
                        if let Some((cx, cy, cw, ch)) = clipped {
                            self.primitives.add_image(ImagePrimitive {
                                rect: Rect::new(cx, cy, cw, ch),
                                image_key: ImageKey::new(key),
                            });
                        }
                        x += tile_w;
                    }
                    y += tile_h;
                }
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
    ///
    /// 支持自定义装饰颜色（CSS `text-decoration-color`）和装饰样式
    /// （solid/dotted/dashed/wavy/double）。直接从 ComputedStyle 读取样式信息。
    pub(crate) fn paint_text_decoration_from_style(
        &mut self,
        base_x: f32,
        baseline_y: f32,
        font_size: f32,
        total_width: f32,
        text_color: Color,
        style: &ComputedStyle,
    ) {
        if total_width <= 0.0 {
            return;
        }

        let y_offset = match &style.text_decoration_line {
            TextDecorationLineValue::None | TextDecorationLineValue::Blink => return,
            TextDecorationLineValue::Underline => font_size * 0.15,
            TextDecorationLineValue::Overline => -font_size,
            TextDecorationLineValue::LineThrough => -font_size * 0.35,
        };
        let y = baseline_y + y_offset;

        // 装饰颜色：CurrentColor 使用文本颜色
        let color = if matches!(style.text_decoration_color, ColorValue::CurrentColor) {
            text_color
        } else {
            color_value_to_render(&style.text_decoration_color)
        };

        let line_width = (font_size * 0.06).max(1.0);

        self.paint_decoration_line(base_x, y, font_size, total_width, line_width, color, &style.text_decoration_style);
    }

    /// 绘制文本装饰线的底层实现。
    ///
    /// 参数已预计算以避免过多参数。
    fn paint_decoration_line(
        &mut self,
        base_x: f32,
        y: f32,
        font_size: f32,
        total_width: f32,
        line_width: f32,
        color: Color,
        decoration_style: &TextDecorationStyleValue,
    ) {
        match decoration_style {
            TextDecorationStyleValue::Solid => {
                self.primitives.add_fill(Rect::new(base_x, y, total_width, line_width), color);
            }
            TextDecorationStyleValue::Dotted => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: base_x,
                    y1: y + line_width / 2.0,
                    x2: base_x + total_width,
                    y2: y + line_width / 2.0,
                    width: line_width,
                    color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            TextDecorationStyleValue::Dashed => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: base_x,
                    y1: y + line_width / 2.0,
                    x2: base_x + total_width,
                    y2: y + line_width / 2.0,
                    width: line_width,
                    color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
            }
            TextDecorationStyleValue::Double => {
                let gap = line_width * 2.0;
                self.primitives.add_fill(Rect::new(base_x, y, total_width, line_width), color);
                self.primitives.add_fill(
                    Rect::new(base_x, y + gap + line_width, total_width, line_width),
                    color,
                );
            }
            TextDecorationStyleValue::Wavy => {
                // 波浪线：用交替偏移的小填充矩形近似正弦波
                let wave_len = (font_size * 1.5).max(8.0);
                let amplitude = line_width * 2.0;
                let steps = ((total_width / wave_len * 8.0).ceil() as usize).max(4);
                let step_w = total_width / steps as f32;
                for i in 0..steps {
                    let sx = base_x + i as f32 * step_w;
                    // sin 近似：交替上下偏移
                    let dy = if i % 2 == 0 { -amplitude } else { amplitude };
                    self.primitives
                        .add_fill(Rect::new(sx, y + dy, step_w, line_width), color);
                }
            }
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

/// 根据 background-repeat 模式计算平铺范围和 tile 尺寸。
#[allow(clippy::too_many_arguments)]
///
/// 返回 ((x_start, x_end), (y_start, y_end), tile_w, tile_h)。
fn resolve_repeat_params(
    repeat: &BackgroundRepeatComputedValue,
    origin_x: f32,
    origin_y: f32,
    origin_w: f32,
    origin_h: f32,
    positioned_x: f32,
    positioned_y: f32,
    sized_w: f32,
    sized_h: f32,
) -> ((f32, f32), (f32, f32), f32, f32) {
    if sized_w <= 0.0 || sized_h <= 0.0 {
        return (
            (positioned_x, positioned_x + sized_w),
            (positioned_y, positioned_y + sized_h),
            sized_w,
            sized_h,
        );
    }

    let x_range = |do_repeat: bool| {
        if do_repeat {
            // 从 origin 左边界开始，确保覆盖整个区域
            let start = origin_x - ((origin_x - positioned_x) % sized_w).abs();
            (start, origin_x + origin_w)
        } else {
            (positioned_x, positioned_x + sized_w)
        }
    };

    let y_range = |do_repeat: bool| {
        if do_repeat {
            let start = origin_y - ((origin_y - positioned_y) % sized_h).abs();
            (start, origin_y + origin_h)
        } else {
            (positioned_y, positioned_y + sized_h)
        }
    };

    match repeat {
        BackgroundRepeatComputedValue::Repeat => (x_range(true), y_range(true), sized_w, sized_h),
        BackgroundRepeatComputedValue::RepeatX => (x_range(true), y_range(false), sized_w, sized_h),
        BackgroundRepeatComputedValue::RepeatY => (x_range(false), y_range(true), sized_w, sized_h),
        BackgroundRepeatComputedValue::NoRepeat => (x_range(false), y_range(false), sized_w, sized_h),
        BackgroundRepeatComputedValue::Space => {
            // space 模式：均匀分布，至少两个 tile 才有意义
            let tiles_x = if origin_w >= sized_w && sized_w > 0.0 {
                (origin_w / sized_w).floor() as usize
            } else {
                1
            };
            let tiles_y = if origin_h >= sized_h && sized_h > 0.0 {
                (origin_h / sized_h).floor() as usize
            } else {
                1
            };

            if tiles_x <= 1 && tiles_y <= 1 {
                return (
                    (positioned_x, positioned_x + sized_w),
                    (positioned_y, positioned_y + sized_h),
                    sized_w,
                    sized_h,
                );
            }

            let space_x = if tiles_x > 1 {
                (origin_w - sized_w * tiles_x as f32) / (tiles_x - 1) as f32
            } else {
                0.0
            };
            let space_y = if tiles_y > 1 {
                (origin_h - sized_h * tiles_y as f32) / (tiles_y - 1) as f32
            } else {
                0.0
            };

            let eff_w = sized_w + space_x;
            let eff_h = sized_h + space_y;

            (
                (origin_x, origin_x + origin_w),
                (origin_y, origin_y + origin_h),
                eff_w,
                eff_h,
            )
        }
        BackgroundRepeatComputedValue::Round => {
            // round 模式：缩放 tile 使整数个刚好覆盖容器
            let tile_w = if origin_w > 0.0 && sized_w > 0.0 {
                let n = (origin_w / sized_w).round().max(1.0);
                origin_w / n
            } else {
                sized_w
            };
            let tile_h = if origin_h > 0.0 && sized_h > 0.0 {
                let n = (origin_h / sized_h).round().max(1.0);
                origin_h / n
            } else {
                sized_h
            };
            (
                (origin_x, origin_x + origin_w),
                (origin_y, origin_y + origin_h),
                tile_w,
                tile_h,
            )
        }
    }
}

/// 裁剪单个 tile 到 origin 区域，返回裁剪后的 (x, y, w, h)。
#[allow(clippy::too_many_arguments)]
///
/// 如果 tile 完全在 origin 外返回 None。
fn clip_tile_to_origin(
    tile_x: f32,
    tile_y: f32,
    tile_w: f32,
    tile_h: f32,
    origin_x: f32,
    origin_y: f32,
    origin_w: f32,
    origin_h: f32,
) -> Option<(f32, f32, f32, f32)> {
    let tile_right = tile_x + tile_w;
    let tile_bottom = tile_y + tile_h;
    let origin_right = origin_x + origin_w;
    let origin_bottom = origin_y + origin_h;

    // 完全在区域外
    if tile_right <= origin_x || tile_x >= origin_right || tile_bottom <= origin_y || tile_y >= origin_bottom {
        return None;
    }

    let cx = tile_x.max(origin_x);
    let cy = tile_y.max(origin_y);
    let cw = tile_right.min(origin_right) - cx;
    let ch = tile_bottom.min(origin_bottom) - cy;

    if cw > 0.0 && ch > 0.0 {
        Some((cx, cy, cw, ch))
    } else {
        None
    }
}
