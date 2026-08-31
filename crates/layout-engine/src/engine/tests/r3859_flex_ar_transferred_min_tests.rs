//! R3859 回归测试：flex item「definite min cross × ratio → min main」transferred size
//! suggestion（CSS Flexbox §4 / css-sizing-4 §4）。
//!
//! taffy 0.12 不解 min-cross→min-main 传递：flex column 容器 width:0 + item
//! `aspect-ratio:1/1; min-width:100px; min-height:0` → item min main(height) 应 =
//! 100×1 = 100（min-width 经 ratio 传递），taffy 塌 0 高。R3859 在
//! `apply_flex_aspect_ratio_item_size` 的 R1013 skip 分支补 min 提升喂给 taffy。
//! driving：flex-aspect-ratio-034/035/036/039（绿方块整体缺失 1.04-2.08%）。
//! R1013 baseline 守卫：css-flexbox 目录失败集合逐字节不变（flex-item-transferred-sizes-padding
//! 回归 +73pp 陷阱未复触——cross 轴有 padding 时跳过）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

#[test]
fn r3859_flex_item_min_cross_transfers_to_min_main() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut styles = std::collections::HashMap::new();
    // 容器：flex column，width 0（definite cross 不可用 → cross 来自 min-width）。
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Flex;
    cs.flex_direction = FlexDirectionValue::Column;
    cs.width = LengthValue::Px(0.0);
    styles.insert(container, cs);
    // item：aspect 1/1 + min-width 100（definite min cross）+ min-height 0 + main(height) auto。
    let mut is = ComputedStyle::default();
    is.display = DisplayValue::Block;
    is.aspect_ratio = Some(1.0);
    is.min_width = LengthValue::Px(100.0);
    is.min_height = LengthValue::Px(0.0);
    styles.insert(item, is);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let item_box = find(&result.root, item).expect("item box");

    assert!(
        (item_box.height - 100.0).abs() < 0.5,
        "R3859: min cross(100) × ratio(1) 应传递为 min main(height)=100，实际 h={}",
        item_box.height
    );
    assert!(
        (item_box.width - 100.0).abs() < 0.5,
        "item width 应保持 min-width=100，实际 w={}",
        item_box.width
    );
}

/// R3862：显式 `min-width:0` 不构成 definite min 约束——stretch cross 传递（R1364）
/// 应正常触发（flex-aspect-ratio-009：容器 height:100 + item aspect 1/1 + min-width:0
/// → stretched cross 100 传 main width=100，ZW 曾塌 0 宽）。
#[test]
fn r3862_zero_main_min_does_not_block_stretch_transfer() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut styles = std::collections::HashMap::new();
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Flex;
    cs.height = LengthValue::Px(100.0); // definite cross → align stretch
    styles.insert(container, cs);
    let mut is = ComputedStyle::default();
    is.display = DisplayValue::Block;
    is.aspect_ratio = Some(1.0);
    is.min_width = LengthValue::Px(0.0);
    styles.insert(item, is);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let item_box = find(&result.root, item).expect("item box");
    assert!(
        (item_box.width - 100.0).abs() < 0.5,
        "R3862: stretched cross(100) × ratio(1) 应传 main width=100，实际 w={}（min-width:0 \
         不应挡 R1364 传递）",
        item_box.width
    );
}

/// R3862 对称守卫：cross 轴 **auto margin** 吸收空间 → item 不被 stretch，禁止用容器
/// cross 传 main（flex-aspect-ratio-010：`margin: auto 0` 居中场景 cross 保持 0）。
#[test]
fn r3862_cross_auto_margin_blocks_stretch_transfer() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut styles = std::collections::HashMap::new();
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Flex;
    cs.height = LengthValue::Px(100.0);
    styles.insert(container, cs);
    let mut is = ComputedStyle::default();
    is.display = DisplayValue::Block;
    is.aspect_ratio = Some(1.0);
    is.min_width = LengthValue::Px(0.0);
    is.margin_top = LengthValue::Auto;
    is.margin_bottom = LengthValue::Auto;
    styles.insert(item, is);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let item_box = find(&result.root, item).expect("item box");
    assert!(
        item_box.width < 0.5,
        "R3862: cross 轴 auto margin（不 stretch）时不得用容器 cross 传 main，width 应保持 \
         0，实际 w={}",
        item_box.width
    );
}
