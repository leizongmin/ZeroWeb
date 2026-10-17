//! R5166 域核探针：行内流 position:absolute 元素的静态位（css-position-3
//! §static-position + css-text-3 #line-break-details「OOF 锚不跨软换行迁移」）。
//! 断言修复后锚落在行内流消费位（文本后、行盒顶），而非 taffy block-flow cursor
//!（修复前 (0,0)/(0,下一行顶)）。
//!
//! kill-switch 反证（`ZW_OOF_INLINE_ANCHOR=0` / `ZW_PSEUDO_OOF_TEXT_BOX=0`）为进程级
//! LazyLock 快照，进程内并行测试不可靠，由 corpus OFF 轮承担（evidence 记录）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn box_pos(result: &crate::engine::LayoutResult, doc: &zero_dom::Document, id: &str) -> Option<(f32, f32)> {
    let node = doc.get_element_by_id(id)?;
    find_box(&result.root, node).map(|b| (b.x, b.y))
}

/// 行内流 abs span 的静态位 = 源序行内光标位（文本后）、行盒顶（css-position-3
/// §static-position：假设 static 时盒的位置）。修复前 taffy 给块游标位 (0,0)/(0,32)。
#[test]
fn r5166_inline_abspos_static_position_follows_inline_flow() {
    // monospace 16px：ch ≈ 9.6px；"Preceding text " 15ch ≈ 122.4px（探针实测）。
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 400px; font: 16px/2 monospace;">
Preceding text <span id=a style="position: absolute">BEFORE</span><b>target element</b><span id=b style="position: absolute">AFTER</span> following text.
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let (ax, ay) = box_pos(&result, &doc, "a").expect("span#a 盒应存在");
    let (bx, by) = box_pos(&result, &doc, "b").expect("span#b 盒应存在");
    // 行内流位：a 在前导文本后，b 在 b 元素盒右缘后（16px monospace "target element"
    // 14ch ≈ 134.4px + 边距）；同属第 1 行行顶。
    assert!(ax > 100.0, "span#a 应锚在前导文本后的行内位（x>100），实际 ({ax},{ay})");
    assert!(bx > ax + 100.0, "span#b 应锚在 b 元素盒之后的行内位，实际 ({bx},{by})");
    assert_eq!(ay, 0.0, "span#a 应在第 1 行行顶，实际 y={ay}");
    assert_eq!(by, 0.0, "span#b 应在第 1 行行顶（非下一行块游标 32），实际 y={by}");
}

/// 行内 wrapper（b 元素）内的 abs span：盒树父是 wrapper（非 IFC 容器），锚位经
/// 树内偏移折算回父内容坐标——合成位（b 原点 + span.x）= 行内流位。
#[test]
fn r5166_nested_abspos_anchor_uses_container_inline_position() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 400px; font: 16px/2 monospace;">
Preceding text <b id=wrap>target <span id=inner style="position: absolute">IN</span> element</b> following text.
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let (ix, iy) = box_pos(&result, &doc, "inner").expect("span#inner 盒应存在");
    let (bx, _) = box_pos(&result, &doc, "wrap").expect("b#wrap 盒应存在");
    // "Preceding text target " = 22ch ≈ 179.5px（16px monospace ch≈8.16）；span.x 为
    // b 内容坐标，合成 b 原点后 = 行内流位（taffy 旧行为落块游标 (0,0)）。
    let abs_x = bx + ix;
    assert!(
        (abs_x - 179.5).abs() < 2.0,
        "b 内 abs span 合成位应 ≈179.5（行内流位），实际 b.x={bx} span.x={ix} 合成={abs_x}"
    );
    assert_eq!(iy, 0.0, "应锚在第 1 行行顶，实际 y={iy}");
}

/// 软换行后的行内位：锚在换行后新行的行内光标位（消费点已在新行）。
#[test]
fn r5166_anchor_after_soft_wrap_lands_on_new_line() {
    // 20ch ≈ 192px 容器："Preceding text " + "target element"（14ch）→
    // "target" 放不下 → "element" 落第 2 行，锚（源序在 b 后）消费点在第 2 行。
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 192px; font: 16px/2 monospace;">
Preceding text <b>target element</b><span id=after style="position: absolute">A</span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let (fx, fy) = box_pos(&result, &doc, "after").expect("span#after 盒应存在");
    // 第 2 行行顶（行高 32 → y=32），行内位 = "element" 后（7ch ≈ 67.2px）。
    assert_eq!(fy, 32.0, "软换行后消费的锚应在第 2 行行顶，实际 ({fx},{fy})");
    assert!(fx > 50.0, "锚 x 应在第 2 行 element 之后的行内位，实际 x={fx}");
}
