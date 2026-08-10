//! Document 原生对象——P1b R3159（escape-hatch 铺路）。
//!
//! 把文档级工厂（[`factories`] 子模块）绑定为 `document` 对象的**方法** + live Document 读为
//! **getter**（`documentElement`/`body`/`head`/`activeElement`）。`__zw_native_get_document()` 工厂
//! 返此对象（同对象身份缓存——gc.rs `DOCUMENT_OBJECT` 单例 weak）。
//!
//! **设计动机**：此前文档级操作仅经 `__zw_native_*` 全局工厂暴露（escape-hatch `__zw_native_get_body()`
//! 等）；本对象把它们收纳进 `document` 命名空间，escape-hatch 收敛路由 `document` 到本对象后，
//! `document.getElementById(...)` / `document.body` 等经原生直读 live Document（不经 shim 字符串桥）。
//!
//! 方法**直接复用** [`factories`] 的 `*invoke`（getElementById/querySelector/createElement 等）——
//! 这些工厂忽略 `this`、查全局 live Document，故绑到 `document` 对象方法与全局工厂语义一致（零重复）。
//! getter 因签名为 [`v8::PropertyCallbackArguments`]（工厂为 [`v8::FunctionCallbackArguments`]），独立写
//! 薄 getter 读 live Document。
//!
//! kill-switch `native_dom` 默认关 → 零回归（与既有 native 绑定同门控）。

use v8;

use zero_dom::NodeId;

use super::factories;
use super::gc::{
    active_element, cache_document, cached_document, document_template_local, set_document_template, with_dom,
    with_dom_mut,
};
use super::{get_or_create_native_element, local_value_to_string};

// ── accessor getter（live Document 读 → native 元素或 null）──────────────

/// `document.documentElement` getter（spec `dom-document-documentelement`）：文档根元素（Document
/// 首个 Element 子节点，HTML 为 <html>）→ native 元素或 `null`（无根元素）。
pub(super) fn native_document_element_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.child_nodes(d.root()).into_iter().find(|&c| d.node_type(c) == Some(1))).flatten();
    native_or_null(scope, id, &mut rv);
}

/// `document.body` getter（spec `dom-document-body`）：首个 `<body>` 元素 → native 元素或 `null`。
pub(super) fn native_body_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("body").into_iter().next()).flatten();
    native_or_null(scope, id, &mut rv);
}

/// `document.head` getter（spec `dom-document-head`）：首个 `<head>` 元素 → native 元素或 `null`。
pub(super) fn native_head_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("head").into_iter().next()).flatten();
    native_or_null(scope, id, &mut rv);
}

/// `document.activeElement` getter（spec `dom-document-activeelement`）：当前焦点元素（`element.focus()`
/// 设、`element.blur()` 清，R3148 `ACTIVE_ELEMENT` slot）→ native 元素或 `null`（无焦点）。
pub(super) fn native_active_element_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    native_or_null(scope, active_element(), &mut rv);
}

/// NodeId → native 元素（`get_or_create_native_element`）或 `null`（无 / stale）的共用 setter。
fn native_or_null(scope: &mut v8::PinScope, id: Option<NodeId>, rv: &mut v8::ReturnValue<v8::Value>) {
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            } else {
                rv.set(v8::null(scope).into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `document.title` getter（spec `dom-document-title`）：读首个 `<title>` 元素 textContent；
/// 无 `<title>` → 空串。经共享 [`factories::read_document_title`]（与 `__zw_native_get_document_title`
/// 工厂共用，DRY）。
pub(super) fn native_document_title_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let title = with_dom(factories::read_document_title).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &title) {
        rv.set(s.into());
    }
}

/// `document.title` setter：值 ToString 后经共享 [`factories::write_document_title`] 写回
///（与 `__zw_native_set_document_title` 工厂共用，DRY）。
pub(super) fn native_document_title_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    _args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| factories::write_document_title(d, &val));
}

// ── `document` ObjectTemplate 构建（方法复用 factories）+ 工厂 ──────────

/// 建 `document` ObjectTemplate 并缓存（gc.rs `DOCUMENT_TEMPLATE`）。幂等（已建则 no-op）。
///
/// 方法经 `v8::FunctionTemplate` 复用 [`factories`] 的 `*invoke`（getElementById/querySelector/
/// createElement 等——忽略 `this`、查全局 live Document）；getter 为本模块薄 getter（live Document 读）。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    if document_template_local(scope).is_some() {
        return; // 幂等：已建模板
    }
    let tmpl = v8::ObjectTemplate::new(scope);

    // getter（live Document 读 → native 元素或 null）。
    if let Some(k) = v8::String::new(scope, "documentElement") {
        tmpl.set_accessor(k.into(), native_document_element_getter);
    }
    if let Some(k) = v8::String::new(scope, "body") {
        tmpl.set_accessor(k.into(), native_body_getter);
    }
    if let Some(k) = v8::String::new(scope, "head") {
        tmpl.set_accessor(k.into(), native_head_getter);
    }
    if let Some(k) = v8::String::new(scope, "activeElement") {
        tmpl.set_accessor(k.into(), native_active_element_getter);
    }
    // R3160 `document.title` get/set（spec `dom-document-title`）：经共享 factories helper（与
    // __zw_native_*_document_title 工厂共用）。
    if let Some(k) = v8::String::new(scope, "title") {
        tmpl.set_accessor_with_setter(k.into(), native_document_title_getter, native_document_title_setter);
    }

    // 方法（复用 factories `*invoke`——忽略 `this`，查全局 live Document，与全局工厂语义一致）。
    set_method(scope, &tmpl, "getElementById", factories::native_element_factory_invoke);
    set_method(scope, &tmpl, "querySelector", factories::native_query_selector_invoke);
    set_method(
        scope,
        &tmpl,
        "querySelectorAll",
        factories::native_query_selector_all_invoke,
    );
    set_method(
        scope,
        &tmpl,
        "getElementsByTagName",
        factories::native_get_elements_by_tag_name_invoke,
    );
    set_method(
        scope,
        &tmpl,
        "getElementsByClassName",
        factories::native_get_elements_by_class_name_invoke,
    );
    set_method(scope, &tmpl, "createElement", factories::native_create_element_invoke);
    set_method(
        scope,
        &tmpl,
        "createTextNode",
        factories::native_create_text_node_invoke,
    );
    set_method(scope, &tmpl, "createComment", factories::native_create_comment_invoke);
    set_method(
        scope,
        &tmpl,
        "createDocumentFragment",
        factories::native_create_document_fragment_invoke,
    );
    // R3161 `createEvent(type)`（spec `dom-document-createevent`，legacy 事件创建）：复用 event 子模块
    // [`event::native_create_event_invoke`]（R3141，type→Event/CustomEvent/MouseEvent/KeyboardEvent 构造器
    // 查找 + `new Ctor("")` 返未初始化 event，待 initEvent 覆写）。补全 document 创建 API 三件套。
    set_method(scope, &tmpl, "createEvent", super::event::native_create_event_invoke);
    // R3162 `importNode(node, deep)` / `adoptNode(node)`（spec `dom-document-importnode` / `-adoptnode`）：
    // importNode 克隆节点（模板实例化高频，复用 clone_node）；adoptNode headless 单文档 = identity。
    set_method(scope, &tmpl, "importNode", factories::native_import_node_invoke);
    set_method(scope, &tmpl, "adoptNode", factories::native_adopt_node_invoke);

    set_document_template(scope, tmpl);
}

/// 注册方法：name → `FunctionTemplate`（callback 复用 factories `*invoke`）。模板 set。
fn set_method(
    scope: &mut v8::PinScope,
    tmpl: &v8::Local<v8::ObjectTemplate>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    if let Some(k) = v8::String::new(scope, name) {
        let ft = v8::FunctionTemplate::builder(callback).build(scope);
        tmpl.set(k.into(), ft.into());
    }
}

/// `__zw_native_get_document()` 工厂回调：返 `document` 对象（单例身份缓存——gc.rs `DOCUMENT_OBJECT`
/// weak；JS 丢引用即 GC，下次取重建）。模板未建（未安装）→ `undefined`（防御）。
pub(super) fn native_get_document_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    // 已缓存且存活 → 复用（spec `document === document` 身份）。
    if let Some(doc) = cached_document(scope) {
        rv.set(doc.into());
        return;
    }
    let Some(tmpl) = document_template_local(scope) else {
        return; // 模板未建 → undefined（防御）
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    cache_document(scope, obj);
    rv.set(obj.into());
}
