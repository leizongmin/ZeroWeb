// js_dom_bridge 测试切片 59（R3909）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-58 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

// ── R3909：selectedness 真态写侧（html-semantics M2 切片 9）──
//
// 生产 shim 路径：part03 R3909 助手块（`_zwSelectednessSetting` 互斥最后者胜 + 无选中
// display size 1 回落首个非 disabled；`_zwAskForReset` 最近祖先 select 上跑；
// `_zwSelectResetOnMutation` childList 汇流触发；`_zwOptSelAttrPre/Post` selected 属性
// 变更步；`_zwSetSelectSelectedIndex` set-the-selected-index）+ part05 selected setter
// 尾跑 ask-for-reset + selectedIndex setter 分支 + part02 Option 构造器 clean 层 +
// part04 form.reset select 复位算法 / select.remove(index) / setAttribute/removeAttribute
// 属性钩 + part01 `_optSelectedClean` 声明与 `_mo_notify` 触发。
// spec https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element 。
//
// 注意（与旧 R2999 属性镜像语义的差异）：display size 1 且 dirtiness false 时 selectedness
// setting 的回落会重新选中首个非 disabled option——对唯一 option 而言 removeAttribute('selected')
// / `o.selected = false` 之后 `.selected` 仍为 true，这是 spec 行为而非 stale 缓存。

// W1+W2：ask-for-a-reset——selected setter 互斥最后者胜；回落跳过 disabled。
#[test]
fn test_r3909_ask_for_reset_exclusivity() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='a9'><option id='a90'></option><option id='a91'></option><option id='a92'></option></select>\
         <select id='b9'><option id='b90' disabled></option><option id='b91'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var a9 = document.getElementById('a9');
var a90 = document.getElementById('a90');
var a91 = document.getElementById('a91');
var a92 = document.getElementById('a92');
// W1：setter 后互斥——受影响 option（setter 目标）绝对胜出，先前选中被清。
a92.selected = true;
globalThis.__w1a = String(a90.selected) + '/' + String(a91.selected) + '/' + String(a92.selected) + '/' + String(a9.selectedIndex);
a91.selected = true;
globalThis.__w1b = String(a90.selected) + '/' + String(a91.selected) + '/' + String(a92.selected) + '/' + String(a9.selectedIndex);
// W2：唯一显式选中被反选后，回落跳过 disabled 命中 index 1。
var b9 = document.getElementById('b9');
var b91 = document.getElementById('b91');
b91.selected = true;
b91.selected = false;
globalThis.__w2a = String(b9.selectedIndex) + '/' + String(b91.selected);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1a").unwrap().value,
        "false/false/true/2",
        "W1：a92.selected=true 后互斥——a90/a91 false，selectedIndex 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1b").unwrap().value,
        "false/true/false/1",
        "W1：再设 a91（列表序早于 a92）——受影响者绝对胜出，a92 被清，selectedIndex 1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2a").unwrap().value,
        "1/true",
        "W2：回落跳过 disabled——selectedIndex 1 且 b91.selected=true"
    );
}

// W3+W4：childList 汇流触发——select.remove(0) 后 size 2 无回落返 -1；插入带 selected
// 属性的 option 跑 selectedness setting（最后者胜）。
#[test]
fn test_r3909_mutation_triggers() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='d9' size='2'><option id='d90'></option><option id='d91'></option></select>\
         <select id='e9'><option id='e90'></option><option id='e91'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var d9 = document.getElementById('d9');
var d90 = document.getElementById('d90');
d90.selected = true;
globalThis.__w3pre = String(d9.selectedIndex);
d9.remove(0);
globalThis.__w3post = String(d9.selectedIndex);
var e9 = document.getElementById('e9');
var n1 = document.createElement('option');
n1.setAttribute('selected', '');
n1.setAttribute('value', 'N1');
e9.appendChild(n1);
globalThis.__w4a = String(e9.selectedIndex) + '/' + String(e9.options.length);
var n2 = document.createElement('option');
n2.setAttribute('selected', '');
n2.setAttribute('value', 'N2');
e9.appendChild(n2);
globalThis.__w4b = String(e9.selectedIndex) + '/' + e9.value;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w3pre").unwrap().value,
        "0",
        "W3 前置：d90.selected=true 后 selectedIndex 0"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3post").unwrap().value,
        "-1",
        "W3：remove(0) 触发 selectedness setting，size 2 无回落 → -1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4a").unwrap().value,
        "2/3",
        "W4：插入带 selected 的 option 触发 setting——新 option 胜出，selectedIndex 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4b").unwrap().value,
        "3/N2",
        "W4：再次插入——列表序最后者胜，selectedIndex 3 / value N2"
    );
}

// W5：set-the-selected-index——全清 + 命中 dirty；-1 后无选中（spec：即使 size 1 也不回落，
// 不跑 ask-for-reset）；options.selectedIndex setter 同路径；越界无命中。
#[test]
fn test_r3909_selected_index_setter() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='g9'><option id='g90' value='V0'></option><option id='g91' value='V1'></option><option id='g92' value='V2'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var g9 = document.getElementById('g9');
g9.selectedIndex = 1;
globalThis.__w5a = String(g9.selectedIndex) + '/' + g9.value + '/' + String(document.getElementById('g91').selected);
g9.selectedIndex = -1;
globalThis.__w5b = String(g9.selectedIndex) + '/' + String(g9.value === '');
g9.options.selectedIndex = 2;
globalThis.__w5c = String(g9.selectedIndex) + '/' + g9.value;
g9.selectedIndex = 99;
globalThis.__w5d = String(g9.selectedIndex);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w5a").unwrap().value,
        "1/V1/true",
        "W5：selectedIndex=1 命中——value V1，g91.selected true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5b").unwrap().value,
        "-1/true",
        "W5：selectedIndex=-1——不回落，无选中且 value 为空串"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5c").unwrap().value,
        "2/V2",
        "W5：options.selectedIndex setter 同路径命中 index 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5d").unwrap().value,
        "-1",
        "W5：越界 99——全清无命中，selectedIndex -1"
    );
}

// W6：form.reset() select 复位算法——selectedness 按-selected-属性重置 + dirtiness 清零。
#[test]
fn test_r3909_form_reset_restore() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <form id='f9b'>\
         <input id='hin' value='X'>\
         <select id='h9'><option id='h90'></option><option id='h91' selected></option></select>\
         </form>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h9 = document.getElementById('h9');
var h90 = document.getElementById('h90');
var h91 = document.getElementById('h91');
h90.selected = true;
globalThis.__w6a = String(h9.selectedIndex);
var hin = document.getElementById('hin');
hin.value = 'Y';
document.getElementById('f9b').reset();
globalThis.__w6b = String(h9.selectedIndex) + '/' + String(h90.selected) + '/' + String(h91.selected);
globalThis.__w6c = String(hin.value);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w6a").unwrap().value,
        "0",
        "W6 前置：h90.selected=true 覆盖默认选中"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w6b").unwrap().value,
        "1/false/true",
        "W6：reset 复位算法——回到 selected 属性持有者 h91"
    );
}

// W7：Option() 构造器——≤3 参恒 selectedness false（defaultSelected 只落 selected 属性），
// 第 4 参落显式初始 selectedness；dirtiness 不置位 → detached 下 selected 属性增删仍生效。
#[test]
fn test_r3909_option_constructor() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var c0 = new Option();
globalThis.__w7a = String(c0.selected) + '/' + String(c0.hasAttribute('selected')) + '/' + String(c0.value === '') + '/' + String(c0.text === '');
var c1 = new Option('T', 'V', true);
globalThis.__w7b = String(c1.selected) + '/' + String(c1.hasAttribute('selected'));
var c2 = new Option('T', 'V', false, true);
globalThis.__w7c = String(c2.selected) + '/' + String(c2.hasAttribute('selected'));
var c3 = new Option('T', 'V', true, false);
globalThis.__w7d = String(c3.selected) + '/' + String(c3.hasAttribute('selected'));
c2.setAttribute('selected', '');
globalThis.__w7e = String(c2.selected);
c2.removeAttribute('selected');
globalThis.__w7f = String(c2.selected);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w7a").unwrap().value,
        "false/false/true/true",
        "W7：无参构造——selectedness false，text/value 均空串"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7b").unwrap().value,
        "false/true",
        "W7：第 3 参 truthy——selected 属性落位但 selectedness 恒 false（≤3 参）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7c").unwrap().value,
        "true/false",
        "W7：第 4 参 truthy——显式初始 selectedness true 且无属性"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7d").unwrap().value,
        "false/true",
        "W7：第 4 参 falsy——selectedness false 且属性在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7e").unwrap().value,
        "true",
        "W7：构造后 setAttribute 反映（detached 无回落）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7f").unwrap().value,
        "false",
        "W7：构造后 removeAttribute 反映（dirtiness false，属性变更步生效）"
    );
}

// W8：selected 内容属性变更步（连接态）——setAttribute 落 clean 层并触发 setting；
// removeAttribute 后 size 1 回落重新选中首个 option（spec 行为，R3909 升级）。
#[test]
fn test_r3909_selected_attr_changed_steps() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='i9'><option id='i90' value='I0'></option><option id='i91' value='I1'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var i9 = document.getElementById('i9');
var i90 = document.getElementById('i90');
var i91 = document.getElementById('i91');
i90.setAttribute('selected', '');
globalThis.__w8a = String(i90.selected) + '/' + String(i9.selectedIndex) + '/' + i9.value;
i90.removeAttribute('selected');
globalThis.__w8b = String(i90.selected) + '/' + String(i9.selectedIndex) + '/' + i9.value;
i91.selected = true;
globalThis.__w8c = String(i90.selected) + '/' + String(i9.selectedIndex) + '/' + i9.value;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w8a").unwrap().value,
        "true/0/I0",
        "W8：setAttribute('selected') 变更步——clean 层落位 selectedIndex 0"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8b").unwrap().value,
        "true/0/I0",
        "W8：removeAttribute 后回落重新选中 i90（size 1 spec 行为，非 stale）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8c").unwrap().value,
        "false/1/I1",
        "W8：随后 i91.selected=true 互斥清掉回落态"
    );
}

// W9：select.add 语义——尾插/前插、祖先链 HierarchyRequestError；options 集合 has/gOPD 陷阱。
#[test]
fn test_r3909_select_add_and_traps() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='j9'><option id='j90' value='J0'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var j9 = document.getElementById('j9');
var jm = new Option('M', '1');
j9.add(jm);
globalThis.__w9a = String(j9.options.length) + '/' + j9.options[1].value;
var jn = new Option('N', '0');
j9.add(jn, j9.options[0]);
globalThis.__w9b = String(j9.options.length) + '/' + j9.options[0].value + '/' + j9.options[1].value;
var anc = document.createElement('option');
var inner = document.createElement('select');
anc.appendChild(inner);
globalThis.__w9c = 'none';
try {
  inner.add(anc);
} catch (e) {
  globalThis.__w9c = e.name;
}
globalThis.__w9d = String(1 in j9.options) + '/' + String(99 in j9.options) + '/' + String('length' in j9.options);
var d0 = Object.getOwnPropertyDescriptor(j9.options, 0);
globalThis.__w9e = d0.value.value + '/' + String(d0.configurable);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w9a").unwrap().value,
        "2/1",
        "W9：add 尾插——length 2，新 option value 1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w9b").unwrap().value,
        "3/0/J0",
        "W9：add before 引用——插到 index 0，原首位移到 index 1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w9c").unwrap().value,
        "HierarchyRequestError",
        "W9：add(select 自身祖先 option) 抛 HierarchyRequestError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w9d").unwrap().value,
        "true/false/true",
        "W9：options has 陷阱——规范索引命中、越界与 length 正确"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w9e").unwrap().value,
        "0/true",
        "W9：options gOPD 陷阱——索引 0 描述符含 option（前插后的 jn，value 0）且 configurable"
    );
}

// W11：select value-missing（HTML §4.10.7 constraint validation）——required 且无
// selectedness true 者或唯一 selected 即 placeholder label option（required +
// display size 1 + list 首项 value 空串 + 无 optgroup 祖先）。
#[test]
fn test_r3909_select_value_missing() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// m1：required size1——首项空串（placeholder）选中 → missing；非占位空 option 选中 → valid。
var m1 = document.createElement('select');
m1.required = true;
var p1 = document.createElement('option');
m1.appendChild(p1);
document.body.appendChild(m1);
var bad1 = String(m1.checkValidity());
var e1 = document.createElement('option');
m1.appendChild(e1);
e1.selected = true;
globalThis.__v1 = bad1 + '/' + String(m1.checkValidity());
// m2：首项在 optgroup 内 → 非 placeholder → required + 空 selected 仍 valid。
var m2 = document.createElement('select');
m2.required = true;
var og2 = document.createElement('optgroup');
var o2 = document.createElement('option');
og2.appendChild(o2);
m2.appendChild(og2);
o2.selected = true;
document.body.appendChild(m2);
globalThis.__v2 = String(m2.checkValidity());
// m3：size 2——无选中 missing，有选中 valid。
var m3 = document.createElement('select');
m3.required = true;
m3.size = 2;
var o3 = document.createElement('option');
m3.appendChild(o3);
document.body.appendChild(m3);
var bad3 = String(m3.checkValidity());
o3.selected = true;
var good3 = String(m3.checkValidity());
globalThis.__v3 = bad3 + '/' + good3;
// m4：首项 value 非空 → 非 placeholder → 选中即 valid（disabled 不改判定）。
var m4 = document.createElement('select');
m4.required = true;
var o4 = document.createElement('option');
o4.value = 'test';
o4.disabled = true;
o4.selected = true;
m4.appendChild(o4);
document.body.appendChild(m4);
globalThis.__v4 = String(m4.checkValidity());
// m5：multiple required——无选中 missing，有选中 valid。
var m5 = document.createElement('select');
m5.required = true;
m5.multiple = true;
var o5 = document.createElement('option');
m5.appendChild(o5);
document.body.appendChild(m5);
var bad5 = String(m5.checkValidity());
o5.selected = true;
globalThis.__v5 = bad5 + '/' + String(m5.checkValidity());
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v1").unwrap().value,
        "false/true",
        "W11：placeholder 选中 missing；非占位空 option 选中 valid"
    );
    assert_eq!(
        sandbox.execute("globalThis.__v2").unwrap().value,
        "true",
        "W11：首项在 optgroup 内非 placeholder——valid"
    );
    assert_eq!(
        sandbox.execute("globalThis.__v3").unwrap().value,
        "false/true",
        "W11：size 2 无选中 missing、有选中 valid"
    );
    assert_eq!(
        sandbox.execute("globalThis.__v4").unwrap().value,
        "true",
        "W11：首项 value 非空非 placeholder（disabled 无关）——valid"
    );
    assert_eq!(
        sandbox.execute("globalThis.__v5").unwrap().value,
        "false/true",
        "W11：multiple 无选中 missing、有选中 valid"
    );
}

// W12：插入重回落（Blink 行为）——非 multiple + size 1 + 新首项插入且当前仅 clean 层
// 回落选中 → 回落态让位重选新首项；dirty-selected 保持。
#[test]
fn test_r3909_insert_refallback() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// w12a：clean 回落选中 + 前插空 option（required）→ 新首项被选中（Chromium T6c）。
var wa = document.createElement('select');
wa.required = true;
var waA = document.createElement('option');
waA.textContent = 'A';
wa.appendChild(waA);
document.body.appendChild(wa);
var waP = document.createElement('option');
wa.prepend(waP);
globalThis.__w12a = String(wa.selectedIndex) + '/' + String(waA.selected);
// w12b：dirty-selected + 前插空 option → 保持（Chromium V1）。
var wb = document.createElement('select');
wb.required = true;
var wbB = document.createElement('option');
wbB.textContent = 'B';
wb.appendChild(wbB);
document.body.appendChild(wb);
wbB.selected = true;
var wbP = document.createElement('option');
wb.prepend(wbP);
globalThis.__w12b = String(wb.selectedIndex) + '/' + String(wbB.selected);
// w12c：尾插 → 不触发重回落。
var wc = document.createElement('select');
wc.required = true;
var wcC = document.createElement('option');
wcC.textContent = 'C';
wc.appendChild(wcC);
document.body.appendChild(wc);
wcC.selected = true;
var wcE = document.createElement('option');
wc.appendChild(wcE);
globalThis.__w12c = String(wc.selectedIndex) + '/' + String(wcE.selected);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w12a").unwrap().value,
        "0/false",
        "W12：clean 回落态让位——新首项（placeholder）选中，原回落项反选"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w12b").unwrap().value,
        "1/true",
        "W12：dirty-selected 保持——前插不改用户选择"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w12c").unwrap().value,
        "0/false",
        "W12：尾插非新首项——不触发重回落，原选中保持于 index 0"
    );
}

// W10：pristine 读取侧回落——无显式状态时 value/selectedIndex 落首个非 disabled；
// display size ≠ 1 不回落；options.selectedIndex 同视图。
#[test]
fn test_r3909_pristine_fallback() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='k9'><option id='k90' value='K0'></option><option id='k91' value='K1'></option><option id='k92' value='K2'></option></select>\
         <select id='k9b' size='2'><option></option><option></option></select>\
         <select id='k9c'><option disabled></option><option value='K1b'></option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var k9 = document.getElementById('k9');
globalThis.__w10a = String(k9.selectedIndex) + '/' + k9.value + '/' + String(k9.options.selectedIndex);
var k9b = document.getElementById('k9b');
globalThis.__w10b = String(k9b.selectedIndex) + '/' + String(k9b.value === '');
var k9c = document.getElementById('k9c');
globalThis.__w10c = String(k9c.selectedIndex);
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w10a").unwrap().value,
        "0/K0/0",
        "W10：pristine size 1——selectedIndex/value/options.selectedIndex 回落 index 0"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w10b").unwrap().value,
        "-1/true",
        "W10：pristine size 2——不回落，-1 且 value 空串"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w10c").unwrap().value,
        "1",
        "W10：pristine 首 option disabled——回落跳到 index 1"
    );
}

#[test]
fn test_r3909_batch_and_dirty_fallback() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// W1：dirty 条目存在即权威——selected=false 后回落仍生效（display size 1 首个非
// disabled）。此前 _zwOptSelState 只短路 dirty-true，插入回落物化的 clean 层透过
// dirty false 误现，setting 走查误判既有 true → 回落失效（WPT select-ask-for-reset
// 「change selectedness of option」t2 段实证）。
var s1 = document.createElement('select');
for (var i = 0; i < 3; ++i) s1.appendChild(document.createElement('option'));
var o1 = s1.children;
o1[2].selected = true;
o1[2].selected = false;
globalThis.__w1 = o1[0].selected + '/' + o1[1].selected + '/' + o1[2].selected + '/' + s1.selectedIndex;
// W2：批 append(first, second)（第二者带 selected 属性）——insertion steps 逐 option
// 依序各跑，re-fallback 清扫不得吞掉**尚未处理**兄弟的属性背书态（清掉曾令 second
// 在自身步骤读 false，script-mutation「removing selected attribute」precondition 1F）。
var s2 = document.createElement('select');
var f2 = new Option('123');
var x2 = new Option('234');
x2.defaultSelected = true;
s2.append(f2, x2);
globalThis.__w2a = s2.selectedIndex + '/' + f2.selected + '/' + x2.selected;
x2.removeAttribute('selected');
globalThis.__w2b = s2.selectedIndex + '/' + f2.selected + '/' + x2.selected;
// W3：逐个 appendChild 同型（回归锚——批路径修复不得破坏单插路径）。
var s3 = document.createElement('select');
var f3 = new Option('123');
var x3 = new Option('234');
x3.setAttribute('selected', '');
s3.appendChild(f3);
s3.appendChild(x3);
globalThis.__w3 = s3.selectedIndex + '/' + f3.selected + '/' + x3.selected;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1").unwrap().value,
        "true/false/false/0",
        "W1：selected=false 后回落命中 index 0（dirty 条目权威，clean 回落态不透过）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2a").unwrap().value,
        "1/false/true",
        "W2a：批 append 属性背书 second 胜出 idx 1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2b").unwrap().value,
        "0/true/false",
        "W2b：移除 selected 属性 → 回落首项"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3").unwrap().value,
        "1/false/true",
        "W3：逐个 appendChild 同型胜出（回归锚）"
    );
}

// W10：select.value= setter 同 turn 真态可见（R3909 写侧统一——HTML §4.10.7 value
// setter steps：全员 selectedness=false、首个 value 匹配者 selectedness=true +
// dirtiness=true）。回归面：form-validation-valueMissing「Selected the option with
// value equals to 1」（createElement select 同 turn validity 读）。
#[test]
fn test_r3909_select_value_setter_same_turn() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='p9'><option value=''>ph</option><option value='1'>one</option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var sel = document.createElement('select');
document.body.appendChild(sel);
sel.add(new Option('test1', ''));
sel.add(new Option('test2', 1));
var o1 = sel.options[0], o2 = sel.options[1];
sel.value = '';
globalThis.__w10a = sel.value + '|' + o1.selected + '/' + o2.selected + '|' + sel.selectedIndex;
sel.value = '1';
globalThis.__w10b = sel.value + '|' + o1.selected + '/' + o2.selected + '|' + sel.selectedIndex
  + '|vm=' + sel.validity.valueMissing;
sel.value = 'nope';
globalThis.__w10c = sel.value + '|' + sel.selectedIndex + '|o2=' + o2.selected;
var p9 = document.getElementById('p9');
var q0 = p9.options[0], q1 = p9.options[1];
p9.value = '1';
globalThis.__w10d = p9.value + '|' + q0.selected + '/' + q1.selected + '|' + p9.selectedIndex;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w10a").unwrap().value,
        "|true/false|0",
        "W10a：value='' 命中占位首项（selectedness true + dirtiness true）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w10b").unwrap().value,
        "1|false/true|1|vm=false",
        "W10b：value='1' 互斥切换 + valueMissing false（同 turn 真态）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w10c").unwrap().value,
        "|-1|o2=false",
        "W10c：无匹配值全员反选（selectedIndex -1、value ''——spec 值面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w10d").unwrap().value,
        "1|false/true|1",
        "W10d：解析 select（sel 形）同语义（SelectOption mutation 照记渲染面）"
    );
    // 渲染面：SelectOption mutation 仍入队（js_dom_bridge.rs apply 侧承载 selected 属性）。
    let mm = mutations.lock().unwrap();
    assert!(
        mm.iter().any(|m| matches!(
            m,
            DomMutation::SelectOption { value, .. } if value == "1"
        )),
        "SelectOption mutation 应照记（渲染/序列化面）"
    );
}

// ── R3910：form 提交 construct-entry-list 对齐（formdata 事件 + 共享 flag）+
// method=dialog 分支 + SubmitEvent/FormDataEvent 构造器 WebIDL 语义 ──
//
// 生产 shim 路径：part04 `_zwFormSubmitNavigate`（顶部 constructing entry list flag
// 检查 → 条目构造窗口置位 `__zwFormDataConstructing`（与 part02 new FormData(form)
// 共用同 spec flag）→ 条目构造后派发 formdata（FormData 与 entry list 共享 _p，
// handler 增删对提交编码可见）→ finally 清位 → method=dialog 分支移至 fire 之后：
// 最近祖先 dialog 为 subject，result = image 按钮坐标 "x,y" / submit 按钮 optional
// value（value 属性 present 才设，含空串）/ null 不改 returnValue）+ part01
// `_zwDialogClose` 未 open 早退（close-the-dialog 步 1）+ part05 两构造器无 new
// TypeError / required 成员校验。
// spec https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-algorithm
// 及 #constructing-entry-list、#the-formdataevent-interface、#submitevent。

// W1：SubmitEvent / FormDataEvent 构造器 WebIDL 语义（WPT form-submission-0
// FormDataEvent.window.js / SubmitEvent.window.js「Failing constructor」断言面）。
#[test]
fn test_r3910_submitevent_formdataevent_ctor_webidl() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
function tryCtor(f) { try { f(); return 'no-throw'; } catch (e) { return e.constructor.name; } }
globalThis.__fe = [
  tryCtor(function () { FormDataEvent('', { formData: new FormData() }); }),
  tryCtor(function () { new FormDataEvent(); }),
  tryCtor(function () { new FormDataEvent('foo'); }),
  tryCtor(function () { new FormDataEvent('foo', null); }),
  tryCtor(function () { new FormDataEvent('foo', { formData: null }); }),
  tryCtor(function () { new FormDataEvent('foo', { formData: 'bar' }); })
].join(',');
var __fd = new FormData();
var __fev = new FormDataEvent('bar', { formData: __fd, bubbles: true });
globalThis.__feOk = (__fev.formData === __fd) + '/' + __fev.bubbles;
globalThis.__se = [
  tryCtor(function () { SubmitEvent(''); }),
  tryCtor(function () { new SubmitEvent(); }),
  tryCtor(function () { new SubmitEvent('foo', { submitter: 'bar' }); })
].join(',');
var __btn = document.createElement('button');
var __sev = new SubmitEvent('bar', { submitter: __btn, bubbles: true });
globalThis.__seOk = (__sev.submitter === __btn)
  + '/' + (new SubmitEvent('x', null).submitter === null)
  + '/' + (new SubmitEvent('x').submitter === null)
  + '/' + __sev.bubbles;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fe").unwrap().value,
        "TypeError,TypeError,TypeError,TypeError,TypeError,TypeError",
        "W1：FormDataEvent 无 new/0 参/缺 init/null init/null formData/非 FormData → TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__feOk").unwrap().value,
        "true/true",
        "W1：成功构造——formData 同一实例 + bubbles 透传"
    );
    assert_eq!(
        sandbox.execute("globalThis.__se").unwrap().value,
        "TypeError,TypeError,TypeError",
        "W1：SubmitEvent 无 new/0 参/submitter 非元素 → TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__seOk").unwrap().value,
        "true/true/true/true",
        "W1：成功构造——submitter 元素实例 + null/缺省 dict → null + bubbles"
    );
}

// W2：dialog.close() 于未 open dialog 为 no-op（不改 returnValue）——close-the-dialog
// 步 1 早退（WPT dialog-close「doesn't have an open attribute aborts the steps」+
// dialog-return-value「no-op close()」断言面）。
#[test]
fn test_r3910_dialog_close_not_open_noop() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var d = document.createElement('dialog');
document.body.appendChild(d);
d.returnValue = 'keep-me';
var __closeFired = 0;
d.addEventListener('close', function () { __closeFired++; });
d.close('x');
globalThis.__r = d.returnValue + '/' + d.hasAttribute('open') + '/' + __closeFired;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "keep-me/false/0",
        "W2：未 open close('x') → returnValue 不变、无 close 事件"
    );
}

// W3：method=dialog 提交——submit 按钮 optional value 三态（有 value 属性 → 设
// returnValue；无属性 → 不改；空串属性 → 设空串），close 事件计数，弹回重开。
#[test]
fn test_r3910_dialog_submit_optional_value() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <dialog id='dgA'><form id='fmA' method='dialog'><button id='btA' value='rv1'>ok</button></form></dialog>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var dg = document.getElementById('dgA');
var bt = document.getElementById('btA');
var __closeN = 0;
dg.addEventListener('close', function () { __closeN++; });
// ①：value 属性 → returnValue = 属性值。
dg.show();
bt.click();
globalThis.__r1 = dg.open + '/' + dg.returnValue + '/' + __closeN;
// ②：移除 value 属性 → returnValue 不改。
dg.show();
dg.returnValue = 'initial';
bt.removeAttribute('value');
bt.click();
globalThis.__r2 = dg.open + '/' + dg.returnValue;
// ③：空串 value 属性 → returnValue 设空串。
dg.show();
dg.returnValue = 'initial2';
bt.setAttribute('value', '');
bt.click();
globalThis.__r3 = dg.open + '/' + dg.returnValue + '/' + __closeN;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "false/rv1/1",
        "W3①：value 属性 → 关闭 + returnValue=rv1 + close 事件 1 次"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "false/initial",
        "W3②：无 value 属性 → 关闭但 returnValue 不改（optional value 缺失）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r3").unwrap().value,
        "false//3",
        "W3③：空串 value 属性 → returnValue 设空串（'false//3' 中段为空串）"
    );
}

// W4：dialog 分支边界——method=dialog 无祖先 dialog → return（无导航无异常）；
// formmethod=dialog 覆盖 form method=post → 关闭最近祖先 dialog。
#[test]
fn test_r3910_dialog_submit_no_ancestor_and_formmethod() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <form id='fmB' method='dialog'><button id='btB'>go</button></form>\
         <dialog id='dgC'><form id='fmC' method='post'><button id='btC' formmethod='dialog'>go</button></form></dialog>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var btB = document.getElementById('btB');
var __navN = 0;
if (typeof navigation === 'object' && navigation && typeof navigation.addEventListener === 'function') {
  navigation.addEventListener('navigate', function () { __navN++; });
}
// ①：method=dialog 无祖先 dialog → return，无导航。
btB.click();
globalThis.__r1 = __navN;
// ②：formmethod=dialog 覆盖 method=post → 关闭祖先 dialog，无 value 属性 → returnValue 不改。
var dg = document.getElementById('dgC');
var bt = document.getElementById('btC');
dg.show();
dg.returnValue = 'pre';
bt.click();
globalThis.__r2 = dg.open + '/' + dg.returnValue + '/' + __navN;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "0",
        "W4①：无祖先 dialog → return（零导航）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "false/pre/0",
        "W4②：formmethod=dialog 覆盖 post → 关闭 dialog，无 value 属性 returnValue 不改"
    );
}

// W5：form.submit() 派发 formdata（bubbles、FormDataEvent 实例、e.formData 为
// FormData）；handler 内重入 form.submit() 经 constructing entry list flag 短路
//（单次 fire）；handler 内 new FormData(form) 抛 InvalidStateError（flag 共享）。
#[test]
fn test_r3910_formdata_event_on_submit() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><form id='fmD'><input id='inD' name='n1' value='v1'></form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var f = document.getElementById('fmD');
var __c = 0, __fdIs = '', __bub = '', __evIs = '', __reErr = '';
f.addEventListener('formdata', function (e) {
  __c++;
  __fdIs = e.formData instanceof FormData;
  __bub = e.bubbles;
  __evIs = e instanceof FormDataEvent;
  try { new FormData(f); __reErr = 'no-throw'; } catch (err) { __reErr = err.name; }
  f.submit(); // 重入：constructing entry list flag true → 短路（不再 fire）
});
f.submit();
globalThis.__r = __c + '/' + __fdIs + '/' + __bub + '/' + __evIs + '/' + __reErr;
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1/true/true/true/InvalidStateError",
        "W5：formdata 单次 fire + bubbles + FormDataEvent/FormData 实例 + 重入 submit 短路 + 共享 flag 抛 InvalidStateError"
    );
}

// W6：formdata handler 对 e.formData 的增删改写提交 entry list（FormData 与 entry
// list 共享存储——POST navigate 事件 formData 读侧可见 handler 追加项）。
#[test]
fn test_r3910_formdata_mutation_reaches_entry_list() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><form id='fmE' method='post' action='/echo'><input id='inE' name='n1' value='v1'></form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://example.com/dir/page.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var f = document.getElementById('fmE');
f.addEventListener('formdata', function (e) { e.formData.append('extra', 'x1'); });
globalThis.__navN1 = 'unset';
globalThis.__navExtra = 'unset';
globalThis.__navSeen = 'no-nav';
if (typeof navigation === 'object' && navigation && typeof navigation.addEventListener === 'function') {
  navigation.addEventListener('navigate', function (e) {
    globalThis.__navSeen = 'nav';
    if (e.formData) {
      globalThis.__navN1 = String(e.formData.get('n1'));
      globalThis.__navExtra = String(e.formData.get('extra'));
    }
    e.preventDefault();
  });
}
f.submit();
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__navSeen").unwrap().value,
        "nav",
        "W6：POST 提交派发 navigate 事件"
    );
    assert_eq!(
        sandbox.execute("globalThis.__navN1 + ',' + globalThis.__navExtra").unwrap().value,
        "v1,x1",
        "W6：entry list 含控件 n1=v1 且 formdata handler 追加的 extra=x1 可见（共享存储）"
    );
}
