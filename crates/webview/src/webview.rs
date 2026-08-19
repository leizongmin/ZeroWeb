//! WebView 主类型 — 可嵌入的网页渲染表面。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, DomMutation, MediaType, PipelineTimings, PrefersColorSchemeValue,
    RenderPipeline, RenderResult, extract_css_image_urls, extract_html_style_text, extract_img_srcs,
    extract_page_scripts_indexed, extract_stylesheet_hrefs, generate_js_dom_shim, image_resource_key,
    register_dom_callbacks, resolve_document_url, script_clear_current_script, script_dispatch_dom_event,
    script_set_current_script,
};
// R3150（闭合 R3121 latent）：script_dispatch_native_event 唯一用法（dispatch_event native_dom 分支）
// 受 `#[cfg(feature = "v8")]` 门控——quickjs feature 下 unused import。独立 gated import 消 latent warning。
#[cfg(feature = "v8")]
use zero_engine::script_dispatch_native_event;
use zero_net::{FetchPriority, HttpClient, HttpResponse, NetError, ResourceLoader, ResourceRequest, is_file_url};
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey, decode_data_uri};

use crate::image_decoder::decode_image;
use zero_page_runtime::{
    ServiceWorkerManager, ServiceWorkerManagerEvent, ServiceWorkerRegistrationKey, ServiceWorkerVersionSlots,
};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_script_sandbox::{SandboxConfig, WorkerEvent, WorkerRuntime};
use zero_security::{ResourceCheckResult, SecurityContext};
use zero_storage::{CacheRequest, FetchInterceptResult, ServiceWorkerRegistry};
use zero_wasm_sandbox::WasmInstance;

use crate::{IndexedDbOwner, WebViewError};

mod user_actions;
pub use user_actions::WebViewUserActionResult;

/// 外部 JS 执行器类型（浏览器 Tab JS 线程注入；为 None 时使用进程内 V8）。
pub type ExternalScriptExecutor = std::sync::Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// 外链脚本源获取器类型（进程内/headless 路径：fetch 外链 `<script src>` / `<script type=module src>` 源）。
/// 入参 `(page_url, script_src)`，返回脚本源（或错误 → 该脚本跳过）。为 None 时外链脚本跳过（离线语义，
/// 与 reftest 一致）。与 `external_script`（多进程执行委托，互斥）独立——本获取器仅进程内 sandbox 路径消费。
pub type ScriptSourceFetcher = std::sync::Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// R34xx：图片源获取器（headless/testharness 路径 fetch `<img src>` / CSS `url()` 图片；
/// None → 走 HTTP 网络）。返回图片字节；返回 None 表示该 URL 无法本地提供（回退网络）。
pub type ImageSourceFetcher = std::sync::Arc<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync>;

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
    /// HTTP 请求超时（秒），`None` 使用默认（30s）。
    ///
    /// 默认超时对真实网络合理，但依赖超时路径的测试（黑洞地址）会实等
    /// 30s；嵌入者可设短超时加快失败反馈。
    pub http_timeout_secs: Option<u64>,
    /// 外部 JS 执行器（浏览器 Tab JS 线程注入；为 None 时使用进程内 V8）。
    #[doc(hidden)]
    pub external_script: Option<ExternalScriptExecutor>,
    /// 外链脚本源获取器（进程内/headless 路径 fetch 外链脚本源；None → 外链脚本跳过，离线语义）。
    pub script_source_fetcher: Option<ScriptSourceFetcher>,
    /// R34xx：图片源获取器（headless/testharness 路径；None → HTTP 网络）。
    pub image_source_fetcher: Option<ImageSourceFetcher>,
    /// R34xx：fetch 处理器（headless/testharness 路径；None → 不注册 `__zw_fetch`，shim 落
    /// ok:false stub——浏览器路径由 app 层 `FetchBridge` 注册异步 `__zw_fetch`，二者互斥）。
    /// 经本配置注册的 `__zw_fetch` 走**同步返回**契约（回调直接返 wire；shim R34xx 支持）。
    pub fetch_handler: Option<zero_engine::fetch_bridge::FetchHandler>,
    /// P1b S2：启用原生 DOM 绑定（RFC `p1b-v8-native-bindings-rfc.md`）。
    ///
    /// 开启时，`run_page_scripts` 在 polyfill 桥之上额外安装原生 `nodeType`/`tagName` 等
    /// getter（`engine::dom_bindings`，经 `Sandbox::install_native_bindings` escape-hatch），
    /// 从 re-parsed `Document` 直读（不经 shim 字符串桥）。默认关 → 零回归。生产接线为
    /// read-only 快照（re-parse cached_html；mutation 同步为后续写入切片）。
    pub native_dom: bool,
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
            http_timeout_secs: None,
            external_script: None,
            script_source_fetcher: None,
            image_source_fetcher: None,
            fetch_handler: None,
            native_dom: false,
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
            .field("http_timeout_secs", &self.http_timeout_secs)
            .field("external_script", &self.external_script.is_some())
            .field("script_source_fetcher", &self.script_source_fetcher.is_some())
            .field("image_source_fetcher", &self.image_source_fetcher.is_some())
            .field("fetch_handler", &self.fetch_handler.is_some())
            .field("native_dom", &self.native_dom)
            .finish()
    }
}

/// WebView 渲染结果。
#[derive(Debug, Clone)]
pub struct WebViewRenderResult {
    /// 渲染图元。
    pub primitives: RenderPrimitives,
    /// 本帧脏区域（S3；空 = 全量光栅化）。
    pub dirty_rects: Vec<(f32, f32, f32, f32)>,
    /// 管线耗时。
    pub timings: PipelineTimings,
}

impl WebViewRenderResult {
    /// 本帧图元（`primitives` 字段的便捷访问）。
    ///
    /// 与 engine `RenderResult::primitives()` API 对称：调用方可对 engine 渲染结果与
    /// WebView 渲染结果统一以 `result.primitives()` 取图元，无需区分类型（字段仍保留供
    /// 需要按值取得 `RenderPrimitives` 的调用方使用）。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }
}

fn render_result_to_webview(result: &RenderResult) -> WebViewRenderResult {
    WebViewRenderResult {
        primitives: result.display_list.primitives.clone(),
        dirty_rects: result.display_list.dirty_rects.clone(),
        timings: result.timings.clone(),
    }
}

/// 外链样式表内 `url(...)` 相对引用 → 绝对（CSS 相对 url 基准 = 样式表 URL）。
/// 大小写不敏感扫描 `url(`（ASCII 小写化保持字节位）；引号内/裸值均可；
/// `data:` 与已含 scheme 的绝对 URL 不动。best-effort（转义函数名如 `U\r4c(` 不处理——
/// tokenizer 解码面由 extract_css_image_urls 负责，重写面覆盖 WPT 平文引用）。
fn absolutize_css_urls(css: &str, stylesheet_url: &str) -> String {
    let lower = css.to_ascii_lowercase();
    let base = url::Url::parse(stylesheet_url).ok();
    let mut out = String::with_capacity(css.len() + 64);
    let mut from = 0usize;
    while let Some(rel) = lower[from..].find("url(") {
        let start = from + rel;
        // url( 之后：可选空白 + 可选引号。
        let mut i = start + 4;
        let bytes = css.as_bytes();
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        let quote = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
            let q = bytes[i];
            i += 1;
            Some(q)
        } else {
            None
        };
        let val_start = i;
        while i < bytes.len() {
            let b = bytes[i];
            if let Some(q) = quote {
                if b == q {
                    break;
                }
            } else if b == b')' {
                break;
            }
            i += 1;
        }
        let value = css[val_start..i].trim();
        let replaced = if value.is_empty() || value.starts_with("data:") || value.contains("://") {
            value.to_string()
        } else {
            match base.as_ref().and_then(|b| b.join(value).ok()) {
                Some(u) => u.to_string(),
                None => value.to_string(),
            }
        };
        out.push_str(&css[from..val_start]);
        out.push_str(&replaced);
        if i >= bytes.len() {
            // 未闭合引号/值——原样保留剩余部分。
            out.push_str(&css[val_start..]);
            return out;
        }
        out.push_str(&css[i..i + 1]); // 闭合引号（若有）或 ')'——原样复制
        from = i + 1;
        if quote.is_some() {
            // 跳过闭合引号后的 ')'（可能夹空白，剩余部分由尾部复制兜底）
            from = i + 2;
        }
    }
    out.push_str(&css[from..]);
    out
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
    /// R3268 canvas 显示链路：CanvasRegistry（JS getContext 与 painter 共享）。
    canvas_registry: std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>>,
    /// js-dom M3 R100：持久 selector→handle 反查表（**倒置**的 handle→selector map）。
    ///
    /// 每次 mutation 应用后，把 `render_with_dom_mutations` 返回的 handle→唯一选择器
    /// 映射 merge 进来（key 取 selector）。JS 侧 `document.querySelector`/
    /// `getElementById` 命中后经 `__zw_handle_for_selector` 反查——命中则返回原
    /// handle proxy（`_proxyCache['@'+handle]` 同对象 identity），使跨 execute 的
    /// 元素引用（Vue `vm.$el` 持有的 `__n0` proxy vs 重新 query 的新 sel proxy）
    /// 身份统一，事件监听器（注册在 handle proxy 的 `_listenerStore` key 下）可达。
    /// 导航/load_html 换代时清空（旧页 handle 在新页无效，镜像 rect_bridge
    /// `HandleSelectorMap` 语义）。R77 验证的反查三件套形态，反查限定在 query
    /// 返回点（不做全局 `_wrapSelector` 前置——R77 教训 #7）。
    selector_handle_map: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
    /// js-dom M3 R100：**共享 append-only** mutations 队列（页面生命周期内单实例）。
    ///
    /// 所有 `register_dom_callbacks` 调用（run_page_scripts / execute_script_with_dom /
    /// dispatch_event）都传**同一 Arc**——读回调（`__zw_get_text_handle` 等）天然看到
    /// 全量历史（HEAD 的「未清空 Vec」语义），写回调 append。apply 按
    /// [`Self::applied_mutations`] 游标只取**新增尾部**（旧批已应用到活 DOM，重放会
    /// 重复建节点）。导航/load_html 换代清空 + 游标归零 + 清 engine 历史。
    shared_mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>>,
    /// [`Self::shared_mutations`] 的已应用游标（apply 尾部的起点）。
    applied_mutations: usize,
    /// HTTP 客户端。
    http_client: HttpClient,
    /// 进程内 JavaScript 沙箱（`external_script` 为 None 时使用）。
    js_sandbox: Option<Box<dyn zero_script_sandbox::Sandbox>>,
    /// IndexedDB 同步宿主桥（进程内 sandbox 路径）。
    indexed_db_bridge: zero_engine::IndexedDbBridge,
    /// IndexedDB origin 的可信页面 URL 来源。
    page_url_wire: std::sync::Arc<std::sync::Mutex<String>>,
    /// DOM shim（generate_js_dom_shim）是否已注入沙箱（M2：幂等保护——
    /// 重复执行会重置 _nodeMap 丢失监听器，故只注入一次）。
    js_shim_initialized: bool,
    /// 外部 JS 执行器（专用 JS 线程）。
    external_script: Option<ExternalScriptExecutor>,
    /// 外链脚本源获取器（进程内/headless 路径 fetch 外链脚本源）。
    script_source_fetcher: Option<ScriptSourceFetcher>,
    image_source_fetcher: Option<ImageSourceFetcher>,
    /// R34xx：同步 `__zw_fetch` 处理器（headless/testharness 本地资源路径）。
    fetch_handler: Option<zero_engine::fetch_bridge::FetchHandler>,
    /// 当前 URL。
    current_url: Option<String>,
    /// 当前导航 epoch（user-action node identity scope）。
    navigation_epoch: u64,
    /// 当前 live Document generation。
    document_generation: u64,
    /// 当前 focus owner。
    focus_owner: Option<zero_page_runtime::PageNodeRef>,
    /// 当前文档页面脚本是否已经执行。
    page_scripts_initialized: bool,
    /// 来源页 URL（导航前的 current_url；`document.referrer` 读，sync 到 pipeline）。
    referrer: Option<String>,
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
    /// 真实 Service Worker runtime/registration 单一 owner。
    sw_manager: std::sync::Arc<std::sync::Mutex<ServiceWorkerManager>>,
    /// Web Worker 实例（Dedicated Worker）。
    workers: HashMap<u64, WorkerRuntime>,
    /// Worker ID 生成器。
    next_worker_id: u64,
    /// WASM 实例缓存 — JS 端 WebAssembly.instantiate() 自动桥接到 wasm-sandbox。
    wasm_instances: HashMap<u64, WasmInstance>,
    // HTTP 响应缓存（性能门禁优化 S6，2026-08-08）：统一走
    // zero_net::shared_http_cache()——webview / fetch_proxy / net_pool 共享一份，
    // 避免同一 URL 在不同路径反复走网络。
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
    /// per-family 行度量与 `ex`/`ch` aspect 映射。字体相对单位默认启用；
    /// line-height 仍由 `ZW_PERFONT_LINEHEIGHT=1` 激活。
    font_metric_map: zero_render_foundation::font::FontFamilyMetricMap,
    /// 用户颜色方案偏好。
    prefers_color_scheme: PrefersColorSchemeValue,
    /// 渲染媒体类型（DC-12 @media print/screen；R1992）。默认 `Screen` = 零行为变更。
    media_type: MediaType,
    /// 安全上下文（HSTS + 混合内容 + CSP）。
    security_context: SecurityContext,
}

impl WebView {
    /// 创建新的 WebView。
    pub fn new(config: WebViewConfig) -> Self {
        Self::new_with_indexed_db_owner(config, IndexedDbOwner::in_memory())
    }

    /// 使用宿主提供的 IndexedDB owner 创建 WebView。
    ///
    /// 同一 browsing context 的 WebView 可共享一个 owner；private context 应使用独立的
    /// [`IndexedDbOwner::in_memory`] owner。
    pub fn new_with_indexed_db_owner(config: WebViewConfig, indexed_db_owner: IndexedDbOwner) -> Self {
        let mut pipeline = RenderPipeline::new(config.width as f32, config.height as f32);
        let canvas_registry: std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>> =
            std::sync::Arc::new(std::sync::Mutex::new(zero_engine::js_dom_bridge::CanvasRegistry::new()));
        pipeline.set_canvas_registry(Some(canvas_registry.clone()));
        // R1996：调试属性指示器（border-collapse/border-spacing/break/overflow-wrap 等，
        // 绘制于元素边角的彩色标记）默认**跳过**——WebView 是生产嵌入 API，不应默认显示
        // 调试标记（与 reftest/product-smoke 的 skip_indicators=true 一致；旧默认 false 致
        // zero-browser 产品页含 table/direction 等属性的元素显示调试标记）。需要指示器的
        // 嵌入者（dev 工具）可调 `set_skip_indicators(false)` 重新开启。
        pipeline.set_skip_indicators(true);
        let http_client = match config.http_timeout_secs {
            Some(secs) => HttpClient::with_timeout(secs),
            None => HttpClient::new(),
        };
        let external_script = config.external_script.clone();
        let script_source_fetcher = config.script_source_fetcher.clone();
        let image_source_fetcher = config.image_source_fetcher.clone();
        let fetch_handler = config.fetch_handler.clone();
        let indexed_db_bridge = zero_engine::IndexedDbBridge::new(indexed_db_owner.handler());
        let page_url_wire = std::sync::Arc::new(std::sync::Mutex::new(String::from("about:blank")));
        // 懒创建：js_sandbox 延后到首次实际执行脚本时初始化（见 ensure_sandbox）。
        // 无脚本页面（多数 WebView 页面）不创建 V8 isolate，显著降低常驻内存
        // （RSS ~0.2G/实例）；首次执行脚本时才有初始化成本，行为等价。
        let js_sandbox = None;
        Self {
            config,
            pipeline,
            canvas_registry,
            selector_handle_map: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            shared_mutations: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            applied_mutations: 0,
            http_client,
            js_sandbox,
            indexed_db_bridge,
            page_url_wire,
            js_shim_initialized: false,
            external_script,
            script_source_fetcher,
            image_source_fetcher,
            fetch_handler,
            current_url: None,
            navigation_epoch: 0,
            document_generation: 0,
            focus_owner: None,
            page_scripts_initialized: false,
            referrer: None,
            title: None,
            loading: false,
            last_render: None,
            cached_html: String::new(),
            cached_css: String::new(),
            event_callbacks: Vec::new(),
            sw_registry: ServiceWorkerRegistry::new(),
            sw_manager: std::sync::Arc::new(std::sync::Mutex::new(ServiceWorkerManager::new())),
            workers: HashMap::new(),
            next_worker_id: 1,
            wasm_instances: HashMap::new(),
            image_cache: ImageCache::default(),
            cached_image_sizes: HashMap::new(),
            cached_image_ratios: HashMap::new(),
            cached_image_no_ratio: HashMap::new(),
            font_resolver: HashMap::new(),
            font_metric_map: HashMap::new(),
            prefers_color_scheme: PrefersColorSchemeValue::Light,
            media_type: MediaType::Screen,
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
        self.pipeline.set_referrer(self.referrer.as_deref());
        let external_css = self.resolve_external_css(html, page_url);
        // R1794：外链 CSS + inline `<style>` 块中的 `url()` 图片引用一并抓取。
        let mut combined_css = external_css.clone();
        combined_css.push('\n');
        combined_css.push_str(&extract_html_style_text(html));
        let css_image_urls = extract_css_image_urls(&combined_css);
        let (image_sizes, image_ratios, image_no_ratio) =
            self.fetch_image_subresources(html, page_url, &css_image_urls);
        self.cached_image_sizes = image_sizes.clone();
        self.cached_image_ratios = image_ratios.clone();
        self.cached_image_no_ratio = image_no_ratio.clone();
        self.pipeline.set_image_sizes(image_sizes);
        // R34xx：@font-face 字体同步加载（headless/testharness 路径——canvas 文本像素光栅
        // 与页面文本的字体栈）。经 image_source_fetcher（wpt-data 文件映射）取字节 →
        // load_font + register_family_alias 进 canvas 注册表共享加载器。浏览器路径
        // （fetcher=None）跳过（async_load 既有 live 字体路径不变）。
        self.load_page_font_faces(&combined_css, page_url);
        self.pipeline.set_image_ratios(image_ratios);
        self.pipeline.set_image_no_ratio(image_no_ratio);
        external_css
    }

    /// R34xx：把系统 sans-serif（DejaVu/Liberation 等带 kern 数据的真字体）预载入 canvas
    /// 注册表共享 FontLoader 并注册为 'sans-serif'（canvas 默认字体的 kerning 面——
    /// 2d.text.drawing.style.fontKerning 的 measure 宽度对比依赖默认字体有 kern 数据；
    /// 镜像 reftest 路径的 load_system_fonts）。headless/testharness 专用；无匹配字体
    /// 或浏览器路径（fetcher=None 场景同样走，无 fetcher 依赖）静默跳过。
    pub fn load_canvas_system_sans_font(&mut self) {
        const CANDIDATES: [&str; 5] = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ];
        let Some(path) = CANDIDATES.iter().find(|p| std::path::Path::new(p).exists()) else {
            return;
        };
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let Ok(reg) = self.canvas_registry.lock() else {
            return;
        };
        let Ok(mut loader) = reg.font_loader.lock() else {
            return;
        };
        if let Ok(id) = loader.load_font(&bytes) {
            loader.register_family_alias("sans-serif", id);
            loader.register_family_alias("sans-serif:700", id);
            loader.register_family_alias("serif", id);
            loader.register_family_alias("monospace", id);
        }
    }

    /// resize / render 前把文档 URL 与图片固有尺寸同步回 pipeline。
    fn sync_pipeline_page_state(&mut self) {
        self.pipeline.set_document_url(self.current_url.as_deref());
        self.pipeline.set_referrer(self.referrer.as_deref());
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

    /// R34xx：headless 路径（testharness/wpt-runner 用 load_html 直接渲染）抓取外链 CSS
    /// 与 `<img>` 子资源——load_html 本身只渲染不抓图（图片抓取在导航路径）；
    /// 嵌入者可用本方法补抓。返回外链 CSS。
    pub fn fetch_page_images(&mut self, html: &str, page_url: &str) -> String {
        let external_css = self.prepare_page_subresources(html, page_url);
        self.cached_css = external_css.clone();
        external_css
    }

    /// 加载 HTML 内容。
    pub fn load_html(&mut self, html: &str, css: Option<&str>) -> WebViewRenderResult {
        self.document_generation = self.document_generation.wrapping_add(1);
        self.focus_owner = None;
        self.pipeline.set_focused_selector(None);
        self.page_scripts_initialized = false;
        // R100：文档换代——旧页 handle 在新页无效，清 selector→handle 反查表
        //（镜像 rect_bridge HandleSelectorMap 的导航清理语义）+ engine 正置表 +
        // 共享 mutations 队列/游标/历史。
        self.selector_handle_map
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        {
            let mut guard = self.shared_mutations.lock().unwrap_or_else(|e| e.into_inner());
            guard.clear();
        }
        self.applied_mutations = 0;
        zero_engine::js_dom_bridge::publish_forward_handle_map(None);
        zero_engine::js_dom_bridge::clear_mutation_history();
        // js-dom M1 L2 R102：清 live 查询源（旧页 Rc 失效；render_html 后由
        // apply/注册路径重新发布新 doc 句柄）。
        zero_engine::js_dom_bridge::publish_live_query_doc(None);
        self.cached_html = html.to_string();
        let css_str = css.unwrap_or("");
        self.cached_css = css_str.to_string();
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.set_media_type(self.media_type);
        let result = self.pipeline.render_html(html, css_str);
        let render_result = render_result_to_webview(&result);
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 脚本修改 DOM 后重新加载 HTML（保留已缓存 CSS，并刷新图片子资源）。
    pub fn reload_html_after_script(&mut self, html: &str) -> WebViewRenderResult {
        self.document_generation = self.document_generation.wrapping_add(1);
        self.focus_owner = None;
        self.cached_html = html.to_string();
        if let Some(page_url) = self.current_url.clone() {
            // R1794：脚本改 DOM 后刷新图片子资源，CSS url() 引用随 cached_css + inline <style> 一起重抓。
            let mut combined_css = self.cached_css.clone();
            combined_css.push('\n');
            combined_css.push_str(&extract_html_style_text(html));
            let css_image_urls = extract_css_image_urls(&combined_css);
            let (image_sizes, image_ratios, image_no_ratio) =
                self.fetch_image_subresources(html, &page_url, &css_image_urls);
            self.cached_image_sizes = image_sizes.clone();
            self.cached_image_ratios = image_ratios.clone();
            self.cached_image_no_ratio = image_no_ratio.clone();
            self.pipeline.set_image_sizes(image_sizes);
            self.pipeline.set_image_ratios(image_ratios);
            self.pipeline.set_image_no_ratio(image_no_ratio);
        }
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.set_media_type(self.media_type);
        let result = self.pipeline.render_html(html, &self.cached_css);
        let render_result = render_result_to_webview(&result);
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 应用 JS 侧 DOM 变更并渲染（M3-S9 生产路径入口，供浏览器 Tab JS worker）。
    ///
    /// 与 `run_page_scripts` 的进程内 mutation 应用同机制（直接改活 DOM，免 HTML
    /// 往返），但额外：① 同步 `cached_html`；② 刷新图片子资源（对齐
    /// `reload_html_after_script` 语义——新插入 `<img>`/CSS url() 需要固有尺寸）；
    /// ③ 返回 handle→唯一选择器映射（P1a gBCR path A，worker 持久 map）。
    pub fn apply_dom_mutations_and_render(
        &mut self,
        mutations: &[DomMutation],
    ) -> Result<(WebViewRenderResult, String, HashMap<String, String>), String> {
        let (result, html_snapshot, handle_selectors) =
            self.pipeline.render_with_dom_mutations(mutations, &self.cached_css)?;
        // R1794：只有内容 DOM 改变才刷新图片子资源。文本控件当前值由 retained 状态持有，
        // 不改变 HTML 快照，也不应让每个字符重扫整页图片。
        if let (Some(mutated), Some(page_url)) = (html_snapshot.as_deref(), self.current_url.clone()) {
            let mut combined_css = self.cached_css.clone();
            combined_css.push('\n');
            combined_css.push_str(&extract_html_style_text(mutated));
            let css_image_urls = extract_css_image_urls(&combined_css);
            let (image_sizes, image_ratios, image_no_ratio) =
                self.fetch_image_subresources(mutated, &page_url, &css_image_urls);
            self.cached_image_sizes = image_sizes.clone();
            self.cached_image_ratios = image_ratios.clone();
            self.cached_image_no_ratio = image_no_ratio.clone();
            self.pipeline.set_image_sizes(image_sizes);
            self.pipeline.set_image_ratios(image_ratios);
            self.pipeline.set_image_no_ratio(image_no_ratio);
        }
        if let Some(mutated) = html_snapshot {
            self.cached_html = mutated;
        }
        let render_result = render_result_to_webview(&result);
        self.last_render = Some(render_result.clone());
        Ok((render_result, self.cached_html.clone(), handle_selectors))
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
            // R34xx：headless/testharness 路径外链样式表经 image_source_fetcher 本地提供
            //（wpt-data 文件映射，与图片子资源同款；None → 回退 HTTP 网络）。
            let resp = if let Some(fetcher) = self.image_source_fetcher.as_ref()
                && let Some(bytes) = fetcher(&abs)
            {
                Ok(HttpResponse {
                    status_code: 200,
                    headers: vec![("content-type".into(), "text/css".into())],
                    body: bytes,
                    url: abs.clone(),
                    redirect_count: 0,
                })
            } else {
                self.resource_get(&abs, FetchPriority::CRITICAL, "style")
            };
            match resp {
                Ok(resp) => {
                    // CSS Syntax §6.2 charset determination：按 BOM / @charset / Content-Type
                    // charset 优先级解码（file:// 下 Content-Type charset 来自 `.headers`
                    // sidecar，file_url.rs 已注入）。旧 `resp.text()` 强制 UTF-8 致
                    // ISO-8859-1/UTF-16BE 等编码的 CSS 非 ASCII 字节变 U+FFFD，选择器失配
                    // （WPT at-charset-071~077 / character-encoding-031~037,041）。
                    let css = zero_net::charset::decode_css_bytes(&resp.body, resp.content_type());
                    // R34xx：外链样式表 url() 相对引用按**样式表 URL** 解析为绝对（CSS
                    // url() 的基准 = 引用它的样式表，非页面 URL）。此前相对 src 混入
                    // combined 后按页面 URL 解析 → 外链 @font-face 字体/背景图 404
                    // （driving: 2d.text.variationSelectors 的 variation-sequences.css
                    // `url(../../resources/vs/...)` 字体引用）。data:/绝对/占位符不动。
                    let css = absolutize_css_urls(&css, &abs);
                    combined.push_str(&css);
                    combined.push('\n');
                }
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
        css_image_urls: &[String],
    ) -> (
        HashMap<u64, (f32, f32)>,
        HashMap<u64, f32>,
        HashMap<u64, (Option<f32>, Option<f32>)>,
    ) {
        // R1794：合并 `<img src>` 与 CSS `url()` 图片引用（background-image /
        // list-style-image / border-image-source）。两类共用同一条 fetch+decode+key 路径：
        // 按绝对 URL 解析后 `image_resource_key` 入 image_sizes/ratios/no_ratio/cache，
        // painter 改后亦按 `image_resource_key(url, document_url)` 查找，端到端一致。
        let srcs = extract_img_srcs(html);
        let mut all_urls: Vec<&String> = srcs.iter().chain(css_image_urls.iter()).collect();
        all_urls.sort();
        all_urls.dedup();
        let mut image_sizes = HashMap::new();
        let mut image_ratios = HashMap::new();
        let mut image_no_ratio = HashMap::new();
        if all_urls.is_empty() {
            return (image_sizes, image_ratios, image_no_ratio);
        }
        let base = url::Url::parse(base_url).ok();
        for src in &all_urls {
            // R1987：data: URI（PNG/JPEG/WebP/SVG，base64 或 url-encoded）→ decode_data_uri 直接
            // 解码（in-scope img 子资源，goal line 118 SVG-as-img；render-foundation 共用按 magic
            // 分派）。data: 非相对，key = image_resource_key(src)（与 painter 查找一致：data: 经
            // is_non_relative_href 原样保留）。失败仅 warn 不阻断（与 HTTP fetch 失败同，宽松降级）。
            let (img, key_hash): (ImageData, u64) = if src.starts_with("data:") {
                match decode_data_uri(src) {
                    Ok(d) => (d, image_resource_key(src, None)),
                    Err(e) => {
                        tracing::warn!("data: URI image decode failed: {e}");
                        continue;
                    }
                }
            } else {
                let abs = match base.as_ref().and_then(|b| b.join(src).ok()) {
                    Some(u) => u.to_string(),
                    None => src.to_string(),
                };
                // 性能门禁优化 S5（2026-08-08）：ImageCache 命中即跳过网络——DOM 变更后
                // reload_html_after_script 每次全页同步重抓图片（webview.rs:452 每图
                // 全新 TCP+TLS，UI 线程阻塞）是「日志资源请求不断」的最大来源。
                let key_hash = image_resource_key(&abs, None);
                match self.image_cache.get(&ImageKey::new(key_hash)) {
                    Some(cached) => (cached.clone(), key_hash),
                    None => {
                        // R34xx：headless/testharness 路径经 image_source_fetcher 本地提供
                        //（wpt-data 文件映射）；None → 回退 HTTP 网络。
                        let fetched = self.image_source_fetcher.as_ref().and_then(|f| f(&abs)).or_else(|| {
                            self.resource_get(&abs, FetchPriority::MEDIUM, "image")
                                .ok()
                                .map(|resp| resp.body)
                        });
                        let bytes = match fetched {
                            Some(bytes) => bytes,
                            None => {
                                tracing::warn!("image {abs} fetch failed");
                                continue;
                            }
                        };
                        match decode_image(&bytes) {
                            Ok(img) => (img, key_hash),
                            Err(e) => {
                                tracing::warn!("image {abs} decode failed (PNG/JPEG/WebP): {e}");
                                continue;
                            }
                        }
                    }
                }
            };
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

    /// R34xx：@font-face 字体同步加载（headless/testharness 路径）。从 CSS 提取 font-face，
    /// 取首个 URL 源（wpt.test → image_source_fetcher 本地字节；其他 URL / 无 fetcher 跳过），
    /// load_font + register_family_alias 进 canvas 注册表共享 FontLoader（canvas 文本像素
    /// 光栅与页面字体栈共用）。浏览器路径（fetcher=None）跳过——async_load 的 live 字体
    /// 路径不变（零回归）。
    fn load_page_font_faces(&mut self, css: &str, base_url: &str) {
        let Some(fetcher) = self.image_source_fetcher.clone() else {
            return;
        };
        let faces = zero_engine::extract_font_faces(css);
        if faces.is_empty() {
            return;
        }
        let base = url::Url::parse(base_url).ok();
        let reg_guard = match self.canvas_registry.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!("canvas registry lock failed: {e}");
                return;
            }
        };
        let mut loader = match reg_guard.font_loader.lock() {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("font loader lock failed: {e}");
                return;
            }
        };
        for (family, sources, weight, is_italic, stretch, size_adjust, features, variations, unicode_ranges) in faces {
            let Some(src) = sources.first() else {
                continue;
            };
            let abs = match base.as_ref().and_then(|b| b.join(src).ok()) {
                Some(u) => u.to_string(),
                None => src.clone(),
            };
            let Some(bytes) = fetcher(&abs) else {
                tracing::warn!(src = %src, "font fetch failed");
                continue;
            };
            match loader.load_font(&bytes) {
                Ok(id) => {
                    loader.register_font_features(id, zero_engine::font_feature_settings_to_opentype(&features));
                    if zero_engine::font_variations_enabled() {
                        loader.register_font_variations(
                            id,
                            zero_engine::font_variation_settings_to_opentype(&variations),
                        );
                    }
                    loader.register_unicode_ranges(id, unicode_ranges);
                    if let Some(scale) = size_adjust {
                        loader.register_font_size_adjust(id, scale);
                    }
                    for alias in zero_render_foundation::font::font_face_aliases(&family, weight, is_italic, stretch) {
                        loader.register_family_alias(&alias, id);
                    }
                }
                Err(e) => tracing::warn!(family = %family, err = %e, "@font-face load failed"),
            }
        }
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
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.focus_owner = None;
        self.pipeline.set_focused_selector(None);
        self.page_scripts_initialized = false;
        let old_url = self.current_url.clone();
        // R3176：referrer = 导航前的页面 URL（document.referrer 读）。
        self.referrer = old_url.clone();
        self.current_url = Some(url.to_string());
        self.set_page_url_wire(url);
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
                self.set_page_url_wire(&https_url);
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

        // `ResourceLoader` 统一执行 private cache、条件再验证、在途合并和调度。
        match self.resource_get(&effective_url, FetchPriority::CRITICAL, "document") {
            Ok(response) => {
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
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.document_generation = self.document_generation.wrapping_add(1);
        self.focus_owner = None;
        self.pipeline.set_focused_selector(None);
        self.page_scripts_initialized = false;
        let old_url = self.current_url.clone();
        // R3176：referrer = 导航前的页面 URL（document.referrer 读）。
        self.referrer = old_url.clone();
        self.current_url = Some(url.to_string());
        self.set_page_url_wire(url);
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
        // R3176：referrer = 导航前的页面 URL（document.referrer 读）。
        self.referrer = self.current_url.clone();
        self.current_url = Some(page_url.to_string());
        self.set_page_url_wire(page_url);
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
        self.pipeline.set_media_type(self.media_type);
        let result = self.pipeline.render_html(&self.cached_html, &self.cached_css);
        let render_result = render_result_to_webview(&result);
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
        self.pipeline.set_media_type(self.media_type);
        let result = self.pipeline.repaint_cached_viewport(&self.cached_css)?;
        let render_result = render_result_to_webview(&result);
        self.last_render = Some(render_result.clone());
        Some(render_result)
    }

    /// 导航前设置文档 URL 与 pipeline 状态（供异步加载使用）。
    ///
    /// 必须丢弃上一文档的 `last_render` / 缓存，否则多进程增量 publish 会把旧帧 IPC 到浏览器。
    pub fn prepare_document_state(&mut self, page_url: &str) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.document_generation = self.document_generation.wrapping_add(1);
        self.focus_owner = None;
        self.pipeline.set_focused_selector(None);
        self.page_scripts_initialized = false;
        self.last_render = None;
        self.cached_html.clear();
        self.cached_css.clear();
        self.cached_image_sizes.clear();
        self.cached_image_ratios.clear();
        self.cached_image_no_ratio.clear();
        self.image_cache.clear();
        self.current_url = Some(page_url.to_string());
        self.set_page_url_wire(page_url);
        self.loading = true;
        self.pipeline.set_document_url(Some(page_url));
        self.security_context.set_page_origin(page_url);
        self.emit_event(&WebViewEvent::LoadStart(page_url.to_string()));
    }

    /// 推进预算渲染会话。
    pub fn advance_budget_session(&mut self, session: &mut BudgetedRenderSession, budget_ms: f64) -> BudgetAdvance {
        self.sync_pipeline_page_state();
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.set_media_type(self.media_type);
        self.pipeline.advance_budgeted_render(session, budget_ms)
    }

    /// 应用预算渲染结果到 WebView 状态。
    pub fn apply_render_result(&mut self, result: RenderResult, page_url: &str, finished: bool) {
        let render_result = render_result_to_webview(&result);
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

    /// 返回当前文档中文本表单控件的 live value 覆盖表。
    pub fn form_control_value_overrides(&self) -> HashMap<String, String> {
        self.pipeline.form_control_value_overrides()
    }

    /// 同步宿主持有的页面焦点 selector，供原生控件状态外观使用。
    #[doc(hidden)]
    pub fn set_focused_selector(&mut self, selector: Option<&str>) {
        self.pipeline.set_focused_selector(selector);
    }

    /// 从当前 live Document 查询 selector 对应的 opaque node handle。
    pub fn page_node_handle_for_selector(&self, selector: &str) -> Option<u64> {
        self.pipeline.page_node_handle_for_selector(selector)
    }

    /// 将当前 live Document 的 opaque node handle 解析为唯一选择器。
    pub fn selector_for_page_node_handle(&self, handle: u64) -> Option<String> {
        self.pipeline.selector_for_page_node_handle(handle)
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
        // 同尺寸 no-op：重建 RenderPipeline 会丢弃渲染状态、触发重解析与图片
        // 重解码（解码完成不保证触发重绘——曾致 parity 采集位图回退到缺图帧且
        // 永不恢复）。视口未变时无需任何处理。
        if self.config.width == width && self.config.height == height {
            return;
        }
        let doc_url = self.current_url.clone();
        let focused_selector = self.pipeline.focused_selector().map(str::to_string);
        let image_sizes = self.cached_image_sizes.clone();
        let image_ratios = self.cached_image_ratios.clone();
        let image_no_ratio = self.cached_image_no_ratio.clone();
        self.config.width = width;
        self.config.height = height;
        self.pipeline = RenderPipeline::new(width as f32, height as f32);
        self.pipeline.set_prefers_color_scheme(self.prefers_color_scheme);
        self.pipeline.set_media_type(self.media_type);
        self.pipeline.set_document_url(doc_url.as_deref());
        self.pipeline.set_focused_selector(focused_selector.as_deref());
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
        self.pipeline.set_font_metric_map(self.font_metric_map.clone());
    }

    /// 设置 CSS font-family 查找表（由宿主从 `FontLoader::build_font_resolver()` 构建）。
    pub fn set_font_resolver(&mut self, resolver: std::collections::HashMap<String, u32>) {
        self.font_resolver = resolver;
        self.pipeline.set_font_resolver(self.font_resolver.clone());
    }

    /// 设置 per-family 行度量与字体相对单位映射。
    ///
    /// 由宿主从 `FontLoader::build_line_metric_map()` 构建并传入。`ex`/`ch` 默认下推；
    /// `ZW_PERFONT_LINEHEIGHT=1` 仅控制 line-height:normal 的真实度量 provider。
    pub fn set_font_metric_map(&mut self, map: zero_render_foundation::font::FontFamilyMetricMap) {
        self.font_metric_map = map.clone();
        self.pipeline.set_font_metric_map(map);
    }

    /// 设置用户颜色方案偏好（影响 `prefers-color-scheme` 媒体查询）。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.prefers_color_scheme = scheme;
        self.pipeline.set_prefers_color_scheme(scheme);
    }

    /// 设置渲染媒体类型（DC-12 @media print/screen 级联；R1992 生产接线）。
    ///
    /// `Print` 使 `@media print` 规则在级联中生效——供浏览器打印预览 / 外部嵌入者
    /// 切换媒体类型。默认 `Screen` = 零行为变更。镜像 `set_prefers_color_scheme`：
    /// 持久化字段 + 即时下推 pipeline，后续 render 入口重放（见各 render 方法）。
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
        self.pipeline.set_media_type(media_type);
    }

    /// 设置是否跳过调试属性指示器（R1996）。
    ///
    /// 默认 `true`（跳过，生产嵌入干净渲染）。dev 工具需要可视化 CSS 属性指示器
    ///（border-collapse/break/overflow-wrap 等元素边角标记）时可传 `false` 重新开启。
    pub fn set_skip_indicators(&mut self, skip: bool) {
        self.pipeline.set_skip_indicators(skip);
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
            .resource_get(url, FetchPriority::HIGH, "script")
            .map_err(|e| WebViewError::Navigation(format!("fetch {url}: {e}")))?;
        resp.text()
            .map_err(|e| WebViewError::Navigation(format!("decode {url}: {e}")))
    }

    /// 经统一资源加载器同步取得 HTTP 子资源；`file:` 保持本地读取语义。
    fn resource_get(&self, url: &str, priority: FetchPriority, destination: &str) -> Result<HttpResponse, NetError> {
        if is_file_url(url) {
            return self.http_client.get(url);
        }
        ResourceLoader::shared()
            .submit(
                ResourceRequest::get(url, priority)
                    .with_destination(destination)
                    .with_timeout_secs(self.http_client.timeout_secs),
            )
            .recv()
            .map_err(|_| NetError::Network("resource loader worker exited".to_string()))?
            .map_err(|error| {
                if error == NetError::Timeout.to_string() {
                    NetError::Timeout
                } else {
                    NetError::Network(error)
                }
            })
    }

    /// 文档布局高度（CSS 逻辑像素）。
    pub fn document_height(&self) -> Option<f32> {
        self.pipeline.document_height()
    }

    /// 取出「自上次 render 后新产生」的过渡事件（R3248 transitionend + R3252 transitionrun/transitionstart）。
    /// 返回 [`zero_engine::TransitionEvent`] 列表（`kind` 区分 Run/Start/End）；每次调用清空缓冲。宿主据此向
    /// JS 派发 `new TransitionEvent(kind.as_event_type(), {propertyName, elapsedTime, bubbles:true})`。
    pub fn take_pending_transition_events(&mut self) -> Vec<zero_engine::TransitionEvent> {
        self.pipeline.take_pending_transition_events()
    }

    /// 取出「自上次 render 后新产生」的动画事件（R3249 animationend + R3250 animationiteration + R3251
    /// animationstart）。返回 [`zero_engine::AnimationEvent`] 列表（`kind` 区分 Start/End/Iteration）；每次调用
    /// 清空缓冲。宿主据此向 JS 派发
    /// `new AnimationEvent(kind.as_event_type(), {animationName, elapsedTime, bubbles:true})`。
    pub fn take_pending_animation_events(&mut self) -> Vec<zero_engine::AnimationEvent> {
        self.pipeline.take_pending_animation_events()
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

        self.ensure_sandbox()?;
        // P1b L1b（R3108）：执行前安装/刷新原生 DOM 绑定（native_dom=true 时），确保脚本
        // 可用 `__zw_native_element_for_id` 且 DOM 源为当前 live Document。
        // js-dom M6 S0q：quickjs feature 下走 QuickJS 原生绑定（quickjs_dom_bindings）。
        #[cfg(any(feature = "v8", feature = "quickjs"))]
        self.install_native_dom_bindings();
        // R3295：确保 DOM shim（`window`/`document`/`addEventListener`/`__zw_user_scroll` 等）
        // 已注入。此前仅 `run_page_scripts_impl` / `dispatch_event` 装 shim，**独立 `execute_script`
        // 调用（外部 embedder 或 BrowserApp 单进程无 js_worker 时的内联脚本路径）shim 未初始化** →
        // 脚本中 `window.addEventListener(...)` 抛 ReferenceError、`__zw_user_scroll` typeof='undefined'。
        // 仿 `dispatch_event`（webview.rs:1486）既定幂等模式（`js_shim_initialized` 守卫，仅首次装一次）。
        // 生产多进程 js_worker 路径经 `external_script` 早 return（:1110），不触此分支，零回归。
        self.ensure_js_shim()?;
        match self.js_sandbox.as_mut().expect("js sandbox").execute(script) {
            Ok(result) => {
                tracing::debug!("execute_script completed in {:.2}ms", result.execution_time_ms);
                // P1b L1b（R3108）：native 写经此路径直改 live cached_doc（不经 polyfill
                // DomMutation 队列）→ 检测并重渲染，使 native 写入可见于渲染。
                #[cfg(feature = "v8")]
                self.sync_render_after_native_dom();
                Ok(result.value)
            }
            Err(e) => Err(WebViewError::Script(format!("{e}"))),
        }
    }

    /// 惰性初始化进程内 JS 沙箱（V8/QuickJS isolate）。
    ///
    /// WebView 创建时不再无条件初始化沙箱（无脚本页面无需 V8 isolate）；
    /// 首次实际执行脚本时（execute_script / run_page_scripts / dispatch_event）
    /// 才创建。`external_script` 模式保持 None（与旧行为一致：无进程内沙箱）。
    ///
    /// https://html.spec.whatwg.org/#scripting — 页面无脚本时跳过脚本执行环境
    /// 初始化是合规优化（脚本执行语义不变，仅延后环境创建时机）。
    fn ensure_sandbox(&mut self) -> Result<(), WebViewError> {
        if self.external_script.is_some() {
            return Err(WebViewError::Script("no js sandbox".to_string()));
        }
        if self.js_sandbox.is_some() {
            return Ok(());
        }
        let js_config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            // 嵌入式页面多为轻 JS：初始堆限小（128MB），避免 V8 按系统内存
            // 预提交大堆；堆按需增长，上限仍由 heap_limit（默认无限制）控制。
            initial_heap_size: 128 * 1024 * 1024,
            ..Default::default()
        };
        // js-dom R84：v8+quickjs 组合态（workspace 单 cargo 调用 feature 并集）下双分支
        // 都编译 → `js_config` 被 move 两次（E0382）。v8 分支 clone 持有；单 feature
        // 语义不变。
        #[cfg(feature = "v8")]
        let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> = Box::new(
            zero_script_sandbox::V8Sandbox::with_config(js_config.clone())
                .map_err(|e| WebViewError::Script(format!("V8 sandbox init: {e}")))?,
        );
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> = Box::new(
            zero_script_sandbox::QuickJSSandbox::with_config(js_config)
                .map_err(|e| WebViewError::Script(format!("QuickJS sandbox init: {e}")))?,
        );
        Self::register_service_worker_host_callbacks(
            &mut *sandbox,
            self.sw_manager.clone(),
            self.script_source_fetcher.clone(),
            self.page_url_wire.clone(),
            self.http_client.timeout_secs,
        );
        self.indexed_db_bridge.register(&mut *sandbox, &self.page_url_wire);
        self.js_sandbox = Some(sandbox);
        Ok(())
    }

    fn set_page_url_wire(&self, page_url: &str) {
        if let Ok(mut current) = self.page_url_wire.lock() {
            *current = page_url.to_string();
        }
    }

    /// P1b L1b（R3108）：在持久 V8 Context 安装/刷新原生 DOM 绑定（kill-switch `native_dom`
    /// 默认关 → 零回归）。
    ///
    /// 每次脚本执行前调用：经 `cached_doc_shared` 取**当前** live Document（`render_html`
    /// 会替换 `cached_doc` 的 `Rc`，须每次刷新 DOM 源，避免 native 绑定指向旧 Document），
    /// 未渲染时回落 re-parse `cached_html` 快照（R3097 行为）。闭合 R3107：此前 native 绑定
    /// 仅 `run_page_scripts_impl` 非空脚本路径安装，`execute_script` 直接调用路径未安装
    /// （`__zw_native_element_for_id` 未定义 → R3107 de-inert 测试 red）。详见 RFC §3.7。
    #[cfg(feature = "v8")]
    fn install_native_dom_bindings(&mut self) {
        if !self.config.native_dom {
            return;
        }
        let Some(sandbox) = self.js_sandbox.as_mut() else {
            return;
        };
        let live = self.pipeline.cached_doc_shared();
        let html = self.cached_html.clone();
        let live_some = live.is_some();
        let _ = sandbox.install_native_bindings(Box::new(move |scope, ctx| {
            if let Some(doc) = live {
                zero_engine::dom_bindings::install_dom_bindings(scope, ctx, doc);
            } else {
                zero_engine::dom_bindings::install_dom_bindings_from_html(scope, ctx, &html);
            }
        }));
        tracing::debug!(live = live_some, "native DOM bindings installed (native_dom=true)");
    }

    /// js-dom goal M6 S0q：QuickJS 版 [`Self::install_native_dom_bindings`]（镜像 V8 逻辑，
    /// kill-switch `native_dom` 默认关 → 零回归）。经 `install_native_bindings_quickjs`
    /// escape-hatch 在持久 QuickJS Context 安装 `quickjs_dom_bindings`（`__zw_native_element_for_id`
    /// 工厂 + nodeType/tagName/nodeName/id 原生 getter，S0q PoC 面）。
    /// js-dom R84：v8+quickjs 组合态同名方法重复定义（E0592）——not(v8) 门控，quickjs-only
    /// 构建不变。
    #[cfg(all(feature = "quickjs", not(feature = "v8")))]
    fn install_native_dom_bindings(&mut self) {
        if !self.config.native_dom {
            return;
        }
        let Some(sandbox) = self.js_sandbox.as_mut() else {
            return;
        };
        let live = self.pipeline.cached_doc_shared();
        let html = self.cached_html.clone();
        let live_some = live.is_some();
        let installed = sandbox.install_native_bindings_quickjs(Box::new(move |ctx| {
            if let Some(doc) = live {
                zero_engine::quickjs_dom_bindings::install_dom_bindings_quickjs(ctx, doc);
            } else {
                zero_engine::quickjs_dom_bindings::install_dom_bindings_quickjs_from_html(ctx, &html);
            }
        }));
        if installed {
            tracing::debug!(
                live = live_some,
                "QuickJS native DOM bindings installed (native_dom=true)"
            );
        }
    }

    /// 幂等确保 DOM shim（[`generate_js_dom_shim`]）已注入进程内沙箱。
    ///
    /// shim 提供 `window`/`document`/`addEventListener`/`__zw_user_scroll` 等 web 全局，
    /// 是页面脚本与独立 `execute_script` 调用的基础。重复注入会重置内部 `_nodeMap` 丢失监听器，
    /// 故 `js_shim_initialized` 守卫仅装一次（与 `run_page_scripts_impl` 同款幂等语义）。
    ///
    /// 调用方须先 `ensure_sandbox`（保证 `js_sandbox = Some`）。生产多进程路径经 `external_script`
    /// 早 return，不触此方法。R3295 引入以闭合「独立 `execute_script` 调用缺 shim」gap。
    fn ensure_js_shim(&mut self) -> Result<(), WebViewError> {
        if self.js_shim_initialized {
            return Ok(());
        }
        let sandbox = self
            .js_sandbox
            .as_mut()
            .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;
        sandbox
            .execute(generate_js_dom_shim())
            .map_err(|e| WebViewError::Script(format!("DOM shim init: {e}")))?;
        self.js_shim_initialized = true;
        Ok(())
    }

    /// P1b L1b（R3108）：native 写触发重渲染，闭合 R3107 caveat ①（native 写「live 且渲染」）。
    ///
    /// native 绑定直接改 live `cached_doc`（不经 `DomMutation` 队列），polyfill 增量路径
    /// （`render_with_dom_mutations`）不会感知。本方法序列化 live Document 一次，比对
    /// `cached_html`：不一致 → 全量重渲染（`repaint_cached_viewport` 重算 style+layout+paint，
    /// native mutation 可任意——属性/树/文本，无法像 polyfill 那样分类增量）+ 同步
    /// `cached_html`/`last_render`，使 native 写入可见于渲染。polyfill 路径已同步（一致）
    /// 或 native 未改 → no-op（零额外开销）。详见 `docs/specs/p1b-v8-native-bindings-rfc.md` §3.7。
    #[cfg(feature = "v8")]
    fn sync_render_after_native_dom(&mut self) {
        if !self.config.native_dom {
            return;
        }
        let live_html = match self.pipeline.cached_doc_shared() {
            Some(doc_rc) => {
                let doc = doc_rc.borrow();
                let root = doc.root();
                doc.outer_html(root)
            }
            None => return,
        };
        if live_html == self.cached_html {
            return;
        }
        if let Some(result) = self.pipeline.repaint_cached_viewport(&self.cached_css) {
            // repaint 只读不改 Document，live_html（repaint 前序列化）仍等于当前 DOM。
            self.cached_html = live_html;
            self.last_render = Some(render_result_to_webview(&result));
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

        // js-dom M3 R100：**mutation 收集 + 应用**——execute 路径此前只跑脚本不应用
        // DOM 变更（`__zw_*` 回调未注册 → JS 侧写静默丢失，或回调仍指向
        // run_page_scripts 已结束的旧 Arc——变更记录进不可达队列永不应用；Vue
        // mount 后 host 侧 querySelector 全 miss 的根因）。镜像
        // `run_page_scripts_impl` 的机制：注册回调 → 执行 → 应用变更
        // （直接改活 DOM + 快照同步）。R100 共享队列：与 run_page_scripts 共用
        // `shared_mutations`（append-only，读回调天然看全量历史），apply 只取新增尾部。
        let mutations = self.shared_mutations.clone();
        let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
            std::sync::Arc::new(std::sync::Mutex::new(self.cached_html.clone()));
        let page_url = self.page_url_wire.clone();
        // R100：惰性沙箱先行——直接调用方（未跑过 run_page_scripts 的 WebView，如
        // wasm_bridge 集成测试）此前经 execute_script→ensure_sandbox 建沙箱；本函数
        // 现在先注册回调，须先 ensure_sandbox（幂等，已有则 no-op）。
        self.ensure_sandbox()?;
        {
            let sandbox = self
                .js_sandbox
                .as_mut()
                .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;
            // js-dom M1 L2 R102：发布 live 查询源（pipeline cached_doc 共享句柄——查询回调
            // 无 pending structural mutations 时直读 live，消 re-parse；None 回落快照路径）。
            zero_engine::js_dom_bridge::publish_live_query_doc(self.pipeline.cached_doc_shared());
            register_dom_callbacks(&mut **sandbox, &mutations, &dom_html, &page_url, &self.canvas_registry);
            // js-dom M3 R100：selector→handle 反查回调——JS 侧 query 命中后按唯一选择器
            // 反查「本元素是否是 createElement 建立的 handle 节点」，命中则包装回原
            // handle proxy（identity 统一，见 WebView.selector_handle_map 文档）。
            Self::register_identity_bridge_callback(&self.selector_handle_map, &mut **sandbox);
        }

        let result = self.execute_script(&full_script)?;

        // 应用本 execute 新增的 DOM 变更（游标尾部；与 run_page_scripts_impl 同机制：
        // 活 DOM 直改 + cached_html 快照同步 + 重渲染）。
        self.apply_pending_shared_mutations()?;

        // 检查是否有 WASM 桥接请求
        let bridge_result = self.process_wasm_bridge(&result)?;
        Ok(bridge_result)
    }

    /// 执行页面 `<script>`（M2：真实 DOM 桥——与 wpt-runner reftest 同机制：
    /// generate_js_dom_shim + register_dom_callbacks，DOM 操作记录为 mutation
    /// 后应用回 HTML 并重新渲染）。
    ///
    /// 注意：本方法执行的是**页面真实 DOM 桥**（mutation 应用机制），与
    /// `execute_script_with_dom`（JS 侧虚拟 DOM polyfill）不同——页面交互
    /// （事件监听器注册等）须用本方法。
    pub fn run_page_scripts(&mut self) -> Result<String, WebViewError> {
        self.run_page_scripts_impl(false)
    }

    /// 严格模式页面脚本执行——与 [`run_page_scripts`](Self::run_page_scripts) 一致，但**首个内联脚本抛异常时
    /// 返 `Err`**（非 warn+continue）。供 WPT runner 等「脚本必须无异常」语义的调用方用（闭合 web_api/js_dom
    /// 测试用例「空洞通过」——既不执行内联 JS，故 API 真损/行为错不会被发现）。
    pub fn run_page_scripts_strict(&mut self) -> Result<String, WebViewError> {
        self.run_page_scripts_impl(true)
    }

    fn run_page_scripts_impl(&mut self, strict: bool) -> Result<String, WebViewError> {
        if self.page_scripts_initialized {
            return Ok(self.cached_html.clone());
        }
        let html = self.cached_html.clone();
        let scripts = extract_page_scripts_indexed(&html);
        if scripts.is_empty() {
            self.page_scripts_initialized = true;
            return Ok(html);
        }
        self.ensure_sandbox()?;
        // P1b L1b（R3108）：原生 DOM 绑定安装抽到 `install_native_dom_bindings`（与
        // `execute_script` 路径共用；每次刷新 live Document 源）。须在下方 `sandbox` 长
        // 借用前调用（helper 内部短暂借 js_sandbox 后释放）。kill-switch 默认关 → 零回归。
        // js-dom M6 S0q：quickjs feature 下走 QuickJS 原生绑定（quickjs_dom_bindings）。
        #[cfg(any(feature = "v8", feature = "quickjs"))]
        self.install_native_dom_bindings();
        let sandbox = self
            .js_sandbox
            .as_mut()
            .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;

        // R100：共享 append-only 队列（读回调全量历史语义，apply 取游标尾部）+
        // identity 反查回调。
        let mutations = self.shared_mutations.clone();
        let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
            std::sync::Arc::new(std::sync::Mutex::new(html.clone()));
        let page_url = self.page_url_wire.clone();
        // js-dom M1 L2 R102：发布 live 查询源（pipeline cached_doc 共享句柄——查询回调
        // 无 pending structural mutations 时直读 live，消 re-parse；None 回落快照路径）。
        zero_engine::js_dom_bridge::publish_live_query_doc(self.pipeline.cached_doc_shared());
        register_dom_callbacks(&mut **sandbox, &mutations, &dom_html, &page_url, &self.canvas_registry);
        Self::register_identity_bridge_callback(&self.selector_handle_map, &mut **sandbox);

        // 原生 DOM 绑定已在 ensure_sandbox 后经 `install_native_dom_bindings` 安装（见上）。

        // R34xx：__zw_get_image_wire —— shim drawImage/createPattern img 元素源（G5）：
        // 按 src 查 image_cache 快照 → 编码 canvas ImageData wire（"w:h;r,g,b,a,..."）。
        let image_data_snapshot = self.image_cache.snapshot_entries();
        let page_url_wire = page_url.clone();
        // R56h：运行时 img（`new Image().src`）——快照 miss 时现场抓取 + 解码
        //（2d.drawImage.svg / zerosource.image 的 loadImage 运行时加载；fetch 经
        // 同一 fetch_handler——wpt-data 本地映射，无网络）。
        let fetch_handler_for_wire = self.fetch_handler.clone();
        sandbox.register_callback(
            "__zw_get_image_wire",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                if src.is_empty() {
                    return String::new();
                }
                let base = page_url_wire.lock().unwrap();
                let abs = zero_engine::resolve_document_url(&base, src);
                let key = zero_engine::image_resource_key(&abs, None);
                let img = match image_data_snapshot.get(&key) {
                    Some(img) => img.clone(),
                    None => {
                        // R56h：运行时图片快照 miss → fetch handler 现场抓取 + 解码。
                        drop(base);
                        let fetched = fetch_handler_for_wire.as_ref().and_then(|handler| {
                            let req = zero_engine::fetch_bridge::FetchRequest {
                                method: "GET".into(),
                                url: abs.clone(),
                                headers: Vec::new(),
                                body: None,
                                body_bytes: None,
                            };
                            handler(&req).ok()
                        });
                        match fetched.and_then(|resp| resp.body_bytes) {
                            Some(bytes) => match zero_render_foundation::image_cache::decode_image_bytes(&bytes) {
                                Ok(img) => img,
                                Err(_) => return String::new(),
                            },
                            None => return String::new(),
                        }
                    }
                };
                let mut out = format!("{}:{};", img.width, img.height);
                let mut first = true;
                for b in &img.pixels {
                    if !first {
                        out.push(',');
                    }
                    first = false;
                    out.push_str(&b.to_string());
                }
                out
            }),
        );
        // R34xx：__zw_get_image_size —— shim img naturalWidth/naturalHeight 查询（G5）。
        // 图片尺寸由 fetch_image_subresources（load_html 时）填入 cached_image_sizes；
        // 注册时 clone 快照（脚本执行在 load_html 后）。
        let image_sizes_snapshot = self.cached_image_sizes.clone();
        let page_url_sizes = page_url.clone();
        sandbox.register_callback(
            "__zw_get_image_size",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                if src.is_empty() {
                    return String::new();
                }
                let base = page_url_sizes.lock().unwrap();
                let abs = zero_engine::resolve_document_url(&base, src);
                let key = zero_engine::image_resource_key(&abs, None);
                match image_sizes_snapshot.get(&key) {
                    Some((w, h)) => format!("{w},{h}"),
                    None => String::new(),
                }
            }),
        );

        // R34xx：__zw_fetch（同步契约）——headless/testharness 宿主经 config.fetch_handler 提供
        // 本地资源（wpt-data 文件映射：/images/*、/fonts/* 等）。None → 不注册（shim typeof-check
        // 落 ok:false stub；浏览器路径由 app 层 FetchBridge 注册异步 __zw_fetch，互斥不重叠）。
        // 回调同步执行 handler 并直接返 wire（shim R34xx 支持同步返回）；wire 格式与 fetch_bridge
        // 相同（__zwfr: status\x1fstatusText\x1fheaders\x1fbody；body 二进制经 __zw_bytes: csv）。
        if let Some(fetch_handler) = self.fetch_handler.clone() {
            sandbox.register_callback(
                "__zw_fetch",
                Box::new(move |args: &[String]| -> String {
                    let method = args.get(1).cloned().unwrap_or_else(|| "GET".to_string());
                    let url = args.get(2).cloned().unwrap_or_default();
                    let headers =
                        zero_engine::fetch_bridge::decode_headers(args.get(3).map(String::as_str).unwrap_or(""));
                    let body_raw = args.get(4).cloned().unwrap_or_default();
                    let (body, body_bytes) =
                        if let Some(bytes) = zero_engine::fetch_bridge::decode_body_bytes_raw(&body_raw) {
                            (None, Some(bytes))
                        } else if body_raw.is_empty() {
                            (None, None)
                        } else {
                            (Some(body_raw), None)
                        };
                    let req = zero_engine::fetch_bridge::FetchRequest {
                        url,
                        method,
                        headers,
                        body,
                        body_bytes,
                    };
                    match fetch_handler(&req) {
                        Ok(resp) => zero_engine::fetch_bridge::serialize_response(&resp),
                        Err(msg) => format!("__zw_fetch_error:{msg}"),
                    }
                }),
            );
        }
        // R34xx：__zw_load_font（同步契约——headless/testharness 的 FontFace.load() 路径；
        // 与 __zw_fetch 同款：回调同步加载并直接返 'ok'/'err'，shim 同步 resolve）。
        // 源 'url("/fonts/CanvasTest.ttf")' → 提取 URL → image_source_fetcher（wpt-data）
        // → registry font_loader load + register_family_alias（weight/italic 键）。
        if let Some(font_fetcher) = self.image_source_fetcher.clone() {
            let reg = self.canvas_registry.clone();
            let page_url_fonts = page_url.clone();
            sandbox.register_callback(
                "__zw_load_font",
                Box::new(move |args: &[String]| -> String {
                    let family = args.first().cloned().unwrap_or_default();
                    let src = args.get(1).cloned().unwrap_or_default();
                    let weight_num = args.get(3).and_then(|s| s.trim().parse::<u16>().ok());
                    let is_italic = args.get(4).map(|s| s == "true").unwrap_or(false);
                    // 提取 url(...) 内的 URL。
                    let url = match src.find("url(") {
                        Some(i) => {
                            let rest = &src[i + 4..];
                            let rest = rest.trim_start();
                            let quote = rest.chars().next().unwrap_or('"');
                            let inner = if quote == '"' || quote == '\'' {
                                rest.get(1..).and_then(|r| r.split(quote).next()).unwrap_or("")
                            } else {
                                rest.split(|c: char| c == ')' || c.is_whitespace()).next().unwrap_or("")
                            };
                            inner.to_string()
                        }
                        None => src.clone(),
                    };
                    if url.is_empty() || family.is_empty() {
                        return "err".to_string();
                    }
                    let base = page_url_fonts.lock().unwrap();
                    let abs = zero_engine::resolve_document_url(&base, &url);
                    let Some(bytes) = font_fetcher(&abs) else {
                        return "err".to_string();
                    };
                    let Ok(reg_guard) = reg.lock() else {
                        return "err".to_string();
                    };
                    let Ok(mut loader) = reg_guard.font_loader.lock() else {
                        return "err".to_string();
                    };
                    match loader.load_font(&bytes) {
                        Ok(id) => {
                            let weight = weight_num.unwrap_or(400);
                            let key = if is_italic {
                                format!("{}:italic:{}", family.to_ascii_lowercase(), weight)
                            } else {
                                format!("{}:{}", family.to_ascii_lowercase(), weight)
                            };
                            loader.register_family_alias(&key, id);
                            loader.register_family_alias(&family, id);
                            "ok".into()
                        }
                        Err(_) => "err".into(),
                    }
                }),
            );
        }
        // R3091：__zw_fetch_script（进程内路径，backed by ScriptSourceFetcher）—— 供 shim Worker 构造器
        //（外链 URL）/ 动态 import() fetch 外链脚本源。fetcher 未配 → 不注册（shim typeof-check no-op）。
        if let Some(fetcher) = self.script_source_fetcher.clone() {
            let fetcher_compile = fetcher.clone();
            sandbox.register_callback(
                "__zw_fetch_script",
                Box::new(move |args| {
                    let page = args.first().map(String::as_str).unwrap_or("");
                    let src = args.get(1).map(String::as_str).unwrap_or("");
                    fetcher(page, src).unwrap_or_default()
                }),
            );
            // R3093：__zw_compile_module（动态 import() 外链 module）—— module prelude 的 __zw_load_module
            // 对未缓存 spec 调此回调。fetch 源经 fetcher + compile_dependency_iife（imports 空存根，transitive
            // fetch defer）→ 返 IIFE（__zw_load_module eval 为 namespace）。browser/renderer 多进程模式各自注册
            //（http fetch + collect_module_deps 递归）；本进程内路径用 fetcher + 单层 imports 空存根。
            sandbox.register_callback(
                "__zw_compile_module",
                Box::new(move |args| {
                    let spec = args.first().map(String::as_str).unwrap_or("");
                    let parent = args.get(1).map(String::as_str).unwrap_or("about:blank");
                    if spec.is_empty() {
                        return String::new();
                    }
                    // R3094：递归 fetch transitive deps（闭合 module graph）。registry 按原 spec 注册源，
                    // 循环防护按解析后 URL；fetch 失败 → 返空（__zw_load_module 抛 Module not found）。
                    let mut reg = zero_script_sandbox::ModuleRegistry::new();
                    let mut visited = std::collections::HashSet::new();
                    let compiled =
                        collect_module_deps_recursive(&fetcher_compile, parent, spec, &mut reg, &mut visited)
                            .ok()
                            .and_then(|_| zero_script_sandbox::compile_dependency_iife(spec, &reg).ok());
                    match compiled {
                        Some(iife) => iife,
                        None => {
                            tracing::warn!("compile module {spec} (transitive fetch)");
                            String::new()
                        }
                    }
                }),
            );
        }

        // DOM shim 只注入一次（重复执行会重置 _nodeMap 丢失监听器）
        if !self.js_shim_initialized {
            if let Err(e) = sandbox.execute(generate_js_dom_shim()) {
                return Err(WebViewError::Script(format!("DOM shim init: {e}")));
            }
            self.js_shim_initialized = true;
        }
        for (script, script_index) in scripts {
            let (code, is_module) = match script {
                zero_engine::pipeline::PageScript::Inline(c) => (c, false),
                // R3083：`<script type="module">` 经 compile_module_script 转 import/export 为经典可执行
                // 代码后执行（旧 InlineModule 与 Inline 同走经典路径→`import` 抛 SyntaxError）。headless 进程内
                // 模式不 fetch 外链模块，故把模块源中引用的 import 标识符预注册为**空存根**（副作用导入 no-op，
                // 命名导入得 undefined/empty namespace）——使模块自身 body 可执行。spec ES Modules Tier 1。
                zero_engine::pipeline::PageScript::InlineModule(c) => (c, true),
                zero_engine::pipeline::PageScript::External(src) => {
                    // R3090：外链经典脚本。若配 script_source_fetcher，fetch 源后按经典脚本执行；
                    // 否则跳过（离线/headless 语义，与 reftest 一致）。external_script 多进程模式不走此路径。
                    match &self.script_source_fetcher {
                        Some(fetch) => {
                            let page_url = self.current_url.as_deref().unwrap_or("about:blank");
                            match fetch(page_url, &src) {
                                Ok(code) => (code, false),
                                Err(e) => {
                                    if strict {
                                        return Err(WebViewError::Script(format!("external script {src}: {e}")));
                                    }
                                    tracing::warn!("外链脚本 fetch 失败 {src}: {e}");
                                    continue;
                                }
                            }
                        }
                        None => continue,
                    }
                }
                zero_engine::pipeline::PageScript::ExternalModule(src) => {
                    // R3090：外链模块脚本。fetch 源后 is_module=true → 走 InlineModule 编译路径
                    //（预注册空存根 + compile_module_script，外链模块的进一步 import 仍为空存根，defer 递归 fetch）。
                    match &self.script_source_fetcher {
                        Some(fetch) => {
                            let page_url = self.current_url.as_deref().unwrap_or("about:blank");
                            match fetch(page_url, &src) {
                                Ok(code) => (code, true),
                                Err(e) => {
                                    if strict {
                                        return Err(WebViewError::Script(format!("external module {src}: {e}")));
                                    }
                                    tracing::warn!("外链模块 fetch 失败 {src}: {e}");
                                    continue;
                                }
                            }
                        }
                        None => continue,
                    }
                }
            };
            let full = if is_module {
                // 预注册空存根 + 编译（import→空 namespace、export→_exports；动态 import() 经 prelude）。
                // R3093：fetcher 配置时只用**静态** import 预存根（动态 import() 留给运行时 __zw_compile_module
                // fetch，避免预存根 empty namespace 短路）；无 fetcher 仍用全量（动态 import 预存根返空 namespace）。
                let mut registry = zero_script_sandbox::ModuleRegistry::new();
                let specs = if self.script_source_fetcher.is_some() {
                    zero_script_sandbox::extract_static_module_import_specifiers(&code)
                } else {
                    zero_script_sandbox::extract_module_import_specifiers(&code)
                };
                for spec in specs {
                    registry.register(&spec, "");
                }
                let url = self.current_url.as_deref().unwrap_or("about:blank");
                match zero_script_sandbox::compile_module_script(&code, url, &registry) {
                    Ok(transformed) => {
                        let prelude = zero_script_sandbox::build_module_runtime_prelude(&registry).unwrap_or_default();
                        format!("__zw_begin_script && __zw_begin_script();\n{prelude}\n{transformed}")
                    }
                    Err(e) => {
                        if strict {
                            return Err(WebViewError::Script(format!("module compile: {e}")));
                        }
                        tracing::warn!("模块脚本编译警告: {e}");
                        continue;
                    }
                }
            } else {
                // classic 脚本：__zw_begin_script 前置（body onload 反射）+ R3258 设/清 document.currentScript
                //（try-finally 保证即便抛错也清；spec classic 执行期 currentScript 指向自身元素）。
                format!(
                    "{set}\ntry{{__zw_begin_script&&__zw_begin_script();\n{code}\n}}finally{{{clear}}}",
                    set = script_set_current_script(script_index),
                    clear = script_clear_current_script(),
                )
            };
            if let Err(e) = sandbox.execute(&full) {
                if strict {
                    return Err(WebViewError::Script(format!("page script: {e}")));
                }
                tracing::warn!("页面脚本执行警告: {e}");
            }
        }
        self.page_scripts_initialized = true;
        let pending = {
            let guard = self.shared_mutations.lock().unwrap_or_else(|e| e.into_inner());
            guard.len() > self.applied_mutations
        };
        if !pending {
            // P1b L1b（R3108）：polyfill 无变更，但 native 绑定可能已直改 live doc → 检测并重渲染
            //（helper 内部比对 live-doc vs cached_html；一致则 no-op）。返回最新 cached_html 快照。
            #[cfg(feature = "v8")]
            self.sync_render_after_native_dom();
            return Ok(self.cached_html.clone());
        }
        // M3-S9：DOM 变更直接应用到活 DOM（pipeline.cached_doc），不再回写 HTML 重 parse
        //（旧路径 apply_mutations_to_html → load_html 全量重建，大页面 parse 占 ~30%）。
        // repaint 后返回新 HTML 快照同步 cached_html（DOM 查询消费的快照须与活 DOM 一致）。
        // R100：共享队列游标尾部（见 apply_pending_shared_mutations 文档）。
        self.apply_pending_shared_mutations()?;
        Ok(self.cached_html.clone())
    }

    /// 向页面元素派发 DOM 事件（M2：如 click/submit），触发页面注册的
    /// 监听器（addEventListener / onclick 属性经 shim 桥接）。基于
    /// `__zw_dispatch_event` shim（reftest 已验证的机制），mutation 应用
    /// 后重新渲染。
    ///
    /// **P1b host→page native 派发（R3121）**：`native_dom` 开启时，页面经 native
    /// `addEventListener` 注册的监听器存于 engine `dom_bindings::gc::LISTENERS`，
    /// polyfill `__zw_dispatch_event` 不达——额外经 `__zw_native_query_selector` +
    /// 原生 `dispatchEvent` 派发（[`script_dispatch_native_event`]）。闭合 S4 host 驱动
    /// 半边；`native_dom` 关（默认）→ 仅 polyfill 路径，零回归。
    pub fn dispatch_event(&mut self, selector: &str, event_type: &str) -> Result<(), WebViewError> {
        self.run_page_scripts()?; // 确保监听器已注册
        let script = script_dispatch_dom_event(selector, event_type, None);
        self.ensure_sandbox()?;
        let sandbox = self
            .js_sandbox
            .as_mut()
            .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;

        // R100：共享 append-only 队列（读回调全量历史语义，apply 取游标尾部）。
        let mutations = self.shared_mutations.clone();
        let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
            std::sync::Arc::new(std::sync::Mutex::new(self.cached_html.clone()));
        let page_url = self.page_url_wire.clone();
        // js-dom M1 L2 R102：发布 live 查询源（pipeline cached_doc 共享句柄——查询回调
        // 无 pending structural mutations 时直读 live，消 re-parse；None 回落快照路径）。
        zero_engine::js_dom_bridge::publish_live_query_doc(self.pipeline.cached_doc_shared());
        register_dom_callbacks(&mut **sandbox, &mutations, &dom_html, &page_url, &self.canvas_registry);
        Self::register_identity_bridge_callback(&self.selector_handle_map, &mut **sandbox);

        // 确保 shim 已注入（无页面脚本时 run_page_scripts 提前返回，shim 未初始化）
        // R3295：改用共享 `ensure_js_shim` helper（与 `execute_script` 同款幂等语义）。
        let sandbox = {
            let _ = sandbox; // 结束上一段 js_sandbox 借用
            self.ensure_js_shim()?;
            self.js_sandbox
                .as_mut()
                .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?
        };

        sandbox
            .execute(&script)
            .map_err(|e| WebViewError::Script(format!("dispatch {event_type}: {e}")))?;
        // P1b host→page native 派发（R3121）：native_dom 开 → 经原生绑定派发到 native LISTENERS
        //（polyfill __zw_dispatch_event 不达）。native 回调直改 live doc，下方 apply 空尾
        // 分支的 sync_render_after_native_dom 拾取重渲染。native_dom 关（默认）→ 跳过，零回归。
        #[cfg(feature = "v8")]
        if self.config.native_dom {
            let _ = sandbox.execute(&script_dispatch_native_event(selector, event_type));
        }
        let pending = {
            let guard = self.shared_mutations.lock().unwrap_or_else(|e| e.into_inner());
            guard.len() > self.applied_mutations
        };
        if !pending {
            // P1b L1b（R3108）：事件处理器经 native 绑定可能已直改 live doc → 检测并重渲染。
            #[cfg(feature = "v8")]
            self.sync_render_after_native_dom();
            return Ok(());
        }
        // M3-S9：DOM 变更直接应用到活 DOM（见 run_page_scripts_impl 同款注释）。
        self.apply_pending_shared_mutations()?;
        Ok(())
    }

    /// js-dom M3 R100：注册 selector→handle 反查回调（JS 侧 query 返回点 identity 统一，
    /// 见 `selector_handle_map` 文档）。三个注册路径（run_page_scripts /
    /// execute_script_with_dom / dispatch_event）统一调用。收 map Arc（调用方先 clone，
    /// 避免与 js_sandbox 可变借用冲突）。
    fn register_identity_bridge_callback(
        sel_map: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
        sandbox: &mut dyn zero_script_sandbox::Sandbox,
    ) {
        let sel_map = sel_map.clone();
        sandbox.register_callback(
            "__zw_handle_for_selector",
            Box::new(move |args: &[String]| -> String {
                let sel = args.first().map(String::as_str).unwrap_or("");
                sel_map
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(sel)
                    .cloned()
                    .unwrap_or_default()
            }),
        );
    }

    /// js-dom M3 R100：应用共享队列的**新增尾部**（`applied_mutations` 游标起）。
    ///
    /// 共享 append-only 队列使读回调（`__zw_get_text_handle` 等 latest-wins 查询）在
    /// 任意后续 execute 中都能看到全量历史——HEAD 的「run_page_scripts 队列不清空」
    /// 语义的显式化。apply 只取尾部（旧批已应用到活 DOM，重放会重复建节点/重复插入）。
    /// 正置 handle→selector 表作为 apply 的持久翻译层（跨 execute 旧 handle 的写
    /// mutation 锚定回 selector）。apply 后 merge handle map + 双表发布。
    fn apply_pending_shared_mutations(&mut self) -> Result<(), WebViewError> {
        let tail: Vec<DomMutation> = {
            let guard = self.shared_mutations.lock().unwrap_or_else(|e| e.into_inner());
            guard[self.applied_mutations.min(guard.len())..].to_vec()
        };
        if tail.is_empty() {
            return Ok(());
        }
        // R100：未 load_html 的直接调用方（wasm_bridge/edge 类测试）——pipeline 无
        // cached_doc，先按当前 cached_html（空则最小空页）初始化渲染一次，使 apply
        // 有活 DOM 可落（旧行为：execute 路径不 apply，无此需求；本函数引入 apply 后
        // 补的等价初始化）。
        if self.pipeline.cached_doc_shared().is_none() {
            let html = if self.cached_html.is_empty() {
                "<html><body></body></html>".to_string()
            } else {
                self.cached_html.clone()
            };
            let result = self.pipeline.render_html(&html, &self.cached_css);
            self.last_render = Some(render_result_to_webview(&result));
        }
        let forward: std::collections::HashMap<String, String> = {
            let sel_map = self.selector_handle_map.lock().unwrap_or_else(|e| e.into_inner());
            sel_map.iter().map(|(s, h)| (h.clone(), s.clone())).collect()
        };
        let (render_result, html_snapshot, handles) = self
            .pipeline
            .render_with_dom_mutations_persistent(&tail, &self.cached_css, Some(&forward))
            .map_err(|e| WebViewError::Script(format!("apply mutations: {e}")))?;
        if let Some(mutated) = html_snapshot {
            self.cached_html = mutated;
        }
        self.applied_mutations += tail.len();
        self.merge_handle_selectors(&handles);
        self.last_render = Some(render_result_to_webview(&render_result));
        Ok(())
    }

    /// js-dom M3 R100：把 apply 返回的 handle→selector 映射 merge 进持久反查表
    /// （`selector_handle_map`，倒置）并发布正置表到 engine thread-local
    /// （`publish_forward_handle_map`——host 回调如 `__zw_get_tag_handle` 的跨
    /// execute 回落查询源）。三个 apply 路径（run_page_scripts / dispatch_event /
    /// execute_script_with_dom）统一走此 helper，保证任一路径建立的 handle 后续
    /// 可锚定。
    fn merge_handle_selectors(&mut self, handles: &std::collections::HashMap<String, String>) {
        if handles.is_empty() {
            return;
        }
        let forward = {
            let mut sel_map = self.selector_handle_map.lock().unwrap_or_else(|e| e.into_inner());
            for (handle, sel) in handles {
                sel_map.insert(sel.clone(), handle.clone());
            }
            sel_map
                .iter()
                .map(|(s, h)| (h.clone(), s.clone()))
                .collect::<std::collections::HashMap<String, String>>()
        };
        zero_engine::js_dom_bridge::publish_forward_handle_map(Some(std::sync::Arc::new(std::sync::Mutex::new(
            forward,
        ))));
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
        let render_result = render_result_to_webview(&result);
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

    /// Fetch, evaluate, install, and activate a Service Worker through the
    /// in-process [`ServiceWorkerManager`].
    ///
    /// `document_url` supplies the registration client's security context.
    /// Script and scope URLs are resolved against it and must remain same-origin.
    pub fn register_service_worker_runtime(
        &mut self,
        script_url: &str,
        scope: Option<&str>,
        document_url: &str,
    ) -> Result<u64, WebViewError> {
        let (script_url, scope, origin) = Self::validate_service_worker_registration(script_url, scope, document_url)?;
        let script = match &self.script_source_fetcher {
            Some(fetcher) => fetcher(document_url, script_url.as_str())
                .map_err(|error| WebViewError::Script(format!("Service Worker fetch failed: {error}")))?,
            None => self.fetch_text_at(script_url.as_str())?,
        };
        self.sw_manager
            .lock()
            .map_err(|_| WebViewError::Script("Service Worker manager lock poisoned".into()))?
            .start_evaluation(
                script_url.as_str(),
                scope.as_str(),
                &origin,
                &script,
                SandboxConfig::default(),
            )
            .map_err(|error| WebViewError::Script(error.to_string()))
    }

    /// Drain Service Worker manager events and advance lifecycle state.
    pub fn poll_service_worker_runtime_events(&mut self) -> Vec<ServiceWorkerManagerEvent> {
        self.sw_manager
            .lock()
            .map(|mut manager| manager.poll())
            .unwrap_or_default()
    }

    /// Inspect one real Service Worker registration version.
    pub fn service_worker_runtime_registration(
        &self,
        registration_id: u64,
    ) -> Option<zero_storage::ServiceWorkerRegistration> {
        self.sw_manager
            .lock()
            .ok()
            .and_then(|manager| manager.registration(registration_id).cloned())
    }

    /// Inspect real version slots for an origin/scope key.
    pub fn service_worker_runtime_slots(
        &self,
        key: &ServiceWorkerRegistrationKey,
    ) -> Option<ServiceWorkerVersionSlots> {
        self.sw_manager.lock().ok().and_then(|manager| manager.slots(key))
    }

    /// Activate a waiting real Service Worker replacement.
    pub fn activate_waiting_service_worker_runtime(&mut self, registration_id: u64) -> Result<(), WebViewError> {
        self.sw_manager
            .lock()
            .map_err(|_| WebViewError::Script("Service Worker manager lock poisoned".into()))?
            .activate_waiting(registration_id)
            .map_err(|error| WebViewError::Script(error.to_string()))
    }

    fn validate_service_worker_registration(
        script_url: &str,
        scope: Option<&str>,
        document_url: &str,
    ) -> Result<(url::Url, url::Url, String), WebViewError> {
        let document = url::Url::parse(document_url)
            .map_err(|error| WebViewError::Script(format!("invalid document URL: {error}")))?;
        let secure = document.scheme() == "https"
            || (document.scheme() == "http"
                && document.host_str().is_some_and(|host| {
                    host == "localhost" || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
                }));
        if !secure {
            return Err(WebViewError::Script(
                "Service Worker registration requires a secure context".into(),
            ));
        }

        let mut script = document
            .join(script_url)
            .map_err(|error| WebViewError::Script(format!("invalid Service Worker script URL: {error}")))?;
        if !matches!(script.scheme(), "http" | "https") {
            return Err(WebViewError::Script(
                "Service Worker script URL must use http or https".into(),
            ));
        }
        if script.origin() != document.origin() {
            return Err(WebViewError::Script(
                "Service Worker script URL must be same-origin".into(),
            ));
        }
        if script.fragment().is_some() {
            return Err(WebViewError::Script(
                "Service Worker script URL must not contain a fragment".into(),
            ));
        }
        script.set_fragment(None);

        let scope = match scope {
            Some(value) => document
                .join(value)
                .map_err(|error| WebViewError::Script(format!("invalid Service Worker scope: {error}")))?,
            None => script
                .join("./")
                .map_err(|error| WebViewError::Script(format!("invalid default Service Worker scope: {error}")))?,
        };
        if !matches!(scope.scheme(), "http" | "https") || scope.origin() != document.origin() {
            return Err(WebViewError::Script(
                "Service Worker scope must be same-origin http(s)".into(),
            ));
        }
        if scope.fragment().is_some() {
            return Err(WebViewError::Script(
                "Service Worker scope must not contain a fragment".into(),
            ));
        }

        Ok((script, scope, document.origin().ascii_serialization()))
    }

    fn register_service_worker_host_callbacks(
        sandbox: &mut dyn zero_script_sandbox::Sandbox,
        manager: std::sync::Arc<std::sync::Mutex<ServiceWorkerManager>>,
        source_fetcher: Option<ScriptSourceFetcher>,
        page_url: std::sync::Arc<std::sync::Mutex<String>>,
        timeout_secs: u64,
    ) {
        let register_manager = manager.clone();
        sandbox.register_callback(
            "__zw_sw_register",
            Box::new(move |args| {
                let script_input = args.first().map(String::as_str).unwrap_or("");
                let scope_input = args.get(1).map(String::as_str).filter(|value| !value.is_empty());
                let result = (|| -> Result<u64, String> {
                    let document_url = page_url
                        .lock()
                        .map_err(|_| "page URL lock poisoned".to_string())?
                        .clone();
                    let (script_url, scope, origin) =
                        Self::validate_service_worker_registration(script_input, scope_input, &document_url)
                            .map_err(|error| error.to_string())?;
                    let source = match &source_fetcher {
                        Some(fetcher) => fetcher(&document_url, script_url.as_str())?,
                        None => Self::fetch_service_worker_script(script_url.as_str(), timeout_secs)?,
                    };
                    let mut manager = register_manager
                        .lock()
                        .map_err(|_| "Service Worker manager lock poisoned".to_string())?;
                    let registration_id = manager
                        .start_evaluation(
                            script_url.as_str(),
                            scope.as_str(),
                            &origin,
                            &source,
                            SandboxConfig::default(),
                        )
                        .map_err(|error| error.to_string())?;
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                    loop {
                        let events = manager.poll();
                        if events.iter().any(|event| {
                            matches!(
                                event,
                                ServiceWorkerManagerEvent::ScriptEvaluated {
                                    registration_id: id
                                } if *id == registration_id
                            )
                        }) {
                            return Ok(registration_id);
                        }
                        if let Some(error) = events.iter().find_map(|event| match event {
                            ServiceWorkerManagerEvent::ScriptFailed {
                                registration_id: id,
                                message,
                                ..
                            } if *id == registration_id => Some(message.clone()),
                            ServiceWorkerManagerEvent::CoordinationFailed {
                                registration_id: id,
                                message,
                            } if *id == registration_id => Some(message.clone()),
                            _ => None,
                        }) {
                            return Err(error);
                        }
                        if std::time::Instant::now() >= deadline {
                            return Err("Service Worker script evaluation timed out".into());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                })();
                match result {
                    Ok(id) => serde_json::json!({"ok": true, "id": id}).to_string(),
                    Err(error) => serde_json::json!({"ok": false, "error": error}).to_string(),
                }
            }),
        );

        let snapshot_manager = manager.clone();
        sandbox.register_callback(
            "__zw_sw_snapshot",
            Box::new(move |args| {
                let Some(registration_id) = args.first().and_then(|value| value.parse::<u64>().ok()) else {
                    return serde_json::json!({"ok": false, "error": "invalid registration id"}).to_string();
                };
                let Ok(mut manager) = snapshot_manager.lock() else {
                    return serde_json::json!({"ok": false, "error": "manager lock poisoned"}).to_string();
                };
                let _ = manager.poll();
                let Some(registration) = manager.registration(registration_id) else {
                    return serde_json::json!({"ok": false, "error": "registration not found"}).to_string();
                };
                serde_json::json!({
                    "ok": true,
                    "id": registration.id,
                    "scriptURL": registration.script_url,
                    "scope": registration.scope,
                    "state": registration.state.to_string(),
                })
                .to_string()
            }),
        );

        let state_changes_manager = manager.clone();
        sandbox.register_callback(
            "__zw_sw_state_changes",
            Box::new(move |args| {
                let Some(registration_id) = args.first().and_then(|value| value.parse::<u64>().ok()) else {
                    return serde_json::json!({"ok": false, "error": "invalid registration id"}).to_string();
                };
                let after_sequence = args.get(1).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
                let Ok(mut manager) = state_changes_manager.lock() else {
                    return serde_json::json!({"ok": false, "error": "manager lock poisoned"}).to_string();
                };
                let _ = manager.poll();
                let Some((latest_sequence, states)) = manager.state_changes_since(registration_id, after_sequence)
                else {
                    return serde_json::json!({"ok": false, "error": "registration not found"}).to_string();
                };
                serde_json::json!({
                    "ok": true,
                    "latestSequence": latest_sequence,
                    "states": states.iter().map(ToString::to_string).collect::<Vec<_>>(),
                })
                .to_string()
            }),
        );

        sandbox.register_callback(
            "__zw_sw_unregister",
            Box::new(move |args| {
                let removed = args
                    .first()
                    .and_then(|value| value.parse::<u64>().ok())
                    .and_then(|registration_id| {
                        manager
                            .lock()
                            .ok()
                            .map(|mut manager| manager.unregister(registration_id))
                    })
                    .unwrap_or(false);
                if removed { "true" } else { "false" }.to_string()
            }),
        );
    }

    fn fetch_service_worker_script(url: &str, timeout_secs: u64) -> Result<String, String> {
        let response = ResourceLoader::shared()
            .submit(
                ResourceRequest::get(url, FetchPriority::HIGH)
                    .with_destination("serviceworker")
                    .with_timeout_secs(timeout_secs),
            )
            .recv()
            .map_err(|_| "resource loader worker exited".to_string())?
            .map_err(|error| error.to_string())?;
        response.text().map_err(|error| error.to_string())
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

        // R3352：用实例的**真实内存字节数**计算 JS 侧 ArrayBuffer 大小。旧实现从 256 字节探测
        //（`read_memory(.., 256)` 上限）推导页数——探测长度恒 ≤256 → `256/65536+1 = 1` 页，
        // 致**任何**带 memory 导出的模块 JS memory.buffer 恒为 65536（1 页），与真实多页内存不符：
        // JS 写偏移 >65536 越界，`grow()` 返回值基线也错。`memory_size` 直接问实例取字节大小。
        let memory_len = if export_names.iter().any(|n| n == "memory") {
            instance.memory_size("memory").filter(|&sz| sz > 0).unwrap_or(65536)
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
            ..Default::default()
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

    /// 清空 HTTP 响应缓存（共享缓存，S6）。
    pub fn clear_http_cache(&mut self) {
        zero_net::shared_http_cache().lock().unwrap().clear();
    }

    /// 返回 HTTP 缓存条目数（共享缓存，S6）。
    pub fn http_cache_len(&self) -> usize {
        zero_net::shared_http_cache().lock().unwrap().len()
    }

    /// 返回 HTTP 缓存总字节数。
    pub fn http_cache_bytes(&self) -> usize {
        zero_net::shared_http_cache().lock().unwrap().total_bytes()
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

/// 递归抓取 module graph（transitive deps），供进程内 `__zw_compile_module`（R3094）闭合 module graph。
/// registry 按**原始 spec**（importer 引用时写的，匹配 transform_import 按原 spec 查 registry）注册源；
/// 循环防护按**解析后 URL**（同一 module 经不同 raw spec 引用视为同一，避免死循环）。
/// fetcher 解析 spec 相对 parent_url。**已知限制**：钻石依赖（同 module 经不同 raw spec 引用）defer
///（仅注册首个 raw spec，第二引用 lookup 落空 → compile 失败；罕见，多数 module graph 为树）。
fn collect_module_deps_recursive(
    fetcher: &ScriptSourceFetcher,
    parent_url: &str,
    spec: &str,
    registry: &mut zero_script_sandbox::ModuleRegistry,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let url = resolve_document_url(parent_url, spec);
    if visited.contains(&url) {
        return Ok(()); // 循环防护（解析后 URL）
    }
    visited.insert(url.clone());
    let source = fetcher(parent_url, spec)?;
    if source.is_empty() {
        return Err(format!("empty source for {spec}"));
    }
    registry.register(spec, &source); // key = raw spec（transform_import 按原 spec 查）
    for imp in zero_script_sandbox::extract_static_module_import_specifiers(&source) {
        collect_module_deps_recursive(fetcher, &url, &imp, registry, visited)?;
    }
    Ok(())
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

// R3334：native_dom=true 的 WebView Drop 时清空 engine dom_bindings 的线程局部状态。
//
// V8 Isolate 随 WebView 销毁，但线程局部（gc.rs：DOM 源 / ObjectTemplate 缓存 / 对象身份映射）
// 仍持指向**已销毁 Isolate** 的 `v8::Global`/`v8::Weak` Handle；同线程建第二个 native WebView 时
// getter 经线程局部读到旧 Handle → `v8::handle.rs "Handle hosted by disposed Isolate"` panic
//（R3332 实测定位；多标签生产 = 多 WebView 同进程，故 native 默认开前必修）。Drop 时 reset 使下一
// WebView 干净启动。默认关路径（native_dom=false）不触——shim 无 Handle 耦合，零回归。
#[cfg(feature = "v8")]
impl Drop for WebView {
    fn drop(&mut self) {
        if self.config.native_dom {
            zero_engine::dom_bindings::reset_native_state();
            // js-dom R84：v8+quickjs 组合态时 QuickJS reset 并入此处（独立 quickjs Drop
            // impl 会重复定义）。
            #[cfg(feature = "quickjs")]
            zero_engine::quickjs_dom_bindings::reset_quickjs_state();
        }
    }
}

// js-dom goal M6 S0q：QuickJS 版 Drop reset（镜像 V8 侧 R3334 语义）。QuickJS Runtime 随
// WebView 销毁，但 quickjs_dom_bindings 的线程局部（DOM 源 / Persistent 对象身份缓存）仍持
// 指向**已销毁 Runtime** 的 Persistent 值；同线程建第二个 native WebView 时 restore 读旧值
// → UnrelatedRuntime/悬垂。Drop 时清空使下一 WebView 干净启动。默认关路径不触（零回归）。
// js-dom R84：v8+quickjs 组合态 Drop 重复实现（E0119）——并入 v8 版 Drop（两 reset 都调，
// feature 各自 gate），quickjs-only 构建保留独立 Drop。
#[cfg(all(feature = "quickjs", not(feature = "v8")))]
impl Drop for WebView {
    fn drop(&mut self) {
        if self.config.native_dom {
            zero_engine::quickjs_dom_bindings::reset_quickjs_state();
        }
    }
}
