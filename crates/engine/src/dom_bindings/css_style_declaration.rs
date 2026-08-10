//! R3151 CSSStyleDeclaration（`element.style`）原生绑定——拆自 mod.rs（RFC §3.2 子模块化）。
//!
//! 内聚内联样式面：`style` getter（Element 模板注册）+ CSSStyleDeclaration 模板（cssText(+setter) /
//! length / item / getPropertyValue / setProperty / removeProperty + **named-property-handler** 拦
//! camelCase 动态属性 `el.style.color` / `el.style.backgroundColor`）。internal slot[0] = owner element
//! NodeId；**live**——读经 owner 元素当前 `style` 属性 `parse_style`，mutation 经 `set_attribute("style",`
//! `serialize)` 写回（与 polyfill CSSStyleDeclaration 一致）。
//!
//! named-property-handler（V8 `ObjectTemplate::set_named_property_handler`）：getter/setter/deleter
//! 拦**任意**属性名——camelCase→kebab 转 CSS 属性名（`backgroundColor`→`background-color`），读/写/删
//! owner `style` 属性。cssText/length/item/getPropertyValue/setProperty/removeProperty 等**保留名**返
//! `Intercepted::kNo`（fallthrough 到模板上注册的方法/accessor），避免被动态属性拦截器吞掉。
//!
//! 身份缓存 STYLE_OBJECTS（同元素返同对象，spec identity——polyfill 旧每调新建，native 修正为 spec 合规）；
//! R3134/R3145 同 pattern weak 化（JS 丢引用即 GC）。
//!
//! 可见性：`native_style_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs 的 Element 模板
//! 注册 + install_dom_bindings 调）；余私有。读 `super::read_node_id` / `super::string_arg` /
//! `super::local_value_to_string` + `super::gc::{...}`。

use v8;

use zero_dom::NodeId;

use super::gc::{
    cache_style, cached_style, decode_node_id, encode_node_id, set_style_template, style_template_local, with_dom,
    with_dom_mut,
};
use super::{local_value_to_string, read_node_id, string_arg};

/// named handler 对这些名返 `kNo`（fallthrough）：① CSSStyleDeclaration 模板方法/accessor
///（cssText/length/item/getPropertyValue/setProperty/removeProperty——避免被动态拦截器吞）；② Object/
/// Promise 协议名（constructor/toString/valueOf/then 等——让原型提供真值，避免 `el.style.constructor` 返
/// 空串破坏对象互操作）。其余任意 string 名按 CSS 属性处理（camelCase→kebab，返值或空串）。
const RESERVED: &[&str] = &[
    "cssText",
    "length",
    "item",
    "getPropertyValue",
    "setProperty",
    "removeProperty",
    // Object / Promise 协议名（fallthrough 到原型，保互操作正确）。
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

/// 从 CSSStyleDeclaration 对象 internal slot[0] 读 owner element NodeId。slot[0]=owner ffi（External 值）。
/// 非 CSD 对象 / stale → `None`。
fn csd_owner(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<NodeId> {
    let ffi = obj.get_internal_field(scope, 0)?.cast::<v8::External>().value() as usize as u64;
    Some(decode_node_id(ffi))
}

// ── style 属性 parse / serialize（CSSOM § 声明序列化简化）──────────────

/// 解析 `style` 属性串为有序 (prop, value) 声明列表（去重保首次出现位置）。格式 `prop: value; prop2: val2`。
/// 空段 / 无冒号段跳过；重复 prop 仅留首次（后续丢弃）。prop/value trim。
fn parse_style(s: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for seg in s.split(';') {
        let Some((k, v)) = seg.split_once(':') else { continue };
        let key = k.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        if out.iter().any(|(pk, _)| *pk == key) {
            continue; // 去重保首
        }
        out.push((key, v.trim().to_string()));
    }
    out
}

/// 序列化声明列表为 `style` 属性串（`prop: value; prop2: val2`，无尾分号）。CSSOM 简化序列化。
fn serialize_style(props: &[(String, String)]) -> String {
    props
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 读 owner 元素当前 style 声明（live——经 `style` 属性 `parse_style`）。
fn current_props(doc: &zero_dom::Document, id: NodeId) -> Vec<(String, String)> {
    doc.get_attribute(id, "style")
        .map(|s| parse_style(&s))
        .unwrap_or_default()
}

/// 写回 owner 元素 `style` 属性（空列表 → 移除属性，spec：无声明时 style 属性为空串；dom set_attribute 写空串）。
fn write_props(doc: &mut zero_dom::Document, id: NodeId, props: &[(String, String)]) {
    let joined = serialize_style(props);
    doc.set_attribute(id, "style", &joined);
}

// ── camelCase ↔ kebab-case 转换（CSSOM 属性名映射简化）────────────────

/// camelCase → kebab-case CSS 属性名（`backgroundColor`→`background-color`）。标准转换：大写前插 '-' + 转小写。
/// 特例 `cssFloat`/`styleFloat` → `float`（spec `float` 暴露为 cssFloat）。
/// **已知限制**：厂商前缀（`webkitTransform` 浏览器→`-webkit-transform`，本实现→`webkit-transform`）不特殊处理
///（headless 现代代码极少用厂商前缀，documented）。
fn camel_to_kebab(name: &str) -> String {
    if name == "cssFloat" || name == "styleFloat" {
        return "float".into();
    }
    let mut out = String::with_capacity(name.len() + 2);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ── `style` getter：返 owner 元素的 CSSStyleDeclaration（缓存保身份）──

/// `style` getter（spec `dom-element-style`）：返 owner 元素的 CSSStyleDeclaration（缓存保身份）。
pub(super) fn native_style_getter(
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
    // 缓存命中 → 同对象（spec `el.style === el.style`）。
    if let Some(cached) = cached_style(scope, ffi) {
        rv.set(cached.into());
        return;
    }
    let Some(tmpl) = style_template_local(scope) else {
        return;
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_style(scope, ffi, obj);
    rv.set(obj.into());
}

// ── named-property-handler：拦 camelCase 动态属性 ──────────────────────

/// reserved 名（模板方法/accessor）→ `kNo`（fallthrough）；非 string 名 → `kNo`。否则返该 CSS 属性值
///（camelCase→kebab，读 owner `style`，未设返空串）。**恒拦截**（CSSStyleDeclaration 对任意名返值）。
fn native_style_named_getter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo; // Symbol 等非 string → fallthrough
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo; // 模板方法/accessor → fallthrough
    }
    let prop = camel_to_kebab(&name);
    let val = with_dom(|d| {
        current_props(d, owner)
            .into_iter()
            .find(|(k, _)| *k == prop)
            .map(|(_, v)| v)
    })
    .flatten()
    .unwrap_or_default();
    if let Some(vs) = v8::String::new(scope, &val) {
        rv.set(vs.into());
    }
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`；否则 camelCase→kebab 写 owner `style` 属性（upsert + 重新序列化）。
fn native_style_named_setter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let prop = camel_to_kebab(&name);
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        if let Some(p) = props.iter_mut().find(|(k, _)| *k == prop) {
            p.1 = val; // upsert（已存在→更新值）
        } else {
            props.push((prop, val));
        }
        write_props(d, owner, &props);
    });
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`；否则 camelCase→kebab 移除 owner `style` 该属性（重序列化）。返 true（已删）。
fn native_style_named_deleter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Boolean>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let prop = camel_to_kebab(&name);
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        props.retain(|(k, _)| *k != prop);
        write_props(d, owner, &props);
    });
    rv.set_bool(true);
    v8::Intercepted::kYes
}

// ── cssText / length / item / getPropertyValue / setProperty / removeProperty ──

/// `cssText` getter（spec `dom-cssstyledeclaration-csstext`）：owner `style` 属性的规范化序列化（parse→serialize）。
fn native_style_css_text_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return;
    };
    let text = with_dom(|d| serialize_style(&current_props(d, owner))).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &text) {
        rv.set(s.into());
    }
}

/// `cssText` setter（spec）：值 ToString → `set_attribute("style", val)`（整体替换，无解析校验）。
fn native_style_css_text_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(owner, "style", &val));
}

/// `length` getter（spec）：owner style 声明数（去重后唯一属性数）。
fn native_style_length_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        rv.set(v8::Integer::new(scope, 0).into());
        return;
    };
    let len = with_dom(|d| current_props(d, owner).len()).unwrap_or(0);
    rv.set(v8::Integer::new(scope, len as i32).into());
}

/// `item(index)`（spec `dom-cssstyledeclaration-item`）：index 处属性名（kebab），越界 → 空串。
fn native_style_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let idx = args.get(0).integer_value(scope).unwrap_or(-1);
    let name = with_dom(|d| current_props(d, owner).get(idx as usize).map(|(k, _)| k.clone())).flatten();
    let name = name.unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &name) {
        rv.set(s.into());
    }
}

/// `getPropertyValue(prop)`（spec）：prop（kebab）值，未设 → 空串。
fn native_style_get_property_value_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let val = with_dom(|d| {
        current_props(d, owner)
            .into_iter()
            .find(|(k, _)| *k == prop)
            .map(|(_, v)| v)
    })
    .flatten()
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// `setProperty(prop, value)`（spec）：prop（kebab）upsert + 重序列化写回。返 undefined（spec）。
fn native_style_set_property_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let val = string_arg(scope, &args, 1);
    if prop.is_empty() {
        return;
    }
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        if let Some(p) = props.iter_mut().find(|(k, _)| *k == prop) {
            p.1 = val.clone();
        } else {
            props.push((prop, val));
        }
        write_props(d, owner, &props);
    });
}

/// `removeProperty(prop)`（spec）：移除 prop（kebab），返旧值（未设 → 空串）。重序列化写回。
fn native_style_remove_property_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let old = with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        let pos = props.iter().position(|(k, _)| *k == prop);
        let old = pos
            .and_then(|i| props.get(i).map(|(_, v)| v.clone()))
            .unwrap_or_default();
        if pos.is_some() {
            props.retain(|(k, _)| *k != prop);
            write_props(d, owner, &props);
        }
        old
    })
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &old) {
        rv.set(s.into());
    }
}

// ── 模板构建 ──────────────────────────────────────────────────────────

/// 建 CSSStyleDeclaration ObjectTemplate（cssText(+setter) / length / item / getPropertyValue /
/// setProperty / removeProperty + named-property-handler 拦 camelCase 动态属性）并缓存（`set_style_template`）。
/// `install_dom_bindings` 调（Element 模板建之后）。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    let csd = v8::ObjectTemplate::new(scope);
    csd.set_internal_field_count(1);
    // cssText accessor（getter 规范化序列化 / setter 整体替换）。
    if let Some(k) = v8::String::new(scope, "cssText") {
        csd.set_accessor_with_setter(k.into(), native_style_css_text_getter, native_style_css_text_setter);
    }
    // length getter。
    if let Some(k) = v8::String::new(scope, "length") {
        csd.set_accessor(k.into(), native_style_length_getter);
    }
    let item = v8::FunctionTemplate::builder(native_style_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "item") {
        csd.set(k.into(), item.into());
    }
    let gpv = v8::FunctionTemplate::builder(native_style_get_property_value_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getPropertyValue") {
        csd.set(k.into(), gpv.into());
    }
    let sp = v8::FunctionTemplate::builder(native_style_set_property_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "setProperty") {
        csd.set(k.into(), sp.into());
    }
    let rp = v8::FunctionTemplate::builder(native_style_remove_property_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeProperty") {
        csd.set(k.into(), rp.into());
    }
    // named-property-handler：拦 camelCase 动态属性（reserved 名 fallthrough 到上面模板方法/accessor）。
    csd.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(native_style_named_getter)
            .setter(native_style_named_setter)
            .deleter(native_style_named_deleter),
    );
    set_style_template(scope, csd);
}
