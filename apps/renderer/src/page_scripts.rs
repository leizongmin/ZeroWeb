//! 渲染进程页面脚本执行 — 加载完成后运行 `<script>` 并处理 DOM 事件。

use std::collections::HashMap;

use tracing::warn;
use zero_engine::{
    DomEventDetail, DomMutation, PageScript, apply_mutations_to_html, apply_mutations_to_html_with_handles,
    enclosing_form_selector, extract_page_scripts, has_attribute, is_checkbox, is_radio, is_submit_button,
    query_tag_from_html, resolve_document_url, script_dispatch_dom_event, script_text_delete, script_text_input,
    toggle_radio_html,
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
/// 待 fire 回调在此派发（OFF 时 shim 早返零开销）。ts 暂传 0（真实 DOMHighResTimeStamp 接入为
/// follow-up，见 p1a-event-loop-raf-slice-design §3.3）。observer tick 先于 rAF，rAF 回调见到的
/// DOM 反映 observer 本帧变更；两者 mutation 合并由 `apply_recorded_mutations` 单次 rerender。
pub fn tick_observers(ctx: &mut PageScriptContext<'_>) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    let _ = ctx.js_worker.execute_script_direct(
        "if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();\
         if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(0);",
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
