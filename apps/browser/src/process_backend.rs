//! 可选多进程 Tab 后端 — 通过 `ProcessManager` 将页面渲染隔离到 `zero-renderer` 子进程。
//!
//! 网络请求由本进程代理（Chromium 式 browser-hosted network）；渲染进程仅通过 `FetchRequest` IPC 访问网络。

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_protocol::ProtocolError;
use zero_protocol::message::{
    DispatchDomEventParams, FetchParams, IpcColorScheme, IpcMediaType, IpcMessage, IpcMessageKind, LoadHtmlParams,
    SetColorSchemeParams, SetMediaTypeParams, SetViewportParams, StorageOpParams, StorageOperation, StorageType,
};
use zero_protocol::process::{ProcessManager, RendererHandle};
use zero_storage::StorageManager;

use crate::fetch_proxy::TabFetchProxy;
use crate::tab_snapshot::TabSnapshot;

/// 是否启用多进程后端（环境变量 `ZERO_BROWSER_MULTIPROCESS`；默认启用）。
pub fn use_multiprocess_backend() -> bool {
    match std::env::var("ZERO_BROWSER_MULTIPROCESS") {
        Ok(v) => v != "0" && !v.eq_ignore_ascii_case("false"),
        Err(_) => true,
    }
}

/// 供 CLI 在解析参数后强制启用多进程。
pub fn set_multiprocess_enabled(enabled: bool) {
    // SAFETY: 在 main 启动早期、单线程环境下设置进程环境变量。
    unsafe {
        std::env::set_var("ZERO_BROWSER_MULTIPROCESS", if enabled { "1" } else { "0" });
    }
}

fn renderer_binary_filename() -> &'static str {
    #[cfg(windows)]
    {
        "zero-renderer.exe"
    }
    #[cfg(not(windows))]
    {
        "zero-renderer"
    }
}

/// 解析 `zero-renderer` 可执行文件路径。
///
/// 发布布局要求 **`zero-renderer` 与 `zero-browser` 同目录**（安装包 / 构建脚本负责保持这一布局）。
/// 查找顺序：
/// 1. `ZERO_RENDERER_PATH` 环境变量
/// 2. `std::env::current_exe()` 所在目录
/// 3. `PATH`（系统级安装等兜底）
fn resolve_renderer_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("ZERO_RENDERER_PATH") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
        tracing::warn!("ZERO_RENDERER_PATH 指向的文件不存在: {}", candidate.display());
    }

    if let Some(sibling) = renderer_binary_beside_current_exe() {
        return Some(sibling);
    }

    find_renderer_in_path()
}

/// 在 **当前进程可执行文件** 所在目录查找 `zero-renderer`。
fn renderer_binary_beside_current_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let sibling = exe.parent()?.join(renderer_binary_filename());
    sibling.is_file().then_some(sibling)
}

fn find_renderer_in_path() -> Option<PathBuf> {
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let candidate = dir.join(renderer_binary_filename());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 多进程 Tab 后端。
pub struct ProcessTabBackend {
    manager: ProcessManager,
    tab_to_renderer: HashMap<TabId, u64>,
    renderer_bin: PathBuf,
    viewport: (u32, u32),
    storage: StorageManager,
    pending_loaded: Vec<(TabId, String, String)>,
    pending_errors: Vec<(TabId, String)>,
    fetch_proxy: TabFetchProxy,
    /// 异步 DOM 事件派发的回执（按 dispatch id 收集，由 TabManager 消费）。
    pending_dispatch_results: Vec<(u64, bool)>,
}

impl ProcessTabBackend {
    /// 创建多进程后端；若找不到 `zero-renderer` 则返回 `None`（由调用方回退单进程 worker）。
    pub fn try_new() -> Option<Self> {
        let renderer_bin = resolve_renderer_binary().unwrap_or_else(|| PathBuf::from(renderer_binary_filename()));
        if !renderer_bin.is_file() {
            let expected = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.join(renderer_binary_filename())))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| renderer_binary_filename().to_string());
            tracing::warn!(
                "未找到 zero-renderer（应与 zero-browser 同目录: {expected}）。\
                 将使用进程内 tab worker。请一并安装/编译两个二进制，或设置 ZERO_RENDERER_PATH，或使用 --single-process。"
            );
            return None;
        }
        tracing::info!("Multi-process renderer binary: {}", renderer_bin.display());
        Some(Self::with_renderer_bin(renderer_bin))
    }

    fn with_renderer_bin(renderer_bin: PathBuf) -> Self {
        Self {
            manager: ProcessManager::new(renderer_bin.to_string_lossy().as_ref()),
            tab_to_renderer: HashMap::new(),
            renderer_bin,
            viewport: (800, 600),
            storage: StorageManager::new(),
            pending_loaded: Vec::new(),
            pending_errors: Vec::new(),
            fetch_proxy: TabFetchProxy::new(),
            pending_dispatch_results: Vec::new(),
        }
    }

    fn send_fetch_response_now(&mut self, tab_id: TabId, request_id: u64, status: u16, body: Vec<u8>) {
        if let Some(renderer) = self.renderer_mut(tab_id) {
            if let Err(e) = renderer.send_fetch_response(request_id, status, Vec::new(), body) {
                tracing::warn!("FetchResponse send failed tab {}: {e}", tab_id.0);
            }
        } else {
            tracing::warn!("FetchResponse dropped: no renderer for tab {}", tab_id.0);
        }
    }

    fn renderer_mut(&mut self, tab_id: TabId) -> Option<&mut RendererHandle> {
        let id = self.tab_to_renderer.get(&tab_id).copied()?;
        self.manager.get_renderer(id)
    }

    fn tab_for_renderer(&self, rid: u64) -> Option<TabId> {
        self.tab_to_renderer.iter().find(|(_, r)| **r == rid).map(|(t, _)| *t)
    }

    fn apply_inbound_message(
        tab_id: TabId,
        snap: &mut TabSnapshot,
        kind: IpcMessageKind,
        pending_loaded: &mut Vec<(TabId, String, String)>,
        pending_errors: &mut Vec<(TabId, String)>,
    ) {
        match kind {
            IpcMessageKind::TitleChanged(title) => snap.title = Some(title),
            IpcMessageKind::LoadComplete => {
                // loading 结束与 paint 提交仅由 ViewPainted / LoadFailed 驱动。
            }
            IpcMessageKind::LoadFailed(err) => {
                snap.loading = false;
                pending_errors.push((tab_id, err.clone()));
                tracing::warn!("Renderer load failed: {err}");
            }
            IpcMessageKind::UrlChanged(url) => snap.url = Some(url),
            IpcMessageKind::ViewPainted(params) => {
                // C2 接线：env ZW_COMPOSITOR_PROCESS=1 时同步转发帧到合成器进程
                // （默认关 = 零行为变更；失败静默不阻断主通路）
                if crate::compositor_client::enabled() {
                    crate::compositor_client::forward_frame((*params).clone());
                }
                if params.navigation_epoch != snap.navigation_epoch {
                    tracing::debug!(
                        "忽略 stale ViewPainted tab {} epoch {} != {}",
                        tab_id.0,
                        params.navigation_epoch,
                        snap.navigation_epoch
                    );
                    return;
                }
                crate::paint_ipc::apply_paint_snapshot(snap, *params);
                // 首帧绘制到达即可结束 loading（脚本预取/执行可能仍在进行）。
                if snap.loading {
                    snap.loading = false;
                    let title = snap.title.clone().unwrap_or_else(|| "页面".to_string());
                    let url = snap.url.clone().unwrap_or_default();
                    pending_loaded.push((tab_id, title, url));
                }
            }
            _ => {}
        }
    }

    fn handle_fetch_request(&mut self, tab_id: TabId, params: FetchParams) {
        self.fetch_proxy.enqueue(tab_id, &params);
    }

    fn drain_pending_fetches(&mut self) {
        for item in self.fetch_proxy.drain() {
            self.send_fetch_response_now(item.tab_id, item.request_id, item.status, item.body);
        }
    }

    /// 导航并在本线程轮询 IPC，直到该 Tab 加载完成/失败或超时（测试/同步场景用）。
    ///
    /// 正常运行时由事件循环 `poll` 驱动 fetch 代理；勿在主/UI 线程调用以免阻塞 winit。
    pub fn navigate_and_service(&mut self, tab_id: TabId, url: &str, snapshots: &mut HashMap<TabId, TabSnapshot>) {
        let epoch = snapshots.get(&tab_id).map(|s| s.navigation_epoch).unwrap_or(0);
        self.navigate(tab_id, url, epoch);
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            self.poll(snapshots, Some(tab_id), true);
            if self.pending_loaded.iter().any(|(t, _, _)| *t == tab_id)
                || self.pending_errors.iter().any(|(t, _)| *t == tab_id)
            {
                return;
            }
            if Instant::now() >= deadline {
                tracing::warn!("navigate_and_service timeout tab {} url {url}", tab_id.0);
                return;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn handle_storage_op(&mut self, tab_id: TabId, params: StorageOpParams) {
        let store = match params.storage_type {
            StorageType::Local => self.storage.local_storage(&params.origin),
            StorageType::Session => self.storage.session_storage(&params.origin),
        };
        let _result = match params.operation {
            StorageOperation::Get => store.get(&params.key).map(|s| s.to_string()),
            StorageOperation::Set => params
                .value
                .as_deref()
                .and_then(|v| store.set(&params.key, v).ok())
                .flatten(),
            StorageOperation::Remove => store.remove(&params.key),
            StorageOperation::Clear => {
                store.clear();
                None
            }
            StorageOperation::Length => Some(store.len().to_string()),
            StorageOperation::Key => store.key(params.key.parse().unwrap_or(0)).map(|s| s.to_string()),
        };
        let _ = tab_id;
        // 渲染进程当前不等待 Storage 响应；后续可扩展 StorageResponse IPC。
    }

    fn handle_crashes(&mut self, snapshots: &mut HashMap<TabId, TabSnapshot>) {
        let crashed = self.manager.check_crashes();
        for (rid, reason) in crashed {
            let Some(tab_id) = self.tab_for_renderer(rid) else {
                continue;
            };
            self.handle_renderer_ipc_lost(tab_id, rid, snapshots, &reason);
        }
    }

    /// IPC 读端断开或子进程退出 — 清理 Tab ↔ renderer 映射并停止向该进程发消息。
    fn handle_renderer_ipc_lost(
        &mut self,
        tab_id: TabId,
        rid: u64,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        reason: &str,
    ) {
        if self.tab_to_renderer.get(&tab_id) == Some(&rid) {
            self.tab_to_renderer.remove(&tab_id);
        }
        self.fetch_proxy.remove_tab(tab_id);
        let _ = self.manager.shutdown_renderer(rid);
        if !self.pending_errors.iter().any(|(t, _)| *t == tab_id) {
            self.pending_errors
                .push((tab_id, format!("渲染进程连接已断开: {reason}")));
        }
        if let Some(snap) = snapshots.get_mut(&tab_id) {
            snap.loading = false;
        }
        tracing::info!("Renderer {rid} for tab {} disconnected: {reason}", tab_id.0);
    }

    fn ipc_recv_disconnected(err: &ProtocolError) -> bool {
        err.is_disconnected() || format!("{err}").contains("IPC 通道已关闭")
    }

    fn send_to_renderer(&mut self, tab_id: TabId, kind: IpcMessageKind) {
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.send(IpcMessage { id: 0, kind }) {
            tracing::warn!("IPC send failed for tab {}: {e}", tab_id.0);
        }
    }

    /// 取出待处理的加载完成事件。
    pub fn take_page_loaded_events(&mut self) -> Vec<(TabId, String, String)> {
        std::mem::take(&mut self.pending_loaded)
    }

    /// 取出待处理的加载失败事件。
    pub fn take_page_error_events(&mut self) -> Vec<(TabId, String)> {
        std::mem::take(&mut self.pending_errors)
    }

    /// 确保 Tab 有对应渲染进程。
    pub fn ensure_renderer(&mut self, tab_id: TabId, viewport: (u32, u32)) {
        self.viewport = viewport;
        if let Some(rid) = self.tab_to_renderer.get(&tab_id).copied() {
            if self.manager.get_renderer(rid).is_some() {
                return;
            }
            self.tab_to_renderer.remove(&tab_id);
        }
        match self.manager.spawn_renderer() {
            Ok(rid) => {
                self.tab_to_renderer.insert(tab_id, rid);
                tracing::info!("Spawned renderer {rid} for tab {}", tab_id.0);
                self.send_to_renderer(
                    tab_id,
                    IpcMessageKind::SetViewport(SetViewportParams {
                        width: viewport.0,
                        height: viewport.1,
                    }),
                );
            }
            Err(e) => {
                tracing::error!("Failed to spawn renderer for tab {}: {e}", tab_id.0);
                self.pending_errors.push((tab_id, format!("无法启动渲染进程: {e}")));
            }
        }
    }

    /// 关闭 Tab 对应渲染进程。
    pub fn remove_renderer(&mut self, tab_id: TabId) {
        self.fetch_proxy.remove_tab(tab_id);
        if let Some(rid) = self.tab_to_renderer.remove(&tab_id) {
            let _ = self.manager.shutdown_renderer(rid);
        }
    }

    /// 标记 Tab 为无痕（fetch 使用仅内存缓存）。
    pub fn set_tab_private(&mut self, tab_id: TabId, private: bool) {
        self.fetch_proxy.set_tab_private(tab_id, private);
    }

    /// Tab 是否仍有 live 渲染进程。
    pub fn has_renderer(&self, tab_id: TabId) -> bool {
        self.tab_to_renderer.contains_key(&tab_id)
    }

    /// 当前 live 渲染进程数量。
    pub fn live_renderer_count(&self) -> usize {
        self.tab_to_renderer.len()
    }

    /// 导航。
    pub fn navigate(&mut self, tab_id: TabId, url: &str, navigation_epoch: u64) {
        self.fetch_proxy.on_navigate(tab_id, url);
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.navigate(url, None, navigation_epoch) {
            tracing::warn!("IPC navigate failed: {e}");
        }
    }

    /// 强制刷新前清除该 URL 的缓存条目（绕过 HTTP 缓存）。
    pub fn invalidate_url_cache(&self, tab_id: TabId, url: &str) {
        self.fetch_proxy.invalidate_url(tab_id, url);
    }

    /// 后退（IPC）。
    pub fn go_back(&mut self, tab_id: TabId) {
        self.send_to_renderer(tab_id, IpcMessageKind::GoBack);
    }

    /// 前进（IPC）。
    pub fn go_forward(&mut self, tab_id: TabId) {
        self.send_to_renderer(tab_id, IpcMessageKind::GoForward);
    }

    /// 加载 HTML（多进程 IPC）。
    pub fn load_html(
        &mut self,
        tab_id: TabId,
        html: &str,
        css: Option<&str>,
        url: Option<&str>,
        navigation_epoch: u64,
    ) {
        self.send_to_renderer(
            tab_id,
            IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: css.map(str::to_string),
                url: url.map(str::to_string),
                navigation_epoch,
            }),
        );
    }

    /// 调整所有 live 渲染进程视口。
    pub fn resize_all(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(tab_id, IpcMessageKind::SetViewport(SetViewportParams { width, height }));
        }
    }

    /// 广播颜色方案到所有 live 渲染进程。
    pub fn set_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        let ipc_scheme = match scheme {
            PrefersColorSchemeValue::Light => IpcColorScheme::Light,
            PrefersColorSchemeValue::Dark => IpcColorScheme::Dark,
        };
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(
                tab_id,
                IpcMessageKind::SetColorScheme(SetColorSchemeParams { scheme: ipc_scheme }),
            );
        }
    }

    /// 广播渲染媒体类型到所有 live 渲染进程（DC-12 @media print；R1993）。
    pub fn set_media_type(&mut self, media_type: zero_engine::MediaType) {
        let ipc_media = match media_type {
            zero_engine::MediaType::Screen => IpcMediaType::Screen,
            zero_engine::MediaType::Print => IpcMediaType::Print,
            _ => IpcMediaType::Screen, // All/其他按 Screen 回退（打印预览仅 Screen/Print）
        };
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(
                tab_id,
                IpcMessageKind::SetMediaType(SetMediaTypeParams { media_type: ipc_media }),
            );
        }
    }

    /// 轮询 IPC 并更新快照；后台 Tab 可降频。
    /// 每帧 IPC 轮询时间预算（毫秒）。超过即提前返回，剩余消息留到下一帧，
    /// 避免单个 renderer 持续吐 `ViewPainted` 把 UI 主线程吃满。
    const POLL_BUDGET: Duration = Duration::from_millis(4);

    pub fn poll(
        &mut self,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        _active_tab: Option<TabId>,
        _poll_background: bool,
    ) -> bool {
        self.drain_pending_fetches();
        let mut changed = false;
        self.handle_crashes(snapshots);
        let mapping: Vec<(TabId, u64)> = self.tab_to_renderer.iter().map(|(k, v)| (*k, *v)).collect();
        let mut disconnected = Vec::new();
        let poll_deadline = Instant::now() + Self::POLL_BUDGET;
        for (tab_id, rid) in mapping {
            if Instant::now() >= poll_deadline {
                break;
            }
            loop {
                let msg = {
                    let Some(renderer) = self.manager.get_renderer(rid) else {
                        break;
                    };
                    match renderer.try_recv() {
                        Ok(Some(m)) => m,
                        Ok(None) => break,
                        Err(e) => {
                            if Self::ipc_recv_disconnected(&e) {
                                disconnected.push((tab_id, rid, format!("{e}")));
                            } else {
                                tracing::debug!("IPC recv: {e}");
                            }
                            break;
                        }
                    }
                };
                changed = true;
                match msg.kind {
                    IpcMessageKind::FetchRequest(params) => {
                        self.handle_fetch_request(tab_id, params);
                    }
                    IpcMessageKind::StorageOp(params) => {
                        self.handle_storage_op(tab_id, params);
                    }
                    IpcMessageKind::DispatchDomEventResult(result) => {
                        self.pending_dispatch_results.push((msg.id, result.default_allowed));
                    }
                    kind => {
                        let snap = snapshots.entry(tab_id).or_default();
                        Self::apply_inbound_message(
                            tab_id,
                            snap,
                            kind,
                            &mut self.pending_loaded,
                            &mut self.pending_errors,
                        );
                    }
                }
                if Instant::now() >= poll_deadline {
                    break;
                }
            }
        }
        for (tab_id, rid, reason) in disconnected {
            self.handle_renderer_ipc_lost(tab_id, rid, snapshots, &reason);
            changed = true;
        }
        changed
    }

    /// 取出异步 DOM 事件派发的回执（dispatch id, default_allowed）。
    pub fn take_dispatch_results(&mut self) -> Vec<(u64, bool)> {
        std::mem::take(&mut self.pending_dispatch_results)
    }

    /// 异步派发 DOM 事件（fire-and-forget）。
    ///
    /// 发出 IPC 后立即返回；渲染进程的 `DispatchDomEventResult` 会在后续 `poll` 里被收集，
    /// 由 `TabManager` 根据回执执行延迟的默认动作（链接导航）。
    /// 这消除了原来“主线程 busy-wait 等 renderer 响应最多 500ms”的卡顿来源。
    pub fn dispatch_dom_event_fire_and_forget(
        &mut self,
        tab_id: TabId,
        dispatch_id: u64,
        selector: Option<&str>,
        event_type: &str,
        key: Option<String>,
        code: Option<String>,
    ) {
        let Some(rid) = self.tab_to_renderer.get(&tab_id).copied() else {
            // 渲染进程不存在：模拟一个“默认允许”回执，让 TabManager 走默认动作路径。
            self.pending_dispatch_results.push((dispatch_id, true));
            return;
        };
        let params = DispatchDomEventParams {
            selector: selector.map(str::to_string),
            // 命中坐标由渲染进程内部 hit-test 决定（基于 selector），主线程不再传 x/y。
            x: 0.0,
            y: 0.0,
            event_type: event_type.to_string(),
            key,
            code,
        };
        let Some(renderer) = self.manager.get_renderer(rid) else {
            self.pending_dispatch_results.push((dispatch_id, true));
            return;
        };
        if renderer
            .send(IpcMessage {
                id: dispatch_id,
                kind: IpcMessageKind::DispatchDomEvent(params),
            })
            .is_err()
        {
            self.pending_dispatch_results.push((dispatch_id, true));
        }
    }
}

impl ProcessTabBackend {
    /// 显式关闭所有渲染子进程。
    ///
    /// `std::process::exit` 会跳过 `Drop`，因此主进程的所有最终退出路径
    /// （窗口关闭按钮、Ctrl+C、事件循环错误）都必须显式调用此方法，
    /// 否则 `zero-renderer.exe` 子进程会成为孤儿，持有自身可执行文件句柄，
    /// 导致下次 `cargo build` 时无法覆盖二进制（Windows `os error 5`）。
    pub fn shutdown_all(&mut self) {
        self.manager.shutdown_all();
    }
}

impl Drop for ProcessTabBackend {
    fn drop(&mut self) {
        self.manager.shutdown_all();
    }
}

#[cfg(test)]
mod navigation_contract_tests {
    use super::ProcessTabBackend;
    use crate::paint_ipc::apply_paint_snapshot;
    use crate::tab_snapshot::TabSnapshot;
    use zero_browser_shell::TabId;
    use zero_protocol::message::IpcMessageKind;
    use zero_protocol::{IpcColor, IpcFill, IpcRect, PaintSnapshotParams};
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
    use zero_webview::WebViewRenderResult;

    fn paint_with_red_fill(epoch: u64) -> PaintSnapshotParams {
        PaintSnapshotParams {
            viewport_width: 800,
            viewport_height: 600,
            document_height: 400.0,
            navigation_epoch: epoch,
            fills: vec![IpcFill {
                rect: IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                color: IpcColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            }],
            rounded_rects: vec![],
            gradients: vec![],
            shadows: vec![],
            images: vec![],
            image_payloads: vec![],
            strokes: vec![],
            path_fills: vec![],
            path_strokes: vec![],
            clips: vec![],
            transforms: vec![],
            filters: vec![],
            blend_modes: vec![],
            glyphs: vec![],
            draw_order: vec![],
            hit_test: None,
        }
    }

    fn legacy_blue_render() -> WebViewRenderResult {
        WebViewRenderResult {
            primitives: RenderPrimitives {
                fills: vec![FillPrimitive {
                    rect: Rect::new(0.0, 0.0, 50.0, 50.0),
                    color: Color::rgb(0, 0, 255),
                }],
                ..RenderPrimitives::new()
            },
            timings: Default::default(),
        }
    }

    #[test]
    fn begin_navigation_discards_previous_paint() {
        let mut snap = TabSnapshot {
            last_render: Some(legacy_blue_render()),
            loading: false,
            ..Default::default()
        };

        snap.begin_navigation("https://example.com".into());

        assert!(snap.last_render.is_none());
        assert!(snap.loading);
        assert!(!snap.should_composite_paint());
        assert_eq!(snap.url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn view_painted_commits_new_frame_and_ends_loading() {
        let tab_id = TabId(1);
        let mut snap = TabSnapshot::default();
        snap.begin_navigation("https://example.com".into());
        let mut pending_loaded = Vec::new();
        let mut pending_errors = Vec::new();

        let epoch = snap.navigation_epoch;
        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(epoch))),
            &mut pending_loaded,
            &mut pending_errors,
        );

        assert!(!snap.loading);
        assert!(snap.should_composite_paint());
        let fill = &snap.last_render.as_ref().unwrap().primitives.fills[0];
        assert_eq!(fill.color.r, 255);
        assert_eq!(fill.color.g, 0);
        assert_eq!(pending_loaded.len(), 1);
        assert!(pending_errors.is_empty());
    }

    #[test]
    fn load_complete_alone_does_not_end_loading_or_change_paint() {
        let tab_id = TabId(1);
        let mut snap = TabSnapshot::default();
        snap.begin_navigation("https://example.com".into());
        snap.last_render = None;
        let mut pending_loaded = Vec::new();
        let mut pending_errors = Vec::new();

        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::LoadComplete,
            &mut pending_loaded,
            &mut pending_errors,
        );

        assert!(snap.loading);
        assert!(!snap.should_composite_paint());
        assert!(pending_loaded.is_empty());
    }

    #[test]
    fn stale_view_painted_is_ignored() {
        let tab_id = TabId(1);
        let mut snap = TabSnapshot::default();
        snap.begin_navigation("https://example.com".into());
        let epoch = snap.navigation_epoch;
        let mut pending_loaded = Vec::new();
        let mut pending_errors = Vec::new();

        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(epoch.wrapping_sub(1)))),
            &mut pending_loaded,
            &mut pending_errors,
        );

        assert!(snap.loading);
        assert!(snap.last_render.is_none());
        assert!(pending_loaded.is_empty());
    }

    #[test]
    fn apply_paint_snapshot_replaces_legacy_blue_with_red() {
        let mut snap = TabSnapshot {
            last_render: Some(legacy_blue_render()),
            ..Default::default()
        };
        apply_paint_snapshot(&mut snap, paint_with_red_fill(0));
        let fill = &snap.last_render.as_ref().unwrap().primitives.fills[0];
        assert_eq!(fill.color.r, 255);
    }
}
