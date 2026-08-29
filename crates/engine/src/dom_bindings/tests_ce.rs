//! P1b 原生 DOM 绑定测试——customElements lifecycle 派发（R362 覆盖率提升切片；
//! custom_elements.rs 89.0% → 90%+）。
//!
//! 覆盖：notify_connect_after_insert（connect/disconnect 状态真转两臂 + fast-path 短路）/
//! notify_disconnect_after_remove（已连子树移除断开派发）/ collect_custom_subtree（多层
//! 子树 pre-order）/ element_tag 非 Element 臂 / notify_attribute_change 与
//! read_attr_change_context 的守卫臂（非元素 / 无连字符 fast-path / polyfill hook 缺失
//! 静默）。共享 [`run_script`]（tests.rs，pub(super)）。

use super::tests::run_script;

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
var afterConnect = __ceCalls.join(',');
// 嵌套子树：custom 内嵌 custom（collect_custom_subtree 多层 pre-order）。
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
var afterDisconnect = __ceCalls.join('|');
afterConnect + '|' + afterDisconnect;
"#;
    let out = run_script(r#"<html><body></body></html>"#, script);
    // 记录行为（R362 观测）：嵌套 insert（ce.appendChild(inner)）触发父子双 connect——
    // my-el 的重复 connect 是 **registry 簿记 finding**（spec：每连接态真转恰一次；
    // mark/unmark 的 is_custom_connected 检查疑似未覆盖嵌套 append 路径），转 CE 专项
    // 记档；本测试锚定现状供回归对照。
    assert_eq!(
        out,
        "connect:my-el|connect:my-el|connect:my-inner|disconnect:my-el|disconnect:my-inner|connect:my-el|disconnect:my-el",
        "R362 CE lifecycle：connect 派发 + 嵌套子树逐层 disconnect（含重复 connect finding 锚定）"
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
