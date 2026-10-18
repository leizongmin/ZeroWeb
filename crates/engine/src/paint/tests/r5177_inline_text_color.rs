//! R5177 回归：display:inline 化元素的行内文本 color 保真（css-ui outline-027 案形）。
//!
//! CSS 2.1 §16.1 color 继承 + R4300 walk 折回祖先吸收语义的 color 盲区：
//! `span > div{display:inline;color:lime}` 的 XX（Ahem 墨块）在 walk 扁平化中被
//! 折回 span 的 pending、run 归因 span → paint 侧 R358 per-fragment color 取
//! span 色（容器黑）——同选择器下 display:inline-block / block 分支文本绿、仅
//! inline 分支黑（outline-027 PNG：第 5/6 列黑块，前 4 列绿）。
//!
//! 修复两处：collect_flat_inline_children 折回臂加 color divergent 判定（对齐
//! R4300d white-space 判例——computed color 与外层不同则子自身构造独立 run；
//! 双通道：layout IFC 有 styles 直判，paint Path B 读 color_divergent_walk_nodes
//! 注入集）；render_fragment 渲染时沿 frag nid 上溯 inline 祖先链补标
//! painted_inline_nodes（拆分归因后外层 inline 载体的 Path B 重跑会双绘，旧折回
//! 归因下外层 nid 自身被 insert 恰好覆盖此面）。
//!
//! kill-switch 回退由 corpus OFF 轮承担（无独立开关；color divergent 判定闭合在
//! collect_items 折回臂 + painter 注入，回退需还原两处）。

use zero_style_system::StyleSystem;

use super::super::painter::Painter;

/// outline-027 案形最小化：span > div(display:inline, color:lime) > "XX"，
/// 走 layout → paint 全管线，断言字形全 lime 且无双绘（修复前 2 黑 glyph；
/// 中间态曾 4 lime 双绘——外层 Path B 重跑）。
#[test]
fn test_r5177_inline_div_text_color() {
    let css = "body { margin: 0 } #d { font: 100px Ahem } #i { display: inline; color: lime }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div id="d"><span><div id="i">XX</div></span></div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    let prims = painter.into_primitives();

    let colors: Vec<(u8, u8, u8)> = prims.glyphs.iter().map(|g| (g.color.r, g.color.g, g.color.b)).collect();
    assert_eq!(
        colors.len(),
        2,
        "XX 两字形且无双绘（旧折回归因 2 黑 / 拆分初期 4 双绘），实得 {colors:?}"
    );
    assert!(
        colors.iter().all(|c| c.1 >= 250 && c.0 == 0 && c.2 == 0),
        "inline div 文本应为 lime，实得 {colors:?}"
    );
}

/// 对照：color 与外层一致的 inline 子不触发拆分（折回行为不变，行断/归因契约
/// 不动）——span(color:red) > div(inline, color:red) 的 XX 仍归因 span 渲染。
#[test]
fn test_r5177_same_color_inline_no_split() {
    let css = "body { margin: 0 } #d { font: 100px Ahem } #s { color: red } #i { display: inline; color: red }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div id="d"><span id="s"><div id="i">XX</div></span></div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    let prims = painter.into_primitives();

    let colors: Vec<(u8, u8, u8)> = prims.glyphs.iter().map(|g| (g.color.r, g.color.g, g.color.b)).collect();
    assert_eq!(colors.len(), 2, "color 一致不拆分，双绘判据同前");
    assert!(
        colors.iter().all(|c| c.0 >= 250 && c.1 == 0 && c.2 == 0),
        "应为 red（span/div 任一归因同色），实得 {colors:?}"
    );
}
