//! CSS contain / unicode-bidi / box-decoration-break / overflow-wrap / text-align-last /
//! break / scroll-area / scroll-snap-stop / container-type 渲染指示器单元测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_dom::Document;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::*;

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<zero_dom::NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: 0.0,
        content_y: 0.0,
        content_width: width,
        content_height: height,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

/// 创建指定样式的 Painter 并渲染一个节点。
fn paint_with_style(style: &ComputedStyle) -> Painter {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    styles.insert(elem, style.clone());

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    painter
}

// ──────────────────────────────────────────────────────
// CSS contain 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_contain_none_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    // contain: None 是默认值，不应产生额外填充
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_contain_strict_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Strict;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_paint_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Paint;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_content_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Content;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_size_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_layout_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Layout;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS unicode-bidi 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_unicode_bidi_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_unicode_bidi_embed_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Embed;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_isolate_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Isolate;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_bidi_override_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::BidiOverride;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_plaintext_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Plaintext;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS box-decoration-break 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_box_decoration_break_slice_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_box_decoration_break_clone_indicator() {
    let mut style = ComputedStyle::default();
    style.box_decoration_break = BoxDecorationBreakValue::Clone;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS overflow-wrap 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_overflow_wrap_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_overflow_wrap_break_word_indicator() {
    let mut style = ComputedStyle::default();
    style.overflow_wrap = OverflowWrapValue::BreakWord;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_overflow_wrap_anywhere_indicator() {
    let mut style = ComputedStyle::default();
    style.overflow_wrap = OverflowWrapValue::Anywhere;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS text-align-last 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_text_align_last_auto_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_text_align_last_center_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Center;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_justify_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Justify;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_right_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Right;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_left_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Left;
    let painter = paint_with_style(&style);
    // Left 映射到 1 条横线指示器
    assert!(painter.primitives().fills.len() >= 1);
}

// ──────────────────────────────────────────────────────
// CSS break 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_break_all_auto_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_break_before_column_indicator() {
    let mut style = ComputedStyle::default();
    style.break_before = BreakValue::Column;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_break_after_page_indicator() {
    let mut style = ComputedStyle::default();
    style.break_after = BreakValue::Page;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_break_inside_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.break_inside = BreakInsideValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_before_always_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_before = PageBreakValue::Always;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_after_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_after = PageBreakValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_inside_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_inside = PageBreakValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS scroll-margin/padding 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_scroll_margin_zero_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_scroll_margin_top_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 10.0;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_margin_all_sides_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 5.0;
    style.scroll_margin_right = 5.0;
    style.scroll_margin_bottom = 5.0;
    style.scroll_margin_left = 5.0;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_padding_length_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_padding_top = ScrollPadding::Length(8.0);
    style.scroll_padding_bottom = ScrollPadding::Length(8.0);
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_padding_auto_no_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_padding_top = ScrollPadding::Auto;
    style.scroll_padding_bottom = ScrollPadding::Auto;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

// ──────────────────────────────────────────────────────
// CSS scroll-snap-stop 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_scroll_snap_stop_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_scroll_snap_stop_always_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_snap_stop = ScrollSnapStop::Always;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS container-type 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_container_type_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_container_type_size_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_container_type_inline_size_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::InlineSize;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_container_type_with_name_extra_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::Size;
    style.container_name = Some("sidebar".to_string());
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// 组合测试
// ──────────────────────────────────────────────────────

#[test]
fn test_contain_plus_unicode_bidi_combined() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Strict;
    style.unicode_bidi = UnicodeBidiValue::BidiOverride;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_break_before_plus_after_combined() {
    let mut style = ComputedStyle::default();
    style.break_before = BreakValue::Page;
    style.break_after = BreakValue::Column;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_scroll_margin_plus_padding_combined() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 5.0;
    style.scroll_margin_bottom = 5.0;
    style.scroll_padding_top = ScrollPadding::Length(8.0);
    style.scroll_padding_bottom = ScrollPadding::Length(8.0);
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_all_indicators_combined() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Layout;
    style.overflow_wrap = OverflowWrapValue::BreakWord;
    style.text_align_last = TextAlignLastValue::Center;
    style.unicode_bidi = UnicodeBidiValue::Embed;
    style.box_decoration_break = BoxDecorationBreakValue::Clone;
    style.break_before = BreakValue::Column;
    style.scroll_margin_top = 5.0;
    style.scroll_snap_stop = ScrollSnapStop::Always;
    style.container_type = ContainerType::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 10);
}

/// R3933（CSS2 replaced elements + SVG2）：inline `<svg>` paint 栅格化。
/// R3990 kill-switch 放开后默认生效：svg 元素产 ImagePrimitive +
/// canvas_images 像素（canvas 同款两段式通路）。
#[test]
fn r3933_inline_svg_paint_rasterizes() {
    let html = r#"<html><body style="margin:0"><div style="position: relative; width: 200px; height: 100px;"><svg width="100" height="50" xmlns="http://www.w3.org/2000/svg" style="position: absolute; left: 0; top: 0; width: 100px; height: 50px;"><rect width="100" height="50" fill="blue"/></svg></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let svg_id = doc
        .get_elements_by_tag_name("svg")
        .into_iter()
        .next()
        .expect("svg element");

    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, c))
    }
    let box_node = find(svg_id, &result.root).expect("svg box");

    // 默认开：1 图元 + canvas_images 像素（100x50 盒）。
    let mut painter = Painter::new();
    let style = ComputedStyle::default();
    painter.paint_svg_element(box_node, 0.0, 0.0, &doc, &styles);
    assert_eq!(painter.primitives().images.len(), 1, "默认开应产 1 个 svg 图元");
    assert_eq!(painter.canvas_images.len(), 1, "canvas_images 应携带栅格化像素");
    let (key, w, h, rgba) = &painter.canvas_images[0];
    assert_eq!((*w, *h), (100, 50), "栅格化尺寸应等于盒尺寸");
    assert!(!rgba.is_empty(), "像素非空");
    assert_ne!(*key, 0, "哈希键非零");
}

/// R3938（CSS Transforms 1 §transform-attribute-specificity）：stylesheet
/// transform 在 svg paint 序列化时级联合成——document-styles-001 形态
/// （`.testRect{transform:rotate(90deg)}` 覆盖 `transform="scale(0.5)"`）。
#[test]
fn r3938_stylesheet_transform_composed_into_serialized_svg() {
    let html = r##"<html><head><style>
        svg { display: block; width: 300px; height: 300px; }
        rect.testRect { transform: rotate(90deg); }
    </style></head><body style="margin:0">
    <svg><rect class="testRect" y="-100" width="100" height="100" fill="#00ff00" transform="scale(0.5)"/></svg>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "svg { display: block; width: 300px; height: 300px; } rect.testRect { transform: rotate(90deg); }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let rect_id = doc.get_elements_by_tag_name("rect").into_iter().next().expect("rect");
    assert!(
        matches!(
            styles.get(&rect_id).map(|s| &s.transform),
            Some(zero_css_parser::values::TransformValue::List(_))
        ),
        "stylesheet transform 应入 computed style"
    );
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, c))
    }
    let box_node = find(svg_id, &result.root).expect("svg box");
    let mut painter = Painter::new();
    painter.paint_svg_element(box_node, 0.0, 0.0, &doc, &styles);
    // 合成后栅格化应成功且像素与「attr=rotate(90)」形态一致——用 canvas_images
    // 内容对照纯 attr 版序列化渲染。
    assert_eq!(painter.canvas_images.len(), 1, "应有栅格化像素");
    let expect = zero_render_foundation::image_cache::rasterize_svg_at(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300"><rect y="-100" width="100" height="100" fill="#00ff00" transform="rotate(90)"/></svg>"##,
        300, 300,
    ).expect("expect");
    let (_, _, _, px) = &painter.canvas_images[0];
    let diff = px
        .chunks(4)
        .zip(expect.pixels.chunks(4))
        .filter(|(p, q)| p != q)
        .count();
    assert_eq!(
        diff, 0,
        "stylesheet transform 应覆盖 attr（合成后与 attr=rotate(90) 一致，diff={diff}）"
    );
}

/// R3987（R3986 修复收口锚，CSS Display 3 §2.4 atomic inlines）：inline svg
/// （replaced 类）的 CSS width 应用**不得依赖子树内空白文本节点**——R3986
/// 两态差异（200x200 vs 6x24）已由 collect_items 的 replaced-inline 原子化修复。
#[test]
fn r3986_anchor_replaced_inline_width_ws_independent() {
    let mk = |svg_inner: &str| {
        format!(
            r##"<html><head><style>svg {{ width: 200px; height: 200px; background: green; }}</style></head><body style="margin:0"><svg>{svg_inner}</svg></body></html>"##
        )
    };
    let doc_of = |html: &str| {
        let doc = zero_dom::parse_html(html);
        let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
        let sheet =
            zero_css_parser::Parser::parse_stylesheet("svg { width: 200px; height: 200px; background: green; }");
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[sheet]);
        let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
            if b.node_id == Some(id) {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, c))
        }
        find(svg_id, &result.root).map(|b| (b.width, b.height))
    };
    let with_ws = mk(
        "\n  <rect x=\"1\" y=\"1\" width=\"196\" height=\"196\" fill=\"red\" transform=\"scale(0.5)\"/>\n  <rect width=\"100\" height=\"100\" fill=\"green\"/>\n",
    );
    let without_ws = mk(
        "<rect x=\"1\" y=\"1\" width=\"196\" height=\"196\" fill=\"red\" transform=\"scale(0.5)\"/><rect width=\"100\" height=\"100\" fill=\"green\"/>",
    );
    let ws = doc_of(&with_ws).expect("ws box");
    let no_ws = doc_of(&without_ws).expect("no-ws box");
    assert_eq!(
        ws, no_ws,
        "修复后两态应一致（replaced inline 原子化——空白文本节点不再影响盒尺寸）"
    );
    assert!(ws.0 >= 200.0, "CSS width:200 应应用（svg 盒宽 ≥200）：{ws:?}");
}

/// R4104（CSS Transforms 1 §transform-box svg 豁免）：inline `<svg>` 子树内元素的
/// CSS transform 不得再走 html TransformPrimitive / 图元级平移通路——R3938 位图路径
/// 已把同一声明合成进序列化源（SVG 语义），双路消费 = 位图二次变换（svgbox-initial：
/// path rotate(90deg) 位图内已生效，外层 TransformPrimitive ty=-100 再平移 → 黑块
/// y 偏 -100）。锚：子树内带 CSS transform 元素的 paint 不产生 TransformPrimitive、
/// 不平移图元；html 侧同形态不受影响。
#[test]
fn r4104_svg_subtree_css_transform_exempt_from_html_transform_path() {
    let html = r##"<html><head><style>
        svg { display: block; width: 400px; height: 300px; }
        #target { transform: rotate(90deg); }
    </style></head><body style="margin:0">
    <svg><path id="target" d="M 200 100 v 100 h 100 v -100" fill="green"/></svg>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "svg { display: block; width: 400px; height: 300px; } #target { transform: rotate(90deg); }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, c))
    }
    let svg_box = find(svg_id, &result.root).expect("svg box");

    // svg 子树整体 paint（path 的 CSS transform 由 R3938 位图路径消费）：无 TransformPrimitive。
    let mut painter = Painter::new();
    painter.paint_svg_element(svg_box, 8.0, 8.0, &doc, &styles);
    assert!(
        painter.primitives().transforms.is_empty(),
        "svg 位图路径不应产 TransformPrimitive（R3938 语义）"
    );

    // 全页 paint：svg 子树内元素的 CSS transform 不得再生成 TransformPrimitive（双重消费）。
    let mut painter_full = Painter::new();
    painter_full.paint(&result.root, &styles, Some(&doc));
    assert!(
        painter_full.primitives().transforms.is_empty(),
        "全页 paint 时 svg 子树 CSS transform 须豁免 html transform 通路（R4104 gate）"
    );

    // html 侧对照：同样式的 div 仍走 html transform 通路（gate 只豁免 svg 子树）。
    let html_div = r##"<html><head><style>
        div { width: 100px; height: 100px; background: green; transform: rotate(90deg); }
    </style></head><body style="margin:0"><div></div></body></html>"##;
    let doc2 = zero_dom::parse_html(html_div);
    let sheet2 = zero_css_parser::Parser::parse_stylesheet(
        "div { width: 100px; height: 100px; background: green; transform: rotate(90deg); }",
    );
    let mut sys2 = zero_style_system::StyleSystem::new();
    sys2.set_viewport(800.0, 600.0);
    let styles2 = sys2.compute_styles(&doc2, &[sheet2]);
    let mut engine2 = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result2 = engine2.compute(&doc2, &styles2);
    let mut painter2 = Painter::new();
    painter2.paint(&result2.root, &styles2, Some(&doc2));
    assert_eq!(
        painter2.primitives().transforms.len(),
        1,
        "html 侧 transform 不受 svg 豁免 gate 影响"
    );
}

/// R4105（CSS Transforms 1 §transform-box/§transform-origin）：transform-origin 相对
/// **参考框左上角**——注入 SVG attr 的旋转中心须加参考框原点偏移。svgbox-stroke-box-001
/// 形态（rect 100,100,100,50 + stroke 20 → stroke-box (90,90,140,70) + origin 20,0 +
/// rotate 90deg）：旋转中心应为用户坐标 (110,90)，注入 translate(110 90) rotate(90)
/// translate(-110 -90)；旧实现漏加框偏移（旋转中心落 (20,0)）→ 图形旋转出画布全白。
#[test]
fn r4105_svg_transform_origin_offset_by_reference_box_origin() {
    let html = r##"<html><head><style>
        svg { display: block; width: 400px; height: 300px; }
        #target {
            fill: green; stroke: black; stroke-width: 20px;
            transform-box: stroke-box; transform-origin: 20px 0px;
            transform: rotate(90deg);
        }
    </style></head><body style="margin:0">
    <svg width="400" height="300"><rect id="target" width="100" height="50" x="100" y="100"/></svg>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "svg { display: block; width: 400px; height: 300px; } #target { fill: green; stroke: black; stroke-width: 20px; transform-box: stroke-box; transform-origin: 20px 0px; transform: rotate(90deg); }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut css_transforms: Vec<(zero_dom::NodeId, String)> = Vec::new();
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    crate::paint::painter::collect_css_transforms(&doc, svg_id, &styles, &mut css_transforms);
    assert_eq!(css_transforms.len(), 1, "rect 的 CSS transform 应被收集");
    let (_, svg_attr) = &css_transforms[0];
    assert!(
        svg_attr.contains("translate(110 90)")
            && svg_attr.contains("rotate(90)")
            && svg_attr.contains("translate(-110 -90)"),
        "origin 20,0 + stroke-box 原点 (90,90) 应合成旋转中心 (110,90)，got: {svg_attr}"
    );
}

/// R4106（CSS Transforms 1 §transform-box 参考框覆盖扩展）：svg_element_bbox 补
/// path 直线族 d 属性 bbox + g/a 容器子形状并集。锚：① path
/// "M 200 100 v 100 h 100 v -100" → bbox (200,100,100,100)，fill-box origin 0,0
/// + rotate 90 → 旋转中心 (200,100) 注入；② 曲线命令 path 返回 None（宁缺勿错）；
///  - ③ g 容器 = 子 rect 并集。
#[test]
fn r4106_svg_element_bbox_path_and_container() {
    let html = r##"<html><head><style>
        svg { display: block; width: 400px; height: 300px; }
        #target {
            fill: green; stroke: black; stroke-width: 50px;
            transform-box: fill-box; transform-origin: 0px 0px;
            transform: rotate(90deg);
        }
    </style></head><body style="margin:0">
    <svg width="400" height="300"><path id="target" d="M 200 100 v 100 h 100 v -100"/></svg>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "svg { display: block; width: 400px; height: 300px; } #target { fill: green; stroke: black; stroke-width: 50px; transform-box: fill-box; transform-origin: 0px 0px; transform: rotate(90deg); }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let mut css_transforms: Vec<(zero_dom::NodeId, String)> = Vec::new();
    crate::paint::painter::collect_css_transforms(&doc, svg_id, &styles, &mut css_transforms);
    assert_eq!(css_transforms.len(), 1, "path 的 CSS transform 应被收集");
    let (_, svg_attr) = &css_transforms[0];
    // path bbox (200,100,100,100)，origin 0,0 → 旋转中心 = bbox 原点 (200,100)。
    assert!(
        svg_attr.contains("translate(200 100)"),
        "path fill-box bbox 原点 (200,100) 应作旋转中心，got: {svg_attr}"
    );

    // 曲线命令 path → bbox None → 无 ref_box → origin 偏移不注入（transform 本身仍直译）。
    let doc2 = zero_dom::parse_html(
        r#"<html><head><style>svg { display: block; width: 400px; height: 300px; } #t { transform-box: fill-box; transform-origin: 0px 0px; transform: rotate(90deg); }</style></head><body style="margin:0"><svg width="400" height="300"><path id="t" d="M 0 0 C 50 0 100 50 100 100"/></svg></body></html>"#,
    );
    let sheet2 = zero_css_parser::Parser::parse_stylesheet(
        "svg { display: block; width: 400px; height: 300px; } #t { transform-box: fill-box; transform-origin: 0px 0px; transform: rotate(90deg); }",
    );
    let mut sys2 = zero_style_system::StyleSystem::new();
    sys2.set_viewport(800.0, 600.0);
    let styles2 = sys2.compute_styles(&doc2, &[sheet2]);
    let svg2 = doc2.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let mut css2: Vec<(zero_dom::NodeId, String)> = Vec::new();
    crate::paint::painter::collect_css_transforms(&doc2, svg2, &styles2, &mut css2);
    assert_eq!(css2.len(), 1, "曲线 path 的 transform 仍被收集");
    assert!(
        !css2[0].1.contains("translate("),
        "曲线 path 无参考框（宁缺勿错）→ 不注入 origin 偏移，got: {}",
        css2[0].1
    );

    // g 容器 = 子形状并集（fill-box-002 形态）。
    let doc3 = zero_dom::parse_html(
        r#"<html><body><svg width="400" height="300"><g id="c"><rect x="50" y="50" width="100" height="100"/></g></svg></body></html>"#,
    );
    let g_id = doc3.get_elements_by_tag_name("g").into_iter().next().expect("g");
    let bbox = crate::paint::painter::svg_element_bbox_for_test(&doc3, g_id);
    assert_eq!(bbox, Some((50.0, 50.0, 100.0, 100.0)), "g 容器 bbox = 子 rect 并集");
}
