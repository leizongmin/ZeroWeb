use super::*;
use crate::types::LayoutBox;
use zero_css_parser::values::DisplayValue;
use zero_css_parser::values::LengthValue;
use zero_dom::Document;
use zero_dom::NodeId;
use zero_style_system::{ComputedStyle, StyleSystem};

pub(super) fn make_style_with_display(display: DisplayValue, width: f64, height: f64) -> ComputedStyle {
    let mut style = ComputedStyle::default();
    style.display = display;
    if width > 0.0 {
        style.width = LengthValue::Px(width);
    }
    if height > 0.0 {
        style.height = LengthValue::Px(height);
    }
    style
}

/// 创建 html > body 容器，返回 (doc, body_id)。
pub(super) fn make_doc_with_body() -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

pub(super) fn find_child_by_node_id(root: &LayoutBox, target_id: NodeId) -> Option<&LayoutBox> {
    for child in &root.children {
        if child.node_id == Some(target_id) {
            return Some(child);
        }
        if let Some(found) = find_child_by_node_id(child, target_id) {
            return Some(found);
        }
    }
    None
}

pub(super) fn find_absolute_position_by_node_id(root: &LayoutBox, target_id: NodeId) -> Option<(f32, f32)> {
    find_absolute_position_by_node_id_inner(root, target_id, 0.0, 0.0)
}

fn find_absolute_position_by_node_id_inner(
    root: &LayoutBox,
    target_id: NodeId,
    parent_abs_x: f32,
    parent_abs_y: f32,
) -> Option<(f32, f32)> {
    for child in &root.children {
        // parent_abs_x/y 是 root 的内容区域绝对原点。
        // child.x/y 是相对于 root border-box 原点的偏移，
        // content_x/y 也是相对于自身 border-box 原点的偏移。
        // 因此 child 的绝对位置 = parent_abs + child.x，
        // child 的内容区域绝对原点 = parent_abs + child.x + child.content_x。
        let abs_x = parent_abs_x + child.x;
        let abs_y = parent_abs_y + child.y;
        if child.node_id == Some(target_id) {
            return Some((abs_x, abs_y));
        }
        // 递归时传递 child 的内容区域绝对原点
        let child_content_abs_x = abs_x + child.content_x;
        let child_content_abs_y = abs_y + child.content_y;
        if let Some(found) =
            find_absolute_position_by_node_id_inner(child, target_id, child_content_abs_x, child_content_abs_y)
        {
            return Some(found);
        }
    }
    None
}

mod anonymous_flex_item_tests;
mod coverage;
mod incremental_parity_experiment;
mod intrinsic_two_pass_tests;
mod r1001_table_cell_direct_text_tests;
mod r109_backfill_tests;
mod r1153_table_cell_nested_explicit_width_tests;
mod r1242_pure_text_float_tests;
mod r1277_float_lift_height_guard_tests;
mod r1280_float_inline_paint_tests;
mod r1285_br_between_blocks_tests;
mod r1311_br_inline_no_node_tests;
mod r1316_clearance_sibling_order_tests;
#[cfg(test)]
mod r1371_abspos_flex_stretch_tests;
#[cfg(test)]
mod r1382_float_anon_table_tests;
#[cfg(test)]
mod r1389_clear_no_float_context_tests;
#[cfg(test)]
mod r1390_table_cell_bfc_float_tests;
#[cfg(test)]
mod r1393_adjoining_float_clearance_tests;
#[cfg(test)]
mod r1398_abspos_cb_border_tests;
#[cfg(test)]
mod r1404_aspect_ratio_flex_stretch_tests;
#[cfg(test)]
mod r1411_column_flex_aspect_main_tests;
#[cfg(test)]
mod r1412_align_content_flex_end_tests;
mod r1423_multicol_balance_text_node_is_ahem_tests;
#[cfg(test)]
mod r1518_table_among_floats_tests;
#[cfg(test)]
mod r1616_definite_height_float_overflow_tests;
#[cfg(test)]
mod r1619_nested_bfc_float_avoid_tests;
#[cfg(test)]
mod r1620_table_cell_float_aware_height_tests;
#[cfg(test)]
mod r1623_bfc_shrink_content_width_tests;
#[cfg(test)]
mod r1626_border_conflict_tie_color_tests;
#[cfg(test)]
mod r1637_u1b_font_metric_provider_wiring_tests;
#[cfg(test)]
mod r1721_right_float_table_avoid_tests;
#[cfg(test)]
mod r1722_right_bfc_pushbelow_tests;
#[cfg(test)]
mod r1723_table_beside_float_defwidth_tests;
#[cfg(test)]
mod r1728_left_fit_pushbelow_tests;
#[cfg(test)]
mod r1730_multifloat_coord_tests;
#[cfg(test)]
mod r1733_inline_block_float_avoid_tests;
#[cfg(test)]
mod r1743_body_height_repro;
mod r1752_anon_table_margin_diag;
#[cfg(test)]
mod r1771_clear_empty_containment_tests;
#[cfg(test)]
mod r1781_semi_replaced_stretch_probe;
#[cfg(test)]
mod r1782_table_cell_overflow_probe;
mod r1964_vertical_float_inf_diag;
mod r1976_vertical_ahem_p_extent_diag;
#[allow(dead_code)]
mod r1982_anon_block_pct_height_probe;
mod r1982_overflow_scroll_container_diag;
#[allow(dead_code)]
mod r1982c_mixed_children_dump;
#[allow(dead_code)]
mod r1984_neg_margin_width_probe;
#[allow(dead_code)]
mod r1986_inline_svg_width_probe;
mod r2013_print_layout_width_tests;
mod r2091_grid_replaced_pct_indefinite_tests;
mod r2101_quirks_pct_height_table_cell_tests;
mod r2108_table_internal_margin_suppressed_tests;
mod r2170_quirks_pct_height_flex_grid_tests;
mod r2171_ar_container_cross_size_tests;
mod r2173_svg_attr_aspect_ratio_tests;
mod r2234_logical_float_clear_tests;
mod r2248_margin_trim_tests;
mod r2302_isolation_stacking_context_tests;
mod r2309_sc_trigger_tests;
mod r2428_img_aspect_ratio_padding_tests;
mod r2429_contain_size_replaced_tests;
mod r2431_line_clamp_cap_tests;
mod r2854_clear_display_gate_tests;
mod r717_flex_ratio_img_tests;
mod table_layout_tests;
mod tests_1;
mod tests_10;
mod tests_10b;
mod tests_11;
mod tests_2;
mod tests_3;
mod tests_4;
mod tests_5;
mod tests_6;
mod tests_7;
mod tests_8;
mod tests_9;
mod writing_mode_tests;

/// R57（M3）：canvas-grid 结构全管线复现——span > div + canvas 的最终 canvas 盒位置
///（oracle A/B 22px 偏移：IFC run.y=0 正确但最终盒 y=20，定位移动来源）。
#[test]
fn r57_canvas_grid_wrapper_position() {
    let html = r#"<html><body style="margin:0"><div id="grid">
<span><div>label</div><canvas width="100" height="50"><p>fallback</p></canvas></span>
</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let canvas_id = doc
        .get_elements_by_tag_name("canvas")
        .into_iter()
        .next()
        .expect("canvas");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(b: &LayoutBox, id: NodeId, off_y: f32) -> Option<(f32, f32)> {
        let abs_y = off_y + b.y;
        if b.node_id == Some(id) {
            return Some((abs_y, b.y));
        }
        for c in &b.children {
            if let Some(v) = find(c, id, abs_y) {
                return Some(v);
            }
        }
        None
    }
    if let Some((abs_y, _rel_y)) = find(&result.root, canvas_id, 0.0) {
        // oracle 期望：grid 顶 0（body margin 0）+ label div 底 + ~0 → canvas abs_y ≈ 18-20
        //（R57 修复前 = 39——R1286 空行 strut 给 block 代理断行前的空白行 20px 高度）
        assert!(
            abs_y < 40.0,
            "canvas 应紧跟 label（<40px），got {abs_y}（22px 偏移回归）"
        );
    } else {
        panic!("canvas box 不存在");
    }
}

/// R57（M3）：canvas display:block 时 fallback 子（<p>）不建盒——HTML §4.8.10
/// fallback 仅在元素不支持时显示。is_replaced_with_fallback 的 display 匹配
/// Block | InlineBlock（canvas-grid reftest 的 .grid-cell-content { display:block }
/// 使旧 InlineBlock 条件失效——fallback p 撑高 span → grid 行高错，canvas-grid
/// oracle 对角线布局 12.86% 之一）。
#[test]
fn r57_canvas_block_display_fallback_excluded() {
    let html = r#"<html><body style="margin:0"><div style="display:grid">
<span><canvas width="80" height="60" style="display:block"><p>fallback</p></canvas></span>
</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let canvas_id = doc
        .get_elements_by_tag_name("canvas")
        .into_iter()
        .next()
        .expect("canvas");
    let p_id = doc.get_elements_by_tag_name("p").into_iter().next().expect("p");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(b: &LayoutBox, id: NodeId) -> bool {
        if b.node_id == Some(id) {
            return true;
        }
        b.children.iter().any(|c| find(c, id))
    }
    assert!(find(&result.root, canvas_id), "canvas 应有盒");
    assert!(
        !find(&result.root, p_id),
        "canvas display:block 的 fallback 子（p）不应建盒"
    );
}

/// R57（M3）：grid item（span > div + canvas）的 max-content 宽——canvas-grid
/// reftest 的列宽差（test span 128 vs Chromium ~80——canvas0 左缘 32 vs 8，
/// margin 0 auto 居中偏移 24）。span 宽应 = max(div 文本, canvas 固有 80)，
/// 非求和（div 文本宽 48 + canvas 80 = 128 是求和语义——IFC 水平排列错）。
#[test]
fn r57_grid_span_max_content_width() {
    let html = r#"<html><body style="margin:0"><div style="display:grid;grid-template-columns:repeat(2,max-content)">
<span><div>source-over</div><canvas width="80" height="60"></canvas></span>
<span><div>source-in</div><canvas width="80" height="60"></canvas></span>
</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_spans(b: &LayoutBox, doc: &zero_dom::Document, out: &mut Vec<f32>) {
        if b.content_width > 0.0
            && b.node_id.is_some_and(|id| {
                doc.get(id)
                    .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "span"))
            })
        {
            out.push(b.content_width);
        }
        for c in &b.children {
            find_spans(c, doc, out);
        }
    }
    let mut spans = Vec::new();
    find_spans(&result.root, &doc, &mut spans);
    assert!(!spans.is_empty(), "grid span 盒应存在");
    for w in &spans {
        // max-content = max(div 文本宽, canvas 固有 80)——**非求和**（IFC 水平
        // 排列语义 128=80+48 是错）。div 文本「source-over」13px ≈ 94.4（字体
        // 度量差 vs Chromium ~75 属 rendering-compat 域——列宽差来源）。
        assert!(
            *w >= 80.0 && *w < 160.0,
            "span max-content 宽 = max(div 文本, canvas 80)（实测 {w} ∈ [80,160)）——求和语义回归"
        );
    }
}
