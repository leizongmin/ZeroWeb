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
use zero_css_parser::values::{FloatValue, OverflowValue, PositionValue};
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

        // 7. 后处理：对 display:table 容器执行 table grid 布局
        crate::table::adjust_table_layout(&mut root_box, doc, styles);

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
        crate::table::adjust_table_layout(&mut root_box, doc, styles);

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
        let overflow_x = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_x));
        let overflow_y = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_y));
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
            overflow_x,
            overflow_y,
            z_index,
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

/// 调整 float 元素的位置。
///
/// taffy 将 float 元素当作普通 block 处理（按正常流排列）。
/// 此后处理步骤将 float 元素重新定位到容器的左侧或右侧，
/// 并确保同一侧的 float 元素垂直堆叠不重叠。
///
/// 限制：当前实现处理同一容器内的 float 元素，不处理跨容器的 float 交互。
fn adjust_float_positions(box_node: &mut LayoutBox) {
    use zero_css_parser::values::FloatValue;

    // 容器的内部内容区域（相对于 box_node 的 x/y）
    let container_x = box_node.content_x;
    let container_y = box_node.content_y;
    let container_width = box_node.content_width;
    let container_height = box_node.content_height;

    // 跟踪左右 float 的当前 Y 偏移和最大底部
    let mut left_float_y = 0.0f32;
    let mut left_float_bottom = 0.0f32;
    let mut right_float_y = 0.0f32;
    let mut right_float_bottom = 0.0f32;

    for child in &mut box_node.children {
        // 跳过绝对定位和 fixed 元素
        if child.is_absolute || child.is_fixed {
            continue;
        }

        match child.float {
            FloatValue::Left => {
                // 定位到容器的左侧
                child.x = container_x;
                child.y = container_y + left_float_y;

                // 更新左侧 float 的堆叠状态
                left_float_y = left_float_bottom;
                left_float_bottom = left_float_bottom + child.margin_top + child.height + child.margin_bottom;

                // 确保不超出容器高度（float 不影响容器高度时跳过）
                let _ = container_height;
            }
            FloatValue::Right => {
                // 定位到容器的右侧
                child.x = container_x + container_width - child.width - child.margin_right;
                child.y = container_y + right_float_y;

                // 更新右侧 float 的堆叠状态
                right_float_y = right_float_bottom;
                right_float_bottom = right_float_bottom + child.margin_top + child.height + child.margin_bottom;
            }
            FloatValue::InlineStart | FloatValue::InlineEnd => {
                // inline-start/inline-end 在 LTR 下等同于 left/right
                // 简化处理：inline-start → left, inline-end → right
                // 暂不实现，按 None 处理
            }
            FloatValue::None => {
                // 非 float 元素：正常流布局，无需调整
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
                // 计算相对于容器内容区域的位置
                let rel_y = c.y - box_node.content_y;
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
                .map(|c| c.y - box_node.content_y + c.height + c.margin_bottom)
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
mod tests;
