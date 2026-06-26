//! 可选多进程 Tab 后端 — 通过 `ProcessManager` 将页面渲染隔离到 `zero-renderer` 子进程。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_protocol::message::{
    HitTestLinkParams, HitTestLinkResultParams, IpcColorScheme, IpcMessage, IpcMessageKind, LoadHtmlParams,
    SetColorSchemeParams, SetViewportParams,
};
use zero_protocol::process::{ProcessManager, RendererHandle};

use crate::tab_snapshot::TabSnapshot;

/// 是否启用多进程后端（环境变量 `ZERO_BROWSER_MULTIPROCESS=1`）。
pub fn use_multiprocess_backend() -> bool {
    std::env::var("ZERO_BROWSER_MULTIPROCESS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// 供 CLI 在解析参数后强制启用多进程。
pub fn set_multiprocess_enabled(enabled: bool) {
    // SAFETY: 在 main 启动早期、单线程环境下设置进程环境变量。
    unsafe {
        std::env::set_var("ZERO_BROWSER_MULTIPROCESS", if enabled { "1" } else { "0" });
    }
}

static NEXT_HIT_TEST_MSG_ID: AtomicU64 = AtomicU64::new(1);

/// 多进程 Tab 后端。
pub struct ProcessTabBackend {
    manager: ProcessManager,
    tab_to_renderer: HashMap<TabId, u64>,
    renderer_bin: PathBuf,
    viewport: (u32, u32),
}

impl ProcessTabBackend {
    /// 创建多进程后端。
    pub fn new() -> Self {
        let renderer_bin = std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent().map(|dir| {
                    #[cfg(windows)]
                    {
                        dir.join("zero-renderer.exe")
                    }
                    #[cfg(not(windows))]
                    {
                        dir.join("zero-renderer")
                    }
                })
            })
            .unwrap_or_else(|| PathBuf::from("zero-renderer"));

        Self {
            manager: ProcessManager::new(renderer_bin.to_string_lossy().as_ref()),
            tab_to_renderer: HashMap::new(),
            renderer_bin,
            viewport: (800, 600),
        }
    }

    fn renderer_mut(&mut self, tab_id: TabId) -> Option<&mut RendererHandle> {
        let id = self.tab_to_renderer.get(&tab_id).copied()?;
        self.manager.get_renderer(id)
    }

    fn apply_inbound_message(snap: &mut TabSnapshot, kind: IpcMessageKind) {
        match kind {
            IpcMessageKind::TitleChanged(title) => snap.title = Some(title),
            IpcMessageKind::LoadComplete => snap.loading = false,
            IpcMessageKind::LoadFailed(err) => {
                snap.loading = false;
                tracing::warn!("Renderer load failed: {err}");
            }
            IpcMessageKind::UrlChanged(url) => snap.url = Some(url),
            IpcMessageKind::ViewPainted(params) => {
                crate::paint_ipc::apply_paint_snapshot(snap, params);
            }
            _ => {}
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

    /// 确保 Tab 有对应渲染进程。
    pub fn ensure_renderer(&mut self, tab_id: TabId, viewport: (u32, u32)) {
        self.viewport = viewport;
        if self.tab_to_renderer.contains_key(&tab_id) {
            return;
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

    /// 所有 live Tab ID（LRU 冻结候选）。
    pub fn live_tab_ids(&self) -> HashMap<TabId, ()> {
        self.tab_to_renderer.keys().map(|&id| (id, ())).collect()
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

    /// 加载 HTML（多进程 IPC）。
    pub fn load_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>, url: Option<&str>) {
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: css.map(str::to_string),
                url: url.map(str::to_string),
            }),
        }) {
            tracing::warn!("IPC load_html failed: {e}");
        }
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

    /// 轮询 IPC 并更新快照。
    pub fn poll(&mut self, snapshots: &mut HashMap<TabId, TabSnapshot>) -> bool {
        let mut changed = false;
        self.manager.check_crashes();
        let mapping: Vec<(TabId, u64)> = self.tab_to_renderer.iter().map(|(k, v)| (*k, *v)).collect();
        for (tab_id, rid) in mapping {
            let Some(renderer) = self.manager.get_renderer(rid) else {
                continue;
            };
            loop {
                match renderer.try_recv() {
                    Ok(Some(msg)) => {
                        changed = true;
                        let snap = snapshots.entry(tab_id).or_default();
                        Self::apply_inbound_message(snap, msg.kind);
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tracing::debug!("IPC recv: {e}");
                        break;
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
            let renderer = self.manager.get_renderer(rid)?;
            match renderer.try_recv() {
                Ok(Some(msg)) => {
                    if msg.id == msg_id {
                        if let IpcMessageKind::HitTestLinkResult(result) = msg.kind {
                            return result.href;
                        }
                        continue;
                    }
                    let snap = snapshots.entry(tab_id).or_default();
                    Self::apply_inbound_message(snap, msg.kind);
                }
                Ok(None) => thread::sleep(Duration::from_millis(1)),
                Err(e) => {
                    tracing::debug!("IPC hit_test recv: {e}");
                    return None;
                }
            }
        }
    }
}

impl Drop for ProcessTabBackend {
    fn drop(&mut self) {
        let _ = self.manager.shutdown_all();
    }
}
