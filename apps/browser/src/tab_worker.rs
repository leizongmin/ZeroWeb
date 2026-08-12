//! 标签页渲染 worker — 每个 Tab 独立 OS 线程，持有 WebView 与异步加载状态。

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_engine::{set_char_measure_fn, set_text_shape_fn};
use zero_render_foundation::font::loader::FontLoader;
use zero_webview::{AsyncPageLoad, InProcessFetchHost, PageLoadStage, WebView, WebViewBuilder, WebViewConfig};

use crate::pages;
use crate::tab_js_worker::TabJsWorkerHandle;
use crate::tab_scripts;
use crate::tab_snapshot::TabSnapshot;
use crate::text_metrics;

/// 每帧在 worker 内推进加载/渲染的时间预算（毫秒）。
pub const TAB_WORKER_FRAME_BUDGET_MS: f64 = 8.0;

/// 发送给 Tab worker 的命令。
pub enum TabWorkerCommand {
    /// 导航到 URL（异步分阶段加载）。
    Navigate(String),
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
            // P1b S3 / R2923：注入生产 fetch handler（经 zero_net::HttpClient::send 真实 HTTP，
            // 支持全方法/头/体）。js_worker 早于 WebView 创建，但 HttpClient::new() 自建 reqwest
            // client，无需 WebView 句柄，故可在 spawn 后立即注入。
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
                } => {
                    let html = wv.html_content().to_string();
                    let detail = if key.is_some() || code.is_some() {
                        Some(zero_engine::DomEventDetail {
                            key,
                            code,
                            ..Default::default()
                        })
                    } else {
                        None
                    };
                    let result = tab_scripts::dispatch_dom_event(
                        &mut wv,
                        javascript_enabled,
                        _js_worker.as_ref(),
                        &selector,
                        &event_type,
                        &html,
                        detail.as_ref(),
                    );
                    if result.html_changed {
                        push_snapshot(&wv, &msg_tx, _js_worker.as_ref());
                    }
                    let _ = msg_tx.send(TabWorkerMessage::DispatchResult {
                        dispatch_id,
                        default_allowed: result.default_allowed,
                        html_changed: result.html_changed,
                    });
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
                            wv.execute_script(&script)
                                .map(|_| ())
                                .map_err(|e| e.to_string())
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
                        js_worker
                            .execute_script_direct(&script)
                            .map_err(|e| e.to_string())
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
                    for (family, weight, is_italic, features, bytes) in loaded {
                        match font_loader.load_font(&bytes) {
                            Ok(id) => {
                                font_loader.register_font_features(id, features);
                                // R2417/R2493：按 (weight, style) 构注册键——bold+italic →
                                // `{family}:700:italic`、bold → `{family}:700`、italic →
                                // `{family}:italic`、regular → plain。bold/italic face 不注册
                                // plain family（避 build_font_resolver「second face=bold」启发式
                                // 顺序错配，R2417）。
                                let want_bold = weight.is_some_and(|w| w >= 600);
                                let key = match (want_bold, is_italic) {
                                    (true, true) => format!("{family}:700:italic"),
                                    (true, false) => format!("{family}:700"),
                                    (false, true) => format!("{family}:italic"),
                                    (false, false) => family.clone(),
                                };
                                font_loader.register_family_alias(&key, id);
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
                    // 注入 runner——finish() 在页面 load 之后派发对应 window 'error'（onerror handler 已就位）。
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
                    // R2943：drain img 元素级 load/error 事件——finish() 经 __zw_dispatch_img_event 派发到
                    // 匹配 src 的 <img> 元素（img.onload/onerror）。
                    let img_events = load.take_img_element_events();
                    if !img_events.is_empty() {
                        if let Some(r) = page_script_runner.as_mut() {
                            r.set_img_events(img_events);
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
