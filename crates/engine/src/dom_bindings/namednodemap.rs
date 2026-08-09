//! R3112 NamedNodeMap（`element.attributes` 集合）原生绑定——拆自 mod.rs（RFC §3.2 子模块化，本轮 R3116）。
//!
//! 内聚属性集合面：`attributes` getter（Element 模板注册）+ NamedNodeMap 模板（length/item/
//! getNamedItem/setNamedItem/removeNamedItem）+ Attr-like plain 对象构造。internal slot[0] =
//! owner element NodeId（方法经 `super::read_node_id` 取 owner）。Attr 返 plain `{name,value}`
//!（documented 限制：完整 Attr 节点 / nodeType=2 后续切片）。
//!
//! 可见性：`native_attributes_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs 的
//! Element 模板注册 + install_dom_bindings 调）；余私有（本模块 build 内部用）。读 `super::read_node_id`
//! / `super::string_arg`（mod.rs 私有——Rust 规则：私有项对后代模块可见）。

use v8;

use super::gc::{
    cache_namednodemap, cached_namednodemap, encode_node_id, namednodemap_template_local, set_namednodemap_template,
    with_dom, with_dom_mut,
};
use super::{read_node_id, string_arg};

/// 构造 Attr-like plain 对象 `{name, value}`（documented 限制：非完整 Attr 节点 / nodeType=2，后续切片）。
fn make_attr_object<'s>(scope: &mut v8::PinScope<'s, '_>, name: &str, value: &str) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);
    if let (Some(k), Some(v)) = (v8::String::new(scope, "name"), v8::String::new(scope, name)) {
        let _ = obj.set(scope, k.into(), v.into());
    }
    if let (Some(k), Some(v)) = (v8::String::new(scope, "value"), v8::String::new(scope, value)) {
        let _ = obj.set(scope, k.into(), v.into());
    }
    obj
}

/// 从对象读字符串属性（setNamedItem 读 attr.name/attr.value 用）。缺省 → 空串。
fn read_str_prop(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, key: &str) -> String {
    v8::String::new(scope, key)
        .and_then(|k| obj.get(scope, k.into()))
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default()
}

/// `attributes` getter（spec `dom-element-attributes`）：返 owner 元素的 NamedNodeMap（缓存保身份）。
pub(super) fn native_attributes_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let ffi = encode_node_id(id);
    // 缓存命中 → 同对象（spec `el.attributes === el.attributes`）。
    if let Some(cached) = cached_namednodemap(scope, ffi) {
        rv.set(cached.into());
        return;
    }
    // 实例化 NamedNodeMap 模板 + internal slot[0] = owner element NodeId。
    let Some(tmpl) = namednodemap_template_local(scope) else {
        return;
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_namednodemap(scope, ffi, obj);
    rv.set(obj.into());
}

/// NamedNodeMap `length` getter（spec `dom-namednodemap-length`）：owner 元素属性数。
fn native_nnm_length_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        rv.set(v8::Integer::new(scope, 0).into());
        return;
    };
    let len = with_dom(|d| d.attribute_names(id).len()).unwrap_or(0);
    rv.set(v8::Integer::new(scope, len as i32).into());
}

/// NamedNodeMap `item(index)`（spec `dom-namednodemap-item`）：index 处属性 → Attr-like 或 null。
fn native_nnm_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let idx = args.get(0).integer_value(scope).unwrap_or(-1);
    let names = with_dom(|d| d.attribute_names(id)).unwrap_or_default();
    match names.get(idx as usize) {
        Some(name) => {
            let value = with_dom(|d| d.get_attribute(id, name)).flatten().unwrap_or_default();
            rv.set(make_attr_object(scope, name, &value).into());
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// NamedNodeMap `getNamedItem(name)`（spec `dom-namednodemap-get-named-item`）：name 属性 → Attr-like 或 null。
fn native_nnm_get_named_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let name = string_arg(scope, &args, 0);
    match with_dom(|d| d.get_attribute(id, &name)).flatten() {
        Some(v) => rv.set(make_attr_object(scope, &name, &v).into()),
        None => rv.set(v8::null(scope).into()),
    }
}

/// NamedNodeMap `setNamedItem(attr)`（spec `dom-namednodemap-set-named-item`）：读 attr.name/attr.value →
/// set_attribute（best-effort；返传入 attr，spec 返被替换 Attr）。
fn native_nnm_set_named_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let attr_val = args.get(0);
    let Ok(attr_obj) = v8::Local::<v8::Object>::try_from(attr_val) else {
        return;
    };
    let name = read_str_prop(scope, attr_obj, "name");
    if name.is_empty() {
        return;
    }
    let value = read_str_prop(scope, attr_obj, "value");
    with_dom_mut(|d| d.set_attribute(id, &name, &value));
    rv.set(attr_val);
}

/// NamedNodeMap `removeNamedItem(name)`（spec `dom-namednodemap-remove-named-item`）：remove_attribute
///（best-effort；返 null，spec 返被移除 Attr）。
fn native_nnm_remove_named_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let name = string_arg(scope, &args, 0);
    with_dom_mut(|d| d.remove_attribute(id, &name));
    rv.set(v8::null(scope).into());
}

/// 建 NamedNodeMap ObjectTemplate（length getter + item/getNamedItem/setNamedItem/removeNamedItem）
/// 并缓存（`set_namednodemap_template`）。`install_dom_bindings` 调（Element 模板建之后）。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    let nnm = v8::ObjectTemplate::new(scope);
    nnm.set_internal_field_count(1);
    if let Some(k) = v8::String::new(scope, "length") {
        nnm.set_accessor(k.into(), native_nnm_length_getter);
    }
    let nnm_item = v8::FunctionTemplate::builder(native_nnm_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "item") {
        nnm.set(k.into(), nnm_item.into());
    }
    let nnm_get = v8::FunctionTemplate::builder(native_nnm_get_named_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getNamedItem") {
        nnm.set(k.into(), nnm_get.into());
    }
    let nnm_set = v8::FunctionTemplate::builder(native_nnm_set_named_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "setNamedItem") {
        nnm.set(k.into(), nnm_set.into());
    }
    let nnm_rm = v8::FunctionTemplate::builder(native_nnm_remove_named_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeNamedItem") {
        nnm.set(k.into(), nnm_rm.into());
    }
    set_namednodemap_template(scope, nnm);
}
