//! R3152 DOMStringMap（`element.dataset`）原生绑定——拆自 mod.rs（RFC §3.2 子模块化）。
//!
//! `data-*` 属性的 camelCase 键 live 视图（spec HTML §3.2.6 `dom-dataset`）。`dataset.fooBar`
//! ↔ `data-foo-bar` 属性。经 **named-property-handler**（V8 `set_named_property_handler`，R3151 首用
//! 于 style，本轮复用）拦任意 camelCase 属性名——camelCase→`data-`kebab 转属性名，读/写/删/枚举
//! owner 元素的 `data-*` 属性（经 dom `get_attribute`/`set_attribute`/`remove_attribute`/
//! `attribute_names`）。
//!
//! 与 CSSStyleDeclaration（R3151）区别：① dataset 缺失键返 **undefined**（对象语义，非 style 的 ""）；
//! ② 无 parse——每 `data-*` 属性独立（无声明列表）；③ 有 enumerator（`Object.keys(el.dataset)` 枚举
//! `data-*` 属性名 → camelCase 键）。
//!
//! reserved 名（Object/Promise 协议：constructor/toString/valueOf/then 等）→ `Intercepted::kNo`
//! fallthrough（让原型提供真值，保对象互操作——同 R3151 style）。身份缓存 DATASET_OBJECTS（同元素返同对象，
//! spec identity；R3134/R3145/R3151 同 pattern weak 化）。
//!
//! 可见性：`native_dataset_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs Element 模板
//! 注册 + install_dom_bindings 调）；余私有。读 `super::read_node_id` / `super::local_value_to_string`
//! + `super::gc::{...}`。

use v8;

use zero_dom::NodeId;

use super::gc::{
    cache_dataset, cached_dataset, dataset_template_local, decode_node_id, encode_node_id, set_dataset_template,
    with_dom, with_dom_mut,
};
use super::{local_value_to_string, read_node_id};

/// named handler 对这些名返 `kNo`（fallthrough 到原型，保对象互操作——避免 `el.dataset.constructor`
/// 返 undefined 破坏对象协议）。其余 string 名按 dataset 键处理。
const RESERVED: &[&str] = &[
    "constructor",
    "toString",
    "toLocaleString",
    "valueOf",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "__proto__",
    "then",
];

/// 从 DOMStringMap 对象 internal slot[0] 读 owner element NodeId。
fn dataset_owner(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<NodeId> {
    let ffi = obj.get_internal_field(scope, 0)?.cast::<v8::External>().value() as usize as u64;
    Some(decode_node_id(ffi))
}

// ── dataset 键转换：camelCase ↔ `data-`kebab（fooBar ↔ data-foo-bar）────────

/// camelCase → `data-`kebab 属性名（`fooBar`→`data-foo-bar`）。大写前插 '-' + 转小写，前缀 `data-`。
fn prop_to_attr(prop: &str) -> String {
    let mut out = String::from("data-");
    for c in prop.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `data-`kebab 属性名 → camelCase 键（`data-foo-bar`→`fooBar`）。去 `data-` 前缀，'-' 后小写转大写。
fn attr_to_prop(attr: &str) -> String {
    let rest = attr.strip_prefix("data-").unwrap_or(attr);
    let mut out = String::new();
    let mut up = false;
    for c in rest.chars() {
        if c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

// ── `dataset` getter：返 owner 元素的 DOMStringMap（缓存保身份）──

/// `dataset` getter（spec `dom-dataset`）：返 owner 元素的 DOMStringMap（缓存保身份）。
pub(super) fn native_dataset_getter(
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
    if let Some(cached) = cached_dataset(scope, ffi) {
        rv.set(cached.into());
        return;
    }
    let Some(tmpl) = dataset_template_local(scope) else {
        return;
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_dataset(scope, ffi, obj);
    rv.set(obj.into());
}

// ── named-property-handler：拦 camelCase 动态键 ↔ data-* 属性 ──────────

/// reserved / 非 string → `kNo`。否则 prop→`data-`kebab 读 owner 该属性：在→返值（Some），不在→undefined
///（None，rv 默认 undefined——对象语义，区别于 style 的 ""）。恒拦截非 reserved string。
fn native_dataset_named_getter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = dataset_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo; // Symbol 等 → fallthrough
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo; // 协议名 → fallthrough 到原型
    }
    let attr = prop_to_attr(&name);
    // 在→返值；不在→undefined（dataset 缺失键返 undefined，对象语义）。
    if let Some(val) = with_dom(|d| d.get_attribute(owner, &attr)).flatten()
        && let Some(vs) = v8::String::new(scope, &val)
    {
        rv.set(vs.into());
    }
    // None → 不 set rv（默认 undefined）；仍 kYes（拦截，值为 undefined）。
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`。否则 prop→`data-`kebab 写 owner 该属性（值 ToString）。
fn native_dataset_named_setter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = dataset_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let attr = prop_to_attr(&name);
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(owner, &attr, &val));
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`。否则 prop→`data-`kebab 移除 owner 该属性。返 true（已删）。
fn native_dataset_named_deleter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Boolean>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = dataset_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let attr = prop_to_attr(&name);
    with_dom_mut(|d| d.remove_attribute(owner, &attr));
    rv.set_bool(true);
    v8::Intercepted::kYes
}

/// enumerator：收集 owner 全部 `data-*` 属性名 → camelCase 键（`Object.keys(el.dataset)`）。
fn native_dataset_named_enumerator(
    scope: &mut v8::PinScope,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Array>,
) {
    let holder = args.holder();
    let Some(owner) = dataset_owner(scope, &holder) else {
        rv.set(v8::Array::new(scope, 0));
        return;
    };
    let keys: Vec<String> = with_dom(|d| {
        d.attribute_names(owner)
            .into_iter()
            .filter(|n| n.starts_with("data-"))
            .map(|n| attr_to_prop(&n))
            .collect()
    })
    .unwrap_or_default();
    let arr = v8::Array::new(scope, keys.len() as i32);
    for (i, k) in keys.into_iter().enumerate() {
        if let Some(s) = v8::String::new(scope, &k) {
            let _ = arr.set_index(scope, i as u32, s.into());
        }
    }
    rv.set(arr);
}

// ── 模板构建 ──────────────────────────────────────────────────────────

/// 建 DOMStringMap ObjectTemplate（named-property-handler 拦 camelCase 动态键 ↔ `data-*` 属性）并缓存
///（`set_dataset_template`）。`install_dom_bindings` 调（Element 模板建之后）。internal slot[0] = owner NodeId。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    let dsm = v8::ObjectTemplate::new(scope);
    dsm.set_internal_field_count(1);
    dsm.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(native_dataset_named_getter)
            .setter(native_dataset_named_setter)
            .deleter(native_dataset_named_deleter)
            .enumerator(native_dataset_named_enumerator),
    );
    set_dataset_template(scope, dsm);
}
