//! 分阶段异步页面加载 — 首帧 HTML、CSS、图片子资源分步推进。

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use zero_engine::image_resource_key;
use zero_engine::{BudgetAdvance, BudgetedRenderSession, extract_img_srcs, extract_stylesheet_hrefs};
use zero_page_runtime::AsyncFetchHost;
use zero_render_foundation::image_cache::{ImageKey, decode_image_bytes};

use crate::net_pool::{fetch_bytes_async, fetch_text_async};
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
    fn fetch_text(&mut self, url: &str) -> Receiver<Result<String, String>> {
        fetch_text_async(url.to_string())
    }

    fn fetch_bytes(&mut self, url: &str) -> Receiver<Result<Vec<u8>, String>> {
        fetch_bytes_async(url.to_string())
    }
}

/// 分阶段异步加载协调器。
pub struct AsyncPageLoad {
    host: Box<dyn AsyncFetchHost>,
    url: String,
    stage: PageLoadStage,
    html: Option<String>,
    css: String,
    css_pending: Vec<(String, Receiver<Result<String, String>>)>,
    img_pending: Vec<(String, u64, BytesFetchRx)>,
    document_rx: Option<Receiver<Result<String, String>>>,
    render_session: Option<BudgetedRenderSession>,
    budget_pending: bool,
    last_error: Option<String>,
}

impl AsyncPageLoad {
    /// 开始加载 URL（主文档走 net_pool，tabworker 默认宿主）。
    pub fn start(url: impl Into<String>) -> Self {
        Self::start_with_host(url, Box::new(InProcessFetchHost))
    }

    /// 用自定义异步抓取宿主开始加载（renderer 经 IPC 复用加载器时使用）。
    pub fn start_with_host(url: impl Into<String>, mut host: Box<dyn AsyncFetchHost>) -> Self {
        let url = url.into();
        let document_rx = host.fetch_text(&url);
        Self {
            host,
            url,
            stage: PageLoadStage::FetchingDocument,
            html: None,
            css: String::new(),
            css_pending: Vec::new(),
            img_pending: Vec::new(),
            document_rx: Some(document_rx),
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

    /// 从已有 HTML 开始（跳过主文档网络，默认 net_pool 宿主）。
    pub fn from_html(url: impl Into<String>, html: String) -> Self {
        Self::from_html_with_host(url, html, Box::new(InProcessFetchHost))
    }

    /// 从已有 HTML + 自定义异步抓取宿主开始。
    pub fn from_html_with_host(url: impl Into<String>, html: String, host: Box<dyn AsyncFetchHost>) -> Self {
        Self {
            host,
            url: url.into(),
            stage: PageLoadStage::FirstPaint,
            html: Some(html),
            css: String::new(),
            css_pending: Vec::new(),
            img_pending: Vec::new(),
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
        !matches!(self.stage, PageLoadStage::Complete | PageLoadStage::Failed)
    }

    /// 在 `budget_ms` 内推进加载与渲染；返回 `true` 表示状态有更新。
    pub fn tick(&mut self, webview: &mut WebView, budget_ms: f64) -> bool {
        let mut changed = false;

        if let Some(rx) = self.document_rx.as_ref()
            && let Ok(result) = rx.try_recv()
        {
            self.document_rx = None;
            match result {
                Ok(html) => {
                    if let Some(title) = extract_document_title(&html) {
                        webview.set_title(&title);
                    }
                    self.html = Some(html);
                    self.stage = PageLoadStage::FirstPaint;
                    self.budget_pending = true;
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
            self.begin_stylesheet_fetch(webview);
            changed = true;
        }

        self.poll_stylesheets(webview, budget_ms, &mut changed);
        self.poll_images(webview, budget_ms, &mut changed);

        changed
    }

    fn advance_render(&mut self, webview: &mut WebView, budget_ms: f64) -> bool {
        let html = match self.html.as_ref() {
            Some(h) => h.clone(),
            None => return false,
        };

        if self.render_session.is_none() {
            webview.prepare_document_state(&self.url);
            webview.set_cached_content(&html, &self.css);
            self.render_session = Some(BudgetedRenderSession::new(html, self.css.clone()));
        }

        let session = self.render_session.as_mut().expect("session");
        match webview.advance_budget_session(session, budget_ms) {
            BudgetAdvance::Complete => {
                if let Some(result) = session.take_result() {
                    let done = matches!(self.stage, PageLoadStage::FetchingImages | PageLoadStage::Complete)
                        && self.img_pending.is_empty();
                    webview.apply_render_result(result, &self.url, done);
                }
                self.render_session = None;
                self.budget_pending = false;
                match self.stage {
                    // 留在 FirstPaint，由 tick() 调用 begin_stylesheet_fetch。
                    PageLoadStage::FirstPaint => {}
                    PageLoadStage::StyledPaint | PageLoadStage::FetchingImages
                        if self.css_pending.is_empty() && self.img_pending.is_empty() =>
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

    fn begin_stylesheet_fetch(&mut self, webview: &mut WebView) {
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
            self.css_pending.push((abs.clone(), self.host.fetch_text(&abs)));
        }
        if self.css_pending.is_empty() {
            self.begin_image_fetch(webview);
        } else {
            self.stage = PageLoadStage::FetchingStylesheets;
        }
    }

    fn poll_stylesheets(&mut self, webview: &mut WebView, budget_ms: f64, changed: &mut bool) {
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
            self.stage = PageLoadStage::StyledPaint;
            self.budget_pending = true;
            *changed = true;
            let _ = self.advance_render(webview, budget_ms);
            self.begin_image_fetch(webview);
        }
    }

    fn begin_image_fetch(&mut self, webview: &mut WebView) {
        let html = match self.html.as_ref() {
            Some(h) => h.as_str(),
            None => return,
        };
        let srcs = extract_img_srcs(html);
        let base = url::Url::parse(&self.url).ok();
        for src in srcs {
            if src.starts_with("data:") {
                continue;
            }
            let abs = match base.as_ref().and_then(|b| b.join(&src).ok()) {
                Some(u) => u.to_string(),
                None => src,
            };
            let key = image_resource_key(&abs, None);
            self.img_pending.push((abs.clone(), key, self.host.fetch_bytes(&abs)));
        }
        if self.img_pending.is_empty() {
            self.stage = PageLoadStage::Complete;
        } else {
            self.stage = PageLoadStage::FetchingImages;
        }
        let _ = webview;
    }

    fn poll_images(&mut self, webview: &mut WebView, budget_ms: f64, changed: &mut bool) {
        if self.stage != PageLoadStage::FetchingImages {
            return;
        }
        let mut sizes: HashMap<u64, (f32, f32)> = webview.cached_image_sizes().clone();
        self.img_pending.retain(|(url, key, rx)| {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(bytes) => match decode_image_bytes(&bytes) {
                        Ok(img) => {
                            let (w, h) = (img.width as f32, img.height as f32);
                            sizes.insert(*key, (w, h));
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
        if self.img_pending.is_empty() {
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
