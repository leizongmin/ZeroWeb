//! 绘制命令生成器 — 模块拆分入口。
//!
//! Painter 结构体定义和核心绘制方法（递归遍历 + 背景绘制）。
//! 边框、效果、文本等子模块通过 `impl Painter` 扩展。

mod border;
mod effects;
mod effects_indicators;
mod text;

use std::collections::{HashMap, HashSet};

use zero_css_parser::values::{ColorValue, FloatValue, LengthValue, VisibilityValue};
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
    /// CSS §14.2 画布背景传播：背景传播到画布的元素 NodeId（html 或 body）。
    /// 该元素的背景（color + image）由 `paint()` 在画布上统一绘制；`paint_background_image`
    /// 跳过该元素自身的图像绘制，避免其 padding-box 起始的图像与画布 (0,0) 起始的图像
    /// 相位错位 double-paint（R507：扩展 R491 的 color-only 传播到含 image）。
    pub(crate) canvas_propagated_node: Option<NodeId>,
    /// R639：NodeId → LayoutBox.height 索引（paint() 开头预扫描布局树填充）。
    /// render_fragment 宏处理某 inline 片段时，box_node 是 **IFC owner**（其文本所在
    /// 容器）而非 inline 本身；为使 per-fragment bg 门控与 paint_node 抑制（在 inline 自身
    /// box 上）一致，宏须用 inline 元素自身 height（经此索引查 owner_id），而非 IFC owner
    /// 的 box_node.height（R638 锁定的 inline-ownership split 修复）。
    pub(crate) inline_heights: HashMap<NodeId, f32>,
    /// 当前文档 URL（解析相对 `<img src>`）。
    pub(crate) document_url: Option<String>,
}

fn is_positioned_child(box_node: &LayoutBox) -> bool {
    box_node.is_absolute || box_node.is_fixed || box_node.is_relative || box_node.is_sticky
}

/// 画布背景传播判定：background-image 图层列表是否含至少一个**可绘制**图层
///（Url 或 Gradient）。`background-image: none` 解析为 `vec![None]`（非空但无实际
/// 图层），不应触发画布传播（CSS §14.2，R879）。
fn has_paintable_bg_image(layers: &[zero_style_system::property::types::BackgroundImageComputedValue]) -> bool {
    layers.iter().any(|l| {
        !matches!(
            l,
            zero_style_system::property::types::BackgroundImageComputedValue::None
        )
    })
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
            // CSS 2.1 Appendix E step 6：主循环（paint_node 步 3/4/5）经 is_positioned_child
            // 过滤**排除**此类子元素，改由其所属 scope（positioned 祖先或根）的
            // collect_positioned_descendants 按 tree order 收集，于 normal flow 之后、正 z-index
            // SC 之前统一 flush（详见 paint_node 步 2/6/7）。此 (3,0) 不再驱动主循环排序，
            // 但仍参与 defer_abspos 等子循环的排序（auto-positioned 排在 real-SC (4,z) 之前）。
            (3, 0)
        }
    } else if matches!(box_node.float, FloatValue::None) {
        (1, 0)
    } else {
        (2, 0)
    }
}

/// 一个被延迟到所属 scope 的 step 6（normal flow 之后）绘制的 positioned descendant。
///
/// 收集时记录**已累积的绝对坐标**，flush 时以 `offset = abs - node.xy` 调用 paint，
/// 使 paint_node 内部 `offset + node.xy = abs` 还原到正确位置。
struct DeferredPositioned<'a> {
    node: &'a LayoutBox,
    abs_x: f32,
    abs_y: f32,
}

/// 子节点的 content-box origin（父 abs + padding + border，扣除 scroll 偏移）。
///
/// paint_node 与 collect_positioned_auto_descendants 共用此函数，确保两者
/// 的偏移累积**完全一致**（避免独立维护两套 offset 逻辑导致发散）。
fn child_content_origin(box_node: &LayoutBox, abs_x: f32, abs_y: f32) -> (f32, f32) {
    let mut cx = abs_x + box_node.padding_left + box_node.border_left;
    let mut cy = abs_y + box_node.padding_top + box_node.border_top;
    if matches!(box_node.overflow_x, OverflowClip::Scroll) {
        cx -= box_node.scroll_x;
    }
    if matches!(box_node.overflow_y, OverflowClip::Scroll) {
        cy -= box_node.scroll_y;
    }
    (cx, cy)
}

/// 判断节点是否需要对子内容裁剪（overflow 或 contain:paint/strict/content 触发）。
///
/// 从 paint_node 抽出，供 collect_positioned_auto_descendants 镜像 defer_abspos 条件，
/// 避免 scan 与 paint 对「该节点是否 defer_abspos」判定发散（致 double-paint 或漏绘）。
fn compute_needs_clip(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    if let Some(node_id) = box_node.node_id
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
    }
}

/// 收集 scope 根子树中所有 positioned descendants（z-index:auto pseudo-SC + real-SC，含嵌套），
/// 按 **tree order**（pre-order DFS）。
///
/// 遇任何 positioned 元素（自成 scope，不论 z-index）**不下钻**，直接收集；遇 in-flow/float
/// 下钻以找嵌套 positioned。镜像 paint_node 的 defer_abspos 排除（非 positioned overflow 的
/// 直接 abspos/fixed 子元素由 defer_abspos 循环绘制，不纳入 flush）与 multicol column-span
/// 跳过（由 multicol 循环处理）。flush 时按 z_index 分 step 2(<0)/6(==0)/7(>0) 绘制。
fn collect_positioned_descendants<'a>(
    box_node: &'a LayoutBox,
    abs_x: f32,
    abs_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    out: &mut Vec<DeferredPositioned<'a>>,
) {
    let (cx, cy) = child_content_origin(box_node, abs_x, abs_y);
    let needs_clip = compute_needs_clip(box_node, styles);
    let self_positioned = is_positioned_child(box_node);
    let is_multicol = box_node.is_multicol;
    let defer_abspos = needs_clip && !self_positioned && !is_multicol;
    for child in &box_node.children {
        // 多列 column-span 子元素由 multicol 循环处理
        if is_multicol && !child.column_span_offsets.is_empty() {
            continue;
        }
        let child_abs_x = cx + child.x;
        let child_abs_y = cy + child.y;
        if is_positioned_child(child) {
            // defer_abspos 处理的直接 abspos/fixed 子元素由其循环绘制，跳过避免 double-paint
            if defer_abspos && (child.is_absolute || child.is_fixed) {
                continue;
            }
            // 所有 positioned（z-index:auto pseudo-SC + real-SC）收集到所属 scope，
            // 不下钻（positioned 自成 scope），flush 时按 z_index 分 step 2/6/7。
            out.push(DeferredPositioned {
                node: child,
                abs_x: child_abs_x,
                abs_y: child_abs_y,
            });
        } else if child.creates_stacking_context {
            // in-flow 但建立堆叠上下文（opacity<1/transform/filter 等 CSS3 SC 触发器）：
            // 自成 scope，其 positioned 后代由它自己的 collect/flush 收集。不下钻避免
            // double-collect（否则本 scope 会把其后代也收进来，与子 scope 重复绘制）。
            // 子元素本身仍在主循环 in-flow 绘制（SC 不改变自身 paint 位置，只隔离后代堆叠）。
            continue;
        } else {
            // in-flow/float 非 scope：下钻找嵌套 positioned
            collect_positioned_descendants(child, child_abs_x, child_abs_y, styles, out);
        }
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
            canvas_propagated_node: None,
            inline_heights: HashMap::new(),
            document_url: None,
        }
    }

    /// 设置当前文档 URL。
    pub fn set_document_url(&mut self, url: Option<&str>) {
        self.document_url = url.map(str::to_string);
    }

    /// 设置 CSS font-family 查找表。
    ///
    /// 由调用方从 `FontLoader::build_font_resolver()` 构建并传入。
    pub fn set_font_resolver(&mut self, resolver: HashMap<String, u32>) {
        self.font_resolver = resolver;
    }

    /// 根据 CSS font-family 与 font-weight 解析 FontId。
    ///
    /// 遍历 font-family 列表，返回第一个匹配的 FontId。
    /// `font-weight >= 600` 时优先查找 `{family}:700` 粗体 face。
    pub(crate) fn resolve_font_id(
        &self,
        font_family: &[String],
        font_weight: &zero_css_parser::values::FontWeightValue,
    ) -> zero_render_foundation::primitive::FontId {
        use zero_css_parser::values::FontWeightValue;
        use zero_render_foundation::primitive::FontId;

        let want_bold = matches!(font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
            || matches!(font_weight, FontWeightValue::Absolute(w) if *w >= 600);

        for family in font_family {
            let name = family.trim_matches('"').trim_matches('\'');
            if want_bold {
                let bold_key = format!("{name}:700");
                if let Some(&id) = self.font_resolver.get(&bold_key) {
                    return FontId(id);
                }
                for (key, &id) in &self.font_resolver {
                    if key.eq_ignore_ascii_case(&bold_key) {
                        return FontId(id);
                    }
                }
            }
            if let Some(&id) = self.font_resolver.get(name) {
                return FontId(id);
            }
            for (key, &id) in &self.font_resolver {
                if key.eq_ignore_ascii_case(name) {
                    return FontId(id);
                }
            }
        }
        if want_bold && let Some(&id) = self.font_resolver.get("sans-serif:700") {
            return FontId(id);
        }
        FontId(0)
    }

    /// 绘制整个布局树。
    ///
    /// 遍历 LayoutBox 树，为每个有样式的节点生成背景和边框填充图元。
    /// 传入 `doc` 以启用行内格式化上下文的文本换行布局。
    pub fn paint(&mut self, layout: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, doc: Option<&Document>) {
        // CSS §14.2 画布背景传播：根元素 html 的背景（color 或 image）覆盖整个画布；
        // 若 html 背景透明（color 透明 + image none）且 body 有背景，则 body 背景传播
        // 到画布。传播包含 color + image（R491 仅 color；R507 扩展 image 平铺整个画布）。
        // 传播元素的背景由画布统一绘制，paint_background_image 跳过该元素自身图像绘制
        //（避免 padding-box 起始图像与画布 (0,0) 起始图像相位错位 double-paint）。
        if self.viewport_w > 0.0
            && self.viewport_h > 0.0
            && let Some(doc) = doc
        {
            use zero_style_system::property::types::ColorValue;
            let html_id = doc.get_elements_by_tag_name("html").into_iter().next();
            let body_id = doc.get_elements_by_tag_name("body").into_iter().next();
            let html_style = html_id.and_then(|id| styles.get(&id));
            let body_style = body_id.and_then(|id| styles.get(&id));
            // CSS §9.2.4/§14.2：根元素 display:none 不生成盒、整个文档树不参与渲染，
            // 故根元素（及作为其后代的 body）背景均不传播到画布（canvas 保持默认白）。
            // 实测 chromium：html{display:none} 渲染为纯白 canvas（root-box-003）。
            let html_is_display_none = html_style.is_some_and(|hs| hs.display == DisplayValue::None);
            // html 有任意背景（color 非透明 或 至少一个非 None 图层）→ html 传播；否则 body。
            // 注意：`background-image: none` 解析为 `vec![None]`（非空但无实际图层），
            // 不应算作「有图片」——否则 html{background:transparent} 会因 [None] 图层误判
            // 为有背景，使 body 的背景无法传播到画布（background-root-005 等，R879）。
            let html_has_bg = !html_is_display_none
                && html_style.is_some_and(|hs| {
                    hs.background_color != ColorValue::Transparent || has_paintable_bg_image(&hs.background_image)
                });
            let (prop_node, prop_style) = if html_has_bg {
                (html_id, html_style)
            } else if html_is_display_none {
                // html display:none → body 作为其后代亦不渲染，不传播。
                (None, None)
            } else {
                (body_id, body_style)
            };
            self.canvas_propagated_node = prop_node;
            if let Some(ps) = prop_style
                && (ps.background_color != ColorValue::Transparent || has_paintable_bg_image(&ps.background_image))
            {
                if ps.background_color != ColorValue::Transparent {
                    self.primitives.add_fill(
                        Rect::new(0.0, 0.0, self.viewport_w, self.viewport_h),
                        color_value_to_render(&ps.background_color),
                    );
                }
                // 画布背景图像：以视口 (0,0,vw,vh) 为 origin 平铺（CSS §14.2：传播的
                // 背景图像 anchored 相对根元素盒 = 画布）。
                self.paint_bg_image_in_origin(0.0, 0.0, self.viewport_w, self.viewport_h, ps);
            }
        }
        // R639：预扫描布局树填充 NodeId→height 索引，供 render_fragment 宏查 owner inline
        // 自身 height（box_node 是 IFC owner 非 inline 本身）。
        self.inline_heights.clear();
        Self::collect_box_heights(layout, &mut self.inline_heights);
        self.paint_node(layout, styles, 0.0, 0.0, doc, true);
    }

    /// R639：递归遍历布局树，收集每个有 node_id 的盒的 height 到索引。
    /// 用于 render_fragment 宏按 owner_id（inline 元素）查其自身 box height，
    /// 而非 IFC owner 的 box_node.height（消除 inline-ownership split 致抑制/per-fragment 分歧）。
    fn collect_box_heights(box_node: &LayoutBox, map: &mut HashMap<NodeId, f32>) {
        if let Some(node_id) = box_node.node_id {
            map.insert(node_id, box_node.height);
        }
        for child in &box_node.children {
            Self::collect_box_heights(child, map);
        }
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

        // R792：overflow 裁剪基线快照。paint_text 绘制盒子**自身**直属文本（在子节点之前），
        // 原快照取于 paint_text 之后，致裁剪范围 [snapshot..end] 只含子节点、漏掉自身文本——
        // overflow!=visible 的盒子（如 max-width-106 的 float+overflow:scroll）直属溢出文本
        // 不被裁到 content-box 而外溢可见。此处先取默认快照（匿名文本项/无样式分支用），
        // 有样式分支在 bg/border/outline 之后、paint_text 之前重赋值，使裁剪范围 = 自身文本 +
        // list marker + 列背景 + 子节点，而 background/border/outline/shadow 保持不裁（CSS：
        // overflow 只裁内容到 padding-box，盒子自身装饰不裁）。
        let mut counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

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
                // R639：仅跨多行 inline 的 background 改由 paint_text 按行片段绘制，box-level 抑制
                //（与 paint_node 同步；单行/空/定位 inline 保留 box-level）。
                let inline_fs_px = match style.font_size {
                    LengthValue::Px(s) => s as f32,
                    _ => 16.0,
                };
                let skip_inline_box_bg = matches!(style.display, DisplayValue::Inline)
                    && style.background_color != ColorValue::Transparent
                    && !box_node.is_absolute
                    && !box_node.is_fixed
                    && box_node.height > inline_fs_px * 1.5
                    && doc.is_some_and(|d| text::has_direct_paintable_text(d, node_id, Some(styles)));
                if style.background_color != ColorValue::Transparent && !skip_inline_box_bg {
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

            // R792：background/border/outline 已绘制完毕；此后（list marker + 自身文本 +
            // 列背景 + 子节点）纳入 overflow 裁剪范围。
            counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

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

        // CSS 2.1 Appendix E:
        // 负 z-index 的 positioned 后代在常规流内容之后方，
        // 非 positioned float 在常规流后代之上，
        // 非负 z-index 的 positioned 后代位于最上层。
        for child_idx in ordered_child_indices(&box_node.children, |_| true) {
            let child = &box_node.children[child_idx];
            self.paint_node_in_rect(child, styles, child_offset_x, child_offset_y, dirty_rect, doc);
        }

        if needs_clip {
            // R793：CSS §11.1.1 — overflow 裁剪到 **padding box**（内容 + padding，border 之内），
            // 非 content box。原实现按 content box 裁剪（起点加 padding、尺寸=content），致溢出内容
            // 落在 content 边与 padding 边之间的条带时被多裁（chromium 保留到 padding 边）。
            let clip_rect = Rect::new(
                abs_x + box_node.border_left,
                abs_y + box_node.border_top,
                box_node.padding_left + box_node.content_width + box_node.padding_right,
                box_node.padding_top + box_node.content_height + box_node.padding_bottom,
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
        is_root_scope: bool,
    ) {
        let abs_x = offset_x + box_node.x;
        let abs_y = offset_y + box_node.y;

        // 判断是否需要裁剪子内容（overflow 或 contain:paint 触发）
        let needs_clip = compute_needs_clip(box_node, styles);

        // 获取该节点对应的计算样式
        // 记录绘制前的图元数量，用于 opacity 应用
        let counts_before = PrimitiveCounts::snapshot(&self.primitives);

        // R792：overflow 裁剪基线快照（默认覆盖匿名文本项/无样式分支）；有样式分支在装饰
        // 绘制后、paint_text 前重赋值，使裁剪范围含 list marker/img/content/自身文本/列背景/
        // 子节点，而排除 background/border/outline/shadow/clip-path 指示（CSS：overflow 只裁
        // 内容到 padding-box，盒子自身装饰不裁）。原快照取于 paint_text 之后致直属文本漏裁。
        let mut counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

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
                // R639：仅跨多行 inline（height>1.5×fs）+ 有文本 + 非定位 的 background 改由
                // paint_text 按行片段绘制，box-level 抑制。单行/空/定位 inline 保留 box-level。
                let inline_fs_px = match style.font_size {
                    LengthValue::Px(s) => s as f32,
                    _ => 16.0,
                };
                let skip_inline_box_bg = matches!(style.display, DisplayValue::Inline)
                    && style.background_color != ColorValue::Transparent
                    && !box_node.is_absolute
                    && !box_node.is_fixed
                    && box_node.height > inline_fs_px * 1.5
                    && doc.is_some_and(|d| text::has_direct_paintable_text(d, node_id, Some(styles)));
                if style.background_color != ColorValue::Transparent && !skip_split_inline_deco && !skip_inline_box_bg {
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

            // R792：装饰（bg/border/outline/clip-path 指示/backdrop-filter/shadow）已绘制完毕；
            // 此后（list marker/img/content/自身文本/列背景/子节点）纳入 overflow 裁剪范围。
            counts_before_children = PrimitiveCounts::snapshot(&self.primitives);

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

        // 6. 递归绘制子节点（子节点偏移 = 父 padding + border，扣除 scroll）
        // visibility: hidden 不阻止子节点绘制，子节点可以覆盖为 visible
        let (child_offset_x, child_offset_y) = child_content_origin(box_node, abs_x, abs_y);

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

        // CSS 2.1 Appendix E 全局 positioned-descendant 延迟（step 2/6/7）：
        // scope 根（positioned 元素、根 html、或任何建立堆叠上下文的元素）收集其子树中
        // **所有** positioned 后代（z-index:auto pseudo-SC + real-SC，含嵌套，经
        // collect_positioned_descendants 按 tree order），按 z_index 分三段绘制：step 2
        //（z<0，normal flow 之前，最负优先）→ steps 3-5（in-flow/float）→ step 6
        //（z==0，即 z-index:auto/0，tree order）→ step 7（z>0，最正优先）。非 scope 节点
        // 不收集/flush，但其主循环只绘制 in-flow/float（positioned 子元素一律由最近 scope
        // 祖先收集）。per-node 排序无法实现全局 tree-order（R503 (3,0) 在 abspos-016 与
        // static-inside-inline/z-index-abspos-004 间不可兼得），故显式收集。
        // ★ creates_stacking_context 含 opacity<1 等 CSS3 SC 触发器（见 engine.rs）：
        // 这些元素的 per-node 效果（opacity/filter/transform/...，paint_node 末尾对
        // [counts_before, now] 应用）必须覆盖其 positioned 后代；若它们非 scope，后代会被
        // 上提到祖先而漏掉效果（opacity:0 不隐藏内容的 R505 回归）。详见 R505。
        let is_scope = self_positioned || box_node.creates_stacking_context || is_root_scope;
        let mut collected_positioned: Vec<DeferredPositioned> = Vec::new();
        if is_scope {
            collect_positioned_descendants(box_node, abs_x, abs_y, styles, &mut collected_positioned);
        }

        // step 2：负 z-index SC（normal flow 之前；collected 已 tree-order，按 z_index 升序稳定排序）
        if is_scope {
            let mut neg_z: Vec<&DeferredPositioned> =
                collected_positioned.iter().filter(|i| i.node.z_index < 0).collect();
            neg_z.sort_by_key(|i| i.node.z_index);
            for item in &neg_z {
                let off_x = item.abs_x - item.node.x;
                let off_y = item.abs_y - item.node.y;
                self.paint_node(item.node, styles, off_x, off_y, doc, false);
            }
        }

        // steps 3/4/5：in-flow / float（仅非 positioned 子元素；positioned 由 scope flush 处理）
        for child_idx in ordered_child_indices(&box_node.children, |child| {
            (!is_multicol || child.column_span_offsets.is_empty())
                && (!defer_abspos || (!child.is_absolute && !child.is_fixed))
                && !is_positioned_child(child)
        }) {
            let child = &box_node.children[child_idx];
            self.paint_node(child, styles, child_offset_x, child_offset_y, doc, false);
        }

        // step 6：z-index:auto/0（positioned-auto + z:0 SC），tree order（z_index==0，无需排序）
        if is_scope {
            for item in collected_positioned.iter().filter(|i| i.node.z_index == 0) {
                let off_x = item.abs_x - item.node.x;
                let off_y = item.abs_y - item.node.y;
                self.paint_node(item.node, styles, off_x, off_y, doc, false);
            }
        }

        // step 7：正 z-index SC（z_index 升序：低 z 先绘、高 z 后绘居上；等 z 保 tree order）。
        // 与 R503 per-node `(key, z_index)` 升序一致；CSS Appendix E step 7 高 z 居上 = 后绘。
        if is_scope {
            let mut pos_z: Vec<&DeferredPositioned> =
                collected_positioned.iter().filter(|i| i.node.z_index > 0).collect();
            pos_z.sort_by_key(|i| i.node.z_index);
            for item in &pos_z {
                let off_x = item.abs_x - item.node.x;
                let off_y = item.abs_y - item.node.y;
                self.paint_node(item.node, styles, off_x, off_y, doc, false);
            }
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
                    self.paint_node(child, styles, frag_offset_x, frag_offset_y, doc, false);

                    super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before_frag, &clip_rect);
                }
            }
        }

        // 如果需要裁剪，将子节点产生的图元裁剪到 padding box 范围内（CSS §11.1.1，见 R793）
        if needs_clip {
            // R793：CSS §11.1.1 — overflow 裁剪到 **padding box**（内容 + padding，border 之内），
            // 非 content box。原实现按 content box 裁剪（起点加 padding、尺寸=content），致溢出内容
            // 落在 content 边与 padding 边之间的条带时被多裁（chromium 保留到 padding 边）。
            let clip_rect = Rect::new(
                abs_x + box_node.border_left,
                abs_y + box_node.border_top,
                box_node.padding_left + box_node.content_width + box_node.padding_right,
                box_node.padding_top + box_node.content_height + box_node.padding_bottom,
            );
            super::helpers::clip_all_primitives_to_rect(&mut self.primitives, &counts_before_children, &clip_rect);
        }

        // 非 positioned overflow 元素：abspos/fixed 子元素移到裁剪之后绘制，
        // 使其 CB 为祖先时不被本 overflow 误裁（CSS §11.1.1）。
        if defer_abspos {
            for child_idx in ordered_child_indices(&box_node.children, |child| child.is_absolute || child.is_fixed) {
                let child = &box_node.children[child_idx];
                self.paint_node(child, styles, child_offset_x, child_offset_y, doc, false);
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
        // CSS §14.2：背景已传播到画布的元素（html/body）不在自身盒上绘制背景色——
        // 画布已以视口 (0,0) 为 origin 统一绘制其 color+image。若此处再绘 color，body（传播
        // 到画布时，html 透明）的 bg color 会覆盖画布 image（background-root-007：body red
        // 覆盖画布 tiled white-square image，ZW 显红而 chromium 全白）。镜像 paint_background_image
        // 的 canvas_propagated_node 跳过（effects.rs:69）。
        if box_node
            .node_id
            .is_some_and(|id| self.canvas_propagated_node == Some(id))
        {
            return;
        }
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
