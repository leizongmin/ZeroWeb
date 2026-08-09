//! S4 EventTarget（addEventListener / removeEventListener / dispatchEvent）原生绑定——拆自 mod.rs
//!（RFC §3.2 子模块化 stage 2，本轮 R3117）。
//!
//! 监听器存线程局部 `gc::LISTENERS`（`(NodeId ffi, 事件类型) → Vec<Global<Value>>`，存 Value 句柄，
//! 调用时降 Function；避 Local<Function>→Local<Value> upcast）。dispatchEvent 在当前 scope 复活 Local
//! 调用（**不冒泡**，最小切片）。**无 finalizer**——移除仅靠 removeEventListener 或 reset（节点 detach
//! 不自动清理，泄漏限制，后续 weak callback）。
//!
//! 可见性：3 个 invoke 为 `pub(super)`（mod.rs Element 模板注册经 `event_target::` 调）。读
//! `super::read_node_id` / `super::string_arg`（mod.rs 私有——Rust 规则：私有项对后代模块可见）。

use v8;

use super::gc::{add_listener, encode_node_id, listeners_local, remove_listener};
use super::{read_node_id, string_arg};

/// `addEventListener(type, listener)`（spec `dom-eventtarget-add-event-listener`）：
/// listener 存为 `Global<Value>`（线程局部 LISTENERS，键 = `(NodeId ffi, 事件类型)`）。
/// 非 function 参 → 忽略（spec 应抛 TypeError，本切片 best-effort）。
pub(super) fn native_add_event_listener_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let event_type = string_arg(scope, &args, 0);
    if event_type.is_empty() {
        return;
    }
    // 仅 function 参持久化（存 Value 句柄，调用时降 Function）。
    if !args.get(1).is_function() {
        return;
    }
    let ffi = encode_node_id(id);
    add_listener(ffi, event_type, v8::Global::new(scope, args.get(1)));
}

/// `removeEventListener(type, listener)`（spec `dom-eventtarget-remove-event-listener`）：
/// 移除与 listener 同身份（strict_equals）的监听器。
pub(super) fn native_remove_event_listener_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let event_type = string_arg(scope, &args, 0);
    let ffi = encode_node_id(id);
    remove_listener(scope, ffi, &event_type, args.get(1));
}

/// `dispatchEvent(event)`（spec `dom-eventtarget-dispatch-event`）：event 为对象读 `.type`，
/// 或直接 type 字符串；按 type 取监听器快照（复活 Local，释放 borrow 避回调再入）逐个调用
///（this = 元素，参 = event）。**不冒泡**（最小切片，后续）。返 true（spec：未 preventDefault）。
pub(super) fn native_dispatch_event_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let ffi = match read_node_id(scope, &this) {
        Some(id) => encode_node_id(id),
        None => {
            rv.set(v8::Boolean::new(scope, true).into());
            return;
        }
    };
    let event = args.get(0);
    // event.type：字符串直接用；对象读 `.type` 属性。
    let event_type = if event.is_string() {
        event
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default()
    } else if let Ok(obj) = v8::Local::<v8::Object>::try_from(event) {
        v8::String::new(scope, "type")
            .and_then(|k| obj.get(scope, k.into()))
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    // 复活监听器 Local 列表（gc.rs 不持 borrow 跨 JS 回调，防再入 panic）。
    let listeners = listeners_local(scope, ffi, &event_type);
    let call_args = [event];
    for listener in listeners {
        if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
            let _ = func.call(scope, this.into(), &call_args);
        }
    }
    rv.set(v8::Boolean::new(scope, true).into());
}
