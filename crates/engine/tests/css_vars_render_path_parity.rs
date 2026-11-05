//! var() 三渲染路径同源 parity（css-variables-1 §3）。
//!
//! 背景（2026-11-04 github.com/trending stale 定谳）：bing-t5 期引入的 CSS 文本级
//! var() 预替换使三条生产渲染路径各持一种 var() 处理形态——预算会话（AsyncPageLoad
//! rerender band，无门替换）、render_html（DOM 突变 drain，R5106 冲突门 + R5386
//! @property 跳过替换）、repaint_cached_viewport（resize，raw css 原生级联解析）。
//! 同 DOM 同视口下，资源带重渲染与 resize 重渲染几何分歧（活体 right 114×8 vs
//! 459×21），stale 态无限期持续。修复 = 移除文本级预替换，全路径统一 raw css +
//! 原生级联解析（style-system `resolve_var_in_cascaded` + R2873 pending 简写）。
//!
//! 本测试锁定该不变式：对同一 fixture（同名类在不同祖先作用域下 var() 取不同值
//! ——github primer 主题形态），render_html / 预算会话 / repaint_cached_viewport /
//! DOM 突变四条路径的几何逐位一致，且 var() 按元素级联取值正确。

use zero_engine::{BudgetAdvance, BudgetedRenderSession, RenderPipeline};

const CSS: &str = r#"
body { margin: 0; }
.t1 { --w: 100px; }
.t2 { --w: 200px; }
.a { width: var(--w); height: 30px; background: rgb(11, 12, 13); }
.b { width: 40px; height: 30px; background: rgb(21, 22, 23); }
"#;

const HTML: &str = r#"<!DOCTYPE html><html><head><style>
body { margin: 0; }
.t1 { --w: 100px; }
.t2 { --w: 200px; }
.a { width: var(--w); height: 30px; background: rgb(11, 12, 13); }
.a2 { background: rgb(31, 32, 33); }
.b { width: 40px; height: 30px; background: rgb(21, 22, 23); }
</style></head><body>
<div class="t1"><div class="a"></div></div>
<div class="t2"><div class="a a2"></div></div>
<div class="b"></div>
</body></html>
"#;

/// 探针：按颜色取图元矩形（(x, y, w, h)，像素级）。
fn rect_of(result: &zero_engine::RenderResult, r: u8, g: u8, b: u8) -> (f32, f32, f32, f32) {
    let fill = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == r && f.color.g == g && f.color.b == b && f.color.a == 255)
        .unwrap_or_else(|| panic!("fill rgb({r},{g},{b}) not found"));
    (
        fill.rect.origin.x,
        fill.rect.origin.y,
        fill.rect.size.width,
        fill.rect.size.height,
    )
}

fn assert_same(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), what: &str) {
    const EPS: f32 = 0.01;
    let close = |x: f32, y: f32| (x - y).abs() <= EPS;
    assert!(
        close(a.0, b.0) && close(a.1, b.1) && close(a.2, b.2) && close(a.3, b.3),
        "{what}: {a:?} != {b:?}"
    );
}

/// render_html 路径：var() 按元素级联取值（t1 作用域 100px / t2 作用域 200px——
/// 文本级 last-wins 替换会两盒同宽，即回归信号）。
#[test]
fn var_resolves_per_element_cascade() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(HTML, CSS);
    let box1 = rect_of(&result, 11, 12, 13);
    let box2 = rect_of(&result, 31, 32, 33);
    assert!(
        (box1.2 - 100.0).abs() < 0.5,
        "t1-scoped --w must resolve to 100px, got {:?}",
        box1
    );
    assert!(
        (box2.2 - 200.0).abs() < 0.5,
        "t2-scoped --w must resolve to 200px, got {:?}",
        box2
    );
}

/// 预算会话路径（AsyncPageLoad rerender band 的渲染形态）与 render_html 几何逐位一致。
#[test]
fn budget_session_matches_render_html() {
    let baseline = RenderPipeline::new(800.0, 600.0).render_html(HTML, CSS);
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let mut session = BudgetedRenderSession::new(HTML, CSS);
    let mut session_result = None;
    for _ in 0..8 {
        if let BudgetAdvance::Complete = pipeline.advance_budgeted_render(&mut session, 4000.0) {
            session_result = session.take_result();
            break;
        }
    }
    let session_result = session_result.expect("budget session must complete");
    for (r, g, b) in [(11u8, 12u8, 13u8), (31u8, 32u8, 33u8), (21u8, 22u8, 23u8)] {
        let expected = rect_of(&baseline, r, g, b);
        let actual = rect_of(&session_result, r, g, b);
        assert_same(actual, expected, "budget session vs render_html rgb({r},{g},{b})");
    }
}

/// repaint_cached_viewport 路径（resize 形态）与 render_html 几何逐位一致。
#[test]
fn repaint_cached_viewport_matches_render_html() {
    let baseline = RenderPipeline::new(800.0, 600.0).render_html(HTML, CSS);
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(HTML, CSS);
    let repaint = pipeline
        .repaint_cached_viewport(CSS)
        .expect("repaint after render_html must have cached doc");
    for (r, g, b) in [(11u8, 12u8, 13u8), (31u8, 32u8, 33u8), (21u8, 22u8, 23u8)] {
        let expected = rect_of(&baseline, r, g, b);
        let actual = rect_of(&repaint, r, g, b);
        assert_same(actual, expected, "repaint vs render_html rgb({r},{g},{b})");
    }
}

/// DOM 突变路径（drain 的 render_with_dom_mutations 形态）：内容中性突变前后几何一致，
/// 且与 render_html 基线一致（同源 css，无文本替换分叉）。
#[test]
fn dom_mutation_render_matches_render_html() {
    use zero_engine::DomMutation;
    let baseline = RenderPipeline::new(800.0, 600.0).render_html(HTML, CSS);
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(HTML, CSS);
    let (result, _snapshot, _handles) = pipeline
        .render_with_dom_mutations(
            &[DomMutation::SetAttr {
                selector: ".b".to_string(),
                name: "data-parity-probe".to_string(),
                value: "1".to_string(),
            }],
            CSS,
        )
        .expect("mutation render must succeed");
    let mutated = result.expect("content mutation must produce a render");
    for (r, g, b) in [(11u8, 12u8, 13u8), (31u8, 32u8, 33u8), (21u8, 22u8, 23u8)] {
        let expected = rect_of(&baseline, r, g, b);
        let actual = rect_of(&mutated, r, g, b);
        assert_same(actual, expected, "mutation vs render_html rgb({r},{g},{b})");
    }
}
