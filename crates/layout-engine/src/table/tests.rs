//! table.rs 表格布局回归测试（从 table.rs 抽出，保持 2000 行约束）。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::DisplayValue;
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
