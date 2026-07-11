//! Taffy 布局完成后的后处理步骤（post-taffy post-processing）。
//!
//! 这些函数在 [`LayoutEngine::compute`](super::LayoutEngine) 的主 taffy 计算之后运行，
//! 对生成的 [`LayoutBox`] 树做相对定位、calc 尺寸调整、百分比高度钳制、float 高度排除、
//! R109 匿名块高度回填、min-height collapse-through 阻止等修正。
//!
//! R965：从 engine.rs 抽出（2000 行规则），纯移动，零行为变化。

use std::collections::HashMap;

use zero_css_parser::values::{
    AlignmentValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue, OverflowValue, PositionValue,
};

use zero_dom::{Document, NodeId};

use zero_style_system::{ComputedStyle, WhiteSpaceValue, WritingModeValue};

use crate::types::{LayoutBox, OverflowClip};

// R965：经 glob 引入 inline_finalization 的 resolve_text_align / store_font_sizes_from_ifc
// 等函数（与 engine.rs 的 `use crate::inline_finalization::*;` 保持一致）。
use crate::inline_finalization::*;

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
pub(super) fn adjust_inline_block_positions(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
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
            // R1147：除 Auto/Pct 外，empty InlineBlock（content_height≈0 但 border 撑出视觉高度）
            // 也须入 ib_sizes——height:0 显式 + border（border-*-width-072/073）的 content_height=0，
            // IFC 会降级零宽。下方 ib_h 逻辑给 border-box height。
            let is_inline_block = matches!(style.display, DisplayValue::InlineBlock);
            let empty_with_visual_h = is_inline_block && child.content_height.abs() < 1.0 && child.height.abs() >= 1.0;
            let needs_fallback = matches!(style.width, LengthValue::Auto | LengthValue::Percentage(_))
                || matches!(style.height, LengthValue::Auto | LengthValue::Percentage(_))
                || empty_with_visual_h;
            if !needs_fallback {
                return None;
            }
            // R1147：empty inline-block（content_height≈0，如 border-top-width 撑高但无内容）
            // 会被 IFC 降级为零宽 TextRun → 后续 inline 重叠（border-{top,bottom}-width-061/062/
            // 072/073 簇）。仅 InlineBlock + 空时用 border-box height（含 border）；InlineTable 有
            // 独立 table 布局尺寸，用 border-box 反回归（border-*-width-applies-to-014，A/B 实测）。
            let ib_h = if is_inline_block && child.content_height.abs() < 1.0 {
                child.height
            } else {
                child.content_height
            };
            Some((node_id, (child.content_width, ib_h)))
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

            // 回退：从子元素布局位置近似，或按 CSS Writing Modes §4.4 合成基线。
            //
            // 空 inline-flex/inline-grid 容器（无子元素）无可用基线，按 §4.4 合成
            // alphabetic 基线 = 容器 margin-box 下沿（border-box 高 + margin-bottom）。
            // 此前此处 return None 会让 IFC 回退到 central（h/2），违反 §4.4——htb 行盒内
            // 须合成 alphabetic 非 central（见 grid-container-baseline-synthesized-001）。
            if child.children.is_empty() {
                let baseline = child.height + child.margin_bottom;
                if baseline > 0.0 {
                    return Some((node_id, baseline));
                }
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
                    // CSS Writing Modes §4.4：首 item 无基线（空元素，无 DOM 子节点）时，
                    // 合成 alphabetic 基线 = 该 item 的 margin-box 下沿（border-box 高 +
                    // margin-bottom）；有内容的 item 保留既有 content-box 底边启发式。
                    let item_empty = c.node_id.is_some_and(|id| doc.first_child(id).is_none());
                    first_item_bottom = if item_empty {
                        c.y + c.height + c.margin_bottom
                    } else {
                        c.y + c.content_height
                    };
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

            // 合成基线可落在 content-box 之外（border-box 下沿或更低），上限用 margin-box
            // 下沿而非 content_height（否则 §4.4 合成的 margin-box 下沿基线会被误拒）。
            if baseline > 0.0 && baseline <= child.height + child.margin_bottom + 1.0 {
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
    store_font_sizes_from_ifc(&inline_ctx, root, doc, styles);

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
pub(super) fn sort_children_by_css_order(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
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
pub(super) fn fix_vertical_mode_abs_pos(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
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
    store_font_sizes_from_ifc(&inline_ctx, root, doc, styles);

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
pub(super) fn apply_relative_offsets_inline(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
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
pub(super) fn apply_calc_size_adjustments(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
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

/// R699（CSS §10.5.1）：非 BFC 块级元素 `height:auto` 且 `overflow` 计算为 `visible`
/// 时，高度只计入 **in-flow** 子元素的 border-box，浮动子元素与绝对定位子元素被
/// **显式忽略**。taffy 把 float 当 in-flow block 计入父 content height，致
/// `#div1{height:auto;overflow:visible} > div{float;left;height:1in}` 的父被撑到 96px
/// （应 0；float 溢出但本例 float 无背景故不可见 → 应「无红」）。
///
/// 自底向上（后序）重算：先递归子元素（子高度修正后再算父，级联自然传播）。仅对
/// `style.height == Auto` 且非 BFC（[`establishes_bfc`]）的块级元素生效——BFC 父
/// （overflow!=visible / flow-root / flex / grid / table 等）**包含**浮动，其高度不受
/// 此规则影响。重算值 = in-flow 子元素（非 float、非 abspos）border-box 底边相对
/// 父内容盒顶的最大值（无 in-flow 子元素 → 0）。
pub(super) fn exclude_floats_from_non_bfc_auto_height(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // 后序：先修正子元素，父级重算时读到的是已修正的子高度。
    for child in &mut box_node.children {
        exclude_floats_from_non_bfc_auto_height(child, styles);
    }
    // 仅 height:auto（内容决定型）非 BFC 块级元素。
    let is_auto_height = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.height, LengthValue::Auto));
    if !is_auto_height || crate::margin_collapse::establishes_bfc(box_node) {
        return;
    }
    // CSS §10.5.1：取 in-flow 子元素 border-box 底边最大值（相对父内容盒顶）。
    // child.y 为子元素 border-box 顶相对父内容盒顶（taffy 已含 margin 折叠后的偏移），
    // 不含子元素 margin-bottom（末子 margin-bottom 与父折叠/悬挂，不计入高度）。
    // ★ 关键守卫：仅当存在 float 子元素时才重算——无 float 时 taffy 的 content_height
    // 已正确（max(child.y+child.height) 公式对负 margin / margin 折叠不精确，无 float 时
    // 强行覆写会误收缩非 float 用例，如 root-box-001 的 p{margin:-1em}）。
    let mut extent: f32 = 0.0;
    let mut has_in_flow = false;
    let mut has_float_child = false;
    for child in &box_node.children {
        let is_float = !matches!(child.float, FloatValue::None);
        let is_abspos = child.is_absolute || child.is_fixed;
        if is_float {
            has_float_child = true;
            continue;
        }
        if is_abspos {
            continue;
        }
        has_in_flow = true;
        extent = extent.max(child.y + child.height);
    }
    if !has_float_child {
        return;
    }
    // 无 in-flow 子元素 → 0；否则 in-flow border-box 底边最大值。
    let new_content_h = if has_in_flow { extent.max(0.0) } else { 0.0 };
    let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
    // 仅当当前内容高确实被 float 撑高时收紧。
    if new_content_h + 0.5 < box_node.content_height {
        box_node.content_height = new_content_h;
        box_node.height = new_content_h + pb;
    }
}

/// R109 §9.2.1.1 匿名块盒高度回填（spec FR-001，env R109_BACKFILL 默认开）。
///
/// compute_final_inline_layouts 存了 inline_layout 但不回填 box height；taffy 经
/// `new_leaf_with_context(style, ctx_node)`（ctx_node = 片段首个文本节点）测匿名块盒，
/// 多节点/多行 inline run 被欠计 → 容器排除部分 inline 高度 → 容器矮 + bg 露白
/// （R935 像素 forensics 症状 b，R938 读码验证）。
///
/// 本 pass 后序遍历：① 匿名块盒（fragment_node_ids.is_some）从其 inline_layout 行盒
/// 回填 content_height（取 max(line.y + line.height)），仅增大不收缩（taffy 欠计场景才补）；
/// ② auto-height 祖先容器按「直系匿名块子增长 delta 之和」扩展自身高度。
///
/// 用 delta 累加而非重算（区别 exclude_floats_from_non_bfc_auto_height 的 max(child.y+h)
/// 重算）：保留 taffy 已算的 margin 折叠/兄弟定位，仅把匿名块盒欠计的高度补回并向上传播。
/// 局限：假设增长的匿名块子是末位 in-flow 子（case b 常见：[block, anon(inline run)]），
/// 非末位 anon 的 delta 仍扩展容器底（bg 修对）但不移后续兄弟（独立子问题，spec TBD）。
///
/// 返回本 box 的高度增长量（供父盒累加）。
pub(super) fn backfill_r109_anon_block_heights(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    use zero_css_parser::values::{FloatValue, LengthValue};
    // 后序 + 兄弟位移：先递归子盒，当一个 in-flow 子盒增长（高度回填），其后续 in-flow 兄弟
    // 必须下移同样像素（post-taffy 改高度不会自动重定位兄弟，R940 实证 container#2 仍位于
    // 旧 y 致重叠）。child.y 相对父内容盒；descendant.y 相对 child，故只移 child.y 即移整子树。
    let mut cumulative_shift: f32 = 0.0;
    for child in &mut box_node.children {
        // 应用累计位移（来自先前增长的兄弟）——仅 in-flow 非 float 非 abspos 子盒。
        if cumulative_shift > 0.0 && !child.is_absolute && !child.is_fixed && matches!(child.float, FloatValue::None) {
            child.y += cumulative_shift;
        }
        let g = backfill_r109_anon_block_heights(child, styles);
        if g > 0.0 && !child.is_absolute && !child.is_fixed {
            cumulative_shift += g;
        }
    }
    let descendant_growth = cumulative_shift;
    let auto_h = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.height, LengthValue::Auto));
    let mut grew: f32 = 0.0;
    // ① 匿名块盒：从 inline_layout 回填自身 content_height（仅增大）。
    if box_node.fragment_node_ids.is_some()
        && let Some(lines) = &box_node.inline_layout
        && !lines.is_empty()
    {
        let content_h = lines.iter().map(|l| l.y + l.height).fold(0.0f32, f32::max);
        if content_h > box_node.content_height + 0.5 {
            grew = content_h - box_node.content_height;
            box_node.content_height = content_h;
            box_node.height += grew;
        }
    }
    // ② auto-height 容器，含匿名块子 / R109 拆分 inline 子 / 后代增长 → 重算 content_height
    //   = max in-flow 非 float 子盒 border-box 底（仅增大）。max-bottom（CSS §10.6.3）覆盖
    //   「anon 自身欠计」+「容器未把已正确 anon 计入」+「R109 split inline 子盒（is_r109_split）
    //   自身高度已被 ①/② 修对但容器 taffy 测高仍欠计」三种。仅增大守卫避负 margin/margin
    //   折叠误收缩（同 R699 策略）。has_r109_split_child 守 narrow：仅含 R109 拆分 inline 直接
    //   子的容器受影响（welcome 无 R109 split，零回归；区别 R1163 broad「全容器」gate 致 welcome
    //   +12.57pp 回归）。
    let has_anon_child = box_node.children.iter().any(|c| c.fragment_node_ids.is_some());
    let has_r109_split_child = box_node.children.iter().any(|c| c.is_r109_split);
    if auto_h && (has_anon_child || has_r109_split_child || descendant_growth > 0.0) {
        let max_bottom = box_node
            .children
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))
            .map(|c| c.y + c.height)
            .fold(0.0f32, f32::max);
        if max_bottom > box_node.content_height + 0.5 {
            let delta = max_bottom - box_node.content_height;
            box_node.content_height = max_bottom;
            box_node.height += delta;
            grew += delta;
        }
    }
    grew
}

/// CSS §8.3.1：min-height 溢出时阻止子元素 margin「穿透」父元素底部。
///
/// 规则（CSS 2.1 §8.3.1「Adjoining margins」）：块级元素的上/下 margin 只有在其
/// `height` 计算为 `auto` 且 `min-height` 为零时才彼此 adjoining，从而允许末子
/// `margin-bottom`「collapse through」父元素底部（父元素有效下 margin = 末子
/// margin-bottom）。当 `min-height` 把块撑到高于其 in-flow 内容时，末子 margin 不再
/// 穿透——父元素下 margin 应回到自身声明值，后续兄弟紧随父元素。
///
/// taffy 0.7 的 CollapsibleMarginSet 未实现此 min-height 细节：`#parent{min-height:100px}`
/// 包 `#child{height:30px;margin-bottom:550px}`（parent 无 border/padding）时，
/// child 的 550px margin 仍穿透 parent，使后续 footer 被推到 y=parent_bottom+550
/// （`margin-collapse-min-height-001`：应 150px 绿块却渲染成 700px）。
///
/// 本 pass 自顶向下：对每个**容器**，遍历其块级 in-flow 子元素，当一个子元素是
/// min-height 溢出型（min-height_px > 其 in-flow 内容 border-box 底边最大值），把它
/// margin_bottom 超出**自身声明值**的部分（=穿透进来的末子 margin）剥离，并把该子
/// 之后所有兄弟 `.y` 上移同样像素，同时收紧容器 content_height/height。
///
/// 仅在「min-height 真正溢出内容」时触发（`margin-collapse-min-height-003`：
/// min-height:5px < content 30px 时 min-height 不生效，穿透仍合法——本 pass 跳过）。
/// 守卫：仅 block-level、height:auto、非 BFC、无 bottom border/padding 的元素受影响
/// （border/padding 已使 margin 不 adjoining；BFC/abspos 已隔离）。
pub(super) fn prevent_collapse_through_min_height(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    use zero_css_parser::values::LengthValue;

    // 先递归子元素（深度优先；本 pass 只读父级与直接子级几何，不依赖子级已修正，
    // 但递归保证整树都被处理）。
    let children_ids: Vec<Option<NodeId>> = box_node.children.iter().map(|c| c.node_id).collect();

    // 收集每个块级 in-flow 子元素的「是否 min-height 溢出」+ 被剥离的 margin 量，
    // 按文档顺序应用累计位移。
    // blocked_margin[i] = 若 children[i] 是 min-height 溢出型且其 margin_bottom 含穿透量，
    //   则为应剥离的像素（>0）；否则 0。
    let mut blocked: Vec<f32> = vec![0.0; box_node.children.len()];
    for (i, child) in box_node.children.iter().enumerate() {
        // 仅 block-level、in-flow（非 float/abspos）子元素。
        if !child.is_block_level || !matches!(child.float, FloatValue::None) || child.is_absolute || child.is_fixed {
            continue;
        }
        let Some(style) = child.node_id.and_then(|id| styles.get(&id)) else {
            continue;
        };
        // 仅 height:auto（min-height 仅对 auto-height 块产生「撑高」语义）。
        if !matches!(style.height, LengthValue::Auto) {
            continue;
        }
        // BFC 元素的 margin 已与外界隔离（不与父/兄弟折叠），无需处理。
        if crate::margin_collapse::establishes_bfc(child) {
            continue;
        }
        // border/padding-bottom > 0 时 margin 已不 adjoining（穿透本就不发生）。
        if child.border_bottom > 0.0 || child.padding_bottom > 0.0 {
            continue;
        }
        // 解析 min-height（仅 Px；百分比需 definite CB，此处保守取 0 即不触发）。
        let min_h_px = match &style.min_height {
            LengthValue::Px(v) => *v as f32,
            _ => 0.0,
        };
        if min_h_px <= 0.0 {
            continue;
        }
        // in-flow 内容 border-box 底边最大值（相对 child 内容盒顶；child.y 为子相对父内容盒顶，
        // 子内孙相对 child 内容盒顶——这里取 child 的 content_height 作为 in-flow 内容伸展量，
        // 已由 taffy 算好；margin 不计入伸展）。
        // content_extent = child 内 in-flow 孙元素的最大 (y + height)，无孙则为 0。
        let content_extent = in_flow_content_extent(child);
        // min-height 溢出 = min-height 把块撑到高于内容。
        if (min_h_px - content_extent) <= 0.5 {
            continue;
        }
        // child 的声明 margin-bottom（Px）；非 Px 视为 0（保守）。
        let declared_mb = match &style.margin_bottom {
            LengthValue::Px(v) => *v as f32,
            _ => 0.0,
        };
        // 穿透量 = 实际 margin_bottom（含 collapse-through 进来的末子 margin）− 声明值。
        let through = (child.margin_bottom - declared_mb).max(0.0);
        if through > 0.5 {
            blocked[i] = through;
        }
    }

    // 应用：累计位移 shift，对每个子元素按文档顺序上移 .y，并在 min-height 溢出子之后
    // 增大 shift（后续兄弟上移）。最后收紧容器自身高度。
    if blocked.iter().any(|b| *b > 0.5) {
        let mut shift: f32 = 0.0;
        let mut total_blocked = 0.0;
        for (i, child) in box_node.children.iter_mut().enumerate() {
            if shift > 0.5
                && child.is_block_level
                && matches!(child.float, FloatValue::None)
                && !child.is_absolute
                && !child.is_fixed
            {
                child.y -= shift;
            }
            if blocked[i] > 0.5 {
                // 剥离穿透 margin：child.margin_bottom 回到声明值。
                child.margin_bottom -= blocked[i];
                shift += blocked[i];
                total_blocked += blocked[i];
            }
        }
        // 收紧容器 content_height / height（剥离的 margin 不再占父高度）。
        if total_blocked > 0.5 {
            box_node.content_height = (box_node.content_height - total_blocked).max(0.0);
            let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
            box_node.height = (box_node.height - total_blocked).max(pb);
        }
    }

    // 递归子元素。
    let _ = children_ids;
    for child in box_node.children.iter_mut() {
        prevent_collapse_through_min_height(child, styles);
    }
}

/// 计算一个块级 LayoutBox 内 in-flow（非 float/abspos）子元素 border-box 底边的
/// 最大值（相对自身内容盒顶），无 in-flow 子元素则返回 0。用于 §8.3.1 判定
/// min-height 是否「溢出内容」。
pub(super) fn in_flow_content_extent(box_node: &LayoutBox) -> f32 {
    let mut extent: f32 = 0.0;
    for child in &box_node.children {
        if !child.is_block_level || !matches!(child.float, FloatValue::None) || child.is_absolute || child.is_fixed {
            continue;
        }
        extent = extent.max(child.y + child.height);
    }
    extent
}

/// 自上而下收紧百分比 max-height。
///
/// `cb_content_height` 为父级（包含块）的**明确**内容高度；为 `None` 表示父级高度
/// 由内容决定（CSS §10.5：此时百分比 height/max-height 视为 auto，不解析）。
pub(super) fn clamp_percentage_max_height(
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

    // 1a) R587：百分比 min-height 相对包含块内容高度解析（与 1) max-height 对称）。
    // min-height:100% on child of definite-height parent → 解析为 cb_h 的百分比作内容高度下限。
    // min-height-094/095：div2 min-height:100%（父 div1 height:1in 明确）→ 96px。
    // 置于 max-height 之后使 min 优先于 max（§10.4）。
    if let (Some(style), Some(cb_h)) = (style.as_ref(), cb_content_height)
        && let LengthValue::Percentage(p) = &style.min_height
    {
        let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
        let is_border_box = matches!(style.box_sizing, BoxSizingValue::BorderBox);
        let min_box_h = *p as f32 / 100.0 * cb_h;
        let min_content_h = if is_border_box {
            (min_box_h - pb).max(0.0)
        } else {
            min_box_h
        };
        if box_node.content_height < min_content_h {
            box_node.content_height = min_content_h;
            box_node.height = min_content_h + pb;
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
            if let Some(spec) = specified_content_h {
                let pb = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                // R586：table height 下限受 max-height 约束（CSS §10.4：max-height cap 优先于
                // height）。否则 height:3in + max-height:1in 经此 floor 拉回 288，覆盖
                // apply_table_size_constraints 已应用的 max-height cap（max-height-applies-to-013）。
                let spec = match &s.max_height {
                    LengthValue::Px(v) if *v != f64::INFINITY => spec.min((*v as f32 - pb).max(0.0)),
                    _ => spec,
                };
                if box_node.content_height < spec {
                    box_node.content_height = spec;
                    box_node.height = spec + pb;
                }
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
pub(super) fn extract_calc_percentage_and_offset(expr: &zero_css_parser::values::CalcExpr) -> Option<(f64, f64)> {
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
pub(super) fn apply_relative_offsets(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
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
pub(super) fn resolve_relative_inset(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> (f32, f32) {
    use zero_css_parser::values::LengthValue;
    let Some(node_id) = box_node.node_id else {
        return (0.0, 0.0);
    };
    let Some(style) = styles.get(&node_id) else {
        return (0.0, 0.0);
    };
    // CSS §9.4.3/§9.3.2：relative 定位偏移——水平取 left（无 left 时取 right，
    // 正值向左偏移），垂直取 top（无 top 时取 bottom，正值向上偏移）（R716）。
    // 本函数仅处理 Px；Em/Rem/Percent 与 taffy 0.7 percent-inset 限制（R711）同谱系。
    let dx = match &style.left {
        LengthValue::Px(v) => *v as f32,
        _ => match &style.right {
            LengthValue::Px(v) => -(*v as f32),
            _ => 0.0,
        },
    };
    let dy = match &style.top {
        LengthValue::Px(v) => *v as f32,
        _ => match &style.bottom {
            LengthValue::Px(v) => -(*v as f32),
            _ => 0.0,
        },
    };
    (dx, dy)
}

/// R711：block-level `position:relative` 的**百分比** inset（**仅 top/bottom `%`**）。
///
/// taffy 0.7 对 relative 元素：应用 Length inset；**水平（left/right %）也已应用**；
/// 但**丢弃垂直（top/bottom %）inset**（R715 实证：`.pct{relative;top:100%}` 不应用）。
/// CSS §9.4.3：top/bottom % 相对包含块高度解析（CB 高不明确→不解析）。本 pass 自上而下
/// 后处理：对 block-level relative 元素（非 abspos/fixed）补上 top/bottom % delta。
///
/// ★ 仅垂直轴——R850 实证 taffy 已应用 left/right %，本 pass 若也应用会 double-count 致
/// left-103/104/113、right-103/104、relpos-calcs-003/004/005 回归（0.46%→4.28%）。
/// inline-level relative（`apply_relative_offsets_inline`，Px）与 root（`resolve_relative_inset`，
/// Px）由各自路径处理，本 pass 与之正交（仅 block-level 垂直 %）。
///
/// 改 `box.y` 后其子树绝对坐标随累积偏移自然跟随（relative 偏移整个子树）。
pub(super) fn apply_block_relative_percent_insets(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_height: f32,
) {
    use zero_css_parser::values::{DisplayValue, LengthValue};
    fn walk(b: &mut LayoutBox, cb_h: Option<f32>, parent_is_grid: bool, styles: &HashMap<NodeId, ComputedStyle>) {
        // top/bottom % 的 CB 高须**明确（definite）**——css-position-3 §relpos-insets：
        // "a relative-positioned element inset doesn't resolve against an indefinite size"
        //（position-relative-006：parent 仅有 min-height 无 height → indefinite → top:-10000%
        // 不解析）。R711 原仅 style.height==Px 视为 definite，漏掉 R1293 实证的 grid item
        // stretch-definite：grid item（style.height==Auto）在定高 grid 容器中经默认
        // align/justify-items:stretch 拉伸到定值 track，其高度 definite（relative-grandchild：
        // grid item auto-h 拉到 100px，孙 div top:-100% 应解析到 -100px；R711 严格 gate 把
        // auto-h grid item 判 None → 不解析 → 红可见）。
        //
        // R1293 gate（block-level）：
        //   definite = style.height==Px
        //              OR (style.height==Auto AND parent_is_grid AND cb_h.is_some())
        // cb_h.is_some() 已编码「父 grid 容器高度 definite」（grid 容器的 my_content_h）。
        // auto-height 普通 block 容器（position-relative-006）parent_is_grid=false → indefinite
        // → 不解析（与 chromium 一致）。inline 元素透传 cb_h（R1044 谱系）。
        // kill-switch `ZW_RELPOS_PCT_AUTO_CB=0`（default-on）回退 R711 严格 gate（仅 ==Px）。
        let strict_r711 = std::env::var("ZW_RELPOS_PCT_AUTO_CB").as_deref() == Ok("0");
        let style = b.node_id.and_then(|id| styles.get(&id));
        let height_definite = if strict_r711 {
            style.is_some_and(|s| matches!(s.height, LengthValue::Px(_)))
        } else {
            style.is_some_and(|s| match s.height {
                LengthValue::Px(_) => true,
                LengthValue::Auto => parent_is_grid && cb_h.is_some(),
                _ => false,
            })
        };
        let my_content_h = if b.is_block_level {
            height_definite.then_some(b.content_height)
        } else {
            cb_h
        };
        // 本盒是否 grid 容器（传给子代作 parent_is_grid）。
        let my_is_grid = style.is_some_and(|s| matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid));

        // 应用本盒 Percent inset（relative，非 abspos/fixed；block-level **和** inline）。
        // ★ 仅 top/bottom%（垂直轴）——R850 实证 taffy 0.7 已应用 left/right%（水平轴），
        // 此处再应用会 double-count 致 left-103/104/113、right-103/104、relpos-calcs-003/004/005
        // 回归（0.46%→4.28%）。taffy 仅丢弃 top/bottom%（R715 实证），故本 pass 只补垂直轴。
        // ★ R1044：移除 is_block_level 门控——inline relative（如 R109-split span）的 top/bottom %
        // 同样被 taffy 丢弃，须一并补（position-relative-001：inline span top:100% 未应用）。
        if b.is_relative
            && !b.is_absolute
            && !b.is_fixed
            && let Some(style) = style
        {
            // CSS §9.4.3：top 优先（否则 bottom，正值向上）。
            let dy = match &style.top {
                LengthValue::Percentage(p) => cb_h.map(|h| *p as f32 / 100.0 * h),
                _ => match &style.bottom {
                    LengthValue::Percentage(p) => cb_h.map(|h| -(*p as f32 / 100.0 * h)),
                    _ => None,
                },
            };
            if let Some(dy) = dy {
                b.y += dy;
            }
        }

        for child in &mut b.children {
            walk(child, my_content_h, my_is_grid, styles);
        }
    }
    // 根的 CB = 视口（ICB）；根无 grid 父。
    walk(box_node, Some(viewport_height), false, styles);
}

/// 将 OverflowValue 转换为 OverflowClip。
pub(super) fn convert_overflow_to_clip(value: &OverflowValue) -> OverflowClip {
    match value {
        OverflowValue::Visible => OverflowClip::Visible,
        OverflowValue::Hidden => OverflowClip::Hidden,
        OverflowValue::Clip => OverflowClip::Clip,
        OverflowValue::Scroll | OverflowValue::Auto => OverflowClip::Scroll,
    }
}
