// 增量布局计算集成测试
//
// 验证 LayoutEngine 的 compute_incremental 功能：
// - 脏节点标记和增量重算
// - 全量重算退化
// - 缓存失效
// - 增量结果与全量结果一致性

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, LengthValue, PositionValue};
use zero_dom::Document;
use zero_layout_engine::{LayoutDirtyTracker, LayoutEngine};
use zero_style_system::ComputedStyle;

// ── 辅助函数 ──

/// 创建 html > body 基础 DOM，返回 (doc, html NodeId, body NodeId)。
fn make_doc_with_body() -> (Document, zero_dom::NodeId, zero_dom::NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, html, body)
}

/// 在 LayoutBox 子树中查找指定 node_id 的盒子。
fn find_box_by_node_id(
    root: &zero_layout_engine::LayoutBox,
    target_id: zero_dom::NodeId,
) -> Option<&zero_layout_engine::LayoutBox> {
    if root.node_id == Some(target_id) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_box_by_node_id(child, target_id) {
            return Some(found);
        }
    }
    None
}

/// 创建包含 3 个 block 子元素的 DOM 和样式。
fn make_three_block_children() -> (Document, zero_dom::NodeId, Vec<zero_dom::NodeId>) {
    let (mut doc, _html, body) = make_doc_with_body();
    let mut children = Vec::new();
    for i in 0..3 {
        let div = doc.create_element("div");
        // 给每个子元素设置不同的 data-index 以便后续识别
        doc.set_attribute(div, "data-index", &i.to_string());
        doc.append_child(body, div).unwrap();
        let text = doc.create_text_node(&format!("child-{i}"));
        doc.append_child(div, text).unwrap();
        children.push(div);
    }
    (doc, body, children)
}

// ── 测试 ──

/// 增量布局 — 基本流程：全量计算后标记脏节点触发增量重算。
///
/// 验证：
/// 1. 首次 compute() 后 has_cached_state() == true
/// 2. 标记脏节点后 compute_incremental() 返回 was_full_recalc == false
/// 3. 增量布局结果非空
#[test]
fn test_incremental_basic_flow() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    // 全量计算
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let full_result = engine.compute(&doc, &styles);
    assert!(engine.has_cached_state(), "全量计算后应有缓存状态");

    // 标记脏节点并增量计算
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[1]);

    let (inc_result, stats) =
        engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc, "有缓存时应为增量计算");
    assert_eq!(stats.dirty_node_count, 1, "应标记 1 个脏节点");
    assert!(stats.layout_ms >= 0.0, "布局耗时应为非负");

    // 增量结果应与全量一致（未改变样式）
    let full_child = find_box_by_node_id(&full_result.root, children[0]).unwrap();
    let inc_child = find_box_by_node_id(&inc_result.root, children[0]).unwrap();
    assert_eq!(full_child.width, inc_child.width, "未修改节点宽度应不变");
    assert_eq!(full_child.height, inc_child.height, "未修改节点高度应不变");
}

/// 增量布局 — 修改样式后全量重算结果正确。
///
/// 验证：修改子元素高度后，标记 full_recalc 触发全量重算，布局盒反映新值。
#[test]
fn test_incremental_style_change_full_recalc() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    // 全量计算
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);
    assert!(engine.has_cached_state());

    // 修改第一个子元素的高度
    styles.get_mut(&children[0]).unwrap().height = LengthValue::Px(100.0);

    // 样式变更需全量重算（增量不更新 taffy node style）
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_full_recalc();

    let (result, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(stats.was_full_recalc, "样式变更应触发全量重算");

    // 验证修改的节点高度变了
    let box0 = find_box_by_node_id(&result.root, children[0]).unwrap();
    assert_eq!(box0.height, 100.0, "高度应更新为 100px");
}

/// 增量布局 — 全量重算退化。
///
/// 当 dirty_tracker 标记 full_recalc 时，应退化为全量计算。
#[test]
fn test_incremental_full_recalc_fallback() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 标记全量重算
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_full_recalc();

    let (_, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(stats.was_full_recalc, "标记全量重算时应退化为全量计算");
}

/// 增量布局 — 无缓存时退化为全量计算。
///
/// 新建引擎未调用 compute() 时，compute_incremental 应退化为全量。
#[test]
fn test_incremental_no_cache_fallback() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    assert!(!engine.has_cached_state(), "新建引擎不应有缓存");

    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[0]);

    let (result, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(stats.was_full_recalc, "无缓存时应退化为全量计算");
    assert!(engine.has_cached_state(), "退化后应有缓存");

    // 结果应正确
    let box0 = find_box_by_node_id(&result.root, children[0]).unwrap();
    assert_eq!(box0.width, 200.0);
    assert_eq!(box0.height, 50.0);
}

/// 增量布局 — 缓存失效。
///
/// 调用 invalidate_cache() 后，后续 compute_incremental 应退化为全量。
#[test]
fn test_incremental_cache_invalidation() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);
    assert!(engine.has_cached_state());

    // 失效缓存
    engine.invalidate_cache();
    assert!(!engine.has_cached_state(), "失效后不应有缓存");

    // 增量计算应退化为全量
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[0]);

    let (_, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(stats.was_full_recalc, "缓存失效后应退化为全量");
}

/// 增量布局 — set_viewport 导致缓存失效。
///
/// 改变视口大小后，缓存自动失效，增量退化为全量。
#[test]
fn test_incremental_viewport_change() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 改变视口
    engine.set_viewport(1024.0, 768.0);
    assert!(!engine.has_cached_state(), "视口变化后缓存应失效");
    assert_eq!(engine.viewport_width, 1024.0);
    assert_eq!(engine.viewport_height, 768.0);

    // 重新全量计算
    let result = engine.compute(&doc, &styles);
    assert_eq!(result.viewport_width, 1024.0);
    assert_eq!(result.viewport_height, 768.0);
}

/// 增量布局 — 多次增量计算连续执行。
///
/// 验证：连续多次增量计算不 panic，缓存状态持续有效。
#[test]
fn test_incremental_multiple_rounds() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 第 1 轮：标记脏节点增量计算
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[0]);
    let (_, s1) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(!s1.was_full_recalc);
    assert!(engine.has_cached_state());

    // 第 2 轮：标记另一个脏节点
    let mut tracker2 = LayoutDirtyTracker::new();
    tracker2.mark_dirty(children[2]);
    let (_, s2) = engine.compute_incremental(&doc, &styles, &mut tracker2, &std::collections::HashMap::new());
    assert!(!s2.was_full_recalc);
    assert!(engine.has_cached_state());

    // 第 3 轮：全量重算
    let mut tracker3 = LayoutDirtyTracker::new();
    tracker3.mark_full_recalc();
    let (_, s3) = engine.compute_incremental(&doc, &styles, &mut tracker3, &std::collections::HashMap::new());
    assert!(s3.was_full_recalc);
    assert!(engine.has_cached_state());
}

/// 增量布局 — 多脏节点同时重算。
///
/// 同时标记多个脏节点，增量计算不 panic，统计正确。
#[test]
fn test_incremental_multiple_dirty_nodes() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 标记所有子元素为脏
    let mut tracker = LayoutDirtyTracker::new();
    for &child_id in &children {
        tracker.mark_dirty(child_id);
    }

    let (result, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc);
    assert_eq!(stats.dirty_node_count, 3, "应标记 3 个脏节点");

    // 所有节点仍存在
    for &child_id in &children {
        assert!(
            find_box_by_node_id(&result.root, child_id).is_some(),
            "所有子元素应存在"
        );
    }
}

/// 增量布局 — 相同样式下增量与全量结果一致性。
///
/// 不修改样式时，增量计算结果应与全量计算结果数值一致。
#[test]
fn test_incremental_vs_full_consistency() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 增量计算（不修改样式，仅标记脏）
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[1]);
    let (inc_result, _) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());

    // 全量计算（新引擎）
    let mut engine2 = LayoutEngine::new(800.0, 600.0);
    let full_result = engine2.compute(&doc, &styles);

    // 比较结果：不修改样式时两者应一致
    for &child_id in &children {
        let inc_box = find_box_by_node_id(&inc_result.root, child_id).unwrap();
        let full_box = find_box_by_node_id(&full_result.root, child_id).unwrap();
        assert_eq!(inc_box.width, full_box.width, "宽度应一致");
        assert_eq!(inc_box.height, full_box.height, "高度应一致");
        assert_eq!(inc_box.x, full_box.x, "x 应一致");
        assert_eq!(inc_box.y, full_box.y, "y 应一致");
    }
}

/// 增量布局 — absolute 定位元素增量更新。
#[test]
fn test_incremental_positioned_element() {
    let (mut doc, _html, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let abs_child = doc.create_element("div");
    doc.append_child(container, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(300.0);
    container_style.position = PositionValue::Relative;
    styles.insert(container, container_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.display = DisplayValue::Block;
    abs_style.position = PositionValue::Absolute;
    abs_style.width = LengthValue::Px(100.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    // 修改 absolute 元素的位置
    styles.get_mut(&abs_child).unwrap().top = LengthValue::Px(20.0);
    styles.get_mut(&abs_child).unwrap().left = LengthValue::Px(30.0);

    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(abs_child);
    let (result, stats) = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc);

    let abs_box = find_box_by_node_id(&result.root, abs_child).unwrap();
    assert!(abs_box.is_absolute, "应是 absolute 定位");
    assert_eq!(abs_box.width, 100.0);
    assert_eq!(abs_box.height, 50.0);
}

/// 增量布局 — dirty tracker drain 后为空。
#[test]
fn test_incremental_tracker_cleared_after_use() {
    let (doc, _body, children) = make_three_block_children();
    let mut styles = HashMap::new();
    for &child_id in &children {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(child_id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let _ = engine.compute(&doc, &styles);

    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(children[0]);
    tracker.mark_dirty(children[1]);
    assert!(tracker.has_dirty());

    let _ = engine.compute_incremental(&doc, &styles, &mut tracker, &std::collections::HashMap::new());
    // tracker 已被 drain_dirty 清空
    assert!(!tracker.has_dirty(), "tracker 应在增量计算后被清空");
}
