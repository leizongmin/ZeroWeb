//! 伪元素合成节点不进 HTML 序列化（css-pseudo-4 §1：伪元素不属于文档树）。
//!
//! 背景（2026-11-04 zinc 定谳，github.com/trending stale 第三分量）：渲染注入的
//! ::before/::after 替身（`zw-pseudo` 盒、伪文本子、content:url 图片）曾无 synthetic
//! 标记，`outer_html` 快照把它们写进 html——drain 渲染（end_script_batch 全量
//! re-parse）把它们当真实布局参与者（github 100+ 幽灵盒灌进 flex 容器，右侧下拉
//! 114×8 塌缩），与原始字节渲染（band/首次加载）几何分歧、与 repaint（活 doc）分歧，
//! 活体 stale 直至 resize。修复 = `NodeData::synthetic` 标记 + 序列化跳过。
//!
//! 本测试锁定两条不变式：
//! 1. 序列化排除：render_html 后 `outer_html(root)` 不含合成标记名/注入文本；
//! 2. 往返 parity：序列化产物 re-parse 再渲染，几何与首次渲染逐位一致
//!    （drain 渲染 ≡ band 渲染的引擎级形态）。

use zero_engine::RenderPipeline;

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

/// needs_box 臂（content:"" + display:block + position:absolute → `zw-pseudo` 盒）：
/// 序列化产物不含合成标记名，往返渲染几何一致。
#[test]
fn synthetic_box_pseudo_not_serialized() {
    let css = "body { margin: 0; } .a::before { content: \"\"; display: block; position: absolute; width: 50px; height: 100px; background: rgb(11, 12, 13); } .a { position: relative; width: 200px; height: 30px; background: rgb(21, 22, 23); }";
    let html = r#"<html><head><style>body { margin: 0; } .a::before { content: ""; display: block; position: absolute; width: 50px; height: 100px; background: rgb(11, 12, 13); } .a { position: relative; width: 200px; height: 30px; background: rgb(21, 22, 23); }</style></head><body><div class="a"></div></body></html>"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let first = pipeline.render_html(html, css);
    let doc_rc = pipeline.cached_doc_shared().expect("cached_doc");
    let serialized = {
        let doc = doc_rc.borrow();
        doc.outer_html(doc.root())
    };
    assert!(
        !serialized.contains("zw-pseudo"),
        "合成 zw-pseudo 盒不得进序列化: {serialized}"
    );

    // 往返：快照 re-parse 再渲染，几何逐位一致。
    let mut rt = RenderPipeline::new(800.0, 600.0);
    let second = rt.render_html(&serialized, css);
    assert_same(
        rect_of(&first, 11, 12, 13),
        rect_of(&second, 11, 12, 13),
        "needs_box 伪元素往返几何一致",
    );
    assert_same(
        rect_of(&first, 21, 22, 23),
        rect_of(&second, 21, 22, 23),
        "宿主盒往返几何一致",
    );
}

/// 文本臂（inline content:"P>" → 注入文本子）：注入文本不得出现在序列化中
///（既泄漏内容又干扰后续 parse），往返几何一致。
#[test]
fn synthetic_text_pseudo_not_serialized() {
    let css = "body { margin: 0; } .b::before { content: \"P>\"; } .b { width: 60px; height: 20px; background: rgb(31, 32, 33); }";
    let html = r#"<html><head><style>body { margin: 0; } .b::before { content: "P>"; } .b { width: 60px; height: 20px; background: rgb(31, 32, 33); }</style></head><body><div class="b"></div></body></html>"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let first = pipeline.render_html(html, css);
    let doc_rc = pipeline.cached_doc_shared().expect("cached_doc");
    let serialized = {
        let doc = doc_rc.borrow();
        doc.outer_html(doc.root())
    };
    // 断言锁定注入点：div.b 序列化为**无子元素**（<style> 源文本里的 content:"P>"
    // 字面量是 CSS 原样保留，与注入泄漏无关）。
    assert!(
        serialized.contains(r#"<div class="b"></div>"#),
        "伪元素注入文本不得进序列化（div.b 应为空元素）: {serialized}"
    );

    let mut rt = RenderPipeline::new(800.0, 600.0);
    let second = rt.render_html(&serialized, css);
    assert_same(
        rect_of(&first, 31, 32, 33),
        rect_of(&second, 31, 32, 33),
        "文本伪元素往返几何一致",
    );
}

/// 泄漏形态回归（zinc 活体 114×8 塌缩的引擎级等价）：ghost 盒若进序列化，re-parse
/// 后成为 flex 容器的**多余 flex item**（gap 空间被多占，flex 分配收窄）——序列化
/// 排除后，往返渲染的 flex 子项宽度必须逐位一致。
#[test]
fn ghost_boxes_do_not_shift_flex_layout_after_roundtrip() {
    // zinc 同构：伪元素宿主 = flex 容器自身（::before 盒化 ghost 落容器子位）。
    // gap 使 ghost item 的存在可测量（修复前：3 item 2 gap → 泄漏后 3 item 3 gap）。
    let css = "body { margin: 0; } .row { display: flex; gap: 10px; } .item { flex: 1; height: 20px; } .row::before { content: \"\"; display: block; position: absolute; width: 16px; height: 16px; background: rgb(41, 42, 43); } .featured { background: rgb(51, 52, 53); }";
    let html = r#"<html><head><style>body { margin: 0; } .row { display: flex; gap: 10px; } .item { flex: 1; height: 20px; } .row::before { content: ""; display: block; position: absolute; width: 16px; height: 16px; background: rgb(41, 42, 43); } .featured { background: rgb(51, 52, 53); }</style></head><body><div class="row"><div class="item featured"></div><div class="item"></div><div class="item"></div></div></body></html>"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let first = pipeline.render_html(html, css);
    let doc_rc = pipeline.cached_doc_shared().expect("cached_doc");
    let serialized = {
        let doc = doc_rc.borrow();
        doc.outer_html(doc.root())
    };

    let mut rt = RenderPipeline::new(800.0, 600.0);
    let second = rt.render_html(&serialized, css);
    assert_same(
        rect_of(&first, 51, 52, 53),
        rect_of(&second, 51, 52, 53),
        "featured 盒往返几何一致（ghost 盒不进布局）",
    );
    assert_same(
        rect_of(&first, 41, 42, 43),
        rect_of(&second, 41, 42, 43),
        "伪元素盒自身往返几何一致",
    );
}
