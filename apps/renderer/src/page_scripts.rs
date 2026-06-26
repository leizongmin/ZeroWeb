//! 渲染进程页面脚本执行 — 加载完成后运行 `<script>` 并处理 DOM 事件。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    apply_mutations_to_html, extract_page_scripts, resolve_document_url, script_dispatch_dom_event,
    DomEventDetail, PageScript, RenderPipeline, RenderResult,
};

use crate::js_worker::{collect_module_deps, RendererJsWorker};

/// DOM 事件派发结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomDispatchResult {
    /// `preventDefault()` 未被调用。
    pub default_allowed: bool,
    /// 页面 HTML 因脚本变更已更新并重渲染。
    pub html_changed: bool,
}

/// 页面脚本执行上下文。
pub struct PageScriptContext<'a> {
    /// 渲染管线。
    pub pipeline: &'a mut RenderPipeline,
    /// 当前 HTML 文档。
    pub html: &'a mut String,
    /// 附加 CSS。
    pub css: &'a str,
    /// 页面 URL。
    pub url: &'a str,
    /// JS worker。
    pub js_worker: &'a RendererJsWorker,
}

/// 按文档顺序执行页面脚本。
pub fn run_page_scripts<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    fetch_text: F,
) -> bool {
    if !javascript_enabled || ctx.html.is_empty() || should_skip_scripts(ctx.url) {
        return false;
    }
    let base = ctx.url.to_string();
    let original_html = ctx.html.clone();
    let mut html = ctx.html.clone();

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
                match fetch_text(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        continue;
                    }
                }
            }
        };

        if let Err(e) = execute_chunk(ctx, &html, is_module, &module_url, &code, &fetch_text) {
            warn!("page script error: {e}");
            continue;
        }

        if let Some(new_html) = apply_recorded_mutations(ctx, &html) {
            html = new_html;
        }
    }

    if html != original_html {
        *ctx.html = html;
        return true;
    }
    false
}

/// 向页面元素派发 DOM 事件。
pub fn dispatch_dom_event(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    selector: &str,
    event_type: &str,
    detail: Option<&DomEventDetail>,
) -> DomDispatchResult {
    if !javascript_enabled || should_skip_scripts(ctx.url) {
        return DomDispatchResult {
            default_allowed: true,
            html_changed: false,
        };
    }
    let script = script_dispatch_dom_event(selector, event_type, detail);
    ctx.js_worker.set_dom_snapshot(ctx.html);
    ctx.js_worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).clear();
    let result_str = match ctx.js_worker.execute_script_direct(&script) {
        Ok(r) => r,
        Err(e) => {
            warn!("dispatch {event_type} on {selector}: {e}");
            return DomDispatchResult {
                default_allowed: true,
                html_changed: false,
            };
        }
    };
    let default_allowed = result_str.trim() != "prevented";
    let html_snap = ctx.html.clone();
    let html_changed = apply_recorded_mutations(ctx, &html_snap).is_some();
    DomDispatchResult {
        default_allowed,
        html_changed,
    }
}

fn execute_chunk<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    html: &str,
    is_module: bool,
    module_url: &str,
    code: &str,
    fetch_text: &F,
) -> Result<(), String> {
    ctx.js_worker.set_dom_snapshot(html);
    ctx.js_worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).clear();
    if is_module {
        let mut registry: HashMap<String, String> = HashMap::new();
        collect_module_deps(fetch_text, module_url, code, &mut registry)?;
        let deps: Vec<(String, String)> = registry.into_iter().collect();
        ctx.js_worker.execute_module(code, module_url, &deps)?;
    } else {
        ctx.js_worker.execute_script_direct(code)?;
    }
    Ok(())
}

fn apply_recorded_mutations(ctx: &mut PageScriptContext<'_>, html: &str) -> Option<String> {
    let recorded = ctx
        .js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if recorded.is_empty() {
        return None;
    }
    match apply_mutations_to_html(html, &recorded) {
        Ok(new_html) => {
            *ctx.html = new_html.clone();
            Some(new_html)
        }
        Err(e) => {
            warn!("apply DOM mutations: {e}");
            None
        }
    }
}

fn should_skip_scripts(url: &str) -> bool {
    url.starts_with("view-source:")
}

/// 经 `zero_net` 直接抓取文本（渲染进程脚本加载用）。
pub fn net_fetch_text(url: &str) -> Result<String, String> {
    zero_net::client::HttpClient::new()
        .get(url)
        .map(|r| String::from_utf8_lossy(&r.body).into_owned())
        .map_err(|e| e.to_string())
}

/// 用当前 HTML/CSS 重渲染页面。
pub fn rerender(ctx: &mut PageScriptContext<'_>) -> RenderResult {
    ctx.pipeline.render_html(ctx.html, ctx.css)
}
