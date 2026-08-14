//! S4 EventTarget（addEventListener / removeEventListener / dispatchEvent）原生绑定——拆自 mod.rs
//!（RFC §3.2 子模块化 stage 2，本轮 R3117）。
//!
//! 监听器存线程局部 `gc::LISTENERS`（`(NodeId ffi, 事件类型) → Vec<(capture, Global<Value>)>`，存 Value 句柄，
//! 调用时降 Function；避 Local<Function>→Local<Value> upcast）。单列表保**全局注册序**（capture/bubble 交错
//! 按 addEventListener 序）。dispatchEvent 在当前 scope 复活 Local 调用，**三阶段派发**（R3128）：capture
//!（祖先 root→parent 倒序、CAPTURING_PHASE=1，仅 capture 监听器）→ target（AT_TARGET=2，**全部监听器按注册序**，
//! 闭合 R3128 限制① R3135）→ bubble（祖先 parent→root 正序、BUBBLING_PHASE=3，仅 bubble 监听器，仅 bubbles）。
//! `event.currentTarget`/`eventPhase` 随传播更新。**无 stopPropagation 捕获态分离**（stopPropagation 跨阶段
//! 生效）；节点包装器 weak 化 + 终结器清监听器（R3133，闭合 detach 泄漏）。
//!
//! 可见性：3 个 invoke 为 `pub(super)`（mod.rs Element 模板注册经 `event_target::` 调）。读
//! `super::read_node_id` / `super::string_arg`（mod.rs 私有——Rust 规则：私有项对后代模块可见）。

use v8;

use zero_dom::NodeId;

use super::gc::{
    active_element, add_listener, encode_node_id, listener_present, listeners_local, remove_listener,
    set_active_element, with_dom,
};
use super::{get_or_create_native_element, read_node_id, string_arg};

/// `addEventListener(type, listener, useCapture?)`（spec `dom-eventtarget-add-event-listener`）：
/// listener 存为 `(capture, Global<Value>)` 追加到线程局部 LISTENERS（键 = `(NodeId ffi, 事件类型)`，
/// 单列表保全局注册序——R3135）。`useCapture`（第 3 参，R3128）支持 bool 或 `{capture: bool}` options
/// 对象，缺省 false——区分 capture（祖先倒序、CAPTURING_PHASE）vs bubble（祖先正序）监听器。非 function 参
/// → 忽略（spec 应抛 TypeError，本切片 best-effort）。
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
    let capture = capture_arg(scope, &args, 2);
    let ffi = encode_node_id(id);
    add_listener(ffi, event_type, capture, v8::Global::new(scope, args.get(1)));
}

/// `removeEventListener(type, listener, useCapture?)`（spec `dom-eventtarget-remove-event-listener`）：
/// 移除与 listener 同身份（strict_equals）的监听器。`capture`（第 3 参）须与 addEventListener 一致
///（spec：capture/bubble 监听器独立键，移除须匹配 capture 标志）。
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
    let capture = capture_arg(scope, &args, 2);
    let ffi = encode_node_id(id);
    remove_listener(scope, ffi, &event_type, capture, args.get(1));
}

/// 读 addEventListener/removeEventListener 第 `idx` 参的 useCapture：bool 直接取（`is_true`），
/// 或 `{capture: bool}` options 对象读 `.capture`；缺省（undefined）→ false（spec）。
fn capture_arg(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32) -> bool {
    let v = args.get(idx);
    if let Ok(opts) = v8::Local::<v8::Object>::try_from(v) {
        v8::String::new(scope, "capture")
            .and_then(|k| opts.get(scope, k.into()))
            .is_some_and(|x| x.is_true())
    } else {
        v.is_true()
    }
}

// ── R3126 stopPropagation / stopImmediatePropagation 注入方法 ──
//
// 监听器内调 `event.stopPropagation()` / `stopImmediatePropagation()` → 设 event 对象内部 flag
//（`__zw_stop` / `__zw_stop_immediate`），[`native_dispatch_event_invoke`] 冒泡循环读 flag 早退。
// 注入仅当 event 无既有同名方法（不覆盖 page 构造的 Event 实例原生方法，待 native Event 构造器）。

/// `event.stopPropagation()` 注入实现：设 `this.__zw_stop = true`（止上溯祖先；当前节点剩余监听器仍触发）。
pub(super) fn native_stop_propagation_invoke(
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
pub(super) fn native_stop_immediate_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let obj = args.this();
    if let Some(k) = v8::String::new(scope, "__zw_stop_immediate") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}

/// 三阶段派发核心（spec `dom-eventtarget-dispatch` 的派发算法），[`native_dispatch_event_invoke`]
/// 与 [`native_element_click_invoke`]（R3147）共用。设 `event.target`（固定）+ 读 `bubbles` + 注入
/// stop 方法（缺失时）+ 复位 stop flag（支持同 event 重派发）+ 三阶段 capture→target→bubble 派发
/// ，派发后复位 `currentTarget`/`eventPhase`。返 `!(cancelable && defaultPrevented)`（spec dispatchEvent
/// 返值语义；click() 同——未 preventDefault 时 true）。
///
/// event_obj 须已就绪（dispatchEvent 经入参标准化，click() 构造 `{type:'click',bubbles,cancelable}`）。
fn dispatch_event_impl(
    scope: &mut v8::PinScope,
    target_id: NodeId,
    this: v8::Local<v8::Object>,
    event_obj: v8::Local<v8::Object>,
) -> bool {
    // event.type（listener 键）：从 event_obj.type 读。
    let event_type = v8::String::new(scope, "type")
        .and_then(|k| event_obj.get(scope, k.into()))
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    // spec：event.target = 派发目标（固定不变）。
    if let Some(k) = v8::String::new(scope, "target") {
        let _ = event_obj.set(scope, k.into(), this.into());
    }
    // srcElement（R32，spec `dom-event-srcelement`）= target 的 legacy 别名，与 target 同步设派发目标。
    if let Some(k) = v8::String::new(scope, "srcElement") {
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
    // R3147 preventDefault 注入（缺失时——同 stop 方法 pattern）：plain event 对象（如 element.click()
    // 合成事件，无 Event 原型）经此获 preventDefault，使监听器 `e.preventDefault()` 可用 + 派发返值正确
    // 反映 defaultPrevented。原生 Event 实例经原型链已有 → 「缺失时」守卫跳过（不覆盖）。
    if let Some(k) = v8::String::new(scope, "preventDefault") {
        let has = event_obj.get(scope, k.into()).is_some_and(|v| v.is_function());
        if !has {
            let tmpl = v8::FunctionTemplate::builder(super::event::native_prevent_default_invoke).build(scope);
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
    // R3128 三阶段派发：capture（祖先 root→parent 倒序）→ target（AT_TARGET）→ bubble（祖先
    // parent→root 正序，仅 bubbles）。经 (node, phase) 访问列表单循环。
    // R3135：每节点取**全部**监听器（注册序，含 capture 标志），按 phase 过滤——target 阶段触发全部
    //（闭合 R3128 限制①：注册序跨 capture/bubble，非旧 capture-桶先/bubble-桶后）；capture 阶段仅 capture，
    // bubble 阶段仅 bubble。stopPropagation 当前节点监听器全尽后才止后续节点；stopImmediatePropagation 立即止。
    let mut visits: Vec<(NodeId, i32)> = Vec::with_capacity(chain.len() * 2);
    for &n in chain[1..].iter().rev() {
        visits.push((n, 1)); // CAPTURING_PHASE（祖先倒序）
    }
    visits.push((target_id, 2)); // AT_TARGET
    for &n in chain[1..].iter() {
        visits.push((n, 3)); // BUBBLING_PHASE（祖先正序）
    }
    let mut halted = false;
    for (node_id, phase) in visits {
        if halted {
            break;
        }
        // 非 bubbles 事件：bubble 阶段（phase==3）整体跳过（capture + target 仍派发）。
        if phase == 3 && !bubbles {
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
            let _ = event_obj.set(scope, (*k).into(), v8::Integer::new(scope, phase).into());
        }
        let ffi = encode_node_id(node_id);
        let recv = curr.map(|c| c.into()).unwrap_or_else(|| this.into());
        // 复活监听器列表（注册序，含 capture 标志；gc.rs 不持 borrow 跨 JS 回调，防再入 panic）。
        let listeners = listeners_local(scope, ffi, &event_type);
        for (cap, listener) in listeners {
            // phase 过滤（spec invoke）：capture 阶段仅 capture 监听器；target 阶段全部；bubble 阶段仅 bubble。
            let invoke = match phase {
                1 => cap,  // CAPTURING_PHASE
                2 => true, // AT_TARGET（全部，注册序）
                _ => !cap, // BUBBLING_PHASE
            };
            if !invoke {
                continue;
            }
            // R3170 spec「inner invoke」：派发期间被 removeEventListener 的监听器 skip（snapshot 仍含其
            // Local，但 map 已删 → strict_equals 身份不再存活）。典型监听器数小，O(n) 存活检查可接受。
            if !listener_present(scope, ffi, &event_type, cap, listener) {
                continue;
            }
            if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
                let _ = func.call(scope, recv, &call_args);
            }
            // stopImmediatePropagation：立即终止（当前节点剩余 + 后续节点）。
            if key_stop_imm
                .and_then(|k| event_obj.get(scope, k.into()))
                .is_some_and(|v| v.is_true())
            {
                halted = true;
                break;
            }
        }
        if halted {
            break;
        }
        // stopPropagation：当前节点监听器全尽，止后续节点（spec：止后续节点非当前剩余）。
        if key_stop
            .and_then(|k| event_obj.get(scope, k.into()))
            .is_some_and(|v| v.is_true())
        {
            halted = true;
        }
    }
    // 派发结束：currentTarget=null、eventPhase=NONE(0)（spec：派发后 currentTarget 为 null）。
    if let Some(k) = &key_ct {
        let _ = event_obj.set(scope, (*k).into(), v8::null(scope).into());
    }
    if let Some(k) = &key_phase {
        let _ = event_obj.set(scope, (*k).into(), v8::Integer::new(scope, 0).into());
    }
    // R3130 返值语义：`!(cancelable && defaultPrevented)`（spec：cancelable 事件被 preventDefault 则 false）。
    // preventDefault 由 R3127/R3129 原型方法设 defaultPrevented=true（仅 cancelable）。
    let cancelable = v8::String::new(scope, "cancelable")
        .and_then(|k| event_obj.get(scope, k.into()))
        .is_some_and(|v| v.is_true());
    let default_prevented = v8::String::new(scope, "defaultPrevented")
        .and_then(|k| event_obj.get(scope, k.into()))
        .is_some_and(|v| v.is_true());
    !(cancelable && default_prevented)
}

/// `dispatchEvent(event)`（spec `dom-eventtarget-dispatch-event`）：event 为对象读 `.type`，
/// 或直接 type 字符串（包成 `{type:str}` 对象）。三阶段派发核心抽到 [`dispatch_event_impl`]（R3147，
/// 与 `element.click()` 共用）。返值 `!(cancelable && defaultPrevented)`。
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
    let result = dispatch_event_impl(scope, target_id, this, event_obj);
    rv.set(v8::Boolean::new(scope, result).into());
}

/// `element.click()`（spec `dom-element-click`）：派发合成 `click` MouseEvent（bubbles + cancelable）
/// 到 this，经 [`dispatch_event_impl`] 三阶段派发。返 `!(cancelable && defaultPrevented)`（spec：未
/// preventDefault 时 true）。**已知限制**：① 无 activation behavior（表单提交 / 锚导航 / popover 触发
/// —— polyfill click() 经 `_zwPopoverTargetActivate` 触发 popovertarget 声明式激活，native 此切片不移植，
/// 默认行为 defer）；② 合成事件为 plain object（监听器读 `e.type==='click'` 正确；`instanceof MouseEvent`
/// 保真为已知限制）。程序化 click 高频（表单提交 / 下载链接 / 按钮激活）。
pub(super) fn native_element_click_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = _args.this();
    let Some(target_id) = read_node_id(scope, &this) else {
        return;
    };
    // 构造 click 事件对象（type/bubbles/cancelable）。plain object——监听器读 e.type==='click'（spec
    // MouseEvent instanceof 保真为已知限制）。
    let event_obj = v8::Object::new(scope);
    if let Some(k) = v8::String::new(scope, "type")
        && let Some(t) = v8::String::new(scope, "click")
    {
        let _ = event_obj.set(scope, k.into(), t.into());
    }
    if let Some(k) = v8::String::new(scope, "bubbles") {
        let _ = event_obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
    if let Some(k) = v8::String::new(scope, "cancelable") {
        let _ = event_obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
    let not_prevented = dispatch_event_impl(scope, target_id, this, event_obj);
    rv.set(v8::Boolean::new(scope, not_prevented).into());
}

// ── R3148/R3149 element.focus() / element.blur() + focusin/focusout（spec dom-element-focus/-blur）──

/// 在 `target_id` 元素上派发简单焦点事件（仅 type，不可取消），复用 [`dispatch_event_impl`] 三阶段派发
/// 核心。`bubbles` 控制是否冒泡：focus/blur **不冒泡**（spec），focusin/focusout **冒泡**（spec——
/// 焦点事件委托唯一手段，jQuery/a11y 库惯用 `document.addEventListener('focusin', ...)`）。
fn dispatch_focus_event(scope: &mut v8::PinScope, target_id: NodeId, event_type: &str, bubbles: bool) {
    let Some(this) = get_or_create_native_element(scope, target_id) else {
        return;
    };
    let event_obj = v8::Object::new(scope);
    if let Some(k) = v8::String::new(scope, "type")
        && let Some(t) = v8::String::new(scope, event_type)
    {
        let _ = event_obj.set(scope, k.into(), t.into());
    }
    if bubbles && let Some(k) = v8::String::new(scope, "bubbles") {
        let _ = event_obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
    // cancelable 缺省 false——焦点事件不可取消。
    let _ = dispatch_event_impl(scope, target_id, this, event_obj);
}

/// `element.focus()`（spec `dom-element-focus`，焦点更新步骤）：若非已聚焦——按 spec 焦点事件序列派发
/// `focusout`（旧，冒泡）→ `focusin`（this，冒泡）→ `blur`（旧，非冒泡）→ `focus`（this，非冒泡），
/// 设 `document.activeElement` = this（gc.rs `ACTIVE_ELEMENT`）。已聚焦（active==this）→ no-op（spec）。
/// R3149：补 focusin/focusout 冒泡版（闭合焦点事件模型——polyfill 旧不派发任何焦点事件）。
/// **已知限制**：不校验可聚焦性（任何元素均可 focus，同 polyfill；spec 须 focusable/tabindex）。
pub(super) fn native_element_focus_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = _args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let prev = active_element();
    if prev == Some(id) {
        return; // 已聚焦 → no-op（spec）
    }
    // spec 焦点事件序列（old→new）：focusout(old,冒泡) → focusin(new,冒泡) → blur(old) → focus(new)。
    if let Some(old) = prev {
        dispatch_focus_event(scope, old, "focusout", true); // 冒泡
    }
    dispatch_focus_event(scope, id, "focusin", true); // 冒泡
    if let Some(old) = prev {
        dispatch_focus_event(scope, old, "blur", false); // 非冒泡
    }
    set_active_element(Some(id));
    dispatch_focus_event(scope, id, "focus", false); // 非冒泡
}

/// `element.blur()`（spec `dom-element-blur`，失焦步骤）：若 this 为当前焦点——派发 `focusout`（this，
/// 冒泡）→ `blur`（this，非冒泡）+ 清 `document.activeElement`（gc.rs `ACTIVE_ELEMENT` = None）。
/// 非当前焦点 → no-op（spec）。R3149：补 focusout 冒泡版。
pub(super) fn native_element_blur_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = _args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    if active_element() != Some(id) {
        return; // 非当前焦点 → no-op
    }
    set_active_element(None);
    dispatch_focus_event(scope, id, "focusout", true); // 冒泡
    dispatch_focus_event(scope, id, "blur", false); // 非冒泡
}
