//! polyfill vs native A/B 行为对照门骨架（M0 must-complete 项 5，js-dom goal）。
//!
//! 目的：为整个 JS↔DOM 桥原生化迁移期（M1 L2 / M2 S6 / M6 QuickJS native）提供
//! 「行为不退化」安全网。对**同一组可观测 DOM 读操作**，断言两条路径返回一致：
//!
//! - **A = native 路径**：`install_dom_bindings` 直接装原生对象（`__zw_native_*`
//!   工厂 + `document` 经 `__zw_native_get_document()` 实例化），JS 持有真 V8 node
//!   对象（internal slot[0] = NodeId）。
//! - **B = polyfill 路径**：`generate_js_dom_shim` + `register_dom_callbacks`（生产
//!   仍走的权威路径），JS 持有 `_makeProxy(sel, handle)` 字符串桥 Proxy。
//!
//! 对照哲学：聚焦**可观测行为等价**（同一 HTML + 同一读操作 → 同一返回串），不强求
//! API 形态同构（native 真对象 vs polyfill Proxy）。native 侧 helper 把 `document`
//! 桥到 global（`var document = __zw_native_get_document()`），使两路径共用
//! `document.querySelector(...)` 脚本形式，对照更纯粹。
//!
//! **双 feature 可参数化**（v1.1 DC-7 铺路）：本骨架的对照用例表（`READ_CASES`）与
//! 断言逻辑与引擎后端无关。M6 QuickJS native 移植完成后，同一断言表将在 `--features
//! quickjs` 矩阵下复跑（QuickJS 侧 `run_native` 待 rquickjs 原生绑定落地后镜像实现）。
//! 当前 polyfill 路径 + native 路径均依赖 V8，故整模块 `#[cfg(feature = "v8")]` 门控
//!（与 `js_dom_bridge_tests` 一致）。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use zero_dom::parse_html;
use zero_script_sandbox::{Sandbox, V8Sandbox};

use super::gc::test_helpers::reset_for_test;
use super::tests::run_script as run_native_raw;
use crate::js_dom_bridge::{CanvasRegistry, DomMutation, generate_js_dom_shim, register_dom_callbacks};

// ── A = native 路径 helper ────────────────────────────────────────

/// native 路径执行 `expr`：复用 `tests::run_script`（建 Isolate + `install_dom_bindings`），
/// 额外把 `document` 桥到 global，使 `expr` 可用 `document.querySelector(...)` 标准形式
///（与 polyfill 路径脚本对称）。
///
/// 与 `run_native_raw` 分离：raw 不挂 `document`（既有多数 native 测试直接用 `__zw_native_*`
/// 工厂），A/B 门需要 `document` 以对照 polyfill 侧脚本。
fn run_native(html: &str, expr: &str) -> String {
    // 桥 `document` 到 global 后再求值 expr。
    let script = format!("var document = __zw_native_get_document(); ({expr})");
    run_native_raw(html, &script)
}

// ── B = polyfill 路径 helper ──────────────────────────────────────

/// polyfill 路径执行 `expr`：复刻 `js_dom_bridge_tests` 标准模式（V8Sandbox + shim +
/// `register_dom_callbacks`）。这是当前生产权威路径（kill-switch 关 → 默认走此）。
fn run_polyfill(html: &str, expr: &str) -> String {
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .expect("V8 sandbox init");
    sandbox.execute(generate_js_dom_shim()).expect("install shim");
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(html.to_string()));
    let page_url = Arc::new(Mutex::new("https://zero.test/ab-compare".to_string()));
    let canvas_registry = Arc::new(Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    let r = sandbox.execute(&format!("String({expr})")).expect("polyfill execute");
    r.value
}

// ── 对照用例表 ───────────────────────────────────────────────────
//
// 每条 = (HTML, JS 读表达式)。两条路径分别求值，断言返回串相等。
// 选读操作（无 mutation）：M0 阶段 L2/S6 未做，写路径行为差异属预期，先守读路径等价。
// 后续 M1 L2-read-only 切片落地后，本表自然成为其 A/B 验收。

/// 一条 A/B 对照用例。
struct AbCase {
    /// 标识（失败时定位）。
    name: &'static str,
    /// 输入 HTML（两路径同一份）。
    html: &'static str,
    /// 读操作 JS 表达式（求值后 String() 转串对照）。
    expr: &'static str,
}

const READ_CASES: &[AbCase] = &[
    AbCase {
        name: "query-selector-tagname",
        html: r#"<html><body><div id="a"><span class="row" data-x="1">x</span><span class="row" data-x="2">y</span></div></body></html>"#,
        expr: r#"document.querySelector('.row').tagName"#,
    },
    AbCase {
        name: "query-selector-all-length",
        html: r#"<html><body><div><span class="row">a</span><span class="row">b</span></div></body></html>"#,
        expr: r#"document.querySelectorAll('.row').length"#,
    },
    AbCase {
        name: "get-element-by-id-tagname",
        html: r#"<html><body><div id="main">M</div></body></html>"#,
        expr: r#"document.getElementById('main').tagName"#,
    },
    AbCase {
        name: "get-attribute",
        html: r#"<html><body><a id="l" href="/p" title="t">A</a></body></html>"#,
        expr: r#"document.getElementById('l').getAttribute('href')"#,
    },
    AbCase {
        name: "get-attribute-missing-null",
        html: r#"<html><body><div id="d">D</div></body></html>"#,
        expr: r#"String(document.getElementById('d').getAttribute('nope'))"#,
    },
    AbCase {
        name: "has-attribute",
        html: r#"<html><body><input id="i" type="text" disabled></body></html>"#,
        expr: r#"String(document.getElementById('i').hasAttribute('disabled'))"#,
    },
    AbCase {
        name: "node-type",
        html: r#"<html><body><div id="d"><span>s</span></div></body></html>"#,
        expr: r#"document.getElementById('d').nodeType"#,
    },
    AbCase {
        name: "id-reflected",
        html: r#"<html><body><p id="para">P</p></body></html>"#,
        expr: r#"document.querySelector('p').id"#,
    },
    AbCase {
        name: "query-selector-descendant",
        html: r#"<html><body><div id="a"><span class="row" data-x="2">y</span></div></body></html>"#,
        expr: r#"document.querySelector('div span').getAttribute('data-x')"#,
    },
];

/// 对单条用例跑 A/B 对照，返回 (native, polyfill)。
fn ab_pair(c: &AbCase) -> (String, String) {
    let native = run_native(c.html, c.expr);
    let polyfill = run_polyfill(c.html, c.expr);
    (native, polyfill)
}

// ── 对照门测试 ───────────────────────────────────────────────────

/// A/B 对照门主测：遍历 `READ_CASES`，断言 native 与 polyfill 每条返回一致。
///
/// 失败消息带用例名 + 两路径实际值，便于定位是 native 退化还是 polyfill 差异。
/// M0 阶段允许少量已知差异（读路径应基本等价；若有差异记 master.md「未解决问题」
/// 作为 M1 L2 修复目标，而非放宽本断言）。
#[test]
fn ab_read_operations_native_equals_polyfill() {
    for c in READ_CASES {
        let (native, polyfill) = ab_pair(c);
        assert_eq!(
            native, polyfill,
            "A/B 对照失败 [{}]：native=`{}` vs polyfill=`{}`\nHTML: {}\nEXPR: {}",
            c.name, native, polyfill, c.html, c.expr
        );
    }
}

/// 单独覆盖 `querySelectorAll` 索引读（两路径都返类数组，第 0 个元素的属性应一致）。
#[test]
fn ab_query_selector_all_indexed_attribute() {
    let html =
        r#"<html><body><ul><li class="item" data-i="0">a</li><li class="item" data-i="1">b</li></ul></body></html>"#;
    let expr = r#"document.querySelectorAll('.item')[0].getAttribute('data-i')"#;
    let native = run_native(html, expr);
    let polyfill = run_polyfill(html, expr);
    assert_eq!(
        native, polyfill,
        "querySelectorAll 索引读 A/B 不一致：native=`{}` polyfill=`{}`",
        native, polyfill
    );
}

/// sanity：native helper 本身工作（document 桥接 + querySelector 命中）。
/// 若本测失败，说明 native 侧 `__zw_native_get_document` 接线或 helper 有误，先修它
/// 再看 A/B 对照（避免把 native helper bug 误判为行为差异）。
#[test]
fn native_helper_document_bridge_works() {
    let html = r#"<html><body><div id="a" class="c">x</div></body></html>"#;
    assert_eq!(run_native(html, "document.getElementById('a').tagName"), "DIV");
    assert_eq!(run_native(html, "document.querySelector('.c').id"), "a");
}

/// sanity：polyfill helper 本身工作（shim + callbacks 装好，document.querySelector 命中）。
#[test]
fn polyfill_helper_shim_callbacks_works() {
    let html = r#"<html><body><div id="a" class="c">x</div></body></html>"#;
    assert_eq!(run_polyfill(html, "document.getElementById('a').tagName"), "DIV");
    assert_eq!(run_polyfill(html, "document.querySelector('.c').id"), "a");
}

// ── 异常路径 A/B 对照（classList token 校验抛 DOMException）─────────
//
// 读路径对照（READ_CASES）验证「正常返回值等价」；异常路径对照验证「校验失败抛相同 DOMException
// （按 name 区分）」。两路径共用 try/catch 脚本形式，断言返串一致。这是 DOMException 抛出语义
// 切片（dom/nodes/Element-classlist ~405 失败修复）的 A/B 验收。

/// 异常路径对照 helper：对 `expr`（应为会抛的 classList 调用）跑两路径，各返 `"threw|<name>"`
/// 或 `"no-throw|<String(value)>"`。两路径用同一 try/catch 包装脚本。
fn ab_catch(html: &str, expr: &str) -> (String, String) {
    // try/catch：捕获异常返 name，否则返 value（与既有 tests_collections token 校验测一致形式）。
    let script =
        format!("(()=>{{ try {{ return 'no-throw|'+String({expr}); }} catch(e) {{ return 'threw|'+e.name; }} }})()");
    let native = run_native(html, &script);
    let polyfill = run_polyfill(html, &script);
    (native, polyfill)
}

/// classList token 校验异常路径：空 token / 含空白 token 两路径都抛对应 name 的 DOMException。
#[test]
fn ab_classlist_token_validation_throws_dom_exception() {
    let html = r#"<html><body><div id="a" class="a"></div></body></html>"#;
    // 空 token → SyntaxError（spec dom-domtokenlist-validation）。
    let (n, p) = ab_catch(html, "document.getElementById('a').classList.add('')");
    assert_eq!(n, p, "classList.add('') A/B 不一致：native=`{n}` polyfill=`{p}`");
    assert!(n.starts_with("threw|SyntaxError"), "应抛 SyntaxError，实际：{n}");

    // 含空白 token → InvalidCharacterError。
    let (n, p) = ab_catch(html, "document.getElementById('a').classList.add('foo bar')");
    assert_eq!(n, p, "classList.add('foo bar') A/B 不一致：native=`{n}` polyfill=`{p}`");
    assert!(
        n.starts_with("threw|InvalidCharacterError"),
        "应抛 InvalidCharacterError，实际：{n}"
    );

    // contains/toggle 同样校验（覆盖校验点一致性）。
    let (n, p) = ab_catch(html, "document.getElementById('a').classList.toggle('')");
    assert_eq!(n, p, "classList.toggle('') A/B 不齐：native=`{n}` polyfill=`{p}`");
    assert!(n.starts_with("threw|"), "toggle('') 应抛，实际：{n}");
}

/// createElement 非法标签名校验异常路径：两路径都抛 InvalidCharacterError DOMException。
/// 对齐 WPT dom/nodes/Document-createElement.html invalid 列表。
#[test]
fn ab_create_element_invalid_name_throws_dom_exception() {
    let html = r#"<html><body></body></html>"#;
    // 数字首 / `<` / 含空白 / `-`首 → InvalidCharacterError（两路径一致）。
    for bad in ["1foo", "<foo", "foo>", "fo o", "-foo"] {
        let escaped = bad.replace('\\', "\\\\").replace('\'', "\\'");
        let (n, p) = ab_catch(html, &format!("document.createElement('{escaped}')"));
        assert_eq!(n, p, "createElement('{bad}') A/B 不一致：native=`{n}` polyfill=`{p}`");
        assert!(
            n.starts_with("threw|InvalidCharacterError"),
            "createElement('{bad}') 应抛 InvalidCharacterError，实际：{n}"
        );
    }
    // 合法标签不抛（防误伤）—— 两路径都返正常元素。
    let (n, p) = ab_catch(html, "document.createElement('foo').tagName");
    assert_eq!(n, p, "createElement('foo') A/B 不一致：native=`{n}` polyfill=`{p}`");
    assert!(n.starts_with("no-throw|"), "createElement('foo') 不应抛，实际：{n}");
}

// ── js-dom M1 L2 R104：三方合一（native(A) = polyfill(B) = renderer(C)）─────────
//
// L2 的验收面：两条写路径对**同一 live Document** 的最终状态等价——
// - A（native）：`install_dom_bindings` 直改 DOM_SOURCE 的 doc（default-on 后的生产路径）；
// - B（polyfill）：`__zw_*` 回调收集 DomMutation → `apply_dom_mutations` 应用（当前生产路径）；
// - C（renderer）：live doc 的最终 outer_html（渲染消费的权威状态）。
// 断言 A 后与 B 后的 outer_html 一致（写路径语义等价），是 DC-1「三方合一，无独立快照」
// 的行为资产。读路径等价已由 READ_CASES 守（R0 起）。

use super::tests::run_script_return_doc_html;
use crate::js_dom_bridge::apply_dom_mutations;

/// B 路径（polyfill 写）执行 `script`（含 DOM 写调用）并返回**应用后** doc 的
/// outer_html：注册回调 → 执行（mutations 收集进队列）→ `apply_dom_mutations` 应用到
/// parse_html(html) 的 doc（与 webview `apply_pending_shared_mutations` 同机制）。
fn run_polyfill_applied_html(html: &str, script: &str) -> String {
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .expect("V8 sandbox init");
    sandbox.execute(generate_js_dom_shim()).expect("install shim");
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(html.to_string()));
    let page_url = Arc::new(Mutex::new("https://zero.test/ab-compare".to_string()));
    let canvas_registry = Arc::new(Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    let r = sandbox.execute(script).expect("polyfill execute");
    // apply 收集到的 mutations 到独立 doc（C 侧读数）。
    let mut doc = parse_html(html);
    let recorded = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    apply_dom_mutations(&mut doc, &recorded).unwrap_or_else(|e| panic!("apply_dom_mutations failed: {e}"));
    let _ = r;
    doc.outer_html(doc.root())
}

/// 三方合一对照用例：同一写序列两路径的 C 侧（outer_html）等价。
struct WriteCase {
    name: &'static str,
    html: &'static str,
    /// 含 DOM 写调用的脚本（两路径同一份；native 侧 `document` 已桥接）。
    script: &'static str,
}

const WRITE_CASES: &[WriteCase] = &[
    WriteCase {
        name: "set-attribute",
        html: r#"<html><body><div id="d">D</div></body></html>"#,
        script: r#"document.getElementById('d').setAttribute('data-k', 'v')"#,
    },
    WriteCase {
        name: "create-append",
        html: r#"<html><body><div id="host"></div></body></html>"#,
        script: r#"var s = document.createElement('span'); s.textContent = 'added'; document.getElementById('host').appendChild(s)"#,
    },
    WriteCase {
        name: "remove-child",
        html: r#"<html><body><ul><li id="x">1</li><li>2</li></ul></body></html>"#,
        script: r#"document.getElementById('x').remove()"#,
    },
];

/// 三方合一门主测：每条写用例，A（native 后 doc）与 B（polyfill apply 后 doc）的
/// outer_html 必须一致。失败消息带两路径 html 便于 diff 定位语义差异。
#[test]
fn l2_three_way_write_paths_converge_r104() {
    for c in WRITE_CASES {
        let (_, native_html) = run_script_return_doc_html(
            c.html,
            &format!("var document = __zw_native_get_document(); {}", c.script),
        );
        let polyfill_html = run_polyfill_applied_html(c.html, c.script);
        assert_eq!(
            native_html, polyfill_html,
            "三方合一失败 [{}]：native doc 与 polyfill apply 后 doc 不一致\nnative:   `{}`\npolyfill: `{}`",
            c.name, native_html, polyfill_html
        );
    }
}
