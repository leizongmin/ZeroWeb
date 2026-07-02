//! 覆盖率补充：runtime 的 UiTree / AccessibilityTree / ImeController。

use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsNode};
use zero_ui_core::widget::{WidgetId, WidgetSpec};
use zero_ui_runtime::{AccessibilityTree, ImeController, UiTree};

#[test]
fn ui_tree_set_root_and_pending_invalidation() {
    let mut tree = UiTree::new();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    tree.set_root(&root);
    // 首次建树 → layout + paint。
    let pending = tree.take_pending();
    assert!(pending.contains(InvalidationFlags::NEEDS_LAYOUT));
    assert!(pending.contains(InvalidationFlags::NEEDS_PAINT));
    // take_pending 清空。
    assert!(tree.take_pending().is_clean());

    // 再次 set_root → reconcile，至少 paint。
    tree.set_root(&root);
    assert!(tree.take_pending().contains(InvalidationFlags::NEEDS_PAINT));

    // mark 外部失效。
    tree.mark(InvalidationFlags::NEEDS_SEMANTICS);
    assert!(tree.take_pending().contains(InvalidationFlags::NEEDS_SEMANTICS));
}

#[test]
fn accessibility_tree_collects_focusables() {
    let mut a11y = AccessibilityTree::new();
    let mut root = SemanticsNode::new(
        WidgetId::new("root"),
        Rect::from_origin_size(Point::ZERO, Size::new(10.0, 10.0)),
        SemanticsFlags::NONE,
    );
    root.children.push(SemanticsNode::new(
        WidgetId::new("btn"),
        Rect::ZERO,
        SemanticsFlags::BUTTON | SemanticsFlags::FOCUSABLE,
    ));
    a11y.set_root(root);
    assert_eq!(a11y.focusables(), vec![WidgetId::new("btn")]);
    assert!(a11y.root().is_some());
}

#[test]
fn ime_controller_change_detection() {
    let mut ime = ImeController::new();
    assert!(ime.update(Some(Rect::ZERO)));
    assert!(!ime.update(Some(Rect::ZERO)));
    assert_eq!(ime.current(), Some(Rect::ZERO));
}
