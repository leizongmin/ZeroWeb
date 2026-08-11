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
