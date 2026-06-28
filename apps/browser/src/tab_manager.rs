//! 标签页运行时管理 — 统一 in-process worker 与可选多进程后端。

use std::collections::HashMap;

use zero_browser_shell::TabId;
use zero_engine::{DomEventDetail, PrefersColorSchemeValue, selector_from_element_hit};
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

use crate::process_backend::{ProcessTabBackend, use_multiprocess_backend};
use crate::tab_scripts::DomDispatchResult;
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
    /// 是否允许 Tab worker 执行页面 JavaScript。
    javascript_enabled: bool,
    /// 各 Tab 最近一次交互目标（用于 keydown 等事件派发）。
    event_targets: HashMap<TabId, String>,
}

impl TabManager {
    /// 创建标签页管理器。
    pub fn new(viewport: (u32, u32), color_scheme: PrefersColorSchemeValue) -> Self {
        let manager = Self {
            workers: HashMap::new(),
            snapshots: HashMap::new(),
            process_backend: if use_multiprocess_backend() {
                ProcessTabBackend::try_new()
            } else {
                None
            },
            viewport,
            color_scheme,
            pending_loaded: Vec::new(),
            pending_errors: Vec::new(),
            poll_tick: 0,
            javascript_enabled: true,
            event_targets: HashMap::new(),
        };
        if use_multiprocess_backend() && manager.process_backend.is_none() {
            tracing::info!("Tab runtime: in-process workers (zero-renderer not available)");
        } else if manager.process_backend.is_some() {
            tracing::info!("Tab runtime: multi-process (zero-renderer child processes)");
        }
        manager
    }

    /// 是否使用多进程后端。
    pub fn is_multiprocess(&self) -> bool {
        self.process_backend.is_some()
    }

    /// 显式终止所有渲染子进程。
    ///
    /// 供 `BrowserApp::shutdown_child_processes` 调用，确保 `std::process::exit`
    /// 之前子进程被 kill；`Drop` 不会被 `process::exit` 触发。
    pub fn shutdown_child_processes(&mut self) {
        if let Some(ref mut backend) = self.process_backend {
            backend.shutdown_all();
        }
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
        if let Some(ref mut backend) = self.process_backend {
            backend.remove_renderer(tab_id);
        }
    }

    /// 标记 Tab 为无痕（多进程 fetch 不写磁盘缓存）。
    pub fn set_tab_private(&mut self, tab_id: TabId, private: bool) {
        if let Some(ref mut backend) = self.process_backend {
            backend.set_tab_private(tab_id, private);
        }
    }

    /// 前台 Tab 切换（每个 Tab 的 worker / 渲染进程保持存活直至用户关闭标签）。
    pub fn on_active_tab_changed(&mut self, _active: Option<TabId>) {}

    /// 导航到 URL。
    pub fn navigate(&mut self, tab_id: TabId, url: String) {
        self.ensure_tab(tab_id);
        if let Some(snap) = self.snapshots.get_mut(&tab_id) {
            snap.begin_navigation(url.clone());
        }
        if let Some(ref mut backend) = self.process_backend {
            let epoch = self.snapshots.get(&tab_id).map(|s| s.navigation_epoch).unwrap_or(0);
            backend.navigate(tab_id, &url, epoch);
            return;
        }
        if let Some(worker) = self.workers.get(&tab_id) {
            worker.send(TabWorkerCommand::Navigate(url));
        }
    }

    /// 强制刷新：清除该 URL 的缓存条目后再 navigate（绕过 HTTP 缓存）。
    pub fn navigate_bypass_cache(&mut self, tab_id: TabId, url: String) {
        self.ensure_tab(tab_id);
        if let Some(ref backend) = self.process_backend {
            backend.invalidate_url_cache(tab_id, &url);
        }
        // 清除后走正常 navigate 流程。
        self.navigate(tab_id, url);
    }

    /// 同步加载 HTML（测试与 zero:// 页面）。
    pub fn load_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>, url: Option<&str>) {
        self.ensure_tab(tab_id);
        if let Some(snap) = self.snapshots.get_mut(&tab_id) {
            snap.begin_navigation(url.unwrap_or("about:blank").to_string());
        }
        if let Some(ref mut backend) = self.process_backend {
            let epoch = self.snapshots.get(&tab_id).map(|s| s.navigation_epoch).unwrap_or(0);
            backend.load_html(tab_id, html, css, url, epoch);
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
        let poll_background = tick.is_multiple_of(5);

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

    /// 链接命中测试（主线程快照，不阻塞渲染进程）。
    pub fn hit_test_link(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<String> {
        if let Some(snap) = self.snapshots.get(&tab_id)
            && let Some(hit_test) = snap.hit_test.as_ref()
            && let Some(href) = hit_test.hit_test_link(x, y)
        {
            return Some(href);
        }
        if let Some(ref mut backend) = self.process_backend {
            return backend.hit_test_link(tab_id, x, y, &mut self.snapshots);
        }
        None
    }

    /// 元素命中测试（主线程快照，不阻塞渲染进程）。
    pub fn hit_test_element(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<zero_engine::ElementHit> {
        if let Some(snap) = self.snapshots.get(&tab_id)
            && let Some(hit_test) = snap.hit_test.as_ref()
            && let Some(hit) = hit_test.hit_test_element(x, y)
        {
            return Some(hit);
        }
        if let Some(ref mut backend) = self.process_backend {
            return backend.hit_test_element(tab_id, x, y, &mut self.snapshots);
        }
        None
    }

    /// 向页面元素派发 DOM 事件并返回是否允许默认行为。
    pub fn dispatch_dom_event(&mut self, tab_id: TabId, selector: &str, event_type: &str) -> bool {
        self.dispatch_dom_event_impl(tab_id, selector, event_type, None)
            .default_allowed
    }

    fn dispatch_dom_event_impl(
        &mut self,
        tab_id: TabId,
        selector: &str,
        event_type: &str,
        detail: Option<DomEventDetail>,
    ) -> DomDispatchResult {
        if !self.javascript_enabled {
            return DomDispatchResult {
                default_allowed: true,
                html_changed: false,
            };
        }
        if let Some(ref mut backend) = self.process_backend {
            let result = backend.dispatch_dom_event(
                tab_id,
                Some(selector),
                0.0,
                0.0,
                event_type,
                detail.as_ref(),
                &mut self.snapshots,
            );
            if result.html_changed {
                self.poll(Some(tab_id));
            }
            self.event_targets.insert(tab_id, selector.to_string());
            return result;
        }
        let result = self
            .workers
            .get(&tab_id)
            .map(|w| {
                w.dispatch_dom_event(
                    selector,
                    event_type,
                    detail.as_ref().and_then(|d| d.key.as_deref()),
                    detail.as_ref().and_then(|d| d.code.as_deref()),
                )
            })
            .unwrap_or(DomDispatchResult {
                default_allowed: true,
                html_changed: false,
            });
        self.event_targets.insert(tab_id, selector.to_string());
        if result.html_changed {
            self.poll(Some(tab_id));
        }
        result
    }

    /// 向当前交互目标派发键盘事件（无目标时发往 `body`）。
    pub fn dispatch_key_event(&mut self, tab_id: TabId, event_type: &str, key: &str, code: &str) -> bool {
        let selector = self
            .event_targets
            .get(&tab_id)
            .cloned()
            .unwrap_or_else(|| "body".to_string());
        let detail = DomEventDetail {
            key: Some(key.to_string()),
            code: Some(code.to_string()),
        };
        self.dispatch_dom_event_impl(tab_id, &selector, event_type, Some(detail))
            .html_changed
    }

    /// 处理页面点击释放：派发 mouseup/click，返回是否允许默认行为（链接导航）。
    pub fn dispatch_page_click(&mut self, tab_id: TabId, doc_x: f32, doc_y: f32) -> bool {
        if let Some(hit) = self.hit_test_element(tab_id, doc_x, doc_y) {
            let selector = selector_from_element_hit(&hit);
            self.event_targets.insert(tab_id, selector.clone());
            let up = self.dispatch_dom_event_impl(tab_id, &selector, "mouseup", None);
            let click = self.dispatch_dom_event_impl(tab_id, &selector, "click", None);
            return up.default_allowed && click.default_allowed;
        }
        true
    }

    /// 处理页面按下：派发 mousedown。
    pub fn dispatch_page_mousedown(&mut self, tab_id: TabId, doc_x: f32, doc_y: f32) {
        if let Some(hit) = self.hit_test_element(tab_id, doc_x, doc_y) {
            let selector = selector_from_element_hit(&hit);
            self.event_targets.insert(tab_id, selector.clone());
            self.dispatch_dom_event_impl(tab_id, &selector, "mousedown", None);
        }
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
