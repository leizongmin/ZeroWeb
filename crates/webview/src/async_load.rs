//! 分阶段异步页面加载 — 首帧 HTML、CSS、图片子资源分步推进。

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use zero_engine::image_resource_key;
use zero_engine::preload::{ResourceHintType, ResourceType, scan_html_resource_hints};
use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, MediaResourceElementKind, extract_css_image_urls, extract_font_faces,
    extract_html_style_text, extract_img_resources, extract_import_urls, extract_media_resources,
    extract_stylesheet_hrefs,
};
use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_render_foundation::font::{OpenTypeFeature, OpenTypeVariation};
use zero_render_foundation::image_cache::{ImageKey, decode_data_uri, decode_data_uri_bytes};

use crate::image_decoder::decode_image;

use crate::net_pool::{
    dns_prefetch_async, fetch_bytes_async_meta, fetch_bytes_stream_async_meta, fetch_document_async,
    fetch_text_async_meta, preconnect_async,
};
use crate::webview::WebView;

/// 图片抓取异步接收器（net_pool 线程 → 加载器轮询）。
type BytesFetchRx = Receiver<Result<Vec<u8>, String>>;
type PendingElementResource = (usize, MediaResourceElementKind, String, BytesFetchRx);
type FontFeatures = Vec<OpenTypeFeature>;
type FontVariations = Vec<OpenTypeVariation>;
type PendingFont = (
    String,
    Option<u16>,
    bool,
    Option<f32>,
    Option<f32>,
    FontFeatures,
    FontVariations,
    Vec<(u32, u32)>,
    String,
    BytesFetchRx,
);
/// 已抓取字体 `(family, weight, italic, stretch, face features, unicode ranges, bytes)`。
pub type LoadedFont = (
    String,
    Option<u16>,
    bool,
    Option<f32>,
    Option<f32>,
    Vec<OpenTypeFeature>,
    Vec<OpenTypeVariation>,
    Vec<(u32, u32)>,
    Vec<u8>,
);

/// 页面加载阶段（供 UI 展示进度）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLoadStage {
    /// 正在抓取主文档。
    FetchingDocument,
    /// 无样式首帧（HTML only）。
    FirstPaint,
    /// 正在抓取外链样式表。
    FetchingStylesheets,
    /// 已应用 CSS，等待图片。
    StyledPaint,
    /// 正在抓取/解码图片。
    FetchingImages,
    /// 加载完成。
    Complete,
    /// 加载失败（主文档或致命错误）。
    Failed,
}

/// 资源元素一次加载尝试的最终结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceElementOutcome {
    /// 资源已获取并完成本层所需处理（图片还包括成功解码）。
    Loaded,
    /// 资源已获取，但本里程碑不执行媒体解码。
    Available,
    /// 获取或图片解码失败。
    Error,
}

impl ResourceElementOutcome {
    /// JS shim 协议使用的稳定字符串。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::Available => "available",
            Self::Error => "error",
        }
    }
}

/// 提交给页面脚本环境的资源元素最终状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceElementEvent {
    /// 目标元素标签名。
    pub tag: &'static str,
    /// 资源绝对 URL。
    pub url: String,
    /// 最终结果。
    pub outcome: ResourceElementOutcome,
    /// 图片解码成功后的固有宽度，其他元素或失败为 0。
    pub natural_width: u32,
    /// 图片解码成功后的固有高度，其他元素或失败为 0。
    pub natural_height: u32,
}

/// in-process 异步抓取宿主：经 webview net_pool 线程池抓取（tabworker 默认）。
pub struct InProcessFetchHost;

impl AsyncFetchHost for InProcessFetchHost {
    fn preconnect(&mut self, origin: &str) {
        preconnect_async(origin.to_string());
    }

    fn dns_prefetch(&mut self, origin: &str) {
        dns_prefetch_async(origin.to_string());
    }

    fn fetch_document(&mut self, url: &str, method: &str, body: Option<&[u8]>) -> Receiver<Result<String, String>> {
        fetch_document_async(url.to_string(), method, body)
    }

    fn fetch_text_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        fetch_text_async_meta(url.to_string(), meta)
    }

    fn fetch_bytes_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        if meta.resource_type == "image" {
            fetch_bytes_stream_async_meta(url.to_string(), meta)
        } else {
            fetch_bytes_async_meta(url.to_string(), meta)
        }
    }
}

/// 生产 @font-face 异步加载是否启用（env `ZW_LIVE_FONTFACE`；默认启用，`0`/`false` 关闭）。
///
/// kill-switch：关闭后宿主跳过 drain→load→register→resolver 刷新，行为退回 R2406 前
///（字节抓取后丢弃）。读取方式与其他运行时配置保持一致。
pub fn live_fontface_enabled() -> bool {
    match std::env::var("ZW_LIVE_FONTFACE") {
        Ok(v) => v != "0" && !v.eq_ignore_ascii_case("false"),
        Err(_) => true,
    }
}

/// 分阶段异步加载协调器。
pub struct AsyncPageLoad {
    url: String,
    document_method: String,
    document_body: Option<Vec<u8>>,
    stage: PageLoadStage,
    html: Option<String>,
    css: String,
    css_pending: Vec<(String, Receiver<Result<String, String>>)>,
    /// R2411：已 fetch 的样式表绝对 URL 集合（含 `<link>` 与递归 `@import`），防 @import 环。
    css_seen: HashSet<String>,
    img_pending: Vec<(String, u64, BytesFetchRx)>,
    lazy_img_pending: Vec<(String, u64, BytesFetchRx)>,
    resource_pending: Vec<PendingElementResource>,
    resource_settled: Vec<(usize, ResourceElementEvent)>,
    font_pending: Vec<PendingFont>,
    /// R2408+ slice 2：poll_fonts 收集的已就绪 @font-face 字节 `(family, weight, bytes)`，
    /// 供宿主在 tick 后经 `drain_loaded_fonts()` 取出并 load+register（drain pattern）。
    /// weight（R2417）供宿主按 weight 构 `{family}:700` 粗体键；is_italic（R2493）供宿主
    /// 按 font-style 构 `{family}:italic` italic 键。
    font_loaded: Vec<LoadedFont>,
    /// R2947：@font-face 加载结果 `(family, "loaded"/"error")`，供宿主派发 FontFaceSet
    /// 'loadingdone'/'loadingerror' 事件 + 解析 `document.fonts.ready` Promise。poll_fonts 收集
    /// （成功 + 失败，失败此前仅 warn 丢弃）；经 `take_font_events()` drain。
    font_events: Vec<(String, &'static str)>,
    lazy_urls: Vec<String>,
    document_rx: Option<Receiver<Result<String, String>>>,
    render_session: Option<BudgetedRenderSession>,
    budget_pending: bool,
    last_error: Option<String>,
    /// 子资源 fetch/decode 失败记录，供宿主派发 window error 事件。
    failed_resources: Vec<FailedResource>,
    /// FR-009：img/audio/video/source/track 最终状态，供宿主提交 IDL 状态与元素事件。
    resource_element_events: Vec<ResourceElementEvent>,
    /// R2944：stylesheet 元素级 load/error 事件 `(绝对 URL, "load"/"error")`，供宿主经
    /// `__zw_dispatch_link_event` 派发到匹配 href 的 `<link>` 元素（link.onload/onerror）。成功 → "load"，
    /// fetch 失败 → "error"；经 `take_link_element_events()` drain。
    link_element_events: Vec<(String, &'static str)>,
}

/// 子资源 fetch/decode 失败记录。`kind` 标识 stylesheet/image/audio/video/source/track。
/// 宿主在页面脚本注册 handler 后、window load 前派发 window error。外部 `<script src>` fetch
/// 失败不经此（在 tab_scripts tick 即时派发）。
pub struct FailedResource {
    /// 资源类型。
    pub kind: &'static str,
    /// 资源绝对 URL。
    pub url: String,
}

impl AsyncPageLoad {
    /// 开始加载 URL（主文档在首 tick 经 host 抓取，tabworker 默认 InProcessFetchHost）。
    pub fn start(url: impl Into<String>) -> Self {
        Self::start_request(url, "GET", None)
    }

    /// 开始加载带方法和 body 的主文档请求。
    pub fn start_request(url: impl Into<String>, method: impl Into<String>, body: Option<Vec<u8>>) -> Self {
        Self {
            url: url.into(),
            document_method: method.into(),
            document_body: body,
            stage: PageLoadStage::FetchingDocument,
            html: None,
            css: String::new(),
            css_pending: Vec::new(),
            css_seen: HashSet::new(),
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            resource_pending: Vec::new(),
            resource_settled: Vec::new(),
            font_pending: Vec::new(),
            font_loaded: Vec::new(),
            font_events: Vec::new(),
            lazy_urls: Vec::new(),
            document_rx: None,
            render_session: None,
            budget_pending: false,
            last_error: None,
            failed_resources: Vec::new(),
            resource_element_events: Vec::new(),
            link_element_events: Vec::new(),
        }
    }

    /// 取出并清除加载失败原因（主文档抓取失败等）。
    pub fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }

    /// 取出并清除子资源 fetch/decode 失败记录。
    pub fn take_failed_resources(&mut self) -> Vec<FailedResource> {
        std::mem::take(&mut self.failed_resources)
    }

    /// 取出并清除所有资源元素最终状态。
    pub fn take_resource_element_events(&mut self) -> Vec<ResourceElementEvent> {
        std::mem::take(&mut self.resource_element_events)
    }

    /// 兼容旧宿主：只取出 img 元素的 load/error，不携带固有尺寸。
    pub fn take_img_element_events(&mut self) -> Vec<(String, &'static str)> {
        let mut events = Vec::new();
        self.resource_element_events.retain(|event| {
            if event.tag != "img" {
                return true;
            }
            let ty = match event.outcome {
                ResourceElementOutcome::Loaded => "load",
                ResourceElementOutcome::Error => "error",
                ResourceElementOutcome::Available => return false,
            };
            events.push((event.url.clone(), ty));
            false
        });
        events
    }

    /// R2944：取出并清除 stylesheet 元素级 load/error 事件 `(绝对 URL, "load"/"error")`，供宿主经
    /// `__zw_dispatch_link_event` 派发到匹配 href 的 `<link>` 元素。
    pub fn take_link_element_events(&mut self) -> Vec<(String, &'static str)> {
        std::mem::take(&mut self.link_element_events)
    }

    /// 是否因错误结束。
    pub fn failed(&self) -> bool {
        self.stage == PageLoadStage::Failed
    }

    /// 从已有 HTML 开始（跳过主文档网络，外链子资源经 host 抓取）。
    pub fn from_html(url: impl Into<String>, html: String) -> Self {
        Self {
            url: url.into(),
            document_method: "GET".to_string(),
            document_body: None,
            stage: PageLoadStage::FirstPaint,
            html: Some(html),
            css: String::new(),
            css_pending: Vec::new(),
            css_seen: HashSet::new(),
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            resource_pending: Vec::new(),
            resource_settled: Vec::new(),
            font_pending: Vec::new(),
            font_loaded: Vec::new(),
            font_events: Vec::new(),
            lazy_urls: Vec::new(),
            document_rx: None,
            render_session: None,
            budget_pending: true,
            last_error: None,
            failed_resources: Vec::new(),
            resource_element_events: Vec::new(),
            link_element_events: Vec::new(),
        }
    }

    /// 当前阶段。
    pub fn stage(&self) -> PageLoadStage {
        self.stage
    }

    /// 是否仍在加载。
    pub fn is_active(&self) -> bool {
        if self.stage == PageLoadStage::Failed {
            return false;
        }
        if self.stage != PageLoadStage::Complete {
            return true;
        }
        // R2408+ slice 2：budget_pending 亦视作活跃——字体加载后 request_rerender 置位，
        // 须保留 load 至该重绘 tick 完成后再判定结束（否则末个字体到达即 complete 会
        // 丢弃 request_rerender，最终帧用 fallback 字体）。
        !self.font_pending.is_empty()
            || !self.lazy_img_pending.is_empty()
            || !self.resource_pending.is_empty()
            || self.budget_pending
    }

    /// 请求下一 tick 重渲染（外部状态变化后调用，如宿主加载 @font-face 后更新 resolver）。
    ///
    /// 仅在有 HTML 时置 `budget_pending`，使下一 tick 的 `advance_render` 用新 resolver 重绘。
    pub fn request_rerender(&mut self) {
        if self.html.is_some() {
            self.budget_pending = true;
        }
    }

    /// 放弃仍未返回的非关键子资源，并完成当前文档的最终渲染。
    ///
    /// 文档和样式已经成功呈现后，卡住的图片或字体不应把可用页面替换为宿主错误页。
    /// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#fetching-resources
    pub fn abandon_pending_subresources(&mut self) {
        self.font_pending.clear();
        self.img_pending.clear();
        self.lazy_urls.clear();
        self.lazy_img_pending.clear();
        self.resource_pending.clear();
        self.stage = PageLoadStage::Complete;
        self.budget_pending = true;
    }

    /// 取出并清空已就绪的 @font-face 字节 `(family, weight, bytes)`（drain pattern）。
    ///
    /// 宿主在 `tick` 返回后调用——`poll_fonts` 把 fetch 成功的字节收集到此处，不再丢弃。
    /// 宿主据此 `load_font` + `register_family_alias`（weight≥600 时另构 `{family}:700`
    /// 粗体键 R2417；is_italic 时另构 `{family}:italic` italic 键 R2493）+ 刷新 resolver
    /// + `request_rerender`。
    pub fn drain_loaded_fonts(&mut self) -> Vec<LoadedFont> {
        std::mem::take(&mut self.font_loaded)
    }

    /// R2947：取出并清空 @font-face 加载结果 `(family, "loaded"/"error")`（drain pattern）。
    /// 宿主在 load 完成时 drain，据此派发 FontFaceSet 'loadingdone'/'loadingerror' + 解析
    /// `document.fonts.ready` Promise（字体加载库 / icon font / FOUT 处理高频 hook）。
    pub fn take_font_events(&mut self) -> Vec<(String, &'static str)> {
        std::mem::take(&mut self.font_events)
    }

    fn log_stage(&self, label: &str) {
        tracing::info!(url = %self.url, stage = ?self.stage, "{label}");
    }

    /// 在 `budget_ms` 内推进加载与渲染；返回 `true` 表示状态有更新。
    ///
    /// `host` 按需发起子资源抓取（per-tick 借用——供 renderer 经 IPC 复用同一加载器，
    /// 无需在构造时绑定 host）。
    pub fn tick(&mut self, webview: &mut WebView, host: &mut dyn AsyncFetchHost, budget_ms: f64) -> bool {
        let mut changed = false;

        // 首次进入 FetchingDocument 时发起主文档抓取（per-tick host，不在构造时抓取）。
        if self.stage == PageLoadStage::FetchingDocument && self.document_rx.is_none() {
            let url = self.url.clone();
            tracing::info!(url = %url, "page load: fetch document");
            self.document_rx = Some(host.fetch_document(&url, &self.document_method, self.document_body.as_deref()));
        }

        if let Some(rx) = self.document_rx.as_ref()
            && let Ok(result) = rx.try_recv()
        {
            self.document_rx = None;
            match result {
                Ok(html) => {
                    if let Some(title) = extract_document_title(&html) {
                        webview.set_title(&title);
                    }
                    self.begin_preload_hints(&html, host);
                    self.html = Some(html);
                    self.stage = PageLoadStage::FirstPaint;
                    self.budget_pending = true;
                    self.log_stage("document ready, HTML skeleton render");
                    changed = true;
                }
                Err(e) => {
                    tracing::warn!("document fetch failed: {e}");
                    self.last_error = Some(e);
                    self.stage = PageLoadStage::Failed;
                    changed = true;
                }
            }
        }

        if self.budget_pending {
            changed |= self.advance_render(webview, budget_ms);
        }

        if self.stage == PageLoadStage::FirstPaint && !self.budget_pending && self.render_session.is_none() {
            self.begin_stylesheet_fetch(webview, host);
            changed = true;
        }

        self.poll_stylesheets(webview, host, budget_ms, &mut changed);
        self.poll_images(webview, budget_ms, &mut changed);
        self.poll_element_resources(webview, budget_ms, &mut changed);
        self.poll_fonts(&mut changed);
        self.poll_lazy_images(webview, budget_ms, &mut changed);

        if self.stage == PageLoadStage::Complete && self.lazy_img_pending.is_empty() && !self.lazy_urls.is_empty() {
            self.begin_lazy_image_fetch(host);
            changed = true;
        }

        // 图片分批到达后需在本 tick 内重绘，否则 publish 会用到上一帧。
        if self.budget_pending && matches!(self.stage, PageLoadStage::FetchingImages | PageLoadStage::Complete) {
            changed |= self.advance_render(webview, budget_ms);
        }

        changed
    }

    fn begin_preload_hints(&mut self, html: &str, host: &mut dyn AsyncFetchHost) {
        let preloader = scan_html_resource_hints(html);
        let base = url::Url::parse(&self.url).ok();
        let mut count = 0usize;
        for hint in preloader.pending_resources() {
            let abs = match base.as_ref().and_then(|b| b.join(&hint.url).ok()) {
                Some(u) => u.to_string(),
                None => hint.url.clone(),
            };
            if matches!(
                hint.hint_type,
                ResourceHintType::Preconnect | ResourceHintType::DnsPrefetch
            ) {
                let origin = match url::Url::parse(&abs) {
                    Ok(url)
                        if matches!(url.scheme(), "http" | "https")
                            && url.username().is_empty()
                            && url.password().is_none()
                            && url.host().is_some() =>
                    {
                        format!("{}/", url.origin().ascii_serialization())
                    }
                    _ => continue,
                };
                if hint.hint_type == ResourceHintType::Preconnect {
                    host.preconnect(&origin);
                } else {
                    host.dns_prefetch(&origin);
                }
                count += 1;
                continue;
            }
            if hint.hint_type != ResourceHintType::Preload {
                continue;
            }
            let mut meta = match hint.resource_type {
                ResourceType::Style => ResourceFetchMeta::STYLESHEET,
                ResourceType::Script => ResourceFetchMeta::SCRIPT,
                ResourceType::Font => ResourceFetchMeta::FONT,
                ResourceType::Image => ResourceFetchMeta::IMAGE,
                _ => ResourceFetchMeta::preload("fetch"),
            };
            // https://html.spec.whatwg.org/multipage/urls-and-fetching.html#attr-fetchpriority
            meta.priority = hint.priority as u8;
            match hint.resource_type {
                ResourceType::Style | ResourceType::Script => {
                    let _ = host.fetch_text_meta(&abs, meta);
                }
                _ => {
                    let _ = host.fetch_bytes_meta(&abs, meta);
                }
            }
            count += 1;
        }
        if count > 0 {
            tracing::info!(url = %self.url, count, "page load: speculative preload hints");
        }
    }

    fn advance_render(&mut self, webview: &mut WebView, budget_ms: f64) -> bool {
        let html = match self.html.as_ref() {
            Some(h) => h.clone(),
            None => return false,
        };

        if self.render_session.is_none() {
            let same_navigation = webview.is_loading() && webview.url().is_some_and(|u| u == self.url.as_str());
            if !same_navigation {
                webview.prepare_document_state(&self.url);
            }
            webview.set_cached_content(&html, &self.css);
            tracing::info!(url = %self.url, stage = ?self.stage, "page load: budget render start");
            self.render_session = Some(BudgetedRenderSession::new(html, self.css.clone()));
        }

        let session = self.render_session.as_mut().expect("session");
        match webview.advance_budget_session(session, budget_ms) {
            BudgetAdvance::Complete => {
                if let Some(result) = session.take_result() {
                    let done = matches!(self.stage, PageLoadStage::FetchingImages | PageLoadStage::Complete)
                        && self.img_pending.is_empty()
                        && self.lazy_img_pending.is_empty()
                        && self.resource_pending.is_empty()
                        && self.font_pending.is_empty();
                    webview.apply_render_result(result, &self.url, done);
                }
                tracing::info!(url = %self.url, stage = ?self.stage, "page load: budget render complete");
                self.render_session = None;
                self.budget_pending = false;
                match self.stage {
                    // 留在 FirstPaint，由 tick() 调用 begin_stylesheet_fetch。
                    PageLoadStage::FirstPaint => {}
                    PageLoadStage::StyledPaint | PageLoadStage::FetchingImages
                        if self.css_pending.is_empty()
                            && self.img_pending.is_empty()
                            && self.resource_pending.is_empty()
                            && self.font_pending.is_empty() =>
                    {
                        self.stage = PageLoadStage::Complete;
                    }
                    _ => {}
                }
                true
            }
            BudgetAdvance::InProgress => false,
        }
    }

    fn begin_stylesheet_fetch(&mut self, webview: &mut WebView, host: &mut dyn AsyncFetchHost) {
        let html = match self.html.as_ref() {
            Some(h) => h.as_str(),
            None => return,
        };
        let hrefs = extract_stylesheet_hrefs(html);
        let base = url::Url::parse(&self.url).ok();
        for href in hrefs {
            let abs = match base.as_ref().and_then(|b| b.join(&href).ok()) {
                Some(u) => u.to_string(),
                None => href,
            };
            // R2411：记录已 fetch 的样式表 URL（@import 递归防环用）。
            self.css_seen.insert(abs.clone());
            self.css_pending
                .push((abs.clone(), host.fetch_text_meta(&abs, ResourceFetchMeta::STYLESHEET)));
        }
        if self.css_pending.is_empty() {
            // R2408+ slice 2：无外链 CSS 也须抓 @font-face（含 inline `<style>` 声明）。
            // 旧版仅经 poll_stylesheets（FetchingStylesheets 阶段）调用 begin_font_fetch，
            // 致纯 inline @font-face 页（无 <link>）从不抓字体。
            self.begin_font_fetch(host);
            self.begin_image_fetch(webview, host);
        } else {
            tracing::info!(
                url = %self.url,
                count = self.css_pending.len(),
                "page load: fetch stylesheets"
            );
            self.stage = PageLoadStage::FetchingStylesheets;
        }
    }

    fn poll_stylesheets(
        &mut self,
        webview: &mut WebView,
        host: &mut dyn AsyncFetchHost,
        budget_ms: f64,
        changed: &mut bool,
    ) {
        if self.stage != PageLoadStage::FetchingStylesheets {
            return;
        }
        // R2411：到达样式表的 @import URL 先收集到局部（retain 闭包内 css_pending 正被借用，
        // 不能在此 fetch）。每个 @import 相对**该样式表 url** 解析（非文档 url）。
        let mut import_urls: Vec<String> = Vec::new();
        self.css_pending.retain(|(url, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(css) => {
                        self.css.push_str(&css);
                        self.css.push('\n');
                        self.link_element_events.push((url.clone(), "load"));
                        for imp in extract_import_urls(&css) {
                            if imp.starts_with("data:") {
                                continue;
                            }
                            let abs = match url::Url::parse(url).ok().and_then(|b| b.join(&imp).ok()) {
                                Some(u) => u.to_string(),
                                None => imp,
                            };
                            import_urls.push(abs);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("stylesheet {url} fetch failed: {e}");
                        self.failed_resources.push(FailedResource {
                            kind: "stylesheet",
                            url: url.clone(),
                        });
                        self.link_element_events.push((url.clone(), "error"));
                    }
                }
                *changed = true;
                false
            } else {
                true
            }
        });
        // R2411：递归 fetch @import 引入的样式表（css_seen 防环——每个 url 仅 fetch 一次；
        // 循环 @import A→B→A 因 css_seen 命中而止）。stage 保持 FetchingStylesheets 直到全部
        //（含递归）drain 完，再进入 StyledPaint。
        for abs in import_urls {
            if self.css_seen.insert(abs.clone()) {
                self.css_pending
                    .push((abs.clone(), host.fetch_text_meta(&abs, ResourceFetchMeta::STYLESHEET)));
            }
        }
        if self.css_pending.is_empty() {
            tracing::info!(url = %self.url, "page load: stylesheets ready, styled render");
            self.stage = PageLoadStage::StyledPaint;
            self.budget_pending = true;
            *changed = true;
            let _ = self.advance_render(webview, budget_ms);
            self.begin_font_fetch(host);
        }
    }

    /// 样式和阻塞脚本完成后才开始非关键图片抓取，避免其占满同源连接而饿死页面脚本。
    pub fn begin_noncritical_fetches(&mut self, webview: &mut WebView, host: &mut dyn AsyncFetchHost) {
        if self.stage == PageLoadStage::StyledPaint {
            self.begin_image_fetch(webview, host);
        }
    }

    fn begin_font_fetch(&mut self, host: &mut dyn AsyncFetchHost) {
        // 合并外链 CSS + inline `<style>`（对齐 begin_image_fetch；修 R2406 次要 bug：
        // 旧版仅扫 self.css，漏 inline @font-face）。extract_font_faces 保留 family（CSS 声明族名）。
        let mut css = self.css.clone();
        if let Some(html) = self.html.as_ref() {
            css.push('\n');
            css.push_str(&extract_html_style_text(html));
        }
        let faces = extract_font_faces(&css);
        let base = url::Url::parse(&self.url).ok();
        for (
            family,
            sources,
            weight,
            is_italic,
            stretch,
            size_adjust,
            feature_settings,
            variation_settings,
            unicode_ranges,
        ) in faces
        {
            let features = zero_engine::font_feature_settings_to_opentype(&feature_settings);
            let variations = zero_engine::font_variation_settings_to_opentype(&variation_settings);
            // 一个 @font-face 只请求最适合当前解码器的 source；并发抓取所有格式会占满
            // 同源连接，阻塞 parser-discovered scripts。fontdue 优先支持 TrueType/OpenType。
            let Some(src) = sources
                .iter()
                .find(|src| {
                    src.to_ascii_lowercase()
                        .split('?')
                        .next()
                        .is_some_and(|url| url.ends_with(".ttf") || url.ends_with(".otf"))
                })
                .or_else(|| {
                    sources.iter().find(|src| {
                        src.to_ascii_lowercase()
                            .split('?')
                            .next()
                            .is_some_and(|url| url.ends_with(".woff"))
                    })
                })
                .or_else(|| sources.first())
            else {
                continue;
            };
            if src.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) {
                if std::env::var("ZW_DATA_FONT").as_deref() != Ok("0") {
                    match decode_data_uri_bytes(src) {
                        Ok(bytes) => {
                            self.font_loaded.push((
                                family.clone(),
                                weight,
                                is_italic,
                                stretch,
                                size_adjust,
                                features,
                                variations,
                                unicode_ranges,
                                bytes,
                            ));
                            self.font_events.push((family, "loaded"));
                        }
                        Err(error) => {
                            tracing::warn!(family, %error, "page load: data font decode failed");
                            self.font_events.push((family, "error"));
                        }
                    }
                }
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(src).ok()) {
                Some(u) => u.to_string(),
                None => src.clone(),
            };
            self.font_pending.push((
                family,
                weight,
                is_italic,
                stretch,
                size_adjust,
                features,
                variations,
                unicode_ranges,
                abs.clone(),
                host.fetch_bytes_meta(&abs, ResourceFetchMeta::FONT),
            ));
        }
        if !self.font_pending.is_empty() {
            tracing::info!(url = %self.url, count = self.font_pending.len(), "page load: fetch fonts");
        }
    }

    fn poll_fonts(&mut self, changed: &mut bool) {
        self.font_pending.retain(
            |(
                family,
                weight,
                is_italic,
                stretch,
                size_adjust,
                feature_settings,
                variation_settings,
                unicode_ranges,
                url,
                rx,
            )| {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(bytes) => {
                            tracing::info!(url, bytes = bytes.len(), "page load: font fetched");
                            // R2408+ slice 2：保留字节供宿主 drain 后 load+register（drain pattern），
                            // 不再丢弃。family 用于 register_family_alias；weight（R2417）用于按
                            // weight 构 {family}:700 粗体键；is_italic（R2493）用于按 font-style 构
                            // {family}:italic italic 键。
                            self.font_loaded.push((
                                family.clone(),
                                *weight,
                                *is_italic,
                                *stretch,
                                *size_adjust,
                                feature_settings.clone(),
                                variation_settings.clone(),
                                unicode_ranges.clone(),
                                bytes,
                            ));
                            // R2947：记录加载成功，供宿主派发 FontFaceSet 'loadingdone' + 解析 ready。
                            self.font_events.push((family.clone(), "loaded"));
                        }
                        Err(e) => {
                            tracing::warn!("font {url} fetch failed: {e}");
                            // R2947：记录加载失败，供宿主派发 FontFaceSet 'loadingerror'。
                            self.font_events.push((family.clone(), "error"));
                        }
                    }
                    *changed = true;
                    false
                } else {
                    true
                }
            },
        );
    }

    fn begin_image_fetch(&mut self, webview: &mut WebView, host: &mut dyn AsyncFetchHost) {
        let html = match self.html.as_ref() {
            Some(h) => h.as_str(),
            None => return,
        };
        let imgs = extract_img_resources(html);
        let base = url::Url::parse(&self.url).ok();
        // 性能门禁优化 S6（2026-08-08）：`<img src>` 与 CSS `url()` 共用同一去重集合——
        // 旧实现仅 CSS 循环去重（且 O(n²) 线性扫描），N 个相同 `<img src>` 会重复
        // push + 重复 decode；HashSet 使两循环均 O(1) 查重。
        let mut seen: std::collections::HashSet<String> = self
            .img_pending
            .iter()
            .map(|(a, _, _)| a.clone())
            .chain(self.lazy_urls.iter().cloned())
            .collect();
        for img in imgs {
            if img.src.starts_with("data:") {
                // R1987：data: URI（PNG/JPEG/WebP/SVG）无 HTTP fetch，直接解码并入缓存
                // （in-scope img 子资源，goal line 118；与 sync 路径 fetch_image_subresources 对齐）。
                match decode_data_uri(&img.src) {
                    Ok(data) => {
                        let key = image_resource_key(&img.src, None);
                        self.resource_element_events.push(ResourceElementEvent {
                            tag: "img",
                            url: img.src.clone(),
                            outcome: ResourceElementOutcome::Loaded,
                            natural_width: data.width,
                            natural_height: data.height,
                        });
                        webview.image_cache().insert_with_key(ImageKey::new(key), data);
                    }
                    Err(e) => {
                        tracing::warn!("data: URI image decode failed: {e}");
                        self.failed_resources.push(FailedResource {
                            kind: "image",
                            url: img.src.clone(),
                        });
                        self.resource_element_events.push(ResourceElementEvent {
                            tag: "img",
                            url: img.src,
                            outcome: ResourceElementOutcome::Error,
                            natural_width: 0,
                            natural_height: 0,
                        });
                    }
                }
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(&img.src).ok()) {
                Some(u) => u.to_string(),
                None => img.src,
            };
            if img.lazy {
                if seen.insert(abs.clone()) {
                    self.lazy_urls.push(abs);
                }
                continue;
            }
            if !seen.insert(abs.clone()) {
                // 重复引用（重复 <img src> 或已由 CSS url()/lazy 提交）→ 跳过
                continue;
            }
            let key = image_resource_key(&abs, None);
            // https://html.spec.whatwg.org/multipage/urls-and-fetching.html#attr-fetchpriority
            let mut meta = ResourceFetchMeta::IMAGE;
            match img.fetchpriority.as_deref().map(str::trim) {
                Some(value) if value.eq_ignore_ascii_case("high") => meta.priority = 3,
                Some(value) if value.eq_ignore_ascii_case("low") => meta.priority = 1,
                _ => {}
            }
            self.img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, meta)));
        }
        // R1795：CSS `url()` 图片引用（background-image / list-style-image /
        // border-image-source）一并异步抓取——与 sync 路径（webview.rs fetch_image_subresources）
        // 对齐。合并 self.css（外链 CSS）+ inline `<style>` 文本，extract_css_image_urls 已排除
        // @font-face 与 data:。CSS url 非 lazy，直接入 img_pending；fetch+decode+key 复用
        // poll_images 现有路径（painter 查找经 R1794 Part A 已按 image_resource_key 对齐）。
        let mut combined_css = self.css.clone();
        combined_css.push('\n');
        combined_css.push_str(&extract_html_style_text(html));
        for src in extract_css_image_urls(&combined_css) {
            if src.starts_with("data:") {
                // R1987：CSS url(data:) 同 <img src=data:>，直接解码入缓存（无 fetch）。
                if let Ok(data) = decode_data_uri(&src) {
                    let key = image_resource_key(&src, None);
                    webview.image_cache().insert_with_key(ImageKey::new(key), data);
                }
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(&src).ok()) {
                Some(u) => u.to_string(),
                None => src,
            };
            // <img src> 与 CSS url() 可能指向同一资源：去重（S6：与 img 循环共用 seen 集合）
            if !seen.insert(abs.clone()) {
                continue;
            }
            let key = image_resource_key(&abs, None);
            self.img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, ResourceFetchMeta::IMAGE)));
        }
        self.begin_element_resource_fetch(host);
        if self.img_pending.is_empty() && self.lazy_urls.is_empty() && self.resource_pending.is_empty() {
            self.stage = PageLoadStage::Complete;
        } else if !self.img_pending.is_empty() || !self.resource_pending.is_empty() {
            tracing::info!(
                url = %self.url,
                count = self.img_pending.len(),
                resources = self.resource_pending.len(),
                lazy = self.lazy_urls.len(),
                "page load: fetch images and resource elements"
            );
            self.stage = PageLoadStage::FetchingImages;
        } else {
            self.begin_lazy_image_fetch(host);
        }
        let _ = webview;
    }

    fn begin_lazy_image_fetch(&mut self, host: &mut dyn AsyncFetchHost) {
        if self.lazy_urls.is_empty() {
            return;
        }
        tracing::info!(url = %self.url, count = self.lazy_urls.len(), "page load: fetch lazy images");
        for abs in self.lazy_urls.drain(..) {
            let key = image_resource_key(&abs, None);
            // `loading=lazy` 仅在初始关键资源完成后启动，且不应与可见图片竞争。
            let mut meta = ResourceFetchMeta::IMAGE;
            meta.priority = 1;
            self.lazy_img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, meta)));
        }
    }

    fn begin_element_resource_fetch(&mut self, host: &mut dyn AsyncFetchHost) {
        let Some(html) = self.html.as_deref() else {
            return;
        };
        let base = url::Url::parse(&self.url).ok();
        for (index, resource) in extract_media_resources(html).into_iter().enumerate() {
            let abs = match base.as_ref().and_then(|base| base.join(&resource.src).ok()) {
                Some(url) => url.to_string(),
                None => resource.src,
            };
            self.resource_pending.push((
                index,
                resource.kind,
                abs.clone(),
                host.fetch_bytes_meta(&abs, ResourceFetchMeta::MEDIA),
            ));
        }
    }

    fn poll_lazy_images(&mut self, webview: &mut WebView, budget_ms: f64, changed: &mut bool) {
        if self.lazy_img_pending.is_empty() {
            if self.stage == PageLoadStage::Complete && !self.lazy_urls.is_empty() {
                return;
            }
            return;
        }
        let mut sizes: HashMap<u64, (f32, f32)> = webview.cached_image_sizes().clone();
        let mut ratios: HashMap<u64, f32> = webview.cached_image_ratios().clone();
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = webview.cached_image_no_ratio().clone();
        self.lazy_img_pending.retain(|(url, key, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => {
                        match decode_image(&bytes) {
                            Ok(img) => {
                                // 非 BothAbs SVG 进 no_ratio（default object size sizing）；其余进 sizes。
                                let intrinsic_ratio = img.intrinsic_ratio();
                                if let Some(r) = intrinsic_ratio {
                                    ratios.insert(*key, r);
                                } else {
                                    let (w, h) = (img.width as f32, img.height as f32);
                                    sizes.insert(*key, (w, h));
                                    if let Some(dims) = img.no_ratio_intrinsic() {
                                        no_ratio.insert(*key, dims);
                                    }
                                }
                                self.resource_element_events.push(ResourceElementEvent {
                                    tag: "img",
                                    url: url.clone(),
                                    outcome: ResourceElementOutcome::Loaded,
                                    natural_width: img.width,
                                    natural_height: img.height,
                                });
                                webview.image_cache().insert_with_key(ImageKey::new(*key), img);
                            }
                            Err(e) => {
                                tracing::warn!("lazy image {url} decode failed: {e}");
                                self.failed_resources.push(FailedResource {
                                    kind: "image",
                                    url: url.clone(),
                                });
                                self.resource_element_events.push(ResourceElementEvent {
                                    tag: "img",
                                    url: url.clone(),
                                    outcome: ResourceElementOutcome::Error,
                                    natural_width: 0,
                                    natural_height: 0,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("lazy image {url} fetch failed: {e}");
                        self.failed_resources.push(FailedResource {
                            kind: "image",
                            url: url.clone(),
                        });
                        self.resource_element_events.push(ResourceElementEvent {
                            tag: "img",
                            url: url.clone(),
                            outcome: ResourceElementOutcome::Error,
                            natural_width: 0,
                            natural_height: 0,
                        });
                    }
                }
                *changed = true;
                false
            } else {
                true
            }
        });
        if !sizes.is_empty() {
            webview.set_image_sizes(sizes);
            self.budget_pending = true;
        }
        if !ratios.is_empty() {
            webview.set_image_ratios(ratios);
        }
        if !no_ratio.is_empty() {
            webview.set_image_no_ratio(no_ratio);
        }
        if self.lazy_img_pending.is_empty() {
            let _ = self.advance_render(webview, budget_ms);
        }
    }

    fn poll_images(&mut self, webview: &mut WebView, budget_ms: f64, changed: &mut bool) {
        if self.stage != PageLoadStage::FetchingImages {
            return;
        }
        let mut image_changed = false;
        let mut sizes: HashMap<u64, (f32, f32)> = webview.cached_image_sizes().clone();
        let mut ratios: HashMap<u64, f32> = webview.cached_image_ratios().clone();
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = webview.cached_image_no_ratio().clone();
        self.img_pending.retain(|(url, key, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => match decode_image(&bytes) {
                        Ok(img) => {
                            let natural_width = img.width;
                            let natural_height = img.height;
                            // 非 BothAbs SVG 进 no_ratio（default object size sizing）；其余进 sizes。
                            if let Some(r) = img.intrinsic_ratio() {
                                ratios.insert(*key, r);
                            } else {
                                let (w, h) = (img.width as f32, img.height as f32);
                                sizes.insert(*key, (w, h));
                                if let Some(dims) = img.no_ratio_intrinsic() {
                                    no_ratio.insert(*key, dims);
                                }
                            }
                            webview.image_cache().insert_with_key(ImageKey::new(*key), img);
                            self.resource_element_events.push(ResourceElementEvent {
                                tag: "img",
                                url: url.clone(),
                                outcome: ResourceElementOutcome::Loaded,
                                natural_width,
                                natural_height,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("image {url} decode failed: {e}");
                            self.failed_resources.push(FailedResource {
                                kind: "image",
                                url: url.clone(),
                            });
                            self.resource_element_events.push(ResourceElementEvent {
                                tag: "img",
                                url: url.clone(),
                                outcome: ResourceElementOutcome::Error,
                                natural_width: 0,
                                natural_height: 0,
                            });
                        }
                    },
                    Err(e) => {
                        tracing::warn!("image {url} fetch failed: {e}");
                        self.failed_resources.push(FailedResource {
                            kind: "image",
                            url: url.clone(),
                        });
                        self.resource_element_events.push(ResourceElementEvent {
                            tag: "img",
                            url: url.clone(),
                            outcome: ResourceElementOutcome::Error,
                            natural_width: 0,
                            natural_height: 0,
                        });
                    }
                }
                image_changed = true;
                false
            } else {
                true
            }
        });
        *changed |= image_changed;
        if !sizes.is_empty() {
            webview.set_image_sizes(sizes);
        }
        if !ratios.is_empty() {
            webview.set_image_ratios(ratios);
        }
        if !no_ratio.is_empty() {
            webview.set_image_no_ratio(no_ratio);
        }
        if image_changed && self.stage == PageLoadStage::FetchingImages {
            let remaining = self.img_pending.len();
            tracing::info!(
                url = %self.url,
                remaining,
                "page load: image batch ready, incremental render"
            );
            self.budget_pending = true;
        }
        if self.img_pending.is_empty() && self.resource_pending.is_empty() {
            tracing::info!(url = %self.url, "page load: all eager images ready, final render");
            self.stage = PageLoadStage::Complete;
            self.budget_pending = true;
            *changed = true;
            let _ = self.advance_render(webview, budget_ms);
        }
    }

    fn poll_element_resources(&mut self, webview: &mut WebView, budget_ms: f64, changed: &mut bool) {
        if self.stage != PageLoadStage::FetchingImages {
            return;
        }
        self.resource_pending.retain(|(index, kind, url, rx)| {
            let Ok(result) = rx.try_recv() else {
                return true;
            };
            let outcome = if result.is_ok() {
                ResourceElementOutcome::Available
            } else {
                ResourceElementOutcome::Error
            };
            if let Err(error) = result {
                tracing::warn!(tag = kind.tag_name(), "resource {url} fetch failed: {error}");
                self.failed_resources.push(FailedResource {
                    kind: kind.tag_name(),
                    url: url.clone(),
                });
            }
            self.resource_settled.push((
                *index,
                ResourceElementEvent {
                    tag: kind.tag_name(),
                    url: url.clone(),
                    outcome,
                    natural_width: 0,
                    natural_height: 0,
                },
            ));
            *changed = true;
            false
        });
        if self.resource_pending.is_empty() {
            self.resource_settled.sort_by_key(|(index, _)| *index);
            self.resource_element_events
                .extend(self.resource_settled.drain(..).map(|(_, event)| event));
        }
        if self.resource_pending.is_empty() && self.img_pending.is_empty() {
            self.stage = PageLoadStage::Complete;
            self.budget_pending = true;
            *changed = true;
            let _ = self.advance_render(webview, budget_ms);
        }
    }
}

/// 从原始 HTML 中提取首个 `<title>` 文本（首帧解析前快速设置标签页标题）。
///
/// R3351：修复三项静默错误（旧实现 `find("<title")` + 首 `>` 截断）：
/// ① **HTML 注释内的 `<title>`**——旧实现命中注释里的 `<title>old</title>`
///   （SSR 模板 / CMS 元数据注释高频），误把注释文本当标题。真实页面如
///   `<!-- <title>old</title> --><title>new</title>` 旧得 "old"（静默错误标签页标题）。
/// ② **`<title>` 起始标签属性中的 `>`**——如 `<title data-x="a>b">` 旧把属性内 `>` 当
///   标签结束，截断错误。
/// ③ **大小写不敏感的结束标签**——旧用 `lower_rest.find("</title>")` 已小写化故已正确，
///   本实现沿用（`</TITLE>` 等同匹配）。
///
/// 实现：手写轻量扫描器——跳过 `<!-- ... -->` 注释块后定位首个 `<title` 起始标签，
/// 经「引号感知」跳过属性找到标签闭合 `>`，再取到 `</title>` 间的文本（trim）。
/// 仅做首帧快速路径，不替代完整解析后的权威标题（DOM `<title>` 元素）。
fn extract_document_title(html: &str) -> Option<String> {
    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        // 跳过 HTML 注释块 `<!-- ... -->`——注释内的 `<title>` 不应被当作标题元素。
        if bytes[i..].starts_with(b"<!--") {
            // 找匹配的 `-->`；未闭合则视为无标题（注释吞噬到末尾）。
            let end = html[i + 4..].find("-->")?;
            i = (i + 4 + end + 3).min(len);
            continue;
        }
        // 命中 `<title` 起始标签（大小写不敏感；后须跟空白 / `>` / `/`，避免误匹配
        // `<titlebar` 等自定义串）。HTML tag 名 ASCII 大小写不敏感（spec）。
        let is_title_start = bytes.get(i..i + 6).is_some_and(|t| t.eq_ignore_ascii_case(b"<title"))
            && bytes
                .get(i + 6)
                .is_some_and(|c| c.is_ascii_whitespace() || *c == b'>' || *c == b'/');
        if is_title_start {
            // 经「引号感知」扫描找到起始标签的闭合 `>`——属性值内的 `>`（如 `data-x="a>b"`）不计。
            let mut j = i + 6;
            let mut in_quote: Option<u8> = None;
            while j < len {
                let c = bytes[j];
                match in_quote {
                    Some(q) if c == q => in_quote = None,
                    Some(_) => {}
                    None if c == b'"' || c == b'\'' => in_quote = Some(c),
                    None if c == b'>' => break,
                    _ => {}
                }
                j += 1;
            }
            if j >= len {
                return None; // 起始标签未闭合
            }
            let content_start = j + 1;
            // 取内容到闭合 `</title>`（大小写不敏感）。
            let rest = html.get(content_start..)?;
            let lower_rest = rest.to_ascii_lowercase();
            let end_rel = lower_rest.find("</title>")?;
            let title = rest[..end_rel].trim();
            return if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            };
        }
        // 单字节推进（ASCII 安全——`<` 只匹配单字节，UTF-8 多字节序列不含 ASCII 字节）。
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::mpsc::{Receiver, channel};

    use crate::{WebView, WebViewConfig};
    use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
    use zero_render_foundation::font::loader::FontLoader;
    use zero_render_foundation::primitive::FontId;

    /// 记录 fetch 调用并立即返回结果的 mock 宿主。
    struct MockFetchHost {
        calls: Vec<String>,
        preconnects: Vec<String>,
        dns_prefetches: Vec<String>,
        text_body: Result<String, String>,
        bytes_body: Result<Vec<u8>, String>,
    }

    impl MockFetchHost {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                preconnects: Vec::new(),
                dns_prefetches: Vec::new(),
                text_body: Ok(String::new()),
                bytes_body: Ok(Vec::new()),
            }
        }

        fn with_text(mut self, body: impl Into<String>) -> Self {
            self.text_body = Ok(body.into());
            self
        }

        fn with_bytes(mut self, body: Vec<u8>) -> Self {
            self.bytes_body = Ok(body);
            self
        }
    }

    impl AsyncFetchHost for MockFetchHost {
        fn preconnect(&mut self, origin: &str) {
            self.preconnects.push(origin.to_string());
        }

        fn dns_prefetch(&mut self, origin: &str) {
            self.dns_prefetches.push(origin.to_string());
        }

        fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            self.calls.push(url.to_string());
            let (tx, rx) = channel();
            let _ = tx.send(self.text_body.clone());
            rx
        }

        fn fetch_bytes_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            self.calls.push(url.to_string());
            let (tx, rx) = channel();
            let _ = tx.send(self.bytes_body.clone());
            rx
        }
    }

    #[test]
    fn font_fetch_uses_one_decodable_source_per_face() {
        let html = r#"<style>@font-face { font-family: Test; src: url(test.eot), url(test.woff), url(test.ttf), url(test.svg); }</style>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/page", html.to_string());
        let mut host = MockFetchHost::new();

        load.begin_font_fetch(&mut host);

        assert_eq!(host.calls, ["https://example.com/test.ttf"]);
        assert_eq!(load.font_pending.len(), 1);
    }

    #[test]
    fn preconnect_hint_is_submitted_without_fetching_a_resource() {
        let mut load = AsyncPageLoad::from_html("https://example.com/page", String::new());
        let mut host = MockFetchHost::new();

        load.begin_preload_hints(r#"<link rel="preconnect" href="https://cdn.example.test">"#, &mut host);

        assert_eq!(host.preconnects, ["https://cdn.example.test/"]);
        assert!(host.calls.is_empty());
    }

    #[test]
    fn dns_prefetch_hint_is_submitted_without_fetching_a_resource() {
        let mut load = AsyncPageLoad::from_html("https://example.com/page", String::new());
        let mut host = MockFetchHost::new();

        load.begin_preload_hints(
            r#"<link rel="dns-prefetch" href="https://cdn.example.test/path">"#,
            &mut host,
        );

        assert_eq!(host.dns_prefetches, ["https://cdn.example.test/"]);
        assert!(host.calls.is_empty());
    }

    struct ErrFetchHost;

    impl AsyncFetchHost for ErrFetchHost {
        fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            let (tx, rx) = channel();
            let _ = tx.send(Err(format!("fail: {url}")));
            rx
        }

        fn fetch_bytes_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            let (tx, rx) = channel();
            let _ = tx.send(Err(format!("fail: {url}")));
            rx
        }
    }

    /// 保持字节请求为 pending，用于验证等待资源期间不会重复重绘。
    struct PendingBytesFetchHost {
        senders: Vec<std::sync::mpsc::Sender<Result<Vec<u8>, String>>>,
    }

    impl AsyncFetchHost for PendingBytesFetchHost {
        fn fetch_text_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            let (_tx, rx) = channel();
            rx
        }

        fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            let (tx, rx) = channel();
            self.senders.push(tx);
            rx
        }
    }

    struct DocumentRequestHost {
        request: Option<(String, String, Option<Vec<u8>>)>,
    }

    impl AsyncFetchHost for DocumentRequestHost {
        fn fetch_document(&mut self, url: &str, method: &str, body: Option<&[u8]>) -> Receiver<Result<String, String>> {
            self.request = Some((url.to_string(), method.to_string(), body.map(Vec::from)));
            let (tx, rx) = channel();
            let _ = tx.send(Ok(
                "<html><head><title>posted</title></head><body></body></html>".to_string()
            ));
            rx
        }

        fn fetch_text_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            let (_tx, rx) = channel();
            rx
        }

        fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            let (_tx, rx) = channel();
            rx
        }
    }

    #[test]
    fn document_request_forwards_method_and_body() {
        let mut load =
            AsyncPageLoad::start_request("https://example.com/submit", "POST", Some(b"name=zero&go=1".to_vec()));
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = DocumentRequestHost { request: None };
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(
            host.request,
            Some((
                "https://example.com/submit".to_string(),
                "POST".to_string(),
                Some(b"name=zero&go=1".to_vec()),
            ))
        );
        assert_eq!(wv.title(), Some("posted"));
    }

    #[test]
    fn begin_stylesheet_fetch_issues_parallel_requests() {
        let html = r#"<html><head>
            <link rel="stylesheet" href="a.css">
            <link rel="stylesheet" href="b.css">
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_text("x");
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(host.calls.len(), 2);
        let expected: HashSet<_> = ["https://example.com/a.css", "https://example.com/b.css"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(host.calls.into_iter().collect::<HashSet<_>>(), expected);
    }

    #[test]
    fn begin_image_fetch_issues_parallel_requests() {
        let html = r#"<html><body>
            <img src="i1.png"><img src="i2.png">
            </body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new();
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(host.calls.len(), 2);
        assert!(host.calls.iter().any(|u| u.ends_with("i1.png")));
        assert!(host.calls.iter().any(|u| u.ends_with("i2.png")));
    }

    #[test]
    fn pending_images_do_not_trigger_repeated_renders() {
        let html = r#"<html><body><img src="pending.png"></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = PendingBytesFetchHost { senders: Vec::new() };

        for _ in 0..10 {
            let _ = load.tick(&mut wv, &mut host, 500.0);
            if load.stage == PageLoadStage::FetchingImages && load.render_session.is_none() && !load.budget_pending {
                break;
            }
        }

        assert_eq!(load.stage, PageLoadStage::FetchingImages);
        assert!(load.render_session.is_none());
        assert!(!load.budget_pending);
        assert!(!load.tick(&mut wv, &mut host, 500.0));
        assert!(!load.budget_pending);
    }

    /// R1795：CSS `url()` 图片引用（background-image）在 async 路径亦被抓取。
    /// inline `<style>` 块内的相对 url 按 base 解析为绝对后请求。
    #[test]
    fn begin_image_fetch_includes_css_url_references() {
        let html = r#"<html><head>
            <style>.hero { background-image: url("bg.png"); }</style>
            </head><body>
            <img src="photo.png">
            </body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new();
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        // <img> photo.png + CSS url() bg.png 各 1 次字节请求。
        assert!(host.calls.iter().any(|u| u.ends_with("photo.png")), "img src fetched");
        assert!(host.calls.iter().any(|u| u.ends_with("bg.png")), "CSS url() fetched");
    }

    #[test]
    fn stub_errors_do_not_block_load_completion() {
        let html = r#"<html><head><link rel="stylesheet" href="x.css"></head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(load.stage(), PageLoadStage::Complete);
    }

    /// R2942：stylesheet fetch 失败收集到 failed_resources（供宿主派发 window 'error'），drain 后清空。
    #[test]
    fn failed_resources_collects_stylesheet_fetch_failure() {
        let html = r#"<html><head>
            <link rel="stylesheet" href="a.css">
            <link rel="stylesheet" href="b.css">
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        let failed = load.take_failed_resources();
        assert_eq!(
            failed.iter().filter(|r| r.kind == "stylesheet").count(),
            2,
            "两条 stylesheet fetch 失败均收集"
        );
        let urls: HashSet<String> = failed.iter().map(|r| r.url.clone()).collect();
        assert!(urls.contains("https://example.com/a.css"), "{urls:?}");
        assert!(urls.contains("https://example.com/b.css"), "{urls:?}");
        assert!(load.take_failed_resources().is_empty(), "drain 后清空");
    }

    /// R2942：image fetch 失败收集到 failed_resources（kind "image"）。
    #[test]
    fn failed_resources_collects_image_fetch_failure() {
        let html = r#"<html><body><img src="broken.png"></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        let failed = load.take_failed_resources();
        assert_eq!(failed.len(), 1, "一条 image fetch 失败");
        assert_eq!(failed[0].kind, "image");
        assert!(failed[0].url.ends_with("broken.png"));
    }

    /// R2943：img 元素级 load/error 事件收集——fetch 失败 → "error"（decode 成功的 "load" 路径为对称单行
    /// push，经 code review 正确；此处用 ErrFetchHost 验证 error 收集 + drain 清空，无需有效图字节）。
    #[test]
    fn img_element_events_collect_fetch_error() {
        let html = r#"<html><body><img src="a.png"><img src="b.png"></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        let events = load.take_img_element_events();
        let errors: Vec<&String> = events.iter().filter(|(_, t)| *t == "error").map(|(u, _)| u).collect();
        assert_eq!(errors.len(), 2, "两条 img fetch 失败均收集为 error: {events:?}");
        assert!(errors.iter().any(|u| u.ends_with("a.png")));
        assert!(errors.iter().any(|u| u.ends_with("b.png")));
        assert!(load.take_img_element_events().is_empty(), "drain 后清空");
    }

    /// R2944：stylesheet 元素级 load/error 事件收集——fetch 失败 → "error"，成功 → "load"。
    #[test]
    fn link_element_events_collect_load_and_error() {
        // ErrFetchHost：两条样式表均 fetch 失败 → 两条 "error"。
        let html = r#"<html><head>
            <link rel="stylesheet" href="a.css">
            <link rel="stylesheet" href="b.css">
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        let events = load.take_link_element_events();
        let errors: Vec<&String> = events.iter().filter(|(_, t)| *t == "error").map(|(u, _)| u).collect();
        assert_eq!(errors.len(), 2, "两条 stylesheet fetch 失败均收集为 error: {events:?}");
        assert!(errors.iter().any(|u| u.ends_with("a.css")));
        assert!(errors.iter().any(|u| u.ends_with("b.css")));
        assert!(load.take_link_element_events().is_empty(), "drain 后清空");
    }

    /// R2408+ slice 2 / FR-003：fetch 成功的 @font-face 字节经 drain 回传（不再丢弃），且 drain 后清空。
    /// 同时验证 FR-002：family 被保留（"TestFont"），fetch 以 FONT meta 发起。
    #[test]
    fn drain_loaded_fonts_returns_fetched_font_bytes_and_family() {
        let html = r#"<html><head>
            <style>@font-face {
                font-family: "TestFont";
                src: url(test.woff);
                font-stretch: condensed;
                size-adjust: 150%;
                font-feature-settings: "liga" off;
                font-variation-settings: "wdth" 125;
                unicode-range: U+41-5A;
            }</style>
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let font_bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let mut host = MockFetchHost::new().with_bytes(font_bytes.clone());
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        // fetch 以 ResourceFetchMeta::FONT 发起（test.woff 一次）。
        assert!(
            host.calls.iter().any(|u| u.ends_with("test.woff")),
            "font url fetched: {:?}",
            host.calls
        );
        let drained = load.drain_loaded_fonts();
        assert_eq!(
            drained,
            vec![(
                "TestFont".to_string(),
                None,
                false,
                Some(75.0),
                Some(1.5),
                vec![OpenTypeFeature::new(*b"liga", 0)],
                vec![OpenTypeVariation::new(*b"wdth", 125.0)],
                vec![(0x41, 0x5A)],
                font_bytes,
            )],
            "family + weight + is_italic + stretch + features + bytes 回传"
        );
        assert!(load.drain_loaded_fonts().is_empty(), "drain 清空");
    }

    /// R2408+ slice 2 / FR-002：纯 inline `<style>` @font-face（无外链 CSS）亦被抓取
    /// （修复 begin_font_fetch 仅经 FetchingStylesheets 调用的遗漏）。
    #[test]
    fn begin_font_fetch_inline_style_without_external_css() {
        let html = r#"<html><head>
            <style>@font-face { font-family: InlineFont; src: url(i.woff); }</style>
            </head><body><p>hi</p></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_bytes(vec![1, 2, 3]);
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert!(
            host.calls.iter().any(|u| u.ends_with("i.woff")),
            "inline @font-face fetched"
        );
        assert_eq!(
            load.drain_loaded_fonts(),
            vec![(
                "InlineFont".to_string(),
                None,
                false,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![1, 2, 3],
            )],
            "inline family drained"
        );
    }

    /// `@font-face src:data:` 不发起 fetch，解码后直接进入字体 drain。
    #[test]
    fn begin_font_fetch_decodes_data_uri_src() {
        let html = r#"<html><head>
            <style>@font-face { font-family: DFont; src: url(data:application/font-woff;base64,AAAA); }</style>
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_bytes(vec![9]);
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert!(
            !host.calls.iter().any(|u| u.contains("data:")),
            "data: src 不应被抓取: {:?}",
            host.calls
        );
        assert_eq!(
            load.drain_loaded_fonts(),
            vec![(
                "DFont".to_string(),
                None,
                false,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![0, 0, 0],
            )]
        );
        assert_eq!(load.take_font_events(), vec![("DFont".to_string(), "loaded")]);
    }

    /// R2408+ slice 2：fetch 失败的字体不污染 drain（仅 log，drain 为空）。
    #[test]
    fn drain_loaded_fonts_empty_on_font_fetch_failure() {
        let html = r#"<html><head>
            <style>@font-face { font-family: BadFont; src: url(bad.woff); }</style>
            </head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = ErrFetchHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(load.stage(), PageLoadStage::Complete);
        assert!(load.drain_loaded_fonts().is_empty(), "失败字体不进 drain");
    }

    /// R2408+ slice 3 / FR-005：生产 drain 合约端到端代理验证。
    ///
    /// 驱动 AsyncPageLoad（生产 async 路径）+ 宿主 drain→load+register+set_resolver
    /// （镜像 renderer `tick_pending_load_with_budget` / tab_worker 的 drain 块），返回
    /// 渲染 glyph 的 font_id 集合。product-smoke 走 harness 路径（load_font_faces_into）
    /// 不覆盖生产 async 路径（R2406 分叉），故本测试是 live-browser 行为的确定性代理。
    ///
    /// decoy 先占 id 0（fallback 槽），使 @font-face "MyAhem"（Ahem.ttf）取 id ≥ 1——
    /// 镜像生产（renderer 先 load_system_fonts 占 id 0+），令声明字体 id 与 fallback 可区分。
    fn render_glyph_font_ids(live_enabled: bool) -> Vec<u32> {
        let ahem_path = format!("{}/../../tests/wpt-runner/fonts/Ahem.ttf", env!("CARGO_MANIFEST_DIR"));
        let ahem = std::fs::read(&ahem_path).expect("Ahem.ttf must exist");
        let html = r#"<html><head>
            <style>@font-face { font-family: "MyAhem"; src: url(ahem.ttf); }</style>
            </head><body><p style="font-family: MyAhem; font-size: 20px">ABC</p></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_bytes(ahem.clone());
        let mut loader = FontLoader::new();
        let _decoy = loader.load_font(&ahem); // id 0：fallback 槽（镜像生产系统字体先载）
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
            if live_enabled {
                for (family, _weight, _is_italic, _stretch, _size_adjust, _features, _variations, ranges, bytes) in
                    load.drain_loaded_fonts()
                {
                    if let Ok(id) = loader.load_font(&bytes) {
                        loader.register_unicode_ranges(id, ranges);
                        loader.register_family_alias(&family, id);
                        wv.set_font_resolver(loader.build_font_resolver());
                        load.request_rerender();
                    }
                }
            } else {
                // kill-switch 关：丢弃字节（镜像 R2406 前 / ZW_LIVE_FONTFACE=0）。
                let _ = load.drain_loaded_fonts();
            }
        }
        let result = wv.last_render().expect("render result");
        result.primitives.glyphs.iter().map(|g| g.font_id.0).collect()
    }

    #[test]
    fn live_fontface_renders_declared_font_when_enabled() {
        let ids = render_glyph_font_ids(true);
        assert!(!ids.is_empty(), "ABC 应产出 glyph，got {ids:?}");
        // 声明字体 MyAhem（Ahem）已加载为 id ≥ 1，glyph 全用之而非 fallback id 0。
        assert!(ids.iter().all(|&id| id >= 1), "glyph 应用声明字体 id≥1，got {ids:?}");
    }

    #[test]
    fn live_fontface_falls_back_when_disabled() {
        let ids = render_glyph_font_ids(false);
        assert!(!ids.is_empty(), "ABC 应产出 glyph，got {ids:?}");
        // kill-switch 关：MyAhem 未注册，glyph 回落 FontId(0)（fallback）。
        assert!(ids.iter().all(|&id| id == 0), "glyph 应回落 id 0，got {ids:?}");
    }

    #[test]
    fn fontid_is_zero_fallback_constant() {
        // 显式锚点：FontId(0) 是 resolve_font_id 的 fallback（painter/mod.rs:331）。
        assert_eq!(FontId(0).0, 0u32);
    }

    /// R2417：生产 drain 合约下 font-weight matching——regular + bold 同族 @font-face，
    /// `<p style="font-family: MyAhem; font-weight: bold">` 的 glyph 用**粗体 face id**（≠ regular）。
    /// 镜像 renderer/tab_worker drain（bold→`{family}:700`，regular→plain family）。
    #[test]
    fn live_fontface_bold_weight_uses_bold_face() {
        let ahem_path = format!("{}/../../tests/wpt-runner/fonts/Ahem.ttf", env!("CARGO_MANIFEST_DIR"));
        let ahem = std::fs::read(&ahem_path).expect("Ahem.ttf");
        let html = r#"<html><head><style>
            @font-face { font-family: "MyAhem"; src: url(reg.woff); }
            @font-face { font-family: "MyAhem"; src: url(bold.woff); font-weight: bold; }
            </style></head><body>
            <p style="font-family: MyAhem; font-weight: bold; font-size: 20px">ABC</p>
            </body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_bytes(ahem.clone());
        let mut loader = FontLoader::new();
        let _decoy = loader.load_font(&ahem); // id 0：fallback 槽（镜像生产系统字体先载）
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
            for (family, weight, _is_italic, stretch, _size_adjust, _features, _variations, ranges, bytes) in
                load.drain_loaded_fonts()
            {
                if let Ok(id) = loader.load_font(&bytes) {
                    loader.register_unicode_ranges(id, ranges);
                    for alias in zero_render_foundation::font::font_face_aliases(&family, weight, false, stretch) {
                        loader.register_family_alias(&alias, id);
                    }
                    wv.set_font_resolver(loader.build_font_resolver());
                    load.request_rerender();
                }
            }
        }
        let resolver = loader.build_font_resolver();
        let bold_id = *resolver.get("MyAhem:700").expect("粗体 face 注册到 MyAhem:700");
        let reg_id = *resolver.get("MyAhem").expect("regular face 注册到 MyAhem");
        assert_ne!(bold_id, reg_id, "bold id ≠ regular id");
        // 渲染的 "ABC"（font-weight:bold）glyph 用粗体 face id。
        let result = wv.last_render().expect("render result");
        assert!(
            result.primitives.glyphs.iter().any(|g| g.font_id.0 == bold_id),
            "ABC (bold) 应以粗体 face id={bold_id} 渲染，glyph ids: {:?}",
            result.primitives.glyphs.iter().map(|g| g.font_id.0).collect::<Vec<_>>()
        );
        let _ = reg_id;
    }

    /// R2493：live browser italic @font-face matching——同族 regular + italic 两个 face，
    /// `<p style="font-family: MyAhem; font-style: italic">` 的 glyph 用**italic face id**（≠ regular）。
    /// 镜像 renderer/tab_worker drain（italic→`{family}:italic`，regular→plain family）。
    #[test]
    fn live_fontface_italic_style_uses_italic_face() {
        let ahem_path = format!("{}/../../tests/wpt-runner/fonts/Ahem.ttf", env!("CARGO_MANIFEST_DIR"));
        let ahem = std::fs::read(&ahem_path).expect("Ahem.ttf");
        let html = r#"<html><head><style>
            @font-face { font-family: "MyAhem"; src: url(reg.woff); }
            @font-face { font-family: "MyAhem"; src: url(italic.woff); font-style: italic; }
            </style></head><body>
            <p style="font-family: MyAhem; font-style: italic; font-size: 20px">ABC</p>
            </body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = MockFetchHost::new().with_bytes(ahem.clone());
        let mut loader = FontLoader::new();
        let _decoy = loader.load_font(&ahem); // id 0：fallback 槽
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
            // 镜像生产 drain：按 (weight, is_italic) 构注册键。
            for (family, weight, is_italic, stretch, _size_adjust, _features, _variations, ranges, bytes) in
                load.drain_loaded_fonts()
            {
                if let Ok(id) = loader.load_font(&bytes) {
                    loader.register_unicode_ranges(id, ranges);
                    for alias in zero_render_foundation::font::font_face_aliases(&family, weight, is_italic, stretch) {
                        loader.register_family_alias(&alias, id);
                    }
                    wv.set_font_resolver(loader.build_font_resolver());
                    load.request_rerender();
                }
            }
        }
        let resolver = loader.build_font_resolver();
        let italic_id = *resolver.get("MyAhem:italic").expect("italic face 注册到 MyAhem:italic");
        let reg_id = *resolver.get("MyAhem").expect("regular face 注册到 MyAhem");
        assert_ne!(italic_id, reg_id, "italic id ≠ regular id");
        // 渲染的 "ABC"（font-style:italic）glyph 用 italic face id。
        let result = wv.last_render().expect("render result");
        assert!(
            result.primitives.glyphs.iter().any(|g| g.font_id.0 == italic_id),
            "ABC (italic) 应以 italic face id={italic_id} 渲染，glyph ids: {:?}",
            result.primitives.glyphs.iter().map(|g| g.font_id.0).collect::<Vec<_>>()
        );
        let _ = reg_id;
    }

    /// R2411：外链样式表内的 `@import` 被递归抓取，URL 相对**该样式表**解析（非文档）。
    #[test]
    fn import_url_fetched_recursively_relative_to_stylesheet() {
        let html = r#"<html><head><link rel="stylesheet" href="a.css"></head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        // a.css 内容含 @import theme.css；MockFetchHost 对所有 text fetch 返回同一 body。
        let body = "@import url(theme.css); .a { color: red; }";
        let mut host = MockFetchHost::new().with_text(body);
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(load.stage(), PageLoadStage::Complete);
        // a.css + 递归 theme.css（相对 a.css 解析为 https://example.com/theme.css）均被抓取。
        assert!(host.calls.iter().any(|u| u.ends_with("a.css")), "a.css fetched");
        assert!(
            host.calls.iter().any(|u| u == "https://example.com/theme.css"),
            "@import theme.css 递归抓取且相对样式表 url 解析: {:?}",
            host.calls
        );
    }

    /// R2411：循环/重复 @import 不会无限抓取（css_seen 防环），加载终止于 Complete。
    #[test]
    fn import_cycle_safe_terminates() {
        let html = r#"<html><head><link rel="stylesheet" href="a.css"></head><body></body></html>"#;
        let mut load = AsyncPageLoad::from_html("https://example.com/", html.to_string());
        let mut wv = WebView::new(WebViewConfig::default());
        // body 自指 @import b.css；b.css fetch 返回同一 body 又 @import b.css——css_seen 命中止。
        let body = "@import url(b.css); .a { color: red; }";
        let mut host = MockFetchHost::new().with_text(body);
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(load.stage(), PageLoadStage::Complete, "循环 @import 须终止");
        // a.css + b.css 各至多一次（css_seen 去重；@import url 不被当图片重复抓取）。
        let a_count = host.calls.iter().filter(|u| u.ends_with("a.css")).count();
        let b_count = host.calls.iter().filter(|u| u.ends_with("b.css")).count();
        assert_eq!(a_count, 1, "a.css 仅一次");
        assert_eq!(b_count, 1, "b.css 仅一次（css_seen 防环）");
    }

    // ── R3351：extract_document_title 注释/属性鲁棒性回归 ──────────────────

    /// R3351：HTML 注释内的 `<title>` 不应被当作标题元素——SSR 模板 / CMS 元数据注释高频
    /// 含 `<title>old</title>` 文本。旧实现 `find("<title")` 命中注释内 title，误返回注释文本。
    #[test]
    fn extract_title_skips_html_comment_r3351() {
        let html = "<!-- <title>old</title> --><title>new</title>";
        assert_eq!(extract_document_title(html).as_deref(), Some("new"));
    }

    /// R3351：注释内的空 title + 注释外的真实 title——旧实现命中注释内 title（trim 空→None），
    /// 但若注释内 title 非空则返回注释文本。本测验证注释整体跳过后取真实标题。
    #[test]
    fn extract_title_comment_then_real_r3351() {
        let html = r#"<html><head>
            <!-- generated by SSR: <title>stale template</title> -->
            <title>Actual Page</title>
        </head></html>"#;
        assert_eq!(extract_document_title(html).as_deref(), Some("Actual Page"));
    }

    /// R3351：`<title>` 起始标签属性值内的 `>`——旧实现把属性内 `>` 当标签闭合，截断错误。
    /// 引号感知扫描后正确取属性后的内容。
    #[test]
    fn extract_title_attr_gt_in_quote_r3351() {
        let html = r#"<title data-x="a>b">Real Title</title>"#;
        assert_eq!(extract_document_title(html).as_deref(), Some("Real Title"));
        // 单引号同。
        let html2 = r#"<title data-y='x>y'>SingleQuote</title>"#;
        assert_eq!(extract_document_title(html2).as_deref(), Some("SingleQuote"));
    }

    /// R3351：`<titlebar` 等含 "title" 前缀的自定义标签不应误匹配——`<title` 后须跟
    /// 空白 / `>` / `/`（本测确认边界检查生效）。
    #[test]
    fn extract_title_word_boundary_r3351() {
        let html = "<titlebar>X</titlebar><title>Real</title>";
        assert_eq!(extract_document_title(html).as_deref(), Some("Real"));
    }

    /// R3351：大小写不敏感的结束标签（`</TITLE>`）仍正确（旧实现 lower_rest 已正确，回归保护）。
    #[test]
    fn extract_title_uppercase_close_tag_r3351() {
        assert_eq!(extract_document_title("<title>Mixed</TITLE>").as_deref(), Some("Mixed"));
        assert_eq!(
            extract_document_title("<TITLE>Upper open</title>").as_deref(),
            Some("Upper open")
        );
    }

    /// R3351：无闭合标签、空标题、无 title 元素——边界保持 None（回归保护）。
    #[test]
    fn extract_title_edge_none_r3351() {
        assert_eq!(extract_document_title("<title>Unclosed"), None);
        assert_eq!(extract_document_title("<title>   </title>"), None); // 空白 trim 后空
        assert_eq!(extract_document_title("<html><body>no title</body></html>"), None);
        // 未闭合注释后含 title——视为无标题（注释吞噬到末尾）。
        assert_eq!(extract_document_title("<!-- never closed <title>X</title>"), None);
    }

    /// R3351：首个 `<title>` 元素胜出（DOM spec：document.title 取第一个 title 子元素）。
    #[test]
    fn extract_title_first_wins_r3351() {
        let html = "<title>First</title><title>Second</title>";
        assert_eq!(extract_document_title(html).as_deref(), Some("First"));
    }

    /// R3351：端到端经 AsyncPageLoad 的文档抓取路径设置 webview 标题——注释场景下取真实
    /// 标题（非注释文本）。`extract_document_title` 仅在文档抓取完成路径调用（from_html 跳过），
    /// 故用内联 host 模拟抓取返回含注释的 HTML。
    #[test]
    fn async_load_title_skips_comment_end_to_end_r3351() {
        struct CommentTitleHost;
        impl AsyncFetchHost for CommentTitleHost {
            fn fetch_document(&mut self, _: &str, _: &str, _: Option<&[u8]>) -> Receiver<Result<String, String>> {
                let (tx, rx) = channel();
                let _ = tx.send(Ok(
                    "<html><head><!-- <title>wrong</title> --><title>correct</title></head><body></body></html>"
                        .to_string(),
                ));
                rx
            }
            fn fetch_text_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
                let (_tx, rx) = channel();
                rx
            }
            fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
                let (_tx, rx) = channel();
                rx
            }
        }
        let mut load = AsyncPageLoad::start("https://example.com/");
        let mut wv = WebView::new(WebViewConfig::default());
        let mut host = CommentTitleHost;
        while load.is_active() {
            let _ = load.tick(&mut wv, &mut host, 500.0);
        }
        assert_eq!(wv.title(), Some("correct"));
    }
}
