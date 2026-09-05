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
