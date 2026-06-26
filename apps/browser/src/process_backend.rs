//! 可选多进程 Tab 后端 — 通过 `ProcessManager` 将页面渲染隔离到 `zero-renderer` 子进程。

use std::collections::HashMap;
use std::path::PathBuf;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
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

    /// 导航。
    pub fn navigate(&mut self, tab_id: TabId, url: &str) {
        let Some(renderer) = self.renderer_mut(tab_id) else {
            return;
        };
        if let Err(e) = renderer.navigate(url, None) {
            tracing::warn!("IPC navigate failed: {e}");
        }
    }

    /// 加载 HTML（多进程路径暂以 URL 导航代替）。
    pub fn load_html(&mut self, tab_id: TabId, _html: &str, _css: Option<&str>, url: Option<&str>) {
        self.navigate(tab_id, url.unwrap_or("about:blank"));
    }

    /// 调整视口（预留 IPC）。
    pub fn resize_all(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
    }

    /// 颜色方案（预留 IPC）。
    pub fn set_color_scheme(&mut self, _scheme: PrefersColorSchemeValue) {}

    /// 轮询 IPC 并更新快照。
    pub fn poll(&mut self, snapshots: &mut HashMap<TabId, TabSnapshot>) -> bool {
        use zero_protocol::message::IpcMessageKind;

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
                        match msg.kind {
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

    /// 链接命中测试（多进程暂未实现）。
    pub fn hit_test_link(&self, _tab_id: TabId, _x: f32, _y: f32) -> Option<String> {
        None
    }
}

impl Drop for ProcessTabBackend {
    fn drop(&mut self) {
        let _ = self.manager.shutdown_all();
    }
}
