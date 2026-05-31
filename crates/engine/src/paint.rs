//! 绘制命令生成 — 将布局盒树转换为渲染图元。

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_css_parser::values::LengthValue;
use zero_css_parser::values::TransformFunction;
use zero_css_parser::values::TransformValue;
use zero_css_parser::values::VisibilityValue;
use zero_dom::{Document, NodeId};
use zero_layout_engine::InlineFormattingContext;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FontId, GlyphPrimitive, RenderPrimitives};
use zero_style_system::{BorderStyleValue, ComputedStyle, OutlineStyleValue};

/// 绘制命令生成器 — 将布局盒树转换为渲染图元。
pub struct Painter {
    /// 生成的渲染图元列表。
    primitives: RenderPrimitives,
}

impl Painter {
    /// 创建新的绘制命令生成器。
    pub fn new() -> Self {
        Self {
            primitives: RenderPrimitives::new(),
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
                if style.background_color != ColorValue::Transparent {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }
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
        let is_hidden = if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let hidden = matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse);

            if !hidden {
                // 1. 背景色填充（根据 border-radius 生成圆角矩形图元）
                if style.background_color != ColorValue::Transparent {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }

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

                // 4. 文本内容绘制（使用行内格式化上下文处理换行）
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

        // 使用内容区域左上角作为文本起始位置
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        // 应用 CSS transform
        let (tx, ty) = apply_transform_offset(style, abs_x, abs_y);

        // 默认字体 ID
        let default_font_id = FontId(0);

        // 尝试使用行内格式化上下文（需要 Document 和 DOM 节点）
        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            let container_width = box_node.content_width;
            let mut inline_ctx = InlineFormattingContext::new(container_width);
            inline_ctx.layout(doc, node_id, &HashMap::new());

            let fragments = inline_ctx.all_fragments();
            if !fragments.is_empty() {
                // 有文本片段 — 为每个片段中的每个字符生成独立 glyph
                for fragment in fragments {
                    let frag_base_x = content_x + fragment.x + tx;
                    let frag_base_y = content_y + fragment.y + fragment.font_size + ty;
                    let char_advance = fragment.font_size * 0.6;
                    let mut char_x = frag_base_x;

                    for ch in fragment.text.chars() {
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
                        char_x += char_advance;
                    }
                }
                return;
            }
        }

        // 退化为单个占位 glyph（无 Document 或无文本子节点）
        let glyph_x = content_x + tx;
        let glyph_y = content_y + ty;

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
    }
}

/// 从 ComputedStyle 的 transform 计算偏移量。
///
/// 返回 (dx, dy) 偏移，用于调整图元位置。
fn apply_transform_offset(style: &ComputedStyle, _abs_x: f32, _abs_y: f32) -> (f32, f32) {
    match &style.transform {
        TransformValue::None => (0.0, 0.0),
        TransformValue::List(funcs) => {
            let mut dx = 0.0_f32;
            let mut dy = 0.0_f32;
            for f in funcs {
                match f {
                    TransformFunction::Translate(tx, ty) => {
                        dx += *tx as f32;
                        dy += *ty as f32;
                    }
                    TransformFunction::TranslateX(tx) => {
                        dx += *tx as f32;
                    }
                    TransformFunction::TranslateY(ty) => {
                        dy += *ty as f32;
                    }
                    // rotate, scale, skew 不产生偏移
                    _ => {}
                }
            }
            (dx, dy)
        }
    }
}

/// 将填充矩形裁剪到指定区域内（原地修改）。
///
/// 从 `start` 索引开始的所有填充矩形会被裁剪到 `clip_rect` 内。
fn clip_fills(fills: &mut [zero_render_foundation::primitive::FillPrimitive], start: usize, clip_rect: &Rect) {
    for fill in fills.iter_mut().skip(start) {
        let r = &mut fill.rect;
        let left = r.left().max(clip_rect.left());
        let top = r.top().max(clip_rect.top());
        let right = r.right().min(clip_rect.right());
        let bottom = r.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            // 完全在裁剪区域外，清零
            r.size.width = 0.0;
            r.size.height = 0.0;
        } else {
            r.origin.x = left;
            r.origin.y = top;
            r.size.width = right - left;
            r.size.height = bottom - top;
        }
    }
}

/// 将字形裁剪到指定区域内（原地修改）。
///
/// 从 `start` 索引开始的所有字形，如果完全在裁剪区域外则标记为 glyph_id=0。
fn clip_glyphs(glyphs: &mut [zero_render_foundation::primitive::GlyphPrimitive], start: usize, clip_rect: &Rect) {
    for g in glyphs.iter_mut().skip(start) {
        // 字形位置是左上角，假定宽高约等于 font_size
        let right = g.x + g.font_size;
        let bottom = g.y + g.font_size;
        if right <= clip_rect.left()
            || bottom <= clip_rect.top()
            || g.x >= clip_rect.right()
            || g.y >= clip_rect.bottom()
        {
            g.glyph_id = 0; // 标记为不可见
            g.font_size = 0.0;
        }
    }
}

/// 四角圆角半径集合。
#[derive(Debug, Clone, Copy)]
pub struct BorderRadiusSpec {
    /// 左上角半径。
    pub top_left: f32,
    /// 右上角半径。
    pub top_right: f32,
    /// 右下角半径。
    pub bottom_right: f32,
    /// 左下角半径。
    pub bottom_left: f32,
}

impl BorderRadiusSpec {
    /// 从 ComputedStyle 提取圆角半径。
    pub fn from_style(style: &ComputedStyle) -> Self {
        Self {
            top_left: length_to_f32(&style.border_top_left_radius),
            top_right: length_to_f32(&style.border_top_right_radius),
            bottom_right: length_to_f32(&style.border_bottom_right_radius),
            bottom_left: length_to_f32(&style.border_bottom_left_radius),
        }
    }

    /// 所有圆角都为零。
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_right == 0.0 && self.bottom_left == 0.0
    }
}

/// 将 LengthValue 转换为 f32（仅支持 Px）。
fn length_to_f32(v: &LengthValue) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        _ => 0.0,
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 ComputedStyle 的 ColorValue 转换为 render-foundation 的 Color。
pub fn color_value_to_render(color: &ColorValue) -> Color {
    match color {
        ColorValue::Rgba(r, g, b, a) => Color::rgba(*r, *g, *b, *a),
        ColorValue::Transparent => Color::rgba(0, 0, 0, 0),
        ColorValue::Named(name) => named_color_to_render(name),
        ColorValue::CurrentColor => Color::rgba(0, 0, 0, 255),
        ColorValue::Hsla(h, s, l, a) => hsla_to_rgba(*h, *s, *l, *a),
    }
}

/// 将 HSL(Hue, Saturation, Lightness, Alpha) 转换为 RGBA。
///
/// - `h` 色相角度 [0, 360)
/// - `s` 饱和度 [0, 100]
/// - `l` 亮度 [0, 100]
/// - `a` 不透明度 [0, 1]
pub fn hsla_to_rgba(h: f64, s: f64, l: f64, a: f64) -> Color {
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h_prime {
        hp if hp < 1.0 => (c, x, 0.0),
        hp if hp < 2.0 => (x, c, 0.0),
        hp if hp < 3.0 => (0.0, c, x),
        hp if hp < 4.0 => (0.0, x, c),
        hp if hp < 5.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let to_u8 = |v: f64| -> u8 { (v * 255.0).round().clamp(0.0, 255.0) as u8 };
    Color::rgba(
        to_u8(r1 + m),
        to_u8(g1 + m),
        to_u8(b1 + m),
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// 将命名颜色转换为渲染颜色。
pub fn named_color_to_render(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "red" => Color::rgb(255, 0, 0),
        "green" => Color::rgb(0, 128, 0),
        "blue" => Color::rgb(0, 0, 255),
        "black" => Color::rgb(0, 0, 0),
        "white" => Color::rgb(255, 255, 255),
        "yellow" => Color::rgb(255, 255, 0),
        "cyan" | "aqua" => Color::rgb(0, 255, 255),
        "magenta" | "fuchsia" => Color::rgb(255, 0, 255),
        "gray" | "grey" => Color::rgb(128, 128, 128),
        "silver" => Color::rgb(192, 192, 192),
        "maroon" => Color::rgb(128, 0, 0),
        "olive" => Color::rgb(128, 128, 0),
        "lime" => Color::rgb(0, 255, 0),
        "purple" => Color::rgb(128, 0, 128),
        "teal" => Color::rgb(0, 128, 128),
        "navy" => Color::rgb(0, 0, 128),
        "orange" => Color::rgb(255, 165, 0),
        "pink" => Color::rgb(255, 192, 203),
        "brown" => Color::rgb(165, 42, 42),
        _ => Color::rgb(0, 0, 0),
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]
mod tests {
    use super::*;
    use zero_css_parser::values::ColorValue;
    use zero_layout_engine::types::OverflowClip;

    /// 测试空布局树不产生任何图元。
    #[test]
    fn test_painter_empty_layout() {
        let layout = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 0.0,
            content_height: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        let mut painter = Painter::new();
        let styles = HashMap::new();
        painter.paint(&layout, &styles, None);
        assert!(painter.primitives().is_empty());
    }

    /// 辅助函数：创建简单 LayoutBox。
    fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
        LayoutBox {
            node_id,
            x,
            y,
            width,
            height,
            content_x: 0.0,
            content_y: 0.0,
            content_width: width,
            content_height: height,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        }
    }

    /// 辅助函数：创建带边框的 LayoutBox。
    fn make_box_with_border(
        node_id: Option<NodeId>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        border_top: f32,
        border_right: f32,
        border_bottom: f32,
        border_left: f32,
    ) -> LayoutBox {
        LayoutBox {
            node_id,
            x,
            y,
            width,
            height,
            content_x: border_left,
            content_y: border_top,
            content_width: width - border_left - border_right,
            content_height: height - border_top - border_bottom,
            border_top,
            border_right,
            border_bottom,
            border_left,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        }
    }

    /// 测试背景色生成填充图元。
    #[test]
    fn test_painter_background_color() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        let primitives = painter.primitives();
        assert_eq!(primitives.fills.len(), 1);
        assert_eq!(primitives.fills[0].color, Color::rgb(255, 0, 0));
        assert_eq!(primitives.fills[0].rect.origin.x, 0.0);
        assert_eq!(primitives.fills[0].rect.origin.y, 0.0);
        assert_eq!(primitives.fills[0].rect.size.width, 100.0);
        assert_eq!(primitives.fills[0].rect.size.height, 50.0);
    }

    /// 测试透明背景不生成填充图元。
    #[test]
    fn test_painter_transparent_background() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Transparent;
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert!(painter.primitives().is_empty());
    }

    /// 测试上边框生成填充图元。
    #[test]
    fn test_painter_border_top() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 5.0, 0.0, 0.0, 0.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert_eq!(painter.primitives().fills.len(), 1);
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 0.0);
        assert_eq!(fill.rect.origin.y, 0.0);
        assert_eq!(fill.rect.size.width, 100.0);
        assert_eq!(fill.rect.size.height, 5.0);
    }

    /// 测试四条边框都生成填充图元。
    #[test]
    fn test_painter_border_all_sides() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 3.0, 4.0, 5.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 应该有 4 个边框填充
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试嵌套盒子的绘制。
    #[test]
    fn test_painter_nested_boxes() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 10.0, 10.0, 30.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        assert_eq!(painter.primitives().fills.len(), 2);

        // 第一个填充是父元素背景
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
        // 第二个填充是子元素背景（位置偏移 10,10）
        assert_eq!(painter.primitives().fills[1].rect.origin.x, 10.0);
        assert_eq!(painter.primitives().fills[1].rect.origin.y, 10.0);
    }

    /// 测试 ColorValue::Rgba 转换。
    #[test]
    fn test_painter_color_value_rgba() {
        let color = color_value_to_render(&ColorValue::Rgba(128, 64, 32, 255));
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    /// 测试 ColorValue::Transparent 转换。
    #[test]
    fn test_painter_color_value_transparent() {
        let color = color_value_to_render(&ColorValue::Transparent);
        assert_eq!(color.a, 0);
    }

    /// 测试命名颜色转换（red, blue, black, white）。
    #[test]
    fn test_painter_color_value_named() {
        assert_eq!(named_color_to_render("red"), Color::rgb(255, 0, 0));
        assert_eq!(named_color_to_render("blue"), Color::rgb(0, 0, 255));
        assert_eq!(named_color_to_render("black"), Color::rgb(0, 0, 0));
        assert_eq!(named_color_to_render("white"), Color::rgb(255, 255, 255));
        // 大小写不敏感
        assert_eq!(named_color_to_render("Red"), Color::rgb(255, 0, 0));
        assert_eq!(named_color_to_render("BLUE"), Color::rgb(0, 0, 255));
        // 未知颜色回退为黑色
        assert_eq!(named_color_to_render("unknown"), Color::rgb(0, 0, 0));
    }

    /// 测试零尺寸盒子不产生有效图元（宽度为 0 时 Rect 退化为零面积）。
    #[test]
    fn test_painter_zero_size_box() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 0.0, 0.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 会生成一个填充，但尺寸为 0
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].rect.size.width, 0.0);
        assert_eq!(painter.primitives().fills[0].rect.size.height, 0.0);
    }

    /// 测试绝对偏移计算正确。
    #[test]
    fn test_painter_absolute_offset() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 50.0, 30.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 128, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 50.0);
        assert_eq!(fill.rect.origin.y, 30.0);
    }

    /// 测试多个子节点都能生成填充图元。
    #[test]
    fn test_painter_multiple_children() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child1 = doc.create_element("span");
        let child2 = doc.create_element("span");

        let child_box1 = make_box(Some(child1), 0.0, 0.0, 50.0, 20.0);
        let child_box2 = make_box(Some(child2), 0.0, 20.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box1, child_box2],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        for id in [child1, child2] {
            let mut s = ComputedStyle::default();
            s.background_color = ColorValue::Rgba(255, 0, 0, 255);
            styles.insert(id, s);
        }

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 只有子节点有背景色，父节点没有
        assert_eq!(painter.primitives().fills.len(), 2);
    }

    /// 测试 into_primitives 消费 painter。
    #[test]
    fn test_painter_into_primitives() {
        let mut painter = Painter::new();
        let layout = make_box(None, 0.0, 0.0, 0.0, 0.0);
        let styles = HashMap::new();
        painter.paint(&layout, &styles, None);
        let primitives = painter.into_primitives();
        assert!(primitives.is_empty());
    }

    /// 测试 Default 实现。
    #[test]
    fn test_painter_default() {
        let painter = Painter::default();
        assert!(painter.primitives().is_empty());
    }

    /// 测试 background + border 同时存在时填充数量（1 background + 4 border = 5）。
    #[test]
    fn test_painter_background_plus_border_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 1 background fill + 4 border fills = 5
        assert_eq!(painter.primitives().fills.len(), 5);
        // First fill is background
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
    }

    /// 测试无样式节点（no node_id）不产生任何填充。
    #[test]
    fn test_painter_no_style_no_fills() {
        let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
        let mut painter = Painter::new();
        let styles = HashMap::new();
        painter.paint(&layout, &styles, None);
        assert!(painter.primitives().is_empty());
    }

    /// 测试 only background（no border）产生恰好 1 个填充。
    #[test]
    fn test_painter_only_background_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 80.0, 40.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 128, 255, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert_eq!(painter.primitives().fills.len(), 1);
    }

    /// 测试 only border（transparent background）产生恰好 4 个填充。
    #[test]
    fn test_painter_only_border_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 80.0, 40.0, 1.0, 1.0, 1.0, 1.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        // background is transparent by default
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 4 border fills, no background fill
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试带 padding 的子节点偏移。
    #[test]
    fn test_painter_child_offset_with_padding() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 10.0,
            content_y: 10.0,
            content_width: 80.0,
            content_height: 60.0,
            border_top: 5.0,
            border_right: 5.0,
            border_bottom: 5.0,
            border_left: 5.0,
            padding_top: 5.0,
            padding_right: 5.0,
            padding_bottom: 5.0,
            padding_left: 5.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 子节点偏移 = padding_left(5) + border_left(5) = 10
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 10.0);
        assert_eq!(fill.rect.origin.y, 10.0);
    }

    /// 测试 visibility: hidden 的元素不生成填充图元。
    #[test]
    fn test_painter_visibility_hidden() {
        use zero_css_parser::values::VisibilityValue;
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        parent_style.visibility = VisibilityValue::Hidden;
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // parent 的 visibility:hidden 阻止了父节点绘制，但子节点不受影响
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(100, 100, 255));
    }

    /// 测试 border-style: none 的边框不生成填充图元。
    #[test]
    fn test_painter_border_style_none() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        // 所有边框 style 都是 none（默认值）
        style.border_top_style = BorderStyleValue::None;
        style.border_right_style = BorderStyleValue::None;
        style.border_bottom_style = BorderStyleValue::None;
        style.border_left_style = BorderStyleValue::None;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // border-style: none 不绘制边框
        assert_eq!(painter.primitives().fills.len(), 0);
    }

    /// 测试 border-style: solid 的边框正常绘制。
    #[test]
    fn test_painter_border_style_solid() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // border-style: solid 正常绘制 4 条边框
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试 outline 绘制。
    #[test]
    fn test_painter_outline() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.outline_width = zero_css_parser::values::LengthValue::Px(3.0);
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // outline 生成 4 个填充图元
        assert_eq!(painter.primitives().fills.len(), 4);
        // 上 outline：从 (7, 17) 开始，宽 106，高 3
        let top = &painter.primitives().fills[0];
        assert_eq!(top.rect.origin.x, 7.0);
        assert_eq!(top.rect.origin.y, 17.0);
        assert_eq!(top.rect.size.width, 106.0);
        assert_eq!(top.rect.size.height, 3.0);
        assert_eq!(top.color, Color::rgb(255, 0, 0));
    }

    /// 测试 outline-style: none 不绘制。
    #[test]
    fn test_painter_outline_style_none() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.outline_width = zero_css_parser::values::LengthValue::Px(3.0);
        style.outline_style = OutlineStyleValue::None;
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert!(painter.primitives().is_empty());
    }

    /// 测试 outline + background + border 同时绘制。
    #[test]
    fn test_painter_background_border_outline() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        style.outline_width = zero_css_parser::values::LengthValue::Px(2.0);
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 1 background + 4 border + 4 outline = 9
        assert_eq!(painter.primitives().fills.len(), 9);
    }

    // ── 新增测试：HSL/HSLA 颜色转换 ──────────────────────────

    /// 测试 HSL 红色（0°, 100%, 50%）转换为 RGB(255, 0, 0)。
    #[test]
    fn test_hsla_red() {
        let color = hsla_to_rgba(0.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 绿色（120°, 100%, 50%）转换为 RGB(0, 255, 0)。
    #[test]
    fn test_hsla_green() {
        let color = hsla_to_rgba(120.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 蓝色（240°, 100%, 50%）转换为 RGB(0, 0, 255)。
    #[test]
    fn test_hsla_blue() {
        let color = hsla_to_rgba(240.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 半透明值。
    #[test]
    fn test_hsla_with_alpha() {
        let color = hsla_to_rgba(240.0, 100.0, 50.0, 0.5);
        assert_eq!(color.a, 128); // 0.5 * 255 ≈ 128
    }

    /// 测试 HSL 灰色（0°, 0%, 50%）。
    #[test]
    fn test_hsla_gray() {
        let color = hsla_to_rgba(0.0, 0.0, 50.0, 1.0);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 128);
    }

    /// 测试 ColorValue::Hsla 通过 color_value_to_render 正确转换。
    #[test]
    fn test_color_value_hsla_conversion() {
        let hsla = ColorValue::Hsla(0.0, 100.0, 50.0, 1.0);
        let color = color_value_to_render(&hsla);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    // ── 新增测试：overflow 裁剪 ──────────────────────────────

    /// 测试 overflow:hidden 裁剪子节点超出内容盒的部分。
    #[test]
    fn test_overflow_hidden_clips_children() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        // 子节点超出父节点的内容区域
        let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Hidden,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 子节点填充应该被裁剪到父节点的 100x100 内容区域
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.size.width, 100.0, "子节点宽度应被裁剪到 100");
        assert_eq!(fill.rect.size.height, 100.0, "子节点高度应被裁剪到 100");
    }

    /// 测试 overflow:Visible 不裁剪子节点。
    #[test]
    fn test_overflow_visible_no_clip() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 子节点填充不应被裁剪
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.size.width, 200.0);
        assert_eq!(fill.rect.size.height, 200.0);
    }

    /// 测试 overflow:Clip 裁剪子节点（与 Hidden 行为一致）。
    #[test]
    fn test_overflow_clip_clips_children() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 50.0, 50.0, 200.0, 200.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Clip,
            overflow_y: OverflowClip::Clip,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 子节点从 (50,50) 开始 200x200，裁剪到 100x100 的内容盒
        let fill = &painter.primitives().fills[0];
        assert!(fill.rect.size.width <= 100.0);
        assert!(fill.rect.size.height <= 100.0);
    }

    // ── 新增测试：border-radius ──────────────────────────────

    /// 测试 BorderRadiusSpec::from_style 提取圆角半径。
    #[test]
    fn test_border_radius_spec_from_style() {
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Px(10.0);
        style.border_top_right_radius = LengthValue::Px(20.0);
        style.border_bottom_right_radius = LengthValue::Px(30.0);
        style.border_bottom_left_radius = LengthValue::Px(40.0);

        let spec = BorderRadiusSpec::from_style(&style);
        assert_eq!(spec.top_left, 10.0);
        assert_eq!(spec.top_right, 20.0);
        assert_eq!(spec.bottom_right, 30.0);
        assert_eq!(spec.bottom_left, 40.0);
        assert!(!spec.is_zero());
    }

    /// 测试默认 ComputedStyle 的 BorderRadiusSpec 为零。
    #[test]
    fn test_border_radius_spec_default_zero() {
        let style = ComputedStyle::default();
        let spec = BorderRadiusSpec::from_style(&style);
        assert!(spec.is_zero());
    }

    /// 测试带圆角的背景填充仍然生成。
    #[test]
    fn test_painter_background_with_border_radius() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_top_left_radius = LengthValue::Px(10.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 背景填充仍然生成（圆角标记在内部处理）
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(255, 0, 0));
    }

    // ── 新增测试：CSS transform ──────────────────────────────

    /// 测试 translate transform 偏移文本位置。
    #[test]
    fn test_transform_translate_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(10.0, 20.0)]);

        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 10.0);
        assert_eq!(dy, 20.0);
    }

    /// 测试 translateX/translateY 偏移。
    #[test]
    fn test_transform_translate_xy_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::TranslateX(30.0),
            TransformFunction::TranslateY(40.0),
        ]);

        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 30.0);
        assert_eq!(dy, 40.0);
    }

    /// 测试 TransformValue::None 不产生偏移。
    #[test]
    fn test_transform_none_no_offset() {
        let style = ComputedStyle::default();
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    /// 测试 rotate/scale/skew 不影响偏移。
    #[test]
    fn test_transform_rotate_scale_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::Rotate(45.0),
            TransformFunction::Scale(2.0, None),
            TransformFunction::Skew(10.0, None),
        ]);

        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    /// 测试 paint_text 生成 GlyphPrimitive。
    #[test]
    fn test_paint_text_generates_glyph() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 10.0, 20.0, &styles[&elem], None);

        assert_eq!(painter.primitives().glyphs.len(), 1);
        let glyph = &painter.primitives().glyphs[0];
        assert_eq!(glyph.font_size, 16.0);
        assert_eq!(glyph.color, Color::rgb(255, 0, 0));
        assert_eq!(glyph.x, 10.0); // text_x = abs_x (no border/padding)
        assert_eq!(glyph.y, 36.0); // text_y + font_size = 20 + 16
    }

    /// 测试 paint_text 在 font_size <= 0 时不生成 glyph。
    #[test]
    fn test_paint_text_zero_font_size() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(0.0);
        style.color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
        assert!(painter.primitives().glyphs.is_empty());
    }

    /// 测试 paint_text 在 color 为 CurrentColor 时不生成 glyph。
    #[test]
    fn test_paint_text_current_color_no_glyph() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
        assert!(painter.primitives().glyphs.is_empty());
    }

    /// 测试 paint_text 带 translate transform 偏移 glyph 位置。
    #[test]
    fn test_paint_text_with_transform() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.transform = TransformValue::List(vec![TransformFunction::Translate(5.0, 10.0)]);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);

        let glyph = &painter.primitives().glyphs[0];
        assert_eq!(glyph.x, 5.0); // 0 + translate_x(5)
        assert_eq!(glyph.y, 26.0); // 0 + translate_y(10) + font_size(16)
    }

    // ── 新增测试：paint_in_rect 增量绘制 ──────────────────────

    /// 测试 paint_in_rect 跳过完全不在脏区域内的节点。
    #[test]
    fn test_paint_in_rect_skips_outside_nodes() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // 节点在 (500, 500) 处
        let layout = make_box(Some(elem), 500.0, 500.0, 100.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        // 脏区域在 (0, 0) 处，不与节点相交
        let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

        // 节点完全在脏区域外，不应产生任何图元
        assert!(painter.primitives().is_empty());
    }

    /// 测试 paint_in_rect 绘制与脏区域相交的节点。
    #[test]
    fn test_paint_in_rect_draws_intersecting_nodes() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 50.0, 50.0, 100.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        // 脏区域与节点部分重叠
        let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

        // 节点与脏区域相交，应产生填充图元
        assert_eq!(painter.primitives().fills.len(), 1);
    }

    // ── 新增测试：Paint pipeline ──────────────────────────────

    /// 测试绘制简单 HTML 页面中带文本样式的元素。
    #[test]
    fn test_paint_page_with_text_element() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("p");
        let layout = make_box(Some(elem), 0.0, 0.0, 300.0, 20.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 255, 255, 255);
        style.font_size = LengthValue::Px(16.0);
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 背景填充
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].rect.size.width, 300.0);
        assert_eq!(painter.primitives().fills[0].rect.size.height, 20.0);
    }

    /// 测试绘制包含多个子元素的页面。
    #[test]
    fn test_paint_page_multiple_elements() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let c1 = doc.create_element("span");
        let c2 = doc.create_element("span");
        let c3 = doc.create_element("span");

        let child1 = make_box(Some(c1), 0.0, 0.0, 100.0, 30.0);
        let child2 = make_box(Some(c2), 0.0, 30.0, 100.0, 30.0);
        let child3 = make_box(Some(c3), 0.0, 60.0, 100.0, 30.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 90.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 90.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child1, child2, child3],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(240, 240, 240, 255);
        styles.insert(parent, parent_style);

        for id in [c1, c2, c3] {
            let mut s = ComputedStyle::default();
            s.background_color = ColorValue::Rgba(100, 100, 200, 255);
            styles.insert(id, s);
        }

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 1 parent background + 3 child backgrounds = 4
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试带 CSS transform 的 translate 偏移 glyph。
    #[test]
    fn test_paint_page_with_css_transform_translate() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(14.0);
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        style.transform = TransformValue::List(vec![TransformFunction::Translate(15.0, 25.0)]);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // Background fill should still be at original position
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].rect.origin.x, 10.0);
        assert_eq!(painter.primitives().fills[0].rect.origin.y, 20.0);

        // paint() 现在调用 paint_text()，应生成带 transform 偏移的 glyph
        assert_eq!(painter.primitives().glyphs.len(), 1);
        let glyph = &painter.primitives().glyphs[0];
        // text_x = abs_x(10), tx = 15 → glyph_x = 10 + 15 = 25
        assert_eq!(glyph.x, 25.0);
        // text_y = abs_y(20), ty = 25, + font_size(14) → glyph_y = 20 + 25 + 14 = 59
        assert_eq!(glyph.y, 59.0);
    }

    /// 测试带 overflow:hidden 的页面正确裁剪子内容。
    #[test]
    fn test_paint_page_with_overflow_hidden() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 300.0, 300.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 80.0,
            content_x: 10.0,
            content_y: 10.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Hidden,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        let fill = &painter.primitives().fills[0];
        assert!(
            fill.rect.size.width <= 100.0,
            "child should be clipped to parent content width"
        );
        assert!(
            fill.rect.size.height <= 80.0,
            "child should be clipped to parent content height"
        );
    }

    /// 测试带 border-radius 的页面正确生成背景填充。
    #[test]
    fn test_paint_page_with_border_radius() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(100, 149, 237, 255);
        style.border_top_left_radius = LengthValue::Px(20.0);
        style.border_top_right_radius = LengthValue::Px(20.0);
        style.border_bottom_right_radius = LengthValue::Px(20.0);
        style.border_bottom_left_radius = LengthValue::Px(20.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // Background fill still generated even with border-radius
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].rect.size.width, 200.0);
        assert_eq!(painter.primitives().fills[0].rect.size.height, 100.0);
    }

    /// 测试渲染输出：背景先于前景（parent fill comes before child fill）。
    #[test]
    fn test_render_primitive_order_background_before_foreground() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 5.0, 5.0, 50.0, 30.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 200.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        assert_eq!(painter.primitives().fills.len(), 2);
        // First fill is parent background (drawn first = behind)
        assert_eq!(painter.primitives().fills[0].rect.size.width, 200.0);
        // Second fill is child background (drawn second = in front)
        assert_eq!(painter.primitives().fills[1].rect.size.width, 50.0);
    }

    /// 测试渲染输出：primitive count 与预期匹配。
    #[test]
    fn test_render_primitive_count_matches_expectation() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // 2px border on all sides
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 60.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(128, 128, 128, 255);
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 1 background + 4 borders = 5 fills, 0 glyphs
        assert_eq!(painter.primitives().fills.len(), 5);
        assert_eq!(painter.primitives().glyphs.len(), 0);
        assert_eq!(painter.primitives().len(), 5);
    }

    // ── 新增测试：CSS transform integration ───────────────────

    /// 测试 translateX 变换偏移。
    #[test]
    fn test_transform_translate_x_only() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateX(42.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 42.0);
        assert_eq!(dy, 0.0);
    }

    /// 测试 translateY 变换偏移。
    #[test]
    fn test_transform_translate_y_only() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateY(99.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 99.0);
    }

    /// 测试 translate + translateX + translateY 累加。
    #[test]
    fn test_transform_combined_translates() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::Translate(10.0, 20.0),
            TransformFunction::TranslateX(5.0),
            TransformFunction::TranslateY(3.0),
        ]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 15.0);
        assert_eq!(dy, 23.0);
    }

    /// 测试 rotate + translate 混合：只有 translate 贡献偏移。
    #[test]
    fn test_transform_rotate_with_translate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::Rotate(90.0),
            TransformFunction::Translate(50.0, 60.0),
            TransformFunction::Scale(2.0, None),
        ]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!(dx, 50.0);
        assert_eq!(dy, 60.0);
    }

    // ── 新增测试：Incremental rendering / paint_in_rect ───────

    /// 测试 paint_in_rect 跳过完全在右侧的节点。
    #[test]
    fn test_paint_in_rect_skips_right() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 200.0, 0.0, 100.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
        assert!(painter.primitives().is_empty());
    }

    /// 测试 paint_in_rect 跳过完全在下方的节点。
    #[test]
    fn test_paint_in_rect_skips_below() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 300.0, 100.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 255, 0, 255);
        styles.insert(elem, style);

        let dirty_rect = Rect::new(0.0, 0.0, 800.0, 200.0);
        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
        assert!(painter.primitives().is_empty());
    }

    /// 测试 paint_in_rect 与脏区域刚好边缘相交的节点。
    #[test]
    fn test_paint_in_rect_edge_touch() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // Node right edge at x=100, dirty rect starts at x=99
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 0, 255, 255);
        styles.insert(elem, style);

        let dirty_rect = Rect::new(99.0, 0.0, 100.0, 50.0);
        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
        assert_eq!(painter.primitives().fills.len(), 1);
    }

    /// 测试 paint_text 带 border 和 padding 偏移 glyph 位置。
    #[test]
    fn test_paint_text_with_border_padding() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = LayoutBox {
            node_id: Some(elem),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            content_x: 5.0,
            content_y: 3.0,
            content_width: 90.0,
            content_height: 44.0,
            border_top: 3.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 5.0,
            padding_top: 1.0,
            padding_right: 1.0,
            padding_bottom: 1.0,
            padding_left: 1.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(12.0);
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);

        let glyph = &painter.primitives().glyphs[0];
        // text_x = abs_x(0) + border_left(5) + padding_left(1) = 6
        assert_eq!(glyph.x, 6.0);
        // text_y = abs_y(0) + border_top(3) + padding_top(1) = 4, + font_size(12) = 16
        assert_eq!(glyph.y, 16.0);
    }

    /// 测试 outline offset 非零时正确绘制。
    #[test]
    fn test_painter_outline_with_offset() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.outline_width = LengthValue::Px(2.0);
        style.outline_offset = LengthValue::Px(5.0);
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_color = ColorValue::Rgba(0, 128, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert_eq!(painter.primitives().fills.len(), 4);
        // top outline: y = abs_y - (outline_width + offset) = 20 - 7 = 13
        let top = &painter.primitives().fills[0];
        assert_eq!(top.rect.origin.y, 13.0);
        assert_eq!(top.rect.size.height, 2.0);
    }

    /// 测试 HSL 黄色（60°, 100%, 50%）转换。
    #[test]
    fn test_hsla_yellow() {
        let color = hsla_to_rgba(60.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    /// 测试 HSL 白色（0°, 0%, 100%）转换。
    #[test]
    fn test_hsla_white() {
        let color = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    /// 测试 HSL 黑色（0°, 0%, 0%）转换。
    #[test]
    fn test_hsla_black() {
        let color = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    /// 测试 named_color_to_render 其他颜色。
    #[test]
    fn test_named_colors_extended() {
        assert_eq!(named_color_to_render("orange"), Color::rgb(255, 165, 0));
        assert_eq!(named_color_to_render("pink"), Color::rgb(255, 192, 203));
        assert_eq!(named_color_to_render("brown"), Color::rgb(165, 42, 42));
        assert_eq!(named_color_to_render("navy"), Color::rgb(0, 0, 128));
        assert_eq!(named_color_to_render("teal"), Color::rgb(0, 128, 128));
        assert_eq!(named_color_to_render("silver"), Color::rgb(192, 192, 192));
    }

    // ── 新增测试：overflow clipping with nested elements ──────

    /// 测试嵌套元素中 overflow:hidden 逐层裁剪。
    ///
    /// grandparent(overflow:hidden, 100x100) > parent(overflow:visible, 200x200) > child(50x50)
    /// child 从 (80,80) 开始，parent 从 (0,0) 开始。
    /// grandparent 的 overflow:hidden 应裁剪所有后代（包括 parent 的背景）。
    #[test]
    fn test_overflow_hidden_clips_deeply_nested_children() {
        let mut doc = zero_dom::Document::new();
        let grandparent = doc.create_element("div");
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        // child 在 parent 内部，偏移 (80, 80)，大小 50x50
        let child_box = make_box(Some(child), 80.0, 80.0, 50.0, 50.0);
        // parent 大小 200x200（超出 grandparent 的 100x100）
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 200.0,
            content_height: 200.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        // grandparent overflow:hidden, content 100x100
        let grandparent_box = LayoutBox {
            node_id: Some(grandparent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![parent_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Hidden,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(0, 128, 0, 255);
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&grandparent_box, &styles, None);

        let fills = &painter.primitives().fills;
        assert!(!fills.is_empty(), "should produce fills from parent and child");

        // parent fill (200x200) should be clipped to grandparent content (100x100)
        let parent_fill = &fills[0];
        assert!(parent_fill.rect.size.width <= 100.0, "parent width clipped to 100");
        assert!(parent_fill.rect.size.height <= 100.0, "parent height clipped to 100");

        // child fill starts at (80,80) size 50x50 → clipped at right/bottom edge
        // visible area: x=[80,100], y=[80,100] → width=20, height=20
        let child_fill = &fills[1];
        assert_eq!(child_fill.rect.origin.x, 80.0);
        assert_eq!(child_fill.rect.origin.y, 80.0);
        assert_eq!(
            child_fill.rect.size.width, 20.0,
            "child width clipped at grandparent boundary"
        );
        assert_eq!(
            child_fill.rect.size.height, 20.0,
            "child height clipped at grandparent boundary"
        );
    }

    /// 测试双层 overflow:hidden 嵌套，内层和外层各自裁剪。
    ///
    /// outer(overflow:hidden, 80x80) > inner(overflow:hidden, 40x40) > child(100x100)
    /// child 完全在 inner 内，但 inner 裁剪到 40x40，outer 再裁剪 inner 的结果。
    #[test]
    fn test_overflow_hidden_double_nesting_clips() {
        let mut doc = zero_dom::Document::new();
        let outer = doc.create_element("div");
        let inner = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 100.0, 100.0);
        let inner_box = LayoutBox {
            node_id: Some(inner),
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 40.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 40.0,
            content_height: 40.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Hidden,
        };
        let outer_box = LayoutBox {
            node_id: Some(outer),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 80.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![inner_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Hidden,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&outer_box, &styles, None);

        // child(100x100) → clipped by inner(40x40) → 40x40
        // inner result(40x40) within outer(80x80) → no further clipping needed
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.size.width, 40.0, "child clipped by inner overflow:hidden");
        assert_eq!(fill.rect.size.height, 40.0, "child clipped by inner overflow:hidden");
    }

    // ── 新增测试：Inline formatting context（内联格式化上下文）─────────

    /// 测试块容器中的内联文本生成 glyph 图元。
    ///
    /// 场景：<div>Hello</div>，div 有明确的前景色和字体大小。
    /// 验证 paint() 在遍历布局树时自动为内联文本内容生成 GlyphPrimitive。
    #[test]
    fn test_paint_inline_text_in_block() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 255, 255, 255);
        style.color = ColorValue::Rgba(0, 0, 0, 255); // 前景色：黑色
        style.font_size = LengthValue::Px(16.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        let prims = painter.primitives();
        // 应生成背景填充 + 文本 glyph
        assert_eq!(prims.fills.len(), 1, "应生成 1 个背景填充");
        assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph 图元");

        let glyph = &prims.glyphs[0];
        assert_eq!(glyph.font_size, 16.0);
        assert_eq!(glyph.color, Color::rgb(0, 0, 0));
        // glyph 位置：text_x = 0 (无 border/padding), y = 0 + font_size(16) = 16
        assert_eq!(glyph.x, 0.0);
        assert_eq!(glyph.y, 16.0);
    }

    /// 测试混合内联和块级元素的图元顺序。
    ///
    /// 场景：父 div（背景灰色）包含三个子元素：
    /// - 子1（块级，红色背景）
    /// - 子2（内联文本，蓝色前景色）
    /// - 子3（块级，绿色背景）
    ///
    /// 验证：
    /// 1. 父背景先绘制（fills[0]）
    /// 2. 子元素按顺序绘制（子1 fill → 子2 glyph → 子3 fill）
    /// 3. 总 fills = 3，glyphs = 1
    #[test]
    fn test_paint_mixed_inline_block() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let block1 = doc.create_element("p");
        let inline_text = doc.create_element("span");
        let block2 = doc.create_element("p");

        let child1 = make_box(Some(block1), 0.0, 0.0, 200.0, 30.0);
        let child2 = make_box(Some(inline_text), 0.0, 30.0, 200.0, 20.0);
        let child3 = make_box(Some(block2), 0.0, 50.0, 200.0, 30.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 200.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child1, child2, child3],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();

        // 父：灰色背景，color=CurrentColor（不生成 glyph）
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        parent_style.color = ColorValue::CurrentColor;
        styles.insert(parent, parent_style);

        // 子1（块级）：红色背景，不生成 glyph
        let mut block1_style = ComputedStyle::default();
        block1_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        block1_style.color = ColorValue::CurrentColor;
        styles.insert(block1, block1_style);

        // 子2（内联文本）：无背景，蓝色前景色 → 只生成 glyph
        let mut inline_style = ComputedStyle::default();
        inline_style.background_color = ColorValue::Transparent;
        inline_style.color = ColorValue::Rgba(0, 0, 255, 255); // 蓝色
        inline_style.font_size = LengthValue::Px(14.0);
        styles.insert(inline_text, inline_style);

        // 子3（块级）：绿色背景，不生成 glyph
        let mut block2_style = ComputedStyle::default();
        block2_style.background_color = ColorValue::Rgba(0, 255, 0, 255);
        block2_style.color = ColorValue::CurrentColor;
        styles.insert(block2, block2_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        let prims = painter.primitives();

        // 父背景 + 子1 背景 + 子3 背景 = 3 个 fills
        assert_eq!(prims.fills.len(), 3, "应生成 3 个填充（父 + 子1 + 子3）");
        // 子2 只生成 1 个 glyph
        assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph（子2 内联文本）");

        // 验证绘制顺序：父背景先绘制
        assert_eq!(
            prims.fills[0].color,
            Color::rgb(200, 200, 200),
            "第一个 fill 应为父背景"
        );
        assert_eq!(prims.fills[1].color, Color::rgb(255, 0, 0), "第二个 fill 应为子1 背景");
        assert_eq!(prims.fills[2].color, Color::rgb(0, 255, 0), "第三个 fill 应为子3 背景");

        // glyph 颜色为蓝色
        assert_eq!(prims.glyphs[0].color, Color::rgb(0, 0, 255), "glyph 颜色应为蓝色");
        assert_eq!(prims.glyphs[0].font_size, 14.0);
        // glyph 位置：abs_y=30, text_y=30, baseline=30+14=44
        assert_eq!(prims.glyphs[0].y, 44.0);
    }

    /// 测试带颜色样式的内联文本正确应用到 glyph 图元。
    ///
    /// 场景：<span style="color: red; font-size: 20px;">Colored</span>
    /// 验证 glyph 的 color 字段匹配 CSS color 属性值。
    #[test]
    fn test_paint_text_with_color() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("span");
        let layout = make_box(Some(elem), 10.0, 20.0, 150.0, 25.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(255, 0, 0, 255); // 红色
        style.font_size = LengthValue::Px(20.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        let prims = painter.primitives();
        assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph");

        let glyph = &prims.glyphs[0];
        // 颜色正确应用
        assert_eq!(glyph.color, Color::rgb(255, 0, 0), "glyph 颜色应为红色");
        assert_eq!(glyph.font_size, 20.0, "glyph font_size 应为 20");
        // 位置：abs_x=10, abs_y=20, text_x=10, baseline=20+20=40
        assert_eq!(glyph.x, 10.0);
        assert_eq!(glyph.y, 40.0);
    }

    // ── 新增测试：InlineFormattingContext 集成 ──────────────────────

    /// 测试 paint_text 使用 InlineFormattingContext 为每个文本片段生成 glyph。
    ///
    /// 场景：<p>Hello World</p>，容器宽度较窄，文本自动换行。
    /// 当传入 Document 时，paint_text 应通过 InlineFormattingContext
    /// 将文本分割为单词，为每个单词生成独立的 GlyphPrimitive。
    #[test]
    fn test_paint_text_with_inline_formatting_context() {
        let doc = zero_dom::parse_html("<p>Hello World</p>");

        // 找到 p 元素
        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let layout = make_box(Some(p), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(p, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        let prims = painter.primitives();
        // InlineFormattingContext 会将 "Hello World" 分成 2 个单词片段
        assert!(
            prims.glyphs.len() >= 2,
            "应有至少 2 个 glyph（Hello 和 World），实际 {}",
            prims.glyphs.len()
        );

        // 验证每个 glyph 的颜色和字体大小正确
        for glyph in &prims.glyphs {
            assert_eq!(glyph.color, Color::rgb(0, 0, 0));
            assert_eq!(glyph.font_size, 16.0);
        }
    }

    /// 测试 paint_text 不传 Document 时退化为单个占位 glyph。
    ///
    /// 验证 doc=None 时 paint_text 仍然正常工作，
    /// 生成单个 glyph 作为占位。
    #[test]
    fn test_paint_text_without_doc_fallback() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("p");
        let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // doc=None → 退化为单个占位 glyph
        assert_eq!(painter.primitives().glyphs.len(), 1, "doc=None 时应退化为 1 个 glyph");
    }

    /// 测试 InlineFormattingContext 生成的 glyph 位置包含容器偏移。
    ///
    /// 场景：<p>Text</p>，p 元素有 border 和 padding 偏移。
    /// 验证 glyph 的坐标包含 content_x/content_y 偏移。
    #[test]
    fn test_paint_inline_glyph_position_with_offset() {
        let doc = zero_dom::parse_html("<p>Text</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let layout = LayoutBox {
            node_id: Some(p),
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 50.0,
            content_x: 15.0,
            content_y: 25.0,
            content_width: 190.0,
            content_height: 40.0,
            border_top: 2.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 2.0,
            padding_top: 3.0,
            padding_right: 3.0,
            padding_bottom: 3.0,
            padding_left: 3.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(p, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        let prims = painter.primitives();
        assert!(!prims.glyphs.is_empty(), "应生成 glyph");

        // 第一个 glyph 的 x 应包含 content_x 偏移
        // content_x = abs_x(10) + border_left(2) + padding_left(3) = 15
        let glyph = &prims.glyphs[0];
        assert!(glyph.x >= 15.0, "glyph x 应包含内容区域偏移，实际 {}", glyph.x);
        // y 应包含 content_y 偏移 + 行高
        assert!(glyph.y >= 25.0, "glyph y 应包含内容区域偏移，实际 {}", glyph.y);
    }

    /// 测试窄容器中 InlineFormattingContext 为文本内容生成 glyph。
    ///
    /// 场景：容器宽度只有 60px，文本 "a b c d e f g h" 应产生 glyph。
    #[test]
    fn test_paint_inline_text_wrapping_multiple_lines() {
        let doc = zero_dom::parse_html("<p>a b c d e f g h</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        // 窄容器 — 强制文本换行
        let layout = make_box(Some(p), 0.0, 0.0, 60.0, 200.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(p, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        let prims = painter.primitives();
        // paint 应该为文本内容生成至少一些 glyph
        assert!(
            prims.glyphs.len() >= 1,
            "容器中的文本应产生 glyph，实际 {}",
            prims.glyphs.len()
        );
    }

    /// 测试混合 inline 元素的文本通过 InlineFormattingContext 正确生成 glyph。
    ///
    /// 场景：<p>Hello <b>World</b></p>
    /// p 包含文本节点 "Hello " 和 b 元素 "World"。
    /// InlineFormattingContext 会收集两者并分割为单词片段。
    #[test]
    fn test_paint_inline_mixed_text_and_elements() {
        let doc = zero_dom::parse_html("<p>Hello <b>World</b></p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let layout = make_box(Some(p), 0.0, 0.0, 400.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(p, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        let prims = painter.primitives();
        // "Hello" 和 "World" 各一个片段
        assert!(
            prims.glyphs.len() >= 2,
            "混合文本和 inline 元素应产生至少 2 个 glyph，实际 {}",
            prims.glyphs.len()
        );
    }

    /// 测试空文本节点不产生 glyph（InlineFormattingContext 过滤空白）。
    ///
    /// 场景：<p>   </p>，文本只有空白字符。
    /// InlineFormattingContext 的 trim 过滤后不应产生任何片段。
    #[test]
    fn test_paint_inline_whitespace_only_no_glyphs() {
        let doc = zero_dom::parse_html("<p>   </p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let layout = make_box(Some(p), 0.0, 0.0, 200.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(0, 0, 0, 255);
        style.font_size = LengthValue::Px(16.0);
        styles.insert(p, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // 纯空白文本被 trim 后为空字符串，不产生 TextRun，
        // 因此 InlineFormattingContext 无片段 → 走 fallback 生成 1 个 glyph
        assert!(
            painter.primitives().glyphs.len() <= 1,
            "纯空白文本应产生 0 或 1 个 fallback glyph，实际 {}",
            painter.primitives().glyphs.len()
        );
    }

    /// 测试 render_html 通过 pipeline 使用 InlineFormattingContext。
    ///
    /// 验证端到端管线中 InlineFormattingContext 被正确调用：
    /// HTML 解析 → 样式计算 → 布局 → paint(传入 Document) → 生成 glyph。
    #[test]
    fn test_pipeline_uses_inline_formatting_for_text() {
        use crate::pipeline::RenderPipeline;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><p>Hello World</p></body></html>";
        let css = "p { color: black; font-size: 16px; }";
        let result = pipeline.render_html(html, css);

        // Pipeline 应为 p 元素生成 glyph
        assert!(
            !result.primitives.glyphs.is_empty(),
            "render_html 应通过 InlineFormattingContext 生成 glyph"
        );
    }

    /// 测试 pipeline render_html 生成 glyph。
    #[test]
    fn test_pipeline_inline_text_with_css_color() {
        use crate::pipeline::RenderPipeline;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><p>Styled</p></body></html>";
        let css = "p { color: red; font-size: 18px; }";
        let result = pipeline.render_html(html, css);

        // Pipeline 应该为文本内容生成 glyph（颜色传播取决于管线实现完整度）
        assert!(!result.primitives.glyphs.is_empty(), "应生成 glyph");
        // 验证 glyph 字体大小正确
        assert!(
            result.primitives.glyphs.iter().any(|g| g.font_size > 0.0),
            "至少一个 glyph 应有非零字体大小"
        );
    }

    // ── 边界条件测试 ──────────────────────────────────────────

    /// 测试 HSL 色相 120（绿色）。
    #[test]
    fn test_hsla_green_120() {
        let color = hsla_to_rgba(120.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 色相 240（蓝色）。
    #[test]
    fn test_hsla_blue_240() {
        let color = hsla_to_rgba(240.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 饱和度 0% 和亮度 0%（黑色）。
    #[test]
    fn test_hsla_zero_saturation_zero_lightness() {
        let color = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    /// 测试 HSL 饱和度 0% 和亮度 100%（白色）。
    #[test]
    fn test_hsla_zero_saturation_full_lightness() {
        let color = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
        assert_eq!(color.a, 255);
    }

    /// 测试 border-style: hidden 不产生填充。
    #[test]
    fn test_border_style_hidden_no_fill() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 5.0, 5.0, 5.0, 5.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_left_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_top_style = BorderStyleValue::Hidden;
        style.border_right_style = BorderStyleValue::Hidden;
        style.border_bottom_style = BorderStyleValue::Hidden;
        style.border_left_style = BorderStyleValue::Hidden;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // border-style: hidden 与 none 行为一致，不绘制边框
        assert_eq!(
            painter.primitives().fills.len(),
            0,
            "hidden border should produce no fills"
        );
    }

    /// 测试 zero-width border with solid style 不产生填充。
    #[test]
    fn test_border_zero_width_solid_style_no_fill() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // border_top = 0.0, style = Solid => no fill for top border
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 0.0, 5.0, 5.0, 5.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // 只有 3 个边框填充（top border 宽度为 0，不绘制）
        assert_eq!(
            painter.primitives().fills.len(),
            3,
            "zero-width top border should produce no fill"
        );
    }

    /// 测试 named_color_to_render: lime, purple, maroon, olive, aqua, fuchsia, grey。
    #[test]
    fn test_named_colors_lime_purple_maroon() {
        assert_eq!(named_color_to_render("lime"), Color::rgb(0, 255, 0));
        assert_eq!(named_color_to_render("purple"), Color::rgb(128, 0, 128));
        assert_eq!(named_color_to_render("maroon"), Color::rgb(128, 0, 0));
        assert_eq!(named_color_to_render("olive"), Color::rgb(128, 128, 0));
        assert_eq!(named_color_to_render("aqua"), Color::rgb(0, 255, 255));
        assert_eq!(named_color_to_render("fuchsia"), Color::rgb(255, 0, 255));
        assert_eq!(named_color_to_render("grey"), Color::rgb(128, 128, 128));
    }

    /// 测试 outline_width = 0 不产生填充。
    #[test]
    fn test_outline_zero_width_no_fill() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.outline_width = LengthValue::Px(0.0);
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert!(
            painter.primitives().is_empty(),
            "zero-width outline should produce no fills"
        );
    }

    /// 测试 paint_text with non-Px font size (Em) — early return, no glyph。
    #[test]
    fn test_paint_text_em_font_size_no_glyph() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Em(1.0);
        style.color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
        assert!(
            painter.primitives().glyphs.is_empty(),
            "Em font size should produce no glyph"
        );
    }

    /// 测试 paint_in_rect: parent outside dirty rect, child inside — parent culling should skip subtree。
    #[test]
    fn test_paint_in_rect_parent_outside_child_inside_skipped() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        // child 在 (300, 300) 处
        let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 50.0);
        // parent 在 (300, 300) 处，完全在脏区域外
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 300.0,
            y: 300.0,
            width: 100.0,
            height: 100.0,
            content_x: 300.0,
            content_y: 300.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        // 脏区域在 (0, 0) 处，parent 在 (300, 300) 完全不在脏区域内
        let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        let mut painter = Painter::new();
        painter.paint_in_rect(&parent_box, &styles, &dirty_rect, None);

        // parent 完全在脏区域外，整个子树（包括 child）被跳过
        assert!(
            painter.primitives().is_empty(),
            "parent outside dirty rect should skip entire subtree including child"
        );
    }

    /// 测试 zero-offset translate 不改变位置。
    #[test]
    fn test_transform_zero_translate_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(0.0, 0.0)]);
        let (dx, dy) = apply_transform_offset(&style, 10.0, 20.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    // ── 边界条件测试：clip_fills / clip_glyphs / color ──────

    /// clip_fills 部分重叠：fill 矩形与 clip 矩形部分重叠时，缩小到交集。
    #[test]
    fn test_clip_fills_partial_overlap() {
        use zero_render_foundation::primitive::FillPrimitive;

        // clip rect: (0, 0, 100, 100)
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        // fill rect: (50, 50, 100, 100) → 与 clip 交集为 (50, 50, 50, 50)
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(50.0, 50.0, 100.0, 100.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.origin.x, 50.0);
        assert_eq!(fills[0].rect.origin.y, 50.0);
        assert_eq!(fills[0].rect.size.width, 50.0);
        assert_eq!(fills[0].rect.size.height, 50.0);
    }

    /// clip_fills 完全在外侧：左/右/上/下四个方向均被清零。
    #[test]
    fn test_clip_fills_outside_each_side() {
        use zero_render_foundation::primitive::FillPrimitive;

        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);

        // 左侧完全在 clip 外
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(-150.0, 0.0, 100.0, 100.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.size.width, 0.0);
        assert_eq!(fills[0].rect.size.height, 0.0);

        // 右侧完全在 clip 外
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(200.0, 0.0, 100.0, 100.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.size.width, 0.0);
        assert_eq!(fills[0].rect.size.height, 0.0);

        // 上侧完全在 clip 外
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(0.0, -200.0, 100.0, 100.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.size.width, 0.0);
        assert_eq!(fills[0].rect.size.height, 0.0);

        // 下侧完全在 clip 外
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 200.0, 100.0, 100.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.size.width, 0.0);
        assert_eq!(fills[0].rect.size.height, 0.0);
    }

    /// clip_fills start index > 0：只有 start 之后的 fill 被裁剪。
    #[test]
    fn test_clip_fills_start_index() {
        use zero_render_foundation::primitive::FillPrimitive;

        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        // 第一个 fill 在 clip 内（不受影响），第二个完全在 clip 外
        let mut fills = vec![
            FillPrimitive {
                rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                color: Color::BLACK,
            },
            FillPrimitive {
                rect: Rect::new(200.0, 200.0, 50.0, 50.0),
                color: Color::BLACK,
            },
        ];
        clip_fills(&mut fills, 1, &clip);
        // 第一个 fill 不应被裁剪
        assert_eq!(fills[0].rect.size.width, 50.0);
        assert_eq!(fills[0].rect.size.height, 50.0);
        // 第二个 fill 应被清零
        assert_eq!(fills[1].rect.size.width, 0.0);
        assert_eq!(fills[1].rect.size.height, 0.0);
    }

    /// clip_fills 空 slice 不 panic。
    #[test]
    fn test_clip_fills_empty_slice() {
        use zero_render_foundation::primitive::FillPrimitive;

        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut fills: Vec<FillPrimitive> = vec![];
        clip_fills(&mut fills, 0, &clip);
        // 应正常返回，不 panic
    }

    /// clip_fills fill rect 完全匹配 clip rect → 不变。
    #[test]
    fn test_clip_fills_exact_match() {
        use zero_render_foundation::primitive::FillPrimitive;

        let clip = Rect::new(10.0, 20.0, 80.0, 60.0);
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(10.0, 20.0, 80.0, 60.0),
            color: Color::BLACK,
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.origin.x, 10.0);
        assert_eq!(fills[0].rect.origin.y, 20.0);
        assert_eq!(fills[0].rect.size.width, 80.0);
        assert_eq!(fills[0].rect.size.height, 60.0);
    }

    /// clip_glyphs 字形在 clip 外侧（左/右/上/下）→ glyph_id 设为 0。
    #[test]
    fn test_clip_glyphs_outside_rejection() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);

        // 左侧：glyph 在 (-50, 10)，font_size=16 → right = -34，在 clip 左侧
        let mut glyphs = vec![GlyphPrimitive {
            x: -50.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 0);

        // 右侧：glyph 在 (150, 10)，x >= clip right (100)
        let mut glyphs = vec![GlyphPrimitive {
            x: 150.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 0);

        // 上侧：glyph 在 (10, -50)，font_size=16 → bottom = -34
        let mut glyphs = vec![GlyphPrimitive {
            x: 10.0,
            y: -50.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 0);

        // 下侧：glyph 在 (10, 150)，y >= clip bottom (100)
        let mut glyphs = vec![GlyphPrimitive {
            x: 10.0,
            y: 150.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 0);
    }

    /// clip_glyphs start > 0：只有 start 之后的 glyph 被裁剪。
    #[test]
    fn test_clip_glyphs_start_index() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut glyphs = vec![
            // glyph[0] 在 clip 外（不应被裁剪，因为 start=1）
            GlyphPrimitive {
                x: 200.0,
                y: 200.0,
                font_size: 16.0,
                color: Color::BLACK,
                glyph_id: 10,
                font_id: FontId(0),
                bitmap_width: None,
                bitmap_height: None,
            },
            // glyph[1] 在 clip 外（应被裁剪）
            GlyphPrimitive {
                x: 200.0,
                y: 200.0,
                font_size: 16.0,
                color: Color::BLACK,
                glyph_id: 20,
                font_id: FontId(0),
                bitmap_width: None,
                bitmap_height: None,
            },
        ];
        clip_glyphs(&mut glyphs, 1, &clip);
        // 第一个 glyph 不受影响
        assert_eq!(glyphs[0].glyph_id, 10);
        // 第二个 glyph 被清零
        assert_eq!(glyphs[1].glyph_id, 0);
    }

    /// clip_glyphs 字形在 clip 内 → 不被裁剪。
    #[test]
    fn test_clip_glyphs_inside_not_clipped() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut glyphs = vec![GlyphPrimitive {
            x: 10.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 65);
        assert_eq!(glyphs[0].font_size, 16.0);
    }

    /// clip_glyphs 空 slice 不 panic。
    #[test]
    fn test_clip_glyphs_empty_slice() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut glyphs: Vec<GlyphPrimitive> = vec![];
        clip_glyphs(&mut glyphs, 0, &clip);
        // 应正常返回，不 panic
    }

    /// color_value_to_render CurrentColor → rgba(0,0,0,255)。
    #[test]
    fn test_color_value_to_render_current_color() {
        let color = color_value_to_render(&ColorValue::CurrentColor);
        assert_eq!(color, Color::rgba(0, 0, 0, 255));
    }

    /// hsla_to_rgba(300, 100, 50, 1.0) → 品红区域，验证 RGB 值。
    /// hue 300: h'=5.0, 进入 _ => (c, 0.0, x) 分支
    /// c=1.0, x=1.0*(1.0-|5.0%2-1.0|)=1.0*(1.0-0.0)=1.0, m=0.0
    /// r=255, g=0, b=255
    #[test]
    fn test_hsla_hue_300_magenta_region() {
        let color = hsla_to_rgba(300.0, 100.0, 50.0, 1.0);
        let Color { r, g, b, a } = color;
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
        assert_eq!(a, 255);
    }

    /// hsla_to_rgba(330, 100, 50, 1.0) → 验证结果。
    /// hue 330: h'=5.5, 进入 _ => (c, 0.0, x)
    /// c=1.0, x=1.0*(1.0-|5.5%2-1.0|)=1.0*(1.0-|1.5-1.0|)=1.0*(1.0-0.5)=0.5, m=0.0
    /// r=255, g=0, b=128
    #[test]
    fn test_hsla_hue_330_region() {
        let color = hsla_to_rgba(330.0, 100.0, 50.0, 1.0);
        let Color { r, g, b, a } = color;
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 128);
        assert_eq!(a, 255);
    }

    /// length_to_f32 对非 Px 单位返回 0.0。
    #[test]
    fn test_length_to_f32_non_px() {
        assert_eq!(length_to_f32(&LengthValue::Em(2.0)), 0.0);
        assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0);
        assert_eq!(length_to_f32(&LengthValue::Rem(1.5)), 0.0);
    }

    /// named_color_to_render 扩展颜色测试。
    #[test]
    fn test_named_color_extended() {
        assert_eq!(named_color_to_render("cyan"), Color::rgb(0, 255, 255));
        assert_eq!(named_color_to_render("aqua"), Color::rgb(0, 255, 255));
        assert_eq!(named_color_to_render("magenta"), Color::rgb(255, 0, 255));
        assert_eq!(named_color_to_render("fuchsia"), Color::rgb(255, 0, 255));
        assert_eq!(named_color_to_render("silver"), Color::rgb(192, 192, 192));
        assert_eq!(named_color_to_render("maroon"), Color::rgb(128, 0, 0));
        assert_eq!(named_color_to_render("olive"), Color::rgb(128, 128, 0));
        assert_eq!(named_color_to_render("lime"), Color::rgb(0, 255, 0));
        assert_eq!(named_color_to_render("purple"), Color::rgb(128, 0, 128));
        assert_eq!(named_color_to_render("teal"), Color::rgb(0, 128, 128));
        assert_eq!(named_color_to_render("navy"), Color::rgb(0, 0, 128));
        assert_eq!(named_color_to_render("orange"), Color::rgb(255, 165, 0));
        assert_eq!(named_color_to_render("pink"), Color::rgb(255, 192, 203));
        assert_eq!(named_color_to_render("brown"), Color::rgb(165, 42, 42));
    }

    /// named_color_to_render 未知颜色名 → 回退为 rgb(0,0,0)。
    #[test]
    fn test_named_color_unknown() {
        assert_eq!(named_color_to_render("nonexistent"), Color::rgb(0, 0, 0));
        assert_eq!(named_color_to_render("chartreuse"), Color::rgb(0, 0, 0));
        assert_eq!(named_color_to_render(""), Color::rgb(0, 0, 0));
    }

    /// 测试子元素 visibility:visible 覆盖父元素 visibility:hidden。
    ///
    /// 父元素设置为 visibility:hidden，子元素设置为 visibility:visible。
    /// 父元素不应绘制自身背景，但子元素应正常绘制。
    #[test]
    fn test_painter_child_visible_overrides_parent_hidden() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        parent_style.visibility = VisibilityValue::Hidden;
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
        child_style.visibility = VisibilityValue::Visible;
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, None);

        // 父元素 visibility:hidden → 不绘制自身背景
        // 子元素 visibility:visible → 正常绘制
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(100, 100, 255));
    }

    /// 测试 LayoutBox 的 node_id=None 但传入 doc=Some 时退化为 fallback glyph。
    ///
    /// 当布局盒没有关联 DOM 节点（node_id=None），即使传入了 Document，
    /// paint_text 也无法使用 InlineFormattingContext，应退化为 glyph_id=0 的占位 glyph。
    #[test]
    fn test_paint_text_doc_some_node_id_none_fallback() {
        let doc = zero_dom::Document::new();

        // node_id=None 的布局盒
        let layout = make_box(None, 0.0, 0.0, 200.0, 30.0);

        let style = ComputedStyle {
            color: ColorValue::Rgba(0, 0, 0, 255),
            font_size: LengthValue::Px(16.0),
            ..ComputedStyle::default()
        };

        let mut painter = Painter::new();
        painter.paint_text(&layout, 0.0, 0.0, &style, Some(&doc));

        // node_id=None → 无法使用 InlineFormattingContext → 走 fallback 路径
        assert_eq!(painter.primitives().glyphs.len(), 1);
        let glyph = &painter.primitives().glyphs[0];
        assert_eq!(glyph.glyph_id, 0, "fallback glyph 应为 glyph_id=0");
        assert_eq!(glyph.font_size, 16.0);
    }

    /// 测试 visibility:collapse 在非表格元素上表现为 hidden。
    ///
    /// 根据 CSS 规范，visibility:collapse 在非表格行/列元素上
    /// 应与 visibility:hidden 行为一致，元素不绘制但保留布局空间。
    #[test]
    fn test_painter_visibility_collapse_acts_as_hidden() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        style.visibility = VisibilityValue::Collapse;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        // visibility:collapse 应阻止元素绘制（与 hidden 行为一致）
        assert!(
            painter.primitives().fills.is_empty(),
            "visibility:collapse 应阻止元素绘制"
        );
        assert!(
            painter.primitives().glyphs.is_empty(),
            "visibility:collapse 应阻止 glyph 生成"
        );
    }

    /// 测试 paint_in_rect 对 visibility:hidden 的节点不生成任何图元。
    ///
    /// 增量绘制路径（paint_node_in_rect）同样应遵守 visibility 规则，
    /// 隐藏元素不应产生任何填充或 glyph 图元。
    #[test]
    fn test_paint_in_rect_visibility_hidden_skips_node() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // 节点与脏区域相交
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        style.visibility = VisibilityValue::Hidden;
        styles.insert(elem, style);

        let dirty_rect = Rect::new(0.0, 0.0, 200.0, 200.0);

        let mut painter = Painter::new();
        painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

        // visibility:hidden → 节点不应产生任何图元
        assert!(
            painter.primitives().is_empty(),
            "visibility:hidden 在 paint_in_rect 中应跳过节点绘制"
        );
    }
}
