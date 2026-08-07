//! 页面 `<script>` 执行 — Tab worker 内在加载完成后调用。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    DomEventDetail, PageScript, apply_mutations_to_html_with_handles, extract_page_scripts, page_script_error_check,
    resolve_document_url, script_dispatch_dom_event, script_report_error, script_wrap_page_caught,
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
    /// R2942：子资源 fetch/decode 失败记录 `(kind, url)`（stylesheet/image，由 tab_worker 从
    /// AsyncPageLoad.take_failed_resources drain 后经 `set_resource_errors` 注入）。finish() 在页面
    /// load 之后派发对应 window 'error' 事件（确保脚本注册的 onerror handler 已就位）。
    resource_errors: Vec<(String, String)>,
    /// R2943：img 元素级 load/error 事件 `(绝对 URL, "load"/"error")`（由 tab_worker 从
    /// AsyncPageLoad.take_img_element_events drain 后注入）。finish() 经 `__zw_dispatch_img_event` 派发到
    /// 匹配 src 的 `<img>` 元素（img.onload/onerror）。
    img_events: Vec<(String, &'static str)>,
    /// R2944：stylesheet 元素级 load/error 事件 `(绝对 URL, "load"/"error")`（由 tab_worker 从
    /// AsyncPageLoad.take_link_element_events drain 后注入）。finish() 经 `__zw_dispatch_link_event` 派发到
    /// 匹配 href 的 `<link>` 元素（link.onload/onerror）。
    link_events: Vec<(String, &'static str)>,
    /// R2947：@font-face 加载结果 `(family, "loaded"/"error")`（由 tab_worker 从
    /// AsyncPageLoad.take_font_events drain 后注入）。finish() 经 `__zw_font_settle` 派 FontFaceSet
    /// 'loadingdone'/'loadingerror' + 解析 `document.fonts.ready`。
    font_events: Vec<(String, &'static str)>,
}

impl PageScriptRunner {
    /// 从当前文档抽取脚本队列；不应执行脚本时返回 `None`。无 `<script>` 但 JS 启用的页面仍返回
    /// 一个立即 inactive 的 runner——使 `finish()` 成为统一的页面生命周期（DOMContentLoaded/load）派发点
    ///（R2941），覆盖无脚本但含 `<body onload>` 内联 handler 的页面。
    pub fn start(wv: &WebView, javascript_enabled: bool) -> Option<Self> {
        if !javascript_enabled {
            return None;
        }
        let html = wv.html_content().to_string();
        if html.is_empty() || !should_run_scripts_for_url(wv.url()) {
            return None;
        }
        let scripts = extract_page_scripts(&html);
        let base = wv.url().unwrap_or("about:blank").to_string();
        let original_html = html.clone();
        Some(Self {
            scripts,
            index: 0,
            base,
            html,
            original_html,
            resource_errors: Vec::new(),
            img_events: Vec::new(),
            link_events: Vec::new(),
            font_events: Vec::new(),
        })
    }

    /// R2942：注入子资源 fetch/decode 失败记录（stylesheet/image），finish() 在 load 之后派发 window 'error'。
    pub fn set_resource_errors(&mut self, errors: Vec<(String, String)>) {
        self.resource_errors = errors;
    }

    /// R2943：注入 img 元素级 load/error 事件 `(绝对 URL, "load"/"error")`，finish() 经
    /// `__zw_dispatch_img_event` 派发到匹配 src 的 `<img>` 元素。
    pub fn set_img_events(&mut self, events: Vec<(String, &'static str)>) {
        self.img_events = events;
    }

    /// R2944：注入 stylesheet 元素级 load/error 事件 `(绝对 URL, "load"/"error")`，finish() 经
    /// `__zw_dispatch_link_event` 派发到匹配 href 的 `<link>` 元素。
    pub fn set_link_events(&mut self, events: Vec<(String, &'static str)>) {
        self.link_events = events;
    }

    /// R2947：注入 @font-face 加载结果 `(family, "loaded"/"error")`，finish() 经 `__zw_font_settle`
    /// 派 FontFaceSet 'loadingdone'/'loadingerror' + 解析 `document.fonts.ready`。
    pub fn set_font_events(&mut self, events: Vec<(String, &'static str)>) {
        self.font_events = events;
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
        // R2944：外部脚本的绝对 src（fetch 成功+执行后派 script 元素 'load'；fetch 失败已在内联分支派 'error'）。
        let external_abs: Option<String> = match script {
            PageScript::External(src) | PageScript::ExternalModule(src) => Some(resolve_document_url(&self.base, src)),
            _ => None,
        };

        let code = match script {
            PageScript::Inline(code) | PageScript::InlineModule(code) => code.clone(),
            PageScript::External(src) | PageScript::ExternalModule(src) => {
                let abs = resolve_document_url(&self.base, src);
                match wv.fetch_text_at(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        // R2942：外部脚本 fetch 失败 → 即时派发 window 'error'（脚本 fetch 在 tick 期同步失败，
                        // 早于后续内联脚本的 onerror 注册即触发，匹配 real browser「fetch 失败即报」语义）。
                        report_resource_error(js_worker, "script", &abs);
                        // R2944：外部脚本元素 'error'（fetch 失败，spec script 元素 error 仅 fetch 失败触发）。
                        dispatch_script_event(js_worker, &abs, "error");
                        self.index += 1;
                        return PageScriptTickResult::Continue;
                    }
                }
            }
        };

        if let Err(e) = execute_script_chunk(wv, js_worker, &self.html, is_module, &module_url, &code, true) {
            warn!("page script error ({}): {e}", label);
            // R2940：报告未捕获脚本错误到 window.onerror / window 'error' 事件（Sentry/analytics hook）。
            report_uncaught_script_error(js_worker, &self.base, &e);
        } else if let Some(abs) = external_abs.as_deref() {
            // R2944：外部脚本 fetch+执行成功 → script 元素 'load'（spec：classic/module 脚本执行成功后派 load）。
            dispatch_script_event(js_worker, abs, "load");
        }

        if let Some(new_html) = apply_recorded_mutations(wv, js_worker, &self.html) {
            self.html = new_html;
        }

        self.index += 1;
        PageScriptTickResult::Continue
    }

    /// 全部脚本跑完后，若 DOM 有变更则一次性重载 HTML；随后派发页面生命周期事件（R2941）。
    pub fn finish(&mut self, wv: &mut WebView, js_worker: Option<&TabJsWorkerHandle>) {
        if self.html != self.original_html {
            wv.reload_html_after_script(&self.html);
        }
        // R2941：脚本阶段完成 → 派发 DOMContentLoaded + load（DOMContentLoaded 先于 load，spec）。
        // analytics onload / jQuery ready / 框架 mount 高频 hook 经此触发。
        dispatch_page_lifecycle(js_worker);
        // R2942：在 load 之后派发子资源 fetch/decode 失败的 window 'error'（stylesheet/image）——
        // 此时脚本注册的 onerror handler 已就位（资源 fetch 失败发生在 async_load 期，早于脚本，
        // 故延后到 load 之后派发以确保 handler 可触）。
        for (kind, url) in &self.resource_errors {
            report_resource_error(js_worker, kind, url);
        }
        // R2943：img 元素级 load/error——经 `__zw_dispatch_img_event` 派发到匹配 src 的 <img> 元素
        //（img.onload/onerror）。延后到 load 之后（同 R2942 理由，确保 handler 已注册）。
        for (url, ty) in &self.img_events {
            dispatch_img_event(js_worker, url, ty);
        }
        // R2944：stylesheet 元素级 load/error——经 `__zw_dispatch_link_event` 派发到匹配 href 的 <link> 元素
        //（link.onload/onerror）。延后到 load 之后（同上理由）。
        for (url, ty) in &self.link_events {
            dispatch_link_event(js_worker, url, ty);
        }
        // R2947：@font-face 加载 settle——经 `__zw_font_settle` 派 FontFaceSet 'loadingdone'/'loadingerror'
        // + 解析 document.fonts.ready。无 @font-face 页面仍 settle（仅 resolve ready，不派事件）。
        // R2950：先把每个 @font-face 字体反映为 FontFace 对象加入 document.fonts（补全 set 语义），再 settle。
        for (family, status) in &self.font_events {
            dispatch_add_fontface(js_worker, family, status);
        }
        let had_loaded = self.font_events.iter().any(|(_, t)| *t == "loaded");
        let had_error = self.font_events.iter().any(|(_, t)| *t == "error");
        dispatch_font_settle(js_worker, had_loaded, had_error);
    }
}

/// R2950：把已加载 @font-face 字体反映为 FontFace 对象加入 document.fonts。经 `script_add_fontface` 生成
/// `__zw_add_fontface(family, status)`——shim 构造 FontFace(family) + 设 status + add（按 family 去重）。
/// best-effort（失败仅 `warn!`）。镜像 renderer `page_scripts::dispatch_add_fontface`。
fn dispatch_add_fontface(js_worker: Option<&TabJsWorkerHandle>, family: &str, status: &str) {
    let Some(worker) = js_worker else { return };
    let report = zero_engine::script_add_fontface(family, status);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("dispatch add fontface ({status} {family}): {e}");
    }
}

/// R2947：派发 @font-face 加载 settle 进 shim。经 `script_font_settle` 生成 `__zw_font_settle(...)`——
/// shim 派 FontFaceSet 'loadingdone'（had_loaded）/ 'loadingerror'（had_error）+ 解析 `document.fonts.ready`。
/// best-effort（失败仅 `warn!`）。镜像 renderer `page_scripts::dispatch_font_settle`。
fn dispatch_font_settle(js_worker: Option<&TabJsWorkerHandle>, had_loaded: bool, had_error: bool) {
    let Some(worker) = js_worker else { return };
    let report = zero_engine::script_font_settle(had_loaded, had_error);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("dispatch font settle: {e}");
    }
}

/// R2941：派发页面生命周期事件（DOMContentLoaded + load）进 shim。均派发到 'html' 选择器
///（document/window listener 同存 `_elKey('html', null)` 键）→ `document.addEventListener('DOMContentLoaded')` /
/// `window.addEventListener('load')` / `window.onload` / `document.onDOMContentLoaded` / `<body onload>`（R2946
/// 反射）触发。派发前先调 `__zw_reflect_body_handlers`——覆盖无 `<script>` 页面（不经 __zw_begin_script，
/// 反射不会随脚本执行触发）。best-effort（报告失败仅 `warn!`，不影响后续）。
fn dispatch_page_lifecycle(js_worker: Option<&TabJsWorkerHandle>) {
    let Some(worker) = js_worker else { return };
    let reflect = zero_engine::script_reflect_body_handlers();
    let dcl = script_dispatch_dom_event("html", "DOMContentLoaded", None);
    let load = script_dispatch_dom_event("html", "load", None);
    if let Err(e) = worker.execute_script_direct(&format!("{reflect} {dcl}; {load}")) {
        warn!("dispatch page lifecycle: {e}");
    }
}

/// R2942：派发子资源 fetch/decode 失败的 window 'error' 事件进 shim（经 R2940 `__zw_report_error` hook →
/// window.onerror legacy 5-arg + window 'error' ErrorEvent）。`kind` = "script" / "stylesheet" / "image"。
/// best-effort（失败仅 `warn!`）。spec：资源 fetch 失败派发 window 'error'（同源资源；headless lenient）。
fn report_resource_error(js_worker: Option<&TabJsWorkerHandle>, kind: &str, url: &str) {
    let Some(worker) = js_worker else { return };
    let msg = format!("Error loading {kind}: {url}");
    let report = script_report_error(&msg, url, 0, 0);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("report resource error ({kind} {url}): {e}");
    }
}

/// R2943：派发 img 元素级 load/error 事件进 shim。经 `script_dispatch_img_event` 生成
/// `__zw_dispatch_img_event(url, type)` 调用串——shim 按 src 绝对 URL 匹配 `<img>` 元素 proxy，
/// 用其自身 selector 派发（保证 listener key 匹配，img.onload/onerror + addEventListener('load'/'error') 触发）。
/// `ty` = "load"（fetch+decode 成功）/ "error"（fetch 或 decode 失败）。best-effort。
fn dispatch_img_event(js_worker: Option<&TabJsWorkerHandle>, url: &str, ty: &str) {
    let Some(worker) = js_worker else { return };
    let report = zero_engine::script_dispatch_img_event(url, ty);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("dispatch img event ({ty} {url}): {e}");
    }
}

/// R2944：派发 stylesheet 元素级 load/error 事件进 shim。经 `script_dispatch_link_event` 生成
/// `__zw_dispatch_link_event(url, type)`——shim 按 href 绝对 URL 匹配 `<link>` 元素 proxy 派发
///（link.onload/onerror 触发）。`ty` = "load" / "error"。best-effort。
fn dispatch_link_event(js_worker: Option<&TabJsWorkerHandle>, url: &str, ty: &str) {
    let Some(worker) = js_worker else { return };
    let report = zero_engine::script_dispatch_link_event(url, ty);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("dispatch link event ({ty} {url}): {e}");
    }
}

/// R2944：派发外部 `<script src>` 元素级 load/error 事件进 shim。经 `script_dispatch_script_event` 生成
/// `__zw_dispatch_script_event(url, type)`——shim 按 src 绝对 URL 匹配 `<script>` 元素 proxy 派发
///（script.onload/onerror 触发）。`ty` = "load"（fetch+执行成功）/ "error"（fetch 失败）。best-effort。
fn dispatch_script_event(js_worker: Option<&TabJsWorkerHandle>, url: &str, ty: &str) {
    let Some(worker) = js_worker else { return };
    let report = zero_engine::script_dispatch_script_event(url, ty);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("dispatch script event ({ty} {url}): {e}");
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
    runner.finish(wv, js_worker);
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
            // classic 页面脚本：顶层 try-catch 包装捕获抛错（防持久 Isolate 中毒 + 让 R2940 报告生效；
            // 见 zero_engine::script_wrap_page_caught）。成功 Ok；抛错 Err(msg)。
            run_page_script_caught(worker, code)?;
        } else {
            worker.execute_script_direct(code).map_err(|e| e.to_string())?;
        }
    } else {
        wv.execute_script(code).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 执行 classic 页面 `<script>` 体，顶层 try-catch 包装未捕获 throw（[`script_wrap_page_caught`]）。
/// 成功 → `Ok(())`；抛错 → sentinel 读出消息 → `Err(msg)`（调用方 `PageScriptRunner::tick` 据此报
/// window.onerror，R2940）。包装器 execute 不会抛（try-catch 兜底），随后的 sentinel 读取 execute
/// 在干净 Isolate 上可靠。镜像 renderer `page_scripts::run_page_script_caught`。
fn run_page_script_caught(worker: &TabJsWorkerHandle, code: &str) -> Result<(), String> {
    let _ = worker.execute_page_script(&script_wrap_page_caught(code));
    match worker.execute_page_script(&page_script_error_check()) {
        Ok(v) if v.is_empty() => Ok(()),
        Ok(msg) => Err(msg),
        Err(e) => Err(e),
    }
}

/// R2940：将未捕获脚本错误经 worker 报告进 shim——`window.onerror`（legacy 5-arg）+ window 'error' 事件，
/// 使 Sentry / analytics / GA 等错误上报库的 hook 触发。报告本身是 best-effort：失败仅记日志，不影响后续
/// 脚本执行（worker 已在 execute_script_chunk 开头 set_dom_snapshot，此处复用该上下文直接执行报告串）。
fn report_uncaught_script_error(js_worker: Option<&TabJsWorkerHandle>, source: &str, message: &str) {
    let Some(worker) = js_worker else { return };
    let report = script_report_error(message, source, 0, 0);
    if let Err(e) = worker.execute_script_direct(&report) {
        warn!("report uncaught script error: {e}");
    }
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
