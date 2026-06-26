//! 标签页渲染 worker — 每个 Tab 独立 OS 线程，持有 WebView 与异步加载状态。

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_engine::set_char_measure_fn;
use zero_render_foundation::font::loader::FontLoader;
use zero_webview::{AsyncPageLoad, PageLoadStage, WebView, WebViewBuilder, WebViewConfig};

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
    /// 链接命中测试。
    HitTestLink {
        x: f32,
        y: f32,
        reply: Sender<Option<String>>,
    },
    /// 关闭 worker。
    Shutdown,
}

/// Worker 发往 UI 线程的消息。
pub enum TabWorkerMessage {
    /// 快照更新。
    Snapshot(TabSnapshot),
    /// 页面标题。
    Title(String),
    /// 加载失败。
    LoadError(String),
    /// 加载阶段变化。
    Stage(PageLoadStage),
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

    /// 同步命中测试链接。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let (tx, rx) = mpsc::channel();
        self.send(TabWorkerCommand::HitTestLink { x, y, reply: tx });
        rx.recv_timeout(Duration::from_millis(50)).ok().flatten()
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
    let mut font_loader = FontLoader::new();
    let font_id = load_system_fonts_worker(&mut font_loader);

    let mut wv = WebViewBuilder::new().width(viewport.0).height(viewport.1).build();
    wv.set_prefers_color_scheme(color_scheme);
    let _ = WebViewConfig::default();

    let mut async_load: Option<AsyncPageLoad> = None;
    let mut pending_sync_html: Option<(String, Option<String>, Option<String>)> = None;

    let push_snapshot = |wv: &WebView, msg_tx: &Sender<TabWorkerMessage>| {
        let _ = msg_tx.send(TabWorkerMessage::Snapshot(TabSnapshot::from_webview(wv)));
    };

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                TabWorkerCommand::Navigate(url) => {
                    tracing::info!("Tab {} navigate: {url}", tab_id.0);
                    wv.prepare_document_state(&url);
                    async_load = Some(AsyncPageLoad::start(url));
                    pending_sync_html = None;
                }
                TabWorkerCommand::LoadHtml { html, css, url } => {
                    pending_sync_html = Some((html, css, url));
                    async_load = None;
                }
                TabWorkerCommand::Resize { width, height } => {
                    with_measure(&font_loader, font_id, || wv.resize(width, height));
                    if wv.last_render().is_some() {
                        with_measure(&font_loader, font_id, || {
                            wv.render();
                        });
                    }
                    push_snapshot(&wv, &msg_tx);
                }
                TabWorkerCommand::SetColorScheme(scheme) => {
                    wv.set_prefers_color_scheme(scheme);
                    if wv.last_render().is_some() {
                        with_measure(&font_loader, font_id, || {
                            wv.render();
                        });
                        push_snapshot(&wv, &msg_tx);
                    }
                }
                TabWorkerCommand::HitTestLink { x, y, reply } => {
                    let href = wv.hit_test_link(x, y);
                    let _ = reply.send(href);
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
            push_snapshot(&wv, &msg_tx);
        }

        if let Some(ref mut load) = async_load {
            let prev_stage = load.stage();
            let changed = with_measure(&font_loader, font_id, || load.tick(&mut wv, TAB_WORKER_FRAME_BUDGET_MS));
            if changed {
                if load.stage() != prev_stage {
                    let _ = msg_tx.send(TabWorkerMessage::Stage(load.stage()));
                }
                push_snapshot(&wv, &msg_tx);
                if let Some(title) = wv.title() {
                    let _ = msg_tx.send(TabWorkerMessage::Title(title.to_string()));
                }
            }
            if !load.is_active() {
                async_load = None;
                if let Some(title) = wv.title() {
                    let _ = msg_tx.send(TabWorkerMessage::Title(title.to_string()));
                }
                push_snapshot(&wv, &msg_tx);
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
    crate::app::load_system_fonts(loader)
}
