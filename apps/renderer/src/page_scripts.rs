//! 渲染进程页面脚本执行 — 加载完成后运行 `<script>` 并处理 DOM 事件。

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Receiver;

use tracing::warn;
use zero_engine::{
    DomEventDetail, DomMutation, PageScript, anchor_hash_target, anchor_javascript_target,
    apply_mutations_to_html_with_handles, extract_page_scripts_indexed, page_script_error_check, resolve_document_url,
    script_call_set_location_hash, script_commit_resource_element_state, script_dispatch_dom_event,
    script_dispatch_link_event, script_dispatch_script_event, script_host_focus, script_report_error,
    script_run_classic_page,
};
#[cfg(test)]
use zero_engine::{
    enclosing_form_selector, is_reset_button, is_submit_button, script_call_form_reset, script_reset_form_controls,
    script_set_control_checked, script_text_control_snapshot, script_text_delete, script_text_delete_without_event,
    script_text_input, script_text_input_without_event,
};
use zero_page_runtime::JsExecutor as _;
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_webview::ResourceElementEvent;
#[cfg(test)]
use zero_webview::ResourceElementOutcome;

use crate::js_worker::{RendererJsWorker, collect_module_deps};

/// DOM 事件派发结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomDispatchResult {
    /// `preventDefault()` 未被调用。
    pub default_allowed: bool,
    /// 页面 HTML 因脚本变更已更新并重渲染。
    pub html_changed: bool,
}

/// P1a form submit 结果（R3054）：submit 事件派发后的两项判定。
/// `html_changed` → 调用方 rerender；`default_allowed` → 未 preventDefault → 调用方据 method=GET 导航。
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubmitOutcome {
    /// submit listener 改了 DOM（调用方单次 rerender）。
    pub html_changed: bool,
    /// submit 事件未被 `preventDefault()`（→ GET 表单应导航）。无 enclosing form / 派发失败 → false。
    pub default_allowed: bool,
}

/// 一次文本编辑后从 JS retained 状态读取的最终快照。
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextControlSnapshot {
    /// 事件监听器执行完毕后的当前值。
    pub value: String,
    /// DOM UTF-16 选区起点。
    pub selection_start: usize,
    /// DOM UTF-16 选区终点。
    pub selection_end: usize,
}

/// 宿主文本编辑结果。
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditOutcome {
    /// DOM mutation 是否改变页面快照。
    pub html_changed: bool,
    /// 编辑完成后的 retained 控件快照。
    pub snapshot: Option<TextControlSnapshot>,
}

/// 页面脚本执行上下文。
pub struct PageScriptContext<'a> {
    /// 当前 HTML 文档（脚本执行后同步更新）。
    pub html: &'a mut String,
    /// 页面 URL。
    pub url: &'a str,
    /// JS worker。
    pub js_worker: &'a RendererJsWorker,
    /// WebView（M3-S9 活 DOM 路径）：Some 时 DOM 变更直接应用活 DOM（免 HTML 往返），
    /// None 时回退 HTML 回写（测试/无 webview 场景）。
    pub webview: Option<&'a mut zero_webview::WebView>,
}

/// 脚本阶段让路检查的入站面（P-B1 导航让路）。
///
/// `deferred` 是主循环的 staging 队列（`deferred_inbound`，非 fetch 消息暂存处）；
/// 让路检查扫描它与 `inbound_rx`，遇文档替换型命令（导航/停止）即中止剩余脚本，
/// 消息由调用方回灌队首优先派发，其余消息原序暂存回 `deferred`。
pub struct ScriptPhaseYield<'a> {
    /// 渲染进程入站消息通道。
    pub inbound_rx: &'a Receiver<IpcMessage>,
    /// 主循环 staging 队列（先于 `inbound_rx` 到达次序）。
    pub deferred: &'a mut VecDeque<IpcMessage>,
}

/// 脚本阶段结果。
pub struct ScriptPhaseResult {
    /// 页面 HTML 因脚本变更已更新（语义同旧 `run_page_scripts` 返回值）。
    pub changed: bool,
    /// 让路时截获的文档替换型命令（剩余脚本被丢弃，调用方回灌队首优先派发）。
    pub aborted: Option<IpcMessage>,
}

/// 按文档顺序执行页面脚本（不可让路版——测试/既有调用方语义不变；生产路径一律
/// 走 [`run_page_scripts_interruptible`]）。
#[cfg(test)]
pub fn run_page_scripts<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    fetch_text: F,
) -> bool {
    run_page_scripts_interruptible(ctx, javascript_enabled, fetch_text, None).changed
}

/// 按文档顺序执行页面脚本；`yield_to_commands` 提供时在脚本间检查入站导航命令
/// 并让路中止（P-B1：同步脚本阶段可长达数秒——V8 执行 + 模块依赖同步取——
/// 命令消息不得排队等整个阶段，2026-10-02 bilibili 二跳导航 15s 超时根因）。
pub fn run_page_scripts_interruptible<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    fetch_text: F,
    mut yield_to_commands: Option<ScriptPhaseYield<'_>>,
) -> ScriptPhaseResult {
    if !javascript_enabled || ctx.html.is_empty() || should_skip_scripts(ctx.url) {
        return ScriptPhaseResult {
            changed: false,
            aborted: None,
        };
    }
    let base = ctx.url.to_string();
    let original_html = ctx.html.clone();
    let mut html = ctx.html.clone();

    let scripts = extract_page_scripts_indexed(&html);

    // 脚本批量渲染边界：同步脚本阶段的 DOM 变更在阶段结束时统一渲染一次
    // （HTML Standard event loop「update the rendering」批量语义）。逐脚本
    // 全量渲染 ~1s/次（169KB/数千节点页面 × 8 次）占死 renderer 主循环 9s+
    // 是 2026-10-02 bilibili 二跳导航 15s 超时的根因。
    if let Some(wv) = ctx.webview.as_deref_mut() {
        wv.begin_script_batch();
    }

    for (script, script_index) in scripts {
        // 脚本间让路检查：遇文档替换型命令即中止剩余脚本。批量边界一并关闭
        // 但不渲染（旧文档即将被导航替换，渲染是纯浪费）；已执行脚本的产出
        // 随文档换代作废——与导航打断图片加载阶段的既有语义一致。
        if let Some(nav) = poll_script_abort(yield_to_commands.as_mut()) {
            if let Some(wv) = ctx.webview.as_deref_mut() {
                wv.abort_script_batch();
            }
            return ScriptPhaseResult {
                changed: false,
                aborted: Some(nav),
            };
        }

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
                        report_resource_error(ctx.js_worker, "script", &abs, false);
                        // R2944 mirror：外部脚本元素 'error'（spec：script 元素 error 仅 fetch 失败触发）。
                        dispatch_script_event(ctx.js_worker, &abs, "error");
                        continue;
                    }
                }
            }
        };

        if let Err(e) = execute_chunk(
            ctx,
            &html,
            is_module,
            &module_url,
            &code,
            &fetch_text,
            script_index,
            external_abs.as_deref(),
        ) {
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

    // 批量边界统一渲染（begin_script_batch 的配对出口；best-effort，失败仅 warn）。
    if let Some(wv) = ctx.webview.as_deref_mut()
        && let Err(e) = wv.end_script_batch()
    {
        warn!("end script batch render: {e}");
    }

    if html != original_html {
        *ctx.html = html;
        return ScriptPhaseResult {
            changed: true,
            aborted: None,
        };
    }
    ScriptPhaseResult {
        changed: false,
        aborted: None,
    }
}

/// 单次让路检查：扫 staging 队列与 inbound，取第一个文档替换型命令。
///
/// 其余消息（含命令前到达者）原序暂存回 `deferred`；截获的命令由调用方回灌队首，
/// 跳到这些消息之前优先派发（导航先行的让路语义）。
fn poll_script_abort(yield_to_commands: Option<&mut ScriptPhaseYield<'_>>) -> Option<IpcMessage> {
    let y = yield_to_commands?;
    let mut nav = None;
    let mut kept = VecDeque::new();
    while let Some(msg) = y.deferred.pop_front() {
        if nav.is_none() && is_navigation_command(&msg) {
            nav = Some(msg);
        } else {
            kept.push_back(msg);
        }
    }
    if nav.is_none() {
        while let Ok(msg) = y.inbound_rx.try_recv() {
            if is_navigation_command(&msg) {
                nav = Some(msg);
                break;
            }
            kept.push_back(msg);
        }
    }
    y.deferred.extend(kept);
    nav
}

/// 让路判定：仅文档替换型命令中断脚本阶段（导航 / 停止加载 / 设置文档内容）；
/// 其余命令（ExecuteScript 等查询执行）保持既有「阶段结束再处理」次序。
fn is_navigation_command(msg: &IpcMessage) -> bool {
    matches!(
        msg.kind,
        IpcMessageKind::Navigate(_) | IpcMessageKind::StopLoading | IpcMessageKind::LoadHtml(_)
    )
}

/// R2940–R2944 mirror：页面脚本阶段收尾——派发页面生命周期 + 子资源/元素级事件进 shim，与 browser
/// `tab_scripts::PageScriptRunner::finish` 对齐（renderer 默认多进程路径此前缺这套派发）。
///
/// - **R2941**：DOMContentLoaded + load（DOMContentLoaded 先于 load，spec）。analytics onload / jQuery
///   ready / 框架 mount 高频 hook 经此触发。即使页面无 `<script>`（仅 `<body onload>` 内联 handler），
///   JS 启用页也应派发——调用方在 `run_page_scripts` 之后无条件调用本函数（gate 由调用方按 JS 启用判断）。
/// - **R2942**：子资源 fetch/decode 失败在页面脚本注册 handler 后、window load 前派发 window error。
/// - **FR-009/R2944**：资源状态与 link load/error 提交给匹配元素。
/// - **R2947**：`font_events` = `(family, "loaded"/"error")` @font-face 加载结果——经 `__zw_font_settle`
///   派发 FontFaceSet 'loadingdone'/'loadingerror' + 解析 `document.fonts.ready` Promise。
///
/// 全部 best-effort（失败仅 `warn!`，不影响后续）。事件由调用方从 `AsyncPageLoad` drain 后传入
///（renderer main 在 load 完成时 drain、stash，脚本阶段消费）。
pub fn finish_page_load(
    js_worker: &RendererJsWorker,
    resource_errors: Vec<(String, String)>,
    resource_events: Vec<ResourceElementEvent>,
    link_events: Vec<(String, &'static str)>,
    font_events: Vec<(String, &'static str)>,
) {
    dispatch_page_lifecycle(js_worker, "DOMContentLoaded");
    // R2942：页面脚本注册 handler 后、window load 前派发资源 window 'error'。
    for (kind, url) in &resource_errors {
        // t2-pb1 fix#5：load 前的导航里程碑派发走优先队列（脚本阶段的同类报告走普通队列）。
        report_resource_error(js_worker, kind, url, true);
    }
    // FR-009：提交 img/media/source/track 状态并派发其规范事件。
    for event in &resource_events {
        dispatch_resource_element_event(js_worker, event);
    }
    // R2944：stylesheet 元素级 load/error，位于 DOMContentLoaded 与 window load 之间。
    for (url, ty) in &link_events {
        dispatch_link_event(js_worker, url, ty);
    }
    // R2947：@font-face 加载 settle——派 FontFaceSet 'loadingdone'/'loadingerror' + 解析 document.fonts.ready。
    // 无 @font-face 页面（font_events 空）仍 settle（仅 resolve ready，不派事件）。
    // R2950：先把每个 @font-face 字体反映为 FontFace 对象加入 document.fonts（补全 set 语义），再 settle。
    // t2-pb1 fix#6：批处理为单次执行——真实站点（bilibili 图标字体族）可达 216 个 @font-face，
    // 逐条 round-trip（每条 ~60ms 队列/往返开销）在 DCL 后追加 13s，是导航 15s 超时的第二级
    // 根因。try/catch 保每条隔离（与逐条 best-effort 等价：单条失败不阻断其余）。
    if !font_events.is_empty() {
        let batch: String = font_events
            .iter()
            .map(|(family, status)| {
                format!(
                    "try{{{}}}catch(_e){{}}",
                    zero_engine::script_add_fontface(family, status)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        // t2-pb1 fix#12：fire-and-forget——批处理与随后的 settle/load 同在优先 FIFO，
        // 提交顺序即执行顺序，主循环不等当前臂跑完。
        if let Err(e) = js_worker.submit_script_priority(&batch) {
            warn!("dispatch add fontface batch ({}): {e}", font_events.len());
        }
    }
    let had_loaded = font_events.iter().any(|(_, t)| *t == "loaded");
    let had_error = font_events.iter().any(|(_, t)| *t == "error");
    dispatch_font_settle(js_worker, had_loaded, had_error);
    dispatch_page_lifecycle(js_worker, "load");
}

/// 派发一个页面生命周期事件。资源 settle 事件位于 DOMContentLoaded 与 window load 之间。
fn dispatch_page_lifecycle(js_worker: &RendererJsWorker, event: &str) {
    let reflect = zero_engine::script_reflect_body_handlers();
    let dispatch = script_dispatch_dom_event("html", event, None);
    // t2-pb1 fix#5/#12：生命周期派发走优先队列且不等待结果——页面回调单臂可达 28-30s
    // （bilibili 实测），同步往返会让主循环停在当前臂后（Navigate IPC 饿死）。
    if let Err(e) = js_worker.submit_script_priority(&format!("{reflect} {dispatch}")) {
        warn!("dispatch page lifecycle {event}: {e}");
    }
}

/// R3248（§transitionend）+ R3252（§transitionrun/§transitionstart）：派发过渡事件进 shim。`events` =
/// [`zero_engine::TransitionEvent`] 列表（由 pipeline `take_pending_transition_events` 产出，`kind` 区分
/// Run/Start/End）。每个经 `script_dispatch_transition_event` 构造
/// `new TransitionEvent(kind.as_event_type(), {propertyName, elapsedTime, bubbles})` 派发到唯一目标元素。
/// best-effort（stale 选择器 / 构造器缺失 → 容错跳过）。UI 编排回调（fade-out 后删元素）依赖。
pub fn dispatch_transition_events(js_worker: &RendererJsWorker, events: &[zero_engine::TransitionEvent]) {
    for ev in events {
        let ty = ev.kind.as_event_type();
        let script = zero_engine::script_dispatch_transition_event(&ev.selector, ty, &ev.property, ev.elapsed);
        // t2-pb1 fix#12：best-effort 派发 fire-and-forget（主循环不等 worker 回合）。
        if let Err(e) = js_worker.submit_script_priority(&script) {
            warn!("dispatch {ty} ({}): {e}", ev.selector);
        }
    }
}

/// R3249（CSS Animations §animationend）+ R3250（§animationiteration）+ R3251（§animationstart）：派发动画
/// 事件进 shim。`events` = [`zero_engine::AnimationEvent`] 列表（由 pipeline `take_pending_animation_events`
/// 产出，`kind` 区分 Start/End/Iteration）。每个经 `script_dispatch_animation_event` 构造
/// `new AnimationEvent(kind.as_event_type(), {animationName, elapsedTime, bubbles})` 派发到唯一目标元素。
/// best-effort（stale 选择器 / 构造器缺失 → 容错跳过）。infinite 动画循环回调靠 Iteration（永不 End）。
pub fn dispatch_animation_events(js_worker: &RendererJsWorker, events: &[zero_engine::AnimationEvent]) {
    for ev in events {
        let ty = ev.kind.as_event_type();
        let script = zero_engine::script_dispatch_animation_event(&ev.selector, ty, &ev.name, ev.elapsed);
        // t2-pb1 fix#12：best-effort 派发 fire-and-forget（主循环不等 worker 回合）。
        if let Err(e) = js_worker.submit_script_priority(&script) {
            warn!("dispatch {ty} ({}): {e}", ev.selector);
        }
    }
}

/// R2940 mirror：未捕获脚本错误经 worker 报告进 shim——`window.onerror`（legacy 5-arg）+ window 'error' 事件，
/// 使 Sentry / analytics / GA 等错误上报库 hook 触发。best-effort。
fn report_uncaught_error(js_worker: &RendererJsWorker, source: &str, message: &str) {
    // R-baidu2/P3：推入脚本错误队列（runtime drain 后经 IPC `ScriptError`
    // → headless `Runtime.exceptionThrown`）。classic 页面脚本被 shim 顶层
    // try-catch 包裹（防 Isolate 中毒），错误经 sentinel 读出后在此变 Err——
    // js_worker Execute 汇点的钩子看不到这类错误，故必须在此推入。
    js_worker.push_script_error(zero_protocol::message::ScriptErrorParams {
        text: message.to_string(),
        source: source.to_string(),
        line_number: 0,
        column_number: 0,
    });
    let report = script_report_error(message, source, 0, 0);
    // t2-pb1 fix#12：best-effort 报告 fire-and-forget（主循环不等 worker 回合）。
    if let Err(e) = js_worker.submit_script_priority(&report) {
        warn!("report uncaught script error: {e}");
    }
}

/// R2942 mirror：派发子资源 fetch/decode 失败的 window 'error' 事件进 shim（经 `__zw_report_error` hook →
/// window.onerror legacy 5-arg + window 'error' ErrorEvent）。`kind` = "script" / "stylesheet" / "image"。best-effort。
/// `priority`（t2-pb1 fix#5）：finish_page_load 内的派发走优先队列；脚本阶段的调用传 false（普通队列）。
fn report_resource_error(js_worker: &RendererJsWorker, kind: &str, url: &str, priority: bool) {
    let msg = format!("Error loading {kind}: {url}");
    let report = script_report_error(&msg, url, 0, 0);
    // t2-pb1 fix#12：priority 分支 fire-and-forget（finish_page_load 路径，主循环不等
    // 当前臂）；非 priority 分支保持同步（脚本阶段内，需在阶段边界前排空）。
    let result = if priority {
        js_worker.submit_script_priority(&report).map(|_| String::new())
    } else {
        js_worker.execute_script_direct(&report)
    };
    if let Err(e) = result {
        warn!("report resource error ({kind} {url}): {e}");
    }
}

fn dispatch_resource_element_event(js_worker: &RendererJsWorker, event: &ResourceElementEvent) {
    let report = script_commit_resource_element_state(
        event.tag,
        &event.url,
        event.outcome.as_str(),
        event.natural_width,
        event.natural_height,
        event.media_duration_ms,
    );
    // t2-pb1 fix#5/#12：优先队列 + fire-and-forget（同 dispatch_page_lifecycle）。
    if let Err(e) = js_worker.submit_script_priority(&report) {
        warn!("commit resource state ({} {}): {e}", event.tag, event.url);
    }
}

/// R2944 mirror：派发 stylesheet 元素级 load/error 事件进 shim。经 `script_dispatch_link_event` 生成
/// `__zw_dispatch_link_event(url, type)`——shim 按 href 绝对 URL 匹配 `<link>` 元素 proxy 派发。best-effort。
fn dispatch_link_event(js_worker: &RendererJsWorker, url: &str, ty: &str) {
    let report = script_dispatch_link_event(url, ty);
    // t2-pb1 fix#5/#12：优先队列 + fire-and-forget（同 dispatch_page_lifecycle）。
    if let Err(e) = js_worker.submit_script_priority(&report) {
        warn!("dispatch link event ({ty} {url}): {e}");
    }
}

/// R2944 mirror：派发外部 `<script src>` 元素级 load/error 事件进 shim。经 `script_dispatch_script_event`
/// 生成 `__zw_dispatch_script_event(url, type)`——shim 按 src 绝对 URL 匹配 `<script>` 元素 proxy 派发。best-effort。
pub(crate) fn dispatch_script_event(js_worker: &RendererJsWorker, url: &str, ty: &str) {
    let report = script_dispatch_script_event(url, ty);
    // t2-pb1 fix#10/#12：宿主 tick 派发走优先队列且不等待结果——页面回调流饱和时
    // 普通通道往返无界（bilibili 动态脚本逐个完成后的事件派发曾把主循环卡在
    // tick_dynamic_scripts 内分钟级，导航 IPC 饿死），优先通道同步往返也要等当前臂。
    if let Err(e) = js_worker.submit_script_priority(&report) {
        warn!("dispatch script event ({ty} {url}): {e}");
    }
}

/// R2947 mirror：派发 @font-face 加载 settle 进 shim。经 `script_font_settle` 生成 `__zw_font_settle(...)`——
/// shim 派 FontFaceSet 'loadingdone'（had_loaded）/ 'loadingerror'（had_error）+ 解析 `document.fonts.ready`。
/// best-effort。无 @font-face 页面（had_loaded=had_error=false）仅解析 ready（字体集从不 loading）。
fn dispatch_font_settle(js_worker: &RendererJsWorker, had_loaded: bool, had_error: bool) {
    let report = zero_engine::script_font_settle(had_loaded, had_error);
    // t2-pb1 fix#5/#12：优先队列 + fire-and-forget（同 dispatch_page_lifecycle；
    // 与字体批处理同通道 FIFO，提交顺序保持 批处理→settle）。
    if let Err(e) = js_worker.submit_script_priority(&report) {
        warn!("dispatch font settle: {e}");
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
    // t2-pb1 fix#13：用户事件派发走优先通道 + 有界等待（快照与脚本成对同通道 FIFO）。
    // 普通通道同步往返在页面回调洪水下逐个等 28-30s 臂（b1 旅程实测：goto 前的鼠标
    // 四连发让 Navigate IPC 滞留 106s）。超时按「结果未知」降级（默认动作放行、
    // html 视为未变），脚本留队列照常执行。
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let result_str = match ctx
        .js_worker
        .execute_script_priority_bounded(&script, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT)
    {
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

/// 宿主焦点治理派发（slice22 focus governance）：一次执行同时完成「页面可见焦点状态同步
/// （`document.activeElement` 读的 shim `_activeElKey`）+ 该相位焦点事件派发」——`focus=true`
/// 获焦相位（focus+focusin）/ `false` 失焦相位（focusout+blur）。事件流与 [`dispatch_dom_event`]
/// 逐字节同通道（shim `__zw_dispatch_event` 同一 UA 通道）；plumbing（快照安装 → 有界等待 →
/// mutation 应用）同款。
///
/// 规范锚：HTML §6.5.2 focusing steps——焦点迁移先更 focused area 再派焦点事件族
/// <https://html.spec.whatwg.org/multipage/interaction.html#focusing-steps>；focus 是 mousedown
/// 的默认动作（UI Events §5.2.2 <https://w3c.github.io/uievents/#focus-event-focus>）。
pub fn dispatch_host_focus(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    selector: &str,
    focus: bool,
) -> DomDispatchResult {
    if !javascript_enabled || should_skip_scripts(ctx.url) {
        return DomDispatchResult {
            default_allowed: true,
            html_changed: false,
        };
    }
    let script = script_host_focus(selector, focus);
    // t2-pb1 fix#13 同款：用户事件派发走优先通道 + 有界等待（快照与脚本成对同通道 FIFO）。
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let result_str = match ctx
        .js_worker
        .execute_script_priority_bounded(&script, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT)
    {
        Ok(r) => r,
        Err(e) => {
            warn!("dispatch host focus({focus}) on {selector}: {e}");
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
    tick_observers_with(ctx, tick_per_task_enabled())
}

/// M3-S2 per-task 开关（进程级缓存——每帧 tick 不重复读 env）。
fn tick_per_task_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_RENDERER_TICK_PER_TASK").as_deref() == Ok("1"))
}

/// `tick_observers` 的模式注入形态（`per_task` = kill-switch
/// `ZW_RENDERER_TICK_PER_TASK`，测试双模式直设避免进程级 env 竞态）。
pub fn tick_observers_with(ctx: &mut PageScriptContext<'_>, per_task: bool) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    // t8/P16：不 pre-clear——本函数在每次帧发布（publish_webview → tick_observers_inner）
    // 末尾运行，而 mutation 队列与异步 turn（TimerBridge 定时器 resolve、CDP evaluate 等）
    // 共享：上一 turn 已入队尚未 drain 的合法写入（如 html5test.co 完成回调的
    // contents/loading 样式写入）会在下一帧渲染前被这里清空，永滞丢失。renderer 侧
    // apply_recorded_mutations 已是 drain（consume-once），pre-clear 无重放可防，纯销毁。
    // 保留 pending → 下方 observer 执行后的 apply 把「异步 turn 写入 + observer 写入」
    // 一并按序落 host（spec：rendering 更新前不丢弃 pending task 的 DOM 变更）。
    // https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering
    // event-loop-spec M3-S2：per-task 模式——每次 execute 只 schedule 首个活跃 observer
    //（`__zw_observers_tick_once`，其回调在本 execute 末 checkpoint 派发）→ 一 observer
    // 一 task 一 checkpoint（spec event loop processing model step 3-6），消除「IO/RO/rAF
    // 全部回调一个 execute」的批量派发违反（evidence/2026-09-11-m1-event-loop-gap-list.md
    // §1.2）。observer 回调可改 DOM → 每轮 execute 后 apply_recorded_mutations；上限 64
    // 轮防回调内重注册死循环。rAF tick 在 observer 队列排空后单独一 execute（rAF 见到的
    // DOM 反映全部 observer 变更，与合并模式的单次 apply 时序一致）。
    // 默认 OFF 维持现合并 tick（零行为变化）。
    if per_task {
        // 无状态游标协议：`tick_once(cursor)` schedule 首个活跃 observer 并返回下一
        // 游标，-1 = 耗尽（上限 64 轮防回调内重注册死循环）。
        // t2-pb1 fix#7：宿主节拍 execute 走优先队列（主循环同步等 reply，不被页面
        // 续体洪水压住——见 js_worker 分派环注释）。
        let mut cursor: i64 = 0;
        for _ in 0..64 {
            let res = ctx
                .js_worker
                .execute_script_direct_priority(&format!(
                    "(function(){{return String(globalThis.__zw_observers_tick_once({cursor}));}})()"
                ))
                .unwrap_or_else(|_| "-1".to_string());
            let next = res.trim().parse::<i64>().unwrap_or(-1);
            let html_snap = ctx.html.clone();
            apply_recorded_mutations(ctx, &html_snap);
            if next < 0 {
                break;
            }
            cursor = next;
        }
        // t2-pb1 fix#12：raf tick 结果被忽略 → fire-and-forget（主循环不等当前臂；
        // observer 回调产生的 mutation 由下一次 apply 落定，至多滞后一帧）。
        let _ = ctx.js_worker.submit_script_priority(
            "if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(globalThis.performance?performance.now():0);",
        );
        let html_snap = ctx.html.clone();
        return apply_recorded_mutations(ctx, &html_snap).is_some();
    }
    // t2-pb1 fix#7/#12：合并 tick 也是主循环节拍，走优先队列且不等待结果（同上，
    // observer/raf 回调 mutation 由下一次 apply 落定，至多滞后一帧）。
    let _ = ctx.js_worker.submit_script_priority(
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
#[cfg(test)]
pub fn apply_text_input(ctx: &mut PageScriptContext<'_>, selector: &str, key: &str) -> TextEditOutcome {
    apply_text_edit(ctx, selector, &script_text_input(selector, key))
}

/// 执行 JavaScript-disabled 路径的 UA 文本插入，不派发页面 listener。
#[cfg(test)]
pub fn apply_text_input_without_events(ctx: &mut PageScriptContext<'_>, selector: &str, text: &str) -> TextEditOutcome {
    apply_text_edit(ctx, selector, &script_text_input_without_event(selector, text))
}

#[cfg(test)]
fn apply_text_edit(ctx: &mut PageScriptContext<'_>, selector: &str, edit_script: &str) -> TextEditOutcome {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let script = format!("{edit_script};{}", script_text_control_snapshot(selector));
    let snapshot = ctx
        .js_worker
        .execute_script_direct(&script)
        .ok()
        .and_then(|value| serde_json::from_str::<(String, usize, usize)>(&value).ok())
        .map(|(value, selection_start, selection_end)| TextControlSnapshot {
            value,
            selection_start,
            selection_end,
        });
    let html_snap = ctx.html.clone();
    TextEditOutcome {
        html_changed: apply_recorded_mutations(ctx, &html_snap).is_some(),
        snapshot,
    }
}

/// 在一个 JS/mutation 批次内派发 IME composition 事件，避免 start+update 各自重渲染。
pub fn dispatch_composition_events(
    ctx: &mut PageScriptContext<'_>,
    javascript_enabled: bool,
    selector: &str,
    events: &[(&str, &str)],
) -> bool {
    if !javascript_enabled || should_skip_scripts(ctx.url) || events.is_empty() {
        return false;
    }
    let script = events
        .iter()
        .map(|(event_type, data)| {
            script_dispatch_dom_event(
                selector,
                event_type,
                Some(&DomEventDetail {
                    data: Some((*data).to_string()),
                    ..Default::default()
                }),
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    if let Err(error) = ctx.js_worker.execute_script_direct(&script) {
        warn!("dispatch composition events on {selector}: {error}");
        return false;
    }
    let html_snapshot = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snapshot).is_some()
}

/// P1a form input：Backspace 删焦点 input/textarea 的末字符 + 派发 'input' 事件。
/// 镜像 `apply_text_input`。返回 value 属性是否变更（调用方据此单次 rerender）。
#[cfg(test)]
pub fn apply_text_delete(ctx: &mut PageScriptContext<'_>, selector: &str) -> TextEditOutcome {
    apply_text_edit(ctx, selector, &script_text_delete(selector))
}

/// 执行 JavaScript-disabled 路径的 UA Backspace，不派发页面 listener。
#[cfg(test)]
pub fn apply_text_delete_without_events(ctx: &mut PageScriptContext<'_>, selector: &str) -> TextEditOutcome {
    apply_text_edit(ctx, selector, &script_text_delete_without_event(selector))
}

/// P1a form submit：click 命中 submit button（`<input type=submit/image>` / `<button>` type≠button）
/// → 解析 enclosing `<form>` → 派发 'submit' 事件。返回 submit 结果（含 default_allowed 供 GET 导航）。
#[cfg(test)]
pub fn apply_submit_on_click(ctx: &mut PageScriptContext<'_>, selector: &str) -> SubmitOutcome {
    if !is_submit_button(ctx.html, selector) {
        return SubmitOutcome::default();
    }
    // click submit button：submitter = 被点的按钮自身（spec：event.submitter = 激活提交的按钮）。
    submit_enclosing_form(ctx, selector, Some(selector))
}

/// P1a form reset（R3050，闭合 R3048 限制⑤）：click 命中 reset button（`<input type=reset>` / `<button type=reset>`）
/// → 解析 enclosing `<form>` → 调 shim `form.reset()`（dispatch cancelable 'reset' 事件 + 未取消则 revert 控件，
/// 复用 R3048 全部 reset 语义）。返回 reset 回调是否改 DOM。无 enclosing form → false。
#[cfg(test)]
pub fn apply_reset_on_click(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    if !is_reset_button(ctx.html, selector) {
        return false;
    }
    let snap = ctx.html.clone();
    let Some(form_sel) = enclosing_form_selector(&snap, selector) else {
        return false;
    };
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    // 调 shim form.reset()（R3048）：reset 事件派发 + 控件恢复 default 经 proxy setter 记 mutation。
    let _ = ctx.js_worker.execute_script_direct(&script_call_form_reset(&form_sel));
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// 执行 JavaScript-disabled 路径的 UA reset，不派发页面 `reset` listener。
#[cfg(test)]
pub fn apply_reset_on_click_without_events(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    if !is_reset_button(ctx.html, selector) {
        return false;
    }
    let snap = ctx.html.clone();
    let Some(form_sel) = enclosing_form_selector(&snap, selector) else {
        return false;
    };
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let _ = ctx
        .js_worker
        .execute_script_direct(&script_reset_form_controls(&form_sel));
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// 重置已解析的 form owner，不派发页面 `reset` listener。
#[cfg(test)]
pub fn apply_form_reset_without_events(ctx: &mut PageScriptContext<'_>, form_selector: &str) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let _ = ctx
        .js_worker
        .execute_script_direct(&script_reset_form_controls(form_selector));
    let html_snap = ctx.html.clone();
    let changed = apply_recorded_mutations(ctx, &html_snap).is_some();
    if changed {
        ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    }
    changed
}

/// 开始宿主默认动作事务，暂缓 listener 排入的 microtask。
#[cfg(test)]
pub fn begin_host_action_transaction(ctx: &mut PageScriptContext<'_>) {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    let _ = ctx
        .js_worker
        .execute_script_direct("__zw_begin_host_action_transaction()");
}

/// 完成宿主默认动作事务，flush microtask 并应用其 DOM mutations。
#[cfg(test)]
pub fn end_host_action_transaction(ctx: &mut PageScriptContext<'_>) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let _ = ctx
        .js_worker
        .execute_script_direct("__zw_end_host_action_transaction()");
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a 导航（R3053，闭合 R3052 限制③）：click 命中 hash 链接（`<a href="#sec">`）→ 调 shim
/// `location.hash = hash`（R3006：更新 hash + 新 history entry + 异步派发 hashchange + 触 onhashchange）。
/// SPA hash 路由核心交互——hash 链接点击驱动前端路由。返回 hashchange listener 是否改 DOM
/// （hash 本身不改 DOM，但 SPA router listener 可能据 hash 切换视图）。无 hash 目标 → false。
/// headless 无 viewport → 不滚动到锚（real browser 会滚到 `id=sec` 元素），仅 hash/hashchange。
pub fn apply_set_hash_on_click(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    // gate：`<a href="#...">` 才设 hash（mirror apply_reset_on_click 防御性再校验 is_reset_button）。
    let Some(hash) = anchor_hash_target(ctx.html, selector) else {
        return false;
    };
    // t2-pb1 F7（首轮缺陷审查 2026-10-02）：点击默认动作路径改优先通道对（快照+脚本同
    // 通道保序，fix#10 语义）+ 有界挂起（与 execute_automation_script_deferrable 同型）。
    // 原 execute_script_direct 无界同步等待——worker 长臂（bilibili timer 级联 28-30s）
    // 时点击处理阻塞 renderer 主循环、Navigate IPC 饿死（fix#15 同族）。挂起时脚本照常
    // 执行，mutation 由下一入口 drain 落定。
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    // 调 location.hash = hash（R3006 全语义：hash 更新 + history entry + hashchange 派发经 _defer microtask）。
    let _ = ctx.js_worker.execute_script_priority_deferrable(
        &script_call_set_location_hash(&hash),
        zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT,
    );
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// P1a 导航（R3057，闭合 R3052 限制②）：click 命中 `<a href="javascript:...">` → 在页面全局执行其 JS 体
///（real browser 语义：javascript: URL click 执行其体，返回值丢弃——非导航）。与 onclick handler 同一
/// JS 执行通路（worker 脚本命令，**非新增 eval 表面**，CSP `script-src` 统辖内联/eval 拦截）。
/// 返回 JS 体执行是否改 DOM（调用方据此单次 rerender）。无 javascript: 目标 → false。
pub fn apply_javascript_href(ctx: &mut PageScriptContext<'_>, selector: &str) -> bool {
    // gate：`<a href="javascript:...">` 才执行（mirror apply_set_hash_on_click 防御性再校验 anchor_hash_target）。
    let Some(js) = anchor_javascript_target(ctx.html, selector) else {
        return false;
    };
    // t2-pb1 F7：优先通道对 + 有界挂起（同 apply_set_hash_on_click 注）。
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    // 执行 JS 体（空体 no-op）。js 为 href 解析后的原始 JS 源（HTML 已解码实体），不经转义——直接执行。
    if !js.is_empty() {
        let _ = ctx
            .js_worker
            .execute_script_priority_deferrable(&js, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT);
    }
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// 共享 submit 核心：解析 enclosing `<form>` → 派发 'submit'（复用 `script_dispatch_dom_event`）
/// → apply。无触发 gate（调用方先判 Enter-in-input / submit-button）。无 enclosing form → 默认 outcome。
/// 返回 submit 结果（R3054：default_allowed = 未 preventDefault，驱动 GET 导航）。
#[cfg(test)]
fn submit_enclosing_form(ctx: &mut PageScriptContext<'_>, selector: &str, submitter: Option<&str>) -> SubmitOutcome {
    let snap = ctx.html.clone();
    let Some(form_sel) = enclosing_form_selector(&snap, selector) else {
        return SubmitOutcome::default();
    };
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    // R2984：SubmitEvent.submitter——click submit button → 该按钮选择器；Enter 隐式提交 → None。
    let detail = DomEventDetail {
        submitter: submitter.map(String::from),
        ..Default::default()
    };
    // R3054：submit 事件可 cancelable——dispatch 返串 "prevented" 表示 preventDefault 调用（同 dispatch_dom_event）。
    let result_str = ctx
        .js_worker
        .execute_script_direct(&script_dispatch_dom_event(&form_sel, "submit", Some(&detail)))
        .unwrap_or_default();
    let default_allowed = result_str.trim() != "prevented";
    let html_snap = ctx.html.clone();
    let html_changed = apply_recorded_mutations(ctx, &html_snap).is_some();
    SubmitOutcome {
        html_changed,
        default_allowed,
    }
}

/// 设置 checkbox/radio checkedness，不派发页面事件。
#[cfg(test)]
pub fn apply_set_checked_without_events(ctx: &mut PageScriptContext<'_>, selector: &str, checked: bool) -> bool {
    apply_state_script(ctx, &script_set_control_checked(selector, checked))
}

#[cfg(test)]
fn apply_state_script(ctx: &mut PageScriptContext<'_>, script: &str) -> bool {
    ctx.js_worker.set_dom_snapshot(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    if let Err(error) = ctx.js_worker.execute_script_direct(script) {
        warn!("apply form control state: {error}");
        return false;
    }
    let html_snap = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snap).is_some()
}

/// 在 live page 脚本上下文执行自动化脚本，并应用其 DOM mutations。
///
/// https://w3c.github.io/webdriver/#execute-script
pub fn execute_automation_script(ctx: &mut PageScriptContext<'_>, script: &str) -> Result<(String, bool), String> {
    // t2-pb1 fix#9：自动化求值走优先通道（快照+执行成对，同通道 FIFO 保持顺序）——宿主
    // 发起的求值（CDP evaluate / Playwright title 等）不排在页面回调积压之后（bilibili
    // timer 臂级联曾把 title 求值压 17.5s，连带饿死其后的导航 IPC）。
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    let value = ctx.js_worker.execute_script_direct_priority(script)?;
    let html_snapshot = ctx.html.clone();
    let changed = apply_recorded_mutations(ctx, &html_snapshot).is_some();
    Ok((value, changed))
}

/// t2-pb1 fix#15：[`execute_automation_script`] 的可挂起形态结果。
pub enum AutomationEvalOutcome {
    /// 同步完成（值 + 是否有 DOM 变更）。
    Done(Result<(String, bool), String>),
    /// 有界等待超时——脚本已提交优先队列照常执行，reply 通道交还调用方挂起续答。
    Deferred(std::sync::mpsc::Receiver<Result<String, String>>),
}

/// t2-pb1 fix#15：[`execute_automation_script`] 的可挂起形态——宿主 Evaluate 家族
/// （CDP evaluate / Playwright 注入与求值）专用。worker 被长臂（bilibili timer 回调
/// 28-30s，纯 JS 执行墙）占住时，主循环至多等 [`zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT`]
/// 即放行导航等 IPC；超时把 reply 通道交还（脚本已在优先队列，结果晚至；mutation
/// 留待 checkpoint drain 落定，与异步回调同语义）。b1 旅程实测：settle 轮询 evaluate
/// 在臂上同步等 21.6s，其后 0.2s 的 Navigate IPC 撞上 15s 看门狗（epoch2 ERR_FAILED）。
pub fn execute_automation_script_deferrable(ctx: &mut PageScriptContext<'_>, script: &str) -> AutomationEvalOutcome {
    ctx.js_worker.set_dom_snapshot_priority(ctx.html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    match ctx
        .js_worker
        .execute_script_priority_deferrable(script, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT)
    {
        Ok(value) => {
            let html_snapshot = ctx.html.clone();
            let changed = apply_recorded_mutations(ctx, &html_snapshot).is_some();
            AutomationEvalOutcome::Done(value.map(|value| (value, changed)))
        }
        Err(rx) => AutomationEvalOutcome::Deferred(rx),
    }
}

/// 提交已经由异步页面任务写入的 DOM 变更。
///
/// `setTimeout`、Promise 和其他宿主回调会在首次页面脚本执行结束后继续运行；这些
/// 变更没有新的同步脚本边界可供 `run_page_scripts` 收集，须由 renderer 事件循环主动
/// 提交到活 DOM。
pub fn drain_pending_dom_mutations(ctx: &mut PageScriptContext<'_>) -> bool {
    // `setTimeout` / fetch 等宿主完成会先投递到 worker 的命令队列；只有进入一次
    // worker 执行边界时，回调才会在页面全局运行并记录 DOM mutation。
    // https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model
    if !ctx.js_worker.take_pending_async_callbacks() {
        return false;
    }
    // t2-pb1 fix#8：checkpoint 有界等待——页面回调积压（bilibili timer 臂级联 ~16s）时
    // 不阻塞 renderer 主循环 15s+（导航 IPC 饿死 → 二跳 ERR_FAILED）。超时放弃本轮：
    // 滞留空脚本为 worker 侧无害 no-op，ready 旗标由积压回调继续处理重新置位，下一轮
    // 重试；回调产生的 mutation 在队列中累积，至下一次成功 checkpoint 一并应用。
    let _ = ctx
        .js_worker
        .execute_script_direct_bounded("", std::time::Duration::from_millis(200));
    let html_snapshot = ctx.html.clone();
    apply_recorded_mutations(ctx, &html_snapshot).is_some()
}

/// 诊断可观测性：外链脚本文本以脚本 URL 命名执行——源尾追加 `//# sourceURL=<url>`
/// 注释。sandbox 执行通路（`v8::Script::compile` 无 ScriptOrigin / 经 wrapper 的间接
/// eval）脚本无名，异常 stack 全显 `<anonymous>`——bilibili video 页站点 loader 的
/// split TypeError 栈帧 `N @ <anonymous>:2:16812` 无法定位到 bundle 实证。V8 对无名
/// 脚本取 sourceURL 为脚本名（`Error.stack` / 未捕获报告显真名）；QuickJS 忽略该
/// 注释（优雅降级）。注释不改变脚本语义，仅命名。
///
/// 供**直接执行路径**（无 wrapper，如 `runtime.rs` tick_dynamic_scripts 动态脚本）使用；
/// classic wrapper 路径的 URL 命名由 [`script_run_classic_page`] 的 `source_url` 参数在
/// eval 源真末尾（导出后缀之后）置注释——V8 仅认末行 sourceURL，前置会被导出后缀整行
/// 拼接污染。URL 内控制字符剔除（换行会把注释后文本变回可执行代码）。
pub(crate) fn append_source_url(code: &str, url: &str) -> String {
    if url.is_empty() {
        return code.to_string();
    }
    let mut named = String::with_capacity(code.len() + url.len() + 16);
    named.push_str(code);
    named.push_str("\n//# sourceURL=");
    named.extend(url.chars().filter(|c| !c.is_control()));
    named
}

#[allow(clippy::too_many_arguments)] // 8 参 = 既有 7 参 + source_url 穿参，签名清晰优于打包结构体
fn execute_chunk<F: Fn(&str) -> Result<String, String>>(
    ctx: &mut PageScriptContext<'_>,
    html: &str,
    is_module: bool,
    module_url: &str,
    code: &str,
    fetch_text: &F,
    script_index: usize,
    source_url: Option<&str>,
) -> Result<(), String> {
    // t2-pb1 fix#10：脚本阶段走优先通道（快照+执行成对，同通道 FIFO 保持顺序）——解析期
    // 脚本先于已排队的 timer/fetch 回调运行是真实浏览器语义（parser 优先于任务队列）；
    // 页面回调流饱和时普通通道往返无界，脚本阶段曾单窗 9s+。
    ctx.js_worker.set_dom_snapshot_priority(html, ctx.url);
    ctx.js_worker.clear_mutations_fresh();
    if is_module {
        let mut registry: HashMap<String, String> = HashMap::new();
        collect_module_deps(fetch_text, module_url, code, &mut registry)?;
        let deps: Vec<(String, String)> = registry.into_iter().collect();
        ctx.js_worker.execute_module(code, module_url, &deps)?;
    } else {
        // classic 页面脚本：顶层 try-catch 包装捕获抛错（防持久 Isolate 中毒 + 让 R2940 报告生效）+
        // 执行期设/清 document.currentScript（R3258，script_run_classic_page）。
        run_page_script_caught(ctx.js_worker, code, script_index, source_url)?;
    }
    Ok(())
}

/// 执行 classic 页面 `<script>` 体，顶层 try-catch 包装未捕获 throw（[`script_run_classic_page`]）+
/// 执行期设/清 `document.currentScript`（R3258）。
/// 成功 → `Ok(())`；抛错 → sentinel 读出消息 → `Err(msg)`（调用方 `run_page_scripts` 据此报 window.onerror）。
/// 包装器 execute 不会抛（try-catch 兜底），随后的 sentinel 读取 execute 在干净 Isolate 上可靠。
fn run_page_script_caught(
    js_worker: &RendererJsWorker,
    code: &str,
    script_index: usize,
    source_url: Option<&str>,
) -> Result<(), String> {
    // t2-pb1 fix#10：脚本阶段执行走优先通道（配对快照同为优先，见 execute_chunk）。
    let _ = js_worker.execute_script_direct_priority(&script_run_classic_page(code, script_index, source_url));
    match js_worker.execute_script_direct_priority(&page_script_error_check()) {
        Ok(v) if v.is_empty() => Ok(()),
        Ok(msg) => Err(msg),
        Err(e) => Err(e),
    }
}

pub(crate) fn apply_recorded_mutations(ctx: &mut PageScriptContext<'_>, html: &str) -> Option<String> {
    let recorded = ctx
        .js_worker
        .mutations()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .drain(..)
        .collect::<Vec<_>>();
    if recorded.is_empty() {
        return None;
    }
    // PR #33 返修（review minor）：drain ⇒ bump_mut_drain_gen 不变式普遍成立——
    // 本站 drain 后不推快照（`ZW_RENDERER_TICK_PER_TASK=1` 的 tick_observers_with
    // per-task 循环排空队列直接下一轮 execute），无配对 view_gen 换代；不 bump 则
    // 下一轮查询恰落同 count 时视图缓存精确命中会端出 pre-drain 视图。其余 drain
    // 站点均有配对 set_dom_snapshot（view_gen 换代）兜底，本站补行使不变式无条件。
    zero_engine::js_dom_bridge::bump_mut_drain_gen();
    // R3254-M7'：JS focus()/blur() 是宿主状态变更（不写 DOM、不经渲染管线）——分离到
    // worker 的 focus 队列，renderer 事件循环在事务边界 drain 同步 retained 焦点状态。
    let focus_changes: Vec<Option<String>> = recorded
        .iter()
        .filter_map(|m| match m {
            DomMutation::FocusChanged { selector } => Some(selector.clone()),
            _ => None,
        })
        .collect();
    if !focus_changes.is_empty()
        && let Ok(mut queue) = ctx.js_worker.focus_changes().lock()
    {
        queue.extend(focus_changes);
    }
    let recorded: Vec<DomMutation> = recorded
        .into_iter()
        .filter(|m| !matches!(m, DomMutation::FocusChanged { .. }))
        .collect();
    if recorded.is_empty() {
        return None;
    }
    // M3-S9：webview 在场时 DOM 变更直接应用活 DOM（pipeline.cached_doc，免 HTML
    // 往返重 parse——与 browser tab_scripts 同机制）；无 webview（测试）回退 HTML 回写。
    if let Some(wv) = ctx.webview.as_deref_mut() {
        return match wv.apply_dom_mutations_and_render(&recorded) {
            Ok((_render, new_html, handle_selectors)) => {
                // P1a gBCR path A：merge handle→唯一选择器映射进 worker 持久 map。
                if !handle_selectors.is_empty()
                    && let Ok(mut map) = ctx.js_worker.handle_selector_map().lock()
                {
                    // iter 克隆 extend（同 webview batch_handle_selectors 惯用法）——
                    // handle_selectors 还要作 evict 的 batch_handles 借用。
                    map.extend(handle_selectors.iter().map(|(h, s)| (h.clone(), s.clone())));
                }
                // slice33：R100 失效契约（webview 侧 evict_removed_identities 同源）——
                // 被移除 handle 的 worker 侧 gBCR 反查表条目同步清除，防 RectBridge
                // 把旧 handle 锚到同选择器的新节点上。
                evict_removed_worker_handles(ctx, &recorded, &handle_selectors);
                *ctx.html = new_html.clone();
                // slice33（RP-3 跨文档残影）：apply 代际换代通知——与 webview
                // `apply_pending_shared_mutations`/`apply_mutations_subset` 的 R379/pa2b
                // 钩子同款（那两条共享队列路径 apply 后执行同一行；本路径此前缺失，
                // shim 侧对 apply 完全无感：移除补偿/解析补偿节点/融合基底缓存跨代际
                // 残留，同 id/同 selector 的新节点解析撞上被移除节点的旧 identity——
                // 残影出生点）。失败静默（钩子缺失 = 旧 shim 版本，零影响）。
                notify_shim_apply_generation(ctx);
                Some(new_html)
            }
            Err(e) => {
                warn!("apply DOM mutations: {e}");
                None
            }
        };
    }
    match apply_mutations_to_html_with_handles(html, &recorded) {
        Ok((new_html, handle_selectors)) => {
            // P1a gBCR path A：merge handle→唯一选择器映射进 worker 持久 map，供 RectBridge
            // handler 解析 handle-identity（createElement 元素）。upsert——同 handle 后续 id/class
            // 变更会更新（同 batch 内）；导航时 worker 清空。空 map（无 createElement）no-op。
            if !handle_selectors.is_empty()
                && let Ok(mut map) = ctx.js_worker.handle_selector_map().lock()
            {
                // iter 克隆 extend（同 path A 注）——handle_selectors 还要作 evict 的
                // batch_handles 借用。
                map.extend(handle_selectors.iter().map(|(h, s)| (h.clone(), s.clone())));
            }
            *ctx.html = new_html.clone();
            // slice33：R100 失效契约（同 webview 路径）。
            evict_removed_worker_handles(ctx, &recorded, &handle_selectors);
            // slice33：同 webview 路径——apply 代际换代通知（HTML 回写路径同边界语义）。
            notify_shim_apply_generation(ctx);
            Some(new_html)
        }
        Err(e) => {
            warn!("apply DOM mutations: {e}");
            None
        }
    }
}

/// slice33（RP-3）：apply 代际换代通知——host apply 完成后在 shim 侧执行
/// `__zw_apply_generation_bump`（R379/pa2b 钩子；定义见 js_dom_shim/part05.js）。
/// 与 webview `apply_pending_shared_mutations`/`apply_mutations_subset` 的既有
/// 通知点同口径：host 真相已更新，shim 同步补偿状态（移除标记、解析补偿节点、
/// 融合基底缓存）整体作废。有界等待（bump 为微秒级脚本；页面回调积压时不阻塞
/// 主循环，与 `drain_pending_dom_mutations` 的 200ms 上限同约定）。
fn notify_shim_apply_generation(ctx: &mut PageScriptContext<'_>) {
    let _ = ctx.js_worker.execute_script_direct_bounded(
        "if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();",
        std::time::Duration::from_millis(200),
    );
}

/// slice33（RP-3）：worker 侧 handle→selector 反查表的 Remove 失效——webview
/// `evict_removed_identities` 的 worker 镜像（gBCR path A 的 RectBridge 解析源）。
/// 仅在 apply 成功后调用（失败时 handle 仍存活，清除会使其 gBCR 失锚）。
/// slice33 缺陷轮 S-2/I-1：补 `Remove { selector }` 形（此前仅 RemoveHandle 臂——
/// selector 形移除的 worker 残账不清）；与 webview Remove 臂同款「同批 rebuild 且
/// 等值则跳过」守卫：`batch_handles`（render 第 3 元）仅含 apply 后仍在树内的
/// handle，同批先删旧位又重建同选择器新节点时 post-apply 绑定指向新 handle，
/// 删了会误杀；batch 成员 live，同批建又删的 handle 不会守卫穿透。
fn evict_removed_worker_handles(
    ctx: &mut PageScriptContext<'_>,
    recorded: &[DomMutation],
    batch_handles: &std::collections::HashMap<String, String>,
) {
    let touches_removed = recorded
        .iter()
        .any(|m| matches!(m, DomMutation::Remove { .. } | DomMutation::RemoveHandle { .. }));
    if !touches_removed {
        return;
    }
    if let Ok(mut map) = ctx.js_worker.handle_selector_map().lock() {
        for mutation in recorded {
            match mutation {
                DomMutation::RemoveHandle { handle } => {
                    map.remove(handle);
                }
                DomMutation::Remove { selector } => {
                    map.retain(|handle, sel| {
                        !(sel.as_str() == selector.as_str() && !batch_handles.contains_key(handle))
                    });
                }
                _ => {}
            }
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
            webview: None,
        };
        let _ = run_page_scripts(&mut ctx, true, |_u| Err::<String, String>("no external fetch".into()));
    }

    /// 外链脚本以脚本 URL 命名执行（`append_source_url` 源尾 `//# sourceURL=` 注释）：
    /// 脚本内 `Error().stack` 顶帧显 src 真名而非 `<anonymous>`。bilibili video 页动态
    /// bundle split TypeError 栈帧全匿名、无法定位 bundle 的可观测性修复钉（V8 对无名
    /// 脚本取 sourceURL 为脚本名；QuickJS 忽略该注释，本测试按默认 feature=v8 运行）。
    #[test]
    fn external_script_error_stack_shows_source_url() {
        let mut worker = RendererJsWorker::spawn(161);
        let html = "<html><body><script src='https://example.com/bundle.js'></script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");

        let mut buf = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut buf,
            url: "https://example.com/page",
            js_worker: &worker,
            webview: None,
        };
        run_page_scripts(&mut ctx, true, |u| {
            assert_eq!(u, "https://example.com/bundle.js");
            Ok("globalThis.__stackProbe = new Error().stack;".into())
        });

        let stack = wait_for_global(&worker, "__stackProbe", 1000);
        // 顶帧 = eval 的脚本文本，显 sourceURL 命名；wrapper 自身帧（间接 eval 调用点）
        // 保持 `<anonymous>`——引擎内部机制帧不在命名范围。
        assert!(
            stack.contains("at eval (https://example.com/bundle.js:"),
            "stack 顶帧应显脚本 URL，实际：{stack}"
        );
        worker.shutdown();
    }

    /// strict 顶层 `var` 形态的 sourceURL 回归钉（缺陷角色 N1 定向闭环）：R201 accessor
    /// 导出后缀拼接在 eval 源上——sourceURL 注释现由 `script_run_classic_page` 置于后缀
    /// **之后**（eval 源真末行）。V8 仅认末行注释，注释被后缀顶离末行时此形态栈帧回退
    /// `<anonymous>`（T4 实测）。WPT strict 测试库（dom/common.js 等）正中此型。
    #[test]
    fn external_script_strict_var_stack_shows_source_url() {
        let mut worker = RendererJsWorker::spawn(161);
        let html = "<html><body><script src='https://example.com/strict-bundle.js'></script></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");

        let mut buf = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut buf,
            url: "https://example.com/page",
            js_worker: &worker,
            webview: None,
        };
        run_page_scripts(&mut ctx, true, |u| {
            assert_eq!(u, "https://example.com/strict-bundle.js");
            Ok("\"use strict\";\nvar __strictProbe = new Error().stack;".into())
        });

        let stack = wait_for_global(&worker, "__strictProbe", 1000);
        assert!(
            stack.contains("at eval (https://example.com/strict-bundle.js:"),
            "strict var 形态 stack 顶帧应仍显脚本 URL，实际：{stack}"
        );
        worker.shutdown();
    }

    /// P-B1 导航让路：脚本间截获入站导航命令——首个脚本执行、其余中止，命令返回
    /// 给调用方，其余消息（inbound 中的非命令消息）原序保留在 deferred。
    /// 导航由首个脚本的 fetch 闭包注入（模拟脚本 1 执行期间导航到达）。
    #[test]
    fn run_page_scripts_yields_to_navigate_between_scripts() {
        let mut worker = RendererJsWorker::spawn(160);
        let html = "<html><body>\
            <script src='https://example.com/a.js'></script>\
            <script src='https://example.com/b.js'></script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");

        let mut buf = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut buf,
            url: "https://example.com/page",
            js_worker: &worker,
            webview: None,
        };
        let (tx, rx) = std::sync::mpsc::channel::<IpcMessage>();
        let tx_fetch = tx.clone();
        let fetch = move |u: &str| -> Result<String, String> {
            // 脚本取回期间入站两条消息：非命令在前、导航在后（到达序）。
            let _ = tx_fetch.send(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Heartbeat,
            });
            let _ = tx_fetch.send(IpcMessage {
                id: 2,
                kind: IpcMessageKind::Navigate(zero_protocol::message::NavigateParams {
                    url: "https://example.com/next".into(),
                    referrer: None,
                    navigation_epoch: 7,
                }),
            });
            if u.ends_with("a.js") {
                Ok("globalThis.__s1 = 'ran';".into())
            } else {
                Ok("globalThis.__s2 = 'ran';".into())
            }
        };

        let mut deferred: VecDeque<IpcMessage> = VecDeque::new();
        let yield_to_commands = ScriptPhaseYield {
            inbound_rx: &rx,
            deferred: &mut deferred,
        };
        let result = run_page_scripts_interruptible(&mut ctx, true, fetch, Some(yield_to_commands));

        assert!(result.aborted.is_some(), "导航命令应中止脚本阶段");
        assert_eq!(result.aborted.as_ref().unwrap().id, 2, "截获的应是导航命令");
        assert!(!result.changed);
        // 首个脚本已在让路检查点后执行。
        let s1 = wait_for_global(&worker, "__s1", 1000);
        assert_eq!(s1, "ran", "首个脚本应在让路检查后执行");
        // 后续脚本被丢弃（给足同步执行时间后仍未出现）。
        std::thread::sleep(std::time::Duration::from_millis(50));
        let s2 = wait_for_global(&worker, "__s2", 0);
        assert_eq!(s2, "undefined", "导航让路后剩余脚本不得执行");
        // 非命令消息原序保留（Heartbeat 自 inbound 扫入 kept 放回 deferred）。
        assert_eq!(deferred.len(), 1, "暂存消息不得丢失");
        assert_eq!(deferred.front().unwrap().id, 1);
        worker.shutdown();
    }

    /// The renderer process must retain and paint nested content inserted by a page script.
    #[test]
    fn page_script_inner_html_updates_webview_frame() {
        let html = r#"<html><head><style>
            .pointsPanel h2 { font-size: 2.3em; color: #aaa; text-align: center; line-height: 40px; margin: 0; }
            .pointsPanel h2 > strong, .pointsPanel h2 > span { display: inline-block; vertical-align: middle; transform: translateY(6px); }
            .pointsPanel h2 > strong { font-family: 'League Gothic', Impact; font-size: 3.8em; color: #0092bf; line-height: 145px; }
        </style></head><body><div id="score"></div><script>
            document.querySelector('#score').innerHTML =
                '<div class="pointsPanel"><h2><span>Your browser scores</span><strong>265</strong></h2></div>';
        </script></body></html>"#;
        let page_url = "https://zero.test/score";
        let mut worker = RendererJsWorker::spawn(150);
        worker.set_dom_snapshot(html, page_url);
        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
        webview.prepare_document_state(page_url);
        webview.load_html(html, None);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: Some(&mut webview),
        };

        assert!(run_page_scripts(&mut ctx, true, |_url| Err::<String, String>(
            "no fetch".into()
        )));
        assert!(ctx.html.contains("265"), "mutated HTML: {}", ctx.html);
        assert!(
            ctx.webview
                .as_ref()
                .expect("webview")
                .last_render()
                .expect("mutation render")
                .primitives()
                .glyphs
                .iter()
                .any(|glyph| glyph.glyph_id == '2' as u32),
            "nested score text must reach the renderer frame"
        );
        worker.shutdown();
    }

    #[test]
    fn async_script_mutation_is_committed_to_webview_frame() {
        let html = r#"<html><body><div id="score"></div></body></html>"#;
        let page_url = "https://zero.test/async-score";
        let mut worker = RendererJsWorker::spawn(151);
        worker.set_dom_snapshot(html, page_url);
        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
        webview.prepare_document_state(page_url);
        webview.load_html(html, None);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: Some(&mut webview),
        };

        // Drain startup microtasks before measuring the steady-state idle path.
        for _ in 0..16 {
            assert!(!drain_pending_dom_mutations(&mut ctx));
        }
        let execution_baseline = worker.execution_count_for_test();
        // 空闲 drain 不得为探测异步任务而进入 JS worker；否则 renderer 会以主循环频率
        // 反复编译空脚本并引发 V8 堆增长。
        for _ in 0..16 {
            assert!(!drain_pending_dom_mutations(&mut ctx));
        }
        assert_eq!(worker.execution_count_for_test(), execution_baseline);

        worker
            .execute_script_direct(
                "setTimeout(function() { document.querySelector('#score').innerHTML = '<strong>265</strong>'; }, 10);",
            )
            .expect("schedule score mutation");
        // Timer expiry only queues a host callback; `drain_pending_dom_mutations`
        // must enter the worker once to execute it.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(250);
        let mut committed = false;
        while !committed && std::time::Instant::now() < deadline {
            committed = drain_pending_dom_mutations(&mut ctx);
            if !committed {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        assert!(committed, "timer mutation was not committed before the deadline");
        // slice33：apply 代际换代通知（notify_shim_apply_generation）在 commit 时多一次
        // worker 执行（+1）——apply 边界的有意工作；空闲 drain 路径不触发（上方两条
        // 基线断言已证），V8 堆增长关注面不变。
        assert_eq!(worker.execution_count_for_test(), execution_baseline + 3);
        assert!(ctx.html.contains("265"), "mutated HTML: {}", ctx.html);
        assert!(
            ctx.webview
                .as_ref()
                .expect("webview")
                .last_render()
                .expect("mutation render")
                .primitives()
                .glyphs
                .iter()
                .any(|glyph| glyph.glyph_id == '2' as u32),
            "asynchronously inserted score must reach the renderer frame"
        );
        worker.shutdown();
    }

    #[test]
    fn styled_inline_block_heading_paints_text() {
        let css = r#"
            .pointsPanel { background: #fff; min-height: 145px; padding: 0 155px 0 0; }
            .pointsPanel h2 { font-size: 2.3em; color: #aaa; text-align: center; line-height: 40px; margin: 0; }
            .pointsPanel h2 > strong, .pointsPanel h2 > span {
                display: inline-block;
                vertical-align: middle;
                transform: translateY(6px);
            }
            .pointsPanel h2 > strong {
                font-family: 'League Gothic', Impact;
                font-weight: normal;
                font-size: 3.8em;
                color: #0092bf;
                line-height: 145px;
                margin: 0 10px;
            }
        "#;
        let html = r#"<html><body><div class="pointsPanel"><h2><span>Your browser scores</span><strong>265</strong><span>out of 586 points</span></h2></div></body></html>"#;
        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
        let result = webview.load_html(html, Some(css));

        assert!(
            result
                .primitives
                .glyphs
                .iter()
                .any(|glyph| glyph.glyph_id == '2' as u32),
            "inline-block heading score must produce a glyph"
        );
    }

    fn set_checked_for_test(ctx: &mut PageScriptContext<'_>, selector: &str, dispatch_events: bool) {
        let previous = zero_engine::checked_radio_group_selector(ctx.html, selector);
        assert!(apply_set_checked_without_events(ctx, selector, true));
        if let Some(previous) = previous.filter(|previous| previous != selector) {
            let _ = apply_set_checked_without_events(ctx, &previous, false);
        }
        if !dispatch_events {
            return;
        }
        let _ = dispatch_dom_event(ctx, true, selector, "input", None);
        let _ = dispatch_dom_event(ctx, true, selector, "change", None);
    }

    #[test]
    fn form_interaction_fixture_updates_input_value_and_result_text() {
        let html = include_str!("../../../examples/forms/form-interaction-test.html");
        let mut worker = RendererJsWorker::spawn(144);
        worker.set_dom_snapshot(html, "file:///examples/forms/form-interaction-test.html");
        run_scripts(html, &worker);

        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "file:///examples/forms/form-interaction-test.html",
            js_worker: &worker,
            webview: None,
        };
        let first = apply_text_input(&mut ctx, "#name", "A");
        assert!(first.html_changed);
        assert_eq!(first.snapshot.expect("first snapshot").value, "A");
        assert_eq!(zero_engine::query_attr_from_html(ctx.html, "#name", "value"), "");
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "输入事件：A");

        let second = apply_text_input(&mut ctx, "#name", "中文");
        assert!(second.html_changed);
        let second_snapshot = second.snapshot.expect("second snapshot");
        assert_eq!(second_snapshot.selection_start, 3);
        assert_eq!(second_snapshot.value, "A中文");
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "输入事件：A中文"
        );

        let note = apply_text_input(&mut ctx, "#note", "第二个输入框");
        assert!(note.html_changed);
        assert_eq!(note.snapshot.expect("note snapshot").value, "第二个输入框");
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#note"), "");

        worker.shutdown();
    }

    #[test]
    fn form_interaction_fixture_complete_sequence() {
        let html = include_str!("../../../examples/forms/form-interaction-test.html");
        let page_url = "https://zero.test/forms?__zero_test_state=1";
        let mut worker = RendererJsWorker::spawn(147);
        worker.set_dom_snapshot(html, page_url);
        run_scripts(html, &worker);

        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: None,
        };
        let state = |html: &str| -> serde_json::Value {
            serde_json::from_str(&zero_engine::query_text_from_html(html, "#test-state")).expect("fixture state JSON")
        };

        let typed = apply_text_input(&mut ctx, "#name", "abc");
        assert_eq!(typed.snapshot.expect("name snapshot").value, "abc");
        assert_eq!(state(ctx.html)["name"], "abc");
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "输入事件：abc");
        assert!(
            zero_engine::query_text_from_html(ctx.html, "title").starts_with("ZERO_TEST_STATE:"),
            "诊断模式必须把结构化状态同步到标题"
        );

        let deleted = apply_text_delete(&mut ctx, "#name");
        assert_eq!(deleted.snapshot.expect("delete snapshot").value, "ab");
        assert_eq!(state(ctx.html)["name"], "ab");
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "输入事件：ab");

        let note = apply_text_input(&mut ctx, "#note", "中文备注");
        assert_eq!(note.snapshot.expect("note snapshot").value, "中文备注");
        assert_eq!(state(ctx.html)["note"], "中文备注");
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "备注输入：中文备注"
        );

        set_checked_for_test(&mut ctx, "#subscribe", true);
        assert_eq!(state(ctx.html)["subscribe"], true);
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "复选框：已选中");

        set_checked_for_test(&mut ctx, "#plan-pro", true);
        assert_eq!(state(ctx.html)["plan"], "pro");
        assert!(!zero_engine::has_attribute(ctx.html, "#plan-basic", "checked"));
        assert!(zero_engine::has_attribute(ctx.html, "#plan-pro", "checked"));
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "套餐：pro");

        let button = dispatch_dom_event(&mut ctx, true, "#click", "click", None);
        assert!(button.html_changed);
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "普通按钮 click 事件已触发。"
        );

        assert!(apply_reset_on_click(&mut ctx, "#reset"));
        let reset_state = state(ctx.html);
        assert_eq!(reset_state["name"], "");
        assert_eq!(reset_state["note"], "");
        assert_eq!(reset_state["subscribe"], false);
        assert_eq!(reset_state["plan"], "basic");
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "表单已重置。");

        let submit = apply_submit_on_click(&mut ctx, "#submit");
        assert!(submit.html_changed);
        assert!(!submit.default_allowed);
        assert_eq!(state(ctx.html)["reason"], "submit");
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "提交事件已触发（已阻止导航）。"
        );

        worker.shutdown();
    }

    #[test]
    fn javascript_disabled_skips_listeners_not_default_actions() {
        let html = r#"<html><body>
            <form id="f">
              <input id="name" value="base">
              <textarea id="note">note</textarea>
              <input id="check" type="checkbox">
              <input id="basic" type="radio" name="plan" checked>
              <input id="pro" type="radio" name="plan">
              <button id="reset" type="reset">Reset</button>
            </form>
            <output id="out">unchanged</output>
            <script>
              document.querySelector('#name').addEventListener('input', () => {
                document.querySelector('#out').textContent = 'input-listener';
              });
              document.querySelector('#check').addEventListener('change', () => {
                document.querySelector('#out').textContent = 'change-listener';
              });
              document.querySelector('#f').addEventListener('reset', () => {
                document.querySelector('#out').textContent = 'reset-listener';
              });
            </script>
        </body></html>"#;
        let page_url = "https://zero.test/js-disabled";
        let mut worker = RendererJsWorker::spawn(148);
        worker.set_dom_snapshot(html, page_url);
        run_scripts(html, &worker);

        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
        webview.load_html(html, None);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: Some(&mut webview),
        };

        let inserted = apply_text_input_without_events(&mut ctx, "#name", "x");
        assert_eq!(inserted.snapshot.expect("input snapshot").value, "xbase");
        assert_eq!(
            ctx.webview
                .as_ref()
                .unwrap()
                .form_control_value_overrides()
                .get("#name")
                .map(String::as_str),
            Some("xbase")
        );
        let deleted = apply_text_delete_without_events(&mut ctx, "#name");
        assert_eq!(deleted.snapshot.expect("delete snapshot").value, "base");
        assert!(apply_text_input_without_events(&mut ctx, "#note", "x").html_changed);
        set_checked_for_test(&mut ctx, "#check", false);
        set_checked_for_test(&mut ctx, "#pro", false);
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#out"), "unchanged");
        assert!(zero_engine::has_attribute(ctx.html, "#check", "checked"));
        assert!(!zero_engine::has_attribute(ctx.html, "#basic", "checked"));
        assert!(zero_engine::has_attribute(ctx.html, "#pro", "checked"));

        assert!(apply_reset_on_click_without_events(&mut ctx, "#reset"));
        worker.set_dom_snapshot(ctx.html, page_url);
        let name = worker
            .execute_script_direct(&script_text_control_snapshot("#name"))
            .expect("name snapshot");
        let note = worker
            .execute_script_direct(&script_text_control_snapshot("#note"))
            .expect("note snapshot");
        assert_eq!(serde_json::from_str::<(String, usize, usize)>(&name).unwrap().0, "base");
        assert_eq!(serde_json::from_str::<(String, usize, usize)>(&note).unwrap().0, "note");
        assert!(!zero_engine::has_attribute(ctx.html, "#check", "checked"));
        assert!(zero_engine::has_attribute(ctx.html, "#basic", "checked"));
        assert!(!zero_engine::has_attribute(ctx.html, "#pro", "checked"));
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#out"), "unchanged");

        assert!(apply_text_input_without_events(&mut ctx, "#name", "after").html_changed);
        set_checked_for_test(&mut ctx, "#check", false);
        set_checked_for_test(&mut ctx, "#pro", false);
        assert_eq!(
            ctx.webview
                .as_ref()
                .unwrap()
                .form_control_value_overrides()
                .get("#name")
                .map(String::as_str),
            Some("afterbase")
        );

        worker.shutdown();
    }

    #[test]
    fn prevented_beforeinput_does_not_mutate_value() {
        let html = r#"<html><body>
            <input id="name" value="base">
            <script>
              globalThis.__inputCount = 0;
              var name = document.querySelector('#name');
              name.setSelectionRange(4, 4);
              name.addEventListener('beforeinput', function(event) {
                if (event.inputType === 'insertText') event.preventDefault();
              });
              name.addEventListener('input', function() { globalThis.__inputCount++; });
            </script>
        </body></html>"#;
        let page_url = "https://example.com/page";
        let mut worker = RendererJsWorker::spawn(149);
        worker.set_dom_snapshot(html, page_url);
        run_scripts(html, &worker);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: None,
        };

        let outcome = apply_text_input(&mut ctx, "#name", "A");
        let snapshot = outcome.snapshot.expect("text snapshot");
        assert_eq!(snapshot.value, "base");
        assert_eq!((snapshot.selection_start, snapshot.selection_end), (4, 4));
        assert!(!outcome.html_changed);
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__inputCount)").unwrap(),
            "0"
        );

        worker.shutdown();
    }

    #[test]
    fn composition_events_preserve_order_and_data() {
        let html = r#"<html><body><input id="name"><script>
            globalThis.__compositionEvents = [];
            var input = document.querySelector('#name');
            ['compositionstart','compositionupdate','compositionend'].forEach(function(type) {
                input.addEventListener(type, function(event) {
                    globalThis.__compositionEvents.push(type + ':' + event.data);
                });
            });
        </script></body></html>"#;
        let mut worker = RendererJsWorker::spawn(145);
        worker.set_dom_snapshot(html, "file:///composition.html");
        run_scripts(html, &worker);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "file:///composition.html",
            js_worker: &worker,
            webview: None,
        };

        assert!(!dispatch_composition_events(
            &mut ctx,
            true,
            "#name",
            &[("compositionstart", "拼"), ("compositionupdate", "拼音")],
        ));
        assert!(!dispatch_composition_events(
            &mut ctx,
            true,
            "#name",
            &[("compositionend", "中文")],
        ));
        assert_eq!(
            worker
                .execute_script_direct("globalThis.__compositionEvents.join('|')")
                .expect("event log"),
            "compositionstart:拼|compositionupdate:拼音|compositionend:中文"
        );
        worker.shutdown();
    }

    #[test]
    fn ime_commit_after_preedit_updates_live_webview() {
        let html = r#"<html><body><textarea id="note"></textarea></body></html>"#;
        let mut worker = RendererJsWorker::spawn(146);
        worker.set_dom_snapshot(html, "file:///ime-commit.html");
        let mut webview = zero_webview::WebViewBuilder::new().width(640).height(480).build();
        webview.prepare_document_state("file:///ime-commit.html");
        webview.load_html(html, None);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "file:///ime-commit.html",
            js_worker: &worker,
            webview: Some(&mut webview),
        };

        assert!(!dispatch_composition_events(
            &mut ctx,
            true,
            "#note",
            &[("compositionstart", "zhongwen"), ("compositionupdate", "zhongwen")],
        ));
        let preedit = zero_engine::DomMutation::SetFormComposition {
            selector: "#note".to_string(),
            text: "zhongwen".to_string(),
            selection_start: 0,
            selection_end: 0,
        };
        ctx.webview
            .as_deref_mut()
            .expect("webview")
            .apply_dom_mutations_and_render(std::slice::from_ref(&preedit))
            .expect("paint preedit");
        assert!(!dispatch_composition_events(
            &mut ctx,
            true,
            "#note",
            &[("compositionend", "中文备注")],
        ));

        let commit = apply_text_input(&mut ctx, "#note", "中文备注");
        assert!(commit.html_changed, "IME commit must apply a retained value mutation");
        assert_eq!(commit.snapshot.expect("committed snapshot").value, "中文备注");
        worker.shutdown();
    }

    #[test]
    fn host_reset_transaction_flushes_microtasks_after_default_action() {
        let html = include_str!("../../../examples/forms/form-interaction-test.html");
        let page_url = "https://zero.test/forms?__zero_test_state=1";
        let mut worker = RendererJsWorker::spawn(148);
        worker.set_dom_snapshot(html, page_url);
        run_scripts(html, &worker);

        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
        webview.prepare_document_state(page_url);
        webview.load_html(html, None);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: page_url,
            js_worker: &worker,
            webview: Some(&mut webview),
        };
        let typed = apply_text_input(&mut ctx, "#name", "ab");
        assert_eq!(typed.snapshot.expect("name snapshot").value, "ab");
        assert!(apply_set_checked_without_events(&mut ctx, "#subscribe", true));
        assert!(apply_set_checked_without_events(&mut ctx, "#plan-basic", false));
        assert!(apply_set_checked_without_events(&mut ctx, "#plan-pro", true));
        assert_eq!(
            worker
                .execute_script_direct(
                    "[document.querySelector('#subscribe').checked,\
                      document.querySelector('#subscribe').defaultChecked].join(',')"
                )
                .expect("checkbox state"),
            "true,false"
        );
        worker
            .execute_script_direct(
                "document.querySelector('#form').addEventListener('reset',function(){\
                 Promise.resolve().then(function(){\
                 globalThis.__promiseResetChecked=document.querySelector('#subscribe').checked;\
                 });\
                 });",
            )
            .expect("register promise reset listener");

        begin_host_action_transaction(&mut ctx);
        let dispatched = dispatch_dom_event(&mut ctx, true, "#form", "reset", None);
        assert!(dispatched.default_allowed);
        assert!(apply_form_reset_without_events(&mut ctx, "#form"));
        assert_eq!(
            worker
                .execute_script_direct(
                    "[document.querySelector('#subscribe').checked,\
                      document.querySelector('#subscribe').defaultChecked].join(',')"
                )
                .expect("reset checkbox state"),
            "false,false"
        );
        assert!(end_host_action_transaction(&mut ctx));
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__promiseResetChecked)")
                .expect("promise reset state"),
            "false"
        );

        let state: serde_json::Value =
            serde_json::from_str(&zero_engine::query_text_from_html(ctx.html, "#test-state"))
                .expect("fixture state JSON");
        assert_eq!(state["reason"], "reset");
        assert_eq!(state["name"], "");
        assert_eq!(state["subscribe"], false);
        assert_eq!(state["plan"], "basic");

        worker.shutdown();
    }

    #[test]
    fn form_interaction_fixture_runs_button_click_handler() {
        let html = include_str!("../../../examples/forms/form-interaction-test.html");
        let mut worker = RendererJsWorker::spawn(145);
        worker.set_dom_snapshot(html, "file:///examples/forms/form-interaction-test.html");
        run_scripts(html, &worker);

        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "file:///examples/forms/form-interaction-test.html",
            js_worker: &worker,
            webview: None,
        };
        let result = dispatch_dom_event(&mut ctx, true, "#click", "click", None);
        assert!(result.html_changed);
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "普通按钮 click 事件已触发。"
        );

        worker.shutdown();
    }

    #[test]
    fn form_interaction_fixture_dispatches_idless_reset_and_submit_buttons() {
        let html = include_str!("../../../examples/forms/form-interaction-test.html");
        let selectors = zero_engine::query_all_selector_list(html, "button")
            .split('|')
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(selectors.len(), 3);
        assert_ne!(selectors[1], selectors[2]);

        let mut worker = RendererJsWorker::spawn(146);
        worker.set_dom_snapshot(html, "file:///examples/forms/form-interaction-test.html");
        run_scripts(html, &worker);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "file:///examples/forms/form-interaction-test.html",
            js_worker: &worker,
            webview: None,
        };

        assert!(apply_reset_on_click(&mut ctx, &selectors[1]));
        assert_eq!(zero_engine::query_text_from_html(ctx.html, "#result"), "表单已重置。");
        let submit = apply_submit_on_click(&mut ctx, &selectors[2]);
        assert!(submit.html_changed);
        assert!(!submit.default_allowed);
        assert_eq!(
            zero_engine::query_text_from_html(ctx.html, "#result"),
            "提交事件已触发（已阻止导航）。"
        );

        worker.shutdown();
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
              globalThis.__imgload = 0;globalThis.__imgOrder=[];\
              document.addEventListener('DOMContentLoaded',function(){__imgOrder.push('dcl');});\
              window.addEventListener('load',function(){__imgOrder.push('window');});\
              img.addEventListener('load', function(){ globalThis.__imgload++;__imgOrder.push('img'); });\
              img.addEventListener('error', function(){ globalThis.__imgerr = 'fired'; });\
            </script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(
            &worker,
            Vec::new(),
            vec![ResourceElementEvent {
                tag: "img",
                url: "https://example.com/a.png".to_string(),
                outcome: ResourceElementOutcome::Loaded,
                natural_width: 3,
                natural_height: 2,
                media_duration_ms: None,
            }],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            worker
                .execute_script_direct(
                    "var probe=document.querySelectorAll('img')[0];\
                     [globalThis.__imgload,probe.complete,probe.naturalWidth,probe.naturalHeight,\
                      globalThis.__imgOrder.join(',')].join('|')",
                )
                .unwrap(),
            "1|true|3|2|dcl,img,window",
            "img 状态先提交、load 仅派发一次且早于 window load"
        );
        worker.shutdown();
    }

    /// fix#18 回归钉（紧随 r2943）：优先通道 fire-and-forget 提交（DCL/img commit/load）
    /// 紧随其后的普通通道探针不得越过它们——分派环 recv 醒来后必须再查优先通道
    /// （check-then-block 竞态的常驻守卫；无此再查时探针在 worker 空闲轮询窗口内
    /// 读到 `0|false|0|0|` 旧状态，r2943 类四例间歇性齐失败）。
    #[test]
    fn normal_probe_cannot_leapfrog_priority_lifecycle_dispatch() {
        let mut worker = RendererJsWorker::spawn(190);
        let html = "<html><body>\
            <img id='i1' src='https://example.com/a.png'>\
            <script>\
              var img = document.querySelectorAll('img')[0];\
              globalThis.__imgload = 0;globalThis.__imgOrder=[];\
              document.addEventListener('DOMContentLoaded',function(){__imgOrder.push('dcl');});\
              window.addEventListener('load',function(){__imgOrder.push('window');});\
              img.addEventListener('load', function(){ globalThis.__imgload++;__imgOrder.push('img'); });\
            </script>\
            </body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        finish_page_load(
            &worker,
            Vec::new(),
            vec![ResourceElementEvent {
                tag: "img",
                url: "https://example.com/a.png".to_string(),
                outcome: ResourceElementOutcome::Loaded,
                natural_width: 3,
                natural_height: 2,
                media_duration_ms: None,
            }],
            Vec::new(),
            Vec::new(),
        );
        let probe = worker
            .execute_script_direct(
                "var probe=document.querySelectorAll('img')[0];\
                 [globalThis.__imgload,probe.complete,probe.naturalWidth,probe.naturalHeight,\
                  globalThis.__imgOrder.join(',')].join('|')",
            )
            .unwrap();
        assert_eq!(probe, "1|true|3|2|dcl,img,window");
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

    /// R2953：事件循环混合异步顺序回归测试。锁住 spec 行为（经 probe 验证当前实现已正确）：
    /// ① 微任务链（M1→M2）在下一 macrotask 前整链排空；② 嵌套 setTimeout（T1 内排 T1b）按注册序
    /// FIFO——T2（脚本期注册，早于 T1b）先于 T1b 触发。预期顺序 T1,M1,M2,T2,T1b。
    /// 覆盖 microtask-before-next-macrotask + 微任务链排空 + timer 注册序 FIFO（R2952 协调线程保证）。
    #[test]
    fn event_loop_mixed_async_order_r2953() {
        let mut worker = RendererJsWorker::spawn(130);
        worker.set_dom_snapshot("<html><body></body></html>", "https://example.com/page");
        worker
            .execute_script_direct(
                "globalThis.__log = [];\
                 setTimeout(function(){\
                   globalThis.__log.push('T1');\
                   Promise.resolve().then(function(){\
                     globalThis.__log.push('M1');\
                     Promise.resolve().then(function(){ globalThis.__log.push('M2'); });\
                   });\
                   setTimeout(function(){ globalThis.__log.push('T1b'); }, 0);\
                 });\
                 setTimeout(function(){ globalThis.__log.push('T2'); }, 0);",
            )
            .unwrap();
        // 轮询直到全部 5 个回调触发。
        let probe = "String(globalThis.__log.length)";
        let start = std::time::Instant::now();
        loop {
            if worker.execute_script_direct(probe).unwrap_or_default() == "5" {
                break;
            }
            if start.elapsed().as_millis() >= 2000 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let log = worker
            .execute_script_direct("String(globalThis.__log.join(','))")
            .unwrap();
        assert_eq!(
            log, "T1,M1,M2,T2,T1b",
            "混合异步顺序 spec 一致：T1 → 微任务链 M1,M2 整链排空（下一 macrotask 前）→ T2（注册早于 T1b）→ T1b"
        );
        worker.shutdown();
    }

    /// R3050：reset 按钮 click → apply_reset_on_click 解析 enclosing form → 调 shim form.reset()
    /// → 派发 'reset' 事件（复用 R3048 reset 语义）。非 reset 按钮 → false 不触发。
    /// revert 控件正确性由 R3048 shim 测试覆盖；本测试验证 click→reset 接线（事件派发）。
    #[test]
    fn apply_reset_on_click_fires_reset_event_r3050() {
        let mut worker = RendererJsWorker::spawn(140);
        let html = "<html><body><form id='f'><input type='reset' id='r'></form></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        worker
            .execute_script_direct(
                "document.querySelector('#f').addEventListener('reset', function(){ globalThis.__rf='yes'; });",
            )
            .unwrap();
        let mut buf = html.to_string();
        let _changed = {
            let mut ctx = PageScriptContext {
                html: &mut buf,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_reset_on_click(&mut ctx, "#r")
        };
        assert_eq!(
            wait_for_global(&worker, "__rf", 1000),
            "yes",
            "reset 按钮 click → form.reset() 派发 reset 事件到 form listener"
        );
        // 非 reset 按钮（type=text）→ apply_reset_on_click false 返回，不调 form.reset（不派 reset 事件）。
        let html2 = "<html><body><form id='f2'><input type='text' id='t'></form></body></html>";
        worker.set_dom_snapshot(html2, "https://example.com/page");
        worker.execute_script_direct("globalThis.__rf='no';").unwrap();
        let mut buf2 = html2.to_string();
        let changed2 = {
            let mut ctx = PageScriptContext {
                html: &mut buf2,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_reset_on_click(&mut ctx, "#t")
        };
        assert!(!changed2, "非 reset 按钮 → apply_reset_on_click 返回 false");
        assert_eq!(
            wait_for_global(&worker, "__rf", 500),
            "no",
            "非 reset 按钮 → 不派发 reset 事件（__rf 保持 'no'）"
        );
        worker.shutdown();
    }

    /// R3053：click 命中 hash 链接（`<a href="#sec">`）→ apply_set_hash_on_click 设 location.hash，
    /// 派发 hashchange 到 window listener（SPA hash 路由核心交互）。非 hash 锚 → false 不派 hashchange。
    #[test]
    fn apply_set_hash_on_click_fires_hashchange_r3053() {
        let mut worker = RendererJsWorker::spawn(141);
        let html = "<html><body><a id='a' href='#sec'>l</a></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");
        worker
            .execute_script_direct("addEventListener('hashchange', function(e){ globalThis.__hc = e.newURL; });")
            .unwrap();
        let mut buf = html.to_string();
        let _changed = {
            let mut ctx = PageScriptContext {
                html: &mut buf,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_set_hash_on_click(&mut ctx, "#a")
        };
        // hash 链接 click → location.hash='#sec' → hashchange.newURL = 当前 url + '#sec'。
        assert_eq!(
            wait_for_global(&worker, "__hc", 1000),
            "https://example.com/page#sec",
            "hash 链接 click → location.hash 设值 + 派发 hashchange（newURL 含 #sec）"
        );
        // location.hash 反映新值。
        assert_eq!(
            worker.execute_script_direct("location.hash").unwrap_or_default(),
            "#sec",
            "location.hash 反映 '#sec'"
        );

        // 非 hash 锚（绝对 href）→ apply_set_hash_on_click 返回 false，不设 hash 不派 hashchange。
        worker.set_dom_snapshot(
            "<html><body><a id='u' href='https://x.com/'>l</a></body></html>",
            "https://example.com/page",
        );
        worker.execute_script_direct("globalThis.__hc='none';").unwrap();
        let mut buf2 = String::from("<html><body><a id='u' href='https://x.com/'>l</a></body></html>");
        let changed2 = {
            let mut ctx = PageScriptContext {
                html: &mut buf2,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_set_hash_on_click(&mut ctx, "#u")
        };
        assert!(!changed2, "非 hash 锚（绝对 href）→ apply_set_hash_on_click 返回 false");
        // __hc 保持 'none'（poll 超时返当前值 'none'，未派 hashchange）。
        assert_eq!(
            wait_for_global(&worker, "__hc", 300),
            "none",
            "非 hash 锚 → 不派发 hashchange（__hc 保持 'none'）"
        );
        worker.shutdown();
    }

    /// R3054：apply_submit_on_click 返回 SubmitOutcome——default_allowed 反映 submit 是否被 preventDefault。
    /// 未 preventDefault → default_allowed=true（→ GET 导航）；preventDefault → false（不导航）。
    #[test]
    fn apply_submit_outcome_tracks_preventdefault_r3054() {
        let mut worker = RendererJsWorker::spawn(142);
        let html = "<html><body><form id='f' action='/s'>\
            <input name='q' value='x'>\
            <button id='b' type='submit'>Go</button>\
            </form></body></html>";

        // ① 无 preventDefault listener → default_allowed=true（应导航）。
        worker.set_dom_snapshot(html, "https://example.com/page");
        let mut buf = html.to_string();
        let outcome = {
            let mut ctx = PageScriptContext {
                html: &mut buf,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_submit_on_click(&mut ctx, "#b")
        };
        assert!(
            outcome.default_allowed,
            "submit 未 preventDefault → default_allowed=true"
        );

        // ② preventDefault listener → default_allowed=false（不应导航）。
        worker.set_dom_snapshot(html, "https://example.com/page");
        worker
            .execute_script_direct(
                "document.getElementById('f').addEventListener('submit', function(e){ e.preventDefault(); globalThis.__pv='yes'; });",
            )
            .unwrap();
        let mut buf2 = html.to_string();
        let outcome2 = {
            let mut ctx = PageScriptContext {
                html: &mut buf2,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_submit_on_click(&mut ctx, "#b")
        };
        assert!(
            !outcome2.default_allowed,
            "submit preventDefault → default_allowed=false（不导航）"
        );
        assert_eq!(
            wait_for_global(&worker, "__pv", 1000),
            "yes",
            "preventDefault listener 触发（submit 事件已派发）"
        );

        // ③ 非 submit 按钮（type=button）→ apply_submit_on_click 返回默认 outcome（不提交）。
        let html3 = "<html><body><form id='f3'><button id='nb' type='button'>No</button></form></body></html>";
        worker.set_dom_snapshot(html3, "https://example.com/page");
        let mut buf3 = html3.to_string();
        let outcome3 = {
            let mut ctx = PageScriptContext {
                html: &mut buf3,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_submit_on_click(&mut ctx, "#nb")
        };
        assert!(
            !outcome3.default_allowed && !outcome3.html_changed,
            "type=button 非 submit → 默认 outcome（不提交/不导航）"
        );
        worker.shutdown();
    }

    #[test]
    fn prevented_reset_and_submit_skip_default_actions() {
        let html = r#"<html><body>
            <form id="f" action="/submitted">
              <input id="name" name="name" value="base">
              <button id="reset" type="reset">Reset</button>
              <button id="submit" type="submit">Submit</button>
            </form>
            <script>
              var form = document.querySelector('#f');
              form.addEventListener('reset', function(event) { event.preventDefault(); });
              form.addEventListener('submit', function(event) { event.preventDefault(); });
            </script>
        </body></html>"#;
        let mut worker = RendererJsWorker::spawn(150);
        worker.set_dom_snapshot(html, "https://example.com/page");
        run_scripts(html, &worker);
        let mut rendered_html = html.to_string();
        let mut ctx = PageScriptContext {
            html: &mut rendered_html,
            url: "https://example.com/page",
            js_worker: &worker,
            webview: None,
        };

        let edited = apply_text_input(&mut ctx, "#name", "x");
        assert_eq!(edited.snapshot.expect("edited snapshot").value, "basex");
        assert!(!apply_reset_on_click(&mut ctx, "#reset"));
        let current = worker
            .execute_script_direct(&script_text_control_snapshot("#name"))
            .expect("current snapshot");
        assert_eq!(
            serde_json::from_str::<(String, usize, usize)>(&current).unwrap().0,
            "basex"
        );

        let submit = apply_submit_on_click(&mut ctx, "#submit");
        assert!(!submit.default_allowed);
        assert!(!submit.html_changed);
        worker.shutdown();
    }

    /// R3057：apply_javascript_href 在 click `<a href="javascript:...">` 时执行 JS 体（页面全局）。
    /// JS 体改 DOM（如 innerHTML）则 apply 返 true；空体 / 非 javascript: href → 不执行。
    #[test]
    fn apply_javascript_href_executes_body_r3057() {
        let mut worker = RendererJsWorker::spawn(143);
        let html = "<html><body><a id='a' href=\"javascript:document.body.setAttribute('data-x','hit')\">run</a></body></html>";
        worker.set_dom_snapshot(html, "https://example.com/page");

        // ① javascript: 体执行 → 改 body 的 data-x 属性 → apply 返 true（DOM 变更）。
        let mut buf = html.to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut buf,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_javascript_href(&mut ctx, "#a")
        };
        assert!(changed, "javascript: 体执行改 body data-x → apply 返 true");
        assert!(
            buf.contains("data-x=\"hit\"") || buf.contains("data-x='hit'"),
            "body data-x=hit 写入 HTML：{buf}"
        );

        // ② 空 javascript: 体 → 执行空脚本 no-op，apply 返 false（无 mutation）。
        let html2 = "<html><body><a id='e' href='javascript:'>x</a></body></html>";
        worker.set_dom_snapshot(html2, "https://example.com/page");
        let mut buf2 = html2.to_string();
        let changed2 = {
            let mut ctx = PageScriptContext {
                html: &mut buf2,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_javascript_href(&mut ctx, "#e")
        };
        assert!(!changed2, "空 javascript: 体 → no-op，apply 返 false");

        // ③ 非 javascript: href（绝对 URL）→ apply 返 false（gate 不命中，不执行）。
        let html3 = "<html><body><a id='u' href='https://x.com/'>l</a></body></html>";
        worker.set_dom_snapshot(html3, "https://example.com/page");
        let mut buf3 = html3.to_string();
        let changed3 = {
            let mut ctx = PageScriptContext {
                html: &mut buf3,
                url: "https://example.com/page",
                js_worker: &worker,
                webview: None,
            };
            apply_javascript_href(&mut ctx, "#u")
        };
        assert!(!changed3, "非 javascript: href → gate 不命中，apply 返 false");
        worker.shutdown();
    }

    /// t2-pb1 fix#15：`execute_automation_script_deferrable` Done 面——worker 空闲时
    /// 有界等待内完成，返回值与同步路径同值同形（Deferred 挂起面由 js_worker
    /// `execute_script_priority_deferrable_*` 三测试覆盖，挂起表续答在 automation
    /// 层由 ev19 端到端覆盖）。
    #[test]
    fn automation_deferrable_done_matches_sync_shape() {
        let mut worker = RendererJsWorker::spawn(81);
        let mut html = String::from("<html><body><div id='a'>t</div></body></html>");
        let mut ctx = PageScriptContext {
            html: &mut html,
            url: "about:blank",
            js_worker: &worker,
            webview: None,
        };
        match execute_automation_script_deferrable(&mut ctx, "String(40+2)") {
            AutomationEvalOutcome::Done(Ok((value, _changed))) => assert_eq!(value, "42"),
            AutomationEvalOutcome::Done(Err(e)) => panic!("worker 空闲时须成功，实际错误 {e}"),
            AutomationEvalOutcome::Deferred(_) => panic!("worker 空闲时须同步完成，实际挂起"),
        }
        worker.shutdown();
    }

    /// t2-pb1 F4（首轮缺陷审查 2026-10-02）：挂起求值晚至完成后，其 DOM mutation
    /// 滞留 worker 队列（期间无 checkpoint 会 apply 它们），补答侧的
    /// `apply_recorded_mutations` 须能认领并落进宿主 HTML——此前补答路径不 apply，
    /// 变更被下一脚本入口的 clear() 静默丢弃（数据丢失类缺陷）。
    #[test]
    fn automation_deferrable_mutations_survive_until_answer_applies() {
        let mut worker = RendererJsWorker::spawn(82);
        // 长臂占住 worker（公共 API：挂起的 deferrable 求值即臂）。
        let busy_rx = match worker.execute_script_priority_deferrable(
            "var s=0;for(var i=0;i<5e8;i++)s+=i;String(s)",
            std::time::Duration::from_millis(1),
        ) {
            Err(rx) => rx,
            _ => panic!("长臂入队应挂起"),
        };
        std::thread::sleep(std::time::Duration::from_millis(120));
        let mut html = String::from("<html><body><div id='a'>t</div></body></html>");
        let mut ctx = PageScriptContext {
            html: &mut html,
            url: "about:blank",
            js_worker: &worker,
            webview: None,
        };
        // 长臂期间挂起的求值：改 DOM + 返回值。
        let rx = match execute_automation_script_deferrable(
            &mut ctx,
            "document.getElementById('a').textContent = 'late'; 'ok'",
        ) {
            AutomationEvalOutcome::Deferred(rx) => rx,
            _ => panic!("长臂期间应挂起"),
        };
        // 臂结束 → 挂起求值照常执行（先于其入队的快照已就位）。
        let _ = busy_rx.recv_timeout(std::time::Duration::from_secs(35));
        let value = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("挂起求值须晚至完成")
            .expect("挂起求值须成功");
        assert_eq!(value, "ok");
        // 补答时点 mutation 仍滞留队列（此间无 checkpoint 会 apply 它们）。
        let recorded_len = ctx
            .js_worker
            .mutations()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len();
        assert!(recorded_len > 0, "挂起求值的 DOM mutation 须滞留队列待补答认领");
        // 补答侧 apply（poll_deferred_automation_replies 的修复路径）→ 变更落宿主 HTML。
        let html_snapshot = ctx.html.clone();
        let applied = apply_recorded_mutations(&mut ctx, &html_snapshot);
        assert!(applied.is_some(), "补答 apply 须产出新 HTML");
        assert!(ctx.html.contains(">late<"), "挂起求值的 DOM 变更须落入宿主 HTML");
        worker.shutdown();
    }
}
