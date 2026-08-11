//! 原生 `HTMLElement` 构造器（P1b S5a，RFC §3.5.1）——注册为全局，使 `class MyEl extends HTMLElement`
//! 经 JS `class extends` 子类化成立（R3262 PoC 验证 internal field 经 super() 继承可行）。
//!
//! **S5a 范围**（kill-switch：随 native_dom 安装）：仅落地 `new HTMLElement()` / `class extends HTMLElement`
//! 的构造器——ctor 建一个**新 detached 元素**（`Document::create_element`，tag='div'，spec 基类抽象无具体 tag），
//! 存 NodeId 进 instance internal slot[0]（生产模式，非 PoC fixed 42），缓存 wrapper（身份映射 + GC weak）。
//! instance_template `nodeType` accessor 复用 [`node::native_node_type_getter`]（经 slot NodeId 读 DOM）。
//!
//! **不在 S5a**（后续切片）：customElements registry 集成（S5b createElement('my-el') upgrade 经
//! `Reflect.construct(registeredCtor)`）、connectedCallback/disconnectedCallback（S5c，复用 `_ceApplyConn`）、
//! upgrade/whenDefined parity（S5d）。S5a 的 `new HTMLElement()` 建 detached div 是基类构造的最小语义
//!（real browser `new HTMLElement()` 抛 Illegal constructor，但 headless 作基类供 extends，禁直接 new 留 S5b）。
//!
//! spec/设计：`docs/specs/p1b-v8-native-bindings-rfc.md` §3.5.1（S5a 切片）；TBD-1 验证见
//! `script-sandbox/src/dom_bindings.rs::poc_native_ctor_subclass`（R3262）。

use v8;

use super::gc::{cache_native_element, encode_node_id, with_dom_mut};
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
fn native_html_element_ctor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(obj) = this.to_object(scope) else {
        return;
    };
    // 建新 detached 元素（tag='div'，spec 基类抽象）。S5a 直接 new 的语义；customElements upgrade
    //（createElement('my-el') 注入已有 NodeId）为 S5b。
    let Some(id) = with_dom_mut(|d| d.create_element("div")) else {
        return;
    };
    let ffi = encode_node_id(id);
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    // 缓存 wrapper（身份映射 + GC weak，复用既有 native 元素模式）——同 NodeId 后续 get_or_create 命中。
    cache_native_element(scope, ffi, obj);
}
