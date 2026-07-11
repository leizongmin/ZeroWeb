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

use zero_css_parser::values::{ClearValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue, PositionValue};

use zero_dom::{Document, NodeId, NodeKind};

use zero_style_system::{ComputedStyle, ZIndexValue};

use crate::dirty::LayoutDirtyTracker;

use crate::tree::{R109Wiring, build_layout_tree_with_r109};

use crate::types::{LayoutBox, LayoutResult, OverflowClip};

use zero_style_system::WritingModeValue;

// R342：float 定位/收缩与 IFC 终化逻辑抽到独立模块（2000 行规则 + Phase A 准备）。
// 通过 glob 引入保持 engine.rs 内调用点不变。
use crate::float_positioning::*;
use crate::inline_finalization::*;

// R831：abspos 后处理（adjust_fixed/absolute_to_* / stretch_fixed / resolve_abspos_against_root_cb）
// 抽出到 engine/abspos.rs（2000 行规则）。5 个自包含函数，零私有 helper 依赖；
// 经 glob 引入保持 engine.rs 内 18 处调用点不变（纯移动，零行为变化）。
mod abspos;
use abspos::*;
// R965：taffy 后处理步骤（adjust_inline_block_positions / sort_children_by_css_order /
// fix_vertical_mode_abs_pos / apply_relative_offsets* / apply_calc_size_adjustments /
// exclude_floats_from_non_bfc_auto_height / backfill_r109_anon_block_heights /
// prevent_collapse_through_min_height / clamp_percentage_max_height / resolve_relative_inset /
// apply_block_relative_percent_insets / convert_overflow_to_clip 等 15 个函数）抽出到
// engine/postprocess.rs（2000 行规则）。纯移动，零行为变化；经 glob 引入保持调用点不变。
mod postprocess;
use postprocess::*;

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
        self.compute_with_img_sizes(doc, styles, HashMap::new(), HashMap::new())
    }

    /// 与 `compute` 相同，但额外注入 `<img>` 的解码固有尺寸（按 DOM NodeId 索引），
    /// 供无 width/height 属性的替换元素回退到固有尺寸（DC-11）。`img_intrinsic_ratios`
    /// 为仅含宽高比、无确定固有尺寸的 SVG 信号（CSS §10.3.2），布局仅设 aspect_ratio。
    pub fn compute_with_img_sizes(
        &mut self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
        img_intrinsic_ratios: HashMap<NodeId, f32>,
    ) -> LayoutResult {
        // R695 复用副本：build_layout_tree_with_r109 按值取走 img_intrinsic_sizes，
        // 此处保留一份供 apply_indefinite_percent_height_to_auto 为替换元素补设固有尺寸。
        let intrinsic_for_r695 = img_intrinsic_sizes.clone();
        // 1. 构建 taffy 树（含 R109 接线产物，仅 R109_WIRE=1 时非空）
        let (mut taffy_tree, root_id, taffy_to_dom, r109) = build_layout_tree_with_r109(
            doc,
            styles,
            self.viewport_width,
            self.viewport_height,
            img_intrinsic_sizes,
            img_intrinsic_ratios,
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
        // 3.2 R695（CSS §10.5）：百分比 height 在不明确包含块上 compute-to-auto。
        //     taffy 0.7 对此回退到 CB 宽度解析（非规范）。此 pass 改写 taffy style，
        //     与 3.1 共用一次重算（两者都 set_style + mark_dirty）。
        let changed_r695 = Self::apply_indefinite_percent_height_to_auto(
            &mut taffy_tree,
            &root_box,
            &dom_to_taffy,
            styles,
            &intrinsic_for_r695,
            self.viewport_height,
        );
        let changed_pct_padding =
            Self::resolve_percentage_padding(&mut taffy_tree, &root_box, &dom_to_taffy, styles, self.viewport_width);
        // R717：aspect-ratio flex item（ratio-only SVG `<img>` 或 CSS aspect-ratio 的 leaf 块）
        // 在 flex 容器内——第一趟 taffy 对 leaf 项无法从 aspect_ratio + Auto-cross 推导 main
        // 尺寸（ collapses）。此处按解析出的 cross 尺寸 + ratio 推导 main（CSS §10.3.2 + Flexbox §4.5）。
        let changed_ratio_img =
            Self::apply_flex_aspect_ratio_item_size(&mut taffy_tree, &root_box, &dom_to_taffy, styles);
        // R1018：四趟后处理 pass 共用同一 first-pass root_box，各自独立 set taffy style。
        // 原先 `||` 短路求值会在前三趟任一 fire 时跳过 apply_intrinsic_content_sizing，
        // 致 flex 容器 shrink-to-fit / block max-content 在含 aspect-ratio/百分比 padding/不明确
        // 百分比 height 的页面失效。改为先求值再合并，确保四趟都执行。
        let changed_intrinsic =
            Self::apply_intrinsic_content_sizing(&mut taffy_tree, &root_box, &dom_to_taffy, styles, doc);
        if changed_r695 || changed_pct_padding || changed_ratio_img || changed_intrinsic {
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

        // CSS §10.1/§9.3.2：根元素 position:absolute/fixed（无 positioned 祖先）的包含块是
        // 初始包含块（视口），其 left/top Length inset 应定位根 border-box。taffy 把根节点
        // 固定在 (0,0) 且不解析根的 position:absolute（根无父级提供 abspos CB 上下文），故
        // `<html style="position:absolute;left:50px;top:50px">` 落在 (0,0) 而非 (50,50)
        // （abspos-containing-block-initial-009b/e/f + 004a-d 簇）。仅 Px；Em/Percent 保守跳过。
        if (root_box.is_absolute || root_box.is_fixed)
            && matches!(root_box.writing_mode, WritingModeValue::HorizontalTb)
        {
            use zero_css_parser::values::LengthValue;
            if let Some(style) = root_box.node_id.and_then(|id| styles.get(&id)) {
                if let LengthValue::Px(v) = &style.left {
                    root_box.x = *v as f32;
                }
                if let LengthValue::Px(v) = &style.top {
                    root_box.y = *v as f32;
                }
            }
        }

        // R711：block-level position:relative 的**百分比** top/bottom inset 被 taffy 0.7 丢弃
        //（R715 实证：Length 与 left/right % relative inset 应用，仅 top/bottom % 不应用）。
        // 此 pass 后处理补上 top/bottom % delta（Px + 水平 % 已由 taffy 处理，无 double-count）。
        apply_block_relative_percent_insets(&mut root_box, styles, self.viewport_height);

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

        // CSS §10.3.3/§10.6.3：根元素（如 <html>）的固定 margin 相对初始包含块（视口）
        // 定位 border-box。taffy 把根节点固定在 (0,0)（根无父级提供定位上下文），根的
        // 声明 margin-top/left 不被应用，致 `<html style="margin:50px">` 的边框盒落在
        // 视口原点而非 (50,50)（abspos-containing-block-initial-009a 簇）。此处补上根
        // 固定 margin 的位置偏移（auto 已由上方居中逻辑处理；百分比/Em 保守跳过）。
        if matches!(root_box.writing_mode, WritingModeValue::HorizontalTb) {
            use zero_css_parser::values::LengthValue;
            let root_style = root_box.node_id.and_then(|id| styles.get(&id));
            if let Some(s) = root_style {
                if let LengthValue::Px(v) = &s.margin_left {
                    root_box.x += *v as f32;
                }
                if let LengthValue::Px(v) = &s.margin_top {
                    root_box.y += *v as f32;
                }
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
        // 5a.1 纯文本 float shrink-to-fit（须在 adjust_float_positions 之前，使定位用收缩后宽度）
        shrink_pure_text_floats(&mut root_box, doc, styles);
        adjust_float_positions(&mut root_box);

        // 5.2 后处理（R699 CSS §10.5.1）：非 BFC 块级元素 height:auto 时高度只计 in-flow
        // 子元素，浮动子元素显式忽略。taffy 把 float 当 in-flow block 计入父 content
        // height，致 overflow:visible 父被 float 子撑高（应塌缩）。须在 adjust_float_positions
        // 之后（float 位置已定）自底向上重算。
        exclude_floats_from_non_bfc_auto_height(&mut root_box, styles);

        // 5.2a 后处理（R1319 §8.3.1 containment 兄弟位移）：clearance containment 已把
        // cleared 元素的 trailing collapse-through 链含入其 content_height，但 taffy 此前已
        // 按「泄漏的 mb」定位后续兄弟（偏低）。位移后续兄弟 + 祖先缩高（delta 传播）。
        // 修复 margin-collapse-clear-012/013 的 #next-yellow 定位。
        shift_siblings_after_clearance_containment(&mut root_box);

        // 5.3 后处理（CSS §8.3.1）：min-height 溢出型块阻止末子 margin collapse-through
        // 穿透父底部。taffy 0.7 CollapsibleMarginSet 未实现此 min-height 细节，须后处理
        // 剥离穿透 margin 并上移后续兄弟（margin-collapse-min-height-001/002 簇）。
        prevent_collapse_through_min_height(&mut root_box, styles);

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
        // 11.6a R1139：root 元素**自身** abspos/fixed + 全 inset + auto 尺寸 → stretch to
        // viewport。stretch_fixed_to_viewport_size 只递归 children 不触 root 自身；root abspos
        // CB=视口（同 fixed 语义）stretch 安全。position-{absolute,fixed}-root-element-{flex,grid}
        // 4 案（html 全 inset，旧 height 塌缩到内容 ~65px ≠ 应 530px）。
        stretch_root_abspos_to_viewport(&mut root_box, self.viewport_width, self.viewport_height, styles);

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

        // 12.1 后处理（R109 §9.2.1.1 匿名块盒高度回填，env R109_BACKFILL 默认开）：
        // compute_final 存了 inline_layout 但不回填 box height；taffy 经 ctx_node（片段
        // 首个文本节点）测匿名块盒高度，多节点/多行 run 欠计 → 容器矮 + bg 露白
        // （R935 症状 b，R938 验证）。此 pass 后序回填匿名块盒 content_height（从 IFC 行盒），
        // 并把增长 delta 加回 auto-height 祖先容器（delta 法保 margin 折叠，非重算）。
        // 详见 docs/goal/rendering-compat/r109-anonymous-block-spec.md FR-001。
        if std::env::var("R109_BACKFILL").as_deref() != Ok("0") {
            backfill_r109_anon_block_heights(&mut root_box, styles);
        }

        // 12.5 后处理：修正 calc(P% ± Npx) 尺寸。
        // taffy 不支持 calc 表达式，convert 层将 calc(100% - 6px) 近似为 Percent(1.0)。
        // 此步骤根据实际百分比计算值和 px 偏移量修正最终尺寸。
        apply_calc_size_adjustments(&mut root_box, styles);

        // 12.6 后处理：百分比 max-height 收紧。
        // taffy 0.7 对 height:auto 的块盒不会按百分比 max-height 收紧最终高度
        // （convert 层已传 Percent，但 block 布局未在内容高度计算后再次 clamp）。
        // CSS §10.7：百分比 max-height 相对包含块高度解析；当包含块高度明确时收紧。
        // 此步骤自上而下传递「明确高度」，对百分比 max-height 的盒做收紧。
        // R588：根元素的百分比 height/min-height/max-height 相对 ICB（视口）解析。
        // 旧 cb=None 使根（如 <html>）的 height:100% 不解析 → html/body/p 百分比高度
        // 链断裂（min-height-percentage-003）。CSS §10：根元素百分比高度相对 ICB。
        // 仅根传视口高度作包含块；后代经 my_definite_content_height 链传播。
        clamp_percentage_max_height(&mut root_box, Some(self.viewport_height), styles);

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
            // 仅水平书写模式的 flex/grid 容器，或 R1018 block-level（width:max-content/fit-content）
            let is_flex_grid = matches!(
                s.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            );
            let is_block = matches!(s.display, DisplayValue::Block);
            if !(is_flex_grid || is_block) || !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                continue;
            }
            // R1015/R1019：扩展 gate——除 MaxContent/MinContent 外，width:Auto + float（shrink-to-fit
            // 上下文）的 flex 容器或 block 容器也触发固有宽度计算。flex container float:left shrink
            // 到 item intrinsic；block container float:left 含 flex/grid 子时 shrink 到子 intrinsic
            //（aspect-ratio-intrinsic-014：float:left block + flex 子 + aspect-ratio item）。
            // float:block 由本 gate 处理（用 block_max_content_width 测 flex 子），float shrink
            // postprocess 路径（adjust_float_positions_with_context）见此宽度后为 no-op，无双重 shrink。
            let is_max_min = matches!(s.width, LengthValue::MaxContent | LengthValue::MinContent);
            let is_auto_float = matches!(s.width, LengthValue::Auto)
                && !matches!(s.float, FloatValue::None)
                && matches!(
                    s.display,
                    DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Block
                );
            if !is_max_min && !is_auto_float {
                continue;
            }
            // R1018：block-level 仅在 width:MaxContent 或 auto-float 时触发（bare fit-content 经
            // parser 映射 MaxContent）。
            // R1304：block + MinContent 经 block_max_content_width 测（max-content 近似——固定宽/
            // 原子内容（img/固定宽子）min==max 正确；文本内容 overestimate 最宽词但远优于 0 塌缩；
            // true min-content 最宽词测量独立子问题，见 intrinsic_sizing.rs:29）。table-intrinsic-size
            // 簇（固定宽 .content 子）min==max 精确命中。kill-switch ZW_MINCONTENT_BLOCK=0 回退旧行为。
            let mincontent_block = std::env::var("ZW_MINCONTENT_BLOCK").as_deref() != Ok("0")
                && matches!(s.width, LengthValue::MinContent);
            if is_block && !matches!(s.width, LengthValue::MaxContent) && !mincontent_block && !is_auto_float {
                continue;
            }
            // R1018：block-level 用 block_max_content_width（对 flex/grid 子分发到专用 intrinsic）。
            // multicol 容器 intrinsic = columns × column-content，block_max_content_width 不解（只测
            // 单子宽）——可测时给出部分正确值（change-intrinsic-width -14pp），不可测时走下方 Auto-fallback。
            // multicol intrinsic sizing 精度（columns × content）独立 gap。
            let intrinsic: Option<f32> = if is_block {
                Some(crate::intrinsic_sizing::block_max_content_width(b, doc, styles))
            } else if matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid) {
                crate::intrinsic_sizing::grid_intrinsic_width(b, doc, styles)
            } else if matches!(
                s.flex_direction,
                FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
            ) {
                crate::intrinsic_sizing::flex_column_intrinsic_width(b, doc, styles)
            } else {
                crate::intrinsic_sizing::flex_row_intrinsic_width(b, doc, styles)
            };
            let Some(intrinsic) = intrinsic else { continue };
            // intrinsic 不可测 → 跳过。否则按上下文判定 apply 条件：
            // - MaxContent/MinContent（grow）：current 比 intrinsic 窄 → grow 到 intrinsic。
            // - Auto+float（R1015 shrink-to-fit）：current 比 intrinsic 宽 → shrink 到 intrinsic。
            // R1018：block + MaxContent（含 bare fit-content）当 intrinsic 不可测（≤1，如 multicol
            // 容器或 aspect-ratio block 子 box_content 无法度量）时，回退 Auto（fill）而非留 0 塌缩
            // ——converter 已把 MaxContent width 映射 0，gate 测不出则元素归零（intrinsic-size-005
            // multicol + aspect-ratio 子回归）。fill（父宽）比 collapse 更接近 fit-content 语义。
            if intrinsic <= 1.0 {
                if is_block
                    && (matches!(s.width, LengthValue::MaxContent) || mincontent_block)
                    && let Some(&taffy_id) = dom_to_taffy.get(&id)
                    && let Ok(mut style) = taffy_tree.style(taffy_id).cloned()
                {
                    style.size.width = taffy::style::Dimension::auto();
                    let _ = taffy_tree.set_style(taffy_id, style);
                    let _ = taffy_tree.mark_dirty(taffy_id);
                    changed = true;
                }
                continue;
            }
            let should_apply = if is_auto_float {
                b.width > intrinsic + 1.0
            } else {
                b.width < intrinsic + 1.0
            };
            if !should_apply {
                continue;
            }
            let Some(&taffy_id) = dom_to_taffy.get(&id) else {
                continue;
            };
            if let Ok(mut style) = taffy_tree.style(taffy_id).cloned() {
                style.size.width = taffy::style::Dimension::length(intrinsic);
                let _ = taffy_tree.set_style(taffy_id, style);
                let _ = taffy_tree.mark_dirty(taffy_id);
                changed = true;
            }
        }
        changed
    }

    /// R717（CSS §10.3.2 + Flexbox §4.5）：`aspect-ratio` flex item（ratio-only SVG `<img>`
    /// 或 CSS `aspect-ratio` 的 leaf 块）在 flex 容器内时，第一趟 taffy 对该 leaf 项无法
    /// 从 `aspect_ratio` + Auto-cross（容器 cross 尺寸在 computed style 中为 Auto，但实际
    /// 解析为视口/包含块尺寸）推导出 main 尺寸——item collapses 到 0。
    ///
    /// `apply_flex_transferred_min_size`（build_layout_tree 期）尝试设 transferred min，
    /// 但它读 `parent_style.width` 仅接受 `LengthValue::Px`，对 Auto 容器（007 驱动案：
    /// `<div style="display:flex;flex-direction:column">` 宽度 Auto→解析 800）提前返回。
    ///
    /// 本 pass 在**第一趟布局后**运行——此时 LayoutBox 已含解析出的 cross 尺寸（经
    /// align-stretch / 包含块解析）。对 leaf flex item（无 in-flow 子元素，故无内容决定 main）
    /// 且 main 轴 CSS 为 auto、taffy style 有 `aspect_ratio` 的项，按 cross × ratio（row）
    /// 或 cross / ratio（column）推导 main 尺寸，改写 taffy `size.main = Length(...)` 并
    /// mark_dirty，由调用方重跑 taffy。仅水平书写模式；仅当 cross>0 且 main 与推导值显著
    /// 不同时触发。leaf 限制避免误覆盖有文本/子内容决定 main 的 flex item。
    fn apply_flex_aspect_ratio_item_size(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};

        fn walk(
            b: &LayoutBox,
            parent_style: Option<&ComputedStyle>,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let my_style = b.node_id.and_then(|id| styles.get(&id));

            // leaf flex item（无 in-flow 子盒）+ 父是 flex 容器 + taffy style 有 aspect_ratio。
            if b.children.is_empty()
                && let Some(id) = b.node_id
                && let Some(ps) = parent_style
                && matches!(ps.display, DisplayValue::Flex | DisplayValue::InlineFlex)
                && let Some(item_style) = my_style
                && let Some(&tid) = dom_to_taffy.get(&id)
                && let Ok(mut st) = taffy_tree.style(tid).cloned()
                && let Some(ratio) = st.aspect_ratio
                && ratio > 0.0
            {
                let is_column = matches!(
                    ps.flex_direction,
                    FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                );
                // main 轴 CSS 须为 auto（否则 converter 已从显式 CSS 处理，不应覆盖）。
                let main_is_auto = if is_column {
                    matches!(item_style.height, LengthValue::Auto)
                } else {
                    matches!(item_style.width, LengthValue::Auto)
                };
                // R1013：非替换 leaf（div + CSS aspect-ratio）+ main 轴 definite min-size 时跳过——
                // 此约束驱动尺寸（transferred-size 由 min-size × ratio 推导 cross），cross→main
                // 反向推导会覆盖并破坏（flex-item-transferred-sizes-padding 回归 +73pp 证）。
                // 替换元素（img/SVG）保留 fixup：其 transferred-size 由固有 ratio + cross 推导正确
                //（flex-aspect-ratio-img-column-006 / row-004 需 fixup 才 <1%，min-size 不改变语义）。
                // R993 driving case（aspect-ratio-intrinsic-size-007 SVG img）+ R994 +2（CSS aspect-ratio
                // leaf 无 min-size）均不受影响。
                let main_has_definite_min = if is_column {
                    matches!(item_style.min_height, LengthValue::Px(_))
                } else {
                    matches!(item_style.min_width, LengthValue::Px(_))
                };
                if main_is_auto && (!main_has_definite_min || b.is_replaced) {
                    // column: main=height, cross=width；row: main=width, cross=height。
                    let (main_resolved, cross_resolved) = if is_column {
                        (b.height, b.width)
                    } else {
                        (b.width, b.height)
                    };
                    let expected_main = if is_column {
                        cross_resolved / ratio
                    } else {
                        cross_resolved * ratio
                    };
                    // 仅当 cross 已解析（>0）且 main 与推导值显著不同（collapsed 或不一致）时改写。
                    if cross_resolved > 0.0 && (main_resolved - expected_main).abs() > 0.5 {
                        if is_column {
                            st.size.height = taffy::style::Dimension::length(expected_main.max(0.5));
                        } else {
                            st.size.width = taffy::style::Dimension::length(expected_main.max(0.5));
                        }
                        let _ = taffy_tree.set_style(tid, st);
                        let _ = taffy_tree.mark_dirty(tid);
                        changed = true;
                    }
                }
            }

            for c in &b.children {
                changed |= walk(c, my_style, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }
        walk(root, None, taffy_tree, dom_to_taffy, styles)
    }

    /// R695（CSS §10.5）：百分比 `height` 仅当包含块高度**明确指定**时才解析，
    /// 否则 compute-to-auto。taffy 0.7 对「百分比 height + 不明确 CB」回退到 CB
    /// **宽度**解析（非规范），致 `grandparent{height:0} > parent{auto} >
    /// child{height:100%}` 链中 child/img 被拉到满宽（如 784）。
    ///
    /// 本 pass 自上而下按**样式**判定 CB 高度明确性（与 [`clamp_percentage_max_height`]
    /// 的 `my_definite_content_height` 同语义），对水平书写模式 normal-flow 块级元素
    /// 的 `height:Percentage`（CB 不明确）改写 taffy `size.height = Auto`。替换元素
    /// 同时补设固有绝对尺寸（无 HTML width/height 属性时 taffy style 不含绝对固有
    /// 尺寸，仅 aspect_ratio）。返回是否有改动；调用方据此重跑 taffy——第二趟里
    /// taffy 正确计算非替换块的内容高度 / 替换元素的固有尺寸，无需手工重算。
    ///
    /// 范围限定：跳过 abspos（由 `adjust_absolute_pct_to_viewport` 处理）；跳过
    /// flex/grid item（其 %height 有独立 stretch 语义，taffy-gated，见 R691）。常见
    /// `html,body{height:100%}` 不受影响——根 CB 为视口（明确），整条链明确。
    fn apply_indefinite_percent_height_to_auto(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
        viewport_height: f32,
    ) -> bool {
        use zero_css_parser::values::{BoxSizingValue, DisplayValue, LengthValue, PositionValue};

        fn walk(
            b: &LayoutBox,
            cb_definite: Option<f32>,
            parent_is_flex_grid: bool,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
            img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
        ) -> bool {
            // 垂直书写模式块轴为 X，高度语义不同——保守跳过整棵子树。
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let style = b.node_id.and_then(|id| styles.get(&id));

            // 本元素提供给子元素的「明确内容高度」（None = 不明确）。
            // 默认沿用父级传入的明确性（无样式节点如匿名盒透传）。
            let mut my_definite: Option<f32> = cb_definite;

            if let Some(s) = style {
                let is_abs = matches!(s.position, PositionValue::Absolute | PositionValue::Fixed);
                if !is_abs && !parent_is_flex_grid {
                    match &s.height {
                        LengthValue::Percentage(p) => match cb_definite {
                            Some(cbh) => {
                                // 明确 CB → 解析为百分比（明确），供子元素继续链。
                                my_definite = Some(*p as f32 / 100.0 * cbh);
                            }
                            None => {
                                // 不明确 CB → compute-to-auto：改写 taffy height 为 Auto。
                                if let Some(id) = b.node_id
                                    && let Some(&tid) = dom_to_taffy.get(&id)
                                    && let Ok(mut st) = taffy_tree.style(tid).cloned()
                                {
                                    st.size.height = taffy::style::Dimension::auto();
                                    // 替换元素补设固有绝对尺寸：taffy 需要绝对值才能
                                    // 在两侧 auto 时定尺寸（aspect_ratio 只够推导比例）。
                                    if b.is_replaced
                                        && let Some(&(iw, ih)) = img_intrinsic_sizes.get(&id)
                                    {
                                        let iw = iw.max(1.0);
                                        let ih = ih.max(1.0);
                                        if matches!(s.width, LengthValue::Auto) {
                                            st.size.width = taffy::style::Dimension::length(iw);
                                        }
                                        st.size.height = taffy::style::Dimension::length(ih);
                                        if st.aspect_ratio.is_none() {
                                            st.aspect_ratio = Some(iw / ih);
                                        }
                                    }
                                    let _ = taffy_tree.set_style(tid, st);
                                    let _ = taffy_tree.mark_dirty(tid);
                                    changed = true;
                                }
                                // 现为 auto（内容决定）→ 子元素 CB 不明确。
                                my_definite = None;
                            }
                        },
                        LengthValue::Px(v) => {
                            // 明确高度：按 box-sizing 折算内容高度供子元素百分比解析。
                            let pb = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
                            my_definite = Some(if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                                (*v as f32 - pb).max(0.0)
                            } else {
                                *v as f32
                            });
                        }
                        _ => {
                            // Auto / Em / Rem 等内容决定型 → 子元素 CB 不明确。
                            my_definite = None;
                        }
                    }
                }
            }

            // 子元素是否为 flex/grid item（其 %height 走独立语义，本 pass 跳过）。
            let child_parent_flex_grid = style.is_some_and(|s| {
                matches!(
                    s.display,
                    DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
                )
            });

            for child in &b.children {
                changed |= walk(
                    child,
                    my_definite,
                    child_parent_flex_grid,
                    taffy_tree,
                    dom_to_taffy,
                    styles,
                    img_intrinsic_sizes,
                );
            }
            changed
        }

        walk(
            root,
            Some(viewport_height),
            false,
            taffy_tree,
            dom_to_taffy,
            styles,
            img_intrinsic_sizes,
        )
    }

    /// CSS §8.3/§8.4：百分比 padding 相对**包含块的内容宽度**解析（与元素自身宽度无关）。
    ///
    /// taffy 0.7 的 `LengthPercentage::Percent` padding 在多数布局路径上解析为 0
    /// （实测 `#box{width:150px;padding:20%}` 在 800px 视口内 pt=0，应 160）。
    /// 本 pass 在第一趟布局（父级 content_width 已确定）后，把百分比 padding 预解析为
    /// 绝对 px，改写 taffy style 为 `Length(px)` 并 mark_dirty，由 compute() 重跑。
    ///
    /// 非循环：百分比 padding 仅依赖父级内容宽（第一趟已知），不依赖元素自身宽度，
    /// 故一次重跑即可收敛（与 R695 %height 同模式）。
    fn resolve_percentage_padding(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        viewport_width: f32,
    ) -> bool {
        use zero_css_parser::values::LengthValue;

        fn walk(
            b: &LayoutBox,
            parent_content_width: f32,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            // 垂直书写模式下块轴为 X，padding 百分比语义不同——保守跳过。
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let style = b.node_id.and_then(|id| styles.get(&id));

            // 本元素提供给子元素的「内容宽度」（百分比 padding 的解析基准）。
            // taffy 第一趟已算出 content_width（b.content_width）；匿名盒透传父级宽度。
            let my_content_width = if b.content_width > 0.0 {
                b.content_width
            } else {
                parent_content_width
            };

            if let Some(s) = style {
                let has_pct = matches!(s.padding_top, LengthValue::Percentage(_))
                    || matches!(s.padding_right, LengthValue::Percentage(_))
                    || matches!(s.padding_bottom, LengthValue::Percentage(_))
                    || matches!(s.padding_left, LengthValue::Percentage(_));
                if has_pct
                    && let Some(id) = b.node_id
                    && let Some(&tid) = dom_to_taffy.get(&id)
                    && let Ok(mut st) = taffy_tree.style(tid).cloned()
                {
                    let resolve = |v: &LengthValue| match v {
                        LengthValue::Percentage(p) => {
                            taffy::style::LengthPercentage::length((*p as f32 / 100.0 * parent_content_width).max(0.0))
                        }
                        // 其它值保持原 taffy 值（converter 已转换）；此处只覆盖百分比。
                        _ => taffy::style::LengthPercentage::length(0.0),
                    };
                    // 仅改写为百分比的边，其余保留 taffy 已转换值。
                    if let LengthValue::Percentage(_) = s.padding_top {
                        st.padding.top = resolve(&s.padding_top);
                    }
                    if let LengthValue::Percentage(_) = s.padding_right {
                        st.padding.right = resolve(&s.padding_right);
                    }
                    if let LengthValue::Percentage(_) = s.padding_bottom {
                        st.padding.bottom = resolve(&s.padding_bottom);
                    }
                    if let LengthValue::Percentage(_) = s.padding_left {
                        st.padding.left = resolve(&s.padding_left);
                    }
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                }
            }

            for child in &b.children {
                changed |= walk(child, my_content_width, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }

        walk(root, viewport_width, taffy_tree, dom_to_taffy, styles)
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
        // 现代 CSS Overflow 行为（Mozilla bug 1880550 / csswg-drafts）：table cell 的
        // `overflow: hidden/clip/auto/scroll` 与普通块盒一致产生裁剪/滚动效果。
        // 旧 CSS 2.1 §17.5「cell 即使 overflow:hidden 也增长含内容、不裁剪」已被现代规范
        // 取代（chromium/IE 现行行为 = 裁剪）。此处 table cell 与非 cell 用同一 overflow 映射。
        let overflow_x = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_x));
        let overflow_y = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_y));
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
        // R1277 ④：记录 height:auto 供 float 后处理收缩守卫（显式高度容器不被收缩）。
        let declared_height_auto =
            computed.is_some_and(|c| matches!(c.height, zero_css_parser::values::LengthValue::Auto));

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
                    // R787 实验：CSS §10.1 abspos/fixed 的 CB = 最近 positioned 祖先
                    // **padding-box**。taffy 给 abspos 的 location 是 content-box origin
                    //（border+padding），减去 padding 得 padding-box origin（border），
                    // 仍保持 abspos border-box 相对约定。viewport-CB abspos 由
                    // adjust_absolute_pct_to_viewport 覆写 x/y，此处调整被覆盖，无冲突。
                    child.x -= padding_left;
                    child.y -= padding_top;
                } else {
                    child.x -= content_x;
                    child.y -= content_y;
                }
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
            declared_height_auto,
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
            had_clearance: false,
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
            text_node_text_transform: HashMap::new(),
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

#[cfg(test)]
mod tests;
