//! P1b S1 原生 DOM 绑定测试——验证 native getter 管线（值传递 + GC + 真实 DOM 读）。
//!
//! 集成测试：建 Isolate+Context + 安装绑定 + 执行脚本读 `nodeType`/`tagName`（不经 shim
//! 字符串桥）。gc 单元测试：NodeId↔u64 编解码、stale 校验、状态隔离。

use std::cell::RefCell;
use std::rc::Rc;

use slotmap::Key;
use zero_dom::{NodeId, parse_html};

use super::gc::test_helpers::{
    attr_cache_alive, dtl_cache_alive, inject_dom_for_test, listener_keys_for, nnm_cache_alive, reset_for_test,
};
use super::{decode_node_id, encode_node_id, install_dom_bindings, node_exists};

/// 在自带 Isolate+Context 上安装绑定并执行脚本，返回结果字符串。
///
/// 镜像 S0 PoC（`script-sandbox::dom_bindings`）的 Isolate/Context 建法；`&mut ContextScope`
/// 经 DerefMut 协化为 `&mut PinScope` 传入 `install_dom_bindings`。每次测试后 `reset_for_test`
/// 清空线程局部状态（隔离）。
pub(super) fn run_script(html: &str, script: &str) -> String {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(html)));
    let isolate = &mut v8::Isolate::new(Default::default());
    let result;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        // 直接安装（不经 kill-switch），验证 native 管线。
        install_dom_bindings(scope, context, Rc::clone(&dom));
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let r = compiled.run(scope).expect("run");
        result = r
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
    }
    reset_for_test();
    result
}

/// 同 [`run_script`]，但额外返回**执行后** live doc 的 `outer_html`（js-dom M1 L2 R104
/// 三方合一验证资产：A(native 写) 与 B(polyfill 写) 的 C 侧读数——两路径的写对同一
/// live Document 的最终状态须等价）。不 reset 前先序列化（reset 清 DOM_SOURCE）。
pub(super) fn run_script_return_doc_html(html: &str, script: &str) -> (String, String) {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(html)));
    let isolate = &mut v8::Isolate::new(Default::default());
    let result;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let r = compiled.run(scope).expect("run");
        result = r
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
    }
    let doc_html = dom.borrow().outer_html(dom.borrow().root());
    reset_for_test();
    (result, doc_html)
}

/// 同 [`run_script`]，但解析后注入页面 URL（镜像 engine 导航层 `set_url`），验证
/// `document.URL`/`documentURI` native getter 经 live Document 读注入值（R3169）。
pub(super) fn run_script_with_url(html: &str, url: &str, script: &str) -> String {
    zero_script_sandbox::ensure_v8_initialized();
    let mut doc = parse_html(html);
    doc.set_url(Some(url.to_string()));
    let dom = Rc::new(RefCell::new(doc));
    let isolate = &mut v8::Isolate::new(Default::default());
    let result;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let r = compiled.run(scope).expect("run");
        result = r
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
    }
    reset_for_test();
    result
}

/// 同 [`run_script`]，但解析后注入 referrer（镜像 engine 导航层 `set_referrer`，referrer = 导航前
/// 的页面 URL），验证 `document.referrer` native getter 经 live Document 读注入值（R3176）。
pub(super) fn run_script_with_referrer(html: &str, referrer: &str, script: &str) -> String {
    zero_script_sandbox::ensure_v8_initialized();
    let mut doc = parse_html(html);
    doc.set_referrer(Some(referrer.to_string()));
    let dom = Rc::new(RefCell::new(doc));
    let isolate = &mut v8::Isolate::new(Default::default());
    let result;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let r = compiled.run(scope).expect("run");
        result = r
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
    }
    reset_for_test();
    result
}

/// `nodeType` + `tagName` 原生 getter：经 internal slot NodeId 直读 Rust DOM，
/// 返 `v8::Integer`/`v8::String`（不经 shim 字符串桥）。spec nodeType Element=1、tagName HTML 大写。
#[test]
fn native_node_type_and_tag_name() {
    let html = r#"<div id="a"><span id="b">x</span></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').nodeType)"), "1");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').tagName)"), "DIV");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('b').tagName)"), "SPAN");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('b').nodeType)"), "1");
}

/// 对象身份：同 id → 同对象（NodeId↔Global 映射，spec identity `===`）。
#[test]
fn native_element_identity() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a') === __zw_native_element_for_id('a'))"
        ),
        "true"
    );
}

/// 未找到 id → `null`（工厂 `get_element_by_id` 返 None 分支）。
#[test]
fn native_element_not_found() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('zzz'))"), "null");
}

/// 链式读 + 多 id 混合（验证多 NodeId 对象互不串扰）。
#[test]
fn native_multiple_elements_distinct() {
    let html = r#"<p id="p1"></p><p id="p2"></p>"#;
    let r = run_script(
        html,
        "(__zw_native_element_for_id('p1').tagName + '/' + __zw_native_element_for_id('p2').tagName)",
    );
    assert_eq!(r, "P/P");
    // 不同 id → 不同对象。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('p1') !== __zw_native_element_for_id('p2'))"
        ),
        "true"
    );
}

// ── S1 只读属性族（nodeName / getAttribute / hasAttribute / id / className）──

/// `nodeName`：Element == tagName（HTML 大写）。
#[test]
fn native_node_name() {
    let html = r#"<div id="a"><span id="b">x</span></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').nodeName)"), "DIV");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('b').nodeName)"), "SPAN");
    // nodeName == tagName（Element 上）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').nodeName === __zw_native_element_for_id('a').tagName)"
        ),
        "true"
    );
}

/// `getAttribute` / `hasAttribute`：读 Element 属性（spec：缺省 → null / false）。
#[test]
fn native_get_attribute_and_has_attribute() {
    let html = r#"<div id="a" class="row big" data-x="42"></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').getAttribute('class'))"),
        "row big"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').getAttribute('data-x'))"),
        "42"
    );
    // 缺省属性 → null（spec `dom-element-getattribute`）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').getAttribute('nope'))"),
        "null"
    );
    // hasAttribute：true / false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').hasAttribute('class'))"),
        "true"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').hasAttribute('nope'))"),
        "false"
    );
}

/// 反射属性 `id` / `className`（缺省 → 空串，spec reflected attr default）。
#[test]
fn native_id_and_class_name_reflected() {
    let html = r#"<div id="a" class="c1 c2"></div><div id="b"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').id)"), "a");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').className)"), "c1 c2");
    // 缺省 → 空串（reflected attr default ""）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('b').className)"), "");
    // className == getAttribute('class')。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').className === __zw_native_element_for_id('a').getAttribute('class'))"
        ),
        "true"
    );
}

// ── S3 selector→NodeId native（querySelector / querySelectorAll，全量选择器引擎）──

/// `querySelector`：全量选择器引擎（tag / `.class` / `[attr=val]` / 后代组合器）→ native 对象，
/// 经 R3098 native getter/getAttribute 读真实属性（无 String 往返）。无匹配 / 非法 → `null`。
#[test]
fn native_query_selector() {
    let html = r#"<div id="root"><span class="row" data-x="1">a</span><span class="row big" data-x="2">b</span></div>"#;
    // tag 选择器 → 首个 span（文档序）。
    assert_eq!(
        run_script(html, "(__zw_native_query_selector('span').getAttribute('data-x'))"),
        "1"
    );
    // `.class` 选择器 → 首个匹配 span。
    assert_eq!(run_script(html, "(__zw_native_query_selector('.row').tagName)"), "SPAN");
    // `[attr=val]` 精确匹配 → data-x="2" 那个。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_query_selector('[data-x=\"2\"]').getAttribute('data-x'))"
        ),
        "2"
    );
    // 后代组合器 div span → 首个 span。
    assert_eq!(
        run_script(html, "(__zw_native_query_selector('div span').nodeType)"),
        "1"
    );
    // 无匹配 → null。
    assert_eq!(run_script(html, "(__zw_native_query_selector('.nope'))"), "null");
    // 非法选择器 → null（parse 失败，无 panic）。
    assert_eq!(run_script(html, "(__zw_native_query_selector('!!!'))"), "null");
}

/// `querySelectorAll`：全部匹配 → V8 Array of native 对象（文档序，含跨 tag `.class` + 组合器）。
#[test]
fn native_query_selector_all() {
    let html = r#"<div id="root"><span class="row" data-x="1">a</span><span class="row big" data-x="2">b</span><p class="row">c</p></div>"#;
    // 全部 span（文档序）。
    assert_eq!(run_script(html, "(__zw_native_query_selector_all('span').length)"), "2");
    // 各元素经 native getter 读属性（文档序 data-x 序）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_query_selector_all('span')[0].getAttribute('data-x') + '/' + __zw_native_query_selector_all('span')[1].getAttribute('data-x'))"
        ),
        "1/2"
    );
    // `.class` 跨 tag（span + span + p = 3 个 .row）。
    assert_eq!(run_script(html, "(__zw_native_query_selector_all('.row').length)"), "3");
    // 后代组合器 div span → 2 个。
    assert_eq!(
        run_script(html, "(__zw_native_query_selector_all('div span').length)"),
        "2"
    );
    // 无匹配 → 空 Array（length 0）。
    assert_eq!(
        run_script(html, "(__zw_native_query_selector_all('.nope').length)"),
        "0"
    );
}

// ── S3b element 子树作用域 querySelector / querySelectorAll（仅后代，排除元素自身）──

/// `element.querySelector(sel)`：元素**子树**作用域 + 仅后代（排除元素自身）。
/// `mid` 自身是 `.x` 但 `.querySelector('.x')` 应返其首个后代 `.x`（span.deep），非自身。
#[test]
fn native_element_query_selector() {
    let html = r#"<div id="root"><div id="mid" class="x" data-i="mid"><span class="x" data-i="deep">a</span></div><span class="x" data-i="sib">b</span></div>"#;
    let mid = "__zw_native_element_for_id('mid')";
    // `.x`：mid 自身 .x 排除 → 首个后代 .x = span.deep。
    assert_eq!(
        run_script(html, &format!("({mid}.querySelector('.x').getAttribute('data-i'))")),
        "deep"
    );
    // 子树作用域：mid 内仅 1 个 span（deep）；兄弟 span.sib 在 mid 外不匹配。
    assert_eq!(
        run_script(html, &format!("({mid}.querySelectorAll('span').length)")),
        "1"
    );
    // tag 后代选择器。
    assert_eq!(
        run_script(html, &format!("({mid}.querySelector('span').getAttribute('data-i'))")),
        "deep"
    );
    // 链式：root.querySelector('#mid') 返 native 对象，其上再 querySelector（R3099 结果→R3098/3b 方法）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('root').querySelector('#mid').querySelector('span').getAttribute('data-i'))"
        ),
        "deep"
    );
    // 无匹配 → null。
    assert_eq!(run_script(html, &format!("({mid}.querySelector('.nope'))")), "null");
}

/// `element.querySelectorAll(sel)`：全部**后代**匹配（文档序，排除元素自身 + 子树外节点）。
#[test]
fn native_element_query_selector_all() {
    let html = r#"<div id="root"><div id="mid" class="x" data-i="mid"><span class="x" data-i="deep">a</span></div><span class="x" data-i="sib">b</span></div>"#;
    // root 子树全部 .x：mid + span.deep + span.sib = 3（root 自身无 class）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('root').querySelectorAll('.x').length)"
        ),
        "3"
    );
    // mid 子树 .x：仅 span.deep（mid 自身 .x 排除，sib 在子树外）= 1。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('mid').querySelectorAll('.x').length)"
        ),
        "1"
    );
    // 文档序 + 各元素 native getter 读：root 下两 span [deep, sib]。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('root').querySelectorAll('span')[0].getAttribute('data-i') + '/' + __zw_native_element_for_id('root').querySelectorAll('span')[1].getAttribute('data-i'))"
        ),
        "deep/sib"
    );
    // 无匹配 → 空 Array。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('mid').querySelectorAll('.nope').length)"
        ),
        "0"
    );
}

// ── S2 写入路径 native（setAttribute / removeAttribute / id setter / className setter）──

/// `setAttribute` / `removeAttribute`：写 Element 属性经 `with_dom_mut` → `Document::set_attribute`，
/// 经 R3098 `getAttribute`/`hasAttribute` 原生回读验证（native 读写 round-trip）。
#[test]
fn native_set_and_remove_attribute() {
    let html = r#"<div id="a" class="old"></div>"#;
    // setAttribute 新增 → getAttribute 读回。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.setAttribute('data-x','42'); return el.getAttribute('data-x');})()"
        ),
        "42"
    );
    // setAttribute 覆盖既有属性。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.setAttribute('class','new'); return el.className;})()"
        ),
        "new"
    );
    // removeAttribute → hasAttribute=false + getAttribute=null。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.removeAttribute('class'); return el.hasAttribute('class')+'';})()"
        ),
        "false"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.removeAttribute('class'); return el.getAttribute('class');})()"
        ),
        "null"
    );
    // setAttribute('id',...) 更新 id_map：新 id 可查，旧 id 失效（同对象身份）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.setAttribute('id','b'); return (__zw_native_element_for_id('b')===el)+'';})()"
        ),
        "true"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.setAttribute('id','b'); return (__zw_native_element_for_id('a')===null)+'';})()"
        ),
        "true"
    );
}

/// 反射属性 `id` / `className` setter（值 ToString 后 set_attribute），与 R3098 getter round-trip。
#[test]
fn native_id_and_class_name_setters() {
    let html = r#"<div id="a"></div>"#;
    // id setter → id getter + getAttribute('id')。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.id='newid'; return el.id+'/'+el.getAttribute('id');})()"
        ),
        "newid/newid"
    );
    // className setter → className getter + getAttribute('class')。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.className='c1 c2'; return el.className+'/'+el.getAttribute('class');})()"
        ),
        "c1 c2/c1 c2"
    );
    // 值 ToString 强转（数字 → 字符串）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.className=42; return el.className;})()"
        ),
        "42"
    );
}

// ── S2 树 mutation native（createElement + appendChild/insertBefore/removeChild + children）──

/// `createElement` + `appendChild` + `children`：原生树构建 + 元素子观察。
#[test]
fn native_create_append_children() {
    let html = r#"<div id="root"></div>"#;
    // createElement 造新对象 → appendChild 落位 → children 观察。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); const c=__zw_native_create_element('div'); c.id='c1'; r.appendChild(c); return r.children.length+'/'+r.children[0].id+'/'+r.children[0].tagName;})()"
        ),
        "1/c1/DIV"
    );
    // appendChild 返回被追加的 child（spec）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); const c=__zw_native_create_element('p'); return (r.appendChild(c)===c)+'';})()"
        ),
        "true"
    );
    // 跨 tag：create('span') 后 append。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); const c=__zw_native_create_element('span'); r.appendChild(c); return r.children[0].tagName;})()"
        ),
        "SPAN"
    );
}

/// `appendChild` re-parent：移动既有节点（detach 旧父 + 挂新父）。
#[test]
fn native_append_child_reparents() {
    let html = r#"<div id="a"><span id="x">x</span></div><div id="b"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b'); b.appendChild(__zw_native_element_for_id('x')); return a.children.length+'/'+b.children.length+'/'+b.children[0].id;})()"
        ),
        "0/1/x"
    );
}

/// `insertBefore(newChild, refChild)` 位置插入（refChild null → 末尾追加）。
#[test]
fn native_insert_before() {
    let html = r#"<div id="root"><span id="s1">1</span><span id="s2">2</span></div>"#;
    // insertBefore(nw, s2) → 顺序 [s1, nw, s2]。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); const nw=__zw_native_create_element('span'); nw.id='nw'; r.insertBefore(nw, __zw_native_element_for_id('s2')); return r.children[0].id+'/'+r.children[1].id+'/'+r.children[2].id;})()"
        ),
        "s1/nw/s2"
    );
    // refChild 缺省 → 末尾追加（同 appendChild）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); const nw=__zw_native_create_element('span'); nw.id='last'; r.insertBefore(nw); return r.children[r.children.length-1].id;})()"
        ),
        "last"
    );
}

/// `removeChild` 移除 + `children` **仅元素**（跳过文本节点）。
#[test]
fn native_remove_child_and_children_element_only() {
    let html = r#"<div id="root"><span id="s1">1</span><span id="s2">2</span></div>"#;
    // removeChild → 剩余 [s2]。
    assert_eq!(
        run_script(
            html,
            "(()=>{const r=__zw_native_element_for_id('root'); r.removeChild(__zw_native_element_for_id('s1')); return r.children.length+'/'+r.children[0].id;})()"
        ),
        "1/s2"
    );
    // children 仅元素：HTML 含文本节点 "hello"/"world"，children 跳过 → 仅 1 个 span。
    let html2 = r#"<div id="root">hello<span id="s1">1</span>world</div>"#;
    assert_eq!(
        run_script(html2, "(__zw_native_element_for_id('root').children.length)"),
        "1"
    );
    assert_eq!(
        run_script(html2, "(__zw_native_element_for_id('root').children[0].id)"),
        "s1"
    );
}

// ── textContent native（子树文本读 + 清子写文本节点）──

/// `textContent` getter：子树文本拼接（含后代 Text 节点）；空子树 → 空串。
#[test]
fn native_text_content_getter() {
    let html = r#"<div id="a">hello <span>world</span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').textContent)"),
        "hello world"
    );
    // 深层嵌套文本拼接。
    let html2 = r#"<div id="a">a<b>b<i>c</i>d</b>e</div>"#;
    assert_eq!(
        run_script(html2, "(__zw_native_element_for_id('a').textContent)"),
        "abcde"
    );
    // 空子树 → 空串。
    let html3 = r#"<div id="a"></div>"#;
    assert_eq!(run_script(html3, "(__zw_native_element_for_id('a').textContent)"), "");
}

/// `textContent` setter：清空既有子节点 + 追加文本节点；空串 → 仅清空。
#[test]
fn native_text_content_setter() {
    // 空元素 set textContent → 读回。
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.textContent='hi'; return el.textContent;})()"
        ),
        "hi"
    );
    // set textContent 替换既有子元素（children 清空，文本节点非元素）。
    let html2 = r#"<div id="a"><span>x</span><span>y</span></div>"#;
    assert_eq!(
        run_script(
            html2,
            "(()=>{const el=__zw_native_element_for_id('a'); el.textContent='replaced'; return el.textContent+'/'+el.children.length;})()"
        ),
        "replaced/0"
    );
    // set textContent='' → 清空（无文本节点，读回空串）。
    assert_eq!(
        run_script(
            html2,
            "(()=>{const el=__zw_native_element_for_id('a'); el.textContent=''; return el.textContent+'/'+el.children.length;})()"
        ),
        "/0"
    );
    // 值 ToString 强转（数字 → 字符串）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.textContent=42; return el.textContent;})()"
        ),
        "42"
    );
}

// ── node-types native（childNodes 含文本/注释 + nodeValue，解锁 Text/Comment 节点可见）──

/// `childNodes`：全部子节点（含文本/注释）；文本节点经同模板包 → nodeType 3 / nodeName #text /
/// nodeValue=data 正确。区别于 `children`（仅元素）。
#[test]
fn native_child_nodes() {
    let html = r#"<div id="a">hello<span id="s">x</span>world</div>"#;
    // childNodes = [text "hello", span, text "world"] = 3。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes.length)"),
        "3"
    );
    // 首子文本节点：nodeType 3 / nodeName #text / nodeValue "hello"。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[0].nodeType)"),
        "3"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[0].nodeName)"),
        "#text"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[0].nodeValue)"),
        "hello"
    );
    // 次子元素：nodeType 1 / nodeValue null。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[1].nodeType)"),
        "1"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[1].nodeValue)"),
        "null"
    );
    // 末子文本 "world"。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').childNodes[2].nodeValue)"),
        "world"
    );
}

/// `childNodes` 见 textContent 写的文本节点（闭合 R3103 限制②：文本节点经 children 不可见、经 childNodes 可见）。
#[test]
fn native_child_nodes_after_text_content_setter() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{const el=__zw_native_element_for_id('a'); el.textContent='hi'; return el.childNodes.length+'/'+el.childNodes[0].nodeType+'/'+el.childNodes[0].nodeValue;})()"
        ),
        "1/3/hi"
    );
}

// ── gc.rs 单元测试 ────────────────────────────────────────────────

/// NodeId↔u64(ffi) 编解码 round-trip（internal slot 值传递基础）。
///
/// 用真实 NodeId（slotmap serial≥1，非 null 保留值 0）验证 encode/decode 互逆——
/// internal slot 存的是 `encode` 的 ffi 值，getter 经 `decode` 还原 NodeId 读 DOM
/// （端到端经 native_*_getter 测试覆盖）。注：slotmap `serial=0` 为 null 保留值，
/// `from_ffi(0)` 非真实 NodeId，故用文档实节点。
#[test]
fn node_id_ffi_round_trip() {
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'><span id='b'>x</span></div>")));
    let ids: Vec<NodeId> = {
        let d = dom.borrow();
        vec![d.get_element_by_id("a").unwrap(), d.get_element_by_id("b").unwrap()]
    };
    inject_dom_for_test(Rc::clone(&dom));
    for id in ids {
        let ffi = encode_node_id(id);
        assert_ne!(ffi, 0, "真实 NodeId ffi 应非 0（slotmap serial≥1）");
        // encode ∘ decode 保 ffi（internal slot 存的即 ffi 值）。
        assert_eq!(encode_node_id(decode_node_id(ffi)), ffi);
        // decode ∘ encode 还原 NodeId（getter 用，native 测试已端到端证明）。
        assert_eq!(decode_node_id(encode_node_id(id)), id);
    }
    reset_for_test();
}

/// stale 校验：文档内已知 NodeId → true；null / 不存在 NodeId → false。
#[test]
fn node_exists_known_and_unknown() {
    let dom = Rc::new(RefCell::new(parse_html(r#"<div id="a"><span id="b">x</span></div>"#)));
    let node_a = dom.borrow().get_element_by_id("a").expect("id a");
    inject_dom_for_test(Rc::clone(&dom));
    assert!(node_exists(node_a), "已知节点应存在");
    // null NodeId（slotmap KeyData::null）不在 SlotMap → stale。
    assert!(!node_exists(NodeId::null()), "null NodeId 应判 stale");
    reset_for_test();
}

/// 状态隔离：reset 后 DOM 源清空（with_dom 返 None）。
#[test]
fn dom_source_cleared_after_reset() {
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'></div>")));
    let node = dom.borrow().get_element_by_id("a").unwrap();
    inject_dom_for_test(dom);
    assert!(node_exists(node));
    reset_for_test();
    // reset 后无 DOM 源 → node_exists 返 false（with_dom None）。
    assert!(!node_exists(node));
}

// ── S4 EventTarget：addEventListener / removeEventListener / dispatchEvent（本轮 R3109）──

/// `addEventListener(type, fn)` + `dispatchEvent({type})` 派发触发监听器（spec `dom-eventtarget`）。
#[test]
fn native_event_target_dispatch_fires_listener() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.addEventListener('click', ()=>{ globalThis.__clicked='yes'; });\
             el.dispatchEvent({type:'click'});\
             return globalThis.__clicked || 'no'; })()"
        ),
        "yes"
    );
}

/// `removeEventListener(type, fn)` 后 dispatch 不再触发（spec 身份匹配 strict_equals）。
#[test]
fn native_event_target_remove_listener() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const fn=()=>{ globalThis.__clicked='yes'; };\
             el.addEventListener('click', fn);\
             el.removeEventListener('click', fn);\
             el.dispatchEvent({type:'click'});\
             return globalThis.__clicked || 'no'; })()"
        ),
        "no"
    );
}

/// 事件类型过滤：dispatchEvent({type:'click'}) 仅触发 click 监听器，不触发 keyup。
#[test]
fn native_event_target_type_filter() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.addEventListener('click',  ()=>{ globalThis.__c='yes'; });\
             el.addEventListener('keyup', ()=>{ globalThis.__k='yes'; });\
             el.dispatchEvent({type:'click'});\
             return (globalThis.__c||'no')+'/'+(globalThis.__k||'no'); })()"
        ),
        "yes/no"
    );
}

/// dispatchEvent 返 true（spec：未 preventDefault）；无监听器也不抛。
#[test]
fn native_event_target_dispatch_returns_true() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').dispatchEvent({type:'x'}))"),
        "true"
    );
}

// ── R3125 dispatchEvent target+bubble 两阶段冒泡 ──
//
// HTML: <div id="parent"><div id="child"></div></div>

/// 冒泡：child.dispatchEvent({type:'click',bubbles:true}) 先触发 child（AT_TARGET=2），
/// 再上溯触发 parent（BUBBLING_PHASE=3）。event.target 全程=child；currentTarget 随层变。
#[test]
fn native_dispatch_event_bubbles_to_ancestor_r3125() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
parent.addEventListener('click',e=>{log.push('parent:'+(e.target===child)+'/'+(e.currentTarget===parent)+'/'+e.eventPhase);});\
child.addEventListener('click',e=>{log.push('child:'+(e.target===child)+'/'+(e.currentTarget===child)+'/'+e.eventPhase);});\
child.dispatchEvent({type:'click',bubbles:true});\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "child:true/true/2,parent:true/true/3",
        "冒泡：child AT_TARGET 先、parent BUBBLING_PHASE 后；target 固定 / currentTarget 随层"
    );
}

/// 非 bubbles 事件不上溯：dispatchEvent({type:'click'})（bubbles 缺省 false）只触发 target。
#[test]
fn native_dispatch_event_non_bubble_stops_at_target_r3125() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
let parent_fired='no';\
parent.addEventListener('click',()=>{parent_fired='yes';});\
child.dispatchEvent({type:'click'});\
return parent_fired;})()";
    assert_eq!(run_script(html, script), "no", "非 bubbles 事件不冒泡到 parent");
}

/// 派发后 event.currentTarget=null、eventPhase=0（spec：派发结束清理）。
#[test]
fn native_dispatch_event_post_dispatch_state_r3125() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{const e={type:'click',bubbles:true};\
__zw_native_element_for_id('a').dispatchEvent(e);\
return e.eventPhase+'/'+(e.currentTarget===null);})()";
    assert_eq!(
        run_script(html, script),
        "0/true",
        "派发后 eventPhase=0 / currentTarget=null"
    );
}

// ── R3126 stopPropagation / stopImmediatePropagation ──
//
// HTML: <div id="parent"><div id="child"></div></div>

/// stopPropagation：止上溯祖先，但当前节点（target）剩余监听器仍触发（spec：只阻后续节点）。
#[test]
fn native_dispatch_event_stop_propagation_r3126() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
child.addEventListener('click',e=>{log.push('child1');e.stopPropagation();});\
child.addEventListener('click',e=>{log.push('child2');});\
parent.addEventListener('click',e=>{log.push('parent');});\
child.dispatchEvent({type:'click',bubbles:true});\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "child1,child2",
        "stopPropagation：target 剩余监听器仍触发，但止上溯 parent"
    );
}

/// stopImmediatePropagation：立即终止——当前节点剩余监听器 + 上溯均不触发。
#[test]
fn native_dispatch_event_stop_immediate_r3126() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
child.addEventListener('click',e=>{log.push('child1');e.stopImmediatePropagation();});\
child.addEventListener('click',e=>{log.push('child2');});\
parent.addEventListener('click',e=>{log.push('parent');});\
child.dispatchEvent({type:'click',bubbles:true});\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "child1",
        "stopImmediatePropagation：剩余监听器 + 上溯均止"
    );
}

/// 派发前复位 stop flag：同一 event 对象二次派发，第二次不调 stop 则正常上溯（防 stale flag 误止）。
#[test]
fn native_dispatch_event_stop_flag_reset_r3126() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
let mode='stop';\
child.addEventListener('click',e=>{if(mode==='stop')e.stopImmediatePropagation();});\
parent.addEventListener('click',e=>{log.push('parent');});\
const ev={type:'click',bubbles:true};\
child.dispatchEvent(ev);\
mode='nostop';\
child.dispatchEvent(ev);\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "parent",
        "flag 复位：二次派发无 stop 时正常上溯（非 stale flag 误止）"
    );
}

// ── R3127 Event / CustomEvent 构造器 ──

/// `new Event(type, {bubbles,cancelable})`：instanceof Event 成立 + type/bubbles/cancelable 读 init dict +
/// 默认派发态（target=null、eventPhase=0、defaultPrevented=false、isTrusted=false）。
#[test]
fn native_event_constructor_basic_r3127() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const e=new Event('click',{bubbles:true,cancelable:true});\
return (e instanceof Event)+'/'+e.type+'/'+e.bubbles+'/'+e.cancelable+'/'+\
(e.target===null)+'/'+e.eventPhase+'/'+e.defaultPrevented+'/'+e.isTrusted;})()";
    assert_eq!(
        run_script(html, script),
        "true/click/true/true/true/0/false/false",
        "new Event：instanceof + type/bubbles/cancelable + 派发态默认"
    );
}

/// 缺省 init dict：bubbles/cancelable 默认 false（spec）。
#[test]
fn native_event_constructor_defaults_r3127() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{const e=new Event('x'); return e.bubbles+'/'+e.cancelable;})()";
    assert_eq!(
        run_script(html, script),
        "false/false",
        "new Event 缺省 init dict → bubbles/cancelable false"
    );
}

/// `Event.timeStamp`（js-dom R22）：DOMHighResTimeStamp——创建时刻的单调 perf time（ms，子毫秒）。
/// 旧实现恒 0 致 WPT `Event-timestamp-safe-resolution.html` 的 `do { e2.timeStamp - e1.timeStamp }
/// while (==0)` 死循环（连续 new MouseEvent 时间戳相同 → 恒 0 差，>60s 卡死 native dom/events 全量）。
/// 修：timeStamp = perf_now_ms()（进程级 OnceLock<Instant> origin elapsed）。验证：① 非 0；② 是有限数；
/// ③ 连续两次创建差值可收敛非死循环（do-while 累积差，模拟 WPT 收集逻辑，限定迭代次数防爆）。
#[test]
fn native_event_timestamp_monotonic_nonzero_r22() {
    let html = r#"<div id="a"></div>"#;
    // 验证 timeStamp 非 0 + Number.isFinite（spec DOMHighResTimeStamp 是有限数，非 0 origin 后恒正）。
    let script = "(()=>{const e=new Event('x'); return (e.timeStamp>0)+'/'+Number.isFinite(e.timeStamp);})()";
    assert_eq!(
        run_script(html, script),
        "true/true",
        "R22 Event.timeStamp 非 0 + 有限数（DOMHighResTimeStamp）"
    );
    // 验证连续创建可收集非零差（模拟 WPT Event-timestamp-safe-resolution do-while 逻辑，限 1e4 迭代防爆）：
    // 旧恒 0 时此循环永不退出（delta 恒 0）；修复后 perf_now_ms 单调推进，有限迭代内必得非零 delta。
    let script2 = "(()=>{\
let nonZero=0;\
for(let i=0;i<1e4;i++){\
  const e1=new MouseEvent('a');const e2=new MouseEvent('b');\
  const d=Math.round((e2.timeStamp-e1.timeStamp)*1000);\
  if(d!==0){nonZero=d;break;}\
}\
return nonZero>0;})()";
    assert_eq!(
        run_script(html, script2),
        "true",
        "R22 连续 new MouseEvent timeStamp 差可收集非零（解锁 WPT do-while 死循环）"
    );
}

/// `new CustomEvent(type, {detail})`：instanceof CustomEvent + detail 读 init dict（任意类型，缺省 null）。
#[test]
fn native_custom_event_constructor_r3127() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const e=new CustomEvent('build',{detail:{name:'zero'}});\
return (e instanceof CustomEvent)+'/'+e.type+'/'+e.detail.name+'/'+\
(new CustomEvent('p').detail===null);})()";
    assert_eq!(
        run_script(html, script),
        "true/build/zero/true",
        "new CustomEvent：instanceof + detail + 缺省 null"
    );
}

/// preventDefault：cancelable 时设 defaultPrevented=true，非 cancelable 时 no-op（spec）。
#[test]
fn native_event_prevent_default_r3127() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const c=new Event('x',{cancelable:true}); c.preventDefault();\
const nc=new Event('x'); nc.preventDefault();\
return c.defaultPrevented+'/'+nc.defaultPrevented;})()";
    assert_eq!(
        run_script(html, script),
        "true/false",
        "preventDefault：cancelable 设 defaultPrevented，非 cancelable no-op"
    );
}

/// Event 实例经原型 stopPropagation 集成 R3125 冒泡：派发时原型方法止上溯（不需派发注入）。
#[test]
fn native_event_dispatch_bubble_stop_r3127() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
child.addEventListener('click',e=>{log.push('child:'+e.eventPhase);e.stopPropagation();});\
parent.addEventListener('click',e=>{log.push('parent');});\
child.dispatchEvent(new Event('click',{bubbles:true}));\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "child:2",
        "new Event 派发：原型 stopPropagation 止上溯（AT_TARGET=2）"
    );
}

// ── R3128 capture 阶段 + useCapture（三阶段派发）──
//
// HTML: <div id="parent"><div id="child"></div></div>

/// 三阶段顺序：parent capture（phase=1）→ child target（phase=2）→ parent bubble（phase=3）。
#[test]
fn native_dispatch_event_capture_phase_order_r3128() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
parent.addEventListener('click',e=>{log.push('parent-capture:'+e.eventPhase);},true);\
child.addEventListener('click',e=>{log.push('child:'+e.eventPhase);});\
parent.addEventListener('click',e=>{log.push('parent-bubble:'+e.eventPhase);});\
child.dispatchEvent(new Event('click',{bubbles:true}));\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "parent-capture:1,child:2,parent-bubble:3",
        "三阶段顺序：capture(1) → target(2) → bubble(3)"
    );
}

/// capture 阶段 stopPropagation：止 target + bubble（capture 跨阶段生效）。
#[test]
fn native_dispatch_event_capture_stop_r3128() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
parent.addEventListener('click',e=>{log.push('cap');e.stopPropagation();},true);\
child.addEventListener('click',e=>{log.push('target');});\
child.dispatchEvent(new Event('click',{bubbles:true}));\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "cap",
        "capture 阶段 stopPropagation 止 target+bubble"
    );
}

/// useCapture 两种形式：bool `true` + `{capture:true}` options 对象——均注册 capture 监听器。
#[test]
fn native_dispatch_event_capture_usecapture_forms_r3128() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
parent.addEventListener('click',e=>{log.push('cap-bool');},true);\
parent.addEventListener('click',e=>{log.push('cap-obj');},{capture:true});\
child.addEventListener('click',e=>{log.push('target');});\
child.dispatchEvent(new Event('click',{bubbles:true}));\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "cap-bool,cap-obj,target",
        "useCapture bool + options 形式均注册 capture 监听器（capture 阶段先于 target）"
    );
}

// ── R3135 target 阶段注册序派发（闭合 R3128 限制①）──

/// target 阶段（AT_TARGET）监听器按**注册序**触发，不论 capture 标志（spec `dom-eventtarget-dispatch-event`
/// invoke：AT_TARGET 触发全部，注册序）。闭合 R3128 限制①——旧实现 capture 桶先/bubble 桶后（B,A,C），
/// 现 A,B,C（注册序：bubble-A, capture-B, bubble-C 交错）。
#[test]
fn native_dispatch_event_target_registration_order_r3135() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const el=__zw_native_element_for_id('a');\
const log=[];\
el.addEventListener('click',()=>log.push('A'));\
el.addEventListener('click',()=>log.push('B'),true);\
el.addEventListener('click',()=>log.push('C'));\
el.dispatchEvent({type:'click'});\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "A,B,C",
        "target 阶段监听器按注册序触发（A,B,C），非旧 capture-桶先（B,A,C）——闭合 R3128 限制①"
    );
}

/// target 阶段 stopImmediatePropagation 止当前节点剩余（注册序中后续）——验证注册序单循环下
/// stop-immediate 仍立即终止当前节点剩余监听器（A 触发后 stop-immediate → B,C 不触发）。
#[test]
fn native_dispatch_event_target_stop_immediate_registration_order_r3135() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const el=__zw_native_element_for_id('a');\
const log=[];\
el.addEventListener('click',()=>log.push('A'));\
el.addEventListener('click',(e)=>{log.push('B');e.stopImmediatePropagation();},true);\
el.addEventListener('click',()=>log.push('C'));\
el.dispatchEvent({type:'click'});\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "A,B",
        "target 注册序 A,B 触发后 stopImmediatePropagation 止 C（注册序单循环下立即终止）"
    );
}

// ── R3129 MouseEvent / KeyboardEvent 构造器（extends Event）──

/// `new MouseEvent(type, {clientX,clientY,altKey,bubbles})`：instanceof MouseEvent + Event 成立 +
/// 坐标/修饰键读 init dict（缺省 0/false）+ Event 基属性（type/bubbles）。
#[test]
fn native_mouse_event_constructor_r3129() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const e=new MouseEvent('click',{clientX:10,clientY:20,altKey:true,bubbles:true});\
return (e instanceof MouseEvent)+'/'+(e instanceof Event)+'/'+e.type+'/'+\
e.clientX+'/'+e.clientY+'/'+e.altKey+'/'+e.shiftKey+'/'+e.bubbles+'/'+\
(e.button===0)+'/'+(e.relatedTarget===null);})()";
    assert_eq!(
        run_script(html, script),
        "true/true/click/10/20/true/false/true/true/true",
        "new MouseEvent：instanceof MouseEvent/Event + 坐标/修饰键 + 缺省"
    );
}

/// `new KeyboardEvent(type, {key,code,ctrlKey})`：instanceof KeyboardEvent + Event 成立 +
/// key/code/修饰键读 init dict（缺省 ""/false）。
#[test]
fn native_keyboard_event_constructor_r3129() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const e=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',ctrlKey:true});\
return (e instanceof KeyboardEvent)+'/'+(e instanceof Event)+'/'+e.type+'/'+\
e.key+'/'+e.code+'/'+e.ctrlKey+'/'+e.shiftKey+'/'+e.repeat+'/'+(e.keyCode===0);})()";
    assert_eq!(
        run_script(html, script),
        "true/true/keydown/Enter/Enter/true/false/false/true",
        "new KeyboardEvent：instanceof KeyboardEvent/Event + key/code/修饰键 + 缺省"
    );
}

/// js-dom M4 R25：native MouseEvent/KeyboardEvent UIEvent.`view` + KeyboardEvent.`which` 补全。
/// 旧实现缺 view（MouseEvent/KeyboardEvent extends UIEvent，WPT Event-subclasses-constructors assert_props
/// 父链检查 `'view' in event` fail）+ KeyboardEvent.which（legacy 属性 = keyCode）。修：set_ui_view 设 view
/// （缺省 null，init dict 对象原样），KeyboardEvent 设 which（缺省回退 keyCode）。
#[test]
fn native_event_view_and_which_r25() {
    let html = r#"<div id="a"></div>"#;
    // MouseEvent view：缺省 null（'view' in = true）+ init dict window 原样。
    let script_m = "(()=>{\
const m=new MouseEvent('click');\
const m2=new MouseEvent('click',{view:globalThis});\
return ('view' in m)+'/'+(m.view===null)+'/'+(m2.view===globalThis);})()";
    assert_eq!(
        run_script(html, script_m),
        "true/true/true",
        "R25 native MouseEvent view（缺省 null + init dict window）"
    );
    // KeyboardEvent view（同 UIEvent 父链）+ which（缺省 0，= keyCode 回退）。
    let script_k = "(()=>{\
const k=new KeyboardEvent('keydown');\
const k2=new KeyboardEvent('keydown',{keyCode:13,which:42});\
return ('view' in k)+'/'+(k.view===null)+'/'+('which' in k)+'/'+k.which+'/'+k2.which;})()";
    assert_eq!(
        run_script(html, script_k),
        "true/true/true/0/42",
        "R25 native KeyboardEvent view（父链）+ which（缺省 0 回退 keyCode，init dict 显式）"
    );
}

/// MouseEvent 实例经继承原型具 stopPropagation——派发时止上溯（原型链可达，非每实例注入）。
#[test]
fn native_mouse_event_inherited_stop_r3129() {
    let html = r#"<div id="parent"><div id="child"></div></div>"#;
    let script = "(()=>{\
const child=__zw_native_element_for_id('child');\
const parent=__zw_native_element_for_id('parent');\
const log=[];\
child.addEventListener('mousedown',e=>{log.push('child');e.stopPropagation();});\
parent.addEventListener('mousedown',e=>{log.push('parent');});\
child.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));\
return log.join(',');})()";
    assert_eq!(
        run_script(html, script),
        "child",
        "MouseEvent 继承 Event 原型 stopPropagation 止上溯"
    );
}

// ── R3130 preventDefault → dispatchEvent 返值语义 ──

/// cancelable 事件 + 监听器 preventDefault → dispatchEvent 返 false（spec：cancelable 被阻止）。
#[test]
fn native_dispatch_event_prevent_default_returns_false_r3130() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const el=__zw_native_element_for_id('a');\
el.addEventListener('e',e=>{e.preventDefault();});\
return el.dispatchEvent(new Event('e',{cancelable:true}));})()";
    assert_eq!(
        run_script(html, script),
        "false",
        "cancelable 事件被 preventDefault → dispatchEvent 返 false"
    );
}

/// cancelable 事件无 preventDefault → 返 true；非 cancelable 事件 + preventDefault → 返 true
///（preventDefault 对非 cancelable no-op，不影响返值）。
#[test]
fn native_dispatch_event_return_value_matrix_r3130() {
    let html = r#"<div id="a"></div>"#;
    // cancelable 无 preventDefault → true。
    let no_prevent = "(()=>{const el=__zw_native_element_for_id('a');\
el.dispatchEvent(new Event('e',{cancelable:true}));\
return el.dispatchEvent(new Event('e2',{cancelable:true}));})()";
    assert_eq!(
        run_script(html, no_prevent),
        "true",
        "cancelable 无 preventDefault → true"
    );
    // 非 cancelable + preventDefault → true（preventDefault no-op）。
    let non_cancel = "(()=>{const el=__zw_native_element_for_id('a');\
el.addEventListener('e',e=>{e.preventDefault();});\
return el.dispatchEvent(new Event('e'));})()"; // cancelable 缺省 false
    assert_eq!(
        run_script(html, non_cancel),
        "true",
        "非 cancelable + preventDefault → true"
    );
}

// ── R3131 createTextNode / createComment / createDocumentFragment 工厂 ──

/// createTextNode：nodeType=3 + textContent 读 + nodeName=#text。
#[test]
fn native_create_text_node_r3131() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const t=__zw_native_create_text_node('hello');\
return t.nodeType+'/'+t.nodeName+'/'+t.textContent;})()";
    assert_eq!(run_script(html, script), "3/#text/hello", "createTextNode nodeType=3");
}

/// createComment：nodeType=8 + nodeName=#comment + textContent。
#[test]
fn native_create_comment_r3131() {
    let html = r#"<div id="a"></div>"#;
    let script = "(()=>{\
const c=__zw_native_create_comment('note');\
return c.nodeType+'/'+c.nodeName+'/'+c.textContent;})()";
    assert_eq!(run_script(html, script), "8/#comment/note", "createComment nodeType=8");
}

/// createDocumentFragment：nodeType=11 + nodeName=#document-fragment + 作为容器接收 appendChild
///（fragment 持有其子节点）。注：`host.appendChild(frag)` 的 flatten 语义（子节点展开进 host、fragment 清空）
/// 是 native appendChild 的独立限制（polyfill 路径经 move_fragment_children 处理），后续切片。
#[test]
fn native_create_document_fragment_r3131() {
    let html = r#"<div id="host"></div>"#;
    // fragment 接两子 → fragment.childNodes.length===2（fragment 作容器）。
    let script = "(()=>{\
const frag=__zw_native_create_document_fragment();\
frag.appendChild(__zw_native_create_element('span'));\
frag.appendChild(__zw_native_create_element('b'));\
return frag.nodeType+'/'+frag.nodeName+'/'+frag.childNodes.length;})()";
    assert_eq!(
        run_script(html, script),
        "11/#document-fragment/2",
        "createDocumentFragment nodeType=11 + 作容器持子节点"
    );
}

/// 完整 native 树构建：div > (b > "text")，全 native 无 polyfill。
#[test]
fn native_build_tree_native_r3131() {
    let html = r#"<div id="host"></div>"#;
    let script = "(()=>{\
const host=__zw_native_element_for_id('host');\
const div=__zw_native_create_element('div');\
const b=__zw_native_create_element('b');\
b.appendChild(__zw_native_create_text_node('hi'));\
div.appendChild(b);\
host.appendChild(div);\
return host.children[0].tagName+'/'+host.children[0].children[0].tagName+'/'+\
host.children[0].children[0].textContent;})()";
    assert_eq!(run_script(html, script), "DIV/B/hi", "全 native 树构建：div > b > 'hi'");
}

// ── P1b S5a（R3264）：原生 HTMLElement 构造器 + JS class extends 子类化 ──────────────
// 验证 `new HTMLElement()` + `class X extends HTMLElement`：ctor 建新元素 + 填 slot[0]=NodeId（生产
// DOM 源，非 PoC fixed 42）+ nodeType accessor 经 slot 读 DOM（subclass 经 super() 继承 slot——R3262 验证）。

/// S5a：`new HTMLElement()` 建新元素，instanceof HTMLElement + nodeType=1（ELEMENT_NODE，经 slot 读 DOM）。
#[test]
fn native_html_element_base_ctor_r3264() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
const el = new HTMLElement();\
return (el instanceof HTMLElement) + '/' + el.nodeType;\
})()";
    // instanceof HTMLElement=true / nodeType=1（ELEMENT_NODE，slot[0] 经 ctor 填 + accessor 读 DOM）。
    assert_eq!(
        run_script(html, script),
        "true/1",
        "new HTMLElement() instanceof HTMLElement + nodeType=1（ELEMENT_NODE）"
    );
}

/// S5a：`class X extends HTMLElement` 子类化——new X() instanceof X+HTMLElement + nodeType=1（slot 经 super() 继承）。
/// 闭合 R3262 PoC 接生产 DOM 源：subclass 实例的 native nodeType accessor 经 instance_template holder=实例
/// 读 slot[0]=NodeId（super() 调 native ctor 填）→ DOM node_type=1。
#[test]
fn native_html_element_subclass_r3264() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
class MyEl extends HTMLElement { constructor() { super(); this.__subRan = true; } }\
const inst = new MyEl();\
return (inst instanceof MyEl) + '/' + (inst instanceof HTMLElement) + '/' + inst.nodeType + '/' + inst.__subRan;\
})()";
    // instanceof MyEl=true / instanceof HTMLElement=true / nodeType=1 / __subRan=true（子类 ctor 执行）。
    assert_eq!(
        run_script(html, script),
        "true/true/1/true",
        "class X extends HTMLElement 子类化：instanceof 双 true + nodeType=1（slot 经 super() 继承）+ 子类 ctor 执行"
    );
}

// ── P1b S5b（R3265）：customElements upgrade——document.createElement('my-el') → native custom 实例 ──
// 验证 native_dom 路径 createElement 命中 registry 时 Reflect.construct registered ctor，super() 经
// thread-local upgrade_node_id 复用 host NodeId 填 slot[0]，产出 instanceof registeredCtor + nodeType=1 的
// native 实例。registry 反查经 polyfill 全局 __zw_native_ce_lookup(tag)（测试内联模拟 polyfill 行为）。

/// S5b：`document.createElement('my-el')` 命中 registry → upgrade native 实例。
/// instanceof registered ctor + nodeType=1（super() 复用 host NodeId 经 slot 读 DOM）+ 子类 ctor 执行。
#[test]
fn native_custom_element_upgrade_r3265() {
    let html = r#"<html><body></body></html>"#;
    // 内联模拟 polyfill customElements registry + __zw_native_ce_lookup（真实 shim 在 part03.js 注册）。
    // native_dom 路径 document.createElement / __zw_native_create_element 经 Rust
    // native_create_element_invoke 反查此 lookup；命中则 Reflect.construct registered ctor（super() 复用
    // thread-local host NodeId）。直接测 __zw_native_create_element（S5b 改动入口，绕过 document 对象创建）。
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]=c; }, get: function(n){ return _ce[n]; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] || null; };\
class MyEl extends HTMLElement { constructor() { super(); this.__upgraded = true; } }\
customElements.define('my-el', MyEl);\
const e = __zw_native_create_element('my-el');\
return (e instanceof MyEl) + '/' + (e instanceof HTMLElement) + '/' + e.nodeType + '/' + e.__upgraded;\
})()";
    // instanceof MyEl=true / instanceof HTMLElement=true / nodeType=1（slot 经 super() 复用 host NodeId）
    // / __upgraded=true（registered ctor 执行，this 是 native 实例）。
    assert_eq!(
        run_script(html, script),
        "true/true/1/true",
        "createElement('my-el') upgrade：instanceof registered ctor + HTMLElement + nodeType=1 + ctor 执行"
    );
}

/// S5b：未注册 tag（普通元素）走 generic Element 路径，不被误 upgrade；registry lookup 缺失 graceful 回退。
#[test]
fn native_custom_element_no_upgrade_unregistered_r3265() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]=c; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] || null; };\
const div = __zw_native_create_element('div');\
return (div instanceof HTMLElement) + '/' + div.nodeType + '/' + div.tagName;\
})()";
    // 普通 div：instanceof HTMLElement=false（generic Element 模板，非 HTMLElement 子类）/ nodeType=1 / tag DIV。
    // 未命中 registry → 不 upgrade，回退 generic 路径。
    assert_eq!(
        run_script(html, script),
        "false/1/DIV",
        "未注册 tag 不 upgrade：generic Element 路径，nodeType=1 + tagName=DIV"
    );
}

/// S5b：registry lookup 缺失（polyfill 未加载 / native_dom 关闭 shim）graceful 回退，不抛。
#[test]
fn native_custom_element_no_lookup_graceful_r3265() {
    let html = r#"<html><body></body></html>"#;
    // 不注册 __zw_native_ce_lookup（模拟 shim 未加载）→ native_create_element_invoke 反查失败 → 回退 generic。
    let script = "(()=>{\
const div = __zw_native_create_element('section');\
return div.nodeType + '/' + div.tagName;\
})()";
    assert_eq!(
        run_script(html, script),
        "1/SECTION",
        "lookup 缺失 graceful 回退：generic Element，nodeType=1 + tagName=SECTION（不抛）"
    );
}

// ── P1b S5c（R3266）：custom element lifecycle——connectedCallback/disconnectedCallback ──
// 验证 native_dom 路径 appendChild/removeChild 经 Rust custom_elements 模块桥接 polyfill
// `__zw_native_ce_notify_connect` 派发 connectedCallback/disconnectedCallback（this=native 实例）。
// 内联模拟 polyfill registry + lookup + notify_connect（真实 shim 在 part03.js 注册）。

/// S5c：appendChild(customEl) 到已连接容器（document root）→ connectedCallback 触发，this 是 native 实例。
#[test]
fn native_custom_element_connected_callback_r3266() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _log = [];\
globalThis.__zw_native_ce_notify_connect = function(instances, connected, tags){\
  for (var i=0;i<instances.length;i++){\
    var entry = _ce[tags[i]];\
    if (!entry) continue;\
    var proto = entry.ctor.prototype;\
    var cb = connected ? proto.connectedCallback : proto.disconnectedCallback;\
    if (typeof cb === 'function') { try { cb.call(instances[i]); } catch(_e){} }\
  }\
};\
class MyEl extends HTMLElement { constructor(){ super(); } connectedCallback(){ this.__conn = true; _log.push('conn:'+this.nodeType); } }\
customElements.define('my-el', MyEl);\
const body = __zw_native_get_body();\
const el = __zw_native_create_element('my-el');\
body.appendChild(el);\
return _log.join(',') + '/' + (el.__conn === true);\
})()";
    // connectedCallback 触发：_log='conn:1'（this=native 实例 nodeType=1）+ el.__conn=true（this 写到 native 实例）。
    assert_eq!(
        run_script(html, script),
        "conn:1/true",
        "appendChild(customEl) 到已连接容器：connectedCallback 触发，this=native 实例（nodeType=1 + 可写属性）"
    );
}

/// S5c：appendChild 到 detached 容器（createElement 后未挂 document）→ 不触发 connectedCallback。
#[test]
fn native_custom_element_no_connect_detached_r3266() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var connCount = 0;\
globalThis.__zw_native_ce_notify_connect = function(instances, connected, tags){\
  for (var i=0;i<instances.length;i++){ if (connected && _ce[tags[i]]) connCount++; }\
};\
class Detached extends HTMLElement { constructor(){ super(); } connectedCallback(){ connCount++; } }\
customElements.define('det-el', Detached);\
const container = __zw_native_create_element('div');\
const el = __zw_native_create_element('det-el');\
container.appendChild(el);\
return connCount;\
})()";
    // detached 容器（container 未挂 document）→ el 连接态未变 → connectedCallback 不触发 → connCount=0。
    assert_eq!(
        run_script(html, script),
        "0",
        "appendChild 到 detached 容器不触发 connectedCallback（parent 链未到 document root）"
    );
}

/// S5c：removeChild(customEl) 从已连接容器 → disconnectedCallback 触发。
#[test]
fn native_custom_element_disconnected_callback_r3266() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _log = [];\
globalThis.__zw_native_ce_notify_connect = function(instances, connected, tags){\
  for (var i=0;i<instances.length;i++){\
    var entry = _ce[tags[i]];\
    if (!entry) continue;\
    var proto = entry.ctor.prototype;\
    var cb = connected ? proto.connectedCallback : proto.disconnectedCallback;\
    if (typeof cb === 'function') { try { cb.call(instances[i]); } catch(_e){} }\
  }\
};\
class Rem extends HTMLElement { constructor(){ super(); } connectedCallback(){ _log.push('c'); } disconnectedCallback(){ _log.push('d'); } }\
customElements.define('rem-el', Rem);\
const body = __zw_native_get_body();\
const el = __zw_native_create_element('rem-el');\
body.appendChild(el);\
body.removeChild(el);\
return _log.join('');
})()";
    // body.appendChild → connectedCallback('c') → body.removeChild → disconnectedCallback('d') → _log='cd'。
    assert_eq!(
        run_script(html, script),
        "cd",
        "removeChild 从已连接 body 触发 disconnectedCallback（先 connected 后 disconnected = 'cd'）"
    );
}

// ── P1b S5d（R3267）：attributeChangedCallback——native setAttribute/removeAttribute 桥接派发 ──
// 验证 native_dom 路径 setAttribute/removeAttribute 经 Rust custom_elements 模块桥接 polyfill
// `__zw_native_ce_notify_attr_change` → `_ce_dispatchAttrChange`（observedAttributes 过滤 + 值真变 + 调
// ctor.prototype.attributeChangedCallback，this=native 实例）。内联模拟 polyfill registry + dispatch。

/// S5d：setAttribute(observed attr) → attributeChangedCallback 触发，收 (name, old=null, new=value)，this=native 实例。
#[test]
fn native_custom_element_attr_change_r3267() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _log = [];\
function _dispatch(entry, inst, name, oldV, newV){\
  var obs = entry.ctor.observedAttributes;\
  if (!obs) return;\
  var matched = false;\
  for (var i=0;i<obs.length;i++){ if (String(obs[i]).toLowerCase()===String(name).toLowerCase()){ matched=true; break; } }\
  if (!matched) return;\
  var o = oldV==null?'':String(oldV); var n = newV==null?'':String(newV);\
  if (o===n) return;\
  var cb = entry.ctor.prototype.attributeChangedCallback;\
  if (typeof cb==='function'){ try{ cb.call(inst, String(name), oldV, newV); }catch(_e){} }\
}\
globalThis.__zw_native_ce_notify_attr_change = function(inst, name, oldV, newV, tag){\
  var entry = _ce[String(tag).toLowerCase()]; if (!entry) return; _dispatch(entry, inst, name, oldV, newV);\
};\
class AttrEl extends HTMLElement {\
  static get observedAttributes(){ return ['foo']; }\
  constructor(){ super(); }\
  attributeChangedCallback(n, o, v){ _log.push(n+'/'+o+'/'+v+'/'+this.nodeType); }\
}\
customElements.define('attr-el', AttrEl);\
const el = __zw_native_create_element('attr-el');\
el.setAttribute('foo', 'bar');\
return _log.join(',');
})()";
    // setAttribute('foo','bar')：foo ∈ observedAttributes，old=null（首次）→ attributeChangedCallback('foo/null/bar/1')。
    assert_eq!(
        run_script(html, script),
        "foo/null/bar/1",
        "setAttribute(observed attr) 触发 attributeChangedCallback：name=foo / old=null / new=bar / this=nodeType 1"
    );
}

/// S5d：setAttribute(未 observed 的 attr) → 不触发 attributeChangedCallback（observedAttributes 过滤）。
#[test]
fn native_custom_element_attr_change_unobserved_r3267() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var calls = 0;\
function _dispatch(entry, inst, name, oldV, newV){\
  var obs = entry.ctor.observedAttributes; if (!obs) return;\
  for (var i=0;i<obs.length;i++){ if (String(obs[i]).toLowerCase()===String(name).toLowerCase()){ calls++; return; } }\
}\
globalThis.__zw_native_ce_notify_attr_change = function(inst, name, oldV, newV, tag){\
  var entry = _ce[String(tag).toLowerCase()]; if (entry) _dispatch(entry, inst, name, oldV, newV);\
};\
class U extends HTMLElement {\
  static get observedAttributes(){ return ['foo']; }\
  constructor(){ super(); }\
  attributeChangedCallback(){ calls++; }\
}\
customElements.define('u-el', U);\
const el = __zw_native_create_element('u-el');\
el.setAttribute('baz', 'qux');\
return calls;\
})()";
    // 'baz' ∉ observedAttributes(['foo']) → 过滤 → 不触发 → calls=0。
    assert_eq!(
        run_script(html, script),
        "0",
        "setAttribute(未 observed attr) 不触发 attributeChangedCallback（observedAttributes 过滤）"
    );
}

/// S5d：removeAttribute(observed attr) → attributeChangedCallback 触发，newVal=null。
#[test]
fn native_custom_element_attr_change_remove_r3267() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _log = [];\
function _dispatch(entry, inst, name, oldV, newV){\
  var obs = entry.ctor.observedAttributes; if (!obs) return;\
  var matched = false;\
  for (var i=0;i<obs.length;i++){ if (String(obs[i]).toLowerCase()===String(name).toLowerCase()){ matched=true; break; } }\
  if (!matched) return;\
  var o = oldV==null?'':String(oldV); var n = newV==null?'':String(newV);\
  if (o===n) return;\
  var cb = entry.ctor.prototype.attributeChangedCallback;\
  if (typeof cb==='function'){ try{ cb.call(inst, String(name), oldV, newV); }catch(_e){} }\
}\
globalThis.__zw_native_ce_notify_attr_change = function(inst, name, oldV, newV, tag){\
  var entry = _ce[String(tag).toLowerCase()]; if (!entry) return; _dispatch(entry, inst, name, oldV, newV);\
};\
class R extends HTMLElement {\
  static get observedAttributes(){ return ['foo']; }\
  constructor(){ super(); }\
  attributeChangedCallback(n, o, v){ _log.push(n+'/'+o+'/'+v); }\
}\
customElements.define('r-el', R);\
const el = __zw_native_create_element('r-el');\
el.setAttribute('foo', '1');\
el.removeAttribute('foo');\
return _log.join(',');
})()";
    // setAttribute('foo','1') → ('foo/null/1')；removeAttribute('foo') → ('foo/1/null')。
    assert_eq!(
        run_script(html, script),
        "foo/null/1,foo/1/null",
        "setAttribute + removeAttribute(observed)：两次 attributeChangedCallback，remove 时 old='1'/new=null"
    );
}

// ── P1b S5 剩余后续（R3269）：customElements.upgrade PoC——setPrototypeOf 方案可行性验证 ──
// upgrade 需把既有 native 元素（parser 建 / createElement 建未注册）变成 registered ctor 实例（保留 NodeId）。
// PoC 验证 Object.setPrototypeOf(nativeEl, registeredCtor.prototype) 是否让 native 元素 instanceof registeredCtor
// 且保留 slot[0]=NodeId（nodeType accessor 仍可读）。
#[test]
fn native_custom_element_upgrade_set_proto_poc_r3269() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
class MyEl extends HTMLElement { constructor(){ super(); } connectedCallback(){ this.__conn = true; } }\
// define 前建元素（模拟 parser 建或 createElement 未注册）——此时未 upgrade，是普通 native 元素
const el = __zw_native_create_element('my-el');\
const beforeInstanceof = (el instanceof MyEl);\
const beforeNodeType = el.nodeType;\
// define 后 setPrototypeOf 升级
customElements.define('my-el', MyEl);\
Object.setPrototypeOf(el, MyEl.prototype);\
const afterInstanceof = (el instanceof MyEl);\
const afterNodeType = el.nodeType;\
return beforeInstanceof + '/' + beforeNodeType + '/' + afterInstanceof + '/' + afterNodeType;\
})()";
    // setPrototypeOf 前：instanceof MyEl=false（普通 native 元素）+ nodeType=1
    // setPrototypeOf 后：instanceof MyEl=true（prototype 切换）+ nodeType=1（slot[0]=NodeId 保留，accessor 仍可读）。
    assert_eq!(
        run_script(html, script),
        "false/1/true/1",
        "upgrade PoC：setPrototypeOf 把 native 元素升级为 registered ctor 实例，保留 slot NodeId"
    );
}

/// S5 后续（R3269）：customElements.upgrade(root) 子树升级——既有元素 setPrototypeOf + connectedCallback。
#[test]
fn native_custom_element_upgrade_subtree_r3269() {
    let html = r#"<html><body></body></html>"#;
    // 内联模拟 polyfill upgrade（真实 shim 在 part03.js _ceUpgradeNode）：DFS setPrototypeOf + connectedCallback。
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _conn = 0;\
class MyEl extends HTMLElement { constructor(){ super(); } connectedCallback(){ _conn++; this.__c = true; } }\
// 建子树：container > my-el（未注册时建，普通 native 元素），挂 body（connected）
const body = __zw_native_get_body();\
const container = __zw_native_create_element('div');\
const el = __zw_native_create_element('my-el');\
container.appendChild(el);\
body.appendChild(container);\
// define 后 upgrade(container)——子树 DFS 应升级 my-el + 触发 connectedCallback（已连 body）
customElements.define('my-el', MyEl);\
function upgrade(node){\
  var tag = node.tagName ? String(node.tagName).toLowerCase() : '';\
  var entry = _ce[tag];\
  if (entry && entry.ctor){\
    Object.setPrototypeOf(node, entry.ctor.prototype);\
    var ccb = entry.ctor.prototype.connectedCallback;\
    if (typeof ccb === 'function'){ try { ccb.call(node); } catch(_e){} }\
  }\
  var c = node.firstChild;\
  while (c){ var n = c.nextSibling; upgrade(c); c = n; }\
}\
upgrade(container);\
return (el instanceof MyEl) + '/' + el.nodeType + '/' + _conn;\
})()";
    // upgrade(container) DFS：container（div，非 custom，跳过）→ my-el（setPrototypeOf 升级 + connectedCallback，
    // 已连 body）→ instanceof MyEl=true + nodeType=1（slot 保留）+ _conn=1（connectedCallback 触发一次）。
    assert_eq!(
        run_script(html, script),
        "true/1/1",
        "upgrade(root) 子树 DFS：既有 native 元素 setPrototypeOf 升级 + connectedCallback（已连 document）"
    );
}

// ── P1b S5 后续（R3268）：HTMLElement Element/Node 接口补全——custom 实例完整 Element API ──
// 验证 native custom 实例（instanceof registered ctor）具备与 generic Element 模板等价的全套
// Element/Node 接口（S5d 仅补 attr+tree-mutation 族，本轮补全查询/内容/导航/子元素/反射属性/复杂对象 getter）。

/// S5 接口补全：custom 实例的查询/内容/导航/子元素接口（innerHTML/textContent/querySelector/children/导航）。
#[test]
fn native_html_element_full_interface_query_content_r3268() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
class MyEl extends HTMLElement { constructor(){ super(); } }\
customElements.define('my-el', MyEl);\
const body = __zw_native_get_body();\
const el = __zw_native_create_element('my-el');\
el.innerHTML = '<b>hi</b><i>x</i>';\
body.appendChild(el);\
return el.innerHTML + '/' + el.textContent + '/' + el.children.length + '/' + el.firstChild.tagName + '/' + el.parentNode.tagName;\
})()";
    // innerHTML='<b>hi</b><i>x</i>'（序列化回读）+ textContent='hix'（两子文本拼接）
    // + children.length=2（b+i）+ firstChild.tagName=B（首个元素子）+ parentNode.tagName=BODY（已挂 body）。
    assert_eq!(
        run_script(html, script),
        "<b>hi</b><i>x</i>/hix/2/B/BODY",
        "custom 实例 Element 接口：innerHTML setter/getter + textContent + children + firstChild + parentNode"
    );
}

/// S5 接口补全：custom 实例的反射属性（id/className/tagName/tabIndex）+ ARIA + 复杂对象 getter（dataset）。
#[test]
fn native_html_element_full_interface_reflected_dataset_r3268() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
class MyEl extends HTMLElement { constructor(){ super(); } }\
customElements.define('my-el', MyEl);\
const el = __zw_native_create_element('my-el');\
el.id = 'foo';\
el.className = 'a b';\
el.dataset.key = 'val';\
el.ariaLabel = 'lbl';\
return el.tagName + '/' + el.id + '/' + el.className + '/' + el.dataset.key + '/' + el.ariaLabel;\
})()";
    // tagName=MY-EL（custom tag 大写）+ id='foo'（反射 setter→getter）+ className='a b'
    // + dataset.key='val'（data-key 反射）+ ariaLabel='lbl'（aria-label 反射）。
    assert_eq!(
        run_script(html, script),
        "MY-EL/foo/a b/val/lbl",
        "custom 实例反射属性 + dataset + ARIA：tagName/id/className/dataset.key/ariaLabel"
    );
}

/// S5 接口补全：custom 实例 querySelector + cloneNode + matches（查询/克隆/匹配接口）。
#[test]
fn native_html_element_full_interface_query_clone_r3268() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
class MyEl extends HTMLElement { constructor(){ super(); } }\
customElements.define('my-el', MyEl);\
const el = __zw_native_create_element('my-el');\
el.innerHTML = '<p class=x>1</p><p>2</p>';\
const found = el.querySelector('.x');\
const clone = el.cloneNode(false);\
return (found ? found.textContent : 'null') + '/' + el.matches('my-el') + '/' + clone.children.length;\
})()";
    // querySelector('.x') 命中首个 p.textContent='1' + matches('my-el')=true（custom tag 匹配）
    // + cloneNode(false) 浅克隆 children.length=0（无子）。
    assert_eq!(
        run_script(html, script),
        "1/true/0",
        "custom 实例 querySelector + matches + cloneNode（查询/匹配/克隆接口）"
    );
}

// ── P1b S5 性能（R3271）：attr change / lifecycle 桥接 fast-path——非 custom tag 跳过 JS 桥接 ──
// custom element 名规范要求含连字符（[a-z]+-[a-z-]+）。无连字符的 tag（div/span/p 等原生 HTML 元素）必非
// custom → Rust 侧 fast-path 直接跳过 JS 桥接（避免非 custom 元素 setAttribute/appendChild 的无谓 JS 调用
// + native 实例构造 + 数组创建）。框架 reconciliation 高频操作普通元素受益。本测试锁定该优化行为（防回归）。

/// R3271 fast-path：非 custom tag（div，无连字符）的 setAttribute 不触发 CE attr change 桥接。
#[test]
fn native_custom_element_fast_path_non_custom_attr_r3271() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _notifyCalls = 0;\
globalThis.__zw_native_ce_notify_attr_change = function(){ _notifyCalls++; };\
const div = __zw_native_create_element('div');\
div.setAttribute('class', 'x');\
div.setAttribute('data-foo', 'bar');\
return _notifyCalls;\
})()";
    // div（无连字符）fast-path 跳过 → __zw_native_ce_notify_attr_change 未被调 → _notifyCalls=0。
    assert_eq!(
        run_script(html, script),
        "0",
        "fast-path：div（无连字符 tag）setAttribute 不触发 CE attr change JS 桥接"
    );
}

/// R3271 fast-path：非 custom tag 的 appendChild 不触发 CE connect 桥接（子树 DFS 过滤无连字符 tag）。
#[test]
fn native_custom_element_fast_path_non_custom_connect_r3271() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _notifyCalls = 0;\
globalThis.__zw_native_ce_notify_connect = function(){ _notifyCalls++; };\
const body = __zw_native_get_body();\
const div = __zw_native_create_element('div');\
const span = __zw_native_create_element('span');\
div.appendChild(span);\
body.appendChild(div);\
return _notifyCalls;\
})()";
    // div/span（无连字符）fast-path 跳过 → __zw_native_ce_notify_connect 未被调 → _notifyCalls=0。
    assert_eq!(
        run_script(html, script),
        "0",
        "fast-path：div/span（无连字符 tag）appendChild 不触发 CE connect JS 桥接"
    );
}

// ── R3272：custom element lifecycle 派发顺序与嵌套 upgrade 安全性 ──

/// R3272 pre-order tree order：appendChild 一个含多个 custom 元素的子树时，connectedCallback
/// 必须按 **pre-order tree order**（根优先，子节点 left→right）触发——与浏览器 spec 一致。
/// 旧实现 DFS 子节点正序压栈 + LIFO pop = reverse tree order（右孩子先），违反 spec；R3272 逆序压栈修正。
/// 子树结构：root(parent-el) → child-a(child-el) + child-b(child-el)；parent 先连，再 a，再 b。
#[test]
fn native_custom_element_connect_preorder_tree_order_r3272() {
    let html = r#"<html><body></body></html>"#;
    // notify 收到的 instances 数组顺序 = collect_custom_subtree 的 DFS 访问序（= 连接态变化收集序，
    // = connectedCallback 触发序）。每实例打 dataset.order，连接时 push 其 order，验 pre-order。
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
var _log = [];\
globalThis.__zw_native_ce_notify_connect = function(instances, connected, tags){\
  if (!connected) return;\
  for (var i=0;i<instances.length;i++){ _log.push(instances[i].dataset.order); }\
};\
class PEl extends HTMLElement { constructor(){ super(); } }\
class CEl extends HTMLElement { constructor(){ super(); } }\
customElements.define('parent-el', PEl);\
customElements.define('child-el', CEl);\
const root = __zw_native_create_element('parent-el'); root.dataset.order='P';\
const a = __zw_native_create_element('child-el'); a.dataset.order='A';\
const b = __zw_native_create_element('child-el'); b.dataset.order='B';\
root.appendChild(a);\
root.appendChild(b);\
const body = __zw_native_get_body();\
body.appendChild(root);\
return _log.join(',');\
})()";
    // pre-order tree order：root(P) → A → B（左到右）。旧 reverse-tree-order 会得 'P,B,A'。
    assert_eq!(
        run_script(html, script),
        "P,A,B",
        "R3272 pre-order tree order：connectedCallback 按根优先 + 子节点 left→right 触发"
    );
}

/// R3272 嵌套 upgrade 栈隔离：custom ctor body 内再 `createElement` 另一个 custom 元素（嵌套 upgrade）时，
/// 内层 push 的 upgrade NodeId **不得覆盖**外层（外层 super() 仍读外层 NodeId）。旧实现单 `Option<NodeId>`
/// 被 set 覆盖 → 外层 super() 读到内层 NodeId（身份错乱）。栈化后内层 push/pop 隔离外层。
/// Inner ctor 仅记自身 nodeType（不递归 createElement）；Outer ctor body 内 createElement('inner-el')
/// 触发嵌套 upgrade（内层 push/pop upgrade slot），之后验证外层实例 NodeId 仍属自身。
#[test]
fn native_custom_element_nested_upgrade_stack_isolation_r3272() {
    let html = r#"<html><body></body></html>"#;
    let script = "(()=>{\
var _ce = {};\
globalThis.customElements = { define: function(n,c){ _ce[n]={ctor:c}; } };\
globalThis.__zw_native_ce_lookup = function(t){ return _ce[t] ? _ce[t].ctor : null; };\
class Inner extends HTMLElement { constructor(){ super(); this.__innerId = this.nodeType; } }\
customElements.define('inner-el', Inner);\
class Outer extends HTMLElement {\
  constructor(){\
    super();\
    this._inner = __zw_native_create_element('inner-el');\
    this.__outerMarked = (this.nodeType === 1);\
  }\
}\
customElements.define('outer-el', Outer);\
const outer = __zw_native_create_element('outer-el');\
return (outer.__outerMarked === true) + '/' + (outer._inner instanceof Inner) + '/' + (outer._inner.__innerId === 1);\
})()";
    // 外层实例 NodeId 正确（outerMarked=true）+ 内层实例 instanceof Inner + 内层自身 nodeType=1（各自独立 upgrade）。
    assert_eq!(
        run_script(html, script),
        "true/true/true",
        "R3272 嵌套 upgrade 栈隔离：外层 ctor body 内 createElement custom 不破坏外层 upgrade slot"
    );
}

// ── R31 DOMException 构造器 / toString / legacy code 常量（spec webidl#idl-DOMException）──

/// `new DOMException(message)` name 缺省 "Error"、message 透传、code 按名查（Error 无 legacy code=0）。
/// 覆盖 dom_exception.rs `native_dom_exception_constructor_invoke` + `code_for_name` 默认分支（_ => 0）。
#[test]
fn native_dom_exception_constructor_defaults_r31() {
    let html = r#"<html><body></body></html>"#;
    // message 透传 + name 缺省 "Error" + code=0（Error 无 legacy code）。
    assert_eq!(
        run_script(html, "(new DOMException('boom').message)"),
        "boom",
        "DOMException message 透传"
    );
    assert_eq!(
        run_script(html, "(new DOMException('boom').name)"),
        "Error",
        "DOMException name 缺省 Error"
    );
    assert_eq!(
        run_script(html, "(new DOMException('boom').code)"),
        "0",
        "DOMException Error 无 legacy code（code_for_name 默认 _ => 0）"
    );
    // message 缺省空串。
    assert_eq!(
        run_script(html, "(new DOMException().message)"),
        "",
        "DOMException message 缺省空串"
    );
}

/// DOMException name → legacy code 全表（spec error-names-table）。覆盖 dom_exception.rs
/// `code_for_name` 各 match 分支（code 13/14/15/18/19/20/21/23/24/25 + 已测的 1/3/4/5/7/8/9/10/11/12）。
#[test]
fn native_dom_exception_name_to_code_table_r31() {
    let html = r#"<html><body></body></html>"#;
    // (name, expected legacy code)——覆盖全表 + 默认分支。
    let table = [
        ("IndexSizeError", 1),
        ("HierarchyRequestError", 3),
        ("WrongDocumentError", 4),
        ("InvalidCharacterError", 5),
        ("NoModificationAllowedError", 7),
        ("NotFoundError", 8),
        ("NotSupportedError", 9),
        ("InUseAttributeError", 10),
        ("InvalidStateError", 11),
        ("SyntaxError", 12),
        ("InvalidModificationError", 13),
        ("NamespaceError", 14),
        ("InvalidAccessError", 15),
        ("SecurityError", 18),
        ("NetworkError", 19),
        ("AbortError", 20),
        ("URLMismatchError", 21),
        ("TimeoutError", 23),
        ("InvalidNodeTypeError", 24),
        ("DataCloneError", 25),
        // 未列入的 name → 0（spec 允许新 name 无 legacy code）。
        ("UnknownNewError", 0),
    ];
    for (name, code) in table {
        let script = format!("(new DOMException('m', '{name}').code)");
        assert_eq!(
            run_script(html, &script),
            code.to_string(),
            "DOMException name={name} 应映射 legacy code={code}"
        );
    }
}

/// `DOMException.prototype.toString()` → `"name: message"`（spec）；message 空串时仅 name。
/// 覆盖 dom_exception.rs `native_dom_exception_to_string_invoke`（此前几乎无测试覆盖）。
#[test]
fn native_dom_exception_to_string_r31() {
    let html = r#"<html><body></body></html>"#;
    // name + ": " + message。
    assert_eq!(
        run_script(html, "(new DOMException('boom', 'SyntaxError').toString())"),
        "SyntaxError: boom",
        "toString → 'name: message'"
    );
    // message 空串 → 仅 name（spec：message 为空时不加 ': '）。
    assert_eq!(
        run_script(html, "(new DOMException('', 'NotFoundError').toString())"),
        "NotFoundError",
        "toString message 空 → 仅 name"
    );
    // 缺省 name Error + 有 message。
    assert_eq!(
        run_script(html, "(new DOMException('oops').toString())"),
        "Error: oops",
        "toString 缺省 name Error + message"
    );
}

/// DOMException legacy code 常量挂构造器自身（`DOMException.SYNTAX_ERR` 等，兼容旧代码）。
/// 覆盖 dom_exception.rs `register_const` 各分支（code 13/14/15/18/19/20/21/23/24/25 等）。
#[test]
fn native_dom_exception_legacy_constants_r31() {
    let html = r#"<html><body></body></html>"#;
    // 抽样覆盖各 register_const（含此前未触发的 INVALID_MODIFICATION_ERR 等）。
    let consts = [
        ("INDEX_SIZE_ERR", 1),
        ("HIERARCHY_REQUEST_ERR", 3),
        ("INVALID_MODIFICATION_ERR", 13),
        ("NAMESPACE_ERR", 14),
        ("INVALID_ACCESS_ERR", 15),
        ("SECURITY_ERR", 18),
        ("NETWORK_ERR", 19),
        ("ABORT_ERR", 20),
        ("URL_MISMATCH_ERR", 21),
        ("TIMEOUT_ERR", 23),
        ("INVALID_NODE_TYPE_ERR", 24),
        ("DATA_CLONE_ERR", 25),
    ];
    for (cn, code) in consts {
        let script = format!("(DOMException.{cn})");
        assert_eq!(
            run_script(html, &script),
            code.to_string(),
            "DOMException.{cn} legacy 常量 = {code}"
        );
    }
}

/// `instance.constructor === DOMException`（WPT assert_throws_dom 最后一步要求）。
/// 覆盖 dom_exception.rs prototype.constructor set 分支（R6 修复 "wrong global" 的正确性回归）。
#[test]
fn native_dom_exception_constructor_identity_r31() {
    let html = r#"<html><body></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(new DOMException('m', 'SyntaxError').constructor === DOMException)"
        ),
        "true",
        "instance.constructor === DOMException（prototype.constructor 链）"
    );
    // build_and_register 幂等：install_dom_bindings 每次脚本新建 context 仅装一次，但断言构造器
    // 稳定可用（同一 context 内 DOMException 不被重建）。
    assert_eq!(
        run_script(html, "(typeof DOMException)"),
        "function",
        "DOMException 构造器为 function"
    );
}

// ── R31 custom element lifecycle 桥接（connect/disconnect/attr-change 派发路径覆盖）──
//
// custom_elements.rs 的 notify_connect_after_insert / notify_disconnect_after_remove /
// notify_attribute_change 派发到 polyfill `__zw_native_ce_notify_connect` /
// `__zw_native_ce_notify_attr_change`。既有测试仅覆盖 fast-path（非 custom tag 跳过），
// 真派发路径（含连字符 tag + 已注册 notify 函数）此前无测试 → custom_elements.rs 多个
// defensive early-return / branch 未覆盖。本组测试经 JS 注册 notify 记录器，驱动 connect
//（appendChild 到 body）/ attr-change（setAttribute）/ disconnect（removeChild）三路径。

/// custom element 连入 document（body.appendChild）→ connectedCallback 派发；
/// 再 removeChild → disconnectedCallback 派发。覆盖 custom_elements.rs notify_connect_after_insert
/// 的 connect 分支 + notify_disconnect_after_remove + dispatch_connect（含 `__zw_native_ce_notify_connect`
/// 已注册 + pairs 非空 → 真派发）。
#[test]
fn native_custom_element_connect_disconnect_lifecycle_r31() {
    let html = r#"<html><body></body></html>"#;
    let script = r#"(() => {
  var _ce = {};
  globalThis.customElements = { define: function (n, c) { _ce[n] = { ctor: c }; } };
  globalThis.__zw_native_ce_lookup = function (t) { return _ce[t] ? _ce[t].ctor : null; };
  var _log = [];
  globalThis.__zw_native_ce_notify_connect = function (insts, conn, tags) {
    _log.push((conn ? 'connect:' : 'disconnect:') + tags.join(','));
  };
  class MyEl extends HTMLElement { constructor() { super(); } }
  customElements.define('my-el', MyEl);
  const el = __zw_native_create_element('my-el');
  const body = __zw_native_get_body();
  body.appendChild(el);
  body.removeChild(el);
  return _log.join('|');
})()"#;
    assert_eq!(
        run_script(html, script),
        "connect:my-el|disconnect:my-el",
        "custom element appendChild→connect / removeChild→disconnect 派发"
    );
}

/// custom element setAttribute（含连字符 tag + 已注册 notify）→ attributeChangedCallback 派发。
/// 覆盖 custom_elements.rs notify_attribute_change 真派发分支（tag 含连字符过 fast-path +
/// `__zw_native_ce_notify_attr_change` 已注册 + instance/name/old/new/tag 派发）。
#[test]
fn native_custom_element_attribute_change_dispatch_r31() {
    let html = r#"<html><body></body></html>"#;
    let script = r#"(() => {
  var _ce = {};
  globalThis.customElements = { define: function (n, c) { _ce[n] = { ctor: c }; } };
  globalThis.__zw_native_ce_lookup = function (t) { return _ce[t] ? _ce[t].ctor : null; };
  var _log = [];
  globalThis.__zw_native_ce_notify_attr_change = function (inst, name, oldv, newv, tag) {
    _log.push(name + ':' + (oldv === null ? 'null' : oldv) + '->' + (newv === null ? 'null' : newv));
  };
  class MyEl extends HTMLElement { constructor() { super(); } }
  customElements.define('my-el', MyEl);
  const el = __zw_native_create_element('my-el');
  el.setAttribute('foo', 'bar');
  el.setAttribute('foo', 'baz');
  el.removeAttribute('foo');
  return _log.join('|');
})()"#;
    assert_eq!(
        run_script(html, script),
        "foo:null->bar|foo:bar->baz|foo:baz->null",
        "custom element setAttribute/removeAttribute → attr-change 派发（old/new/null 全路径）"
    );
}

/// 非 custom tag（无连字符）setAttribute 不触发 attr-change 桥接（R3271 fast-path）；
/// 含连字符但未注册 CE 的 tag（如 my-unregistered）仍走桥接但 polyfill registry 查无 → 不派发回调。
/// 覆盖 notify_attribute_change 的 fast-path 跳过分支（`!tag.contains('-')` return）。
#[test]
fn native_custom_element_attr_change_fast_path_skip_r31() {
    let html = r#"<html><body></body></html>"#;
    let script = r#"(() => {
  var _log = [];
  globalThis.__zw_native_ce_notify_attr_change = function () { _log.push('called'); };
  const div = __zw_native_create_element('div');
  div.setAttribute('a', 'b');
  return _log.length + '|div-ok:' + div.getAttribute('a');
})()"#;
    // fast-path：notify_attr_change 在 Rust 侧 `!tag.contains('-')` return，_log 为空；setAttribute 正常生效。
    assert_eq!(
        run_script(html, script),
        "0|div-ok:b",
        "非 custom tag（div）setAttribute 走 fast-path 不触发 attr-change 桥接，属性正常设置"
    );
}

// ── R31 CSSStyleDeclaration 优先级 / item 边界 / named-deleter 路径覆盖（spec dom-cssstyledeclaration）──
//
// css_style_declaration.rs：getPropertyPriority（"important" 读取）+ setProperty important upsert 分支
// + item() 越界空串 + named-deleter（`delete el.style.color`）此前无原生测试覆盖。

/// `getPropertyPriority`：important 声明返 "important"，非 important / 未设返 ""；`setProperty(prop,val,'important')`
/// upsert 时设 important 标志。覆盖 css_style_declaration.rs get_property_priority + set_property important 分支。
#[test]
fn native_style_property_priority_r31() {
    let html = r#"<html><body><div id="a"></div></body></html>"#;
    // setProperty 带 'important' → getPropertyPriority 返 "important"。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  el.style.setProperty('color', 'red', 'important');
  return el.style.getPropertyPriority('color');
})()"#
        ),
        "important",
        "setProperty 第 3 参 'important' → getPropertyPriority 返 important"
    );
    // 非重要声明 → ""（priority 第 3 参空）。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  el.style.setProperty('margin', '5px');
  return '[' + el.style.getPropertyPriority('margin') + ']';
})()"#
        ),
        "[]",
        "setProperty 无 priority → getPropertyPriority 返空串"
    );
    // 未设属性 → ""。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  return '[' + el.style.getPropertyPriority('nope') + ']';
})()"#
        ),
        "[]",
        "未设属性 getPropertyPriority 返空串"
    );
    // 重要标志 upsert：已存在声明 → setProperty important 更新值 + 设 important 标志。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  el.style.setProperty('color', 'red');
  el.style.setProperty('color', 'blue', 'important');
  return el.style.getPropertyValue('color') + '/' + el.style.getPropertyPriority('color');
})()"#
        ),
        "blue/important",
        "setProperty important upsert：已存在声明更新值 + 设 important 标志"
    );
}

/// `item(index)` 越界 → 空串（spec dom-cssstyledeclaration-item）；named-deleter
/// （`delete el.style.color`）移除声明（spec IDL deleter）。
#[test]
fn native_style_item_boundary_and_named_deleter_r31() {
    let html = r#"<html><body><div id="a"></div></body></html>"#;
    // item() 越界 → 空串。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  el.style.color = 'red';
  return '[' + el.style.item(5) + ']';
})()"#
        ),
        "[]",
        "item() 越界返空串"
    );
    // 负 index → 空串（integer_value 负 → get(neg as usize) None）。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  return '[' + el.style.item(-1) + ']';
})()"#
        ),
        "[]",
        "item(-1) 负 index 返空串"
    );
    // named-deleter：delete el.style.color 移除声明（spec IDL deleter）。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  el.style.color = 'red';
  el.style.margin = '5px';
  delete el.style.color;
  return el.style.length + '/[' + el.style.color + ']/' + el.style.item(0);
})()"#
        ),
        "1/[]/margin",
        "delete el.style.color（named-deleter）移除 color 声明，剩 margin"
    );
}

// ── R32 Event.srcElement（legacy IE 别名 = target，spec dom-event-srcelement）──

/// `new Event('t')` srcElement 初始 null（dispatch 前 target 未设）；dispatch 期 srcElement === target
///（派发目标）。覆盖 event.rs set_event_init srcElement null 初始化 + event_target.rs dispatch 同步设。
#[test]
fn native_event_src_element_r32() {
    let html = r#"<html><body><div id="a"></div></body></html>"#;
    // 初始：new Event srcElement === null（spec srcElement getter 返 target，dispatch 前 target=null）。
    assert_eq!(
        run_script(html, r#"(new Event('test').srcElement === null)"#),
        "true",
        "new Event srcElement 初始 null（dispatch 前 target=null）"
    );
    // dispatch 期 srcElement === target === 派发目标元素。
    assert_eq!(
        run_script(
            html,
            r#"(() => {
  const el = __zw_native_element_for_id('a');
  let got = 'unset';
  el.addEventListener('test', e => { got = (e.srcElement === el) + '/' + (e.srcElement === e.target); });
  el.dispatchEvent(new Event('test'));
  return got;
})()"#
        ),
        "true/true",
        "dispatch 期 srcElement === target === 派发目标元素"
    );
}

// ── R36 node.rs mutation 错误分支 coverage（appendChild/insertBefore/removeChild/replaceChild
//    的 DomError→DOMException 抛出路径，spec dom-node-* HierarchyRequestError/NotFoundError）──

/// native 树 mutation 错误路径：appendChild cycle → HierarchyRequestError；removeChild 非 child →
/// NotFoundError；replaceChild oldChild 不在 parent → NotFoundError。覆盖 node.rs `Some(Err(e))` 分支
/// （dom_error_exception 映射 WouldCreateCycle→HierarchyRequestError、NotAChild→NotFoundError）。
#[test]
fn native_node_mutation_error_paths_r36() {
    let html = r#"<div id="root"><span id="c1"></span><b id="c2"></b></div>"#;
    // appendChild cycle：parent.appendChild(parent) → HierarchyRequestError（new child 是 parent 祖先）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const r = __zw_native_element_for_id('root'); try { r.appendChild(r); return 'no-throw'; } catch (e) { return e.name; } })()"#
        ),
        "HierarchyRequestError",
        "appendChild(parent) cycle → HierarchyRequestError"
    );
    // appendChild cycle（孙→祖）：c1.appendChild(root) → root 是 c1 祖先 → HierarchyRequestError。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const r = __zw_native_element_for_id('root'); const c1 = __zw_native_element_for_id('c1'); try { c1.appendChild(r); return 'no-throw'; } catch (e) { return e.name; } })()"#
        ),
        "HierarchyRequestError",
        "appendChild(ancestor) cycle → HierarchyRequestError"
    );
    // removeChild 非 child：root.removeChild(c2 的副本/非自身 child) — 用新建元素（不在 root 下）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const r = __zw_native_element_for_id('root'); const orphan = __zw_native_create_element('div'); try { r.removeChild(orphan); return 'no-throw'; } catch (e) { return e.name; } })()"#
        ),
        "NotFoundError",
        "removeChild(非 child) → NotFoundError"
    );
    // replaceChild oldChild 不在 parent：root.replaceChild(new, orphan) → NotFoundError。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const r = __zw_native_element_for_id('root'); const nw = __zw_native_create_element('p'); const orphan = __zw_native_create_element('i'); try { r.replaceChild(nw, orphan); return 'no-throw'; } catch (e) { return e.name; } })()"#
        ),
        "NotFoundError",
        "replaceChild(new, 非 child oldChild) → NotFoundError"
    );
    // insertBefore cycle：root.insertBefore(root, c1) → root 是自身祖先 → HierarchyRequestError。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const r = __zw_native_element_for_id('root'); const c1 = __zw_native_element_for_id('c1'); try { r.insertBefore(r, c1); return 'no-throw'; } catch (e) { return e.name; } })()"#
        ),
        "HierarchyRequestError",
        "insertBefore(self, ref) cycle → HierarchyRequestError"
    );
}

// ── R36 element.rs aria/role IDL 反射 coverage（idl_to_attr 全分支：aria*→aria-x、role→role、非 aria 原样）──

/// WAI-ARIA IDL 反射：`el.ariaLabel`↔`aria-label`（aria 前缀+大写→连字符小写）、`el.role`↔`role`、
/// aria 属性缺省 ""。覆盖 element.rs `idl_to_attr` + aria_reflected_getter/setter + read/write_reflected_attr。
#[test]
fn native_aria_role_idl_reflection_r36() {
    let html = r#"<div id="el"></div>"#;
    // ariaLabel setter → aria-label content 属性；getter 回读。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const el = __zw_native_element_for_id('el'); el.ariaLabel = 'Save'; return el.getAttribute('aria-label') + '/' + el.ariaLabel; })()"#
        ),
        "Save/Save",
        "ariaLabel ↔ aria-label 反射（idl_to_attr aria 分支）"
    );
    // role setter → role content 属性；getter 回读（idl_to_attr role 特殊分支）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const el = __zw_native_element_for_id('el'); el.role = 'button'; return el.getAttribute('role') + '/' + el.role; })()"#
        ),
        "button/button",
        "role ↔ role 反射（idl_to_attr role 分支）"
    );
    // aria 属性缺省 ""（无属性时 getter 返空串，spec WAI-ARIA IDL 反射缺省 ""）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const el = __zw_native_element_for_id('el'); return el.ariaLabel + '/' + el.role; })()"#
        ),
        "/",
        "aria/role 反射缺省空串"
    );
    // ariaLabelledBy → aria-labelledby（驼峰多段，idl_to_attr rest.to_ascii_lowercase）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const el = __zw_native_element_for_id('el'); el.ariaLabelledBy = 't1 t2'; return el.getAttribute('aria-labelledby'); })()"#
        ),
        "t1 t2",
        "ariaLabelledBy ↔ aria-labelledby（多段驼峰小写）"
    );
    // aria setter null → aria 属性 "null"（非 LegacyNullToEmptyString，spec null→\"null\"）。
    assert_eq!(
        run_script(
            html,
            r#"(() => { const el = __zw_native_element_for_id('el'); el.ariaLabel = null; return el.getAttribute('aria-label'); })()"#
        ),
        "null",
        "aria setter null → content 属性 \"null\"（非 LegacyNullToEmptyString）"
    );
}
