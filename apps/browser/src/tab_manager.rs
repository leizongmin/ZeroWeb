//! 标签页运行时管理 — 统一 in-process worker 与可选多进程后端。

use std::collections::HashMap;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

use crate::process_backend::{ProcessTabBackend, use_multiprocess_backend};
use crate::tab_snapshot::TabSnapshot;
use crate::tab_worker::{TabWorkerCommand, TabWorkerHandle, TabWorkerMessage};

/// 标签页运行时（worker 或多进程）的统一管理器。
pub struct TabManager {
    workers: HashMap<TabId, TabWorkerHandle>,
    snapshots: HashMap<TabId, TabSnapshot>,
    process_backend: Option<ProcessTabBackend>,
    viewport: (u32, u32),
    color_scheme: PrefersColorSchemeValue,
    pending_loaded: Vec<(TabId, String, String)>,
    pending_errors: Vec<(TabId, String)>,
    poll_tick: u64,
}

impl TabManager {
    /// 创建标签页管理器。
    pub fn new(viewport: (u32, u32), color_scheme: PrefersColorSchemeValue) -> Self {
        Self {
            workers: HashMap::new(),
            snapshots: HashMap::new(),
            process_backend: if use_multiprocess_backend() {
                Some(ProcessTabBackend::new())
            } else {
                None
            },
            viewport,
            color_scheme,
            pending_loaded: Vec::new(),
            pending_errors: Vec::new(),
            poll_tick: 0,
        }
    }

    /// 是否使用多进程后端。
    pub fn is_multiprocess(&self) -> bool {
        self.process_backend.is_some()
    }

    /// 更新默认视口（新 Tab 使用）。
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
    }

    /// 更新颜色方案并通知所有 Tab。
    pub fn set_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        if self.color_scheme == scheme {
            return;
        }
        self.color_scheme = scheme;
        for worker in self.workers.values() {
            worker.send(TabWorkerCommand::SetColorScheme(scheme));
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.set_color_scheme(scheme);
        }
    }

    /// 确保 Tab 存在 worker / 进程。
    pub fn ensure_tab(&mut self, tab_id: TabId) {
        if self.workers.contains_key(&tab_id) {
            return;
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.ensure_renderer(tab_id, self.viewport);
            self.snapshots.entry(tab_id).or_default();
            return;
        }
        let worker = TabWorkerHandle::spawn(tab_id, self.viewport, self.color_scheme);
        self.workers.insert(tab_id, worker);
        self.snapshots.entry(tab_id).or_default();
    }

    /// 移除 Tab。
    pub fn remove_tab(&mut self, tab_id: TabId) {
        self.workers.remove(&tab_id);
        self.snapshots.remove(&tab_id);
        if let Some(ref mut backend) = self.process_backend {
            backend.remove_renderer(tab_id);
        }
    }

    /// 导航到 URL。
    pub fn navigate(&mut self, tab_id: TabId, url: String) {
        self.ensure_tab(tab_id);
        if let Some(ref mut backend) = self.process_backend {
            backend.navigate(tab_id, &url);
            if let Some(snap) = self.snapshots.get_mut(&tab_id) {
                snap.loading = true;
                snap.url = Some(url);
            }
            return;
        }
        if let Some(worker) = self.workers.get(&tab_id) {
            worker.send(TabWorkerCommand::Navigate(url));
            if let Some(snap) = self.snapshots.get_mut(&tab_id) {
                snap.loading = true;
            }
        }
    }

    /// 同步加载 HTML（测试与 zero:// 页面）。
    pub fn load_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>, url: Option<&str>) {
        self.ensure_tab(tab_id);
        if let Some(ref mut backend) = self.process_backend {
            backend.load_html(tab_id, html, css, url);
            return;
        }
        if let Some(worker) = self.workers.get(&tab_id) {
            worker.send(TabWorkerCommand::LoadHtml {
                html: html.to_string(),
                css: css.map(str::to_string),
                url: url.map(str::to_string),
            });
        }
    }

    /// 调整所有 Tab 视口。
    pub fn resize_all(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
        for worker in self.workers.values() {
            worker.send(TabWorkerCommand::Resize { width, height });
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.resize_all(width, height);
        }
    }

    /// 轮询 Tab 更新快照；`active_tab` 为当前前台标签（后台 Tab 降低轮询频率）。
    pub fn poll(&mut self, active_tab: Option<TabId>) -> bool {
        #[cfg(test)]
        let _poll_guard = crate::test_sync::tab_runtime_test_guard();
        let tick = self.poll_tick;
        self.poll_tick = self.poll_tick.wrapping_add(1);
        let poll_background = tick % 5 == 0;

        let mut changed = false;
        if let Some(ref mut backend) = self.process_backend {
            changed |= backend.poll(&mut self.snapshots);
        }
        for (tab_id, worker) in &self.workers {
            let is_active = active_tab == Some(*tab_id);
            if !is_active && !poll_background {
                continue;
            }
            while let Some(msg) = worker.try_recv() {
                changed = true;
                match msg {
                    TabWorkerMessage::Snapshot(snap) => {
                        self.snapshots.insert(*tab_id, snap);
                    }
                    TabWorkerMessage::Title(title) => {
                        if let Some(s) = self.snapshots.get_mut(tab_id) {
                            s.title = Some(title.clone());
                        }
                        let url = self
                            .snapshots
                            .get(tab_id)
                            .and_then(|s| s.url.clone())
                            .unwrap_or_default();
                        self.pending_loaded.push((*tab_id, title, url));
                    }
                    TabWorkerMessage::LoadError(err) => {
                        tracing::warn!("Tab {} load error: {err}", tab_id.0);
                        if let Some(s) = self.snapshots.get_mut(tab_id) {
                            s.loading = false;
                        }
                        self.pending_errors.push((*tab_id, err));
                    }
                    TabWorkerMessage::Stage(stage) => {
                        if let Some(s) = self.snapshots.get_mut(tab_id) {
                            s.loading = !matches!(
                                stage,
                                zero_webview::PageLoadStage::Complete | zero_webview::PageLoadStage::Failed
                            );
                        }
                    }
                }
            }
        }
        changed
    }

    /// 测试用：阻塞直到所有 worker 队列清空（近似 idle）。
    #[cfg(test)]
    pub fn poll_until_idle(&mut self, max_rounds: usize) {
        for _ in 0..max_rounds {
            self.poll(None);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// 获取 Tab 快照。
    pub fn snapshot(&self, tab_id: TabId) -> Option<&TabSnapshot> {
        self.snapshots.get(&tab_id)
    }

    /// 可变快照（更新 image_cache 等）。
    pub fn snapshot_mut(&mut self, tab_id: TabId) -> Option<&mut TabSnapshot> {
        self.snapshots.get_mut(&tab_id)
    }

    /// 活跃 Tab 最近一次渲染。
    pub fn last_render(&self, tab_id: TabId) -> Option<&WebViewRenderResult> {
        self.snapshots.get(&tab_id)?.last_render.as_ref()
    }

    /// 活跃 Tab 图片缓存（绘制时可变借用）。
    pub fn image_cache_mut(&mut self, tab_id: TabId) -> Option<&mut ImageCache> {
        Some(&mut self.snapshots.get_mut(&tab_id)?.image_cache)
    }

    /// 链接命中测试。
    pub fn hit_test_link(&self, tab_id: TabId, x: f32, y: f32) -> Option<String> {
        if let Some(ref backend) = self.process_backend {
            return backend.hit_test_link(tab_id, x, y);
        }
        self.workers.get(&tab_id)?.hit_test_link(x, y)
    }

    /// 文档高度。
    pub fn document_height(&self, tab_id: TabId) -> Option<f32> {
        self.snapshots.get(&tab_id)?.document_height
    }

    /// Tab 是否仍在加载。
    pub fn is_loading(&self, tab_id: TabId) -> bool {
        self.snapshots.get(&tab_id).is_some_and(|s| s.loading)
    }

    /// 任意 Tab 是否仍在加载。
    pub fn any_loading(&self) -> bool {
        self.snapshots.values().any(|s| s.loading)
    }

    /// 取出待处理的页面加载完成事件。
    pub fn take_page_loaded_events(&mut self) -> Vec<(TabId, String, String)> {
        std::mem::take(&mut self.pending_loaded)
    }

    /// 取出待处理的页面加载失败事件。
    pub fn take_page_error_events(&mut self) -> Vec<(TabId, String)> {
        std::mem::take(&mut self.pending_errors)
    }

    /// Tab 是否已注册。
    pub fn has_tab(&self, tab_id: TabId) -> bool {
        self.workers.contains_key(&tab_id)
            || self.snapshots.contains_key(&tab_id)
            || self
                .process_backend
                .as_ref()
                .is_some_and(|_| self.snapshots.contains_key(&tab_id))
    }

    /// 测试用：逻辑视口尺寸。
    #[cfg(test)]
    pub fn logical_viewport(&self) -> (u32, u32) {
        self.viewport
    }
}
