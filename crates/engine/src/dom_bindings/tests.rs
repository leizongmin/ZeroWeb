//! P1b S1 原生 DOM 绑定测试——验证 native getter 管线（值传递 + GC + 真实 DOM 读）。
//!
//! 集成测试：建 Isolate+Context + 安装绑定 + 执行脚本读 `nodeType`/`tagName`（不经 shim
//! 字符串桥）。gc 单元测试：NodeId↔u64 编解码、stale 校验、状态隔离。

use std::cell::RefCell;
use std::rc::Rc;

use slotmap::Key;
use zero_dom::{NodeId, parse_html};

use super::gc::test_helpers::{inject_dom_for_test, reset_for_test};
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
