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

use zero_css_parser::values::{
    AlignmentValue, ClearValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue, OverflowValue, PositionValue,
};

use zero_dom::{Document, NodeId, NodeKind};

use zero_style_system::{ComputedStyle, ZIndexValue};

use crate::dirty::LayoutDirtyTracker;

use crate::tree::{R109Wiring, build_layout_tree_with_r109};

use crate::types::{LayoutBox, LayoutResult, OverflowClip};

use zero_style_system::{WhiteSpaceValue, WritingModeValue};

// R342：float 定位/收缩与 IFC 终化逻辑抽到独立模块（2000 行规则 + Phase A 准备）。
// 通过 glob 引入保持 engine.rs 内调用点不变。
use crate::float_positioning::*;
use crate::inline_finalization::*;

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
    /// R109 接线产物（仅 R109_WIRE=1 时非空），供增量 extract 复用。
    r109: R109Wiring,
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
        self.compute_with_img_sizes(doc, styles, HashMap::new())
    }

    /// 与 `compute` 相同，但额外注入 `<img>` 的解码固有尺寸（按 DOM NodeId 索引），
    /// 供无 width/height 属性的替换元素回退到固有尺寸（DC-11）。
    pub fn compute_with_img_sizes(
        &mut self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
    ) -> LayoutResult {
        // 1. 构建 taffy 树（含 R109 接线产物，仅 R109_WIRE=1 时非空）
        let (mut taffy_tree, root_id, taffy_to_dom, r109) = build_layout_tree_with_r109(
            doc,
            styles,
            self.viewport_width,
            self.viewport_height,
            img_intrinsic_sizes,
        );

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

        // 3. 提取 LayoutBox 树（根元素使用 HorizontalTb 作为父级 writing mode）
        let mut root_box = Self::extract_layout(
            &taffy_tree,
            root_id,
            &taffy_to_dom,
            styles,
            &WritingModeValue::HorizontalTb,
            doc,
            &r109,
        );
        // 3.1 两趟固有宽度布局：width:max-content/min-content 的 flex/grid 容器
        // 在第一趟已塌缩为 ~0（converter MaxContent→length(0)）。此处测量其 intrinsic
        // 宽度，对可测且大于当前宽度的容器，把对应 taffy 节点宽度设为 intrinsic 并
        // mark_dirty 后重跑布局，再重新提取（其子元素按新宽度重新布局）。
        // 仅水平书写模式起步；intrinsic 不可测（如纯文本 item）的容器保持塌缩（中性）。
        if Self::apply_intrinsic_content_sizing(&mut taffy_tree, &root_box, &dom_to_taffy, styles, doc) {
            // 重跑 taffy 布局：set_style+mark_dirty 后需重新计算受影响子树。
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
            root_box = Self::extract_layout(
                &taffy_tree,
                root_id,
                &taffy_to_dom,
                styles,
                &WritingModeValue::HorizontalTb,
                doc,
                &r109,
            );
        }
        // CSS 2.1 §9.4.3：position:relative 的根元素（如 <html style="position:relative">）
        // 需应用 top/left inset 偏移。非根 block-level 元素的 relative inset 由 taffy
        // 应用到 layout.location，但 taffy 0.7 对**根节点**不应用（根总在 0,0）。
        // 此处手动补上根的 relative 偏移，使根及其 abspos 后代（CB=根 padding box）
        // 整体偏移到正确视觉位置。
        if root_box.is_relative {
            let (dx, dy) = resolve_relative_inset(&root_box, styles);
            if dx != 0.0 || dy != 0.0 {
                root_box.x += dx;
                root_box.y += dy;
            }
        }

        // CSS §10.3.3：根元素（如 <html>）margin-left/right 均为 auto 且边框盒宽度小于
        // 视口时应水平居中。taffy 对**嵌套** block 正确处理 auto margin 居中，但对**根
        // 节点**不应用（根无父级提供居中上下文，taffy 把根左对齐到 0）。此处补上根居中。
        //（display:table 的根由 shrink_table_to_block_content 在收缩后单独居中，此处跳过
        //   避免双重居中；仅水平书写模式，垂直模式块轴为 Y 不在此处理。）
        if matches!(root_box.writing_mode, WritingModeValue::HorizontalTb) {
            let root_style = root_box.node_id.and_then(|id| styles.get(&id));
            let is_table_root = root_style.is_some_and(|s| {
                matches!(
                    s.display,
                    zero_css_parser::values::DisplayValue::Table | zero_css_parser::values::DisplayValue::InlineTable
                )
            });
            let both_auto = root_style.is_some_and(|s| {
                matches!(s.margin_left, zero_css_parser::values::LengthValue::Auto)
                    && matches!(s.margin_right, zero_css_parser::values::LengthValue::Auto)
            });
            if both_auto && !is_table_root && root_box.width + 0.5 < self.viewport_width {
                let margin = (self.viewport_width - root_box.width) / 2.0;
                root_box.x = margin;
                root_box.margin_left = margin;
                root_box.margin_right = margin;
            }
        }

        // 3.5 从 taffy 缓存中提取 flex/grid 容器的基线信息
        // taffy 内部计算了 first_baselines 但未通过公开 API 暴露，
        // 通过 cached_baselines() 补丁访问。
        LayoutEngine::extract_baselines_recursive(&taffy_tree, root_id, &taffy_to_dom, &mut root_box, 0);

        // 4. 后处理：将 fixed 元素的坐标调整为视口相对
        adjust_fixed_to_viewport(&mut root_box, 0.0, 0.0);

        // 5. 后处理：调整 float 元素位置
        // 5a. 先标记孤立 table-internal 元素为匿名 table 根（建立 BFC），供 adjust_float_positions 识别
        mark_anonymous_table_roots(&mut root_box, styles, false);
        adjust_float_positions(&mut root_box);

        // 5.5 后处理：垂直书写模式下 width:auto 块级元素收缩到内容块轴跨度
        shrink_vertical_blocks_to_content(&mut root_box, styles, &WritingModeValue::HorizontalTb);

        // 5.6 后处理：width:auto 的 inline-block 收缩到内容宽度（shrink-to-fit，§10.3.9）
        shrink_inline_blocks_to_content(&mut root_box, doc, styles);

        // 5.7 后处理（R109 §9.2.1.1）：split inline 的匿名块片段收缩到文本宽 +
        // fragment border 边选择（首片段开放右、末片段开放左），使 inline 的
        // border/background 落在文本宽而非全宽（inline-box-001 等用例）。
        // R109 默认启用；未生成匿名块片段时 fragment_node_ids 恒 None，此步为 no-op。
        crate::r109::shrink_r109_anon_blocks(&mut root_box, doc, styles);

        // 6. 后处理：为包含 float 元素的容器重新测量文本，使文本环绕 float 排列
        remeasure_text_with_float_exclusions(&mut root_box, doc, styles);

        // 6.5 后处理：为仅包含 inline 子元素的容器重新测量内容高度
        // 空 inline 元素的 line-height 贡献需要通过 IFC 正确计算
        remeasure_inline_only_containers(&mut root_box, doc, styles);

        // 7. 后处理：CSS margin 折叠 — taffy 0.7 已内置块级 margin 折叠（CollapsibleMarginSet）
        // 不需要额外后处理

        // 8. 后处理：对 display:table 容器执行 table grid 布局
        crate::table::adjust_table_layout(&mut root_box, doc, styles);

        // 9. 后处理：对 column-count/column-width 容器执行多列布局
        crate::multicol::adjust_multicol_layout(&mut root_box, styles);

        // 10. 后处理：对包含 inline-block 子元素的容器，重新定位 inline-block 元素
        adjust_inline_block_positions(&mut root_box, doc, styles);

        // 10.5 后处理：修正垂直书写模式下绝对定位元素的静态位置
        fix_vertical_mode_abs_pos(&mut root_box, doc, styles);

        // 10.6 后处理：对 flex/grid 容器的子元素按 CSS order 排序
        // taffy 0.7 不支持 CSS order 属性，因此需要在后处理中排序。
        sort_children_by_css_order(&mut root_box, styles);

        // 11. 后处理：taffy 已对 block-level position:relative 元素应用 inset 偏移到 layout.location。
        // 但 inline-level 元素（如 <img>）由 inline layout 定位，taffy 不会处理其 relative offset。
        // 仅对 inline-level relative 元素应用偏移，避免 block-level 元素双重偏移。
        apply_relative_offsets_inline(&mut root_box, styles);

        // 11.5 后处理：修正没有 positioned ancestor 的 absolute 元素的**百分比**
        // inset 与尺寸。
        // CSS 2.1 §10.1：absolute 元素无 positioned ancestor 时，containing block 是
        // 初始包含块（视口）。taffy 用静态父作为 containing block，导致 width:50% 等百分比
        // 按父宽度解析。本步骤仅重解析百分比（Length/Auto 不动），避免旧版
        // adjust_absolute_to_initial_containing_block 同时调整 x/y 与 auto 宽高导致的回归。
        adjust_absolute_pct_to_viewport(
            &mut root_box,
            0.0,
            0.0,
            self.viewport_width,
            self.viewport_height,
            styles,
            false,
        );

        // 11.6 后处理：position:fixed 全-inset stretch 尺寸（CSS §10.3.18 / §10.6.4）。
        // fixed 元素 CB=视口；taffy 按 positioned 祖先 stretch 致尺寸不足。仅修 fixed
        // （位置已由 4. adjust_fixed_to_viewport 修正），不动 absolute 避旧回归。
        stretch_fixed_to_viewport_size(&mut root_box, self.viewport_width, self.viewport_height, styles);

        // 11.7 后处理：containing block = 根（positioned root）的 abspos 按根 padding-box
        // 重解析（CSS §10.1.2/§10.3.18/§10.6.4）。taffy 0.7 root quirk：根作 positioned
        // 祖先时不作 CB，误用静态父（abspos-containing-block-005/006）；R123 同谱系。
        // 仅根 positioned 时介入；非根 positioned 祖先 taffy 已正确（递归置 false 不触）。
        let root_is_positioned =
            root_box.is_absolute || root_box.is_fixed || root_box.is_relative || root_box.is_sticky;
        if root_is_positioned {
            // 先拷贝根几何到局部（避免与下方 &mut root_box 借用冲突）
            let (root_x, root_y) = (root_box.x, root_box.y);
            let (cb_origin_x, cb_origin_y) = (root_x + root_box.border_left, root_y + root_box.border_top);
            let cb_width = root_box.width - root_box.border_left - root_box.border_right;
            let cb_height = root_box.height - root_box.border_top - root_box.border_bottom;
            resolve_abspos_against_root_cb(
                &mut root_box,
                root_x,
                root_y,
                cb_origin_x,
                cb_origin_y,
                cb_width,
                cb_height,
                styles,
                true,
            );
        }

        // 12. 后处理：Final Inline Layout Pass（Phase A）。
        // 为含有直接文本子节点的容器计算最终行内布局并存储结果。
        // paint 系统消费存储的 IFC 结果，不再重跑 IFC。
        compute_final_inline_layouts(&mut root_box, doc, styles, &[]);

        // 12.5 后处理：修正 calc(P% ± Npx) 尺寸。
        // taffy 不支持 calc 表达式，convert 层将 calc(100% - 6px) 近似为 Percent(1.0)。
        // 此步骤根据实际百分比计算值和 px 偏移量修正最终尺寸。
        apply_calc_size_adjustments(&mut root_box, styles);

        // 12.6 后处理：百分比 max-height 收紧。
        // taffy 0.7 对 height:auto 的块盒不会按百分比 max-height 收紧最终高度
        // （convert 层已传 Percent，但 block 布局未在内容高度计算后再次 clamp）。
        // CSS §10.7：百分比 max-height 相对包含块高度解析；当包含块高度明确时收紧。
        // 此步骤自上而下传递「明确高度」，对百分比 max-height 的盒做收紧。
        clamp_percentage_max_height(&mut root_box, None, styles);

        // 诊断（不改变布局）：对 shrink-to-fit 候选容器（inline-flex/inline-grid/float:flex/
        // float:grid 的 width:auto，或任意 flex/grid 的 width:max-content/min-content）打印
        // 测得的固有宽度 vs 当前宽度，供 flex-grid 两趟布局（见 intrinsic_sizing / 设计草图）
        // 验证测量正确性。Round A：仅测量+打印，零布局副作用。
        if std::env::var("INTRINSIC_DBG").is_ok() {
            crate::intrinsic_sizing::debug_dump_shrink_candidates(&root_box, doc, styles);
        }
        // R109（CSS2 §9.2.1.1）诊断：对 inline 含 block 子元素的元素打印其匿名块拆分片段。
        // 仅 eprintln，零布局副作用。env R109_DBG=1 启用，为后续匿名块生成接线验证结构。
        if std::env::var("R109_DBG").is_ok() {
            crate::inline_block_split::debug_dump_inline_block_splits(&root_box, doc, styles);
        }

        // 缓存 taffy 状态用于后续增量计算
        self.cached_state = Some(CachedLayoutState {
            taffy: taffy_tree,
            root_id,
            dom_to_taffy,
            taffy_to_dom,
            r109,
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
        let mut root_box = Self::extract_layout(
            &cached.taffy,
            cached.root_id,
            &cached.taffy_to_dom,
            styles,
            &WritingModeValue::HorizontalTb,
            doc,
            &cached.r109,
        );
        adjust_fixed_to_viewport(&mut root_box, 0.0, 0.0);
        // margin 折叠由 taffy 0.7 内置处理
        crate::table::adjust_table_layout(&mut root_box, doc, styles);
        crate::multicol::adjust_multicol_layout(&mut root_box, styles);
        sort_children_by_css_order(&mut root_box, styles);
        // taffy 已在 layout.location 中包含 position:relative 的 inset 偏移，无需额外后处理

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
    ///
    /// 从 taffy 布局缓存中提取 flex/grid 容器的 first_baseline。
    ///
    /// taffy 在布局计算时为 flex 和 grid 容器计算 `first_baselines`，
    /// 但该值在 `LayoutOutput → Layout` 转换中被丢弃。
    /// 通过 taffy 补丁的 `cached_baselines()` 方法访问缓存中的基线值，
    /// 存储到 LayoutBox.taffy_baseline 供 `adjust_inline_block_positions` 使用。
    #[allow(clippy::only_used_in_recursion)]
    fn extract_baselines_recursive(
        taffy: &TaffyTree<NodeId>,
        taffy_id: taffy::NodeId,
        taffy_to_dom: &HashMap<taffy::NodeId, NodeId>,
        box_node: &mut LayoutBox,
        depth: usize,
    ) {
        // 尝试从 taffy 缓存提取基线
        if let Some(baselines) = taffy.cached_baselines(taffy_id) {
            if let Some(y_baseline) = baselines.y {
                box_node.taffy_baseline = Some(y_baseline);
            }
        }

        // 递归处理子元素
        let child_taffy_ids = taffy.children(taffy_id).unwrap_or_default();
        for (i, child_taffy) in child_taffy_ids.iter().enumerate() {
            if i < box_node.children.len() {
                Self::extract_baselines_recursive(
                    taffy,
                    *child_taffy,
                    taffy_to_dom,
                    &mut box_node.children[i],
                    depth + 1,
                );
            }
        }
    }

    /// 两趟固有宽度布局的第一趟修正：对 `width:max-content`/`min-content` 的
    /// flex/grid 容器提升宽度到测得的 intrinsic。
    ///
    /// 这些容器在第一趟布局中塌缩为 ~0（converter 把 MaxContent/MinContent 映射为
    /// `length(0)`，与旧「resolve 为 Px(0)」行为中性）。`intrinsic_sizing` 模块基于
    /// **显式宽度**测量其 max-content 宽度（不依赖塌缩后的布局宽度），若可测
    /// （>0）且大于当前宽度，则把对应 taffy 节点的 size.width 设为 intrinsic 并
    /// `mark_dirty`。调用方随后重跑 `compute_layout_with_measure` 并重新提取，
    /// 该容器及其子元素即按 intrinsic 宽度重新布局（grid track / flex item 重新分配）。
    ///
    /// 安全性：仅「可测且确实更宽」时才改动（0→intrinsic 纯改善，非破坏）；
    /// intrinsic 不可测（如纯文本 item，Round C IFC 文本测量未就绪）的容器保持塌缩。
    /// 仅水平书写模式、width 为 MaxContent/MinContent 的 flex/grid 容器。
    ///
    /// 返回是否有节点被修改。
    fn apply_intrinsic_content_sizing(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        doc: &Document,
    ) -> bool {
        let mut changed = false;
        let mut stack: Vec<&LayoutBox> = vec![root];
        while let Some(b) = stack.pop() {
            stack.extend(b.children.iter());
            let Some(id) = b.node_id else { continue };
            let Some(s) = styles.get(&id) else { continue };
            // 仅水平书写模式的 flex/grid 容器，且 width 为 max-content/min-content
            let is_container = matches!(
                s.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            );
            if !is_container || !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                continue;
            }
            if !matches!(s.width, LengthValue::MaxContent | LengthValue::MinContent) {
                continue;
            }
            let intrinsic = if matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid) {
                crate::intrinsic_sizing::grid_intrinsic_width(b, doc, styles)
            } else {
                crate::intrinsic_sizing::flex_row_intrinsic_width(b, doc, styles)
            };
            let Some(intrinsic) = intrinsic else { continue };
            // intrinsic 不可测或容器已足够宽 → 跳过（保持塌缩/当前行为，中性）
            if intrinsic <= 1.0 || b.width >= intrinsic + 1.0 {
                continue;
            }
            let Some(&taffy_id) = dom_to_taffy.get(&id) else {
                continue;
            };
            if let Ok(mut style) = taffy_tree.style(taffy_id).cloned() {
                style.size.width = taffy::style::Dimension::Length(intrinsic);
                let _ = taffy_tree.set_style(taffy_id, style);
                let _ = taffy_tree.mark_dirty(taffy_id);
                changed = true;
            }
        }
        changed
    }

    /// 当父元素具有垂直书写模式时，taffy 的布局结果是轴交换后的，
    /// 需要在提取时交换回来以获得正确的视觉坐标。
    fn extract_layout(
        taffy: &TaffyTree<NodeId>,
        taffy_id: taffy::NodeId,
        taffy_to_dom: &HashMap<taffy::NodeId, NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        parent_writing_mode: &WritingModeValue,
        doc: &Document,
        r109: &R109Wiring,
    ) -> LayoutBox {
        let layout = taffy.layout(taffy_id).cloned().unwrap_or_default();
        let dom_id = taffy_to_dom.get(&taffy_id).copied();
        // R109：匿名块片段（在 fragment_registry 中的 taffy 节点）→ 写片段节点覆盖；
        // 其 node_id=inline（tree.rs 已映射），故 is_block_level 由下面强制为 true。
        let fragment_node_ids = r109.fragment_registry.get(&taffy_id).cloned();
        let is_anon_fragment = fragment_node_ids.is_some();
        let is_r109_split = dom_id.is_some_and(|id| r109.split_parents.contains(&id));
        let r109_first_fragment = r109.first_inline_fragments.contains(&taffy_id);
        let r109_last_fragment = r109.last_inline_fragments.contains(&taffy_id);

        // 检测匿名文本项：node_id 指向文本节点（flex/grid 容器中的匿名项）
        // 文本节点没有 ComputedStyle，使用父元素的样式
        let is_anonymous_text_item =
            dom_id.is_some_and(|id| doc.get(id).is_some_and(|n| matches!(&n.kind, NodeKind::Text(_))));

        // 获取 ComputedStyle 用于提取定位和溢出信息
        // 对于匿名文本项，使用父元素的样式（文本节点继承父元素样式）
        let computed = if is_anonymous_text_item {
            dom_id
                .and_then(|id| doc.parent_node(id))
                .and_then(|pid| styles.get(&pid))
        } else {
            dom_id.and_then(|id| styles.get(&id))
        };

        // 获取此元素自身的 writing mode
        let own_writing_mode = computed.map_or(WritingModeValue::HorizontalTb, |s| s.writing_mode.clone());

        let is_absolute = computed.is_some_and(|s| matches!(s.position, PositionValue::Absolute));
        let is_fixed = computed.is_some_and(|s| matches!(s.position, PositionValue::Fixed));
        // 替换元素（有固有尺寸）：img/video/iframe/embed/object/svg/canvas。
        // CSS §10.3.8/§10.6.6 对其 auto 尺寸按固有尺寸解析，不走 §10.3.18/§10.6.4
        // 全-inset stretch。标记供 abspos stretch 后处理跳过（避免覆写固有尺寸）。
        let is_replaced = dom_id.is_some_and(|id| {
            doc.get(id).is_some_and(|n| match &n.kind {
                zero_dom::NodeKind::Element(elem) => matches!(
                    elem.local_name(),
                    "img" | "video" | "iframe" | "embed" | "object" | "svg" | "canvas"
                ),
                _ => false,
            })
        });
        let is_sticky = computed.is_some_and(|s| matches!(s.position, PositionValue::Sticky));
        let float = computed.map_or(FloatValue::None, |s| s.float.clone());
        let clear = computed.map_or(ClearValue::None, |s| {
            if matches!(
                s.display,
                DisplayValue::TableRowGroup
                    | DisplayValue::TableHeaderGroup
                    | DisplayValue::TableFooterGroup
                    | DisplayValue::TableRow
                    | DisplayValue::TableCell
                    | DisplayValue::TableColumn
                    | DisplayValue::TableColumnGroup
            ) {
                ClearValue::None
            } else {
                s.clear.clone()
            }
        });
        // CSS 2.1 §17.5：table cell 的 height 为最小高度，cell 始终扩展以包含内容。
        // 但显式设置 overflow: auto/scroll 的 table cell 应创建可滚动容器。
        // 仅当 overflow 为 hidden/clip 时，table cell 不产生裁剪效果（保持 Visible）。
        let is_table_cell = computed.is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
        let overflow_x = if is_table_cell {
            computed.map_or(OverflowClip::Visible, |s| {
                match convert_overflow_to_clip(&s.overflow_x) {
                    OverflowClip::Hidden | OverflowClip::Clip => OverflowClip::Visible,
                    other => other,
                }
            })
        } else {
            computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_x))
        };
        let overflow_y = if is_table_cell {
            computed.map_or(OverflowClip::Visible, |s| {
                match convert_overflow_to_clip(&s.overflow_y) {
                    OverflowClip::Hidden | OverflowClip::Clip => OverflowClip::Visible,
                    other => other,
                }
            })
        } else {
            computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_y))
        };
        // CSS 2.1 §9.4.1: display:flow-root 和 display:inline-block 都建立 BFC
        let is_flow_root =
            computed.is_some_and(|s| matches!(s.display, DisplayValue::FlowRoot | DisplayValue::InlineBlock));
        let is_multicol = computed.is_some_and(|s| {
            use zero_style_system::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue};
            !matches!(s.column_count, ColumnCountComputedValue::Auto)
                || !matches!(s.column_width, ColumnWidthComputedValue::Auto)
        });
        let is_layout_container = computed.is_some_and(|s| {
            matches!(
                s.display,
                DisplayValue::Flex
                    | DisplayValue::InlineFlex
                    | DisplayValue::Grid
                    | DisplayValue::InlineGrid
                    | DisplayValue::Table
                    | DisplayValue::InlineTable
            )
        });
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
        }) || is_anon_fragment;
        let is_relative =
            computed.is_some_and(|s| matches!(s.position, PositionValue::Relative | PositionValue::Sticky));
        let is_positioned = is_absolute || is_fixed || is_relative;
        let z_index = computed.map_or(0, |s| match s.z_index {
            ZIndexValue::Auto => 0,
            ZIndexValue::Integer(z) => z,
        });
        // CSS 堆叠上下文触发器：
        // (1) CSS 2.1：positioned 元素 + z-index 显式整数（z-index: auto 不建 SC，
        //     其 positioned 后代参与父级 SC）。
        // (2) opacity < 1（CSS3）：opacity<1 建立堆叠上下文。R504 的全局 positioned-
        //     descendant 延迟会把 positioned 后代上提到最近 scope 祖先；若 opacity 元素
        //     非 scope，其 positioned 后代被上提到祖先的图元范围之外，paint_node 末尾
        //     对 [counts_before, now] 应用的 opacity（painter/mod.rs）会漏掉它们——
        //     opacity:0 不隐藏内容的回归（R505）。故 opacity<1 元素必须为 scope。
        let creates_stacking_context = (is_positioned
            && computed.is_some_and(|s| matches!(s.z_index, ZIndexValue::Integer(_))))
            || computed.is_some_and(|s| s.opacity < 1.0);

        // 从 taffy 提取原始值
        let mut x = layout.location.x;
        let mut y = layout.location.y;
        let mut width = layout.size.width;
        let mut height = layout.size.height;
        let mut border_top = layout.border.top;
        let mut border_right = layout.border.right;
        let mut border_bottom = layout.border.bottom;
        let mut border_left = layout.border.left;
        let mut padding_top = layout.padding.top;
        let mut padding_right = layout.padding.right;
        let mut padding_bottom = layout.padding.bottom;
        let mut padding_left = layout.padding.left;
        let mut margin_top = layout.margin.top;
        let mut margin_right = layout.margin.right;
        let mut margin_bottom = layout.margin.bottom;
        let mut margin_left = layout.margin.left;

        // 当父元素具有垂直书写模式时，taffy 的布局结果是轴交换后的，
        // 需要交换回正确的视觉坐标。
        // 与 tree.rs 中的 apply_vertical_writing_mode 轴交换配合使用：
        // 输入时将 CSS 垂直属性交换到 taffy 水平模型，输出时交换回视觉坐标。
        if matches!(
            parent_writing_mode,
            WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
        ) {
            // 交换位置
            std::mem::swap(&mut x, &mut y);
            // 交换尺寸
            std::mem::swap(&mut width, &mut height);
            // 交换边框
            std::mem::swap(&mut border_top, &mut border_left);
            std::mem::swap(&mut border_bottom, &mut border_right);
            // 交换内边距
            std::mem::swap(&mut padding_top, &mut padding_left);
            std::mem::swap(&mut padding_bottom, &mut padding_right);
            // 交换外边距
            std::mem::swap(&mut margin_top, &mut margin_left);
            std::mem::swap(&mut margin_bottom, &mut margin_right);
        }

        // declared_margin_top: 计算样式声明的 margin-top（仅水平书写模式 + Px 长度时有效）。
        // 用于检测 taffy 把 float 当作普通 block 导致容器 margin-top 与首个 float
        // 子元素的 margin 错误折叠（CSS §8.3.1：float 的 margin 不折叠）。
        // 非 Px 长度（Percent/Auto）或垂直书写模式下回退为布局值，不触发修正。
        let declared_margin_top = if matches!(parent_writing_mode, WritingModeValue::HorizontalTb) {
            computed
                .and_then(|c| match &c.margin_top {
                    zero_css_parser::values::LengthValue::Px(v) => Some(*v as f32),
                    _ => None,
                })
                .unwrap_or(margin_top)
        } else {
            margin_top
        };
        // CSS §10.3.5：width:auto 的浮动元素应 shrink-to-fit。记录 width:auto 标记
        //（仅水平书写模式）供 float 后处理收缩宽度。
        let declared_width_auto = matches!(parent_writing_mode, WritingModeValue::HorizontalTb)
            && computed.is_some_and(|c| matches!(c.width, zero_css_parser::values::LengthValue::Auto));

        // 计算内容区域
        let content_x = border_left + padding_left;
        let content_y = border_top + padding_top;
        let content_width = (width - border_left - border_right - padding_left - padding_right).max(0.0);
        let content_height = (height - border_top - border_bottom - padding_top - padding_bottom).max(0.0);

        // 递归提取子节点（使用此元素自身的 writing mode）
        let children_taffy = taffy.children(taffy_id).unwrap_or_default();
        let mut children_boxes = Vec::with_capacity(children_taffy.len());
        for child_taffy in &children_taffy {
            children_boxes.push(Self::extract_layout(
                taffy,
                *child_taffy,
                taffy_to_dom,
                styles,
                &own_writing_mode,
                doc,
                r109,
            ));
        }

        // taffy 的 `Layout::location` 是子节点 border box 相对**父 border box** 的偏移，
        // 已包含父 padding+border（见 taffy `Layout::content_box_y = location.y + border +
        // padding`）。但本引擎的绘制层与后处理（inline IFC 子节点、abspos 线程）一致采用
        // 「子节点坐标相对父**内容盒**」的约定（painter 在 child_offset 上叠加 padding+border）。
        // 因此对 taffy 定位的块级/inline-block 子节点，需把其 border-box 相对坐标换算为
        // 内容盒相对坐标（减去自身 content_x/y），否则父级每有 padding/border 就把子树整体
        // 下移/右移一份（重复计入），如 welcome.html 顶部 36px 垂直偏移。
        // 仅水平书写模式应用（垂直模式轴交换路径另算，避免回归）。
        // 注意：float 后处理会覆写浮动子节点的位置，此处换算对它们无害；
        // abspos/fixed 子节点由 adjust_absolute_* 线程按 border-box 相对约定单独处理
        //（其坐标语义与绘制层双计补偿自洽），此处跳过以保持其既有行为。
        if matches!(own_writing_mode, WritingModeValue::HorizontalTb) {
            for child in &mut children_boxes {
                if child.is_absolute || child.is_fixed {
                    continue;
                }
                child.x -= content_x;
                child.y -= content_y;
            }
        }

        LayoutBox {
            node_id: dom_id,
            x,
            y,
            width,
            height,
            content_x,
            content_y,
            content_width,
            content_height,
            border_top,
            border_right,
            border_bottom,
            border_left,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            declared_margin_top,
            declared_width_auto,
            children: children_boxes,
            is_absolute,
            is_replaced,
            is_fixed,
            is_sticky,
            float,
            clear,
            overflow_x,
            overflow_y,
            z_index,
            creates_stacking_context,
            scroll_x: 0.0,
            scroll_y: 0.0,
            is_flow_root,
            is_multicol,
            is_layout_container,
            is_anon_table_root: false,
            column_gap: 0.0,
            is_block_level,
            is_relative,
            collapsed_border_color_overrides: [None; 4],
            collapsed_border_style_overrides: [const { None }; 4],
            collapsed_border_outer_edge: [false; 4],
            writing_mode: own_writing_mode.clone(),
            is_anonymous_text_item,
            css_order: computed.as_ref().map(|s| s.order).unwrap_or(0),
            column_span_offsets: Vec::new(),
            inline_layout: None,
            inline_layout_width: 0.0,
            text_node_font_sizes: HashMap::new(),
            text_node_is_ahem: HashMap::new(),
            text_node_letter_spacing: HashMap::new(),
            text_node_line_heights: HashMap::new(),
            inline_element_metrics: HashMap::new(),
            inline_element_margins: HashMap::new(),
            taffy_baseline: None,
            fragment_node_ids,
            is_r109_split,
            r109_first_fragment,
            r109_last_fragment,
            table_col_backgrounds: Vec::new(),
        }
    }
}

/// 对 position:relative 元素应用视觉偏移。
///
/// CSS 2.1 §9.4.3：相对定位的元素在正常流中布局，然后根据 top/left/right/bottom
/// 值进行偏移。偏移不影响后续元素的布局位置。
///
/// 此函数在所有其他后处理（float、table、multicol）之后执行，
/// 仅修改元素自身的 x/y 坐标，不改变其布局尺寸或影响其他元素。
///
/// 注意：只偏移元素自身，不递归偏移子元素。因为 LayoutBox 的坐标系是相对的
/// 后处理：对包含 `display: inline-block` 子元素的容器，重新定位 inline-block 元素。
///
/// taffy 将 inline-block 映射为 Block，导致这些子元素垂直堆叠。
/// 此函数运行 InlineFormattingContext 获取正确的水平并排位置，
/// 然后将 inline-block 子元素的 LayoutBox 坐标更新为行内格式化结果。
///
/// 跳过 flex/grid/inline-flex/inline-grid 容器——它们的子元素由 flex/grid 布局定位。
fn adjust_inline_block_positions(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // 先递归处理子元素
    for child in &mut root.children {
        adjust_inline_block_positions(child, doc, styles);
    }

    let Some(container_node_id) = root.node_id else {
        return;
    };

    // 跳过 flex/grid 容器——它们的子元素由 flex/grid 布局定位
    // 跳过表格单元格——position_cells 已处理 vertical-align 定位，IFC 重新定位会覆盖
    if let Some(container_style) = styles.get(&container_node_id)
        && matches!(
            container_style.display,
            DisplayValue::Flex
                | DisplayValue::InlineFlex
                | DisplayValue::Grid
                | DisplayValue::InlineGrid
                | DisplayValue::TableCell
        )
    {
        return;
    }

    // 收集原子行内级子元素（inline-block / inline-flex / inline-grid / inline-table / img）的索引
    // 注意：绝对定位和 fixed 元素脱离正常流，不应由 IFC 重新定位
    let ib_indices: Vec<usize> = root
        .children
        .iter()
        .enumerate()
        .filter(|(_, child)| {
            // 绝对定位和 fixed 元素脱离正常流，不参与 IFC 布局
            if child.is_absolute || child.is_fixed {
                return false;
            }
            child.node_id.is_some_and(|id| {
                // <img> 替换元素始终作为原子行内级盒参与 IFC
                if let Some(node_data) = doc.get(id) {
                    if let zero_dom::NodeKind::Element(elem) = &node_data.kind {
                        if elem.local_name() == "img" {
                            return true;
                        }
                    }
                }
                styles.get(&id).is_some_and(|s| {
                    matches!(
                        s.display,
                        DisplayValue::InlineBlock
                            | DisplayValue::InlineFlex
                            | DisplayValue::InlineGrid
                            | DisplayValue::InlineTable
                    )
                })
            })
        })
        .map(|(i, _)| i)
        .collect();

    // 如果没有原子行内级子元素，无需处理
    if ib_indices.is_empty() {
        return;
    }

    // 构建 inline-block 子元素的 LayoutBox 尺寸映射
    // 包含 CSS width 或 height 为 Auto/Percentage 的元素
    // （Percentage 无法在 IFC 中直接解析，需要 taffy 布局后的结果回填）
    let ib_sizes: HashMap<NodeId, (f32, f32)> = ib_indices
        .iter()
        .filter_map(|&idx| {
            let child = &root.children[idx];
            let node_id = child.node_id?;
            let style = styles.get(&node_id)?;
            let needs_fallback = matches!(style.width, LengthValue::Auto | LengthValue::Percentage(_))
                || matches!(style.height, LengthValue::Auto | LengthValue::Percentage(_));
            if !needs_fallback {
                return None;
            }
            Some((node_id, (child.content_width, child.content_height)))
        })
        .collect();

    // 为 inline-flex/inline-grid 元素计算基线覆盖
    // CSS Flexbox §8.5: 容器基线从第一个 flex line 中参与 baseline 对齐的项合成。
    // 优先使用 taffy 计算的 first_baselines（通过 cached_baselines 补丁获取），
    // 回退到从子元素布局位置近似。
    // 仅对水平方向 flex 容器应用（Row/RowReverse），因为垂直方向的基线合成逻辑不同。
    //
    // 算法：
    // 1. 优先使用 taffy 计算的 first_baseline（如果可用）
    // 2. 回退到从第一行子元素布局位置近似
    let baseline_overrides: HashMap<NodeId, f32> = ib_indices
        .iter()
        .filter_map(|&idx| {
            let child = &root.children[idx];
            let node_id = child.node_id?;
            let style = styles.get(&node_id)?;
            // 仅对 inline-flex/inline-grid 且水平方向的容器应用
            let is_horizontal_flex = matches!(style.display, DisplayValue::InlineFlex | DisplayValue::InlineGrid)
                && matches!(
                    style.flex_direction,
                    FlexDirectionValue::Row | FlexDirectionValue::RowReverse
                );
            if !is_horizontal_flex {
                return None;
            }

            // 优先使用 taffy 缓存的基线
            if let Some(taffy_bl) = child.taffy_baseline {
                if taffy_bl > 0.0 && taffy_bl < child.content_height {
                    return Some((node_id, taffy_bl));
                }
            }

            // 回退：从子元素布局位置近似
            if child.children.is_empty() {
                return None;
            }
            // 找到第一行：y 值最小的一组子元素
            let min_y = child.children.iter().map(|c| c.y).fold(f32::MAX, f32::min);
            let first_row: Vec<_> = child.children.iter().filter(|c| (c.y - min_y).abs() < 1.0).collect();

            // 检查容器是否全局设置 align-items: baseline
            let container_align_baseline = matches!(style.align_items, AlignmentValue::Baseline);

            // 收集第一行中参与 baseline 对齐的子元素的基线贡献
            let mut baseline_contributions: Vec<f32> = Vec::new();
            let mut first_item_bottom = 0.0f32;

            for (i, c) in first_row.iter().enumerate() {
                // 从子元素的样式获取 font-size 和 align-self
                let c_font_size: f32 = c
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .map(|s| match &s.font_size {
                        LengthValue::Px(px) => *px as f32,
                        LengthValue::Em(em) => (em * 16.0) as f32,
                        LengthValue::Rem(rem) => (rem * 16.0) as f32,
                        LengthValue::Percentage(p) => (p * 16.0 / 100.0) as f32,
                        _ => 16.0,
                    })
                    .unwrap_or(c.content_height);

                // 子元素参与 baseline 对齐的条件：
                // align-self: baseline（显式），或 align-self: auto + 容器 align-items: baseline
                // align-self: stretch 是显式退出 baseline 对齐，不参与。
                let is_baseline_aligned = c
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .map(|s| {
                        matches!(s.align_self, AlignmentValue::Baseline)
                            || (container_align_baseline && matches!(s.align_self, AlignmentValue::Auto))
                    })
                    .unwrap_or(false);

                // 记录第一个子元素的底边作为回退
                if i == 0 {
                    first_item_bottom = c.y + c.content_height;
                }

                if is_baseline_aligned {
                    // 使用 font-size 近似文本基线位置
                    // 基线 = item.y + font_size（ascent 近似）
                    baseline_contributions.push(c.y + c_font_size);
                }
            }

            // 如果没有 baseline 对齐的子元素，使用第一个子元素的底边作为回退
            let baseline = if !baseline_contributions.is_empty() {
                baseline_contributions.into_iter().fold(0.0f32, f32::max)
            } else {
                first_item_bottom
            };

            if baseline > 0.0 && baseline < child.content_height {
                Some((node_id, baseline))
            } else {
                None
            }
        })
        .collect();
    // 运行 InlineFormattingContext 获取行内布局坐标
    let container_width = root.content_width;
    let is_vertical = matches!(
        root.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    );
    let is_vertical_rtl = matches!(root.writing_mode, WritingModeValue::VerticalRl);
    let container_text_align = resolve_text_align(styles.get(&container_node_id));
    // white-space: nowrap/pre 禁止换行——inline-block 超出容器宽度时应溢出而非换行。
    // 此前未把容器的 white_space 传给 IFC（no_wrap 恒 false），致 nowrap 容器内的
    // inline-block 被错误换行（flexbox_flex-*-shrink REF：div nowrap 内 4 个 inline-block
    // 总宽>容器，第 4 个被换到第二行 → 与 flex test 单行溢出不一致）。
    let no_wrap = styles
        .get(&container_node_id)
        .is_some_and(|s| matches!(s.white_space, WhiteSpaceValue::Pre | WhiteSpaceValue::Nowrap));
    let mut inline_ctx = crate::inline::InlineFormattingContext::new(container_width)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_text_align(container_text_align)
        .with_inline_block_sizes(ib_sizes)
        .with_baseline_overrides(baseline_overrides)
        .with_no_wrap(no_wrap);
    inline_ctx.layout(doc, container_node_id, styles);

    // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
    store_font_sizes_from_ifc(&inline_ctx, root);

    // 将 fragment 坐标应用到 inline-block 子元素的 LayoutBox
    // 使用 all_fragments_with_line_y() 获取包含行盒 Y 偏移的绝对坐标
    let fragments = inline_ctx.all_fragments_with_line_y();
    for idx in &ib_indices {
        let child = &mut root.children[*idx];
        let Some(child_node_id) = child.node_id else {
            continue;
        };

        // 查找匹配的 fragment（node_id 一致，font_size==0 表示 inline-block）
        if let Some(fragment) = fragments
            .iter()
            .find(|f| f.node_id == child_node_id && f.font_size == 0.0 && f.width > 0.0)
        {
            child.x = fragment.x;
            child.y = fragment.y;
        }
    }
}

/// 后处理：对 flex/grid 容器的子元素按 CSS `order` 属性排序。
///
/// CSS Flexbox §5.4: flex item 可以通过 `order` 属性改变视觉顺序。
/// taffy 0.7 不支持 CSS `order`，因此在后处理中对 flex/grid 容器的
/// 直接子元素按 `css_order` 字段排序。order 值小的排在前面。
/// order 相同时保持原始 DOM 顺序（使用原始索引作为稳定排序键）。
fn sort_children_by_css_order(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    // 先递归处理子元素
    for child in &mut root.children {
        sort_children_by_css_order(child, styles);
    }

    // 仅对 flex 或 grid 容器排序
    let is_flex_or_grid = root.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
        matches!(
            s.display,
            zero_style_system::property::types::DisplayValue::Flex
                | zero_style_system::property::types::DisplayValue::InlineFlex
                | zero_style_system::property::types::DisplayValue::Grid
                | zero_style_system::property::types::DisplayValue::InlineGrid
        )
    });

    if !is_flex_or_grid {
        return;
    }

    // 检查是否有任何 in-flow 子元素的 order 不为 0
    // （abspos 不受 `order` 重排，见 tree.rs 同源注释，不应触发排序）
    let has_non_zero_order = root.children.iter().any(|c| !c.is_absolute && c.css_order != 0);
    if !has_non_zero_order {
        return;
    }

    // 稳定排序：按 css_order 升序，order 相同时保持原始 DOM 顺序
    // 使用索引作为稳定排序键。abspos（is_absolute）强制 order=0 → stable sort
    // 保持其 DOM 相对顺序（CSS Flexbox §8.1：`order` 不重排 abspos，flexbox-paint-ordering-003）
    let mut indexed: Vec<(usize, i32)> = root
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| (i, if c.is_absolute { 0 } else { c.css_order }))
        .collect();
    indexed.sort_by_key(|&(idx, order)| (order, idx as i32));

    // 按排序后的顺序重新排列子元素
    let sorted_indices: Vec<usize> = indexed.iter().map(|&(i, _)| i).collect();
    let original = std::mem::take(&mut root.children);
    root.children = sorted_indices.iter().map(|&i| original[i].clone()).collect();
}

/// 2. 查找 abs-pos 元素在文本流中的位置
/// 3. 仅当 taffy 给出的位置明显偏离 IFC 位置时才修正
fn fix_vertical_mode_abs_pos(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // 先递归处理子元素
    for child in &mut root.children {
        fix_vertical_mode_abs_pos(child, doc, styles);
    }

    // 仅处理垂直书写模式的容器
    if !matches!(
        root.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    ) {
        return;
    }

    // 查找有 abs-pos 子元素的容器
    let has_abs_children = root.children.iter().any(|c| c.is_absolute);
    if !has_abs_children {
        return;
    }

    let Some(container_node_id) = root.node_id else {
        return;
    };

    // 仅处理作为 abs-pos 子元素 containing block 的容器。
    // CSS 2.1 §10.1：containing block 是最近的 position != static 的祖先。
    // 非 containing block 的祖先不应干预 abs-pos 元素的静态位置计算。
    let is_containing_block = styles
        .get(&container_node_id)
        .is_some_and(|s| !matches!(s.position, PositionValue::Static));
    if !is_containing_block {
        return;
    }

    // 运行 IFC（垂直模式）获取所有片段坐标
    let is_vertical = true;
    let is_vertical_rtl = matches!(root.writing_mode, WritingModeValue::VerticalRl);
    // 轴交换后：content_width = 视觉高度（行内方向），content_height = 视觉宽度（块方向）
    // IFC 的"行宽"是行内方向的可用尺寸 = 视觉高度 = content_width
    let container_width = root.content_width;
    if container_width <= 0.0 {
        return;
    }
    let container_text_align = resolve_text_align(styles.get(&container_node_id));
    let mut inline_ctx = crate::inline::InlineFormattingContext::new(container_width)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_text_align(container_text_align);
    inline_ctx.layout(doc, container_node_id, styles);

    // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
    store_font_sizes_from_ifc(&inline_ctx, root);

    // 将 IFC 片段坐标应用到 abs-pos 子元素
    let fragments = inline_ctx.all_fragments();
    for child in &mut root.children {
        if !child.is_absolute {
            continue;
        }
        let Some(child_node_id) = child.node_id else {
            continue;
        };

        // 查找匹配的 fragment（node_id 一致）
        if let Some(fragment) = fragments.iter().find(|f| f.node_id == child_node_id) {
            // 仅在所有 inset 为 auto 时才修正静态位置
            let style = styles.get(&child_node_id);
            let all_inset_auto = style.is_some_and(|s| {
                matches!(s.top, zero_css_parser::values::LengthValue::Auto)
                    && matches!(s.bottom, zero_css_parser::values::LengthValue::Auto)
            });

            if all_inset_auto {
                // IFC 提供的静态位置比 taffy 的水平模型更准确
                // 始终使用 IFC 位置（仅在有差异时更新）
                let dx = (child.x - fragment.x).abs();
                let dy = (child.y - fragment.y).abs();
                if dx > 0.01 || dy > 0.01 {
                    child.x = fragment.x;
                    child.y = fragment.y;
                }

                // CSS §10.3.7 + writing-modes §7.1：vertical-rl 下 abspos 的物理
                // height（= inline 轴跨度）在 height:auto 时应 shrink-to-fit 到内容
                // inline 跨度，而非填满 CB cross-axis。taffy 把 auto height 当
                // cross-axis stretch（给 320=CB 高），fragment.width 是 IFC 计算的
                // 内容 inline 跨度（垂直模式下 = 单行/字形的视觉竖向高度）。
                // 仅当 style.height 为 auto 时收缩（尊重显式 height）。
                let height_auto = style.is_some_and(|s| matches!(s.height, zero_css_parser::values::LengthValue::Auto));
                if height_auto {
                    let content_h = fragment.width.max(fragment.font_size);
                    if (child.height - content_h).abs() > 0.01 && content_h > 0.0 {
                        child.height = content_h;
                        // content_height 同步（无 border/padding 时 = height）
                        child.content_height = child.content_height.min(content_h).max(0.0);
                    }
                }

                // CSS §10.3.7 + writing-modes §7.1：direction:rtl 下 abspos 静态位置镜像。
                // all-three-auto（top/bottom 即 left/right 均为 auto）时，ltr 把 inline-start
                // 边（=top 角色）置静态位置、内容自 inline-start 向 end 排；rtl 把 inline-end
                // 边（=bottom 角色）置静态位置、内容反向排。两者最终盒位沿 inline 轴镜像：
                //   rtl_top = CB_inline_extent - ltr_top - height
                // container_width 在垂直模式 = CB 视觉高度（inline 可用尺寸，见上方注释）。
                // 旧实现在 rtl 下与 ltr 渲染完全相同（诊断实证），致 abs-pos-non-replaced-vrl
                // 的 rtl 子集（012/122/130 ~5%）远高于 ltr（002 ~1.3%）。
                let cb_direction_rtl = styles
                    .get(&container_node_id)
                    .is_some_and(|s| matches!(s.direction, zero_style_system::property::types::DirectionValue::Rtl));
                if cb_direction_rtl {
                    child.y = (container_width - child.y - child.height).max(0.0);
                }
            }
        }
    }
}

/// 已禁用：taffy 0.7 已在 layout.location 中包含 position:relative 的 inset 偏移，
/// 不需要额外后处理。保留此函数供参考和潜在的未来使用。
#[allow(dead_code)]
/// 对 inline-level position:relative 元素应用视觉偏移。
///
/// taffy 已在 layout.location 中包含 block-level 元素的 relative inset，
/// 因此只需处理 inline-level 元素（如 <img>、<span> 等由 inline layout 定位的元素）。
/// 对 block-level 元素跳过，避免双重偏移。
fn apply_relative_offsets_inline(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    let is_rel = root.node_id.is_some_and(|id| {
        styles
            .get(&id)
            .is_some_and(|s| matches!(s.position, PositionValue::Relative))
    });

    if is_rel {
        // 仅对真正的 inline-level 元素应用偏移
        // block-level 元素的 relative offset 已由 taffy 处理
        // table 内部元素（row-group/row/cell 等）由 table 布局算法处理
        let is_inline_level = root.node_id.is_some_and(|id| {
            styles
                .get(&id)
                .is_some_and(|s| matches!(s.display, DisplayValue::Inline | DisplayValue::InlineBlock))
        });
        if is_inline_level {
            // R109 §9.2.1.1：split inline（display:inline，converter 映射为 taffy Block）
            // 及其匿名块片段共享 inline 的 node_id。taffy 已按 block 单次施加 relative
            // offset；此处再按 computed-Inline 施加会双重计数（inline-box-002 的
            // position:relative;top:2in 致片段偏低 2×192px 出视口）。is_r109_split 对
            // 父盒与片段均为 true，整体跳过让 taffy 单次处理。
            if !root.is_r109_split {
                let (dx, dy) = resolve_relative_inset(root, styles);
                if dx != 0.0 || dy != 0.0 {
                    root.x += dx;
                    root.y += dy;
                }
            }
        }
    }
    for child in &mut root.children {
        apply_relative_offsets_inline(child, styles);
    }
}

/// Final Inline Layout Pass（Phase A）。
///
/// 后处理：修正 `calc(P% ± Npx)` 计算的尺寸。
///
/// taffy 不支持 calc 表达式。converter 将 `calc(100% - 6px)` 近似为 `Percent(1.0)`，
/// taffy 按百分比计算出正确的基准尺寸，但缺少 px 偏移量的修正。
/// 此函数遍历布局树，对使用了 calc 的 width/height 属性施加 px 偏移量修正。
fn apply_calc_size_adjustments(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    for child in &mut root.children {
        apply_calc_size_adjustments(child, styles);
    }

    let Some(node_id) = root.node_id else { return };
    let Some(style) = styles.get(&node_id) else { return };

    // 检查 width 是否为 calc(P% ± Npx) 模式
    if let LengthValue::Calc(expr) = &style.width {
        if let Some((pct, px_offset)) = extract_calc_percentage_and_offset(expr) {
            let base_width = pct / 100.0 * root.width as f64;
            let adjusted = (base_width + px_offset).max(0.0) as f32;
            if (adjusted - root.width).abs() > 0.01 {
                let diff = adjusted - root.width;
                root.width = adjusted;
                root.content_width = (root.content_width + diff).max(0.0);
            }
        }
    }

    // 检查 height 是否为 calc(P% ± Npx) 模式
    if let LengthValue::Calc(expr) = &style.height {
        if let Some((pct, px_offset)) = extract_calc_percentage_and_offset(expr) {
            let base_height = pct / 100.0 * root.height as f64;
            let adjusted = (base_height + px_offset).max(0.0) as f32;
            if (adjusted - root.height).abs() > 0.01 {
                let diff = adjusted - root.height;
                root.height = adjusted;
                root.content_height = (root.content_height + diff).max(0.0);
            }
        }
    }
}

/// 自上而下收紧百分比 max-height。
///
/// `cb_content_height` 为父级（包含块）的**明确**内容高度；为 `None` 表示父级高度
/// 由内容决定（CSS §10.5：此时百分比 height/max-height 视为 auto，不解析）。
fn clamp_percentage_max_height(
    box_node: &mut LayoutBox,
    cb_content_height: Option<f32>,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::{BoxSizingValue, LengthValue};

    // absolute 元素的包含块语义不同（由 positioned ancestor / 视口决定），
    // 不在此处处理，避免与 adjust_absolute_pct_to_viewport 重叠。
    let style = if box_node.is_absolute {
        None
    } else {
        box_node.node_id.and_then(|id| styles.get(&id))
    };

    // 1) 收紧：百分比 max-height 相对包含块内容高度解析
    if let (Some(style), Some(cb_h)) = (style.as_ref(), cb_content_height)
        && let LengthValue::Percentage(p) = &style.max_height
    {
        let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
        let is_border_box = matches!(style.box_sizing, BoxSizingValue::BorderBox);
        // max-height 按 box-sizing 作用在边框盒或内容盒
        let max_box_h = *p as f32 / 100.0 * cb_h;
        let max_content_h = if is_border_box {
            (max_box_h - pb).max(0.0)
        } else {
            max_box_h
        };
        if box_node.content_height > max_content_h {
            let clamped = max_content_h;
            box_node.content_height = clamped;
            box_node.height = clamped + pb;
        }
    }

    // 1.5) Table 高度作为内容高度下限（CSS 2.1 §17.5.3）。
    // table 后处理（apply_table_size_constraints）此前完全忽略 style.height，仅用
    // intrinsic 行高填表格高度。CSS 规定 table 的 'height' 是内容高度的「下限」
    // （min 语义）：表格至少这么高，内容更高则增长。此处把 style.height
    // （Px 或可解析百分比）解析为内容高度下限，与已计算的 content_height 取 max。
    // 在此自上而下 pass 中处理以复用 cb_content_height 的「明确高度」语义：
    // 百分比仅当包含块高度明确时才解析，否则忽略（CSS §10.5）。
    if let Some(s) = style {
        let is_table = matches!(
            s.display,
            zero_css_parser::values::DisplayValue::Table | zero_css_parser::values::DisplayValue::InlineTable
        );
        if is_table {
            let specified_content_h: Option<f32> = match &s.height {
                LengthValue::Px(v) => {
                    let pb =
                        box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                    let c = if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                        (*v as f32 - pb).max(0.0)
                    } else {
                        *v as f32
                    };
                    Some(c)
                }
                LengthValue::Percentage(p) => cb_content_height.map(|cb| *p as f32 / 100.0 * cb),
                _ => None,
            };
            if let Some(spec) = specified_content_h
                && box_node.content_height < spec
            {
                let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                box_node.content_height = spec;
                box_node.height = spec + pb;
            }
        }
    }

    // 2) 计算本盒的「明确内容高度」供子元素百分比解析：
    //    - height: Px → 明确（按 box-sizing 折算内容高）
    //    - height: Percentage 且包含块明确 → 解析后明确
    //    - 其他（auto / 内容决定）→ 不明确，子元素百分比不解析
    let my_definite_content_height = style.and_then(|s| match &s.height {
        LengthValue::Px(v) => {
            let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
            let is_border_box = matches!(s.box_sizing, BoxSizingValue::BorderBox);
            let content = if is_border_box {
                (*v as f32 - pb).max(0.0)
            } else {
                *v as f32
            };
            Some(content)
        }
        LengthValue::Percentage(p) => cb_content_height.map(|cb| *p as f32 / 100.0 * cb),
        _ => None,
    });

    for child in &mut box_node.children {
        clamp_percentage_max_height(child, my_definite_content_height, styles);
    }
}

/// 从 calc 表达式中提取百分比和 px 偏移量。
///
/// 对于 `calc(100% - 6px)`，返回 `Some((100.0, -6.0))`。
/// 对于 `calc(50% + 10px)`，返回 `Some((50.0, 10.0))`。
/// 仅支持 `P% ± Npx` 和纯 `P%` 模式。
fn extract_calc_percentage_and_offset(expr: &zero_css_parser::values::CalcExpr) -> Option<(f64, f64)> {
    use zero_css_parser::values::{CalcExpr, CalcOp, LengthValue};
    match expr {
        CalcExpr::Length(LengthValue::Percentage(pct)) => Some((*pct, 0.0)),
        CalcExpr::BinaryOp(left, op, right) => {
            let left_pct = match left.as_ref() {
                CalcExpr::Length(LengthValue::Percentage(pct)) => Some(*pct),
                _ => None,
            };
            let left_px = match left.as_ref() {
                CalcExpr::Length(LengthValue::Px(v)) => Some(*v),
                _ => None,
            };
            let right_pct = match right.as_ref() {
                CalcExpr::Length(LengthValue::Percentage(pct)) => Some(*pct),
                _ => None,
            };
            let right_px = match right.as_ref() {
                CalcExpr::Length(LengthValue::Px(v)) => Some(*v),
                _ => None,
            };

            match (op, left_pct, left_px, right_pct, right_px) {
                // P% - Npx
                (CalcOp::Subtract, Some(pct), _, None, Some(px)) => Some((pct, -px)),
                // P% + Npx
                (CalcOp::Add, Some(pct), _, None, Some(px)) => Some((pct, px)),
                // Npx - P% (unusual but valid)
                (CalcOp::Subtract, None, Some(_px), Some(_pct), _) => None,
                // Npx + P%
                (CalcOp::Add, None, Some(px), Some(pct), _) => Some((pct, px)),
                // P% - P% (not handled)
                (CalcOp::Subtract, Some(_), _, Some(_), _) => None,
                _ => None,
            }
        }
        _ => None,
    }
}

/// 全元素 position:relative 偏移（已弃用 — 会与 taffy block-level 偏移双重计数）。
/// 保留供参考，新代码使用 apply_relative_offsets_inline。
#[allow(dead_code)]
fn apply_relative_offsets(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    // 仅对 position:relative 应用视觉偏移（不含 sticky，sticky 偏移需宿主层滚动驱动）
    let is_rel = root.node_id.is_some_and(|id| {
        styles
            .get(&id)
            .is_some_and(|s| matches!(s.position, PositionValue::Relative))
    });
    if is_rel {
        let (dx, dy) = resolve_relative_inset(root, styles);
        if dx != 0.0 || dy != 0.0 {
            root.x += dx;
            root.y += dy;
        }
    }
    for child in &mut root.children {
        apply_relative_offsets(child, styles);
    }
}

/// 从 ComputedStyle 中解析 position:relative 的 top/left 偏移量。
#[allow(dead_code)]
fn resolve_relative_inset(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> (f32, f32) {
    use zero_css_parser::values::LengthValue;
    let Some(node_id) = box_node.node_id else {
        return (0.0, 0.0);
    };
    let Some(style) = styles.get(&node_id) else {
        return (0.0, 0.0);
    };
    let dx = match &style.left {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    };
    let dy = match &style.top {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    };
    (dx, dy)
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
        // R324：fixed 元素须视口相对。taffy 0.7 把 fixed 当 absolute 处理（containing
        // block = 最近 positioned 祖先），故 box.x/y 编码的是相对该祖先的 left/top。
        // 视口相对 = 同一 left/top 数值但相对视口 → 需从累积祖先偏移中【扣除】
        //（而非旧实现的「加上」——旧实现仅在 parent_offset==0 时碰巧正确，对有偏移
        // positioned 祖先的 fixed 会 over-correct，如 fixed-inside-relative-ancestor）。
        box_node.x -= parent_offset_x;
        box_node.y -= parent_offset_y;
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

/// 将没有 positioned ancestor 的 absolute 元素修正为相对于初始包含块。
///
/// 仅对 `position:absolute` 且路径上不存在 `position != static` 祖先的元素生效。
/// 这避免 body 的外边距或静态祖先的偏移被重复计入 abs-pos 元素坐标。
///
/// 注意：此功能当前导致多个回归，暂不启用。
#[allow(dead_code)]
fn adjust_absolute_to_initial_containing_block(
    box_node: &mut LayoutBox,
    current_content_origin_x: f32,
    current_content_origin_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    has_positioned_ancestor: bool,
) {
    let child_has_positioned_ancestor = has_positioned_ancestor
        || box_node.is_absolute
        || box_node.is_fixed
        || box_node.is_relative
        || box_node.is_sticky;

    for child in &mut box_node.children {
        // 使用 child_has_positioned_ancestor 而非 has_positioned_ancestor，
        // 因为当前节点自身（如 position:relative）也是 positioned ancestor。
        if child.is_absolute && !child_has_positioned_ancestor {
            child.x -= current_content_origin_x;
            child.y -= current_content_origin_y;

            if let Some(style) = child.node_id.and_then(|node_id| styles.get(&node_id)) {
                if matches!(style.width, zero_css_parser::values::LengthValue::Auto) {
                    child.width += (viewport_width - box_node.content_width).max(0.0);
                }
                if matches!(style.height, zero_css_parser::values::LengthValue::Auto) {
                    child.height += (viewport_height - box_node.content_height).max(0.0);
                }
            }
        }

        let child_content_origin_x = current_content_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_content_origin_y = current_content_origin_y + box_node.border_top + box_node.padding_top + child.y;
        adjust_absolute_to_initial_containing_block(
            child,
            child_content_origin_x,
            child_content_origin_y,
            viewport_width,
            viewport_height,
            styles,
            child_has_positioned_ancestor || child.is_absolute,
        );
    }
}

/// 修正无 positioned ancestor 的 absolute 元素的**百分比** inset 与尺寸。
///
/// CSS 2.1 §10.1：absolute 元素无 positioned ancestor 时，containing block 是
/// 初始包含块（视口）。但 taffy 用静态父作为 containing block，导致 `width:50%`、
/// `left:50%` 等百分比按父宽度（而非视口宽度）解析。
///
/// 本函数**只重解析百分比**（Length/Percent::Auto 不动），避免历史上
/// `adjust_absolute_to_initial_containing_block` 因同时调整 x/y 偏移和 auto 宽高
/// 导致的回归（static-inside-inline-block、background-329 等）。
///
/// 坐标系：LayoutBox.x/y 相对父内容盒原点。paint 链逐层累加得到视口绝对坐标。
/// `current_content_origin_x/y` 是当前盒内容盒原点的视口绝对坐标。
fn adjust_absolute_pct_to_viewport(
    box_node: &mut LayoutBox,
    current_content_origin_x: f32,
    current_content_origin_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    has_positioned_ancestor: bool,
) {
    use zero_css_parser::values::LengthValue;
    let child_has_positioned_ancestor = has_positioned_ancestor
        || box_node.is_absolute
        || box_node.is_fixed
        || box_node.is_relative
        || box_node.is_sticky;

    for child in &mut box_node.children {
        if child.is_absolute
            && !child_has_positioned_ancestor
            && let Some(style) = child.node_id.and_then(|node_id| styles.get(&node_id))
        {
            // 仅当 width 为百分比时按视口重解析
            if let LengthValue::Percentage(p) = &style.width {
                child.width = *p as f32 / 100.0 * viewport_width;
            }
            if let LengthValue::Percentage(p) = &style.height {
                child.height = *p as f32 / 100.0 * viewport_height;
            }
            // auto 尺寸 + 全长度 inset → stretch（CSS §10.3.18 / §10.6.4，仅非替换）。
            // 仅当 left+right（或 top+bottom）均为长度且尺寸为 auto 时按视口 CB
            // stretch；与历史 adjust_absolute_to_initial_containing_block 的「无条件
            // 扩张 auto 宽高」（width += viewport - content，致 static-inside-inline-block
            // / background-329 回归）不同——本块严格匹配 spec 的「双 inset 才 stretch」，
            // 不动 x/y（位置已由下方 Px left/top 块设好）。`!is_replaced` 仅守卫本块：
            // §10.3.8 替换元素 auto 尺寸按固有尺寸解析（非 stretch），但 §10.1.4 的
            // viewport-CB 定位与百分比尺寸对替换/非替换同等适用，故守卫不扩到整分支
            // （避免误关 R98 位置/百分比尺寸解析，致替换 abspos 定位回退）。
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (viewport_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (viewport_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // left/top 百分比：目标视口绝对坐标 = p/100 * viewport，转回父相对坐标
            if let LengthValue::Percentage(p) = &style.left {
                let target_viewport_x = *p as f32 / 100.0 * viewport_width;
                child.x = target_viewport_x - current_content_origin_x;
            }
            if let LengthValue::Percentage(p) = &style.top {
                let target_viewport_y = *p as f32 / 100.0 * viewport_height;
                child.y = target_viewport_y - current_content_origin_y;
            }
            // left/top 为长度（Px）时：CSS 2.1 §10.1 规定无 positioned ancestor 的
            // absolute 元素以初始包含块（视口）为 containing block。taffy 用静态父
            // 作 containing block，导致 top:118px 解析为静态父相对坐标。此处把目标
            // 视口坐标（= px 值）转回父相对坐标，与百分比路径同机制（不调整 auto
            // 宽高，避免历史上 auto 宽高扩张导致的回归）。
            if let LengthValue::Px(px) = &style.left {
                child.x = (*px as f32) - current_content_origin_x;
            }
            if let LengthValue::Px(px) = &style.top {
                child.y = (*px as f32) - current_content_origin_y;
            }
            // right/bottom 为长度且 left/top 为 auto 时：CSS 2.1 §10.1 无 positioned
            // ancestor 的 absolute 元素 CB=视口。left:auto + right:Px → 右边对齐视口
            // 右缘，由已解析的 width 反解 left（§10.3.18 rule 2）：
            // target_x = viewport_w - right - width。须在 width/height 解析后执行
            // （上方百分比/auto-stretch 块已设好 child.width/height）。left/top 已为
            // Px 时由上方块处理；双 inset 全 Px 的 over-constrained（LTR）忽略 right。
            // right/bottom 百分比仅当对应尺寸为 auto 时才影响位置，当前不处理。
            if matches!(style.left, LengthValue::Auto)
                && let LengthValue::Px(right) = &style.right
            {
                let target_viewport_x = viewport_width - (*right as f32) - child.width;
                child.x = target_viewport_x - current_content_origin_x;
            }
            if matches!(style.top, LengthValue::Auto)
                && let LengthValue::Px(bottom) = &style.bottom
            {
                let target_viewport_y = viewport_height - (*bottom as f32) - child.height;
                child.y = target_viewport_y - current_content_origin_y;
            }
        }

        // 递归：用（可能已修改的）child 位置计算其内容盒原点
        let child_content_origin_x = current_content_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_content_origin_y = current_content_origin_y + box_node.border_top + box_node.padding_top + child.y;
        adjust_absolute_pct_to_viewport(
            child,
            child_content_origin_x,
            child_content_origin_y,
            viewport_width,
            viewport_height,
            styles,
            child_has_positioned_ancestor || child.is_absolute,
        );
    }
}

/// 对 position:fixed 元素的全-inset stretch 尺寸后处理（CSS §10.3.18 / §10.6.4）。
///
/// fixed 元素的 containing block 是视口。当 top+bottom 均为长度且 height:auto 时，
/// height = viewport_h - top - bottom；left+right 均为长度且 width:auto 时，
/// width = viewport_w - left - right。taffy 0.7 把 fixed 当 absolute 处理
/// （CB=最近 positioned 祖先），尺寸按该祖先而非视口 stretch，导致全-inset fixed
/// 元素尺寸不足（典型：全 0 inset 应覆盖视口却塌缩为内容固有尺寸）。
///
/// 仅处理 fixed（CB=视口无条件已知，零位置风险——位置已由 adjust_fixed_to_viewport
/// 修正）。不处理 absolute（CB=positioned 祖先，layout 后方知；历史
/// adjust_absolute_to_initial_containing_block 同调 auto 宽高致多回归故禁用）。
fn stretch_fixed_to_viewport_size(
    box_node: &mut LayoutBox,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if child.is_fixed
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // height: auto + 全长度 top+bottom → stretch
            if matches!(style.height, LengthValue::Auto)
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (viewport_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // width: auto + 全长度 left+right → stretch
            if matches!(style.width, LengthValue::Auto)
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (viewport_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            // 百分比尺寸：fixed 的 CB 恒为视口（CSS §10.1），百分比相对视口解析。
            // taffy 按 positioned 祖先解析（如 body CB），此处按视口重算。
            if let LengthValue::Percentage(p) = &style.height {
                child.height = (*p as f32 / 100.0) * viewport_height;
            }
            if let LengthValue::Percentage(p) = &style.width {
                child.width = (*p as f32 / 100.0) * viewport_width;
            }
        }
        stretch_fixed_to_viewport_size(child, viewport_width, viewport_height, styles);
    }
}

/// 对「containing block = 根元素（positioned root）」的 abspos 元素按根 padding-box
/// 重解析百分比尺寸与 Px/百分比 inset 位置（CSS §10.1.2/§10.3.18/§10.6.4）。
///
/// taffy 0.7 的 root quirk：当根元素（如 `<html style="position:relative">`）是 abspos
/// 后代的最近 positioned 祖先时，taffy 不把根当作 CB，而是误用静态父（如 body），
/// 致 abspos 百分比尺寸按父宽度解析、位置偏移（abspos-containing-block-005/006 实证，
/// 对照 bottom-offset-percentage-001 的**非根** positioned 祖先 `#div1` taffy 正确）。
/// 与 R123（根 relative inset 不应用）同属 taffy root quirk 谱系。本 pass 在 extract 后
/// 按根 padding-box 补解析。
///
/// 仅处理「最近 positioned 祖先 = 根」的 abspos（`nearest_pos_ancestor_is_root`）：
/// 非根 positioned 祖先（如 `#div1`）由 taffy 正确处理，本 pass 通过递归把
/// `nearest_pos_ancestor_is_root` 在遇到任何非根 positioned 元素时置 false，不介入。
#[allow(clippy::too_many_arguments)]
fn resolve_abspos_against_root_cb(
    box_node: &mut LayoutBox,
    current_box_origin_x: f32,
    current_box_origin_y: f32,
    cb_origin_x: f32,
    cb_origin_y: f32,
    cb_width: f32,
    cb_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    nearest_pos_ancestor_is_root: bool,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if child.is_absolute
            && nearest_pos_ancestor_is_root
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // 百分比尺寸：相对根 padding-box（CB）
            if let LengthValue::Percentage(p) = &style.width {
                child.width = *p as f32 / 100.0 * cb_width;
            }
            if let LengthValue::Percentage(p) = &style.height {
                child.height = *p as f32 / 100.0 * cb_height;
            }
            // auto 尺寸 + 全长度 inset → stretch（§10.3.18/§10.6.4，仅非替换）
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (cb_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (cb_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // left/top 百分比：目标视口绝对坐标 = cb_origin + p% * cb，转回父相对坐标
            if let LengthValue::Percentage(p) = &style.left {
                let target_x = cb_origin_x + *p as f32 / 100.0 * cb_width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let LengthValue::Percentage(p) = &style.top {
                let target_y = cb_origin_y + *p as f32 / 100.0 * cb_height;
                child.y = target_y - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // left/top Px：目标视口绝对坐标 = cb_origin + px
            if let LengthValue::Px(px) = &style.left {
                child.x =
                    cb_origin_x + (*px as f32) - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let LengthValue::Px(px) = &style.top {
                child.y =
                    cb_origin_y + (*px as f32) - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // right/bottom Px 且 left/top 为 auto：右/下边对齐 CB 右/下缘（§10.3.18 rule 2）
            if matches!(style.left, LengthValue::Auto)
                && let LengthValue::Px(right) = &style.right
            {
                let target_x = cb_origin_x + cb_width - (*right as f32) - child.width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if matches!(style.top, LengthValue::Auto)
                && let LengthValue::Px(bottom) = &style.bottom
            {
                let target_y = cb_origin_y + cb_height - (*bottom as f32) - child.height;
                child.y = target_y - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
        }

        // 递归：遇到非根 positioned 元素时，其后代的最近 positioned 祖先不再是根 → false
        let child_nearest_is_root = if child.is_absolute || child.is_fixed || child.is_relative || child.is_sticky {
            false
        } else {
            nearest_pos_ancestor_is_root
        };
        let child_box_origin_x = current_box_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_box_origin_y = current_box_origin_y + box_node.border_top + box_node.padding_top + child.y;
        resolve_abspos_against_root_cb(
            child,
            child_box_origin_x,
            child_box_origin_y,
            cb_origin_x,
            cb_origin_y,
            cb_width,
            cb_height,
            styles,
            child_nearest_is_root,
        );
    }
}

#[cfg(test)]
mod tests;
