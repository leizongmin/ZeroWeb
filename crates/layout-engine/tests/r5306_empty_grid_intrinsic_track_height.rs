//! R5306：无子项 grid 容器内在高度须计入显式轨道回归钉。
//!
//! dynamic-grid-with-auto-fill 纯 taffy 层复现：`display:Grid` 且 `child_count == 0`
//! 的节点被派发到叶子路径（`(_, false) => compute_leaf_layout`），grid-template-rows
//! 被整体忽略 → 空 grid 容器内在高度恒 0（有 min-height 时只剩钳制值）。浏览器语义
//! 是网格容器的内在尺寸由轨道求和（与是否有放置项无关），Chromium 空 grid 按轨道计高。
//! 修复：grid 派发不再按子项分岔，无子项也走 `compute_grid_layout`。
//!
//! https://drafts.csswg.org/css-grid-1/#intrinsic-sizes

use taffy::style_helpers::*;
use taffy::{TaffyTree, prelude::*};

/// 空按行 auto-fill 轨道：容器内在高度 = 轨道高 100（修复前 0）。
#[test]
fn empty_grid_auto_fill_row_gives_intrinsic_height() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let target = taffy
        .new_leaf(Style {
            display: Display::Block,
            size: Size {
                width: Dimension::length(200.0),
                height: Dimension::length(50.0),
            },
            ..Default::default()
        })
        .unwrap();
    let grid = taffy
        .new_leaf(Style {
            display: Display::Grid,
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::auto(),
            },
            grid_template_rows: vec![repeat(RepetitionCount::AutoFill, vec![length(100.0_f32)])],
            ..Default::default()
        })
        .unwrap();
    taffy.set_children(target, &[grid]).unwrap();
    taffy.compute_layout(target, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(grid).unwrap().size.height, 100.0);
}

/// 同上带 `min-height:100%`（父高 50）：轨道高 100 不被钳制值覆盖（修复前 50）。
#[test]
fn empty_grid_auto_fill_row_survives_min_height_percent() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let target = taffy
        .new_leaf(Style {
            display: Display::Block,
            size: Size {
                width: Dimension::length(200.0),
                height: Dimension::length(50.0),
            },
            ..Default::default()
        })
        .unwrap();
    let grid = taffy
        .new_leaf(Style {
            display: Display::Grid,
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::auto(),
            },
            min_size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            grid_template_rows: vec![repeat(RepetitionCount::AutoFill, vec![length(100.0_f32)])],
            ..Default::default()
        })
        .unwrap();
    taffy.set_children(target, &[grid]).unwrap();
    taffy.compute_layout(target, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(grid).unwrap().size.height, 100.0);
}

/// 固定显式轨道同域：空 grid `grid-template-rows: 100px` 内在高度 100（修复前 0）。
#[test]
fn empty_grid_definite_row_gives_intrinsic_height() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let grid = taffy
        .new_leaf(Style {
            display: Display::Grid,
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::auto(),
            },
            grid_template_rows: vec![length(100.0_f32)],
            ..Default::default()
        })
        .unwrap();
    taffy
        .compute_layout(
            grid,
            Size {
                width: AvailableSpace::Definite(800.0),
                height: AvailableSpace::MaxContent,
            },
        )
        .unwrap();
    assert_eq!(taffy.layout(grid).unwrap().size.height, 100.0);
}

/// 无轨道空 grid 语义不回归：无 template 轨道时内在高度仍为 0。
#[test]
fn empty_grid_without_tracks_keeps_zero_intrinsic_height() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let grid = taffy
        .new_leaf(Style {
            display: Display::Grid,
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::auto(),
            },
            ..Default::default()
        })
        .unwrap();
    taffy
        .compute_layout(
            grid,
            Size {
                width: AvailableSpace::Definite(800.0),
                height: AvailableSpace::MaxContent,
            },
        )
        .unwrap();
    assert_eq!(taffy.layout(grid).unwrap().size.height, 0.0);
}
