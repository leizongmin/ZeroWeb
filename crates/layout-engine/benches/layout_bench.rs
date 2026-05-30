//! 布局引擎性能基准测试。
#![allow(clippy::field_reassign_with_default)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};
use zero_dom::Document;
use zero_layout_engine::LayoutEngine;
use zero_style_system::ComputedStyle;

/// 创建 N 个 block 元素的 DOM + 样式。
fn make_block_doc(n: usize) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let mut styles = HashMap::new();
    for _ in 0..n {
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Block;
        style.width = LengthValue::Px(100.0);
        style.height = LengthValue::Px(30.0);
        styles.insert(div, style);
    }

    (doc, styles)
}

/// 创建 N 个 flex 子项的 DOM + 样式。
fn make_flex_doc(n: usize) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let container = doc.create_element("div");
    doc.append_child(html, container).unwrap();

    let mut styles = HashMap::new();

    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.width = LengthValue::Px(8000.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for _ in 0..n {
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(item, item_style);
    }

    (doc, styles)
}

/// 创建 rows x cols 的 grid DOM + 样式。
fn make_grid_doc(rows: usize, cols: usize) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let grid = doc.create_element("div");
    doc.append_child(html, grid).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.width = LengthValue::Px(800.0);
    grid_style.height = LengthValue::Px(800.0);
    styles.insert(grid, grid_style);

    for _ in 0..rows * cols {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(item, item_style);
    }

    (doc, styles)
}

/// 创建 depth 层嵌套的 DOM + 样式。
fn make_deep_nesting_doc(depth: usize) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let mut styles = HashMap::new();
    let mut current = html;

    for _ in 0..depth {
        let div = doc.create_element("div");
        doc.append_child(current, div).unwrap();
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(100.0);
        style.height = LengthValue::Px(100.0);
        styles.insert(div, style);
        current = div;
    }

    (doc, styles)
}

/// 创建 N 个兄弟元素的 DOM + 样式。
fn make_wide_doc(n: usize) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let mut styles = HashMap::new();
    for _ in 0..n {
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(50.0);
        style.height = LengthValue::Px(30.0);
        styles.insert(div, style);
    }

    (doc, styles)
}

/// 基准：1000 个 block 元素的布局。
fn bench_block_layout_1000_elements(c: &mut Criterion) {
    let (doc, styles) = make_block_doc(1000);
    c.bench_function("block_layout_1000_elements", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(800.0, 600.0);
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

/// 基准：1000 个 flex 子项的布局。
fn bench_flex_layout_1000_elements(c: &mut Criterion) {
    let (doc, styles) = make_flex_doc(1000);
    c.bench_function("flex_layout_1000_elements", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(8000.0, 600.0);
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

/// 基准：10x10 grid 布局。
fn bench_grid_layout_100_elements(c: &mut Criterion) {
    let (doc, styles) = make_grid_doc(10, 10);
    c.bench_function("grid_layout_100_elements", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(800.0, 800.0);
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

/// 基准：50 层深度嵌套。
fn bench_deep_nesting_50_levels(c: &mut Criterion) {
    let (doc, styles) = make_deep_nesting_doc(50);
    c.bench_function("deep_nesting_50_levels", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(800.0, 600.0);
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

/// 基准：500 个兄弟元素。
fn bench_wide_tree_500_children(c: &mut Criterion) {
    let (doc, styles) = make_wide_doc(500);
    c.bench_function("wide_tree_500_children", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(800.0, 6000.0);
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

/// 基准：增量布局（计算两次，模拟更新）。
fn bench_incremental_layout(c: &mut Criterion) {
    let (doc, styles) = make_block_doc(500);
    c.bench_function("incremental_layout", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(800.0, 600.0);
            // 第一次布局
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
            // 第二次布局（模拟增量更新）
            black_box(engine.compute(black_box(&doc), black_box(&styles)));
        })
    });
}

criterion_group!(
    benches,
    bench_block_layout_1000_elements,
    bench_flex_layout_1000_elements,
    bench_grid_layout_100_elements,
    bench_deep_nesting_50_levels,
    bench_wide_tree_500_children,
    bench_incremental_layout,
);
criterion_main!(benches);
