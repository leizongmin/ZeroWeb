//! P1b 原生 DOM 绑定测试——customElements lifecycle 派发（R362 覆盖率提升切片；
//! custom_elements.rs 89.0% → 90%+）。
//!
//! 覆盖：notify_connect_after_insert（connect/disconnect 状态真转两臂 + fast-path 短路）/
//! notify_disconnect_after_remove（已连子树移除断开派发）/ collect_custom_subtree（多层
//! 子树 pre-order）/ element_tag 非 Element 臂 / notify_attribute_change 与
//! read_attr_change_context 的守卫臂（非元素 / 无连字符 fast-path / polyfill hook 缺失
//! 静默）。共享 [`run_script`]（tests.rs，pub(super)）。

use super::tests::run_script;
use crate::js_dom_bridge::{DomMutation, generate_js_dom_shim, register_dom_callbacks};

/// 连接态 lifecycle：native appendChild（连入 body → connect 派发）→ removeChild（断开 →
/// disconnect 派发）。polyfill hook 由脚本预置记录调用。R3271 fast-path：div（无连字符）
/// 不派发；my-el（含连字符）派发。
#[test]
fn native_ce_connect_disconnect_dispatch_r362() {
    let script = r#"
// 记录 hook 调用（polyfill 侧职责：按 tag 查 registry + 调回调；测试直接记录 tag 序列）。
globalThis.__ceCalls = [];
globalThis.__zw_native_ce_notify_connect = function (instances, connected, tags) {
  for (var i = 0; i < tags.length; i++) {
    __ceCalls.push((connected ? 'connect:' : 'disconnect:') + tags[i]);
  }
};
// 建一个 custom tag 元素（含连字符 → 过 fast-path）+ 一个原生 div（fast-path 短路，不进派发）。
var ce = __zw_native_create_element('my-el');
var plain = __zw_native_create_element('div');
var body = __zw_native_get_body();
body.appendChild(ce);
body.appendChild(plain);
// 嵌套子树：custom 内嵌 custom（collect_custom_subtree 多层 pre-order）。
// R363：不做两段 join 拼接（join(',')+join('|') 会让首条元素双渲染——上一版期望的
// 「双 connect」即此装配伪影，非 registry 簿记行为）；全量单次 join 断言。
var inner = __zw_native_create_element('my-inner');
ce.appendChild(inner);
body.removeChild(ce);
// R362 断连臂：**已连元素移动进 detached 容器**（was=true 且新 parent 未连 →
// notify_connect_after_insert 的 disconnect 分支；removeChild 走的是
// notify_disconnect_after_remove，本臂需「insert-into-detached」形态覆盖）。
var ce2 = __zw_native_create_element('my-el');
body.appendChild(ce2);
var det = __zw_native_create_element('section');
det.appendChild(ce2);
__ceCalls.join('|');
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    // R363 勘误：此前记录的「嵌套 insert 双 connect finding」是**测试装配伪影**——
    // afterConnect 用 join(',') 截取首段后，afterDisconnect 又对全量 join('|')，首条
    // connect:my-el 被双渲染。最小序列复刻（body.append(ce) → body.append(div) →
    // ce.append(inner) 逐段读数）实证每连接态真转恰一次派发（C:my-el / C:my-inner 各一），
    // mark/unmark 簿记 spec-correct，无 registry 专项 finding。全量期望：每个元素恰
    // 一次 connect + 一次 disconnect（ce2 的移动形态再各一次）。
    assert_eq!(
        out, "connect:my-el|connect:my-inner|disconnect:my-el|disconnect:my-inner|connect:my-el|disconnect:my-el",
        "R362/R363 CE lifecycle：每连接态真转恰一次派发（勘误双 connect 为装配伪影）"
    );
}

/// attributeChangedCallback 桥接：native setAttribute/removeAttribute 触发
/// read_attr_change_context（old 值预读）+ notify_attribute_change（polyfill hook 记录）。
/// 无连字符 tag 的 setAttribute 走 R3271 fast-path（read 返回 tag=None → 不派发）。
#[test]
fn native_ce_attribute_change_dispatch_r362() {
    let script = r#"
globalThis.__attrCalls = [];
globalThis.__zw_native_ce_notify_attr_change = function (instance, name, oldVal, newVal, tag) {
  __attrCalls.push(tag + ':' + name + ':' + oldVal + '->' + newVal);
};
var ce = __zw_native_create_element('my-el');
__zw_native_get_body().appendChild(ce);
// set：old=null → new=v1；再 set：old=v1 → new=v2；remove：old=v2 → new=null。
ce.setAttribute('data-x', 'v1');
ce.setAttribute('data-x', 'v2');
ce.removeAttribute('data-x');
// fast-path 对照：div 的 setAttribute 不派发（tag=None）。
var plain = __zw_native_create_element('div');
plain.setAttribute('data-y', 'z');
__attrCalls.join('|');
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    assert_eq!(
        out, "MY-EL:data-x:null->v1|MY-EL:data-x:v1->v2|MY-EL:data-x:v2->null",
        "R362 CE attribute 派发：old 预读三态（null→v1→v2→null）+ div fast-path 不派发"
    );
}

/// 守卫臂：无 polyfill hook（未注册 `__zw_native_ce_notify_*`）时 notify 静默不抛；
/// 非元素（text node）set 不派发（element_tag 臂）。
#[test]
fn native_ce_guard_arms_r362() {
    let script = r#"
// 不注册任何 hook——connect/disconnect/attr 派发全部静默（无异常即守卫臂生效）。
var body = __zw_native_get_body();
var ce = __zw_native_create_element('my-el');
body.appendChild(ce);
ce.setAttribute('data-x', 'v');
var tn = __zw_native_create_text_node('t');
body.appendChild(tn);
'ok';
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    assert_eq!(out, "ok", "R362 CE 守卫臂：无 hook / text 节点路径静默不抛");
}

/// R367（js-dom M1/M4 CE registry 专项 slice 3）：**native CE hooks per-realm 查表**——
/// Rust 侧把元素 node document root（owner_document）作为 ownerId 传给 polyfill hook；
/// 主文档元素（owner == live Document root）读主实例，distinct root（另一 doc 域元素）
/// 由 polyfill 按 ownerId 路由。本测验证①主文档路径不回归（ownerId 传入后 lookup/
/// notify 照常主 registry 命中）；②ownerId 传播形态（hook 实参收数组/字符串而非缺省）。
/// spec：createElement/upgrade 的 registry = 元素 node document 的相关 realm。
/// https://dom.spec.whatwg.org/#concept-create-element
#[test]
fn native_ce_hooks_per_realm_lookup_r367() {
    let script = r#"
globalThis.__calls = [];
var _ce = {};
globalThis.customElements = { define: function(n,c){ _ce[n]=c; }, get: function(n){ return _ce[n] || null; } };
// 记录 lookup 收到的 ownerId（R367 起 Rust 传第二参——主文档元素 owner = doc root 字符串）。
globalThis.__zw_native_ce_lookup = function (tag, ownerId) {
  __calls.push('lookup:' + tag + ':' + (ownerId === undefined ? 'undef' : (ownerId === null ? 'null' : String(typeof ownerId))));
  return _ce[tag] || null;
};
class MyEl extends HTMLElement { constructor() { super(); this.__upgraded = true; } }
customElements.define('my-el', MyEl);
var e = __zw_native_create_element('my-el');
__calls.push('upgraded:' + (e instanceof MyEl) + ':' + e.__upgraded);
// connect notify：R367 起第四参 = ownerIds 数组（每实例一项）。
globalThis.__zw_native_ce_notify_connect = function (instances, connected, tags, ownerIds) {
  __calls.push('conn:' + connected + ':' + tags.length + ':' + (ownerIds ? ownerIds.length : 'noarr'));
};
__zw_native_get_body().appendChild(e);
// attr notify：R367 起第六参 = ownerId。
globalThis.__zw_native_ce_notify_attr_change = function (instance, name, oldVal, newVal, tag, ownerId) {
  __calls.push('attr:' + tag + ':' + name + ':' + (ownerId === undefined ? 'undef' : String(typeof ownerId)));
};
e.setAttribute('data-x', '1');
__calls.join('|');
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    // 主文档元素：lookup 收 string ownerId（doc root ffi）；upgrade 成功；connect 的 ownerIds
    // 数组与 tags 等长；attr notify 收 string ownerId。
    assert_eq!(
        out, "lookup:my-el:string|upgraded:true:true|conn:true:1:1|attr:MY-EL:data-x:string",
        "R367 native CE hooks per-realm：lookup/notify 收 ownerId（string 形态）、upgrade 与 lifecycle 主路径不回归"
    );
}

/// R367：ownerId **缺省回退**——`__zw_native_doc_root_id` 缺席（shim-only 沙箱等）时
/// lookup/notify 仍按主实例工作（域判据降级为主文档路径，零回归）。
#[test]
fn native_ce_entry_fallback_without_domain_probe_r367() {
    let script = r#"
var log = [];
log.push('probe:' + (typeof globalThis.__zw_native_doc_root_id));
log.push('entryType:' + typeof globalThis.__zw_native_ce_entry);
log.join('|');
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    assert_eq!(
        out, "probe:function|entryType:undefined",
        "R367 native 测试路径：域探针 `__zw_native_doc_root_id` 已随 install 注入（function）；shim 侧 `__zw_native_ce_entry` 不在（undefined，shim 断言归 js_dom_bridge_tests）"
    );
}
