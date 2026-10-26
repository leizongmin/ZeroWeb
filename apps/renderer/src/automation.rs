//! Live renderer automation operations shared by WebDriver and WPT testdriver.

use zero_protocol::message::{
    AutomationElementRef, AutomationError, AutomationErrorCode, AutomationKey, AutomationOperation, AutomationRequest,
    AutomationResponse, AutomationResult, AutomationStateQuery, AutomationValue, IpcMessage, IpcMessageKind,
    KeyboardEventParams, KeyboardEventType,
};

use super::{PageScriptContext, RendererRuntime};
use crate::js_worker::RendererJsWorker;

use std::collections::VecDeque;

/// t2-pb1 fix#15：挂起中的自动化 Evaluate 回复——Evaluate 家族（ExecuteScript /
/// EvaluateRetaining）在 worker 被长臂（bilibili timer 回调 28-30s）占住时，主循环
/// 有界等待 [`zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT`] 后放行导航等 IPC，
/// 脚本留在优先队列照常执行，本结构由 renderer 主循环轮询补答。
pub(crate) struct PendingAutomationReply {
    pub(crate) request_id: u64,
    pub(crate) rx: std::sync::mpsc::Receiver<Result<String, String>>,
    pub(crate) kind: DeferredEvalKind,
}

/// t2-pb1 fix#15：挂起求值的响应构造方式（与同步路径同一段包络解析）。
pub(crate) enum DeferredEvalKind {
    /// ExecuteScript：JSON 包络字符串 → [`AutomationResult::Value`]。
    ScriptValue,
    /// EvaluateRetaining：句柄/值包络（pending 信号 → evaluate cannot be awaited）。
    Retaining,
    /// t2-pb1 fix#15b：CallFunctionOnHandle/Release 家族——句柄/值包络，pending 信号
    /// 续跑 awaitPromise 落定循环（与同步路径同语义；release 脚本不会产生 pending）。
    HandleOp { group: String, return_by_value: bool },
}

/// t2-pb1 fix#15b：[`RendererRuntime::run_handle_operation_deferrable`] 的结果——
/// Deferred 表示已入挂起表，调用方返回 Empty 占位（被长度差检测拦截）。
enum AutomationHandleFlow {
    Done(HandleOutcome),
    Deferred,
}

// t-p106 诊断：automation deferred-reply 三点仪拍侧信道（ZW_EXEC_TIMING 同型 env 门控，
// 零默认开销）。worker reply.send / renderer poll 交付 / headless 匹配三点带 epoch 毫秒
// 落同一文件，时间轴对齐后锁定回复丢失环节。仅诊断用，不参与行为。
static AUTOMATION_TRACE_FILE: std::sync::LazyLock<std::sync::Mutex<Option<std::fs::File>>> =
    std::sync::LazyLock::new(|| match std::env::var("ZW_AUTOMATION_TRACE") {
        Ok(path) if path != "1" => {
            std::sync::Mutex::new(std::fs::OpenOptions::new().create(true).append(true).open(path).ok())
        }
        _ => std::sync::Mutex::new(None),
    });

/// t-p106 诊断：仪拍是否启用（未配置 `ZW_AUTOMATION_TRACE` 文件路径时恒 false，
/// 调用点先查本函数再 format，热路径零格式化开销）。
pub(crate) fn automation_trace_enabled() -> bool {
    AUTOMATION_TRACE_FILE
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// t-p106 诊断：追加一行仪拍（调用方自含前缀与时间戳）。文件不可写时静默跳过。
pub(crate) fn automation_trace_line(line: &str) {
    if let Ok(mut guard) = AUTOMATION_TRACE_FILE.lock()
        && let Some(file) = guard.as_mut()
    {
        use std::io::Write as _;
        let _ = writeln!(file, "{line}");
    }
}

/// t-p106 诊断：epoch 毫秒时间戳（对齐 ZW_EXEC_TIMING 的 t= 字段口径）。
pub(crate) fn automation_trace_epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

/// t-p106：ScriptValue/Retaining 就绪值 → 回复结果——poll 补答与页面脚本阶段中交付
/// （[`deliver_ready_deferred_replies`]）共用的包络映射；HandleOp 有 awaitPromise 状态机
/// 续跑，不经本函数。
fn eval_value_to_result(kind: &DeferredEvalKind, value: &str) -> Result<AutomationResult, AutomationError> {
    match kind {
        DeferredEvalKind::ScriptValue => Ok(AutomationResult::Value(automation_value_from_script(value))),
        DeferredEvalKind::Retaining => match parse_handle_operation_envelope(value) {
            Ok(HandleOutcome::Result(result)) => Ok(result),
            Ok(HandleOutcome::Pending) => Err(internal_error("evaluate cannot be awaited".into())),
            Err(e) => Err(e),
        },
        DeferredEvalKind::HandleOp { .. } => Err(internal_error("handle op has no eval-style completion".into())),
    }
}

/// t-p106 形态 A/B 丢失修复：页面脚本阶段中交付挂起自动化回复。
///
/// 主循环被同步页面脚本阶段阻塞期间，16ms tick 的 poll 补答不可达——github.com 实测
/// 单阶段阻塞 42s+（RUN/tmp/p106-diag/d2-areply.trace：r-defer req=2 → r-phase-end
/// dur_ms=42257 → r-poll req=2，headless 35s deadline 先到，迟到回复按消息 id 不匹配
/// 被丢弃）。脚本间让路点调用本函数取走已就绪条目并即答，返回成功交付数（>0 表示
/// worker 可能累积了该批求值的 DOM mutation，调用方须 apply_recorded_mutations 收编进
/// 阶段批量渲染边界，与 t2-pb1 F4 的 poll 补答 apply 同语义）。
///
/// HandleOp 待续条目不取：try_recv 会吃掉 pending 信号，awaitPromise 续跑依赖主循环
/// 全量状态，原样留待 poll 补答。`send` 由调用方闭包提供（只借 outbound/compositor
/// 字段，与脚本阶段对 cached_html/js_worker/webview 的借用不相交）；epoch/generation
/// 为调用方快照——阶段内无导航（让路中止语义保证），与 poll 发送时读取同值。
pub(super) fn deliver_ready_deferred_replies(
    pending: &mut Vec<PendingAutomationReply>,
    send: &mut dyn FnMut(IpcMessage) -> Result<(), String>,
    navigation_epoch: u64,
    document_generation: u64,
) -> usize {
    let mut delivered = 0usize;
    let mut kept = Vec::new();
    for reply in pending.drain(..) {
        if matches!(reply.kind, DeferredEvalKind::HandleOp { .. }) {
            kept.push(reply);
            continue;
        }
        let result = match reply.rx.try_recv() {
            Ok(Ok(value)) => eval_value_to_result(&reply.kind, &value),
            Ok(Err(message)) => Err(automation_error(AutomationErrorCode::JavascriptError, message)),
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                kept.push(reply);
                continue;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(automation_error(
                AutomationErrorCode::JavascriptError,
                "deferred evaluate cancelled (navigation reset or worker exit)",
            )),
        };
        // t-p106 诊断：阶段中交付点仪拍（与 poll 交付点同侧信道，via=phase 区分）。
        if automation_trace_enabled() {
            let outcome = if result.is_ok() { "ok" } else { "err" };
            automation_trace_line(&format!(
                "[areply] t={} r-poll req={} outcome={outcome} via=phase",
                automation_trace_epoch_ms(),
                reply.request_id
            ));
        }
        if result.is_ok() {
            delivered += 1;
        }
        let message = IpcMessage {
            id: reply.request_id,
            kind: IpcMessageKind::AutomationResponse(AutomationResponse {
                navigation_epoch,
                document_generation,
                result,
            }),
        };
        // 单条发送失败（IPC 断连）不中断——与 poll 补答路径同（t2-pb1 F8①）。
        if let Err(e) = send(message) {
            tracing::warn!("deferred automation reply send failed: {e}");
        }
    }
    *pending = kept;
    delivered
}

/// t-p106 第二机制：让路点 dispatch 阶段中到达的自动化 Evaluate。
///
/// 自动化 Evaluate（CDP `Runtime.evaluate` / Playwright 注入求值）不属于页面脚本
/// 管线，是宿主动作：允许在文档中间态执行（真实浏览器 DevTools/CDP 同语义，
/// https://chromedevtools.github.io/devtools-protocol/tot/Runtime/#method-evaluate）。
/// 但同步阶段阻塞主循环期间 `handle_automation_request` 不可达，staged 队列中的
/// eval 只能等阶段结束才 dispatch——github.com 实测（RUN/tmp/p106-diag/d3-*）阶段
/// 68.6s 内到达的求值全部滞留，阶段后重放补齐 ~28s，headless 35s deadline 先到。
///
/// 本函数在脚本间让路点按 FIFO 抽取 staged 中的 ExecuteScript / EvaluateRetaining，
/// 以当前 `html` 快照 dispatch 进 worker：
///
/// * 快照/代际语义与 [`super::page_scripts::execute_automation_script_deferrable`]
///   同型——`set_dom_snapshot_priority` + `clear_mutations_fresh` 配对；调用方顺序
///   保证本函数在导航让路检查之后、交付 apply 之前执行，阶段内无导航（让路中止
///   语义保证），eval 记录的 mutation 由同让路点 apply 收编，不跨 document_generation
///   （t3 defect-r1 D1 的收编门语义不变）；
/// * HandleOp 家族（awaitPromise 状态机依赖主循环全量状态）与同步操作（Find/Click
///   等需主循环元素状态）不抽取，保持 staged 留给主循环；
/// * 让路点 worker 恰空闲时求值立即完成：合成已就绪挂起条目走统一交付路径
///   （[`deliver_ready_deferred_replies`]），交付语义单一；
/// * 空 script 不在此处替答——保持 staged，由主循环按 InvalidArgument 即答。
pub(super) fn dispatch_staged_automation_evals(
    staged: &mut VecDeque<IpcMessage>,
    html: &str,
    url: &str,
    js_worker: &RendererJsWorker,
    pending: &mut Vec<PendingAutomationReply>,
) {
    let mut evals = Vec::new();
    let mut kept = VecDeque::new();
    while let Some(message) = staged.pop_front() {
        let eval = match &message.kind {
            IpcMessageKind::AutomationRequest(request) => match &request.operation {
                // 与 handle_automation_request 的 ExecuteScript 臂同款校验与包络。
                AutomationOperation::ExecuteScript { script, arguments } if !script.is_empty() => {
                    let arguments =
                        serde_json::Value::Array(arguments.iter().map(automation_value_to_json).collect::<Vec<_>>());
                    Some((
                        message.id,
                        format!(
                            "(function(){{var __zw_value=(function(){{{script}\n}}).apply(null,{arguments});\
                             return JSON.stringify({{defined:typeof __zw_value!=='undefined',value:__zw_value}});}})()"
                        ),
                        DeferredEvalKind::ScriptValue,
                    ))
                }
                AutomationOperation::EvaluateRetaining {
                    script,
                    group,
                    return_by_value,
                } if !script.is_empty() => {
                    let group = group.clone().unwrap_or_else(|| DEFAULT_OBJECT_GROUP.to_string());
                    Some((
                        message.id,
                        evaluate_retaining_script(script, &group, *return_by_value),
                        DeferredEvalKind::Retaining,
                    ))
                }
                _ => None,
            },
            _ => None,
        };
        match eval {
            Some(eval) => evals.push(eval),
            None => kept.push_back(message),
        }
    }
    *staged = kept;
    for (request_id, source, kind) in evals {
        // t-p106 诊断：阶段中 dispatch 点仪拍（via=phase，与交付点 r-poll 区分）。
        if automation_trace_enabled() {
            automation_trace_line(&format!(
                "[areply] t={} r-dispatch req={request_id} via=phase",
                automation_trace_epoch_ms()
            ));
        }
        js_worker.set_dom_snapshot_priority(html, url);
        js_worker.clear_mutations_fresh();
        match js_worker.execute_script_priority_deferrable(&source, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT) {
            // 让路点 worker 空闲常立即完成：合成已就绪条目，交付路径照常补答
            //（含脚本错误 Err——交付点按 JavascriptError 应答，与同步路径同语义）。
            Ok(value) => {
                let (reply_tx, reply_rx) = std::sync::mpsc::channel();
                let _ = reply_tx.send(value);
                drop(reply_tx);
                pending.push(PendingAutomationReply {
                    request_id,
                    rx: reply_rx,
                    kind,
                });
            }
            Err(rx) => pending.push(PendingAutomationReply { request_id, rx, kind }),
        }
    }
}

impl RendererRuntime {
    pub(super) fn handle_automation_request(
        &mut self,
        request_id: u64,
        request: AutomationRequest,
    ) -> Result<(), String> {
        // t2-pb1 fix#15：Evaluate 家族可挂起——挂起请求进 pending 表（长度差为标记），
        // 不立即发送响应，由主循环 [`Self::poll_deferred_automation_replies`] 补答。
        let pending_before = self.pending_automation_replies.len();
        let result = self.execute_automation_request(request_id, request);
        if self.pending_automation_replies.len() > pending_before {
            return Ok(());
        }
        self.send_regular_with_id(
            request_id,
            IpcMessageKind::AutomationResponse(AutomationResponse {
                navigation_epoch: self.navigation_epoch,
                document_generation: self.document_generation,
                result,
            }),
        )
    }

    fn execute_automation_request(
        &mut self,
        request_id: u64,
        request: AutomationRequest,
    ) -> Result<AutomationResult, AutomationError> {
        let font_loader = self.font_loader.duplicate();
        let font_id = self.font_id;
        super::text_metrics::with_measure_ctx_opt(&font_loader, font_id, || {
            self.execute_automation_operation(request_id, request.operation)
        })
    }

    /// t2-pb1 fix#15：Evaluate 家族的可挂起执行。有界等待内完成 → 与同步路径完全一致
    /// （尾随同步 + 包络转换）；超时 → reply 通道入 pending 表挂起续答（返回值仅占位，
    /// 被 [`Self::handle_automation_request`] 的长度差检测拦截，不会作为响应发出）。
    /// 仅此两操作可挂起：其余操作的结果驱动后续分支（句柄状态机），挂起会破坏语义。
    fn execute_eval_deferrable(
        &mut self,
        request_id: u64,
        source: String,
        kind: DeferredEvalKind,
    ) -> Result<AutomationResult, AutomationError> {
        let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let flow = {
            let mut context = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
                webview: self.webview.as_mut(),
            };
            super::page_scripts::execute_automation_script_deferrable(&mut context, &source)
        };
        match flow {
            super::page_scripts::AutomationEvalOutcome::Done(outcome) => {
                let (value, changed) =
                    outcome.map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?;
                self.automation_eval_sync_tail(changed)?;
                Ok(match kind {
                    DeferredEvalKind::ScriptValue => AutomationResult::Value(automation_value_from_script(&value)),
                    DeferredEvalKind::Retaining => match parse_handle_operation_envelope(&value)? {
                        HandleOutcome::Result(result) => result,
                        HandleOutcome::Pending => {
                            return Err(internal_error("evaluate cannot be awaited".into()));
                        }
                    },
                    // fix#15b：HandleOp 只由 run_handle_operation_deferrable 构造，
                    // 不会流经本函数（其 Done 路径自带状态机收尾）。
                    DeferredEvalKind::HandleOp { .. } => {
                        return Err(internal_error("handle op has no eval-style completion".into()));
                    }
                })
            }
            super::page_scripts::AutomationEvalOutcome::Deferred(rx) => {
                // t-p106 诊断：挂起入口仪拍——回复丢失排查起点（request_id + 挂起表深度）。
                if automation_trace_enabled() {
                    automation_trace_line(&format!(
                        "[areply] t={} r-defer req={request_id} pending={}",
                        automation_trace_epoch_ms(),
                        self.pending_automation_replies.len() + 1
                    ));
                }
                self.pending_automation_replies
                    .push(PendingAutomationReply { request_id, rx, kind });
                Ok(AutomationResult::Empty)
            }
        }
    }

    /// t2-pb1 fix#15：补答挂起的自动化 Evaluate（主循环每轮调用，空表零开销）。
    /// 通道关闭（导航复位清队 fix#11 / worker 退出）→ 回脚本异常面错误，与同步路径
    /// 一致；playwright 对已放弃上下文的响应按 request_id 丢弃。
    pub(super) fn poll_deferred_automation_replies(&mut self) -> Result<(), String> {
        if self.pending_automation_replies.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pending_automation_replies);
        let mut still_pending = Vec::new();
        // t-p106 诊断：poll 交付点仪拍——try_recv 结果逐条落盘（Empty 不记，避免 16ms
        // tick 刷屏）；still_pending 归还时记 pending_left。
        let trace_poll = automation_trace_enabled();
        for reply in pending {
            let mut result: Result<AutomationResult, AutomationError> = match reply.rx.try_recv() {
                // t-p106：ScriptValue/Retaining 映射抽为 [`eval_value_to_result`]，与
                // 页面脚本阶段中交付（[`deliver_ready_deferred_replies`]）共用。
                Ok(Ok(value)) => match reply.kind {
                    DeferredEvalKind::ScriptValue | DeferredEvalKind::Retaining => {
                        eval_value_to_result(&reply.kind, &value)
                    }
                    // fix#15b：句柄操作补答后按状态机收尾——pending 信号续跑 awaitPromise
                    // 落定循环（此时 worker 已空闲，循环内每轮执行不再排长臂之后）。
                    DeferredEvalKind::HandleOp { group, return_by_value } => {
                        match parse_handle_operation_envelope(&value) {
                            Ok(HandleOutcome::Result(result)) => Ok(result),
                            Ok(HandleOutcome::Pending) => self.await_pending_operation(&group, return_by_value),
                            Err(e) => Err(e),
                        }
                    }
                },
                Ok(Err(message)) => Err(automation_error(AutomationErrorCode::JavascriptError, message)),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    still_pending.push(reply);
                    continue;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(automation_error(
                    AutomationErrorCode::JavascriptError,
                    "deferred evaluate cancelled (navigation reset or worker exit)",
                )),
            };
            // t-p106 诊断：交付点结果（ok/err/disc）——回复到达 poll 的时刻即交付时刻。
            if trace_poll {
                let outcome = if result.is_ok() { "ok" } else { "err" };
                automation_trace_line(&format!(
                    "[areply] t={} r-poll req={} outcome={outcome}",
                    automation_trace_epoch_ms(),
                    reply.request_id
                ));
            }
            // t2-pb1 F4（首轮缺陷审查 2026-10-02）：挂起求值补答的同步尾。挂起路径
            // 此前不 apply_recorded_mutations——脚本晚至执行产生的 DOM mutation 滞留
            // worker 队列，被下一脚本/交互入口的 clear() 静默丢弃（数据丢失类：宿主
            // 侧渲染永久缺该变更，长臂饱和期是常态窗口）。补答成功即按同步路径同款
            // 收尾（焦点/缓存 HTML 回读、console 与 document.write drain、按需发布）。
            if result.is_ok() {
                // T68 返修（缺陷首轮 D4）：补答后的 mutation 渲染与其余产帧路径
                // 同点包 measure ctx（此前仅同步臂包裹）。
                let font_loader = self.font_loader.duplicate();
                let font_id = self.font_id;
                let changed = {
                    let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
                    let mut context = super::page_scripts::PageScriptContext {
                        html: &mut self.cached_html,
                        url: &current_url,
                        js_worker: &self.js_worker,
                        webview: self.webview.as_mut(),
                    };
                    let html_snapshot = context.html.clone();
                    super::text_metrics::with_measure_ctx_opt(&font_loader, font_id, || {
                        super::page_scripts::apply_recorded_mutations(&mut context, &html_snapshot).is_some()
                    })
                };
                if let Err(e) = self.automation_eval_sync_tail(changed) {
                    result = Err(e);
                }
            }
            // t2-pb1 F8①：单条补答发送失败（IPC 断连）不中断循环——`?` 早退会跳过
            // 末尾的 still_pending 归还，剩余挂起 reply 整体丢失。失败仅告警续跑。
            if let Err(e) = self.send_regular_with_id(
                reply.request_id,
                IpcMessageKind::AutomationResponse(AutomationResponse {
                    navigation_epoch: self.navigation_epoch,
                    document_generation: self.document_generation,
                    result,
                }),
            ) {
                tracing::warn!("deferred automation reply send failed: {e}");
            }
        }
        self.pending_automation_replies = still_pending;
        // t-p106 诊断：本轮 poll 收尾——仍挂起数（0 = 表空，下次 tick 短路）。
        if trace_poll {
            automation_trace_line(&format!(
                "[areply] t={} r-poll-done pending_left={}",
                automation_trace_epoch_ms(),
                self.pending_automation_replies.len()
            ));
        }
        Ok(())
    }

    fn execute_automation_operation(
        &mut self,
        request_id: u64,
        operation: AutomationOperation,
    ) -> Result<AutomationResult, AutomationError> {
        match operation {
            AutomationOperation::FindElement { using: _, value } => {
                if value.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element locator must not be empty",
                    ));
                }
                let handle = self
                    .webview
                    .as_ref()
                    .and_then(|webview| webview.page_node_handle_for_selector(&value))
                    .ok_or_else(|| automation_error(AutomationErrorCode::NoSuchElement, "element not found"))?;
                Ok(AutomationResult::Element(Some(self.automation_element_ref(handle))))
            }
            AutomationOperation::FindElements { using: _, value } => {
                if value.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element locator must not be empty",
                    ));
                }
                // https://w3c.github.io/webdriver/#find-elements — 无匹配返回空列表而非错误。
                let handles = self
                    .webview
                    .as_ref()
                    .map(|webview| webview.page_node_handles_for_selector(&value))
                    .unwrap_or_default();
                let references = handles
                    .into_iter()
                    .map(|handle| self.automation_element_ref(handle))
                    .collect();
                Ok(AutomationResult::Elements(references))
            }
            AutomationOperation::ElementClick { element } => {
                let selector = self.selector_for_automation_element(element)?;
                self.automation_click(&selector).map_err(internal_error)?;
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::SendKeys { element, keys } => {
                let selector = self.selector_for_automation_element(element)?;
                if self.interaction.focus_owner() != Some(selector.as_str()) {
                    self.blur_focused().map_err(internal_error)?;
                    self.focus_target(&selector).map_err(internal_error)?;
                }
                for key in keys {
                    self.automation_send_key(key).map_err(internal_error)?;
                }
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::ElementState { element, query } => {
                // https://w3c.github.io/webdriver/#element-state — 状态读经唯一选择器在
                // 页面脚本上下文求值，与渲染管线同一 live document。
                let selector = self.selector_for_automation_element(element)?;
                let script = element_state_script(&selector, &query);
                self.execute_state_script(script)
            }
            AutomationOperation::ElementClear { element } => {
                // https://w3c.github.io/webdriver/#element-clear — 可编辑元素置空 value，
                // 可勾选元素保持勾选语义不变（仅清文本类）。
                let selector = self.selector_for_automation_element(element)?;
                if self.interaction.focus_owner() != Some(selector.as_str()) {
                    self.blur_focused().map_err(internal_error)?;
                    self.focus_target(&selector).map_err(internal_error)?;
                }
                let script = format!(
                    "(function(){{var el=document.querySelector({selector:?});\
                     if(!el)return JSON.stringify({{ok:false}});\
                     var tag=el.tagName.toLowerCase();\
                     if(tag==='textarea'||(tag==='input'&&el.type!=='checkbox'&&el.type!=='radio'&&el.type!=='file')){{\
                     el.value='';el.dispatchEvent(new Event('input',{{bubbles:true}}));}}\
                     return JSON.stringify({{ok:true}});}})()"
                );
                let value = self.run_page_context_script(&script)?;
                let parsed = automation_value_from_script(&value);
                let AutomationValue::Object(entries) = parsed else {
                    return Err(internal_error("unexpected clear result".into()));
                };
                let ok = entries
                    .iter()
                    .any(|(k, v)| k == "ok" && *v == AutomationValue::Bool(true));
                if !ok {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element is not clearable",
                    ));
                }
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::GetActiveElement => {
                let element = self
                    .interaction
                    .focus_owner()
                    .and_then(|selector| {
                        self.webview
                            .as_ref()
                            .and_then(|webview| webview.page_node_handle_for_selector(selector))
                    })
                    .map(|handle| self.automation_element_ref(handle));
                Ok(AutomationResult::Element(element))
            }
            AutomationOperation::ExecuteScript { script, arguments } => {
                if script.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "script must not be empty",
                    ));
                }
                let arguments =
                    serde_json::Value::Array(arguments.iter().map(automation_value_to_json).collect::<Vec<_>>());
                let source = format!(
                    "(function(){{var __zw_value=(function(){{{script}\n}}).apply(null,{arguments});\
                     return JSON.stringify({{defined:typeof __zw_value!=='undefined',value:__zw_value}});}})()"
                );
                // t2-pb1 fix#15：Evaluate 家族可挂起（worker 长臂时主循环 2s 放行）。
                self.execute_eval_deferrable(request_id, source, DeferredEvalKind::ScriptValue)
            }
            AutomationOperation::EvaluateRetaining {
                script,
                group,
                return_by_value,
            } => {
                if script.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "script must not be empty",
                    ));
                }
                let group = group.unwrap_or_else(|| DEFAULT_OBJECT_GROUP.to_string());
                // 对象结果保留进注册表（句柄随文档换代经 JS context 重建自然失效）。
                let source = evaluate_retaining_script(&script, &group, return_by_value);
                // t2-pb1 fix#15：Evaluate 家族可挂起（pending 信号的拒绝语义移入
                // execute_eval_deferrable 的补答路径，与同步路径同文案）。
                self.execute_eval_deferrable(request_id, source, DeferredEvalKind::Retaining)
            }
            AutomationOperation::CallFunctionOnHandle {
                handle,
                function_declaration,
                arguments,
                return_by_value,
                await_promise,
                group,
            } => {
                let group = group.unwrap_or_else(|| DEFAULT_OBJECT_GROUP.to_string());
                let arguments = serde_json::Value::Array(arguments.iter().map(automation_value_to_json).collect());
                let source = call_function_on_handle_script(
                    handle,
                    &function_declaration,
                    &arguments.to_string(),
                    return_by_value,
                    await_promise,
                    &group,
                );
                // t2-pb1 fix#15b：playwright 的 title()/evaluate() 都走此操作，可挂起。
                match self.run_handle_operation_deferrable(request_id, &source, &group, return_by_value)? {
                    AutomationHandleFlow::Deferred => Ok(AutomationResult::Empty),
                    AutomationHandleFlow::Done(HandleOutcome::Pending) => {
                        self.await_pending_operation(&group, return_by_value)
                    }
                    AutomationHandleFlow::Done(HandleOutcome::Result(result)) => Ok(result),
                }
            }
            AutomationOperation::ReleaseHandle { handle } => {
                match self.run_handle_operation_deferrable(
                    request_id,
                    &release_handle_script(handle),
                    DEFAULT_OBJECT_GROUP,
                    true,
                )? {
                    AutomationHandleFlow::Deferred | AutomationHandleFlow::Done(_) => Ok(AutomationResult::Empty),
                }
            }
            AutomationOperation::ReleaseObjectGroup { group } => {
                match self.run_handle_operation_deferrable(
                    request_id,
                    &release_object_group_script(&group),
                    DEFAULT_OBJECT_GROUP,
                    true,
                )? {
                    AutomationHandleFlow::Deferred | AutomationHandleFlow::Done(_) => Ok(AutomationResult::Empty),
                }
            }
            AutomationOperation::Unsupported { name } => Err(automation_error(
                AutomationErrorCode::UnsupportedOperation,
                format!("unsupported automation operation: {name}"),
            )),
        }
    }

    fn automation_element_ref(&self, node_handle: u64) -> AutomationElementRef {
        AutomationElementRef {
            navigation_epoch: self.navigation_epoch,
            document_generation: self.document_generation,
            node_handle,
        }
    }

    /// 脚本执行后的尾随同步（同步与 fix#15 可挂起路径共用）：焦点/缓存 HTML 回读、
    /// console 与 document.write drain、DOM 变更发布。
    fn automation_eval_sync_tail(&mut self, changed: bool) -> Result<(), AutomationError> {
        self.sync_focus_from_js();
        self.sync_cached_html_from_webview();
        // S11：脚本执行产生的 console 输出先于 AutomationResponse 转发（headless 在
        // 自动化往返中消费并入同一命令的事件排空——晚了要等下一条命令才可见）。
        self.tick_console_log_drain();
        // S16：document.write 落定信号同尾 drain（PW setContent 在 evaluate 返回前
        // 需要 console tag 与 load 生命周期重发均在途）。
        self.tick_document_write_drain();
        if changed {
            self.publish_webview(None, true).map_err(internal_error)?;
        }
        Ok(())
    }

    /// 在页面脚本上下文执行 `source`，返回脚本 stdout（JSON 包络字符串）并同步 DOM 变更。
    fn run_page_context_script(&mut self, source: &str) -> Result<String, AutomationError> {
        let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let (value, changed) = {
            let mut context = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
                webview: self.webview.as_mut(),
            };
            super::page_scripts::execute_automation_script(&mut context, source)
                .map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?
        };
        self.automation_eval_sync_tail(changed)?;
        Ok(value)
    }

    /// 句柄操作的公共尾：执行生成脚本、解包络（含 pending/句柄/错误信号）。
    /// 同步形态仅保留给 awaitPromise 落定循环内部（worker 刚执行完我方轮询脚本，
    /// 无长臂积压）。
    fn run_handle_operation(&mut self, source: &str) -> Result<HandleOutcome, AutomationError> {
        let value = self.run_page_context_script(source)?;
        parse_handle_operation_envelope(&value)
    }

    /// t2-pb1 fix#15b：[`Self::run_handle_operation`] 的可挂起形态。有界等待内完成 →
    /// 与同步路径一致（尾随同步 + 包络状态机）；超时 → reply 通道连同句柄操作延续
    /// 信息（group/return_by_value）入挂起表，主循环不因 worker 长臂阻塞。bilibili
    /// 实测：30s timer 臂期间 playwright `page.title()`（CallFunctionOnHandle）曾占死
    /// 主循环 29s，后续导航 IPC 饿死到 bwfl 必炸。返回 Empty 占位同 Evaluate 家族
    /// （被 [`Self::handle_automation_request`] 的长度差检测拦截）。
    fn run_handle_operation_deferrable(
        &mut self,
        request_id: u64,
        source: &str,
        group: &str,
        return_by_value: bool,
    ) -> Result<AutomationHandleFlow, AutomationError> {
        let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let flow = {
            let mut context = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
                webview: self.webview.as_mut(),
            };
            super::page_scripts::execute_automation_script_deferrable(&mut context, source)
        };
        match flow {
            super::page_scripts::AutomationEvalOutcome::Done(outcome) => {
                let (value, changed) =
                    outcome.map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?;
                self.automation_eval_sync_tail(changed)?;
                Ok(AutomationHandleFlow::Done(parse_handle_operation_envelope(&value)?))
            }
            super::page_scripts::AutomationEvalOutcome::Deferred(rx) => {
                self.pending_automation_replies.push(PendingAutomationReply {
                    request_id,
                    rx,
                    kind: DeferredEvalKind::HandleOp {
                        group: group.to_string(),
                        return_by_value,
                    },
                });
                Ok(AutomationHandleFlow::Deferred)
            }
        }
    }

    /// `awaitPromise` 落定循环：`__zwAutomationAwait` 由首个 execute 投递，此后每轮
    /// 泵宿主 timer 回调 + 读 done 哨兵（execute 边界 drain microtask，见
    /// script-sandbox `perform_microtask_checkpoint` / QuickJS job queue drain），
    /// 直到落定或有界超时。落定值留在注册表脚本侧读取（保对象本体，不走 JSON 往返）。
    fn await_pending_operation(
        &mut self,
        group: &str,
        return_by_value: bool,
    ) -> Result<AutomationResult, AutomationError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(AWAIT_PROMISE_TIMEOUT_SECS);
        loop {
            {
                let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
                let mut context = PageScriptContext {
                    html: &mut self.cached_html,
                    url: &current_url,
                    js_worker: &self.js_worker,
                    webview: self.webview.as_mut(),
                };
                // await 期间 timer 回调的 DOM 变更照常提交到活 DOM。
                super::page_scripts::drain_pending_dom_mutations(&mut context);
            }
            let state = self
                .js_worker
                // t2-pb1 fix#9：await 轮询同走优先通道（宿主发起，不排页面回调积压之后）。
                .execute_script_direct_priority(AWAIT_POLL_SCRIPT)
                .map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?;
            let settled = serde_json::from_str::<serde_json::Value>(&state)
                .ok()
                .and_then(|value| value.get("done").and_then(serde_json::Value::as_bool));
            if settled == Some(true) {
                break;
            }
            if std::time::Instant::now() >= deadline {
                return Err(automation_error(AutomationErrorCode::Timeout, "awaitPromise timed out"));
            }
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        let source = pending_settled_script(group, return_by_value);
        match self.run_handle_operation(&source)? {
            HandleOutcome::Pending => Err(internal_error(
                "await sentinel reported done but tail read pending".into(),
            )),
            HandleOutcome::Result(result) => Ok(result),
        }
    }

    /// 元素状态查询的公共尾：执行生成的脚本并解 JSON 包络。
    fn execute_state_script(&mut self, script: String) -> Result<AutomationResult, AutomationError> {
        let value = self.run_page_context_script(&script)?;
        Ok(AutomationResult::Value(automation_value_from_script(&value)))
    }

    fn selector_for_automation_element(&self, element: AutomationElementRef) -> Result<String, AutomationError> {
        if element.navigation_epoch != self.navigation_epoch || element.document_generation != self.document_generation
        {
            return Err(automation_error(
                AutomationErrorCode::StaleElementReference,
                "element belongs to an old document",
            ));
        }
        self.webview
            .as_ref()
            .and_then(|webview| webview.selector_for_page_node_handle(element.node_handle))
            .ok_or_else(|| {
                automation_error(
                    AutomationErrorCode::StaleElementReference,
                    "element no longer exists in the live document",
                )
            })
    }

    fn automation_click(&mut self, selector: &str) -> Result<(), String> {
        // https://w3c.github.io/webdriver/#element-click
        let (click, checked_handled) = self.dispatch_checked_click(selector.to_string())?;
        if self.interaction.focus_owner() != Some(selector) {
            self.blur_focused()?;
            self.focus_target(selector)?;
        }
        if click.default_allowed && !checked_handled && !self.activate_form_control_at(selector)? {
            self.activate_label_at(selector)?;
        }
        Ok(())
    }

    fn automation_send_key(&mut self, key: AutomationKey) -> Result<(), String> {
        // https://w3c.github.io/webdriver/#element-send-keys
        match key {
            AutomationKey::Text(text) => {
                for character in text.chars() {
                    let value = character.to_string();
                    // 字符键携带字符值（character value）——renderer 据此派 keypress 并插入。
                    self.automation_key_event(&value, "Unidentified", false, Some(&value), KeyboardEventType::Down)?;
                    self.automation_key_event(&value, "Unidentified", false, None, KeyboardEventType::Up)?;
                }
            }
            AutomationKey::Tab => {
                self.automation_key_event("Tab", "Tab", false, None, KeyboardEventType::Down)?;
                self.automation_key_event("Tab", "Tab", false, None, KeyboardEventType::Up)?;
            }
            AutomationKey::ShiftTab => {
                self.automation_key_event("Tab", "Tab", true, None, KeyboardEventType::Down)?;
                self.automation_key_event("Tab", "Tab", true, None, KeyboardEventType::Up)?;
            }
            AutomationKey::Backspace => {
                self.automation_key_event("Backspace", "Backspace", false, None, KeyboardEventType::Down)?;
                self.automation_key_event("Backspace", "Backspace", false, None, KeyboardEventType::Up)?;
            }
            AutomationKey::Enter => {
                self.automation_key_event("Enter", "Enter", false, None, KeyboardEventType::Down)?;
                self.automation_key_event("Enter", "Enter", false, None, KeyboardEventType::Up)?;
            }
        }
        Ok(())
    }

    fn automation_key_event(
        &mut self,
        key: &str,
        code: &str,
        shift: bool,
        text: Option<&str>,
        event_type: KeyboardEventType,
    ) -> Result<(), String> {
        self.handle_keyboard_event(KeyboardEventParams {
            key: key.to_string(),
            code: code.to_string(),
            text: text.map(|t| t.to_string()),
            ctrl: false,
            shift,
            alt: false,
            meta: false,
            event_type,
        })
    }
}

/// 生成元素状态查询脚本：按唯一选择器定位元素，按 query 项读取状态，返 JSON 包络。
///
/// 选择器以 JS 字符串字面量（`{:?}`）内插，脚本环境与渲染管线同一 live document。
/// 找不到元素 → `{"ok":false}`（上游层把引用过期/缺失分开报）。
fn element_state_script(selector: &str, query: &AutomationStateQuery) -> String {
    let find = format!("var el=document.querySelector({selector:?});if(!el)return JSON.stringify({{ok:false}});");
    let body = match query {
        // W3C Get Element Text：渲染文本近似 = textContent（可见性过滤未实现，FIXME）。
        AutomationStateQuery::Text => {
            format!("{find}return JSON.stringify({{ok:true,value:el.textContent==null?null:String(el.textContent)}});")
        }
        // W3C Get Element Rect：shim `getBoundingClientRect` 真值（RectBridge 注册时）。
        AutomationStateQuery::Rect => format!(
            "{find}var r=el.getBoundingClientRect();\
             return JSON.stringify({{ok:true,value:{{x:r.x,y:r.y,width:r.width,height:r.height}}}});"
        ),
        // W3C Is Element Enabled：非表单元素恒 true（无 disabled 语义）。
        AutomationStateQuery::Enabled => format!(
            "{find}var disabled=false;\
             if(el.disabled===true)disabled=true;\
             else if(typeof el.hasAttribute==='function'&&el.hasAttribute('disabled'))disabled=true;\
             return JSON.stringify({{ok:true,value:!disabled}});"
        ),
        // W3C Is Element Selected：option 的 selected / checkbox·radio 的 checkedness。
        AutomationStateQuery::Selected => format!(
            "{find}var selected=false;\
             if(el.tagName.toLowerCase()==='option')selected=!!el.selected;\
             else if(el.type==='checkbox'||el.type==='radio')selected=!!el.checked;\
             return JSON.stringify({{ok:true,value:selected}});"
        ),
        // W3C Get Element Attribute：内容属性值（不存在 → null）。
        AutomationStateQuery::Attribute(name) => format!(
            "{find}var v=el.getAttribute({name:?});\
             return JSON.stringify({{ok:true,value:v===null?null:String(v)}});"
        ),
        // W3C Get Element Property：DOM 属性直读（undefined → null）。
        AutomationStateQuery::Property(name) => format!(
            "{find}var v=el[{name:?}];\
             return JSON.stringify({{ok:true,value:v===undefined?null:v}});"
        ),
        // W3C Get Element CSS Value：计算样式（shim getComputedStyle → host 真值）。
        AutomationStateQuery::CssValue(name) => format!(
            "{find}var cs=getComputedStyle(el);var v=cs.getPropertyValue({name:?});\
             return JSON.stringify({{ok:true,value:String(v)}});"
        ),
    };
    format!("(function(){{{body}}})()")
}

fn automation_error(code: AutomationErrorCode, message: impl Into<String>) -> AutomationError {
    AutomationError {
        code,
        message: message.into(),
    }
}

fn internal_error(message: String) -> AutomationError {
    automation_error(AutomationErrorCode::Internal, message)
}

fn automation_value_to_json(value: &AutomationValue) -> serde_json::Value {
    match value {
        AutomationValue::Null => serde_json::Value::Null,
        AutomationValue::Bool(value) => serde_json::Value::Bool(*value),
        AutomationValue::Number(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        AutomationValue::String(value) => serde_json::Value::String(value.clone()),
        AutomationValue::Array(values) => {
            serde_json::Value::Array(values.iter().map(automation_value_to_json).collect())
        }
        AutomationValue::Object(entries) => serde_json::Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), automation_value_to_json(value)))
                .collect(),
        ),
        // 句柄参数以标记对象下传，页面脚本在实参列表顶层还原为保留对象
        //（CDP `Runtime.callFunctionOn` 的 `arguments[].objectId` 只出现在实参顶层）。
        AutomationValue::Handle(handle) => serde_json::json!({ "__zwHandleRef": handle.id }),
    }
}

/// 未显式指定 `objectGroup` 时句柄落入的默认组。
const DEFAULT_OBJECT_GROUP: &str = "zw-automation";
/// `awaitPromise` 落定循环的有界超时（秒）——headless 侧自动化 IPC 超时为 10s，留余量。
const AWAIT_PROMISE_TIMEOUT_SECS: u64 = 8;

/// 落定哨兵：只读 done 标志（落定值本体留在 `__zwAutomationAwait`，由尾部脚本按
/// returnByValue 语义取用）。
const AWAIT_POLL_SCRIPT: &str =
    "JSON.stringify(globalThis.__zwAutomationAwait?{done:globalThis.__zwAutomationAwait.done}:{done:null})";

/// 句柄注册表 bootstrap 片段——每个操作脚本内联一份，幂等（首次执行时初始化）。
/// 注册表活在页面 JS context 全局对象上：导航换代 context 销毁重建
///（`reset_document_state` → `sandbox.reset_context`）→ 全部句柄自然失效，
/// 与 CDP「execution context destroyed」语义一致。
const HANDLE_REGISTRY_JS: &str = r#"
var __zw = globalThis.__zwAutomationHandles = globalThis.__zwAutomationHandles || (function () {
  var r = { next: 1, byId: Object.create(null), groups: Object.create(null) };
  r.retain = function (v, group) {
    if (r.next > 65536) throw new Error('automation handle registry full');
    var id = r.next;
    r.next = r.next + 1;
    r.byId[id] = v;
    (r.groups[group] = r.groups[group] || Object.create(null))[id] = true;
    return id;
  };
  r.tail = function (v, group, byValue) {
    var t = typeof v;
    if (byValue || v === null || v === undefined || t === 'boolean' || t === 'number' || t === 'string') {
      if (v === undefined) return JSON.stringify({ defined: false });
      return JSON.stringify({ defined: true, value: v });
    }
    // node 性随句柄上报：shim DOM 节点具 nodeType（CDP subtype:"node" 判定面）。
    var isNode = v && typeof v === 'object' && typeof v.nodeType === 'number' && v.nodeType >= 1 && v.nodeType <= 12;
    return JSON.stringify({ defined: true, handle: r.retain(v, group), node: !!isNode });
  };
  return r;
})();"#;

/// [`AutomationOperation::EvaluateRetaining`] 脚本：执行脚本并按
/// returnByValue 语义处理结果（false：对象 → 注册表句柄；true：对象按值深序列化）。
/// 脚本语义（Chromium `Runtime.evaluate`：按 **Script 语句序列**编译，completion value
/// 为结果）——t6（siteopt qq-20261016-r1）：旧实现 `(expression)` 包装把语句序列变非法
/// 表达式（`window.scrollBy(0,800); 1` → `(window.scrollBy(0,800); 1)` 抛
/// "SyntaxError: Unexpected token ';'"，qq.com journey 每 evaluate 一条 pageerror，
/// trial/t6/journey/baseline-r1-9817.json 实证 ×18/轮）。改经**间接 eval** 执行：语句序列
/// 合法、completion value 即求值结果、`var`/`function` 声明落全局（Chrome global-scope 语义）；
/// `__zwRealEval||eval` 走原生 eval（eval 门禁包装替换 globalThis.eval 时，同 script_gen
/// M2-s6 先例）。
fn evaluate_retaining_script(script: &str, group: &str, return_by_value: bool) -> String {
    let by_value = if return_by_value { "true" } else { "false" };
    let script_literal = format!("'{}'", zero_engine::escape_js_string(script));
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}\nvar __zwV = (0,globalThis.__zwRealEval||eval)({script_literal});\nreturn __zw.tail(__zwV, {group:?}, {by_value});}})()"
    )
}

/// [`AutomationOperation::CallFunctionOnHandle`] 脚本：句柄对象为 `this` 调用函数，
/// 顶层句柄实参还原为保留对象；`awaitPromise` 时 thenable 结果转落定哨兵。
#[allow(clippy::too_many_arguments)]
fn call_function_on_handle_script(
    handle: u64,
    function_declaration: &str,
    arguments_json: &str,
    return_by_value: bool,
    await_promise: bool,
    group: &str,
) -> String {
    let await_open = if await_promise { "true" } else { "false" };
    let by_value = if return_by_value { "true" } else { "false" };
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwT = __zw.byId[{handle}];
if (!__zwT) return JSON.stringify({{defined:true,zwMiss:true}});
var __zwF = ({function_declaration});
var __zwA = {arguments_json};
for (var __zwI = 0; __zwI < __zwA.length; __zwI++) {{
  var __zwM = __zwA[__zwI];
  // 仅对象形态的标记（{{__zwHandleRef:n}}）还原为保留对象；裸 falsy 实参
  // （false/0/''）不得进入查找——`falsy && x` 会被误判为标记（实测根因）。
  if (__zwM && typeof __zwM === 'object' && __zwM.__zwHandleRef !== undefined) {{
    __zwA[__zwI] = __zw.byId[__zwM.__zwHandleRef];
    if (!__zwA[__zwI]) return JSON.stringify({{defined:true,zwMiss:true,dbgWanted:__zwM.__zwHandleRef}});
  }}
}}
var __zwV;
try {{ __zwV = __zwF.apply(__zwT, __zwA); }}
catch (__zwE) {{ return JSON.stringify({{defined:true,zwThrow:String(__zwE && __zwE.message || __zwE)}}); }}
if ({await_open} && __zwV && typeof __zwV.then === 'function') {{
  globalThis.__zwAutomationAwait = {{ done: false }};
  Promise.resolve(__zwV).then(
    function (v) {{ globalThis.__zwAutomationAwait = {{ done: true, ok: true, value: v }}; }},
    function (e) {{ globalThis.__zwAutomationAwait = {{ done: true, ok: false, error: String(e && e.message || e) }}; }}
  );
  return JSON.stringify({{ defined: true, pending: true }});
}}
return __zw.tail(__zwV, {group:?}, {by_value});}})()"
    )
}

/// `awaitPromise` 落定后的收尾脚本：从 `__zwAutomationAwait` 取落定值本体并套用
/// returnByValue 语义（保对象本体，不走 JSON 往返）。
fn pending_settled_script(group: &str, return_by_value: bool) -> String {
    let by_value = if return_by_value { "true" } else { "false" };
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwA = globalThis.__zwAutomationAwait;
globalThis.__zwAutomationAwait = undefined;
if (!__zwA || !__zwA.done) return JSON.stringify({{defined:true,pending:true}});
if (!__zwA.ok) return JSON.stringify({{defined:true,zwThrow:String(__zwA.error)}});
return __zw.tail(__zwA.value, {group:?}, {by_value});}})()"
    )
}

/// [`AutomationOperation::ReleaseHandle`] 脚本：从全部组与注册表移除句柄。
fn release_handle_script(handle: u64) -> String {
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
for (var __zwK in __zw.groups) delete __zw.groups[__zwK][{handle}];
delete __zw.byId[{handle}];
return JSON.stringify({{defined:true,value:true}});}})()"
    )
}

/// [`AutomationOperation::ReleaseObjectGroup`] 脚本：整组移除注册表句柄。
fn release_object_group_script(group: &str) -> String {
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwIds = __zw.groups[{group:?}];
if (__zwIds) for (var __zwId in __zwIds) delete __zw.byId[__zwId];
delete __zw.groups[{group:?}];
return JSON.stringify({{defined:true,value:true}});}})()"
    )
}

/// 句柄操作包络的解析结果：终值或 `awaitPromise` 落定哨兵。
enum HandleOutcome {
    Result(AutomationResult),
    Pending,
}

/// 解包络：`zwThrow`/`zwMiss` → 脚本错误；`handle` → 句柄引用；`pending` → 落定哨兵；
/// 其余走既有包络解析（`{defined,value}`）。
fn parse_handle_operation_envelope(value: &str) -> Result<HandleOutcome, AutomationError> {
    let envelope = match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Object(envelope)) if envelope.contains_key("defined") => envelope,
        _ => {
            return Ok(HandleOutcome::Result(AutomationResult::Value(
                automation_value_from_script(value),
            )));
        }
    };
    if envelope.get("zwThrow").and_then(serde_json::Value::as_str).is_some() {
        let message = envelope
            .get("zwThrow")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        return Err(automation_error(
            AutomationErrorCode::JavascriptError,
            message.to_string(),
        ));
    }
    if envelope.get("zwMiss").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(automation_error(
            AutomationErrorCode::JavascriptError,
            "object handle is unknown or has been released".to_string(),
        ));
    }
    if envelope.get("pending").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(HandleOutcome::Pending);
    }
    if let Some(handle) = envelope.get("handle").and_then(serde_json::Value::as_u64) {
        let node = envelope
            .get("node")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        return Ok(HandleOutcome::Result(AutomationResult::Value(AutomationValue::Handle(
            zero_protocol::message::AutomationHandleRef { id: handle, node },
        ))));
    }
    Ok(HandleOutcome::Result(AutomationResult::Value(
        automation_value_from_script(value),
    )))
}

fn automation_value_from_script(value: &str) -> AutomationValue {
    if value == "undefined" || value == "null" {
        return AutomationValue::Null;
    }
    match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Object(mut envelope)) if envelope.contains_key("defined") => {
            if envelope.get("defined").and_then(serde_json::Value::as_bool) != Some(true) {
                AutomationValue::Null
            } else {
                automation_value_from_json(envelope.remove("value").unwrap_or(serde_json::Value::Null))
            }
        }
        Ok(value) => automation_value_from_json(value),
        Err(_) => AutomationValue::String(value.to_string()),
    }
}

fn automation_value_from_json(value: serde_json::Value) -> AutomationValue {
    match value {
        serde_json::Value::Null => AutomationValue::Null,
        serde_json::Value::Bool(value) => AutomationValue::Bool(value),
        serde_json::Value::Number(value) => AutomationValue::Number(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => AutomationValue::String(value),
        serde_json::Value::Array(values) => {
            AutomationValue::Array(values.into_iter().map(automation_value_from_json).collect())
        }
        serde_json::Value::Object(entries) => AutomationValue::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_from_json(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use zero_protocol::message::FramePublishMode;

    use super::*;

    fn runtime() -> RendererRuntime {
        let html = "<html><body><input id=\"name\"><input id=\"check\" type=\"checkbox\"></body></html>";
        let url = "https://zero.test/automation";
        let (_tx, rx) = mpsc::channel();
        let mut runtime = RendererRuntime::with_io(301, FramePublishMode::Legacy, Box::new(std::io::sink()), rx);
        runtime.compositor_publish = None;
        runtime.stub_network = true;
        runtime.current_url = Some(url.into());
        runtime.cached_html = html.into();
        runtime.navigation_epoch = 9;
        runtime.document_generation = 1;
        runtime.webview.as_mut().unwrap().prepare_document_state(url);
        runtime.webview.as_mut().unwrap().load_html(html, None);
        {
            let mut context = PageScriptContext {
                html: &mut runtime.cached_html,
                url,
                js_worker: &runtime.js_worker,
                webview: runtime.webview.as_mut(),
            };
            super::super::page_scripts::run_page_scripts(&mut context, true, |_url| {
                Err::<String, String>("no fetch".into())
            });
        }
        runtime
    }

    fn find(runtime: &mut RendererRuntime, selector: &str) -> AutomationElementRef {
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::FindElement {
                        using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                        value: selector.into(),
                    },
                },
            )
            .expect("find element");
        let AutomationResult::Element(Some(element)) = result else {
            panic!("expected element");
        };
        element
    }

    #[test]
    fn live_automation_updates_form_and_rejects_stale_reference() {
        let mut runtime = runtime();
        let name = find(&mut runtime, "#name");
        let check = find(&mut runtime, "#check");

        runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::SendKeys {
                        element: name,
                        keys: vec![AutomationKey::Text("Aé".into())],
                    },
                },
            )
            .expect("send unicode keys");
        assert_eq!(
            runtime
                .execute_automation_request(
                    1,
                    AutomationRequest {
                        operation: AutomationOperation::ExecuteScript {
                            script: "return document.getElementById('name').value;".into(),
                            arguments: Vec::new(),
                        },
                    }
                )
                .expect("read live input"),
            AutomationResult::Value(AutomationValue::String("Aé".into()))
        );

        runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::ElementClick { element: check },
                },
            )
            .expect("click checkbox");
        assert_eq!(
            runtime
                .execute_automation_request(
                    1,
                    AutomationRequest {
                        operation: AutomationOperation::ExecuteScript {
                            script: "return document.getElementById('check').checked;".into(),
                            arguments: Vec::new(),
                        },
                    }
                )
                .expect("read live checkedness"),
            AutomationResult::Value(AutomationValue::Bool(true))
        );
        assert_eq!(
            runtime
                .execute_automation_request(
                    1,
                    AutomationRequest {
                        operation: AutomationOperation::GetActiveElement,
                    }
                )
                .expect("active element"),
            AutomationResult::Element(Some(check))
        );

        let stale = AutomationElementRef {
            document_generation: check.document_generation.saturating_sub(1),
            ..check
        };
        let error = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::ElementClick { element: stale },
                },
            )
            .expect_err("stale click must fail");
        assert_eq!(error.code, AutomationErrorCode::StaleElementReference);
    }

    #[test]
    fn find_elements_returns_all_matches_in_document_order() {
        let mut runtime = runtime();
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::FindElements {
                        using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                        value: "input".into(),
                    },
                },
            )
            .expect("find elements");
        let AutomationResult::Elements(references) = result else {
            panic!("expected elements list");
        };
        assert_eq!(references.len(), 2, "两个 input 都应命中");

        // 空匹配返回空列表（W3C：非错误）。
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::FindElements {
                        using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                        value: "#missing".into(),
                    },
                },
            )
            .expect("find elements no match");
        let AutomationResult::Elements(references) = result else {
            panic!("expected elements list");
        };
        assert!(references.is_empty(), "无匹配应返回空列表");
    }

    #[test]
    fn unsupported_automation_operation_is_explicit() {
        let mut runtime = runtime();
        let error = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::Unsupported {
                        name: "test_driver.set_permission".into(),
                    },
                },
            )
            .expect_err("unsupported operation");
        assert_eq!(error.code, AutomationErrorCode::UnsupportedOperation);
    }

    // ── objectId 句柄桥（CDP Runtime.evaluate returnByValue:false 语义）──

    /// t6（siteopt qq-20261016-r1）：`Runtime.evaluate` 按 **Script 语句序列**编译
    /// （Chrome 语义，completion value 为结果）。旧 `(expression)` 包装对语句序列抛
    /// "SyntaxError: Unexpected token ';'"（qq.com journey 每 evaluate 一条 pageerror，
    /// trial/t6/journey/baseline-r1-9817.json ×18/轮）。回归钉：多语句 + 尾分号 +
    /// completion value + `var` 全局可见四断言。
    #[test]
    fn evaluate_accepts_statement_sequences() {
        let mut runtime = runtime();
        // 语句序列 + 尾分号：旧包装 `(a(); b)` 编译失败，Chrome 返回 completion value 2。
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::EvaluateRetaining {
                        script: "globalThis.__zwT6Seq = 1; __zwT6Seq + 1;".into(),
                        group: None,
                        return_by_value: true,
                    },
                },
            )
            .expect("statement sequence must evaluate");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(2.0)));
        // `var` 声明落全局（indirect eval global-scope 语义，Chrome 对齐）：后续 evaluate 可见。
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::EvaluateRetaining {
                        script: "var __zwT6Var = 41; __zwT6Var + 1".into(),
                        group: None,
                        return_by_value: true,
                    },
                },
            )
            .expect("var declare evaluate");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(42.0)));
    }

    /// 保留对象返回句柄引用、原始类型按值返回。
    #[test]
    fn evaluate_retaining_splits_objects_and_primitives() {
        let mut runtime = runtime();
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::EvaluateRetaining {
                        script: "1 + 2".into(),
                        group: None,
                        return_by_value: false,
                    },
                },
            )
            .expect("retain primitive");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(3.0)));

        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::EvaluateRetaining {
                        script: "({a: 1, b: 21})".into(),
                        group: None,
                        return_by_value: false,
                    },
                },
            )
            .expect("retain object");
        let AutomationResult::Value(AutomationValue::Handle(_)) = result else {
            panic!("object result must be a handle, got {result:?}");
        };
    }

    /// 句柄对象作 `this` 调用函数；句柄实参还原为保留对象。
    #[test]
    fn call_function_on_handle_resolves_this_and_handle_arguments() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 20})");
        let argument = retain_object(&mut runtime, "({v: 1})");
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: target.id,
                        function_declaration: "(function (o) { return this.v + o.v; })".into(),
                        arguments: vec![AutomationValue::Handle(argument)],
                        return_by_value: true,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect("call on handle");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(21.0)));
    }

    /// returnByValue:false 的对象结果保留为新句柄（Playwright 句柄链）。
    #[test]
    fn call_function_on_handle_retains_object_results() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 5})");
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: target.id,
                        function_declaration: "(function () { return { nested: this.v * 2 }; })".into(),
                        arguments: vec![],
                        return_by_value: false,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect("retain call result");
        let AutomationResult::Value(AutomationValue::Handle(nested)) = result else {
            panic!("object result must be a handle, got {result:?}");
        };
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: nested.id,
                        function_declaration: "(function () { return this.nested + 1; })".into(),
                        arguments: vec![],
                        return_by_value: true,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect("read nested via handle");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(11.0)));
    }

    /// `awaitPromise`：已落定 Promise 与 setTimeout 异步落定都等待后返回。
    #[test]
    fn call_function_on_handle_awaits_promises() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 1})");
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: target.id,
                        function_declaration: "(function () { return Promise.resolve(7); })".into(),
                        arguments: vec![],
                        return_by_value: true,
                        await_promise: true,
                        group: None,
                    },
                },
            )
            .expect("await settled promise");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(7.0)));

        let result = runtime
            .execute_automation_request(1, AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: target.id,
                    function_declaration: "(function () { var self = this; return new Promise(function (r) { setTimeout(function () { r(self.v + 3); }, 30); }); })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: true,
                    group: None,
                },
            })
            .expect("await timer promise");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(4.0)));
    }

    /// releaseHandle 后句柄引用失效（CDP releaseObject 配对语义）。
    #[test]
    fn release_handle_invalidates_reference() {
        let mut runtime = runtime();
        let handle = retain_object(&mut runtime, "({v: 1})");
        runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::ReleaseHandle { handle: handle.id },
                },
            )
            .expect("release handle");
        let error = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: handle.id,
                        function_declaration: "(function () { return this.v; })".into(),
                        arguments: vec![],
                        return_by_value: true,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect_err("released handle must fail");
        assert_eq!(error.code, AutomationErrorCode::JavascriptError);
    }

    /// releaseObjectGroup 整组失效，组外句柄不受影响。
    #[test]
    fn release_object_group_scopes_to_group() {
        let mut runtime = runtime();
        let grouped = retain_object_in_group(&mut runtime, "({v: 1})", "gtest");
        let untouched = retain_object_in_group(&mut runtime, "({v: 2})", "other");
        runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::ReleaseObjectGroup { group: "gtest".into() },
                },
            )
            .expect("release group");
        let error = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: grouped.id,
                        function_declaration: "(function () { return this.v; })".into(),
                        arguments: vec![],
                        return_by_value: true,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect_err("grouped handle must be released");
        assert_eq!(error.code, AutomationErrorCode::JavascriptError);
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::CallFunctionOnHandle {
                        handle: untouched.id,
                        function_declaration: "(function () { return this.v; })".into(),
                        arguments: vec![],
                        return_by_value: true,
                        await_promise: false,
                        group: None,
                    },
                },
            )
            .expect("other group survives");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(2.0)));
    }

    fn retain_object(runtime: &mut RendererRuntime, script: &str) -> zero_protocol::message::AutomationHandleRef {
        retain_object_in_group(runtime, script, "zw-automation")
    }

    fn retain_object_in_group(
        runtime: &mut RendererRuntime,
        script: &str,
        group: &str,
    ) -> zero_protocol::message::AutomationHandleRef {
        let result = runtime
            .execute_automation_request(
                1,
                AutomationRequest {
                    operation: AutomationOperation::EvaluateRetaining {
                        script: script.into(),
                        group: Some(group.into()),
                        return_by_value: false,
                    },
                },
            )
            .expect("retain object");
        let AutomationResult::Value(AutomationValue::Handle(handle)) = result else {
            panic!("expected handle, got {result:?}");
        };
        handle
    }

    /// t2-pb1 返修 N1（定向复核缺口，2026-10-02）：F4 补答接线真实路径钉。挂起求值
    /// 晚至完成后，[`Self::poll_deferred_automation_replies`] 须在**真实补答路径**内
    /// apply mutation 并落进 `cached_html`——此前 page_scripts 层测试手工复刻了补答侧
    /// apply（不走 poll），revert automation.rs 的 poll 补答 hunk 时现存测试全绿
    ///（假绿面）；本测试闭合该缺口：移除 poll 内 apply+sync tail 即红。
    #[test]
    fn deferred_reply_poll_applies_mutations_via_real_path() {
        let mut rt = runtime();
        // 长臂占住 worker（15e8：debug ~6s，确保后续脚本 2s 有界等待稳定超时挂起；
        // 5e8 约 2.1s 与 USER_ACTION_SCRIPT_TIMEOUT 同量级，会漂移出假 Done 分支）。
        let busy = rt
            .js_worker
            .execute_script_priority_deferrable(
                "var s=0;for(var i=0;i<15e8;i++)s+=i;String(s)",
                std::time::Duration::from_millis(1),
            )
            .expect_err("长臂应挂起");
        std::thread::sleep(std::time::Duration::from_millis(120));
        // 挂起的自动化脚本：改 DOM + 返回值（入 pending 表，handle_automation_request
        // 按长度差检测不回信）。
        rt.handle_automation_request(
            77,
            AutomationRequest {
                operation: AutomationOperation::ExecuteScript {
                    script: "document.body.setAttribute('data-late','yes'); 'ok'".into(),
                    arguments: vec![],
                },
            },
        )
        .expect("挂起路径应成功入队");
        // 长臂完成 → worker 随后执行挂起求值（mutation 入共享队列、reply 投递 pending
        // 通道）；poll 轮询到补答成功为止（eval 在长臂结束后才执行）。
        let _ = busy.recv_timeout(std::time::Duration::from_secs(35));
        let mut applied = false;
        for _ in 0..100 {
            rt.poll_deferred_automation_replies().expect("poll 补答");
            if rt.cached_html.contains("data-late") {
                applied = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(
            applied,
            "补答路径须 apply mutation 并落进 cached_html（F4 接线真实路径）"
        );
    }

    /// t-p106 outbound 捕获：共享缓冲替代 io::sink，阶段返回后直接检查回复帧是否已写出。
    struct CaptureWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("capture lock").extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// t-p106 形态 A 回归：挂起求值就绪后、同步页面脚本阶段阻塞主循环期间，回复必须在
    /// 阶段内（脚本间让路点）送达 outbound——此前唯一交付点是主循环 16ms tick 的
    /// [`RendererRuntime::poll_deferred_automation_replies`]，整段阶段不可达（github.com
    /// 实测单阶段阻塞 42s+，headless 35s deadline 先到，迟到回复按消息 id 不匹配被丢弃；
    /// 证据 RUN/tmp/p106-diag/d2-areply.trace：r-defer req=2 → r-phase-end dur_ms=42257
    /// → r-poll req=2 → h-miss wait=3 got=2）。本测试把 runtime 侧交付闭包置 None
    /// （移除脚本间让路点交付）即红：阶段返回时 outbound 无回复帧。
    ///
    /// 挂起条目直接构造（rx 预置完成值），不經 2s 有界等待挂起路径——该路径已由
    /// [`deferred_reply_poll_applies_mutations_via_real_path`] 的长臂钉住；本测避免再添
    /// 长臂 CPU 烧（模块内 6 路并行 V8 的时序 margin 见 tick_per_task_tests d1 注）。
    #[test]
    fn deferred_reply_delivered_during_page_script_phase() {
        let captured: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // 5 条 ~300ms 忙脚本（IIFE 隔离 const 作用域）：阶段总时长 ~1.5s，为让路点交付
        // 留出多个检查窗；不预跑（runtime() 同款初始 run_page_scripts 会先烧一遍）。
        let busy_script = "<script>(()=>{const t=Date.now();while(Date.now()-t<300);})();</script>";
        let html = format!("<html><body>{}</body></html>", busy_script.repeat(5));
        let url = "https://zero.test/p106-phase-delivery";
        let (_tx, rx) = mpsc::channel();
        let mut rt = RendererRuntime::with_io(
            301,
            FramePublishMode::Legacy,
            Box::new(CaptureWriter(std::sync::Arc::clone(&captured))),
            rx,
        );
        rt.compositor_publish = None;
        rt.stub_network = true;
        rt.current_url = Some(url.into());
        rt.cached_html = html.clone();
        rt.navigation_epoch = 9;
        rt.document_generation = 1;
        rt.webview.as_mut().unwrap().prepare_document_state(url);
        rt.webview.as_mut().unwrap().load_html(html.as_str(), None);

        // 形态 A 状态：挂起条目就绪（reply 通道已有完成值）+ 真实 DOM mutation 滞留
        // worker 队列（宿主侧尚未 apply——与挂起求值晚至完成后的滞留同型，交付点须收编）。
        rt.js_worker
            .execute_script_direct("document.body.setAttribute('data-p106','yes');")
            .expect("mutation 落队");
        let (reply_tx, reply_rx) = mpsc::channel();
        reply_tx
            .send(Ok("p106-phase-delivery".to_string()))
            .expect("预置完成值");
        rt.pending_automation_replies.push(PendingAutomationReply {
            request_id: 77,
            rx: reply_rx,
            kind: DeferredEvalKind::ScriptValue,
        });

        // 同步页面脚本阶段（真实路径）：修复后首个脚本间让路点即交付。
        rt.after_page_html_loaded_with_cache(std::collections::HashMap::new())
            .expect("阶段执行");

        let captured_bytes = captured.lock().expect("capture lock");
        // outbound 是 4 字节 LE 长度前缀 + bincode 载荷的帧流（PipeTransport::send_frame），
        // 用协议自身解码器逐帧解出后断言（字符串断言对二进制帧不成立）。
        let mut delivered_reply = None;
        {
            let mut rest = &captured_bytes[..];
            while rest.len() >= 4 {
                let len = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                if rest.len() < 4 + len {
                    break;
                }
                let (payload, tail) = rest.split_at(4 + len);
                rest = tail;
                if let Ok(msg) = zero_protocol::deserialize(&payload[4..])
                    && let zero_protocol::message::IpcMessageKind::AutomationResponse(response) = msg.kind
                    && msg.id == 77
                {
                    delivered_reply = Some(response.result);
                }
            }
        }
        let reply = delivered_reply.expect("挂起回复帧须在阶段内写出（形态 A：主循环被阶段阻塞时迟到回复被丢弃）");
        assert_eq!(
            reply,
            Ok(AutomationResult::Value(AutomationValue::String(
                "p106-phase-delivery".into()
            ))),
            "阶段内交付的回复须携带求值结果"
        );
        assert!(
            rt.cached_html.contains("data-p106"),
            "交付的求值 mutation 须收编进 cached_html（t2-pb1 F4 语义不回退）"
        );
    }

    /// t-p106 形态 B 回归（第二机制）：阶段中到达的自动化 eval 须在阶段内 dispatch、
    /// 回复在阶段内交付——此前 staged eval 只能等阶段结束由主循环 handle_automation_request
    /// 补跑（d3 实测阶段 68.6s 内到达的求值全部滞留，阶段后重放补齐 ~28s，headless 35s
    /// deadline 先到；证据 RUN/tmp/p106-diag/d3-*）。eval 由独立线程在阶段进行中（~500ms，
    /// 落在第 1-2 条忙脚本之间）经 inbound 通道注入，走真实收编路径（poll_script_abort
    /// staging → 脚本间让路点 dispatch → 同点交付）。本测试把 runtime 侧 automation_hook
    /// 置 None（移除阶段中 dispatch/交付）即红：消息滞留 staged，阶段返回时 outbound
    /// 无回复帧。
    #[test]
    fn staged_eval_dispatched_and_delivered_during_page_script_phase() {
        let captured: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // 5 条 ~300ms 忙脚本（IIFE 隔离 const 作用域）：阶段总时长 ~1.5s，注入点落在
        // 阶段中段，其后仍有多个让路点供 dispatch/交付；不预跑（同形态 A 测的理由）。
        let busy_script = "<script>(()=>{const t=Date.now();while(Date.now()-t<300);})();</script>";
        let html = format!("<html><body>{}</body></html>", busy_script.repeat(5));
        let url = "https://zero.test/p106-phase-dispatch";
        let (tx, rx) = mpsc::channel();
        let mut rt = RendererRuntime::with_io(
            301,
            FramePublishMode::Legacy,
            Box::new(CaptureWriter(std::sync::Arc::clone(&captured))),
            rx,
        );
        rt.compositor_publish = None;
        rt.stub_network = true;
        rt.current_url = Some(url.into());
        rt.cached_html = html.clone();
        rt.navigation_epoch = 9;
        rt.document_generation = 1;
        rt.webview.as_mut().unwrap().prepare_document_state(url);
        rt.webview.as_mut().unwrap().load_html(html.as_str(), None);

        // 阶段进行中注入自动化 eval（宿主动作；ExecuteScript 包络要求显式 return）。
        let injector_tx = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = injector_tx.send(IpcMessage {
                id: 78,
                kind: IpcMessageKind::AutomationRequest(AutomationRequest {
                    operation: AutomationOperation::ExecuteScript {
                        script: "document.body.setAttribute('data-p106-b','dispatched'); return 'p106-phase-dispatch';"
                            .to_string(),
                        arguments: vec![],
                    },
                }),
            });
        });

        // 同步页面脚本阶段（真实路径）：修复后 eval 在其后首个脚本间让路点 dispatch、
        // 同点交付，剩余页面脚本照常执行。
        rt.after_page_html_loaded_with_cache(std::collections::HashMap::new())
            .expect("阶段执行");

        let captured_bytes = captured.lock().expect("capture lock");
        let mut delivered_reply = None;
        {
            let mut rest = &captured_bytes[..];
            while rest.len() >= 4 {
                let len = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                if rest.len() < 4 + len {
                    break;
                }
                let (payload, tail) = rest.split_at(4 + len);
                rest = tail;
                if let Ok(msg) = zero_protocol::deserialize(&payload[4..])
                    && let zero_protocol::message::IpcMessageKind::AutomationResponse(response) = msg.kind
                    && msg.id == 78
                {
                    delivered_reply = Some(response.result);
                }
            }
        }
        let reply = delivered_reply
            .expect("阶段中到达的 eval 回复帧须在阶段内写出（移除阶段中 dispatch/交付即滞留 staged 无人补答）");
        assert_eq!(
            reply,
            Ok(AutomationResult::Value(AutomationValue::String(
                "p106-phase-dispatch".into()
            ))),
            "阶段内交付的回复须携带求值结果"
        );
        assert!(
            rt.cached_html.contains("data-p106-b"),
            "阶段中 dispatch 的求值 mutation 须由让路点 apply 收编（不跨 document_generation）"
        );
    }
}
