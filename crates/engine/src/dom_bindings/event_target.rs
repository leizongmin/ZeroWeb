//! S4 EventTarget（addEventListener / removeEventListener / dispatchEvent）原生绑定——拆自 mod.rs
//!（RFC §3.2 子模块化 stage 2，本轮 R3117）。
//!
//! 监听器存线程局部 `gc::LISTENERS`（`(NodeId ffi, 事件类型) → Vec<Global<Value>>`，存 Value 句柄，
//! 调用时降 Function；避 Local<Function>→Local<Value> upcast）。dispatchEvent 在当前 scope 复活 Local
//! 调用，**target + bubble 两阶段冒泡**（R3125）：沿 target→祖先链逐层派发，`event.currentTarget`/
//! `eventPhase` 随传播更新（AT_TARGET=2 / BUBBLING_PHASE=3），`event.bubbles` 控制是否上溯祖先。
//! **无 capture 阶段**（需 useCapture 跟踪，后续）、**无 stopPropagation**（后续）。**无 finalizer**——
//! 移除仅靠 removeEventListener 或 reset（节点 detach 不自动清理，泄漏限制，后续 weak callback）。
//!
//! 可见性：3 个 invoke 为 `pub(super)`（mod.rs Element 模板注册经 `event_target::` 调）。读
//! `super::read_node_id` / `super::string_arg`（mod.rs 私有——Rust 规则：私有项对后代模块可见）。

use v8;

use zero_dom::NodeId;

use super::gc::{add_listener, encode_node_id, listeners_local, remove_listener, with_dom};
use super::{get_or_create_native_element, read_node_id, string_arg};

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

// ── R3126 stopPropagation / stopImmediatePropagation 注入方法 ──
//
// 监听器内调 `event.stopPropagation()` / `stopImmediatePropagation()` → 设 event 对象内部 flag
//（`__zw_stop` / `__zw_stop_immediate`），[`native_dispatch_event_invoke`] 冒泡循环读 flag 早退。
// 注入仅当 event 无既有同名方法（不覆盖 page 构造的 Event 实例原生方法，待 native Event 构造器）。

/// `event.stopPropagation()` 注入实现：设 `this.__zw_stop = true`（止上溯祖先；当前节点剩余监听器仍触发）。
fn native_stop_propagation_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let obj = args.this();
    if let Some(k) = v8::String::new(scope, "__zw_stop") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}

/// `event.stopImmediatePropagation()` 注入实现：设 `this.__zw_stop_immediate = true`（止当前节点剩余
/// 监听器 + 上溯——立即终止整个派发）。spec `dom-event-stop-immediate-propagation`。
fn native_stop_immediate_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let obj = args.this();
    if let Some(k) = v8::String::new(scope, "__zw_stop_immediate") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}

/// `dispatchEvent(event)`（spec `dom-eventtarget-dispatch-event`）：event 为对象读 `.type`，
/// 或直接 type 字符串（包成 `{type:str}` 对象）。**target + bubble 两阶段冒泡**（R3125，闭合 R3109
/// 不冒泡限制）：沿 target→祖先链逐层派发，每层取监听器快照（复活 Local，释放 borrow 避回调再入）调用
///（this = 当前层元素，参 = event）。`event.target` = 派发目标（固定）；`event.currentTarget` = 当前层
/// 元素（随传播变）；`event.eventPhase` = AT_TARGET(2) / BUBBLING_PHASE(3)；`event.bubbles` 控制是否
/// 上溯祖先（默认 false）。派发后 `currentTarget=null`、`eventPhase=0`（spec）。
///
/// **stopPropagation / stopImmediatePropagation**（R3126）：注入两方法（缺失时）+ 派发前复位内部 flag
///（支持同 event 对象重派发）。监听器调 stopPropagation → 止上溯（当前节点剩余监听器仍触发）；
/// stopImmediatePropagation → 立即终止（当前节点剩余监听器 + 上溯均止）。
///
/// **限制**（后续切片）：① 无 capture 阶段（需 addEventListener useCapture 跟踪 + 倒序祖先派发）；
/// ② 无 preventDefault 返值语义（恒返 true）。返 true（spec：未 preventDefault）。
pub(super) fn native_dispatch_event_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let target_id = match read_node_id(scope, &this) {
        Some(id) => id,
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
    // event 对象：对象原样用；字符串/其他包成 {type:str}（dispatchEvent 入参标准化）。
    let event_obj = match v8::Local::<v8::Object>::try_from(event) {
        Ok(obj) => obj,
        Err(_) => {
            let obj = v8::Object::new(scope);
            if let Some(k) = v8::String::new(scope, "type") {
                let _ = obj.set(scope, k.into(), event);
            }
            obj
        }
    };
    // spec：event.target = 派发目标（固定不变）。
    if let Some(k) = v8::String::new(scope, "target") {
        let _ = event_obj.set(scope, k.into(), this.into());
    }
    // event.bubbles（缺省 false）控制是否上溯祖先。
    let bubbles = v8::String::new(scope, "bubbles")
        .and_then(|k| event_obj.get(scope, k.into()))
        .is_some_and(|v| v.is_true());
    // R3126 stopPropagation / stopImmediatePropagation：注入 stop 方法（缺失时）+ 复位内部 flag
    //（每次派发 fresh，支持同 event 对象重派发）。监听器调 stopPropagation 设 __zw_stop=true（止上溯）；
    // stopImmediatePropagation 设 __zw_stop_immediate=true（止当前节点剩余监听器 + 上溯）。
    let key_stop = v8::String::new(scope, "__zw_stop");
    let key_stop_imm = v8::String::new(scope, "__zw_stop_immediate");
    if let Some(k) = &key_stop {
        let _ = event_obj.set(scope, (*k).into(), v8::Boolean::new(scope, false).into());
    }
    if let Some(k) = &key_stop_imm {
        let _ = event_obj.set(scope, (*k).into(), v8::Boolean::new(scope, false).into());
    }
    // 注入 stopPropagation / stopImmediatePropagation（仅当 event 无既有同名方法，不覆盖原生 Event）。
    if let Some(k) = v8::String::new(scope, "stopPropagation") {
        let has = event_obj.get(scope, k.into()).is_some_and(|v| v.is_function());
        if !has {
            let tmpl = v8::FunctionTemplate::builder(native_stop_propagation_invoke).build(scope);
            if let Some(f) = tmpl.get_function(scope) {
                let _ = event_obj.set(scope, k.into(), f.into());
            }
        }
    }
    if let Some(k) = v8::String::new(scope, "stopImmediatePropagation") {
        let has = event_obj.get(scope, k.into()).is_some_and(|v| v.is_function());
        if !has {
            let tmpl = v8::FunctionTemplate::builder(native_stop_immediate_invoke).build(scope);
            if let Some(f) = tmpl.get_function(scope) {
                let _ = event_obj.set(scope, k.into(), f.into());
            }
        }
    }
    // 沿 parent 链收集 [target, parent, ..., root]（bubble 序；target 在首）。with_dom 闭包内纯读
    // 收集 NodeId，释放 borrow 后再逐层派发（派发可能再入 addEventListener/removeEventListener）。
    let chain: Vec<NodeId> = with_dom(|d| {
        let mut chain = vec![target_id];
        let mut cur = d.get(target_id).and_then(|n| n.parent);
        while let Some(p) = cur {
            chain.push(p);
            cur = d.get(p).and_then(|n| n.parent);
        }
        chain
    })
    .unwrap_or_default();
    let key_ct = v8::String::new(scope, "currentTarget");
    let key_phase = v8::String::new(scope, "eventPhase");
    let call_args = [event_obj.into()];
    'chain: for (i, &node_id) in chain.iter().enumerate() {
        // 首层 = target（AT_TARGET）；其后 = bubble（仅当 bubbles=true）。非 bubbles 事件只派发 target。
        if i > 0 && !bubbles {
            break;
        }
        // 当前层 native 元素（currentTarget + listener this）。get_or_create 对任意 NodeId 返包装
        //（Document 等非 Element 亦得包装，但其上无可达 native 监听器，currentTarget 不被观测）。
        let curr = get_or_create_native_element(scope, node_id);
        if let Some(c) = &curr
            && let Some(k) = &key_ct
        {
            let _ = event_obj.set(scope, (*k).into(), (*c).into());
        }
        if let Some(k) = &key_phase {
            let phase = if i == 0 { 2 } else { 3 }; // AT_TARGET / BUBBLING_PHASE
            let _ = event_obj.set(scope, (*k).into(), v8::Integer::new(scope, phase).into());
        }
        // 复活监听器 Local 列表（gc.rs 不持 borrow 跨 JS 回调，防再入 panic）。
        let ffi = encode_node_id(node_id);
        let listeners = listeners_local(scope, ffi, &event_type);
        // listener this = 当前层元素（spec：监听器 this = currentTarget）；无包装回落 target this。
        let recv = curr.map(|c| c.into()).unwrap_or_else(|| this.into());
        for listener in listeners {
            if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
                let _ = func.call(scope, recv, &call_args);
            }
            // stopImmediatePropagation：止当前节点剩余监听器 + 上溯 → 立即终止整个派发。
            if key_stop_imm
                .and_then(|k| event_obj.get(scope, k.into()))
                .is_some_and(|v| v.is_true())
            {
                break 'chain;
            }
        }
        // stopPropagation：当前节点监听器已尽，止上溯祖先（i==0 时也防再入祖先）。
        if key_stop
            .and_then(|k| event_obj.get(scope, k.into()))
            .is_some_and(|v| v.is_true())
        {
            break 'chain;
        }
    }
    // 派发结束：currentTarget=null、eventPhase=NONE(0)（spec：派发后 currentTarget 为 null）。
    if let Some(k) = &key_ct {
        let _ = event_obj.set(scope, (*k).into(), v8::null(scope).into());
    }
    if let Some(k) = &key_phase {
        let _ = event_obj.set(scope, (*k).into(), v8::Integer::new(scope, 0).into());
    }
    rv.set(v8::Boolean::new(scope, true).into());
}
