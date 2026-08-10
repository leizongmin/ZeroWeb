//! 效果绘制 — box-shadow、背景图片、CSS filter、mix-blend-mode、resize 手柄。
//!
//! 包含 paint_box_shadow、paint_background_image、apply_filter、apply_blend_mode、
//! paint_resize_handle、paint_text_decoration，
//! 以及 background-position/size 辅助函数。
//! 还包含 CSS 交互/提示属性指示器：cursor、image-rendering、isolation、
//! will-change、pointer-events、user-select、overscroll-behavior、touch-action。

use zero_css_parser::values::{ColorValue, LengthValue};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, FilterKind, FilterPrimitive, GradientPrimitive, LineCap, LineStyle, ShadowPrimitive,
    StrokePrimitive,
};
use zero_style_system::{
    AccentColorComputedValue, AppearanceComputedValue, BackgroundAttachmentComputedValue, BackgroundClipComputedValue,
    BackgroundImageComputedValue, BackgroundOriginComputedValue, BackgroundPositionComputedValue,
    BackgroundRepeatComputedValue, BackgroundSizeComputedValue, BgSizeComponentComputed, CaretColorComputedValue,
    ComputedStyle, FilterComputedValue, HyphensComputedValue, LineClampComputedValue, MixBlendModeComputedValue,
    QuotesComputedValue, ResizeValue, ScrollbarGutterComputedValue, ScrollbarWidthComputedValue,
    TextDecorationStyleValue, TextWrapComputedValue,
};

use super::super::color::color_value_to_render;
use super::super::helpers::{PrimitiveCounts, gradient_to_primitive, image_resource_key};
use super::effects_indicators::clip_tile_to_origin;

impl super::Painter {
    /// 绘制 box-shadow（盒阴影效果）。
    pub(super) fn paint_box_shadow(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        // CSS Backgrounds §7.2：多阴影按列表顺序绘制（先列先绘=底层）。空 Vec = none。
        // R2304：从单阴影改多阴影迭代。inset 仍未在 ShadowPrimitive 表达（paint 忽略，
        // 与既有行为一致；inset 渲染是独立更深 lever）。
        for shadow in &style.box_shadow {
            if shadow.offset_x == 0.0
                && shadow.offset_y == 0.0
                && shadow.blur_radius == 0.0
                && shadow.spread_radius == 0.0
            {
                continue;
            }

            // R2476：inset 阴影 perimeter = padding box（border 内），outset = border box。
            //（CSS Backgrounds §7.1：inner shadow casts as if border-box exterior is opaque,
            //  perimeter = padding edge。）box_node.width 为 border-box 宽。
            let rect = if shadow.inset {
                Rect::new(
                    abs_x + box_node.border_left,
                    abs_y + box_node.border_top,
                    (box_node.width - box_node.border_left - box_node.border_right).max(0.0),
                    (box_node.height - box_node.border_top - box_node.border_bottom).max(0.0),
                )
            } else {
                Rect::new(abs_x, abs_y, box_node.width, box_node.height)
            };
            // box-shadow 颜色：`currentColor` 使用元素自身计算 `color`（CSS-Color §resolving）。
            // color_value_to_render 无元素上下文会把 CurrentColor 回落为黑色，致 `color:transparent`
            // 时阴影错误地实心可见。driving: WPT box-shadow-currentcolor（与 text-decoration /
            // border 同族 currentColor 解析）。style.color 自身若仍为 CurrentColor（未解析继承），
            // color_value_to_render 回落黑色 = 既有行为，零回归。
            let color = if matches!(shadow.color, ColorValue::CurrentColor) {
                color_value_to_render(&style.color)
            } else {
                color_value_to_render(&shadow.color)
            };

            self.primitives.add_shadow(ShadowPrimitive {
                rect,
                color,
                offset_x: shadow.offset_x,
                offset_y: shadow.offset_y,
                blur_radius: shadow.blur_radius,
                spread_radius: shadow.spread_radius,
                inset: shadow.inset,
            });
        }
    }

    /// 绘制背景图片 / 渐变（支持多图层）。
    ///
    /// CSS 规范要求多图层按逆序渲染（最后一层在最底部）。
    /// 支持 background-repeat 渲染：根据 repeat 模式生成平铺的 ImagePrimitive。
    pub(super) fn paint_background_image(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if style.background_image.is_empty() {
            return;
        }
        // CSS §14.2：背景已传播到画布的元素（html/body）不在自身盒上绘制背景图像——
        // 画布已以视口 (0,0) 为 origin 平铺该图像；若此处再按元素 padding-box origin
        // 绘制，两者相位错位会产生可见的错位重影（R507）。
        if box_node
            .node_id
            .is_some_and(|id| self.canvas_propagated_node == Some(id))
        {
            return;
        }

        // 计算 background-origin 定位区域（positioning area）
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

        // R2312：painting area（clip）= background-clip box（CSS Backgrounds §3.7）。
        // 旧 impl 误把 origin box 当 clip，致 background-position 负偏移露出 border/padding 区时
        // 被错误裁掉（origin-content-box_with_position 等），且 background-clip≠origin 时图像
        // 裁剪/平铺区域错误。text 变体按既有简化当 content-box（无 glyph-mask 能力）。
        let (clip_x, clip_y, clip_w, clip_h) = match style.background_clip {
            BackgroundClipComputedValue::BorderBox => (abs_x, abs_y, box_node.width, box_node.height),
            BackgroundClipComputedValue::PaddingBox => (
                abs_x + box_node.border_left,
                abs_y + box_node.border_top,
                box_node.width - box_node.border_left - box_node.border_right,
                box_node.height - box_node.border_top - box_node.border_bottom,
            ),
            BackgroundClipComputedValue::ContentBox | BackgroundClipComputedValue::Text => (
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            ),
        };

        self.paint_bg_image_in_origin(
            origin_x, origin_y, origin_w, origin_h, clip_x, clip_y, clip_w, clip_h, style, 0.0, 0.0,
        );
    }

    /// R2063：background-attachment: fixed 的 bg-image 绘制入口。
    ///
    /// CSS §14.1：attachment:fixed 时 positioning area = 初始包含块（视口），painting area
    /// 仍为元素 background-origin 盒。即在元素盒上「开窗」显示锚定视口的平铺图像。driving
    /// test：background-attachment-applies-to-*（10 案，img attachment:fixed + repeat-x，元素
    /// 仅显示与视口锚定 tile 重叠的条带）。旧实现把 fixed 当 scroll（锚定元素盒）→ 整块图像
    /// 而非视口条带。kill-switch `ZW_BG_ATTACHMENT_FIXED=0`。
    pub(super) fn paint_background_image_fixed(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if std::env::var("ZW_BG_ATTACHMENT_FIXED").as_deref() == Ok("0") {
            // 回退：fixed 当 scroll（旧行为）——origin=clip=元素盒。
            self.paint_background_image(box_node, abs_x, abs_y, style);
            return;
        }
        if style.background_image.is_empty() {
            return;
        }
        if box_node
            .node_id
            .is_some_and(|id| self.canvas_propagated_node == Some(id))
        {
            return;
        }
        // painting area（clip）= 元素 background-origin 盒（与 paint_background_image 同源）。
        let (clip_x, clip_y, clip_w, clip_h) = match style.background_origin {
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
        // positioning area（origin）= 视口（初始包含块）。
        self.paint_bg_image_in_origin(
            0.0,
            0.0,
            self.viewport_w,
            self.viewport_h,
            clip_x,
            clip_y,
            clip_w,
            clip_h,
            style,
            0.0,
            0.0,
        );
    }

    /// 在指定矩形内绘制 background-image（含多图层逆序、size/position/repeat 解析、
    /// 平铺裁剪）。
    ///
    /// **positioning area**（`origin_*`）= background-size / background-position 的解析基准
    ///（CSS §14.1：attachment:fixed 时 = 初始包含块即视口，否则 = background-origin 盒）。
    /// **painting area**（`clip_*`）= 平铺范围与 tile 裁剪区域（元素的 background-origin 盒；
    /// 画布传播时 = 视口）。正常元素 `origin` ≡ `clip`；attachment:fixed 时 `origin`=视口、
    /// `clip`=元素盒，二者分离才能正确呈现「图像锚定视口、裁剪到元素」语义（R2063）。
    ///
    /// 元素背景由 `paint_background_image` 计算 origin/clip 后调用本函数；画布背景传播
    ///（CSS §14.2）直接以视口 (0,0,vw,vh) 同时作 origin+clip 调用本函数。
    pub(crate) fn paint_bg_image_in_origin(
        &mut self,
        origin_x: f32,
        origin_y: f32,
        origin_w: f32,
        origin_h: f32,
        clip_x: f32,
        clip_y: f32,
        clip_w: f32,
        clip_h: f32,
        style: &ComputedStyle,
        anchor_x: f32,
        anchor_y: f32,
    ) {
        use zero_render_foundation::image_cache::ImageKey;
        use zero_render_foundation::primitive::ImagePrimitive;

        if style.background_image.is_empty() {
            return;
        }

        // 解析背景图像固有尺寸（从 image_sizes 缓存查找）。
        // background-size: auto 时使用图像的原始像素尺寸，而非容器尺寸。
        let default_intrinsic = (origin_w, origin_h);
        let first_url_hash = style.background_image.iter().find_map(|layer| match layer {
            BackgroundImageComputedValue::Url(url) => Some(image_resource_key(url, self.document_url.as_deref())),
            _ => None,
        });
        let (img_w, img_h) = first_url_hash
            .and_then(|h| self.get_image_size(h))
            .unwrap_or(default_intrinsic);

        // CSS 规范：多图层逆序渲染（最后一层在最底，第一层在最上）。
        // R2311：background-position/size/repeat 均为多层 `<...>#`，按图层 cyclic 取值
        //（`longhands[layer_idx % longhands.len()]`）。单值 longhand（len=1）→ 所有层取 [0]
        // = 旧「单值应用到所有层」行为，**byte-identical 零回归**；仅多层 longhand 改变输出。
        // enumerate().rev()：tuple 的 index 即原始图层下标（rev 仅反序，index 仍随元素）。
        for (layer_idx, layer) in style.background_image.iter().enumerate().rev() {
            let size = &style.background_size[layer_idx % style.background_size.len()];
            let position = &style.background_position[layer_idx % style.background_position.len()];
            let repeat = &style.background_repeat[layer_idx % style.background_repeat.len()];

            // size/position 相对 positioning area（origin）解析（fixed 时 origin=视口）。
            let (sized_w, sized_h) = resolve_background_size(size, origin_w, origin_h, img_w, img_h);
            let (offset_x, offset_y) = resolve_background_position(position, origin_w, origin_h, sized_w, sized_h);

            // positioned = 定位区域 origin + bg-position offset + anchor 偏移。
            // R1428：anchor 用于 canvas 传播（CSS §14.2.3 根背景传播到画布时，positioning area =
            // 根元素盒含 margin 偏移，painting area = 画布）；正常元素 anchor=0（positioned=origin+offset）。
            // R2063：attachment:fixed 时 origin=视口(0,0) → positioned=视口锚定（图像不随元素滚动）。
            let positioned_x = origin_x + offset_x + anchor_x;
            let positioned_y = origin_y + offset_y + anchor_y;

            match layer {
                BackgroundImageComputedValue::None => {}
                BackgroundImageComputedValue::Url(url) => {
                    let key = image_resource_key(url, self.document_url.as_deref());

                    // R2312：repeat 平铺 painting area（clip）/ space·round 适配 positioning area（origin）。
                    let (repeat_x, repeat_y, tile_w, tile_h) = resolve_repeat_params(
                        repeat,
                        origin_x,
                        origin_y,
                        origin_w,
                        origin_h,
                        clip_x,
                        clip_y,
                        clip_w,
                        clip_h,
                        positioned_x,
                        positioned_y,
                        sized_w,
                        sized_h,
                    );

                    let mut y = repeat_y.0;
                    while y < repeat_y.1 {
                        let mut x = repeat_x.0;
                        while x < repeat_x.1 {
                            let clipped = clip_tile_to_origin(x, y, tile_w, tile_h, clip_x, clip_y, clip_w, clip_h);
                            if let Some((cx, cy, cw, ch)) = clipped {
                                self.primitives.add_image(ImagePrimitive {
                                    rect: Rect::new(cx, cy, cw, ch),
                                    image_key: ImageKey::new(key),
                                    clip: None,
                                });
                            }
                            x += tile_w;
                        }
                        y += tile_h;
                    }
                }
                BackgroundImageComputedValue::Gradient(gradient) => {
                    let rect = Rect::new(positioned_x, positioned_y, sized_w, sized_h);
                    if let Some(prim) = gradient_to_primitive(gradient, &rect, &style.color) {
                        self.primitives.add_gradient(prim);
                    }
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

        // 多值组合支持（CSS Text Decoration §3）：`underline overline line-through`
        // 可任意组合，每条启用的装饰线独立绘制。无任何装饰线（含 obsolete blink）早退。
        if !style.text_decoration_line.has_any() {
            return;
        }

        // 装饰颜色：CurrentColor 使用文本颜色
        let color = if matches!(style.text_decoration_color, ColorValue::CurrentColor) {
            text_color
        } else {
            color_value_to_render(&style.text_decoration_color)
        };

        // R1402：text-decoration-thickness 显式长度覆盖默认厚度。
        // 长度按 device px 向下取整（chromium 行为，text-decoration-thickness-length-rounding：
        // 2.3px→2px）。auto/from-font 保留字体度量近似（font_size×0.06，min 1px）。
        let line_width = match style.text_decoration_thickness {
            zero_style_system::TextDecorationThicknessValue::Length(n) => (n as f32).floor().max(1.0),
            zero_style_system::TextDecorationThicknessValue::Auto => (font_size * 0.06).max(1.0),
        };

        // R1607：text-decoration-inset 在 inline 轴内缩/延伸装饰线两端。
        // start 内缩 inline-start（正值=向内，负值=向外延伸），end 内缩 inline-end。
        // em/rem 按 font_size 解析（driver test text-decoration-inset-005 用 em）。
        let inset_to_px = |lv: &LengthValue| match lv {
            LengthValue::Px(n) => *n as f32,
            LengthValue::Em(n) => *n as f32 * font_size,
            // ch/rem 等罕见单位未解析 → 视作 0（不影响默认渲染）
            _ => 0.0,
        };
        let inset_start = inset_to_px(&style.text_decoration_inset.start);
        let inset_end = inset_to_px(&style.text_decoration_inset.end);
        let line_x = base_x + inset_start;
        let line_w = total_width - inset_start - inset_end;
        if line_w <= 0.0 {
            return;
        }

        // R2522：text-underline-offset 下划线额外下沉（CSS Text Decoration 4 §2.5）。
        // auto = 0（保留既有 baseline+font_size×0.15 位置，字节不变）；正值=下沉、负值=上抬。
        // 仅 underline 受影响（overline/line-through 不受）。em/rem/% 按 font_size resolve
        // （% 相对 1em = font_size；driver test 002 用 px、percentage 用 %）。
        let underline_offset_px = match &style.text_underline_offset {
            zero_css_parser::values::TextUnderlineOffsetValue::Auto => 0.0,
            zero_css_parser::values::TextUnderlineOffsetValue::Length(lv) => match lv {
                LengthValue::Px(n) => *n as f32,
                LengthValue::Em(n) | LengthValue::Rem(n) => *n as f32 * font_size,
                LengthValue::Percentage(n) => *n as f32 / 100.0 * font_size,
                // ch/v* 罕见且须 viewport/字宽上下文 → 视作 0（与 inset_to_px 一致）。
                _ => 0.0,
            },
        };

        // 每条启用的装饰线在其 y 偏移绘制（单值时仅一条，与历史行为字节一致；
        // 多值时各线共享 color/width/inset，仅 y 偏移不同）。
        let decor_style = &style.text_decoration_style;
        if style.text_decoration_line.underline {
            self.paint_decoration_line(
                line_x,
                baseline_y + font_size * 0.15 + underline_offset_px,
                line_w,
                line_width,
                color,
                decor_style,
            );
        }
        if style.text_decoration_line.overline {
            self.paint_decoration_line(line_x, baseline_y - font_size, line_w, line_width, color, decor_style);
        }
        if style.text_decoration_line.line_through {
            self.paint_decoration_line(
                line_x,
                baseline_y - font_size * 0.35,
                line_w,
                line_width,
                color,
                decor_style,
            );
        }
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
        // R2306：filter 多函数列表（CSS Filter Effects：<filter-function>+）。空 Vec = none。
        // render 侧 FilterPrimitive.filters: Vec<FilterKind> 已支持多函数顺序应用。
        let filters: Vec<_> = style.filter.iter().map(filter_computed_to_kind).collect();
        if filters.is_empty() {
            return;
        }

        let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
        self.primitives.add_filter(FilterPrimitive { rect, filters });
    }

    /// 应用 CSS backdrop-filter（对元素背后内容应用滤镜）。
    ///
    /// backdrop-filter 在元素自身内容绘制之前应用，影响该元素区域内的所有已绘制内容。
    pub(super) fn apply_backdrop_filter(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        // R2306：backdrop-filter 多函数列表（同 filter）。空 Vec = none。
        let filters: Vec<_> = style.backdrop_filter.iter().map(filter_computed_to_kind).collect();
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
        let default_font_id = self
            .resolve_font_id(&style.font_family, &style.font_weight, &style.font_style)
            .0;

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
                    font_glyph_index: None,
                    source: None,
                    font_id: default_font_id,
                    bitmap_width: None,
                    bitmap_height: None,
                    rotation: 0.0,
                    synthetic_italic: false,
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
                    font_glyph_index: None,
                    source: None,
                    font_id: default_font_id,
                    bitmap_width: None,
                    bitmap_height: None,
                    rotation: 0.0,
                    synthetic_italic: false,
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

    /// 应用 CSS mask-image 蒙版效果。
    ///
    /// 对元素及其子元素产生的图元应用蒙版裁剪：
    /// - 渐变蒙版：裁剪到渐变区域，并应用渐变式 alpha 衰减
    /// - URL 蒙版：暂不支持（需要图像加载基础设施）
    pub(super) fn apply_mask_image(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        counts_before: &PrimitiveCounts,
    ) {
        let mask_rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);

        for layer in &style.mask_image {
            match layer {
                BackgroundImageComputedValue::Gradient(gradient) => {
                    if let Some(gradient_prim) =
                        super::super::helpers::gradient_to_primitive(gradient, &mask_rect, &style.color)
                    {
                        // 渐变蒙版：将元素裁剪到渐变边界矩形
                        super::super::helpers::clip_all_primitives_to_rect(
                            &mut self.primitives,
                            counts_before,
                            &gradient_prim.rect,
                        );

                        // 对蒙版区域内的图元应用渐变式 alpha 衰减
                        let alpha_factor = compute_gradient_mask_alpha(&gradient_prim);
                        if alpha_factor < 1.0 {
                            super::super::helpers::apply_opacity_to_new_primitives(
                                &mut self.primitives,
                                counts_before,
                                alpha_factor as f32,
                            );
                        }
                    }
                }
                BackgroundImageComputedValue::Url(_) => {
                    // URL 蒙版需要图像加载基础设施，暂不实现
                }
                BackgroundImageComputedValue::None => {}
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
        // R2878：两值语法 `<w> <h>`（CSS Backgrounds §3.9）。每维独立解析；auto 维由另一维 +
        // 固有比推导，无固有比（渐变等）则取定位区该维尺寸；两维皆 auto 取固有尺寸。
        BackgroundSizeComputedValue::TwoValue(cw, ch) => {
            let has_ratio = img_w > 0.0 && img_h > 0.0;
            let w_fixed = match cw {
                BgSizeComponentComputed::Length(px) => Some(*px),
                BgSizeComponentComputed::Percent(p) => Some(container_w * p / 100.0),
                BgSizeComponentComputed::Auto => None,
            };
            let h_fixed = match ch {
                BgSizeComponentComputed::Length(px) => Some(*px),
                BgSizeComponentComputed::Percent(p) => Some(container_h * p / 100.0),
                BgSizeComponentComputed::Auto => None,
            };
            let w = match w_fixed {
                Some(w) => w,
                None => match h_fixed {
                    Some(h) if has_ratio => h * img_w / img_h,
                    Some(_) => container_w,
                    None if has_ratio => img_w,
                    None => container_w,
                },
            };
            let h = match h_fixed {
                Some(h) => h,
                None => match w_fixed {
                    Some(w) if has_ratio => w * img_h / img_w,
                    Some(_) => container_h,
                    None if has_ratio => img_h,
                    None => container_h,
                },
            };
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
        BackgroundPositionComputedValue::Calc(expr) => {
            // R2313：calc/min/max/clamp，% 相对 (container-image)（与 Percent 同语义，
            // eval_calc 的 parent_length 即 % 基准）。px/% 求值；em/vw 等需 font/viewport
            // 上下文（此处无）→ None → 回退 0.0（paint 期无 style 上下文，边界限制）。
            zero_css_parser::values::eval_calc(expr, Some((container_size - image_size) as f64)).unwrap_or(0.0) as f32
        }
        BackgroundPositionComputedValue::TwoValue(_, _) => 0.0,
        BackgroundPositionComputedValue::EdgeOffset(side, offset) => {
            // R2478：3/4 值「边缘+偏移」对（CSS Backgrounds §3.6）。offset 从命名边度量，
            // 递归用 resolve_position_component（offset 必为 length/percent/calc，故不产生
            // 关键字/TwoValue/EdgeOffset 再入）。left/top = offset 本身；right/bottom 翻转
            // （位置 = (container-image) - offset），与 bare right/bottom 关键字一致（right 0%≡right）。
            let off = resolve_position_component(offset, container_size, image_size);
            match side {
                zero_css_parser::values::BackgroundEdge::Left | zero_css_parser::values::BackgroundEdge::Top => off,
                zero_css_parser::values::BackgroundEdge::Right | zero_css_parser::values::BackgroundEdge::Bottom => {
                    (container_size - image_size) - off
                }
            }
        }
    }
}

/// 计算 background-position 的 (x, y) 像素偏移。
///
/// CSS 规范：单关键字时，horizontal keyword（left/right）应用于 x 轴，y 默认 center；
/// vertical keyword（top/bottom）应用于 y 轴，x 默认 center。
pub(super) fn resolve_background_position(
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
        // 垂直关键字：应用于 y 轴，x 默认 center
        BackgroundPositionComputedValue::Top | BackgroundPositionComputedValue::Bottom => (
            resolve_position_component(&BackgroundPositionComputedValue::Center, container_w, img_w),
            resolve_position_component(pos, container_h, img_h),
        ),
        // 水平关键字或 center：应用于 x 轴，y 默认 center
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
    clip_x: f32,
    clip_y: f32,
    clip_w: f32,
    clip_h: f32,
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

    // R2312：repeat/repeat-x/repeat-y 按 CSS §3.4 平铺覆盖 painting area（clip box）；
    // space/round 按 positioning area（origin box）适配（见下方 space/round 分支用 origin_*）。
    let x_range = |do_repeat: bool| {
        if do_repeat {
            // 从 clip 左边界开始，确保覆盖整个 painting area
            let start = clip_x - ((clip_x - positioned_x) % sized_w).abs();
            (start, clip_x + clip_w)
        } else {
            (positioned_x, positioned_x + sized_w)
        }
    };

    let y_range = |do_repeat: bool| {
        if do_repeat {
            let start = clip_y - ((clip_y - positioned_y) % sized_h).abs();
            (start, clip_y + clip_h)
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

/// 计算渐变蒙版的平均 alpha 衰减因子。
///
/// 简化实现：取渐变 stops 的平均 alpha 值。
fn compute_gradient_mask_alpha(gradient: &GradientPrimitive) -> f64 {
    if gradient.stops.is_empty() {
        return 1.0;
    }
    let total_alpha: f64 = gradient.stops.iter().map(|s| s.color.a as f64 / 255.0).sum();
    total_alpha / gradient.stops.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{GradientKind, GradientPrimitive, GradientStop};

    /// 测试 compute_gradient_mask_alpha — 空 stops 返回 1.0。
    #[test]
    fn test_mask_alpha_empty_stops() {
        let gradient = GradientPrimitive {
            interpolation: Default::default(),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            stops: vec![],
            repeating: false,
        };
        assert_eq!(compute_gradient_mask_alpha(&gradient), 1.0);
    }

    /// 测试 compute_gradient_mask_alpha — 全不透明 stops 返回 1.0。
    #[test]
    fn test_mask_alpha_fully_opaque() {
        let gradient = GradientPrimitive {
            interpolation: Default::default(),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::rgba(255, 0, 0, 255),
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::rgba(0, 0, 255, 255),
                },
            ],
            repeating: false,
        };
        assert!((compute_gradient_mask_alpha(&gradient) - 1.0).abs() < 0.001);
    }

    /// 测试 compute_gradient_mask_alpha — 半透明 stops 返回约 0.502。
    #[test]
    fn test_mask_alpha_half_transparent() {
        let gradient = GradientPrimitive {
            interpolation: Default::default(),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::rgba(0, 0, 0, 128),
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::rgba(0, 0, 0, 128),
                },
            ],
            repeating: false,
        };
        let expected = 128.0 / 255.0;
        assert!((compute_gradient_mask_alpha(&gradient) - expected).abs() < 0.01);
    }
}
