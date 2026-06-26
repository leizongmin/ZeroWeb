//! 页面 `<script>` 执行 — Tab worker 内在加载完成后调用。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    apply_mutations_to_html, extract_page_scripts, resolve_document_url, script_dispatch_dom_event, DomEventDetail,
    PageScript,
};
use zero_webview::WebView;

use crate::tab_js_worker::{collect_module_deps, TabJsWorkerHandle};

/// DOM 事件派发结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomDispatchResult {
    /// `preventDefault()` 未被调用（默认行为可继续）。
    pub default_allowed: bool,
    /// 页面 HTML 因脚本变更已更新并重渲染。
    pub html_changed: bool,
}

/// 按文档顺序执行页面脚本（内联 + 外链 `src`），将 DOM 变更同步回文档并重渲染。
pub fn run_page_scripts(wv: &mut WebView, javascript_enabled: bool, js_worker: Option<&TabJsWorkerHandle>) {
    if !javascript_enabled {
        return;
    }
    let mut html = wv.html_content().to_string();
    if html.is_empty() {
        return;
    }
    if !should_run_scripts_for_url(wv.url()) {
        return;
    }
    let base = wv.url().unwrap_or("about:blank").to_string();
    let original_html = html.clone();

    for script in extract_page_scripts(&html) {
        let is_module = matches!(
            &script,
            PageScript::InlineModule(_) | PageScript::ExternalModule(_)
        );
        let module_url = match &script {
            PageScript::ExternalModule(src) => resolve_document_url(&base, src),
            PageScript::InlineModule(_) => base.clone(),
            _ => String::new(),
        };

        let code = match script {
            PageScript::Inline(code) | PageScript::InlineModule(code) => code,
            PageScript::External(src) | PageScript::ExternalModule(src) => {
                let abs = resolve_document_url(&base, &src);
                match wv.fetch_text_at(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        continue;
                    }
                }
            }
        };

        if let Err(e) = execute_script_chunk(wv, js_worker, &html, is_module, &module_url, &code) {
            warn!("page script error: {e}");
            continue;
        }

        if let Some(new_html) = apply_recorded_mutations(wv, js_worker, &html) {
            html = new_html;
        }
    }

    if html != original_html {
        wv.reload_html_after_script(&html);
    }
}

/// 向页面元素派发 DOM 事件（宿主输入 → JS 监听器 → DOM 变更 → 重渲染）。
pub fn dispatch_dom_event(
    wv: &mut WebView,
    javascript_enabled: bool,
    js_worker: Option<&TabJsWorkerHandle>,
    selector: &str,
    event_type: &str,
    html: &str,
    detail: Option<&DomEventDetail>,
) -> DomDispatchResult {
    if !javascript_enabled || !should_run_scripts_for_url(wv.url()) {
        return DomDispatchResult {
            default_allowed: true,
            html_changed: false,
        };
    }
    let script = script_dispatch_dom_event(selector, event_type, detail);
    let result_str = if let Some(worker) = js_worker {
        worker.set_dom_snapshot(html);
        worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).clear();
        match worker.execute_script_direct(&script) {
            Ok(r) => r,
            Err(e) => {
                warn!("dispatch {event_type} on {selector}: {e}");
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                };
            }
        }
    } else {
        match wv.execute_script(&script) {
            Ok(r) => r,
            Err(e) => {
                warn!("dispatch {event_type} on {selector}: {e}");
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                };
            }
        }
    };
    let default_allowed = result_str.trim() != "prevented";
    let html_changed = apply_recorded_mutations(wv, js_worker, html).is_some();
    DomDispatchResult {
        default_allowed,
        html_changed,
    }
}

fn execute_script_chunk(
    wv: &mut WebView,
    js_worker: Option<&TabJsWorkerHandle>,
    html: &str,
    is_module: bool,
    module_url: &str,
    code: &str,
) -> Result<(), String> {
    if let Some(worker) = js_worker {
        worker.set_dom_snapshot(html);
        worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
    if is_module {
        let worker = js_worker.ok_or("ES module requires JS worker")?;
        let mut registry: HashMap<String, String> = HashMap::new();
        let fetch = |url: &str| wv.fetch_text_at(url).map_err(|e| e.to_string());
        collect_module_deps(&fetch, module_url, code, &mut registry)?;
        let deps: Vec<(String, String)> = registry.into_iter().collect();
        worker.execute_module(code, module_url, &deps)?;
    } else {
        wv.execute_script(code).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn apply_recorded_mutations(
    wv: &mut WebView,
    js_worker: Option<&TabJsWorkerHandle>,
    html: &str,
) -> Option<String> {
    let recorded = js_worker
        .map(|w| w.mutations().lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_default();
    if recorded.is_empty() {
        return None;
    }
    match apply_mutations_to_html(html, &recorded) {
        Ok(new_html) => {
            wv.reload_html_after_script(&new_html);
            Some(new_html)
        }
        Err(e) => {
            warn!("apply DOM mutations: {e}");
            None
        }
    }
}

fn should_run_scripts_for_url(url: Option<&str>) -> bool {
    match url {
        Some(u) if u.starts_with("view-source:") => false,
        _ => true,
    }
}
