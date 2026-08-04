//! 页面 `<script>` 执行 — Tab worker 内在加载完成后调用。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    DomEventDetail, PageScript, apply_mutations_to_html_with_handles, extract_page_scripts, resolve_document_url,
    script_dispatch_dom_event,
};
use zero_webview::WebView;

use crate::tab_js_worker::{TabJsWorkerHandle, collect_module_deps};

/// DOM 事件派发结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomDispatchResult {
    /// `preventDefault()` 未被调用（默认行为可继续）。
    pub default_allowed: bool,
    /// 页面 HTML 因脚本变更已更新并重渲染。
    pub html_changed: bool,
}

/// 分片执行页面脚本，避免在 tab worker 循环内一次性跑完所有 `<script>` 阻塞 UI。
pub struct PageScriptRunner {
    scripts: Vec<PageScript>,
    index: usize,
    base: String,
    html: String,
    original_html: String,
}

impl PageScriptRunner {
    /// 从当前文档抽取脚本队列；无脚本或不应执行时返回 `None`。
    pub fn start(wv: &WebView, javascript_enabled: bool) -> Option<Self> {
        if !javascript_enabled {
            return None;
        }
        let html = wv.html_content().to_string();
        if html.is_empty() || !should_run_scripts_for_url(wv.url()) {
            return None;
        }
        let scripts = extract_page_scripts(&html);
        if scripts.is_empty() {
            return None;
        }
        let base = wv.url().unwrap_or("about:blank").to_string();
        let original_html = html.clone();
        Some(Self {
            scripts,
            index: 0,
            base,
            html,
            original_html,
        })
    }

    /// 是否还有待执行脚本。
    pub fn is_active(&self) -> bool {
        self.index < self.scripts.len()
    }

    /// 执行下一个 `<script>`；返回是否建议推送快照。
    pub fn tick(&mut self, wv: &mut WebView, js_worker: Option<&TabJsWorkerHandle>) -> PageScriptTickResult {
        if !self.is_active() {
            return PageScriptTickResult::Idle;
        }

        let script = &self.scripts[self.index];
        let label = page_script_label(script);
        let is_module = matches!(script, PageScript::InlineModule(_) | PageScript::ExternalModule(_));
        let module_url = match script {
            PageScript::ExternalModule(src) => resolve_document_url(&self.base, src),
            PageScript::InlineModule(_) => self.base.clone(),
            _ => String::new(),
        };

        let code = match script {
            PageScript::Inline(code) | PageScript::InlineModule(code) => code.clone(),
            PageScript::External(src) | PageScript::ExternalModule(src) => {
                let abs = resolve_document_url(&self.base, src);
                match wv.fetch_text_at(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        self.index += 1;
                        return PageScriptTickResult::Continue;
                    }
                }
            }
        };

        if let Err(e) = execute_script_chunk(wv, js_worker, &self.html, is_module, &module_url, &code, true) {
            warn!("page script error ({}): {e}", label);
        }

        if let Some(new_html) = apply_recorded_mutations(wv, js_worker, &self.html) {
            self.html = new_html;
        }

        self.index += 1;
        PageScriptTickResult::Continue
    }

    /// 全部脚本跑完后，若 DOM 有变更则一次性重载 HTML。
    pub fn finish(&mut self, wv: &mut WebView) {
        if self.html != self.original_html {
            wv.reload_html_after_script(&self.html);
        }
    }
}

/// 分片执行单步结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageScriptTickResult {
    /// 无待执行脚本。
    Idle,
    /// 已执行一步（或跳过一步），可继续 tick。
    Continue,
}

/// 按文档顺序执行页面脚本（内联 + 外链 `src`），将 DOM 变更同步回文档并重渲染。
///
/// 测试与简单页面仍可用；生产路径请用 [`PageScriptRunner`] 分片执行。
pub fn run_page_scripts(wv: &mut WebView, javascript_enabled: bool, js_worker: Option<&TabJsWorkerHandle>) {
    let mut runner = match PageScriptRunner::start(wv, javascript_enabled) {
        Some(r) => r,
        None => return,
    };
    while runner.is_active() {
        runner.tick(wv, js_worker);
    }
    runner.finish(wv);
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
        let page_url = wv.url().unwrap_or("about:blank");
        worker.set_dom_snapshot(html, page_url);
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
    page_script: bool,
) -> Result<(), String> {
    let page_url = wv.url().unwrap_or("about:blank");
    if let Some(worker) = js_worker {
        worker.set_dom_snapshot(html, page_url);
        worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
    if is_module {
        let worker = js_worker.ok_or("ES module requires JS worker")?;
        let mut registry: HashMap<String, String> = HashMap::new();
        let fetch = |url: &str| wv.fetch_text_at(url).map_err(|e| e.to_string());
        collect_module_deps(&fetch, module_url, code, &mut registry)?;
        let deps: Vec<(String, String)> = registry.into_iter().collect();
        worker.execute_module(code, module_url, &deps)?;
    } else if let Some(worker) = js_worker {
        if page_script {
            worker.execute_page_script(code).map_err(|e| e.to_string())?;
        } else {
            worker.execute_script_direct(code).map_err(|e| e.to_string())?;
        }
    } else {
        wv.execute_script(code).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn apply_recorded_mutations(wv: &mut WebView, js_worker: Option<&TabJsWorkerHandle>, html: &str) -> Option<String> {
    let recorded = js_worker
        .map(|w| w.mutations().lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_default();
    if recorded.is_empty() {
        return None;
    }
    match apply_mutations_to_html_with_handles(html, &recorded) {
        Ok((new_html, handle_selectors)) => {
            // P1a gBCR path A：merge handle→唯一选择器映射进 worker 持久 map，供 RectBridge
            // handler 解析 handle-identity（createElement 元素）。upsert；导航时 worker 清空。
            if !handle_selectors.is_empty() {
                if let Some(w) = js_worker {
                    if let Ok(mut map) = w.handle_selector_map().lock() {
                        map.extend(handle_selectors);
                    }
                }
            }
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
    !matches!(url, Some(u) if u.starts_with("view-source:"))
}

fn page_script_label(script: &PageScript) -> String {
    match script {
        PageScript::Inline(code) => {
            let line = code.lines().next().unwrap_or("").trim();
            let preview: String = line.chars().take(72).collect();
            format!("inline: {preview}")
        }
        PageScript::InlineModule(code) => {
            let line = code.lines().next().unwrap_or("").trim();
            let preview: String = line.chars().take(72).collect();
            format!("inline module: {preview}")
        }
        PageScript::External(src) => format!("external: {src}"),
        PageScript::ExternalModule(src) => format!("external module: {src}"),
    }
}
