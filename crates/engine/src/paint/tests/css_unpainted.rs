//! CSS quotes / text-wrap / line-clamp / scrollbar-gutter / background-attachment / hyphens 渲染测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    BackgroundAttachmentComputedValue, ComputedStyle, HyphensComputedValue, LineClampComputedValue,
    QuotesComputedValue, ScrollbarGutterComputedValue, ScrollbarWidthComputedValue,
};

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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

// === scrollbar-gutter 测试 ===

#[test]
fn test_scrollbar_gutter_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // scrollbar-gutter: auto 不应生成 gutter 指示器
    let prims = painter.primitives();
    let has_gutter = prims
        .fills
        .iter()
        .any(|f| f.color.r == 245 && f.color.g == 245 && f.color.b == 245 && f.color.a == 120);
    assert!(!has_gutter, "scrollbar-gutter: auto 不应生成 gutter 指示器");
}

#[test]
fn test_scrollbar_gutter_stable_generates_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // stable 应生成右侧 10px 宽的 gutter
    let has_gutter = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 10.0 && f.color.r == 245 && f.color.a == 120);
    assert!(has_gutter, "scrollbar-gutter: stable 应生成右侧 gutter");
}

#[test]
fn test_scrollbar_gutter_stable_both_edges_generates_two() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::StableBothEdges;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let gutter_count = prims
        .fills
        .iter()
        .filter(|f| f.color.r == 245 && f.color.a == 120 && f.rect.size.width == 10.0)
        .count();
    assert_eq!(
        gutter_count, 2,
        "scrollbar-gutter: stable both-edges 应生成左右两个 gutter"
    );
}

#[test]
fn test_scrollbar_gutter_thin_scrollbar_width() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
    style.scrollbar_width = ScrollbarWidthComputedValue::Thin;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // thin scrollbar 应生成 6px 宽的 gutter
    let has_thin_gutter = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 6.0 && f.color.r == 245 && f.color.a == 120);
    assert!(has_thin_gutter, "scrollbar-gutter: stable + thin 应生成 6px gutter");
}

// === background-attachment 测试 ===

#[test]
fn test_background_attachment_scroll_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(!has_pin, "background-attachment: scroll 不应生成固定背景指示器");
}

#[test]
fn test_background_attachment_fixed_generates_pin() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_attachment = vec![BackgroundAttachmentComputedValue::Fixed];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 固定背景应生成蓝色图钉指示器
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(has_pin, "background-attachment: fixed 应生成图钉指示器");
}

#[test]
fn test_background_attachment_local_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_attachment = vec![BackgroundAttachmentComputedValue::Local];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(!has_pin, "background-attachment: local 不应生成固定背景指示器");
}

// === hyphens 测试 ===

#[test]
fn test_hyphens_auto_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.hyphens = HyphensComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // hyphens: auto 应在底部生成横线指示器
    let has_hyphen_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 8.0 && f.rect.size.height == 1.0 && f.color.a == 160);
    assert!(has_hyphen_indicator, "hyphens: auto 应生成横线指示器");
}

#[test]
fn test_hyphens_none_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_hyphen_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 8.0 && f.rect.size.height == 1.0 && f.color.a == 160);
    assert!(!has_hyphen_indicator, "hyphens: none 不应生成指示器");
}

// === quotes 测试 ===
// R3895：quotes 是纯求值属性（CSS Content 3 §2.2），paint 层不做盒级引号绘制；
// <q> 标记由 inline 侧 resolve_q_quotes 注入文本流（R2246）。

#[test]
fn test_quotes_pairs_no_box_level_glyphs() {
    // quotes: Pairs 的盒子自身不绘制引号 glyph（标记只来自 <q> 内容注入）
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.quotes = QuotesComputedValue::Pairs(vec![("\u{201C}".to_string(), "\u{201D}".to_string())]);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let open_quote_code = '\u{201C}' as u32;
    let close_quote_code = '\u{201D}' as u32;
    let has_open = prims.glyphs.iter().any(|g| g.glyph_id == open_quote_code);
    let has_close = prims.glyphs.iter().any(|g| g.glyph_id == close_quote_code);
    assert!(!has_open, "quotes: Pairs 盒级不绘制开引号 glyph");
    assert!(!has_close, "quotes: Pairs 盒级不绘制闭引号 glyph");
}

#[test]
fn test_quotes_none_no_glyphs() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("q");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.quotes = QuotesComputedValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let open_quote_code = '\u{201C}' as u32;
    let close_quote_code = '\u{201D}' as u32;
    let has_open = prims.glyphs.iter().any(|g| g.glyph_id == open_quote_code);
    let has_close = prims.glyphs.iter().any(|g| g.glyph_id == close_quote_code);
    assert!(!has_open, "quotes: none 不应生成引号 glyph");
    assert!(!has_close, "quotes: none 不应生成引号 glyph");
}

#[test]
fn test_quotes_auto_no_glyphs() {
    // quotes: auto 是默认值，不应为非 <q> 元素生成引号
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // Auto 不应生成引号 glyph（仅 Pairs 时渲染）
    // 检查不应有 « » 或 " " 引号 glyph
    let auto_open = '\u{00AB}' as u32; // «
    let auto_close = '\u{00BB}' as u32; // »
    let has_auto_quotes = prims
        .glyphs
        .iter()
        .any(|g| g.glyph_id == auto_open || g.glyph_id == auto_close);
    assert!(!has_auto_quotes, "quotes: auto (默认) 不应生成 « » 引号");
}

// === text-wrap 测试 ===

#[test]
fn test_text_wrap_nowrap_override() {
    let style = ComputedStyle::default();
    assert!(
        Painter::resolve_text_wrap(&style).is_none(),
        "text-wrap: wrap (默认) 不应覆盖换行设置"
    );
}

// === line-clamp 测试 ===

#[test]
fn test_line_clamp_none_default() {
    let style = ComputedStyle::default();
    assert!(
        Painter::resolve_line_clamp(&style).is_none(),
        "line-clamp: none (默认) 不应限制行数"
    );
}

#[test]
fn test_line_clamp_count_returns_value() {
    let mut style = ComputedStyle::default();
    style.line_clamp = LineClampComputedValue::Count(3);
    let result = Painter::resolve_line_clamp(&style);
    assert_eq!(result, Some(3), "line-clamp: 3 应返回 Some(3)");
}

// === 无节点 ID 不崩溃测试 ===

#[test]
fn test_scrollbar_gutter_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 200.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_background_attachment_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_hyphens_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 30.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_quotes_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 30.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

/// CSS2 §9.3 / CSS Display 3 §2.1：display:none 子树不生成任何盒——其文本不得
/// 出现在绘制图元。回归背景：布局趟把 none 元素折为 0×0 Display::None 叶保留
/// 在盒树，绘制趟（含 paint Path B 空 styles 重跑 IFC）曾把 author 规则隐藏的
/// 文本画在盒位置，与后随兄弟叠字（baidu 顶栏右上叠字最小复现）。
#[test]
fn display_none_subtree_text_not_painted() {
    let html = r#"<html><head><style>.hidden { display: none; }</style></head>
<body style="margin:0"><div>BEFORE</div><div class="hidden">SECRET</div><div>TAIL<span class="hidden">INNER</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(".hidden { display: none; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));

    let text: String = painter
        .primitives()
        .glyphs
        .iter()
        .filter_map(|g| char::from_u32(g.glyph_id))
        .collect();
    assert!(!text.contains("SECRET"), "块级 display:none 文本不应绘制，实际: {text}");
    assert!(
        !text.contains("INNER"),
        "inline display:none 文本不应绘制，实际: {text}"
    );
    assert!(text.contains("BEFORE"), "正常块文本应绘制，实际: {text}");
    assert!(text.contains("TAIL"), "正常 inline 文本应绘制，实际: {text}");
}

/// 同上，**嵌套**形状（PR #39 审查发现 #1）：隐藏 span 在可见 inline（b）内部。
/// layout 期 b 已入 `inline_block_child_nodes` 存储信号（非 inline 白名单与
/// `has_block_level_child` 同表、含 None）→ Path B 下 b 同样不走 walk，实际生效
/// 门是 flatten 吸收路径的 `collect_text_excluding` 剪枝（注入集通道）；walk 子
/// 循环门由 layout 嵌套例 #1 钉住。本例锚定非直子泄漏端到端不再发生。
#[test]
fn display_none_nested_inline_text_not_painted() {
    let html = r#"<html><head><style>.hidden { display: none; }</style></head>
<body style="margin:0"><div>BEFORE</div><div>TAIL<b>BOLD<span class="hidden">NESTED</span>AFTER</b>TAIL2</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(".hidden { display: none; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));

    let text: String = painter
        .primitives()
        .glyphs
        .iter()
        .filter_map(|g| char::from_u32(g.glyph_id))
        .collect();
    assert!(
        !text.contains("NESTED"),
        "嵌套 inline display:none 文本不应绘制，实际: {text}"
    );
    assert!(
        text.contains("BOLD") && text.contains("AFTER"),
        "外层可见 inline 文本应绘制，实际: {text}"
    );
    assert!(
        text.contains("BEFORE") && text.contains("TAIL"),
        "兄弟文本应绘制，实际: {text}"
    );
}

/// 同 display_none_subtree_text_not_painted，但走**脏矩形路径** `paint_in_rect`
/// ——浏览器增量重绘走此路径（baidu 右上叠字实际表现通道），守卫与主路径同步
///（R3768/R3769 教训）。钉住两条绘制路由的端到端清洁等价。注：负控制（仅撤
/// painter 守卫）本测试仍绿——收集层门（walk/flatten 排除）是这些形状的有效
/// 防线，painter 守卫为纵深防御（收集门失效时二道拦截），其独立判别由 reftest
/// 负控制（撤全部修复 3.19% FAIL，15,319px）覆盖。
#[test]
fn display_none_subtree_not_painted_in_rect() {
    let html = r#"<html><head><style>.hidden { display: none; }</style></head>
<body style="margin:0"><div>BEFORE</div><div class="hidden">SECRET</div><div>TAIL<span class="hidden">INNER</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(".hidden { display: none; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut painter = Painter::new();
    let dirty = zero_render_foundation::geometry::Rect::new(0.0, 0.0, 800.0, 600.0);
    painter.paint_in_rect(&result.root, &styles, &dirty, Some(&doc));

    let text: String = painter
        .primitives()
        .glyphs
        .iter()
        .filter_map(|g| char::from_u32(g.glyph_id))
        .collect();
    assert!(
        !text.contains("SECRET"),
        "脏矩形路径块级 display:none 文本不应绘制，实际: {text}"
    );
    assert!(
        !text.contains("INNER"),
        "脏矩形路径 inline display:none 文本不应绘制，实际: {text}"
    );
    assert!(text.contains("BEFORE"), "正常块文本应绘制，实际: {text}");
    assert!(text.contains("TAIL"), "正常 inline 文本应绘制，实际: {text}");
}
