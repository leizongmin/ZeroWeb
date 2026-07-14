//! WebView 主类型 — 可嵌入的网页渲染表面。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, PipelineTimings, PrefersColorSchemeValue, RenderPipeline, RenderResult,
    extract_img_srcs, extract_stylesheet_hrefs, image_resource_key, resolve_document_url,
};
use zero_net::{CacheLookup, HttpCache, HttpClient, NetError, is_file_url};
use zero_render_foundation::image_cache::{ImageCache, ImageKey, decode_image_bytes};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_script_sandbox::{SandboxConfig, WorkerEvent, WorkerRuntime};
use zero_security::{ResourceCheckResult, SecurityContext};
use zero_storage::{CacheRequest, FetchInterceptResult, ServiceWorkerRegistry};
use zero_wasm_sandbox::WasmInstance;

use crate::WebViewError;

/// 外部 JS 执行器类型（浏览器 Tab JS 线程注入；为 None 时使用进程内 V8）。
pub type ExternalScriptExecutor = std::sync::Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// WebView 配置。
#[derive(Clone)]
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
    /// 外部 JS 执行器（浏览器 Tab JS 线程注入；为 None 时使用进程内 V8）。
    #[doc(hidden)]
    pub external_script: Option<ExternalScriptExecutor>,
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
            external_script: None,
        }
    }
}

impl std::fmt::Debug for WebViewConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebViewConfig")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("transparent", &self.transparent)
            .field("user_agent", &self.user_agent)
            .field("url", &self.url)
            .field("devtools", &self.devtools)
            .field("external_script", &self.external_script.is_some())
            .finish()
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
    /// 进程内 JavaScript 沙箱（`external_script` 为 None 时使用）。
    js_sandbox: Option<Box<dyn zero_script_sandbox::Sandbox>>,
    /// 外部 JS 执行器（专用 JS 线程）。
    external_script: Option<ExternalScriptExecutor>,
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
    event_callbacks: Vec<Option<EventCallback>>,
    /// Service Worker 注册表。
    sw_registry: ServiceWorkerRegistry,
    /// Web Worker 实例（Dedicated Worker）。
    workers: HashMap<u64, WorkerRuntime>,
    /// Worker ID 生成器。
    next_worker_id: u64,
    /// WASM 实例缓存 — JS 端 WebAssembly.instantiate() 自动桥接到 wasm-sandbox。
    wasm_instances: HashMap<u64, WasmInstance>,
    /// HTTP 响应缓存。
    http_cache: HttpCache,
    /// 已解码图片子资源缓存（`<img src>` 等）。
    ///
    /// 由 `fetch_url` 在导航时抓取 + 解码填充（见 `fetch_image_subresources`），
    /// 供下游渲染器（browser 的 render_cpu/render_gpu）绘制时消费——`<img>` 才能
    /// 显示真实像素而非占位（goal doc DC-13 P1「图片子资源 / ImageCache 未贯通」）。
    image_cache: ImageCache,
    /// 已抓取图片的固有尺寸（url hash → (w,h)），resize/render 时回填 pipeline。
    cached_image_sizes: HashMap<u64, (f32, f32)>,
    /// ratio-only 图片信号（url hash → width/height 比，CSS §10.3.2 仅 SVG 出现）。
    /// %-dim / viewBox-only SVG 无确定固有尺寸、仅有 viewBox 宽高比，布局仅设 aspect_ratio。
    cached_image_ratios: HashMap<u64, f32>,
    /// no-ratio 图片信号（url hash → (真实固有宽, 真实固有高)，各 Option，CSS §10.3.2 仅 SVG）。
    /// width/height 非双绝对且无 viewBox 的 SVG 既无确定固有尺寸也无固有宽高比，布局不设
    /// aspect_ratio，缺失维按 default object size（宽 300 / 高 150）回退。
    cached_image_no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>,
    /// CSS font-family → font_id，供 paint 解析 font-weight 粗体 face。
    font_resolver: std::collections::HashMap<String, u32>,
    /// 用户颜色方案偏好。
    prefers_color_scheme: PrefersColorSchemeValue,
    /// 安全上下文（HSTS + 混合内容 + CSP）。
    security_context: SecurityContext,
}

impl WebView {
    /// 创建新的 WebView。
    pub fn new(config: WebViewConfig) -> Self {
        let pipeline = RenderPipeline::new(config.width as f32, config.height as f32);
        let http_client = HttpClient::new();
        let external_script = config.external_script.clone();
        let js_sandbox = if external_script.is_some() {
            None
        } else {
            let js_config = zero_script_sandbox::SandboxConfig {
                persistent_context: true,
                ..Default::default()
            };
            #[cfg(feature = "v8")]
            let sandbox: Box<dyn zero_script_sandbox::Sandbox> = Box::new(
                zero_script_sandbox::V8Sandbox::with_config(js_config)
                    .expect("V8 sandbox initialization should succeed"),
            );
            #[cfg(feature = "quickjs")]
            let sandbox: Box<dyn zero_script_sandbox::Sandbox> = Box::new(
                zero_script_sandbox::QuickJSSandbox::with_config(js_config)
                    .expect("QuickJS sandbox initialization should succeed"),
            );
            Some(sandbox)
        };
        Self {
            config,
            pipeline,
            http_client,
            js_sandbox,
            external_script,
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
            http_cache: HttpCache::open_persistent(),
            image_cache: ImageCache::default(),
            cached_image_sizes: HashMap::new(),
            cached_image_ratios: HashMap::new(),
            cached_image_no_ratio: HashMap::new(),
            font_resolver: HashMap::new(),
            prefers_color_scheme: PrefersColorSchemeValue::Light,
            security_context: SecurityContext::new(),
        }
    }

    /// 注册事件回调。
    ///
    /// 回调在 load_html / load_url / fetch_url 等操作触发状态变更时调用。
    /// 返回回调的索引，可用于后续移除。
    pub fn on_event(&mut self, callback: impl FnMut(&WebViewEvent) + 'static) -> usize {
        // CSS-07: 使用 Option 槽位，避免 remove 后索引偏移
        let callback = Rc::new(RefCell::new(callback)) as Rc<RefCell<dyn FnMut(&WebViewEvent)>>;
        // 尝试复用已移除的空槽位
        if let Some(empty_slot) = self.event_callbacks.iter().position(|s| s.is_none()) {
            self.event_callbacks[empty_slot] = Some(callback);
            return empty_slot;
        }
        let idx = self.event_callbacks.len();
        self.event_callbacks.push(Some(callback));
        idx
    }

    /// 移除事件回调。
    ///
    /// 传入 `on_event` 返回的索引。返回 `true` 表示成功移除。
    pub fn remove_event_callback(&mut self, index: usize) -> bool {
        if index < self.event_callbacks.len() && self.event_callbacks[index].is_some() {
            self.event_callbacks[index] = None;
            true
        } else {
            false
        }
    }

    /// 内部：分发事件到所有已注册的回调。
    fn emit_event(&self, event: &WebViewEvent) {
        for slot in self.event_callbacks.iter().flatten() {
            let mut cb = slot.borrow_mut();
            cb(event);
        }
    }

    /// 抓取外链 CSS 与 `<img>` 子资源，并设置 pipeline 文档 URL。
    fn prepare_page_subresources(&mut self, html: &str, page_url: &str) -> String {
        self.pipeline.set_document_url(Some(page_url));
        let external_css = self.resolve_external_css(html, page_url);
        let (image_sizes, image_ratios, image_no_ratio) = self.fetch_image_subresources(html, page_url);
        self.cached_image_sizes = image_sizes.clone();
        self.cached_image_ratios = image_ratios.clone();
        self.cached_image_no_ratio = image_no_ratio.clone();
        self.pipeline.set_image_sizes(image_sizes);
        self.pipeline.set_image_ratios(image_ratios);
        self.pipeline.set_image_no_ratio(image_no_ratio);
        external_css
    }

    /// resize / render 前把文档 URL 与图片固有尺寸同步回 pipeline。
    fn sync_pipeline_page_state(&mut self) {
        self.pipeline.set_document_url(self.current_url.as_deref());
        if !self.cached_image_sizes.is_empty() {
            self.pipeline.set_image_sizes(self.cached_image_sizes.clone());
        }
        if !self.cached_image_ratios.is_empty() {
            self.pipeline.set_image_ratios(self.cached_image_ratios.clone());
        }
        if !self.cached_image_no_ratio.is_empty() {
            self.pipeline.set_image_no_ratio(self.cached_image_no_ratio.clone());
        }
    }

    /// 加载 HTML 内容。
    pub fn load_html(&mut self, html: &str, css: Option<&str>) -> WebViewRenderResult {
        self.cached_html = html.to_string();
        let css_str = css.unwrap_or("");
        self.cached_css = css_str.to_string();
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        let result = self.pipeline.render_html(html, css_str);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 脚本修改 DOM 后重新加载 HTML（保留已缓存 CSS，并刷新图片子资源）。
    pub fn reload_html_after_script(&mut self, html: &str) -> WebViewRenderResult {
        self.cached_html = html.to_string();
        if let Some(page_url) = self.current_url.clone() {
            let (image_sizes, image_ratios, image_no_ratio) = self.fetch_image_subresources(html, &page_url);
            self.cached_image_sizes = image_sizes.clone();
            self.cached_image_ratios = image_ratios.clone();
            self.cached_image_no_ratio = image_no_ratio.clone();
            self.pipeline.set_image_sizes(image_sizes);
            self.pipeline.set_image_ratios(image_ratios);
            self.pipeline.set_image_no_ratio(image_no_ratio);
        }
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        let result = self.pipeline.render_html(html, &self.cached_css);
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
    /// 抓取 HTML 中所有外链 `<link rel="stylesheet">` 引用的 CSS 并合并。
    ///
    /// goal doc P1 缺口「外部样式表加载缺失」修复：URL 导航路径下，按 base URL
    /// 解析每个外链 href，逐个抓取，合并为单个 CSS 字符串（随后随 `load_html`
    /// 注入级联）。href 解析与抓取由 webview 层（持有 base URL 与 http client）
    /// 负责，DOM 内 link 提取由 `zero_engine::extract_stylesheet_hrefs` 负责，
    /// 保持 engine 不直接耦合网络。任一链接抓取失败仅记录日志、不阻断页面加载
    ///（与浏览器宽松行为一致）。
    fn resolve_external_css(&self, html: &str, base_url: &str) -> String {
        let hrefs = extract_stylesheet_hrefs(html);
        if hrefs.is_empty() {
            return String::new();
        }
        let base = url::Url::parse(base_url).ok();
        let mut combined = String::new();
        for href in &hrefs {
            let abs = match base.as_ref().and_then(|b| b.join(href).ok()) {
                Some(u) => u.to_string(),
                None => href.clone(),
            };
            match self.http_client.get(&abs) {
                Ok(resp) => match resp.text() {
                    Ok(css) => {
                        combined.push_str(&css);
                        combined.push('\n');
                    }
                    Err(e) => tracing::warn!("external stylesheet {abs} decode failed: {e}"),
                },
                Err(e) => tracing::warn!("external stylesheet {abs} fetch failed: {e}"),
            }
        }
        combined
    }

    /// 抓取并解码 HTML 中所有 `<img src>` 引用的图片子资源。
    ///
    /// goal doc P1 缺口「图片子资源 / ImageCache 未贯通」修复：按 base URL 解析每个
    /// `<img src>`，逐个 HTTP 抓取，解码为 `ImageData`（按魔数字节分发 PNG/JPEG；
    /// SVG 栅格化同模式后续），写入 `self.image_cache`（键 = `simple_hash(abs_url)`，
    /// 与 pipeline 的 image_sizes
    /// 及渲染器查找一致），并返回 `image_sizes`（url hash → (w,h)）供 pipeline 对无
    /// width/height 属性的 `<img>` 注入固有尺寸（DC-11 替换元素固有尺寸）。
    /// `data:` URI 暂不支持（跳过）；抓取/解码失败仅 warn 不阻断（宽松降级）。
    #[allow(clippy::type_complexity)]
    fn fetch_image_subresources(
        &mut self,
        html: &str,
        base_url: &str,
    ) -> (
        HashMap<u64, (f32, f32)>,
        HashMap<u64, f32>,
        HashMap<u64, (Option<f32>, Option<f32>)>,
    ) {
        let srcs = extract_img_srcs(html);
        let mut image_sizes = HashMap::new();
        let mut image_ratios = HashMap::new();
        let mut image_no_ratio = HashMap::new();
        if srcs.is_empty() {
            return (image_sizes, image_ratios, image_no_ratio);
        }
        let base = url::Url::parse(base_url).ok();
        for src in &srcs {
            // data: URI 暂不支持解码（后续可扩展）。
            if src.starts_with("data:") {
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(src).ok()) {
                Some(u) => u.to_string(),
                None => src.clone(),
            };
            let bytes = match self.http_client.get(&abs) {
                Ok(resp) => resp.body,
                Err(e) => {
                    tracing::warn!("image {abs} fetch failed: {e}");
                    continue;
                }
            };
            let img = match decode_image_bytes(&bytes) {
                Ok(img) => img,
                Err(e) => {
                    tracing::warn!("image {abs} decode failed (PNG/JPEG): {e}");
                    continue;
                }
            };
            let key_hash = image_resource_key(&abs, None);
            let key = ImageKey::new(key_hash);
            // R717：ratio-only SVG（%-dim / viewBox-only）无确定固有尺寸，仅有 viewBox 比——
            // 进 image_ratios，**不**进 image_sizes（任何确定 size 都会被 taffy 当作固有高度，
            // 阻止 flex ratio-derivation）。no-ratio SVG（CSS §10.3.2，width/height 非双绝对且
            // 无 viewBox）进 image_no_ratio（真实固有维，布局不设 aspect_ratio）——亦保留在
            // image_sizes 供背景图 background-size:auto 读 pixmap 尺寸。其余图像走 image_sizes。
            if let Some(ratio) = img.intrinsic_ratio() {
                image_ratios.insert(key_hash, ratio);
            } else {
                // R1438：一维 abs + 另一维缺失 + viewBox 的 SVG，usvg pixmap 对缺失维用原始
                // viewBox 值（bogus），须用计算的 computed_intrinsic 覆盖 pixmap 用于 image_sizes。
                let (w, h) = img
                    .computed_intrinsic()
                    .unwrap_or((img.width as f32, img.height as f32));
                image_sizes.insert(key_hash, (w, h));
                if let Some(dims) = img.no_ratio_intrinsic() {
                    image_no_ratio.insert(key_hash, dims);
                }
            }
            self.image_cache.insert_with_key(key, img);
        }
        (image_sizes, image_ratios, image_no_ratio)
    }

    /// 获取已解码图片子资源缓存的可变引用，供下游渲染器绘制时消费。
    ///
    /// browser 的 render_cpu / render_gpu 应在绘制帧时传入 `Some(&mut webview.image_cache())`
    /// 而非 `None`，使 `<img>` 渲染真实像素。
    pub fn image_cache(&mut self) -> &mut ImageCache {
        &mut self.image_cache
    }

    /// 图片缓存只读引用（快照用）。
    pub fn image_cache_ref(&self) -> &ImageCache {
        &self.image_cache
    }

    /// 复制图片缓存供 UI 线程快照。
    pub fn snapshot_image_cache(&self) -> ImageCache {
        self.image_cache.duplicate_for_snapshot()
    }

    /// 加载 URL（同步 HTTP GET）。
    ///
    /// 通过 zero-net 发起 HTTP 请求，获取 HTML 并渲染。
    /// 整个过程是同步阻塞的。
    /// 如果请求失败，加载状态会被重置，并返回错误。
    ///
    /// 子资源加载（goal doc DC-13）：
    /// - 外链样式表：抓取 `<link rel="stylesheet">` 引用的 CSS（按 base URL 解析、
    ///   逐个 HTTP 抓取、合并），随页面 HTML 一并注入级联（见 `resolve_external_css`）。
    /// - 图片子资源：抓取并解码 `<img src>` 引用的图片（见 `fetch_image_subresources`），
    ///   填充 `image_cache` 供下游渲染器绘制，并设 `image_sizes` 供 `<img>` 固有尺寸布局。
    ///
    /// 任一子资源抓取/解码失败仅记录日志、不阻断页面加载（宽松降级）。
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

        // 更新安全上下文的页面源
        self.security_context.set_page_origin(url);

        // 安全检查：HSTS 升级 + 混合内容阻止
        let effective_url = match self.security_context.check_resource_url(url, "document") {
            ResourceCheckResult::Allow => url.to_string(),
            ResourceCheckResult::Upgraded(https_url) => {
                tracing::info!("Security upgrade: {url} → {https_url}");
                self.current_url = Some(https_url.clone());
                https_url
            }
            ResourceCheckResult::Blocked(reason) => {
                self.loading = false;
                self.emit_event(&WebViewEvent::LoadFailed(url.to_string(), reason.clone()));
                return Err(WebViewError::Navigation(reason));
            }
        };

        // 尝试 Service Worker 拦截
        if let Some(origin) = Self::extract_origin(&effective_url) {
            let request = CacheRequest::new(&effective_url);
            match self.sw_registry.intercept_fetch(&request, &origin) {
                FetchInterceptResult::Cached(response) | FetchInterceptResult::Responded(response) => {
                    tracing::info!("Service Worker intercepted fetch for {effective_url}");
                    let html = String::from_utf8(response.body).map_err(|e| {
                        self.loading = false;
                        self.emit_event(&WebViewEvent::LoadFailed(
                            effective_url.to_string(),
                            format!("SW response body is not valid UTF-8: {e}"),
                        ));
                        WebViewError::Navigation(format!("SW response body is not valid UTF-8: {e}"))
                    })?;
                    let external_css = self.prepare_page_subresources(&html, &effective_url);
                    let render_result = self.load_html(&html, Some(&external_css));
                    self.loading = false;
                    self.emit_event(&WebViewEvent::LoadEnd(effective_url.to_string()));
                    return Ok(render_result);
                }
                _ => {
                    // PassThrough / NoWorker / Error — 继续正常网络请求
                }
            }
        }

        // 检查 HTTP 缓存（本地 file: 页面不缓存，避免磁盘变更后读到旧内容）
        if !is_file_url(&effective_url) {
            match self.http_cache.lookup(&effective_url, &[]) {
                CacheLookup::Hit(cached) => {
                    tracing::info!("HTTP cache hit for {effective_url}");
                    let html = String::from_utf8(cached.body).map_err(|e| {
                        self.loading = false;
                        self.emit_event(&WebViewEvent::LoadFailed(
                            effective_url.to_string(),
                            format!("Cached response body is not valid UTF-8: {e}"),
                        ));
                        WebViewError::Navigation(format!("Cached response body is not valid UTF-8: {e}"))
                    })?;
                    let external_css = self.prepare_page_subresources(&html, &effective_url);
                    let render_result = self.load_html(&html, Some(&external_css));
                    self.loading = false;
                    self.emit_event(&WebViewEvent::LoadEnd(effective_url.to_string()));
                    return Ok(render_result);
                }
                CacheLookup::Revalidate {
                    conditional_headers, ..
                } => match self.http_client.get_with_headers(&effective_url, &conditional_headers) {
                    Ok(response) if response.status_code == 304 => {
                        if let Some(cached) = self.http_cache.not_modified(&effective_url, &[], &response) {
                            let html = String::from_utf8(cached.body)
                                .map_err(|e| WebViewError::Navigation(format!("Cached body invalid UTF-8: {e}")))?;
                            tracing::info!("HTTP 304 revalidated for {effective_url}");
                            let external_css = self.prepare_page_subresources(&html, &effective_url);
                            let render_result = self.load_html(&html, Some(&external_css));
                            self.loading = false;
                            self.emit_event(&WebViewEvent::LoadEnd(effective_url.to_string()));
                            return Ok(render_result);
                        }
                    }
                    Ok(response) if (200..300).contains(&response.status_code) => {
                        let _ = self.http_cache.put(&effective_url, &response);
                        let html = response.text().map_err(|e| WebViewError::Navigation(e.to_string()))?;
                        tracing::info!("Fetched {} bytes from {effective_url} (revalidate)", html.len());
                        let external_css = self.prepare_page_subresources(&html, &effective_url);
                        let render_result = self.load_html(&html, Some(&external_css));
                        self.loading = false;
                        self.emit_event(&WebViewEvent::LoadEnd(effective_url.to_string()));
                        return Ok(render_result);
                    }
                    Ok(_) | Err(_) => {}
                },
                CacheLookup::Miss => {}
            }
        }

        // 发起 HTTP 请求
        match self.http_client.get(&effective_url) {
            Ok(response) => {
                if !is_file_url(&effective_url) {
                    // 尝试将响应存入 HTTP 缓存
                    let _ = self.http_cache.put(&effective_url, &response);
                }

                let html = response.text().map_err(|e| {
                    self.loading = false;
                    self.emit_event(&WebViewEvent::LoadFailed(
                        effective_url.to_string(),
                        format!("Failed to decode response body: {e}"),
                    ));
                    WebViewError::Navigation(format!("Failed to decode response body: {e}"))
                })?;

                tracing::info!("Fetched {} bytes from {effective_url}", html.len());

                // 抓取外链样式表（`<link rel="stylesheet">`），合并后注入级联。
                let external_css = self.prepare_page_subresources(&html, &effective_url);

                // 渲染 HTML
                let render_result = self.load_html(&html, Some(&external_css));
                self.loading = false;
                self.emit_event(&WebViewEvent::LoadEnd(effective_url.to_string()));
                Ok(render_result)
            }
            Err(NetError::Timeout) => {
                self.loading = false;
                let msg = format!("Request to {effective_url} timed out");
                self.emit_event(&WebViewEvent::LoadFailed(effective_url.to_string(), msg.clone()));
                Err(WebViewError::Navigation(msg))
            }
            Err(NetError::Proxy(detail)) => {
                self.loading = false;
                let msg = format!("Proxy error fetching {effective_url}: {detail}");
                self.emit_event(&WebViewEvent::LoadFailed(effective_url.to_string(), msg.clone()));
                Err(WebViewError::Navigation(msg))
            }
            Err(e) => {
                self.loading = false;
                let msg = format!("Failed to fetch {effective_url}: {e}");
                self.emit_event(&WebViewEvent::LoadFailed(effective_url.to_string(), msg.clone()));
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

    /// 外部已获取 HTML 后完成加载（抓取 CSS/图片子资源并渲染）。
    ///
    /// 供浏览器异步 HTTP fetch 路径使用，行为与 [`Self::fetch_url`] 的子资源处理一致。
    pub fn complete_fetched_page(&mut self, html: &str, page_url: &str) -> WebViewRenderResult {
        self.current_url = Some(page_url.to_string());
        let external_css = self.prepare_page_subresources(html, page_url);
        let result = self.load_html(html, Some(&external_css));
        self.loading = false;
        self.emit_event(&WebViewEvent::LoadEnd(page_url.to_string()));
        result
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
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        let result = self.pipeline.render_html(&self.cached_html, &self.cached_css);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 在已有 DOM/布局缓存上增量重绘视口（resize 等场景）；无缓存时返回 `None`。
    pub fn render_incremental(&mut self) -> Option<WebViewRenderResult> {
        if self.cached_html.is_empty() {
            return None;
        }
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        let result = self.pipeline.repaint_cached_viewport(&self.cached_css)?;
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        Some(render_result)
    }

    /// 导航前设置文档 URL 与 pipeline 状态（供异步加载使用）。
    ///
    /// 必须丢弃上一文档的 `last_render` / 缓存，否则多进程增量 publish 会把旧帧 IPC 到浏览器。
    pub fn prepare_document_state(&mut self, page_url: &str) {
        self.last_render = None;
        self.cached_html.clear();
        self.cached_css.clear();
        self.cached_image_sizes.clear();
        self.cached_image_ratios.clear();
        self.cached_image_no_ratio.clear();
        self.image_cache.clear();
        self.current_url = Some(page_url.to_string());
        self.loading = true;
        self.pipeline.set_document_url(Some(page_url));
        self.security_context.set_page_origin(page_url);
        self.emit_event(&WebViewEvent::LoadStart(page_url.to_string()));
    }

    /// 推进预算渲染会话。
    pub fn advance_budget_session(&mut self, session: &mut BudgetedRenderSession, budget_ms: f64) -> BudgetAdvance {
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.advance_budgeted_render(session, budget_ms)
    }

    /// 应用预算渲染结果到 WebView 状态。
    pub fn apply_render_result(&mut self, result: RenderResult, page_url: &str, finished: bool) {
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result);
        if finished {
            self.loading = false;
            self.emit_event(&WebViewEvent::LoadEnd(page_url.to_string()));
        }
    }

    /// 设置缓存 HTML/CSS（异步管线渲染前调用）。
    pub fn set_cached_content(&mut self, html: &str, css: &str) {
        self.cached_html = html.to_string();
        self.cached_css = css.to_string();
    }

    /// 已缓存图片固有尺寸。
    pub fn cached_image_sizes(&self) -> &HashMap<u64, (f32, f32)> {
        &self.cached_image_sizes
    }

    /// 更新图片固有尺寸并同步到 pipeline。
    pub fn set_image_sizes(&mut self, sizes: HashMap<u64, (f32, f32)>) {
        self.cached_image_sizes = sizes.clone();
        self.pipeline.set_image_sizes(sizes);
    }

    /// 更新 ratio-only 图片信号并同步到 pipeline（CSS §10.3.2，仅 SVG 出现）。
    pub fn set_image_ratios(&mut self, ratios: HashMap<u64, f32>) {
        self.cached_image_ratios = ratios.clone();
        self.pipeline.set_image_ratios(ratios);
    }

    /// 获取已抓取图片的 ratio-only 信号快照（供增量加载合并）。
    pub fn cached_image_ratios(&self) -> &HashMap<u64, f32> {
        &self.cached_image_ratios
    }

    /// 更新 no-ratio 图片信号并同步到 pipeline（CSS §10.3.2，仅 no-ratio SVG 出现）。
    pub fn set_image_no_ratio(&mut self, no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>) {
        self.cached_image_no_ratio = no_ratio.clone();
        self.pipeline.set_image_no_ratio(no_ratio);
    }

    /// 获取已抓取图片的 no-ratio 信号快照（供增量加载合并）。
    pub fn cached_image_no_ratio(&self) -> &HashMap<u64, (Option<f32>, Option<f32>)> {
        &self.cached_image_no_ratio
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

    /// 当前缓存的 HTML 源码（用于提取 `<title>` 等）。
    pub fn html_content(&self) -> &str {
        &self.cached_html
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
        let doc_url = self.current_url.clone();
        let image_sizes = self.cached_image_sizes.clone();
        let image_ratios = self.cached_image_ratios.clone();
        let image_no_ratio = self.cached_image_no_ratio.clone();
        self.config.width = width;
        self.config.height = height;
        self.pipeline = RenderPipeline::new(width as f32, height as f32);
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.set_document_url(doc_url.as_deref());
        if !image_sizes.is_empty() {
            self.pipeline.set_image_sizes(image_sizes);
        }
        if !image_ratios.is_empty() {
            self.pipeline.set_image_ratios(image_ratios);
        }
        if !image_no_ratio.is_empty() {
            self.pipeline.set_image_no_ratio(image_no_ratio);
        }
        self.pipeline.set_font_resolver(self.font_resolver.clone());
    }

    /// 设置 CSS font-family 查找表（由宿主从 `FontLoader::build_font_resolver()` 构建）。
    pub fn set_font_resolver(&mut self, resolver: std::collections::HashMap<String, u32>) {
        self.font_resolver = resolver;
        self.pipeline.set_font_resolver(self.font_resolver.clone());
    }

    /// 设置用户颜色方案偏好（影响 `prefers-color-scheme` 媒体查询）。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.prefers_color_scheme = scheme;
        self.pipeline.set_prefers_color_scheme(scheme);
    }

    /// 命中测试链接，坐标为 WebView 视口内的 CSS 逻辑像素。
    ///
    /// 若存在当前页面 URL，会将相对 `href` 解析为绝对 URL 后再返回。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let href = self.pipeline.hit_test_link(x, y)?;
        Some(match self.current_url.as_deref() {
            Some(base) => resolve_document_url(base, &href),
            None => href,
        })
    }

    /// 命中测试图片，返回绝对化后的 `src`。
    pub fn hit_test_image(&self, x: f32, y: f32) -> Option<String> {
        let src = self.pipeline.hit_test_image(x, y)?;
        Some(match self.current_url.as_deref() {
            Some(base) => resolve_document_url(base, &src),
            None => src,
        })
    }

    /// 命中测试元素，坐标为 WebView 视口内的 CSS 逻辑像素。
    pub fn hit_test_element(&self, x: f32, y: f32) -> Option<zero_engine::ElementHit> {
        self.pipeline.hit_test_element(x, y)
    }

    /// 构建主线程命中测试快照（与最近一次渲染一致）。
    pub fn build_hit_test_cache(&self) -> Option<zero_engine::HitTestCache> {
        self.pipeline.build_hit_test_cache()
    }

    /// 抓取 URL 文本资源（用于外链脚本等）。
    pub fn fetch_text_at(&self, url: &str) -> Result<String, WebViewError> {
        let resp = self
            .http_client
            .get(url)
            .map_err(|e| WebViewError::Navigation(format!("fetch {url}: {e}")))?;
        resp.text()
            .map_err(|e| WebViewError::Navigation(format!("decode {url}: {e}")))
    }

    /// 文档布局高度（CSS 逻辑像素）。
    pub fn document_height(&self) -> Option<f32> {
        self.pipeline.document_height()
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

        if let Some(ext) = &self.external_script {
            return ext(script).map_err(WebViewError::Script);
        }

        match self.js_sandbox.as_mut().expect("js sandbox").execute(script) {
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
        if !self.cached_css.is_empty() {
            self.cached_css.push('\n');
        }
        self.cached_css.push_str(css);
        self.sync_pipeline_page_state();
        let html = if self.cached_html.is_empty() {
            "<html><body></body></html>"
        } else {
            &self.cached_html
        };
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

    /// 处理 JS 端 WebAssembly 桥接请求。
    ///
    /// 支持 `__WASM_BRIDGE__:`（实例化）和 `__WASM_COMPILE__:`（编译）两种桥接命令。
    /// 实例化时自动执行 `_start`/`_initialize` 导出函数，
    /// 读取 WASM 内存状态，并将可调用的导出函数注入回 JS 环境。
    fn process_wasm_bridge(&mut self, script_output: &str) -> Result<String, WebViewError> {
        // 先处理挂起的 WASM 导出调用队列
        self.process_wasm_calls()?;

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

        if probe_result.is_empty() {
            return Ok(script_output.to_string());
        }

        // 处理实例化桥接命令
        if let Some(json_str) = probe_result.strip_prefix("__WASM_BRIDGE__:") {
            return self.handle_wasm_instantiate_bridge(json_str, script_output);
        }

        // 处理编译桥接命令
        if let Some(json_str) = probe_result.strip_prefix("__WASM_COMPILE__:") {
            return self.handle_wasm_compile_bridge(json_str, script_output);
        }

        // 未知桥接前缀，忽略
        Ok(script_output.to_string())
    }

    /// 处理 WASM 实例化桥接命令。
    ///
    /// 编译 WASM 字节码，创建实例，自动执行 _start/_initialize，
    /// 读取内存状态，注入可调用的导出函数回 JS。
    fn handle_wasm_instantiate_bridge(&mut self, json_str: &str, script_output: &str) -> Result<String, WebViewError> {
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

        // 通过 wasm-sandbox 编译
        let sandbox = zero_wasm_sandbox::WasmSandbox::new();
        let module = match sandbox.compile(&wasm_bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WASM bridge: compile error: {e}");
                // SEC-08: 转义错误消息中的 JS 特殊字符，防止注入
                let err_msg = escape_js_string(&format!("{e}"));
                let err_script = format!(
                    "if (!globalThis.__wasm_errors__) globalThis.__wasm_errors__ = {{}}; \
                     globalThis.__wasm_errors__[{instance_id}] = 'compile: {err_msg}'"
                );
                let _ = self.execute_script(&err_script);
                return Ok(script_output.to_string());
            }
        };

        let export_names = module.exports();

        // 实例化
        let mut instance = match module.instantiate(&sandbox) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("WASM bridge: instantiate error: {e}");
                // SEC-08: 转义错误消息中的 JS 特殊字符，防止注入
                let err_msg = escape_js_string(&format!("{e}"));
                let err_script = format!(
                    "if (!globalThis.__wasm_errors__) globalThis.__wasm_errors__ = {{}}; \
                     globalThis.__wasm_errors__[{instance_id}] = 'instantiate: {err_msg}'"
                );
                let _ = self.execute_script(&err_script);
                return Ok(script_output.to_string());
            }
        };

        // 自动执行 WASM 初始化函数（_start 或 _initialize）
        if instance.has_func("_start") {
            if let Err(e) = instance.call("_start", &[]) {
                tracing::debug!("WASM bridge: _start error (may be expected): {e}");
            }
        } else if instance.has_func("_initialize") && instance.call("_initialize", &[]).is_err() {
            tracing::debug!("WASM bridge: _initialize error (may be expected)");
        }

        // 读取 WASM 内存状态（如果有 memory 导出）
        let memory_bytes = instance.read_memory("memory", 0, 256).unwrap_or_default();
        let memory_len = if export_names.iter().any(|n| n == "memory") {
            // WASM 内存以页（65536 字节）为单位
            let pages = 1.max(memory_bytes.len() / 65536 + 1);
            pages * 65536
        } else {
            65536
        };

        // 缓存 WASM 实例
        self.wasm_instances.insert(instance_id, instance);

        // 构建可调用的导出函数
        let exports_json = serde_json::to_string(&export_names).unwrap_or_else(|_| "[]".to_string());

        // 生成每个导出函数的 JS 可调用包装
        let mut export_fn_scripts = Vec::new();
        for export_name in &export_names {
            // 跳过特殊导出
            if export_name == "memory" || export_name == "_start" || export_name == "_initialize" {
                continue;
            }
            let name = export_name.as_str();
            let escaped_name = name.replace('\'', "\\'");
            export_fn_scripts.push(format!(
                r#"'{escaped_name}': function() {{
                    var callId = WebAssembly._nextCallId++;
                    var args = Array.prototype.slice.call(arguments);
                    var numArgs = args.map(function(a) {{
                        if (typeof a === 'number') return a|0;
                        return 0;
                    }});
                    WebAssembly._callQueue.push({{instanceId: {instance_id}, name: '{escaped_name}', args: numArgs, callId: callId}});
                    // 检查是否有缓存结果
                    if (WebAssembly._callResults[callId] !== undefined) {{
                        var r = WebAssembly._callResults[callId];
                        delete WebAssembly._callResults[callId];
                        return r;
                    }}
                    return 0;
                }}"#
            ));
        }
        let export_fns = export_fn_scripts.join(",\n");

        // 注入完整的实例对象到 JS 环境
        let inject_script = format!(
            r#"
            (function() {{
                if (!globalThis.__wasm_results__) globalThis.__wasm_results__ = {{}};
                var exports = {{
                    memory: {{
                        buffer: new ArrayBuffer({memory_len}),
                        grow: function(delta) {{ return Math.floor({memory_len} / 65536) + delta; }},
                        byteLength: {memory_len}
                    }},
                    __wasm_export_names__: {exports_json},
                    __host_backed__: true,
                    {export_fns}
                }};
                // 如果有内存数据，写入 buffer
                // （注意：JS 侧通过 DataView 写入 base64 解码后的字节）
                globalThis.__wasm_results__[{instance_id}] = {{
                    _id: {instance_id},
                    _hostBacked: true,
                    exports: exports
                }};
                // 同时更新 _instances 中的 stub
                if (typeof WebAssembly !== 'undefined' && WebAssembly._instances[{instance_id}]) {{
                    WebAssembly._instances[{instance_id}].stub = globalThis.__wasm_results__[{instance_id}];
                }}
            }})();
            "#,
        );
        let _ = self.execute_script(&inject_script);

        tracing::debug!(
            "WASM bridge: instance {} created, {} exports: {:?}",
            instance_id,
            export_names.len(),
            export_names
        );

        Ok(script_output.to_string())
    }

    /// 处理 WASM 编译桥接命令。
    ///
    /// 仅编译 WASM 字节码为模块（不实例化），缓存模块信息供后续 instantiate 使用。
    fn handle_wasm_compile_bridge(&mut self, json_str: &str, script_output: &str) -> Result<String, WebViewError> {
        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("WASM compile bridge: invalid JSON: {e}");
                return Ok(script_output.to_string());
            }
        };

        let module_id = parsed["id"].as_u64().unwrap_or(0);
        let b64_bytes = match parsed["bytes"].as_str() {
            Some(b) => b,
            None => return Ok(script_output.to_string()),
        };

        let wasm_bytes = match base64_decode(b64_bytes) {
            Ok(b) => b,
            Err(_) => return Ok(script_output.to_string()),
        };

        let sandbox = zero_wasm_sandbox::WasmSandbox::new();
        let module = match sandbox.compile(&wasm_bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WASM compile bridge: compile error: {e}");
                return Ok(script_output.to_string());
            }
        };

        let export_names = module.exports();
        let exports_json = serde_json::to_string(&export_names).unwrap_or_else(|_| "[]".to_string());

        // 不需要实例化，仅注入编译结果
        let inject_script = format!(
            r#"
            if (!globalThis.__wasm_compiled__) globalThis.__wasm_compiled__ = {{}};
            globalThis.__wasm_compiled__[{module_id}] = {{
                _id: {module_id},
                _bytes: globalThis.WebAssembly._modules[{module_id}],
                _compiled: true,
                exports: function() {{ return {exports_json}; }}
            }};
            "#,
        );
        let _ = self.execute_script(&inject_script);

        Ok(script_output.to_string())
    }

    /// 处理 JS 端挂起的 WASM 导出函数调用队列。
    ///
    /// JS 侧每次调用导出函数时，将参数存入 `WebAssembly._callQueue`。
    /// 此方法读取队列，通过 wasm-sandbox 执行调用，将结果注入回 JS。
    fn process_wasm_calls(&mut self) -> Result<(), WebViewError> {
        // 探测调用队列
        let probe_script = r#"
            (function() {
                if (typeof WebAssembly === 'undefined' || !WebAssembly._callQueue || WebAssembly._callQueue.length === 0) {
                    return '[]';
                }
                var queue = WebAssembly._callQueue.slice();
                WebAssembly._callQueue = [];
                return JSON.stringify(queue);
            })()
        "#;

        let queue_json = self.execute_script(probe_script).unwrap_or_else(|_| "[]".to_string());

        let calls: Vec<serde_json::Value> = match serde_json::from_str(&queue_json) {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };

        if calls.is_empty() {
            return Ok(());
        }

        tracing::debug!("WASM bridge: processing {} pending export calls", calls.len());

        // 执行每个调用并收集结果
        let mut results_script = String::from(
            "if (typeof WebAssembly !== 'undefined') WebAssembly._callResults = WebAssembly._callResults || {};\n",
        );

        for call in &calls {
            let instance_id = call["instanceId"].as_u64().unwrap_or(0);
            let name = call["name"].as_str().unwrap_or("");
            let call_id = call["callId"].as_u64().unwrap_or(0);
            let args_array = call["args"].as_array();

            // 构造 WASM 参数
            let wasm_args: Vec<zero_wasm_sandbox::WasmValue> = args_array
                .map(|arr| {
                    arr.iter()
                        .map(|v| zero_wasm_sandbox::WasmValue::I32(v.as_i64().unwrap_or(0) as i32))
                        .collect()
                })
                .unwrap_or_default();

            // 执行调用
            if let Some(instance) = self.wasm_instances.get_mut(&instance_id) {
                match instance.call(name, &wasm_args) {
                    Ok(results) => {
                        let result_val = if results.is_empty() {
                            "null".to_string()
                        } else {
                            // 取第一个返回值
                            results[0].to_string()
                        };
                        results_script.push_str(&format!("WebAssembly._callResults[{call_id}] = {result_val};\n"));
                    }
                    Err(e) => {
                        tracing::debug!("WASM call error for {name}: {e}");
                        results_script.push_str(&format!("WebAssembly._callResults[{call_id}] = null;\n"));
                    }
                }
            } else {
                results_script.push_str(&format!("WebAssembly._callResults[{call_id}] = null;\n"));
            }
        }

        // 注入结果回 JS
        let _ = self.execute_script(&results_script);

        Ok(())
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

    /// 获取安全上下文（只读）。
    ///
    /// 安全上下文包含 HSTS 存储和混合内容策略。
    pub fn security_context(&self) -> &SecurityContext {
        &self.security_context
    }

    /// 获取安全上下文（可变）。
    pub fn security_context_mut(&mut self) -> &mut SecurityContext {
        &mut self.security_context
    }

    /// 检查子资源 URL 是否可以安全加载。
    ///
    /// 执行 HSTS 升级 + 混合内容检测。用于页面内的 CSS/JS/图片等子资源加载。
    pub fn check_subresource_url(&mut self, url: &str, resource_type: &str) -> ResourceCheckResult {
        self.security_context.check_resource_url(url, resource_type)
    }
}

/// 将字节编码为 base64 字符串。
///
/// WASM 桥接使用 base64 在 JS 和 Rust 之间传递内存数据。
#[allow(dead_code)]
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let a = chunk[0];
        let b = if chunk.len() > 1 { chunk[1] } else { 0 };
        let c = if chunk.len() > 2 { chunk[2] } else { 0 };
        result.push(B64[(a >> 2) as usize] as char);
        result.push(B64[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            B64[(((b & 0xF) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            B64[(c & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    result
}

/// 将 base64 字符串解码为字节。
///
/// WASM 桥接使用 base64 在 JS 和 Rust 之间传递 WASM 字节码。
pub(crate) fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
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

/// 转义字符串中的 JavaScript 特殊字符，防止注入。
///
/// 替换 `'`、`\`、`</script>` 等字符为安全序列。
fn escape_js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '<' => {
                // 检查是否为 </script> 序列（不区分大小写）
                out.push('\\');
                out.push('x');
                out.push('3');
                out.push('c');
            }
            _ => out.push(ch),
        }
    }
    out
}
