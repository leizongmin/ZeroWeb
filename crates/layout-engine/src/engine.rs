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
use crate::inline::{FloatExclusion, InlineFormattingContext, TextAlign};
use crate::tree::{R109Wiring, build_layout_tree_with_r109};
use crate::types::{LayoutBox, LayoutResult, OverflowClip};
use zero_style_system::WritingModeValue;

/// 从 ComputedStyle 读取 text-align 并转换为 IFC 的 TextAlign 枚举。
fn resolve_text_align(style: Option<&ComputedStyle>) -> TextAlign {
    use zero_style_system::property::TextAlignValue;
    let align = style.map(|s| &s.text_align).unwrap_or(&TextAlignValue::Start);
    match align {
        TextAlignValue::Left | TextAlignValue::Start => TextAlign::Left,
        TextAlignValue::Right | TextAlignValue::End => TextAlign::Right,
        TextAlignValue::Center => TextAlign::Center,
        TextAlignValue::Justify => TextAlign::Justify,
    }
}

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
        shrink_inline_blocks_to_content(&mut root_box, styles);

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

        // 12. 后处理：Final Inline Layout Pass（Phase A）。
        // 为含有直接文本子节点的容器计算最终行内布局并存储结果。
        // paint 系统消费存储的 IFC 结果，不再重跑 IFC。
        compute_final_inline_layouts(&mut root_box, doc, styles);

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
        // CSS 2.1：positioned 元素 + z-index 为显式整数时创建堆叠上下文。
        // z-index: auto 不创建堆叠上下文——其 positioned 后代参与父级堆叠上下文。
        let creates_stacking_context =
            is_positioned && computed.is_some_and(|s| matches!(s.z_index, ZIndexValue::Integer(_)));

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
    let mut inline_ctx = crate::inline::InlineFormattingContext::new(container_width)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_text_align(container_text_align)
        .with_inline_block_sizes(ib_sizes)
        .with_baseline_overrides(baseline_overrides);
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

/// 将 IFC 片段结果存储到 LayoutBox.inline_layout，供 paint 系统复用。
///
/// 避免在 paint 阶段重新运行 IFC（paint IFC 使用空 styles 导致字体度量不一致）。
/// TODO: 当前被注释掉 — 基线计算修复后启用
#[allow(dead_code)]
fn store_inline_layout_results(
    inline_ctx: &crate::inline::InlineFormattingContext,
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if !inline_ctx.lines.is_empty() {
        let container_width = box_node.content_width;
        let stored: Vec<crate::types::InlineLayoutLine> = inline_ctx
            .lines
            .iter()
            .map(|line| crate::types::InlineLayoutLine {
                y: line.y,
                height: line.height,
                fragments: line
                    .runs
                    .iter()
                    .map(|frag| {
                        let is_ahem = box_node.node_id.is_some_and(|id| {
                            styles
                                .get(&id)
                                .is_some_and(|s| s.font_family.contains(&"Ahem".to_string()))
                        });
                        crate::types::InlineLayoutFragment {
                            x: frag.x,
                            y: frag.y,
                            width: frag.width,
                            height: frag.height,
                            font_size: frag.font_size,
                            is_ahem,
                            text: frag.text.clone(),
                            node_id: Some(frag.node_id),
                        }
                    })
                    .collect(),
            })
            .collect();
        box_node.inline_layout = Some(stored);
        box_node.inline_layout_width = container_width;
    }
}

/// 从 IFC 片段中提取各文本节点的 font_size、is_ahem 标志、letter-spacing 和 line-height 并存储到 LayoutBox。
///
/// paint 系统在运行空 styles IFC 时无法获取正确的 font_size、字体信息、letter-spacing 和 line-height，
/// 导致基线偏移、字符宽度、间距和行盒高度计算错误。通过此函数存储 layout IFC 的相关值，
/// paint 可以在渲染时使用正确的值。
fn store_font_sizes_from_ifc(inline_ctx: &crate::inline::InlineFormattingContext, box_node: &mut LayoutBox) {
    for line in &inline_ctx.lines {
        for frag in &line.runs {
            box_node.text_node_font_sizes.insert(frag.node_id, frag.font_size);
            box_node.text_node_is_ahem.insert(frag.node_id, frag.is_ahem);
            box_node
                .text_node_letter_spacing
                .insert(frag.node_id, frag.letter_spacing);
            // line-height 不影响行断（仅影响垂直定位），传递到 paint IFC 是安全的。
            // 使用片段的 height 作为行盒高度贡献（已含 line-height + padding + border）。
            box_node.text_node_line_heights.insert(frag.node_id, frag.height);
            // 内联元素片段（node_id 是元素 NodeId 而非文本节点 NodeId）：
            // 存储其 (font_size, line_height) 供 paint IFC 使用。
            // 内联元素在 paint IFC 中无法获取自己的样式，导致使用默认值。
            // line_height 近似使用 height（对文本片段来说等于 run.line_height）。
            box_node
                .inline_element_metrics
                .insert(frag.node_id, (frag.font_size, frag.height));
            // 内联元素的水平 margin 不影响行断（仅影响水平偏移），传递到 paint IFC 是安全的。
            box_node
                .inline_element_margins
                .insert(frag.node_id, (frag.margin_left, frag.margin_right));
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct InlineVisualMetrics {
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
    padding_left: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    border_left: f32,
}

fn resolve_px_length(value: &LengthValue) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    }
}

fn extract_inline_visual_metrics(style: &ComputedStyle) -> InlineVisualMetrics {
    InlineVisualMetrics {
        padding_top: resolve_px_length(&style.padding_top),
        padding_right: resolve_px_length(&style.padding_right),
        padding_bottom: resolve_px_length(&style.padding_bottom),
        padding_left: resolve_px_length(&style.padding_left),
        border_top: resolve_px_length(&style.border_top_width),
        border_right: resolve_px_length(&style.border_right_width),
        border_bottom: resolve_px_length(&style.border_bottom_width),
        border_left: resolve_px_length(&style.border_left_width),
    }
}

/// 将 IFC 计算出的直接 inline 子元素几何写回 LayoutBox。
///
/// 仅处理「单个 fragment 即可完整表示」的简单 inline 元素：
/// - `display:inline`
/// - 非 absolute/fixed
/// - 在当前 IFC 中恰好对应一个 fragment
///
/// 这样可以让 paint 阶段使用更接近真实 inline box 的几何去绘制背景/边框，
/// 避免 taffy 将 inline 元素当作 block 后得到的零尺寸或错误尺寸。
fn sync_inline_child_boxes_from_ifc(
    box_node: &mut LayoutBox,
    inline_ctx: &InlineFormattingContext,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let fragments = inline_ctx.all_fragments_with_line_y();

    for child in &mut box_node.children {
        if child.is_block_level || child.is_absolute || child.is_fixed {
            continue;
        }

        let Some(child_id) = child.node_id else {
            continue;
        };
        let Some(style) = styles.get(&child_id) else {
            continue;
        };
        if !matches!(style.display, DisplayValue::Inline) {
            continue;
        }

        let mut matching = fragments.iter().filter(|fragment| fragment.node_id == child_id);
        let Some(fragment) = matching.next() else {
            continue;
        };
        if matching.next().is_some() {
            continue;
        }
        // 跳过含文本内容的 fragment：
        // 文本 fragment 的位置来自 layout IFC（使用真实样式），
        // 而 paint 阶段运行独立的 paint IFC（使用空样式），
        // 两者行断行为不同，直接使用 layout IFC 坐标会导致背景与文字错位。
        // 仅对空 inline 元素（零宽度 TextRun）应用几何修正。
        if !fragment.text.is_empty() {
            continue;
        }

        let metrics = extract_inline_visual_metrics(style);
        child.x = fragment.x;
        child.y = fragment.y - metrics.padding_top - metrics.border_top;
        child.width =
            fragment.width + metrics.padding_left + metrics.padding_right + metrics.border_left + metrics.border_right;
        child.height =
            fragment.height + metrics.padding_top + metrics.padding_bottom + metrics.border_top + metrics.border_bottom;
        child.content_x = metrics.border_left + metrics.padding_left;
        child.content_y = metrics.border_top + metrics.padding_top;
        child.content_width = fragment.width;
        child.content_height = fragment.height;
        child.padding_top = metrics.padding_top;
        child.padding_right = metrics.padding_right;
        child.padding_bottom = metrics.padding_bottom;
        child.padding_left = metrics.padding_left;
        child.border_top = metrics.border_top;
        child.border_right = metrics.border_right;
        child.border_bottom = metrics.border_bottom;
        child.border_left = metrics.border_left;
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

    // 检查是否有任何子元素的 order 不为 0
    let has_non_zero_order = root.children.iter().any(|c| c.css_order != 0);
    if !has_non_zero_order {
        return;
    }

    // 稳定排序：按 css_order 升序，order 相同时保持原始 DOM 顺序
    // 使用索引作为稳定排序键
    let mut indexed: Vec<(usize, i32)> = root
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.css_order))
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

/// 为含有直接文本子节点的容器计算最终行内布局并存储 IFC 片段结果。
/// paint 系统消费这些结果渲染文字，不再重跑 IFC。
///
/// 使用与 paint-IFC 相同的空样式 + override maps 上下文，
/// 确保存储结果与 paint 路径完全一致，零回归。
fn compute_final_inline_layouts(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // 先递归处理子节点
    for child in &mut root.children {
        compute_final_inline_layouts(child, doc, styles);
    }

    // 仅处理有 node_id 且含有直接文本子节点的容器
    let Some(node_id) = root.node_id else { return };
    let Some(_) = doc.get(node_id) else { return };

    // 跳过 flex/grid/table 容器（它们不需要独立的 inline layout）
    let Some(style) = styles.get(&node_id) else { return };
    use zero_css_parser::values::DisplayValue;
    if matches!(
        style.display,
        DisplayValue::Flex
            | DisplayValue::InlineFlex
            | DisplayValue::Grid
            | DisplayValue::InlineGrid
            | DisplayValue::Table
            | DisplayValue::InlineTable
    ) {
        return;
    }

    // 跳过多列容器（多列在 paint 阶段按列分配 IFC 内容，不适合预存储）
    if root.is_multicol {
        return;
    }

    // 跳过非块级元素（display: inline）：
    // 这些元素的文本内容已经参与父级 IFC 排列，不需要单独存储。
    // 如果为它们也存储 inline_layout，paint 系统会双重渲染文本——
    // 一次从父级 IFC（含 float exclusion），一次从自身 IFC（无 float exclusion），
    // 导致文本与 float 重叠。
    // inline-block/inline-flex/inline-grid 虽然也是 inline-level，
    // 但它们有独立的布局上下文，is_block_level 不会是 false。
    if !root.is_block_level {
        return;
    }

    // 检查是否有直接文本子节点
    let mut has_text_children = root.children.iter().any(|c| c.is_anonymous_text_item)
        || doc
            .child_nodes(node_id)
            .iter()
            .any(|child_id| doc.get(*child_id).is_some_and(|n| matches!(&n.kind, NodeKind::Text(_))));
    // PHASEA stored-line-boxes 路径（默认启用；env PHASEA_STORE_EXT=0 关闭）：也覆盖含 **inline-level** 元素子节点且**无 block-level
    // 元素子节点**的容器（纯 inline 内容，如 div>span 间接文本）。compute_final 传真实 styles 给
    // IFC（line 1851），存储行盒度量正确，paint use_stored 渲染解 Phase A font_size（font-051 实证）。
    // **排除混合 inline+block 内容**（如 block-in-inline R109 inline-box-001、span+h4 multicol-
    // block-no-clip-001）：此类容器的存储路径与现 paint 重跑在匿名块/碎片化上分歧致回归。
    if !has_text_children && std::env::var("PHASEA_STORE_EXT").as_deref() != Ok("0") {
        use zero_css_parser::values::DisplayValue;
        let is_inline_display = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let child_ids: Vec<NodeId> = doc.child_nodes(node_id);
        let child_displays: Vec<Option<&DisplayValue>> =
            child_ids.iter().map(|c| styles.get(c).map(|s| &s.display)).collect();
        let has_inline_elem = child_displays.iter().any(|d| d.is_some_and(is_inline_display));
        let has_block_elem = child_displays
            .iter()
            .any(|d| d.is_some_and(|dd| !is_inline_display(dd)));
        // 进一步要求 inline-level 子元素为**叶文本容器**（无元素子节点）：排除 block-in-inline
        //（inline 子元素含 block 后代，如 inline-box-002 的 div2>div3，R109 碎片化存储路径无法处理）。
        let inline_children_have_elem = child_ids.iter().any(|c| {
            styles.get(c).is_some_and(|s| is_inline_display(&s.display))
                && doc
                    .child_nodes(*c)
                    .iter()
                    .any(|gc| doc.get(*gc).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        });
        has_text_children = has_inline_elem && !has_block_elem && !inline_children_have_elem;
    }
    if !has_text_children {
        return;
    }

    // 创建 IFC 并使用与 paint_text 相同的 CSS 属性配置
    use crate::inline::InlineFormattingContext;
    use crate::inline::{TextAlign, WordBreakMode};
    use crate::types::InlineLayoutFragment;
    use crate::types::InlineLayoutLine;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::property::types::{OverflowWrapValue, WhiteSpaceValue, WordBreakValue};

    let container_width = root.content_width;

    // 解析 CSS 属性（与 paint_text 相同的配置）
    let break_word = matches!(
        style.overflow_wrap,
        OverflowWrapValue::BreakWord | OverflowWrapValue::Anywhere
    );
    let (no_wrap, preserve_whitespace) = match &style.white_space {
        WhiteSpaceValue::Pre => (true, true),
        WhiteSpaceValue::PreWrap => (false, true),
        WhiteSpaceValue::PreLine => (false, false),
        WhiteSpaceValue::Nowrap => (true, false),
        _ => (false, false),
    };
    let break_word = break_word
        || !no_wrap
            && matches!(
                style.overflow_wrap,
                OverflowWrapValue::BreakWord | OverflowWrapValue::Anywhere
            );
    let word_break_mode = match &style.word_break {
        WordBreakValue::BreakAll => WordBreakMode::BreakAll,
        WordBreakValue::KeepAll => WordBreakMode::KeepAll,
        _ => WordBreakMode::Normal,
    };
    let text_align = match &style.text_align {
        zero_style_system::TextAlignValue::Left | zero_style_system::TextAlignValue::Start => TextAlign::Left,
        zero_style_system::TextAlignValue::Right | zero_style_system::TextAlignValue::End => TextAlign::Right,
        zero_style_system::TextAlignValue::Center => TextAlign::Center,
        zero_style_system::TextAlignValue::Justify => TextAlign::Justify,
    };
    let text_indent_px = match &style.text_indent {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    };
    let tab_size_px = match &style.tab_size {
        zero_style_system::TabSizeValue::Number(n) => *n as f32 * 8.0,
        zero_style_system::TabSizeValue::Length(LengthValue::Px(v)) => *v as f32,
        _ => 8.0,
    };
    let is_vertical = matches!(
        root.writing_mode,
        zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
    );
    let is_vertical_rtl = matches!(root.writing_mode, zero_style_system::WritingModeValue::VerticalRl);

    // 构造与 paint-IFC 相同的 override maps。
    // 仅纳入文本节点片段：text_node_* 混入了内联元素片段（如 <img>，font_size=0、height=96），
    // 它们与文本片段共享同一父元素；直接 collect 时 last-write-wins，结果随 HashMap 迭代
    // 顺序（每进程随机）变化 → 渲染非确定性。过滤为纯文本节点后结果确定。
    let is_text = |tn: NodeId| matches!(doc.get(tn).map(|n| &n.kind), Some(NodeKind::Text(_)));
    let parent_font_sizes: HashMap<NodeId, f32> = root
        .text_node_font_sizes
        .iter()
        .filter_map(|(&text_node_id, &fs)| {
            if !is_text(text_node_id) {
                return None;
            }
            doc.parent_node(text_node_id).map(|pid| (pid, fs))
        })
        .collect();

    let parent_is_ahem: HashMap<NodeId, bool> = root
        .text_node_is_ahem
        .iter()
        .filter_map(|(&text_node_id, &is_ahem)| {
            if !is_text(text_node_id) {
                return None;
            }
            doc.parent_node(text_node_id).map(|pid| (pid, is_ahem))
        })
        .collect();

    let parent_letter_spacing: HashMap<NodeId, f32> = root
        .text_node_letter_spacing
        .iter()
        .filter_map(|(&text_node_id, &ls)| {
            if !is_text(text_node_id) {
                return None;
            }
            doc.parent_node(text_node_id).map(|pid| (pid, ls))
        })
        .collect();

    let parent_line_heights: HashMap<NodeId, f32> = root
        .text_node_line_heights
        .iter()
        .filter_map(|(&text_node_id, &lh)| {
            if !is_text(text_node_id) {
                return None;
            }
            doc.parent_node(text_node_id).map(|pid| (pid, lh))
        })
        .collect();

    // 收集容器内的浮动排除区域
    let exclusions: Vec<crate::inline::FloatExclusion> = root
        .children
        .iter()
        .filter(|c| !matches!(c.float, zero_css_parser::values::FloatValue::None))
        .filter_map(|c| {
            let rel_y = c.y;
            if rel_y < 0.0 || c.width <= 0.0 || c.height <= 0.0 {
                return None;
            }
            Some(crate::inline::FloatExclusion {
                y: rel_y + c.margin_top,
                height: c.height + c.margin_bottom,
                width: c.width + c.margin_left + c.margin_right,
                is_left: matches!(c.float, zero_css_parser::values::FloatValue::Left),
            })
        })
        .collect();

    let mut inline_ctx = InlineFormattingContext::new(container_width)
        .with_text_align(text_align)
        .with_break_word(break_word)
        .with_no_wrap(no_wrap)
        .with_preserve_whitespace(preserve_whitespace)
        .with_word_break(word_break_mode)
        .with_text_indent(text_indent_px)
        .with_tab_size(tab_size_px)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_font_size_overrides(parent_font_sizes)
        .with_is_ahem_overrides(parent_is_ahem)
        .with_letter_spacing_overrides(parent_letter_spacing)
        .with_line_height_overrides(parent_line_heights)
        .with_inline_element_metrics(root.inline_element_metrics.clone())
        .with_margin_overrides(root.inline_element_margins.clone());

    if !exclusions.is_empty() {
        inline_ctx = inline_ctx.with_float_exclusions(exclusions);
    }

    // R109 §9.2.1.1：匿名块片段只收集其片段的 inline 内容（fragment_node_ids），
    // 而非 inline 元素的全部 DOM 子节点。
    if let Some(frag) = root.fragment_node_ids.clone() {
        inline_ctx.set_fragment_node_ids(frag);
    }

    // R84：用真实样式跑 IFC。仅当结果为**单行**且容器为**纯 Ahem 字体**时存储：
    // - 单行：line-breaking 不受样式影响，真实样式只修正 font-size/baseline，安全。
    // - 纯 Ahem（font-family 恰好为 ["Ahem"]）：避免多字体列表（如 "Courier New, Ahem"）
    //   在真实样式下的 font 解析/fallback 差异导致回归。
    // 其余情况不存储——paint 回退到非存储路径（空样式），保持与 baseline 一致，避免回归。
    inline_ctx.layout(doc, node_id, styles);
    let is_pure_ahem = style.font_family.len() == 1 && style.font_family[0].eq_ignore_ascii_case("Ahem");
    if inline_ctx.lines.len() > 1 || !is_pure_ahem {
        return;
    }

    // 转换 IFC 结果为 InlineLayoutLine/InlineLayoutFragment
    let lines: Vec<InlineLayoutLine> = inline_ctx
        .lines
        .iter()
        .map(|line| InlineLayoutLine {
            y: line.y,
            height: line.height,
            fragments: line
                .runs
                .iter()
                .map(|frag| InlineLayoutFragment {
                    x: frag.x,
                    y: frag.y,
                    width: frag.width,
                    height: frag.height,
                    font_size: frag.font_size,
                    is_ahem: frag.is_ahem,
                    text: frag.text.clone(),
                    node_id: Some(frag.node_id),
                })
                .collect(),
        })
        .collect();

    if !lines.is_empty() {
        root.inline_layout = Some(lines);
        root.inline_layout_width = container_width;
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

fn measure_text_content(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
) -> Size<f32> {
    // 检查是否为文本节点（匿名 flex/grid item）
    // 在 flex/grid 容器中，文本节点被包装为匿名 taffy 节点参与布局。
    if let Some(node) = doc.get(dom_id)
        && let NodeKind::Text(text_data) = &node.kind
    {
        let text = text_data.content.trim().to_string();
        if text.is_empty() {
            return Size::ZERO;
        }
        // 获取父元素的 ComputedStyle 用于字体指标
        let parent_style = doc.parent_node(dom_id).and_then(|pid| styles.get(&pid));
        let (font_size, line_height) = crate::inline::resolve_font_metrics(parent_style);
        let is_ahem = parent_style
            .map(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")))
            .unwrap_or(false);

        // 包含 letter-spacing：CSS 规范中 letter-spacing 适用于每个字符
        let letter_spacing: f32 = parent_style
            .map(|s| match &s.letter_spacing {
                zero_style_system::property::types::LengthValue::Px(v) => *v as f32,
                _ => 0.0,
            })
            .unwrap_or(0.0);
        let measured_width: f32 = text
            .chars()
            .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem) + letter_spacing)
            .sum();

        return Size {
            width: known_dimensions.width.unwrap_or(measured_width),
            height: known_dimensions.height.unwrap_or(line_height),
        };
    }

    if !has_inline_content(doc, styles, dom_id) {
        // 无行内内容的叶节点（如空的 flex/grid 子元素）：
        // 尺寸来自 known_dimensions（taffy 已知的尺寸），
        // 回退到 CSS computed style 的显式 width/height。
        // 注意：taffy flexbox 在 measure callback 中会将主轴 known_dimensions 设为 None
        // （因为主轴尺寸由 flex 布局控制），所以需要从 computed style 获取。
        let style = styles.get(&dom_id);
        let explicit_w = known_dimensions.width.or_else(|| {
            style.and_then(|s| match &s.width {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
        });
        let explicit_h = known_dimensions.height.or_else(|| {
            style.and_then(|s| match &s.height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
        });
        return Size {
            width: explicit_w.unwrap_or(0.0),
            height: explicit_h.unwrap_or(0.0),
        };
    }

    let width = known_dimensions
        .width
        .or(available_space.width.into_option())
        .unwrap_or(f32::INFINITY)
        .max(0.0);
    let is_vertical = doc
        .parent_node(dom_id)
        .and_then(|pid| styles.get(&pid))
        .is_some_and(|s| {
            matches!(
                s.writing_mode,
                WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
            )
        });
    let is_vertical_rtl = doc
        .parent_node(dom_id)
        .and_then(|pid| styles.get(&pid))
        .is_some_and(|s| matches!(s.writing_mode, WritingModeValue::VerticalRl));
    // 收集 inline-block 子元素的尺寸，供 IFC 正确计算行盒和换行。
    // resolve_inline_block_dimension 对 Percentage 值返回 0，
    // 需要用容器宽度解析百分比后提供给 IFC。
    let ib_sizes: HashMap<NodeId, (f32, f32)> = doc
        .child_nodes(dom_id)
        .iter()
        .filter_map(|&child_id| {
            let child_node = doc.get(child_id)?;
            if !matches!(&child_node.kind, NodeKind::Element(_)) {
                return None;
            }
            let style = styles.get(&child_id)?;
            if !matches!(style.display, DisplayValue::InlineBlock) {
                return None;
            }
            let w = crate::inline::resolve_inline_block_dimension(&style.width, style, true);
            let h = crate::inline::resolve_inline_block_dimension(&style.height, style, false);
            // Percentage 宽度用 container_width 解析
            let resolved_w = if w > 0.0 {
                w
            } else if let LengthValue::Percentage(pct) = &style.width {
                (*pct as f32 / 100.0) * width
            } else {
                0.0
            };
            let resolved_h = if h > 0.0 {
                h
            } else if let LengthValue::Percentage(pct) = &style.height {
                (*pct as f32 / 100.0) * width
            } else {
                0.0
            };
            if resolved_w > 0.0 || resolved_h > 0.0 {
                Some((child_id, (resolved_w, resolved_h)))
            } else {
                None
            }
        })
        .collect();
    let mut inline_ctx = InlineFormattingContext::new(width)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_inline_block_sizes(ib_sizes);
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

/// 检查容器是否包含行内级内容（文本节点或行内级元素）。
///
/// CSS 2.1 规范要求空 inline 元素仍通过 line-height + padding + border
/// 贡献到行盒高度。仅检查文本节点会遗漏仅包含空 inline 元素的容器，
/// 导致 IFC 不被调用，行盒高度计算不正确。
fn has_inline_content(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    // 快速路径：有直接文本子节点
    if has_direct_text(doc, dom_id) {
        return true;
    }

    // 检查是否有 inline-level 元素子节点
    use zero_style_system::property::types::DisplayValue;
    doc.child_nodes(dom_id).iter().any(|child_id| {
        if let Some(node) = doc.get(*child_id)
            && let NodeKind::Element(_elem_data) = &node.kind
            && let Some(style) = styles.get(child_id)
        {
            return matches!(style.display, DisplayValue::Inline | DisplayValue::InlineBlock);
        }
        false
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
            // right/bottom 百分比仅当对应尺寸为 auto 时才影响尺寸/位置；
            // 当前不处理（避免与 width/height 重叠计算引入复杂性与回归）。
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
    let content_abs_y = box_node.y + box_node.content_y;
    adjust_float_positions_with_context(box_node, content_abs_y, 0.0, 0.0);
}

/// 垂直书写模式下 width:auto 块级元素收缩到内容（CSS §10.3.3 + CSS Writing Modes §7.1）。
///
/// 规范：垂直书写模式（vertical-rl/lr）中，块级元素的 block-size 为物理 width。
/// block-size:auto 时应基于内容收缩（同水平模式下的 height:auto），而非填满包含块。
///
/// 当前架构的轴交换以**父元素**书写模式为键（converter/tree.rs 与 engine.rs
/// extract_layout）：仅当父元素为垂直模式时才交换子元素几何，使 taffy 以水平模型布局。
/// 但元素**自身**书写模式决定其 width 是 block-size 还是 inline-size。当元素自身为垂直
/// 模式而其父元素为水平模式时（典型场景：body 内一个 `writing-mode: vertical-rl` 的
/// div），轴交换不触发，taffy 把 width:auto 当作行内填充（填满容器宽度），违反规范。
///
/// 此后处理在 float 定位之后遍历布局树，对这类块按内容块轴跨度（最右侧流内子元素
/// margin-box 右缘）收缩 width，并尊重 min-width/max-width。
///
/// **自限性**：仅当内容右缘窄于当前 width 时才收缩，对子元素已正确铺满到内容右缘的
/// 垂直块为 no-op，从而对正确布局的用例零回归。
fn shrink_vertical_blocks_to_content(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_writing_mode: &WritingModeValue,
) {
    let own_vertical = matches!(
        box_node.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    );
    let parent_horizontal = matches!(parent_writing_mode, WritingModeValue::HorizontalTb);

    if own_vertical && parent_horizontal && box_node.is_block_level && !box_node.is_absolute && !box_node.is_fixed {
        let width_auto = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.width, LengthValue::Auto));
        if width_auto {
            // 内容块轴跨度 = 最右侧流内子元素 margin-box 右缘（相对父 border-box）。
            let content_extent = box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .map(|c| c.x + c.width + c.margin_right)
                .fold(0.0f32, f32::max);
            // 解析 min-width/max-width（style 解析阶段 em/rem 等已解析为 Px）。
            let (min_w, max_w) = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .map(|s| {
                    let lo = match &s.min_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => 0.0,
                    };
                    let hi = match &s.max_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => f32::MAX,
                    };
                    (lo, hi)
                })
                .unwrap_or((0.0, f32::MAX));
            let new_width = content_extent.max(min_w).min(max_w).min(box_node.width);
            if new_width + 0.5 < box_node.width {
                let frame =
                    box_node.border_left + box_node.border_right + box_node.padding_left + box_node.padding_right;
                box_node.width = new_width;
                box_node.content_width = (new_width - frame).max(0.0);
            }
        }
    }

    let pw = box_node.writing_mode.clone();
    for child in &mut box_node.children {
        shrink_vertical_blocks_to_content(child, styles, &pw);
    }
}

/// 后处理：`width:auto` 的 inline-block 收缩到内容宽度（CSS §10.3.9 shrink-to-fit）。
///
/// taffy 0.7 把 width:auto 的 inline-block 拉伸到可用宽度（如同 block），违反
/// inline-block 应 shrink-to-fit 到 max-content 的规范。此处读取流内 block 级子元素
/// 已布局的宽度（margin-box），取最大值作为内容宽度，仅在内容确实更窄时收缩
/// （内容更宽或显式宽度时为 no-op）。与 R129 float-shrink / R138 table-shrink 同谱系，
/// 但作用对象是 inline-block——其子元素已是正确尺寸（如显式 width 的 block），
/// 故仅收缩盒尺寸本身，不重排子元素。
fn shrink_inline_blocks_to_content(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    let own_horizontal = matches!(box_node.writing_mode, WritingModeValue::HorizontalTb);
    if own_horizontal && !box_node.is_absolute && !box_node.is_fixed {
        let is_inline_block = box_node.node_id.is_some_and(|id| {
            styles
                .get(&id)
                .is_some_and(|s| matches!(s.display, DisplayValue::InlineBlock))
        });
        let width_auto = box_node
            .node_id
            .is_some_and(|id| styles.get(&id).is_some_and(|s| matches!(s.width, LengthValue::Auto)));
        if is_inline_block && width_auto {
            // 内容最大宽度（max-content）：inline 级子元素水平求和（假设不换行），
            // block 级子元素垂直堆叠取最大；取两者较大值。子元素宽度已是 taffy
            // 正确布局结果（如显式 width 的 block / inline-block）。
            let mut inline_sum = 0.0f32;
            let mut block_max = 0.0f32;
            for c in &box_node.children {
                if c.is_absolute || c.is_fixed {
                    continue;
                }
                let outer_w = c.width + c.margin_left + c.margin_right;
                if c.is_block_level {
                    block_max = block_max.max(outer_w);
                } else {
                    inline_sum += outer_w.max(0.0);
                }
            }
            let content_max_w = inline_sum.max(block_max);
            if content_max_w > 0.0 {
                let frame =
                    box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
                let shrink_border_box = content_max_w + frame;
                if shrink_border_box + 0.5 < box_node.width {
                    box_node.width = shrink_border_box;
                    box_node.content_width = content_max_w;
                }
            }
        }
    }

    for child in &mut box_node.children {
        shrink_inline_blocks_to_content(child, styles);
    }
}

/// 标记孤立 table-internal 元素（CSS Tables §2.4）为匿名 table 根。
///
/// 当 `display:table-row-group/table-row/table-cell/...` 出现在非 table 上下文中
///（父元素非 table/table-internal）时，CSS 规范应为其生成匿名 table 包装盒。
/// 此预遍历近似该行为：把这类孤立元素的 `is_anon_table_root` 置真，使其在
/// `establishes_bfc` 中被视为匿名 table（建立 BFC，隔离 margin 折叠 + 包含浮动）。
/// 在 `adjust_float_positions` 之前运行，确保 float exclusion 识别这些容器。
fn mark_anonymous_table_roots(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    in_table_context: bool,
) {
    let display = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| s.display.clone());
    let is_table = matches!(display, Some(DisplayValue::Table | DisplayValue::InlineTable));
    let is_table_internal = matches!(
        display,
        Some(
            DisplayValue::TableRowGroup
                | DisplayValue::TableHeaderGroup
                | DisplayValue::TableFooterGroup
                | DisplayValue::TableRow
                | DisplayValue::TableCell
                | DisplayValue::TableCaption
        )
    );

    if is_table_internal && !in_table_context {
        box_node.is_anon_table_root = true;
        // 孤立 table-internal 充当匿名 table（块级），使其被 adjust_float_positions
        // 的 clear / BFC float-exclusion 逻辑（要求 is_block_level）正确处理。
        box_node.is_block_level = true;
    }

    let child_context = in_table_context || is_table || is_table_internal;
    for child in &mut box_node.children {
        mark_anonymous_table_roots(child, styles, child_context);
    }
}

fn adjust_float_positions_with_context(
    box_node: &mut LayoutBox,
    box_content_abs_y: f32,
    inherited_left_bottom_abs: f32,
    inherited_right_bottom_abs: f32,
) {
    use zero_css_parser::values::ClearValue;
    use zero_css_parser::values::FloatValue;

    // 容器的内容区域宽度
    let container_width = box_node.content_width;

    // CSS Flexbox §4 / Grid §4 / Tables §2.4：flex/grid/table 容器的流内子元素
    //（即布局项）其 `float` 与 `clear` 不产生浮动或清除效果——`float` 计算为 `none`。
    // taffy 内部已据此布局，但 ZeroWeb 的浮动后处理（本函数）按 `child.float` 重新
    // 定位，会把带 `float:right` 的 flex item 误推到容器右缘。此处对布局容器父级的
    // 直接子元素将 `float` 归零，使后处理（含 paint 的 float 排斥/绘制）一致忽略它。
    if box_node.is_layout_container {
        for child in &mut box_node.children {
            child.float = FloatValue::None;
        }
    }

    // taffy 子元素的 Y 坐标是相对于父元素的 border-box 原点，
    // 而 flow_bottom / line_y 等追踪变量是相对于 content area 原点。
    // 当容器有 border-top 或 padding-top 时，需要加上偏移量。
    let content_y_offset = box_node.border_top + box_node.padding_top;
    let inherited_left_bottom = (inherited_left_bottom_abs - box_content_abs_y).max(0.0);
    let inherited_right_bottom = (inherited_right_bottom_abs - box_content_abs_y).max(0.0);

    // CSS §8.3.1 修正：float 的 margin 不与父容器折叠。但 taffy 把 float 当作普通
    // block 排列，当容器的首个流内子元素是 float 且容器无 border-top/padding-top
    //（margin 可与子元素折叠）时，容器的 margin-top 会被折叠到该 float 的 margin
    //（取 max），使容器（及其全部内容）整体偏低。此处把多折叠的量从容器 y 中扣除
    // 并恢复 margin_top。
    //
    // 精确门控（四重条件，确保只修真正的 float-margin 折叠，排除 taffy 把 float 当
    // block 引起的其它膨胀）：
    //   1. 容器无 border-top/padding-top（margin 可与首个子元素折叠）
    //   2. 容器布局 margin_top > 声明值（发生了膨胀）
    //   3. 容器 margin_top == 首个 float 子元素 margin_top（容器 mt 被折叠到该 float mt）
    //   4. 该 float 子元素的 margin_top == 其声明值（float 的 mt 自身未被 taffy 膨胀）
    if content_y_offset == 0.0 && box_node.margin_top > box_node.declared_margin_top + 0.01 {
        if let Some(fc) = box_node
            .children
            .iter()
            .find(|c| !c.is_absolute && !c.is_fixed)
            .filter(|c| !matches!(c.float, FloatValue::None))
        {
            let container_absorbed_float_mt = (box_node.margin_top - fc.margin_top).abs() < 0.01;
            let float_mt_is_clean = (fc.margin_top - fc.declared_margin_top).abs() < 0.01;
            if container_absorbed_float_mt && float_mt_is_clean {
                let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                box_node.y -= over_collapse;
                box_node.margin_top = box_node.declared_margin_top;
            }
        }
    }

    // 第一阶段：重新定位 float 元素，记录每个 float 在 taffy 布局中占据的垂直空间
    //
    // CSS 2.1 §9.5.1 float 定位规则：
    // 1. Float 必须尽可能高（"as high as possible"）
    // 2. Float 的 outer top 不得高于前面元素生成的块的 outer top
    // 3. Float 不参与 margin 折叠
    //
    // 关键：min_line_y 必须基于正常流内容的实际位置（考虑 margin 折叠），
    // 而不是 taffy 的 Y（taffy 将 float 当作 block，包含了前元素的 margin 折叠）。
    let mut line_y = 0.0f32;
    let mut line_max_height = 0.0f32;
    let mut left_used_width = 0.0f32;
    let mut right_used_width = 0.0f32;
    let mut left_float_bottom = inherited_left_bottom;
    let mut right_float_bottom = inherited_right_bottom;

    // 正常流的垂直位置追踪（用于计算 float 的最小 Y）
    // 关键：flow_bottom 必须独立于 taffy 的 Y 来计算。
    // taffy 将 float 当作 block 排列，导致后续正常流元素的 Y 偏移过大。
    // 我们通过累加正常流元素的高度 + margin 折叠来独立追踪 flow_bottom。
    let mut flow_bottom = 0.0f32; // 上一个正常流元素的 border-bottom（相对于容器 content area）
    let mut last_flow_mb = 0.0f32; // 上一个正常流元素的 margin-bottom
    // 第一个流内子元素的 margin-top 会与无 border-top/padding-top 的父容器折叠
    //（CSS §8.3.1），此时子元素位于容器 content 原点，其 margin-top 不应计入 flow_bottom，
    // 否则后续 float 的 min_float_y 会把该 margin-top 双重计入，使 float 偏低。
    let mut first_in_flow = true;

    // 记录每个 float 子元素在 taffy 布局中的 Y 和高度，用于后续偏移修正
    let mut float_taffy_y: Vec<(usize, f32, f32)> = Vec::new(); // (index, taffy_y, outer_height)

    for (idx, child) in box_node.children.iter_mut().enumerate() {
        // 跳过绝对定位和 fixed 元素
        if child.is_absolute || child.is_fixed {
            continue;
        }

        // 正常流元素：独立更新 flow_bottom
        // 不使用 taffy 的 Y（包含 float 垂直空间），而是自行累加
        if matches!(child.float, FloatValue::None) && !child.is_absolute && !child.is_fixed {
            // 第一个流内子元素的 margin-top 与无 border-top/padding-top 的父容器折叠：
            // margin-top 上浮到父容器外，子元素位于 content 原点，不计入 flow_bottom。
            let parent_collapses_first = first_in_flow && content_y_offset == 0.0;
            first_in_flow = false;
            // 独立计算正常流位置：使用 margin 折叠
            if crate::margin_collapse::is_empty_block(child) {
                let collapsed_self_margin =
                    crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
            } else {
                let collapsed_margin = if parent_collapses_first {
                    0.0
                } else {
                    crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top)
                };
                let child_y = flow_bottom + collapsed_margin;
                let child_border_bottom = child_y + child.height;
                flow_bottom = child_border_bottom;
                last_flow_mb = child.margin_bottom;
            }
            // 处理非 float 元素的 clear 属性（延迟到第二阶段）
            if matches!(child.float, FloatValue::None) {
                continue;
            }
        }

        // 记录 float 元素的 taffy Y 位置和高度
        let child_outer_height = child.margin_top + child.height + child.margin_bottom;
        float_taffy_y.push((idx, child.y, child_outer_height));

        // CSS §10.3.5：width:auto 的浮动非替换元素应 shrink-to-fit 到内容宽度。
        // taffy 把 float 当作普通 block（填满可用宽度），此处对 width:auto 且有
        // 块级子元素的 float 收缩到子元素最大 border-box 宽度（仅当窄于当前宽度）。
        // 纯文本内容（无块级子元素）的 float 保持 taffy 宽度——其 shrink-to-fit 需
        // IFC 测量，留作后续。仅当内容确实更窄时才收缩，对内容更宽或显式宽度的 float 为 no-op。
        if child.declared_width_auto {
            let block_child_widths: Vec<f32> = child
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed && c.is_block_level)
                .map(|c| c.width)
                .collect();
            let content_max_w = block_child_widths.iter().copied().fold(0.0f32, f32::max);
            // 有块级子元素时收缩到内容宽度（content_max_w + padding + border）。
            // **content_max_w 可能为 0**（如 visibility:collapse 的 flex item 主尺寸归零，
            // 或空内容块）——旧条件 `content_max_w > 0.0` 在此跳过收缩致 float 撑满全宽
            //（flexbox-collapsed-item-horiz-001 根因，R300）。改为「有块级子元素即收缩」：
            // 空内容 float 收缩到 padding+border（最小盒），仍比全宽更接近 shrink-to-fit 语义。
            // 无块级子元素（纯文本 float）保持 taffy 宽度——其 shrink-to-fit 需 IFC 测量，留后续。
            if !block_child_widths.is_empty() {
                let shrink_border_box =
                    content_max_w + child.padding_left + child.padding_right + child.border_left + child.border_right;
                if shrink_border_box < child.width {
                    child.width = shrink_border_box;
                    child.content_width = content_max_w;
                }
            }
        }

        // 计算浮动元素的总占用尺寸（含 margin）
        let child_outer_width = child.margin_left + child.width + child.margin_right;

        // CSS 2.1 §9.5.1 float 定位约束：
        // 1. Float 的 outer top 不得高于前面正常流元素生成的块盒的 outer top
        // 2. Float 不参与 margin 折叠
        //
        // CSS 2.1 §9.5.1：float 的 margin box 顶边按正常流规则确定。
        // 这里的 `line_y` / `child.y` 都追踪的是 border-box 顶边，因此用于
        // 约束 float 最小垂直位置时，不能再次把当前元素的 margin-top 加进去，
        // 否则后续 `child.y = line_y + margin_top` 会双计 margin-top。
        let min_float_y = flow_bottom + last_flow_mb;
        if min_float_y > line_y {
            line_y = min_float_y;
            left_used_width = 0.0;
            right_used_width = 0.0;
            line_max_height = 0.0;
        }

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
                child.y = content_y_offset + line_y + child.margin_top;

                left_used_width += child_outer_width;
                let new_bottom = line_y + child_outer_height;
                left_float_bottom = left_float_bottom.max(new_bottom);
            }
            FloatValue::Right => {
                right_used_width += child_outer_width;
                child.x = container_width - right_used_width + child.margin_left;
                child.y = content_y_offset + line_y + child.margin_top;

                let new_bottom = line_y + child_outer_height;
                right_float_bottom = right_float_bottom.max(new_bottom);
            }
            FloatValue::InlineStart | FloatValue::InlineEnd | FloatValue::None => {}
        }

        // CSS 2.1 §9.5.1：零高度浮动元素（margin-box 高度为 0）不推进 line_y。
        // 一个没有内容、没有 border、没有 padding 的空浮动元素不应占据垂直空间，
        // 后续浮动元素应从相同的 line_y 开始。
        if child_outer_height > 0.0 {
            line_max_height = line_max_height.max(child_outer_height);
        }
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
    let has_active_float_context =
        !float_taffy_y.is_empty() || inherited_left_bottom > 0.0 || inherited_right_bottom > 0.0;
    let mut child_float_contexts: Vec<(f32, f32)> =
        vec![(inherited_left_bottom, inherited_right_bottom); box_node.children.len()];

    if has_active_float_context {
        let mut float_y_offset = 0.0f32;
        // 追踪正常流内容的位置，用于 clearance 假设位置计算
        let mut flow_bottom = 0.0f32; // 上一个非 float 流内元素的 border-bottom
        let mut last_flow_mb = 0.0f32; // 上一个非 float 流内元素的 margin-bottom
        let mut active_left_float_bottom = inherited_left_bottom;
        let mut active_right_float_bottom = inherited_right_bottom;

        // 收集浮动元素的几何信息，用于 BFC 排斥计算
        // 使用实际坐标（Phase 1 已完成定位），避免重复计算
        // 收集浮动元素的几何信息，用于 BFC 排斥计算
        // 使用实际坐标（Phase 1 已完成定位），避免重复计算
        // 注意：c.y 已包含 margin_top（Phase 1 定位：line_y + margin_top），
        // 因此 float_h 只需 height + margin_bottom，避免 margin_top 双重计数。
        let float_geometries: Vec<(FloatValue, f32, f32, f32, f32, f32)> = box_node
            .children
            .iter()
            .filter(|c| !matches!(c.float, FloatValue::None))
            .map(|c| {
                (
                    c.float.clone(),
                    c.x,                        // 边框盒左边（已含 margin_left 偏移）
                    c.y,                        // 边框盒顶部（已含 margin_top 偏移）
                    c.width,                    // 边框盒宽度（不含 margin）
                    c.height + c.margin_bottom, // 从边框盒顶部到 margin-box 底部
                    c.margin_right,             // 右 margin（用于 BFC 排斥计算）
                )
            })
            .collect();

        for (idx, child) in box_node.children.iter_mut().enumerate() {
            child_float_contexts[idx] = (active_left_float_bottom, active_right_float_bottom);
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if !matches!(child.float, FloatValue::None) {
                // float 元素：将其 taffy 高度加入 offset
                // CSS 2.1：零高度浮动元素不占据垂直空间

                // CSS 2.1 §9.5.1：float 元素不应高于正常流内容的位置。
                // Phase 1 定位 float 时不知道 normal flow 的位置，
                // 这里修正：将 float 的 Y 推到至少与当前流位置齐平。
                // 注意：flow_bottom 是 content-relative，child.y 是 border-relative
                let child_content_y = child.y - content_y_offset;
                if child_content_y < flow_bottom {
                    let shift = flow_bottom - child_content_y;
                    child.y = content_y_offset + flow_bottom;
                    // 仅更新此 float 所在侧的 float_bottom 追踪
                    match child.float {
                        FloatValue::Left => active_left_float_bottom += shift,
                        FloatValue::Right => active_right_float_bottom += shift,
                        _ => {}
                    }
                }

                let float_total_height = child.margin_top + child.height + child.margin_bottom;
                float_y_offset += float_total_height;
                let child_bottom = child.y - content_y_offset + child.height + child.margin_bottom;
                match child.float {
                    FloatValue::Left => active_left_float_bottom = active_left_float_bottom.max(child_bottom),
                    FloatValue::Right => active_right_float_bottom = active_right_float_bottom.max(child_bottom),
                    _ => {}
                }
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
                flow_bottom = flow_bottom.max(child.y - content_y_offset + child.height);
                continue;
            }

            // CSS 2.1 §9.5.2 Clearance 计算
            // 假设位置（hypothetical position）：无 clear 时元素应在的位置，
            // 基于正常流的 flow_bottom + margin 折叠计算。
            //
            // CSS 2.1 §9.5.2 clearance 算法：
            // 1. 计算 hypothetical position（假设 clear:none，含 margin 折叠）
            // 2. 计算 clearance = max(0, clear_bottom - hypothetical_position)
            // 3. 如果 clearance > 0：元素推到 clear_bottom
            // 4. 如果 clearance == 0：clear 仍阻止 margin 折叠
            //    元素放在 flow_bottom + margin_top（不折叠）
            match child.clear {
                ClearValue::Left | ClearValue::Right | ClearValue::Both => {
                    let clear_bottom = match child.clear {
                        ClearValue::Left => active_left_float_bottom,
                        ClearValue::Right => active_right_float_bottom,
                        _ => active_left_float_bottom.max(active_right_float_bottom),
                    };
                    // 假设位置：基于正常流的 flow_bottom + margin 折叠
                    // CSS 2.1 §9.5.2：「as if 'clear' were 'none'」
                    let collapsed_margin = crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top);
                    let hypothetical_y = flow_bottom + collapsed_margin;

                    if clear_bottom > hypothetical_y {
                        // 正 clearance：margin 不折叠
                        // CSS 2.1 §9.5.2 C1/C2 双路径算法：
                        // C1（含 margin 折叠）：clearance = clear_bottom - hypothetical_y
                        // C2（不含 margin 折叠）：clearance = clear_bottom - (flow_bottom + child.margin_top)
                        // 最终位置 = max(clear_bottom, flow_bottom + child.margin_top)
                        // 当元素自身 margin-top 足够大时，即使不折叠也已在浮动之下
                        let uncollapsed_pos = flow_bottom + child.margin_top;
                        child.y = content_y_offset + clear_bottom.max(uncollapsed_pos);
                    } else if (clear_bottom - hypothetical_y).abs() < 0.001 {
                        // 零 clearance（hypothetical_y ≈ clear_bottom）：
                        // CSS 2.1 §9.5.2：clearance 引入后，位置 = hypothetical + clearance。
                        // clearance = max(clear_bottom - P, H - P)，其中 P 为不折叠边距位置。
                        // 当 H == clear_bottom 时，P + clearance = H = clear_bottom。
                        // 因此元素位置 = hypothetical_y（使用折叠边距计算的假设位置）。
                        // 零 clearance 仍阻止 margin 折叠（CSSWG resolution），
                        // 但视觉位置与假设位置相同。
                        child.y = content_y_offset + hypothetical_y;
                    } else {
                        // hypothetical_y > clear_bottom：元素已过浮动，
                        // 无需 clearance，margin 正常折叠。
                        child.y = content_y_offset + hypothetical_y;
                    }
                    float_y_offset = (original_taffy_y - child.y).max(0.0);
                }
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {
                    // 非 clear 的普通元素：使用独立的 flow_bottom 追踪计算正确位置
                    // 简单的 child.y -= float_y_offset 无法正确处理 margin 折叠，
                    // 因为 taffy 将 float 当作 block 排列，其 margin 折叠方式
                    // 与 float 不存在时的折叠方式不同。
                    if float_y_offset > 0.0 {
                        // CSS 2.1：非 clear 元素的位置 = 正常流位置（假设 float 不存在）
                        let collapsed_margin =
                            crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top);
                        let correct_y = flow_bottom + collapsed_margin;
                        child.y = content_y_offset + correct_y;
                        // 更新 float_y_offset 以反映 taffy Y 与正确 Y 的差异
                        float_y_offset = (original_taffy_y - child.y).max(0.0);
                    }
                }
            }

            // 空块自身的上下 margin 会自折叠，并继续传递给后继兄弟；
            // 它自身不应把 flow_bottom 往下推进一段“实心高度”。
            if crate::margin_collapse::is_empty_block(child) {
                let collapsed_self_margin =
                    crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
            } else {
                // 更新流内容追踪（使用 content-relative 坐标）
                flow_bottom = child.y - content_y_offset + child.height;
                last_flow_mb = child.margin_bottom;
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

                for (float_dir, float_x, float_y, float_border_w, float_h, float_margin_r) in &float_geometries {
                    let float_top = *float_y;
                    let float_bottom = *float_y + *float_h;

                    // 检查垂直重叠
                    if !(child_top < float_bottom && child_bottom > float_top) {
                        continue;
                    }
                    match float_dir {
                        FloatValue::Left => {
                            // 左浮动：将 BFC 元素推到浮动元素的 margin-box 右侧
                            // float_x 是边框盒左边，加上边框宽度和右 margin
                            let avoidance_x = float_x + float_border_w + float_margin_r;
                            if avoidance_x > child.x {
                                child.x = avoidance_x;
                                // 缩小宽度以不超出容器
                                let max_width = container_width - child.x;
                                if child.width > max_width {
                                    child.width = max_width.max(0.0);
                                }
                            }
                        }
                        FloatValue::Right if child.x + child.width > *float_x => {
                            // 右浮动：缩小 BFC 元素宽度以不重叠 float 的 margin-box
                            let new_width = float_x - child.x;
                            child.width = new_width.max(0.0);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 调整容器高度：
    // CSS 2.1 §10.6.7：建立 BFC 的容器必须包含浮动元素的外底边
    // （margin-box bottom）。非 BFC 容器中，浮动元素不贡献高度，
    // 因此需要收缩容器以去除 taffy 将 float 当作 block 时多计算的空间。
    //
    // 注意：子元素的 x/y 坐标是相对于父元素 content area 的，
    // 所以 content_bottom 也是相对于 content area 顶部的。
    // 不需要减去 content_y（那是相对于自身 border-box 的局部偏移，不含位置量）。
    if !float_taffy_y.is_empty() {
        let establishes_bfc = crate::margin_collapse::establishes_bfc(box_node);
        // 多列容器虽然建立 BFC（阻止 margin 折叠），
        // 但其内容通过列分布，不应使用 BFC 的浮动包含高度逻辑。
        // 使用非 BFC 路径按正常流内容高度收缩。
        let use_bfc_float_containment = establishes_bfc && !box_node.is_multicol;

        if use_bfc_float_containment {
            // BFC 容器：浮动元素已包含在高度中（taffy 正确计算了含 float 的 auto height）
            // 不需要收缩。但需要确保高度至少覆盖到最低浮动元素的外底边。
            let float_bottom = box_node
                .children
                .iter()
                .filter(|c| !matches!(c.float, FloatValue::None) && !c.is_absolute && !c.is_fixed)
                .fold(0.0f32, |max_y, c| {
                    let bottom = c.y + c.height + c.margin_bottom;
                    max_y.max(bottom)
                });
            if float_bottom > box_node.content_height {
                box_node.content_height = float_bottom;
                let new_total = float_bottom
                    + box_node.padding_top
                    + box_node.padding_bottom
                    + box_node.border_top
                    + box_node.border_bottom;
                if new_total > box_node.height {
                    box_node.height = new_total;
                }
            }
        } else {
            // 非 BFC 容器：浮动元素不贡献高度，收缩容器
            let content_bottom =
                box_node
                    .children
                    .iter()
                    .filter(|c| !c.is_absolute && !c.is_fixed)
                    .fold(0.0f32, |max_y, c| {
                        let bottom = c.y + c.height + c.margin_bottom;
                        max_y.max(bottom)
                    });
            let content_height = content_bottom.max(0.0);
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
    }

    // 递归处理子容器
    for (idx, child) in box_node.children.iter_mut().enumerate() {
        let (left_ctx, right_ctx) = child_float_contexts
            .get(idx)
            .copied()
            .unwrap_or((inherited_left_bottom, inherited_right_bottom));
        let child_content_abs_y = box_content_abs_y + child.y + child.content_y;
        if crate::margin_collapse::establishes_bfc(child) {
            adjust_float_positions_with_context(child, child_content_abs_y, 0.0, 0.0);
        } else {
            adjust_float_positions_with_context(
                child,
                child_content_abs_y,
                box_content_abs_y + left_ctx,
                box_content_abs_y + right_ctx,
            );
        }
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

        // 如果有排除区域且容器有行内级内容
        if !exclusions.is_empty()
            && let Some(dom_id) = box_node.node_id
            && has_inline_content(doc, styles, dom_id)
        {
            // 收集 inline-block 子元素的 LayoutBox 尺寸
            let ib_sizes: HashMap<NodeId, (f32, f32)> = box_node
                .children
                .iter()
                .filter(|c| {
                    c.node_id.is_some_and(|id| {
                        styles
                            .get(&id)
                            .is_some_and(|s| matches!(s.display, DisplayValue::InlineBlock))
                    })
                })
                .filter_map(|c| {
                    let node_id = c.node_id?;
                    Some((node_id, (c.content_width, c.content_height)))
                })
                .collect();

            // 重新运行 inline layout with float exclusions
            let container_width = box_node.content_width;
            let is_vertical = matches!(
                box_node.writing_mode,
                WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
            );
            let is_vertical_rtl = matches!(box_node.writing_mode, WritingModeValue::VerticalRl);
            let text_align = resolve_text_align(styles.get(&dom_id));
            let mut inline_ctx = InlineFormattingContext::new(container_width)
                .with_float_exclusions(exclusions)
                .with_vertical(is_vertical)
                .with_vertical_rtl(is_vertical_rtl)
                .with_text_align(text_align)
                .with_inline_block_sizes(ib_sizes);
            inline_ctx.layout(doc, dom_id, styles);

            // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
            store_font_sizes_from_ifc(&inline_ctx, box_node);
            sync_inline_child_boxes_from_ifc(box_node, &inline_ctx, styles);

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
            // 更新容器的内容高度：文本环绕 float 后可能需要更大的高度
            if content_height > box_node.content_height {
                let diff = content_height - box_node.content_height;
                box_node.content_height = content_height;
                box_node.height += diff;
            }
        }
    }

    // 递归处理子容器
    for child in &mut box_node.children {
        remeasure_text_with_float_exclusions(child, doc, styles);
    }
}

/// 为包含行内级子元素但无 float 的容器重新测量内容高度。
///
/// 当一个 block 容器只包含 inline 或 inline-block 子元素时（无文本节点），
/// taffy 将这些元素当作 block 排列，无法正确计算行盒高度。
/// 此函数检测这类容器，运行 IFC 获取正确的内容高度。
///
/// 典型场景：`<div><span style="line-height:5"></span></div>`
/// 空 span 的 line-height 应贡献到行盒高度，但 taffy 无法处理此情况。
fn remeasure_inline_only_containers(box_node: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // flex/grid 容器不走 IFC 重算——它们的子元素是 flex/grid item，
    // 尺寸由 taffy 决定，不应被 IFC 片段覆盖。
    // table 容器仅在有 table-internal 子元素时跳过（如 tbody/tr/td）；
    // 无 table-internal 子元素的 table 容器行为等价于 block，需要 IFC 重算。
    if box_node.is_layout_container {
        let is_table_without_internals = box_node.node_id.is_some_and(|id| {
            styles
                .get(&id)
                .is_some_and(|s| matches!(s.display, DisplayValue::Table | DisplayValue::InlineTable))
        }) && !box_node.children.iter().any(|c| {
            c.node_id.is_some_and(|cid| {
                styles.get(&cid).is_some_and(|s| {
                    matches!(
                        s.display,
                        DisplayValue::TableRowGroup
                            | DisplayValue::TableHeaderGroup
                            | DisplayValue::TableFooterGroup
                            | DisplayValue::TableRow
                            | DisplayValue::TableCell
                            | DisplayValue::TableColumn
                            | DisplayValue::TableColumnGroup
                            | DisplayValue::TableCaption
                    )
                })
            })
        });
        if !is_table_without_internals {
            // 仍然递归处理子容器
            for child in &mut box_node.children {
                remeasure_inline_only_containers(child, doc, styles);
            }
            return;
        }
    }

    // 检查此容器是否有 inline-level 子元素（is_block_level == false）
    // 且不包含 float 子元素（float 容器由 remeasure_text_with_float_exclusions 处理）
    let has_floats = box_node.children.iter().any(|c| !matches!(c.float, FloatValue::None));
    let has_inline_children = box_node
        .children
        .iter()
        .any(|c| !c.is_block_level && !c.is_absolute && !c.is_fixed);
    // R105：仅含直接 DOM 文本（无 inline 元素子，文本不生成独立 LayoutBox 子）且 taffy 未测量
    // （content_height≈0）的块也需要 remeasure——否则其 font_size 不会被 store_font_sizes_from_ifc
    // 存储，paint IFC 默认 16，导致大字号（100px）reftest（如 inline-formatting-context-008）渲染成 16px。
    // content_height≈0 守卫避免覆盖 taffy 已正确测量的块（font-feature/multicol-fill-auto/abspos 回归源）。
    let has_dom_text = box_node.node_id.is_some_and(|id| {
        doc.child_nodes(id)
            .iter()
            .any(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
    });
    let needs_dom_text_remeasure =
        has_dom_text && box_node.content_height < 1.0 && box_node.children.iter().all(|c| c.is_absolute || c.is_fixed);

    if !has_floats
        && (has_inline_children || needs_dom_text_remeasure)
        && let Some(dom_id) = box_node.node_id
        && let Some(style) = styles.get(&dom_id)
        && matches!(style.height, LengthValue::Auto)
    {
        let container_width = box_node.content_width;
        let is_vertical = matches!(
            box_node.writing_mode,
            WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
        );
        let is_vertical_rtl = matches!(box_node.writing_mode, WritingModeValue::VerticalRl);
        let text_align = resolve_text_align(styles.get(&dom_id));
        // 收集 inline-block 子元素的 LayoutBox 尺寸，供 IFC 解析百分比宽度。
        let ib_sizes: HashMap<NodeId, (f32, f32)> = box_node
            .children
            .iter()
            .filter(|c| {
                c.node_id.is_some_and(|id| {
                    styles
                        .get(&id)
                        .is_some_and(|s| matches!(s.display, DisplayValue::InlineBlock))
                })
            })
            .filter_map(|c| {
                let node_id = c.node_id?;
                Some((node_id, (c.content_width, c.content_height)))
            })
            .collect();
        let ib_sizes_for_mc = ib_sizes.clone();
        let mut inline_ctx = InlineFormattingContext::new(container_width)
            .with_vertical(is_vertical)
            .with_vertical_rtl(is_vertical_rtl)
            .with_text_align(text_align)
            .with_inline_block_sizes(ib_sizes);
        inline_ctx.layout(doc, dom_id, styles);

        // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
        store_font_sizes_from_ifc(&inline_ctx, box_node);
        sync_inline_child_boxes_from_ifc(box_node, &inline_ctx, styles);

        let full_height = inline_ctx.total_height();
        // balance 模式多列容器：按列宽单独测量，计算均衡分布后的高度
        // （tallest column = ceil(num_lines / col_count) 行），使容器高度匹配
        // 分配后的列内容，而非全宽 IFC 的较短高度。
        let content_height = if let Some((cw, cols)) = crate::multicol::balance_column_geometry(style, container_width)
        {
            let mut col_ctx = InlineFormattingContext::new(cw)
                .with_vertical(is_vertical)
                .with_vertical_rtl(is_vertical_rtl)
                .with_text_align(text_align)
                .with_inline_block_sizes(ib_sizes_for_mc);
            col_ctx.layout(doc, dom_id, styles);
            let total = col_ctx.total_height();
            let n = col_ctx.lines.len();
            if n > 0 && cols > 0 {
                n.div_ceil(cols) as f32 * (total / n as f32)
            } else {
                total
            }
        } else {
            full_height
        };
        if content_height > box_node.content_height {
            // 如果 IFC 计算的高度大于 taffy 的高度，更新容器高度
            let diff = content_height - box_node.content_height;
            box_node.content_height = content_height;
            box_node.height += diff;
        } else if content_height < box_node.content_height {
            // 纯 inline-level 容器且非特殊布局容器：允许减小高度。
            // taffy 将 inline 元素映射为 Block，会错误地包含 inline 元素的垂直 margin，
            // 而 CSS 2.1 规定 inline 元素的 margin-top/margin-bottom 不影响行盒高度。
            let has_block_children = box_node
                .children
                .iter()
                .any(|c| c.is_block_level && !c.is_absolute && !c.is_fixed);
            let is_layout_container = matches!(
                style.display,
                DisplayValue::Flex
                    | DisplayValue::InlineFlex
                    | DisplayValue::Grid
                    | DisplayValue::InlineGrid
                    | DisplayValue::Table
                    | DisplayValue::InlineTable
            );
            if !has_block_children && !is_layout_container {
                let diff = content_height - box_node.content_height;
                box_node.content_height = content_height;
                box_node.height += diff;
            }
        }
    }

    // 递归处理子容器，并在 inline-only 容器收缩后把后续普通流兄弟一并上移。
    let mut idx = 0usize;
    while idx < box_node.children.len() {
        let old_height = box_node.children[idx].height;
        let old_content_height = box_node.children[idx].content_height;
        remeasure_inline_only_containers(&mut box_node.children[idx], doc, styles);
        let height_delta = box_node.children[idx].height - old_height;
        let content_height_delta = box_node.children[idx].content_height - old_content_height;
        let shrink_delta = height_delta.min(content_height_delta);
        if shrink_delta < -0.01
            && matches!(box_node.children[idx].float, FloatValue::None)
            && !box_node.children[idx].is_absolute
            && !box_node.children[idx].is_fixed
            // 垂直书写模式下块流方向为水平（x 轴），「高度」是 inline 轴跨度。
            // inline 轴收缩不会在块轴留下空隙，故不应移动后续块兄弟（它们按 x 排列）。
            // 旧代码无条件 `sibling.y += shrink_delta` 会把垂直模式的兄弟推到负 y（屏幕外），
            // 例如 writing-mode:vertical-rl 根页面整页渲染为空白（box-offsets-rel-pos-vrl-004）。
            && matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)
        {
            for sibling in box_node.children.iter_mut().skip(idx + 1) {
                if sibling.is_absolute || sibling.is_fixed || !matches!(sibling.float, FloatValue::None) {
                    continue;
                }
                sibling.y += shrink_delta;
            }
        }
        idx += 1;
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

    /// `<img>` 无 width/height 属性时应使用解码固有尺寸（DC-11 替换元素固有尺寸）。
    #[test]
    fn test_img_intrinsic_size_from_decoded() {
        let html = r#"<html><body style="margin:0"><img src="logo.jpg"></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);

        // 模拟解码后的固有尺寸
        let img_id = doc
            .get_elements_by_tag_name("img")
            .into_iter()
            .next()
            .expect("img element exists");
        let mut img_sizes = HashMap::new();
        img_sizes.insert(img_id, (120.0, 90.0));

        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);

        // 在布局树中找到 img 盒，断言其尺寸 ≈ 解码固有尺寸
        let mut found = None;
        let mut stack = vec![&result.root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(img_id) {
                found = Some((b.width, b.height));
                break;
            }
            stack.extend(b.children.iter());
        }
        let (w, h) = found.expect("img box found in layout tree");
        assert!((w - 120.0).abs() < 1.0, "img width should use intrinsic 120, got {w}");
        assert!((h - 90.0).abs() < 1.0, "img height should use intrinsic 90, got {h}");
    }

    /// 回归：CSS §10.3/§10.6 替换元素——`<img>` 仅设 CSS width（height auto）时，
    /// height 应按固有宽高比从 width 推导，而非用固有绝对高度。
    /// 旧 bug：正方形 SVG（intrinsic 441×441）+ width:80px 渲染成 80×441（巨高），
    /// 致真实页面 logo（仅设 width 或 height）严重变形（wintertc logo）。
    #[test]
    fn test_img_width_set_height_auto_preserves_aspect() {
        let html = r#"<html><body style="margin:0"><img src="logo.svg" style="width:80px"></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
        let mut img_sizes = HashMap::new();
        img_sizes.insert(img_id, (441.0, 441.0)); // 正方形固有尺寸
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);
        let mut found = None;
        let mut stack = vec![&result.root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(img_id) {
                found = Some((b.width, b.height));
                break;
            }
            stack.extend(b.children.iter());
        }
        let (w, h) = found.expect("img box found");
        assert!((w - 80.0).abs() < 1.0, "img width should be 80 (CSS), got {w}");
        assert!(
            (h - 80.0).abs() < 1.5,
            "img height should be aspect-preserved ~80 (square @ width 80), got {h}"
        );
    }

    /// 对称：仅设 CSS height（width auto）时，width 按固有比例推导。
    #[test]
    fn test_img_height_set_width_auto_preserves_aspect() {
        let html = r#"<html><body style="margin:0"><img src="logo.svg" style="height:48px"></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
        let mut img_sizes = HashMap::new();
        img_sizes.insert(img_id, (200.0, 100.0)); // 2:1 宽图
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);
        let mut found = None;
        let mut stack = vec![&result.root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(img_id) {
                found = Some((b.width, b.height));
                break;
            }
            stack.extend(b.children.iter());
        }
        let (w, h) = found.expect("img box found");
        assert!((h - 48.0).abs() < 1.0, "img height should be 48 (CSS), got {h}");
        assert!(
            (w - 96.0).abs() < 1.5,
            "img width should be aspect-preserved ~96 (2:1 @ height 48), got {w}"
        );
    }

    /// width:auto 的浮动元素，块级子元素全 0 宽（如 visibility:collapse 的 flex item
    /// 主尺寸归零，或空内容块）时，应 shrink-to-fit 收缩到 padding+border，
    /// 而非撑满容器全宽。旧实现 `content_max_w > 0.0` 条件在此跳过收缩（R300/R301）。
    #[test]
    fn test_float_with_zero_width_block_child_shrinks() {
        use zero_css_parser::values::FloatValue;
        let html = r#"<html><body style="margin:0"><div style="float:left"><div style="width:0px;height:10px"></div></div></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 找到 float div（float != None，含 width:0 块级子元素）
        let mut float_w = None;
        fn walk(b: &LayoutBox, out: &mut Option<f32>) {
            if out.is_some() {
                return;
            }
            if b.float != FloatValue::None && b.children.iter().any(|c| c.is_block_level) {
                *out = Some(b.width);
                return;
            }
            for c in &b.children {
                walk(c, out);
            }
        }
        walk(&result.root, &mut float_w);
        let w = float_w.expect("should find a float with block-level children");
        assert!(
            w < 50.0,
            "width:auto float with 0-width block child should shrink (<<800), got {w} (old bug left it full-width)"
        );
    }
}

#[cfg(test)]
mod writing_mode_tests {
    use super::*;
    use zero_style_system::StyleSystem;

    /// 回归：`remeasure_inline_only_containers` 在子元素 inline 重测量使「高度」收缩时，
    /// 无条件地把后续普通流兄弟按收缩量上移 `sibling.y += shrink_delta`。
    /// 该逻辑仅适用于水平书写模式（块流方向为 y 轴）。在垂直书写模式中块流方向为 x 轴、
    /// 「高度」是 inline 轴跨度，inline 轴收缩不在块轴留空隙，不应移动块兄弟；
    /// 旧代码会把兄弟推到负 y（屏幕外），导致 writing-mode:vertical-rl 根页面整页空白
    ///（如 box-offsets-rel-pos-vrl-004）。
    #[test]
    fn test_vertical_rl_block_sibling_not_pushed_offscreen() {
        // 复刻 box-offsets-rel-pos-vrl-004 的 body 结构（含 4 个相对定位小块兄弟）。
        // 垂直书写模式下 taffy 把 <p> 的 inline 轴高度赋成接近 body 内容高度，
        // IFC 重测量后大幅收缩，旧代码会把后续块兄弟按收缩量推到负 y（整页空白）。
        let html = r#"<html style="writing-mode:vertical-rl"><body>
          <p><img src="p.png" width="304" height="35" /></p>
          <div style="width:100px;height:100px;padding:50px;border:50px solid orange;margin-right:8px;position:static"><img src="l.png" width="100" height="100" /></div>
          <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:75px;top:50px">TL</div>
          <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:275px;top:50px">TR</div>
          <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:125px;top:225px">BL</div>
          <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:325px;top:225px">BR</div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 任何普通流盒子都不应被推到视口之上（负 y）。
        fn any_offscreen_top(b: &LayoutBox) -> bool {
            if !b.is_absolute && !b.is_fixed && b.y < -0.5 {
                return true;
            }
            b.children.iter().any(any_offscreen_top)
        }
        assert!(
            !any_offscreen_top(&result.root),
            "vertical-rl page has in-flow boxes pushed off-screen (negative y)"
        );
    }

    /// 回归：CSS Flexbox §4 / Grid §4 / Tables §2.4 规定，flex/grid/table 容器的
    /// 流内子元素（布局项）其 `float` 不产生浮动效果——`float` 计算为 `none`。
    /// 旧代码的浮动后处理按 `child.float` 重新定位，把带 `float:right` 的 flex item
    /// 误推到容器右缘（css-flexbox-test1 / css-flexbox-row）。布局容器父级的直接子元素
    /// 的 `float` 应被归零，使后处理（含 paint 的 float 排斥/绘制）一致忽略它。
    #[test]
    fn test_flex_item_float_is_ignored() {
        // 复刻 css-flexbox-test1 结构：flex 容器 + 带 float:right 的子元素。
        // float:right 在非 flex 容器中会把元素推到右缘；在 flex 容器中应被忽略。
        let html = r#"<html><body style="margin:0">
          <div style="display:flex; width:600px; height:100px">
            <div id="a" style="width:100px; height:50px; float:right; background:orange">A</div>
            <div id="b" style="width:100px; height:50px; background:blue">B</div>
          </div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 定位 flex 容器（display:flex → is_layout_container=true）的两个子元素。
        fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let zero_dom::NodeKind::Element(elem) = &n.kind
                && elem.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let a = find("a", &doc, &result.root).expect("flex item #a");
        let b = find("b", &doc, &result.root).expect("flex item #b");

        // float:right 被忽略：A 不应被推到容器右缘（~600）而与 B 相邻排列在左侧。
        // 若 float 仍生效，A.x 会接近容器宽度（500+）。
        assert!(
            a.x < 200.0,
            "flex item with float:right should not be floated to right edge (a.x={})",
            a.x
        );
        // B（无 float）位置不受影响。
        assert!(b.x < 300.0, "flex item B position sane (b.x={})", b.x);
    }

    /// 回归：vertical-rl 容器内 abspos 子元素 height:auto 应 shrink-to-fit 到内容
    /// inline 跨度（CSS §10.3.7 + writing-modes §7.1），而非填满 CB cross-axis。
    ///
    /// 复刻 abs-pos-non-replaced-vrl-006 结构。taffy 把 abspos auto height 当
    /// cross-axis stretch（给 320=CB 高），fix_vertical_mode_abs_pos 应收缩到
    /// 内容（单 80px 字形的 inline 跨度）。
    #[test]
    fn test_abspos_vertical_rl_height_auto_shrink_to_fit() {
        let html = r#"<html style="writing-mode:vertical-rl"><body>
          <div id="cb" style="background:red; direction:ltr; font:80px/1 monospace; height:320px; width:320px; position:relative">12<span id="s" style="position:absolute; top:auto; bottom:auto; height:auto; color:green">X</span></div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let zero_dom::NodeKind::Element(elem) = &n.kind
                && elem.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let s = find("s", &doc, &result.root).expect("span #s");
        assert!(s.is_absolute, "span should be absolute");
        // 旧 bug：height=320（填满 CB cross-axis）。修复后应 shrink 到内容（80px 字形）。
        assert!(
            s.height < 200.0,
            "abspos height:auto in vertical-rl should shrink-to-fit to content, not fill CB (got h={})",
            s.height
        );
        assert!(
            s.height >= 79.0 && s.height <= 81.0,
            "expected content height ~80px (single 80px glyph), got h={}",
            s.height
        );
    }

    /// 回归：CSS §10.3.3 — 根元素 margin-left/right 均为 auto 且边框盒宽度小于视口时，
    /// 应水平居中。taffy 对嵌套 block 正确处理 auto margin 居中，但对根节点不应用
    ///（根无父级提供居中上下文）。验证 compute() 的根居中补丁（html-display-table-ref 用例）。
    #[test]
    fn test_root_block_margin_auto_centers() {
        // 根 html 显式窄宽度 + margin:auto → 应居中于 800px 视口（同 html-display-table-ref）。
        let html = r#"<html style="width:280px;margin:auto"><body style="margin:0">
          <div id="t" style="width:280px;height:300px;background:yellow"></div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        // html 根边框盒宽度 = 280，居中偏移 = (800-280)/2 = 260。
        let html_box = &result.root;
        assert!(
            (html_box.x - 260.0).abs() < 1.0,
            "root block with width<viewport + margin:auto should be centered (x≈260, got {})",
            html_box.x
        );
    }

    /// 回归：CSS §17.5.2 — display:table 容器 width:auto 收缩到内容后，margin:auto
    /// 应居中。验证 shrink_table_to_block_content 的居中补丁（html-display-table 用例）。
    #[test]
    fn test_display_table_margin_auto_centers() {
        // html display:table 内含窄内容（200+80 inline-block），margin:auto → 收缩并居中。
        let html = r#"<html style="display:table;margin:auto;border:10px solid green"><body style="margin:0">
          <div style="width:200px;height:50px;display:inline-block"></div><div style="width:80px;height:50px;display:inline-block"></div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        let html_box = &result.root;
        // 内容 ≈ 280 + 20 border = 300；居中偏移 = (800-300)/2 = 250。
        assert!(
            html_box.x > 200.0,
            "display:table with margin:auto should be centered (x>200, got {}) — shrink-to-fit centering missing",
            html_box.x
        );
        assert!(
            html_box.width < 400.0,
            "display:table should shrink to content (width<400, got {}) — shrink missing",
            html_box.width
        );
    }
}

#[cfg(test)]
mod anonymous_flex_item_tests {
    use super::*;
    use zero_css_parser::values::DisplayValue;
    use zero_style_system::StyleSystem;

    /// 测试 flex 容器中的文本节点生成匿名 flex item。
    /// CSS Flexbox §4：flex 容器中每个连续文本运行应生成匿名 flex item。
    #[test]
    fn test_anonymous_flex_item_created() {
        let html = r#"<html><body style="margin:0"><div style="display:flex">text node</div></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 找到 flex 容器
        let found_flex = false;
        let mut found_anonymous_text = false;
        let mut stack = vec![&result.root];
        while let Some(box_node) = stack.pop() {
            // 检查是否为匿名文本项
            if box_node.is_anonymous_text_item {
                found_anonymous_text = true;
                // 匿名文本项应有非零尺寸
                assert!(box_node.width > 0.0, "anonymous flex item should have width > 0");
                assert!(box_node.height > 0.0, "anonymous flex item should have height > 0");
                // node_id 应指向文本节点
                if let Some(nid) = box_node.node_id {
                    if let Some(n) = doc.get(nid) {
                        assert!(
                            matches!(&n.kind, zero_dom::NodeKind::Text(_)),
                            "anonymous item node_id should point to a text node"
                        );
                    }
                }
            }
            stack.extend(&box_node.children);
        }

        assert!(
            found_anonymous_text,
            "should find at least one anonymous text item in flex container"
        );
        let _ = found_flex;
    }

    /// 测试多个文本节点和元素混合在 flex 容器中。
    /// "a a" <div>x x</div> "b b" 应生成 3 个 flex items（2 个匿名 + 1 个元素）。
    #[test]
    fn test_mixed_text_and_element_flex_items() {
        let html = r#"<html><body style="margin:0"><div style="display:flex">a a<div>x x</div>b b</div></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 找到 flex 容器（display:flex 的 div）
        let mut flex_container: Option<&crate::types::LayoutBox> = None;
        let mut stack = vec![&result.root];
        while let Some(box_node) = stack.pop() {
            if let Some(nid) = box_node.node_id {
                if let Some(style) = styles.get(&nid) {
                    if matches!(style.display, DisplayValue::Flex | DisplayValue::InlineFlex) {
                        flex_container = Some(box_node);
                        break;
                    }
                }
            }
            stack.extend(&box_node.children);
        }

        let container = flex_container.expect("should find flex container");
        // 应有 3 个子项：2 个匿名文本 + 1 个 div 元素
        assert_eq!(
            container.children.len(),
            3,
            "flex container should have 3 children (2 anonymous text + 1 element)"
        );

        let anonymous_count = container.children.iter().filter(|c| c.is_anonymous_text_item).count();
        assert_eq!(anonymous_count, 2, "should have 2 anonymous text items");

        let element_count = container.children.iter().filter(|c| !c.is_anonymous_text_item).count();
        assert_eq!(element_count, 1, "should have 1 element child");
    }

    /// 测试非 flex 容器中的文本节点不会生成匿名项。
    #[test]
    fn test_no_anonymous_items_in_block_container() {
        let html = r#"<html><body style="margin:0"><div>text node</div></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 确保整个布局树中没有匿名文本项
        let mut stack = vec![&result.root];
        while let Some(box_node) = stack.pop() {
            assert!(
                !box_node.is_anonymous_text_item,
                "block container should not create anonymous text items"
            );
            stack.extend(&box_node.children);
        }
    }

    /// 回归：CSS §10.3.9 — width:auto 的 inline-block 应 shrink-to-fit 到内容最大宽度。
    /// 旧 bug：taffy 把 width:auto 的 inline-block 拉伸到可用宽度（如同 block），
    /// 含显式宽度 block 子元素的 inline-block 被错误填满 784px 而非收缩到子元素宽度。
    /// 验证 shrink_inline_blocks_to_content 后处理（baseline-block-with-overflow-001 用例）。
    #[test]
    fn test_inline_block_width_auto_shrink_to_fit() {
        // inline-block `.outer`（width:auto）含一个显式 width:30px 的 block 子元素，
        // 应收缩到 ~30px，而非填满 800px 视口。
        let html = r#"<html><body style="margin:0">
          <div class="outer" id="o" style="display:inline-block;background:orange;padding:4px">
            <div class="inner" id="i" style="width:30px;height:30px;background:blue"></div>
          </div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let zero_dom::NodeKind::Element(elem) = &n.kind
                && elem.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let o = find("o", &doc, &result.root).expect("inline-block #o");
        // 收缩后 border-box = 30(内容) + 4+4(左右 padding) = 38，远小于 784。
        assert!(
            o.width < 100.0,
            "width:auto inline-block should shrink-to-fit to content (~38px), not fill available (got w={})",
            o.width
        );
        assert!(
            (o.width - 38.0).abs() < 1.0,
            "expected inline-block width ~38px (30 content + 8 padding), got w={}",
            o.width
        );
    }
}

#[cfg(test)]
mod intrinsic_two_pass_tests {
    use super::*;
    use zero_style_system::StyleSystem;

    /// 辅助：按 id 在布局树中查找 LayoutBox。
    fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(elem) = &n.kind
            && elem.get_attribute("id").as_deref() == Some(id)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, doc, c))
    }

    /// 回归：CSS intrinsic sizing — `width:max-content` 的 grid 容器应收缩到其
    /// max-content 宽度（2 item × (50 content + 40 padding) = 180），而非塌缩为 ~0
    /// （converter MaxContent→length(0)）或填满视口（旧 Auto→fill）。
    /// 验证两趟固有宽度布局（apply_intrinsic_content_sizing）把 grid 提升到 intrinsic。
    #[test]
    fn test_grid_width_max_content_sized_to_intrinsic() {
        // 复刻 child-border-box-and-max-content-001 结构：
        // grid(width:max-content, grid-auto-columns:1fr, column flow) > 2 item >
        // .content(width:50px)。grid intrinsic = 2×(50+40 padding) + 2 border ≈ 182。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-auto-columns:1fr;grid-auto-flow:column;border:1px solid red;width:max-content">
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
          </div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        let g = find("g", &doc, &result.root).expect("grid #g");
        // 不应塌缩（~2px）也不应填满（~784），应在 ~180px。
        assert!(
            g.width > 100.0,
            "width:max-content grid should be sized to intrinsic (~182px), not collapsed (got w={})",
            g.width
        );
        assert!(
            g.width < 400.0,
            "width:max-content grid should shrink-to-fit (~182px), not fill viewport (got w={})",
            g.width
        );
        assert!(
            (g.width - 182.0).abs() < 5.0,
            "expected grid width ~182px (2×(50+40)+border), got w={}",
            g.width
        );
    }

    /// 回归：intrinsic 不可测的 max-content 容器（纯文本 item）保持塌缩，
    /// 不应被填满（验证不可测回退不会引入旧 Auto→fill 的 net -5 回归）。
    #[test]
    fn test_unmeasurable_max_content_does_not_fill() {
        // 纯文本 flex item 无显式宽度 → intrinsic 测量返回 None（Round C IFC 文本测量未就绪）
        // → 容器应保持塌缩（length(0)），而非填满视口。
        let html = r#"<html><body style="margin:0">
          <div id="f" style="display:flex;width:max-content"><div>text</div></div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        let f = find("f", &doc, &result.root).expect("flex #f");
        // 不应填满视口（<700），证明不可测容器未被 auto-fill。
        assert!(
            f.width < 700.0,
            "unmeasurable width:max-content flex must not fill viewport (got w={}); \
             would regress 5 cases like R181c",
            f.width
        );
    }

    /// R324：position:fixed 须视口相对，即使位于有偏移的 positioned 祖先内。
    ///
    /// taffy 0.7 把 fixed 当 absolute 处理（containing block = 最近 positioned 祖先），
    /// 故 fixed 的 left/top 被解析为相对该祖先。`adjust_fixed_to_viewport` 须从累积
    /// 祖先偏移中**扣除**（而非旧实现的「加上」），使其最终绝对坐标 = (left, top) 视口相对。
    /// 旧「加上」实现仅在 parent_offset==0 时碰巧正确，对有 margin-offset 的 relative
    /// 祖先内的 fixed 会 over-correct。本测试构造该场景，断言 fixed 视口相对、absolute
    /// 兄弟仍祖先相对。
    #[test]
    fn test_fixed_is_viewport_relative_inside_offset_positioned_ancestor() {
        let html = r#"<html><body style="margin:0">
      <div style="position:relative; margin-top:100px; margin-left:50px; width:400px; height:300px">
        <div style="position:absolute; top:20px; left:20px; width:50px; height:50px"></div>
        <div style="position:fixed; top:20px; left:20px; width:50px; height:50px"></div>
      </div>
    </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 与 painter 一致的绝对坐标累积：abs = parent_offset + box.x/y，
        // 子元素 offset = abs + padding + border（本测试盒均无 padding/border，故相等）。
        fn collect(b: &crate::types::LayoutBox, ox: f32, oy: f32, out: &mut Vec<(bool, bool, f32, f32)>) {
            let ax = ox + b.x;
            let ay = oy + b.y;
            out.push((b.is_fixed, b.is_absolute, ax, ay));
            let child_ox = ax + b.padding_left + b.border_left;
            let child_oy = ay + b.padding_top + b.border_top;
            for c in &b.children {
                collect(c, child_ox, child_oy, out);
            }
        }
        let mut positions = Vec::new();
        collect(&result.root, 0.0, 0.0, &mut positions);

        let fixed = positions
            .iter()
            .find(|(f, _, _, _)| *f)
            .expect("should have a position:fixed box");
        let absolute = positions
            .iter()
            .find(|(f, a, _, _)| !*f && *a)
            .expect("should have a position:absolute box");

        // R324：fixed 视口相对 = (left 20, top 20)，不受 relative 祖先 margin(50,100) 影响
        assert!(
            (fixed.2 - 20.0).abs() < 1.0,
            "fixed x should be viewport-relative ~20, got {}",
            fixed.2
        );
        assert!(
            (fixed.3 - 20.0).abs() < 1.0,
            "fixed y should be viewport-relative ~20, got {}",
            fixed.3
        );
        // absolute 仍祖先相对 = 祖先(50,100) + (20,20) = (70,120)
        assert!(
            (absolute.2 - 70.0).abs() < 1.0,
            "absolute x should be ancestor-relative ~70, got {}",
            absolute.2
        );
        assert!(
            (absolute.3 - 120.0).abs() < 1.0,
            "absolute y should be ancestor-relative ~120, got {}",
            absolute.3
        );
    }
}

#[cfg(test)]
mod tests;
