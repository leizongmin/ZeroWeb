//! WebDriver session actor backed by a live `zero-renderer` child.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use zero_net::{HttpClient, HttpMethod, HttpRequest};
use zero_protocol::ProtocolError;
use zero_protocol::message::{
    AutomationElementRef, AutomationError, AutomationErrorCode, AutomationKey, AutomationLocatorStrategy,
    AutomationOperation, AutomationRequest, AutomationResult, AutomationStateQuery, AutomationValue, FetchParams,
    FramePublishMode, IpcMessage, IpcMessageKind, ServiceWorkerClientMessages, ServiceWorkerError,
    ServiceWorkerErrorCode, ServiceWorkerOperation, ServiceWorkerRequestParams, ServiceWorkerResponseParams,
    ServiceWorkerResult, ServiceWorkerStateChanges, SetViewportParams,
};
use zero_protocol::paint_snapshot::PaintSnapshotParams;
use zero_protocol::process::RendererHandle;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::image_cache::{ImageCache, ImageData};

const NAVIGATION_TIMEOUT: Duration = Duration::from_secs(15);
const AUTOMATION_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ELEMENT_REFERENCES: usize = 4096;

/// 会话脚本/页面加载/隐式等待超时配置（W3C timeouts endpoint 的存储态）。
///
/// 默认值刻意偏离 W3C 规范默认（300s/300s/0s）：自动化驱动场景 fail-fast 更安全，
/// 且既有集成测试依赖 15s 导航上限。见 evidence/endpoint-matrix.md 注记。
#[derive(Debug, Clone, Copy)]
pub struct SessionTimeouts {
    pub script: Duration,
    pub page_load: Duration,
    pub implicit: Duration,
}

impl Default for SessionTimeouts {
    fn default() -> Self {
        Self {
            script: AUTOMATION_TIMEOUT,
            page_load: NAVIGATION_TIMEOUT,
            implicit: Duration::ZERO,
        }
    }
}

pub struct Driver {
    sessions: HashMap<String, Session>,
    next_session_id: u64,
    renderer_bin: PathBuf,
}

/// 会话窗口句柄（单窗口架构固定值；W3C window handle 为不透明字符串）。
const WINDOW_HANDLE: &str = "zero-1";

/// 会话窗口状态（W3C WindowRect/maximize/fullscreen 面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Maximized,
    Fullscreen,
}
///
/// `history_epoch` 补 renderer back/forward 路径（`reload_history_entry`）不 bump
/// `document_generation` 的缺口——跨历史条目的引用由本层直接判 stale，防止同
/// node handle 在新文档被误复用（保守安全优先）。
struct ElementRecord {
    reference: AutomationElementRef,
    history_epoch: u64,
}

struct Session {
    renderer: RendererHandle,
    http: HttpClient,
    title: String,
    url: String,
    navigation_epoch: u64,
    history_epoch: u64,
    timeouts: SessionTimeouts,
    viewport: (u32, u32),
    window_state: WindowState,
    next_request_id: u64,
    next_element_id: u64,
    elements: HashMap<String, ElementRecord>,
    reverse_elements: HashMap<AutomationElementRef, String>,
    /// 最近一帧绘制快照（Legacy 模式 ViewPainted 维护）——screenshot 数据源。
    last_paint: Option<PaintSnapshotParams>,
    /// 跨帧累积的图片像素缓存：renderer `sent_keys` 去重后每张图只发一次 payload，
    /// screenshot 光栅化时必须能取回历史图片像素（导航不重建 renderer，缓存跨
    /// 导航保活；renderer 侧 key 随 `sent_image_keys.clear()` 重置——新页同 key
    /// 会重发 payload，`insert_with_key` 覆盖即正确语义）。
    image_cache: ImageCache,
}

#[derive(Debug)]
pub struct DriverError {
    pub code: &'static str,
    pub message: String,
}

impl DriverError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Driver {
    pub fn new() -> Result<Self, DriverError> {
        let renderer_bin = resolve_renderer_binary()
            .ok_or_else(|| DriverError::new("session not created", "zero-renderer binary not found"))?;
        Ok(Self {
            sessions: HashMap::new(),
            next_session_id: 1,
            renderer_bin,
        })
    }

    pub fn create_session(&mut self) -> Result<String, DriverError> {
        let id = format!("{:016x}", self.next_session_id);
        self.next_session_id += 1;
        let mut renderer = RendererHandle::spawn(self.renderer_bin.to_string_lossy().as_ref())
            .map_err(|error| DriverError::new("session not created", error.to_string()))?;
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetFramePublishMode(FramePublishMode::Legacy),
            })
            .map_err(protocol_error)?;
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetViewport(SetViewportParams {
                    width: 800,
                    height: 600,
                    device_scale_factor: 1.0,
                }),
            })
            .map_err(protocol_error)?;
        self.sessions.insert(
            id.clone(),
            Session {
                renderer,
                http: HttpClient::new(),
                title: String::new(),
                url: "about:blank".to_string(),
                navigation_epoch: 0,
                history_epoch: 1,
                timeouts: SessionTimeouts::default(),
                viewport: (800, 600),
                window_state: WindowState::Normal,
                next_request_id: 1,
                next_element_id: 1,
                elements: HashMap::new(),
                reverse_elements: HashMap::new(),
                last_paint: None,
                image_cache: ImageCache::new(64, 64 << 20),
            },
        );
        Ok(id)
    }

    pub fn delete_session(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    /// 查询 session 是否存活（Get Session Capabilities 用）。
    pub fn session_exists(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    pub fn navigate(&mut self, id: &str, url: &str) -> Result<(), DriverError> {
        self.session_mut(id)?.navigate(url)
    }

    /// Get Current URL。以 renderer UrlChanged 维护的会话态为准。
    pub fn url(&mut self, id: &str) -> Result<String, DriverError> {
        Ok(self.session_mut(id)?.url.clone())
    }

    /// Back。跨历史条目 → 本地 history_epoch 递增（renderer back 路径不 bump
    /// document_generation，元素引用守卫由本层负责）。
    pub fn go_back(&mut self, id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session.renderer.go_back().map_err(protocol_error)?;
        session.await_history_navigation()
    }

    /// Forward。语义同 go_back。
    pub fn go_forward(&mut self, id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session.renderer.go_forward().map_err(protocol_error)?;
        session.await_history_navigation()
    }

    /// Refresh。导航语义（epoch+1），走既有 navigate 等待路径。
    pub fn refresh(&mut self, id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session.history_epoch = session.history_epoch.wrapping_add(1).max(1);
        session
            .renderer
            .send(zero_protocol::IpcMessage {
                id: 0,
                kind: zero_protocol::message::IpcMessageKind::Reload,
            })
            .map_err(protocol_error)?;
        session.await_load_complete()
    }

    /// GET /session/{id}/timeouts。
    pub fn timeouts(&mut self, id: &str) -> Result<SessionTimeouts, DriverError> {
        Ok(self.session_mut(id)?.timeouts)
    }

    /// POST /session/{id}/timeouts。W3C：每字段可选，只更新出现的字段。
    pub fn set_timeouts(
        &mut self,
        id: &str,
        script: Option<Duration>,
        page_load: Option<Duration>,
        implicit: Option<Duration>,
    ) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        if let Some(value) = script {
            session.timeouts.script = value;
        }
        if let Some(value) = page_load {
            session.timeouts.page_load = value;
        }
        if let Some(value) = implicit {
            session.timeouts.implicit = value;
        }
        Ok(())
    }

    /// GET /window/handle（当前窗口句柄；单窗口架构）。
    pub fn window_handle(&mut self, id: &str) -> Result<&'static str, DriverError> {
        self.session_mut(id)?;
        Ok(WINDOW_HANDLE)
    }

    /// GET /session/{id}/screenshot（W3C Take Screenshot，视口语义）。
    pub fn screenshot(&mut self, id: &str) -> Result<String, DriverError> {
        let session = self.session_mut(id)?;
        session_screenshot(session)
    }

    /// GET /window/handles（单窗口 → 单元素列表）。
    pub fn window_handles(&mut self, id: &str) -> Result<Vec<&'static str>, DriverError> {
        self.session_mut(id)?;
        Ok(vec![WINDOW_HANDLE])
    }

    /// GET /window/rect。x/y 恒 0（无宿主窗口坐标）；width/height 为会话视口。
    pub fn window_rect(&mut self, id: &str) -> Result<(u32, u32, u32, u32), DriverError> {
        let session = self.session_mut(id)?;
        Ok((0, 0, session.viewport.0, session.viewport.1))
    }

    /// POST /window/rect：调整视口（经 SetViewport 既有链路）。x/y 忽略（无宿主窗口）。
    pub fn set_window_rect(&mut self, id: &str, width: u32, height: u32) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session
            .renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetViewport(SetViewportParams {
                    width,
                    height,
                    device_scale_factor: 1.0,
                }),
            })
            .map_err(protocol_error)?;
        session.viewport = (width, height);
        Ok(())
    }

    /// POST /window/maximize：单窗口 headless 场景记状态并回最大视口（无宿主窗口语义）。
    pub fn maximize_window(&mut self, id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session.window_state = WindowState::Maximized;
        Ok(())
    }

    /// POST /window/fullscreen：同 maximize 的状态记录路线。
    pub fn fullscreen_window(&mut self, id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        session.window_state = WindowState::Fullscreen;
        Ok(())
    }

    pub fn title(&mut self, id: &str) -> Result<String, DriverError> {
        let result = self.session_mut(id)?.request(AutomationOperation::ExecuteScript {
            script: "return document.title;".into(),
            arguments: Vec::new(),
        })?;
        match result {
            AutomationResult::Value(AutomationValue::String(title)) => Ok(title),
            _ => Ok(String::new()),
        }
    }

    pub fn find_element(&mut self, id: &str, selector: String) -> Result<String, DriverError> {
        let session = self.session_mut(id)?;
        let result = session.request(AutomationOperation::FindElement {
            using: AutomationLocatorStrategy::CssSelector,
            value: selector,
        })?;
        let AutomationResult::Element(Some(element)) = result else {
            return Err(DriverError::new("no such element", "element not found"));
        };
        session.register_element(element)
    }

    pub fn click_element(&mut self, id: &str, opaque_id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        session
            .request(AutomationOperation::ElementClick { element })
            .map(|_| ())
    }

    pub fn send_keys(&mut self, id: &str, opaque_id: &str, keys: Vec<AutomationKey>) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        session
            .request(AutomationOperation::SendKeys { element, keys })
            .map(|_| ())
    }

    pub fn active_element(&mut self, id: &str) -> Result<Option<String>, DriverError> {
        let session = self.session_mut(id)?;
        let result = session.request(AutomationOperation::GetActiveElement)?;
        let AutomationResult::Element(element) = result else {
            return Err(DriverError::new("unknown error", "invalid active element response"));
        };
        element.map(|element| session.register_element(element)).transpose()
    }

    /// Find Elements（复数）。空匹配返回空 Vec（W3C：非错误）。
    pub fn find_elements(&mut self, id: &str, selector: String) -> Result<Vec<String>, DriverError> {
        let session = self.session_mut(id)?;
        let result = session.request(AutomationOperation::FindElements {
            using: AutomationLocatorStrategy::CssSelector,
            value: selector,
        })?;
        let AutomationResult::Elements(references) = result else {
            return Err(DriverError::new("unknown error", "invalid find elements response"));
        };
        references
            .into_iter()
            .map(|element| session.register_element(element))
            .collect()
    }

    /// 元素状态读族公共体：按引用 + query 求值，解 AutomationValue。
    fn element_state(
        &mut self,
        id: &str,
        opaque_id: &str,
        query: AutomationStateQuery,
    ) -> Result<serde_json::Value, DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        let result = session.request(AutomationOperation::ElementState { element, query })?;
        let AutomationResult::Value(value) = result else {
            return Err(DriverError::new("unknown error", "invalid element state response"));
        };
        // renderer 对 stale 引用在 selector 解析时已报 stale element reference；
        // 找不到元素（不该发生，引用带守卫）→ no such element。
        match value {
            AutomationValue::Object(entries) => {
                let mut entries = entries.into_iter();
                let ok = entries.any(|(k, v)| k == "ok" && v == AutomationValue::Bool(true));
                if !ok {
                    return Err(DriverError::new("no such element", "element not found"));
                }
                Ok(entries
                    .find(|(k, _)| k == "value")
                    .map(|(_, v)| automation_value_to_json(v))
                    .unwrap_or(serde_json::Value::Null))
            }
            _ => Ok(automation_value_to_json(value)),
        }
    }

    pub fn element_text(&mut self, id: &str, opaque_id: &str) -> Result<String, DriverError> {
        let value = self.element_state(id, opaque_id, AutomationStateQuery::Text)?;
        Ok(value.as_str().unwrap_or_default().to_string())
    }

    pub fn element_rect(&mut self, id: &str, opaque_id: &str) -> Result<serde_json::Value, DriverError> {
        self.element_state(id, opaque_id, AutomationStateQuery::Rect)
    }

    pub fn element_enabled(&mut self, id: &str, opaque_id: &str) -> Result<bool, DriverError> {
        let value = self.element_state(id, opaque_id, AutomationStateQuery::Enabled)?;
        Ok(value.as_bool().unwrap_or(false))
    }

    pub fn element_selected(&mut self, id: &str, opaque_id: &str) -> Result<bool, DriverError> {
        let value = self.element_state(id, opaque_id, AutomationStateQuery::Selected)?;
        Ok(value.as_bool().unwrap_or(false))
    }

    pub fn element_attribute(
        &mut self,
        id: &str,
        opaque_id: &str,
        name: String,
    ) -> Result<serde_json::Value, DriverError> {
        self.element_state(id, opaque_id, AutomationStateQuery::Attribute(name))
    }

    pub fn element_property(
        &mut self,
        id: &str,
        opaque_id: &str,
        name: String,
    ) -> Result<serde_json::Value, DriverError> {
        self.element_state(id, opaque_id, AutomationStateQuery::Property(name))
    }

    pub fn element_css_value(&mut self, id: &str, opaque_id: &str, name: String) -> Result<String, DriverError> {
        let value = self.element_state(id, opaque_id, AutomationStateQuery::CssValue(name))?;
        Ok(value.as_str().unwrap_or_default().to_string())
    }

    /// Element Clear。可编辑元素置空 value。
    pub fn clear_element(&mut self, id: &str, opaque_id: &str) -> Result<(), DriverError> {
        let session = self.session_mut(id)?;
        let element = session.element(opaque_id)?;
        session
            .request(AutomationOperation::ElementClear { element })
            .map(|_| ())
    }

    /// Get Page Source。以 live document 序列化为准（ExecuteScript outerHTML）。
    pub fn page_source(&mut self, id: &str) -> Result<String, DriverError> {
        let result = self.session_mut(id)?.request(AutomationOperation::ExecuteScript {
            script: "return document.documentElement.outerHTML;".into(),
            arguments: Vec::new(),
        })?;
        let AutomationResult::Value(AutomationValue::String(source)) = result else {
            return Err(DriverError::new(
                "unknown error",
                "document source unavailable (no live document)",
            ));
        };
        Ok(source)
    }

    pub fn execute_script(
        &mut self,
        id: &str,
        script: String,
        arguments: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, DriverError> {
        let arguments = arguments.into_iter().map(automation_value_from_json).collect();
        let result = self
            .session_mut(id)?
            .request(AutomationOperation::ExecuteScript { script, arguments })?;
        let AutomationResult::Value(value) = result else {
            return Ok(serde_json::Value::Null);
        };
        Ok(automation_value_to_json(value))
    }

    /// Execute Async Script（https://w3c.github.io/webdriver/#execute-async-script）。
    ///
    /// 实现路线：脚本包 `function(arguments, callback)` 装载进页面全局 ticket 变量，
    /// callback 调用即写 ticket；随后经同步 ExecuteScript 轮询 ticket——探测间隙 renderer
    /// 主循环自然驱动 microtask/定时器回调（drain_pending_script_mutations 每拍执行），
    /// 完成条件与脚本超时都在本层（会话 timeouts.script 持有方），零协议/零 runtime.rs 改动。
    pub fn execute_script_async(
        &mut self,
        id: &str,
        script: String,
        arguments: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, DriverError> {
        let arguments_json = serde_json::Value::Array(
            arguments
                .into_iter()
                .map(automation_value_from_json)
                .map(automation_value_to_json)
                .collect(),
        );
        let session = self.session_mut(id)?;
        let ticket = format!("__zw_async_{}", session.next_request_id);
        // 1) 装载：定义全局 ticket（undefined）+ callback 写入器，立即执行脚本体。
        let install = format!(
            "(function(){{globalThis.{ticket}=undefined;\
             var __zw_callback=function(v){{globalThis.{ticket}=(v===undefined)?null:v;}};\
             (function(){{{script}\n}}).apply(null,{arguments_json}.concat([__zw_callback]));}})()"
        );
        session.request(AutomationOperation::ExecuteScript {
            script: install,
            arguments: Vec::new(),
        })?;

        // 2) 轮询：probe → 命中即读取 + 清理；未命中等待后继续（renderer 主循环推进页面任务）。
        let timeout = session.timeouts.script;
        let deadline = Instant::now() + timeout;
        loop {
            let result = session.request(AutomationOperation::ExecuteScript {
                // String() 包一层：JSON.stringify(undefined) 返 JS undefined（非字符串），
                // 未命中时脚本值会是 Null——统一为字符串 "undefined" 哨兵。
                script: format!("return String(JSON.stringify(globalThis.{ticket}));"),
                arguments: Vec::new(),
            })?;
            let AutomationResult::Value(AutomationValue::String(raw)) = result else {
                return Err(DriverError::new("unknown error", "async probe failed"));
            };
            if raw != "undefined" {
                let _ = session.request(AutomationOperation::ExecuteScript {
                    script: format!("globalThis.{ticket}=undefined;"),
                    arguments: Vec::new(),
                });
                let value = serde_json::from_str::<serde_json::Value>(&raw)
                    .map(automation_value_from_json)
                    .map(automation_value_to_json)
                    .unwrap_or(serde_json::Value::Null);
                return Ok(value);
            }
            if Instant::now() >= deadline {
                return Err(DriverError::new(
                    "javascript error",
                    format!("async script callback not called within {timeout:?}"),
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn session_mut(&mut self, id: &str) -> Result<&mut Session, DriverError> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| DriverError::new("no such session", "session not found"))
    }
}

impl Session {
    fn navigate(&mut self, url: &str) -> Result<(), DriverError> {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1).max(1);
        self.history_epoch = self.history_epoch.wrapping_add(1).max(1);
        self.title.clear();
        let timeout = self.timeouts.page_load;
        self.renderer
            .navigate(url, None, self.navigation_epoch)
            .map_err(protocol_error)?;
        self.await_load_complete_with(timeout)?;
        self.url = url.to_string();
        Ok(())
    }

    /// 等待 back/forward 触发的重载完成。
    ///
    /// renderer 历史边界 no-op（栈起点/终点直接回 Ok，不发生加载）→ 立即返回成功，
    /// 与 ChromeDriver 边界行为一致（见 master.md 决策记录）。
    fn await_history_navigation(&mut self) -> Result<(), DriverError> {
        self.history_epoch = self.history_epoch.wrapping_add(1).max(1);
        self.title.clear();
        self.await_load_complete_with(self.timeouts.page_load)
    }

    /// 以会话配置的 page load 超时等待 LoadComplete。
    fn await_load_complete(&mut self) -> Result<(), DriverError> {
        self.await_load_complete_with(self.timeouts.page_load)
    }

    fn await_load_complete_with(&mut self, timeout: Duration) -> Result<(), DriverError> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                let stderr_tail = self.renderer.stderr_tail();
                if stderr_tail.is_empty() {
                    return Err(DriverError::new("timeout", "navigation timed out"));
                }
                return Err(DriverError::new(
                    "timeout",
                    format!("navigation timed out; renderer stderr tail: {stderr_tail}"),
                ));
            }
            match self.renderer.try_recv().map_err(protocol_error)? {
                Some(message) => {
                    if let IpcMessageKind::ViewPainted(paint) = message.kind {
                        record_paint_snapshot(
                            &mut self.last_paint,
                            &mut self.image_cache,
                            self.navigation_epoch,
                            &paint,
                        );
                        continue;
                    }
                    match handle_renderer_message(
                        &self.http,
                        &mut self.title,
                        &mut self.url,
                        &mut self.renderer,
                        message,
                    )? {
                        RendererEvent::LoadComplete => return Ok(()),
                        RendererEvent::LoadFailed(message) => {
                            return Err(DriverError::new("unknown error", message));
                        }
                        RendererEvent::Other => {}
                    }
                }
                None => {
                    if !self.renderer.is_alive() {
                        return Err(DriverError::new("unknown error", "renderer exited during navigation"));
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
        }
    }

    fn request(&mut self, operation: AutomationOperation) -> Result<AutomationResult, DriverError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let http = &self.http;
        let title = &mut self.title;
        let url = &mut self.url;
        let epoch = self.navigation_epoch;
        let last_paint = &mut self.last_paint;
        let image_cache = &mut self.image_cache;
        let timeout = self.timeouts.script;
        let response = self
            .renderer
            .request_automation(
                request_id,
                AutomationRequest { operation },
                timeout,
                |renderer, message| {
                    if let IpcMessageKind::ViewPainted(paint) = &message.kind {
                        record_paint_snapshot(last_paint, image_cache, epoch, paint);
                        return Ok(());
                    }
                    handle_renderer_message(http, title, url, renderer, message)
                        .map(|_| ())
                        .map_err(|error| ProtocolError::Process(error.message))
                },
            )
            .map_err(protocol_error)?;
        response.result.map_err(automation_error)
    }

    fn register_element(&mut self, element: AutomationElementRef) -> Result<String, DriverError> {
        if let Some(existing) = self.reverse_elements.get(&element) {
            return Ok(existing.clone());
        }
        if self.elements.len() >= MAX_ELEMENT_REFERENCES {
            return Err(DriverError::new("unknown error", "element reference limit reached"));
        }
        let id = format!("e{:016x}", self.next_element_id);
        self.next_element_id += 1;
        self.elements.insert(
            id.clone(),
            ElementRecord {
                reference: element,
                history_epoch: self.history_epoch,
            },
        );
        self.reverse_elements.insert(element, id.clone());
        Ok(id)
    }

    fn element(&self, opaque_id: &str) -> Result<AutomationElementRef, DriverError> {
        let record = self
            .elements
            .get(opaque_id)
            .ok_or_else(|| DriverError::new("no such element", "unknown element reference"))?;
        if record.history_epoch != self.history_epoch {
            // https://w3c.github.io/webdriver/#dfn-stale — 跨导航/历史条目的引用一律 stale。
            return Err(DriverError::new(
                "stale element reference",
                "element belongs to an earlier navigation",
            ));
        }
        Ok(record.reference)
    }
}

enum RendererEvent {
    LoadComplete,
    LoadFailed(String),
    Other,
}

/// 记录最新一帧绘制快照并累积图片像素。
///
/// - stale 帧（epoch 不匹配）直接丢弃——browser `process_backend` 同语义。
/// - image payload 注入跨帧 `ImageCache`（S8 去重语义：renderer 每张图只发一次
///   像素，后续帧只带 key；screenshot 光栅化按 key 回查本缓存）。
fn record_paint_snapshot(
    last_paint: &mut Option<PaintSnapshotParams>,
    image_cache: &mut ImageCache,
    session_epoch: u64,
    paint: &PaintSnapshotParams,
) {
    if paint.navigation_epoch != session_epoch {
        tracing::debug!(
            "忽略 stale ViewPainted epoch {} != {}",
            paint.navigation_epoch,
            session_epoch
        );
        return;
    }
    for payload in &paint.image_payloads {
        if let Ok(data) = ImageData::from_rgba(payload.rgba.clone(), payload.width, payload.height) {
            image_cache.insert_with_key(
                zero_render_foundation::image_cache::ImageKey::new(payload.image_key),
                data,
            );
        }
    }
    *last_paint = Some((*paint).clone());
}

/// GET /session/{id}/screenshot 的会话侧实现。
///
/// 以最近一帧 ViewPainted 为源：公共转换层转 `RenderPrimitives` → CPU
/// `render_full_scene` 光栅化（与 compositor rasterize/browser headless 同路）
/// → PNG。W3C Take Screenshot 语义按视口截取（`screenWidth`/`screenHeight`
/// = 会话视口；初始 about:blank 无帧时对齐规范 unable to capture screen）。
fn session_screenshot(session: &mut Session) -> Result<String, DriverError> {
    let Some(paint) = session.last_paint.as_ref() else {
        return Err(DriverError::new(
            "unable to capture screen",
            "no painted frame available for screenshot",
        ));
    };
    let primitives = zero_paint_convert::to_render_primitives(paint.clone());
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(1024);
    let fb = zero_render_foundation::cpu::render_full_scene(
        paint.viewport_width.max(1),
        paint.viewport_height.max(1),
        if paint.device_scale_factor.is_finite() && paint.device_scale_factor > 0.0 {
            paint.device_scale_factor
        } else {
            1.0
        },
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut session.image_cache),
        &[],
        &[],
        &[],
        &[],
    );
    framebuffer_to_png_base64(&fb)
}

/// 把 RGBA8 FrameBuffer 编码为 base64 PNG（browser headless R1601 同形态）。
fn framebuffer_to_png_base64(fb: &zero_render_foundation::surface::FrameBuffer) -> Result<String, DriverError> {
    use base64::Engine;
    use png::{BitDepth, ColorType, Encoder};
    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = Encoder::new(&mut png_buf, fb.width, fb.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| DriverError::new("unknown error", e.to_string()))?;
        writer
            .write_image_data(&fb.data)
            .map_err(|e| DriverError::new("unknown error", e.to_string()))?;
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(&png_buf))
}

fn handle_renderer_message(
    http: &HttpClient,
    title: &mut String,
    url: &mut String,
    renderer: &mut RendererHandle,
    message: IpcMessage,
) -> Result<RendererEvent, DriverError> {
    let IpcMessage { id, kind } = message;
    match kind {
        IpcMessageKind::FetchRequest(params) => {
            proxy_fetch(http, renderer, params)?;
            Ok(RendererEvent::Other)
        }
        IpcMessageKind::TitleChanged(value) => {
            *title = value;
            Ok(RendererEvent::Other)
        }
        // 重定向 / hash 导航等场景 renderer 会主动发 UrlChanged——会话态 URL 跟随真值。
        IpcMessageKind::UrlChanged(value) => {
            *url = value;
            Ok(RendererEvent::Other)
        }
        IpcMessageKind::LoadComplete => Ok(RendererEvent::LoadComplete),
        IpcMessageKind::LoadFailed(message) => {
            tracing::warn!(message = %message, "webdriver observed load failed");
            Ok(RendererEvent::LoadFailed(message))
        }
        IpcMessageKind::CrashNotification(message) => {
            tracing::warn!(message = %message, "webdriver observed renderer crash");
            Ok(RendererEvent::LoadFailed(message))
        }
        IpcMessageKind::ServiceWorkerRequest(params) => {
            renderer
                .send(IpcMessage {
                    id,
                    kind: IpcMessageKind::ServiceWorkerResponse(webdriver_service_worker_response(params)),
                })
                .map_err(protocol_error)?;
            Ok(RendererEvent::Other)
        }
        _ => Ok(RendererEvent::Other),
    }
}

fn webdriver_service_worker_response(params: ServiceWorkerRequestParams) -> ServiceWorkerResponseParams {
    if let Err(message) = params.validate() {
        return service_worker_error(ServiceWorkerErrorCode::InvalidArgument, message);
    }
    let result = match params.operation {
        ServiceWorkerOperation::Controller | ServiceWorkerOperation::GetRegistration { .. } => {
            Ok(ServiceWorkerResult::OptionalSnapshot(None))
        }
        ServiceWorkerOperation::GetRegistrations => Ok(ServiceWorkerResult::Snapshots(Vec::new())),
        ServiceWorkerOperation::StateChanges { .. } => {
            Ok(ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
                latest_sequence: 0,
                states: Vec::new(),
                claim_clients: false,
            }))
        }
        ServiceWorkerOperation::ClientMessages { .. } => {
            Ok(ServiceWorkerResult::ClientMessages(ServiceWorkerClientMessages {
                latest_sequence: 0,
                messages: Vec::new(),
            }))
        }
        ServiceWorkerOperation::ObserveWindowClient { .. } | ServiceWorkerOperation::RemoveWindowClient { .. } => {
            Ok(ServiceWorkerResult::Empty)
        }
        ServiceWorkerOperation::Register { .. }
        | ServiceWorkerOperation::Snapshot { .. }
        | ServiceWorkerOperation::Unregister { .. }
        | ServiceWorkerOperation::ActivateWaiting { .. }
        | ServiceWorkerOperation::PostMessage { .. }
        | ServiceWorkerOperation::Update { .. } => Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::InvalidState,
            message: "Service Worker registration is not supported by WebDriver sessions".into(),
        }),
    };
    ServiceWorkerResponseParams { result }
}

fn service_worker_error(code: ServiceWorkerErrorCode, message: impl Into<String>) -> ServiceWorkerResponseParams {
    ServiceWorkerResponseParams {
        result: Err(ServiceWorkerError {
            code,
            message: message.into(),
        }),
    }
}

fn proxy_fetch(http: &HttpClient, renderer: &mut RendererHandle, params: FetchParams) -> Result<(), DriverError> {
    let method = match params.method.to_ascii_uppercase().as_str() {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        _ => HttpMethod::Get,
    };
    let response = http.send(HttpRequest {
        method,
        url: params.url,
        headers: params.headers,
        body: params.body,
    });
    match response {
        Ok(response) => renderer
            .send_fetch_response(params.request_id, response.status_code, response.headers, response.body)
            .map_err(protocol_error),
        Err(error) => renderer
            .send_fetch_response(params.request_id, 0, Vec::new(), error.to_string().into_bytes())
            .map_err(protocol_error),
    }
}

fn automation_error(error: AutomationError) -> DriverError {
    let code = match error.code {
        AutomationErrorCode::NoSuchElement => "no such element",
        AutomationErrorCode::StaleElementReference => "stale element reference",
        AutomationErrorCode::InvalidArgument => "invalid argument",
        AutomationErrorCode::UnsupportedOperation => "unsupported operation",
        AutomationErrorCode::JavascriptError => "javascript error",
        AutomationErrorCode::Timeout => "timeout",
        AutomationErrorCode::Internal => "unknown error",
    };
    DriverError::new(code, error.message)
}

fn protocol_error(error: ProtocolError) -> DriverError {
    let code = if error.to_string().contains("timeout") {
        "timeout"
    } else {
        "unknown error"
    };
    DriverError::new(code, error.to_string())
}

pub fn parse_webdriver_keys(text: &str) -> Vec<AutomationKey> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut shift = false;
    let flush = |result: &mut Vec<AutomationKey>, buffer: &mut String| {
        if !buffer.is_empty() {
            result.push(AutomationKey::Text(std::mem::take(buffer)));
        }
    };
    for character in text.chars() {
        match character {
            '\u{E000}' => {
                flush(&mut result, &mut buffer);
                shift = false;
            }
            '\u{E003}' => {
                flush(&mut result, &mut buffer);
                result.push(AutomationKey::Backspace);
            }
            '\u{E004}' => {
                flush(&mut result, &mut buffer);
                result.push(if shift {
                    AutomationKey::ShiftTab
                } else {
                    AutomationKey::Tab
                });
            }
            '\u{E006}' | '\u{E007}' => {
                flush(&mut result, &mut buffer);
                result.push(AutomationKey::Enter);
            }
            '\u{E008}' => {
                flush(&mut result, &mut buffer);
                shift = true;
            }
            _ => buffer.push(character),
        }
    }
    flush(&mut result, &mut buffer);
    result
}

fn automation_value_from_json(value: serde_json::Value) -> AutomationValue {
    match value {
        serde_json::Value::Null => AutomationValue::Null,
        serde_json::Value::Bool(value) => AutomationValue::Bool(value),
        serde_json::Value::Number(value) => AutomationValue::Number(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => AutomationValue::String(value),
        serde_json::Value::Array(values) => {
            AutomationValue::Array(values.into_iter().map(automation_value_from_json).collect())
        }
        serde_json::Value::Object(entries) => AutomationValue::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_from_json(value)))
                .collect(),
        ),
    }
}

fn automation_value_to_json(value: AutomationValue) -> serde_json::Value {
    match value {
        AutomationValue::Null => serde_json::Value::Null,
        AutomationValue::Bool(value) => serde_json::Value::Bool(value),
        AutomationValue::Number(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        AutomationValue::String(value) => serde_json::Value::String(value),
        AutomationValue::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(automation_value_to_json).collect())
        }
        AutomationValue::Object(entries) => serde_json::Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_to_json(value)))
                .collect(),
        ),
    }
}

fn renderer_binary_filename() -> &'static str {
    if cfg!(windows) {
        "zero-renderer.exe"
    } else {
        "zero-renderer"
    }
}

fn resolve_renderer_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("ZERO_RENDERER_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_zero-renderer") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
    {
        for directory in [Some(parent), parent.parent()].into_iter().flatten() {
            let candidate = directory.join(renderer_binary_filename());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(renderer_binary_filename()))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webdriver_special_keys_are_typed() {
        assert_eq!(
            parse_webdriver_keys("A\u{E003}\u{E008}\u{E004}\u{E000}B"),
            vec![
                AutomationKey::Text("A".into()),
                AutomationKey::Backspace,
                AutomationKey::ShiftTab,
                AutomationKey::Text("B".into()),
            ]
        );
    }

    #[test]
    fn webdriver_service_worker_queries_return_empty_results() {
        assert_eq!(
            webdriver_service_worker_response(ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::Controller,
            })
            .result,
            Ok(ServiceWorkerResult::OptionalSnapshot(None))
        );
        assert_eq!(
            webdriver_service_worker_response(ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::GetRegistrations,
            })
            .result,
            Ok(ServiceWorkerResult::Snapshots(Vec::new()))
        );
        assert_eq!(
            webdriver_service_worker_response(ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::StateChanges {
                    registration_id: 1,
                    after_sequence: 9,
                },
            })
            .result,
            Ok(ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
                latest_sequence: 0,
                states: Vec::new(),
                claim_clients: false,
            }))
        );
    }

    #[test]
    fn webdriver_service_worker_registration_returns_stable_error() {
        let response = webdriver_service_worker_response(ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Register {
                script_url: "/sw.js".into(),
                scope: None,
                document_url: "https://example.test/".into(),
                update_via_cache: zero_protocol::ServiceWorkerUpdateViaCacheWire::Imports,
                script_type: zero_protocol::ServiceWorkerScriptTypeWire::Classic,
            },
        });
        let error = response.result.expect_err("registration should be unsupported");
        assert_eq!(error.code, ServiceWorkerErrorCode::InvalidState);
        assert!(error.message.contains("WebDriver sessions"));
    }
}
