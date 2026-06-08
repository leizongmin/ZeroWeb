//! 布局引擎协调器。
//!
//! [`LayoutEngine`] 接收 DOM 和计算样式，通过 taffy 计算布局，
//! 输出 [`LayoutResult`]（布局盒树）。
//!
//! 支持两种计算模式：
//! - **全量计算** (`compute`): 每次从 DOM 重新构建 taffy 树并完整计算。
//! - **增量计算** (`compute_incremental`): 复用缓存的 taffy 树，仅重算脏节点。

use std::collections::HashMap;
use taffy::prelude::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, OverflowValue, PositionValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::{ComputedStyle, ZIndexValue};

use crate::dirty::LayoutDirtyTracker;
use crate::inline::{FloatExclusion, InlineFormattingContext};
use crate::tree::build_layout_tree;
use crate::types::{LayoutBox, LayoutResult, OverflowClip};

/// 缓存的 taffy 布局状态 — 用于增量重算。
///
/// 保存 taffy 树、根节点和映射关系，避免每次全量重建。
pub struct CachedLayoutState {
    /// taffy 布局树。
    taffy: TaffyTree<NodeId>,
    /// taffy 根节点 ID。
    root_id: taffy::NodeId,
    /// DOM NodeId → taffy NodeId 映射。
    dom_to_taffy: HashMap<NodeId, taffy::NodeId>,
    /// taffy NodeId → DOM NodeId 反向映射。
    taffy_to_dom: HashMap<taffy::NodeId, NodeId>,
}

/// 增量布局结果 — 包含布局输出和增量计算统计。
#[derive(Debug, Clone)]
pub struct IncrementalLayoutStats {
    /// 标记为脏的节点数量。
    pub dirty_node_count: usize,
    /// 是否退化为全量重算。
    pub was_full_recalc: bool,
    /// 布局计算耗时（毫秒）。
    pub layout_ms: f64,
}

/// 布局引擎 — 接收 DOM + 计算样式，输出布局盒树。
///
/// 使用 taffy 作为底层布局算法实现，支持 Block、Flexbox 和 Grid 布局。
/// 支持全量计算和增量计算两种模式。
pub struct LayoutEngine {
    /// 视口宽度。
    pub viewport_width: f32,
    /// 视口高度。
    pub viewport_height: f32,
    /// 缓存的 taffy 布局状态（可选，用于增量计算）。
    cached_state: Option<CachedLayoutState>,
}

impl LayoutEngine {
    /// 创建新的布局引擎实例。
    ///
    /// # 参数
    ///
    /// - `viewport_width` — 视口宽度（像素）
    /// - `viewport_height` — 视口高度（像素）
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            cached_state: None,
        }
    }

    /// 计算整个文档的布局（全量）。
    ///
    /// # 流程
    ///
    /// 1. 从 DOM + 计算样式构建 taffy 树
    /// 2. 使用 taffy 计算布局
    /// 3. 从 taffy 结果中提取 LayoutBox 树
    ///
    /// # 参数
    ///
    /// - `doc` — DOM 文档
    /// - `styles` — 元素 NodeId → ComputedStyle 映射
    pub fn compute(&mut self, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> LayoutResult {
        // 1. 构建 taffy 树
        let (mut taffy_tree, root_id, taffy_to_dom) =
            build_layout_tree(doc, styles, self.viewport_width, self.viewport_height);

        // 构建 dom→taffy 反向映射
        let dom_to_taffy: HashMap<NodeId, taffy::NodeId> =
            taffy_to_dom.iter().map(|(&t_id, &d_id)| (d_id, t_id)).collect();

        // 2. 计算布局
        let available_space = taffy::geometry::Size {
            width: AvailableSpace::Definite(self.viewport_width),
            height: AvailableSpace::Definite(self.viewport_height),
        };
        let _ = taffy_tree.compute_layout_with_measure(
            root_id,
            available_space,
            |known_dimensions, available_space, _node_id, context, _style| {
                let dom_id = match context {
                    Some(id) => *id,
                    None => return Size::ZERO,
                };
                measure_text_content(doc, styles, dom_id, known_dimensions, available_space)
            },
        );

        // 3. 提取 LayoutBox 树
        let mut root_box = Self::extract_layout(&taffy_tree, root_id, &taffy_to_dom, styles);

        // 4. 后处理：将 fixed 元素的坐标调整为视口相对
        adjust_fixed_to_viewport(&mut root_box, 0.0, 0.0);

        // 5. 后处理：调整 float 元素位置
        adjust_float_positions(&mut root_box);

        // 6. 后处理：为包含 float 元素的容器重新测量文本，使文本环绕 float 排列
        remeasure_text_with_float_exclusions(&mut root_box, doc, styles);

        // 7. 后处理：CSS margin 折叠 — taffy 0.7 已内置块级 margin 折叠（CollapsibleMarginSet）
        // 不需要额外后处理

        // 8. 后处理：对 display:table 容器执行 table grid 布局
        crate::table::adjust_table_layout(&mut root_box, doc, styles);

        // 9. 后处理：对 column-count/column-width 容器执行多列布局
        crate::multicol::adjust_multicol_layout(&mut root_box, styles);

        // 缓存 taffy 状态用于后续增量计算
        self.cached_state = Some(CachedLayoutState {
            taffy: taffy_tree,
            root_id,
            dom_to_taffy,
            taffy_to_dom,
        });

        LayoutResult {
            root: root_box,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
        }
    }

    /// 增量布局计算 — 只重算脏节点及其祖先路径。
    ///
    /// 复用缓存的 taffy 树，通过 taffy 的 `mark_dirty` 机制
    /// 仅重算受影响子树的布局。
    ///
    /// # 参数
    ///
    /// - `doc` — DOM 文档
    /// - `styles` — 元素 NodeId → ComputedStyle 映射
    /// - `dirty_tracker` — 脏节点追踪器（消耗性，调用后清空）
    ///
    /// # 返回值
    ///
    /// 返回 (LayoutResult, IncrementalLayoutStats)。
    /// 如果无缓存状态或需要全量重算，退化为 `compute()`。
    pub fn compute_incremental(
        &mut self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        dirty_tracker: &mut LayoutDirtyTracker,
    ) -> (LayoutResult, IncrementalLayoutStats) {
        let use_start = std::time::Instant::now();

        // 如果需要全量重算或无缓存，退化为全量计算
        if dirty_tracker.is_full_recalc() || self.cached_state.is_none() {
            let was_full = dirty_tracker.is_full_recalc() || self.cached_state.is_none();
            let dirty_count = dirty_tracker.dirty_count();
            dirty_tracker.clear();
            let result = self.compute(doc, styles);
            let layout_ms = use_start.elapsed().as_secs_f64() * 1000.0;
            return (
                result,
                IncrementalLayoutStats {
                    dirty_node_count: dirty_count,
                    was_full_recalc: was_full,
                    layout_ms,
                },
            );
        }

        let cached = self.cached_state.as_mut().expect("cached_state checked above");

        // 标记脏节点的 taffy 节点为脏
        let mut marked_count = 0usize;
        let dirty_nodes = dirty_tracker.drain_dirty();
        for dom_id in &dirty_nodes {
            if let Some(&taffy_id) = cached.dom_to_taffy.get(dom_id) {
                let _ = cached.taffy.mark_dirty(taffy_id);
                marked_count += 1;
            }
        }

        // 重新计算布局（taffy 只重算脏节点）
        let available_space = taffy::geometry::Size {
            width: AvailableSpace::Definite(self.viewport_width),
            height: AvailableSpace::Definite(self.viewport_height),
        };
        let _ = cached.taffy.compute_layout_with_measure(
            cached.root_id,
            available_space,
            |known_dimensions, available_space, _node_id, context, _style| {
                let dom_id = match context {
                    Some(id) => *id,
                    None => return Size::ZERO,
                };
                measure_text_content(doc, styles, dom_id, known_dimensions, available_space)
            },
        );

        // 提取布局结果
        let mut root_box = Self::extract_layout(&cached.taffy, cached.root_id, &cached.taffy_to_dom, styles);
        adjust_fixed_to_viewport(&mut root_box, 0.0, 0.0);
        // margin 折叠由 taffy 0.7 内置处理
        crate::table::adjust_table_layout(&mut root_box, doc, styles);
        crate::multicol::adjust_multicol_layout(&mut root_box, styles);

        let layout_ms = use_start.elapsed().as_secs_f64() * 1000.0;

        let result = LayoutResult {
            root: root_box,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
        };

        (
            result,
            IncrementalLayoutStats {
                dirty_node_count: marked_count,
                was_full_recalc: false,
                layout_ms,
            },
        )
    }

    /// 使缓存的布局状态失效。
    ///
    /// 在视口大小变化等需要全局重算的场景调用。
    pub fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }

    /// 检查是否有缓存状态可用于增量计算。
    pub fn has_cached_state(&self) -> bool {
        self.cached_state.is_some()
    }

    /// 更新视口大小并使缓存失效。
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.invalidate_cache();
    }

    /// 从 taffy 布局结果中提取 LayoutBox 树。
    fn extract_layout(
        taffy: &TaffyTree<NodeId>,
        taffy_id: taffy::NodeId,
        taffy_to_dom: &HashMap<taffy::NodeId, NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> LayoutBox {
        let layout = taffy.layout(taffy_id).cloned().unwrap_or_default();
        let dom_id = taffy_to_dom.get(&taffy_id).copied();

        // 获取 ComputedStyle 用于提取定位和溢出信息
        let computed = dom_id.and_then(|id| styles.get(&id));

        let is_absolute = computed.is_some_and(|s| matches!(s.position, PositionValue::Absolute));
        let is_fixed = computed.is_some_and(|s| matches!(s.position, PositionValue::Fixed));
        let is_sticky = computed.is_some_and(|s| matches!(s.position, PositionValue::Sticky));
        let float = computed.map_or(FloatValue::None, |s| s.float.clone());
        let clear = computed.map_or(ClearValue::None, |s| s.clear.clone());
        let overflow_x = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_x));
        let overflow_y = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_y));
        // CSS 2.1 §9.4.1: display:flow-root 和 display:inline-block 都建立 BFC
        let is_flow_root = computed.is_some_and(|s| {
            matches!(s.display, DisplayValue::FlowRoot | DisplayValue::InlineBlock)
        });
        // CSS 2.1 §9.2.2: clear 属性仅适用于块级元素。
        // 块级元素 = display 为 block, list-item, table 的元素，
        // 以及 flex/grid 容器。table 内部元素（row-group, row, cell 等）
        // 不是块级元素，clear 不应生效。
        // 浮动的 inline/inline-block 元素自动变为块级。
        let is_block_level = computed.is_some_and(|s| {
            matches!(
                s.display,
                DisplayValue::Block
                    | DisplayValue::Flex
                    | DisplayValue::InlineFlex
                    | DisplayValue::Grid
                    | DisplayValue::InlineGrid
                    | DisplayValue::ListItem
                    | DisplayValue::FlowRoot
                    | DisplayValue::Table
                    | DisplayValue::InlineTable
                    | DisplayValue::TableCaption
            ) || !matches!(s.float, FloatValue::None)
                && matches!(s.display, DisplayValue::Inline | DisplayValue::InlineBlock)
        });
        let is_relative =
            computed.is_some_and(|s| matches!(s.position, PositionValue::Relative | PositionValue::Sticky));
        let z_index = computed.map_or(0, |s| match s.z_index {
            ZIndexValue::Auto => 0,
            ZIndexValue::Integer(z) => z,
        });

        // 计算内容区域
        let content_x = layout.location.x + layout.border.left + layout.padding.left;
        let content_y = layout.location.y + layout.border.top + layout.padding.top;
        let content_width =
            (layout.size.width - layout.border.left - layout.border.right - layout.padding.left - layout.padding.right)
                .max(0.0);
        let content_height = (layout.size.height
            - layout.border.top
            - layout.border.bottom
            - layout.padding.top
            - layout.padding.bottom)
            .max(0.0);

        // 递归提取子节点
        let children_taffy = taffy.children(taffy_id).unwrap_or_default();
        let mut children_boxes = Vec::with_capacity(children_taffy.len());
        for child_taffy in &children_taffy {
            children_boxes.push(Self::extract_layout(taffy, *child_taffy, taffy_to_dom, styles));
        }

        LayoutBox {
            node_id: dom_id,
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            content_x,
            content_y,
            content_width,
            content_height,
            border_top: layout.border.top,
            border_right: layout.border.right,
            border_bottom: layout.border.bottom,
            border_left: layout.border.left,
            padding_top: layout.padding.top,
            padding_right: layout.padding.right,
            padding_bottom: layout.padding.bottom,
            padding_left: layout.padding.left,
            margin_top: layout.margin.top,
            margin_right: layout.margin.right,
            margin_bottom: layout.margin.bottom,
            margin_left: layout.margin.left,
            children: children_boxes,
            is_absolute,
            is_fixed,
            is_sticky,
            float,
            clear,
            overflow_x,
            overflow_y,
            z_index,
            scroll_x: 0.0,
            scroll_y: 0.0,
            is_flow_root,
            is_block_level,
            is_relative,
        }
    }
}

fn measure_text_content(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
) -> Size<f32> {
    if !has_direct_text(doc, dom_id) {
        return Size::ZERO;
    }

    let width = known_dimensions
        .width
        .or(available_space.width.into_option())
        .unwrap_or(f32::INFINITY)
        .max(0.0);
    let mut inline_ctx = InlineFormattingContext::new(width);
    inline_ctx.layout(doc, dom_id, styles);

    let measured_width = inline_ctx
        .all_fragments()
        .iter()
        .map(|fragment| fragment.x + fragment.width)
        .fold(0.0_f32, f32::max);

    Size {
        width: known_dimensions.width.unwrap_or(measured_width),
        height: known_dimensions.height.unwrap_or(inline_ctx.total_height()),
    }
}

fn has_direct_text(doc: &Document, dom_id: NodeId) -> bool {
    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}

/// 将 OverflowValue 转换为 OverflowClip。
fn convert_overflow_to_clip(value: &OverflowValue) -> OverflowClip {
    match value {
        OverflowValue::Visible => OverflowClip::Visible,
        OverflowValue::Hidden => OverflowClip::Hidden,
        OverflowValue::Clip => OverflowClip::Clip,
        OverflowValue::Scroll | OverflowValue::Auto => OverflowClip::Scroll,
    }
}

/// 递归调整 fixed 定位元素的坐标为视口相对。
///
/// taffy 将 `position: fixed` 当作 `absolute` 处理，坐标是相对于包含块的。
/// 此函数在布局完成后遍历布局树，将 fixed 元素的坐标加上祖先累积偏移，
/// 使其变为相对于视口的绝对坐标。
fn adjust_fixed_to_viewport(box_node: &mut LayoutBox, parent_offset_x: f32, parent_offset_y: f32) {
    if box_node.is_fixed {
        // fixed 元素：加上祖先偏移使其成为视口相对坐标
        box_node.x += parent_offset_x;
        box_node.y += parent_offset_y;
    }

    let offset_x = if box_node.is_fixed {
        0.0
    } else {
        parent_offset_x + box_node.x
    };
    let offset_y = if box_node.is_fixed {
        0.0
    } else {
        parent_offset_y + box_node.y
    };

    for child in &mut box_node.children {
        adjust_fixed_to_viewport(child, offset_x, offset_y);
    }
}

/// 调整 float 元素的位置，并处理 clear 属性。
///
/// taffy 将 float 元素当作普通 block 处理（按正常流排列）。
/// 此后处理步骤将 float 元素重新定位到容器的左侧或右侧，
/// 支持水平并排放置、clear 属性（含 float 元素自身的 clear）。
///
/// ## 实现要点
///
/// - 同侧 float 水平并排放置（CSS 2.1 §9.5.1），空间不足时换行
/// - float 元素的 clear 属性正确生效
/// - 非 float 块元素的 clear 使用 max(normal_flow_Y, float_bottom)
/// - 非 float、非 clear 元素的 Y 偏移扣除 float 元素占据的垂直空间
fn adjust_float_positions(box_node: &mut LayoutBox) {
    use zero_css_parser::values::ClearValue;
    use zero_css_parser::values::FloatValue;

    // 容器的内容区域宽度
    let container_width = box_node.content_width;

    // 第一阶段：重新定位 float 元素，记录每个 float 在 taffy 布局中占据的垂直空间
    let mut line_y = 0.0f32;
    let mut line_max_height = 0.0f32;
    let mut left_used_width = 0.0f32;
    let mut right_used_width = 0.0f32;
    let mut left_float_bottom = 0.0f32;
    let mut right_float_bottom = 0.0f32;

    // 记录每个 float 子元素在 taffy 布局中的 Y 和高度，用于后续偏移修正
    let mut float_taffy_y: Vec<(usize, f32, f32)> = Vec::new(); // (index, taffy_y, outer_height)

    for (idx, child) in box_node.children.iter_mut().enumerate() {
        // 跳过绝对定位和 fixed 元素
        if child.is_absolute || child.is_fixed {
            continue;
        }

        // 处理非 float 元素的 clear 属性（延迟到第二阶段）
        if matches!(child.float, FloatValue::None) {
            continue;
        }

        // 记录 float 元素的 taffy Y 位置和高度
        let child_outer_height = child.margin_top + child.height + child.margin_bottom;
        float_taffy_y.push((idx, child.y, child_outer_height));

        // 计算浮动元素的总占用尺寸（含 margin）
        let child_outer_width = child.margin_left + child.width + child.margin_right;

        // 处理 float 元素自身的 clear 属性
        match child.clear {
            ClearValue::Left => {
                if left_float_bottom > line_y {
                    line_y = left_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Right => {
                if right_float_bottom > line_y {
                    line_y = right_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Both => {
                let clear_y = left_float_bottom.max(right_float_bottom);
                if clear_y > line_y {
                    line_y = clear_y;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {}
        }

        // 检查当前行是否有足够空间放置此浮动元素
        let available_width = container_width - left_used_width - right_used_width;
        if child_outer_width > available_width && line_max_height > 0.0 {
            line_y += line_max_height;
            left_used_width = 0.0;
            right_used_width = 0.0;
            line_max_height = 0.0;
        }

        match child.float {
            FloatValue::Left => {
                child.x = left_used_width + child.margin_left;
                child.y = line_y + child.margin_top;

                left_used_width += child_outer_width;
                let new_bottom = line_y + child_outer_height;
                left_float_bottom = left_float_bottom.max(new_bottom);
            }
            FloatValue::Right => {
                right_used_width += child_outer_width;
                child.x = container_width - right_used_width + child.margin_left;
                child.y = line_y + child.margin_top;

                let new_bottom = line_y + child_outer_height;
                right_float_bottom = right_float_bottom.max(new_bottom);
            }
            FloatValue::InlineStart | FloatValue::InlineEnd | FloatValue::None => {}
        }

        line_max_height = line_max_height.max(child_outer_height);
    }

    // 第二阶段：修正非 float 子元素的 Y 位置 + BFC 浮动排斥
    // CSS 规范中 float 元素脱离正常流，不应占据垂直空间。
    // taffy 将 float 当作正常 block 排列，导致后续非 float 元素的 Y 偏移过大。
    // 策略：维护一个 float_y_offset（累积 float 在 taffy 中占据的垂直空间），
    // 对每个非 float 子元素从 Y 中扣除 offset；clear 元素消耗 offset。
    //
    // BFC 浮动排斥（CSS 2.1 §9.5）：建立 BFC 的块级元素不得与浮动元素重叠。
    // 当一个非 float 块级元素建立 BFC 且与浮动元素垂直重叠时，
    // 需将其水平位置偏移到浮动元素旁边。
    if !float_taffy_y.is_empty() {
        let mut float_y_offset = 0.0f32;

        // 收集浮动元素的几何信息，用于 BFC 排斥计算
        let float_geometries: Vec<(FloatValue, f32, f32, f32, f32)> = box_node
            .children
            .iter()
            .filter(|c| !matches!(c.float, FloatValue::None))
            .map(|c| {
                (
                    c.float.clone(),
                    c.x,
                    c.y,
                    c.width + c.margin_left + c.margin_right,
                    c.height + c.margin_top + c.margin_bottom,
                )
            })
            .collect();

        for child in box_node.children.iter_mut() {
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if !matches!(child.float, FloatValue::None) {
                // float 元素：将其 taffy 高度加入 offset
                float_y_offset += child.margin_top + child.height + child.margin_bottom;
                continue;
            }

            // 保存 taffy 的原始 Y（调整前）
            let original_taffy_y = child.y;

            // CSS 规范：clear 属性仅适用于块级元素（CSS 2.1 §13.5）
            if !child.is_block_level {
                // 非块级元素（如 inline）：扣除 float offset，不处理 clear
                if float_y_offset > 0.0 {
                    child.y -= float_y_offset;
                }
                continue;
            }

            // CSS 2.1 §9.5.2 Clearance 计算
            // clearance 引入后，margin 折叠被阻止。taffy 的布局已包含 margin 折叠，
            // 因此 clear 元素的「假设位置」（无 clear 时）需用折叠后的位置。
            // 但 clearance 本身阻止了 clear 元素与前一兄弟的 margin 折叠，
            // 所以 uncollapsed 位置 = 前一兄弟底部 + 当前元素 margin-top。
            // 这里用 taffy_y - float_offset 作为含 margin 折叠的位置，
            // 并与 clear_bottom 取最大值，确保在浮动下方。
            // 若 clear_bottom > normal_y，则 clearance = clear_bottom - normal_y，
            // 等效于将元素推到浮动下方并阻止了 margin 折叠。
            match child.clear {
                ClearValue::Left | ClearValue::Right | ClearValue::Both => {
                    let clear_bottom = match child.clear {
                        ClearValue::Left => left_float_bottom,
                        ClearValue::Right => right_float_bottom,
                        _ => left_float_bottom.max(right_float_bottom),
                    };
                    // 假设位置：无 clear 时元素应在的位置（含 margin 折叠）
                    let normal_y = original_taffy_y - float_y_offset;
                    // clearance = max(0, clear_bottom - normal_y)
                    // 当 clearance > 0 时，margin 折叠被阻止，元素被推到 clear_bottom
                    // 当 clearance == 0 时（normal_y >= clear_bottom），margin 折叠仍被阻止
                    // 但元素位置不变（零 clearance 不等于无 clearance）
                    if clear_bottom > 0.0 && clear_bottom > normal_y {
                        // 需要 clearance：将元素推到浮动下方
                        child.y = clear_bottom;
                    } else {
                        // 零 clearance 或无浮动：保持正常位置
                        // 但 CSS 规定即使 clearance 为 0，margin 折叠也被阻止
                        child.y = normal_y;
                    }
                    // clear 元素消耗 offset：
                    // float_y_offset = original_taffy_y - child.y
                    // 使得后续元素：taffy_y - offset = child.y + child.height
                    float_y_offset = (original_taffy_y - child.y).max(0.0);
                }
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {
                    // 非 clear 的普通元素：从 Y 中扣除 float 占据的垂直空间
                    if float_y_offset > 0.0 {
                        child.y -= float_y_offset;
                    }
                }
            }

            // BFC 浮动排斥（CSS 2.1 §9.5）：
            // 建立 BFC 的块级元素不得与同容器的浮动元素重叠。
            // 当 BFC 元素的垂直范围与浮动元素重叠时，水平偏移以避开浮动。
            if child.is_block_level
                && !child.is_absolute
                && !child.is_fixed
                && matches!(child.float, FloatValue::None)
                && crate::margin_collapse::establishes_bfc(child)
            {
                let child_top = child.y;
                let child_bottom = child.y + child.height;

                for (float_dir, float_x, float_y, float_w, float_h) in &float_geometries {
                    let float_top = *float_y;
                    let float_bottom = *float_y + *float_h;

                    // 检查垂直重叠
                    if child_top < float_bottom && child_bottom > float_top {
                        match float_dir {
                            FloatValue::Left => {
                                // 左浮动：将 BFC 元素推到浮动元素右侧
                                let avoidance_x = float_x + *float_w;
                                if avoidance_x > child.x {
                                    child.x = avoidance_x;
                                    // 缩小宽度以不超出容器
                                    let max_width = container_width - child.x;
                                    if child.width > max_width {
                                        child.width = max_width.max(0.0);
                                    }
                                }
                            }
                            FloatValue::Right => {
                                // 右浮动：缩小 BFC 元素宽度以不重叠
                                let right_float_left = container_width - *float_w;
                                if child.x + child.width > right_float_left {
                                    let new_width = right_float_left - child.x;
                                    child.width = new_width.max(0.0);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // 调整容器高度：当 float 元素占据的垂直空间被移除后，
    // 容器高度应基于子元素的实际位置重新计算。
    // 否则容器底部会留有空白间隙。
    if !float_taffy_y.is_empty() {
        let content_bottom =
            box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .fold(0.0f32, |max_y, c| {
                    let bottom = c.y + c.height + c.margin_bottom;
                    max_y.max(bottom)
                });
        let content_top = box_node.content_y;
        let content_height = (content_bottom - content_top).max(0.0);
        // 如果内容区域实际高度小于 taffy 计算的高度，收缩容器
        if content_height < box_node.content_height {
            box_node.content_height = content_height;
            // 更新总高度（包含 padding + border）
            let new_total = content_height
                + box_node.padding_top
                + box_node.padding_bottom
                + box_node.border_top
                + box_node.border_bottom;
            // 仅当新高度更小时才更新（不扩大容器）
            if new_total < box_node.height {
                box_node.height = new_total;
            }
        }
    }

    // 递归处理子容器
    for child in &mut box_node.children {
        adjust_float_positions(child);
    }
}

/// 为包含 float 元素的容器重新测量行内文本，使文本环绕 float 排列。
///
/// 工作原理：
/// 1. 遍历 LayoutBox 树，找到同时包含 float 子元素和直接文本内容的容器
/// 2. 收集容器内的 float 元素的几何信息，构建 FloatExclusion 列表
/// 3. 使用 float exclusions 重新运行 InlineFormattingContext 排列文本
/// 4. 用重新排列后的行盒更新容器的内部布局信息
fn remeasure_text_with_float_exclusions(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // 收集此容器的 float 排除区域
    let has_floats = box_node.children.iter().any(|c| !matches!(c.float, FloatValue::None));

    if has_floats {
        // 构建 float 排除区域列表
        let exclusions: Vec<FloatExclusion> = box_node
            .children
            .iter()
            .filter(|c| !matches!(c.float, FloatValue::None))
            .filter_map(|c| {
                // c.y 现在是相对于父级内容区域的坐标（与 taffy 一致）
                let rel_y = c.y;
                if rel_y < 0.0 || c.width <= 0.0 || c.height <= 0.0 {
                    return None;
                }
                Some(FloatExclusion {
                    y: rel_y + c.margin_top,
                    height: c.height + c.margin_bottom,
                    width: c.width + c.margin_left + c.margin_right,
                    is_left: matches!(c.float, FloatValue::Left),
                })
            })
            .collect();

        // 如果有排除区域且容器有直接文本内容
        if !exclusions.is_empty()
            && let Some(dom_id) = box_node.node_id
            && has_direct_text(doc, dom_id)
        {
            // 重新运行 inline layout with float exclusions
            let container_width = box_node.content_width;
            let mut inline_ctx = InlineFormattingContext::new(container_width).with_float_exclusions(exclusions);
            inline_ctx.layout(doc, dom_id, styles);

            // 容器高度需要包含 float 元素占用的空间
            let text_height = inline_ctx.total_height();
            let float_bottom = box_node
                .children
                .iter()
                .filter(|c| !matches!(c.float, FloatValue::None))
                .map(|c| c.y + c.height + c.margin_bottom)
                .fold(0.0_f32, f32::max);

            // 使用文本和 float 中较大的高度
            let content_height = text_height.max(float_bottom);
            let _ = content_height; // 保留用于未来高度调整
        }
    }

    // 递归处理子容器
    for child in &mut box_node.children {
        remeasure_text_with_float_exclusions(child, doc, styles);
    }
}

#[cfg(test)]
mod table_layout_tests {
    use super::*;
    use zero_css_parser::values::DisplayValue;
    use zero_style_system::StyleSystem;

    #[test]
    fn test_table_styles_correct() {
        let html = r#"<html><body><table><tr><td>cell</td></tr></table></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);

        let root = doc.root();
        let mut stack = vec![root];
        let mut found_table = false;
        while let Some(nid) = stack.pop() {
            if let Some(style) = styles.get(&nid) {
                if let Some(n) = doc.get(nid) {
                    if let zero_dom::NodeKind::Element(elem) = &n.kind {
                        if elem.local_name() == "table" {
                            found_table = true;
                            assert_eq!(style.display, DisplayValue::Table, "table should have display:table");
                        }
                    }
                }
            }
            if let Some(n) = doc.get(nid) {
                stack.extend(n.children.iter().copied());
            }
        }

        assert!(found_table, "should find <table> element");
    }

    #[test]
    fn test_table_layout_runs() {
        let html = r#"<html><body style="margin:0"><table style="width:200px"><tr><td style="width:100px;height:40px"></td><td style="width:100px;height:40px"></td></tr></table></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // Should not crash, and root should have non-zero size
        assert!(result.root.width > 0.0);
        assert!(result.root.height > 0.0);
    }
}

#[cfg(test)]
mod tests;
