//! R3112 NamedNodeMap（`element.attributes` 集合）+ R3122 Attr 节点原生绑定——拆自 mod.rs
//!（RFC §3.2 子模块化 R3116；R3122 补完整 Attr 节点）。
//!
//! 内聚属性集合面：`attributes` getter（Element 模板注册）+ NamedNodeMap 模板（length/item/
//! getNamedItem/setNamedItem/removeNamedItem）+ Attr 节点模板（R3122：nodeType=2 / name / nodeName /
//! value(+setter) / nodeValue / textContent / ownerElement）。NamedNodeMap internal slot[0] =
//! owner element NodeId；Attr internal slot[0]=owner ffi、slot[1]=attr 名 arena idx（getter 经 gc.rs
//! ATTR_NAMES arena 复原名）。Attr 身份缓存 ATTR_OBJECTS（同 (owner,name) 返同对象，spec identity）。
//!
//! 可见性：`native_attributes_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs 的
//! Element 模板注册 + install_dom_bindings 调）；余私有（本模块 build 内部用）。读 `super::read_node_id`
//! / `super::string_arg` / `super::get_or_create_native_element` / `super::local_value_to_string`
//!（mod.rs 私有——Rust 规则：私有项对后代模块可见）。

use v8;

use zero_dom::NodeId;

use super::gc::{
    add_attr_name, attr_name, attr_template_local, cache_attr, cache_namednodemap, cached_attr, cached_namednodemap,
    decode_node_id, encode_node_id, namednodemap_template_local, set_attr_template, set_namednodemap_template,
    with_dom, with_dom_mut,
};
use super::{get_or_create_native_element, local_value_to_string, read_node_id, string_arg};

/// 从 Attr 对象 internal slot 读 `(owner NodeId, attr name)`。slot[0]=owner ffi（External ptr 值）、
/// slot[1]=attr 名 arena idx+1（External ptr 值，+1 避 0 指针）。非 Attr 对象 / stale → `None`。
fn attr_owner_and_name(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<(NodeId, String)> {
    let owner_ffi = obj.get_internal_field(scope, 0)?.cast::<v8::External>().value() as usize as u64;
    let owner_id = decode_node_id(owner_ffi);
    let idx_plus1 = obj.get_internal_field(scope, 1)?.cast::<v8::External>().value() as usize;
    let name = attr_name((idx_plus1 - 1) as u32)?;
    Some((owner_id, name))
}

/// 构造 Attr 节点对象（spec `dom-attr`：nodeType=2 / name / value / ownerElement）。internal slot[0]=owner ffi、
/// slot[1]=attr 名 arena idx+1（getter 经 gc.rs `ATTR_NAMES` 复原名）。身份缓存 `ATTR_OBJECTS`：
/// 同 (owner, name) 返同对象（spec identity）。value 经 getter **live** 读 owner 元素当前属性（不缓存值）。
fn make_attr_node<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner_id: NodeId,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let owner_ffi = encode_node_id(owner_id);
    // 身份缓存命中 → 同对象。
    if let Some(cached) = cached_attr(scope, owner_ffi, name) {
        return Some(cached);
    }
    let tmpl = attr_template_local(scope)?;
    let obj = tmpl.new_instance(scope)?;
    let owner_ptr = owner_ffi as usize as *mut std::ffi::c_void;
    let _ = obj.set_internal_field(0, v8::External::new(scope, owner_ptr).into());
    // idx+1 避 0 指针（slotmap ffi 非 0 约定同此）。
    let idx_ptr = (add_attr_name(name.to_string()) as usize + 1) as *mut std::ffi::c_void;
    let _ = obj.set_internal_field(1, v8::External::new(scope, idx_ptr).into());
    cache_attr(scope, owner_ffi, name, obj);
    Some(obj)
}

/// Attr `nodeType` getter（spec `dom-node-nodetype`）：Attr = 2。非 Attr 对象 → undefined。
fn native_attr_node_type_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    if attr_owner_and_name(scope, &holder).is_some() {
        rv.set(v8::Integer::new(scope, 2).into());
    }
}

/// Attr `name` / `nodeName` getter（spec `dom-attr-name` / `dom-node-nodename`）：属性限定名。
fn native_attr_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some((_owner, name)) = attr_owner_and_name(scope, &holder) else {
        return;
    };
    if let Some(s) = v8::String::new(scope, &name) {
        rv.set(s.into());
    }
}

/// Attr `value` / `nodeValue` / `textContent` getter（spec `dom-attr-value`）：owner 元素该属性**当前**值（live）。
fn native_attr_value_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some((owner, name)) = attr_owner_and_name(scope, &holder) else {
        return;
    };
    let val = with_dom(|d| d.get_attribute(owner, &name))
        .flatten()
        .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// Attr `value` / `nodeValue` / `textContent` setter（spec `dom-attr-value`）：值 ToString → set_attribute（live 写回 owner）。
fn native_attr_value_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some((owner, name)) = attr_owner_and_name(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(owner, &name, &val));
}

/// Attr `ownerElement` getter（spec `dom-attr-owner-element`）：所属 native 元素（或 null）。
fn native_attr_owner_element_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some((owner, _name)) = attr_owner_and_name(scope, &holder) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, owner) {
        rv.set(obj.into());
    }
}

/// 从对象读字符串属性（setNamedItem 读 attr.name/attr.value 用，兼容 plain 对象与 Attr 节点）。缺省 → 空串。
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

/// NamedNodeMap `item(index)`（spec `dom-namednodemap-item`）：index 处属性 → Attr 节点或 null。
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
            if let Some(attr) = make_attr_node(scope, id, name) {
                rv.set(attr.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// NamedNodeMap `getNamedItem(name)`（spec `dom-namednodemap-get-named-item`）：name 属性 → Attr 节点或 null。
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
    // 属性存在性校验（不存在 → null）；Attr 节点 value 经 getter live 读。
    let exists = with_dom(|d| d.get_attribute(id, &name)).flatten().is_some();
    if exists {
        if let Some(attr) = make_attr_node(scope, id, &name) {
            rv.set(attr.into());
        }
    } else {
        rv.set(v8::null(scope).into());
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
/// 同时建 Attr ObjectTemplate（R3122，`set_attr_template`）。
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

    // R3122 Attr 节点模板：internal slot[0]=owner ffi、slot[1]=attr 名 arena idx+1。
    let attr = v8::ObjectTemplate::new(scope);
    attr.set_internal_field_count(2);
    if let Some(k) = v8::String::new(scope, "nodeType") {
        attr.set_accessor(k.into(), native_attr_node_type_getter);
    }
    if let Some(k) = v8::String::new(scope, "name") {
        attr.set_accessor(k.into(), native_attr_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "nodeName") {
        attr.set_accessor(k.into(), native_attr_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "value") {
        attr.set_accessor_with_setter(k.into(), native_attr_value_getter, native_attr_value_setter);
    }
    if let Some(k) = v8::String::new(scope, "nodeValue") {
        attr.set_accessor_with_setter(k.into(), native_attr_value_getter, native_attr_value_setter);
    }
    if let Some(k) = v8::String::new(scope, "textContent") {
        attr.set_accessor_with_setter(k.into(), native_attr_value_getter, native_attr_value_setter);
    }
    if let Some(k) = v8::String::new(scope, "ownerElement") {
        attr.set_accessor(k.into(), native_attr_owner_element_getter);
    }
    set_attr_template(scope, attr);
}
