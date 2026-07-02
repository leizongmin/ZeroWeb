//! 覆盖率补充：render 层的 Layer / RenderNode / Scene 构造与组合。

use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::WidgetId;
use zero_ui_render::Layer;
use zero_ui_render::render_node::{RenderNode, RenderPrimitive};
use zero_ui_render::scene::{Scene, SceneEntry};

#[test]
fn layer_default_opacity_and_offset() {
    let layer = Layer::new(WidgetId::new("card"), Rect::from_ltrb(0.0, 0.0, 10.0, 10.0));
    assert_eq!(layer.opacity, 1.0);
    assert_eq!(layer.offset, zero_ui_core::geometry::Vec2::ZERO);
    assert_eq!(layer.id, WidgetId::new("card"));
}

#[test]
fn render_node_holds_primitives_and_children() {
    let mut parent = RenderNode::new(WidgetId::new("col"), Rect::ZERO);
    parent.primitives.push(RenderPrimitive::FillRect {
        rect: Rect::from_ltrb(0.0, 0.0, 100.0, 40.0),
        color: Color::BLACK,
        rounding: zero_ui_core::geometry::Rounding::ZERO,
    });
    let child = RenderNode::new(WidgetId::new("btn"), Rect::from_ltrb(0.0, 0.0, 40.0, 20.0));
    parent.children.push(child);
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.primitives.len(), 1);
}

#[test]
fn scene_extend_merges_entries() {
    let mut a = Scene::new();
    a.push(SceneEntry {
        source: WidgetId::new("x"),
        clip: None,
        primitive: RenderPrimitive::FillRect {
            rect: Rect::from_origin_size(Point::ZERO, Size::new(1.0, 1.0)),
            color: Color::WHITE,
            rounding: zero_ui_core::geometry::Rounding::ZERO,
        },
    });
    let mut b = a.clone();
    b.extend(&a);
    assert_eq!(b.entries.len(), 2);
}
