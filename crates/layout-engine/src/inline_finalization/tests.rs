use super::{
    ComputedStyle, FinalInlineContext, InlineFontContext, InlineFormattingContext, LayoutBox, TextAlign,
    compute_final_inline_layouts, extract_inline_visual_metrics, measure_text_content, resolve_text_align,
    resolve_text_align_last, resolve_text_indent, sync_inline_block_positions_from_ifc,
    vertical_decoration_free_with_mode,
};
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::Document;
use zero_style_system::property::{
    BorderStyleValue, ColumnCountComputedValue, ColumnFillComputedValue, DirectionValue, TextAlignLastValue,
    TextAlignValue, WhiteSpaceValue,
};

#[test]
fn test_resolve_text_align_start_end_direction_aware() {
    let mut style = ComputedStyle::default();
    style.direction = DirectionValue::Ltr;
    style.text_align = TextAlignValue::Start;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    style.text_align = TextAlignValue::End;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
    style.text_align = TextAlignValue::Left;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    style.direction = DirectionValue::Rtl;
    style.text_align = TextAlignValue::Start;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
    style.text_align = TextAlignValue::End;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    assert_eq!(resolve_text_align(None), TextAlign::Left);
}

#[test]
fn test_resolve_text_align_last_mapping() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Auto;
    assert_eq!(resolve_text_align_last(Some(&style)), None);
    style.text_align_last = TextAlignLastValue::Justify;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Justify));
    style.text_align_last = TextAlignLastValue::Right;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.text_align_last = TextAlignLastValue::Center;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Center));
    style.text_align_last = TextAlignLastValue::Left;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
    assert_eq!(resolve_text_align_last(None), None);
    style.direction = DirectionValue::Ltr;
    style.text_align_last = TextAlignLastValue::Start;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
    style.text_align_last = TextAlignLastValue::End;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.direction = DirectionValue::Rtl;
    style.text_align_last = TextAlignLastValue::Start;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.text_align_last = TextAlignLastValue::End;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
}

#[test]
fn test_resolve_text_indent_px_em_percentage() {
    assert_eq!(
        resolve_text_indent(&LengthValue::Px(40.0), &LengthValue::Px(16.0), 800.0),
        40.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Em(5.0), &LengthValue::Px(16.0), 800.0),
        80.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Percentage(50.0), &LengthValue::Px(16.0), 800.0),
        400.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Auto, &LengthValue::Px(16.0), 800.0),
        0.0
    );
}

#[test]
fn test_resolve_text_indent_relative_lengths() {
    assert_eq!(
        resolve_text_indent(&LengthValue::Ch(4.0), &LengthValue::Px(20.0), 800.0),
        40.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Rem(2.0), &LengthValue::Px(20.0), 800.0),
        32.0
    );
}

#[test]
fn test_extract_inline_visual_metrics_relative_lengths() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.padding_left = LengthValue::Em(1.0);
    style.padding_right = LengthValue::Ch(2.0);
    style.border_right_width = LengthValue::Em(0.5);
    style.border_right_style = BorderStyleValue::Solid;

    let metrics = extract_inline_visual_metrics(&style);

    assert_eq!(metrics.padding_left, 20.0);
    assert_eq!(metrics.padding_right, 20.0);
    assert_eq!(metrics.border_right, 10.0);
}

/// R4007（CSS §8.5.3）：border-style = none/hidden 时该边 border-width 计算为 0——
/// computed border-width 初始 = medium(3px)，不抑制则 sync_inline_child_boxes 把幻影
/// 3px 边框写入 display:Inline 的替换元素盒（007-ref svg 784×392 膨成 790×398 @ y=-3）。
#[test]
fn r4007_border_style_none_suppresses_width_in_inline_metrics() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    // 默认 border-width = medium(3px)，style 缺省 = None。
    let metrics = extract_inline_visual_metrics(&style);
    assert_eq!(metrics.border_top, 0.0);
    assert_eq!(metrics.border_right, 0.0);
    assert_eq!(metrics.border_bottom, 0.0);
    assert_eq!(metrics.border_left, 0.0);

    // 显式宽度 + hidden 同样归零；solid 则保留。
    style.border_top_width = LengthValue::Px(10.0);
    style.border_top_style = BorderStyleValue::Hidden;
    style.border_bottom_width = LengthValue::Px(10.0);
    style.border_bottom_style = BorderStyleValue::Solid;
    let metrics = extract_inline_visual_metrics(&style);
    assert_eq!(metrics.border_top, 0.0, "hidden 边宽度计 0");
    assert_eq!(metrics.border_bottom, 10.0, "solid 边宽度保留");
}

/// R3625：空叶节点测量回退到 CSS width/height 时，也要解析 residual real length。
#[test]
fn r3625_empty_leaf_measure_resolves_residual_explicit_size() {
    use taffy::geometry::Size;
    use taffy::style::AvailableSpace;

    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.width = LengthValue::Em(5.0);
    style.height = LengthValue::Ch(4.0);
    styles.insert(div, style);

    let size = measure_text_content(
        &doc,
        &styles,
        div,
        Size {
            width: None,
            height: None,
        },
        Size {
            width: AvailableSpace::Definite(800.0),
            height: AvailableSpace::Definite(600.0),
        },
        &HashMap::new(),
        Default::default(),
    );

    assert!(
        (size.width - 100.0).abs() < 0.01,
        "empty leaf width:5em should resolve against font-size:20px, got {}",
        size.width
    );
    assert!(
        (size.height - 40.0).abs() < 0.01,
        "empty leaf height:4ch should resolve against font-size:20px, got {}",
        size.height
    );
}

/// R3626：inline-only multicol column-fill:auto 的列高预算也要解析 residual real length。
#[test]
fn r3626_multicol_auto_fill_resolves_residual_height_budget() {
    use zero_dom::parse_html;

    let doc = parse_html("<div>aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa aa</div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_fill = ColumnFillComputedValue::Auto;
    style.column_gap = LengthValue::Px(0.0);
    style.font_size = LengthValue::Px(20.0);
    style.height = LengthValue::Em(5.0);
    styles.insert(div, style);

    let mut layout_box = LayoutBox {
        node_id: Some(div),
        width: 200.0,
        height: 1000.0,
        content_width: 200.0,
        content_height: 1000.0,
        is_multicol: true,
        is_block_level: true,
        ..Default::default()
    };
    let mut paint_skip = std::collections::HashSet::new();
    let finalized_inline_blocks = Default::default();
    let mut context = FinalInlineContext::new(&mut paint_skip, InlineFontContext::default(), &finalized_inline_blocks);

    compute_final_inline_layouts(&mut layout_box, &doc, &styles, &[], &HashMap::new(), &mut context);

    let lines = layout_box
        .inline_layout
        .as_ref()
        .expect("multicol auto-fill should store fragmented inline layout");
    let column_two_x = lines
        .iter()
        .flat_map(|line| line.fragments.iter().map(|fragment| fragment.x))
        .fold(0.0_f32, f32::max);
    assert!(
        column_two_x >= 100.0,
        "height:5em at 20px should use a 100px column budget and move overflow lines to column 2, max x={}",
        column_two_x
    );
}

/// R3627：CSS `tab-size:<length>` 是实际 tab stop 长度，不是空格倍数。
#[test]
fn r3627_tab_size_length_resolves_to_px_stop_width() {
    use zero_dom::parse_html;

    let doc = parse_html("<div>a\tx</div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_family = vec!["Ahem".to_string()];
    style.font_size = LengthValue::Px(20.0);
    style.white_space = WhiteSpaceValue::PreWrap;
    style.tab_size = zero_style_system::TabSizeValue::Length(LengthValue::Em(2.0));
    styles.insert(div, style);

    let mut layout_box = LayoutBox {
        node_id: Some(div),
        width: 800.0,
        content_width: 800.0,
        is_block_level: true,
        ..Default::default()
    };
    let mut paint_skip = std::collections::HashSet::new();
    let finalized_inline_blocks = Default::default();
    let mut context = FinalInlineContext::new(&mut paint_skip, InlineFontContext::default(), &finalized_inline_blocks);

    compute_final_inline_layouts(&mut layout_box, &doc, &styles, &[], &HashMap::new(), &mut context);

    let lines = layout_box
        .inline_layout
        .as_ref()
        .expect("pre-wrap text should store inline layout");
    let x_pos = lines[0]
        .fragments
        .iter()
        .find(|fragment| fragment.text.contains('x'))
        .map(|fragment| fragment.x)
        .expect("line should contain x fragment");
    assert!(
        (x_pos - 40.0).abs() < 0.01,
        "tab-size:2em at font-size:20px should create 40px tab stops, got x={}",
        x_pos
    );
}

#[test]
fn horizontal_decoration_gate_skips_subtree_scan() {
    let scans = std::cell::Cell::new(0);
    assert!(vertical_decoration_free_with_mode(true, false, || {
        scans.set(scans.get() + 1);
        true
    }));
    assert_eq!(scans.get(), 0);

    assert!(!vertical_decoration_free_with_mode(true, true, || {
        scans.set(scans.get() + 1);
        true
    }));
    assert_eq!(scans.get(), 1);
}

#[test]
fn inline_block_position_reuse_is_complete_and_fail_closed() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let text = doc.create_text_node("prefix");
    let inline_block = doc.create_element("span");
    doc.append_child(container, text).unwrap();
    doc.append_child(container, inline_block).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut inline_block_style = ComputedStyle::default();
    inline_block_style.display = DisplayValue::InlineBlock;
    styles.insert(inline_block, inline_block_style);

    let mut sizes = HashMap::new();
    sizes.insert(inline_block, (40.0, 2.0));
    let mut context = InlineFormattingContext::new(200.0).with_inline_block_sizes(sizes);
    context.layout(&doc, container, &styles);
    let stale_y = context
        .all_fragments_with_line_y()
        .into_iter()
        .find(|fragment| fragment.node_id == inline_block)
        .unwrap()
        .y;

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(inline_block),
            width: 40.0,
            height: 25.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    let mut final_sizes = HashMap::new();
    final_sizes.insert(inline_block, (40.0, 25.0));
    assert!(context.refresh_reused_inline_block_metrics(&doc, &styles, &final_sizes));
    assert!(sync_inline_block_positions_from_ifc(&mut root, &context, &doc, &styles));
    assert!(root.children[0].x > 0.0);
    assert!(root.children[0].y < stale_y);

    styles.get_mut(&inline_block).unwrap().display = DisplayValue::InlineFlex;
    assert!(!context.refresh_reused_inline_block_metrics(&doc, &styles, &final_sizes));
    assert!(!sync_inline_block_positions_from_ifc(
        &mut root, &context, &doc, &styles
    ));
}

/// R3991（CSS Display 3 §2.3 run-in box）：并入 run-in 的容器 IFC 前置收集 run-in 的
/// inline 内容（首行开头按 run-in 元素自身扁平化 node_id 记），且 run-in 代理盒回填 +
/// paint_skip 登记（自身无 taffy 盒，文本由本容器 IFC 绘制）。
#[test]
fn r3991_run_in_prepended_collects_run_in_content_first() {
    use zero_dom::parse_html;

    // <div id=runin>Run-in header</div><div id=target>Start</div>
    // run-in 并入 target 首行：target 的 IFC 首片段 = run-in 元素（node_id=run-in）。
    let doc = parse_html("<div><div id=\"runin\">Run-in header</div><div id=\"target\">Start</div></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let children = doc.child_nodes(outer);
    let run_in = children[0];
    let target = children[1];

    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    run_in_style.font_family = vec!["Ahem".to_string()];
    run_in_style.font_size = LengthValue::Px(20.0);
    styles.insert(run_in, run_in_style);
    let mut target_style = ComputedStyle::default();
    target_style.display = DisplayValue::Block;
    target_style.font_family = vec!["Ahem".to_string()];
    target_style.font_size = LengthValue::Px(20.0);
    styles.insert(target, target_style);

    let mut layout_box = LayoutBox {
        node_id: Some(target),
        width: 800.0,
        content_width: 800.0,
        is_block_level: true,
        // build_subtree 注册（后继块视角）：run-in 元素并入本容器首行。
        run_in_prepended: Some(run_in),
        ..Default::default()
    };
    let mut paint_skip = std::collections::HashSet::new();
    let finalized_inline_blocks = Default::default();
    let mut context = FinalInlineContext::new(&mut paint_skip, InlineFontContext::default(), &finalized_inline_blocks);

    compute_final_inline_layouts(&mut layout_box, &doc, &styles, &[], &HashMap::new(), &mut context);

    let lines = layout_box
        .inline_layout
        .as_ref()
        .expect("run-in merge container should store inline layout");
    let first = lines[0].fragments.first().expect("first line should have fragments");
    // 前置收集：首片段 = run-in 的文本子（node_id = 文本节点，其父 = run-in 元素），
    // 而非 target 自身的 "Start" 文本。
    let first_parent = first.node_id.and_then(|id| doc.parent_node(id));
    assert_eq!(
        first_parent,
        Some(run_in),
        "run-in inline content should be prepended to the first line"
    );
    assert!(
        first.text.contains("Run-in"),
        "first fragment should be the run-in text, got {:?}",
        first.text
    );
    // run-in 代理盒回填（hit-test 可见）+ paint_skip 登记（防双绘）。
    let run_in_box = layout_box
        .children
        .iter()
        .find(|c| c.node_id == Some(run_in))
        .expect("run-in proxy box should be backfilled");
    assert!(run_in_box.width > 0.0, "run-in proxy box should cover its text");
    assert!(paint_skip.contains(&run_in), "run-in should be paint-skipped");
}

/// R3991：run-in 判定——前驱无块级兄弟且后继为块级 → 并入；后继为 inline / 前驱有块级 /
/// 无后继 / 自身含块级子 → 不并入（返回 None，降级普通块盒）。
#[test]
fn r3991_run_in_sibling_predicate() {
    use zero_dom::parse_html;

    use crate::tree::run_in_following_block_sibling;

    // 并入形态：<div class=run-in>..</div><div>..</div>
    let doc = parse_html("<div><div id=\"r\">Run-in</div><div id=\"t\">Block</div></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let r = doc.child_nodes(outer)[0];
    let t = doc.child_nodes(outer)[1];
    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    styles.insert(r, run_in_style);
    // 后继块须有样式条目（生产 styles 覆盖全元素；无条目 = 未知 display，不判块）。
    let mut target_style = ComputedStyle::default();
    target_style.display = DisplayValue::Block;
    styles.insert(t, target_style);
    assert_eq!(run_in_following_block_sibling(&doc, &styles, r), Some(t));

    // 前驱块级兄弟 → 不并入（spec fallback）。
    let doc = parse_html("<div><div id=\"pre\">Pre</div><div id=\"r\">Run-in</div><div id=\"t\">Block</div></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let r = doc.child_nodes(outer)[1];
    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    styles.insert(r, run_in_style);
    assert_eq!(run_in_following_block_sibling(&doc, &styles, r), None);

    // 后继 inline → 不并入。
    let doc = parse_html("<div><div id=\"r\">Run-in</div><span id=\"t\">Inline</span></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let r = doc.child_nodes(outer)[0];
    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    styles.insert(r, run_in_style);
    assert_eq!(run_in_following_block_sibling(&doc, &styles, r), None);

    // 无后继 → 不并入。
    let doc = parse_html("<div><div id=\"r\">Run-in</div></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let r = doc.child_nodes(outer)[0];
    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    styles.insert(r, run_in_style);
    assert_eq!(run_in_following_block_sibling(&doc, &styles, r), None);

    // run-in 自身含块级子 → 降级（不并入）。
    let doc = parse_html("<div><div id=\"r\">Run-in<div></div></div><div id=\"t\">Block</div></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let outer = doc.first_child(body).unwrap();
    let r = doc.child_nodes(outer)[0];
    let mut styles = HashMap::new();
    let mut run_in_style = ComputedStyle::default();
    run_in_style.display = DisplayValue::RunIn;
    styles.insert(r, run_in_style);
    assert_eq!(run_in_following_block_sibling(&doc, &styles, r), None);
}
