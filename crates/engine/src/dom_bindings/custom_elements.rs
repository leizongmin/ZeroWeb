//! P1b S5c custom element lifecycle 桥接（R3266，RFC §3.5.1 S5c）。
//!
//! native_dom 路径 `appendChild`/`insertBefore`/`removeChild` 经 Rust 直接改 live Document，**绕过**
//! polyfill 的 `_ceApplyConn`（基于 sel/handle 的连接态派发），故 connectedCallback/disconnectedCallback
//! 不触发。本模块桥接：DOM mutation 成功后收集受影响子树的 custom 元素，判定连接状态变化（Rust
//! `CONNECTED_CUSTOM` 权威追踪），状态真转时调 polyfill JS `__zw_native_ce_notify_connect` 派发回调
//!（以 native 实例作 `this`，复用 polyfill `_ce_registry` + ctor.prototype）。
//!
//! **职责分离**：Rust 负责树逻辑（什么变了、连没连）——有 DOM 树权威；JS 负责 JS 对象逻辑（调
//! ctor.prototype 回调）——有 ctor/prototype 引用。避免在 Rust 重写 `_ceApplyConn` 的 sel/handle 体系。
//!
//! **连接判定**（headless 近似）：节点 parent 链到达 document root（祖先 parent=None）= 连入 document。
//! detached 容器（createElement 后未 appendChild 到 document）的 appendChild 不触发 connected。
//!
//! **切片边界**：本片处理被 append/remove 的子树中所有 custom 元素的连接态变化（DFS 子树，镜像
//! polyfill `_ceApplyConn` 子树传播）。attributeChangedCallback 经 setAttribute polyfill trap 已就绪
//!（native_dom 下 setAttribute 走 Rust，attr change 派发为 S5c 后续 / S5d）。

use v8;

use zero_dom::{NodeId, NodeKind};

use super::gc::{encode_node_id, is_custom_connected, mark_custom_connected, unmark_custom_connected, with_dom};
use super::get_or_create_native_element;

/// `appendChild`/`insertBefore` 成功后调：对 `child` 子树的所有 custom 元素，按其（经新 parent 链）
/// 是否连入 document 判定连接态变化，真转则桥接 JS 派发 connectedCallback（连）/disconnectedCallback（断）。
///
/// `parent_id` = 新父节点（`this`）；`child_id` = 被插入的子树根。fragment 已在 mutation 中 flatten，
/// 故 `child_id` 是真实入树节点。子树 DFS 镜像 polyfill `_ceApplyConn` 的 pre-order 传播。
pub(super) fn notify_connect_after_insert(scope: &mut v8::PinScope, parent_id: NodeId, child_id: NodeId) {
    // 子树是否连入 document 取决于新 parent 链（child 经 parent 到 root）。
    let parent_connected = is_connected_to_document(parent_id);
    let mut to_connect: Vec<NodeId> = Vec::new();
    let mut to_disconnect: Vec<NodeId> = Vec::new();
    collect_custom_subtree(child_id, &mut |id, _tag| {
        let ffi = encode_node_id(id);
        let was = is_custom_connected(ffi);
        if parent_connected && !was {
            to_connect.push(id);
        } else if !parent_connected && was {
            to_disconnect.push(id);
        }
    });
    if to_connect.is_empty() && to_disconnect.is_empty() {
        return;
    }
    // 先标记（防派发期间回调再 mutation 导致状态错乱），再批量派发。
    for &id in to_connect.iter() {
        mark_custom_connected(encode_node_id(id));
    }
    for &id in to_disconnect.iter() {
        unmark_custom_connected(encode_node_id(id));
    }
    if !to_connect.is_empty() {
        dispatch_connect(scope, &to_connect, true);
    }
    if !to_disconnect.is_empty() {
        dispatch_connect(scope, &to_disconnect, false);
    }
}

/// `removeChild` 成功后调：被移除子树的所有 custom 元素断开 document（若先前已连）→ 派发 disconnectedCallback。
///
/// `child_id` 已从 parent 摘除（mutation 完成），故其 parent 链不再到 document root。
pub(super) fn notify_disconnect_after_remove(scope: &mut v8::PinScope, child_id: NodeId) {
    let mut to_disconnect: Vec<NodeId> = Vec::new();
    collect_custom_subtree(child_id, &mut |id, _tag| {
        if is_custom_connected(encode_node_id(id)) {
            to_disconnect.push(id);
        }
    });
    if to_disconnect.is_empty() {
        return;
    }
    for &id in to_disconnect.iter() {
        unmark_custom_connected(encode_node_id(id));
    }
    dispatch_connect(scope, &to_disconnect, false);
}

/// 桥接 JS 派发 connectedCallback/disconnectedCallback：把 custom 元素打成 (native 实例, tag) 数组
/// 传给 polyfill `__zw_native_ce_notify_connect(instances, connected, tags)`，polyfill 按 tag 查
/// `_ce_registry` + 调 ctor.prototype 回调（this=native 实例）。JS 函数缺失 / 派发失败 → 静默（不抛）。
fn dispatch_connect(scope: &mut v8::PinScope, ids: &[NodeId], connected: bool) {
    let context = scope.get_current_context();
    let global = context.global(scope);
    let Some(notify_key) = v8::String::new(scope, "__zw_native_ce_notify_connect") else {
        return;
    };
    let Some(notify_val) = global.get(scope, notify_key.into()) else {
        return; // polyfill 未注册（native_dom 关闭 shim 等）→ 无派发，静默。
    };
    let Ok(notify) = v8::Local::<v8::Function>::try_from(notify_val) else {
        return;
    };
    // 构造 (instances[], tags[]) 两并列数组——polyfill 按 tag 查 registry，按 index 配对实例。
    // 用 Array::new + set_index（v8::Array::new_with_elements 非 rusty_v8 公共 API）。
    let mut pairs: Vec<(v8::Local<v8::Object>, String)> = Vec::with_capacity(ids.len());
    for &id in ids {
        let Some(obj) = get_or_create_native_element(scope, id) else {
            continue;
        };
        let Some(tag) = element_tag(id) else {
            continue;
        };
        pairs.push((obj, tag.to_lowercase()));
    }
    if pairs.is_empty() {
        return;
    }
    let inst_arr = v8::Array::new(scope, pairs.len() as i32);
    let tag_arr = v8::Array::new(scope, pairs.len() as i32);
    for (i, (obj, tag)) in pairs.into_iter().enumerate() {
        let _ = inst_arr.set_index(scope, i as u32, obj.into());
        if let Some(t) = v8::String::new(scope, &tag) {
            let _ = tag_arr.set_index(scope, i as u32, t.into());
        }
    }
    let conn_v = v8::Boolean::new(scope, connected);
    let _ = notify.call(scope, global.into(), &[inst_arr.into(), conn_v.into(), tag_arr.into()]);
}

/// DFS 收集 `root` 子树（含 root 自身）的所有元素节点，对每个调 `f(id, tag)`。pre-order（根优先）。
/// 仅 Element 节点参与（custom 元素必为 Element）；Text/Comment 跳过。
fn collect_custom_subtree(root: NodeId, f: &mut impl FnMut(NodeId, String)) {
    with_dom(|d| {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = d.get(id) else {
                continue;
            };
            if let NodeKind::Element(e) = &node.kind {
                f(id, e.tag_name());
                // 子节点压栈（next_sibling 经 Document 查——NodeData 无该方法）。
                let mut c = node.first_child();
                while let Some(child) = c {
                    stack.push(child);
                    c = d.next_sibling(child);
                }
            }
        }
    });
}

/// 节点 parent 链是否到达 document root（祖先链顶 == `Document::root()`）= 连入 document。
/// detached 元素（createElement 未 appendChild，自身为子树根但非 document root）/ shadow root / fragment
/// → 顶节点 ≠ document root → false。需显式比 document root：detached 子树根 parent=None 与 document root
///（html 元素，parent=None）都无 parent，仅靠 parent=None 无法区分，须比 NodeId 是否 == `d.root()`。
fn is_connected_to_document(id: NodeId) -> bool {
    with_dom(|d| {
        let doc_root = d.root();
        let mut cur = Some(id);
        while let Some(c) = cur {
            if c == doc_root {
                return true; // 祖先链到达 document root
            }
            cur = d.parent_node(c);
        }
        false // parent 链耗尽未到 document root（detached 子树）
    })
    .unwrap_or(false)
}

/// 读元素 tag_name（仅 Element，非 Element → None）。
fn element_tag(id: NodeId) -> Option<String> {
    with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.tag_name()),
            _ => None,
        })
    })
    .flatten()
}
