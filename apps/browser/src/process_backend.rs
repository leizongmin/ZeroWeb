//! 可选多进程 Tab 后端 — 通过 `ProcessManager` 将页面渲染隔离到 `zero-renderer` 子进程。
//!
//! 网络请求由本进程代理（Chromium 式 browser-hosted network）；渲染进程仅通过 `FetchRequest` IPC 访问网络。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_engine::{DomEventDetail, ElementHit};
use zero_protocol::message::{
    DispatchDomEventParams, FetchParams, HitTestElementResultParams, HitTestLinkParams, IpcColorScheme, IpcMessage,
    IpcMessageKind, LoadHtmlParams, SetColorSchemeParams, SetViewportParams, StorageOpParams, StorageOperation,
    StorageType,
};
use zero_protocol::process::{ProcessManager, RendererHandle};
use zero_storage::StorageManager;

use crate::tab_scripts::DomDispatchResult;
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

static NEXT_HIT_TEST_MSG_ID: AtomicU64 = AtomicU64::new(1);

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
    pending_fetches: Vec<PendingFetch>,
}

struct PendingFetch {
    tab_id: TabId,
    request_id: u64,
    url: String,
    rx: Receiver<(u16, Vec<u8>)>,
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
            pending_fetches: Vec::new(),
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
                snap.loading = false;
                let title = snap.title.clone().unwrap_or_else(|| "页面".to_string());
                let url = snap.url.clone().unwrap_or_default();
                pending_loaded.push((tab_id, title, url));
            }
            IpcMessageKind::LoadFailed(err) => {
                snap.loading = false;
                pending_errors.push((tab_id, err.clone()));
                tracing::warn!("Renderer load failed: {err}");
            }
            IpcMessageKind::UrlChanged(url) => snap.url = Some(url),
            IpcMessageKind::ViewPainted(params) => {
                crate::paint_ipc::apply_paint_snapshot(snap, *params);
                snap.clear_browser_owned_hit_test();
            }
            _ => {}
        }
    }

    fn handle_fetch_request(&mut self, tab_id: TabId, params: FetchParams) {
        let url = params.url.clone();
        let request_id = params.request_id;
        tracing::info!("Browser fetch proxy tab {}: {url}", tab_id.0);
        let (tx, rx) = mpsc::channel();
        let fetch_url = url.clone();
        thread::spawn(move || {
            let client = zero_net::HttpClient::new();
            let result = match client.get(&fetch_url) {
                Ok(resp) => (resp.status_code, resp.body),
                Err(e) => {
                    tracing::warn!("browser fetch proxy failed ({fetch_url}): {e}");
                    (0, format!("网络请求失败: {e}").into_bytes())
                }
            };
            let _ = tx.send(result);
        });
        self.pending_fetches.push(PendingFetch {
            tab_id,
            request_id,
            url,
            rx,
        });
    }

    fn drain_pending_fetches(&mut self) {
        let mut still_pending = Vec::new();
        let mut completed = Vec::new();
        for pending in self.pending_fetches.drain(..) {
            match pending.rx.try_recv() {
                Ok((status, body)) => {
                    tracing::info!(
                        "Browser fetch proxy done tab {}: {} status={status} bytes={}",
                        pending.tab_id.0,
                        pending.url,
                        body.len()
                    );
                    completed.push((pending.tab_id, pending.request_id, status, body));
                }
                Err(mpsc::TryRecvError::Empty) => still_pending.push(pending),
                Err(mpsc::TryRecvError::Disconnected) => {
                    tracing::warn!(
                        "Browser fetch proxy thread dropped tab {}: {}",
                        pending.tab_id.0,
                        pending.url
                    );
                    completed.push((
                        pending.tab_id,
                        pending.request_id,
                        0,
                        "网络请求失败: fetch worker exited".as_bytes().to_vec(),
                    ));
                }
            }
        }
        self.pending_fetches = still_pending;
        for (tab_id, request_id, status, body) in completed {
            self.send_fetch_response_now(tab_id, request_id, status, body);
        }
    }

    /// 导航并在本线程轮询 IPC，直到该 Tab 加载完成/失败或超时（测试/同步场景用）。
    ///
    /// 正常运行时由事件循环 `poll` 驱动 fetch 代理；勿在主/UI 线程调用以免阻塞 winit。
    pub fn navigate_and_service(&mut self, tab_id: TabId, url: &str, snapshots: &mut HashMap<TabId, TabSnapshot>) {
        self.navigate(tab_id, url);
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
            self.tab_to_renderer.remove(&tab_id);
            self.pending_errors.push((tab_id, "渲染进程已崩溃".to_string()));
            if let Some(snap) = snapshots.get_mut(&tab_id) {
                snap.loading = false;
            }
            tracing::warn!("Renderer {rid} for tab {}: {reason}", tab_id.0);
        }
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
        if let Some(rid) = self.tab_to_renderer.remove(&tab_id) {
            let _ = self.manager.shutdown_renderer(rid);
        }
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
    pub fn navigate(&mut self, tab_id: TabId, url: &str) {
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.navigate(url, None) {
            tracing::warn!("IPC navigate failed: {e}");
        }
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
    pub fn load_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>, url: Option<&str>) {
        self.send_to_renderer(
            tab_id,
            IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: css.map(str::to_string),
                url: url.map(str::to_string),
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

    /// 轮询 IPC 并更新快照；后台 Tab 可降频。
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
        for (tab_id, rid) in mapping {
            loop {
                let msg = {
                    let Some(renderer) = self.manager.get_renderer(rid) else {
                        break;
                    };
                    match renderer.try_recv() {
                        Ok(Some(m)) => m,
                        Ok(None) => break,
                        Err(e) => {
                            tracing::debug!("IPC recv: {e}");
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
            }
        }
        changed
    }

    /// 链接命中测试（同步 IPC 请求/响应）。
    pub fn hit_test_link(
        &mut self,
        tab_id: TabId,
        x: f32,
        y: f32,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
    ) -> Option<String> {
        let rid = *self.tab_to_renderer.get(&tab_id)?;
        let msg_id = NEXT_HIT_TEST_MSG_ID.fetch_add(1, Ordering::Relaxed);
        {
            let renderer = self.manager.get_renderer(rid)?;
            renderer
                .send(IpcMessage {
                    id: msg_id,
                    kind: IpcMessageKind::HitTestLink(HitTestLinkParams { x, y }),
                })
                .ok()?;
        }

        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            if Instant::now() >= deadline {
                return None;
            }
            let msg = {
                let renderer = self.manager.get_renderer(rid)?;
                match renderer.try_recv() {
                    Ok(Some(m)) => m,
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        tracing::debug!("IPC hit_test recv: {e}");
                        return None;
                    }
                }
            };
            if msg.id == msg_id {
                if let IpcMessageKind::HitTestLinkResult(result) = msg.kind {
                    return result.href;
                }
                continue;
            }
            match msg.kind {
                IpcMessageKind::FetchRequest(params) => {
                    self.handle_fetch_request(tab_id, params);
                }
                kind => {
                    let snap = snapshots.entry(tab_id).or_default();
                    Self::apply_inbound_message(tab_id, snap, kind, &mut self.pending_loaded, &mut self.pending_errors);
                }
            }
        }
    }

    /// 元素命中测试（同步 IPC 请求/响应）。
    pub fn hit_test_element(
        &mut self,
        tab_id: TabId,
        x: f32,
        y: f32,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
    ) -> Option<ElementHit> {
        let rid = *self.tab_to_renderer.get(&tab_id)?;
        let msg_id = NEXT_HIT_TEST_MSG_ID.fetch_add(1, Ordering::Relaxed);
        {
            let renderer = self.manager.get_renderer(rid)?;
            renderer
                .send(IpcMessage {
                    id: msg_id,
                    kind: IpcMessageKind::HitTestElement(HitTestLinkParams { x, y }),
                })
                .ok()?;
        }

        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            if Instant::now() >= deadline {
                return None;
            }
            let msg = {
                let renderer = self.manager.get_renderer(rid)?;
                match renderer.try_recv() {
                    Ok(Some(m)) => m,
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        tracing::debug!("IPC hit_test_element recv: {e}");
                        return None;
                    }
                }
            };
            if msg.id == msg_id {
                if let IpcMessageKind::HitTestElementResult(result) = msg.kind {
                    return element_hit_from_ipc(result);
                }
                continue;
            }
            match msg.kind {
                IpcMessageKind::FetchRequest(params) => {
                    self.handle_fetch_request(tab_id, params);
                }
                kind => {
                    let snap = snapshots.entry(tab_id).or_default();
                    Self::apply_inbound_message(tab_id, snap, kind, &mut self.pending_loaded, &mut self.pending_errors);
                }
            }
        }
    }

    /// DOM 事件派发（同步 IPC 请求/响应）。
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_dom_event(
        &mut self,
        tab_id: TabId,
        selector: Option<&str>,
        x: f32,
        y: f32,
        event_type: &str,
        detail: Option<&DomEventDetail>,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
    ) -> DomDispatchResult {
        let rid = match self.tab_to_renderer.get(&tab_id) {
            Some(r) => *r,
            None => {
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                };
            }
        };
        let msg_id = NEXT_HIT_TEST_MSG_ID.fetch_add(1, Ordering::Relaxed);
        let params = DispatchDomEventParams {
            selector: selector.map(str::to_string),
            x,
            y,
            event_type: event_type.to_string(),
            key: detail.and_then(|d| d.key.clone()),
            code: detail.and_then(|d| d.code.clone()),
        };
        {
            let Some(renderer) = self.manager.get_renderer(rid) else {
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                };
            };
            if renderer
                .send(IpcMessage {
                    id: msg_id,
                    kind: IpcMessageKind::DispatchDomEvent(params),
                })
                .is_err()
            {
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                };
            }
        }

        let mut html_changed = false;
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            if Instant::now() >= deadline {
                return DomDispatchResult {
                    default_allowed: true,
                    html_changed,
                };
            }
            let msg = {
                let Some(renderer) = self.manager.get_renderer(rid) else {
                    return DomDispatchResult {
                        default_allowed: true,
                        html_changed,
                    };
                };
                match renderer.try_recv() {
                    Ok(Some(m)) => m,
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        tracing::debug!("IPC dispatch_dom_event recv: {e}");
                        return DomDispatchResult {
                            default_allowed: true,
                            html_changed,
                        };
                    }
                }
            };
            if msg.id == msg_id {
                if let IpcMessageKind::DispatchDomEventResult(result) = msg.kind {
                    return DomDispatchResult {
                        default_allowed: result.default_allowed,
                        html_changed,
                    };
                }
                continue;
            }
            match msg.kind {
                IpcMessageKind::FetchRequest(params) => {
                    self.handle_fetch_request(tab_id, params);
                }
                IpcMessageKind::ViewPainted(params) => {
                    html_changed = true;
                    let snap = snapshots.entry(tab_id).or_default();
                    crate::paint_ipc::apply_paint_snapshot(snap, *params);
                    snap.clear_browser_owned_hit_test();
                }
                kind => {
                    let snap = snapshots.entry(tab_id).or_default();
                    Self::apply_inbound_message(tab_id, snap, kind, &mut self.pending_loaded, &mut self.pending_errors);
                }
            }
        }
    }
}

fn element_hit_from_ipc(result: HitTestElementResultParams) -> Option<ElementHit> {
    let tag_name = result.tag_name?;
    Some(ElementHit {
        tag_name,
        id: result.id,
        class_name: result.class_name,
        x: result.x,
        y: result.y,
        width: result.width,
        height: result.height,
    })
}

impl Drop for ProcessTabBackend {
    fn drop(&mut self) {
        self.manager.shutdown_all();
    }
}
