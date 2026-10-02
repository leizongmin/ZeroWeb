//! CDP `Page.addScriptToEvaluateOnNewDocument` 预注入钉住（slice20 / R5022）。
//!
//! 锁定机制面：登记列表跨文档持久（整体替换语义）；每个新文档在**页面脚本执行前**
//! 执行全部已登记脚本。Chromium 语义——「evaluated in the frame whenever the document
//! is created」，实现于任何页面脚本求值前（document readyState 仍为 loading）：
//! https://chromedevtools.github.io/devtools-protocol/tot/Page/#method-addScriptToEvaluateOnNewDocument
//! 旧行为（browser 侧导航完成后重放，readyState=complete）时序与 Chromium 相反，
//! 依赖注入早于页面脚本的站点 boot 插桩全部失真。kill-switch `ZW_CDP_PREDOC_SCRIPTS=0`
//! 回退旧行为（env set_var 在并行测试下有竞态，回退面不在此测——browser 侧
//! `emit_navigation_event_family` 门控保持重放路径结构不变）。

use super::*;
use zero_protocol::message::PreDocumentScriptsParams;

/// 注册预注入脚本（整体替换语义，模拟浏览器侧每次登记重发全量）。
fn register_predoc(runtime: &mut RendererRuntime, sources: Vec<String>) {
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::PreDocumentScripts(PreDocumentScriptsParams { sources }),
        })
        .unwrap();
}

/// LoadHtml 并驱动加载至脚本阶段收口（inline 文档无外部脚本，首几个 tick 内完成）。
fn load_and_run_scripts(runtime: &mut RendererRuntime, html: &str, url: &str) {
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: None,
                url: Some(url.to_string()),
                navigation_epoch: 1,
            }),
        })
        .unwrap();
    let mut ticks = 0;
    loop {
        runtime.tick_pending_load().unwrap();
        runtime.tick_script_prefetch().unwrap();
        if runtime.pending_load.is_none() && runtime.pending_script_prefetch.is_none() {
            break;
        }
        ticks += 1;
        assert!(ticks < 1000, "load/prefetch 未收口");
    }
}

fn runtime_with_sink(renderer_id: u64) -> RendererRuntime {
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime
}

#[test]
fn predoc_script_runs_before_page_scripts() {
    let mut runtime = runtime_with_sink(9301);
    register_predoc(
        &mut runtime,
        vec!["globalThis.__order = (globalThis.__order || '') + 'D';".into()],
    );
    load_and_run_scripts(
        &mut runtime,
        "<html><body><script>globalThis.__order = (globalThis.__order || '') + 'P';</script></body></html>",
        "https://zero.test/predoc-timing",
    );
    assert_eq!(
        runtime.js_worker.execute_script_direct("globalThis.__order").unwrap(),
        "DP",
        "预注入脚本必须先于页面脚本执行（Chromium document-start 语义）"
    );
}

#[test]
fn predoc_registration_replaces_list_and_persists_across_documents() {
    let mut runtime = runtime_with_sink(9302);
    register_predoc(
        &mut runtime,
        vec!["globalThis.__order = (globalThis.__order || '') + 'D1';".into()],
    );
    // 浏览器侧每次登记重发全量 → renderer 整体替换（非追加）：D1 被 D2 覆盖。
    register_predoc(
        &mut runtime,
        vec!["globalThis.__order = (globalThis.__order || '') + 'D2';".into()],
    );
    load_and_run_scripts(
        &mut runtime,
        "<html><body><script>globalThis.__order = (globalThis.__order || '') + 'P';</script></body></html>",
        "https://zero.test/predoc-doc1",
    );
    assert_eq!(
        runtime.js_worker.execute_script_direct("globalThis.__order").unwrap(),
        "D2P",
        "整体替换：仅最新登记列表生效"
    );
    // 跨文档持久：登记不随导航清空，第二个新文档再次先于页面脚本执行。
    load_and_run_scripts(
        &mut runtime,
        "<html><body><script>globalThis.__order = (globalThis.__order || '') + 'P';</script></body></html>",
        "https://zero.test/predoc-doc2",
    );
    assert_eq!(
        runtime.js_worker.execute_script_direct("globalThis.__order").unwrap(),
        "D2P",
        "跨文档持久：新文档重复执行登记列表（每文档一次）"
    );
}

#[test]
fn predoc_script_skipped_when_js_disabled() {
    let mut runtime = runtime_with_sink(9303);
    // 与页面脚本同门槛：JS 关闭 → 无脚本执行面，预注入同样跳过。
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetJavascriptEnabled(false),
        })
        .unwrap();
    register_predoc(&mut runtime, vec!["globalThis.__order = 'D';".into()]);
    load_and_run_scripts(
        &mut runtime,
        "<html><body><script>globalThis.__order = 'P';</script></body></html>",
        "https://zero.test/predoc-js-off",
    );
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("String(globalThis.__order)")
            .unwrap(),
        "undefined",
        "JS 关闭时预注入不执行（与页面脚本同门槛）"
    );
}

#[test]
fn predoc_kill_switch_value_matrix() {
    // env 值矩阵钉（评审 T-I1/缺陷 S1）：钉住 predoc_enabled_for 的现语义——
    // 仅字面 "0" 关断；unset/其余任意值一律 on。纯值核心直测，不做 env set_var
    // （并行测试下有竞态）。若实现意外翻转为白名单（如 `== Ok("1")`），此钉即红。
    assert!(super::predoc_enabled_for(None), "未设 → on（默认开）");
    assert!(!super::predoc_enabled_for(Some("0")), "\"0\" → 关断");
    assert!(super::predoc_enabled_for(Some("1")), "\"1\" → on（非关断值）");
    assert!(
        super::predoc_enabled_for(Some("00")),
        "\"00\" → on（字面精确匹配，防前缀/数值化误判）"
    );
    assert!(
        super::predoc_enabled_for(Some("false")),
        "\"false\" → on（非关断值，语义=仅 \"0\" 关断）"
    );
    assert!(super::predoc_enabled_for(Some("")), "空串 → on（非关断值）");
}
