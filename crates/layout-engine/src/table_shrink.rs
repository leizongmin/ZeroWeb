//! 对「无正规表格子元素」的 `display:table` 容器执行收缩适应。
//!
//! 从 `table.rs` 拆分出来，避免该文件继续膨胀（单文件行数控制）。

use std::collections::HashMap;

use zero_css_parser::values::{BoxSizingValue, DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::table::is_table_internal;
use crate::types::LayoutBox;

/// 对「无正规表格子元素」的 display:table 容器执行收缩适应。
///
/// CSS Tables §2.4（匿名盒生成）：当 `display:table` 容器的子元素不是
/// table-row / table-cell / table-row-group 等正规表格盒时（典型：`<html
/// style="display:table">` 内的 `<body>` block），规范要求为它们生成匿名
/// row+cell 包装盒，使 table 收缩到内容的固有宽度/高度，而非像 block 那样
/// 填满包含块。
///
/// 由于 ZeroWeb 的 table 布局是对 taffy 结果的后处理（无法重新生成匿名盒
/// 并重跑布局），这里用一个近似实现：当 `build_grid` 产出的 grid 为空时，直接
/// 计算 block 子元素的 max-content 宽度与内容高度，收缩 table 及其 block
/// 子元素的盒尺寸（不改变子元素的 x/y 位置——它们已由 taffy 正确定位到 table
/// 的内容盒起点）。
///
/// 仅在 grid 为空时触发，所有有正规表格结构的 table（已通过的 css-tables 用例）
/// 都有非空 grid，故此函数对它们零影响。
pub(crate) fn shrink_table_to_block_content(table_box: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    // 收集 block 级「非正规表格子元素」（按 DOM 顺序）。
    let block_indices: Vec<usize> = table_box
        .children
        .iter()
        .enumerate()
        .filter(|(_, ch)| {
            // 仅处理真实元素节点（跳过匿名盒，如 head 元素产生的空盒）
            let Some(id) = ch.node_id else {
                return false;
            };
            let Some(s) = styles.get(&id) else {
                return false;
            };
            // 排除 display:none / contents / table-internal（这些不该走此路径）
            if matches!(s.display, DisplayValue::None | DisplayValue::Contents) || is_table_internal(&s.display) {
                return false;
            }
            // block 级显示类型（body、div 等流内 block 子元素）
            matches!(
                s.display,
                DisplayValue::Block
                    | DisplayValue::Flex
                    | DisplayValue::Grid
                    | DisplayValue::FlowRoot
                    | DisplayValue::ListItem
                    | DisplayValue::Flow
            )
        })
        .map(|(i, _)| i)
        .collect();

    if block_indices.is_empty() {
        // 无 block 级「非正规表格子元素」。
        // 完全为空（无任何子盒）的 display:table 仍须收缩到 max-content=0（仅
        // padding+border），CSS Tables §2.4 / CSS 2.1 §17.5.2——典型：
        // `<div style="display:table;padding:155px">` 应为 312×312，而非像 block
        // 那样填满包含块。仅有匿名盒（如 head 产生的空盒）时无真实 in-flow 内容，
        // 保持 no-op（历史行为，不改变既有用例）。
        if table_box.children.is_empty() {
            shrink_empty_table_to_padding_border(table_box, styles);
        }
        return;
    }

    // 计算 block 子元素的 max-content 宽度（多个 block 子元素垂直堆叠 → 取最大）
    let max_content_width: f32 = block_indices
        .iter()
        .filter_map(|&i| table_box.children.get(i))
        .map(|ch| block_max_content_width(ch, styles))
        .fold(0.0f32, f32::max);

    if max_content_width <= 0.0 {
        return;
    }

    // 解析 table 的显式 width / min-width / max-width（内容盒语义）
    let table_style = table_box.node_id.and_then(|id| styles.get(&id));
    let is_border_box = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.box_sizing, BoxSizingValue::BorderBox));
    let padding_border_w =
        table_box.padding_left + table_box.padding_right + table_box.border_left + table_box.border_right;
    let padding_border_h =
        table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;

    // 默认收缩到 max-content；显式 width 时尊重显式宽度
    let mut final_content_width = max_content_width;
    if let Some(s) = table_style
        && let LengthValue::Px(v) = &s.width
    {
        final_content_width = if is_border_box {
            (*v as f32 - padding_border_w).max(0.0)
        } else {
            *v as f32
        };
    }
    // min-width / max-width 约束
    if let Some(s) = table_style
        && let LengthValue::Px(v) = &s.min_width
    {
        let min_c = if is_border_box {
            (*v as f32 - padding_border_w).max(0.0)
        } else {
            *v as f32
        };
        final_content_width = final_content_width.max(min_c);
    }
    if let Some(s) = table_style
        && let LengthValue::Px(v) = &s.max_width
        && *v != f64::INFINITY
    {
        let max_c = if is_border_box {
            (*v as f32 - padding_border_w).max(0.0)
        } else {
            *v as f32
        };
        final_content_width = final_content_width.min(max_c);
    }
    // 不超过当前可用宽度（taffy 分配的容器宽度）
    final_content_width = final_content_width.min(table_box.content_width).max(0.0);

    // 计算 table 内容高度：最低 block 子元素底缘相对内容盒的高度
    let content_box_top = table_box.border_top + table_box.padding_top;
    let content_height: f32 = block_indices
        .iter()
        .filter_map(|&i| table_box.children.get(i))
        .map(|ch| (ch.y - content_box_top + ch.height + ch.margin_bottom).max(0.0))
        .fold(0.0f32, f32::max)
        .max(0.0);

    // 收缩 block 子元素宽度，使其盒匹配新的 table 内容宽度（block 填满 table 内容盒）
    for &i in &block_indices {
        let Some(child) = table_box.children.get_mut(i) else {
            continue;
        };
        child.width = final_content_width;
        let cw =
            (final_content_width - child.border_left - child.border_right - child.padding_left - child.padding_right)
                .max(0.0);
        child.content_width = cw;
    }

    // 收缩 table 自身尺寸（width:auto 的 table 应收缩到内容，而非 taffy 的块级填充）
    let old_border_width = table_box.width;
    let new_border_width = final_content_width + padding_border_w;
    table_box.content_width = final_content_width;
    table_box.width = new_border_width;
    // CSS §17.5.2：table 的显式 height 作 min-height 语义（table 高度由内容决定但不小于
    // 显式值）。CSS §10.4/§17.5.2 + quirks 表高度（已预烘焙）：width/height 显式时须尊重。
    // orphan display:table-cell（如 percentages-grandchildren-quirks-mode：table-cell
    // height:100px + 0 内容子）显式 height 不应被 0 内容塌缩。
    let explicit_height = table_style
        .as_ref()
        .and_then(|s| match &s.height {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let final_content_height = content_height.max((explicit_height - padding_border_h).max(0.0));
    table_box.content_height = final_content_height;
    table_box.height = final_content_height + padding_border_h;

    // CSS §10.3.3 / §17.5.2：width:auto 的 display:table 收缩到内容后，若
    // margin-left/right 均为 auto，应水平居中于包含块。taffy 在收缩前（table 仍
    // 填满包含块）已把 auto margin 解析为 0，收缩后未重新居中（典型：<html
    // style="display:table;margin:auto">）。这里把 table 平移到居中位置。
    //（子元素 x 相对于 table 内容盒，paint 累积偏移 offset_x+box.x，故只改 table.x
    //   即可让整棵 table 子树整体居中，无需逐子元素平移。）
    let both_margins_auto = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.margin_left, LengthValue::Auto) && matches!(s.margin_right, LengthValue::Auto));
    if both_margins_auto && new_border_width + 0.5 < old_border_width {
        let margin = (old_border_width - new_border_width) / 2.0;
        table_box.x += margin;
        table_box.margin_left = margin;
        table_box.margin_right = margin;
    }
}

/// 对完全为空（无任何子盒）且 width:auto 的 display:table 容器执行收缩适应。
///
/// 空 table 的 max-content 宽/高均为 0，故 width:auto 的 table 应收缩到仅
/// padding+border（CSS Tables §2.4 + CSS 2.1 §17.5.2），而非像 block 那样
/// 填满包含块。min-width 下限仍被尊重（内容盒语义）。margin:auto 收缩后水平
/// 居中（与 [`shrink_table_to_block_content`] 一致）。
///
/// **仅 width:auto 触发**：显式 width（px/percent/em/calc 等）已由 taffy 正确
/// 解析（abspos `width:50%`→CB、normal-flow `width:100%`→CB），覆盖会破坏这些
/// 用例（absolute-tables-007/012/016、subpixel-table-width-001 均为显式宽度）。
fn shrink_empty_table_to_padding_border(table_box: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    let table_style = table_box.node_id.and_then(|id| styles.get(&id));

    // 仅 width:auto 收缩；显式 width 由 taffy 解析，此处不动。
    let is_auto_width = table_style
        .as_ref()
        .map(|s| matches!(s.width, LengthValue::Auto))
        .unwrap_or(false);
    if !is_auto_width {
        return;
    }

    let is_border_box = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.box_sizing, BoxSizingValue::BorderBox));
    let padding_border_w =
        table_box.padding_left + table_box.padding_right + table_box.border_left + table_box.border_right;
    let padding_border_h =
        table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;

    // max-content=0；min-width 下限约束
    let mut final_content_width = 0.0f32;
    if let Some(s) = table_style
        && let LengthValue::Px(v) = &s.min_width
    {
        let min_c = if is_border_box {
            (*v as f32 - padding_border_w).max(0.0)
        } else {
            *v as f32
        };
        final_content_width = final_content_width.max(min_c);
    }
    if let Some(s) = table_style
        && let LengthValue::Px(v) = &s.max_width
        && *v != f64::INFINITY
    {
        let max_c = if is_border_box {
            (*v as f32 - padding_border_w).max(0.0)
        } else {
            *v as f32
        };
        final_content_width = final_content_width.min(max_c);
    }
    // 不超过当前可用宽度
    final_content_width = final_content_width.min(table_box.content_width).max(0.0);

    let old_border_width = table_box.width;
    let new_border_width = final_content_width + padding_border_w;
    table_box.content_width = final_content_width;
    table_box.width = new_border_width;
    table_box.content_height = 0.0;
    table_box.height = padding_border_h;

    // margin:auto 居中（收缩后 taffy 未重新居中）
    let both_margins_auto = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.margin_left, LengthValue::Auto) && matches!(s.margin_right, LengthValue::Auto));
    if both_margins_auto && new_border_width + 0.5 < old_border_width {
        let margin = (old_border_width - new_border_width) / 2.0;
        table_box.x += margin;
        table_box.margin_left = margin;
        table_box.margin_right = margin;
    }
}

/// 估算 block 容器的 max-content（最大内容）宽度。
///
/// max-content 假设无强制换行：
/// - 行内级子元素（inline / inline-block 等）在同一行水平排列 → 宽度求和；
/// - block 级子元素垂直堆叠 → 取各自 max-content 的最大值。
///
/// 返回值含容器自身的水平 padding+border。
fn block_max_content_width(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;

    for child in &box_node.children {
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
            // 行内级子元素水平求和（max-content 假设不换行）
            inline_sum += outer_w.max(0.0);
        } else {
            // block 级子元素递归取最大
            block_max = block_max.max(block_max_content_width(child, styles));
        }
    }

    let inner = inline_sum.max(block_max);
    // 元素自身显式 Px 宽度是 max-content 的下界：shrink-to-fit 不应把 table 收缩到
    // 小于 block 子元素的显式宽度（如 `<div style="width:100px">` 内无内容时不应算 0，
    // 否则 orphan table-cell / display:table 容器会留在满宽——percentages-grandchildren-
    // quirks-mode-001/002 驱动案：table-cell 784px vs 应 100px）。
    let own_explicit_w = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = inner.max(own_explicit_w);
    inner + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LayoutBox;
    use slotmap::SlotMap;

    /// 构造一个最小可用 LayoutBox（仅设置几何字段）。
    fn make_box(width: f32, height: f32) -> LayoutBox {
        let mut b = LayoutBox::default();
        b.width = width;
        b.height = height;
        b.content_width = width;
        b.content_height = height;
        b
    }

    /// 生成 N 个有效的 NodeId（通过 SlotMap 插入）。
    fn make_node_ids(n: usize) -> Vec<NodeId> {
        let mut map: SlotMap<NodeId, ()> = SlotMap::with_key();
        (0..n).map(|_| map.insert(())).collect()
    }

    /// 构造带 node_id + display 的子盒，并登记到 styles map。
    fn make_styled_child(
        node_id: NodeId,
        display: DisplayValue,
        width: f32,
        height: f32,
        styles: &mut HashMap<NodeId, ComputedStyle>,
    ) -> LayoutBox {
        let mut b = make_box(width, height);
        b.node_id = Some(node_id);
        let mut s = ComputedStyle::default();
        s.display = display;
        styles.insert(node_id, s);
        b
    }

    /// `<html style="display:table">` 包含一个 block `<body>`，body 内有两个
    /// inline-block（200px + 80px）。table 应收缩到 280px 内容宽（+border），
    /// 不应填满容器（780px）。
    #[test]
    fn test_shrink_table_to_inline_block_content() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(4);
        let [html_id, body_id, ib1_id, ib2_id] = [ids[0], ids[1], ids[2], ids[3]];

        // html: display:table, border 10px, 容器宽 780（模拟视口填充）
        let mut table = make_box(800.0, 600.0);
        table.content_width = 780.0;
        table.border_left = 10.0;
        table.border_right = 10.0;
        table.border_top = 10.0;
        table.border_bottom = 10.0;
        table.node_id = Some(html_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        styles.insert(html_id, ts);

        // body: display:Block，宽 780（taffy 块级填充），高 300，y=10（=border_top）
        let mut body = make_styled_child(body_id, DisplayValue::Block, 780.0, 300.0, &mut styles);
        body.y = 10.0;

        // body 内两个 inline-block：200px + 80px（水平排列 = 280px max-content）
        body.children.push(make_styled_child(
            ib1_id,
            DisplayValue::InlineBlock,
            200.0,
            300.0,
            &mut styles,
        ));
        body.children.push(make_styled_child(
            ib2_id,
            DisplayValue::InlineBlock,
            80.0,
            300.0,
            &mut styles,
        ));

        table.children.push(body);

        shrink_table_to_block_content(&mut table, &styles);

        // table 收缩到 280 内容宽 + 20 border = 300（而非 800）
        assert_eq!(
            table.content_width, 280.0,
            "table content width should shrink to inline-block sum (280)"
        );
        assert_eq!(table.width, 300.0, "table width should be 280 content + 20 border");
        // 内容高度 = body 高度 300，总高 = 300 + 20 border = 320
        assert_eq!(table.content_height, 300.0);
        assert_eq!(table.height, 320.0);
        // body 宽度收缩到 280（填满 table 内容盒）
        assert_eq!(table.children[0].width, 280.0);
        assert_eq!(table.children[0].content_width, 280.0);
    }

    /// 显式 width 的 table 应尊重显式宽度（不被 max-content 撑大或缩小）。
    #[test]
    fn test_shrink_table_respects_explicit_width() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(3);
        let [html_id, body_id, ib_id] = [ids[0], ids[1], ids[2]];

        let mut table = make_box(800.0, 600.0);
        table.content_width = 780.0;
        table.node_id = Some(html_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        ts.width = LengthValue::Px(400.0);
        styles.insert(html_id, ts);

        let mut body = make_styled_child(body_id, DisplayValue::Block, 780.0, 100.0, &mut styles);
        body.y = 0.0;
        // body 内一个 50px inline-block（max-content 远小于显式 400）
        body.children.push(make_styled_child(
            ib_id,
            DisplayValue::InlineBlock,
            50.0,
            100.0,
            &mut styles,
        ));
        table.children.push(body);

        shrink_table_to_block_content(&mut table, &styles);

        // 显式 width:400px（content-box 语义）应被尊重
        assert_eq!(table.content_width, 400.0, "explicit width should be respected");
    }

    /// R995 orphan `display:table-cell`（无 table 祖先）经 layout_table → 本函数收缩。
    /// 驱动案 percentages-grandchildren-quirks-mode-001：table-cell height:100px 含一个
    /// width:100px 的 block 子（其子 0 内容）。cell 应收缩到子显式 width 100px，且尊重自身
    /// 显式 height 100px（不被 0 内容子塌缩）。修复前 max_content_width=0 致早退，cell 留 784px。
    #[test]
    fn test_shrink_orphan_table_cell_respects_child_explicit_width_and_own_height() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(2);
        let [cell_id, child_id] = [ids[0], ids[1]];

        // orphan table-cell：taffy 拉伸到 784×100（满宽），height:100px 显式
        let mut table = make_box(784.0, 100.0);
        table.content_width = 784.0;
        table.node_id = Some(cell_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::TableCell;
        ts.height = LengthValue::Px(100.0);
        styles.insert(cell_id, ts);

        // block 子：显式 width:100px，0 内容（height 0）
        let mut child = make_box(100.0, 0.0);
        child.node_id = Some(child_id);
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::Block;
        cs.width = LengthValue::Px(100.0);
        styles.insert(child_id, cs);
        table.children.push(child);

        shrink_table_to_block_content(&mut table, &styles);

        // cell 收缩到子显式 width 100px（非 784 满宽）
        assert_eq!(
            table.width, 100.0,
            "orphan table-cell should shrink to child explicit width"
        );
        assert_eq!(table.content_width, 100.0);
        // cell 自身显式 height 100px 不被 0 内容子塌缩
        assert_eq!(
            table.height, 100.0,
            "orphan table-cell explicit height should be respected"
        );
    }

    /// 没有 block 级子元素时函数为 no-op。
    #[test]
    fn test_shrink_table_noop_without_block_children() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(1);
        let html_id = ids[0];

        let mut table = make_box(800.0, 600.0);
        table.content_width = 780.0;
        table.node_id = Some(html_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        styles.insert(html_id, ts);
        // 仅一个匿名空盒（node_id=None）—— 不构成 block 内容
        table.children.push(make_box(0.0, 0.0));

        let (w_before, h_before) = (table.width, table.height);
        shrink_table_to_block_content(&mut table, &styles);

        assert_eq!(table.width, w_before, "no block children → no-op");
        assert_eq!(table.height, h_before);
    }

    /// 完全为空（无任何子盒）的 display:table 应收缩到 padding+border，而非填满容器。
    /// 对应 WPT css-tables/empty-table-height.html：`<div style="display:table;
    /// padding:155px; border:1px solid black">` 应为 312×312。
    #[test]
    fn test_shrink_empty_table_to_padding_border() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(1);
        let table_id = ids[0];

        // taffy 把 display:table 当 block 填满容器：宽 784（=body 内容宽），高 0（空内容）
        let mut table = make_box(784.0, 0.0);
        table.content_width = 784.0;
        // padding 155 各侧 + border 1 各侧
        table.padding_left = 155.0;
        table.padding_right = 155.0;
        table.padding_top = 155.0;
        table.padding_bottom = 155.0;
        table.border_left = 1.0;
        table.border_right = 1.0;
        table.border_top = 1.0;
        table.border_bottom = 1.0;
        table.node_id = Some(table_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        styles.insert(table_id, ts);
        // 无任何子盒（truly empty）

        shrink_table_to_block_content(&mut table, &styles);

        // max-content=0 → 仅 padding+border = 155*2+1*2 = 312，而非 784
        assert_eq!(table.content_width, 0.0, "empty table content width = 0");
        assert_eq!(table.width, 312.0, "empty table width = padding+border (312)");
        assert_eq!(table.content_height, 0.0);
        assert_eq!(table.height, 312.0, "empty table height = padding+border (312)");
    }

    /// 完全为空但 width 非默认（显式 px / percent）的 display:table 不应被
    /// 本收缩路径改动——显式 width 由 taffy 解析（abspos width:50%→CB、
    /// normal-flow width:100%→CB）。覆盖会破坏 absolute-tables-007/012/016、
    /// subpixel-table-width-001（均为显式宽度）。
    #[test]
    fn test_shrink_empty_table_skips_explicit_width() {
        let mut styles = HashMap::new();
        let ids = make_node_ids(1);
        let table_id = ids[0];

        let mut table = make_box(784.0, 0.0);
        table.content_width = 784.0;
        table.padding_left = 10.0;
        table.padding_right = 10.0;
        table.node_id = Some(table_id);
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        ts.width = LengthValue::Percentage(50.0); // 显式 50%（taffy 已解析）
        styles.insert(table_id, ts);

        let (w_before, h_before) = (table.width, table.height);
        shrink_table_to_block_content(&mut table, &styles);

        // 显式 width → no-op（尊重 taffy 解析）
        assert_eq!(table.width, w_before, "explicit width → no shrink");
        assert_eq!(table.height, h_before);
    }
}
