// slice49（github home exc1 根因钉：customElements.polyfillWrapFlushCallback 缺失）：
// github ce-vendors 以 `void 0===window.Reflect||void 0===window.customElements||
// window.customElements.polyfillWrapFlushCallback` 真值探测「原生 customElements」并
// 早退 legacy 补丁——Chrome 原生暴露该 Chromium 钩子，补丁块在生产 Chrome 永不执行。
// shim 缺失该方法 → 补丁误触发：window.HTMLElement 被
// `Reflect.construct(HTMLElement,[],this.constructor)` 仿制函数替换，此后所有
// `class X extends HTMLElement`（turbo FrameElement 形态）构造面断：createElement
// 升级路径的 HTMLElement super() 钩子（R94 `_zwCeExisting` 消费点）不再可达，
// this 与 handle 包裹元素脱钩，原型读侧丢失——生产实测
// `createElement("turbo-frame").delegate` 为 undefined →
// `Object.getPrototypeOf(undefined)` 抛 "Cannot convert undefined or null to
// object"（behaviors 模块 eval 内，t2e 同运行 dump 616:17045 定位，2026-10-09）。
// 本钉以 ce-vendors 原样守卫+补丁体与 FrameElement 形态类常驻三断言：
// ①方法面存在（typeof === "function"）；②补丁块执行后 HTMLElement 身份不被替换
// （修复后 guard 真值早退）；③补丁形态顺序下 define + createElement 的 delegate
// 读侧完好（object 而非 undefined）。
// https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry
// （polyfillWrapFlushCallback 为 Chromium 相容面，spec 无此 API；方法存在性即语义）

/// slice49 钉共用沙箱装配（s43_sandbox 同款：shim + 快照 + DOM 回调）。
macro_rules! s49_sandbox {
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
        let page_url = Arc::new(Mutex::new("https://zero.test/s49".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

#[test]
fn ce_polyfill_wrap_flush_callback_guard_p49() {
    let mut sandbox = s49_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            r#"
            var __he49 = window.HTMLElement;
            // github ce-vendors 原样守卫 + 补丁体（minified 形态照抄，t2e dump 6n 条目）。
            !function(){
              if(void 0===window.Reflect||void 0===window.customElements||window.customElements.polyfillWrapFlushCallback)return;
              let e=HTMLElement;
              window.HTMLElement=({HTMLElement:function(){return Reflect.construct(e,[],this.constructor)}}).HTMLElement,
              HTMLElement.prototype=e.prototype,
              HTMLElement.prototype.constructor=HTMLElement,
              Object.setPrototypeOf(HTMLElement,e)
            }();
            // FrameElement 形态（turbo 6n）：ctor 建 delegate 存储，getter 读侧。
            class FrameEl49 extends HTMLElement {
              constructor() { super(); this._d49 = { marked: true }; }
              get delegate() { return this._d49; }
            }
            customElements.define('frame-el-49', FrameEl49);
            var el49 = document.createElement('frame-el-49');
            globalThis.__r_cap = typeof customElements.polyfillWrapFlushCallback;
            globalThis.__r_he_same = String(window.HTMLElement === __he49);
            globalThis.__r_deleg = typeof el49.delegate;
            globalThis.__r_deleg_marked = String(el49.delegate && el49.delegate.marked === true);
            "#,
        )
        .unwrap();
    let verdict = sandbox
        .execute("JSON.stringify([__r_cap,__r_he_same,__r_deleg,__r_deleg_marked])")
        .unwrap()
        .value;
    assert_eq!(
        verdict,
        r#"["function","true","object","true"]"#,
        "polyfillWrapFlushCallback 相容面（exc1 根因）：cap=方法存在 / he_same=HTMLElement 未被 ce-vendors 补丁替换 / deleg=createElement 后 delegate 读侧为 object / deleg_marked=delegate 内容可达"
    );
}
