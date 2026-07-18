//! table.rs 表格布局回归测试（从 table.rs 抽出，保持 2000 行约束）。

use super::*;
// R964：count_col_elements 抽到 table_grid 模块。
// R1718：collect_col_widths 同模块。
use crate::table_grid::{collect_col_widths, count_col_elements};
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, VisibilityValue};
use zero_dom::Document;
use zero_style_system::ComputedStyle;

/// CSS §17.2.1：display:table 的连续直接 table-cell 子元素应合并到同一个
/// 匿名 table-row（水平多列），而非每个 cell 各占一行。回归 subpixel-table-cell-width-001。
#[test]
fn test_build_grid_consecutive_direct_cells_share_one_row() {
    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell1_id = doc.create_element("div");
    let cell2_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    styles.insert(cell1_id, cs.clone());
    styles.insert(cell2_id, cs);

    let table_box = LayoutBox {
        node_id: Some(table_id),
        children: vec![
            LayoutBox {
                node_id: Some(cell1_id),
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(cell2_id),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);

    // 2 个直接 cell → 1 个匿名行（2 列），而非 2 行各 1 列
    assert_eq!(
        grid.rows.len(),
        1,
        "consecutive direct cells should share one anonymous row"
    );
    assert_eq!(grid.col_count, 2, "two cells should produce 2 columns");
    let row = &grid.rows[0];
    assert!(row.is_anonymous, "the row wrapping direct cells should be anonymous");
    assert_eq!(row.cells.len(), 2);
    assert_eq!(row.cells[0].col_start, 0);
    assert_eq!(row.cells[0].col_end, 1);
    assert_eq!(row.cells[1].col_start, 1);
    assert_eq!(row.cells[1].col_end, 2);
    // cell.child_index 指向 table_box.children（配合 is_anonymous 导航）
    assert_eq!(row.cells[0].child_index, 0);
    assert_eq!(row.cells[1].child_index, 1);
}

/// CSS Tables §4.1：table-row 上 `visibility:collapse` 的行应被标记为折叠
/// （高度为 0、不贡献 border-spacing）。镜像列折叠 `detect_collapsed_columns`。
/// 匿名行（无 table-row 元素）不可折叠。
#[test]
fn test_build_grid_detects_collapsed_rows() {
    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let row1_id = doc.create_element("div");
    let row2_id = doc.create_element("div");
    let cell1_id = doc.create_element("div");
    let cell2_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);

    let mut row1_style = ComputedStyle::default();
    row1_style.display = DisplayValue::TableRow;
    styles.insert(row1_id, row1_style);

    let mut row2_style = ComputedStyle::default();
    row2_style.display = DisplayValue::TableRow;
    row2_style.visibility = VisibilityValue::Collapse; // 第 2 行折叠
    styles.insert(row2_id, row2_style);

    let cell_style = {
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::TableCell;
        cs
    };
    styles.insert(cell1_id, cell_style.clone());
    styles.insert(cell2_id, cell_style);

    let table_box = LayoutBox {
        node_id: Some(table_id),
        children: vec![
            LayoutBox {
                node_id: Some(row1_id),
                children: vec![LayoutBox {
                    node_id: Some(cell1_id),
                    ..Default::default()
                }],
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(row2_id),
                children: vec![LayoutBox {
                    node_id: Some(cell2_id),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);

    assert_eq!(grid.rows.len(), 2, "two table-row children → two rows");
    assert!(
        !grid.rows[0].is_anonymous && !grid.rows[1].is_anonymous,
        "table-row children produce non-anonymous rows"
    );
    assert_eq!(
        grid.collapsed_rows,
        vec![false, true],
        "only the 2nd row (visibility:collapse) should be collapsed"
    );
}

/// CSS Tables §4：`<col>`/`<colgroup>` 元素定义网格列。
/// count_col_elements 应统计 colgroup 内 col 的 span 之和，
/// 以及无内部 col 时 colgroup 自身的 span。
#[test]
fn test_count_col_elements() {
    let mut doc = Document::new();
    let mut styles = HashMap::new();

    // 场景 1：colgroup（作为 table 子元素）内含 4 个 col（各 span=1）
    let mut cg1 = make_box(&mut doc, &mut styles, DisplayValue::TableColumnGroup);
    let c1 = make_box(&mut doc, &mut styles, DisplayValue::TableColumn);
    let c2 = make_box(&mut doc, &mut styles, DisplayValue::TableColumn);
    let c3 = make_box(&mut doc, &mut styles, DisplayValue::TableColumn);
    let c4 = make_box(&mut doc, &mut styles, DisplayValue::TableColumn);
    cg1.children = vec![c1, c2, c3, c4];
    let table1 = LayoutBox {
        children: vec![cg1],
        ..Default::default()
    };
    assert_eq!(
        count_col_elements(&table1, &styles, &doc),
        4,
        "colgroup with 4 inner cols"
    );

    // 场景 2：直接 col 子元素（span=2）
    let d1 = make_box(&mut doc, &mut styles, DisplayValue::TableColumn);
    doc.set_attribute(d1.node_id.unwrap(), "span", "2");
    let table2 = LayoutBox {
        children: vec![d1.clone()],
        ..Default::default()
    };
    assert_eq!(count_col_elements(&table2, &styles, &doc), 2, "direct col with span=2");

    // 场景 3：colgroup 无内部 col，自身 span=3
    let cg3 = make_box(&mut doc, &mut styles, DisplayValue::TableColumnGroup);
    doc.set_attribute(cg3.node_id.unwrap(), "span", "3");
    let table3 = LayoutBox {
        children: vec![cg3.clone()],
        ..Default::default()
    };
    assert_eq!(
        count_col_elements(&table3, &styles, &doc),
        3,
        "colgroup span=3, no inner col"
    );

    // 场景 4：无 col 元素（仅 cell）→ 0
    let cell = make_box(&mut doc, &mut styles, DisplayValue::TableCell);
    let table4 = LayoutBox {
        children: vec![cell],
        ..Default::default()
    };
    assert_eq!(count_col_elements(&table4, &styles, &doc), 0, "no col elements");
}

/// 创建一个 display 为 d 的元素，注册 computed style，返回对应 LayoutBox。
fn make_box(doc: &mut Document, styles: &mut HashMap<NodeId, ComputedStyle>, d: DisplayValue) -> LayoutBox {
    let id = doc.create_element("div");
    let mut s = ComputedStyle::default();
    s.display = d;
    styles.insert(id, s);
    LayoutBox {
        node_id: Some(id),
        ..Default::default()
    }
}

/// R289：表格单元格 vertical-align:bottom 应把内容压到 **content 区** 底部，
/// 不溢出到 border 区。回归 background-043（img 偏低 6px = border 之和）。
///
/// 旧实现 `available = cell_box.height - content_height` 用了 border-box 高，
/// 子元素 y 又相对 content box 度量，导致 valign:bottom/middle 把内容推出 content 区
/// （多算 border+padding）。修复后用 `cell_box.content_height`。
#[test]
fn test_table_cell_valign_bottom_stays_in_content_box() {
    use zero_css_parser::values::{LengthValue, VerticalAlignValue};
    use zero_style_system::property::types::BorderStyleValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    // table: height 206px（R89 行高分配会把单元格撑到 ~206 border-box）
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    ts.height = LengthValue::Px(206.0);
    styles.insert(table_id, ts);
    // cell: border 3px 四边 + vertical-align:bottom
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.border_top_width = LengthValue::Px(3.0);
    cs.border_right_width = LengthValue::Px(3.0);
    cs.border_bottom_width = LengthValue::Px(3.0);
    cs.border_left_width = LengthValue::Px(3.0);
    cs.border_top_style = BorderStyleValue::Solid;
    cs.border_right_style = BorderStyleValue::Solid;
    cs.border_bottom_style = BorderStyleValue::Solid;
    cs.border_left_style = BorderStyleValue::Solid;
    cs.vertical_align = VerticalAlignValue::Bottom;
    styles.insert(cell_id, cs);

    // 单元格内一个 15px 高的内容（模拟 background-043 的 <img height=15>）
    let content_box = LayoutBox {
        height: 15.0,
        width: 100.0,
        ..Default::default()
    };
    // cell LayoutBox：border 由 extract_layout 从 taffy 回读，测试直接设
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        border_top: 3.0,
        border_right: 3.0,
        border_bottom: 3.0,
        border_left: 3.0,
        children: vec![content_box],
        ..Default::default()
    };
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 200.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    position_cells(&mut table_box, &grid, &[200.0], 0.0, 0.0, &styles);

    let cell = &table_box.children[0];
    let content = &cell.children[0];
    // 内容底 = content.y + content.height，应 ≤ cell.content_height（不溢出 content 区到 border）
    let content_bottom = content.y + content.height;
    assert!(
        content_bottom <= cell.content_height + 0.5,
        "valign:bottom content bottom ({content_bottom}) should stay within content_height ({}) \
         (border-box height={}); old bug pushed it ~border past content area",
        cell.content_height,
        cell.height
    );
    // 旧 bug 会把 content_bottom 推到 cell.height（border-box）——明确断言未发生
    assert!(
        content_bottom < cell.height,
        "content bottom ({content_bottom}) must not reach border-box height ({}) \
         (would mean valign used border-box instead of content-box)",
        cell.height
    );
}

/// R1717：text-only table-cell（无 child box，文本经 IFC 在 paint 期渲染）的 vertical-align
/// 内容偏移写入 `cell.valign_offset`，供 paint_text 偏移文本起点。
///
/// 单元格预-extra 高度（text 块高度）= post-extra height − row_extra。本测试构造 cell
/// 初始 height=18（模拟文本块高度）+ table height=120 → row_extra≈102，valign:middle
/// → valign_offset ≈ (content_h − 18)/2。valign:top 不触发（gate 排除）。
#[test]
fn test_table_cell_text_valign_offset_for_text_only_cell() {
    use zero_css_parser::values::{LengthValue, VerticalAlignValue};

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    ts.height = LengthValue::Px(120.0);
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.vertical_align = VerticalAlignValue::Middle;
    styles.insert(cell_id, cs);

    // text-only cell：无 children（文本经 IFC paint 期渲染），初始 height=18 = 文本块高度
    //（content_row_heights 据此算 row_extra；position_cells 随后把 height 撑到 row_height）。
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        height: 18.0,
        ..Default::default()
    };
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 200.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    position_cells(&mut table_box, &grid, &[200.0], 0.0, 0.0, &styles);

    let cell = &table_box.children[0];
    // valign:middle → 文本居中，offset ≈ (content_height − text_h)/2 = (120−18)/2 = 51
    assert!(
        cell.valign_offset > 40.0 && cell.valign_offset < 60.0,
        "valign:middle text-only cell valign_offset should be ~51 (centered), got {}",
        cell.valign_offset
    );

    // valign:top → gate 排除，valign_offset 应保持 0（top 不位移文本）
    styles.get_mut(&cell_id).unwrap().vertical_align = VerticalAlignValue::Top;
    let cell_box2 = LayoutBox {
        node_id: Some(cell_id),
        height: 18.0,
        ..Default::default()
    };
    let mut table_box2 = LayoutBox {
        node_id: Some(table_id),
        content_width: 200.0,
        children: vec![cell_box2],
        ..Default::default()
    };
    let grid2 = build_grid(&table_box2, &doc, &styles);
    position_cells(&mut table_box2, &grid2, &[200.0], 0.0, 0.0, &styles);
    assert!(
        table_box2.children[0].valign_offset == 0.0,
        "valign:top should not set valign_offset, got {}",
        table_box2.children[0].valign_offset
    );
}

/// R1718：`<colgroup><col width="40%"><col width="30%"><col width="30%">` → 列宽按 %
/// 解析（available_width=300 → 120/90/90）。colgroup 含 col 子时按各 col 的 width 映射。
/// col 无 box，须在 grid 直接读 DOM 属性（compute_column_widths 此前忽略 col width）。
#[test]
fn test_collect_col_widths_resolves_percent_and_px() {
    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cg_id = doc.create_element("div");
    let col0 = doc.create_element("div");
    let col1 = doc.create_element("div"); // 无 width 属性 → None
    let col2 = doc.create_element("div");
    let _ = doc.append_child(root, table_id);
    let _ = doc.append_child(table_id, cg_id);
    let _ = doc.append_child(cg_id, col0);
    let _ = doc.append_child(cg_id, col1);
    let _ = doc.append_child(cg_id, col2);
    doc.set_attribute(col0, "width", "40%");
    doc.set_attribute(col2, "width", "60"); // px（无 %）

    let mut styles = HashMap::new();
    let mk = |d: DisplayValue| {
        let mut s = ComputedStyle::default();
        s.display = d;
        s
    };
    styles.insert(table_id, mk(DisplayValue::Table));
    styles.insert(cg_id, mk(DisplayValue::TableColumnGroup));
    styles.insert(col0, mk(DisplayValue::TableColumn));
    styles.insert(col1, mk(DisplayValue::TableColumn));
    styles.insert(col2, mk(DisplayValue::TableColumn));

    let col = |nid| LayoutBox {
        node_id: Some(nid),
        ..Default::default()
    };
    let colgroup = LayoutBox {
        node_id: Some(cg_id),
        children: vec![col(col0), col(col1), col(col2)],
        ..Default::default()
    };
    let table_box = LayoutBox {
        node_id: Some(table_id),
        children: vec![colgroup],
        ..Default::default()
    };

    let widths = collect_col_widths(&table_box, 3, &styles, &doc, 300.0);
    // 40% of 300 = 120，col1 无 width → None，"60" px = 60。
    assert_eq!(widths, vec![Some(120.0), None, Some(60.0)]);
}

/// R769：table-cell 建立 BFC（CSS §9.4.1），其首子 margin-top 不应向上穿透单元格
/// 而丢失——BFC 的 margin 不与子元素折叠（§8.3.1）。但 taffy 把单元格按普通 Block
/// 布局，把首子 margin-top 折叠上提到 `cell.margin_top`；自定义表格布局忽略单元格
/// margin → 内容从 content-box 顶（y=0）开始，等量空白留底部（cell_content_height
/// 已按 child.margin_top 计入高度）。`position_cells` 把内容子树下移 `cell.margin_top`，
/// 把顶部留白从底部移到顶部，对齐 Chromium BFC margin 包含语义。
/// 回归 margin-collapse-110/111（chromium Oracle 14.80%→4.75%）。
#[test]
fn test_table_cell_bfc_first_child_margin_top_preserved() {
    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    styles.insert(cell_id, cs);

    // 单元格内一个 30px 高的内容块，自身 margin-top/bottom = 50px（模拟
    // margin-collapse-110 的 `div { margin: 1em 0 }`）。taffy 折叠后首子 y=0。
    let content_box = LayoutBox {
        height: 30.0,
        width: 100.0,
        margin_top: 50.0,
        margin_bottom: 50.0,
        ..Default::default()
    };
    // cell.margin_top = 50px：taffy 把首子 margin-top 折叠上提到单元格（BFC
    // 单元格应包含此 margin 作顶部留白，而非丢失）。cell_content_height 会按
    // child.margin_top 计入 50 → 总高 130，故下移后仍在 content 区内。
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        margin_top: 50.0,
        content_width: 100.0,
        children: vec![content_box],
        ..Default::default()
    };
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 100.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    position_cells(&mut table_box, &grid, &[100.0], 0.0, 0.0, &styles);

    let cell = &table_box.children[0];
    let content = &cell.children[0];
    // 首子应被下移 cell.margin_top（BFC 顶部留白），而非停在 y=0（旧 bug 丢失 margin）
    assert_eq!(
        content.y, 50.0,
        "first child should shift down by cell.margin_top (BFC margin containment), \
         got y={}; old behavior left content at y=0 losing the top margin",
        content.y
    );
    // 下移后内容底仍在 content 区内（顶部留白从底部移到顶部，总高不变）
    let content_bottom = content.y + content.height;
    assert!(
        content_bottom <= cell.content_height + 0.5,
        "shifted content bottom ({content_bottom}) must stay within content_height ({})",
        cell.content_height
    );
}

/// R769c：auto-width（shrink-to-fit）`display:table` 的行/单元格宽度须用列宽之和，
/// 而非 taffy 拉伸的容器宽。旧实现 `position_cells` 用 `table_box.content_width`
/// （taffy 对 auto 表拉伸到容器，如 784），早于 `apply_table_size_constraints` 收缩表盒，
/// 致行（背景）保持拉伸宽而表盒收缩——anonymous-table-cell-margin-collapsing 的绿行
/// 渲染 784px 宽（应 ~100）。修复：`table_content_width = min(content_width, col_sum+spacing)`。
#[test]
fn test_auto_width_table_shrink_to_fit_uses_col_sum_not_container() {
    use zero_css_parser::values::{DisplayValue, LengthValue};

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let row_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table; // 无 width → auto → shrink-to-fit
    styles.insert(table_id, ts);
    let mut rs = ComputedStyle::default();
    rs.display = DisplayValue::TableRow;
    styles.insert(row_id, rs);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.width = LengthValue::Px(100.0); // 显式 100px cell
    styles.insert(cell_id, cs);

    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        width: 100.0,
        ..Default::default()
    };
    let row_box = LayoutBox {
        node_id: Some(row_id),
        children: vec![cell_box],
        ..Default::default()
    };
    // taffy 对 auto-width 表拉伸到容器宽 784（典型视口填充）
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 784.0,
        width: 784.0,
        children: vec![row_box],
        ..Default::default()
    };

    layout_table(&mut table_box, &doc, &styles);

    // auto-width 表应 shrink-to-fit 到内容（~100），而非保持容器宽 784
    assert!(
        table_box.content_width < 200.0,
        "auto-width table content_width should shrink-to-fit to ~100 (col sum), got {} \
         (old bug kept taffy's stretched 784)",
        table_box.content_width
    );
}

/// CSS §17.6.1（separated borders model）：border-spacing 不仅分隔相邻 cell，还构成
/// 表格四边周界（外缘 cell 与 table 边缘之间）。旧实现只计列间 spacing，漏掉周界 → 带
/// border-spacing 的表尺寸偏小（visibility-collapse-border-spacing-002：1 列 100px +
/// spacing 50 → 应 200px，旧实现 100px）。同时空 cell（无内容）行高应为 0——chromium 对
/// 空 cell 不应用 strut（旧 20px 默认致带 border-spacing 的空表/折叠行表尺寸偏大）。
#[test]
fn test_separated_border_spacing_perimeter_and_empty_row_zero() {
    use zero_css_parser::values::LengthValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let row_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table; // 默认 border-collapse: separate
    ts.border_spacing.horizontal = 50.0;
    ts.border_spacing.vertical = 100.0;
    styles.insert(table_id, ts);
    let mut rs = ComputedStyle::default();
    rs.display = DisplayValue::TableRow;
    styles.insert(row_id, rs);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.width = LengthValue::Px(100.0); // 显式 100px 列宽
    styles.insert(cell_id, cs);

    // 空 cell（无子内容）
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        width: 100.0,
        height: 0.0,
        ..Default::default()
    };
    let row_box = LayoutBox {
        node_id: Some(row_id),
        children: vec![cell_box],
        ..Default::default()
    };
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 784.0,
        width: 784.0,
        children: vec![row_box],
        ..Default::default()
    };

    layout_table(&mut table_box, &doc, &styles);

    // 周界 spacing：左 50 + cell 100 + 右 50 = 200（旧实现仅 100，漏周界）
    assert!(
        (table_box.content_width - 200.0).abs() < 1.0,
        "separated-border table content_width should include perimeter spacing (50+100+50=200), \
         got {} (old impl gave 100, missing perimeter)",
        table_box.content_width,
    );
    // 空 cell 行高 0（无 strut）+ 上下周界 spacing 100+100 = 200
    assert!(
        (table_box.content_height - 200.0).abs() < 1.0,
        "empty-cell row should be 0 (no strut) + top/bottom perimeter spacing 100+100=200, \
         got {} (old impl added 20px strut)",
        table_box.content_height,
    );
}

/// R292：`border-collapse: collapse` 时，表四边缘的 cell border 与 table border
/// 折叠——table border 胜出（更宽）时覆盖 cell border，后者不应再叠加进表尺寸。
/// 旧实现把「列宽含半 cell border（border-center 语义）」+「表 border 整圈」两者
/// 都计入 → 表尺寸偏大（subpixel-collapsed-borders-001 宽多 5px、高多 ~10px）。
///
/// 本用例镜像 subpixel-collapsed-borders-001 几何：1×1 表，table border 5px，
/// cell border 4.95px。期望表 content 宽被扣除外边缘被覆盖的 cell border，
/// 接近 CSS2 §17.6.2 折叠语义的几何（而非旧的双重计入）。
#[test]
fn test_collapse_table_size_deducts_covered_edge_cell_border() {
    use zero_css_parser::values::LengthValue;
    use zero_style_system::property::types::{BorderCollapseValue, BorderStyleValue};

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    // table: border-collapse collapse + 四边 5px border
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    ts.border_collapse = BorderCollapseValue::Collapse;
    styles.insert(table_id, ts);
    // cell: 四边 4.95px border（table border 5 胜出，覆盖 cell 边缘 border）
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.border_top_width = LengthValue::Px(4.95);
    cs.border_right_width = LengthValue::Px(4.95);
    cs.border_bottom_width = LengthValue::Px(4.95);
    cs.border_left_width = LengthValue::Px(4.95);
    cs.border_top_style = BorderStyleValue::Solid;
    cs.border_right_style = BorderStyleValue::Solid;
    cs.border_bottom_style = BorderStyleValue::Solid;
    cs.border_left_style = BorderStyleValue::Solid;
    styles.insert(cell_id, cs);

    // cell LayoutBox：border 由 extract_layout 从 taffy 回读，测试直接设 4.95
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        border_top: 4.95,
        border_right: 4.95,
        border_bottom: 4.95,
        border_left: 4.95,
        ..Default::default()
    };
    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        // table border 5px（extract_layout 回读）
        border_top: 5.0,
        border_right: 5.0,
        border_bottom: 5.0,
        border_left: 5.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    assert!(!grid.rows.is_empty(), "grid should have at least one row");

    // 模拟 compute_column_widths 的 collapse 列宽：content(50) + (bl+br)/2 = 54.95
    let col_width = 50.0_f32 + (4.95 + 4.95) / 2.0; // 54.95
    // 行高含整 cell border top+bottom
    let total_row_height = 50.0_f32 + 4.95 + 4.95; // 59.9

    apply_table_size_constraints(&mut table_box, &grid, total_row_height, &[col_width], 0.0, &styles);

    // 期望：左+右边缘 cell border 被 table border 覆盖 → 扣 (bl+br)/2 = 4.95
    // content_width = col_width(54.95) - 4.95 = 50.0（旧 bug 会留 54.95，多算 ~5px）
    let expected_content_width = 50.0_f32;
    assert!(
        (table_box.content_width - expected_content_width).abs() < 0.5,
        "collapse table content_width ({}) should deduct covered edge cell border to ≈{} \
         (col_width={}); old double-count bug left it at {}",
        table_box.content_width,
        expected_content_width,
        col_width,
        col_width,
    );
    // 期望：顶+底边缘 cell border 被覆盖 → 扣 bt+bb = 9.9
    // content_height = row_height(59.9) - 9.9 = 50.0（旧 bug 会留 59.9，多算 ~10px）
    let expected_content_height = 50.0_f32;
    assert!(
        (table_box.content_height - expected_content_height).abs() < 0.5,
        "collapse table content_height ({}) should deduct covered top+bottom cell border to ≈{} \
         (row_height={}); old double-count bug left it at {}",
        table_box.content_height,
        expected_content_height,
        total_row_height,
        total_row_height,
    );
}

/// `<col>`/`<colgroup>` 的 background-color 应被收集为列背景（CSS Tables §17.5.3）。
/// visibility:collapse 的列跳过，非透明背景列记录 (node_id, x, width)。
#[test]
fn test_collect_table_col_backgrounds() {
    use zero_css_parser::values::{ColorValue, VisibilityValue};
    use zero_style_system::property::types::DisplayValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("table");
    let col1_id = doc.create_element("col"); // red
    let col2_id = doc.create_element("col"); // blue, visibility:collapse
    let col3_id = doc.create_element("col"); // green
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mk_col_style = |bg: ColorValue, vis: VisibilityValue| {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::TableColumn;
        s.background_color = bg;
        s.visibility = vis;
        s
    };
    styles.insert(
        col1_id,
        mk_col_style(ColorValue::Rgba(255, 0, 0, 255), VisibilityValue::Visible),
    );
    styles.insert(
        col2_id,
        mk_col_style(ColorValue::Rgba(0, 0, 255, 255), VisibilityValue::Collapse),
    );
    styles.insert(
        col3_id,
        mk_col_style(ColorValue::Rgba(0, 128, 0, 255), VisibilityValue::Visible),
    );

    let mut table_box = LayoutBox {
        node_id: Some(table_id),
        children: vec![
            LayoutBox {
                node_id: Some(col1_id),
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(col2_id),
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(col3_id),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // 3 列，col2 折叠；col_widths = [65, 0, 160]（折叠列宽 0）
    let grid = TableGrid {
        rows: Vec::new(),
        col_count: 3,
        collapsed_cols: vec![false, true, false],
        collapsed_rows: Vec::new(),
    };
    let col_widths = [65.0f32, 0.0, 160.0];

    collect_table_col_backgrounds(&mut table_box, &grid, &col_widths, 0.0, &styles, &doc);

    // 仅 col1(red) 和 col3(green)；col2(blue) 折叠跳过；x/width 镜像单元格定位
    let entries = &table_box.table_col_backgrounds;
    assert_eq!(entries.len(), 2, "collapsed col should be skipped");
    assert_eq!(entries[0].0, col1_id, "first entry = col1 (red)");
    assert!(
        (entries[0].1 - 0.0).abs() < 0.01 && (entries[0].2 - 65.0).abs() < 0.01,
        "col1 x=0 w=65"
    );
    assert_eq!(entries[1].0, col3_id, "second entry = col3 (green)");
    // col2 折叠不引入 spacing/位移 → col3 从 x=65 起
    assert!(
        (entries[1].1 - 65.0).abs() < 0.01 && (entries[1].2 - 160.0).abs() < 0.01,
        "col3 x=65 w=160 (collapsed col2 contributes no offset), got x={} w={}",
        entries[1].1,
        entries[1].2
    );
}

/// CSS Tables §17.5.2.1：table-layout:fixed + 显式 width 时，内容列宽和若超出 width，
/// 应按比例收缩列到 width（内容溢出 cell 由 cell 的 overflow 裁剪），而非让内容撑宽表格。
/// 回归 table-cell-overflow-auto-scrolled（fixed 表 width:100px 含 200px 内容 div）。
#[test]
fn test_fixed_layout_caps_columns_at_explicit_width_when_content_wider() {
    use zero_css_parser::values::LengthValue;
    use zero_style_system::TableLayoutValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    ts.table_layout = TableLayoutValue::Fixed;
    ts.width = LengthValue::Px(100.0);
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    styles.insert(cell_id, cs);

    // cell 内 200px 宽内容（比 table width 100px 更宽 → 应溢出而非撑宽表）
    let content = LayoutBox {
        width: 200.0,
        ..Default::default()
    };
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        children: vec![content],
        ..Default::default()
    };
    let table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 100.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    let col_widths = compute_column_widths(&table_box, &grid, &styles, &doc);

    // 修复前：内容 200px 撑宽列到 ~200；修复后：fixed + width:100px 收缩列到 ~100
    let total: f32 = col_widths.iter().sum();
    assert!(
        (total - 100.0).abs() < 2.0,
        "fixed-layout table (width:100px) with 200px content should cap column sum at ~100, got {}",
        total
    );
}

/// R364：显式 width 列在扩展填满容器时冻结，仅 auto 列吸收剩余空间。
/// CSS Tables auto 布局：显式 width 单元格的列不增长（chromium 行为）。
/// definite-width 表（width:200px）含 width:20px cell + auto cell：20px 列保持 20，
/// auto 列吸收剩余 ~180。修复前按比例扩展会把 20px 列撑到 ~133。
#[test]
fn test_r364_explicit_width_column_frozen_during_expansion() {
    use zero_css_parser::values::LengthValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_a = doc.create_element("div"); // width:20px（显式）
    let cell_b = doc.create_element("div"); // width:auto
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    ts.width = LengthValue::Px(200.0); // definite table width → 扩展填满触发
    styles.insert(table_id, ts);
    let mut ca = ComputedStyle::default();
    ca.display = DisplayValue::TableCell;
    ca.width = LengthValue::Px(20.0); // 显式 width
    styles.insert(cell_a, ca);
    let mut cb = ComputedStyle::default();
    cb.display = DisplayValue::TableCell; // width:auto
    styles.insert(cell_b, cb);

    let cell_a_box = LayoutBox {
        node_id: Some(cell_a),
        width: 20.0,
        content_width: 20.0,
        ..Default::default()
    };
    let cell_b_box = LayoutBox {
        node_id: Some(cell_b),
        width: 10.0,
        content_width: 10.0,
        ..Default::default()
    };
    let table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 200.0,
        children: vec![cell_a_box, cell_b_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    let col_widths = compute_column_widths(&table_box, &grid, &styles, &doc);
    assert_eq!(col_widths.len(), 2, "应有 2 列");
    // 显式 20px 列冻结（不吸收剩余空间），保持 ~20 而非被比例撑大
    assert!(
        (col_widths[0] - 20.0).abs() < 3.0,
        "显式 width:20px 列应冻结在 ~20，got {}",
        col_widths[0]
    );
    // auto 列吸收剩余空间 → ~180（200 - 20）
    assert!(
        (col_widths[1] - 180.0).abs() < 5.0,
        "auto 列应吸收剩余 ~180，got {}",
        col_widths[1]
    );
}

/// R364b：显式 width 小于单元格 min-content 时，列宽取 max(explicit, min-content)。
/// 显式 3px cell（min-content ~9.6px 来自 compute_cell_intrinsic_width）→ 列宽 ~9.6 而非 3
///（修复前 cell_used_width 显式分支直接返回 explicit，不 floor 到 min-content）。
#[test]
fn test_r364b_explicit_width_floored_at_min_content() {
    use zero_css_parser::values::LengthValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.width = LengthValue::Px(3.0); // 显式 3px（>= 2.0 阈值 → 走 explicit 分支）< min-content
    styles.insert(cell_id, cs);

    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        width: 3.0,
        content_width: 3.0,
        ..Default::default()
    };
    let table_box = LayoutBox {
        node_id: Some(table_id),
        content_width: 200.0,
        children: vec![cell_box],
        ..Default::default()
    };

    let grid = build_grid(&table_box, &doc, &styles);
    let col_widths = compute_column_widths(&table_box, &grid, &styles, &doc);
    // 列宽应 floor 到 min-content（~9.6），远大于显式 3px（修复前会返回 3）
    assert!(
        col_widths[0] >= 8.0,
        "显式 width:3px < min-content 时列宽应 floor 到 ~9.6，got {}",
        col_widths[0]
    );
}

/// R702/R679：cell 含 block 子（width ≈ cell width）+ 文本（含大量空白）时，
/// `compute_cell_intrinsic_width` 应返回 `box_content_max_width`（内容 max-content），
/// 而非 `collect_text_length` 把空白也计入的 char_count 估算。修复前：block 子
/// width ≈ cell width 时 else-if 跳过 → 落 text path → char_count（含空白）过大
/// → table 不 shrink-to-fit（margin-collapse-101 table 1446px 溢出 viewport）。
#[test]
fn test_r702_cell_intrinsic_uses_max_content_not_whitespace_charcount() {
    use zero_css_parser::values::LengthValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("div");
    let cell_id = doc.create_element("div");
    let inner_id = doc.create_element("div");
    let text_id = doc.create_text_node("A\n\n\n\n\n"); // 「A」+ 大量空白（块间换行）
    let _ = doc.append_child(root, table_id);
    let _ = doc.append_child(table_id, cell_id);
    let _ = doc.append_child(cell_id, inner_id);
    let _ = doc.append_child(inner_id, text_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.font_size = LengthValue::Px(50.0);
    styles.insert(cell_id, cs);
    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Block;
    inner_style.font_size = LengthValue::Px(50.0);
    styles.insert(inner_id, inner_style);

    // inner.width ≈ cell.width → compute_cell_intrinsic_width 的 else-if（child < cell*0.95）跳过 → text path
    let inner_box = LayoutBox {
        node_id: Some(inner_id),
        width: 778.0,
        content_width: 778.0,
        ..Default::default()
    };
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        width: 778.0,
        content_width: 778.0,
        children: vec![inner_box],
        ..Default::default()
    };

    let intrinsic = compute_cell_intrinsic_width(&cell_box, &styles, &doc);
    // 修复前 char_count = 6 chars × char_width(30) = 180；修复后 box_content_max_width ≈ 「A」宽
    assert!(
        intrinsic < 80.0,
        "cell intrinsic 应用 box_content_max_content（~「A」宽），不应被空白 char_count 撑大，got {}",
        intrinsic
    );
}

/// R1131 slice 3：grow_vrl_cell_block_extent 单测。
/// 验证 None scale / rowspan gate / Some scale + text → N×fs 增长 三条路径。
#[test]
fn test_r1131_grow_vrl_cell_block_extent() {
    use zero_css_parser::values::LengthValue;

    // doc：cell 元素 + 文本 "AAAAA"（5 非空白 char）
    let mut doc = Document::new();
    let root = doc.root();
    let cell_id = doc.create_element("td");
    let text_id = doc.create_text_node("AA BB CC DD");
    let _ = doc.append_child(root, cell_id);
    let _ = doc.append_child(cell_id, text_id);

    // styles：cell font_size = 20px
    let mut styles = HashMap::new();
    let mut cs = ComputedStyle::default();
    cs.font_size = LengthValue::Px(20.0);
    styles.insert(cell_id, cs);

    // cb：width=40（taffy 原单字符宽），node_id=cell_id
    let mut cb = LayoutBox::default();
    cb.node_id = Some(cell_id);
    cb.width = 40.0;

    // cell：colspan=1, col [0,1)
    let cell = TableCell {
        child_index: 0,
        colspan: 1,
        rowspan: 1,
        col_start: 0,
        col_end: 1,
        parent_rg_idx: None,
    };
    let grid = TableGrid {
        rows: vec![],
        col_count: 1,
        collapsed_cols: vec![],
        collapsed_rows: vec![],
    };
    let col_widths = [100.0_f32];
    // R1146：新签名取 final_col_widths（已含 cap 分布）+ cap_fired: bool，不再内部 ×scale。
    // 模拟 vrl_cap_scale=0.25 后的 final_col_widths（100×0.25=25）。
    let scaled = [25.0_f32];

    // 1. cap_fired=false → cb.width
    let w = grow_vrl_cell_block_extent(&cb, &cell, &col_widths, false, 0.0, &grid, &styles, &doc);
    assert_eq!(w, 40.0, "cap_fired=false returns cb.width");

    // 2. rowspan>1 → cb.width（gate，避 vrl-006 回归）
    let mut cell_rs = cell.clone();
    cell_rs.rowspan = 2;
    let w = grow_vrl_cell_block_extent(&cb, &cell_rs, &scaled, true, 0.0, &grid, &styles, &doc);
    assert_eq!(w, 40.0, "rowspan>1 gated → cb.width");

    // 3. cap_fired=true + multi-word text → word-based packing 增长。text "AA BB CC DD"
    //    = 4 words × 2 char × 20 = 40/word；cell_h_scaled = final_col_widths[0] = 25。
    //    每 word 40 > 25 故每 word 一列 → N=4；grown = 4×20 = 80。
    let w = grow_vrl_cell_block_extent(&cb, &cell, &scaled, true, 0.0, &grid, &styles, &doc);
    assert!(
        (w - 80.0).abs() < 0.01,
        "scale 0.25 + 4-word (each 40>cell_h 25) → N=4 cols ×20 = 80, got {}",
        w
    );
    assert!(w > cb.width, "grown > original width");
}

#[test]
fn test_top_caption_extent_sums_top_caption_heights() {
    // R1653：top_caption_extent（caption-side:top 默认）= Σ caption 子盒高度；
    // caption-side:bottom 排除（由 post-processing 移到表底）；非 caption 子盒不计。
    use zero_style_system::property::CaptionSideValue;

    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("table");
    let cap1_id = doc.create_element("caption");
    let cap2_id = doc.create_element("caption");
    let thead_id = doc.create_element("thead");
    let _ = doc.append_child(root, table_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cap1 = ComputedStyle::default();
    cap1.display = DisplayValue::TableCaption; // caption-side:top（默认）
    styles.insert(cap1_id, cap1);
    let mut cap2 = ComputedStyle::default();
    cap2.display = DisplayValue::TableCaption;
    cap2.caption_side = CaptionSideValue::Bottom; // 排除
    styles.insert(cap2_id, cap2);
    let mut th = ComputedStyle::default();
    th.display = DisplayValue::TableHeaderGroup;
    styles.insert(thead_id, th);

    let table_box = LayoutBox {
        node_id: Some(table_id),
        children: vec![
            LayoutBox {
                node_id: Some(cap1_id),
                height: 19.0,
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(cap2_id),
                height: 15.0,
                ..Default::default()
            },
            LayoutBox {
                node_id: Some(thead_id),
                height: 33.0,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    // 仅 cap1（19）计入；cap2 bottom（15）排除；thead（33）非 caption 排除。
    assert_eq!(
        top_caption_extent(&table_box, &styles),
        19.0,
        "only top caption (cap1=19) counted; bottom caption + thead excluded"
    );
}
