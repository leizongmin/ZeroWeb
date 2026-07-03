//! 命中测试（spec §8.4.1 `hit_test.rs` / §8.4.3 事件路由）。
//!
//! 给定 Render tree 与一个点，返回最顶层（后绘制 = z 序最高）包含该点的可命中节点 id。

use crate::render_node::RenderNode;
use zero_ui_core::geometry::Point;
use zero_ui_core::widget::WidgetId;

/// 在 RenderNode 树上做命中测试：后序遍历子节点（子在上），首个命中即返回。
///
/// **坐标契约**（深度审查 lei-deep-review 澄清）：`node.rect`、`node.clip` 与 `point`
/// 须在同一坐标空间（通常为绝对坐标）；子节点的 rect 直接用同一空间判定，**不**递归减偏移
/// （实现把 `point` 原样传给子节点）。靠后的子节点 = z 序更高，故 `children.iter().rev()`
/// 先遍历最上层。
pub fn hit_test(node: &RenderNode, point: Point) -> Option<WidgetId> {
    // 不在节点矩形内（考虑裁剪）则跳过。
    if !node.rect.contains(point) {
        return None;
    }
    if let Some(clip) = node.clip
        && !clip.contains(point)
    {
        return None;
    }
    // 子节点在上：从后往前找。
    for child in node.children.iter().rev() {
        // 子坐标是相对父的；用父坐标直接传。
        if let Some(id) = hit_test(child, point) {
            return Some(id);
        }
    }
    Some(node.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Rect;

    fn node(id: &str, rect: Rect, children: Vec<RenderNode>) -> RenderNode {
        RenderNode {
            id: WidgetId::new(id),
            rect,
            clip: None,
            primitives: Vec::new(),
            children,
        }
    }

    #[test]
    fn topmost_child_wins() {
        // 父 100x100，两个重叠子（后绘制的 child_b 在上）。
        let tree = node(
            "root",
            Rect::from_ltrb(0.0, 0.0, 100.0, 100.0),
            vec![
                node("child_a", Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), vec![]),
                node("child_b", Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), vec![]),
            ],
        );
        // 点 (10,10) 同时落在两个子节点上 → 命中靠后（z 序更高）的 child_b。
        assert_eq!(hit_test(&tree, Point::new(10.0, 10.0)), Some(WidgetId::new("child_b")));
    }

    #[test]
    fn miss_returns_none_or_parent() {
        let tree = node(
            "root",
            Rect::from_ltrb(0.0, 0.0, 100.0, 100.0),
            vec![node("child", Rect::from_ltrb(0.0, 0.0, 10.0, 10.0), vec![])],
        );
        // 点落在父内但子外 → 命中父。
        assert_eq!(hit_test(&tree, Point::new(50.0, 50.0)), Some(WidgetId::new("root")));
        // 点在整树外 → None。
        assert_eq!(hit_test(&tree, Point::new(200.0, 200.0)), None);
    }
}
