//! R5296（CSS2 §10.6.2/§10.8.1）：单行大 line-height inline 的 box-level 背景门。
//!
//! 修复前：多行判据 `content_height > 1.5×fs` 把「单行、line-height 膨胀」的
//! inline（content_height = 行盒高随 line-height/继承膨胀）误判多行——box-level
//! 背景被抑制落 R639 fragment 路径旧 `frag.y` 位（= baseline−line_height），背景
//! 矩形整体上溢半 leading 量、覆盖邻块内容（driving: content-height-001：
//! lh 200/fs 50 单行 span bg 画到 y=−28.6，盖住 `<p>` 文本，✗2.50%）。
//!
//! 修复后：判据第二输入取 inline 自身 computed line-height 解析值（继承链生效）
//! 与 run 行高最大值（R4332 border 门同式精神）——单行膨胀 span 回 box-level 路
//! 径，经 R5171 `inline_ca_paint_override` 按内容区锚定绘制；真跨行（content
//! ≈ Σ行高）仍走 per-fragment（本文件多行对照锁定）。
//!
//! kill-switch `ZW_INLINE_BG_ML_GATE=0` 回退 `1.5×fs` 高度代理（与 text.rs
//! R639 fragment 门成对，同开同关）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::StyleSystem;

use super::super::painter::Painter;

/// 搭建「inline-block div（大 line-height）+ 蓝底单行 span」页并走 layout。
///
/// P41 拆出共享体：主走查钉（[`paint_inflated_span_page`]）与脏矩形路线钉
///（[`paint_inflated_span_page_in_rect`]）共用页搭建，保证两钉同页同几何。
/// 返回（doc、styles、layout 根、span 布局盒——树几何 = 行盒域，R4379/R4383 判例）。
fn build_inflated_span_page(
    line_height_css: &str,
) -> (
    zero_dom::Document,
    std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    std::sync::Arc<LayoutBox>,
    LayoutBox,
) {
    let css = format!(
        "body {{ margin: 0 }} #d {{ font: 100px Ahem; display: inline-block; line-height: {line_height_css}; }} #s {{ background: #00F; }}"
    );
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<p>lead</p>
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
    (doc, styles, result.root, span)
}

/// 搭建页并经主走查 `Painter::paint` 施绘（r5296 门态消费走 `paint_node_inner`
/// skip 门臂）。
///
/// `ca_expand`/`ml_gate` 经 [`Painter::set_inline_gates_for_test`] 构造后直改
/// 门态字段（T60：进程内 set_var 翻转窗与并行用例构造窗重叠会错读门态）。
/// 返回（图元、span 布局盒——树几何 = 行盒域，R4379/R4383 判例）。
fn paint_inflated_span_page(line_height_css: &str, ca_expand: bool, ml_gate: bool) -> (RenderPrimitives, LayoutBox) {
    let (doc, styles, root, span) = build_inflated_span_page(line_height_css);
    let mut painter = Painter::new();
    painter.set_inline_gates_for_test(ca_expand, ml_gate);
    painter.paint(&root, &styles, Some(&doc));
    (painter.into_primitives(), span)
}

/// P41：同页经脏矩形增量路线 [`Painter::paint_in_rect`] 施绘（门态消费走
/// `paint_node_in_rect` 的 `ml_gate_lh` 臂——与主走查 `paint_node_inner` 臂谓词
/// 同源、代码独立复制）。脏区覆盖全页，增量快剔不剪除目标子树。
fn paint_inflated_span_page_in_rect(
    line_height_css: &str,
    ca_expand: bool,
    ml_gate: bool,
) -> (RenderPrimitives, LayoutBox) {
    let (doc, styles, root, span) = build_inflated_span_page(line_height_css);
    let mut painter = Painter::new();
    painter.set_inline_gates_for_test(ca_expand, ml_gate);
    let dirty = zero_render_foundation::geometry::Rect::new(0.0, 0.0, 800.0, 600.0);
    painter.paint_in_rect(&root, &styles, &dirty, Some(&doc));
    (painter.into_primitives(), span)
}

/// 蓝色填充矩形集合（背景与边框条带同一 add_fill 发射面）。
fn blue_fills(prims: &RenderPrimitives) -> Vec<(f32, f32, f32, f32)> {
    prims
        .fills
        .iter()
        .filter(|f| f.color.r == 0 && f.color.g == 0 && f.color.b >= 250)
        .map(|f| (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height))
        .collect()
}

/// R5296 主判例：单行 span（lh 200px 膨胀盒）背景回 box-level 并内容区锚定。
///
/// Ahem 100px dormant 常数 content area = 1.0×fs = 100（ascent 0.8 + descent 0.2，
/// inline_finalization 记录臂同源）；树盒高 = 行盒 200。收缩锚定 ⇒ 背景矩形
/// ⊆ 树盒（修复前 frag.y 位 = baseline−200 上溢出盒）。双相位共用一测试函数
///（门态经 `set_inline_gates_for_test` 注入，T60 竞态纪律：不触进程级 env）。
#[test]
fn r5296_single_line_inflated_lh_span_bg_box_level_content_anchored() {
    // 相位① 默认（门开）：背景 = 内容区锚定，收缩进树盒。
    let (prims, span) = paint_inflated_span_page("200px", true, true);
    let fills = blue_fills(&prims);
    assert_eq!(fills.len(), 1, "单 span 单行应恰一背景矩形，实得 {fills:?}");
    let (x, y, w, h) = fills[0];
    assert!(
        (h - 100.0).abs() <= 1.0,
        "背景高应 = Ahem 内容区 100（内容区锚定），实得 {h}"
    );
    assert!((w - 200.0).abs() <= 1.0, "背景宽应 = 2×Ahem 字符 advance 200，实得 {w}");
    assert!(
        y >= span.y - 0.5 && y + h <= span.y + span.height + 0.5,
        "收缩锚定：背景 [{y},{}] 应 ⊆ 树盒 [{},{}]",
        y + h,
        span.y,
        span.y + span.height
    );

    // 相位② 门关（setter 双 false ≡ kill-switch `ZW_INLINE_BG_ML_GATE=0` +
    // `ZW_INLINE_CA_EXPAND=0`）：回退高度代理 → fragment 路径旧 frag.y 位
    //（背景 = [baseline−200, baseline]，高 200、上溢出树盒）。
    // R5350：扩张轴（默认开）把 fragment bg 站整盒化（内容区锚），本相位钉
    // R5296 门关旧几何——同钉轴关态。
    let (prims_off, span_off) = paint_inflated_span_page("200px", false, false);
    let fills_off = blue_fills(&prims_off);
    assert_eq!(fills_off.len(), 1, "kill-switch 相位应恰一背景矩形，实得 {fills_off:?}");
    let (_, y_off, _, h_off) = fills_off[0];
    assert!(
        (h_off - 200.0).abs() <= 1.0,
        "门关应回退行盒域绘制（高 200），实得 {h_off}"
    );
    assert!(
        y_off < span_off.y - 1.0,
        "门关应复现修复前上溢（frag.y 位低于树盒顶），实得 y={y_off} 盒顶={}",
        span_off.y
    );
}

/// 多行对照：真跨行 span 仍走 per-fragment 路径（box-level 抑制不变量保持）。
///
/// 窄容器内 3 个词折 3 行（词间可断），content_height ≈ Σ行高 > 1.5×判据阈值
/// ⇒ 抑制生效、R639 逐行发射 3 条背景矩形。
#[test]
fn r5296_true_multiline_span_bg_still_per_fragment() {
    let css = "body { margin: 0 } #d { font: 20px Ahem; width: 45px; } #s { background: #00F; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div id="d"><span id="s">XX XX XX</span></div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    let fills = blue_fills(&painter.into_primitives());
    assert_eq!(fills.len(), 3, "真跨行 span 应逐行发射 3 条背景，实得 {fills:?}");
    let mut ys: Vec<f32> = fills.iter().map(|f| f.1).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        ys[1] - ys[0] > 1.0 && ys[2] - ys[1] > 1.0,
        "3 条背景应分处 3 行（y 互异），实得 {ys:?}"
    );
}

/// P41（T60 TV-F2 承接）：脏矩形增量路线的 ml_gate_lh 臂单点回归钉。
///
/// T60 TV 变异定谳：主走查臂（`paint_node_inner` skip 门，4141 域）单点恒开可被
/// 上方钉案相位②检出（MUT2 红），而 `paint_node_in_rect` 臂单点恒开 2P/0F 绿
///（MUT1）——既有钉案只驱动 `Painter::paint` 主走查，不触脏矩形路线，该臂无独立
/// 断言。本钉以同页（共享 [`build_inflated_span_page`]）经 `paint_in_rect` 驱动
/// 该臂消费点，双相位按探针实测形状锁定（见下方「形状参考」注释）：门关相位
/// 仅剩 fragment 旧几何；脏矩形臂单点恒开变异（抑制失效回 box-level 双矩形）下
/// 门关相位必红。
#[test]
fn r5296_dirty_rect_route_ml_gate_pinned() {
    let approx = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
        (a.0 - b.0).abs() <= 0.5 && (a.1 - b.1).abs() <= 0.5 && (a.2 - b.2).abs() <= 0.5 && (a.3 - b.3).abs() <= 0.5
    };

    // 相位① 门开（默认）：box-level bg 施画——内容区锚定矩形在场。脏矩形路线
    // 现状另发射行盒域矩形（见上「形状参考」注释；门逻辑 T60 已字段化，本切片
    // 未触，双矩形为该路线既有形状）。臂回归漂移时两矩形集合必变。
    let (prims, _span) = paint_inflated_span_page_in_rect("200px", true, true);
    let fills = blue_fills(&prims);
    assert_eq!(fills.len(), 2, "门开现状 = 行盒域 + 内容锚定两条，实得 {fills:?}");
    assert!(
        fills.iter().any(|&f| approx(f, (0.0, 100.0, 200.0, 100.0))),
        "门开应含内容区锚定矩形 (0,100,200,100)，实得 {fills:?}"
    );
    assert!(
        fills.iter().any(|&f| approx(f, (0.0, 50.0, 200.0, 200.0))),
        "门开应含行盒域矩形 (0,50,200,200)，实得 {fills:?}"
    );

    // 相位② 门关（setter 双 false ≡ kill-switch）：抑制阈值回退 1.5×fs 高度代理
    // → box-level bg 抑制生效 → 仅剩 fragment 路径旧 frag.y 位 (0,-20,200,200)
    //（= baseline−lh 上溢，与主走查钉相位②同源几何）。脏矩形臂单点恒开变异
    //（抑制失效回 box-level）下本相位必红。
    let (prims_off, _span_off) = paint_inflated_span_page_in_rect("200px", false, false);
    let fills_off = blue_fills(&prims_off);
    assert_eq!(fills_off.len(), 1, "门关应仅剩 fragment 旧几何一条，实得 {fills_off:?}");
    assert!(
        approx(fills_off[0], (0.0, -20.0, 200.0, 200.0)),
        "门关 fragment 几何应 = (0,-20,200,200)，实得 {:?}",
        fills_off[0]
    );
}

// P41 脏矩形路线双相位形状参考（探针实测 2026-11-11）：
// - 门开 = [(0, 50, 200, 200), (0, 100, 200, 100)]（行盒域 + 内容区锚定两条——
//   该路线 bg 臂未做 R5171 收敛的既有形状，非本切片引入）；
// - 门关 = [(0, -20, 200, 200)]（仅 fragment 路径旧 frag.y 位，与主走查钉相位②
//   同源几何）。
