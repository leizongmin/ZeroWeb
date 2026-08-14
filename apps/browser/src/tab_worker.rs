//! 标签页渲染 worker — 每个 Tab 独立 OS 线程，持有 WebView 与异步加载状态。

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_engine::{set_char_measure_fn, set_text_shape_fn};
use zero_protocol::message::{ImeEventParams, ImeEventType};
use zero_render_foundation::font::loader::FontLoader;
use zero_webview::{AsyncPageLoad, InProcessFetchHost, PageLoadStage, WebView, WebViewBuilder, WebViewConfig};

use crate::pages;
use crate::tab_js_worker::TabJsWorkerHandle;
use crate::tab_scripts;
use crate::tab_snapshot::TabSnapshot;
use crate::text_metrics;

fn is_printable_key(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(c) if !c.is_control()) && chars.next().is_none()
}

fn execute_shared_action(
    wv: &mut WebView,
    js_worker: Option<&TabJsWorkerHandle>,
    javascript_enabled: bool,
    selector: &str,
    action: zero_page_runtime::HtmlUserAction,
) -> Option<zero_webview::WebViewUserActionResult> {
    let target = wv.page_node_ref_for_selector(selector)?;
    let request = zero_page_runtime::HtmlActionRequest {
        target,
        action,
        shift: false,
    };
    match js_worker {
        Some(worker) => wv
            .dispatch_external_user_action_with_javascript(worker, javascript_enabled, request)
            .ok(),
        None => wv
            .dispatch_user_action_with_javascript(javascript_enabled, request)
            .ok(),
    }
}

fn set_shared_text_selection(
    wv: &mut WebView,
    js_worker: Option<&TabJsWorkerHandle>,
    selector: &str,
    selection: Option<(u32, u32)>,
) {
    let Some((start, end)) = selection else { return };
    let Some(target) = wv.page_node_ref_for_selector(selector) else {
        return;
    };
    match js_worker {
        Some(worker) => {
            let _ = wv.set_external_text_control_selection(worker, target, start as usize, end as usize);
        }
        None => {
            let _ = wv.set_text_control_selection(target, start as usize, end as usize);
        }
    }
}

fn action_focus_selector(wv: &WebView, result: &zero_webview::WebViewUserActionResult) -> Option<Option<String>> {
    result.effects.iter().find_map(|effect| match effect {
        zero_page_runtime::PageEffect::Focus(node) => {
            Some(node.and_then(|node| wv.selector_for_page_node_handle(node.node().get())))
        }
        _ => None,
    })
}

fn action_navigation(result: &zero_webview::WebViewUserActionResult) -> Option<(String, String, Option<String>)> {
    result.effects.iter().find_map(|effect| match effect {
        zero_page_runtime::PageEffect::Navigate(intent) => {
            Some((intent.url.clone(), intent.method.clone(), intent.body.clone()))
        }
        zero_page_runtime::PageEffect::SetFragment { .. } => None,
        _ => None,
    })
}

/// 每帧在 worker 内推进加载/渲染的时间预算（毫秒）。
pub const TAB_WORKER_FRAME_BUDGET_MS: f64 = 8.0;

/// 发送给 Tab worker 的命令。
pub enum TabWorkerCommand {
    /// 导航到 URL（异步分阶段加载）。
    Navigate(String),
    /// 导航主文档请求（表单 POST 等）。
    NavigateRequest {
        /// 请求 URL。
        url: String,
        /// HTTP 方法。
        method: String,
        /// 可选请求体。
        body: Option<String>,
    },
    /// 同步加载 HTML（测试 / zero:// 页面）。
    LoadHtml {
        html: String,
        css: Option<String>,
        url: Option<String>,
    },
    /// 调整视口。
    Resize { width: u32, height: u32 },
    /// 更新颜色方案。
    SetColorScheme(PrefersColorSchemeValue),
    /// 更新渲染媒体类型（DC-12 @media print 打印预览；R1993）。
    SetMediaType(zero_engine::MediaType),
    /// 异步向页面元素派发 DOM 事件（click / keydown 等）。
    ///
    /// 结果通过 `TabWorkerMessage::DispatchResult` 异步回送，避免 UI 主线程阻塞等待。
    DispatchDomEvent {
        dispatch_id: u64,
        selector: String,
        event_type: String,
        key: Option<String>,
        code: Option<String>,
        /// Shift 修饰键（R3254-M10：Shift+Tab 反向焦点导航）。
        shift: bool,
        /// 指针定位得到的 UTF-16 selection。
        selection: Option<(u32, u32)>,
    },
    /// 平台 IME 生命周期事件。
    ImeEvent {
        selector: Option<String>,
        params: ImeEventParams,
    },
    /// 更新是否允许执行 JavaScript。
    SetJavascriptEnabled(bool),
    /// R3293（S0）：用户滚动 fire-and-forget 注入（单进程路径）——执行 `script_user_scroll`
    /// 注入 `__zw_user_scroll` → 派 'scroll' + 更 window.scrollY。无回执（滚动不需 default-action）。
    UserScroll { delta_x: f32, delta_y: f32 },
    /// 测试用：在 worker 的 WebView 上执行 JS 并同步回执结果（经 reply channel）。
    /// 供单进程 BrowserApp 级集成测试读回页面 JS 状态（如滚动 listener 触发计数）。
    /// 非测试代码不应使用——同步回执会阻塞 worker 线程命令循环。
    #[cfg(test)]
    ExecuteScriptForTest {
        script: String,
        reply: std::sync::mpsc::Sender<Result<String, String>>,
    },
    /// 关闭 worker。
    Shutdown,
}

/// Worker 发往 UI 线程的消息。
#[allow(clippy::large_enum_variant)]
pub enum TabWorkerMessage {
    /// 快照更新。
    Snapshot(TabSnapshot),
    /// 页面标题。
    Title(String),
    /// 加载失败。
    LoadError(String),
    /// 加载阶段变化。
    Stage(PageLoadStage),
    /// 异步 DOM 事件派发的回执。
    DispatchResult {
        dispatch_id: u64,
        default_allowed: bool,
        html_changed: bool,
    },
    /// R3254-M10：页面焦点所有者变更（Tab 默认动作 / JS focus() 镜像——TabManager 同步
    /// `event_targets`，与多进程 `FocusOwnerChanged` 回执同语义）。None 表示失焦。
    FocusChanged(Option<String>),
    /// R3254-M10：表单提交导航请求（Enter 默认动作——TabManager 转发导航）。
    SubmitNavigation {
        /// 提交目标 URL（GET 含序列化 query；POST 为 action URL）。
        url: String,
        /// HTTP 方法（"GET" / "POST"）。
        method: String,
        /// POST body。
        body: Option<String>,
    },
}

/// Tab worker 句柄（UI 线程持有）。
pub struct TabWorkerHandle {
    cmd_tx: Sender<TabWorkerCommand>,
    msg_rx: Receiver<TabWorkerMessage>,
    join: Option<JoinHandle<()>>,
}

impl TabWorkerHandle {
    /// 启动新 Tab worker。
    pub fn spawn(tab_id: TabId, viewport: (u32, u32), color_scheme: PrefersColorSchemeValue) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        let join = thread::Builder::new()
            .name(format!("tab-worker-{}", tab_id.0))
            .spawn(move || tab_worker_main(tab_id, viewport, color_scheme, cmd_rx, msg_tx))
            .expect("spawn tab worker");

        Self {
            cmd_tx,
            msg_rx,
            join: Some(join),
        }
    }

    /// 发送命令（忽略 channel 关闭错误）。
    pub fn send(&self, cmd: TabWorkerCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// 非阻塞接收 worker 消息。
    pub fn try_recv(&self) -> Option<TabWorkerMessage> {
        self.msg_rx.try_recv().ok()
    }

    /// 关闭 worker 并等待线程退出。
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(TabWorkerCommand::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for TabWorkerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn tab_worker_main(
    tab_id: TabId,
    viewport: (u32, u32),
    color_scheme: PrefersColorSchemeValue,
    cmd_rx: Receiver<TabWorkerCommand>,
    msg_tx: Sender<TabWorkerMessage>,
) {
    set_char_measure_fn(text_metrics::measure_char);
    set_text_shape_fn(text_metrics::shape_text);
    let mut font_loader = FontLoader::new();
    let font_id = load_system_fonts_worker(&mut font_loader);

    let _js_worker = {
        #[cfg(not(test))]
        {
            let js_worker = TabJsWorkerHandle::spawn(tab_id);
            // P1b S3 / R2923：注入生产 fetch handler（经 ResourceLoader 真实 HTTP，支持全方法/头/体）。
            // js_worker 早于 WebView 创建；共享加载器不依赖 WebView 句柄，故可在 spawn 后立即注入。
            js_worker.set_fetch_handler(crate::tab_js_worker::default_fetch_handler());
            Some(js_worker)
        }
        #[cfg(test)]
        {
            None::<TabJsWorkerHandle>
        }
    };

    let mut builder = WebViewBuilder::new().width(viewport.0).height(viewport.1);
    if let Some(ref js_worker) = _js_worker {
        builder = builder.external_script(js_worker.executor());
    }
    let mut wv = builder.build();
    wv.set_prefers_color_scheme(color_scheme);
    // R2413：初始化 font_resolver（系统字体）+ per-family 行度量——镜像 renderer 进程
    // `with_io`（main.rs:170-173）。旧版 tab_worker（in-process 回退路径：
    // `ZERO_BROWSER_MULTIPROCESS=0` 或 renderer binary 不可用时 tab_manager 回退）从不设
    // resolver → webview font_resolver 为空 → painter resolve_font_id 全回落 FontId(0)，
    // CSS font-family 被忽略，直至 @font-face 经 R2409 drain 加载后才更新；font_metric_map
    // 亦漏（R2202 dormant 默认零影响，但激活 ZW_PERFONT_LINEHEIGHT=1 时 in-process 会缺）。
    // 此初始设置使 in-process 路径与 renderer 一致。
    wv.set_font_resolver(font_loader.build_font_resolver());
    wv.set_font_metric_map(font_loader.build_line_metric_map());
    let _ = WebViewConfig::default();

    let mut async_load: Option<AsyncPageLoad> = None;
    let mut fetch_host = InProcessFetchHost;
    let mut pending_sync_html: Option<(String, Option<String>, Option<String>)> = None;
    let mut page_script_runner: Option<tab_scripts::PageScriptRunner> = None;
    let mut javascript_enabled = true;
    let mut composing_controls = HashSet::new();

    let push_snapshot = |wv: &WebView, msg_tx: &Sender<TabWorkerMessage>, js_worker: Option<&TabJsWorkerHandle>| {
        let snapshot = TabSnapshot::from_webview(wv);
        // P1a gBCR：render 后用最新 layout 填 rect snapshot——js_worker 的 RectBridge handler
        // 经 identity(selector)→NodeId 查此 snapshot 返真实 DOMRect（未填/未命中→零 rect，零回归）。
        // 复用 `snapshot.hit_test`（`from_webview` 已建），避免二次 `build_hit_test_cache`。
        if let Some(worker) = js_worker
            && let Some(cache) = snapshot.hit_test.as_ref()
        {
            cache.fill_layout_rect_snapshot(&worker.rect_snapshot());
            // P1a elementFromPoint：render 后 swap 最新 `Arc<HitTestCache>` 进共享槽（无数据 clone，
            // 仅引用计数）→ js_worker 的 `__zw_elementFromPoint` 读它求 `(x,y)` 命中元素。
            *worker.element_from_point_cache().lock().unwrap() = Some(std::sync::Arc::new(cache.clone()));
        }
        let _ = msg_tx.send(TabWorkerMessage::Snapshot(snapshot));
    };

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                TabWorkerCommand::Navigate(url) => {
                    tracing::info!("Tab {} navigate: {url}", tab_id.0);
                    wv.prepare_document_state(&url);
                    async_load = Some(AsyncPageLoad::start(url));
                    pending_sync_html = None;
                    page_script_runner = None;
                }
                TabWorkerCommand::NavigateRequest { url, method, body } => {
                    tracing::info!("Tab {} navigate: {method} {url}", tab_id.0);
                    wv.prepare_document_state(&url);
                    async_load = Some(AsyncPageLoad::start_request(url, method, body.map(String::into_bytes)));
                    pending_sync_html = None;
                    page_script_runner = None;
                }
                TabWorkerCommand::LoadHtml { html, css, url } => {
                    pending_sync_html = Some((html, css, url));
                    async_load = None;
                    page_script_runner = None;
                }
                TabWorkerCommand::Resize { width, height } => {
                    with_measure(&font_loader, font_id, || wv.resize(width, height));
                    if wv.last_render().is_some() {
                        with_measure(&font_loader, font_id, || {
                            if wv.render_incremental().is_none() {
                                wv.render();
                            }
                        });
                    }
                    push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                }
                TabWorkerCommand::SetColorScheme(scheme) => {
                    wv.set_prefers_color_scheme(scheme);
                    if wv.last_render().is_some() {
                        with_measure(&font_loader, font_id, || {
                            if wv.render_incremental().is_none() {
                                wv.render();
                            }
                        });
                        push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                    }
                }
                TabWorkerCommand::SetMediaType(media_type) => {
                    wv.set_media_type(media_type);
                    if wv.last_render().is_some() {
                        with_measure(&font_loader, font_id, || {
                            if wv.render_incremental().is_none() {
                                wv.render();
                            }
                        });
                        push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                    }
                }
                TabWorkerCommand::DispatchDomEvent {
                    dispatch_id,
                    selector,
                    event_type,
                    key,
                    code,
                    shift,
                    selection,
                } => {
                    let html = wv.html_content().to_string();
                    if event_type == "mousedown" {
                        set_shared_text_selection(&mut wv, _js_worker.as_ref(), &selector, selection);
                    }
                    let detail = if key.is_some() || code.is_some() {
                        Some(zero_engine::DomEventDetail {
                            key: key.clone(),
                            code: code.clone(),
                            ..Default::default()
                        })
                    } else {
                        None
                    };
                    let result = if event_type == "click" {
                        let is_checked_control =
                            zero_engine::is_checkbox(&html, &selector) || zero_engine::is_radio(&html, &selector);
                        let is_disabled_button = zero_engine::has_attribute(&html, &selector, "disabled")
                            && (zero_engine::is_reset_button(&html, &selector)
                                || zero_engine::is_submit_button(&html, &selector));
                        if is_checked_control || is_disabled_button {
                            match execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::Activate,
                            ) {
                                Some(action)
                                    if action.noop_reason
                                        != Some(zero_page_runtime::ActionNoopReason::AlreadySelected) =>
                                {
                                    tab_scripts::DomDispatchResult {
                                        default_allowed: !action.canceled && action.noop_reason.is_none(),
                                        html_changed: action.changed,
                                    }
                                }
                                _ => tab_scripts::dispatch_dom_event(
                                    &mut wv,
                                    javascript_enabled,
                                    _js_worker.as_ref(),
                                    &selector,
                                    &event_type,
                                    &html,
                                    detail.as_ref(),
                                ),
                            }
                        } else {
                            tab_scripts::dispatch_dom_event(
                                &mut wv,
                                javascript_enabled,
                                _js_worker.as_ref(),
                                &selector,
                                &event_type,
                                &html,
                                detail.as_ref(),
                            )
                        }
                    } else {
                        tab_scripts::dispatch_dom_event(
                            &mut wv,
                            javascript_enabled,
                            _js_worker.as_ref(),
                            &selector,
                            &event_type,
                            &html,
                            detail.as_ref(),
                        )
                    };
                    // R3254-M10：keydown 默认动作镜像 renderer（Tab/Shift+Tab 焦点导航、
                    // printable 插入、Backspace 删除、Enter 换行/提交）。焦点切换只回传
                    // FocusChanged 让 TabManager 同步 event_targets（worker 无焦点状态机；
                    // click 焦点切换/blur/change 生命周期为已知单进程限制，见 TODO）。
                    let mut focus_change: Option<Option<String>> = None;
                    let mut navigation: Option<(String, String, Option<String>)> = None;
                    let mut input_changed = false;
                    if result.default_allowed && event_type == "keydown" {
                        if key.as_deref() == Some("Tab") {
                            if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::MoveFocus { forward: !shift },
                            ) {
                                input_changed |= action.changed;
                                focus_change = action_focus_selector(&wv, &action);
                            }
                        } else if key.as_deref().is_some_and(is_printable_key) {
                            if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::InsertText {
                                    text: key.clone().unwrap_or_default(),
                                },
                            ) {
                                input_changed |= action.changed;
                            }
                        } else if key.as_deref() == Some("Backspace") {
                            if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::DeleteBackward,
                            ) {
                                input_changed |= action.changed;
                            }
                        } else if key.as_deref() == Some("Enter") {
                            if zero_engine::query_tag_from_html(&html, &selector).eq_ignore_ascii_case("textarea") {
                                // textarea 的 Enter 是换行（不提交）。
                                if let Some(action) = execute_shared_action(
                                    &mut wv,
                                    _js_worker.as_ref(),
                                    javascript_enabled,
                                    &selector,
                                    zero_page_runtime::HtmlUserAction::InsertText { text: "\n".to_string() },
                                ) {
                                    input_changed |= action.changed;
                                }
                            } else if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::Submit,
                            ) {
                                input_changed |= action.changed;
                                navigation = action_navigation(&action);
                            }
                        }
                    }
                    if result.default_allowed && event_type == "click" {
                        let current_html = wv.html_content().to_string();
                        if zero_engine::is_reset_button(&current_html, &selector) {
                            if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::Reset,
                            ) {
                                input_changed |= action.changed;
                            }
                        } else if zero_engine::is_submit_button(&current_html, &selector) {
                            if let Some(action) = execute_shared_action(
                                &mut wv,
                                _js_worker.as_ref(),
                                javascript_enabled,
                                &selector,
                                zero_page_runtime::HtmlUserAction::Submit,
                            ) {
                                input_changed |= action.changed;
                                navigation = action_navigation(&action);
                            }
                        }
                    }
                    if result.html_changed || input_changed {
                        // apply_recorded_mutations 已完成 live DOM 增量渲染。
                        push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                    }
                    if let Some(focus) = focus_change {
                        let _ = msg_tx.send(TabWorkerMessage::FocusChanged(focus));
                    }
                    if let Some((url, method, body)) = navigation {
                        let _ = msg_tx.send(TabWorkerMessage::SubmitNavigation { url, method, body });
                    }
                    let _ = msg_tx.send(TabWorkerMessage::DispatchResult {
                        dispatch_id,
                        default_allowed: result.default_allowed,
                        html_changed: result.html_changed || input_changed,
                    });
                }
                TabWorkerCommand::ImeEvent { selector, params } => {
                    let Some(selector) = selector else { continue };
                    let html = wv.html_content().to_string();
                    let changed = match params.event_type {
                        ImeEventType::Enabled => false,
                        ImeEventType::Preedit => {
                            let was_composing = composing_controls.contains(&selector);
                            let events = if params.text.is_empty() && was_composing {
                                vec![("compositionupdate", ""), ("compositionend", "")]
                            } else if !params.text.is_empty() && !was_composing {
                                vec![
                                    ("compositionstart", params.text.as_str()),
                                    ("compositionupdate", params.text.as_str()),
                                ]
                            } else {
                                vec![("compositionupdate", params.text.as_str())]
                            };
                            let event_changed = events.into_iter().fold(false, |changed, (event_type, data)| {
                                changed
                                    | tab_scripts::dispatch_dom_event(
                                        &mut wv,
                                        javascript_enabled,
                                        _js_worker.as_ref(),
                                        &selector,
                                        event_type,
                                        &html,
                                        Some(&zero_engine::DomEventDetail {
                                            data: Some(data.to_string()),
                                            ..Default::default()
                                        }),
                                    )
                                    .html_changed
                            });
                            if params.text.is_empty() {
                                composing_controls.remove(&selector);
                            } else {
                                composing_controls.insert(selector.clone());
                            }
                            event_changed
                                | tab_scripts::apply_ime_preedit_default(
                                    &mut wv,
                                    _js_worker.as_ref(),
                                    &selector,
                                    &params.text,
                                    &html,
                                    // R3254-L3：传平台光标/选区（组合内移动/选择）。
                                    Some((params.cursor_start.unwrap_or(0), params.cursor_end.unwrap_or(0))),
                                )
                        }
                        ImeEventType::Commit => {
                            let event_changed = composing_controls.remove(&selector)
                                && tab_scripts::dispatch_dom_event(
                                    &mut wv,
                                    javascript_enabled,
                                    _js_worker.as_ref(),
                                    &selector,
                                    "compositionend",
                                    &html,
                                    Some(&zero_engine::DomEventDetail {
                                        data: Some(params.text.clone()),
                                        ..Default::default()
                                    }),
                                )
                                .html_changed;
                            event_changed
                                | if params.text.is_empty() {
                                    tab_scripts::apply_ime_preedit_default(
                                        &mut wv,
                                        _js_worker.as_ref(),
                                        &selector,
                                        "",
                                        &html,
                                        None,
                                    )
                                } else {
                                    tab_scripts::apply_text_input_default(
                                        &mut wv,
                                        javascript_enabled,
                                        _js_worker.as_ref(),
                                        &selector,
                                        &params.text,
                                        &html,
                                        false,
                                    )
                                }
                        }
                        ImeEventType::Disabled => {
                            let event_changed = composing_controls.remove(&selector)
                                && tab_scripts::dispatch_dom_event(
                                    &mut wv,
                                    javascript_enabled,
                                    _js_worker.as_ref(),
                                    &selector,
                                    "compositionend",
                                    &html,
                                    Some(&zero_engine::DomEventDetail {
                                        data: Some(String::new()),
                                        ..Default::default()
                                    }),
                                )
                                .html_changed;
                            event_changed
                                | tab_scripts::apply_ime_preedit_default(
                                    &mut wv,
                                    _js_worker.as_ref(),
                                    &selector,
                                    "",
                                    &html,
                                    None,
                                )
                        }
                    };
                    if changed {
                        push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                    }
                }
                TabWorkerCommand::SetJavascriptEnabled(enabled) => {
                    javascript_enabled = enabled;
                }
                TabWorkerCommand::UserScroll { delta_x, delta_y } => {
                    // R3293（S0）：用户滚动注入（单进程路径）。注入到页面 JS 真实上下文：
                    // 生产（`_js_worker = Some`）走 `js_worker.execute_script_direct`（TabJsWorkerHandle
                    // 持久上下文 + shim）；测试（`_js_worker = None`）走 `wv.execute_script`（WebView
                    // 内部沙箱，shim 内置）。gate javascript_enabled + best-effort（typeof 守卫防 shim
                    // 未装静默）。fire-and-forget 无回执——主线程已在 apply_page_scroll_delta 完成视觉滚动。
                    if javascript_enabled {
                        let script = zero_engine::script_user_scroll(delta_x as f64, delta_y as f64);
                        let res: Result<(), String> = if let Some(js_worker) = _js_worker.as_ref() {
                            js_worker
                                .execute_script_direct(&script)
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        } else {
                            wv.execute_script(&script).map(|_| ()).map_err(|e| e.to_string())
                        };
                        if let Err(e) = res {
                            tracing::warn!("dispatch user scroll (single-process): {e}");
                        }
                    }
                }
                #[cfg(test)]
                TabWorkerCommand::ExecuteScriptForTest { script, reply } => {
                    // 读回页面 JS 状态：生产走 _js_worker，测试（无 js_worker）走 wv 内部沙箱。
                    let result: Result<String, String> = if let Some(js_worker) = _js_worker.as_ref() {
                        js_worker.execute_script_direct(&script).map_err(|e| e.to_string())
                    } else {
                        wv.execute_script(&script).map_err(|e| e.to_string())
                    };
                    let _ = reply.send(result);
                }
                TabWorkerCommand::Shutdown => {
                    tracing::debug!("Tab worker {} shutting down", tab_id.0);
                    return;
                }
            }
        }

        // R3254-M10：drain 页面 JS focus()/blur() 变更（任意脚本执行均可产生）→ 回传
        // TabManager 同步 event_targets（与多进程 FocusOwnerChanged 回执同语义）。
        // 采纳前过 is_focusable_selector 校验（shim 不校验可聚焦性——不可聚焦元素的
        // JS focus 忽略，与多进程 sync_focus_from_js 行为一致）。
        if let Some(js_worker) = _js_worker.as_ref() {
            let changes = {
                let queue = js_worker.focus_changes();
                let mut queue = queue.lock().unwrap_or_else(|e| e.into_inner());
                std::mem::take(&mut *queue)
            };
            for change in changes {
                if let Some(sel) = change.as_deref()
                    && !zero_engine::is_focusable_selector(wv.html_content(), sel)
                {
                    continue;
                }
                let _ = msg_tx.send(TabWorkerMessage::FocusChanged(change));
            }
        }

        if let Some((html, css, url)) = pending_sync_html.take() {
            if let Some(u) = url {
                wv.prepare_document_state(&u);
            }
            with_measure(&font_loader, font_id, || {
                wv.load_html(&html, css.as_deref());
            });
            page_script_runner = tab_scripts::PageScriptRunner::start(&wv, javascript_enabled);
            if let Some(title) = pages::extract_html_title(&html) {
                wv.set_title(&title);
            }
            push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
            let title = page_title_from_webview(&wv);
            let _ = msg_tx.send(TabWorkerMessage::Title(title));
        }

        if let Some(ref mut runner) = page_script_runner {
            runner.tick(&mut wv, _js_worker.as_ref());
            push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
            if !runner.is_active() {
                runner.finish(&mut wv, _js_worker.as_ref());
                let title = page_title_from_webview(&wv);
                let _ = msg_tx.send(TabWorkerMessage::Title(title));
                push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                page_script_runner = None;
            }
        }

        if let Some(ref mut load) = async_load {
            let prev_stage = load.stage();
            let changed = with_measure(&font_loader, font_id, || {
                load.tick(&mut wv, &mut fetch_host, TAB_WORKER_FRAME_BUDGET_MS)
            });
            // R2408+ slice 2：drain @font-face 字节 → load_font + register_family_alias →
            // 刷新 webview font_resolver → 请求重绘。须在 with_measure 闭包外（闭包内
            // font_loader 被不可变借做文本度量）。env `ZW_LIVE_FONTFACE` kill-switch 默认开。
            if zero_webview::live_fontface_enabled() {
                let loaded = load.drain_loaded_fonts();
                if !loaded.is_empty() {
                    let mut updated = false;
                    for (
                        family,
                        weight,
                        is_italic,
                        stretch,
                        size_adjust,
                        features,
                        variations,
                        unicode_ranges,
                        bytes,
                    ) in loaded
                    {
                        match font_loader.load_font(&bytes) {
                            Ok(id) => {
                                font_loader.register_font_features(id, features);
                                if zero_engine::font_variations_enabled() {
                                    font_loader.register_font_variations(id, variations);
                                }
                                font_loader.register_unicode_ranges(id, unicode_ranges);
                                if let Some(scale) = size_adjust {
                                    font_loader.register_font_size_adjust(id, scale);
                                }
                                for alias in
                                    zero_render_foundation::font::font_face_aliases(&family, weight, is_italic, stretch)
                                {
                                    font_loader.register_family_alias(&alias, id);
                                }
                                updated = true;
                            }
                            Err(e) => tracing::warn!(family = %family, err = %e, "live @font-face load failed"),
                        }
                    }
                    if updated {
                        let resolver = font_loader.build_font_resolver();
                        wv.set_font_resolver(resolver);
                        load.request_rerender();
                    }
                }
            }
            if changed {
                if load.stage() != prev_stage {
                    let _ = msg_tx.send(TabWorkerMessage::Stage(load.stage()));
                }
                push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
            }
            if !load.is_active() {
                if let Some(err) = load.take_error() {
                    let page_url = wv.url().unwrap_or("about:blank");
                    let error_page = pages::generate_error_page(page_url, &err);
                    with_measure(&font_loader, font_id, || {
                        wv.load_html(&error_page, None);
                    });
                    let _ = msg_tx.send(TabWorkerMessage::LoadError(err));
                    let _ = msg_tx.send(TabWorkerMessage::Title("加载失败".to_string()));
                } else {
                    page_script_runner = tab_scripts::PageScriptRunner::start(&wv, javascript_enabled);
                    // R2942：drain async_load 期收集的子资源 fetch/decode 失败（stylesheet/image），
                    // 注入 runner——finish() 在页面脚本之后、window load 前派发 window error。
                    let failed: Vec<(String, String)> = load
                        .take_failed_resources()
                        .into_iter()
                        .map(|r| (r.kind.to_string(), r.url))
                        .collect();
                    if !failed.is_empty() {
                        if let Some(r) = page_script_runner.as_mut() {
                            r.set_resource_errors(failed);
                        }
                    }
                    // FR-009：drain 资源元素最终状态，finish() 提交 IDL 状态与规范事件。
                    let resource_element_events = load.take_resource_element_events();
                    if !resource_element_events.is_empty() {
                        if let Some(r) = page_script_runner.as_mut() {
                            r.set_resource_element_events(resource_element_events);
                        }
                    }
                    // R2944：drain stylesheet 元素级 load/error 事件——finish() 经 __zw_dispatch_link_event 派发到
                    // 匹配 href 的 <link> 元素（link.onload/onerror）。
                    let link_events = load.take_link_element_events();
                    if !link_events.is_empty() {
                        if let Some(r) = page_script_runner.as_mut() {
                            r.set_link_events(link_events);
                        }
                    }
                    // R2947：drain @font-face 加载结果——finish() 经 __zw_font_settle 派 FontFaceSet
                    // 'loadingdone'/'loadingerror' + 解析 document.fonts.ready（字体加载库 / icon font / FOUT）。
                    let font_events = load.take_font_events();
                    if !font_events.is_empty() {
                        if let Some(r) = page_script_runner.as_mut() {
                            r.set_font_events(font_events);
                        }
                    }
                    let title = page_title_from_webview(&wv);
                    let _ = msg_tx.send(TabWorkerMessage::Title(title));
                }
                async_load = None;
                push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
            }
        }

        thread::sleep(Duration::from_millis(1));
    }
}

fn with_measure<F, R>(loader: &FontLoader, font_id: Option<u32>, f: F) -> R
where
    F: FnOnce() -> R,
{
    text_metrics::with_measure_ctx_opt(loader, font_id, f)
}

fn load_system_fonts_worker(loader: &mut FontLoader) -> Option<u32> {
    // 进程级共享（生产与测试同路径）：主线程/其他 worker 已解析则复用——19MB CJK
    // 每 worker 独立解析 ~0.5-2.9s + ~40-60MB/份，多标签页重复成本线性增长。
    let (shared, id) = crate::app::shared_system_fonts();
    *loader = shared.duplicate();
    id
}

fn page_title_from_webview(wv: &WebView) -> String {
    wv.title()
        .map(str::to_string)
        .or_else(|| pages::extract_html_title(wv.html_content()))
        .unwrap_or_else(|| wv.url().unwrap_or("页面").to_string())
}

#[cfg(test)]
mod action_adapter_tests {
    use super::*;

    #[test]
    fn shared_text_action_honors_beforeinput_and_emits_input() {
        let html = r#"<html><body><input id="name"><input id="next"></body></html>"#;
        let url = "https://zero.test/tab-worker-actions";
        let mut worker = TabJsWorkerHandle::spawn(TabId(901));
        worker.set_dom_snapshot(html, url);
        worker
            .execute_script_direct(
                "globalThis.__events=[];\
                 var input=document.querySelector('#name');\
                 input.addEventListener('beforeinput',function(event){\
                   globalThis.__events.push('beforeinput:'+event.data);\
                   if(event.data==='X')event.preventDefault();\
                 });\
                 input.addEventListener('input',function(event){\
                   globalThis.__events.push('input:'+event.data);\
                 });",
            )
            .expect("register input listeners");

        let mut wv = WebView::new(WebViewConfig::default());
        wv.prepare_document_state(url);
        wv.load_html(html, None);
        let inserted = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#name",
            zero_page_runtime::HtmlUserAction::InsertText { text: "A".to_string() },
        )
        .expect("insert");
        assert!(inserted.changed);
        let canceled = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#name",
            zero_page_runtime::HtmlUserAction::InsertText { text: "X".to_string() },
        )
        .expect("canceled insert");
        assert!(canceled.canceled);
        assert_eq!(
            wv.form_control_value_overrides().get("#name").map(String::as_str),
            Some("A")
        );
        assert_eq!(
            worker
                .execute_script_direct("globalThis.__events.join(',')")
                .expect("event log"),
            "beforeinput:A,input:A,beforeinput:X"
        );
        let focused = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#name",
            zero_page_runtime::HtmlUserAction::MoveFocus { forward: true },
        )
        .expect("focus");
        assert_eq!(action_focus_selector(&wv, &focused), Some(Some("#next".to_string())));

        worker.shutdown();
    }

    #[test]
    fn shared_checked_and_form_actions_preserve_transaction_semantics() {
        let html = r#"<html><body>
            <form id="form" action="https://zero.test/submitted" method="get">
              <input id="name" name="name" value="base">
              <input id="check" type="checkbox">
              <input id="disabled" type="checkbox" disabled>
              <input id="basic" type="radio" name="plan" value="basic" checked>
              <input id="pro" type="radio" name="plan" value="pro">
              <button id="reset" type="reset">Reset</button>
              <button id="submit" type="submit" name="go" value="1">Submit</button>
            </form>
        </body></html>"#;
        let url = "https://zero.test/form";
        let mut worker = TabJsWorkerHandle::spawn(TabId(902));
        worker.set_dom_snapshot(html, url);
        worker
            .execute_script_direct(
                "globalThis.__events=[];\
                 globalThis.__cancelClick=false;\
                 globalThis.__cancelReset=false;\
                 globalThis.__cancelSubmit=false;\
                 var check=document.querySelector('#check');\
                 check.addEventListener('click',function(event){\
                   globalThis.__events.push('click:'+check.checked);\
                   if(globalThis.__cancelClick)event.preventDefault();\
                 });\
                 check.addEventListener('input',function(){globalThis.__events.push('input:'+check.checked);});\
                 check.addEventListener('change',function(){globalThis.__events.push('change:'+check.checked);});\
                 var form=document.querySelector('#form');\
                 form.addEventListener('reset',function(event){\
                   if(globalThis.__cancelReset)event.preventDefault();\
                   queueMicrotask(function(){globalThis.__resetValue=document.querySelector('#name').value;});\
                 });\
                 form.addEventListener('submit',function(event){\
                   document.querySelector('#name').value='listener';\
                   if(globalThis.__cancelSubmit)event.preventDefault();\
                 });",
            )
            .expect("register form listeners");

        let mut wv = WebView::new(WebViewConfig::default());
        wv.prepare_document_state(url);
        wv.load_html(html, None);
        let checked = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#check",
            zero_page_runtime::HtmlUserAction::Activate,
        )
        .expect("checkbox plan");
        assert!(!checked.canceled);
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#check').checked)")
                .expect("checked"),
            "true"
        );
        assert_eq!(
            worker
                .execute_script_direct("globalThis.__events.join(',')")
                .expect("events"),
            "click:true,input:true,change:true"
        );

        worker
            .execute_script_direct("globalThis.__cancelClick=true")
            .expect("cancel click");
        let canceled = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#check",
            zero_page_runtime::HtmlUserAction::Activate,
        )
        .expect("checkbox plan");
        assert!(canceled.canceled);
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#check').checked)")
                .expect("rolled back"),
            "true"
        );
        assert_eq!(
            worker
                .execute_script_direct("globalThis.__events.join(',')")
                .expect("events"),
            "click:true,input:true,change:true,click:false"
        );
        let disabled = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#disabled",
            zero_page_runtime::HtmlUserAction::Activate,
        )
        .expect("disabled checkbox");
        assert_eq!(
            disabled.noop_reason,
            Some(zero_page_runtime::ActionNoopReason::DisabledTarget)
        );
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#disabled').checked)")
                .expect("disabled checkedness"),
            "false"
        );

        let radio = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#pro",
            zero_page_runtime::HtmlUserAction::Activate,
        )
        .expect("radio plan");
        assert!(!radio.canceled);
        assert_eq!(
            worker
                .execute_script_direct(
                    "[document.querySelector('#basic').checked,\
                      document.querySelector('#pro').checked].join(',')"
                )
                .expect("radio state"),
            "false,true"
        );

        let dirty = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#name",
            zero_page_runtime::HtmlUserAction::InsertText {
                text: "dirty".to_string(),
            },
        )
        .expect("dirty input");
        assert!(dirty.changed);
        let reset_result = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#reset",
            zero_page_runtime::HtmlUserAction::Reset,
        )
        .expect("reset");
        assert!(reset_result.changed);
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__resetValue)")
                .expect("reset microtask"),
            "base"
        );

        let dirty = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#name",
            zero_page_runtime::HtmlUserAction::InsertText {
                text: "dirty".to_string(),
            },
        )
        .expect("dirty input again");
        assert!(dirty.changed);
        worker
            .execute_script_direct("globalThis.__cancelReset=true")
            .expect("cancel reset");
        let canceled_reset = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#reset",
            zero_page_runtime::HtmlUserAction::Reset,
        )
        .expect("canceled reset");
        assert!(canceled_reset.canceled);
        assert_eq!(
            worker
                .execute_script_direct("document.querySelector('#name').value")
                .expect("dirty value"),
            "basedirty"
        );

        let submitted = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#submit",
            zero_page_runtime::HtmlUserAction::Submit,
        )
        .expect("submit");
        assert_eq!(
            action_navigation(&submitted),
            Some((
                "https://zero.test/submitted?name=listener&plan=basic&go=1".to_string(),
                "GET".to_string(),
                None,
            ))
        );
        worker
            .execute_script_direct("globalThis.__cancelSubmit=true")
            .expect("cancel submit");
        let canceled_submit = execute_shared_action(
            &mut wv,
            Some(&worker),
            true,
            "#submit",
            zero_page_runtime::HtmlUserAction::Submit,
        )
        .expect("canceled submit");
        assert!(canceled_submit.canceled);
        assert!(action_navigation(&canceled_submit).is_none());

        worker.shutdown();
    }

    #[test]
    fn shared_actions_keep_ua_defaults_when_javascript_is_disabled() {
        let html = r#"<html><body>
            <form id="form" action="https://zero.test/submitted">
              <input id="name" name="name" value="base">
              <input id="check" name="check" type="checkbox">
              <button id="reset" type="reset">Reset</button>
              <button id="submit" type="submit" name="go" value="1">Submit</button>
            </form>
        </body></html>"#;
        let url = "https://zero.test/form";
        let mut worker = TabJsWorkerHandle::spawn(TabId(903));
        worker.set_dom_snapshot(html, url);
        worker
            .execute_script_direct(
                "globalThis.__events=[];\
                 document.querySelector('#name').addEventListener('beforeinput',function(event){\
                   globalThis.__events.push('beforeinput');event.preventDefault();\
                 });\
                 document.querySelector('#check').addEventListener('click',function(event){\
                   globalThis.__events.push('click');event.preventDefault();\
                 });\
                 document.querySelector('#form').addEventListener('reset',function(event){\
                   globalThis.__events.push('reset');event.preventDefault();\
                 });\
                 document.querySelector('#form').addEventListener('submit',function(event){\
                   globalThis.__events.push('submit');event.preventDefault();\
                 });",
            )
            .expect("register listeners");

        let mut wv = WebView::new(WebViewConfig::default());
        wv.prepare_document_state(url);
        wv.load_html(html, None);
        assert!(
            execute_shared_action(
                &mut wv,
                Some(&worker),
                false,
                "#name",
                zero_page_runtime::HtmlUserAction::InsertText { text: "X".to_string() },
            )
            .expect("insert")
            .changed
        );
        assert!(
            execute_shared_action(
                &mut wv,
                Some(&worker),
                false,
                "#check",
                zero_page_runtime::HtmlUserAction::Activate,
            )
            .expect("activate")
            .changed
        );
        assert!(
            execute_shared_action(
                &mut wv,
                Some(&worker),
                false,
                "#reset",
                zero_page_runtime::HtmlUserAction::Reset,
            )
            .expect("reset")
            .changed
        );
        let submitted = execute_shared_action(
            &mut wv,
            Some(&worker),
            false,
            "#submit",
            zero_page_runtime::HtmlUserAction::Submit,
        )
        .expect("submit");

        assert_eq!(
            worker
                .execute_script_direct(
                    "[document.querySelector('#name').value,\
                      document.querySelector('#check').checked,\
                      globalThis.__events.join(',')].join('|')"
                )
                .expect("state"),
            "base|false|"
        );
        assert_eq!(
            action_navigation(&submitted),
            Some((
                "https://zero.test/submitted?name=base&go=1".to_string(),
                "GET".to_string(),
                None,
            ))
        );

        worker.shutdown();
    }

    #[test]
    fn pointer_selection_uses_utf16_paint_boundary() {
        let html = r#"<html><body><input id="name" value="i中😀W"></body></html>"#;
        let url = "https://zero.test/pointer-selection";
        let mut worker = TabJsWorkerHandle::spawn(TabId(904));
        worker.set_dom_snapshot(html, url);
        let mut wv = WebView::new(WebViewConfig::default());
        wv.prepare_document_state(url);
        wv.load_html(html, None);

        set_shared_text_selection(&mut wv, Some(&worker), "#name", Some((2, 2)));
        assert!(
            execute_shared_action(
                &mut wv,
                Some(&worker),
                true,
                "#name",
                zero_page_runtime::HtmlUserAction::InsertText { text: "X".to_string() },
            )
            .expect("insert")
            .changed
        );
        assert_eq!(
            worker
                .execute_script_direct("document.getElementById('name').value")
                .expect("value"),
            "i中X😀W"
        );

        worker.shutdown();
    }
}
