//! 标签页运行时管理 — 统一 in-process worker 与可选多进程后端。

use std::collections::HashMap;

use zero_browser_shell::TabId;
use zero_engine::PrefersColorSchemeValue;
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

use crate::process_backend::{ProcessTabBackend, use_multiprocess_backend};
use crate::tab_lru::TabLruPolicy;
use crate::tab_restore::TabRestorePayload;
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
    lru: TabLruPolicy,
    last_active: Option<TabId>,
    /// 最近一次加载方式（LRU 解冻用）。
    last_restore: HashMap<TabId, TabRestorePayload>,
    /// 是否允许 Tab worker 执行页面 JavaScript。
    javascript_enabled: bool,
}

impl TabManager {
    /// 创建标签页管理器。
    pub fn new(viewport: (u32, u32), color_scheme: PrefersColorSchemeValue) -> Self {
        let manager = Self {
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
            lru: TabLruPolicy::default(),
            last_active: None,
            last_restore: HashMap::new(),
            javascript_enabled: true,
        };
        let max_live = manager.lru.max_live();
        tracing::info!("Tab LRU max live workers: {max_live} (ZERO_BROWSER_MAX_LIVE_TABS)");
        manager
    }

    /// 是否使用多进程后端。
    pub fn is_multiprocess(&self) -> bool {
        self.process_backend.is_some()
    }

    /// 更新是否允许执行页面 JavaScript。
    pub fn set_javascript_enabled(&mut self, enabled: bool) {
        if self.javascript_enabled == enabled {
            return;
        }
        self.javascript_enabled = enabled;
        for worker in self.workers.values() {
            worker.send(TabWorkerCommand::SetJavascriptEnabled(enabled));
        }
    }

    /// 读取 Tab 页面 HTML 源码（快照）。
    pub fn page_html(&self, tab_id: TabId) -> Option<String> {
        self.snapshots.get(&tab_id)?.html_source.clone()
    }

    /// 读取 Tab 当前 URL（快照）。
    pub fn page_url(&self, tab_id: TabId) -> Option<String> {
        self.snapshots.get(&tab_id)?.url.clone()
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
        if self.lru.is_frozen(tab_id) {
            self.thaw_tab_if_frozen(tab_id);
        }
        if self.workers.contains_key(&tab_id) {
            return;
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.ensure_renderer(tab_id, self.viewport);
            self.snapshots.entry(tab_id).or_default();
            return;
        }
        let worker = TabWorkerHandle::spawn(tab_id, self.viewport, self.color_scheme);
        worker.send(TabWorkerCommand::SetJavascriptEnabled(self.javascript_enabled));
        self.workers.insert(tab_id, worker);
        self.snapshots.entry(tab_id).or_default();
    }

    /// 移除 Tab。
    pub fn remove_tab(&mut self, tab_id: TabId) {
        if let Some(mut worker) = self.workers.remove(&tab_id) {
            worker.shutdown();
        }
        self.snapshots.remove(&tab_id);
        self.lru.remove_tab(tab_id);
        self.last_restore.remove(&tab_id);
        if let Some(ref mut backend) = self.process_backend {
            backend.remove_renderer(tab_id);
        }
    }

    /// 前台 Tab 切换 — 触发 LRU 冻结/解冻。
    pub fn on_active_tab_changed(&mut self, active: Option<TabId>) {
        if let Some(prev) = self.last_active
            && Some(prev) != active
        {
            self.lru.note_deactivated(prev);
        }
        self.last_active = active;

        if let Some(id) = active {
            if self.lru.is_frozen(id) {
                self.thaw_tab_if_frozen(id);
            }
        }
        self.enforce_lru_limit(active);
    }

    fn apply_restore(&mut self, tab_id: TabId, payload: TabRestorePayload) {
        self.ensure_tab(tab_id);
        match payload {
            TabRestorePayload::Navigate(url) => {
                self.navigate(tab_id, url);
            }
            TabRestorePayload::LoadHtml { html, css, url } => {
                self.load_html(tab_id, &html, css.as_deref(), url.as_deref());
            }
        }
    }

    fn thaw_tab_if_frozen(&mut self, tab_id: TabId) {
        if !self.lru.is_frozen(tab_id) {
            return;
        }
        let restore = self.lru.thaw(tab_id);

        if let Some(ref mut backend) = self.process_backend {
            if backend.has_renderer(tab_id) {
                return;
            }
            backend.ensure_renderer(tab_id, self.viewport);
            if let Some(payload) = restore.or_else(|| {
                self.last_restore.get(&tab_id).cloned().or_else(|| {
                    self.snapshots
                        .get(&tab_id)
                        .and_then(|s| s.url.as_ref().map(|u| TabRestorePayload::from_url(u)))
                })
            }) {
                self.apply_restore(tab_id, payload);
            }
            return;
        }

        if self.workers.contains_key(&tab_id) {
            return;
        }
        let worker = TabWorkerHandle::spawn(tab_id, self.viewport, self.color_scheme);
        if let Some(payload) = restore.or_else(|| {
            self.last_restore.get(&tab_id).cloned().or_else(|| {
                self.snapshots
                    .get(&tab_id)
                    .and_then(|s| s.url.as_ref().map(|u| TabRestorePayload::from_url(u)))
            })
        }) {
            match payload {
                TabRestorePayload::Navigate(url) => {
                    worker.send(TabWorkerCommand::Navigate(url));
                }
                TabRestorePayload::LoadHtml { html, css, url } => {
                    worker.send(TabWorkerCommand::LoadHtml { html, css, url });
                }
            }
            if let Some(snap) = self.snapshots.get_mut(&tab_id) {
                snap.loading = true;
            }
        }
        self.workers.insert(tab_id, worker);
    }

    fn enforce_lru_limit(&mut self, active: Option<TabId>) {
        loop {
            let live_count = if let Some(ref backend) = self.process_backend {
                backend.live_renderer_count()
            } else {
                self.workers.len()
            };
            if !self.lru.should_freeze(live_count, active) {
                break;
            }
            let live = if let Some(ref backend) = self.process_backend {
                backend.live_tab_ids()
            } else {
                self.workers.keys().map(|&id| (id, ())).collect()
            };
            let Some(victim) = self.lru.pick_freeze_victim(active, &live) else {
                break;
            };
            self.freeze_tab(victim);
        }
    }

    fn freeze_tab(&mut self, tab_id: TabId) {
        let restore = self.last_restore.get(&tab_id).cloned().or_else(|| {
            self.snapshots
                .get(&tab_id)
                .and_then(|s| s.url.as_ref().map(|u| TabRestorePayload::from_url(u)))
        });
        if let Some(ref mut backend) = self.process_backend {
            backend.remove_renderer(tab_id);
        } else if let Some(mut worker) = self.workers.remove(&tab_id) {
            worker.shutdown();
        }
        if let Some(snap) = self.snapshots.get_mut(&tab_id) {
            snap.image_cache.clear();
        }
        self.lru.mark_frozen(tab_id, restore);
        tracing::debug!("Froze tab {} (LRU, max {})", tab_id.0, self.lru.max_live());
    }

    /// 导航到 URL。
    pub fn navigate(&mut self, tab_id: TabId, url: String) {
        self.last_restore
            .insert(tab_id, TabRestorePayload::Navigate(url.clone()));
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
        self.last_restore.insert(
            tab_id,
            TabRestorePayload::LoadHtml {
                html: html.to_string(),
                css: css.map(str::to_string),
                url: url.map(str::to_string),
            },
        );
        self.ensure_tab(tab_id);
        if let Some(ref mut backend) = self.process_backend {
            backend.load_html(tab_id, html, css, url);
            if let Some(snap) = self.snapshots.get_mut(&tab_id) {
                snap.loading = true;
                snap.url = url.map(str::to_string);
            }
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
            changed |= backend.poll(&mut self.snapshots, active_tab, poll_background);
            self.pending_loaded.extend(backend.take_page_loaded_events());
            self.pending_errors.extend(backend.take_page_error_events());
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
    pub fn hit_test_link(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<String> {
        if let Some(ref mut backend) = self.process_backend {
            return backend.hit_test_link(tab_id, x, y, &mut self.snapshots);
        }
        self.workers.get(&tab_id)?.hit_test_link(x, y)
    }

    /// 元素命中测试（审查元素）。
    pub fn hit_test_element(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<zero_engine::ElementHit> {
        if self.process_backend.is_some() {
            return None;
        }
        self.workers.get(&tab_id)?.hit_test_element(x, y)
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

    /// 测试用：确保存在快照条目（不启动 worker）。
    #[cfg(test)]
    pub fn ensure_snapshot_for_test(&mut self, tab_id: TabId) {
        self.snapshots.entry(tab_id).or_default();
    }

    /// 测试用：逻辑视口尺寸。
    #[cfg(test)]
    pub fn logical_viewport(&self) -> (u32, u32) {
        self.viewport
    }
}
