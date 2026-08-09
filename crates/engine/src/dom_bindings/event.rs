//! 原生 `Event` / `CustomEvent` 构造器（R3127）——注册为全局，使 `new Event(type, opts)` 产出标准
//! event 对象（`instanceof Event` 成立），stop / preventDefault 方法上**原型**（共享，非每次派发注入）。
//!
//! spec DOM `Event`：构造器读 `(type, eventInitDict?)`；init dict `bubbles`/`cancelable`/`composed`
//!（缺省 false）。派发态属性 `target`/`currentTarget`（null）、`eventPhase`（NONE=0）、
//! `defaultPrevented`/`isTrusted`（false）、`timeStamp`（0，沙箱无 perf timer，后续）。
//! [`event_target::native_dispatch_event_invoke`] 派发时覆写 `target`/`currentTarget`/`eventPhase`。
//!
//! **与 R3125/R3126 集成**：Event 实例经原型具 `stopPropagation`/`stopImmediatePropagation`/`preventDefault`
//! → [`event_target`] 派发的「缺失时注入」检查（`get(...).is_function()`）命中既有 → 不重复注入（原型方法
//! 经 `get` 透明解析）。闭合 R3124 限制③（plain object 非 `instanceof Event`）+ R3126 限制③（stop 方法
//! 注入非原型）。
//!
//! 可见性：`build_and_register` 为 `pub(super)`（mod.rs `install_dom_bindings` 全局注册调）；构造器/方法
//! 回调为私有。复用 `super::event_target::{native_stop_propagation_invoke, native_stop_immediate_invoke}`
//!（原型 stop 方法）+ `super::string_arg`（构造器读 type）。

use v8;

use super::event_target::{native_stop_immediate_invoke, native_stop_propagation_invoke};
use super::string_arg;

/// 构建并注册 `Event` + `CustomEvent` 全局构造器（mod.rs `install_dom_bindings` 末调）。
///
/// 两构造器经 FunctionTemplate 建（`new Event(...)` → V8 造实例设原型 → 调构造器回调设 init 属性）；
/// 原型模板挂 `stopPropagation`/`stopImmediatePropagation`/`preventDefault`（共享方法，非实例属性）。
/// `new Event('x') instanceof Event` 因原型链成立。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    let event_tmpl = v8::FunctionTemplate::builder(native_event_constructor_invoke).build(scope);
    register_ctor_with_proto(scope, global, "Event", event_tmpl);
    let ce_tmpl = v8::FunctionTemplate::builder(native_custom_event_constructor_invoke).build(scope);
    register_ctor_with_proto(scope, global, "CustomEvent", ce_tmpl);
}

/// 挂原型方法（stopPropagation/stopImmediatePropagation/preventDefault）到构造器模板 + 注册为全局 `name`。
fn register_ctor_with_proto(
    scope: &mut v8::PinScope,
    global: v8::Local<v8::Object>,
    name: &str,
    tmpl: v8::Local<v8::FunctionTemplate>,
) {
    let proto = tmpl.prototype_template(scope);
    let stop = v8::FunctionTemplate::builder(native_stop_propagation_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "stopPropagation") {
        proto.set(k.into(), stop.into());
    }
    let stop_imm = v8::FunctionTemplate::builder(native_stop_immediate_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "stopImmediatePropagation") {
        proto.set(k.into(), stop_imm.into());
    }
    let pd = v8::FunctionTemplate::builder(native_prevent_default_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "preventDefault") {
        proto.set(k.into(), pd.into());
    }
    let Some(f) = tmpl.get_function(scope) else {
        return;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = global.set(scope, key.into(), f.into());
}

/// 读 eventInitDict 第 `idx` 参的布尔属性（`bubbles`/`cancelable`，缺省 false）。
fn init_bool(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32, name: &str) -> bool {
    let Ok(opts) = v8::Local::<v8::Object>::try_from(args.get(idx)) else {
        return false;
    };
    v8::String::new(scope, name)
        .and_then(|k| opts.get(scope, k.into()))
        .is_some_and(|v| v.is_true())
}

/// 设 event 实例的 init 属性（type/bubbles/cancelable/composed + 派发态默认）。
fn set_event_init(
    scope: &mut v8::PinScope,
    obj: v8::Local<v8::Object>,
    event_type: &str,
    bubbles: bool,
    cancelable: bool,
) {
    fn set_str(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, name: &str, val: &str) {
        if let (Some(k), Some(v)) = (v8::String::new(scope, name), v8::String::new(scope, val)) {
            let _ = obj.set(scope, k.into(), v.into());
        }
    }
    fn set_bool(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, name: &str, val: bool) {
        if let Some(k) = v8::String::new(scope, name) {
            let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, val).into());
        }
    }
    set_str(scope, obj, "type", event_type);
    set_bool(scope, obj, "bubbles", bubbles);
    set_bool(scope, obj, "cancelable", cancelable);
    set_bool(scope, obj, "composed", false);
    // 派发态属性默认（dispatchEvent 派发时覆写 target/currentTarget/eventPhase）。
    if let Some(k) = v8::String::new(scope, "target") {
        let _ = obj.set(scope, k.into(), v8::null(scope).into());
    }
    if let Some(k) = v8::String::new(scope, "currentTarget") {
        let _ = obj.set(scope, k.into(), v8::null(scope).into());
    }
    if let Some(k) = v8::String::new(scope, "eventPhase") {
        let _ = obj.set(scope, k.into(), v8::Integer::new(scope, 0).into());
    }
    set_bool(scope, obj, "defaultPrevented", false);
    set_bool(scope, obj, "isTrusted", false);
    // timeStamp：沙箱无 perf timer（Date.now 受限），暂 0（后续接 performance.now()）。
    if let Some(k) = v8::String::new(scope, "timeStamp") {
        let _ = obj.set(scope, k.into(), v8::Integer::new(scope, 0).into());
    }
}

/// `new Event(type, eventInitDict?)` 构造器（spec `dom-event-constructor`）：设 init 属性于 `this`。
/// 不设 ReturnValue（undefined → V8 用 `this`，标准构造器语义）。
fn native_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
}

/// `new CustomEvent(type, eventInitDict?)` 构造器（spec `dom-customevent-constructor`）：Event +
/// `detail`（init dict `detail` 字段，缺省 null）。复用 [`native_event_constructor_invoke`] 的 init 属性。
fn native_custom_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
    // detail：从 init dict 读原值（任意类型），缺省 null。
    let detail = match v8::Local::<v8::Object>::try_from(args.get(1)) {
        Ok(opts) => v8::String::new(scope, "detail")
            .and_then(|k| opts.get(scope, k.into()))
            .unwrap_or_else(|| v8::null(scope).into()),
        Err(_) => v8::null(scope).into(),
    };
    if let Some(k) = v8::String::new(scope, "detail") {
        let _ = this.set(scope, k.into(), detail);
    }
}

/// `event.preventDefault()` 原型方法（spec `dom-event-prevent-default`）：仅当 `cancelable` 时设
/// `defaultPrevented=true`。headless 无浏览器默认行为，故仅记账（供 dispatchEvent 返值语义后续）。
fn native_prevent_default_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let obj = args.this();
    let cancelable = v8::String::new(scope, "cancelable")
        .and_then(|k| obj.get(scope, k.into()))
        .is_some_and(|v| v.is_true());
    if cancelable && let Some(k) = v8::String::new(scope, "defaultPrevented") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}
