//! 全局工厂回调——拆自 mod.rs（RFC §3.2 子模块化 stage 3，本轮 R3118；createText/Comment/Fragment R3131；
//! documentElement/body/head R3136）。
//!
//! 10 个**全局**工厂（注册于 `ctx.global`，非 Element 模板成员）：
//! - `__zw_native_element_for_id(idStr)`：`get_element_by_id` → native 元素；
//! - `__zw_native_query_selector(sel)` / `__zw_native_query_selector_all(sel)`：文档根下
//!   全量选择器引擎匹配 → native 元素 / V8 Array（spec `dom-parentnode-queryselector(-all)`）；
//! - `__zw_native_create_element(tag)`：`Document::create_element` → native 元素（未挂载）；
//! - `__zw_native_create_text_node(text)` / `__zw_native_create_comment(text)` /
//!   `__zw_native_create_document_fragment()`（R3131）：造 Text(3)/Comment(8)/Fragment(11) 节点 →
//!   native 对象（未挂载），闭合 native 树构建集（createElement + createText/Comment/Fragment + appendChild）；
//! - `__zw_native_get_document_element()` / `__zw_native_get_body()` / `__zw_native_get_head()`
//!   （R3136）：文档级只读属性 → native 元素或 `null`（spec `dom-document-(documentelement|body|head)`）。
//!
//! 区别于 mod.rs `native_element_query_selector(-all)_invoke`（**元素子树作用域**，注册于 Element
//! 模板，root = `args.this()` 元素 + 排除自身）——本模块的是**文档级**（root = 文档根）。
//!
//! 可见性：10 个 invoke 为 `pub(super)`（mod.rs `install_dom_bindings` 注册经 `factories::` 调）。
//! 读 `super::string_arg` / `super::get_or_create_native_element`（mod.rs 私有——Rust 规则：私有项
//! 对后代模块可见）+ `super::gc::{with_dom, with_dom_mut}`。

use v8;

use zero_dom::NodeId;

use super::gc::{with_dom, with_dom_mut};
use super::{get_or_create_native_element, string_arg};

/// 工厂回调：`__zw_native_element_for_id(idStr)` → 解析 `get_element_by_id` →
/// NodeId → 创建/查找 native element 对象（NodeId↔对象身份映射 + stale 重建）。
///
/// 未找到 id → `null`。NodeId 编码进 internal slot[0]（`v8::External` ptr 值）。
pub(super) fn native_element_factory_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id_str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let Some(node_id) = with_dom(|d| d.get_element_by_id(&id_str)).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, node_id) {
        rv.set(obj.into());
    }
    // 无 Element 模板（未安装）→ undefined（防御，正常路径模板已 set）。
}

/// `__zw_native_query_selector(sel)`：spec `dom-parentnode-queryselector`——
/// 文档根下按**全量选择器引擎**（[`zero_dom::Document::query_selector`]，消费 tag/`*`/
/// `#id`/`.class`/`[attr]`+运算符/伪类/组合器）找首个匹配元素 → native 对象。
///
/// 无匹配 / 空 / 非法选择器 → `null`（`parse_selector_chain` 失败返 `None`，无 panic）。
pub(super) fn native_query_selector_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let Some(id) = with_dom(|d| d.query_selector(d.root(), sel.trim())).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_query_selector_all(sel)`：spec `dom-parentnode-queryselectorall`——
/// 文档根下按全量选择器引擎（[`zero_dom::Document::query_selector_all`]）收集全部匹配
/// 元素 → V8 `Array` of native 对象（文档序）。空 / 非法选择器 → 空 `Array`。
pub(super) fn native_query_selector_all_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(d.root(), sel.trim())).unwrap_or_default();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `__zw_native_create_element(tag)`：spec `dom-document-createelement`——
/// `Document::create_element(tag)` 造新 Element NodeId → native 对象（**未挂载**，需 appendChild）。
/// 空/缺省 tag → `div`（与 polyfill create_element 一致，spec 实际应抛，本切片 best-effort）。
pub(super) fn native_create_element_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let mut tag = string_arg(scope, &args, 0);
    if tag.trim().is_empty() {
        tag = "div".to_string();
    }
    // create（borrow_mut 释放后）→ 包 native 对象（get_or_create_native_element 内含 stale 校验）。
    let Some(id) = with_dom_mut(|d| d.create_element(tag.trim())) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_text_node(text)`：spec `dom-document-createtextnode`——
/// `Document::create_text_node` 造 Text NodeId → native 对象（nodeType=3，**未挂载**）。Text/Comment/
/// Fragment 共用 Element 包装模板（nodeType getter 经 NodeKind 读 DOM → 3/8/11，非 Element 亦正确）。
pub(super) fn native_create_text_node_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let text = string_arg(scope, &args, 0);
    let Some(id) = with_dom_mut(|d| d.create_text_node(&text)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_comment(text)`：spec `dom-document-createcomment`——
/// `Document::create_comment` 造 Comment NodeId → native 对象（nodeType=8，**未挂载**）。
pub(super) fn native_create_comment_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let text = string_arg(scope, &args, 0);
    let Some(id) = with_dom_mut(|d| d.create_comment(&text)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_document_fragment()`：spec `dom-document-createdocumentfragment`——
/// `Document::create_document_fragment` 造 DocumentFragment NodeId → native 对象（nodeType=11，**未挂载**）。
/// appendChild/insertBefore fragment 经 Document 自动 flatten 子节点（spec：insert fragment 等价插其子）。
pub(super) fn native_create_document_fragment_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let Some(id) = with_dom_mut(|d| d.create_document_fragment()) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

// ── R3136 文档级只读属性工厂（documentElement / body / head）──

/// `__zw_native_get_document_element()`：spec `dom-document-documentelement`——文档根元素
///（Document 节点的首个 Element 子节点；HTML 文档为 <html>）。返 native 元素或 `null`（无根元素）。
pub(super) fn native_get_document_element_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    // spec 定义：root element = Document 节点（`d.root()`）的首个 Element 子节点。
    let id = with_dom(|d| d.child_nodes(d.root()).into_iter().find(|&c| d.node_type(c) == Some(1))).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `__zw_native_get_body()`：spec `dom-document-body`——文档的 `<body>` 元素（首个 body 元素）。
/// 返 native 元素或 `null`（无 body）。
pub(super) fn native_get_body_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("body").into_iter().next()).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `__zw_native_get_head()`：spec `dom-document-head`——文档的 `<head>` 元素（首个 head 元素）。
/// 返 native 元素或 `null`（无 head）。
pub(super) fn native_get_head_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("head").into_iter().next()).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}
