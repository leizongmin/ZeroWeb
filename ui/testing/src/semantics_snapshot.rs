//! Semantics snapshot — 把 a11y 树序列化为确定性字符串（spec FR-016 testing）。

use zero_ui_core::semantics::SemanticsNode;

/// 生成 a11y 树的确定性快照（深度优先，带缩进）。
pub fn snapshot_semantics(root: &SemanticsNode) -> String {
    let mut out = String::new();
    write_node(root, 0, &mut out);
    out
}

fn write_node(node: &SemanticsNode, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    out.push_str(&format!(
        "{}{} rect={},{},{},{} flags=0x{:x}",
        indent,
        node.id.0.as_str(),
        node.rect.left(),
        node.rect.top(),
        node.rect.right(),
        node.rect.bottom(),
        node.flags.0
    ));
    if let Some(v) = &node.value {
        out.push_str(&format!(" value={}", v.as_str()));
    }
    out.push('\n');
    for c in &node.children {
        write_node(c, depth + 1, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::{Point, Rect, Size};
    use zero_ui_core::semantics::SemanticsFlags;
    use zero_ui_core::widget::WidgetId;

    #[test]
    fn snapshot_renders_tree() {
        let root = SemanticsNode {
            id: WidgetId::new("root"),
            rect: Rect::from_origin_size(Point::ZERO, Size::new(100.0, 100.0)),
            flags: SemanticsFlags::NONE,
            label: None,
            value: None,
            children: vec![SemanticsNode::new(
                WidgetId::new("btn"),
                Rect::ZERO,
                SemanticsFlags::BUTTON | SemanticsFlags::FOCUSABLE,
            )],
        };
        let snap = snapshot_semantics(&root);
        assert!(snap.contains("root rect=0,0,100,100"));
        assert!(snap.contains("btn rect=0,0,0,0 flags=0x11")); // BUTTON(0x10)|FOCUSABLE(0x01)
    }
}
