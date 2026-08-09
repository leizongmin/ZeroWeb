//! P1b S1 bench——原生 DOM 绑定（live Document 直读）vs polyfill 字符串桥（HTML 快照重解析）单次读取开销。
//!
//! RFC `docs/specs/p1b-v8-native-bindings-rfc.md` §4 S0 bench gate：测 native 单次读取
//! 开销并对照 polyfill，量化 P1 痛点（polyfill 每次操作**重解析 HTML 快照串**）。
//!
//! 两路同读 `tagName`（公平对照）：
//! - **native**：`el.tagName` 经 accessor getter 直读 Rust `Document`（NodeId → SlotMap
//!   lookup → 原生 `v8::String`，无 HTML 重解析、无 String 编解码 args）。
//! - **polyfill**：`__zw_poly_tag(sel)` 经 String 回调 → `parse_html(快照串)` 重解析 →
//!   `find_by_selector` → 读 local_name → 返 `v8::String`。镜像真实 `__zw_get_tag`
//!   路径（`js_dom_bridge` 每次 `parse_html(dom_html)` 重解析，P1 根因）。
//!
//! 单 isolate（线程局部 Global 不可跨 isolate）；两路同装一个 context。script 预编译，
//! `iter` 只测 run（公共 scope/compile 开销在比值中抵消）。

use std::cell::RefCell;
use std::rc::Rc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_dom::{NodeKind, parse_html};
use zero_engine::dom_bindings::install_dom_bindings;
use zero_engine::find_by_selector;

/// bench 用 HTML（多元素，确保 lookup 非平凡）。
const HTML: &str = r#"<div id="a"><span id="b">x</span><p id="c"></p><section id="d"></section></div>"#;

// polyfill HTML 快照串（线程局部；polyfill 回调每次重解析）。
thread_local! {
    static POLY_HTML: RefCell<String> = const { RefCell::new(String::new()) };
}

/// polyfill 回调：`__zw_poly_tag(sel)` → 重解析 HTML 快照 → find_by_selector → local_name 大写。
///
/// 镜像真实 `__zw_get_tag`（`parse_html(dom_html)` 每次重解析，P1 根因）。args→Rust String +
/// 重解析 + 查询 + 返 v8::String（完整 String 桥开销）。
fn poly_tag_invoke(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue<v8::Value>) {
    let sel = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let html = POLY_HTML.with(|h| h.borrow().clone());
    // 真实 polyfill：每次重解析 HTML 快照（P1 痛点根因）。
    let doc = parse_html(&html);
    let tag = find_by_selector(&doc, &sel)
        .and_then(|n| {
            doc.get(n).and_then(|nd| match &nd.kind {
                NodeKind::Element(e) => Some(e.local_name().to_ascii_uppercase()),
                _ => None,
            })
        })
        .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &tag) {
        rv.set(s.into());
    }
}

/// 安装 polyfill `__zw_poly_tag` 全局函数（String 桥 + HTML 重解析）。
fn install_polyfill(scope: &mut v8::PinScope, ctx: v8::Local<v8::Context>) {
    POLY_HTML.with(|h| *h.borrow_mut() = HTML.to_string());
    let global = ctx.global(scope);
    let f = v8::FunctionTemplate::builder(poly_tag_invoke)
        .build(scope)
        .get_function(scope)
        .expect("poly factory");
    let Some(key) = v8::String::new(scope, "__zw_poly_tag") else {
        return;
    };
    let _ = global.set(scope, key.into(), f.into());
}

/// 单 isolate bench 助手：持持久 Context + 多预编译脚本。
struct Bench {
    isolate: v8::OwnedIsolate,
    ctx: v8::Global<v8::Context>,
    scripts: Vec<(&'static str, v8::Global<v8::Script>)>,
}

impl Bench {
    /// 建持久 Context，经 `install` 安装绑定。
    fn new(install: impl FnOnce(&mut v8::PinScope, v8::Local<v8::Context>)) -> Self {
        let mut isolate = v8::Isolate::new(Default::default());
        let ctx;
        {
            let _enter = enter_isolate(&mut isolate);
            v8::scope!(let scope, &mut isolate);
            let context = v8::Context::new(scope, Default::default());
            let scope = &mut v8::ContextScope::new(scope, context);
            install(scope, context);
            ctx = v8::Global::new(scope, context);
        }
        Self {
            isolate,
            ctx,
            scripts: Vec::new(),
        }
    }

    /// 预编译脚本（key → Global<Script>）。
    fn compile(&mut self, key: &'static str, expr: &str) {
        let _enter = enter_isolate(&mut self.isolate);
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(scope, &self.ctx);
        let scope = &mut v8::ContextScope::new(scope, context);
        let code = v8::String::new(scope, expr).expect("v8 string");
        let s = v8::Script::compile(scope, code, None).expect("compile");
        self.scripts.push((key, v8::Global::new(scope, s)));
    }

    /// 执行指定 key 的预编译脚本，返回结果字符串。
    fn run(&mut self, key: &str) -> String {
        let _enter = enter_isolate(&mut self.isolate);
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(scope, &self.ctx);
        let scope = &mut v8::ContextScope::new(scope, context);
        let idx = self.scripts.iter().position(|(k, _)| *k == key).expect("script key");
        let script = v8::Local::new(scope, &self.scripts[idx].1);
        let r = script.run(scope).expect("run");
        r.to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default()
    }
}

/// 进入 isolate（V8 `scope!` 宏不自动 enter；镜像 `V8Sandbox::execute` 的 enter/exit）。
fn enter_isolate(isolate: &mut v8::OwnedIsolate) -> IsolateExitGuard {
    // SAFETY: 单线程 bench，enter/exit 配对（guard drop 时 exit）；OwnedIsolate 在 Bench 存活期内。
    unsafe {
        isolate.enter();
    }
    IsolateExitGuard {
        isolate: isolate as *mut _,
    }
}

/// 退出 isolate 的 RAII guard（drop 调 `Isolate::exit`，配对 [`enter_isolate`] 的 enter）。
struct IsolateExitGuard {
    isolate: *mut v8::OwnedIsolate,
}

impl Drop for IsolateExitGuard {
    fn drop(&mut self) {
        // SAFETY: guard 仅由 enter_isolate 创建，isolate 在 Bench 存活期内。
        unsafe {
            (*self.isolate).exit();
        }
    }
}

fn bench_dom_bindings(c: &mut Criterion) {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(HTML)));

    // 单 isolate：native + polyfill 同装一个 context（线程局部 Global 不可跨 isolate）。
    let mut bench = Bench::new(|scope, ctx| {
        install_dom_bindings(scope, ctx, Rc::clone(&dom));
        install_polyfill(scope, ctx);
    });
    bench.compile("native_tag", "(__zw_native_element_for_id('a').tagName)");
    bench.compile("native_node_type", "(__zw_native_element_for_id('a').nodeType)");
    bench.compile("poly_tag", "(__zw_poly_tag('#a'))");

    // 正确性自检（非计时）：两路 tagName 应一致（"DIV"）。
    assert_eq!(bench.run("native_tag"), "DIV", "native tagName");
    assert_eq!(bench.run("poly_tag"), "DIV", "polyfill tagName");
    assert_eq!(bench.run("native_node_type"), "1", "native nodeType");

    c.bench_function("native_tag_name", |b| b.iter(|| black_box(bench.run("native_tag"))));
    c.bench_function("polyfill_tag_name", |b| b.iter(|| black_box(bench.run("poly_tag"))));
    c.bench_function("native_node_type", |b| {
        b.iter(|| black_box(bench.run("native_node_type")))
    });
}

criterion_group!(benches, bench_dom_bindings);
criterion_main!(benches);
