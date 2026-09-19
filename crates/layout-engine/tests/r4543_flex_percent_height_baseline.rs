//! R4543（P 轮）：flex item 子节点百分比高度解析的 taffy 层基线锚。
//!
//! 背景：ZW 渲染「flex 容器（height:auto）+ flex item（definite height 100px）+
//! 块级子 div（height:100%）」时子高度解析为 0（不绘制）；本文件验证该场景在
//! taffy 层（crates/taffy-local，纯 API）四种形态下全部正确解析——即 bug 不在
//! taffy，而在 ZW 的 taffy 树构建/样式转换/后处理链（layout-engine 集成面）。
//! 这五条基线同时锚定 taffy fork 的行为：若未来 fork 改动使任一条转红，
//! 即为 taffy 层回归，须先于 ZW 集成面修复。
//! 诊断链记档：docs/goal/rendering-compat/evidence/r4543-flex-percent-height-2026-09-20.md

//! R4543 最小 taffy repro：flex item（definite height）的 block 子节点 height:100% 解析。

use taffy::prelude::*;

fn build(
    container_height: taffy::style::Dimension,
    item_height: taffy::style::Dimension,
) -> (TaffyTree<()>, taffy::NodeId, taffy::NodeId) {
    let mut tree: taffy::TaffyTree<()> = TaffyTree::new();
    let child = tree
        .new_leaf(Style {
            display: Display::Block,
            size: Size {
                width: length(50.0_f32),
                height: percent(1.0_f32),
            },
            ..Default::default()
        })
        .unwrap();
    let item = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: Size {
                    width: length(100.0_f32),
                    height: item_height,
                },
                ..Default::default()
            },
            &[child],
        )
        .unwrap();
    let container = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                size: Size {
                    width: auto(),
                    height: container_height,
                },
                ..Default::default()
            },
            &[item],
        )
        .unwrap();
    (tree, container, child)
}

fn compute_and_measure(container_height: taffy::style::Dimension, item_height: taffy::style::Dimension) -> f32 {
    let (mut tree, container, child) = build(container_height, item_height);
    tree.compute_layout(
        container,
        taffy::Size::<taffy::AvailableSpace> {
            width: taffy::AvailableSpace::Definite(200.0),
            height: taffy::AvailableSpace::Definite(200.0),
        },
    )
    .unwrap();
    tree.layout(child).unwrap().size.height
}

/// 基线场景（R4543 探针形态）：flex 容器高 auto + item definite 100px + 子 height:100%。
#[test]
fn flex_item_child_percent_height_resolves() {
    let h = compute_and_measure(auto(), length(100.0_f32));
    assert!(
        (h - 100.0).abs() < 0.01,
        "flex item 子节点 height:100% 应解析为 100，实际 {h}"
    );
}

/// 控制组 1：block 父（非 flex 子树）definite 高 + 子 height:100%。
#[test]
fn block_parent_child_percent_height_resolves() {
    let h = compute_and_measure_block();
    assert!(
        (h - 100.0).abs() < 0.01,
        "block 父子节点 height:100% 应解析为 100，实际 {h}"
    );
}

fn compute_and_measure_block() -> f32 {
    let mut tree: taffy::TaffyTree<()> = TaffyTree::new();
    let child = tree
        .new_leaf(Style {
            display: Display::Block,
            size: Size {
                width: length(50.0_f32),
                height: percent(1.0_f32),
            },
            ..Default::default()
        })
        .unwrap();
    let parent = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: Size {
                    width: length(100.0_f32),
                    height: length(100.0_f32),
                },
                ..Default::default()
            },
            &[child],
        )
        .unwrap();
    tree.compute_layout(
        parent,
        taffy::Size::<taffy::AvailableSpace> {
            width: taffy::AvailableSpace::Definite(200.0),
            height: taffy::AvailableSpace::Definite(200.0),
        },
    )
    .unwrap();
    tree.layout(child).unwrap().size.height
}

/// 变体：容器 height definite + item definite。
#[test]
fn flex_container_definite_height_child_percent() {
    let h = compute_and_measure(length(100.0_f32), length(100.0_f32));
    println!("container-definite: child h = {h}");
    assert!(
        (h - 100.0).abs() < 0.01,
        "容器 definite 时子 height:100% 应解析为 100，实际 {h}"
    );
}

/// 变体：item 无自身 height，靠 stretch（容器 definite height）。
#[test]
fn flex_item_stretch_child_percent() {
    let h = compute_and_measure(length(100.0_f32), auto());
    println!("item-stretch: child h = {h}");
    assert!(
        (h - 100.0).abs() < 0.01,
        "stretch item 子 height:100% 应解析为 100，实际 {h}"
    );
}

/// 变体：走 compute_layout_with_measure（镜像 ZW 管线）。
#[test]
fn flex_item_child_percent_with_measure() {
    let (mut tree, container, child) = build(auto(), length(100.0_f32));
    let available_space = taffy::Size::<taffy::AvailableSpace> {
        width: taffy::AvailableSpace::Definite(200.0),
        height: taffy::AvailableSpace::Definite(200.0),
    };
    let _ = tree.compute_layout_with_measure(container, available_space, |_, _, _, _, _| taffy::Size::ZERO);
    let h = tree.layout(child).unwrap().size.height;
    println!("with-measure: child h = {h}");
    assert!(
        (h - 100.0).abs() < 0.01,
        "measure 管线下子 height:100% 应解析为 100，实际 {h}"
    );
}
