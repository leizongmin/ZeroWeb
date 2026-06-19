#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

//! CSS resize / scroll-container / overflow-hidden 渲染集成测试。
//!
//! 从 `visual.rs` 拆分以控制单文件体积（2000 行规则）。这三个主题同属
//! 「滚动 / 调整大小 / 溢出」渲染集成聚类，合并到本模块。`make_box` 复用
//! `visual.rs` 的 `pub(super)` 实现。

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::visual::make_box;

// ═══════════════════════════════════════════════════════════════
//  CSS resize 渲染集成测试
// ═══════════════════════════════════════════════════════════════

/// 测试 resize:both 生成调整手柄 stroke 图元。
#[test]
fn test_resize_both_generates_strokes() {
    use zero_style_system::ResizeValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.resize = ResizeValue::Both;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().strokes.len() >= 3,
        "resize:both should generate at least 3 stroke primitives, got {}",
        painter.primitives().strokes.len()
    );
}

/// 测试 resize:none 不生成调整手柄。
#[test]
fn test_resize_none_no_extra_strokes() {
    use zero_style_system::ResizeValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.resize = ResizeValue::None;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().strokes.len(),
        0,
        "resize:none should not generate resize handle strokes"
    );
}

/// 测试 resize:horizontal 生成水平手柄 stroke 图元。
#[test]
fn test_resize_horizontal_generates_strokes() {
    use zero_style_system::ResizeValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.resize = ResizeValue::Horizontal;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().strokes.len() >= 2,
        "resize:horizontal should generate at least 2 stroke primitives, got {}",
        painter.primitives().strokes.len()
    );
}

// ── 滚动容器偏移测试 ──────────────────────────────────────────

/// 测试 overflow:scroll + scroll_y 偏移使子元素向上移动。
#[test]
fn test_scroll_container_offset_y() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // 子元素在 y=50 位置，100x100 大小
    let child_box = make_box(Some(child), 0.0, 50.0, 100.0, 100.0);
    let mut parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 100.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Scroll,
        ..Default::default()
    };
    // 滚动 30px，子元素应向上移动 30px
    parent_box.scroll_y = 30.0;

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 子元素原始位置: y=50, scroll_y=30 → 实际绘制位置 y=50-30=20
    // 被裁剪到 content area [0, 100]，所以 fill.rect.origin.y 应为 20
    let fill = &painter.primitives().fills[0];
    assert!(
        (fill.rect.origin.y - 20.0).abs() < 0.01,
        "child y should be 20 after scroll offset, got {}",
        fill.rect.origin.y
    );
}

/// 测试 overflow:scroll 无滚动偏移时子元素位置不变。
#[test]
fn test_scroll_container_zero_offset() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 100.0, 100.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 100.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Scroll,
        overflow_y: OverflowClip::Scroll,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // scroll_x=0, scroll_y=0 → 子元素位置不变
    let fill = &painter.primitives().fills[0];
    assert!(
        fill.rect.origin.x.abs() < 0.01,
        "child x should be 0 with zero scroll offset, got {}",
        fill.rect.origin.x
    );
    assert!(
        fill.rect.origin.y.abs() < 0.01,
        "child y should be 0 with zero scroll offset, got {}",
        fill.rect.origin.y
    );
}

/// 测试 overflow:hidden 不应用滚动偏移（hidden 不是滚动容器）。
#[test]
fn test_overflow_hidden_no_scroll_offset() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 50.0, 100.0, 100.0);
    let mut parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 100.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };
    // 即使设置了 scroll_y，overflow:hidden 不应用滚动偏移
    parent_box.scroll_y = 50.0;

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 128, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // overflow:hidden 不应用滚动偏移，子元素位置不变
    let fill = &painter.primitives().fills[0];
    assert!(
        (fill.rect.origin.y - 50.0).abs() < 0.01,
        "overflow:hidden should not apply scroll offset, child y should be 50, got {}",
        fill.rect.origin.y
    );
}
