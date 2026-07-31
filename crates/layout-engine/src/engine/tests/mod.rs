use super::*;
use crate::types::LayoutBox;
use zero_css_parser::values::DisplayValue;
use zero_css_parser::values::LengthValue;
use zero_dom::Document;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

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
mod r717_flex_ratio_img_tests;
mod table_layout_tests;
mod tests_1;
mod tests_10;
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
