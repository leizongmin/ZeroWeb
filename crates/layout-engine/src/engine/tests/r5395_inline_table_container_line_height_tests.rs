//! R5395：纯 inline-table 行容器高度按 IFC 行盒合成（CSS2 §9.2.2/§9.7 × §10.6.3）。
//!
//! engine.rs `LayoutBox.is_block_level` 把 InlineTable 列为块级（taffy Table 映射视角），
//! `remeasure_inline_only_containers` 入口 `has_inline_children` 据此把纯 inline-table
//! 行容器排除在重测外 → 容器高保持 taffy 块堆叠值。实证（margin-collapse-134 ref 页）：
//! div > inline-table×2 并排单行，容器高 124 应 70（多计一个盒高 54 的块流槽），后继
//! 块整体下坠一行（table3 [136,190) 落到 [190,244)，+3.02% 像素差）。R5096 判例：
//! inline-table 是行内级（CSS2 §9.2.2）——本轮把 inline-table 子计入入口门（R5218
//! inline-flex/inline-grid 同域扩展），收缩臂豁免由 R5242/R5243 既有臂承接，容器收缩
//! 到 IFC 行盒高。R5218 曾把 InlineTable 留在豁免外（margin-applies-to-014 顾虑），
//! 本轮随同开关 `ZW_INLINE_TABLE_IB_SIZING`（=0 整链回退：入口不进、尺寸不收、cell
//! 臂不启）一并豁免，corpus A/B 复核该案不回归。
//!
//! auto 宽空 cell 定宽表的原子宽由 `inline_table_auto_max_content` cell 臂补源
//!（intrinsic_sizing.rs R5395）：纯文本 walk 对空 cell 测 0 → 调用点旧回退 `c.width`
//! = taffy Block 拉伸伪影全宽（R4032，实证 768）→ break_lines 必断行 → IFC 两行 76
//! 应单行 38，GROW 臂把容器高写回 76。

use super::*;
use crate::types::LayoutBox;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::BorderStyleValue;

/// r5218 同款：block 容器（300 宽、无显式高）+ N 个 40×20 原子行内级子。
fn atomic_row_case(displays: &[DisplayValue]) -> (LayoutBox, Vec<NodeId>) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut ids = Vec::new();
    for display in displays {
        let child = doc.create_element("div");
        doc.append_child(container, child).unwrap();
        let mut st = ComputedStyle::default();
        st.display = display.clone();
        st.width = LengthValue::Px(40.0);
        st.height = LengthValue::Px(20.0);
        styles.insert(child, st);
        ids.push(child);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let container_box = find_child_by_node_id(&result.root, container)
        .expect("container found")
        .clone();
    (container_box, ids)
}

/// parse_html + 全 viewport 布局，返回 (doc, result)。
fn compute_html(html: &str) -> (zero_dom::Document, crate::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 返回文档中第一个 `tag` 元素的 box（深度优先）。
fn first_element_box<'a>(doc: &zero_dom::Document, root: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
    fn walk_id(doc: &zero_dom::Document, id: NodeId, tag: &str) -> Option<NodeId> {
        if let Some(zero_dom::NodeKind::Element(e)) = doc.get(id).map(|n| &n.kind) {
            if e.local_name() == tag {
                return Some(id);
            }
        }
        for c in doc.child_nodes(id) {
            if let Some(f) = walk_id(doc, c, tag) {
                return Some(f);
            }
        }
        None
    }
    let id = walk_id(doc, doc.root(), tag)?;
    find_child_by_node_id(root, id)
}

/// 纯 inline-table 行（R5218 同构 40×20 双子）：容器高 = 行盒高 20（修前 taffy 块堆叠
/// 40），两子并排同行。R5218 旧断言「InlineTable 不入豁免保持堆叠 40」由本判例推翻。
#[test]
fn r5395_inline_table_children_row_height_not_stacked() {
    let (container_box, ids) = atomic_row_case(&[DisplayValue::InlineTable, DisplayValue::InlineTable]);
    assert!(
        (container_box.height - 20.0).abs() < 0.5,
        "容器高应为行盒高 20（修前 taffy 块堆叠 40），实际 {}",
        container_box.height
    );
    let b0 = find_child_by_node_id(&container_box, ids[0]).expect("child0 found");
    let b1 = find_child_by_node_id(&container_box, ids[1]).expect("child1 found");
    assert!(
        (b0.y - b1.y).abs() < 0.5,
        "两子应同行（y 相等），实际 {} vs {}",
        b0.y,
        b1.y
    );
    assert!(b1.x > b0.x + 39.0, "两子应水平并排，实际 x {} vs {}", b0.x, b1.x);
}

/// 空 cell 定宽表单行（margin-collapse-134 ref 最小形）：td 128×16、表 border 3px、
/// ml/mt 16 → 原子 border-box 134、margin-box 38；两表并排 → 容器高 38（修前 76：
/// mc=0 回退 c.width=768 taffy 伪影断成两行后 GROW 双计）。
#[test]
fn r5395_empty_cell_definite_table_single_line() {
    let td = "height:16px;width:128px;padding:0";
    let tbl = "display:inline-table;border:3px solid black;border-spacing:0;margin:16px 0 auto 16px";
    let html = format!(
        r#"<html><body><div><table style="{tbl}"><tr><td style="{td}"></td></tr></table><table style="{tbl}"><tr><td style="{td}"></td></tr></table></div></body></html>"#
    );
    let (doc, result) = compute_html(&html);
    let div = first_element_box(&doc, &result.root, "div").expect("div box found");
    assert!(
        (div.height - 38.0).abs() < 0.5,
        "容器高应为一个 margin-box 38（修前 76 两行/60 taffy 堆叠），实际 {}",
        div.height
    );
    assert_eq!(div.children.len(), 2, "div 直子应为两表");
    let (t0, t1) = (&div.children[0], &div.children[1]);
    for (i, t) in [t0, t1].into_iter().enumerate() {
        assert!(
            (t.width - 134.0).abs() < 0.5,
            "表{i} border-box 宽应为 134（cell 臂 128+3+3；修前 768 taffy 伪影），实际 {}",
            t.width
        );
        assert!(
            (t.height - 22.0).abs() < 0.5,
            "表{i} 高应 22（td 16 + border 6），实际 {}",
            t.height
        );
    }
    assert!(
        (t0.x - 16.0).abs() < 0.5 && (t1.x - 166.0).abs() < 0.5,
        "两表应 ml16 并排 x=16/166，实际 {} / {}",
        t0.x,
        t1.x
    );
    assert!(
        (t0.y - t1.y).abs() < 0.5 && (t0.y - 16.0).abs() < 0.5,
        "两表应同行 y=16（mt16 行盒），实际 {} vs {}",
        t0.y,
        t1.y
    );
}

/// 异高基线行：h22 + h40（td 16/34）默认 va:baseline → 容器高 = 高盒 40，矮盒底对齐
/// y=18（修前 62 = 22+40 块堆叠）。
#[test]
fn r5395_mixed_height_baseline_row() {
    let tbl = "display:inline-table;border:3px solid black;border-spacing:0;margin:0 0 auto 16px";
    let html = format!(
        r#"<html><body><div><table style="{tbl}"><tr><td style="height:16px;width:128px;padding:0"></td></tr></table><table style="{tbl}"><tr><td style="height:34px;width:128px;padding:0"></td></tr></table></div></body></html>"#
    );
    let (doc, result) = compute_html(&html);
    let div = first_element_box(&doc, &result.root, "div").expect("div box found");
    assert!(
        (div.height - 40.0).abs() < 0.5,
        "容器高应为最高盒 40（修前 62 块堆叠），实际 {}",
        div.height
    );
    let (t0, t1) = (&div.children[0], &div.children[1]);
    assert!(
        (t0.height - 22.0).abs() < 0.5 && (t1.height - 40.0).abs() < 0.5,
        "两表应 22/40，实际 {} / {}",
        t0.height,
        t1.height
    );
    assert!(
        (t0.y - 18.0).abs() < 0.5 && (t1.y - 0.0).abs() < 0.5,
        "矮表应基线对齐 y=18，实际 {} / {}",
        t0.y,
        t1.y
    );
}

/// 混排安全锚（R4108 域）：块级 p（18 高、m:0）+ inline-table → 容器高 40 = p 18 +
/// 表行盒 22，表 y=18 不叠 p（remeasure IFC 以流原点起行，块级子先行）。
#[test]
fn r5395_mixed_block_inline_table_safety_anchor() {
    let tbl = "display:inline-table;border:3px solid black;border-spacing:0;margin:0 0 auto 16px";
    let html = format!(
        r#"<html><body><div><p style="margin:0;height:18px">ab</p><table style="{tbl}"><tr><td style="height:16px;width:128px;padding:0"></td></tr></table></div></body></html>"#
    );
    let (doc, result) = compute_html(&html);
    let div = first_element_box(&doc, &result.root, "div").expect("div box found");
    assert!(
        (div.height - 40.0).abs() < 0.5,
        "混排容器高应 40（p 18 + 行盒 22），实际 {}",
        div.height
    );
    let p = div
        .children
        .iter()
        .find(|c| c.height > 17.0 && c.height < 19.0)
        .expect("p box found");
    let t = div
        .children
        .iter()
        .find(|c| (c.height - 22.0).abs() < 0.5)
        .expect("table box found");
    assert!(
        (p.y - 0.0).abs() < 0.5 && (t.y - 18.0).abs() < 0.5,
        "p 应 y=0、表应紧随 y=18，实际 {} / {}",
        p.y,
        t.y
    );
}

/// cell 臂单元锚（intrinsic_sizing R5395）：空 cell 定宽表（td 128、表 border 3）
/// `count_definite_cells=true` = 134；`false` = 0（旧文本 walk 语义，float Σ 臂与
/// R5243 锚域逐字节不变）。
#[test]
fn r5395_cell_arm_max_content_anchor() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("table");
    doc.append_child(body, table).unwrap();
    let tbody = doc.create_element("tbody");
    doc.append_child(table, tbody).unwrap();
    let tr = doc.create_element("tr");
    doc.append_child(tbody, tr).unwrap();
    let td = doc.create_element("td");
    doc.append_child(tr, td).unwrap();

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::InlineTable;
    // border 宽须配 style 非 None 才入账（R5395 cell 臂 style 守卫——直建 default
    // 的 border_style=None，宽度 medium 不代表实际边框）。
    ts.border_left_width = LengthValue::Px(3.0);
    ts.border_right_width = LengthValue::Px(3.0);
    ts.border_left_style = BorderStyleValue::Solid;
    ts.border_right_style = BorderStyleValue::Solid;
    styles.insert(table, ts);
    let mut tds = ComputedStyle::default();
    tds.display = DisplayValue::TableCell;
    tds.width = LengthValue::Px(128.0);
    styles.insert(td, tds);
    let mut row = ComputedStyle::default();
    row.display = DisplayValue::TableRow;
    styles.insert(tr, row);
    let mut group = ComputedStyle::default();
    group.display = DisplayValue::TableRowGroup;
    styles.insert(tbody, group);

    let with_cells = crate::intrinsic_sizing::inline_table_auto_max_content(table, &doc, &styles, true);
    assert!(
        (with_cells - 134.0).abs() < 0.5,
        "cell 臂应 = 定宽 cell 128 + 横 border 6 = 134，实际 {}",
        with_cells
    );
    let legacy = crate::intrinsic_sizing::inline_table_auto_max_content(table, &doc, &styles, false);
    assert!(
        legacy.abs() < 0.5,
        "旧口径（false）空 cell 纯文本 walk 应 0，实际 {}",
        legacy
    );
}
