use super::{
    ComputedStyle, FinalInlineContext, InlineFontContext, InlineFormattingContext, LayoutBox, TextAlign,
    TextGroupAlign, compute_final_inline_layouts, extract_inline_visual_metrics, measure_text_content,
    resolve_text_align, resolve_text_align_last, resolve_text_group_align, resolve_text_indent,
    sync_inline_block_positions_from_ifc, sync_inline_child_boxes_from_ifc, vertical_decoration_free_with_mode,
};
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::Document;
use zero_style_system::property::{
    BorderStyleValue, ColumnCountComputedValue, ColumnFillComputedValue, DirectionValue, LineHeightValue,
    TextAlignLastValue, TextAlignValue, WhiteSpaceValue,
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
        None,
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
        is_run_in_merged: false,
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

#[test]
fn test_resolve_text_group_align_mapping() {
    // R4213（CSS Text 4 #text-group-align-property）：物理值直映射，start/end 方向感知。
    let mut style = ComputedStyle::default();
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::None;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::None);
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::Center;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Center);
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::Left;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Left);
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::Right;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Right);
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::Start;
    style.direction = DirectionValue::Ltr;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Left);
    style.direction = DirectionValue::Rtl;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Right);
    style.text_group_align = zero_style_system::property::TextGroupAlignValue::End;
    style.direction = DirectionValue::Ltr;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Right);
    style.direction = DirectionValue::Rtl;
    assert_eq!(resolve_text_group_align(Some(&style)), TextGroupAlign::Left);
    assert_eq!(resolve_text_group_align(None), TextGroupAlign::None);
}

/// R4297（CSS2 §10.3.1 inline 非原子盒行位）：折行到第二行的 inline span，其 LayoutBox
/// 锚定 IFC 首行盒顶（taffy 把 display:inline 当 block 堆叠 → y=0），同步后 y = 首个
/// fragment 所在行盒的 line.y（>0）。宽 = max(并集, taffy)——不因重写收缩。
#[test]
fn r4297_wrapped_inline_child_anchors_to_first_fragment_line() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    // 前驱文本占满首行 → span 折到第二行起（R4296 border-padding-bleed-001 拓扑）。
    let lead = doc.create_text_node("xx xx xx xx");
    let span = doc.create_element("span");
    let text = doc.create_text_node("yy yy yy yy");
    doc.append_child(container, lead).unwrap();
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(50.0);
    context.layout(&doc, container, &styles);
    assert!(
        context.lines.len() >= 2,
        "前置条件：文本须折行（lines={}）",
        context.lines.len()
    );

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 20.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let child = &root.children[0];
    let span_line_y = context
        .lines
        .iter()
        .find(|line| line.runs.iter().any(|run| run.node_id == span))
        .map(|line| line.y)
        .unwrap();
    assert!(span_line_y > 0.0, "前置条件：span 须折到非首行");
    assert!(
        (child.y - span_line_y).abs() < 0.5,
        "span 盒 y 应取首个 fragment 所在行盒顶 {}（taffy 块堆叠位 0），实际 {}",
        span_line_y,
        child.y
    );
}

/// R4297：前驱 inline 兄弟（文本节点）之后的 span，x 取其 fragment 位（非行首 0）。
#[test]
fn r4297_preceded_by_text_sibling_takes_fragment_x() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let lead = doc.create_text_node("AB");
    let span = doc.create_element("span");
    let text = doc.create_text_node("xx");
    doc.append_child(container, lead).unwrap();
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);
    let frag_x = context
        .all_fragments_with_line_y()
        .into_iter()
        .find(|fragment| fragment.node_id == span)
        .expect("span 扁平文本 fragment")
        .x;
    assert!(frag_x > 0.0, "前置条件：span 前有「AB」文本，fragment x 应 > 0");

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    assert!(
        (root.children[0].x - frag_x).abs() < 0.5,
        "span 盒 x 应取 fragment 位 {}（taffy 块堆叠位 = 行首 0 会覆盖前驱「AB」），实际 {}",
        frag_x,
        root.children[0].x
    );
}

/// R4297：垂直 padding 计入重写后的盒高（CSS border-box 语义），盒 y 上移 padding_top；
/// 替换元素（原子 inline）不走文本路径（其 fragment 可能是扁平化子树文本）。
#[test]
fn r4297_vertical_padding_baked_into_height_and_replaced_skipped() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("xx");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.padding_top = LengthValue::Px(10.0);
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);

    // 替换元素：几何保持 taffy 值（宽不被并集收缩/外扩，y 不移动）。
    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            is_replaced: true,
            x: 5.0,
            y: 7.0,
            width: 30.0,
            height: 20.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    assert_eq!(root.children[0].y, 7.0, "替换元素不走文本路径：y 不动");
    assert_eq!(root.children[0].width, 30.0, "替换元素：宽不动");

    // 非 replaced：padding_top 计入盒高、y 上移 padding_top。
    let mut root2 = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 30.0,
            height: 20.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root2, &context, &styles);
    let child = &root2.children[0];
    let line_y = context.lines[0].y;
    let frag_h = context.all_fragments_with_line_y()[0].height;
    assert!(
        (child.y - (line_y - 10.0)).abs() < 0.5,
        "y = 行盒顶 − padding_top，实际 {}",
        child.y
    );
    assert!(
        (child.height - (frag_h + 10.0)).abs() < 0.5,
        "盒高 = 并集高 + padding_top（{} + 10），实际 {}",
        frag_h,
        child.height
    );
}

// ── slice13：inline 盒 getBoundingClientRect y/h 上报语义 = content area
//（CSS2 §10.6.2/§10.8），dormant 常数锚定，**只记录不上树** ──
// 布局树 y/h 保持行盒几何（流 bookkeeping——R4500 收缩回收/兄弟位移/绘制——全部
// 基于行盒，锚定值入树会串位：r3773 clamp 容器 128→136 实证）。上报值记录在
// `LayoutBox::inline_reported_rect`，由 hit-test/rect 快照层消费（gBCR 单一出口）。

/// slice13 症状面（baidu 导航锚形状，抽取为域名无关最小用例）：inline 元素显式
/// line-height(23px) 下，旧行为上报行盒几何（y=行盒顶、h=行高 23）；Chrome/CSS 语义
/// 是 content area（CSS2 §10.6.2：主字体 A+D，与 line-height 无关）。dormant（无
/// provider）下用引擎常数 0.928/0.236 合成：fs=13 → content h=15.132、y=行顶+5。
/// 负控制：`ZW_INLINE_CONTENT_AREA=0 cargo test slice13` 下 reported=None 本测试显红
///（回到行盒 h=23/y=行顶，偏离 ≥7.87px（h）/5px（y），容差 0.5 → 裕度 ≥10×）。
#[test]
fn slice13_inline_rect_is_content_area_not_line_box() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("更多");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(13.0);
    span_style.line_height = LineHeightValue::Length(LengthValue::Px(23.0));
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 26.0,
            height: 23.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let child = &root.children[0];
    let (line_y, baseline_y) = (context.lines[0].y, context.lines[0].baseline_y);
    // 树回归守卫：布局树保持行盒几何（流 bookkeeping 不变——pivot 语义核心）。
    assert!(
        (child.y - line_y).abs() < 0.5 && (child.height - 23.0).abs() < 0.5,
        "布局树 y/h 须保持行盒几何（y={}, h={}，行顶 {}）",
        child.y,
        child.height,
        line_y
    );
    // 上报语义：content area（baseline − A 顶起，高 = A+D = fs×1.164）。
    let reported = child.inline_reported_rect.expect("单行 inline 应记录上报矩形");
    assert!(
        (reported.1 - 13.0 * (0.928 + 0.236)).abs() < 0.5,
        "上报 h 应为 content area 15.132，不得为行高 23，实际 {}",
        reported.1
    );
    assert!(
        (reported.0 - (line_y + baseline_y - 13.0 * 0.928)).abs() < 0.5,
        "上报 y 应锚 content area 顶（baseline − A），实际 {}（行顶 {} 基线 {}）",
        reported.0,
        line_y,
        baseline_y
    );
}

/// 邻近变体：line-height:normal——content 高 = 16×1.164 = 18.624 与旧行高并集一致
///（高度行为不变），y 从行盒顶收敛到基线锚（half-leading (18.624−16)/2 = 1.312）。
#[test]
fn slice13_inline_rect_normal_line_height_height_preserved() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("More");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(16.0);
    span_style.line_height = LineHeightValue::Normal;
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 30.0,
            height: 18.624,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let child = &root.children[0];
    let (line_y, baseline_y) = (context.lines[0].y, context.lines[0].baseline_y);
    let reported = child
        .inline_reported_rect
        .expect("normal lh 单行 inline 应记录上报矩形");
    assert!(
        (reported.1 - 16.0 * 1.164).abs() < 0.5,
        "lh normal 下上报高 = fs×1.164 = 18.624（与旧并集一致），实际 {}",
        reported.1
    );
    assert!(
        (reported.0 - (line_y + baseline_y - 16.0 * 0.928)).abs() < 0.25,
        "上报 y 应为基线锚（行顶+1.312；容差 0.25 → 负控制裕度 5.2×），实际 {}（行顶 {}）",
        reported.0,
        line_y
    );
}

/// 邻近变体：Ahem 字体——ascent 0.8 / descent 0.2（upem 精确值），content 高 = fs
/// 恰为 13；行高 23 时 content top = (23−13)/2 = 行顶+5。
#[test]
fn slice13_inline_rect_ahem_uses_ahem_ratios() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("xxxx");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(13.0);
    span_style.line_height = LineHeightValue::Length(LengthValue::Px(23.0));
    span_style.font_family = vec!["Ahem".to_string()];
    styles.insert(span, span_style);

    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 52.0,
            height: 23.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let child = &root.children[0];
    let (line_y, baseline_y) = (context.lines[0].y, context.lines[0].baseline_y);
    let reported = child.inline_reported_rect.expect("Ahem 单行 inline 应记录上报矩形");
    assert!(
        (reported.1 - 13.0).abs() < 0.5,
        "Ahem 上报高 = fs = 13（0.8+0.2），实际 {}",
        reported.1
    );
    assert!(
        (reported.0 - (line_y + baseline_y - 13.0 * 0.8)).abs() < 0.5,
        "Ahem 上报顶 = baseline − 0.8fs，实际 {}（行顶 {} 基线 {}）",
        reported.0,
        line_y,
        baseline_y
    );
}

/// 边界臂：同线嵌套不同字号——span 自身 13px 片段（「嵌」「套」）与 b(28px) 同线，
/// span 的上报 content area 仍按 **span 自身字体**（Chrome v7 实证 h=15 不含子字号）；
/// b 按 28px 字体上报（h=32.592）。树几何均保持行盒并集。
#[test]
fn slice13_same_line_nested_font_sizes_anchor_content_area() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let t1 = doc.create_text_node("嵌");
    let b = doc.create_element("b");
    let t2 = doc.create_text_node("Nest");
    let t3 = doc.create_text_node("套");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, t1).unwrap();
    doc.append_child(span, b).unwrap();
    doc.append_child(b, t2).unwrap();
    doc.append_child(span, t3).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(13.0);
    span_style.line_height = LineHeightValue::Length(LengthValue::Px(23.0));
    styles.insert(span, span_style);
    let mut b_style = ComputedStyle::default();
    b_style.display = DisplayValue::Inline;
    b_style.font_size = LengthValue::Px(28.0);
    b_style.line_height = LineHeightValue::Length(LengthValue::Px(23.0));
    styles.insert(b, b_style);

    let mut context = InlineFormattingContext::new(400.0);
    context.layout(&doc, container, &styles);

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![
            LayoutBox {
                node_id: Some(span),
                x: 0.0,
                y: 0.0,
                width: 83.0,
                height: 23.0,
                ..LayoutBox::default()
            },
            LayoutBox {
                node_id: Some(b),
                x: 13.0,
                y: 0.0,
                width: 57.0,
                height: 23.0,
                ..LayoutBox::default()
            },
        ],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let (line_y, baseline_y) = (context.lines[0].y, context.lines[0].baseline_y);
    let span_child = &root.children[0];
    let span_reported = span_child.inline_reported_rect.expect("同线嵌套 span 应记录上报矩形");
    assert!(
        (span_reported.1 - 13.0 * (0.928 + 0.236)).abs() < 0.5,
        "同线嵌套字号 span 上报高按自身字体 15.132（不含子字号），实际 {}",
        span_reported.1
    );
    assert!(
        (span_reported.0 - (line_y + baseline_y - 13.0 * 0.928)).abs() < 0.5,
        "span 上报顶 = 行基线 − 0.928×13，实际 {}（基线 {}）",
        span_reported.0,
        baseline_y
    );
    let b_child = &root.children[1];
    assert!(
        b_child.inline_reported_rect.is_none(),
        "b 的上报在其自身容器（span）walk 落定，容器级 sync 不越级记录，实际 {:?}",
        b_child.inline_reported_rect
    );
    // b 的几何由 span 作为容器时的 sync 落定（管线两级 walk）——此处对 span 作用域
    // 重放同一 sync。
    let mut span_ctx = InlineFormattingContext::new(400.0);
    span_ctx.layout(&doc, span, &styles);
    let mut span_root = LayoutBox {
        node_id: Some(span),
        children: vec![LayoutBox {
            node_id: Some(b),
            x: 13.0,
            y: 0.0,
            width: 57.0,
            height: 23.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut span_root, &span_ctx, &styles);
    let b_reported = span_root.children[0].inline_reported_rect.expect("b 应记录上报矩形");
    let b_line = &span_ctx.lines[0];
    assert!(
        (b_reported.1 - 28.0 * (0.928 + 0.236)).abs() < 0.5,
        "b 上报高 = 28×1.164 = 32.592，实际 {}",
        b_reported.1
    );
    assert!(
        (b_reported.0 - (b_line.y + b_line.baseline_y - 28.0 * 0.928)).abs() < 0.5,
        "b 上报顶 = 行基线 − 0.928×28，实际 {}（基线 {}）",
        b_reported.0,
        b_line.baseline_y
    );
}

/// 挂账臂：跨行 wrap 的 inline（多片段 union）语义不变——不记录上报矩形
///（快照回退行盒并集几何），树几何亦保持并集。多片段 content area 并集
///（Chrome: 各片段 content area 的并包盒）待后续切片。
#[test]
fn slice13_multiline_wrapped_inline_keeps_union_path() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("嵌套套套套套套套套");
    doc.append_child(container, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(30.0);
    span_style.line_height = LineHeightValue::Normal;
    styles.insert(span, span_style);

    // 200px 宽 + 30px 字号 → CJK 逐字 30px 宽必折行（多行）。
    let mut context = InlineFormattingContext::new(200.0);
    context.layout(&doc, container, &styles);
    assert!(context.lines.len() >= 2, "用例前提：文本折为多行");

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(span),
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 46.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    sync_inline_child_boxes_from_ifc(&mut root, &context, &styles);
    let child = &root.children[0];
    assert!(
        child.inline_reported_rect.is_none(),
        "跨行 wrap inline 不记录上报矩形（并集语义挂账），实际 {:?}",
        child.inline_reported_rect
    );
    assert!(
        (child.y - context.lines[0].y).abs() < 0.5,
        "树 y 保持首行行盒顶（union 旧行为），实际 {}（首行顶 {}）",
        child.y,
        context.lines[0].y
    );
}
