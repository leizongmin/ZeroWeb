//! 绘制命令生成器 — 模块拆分入口。
//!
//! Painter 结构体定义和核心绘制方法（递归遍历 + 背景绘制）。
//! 边框、效果、文本等子模块通过 `impl Painter` 扩展。

mod border;
mod effects;
mod effects_indicators;
mod text;

use std::collections::{HashMap, HashSet};

use zero_css_parser::values::{ColorValue, FloatValue, VisibilityValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{RenderPrimitives, RoundedRectPrimitive};
use zero_style_system::property::types::DisplayValue;
use zero_style_system::{
    AccentColorComputedValue, AppearanceComputedValue, BackgroundAttachmentComputedValue, BackgroundClipComputedValue,
    BorderCollapseValue, CaretColorComputedValue, ClipPathComputedValue, ComputedStyle, ContainComputedValue,
    HyphensComputedValue, ImageRenderingValue, IsolationValue, MixBlendModeComputedValue, OverscrollBehaviorValue,
    PointerEventsValue, QuotesComputedValue, ResizeValue, ScrollbarGutterComputedValue, TouchActionValue,
    UserSelectValue, WillChangeValue,
};

use super::color::color_value_to_render;
use super::helpers::{PrimitiveCounts, apply_opacity_to_new_primitives, circle_to_polygon, ellipse_to_polygon};

/// 绘制命令生成器 — 将布局盒树转换为渲染图元。
pub struct Painter {
    /// 生成的渲染图元列表。
    pub(crate) primitives: RenderPrimitives,
    /// 已由父级行内格式化上下文绘制过文本的节点。
    pub(crate) painted_inline_nodes: HashSet<NodeId>,
    /// CSS 计数器状态（计数器名 → 当前值）。
    pub(crate) counters: HashMap<String, i32>,
    /// 是否跳过属性指示器（用于 reftest 精确对比）。
    ///
    /// 指示器是绘制在元素边角的调试标记（如 border-collapse 橙色双线），
    /// 会干扰像素级 reftest 对比。设为 true 时跳过所有指示器。
    pub skip_indicators: bool,
    /// 图像固有尺寸缓存（image_key hash → (width, height)）。
    ///
    /// 用于 background-image 的 background-size: auto 计算。
    /// 在绘制开始前由调用方从 ImageCache 填充。
    pub image_sizes: HashMap<u64, (f32, f32)>,
    /// CSS font-family 查找表（字体族名 → FontId）。
    ///
    /// 由调用方从 FontLoader.build_font_resolver() 构建并传入。
    /// 用于将 CSS font-family 列表解析为具体的 FontId。
    font_resolver: HashMap<String, u32>,
    /// 视口宽度（像素）。用于 CSS §14.2 画布背景传播——根元素（html）的背景
    /// 覆盖整个画布；若根背景透明且 body 有背景，则 body 背景传播到画布。
    /// 由调用方（pipeline）在 paint 前设置；测试默认 0.0（不绘制画布背景）。
    pub viewport_w: f32,
    /// 视口高度（像素）。语义同 `viewport_w`。
    pub viewport_h: f32,
}

fn is_positioned_child(box_node: &LayoutBox) -> bool {
    box_node.is_absolute || box_node.is_fixed || box_node.is_relative || box_node.is_sticky
}

fn child_paint_sort_key(box_node: &LayoutBox) -> (u8, i32) {
    if is_positioned_child(box_node) {
        if box_node.z_index < 0 {
            // CSS 2.1 Appendix E step 2: negative z-index painted first
            (0, box_node.z_index)
        } else if box_node.creates_stacking_context {
            // CSS 2.1 Appendix E step 6/7: stacking context painted last
            (4, box_node.z_index)
        } else {
            // z-index: auto positioned (abspos/relative/fixed/sticky, 不建 SC)。
            // CSS 2.1 Appendix E step 6：positioned descendants with z-index:auto
            // paint AFTER normal flow (steps 3-5) 与 floats (step 4)，彼此按 tree order。
            // 旧值 (1,0)（与 in-flow 并列）致 abspos 先于 in-flow 兄弟绘制被覆盖
            // （absolute-replaced-width-006：img 被 div div 橙色背景覆写）。
            // 升到 (3,0)（在 in-flow (1) 与 float (2) 之后、SC (4) 之前）修正之，
            // 同时保留 abspos/relative 间的 tree order（top-019：#div2 红 abspos 先、
            // #div3 relative border 后覆盖→无红）。
            (3, 0)
        }
    } else if matches!(box_node.float, FloatValue::None) {
        (1, 0)
    } else {
        (2, 0)
    }
}

fn ordered_child_indices<F>(children: &[LayoutBox], mut include: F) -> Vec<usize>
where
    F: FnMut(&LayoutBox) -> bool,
{
    let mut ordered: Vec<usize> = children
        .iter()
        .enumerate()
        .filter_map(|(idx, child)| include(child).then_some(idx))
        .collect();
    ordered.sort_by(|&left, &right| {
        child_paint_sort_key(&children[left])
            .cmp(&child_paint_sort_key(&children[right]))
            .then(left.cmp(&right))
    });
    ordered
}

impl Painter {
    /// 创建新的绘制命令生成器。
    pub fn new() -> Self {
        Self {
            primitives: RenderPrimitives::new(),
            painted_inline_nodes: HashSet::new(),
            counters: HashMap::new(),
            skip_indicators: false,
            image_sizes: HashMap::new(),
            font_resolver: HashMap::new(),
            viewport_w: 0.0,
            viewport_h: 0.0,
        }
    }

    /// 设置 CSS font-family 查找表。
    ///
    /// 由调用方从 `FontLoader::build_font_resolver()` 构建并传入。
    pub fn set_font_resolver(&mut self, resolver: HashMap<String, u32>) {
        self.font_resolver = resolver;
    }

    /// 根据 CSS font-family 列表解析 FontId。
    ///
    /// 遍历 font-family 列表，返回第一个匹配的 FontId。
    /// 支持：
    /// - 具体字体族名（如 "Ahem", "DejaVu Sans"）
    /// - 通用字体族名（如 "sans-serif", "serif", "monospace"）
    /// - 回退到 FontId(0)（第一个加载的字体）
    pub(crate) fn resolve_font_id(&self, font_family: &[String]) -> zero_render_foundation::primitive::FontId {
        use zero_render_foundation::primitive::FontId;
        for family in font_family {
            // 去除引号
            let name = family.trim_matches('"').trim_matches('\'');
            if let Some(&id) = self.font_resolver.get(name) {
                return FontId(id);
            }
            // 大小写不敏感匹配（CSS font-family 不区分大小写）
            for (key, &id) in &self.font_resolver {
                if key.eq_ignore_ascii_case(name) {
                    return FontId(id);
                }
            }
        }
        FontId(0)
    }

    /// 绘制整个布局树。
    ///
    /// 遍历 LayoutBox 树，为每个有样式的节点生成背景和边框填充图元。
    /// 传入 `doc` 以启用行内格式化上下文的文本换行布局。
    pub fn paint(&mut self, layout: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, doc: Option<&Document>) {
        // CSS §14.2 画布背景传播：根元素（html）的背景色覆盖整个画布；若根背景透明
        // 且 body 有背景色，则 body 背景色传播到画布。在绘制树之前先填充画布背景，
        // 使整个视口（含 body margin / 超出根盒的区域）呈现该背景色。根/body 自身
        // 背景仍照常绘制（同色叠加，无可见重绘）。
        if self.viewport_w > 0.0
            && self.viewport_h > 0.0
            && let Some(doc) = doc
        {
            let mut canvas_color: Option<zero_style_system::property::types::ColorValue> = None;
            // 根元素 html 的背景色优先；透明则取 body 背景色。
            let html_id = doc.get_elements_by_tag_name("html").into_iter().next();
            if let Some(hid) = html_id
                && let Some(hs) = styles.get(&hid)
                && hs.background_color != zero_style_system::property::types::ColorValue::Transparent
            {
                canvas_color = Some(hs.background_color.clone());
            }
            if canvas_color.is_none()
                && let Some(bid) = doc.get_elements_by_tag_name("body").into_iter().next()
                && let Some(bs) = styles.get(&bid)
                && bs.background_color != zero_style_system::property::types::ColorValue::Transparent
            {
                canvas_color = Some(bs.background_color.clone());
            }
            if let Some(c) = canvas_color {
                self.primitives.add_fill(
                    Rect::new(0.0, 0.0, self.viewport_w, self.viewport_h),
                    color_value_to_render(&c),
                );
            }
        }
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

        let is_hidden = if box_node.is_anonymous_text_item {
            // 匿名文本项（flex/grid 容器中的文本节点）
            if let Some(doc) = doc
                && let Some(node_id) = box_node.node_id
            {
                let parent_style = doc
                    .parent_node(node_id)
                    .and_then(|pid| styles.get(&pid).cloned())
                    .unwrap_or_default();
                if !matches!(
                    parent_style.visibility,
                    VisibilityValue::Hidden | VisibilityValue::Collapse
                ) {
                    self.paint_anonymous_text_item(box_node, abs_x, abs_y, &parent_style, doc, node_id);
                }
            }
            false
        } else if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let hidden = matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse);

            let skip_empty_cell = matches!(style.empty_cells, zero_style_system::EmptyCellsComputedValue::Hide)
                && box_node.children.is_empty()
                && matches!(style.border_collapse, zero_style_system::BorderCollapseValue::Separate);

            // CSS 2.1 §17.5.3/17.5.4：行组和行无 border/padding/margin，但 background 仍应用
            let is_table_internal = matches!(
                style.display,
                DisplayValue::TableRowGroup
                    | DisplayValue::TableHeaderGroup
                    | DisplayValue::TableFooterGroup
                    | DisplayValue::TableRow
            );

            if !hidden && !skip_empty_cell {
                if !is_table_internal {
                    self.paint_box_shadow(box_node, abs_x, abs_y, style);
                }
                if style.background_color != ColorValue::Transparent {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }
                self.paint_background_image(box_node, abs_x, abs_y, style);
                // border-collapse: collapse 时，表格外边框由边缘单元格绘制，
                // 表格元素本身不绘制边框（避免与单元格边框重叠）
                let is_collapsed_table = matches!(style.display, DisplayValue::Table | DisplayValue::InlineTable)
                    && matches!(style.border_collapse, BorderCollapseValue::Collapse);
                if (box_node.border_top > 0.0
                    || box_node.border_right > 0.0
                    || box_node.border_bottom > 0.0
                    || box_node.border_left > 0.0)
                    && !is_collapsed_table
                {
                    self.paint_borders(box_node, abs_x, abs_y, style);
                }
                if !is_table_internal {
                    self.paint_outline(box_node, abs_x, abs_y, style);
                }
            }

            if !hidden {
                if let Some(doc) = doc {
                    self.paint_list_marker(box_node, abs_x, abs_y, style, doc);
                }
                self.paint_text(box_node, abs_x, abs_y, style, doc, Some(styles));
            }

            hidden
        } else {
            false
        };

        // CSS Tables §17.5.3 列背景：<col>/<colgroup> 的 background-color 在单元格之下、
        // 按列跨满表格高度绘制（几何由 layout 层 collect_table_col_backgrounds 写入）。
        if !is_hidden {
            self.paint_table_col_backgrounds(box_node, abs_x, abs_y, styles);
        }

        let child_offset_x = abs_x + box_node.padding_left + box_node.border_left;
        let child_offset_y = abs_y + box_node.padding_top + box_node.border_top;

        let counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

        // CSS 2.1 Appendix E:
        // 负 z-index 的 positioned 后代在常规流内容之后方，
        // 非 positioned float 在常规流后代之上，
        // 非负 z-index 的 positioned 后代位于最上层。
        for child_idx in ordered_child_indices(&box_node.children, |_| true) {
            let child = &box_node.children[child_idx];
            self.paint_node_in_rect(child, styles, child_offset_x, child_offset_y, dirty_rect, doc);
        }

        if needs_clip {
            let clip_rect = Rect::new(
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            );
            super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before_children, &clip_rect);
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

        // 判断是否需要裁剪子内容（overflow 或 contain:paint 触发）
        let needs_clip = if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            box_node.overflow_x != OverflowClip::Visible
                || box_node.overflow_y != OverflowClip::Visible
                || matches!(
                    style.contain,
                    ContainComputedValue::Paint
                        | ContainComputedValue::Strict
                        | ContainComputedValue::Content
                        | ContainComputedValue::Custom(_)
                )
        } else {
            box_node.overflow_x != OverflowClip::Visible || box_node.overflow_y != OverflowClip::Visible
        };

        // 获取该节点对应的计算样式
        // 记录绘制前的图元数量，用于 opacity 应用
        let counts_before = PrimitiveCounts::snapshot(&self.primitives);

        let is_hidden = if box_node.is_anonymous_text_item {
            // 匿名文本项（flex/grid 容器中的文本节点）
            if let Some(doc) = doc
                && let Some(node_id) = box_node.node_id
            {
                let parent_style = doc
                    .parent_node(node_id)
                    .and_then(|pid| styles.get(&pid).cloned())
                    .unwrap_or_default();
                if !matches!(
                    parent_style.visibility,
                    VisibilityValue::Hidden | VisibilityValue::Collapse
                ) {
                    self.paint_anonymous_text_item(box_node, abs_x, abs_y, &parent_style, doc, node_id);
                }
            }
            false
        } else if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let hidden = matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse);

            // empty-cells:hide — 空表格单元格不绘制背景和边框
            let skip_empty_cell = matches!(style.empty_cells, zero_style_system::EmptyCellsComputedValue::Hide)
                && box_node.children.is_empty()
                && matches!(style.border_collapse, zero_style_system::BorderCollapseValue::Separate);

            // CSS 2.1 §17.5.3/17.5.4：行组（tbody/thead/tfoot）和行（tr）
            // 的 border/padding/margin 无视觉效果，但 background 仍然应用。
            // Layout 层 zero_box_model() 已归零 border/padding/margin（阻止边框绘制），
            // paint 层需跳过 box-shadow 和 outline 等依赖盒模型的装饰效果。
            let is_table_internal = matches!(
                style.display,
                DisplayValue::TableRowGroup
                    | DisplayValue::TableHeaderGroup
                    | DisplayValue::TableFooterGroup
                    | DisplayValue::TableRow
            );

            if !hidden && !skip_empty_cell {
                // R109 §9.2.1.1：被拆分的 inline 父盒自身不绘制盒装饰（背景/边框/阴影）——
                // 其 border/background 已下放到匿名块片段（带 fragment_node_ids），由片段
                // 收缩到文本宽后绘制。父盒只作为结构包裹，绘制装饰会画全宽（错）。
                let skip_split_inline_deco = box_node.is_r109_split && box_node.fragment_node_ids.is_none();

                // -1. backdrop-filter（对元素背后内容应用滤镜，在自身绘制之前）
                if !is_table_internal {
                    self.apply_backdrop_filter(box_node, abs_x, abs_y, style);
                }

                // 0. box-shadow（位于背景之下，行组/行无盒模型故无阴影）
                if !is_table_internal && !skip_split_inline_deco {
                    self.paint_box_shadow(box_node, abs_x, abs_y, style);
                }

                // 1. 背景色填充（行组/行仍可渲染背景）
                if style.background_color != ColorValue::Transparent && !skip_split_inline_deco {
                    self.paint_background(box_node, abs_x, abs_y, style);
                }

                // 1b. 背景图片（行组/行仍可渲染背景图片）
                if !skip_split_inline_deco {
                    self.paint_background_image(box_node, abs_x, abs_y, style);
                }

                // 2. 边框填充（zero_box_model 已归零，但保留防护检查）
                if !skip_split_inline_deco
                    && (box_node.border_top > 0.0
                        || box_node.border_right > 0.0
                        || box_node.border_bottom > 0.0
                        || box_node.border_left > 0.0)
                {
                    self.paint_borders(box_node, abs_x, abs_y, style);
                }

                // 2b. Border-image 绘制（替换或覆盖常规边框）
                if !skip_split_inline_deco {
                    self.paint_border_image(box_node, abs_x, abs_y, style);
                }

                // 2c. Column-rule 绘制（多列之间的分隔线）
                self.paint_column_rules(box_node, abs_x, abs_y, style);

                // 3. Outline 绘制（位于 border 外侧）
                self.paint_outline(box_node, abs_x, abs_y, style);

                // 3b. clip-path 视觉指示器（仅用于未实现的非矩形形状）
                // 注意：inset() 已在下方应用实际裁剪，此处仅绘制其他形状的指示线
                if !matches!(
                    style.clip_path,
                    ClipPathComputedValue::None | ClipPathComputedValue::Inset { .. }
                ) {
                    self.paint_clip_path(box_node, abs_x, abs_y, style);
                }
            }

            // 列表标记和文本始终绘制（不受 empty-cells 影响）
            if !hidden {
                // 4. 列表标记绘制（bullets/numbers，位于文本之前）
                if let Some(doc) = doc {
                    self.paint_list_marker(box_node, abs_x, abs_y, style, doc);

                    // 4b. <img> 元素绘制（含 object-fit）
                    self.paint_img_element(box_node, abs_x, abs_y, style, doc);
                }

                // 4c. CSS `content` 属性生成的文本（在普通文本之前）
                self.paint_content(box_node, abs_x, abs_y, style);

                // 5. 文本内容绘制（含 text-shadow，使用行内格式化上下文处理换行）
                self.paint_text(box_node, abs_x, abs_y, style, doc, Some(styles));
            }

            hidden
        } else {
            false
        };

        // 6. 递归绘制子节点（子节点偏移 = 父 padding + border）
        // visibility: hidden 不阻止子节点绘制，子节点可以覆盖为 visible
        let mut child_offset_x = abs_x + box_node.padding_left + box_node.border_left;
        let mut child_offset_y = abs_y + box_node.padding_top + box_node.border_top;

        // 滚动容器：将子元素向上/左偏移 scroll_x/scroll_y
        // scroll_y > 0 表示内容已向下滚动，因此子元素需要向上移动
        if matches!(box_node.overflow_x, OverflowClip::Scroll) {
            child_offset_x -= box_node.scroll_x;
        }
        if matches!(box_node.overflow_y, OverflowClip::Scroll) {
            child_offset_y -= box_node.scroll_y;
        }

        // 5b. CSS 计数器处理（在子节点绘制前，按 reset → set → increment 顺序）
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            self.update_counters(style);
        }

        // CSS Tables §17.5.3 列背景：<col>/<colgroup> 的 background-color 在单元格之下、
        // 按列跨满表格高度绘制（几何由 layout 层 collect_table_col_backgrounds 写入）。
        if !is_hidden {
            self.paint_table_col_backgrounds(box_node, abs_x, abs_y, styles);
        }

        // 记录子节点绘制前的图元数量，用于裁剪
        let counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

        // CSS §11.1.1：overflow 仅裁剪 CB 为本元素或其后代的 positioned 后代。
        // 当 overflow 元素自身**非 positioned** 时，其 abspos 后代的 CB 必为祖先
        //（或中间有 positioned 后代）。常见情形（overflow 元素非 positioned、
        // abspos 与 overflow 之间无 positioned 元素）下，abspos 的 CB 在 overflow 之上，
        // 不应被本 overflow 裁剪。ZeroWeb 原先把 abspos 当普通子元素绘制被误裁。
        // 此处对「非 positioned 的 overflow 元素」把 abspos/fixed 子元素移到 overflow
        // 裁剪之后绘制（positioned overflow 元素保持原行为，避免 z-order 回归）。
        let is_multicol = box_node.is_multicol;
        let self_positioned = box_node.is_absolute || box_node.is_fixed || box_node.is_relative || box_node.is_sticky;
        let defer_abspos = needs_clip && !self_positioned && !is_multicol;
        for child_idx in ordered_child_indices(&box_node.children, |child| {
            (!is_multicol || child.column_span_offsets.is_empty())
                && (!defer_abspos || (!child.is_absolute && !child.is_fixed))
        }) {
            let child = &box_node.children[child_idx];
            self.paint_node(child, styles, child_offset_x, child_offset_y, doc);
        }

        // 多列子元素按列区域渲染。
        // 每个子元素在分配到的列位置渲染，裁剪到「列宽度 + 右半间隙」范围，
        // 允许内容延伸到列间隙但不进入相邻列。
        // 对于 column breaking 的子元素（多个片段），每个片段额外裁剪到列高。
        if is_multicol {
            let content_x = abs_x + box_node.border_left + box_node.padding_left;
            let content_y = abs_y + box_node.border_top + box_node.padding_top;

            // 获取列间距用于扩展裁剪区域
            // 优先使用 layout 层存储的 column_gap，回退到从 column_span_offsets 推算
            let gap = if box_node.column_gap > 0.0 {
                box_node.column_gap
            } else {
                box_node
                    .children
                    .iter()
                    .find_map(|c| {
                        if c.column_span_offsets.len() >= 2 {
                            let (first_x, _, _, first_w) = c.column_span_offsets[0];
                            let (second_x, _, _, _) = c.column_span_offsets[1];
                            Some(second_x - first_x - first_w)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0)
            };

            for child in &box_node.children {
                if child.column_span_offsets.is_empty() {
                    continue;
                }

                let is_breaking = child.column_span_offsets.len() > 1;

                for &(frag_x, frag_y, col_x, col_w) in &child.column_span_offsets {
                    let frag_abs_x = content_x + frag_x;
                    let frag_abs_y = content_y + frag_y;

                    // 裁剪区域：列宽 + 右半间隙，允许内容延伸到间隙
                    // 对于 breaking 子元素，还裁剪到列高度
                    let clip_w = col_w + gap / 2.0;
                    let clip_h = if is_breaking {
                        // breaking 子元素裁剪到列高度，显示对应片段
                        box_node.content_height
                    } else {
                        // 非 breaking 子元素不裁剪高度
                        box_node.content_height + 1000.0 // 足够大的值
                    };
                    let clip_rect = Rect::new(content_x + col_x, content_y, clip_w, clip_h);

                    let counts_before_frag = PrimitiveCounts::snapshot(&self.primitives);

                    let frag_offset_x = frag_abs_x - child.x;
                    let frag_offset_y = frag_abs_y - child.y;
                    self.paint_node(child, styles, frag_offset_x, frag_offset_y, doc);

                    super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before_frag, &clip_rect);
                }
            }
        }

        // 如果需要裁剪，将子节点产生的图元裁剪到内容盒范围内
        if needs_clip {
            let clip_rect = Rect::new(
                abs_x + box_node.border_left + box_node.padding_left,
                abs_y + box_node.border_top + box_node.padding_top,
                box_node.content_width,
                box_node.content_height,
            );
            super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before_children, &clip_rect);
        }

        // 非 positioned overflow 元素：abspos/fixed 子元素移到裁剪之后绘制，
        // 使其 CB 为祖先时不被本 overflow 误裁（CSS §11.1.1）。
        if defer_abspos {
            for child_idx in ordered_child_indices(&box_node.children, |child| child.is_absolute || child.is_fixed) {
                let child = &box_node.children[child_idx];
                self.paint_node(child, styles, child_offset_x, child_offset_y, doc);
            }
        }

        // clip-path: inset() — 实际裁剪（对元素及其所有子元素的图元应用矩形裁剪）
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            match &style.clip_path {
                ClipPathComputedValue::Inset {
                    top,
                    right,
                    bottom,
                    left,
                    ..
                } => {
                    let w = box_node.width;
                    let h = box_node.height;
                    let t = super::helpers::length_to_f32(top);
                    let r = super::helpers::length_to_f32(right);
                    let b = super::helpers::length_to_f32(bottom);
                    let l = super::helpers::length_to_f32(left);
                    let clip_rect = Rect::new(abs_x + l, abs_y + t, w - l - r, h - t - b);
                    super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before, &clip_rect);
                }
                ClipPathComputedValue::Circle { radius, position } => {
                    let w = box_node.width;
                    let h = box_node.height;
                    let r = match radius {
                        zero_style_system::ClipPathRadius::Length(l) => super::helpers::length_to_f32(l),
                        zero_style_system::ClipPathRadius::ClosestSide => {
                            let cx = position
                                .as_ref()
                                .map(|(x, _)| super::helpers::length_to_f32(x))
                                .unwrap_or(w / 2.0);
                            let cy = position
                                .as_ref()
                                .map(|(_, y)| super::helpers::length_to_f32(y))
                                .unwrap_or(h / 2.0);
                            cx.min(w - cx).min(cy.min(h - cy))
                        }
                        zero_style_system::ClipPathRadius::FarthestSide => {
                            let cx = position
                                .as_ref()
                                .map(|(x, _)| super::helpers::length_to_f32(x))
                                .unwrap_or(w / 2.0);
                            let cy = position
                                .as_ref()
                                .map(|(_, y)| super::helpers::length_to_f32(y))
                                .unwrap_or(h / 2.0);
                            cx.max(w - cx).max(cy.max(h - cy))
                        }
                    };
                    let cx = position
                        .as_ref()
                        .map(|(x, _)| super::helpers::length_to_f32(x))
                        .unwrap_or(w / 2.0);
                    let cy = position
                        .as_ref()
                        .map(|(_, y)| super::helpers::length_to_f32(y))
                        .unwrap_or(h / 2.0);
                    let polygon = circle_to_polygon(abs_x + cx, abs_y + cy, r, 24);
                    super::helpers::clip_all_primitives_to_polygon(&mut self.primitives, &counts_before, &polygon);
                }
                ClipPathComputedValue::Ellipse { rx, ry, position } => {
                    let w = box_node.width;
                    let h = box_node.height;
                    let rx_v = match rx {
                        zero_style_system::ClipPathRadius::Length(l) => super::helpers::length_to_f32(l),
                        _ => w / 2.0,
                    };
                    let ry_v = match ry {
                        zero_style_system::ClipPathRadius::Length(l) => super::helpers::length_to_f32(l),
                        _ => h / 2.0,
                    };
                    let cx = position
                        .as_ref()
                        .map(|(x, _)| super::helpers::length_to_f32(x))
                        .unwrap_or(w / 2.0);
                    let cy = position
                        .as_ref()
                        .map(|(_, y)| super::helpers::length_to_f32(y))
                        .unwrap_or(h / 2.0);
                    let polygon = ellipse_to_polygon(abs_x + cx, abs_y + cy, rx_v, ry_v, 24);
                    super::helpers::clip_all_primitives_to_polygon(&mut self.primitives, &counts_before, &polygon);
                }
                ClipPathComputedValue::Polygon { points, .. } => {
                    let polygon: Vec<(f32, f32)> = points
                        .iter()
                        .map(|(x, y)| {
                            (
                                abs_x + super::helpers::length_to_f32(x),
                                abs_y + super::helpers::length_to_f32(y),
                            )
                        })
                        .collect();
                    if polygon.len() >= 3 {
                        super::helpers::clip_all_primitives_to_polygon(&mut self.primitives, &counts_before, &polygon);
                    }
                }
                _ => {}
            }
        }

        // CSS clip: rect() — 仅对绝对定位元素生效的矩形裁剪
        if box_node.is_absolute
            && let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && let zero_css_parser::values::ClipRectValue::Rect(top, right, bottom, left) = &style.clip
        {
            let t = super::helpers::length_to_f32(top);
            let r = super::helpers::length_to_f32(right);
            let b = super::helpers::length_to_f32(bottom);
            let l = super::helpers::length_to_f32(left);
            // clip: rect() 坐标相对于元素的边框盒
            let clip_rect = Rect::new(abs_x + l, abs_y + t, r - l, b - t);
            super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before, &clip_rect);
        }

        // CSS filter — 对元素及其子元素产生的图元应用滤镜效果
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            self.apply_filter(box_node, abs_x, abs_y, style);
        }

        // CSS transform — 为含 rotate/scale/skew 的元素生成 TransformPrimitive
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            let rect = Rect::new(abs_x, abs_y, box_node.width, box_node.height);
            super::helpers::apply_transform(style, &rect, &mut self.primitives);
        }

        // 应用 opacity（对当前节点及其子节点产生的所有图元进行 alpha 衰减）
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && style.opacity < 1.0
        {
            let opacity = style.opacity as f32;
            apply_opacity_to_new_primitives(&mut self.primitives, &counts_before, opacity);
        }

        // CSS mask-image — 对元素及其子元素应用蒙版裁剪
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && !style.mask_image.is_empty()
        {
            self.apply_mask_image(box_node, abs_x, abs_y, style, &counts_before);
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

        // ── 属性指示器（reftest 模式下跳过，避免干扰像素对比）──
        if !self.skip_indicators {
            // CSS accent-color — 绘制强调色指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.accent_color, AccentColorComputedValue::Auto)
            {
                self.paint_accent_color_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS caret-color — 绘制光标颜色指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.caret_color, CaretColorComputedValue::Auto)
            {
                self.paint_caret_color_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS scrollbar-width — 绘制滚动条指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_scrollbar_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS appearance — 绘制原生控件外观
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(
                    style.appearance,
                    AppearanceComputedValue::None | AppearanceComputedValue::Auto
                )
            {
                self.paint_appearance(box_node, abs_x, abs_y, style);
            }

            // CSS scrollbar-gutter — 预留滚动条空间
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Auto)
            {
                self.paint_scrollbar_gutter(box_node, abs_x, abs_y, style);
            }

            // CSS background-attachment: fixed — 固定背景指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.background_attachment, BackgroundAttachmentComputedValue::Fixed)
            {
                self.paint_background_attachment_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS hyphens: auto — 连字符指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.hyphens, HyphensComputedValue::Auto)
            {
                self.paint_hyphens_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS quotes — 引号标记
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.quotes, QuotesComputedValue::Pairs(_))
            {
                self.paint_quotes(box_node, abs_x, abs_y, style, 0);
            }

            // CSS cursor — 光标类型指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_cursor_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS image-rendering — 图片质量指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.image_rendering, ImageRenderingValue::Auto)
            {
                self.paint_image_rendering_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS isolation: isolate — 堆叠上下文指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.isolation, IsolationValue::Isolate)
            {
                self.paint_isolation_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS will-change — 性能提示指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.will_change, WillChangeValue::Auto)
            {
                self.paint_will_change_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS pointer-events: none — 点击穿透指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.pointer_events, PointerEventsValue::None)
            {
                self.paint_pointer_events_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS user-select: none — 文本不可选择指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && matches!(style.user_select, UserSelectValue::None)
            {
                self.paint_user_select_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS overscroll-behavior — 滚动边界限制指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(style.overscroll_behavior_x, OverscrollBehaviorValue::Auto)
            {
                self.paint_overscroll_behavior_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS touch-action — 触摸行为指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && !matches!(
                    style.touch_action,
                    TouchActionValue::Auto | TouchActionValue::Manipulation
                )
            {
                self.paint_touch_action_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS scroll-snap — 吸附轴和对齐点指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::property::types::ScrollSnapStrictness;
                if !matches!(style.scroll_snap_type.strictness, ScrollSnapStrictness::None) {
                    self.paint_scroll_snap_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS perspective — 3D 透视上下文指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::property::types::LengthValue;
                if let LengthValue::Px(v) = style.perspective
                    && v > 0.0
                {
                    self.paint_perspective_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS backface-visibility: hidden — 背面不可见指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::property::types::BackfaceVisibilityValue;
                if matches!(style.backface_visibility, BackfaceVisibilityValue::Hidden) {
                    self.paint_backface_visibility_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS transform-style: preserve-3d — 3D 渲染上下文指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::property::types::TransformStyleValue;
                if matches!(style.transform_style, TransformStyleValue::Preserve3d) {
                    self.paint_transform_style_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS border-spacing — 表格单元格间距指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
                && (style.border_spacing.horizontal > 0.0 || style.border_spacing.vertical > 0.0)
            {
                self.paint_border_spacing_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS caption-side — 表格标题位置指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::property::types::CaptionSideValue;
                if matches!(style.caption_side, CaptionSideValue::Bottom) {
                    self.paint_caption_side_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS direction — 文本方向指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_direction_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS tab-size — 制表符宽度指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_tab_size_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS border-collapse — 边框合并指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_border_collapse_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS table-layout — 表格布局模式指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_table_layout_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS font-variant-numeric — 数字变体指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_font_variant_numeric_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS contain — 包含指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::ContainComputedValue;
                if !matches!(style.contain, ContainComputedValue::None) {
                    self.paint_contain_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS unicode-bidi — 双向文本覆盖指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                use zero_style_system::UnicodeBidiValue;
                if !matches!(style.unicode_bidi, UnicodeBidiValue::Normal) {
                    self.paint_unicode_bidi_indicator(box_node, abs_x, abs_y, style);
                }
            }

            // CSS box-decoration-break — 装饰断行指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_box_decoration_break_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS overflow-wrap — 断词模式指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_overflow_wrap_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS text-align-last — 末行对齐指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_text_align_last_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS break-before/after/inside + page-break-* — 断点指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_break_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS scroll-margin/padding — 滚动吸附区域指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_scroll_area_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS scroll-snap-stop:always — 强制停止标记
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_scroll_snap_stop_indicator(box_node, abs_x, abs_y, style);
            }

            // CSS container-type — 容器查询上下文指示器
            if let Some(node_id) = box_node.node_id
                && let Some(style) = styles.get(&node_id)
            {
                self.paint_container_type_indicator(box_node, abs_x, abs_y, style);
            }
        } // end skip_indicators guard

        let _ = is_hidden; // visibility 在 if let 块内处理
    }

    /// 更新 CSS 计数器状态（reset → set → increment 顺序）。
    pub(crate) fn update_counters(&mut self, style: &ComputedStyle) {
        use zero_css_parser::values::CounterActionValue;

        // 1. counter-reset — 重置计数器为指定值（默认 0）
        for CounterActionValue { name, value } in &style.counter_reset {
            let v = value.unwrap_or(0);
            self.counters.insert(name.clone(), v);
        }

        // 2. counter-set — 直接设置计数器值（不创建新作用域）
        for CounterActionValue { name, value } in &style.counter_set {
            let v = value.unwrap_or(0);
            self.counters.insert(name.clone(), v);
        }

        // 3. counter-increment — 递增计数器（默认 +1）
        for CounterActionValue { name, value } in &style.counter_increment {
            let v = value.unwrap_or(1);
            *self.counters.entry(name.clone()).or_insert(0) += v;
        }
    }

    /// 获取指定计数器的当前值。
    pub fn get_counter(&self, name: &str) -> Option<i32> {
        self.counters.get(name).copied()
    }

    /// 绘制表格列背景（CSS Tables §17.5.3）。
    ///
    /// `<col>`/`<colgroup>` 不生成常规流盒，其 `background-color` 须按列跨满表格
    /// content 高度绘制，位于单元格背景之下。几何 `(node_id, x_offset, width)` 相对
    /// 表格 content box（由 layout 层 `collect_table_col_backgrounds` 写入）。
    /// 仅绘制 background-color（col 上 background-image 极罕见，暂不支持）。
    fn paint_table_col_backgrounds(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) {
        if box_node.table_col_backgrounds.is_empty() {
            return;
        }
        let content_x = abs_x + box_node.padding_left + box_node.border_left;
        let content_y = abs_y + box_node.padding_top + box_node.border_top;
        let h = box_node.content_height;
        if h <= 0.0 {
            return;
        }
        for (node_id, x_off, w) in &box_node.table_col_backgrounds {
            let Some(style) = styles.get(node_id) else { continue };
            if matches!(style.background_color, ColorValue::Transparent) {
                continue;
            }
            if *w <= 0.0 {
                continue;
            }
            self.primitives.add_fill(
                Rect::new(content_x + *x_off, content_y, *w, h),
                color_value_to_render(&style.background_color),
            );
        }
    }

    /// 绘制背景（考虑 border-radius）。
    ///
    /// 当 border-radius 为零时退化为普通矩形填充。
    fn paint_background(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let radii = super::helpers::BorderRadiusSpec::from_style(style);

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
            // 圆角矩形：通过 add_rounded_rect 记录 DrawOp（draw_order 是默认渲染路径，
            // 直接 push 到 rounded_rects 会绕过 DrawOp 记录导致圆角背景被丢弃）。
            self.primitives.add_rounded_rect(RoundedRectPrimitive {
                rect: Rect::new(clip_x, clip_y, clip_w, clip_h),
                color: color_value_to_render(&style.background_color),
                top_left_radius: radii.top_left,
                top_right_radius: radii.top_right,
                bottom_right_radius: radii.bottom_right,
                bottom_left_radius: radii.bottom_left,
            });
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

    /// 查找图像固有尺寸。
    ///
    /// 通过 URL 的 hash 值查找图像的 (width, height)。
    /// 如果未找到，返回 None。
    pub(crate) fn get_image_size(&self, url_hash: u64) -> Option<(f32, f32)> {
        self.image_sizes.get(&url_hash).copied()
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}
