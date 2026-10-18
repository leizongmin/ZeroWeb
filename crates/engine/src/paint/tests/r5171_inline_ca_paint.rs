//! R5171（CSS2 §10.6.2/§10.8.1）：单行 inline 非替换盒的 content area = 元素主字体
//! A+D，与 line-height 无关——半 leading 属行盒不属 inline 盒。布局树保持行盒域
//!（流 bookkeeping，r3773 判例），box-level 背景/边框绘制消费 `inline_reported_rect`
//!（R4379/R4383 记录臂）的内容区锚定 border-box。
//!
//! 修复前：box-level 装饰按树几何（行盒 + padding/border）绘制，显式 line-height 页
//! 多涂整条半 leading 带（driving: inline-non-replaced-height-002/003 各 1.04%：
//! span 蓝 250px 应 200px；border-bottom 落点低 25px）。
//!
//! kill-switch `ZW_INLINE_CA_PAINT=0` 回退（Rally 判别经外部 env corpus 承担）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::StyleSystem;

use super::super::painter::Painter;

/// 搭建 Ahem 100px 单行 span 页并走 layout → paint 全管线。
///
/// 返回（图元、span 盒——树几何 = 行盒域 border-box：y = 行盒顶 − pt − bt）。
fn paint_span_page(span_css: &str, div_css: &str) -> (RenderPrimitives, LayoutBox) {
    let css = format!("body {{ margin: 0 }} #d {{ font: 100px Ahem; {div_css} }} #s {{ {span_css} }}");
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div id="d"><span id="s">XX</span></div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(&css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let span_id = doc.get_element_by_id("s").expect("span 应存在");
    let span = {
        let mut found = None;
        let mut stack = vec![(*result.root).clone()];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(span_id) {
                found = Some(b);
                break;
            }
            for c in b.children.iter() {
                stack.push(c.clone());
            }
        }
        found.expect("span 布局盒应存在")
    };
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    (painter.into_primitives(), span)
}

/// 蓝色（pure blue）填充矩形集合（背景色与边框条带同一 add_fill 发射面）。
fn blue_fills(prims: &RenderPrimitives) -> Vec<(f32, f32, f32, f32)> {
    prims
        .fills
        .iter()
        .filter(|f| f.color.r == 0 && f.color.g == 0 && f.color.b >= 250)
        .map(|f| (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height))
        .collect()
}

/// 002 形态：`padding:50px 0` + `line-height:150px`——内容区 100px（Ahem A+D），
/// 真实 border-box = 200px @ 行盒顶+25−50；旧行为按树几何 = 250px @ 行盒顶−50。
#[test]
fn r5171_single_line_inline_bg_uses_content_area_border_box() {
    let (prims, span) = paint_span_page("padding: 50px 0; background: blue", "line-height: 150px");
    let blues = blue_fills(&prims);
    assert!(
        !blues.is_empty(),
        "前置：span 蓝背景应发射 fill（树几何 y={} h={}）",
        span.y,
        span.height
    );
    let hit = blues
        .iter()
        .copied()
        .find(|(_, y, _, _)| (y - (span.y + 25.0)).abs() < 1.0)
        .unwrap_or_else(|| {
            panic!(
                "应存在内容区锚定（树 y+25 = {}）的蓝背景 fill，实际 {blues:?}",
                span.y + 25.0
            )
        });
    assert!(
        (hit.3 - 200.0).abs() < 1.0,
        "蓝背景高应 200（内容 100 + padding 100），实际 {}（旧行为 250 = 行盒 150 + padding）",
        hit.3
    );
}

/// 003 形态：`border-bottom:50px solid blue` + `line-height:150px`——底部边框条带
/// 应贴内容区底（顶 = 树 y + 半 leading 25 + 内容 100 = 树 y+125）；旧行为贴树盒底
///（顶 = 树 y+150，低 25px）。
#[test]
fn r5171_single_line_inline_border_bottom_anchors_content_area() {
    let (prims, span) = paint_span_page("border-bottom: 50px solid blue", "line-height: 150px");
    let blues = blue_fills(&prims);
    let expected_y = span.y + 125.0; // 内容区底 = 树 y + (150−100)/2 + 100
    let hit = blues
        .iter()
        .copied()
        .find(|b| (b.1 - expected_y).abs() < 1.0 && (b.3 - 50.0).abs() < 1.0)
        .unwrap_or_else(|| {
            panic!(
                "应存在内容区锚定的 50px 底部条带（顶 = 树 y+125 = {expected_y}），实际 {blues:?}（树盒 h={}）",
                span.height
            )
        });
    assert_eq!(hit.3, 50.0);
}

/// normal line-height 对照：引擎常数行高 ≡ 字体 A+D（hl=0），报告值 ≡ 树值——
/// 背景仍 = 树几何（该形态行盒高本就等于内容区 + padding，锁定零扰动）。
#[test]
fn r5171_normal_line_height_page_unchanged() {
    let (prims, span) = paint_span_page("padding: 50px 0; background: blue", "line-height: normal");
    let blues = blue_fills(&prims);
    let hit = blues
        .iter()
        .copied()
        .find(|(_, y, _, _)| (y - span.y).abs() < 1.0)
        .unwrap_or_else(|| panic!("normal 行高页背景应仍在树盒位（y={}），实际 {blues:?}", span.y));
    assert!(
        (hit.3 - 200.0).abs() < 1.0,
        "normal 行高页背景高应 200（行盒 100 = A+D + padding 100），实际 {}",
        hit.3
    );
}
