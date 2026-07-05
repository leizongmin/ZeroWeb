//! Reconcile — WidgetSpec 树到 retained HostNode 树的对齐（P0-2 拆分）。
//!
//! 入口：
//! - [`build_node`]：首次建树（递归 spec → HostNode + mount + update）
//! - [`reconcile_node`] / [`reconcile_children`]：声明树变化时按 WidgetId 复用既有实例，
//!   props 变化调 `Widget::update`，结构变化标 NEEDS_LAYOUT/NEEDS_PAINT。
//!
//! 复用规则（P0-1 key 化）：同 WidgetId 跨位置匹配，避免列表前部插入/删除触发后续同 id
//! 节点全部重建（焦点 / 文本光标 / 滚动位置等 retained 状态丢失）。

use compact_str::CompactString;
use zero_ui_core::binding::PropsMap;
use zero_ui_core::geometry::{Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::widget::{MountCtx, UpdateCtx, WidgetId, WidgetSpec};

use super::{HostNode, WidgetRegistry};

/// 复用条件：同 `WidgetId`（非匿名）+ 同 `ComponentType`（Flutter `canUpdate` 语义）。
pub(super) fn same_node(node: &HostNode, spec: &WidgetSpec) -> bool {
    match &spec.id {
        Some(id) => &node.id == id && node.component == spec.component,
        None => false,
    }
}

pub(super) fn build_node(spec: &WidgetSpec, registry: &WidgetRegistry, epoch: u32) -> HostNode {
    let id = spec.id.clone().unwrap_or_else(|| WidgetId::new("__anonymous__"));
    let mut node = HostNode {
        id: id.clone(),
        component: spec.component.clone(),
        props: spec.props.clone(),
        widget: None,
        children: Vec::with_capacity(spec.children.len()),
        cached_size: Size::ZERO,
        cached_rect: Rect::ZERO,
        invalidation: InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT,
        epoch,
        focusable: false,
        scroll_offset: 0.0,
        content_height: 0.0,
    };
    if let Some(mut w) = registry.build(spec) {
        let mut flags = InvalidationFlags::CLEAN;
        w.mount(&mut MountCtx {
            id: &node.id,
            invalidation: &mut flags,
        });
        // 初始 props 同步：受控控件（如 TextField）需从 props 初始化内部状态，
        // 而非仅在 reconcile 时才看到 props。
        w.update(
            &mut UpdateCtx {
                invalidation: &mut flags,
            },
            &node.props,
        );
        node.invalidation |= flags;
        node.focusable = w.focusable();
        node.widget = Some(w);
    }
    for child in &spec.children {
        node.children.push(build_node(child, registry, epoch));
    }
    node
}

/// 把既有节点与新 spec 对齐：props 变化 → update；children 递归 reconcile。
pub(super) fn reconcile_node(node: &mut HostNode, spec: &WidgetSpec, registry: &WidgetRegistry, epoch: u32) {
    if node.props != spec.props {
        if let Some(w) = node.widget.as_mut() {
            let mut flags = InvalidationFlags::CLEAN;
            w.update(
                &mut UpdateCtx {
                    invalidation: &mut flags,
                },
                &spec.props,
            );
            // P0-1：widget 自己在 update 里决定是否 NEEDS_LAYOUT（如 label 变长、size prop 变）。
            // 框架不再粗暴标 NEEDS_LAYOUT——只把 widget 报告的 invalidation 累加。
            // NEEDS_PAINT 始终标记（props 变化至少要重画），即便 widget 忘记标。
            node.invalidation |= flags | InvalidationFlags::NEEDS_PAINT;
        }
        node.props = spec.props.clone();
    }
    reconcile_children(&mut node.children, &spec.children, registry, epoch);
}

pub(super) fn reconcile_children(
    existing: &mut Vec<HostNode>,
    new_specs: &[WidgetSpec],
    registry: &WidgetRegistry,
    epoch: u32,
) {
    // P0-1 key 化复用：按 WidgetId 跨位置匹配，避免列表前部插入/删除导致后续同 id 节点全部重建
    // （状态丢失：焦点 / 文本光标 / 滚动位置）。
    //
    // 算法：
    // 1. 把 existing 包装成 Vec<Option<HostNode>>，便于「按 id 查并 take」。
    // 2. 第一遍：建立 id -> slot index 的索引（仅 id 非匿名节点入索引）。
    // 3. 第二遍：对每个 new_spec：
    //    - 有 id 且在索引命中 → take 该 slot 复用，标记 slot 已用。
    //    - 否则尝试按位置（existing[i]）匹配（保留无 id 子节点按 index 复用的历史行为）。
    //    - 都不命中 → build 新节点。
    // 4. 已被 take 的 slot 在第二遍按位置匹配时跳过。
    let mut slots: Vec<Option<HostNode>> = existing.drain(..).map(Some).collect();

    // id -> slot index（多 slot 同 id 取第一个，与历史「按位置首个匹配」语义最近）。
    let mut id_index: hashbrown::HashMap<CompactString, usize> = hashbrown::HashMap::with_capacity(slots.len());
    for (i, s) in slots.iter().enumerate() {
        if let Some(node) = s
            && node.id.0 != "__anonymous__"
        {
            id_index.entry(node.id.0.clone()).or_insert(i);
        }
    }

    let mut taken: Vec<bool> = vec![false; slots.len()];
    let mut next: Vec<HostNode> = Vec::with_capacity(new_specs.len());
    for (i, spec) in new_specs.iter().enumerate() {
        // 优先：按 id 命中（跨位置）。
        let reuse_slot: Option<usize> = match &spec.id {
            Some(id) if id.0 != "__anonymous__" => id_index.get(&id.0).copied(),
            _ => None,
        };
        if let Some(slot_idx) = reuse_slot
            && !taken[slot_idx]
            && let Some(node) = &slots[slot_idx]
            && same_node(node, spec)
        {
            let mut el = slots[slot_idx].take().unwrap();
            taken[slot_idx] = true;
            reconcile_node(&mut el, spec, registry, epoch);
            next.push(el);
            continue;
        }
        // 回落：按位置匹配（无 id 子节点 / id 未命中场景）。
        if i < slots.len()
            && !taken[i]
            && let Some(Some(node)) = slots.get(i)
            && same_node(node, spec)
        {
            let mut el = slots[i].take().unwrap();
            taken[i] = true;
            reconcile_node(&mut el, spec, registry, epoch);
            next.push(el);
            continue;
        }
        // 都不命中 → 新建。
        next.push(build_node(spec, registry, epoch));
    }
    *existing = next;
}

/// 用于 build_node 静默消费 PropsMap（避免未使用 import 警告）。
#[allow(dead_code)]
fn _props_unused(_: &PropsMap) {}
