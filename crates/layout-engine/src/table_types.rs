//! CSS Table 布局 — 数据类型与叶 helper。
//!
//! R832 从 `table.rs` 抽出（2000 行规则）：表数据类型（TableCell / TableRow /
//! TableGrid）+ 自包含叶 helper（display 判定、colspan/rowspan/span 读取、
//! border-spacing/inset 解析、行/单元格盒访问、单元格固有宽度估算、文本长度收集）。
//! 这些函数**不调用** table.rs 的大布局函数（build_grid/compute_column_widths/
//! position_cells 等），依赖单向（table.rs → 本模块），经 table.rs 的
//! `use crate::table_types::*` 调用。纯移动，零行为变化。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue};

use zero_dom::NodeId;

use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// 一个表格单元格的信息。
#[derive(Debug, Clone)]
pub(crate) struct TableCell {
    /// 在行 LayoutBox children 中的索引。
    pub(crate) child_index: usize,
    /// colspan 值（默认 1）。
    pub(crate) colspan: usize,
    /// rowspan 值（默认 1）。
    pub(crate) rowspan: usize,
    /// 单元格跨的列范围 [start, end)。
    pub(crate) col_start: usize,
    pub(crate) col_end: usize,
    /// 嵌套行组中的单元格：指向 table_box.children 中的行组索引。
    /// None 表示单元格在 row_box.children[child_index] 中查找（默认）。
    /// Some(rg_idx) 表示单元格在 table_box.children[rg_idx].children[child_index] 中查找。
    /// 用于孤立行组（table_box 本身是行组）中混合嵌套行组和直接子单元格的匿名行。
    pub(crate) parent_rg_idx: Option<usize>,
}

/// 一个表格行的信息。
#[derive(Debug, Clone)]
pub(crate) struct TableRow {
    /// 在 table LayoutBox children 中的索引。
    /// 当行是直接子 table-row 时，这是 table_box.children 中的索引。
    pub(crate) child_index: usize,
    /// 行所在的行组（tbody/thead/tfoot）在 table LayoutBox children 中的索引。
    /// None 表示行是 table 的直接 table-row 子元素。
    pub(crate) row_group_index: Option<usize>,
    /// 行内的单元格列表。
    pub(crate) cells: Vec<TableCell>,
    /// 是否为匿名行（cells 直接在 row-group 中，无包裹 table-row）。
    pub(crate) is_anonymous: bool,
}

/// 解析后的表格网格结构。
#[derive(Debug)]
pub(crate) struct TableGrid {
    /// 行列表。
    pub(crate) rows: Vec<TableRow>,
    /// 总列数。
    pub(crate) col_count: usize,
    /// 每列是否被 visibility:collapse 折叠。
    /// CSS Tables §4.1：visibility:collapse 的列宽度为 0，不参与布局。
    pub(crate) collapsed_cols: Vec<bool>,
    /// 每行是否被 visibility:collapse 折叠。
    /// CSS Tables §4.1：visibility:collapse 的行高度为 0，不参与布局，亦不贡献
    /// 与相邻行之间的 border-spacing（与 `collapsed_cols` 对称）。
    pub(crate) collapsed_rows: Vec<bool>,
}
/// 获取 LayoutBox 对应的 display 值。
pub(crate) fn get_display(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> Option<DisplayValue> {
    box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| s.display.clone())
}

/// 判断 display 值是否为某种 table-row 类型。
pub(crate) fn is_table_row(display: &DisplayValue) -> bool {
    matches!(display, DisplayValue::TableRow)
}

/// 判断 display 值是否为 table-cell。
pub(crate) fn is_table_cell(display: &DisplayValue) -> bool {
    matches!(display, DisplayValue::TableCell)
}

/// 判断 display 值是否为行组（tbody/thead/tfoot）。
pub(crate) fn is_row_group(display: &DisplayValue) -> bool {
    matches!(
        display,
        DisplayValue::TableRowGroup | DisplayValue::TableHeaderGroup | DisplayValue::TableFooterGroup
    )
}

/// 判断 display 值是否为 table 内部元素（需要匿名 table 包装）。
pub(crate) fn is_table_internal(display: &DisplayValue) -> bool {
    matches!(
        display,
        DisplayValue::TableRowGroup
            | DisplayValue::TableHeaderGroup
            | DisplayValue::TableFooterGroup
            | DisplayValue::TableRow
            | DisplayValue::TableCell
            | DisplayValue::TableColumn
            | DisplayValue::TableColumnGroup
            | DisplayValue::TableCaption
    )
}

/// 行组的排序优先级。
///
/// CSS 规范要求 thead 在 tbody 之前，tbody 在 tfoot 之前，
/// 无论 DOM 顺序如何。
pub(crate) fn row_group_sort_priority(display: &DisplayValue) -> u8 {
    match display {
        DisplayValue::TableHeaderGroup => 0,
        DisplayValue::TableRowGroup => 1,
        DisplayValue::TableFooterGroup => 2,
        _ => 3,
    }
}

/// 从 DOM 中读取元素的 colspan 属性值。
pub(crate) fn get_colspan(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "colspan")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从 DOM 属性中读取 rowspan 值（默认 1）。
pub(crate) fn get_rowspan(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "rowspan")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从 ComputedStyle 中读取 border-spacing 值。
pub(crate) fn get_border_spacing(style: &ComputedStyle) -> (f32, f32) {
    (style.border_spacing.horizontal, style.border_spacing.vertical)
}

/// 从 ComputedStyle 中读取 position: relative 的 inset 偏移量。
///
/// `horizontal` 为 true 时读取 left，否则读取 top。
pub(crate) fn resolve_length_inset(
    box_node: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    horizontal: bool,
) -> f32 {
    use zero_css_parser::values::LengthValue;
    let Some(node_id) = box_node.node_id else {
        return 0.0;
    };
    let Some(style) = styles.get(&node_id) else {
        return 0.0;
    };
    let value = if horizontal { &style.left } else { &style.top };
    match value {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    }
}
/// 从 DOM 中读取元素的 span 属性值（用于 col/colgroup）。
pub(crate) fn get_span(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "span")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从一个 table-row 子元素构建 TableRow。
pub(crate) fn build_row(child_idx: usize, row_box: &LayoutBox, doc: &zero_dom::Document) -> TableRow {
    let mut cells = Vec::new();
    let mut col_cursor = 0usize;

    for (cell_idx, cell_child) in row_box.children.iter().enumerate() {
        // CSS 2.1 §9.7 + §9.5：floated / 绝对定位子元素脱离正常流，不参与 table grid。
        // 典型场景 float-applies-to-007：`display:table-cell` + `float:right` 经 §9.7 块化为
        // Block+float，此时它不再是 table cell，而应作为浮动块布局（脱离 table-row 流）。
        // 旧实现把所有 table-row 子元素无条件当 cell 收集，致块化后的浮动 cell 仍被当 cell
        // 撑满行宽（ZW 784px 应 96px 浮右）。此处跳过 out-of-flow 子，让它走浮动布局路径。
        if !matches!(cell_child.float, FloatValue::None) || cell_child.is_absolute || cell_child.is_fixed {
            continue;
        }
        let colspan = get_colspan(cell_child, doc);
        let rowspan = get_rowspan(cell_child, doc);
        let col_start = col_cursor;
        let col_end = col_start + colspan;
        cells.push(TableCell {
            child_index: cell_idx,
            colspan,
            rowspan,
            col_start,
            col_end,
            parent_rg_idx: None,
        });
        col_cursor = col_end;
    }

    TableRow {
        child_index: child_idx,
        row_group_index: None, // 由调用方设置
        cells,
        is_anonymous: false,
    }
}
/// 获取行盒 — 处理直接 table-row、row-group 内的行和匿名行三种情况。
///
/// 当 `row.row_group_index` 为 Some 时，行在 row-group 内。
/// 当 `row.is_anonymous` 为 true 时，行是匿名行，行盒为 row-group 本身。
/// 当为 None 时，行是 table 的直接 children[row.child_index]。
pub(crate) fn get_row_box<'a>(table_box: &'a LayoutBox, row: &TableRow) -> Option<&'a LayoutBox> {
    match row.row_group_index {
        Some(rg_idx) => {
            let row_group = table_box.children.get(rg_idx)?;
            if row.is_anonymous {
                // 匿名行：cells 是 row-group 的直接子元素，row-group 即为行盒
                Some(row_group)
            } else {
                row_group.children.get(row.child_index)
            }
        }
        None => {
            if row.is_anonymous {
                // 孤立匿名行：table_box 本身是行组，行盒就是 table_box
                Some(table_box)
            } else {
                // 直接 table-row：table_box.children[row.child_index]
                table_box.children.get(row.child_index)
            }
        }
    }
}

/// 获取单元格盒。
/// 获取单元格盒（不可变引用）。
///
/// 根据 cell.parent_rg_idx 决定查找路径：
/// - None: 直接在 row_box.children 中查找
/// - Some(rg_idx): 在 row_box.children[rg_idx].children 中查找（嵌套行组场景）
pub(crate) fn get_cell_box<'a>(row_box: &'a LayoutBox, cell: &TableCell) -> Option<&'a LayoutBox> {
    if let Some(rg_idx) = cell.parent_rg_idx {
        row_box
            .children
            .get(rg_idx)
            .and_then(|rg| rg.children.get(cell.child_index))
    } else {
        row_box.children.get(cell.child_index)
    }
}

/// 估算单元格的固有内容宽度。
///
/// 当 CSS width:0 被应用时，taffy 会将单元格布局为 0 宽度。
/// 但 CSS 表格规范要求 width:0 解析为 min-content 宽度。
/// 计算单元格的最小内容宽度。
///
/// 策略：
/// 1. 检查子元素是否有显式 CSS width → 使用这些宽度之和
/// 2. 如果 cell_box.width 接近 0（taffy 将子元素也约束为 0），
///    从 DOM 文本内容和字体大小估算 min-content 宽度
/// 3. 否则用字体大小估算单字符宽度作为最小宽度
pub(crate) fn compute_cell_intrinsic_width(
    cell_box: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> f32 {
    let padding = cell_box.padding_left + cell_box.padding_right;
    let is_zero_width = cell_box.width < 2.0;

    // 尝试从子元素计算内容宽度
    let mut content_width = 0.0f32;
    let mut has_explicit_child = false;

    for child in &cell_box.children {
        // 检查子元素是否有显式 CSS width
        let child_has_explicit_width = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                use zero_css_parser::values::LengthValue;
                !matches!(s.width, LengthValue::Auto)
            })
            .unwrap_or(false);

        if child_has_explicit_width {
            // 有显式 width 的子元素：使用其 outer width
            content_width = content_width.max(child.width + child.margin_left + child.margin_right);
            has_explicit_child = true;
        } else if child.width > 0.0 && (!is_zero_width && child.width < cell_box.width * 0.95) {
            // 非 0 宽度单元格：子元素宽度远小于 cell 宽度时使用
            content_width = content_width.max(child.width + child.margin_left + child.margin_right);
            has_explicit_child = true;
        }
    }

    // R1001：cell 的**直接匿名 inline 文本**（cell 的直接文本节点子元素）。
    // box_content_max_width 仅测叶盒文本 + block 子递归；cell 的直接文本（非叶、与 block 子
    // 混合生成匿名 block）被漏测，致含直接文本的 cell 塌缩（table-cell-overflow-explicit-height
    // twin：tall div block 子 + "Can you see this text?" 直接文本，cell 测 8px 应 ~211px）。
    // 仅测**直接**文本节点（非 text_content 全后代）——多 block cell 的文本在 block 后代内，
    // cell 直接文本=0，避免过计（margin-collapse-101 安全）。
    let direct_text_w = cell_direct_text_width(cell_box, styles, doc);

    if has_explicit_child && content_width > 0.0 {
        // R2050：返回 cell 的 border-box 宽度（content + padding + border），与下方
        // box_content_max_width 路径（line 143 返回 inner + padding + border）语义一致。
        // 旧实现漏掉 border，致含显式宽子元素（如 `<td><div style="width:50px">`）的
        // cell 列宽 = content+padding（55）而非 border-box（59），separated 表整体偏窄、
        // 行背景右侧短缺（table-backgrounds-bs-row-001：aqua 行 bg 宽 287 vs ref 303）。
        return content_width.max(direct_text_w) + padding + cell_box.border_left + cell_box.border_right;
    }

    // 当 cell_box.width 接近 0 时，taffy 将所有子元素也约束为 0，
    // 无法从 layout 结果获取真实内容宽度。
    // 从 DOM 文本内容估算 min-content 宽度。
    let (font_size, is_ahem) = cell_box
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| {
            use zero_css_parser::values::LengthValue;
            let fs = match &s.font_size {
                LengthValue::Px(v) => *v as f32,
                LengthValue::Em(v) => *v as f32,
                LengthValue::Rem(v) => *v as f32,
                _ => 16.0,
            };
            let ahem = s.font_family.contains(&"Ahem".to_string());
            (fs, ahem)
        })
        .unwrap_or((16.0, false));

    let char_width = if is_ahem { font_size } else { font_size * 0.6 };

    // 收集单元格内的文本内容长度
    let text_len = collect_text_length(cell_box, doc);
    if text_len > 0 {
        // R702/R679：优先用 intrinsic max-content（DOM-based，不依赖 layout 结果）。
        // collect_text_length 把 block 子之间的空白/换行也计入字符数，致 char_count
        // 估算严重过宽（如 margin-collapse-101：31 字符含大量块间空白 → 930px 列，
        // table 1446px 溢出 viewport；应 shrink-to-fit 到内容 max-content）。
        // box_content_max_width 返回 border-box，即 cell 对列的宽度贡献。
        // R1001：与 cell 直接文本取 max（直接匿名 inline 内容）。
        let intrinsic = crate::intrinsic_sizing::box_content_max_width(cell_box, doc, styles);
        let result = intrinsic.max(direct_text_w);
        if result > 0.0 {
            return result;
        }
        return char_width * text_len as f32 + padding;
    }

    // R1153：无直接文本也无直接显式宽子元素时，用递归 max-content 捕获**嵌套**显式宽后代。
    // compute_cell_intrinsic_width 的直接子元素循环只看一层，会漏测 td > div > div{width:100px}
    // 这类嵌套结构（c5503-mrgn-b-000 td.control：直接子 .teal/.aqua 无显式宽，但孙 .blank
    // 有 width:100px → control 列应 ~100px，旧实现回落 char_width+padding=9.6 致表塌缩）。
    // box_content_max_width 递归 block 子 + 叶盒显式宽，返回 border-box（含 cell padding+border）。
    // ★ gated：仅当 cell 直接子元素**全为 block 级**时采用。含 inline 直接子元素的 cell
    // 走此回退路径时，inline 子（如 span 含 block canvas）的 width 不可靠（R109 inline-
    // containing-block 纠缠），递归会过测致 percent-height-replaced-in-percent-cell-004
    // 表爆炸（3.38→88%）。全 block 子（c5503 .teal/.aqua div）递归可靠。
    let all_block_children = cell_box.children.iter().all(|c| {
        if c.is_absolute || c.is_fixed {
            return true;
        }
        c.node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                !matches!(
                    s.display,
                    DisplayValue::Inline
                        | DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
            .unwrap_or(true)
    });
    if all_block_children {
        let recursed = crate::intrinsic_sizing::box_content_max_width(cell_box, doc, styles);
        if recursed > char_width + padding {
            return recursed;
        }
    }

    char_width + padding
}

/// R1001：测量 cell 的**直接文本节点**子元素的 max-content 宽度（cell 的匿名 inline 内容）。
///
/// 仅遍历 cell 的 DOM 直接子节点中的文本节点（非全后代），用 cell 的 font 度量。
/// 用于 `compute_cell_intrinsic_width` 补测 cell 直接文本（box_content_max_width 漏测的非叶
/// 直接文本）。返回 0 = 无直接文本。
fn cell_direct_text_width(
    cell_box: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> f32 {
    use zero_dom::NodeKind;
    let Some(id) = cell_box.node_id else { return 0.0 };
    let Some(style) = styles.get(&id) else { return 0.0 };
    let text_children: Vec<NodeId> = doc
        .child_nodes(id)
        .into_iter()
        .filter(|&cid| doc.get(cid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
        .collect();
    if text_children.is_empty() {
        return 0.0;
    }
    crate::intrinsic_sizing::fragment_inline_max_width(style, &text_children, doc)
}

/// 递归收集 LayoutBox 子树中的文本字符数。
/// 使用 DOM text_content() 方法获取元素的完整文本内容。
/// 注意：使用 .chars().count() 计算字符数而非 .len()（字节数），
/// 因为多字节 Unicode 字符（如 CJK）的字节数不等于字符数。
pub(crate) fn collect_text_length(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    let mut len = 0;
    if let Some(node_id) = box_node.node_id {
        if let Some(text) = doc.text_content(node_id) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                len += trimmed.chars().count();
            }
        }
    }
    len
}

/// R1146：计算 vertical 表每列的 min-content（最长不可断 word 的宽度）。
///
/// CSS §17.5.2.2 auto table layout：列宽下限 = min-content。word 按普通空白断
/// （space/tab/newline），nbsp（U+00A0）是非断空白，留在 word 内参与宽度。每 word 宽 =
/// char 数（含 nbsp，Ahem 下每 glyph = fs；变宽字体近似 0.6×fs）× char_width。colspan>1
/// 的 cell 不贡献 min-content floor（其最长 word 应由多列合计承载，保守跳过避免过估每列）。
pub(crate) fn compute_col_min_content(
    table_box: &LayoutBox,
    grid: &TableGrid,
    n_cols: usize,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> Vec<f32> {
    let mut min_content = vec![0.0f32; n_cols];
    for row in &grid.rows {
        let Some(rb) = get_row_box(table_box, row) else {
            continue;
        };
        for cell in &row.cells {
            if cell.colspan != 1 {
                continue;
            }
            let cb = if let Some(rg_idx) = cell.parent_rg_idx {
                rb.children.get(rg_idx).and_then(|rg| rg.children.get(cell.child_index))
            } else {
                rb.children.get(cell.child_index)
            };
            let Some(cb) = cb else { continue };
            let Some(nid) = cb.node_id else { continue };
            let Some(text) = doc.text_content(nid) else { continue };
            let (fs, ahem) = styles
                .get(&nid)
                .map(|s| {
                    let fs = match &s.font_size {
                        zero_css_parser::values::LengthValue::Px(v) => *v as f32,
                        _ => 16.0,
                    };
                    (fs, s.font_family.iter().any(|f| f.contains("Ahem")))
                })
                .unwrap_or((16.0, false));
            let cw = if ahem { fs } else { fs * 0.6 };
            if cw <= 0.0 {
                continue;
            }
            // word = 按 break-opportunity 空白分割（nbsp U+00A0 留 word 内）。
            let longest = text
                .split(|c: char| c.is_whitespace() && c != '\u{00A0}')
                .map(|w| w.chars().count() as f32 * cw)
                .fold(0.0f32, f32::max);
            if cell.col_start < n_cols {
                min_content[cell.col_start] = min_content[cell.col_start].max(longest);
            }
        }
    }
    min_content
}

/// α-4b-1（RFC `vertical-mode-table-rl-transpose-rfc.md` §4.1）：对 `writing-mode:
/// vertical-rl/lr` 的表，把行/cell 定位从 horizontal-tb **转置** 到 vertical——
/// 行沿 x（vertical-rl 右到左 / vertical-lr 左到右），cell 沿 y 顶到底。
///
/// `col_widths`（逻辑列宽，WM-agnostic）映射为 cell 的 y 高度；行的 x 宽 = 该行
/// cell 内容宽的最大值。WM gate：仅 vertical-rl/lr 触发，horizontal-tb 走原
/// `position_cells`（字节一致零回归）。
///
/// **α-4b-1 范围**：简单表（无 colspan/rowspan）+ border-spacing 轴互换。
/// 有 colspan/rowspan 时回退到 `position_cells`（旧行为），留给 α-4b-2。
/// row-extras（table height 展开）/ vertical-align / caption-side 在转置轴的
/// 处理留给 α-4b-4。
/// R1131 slice 3：vrl_cap_scale 触发时增长 cell 的 block extent（x 宽）以容纳 IFC wrap 列数。
///
/// vrl 表 inline extent 被 cap（vrl_cap_scale）→ 文本强制 wrap 成 N 列沿 block 轴（x）排。
/// 旧 cell.width（taffy 原值，~单字符宽）不足以容纳 N 列，wrap 列溢出 cell 盒 →
/// row-progression-vrl 簇残余 11-13%。本函数按 N = ceil(文本像素高 / cell_h_scaled)
/// 增长 cell.width = N × fs。
///
/// **post-R1100 重试**：R1116 试 area-conservation 增长 0-flip，因彼时 vertical IFC
/// container_width=0（R1050）致文本不 wrap；R1100（α-1）修 IFC container_width WM-aware
/// 后文本能 wrap，故 cell.width 增长方有意义。
///
/// **gate**：rowspan>1 跳过（ZW rowspan partial，避 vrl-006 回归，R1110 先例）；
/// vrl_cap_scale 仅 vrl 触发故天然 vrl-only（避 vlr Path A/B 发散，R1119）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn grow_vrl_cell_block_extent(
    cb: &LayoutBox,
    cell: &TableCell,
    final_col_widths: &[f32],
    cap_fired: bool,
    spacing_x: f32,
    grid: &TableGrid,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> f32 {
    // cap 未触发（表 inline extent 未超 target）→ cell.width 保持 taffy 原值。
    if !cap_fired {
        return cb.width;
    }
    // rowspan gate
    if cell.rowspan > 1 {
        return cb.width;
    }
    let fs = cb
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| match s.font_size {
            zero_css_parser::values::LengthValue::Px(v) => v as f32,
            _ => 16.0,
        })
        .unwrap_or(16.0);
    if fs <= 0.0 {
        return cb.width;
    }
    // cell_h_scaled：cell 的 inline extent（y）= Σ final_col_widths[col] + colspan gap。
    // R1146：final_col_widths 已含 min-content floor（CSS auto-layout 分布），不再 ×scale。
    let mut cell_h_scaled = 0.0f32;
    let mut spanned_non_collapsed = 0usize;
    for col in cell.col_start..cell.col_end {
        if col < final_col_widths.len() {
            cell_h_scaled += final_col_widths[col];
            if !grid.collapsed_cols.get(col).copied().unwrap_or(false) {
                spanned_non_collapsed += 1;
            }
        }
    }
    if spanned_non_collapsed > 1 {
        cell_h_scaled += (spanned_non_collapsed - 1) as f32 * spacing_x;
    }
    if cell_h_scaled <= 0.0 {
        return cb.width;
    }
    // 列数 N 按 **word-based 贪心 packing** 计（非连续 char 数）。IFC 实际换行按 word
    //（break opportunity = 空白除 nbsp U+00A0）；R1131 原 char 公式低估列数（如 4 word
    // 各 1 列，char 公式给 3 列）→ cell 过窄 → wrap 列溢出（row-progression 残余）。
    // 每 word 高 = char 数（含 nbsp，nbsp 渲染）× fs；贪心 pack 到 cell_h_scaled 满即换列。
    let n = cb
        .node_id
        .and_then(|id| doc.text_content(id))
        .map(|t| {
            // word = 按 break-opportunity 空白分割（nbsp U+00A0 不分割，留在 word 内）。
            let words = t.split(|c: char| c.is_whitespace() && c != '\u{00A0}');
            let mut cols = 1usize;
            let mut col_h = 0.0f32;
            for word in words {
                let wc = word.chars().count() as f32;
                if wc == 0.0 {
                    continue;
                }
                let word_h = wc * fs;
                if col_h + word_h > cell_h_scaled && col_h > 0.0 {
                    cols += 1;
                    col_h = word_h;
                } else {
                    col_h += word_h;
                }
            }
            cols.max(1) as f32
        })
        .unwrap_or(1.0);
    (n * fs).max(cb.width)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// build_row 须跳过 out-of-flow（floated / 绝对定位）子元素——它们经 §9.7 块化后
    /// 脱离 table-row 流，不应被当 cell 收集。float-applies-to-007 回归保护。
    #[test]
    fn build_row_skips_floated_children() {
        let doc = zero_dom::Document::new();
        // 行内 3 个子：正常 cell / floated cell / 绝对定位 cell
        let row_box = LayoutBox {
            children: vec![
                LayoutBox { ..LayoutBox::default() },
                LayoutBox {
                    float: FloatValue::Right,
                    ..LayoutBox::default()
                },
                LayoutBox {
                    is_absolute: true,
                    ..LayoutBox::default()
                },
            ],
            ..LayoutBox::default()
        };
        let row = build_row(0, &row_box, &doc);
        // 仅第 1 个（in-flow）子被收集为 cell；floated 与 abspos 被跳过
        assert_eq!(row.cells.len(), 1);
        assert_eq!(row.cells[0].child_index, 0);
        assert_eq!(row.cells[0].colspan, 1);
    }

    /// 正常 in-flow 子元素（含 is_fixed=false、float=None）应全部收集为 cell。
    #[test]
    fn build_row_collects_all_inflow_children() {
        let doc = zero_dom::Document::new();
        let row_box = LayoutBox {
            children: vec![
                LayoutBox { ..LayoutBox::default() },
                LayoutBox { ..LayoutBox::default() },
                LayoutBox { ..LayoutBox::default() },
            ],
            ..LayoutBox::default()
        };
        let row = build_row(0, &row_box, &doc);
        assert_eq!(row.cells.len(), 3);
        // col_end 累计：每个 colspan=1 → 1,2,3
        assert_eq!(row.cells[2].col_end, 3);
    }
}
