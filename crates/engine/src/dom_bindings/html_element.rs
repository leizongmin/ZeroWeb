//! 原生 `HTMLElement` 构造器（P1b S5a，RFC §3.5.1）——注册为全局，使 `class MyEl extends HTMLElement`
//! 经 JS `class extends` 子类化成立（R3262 PoC 验证 internal field 经 super() 继承可行）。
//!
//! **S5a 范围**（kill-switch：随 native_dom 安装）：仅落地 `new HTMLElement()` / `class extends HTMLElement`
//! 的构造器——ctor 建一个**新 detached 元素**（`Document::create_element`，tag='div'，spec 基类抽象无具体 tag），
//! 存 NodeId 进 instance internal slot[0]（生产模式，非 PoC fixed 42），缓存 wrapper（身份映射 + GC weak）。
//! instance_template `nodeType` accessor 复用 [`node::native_node_type_getter`]（经 slot NodeId 读 DOM）。
//!
//! **不在 S5a**（后续切片）：connectedCallback/disconnectedCallback（S5c，复用 `_ceApplyConn`）、
//! upgrade/whenDefined parity（S5d）。S5a 的 `new HTMLElement()` 建 detached div 是基类构造的最小语义
//!（real browser `new HTMLElement()` 抛 Illegal constructor，但 headless 作基类供 extends，禁直接 new 留 S5b）。
//!
//! **S5b（R3265）**：ctor 优先读 [`gc::upgrade_node_id`]——`native_create_element_invoke` 命中 polyfill
//! `_ce_registry` 时设该槽（host 已建元素 NodeId），再调 registered ctor 的 `new_instance`（super() 走本
//! ctor 复用该 NodeId 填 slot[0]），使 `document.createElement('my-el')` 返 native custom 实例（instanceof
//! registered ctor + nodeType=1）。registry 反查经 polyfill 全局 `__zw_native_ce_lookup(tag)`。
//!
//! spec/设计：`docs/specs/p1b-v8-native-bindings-rfc.md` §3.5.1（S5a 切片）；TBD-1 验证见
//! `script-sandbox/src/dom_bindings.rs::poc_native_ctor_subclass`（R3262）。

use v8;

use super::gc::{cache_native_element, encode_node_id, upgrade_node_id, with_dom_mut};
use super::node::native_node_type_getter;

/// 构建并注册全局 `HTMLElement` 构造器（`install_dom_bindings` 调）。
///
/// FunctionTemplate 构造器：instance_template internal_field_count=1（实例 slot[0] 存 NodeId），
/// ctor 回调 [`native_html_element_ctor_invoke`] 建新 detached 元素 + 填 slot[0] + 缓存 wrapper。
/// instance_template `nodeType` accessor 复用 [`node::native_node_type_getter`]（holder=实例，经 slot
/// NodeId 读 DOM node_type；subclass 实例经 super() 继承 slot，accessor 透明可达——R3262 验证）。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    let tmpl = v8::FunctionTemplate::builder(native_html_element_ctor_invoke).build(scope);
    tmpl.instance_template(scope).set_internal_field_count(1);
    // nodeType accessor 挂 instance_template（holder=实例，有 slot）；prototype_template 的 holder=原型无 slot
    //（R3262 PoC 调试结论）。
    if let Some(k) = v8::String::new(scope, "nodeType") {
        tmpl.instance_template(scope)
            .set_accessor(k.into(), native_node_type_getter);
    }
    if let (Some(f), Some(key)) = (tmpl.get_function(scope), v8::String::new(scope, "HTMLElement")) {
        let _ = global.set(scope, key.into(), f.into());
    }
}

/// `new HTMLElement()` / `class X extends HTMLElement` 的 `super()` 构造器回调：建新 detached 元素
///（`Document::create_element('div')`，基类抽象无具体 tag）→ NodeId 进 `this` slot[0]（External ptr 值）
/// → 缓存 wrapper（身份映射，与既有 native 元素一致）。subclass 实例经 super() 调此 ctor，slot 继承（R3262）。
///
/// **S5b upgrade 分支**（R3265）：custom element upgrade 经 `native_create_element_invoke` 在调
/// registered ctor 前设 [`gc::upgrade_node_id`]（host 已建元素得 NodeId，tag=`my-el`），ctor `super()`
/// 调此 ctor 时优先用该 NodeId（**不建新 div**），使 custom 实例与 host 建的元素同 NodeId。
fn native_html_element_ctor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(obj) = this.to_object(scope) else {
        return;
    };
    // S5b upgrade：customElements upgrade 在途（`native_create_element_invoke` 设）→ 复用 host 建的
    // NodeId（同 tag='my-el' 元素），避免建新 detached div 与 host 元素脱节。无 upgrade 在途 → S5a
    // 直接 new 语义（建新 detached div）。
    let id = upgrade_node_id().or_else(|| with_dom_mut(|d| d.create_element("div")));
    let Some(id) = id else {
        return;
    };
    let ffi = encode_node_id(id);
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    // 缓存 wrapper（身份映射 + GC weak，复用既有 native 元素模式）——同 NodeId 后续 get_or_create 命中。
    cache_native_element(scope, ffi, obj);
}
