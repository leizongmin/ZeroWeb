//! WebView 主类型 — 可嵌入的网页渲染表面。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zero_engine::{PipelineTimings, RenderPipeline};
use zero_net::{HttpClient, NetError};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_script_sandbox::{SandboxConfig, WorkerEvent, WorkerRuntime};
use zero_storage::{CacheRequest, FetchInterceptResult, ServiceWorkerRegistry};

use crate::WebViewError;

/// WebView 配置。
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// 视口宽度。
    pub width: u32,
    /// 视口高度。
    pub height: u32,
    /// 是否透明背景。
    pub transparent: bool,
    /// 用户代理字符串。
    pub user_agent: Option<String>,
    /// 初始 URL。
    pub url: Option<String>,
    /// 是否启用开发者工具。
    pub devtools: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            transparent: false,
            user_agent: None,
            url: None,
            devtools: false,
        }
    }
}

/// WebView 渲染结果。
#[derive(Debug, Clone)]
pub struct WebViewRenderResult {
    /// 渲染图元。
    pub primitives: RenderPrimitives,
    /// 管线耗时。
    pub timings: PipelineTimings,
}

/// WebView 事件回调。
#[derive(Debug, Clone)]
pub enum WebViewEvent {
    /// 页面开始加载。
    LoadStart(String),
    /// 页面加载完成。
    LoadEnd(String),
    /// 页面加载失败。
    LoadFailed(String, String),
    /// 标题变更。
    TitleChanged(String),
    /// URL 变更。
    UrlChanged(String),
}

/// 事件回调函数类型。
pub type EventCallback = Rc<RefCell<dyn FnMut(&WebViewEvent)>>;

/// WebView — 可嵌入的网页渲染表面。
pub struct WebView {
    /// 配置。
    config: WebViewConfig,
    /// 渲染管线。
    pipeline: RenderPipeline,
    /// HTTP 客户端。
    http_client: HttpClient,
    /// JavaScript 沙箱。
    js_sandbox: zero_script_sandbox::V8Sandbox,
    /// 当前 URL。
    current_url: Option<String>,
    /// 页面标题。
    title: Option<String>,
    /// 是否正在加载。
    loading: bool,
    /// 上次渲染结果。
    last_render: Option<WebViewRenderResult>,
    /// 缓存的 HTML（用于 inject_css 重新渲染）。
    cached_html: String,
    /// 缓存的 CSS（用于 render 重新渲染）。
    cached_css: String,
    /// 事件回调列表。
    event_callbacks: Vec<EventCallback>,
    /// Service Worker 注册表。
    sw_registry: ServiceWorkerRegistry,
    /// Web Worker 实例（Dedicated Worker）。
    workers: HashMap<u64, WorkerRuntime>,
    /// Worker ID 生成器。
    next_worker_id: u64,
}

impl WebView {
    /// 创建新的 WebView。
    pub fn new(config: WebViewConfig) -> Self {
        let pipeline = RenderPipeline::new(config.width as f32, config.height as f32);
        let http_client = HttpClient::new();
        let js_sandbox = zero_script_sandbox::V8Sandbox::new().expect("V8 sandbox initialization should succeed");
        Self {
            config,
            pipeline,
            http_client,
            js_sandbox,
            current_url: None,
            title: None,
            loading: false,
            last_render: None,
            cached_html: String::new(),
            cached_css: String::new(),
            event_callbacks: Vec::new(),
            sw_registry: ServiceWorkerRegistry::new(),
            workers: HashMap::new(),
            next_worker_id: 1,
        }
    }

    /// 注册事件回调。
    ///
    /// 回调在 load_html / load_url / fetch_url 等操作触发状态变更时调用。
    /// 返回回调的索引，可用于后续移除。
    pub fn on_event(&mut self, callback: impl FnMut(&WebViewEvent) + 'static) -> usize {
        let idx = self.event_callbacks.len();
        self.event_callbacks.push(Rc::new(RefCell::new(callback)));
        idx
    }

    /// 移除事件回调。
    ///
    /// 传入 `on_event` 返回的索引。返回 `true` 表示成功移除。
    pub fn remove_event_callback(&mut self, index: usize) -> bool {
        if index < self.event_callbacks.len() {
            self.event_callbacks.remove(index);
            true
        } else {
            false
        }
    }

    /// 内部：分发事件到所有已注册的回调。
    fn emit_event(&self, event: &WebViewEvent) {
        for callback in &self.event_callbacks {
            let mut cb = callback.borrow_mut();
            cb(event);
        }
    }

    /// 加载 HTML 内容。
    pub fn load_html(&mut self, html: &str, css: Option<&str>) -> WebViewRenderResult {
        self.cached_html = html.to_string();
        let css_str = css.unwrap_or("");
        self.cached_css = css_str.to_string();
        let result = self.pipeline.render_html(html, css_str);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 从 URL 中提取 origin（scheme + host + port）。
    ///
    /// `"https://example.com:8443/path?q=1"` → `"https://example.com:8443"`
    pub fn extract_origin(url: &str) -> Option<String> {
        url::Url::parse(url).ok().map(|u| u.origin().ascii_serialization())
    }

    /// 加载 URL（同步 HTTP GET）。
    ///
    /// 通过 zero-net 发起 HTTP 请求，获取 HTML 并渲染。
    /// 整个过程是同步阻塞的。
    /// 如果请求失败，加载状态会被重置，并返回错误。
    pub fn fetch_url(&mut self, url: &str) -> Result<WebViewRenderResult, WebViewError> {
        tracing::info!("Fetching URL: {url}");

        // 设置加载状态
        let old_url = self.current_url.clone();
        self.current_url = Some(url.to_string());
        self.loading = true;
        self.emit_event(&WebViewEvent::LoadStart(url.to_string()));

        if old_url.as_deref() != Some(url) {
            self.emit_event(&WebViewEvent::UrlChanged(url.to_string()));
        }

        // 尝试 Service Worker 拦截
        if let Some(origin) = Self::extract_origin(url) {
            let request = CacheRequest::new(url);
            match self.sw_registry.intercept_fetch(&request, &origin) {
                FetchInterceptResult::Cached(response) | FetchInterceptResult::Responded(response) => {
                    tracing::info!("Service Worker intercepted fetch for {url}");
                    let html = String::from_utf8(response.body).map_err(|e| {
                        self.loading = false;
                        self.emit_event(&WebViewEvent::LoadFailed(
                            url.to_string(),
                            format!("SW response body is not valid UTF-8: {e}"),
                        ));
                        WebViewError::Navigation(format!("SW response body is not valid UTF-8: {e}"))
                    })?;
                    let render_result = self.load_html(&html, None);
                    self.loading = false;
                    self.emit_event(&WebViewEvent::LoadEnd(url.to_string()));
                    return Ok(render_result);
                }
                _ => {
                    // PassThrough / NoWorker / Error — 继续正常网络请求
                }
            }
        }

        // 发起 HTTP 请求
        match self.http_client.get(url) {
            Ok(response) => {
                let html = response.text().map_err(|e| {
                    self.loading = false;
                    self.emit_event(&WebViewEvent::LoadFailed(
                        url.to_string(),
                        format!("Failed to decode response body: {e}"),
                    ));
                    WebViewError::Navigation(format!("Failed to decode response body: {e}"))
                })?;

                tracing::info!("Fetched {} bytes from {url}", html.len());

                // 渲染 HTML
                let render_result = self.load_html(&html, None);
                self.loading = false;
                self.emit_event(&WebViewEvent::LoadEnd(url.to_string()));
                Ok(render_result)
            }
            Err(NetError::Timeout) => {
                self.loading = false;
                let msg = format!("Request to {url} timed out");
                self.emit_event(&WebViewEvent::LoadFailed(url.to_string(), msg.clone()));
                Err(WebViewError::Navigation(msg))
            }
            Err(e) => {
                self.loading = false;
                let msg = format!("Failed to fetch {url}: {e}");
                self.emit_event(&WebViewEvent::LoadFailed(url.to_string(), msg.clone()));
                Err(WebViewError::Navigation(msg))
            }
        }
    }

    /// 加载 URL（非阻塞 — 仅设置状态）。
    ///
    /// 仅更新 URL 和 loading 标志，不发起网络请求。
    /// 用于需要异步/外部驱动的加载场景。
    /// 调用方应随后调用 `fetch_url` 或 `complete_load` 来完成加载。
    pub fn load_url(&mut self, url: &str) {
        let old_url = self.current_url.clone();
        self.current_url = Some(url.to_string());
        self.loading = true;
        self.emit_event(&WebViewEvent::LoadStart(url.to_string()));
        if old_url.as_deref() != Some(url) {
            self.emit_event(&WebViewEvent::UrlChanged(url.to_string()));
        }
    }

    /// 完成加载（手动标记加载结束并渲染 HTML）。
    ///
    /// 用于配合 `load_url` 使用：先 `load_url` 设置状态，
    /// 外部获取到 HTML 内容后调用 `complete_load` 渲染并结束加载。
    pub fn complete_load(&mut self, html: &str, css: Option<&str>) -> WebViewRenderResult {
        let url = self.current_url.clone().unwrap_or_default();
        let result = self.load_html(html, css);
        self.loading = false;
        self.emit_event(&WebViewEvent::LoadEnd(url));
        result
    }

    /// 标记加载失败。
    pub fn fail_load(&mut self, error: &str) {
        let url = self.current_url.clone().unwrap_or_default();
        self.loading = false;
        self.emit_event(&WebViewEvent::LoadFailed(url, error.to_string()));
    }

    /// 重新渲染（用于 resize 等场景）。
    pub fn render(&mut self) -> WebViewRenderResult {
        let result = self.pipeline.render_html(&self.cached_html, &self.cached_css);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 获取当前 URL。
    pub fn url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    /// 设置页面标题（触发 TitleChanged 事件）。
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
        self.emit_event(&WebViewEvent::TitleChanged(title.to_string()));
    }

    /// 获取页面标题。
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// 获取配置。
    pub fn config(&self) -> &WebViewConfig {
        &self.config
    }

    /// 是否正在加载。
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// 获取上次渲染结果。
    pub fn last_render(&self) -> Option<&WebViewRenderResult> {
        self.last_render.as_ref()
    }

    /// 调整大小。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.pipeline = RenderPipeline::new(width as f32, height as f32);
    }

    /// 执行 JavaScript。
    ///
    /// 需要 zero-script-sandbox 后端引擎（V8/QuickJS）。
    /// 当前尚未集成 JS 引擎，返回 `WebViewError::NotImplemented`。
    /// 执行 JavaScript 脚本。
    ///
    /// 在 WebView 的 JavaScript 沙箱中执行脚本，返回结果的字符串表示。
    ///
    /// # 错误
    ///
    /// - [`WebViewError::Script`] — 脚本编译或运行时错误
    /// - [`WebViewError::InvalidInput`] — 脚本为空
    pub fn execute_script(&mut self, script: &str) -> Result<String, WebViewError> {
        tracing::debug!("execute_script called: {} bytes", script.len());

        match self.js_sandbox.execute(script) {
            Ok(result) => {
                tracing::debug!("execute_script completed in {:.2}ms", result.execution_time_ms);
                Ok(result.value)
            }
            Err(e) => Err(WebViewError::Script(format!("{e}"))),
        }
    }

    /// 执行带有 DOM API 环境的 JavaScript。
    ///
    /// 在执行用户脚本前，先注入 DOM API polyfill，
    /// 使得脚本可以使用 `document.getElementById` 等 DOM 操作。
    ///
    /// # 错误
    ///
    /// 与 [`execute_script`](Self::execute_script) 相同。
    pub fn execute_script_with_dom(&mut self, script: &str) -> Result<String, WebViewError> {
        tracing::debug!("execute_script_with_dom called: {} bytes", script.len());

        let polyfill = zero_engine::generate_dom_api_polyfill();
        let full_script = format!("{polyfill}\n{script}");

        self.execute_script(&full_script)
    }

    /// 注入 CSS（重新渲染）。
    pub fn inject_css(&mut self, css: &str) -> WebViewRenderResult {
        let html = if self.cached_html.is_empty() {
            "<html><body></body></html>"
        } else {
            &self.cached_html
        };
        // 追加到缓存的 CSS，而不是替换
        if !self.cached_css.is_empty() {
            self.cached_css.push('\n');
        }
        self.cached_css.push_str(css);
        let result = self.pipeline.render_html(html, &self.cached_css);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 注册 Service Worker。
    ///
    /// 返回新注册的 ID。
    pub fn register_service_worker(&mut self, script_url: &str, scope: &str, origin: &str) -> u64 {
        self.sw_registry.register(script_url, scope, origin)
    }

    /// 安装 Service Worker。
    ///
    /// 将指定 ID 的 SW 推进到 `Installed` 状态。
    pub fn install_service_worker(&mut self, id: u64) -> bool {
        self.sw_registry.install(id)
    }

    /// 激活 Service Worker。
    ///
    /// 将指定 ID 的 SW 推进到 `Activated` 状态，使其可以拦截 fetch 请求。
    pub fn activate_service_worker(&mut self, id: u64) -> bool {
        self.sw_registry.activate(id)
    }

    /// 注销 Service Worker。
    pub fn unregister_service_worker(&mut self, id: u64) -> bool {
        self.sw_registry.unregister(id)
    }

    /// 获取 Service Worker 注册表（只读）。
    pub fn service_worker_registry(&self) -> &ServiceWorkerRegistry {
        &self.sw_registry
    }

    /// 获取 Service Worker 注册表（可变）。
    pub fn service_worker_registry_mut(&mut self) -> &mut ServiceWorkerRegistry {
        &mut self.sw_registry
    }

    /// 编译并执行 WASM 模块。
    ///
    /// 使用 zero-wasm-sandbox 编译 WASM 字节码，实例化后调用指定的导出函数。
    /// 返回函数调用结果的字符串表示。
    ///
    /// # 参数
    /// - `wasm_bytes`: WASM 模块的二进制字节
    /// - `function_name`: 要调用的导出函数名
    /// - `args`: 传递给函数的参数
    ///
    /// # 错误
    /// - [`WebViewError::Script`] — WASM 编译、实例化或调用错误
    pub fn execute_wasm(
        &self,
        wasm_bytes: &[u8],
        function_name: &str,
        args: &[zero_wasm_sandbox::WasmValue],
    ) -> Result<String, WebViewError> {
        tracing::debug!("execute_wasm: {} bytes, function: {}", wasm_bytes.len(), function_name);

        let sandbox = zero_wasm_sandbox::WasmSandbox::new();
        let module = sandbox
            .compile(wasm_bytes)
            .map_err(|e| WebViewError::Script(format!("WASM compile error: {e}")))?;

        let mut instance = module
            .instantiate(&sandbox)
            .map_err(|e| WebViewError::Script(format!("WASM instantiate error: {e}")))?;

        let results = instance
            .call(function_name, args)
            .map_err(|e| WebViewError::Script(format!("WASM call error: {e}")))?;

        if results.is_empty() {
            Ok("void".to_string())
        } else {
            Ok(results.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
        }
    }

    // ── Web Worker 管理 ──

    /// 创建 Dedicated Web Worker。
    ///
    /// Worker 在独立线程中运行自己的 V8 持久上下文。
    /// 通过 [`post_message_to_worker`](Self::post_message_to_worker) 发送消息，
    /// 通过 [`poll_worker_events`](Self::poll_worker_events) 接收消息。
    ///
    /// # 参数
    ///
    /// - `script` — Worker 初始化时执行的脚本代码
    ///
    /// # 返回
    ///
    /// Worker ID（用于后续操作）。
    pub fn create_worker(&mut self, script: &str) -> Result<u64, WebViewError> {
        let config = SandboxConfig {
            heap_limit: 0,
            timeout_ms: 0,
            persistent_context: false,
        };
        let worker = WorkerRuntime::new(script, config)
            .map_err(|e| WebViewError::Script(format!("Failed to create worker: {e}")))?;
        let id = self.next_worker_id;
        self.next_worker_id += 1;
        self.workers.insert(id, worker);
        Ok(id)
    }

    /// 创建 Dedicated Web Worker（自定义配置）。
    ///
    /// 与 [`create_worker`](Self::create_worker) 相同，但允许指定堆限制等配置。
    pub fn create_worker_with_config(&mut self, script: &str, config: SandboxConfig) -> Result<u64, WebViewError> {
        let worker = WorkerRuntime::new(script, config)
            .map_err(|e| WebViewError::Script(format!("Failed to create worker: {e}")))?;
        let id = self.next_worker_id;
        self.next_worker_id += 1;
        self.workers.insert(id, worker);
        Ok(id)
    }

    /// 向 Worker 发送消息。
    ///
    /// 消息以 JSON 字符串形式传递，Worker 端通过 `onmessage` 回调接收。
    pub fn post_message_to_worker(&mut self, worker_id: u64, message: &str) -> Result<(), WebViewError> {
        let worker = self
            .workers
            .get_mut(&worker_id)
            .ok_or_else(|| WebViewError::Script(format!("Worker {worker_id} not found")))?;
        worker
            .post_message(message)
            .map_err(|e| WebViewError::Script(format!("Failed to post message to worker {worker_id}: {e}")))
    }

    /// 向 Worker 发送额外脚本执行请求。
    pub fn execute_worker_script(&mut self, worker_id: u64, code: &str) -> Result<(), WebViewError> {
        let worker = self
            .workers
            .get_mut(&worker_id)
            .ok_or_else(|| WebViewError::Script(format!("Worker {worker_id} not found")))?;
        worker
            .execute_script(code)
            .map_err(|e| WebViewError::Script(format!("Failed to execute script on worker {worker_id}: {e}")))
    }

    /// 非阻塞地轮询 Worker 发出的事件。
    ///
    /// 返回 `(worker_id, event)` 对的列表。调用后内部缓冲被清空。
    pub fn poll_worker_events(&mut self) -> Vec<(u64, WorkerEvent)> {
        let mut events = Vec::new();
        let ids: Vec<u64> = self.workers.keys().copied().collect();
        for id in ids {
            if let Some(worker) = self.workers.get_mut(&id) {
                while let Some(event) = worker.try_recv() {
                    events.push((id, event));
                }
            }
        }
        events
    }

    /// 终止 Worker。
    ///
    /// Worker 线程会被强制停止，已终止的 Worker 不能再使用。
    pub fn terminate_worker(&mut self, worker_id: u64) -> bool {
        if let Some(mut worker) = self.workers.remove(&worker_id) {
            worker.terminate();
            true
        } else {
            false
        }
    }

    /// 获取 Worker 数量。
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// 检查 Worker 是否存在且仍在运行。
    pub fn is_worker_running(&self, worker_id: u64) -> bool {
        self.workers.get(&worker_id).is_some_and(|w| w.is_running())
    }

    /// 终止所有 Worker。
    pub fn terminate_all_workers(&mut self) {
        let ids: Vec<u64> = self.workers.keys().copied().collect();
        for id in ids {
            if let Some(mut worker) = self.workers.remove(&id) {
                worker.terminate();
            }
        }
    }
}
