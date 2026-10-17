// js_dom_bridge 测试切片 58（R3908）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-57 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

// ── R3908：HTMLOptionsCollection / selectedOptions / option list 基建（html-semantics M2 切片 8）──
//
// 生产 shim 路径：part03 `_zwListOfOptions`（spec list-of-options 统一构造：OPTION 收录
// 不递归 / HR+SELECT 子树排除 / 其余容器递归）+ `_zwOptionsCollection`（get+set 双 trap：
// length/item/namedItem/add/remove/selectedIndex/value/索引 + length setter 与 [i] setter）
// + `_zwSelectedOptionsCollection`（HTMLCollection 形 + [SameObject] + 非 multiple 归一
// 视图）+ part04 SELECT 长度/索引 setter 委托。
// spec https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#htmloptionscollection 。
// 验证：W1 handle 形集合存在性；W2 length setter grow/shrink/等值；W3 索引 setter
//（null 移除/补齐后追加/replace）；W4 add 四形态（同元素 no-op/null 追加/元素插入/
// 数字索引/嵌套 reference 进 optgroup）；W5 remove；W6 namedItem（id/name 首匹配 +
// select.namedItem 委托）；W7 selectedOptions（multiple 过滤/非 multiple 归一/[SameObject]
// + live/constructor）；W8 list-of-options 嵌套排除（option/hr/select 子树排除、div 包裹
// 计入）；W9 handle 形 select.value/selectedIndex。
#[test]
fn test_html_options_collection_r3908() {
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
         <select id='s9'><option value='a'>1</option><option value='b' selected>2</option><option value='c'>3</option></select>\
         <select id='s9n'><option id='n1'>one</option><option>two</option><option>three</option></select>\
         <select id='s9i'><option selected>A</option><option selected>B</option><option>C</option></select>\
         <select id='s9m' multiple><option id='m1' selected>1</option><option id='m2' selected>2</option><option id='m3'>3</option></select>\
         <select id='s9m3' multiple><option id='q1' selected>1</option><option id='q2' selected>2</option><option id='q3' selected>3</option></select>\
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
// W1：handle 形 options 集合存在性（此前仅 sel 形——createElement select 的
// options.add/namedItem/selectedOptions 全 TypeError 面）。
var hs = document.createElement('select');
var o1 = document.createElement('option');
o1.value = 'x';
hs.appendChild(o1);
globalThis.__w1a = hs.options.length + '/' + (hs.options[0] === hs.firstChild);
globalThis.__w1b = (typeof hs.options.add) + '/' + (typeof hs.options.remove) + '/'
    + (typeof hs.options.namedItem) + '/' + (typeof hs.options.item);
globalThis.__w1c = (typeof hs.namedItem) + '/' + (typeof hs.selectedOptions);
// W2：select.length setter——grow 补空 option（无属性/无子）、shrink 末端裁剪、等值 no-op。
hs.length = 3;
globalThis.__w2a = hs.options.length + '/' + hs.children.length + '/'
    + hs.children[1].localName + '/' + hs.children[1].attributes.length + '/'
    + hs.children[1].childNodes.length;
globalThis.__w2b = hs.children[1].namespaceURI;
hs.length = 1;
globalThis.__w2c = hs.options.length + '/' + hs.children.length;
hs.length = 1;
globalThis.__w2d = hs.options.length;
// W3：索引 setter——null 移除；n>length 先补空 option 再追加；replace。
var hs2 = document.createElement('select');
hs2.length = 4;
var os2 = hs2.options;
os2[3] = null;
globalThis.__w3a = hs2.options.length + '/' + hs2.children.length;
var newOpt = document.createElement('option');
newOpt.value = '-1';
var repOpt = newOpt.cloneNode(true);
repOpt.value = 'a';
os2[5] = newOpt;
os2[0] = repOpt;
var w3b = [];
for (var i = 0; i < hs2.length; i++) w3b.push(hs2[i].value);
globalThis.__w3b = w3b.join(',');
// W4：add——同元素 no-op；null 追加；元素插入；数字索引；嵌套 reference 进 optgroup。
var s9 = document.getElementById('s9');
var aOpt = s9.options[0];
var aOg = null;
s9.options.add(aOpt, aOpt);
globalThis.__w4a = s9.options.length;
var a1 = document.createElement('option');
a1.value = 'app';
s9.options.add(a1, null);
globalThis.__w4b = s9.options[s9.options.length - 1].value;
var a2 = document.createElement('option');
a2.value = 'ins';
s9.options.add(a2, s9.options[0]);
globalThis.__w4c = s9.options[0].value;
var a3 = document.createElement('option');
a3.value = 'idx';
s9.options.add(a3, 0);
globalThis.__w4d = s9.options[0].value;
var nsel = document.createElement('select');
var nog = document.createElement('optgroup');
var nOptInOg = document.createElement('option');
nOptInOg.value = 'inog';
nog.appendChild(nOptInOg);
nsel.appendChild(nog);
var nNew = document.createElement('option');
nNew.value = 'newinog';
nsel.options.add(nNew, nOptInOg);
globalThis.__w4e = (nOptInOg.parentNode === nog) + '/' + nog.lastChild.value + '/'
    + nsel.options.length;
// W5：remove——列表项移除 + 越界 no-op。
nsel.options.remove(0);
globalThis.__w5a = nsel.options.length + '/' + (nsel.firstChild === nog);
nsel.options.remove(9);
globalThis.__w5b = nsel.options.length;
// W6：namedItem——id/name 首匹配、miss null、select.namedItem 委托。
var s9n = document.getElementById('s9n');
globalThis.__w6a = s9n.namedItem('n1').value + '/' + (s9n.namedItem('nope') === null);
globalThis.__w6b = s9n.options.namedItem('n1').value;
// W7：selectedOptions——multiple 过滤；非 multiple 归一（无选中回落首项/多选中最后者
// 胜）；[SameObject] + live；constructor。
var s9m = document.getElementById('s9m');
globalThis.__w7a = s9m.selectedOptions.length + '/' + (s9m.selectedOptions.item(0) === s9m.children[0])
    + '/' + (s9m.selectedOptions.constructor === globalThis.HTMLCollection);
var s9n2 = document.getElementById('s9n');
globalThis.__w7b = s9n2.selectedOptions.length + '/' + (s9n2.selectedOptions[0] === s9n2.children[0]);
var s9i = document.getElementById('s9i');
globalThis.__w7c = s9i.selectedOptions.length + '/' + s9i.selectedOptions[0].textContent;
var s9m3 = document.getElementById('s9m3');
var before = s9m3.selectedOptions;
var lenBefore = before.length;
s9m3.selectedOptions[1].selected = false;
globalThis.__w7d = (before === s9m3.selectedOptions) + '/' + lenBefore + '/'
    + before.length + '/' + s9m3.selectedOptions.length;
globalThis.__w7e = s9m.selectedOptions.namedItem('m2').id;
// W8：list-of-options 嵌套排除（handle 形）——option/hr/select 子树排除、div 包裹计入。
var esel = document.createElement('select');
esel.innerHTML = '<option>1';
var baseOpt = esel.firstChild;
var hrWrap = document.createElement('hr');
var hrOpt = document.createElement('option');
hrOpt.setAttribute('selected', '');
hrOpt.textContent = '2';
hrWrap.appendChild(hrOpt);
esel.appendChild(hrWrap);
globalThis.__w8a = esel.options.length + '/' + esel.selectedOptions.length + '/'
    + (esel.selectedOptions[0] === baseOpt) + '/' + esel.value;
var ssel = document.createElement('select');
var sInner = document.createElement('select');
var sOpt = document.createElement('option');
sOpt.setAttribute('selected', '');
sOpt.textContent = 'nested';
sInner.appendChild(sOpt);
ssel.appendChild(sInner);
globalThis.__w8b = ssel.options.length + '/' + ssel.selectedOptions.length;
var dsel = document.createElement('select');
dsel.multiple = true;
var dWrap = document.createElement('div');
var dOpt = document.createElement('option');
dOpt.setAttribute('selected', '');
dOpt.textContent = 'divwrap';
dWrap.appendChild(dOpt);
dsel.appendChild(dWrap);
globalThis.__w8c = dsel.options.length + '/' + dsel.selectedOptions.length + '/'
    + (dsel.selectedOptions[0] === dOpt);
var osel = document.createElement('select');
var oOuter = document.createElement('option');
var oInner = document.createElement('option');
oInner.setAttribute('selected', '');
oInner.textContent = 'inner';
oOuter.appendChild(oInner);
osel.appendChild(oOuter);
globalThis.__w8d = osel.options.length + '/' + (osel.options[0] === oOuter);
// W9：handle 形 select.value / selectedIndex。
var vsel = document.createElement('select');
var vOpt = document.createElement('option');
vOpt.setAttribute('value', 'v9');
vOpt.textContent = 'txt';
vsel.appendChild(vOpt);
globalThis.__w9a = vsel.value + '/' + vsel.selectedIndex;
"##,
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__w1a").unwrap().value,
        "1/true",
        "W1：handle 形 options 长度与索引项"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1b").unwrap().value,
        "function/function/function/function",
        "W1：add/remove/namedItem/item 方法存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1c").unwrap().value,
        "function/object",
        "W1：select.namedItem 与 selectedOptions 存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2a").unwrap().value,
        "3/3/option/0/0",
        "W2：grow 补空 option（无属性无子）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2b").unwrap().value,
        "http://www.w3.org/1999/xhtml",
        "W2：新 option 为 HTML ns"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2c").unwrap().value,
        "1/1",
        "W2：shrink 末端裁剪"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2d").unwrap().value,
        "1",
        "W2：等值 length no-op"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3a").unwrap().value,
        "3/3",
        "W3：[i]=null 移除"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3b").unwrap().value,
        "a,,,,,-1",
        "W3：n>length 补空后追加 + replace"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4a").unwrap().value,
        "3",
        "W4：add(before===element) no-op"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4b").unwrap().value,
        "app",
        "W4：add(el, null) 追加"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4c").unwrap().value,
        "ins",
        "W4：add(el, element) 前插"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4d").unwrap().value,
        "idx",
        "W4：add(el, 0) 索引插入"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4e").unwrap().value,
        "true/inog/2",
        "W4：嵌套 reference 插入 optgroup 父内（before 前插，reference 仍 lastChild）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5a").unwrap().value,
        "1/true",
        "W5：remove 移除列表项"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5b").unwrap().value,
        "1",
        "W5：remove 越界 no-op"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w6a").unwrap().value,
        "one/true",
        "W6：select.namedItem id 匹配 + miss null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w6b").unwrap().value,
        "one",
        "W6：options.namedItem 委托"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7a").unwrap().value,
        "2/true/true",
        "W7：multiple 过滤 + item + HTMLCollection 实例"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7b").unwrap().value,
        "1/true",
        "W7：非 multiple 无选中回落首项"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7c").unwrap().value,
        "1/B",
        "W7：非 multiple 多选中最后者胜"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7d").unwrap().value,
        "true/3/2/2",
        "W7：[SameObject] + live（取消选中后长度更新）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7e").unwrap().value,
        "m2",
        "W7：selectedOptions.namedItem"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8a").unwrap().value,
        "1/1/true/1",
        "W8：hr 包裹 option 排除（list/选中/value 三面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8b").unwrap().value,
        "0/0",
        "W8：嵌套 select 子树排除"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8c").unwrap().value,
        "1/1/true",
        "W8：div 包裹 option 计入"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w8d").unwrap().value,
        "1/true",
        "W8：option 内嵌 option 排除（仅外层收录）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w9a").unwrap().value,
        "v9/0",
        "W9：handle 形 select.value/selectedIndex"
    );
}
