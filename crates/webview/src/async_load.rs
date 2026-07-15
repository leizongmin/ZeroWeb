//! 分阶段异步页面加载 — 首帧 HTML、CSS、图片子资源分步推进。

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use zero_engine::image_resource_key;
use zero_engine::preload::{ResourceHintType, ResourceType, scan_html_resource_hints};
use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, extract_font_face_urls, extract_img_resources, extract_stylesheet_hrefs,
};
use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_render_foundation::image_cache::{ImageKey, decode_image_bytes};

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

/// 分阶段异步加载协调器。
pub struct AsyncPageLoad {
    url: String,
    stage: PageLoadStage,
    html: Option<String>,
    css: String,
    css_pending: Vec<(String, Receiver<Result<String, String>>)>,
    img_pending: Vec<(String, u64, BytesFetchRx)>,
    lazy_img_pending: Vec<(String, u64, BytesFetchRx)>,
    font_pending: Vec<(String, BytesFetchRx)>,
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
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            font_pending: Vec::new(),
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
            img_pending: Vec::new(),
            lazy_img_pending: Vec::new(),
            font_pending: Vec::new(),
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
        !self.font_pending.is_empty() || !self.lazy_img_pending.is_empty()
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
            self.css_pending
                .push((abs.clone(), host.fetch_text_meta(&abs, ResourceFetchMeta::STYLESHEET)));
        }
        if self.css_pending.is_empty() {
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
        self.css_pending.retain(|(url, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(css) => {
                        self.css.push_str(&css);
                        self.css.push('\n');
                    }
                    Err(e) => tracing::warn!("stylesheet {url} fetch failed: {e}"),
                }
                *changed = true;
                false
            } else {
                true
            }
        });
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
        let urls = extract_font_face_urls(&self.css);
        let base = url::Url::parse(&self.url).ok();
        for href in urls {
            let abs = match base.as_ref().and_then(|b| b.join(&href).ok()) {
                Some(u) => u.to_string(),
                None => href,
            };
            self.font_pending
                .push((abs.clone(), host.fetch_bytes_meta(&abs, ResourceFetchMeta::FONT)));
        }
        if !self.font_pending.is_empty() {
            tracing::info!(url = %self.url, count = self.font_pending.len(), "page load: fetch fonts");
        }
    }

    fn poll_fonts(&mut self, changed: &mut bool) {
        self.font_pending.retain(|(url, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => tracing::info!(url, bytes = bytes.len(), "page load: font fetched"),
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
}
