//! R1982 characterization：`overflow:auto/scroll` 滚动容器的静态 sizing 行为（DC-11）。
//!
//! DC-11 unchecked 项「Overflow: scroll/auto — 可滚动容器，scroll 偏移正确应用到子元素布局」。
//! master.md 旧记「无真正滚动容器，浏览器层手动 scroll_y 偏移」。本测试 empirical probe **静态
//! 可验**部分：滚动容器（explicit height + 溢出 content）的 **outer size 是否保持指定高度**
//! （非撑满 content）。scroll 偏移本身（动态、host 层）非静态可验，不在本测试范围。
//!
//! 实测结论（R1982）：taffy 经 converter:85 接收 overflow 字段后，**outer height 正确保持显式
//! height**（auto/scroll/visible 三种皆 height=100，不因 300px content 撑满）。结合 R1861（paint
//! clip 已工作），**DC-11 溢出的静态渲染部分已满足**；残余 = interactive scroll offset（host 层），
//! 非 rendering-compat reftest 范围。本测试作 durable regression guard：若未来改动破坏滚动容器
//! sizing（如 postprocess 误撑满），此处断言失败。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 深度优先找到第一个带 id="target" 的元素 LayoutBox。
fn find_target<'a>(root: &'a LayoutBox, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
    let is_target = root.node_id.is_some_and(|id| {
        doc.get(id).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.get_attribute("id").is_some_and(|v| v == "target"),
            _ => false,
        })
    });
    if is_target {
        return Some(root);
    }
    for child in &root.children {
        if let Some(b) = find_target(child, doc) {
            return Some(b);
        }
    }
    None
}

/// 公共：渲染 overflow=<kind> + height:100px 容器（内含 300px 子），返回容器几何。
fn render_overflow_container(overflow_kind: &str) -> (f32, f32) {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="target" style="overflow:{}; height:100px; width:200px; background:red;">
  <div style="height:300px; width:100px; background:blue;"></div>
</div>
</body></html>"#,
        overflow_kind
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let c = find_target(&result.root, &doc).expect("should find #target");
    (c.width, c.height)
}

#[test]
fn r1982_overflow_auto_container_keeps_explicit_height() {
    // chromium / CSS §11：overflow:auto 容器 outer height = 显式 100px（不撑满 300px content）。
    let (w, h) = render_overflow_container("auto");
    assert!((w - 200.0).abs() < 1.0, "overflow:auto width={} (expect 200)", w);
    assert!(
        (h - 100.0).abs() < 1.0,
        "overflow:auto height={} (expect explicit 100, not 300)",
        h
    );
}

#[test]
fn r1982_overflow_scroll_container_keeps_explicit_height() {
    let (w, h) = render_overflow_container("scroll");
    assert!((w - 200.0).abs() < 1.0, "overflow:scroll width={} (expect 200)", w);
    assert!(
        (h - 100.0).abs() < 1.0,
        "overflow:scroll height={} (expect explicit 100, not 300)",
        h
    );
}

#[test]
fn r1982_overflow_visible_container_keeps_explicit_height() {
    // 对照：overflow:visible + 显式 height → outer height 仍 = 显式值（content 溢出可见但盒高不变）。
    let (w, h) = render_overflow_container("visible");
    assert!((w - 200.0).abs() < 1.0, "overflow:visible width={} (expect 200)", w);
    assert!(
        (h - 100.0).abs() < 1.0,
        "overflow:visible height={} (expect explicit 100)",
        h
    );
}

#[test]
fn r1982_position_sticky_at_scroll0_acts_as_relative() {
    // DC-11 sticky：scroll=0 时 sticky 应等价于 relative（offset 应用）。converter:366 把 Sticky
    // 映射为 taffy Relative，engine.rs:1419 对 Relative|Sticky 应用 inset。本测试实证 sticky
    // 静态 offset 生效（动态 sticking 是 host 层，非静态可验）。
    let html = r#"<html><body style="margin:0">
<div id="prev" style="height:50px"></div>
<div id="target" style="position:sticky; top:10px; height:20px; width:80px"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let target = find_target(&result.root, &doc).expect("should find #target");
    // static 位置 y=50（前一个 50px 块之后）+ sticky/relative top:10px offset → y=60。
    assert!(
        (target.y - 60.0).abs() < 1.0,
        "position:sticky top:10px y={} (expect 60 = static 50 + offset 10)",
        target.y
    );
}
