//! 分阶段异步页面加载 — 首帧 HTML、CSS、图片子资源分步推进。

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use zero_engine::image_resource_key;
use zero_engine::preload::{ResourceHintType, ResourceType, scan_html_resource_hints};
use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, extract_css_image_urls, extract_font_faces, extract_html_style_text,
    extract_img_resources, extract_import_urls, extract_stylesheet_hrefs,
};
use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_render_foundation::image_cache::{ImageKey, decode_data_uri, decode_image_bytes};

use crate::net_pool::{fetch_bytes_async_meta, fetch_text_async_meta};
use crate::webview::WebView;

/// 图片抓取异步接收器（net_pool 线程 → 加载器轮询）。
type BytesFetchRx = Receiver<Result<Vec<u8>, String>>;

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

/// in-process 异步抓取宿主：经 webview net_pool 线程池抓取（tabworker 默认）。
pub struct InProcessFetchHost;

impl AsyncFetchHost for InProcessFetchHost {
    fn fetch_text_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        fetch_text_async_meta(url.to_string(), meta)
    }

    fn fetch_bytes_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        fetch_bytes_async_meta(url.to_string(), meta)
    }
}

/// 生产 @font-face 异步加载是否启用（env `ZW_LIVE_FONTFACE`；默认启用，`0`/`false` 关闭）。
///
/// kill-switch：关闭后宿主跳过 drain→load→register→resolver 刷新，行为退回 R2406 前
///（字节抓取后丢弃）。读取方式对齐既有 env 模式（如 `ZERO_BROWSER_MULTIPROCESS`）。
pub fn live_fontface_enabled() -> bool {
    match std::env::var("ZW_LIVE_FONTFACE") {
        Ok(v) => v != "0" && !v.eq_ignore_ascii_case("false"),
        Err(_) => true,
    }
}

/// 分阶段异步加载协调器。
pub struct AsyncPageLoad {
    url: String,
    stage: PageLoadStage,
    html: Option<String>,
    css: String,
    css_pending: Vec<(String, Receiver<Result<String, String>>)>,
    /// R2411：已 fetch 的样式表绝对 URL 集合（含 `<link>` 与递归 `@import`），防 @import 环。
    css_seen: HashSet<String>,
    img_pending: Vec<(String, u64, BytesFetchRx)>,
    lazy_img_pending: Vec<(String, u64, BytesFetchRx)>,
    font_pending: Vec<(String, Option<u16>, String, BytesFetchRx)>,
    /// R2408+ slice 2：poll_fonts 收集的已就绪 @font-face 字节 `(family, weight, bytes)`，
    /// 供宿主在 tick 后经 `drain_loaded_fonts()` 取出并 load+register（drain pattern）。
    /// weight（R2417）供宿主按 weight 构 `{family}:700` 粗体键。
    font_loaded: Vec<(String, Option<u16>, Vec<u8>)>,
    lazy_urls: Vec<String>,
    document_rx: Option<Receiver<Result<String, String>>>,
    render_session: Option<BudgetedRenderSession>,
    budget_pending: bool,
    last_error: Option<String>,
}

impl AsyncPageLoad {
    /// 开始加载 URL（主文档在首 tick 经 host 抓取，tabworker 默认 InProcessFetchHost）。
    pub fn start(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            stage: PageLoadStage::FetchingDocument,
            html: None,
            css: String::new(),
            css_pending: Vec::new(),
            css_seen: HashSet::new(),
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            font_pending: Vec::new(),
            font_loaded: Vec::new(),
            lazy_urls: Vec::new(),
            document_rx: None,
            render_session: None,
            budget_pending: false,
            last_error: None,
        }
    }

    /// 取出并清除加载失败原因（主文档抓取失败等）。
    pub fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }

    /// 是否因错误结束。
    pub fn failed(&self) -> bool {
        self.stage == PageLoadStage::Failed
    }

    /// 从已有 HTML 开始（跳过主文档网络，外链子资源经 host 抓取）。
    pub fn from_html(url: impl Into<String>, html: String) -> Self {
        Self {
            url: url.into(),
            stage: PageLoadStage::FirstPaint,
            html: Some(html),
            css: String::new(),
            css_pending: Vec::new(),
            css_seen: HashSet::new(),
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            font_pending: Vec::new(),
            font_loaded: Vec::new(),
            lazy_urls: Vec::new(),
            document_rx: None,
            render_session: None,
            budget_pending: true,
            last_error: None,
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
        !self.font_pending.is_empty() || !self.lazy_img_pending.is_empty() || self.budget_pending
    }

    /// 请求下一 tick 重渲染（外部状态变化后调用，如宿主加载 @font-face 后更新 resolver）。
    ///
    /// 仅在有 HTML 时置 `budget_pending`，使下一 tick 的 `advance_render` 用新 resolver 重绘。
    pub fn request_rerender(&mut self) {
        if self.html.is_some() {
            self.budget_pending = true;
        }
    }

    /// 取出并清空已就绪的 @font-face 字节 `(family, weight, bytes)`（drain pattern）。
    ///
    /// 宿主在 `tick` 返回后调用——`poll_fonts` 把 fetch 成功的字节收集到此处，不再丢弃。
    /// 宿主据此 `load_font` + `register_family_alias`（weight≥600 时另构 `{family}:700`
    /// 粗体键，R2417）+ 刷新 resolver + `request_rerender`。
    pub fn drain_loaded_fonts(&mut self) -> Vec<(String, Option<u16>, Vec<u8>)> {
        std::mem::take(&mut self.font_loaded)
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
            self.document_rx = Some(host.fetch_text_meta(&url, ResourceFetchMeta::DOCUMENT));
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
            if hint.hint_type != ResourceHintType::Preload {
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(&hint.url).ok()) {
                Some(u) => u.to_string(),
                None => hint.url.clone(),
            };
            let meta = match hint.resource_type {
                ResourceType::Style => ResourceFetchMeta::STYLESHEET,
                ResourceType::Script => ResourceFetchMeta::SCRIPT,
                ResourceType::Font => ResourceFetchMeta::FONT,
                ResourceType::Image => ResourceFetchMeta::IMAGE,
                _ => ResourceFetchMeta::preload("fetch"),
            };
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
                    Err(e) => tracing::warn!("stylesheet {url} fetch failed: {e}"),
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
        for (family, sources, weight) in faces {
            for src in &sources {
                // data: 不走 fetch（与图片 data: 路径一致）；local() 已被 css-parser 排除。
                if src.starts_with("data:") {
                    continue;
                }
                // 抓取所有非 data 源（woff2/woff/ttf）——fontdue 对 woff2 加载会失败被跳过，
                // loader 跌代到可解码的源；保证 woff2-first 声明仍能注册（RFC §8.4）。
                let abs = match base.as_ref().and_then(|b| b.join(src).ok()) {
                    Some(u) => u.to_string(),
                    None => src.clone(),
                };
                self.font_pending.push((
                    family.clone(),
                    weight,
                    abs.clone(),
                    host.fetch_bytes_meta(&abs, ResourceFetchMeta::FONT),
                ));
            }
        }
        if !self.font_pending.is_empty() {
            tracing::info!(url = %self.url, count = self.font_pending.len(), "page load: fetch fonts");
        }
    }

    fn poll_fonts(&mut self, changed: &mut bool) {
        self.font_pending.retain(|(family, weight, url, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => {
                        tracing::info!(url, bytes = bytes.len(), "page load: font fetched");
                        // R2408+ slice 2：保留字节供宿主 drain 后 load+register（drain pattern），
                        // 不再丢弃。family 用于 register_family_alias；weight（R2417）用于按
                        // weight 构 {family}:700 粗体键。
                        self.font_loaded.push((family.clone(), *weight, bytes));
                    }
                    Err(e) => tracing::warn!("font {url} fetch failed: {e}"),
                }
                *changed = true;
                false
            } else {
                true
            }
        });
    }

    fn begin_image_fetch(&mut self, webview: &mut WebView, host: &mut dyn AsyncFetchHost) {
        let html = match self.html.as_ref() {
            Some(h) => h.as_str(),
            None => return,
        };
        let imgs = extract_img_resources(html);
        let base = url::Url::parse(&self.url).ok();
        for img in imgs {
            if img.src.starts_with("data:") {
                // R1987：data: URI（PNG/JPEG/WebP/SVG）无 HTTP fetch，直接解码并入缓存
                // （in-scope img 子资源，goal line 118；与 sync 路径 fetch_image_subresources 对齐）。
                if let Ok(data) = decode_data_uri(&img.src) {
                    let key = image_resource_key(&img.src, None);
                    webview.image_cache().insert_with_key(ImageKey::new(key), data);
                }
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(&img.src).ok()) {
                Some(u) => u.to_string(),
                None => img.src,
            };
            if img.lazy {
                self.lazy_urls.push(abs);
                continue;
            }
            let key = image_resource_key(&abs, None);
            self.img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, ResourceFetchMeta::IMAGE)));
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
            // <img src> 与 CSS url() 可能指向同一资源：去重，避免重复抓取。
            if self.img_pending.iter().any(|(a, _, _)| *a == abs) || self.lazy_urls.contains(&abs) {
                continue;
            }
            let key = image_resource_key(&abs, None);
            self.img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, ResourceFetchMeta::IMAGE)));
        }
        if self.img_pending.is_empty() && self.lazy_urls.is_empty() {
            self.stage = PageLoadStage::Complete;
        } else if !self.img_pending.is_empty() {
            tracing::info!(
                url = %self.url,
                count = self.img_pending.len(),
                lazy = self.lazy_urls.len(),
                "page load: fetch images"
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
            self.lazy_img_pending
                .push((abs.clone(), key, host.fetch_bytes_meta(&abs, ResourceFetchMeta::IMAGE)));
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
                        if let Ok(img) = decode_image_bytes(&bytes) {
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
                            webview.image_cache().insert_with_key(ImageKey::new(*key), img);
                        }
                    }
                    Err(e) => tracing::warn!("lazy image {url} fetch failed: {e}"),
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
        let mut sizes: HashMap<u64, (f32, f32)> = webview.cached_image_sizes().clone();
        let mut ratios: HashMap<u64, f32> = webview.cached_image_ratios().clone();
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = webview.cached_image_no_ratio().clone();
        self.img_pending.retain(|(url, key, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => match decode_image_bytes(&bytes) {
                        Ok(img) => {
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
                        }
                        Err(e) => tracing::warn!("image {url} decode failed: {e}"),
                    },
                    Err(e) => tracing::warn!("image {url} fetch failed: {e}"),
                }
                *changed = true;
                false
            } else {
                true
            }
        });
        if !sizes.is_empty() {
            webview.set_image_sizes(sizes);
        }
        if !ratios.is_empty() {
            webview.set_image_ratios(ratios);
        }
        if !no_ratio.is_empty() {
            webview.set_image_no_ratio(no_ratio);
        }
        if *changed && self.stage == PageLoadStage::FetchingImages {
            let remaining = self.img_pending.len();
            tracing::info!(
                url = %self.url,
                remaining,
                "page load: image batch ready, incremental render"
            );
            self.budget_pending = true;
        }
        if self.img_pending.is_empty() {
            tracing::info!(url = %self.url, "page load: all eager images ready, final render");
            self.stage = PageLoadStage::Complete;
            self.budget_pending = true;
            *changed = true;
            let _ = self.advance_render(webview, budget_ms);
        }
    }
}

fn extract_document_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let tag_start = lower.find("<title")?;
    let content_start = html[tag_start..].find('>')? + tag_start + 1;
    let rest = &html[content_start..];
    let lower_rest = rest.to_ascii_lowercase();
    let end = lower_rest.find("</title>")?;
    let title = rest[..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
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
        text_body: Result<String, String>,
        bytes_body: Result<Vec<u8>, String>,
    }

    impl MockFetchHost {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
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

    /// R2408+ slice 2 / FR-003：fetch 成功的 @font-face 字节经 drain 回传（不再丢弃），且 drain 后清空。
    /// 同时验证 FR-002：family 被保留（"TestFont"），fetch 以 FONT meta 发起。
    #[test]
    fn drain_loaded_fonts_returns_fetched_font_bytes_and_family() {
        let html = r#"<html><head>
            <style>@font-face { font-family: "TestFont"; src: url(test.woff); }</style>
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
            vec![("TestFont".to_string(), None, font_bytes)],
            "family + weight + bytes 回传"
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
            vec![("InlineFont".to_string(), None, vec![1, 2, 3])],
            "inline family drained"
        );
    }

    /// R2408+ slice 2 / FR-002：@font-face `src: url(data:...)` 不发起 fetch（与图片 data: 一致）。
    #[test]
    fn begin_font_fetch_skips_data_uri_src() {
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
        assert!(load.drain_loaded_fonts().is_empty(), "无 data: 字体被加载");
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
                for (family, _weight, bytes) in load.drain_loaded_fonts() {
                    if let Ok(id) = loader.load_font(&bytes) {
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
            for (family, weight, bytes) in load.drain_loaded_fonts() {
                if let Ok(id) = loader.load_font(&bytes) {
                    if weight.is_some_and(|w| w >= 600) {
                        loader.register_family_alias(&format!("{family}:700"), id);
                    } else {
                        loader.register_family_alias(&family, id);
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
}
