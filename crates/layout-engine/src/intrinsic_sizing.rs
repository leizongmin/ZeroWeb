//! 容器固有宽度（intrinsic / max-content）测量工具。
//!
//! 为 flex/grid 容器两趟固有宽度布局（见 `docs/goal/rendering-compat/flex-grid-two-pass-design.md`）
//! 提供测量基础。本模块**不参与布局**（compute() 不调用其改变布局的函数），
//! 仅提供纯计算函数 + 单元测试 + 可选诊断打印（env-gated），分轮渐进接线。
//!
//! 关键区别于 `table_shrink::block_max_content_width`：本模块的 `box_content_max_width`
//! 对「叶 block 显式 width」回退到自身显式宽度（R138 的函数对此返回 0，会漏测
//! `<div style="width:30px">` 这类叶盒），故 grid item 的固有宽度才能正确测量。

use std::collections::HashMap;

use zero_css_parser::values::{BoxSizingValue, DisplayValue, FlexDirectionValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::FlexBasisValue;

use crate::types::LayoutBox;

/// 计算一个盒的「内容最大宽度」（max-content）。
///
/// 递归规则（CSS intrinsic sizing）：
/// - inline 级子元素（含 inline-block）→ 水平求和（max-content 假设不换行）
/// - block 级子元素 → 取最大者的内容宽度
/// - **叶盒（无有效子元素贡献）且有显式 Px width → 回退到自身显式 width**
///   （这是与 `table_shrink::block_max_content_width` 的关键差异）
/// - **叶盒的文本内容**（Round C）：纯文本 item 此前测 0 致 flex/grid 容器 intrinsic
///   塌缩；此处按元素 font 度量逐字符累加文本宽度（Ahem 等宽=font_size）。
///   仅 max-content（不换行）；min-content（最宽词）独立子问题暂不实现。
///
/// 返回值含 box 自身的水平 padding+border（border-box 贡献）。
pub(crate) fn box_content_max_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        let is_inline_level = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                matches!(
                    s.display,
                    DisplayValue::Inline
                        | DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
            .unwrap_or(false);
        let outer_w = child.width + child.margin_left + child.margin_right;
        if is_inline_level {
            inline_sum += outer_w.max(0.0);
        } else {
            block_max = block_max.max(box_content_max_width(child, doc, styles));
        }
    }

    let children_inner = inline_sum.max(block_max);

    // 叶盒回退：无有效子元素贡献时，用自身显式 Px width（content-box 语义）。
    // 显式 width 的叶盒（如 `<div style="width:50px">`）其 max-content 即该宽度。
    let own_explicit = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child {
        // 叶盒：显式宽或文本内容宽（Round C）。纯文本 item（无 LayoutBox 子元素）
        // 之前测 0，现按 DOM 文本内容度量。取 max 避免显式宽被文本低估。
        let text_w = box_node
            .node_id
            .map_or(0.0, |id| text_content_max_width(id, doc, styles));
        own_explicit.max(text_w)
    } else if children_inner < own_explicit {
        own_explicit
    } else {
        children_inner
    };

    inner + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right
}

/// R1018：block-level 容器的 max-content 宽度，对 flex/grid **子容器**分发到专用 intrinsic 函数。
///
/// 区别于 [`box_content_max_width`] 的通用递归：当 block 的子元素本身是 flex/grid 容器时，
/// flex/grid 容器的 intrinsic 宽度须用专用测量（`flex_row_intrinsic_width` 等，含 transferred
/// sizing / aspect-ratio 推导），而非通用递归（通用递归对 aspect-ratio 空 item 测 0）。
///
/// 用于 `width:max-content`/`fit-content` block 的 shrink-to-fit（CSS css-sizing-3）。返回 border-box。
/// 仅水平书写模式。leaf 文本/显式宽回退同 [`box_content_max_width`]。
pub(crate) fn block_max_content_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        let child_style = child.node_id.and_then(|id| styles.get(&id));
        let is_inline_level = child_style
            .map(|s| {
                matches!(
                    s.display,
                    DisplayValue::Inline
                        | DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
            .unwrap_or(false);
        if is_inline_level {
            // inline-level 子：用 outer_w（已布局宽度）求和。inline-flex/inline-grid 的
            // intrinsic 测量由 shrink_inline_blocks_to_content（R180/R1017）路径处理，此处不重复。
            inline_sum += (child.width + child.margin_left + child.margin_right).max(0.0);
            continue;
        }
        // block-level 子：若是 flex/grid 容器，dispatch 到专用 intrinsic 函数（R1018 关键）。
        let child_intrinsic = child_style
            .map(|s| match s.display {
                DisplayValue::Flex | DisplayValue::InlineFlex => {
                    let base = if matches!(
                        s.flex_direction,
                        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                    ) {
                        flex_column_intrinsic_width(child, doc, styles)
                    } else {
                        flex_row_intrinsic_width(child, doc, styles)
                    };
                    base.unwrap_or(0.0)
                }
                DisplayValue::Grid | DisplayValue::InlineGrid => {
                    grid_intrinsic_width(child, doc, styles).unwrap_or(0.0)
                }
                _ => box_content_max_width(child, doc, styles),
            })
            .unwrap_or_else(|| box_content_max_width(child, doc, styles));
        block_max = block_max.max(child_intrinsic + child.margin_left + child.margin_right);
    }

    let children_inner = inline_sum.max(block_max);

    // leaf 回退同 box_content_max_width：显式 Px width 或文本内容宽。
    let own_explicit = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child {
        let text_w = box_node
            .node_id
            .map_or(0.0, |id| text_content_max_width(id, doc, styles));
        own_explicit.max(text_w)
    } else if children_inner < own_explicit {
        own_explicit
    } else {
        children_inner
    };

    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;

    // R1020：multicol 容器（column-count:N）shrink-to-fit intrinsic = N × column_content + (N-1) × gap，
    // **仅当所有 in-flow 子都是 leaf（无元素子）**——驱动案 change-intrinsic-width（columns:2 +
    // 2 个 50px leaf 子 → 100）、intrinsic-width-change-column-count（columns:4 + 25px leaf → 100）。
    // column-span:all 子（含嵌套元素，intrinsic-size-002/003/004）跨全宽不应乘 N——其有元素子，
    // 守卫跳过，回落 max（span:all content = 全宽）。ZW 暂未解析 column-span，用「子是否 leaf」
    // 作代理判定（span:all 通常含嵌套结构）。
    let col_count = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match s.column_count {
            zero_style_system::ColumnCountComputedValue::Number(n) => Some(n as usize),
            _ => None,
        });
    if let Some(n) = col_count
        && n >= 2
        && box_node
            .children
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed)
            .all(|c| c.children.iter().all(|gc| gc.is_absolute || gc.is_fixed))
    {
        let gap_px = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .and_then(|s| match &s.column_gap {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
            .unwrap_or(0.0);
        return (n as f32) * inner + ((n - 1) as f32) * gap_px + frame;
    }

    inner + frame
}

/// 测量一个 DOM 元素的文本内容 max-content 宽度（Round C：纯文本 flex/grid item 测量）。
///
/// 遍历 DOM 后代收集全部文本（`Document::text_content`），按 CSS 白空格折叠规则折叠后，
/// 用元素 font 度量逐字符累加宽度（复用 IFC 的 `estimate_char_width`：Ahem 等宽=font_size，
/// 其它字体按字符近似宽）。仅 max-content（假设不换行）；min-content（最宽词）独立子问题。
fn text_content_max_width(node_id: NodeId, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let text = doc.text_content(node_id).unwrap_or_default();
    let collapsed = crate::inline::collapse_whitespace(&text);
    if collapsed.is_empty() {
        return 0.0;
    }
    let style = styles.get(&node_id);
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(style);
    let is_ahem = style.is_some_and(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")));
    collapsed
        .chars()
        .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem))
        .sum()
}

/// R109 §9.2.1.1：测量 split inline 的一个匿名块片段的 inline 内容 max-content 宽度。
///
/// 片段内的 DOM 子节点（文本节点 + inline-level 元素）按 inline 级求和（max-content
/// 假设不换行），字体度量取自 split inline 自身（片段继承其 font-family/size）。
/// 用于匿名块收缩到文本宽，使 inline 的 border/background 落在文本宽而非全宽
/// （inline-box-001 等 §9.2.1.1 用例）。返回 0 = 不可测（无文本）。
pub(crate) fn fragment_inline_max_width(
    inline_style: &ComputedStyle,
    fragment_node_ids: &[NodeId],
    doc: &Document,
) -> f32 {
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(Some(inline_style));
    let is_ahem = inline_style.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem"));
    let mut total = 0.0f32;
    for nid in fragment_node_ids {
        let text = doc.text_content(*nid).unwrap_or_default();
        let collapsed = crate::inline::collapse_whitespace(&text);
        total += collapsed
            .chars()
            .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem))
            .sum::<f32>();
    }
    total
}

/// 计算 flex item 的主轴 base size（CSS Flexbox §9.2 flex base size）。
///
/// 优先级：`flex-basis` 显式长度 > `width` 显式长度 > 内容 max-content。
/// - `flex-basis: auto`/`content` → 回退到 width 或内容
/// - 无法确定（无显式值且内容为 0）→ 返回 0.0（调用方应作 no-op 处理）
///
/// 返回 border-box 贡献（含 item 自身 padding+border，不含 margin——margin 由容器求和时加）。
fn flex_item_base_size(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    container_cross: Option<f32>,
) -> f32 {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    // 1. flex-basis 显式长度优先
    if let Some(s) = style
        && let FlexBasisValue::Length(len) = &s.flex_basis
    {
        if let LengthValue::Px(v) = len {
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            return (*v as f32) + frame;
        }
    }
    // 2. width 显式长度
    if let Some(s) = style
        && let LengthValue::Px(v) = &s.width
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return (*v as f32) + frame;
    }
    // 2.5 R1015/R1017：aspect-ratio transferred-size——width:auto + aspect_ratio + definite main。
    // main 来源优先级：(a) item 自身 height Px；(b) item min-height Px 地板；(c) R1017 container-
    // stretch cross（容器 definite height Px 拉伸 item，如 inline-flex height:100px；经
    // shrink_inline_blocks_to_content IFC 路径调用，绕过 R1016 的 taffy gate 墙）。
    if let Some(s) = style
        && matches!(s.width, LengthValue::Auto)
        && let Some(ratio) = s.aspect_ratio.filter(|&r| r > 0.0)
    {
        let main = match &s.height {
            LengthValue::Px(v) => Some(*v as f32),
            _ => match &s.min_height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => container_cross,
            },
        };
        if let Some(main) = main {
            return aspect_ratio_transferred_width(s, box_node, main, ratio);
        }
    }
    // 3. 内容 max-content（Round C：含纯文本 item 的文本宽度）
    box_content_max_width(box_node, doc, styles)
}

/// R1015：aspect-ratio transferred width（非替换 item）。`main` = item definite main-size（height）
/// 的 Px 数值（border-box 或 content-box 由 `box-sizing` 决定）。返回 border-box width。
///
/// - `border-box`：aspect-ratio 作用于 border-box，width_bb = height_bb × ratio = main × ratio。
/// - `content-box`：aspect-ratio 作用于 content-box，width_content = main × ratio，
///   border-box width = width_content + 水平 frame。
fn aspect_ratio_transferred_width(s: &ComputedStyle, box_node: &LayoutBox, main: f32, ratio: f32) -> f32 {
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
        main * ratio
    } else {
        main * ratio + frame
    }
}

/// 计算一个**水平 flex 行容器**的固有宽度（max-content 主尺寸）。
///
/// = Σ flex item base size + item margins + gaps + 容器水平 padding/border。
/// 仅对 `display:flex`/`inline-flex` 且主轴为水平（flex-direction: row/row-reverse）的容器有意义。
/// 返回 None 表示无法确定（如无流内 item）。
pub(crate) fn flex_row_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    // R1017：容器 definite cross（height Px）作 item stretch 源——item 无自身 main 时，
    // width = container_content_height × ratio（inline-flex height:100px + item aspect-ratio:1/1）。
    // R1018：百分比/auto height 经 taffy 第一趟已解析到 LayoutBox.height（border-box），
    // 作 fallback container_cross（flex 子 height:100% 在 definite-height 父内已解析）。
    let container_cross = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.height {
            LengthValue::Px(v) => {
                let vframe =
                    box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                let content = if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                    (*v as f32) - vframe
                } else {
                    *v as f32
                };
                Some(content.max(0.0))
            }
            _ => {
                // 非 Px（百分比/auto/em）：用 taffy 第一趟解析的 border-box height 减 frame。
                let vframe =
                    box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                let resolved = (box_node.height - vframe).max(0.0);
                (resolved > 0.0).then_some(resolved)
            }
        });
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        // 仅统计直接 flex item（block 级流内子元素）
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            sum += flex_item_base_size(child, doc, styles, container_cross) + child.margin_left + child.margin_right;
        }
    }
    if count == 0 {
        return None;
    }
    let gap = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    Some(sum + gap * (count - 1) as f32 + frame)
}

/// 计算一个**垂直 flex 列容器**的固有宽度（cross 轴 max-content）。
///
/// = max(item base size + item margins) + 容器水平 padding/border。列容器的主轴是垂直，
/// cross 轴（width）取最宽 item（非 row 的求和）。R1015：驱动案 flex-item-transferred-sizes-padding
///（float:left + flex-direction:column + item aspect-ratio:1/1 + min-height:100px）。
/// 返回 None 表示无法确定（如无流内 item）。
pub(crate) fn flex_column_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    let mut max = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            // column：computing container width（cross）— container_cross = width 是循环，传 None。
            let base = flex_item_base_size(child, doc, styles, None) + child.margin_left + child.margin_right;
            if base > max {
                max = base;
            }
        }
    }
    if count == 0 {
        return None;
    }
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    Some(max + frame)
}

/// 计算一个 **grid 容器**的固有宽度（max-content 主尺寸）。
///
/// 近似实现（taffy 0.7 无原生 grid auto-track 扩展，此处用 item base size 估算）：
/// - `grid-auto-flow: column`（item 水平排列）→ Σ item base size + gaps
/// - 其它（默认 row，item 垂直堆叠）→ max item base size
///
/// 其中 item base size = `box_content_max_width`（含叶显式宽回退，故 `.item > .content(50px)`
/// 会测为 50+frame）。返回 None 表示无流内 item。
pub(crate) fn grid_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    let is_column_flow = style
        .map(|s| {
            // grid-auto-flow 含 "column" → item 水平排列
            matches!(
                s.grid_auto_flow,
                zero_style_system::property::types::GridAutoFlowValue::Column
                    | zero_style_system::property::types::GridAutoFlowValue::ColumnDense
            )
        })
        .unwrap_or(false);
    let gap = style
        .and_then(|s| match &s.column_gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let mut sum = 0.0f32;
    let mut max_w = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            let base = box_content_max_width(child, doc, styles) + child.margin_left + child.margin_right;
            sum += base;
            max_w = max_w.max(base);
        }
    }
    if count == 0 {
        return None;
    }
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    // 显式 grid-template-columns 时，每个 item 落入一个独立列，grid 的 max-content
    // 宽度 = 各列 max-content 之和（而非默认 row flow 单列取最大）。
    // 保守守卫：仅当显式 track 数 >= item 数时求和（每 item 独占一列），避免 item
    // 跨行换列导致过计。fit-content(L)/固定长度 track 的 L 钳制未建模（item 的
    // min-content 地板通常已 >= L，故不缩窄；残余边界由 reftest 验证）。
    let multi_column = is_column_flow || style.and_then(count_explicit_grid_columns).is_some_and(|n| n >= count);
    let inner = if multi_column {
        sum + gap * (count - 1) as f32
    } else {
        max_w
    };
    Some(inner + frame)
}

/// 统计显式 `grid-template-columns` 定义的 track 数（用于 grid 内在宽度测量）。
///
/// 括号感知按空白分割：`fit-content(30px)`、`minmax(a,b)`、`repeat(n, ...)` 各算 1 个
/// token（`repeat` 展开计数复杂，保守按 1 计——只会少计 track 数，不会误判为多列）。
/// 返回 `None` 表示无显式列定义（默认 None 或 `none`）。
fn count_explicit_grid_columns(s: &ComputedStyle) -> Option<usize> {
    let cols = s.grid_template_columns.as_deref()?.trim();
    if cols.is_empty() || cols.eq_ignore_ascii_case("none") {
        return None;
    }
    let mut count = 0usize;
    let mut depth = 0i32;
    let mut in_token = false;
    for ch in cols.chars() {
        match ch {
            '(' => {
                depth += 1;
                in_token = true;
            }
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if in_token {
                    count += 1;
                    in_token = false;
                }
            }
            _ => in_token = true,
        }
    }
    if in_token {
        count += 1;
    }
    (count > 0).then_some(count)
}

/// 判断一个盒是否是 flex/grid 行容器（display:flex/inline-flex/grid/inline-grid）。
fn is_flex_grid_container(s: &ComputedStyle) -> bool {
    matches!(
        s.display,
        DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
    )
}

/// 诊断：遍历布局树，对 shrink-to-fit 候选容器打印测得的固有宽度 vs 当前宽度。
///
/// 候选 = flex/grid 容器且（width 为 auto/max-content/min-content，或容器本身是 inline-level
/// 或 float——这些应 shrink-to-fit 而非填满）。**仅 eprintln，不改变任何布局状态**（Round A）。
pub(crate) fn debug_dump_shrink_candidates(root: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    fn walk(b: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
        let Some(id) = b.node_id else {
            for c in &b.children {
                walk(c, doc, styles);
            }
            return;
        };
        let Some(s) = styles.get(&id) else {
            for c in &b.children {
                walk(c, doc, styles);
            }
            return;
        };
        if is_flex_grid_container(s) {
            let width_indefinite = matches!(
                s.width,
                LengthValue::Auto | LengthValue::MaxContent | LengthValue::MinContent
            );
            let is_inline = matches!(s.display, DisplayValue::InlineFlex | DisplayValue::InlineGrid);
            let is_float = !matches!(b.float, zero_css_parser::values::FloatValue::None);
            if width_indefinite || is_inline || is_float {
                let intrinsic = if matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid) {
                    grid_intrinsic_width(b, doc, styles)
                } else {
                    flex_row_intrinsic_width(b, doc, styles)
                };
                if let Some(intrinsic) = intrinsic {
                    eprintln!(
                        "INTRINSIC_DBG: {:?} width={:?} float={:?} current_w={} intrinsic_w={} (delta={:.1})",
                        s.display,
                        s.width,
                        b.float,
                        b.width,
                        intrinsic,
                        b.width - intrinsic
                    );
                }
            }
        }
        for c in &b.children {
            walk(c, doc, styles);
        }
    }
    walk(root, doc, styles);
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::NodeKind;

    /// 用 DOM 解析真实 HTML 计算样式，验证端到端测量。
    fn compute_intrinsic(html: &str, target_id: &str) -> Option<f32> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find(target_id, &doc, &result.root)?;
        flex_row_intrinsic_width(target, &doc, &styles)
    }

    /// 用 DOM 解析真实 HTML 计算样式，验证 grid 固有宽度测量（column flow 求和）。
    fn compute_grid_intrinsic(html: &str, target_id: &str) -> Option<f32> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find(target_id, &doc, &result.root)?;
        grid_intrinsic_width(target, &doc, &styles)
    }

    #[test]
    fn test_grid_column_flow_sums_items() {
        // child-border-box-and-max-content 结构：grid-auto-flow:column，2 item，
        // 每个 item = .content(50) + padding 20×2 = 90 → grid 固有 = 180。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-auto-columns:1fr;grid-auto-flow:column">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!((w - 180.0).abs() < 2.0, "expected ~180px (2×(50+40)), got {}", w);
    }

    #[test]
    fn test_grid_row_flow_takes_max() {
        // 默认 grid-auto-flow:row → item 垂直堆叠 → 取最大 item 宽度（50）。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!((w - 50.0).abs() < 1.0, "expected ~50px (max item), got {}", w);
    }

    #[test]
    fn test_grid_explicit_columns_sum_items() {
        // child-border-box-and-max-content-002 结构：显式 grid-template-columns
        // 2 个 fit-content track，2 item 各占一列 → grid 固有 = 各 item 求和（180），
        // 而非默认 row flow 的取最大（90）。item = .content(50) + padding 20×2 = 90。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-template-columns:fit-content(30px) fit-content(80px)">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!(
            (w - 180.0).abs() < 2.0,
            "expected ~180px (2×90, explicit columns sum), got {}",
            w
        );
    }

    #[test]
    fn test_grid_explicit_columns_fewer_tracks_takes_max() {
        // 显式 1 个 track，2 个 item → item 会换行到第 2 行复用同一列；
        // 保守取最大 item 宽度（不冒险过计），而非求和。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-template-columns:100px">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!(
            (w - 50.0).abs() < 1.0,
            "expected ~50px (max item, fewer tracks than items), got {}",
            w
        );
    }

    #[test]
    fn test_leaf_explicit_width_fallback() {
        // `.item > .content(width:50px)`：item max-content 应含 content 的 50px
        // （box_content_max_width 对叶 content 回退到 50）。
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        // 单 item width:50 → 50（无 padding/border）
        assert!((w - 50.0).abs() < 1.0, "expected ~50px, got {}", w);
    }

    #[test]
    fn test_flex_row_sum_two_items() {
        // 两个显式宽 item：30 + 50 = 80（行固有宽度）
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!((w - 80.0).abs() < 1.0, "expected ~80px (30+50), got {}", w);
    }

    #[test]
    fn test_flex_basis_overrides_width() {
        // flex-basis 显式优先于 width：flex-basis:40px + width:50px → base 40
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="flex-basis:40px;width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 40.0).abs() < 1.0,
            "flex-basis should win (expected ~40), got {}",
            w
        );
    }

    #[test]
    fn test_item_padding_adds_to_base() {
        // item 有 padding：width:50 + padding 10+10 = 70 border-box base
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:50px;padding:0 10px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!((w - 70.0).abs() < 1.0, "expected ~70 (50+20 padding), got {}", w);
    }

    #[test]
    fn test_text_only_item_measured_round_c() {
        // Round C：纯文本 flex item（Ahem 10px 等宽）此前测 0，现按文本内容度量。
        // 5 字符 "XXXXX" × 10px = 50px（item 无 padding/border/margin）。
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex;font:10px/1 Ahem">
            <div>XXXXX</div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 50.0).abs() < 1.0,
            "expected ~50px (5×10px Ahem text, Round C), got {}",
            w
        );
    }

    #[test]
    fn test_nested_explicit_child_grid_like() {
        // grid item 场景：`.item(padding 20) > .content(width:50)` → item 内容 max = 50+40 = 90
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 90.0).abs() < 1.0,
            "expected ~90 (50 content + 40 padding), got {}",
            w
        );
    }

    #[test]
    fn test_empty_container_returns_none() {
        let html = r#"<html><body style="margin:0"><div id="c" style="display:flex"></div></body></html>"#;
        let w = compute_intrinsic(html, "c");
        assert!(w.is_none(), "empty flex container should return None");
    }
}
