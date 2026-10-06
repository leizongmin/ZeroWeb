// slice37（NPO 收口）：Window named properties object 语义钉。named property 从
// globalThis own 数据属性迁入 WindowProperties 原型层（WebIDL §3.7.4）——链形、
// 描述符、遮蔽可见性、delete 双臂、gPN 排除、可枚举守恒、子 realm 链逐面钉死。
// Chrome 154 对照探针归档：diag/evidence/slice37/repro/（chrome-control.json）。
// https://webidl.spec.whatwg.org/#named-properties-object
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

/// slice37 钉共用沙箱装配（同 s36_sandbox!）。
macro_rules! s37_sandbox {
    ($html:expr) => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
        let dom_html = Arc::new(Mutex::new($html.to_string()));
        let page_url = Arc::new(Mutex::new("https://zero.test/s37".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①链形 + Window 构造器面：window → Window.prototype → WindowProperties(NPO) →
// EventTarget.prototype → Object.prototype；class string 双面（toString(window) =
// [object Window]、toString(NPO) = [object WindowProperties]——WebIDL §3.7.3 接口
// 对象 + §3.7.4 NPO [[PrototypeOf]] 站位）。WPT window-named-properties "Static id"
// / prototype.html "Window is not defined" 的结构根面。
#[test]
fn npo_chain_shape_s37() {
    let mut sandbox = s37_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var w37 = Object.getPrototypeOf(window);
             globalThis.__r_is_winproto = w37 === Window.prototype;
             var n37 = Object.getPrototypeOf(w37);
             globalThis.__r_npo_not_win = n37 !== window && n37 !== Object.prototype;
             globalThis.__r_npo_proto_et = Object.getPrototypeOf(n37) === EventTarget.prototype;
             globalThis.__r_et_proto_obj = Object.getPrototypeOf(EventTarget.prototype) === Object.prototype;
             globalThis.__r_win_ctor = window.constructor === Window;
             globalThis.__r_typeof_window_fn = typeof Window === 'function';
             globalThis.__r_tostr_win = Object.prototype.toString.call(window);
             globalThis.__r_tostr_npo = Object.prototype.toString.call(n37);
             var wd37 = Object.getOwnPropertyDescriptor(globalThis, 'Window');
             globalThis.__r_win_desc = wd37.writable + ',' + wd37.enumerable + ',' + wd37.configurable;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_is_winproto").unwrap().value,
        "true",
        "window 的 [[Prototype]] 站 Window.prototype（WebIDL §3.7.3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_npo_not_win").unwrap().value,
        "true",
        "Window.prototype 之下是独立 NPO（非 window/非 Object.prototype）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_npo_proto_et").unwrap().value,
        "true",
        "NPO [[Prototype]] = EventTarget.prototype（WebIDL §3.7.4 inherited proto）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_et_proto_obj")
            .unwrap()
            .value,
        "true",
        "EventTarget.prototype 之下 Object.prototype（链闭合）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_win_ctor").unwrap().value,
        "true",
        "window.constructor === Window（接口对象面）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_typeof_window_fn")
            .unwrap()
            .value,
        "true",
        "typeof Window === 'function'（prototype.html 'Window is not defined' 根修）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tostr_win").unwrap().value,
        "[object Window]",
        "class string [object Window]"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tostr_npo").unwrap().value,
        "[object WindowProperties]",
        "NPO class string [object WindowProperties]（WebIDL §3.7.4）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_win_desc").unwrap().value,
        "true,false,true",
        "globalThis.Window 描述符 w/e/c = true/false/false+true（Chrome 同款）"
    );
}

// ②NPO 描述符 + gPN 排除：named prop 在 NPO 层 gsp = {w:true,e:false,c:true}、
// hasOwnProperty 真 而 getOwnPropertyNames 不含（值经 trap 合成，不落 target own）。
// WPT window-named-properties "Static id"/"duplicate property names" 取值面 + Chrome
// 探针 npoGpnHasBar=false 面。
#[test]
fn npo_descriptor_gpn_s37() {
    let mut sandbox = s37_sandbox!("<html><body><img name='np37' id='npi37'></body></html>");
    sandbox
        .execute(
            "var gsp37 = Object.getPrototypeOf(Object.getPrototypeOf(window));
             globalThis.__r_hasown = gsp37.hasOwnProperty('np37');
             var d37 = Object.getOwnPropertyDescriptor(gsp37, 'np37');
             globalThis.__r_desc = d37 === undefined ? 'none'
               : (d37.writable + ',' + d37.enumerable + ',' + d37.configurable);
             globalThis.__r_desc_val_img = d37 !== undefined && d37.value === document.getElementsByName('np37')[0];
             globalThis.__r_gpn_absent = Object.getOwnPropertyNames(gsp37).indexOf('np37') === -1;
             globalThis.__r_in_window = 'np37' in window;
             globalThis.__r_win_get = window.np37 === document.getElementsByName('np37')[0];
             var tt37 = Object.getOwnPropertyDescriptor(gsp37, Symbol.toStringTag);
             globalThis.__r_ttag = tt37 !== undefined && tt37.value === 'WindowProperties';")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_hasown").unwrap().value,
        "true",
        "NPO.hasOwnProperty(named) 真（own 观察面，Chrome 同款）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_desc").unwrap().value,
        "true,false,true",
        "NPO gsp 描述符 = writable:true / enumerable:false / configurable:true"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_desc_val_img")
            .unwrap()
            .value,
        "true",
        "NPO gsp value = 命中元素本体"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_gpn_absent")
            .unwrap()
            .value,
        "true",
        "getOwnPropertyNames(NPO) 不含 named prop（Chrome 同款排除面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_in_window").unwrap().value,
        "true",
        "'np37' in window 经链可见"
    );
    assert_eq!(sandbox.execute("globalThis.__r_win_get").unwrap().value, "true", "window.np37 取值 = 元素本体");
    assert_eq!(
        sandbox.execute("globalThis.__r_ttag").unwrap().value,
        "true",
        "NPO Symbol.toStringTag gsp = 'WindowProperties'"
    );
}

// ③遮蔽可见性（Chrome 实测裁定）：仅 NPO 之上（EventTarget.prototype →
// Object.prototype）own 属性隐名；window own expando 不隐、Window.prototype own
// 不隐（本面复测 expando 臂 + ET 臂 + 'constructor' 名臂——WPT prototype.html
// test 1/3/4 与 window-named-properties 'constructor' 面的钉化）。
#[test]
fn npo_shadow_visibility_s37() {
    let mut sandbox = s37_sandbox!(
        "<html><body><img name='sh37'><img name='hider37'><iframe name='constructor'></iframe></body></html>"
    );
    sandbox
        .execute(
            "var img37 = document.getElementsByName('sh37')[0];
             window.sh37 = 'expando';
             globalThis.__r_expando_wins = window.sh37 === 'expando';
             var gsp37 = Object.getPrototypeOf(Object.getPrototypeOf(window));
             globalThis.__r_npo_alive = Object.getOwnPropertyDescriptor(gsp37, 'sh37').value === img37;
             delete window.sh37;
             globalThis.__r_npo_back = window.sh37 === img37;
             Object.defineProperty(EventTarget.prototype, 'hider37',
               { value: 'et', writable: true, enumerable: false, configurable: true });
             globalThis.__r_et_hides = window.hider37 === 'et';
             globalThis.__r_et_gsp_none = Object.getOwnPropertyDescriptor(gsp37, 'hider37') === undefined;
             delete EventTarget.prototype.hider37;
             globalThis.__r_et_unhide = window.hider37 === document.getElementsByName('hider37')[0];
             globalThis.__r_ctor_not_img = window.constructor !== document.getElementsByName('constructor')[0]
               && window.constructor === Window;
             globalThis.__r_ctor_gsp_none = Object.getOwnPropertyDescriptor(gsp37, 'constructor') === undefined;")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r_expando_wins")
            .unwrap()
            .value,
        "true",
        "window own expando 赋值胜出（ordinary own-first，Chrome 同款）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_npo_alive").unwrap().value,
        "true",
        "own expando 不隐 NPO 属性（prototype.html test 1 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_npo_back").unwrap().value,
        "true",
        "own expando 删除后回落 NPO 值"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_et_hides").unwrap().value,
        "true",
        "EventTarget.prototype own 隐同名 named prop（prototype.html test 3 面）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_et_gsp_none")
            .unwrap()
            .value,
        "true",
        "隐名时 NPO gsp = undefined（可见性算法 2 步）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_et_unhide")
            .unwrap()
            .value,
        "true",
        "ET own 移除后 named prop 复现（prototype.html test 4 面）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_ctor_not_img")
            .unwrap()
            .value,
        "true",
        "'constructor' 名被 ET.prototype.constructor own 隐（window.constructor = Window）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_ctor_gsp_none")
            .unwrap()
            .value,
        "true",
        "隐名 'constructor' 在 NPO 层 gsp = undefined（Chrome arb ctor.npoOwn=false）"
    );
}

// ④delete 双臂（WebIDL §3.7.4.3 [[Delete]]→false 的 JS 观察面）+ window 级复活
//（slice36 FIXME ⑥ 收口）：NPO 级 sloppy ret=false / strict TypeError（Chrome 探针
// s4 同款）；window 级 delete 不命中 own（named prop 已不在 globalThis own）→ 再读
// 经 NPO 复活（Chrome arb delWin 复活面；ret 值偏差=经链传播 false，manifest 申报）。
#[test]
fn npo_delete_faces_s37() {
    let mut sandbox = s37_sandbox!("<html><body><img name='del37'></body></html>");
    sandbox
        .execute(
            "var img37 = document.getElementsByName('del37')[0];
             var gsp37 = Object.getPrototypeOf(Object.getPrototypeOf(window));
             globalThis.__r_npo_sloppy = delete gsp37.del37;
             globalThis.__r_strict_threw = false;
             try { (function () { 'use strict'; delete gsp37.del37; })(); }
             catch (_e37) { globalThis.__r_strict_threw = true; }
             delete window.del37;
             globalThis.__r_win_resurrect = window.del37 === img37;
             globalThis.__r_own_absent = !Object.prototype.hasOwnProperty.call(globalThis, 'del37');")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r_npo_sloppy")
            .unwrap()
            .value,
        "false",
        "NPO 级 sloppy delete ret=false（WebIDL [[Delete]]→false）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_strict_threw")
            .unwrap()
            .value,
        "true",
        "NPO 级 strict delete TypeError（Chrome 探针 s4 同款）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_win_resurrect")
            .unwrap()
            .value,
        "true",
        "window 级 delete 后再读复活（FIXME ⑥ 收口，Chrome 同款）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_own_absent")
            .unwrap()
            .value,
        "true",
        "named prop 不在 globalThis own（NPO 化本量——反『own 安装面』回归）"
    );
}

// ⑤可枚举守恒：named prop 与 NPO 化新增全局（__zwNPO/__zwNAGet/__zwNADelete/
// __zwNAOwnKeys）均不入 for-in(window)/Object.keys(window)；ET.prototype 三方法
// 非可枚举（NPO 接线前重定义，for-in(window) 747 基线守恒的结构面）。
#[test]
fn npo_enumerable_faces_s37() {
    let mut sandbox = s37_sandbox!("<html><body><img name='en37'></body></html>");
    sandbox
        .execute(
            "var bad37 = [];
             for (var k37 in window) {
               if (k37 === 'en37' || k37 === '__zwNPO' || k37 === '__zwNAGet'
                 || k37 === '__zwNADelete' || k37 === '__zwNAOwnKeys') bad37.push(k37);
             }
             globalThis.__r_forin_bad = bad37.join(',');
             globalThis.__r_keys_absent = Object.keys(window).indexOf('en37') === -1;
             var et37 = ['addEventListener', 'removeEventListener', 'dispatchEvent'];
             var etBad37 = [];
             for (var i37 = 0; i37 < et37.length; i37++) {
               var ed37 = Object.getOwnPropertyDescriptor(EventTarget.prototype, et37[i37]);
               if (!ed37 || ed37.enumerable) etBad37.push(et37[i37]);
             }
             globalThis.__r_et_nonenum = etBad37.join(',');")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r_forin_bad")
            .unwrap()
            .value,
        "",
        "for-in(window) 不暴露 named prop 与 NPO 内部全局（可枚举守恒）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_keys_absent")
            .unwrap()
            .value,
        "true",
        "Object.keys(window) 不含 named prop（e:false）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_et_nonenum")
            .unwrap()
            .value,
        "",
        "EventTarget.prototype 三方法 enumerable:false（接线前重定义面）"
    );
}

// ⑥子 realm 链（cross-global 面）：iframe contentWindow → childWinProto（own
// constructor=Object）→ childNPO（class string WindowProperties）→ EventTarget.prototype
//（共享主 realm）→ Object.prototype——cross-global-npo.html 五层走查的结构钉。
#[test]
fn npo_child_realm_chain_s37() {
    let mut sandbox = s37_sandbox!("<html><body><iframe name='cw37'></iframe></body></html>");
    sandbox
        .execute(
            "var cw37 = document.getElementsByTagName('iframe')[0].contentWindow;
             globalThis.__r_main_sees = window.cw37 === cw37;
             var p1 = Object.getPrototypeOf(cw37);
             globalThis.__r_child_ctor = p1.constructor === Object;
             var p2 = Object.getPrototypeOf(p1);
             globalThis.__r_child_npo_str = Object.prototype.toString.call(p2);
             globalThis.__r_child_npo_et = Object.getPrototypeOf(p2) === EventTarget.prototype;
             globalThis.__r_child_npo_not_main = p2 !== Object.getPrototypeOf(Object.getPrototypeOf(window));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_main_sees").unwrap().value,
        "true",
        "主 window 经 NPO live 扫描见 iframe 名（值=contentWindow）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_child_ctor")
            .unwrap()
            .value,
        "true",
        "child window [[Prototype]] own constructor = Object（cross-global 口径）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_child_npo_str")
            .unwrap()
            .value,
        "[object WindowProperties]",
        "child NPO class string（每子 realm 独立 NPO）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_child_npo_et")
            .unwrap()
            .value,
        "true",
        "child NPO [[Prototype]] = EventTarget.prototype（共享主 realm）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r_child_npo_not_main")
            .unwrap()
            .value,
        "true",
        "child NPO ≠ 主 NPO（per-realm 实例）"
    );
}
