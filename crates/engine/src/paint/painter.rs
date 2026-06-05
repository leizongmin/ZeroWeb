//! 绘制命令生成器 — Painter 结构体及其绘制方法。

use std::collections::{HashMap, HashSet};

use zero_css_parser::values::ColorValue;
use zero_css_parser::values::LengthValue;
use zero_css_parser::values::ListStyleTypeValue;
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
    BlendMode, BlendModePrimitive, FilterKind, FilterPrimitive, FontId, GlyphPrimitive, ImagePrimitive, LineCap,
    LineStyle, RenderPrimitives, ShadowPrimitive, StrokePrimitive,
};
use zero_style_system::{
    BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundOriginComputedValue,
    BackgroundPositionComputedValue, BackgroundSizeComputedValue, BorderImageSourceComputedValue, BorderStyleValue,
    ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue, ColumnWidthComputedValue,
    ComputedStyle, FilterComputedValue, MixBlendModeComputedValue, OutlineStyleValue, ResizeValue,
    TextDecorationLineValue, TextOverflowValue,
};

use super::color::color_value_to_render;
use super::helpers::{
    BorderRadiusSpec, PrimitiveCounts, apply_opacity_to_new_primitives, apply_text_transform, clip_fills, clip_glyphs,
    gradient_to_primitive, length_to_f32, simple_hash,
};

/// 边框边缘规格 — 描述一条边框的几何位置和方向。
struct BorderEdgeSpec {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    is_horizontal: bool,
    /// 垂直边框时，填充区域是否向左延伸（右侧边框为 true）。
    extend_left: bool,
}

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

            let skip_empty_cell = matches!(style.empty_cells, zero_style_system::EmptyCellsComputedValue::Hide)
                && box_node.children.is_empty();

            if !hidden && !skip_empty_cell {
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
            }

            if !hidden {
                if let Some(doc) = doc {
                    self.paint_list_marker(box_node, abs_x, abs_y, style, doc);
                }
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

            // empty-cells:hide — 空表格单元格不绘制背景和边框
            let skip_empty_cell = matches!(style.empty_cells, zero_style_system::EmptyCellsComputedValue::Hide)
                && box_node.children.is_empty();

            if !hidden && !skip_empty_cell {
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

                // 2b. Border-image 绘制（替换或覆盖常规边框）
                self.paint_border_image(box_node, abs_x, abs_y, style);

                // 2c. Column-rule 绘制（多列之间的分隔线）
                self.paint_column_rules(box_node, abs_x, abs_y, style);

                // 3. Outline 绘制（位于 border 外侧）
                self.paint_outline(box_node, abs_x, abs_y, style);
            }

            // 列表标记和文本始终绘制（不受 empty-cells 影响）
            if !hidden {
                // 4. 列表标记绘制（bullets/numbers，位于文本之前）
                if let Some(doc) = doc {
                    self.paint_list_marker(box_node, abs_x, abs_y, style, doc);
                }

                // 5. 文本内容绘制（含 text-shadow，使用行内格式化上下文处理换行）
                self.paint_text(box_node, abs_x, abs_y, style, doc);
            }

            hidden
        } else {
            false
        };

        // 6. 递归绘制子节点（子节点偏移 = 父 padding + border）
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

        // CSS mix-blend-mode — 对元素及其子元素产生的图元应用混合模式
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && !matches!(style.mix_blend_mode, MixBlendModeComputedValue::Normal)
        {
            self.apply_blend_mode(box_node, abs_x, abs_y, style);
        }

        // CSS resize — 绘制调整大小手柄指示器
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && !matches!(style.resize, ResizeValue::None)
        {
            self.paint_resize_handle(box_node, abs_x, abs_y, style);
        }

        let _ = is_hidden; // visibility 在 if let 块内处理
    }

    /// 绘制背景（考虑 border-radius）。
    ///
    /// 当 border-radius 为零时退化为普通矩形填充。
    fn paint_background(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let radii = BorderRadiusSpec::from_style(style);

        // 根据 background-clip 决定背景绘制区域
        let (clip_x, clip_y, clip_w, clip_h) = match style.background_clip {
            BackgroundClipComputedValue::BorderBox => (abs_x, abs_y, box_node.width, box_node.height),
            BackgroundClipComputedValue::PaddingBox => (
                abs_x + box_node.border_left,
                abs_y + box_node.border_top,
                box_node.width - box_node.border_left - box_node.border_right,
                box_node.height - box_node.border_top - box_node.border_bottom,
            ),
            BackgroundClipComputedValue::ContentBox => (
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            ),
            BackgroundClipComputedValue::Text => {
                // background-clip: text — 暂按 content-box 处理
                (
                    abs_x + box_node.border_left + box_node.padding_left,
                    abs_y + box_node.border_top + box_node.padding_top,
                    box_node.content_width,
                    box_node.content_height,
                )
            }
        };

        if radii.is_zero() {
            // 无圆角：简单矩形填充
            self.primitives.add_fill(
                Rect::new(clip_x, clip_y, clip_w, clip_h),
                color_value_to_render(&style.background_color),
            );
        } else {
            // 圆角矩形：生成带圆角信息的填充图元
            self.primitives.add_fill(
                Rect::new(clip_x, clip_y, clip_w, clip_h),
                color_value_to_render(&style.background_color),
            );
            // 存储圆角信息（当前架构下 FillPrimitive 没有圆角字段，
            // 通过附加的元数据图元标记圆角）
            self.add_rounded_rect_metadata(clip_x, clip_y, clip_w, clip_h, &radii);
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

    /// 绘制边框（4 条边，支持多种 border-style）。
    ///
    /// 分别绘制上、右、下、左四条边框。根据 border-style 生成不同类型的图元：
    /// - Solid/None/Hidden：填充矩形（原有行为）
    /// - Dotted：圆头点线描边
    /// - Dashed：方头虚线描边
    /// - Double：双线填充矩形（中间留空隙）
    /// - Groove/Ridge/Inset/Outset：3D 效果双色填充
    fn paint_borders(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let w = box_node.width;
        let h = box_node.height;

        // 上边框
        if box_node.border_top > 0.0
            && style.border_top_style != BorderStyleValue::None
            && style.border_top_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y,
                    x2: abs_x + w,
                    y2: abs_y,
                    thickness: box_node.border_top,
                    is_horizontal: true,
                    extend_left: false,
                },
                &style.border_top_style,
                &style.border_top_color,
            );
        }

        // 右边框
        if box_node.border_right > 0.0
            && style.border_right_style != BorderStyleValue::None
            && style.border_right_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x + w,
                    y1: abs_y + box_node.border_top,
                    x2: abs_x + w,
                    y2: abs_y + h - box_node.border_bottom,
                    thickness: box_node.border_right,
                    is_horizontal: false,
                    extend_left: true,
                },
                &style.border_right_style,
                &style.border_right_color,
            );
        }

        // 下边框
        if box_node.border_bottom > 0.0
            && style.border_bottom_style != BorderStyleValue::None
            && style.border_bottom_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + h,
                    x2: abs_x + w,
                    y2: abs_y + h,
                    thickness: box_node.border_bottom,
                    is_horizontal: true,
                    extend_left: false,
                },
                &style.border_bottom_style,
                &style.border_bottom_color,
            );
        }

        // 左边框
        if box_node.border_left > 0.0
            && style.border_left_style != BorderStyleValue::None
            && style.border_left_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + box_node.border_top,
                    x2: abs_x,
                    y2: abs_y + h - box_node.border_bottom,
                    thickness: box_node.border_left,
                    is_horizontal: false,
                    extend_left: false,
                },
                &style.border_left_style,
                &style.border_left_color,
            );
        }
    }

    /// 绘制 border-image。
    ///
    /// 当 border-image-source 不为 none 时，将图片按 slice 分割为
    /// 9 个区域（4 角 + 4 边 + 中心），分别绘制到边框的对应区域。
    /// 当前实现支持 stretch 模式（默认），生成 ImagePrimitive 图元。
    fn paint_border_image(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let url = match &style.border_image_source {
            BorderImageSourceComputedValue::None => return,
            BorderImageSourceComputedValue::Url(u) => u.clone(),
        };

        let bt = box_node.border_top;
        let br = box_node.border_right;
        let bb = box_node.border_bottom;
        let bl = box_node.border_left;

        // 至少有一条边框才绘制
        if bt <= 0.0 && br <= 0.0 && bb <= 0.0 && bl <= 0.0 {
            return;
        }

        let w = box_node.width;
        let h = box_node.height;
        let key = simple_hash(&url);

        // 辅助：创建 ImagePrimitive（每次创建新的 ImageKey，因为 ImageKey 不是 Copy）
        let make_img = |rect: Rect| ImagePrimitive {
            rect,
            image_key: ImageKey::new(key),
        };

        // 边框区域的坐标
        let bx = abs_x;
        let by = abs_y;

        // 四条边的尺寸
        let edge_h_w = (w - bl - br).max(0.0);
        let edge_v_h = (h - bt - bb).max(0.0);

        let fill = style.border_image_slice.fill;

        // 中心区域（当 fill 为 true 时绘制）
        if fill && edge_h_w > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by + bt, edge_h_w, edge_v_h)));
        }

        // 四个角
        if bl > 0.0 && bt > 0.0 {
            self.primitives.add_image(make_img(Rect::new(bx, by, bl, bt)));
        }
        if br > 0.0 && bt > 0.0 {
            self.primitives.add_image(make_img(Rect::new(bx + w - br, by, br, bt)));
        }
        if br > 0.0 && bb > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + w - br, by + h - bb, br, bb)));
        }
        if bl > 0.0 && bb > 0.0 {
            self.primitives.add_image(make_img(Rect::new(bx, by + h - bb, bl, bb)));
        }

        // 四条边（stretch 模式）
        if edge_h_w > 0.0 && bt > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by, edge_h_w, bt)));
        }
        if br > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + w - br, by + bt, br, edge_v_h)));
        }
        if edge_h_w > 0.0 && bb > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by + h - bb, edge_h_w, bb)));
        }
        if bl > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx, by + bt, bl, edge_v_h)));
        }
    }

    /// 绘制多列布局的 column-rule（列之间的分隔线）。
    ///
    /// 根据 column-count 或 column-width 计算列数和列间距，
    /// 在列之间绘制 column-rule 样式的垂直线。
    fn paint_column_rules(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        // 计算 column-count
        let count = match &style.column_count {
            ColumnCountComputedValue::Auto => {
                // 如果 column-width 也为 auto，则只有一列，不绘制 rule
                match &style.column_width {
                    ColumnWidthComputedValue::Auto => return,
                    ColumnWidthComputedValue::Length(LengthValue::Px(w)) => {
                        let content_w = box_node.content_width;
                        if content_w <= 0.0 || *w <= 0.0 {
                            return;
                        }
                        // column-gap 解析为 column_gap (LengthValue)
                        let gap: f32 = match style.column_gap {
                            LengthValue::Px(g) => g as f32,
                            _ => 0.0,
                        };
                        ((content_w + gap) / (*w as f32 + gap)).max(1.0).floor() as u32
                    }
                    _ => return,
                }
            }
            ColumnCountComputedValue::Number(n) => *n,
        };

        // 至少需要 2 列才绘制 rule
        if count < 2 {
            return;
        }

        // 检查 column-rule-style
        if matches!(
            style.column_rule_style,
            ColumnRuleStyleComputedValue::None | ColumnRuleStyleComputedValue::Hidden
        ) {
            return;
        }

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        let content_w = box_node.content_width;
        let content_h = box_node.content_height;

        if content_w <= 0.0 || content_h <= 0.0 {
            return;
        }

        // column-gap
        let gap: f32 = match style.column_gap {
            LengthValue::Px(g) => g as f32,
            _ => 0.0,
        };

        // column-rule-width
        let rule_w: f32 = match &style.column_rule_width {
            ColumnRuleWidthComputedValue::Medium => 2.0,
            ColumnRuleWidthComputedValue::Thin => 1.0,
            ColumnRuleWidthComputedValue::Thick => 3.0,
            ColumnRuleWidthComputedValue::Length(LengthValue::Px(w)) => *w as f32,
            _ => 1.0,
        };

        let rule_color = color_value_to_render(&style.column_rule_color);

        // 列宽 = (content_w - (count-1)*gap) / count
        let col_w = (content_w - (count as f32 - 1.0) * gap) / count as f32;
        if col_w <= 0.0 {
            return;
        }

        // 在每两列之间绘制 rule
        for i in 1..count {
            let rule_x = content_x + i as f32 * col_w + (i as f32 - 0.5) * gap - rule_w / 2.0;
            let rule_x = rule_x.max(content_x);
            match style.column_rule_style {
                ColumnRuleStyleComputedValue::Solid => {
                    self.primitives
                        .add_fill(Rect::new(rule_x, content_y, rule_w, content_h), rule_color);
                }
                ColumnRuleStyleComputedValue::Dotted => {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: rule_x + rule_w / 2.0,
                        y1: content_y,
                        x2: rule_x + rule_w / 2.0,
                        y2: content_y + content_h,
                        width: rule_w,
                        color: rule_color,
                        style: LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
                ColumnRuleStyleComputedValue::Dashed => {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: rule_x + rule_w / 2.0,
                        y1: content_y,
                        x2: rule_x + rule_w / 2.0,
                        y2: content_y + content_h,
                        width: rule_w,
                        color: rule_color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
                _ => {
                    // 其他样式退化为 solid
                    self.primitives
                        .add_fill(Rect::new(rule_x, content_y, rule_w, content_h), rule_color);
                }
            }
        }
    }

    /// 绘制单条边框（根据 border-style 生成合适的图元）。
    fn paint_border_edge(&mut self, spec: &BorderEdgeSpec, border_style: &BorderStyleValue, color: &ColorValue) {
        let render_color = color_value_to_render(color);

        match border_style {
            BorderStyleValue::None | BorderStyleValue::Hidden => {}
            BorderStyleValue::Solid => {
                self.primitives.add_fill(self.border_fill_rect(spec), render_color);
            }
            BorderStyleValue::Dotted => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1,
                    y1: spec.y1,
                    x2: spec.x2,
                    y2: spec.y2,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            BorderStyleValue::Dashed => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1,
                    y1: spec.y1,
                    x2: spec.x2,
                    y2: spec.y2,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
            }
            BorderStyleValue::Double => {
                let gap = (spec.thickness / 3.0).max(1.0);
                let line_w = ((spec.thickness - gap) / 2.0).max(1.0);
                if spec.is_horizontal {
                    self.primitives
                        .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, line_w), render_color);
                    self.primitives.add_fill(
                        Rect::new(spec.x1, spec.y1 + line_w + gap, spec.x2 - spec.x1, line_w),
                        render_color,
                    );
                } else {
                    let outer_x = if spec.extend_left {
                        spec.x1 - spec.thickness
                    } else {
                        spec.x1
                    };
                    self.primitives
                        .add_fill(Rect::new(outer_x, spec.y1, line_w, spec.y2 - spec.y1), render_color);
                    self.primitives.add_fill(
                        Rect::new(outer_x + line_w + gap, spec.y1, line_w, spec.y2 - spec.y1),
                        render_color,
                    );
                }
            }
            BorderStyleValue::Groove => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &light, &dark);
            }
            BorderStyleValue::Ridge => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &dark, &light);
            }
            BorderStyleValue::Inset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &darker, &lighter);
            }
            BorderStyleValue::Outset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &lighter, &darker);
            }
        }
    }

    /// 根据 BorderEdgeSpec 计算填充矩形。
    fn border_fill_rect(&self, spec: &BorderEdgeSpec) -> Rect {
        if spec.is_horizontal {
            Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, spec.thickness)
        } else if spec.extend_left {
            Rect::new(spec.x1 - spec.thickness, spec.y1, spec.thickness, spec.y2 - spec.y1)
        } else {
            Rect::new(spec.x1, spec.y1, spec.thickness, spec.y2 - spec.y1)
        }
    }

    /// 绘制 3D 效果边框（groove/ridge/inset/outset 使用）。
    ///
    /// 将边框分为上下两半（水平边）或左右两半（垂直边），分别使用不同颜色。
    fn paint_3d_border(&mut self, spec: &BorderEdgeSpec, first_color: &Color, second_color: &Color) {
        let half = spec.thickness / 2.0;
        if spec.is_horizontal {
            self.primitives
                .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, half), *first_color);
            self.primitives.add_fill(
                Rect::new(spec.x1, spec.y1 + half, spec.x2 - spec.x1, half),
                *second_color,
            );
        } else {
            let fill_x = if spec.extend_left {
                spec.x1 - spec.thickness
            } else {
                spec.x1
            };
            self.primitives
                .add_fill(Rect::new(fill_x, spec.y1, half, spec.y2 - spec.y1), *first_color);
            self.primitives.add_fill(
                Rect::new(fill_x + half, spec.y1, half, spec.y2 - spec.y1),
                *second_color,
            );
        }
    }

    /// 绘制 outline（位于 border 外侧，支持多种 outline-style）。
    ///
    /// outline 绘制为 4 条边框段，根据 outline-style 生成不同图元类型。
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

        // 计算外侧矩形坐标
        let ox = abs_x - total_offset;
        let oy = abs_y - total_offset;
        let ow = w + 2.0 * total_offset;
        let oh = h + 2.0 * total_offset;

        match style.outline_style {
            OutlineStyleValue::None => {}
            OutlineStyleValue::Solid => {
                // 实线：4 个填充矩形
                // 上
                self.primitives.add_fill(Rect::new(ox, oy, ow, outline_width), color);
                // 下
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, outline_width), color);
                // 左
                self.primitives.add_fill(
                    Rect::new(ox, oy + outline_width, outline_width, oh - 2.0 * outline_width),
                    color,
                );
                // 右
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        outline_width,
                        oh - 2.0 * outline_width,
                    ),
                    color,
                );
            }
            OutlineStyleValue::Dotted => {
                // 点线：4 条圆头点线描边
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                // 上
                self.primitives.add_stroke(StrokePrimitive {
                    x1: ox,
                    y1: mid_y_top,
                    x2: ox + ow,
                    y2: mid_y_top,
                    width: outline_width,
                    color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
                // 下
                self.primitives.add_stroke(StrokePrimitive {
                    x1: ox,
                    y1: mid_y_bottom,
                    x2: ox + ow,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
                // 左
                self.primitives.add_stroke(StrokePrimitive {
                    x1: mid_x_left,
                    y1: mid_y_top,
                    x2: mid_x_left,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
                // 右
                self.primitives.add_stroke(StrokePrimitive {
                    x1: mid_x_right,
                    y1: mid_y_top,
                    x2: mid_x_right,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            OutlineStyleValue::Dashed => {
                // 虚线：4 条方头虚线描边
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                // 上
                self.primitives.add_stroke(StrokePrimitive {
                    x1: ox,
                    y1: mid_y_top,
                    x2: ox + ow,
                    y2: mid_y_top,
                    width: outline_width,
                    color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
                // 下
                self.primitives.add_stroke(StrokePrimitive {
                    x1: ox,
                    y1: mid_y_bottom,
                    x2: ox + ow,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
                // 左
                self.primitives.add_stroke(StrokePrimitive {
                    x1: mid_x_left,
                    y1: mid_y_top,
                    x2: mid_x_left,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
                // 右
                self.primitives.add_stroke(StrokePrimitive {
                    x1: mid_x_right,
                    y1: mid_y_top,
                    x2: mid_x_right,
                    y2: mid_y_bottom,
                    width: outline_width,
                    color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
            }
            OutlineStyleValue::Double => {
                // 双线
                let gap = (outline_width / 3.0).max(1.0);
                let line_w = ((outline_width - gap) / 2.0).max(1.0);
                // 外线
                self.primitives.add_fill(Rect::new(ox, oy, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - line_w, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + line_w, line_w, oh - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ox + ow - line_w, oy + line_w, line_w, oh - 2.0 * line_w),
                    color,
                );
                // 内线
                let ix = ox + line_w + gap;
                let iy = oy + line_w + gap;
                let iw = ow - 2.0 * (line_w + gap);
                let ih = oh - 2.0 * (line_w + gap);
                self.primitives.add_fill(Rect::new(ix, iy, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + ih - line_w, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + line_w, line_w, ih - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ix + iw - line_w, iy + line_w, line_w, ih - 2.0 * line_w),
                    color,
                );
            }
            OutlineStyleValue::Groove | OutlineStyleValue::Ridge => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Groove) {
                    groove_ridge_colors(&color)
                } else {
                    let (l, d) = groove_ridge_colors(&color);
                    (d, l)
                };
                let half = outline_width / 2.0;
                // 上（外半 first，内半 second）
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                // 下
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                // 左
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                // 右
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
            OutlineStyleValue::Inset | OutlineStyleValue::Outset => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Inset) {
                    (darken(&color, 0.3), lighten(&color, 0.3))
                } else {
                    (lighten(&color, 0.3), darken(&color, 0.3))
                };
                let half = outline_width / 2.0;
                // 上
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                // 下
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                // 左
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                // 右
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
        }
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
    /// 支持 background-position（关键字、百分比、长度、双值组合）和
    /// background-size（auto、cover、contain、长度、百分比）。
    fn paint_background_image(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
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

        // 假设背景图片原始尺寸（无真实图片元数据时，使用容器尺寸）
        let img_w = origin_w;
        let img_h = origin_h;

        // 计算 background-size
        let (sized_w, sized_h) = resolve_background_size(&style.background_size, origin_w, origin_h, img_w, img_h);

        // 计算 background-position 偏移
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

    /// 绘制列表标记（disc/circle/square/decimal 等）。
    ///
    /// 检查当前 DOM 节点是否为 `<li>` 元素，且 list-style-type 非 None。
    /// 根据标记类型在内容区域左侧生成对应的图元。
    fn paint_list_marker(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        // 仅对 <li> 元素绘制标记
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // 检查是否为 li 元素
        match &node.kind {
            NodeKind::Element(elem) if elem.local_name() == "li" => {}
            _ => return,
        }

        // list-style-image 优先于 list-style-type
        match &style.list_style_image {
            zero_style_system::ListStyleImageComputedValue::Url(url) => {
                // 图片标记：使用 ImagePrimitive，尺寸约 1em
                let font_size: f32 = match style.font_size {
                    LengthValue::Px(s) => s as f32,
                    _ => 16.0,
                };
                let img_size = font_size;
                let marker_x = abs_x + box_node.border_left - img_size * 1.5;
                let marker_y = abs_y + box_node.border_top + box_node.padding_top;
                self.primitives.add_image(ImagePrimitive {
                    rect: Rect::new(marker_x, marker_y, img_size, img_size),
                    image_key: ImageKey::new(simple_hash(url)),
                });
                return;
            }
            zero_style_system::ListStyleImageComputedValue::None => {}
        }

        // 检查 list-style-type
        if style.list_style_type == ListStyleTypeValue::None {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => 16.0,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = color_value_to_render(&style.color);
        let default_font_id = FontId(0);

        // 标记位于内容区域左侧
        let marker_size = font_size * 0.4; // 标记符号大小
        let marker_x = abs_x + box_node.border_left; // 标记起始 x（border 内侧）
        let marker_y = abs_y + box_node.border_top + box_node.padding_top; // 与首行文本对齐

        // 根据 list-style-position 决定标记位置
        // Outside: 标记在内容区域左侧外
        // Inside: 标记在内容区域内部
        let actual_marker_x = match style.list_style_position {
            zero_css_parser::values::ListStylePositionValue::Outside => marker_x - marker_size * 2.5,
            zero_css_parser::values::ListStylePositionValue::Inside => marker_x + marker_size * 0.5,
        };

        match style.list_style_type {
            ListStyleTypeValue::Disc => {
                // 实心圆点：小填充矩形（近似圆形）
                self.primitives.add_fill(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                );
            }
            ListStyleTypeValue::Circle => {
                // 空心圆：使用描边矩形近似
                self.primitives.add_stroke(StrokePrimitive {
                    x1: actual_marker_x,
                    y1: marker_y + font_size * 0.3 - marker_size / 2.0 + marker_size / 2.0,
                    x2: actual_marker_x + marker_size,
                    y2: marker_y + font_size * 0.3 - marker_size / 2.0 + marker_size / 2.0,
                    width: marker_size,
                    color,
                    style: LineStyle::Solid,
                    cap: LineCap::Round,
                });
            }
            ListStyleTypeValue::Square => {
                // 方形标记
                self.primitives.add_fill(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                );
            }
            ListStyleTypeValue::Decimal | ListStyleTypeValue::DecimalLeadingZero => {
                // 计算列表项在兄弟中的索引
                let index = self.compute_list_item_index(doc, node_id);
                let text = if matches!(style.list_style_type, ListStyleTypeValue::DecimalLeadingZero) && index < 10 {
                    format!("0{index}.")
                } else {
                    format!("{index}.")
                };
                // 渲染数字标记为 glyph
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85);
                }
            }
            ListStyleTypeValue::LowerAlpha | ListStyleTypeValue::UpperAlpha => {
                let index = self.compute_list_item_index(doc, node_id);
                let ch = if index > 0 && index <= 26 {
                    let base = if matches!(style.list_style_type, ListStyleTypeValue::LowerAlpha) {
                        b'a'
                    } else {
                        b'A'
                    };
                    (base + (index - 1) as u8) as char
                } else {
                    '?'
                };
                let text = format!("{ch}.");
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85);
                }
            }
            ListStyleTypeValue::LowerRoman | ListStyleTypeValue::UpperRoman => {
                let index = self.compute_list_item_index(doc, node_id);
                let roman = to_roman(index);
                let text = if matches!(style.list_style_type, ListStyleTypeValue::LowerRoman) {
                    format!("{}.", roman.to_lowercase())
                } else {
                    format!("{roman}.")
                };
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85);
                }
            }
            ListStyleTypeValue::None => {}
        }
    }

    /// 计算当前列表项在其兄弟中的 1-based 索引。
    ///
    /// 遍历父节点的子元素，统计当前节点之前有多少个 <li> 兄弟。
    fn compute_list_item_index(&self, doc: &Document, node_id: NodeId) -> usize {
        let parent_id = match doc.parent_node(node_id) {
            Some(id) => id,
            None => return 1,
        };

        let mut index = 0;
        let mut found = false;
        for child_id in doc.child_nodes(parent_id) {
            if child_id == node_id {
                found = true;
                break;
            }
            if let Some(child) = doc.get(child_id)
                && let NodeKind::Element(elem) = &child.kind
                && elem.local_name() == "li"
            {
                index += 1;
            }
        }

        if found { index + 1 } else { 1 }
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

    // ═══════════════════════════════════════════════════════════════
    //  CSS mix-blend-mode 混合模式
    // ═══════════════════════════════════════════════════════════════

    /// 应用 CSS mix-blend-mode — 生成 BlendModePrimitive 标记元素区域需要混合。
    fn apply_blend_mode(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
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

    // ═══════════════════════════════════════════════════════════════
    //  CSS resize 调整大小手柄
    // ═══════════════════════════════════════════════════════════════

    /// 绘制 resize 手柄指示器 — 在元素右下角绘制三条小斜线。
    fn paint_resize_handle(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let handle_size = 8.0;
        let corner_x = abs_x + box_node.width - handle_size;
        let corner_y = abs_y + box_node.height - handle_size;

        // 手柄颜色：半透明灰色
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

/// Groove/Ridge 颜色对生成。
///
/// 返回 (亮色, 暗色)，亮色用于高光部分，暗色用于阴影部分。
fn groove_ridge_colors(color: &Color) -> (Color, Color) {
    let light = lighten(color, 0.3);
    let dark = darken(color, 0.3);
    (light, dark)
}

/// 使颜色变亮。
fn lighten(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 + (255.0 - color.r as f32) * amount).min(255.0) as u8,
        (color.g as f32 + (255.0 - color.g as f32) * amount).min(255.0) as u8,
        (color.b as f32 + (255.0 - color.b as f32) * amount).min(255.0) as u8,
        color.a,
    )
}

/// 使颜色变暗。
fn darken(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.g as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.b as f32 * (1.0 - amount)).min(255.0) as u8,
        color.a,
    )
}

/// 将数字转换为罗马数字字符串（1-based）。
fn to_roman(mut num: usize) -> String {
    if num == 0 {
        return "0".to_string();
    }
    let pairs = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for (value, symbol) in &pairs {
        while num >= *value {
            result.push_str(symbol);
            num -= value;
        }
    }
    result
}

fn has_direct_paintable_text(doc: &Document, node_id: NodeId) -> bool {
    doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}

// ── background-position / background-size 辅助函数 ─────────────────────────

/// 计算 background-size 后的图片尺寸。
///
/// 返回 (width, height) 像素值。
fn resolve_background_size(
    size: &BackgroundSizeComputedValue,
    container_w: f32,
    container_h: f32,
    img_w: f32,
    img_h: f32,
) -> (f32, f32) {
    match size {
        BackgroundSizeComputedValue::Auto => {
            // auto：保持原始图片尺寸（无真实元数据时等于容器尺寸）
            (img_w, img_h)
        }
        BackgroundSizeComputedValue::Cover => {
            // cover：缩放图片以完全覆盖容器，保持宽高比
            if img_w <= 0.0 || img_h <= 0.0 || container_w <= 0.0 || container_h <= 0.0 {
                return (container_w, container_h);
            }
            let scale_x = container_w / img_w;
            let scale_y = container_h / img_h;
            let scale = scale_x.max(scale_y);
            (img_w * scale, img_h * scale)
        }
        BackgroundSizeComputedValue::Contain => {
            // contain：缩放图片以完整显示在容器内，保持宽高比
            if img_w <= 0.0 || img_h <= 0.0 || container_w <= 0.0 || container_h <= 0.0 {
                return (container_w, container_h);
            }
            let scale_x = container_w / img_w;
            let scale_y = container_h / img_h;
            let scale = scale_x.min(scale_y);
            (img_w * scale, img_h * scale)
        }
        BackgroundSizeComputedValue::Length(px) => {
            // 长度值：指定宽度，高度自动保持宽高比
            let w = *px;
            let h = if img_w > 0.0 { w * img_h / img_w } else { container_h };
            (w, h)
        }
        BackgroundSizeComputedValue::Percent(pct) => {
            // 百分比：相对于容器尺寸
            let w = container_w * pct / 100.0;
            let h = if img_w > 0.0 { w * img_h / img_w } else { container_h };
            (w, h)
        }
    }
}

/// 将 background-position 单个分量解析为像素偏移。
///
/// `container_size` 是定位区域尺寸，`image_size` 是图片尺寸。
fn resolve_position_component(pos: &BackgroundPositionComputedValue, container_size: f32, image_size: f32) -> f32 {
    match pos {
        BackgroundPositionComputedValue::Left | BackgroundPositionComputedValue::Top => 0.0,
        BackgroundPositionComputedValue::Center => (container_size - image_size) / 2.0,
        BackgroundPositionComputedValue::Right | BackgroundPositionComputedValue::Bottom => {
            (container_size - image_size).max(0.0)
        }
        BackgroundPositionComputedValue::Length(px) => *px,
        BackgroundPositionComputedValue::Percent(pct) => {
            // CSS 百分比定位：offset = (container - image) * pct / 100
            (container_size - image_size) * pct / 100.0
        }
        BackgroundPositionComputedValue::TwoValue(_, _) => {
            // 双值嵌套不应出现在分量解析中，回退到 0
            0.0
        }
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
        // 单个值：水平方向按指定定位，垂直方向居中
        single => (
            resolve_position_component(single, container_w, img_w),
            resolve_position_component(&BackgroundPositionComputedValue::Center, container_h, img_h),
        ),
    }
}
