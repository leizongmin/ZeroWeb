//! s372（WPT html/semantics dialog-popover-closedby-complex）：`[popover]` UA
//! `position: fixed`（rendering.html#css-user-agent-style-sheet）落地后的几何契约
//! ——popover 出流 + 收缩宽度不塌 0，dialog/popover 盒互不重叠（light-dismiss
//! click 命中前提）。诊断期 eprintln 全盒 dump，定谳后收紧断言。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn dump(name: &str, b: &LayoutBox, depth: usize) {
    eprintln!(
        "{}{:?} node={:?} pos=({:.1},{:.1}) size={:.1}x{:.1} abs={} fixed={}",
        "  ".repeat(depth),
        name,
        b.node_id,
        b.x,
        b.y,
        b.width,
        b.height,
        b.is_absolute,
        b.is_fixed
    );
    for c in &b.children {
        dump("child", c, depth + 1);
    }
}

#[test]
fn s372_popover_ua_fixed_geometry_probe() {
    let html = r#"<html><body style="margin:0">
<dialog class=dialogA open style="position: fixed; top: 100px; bottom: auto; padding:0; margin:0;">Dialog 1
  <div class=popoverA popover style="top: 150px; bottom: auto; padding:0; margin:0;">Popover 1
    <dialog class=dialogB open style="position: fixed; top: 200px; bottom: auto; padding:0; margin:0;">Dialog 2
      <div class=popoverB popover style="top: 250px; bottom: auto; padding:0; margin:0;">Popover 2</div>
    </dialog>
  </div>
</dialog>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    for (cls, tag) in [
        ("dialogA", "dialog"),
        ("popoverA", "div"),
        ("dialogB", "dialog"),
        ("popoverB", "div"),
    ] {
        let id = doc
            .get_elements_by_tag_name(tag)
            .into_iter()
            .find(|&id| {
                doc.get(id)
                    .and_then(|n| match &n.kind {
                        zero_dom::NodeKind::Element(e) => Some(e.get_attribute("class").map(|s| s.to_string())),
                        _ => None,
                    })
                    .flatten()
                    .is_some_and(|c| c == cls)
            })
            .expect(cls);
        let b = find_box(&result.root, id).expect("box");
        dump(cls, b, 0);
    }
    // 宽度契约：全部盒 shrink-to-fit 非 0（塌 0 补测臂吃到 fixed）；叶盒 popoverB
    // 文本测量 62 基准锚（taffy R5104 measure 臂既有语义）。
    for (cls, tag) in [
        ("dialogA", "dialog"),
        ("popoverA", "div"),
        ("dialogB", "dialog"),
        ("popoverB", "div"),
    ] {
        let id = doc
            .get_elements_by_tag_name(tag)
            .into_iter()
            .find(|&id| {
                doc.get(id)
                    .and_then(|n| match &n.kind {
                        zero_dom::NodeKind::Element(e) => Some(e.get_attribute("class").map(|s| s.to_string())),
                        _ => None,
                    })
                    .flatten()
                    .is_some_and(|c| c == cls)
            })
            .expect(cls);
        let b = find_box(&result.root, id).expect("box");
        assert!(
            b.width > 0.5,
            "s372: {cls} fixed 盒宽应 shrink-to-fit 非 0（塌 0 补测扩展），实际 {}",
            b.width
        );
        // 单行文本高（宽收缩后折行消失）——真浏览器同形 18px 行盒。
        assert!(
            b.height <= 20.0,
            "s372: {cls} fixed 盒高应单行（宽塌 0 折行消失），实际 {}",
            b.height
        );
    }
    // 叶盒锚：popoverB 纯文本单行 max-content（默认 16px sans ≈74；父 CB 链传播后
    // taffy measure 可用空间充足，窄可用空间伪影钳 62 不复现）。
    let pb = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .find(|&id| {
            doc.get(id)
                .and_then(|n| match &n.kind {
                    zero_dom::NodeKind::Element(e) => Some(e.get_attribute("class").map(|s| s.to_string())),
                    _ => None,
                })
                .flatten()
                .is_some_and(|c| c == "popoverB")
        })
        .expect("popoverB");
    let pb_box = find_box(&result.root, pb).expect("popoverB box");
    assert!(
        (pb_box.width - 74.0).abs() < 3.0,
        "s372: popoverB 叶盒文本测量锚应 74±3（单行 max-content），实际 {}",
        pb_box.width
    );
}
