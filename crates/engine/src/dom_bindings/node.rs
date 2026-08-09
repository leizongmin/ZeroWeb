//! Node 基类原生绑定——拆自 mod.rs（RFC §3.2 子模块化 stage 4a，本轮 R3119）。
//!
//! DOM Node 基类面（spec DOM Standard `Node`）：nodeType / nodeName / nodeValue(+setter) /
//! textContent(+setter) / childNodes / parentNode / firstChild / lastChild / nextSibling /
//! previousSibling / hasChildNodes / appendChild / insertBefore / removeChild / replaceChild /
//! cloneNode / contains。Element 特有面（tagName / id / className / getAttribute* / children /
//! querySelector(-All) / innerHTML / outerHTML）仍在 mod.rs（待 element.rs，下一切片 stage 4b）。
//!
//! 可见性：注册于 Element 模板的 getter/invoke 为 `pub(super)`（mod.rs `install_dom_bindings`
//! 注册经 `node::` 调）；`node_name` / `node_value` / `node_relation_getter` / `set_native_element`
//! 为本模块私有助手。读 `super::read_node_id` / `super::node_id_from_value` /
//! `super::get_or_create_native_element` / `super::local_value_to_string`（mod.rs 私有共享——
//! Rust 规则：私有项对后代模块可见）+ `super::gc::{with_dom, with_dom_mut}`。

use v8;

use zero_dom::{Document, DomError, NodeId, NodeKind};

use super::gc::{with_dom, with_dom_mut};
use super::{get_or_create_native_element, local_value_to_string, node_id_from_value, read_node_id};

// ── accessor getter（ZST fn；状态经 gc.rs 线程局部）─────────────────

/// `nodeType` getter：读 internal slot[0] NodeId → `Document::node_type` → `v8::Integer`。
///
/// stale（节点移除）或无 NodeId → 留 undefined（spec detached 行为）。
pub(super) fn native_node_type_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    // with_dom 返 Option<Option<u8>>（外层=无 DOM 源，内层=节点无 node_type）。
    let nt: Option<u8> = with_dom(|d| d.node_type(id)).flatten();
    if let Some(nt) = nt {
        rv.set(v8::Integer::new(scope, i32::from(nt)).into());
    }
}

/// `nodeName` getter：spec `dom-node-nodename`——Element=tagName（HTML 大写），
/// 其他节点类型为固定串（#text/#comment/#document/#document-fragment）。
///
/// native 对象经 `get_element_by_id` 创建，均为 Element，故主路径 nodeName==tagName；
/// 非 Element 分支为 spec 合规防御（PI/DocumentType 的 target/name 近似，元素主导）。
pub(super) fn native_node_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let nm: Option<String> = with_dom(|d| node_name(d, id)).flatten();
    let Some(nm) = nm else {
        return;
    };
    if let Some(s) = v8::String::new(scope, &nm) {
        rv.set(s.into());
    }
}

/// Rust 侧 nodeName 计算（spec `dom-node-nodename`）。
fn node_name(doc: &Document, id: NodeId) -> Option<String> {
    let n = doc.get(id)?;
    Some(match &n.kind {
        NodeKind::Element(e) => e.local_name().to_ascii_uppercase(),
        NodeKind::Text(_) => "#text".into(),
        NodeKind::Comment(_) => "#comment".into(),
        NodeKind::Document(_) => "#document".into(),
        NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => "#document-fragment".into(),
        // PI 的 nodeName=target、DocumentType=name；native 对象均为 Element，此处近似防御。
        NodeKind::ProcessingInstruction(_) => "#processing-instruction".into(),
        NodeKind::DocumentType(_) => "#document-type".into(),
    })
}

/// `textContent` getter（spec `dom-node-textcontent`）：子树文本拼接（`Document::text_content`，
/// 递归收集后代 Text 节点 data）。空子树 → `""`。
pub(super) fn native_text_content_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let text = with_dom(|d| d.text_content(id)).flatten().unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &text) {
        rv.set(s.into());
    }
}

/// `textContent` setter（spec `dom-node-textcontent`）：值 ToString 后**清空全部子节点**，
/// 非空则追加单 Text 节点（`create_text_node` + `append_child`）。空串 → 仅清空（不添空 Text 节点）。
pub(super) fn native_text_content_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| {
        // 移除全部子节点（先收集 NodeId 避免边遍历边改）。
        let children = d.child_nodes(id);
        for c in children {
            let _ = d.remove_child(id, c);
        }
        // 非空 → 追加文本节点。
        if !val.is_empty() {
            let text_id = d.create_text_node(&val);
            let _ = d.append_child(id, text_id);
        }
    });
}

/// `childNodes` getter（spec `dom-node-childnodes`）：**全部子节点**（含文本/注释）→ V8 Array of
/// native 对象（文档序）。区别于 [`super::native_children_getter`]（仅元素）——文本/注释节点经同一模板
/// 包后 nodeType(3/8)/nodeName/textContent 正确（node-type-aware getter）。
pub(super) fn native_child_nodes_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let child_ids: Vec<NodeId> = with_dom(|d| d.child_nodes(id)).unwrap_or_default();
    let arr = v8::Array::new(scope, child_ids.len() as i32);
    for (i, cid) in child_ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, cid) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `nodeValue` getter（spec `dom-node-nodevalue`）：Text/Comment/PI=data；其余（Element/Document/
/// DocumentFragment/ShadowRoot/DocumentType）=null。区别于 `textContent`（Element 返子树文本）。
pub(super) fn native_node_value_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let Some(val) = with_dom(|d| node_value(d, id)).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// Rust 侧 nodeValue 计算（spec `dom-node-nodevalue`）。Text/Comment/PI=data；其余 None（→null）。
fn node_value(doc: &Document, id: NodeId) -> Option<String> {
    let n = doc.get(id)?;
    Some(match &n.kind {
        NodeKind::Text(t) => t.content.clone(),
        NodeKind::Comment(c) => c.content.clone(),
        NodeKind::ProcessingInstruction(p) => p.data.clone(),
        // Element/Document/DocumentFragment/ShadowRoot/DocumentType → null（spec）。
        _ => return None,
    })
}

// ── 树 mutation 方法（Node 上：appendChild / insertBefore / removeChild / replaceChild）──

/// `appendChild(child)`：spec `dom-node-appendchild`——`args.this()`=parent，参=child native
/// 若 `node` 为 DocumentFragment，把其子节点逐个移到 `parent`（`ref_node` None→append、Some→insert-before），
/// fragment 清空（spec：insert fragment 等价插其子并清空）；否则直接 append/insert `node`。复用 polyfill
/// `move_fragment_children` 思路（快照子列表，逐个 append/insert——Document 移动操作自动从 fragment detach）。
/// 供 [`native_append_child_invoke`] / [`native_insert_before_invoke`]（R3132）+ [`insert_variadic`] 现代插入族
/// prepend/append/before/after/replaceWith（R3144）共用。
fn insert_with_fragment_flatten(
    doc: &mut Document,
    parent: NodeId,
    node: NodeId,
    ref_node: Option<NodeId>,
) -> Result<(), DomError> {
    let is_fragment = doc
        .get(node)
        .is_some_and(|n| matches!(n.kind, NodeKind::DocumentFragment));
    if is_fragment {
        // 快照 fragment 子列表（移动过程中子从 fragment 移除，故先收集）。
        let kids: Vec<NodeId> = doc.get(node).map(|n| n.children.clone()).unwrap_or_default();
        for child in kids {
            match ref_node {
                Some(r) => doc.insert_before(parent, child, r)?,
                None => doc.append_child(parent, child)?,
            }
        }
        Ok(())
    } else {
        match ref_node {
            Some(r) => doc.insert_before(parent, node, r),
            None => doc.append_child(parent, node),
        }
    }
}

/// 对象；`Document::append_child` 移动（含 re-parent、cycle 检测）。成功返 child 对象（spec），
/// Err（cycle/not-found）→ best-effort 留 undefined（不抛，限制记录）。
/// R3132：DocumentFragment 参 → flatten（子节点移到 parent、fragment 清空，spec）。
pub(super) fn native_append_child_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    let ok = with_dom_mut(|d| insert_with_fragment_flatten(d, parent, child, None))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        set_native_element(scope, child, &mut rv);
    }
}

/// `insertBefore(newChild, refChild)`：spec `dom-node-insertbefore`——parent=this，参 0=newChild、
/// 参 1=refChild native 对象；`Document::insert_before`。`refChild` 缺省/null → 末尾追加（spec）。
/// 成功返 newChild 对象；Err → best-effort 留 undefined。
/// R3132：DocumentFragment 参 → flatten（子节点插到 refNode 前、fragment 清空，spec）。
pub(super) fn native_insert_before_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(new_child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    // refChild null/缺省 → 末尾追加（spec：ref 为 null 时同 appendChild）。
    let ref_child = node_id_from_value(scope, args.get(1));
    let ok = with_dom_mut(|d| insert_with_fragment_flatten(d, parent, new_child, ref_child))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        set_native_element(scope, new_child, &mut rv);
    }
}

/// `removeChild(child)`：spec `dom-node-removechild`——parent=this，参=child native 对象；
/// `Document::remove_child`。成功返被移除的 child 对象（spec）；Err → best-effort 留 undefined。
pub(super) fn native_remove_child_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    let ok = with_dom_mut(|d| d.remove_child(parent, child))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        set_native_element(scope, child, &mut rv);
    }
}

/// `replaceChild(newChild, oldChild)`：spec `dom-node-replace-child`——parent=this，参为两个 native
/// 元素对象（读 internal slot NodeId）；`Document::replace_child`。成功返 oldChild（spec）。
pub(super) fn native_replace_child_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(new_child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    let Some(old_child) = node_id_from_value(scope, args.get(1)) else {
        return;
    };
    let ok = with_dom_mut(|d| d.replace_child(parent, new_child, old_child))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        // spec：返被替换的 oldChild。
        set_native_element(scope, old_child, &mut rv);
    }
}

/// `element.remove()`：spec `dom-childnode-remove`——**自移除**（从父节点摘除自身）。无父（detached）
/// → no-op（spec）。区别于 `removeChild`（parent.removeChild(child)）：remove 在子节点上调用，找自身
/// parent 后 remove_child(self)。ChildNode mixin（Element/Text/Comment 均可，注册于 Element 模板）。
pub(super) fn native_element_remove_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    with_dom_mut(|d| {
        // 找自身 parent，有则 remove_child(parent, self)；无 parent（detached）no-op（spec）。
        if let Some(parent) = d.parent_node(id) {
            let _ = d.remove_child(parent, id);
        }
    });
}

// ── R3143 现代 ChildNode/ParentNode 插入族（prepend/append/before/after/replaceWith）──

/// 插入位置（spec prepend/append/before/after/replaceWith）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum InsertPos {
    Prepend,
    Append,
    Before,
    After,
    ReplaceWith,
}

/// variadic 插入项：既有节点（NodeId）或文本（字符串 → 后续 create_text_node）。
enum InsertItem {
    Node(NodeId),
    Text(String),
}

/// 共享 variadic 插入助手（prepend/append/before/after/replaceWith 用）。spec：
/// - `append(...items)`：插到 `self`（作 parent）末尾；
/// - `prepend(...items)`：插到 `self`（作 parent）首子前；
/// - `before(...items)` / `after(...items)`：插到 `self` 的父中 self 前/后；
/// - `replaceWith(...items)`：在 self 位置插 items 后移除 self。
///
/// items 含节点（native 元素读 NodeId）与字符串（→ 文本节点）；非节点参 ToString。**两遍**：① 经 scope 收集
/// items（Node/Text，避 scope 跨 DOM borrow）；② `with_dom_mut` 按 position 算 (parent, ref_node, remove_self)
/// 逐 item 插入 ref_node 前（ref 固定 = 原首子/原 next sibling/self，故 arg 序 = DOM 序）。before/after/
/// replaceWith 无 parent（detached）→ no-op。
///
/// R3144：DocumentFragment 参 → flatten（子节点展开进 parent、fragment 清空，spec），与 R3132
/// appendChild/insertBefore 同语义——经 [`insert_with_fragment_flatten`] 统一处理（非 fragment 直接 insert/append，
/// fragment 快照子逐个插；ref 固定保证子内序 = fragment 内序）。
fn insert_variadic(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, self_id: NodeId, pos: InsertPos) {
    // Pass 1：收集 items（Node 或 Text）——经 scope 读 NodeId / ToString，不持 DOM borrow。
    let n = args.length();
    let mut items: Vec<InsertItem> = Vec::with_capacity(n.max(0) as usize);
    for i in 0..n {
        let arg = args.get(i);
        match node_id_from_value(scope, arg) {
            Some(id) => items.push(InsertItem::Node(id)),
            None => items.push(InsertItem::Text(local_value_to_string(scope, arg))),
        }
    }
    // Pass 2：DOM mutation——算 (parent, ref_node, remove_self) 后逐 item 插入。
    with_dom_mut(|d| {
        let (parent, ref_node, remove_self) = match pos {
            InsertPos::Append => (self_id, None, false),
            InsertPos::Prepend => (self_id, d.first_child(self_id), false),
            InsertPos::Before | InsertPos::ReplaceWith => {
                let parent = d.parent_node(self_id)?; // 无 parent → no-op
                (parent, Some(self_id), pos == InsertPos::ReplaceWith)
            }
            InsertPos::After => {
                let parent = d.parent_node(self_id)?;
                (parent, d.next_sibling(self_id), false)
            }
        };
        for item in &items {
            let node_id = match item {
                InsertItem::Node(id) => *id,
                InsertItem::Text(s) => d.create_text_node(s),
            };
            // R3144：DocumentFragment 参 → flatten（子节点移到 parent、fragment 清空，spec）；
            // 非 fragment 直接 insert_before/append_child（与 flatten 内非 fragment 分支等价）。
            let _ = insert_with_fragment_flatten(d, parent, node_id, ref_node);
        }
        if remove_self {
            let _ = d.remove_child(parent, self_id);
        }
        Some(())
    });
}

/// `element.prepend(...items)`：spec `dom-parentnode-prepend`——items 插到 self（作 parent）首子前。
pub(super) fn native_element_prepend_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    insert_variadic(scope, args, id, InsertPos::Prepend);
}

/// `element.append(...items)`：spec `dom-parentnode-append`——items 插到 self（作 parent）末尾。
pub(super) fn native_element_append_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    insert_variadic(scope, args, id, InsertPos::Append);
}

/// `element.before(...items)`：spec `dom-childnode-before`——items 插到 self 父中 self 前。detached → no-op。
pub(super) fn native_element_before_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    insert_variadic(scope, args, id, InsertPos::Before);
}

/// `element.after(...items)`：spec `dom-childnode-after`——items 插到 self 父中 self 后。detached → no-op。
pub(super) fn native_element_after_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    insert_variadic(scope, args, id, InsertPos::After);
}

/// `element.replaceWith(...items)`：spec `dom-childnode-replacewith`——在 self 位置插 items 后移除 self。detached → no-op。
pub(super) fn native_element_replace_with_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    insert_variadic(scope, args, id, InsertPos::ReplaceWith);
}

/// `nodeValue` setter（spec `dom-node-nodevalue`）：值 ToString 后，Text/Comment/PI 改 content/data
///（`Document::set_node_value`），其余 no-op（spec）。写入经 R3108 `sync_render_after_native_dom` 重渲染。
pub(super) fn native_node_value_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let this = args.holder();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let s = value
        .to_string(scope)
        .map(|v| v.to_rust_string_lossy(scope))
        .unwrap_or_default();
    with_dom_mut(|d| d.set_node_value(id, &s));
}

/// mutation 方法成功尾共用：把 NodeId 包成 native 对象 set 到 `rv`（appendChild/insertBefore/
/// removeChild 成功返被操作节点对象）。抽离以避 `if ok { if let ... }` 嵌套（MSRV 1.85 无 let-chains）。
fn set_native_element(scope: &mut v8::PinScope, id: NodeId, rv: &mut v8::ReturnValue<v8::Value>) {
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

// ── cloneNode / contains（Node 上）──

/// `cloneNode(deep)`（spec `dom-node-clonenode`）：复用 `Document::clone_node`——克隆元素 + 属性，
/// deep=true 递归克隆子树；返新 native 元素（**未挂载**，需 appendChild；克隆节点 id 不注册 id_map，
/// 与源共享 id 值，调用方挂载后应改设唯一 id）。
pub(super) fn native_clone_node_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    // deep 缺省 → false（spec：cloneNode() 浅克隆）。
    let deep = args.get(0).boolean_value(scope);
    let Some(new_id) = with_dom_mut(|d| d.clone_node(id, deep)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, new_id) {
        rv.set(obj.into());
    }
}

/// `contains(node)`（spec `dom-node-contains`）：node 是否为本元素自身或后代（walk parent 链）。
/// null/undefined/非 native 参 → false（spec：contains(null)===false）。
pub(super) fn native_contains_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(self_id) = read_node_id(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let Some(target_id) = node_id_from_value(scope, args.get(0)) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    // 从 target 沿 parent 链上溯，命中 self_id 则 true（含自身）。
    let contains = with_dom(|d| {
        let mut cur = Some(target_id);
        while let Some(c) = cur {
            if c == self_id {
                return true;
            }
            cur = d.parent_node(c);
        }
        false
    })
    .unwrap_or(false);
    rv.set(v8::Boolean::new(scope, contains).into());
}

// ── 节点导航 getter（parentNode / firstChild / lastChild / nextSibling / previousSibling / hasChildNodes）──

/// 节点导航 getter 共用尾：读 holder NodeId → 经 `rel` 取相关 NodeId → 包 native 节点对象（或 null）。
/// 5 个 getter（parentNode/firstChild/lastChild/nextSibling/previousSibling）均为 ZST fn 项（v8 accessor
/// 须 fn 项不能 cast fn 指针），故逐成员薄壳调本共用尾（镜像 install_dom_bindings 既有 getter 模式）。
fn node_relation_getter(
    scope: &mut v8::PinScope,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
    rel: impl Fn(&Document, NodeId) -> Option<NodeId>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    match with_dom(|d| rel(d, id)).flatten() {
        Some(rid) => {
            if let Some(obj) = get_or_create_native_element(scope, rid) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

pub(super) fn native_parent_node_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    rv: v8::ReturnValue<v8::Value>,
) {
    node_relation_getter(scope, args, rv, |d, id| d.parent_node(id));
}

pub(super) fn native_first_child_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    rv: v8::ReturnValue<v8::Value>,
) {
    node_relation_getter(scope, args, rv, |d, id| d.first_child(id));
}

pub(super) fn native_last_child_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    rv: v8::ReturnValue<v8::Value>,
) {
    node_relation_getter(scope, args, rv, |d, id| d.last_child(id));
}

pub(super) fn native_next_sibling_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    rv: v8::ReturnValue<v8::Value>,
) {
    node_relation_getter(scope, args, rv, |d, id| d.next_sibling(id));
}

pub(super) fn native_previous_sibling_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    rv: v8::ReturnValue<v8::Value>,
) {
    node_relation_getter(scope, args, rv, |d, id| d.previous_sibling(id));
}

/// `hasChildNodes()`（spec `dom-node-has-child-nodes`）：this 有子节点 → true。
pub(super) fn native_has_child_nodes_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let has = match read_node_id(scope, &this) {
        Some(id) => with_dom(|d| d.has_child_nodes(id)).unwrap_or(false),
        None => false,
    };
    rv.set(v8::Boolean::new(scope, has).into());
}
