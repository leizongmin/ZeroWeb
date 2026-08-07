//! 渲染进程页面脚本执行 — 加载完成后运行 `<script>` 并处理 DOM 事件。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    DomEventDetail, DomMutation, PageScript, apply_mutations_to_html, apply_mutations_to_html_with_handles,
    enclosing_form_selector, extract_page_scripts, has_attribute, is_checkbox, is_radio, is_submit_button,
    page_script_error_check, query_tag_from_html, resolve_document_url, script_dispatch_dom_event,
    script_dispatch_img_event, script_dispatch_link_event, script_dispatch_script_event, script_report_error,
    script_text_delete, script_text_input, script_wrap_page_caught, toggle_radio_html,
};

use crate::js_worker::{RendererJsWorker, collect_module_deps};

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
    /// 当前 HTML 文档（脚本执行后同步更新）。
    pub html: &'a mut String,
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
        let is_module = matches!(&script, PageScript::InlineModule(_) | PageScript::ExternalModule(_));
        let module_url = match &script {
            PageScript::ExternalModule(src) => resolve_document_url(&base, src),
            PageScript::InlineModule(_) => base.clone(),
            _ => String::new(),
        };
        // R2944 mirror：外部脚本的绝对 src（fetch 成功+执行后派 script 元素 'load'；fetch 失败在下方分支派 'error'）。
        let external_abs: Option<String> = match &script {
            PageScript::External(src) | PageScript::ExternalModule(src) => Some(resolve_document_url(&base, src)),
            _ => None,
        };

        let code = match script {
            PageScript::Inline(code) | PageScript::InlineModule(code) => code,
            PageScript::External(_) | PageScript::ExternalModule(_) => {
                let abs = external_abs.clone().unwrap_or_default();
                match fetch_text(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        // R2942 mirror：外部脚本 fetch 失败 → 即时派 window 'error'（脚本 fetch 同步失败，
                        // 早于后续脚本 onerror 注册即触发，匹配 real browser「fetch 失败即报」语义）。
                        report_resource_error(ctx.js_worker, "script", &abs);
                        // R2944 mirror：外部脚本元素 'error'（spec：script 元素 error 仅 fetch 失败触发）。
                        dispatch_script_event(ctx.js_worker, &abs, "error");
                        continue;
                    }
                }
            }
        };

        if let Err(e) = execute_chunk(ctx, &html, is_module, &module_url, &code, &fetch_text) {
            warn!("page script error: {e}");
            // R2940 mirror：未捕获脚本错误 → window.onerror（legacy 5-arg）+ window 'error' ErrorEvent，
            // 使 Sentry / analytics / GA 等错误上报库 hook 触发（与 browser tab_scripts 对齐）。
            report_uncaught_error(ctx.js_worker, &base, &e);
            continue;
        } else if let Some(abs) = external_abs.as_deref() {
            // R2944 mirror：外部脚本 fetch+执行成功 → script 元素 'load'（spec：classic/module 脚本执行成功后派 load）。
            dispatch_script_event(ctx.js_worker, abs, "load");
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

/// R2940–R2944 mirror：页面脚本阶段收尾——派发页面生命周期 + 子资源/元素级事件进 shim，与 browser
/// `tab_scripts::PageScriptRunner::finish` 对齐（renderer 默认多进程路径此前缺这套派发）。
///
/// - **R2941**：DOMContentLoaded + load（DOMContentLoaded 先于 load，spec）。analytics onload / jQuery
///   ready / 框架 mount 高频 hook 经此触发。即使页面无 `<script>`（仅 `<body onload>` 内联 handler），
///   JS 启用页也应派发——调用方在 `run_page_scripts` 之后无条件调用本函数（gate 由调用方按 JS 启用判断）。
/// - **R2942**：`resource_errors` = `(kind, url)` 子资源 fetch/decode 失败 → window 'error'（stylesheet/image），
///   延后到 load 之后派发（资源 fetch 失败发生在 async_load 期早于脚本，故延后确保 handler 已注册）。
/// - **R2943/R2944**：`img_events` / `link_events` = `(绝对 URL, "load"/"error")` 元素级 load/error——经
///   `__zw_dispatch_img_event` / `__zw_dispatch_link_event` 派发到匹配 src/href 的元素（img/link.onload/onerror）。
/// - **R2947**：`font_events` = `(family, "loaded"/"error")` @font-face 加载结果——经 `__zw_font_settle`
///   派发 FontFaceSet 'loadingdone'/'loadingerror' + 解析 `document.fonts.ready` Promise。
///
/// 全部 best-effort（失败仅 `warn!`，不影响后续）。事件由调用方从 `AsyncPageLoad` drain 后传入
///（renderer main 在 load 完成时 drain、stash，脚本阶段消费）。
pub fn finish_page_load(
    js_worker: &RendererJsWorker,
    resource_errors: Vec<(String, String)>,
    img_events: Vec<(String, &'static str)>,
    link_events: Vec<(String, &'static str)>,
    font_events: Vec<(String, &'static str)>,
) {
    // R2941：脚本阶段完成 → 派发 DOMContentLoaded + load。
    dispatch_page_lifecycle(js_worker);
    // R2942：在 load 之后派发子资源 fetch/decode 失败的 window 'error'（stylesheet/image）。
    for (kind, url) in &resource_errors {
        report_resource_error(js_worker, kind, url);
    }
    // R2943：img 元素级 load/error——延后到 load 之后（同 R2942 理由，确保 handler 已注册）。
    for (url, ty) in &img_events {
        dispatch_img_event(js_worker, url, ty);
    }
    // R2944：stylesheet 元素级 load/error——延后到 load 之后（同上理由）。
    for (url, ty) in &link_events {
        dispatch_link_event(js_worker, url, ty);
    }
    // R2947：@font-face 加载 settle——派 FontFaceSet 'loadingdone'/'loadingerror' + 解析 document.fonts.ready。
    // 无 @font-face 页面（font_events 空）仍 settle（仅 resolve ready，不派事件）。
    // R2950：先把每个 @font-face 字体反映为 FontFace 对象加入 document.fonts（补全 set 语义），再 settle。
    for (family, status) in &font_events {
        dispatch_add_fontface(js_worker, family, status);
    }
    let had_loaded = font_events.iter().any(|(_, t)| *t == "loaded");
    let had_error = font_events.iter().any(|(_, t)| *t == "error");
    dispatch_font_settle(js_worker, had_loaded, had_error);
}

/// R2941 mirror：派发页面生命周期事件（DOMContentLoaded + load）进 shim。均派发到 'html' 选择器
///（document/window listener 同存 `_elKey('html', null)` 键）→ `document.addEventListener('DOMContentLoaded')` /
/// `window.addEventListener('load')` / `window.onload` / `document.onDOMContentLoaded` / `<body onload>`（R2946
/// 反射）触发。派发前先调 `__zw_reflect_body_handlers`——覆盖无 `<script>` 页面（不经 __zw_begin_script，
/// 反射不会随脚本执行触发）。best-effort。
fn dispatch_page_lifecycle(js_worker: &RendererJsWorker) {
    let reflect = zero_engine::script_reflect_body_handlers();
    let dcl = script_dispatch_dom_event("html", "DOMContentLoaded", None);
    let load = script_dispatch_dom_event("html", "load", None);
    if let Err(e) = js_worker.execute_script_direct(&format!("{reflect} {dcl}; {load}")) {
        warn!("dispatch page lifecycle: {e}");
    }
}

/// R2940 mirror：未捕获脚本错误经 worker 报告进 shim——`window.onerror`（legacy 5-arg）+ window 'error' 事件，
/// 使 Sentry / analytics / GA 等错误上报库 hook 触发。best-effort。
fn report_uncaught_error(js_worker: &RendererJsWorker, source: &str, message: &str) {
    let report = script_report_error(message, source, 0, 0);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("report uncaught script error: {e}");
    }
}

/// R2942 mirror：派发子资源 fetch/decode 失败的 window 'error' 事件进 shim（经 `__zw_report_error` hook →
/// window.onerror legacy 5-arg + window 'error' ErrorEvent）。`kind` = "script" / "stylesheet" / "image"。best-effort。
fn report_resource_error(js_worker: &RendererJsWorker, kind: &str, url: &str) {
    let msg = format!("Error loading {kind}: {url}");
    let report = script_report_error(&msg, url, 0, 0);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("report resource error ({kind} {url}): {e}");
    }
}

/// R2943 mirror：派发 img 元素级 load/error 事件进 shim。经 `script_dispatch_img_event` 生成
/// `__zw_dispatch_img_event(url, type)`——shim 按 src 绝对 URL 匹配 `<img>` 元素 proxy 派发。best-effort。
fn dispatch_img_event(js_worker: &RendererJsWorker, url: &str, ty: &str) {
    let report = script_dispatch_img_event(url, ty);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("dispatch img event ({ty} {url}): {e}");
    }
}

/// R2944 mirror：派发 stylesheet 元素级 load/error 事件进 shim。经 `script_dispatch_link_event` 生成
/// `__zw_dispatch_link_event(url, type)`——shim 按 href 绝对 URL 匹配 `<link>` 元素 proxy 派发。best-effort。
fn dispatch_link_event(js_worker: &RendererJsWorker, url: &str, ty: &str) {
    let report = script_dispatch_link_event(url, ty);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("dispatch link event ({ty} {url}): {e}");
    }
}

/// R2944 mirror：派发外部 `<script src>` 元素级 load/error 事件进 shim。经 `script_dispatch_script_event`
/// 生成 `__zw_dispatch_script_event(url, type)`——shim 按 src 绝对 URL 匹配 `<script>` 元素 proxy 派发。best-effort。
fn dispatch_script_event(js_worker: &RendererJsWorker, url: &str, ty: &str) {
    let report = script_dispatch_script_event(url, ty);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("dispatch script event ({ty} {url}): {e}");
    }
}

/// R2947 mirror：派发 @font-face 加载 settle 进 shim。经 `script_font_settle` 生成 `__zw_font_settle(...)`——
/// shim 派 FontFaceSet 'loadingdone'（had_loaded）/ 'loadingerror'（had_error）+ 解析 `document.fonts.ready`。
/// best-effort。无 @font-face 页面（had_loaded=had_error=false）仅解析 ready（字体集从不 loading）。
fn dispatch_font_settle(js_worker: &RendererJsWorker, had_loaded: bool, had_error: bool) {
    let report = zero_engine::script_font_settle(had_loaded, had_error);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("dispatch font settle: {e}");
    }
}

/// R2950 mirror：把已加载 @font-face 字体反映为 FontFace 对象加入 document.fonts。经
/// `script_add_fontface` 生成 `__zw_add_fontface(family, status)`——shim 构造 FontFace(family) + 设
/// status + add（按 family 去重）。best-effort。补全 FontFaceSet 语义（set 含文档 @font-face 字体）。
fn dispatch_add_fontface(js_worker: &RendererJsWorker, family: &str, status: &str) {
    let report = zero_engine::script_add_fontface(family, status);
    if let Err(e) = js_worker.execute_script_direct(&report) {
        warn!("dispatch add fontface ({status} {family}): {e}");
    }
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
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
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

/// P1a Slice 2b：render 后触发 observer 重算。镜像 `dispatch_dom_event` 的
/// set_snapshot→clear→execute→apply 流程，script = `__zw_observers_tick()`。
/// IO/RO 的 `_schedule()` 复算所有 target，仅在 cross-threshold（IO）/ size-change（RO）时
/// 派发后续通知——让 `observe()` 之后的真实 render（snapshot 已填真实 rect）触发 observer 回调。
/// 返回 observer 回调是否改了 DOM（调用方据此单次 rerender，防反馈环）。
///
/// R2713b：同一 post-render tick 附带 `__zw_raf_tick`——帧驱动 rAF（`ZW_RAF_FRAME_DRIVEN=1`）的
/// 待 fire 回调在此派发（OFF 时 shim 早返零开销）。ts 传 `performance.now()`（R2768 land 的
/// DOMHighResTimeStamp，单调 ms 自 time origin 起）；performance 缺失（旧 shim）兜底 0。observer
/// tick 先于 rAF，rAF 回调见到的 DOM 反映 observer 本帧变更；两者 mutation 合并由
/// `apply_recorded_mutations` 单次 rerender。
pub fn tick_observers(ctx: &mut PageScriptContext<'_>) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx.js_worker.execute_script_direct(
        "if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();\
         if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(globalThis.performance?performance.now():0);",
    );
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a form input：向焦点 input/textarea 注入一个文本字符（更新 value 属性 + 派发 'input' 事件）。
/// 镜像 `dispatch_dom_event` 的 set_snapshot→clear→execute→apply 流程，script = `__zw_text_input`。
/// 非 input/textarea 目标 shim 内 no-op。返回 value 属性是否变更（调用方据此单次 rerender）。
/// 调用方须先判定 `key` 为单字符可打印键（见 `main::is_printable_key`）。
pub fn apply_text_input(ctx: &mut PageScriptContext<'_>, selector: &str, key: &str) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx.js_worker.execute_script_direct(&script_text_input(selector, key));
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a form input：Backspace 删焦点 input/textarea 的末字符 + 派发 'input' 事件。
/// 镜像 `apply_text_input`。返回 value 属性是否变更（调用方据此单次 rerender）。
pub fn apply_text_delete(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx.js_worker.execute_script_direct(&script_text_delete(selector));
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a form submit：Enter 在单行 `<input>`（非 textarea）→ 解析 enclosing `<form>` → 派发
/// 'submit' 事件。textarea 的 Enter 为换行不提交；input 无 enclosing form 不提交。
/// 返回 submit 回调是否改 DOM（调用方据此单次 rerender）。
pub fn apply_submit_on_enter(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    // 仅单行 input 的 Enter 触发 submit（textarea 的 Enter 为换行）。
    if !query_tag_from_html(ctx.html, selector).eq_ignore_ascii_case("input") {
        return false;
    }
    submit_enclosing_form(ctx, selector)
}

/// P1a form submit：click 命中 submit button（`<input type=submit/image>` / `<button>` type≠button）
/// → 解析 enclosing `<form>` → 派发 'submit' 事件。返回 submit 回调是否改 DOM。
pub fn apply_submit_on_click(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    if !is_submit_button(ctx.html, selector) {
        return false;
    }
    submit_enclosing_form(ctx, selector)
}

/// 共享 submit 核心：解析 enclosing `<form>` → 派发 'submit'（复用 `script_dispatch_dom_event`）
/// → apply。无触发 gate（调用方先判 Enter-in-input / submit-button）。无 enclosing form → false。
fn submit_enclosing_form(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    let snap = ctx.html.clone();
    let Some(form_sel) = enclosing_form_selector(&snap, selector) else {
        return false;
    };
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx
        .js_worker
        .execute_script_direct(&script_dispatch_dom_event(&form_sel, "submit", None));
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a checkbox：click `<input type=checkbox>` → 翻转 `checked` 属性（boolean：存在→`RemoveAttr`，
/// 不存在→`SetAttr` 空值）+ 派发 'change' 事件。change listener 经 `el.checked` 读翻转后状态。
/// 返回 true（checked 翻转总改 DOM，调用方 rerender）。
pub fn apply_toggle_checkbox(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    let snap = ctx.html.clone();
    if !is_checkbox(&snap, selector) {
        return false;
    }
    let mutation = if has_attribute(&snap, selector, "checked") {
        DomMutation::RemoveAttr {
            selector: selector.into(),
            name: "checked".into(),
        }
    } else {
        DomMutation::SetAttr {
            selector: selector.into(),
            name: "checked".into(),
            value: String::new(),
        }
    };
    if let Ok(new_html) = apply_mutations_to_html(&snap, std::slice::from_ref(&mutation)) {
        *ctx.html = new_html;
    }
    // 派发 'change'（dom_html 已含翻转后 checked，listener 经 el.checked / hasAttribute 读到新状态）。
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx
        .js_worker
        .execute_script_direct(&script_dispatch_dom_event(selector, "change", None));
    let html_snap = ctx.html.clone();
    let _ = apply_recorded_mutations(ctx, &html_snap);
    true
}

/// P1a radio：click `<input type=radio>` → set `checked` on it + `toggle_radio_html` 解析同 name
/// 组兄弟 unset → 派发 'change' 事件。返回 true（radio toggle 总改 DOM，调用方 rerender）。
pub fn apply_toggle_radio(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    let snap = ctx.html.clone();
    if !is_radio(&snap, selector) {
        return false;
    }
    if let Some(new_html) = toggle_radio_html(&snap, selector) {
        *ctx.html = new_html;
    }
    // 派发 'change'（dom_html 已含 target checked + 同组兄弟 unset）。
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx
        .js_worker
        .execute_script_direct(&script_dispatch_dom_event(selector, "change", None));
    let html_snap = ctx.html.clone();
    let _ = apply_recorded_mutations(ctx, &html_snap);
    true
}

fn execute_chunk<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    html: &str,
    is_module: bool,
    module_url: &str,
    code: &str,
    fetch_text: &F,
) -> Result<(), String> {
    ctx.js_worker.set_dom_snapshot(html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    if is_module {
        let mut registry: HashMap<String, String> = HashMap::new();
        collect_module_deps(fetch_text, module_url, code, &mut registry)?;
        let deps: Vec<(String, String)> = registry.into_iter().collect();
        ctx.js_worker.execute_module(code, module_url, &deps)?;
    } else {
        // classic 页面脚本：顶层 try-catch 包装捕获抛错（防持久 Isolate 中毒 + 让 R2940 报告生效）。
        run_page_script_caught(ctx.js_worker, code)?;
    }
    Ok(())
}

/// 执行 classic 页面 `<script>` 体，顶层 try-catch 包装未捕获 throw（[`script_wrap_page_caught`]）。
/// 成功 → `Ok(())`；抛错 → sentinel 读出消息 → `Err(msg)`（调用方 `run_page_scripts` 据此报 window.onerror）。
/// 包装器 execute 不会抛（try-catch 兜底），随后的 sentinel 读取 execute 在干净 Isolate 上可靠。
fn run_page_script_caught(js_worker: &RendererJsWorker, code: &str) -> Result<(), String> {
    let _ = js_worker.execute_script_direct(&script_wrap_page_caught(code));
    match js_worker.execute_script_direct(&page_script_error_check()) {
        Ok(v) if v.is_empty() => Ok(()),
        Ok(msg) => Err(msg),
        Err(e) => Err(e),
    }
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
    match apply_mutations_to_html_with_handles(html, &recorded) {
        Ok((new_html, handle_selectors)) => {
            // P1a gBCR path A：merge handle→唯一选择器映射进 worker 持久 map，供 RectBridge
            // handler 解析 handle-identity（createElement 元素）。upsert——同 handle 后续 id/class
            // 变更会更新（同 batch 内）；导航时 worker 清空。空 map（无 createElement）no-op。
            if !handle_selectors.is_empty()
                && let Ok(mut map) = ctx.js_worker.handle_selector_map().lock()
            {
                map.extend(handle_selectors);
            }
            *ctx.html = new_html.clone();
            Some(new_html)
        }
        Err(e) => {
            warn!("apply DOM mutations: {e}");
            None
        }
    }
}

/// 渲染进程是否允许直连网络（仅测试；生产路径应经 Browser 进程 `FetchRequest`）。
pub fn should_skip_scripts(url: &str) -> bool {
    url.starts_with("view-source:")
}

#[cfg(test)]
mod tests {
    //! R2940–R2944 renderer mirror 驱动测试——验证默认多进程路径（renderer `page_scripts`）与 browser
    //! `tab_scripts` 的事件 API parity。经 `RendererJsWorker`（装同款 `js_dom_shim`）直接驱动
    //! `run_page_scripts` + `finish_page_load`，轮询 `globalThis.__*` 断言事件派发。
    use super::*;
    use crate::js_worker::RendererJsWorker;

    /// 轮询 `globalThis.{key}` 直到非 undefined（或超时返当前值）。镜像 `js_worker::tests` 模式——
    /// 事件派发经 `execute_script_direct` 同步执行，listener 在调用内触发；超时兜底防 flaky。
    fn wait_for_global(worker: &RendererJsWorker, key: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe)
                && v != "undefined"
            {
                return v;
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    /// 在 worker 上跑 `run_page_scripts`（inline 脚本经 execute 执行；external fetch 恒失败）。
    /// 用独立 `html` buffer——脚本副作用（listener 注册 / 抛错）发生在 worker 持久 V8 上下文，buffer
    /// 仅承载 mutation apply（测试不断言 DOM 变更）。
    fn run_scripts(html: &str, worker: &RendererJsWorker) {
        let mut buf = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut buf,
            url: "https://example.com/page",
            js_worker: worker,
        };
        let _ = run_page_scripts(&mut ctx, true, |_u| Err::<String, String>("no external fetch".into()));
    }

    /// R2941 mirror：finish_page_load 派发 DOMContentLoaded + load。inline 脚本注册 window listener，
    /// run_page_scripts 执行注册，finish_page_load 派发——listener 触发（analytics onload / jQuery ready）。
    #[test]
    fn finish_page_load_dispatches_lifecycle_r2941() {
        let mut worker = RendererJsWorker::spawn(110);
        let html = "<html><body>\
            <script>\
              window.addEventListener('DOMContentLoaded', function(){ globalThis.__dcl = 'fired'; });\
              window.addEventListener('load', function(){ globalThis.__load = 'fired'; });\
            </script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(&worker, Vec::new(), Vec::new(), Vec::new(), Vec::new());
        assert_eq!(
            wait_for_global(&worker, "__dcl", 1000),
            "fired",
            "DOMContentLoaded 派发"
        );
        assert_eq!(wait_for_global(&worker, "__load", 1000), "fired", "load 派发");
        worker.shutdown();
    }

    /// R2941 mirror：无 `<script>` 页仍派发 lifecycle。调用方在 `run_page_scripts`（无脚本 → no-op）之后
    /// 无条件调用 `finish_page_load`——使扩展/polyfill 预注册的 window load listener 触发（镜像 browser
    /// `PageScriptRunner::start` 对无脚本 JS 启用页仍返回 runner 让 finish() 派 lifecycle 的语义）。
    #[test]
    fn finish_page_load_lifecycle_for_scriptless_page_r2941() {
        let mut worker = RendererJsWorker::spawn(111);
        // 无 `<script>` 页面；listener 由「预装 polyfill」直接注册（不经 run_page_scripts）。
        let html = "<html><body><p>no scripts</p></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        let _ = worker
            .execute_script_direct("window.addEventListener('load', function(){ globalThis.__load = 'fired'; });");
        // run_page_scripts 无脚本直接返回 false（no-op），finish_page_load 仍派 load。
        run_scripts(html, &worker);
        finish_page_load(&worker, Vec::new(), Vec::new(), Vec::new(), Vec::new());
        assert_eq!(
            wait_for_global(&worker, "__load", 1000),
            "fired",
            "无脚本页 finish_page_load 仍派发 load（lifecycle 不依赖 <script> 存在）"
        );
        worker.shutdown();
    }

    /// R2946 mirror：`<body onload="...">` 内联 handler 经 body→window 反射为 window.onload，
    /// finish_page_load 派 load 时触发（此前 body onload 在两路径均不触发——R2945 测试时发现的缺口）。
    /// 无 `<script>` 页面，反射由 finish_page_load 内 dispatch_page_lifecycle 前置的 __zw_reflect_body_handlers 触发。
    #[test]
    fn finish_page_load_fires_body_onload_r2946() {
        let mut worker = RendererJsWorker::spawn(116);
        let html = "<html><body onload=\"globalThis.__bodyload='fired'\"></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        // 无 <script>：run_page_scripts no-op，finish_page_load 反射 body onload + 派 load 触发。
        run_scripts(html, &worker);
        finish_page_load(&worker, Vec::new(), Vec::new(), Vec::new(), Vec::new());
        assert_eq!(
            wait_for_global(&worker, "__bodyload", 1000),
            "fired",
            "<body onload> 经反射为 window.onload，finish_page_load 派 load 触发"
        );
        worker.shutdown();
    }

    /// R2946 mirror：有 `<script>` 页面，body onload 反射在首个脚本执行前（__zw_begin_script）发生，
    /// 随后脚本可读 window.onload（=反射的 body handler）——验证反射时序对脚本可见。
    #[test]
    fn body_onload_reflected_before_first_script_r2946() {
        let mut worker = RendererJsWorker::spawn(117);
        let html = "<html><body onload=\"globalThis.__bodyload='fired'\">\
                    <script>if (typeof window.onload === 'function') { window.onload({}); }</script>\
                    </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        // run_page_scripts 抽 <script> 执行；execute_chunk 的 __zw_begin_script 前置反射 body onload → window.onload，
        // 随后脚本读 window.onload（function）并调用 → __bodyload 触发（证明反射早于脚本、对脚本可见）。
        run_scripts(html, &worker);
        assert_eq!(
            wait_for_global(&worker, "__bodyload", 1000),
            "fired",
            "body onload 反射早于首脚本执行，脚本可读 window.onload 并调用"
        );
        worker.shutdown();
    }

    /// R2940 mirror：第二个 inline 脚本抛错 → execute_chunk Err → report_uncaught_error → window.onerror
    /// 触发（Sentry/analytics hook）。第一个脚本先注册 window.onerror。
    #[test]
    fn run_page_scripts_reports_uncaught_error_r2940() {
        let mut worker = RendererJsWorker::spawn(112);
        let html = "<html><body>\
            <script>window.onerror = function(msg){ globalThis.__err = String(msg); return true; };</script>\
            <script>throw new Error('boom-renderer');</script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        let err = wait_for_global(&worker, "__err", 1000);
        assert!(
            err.contains("boom-renderer"),
            "window.onerror 应收到抛错信息，got: {err}"
        );
        worker.shutdown();
    }

    /// R2942 mirror：finish_page_load 派发 stylesheet fetch 失败的 window 'error'（经 __zw_report_error hook
    /// → window.onerror legacy 5-arg）。模拟 host 从 AsyncPageLoad.take_failed_resources drain 注入。
    #[test]
    fn finish_page_load_dispatches_resource_window_error_r2942() {
        let mut worker = RendererJsWorker::spawn(113);
        let html = "<html><body>\
            <script>window.onerror = function(msg, src){ globalThis.__rerr = String(msg) + '|' + String(src); return true; };</script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(
            &worker,
            vec![("stylesheet".to_string(), "https://example.com/missing.css".to_string())],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let got = wait_for_global(&worker, "__rerr", 1000);
        assert!(got.contains("stylesheet"), "window 'error' 含资源 kind，got: {got}");
        assert!(got.contains("missing.css"), "window 'error' 含资源 url，got: {got}");
        worker.shutdown();
    }

    /// R2943 mirror：finish_page_load 派发 img 元素级 load——经 __zw_dispatch_img_event 按 src 绝对 URL
    /// 匹配 `<img>` 元素 proxy 派发（img.onload/addEventListener('load') 触发）。
    #[test]
    fn finish_page_load_dispatches_img_event_r2943() {
        let mut worker = RendererJsWorker::spawn(114);
        let html = "<html><body>\
            <img id='i1' src='https://example.com/a.png'>\
            <script>\
              var img = document.querySelectorAll('img')[0];\
              img.addEventListener('load', function(){ globalThis.__imgload = 'fired'; });\
              img.addEventListener('error', function(){ globalThis.__imgerr = 'fired'; });\
            </script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(
            &worker,
            Vec::new(),
            vec![("https://example.com/a.png".to_string(), "load")],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            wait_for_global(&worker, "__imgload", 1000),
            "fired",
            "img load 元素级事件派发"
        );
        worker.shutdown();
    }

    /// R2944 mirror：finish_page_load 派发 stylesheet (`<link>`) 元素级 load——经 __zw_dispatch_link_event
    /// 按 href 绝对 URL 匹配 `<link>` 元素 proxy 派发（link.onload 触发）。
    #[test]
    fn finish_page_load_dispatches_link_event_r2944() {
        let mut worker = RendererJsWorker::spawn(115);
        let html = "<html><body>\
            <link rel='stylesheet' href='https://example.com/s.css'>\
            <script>\
              var link = document.querySelectorAll('link')[0];\
              link.addEventListener('load', function(){ globalThis.__linkload = 'fired'; });\
            </script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(
            &worker,
            Vec::new(),
            Vec::new(),
            vec![("https://example.com/s.css".to_string(), "load")],
            Vec::new(),
        );
        assert_eq!(
            wait_for_global(&worker, "__linkload", 1000),
            "fired",
            "link load 元素级事件派发"
        );
        worker.shutdown();
    }

    /// R2947 mirror：`document.fonts.ready` Promise 在 finish_page_load 后解析（字体加载库 / FOUT 处理高频 hook）。
    /// 页面注册 `document.fonts.ready.then(...)`，finish_page_load 经 `__zw_font_settle` 解析 ready。
    #[test]
    fn finish_page_load_resolves_fonts_ready_r2947() {
        let mut worker = RendererJsWorker::spawn(118);
        let html = "<html><body><script>\
                    document.fonts.ready.then(function(){ globalThis.__fontsready = 'resolved'; });\
                    </script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        // 无 @font-face（font_events 空）→ settle 仍 resolve ready（字体集从不 loading）。
        finish_page_load(&worker, Vec::new(), Vec::new(), Vec::new(), Vec::new());
        assert_eq!(
            wait_for_global(&worker, "__fontsready", 1000),
            "resolved",
            "document.fonts.ready 在 finish_page_load 后解析"
        );
        worker.shutdown();
    }

    /// R2947 mirror：有 @font-face 加载成功 → FontFaceSet 'loadingdone' 事件派发（含 addEventListener + IDL handler）。
    #[test]
    fn finish_page_load_dispatches_loadingdone_r2947() {
        let mut worker = RendererJsWorker::spawn(119);
        let html = "<html><body><script>\
                    document.fonts.addEventListener('loadingdone', function(){ globalThis.__loadingdone='fired'; });\
                    document.fonts.onloadingdone = function(){ globalThis.__idl='fired'; };\
                    </script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        // 一个 @font-face 加载成功（had_loaded=true）→ 派 loadingdone。
        finish_page_load(
            &worker,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![("MyFont".to_string(), "loaded")],
        );
        assert_eq!(
            wait_for_global(&worker, "__loadingdone", 1000),
            "fired",
            "FontFaceSet loadingdone 事件派发（addEventListener）"
        );
        assert_eq!(
            wait_for_global(&worker, "__idl", 1000),
            "fired",
            "FontFaceSet onloadingdone IDL handler 触发"
        );
        worker.shutdown();
    }

    /// R2947 mirror：@font-face 加载失败 → FontFaceSet 'loadingerror' 事件派发。
    #[test]
    fn finish_page_load_dispatches_loadingerror_r2947() {
        let mut worker = RendererJsWorker::spawn(120);
        let html = "<html><body><script>\
                    document.fonts.addEventListener('loadingerror', function(){ globalThis.__loadingerr='fired'; });\
                    </script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        // 一个 @font-face 加载失败（had_error=true）→ 派 loadingerror。
        finish_page_load(
            &worker,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![("BadFont".to_string(), "error")],
        );
        assert_eq!(
            wait_for_global(&worker, "__loadingerr", 1000),
            "fired",
            "FontFaceSet loadingerror 事件派发（@font-face 加载失败）"
        );
        worker.shutdown();
    }

    /// R2950 mirror：finish_page_load 把 font_events 反映为 FontFace 对象加入 document.fonts（补全 set 语义）。
    /// 经 finish_page_load 传 font_events，验证 document.fonts.size/values 反映 @font-face 字体。
    #[test]
    fn finish_page_load_reflects_fontface_r2950() {
        let mut worker = RendererJsWorker::spawn(121);
        let html = "<html><body><script>\
                    globalThis.__probe = function(){ return document.fonts.size; };\
                    </script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        // 初始 document.fonts 空（无程序化 add）。
        assert_eq!(
            worker.execute_script_direct("String(document.fonts.size)").unwrap(),
            "0",
            "初始 document.fonts 空"
        );
        // finish_page_load 传 2 个 @font-face 加载结果 → 反映为 FontFace 加入 set。
        finish_page_load(
            &worker,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![("MyFont".to_string(), "loaded"), ("BadFont".to_string(), "error")],
        );
        assert_eq!(
            worker.execute_script_direct("String(document.fonts.size)").unwrap(),
            "2",
            "finish_page_load 反映 2 个 @font-face 字体 → document.fonts.size=2"
        );
        // 收集 family 验证。
        let families = worker
            .execute_script_direct(
                "globalThis.__f=[];document.fonts.forEach(function(f){globalThis.__f.push(f.family);});\
                 String(globalThis.__f.sort().join(','))",
            )
            .unwrap();
        assert_eq!(families, "BadFont,MyFont", "document.fonts 迭代得反映的 family");
        worker.shutdown();
    }

    /// R2952：setTimeout(fn, 0) FIFO 顺序——多个 0-delay 定时器按注册序触发（修此前 per-timer
    /// 子线程竞态致顺序不确定）。单协调线程 + (expiry, seq) min-heap 保证。
    #[test]
    fn settimeout_zero_delay_fifo_order_r2952() {
        let mut worker = RendererJsWorker::spawn(122);
        worker.set_dom_snapshot("<html><body></body></html>", "https://example.com/page");
        // 注册 20 个 setTimeout(fn, 0)，各 push 自己的索引。
        worker
            .execute_script_direct(
                "globalThis.__order = [];\
                 for (var i = 0; i < 20; i++) { (function(k){ setTimeout(function(){ globalThis.__order.push(k); }, 0); })(i); }",
            )
            .unwrap();
        // 轮询直到全部 20 个回调触发。
        let probe = "String(globalThis.__order.length)";
        let start = std::time::Instant::now();
        loop {
            if worker.execute_script_direct(probe).unwrap_or_default() == "20" {
                break;
            }
            if start.elapsed().as_millis() >= 2000 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let order = worker
            .execute_script_direct("String(globalThis.__order.join(','))")
            .unwrap();
        assert_eq!(
            order, "0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19",
            "setTimeout(fn, 0) 按注册序 FIFO 触发（单协调线程，非竞态）"
        );
        worker.shutdown();
    }
}
