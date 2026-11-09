//! R5428 视口滚动（root-scroller）常驻 pin。
//!
//! 'html' 滚动记录 = 视口滚动（cssom-view-1 §11.2：根元素可滚溢出传播至 viewport，
//! 滚动根元素 = 滚动视口）——reftest canvas 即视口窗：文档子树整体平移、画布背景
//! 保持视口锚定。shim 侧 documentElement.scrollTop 经 M2-S7 委托 window.scrollTo、
//! 与 scrollX/scrollY 同镜像 `_winScroll` 槽、`__zw_dump_scroll_offsets` 以 'html'
//! 回流（驱动：WPT css-overflow overflow-clip-margin-007/012——此前 handler 已执行
//! 但窗口槽既不导出也不消费，双侧停在不滚动位）。
//!
//! 三块语义：
//! 1. 'html' 记录抽取为 viewport_scroll（不入 per-element map——根盒重复消费双计）
//!    并平移文档子树；
//! 2. root-scroller 的 client 面按视口量取（cssom-view-1 §11.2 scrollTop 值域 =
//!    滚动区跨度 − 视口尺寸；根盒 auto 高 = 文档高，按盒面钳会归零）；
//! 3. 普通滚动容器（overflow hidden + 自记录）路径不受影响（R4353 回归卫）。
//!
//! 观测口径：overflow 裁剪 rewrite 对 fill 按 clip rect 钳制边（非真交），断言取
//! 平移后可分辨的几何（子块部分露出裁剪窗）。

use zero_layout_engine::LayoutEngine;
use zero_render_foundation::primitive::FillPrimitive;
use zero_style_system::StyleSystem;

use super::super::painter::Painter;

/// 找背景为纯红的 fill 图元（测试页唯一红源 = 目标 div 背景）。
fn red_fills(prims: &zero_render_foundation::primitive::RenderPrimitives) -> Vec<&FillPrimitive> {
    prims
        .fills
        .iter()
        .filter(|f| f.color.r >= 200 && f.color.g < 60 && f.color.b < 60)
        .collect()
}

/// 搭建「body(margin 0) > [spacer 高 600, div#t(红 100×100)]」页面（文档 700 高 >
/// 视口 600，滚动范围 100），注入滚动记录后走 layout → paint 全管线。
/// 返回 (红 fill 图元, viewport_scroll 抽取值, per-element map 大小, pending 记录数)。
fn paint_scrolled_page(scrolls: &[(&str, f32, f32)]) -> (Vec<FillPrimitive>, Option<(f32, f32)>, usize, usize) {
    let html = r#"<html><body style="margin:0"><div style="height:600px"></div><div id="t" style="width:100px;height:100px;background:red"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.viewport_w = 800.0;
    painter.viewport_h = 600.0;
    painter.set_scroll_offsets(scrolls.iter().map(|(s, x, y)| (s.to_string(), *x, *y)).collect());
    painter.paint(&result.root, &styles, Some(&doc));
    let viewport = painter.viewport_scroll;
    let per_elem = painter.scroll_offsets.len();
    let pending = painter.pending_scroll_offsets.len();
    let prims = painter.into_primitives();
    (
        red_fills(&prims).into_iter().cloned().collect(),
        viewport,
        per_elem,
        pending,
    )
}

/// 'html' 记录抽取为 viewport_scroll、不入 per-element map，并平移文档子树（语义 1）。
#[test]
fn r5428_html_record_becomes_viewport_scroll() {
    // 滚动 100（范围内）→ 红块 y = 600 − 100 = 500（不平移则 600）
    let (reds, viewport, per_elem, pending) = paint_scrolled_page(&[("html", 0.0, 100.0)]);
    assert_eq!(pending, 1, "'html' 记录须在 pending 面");
    assert_eq!(viewport, Some((0.0, 100.0)), "'html' 记录须抽取为视口滚动");
    assert_eq!(per_elem, 0, "'html' 记录不得入 per-element map（根盒双计）");
    assert_eq!(reds.len(), 1, "测试页唯一红源");
    assert!(
        (reds[0].rect.origin.y - 500.0).abs() < 0.5,
        "视口滚动 100 后红块须平移到 y=500（实得 {}）",
        reds[0].rect.origin.y
    );
}

/// root-scroller 的 client 面按视口量取（语义 2）——滚动 300 超出可滚范围
/// 700 − 600 = 100：钳到 100（若按根盒 auto 高 700 量取会钳到 0，不平移）。
#[test]
fn r5428_root_scroller_clamps_against_viewport_client() {
    let (reds, viewport, _, _) = paint_scrolled_page(&[("html", 0.0, 300.0)]);
    assert_eq!(viewport, Some((0.0, 300.0)), "记录面保留原始请求值（钳制在应用点）");
    assert_eq!(reds.len(), 1);
    assert!(
        (reds[0].rect.origin.y - 500.0).abs() < 0.5,
        "滚动 300 须钳到可滚范围 100（红块 y=500，实得 {}）",
        reds[0].rect.origin.y
    );
}

/// 普通滚动容器（overflow hidden + 自记录）路径不受影响（语义 3，R4353 回归卫）。
/// scroller 100×100 内红块 margin-top 150（窗下 50）、滚动 100 → 红块现于 y=50、
/// 伸出窗底 50（不平移则仍在窗下不可见）。
#[test]
fn r5428_element_scroller_path_unchanged() {
    let html = r#"<html><body style="margin:0"><div id="s" style="width:100px;height:100px;overflow:hidden"><div style="height:100px;margin-top:150px;background:red"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.viewport_w = 800.0;
    painter.viewport_h = 600.0;
    painter.set_scroll_offsets(vec![("div#s".to_string(), 0.0, 100.0)]);
    painter.paint(&result.root, &styles, Some(&doc));
    assert_eq!(painter.viewport_scroll, None, "非 'html' 记录不抽取视口滚动");
    assert_eq!(painter.scroll_offsets.len(), 1, "元素记录留 per-element map");
    let prims = painter.into_primitives();
    let reds = red_fills(&prims);
    assert_eq!(reds.len(), 1, "红块滚入窗内须发射");
    assert!(
        (reds[0].rect.origin.y - 50.0).abs() < 0.5 && (reds[0].rect.size.height - 50.0).abs() < 0.5,
        "滚动 100 后红块应于 y=50、伸出窗底裁剩 50 高（实得 y={} h={})",
        reds[0].rect.origin.y,
        reds[0].rect.size.height
    );
}

/// 定位 flush 随视口平移（R5428 收藏原点平移臂）——根 scope 收集早于子盒递归
/// R4353 块，收藏原点不平移则被根 scope 认领的 positioned 后代全部绘在未滚位
///（驱动：contain-layout-021 fixed CB = contain:layout html、fixed-z-index-blend
/// relative z-3 文档内容，二者此前伪绿）。FIXME：fixed CB 域分析未建模
///（真 viewport-fixed 亦随滚，见 painter 侧 FIXME 注记）。
#[test]
fn r5428_positioned_flush_follows_viewport_scroll() {
    // abspos 红块 top:600（layout 位 600，视口 600 下不可见）+ 视口滚动 100 → 现于 y=500
    let html = r#"<html><body style="margin:0"><div style="position:relative;top:600px;width:100px;height:100px;background:red"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.viewport_w = 800.0;
    painter.viewport_h = 600.0;
    painter.set_scroll_offsets(vec![("html".to_string(), 0.0, 100.0)]);
    painter.paint(&result.root, &styles, Some(&doc));
    let prims = painter.into_primitives();
    let reds = red_fills(&prims);
    assert_eq!(reds.len(), 1, "定位红块须发射");
    assert!(
        (reds[0].rect.origin.y - 500.0).abs() < 0.5,
        "定位 flush 须随视口滚动平移（红块 y=500，实得 {}）",
        reds[0].rect.origin.y
    );
}
