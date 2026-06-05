//! WebView 主类型 — 可嵌入的网页渲染表面。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zero_engine::{PipelineTimings, PrefersColorSchemeValue, RenderPipeline};
use zero_net::{HttpCache, HttpClient, NetError};
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
    /// WASM 实例缓存 — JS 端 WebAssembly.instantiate() 自动桥接到 wasm-sandbox。
    wasm_instances: HashMap<u64, zero_wasm_sandbox::WasmInstance>,
    /// HTTP 响应缓存。
    http_cache: HttpCache,
    /// 用户颜色方案偏好。
    prefers_color_scheme: PrefersColorSchemeValue,
}

impl WebView {
    /// 创建新的 WebView。
    pub fn new(config: WebViewConfig) -> Self {
        let pipeline = RenderPipeline::new(config.width as f32, config.height as f32);
        let http_client = HttpClient::new();
        // 启用持久化上下文：WebAssembly 桥接和 DOM polyfill 需要跨 execute_script 保持状态
        let js_config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let js_sandbox =
            zero_script_sandbox::V8Sandbox::with_config(js_config).expect("V8 sandbox initialization should succeed");
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
            wasm_instances: HashMap::new(),
            http_cache: HttpCache::new(),
            prefers_color_scheme: PrefersColorSchemeValue::Light,
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
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
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

        // 检查 HTTP 缓存
        if let Some(cached) = self.http_cache.get(url) {
            tracing::info!("HTTP cache hit for {url}");
            let html = String::from_utf8(cached.body).map_err(|e| {
                self.loading = false;
                self.emit_event(&WebViewEvent::LoadFailed(
                    url.to_string(),
                    format!("Cached response body is not valid UTF-8: {e}"),
                ));
                WebViewError::Navigation(format!("Cached response body is not valid UTF-8: {e}"))
            })?;
            let render_result = self.load_html(&html, None);
            self.loading = false;
            self.emit_event(&WebViewEvent::LoadEnd(url.to_string()));
            return Ok(render_result);
        }

        // 发起 HTTP 请求
        match self.http_client.get(url) {
            Ok(response) => {
                // 尝试将响应存入 HTTP 缓存
                let _ = self.http_cache.put(url, &response);

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
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
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
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
    }

    /// 设置用户颜色方案偏好（影响 `prefers-color-scheme` 媒体查询）。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.prefers_color_scheme = scheme;
        self.pipeline.set_prefers_color_scheme(scheme);
    }

    /// 命中测试链接，坐标为 WebView 视口内的 CSS 逻辑像素。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        self.pipeline.hit_test_link(x, y)
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
    /// 同时自动桥接 `WebAssembly.instantiate()` 到 wasm-sandbox。
    ///
    /// # 错误
    ///
    /// 与 [`execute_script`](Self::execute_script) 相同。
    pub fn execute_script_with_dom(&mut self, script: &str) -> Result<String, WebViewError> {
        tracing::debug!("execute_script_with_dom called: {} bytes", script.len());

        let polyfill = zero_engine::generate_dom_api_polyfill();
        let full_script = format!("{polyfill}\n{script}");

        let result = self.execute_script(&full_script)?;

        // 检查是否有 WASM 桥接请求
        let bridge_result = self.process_wasm_bridge(&result)?;
        Ok(bridge_result)
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

    /// 处理 JS 端 WebAssembly.instantiate() 的桥接请求。
    ///
    /// 当 JS polyfill 检测到 WebAssembly.instantiate() 调用时，
    /// 输出 `__WASM_BRIDGE__:` 前缀的 JSON 命令。
    /// 此方法解析命令，通过 wasm-sandbox 编译执行，并将结果注入回 JS 环境。
    fn process_wasm_bridge(&mut self, script_output: &str) -> Result<String, WebViewError> {
        // 探测 JS 端是否有挂起的 WASM 桥接请求
        let probe_script = r#"
            (function() {
                if (typeof WebAssembly !== 'undefined' && WebAssembly._pendingBridge) {
                    var bridge = WebAssembly._pendingBridge;
                    WebAssembly._pendingBridge = null;
                    return bridge;
                }
                return '';
            })()
        "#;

        let probe_result = self.execute_script(probe_script).unwrap_or_default();

        if !probe_result.starts_with("__WASM_BRIDGE__:") {
            // 无 WASM 桥接请求，返回原始输出
            return Ok(script_output.to_string());
        }

        let json_str = &probe_result["__WASM_BRIDGE__:".len()..];
        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("WASM bridge: invalid JSON from polyfill: {e}");
                return Ok(script_output.to_string());
            }
        };

        let instance_id = parsed["id"].as_u64().unwrap_or(0);
        let b64_bytes = match parsed["bytes"].as_str() {
            Some(b) => b,
            None => {
                tracing::warn!("WASM bridge: missing bytes field");
                return Ok(script_output.to_string());
            }
        };

        // 解码 base64
        let wasm_bytes = match base64_decode(b64_bytes) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("WASM bridge: base64 decode error: {e}");
                return Ok(script_output.to_string());
            }
        };

        tracing::debug!(
            "WASM bridge: compiling {} bytes, instance_id={}",
            wasm_bytes.len(),
            instance_id
        );

        // 通过 wasm-sandbox 编译和实例化
        let sandbox = zero_wasm_sandbox::WasmSandbox::new();
        let module = match sandbox.compile(&wasm_bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WASM bridge: compile error: {e}");
                return Ok(script_output.to_string());
            }
        };

        let export_names = module.exports();
        let exports_json = serde_json::to_string(&export_names).unwrap_or_else(|_| "[]".to_string());

        let instance = match module.instantiate(&sandbox) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("WASM bridge: instantiate error: {e}");
                return Ok(script_output.to_string());
            }
        };

        // 缓存 WASM 实例
        self.wasm_instances.insert(instance_id, instance);

        // 注入结果回 JS 环境：设置 __wasm_results__
        let inject_script = format!(
            r#"
            if (!globalThis.__wasm_results__) globalThis.__wasm_results__ = {{}};
            globalThis.__wasm_results__[{instance_id}] = {{
                _id: {instance_id},
                _hostBacked: true,
                exports: {{
                  memory: {{ buffer: new ArrayBuffer(65536), grow: function() {{ return 1; }}, byteLength: 65536 }},
                  __wasm_export_names__: {exports_json}
                }}
            }};
            "#,
        );
        let _ = self.execute_script(&inject_script);

        Ok(script_output.to_string())
    }

    /// 调用已实例化的 WASM 模块的导出函数。
    ///
    /// 配合 `execute_script_with_dom` 的自动桥接使用：
    /// JS 调用 WebAssembly.instantiate() 后，WASM 模块被缓存，
    /// 通过此方法调用其导出函数。
    ///
    /// # 参数
    /// - `instance_id`: JS 端 WebAssembly._instances 中的实例 ID
    /// - `function_name`: 导出函数名
    /// - `args`: 传递给函数的参数
    pub fn call_wasm_export(
        &mut self,
        instance_id: u64,
        function_name: &str,
        args: &[zero_wasm_sandbox::WasmValue],
    ) -> Result<String, WebViewError> {
        let instance = self
            .wasm_instances
            .get_mut(&instance_id)
            .ok_or_else(|| WebViewError::Script(format!("WASM instance {instance_id} not found")))?;

        let results = instance
            .call(function_name, args)
            .map_err(|e| WebViewError::Script(format!("WASM call error: {e}")))?;

        if results.is_empty() {
            Ok("void".to_string())
        } else {
            Ok(results.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
        }
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

    /// 清空 HTTP 响应缓存。
    pub fn clear_http_cache(&mut self) {
        self.http_cache.clear();
    }

    /// 返回 HTTP 缓存条目数。
    pub fn http_cache_len(&self) -> usize {
        self.http_cache.len()
    }

    /// 返回 HTTP 缓存总字节数。
    pub fn http_cache_bytes(&self) -> usize {
        self.http_cache.total_bytes()
    }
}

/// 将 base64 字符串解码为字节。
///
/// WASM 桥接使用 base64 在 JS 和 Rust 之间传递 WASM 字节码。
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // 查找表
    let mut lookup = [0u8; 256];
    for (i, &b) in B64.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }

    let input_bytes = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);

    let mut i = 0;
    while i + 4 <= input_bytes.len() {
        let a = lookup[input_bytes[i] as usize] as u32;
        let b = lookup[input_bytes[i + 1] as usize] as u32;
        let c = if input_bytes[i + 2] == b'=' {
            0
        } else {
            lookup[input_bytes[i + 2] as usize] as u32
        };
        let d = if input_bytes[i + 3] == b'=' {
            0
        } else {
            lookup[input_bytes[i + 3] as usize] as u32
        };

        result.push(((a << 2) | (b >> 4)) as u8);
        if input_bytes[i + 2] != b'=' {
            result.push((((b & 0xF) << 4) | (c >> 2)) as u8);
        }
        if input_bytes[i + 3] != b'=' {
            result.push((((c & 0x3) << 6) | d) as u8);
        }

        i += 4;
    }

    Ok(result)
}
