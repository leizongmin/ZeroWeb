//! P1b S1 原生 DOM 绑定测试——验证 native getter 管线（值传递 + GC + 真实 DOM 读）。
//!
//! 集成测试：建 Isolate+Context + 安装绑定 + 执行脚本读 `nodeType`/`tagName`（不经 shim
//! 字符串桥）。gc 单元测试：NodeId↔u64 编解码、stale 校验、状态隔离。

use std::cell::RefCell;
use std::rc::Rc;

use slotmap::Key;
use zero_dom::{NodeId, parse_html};

use super::gc::test_helpers::{
    attr_cache_alive, inject_dom_for_test, listener_keys_for, nnm_cache_alive, reset_for_test,
};
use super::{decode_node_id, encode_node_id, install_dom_bindings, node_exists};

/// 在自带 Isolate+Context 上安装绑定并执行脚本，返回结果字符串。
///
/// 镜像 S0 PoC（`script-sandbox::dom_bindings`）的 Isolate/Context 建法；`&mut ContextScope`
/// 经 DerefMut 协化为 `&mut PinScope` 传入 `install_dom_bindings`。每次测试后 `reset_for_test`
/// 清空线程局部状态（隔离）。
fn run_script(html: &str, script: &str) -> String {
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

// ── R3136 文档级只读属性工厂（documentElement / body / head）──

/// `__zw_native_get_document_element()`：返文档根元素 <html>（nodeType=1、tagName=HTML）。
/// `__zw_native_get_body()` / `__zw_native_get_head()`：返 <body>/<head> native 元素。
#[test]
fn native_document_properties_r3136() {
    let html = r#"<html><head><title>t</title></head><body><div id="a">x</div></body></html>"#;
    assert_eq!(
        run_script(html, "(__zw_native_get_document_element().tagName)"),
        "HTML",
        "documentElement 为 <html> 根元素"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_document_element().nodeType)"),
        "1",
        "documentElement nodeType=1（Element）"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_body().tagName)"),
        "BODY",
        "body 为 <body> 元素"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_head().tagName)"),
        "HEAD",
        "head 为 <head> 元素"
    );
}

/// 文档属性与既有 querySelector / getElementById 一致：body 内 `#a` 经 body.querySelector 可达。
/// 验证 documentElement/body/head 返的对象与 element 工厂共享同一 NodeId↔对象映射（身份）。
#[test]
fn native_document_properties_identity_and_navigation_r3136() {
    let html = r#"<html><head></head><body><div id="a">x</div></body></html>"#;
    // body.querySelector('#a') === getElementById('a')（同一 NodeId → 同对象）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_body().querySelector('#a') === __zw_native_element_for_id('a'))"
        ),
        "true",
        "body.querySelector('#a') 与 getElementById 返同对象（NodeId↔对象映射共享）"
    );
    // documentElement 包含 body（documentElement.contains(body)）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document_element().contains(__zw_native_get_body()))"
        ),
        "true",
        "documentElement.contains(body)===true（<html> 含 <body>）"
    );
}

/// 运行时移除 body 后 get_body() 返 null（spec：无对应元素时 null）——html5ever 总把片段归一化
/// 成完整 <html><head><body>，故 null 路径经 removeChild detach 触发（get_elements_by_tag_name DFS
/// 从 root 不再可达 detached 节点）。验证工厂 None 分支（返 null）正确。
#[test]
fn native_document_properties_absent_null_r3136() {
    let html = r#"<html><head></head><body><div id="a"></div></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const de=__zw_native_get_document_element();\
             de.removeChild(__zw_native_get_body());\
             return (__zw_native_get_body()===null); })()"
        ),
        "true",
        "removeChild(body) 后 get_body()===null（DFS 不达 detached 节点）"
    );
}

// ── R3137 document.getElementsByTagName(name) 工厂 ──

/// `__zw_native_get_elements_by_tag_name(name)`：按标签名（大小写不敏感）收集文档序 V8 Array。
/// 多个 span 文档序 + 大小写不敏感（'SPAN' 匹配 span）+ 无匹配空数组。
#[test]
fn native_get_elements_by_tag_name_r3137() {
    let html = r#"<div id="root"><span class="a">1</span><span class="b">2</span><p>x</p></div>"#;
    // 多个 span（文档序）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('span').length)"),
        "2",
        "getElementsByTagName('span').length===2（文档序全部 span）"
    );
    // 文档序读属性（区分两 span）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_tag_name('span')[0].className+'/'+\
             __zw_native_get_elements_by_tag_name('span')[1].className)"
        ),
        "a/b",
        "文档序 span[0]/[1] className 区分"
    );
    // 大小写不敏感（'SPAN' 匹配 span，HTML 元素）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('SPAN').length)"),
        "2",
        "getElementsByTagName 大小写不敏感（'SPAN' 匹配 span）"
    );
    // 无匹配 → 空 Array（length 0）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('nope').length)"),
        "0",
        "无匹配 → 空 Array"
    );
}

/// `getElementsByTagName('*')` 匹配**全部元素**（spec 通配）——含 root、span、p 等所有 Element
///（文档序）。验证 `*` 通配路径（经 get_elements_by_tag_name_ns 内置通配）。
#[test]
fn native_get_elements_by_tag_name_wildcard_r3137() {
    let html = r#"<html><head></head><body><div id="root"><span>1</span><p>2</p></div></body></html>"#;
    // `*` 返全部元素（html/head/body/div/span/p，文档序），至少 6 个。
    let all = run_script(html, "(__zw_native_get_elements_by_tag_name('*').length)");
    let n: i64 = all.parse().unwrap_or(0);
    assert!(
        n >= 6,
        "getElementsByTagName('*') 须含 html/head/body/div/span/p（≥6），实得 {all}"
    );
    // 通配结果含具体 tag（span 在内）。
    let script = "(()=>{const all=__zw_native_get_elements_by_tag_name('*');\
    return all.some(e=>e.tagName==='SPAN')+'/'+all.some(e=>e.tagName==='P');})()";
    assert_eq!(
        run_script(html, script),
        "true/true",
        "getElementsByTagName('*') 含 span 与 p（通配全元素）"
    );
}

/// 身份：getElementsByTagName 返的对象与 getElementById/querySelector 共享 NodeId↔对象映射（同对象）。
#[test]
fn native_get_elements_by_tag_name_identity_r3137() {
    let html = r#"<div id="a"><span>x</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_tag_name('div')[0] === __zw_native_element_for_id('a'))"
        ),
        "true",
        "getElementsByTagName('div')[0] === getElementById('a')（身份缓存共享）"
    );
}

// ── R3138 document.getElementsByClassName(name) 工厂 ──

/// `__zw_native_get_elements_by_class_name(name)`：按类名收集文档序 V8 Array。
/// 多元素同 class + 无匹配空数组 + 空/空白串空数组。
#[test]
fn native_get_elements_by_class_name_r3138() {
    let html = r#"<div id="root"><span class="row">1</span><span class="row big">2</span><p class="row">3</p></div>"#;
    // 单 class "row" → 3 个（两 span + p）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row').length)"),
        "3",
        "getElementsByClassName('row').length===3（两 span + p）"
    );
    // 文档序读 tagName（span/span/p）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const a=__zw_native_get_elements_by_class_name('row');\
            return a[0].tagName+'/'+a[1].tagName+'/'+a[2].tagName;})()"
        ),
        "SPAN/SPAN/P",
        "文档序 row 元素 tagName：span/span/p"
    );
    // 单 class "big" → 1 个（仅第二 span）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('big').length)"),
        "1",
        "getElementsByClassName('big').length===1（仅 class='row big'）"
    );
    // 无匹配 → 空 Array。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('nope').length)"),
        "0",
        "无匹配 → 空 Array"
    );
    // 空/空白串 → 空 Array（spec：空 names 不匹配）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('   ').length)"),
        "0",
        "空白串 → 空 Array"
    );
}

/// **多类 spec 合规**：`'row big'`（空格分隔）→ 含【全部】两类的元素（仅 class='row big' 那个）。
/// 闭合 dom `get_elements_by_class_name` 单 token 限制——本工厂 split + 过滤实现 spec 语义。
#[test]
fn native_get_elements_by_class_name_multi_class_r3138() {
    let html = r#"<div><span class="row big">a</span><span class="row">b</span><span class="big">c</span></div>"#;
    // 'row big' → 仅含两类的 1 个（第一个 span）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row big').length)"),
        "1",
        "多类 'row big' → 含全部两类的 1 个（spec 合规）"
    );
    // 文档序 + 顺序无关（'big row' 同结果）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('big row').length)"),
        "1",
        "多类顺序无关（'big row' 同 'row big'）"
    );
    // 该元素 tagName = SPAN。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row big')[0].tagName)"),
        "SPAN",
        "多类匹配元素 tagName=SPAN"
    );
}

/// 身份：getElementsByClassName 返对象与 getElementById 共享 NodeId↔对象映射（同对象）。
#[test]
fn native_get_elements_by_class_name_identity_r3138() {
    let html = r#"<div id="a" class="row"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_class_name('row')[0] === __zw_native_element_for_id('a'))"
        ),
        "true",
        "getElementsByClassName('row')[0] === getElementById('a')（身份缓存共享）"
    );
}

// ── R3139 document.title getter/setter 工厂 ──

/// `__zw_native_get_document_title()`：读首个 `<title>` textContent；无 title → 空串。
#[test]
fn native_document_title_getter_r3139() {
    let html = r#"<html><head><title>Hello</title></head><body></body></html>"#;
    assert_eq!(
        run_script(html, "(__zw_native_get_document_title())"),
        "Hello",
        "document.title 读首个 <title> textContent"
    );
    // 无 <title> → 空串。
    let html_notitle = r#"<html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(html_notitle, "(__zw_native_get_document_title())"),
        "",
        "无 <title> → 空串"
    );
}

/// `__zw_native_set_document_title(str)`：存在 <title> → 改其 textContent；getter 回读新值。
#[test]
fn native_document_title_setter_existing_r3139() {
    let html = r#"<html><head><title>Old</title></head><body></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('New');\
            return __zw_native_get_document_title(); })()"
        ),
        "New",
        "setter 改既有 <title> textContent → getter 回读新值"
    );
}

/// setter 在无 `<title>` 时于 `<head>` 建 `<title>` 设文本——getter 回读 + head 含新建 title 元素。
#[test]
fn native_document_title_setter_create_missing_r3139() {
    let html = r#"<html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('Created');\
            return __zw_native_get_document_title(); })()"
        ),
        "Created",
        "无 <title> 时 setter 在 <head> 建 <title> → getter 回读"
    );
    // 新建 title 在 head 内（getElementsByTagName('title') 命中）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('X');\
            return __zw_native_get_elements_by_tag_name('title').length; })()"
        ),
        "1",
        "setter 建 <title> 后 getElementsByTagName('title').length===1"
    );
}

// ── R3132 appendChild/insertBefore(fragment) flatten ──

/// host.appendChild(frag)：fragment 子节点展开进 host + fragment 清空（spec flatten）。
#[test]
fn native_append_child_fragment_flatten_r3132() {
    let html = r#"<div id="host"></div>"#;
    let script = "(()=>{\
const host=__zw_native_element_for_id('host');\
const frag=__zw_native_create_document_fragment();\
frag.appendChild(__zw_native_create_element('span'));\
frag.appendChild(__zw_native_create_element('b'));\
host.appendChild(frag);\
return host.children.length+'/'+host.children[0].tagName+'/'+host.children[1].tagName+'/'+\
frag.childNodes.length;})()";
    assert_eq!(
        run_script(html, script),
        "2/SPAN/B/0",
        "appendChild(frag) flatten：子进 host + fragment 清空"
    );
}

/// host.insertBefore(frag, ref)：fragment 子节点插到 ref 前 + fragment 清空。
#[test]
fn native_insert_before_fragment_flatten_r3132() {
    let html = r#"<div id="host"><i id="ref">r</i></div>"#;
    let script = "(()=>{\
const host=__zw_native_element_for_id('host');\
const ref=__zw_native_element_for_id('ref');\
const frag=__zw_native_create_document_fragment();\
frag.appendChild(__zw_native_create_element('span'));\
host.insertBefore(frag, ref);\
return host.children[0].tagName+'/'+host.children[1].tagName+'/'+host.children.length;})()";
    assert_eq!(
        run_script(html, script),
        "SPAN/I/2",
        "insertBefore(frag, ref) flatten：子插 ref 前"
    );
}

// ── R3110 节点导航 getter（parentNode / firstChild / lastChild / nextSibling / previousSibling / hasChildNodes）──
//
// HTML: <div id="root"><span id="s1">hello</span><span id="s2"></span></div>
// root 子节点 = [span#s1, span#s2]（标签间无空白文本）；s1 子 = 文本 "hello"。

/// parentNode + nextSibling + previousSibling（spec `dom-node-parent-node` / `-next-sibling` / `-previous-sibling`）。
#[test]
fn native_node_navigation_parent_and_siblings() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const s1=__zw_native_element_for_id('s1');\
             return s1.parentNode.id+'/'+s1.nextSibling.id+'/'+__zw_native_element_for_id('s2').previousSibling.id; })()"
        ),
        "root/s2/s1"
    );
}

/// firstChild + lastChild（spec `dom-node-first-child` / `-last-child`）。
#[test]
fn native_node_navigation_first_last_child() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const r=__zw_native_element_for_id('root');\
             return r.firstChild.id+'/'+r.lastChild.id; })()"
        ),
        "s1/s2"
    );
}

/// hasChildNodes（spec `dom-node-has-child-nodes`）：有子 → true；空 span → false。
#[test]
fn native_node_navigation_has_child_nodes() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ return __zw_native_element_for_id('root').hasChildNodes()+'/'+__zw_native_element_for_id('s2').hasChildNodes(); })()"
        ),
        "true/false"
    );
}

/// firstChild 返文本节点（nodeType=3）——导航返回非 Element 子节点，包同一模板（R3104 node-type-aware）。
#[test]
fn native_node_navigation_text_first_child() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('s1').firstChild.nodeType)"),
        "3"
    );
}

/// nextSibling 越界 → null（spec detached/无兄弟返 null，非 undefined）。
#[test]
fn native_node_navigation_null_relation() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('s2').nextSibling === null)"),
        "true"
    );
}

// ── R3111 replaceChild native + nodeValue/data setter ──

/// `replaceChild(newChild, oldChild)`（spec `dom-node-replace-child`）：newChild 替换 oldChild 位置，
/// 返 oldChild。补全树 mutation 集（appendChild/insertBefore/removeChild/replaceChild）。
#[test]
fn native_replace_child() {
    let html = r#"<div id="root"><span id="s1">1</span><span id="s2">2</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const r=__zw_native_element_for_id('root');\
             const nw=__zw_native_create_element('span'); nw.id='nw';\
             const old=r.replaceChild(nw, __zw_native_element_for_id('s1'));\
             return r.children[0].id+'/'+r.children[1].id+'/'+old.id; })()"
        ),
        "nw/s2/s1"
    );
}

/// `nodeValue` setter on Text（spec `dom-node-nodevalue` setter）：改文本节点内容，读回见新值。
#[test]
fn native_node_value_setter_text() {
    let html = r#"<div id="root">hello</div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('root').firstChild;\
             t.nodeValue='world'; return t.nodeValue+'/'+t.nodeType; })()"
        ),
        "world/3"
    );
}

/// `nodeValue` setter on Comment（spec）：改注释内容，读回见新值。
#[test]
fn native_node_value_setter_comment() {
    let html = r#"<div id="root"><!--c--></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const c=__zw_native_element_for_id('root').firstChild;\
             c.nodeValue='d'; return c.nodeValue+'/'+c.nodeType; })()"
        ),
        "d/8"
    );
}

/// `nodeValue` setter on Element → no-op（spec：Element/Document 设 nodeValue 无效；getter 返 null）。
#[test]
fn native_node_value_setter_element_noop() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.nodeValue='x'; return String(e.nodeValue); })()"
        ),
        "null"
    );
}

// ── R3112 NamedNodeMap（element.attributes 集合）──
//
// HTML: <div id="a" class="row" data-x="42"></div>（属性源序 id/class/data-x）。

/// `attributes.length` + 身份（`el.attributes === el.attributes`，spec live 同对象）。
#[test]
fn native_attributes_length_and_identity() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             return el.attributes.length+'/'+(el.attributes === el.attributes); })()"
        ),
        "3/true"
    );
}

/// `item(index)`：源序属性 → Attr-like {name, value}；越界 → null。
#[test]
fn native_attributes_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return a.item(0).name+'/'+a.item(2).name+'/'+a.item(2).value+'/'+(a.item(9)===null); })()"
        ),
        "id/data-x/42/true"
    );
}

/// `getNamedItem(name)`：有 → {name,value}；无 → null。
#[test]
fn native_attributes_get_named_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return a.getNamedItem('class').value+'/'+(a.getNamedItem('nope')===null); })()"
        ),
        "row/true"
    );
}

/// `setNamedItem({name,value})` + `removeNamedItem(name)`：写回 owner 元素属性（getAttribute/hasAttribute 见）。
#[test]
fn native_attributes_set_and_remove_named_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); const a=el.attributes;\
             a.setNamedItem({name:'data-y', value:'7'});\
             a.removeNamedItem('class');\
             return el.getAttribute('data-y')+'/'+el.hasAttribute('class'); })()"
        ),
        "7/false"
    );
}

// ── R3122 Attr 节点（完整 Attr：nodeType=2/name/value/ownerElement）──
//
// getNamedItem / item 返 Attr 节点对象（非 plain {name,value}）。闭合 R3112 plain-object 限制。

/// Attr 节点面：nodeType=2、name=nodeName、value live、ownerElement===owner 元素、身份（同 attr 同对象）。
#[test]
fn native_attr_node_surface() {
    let html = r#"<div id="a" class="row"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const at=el.attributes.getNamedItem('class');\
             return at.nodeType+'/'+at.name+'/'+at.nodeName+'/'+at.value+'/'+(at.ownerElement===el); })()"
        ),
        "2/class/class/row/true"
    );
    // value setter 经 set_attribute 写回 owner 元素（live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.attributes.getNamedItem('class').value='new';\
             return el.getAttribute('class'); })()"
        ),
        "new"
    );
    // nodeValue / textContent === value（Node 接口面）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const at=__zw_native_element_for_id('a').attributes.getNamedItem('class');\
             return at.nodeValue+'/'+at.textContent; })()"
        ),
        "row/row"
    );
    // 身份：同 (owner, name) 返同对象（spec identity）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return (a.getNamedItem('class')===a.getNamedItem('class'))+'/'+\
             (a.getNamedItem('class')===a.item(1)); })()"
        ),
        "true/true"
    );
    // item(0) 返 Attr 节点（nodeType=2 + name=id）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const at=__zw_native_element_for_id('a').attributes.item(0);\
             return at.nodeType+'/'+at.name; })()"
        ),
        "2/id"
    );
}

// ── R3113 innerHTML / outerHTML 序列化 getter ──

/// `innerHTML`（子节点序列化拼接）+ `outerHTML`（含自身 tag）。
#[test]
fn native_inner_outer_html() {
    let html = r#"<div id="a"><b>hi</b>!</div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').innerHTML)"),
        r#"<b>hi</b>!"#
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').outerHTML)"),
        r#"<div id="a"><b>hi</b>!</div>"#
    );
}

/// `outerHTML` 反映 native 属性写（live 序列化）。
#[test]
fn native_outer_html_reflects_attribute() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.setAttribute('data-x','9');\
             return e.outerHTML; })()"
        ),
        r#"<div id="a" data-x="9"></div>"#
    );
}

/// `innerHTML` 反映 native 文本写（nodeValue）——R3108 重渲染后 live 序列化见新文本。
#[test]
fn native_inner_html_reflects_text_write() {
    let html = r#"<div id="a"><span id="s">old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('s').firstChild.nodeValue='new';\
             return __zw_native_element_for_id('a').innerHTML; })()"
        ),
        r#"<span id="s">new</span>"#
    );
}

// ── R3123 innerHTML / outerHTML setter（解析 HTML 片段清子/替换自身）──

/// `innerHTML` setter：设含 markup 片段 → 替换现有子节点（旧子清空，新片段深拷贝追加）。
/// getter live 序列化回读验证（旧 `<span>old</span>` 应被替换为 `<b>new</b><i>x</i>`）。
#[test]
fn native_inner_html_setter_replaces_children() {
    let html = r#"<div id="a"><span>old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.innerHTML='<b>new</b><i>x</i>';\
             return e.innerHTML; })()"
        ),
        r#"<b>new</b><i>x</i>"#
    );
}

/// `innerHTML` setter 无 markup（纯文本）→ 单文本节点（不走片段解析路径）。
#[test]
fn native_inner_html_setter_plain_text() {
    let html = r#"<div id="a"><b>x</b></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.innerHTML='hello';\
             return e.innerHTML+'/'+e.firstChild.nodeType+'/'+e.childNodes.length; })()"
        ),
        "hello/3/1"
    );
}

/// `innerHTML` setter 空串 → 清空所有子节点。
#[test]
fn native_inner_html_setter_empty_clears() {
    let html = r#"<div id="a"><b>x</b>!</div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').innerHTML='';\
             return __zw_native_element_for_id('a').childNodes.length+'/'+__zw_native_element_for_id('a').hasChildNodes(); })()"
        ),
        "0/false"
    );
}

/// `outerHTML` setter：元素整体替换为片段顶层节点。原元素从 DOM 移除（id 失效），
/// 父节点 innerHTML 反映新内容。验证经父节点回读（原 id 'a' 已 detach）。
#[test]
fn native_outer_html_setter_replaces_self() {
    let html = r#"<div id="p"><span id="a">old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML='<b id=\"c\">new</b>';\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        r#"<b id="c">new</b>"#
    );
}

/// `outerHTML` setter 空串 → 仅移除目标（spec：`el.outerHTML=''` 移除元素）。
#[test]
fn native_outer_html_setter_empty_removes() {
    let html = r#"<div id="p"><span id="a">x</span><i>y</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML='';\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        r#"<i>y</i>"#
    );
}

// ── R3114 cloneNode(deep) ──
//
// HTML: <div id="a" class="x"><span id="s">hi</span></div>

/// `cloneNode(false)` 浅克隆：同 tag + 属性，无子节点；新对象（≠源）。
#[test]
fn native_clone_node_shallow() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); const c=el.cloneNode(false);\
             return c.tagName+'/'+c.getAttribute('class')+'/'+c.children.length+'/'+(c!==el); })()"
        ),
        "DIV/x/0/true"
    );
}

/// `cloneNode(true)` 深克隆：含子树（span + 文本），子节点经 native 读见。
#[test]
fn native_clone_node_deep() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const c=__zw_native_element_for_id('a').cloneNode(true);\
             return c.children.length+'/'+c.children[0].tagName+'/'+c.children[0].textContent; })()"
        ),
        "1/SPAN/hi"
    );
}

/// `cloneNode()` 缺省 deep → false（spec：浅克隆）。
#[test]
fn native_clone_node_default_shallow() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').cloneNode().children.length)"),
        "0"
    );
}

// ── R3115 contains(node) ──
//
// HTML: <div id="a"><div id="b"><span id="c">x</span></div></div>

/// `contains`：后代 / 自身 / 非后代（walk parent 链）。
#[test]
fn native_contains_relations() {
    let html = r#"<div id="a"><div id="b"><span id="c">x</span></div></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'), c=__zw_native_element_for_id('c');\
             return a.contains(c)+'/'+a.contains(a)+'/'+c.contains(a); })()"
        ),
        "true/true/false"
    );
}

/// `contains(null)` → false（spec：contains(null)===false；非 node 参亦 false）。
#[test]
fn native_contains_null() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').contains(null))"),
        "false"
    );
}

// ── R3133 节点包装器终结器（闭合 R3109 LISTENERS detach 泄漏）──

/// 重新附加语义：removeChild 不清监听器——detach 后 re-append，监听器仍触发（spec：
/// 节点从 DOM 移除不丢弃监听器，跨 detach/重附加保留）。此为终结器设计前提（仅包装器真被 GC
/// 才清 LISTENERS，而非 removeChild 时清）。
#[test]
fn native_listener_survives_detach_reattach_r3133() {
    let html = r#"<div id="host"><span id="a"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
             const el=__zw_native_element_for_id('a');\
             el.addEventListener('click', ()=>{ globalThis.__fired='yes'; });\
             host.removeChild(el);\
             host.appendChild(el);\
             el.dispatchEvent({type:'click'});\
             return globalThis.__fired || 'no'; })()"
        ),
        "yes"
    );
}

/// 终结器回收：脚本 add 2 监听器后丢包装器引用（仅 weak 缓存持）→ 强制 GC 收包装器 →
/// guaranteed 终结器清本节点 LISTENERS → 条目归 0（闭合 R3109：旧实现 detached 节点监听器永驻）。
#[test]
fn native_finalizer_cleans_listeners_on_gc_r3133() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'></div>")));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let cleaned;
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // IIFE 结束 el 出作用域 → JS 强引用断（仅 weak 缓存持包装器）。
        let script = "(()=>{ const el=__zw_native_element_for_id('a');\
             el.addEventListener('click',()=>{});\
             el.addEventListener('keyup',()=>{});\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        // 多轮 GC 收 weak-held 包装器 → guaranteed 终结器（GC 第二遍）清本节点 LISTENERS。
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        cleaned = listener_keys_for(ffi);
    }
    assert_eq!(
        cleaned, 0,
        "包装器 GC 后终结器应清本节点全部监听器（R3109 detach 泄漏闭合）"
    );
    reset_for_test();
}

/// 终结器不误伤活跃节点：包装器仍被 JS 强引用（globalThis 持）→ 不被 GC → 监听器保留。
/// 防回归：终结器仅在包装器真无引用时触发，不会清仍在用节点的监听器。
#[test]
fn native_finalizer_keeps_listeners_while_referenced_r3133() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'></div>")));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let kept;
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // globalThis.__el 持强引用 → 包装器不被 GC → 监听器保留。
        let script = "(()=>{ globalThis.__el=__zw_native_element_for_id('a');\
             globalThis.__el.addEventListener('click',()=>{});\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        kept = listener_keys_for(ffi);
    }
    assert_eq!(kept, 1, "包装器仍被 JS 强引用时不应被 GC，监听器须保留（防终结器误伤）");
    reset_for_test();
}

// ── R3134 NNM/ATTR 身份缓存 weak 化（闭合同 pattern 泄漏，R3133 已知限制①）──

/// NNM/Attr weak 回收：脚本建 NNM + Attr 后丢引用（仅 weak 缓存持）→ 强制 GC → weak 句柄死
/// （对象可回收）。闭合 R3133 已知限制①——旧实现 strong Global 永驻，JS 丢引用亦不回收。
/// 元素（globalThis 持强引用）不被 GC，仅 NNM/Attr 回收，证明泄漏闭合且不影响活跃元素。
#[test]
fn native_nnm_attr_cache_reclaimable_on_gc_r3134() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(r#"<div id="a" class="row"></div>"#)));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let (nnm_alive, attr_alive);
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // globalThis.__el 持元素强引用（元素不被 GC）；NNM/Attr 仅局部持，IIFE 结束即断。
        let script = "(()=>{ globalThis.__el=__zw_native_element_for_id('a');\
             void globalThis.__el.attributes;\
             void globalThis.__el.attributes.getNamedItem('class');\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        nnm_alive = nnm_cache_alive(ffi);
        attr_alive = attr_cache_alive(ffi, "class");
    }
    assert!(
        !nnm_alive,
        "NNM 丢 JS 引用后应可 GC（weak 死），闭合 R3133 限制① strong-Global 泄漏"
    );
    assert!(
        !attr_alive,
        "Attr 丢 JS 引用后应可 GC（weak 死），闭合 R3133 限制① strong-Global 泄漏"
    );
    reset_for_test();
}

/// NNM/Attr 身份保持：JS 持引用期间 weak 活 → 同对象（spec identity `el.attributes === el.attributes`、
/// `getNamedItem('x') === getNamedItem('x')`）。防回归：weak 化不破坏身份。
#[test]
fn native_nnm_attr_identity_while_referenced_r3134() {
    let html = r#"<div id="a" class="row"></div>"#;
    // NNM 身份：同元素 .attributes 两次取，JS 持比较 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').attributes === __zw_native_element_for_id('a').attributes)"
        ),
        "true"
    );
    // Attr 身份：同 (owner, name) getNamedItem 两次，JS 持比较 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return (a.getNamedItem('class')===a.getNamedItem('class')); })()"
        ),
        "true"
    );
}
