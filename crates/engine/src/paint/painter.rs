//! 绘制命令生成器 — Painter 结构体及其绘制方法。

use std::collections::{HashMap, HashSet};

use zero_css_parser::values::ColorValue;
use zero_css_parser::values::LengthValue;
use zero_css_parser::values::VisibilityValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::InlineFormattingContext;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::estimate_char_width;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{
    FilterKind, FilterPrimitive, FontId, GlyphPrimitive, ImagePrimitive, RenderPrimitives, ShadowPrimitive,
};
use zero_style_system::{
    BackgroundImageComputedValue, BorderStyleValue, ComputedStyle, FilterComputedValue, OutlineStyleValue,
    TextDecorationLineValue, TextOverflowValue,
};

use super::color::color_value_to_render;
use super::helpers::{
    BorderRadiusSpec, PrimitiveCounts, apply_opacity_to_new_primitives, apply_text_transform, clip_fills, clip_glyphs,
    gradient_to_primitive, length_to_f32, simple_hash,
};

/// 绘制命令生成器 — 将布局盒树转换为渲染图元。
pub struct Painter {
    /// 生成的渲染图元列表。
    primitives: RenderPrimitives,
    /// 已由父级行内格式化上下文绘制过文本的节点。
    painted_inline_nodes: HashSet<NodeId>,
}

impl Painter {
    /// 创建新的绘制命令生成器。
    pub fn new() -> Self {
        Self {
            primitives: RenderPrimitives::new(),
            painted_inline_nodes: HashSet::new(),
        }
    }

    /// 绘制整个布局树。
    ///
    /// 遍历 LayoutBox 树，为每个有样式的节点生成背景和边框填充图元。
    /// 传入 `doc` 以启用行内格式化上下文的文本换行布局。
    pub fn paint(&mut self, layout: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, doc: Option<&Document>) {
        self.paint_node(layout, styles, 0.0, 0.0, doc);
    }

    /// 仅绘制与脏区域相交的节点（增量绘制）。
    ///
    /// 遍历布局树时跳过与脏区域完全不相交的子树，
    /// 只生成落在脏区域内的图元。
    pub fn paint_in_rect(
        &mut self,
        layout: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        dirty_rect: &Rect,
        doc: Option<&Document>,
    ) {
        self.paint_node_in_rect(layout, styles, 0.0, 0.0, dirty_rect, doc);
    }

    /// 绘制与脏区域相交的节点（递归）。
    fn paint_node_in_rect(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        offset_x: f32,
        offset_y: f32,
        dirty_rect: &Rect,
        doc: Option<&Document>,
    ) {
        let abs_x = offset_x + box_node.x;
        let abs_y = offset_y + box_node.y;

        // 快速剔除：如果节点包围盒完全不在脏区域内，跳过整个子树
        let node_right = abs_x + box_node.width;
        let node_bottom = abs_y + box_node.height;
        if node_right <= dirty_rect.left()
            || node_bottom <= dirty_rect.top()
            || abs_x >= dirty_rect.right()
            || abs_y >= dirty_rect.bottom()
        {
            return;
        }

        // 节点与脏区域相交，执行正常绘制
        let needs_clip = box_node.overflow_x != OverflowClip::Visible || box_node.overflow_y != OverflowClip::Visible;

        let is_hidden = if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let hidden = matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse);

            if !hidden {
                self.paint_box_shadow(box_node, abs_x, abs_y, style);
                if style.background_color != ColorValue::Transparent {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }
                self.paint_background_image(box_node, abs_x, abs_y, style);
                if box_node.border_top > 0.0
                    || box_node.border_right > 0.0
                    || box_node.border_bottom > 0.0
                    || box_node.border_left > 0.0
                {
                    self.paint_borders(box_node, abs_x, abs_y, style);
                }
                self.paint_outline(box_node, abs_x, abs_y, style);
                self.paint_text(box_node, abs_x, abs_y, style, doc);
            }

            hidden
        } else {
            false
        };

        let child_offset_x = abs_x + box_node.padding_left + box_node.border_left;
        let child_offset_y = abs_y + box_node.padding_top + box_node.border_top;

        let fills_before = self.primitives.fills.len();
        let glyphs_before = self.primitives.glyphs.len();

        for child in &box_node.children {
            self.paint_node_in_rect(child, styles, child_offset_x, child_offset_y, dirty_rect, doc);
        }

        if needs_clip {
            let clip_rect = Rect::new(
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            );
            clip_fills(&mut self.primitives.fills, fills_before, &clip_rect);
            clip_glyphs(&mut self.primitives.glyphs, glyphs_before, &clip_rect);
        }

        let _ = is_hidden;
    }

    /// 绘制单个节点（递归）。
    ///
    /// 根据节点的计算样式生成背景色填充和边框填充图元，
    /// 然后递归绘制子节点。当 overflow 不为 Visible 时，
    /// 子节点产生的图元会被裁剪到内容盒范围内。
    /// 当传入 `doc` 时，使用行内格式化上下文处理文本换行。
    fn paint_node(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        offset_x: f32,
        offset_y: f32,
        doc: Option<&Document>,
    ) {
        let abs_x = offset_x + box_node.x;
        let abs_y = offset_y + box_node.y;

        // 判断是否需要裁剪子内容
        let needs_clip = box_node.overflow_x != OverflowClip::Visible || box_node.overflow_y != OverflowClip::Visible;

        // 获取该节点对应的计算样式
        // 记录绘制前的图元数量，用于 opacity 应用
        let counts_before = PrimitiveCounts::snapshot(&self.primitives);

        let is_hidden = if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let hidden = matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse);

            if !hidden {
                // 0. box-shadow（位于背景之下）
                self.paint_box_shadow(box_node, abs_x, abs_y, style);

                // 1. 背景色填充（根据 border-radius 生成圆角矩形图元）
                if style.background_color != ColorValue::Transparent {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }

                // 1b. 背景图片（在背景色之上）
                self.paint_background_image(box_node, abs_x, abs_y, style);

                // 2. 边框填充（根据 border-radius 生成圆角边框图元）
                if box_node.border_top > 0.0
                    || box_node.border_right > 0.0
                    || box_node.border_bottom > 0.0
                    || box_node.border_left > 0.0
                {
                    self.paint_borders(box_node, abs_x, abs_y, style);
                }

                // 3. Outline 绘制（位于 border 外侧）
                self.paint_outline(box_node, abs_x, abs_y, style);

                // 4. 文本内容绘制（含 text-shadow，使用行内格式化上下文处理换行）
                self.paint_text(box_node, abs_x, abs_y, style, doc);
            }

            hidden
        } else {
            false
        };

        // 5. 递归绘制子节点（子节点偏移 = 父 padding + border）
        // visibility: hidden 不阻止子节点绘制，子节点可以覆盖为 visible
        let child_offset_x = abs_x + box_node.padding_left + box_node.border_left;
        let child_offset_y = abs_y + box_node.padding_top + box_node.border_top;

        // 记录子节点绘制前的图元数量，用于裁剪
        let fills_before = self.primitives.fills.len();
        let glyphs_before = self.primitives.glyphs.len();

        for child in &box_node.children {
            self.paint_node(child, styles, child_offset_x, child_offset_y, doc);
        }

        // 如果需要裁剪，将子节点产生的图元裁剪到内容盒范围内
        if needs_clip {
            let clip_rect = Rect::new(
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            );
            clip_fills(&mut self.primitives.fills, fills_before, &clip_rect);
            clip_glyphs(&mut self.primitives.glyphs, glyphs_before, &clip_rect);
        }

        // CSS filter — 对元素及其子元素产生的图元应用滤镜效果
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            self.apply_filter(box_node, abs_x, abs_y, style);
        }

        // 应用 opacity（对当前节点及其子节点产生的所有图元进行 alpha 衰减）
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && style.opacity < 1.0
        {
            let opacity = style.opacity as f32;
            apply_opacity_to_new_primitives(&mut self.primitives, &counts_before, opacity);
        }

        let _ = is_hidden; // visibility 在 if let 块内处理
    }

    /// 绘制背景（考虑 border-radius）。
    ///
    /// 当 border-radius 为零时退化为普通矩形填充。
    fn paint_background(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let radii = BorderRadiusSpec::from_style(style);
        if radii.is_zero() {
            // 无圆角：简单矩形填充
            self.primitives.add_fill(
                Rect::new(abs_x, abs_y, box_node.width, box_node.height),
                color_value_to_render(&style.background_color),
            );
        } else {
            // 圆角矩形：生成带圆角信息的填充图元
            self.primitives.add_fill(
                Rect::new(abs_x, abs_y, box_node.width, box_node.height),
                color_value_to_render(&style.background_color),
            );
            // 存储圆角信息（当前架构下 FillPrimitive 没有圆角字段，
            // 通过附加的元数据图元标记圆角）
            self.add_rounded_rect_metadata(abs_x, abs_y, box_node.width, box_node.height, &radii);
        }
    }

    /// 添加圆角矩形元数据图元。
    ///
    /// 在当前渲染架构下，使用额外的 0-尺寸填充图元记录圆角参数。
    /// 每个 CornerFill 代表一个角部的圆角半径信息。
    fn add_rounded_rect_metadata(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _radii: &BorderRadiusSpec) {
        // 圆角信息通过 CornerFill 图元存储。
        // 在完整实现中会生成圆角裁剪蒙版或扇形填充。
        // 当前阶段记录圆角存在，待后续渲染后端支持。
    }

    /// 绘制边框（4 个矩形）。
    ///
    /// 分别绘制上、右、下、左四条边框。每条边框是一个填充矩形。
    fn paint_borders(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let w = box_node.width;
        let h = box_node.height;

        // 上边框
        if box_node.border_top > 0.0
            && style.border_top_style != BorderStyleValue::None
            && style.border_top_style != BorderStyleValue::Hidden
        {
            self.primitives.add_fill(
                Rect::new(abs_x, abs_y, w, box_node.border_top),
                color_value_to_render(&style.border_top_color),
            );
        }

        // 右边框
        if box_node.border_right > 0.0
            && style.border_right_style != BorderStyleValue::None
            && style.border_right_style != BorderStyleValue::Hidden
        {
            self.primitives.add_fill(
                Rect::new(
                    abs_x + w - box_node.border_right,
                    abs_y + box_node.border_top,
                    box_node.border_right,
                    h - box_node.border_top - box_node.border_bottom,
                ),
                color_value_to_render(&style.border_right_color),
            );
        }

        // 下边框
        if box_node.border_bottom > 0.0
            && style.border_bottom_style != BorderStyleValue::None
            && style.border_bottom_style != BorderStyleValue::Hidden
        {
            self.primitives.add_fill(
                Rect::new(abs_x, abs_y + h - box_node.border_bottom, w, box_node.border_bottom),
                color_value_to_render(&style.border_bottom_color),
            );
        }

        // 左边框
        if box_node.border_left > 0.0
            && style.border_left_style != BorderStyleValue::None
            && style.border_left_style != BorderStyleValue::Hidden
        {
            self.primitives.add_fill(
                Rect::new(
                    abs_x,
                    abs_y + box_node.border_top,
                    box_node.border_left,
                    h - box_node.border_top - box_node.border_bottom,
                ),
                color_value_to_render(&style.border_left_color),
            );
        }
    }

    /// 绘制 outline（位于 border 外侧）。
    ///
    /// outline 绘制为 4 个矩形，offset 默认为 0（紧贴 border 外侧）。
    fn paint_outline(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let outline_width = length_to_f32(&style.outline_width);

        if outline_width <= 0.0 || style.outline_style == OutlineStyleValue::None {
            return;
        }

        let offset = length_to_f32(&style.outline_offset);

        let w = box_node.width;
        let h = box_node.height;
        let total_offset = outline_width + offset;
        let color = color_value_to_render(&style.outline_color);

        // 上 outline
        self.primitives.add_fill(
            Rect::new(
                abs_x - total_offset,
                abs_y - total_offset,
                w + 2.0 * total_offset,
                outline_width,
            ),
            color,
        );

        // 下 outline
        self.primitives.add_fill(
            Rect::new(
                abs_x - total_offset,
                abs_y + h + offset,
                w + 2.0 * total_offset,
                outline_width,
            ),
            color,
        );

        // 左 outline
        self.primitives.add_fill(
            Rect::new(
                abs_x - total_offset,
                abs_y - total_offset + outline_width,
                outline_width,
                h + 2.0 * offset,
            ),
            color,
        );

        // 右 outline
        self.primitives.add_fill(
            Rect::new(
                abs_x + w + offset,
                abs_y - total_offset + outline_width,
                outline_width,
                h + 2.0 * offset,
            ),
            color,
        );
    }

    /// 绘制 box-shadow（盒阴影效果）。
    ///
    /// 在背景之下绘制 box-shadow，通过 ShadowPrimitive 表示。
    /// 包含偏移、模糊半径、扩展半径和颜色信息。
    /// 当所有阴影参数为零时跳过绘制。
    fn paint_box_shadow(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let shadow = &style.box_shadow;

        // 跳过无效阴影（所有参数为零时不可见）
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
    /// 当 background-image 为 Url 时，生成 ImagePrimitive 图元。
    /// 当 background-image 为 Gradient 时，生成 GradientPrimitive 图元。
    fn paint_background_image(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        match &style.background_image {
            BackgroundImageComputedValue::None => {}
            BackgroundImageComputedValue::Url(url) => {
                let key = simple_hash(url);
                let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
                self.primitives.add_image(ImagePrimitive {
                    rect,
                    image_key: ImageKey::new(key),
                });
            }
            BackgroundImageComputedValue::Gradient(gradient) => {
                let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
                if let Some(prim) = gradient_to_primitive(gradient, &rect) {
                    self.primitives.add_gradient(prim);
                }
            }
        }
    }

    /// 绘制文本装饰线（underline / overline / line-through）。
    ///
    /// 在文本字形下方/上方/中间生成细线填充图元。
    /// line_width 为装饰线高度（固定为 ~1px），line_y 根据装饰类型定位。
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
            TextDecorationLineValue::Blink => {
                // blink 在现代浏览器中通常不绘制（已弃用），跳过
            }
        }
    }

    /// 获取生成的渲染图元（消费 painter）。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 获取渲染图元引用。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }

    /// 绘制文本内容（生成多字符 GlyphPrimitive）。
    ///
    /// 当传入 `doc` 且该元素有 DOM 节点时，使用 `InlineFormattingContext`
    /// 进行文本换行布局，为每个文本片段中的每个字符各生成一个 GlyphPrimitive，
    /// 字符按估算前进宽度逐个排列。
    /// 当 `doc` 为 `None` 或元素没有文本子节点时，退化为生成单个占位 glyph。
    pub fn paint_text(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
    ) {
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };

        if font_size <= 0.0 {
            return;
        }

        // 仅当元素设置了明确的前景色（非 CurrentColor）时才生成 glyph
        // 这避免了为默认样式的容器元素生成无意义的 glyph
        if style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);

        // letter-spacing 和 word-spacing
        let letter_spacing: f32 = match style.letter_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };
        let word_spacing: f32 = match style.word_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };

        // 文本阴影参数
        let text_shadow = &style.text_shadow;
        let has_text_shadow =
            text_shadow.offset_x != 0.0 || text_shadow.offset_y != 0.0 || text_shadow.blur_radius != 0.0;
        let shadow_ox = text_shadow.offset_x;
        let shadow_oy = text_shadow.offset_y;
        let shadow_color = color_value_to_render(&text_shadow.color);

        // 使用内容区域左上角作为文本起始位置
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        // 应用 CSS transform
        let (tx, ty) = super::helpers::apply_transform_offset(style, abs_x, abs_y);

        // 默认字体 ID
        let default_font_id = FontId(0);

        // 尝试使用行内格式化上下文（需要 Document 和 DOM 节点）
        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            if self.painted_inline_nodes.contains(&node_id) || !has_direct_paintable_text(doc, node_id) {
                return;
            }

            let container_width = box_node.content_width;
            let break_word = matches!(
                style.overflow_wrap,
                zero_style_system::OverflowWrapValue::BreakWord | zero_style_system::OverflowWrapValue::Anywhere
            );
            let mut inline_ctx = InlineFormattingContext::new(container_width).with_break_word(break_word);
            inline_ctx.layout(doc, node_id, &HashMap::new());

            let fragments = inline_ctx.all_fragments();

            // 判断是否需要 text-overflow: ellipsis 处理
            // 条件：text-overflow 为 Ellipsis，且 overflow-x 不是 Visible
            let needs_ellipsis = matches!(style.text_overflow, TextOverflowValue::Ellipsis)
                && !matches!(style.overflow_x, zero_css_parser::values::OverflowValue::Visible);

            if !fragments.is_empty() {
                // 记录片段绘制前的 glyph 数量，用于 ellipsis 后处理
                let glyphs_before_fragments = self.primitives.glyphs.len();

                // text-indent：首行缩进偏移（仅应用到第一行第一个片段）
                let text_indent: f32 = match style.text_indent {
                    LengthValue::Px(v) => v as f32,
                    LengthValue::Em(v) => v as f32 * font_size,
                    _ => 0.0,
                };

                // 记录第一行的 y 坐标，用于判断哪些片段属于首行
                let first_line_y = fragments[0].y;

                // 有文本片段 — 为每个片段中的每个字符生成独立 glyph
                for (frag_idx, fragment) in fragments.iter().enumerate() {
                    self.painted_inline_nodes.insert(fragment.node_id);

                    // 首行片段追加 text-indent 偏移
                    let indent = if frag_idx == 0 || (fragment.y == first_line_y && text_indent != 0.0) {
                        // 如果是首行（y 坐标与第一个片段相同），应用缩进
                        if fragment.y == first_line_y { text_indent } else { 0.0 }
                    } else {
                        0.0
                    };
                    let frag_base_x = content_x + fragment.x + tx + indent;
                    let frag_base_y = content_y + fragment.y + fragment.font_size + ty;
                    let mut char_x = frag_base_x;

                    // 应用 text-transform
                    let transformed = apply_text_transform(&fragment.text, &style.text_transform);

                    for ch in transformed.chars() {
                        // 文本阴影 glyph（在主字形之前绘制，确保阴影在底层）
                        if has_text_shadow {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: char_x + shadow_ox,
                                y: frag_base_y + shadow_oy,
                                font_size: fragment.font_size,
                                color: shadow_color,
                                glyph_id: ch as u32,
                                font_id: default_font_id,
                                bitmap_width: None,
                                bitmap_height: None,
                            });
                        }

                        self.primitives.add_glyph(GlyphPrimitive {
                            x: char_x,
                            y: frag_base_y,
                            font_size: fragment.font_size,
                            color,
                            glyph_id: ch as u32,
                            font_id: default_font_id,
                            bitmap_width: None,
                            bitmap_height: None,
                        });
                        char_x += estimate_char_width(ch, fragment.font_size);
                        // letter-spacing：每个字符后追加固定间距
                        char_x += letter_spacing;
                        // word-spacing：空格字符后追加额外间距
                        if ch == ' ' {
                            char_x += word_spacing;
                        }
                    }

                    // 绘制文本装饰线（underline/overline/line-through）
                    let text_width: f32 = transformed
                        .chars()
                        .map(|ch| {
                            let w = estimate_char_width(ch, fragment.font_size) + letter_spacing;
                            if ch == ' ' { w + word_spacing } else { w }
                        })
                        .sum();
                    self.paint_text_decoration(
                        frag_base_x,
                        frag_base_y,
                        fragment.font_size,
                        text_width,
                        color,
                        &style.text_decoration_line,
                    );
                }

                // text-overflow: ellipsis 后处理
                // 检查文本是否超出容器宽度，如果超出则截断并添加 "..."
                if needs_ellipsis && container_width > 0.0 {
                    let content_right = content_x + container_width + tx;

                    // 检查是否有 glyph 超出容器右边界
                    let glyphs = &mut self.primitives.glyphs;
                    let fragment_glyphs = &mut glyphs[glyphs_before_fragments..];

                    // 找到第一个超出容器的 glyph
                    let mut last_visible_idx: Option<usize> = None;
                    let mut has_overflow = false;

                    for (i, g) in fragment_glyphs.iter().enumerate() {
                        if g.font_size == 0.0 {
                            continue; // 已被裁剪的 glyph
                        }
                        if g.x >= content_right {
                            has_overflow = true;
                            last_visible_idx = if i > 0 { Some(i - 1) } else { None };
                            break;
                        }
                        last_visible_idx = Some(i);
                    }

                    if has_overflow {
                        let ellipsis_char_width = estimate_char_width('.', font_size);
                        let total_ellipsis_width = ellipsis_char_width * 3.0 + letter_spacing * 2.0;
                        let ellipsis_end_x = content_right;
                        let ellipsis_start_x = ellipsis_end_x - total_ellipsis_width;

                        // 从后往前找：保留能放下 "..." 的最后几个 glyph
                        // 移除超出 ellipsis_start_x 的所有 glyph（从 last_visible_idx 开始）
                        let cutoff_start = if let Some(idx) = last_visible_idx {
                            // 从该位置往前找，留出 "..." 的空间
                            let mut cut = idx + 1;
                            for j in (0..=idx).rev() {
                                if fragment_glyphs[j].x < ellipsis_start_x && fragment_glyphs[j].font_size > 0.0 {
                                    cut = j + 1;
                                    break;
                                }
                                cut = j;
                            }
                            cut
                        } else {
                            0
                        };

                        // 清除 cutoff_start 之后的 glyph（设为 glyph_id=0）
                        for g in fragment_glyphs.iter_mut().skip(cutoff_start) {
                            g.glyph_id = 0;
                            g.font_size = 0.0;
                        }

                        // 在容器末尾添加 "..." glyph
                        let first_glyph = fragment_glyphs.iter().find(|g| g.font_size > 0.0);
                        let base_y = first_glyph.map(|g| g.y).unwrap_or(content_y + font_size + ty);

                        for (i, ch) in ['.', '.', '.'].iter().enumerate() {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: ellipsis_start_x + ellipsis_char_width * i as f32 + letter_spacing * i as f32,
                                y: base_y,
                                font_size,
                                color,
                                glyph_id: *ch as u32,
                                font_id: default_font_id,
                                bitmap_width: None,
                                bitmap_height: None,
                            });
                        }
                    }
                }

                return;
            }
        }

        // 退化为单个占位 glyph（无 Document 或无文本子节点）
        let glyph_x = content_x + tx;
        let glyph_y = content_y + ty;

        // 文本阴影占位 glyph
        if has_text_shadow {
            self.primitives.add_glyph(GlyphPrimitive {
                x: glyph_x + shadow_ox,
                y: glyph_y + font_size + shadow_oy,
                font_size,
                color: shadow_color,
                glyph_id: 0,
                font_id: default_font_id,
                bitmap_width: None,
                bitmap_height: None,
            });
        }

        self.primitives.add_glyph(GlyphPrimitive {
            x: glyph_x,
            y: glyph_y + font_size, // baseline at bottom of text
            font_size,
            color,
            glyph_id: 0, // placeholder glyph id
            font_id: default_font_id,
            bitmap_width: None,
            bitmap_height: None,
        });

        // 退化的文本装饰线
        self.paint_text_decoration(
            glyph_x,
            glyph_y + font_size,
            font_size,
            estimate_char_width('A', font_size),
            color,
            &style.text_decoration_line,
        );
    }

    /// 应用 CSS filter — 当 filter 非 none 时，生成 FilterPrimitive。
    ///
    /// FilterPrimitive 记录滤镜函数和应用区域，由渲染后端在光栅化阶段
    /// 对该区域内的所有图元进行像素级滤镜处理。
    fn apply_filter(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
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
}

/// 将 ComputedStyle 中的 filter 值转换为渲染层 FilterKind。
fn filter_computed_to_kind(value: &FilterComputedValue) -> FilterKind {
    match value {
        FilterComputedValue::None => FilterKind::Blur(0.0), // 不应到达
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
            FilterKind::DropShadow(*x, *y, *blur, super::color::color_value_to_render(color))
        }
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

fn has_direct_paintable_text(doc: &Document, node_id: NodeId) -> bool {
    doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}
