//! R5010：vendored taffy 块容器 `position:Relative` 子盒静态流位回归钉。
//!
//! transform-input-001 带 17 案定因的纯 taffy 层复现：块容器 [前驱块(h19+mb16),
//! Relative 子(h32+mt10, inset 全 auto)] 中，Relative 子应落**静态流位**
//! （taffy 不折叠 margin → 19+16+10=45）；bug 态 = 容器原点 + 自身 margin（10，
//! 前驱块流贡献丢失，行为同 Absolute）。ZW 渲染管线实测：transform-input-001
//! REF 页 input 栈整体上移 25px 与此同构。

use taffy::{AvailableSpace, Display, Position, Style, TaffyTree};

#[test]
fn relative_child_places_at_static_flow_position() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let prev = taffy
        .new_leaf(Style {
            display: Display::Block,
            size: taffy::Size { width: taffy::Dimension::length(100.0), height: taffy::Dimension::length(19.0) },
            margin: taffy::Rect {
                left: taffy::LengthPercentageAuto::length(0.0),
                right: taffy::LengthPercentageAuto::length(0.0),
                top: taffy::LengthPercentageAuto::length(0.0),
                bottom: taffy::LengthPercentageAuto::length(16.0),
            },
            ..Default::default()
        })
        .unwrap();
    let rel = taffy
        .new_leaf(Style {
            display: Display::Block,
            position: Position::Relative,
            size: taffy::Size { width: taffy::Dimension::length(100.0), height: taffy::Dimension::length(32.0) },
            margin: taffy::Rect {
                left: taffy::LengthPercentageAuto::length(0.0),
                right: taffy::LengthPercentageAuto::length(0.0),
                top: taffy::LengthPercentageAuto::length(10.0),
                bottom: taffy::LengthPercentageAuto::length(0.0),
            },
            ..Default::default()
        })
        .unwrap();
    let root_style = Style { display: Display::Block, ..Default::default() };
    let root = taffy.new_with_children(root_style, &[prev, rel]).unwrap();
    taffy
        .compute_layout(root, taffy::Size { width: AvailableSpace::Definite(800.0), height: AvailableSpace::MaxContent })
        .unwrap();
    let loc = taffy.layout(rel).unwrap().location;
    assert!(
        (loc.y - 45.0).abs() < 0.5 || (loc.y - 35.0).abs() < 0.5,
        "relative 子应落静态流位（45，或折叠语义 35），实测 {loc:?}（bug 态 = 10）"
    );
}
