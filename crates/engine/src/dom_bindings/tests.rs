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
