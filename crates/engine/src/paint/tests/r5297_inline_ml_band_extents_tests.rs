//! R5297（css-break-3 §5.3 slice + CSS2.1 §10.8.1 content area）：多片段 slice
//! inline 的背景按「owner 行连续 x 域 + 内容区竖直锚」绘制。
//!
//! 修复前：多片段 slice 误随单片段 kill-switch（默认关）回落基线锚+词宽——
//! 背景逐词 run 矩形、竖直贴行盒域、止步于子 inline 边界（c42-ibx-ht-000
//! 实证：chromium 为内容区带 [b−a, b+d] 行间留 half-leading 缝、行片段连续域
//! 贯穿子 inline 穴）；且 inline 自盒存储 IFC 的 pass 中行域止步盒误取自身，
//! 自身表项全被剔除（查表 miss 回落词宽）。
//!
//! 修复后：
//! 1. 竖直 = 内容区锚（R5120 content box 对多片段 slice 生效）；
//! 2. x = owner 行连续域（owner_line_band_extent_map：run 归属 owner 沿祖先链
//!    累计，含子 inline 领土；止步盒仅块容器，inline 自绘 pass 不止步）；
//! 3. 同带随片段重复发射但逐行全等（连续带语义，非逐词碎片）。

use zero_render_foundation::primitive::RenderPrimitives;

use super::super::Painter;

/// 搭建「4 行折叠 span（蓝底）内嵌 em 子 inline（跨 L2→L3）」页：Ahem 10px/12px、
/// 宽 75，折行 `aa bb / cc dd / ee ff / gg hh`（dd 属 em 落 L2、ee 属 em 落 L3，
/// 每行都保有 span 自身词——bg 按 owner 发射，纯子 inline 行无线索）。
fn paint_nested_wrap_page() -> RenderPrimitives {
    let css = "body { margin: 0 } #d { font: 10px Ahem; line-height: 12px; width: 75px; } \
               #s { background: #00F; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div id="d"><span id="s">aa bb cc <em>dd ee</em> ff gg hh</span></div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    painter.into_primitives()
}

/// 蓝色填充矩形集合（span 背景唯一发射面，em 无背景）。
fn blue_fills(prims: &RenderPrimitives) -> Vec<(f32, f32, f32, f32)> {
    prims
        .fills
        .iter()
        .filter(|f| f.color.r == 0 && f.color.g == 0 && f.color.b >= 250)
        .map(|f| (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height))
        .collect()
}

/// R5297 主判例：折叠 span 背景逐行连续带 + 子 inline 领土贯穿。
///
/// 行域 [0,50]（词宽 20 + 空格 10）：L2 的带自 cc（span 词 [0,20]）累计 dd
/// 领土 [30,50]；L3 的带自 ff（span 词 [30,50]）累计 ee 领土 [0,20]——带宽 50
/// 证明祖先链累计贯穿子 inline 穴。
///
/// 竖直锚：单测进程无字体库度量（fallback_line_metrics_for_paint → None），
/// R5120 content box 回退行盒锚 (12k, 12)；内容区锚 (12k+1, 10) 的判例由
/// corpus c42-ibx-ht-000 reftest 对 + oracle 截图锁定（真实字体栈）。
#[test]
fn r5297_ml_span_bg_line_contiguous_across_child_inline() {
    let fills = blue_fills(&paint_nested_wrap_page());
    assert!(!fills.is_empty(), "应逐行发射背景带");

    let line_of = |y: f32| (y / 12.0).round() as i32;
    let mut by_line: Vec<(i32, Vec<(f32, f32, f32, f32)>)> = Vec::new();
    for f in &fills {
        let line = line_of(f.1);
        match by_line.iter_mut().find(|(l, _)| *l == line) {
            Some((_, v)) => v.push(*f),
            None => by_line.push((line, vec![*f])),
        }
    }
    by_line.sort_by_key(|(l, _)| *l);
    assert_eq!(by_line.len(), 4, "背景应分处 4 行，实得 {by_line:?}");

    // 期望带：(x, y, w, h)——y = 12k（行盒锚回退），h = 12，
    // x/w = 行连续域 [0,50]。同带随片段重复发射全等矩形（div/span 双 pass
    // 亦可能重复同几何），只断言组内全等。
    for (idx, (line, rects)) in by_line.iter().enumerate() {
        assert_eq!(*line, idx as i32, "行序应 L1→L4，实得 {by_line:?}");
        let e = (0.0, 12.0 * idx as f32, 50.0, 12.0);
        assert!(!rects.is_empty(), "L{} 应至少一条背景带", idx + 1);
        for r in rects {
            assert!(
                (r.0 - e.0).abs() < 0.5
                    && (r.1 - e.1).abs() < 0.5
                    && (r.2 - e.2).abs() < 0.5
                    && (r.3 - e.3).abs() < 0.5,
                "L{} 背景带应 = {e:?}（连续域），实得 {r:?}",
                idx + 1
            );
        }
    }
}
