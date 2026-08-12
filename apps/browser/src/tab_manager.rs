//! 标签页运行时管理 — 统一 in-process worker 与可选多进程后端。

use std::collections::{HashMap, VecDeque};

use zero_browser_shell::TabId;
use zero_engine::{DomEventDetail, MediaType, PrefersColorSchemeValue, selector_from_element_hit};
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

use crate::process_backend::{ProcessTabBackend, use_multiprocess_backend};
use crate::tab_snapshot::{CompositorFrame, TabSnapshot};
use crate::tab_worker::{TabWorkerCommand, TabWorkerHandle, TabWorkerMessage};

/// 异步派发 DOM 事件后，由 `TabManager` 在收到渲染进程回执时入队的后续动作。
///
/// 仿 Chrome：导航等默认动作必须等 `click` 事件的 `default_allowed` 回执到达后才执行。
#[derive(Debug, Clone)]
pub enum PendingTabAction {
    /// 在当前活动标签打开链接（普通点击）。
    NavigateActiveTab(String),
    /// 在后台新标签打开链接（Ctrl/Cmd+点击）。
    OpenBackgroundTab(String),
    /// 请求主线程重绘。
    RequestRedraw,
}

/// 一笔在途的异步派发：哪个 Tab、回执允许默认动作时要做什么。
struct PendingDispatch {
    tab_id: TabId,
    on_allowed: Option<PendingTabAction>,
}

/// 标签页运行时（worker 或多进程）的统一管理器。
pub struct TabManager {
    workers: HashMap<TabId, TabWorkerHandle>,
    snapshots: HashMap<TabId, TabSnapshot>,
    /// 每标签页快照序号（性能门禁优化 S1，2026-08-08）：新快照到达时递增，
    /// 浏览器滚动 blit 据此判断页面内容是否变更（变更则失效保留帧缓冲）。
    snapshot_seq: HashMap<TabId, u64>,
    process_backend: Option<ProcessTabBackend>,
    viewport: (u32, u32),
    color_scheme: PrefersColorSchemeValue,
    /// 当前渲染媒体类型（DC-12 @media print；transient 打印预览，非新 Tab 默认）。
    media_type: MediaType,
    pending_loaded: Vec<(TabId, String, String)>,
    pending_errors: Vec<(TabId, String)>,
    poll_tick: u64,
    /// 是否允许 Tab worker 执行页面 JavaScript。
    javascript_enabled: bool,
    /// 各 Tab 最近一次交互目标（用于 keydown 等事件派发）。
    event_targets: HashMap<TabId, String>,
    /// dispatch_id → PendingDispatch。
    pending_dispatch: HashMap<u64, PendingDispatch>,
    /// 已 resolved、等待主事件循环消费的延迟动作。
    pending_actions: VecDeque<(TabId, PendingTabAction)>,
    /// 下一个 dispatch_id。
    next_dispatch_id: u64,
}

impl TabManager {
    /// 创建标签页管理器。
    pub fn new(viewport: (u32, u32), color_scheme: PrefersColorSchemeValue) -> Self {
        let manager = Self {
            workers: HashMap::new(),
            snapshots: HashMap::new(),
            snapshot_seq: HashMap::new(),
            process_backend: if use_multiprocess_backend() {
                ProcessTabBackend::try_new()
            } else {
                None
            },
            viewport,
            color_scheme,
            media_type: MediaType::Screen,
            pending_loaded: Vec::new(),
            pending_errors: Vec::new(),
            poll_tick: 0,
            javascript_enabled: true,
            event_targets: HashMap::new(),
            pending_dispatch: HashMap::new(),
            pending_actions: VecDeque::new(),
            next_dispatch_id: 1,
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
        } else {
            crate::compositor_client::shutdown();
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

    /// 更新渲染媒体类型并通知所有 Tab（DC-12 @media print 打印预览；R1993）。
    ///
    /// 镜像 `set_color_scheme`：dedup + 持久化 + 广播 in-process workers + 多进程 backend。
    /// 语义为**transient 打印预览**（非新 Tab 默认：新 Tab 仍 Screen；打印预览是当前
    /// 已打开 Tab 的即时视图切换）。
    pub fn set_media_type(&mut self, media_type: MediaType) {
        if self.media_type == media_type {
            return;
        }
        self.media_type = media_type;
        for worker in self.workers.values() {
            worker.send(TabWorkerCommand::SetMediaType(media_type));
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.set_media_type(media_type);
        }
    }

    /// 当前渲染媒体类型（供 UI toggle 判方向；R1993）。
    pub fn media_type(&self) -> MediaType {
        self.media_type
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
    pub fn poll(&mut self, active_tab: Option<TabId>, browser_gpu_present: bool) -> bool {
        let tick = self.poll_tick;
        self.poll_tick = self.poll_tick.wrapping_add(1);
        let poll_background = tick.is_multiple_of(5);

        let mut changed = false;
        if let Some(ref mut backend) = self.process_backend {
            backend.set_browser_gpu_present(browser_gpu_present);
            changed |= backend.poll(&mut self.snapshots, &mut self.snapshot_seq, active_tab, poll_background);
            self.pending_loaded.extend(backend.take_page_loaded_events());
            self.pending_errors.extend(backend.take_page_error_events());
            for (dispatch_id, default_allowed) in backend.take_dispatch_results() {
                Self::resolve_dispatch(
                    &mut self.pending_dispatch,
                    &mut self.pending_actions,
                    dispatch_id,
                    default_allowed,
                );
            }
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
                        // 性能门禁优化 S1（2026-08-08）：快照到达 = 页面内容变更 →
                        // 递增快照序号，滚动 blit 据此失效保留帧缓冲
                        *self.snapshot_seq.entry(*tab_id).or_insert(0) += 1;
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
                    TabWorkerMessage::DispatchResult {
                        dispatch_id,
                        default_allowed,
                        ..
                    } => {
                        Self::resolve_dispatch(
                            &mut self.pending_dispatch,
                            &mut self.pending_actions,
                            dispatch_id,
                            default_allowed,
                        );
                    }
                }
            }
        }
        changed
    }

    /// 根据 default_allowed 回执决定是否触发 on_allowed 动作。
    /// 派发记录在 pending_dispatch 中移除（一次性）。
    fn resolve_dispatch(
        pending_dispatch: &mut HashMap<u64, PendingDispatch>,
        pending_actions: &mut VecDeque<(TabId, PendingTabAction)>,
        dispatch_id: u64,
        default_allowed: bool,
    ) {
        if let Some(dispatch) = pending_dispatch.remove(&dispatch_id)
            && default_allowed
            && let Some(action) = dispatch.on_allowed
        {
            pending_actions.push_back((dispatch.tab_id, action));
        }
    }

    /// 取出已 resolved 的延迟动作（由主事件循环消费）。
    pub fn take_pending_actions(&mut self) -> VecDeque<(TabId, PendingTabAction)> {
        std::mem::take(&mut self.pending_actions)
    }

    /// 测试用：阻塞直到所有 worker 队列清空（近似 idle）。
    #[cfg(test)]
    pub fn poll_until_idle(&mut self, max_rounds: usize) {
        for _ in 0..max_rounds {
            self.poll(None, false);
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

    /// 活跃 Tab 最新可显示的 compositor 页面位图。
    pub fn compositor_frame(&self, tab_id: TabId) -> Option<&CompositorFrame> {
        self.snapshots.get(&tab_id)?.compositor_frame.as_ref()
    }

    /// 活跃 Tab 最新 compositor present 全窗口位图（RFC 4.4-S3）。
    pub fn compositor_present(&self, tab_id: TabId) -> Option<&CompositorFrame> {
        self.snapshots.get(&tab_id)?.compositor_present.as_ref()
    }

    /// 克隆 present 帧 RGBA（用于 CPU present；需可变借用 image_cache）。
    pub fn compositor_present_pixels(&mut self, tab_id: TabId) -> Option<(u32, u32, Vec<u8>)> {
        let snap = self.snapshots.get_mut(&tab_id)?;
        let present = snap.compositor_present.as_ref()?;
        let img = snap.image_cache.get(&present.image_key)?;
        Some((present.width, present.height, img.pixels.clone()))
    }

    /// 活跃 Tab 图片缓存（绘制时可变借用）。
    pub fn image_cache_mut(&mut self, tab_id: TabId) -> Option<&mut ImageCache> {
        Some(&mut self.snapshots.get_mut(&tab_id)?.image_cache)
    }

    /// 链接命中测试（主线程快照）。
    ///
    /// 缓存 miss（例如页面尚未绘制首帧）时返回 `None`，不发起同步 IPC —
    /// 这样即便 renderer 进程 CPU 100%，hover 也不会拖累 UI 响应。
    pub fn hit_test_link(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<String> {
        let snap = self.snapshots.get(&tab_id)?;
        let hit_test = snap.hit_test.as_ref()?;
        hit_test.hit_test_link(x, y)
    }

    /// 图片命中测试：返回 `src`（绝对化，主线程快照）。
    pub fn hit_test_image(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<String> {
        let snap = self.snapshots.get(&tab_id)?;
        let hit_test = snap.hit_test.as_ref()?;
        hit_test.hit_test_image(x, y)
    }

    /// 元素命中测试（主线程快照）。
    pub fn hit_test_element(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<zero_engine::ElementHit> {
        let snap = self.snapshots.get(&tab_id)?;
        let hit_test = snap.hit_test.as_ref()?;
        hit_test.hit_test_element(x, y)
    }

    /// 异步向页面元素派发 DOM 事件（fire-and-forget）。
    ///
    /// `on_allowed` 在 `default_allowed = true` 时由后续 poll 触发；用于把链接导航等默认动作
    /// 延迟到事件回执确认之后。仿 Chrome：导航不会先于 click handler 的 `preventDefault`。
    fn dispatch_dom_event_async(
        &mut self,
        tab_id: TabId,
        selector: &str,
        event_type: &str,
        detail: Option<DomEventDetail>,
        on_allowed: Option<PendingTabAction>,
    ) {
        // JS 禁用：没有 handler 会 preventDefault，直接走默认动作。
        if !self.javascript_enabled {
            if let Some(action) = on_allowed {
                self.pending_actions.push_back((tab_id, action));
            }
            return;
        }
        let dispatch_id = self.next_dispatch_id;
        self.next_dispatch_id += 1;
        let key = detail.as_ref().and_then(|d| d.key.clone());
        let code = detail.as_ref().and_then(|d| d.code.clone());

        let sent = if let Some(ref mut backend) = self.process_backend {
            backend.dispatch_dom_event_fire_and_forget(tab_id, dispatch_id, Some(selector), event_type, key, code);
            true
        } else if let Some(worker) = self.workers.get(&tab_id) {
            worker.send(TabWorkerCommand::DispatchDomEvent {
                dispatch_id,
                selector: selector.to_string(),
                event_type: event_type.to_string(),
                key,
                code,
            });
            true
        } else {
            false
        };

        if !sent {
            // 没有可用的渲染端：模拟“默认允许”立即执行 on_allowed。
            if let Some(action) = on_allowed {
                self.pending_actions.push_back((tab_id, action));
            }
            return;
        }

        self.event_targets.insert(tab_id, selector.to_string());
        self.pending_dispatch
            .insert(dispatch_id, PendingDispatch { tab_id, on_allowed });
    }

    /// 向页面元素派发 DOM 事件（无默认动作）。
    pub fn dispatch_dom_event(&mut self, tab_id: TabId, selector: &str, event_type: &str) {
        self.dispatch_dom_event_async(tab_id, selector, event_type, None, None);
    }

    /// 向当前交互目标派发键盘事件（无目标时发往 `body`）。
    pub fn dispatch_key_event(&mut self, tab_id: TabId, event_type: &str, key: &str, code: &str) {
        let selector = self
            .event_targets
            .get(&tab_id)
            .cloned()
            .unwrap_or_else(|| "body".to_string());
        let detail = DomEventDetail {
            key: Some(key.to_string()),
            code: Some(code.to_string()),
            ..Default::default()
        };
        self.dispatch_dom_event_async(tab_id, &selector, event_type, Some(detail), None);
    }

    /// R3293（S0）：向页面 JS 派发「用户滚动」事件（fire-and-forget，闭合 R3253 主路径不可达 gap）。
    ///
    /// 多路径：多进程经 `process_backend.send_user_scroll` 发 `ScrollEvent` IPC（激活既有 renderer
    /// `handle_scroll_event` R3253 路径）；单进程经 `TabWorkerCommand::UserScroll` worker 线程注入
    /// `__zw_user_scroll`。JS 禁用 / 无可用渲染端时 no-op（best-effort，不影响视觉滚动）。无回执。
    /// 调用方 `apply_page_scroll_delta` 已完成视觉滚动 + 重绘，本方法仅补「页面 JS 可观察 'scroll'」半边。
    pub fn dispatch_user_scroll(&mut self, tab_id: TabId, delta_x: f32, delta_y: f32) {
        if !self.javascript_enabled {
            return;
        }
        if let Some(ref mut backend) = self.process_backend {
            backend.send_user_scroll(tab_id, delta_x, delta_y);
        } else if let Some(worker) = self.workers.get(&tab_id) {
            worker.send(TabWorkerCommand::UserScroll { delta_x, delta_y });
        }
    }

    /// 处理页面点击释放：异步派发 mouseup + click。
    ///
    /// 若鼠标命中链接，会把“导航 / 后台新标签”作为 `on_allowed` 注册到 click 的回执上，
    /// 由 `take_pending_actions` 在主事件循环中执行（仿 Chrome 延迟导航）。
    /// `background_tab` = Ctrl/Cmd+点击 → 后台新标签打开。
    pub fn dispatch_page_click(&mut self, tab_id: TabId, doc_x: f32, doc_y: f32, background_tab: bool) {
        let Some(hit) = self.hit_test_element(tab_id, doc_x, doc_y) else {
            return;
        };
        let selector = selector_from_element_hit(&hit);
        self.event_targets.insert(tab_id, selector.clone());

        // mouseup 不触发导航；click 才是浏览器选择链接导航的时机。
        self.dispatch_dom_event_async(tab_id, &selector, "mouseup", None, None);

        let on_allowed = self.hit_test_link(tab_id, doc_x, doc_y).map(|href| {
            if background_tab {
                PendingTabAction::OpenBackgroundTab(href)
            } else {
                PendingTabAction::NavigateActiveTab(href)
            }
        });
        self.dispatch_dom_event_async(tab_id, &selector, "click", None, on_allowed);
    }

    /// 处理页面按下：异步派发 mousedown。
    pub fn dispatch_page_mousedown(&mut self, tab_id: TabId, doc_x: f32, doc_y: f32) {
        if let Some(hit) = self.hit_test_element(tab_id, doc_x, doc_y) {
            let selector = selector_from_element_hit(&hit);
            self.event_targets.insert(tab_id, selector.clone());
            self.dispatch_dom_event_async(tab_id, &selector, "mousedown", None, None);
        }
    }

    /// 文档高度。
    pub fn document_height(&self, tab_id: TabId) -> Option<f32> {
        self.snapshots.get(&tab_id)?.document_height
    }

    /// 文档内容宽度估计（快照缓存，性能门禁优化 S3，2026-08-08）。
    pub fn document_width(&self, tab_id: TabId) -> Option<f32> {
        self.snapshots.get(&tab_id)?.document_width
    }

    /// 快照序号（性能门禁优化 S1，2026-08-08）：新快照到达时递增，
    /// 滚动 blit 据此失效保留帧缓冲。
    pub fn snapshot_seq(&self, tab_id: TabId) -> u64 {
        self.snapshot_seq.get(&tab_id).copied().unwrap_or(0)
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

    /// 测试用：在 tab 的渲染 worker WebView 上执行 JS 并同步回读结果（单进程路径）。
    /// 经 `TabWorkerCommand::ExecuteScriptForTest` + reply channel——worker 线程执行后回执。
    /// 供 BrowserApp 级集成测试读回页面 JS 状态（如 R3294 滚动 listener 触发计数）。
    /// 无 in-process worker / 多进程时返 Err。同步阻塞调用线程至 worker 处理完命令。
    #[cfg(test)]
    pub fn test_execute_script(&self, tab_id: TabId, script: &str) -> Result<String, String> {
        use std::sync::mpsc::channel;
        let worker = self.workers.get(&tab_id).ok_or("no in-process worker for tab")?;
        let (reply_tx, reply_rx) = channel();
        worker.send(crate::tab_worker::TabWorkerCommand::ExecuteScriptForTest {
            script: script.to_string(),
            reply: reply_tx,
        });
        // worker 线程异步处理命令；测试调用方需先经 `poll` 排空消息触发命令循环。
        // 此处阻塞等待回执（worker 处理 ExecuteScriptForTest 时 reply.send）。
        reply_rx
            .recv()
            .map_err(|e| format!("worker reply channel closed: {e}"))?
    }
}
