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
mod clearfix_nested_float_clearance_tests;
mod coverage;
mod downloaded_font_metrics;
mod incremental_parity_experiment;
mod intrinsic_two_pass_tests;
mod layout_incremental_oracle;
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
mod r3755_bfc_margin_collapse_tests;
mod r3765_block_ar_tests;
mod r3769_cross_block_cap_zero_tests;
mod r3770_pre_text_with_oof_child_tests;
mod r3779_float_no_ghost_line_tests;
mod r3780_cross_block_extent_inflow_tests;
mod r3792_aspect_ratio_intrinsic_tests;
mod r3793_fieldset_clamp_skip_tests;
mod r3794_replaced_intrinsic_keyword_tests;
mod r3807_bfc_zero_shift_float_plain_flow_tests;
mod r3808_clear_mt_leak_revert_tests;
mod r3809_va_margin_box_align_tests;
mod r3817_anon_cell_center_tests;
/// R3858 abspos 非根 positioned 祖先 CB 重解析测试。
mod r3858_abspos_nested_cb_tests;
/// R3859 flex item min-cross→min-main transferred size 测试。
mod r3859_flex_ar_transferred_min_tests;
/// R3860 grid item stretch 轴钳制 + ratio 传递测试。
mod r3860_grid_ar_stretch_transfer_tests;
mod r3893_block_mixed_flag_tests;
#[allow(dead_code)]
mod r3893_mixed_body_probe;
mod r3929_abspos_shrink_to_fit_tests;
mod r3992_run_in_merge_tests;
mod r4023_inline_block_block_child_sync_tests;
mod r4029_cell_intrinsic_destretch_tests;
mod r4033_empty_ib_frame_shrink_tests;
mod r4034b_contain_size_content_tests;
mod r4037_abspos_fitcontent_subtree_tests;
mod r4043_nested_inline_store_tests;
#[allow(dead_code)]
mod r4043_nested_q_stack_probe;
mod r4108_mixed_replaced_inline_last_tests;
mod r4112_anon_fragment_percent_cb_tests;
mod r4134_empty_inline_frame_intrinsic_tests;
mod r4140_inline_first_mt_unhoist_tests;
mod r4149_width_keyword_clamp_tests;
mod r4176_negative_margin_float_tests;
#[allow(dead_code)]
mod r4499_spanner_region_diag;
/// R5261 cell intrinsic inline 级显式宽子相邻 run 求和（CSS2 §17.5.2 + css-sizing-3）。
mod r5261_cell_inline_run_intrinsic_tests;
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
    // R4408 开门注：交错 walk default-on 下 span 被 R109 block-in-inline 分裂为
    // [div fragment][canvas fragment]，各 fragment 分别量自身内容（div 文本 ≈76.8 /
    // canvas 80）——per-fragment 宽即正确语义；span 的 max-content = 跨 fragment 取 max。
    // 旧断言逐 fragment ≥80 对分裂结构过严（canvas 不在 div fragment 内）。
    let max_span_w = spans.iter().cloned().fold(0.0f32, f32::max);
    assert!(
        (80.0..160.0).contains(&max_span_w),
        "span max-content 宽 = max(div 文本, canvas 80)（跨 fragment max 实测 {max_span_w} ∈ [80,160)）——求和语义回归"
    );
}

// ── R3912：taffy aspect_ratio border-box 语义 × content-box 盒修正 ──

/// R3912（css-sizing-4 §4.1）：box-sizing:content-box（默认）的非替换块盒 + 裸
/// <ratio> + 一侧显式时，taffy 0.12 把 ratio 施于 border-box → auto 侧推导错误
///（block-aspect-ratio-005：width:50 + pl:50 → bb 宽 100 → 高 100，应 content
/// 50/1 = 50）。build_layout_tree 后 taffy size.height 应为 transferred content 高。
#[test]
fn r3912_content_box_ratio_transfers_content_height() {
    use zero_css_parser::values::LengthValue;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let div = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, div);

    let mut styles = std::collections::HashMap::new();
    styles.insert(html, zero_style_system::ComputedStyle::default());
    styles.insert(body, zero_style_system::ComputedStyle::default());
    let mut s = zero_style_system::ComputedStyle::default();
    s.display = zero_css_parser::values::DisplayValue::Block;
    s.width = LengthValue::Px(50.0);
    s.aspect_ratio = Some(1.0);
    s.padding_left = LengthValue::Px(50.0);
    styles.insert(div, s);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let layout = find_box(&result.root, div).expect("div LayoutBox");
    // content 50/1 = 50 高（border-box 高 = 50，无垂直 pb）。
    assert!(
        (layout.height - 50.0).abs() < 0.5,
        "content-box ratio must transfer content height 50, got {}",
        layout.height
    );
    assert!((layout.width - 100.0).abs() < 0.5, "border width = 50 + pl 50");
}

/// R3912（css-sizing-4 §4.2）：transferred 用 **min/max 钳后**显式侧——
/// width:300 + max-width:100 + ratio 2/1 → 高 = 100/2 = 50（033 div1），
/// 旧按 300 传 150 溢出。
#[test]
fn r3912_ratio_transfer_uses_clamped_width() {
    use zero_css_parser::values::LengthValue;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let div = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, div);

    let mut styles = std::collections::HashMap::new();
    styles.insert(html, zero_style_system::ComputedStyle::default());
    styles.insert(body, zero_style_system::ComputedStyle::default());
    let mut s = zero_style_system::ComputedStyle::default();
    s.display = zero_css_parser::values::DisplayValue::Block;
    s.width = LengthValue::Px(300.0);
    s.max_width = LengthValue::Px(100.0);
    s.aspect_ratio = Some(2.0);
    styles.insert(div, s);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let layout = find_box(&result.root, div).expect("div LayoutBox");
    assert!(
        (layout.height - 50.0).abs() < 0.5,
        "transferred height must use max-width-clamped width (100/2=50), got {}",
        layout.height
    );
}

/// R3912 对称面：height 显式 + width:auto + ratio → width = height × ratio，
/// 且显式 height 不被钳后宽度反推（047：height:50 + min-h:100 + ratio 1/2 +
/// min-w:100 → 100×100，旧 200 高）。
#[test]
fn r3912_explicit_height_not_rederived_from_clamped_width() {
    use zero_css_parser::values::LengthValue;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let div = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, div);

    let mut styles = std::collections::HashMap::new();
    styles.insert(html, zero_style_system::ComputedStyle::default());
    styles.insert(body, zero_style_system::ComputedStyle::default());
    let mut s = zero_style_system::ComputedStyle::default();
    s.display = zero_css_parser::values::DisplayValue::Block;
    s.height = LengthValue::Px(50.0);
    s.min_height = LengthValue::Px(100.0);
    s.min_width = LengthValue::Px(100.0);
    s.aspect_ratio = Some(0.5);
    styles.insert(div, s);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let layout = find_box(&result.root, div).expect("div LayoutBox");
    assert!(
        (layout.height - 100.0).abs() < 0.5 && (layout.width - 100.0).abs() < 0.5,
        "expected 100x100 (min-h lift + transferred width), got {}x{}",
        layout.width,
        layout.height
    );
}

/// R3994（css-sizing-4 §4.1/§4.2）：带 element 子盒的 width 侧——transferred 宽
/// **不钳** content-based minimum（043 assert）。子 definite Px 宽 100 > transferred 50
/// → 宽 100（R3912 首版的 skip gate 已被语义实现取代；旧锚定 50 的测试随 043 翻绿废止）。
#[test]
fn r3994_width_side_transferred_does_not_clamp_content_minimum() {
    use zero_css_parser::values::LengthValue;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let div = doc.create_element("div");
    let child = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, div);
    let _ = doc.append_child(div, child);

    let mut styles = std::collections::HashMap::new();
    styles.insert(html, zero_style_system::ComputedStyle::default());
    styles.insert(body, zero_style_system::ComputedStyle::default());
    let mut s = zero_style_system::ComputedStyle::default();
    s.display = zero_css_parser::values::DisplayValue::Block;
    s.height = LengthValue::Px(200.0);
    s.max_height = LengthValue::Px(100.0);
    s.aspect_ratio = Some(0.5);
    // default ComputedStyle 的 border_width = medium Px(3)——清零使 border-box = content-box。
    s.border_top_width = LengthValue::Px(0.0);
    s.border_right_width = LengthValue::Px(0.0);
    s.border_bottom_width = LengthValue::Px(0.0);
    s.border_left_width = LengthValue::Px(0.0);
    styles.insert(div, s);
    let mut cs = zero_style_system::ComputedStyle::default();
    cs.display = zero_css_parser::values::DisplayValue::Block;
    cs.width = LengthValue::Px(100.0);
    cs.border_top_width = LengthValue::Px(0.0);
    cs.border_right_width = LengthValue::Px(0.0);
    cs.border_bottom_width = LengthValue::Px(0.0);
    cs.border_left_width = LengthValue::Px(0.0);
    styles.insert(child, cs);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let layout = find_box(&result.root, div).expect("div LayoutBox");
    // 子 min-content 贡献 100（content 100 + 清零 frame）> transferred 50 → 宽 100。
    assert!(
        (layout.width - 100.0).abs() < 0.5,
        "transferred width must not clamp content-based minimum (expect 100), got {}",
        layout.width
    );
}

/// 递归按 NodeId 查找 LayoutBox。
fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_box(c, id))
}

// ── R3913：row flex 容器 cross 从 item flexed main × ratio 传递 ──

/// flex-aspect-ratio-011 语义（csswg #line-sizing）：容器 width:100 + item
/// width:50 + aspect 1/1 + flex:1 → item flexed 到 100 → transferred cross 100
/// → 容器高 100（旧按指定宽 50 传 → 高 50）。
#[test]
fn r3913_flex_container_cross_from_flexed_main() {
    use zero_css_parser::values::{DisplayValue, LengthValue};

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let container = doc.create_element("div");
    let item = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, container);
    let _ = doc.append_child(container, item);

    let mut styles = std::collections::HashMap::new();
    for id in [html, body] {
        styles.insert(id, zero_style_system::ComputedStyle::default());
    }
    let mut c = zero_style_system::ComputedStyle::default();
    c.display = DisplayValue::Flex;
    c.width = LengthValue::Px(100.0);
    styles.insert(container, c);
    let mut it = zero_style_system::ComputedStyle::default();
    it.display = DisplayValue::Block;
    it.width = LengthValue::Px(50.0);
    it.aspect_ratio = Some(1.0);
    it.flex_grow = 1.0;
    styles.insert(item, it);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let container_box = find_box(&result.root, container).expect("container LayoutBox");
    let item_box = find_box(&result.root, item).expect("item LayoutBox");
    // flexed main = 100（flex:1 grow）→ transferred cross = 100。
    assert!(
        (item_box.height - 100.0).abs() < 0.5,
        "item cross must transfer from flexed main 100, got {}",
        item_box.height
    );
    assert!(
        (container_box.height - 100.0).abs() < 0.5,
        "container cross must come from item transferred cross, got {}",
        container_box.height
    );
}

/// 非 flexed 场景守卫：item 无 ratio 时容器 cross 不被本 pass 触碰。
#[test]
fn r3913_no_ratio_item_untouched() {
    use zero_css_parser::values::{DisplayValue, LengthValue};

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let container = doc.create_element("div");
    let item = doc.create_element("div");
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, container);
    let _ = doc.append_child(container, item);

    let mut styles = std::collections::HashMap::new();
    for id in [html, body] {
        styles.insert(id, zero_style_system::ComputedStyle::default());
    }
    let mut c = zero_style_system::ComputedStyle::default();
    c.display = DisplayValue::Flex;
    c.width = LengthValue::Px(100.0);
    styles.insert(container, c);
    let mut it = zero_style_system::ComputedStyle::default();
    it.display = DisplayValue::Block;
    it.width = LengthValue::Px(50.0);
    it.height = LengthValue::Px(30.0);
    styles.insert(item, it);

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    let container_box = find_box(&result.root, container).expect("container LayoutBox");
    // 无 ratio：cross = item 高 30（taffy 既有）。
    assert!(
        (container_box.height - 30.0).abs() < 0.5,
        "no-ratio item must keep taffy baseline cross 30, got {}",
        container_box.height
    );
}

// ── R3918：单 IFC 容器的原子内线 clamp 隐藏 ──

/// line-clamp-032 语义（css-overflow-4）：容器 line-clamp:4 全由原子内线（inline-block
/// span）组成、无直接文本时，clamp 点后的原子内线须隐藏。跨块 clamp pass 的原子内线
/// 行计数（round(高/容器 lh)）消耗预算，耗尽后隐藏后续。
#[test]
fn r3918_atomic_inline_after_clamp_point_hidden() {
    use zero_css_parser::values::{DisplayValue, LengthValue};

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    let body = doc.create_element("body");
    let container = doc.create_element("div");
    let mut spans = Vec::new();
    let _ = doc.append_child(root, html);
    let _ = doc.append_child(html, body);
    let _ = doc.append_child(body, container);
    for _ in 0..8 {
        let s = doc.create_element("span");
        let _ = doc.append_child(container, s);
        spans.push(s);
    }

    let mut styles = std::collections::HashMap::new();
    for id in [html, body] {
        styles.insert(id, zero_style_system::ComputedStyle::default());
    }
    let mut c = zero_style_system::ComputedStyle::default();
    c.display = DisplayValue::Block;
    c.height = LengthValue::Px(200.0);
    c.line_clamp = zero_style_system::property::types::LineClampComputedValue::Count(4);
    styles.insert(container, c);
    for (i, &id) in spans.iter().enumerate() {
        let mut s = zero_style_system::ComputedStyle::default();
        s.display = DisplayValue::InlineBlock;
        s.width = LengthValue::Px(150.0);
        s.height = LengthValue::Px(25.0);
        styles.insert(id, s);
        let _ = i;
    }

    let result = crate::LayoutEngine::new(800.0, 600.0).compute(&doc, &styles);
    // 前 4 个 span（clamp 预算内）保持可见几何；后 4 个（clamp 点后）隐藏清零。
    for (i, &id) in spans.iter().enumerate() {
        let b = find_box(&result.root, id).expect("span LayoutBox");
        if i < 4 {
            assert!(
                b.height > 0.5,
                "span {i} before clamp point must stay visible, got h={}",
                b.height
            );
        } else {
            assert!(b.line_clamp_hidden, "span {i} after clamp point must be hidden");
        }
    }
}
mod r4181_clamp_descendant_bmp_tests;
mod r4185_flex_line_clamp_tests;
mod r4193_inline_size_containment_tests;
mod r4195_legend_inline_size_tests;
mod r4331_br_in_inline_walk_tests;
mod r4398_preline_forced_break_tests;
mod r4941_hoisted_atomic_probe;
mod r4986_vertical_ar_tests;
mod r4987_abspos_ar_tests;
mod r4988_flex_ar_min_tests;
mod r5024_flex_subtree_inline_leaf_tests;
mod s10_hotlist_geometry_tests;
mod s19_inline_atomic_geometry_tests;

mod r5047_flex_ar_flexed_main_tests;

mod r5048_replaced_kw_border_box_tests;
/// R5073 abspos float 子 shrink-to-fit（taffy Definite 臂 float 贡献）测试。
mod r5073_abspos_float_shrink_tests;
/// R5074 abspos auto margin inset 门（§10.3.7 rule 3/4）测试。
mod r5074_abspos_auto_margin_inset_tests;
/// R5077 float 底边距边缘 floor（§9.5.1 + §8.3.1）测试。
mod r5077_float_prev_mb_floor_tests;
/// R5078 iframe default object size（§10.3.2 + §10.6.2）测试。
mod r5078_iframe_default_size_tests;
/// R5080 colgroup/column min-max-height 不适用（§17.3）测试。
mod r5080_column_vsize_tests;
/// R5081 伪元素文本节点 white-space 自身条目优先测试。
mod r5081_pseudo_ws_tests;
/// R5095 原子行内级盒行盒组装（inline-table va 对齐豁免重叠下推）测试。
mod r5095_inline_table_line_box_tests;
/// R5097 display:table 直接散文本子匿名 cell 收缩适应（CSS Tables §17.2.1）测试。
mod r5097_table_raw_text_anon_cell_tests;
/// R5099 行指定高 floor（css-tables-3 §3.10.1 / CSS2 §17.5.3）测试。
mod r5099_row_spec_floor_tests;
/// R5100 collapse 边框胜出宽度入 cell border-box（CSS2 §17.6.1）测试。
mod r5100_collapse_border_box_growth_tests;
/// R5104 auto 宽 abspos 单行测量宽度上取整（CSS2 §10.3.7 shrink-to-fit）测试。
mod r5104_abspos_shrink_to_fit_ceil_tests;
/// R5108 块盒 px max-height 钳制 + 塌穿泄漏 margin 的容器 margin-box 计入测试。
mod r5108_max_height_margin_box_tests;
/// R5109 诊断探针（临时）：margin-collapse-157/142 形态 margin 泄漏路径。
/// R5109 clearance 修正 extent 缩减祖先传播 + 空块末链自折叠剔除测试。
mod r5109_clearance_extent_propagation_tests;
/// R5110 替换元素 max 侧 min-content/max-content 关键字钳制测试。
mod r5110_replaced_mincontent_tests;
mod r5117_calc_cb_base_tests;
/// R5148 fieldset 自身 multicol 容器时渲染 legend 不入列流（chromium fieldset/legend
/// 特殊布局 × css-multicol；multicol-span-all-fieldset-001/002/003 驱动）。
mod r5148_fieldset_legend_multicol_tests;
/// R5152 contain:size 的 multicol 按「无内容」尺寸化、列几何仍参与（css-contain-1 §3.1
/// × css-multicol-1；contain-size-multicol-001/002/003 驱动）。
mod r5152_contain_size_multicol_sizing_tests;
/// R5153 container-type 蕴含 containment 布局接入 + containment 下 aspect-ratio
/// 高转移（css-contain-5 §3.1 × css-sizing-4 §4.1；contain-size-014 /
/// canvas-contain-size 驱动）。
mod r5153_container_type_containment_tests;
mod r5154_last_baseline_fallback_tests;
/// R5156 multicol 空盒（无流内内容 → 无断点机会）不可断——超高单片段原地溢出
///（css-break-3 §5.1 × css-multicol-1；contain-size-breaks-001-ref 驱动）。
mod r5156_multicol_empty_unbreakable_tests;
/// R5157 contain:size 的 grid track 几何尺寸化 + grid-gap legacy 别名
///（css-contain-1 §3.1 × css-align-3；contain-size-grid-002 驱动）。
mod r5157_contain_size_grid_tests;
/// R5158 body→viewport overflow 传播的 taffy 半边——body{overflow:hidden} 父子
/// margin 塌缩归位（css-overflow-3 §3.3；contain-size-grid-002 残差驱动）。
mod r5158_body_overflow_prop_taffy_tests;
mod r5160_bfc_float_order_tests;
/// R5161 跨 run 词中拼接边界无软换行机会（css-text-3 §5.2 / UAX#14；
/// contain-paint-047 驱动）。
mod r5161_cross_run_glue_tests;
/// R5162 table-layout:fixed 列宽分布 §17.5.2.1（specified 列保持 + auto 列均分；
/// 真 col CSS 宽 DOM 拾取；contain-paint-cell-001/002 驱动）。
mod r5162_table_fixed_dist_tests;
/// R5163 caption auto 宽在 size containment 下维持 fill 表宽（§17.4.1；
/// contain-paint-size-001/contain-strict-001 驱动）。
mod r5163_caption_contain_fill_tests;
/// R5164 auto 高 inline-block strut 估计 + table 行位 claim 门（csswg#8358；
/// css-tables/whitespace-001 驱动）。
mod r5164_inline_block_auto_height_tests;
/// R5166 域核探针：inline 流 abs-pos 静态位（css-position-3 §static-position）。
mod r5166_oof_inline_static_pos_probe;
mod r5167_table_anon_font_probe;
mod r5168_anon_cell_wrap;
mod r5169_anon_cell_run_paint;
mod r5170_table_rtl_col_order;
mod r5172_caption_shrink_to_fit;
/// R5178 替换元素 min/max-width/height content 关键字 transferred size（csswg #3973）。
mod r5178_replaced_content_kw_tests;
/// R5205 clamp 收缩兄弟级联 + 末子 margin 吸收（line-clamp-023/025 锚点）。
mod r5205_clamp_shrink_cascade_tests;
/// R5206 clamp 内层 extent 末子 declared margin 吸收（line-clamp-022/024 锚点）。
mod r5206_clamp_inner_margin_tests;
/// R5207 匿名表根 §9.5 float 推开 + 保 shrink 宽（clear-applies-to-005/006 锚点）。
mod r5207_anon_table_float_tests;
/// R5208 blocks-divide-tables：分隔符后孤立 row 停分隔符下 + 虚高塌缩收缩传播
///（table-anonymous-objects-081~084 锚点）。
mod r5208_blocks_divide_tables_tests;
/// R5209 行组裸文本 → 匿名 cell + 孤立 retrofit 匿名行 cell 自携行 y
///（table-anonymous-objects-161~166 锚点）。
mod r5209_row_group_stray_text_tests;
/// R5210 float 重定位链值 mt 归位（clear-on-child-with-margins-2 锚点）。
mod r5210_clear_chain_mt_refold_tests;
/// R5211 float 子不阻断父盒 collapse-through + 塌穿链分离点断链门
///（margin-collapse-114 / adjoining-float-before-clearance /
/// no-clearance-due-to-large-margin 锚点）。
mod r5211_float_child_collapse_through_tests;
/// R5212 direct-float forced clearance 邻接吸收（§9.5.2 × §8.3.1，
/// adjoining-float-nested-forced-clearance / adjoining-float-new-fc 锚点）。
mod r5212_forced_clear_absorb_tests;
/// R5213 table-among-floats 区间化 fit（左右 float 夹缝 beside）+ D 步扩高上传基线
/// root.height + 浮动 caption x 豁免（floats-wrap-bfc-002/004/006、
/// float-applies-to-015 锚点）。
mod r5213_table_among_floats_interval_tests;
/// R5214 float 摆放 earliest-fit（§9.5.1 规则 5 as high as possible，带步进替代行推进）
/// + R5070 table 内部盒载体豁免（floats-wrap-bfc-004 双 right float 锚点）。
mod r5214_float_earliest_fit_tests;
/// R5215（CSS2 §9.2.1.1）原子 inline-level 子流参与者——002l 正反页几何锚点。
mod r5215_atomic_flow_tests;
/// R5216（CSS2 §9.5 rule 8）行盒避 float 下绕——inline-002l/r 断行新行几何锚点。
mod r5216_line_float_fit_tests;
/// R5217（CSS2 §9.5 规则 2 × §10.3.3）BFC 旁置 float margin-aware 钉位解几何锚点。
mod r5217_bfc_margin_pin_beside_tests;
/// R5218（CSS2 §9.2.1 × §10.6.3）含 inline-flex/inline-grid 子的块容器行盒高合成锚点。
mod r5218_atomic_inline_container_height_tests;
/// R5219（css-sizing-3 #max-content）fit-content/max-content 真空子树塌 0 + 盲区回退
/// 保持锚点。
mod r5219_vacuum_fitcontent_collapse_tests;
/// R5220（CSS2 §8.4 × css-sizing-3 #fit-content-size）% padding 解析基准——塌 0 真实
/// CB 传真值 0 锚点。
mod r5220_pct_padding_collapse_tests;
/// R5221（CSS2 §9.5 × §10.3.3）多 float 纵向堆叠下绕流游标修正锚点。
mod r5221_wrap_floor_multi_float_tests;
/// R5223（CSS2 §10.3.7 × §10.8）静态位 abspos 容器 IFC 存储 + 根级 run 不继承容器 va 锚点。
mod r5223_static_abspos_ifc_store_tests;
/// R5224（CSS2 §10.3.7 × css-position-3 §4.1）rtl 行首 inline abspos 静态位镜像锚点。
mod r5224_rtl_inline_abspos_static_tests;
/// R5225（css-position-3 §relative-positioning）display:inline 非替换盒 relative inset 单次施加锚点。
mod r5225_rel_inline_offset_tests;
/// R5226（CSS2 §9.2.1.1 + CSS Text §3）nbsp-only inline 内容推进块流锚点。
mod r5226_nbsp_flow_anon_block_tests;
/// R5227（CSS2 §9.2.1.1 × §9.5.1 × §10.3.7）OOF 静态位/float 行顶锚参与 preserved 行锚点。
mod r5227_oof_static_below_preserved_line_tests;
/// R5228（CSS2 §8.3.1 × §9.5.1）float 前置容器首流内子重放不重放已折叠 margin。
mod r5228_float_first_margin_replay_tests;
/// R5235 D1：rel span 包裹原子 img 后文本 run advance 丢失复现（R2156 提升路径）。
mod r5235_rel_span_atomic_advance_tests;
/// R5236 D2（CSS2 §10.3.8）：含 replaced 原子后代的 auto 宽 abspos shrink-to-fit 诊断锚。
mod r5236_abspos_stf_replaced_child_tests;
/// R5242（CSS2 §9.2.2 × §10.3.5）float 收缩对 definite 宽 inline-table 子按行内级
/// Σ 求和锚点。
mod r5242_inline_table_float_anchor;
/// R5243（CSS2 §10.3.5 × §17.2.1 × §17.5.1）auto 宽 inline-table 入 float Σ、
/// table-row 裸文本匿名 cell 与纯 inline 容器锚定判据锚点。
mod r5243_inline_table_auto_measure_anchor;
/// R5244（UAX #9 HL1）CSS direction 定段落基方向锚点——LTR 块 R-first run 视觉序
/// = 逻辑序、plaintext HL4 例外与 rtl 臂回归守。
mod r5244_bidi_ltr_para_base_anchor;
/// R5245（CSS2 §9.2.1.1 × §9.5）float 宿主 R109 匿名块流内化（不继承宿主 float 旗标）
/// 与匿名块片段 intrinsic 片段级测量锚点——set 4 形态宿主收缩宽、匿名块堆叠与非
/// float 宿主回归守。
mod r5245_anon_float_anchor;
/// R5246（css-writing-modes-3 §2.2 × css-text-3 §5.2）U+2028/U+2029 强制断行类 +
/// LS 段落连续行级 L2 锚点——pre 容器 LS/PS 折行行数与 RTL 混排跨 LS 两行守。
mod r5246_forced_break_ls_ps_anchor;
mod r5247_vertical_inline_metric_transpose;
/// R5248（CSS2 §10.3.x × R4332 折入语义）inline 树盒 x 双扣 border 修正锚点——
/// 盒 x = 流内内容首缘 − 流内落账 frame（折入落流按首 run ml−raw 检测），
/// box-level 边框/背景归位守。
mod r5248_inline_border_x_double_count;
/// R5249（CSS2 §10.2/§8.4 × R4332 折入语义）element-first 首槽 spacer 锚点——
/// 首内容为元素子时外层自身 ml+pl 以零宽 spacer 落首槽、fold 仍落自名 run，
/// 流内推进与内层盒守。
mod r5249_element_first_fold_spacer_anchor;
/// R5277：frame-only 空 caption 在 auto 表的收缩 + 表宽下界。
mod r5277_caption_frame_only_auto_table_tests;
/// R5278：collapse cell 胜出缘外半带与表自身 border 带同条不双计。
mod r5278_collapse_outer_band_size_tests;
/// R5279：collapse 表 border-spacing 整体无效。
mod r5279_collapse_ignores_border_spacing_tests;
/// R5280：separated 表层背景 cell 分段（列条目定位区/分段 + 行组层 bg_cell_segments）。
mod r5280_table_layer_bg_cell_segments_tests;
/// R5293（CSS2 §10.5 × §10.6.4 × §8.3.1）auto 高 abs 盒内容派生高不回灌为子代
/// 百分比高解析基 + table box 仅 bottom 侧 margin 隔离
///（child-bottom-margin-in-unresolvable-percentage-height、
/// margin-applies-to-015 锚点）。
mod r5293_abs_auto_height_pct_base_tests;
/// R5294 结构锚：动态构建 DOM 与静态同形标记布局逐盒一致（dir-style-04 排查
/// 中排除布局面路径分歧——实际根因在 engine innerHTML fragment 解析 trim）。
mod r5294_dyn_trailing_space_tests;
/// R5295（CSS2 §8.4）：换行 padded 行内盒 border-box 左缘写回——逐片段盒缘
/// 收敛（run 首片段扣自身 pl、续行/空片段不扣），负偏回归守护。
mod r5295_wrapped_padding_box_x_tests;

mod r5250_inline_relpos_shift_map;

/// t-p113（CSS Flexbox §9.1 item main size 由内容解析）taffy round_layout 累计整数
/// 取整把文本固有宽削到内容宽以下 → Phase A IFC 假断行；IFC 约束宽改取未取整层。
mod p113_flex_fractional_content_width_tests;
mod p116_advance_probe_tests;
mod p116_text_atomic_inline_line_tests;
/// R5251（CSS2 §9.2.1.1 block-in-inline）容器层升格拆分锚——含块 span 提升块子
/// 垂直堆叠 + margin collapse，inline 文本片段并入容器匿名块序。
mod r5251_block_in_inline_deep_split;
/// R5252（CSS2 §8.3.1 + §9.2.1.1）块→匿名行盒边界锚定 collapse gap 锚——原子
/// inline 的 margin-top 不与前驱块级兄弟 margin-bottom 块级 collapse。
mod r5252_anchor_collapse_gap;
/// R5275（CSS Text 3 §3 + CSS2 §10.3.5）preserve 纯空白 float 收缩锚——measure 空叶
/// 早退对 pre 纯 tab div 判「无 inline 内容」曾致 float 0 宽整缺（active-selection-063）。
mod r5275_preserved_ws_float_measure_tests;
/// R5300（CSS2 §10.3.7 shrink-to-fit 链）嵌套 abs-pos 链 CB 宽传播锚——单趟 walk
/// 写定宽度即时回写子代栈槽 cb_width，嵌套 branch 内层不再被过期 CB（父代塌缩宽）
/// 钳死（text-decoration-subelements-002 ref 侧 overflow:hidden 裁空内容根因）。
mod r5300_abspos_stf_cb_prop_tests;
/// R5315（css-position-3 §static-position）inline-block OOF 行内流静态位锚——
/// R5166 锚发射门扩 InlineBlock（UA replaced 组），abspos img 不再落块游标位盖住
/// 前驱 atomic inline-block（vertical-align-baseline-004a/005a）。
mod r5315_inline_block_oof_static_pos_tests;
/// R5316（CSS2 §10.8.1 va:text-* × 负 leading）text-top/text-bottom 基线锚定围合——
/// 行盒按父 content area 需求生长（负 leading 域），正 leading 域 max 合成零扰动
///（vertical-align-negative-leading-001）。
mod r5316_va_text_neg_leading_tests;
/// R5329（css-sizing-3 + css-flexbox-1 §4.5）替换内容叶 intrinsic 测量锚——img 叶
/// MinContent/MaxContent 回固有尺寸（非 0），flex item automatic min 的 specified/
/// content size suggestion 生效（flex-minimum-width-flex-items-005/006）。
mod r5329_replaced_leaf_intrinsic_min_tests;
/// R5330（css-flexbox-1 §4.1）column flex item 主轴高回填豁免锚——指定 min-height 非
/// auto 的 item 保持 flex 定尺不被 R1743 max-bottom 回填（flex-minimum-height-flex-items-027）。
mod r5330_col_flex_item_no_backfill_tests;
/// R5331（css-flexbox-1 §9.9.1）column flex 容器内在主轴定尺 item 内容贡献探针锚——
/// specified main size 只能经 automatic minimum 进入容器内在尺寸
///（flex-minimum-height-flex-items-029/030）。
mod r5331_flex_intrinsic_main_probe_tests;
/// R5332（css-flexbox-1 §4.5/§9.4）border-box flex 容器 transferred/stretch 目标按
/// content box cross 锚——img stretch 到容器 content cross 而非 outer cross
///（flex-cross-size-border-box-001）。
mod r5332_border_box_flex_cross_tests;
/// R5333（css-values-4 math × css-flexbox-1 §resolve-flexible-lengths）无穷 flex
/// 因子锚——`flex-grow: calc(infinity)` 吸收全部 free space、普通 math 长手求值、
/// shrink ∞ 拒收守卫（flex-grow-009）。
mod r5333_flex_infinite_grow_tests;
/// R5334（css-flexbox-1 §4.5 + css-overflow-3 §3.1）scroll container flex item
/// 自动最小 = 0 锚——overflow 非 visible 的 column item 不被 R1743 回填撑回内容高、
/// overflow visible 自动最小保留、row 轴 cross 域既有行为守卫
///（fieldset-as-item-overflow）。
mod r5334_flex_scroll_item_min_tests;

/// R5338（css-flexbox-1 §8.5 baseline alignment）：flex 叶首行基线通道锚——
/// baseline 对齐三项异高同顶（margin 参与基线偏移）、flex-start 守卫不变。
mod r5338_flex_leaf_baseline_tests;

/// R5346（quirks.spec #the-percentage-height-calculation-quirk）：quirks % 高的
/// flex/grid item 链语义——item 入链落账、非 item 后代 definite 链放行、item 自身
/// CB 不明确时 quirk 不适用。
mod r5346_quirks_pct_height_flex_item_chain_tests;

/// R5348（CSS2 §10.3.9 shrink-to-fit）：inline-block width:auto 收缩 pass 的 grow
/// 补腿——definite/transferred 流内子已布局宽喂 grow（014 主驱）、contain:size
/// 不参与、可换行文本停在可用宽。
mod r5348_inline_block_shrink_grow_tests;

/// R5369 探针：zero-width floats + clear 几何（WPT zero-width-floats 驱动）。
mod r5369_zero_width_floats_clear_tests;
