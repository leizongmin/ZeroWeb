//! 可选多进程 Tab 后端 — 通过 `ProcessManager` 将页面渲染隔离到 `zero-renderer` 子进程。
//!
//! 网络请求由本进程代理（Chromium 式 browser-hosted network）；渲染进程仅通过 `FetchRequest` IPC 访问网络。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use zero_browser_shell::TabId;
use zero_engine::{IndexedDbHandler, PrefersColorSchemeValue};
use zero_protocol::ProtocolError;
use zero_protocol::message::{
    AutomationOperation, AutomationRequest, AutomationResponse, DispatchDomEventParams, FetchParams, FocusChangeInfo,
    FramePublishMode, ImeEventParams, IndexedDbConnectionEventAckParams, IndexedDbConnectionEventParams,
    IndexedDbRequestParams, IndexedDbResponseParams, IpcColorScheme, IpcMediaType, IpcMessage, IpcMessageKind,
    LoadHtmlParams, NavigationCommittedParams, NavigationStartedParams, ScrollEventParams, ServiceWorkerRequestParams,
    SetColorSchemeParams, SetMediaTypeParams, SetViewportParams, StorageOpParams, StorageOperation, StorageType,
};
use zero_protocol::process::{ProcessManager, RendererHandle};
use zero_storage::StorageManager;

use crate::fetch_proxy::{ServiceWorkerScriptRequestMode, TabFetchProxy};
use crate::service_worker_owner::{
    BrowserServiceWorkerOwner, CompletedServiceWorkerResponse, ServiceWorkerRequestDisposition,
};
use crate::tab_snapshot::{CompositorSubmission, TabSnapshot};
use indexed_db_connections::{
    ConnectionKey, ConnectionRequestStatus, ConnectionWireRequest, IndexedDbConnectionOwner, parse_connection_request,
};
use indexed_db_transactions::{IndexedDbTransactionOwner, parse_transaction_request};

#[path = "process_backend/indexed_db_connections.rs"]
mod indexed_db_connections;
#[cfg(test)]
#[path = "process_backend/indexed_db_owner_tests.rs"]
mod indexed_db_owner_tests;
#[path = "process_backend/indexed_db_transactions.rs"]
mod indexed_db_transactions;
#[cfg(test)]
#[path = "process_backend/service_worker_owner_tests.rs"]
mod service_worker_owner_tests;

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

fn browser_storage_manager() -> Result<StorageManager, zero_storage::StorageError> {
    if cfg!(test) || zero_runtime_config::enabled_when_true("ZERO_PRIVATE") {
        Ok(StorageManager::new())
    } else {
        StorageManager::with_indexed_db_persistence(zero_storage::default_indexed_db_dir())
    }
}

fn browser_service_worker_owner() -> BrowserServiceWorkerOwner {
    if cfg!(test) || zero_runtime_config::enabled_when_true("ZERO_PRIVATE") {
        BrowserServiceWorkerOwner::new()
    } else {
        BrowserServiceWorkerOwner::with_persistence(zero_storage::default_service_worker_state_path())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingIndexedDbNavigation {
    url: String,
    navigation_epoch: u64,
}

/// 解析 `zero-renderer` 可执行文件路径。
///
/// macOS `.app` 发布布局使用标准嵌套 Helper app；其他平台及本地构建保持
/// **`zero-renderer` 与 `zero-browser` 同目录**。
/// 查找顺序：
/// 1. `ZERO_RENDERER_PATH` 环境变量
/// 2. `CARGO_BIN_EXE_zero-renderer`（cargo test / cargo run 注入——测试环境 PATH 不含
///    `target/debug`，此前多进程测试全部静默回退单进程 worker，见 R3254 修复）
/// 3. macOS `ZeroBrowser Helper (Renderer).app`
/// 4. `std::env::current_exe()` 所在目录（含测试二进制 `target/debug/deps/` 上溯 `target/debug/`）
/// 5. `PATH`（系统级安装等兜底）
pub(crate) fn resolve_renderer_binary() -> Option<PathBuf> {
    if let Some(candidate) = zero_runtime_config::optional_path("ZERO_RENDERER_PATH") {
        if candidate.is_file() {
            return Some(candidate);
        }
        tracing::warn!("ZERO_RENDERER_PATH 指向的文件不存在: {}", candidate.display());
    }

    if let Ok(path) = std::env::var("CARGO_BIN_EXE_zero-renderer") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Some(candidate) = renderer_binary_near_current_exe() {
        return Some(candidate);
    }

    // cargo test 测试二进制位于 `target/debug/deps/`——上溯到 `target/debug/`。
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
        && let Some(grandparent) = parent.parent()
    {
        let candidate = grandparent.join(renderer_binary_filename());
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    find_renderer_in_path()
}

fn renderer_candidates_near_executable(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    if let Some(macos_dir) = exe.parent()
        && macos_dir.file_name().and_then(|name| name.to_str()) == Some("MacOS")
        && let Some(contents_dir) = macos_dir.parent()
        && contents_dir.file_name().and_then(|name| name.to_str()) == Some("Contents")
    {
        candidates.push(
            contents_dir
                .join("Frameworks")
                .join("ZeroBrowser Helper (Renderer).app")
                .join("Contents")
                .join("MacOS")
                .join("ZeroBrowser Helper (Renderer)"),
        );
    }

    if let Some(dir) = exe.parent() {
        candidates.push(dir.join(renderer_binary_filename()));
    }
    candidates
}

/// 在当前应用的 macOS Helper bundle 或可执行文件同目录查找 renderer。
fn renderer_binary_near_current_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    renderer_candidates_near_executable(&exe)
        .into_iter()
        .find(|candidate| candidate.is_file())
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
    device_scale_factor: f32,
    javascript_enabled: bool,
    storage: Arc<Mutex<StorageManager>>,
    private_storage: Arc<Mutex<StorageManager>>,
    private_tabs: HashSet<TabId>,
    indexed_db_handlers: HashMap<u64, IndexedDbHandler>,
    indexed_db_connections: IndexedDbConnectionOwner,
    indexed_db_transactions: IndexedDbTransactionOwner,
    indexed_db_origins: HashMap<u64, String>,
    pending_indexed_db_navigations: HashMap<u64, PendingIndexedDbNavigation>,
    committed_document_urls: HashMap<u64, String>,
    committed_document_epochs: HashMap<u64, u64>,
    indexed_db_init_error: Option<String>,
    pending_loaded: Vec<(TabId, String, String)>,
    pending_errors: Vec<(TabId, String)>,
    fetch_proxy: TabFetchProxy,
    service_worker_owner: BrowserServiceWorkerOwner,
    /// 异步 DOM 事件派发的回执（按 dispatch id 收集，由 TabManager 消费）。
    pending_dispatch_results: Vec<(u64, bool)>,
    /// 页面焦点所有者变更（R3254-H1，由 TabManager 消费同步 event_targets）。
    pending_focus_changes: Vec<(TabId, FocusChangeInfo)>,
    /// live renderer 自动化脚本回执（request id → tab + response）。
    pending_automation_responses: HashMap<u64, (TabId, AutomationResponse)>,
    /// 上一次轮询观察到的 compositor client 状态，用于只处理一次断线边沿。
    compositor_status: crate::compositor_client::CompositorStatus,
    /// Browser 窗口 GPU 渲染器是否可用（dma-buf 导入 vs RGBA 回退）。
    browser_gpu_present: bool,
}

#[cfg(test)]
thread_local! {
    static TEST_RENDERER_OUTBOUND: std::cell::RefCell<Vec<IpcMessageKind>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub fn take_test_renderer_outbound() -> Vec<IpcMessageKind> {
    TEST_RENDERER_OUTBOUND.with(|log| {
        let mut buf = log.borrow_mut();
        std::mem::take(&mut *buf)
    })
}

impl ProcessTabBackend {
    /// 暂存最新绘制帧，避免 UI 线程逐个转换 renderer 积压的完整页面快照。
    fn defer_latest_paint(latest_paint: &mut Option<IpcMessageKind>, kind: IpcMessageKind) -> Option<IpcMessageKind> {
        if matches!(
            &kind,
            IpcMessageKind::ViewPainted(_) | IpcMessageKind::CompositorFrame { .. }
        ) {
            *latest_paint = Some(kind);
            None
        } else {
            Some(kind)
        }
    }

    /// 创建多进程后端。缺失的 renderer binary 会在创建 tab 时作为启动错误报告。
    pub fn try_new() -> Self {
        let renderer_bin = resolve_renderer_binary().unwrap_or_else(|| PathBuf::from(renderer_binary_filename()));
        tracing::info!("Multi-process renderer binary: {}", renderer_bin.display());
        Self::with_renderer_bin(renderer_bin)
    }

    fn with_renderer_bin(renderer_bin: PathBuf) -> Self {
        Self::with_renderer_bin_and_service_worker_owner(renderer_bin, browser_service_worker_owner())
    }

    #[cfg(test)]
    fn with_renderer_bin_and_service_worker_persistence(renderer_bin: PathBuf, path: PathBuf) -> Self {
        Self::with_renderer_bin_and_service_worker_owner(
            renderer_bin,
            BrowserServiceWorkerOwner::with_persistence(path),
        )
    }

    fn with_renderer_bin_and_service_worker_owner(
        renderer_bin: PathBuf,
        service_worker_owner: BrowserServiceWorkerOwner,
    ) -> Self {
        let storage = browser_storage_manager();
        let (storage, storage_error) = match storage {
            Ok(storage) => (storage, None),
            Err(error) => (StorageManager::new(), Some(error.to_string())),
        };
        let storage = Arc::new(Mutex::new(storage));
        Self {
            manager: ProcessManager::new(renderer_bin.to_string_lossy().as_ref()),
            tab_to_renderer: HashMap::new(),
            renderer_bin,
            viewport: (800, 600),
            device_scale_factor: 1.0,
            javascript_enabled: true,
            storage,
            private_storage: Arc::new(Mutex::new(StorageManager::new())),
            private_tabs: HashSet::new(),
            indexed_db_handlers: HashMap::new(),
            indexed_db_connections: IndexedDbConnectionOwner::default(),
            indexed_db_transactions: IndexedDbTransactionOwner::default(),
            indexed_db_origins: HashMap::new(),
            pending_indexed_db_navigations: HashMap::new(),
            committed_document_urls: HashMap::new(),
            committed_document_epochs: HashMap::new(),
            indexed_db_init_error: storage_error,
            pending_loaded: Vec::new(),
            pending_errors: Vec::new(),
            fetch_proxy: TabFetchProxy::new(),
            service_worker_owner,
            pending_dispatch_results: Vec::new(),
            pending_focus_changes: Vec::new(),
            pending_automation_responses: HashMap::new(),
            compositor_status: crate::compositor_client::status(),
            browser_gpu_present: false,
        }
    }

    /// 更新 Browser GPU 呈现是否可用（影响 compositor dma-buf 导入 vs RGBA 回退）。
    pub fn set_browser_gpu_present(&mut self, present: bool) {
        self.browser_gpu_present = present;
    }

    fn enters_compositor_fallback(
        previous: crate::compositor_client::CompositorStatus,
        current: crate::compositor_client::CompositorStatus,
    ) -> bool {
        matches!(
            previous,
            crate::compositor_client::CompositorStatus::Starting | crate::compositor_client::CompositorStatus::Healthy
        ) && current == crate::compositor_client::CompositorStatus::Disconnected
    }

    fn observe_compositor_status(
        &mut self,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
    ) -> bool {
        let current = crate::compositor_client::status();
        self.observe_compositor_status_value(current, snapshots, snapshot_seq)
    }

    fn observe_compositor_status_value(
        &mut self,
        current: crate::compositor_client::CompositorStatus,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
    ) -> bool {
        let fallback = Self::enters_compositor_fallback(self.compositor_status, current);
        self.compositor_status = current;
        if !fallback {
            return false;
        }

        for (tab_id, snapshot) in snapshots {
            snapshot.clear_compositor_state();
            *snapshot_seq.entry(*tab_id).or_insert(0) += 1;
        }
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(tab_id, IpcMessageKind::SetFramePublishMode(FramePublishMode::Legacy));
            self.send_to_renderer(tab_id, IpcMessageKind::RequestFrame);
        }
        tracing::warn!("Compositor disconnected; switched isolated renderers to legacy frame publishing");
        true
    }

    fn send_fetch_response_now(
        &mut self,
        tab_id: TabId,
        request_id: u64,
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) {
        if let Some(renderer) = self.renderer_mut(tab_id) {
            if let Err(e) = renderer.send_fetch_response(request_id, status, headers, body) {
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
        snapshot_seq: &mut HashMap<TabId, u64>,
        pending_loaded: &mut Vec<(TabId, String, String)>,
        pending_errors: &mut Vec<(TabId, String)>,
    ) {
        match kind {
            IpcMessageKind::TitleChanged(title) => {
                if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                    tracing::info!(
                        "SMOKE_EVENT component=browser event=title_changed tab={} length={}",
                        tab_id.0,
                        title.len()
                    );
                }
                snap.title = Some(title);
            }
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
                if params.navigation_epoch != snap.navigation_epoch {
                    tracing::debug!(
                        "忽略 stale ViewPainted tab {} epoch {} != {}",
                        tab_id.0,
                        params.navigation_epoch,
                        snap.navigation_epoch
                    );
                    return;
                }
                if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                    tracing::info!(
                        "SMOKE_EVENT component=browser event=legacy_view_painted tab={} epoch={}",
                        tab_id.0,
                        params.navigation_epoch
                    );
                }
                crate::paint_ipc::apply_paint_snapshot(snap, *params);
                // 性能门禁优化 S1（2026-08-08）：快照到达 = 页面内容变更 →
                // 递增快照序号，滚动 blit 据此失效保留帧缓冲
                *snapshot_seq.entry(tab_id).or_insert(0) += 1;
                // 首帧绘制到达即可结束 loading（脚本预取/执行可能仍在进行）。
                if snap.loading {
                    snap.loading = false;
                    let title = snap.title.clone().unwrap_or_else(|| "页面".to_string());
                    let url = snap.url.clone().unwrap_or_default();
                    pending_loaded.push((tab_id, title, url));
                }
            }
            IpcMessageKind::CompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
                paint,
            } => {
                if crate::compositor_client::status() == crate::compositor_client::CompositorStatus::Disconnected {
                    return;
                }
                if paint.navigation_epoch != navigation_epoch {
                    tracing::warn!(
                        "忽略 compositor frame tab {}: envelope epoch {} != paint epoch {}",
                        tab_id.0,
                        navigation_epoch,
                        paint.navigation_epoch
                    );
                    return;
                }
                let submission = CompositorSubmission {
                    surface_id,
                    navigation_epoch,
                    frame_id,
                };
                if !snap.record_compositor_submission(submission) {
                    tracing::debug!(
                        "忽略 stale compositor frame tab {} surface {} epoch {} frame {}",
                        tab_id.0,
                        surface_id,
                        navigation_epoch,
                        frame_id
                    );
                    return;
                }
                // compositor 模式同时解码全文档图元到 last_render：compositor 回读位图
                // 仅覆盖未滚动视口（位图平移超一帧高度即空白），滚动时显示侧回落
                // 图元平移路径（app_render），任意滚动量渲染正确内容。
                crate::paint_ipc::apply_paint_snapshot(snap, (*paint).clone());
                crate::compositor_client::forward_frame(surface_id, navigation_epoch, frame_id, *paint);
            }
            _ => {}
        }
    }

    fn poll_compositor_frames(
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
        pending_loaded: &mut Vec<(TabId, String, String)>,
        browser_gpu_present: bool,
    ) -> bool {
        #[cfg(not(target_os = "linux"))]
        let _ = browser_gpu_present;

        let mut changed = false;
        for (tab_id, snap) in snapshots {
            let Some(submission) = snap.compositor_submission else {
                continue;
            };
            let Some(frame) = crate::compositor_client::get_frame(
                submission.surface_id,
                submission.navigation_epoch,
                submission.frame_id,
            ) else {
                continue;
            };
            let completed = CompositorSubmission {
                surface_id: frame.surface_id,
                navigation_epoch: frame.navigation_epoch,
                frame_id: frame.frame_id,
            };
            #[cfg(target_os = "linux")]
            let committed = if let Some(dmabuf) = frame.dmabuf {
                let use_gpu_import = zero_protocol::browser_gpu_dmabuf_import_enabled() && browser_gpu_present;
                if use_gpu_import {
                    use zero_render_foundation::gpu::{ExportedGpuFrame, map_linear_rgba};
                    // CPU 影子供 headless GPU 捕获渲染器回退绘制（无导入纹理）；
                    // 窗口渲染仍走 compositor_import。fd 经 try_clone 供映射用，
                    // 原 fd 仍交给 pending 导入；映射失败仅损失捕获保底。
                    let shadow = dmabuf.fd.try_clone().ok().and_then(|fd| {
                        let export = ExportedGpuFrame {
                            fd,
                            width: frame.width,
                            height: frame.height,
                            stride: dmabuf.stride,
                            drm_fourcc: dmabuf.drm_fourcc,
                            drm_modifier: dmabuf.drm_modifier,
                            sync_fd: None,
                        };
                        map_linear_rgba(&export).ok()
                    });
                    snap.commit_compositor_dmabuf(
                        completed,
                        frame.width,
                        frame.height,
                        frame.scroll_x,
                        frame.scroll_y,
                        crate::tab_snapshot::CompositorDmabufPending {
                            fd: dmabuf.fd,
                            width: frame.width,
                            height: frame.height,
                            stride: dmabuf.stride,
                            drm_fourcc: dmabuf.drm_fourcc,
                            drm_modifier: dmabuf.drm_modifier,
                        },
                        shadow,
                    )
                } else {
                    use zero_render_foundation::gpu::{ExportedGpuFrame, map_linear_rgba};
                    let export = ExportedGpuFrame {
                        fd: dmabuf.fd,
                        width: frame.width,
                        height: frame.height,
                        stride: dmabuf.stride,
                        drm_fourcc: dmabuf.drm_fourcc,
                        drm_modifier: dmabuf.drm_modifier,
                        sync_fd: None,
                    };
                    match map_linear_rgba(&export) {
                        Ok(rgba) => snap.commit_compositor_frame(
                            completed,
                            frame.width,
                            frame.height,
                            rgba,
                            frame.scroll_x,
                            frame.scroll_y,
                        ),
                        Err(error) => {
                            tracing::warn!("compositor dma-buf RGBA 回退失败: {error}");
                            false
                        }
                    }
                }
            } else {
                snap.commit_compositor_frame(
                    completed,
                    frame.width,
                    frame.height,
                    frame.rgba,
                    frame.scroll_x,
                    frame.scroll_y,
                )
            };
            #[cfg(not(target_os = "linux"))]
            let committed = snap.commit_compositor_frame(
                completed,
                frame.width,
                frame.height,
                frame.rgba,
                frame.scroll_x,
                frame.scroll_y,
            );
            if !committed {
                tracing::debug!(
                    "忽略 stale compositor result tab {} surface {} epoch {} frame {}",
                    tab_id.0,
                    frame.surface_id,
                    frame.navigation_epoch,
                    frame.frame_id
                );
                continue;
            }
            *snapshot_seq.entry(*tab_id).or_insert(0) += 1;
            #[cfg(target_os = "linux")]
            if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                let gpu_direct = snap.compositor_frame.as_ref().is_some_and(|f| f.gpu_direct);
                if gpu_direct {
                    tracing::info!(
                        "SMOKE_EVENT component=browser event=compositor_dmabuf_adopted tab={} surface={} epoch={} frame={}",
                        tab_id.0,
                        frame.surface_id,
                        frame.navigation_epoch,
                        frame.frame_id
                    );
                } else {
                    tracing::info!(
                        "SMOKE_EVENT component=browser event=compositor_bitmap_adopted tab={} surface={} epoch={} frame={}",
                        tab_id.0,
                        frame.surface_id,
                        frame.navigation_epoch,
                        frame.frame_id
                    );
                }
            }
            #[cfg(not(target_os = "linux"))]
            if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                tracing::info!(
                    "SMOKE_EVENT component=browser event=compositor_bitmap_adopted tab={} surface={} epoch={} frame={}",
                    tab_id.0,
                    frame.surface_id,
                    frame.navigation_epoch,
                    frame.frame_id
                );
            }
            changed = true;
            if snap.loading {
                snap.loading = false;
                let title = snap.title.clone().unwrap_or_else(|| "页面".to_string());
                let url = snap.url.clone().unwrap_or_default();
                pending_loaded.push((*tab_id, title, url));
            }
        }
        changed
    }

    fn poll_compositor_present_frames(
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
    ) -> bool {
        if !crate::compositor_client::present_enabled() {
            return false;
        }
        let mut changed = false;
        for (tab_id, snap) in snapshots {
            let Some(page) = snap.compositor_frame.as_ref() else {
                continue;
            };
            let Some((width, height, rgba)) = crate::compositor_client::take_present_frame(page.surface_id) else {
                continue;
            };
            if !snap.commit_compositor_present_frame(page.surface_id, width, height, rgba) {
                continue;
            }
            *snapshot_seq.entry(*tab_id).or_insert(0) += 1;
            changed = true;
        }
        changed
    }

    fn handle_fetch_request(&mut self, tab_id: TabId, params: FetchParams) {
        self.fetch_proxy.enqueue(tab_id, &params);
    }

    fn update_pending_indexed_db_navigation_from_fetch(&mut self, tab_id: TabId, headers: &[(String, String)]) {
        let is_document = headers
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-resource-type") && value == "document");
        let final_url = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("x-zero-final-url"))
            .map(|(_, value)| value);
        if is_document
            && let Some(final_url) = final_url
            && let Some(renderer_id) = self.tab_to_renderer.get(&tab_id)
            && let Some(pending) = self.pending_indexed_db_navigations.get_mut(renderer_id)
        {
            pending.url.clone_from(final_url);
        }
    }

    fn drain_pending_fetches(&mut self) {
        for item in self.fetch_proxy.drain() {
            self.update_pending_indexed_db_navigation_from_fetch(item.tab_id, &item.headers);
            self.send_fetch_response_now(item.tab_id, item.request_id, item.status, item.headers, item.body);
        }
    }

    fn handle_service_worker_request(
        &mut self,
        tab_id: TabId,
        renderer_id: u64,
        request_id: u64,
        params: ServiceWorkerRequestParams,
    ) {
        let authority = self.committed_document_urls.get(&renderer_id).cloned();
        let client_id = self.service_worker_client_id(renderer_id);
        let disposition = self.service_worker_owner.begin_request_for_client(
            tab_id,
            self.private_tabs.contains(&tab_id),
            request_id,
            authority.as_deref(),
            &client_id,
            params,
        );
        match disposition {
            ServiceWorkerRequestDisposition::Respond(response) => {
                self.send_service_worker_response_now(response);
            }
            ServiceWorkerRequestDisposition::Fetch(plan) => {
                let receiver = self.fetch_proxy.fetch_service_worker_script(
                    plan.tab_id(),
                    plan.script_url(),
                    plan.bypass_cache(),
                    ServiceWorkerScriptRequestMode::SameOrigin,
                    true,
                );
                self.service_worker_owner.attach_fetch(plan, receiver);
            }
        }
    }

    fn service_worker_client_id(&self, renderer_id: u64) -> String {
        format!(
            "{}:{}",
            renderer_id,
            self.committed_document_epochs.get(&renderer_id).copied().unwrap_or(0)
        )
    }

    fn drain_service_worker_responses(&mut self) {
        for response in self.service_worker_owner.poll() {
            self.send_service_worker_response_now(response);
        }
        for plan in self.service_worker_owner.take_update_fetch_plans() {
            let receiver = self.fetch_proxy.fetch_service_worker_script(
                plan.tab_id(),
                plan.script_url(),
                plan.bypass_cache(),
                ServiceWorkerScriptRequestMode::SameOrigin,
                true,
            );
            self.service_worker_owner.attach_fetch(plan, receiver);
        }
        for plan in self.service_worker_owner.take_import_fetch_plans() {
            let tab_id = plan.tab_id();
            let bypass_cache = plan.bypass_cache();
            let request_mode = if plan.is_module() {
                ServiceWorkerScriptRequestMode::Cors
            } else {
                ServiceWorkerScriptRequestMode::NoCors
            };
            let receivers = plan
                .urls()
                .iter()
                .map(|url| {
                    self.fetch_proxy
                        .fetch_service_worker_script(tab_id, url, bypass_cache, request_mode, false)
                })
                .collect();
            self.service_worker_owner.attach_import_fetches(plan, receivers);
        }
    }

    /// 下发 SW 托管命令到宿主 renderer（求值/生命周期/停止在 renderer 进程执行）。
    fn drain_service_worker_host_commands(&mut self) {
        for outgoing in self.service_worker_owner.take_host_commands() {
            let Some(renderer) = self.renderer_mut(outgoing.tab_id) else {
                continue;
            };
            if let Err(error) = renderer.send(IpcMessage {
                id: outgoing.params.registration_id,
                kind: IpcMessageKind::ServiceWorkerHostCommand(outgoing.params),
            }) {
                tracing::warn!(
                    "ServiceWorkerHostCommand send failed tab {}: {error}",
                    outgoing.tab_id.0
                );
            }
        }
    }

    fn send_service_worker_response_now(&mut self, response: CompletedServiceWorkerResponse) {
        let Some(renderer) = self.renderer_mut(response.tab_id) else {
            return;
        };
        if let Err(error) = renderer.send(IpcMessage {
            id: response.request_id,
            kind: IpcMessageKind::ServiceWorkerResponse(response.params),
        }) {
            tracing::warn!("ServiceWorkerResponse send failed tab {}: {error}", response.tab_id.0);
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
            // 测试/同步辅助路径：快照序号用临时 map（不影响运行期 blit 失效判定）
            self.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
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
        let Ok(mut storage) = self.storage.lock() else {
            tracing::warn!("Storage manager lock poisoned");
            return;
        };
        let store = match params.storage_type {
            StorageType::Local => storage.local_storage(&params.origin),
            StorageType::Session => storage.session_storage(&params.origin),
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

    fn remove_indexed_db_renderer_state(&mut self, renderer_id: u64) {
        self.indexed_db_handlers.remove(&renderer_id);
        self.indexed_db_connections.remove_renderer(renderer_id);
        self.indexed_db_transactions.remove_renderer(renderer_id);
        self.indexed_db_origins.remove(&renderer_id);
        self.pending_indexed_db_navigations.remove(&renderer_id);
        self.committed_document_urls.remove(&renderer_id);
        self.committed_document_epochs.remove(&renderer_id);
    }

    fn stage_indexed_db_navigation(&mut self, renderer_id: u64, url: &str, navigation_epoch: u64) {
        if let Some(tab_id) = self.tab_for_renderer(renderer_id) {
            self.service_worker_owner.disconnect_tab(tab_id);
        }
        self.remove_indexed_db_renderer_state(renderer_id);
        self.pending_indexed_db_navigations.insert(
            renderer_id,
            PendingIndexedDbNavigation {
                url: url.to_string(),
                navigation_epoch,
            },
        );
    }

    fn handle_navigation_started(
        &mut self,
        tab_id: TabId,
        renderer_id: u64,
        snapshot: &mut TabSnapshot,
        params: NavigationStartedParams,
    ) {
        let candidate = PendingIndexedDbNavigation {
            url: params.url.clone(),
            navigation_epoch: params.navigation_epoch,
        };
        if let Some(pending) = self.pending_indexed_db_navigations.get(&renderer_id)
            && pending != &candidate
        {
            tracing::warn!(
                "Rejected mismatched navigation start tab {} renderer {} epoch {}",
                tab_id.0,
                renderer_id,
                params.navigation_epoch
            );
            return;
        }
        if !self.pending_indexed_db_navigations.contains_key(&renderer_id) {
            if params.navigation_epoch == snapshot.navigation_epoch.wrapping_add(1) {
                snapshot.begin_navigation(params.url.clone());
            } else if params.navigation_epoch != snapshot.navigation_epoch {
                tracing::warn!(
                    "Rejected stale navigation start tab {} renderer {} epoch {} != {}",
                    tab_id.0,
                    renderer_id,
                    params.navigation_epoch,
                    snapshot.navigation_epoch
                );
                return;
            }
            self.fetch_proxy.on_navigate(tab_id, &params.url);
        }
        self.stage_indexed_db_navigation(renderer_id, &params.url, params.navigation_epoch);
    }

    fn handle_navigation_committed(&mut self, tab_id: TabId, renderer_id: u64, params: NavigationCommittedParams) {
        let Some(pending) = self.pending_indexed_db_navigations.get(&renderer_id) else {
            tracing::warn!(
                "Rejected navigation commit without start tab {} renderer {} epoch {}",
                tab_id.0,
                renderer_id,
                params.navigation_epoch
            );
            return;
        };
        if pending.navigation_epoch != params.navigation_epoch || pending.url != params.url {
            tracing::warn!(
                "Rejected mismatched navigation commit tab {} renderer {} epoch {}",
                tab_id.0,
                renderer_id,
                params.navigation_epoch
            );
            return;
        }
        let pending = self
            .pending_indexed_db_navigations
            .remove(&renderer_id)
            .expect("pending navigation checked above");
        // https://storage.spec.whatwg.org/#storage-keys
        let origin = zero_engine::indexed_db_origin(&pending.url);
        self.indexed_db_handlers.remove(&renderer_id);
        self.indexed_db_origins.insert(renderer_id, origin);
        self.committed_document_urls.insert(renderer_id, pending.url);
        self.committed_document_epochs
            .insert(renderer_id, pending.navigation_epoch);
        let client_id = self.service_worker_client_id(renderer_id);
        if let Some(committed_url) = self.committed_document_urls.get(&renderer_id).cloned()
            && let Err(error) = self.service_worker_owner.observe_committed_top_level_client(
                tab_id,
                self.private_tabs.contains(&tab_id),
                &client_id,
                &committed_url,
            )
        {
            tracing::warn!("Rejected committed Service Worker client: {error}");
        }
    }

    fn handle_indexed_db_connection_request(
        &mut self,
        renderer_id: u64,
        private: bool,
        origin: &str,
        request: ConnectionWireRequest,
    ) -> Result<String, String> {
        match request {
            ConnectionWireRequest::ConnectionCapabilities => Ok(serde_json::json!({
                "crossRenderer": true,
                "transactionScheduling": true,
            })
            .to_string()),
            ConnectionWireRequest::RegisterConnection {
                connection,
                database,
                version,
            } => {
                let storage = if private {
                    Arc::clone(&self.private_storage)
                } else {
                    Arc::clone(&self.storage)
                };
                let storage = storage
                    .lock()
                    .map_err(|_| "UnknownError: IndexedDB storage lock is poisoned".to_string())?;
                let database_version = storage
                    .indexed_db(origin, &database)
                    .map(|database| database.version)
                    .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
                if database_version != version {
                    return Err("VersionError: IndexedDB connection version does not match storage".to_string());
                }
                drop(storage);
                self.indexed_db_connections.register(
                    ConnectionKey {
                        renderer_id,
                        connection_id: connection,
                    },
                    private,
                    origin,
                    &database,
                )?;
                Ok(serde_json::json!({"registered": true}).to_string())
            }
            ConnectionWireRequest::CloseConnection { connection } => {
                self.indexed_db_connections.close(ConnectionKey {
                    renderer_id,
                    connection_id: connection,
                })?;
                Ok(serde_json::json!({"closed": true}).to_string())
            }
            ConnectionWireRequest::RequestConnectionChange { database, new_version } => {
                let storage = if private {
                    Arc::clone(&self.private_storage)
                } else {
                    Arc::clone(&self.storage)
                };
                let old_version = storage
                    .lock()
                    .map_err(|_| "UnknownError: IndexedDB storage lock is poisoned".to_string())?
                    .indexed_db(origin, &database)
                    .map(|database| database.version)
                    .unwrap_or(0);
                let (request_id, events) = self.indexed_db_connections.begin_request(
                    renderer_id,
                    private,
                    origin,
                    &database,
                    old_version,
                    new_version,
                )?;
                self.send_indexed_db_connection_events(events);
                Ok(match request_id {
                    Some(request_id) => serde_json::json!({
                        "ready": false,
                        "request": request_id,
                        "oldVersion": old_version,
                    }),
                    None => serde_json::json!({
                        "ready": true,
                        "oldVersion": old_version,
                    }),
                }
                .to_string())
            }
            ConnectionWireRequest::PollConnectionChange { request } => {
                let (request_private, request_origin, request_database) =
                    self.indexed_db_connections.request_scope(renderer_id, request)?;
                let storage = if request_private {
                    Arc::clone(&self.private_storage)
                } else {
                    Arc::clone(&self.storage)
                };
                let current_old_version = storage
                    .lock()
                    .map_err(|_| "UnknownError: IndexedDB storage lock is poisoned".to_string())?
                    .indexed_db(&request_origin, &request_database)
                    .map(|database| database.version)
                    .unwrap_or(0);
                let update = self
                    .indexed_db_connections
                    .status(renderer_id, request, current_old_version)?;
                self.send_indexed_db_connection_events(update.events);
                Ok(serde_json::json!({
                    "ready": update.status == ConnectionRequestStatus::Ready,
                    "blocked": update.status == ConnectionRequestStatus::Blocked,
                })
                .to_string())
            }
        }
    }

    fn send_indexed_db_connection_events(&mut self, events: Vec<indexed_db_connections::ConnectionEvent>) {
        for event in events {
            let Some(tab_id) = self.tab_for_renderer(event.target.renderer_id) else {
                self.indexed_db_connections.remove_renderer(event.target.renderer_id);
                continue;
            };
            self.send_to_renderer(
                tab_id,
                IpcMessageKind::IndexedDbConnectionEvent(IndexedDbConnectionEventParams {
                    connection_id: event.target.connection_id,
                    request_id: event.request_id,
                    old_version: event.old_version,
                    new_version: event.new_version,
                }),
            );
        }
    }

    fn handle_indexed_db_connection_event_ack(&mut self, renderer_id: u64, params: IndexedDbConnectionEventAckParams) {
        if let Err(error) = self.indexed_db_connections.acknowledge(
            params.request_id,
            ConnectionKey {
                renderer_id,
                connection_id: params.connection_id,
            },
        ) {
            tracing::warn!("Rejected IndexedDB connection event ack from renderer {renderer_id}: {error}");
        }
    }

    fn handle_indexed_db_request(&mut self, tab_id: TabId, request_id: u64, params: IndexedDbRequestParams) {
        const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
        let private = self.private_tabs.contains(&tab_id);
        let Some(renderer_id) = self.tab_to_renderer.get(&tab_id).copied() else {
            return;
        };
        let result = if params.request.len() > MAX_REQUEST_BYTES {
            Err("UnknownError: IndexedDB request exceeds 8 MiB".to_string())
        } else if !self.indexed_db_origins.contains_key(&renderer_id) {
            Err("SecurityError: IndexedDB is unavailable before navigation commit".to_string())
        } else if !private && let Some(error) = &self.indexed_db_init_error {
            Err(format!(
                "UnknownError: IndexedDB storage initialization failed: {error}"
            ))
        } else {
            let origin = self
                .indexed_db_origins
                .get(&renderer_id)
                .expect("origin presence checked above")
                .clone();
            let storage = if private {
                Arc::clone(&self.private_storage)
            } else {
                Arc::clone(&self.storage)
            };
            match parse_connection_request(&params.request) {
                Ok(Some(request)) => self.handle_indexed_db_connection_request(renderer_id, private, &origin, request),
                Ok(None) => match parse_transaction_request(&params.request) {
                    Ok(Some(request)) => self.handle_indexed_db_transaction_request(
                        renderer_id,
                        private,
                        &origin,
                        storage,
                        &params.request,
                        request,
                    ),
                    Ok(None) => {
                        let handler = Arc::clone(
                            self.indexed_db_handlers
                                .entry(renderer_id)
                                .or_insert_with(|| zero_page_runtime::indexed_db_handler(storage)),
                        );
                        handler(&origin, &params.request)
                    }
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            }
        };
        let params = match result {
            Ok(response) => IndexedDbResponseParams {
                response: Some(response),
                error: None,
            },
            Err(error) => IndexedDbResponseParams {
                response: None,
                error: Some(error),
            },
        };
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(error) = renderer.send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::IndexedDbResponse(params),
        }) {
            tracing::warn!("IndexedDbResponse send failed tab {}: {error}", tab_id.0);
        }
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
        self.service_worker_owner.disconnect_tab(tab_id);
        // renderer 进程死亡：其托管的 SW runtime 一并失效，注入 Closed 推进状态机。
        self.service_worker_owner.fail_tab_hosted_runtimes(tab_id);
        self.remove_indexed_db_renderer_state(rid);
        self.fetch_proxy.remove_tab(tab_id);
        let _ = self.manager.shutdown_renderer(rid);
        // R3254-F10：renderer 意外退出自动重启（一次）并按快照 URL 重新导航——此前
        // 页面永久失败（pending_errors + loading=false，无恢复路径）。正常关闭
        //（remove_tab）不走本路径（poll 断开检测专用）。
        if let Some(url) = snapshots.get(&tab_id).and_then(|s| s.url.clone()) {
            let epoch = snapshots
                .get(&tab_id)
                .map(|s| s.navigation_epoch.wrapping_add(1))
                .unwrap_or(0);
            self.ensure_renderer(tab_id, self.viewport);
            self.navigate(tab_id, &url, epoch);
            tracing::error!(
                renderer_id = rid,
                tab_id = tab_id.0,
                %reason,
                %url,
                "Renderer crashed; respawning and navigating the tab"
            );
            return;
        }
        if !self.pending_errors.iter().any(|(t, _)| *t == tab_id) {
            self.pending_errors
                .push((tab_id, format!("渲染进程连接已断开: {reason}")));
        }
        if let Some(snap) = snapshots.get_mut(&tab_id) {
            snap.loading = false;
        }
        tracing::error!(renderer_id = rid, tab_id = tab_id.0, %reason, "Renderer disconnected");
    }

    fn ipc_recv_disconnected(err: &ProtocolError) -> bool {
        err.is_disconnected() || format!("{err}").contains("IPC 通道已关闭")
    }

    fn send_to_renderer(&mut self, tab_id: TabId, kind: IpcMessageKind) {
        #[cfg(test)]
        TEST_RENDERER_OUTBOUND.with(|log| log.borrow_mut().push(kind.clone()));
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.send(IpcMessage { id: 0, kind }) {
            tracing::warn!("IPC send failed for tab {}: {e}", tab_id.0);
        }
    }

    /// 测试：观察 compositor 状态。
    #[cfg(test)]
    pub fn observe_compositor_status_for_test(
        &mut self,
        current: crate::compositor_client::CompositorStatus,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
    ) -> bool {
        self.observe_compositor_status_value(current, snapshots, snapshot_seq)
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
            self.remove_indexed_db_renderer_state(rid);
        }
        match self.manager.spawn_renderer() {
            Ok(rid) => {
                self.tab_to_renderer.insert(tab_id, rid);
                tracing::info!("Spawned renderer {rid} for tab {}", tab_id.0);
                // 首个 renderer 接入：恢复持久化 SW（求值下放该 renderer）。
                self.service_worker_owner.flush_deferred_restores(tab_id);
                if self.compositor_status == crate::compositor_client::CompositorStatus::Disconnected {
                    self.send_to_renderer(tab_id, IpcMessageKind::SetFramePublishMode(FramePublishMode::Legacy));
                }
                self.send_to_renderer(
                    tab_id,
                    IpcMessageKind::SetViewport(SetViewportParams {
                        width: viewport.0,
                        height: viewport.1,
                        device_scale_factor: self.device_scale_factor,
                    }),
                );
                self.send_to_renderer(tab_id, IpcMessageKind::SetJavascriptEnabled(self.javascript_enabled));
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
        self.service_worker_owner.remove_tab(tab_id);
        self.private_tabs.remove(&tab_id);
        if let Some(rid) = self.tab_to_renderer.remove(&tab_id) {
            self.remove_indexed_db_renderer_state(rid);
            crate::compositor_client::release_surface(rid);
            let _ = self.manager.shutdown_renderer(rid);
        }
    }

    /// 标记 Tab 为无痕（fetch 使用仅内存缓存）。
    pub fn set_tab_private(&mut self, tab_id: TabId, private: bool) {
        self.fetch_proxy.set_tab_private(tab_id, private);
        if private {
            self.private_tabs.insert(tab_id);
        } else {
            self.private_tabs.remove(&tab_id);
            self.service_worker_owner.remove_private_profile(tab_id);
        }
        if let Some(renderer_id) = self.tab_to_renderer.get(&tab_id) {
            self.indexed_db_handlers.remove(renderer_id);
            self.indexed_db_connections.remove_renderer(*renderer_id);
            self.indexed_db_transactions.remove_renderer(*renderer_id);
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
    pub fn navigate(&mut self, tab_id: TabId, url: &str, navigation_epoch: u64) {
        self.fetch_proxy.on_navigate(tab_id, url);
        let Some(renderer_id) = self.tab_to_renderer.get(&tab_id).copied() else {
            return;
        };
        self.stage_indexed_db_navigation(renderer_id, url, navigation_epoch);
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
        let page_url = url.unwrap_or("about:blank");
        let Some(renderer_id) = self.tab_to_renderer.get(&tab_id).copied() else {
            return;
        };
        self.stage_indexed_db_navigation(renderer_id, page_url, navigation_epoch);
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
    pub fn resize_all(&mut self, width: u32, height: u32, device_scale_factor: f32) {
        self.viewport = (width, height);
        self.device_scale_factor = device_scale_factor;
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(
                tab_id,
                IpcMessageKind::SetViewport(SetViewportParams {
                    width,
                    height,
                    device_scale_factor,
                }),
            );
        }
    }

    /// 广播页面 JavaScript 执行策略；用户代理默认动作不受该开关影响。
    pub fn set_javascript_enabled(&mut self, enabled: bool) {
        self.javascript_enabled = enabled;
        let tabs: Vec<TabId> = self.tab_to_renderer.keys().copied().collect();
        for tab_id in tabs {
            self.send_to_renderer(tab_id, IpcMessageKind::SetJavascriptEnabled(enabled));
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

    fn renderer_poll_order(
        mut mapping: Vec<(TabId, u64)>,
        active_tab: Option<TabId>,
        poll_background: bool,
    ) -> Vec<(TabId, u64)> {
        if let Some(active_tab) = active_tab {
            mapping.retain(|(tab_id, _)| *tab_id == active_tab || poll_background);
            mapping.sort_unstable_by_key(|(tab_id, _)| *tab_id != active_tab);
        }
        mapping
    }

    pub fn poll(
        &mut self,
        snapshots: &mut HashMap<TabId, TabSnapshot>,
        snapshot_seq: &mut HashMap<TabId, u64>,
        active_tab: Option<TabId>,
        poll_background: bool,
    ) -> bool {
        self.service_worker_owner.set_focused_tab(active_tab);
        self.drain_pending_fetches();
        self.drain_service_worker_responses();
        self.drain_service_worker_host_commands();
        let mut changed = self.observe_compositor_status(snapshots, snapshot_seq);
        self.handle_crashes(snapshots);
        let mapping = Self::renderer_poll_order(
            self.tab_to_renderer.iter().map(|(k, v)| (*k, *v)).collect(),
            active_tab,
            poll_background,
        );
        let mut disconnected = Vec::new();
        let poll_deadline = Instant::now() + Self::POLL_BUDGET;
        for (tab_id, rid) in mapping {
            if Instant::now() >= poll_deadline {
                break;
            }
            let mut latest_paint = None;
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
                let Some(kind) = Self::defer_latest_paint(&mut latest_paint, msg.kind) else {
                    if Instant::now() >= poll_deadline {
                        break;
                    }
                    continue;
                };
                match kind {
                    IpcMessageKind::FetchRequest(params) => {
                        self.handle_fetch_request(tab_id, params);
                    }
                    IpcMessageKind::StorageOp(params) => {
                        self.handle_storage_op(tab_id, params);
                    }
                    IpcMessageKind::IndexedDbRequest(params) => {
                        self.handle_indexed_db_request(tab_id, msg.id, params);
                    }
                    IpcMessageKind::IndexedDbConnectionEventAck(params) => {
                        self.handle_indexed_db_connection_event_ack(rid, params);
                    }
                    IpcMessageKind::ServiceWorkerRequest(params) => {
                        self.handle_service_worker_request(tab_id, rid, msg.id, params);
                    }
                    IpcMessageKind::ServiceWorkerHostEvent(params) => {
                        let private = self.private_tabs.contains(&tab_id);
                        self.service_worker_owner.inject_host_event(tab_id, private, params);
                    }
                    IpcMessageKind::NavigationStarted(params) => {
                        let snapshot = snapshots.entry(tab_id).or_default();
                        self.handle_navigation_started(tab_id, rid, snapshot, params);
                    }
                    IpcMessageKind::NavigationCommitted(params) => {
                        self.handle_navigation_committed(tab_id, rid, params);
                    }
                    IpcMessageKind::DispatchDomEventResult(result) => {
                        self.pending_dispatch_results.push((msg.id, result.default_allowed));
                    }
                    IpcMessageKind::FocusOwnerChanged(info) => {
                        self.pending_focus_changes.push((tab_id, info));
                    }
                    IpcMessageKind::AutomationResponse(response) => {
                        self.pending_automation_responses.insert(msg.id, (tab_id, response));
                    }
                    kind => {
                        let snap = snapshots.entry(tab_id).or_default();
                        Self::apply_inbound_message(
                            tab_id,
                            snap,
                            kind,
                            snapshot_seq,
                            &mut self.pending_loaded,
                            &mut self.pending_errors,
                        );
                    }
                }
                if Instant::now() >= poll_deadline {
                    break;
                }
            }
            if let Some(kind) = latest_paint {
                let snap = snapshots.entry(tab_id).or_default();
                Self::apply_inbound_message(
                    tab_id,
                    snap,
                    kind,
                    snapshot_seq,
                    &mut self.pending_loaded,
                    &mut self.pending_errors,
                );
            }
        }
        for (tab_id, rid, reason) in disconnected {
            self.handle_renderer_ipc_lost(tab_id, rid, snapshots, &reason);
            changed = true;
        }
        changed |= self.observe_compositor_status(snapshots, snapshot_seq);
        if self.compositor_status == crate::compositor_client::CompositorStatus::Healthy {
            changed |= Self::poll_compositor_frames(
                snapshots,
                snapshot_seq,
                &mut self.pending_loaded,
                self.browser_gpu_present,
            );
            changed |= Self::poll_compositor_present_frames(snapshots, snapshot_seq);
        }
        self.drain_service_worker_responses();
        self.drain_service_worker_host_commands();
        changed
    }

    /// 取出异步 DOM 事件派发的回执（dispatch id, default_allowed）。
    pub fn take_dispatch_results(&mut self) -> Vec<(u64, bool)> {
        std::mem::take(&mut self.pending_dispatch_results)
    }

    /// 取出页面焦点变更回执（R3254-H1）。
    pub fn take_focus_changes(&mut self) -> Vec<(TabId, FocusChangeInfo)> {
        std::mem::take(&mut self.pending_focus_changes)
    }

    /// 向 live renderer 发起异步自动化操作。
    pub fn send_automation_request(
        &mut self,
        tab_id: TabId,
        request_id: u64,
        operation: AutomationOperation,
    ) -> Result<(), String> {
        let renderer = self
            .renderer_mut(tab_id)
            .ok_or_else(|| format!("no live renderer for tab {}", tab_id.0))?;
        renderer
            .send(IpcMessage {
                id: request_id,
                kind: IpcMessageKind::AutomationRequest(AutomationRequest { operation }),
            })
            .map_err(|error| format!("failed to send automation request for tab {}: {error}", tab_id.0))
    }

    /// 取出指定请求的自动化回执。
    pub fn take_automation_response(&mut self, tab_id: TabId, request_id: u64) -> Option<AutomationResponse> {
        let (response_tab, response) = self.pending_automation_responses.remove(&request_id)?;
        if response_tab == tab_id {
            Some(response)
        } else {
            tracing::warn!(
                "automation response tab mismatch: request={request_id} expected={} actual={}",
                tab_id.0,
                response_tab.0
            );
            None
        }
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
        params: DispatchDomEventParams,
    ) {
        let Some(rid) = self.tab_to_renderer.get(&tab_id).copied() else {
            // 渲染进程不存在：模拟一个“默认允许”回执，让 TabManager 走默认动作路径。
            self.pending_dispatch_results.push((dispatch_id, true));
            return;
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

    /// 把平台 IME 生命周期事件直接转发给 renderer 的页面输入状态机。
    pub fn dispatch_ime_event(&mut self, tab_id: TabId, message_id: u64, params: ImeEventParams) {
        let Some(rid) = self.tab_to_renderer.get(&tab_id).copied() else {
            return;
        };
        let Some(renderer) = self.manager.get_renderer(rid) else {
            return;
        };
        let _ = renderer.send(IpcMessage {
            id: message_id,
            kind: IpcMessageKind::ImeEvent(params),
        });
    }

    /// R3293（S0）/ R3298（S1）：向渲染进程派发「用户滚动」事件（fire-and-forget，无回执）。
    ///
    /// 闭合 R3253 主路径不可达 gap：browser 用户滚动经此发 `ScrollEvent` IPC → renderer
    /// `handle_scroll_event` 注入 `__zw_user_scroll` → 派 'scroll' + 更 window.scrollY。
    /// 既有 R3253 renderer 路径此前无生产调用方（仅 `#[test]` harness 可达），本方法激活它。
    /// 无回执（滚动不需 default-action 语义）；renderer 不存在时静默跳过（best-effort）。
    ///
    /// R3298（S1）：新增 `cursor_x`/`cursor_y`（滚轮发生处的视口物理坐标，相对 WebView 内容区），
    /// 供 renderer S2/S4 命中可滚动祖先容器；当前 renderer 仅记录坐标（S2 链路验证），元素级滚动
    /// 视觉依赖 S3 layout 几何暴露（渲染流域协调点，未实现）。
    pub fn send_user_scroll(&mut self, tab_id: TabId, delta_x: f32, delta_y: f32, cursor_x: f32, cursor_y: f32) {
        self.send_to_renderer(
            tab_id,
            IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x,
                delta_y,
                cursor_x,
                cursor_y,
            }),
        );
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
        crate::compositor_client::shutdown();
    }
}

impl Drop for ProcessTabBackend {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod renderer_path_tests {
    use super::{renderer_binary_filename, renderer_candidates_near_executable};
    use std::path::{Path, PathBuf};

    #[test]
    fn local_build_uses_sibling_renderer() {
        let executable = Path::new("/workspace/target/release/zero-browser");
        let candidates = renderer_candidates_near_executable(executable);

        assert_eq!(
            candidates.last(),
            Some(&PathBuf::from("/workspace/target/release").join(renderer_binary_filename()))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_app_prefers_renderer_helper_bundle() {
        let executable = Path::new("/Applications/ZeroBrowser.app/Contents/MacOS/ZeroBrowser");
        let candidates = renderer_candidates_near_executable(executable);

        assert_eq!(
            candidates,
            vec![
                PathBuf::from(
                    "/Applications/ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Renderer).app/Contents/MacOS/ZeroBrowser Helper (Renderer)"
                ),
                PathBuf::from("/Applications/ZeroBrowser.app/Contents/MacOS/zero-renderer"),
            ]
        );
    }
}

#[cfg(test)]
mod navigation_contract_tests {
    use super::ProcessTabBackend;
    use crate::paint_ipc::apply_paint_snapshot;
    use crate::tab_snapshot::{CompositorSubmission, PageRenderResult, TabSnapshot};
    use zero_browser_shell::TabId;
    use zero_protocol::message::IpcMessageKind;
    use zero_protocol::{
        IpcColor, IpcFill, IpcHitTestCache, IpcHitTestLayoutNode, IpcHitTestNodeMeta, IpcRect, PaintSnapshotParams,
    };
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

    fn paint_with_red_fill(epoch: u64) -> PaintSnapshotParams {
        PaintSnapshotParams {
            viewport_width: 800,
            viewport_height: 600,
            device_scale_factor: 1.0,
            document_height: 400.0,
            navigation_epoch: epoch,
            document_generation: 1,
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
            glyph_text_runs: vec![],
            font_variations: vec![],
            glyphs: vec![],
            draw_order: vec![],
            dirty_rects: vec![],
            hit_test: None,
            text_control_boundaries: vec![],
        }
    }

    fn legacy_blue_render() -> PageRenderResult {
        PageRenderResult {
            primitives: RenderPrimitives {
                fills: vec![FillPrimitive {
                    rect: Rect::new(0.0, 0.0, 50.0, 50.0),
                    color: Color::rgb(0, 0, 255),
                }],
                ..RenderPrimitives::new()
            },
            dirty_rects: Vec::new(),
        }
    }

    #[test]
    fn renderer_poll_order_prioritizes_active_tab_and_throttles_background_tabs() {
        let mapping = vec![(TabId(1), 11), (TabId(2), 22), (TabId(3), 33)];

        assert_eq!(
            ProcessTabBackend::renderer_poll_order(mapping.clone(), Some(TabId(3)), false),
            vec![(TabId(3), 33)]
        );
        let with_background = ProcessTabBackend::renderer_poll_order(mapping, Some(TabId(3)), true);
        assert_eq!(with_background[0], (TabId(3), 33));
        assert_eq!(with_background.len(), 3);
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
        let mut snapshot_seq = std::collections::HashMap::new();
        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(epoch))),
            &mut snapshot_seq,
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

        let mut snapshot_seq = std::collections::HashMap::new();
        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::LoadComplete,
            &mut snapshot_seq,
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

        let mut snapshot_seq = std::collections::HashMap::new();
        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(epoch.wrapping_sub(1)))),
            &mut snapshot_seq,
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
        assert_eq!(snap.painted_content_height, Some(100.0));
    }

    #[test]
    fn paint_backlog_keeps_only_latest_frame_and_preserves_control_messages() {
        let mut latest = None;

        assert!(
            ProcessTabBackend::defer_latest_paint(
                &mut latest,
                IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(1))),
            )
            .is_none()
        );
        let control =
            ProcessTabBackend::defer_latest_paint(&mut latest, IpcMessageKind::TitleChanged("latest title".into()));
        assert!(matches!(
            control,
            Some(IpcMessageKind::TitleChanged(title)) if title == "latest title"
        ));
        assert!(
            ProcessTabBackend::defer_latest_paint(
                &mut latest,
                IpcMessageKind::ViewPainted(Box::new(paint_with_red_fill(2))),
            )
            .is_none()
        );

        assert!(matches!(
            latest,
            Some(IpcMessageKind::ViewPainted(params)) if params.navigation_epoch == 2
        ));
    }

    #[test]
    fn compositor_frame_decodes_page_primitives_and_metadata() {
        let tab_id = TabId(4);
        let mut snap = TabSnapshot {
            navigation_epoch: 5,
            loading: true,
            ..Default::default()
        };
        let mut pending_loaded = Vec::new();
        let mut pending_errors = Vec::new();
        let mut snapshot_seq = std::collections::HashMap::new();
        let mut paint = paint_with_red_fill(5);
        paint.document_height = 1200.0;
        paint.hit_test = Some(IpcHitTestCache {
            doc_root: 1,
            layout_root: IpcHitTestLayoutNode {
                node_id: Some(1),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                children: Vec::new(),
            },
            nodes: std::iter::once((
                1,
                IpcHitTestNodeMeta {
                    tag_name: "a".to_string(),
                    id: None,
                    class_name: None,
                    selector: "a".to_string(),
                    href: Some("https://example.com".to_string()),
                    src: None,
                },
            ))
            .collect(),
            parents: Default::default(),
        });

        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::CompositorFrame {
                surface_id: 44,
                navigation_epoch: 5,
                frame_id: 9,
                paint: Box::new(paint),
            },
            &mut snapshot_seq,
            &mut pending_loaded,
            &mut pending_errors,
        );

        assert_eq!(
            snap.compositor_submission,
            Some(CompositorSubmission {
                surface_id: 44,
                navigation_epoch: 5,
                frame_id: 9,
            })
        );
        assert_eq!(snap.document_height, Some(1200.0));
        assert!(snap.document_width.is_some());
        assert!(snap.hit_test.is_some());
        // compositor 模式同步解码全文档图元（滚动时显示侧回落图元平移路径）
        let render = snap.last_render.as_ref().expect("compositor paint decodes primitives");
        assert_eq!(render.primitives.fills.len(), 1, "page fills decoded from paint");
        assert!(snap.loading, "loading ends only after a completed compositor bitmap");
        assert!(pending_loaded.is_empty());
    }

    #[test]
    fn paint_backlog_keeps_latest_compositor_frame_per_renderer() {
        let mut latest = None;
        for frame_id in [10, 11] {
            assert!(
                ProcessTabBackend::defer_latest_paint(
                    &mut latest,
                    IpcMessageKind::CompositorFrame {
                        surface_id: 55,
                        navigation_epoch: 2,
                        frame_id,
                        paint: Box::new(paint_with_red_fill(2)),
                    },
                )
                .is_none()
            );
        }

        assert!(matches!(
            latest,
            Some(IpcMessageKind::CompositorFrame {
                surface_id: 55,
                navigation_epoch: 2,
                frame_id: 11,
                ..
            })
        ));
    }
}

#[cfg(test)]
mod compositor_fallback_tests {
    use super::{ProcessTabBackend, take_test_renderer_outbound};
    use crate::compositor_client::CompositorStatus;
    use crate::tab_snapshot::{CompositorFrame, CompositorSubmission, TabSnapshot};
    use std::path::PathBuf;
    use zero_browser_shell::TabId;
    use zero_protocol::message::{FramePublishMode, IpcMessageKind};
    use zero_protocol::{IpcColor, IpcFill, IpcRect, PaintSnapshotParams};

    fn lock_multiprocess_tests() -> std::sync::MutexGuard<'static, ()> {
        crate::tests::MULTIPROCESS_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn startup_failure_enters_fallback_once() {
        let mut previous = CompositorStatus::Starting;
        let current = CompositorStatus::Disconnected;

        assert!(ProcessTabBackend::enters_compositor_fallback(previous, current));
        previous = current;
        assert!(!ProcessTabBackend::enters_compositor_fallback(previous, current));
    }

    #[test]
    fn runtime_disconnect_enters_fallback_once() {
        let mut previous = CompositorStatus::Healthy;
        let current = CompositorStatus::Disconnected;

        assert!(ProcessTabBackend::enters_compositor_fallback(previous, current));
        previous = current;
        assert!(!ProcessTabBackend::enters_compositor_fallback(previous, current));
    }

    #[test]
    fn disabled_and_healthy_states_do_not_enter_fallback() {
        assert!(!ProcessTabBackend::enters_compositor_fallback(
            CompositorStatus::Disabled,
            CompositorStatus::Disabled,
        ));
        assert!(!ProcessTabBackend::enters_compositor_fallback(
            CompositorStatus::Starting,
            CompositorStatus::Healthy,
        ));
        assert!(!ProcessTabBackend::enters_compositor_fallback(
            CompositorStatus::Healthy,
            CompositorStatus::Healthy,
        ));
    }

    #[test]
    fn tab_remove_drops_renderer_surface_mapping() {
        let _multiprocess_guard = lock_multiprocess_tests();
        let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
        let tab_id = TabId(17);
        backend.tab_to_renderer.insert(tab_id, 44);

        backend.remove_renderer(tab_id);

        assert!(!backend.tab_to_renderer.contains_key(&tab_id));
    }

    /// R3293（S0）/ R3298（S1）：`send_user_scroll` 向渲染进程发 `ScrollEvent` IPC（激活既有 R3253
    /// renderer `handle_scroll_event` 路径，闭合其主路径不可达 gap）。验证多进程派发路径发正确 IPC
    /// kind + delta + cursor 透传（R3298 S1 新增 cursor_x/y 字段）。`send_to_renderer` 经
    /// `TEST_RENDERER_OUTBOUND` 测试桩捕获出站 IPC。
    #[test]
    #[serial_test::serial]
    fn send_user_scroll_emits_scroll_event_ipc_r3293() {
        let _multiprocess_guard = lock_multiprocess_tests();
        let _ = take_test_renderer_outbound(); // 清前序测试残留
        let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
        let tab_id = TabId(23);
        backend.tab_to_renderer.insert(tab_id, 7);

        backend.send_user_scroll(tab_id, 0.0, 120.0, 320.0, 480.0);

        let outbound = take_test_renderer_outbound();
        let scroll_msgs: Vec<_> = outbound
            .iter()
            .filter_map(|k| match k {
                IpcMessageKind::ScrollEvent(p) => Some((p.delta_x, p.delta_y, p.cursor_x, p.cursor_y)),
                _ => None,
            })
            .collect();
        assert!(
            scroll_msgs
                .iter()
                .any(|(dx, dy, cx, cy)| *dx == 0.0 && *dy == 120.0 && *cx == 320.0 && *cy == 480.0),
            "send_user_scroll 应发 ScrollEvent(delta=0,120, cursor=320,480) IPC，实得出站 ScrollEvent: {scroll_msgs:?}"
        );
    }

    fn compositor_test_bin() -> String {
        if let Ok(bin) = std::env::var("CARGO_BIN_EXE_zero-compositor") {
            return bin;
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/zero-compositor")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn compositor_disconnect_switches_isolated_renderers_to_legacy_frames() {
        let _multiprocess_guard = lock_multiprocess_tests();
        let _ = take_test_renderer_outbound();

        let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
        backend.compositor_status = CompositorStatus::Healthy;
        let tab_id = TabId(1);
        backend.tab_to_renderer.insert(tab_id, 99);

        let mut snapshots = std::collections::HashMap::from([(tab_id, TabSnapshot::default())]);
        let mut snapshot_seq = std::collections::HashMap::new();
        let snap = snapshots.get_mut(&tab_id).unwrap();
        snap.compositor_submission = Some(CompositorSubmission {
            surface_id: 99,
            navigation_epoch: 1,
            frame_id: 1,
        });

        assert!(backend.observe_compositor_status_for_test(
            CompositorStatus::Disconnected,
            &mut snapshots,
            &mut snapshot_seq
        ));
        assert_eq!(backend.compositor_status, CompositorStatus::Disconnected);
        let snap = snapshots.get(&tab_id).unwrap();
        assert!(snap.compositor_submission.is_none(), "compositor 断线应清除过期提交态");
        assert!(snap.compositor_frame.is_none());

        let outbound = take_test_renderer_outbound();
        assert!(
            outbound
                .iter()
                .any(|kind| matches!(kind, IpcMessageKind::SetFramePublishMode(FramePublishMode::Legacy))),
            "断线后应发 SetFramePublishMode(Legacy)，got {outbound:?}"
        );
        assert!(
            outbound.iter().any(|kind| matches!(kind, IpcMessageKind::RequestFrame)),
            "断线后应发 RequestFrame，got {outbound:?}"
        );
    }

    #[test]
    fn compositor_crash_legacy_viewpainted_restores_tab_render() {
        fn red_paint(epoch: u64) -> PaintSnapshotParams {
            PaintSnapshotParams {
                navigation_epoch: epoch,
                fills: vec![IpcFill {
                    rect: IpcRect {
                        x: 0.0,
                        y: 0.0,
                        width: 64.0,
                        height: 64.0,
                    },
                    color: IpcColor {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                }],
                ..Default::default()
            }
        }

        let tab_id = TabId(1);
        let mut snap = TabSnapshot {
            navigation_epoch: 1,
            compositor_submission: Some(CompositorSubmission {
                surface_id: 99,
                navigation_epoch: 1,
                frame_id: 1,
            }),
            compositor_frame: Some(CompositorFrame {
                surface_id: 99,
                navigation_epoch: 1,
                frame_id: 1,
                width: 64,
                height: 64,
                image_key: zero_render_foundation::image_cache::ImageKey::new(1),
                #[cfg(target_os = "linux")]
                gpu_direct: false,
            }),
            ..Default::default()
        };

        let mut pending_loaded = Vec::new();
        let mut pending_errors = Vec::new();
        let mut snapshot_seq = std::collections::HashMap::new();
        ProcessTabBackend::apply_inbound_message(
            tab_id,
            &mut snap,
            IpcMessageKind::ViewPainted(Box::new(red_paint(1))),
            &mut snapshot_seq,
            &mut pending_loaded,
            &mut pending_errors,
        );

        assert!(snap.last_render.is_some(), "Legacy ViewPainted 应恢复页面渲染");
        assert!(snap.compositor_submission.is_none() || snap.should_composite_paint());
    }
}
