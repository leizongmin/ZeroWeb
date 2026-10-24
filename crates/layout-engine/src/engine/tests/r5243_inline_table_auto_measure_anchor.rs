//! R5243：auto 宽 inline-table 入 float 收缩 Σ + table-row 裸文本匿名 cell（CSS2
//! §10.3.5 × §17.2.1 × §17.5.1）。
//!
//! R5242 谓词 FIXME 收账的三件套：
//! ① float Σ 对 auto 宽 inline-table 按 [`crate::intrinsic_sizing::
//!    inline_table_auto_max_content`] 独立测量（逐行 max，非整表连段）+ 表自身
//!    frame/margin 入账——调用方 auto_table_sum 全额计 margin-box，R4920 Σ 函数的
//!    InlineTable 防御臂对 auto 表 skip 防 margin 双计（margin-right-applies-to-014
//!    实证 float 70→120）；
//! ② table-row 直接裸文本子入布局序列（tree.rs 匿名 cell 级联，真表上下文门控）——
//!    修前 build_row 零 cell 表塌 0×0（table-vertical-align-baseline-009）；
//! ③ 纯 inline 容器的 y 锚定判据改 display 基准（postprocess `has_block_child`），
//!    inline-table 盒包装的 is_block_level 旗标不再误触发锚定（009 float 内整行
//!    原子被拉到 taffy 伪堆叠位）。
//!
//! kill-switch：① `ZW_FLOAT_INLINE_TABLE_MEASURE=0`；② `ZW_TABLE_ROW_TEXT_ANON=0`；
//! ③ `ZW_ANCHOR_REALBLOCK_DISPLAY=0`。OnceLock 单读 + corpus OFF 反证协议
//!（R3858/R5240 先例），单测进程内不设 env（默认全 ON）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::BorderStyleValue;

/// R5243 锚①（margin-right-applies-to-014 形态）：float 内 auto 宽 inline-table
/// （空 cell、border-right 10、margin-right 50）——wrapper 收缩到表 margin-box
/// 60，margin 只计一次。修前双计签名：Σ 函数 InlineTable 防御臂 ml+mr(50) +
/// auto_table_sum 全额(60) → wrapper 110。
#[test]
fn r5243_float_auto_inline_table_sigma_margin_counted_once() {
    let (mut doc, body) = make_doc_with_body();
    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();
    let table = doc.create_element("table");
    doc.append_child(wrapper, table).unwrap();
    let row = doc.create_element("div");
    doc.append_child(table, row).unwrap();
    let cell = doc.create_element("div");
    doc.append_child(row, cell).unwrap();

    let mut styles = HashMap::new();

    let mut ws = ComputedStyle::default();
    ws.display = DisplayValue::Block;
    ws.float = FloatValue::Left;
    styles.insert(wrapper, ws);

    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::InlineTable;
    // auto 宽（default width 保持 Auto）+ 右缘框与外距——margin-box = 10+0+50 = 60。
    ts.border_right_width = LengthValue::Px(10.0);
    ts.border_right_style = BorderStyleValue::Solid;
    ts.margin_right = LengthValue::Px(50.0);
    styles.insert(table, ts);

    let mut rows = ComputedStyle::default();
    rows.display = DisplayValue::TableRow;
    styles.insert(row, rows);

    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    cs.height = LengthValue::Px(200.0);
    styles.insert(cell, cs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let wrapper_box = find_child_by_node_id(&result.root, wrapper).expect("wrapper found");

    assert!(
        (wrapper_box.width - 60.0).abs() < 0.5,
        "float 应收缩到 auto 表 margin-box 60（margin 单次计入；双计签名 110），实际 {}",
        wrapper_box.width
    );
}

/// R5243 锚②（table-vertical-align-baseline-009 形态）：inline-table 两行各一 cell
/// 一字——auto max-content = **行间取 max**（单行宽），非整表连段（两行连测 = 2×）。
#[test]
fn r5243_inline_table_auto_max_content_per_row_max() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut fs = ComputedStyle::default();
    fs.display = DisplayValue::Block;
    fs.font_size = LengthValue::Px(50.0);
    styles.insert(container, fs);

    let build_table = |doc: &mut Document, container: zero_dom::NodeId, rows: usize| -> zero_dom::NodeId {
        let table = doc.create_element("table");
        doc.append_child(container, table).unwrap();
        for _ in 0..rows {
            let row = doc.create_element("div");
            doc.append_child(table, row).unwrap();
            let cell = doc.create_element("div");
            doc.append_child(row, cell).unwrap();
            let t = doc.create_text_node("X");
            doc.append_child(cell, t).ok();
        }
        table
    };

    let one_row = build_table(&mut doc, container, 1);
    let two_rows = build_table(&mut doc, container, 2);

    let single = crate::intrinsic_sizing::inline_table_auto_max_content(one_row, &doc, &styles);
    let double = crate::intrinsic_sizing::inline_table_auto_max_content(two_rows, &doc, &styles);

    assert!(single > 0.0, "单行表 max-content 应非零，实际 {single}");
    assert!(
        (double - single).abs() < 0.5,
        "两行表 max-content 应取行 max（{single}）而非行连段（{}），实际 {double}",
        single * 2.0
    );
}

/// R5243 锚③（修 B 主体）：真表上下文内 table-row 的裸文本子合成匿名 cell——row
/// 盒出现以宿主行为 node_id 的合成 cell 子、表不再塌 0×0；孤儿 row（DOM 祖先无
/// table）维持行级直绘不盒化（run-in-table-row-between / flexbox-with-pseudo-
/// elements-003 回归面，corpus 守）。
#[test]
fn r5243_table_row_bare_text_anon_cell_in_table_context() {
    let (mut doc, body) = make_doc_with_body();

    // 真表：inline-table > row > 裸文本。
    let table = doc.create_element("table");
    doc.append_child(body, table).unwrap();
    let row = doc.create_element("div");
    doc.append_child(table, row).unwrap();
    let t = doc.create_text_node("X");
    doc.append_child(row, t).ok();

    // 孤儿 row：body 直系，无 table 祖先。
    let orphan = doc.create_element("div");
    doc.append_child(body, orphan).unwrap();
    let orphan_row = doc.create_element("div");
    doc.append_child(orphan, orphan_row).unwrap();
    let ot = doc.create_text_node("Y");
    doc.append_child(orphan_row, ot).ok();

    let mut styles = HashMap::new();

    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::InlineTable;
    styles.insert(table, ts);

    let mut rs = ComputedStyle::default();
    rs.display = DisplayValue::TableRow;
    rs.font_size = LengthValue::Px(50.0);
    styles.insert(row, rs);
    let mut ors = ComputedStyle::default();
    ors.display = DisplayValue::TableRow;
    ors.font_size = LengthValue::Px(50.0);
    styles.insert(orphan_row, ors);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id(&result.root, table).expect("table found");
    assert!(
        table_box.width > 1.0 && table_box.height > 1.0,
        "真表内裸文本应合成 cell 撑起表（修前零 cell 塌 0×0），实际 {}×{}",
        table_box.width,
        table_box.height
    );
    let row_box = find_child_by_node_id(&result.root, row).expect("row found");
    assert!(
        row_box.children.iter().any(|c| c.node_id == Some(row)),
        "row 盒应有宿主行为 node_id 的合成 cell 子（R5168 盒级 drain），实际子数 {}",
        row_box.children.len()
    );

    // 孤儿 row：不合成 cell（无 node_id==自身 的子盒），文本走行级直绘。
    let orphan_box = find_child_by_node_id(&result.root, orphan_row).expect("orphan row found");
    assert!(
        !orphan_box.children.iter().any(|c| c.node_id == Some(orphan_row)),
        "孤儿 row 不得合成匿名 cell（run-in 族回归面），实际子数 {}",
        orphan_box.children.len()
    );
}

/// R5243 锚④（border-collapse-empty-row 反向守）：纯 inline-table 容器（无块级子、
/// 无文本）不触发 y 锚定——两表按 IFC 基线错排（高表 y 更小），不被拉平到首原子
/// taffy 伪堆叠位（修前 is_block_level 盒旗标误判 → 统一锚偏移 → y 一致）。
#[test]
fn r5243_pure_inline_table_container_baseline_stagger_kept() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Block;
    styles.insert(container, cs);

    let mut build = |doc: &mut Document, container: zero_dom::NodeId, cell_h: f64| -> zero_dom::NodeId {
        let table = doc.create_element("table");
        doc.append_child(container, table).unwrap();
        let row = doc.create_element("div");
        doc.append_child(table, row).unwrap();
        let cell = doc.create_element("div");
        doc.append_child(row, cell).unwrap();
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::InlineTable;
        styles.insert(table, ts);
        let mut rs = ComputedStyle::default();
        rs.display = DisplayValue::TableRow;
        styles.insert(row, rs);
        let mut cellst = ComputedStyle::default();
        cellst.display = DisplayValue::TableCell;
        cellst.height = LengthValue::Px(cell_h);
        styles.insert(cell, cellst);
        table
    };

    let short = build(&mut doc, container, 100.0);
    let tall = build(&mut doc, container, 150.0);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let short_box = find_child_by_node_id(&result.root, short).expect("short table found");
    let tall_box = find_child_by_node_id(&result.root, tall).expect("tall table found");
    assert!(
        tall_box.height > short_box.height,
        "表高应随 cell 高（{} vs {}）",
        tall_box.height,
        short_box.height
    );
    assert!(
        tall_box.y < short_box.y - 1.0,
        "纯 inline 容器内两表应按 IFC 基线错排（高表 y 更小；修前锚定拉平 y 一致），实际 tall.y={} short.y={}",
        tall_box.y,
        short_box.y
    );
}
