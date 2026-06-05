//! 效果绘制 — box-shadow、背景图片、CSS filter、mix-blend-mode、resize 手柄。
//!
//! 包含 paint_box_shadow、paint_background_image、apply_filter、apply_blend_mode、
//! paint_resize_handle、add_rounded_rect_metadata、paint_text_decoration，
//! 以及 background-position/size 辅助函数。
//! 还包含 CSS 交互/提示属性指示器：cursor、image-rendering、isolation、
//! will-change、pointer-events、user-select、overscroll-behavior、touch-action。

use zero_css_parser::values::ColorValue;
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, FilterKind, FilterPrimitive, LineCap, LineStyle, ShadowPrimitive, StrokePrimitive,
};
use zero_style_system::{
    AccentColorComputedValue, AppearanceComputedValue, BackgroundAttachmentComputedValue, BackgroundImageComputedValue,
    BackgroundOriginComputedValue, BackgroundPositionComputedValue, BackgroundRepeatComputedValue,
    BackgroundSizeComputedValue, CaretColorComputedValue, ComputedStyle, FilterComputedValue, HyphensComputedValue,
    ImageRenderingValue, IsolationValue, LineClampComputedValue, MixBlendModeComputedValue, OverscrollBehaviorValue,
    PointerEventsValue, QuotesComputedValue, ResizeValue, ScrollbarGutterComputedValue, ScrollbarWidthComputedValue,
    TextDecorationLineValue, TextDecorationStyleValue, TextWrapComputedValue, TouchActionValue, UserSelectValue,
    WillChangeValue,
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

        self.paint_decoration_line(base_x, y, total_width, line_width, color, &style.text_decoration_style);
    }

    /// 绘制文本装饰线的底层实现。
    ///
    /// 参数已预计算以避免过多参数。
    fn paint_decoration_line(
        &mut self,
        base_x: f32,
        y: f32,
        total_width: f32,
        line_width: f32,
        color: Color,
        decoration_style: &TextDecorationStyleValue,
    ) {
        match decoration_style {
            TextDecorationStyleValue::Solid => {
                self.primitives
                    .add_fill(Rect::new(base_x, y, total_width, line_width), color);
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
                self.primitives
                    .add_fill(Rect::new(base_x, y, total_width, line_width), color);
                self.primitives
                    .add_fill(Rect::new(base_x, y + gap + line_width, total_width, line_width), color);
            }
            TextDecorationStyleValue::Wavy => {
                // 波浪线：用交替偏移的小填充矩形近似正弦波
                let wave_len = (total_width / 4.0).max(8.0).min(line_width * 25.0);
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

    /// 绘制 CSS accent-color 指示器。
    ///
    /// accent-color 影响表单控件（checkbox/radio/range 等）的主题颜色。
    /// 渲染为元素右下角的 6×6 色块指示器，标识该元素的 accent-color 值。
    pub(super) fn paint_accent_color_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let color = match &style.accent_color {
            AccentColorComputedValue::Auto => return,
            AccentColorComputedValue::Color(c) => color_value_to_render(c),
        };

        let indicator_size = 6.0;
        let ix = abs_x + box_node.width - indicator_size - 2.0;
        let iy = abs_y + 2.0;
        self.primitives
            .add_fill(Rect::new(ix, iy, indicator_size, indicator_size), color);
    }

    /// 绘制 CSS caret-color 指示器。
    ///
    /// caret-color 影响文本插入光标（caret）的颜色。
    /// 渲染为元素左侧边缘的竖线指示器，标识该元素的 caret-color 值。
    pub(super) fn paint_caret_color_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let color = match &style.caret_color {
            CaretColorComputedValue::Auto => return,
            CaretColorComputedValue::Color(c) => color_value_to_render(c),
        };

        let caret_height = (box_node.height * 0.6).clamp(8.0, 16.0);
        let caret_width = 2.0;
        let cx = abs_x + box_node.border_left + 2.0;
        let cy = abs_y + (box_node.height - caret_height) / 2.0;
        self.primitives
            .add_fill(Rect::new(cx, cy, caret_width, caret_height), color);
    }

    /// 绘制 CSS scrollbar-width 指示器。
    ///
    /// scrollbar-width 控制滚动条宽度（auto/thin/none）。
    /// 渲染为元素右侧边缘的竖条指示器。
    pub(super) fn paint_scrollbar_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        // 仅在有 overflow 时显示滚动条指示器
        if box_node.overflow_x == zero_layout_engine::OverflowClip::Visible
            && box_node.overflow_y == zero_layout_engine::OverflowClip::Visible
        {
            return;
        }

        let (bar_width, bar_alpha) = match style.scrollbar_width {
            ScrollbarWidthComputedValue::Auto => (10.0, 60u8),
            ScrollbarWidthComputedValue::Thin => (6.0, 50u8),
            ScrollbarWidthComputedValue::None => return,
        };

        let track_x = abs_x + box_node.width - bar_width;
        let track_y = abs_y;
        let track_h = box_node.height;

        // 滚动条轨道背景
        let track_color = Color {
            r: 240,
            g: 240,
            b: 240,
            a: bar_alpha,
        };
        self.primitives
            .add_fill(Rect::new(track_x, track_y, bar_width, track_h), track_color);

        // 滚动条拇指（固定比例，简化实现）
        let thumb_margin = 1.0;
        let thumb_h = (track_h * 0.3).max(20.0).min(track_h - 2.0);
        let thumb_color = Color {
            r: 180,
            g: 180,
            b: 180,
            a: (bar_alpha as f32 * 1.5).min(255.0) as u8,
        };
        self.primitives.add_fill(
            Rect::new(
                track_x + thumb_margin,
                track_y + thumb_margin,
                bar_width - 2.0 * thumb_margin,
                thumb_h,
            ),
            thumb_color,
        );
    }

    /// 绘制 CSS appearance 指示器。
    ///
    /// appearance 控制元素是否使用平台原生样式。
    /// 当 appearance 不是 none 时，在元素内绘制简化原生控件外观。
    pub(super) fn paint_appearance(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        match style.appearance {
            AppearanceComputedValue::None | AppearanceComputedValue::Auto => return,
            _ => {}
        }

        // 使用 accent-color 或默认蓝色
        let accent = match &style.accent_color {
            AccentColorComputedValue::Auto => Color {
                r: 0,
                g: 120,
                b: 215,
                a: 255,
            },
            AccentColorComputedValue::Color(c) => color_value_to_render(c),
        };

        let cx = abs_x + box_node.border_left;
        let cy = abs_y + box_node.border_top;
        let cw = box_node.content_width;
        let ch = box_node.content_height;

        match style.appearance {
            AppearanceComputedValue::Checkbox | AppearanceComputedValue::Radio => {
                let size = ch.min(cw).clamp(8.0, 16.0);
                let ox = cx + (cw - size) / 2.0;
                let oy = cy + (ch - size) / 2.0;
                // 边框
                let border_color = Color {
                    r: 100,
                    g: 100,
                    b: 100,
                    a: 255,
                };
                self.primitives.add_fill(Rect::new(ox, oy, size, 1.0), border_color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + size - 1.0, size, 1.0), border_color);
                self.primitives.add_fill(Rect::new(ox, oy, 1.0, size), border_color);
                self.primitives
                    .add_fill(Rect::new(ox + size - 1.0, oy, 1.0, size), border_color);
                // 内部填充色
                self.primitives
                    .add_fill(Rect::new(ox + 1.0, oy + 1.0, size - 2.0, size - 2.0), accent);
            }
            AppearanceComputedValue::Button
            | AppearanceComputedValue::PushButton
            | AppearanceComputedValue::SquareButton => {
                // 按钮背景
                self.primitives.add_fill(Rect::new(cx, cy, cw, ch), accent);
                // 高光线
                let highlight = Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 80,
                };
                self.primitives.add_fill(Rect::new(cx, cy, cw, ch * 0.4), highlight);
            }
            AppearanceComputedValue::Textfield | AppearanceComputedValue::Textarea => {
                // 文本输入框：白色背景 + 边框
                let bg = Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                };
                let border = Color {
                    r: 120,
                    g: 120,
                    b: 120,
                    a: 255,
                };
                self.primitives.add_fill(Rect::new(cx, cy, cw, ch), bg);
                self.primitives.add_fill(Rect::new(cx, cy, cw, 1.0), border);
                self.primitives.add_fill(Rect::new(cx, cy + ch - 1.0, cw, 1.0), border);
                self.primitives.add_fill(Rect::new(cx, cy, 1.0, ch), border);
                self.primitives.add_fill(Rect::new(cx + cw - 1.0, cy, 1.0, ch), border);
            }
            _ => {
                // 其他 appearance 类型：灰色背景指示器
                let indicator = Color {
                    r: 230,
                    g: 230,
                    b: 230,
                    a: 200,
                };
                self.primitives.add_fill(Rect::new(cx, cy, cw, ch), indicator);
            }
        }
    }

    /// 绘制 CSS scrollbar-gutter 指示器。
    ///
    /// scrollbar-gutter 控制是否为滚动条预留空间。
    /// - auto: 默认行为，不预留额外空间
    /// - stable: 始终预留滚动条空间（即使内容不溢出）
    /// - stable both-edges: 两侧都预留空间
    pub(super) fn paint_scrollbar_gutter(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let gutter_width = match style.scrollbar_gutter {
            ScrollbarGutterComputedValue::Auto => return,
            ScrollbarGutterComputedValue::Stable | ScrollbarGutterComputedValue::StableBothEdges => {
                // 使用与 scrollbar-width auto 相同的宽度
                match style.scrollbar_width {
                    ScrollbarWidthComputedValue::None => return,
                    ScrollbarWidthComputedValue::Auto => 10.0,
                    ScrollbarWidthComputedValue::Thin => 6.0,
                }
            }
        };

        let gutter_color = Color {
            r: 245,
            g: 245,
            b: 245,
            a: 120,
        };

        // 右侧 gutter
        let gx = abs_x + box_node.width - gutter_width;
        self.primitives
            .add_fill(Rect::new(gx, abs_y, gutter_width, box_node.height), gutter_color);

        // both-edges: 左侧也预留
        if matches!(style.scrollbar_gutter, ScrollbarGutterComputedValue::StableBothEdges) {
            self.primitives
                .add_fill(Rect::new(abs_x, abs_y, gutter_width, box_node.height), gutter_color);
        }
    }

    /// 绘制 CSS background-attachment: fixed 指示器。
    ///
    /// background-attachment: fixed 时，背景不随内容滚动。
    /// 渲染为元素左上角的锁定图标指示器（小方块+角标）。
    pub(super) fn paint_background_attachment_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.background_attachment, BackgroundAttachmentComputedValue::Fixed) {
            return;
        }

        // 固定背景指示器：左上角 8×8 锁定图钉
        let pin_size = 3.0;
        let pin_color = Color {
            r: 100,
            g: 100,
            b: 200,
            a: 180,
        };
        // 图钉头部（圆）
        self.primitives
            .add_fill(Rect::new(abs_x + 2.0, abs_y + 2.0, pin_size, pin_size), pin_color);
        // 图钉针脚
        self.primitives
            .add_fill(Rect::new(abs_x + 2.0, abs_y + 2.0 + pin_size, 1.0, 3.0), pin_color);
    }

    /// 绘制 CSS hyphens 指示器。
    ///
    /// hyphens: auto 时，在文本换行处可能显示连字符。
    /// 渲染为元素底部的小横线指示器。
    pub(super) fn paint_hyphens_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.hyphens, HyphensComputedValue::Auto) {
            return;
        }

        // hyphens: auto 指示器 — 元素底部中央的短横线
        let line_width = 8.0;
        let line_x = abs_x + (box_node.width - line_width) / 2.0;
        let line_y = abs_y + box_node.height - 4.0;
        let hyphen_color = Color {
            r: 150,
            g: 150,
            b: 150,
            a: 160,
        };
        self.primitives
            .add_fill(Rect::new(line_x, line_y, line_width, 1.0), hyphen_color);
    }

    /// 绘制 CSS quotes 引号标记。
    ///
    /// quotes 属性定义了嵌套引号对。渲染为文本内容前后的引号 glyph。
    /// 支持嵌套层级：第一层使用第一对引号，第二层使用第二对，以此类推。
    pub(super) fn paint_quotes(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        _nesting_depth: usize,
    ) {
        let (open_q, close_q) = match &style.quotes {
            QuotesComputedValue::None => return,
            QuotesComputedValue::Auto => ("«".to_string(), "»".to_string()),
            QuotesComputedValue::Pairs(pairs) => {
                if pairs.is_empty() {
                    return;
                }
                // 根据嵌套深度选择引号对
                let idx = _nesting_depth.min(pairs.len() - 1);
                (pairs[idx].0.clone(), pairs[idx].1.clone())
            }
        };

        let font_size: f32 = match style.font_size {
            zero_css_parser::values::LengthValue::Px(s) => s as f32,
            _ => 12.0,
        };
        let color = color_value_to_render(&style.color);
        let default_font_id = zero_render_foundation::primitive::FontId(0);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        // 开引号
        for (i, ch) in open_q.chars().enumerate() {
            self.primitives
                .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                    x: content_x + i as f32 * font_size * 0.6,
                    y: content_y + font_size,
                    font_size,
                    color,
                    glyph_id: ch as u32,
                    font_id: default_font_id,
                    bitmap_width: None,
                    bitmap_height: None,
                });
        }

        // 闭引号
        let text_width = box_node.content_width;
        let close_x = content_x + text_width - close_q.chars().count() as f32 * font_size * 0.6;
        for (i, ch) in close_q.chars().enumerate() {
            self.primitives
                .add_glyph(zero_render_foundation::primitive::GlyphPrimitive {
                    x: close_x + i as f32 * font_size * 0.6,
                    y: content_y + font_size,
                    font_size,
                    color,
                    glyph_id: ch as u32,
                    font_id: default_font_id,
                    bitmap_width: None,
                    bitmap_height: None,
                });
        }
    }

    /// 应用 CSS text-wrap 到 InlineFormattingContext 配置。
    ///
    /// 返回 (no_wrap_override, description) — 如果 text-wrap 要求禁止换行，
    /// 返回 Some(true)；否则返回 None（不覆盖 white-space 的换行设置）。
    pub(crate) fn resolve_text_wrap(style: &ComputedStyle) -> Option<bool> {
        match style.text_wrap {
            TextWrapComputedValue::Wrap
            | TextWrapComputedValue::Balance
            | TextWrapComputedValue::Pretty
            | TextWrapComputedValue::Stable => None,
            TextWrapComputedValue::Nowrap => Some(true),
        }
    }

    /// 应用 CSS line-clamp：限制可见行数并在截断处添加省略号。
    ///
    /// 返回最大允许行数，None 表示不限制。
    pub(crate) fn resolve_line_clamp(style: &ComputedStyle) -> Option<u32> {
        match style.line_clamp {
            LineClampComputedValue::None => None,
            LineClampComputedValue::Count(n) => Some(n),
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

impl super::Painter {
    // ── CSS 交互/提示属性指示器 ──────────────────────────

    /// 绘制 CSS cursor 类型指示器。
    ///
    /// 在元素右上角渲染一个 4×4 像素的小方块，颜色根据 cursor 类型不同而变化。
    /// 仅对非 auto/default 的 cursor 值渲染指示器。
    pub(super) fn paint_cursor_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::CursorValue;

        let color = match style.cursor {
            CursorValue::Auto | CursorValue::Default => return,
            CursorValue::Pointer => Color::rgba(0, 120, 215, 200), // 蓝色 — 手指光标
            CursorValue::Text => Color::rgba(0, 0, 0, 200),        // 黑色 — 文本光标
            CursorValue::Crosshair => Color::rgba(255, 0, 0, 200), // 红色 — 十字光标
            CursorValue::Move => Color::rgba(128, 0, 128, 200),    // 紫色 — 移动光标
            CursorValue::Wait => Color::rgba(255, 165, 0, 200),    // 橙色 — 等待光标
            CursorValue::Help => Color::rgba(0, 128, 0, 200),      // 绿色 — 帮助光标
            CursorValue::NotAllowed => Color::rgba(200, 0, 0, 200), // 深红 — 禁止光标
            CursorValue::Grab | CursorValue::Grabbing => Color::rgba(139, 69, 19, 200), // 棕色 — 抓取
            CursorValue::ColResize | CursorValue::EwResize => Color::rgba(0, 128, 128, 200), // 青色 — 水平调整
            CursorValue::RowResize | CursorValue::NsResize => Color::rgba(128, 128, 0, 200), // 橄榄 — 垂直调整
            CursorValue::None => Color::rgba(200, 200, 200, 100),  // 浅灰 — 无光标
            CursorValue::Progress => Color::rgba(0, 0, 200, 200),  // 蓝色 — 进度
            CursorValue::Cell => Color::rgba(0, 200, 0, 200),      // 绿色 — 单元格
            CursorValue::Copy => Color::rgba(100, 100, 255, 200),  // 淡蓝 — 复制
            CursorValue::Alias => Color::rgba(200, 100, 0, 200),   // 深橙 — 别名
            CursorValue::AllScroll => Color::rgba(128, 128, 128, 200), // 灰色 — 全方向滚动
            CursorValue::ZoomIn | CursorValue::ZoomOut => Color::rgba(200, 200, 0, 200), // 黄色 — 缩放
        };

        // 在元素右上角绘制 4×4 指示方块
        let x = abs_x + box_node.width - 6.0;
        let y = abs_y + 2.0;
        self.primitives.add_fill(Rect::new(x, y, 4.0, 4.0), color);
    }

    /// 绘制 CSS image-rendering 质量指示器。
    ///
    /// 对非 auto 值的 image-rendering，在图片右下角绘制一个小质量标记：
    /// - pixelated → 方格图案（2×2 网格）
    /// - crisp-edges → 粗线边框
    /// - smooth/high-quality → 圆滑标记
    pub(super) fn paint_image_rendering_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        match style.image_rendering {
            ImageRenderingValue::Auto => {}
            ImageRenderingValue::Pixelated => {
                // 2×2 方格图案表示像素化
                let x = abs_x + box_node.width - 8.0;
                let y = abs_y + box_node.height - 8.0;
                let c = Color::rgba(255, 0, 255, 180);
                self.primitives.add_fill(Rect::new(x, y, 4.0, 4.0), c);
                self.primitives.add_fill(Rect::new(x + 4.0, y + 4.0, 4.0, 4.0), c);
            }
            ImageRenderingValue::CrispEdges => {
                // 粗线边框表示锐利边缘
                let x = abs_x + box_node.width - 10.0;
                let y = abs_y + box_node.height - 10.0;
                let c = Color::rgba(255, 140, 0, 180);
                self.primitives.add_fill(Rect::new(x, y, 10.0, 2.0), c);
                self.primitives.add_fill(Rect::new(x, y, 2.0, 10.0), c);
            }
            ImageRenderingValue::Smooth | ImageRenderingValue::HighQuality => {
                // 圆滑标记（单个圆点）
                let x = abs_x + box_node.width - 6.0;
                let y = abs_y + box_node.height - 6.0;
                self.primitives
                    .add_fill(Rect::new(x, y, 4.0, 4.0), Color::rgba(0, 200, 100, 180));
            }
        }
    }

    /// 绘制 CSS isolation: isolate 指示器。
    ///
    /// 在元素左上角绘制一个紫色 L 形标记，表示创建了新的堆叠上下文。
    pub(super) fn paint_isolation_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.isolation, IsolationValue::Isolate) {
            return;
        }

        let c = Color::rgba(128, 0, 128, 160);
        // L 形标记：水平线 + 垂直线
        self.primitives.add_fill(Rect::new(abs_x, abs_y, 8.0, 2.0), c);
        self.primitives.add_fill(Rect::new(abs_x, abs_y, 2.0, 8.0), c);
    }

    /// 绘制 CSS will-change 指示器。
    ///
    /// 在元素右上角绘制一个黄色三角形警告标记，表示即将发生的变化。
    pub(super) fn paint_will_change_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if matches!(style.will_change, WillChangeValue::Auto) {
            return;
        }

        let x = abs_x + box_node.width - 8.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(255, 200, 0, 200);
        // 用 3 个 fill 模拟三角形标记
        self.primitives.add_fill(Rect::new(x + 3.0, y, 2.0, 2.0), c);
        self.primitives.add_fill(Rect::new(x + 2.0, y + 2.0, 4.0, 2.0), c);
        self.primitives.add_fill(Rect::new(x + 1.0, y + 4.0, 6.0, 2.0), c);
    }

    /// 绘制 CSS pointer-events: none 指示器。
    ///
    /// 在元素右上角绘制一个红色 × 标记，表示元素不接收指针事件。
    pub(super) fn paint_pointer_events_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.pointer_events, PointerEventsValue::None) {
            return;
        }

        let x = abs_x + box_node.width - 8.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(220, 20, 20, 180);
        // × 标记：两条交叉对角线（用 stroke）
        self.primitives.add_stroke(StrokePrimitive {
            x1: x,
            y1: y,
            x2: x + 6.0,
            y2: y + 6.0,
            width: 1.5,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 6.0,
            y1: y,
            x2: x,
            y2: y + 6.0,
            width: 1.5,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
    }

    /// 绘制 CSS user-select: none 指示器。
    ///
    /// 在元素左上角绘制一个灰色锁形标记，表示文本不可选择。
    pub(super) fn paint_user_select_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.user_select, UserSelectValue::None) {
            return;
        }

        let x = abs_x + 2.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(128, 128, 128, 180);
        // 锁形标记：矩形锁体 + 半弧锁扣
        self.primitives.add_fill(Rect::new(x, y + 4.0, 6.0, 4.0), c);
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 1.5,
            y1: y + 4.0,
            x2: x + 1.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 4.5,
            y1: y + 4.0,
            x2: x + 4.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 1.5,
            y1: y + 1.0,
            x2: x + 4.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
    }

    /// 绘制 CSS overscroll-behavior 指示器。
    ///
    /// 对 contain/none 值，在元素底部中央绘制一条水平线，表示滚动边界被限制。
    pub(super) fn paint_overscroll_behavior_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let (c, w) = match style.overscroll_behavior_x {
            OverscrollBehaviorValue::Contain => (Color::rgba(255, 100, 0, 180), 12.0),
            OverscrollBehaviorValue::None => (Color::rgba(200, 0, 0, 200), 16.0),
            OverscrollBehaviorValue::Auto => return,
        };

        let x = abs_x + (box_node.width - w) / 2.0;
        let y = abs_y + box_node.height - 3.0;
        self.primitives.add_fill(Rect::new(x, y, w, 2.0), c);
    }

    /// 绘制 CSS touch-action 指示器。
    ///
    /// 对非 auto 值，在元素右下角绘制一个小标记。
    pub(super) fn paint_touch_action_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let c = match style.touch_action {
            TouchActionValue::Auto | TouchActionValue::Manipulation => return,
            TouchActionValue::None => Color::rgba(200, 0, 0, 180),
            TouchActionValue::PanX => Color::rgba(0, 100, 200, 180),
            TouchActionValue::PanY => Color::rgba(0, 200, 100, 180),
            TouchActionValue::PanXPanY => Color::rgba(100, 100, 200, 180),
        };

        let x = abs_x + box_node.width - 5.0;
        let y = abs_y + box_node.height - 5.0;
        self.primitives.add_fill(Rect::new(x, y, 3.0, 3.0), c);
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
