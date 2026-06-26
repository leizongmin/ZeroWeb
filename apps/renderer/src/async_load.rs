//! 渲染进程分阶段页面加载 — 经 IPC 抓取子资源 + 预算渲染。

use std::collections::HashMap;

use zero_engine::{
    BudgetAdvance, BudgetedRenderSession, RenderPipeline, extract_img_srcs, extract_stylesheet_hrefs,
    image_resource_key, resolve_document_url,
};
use zero_page_runtime::PageLoadHost;
use zero_render_foundation::image_cache::decode_image_bytes;

const FRAME_BUDGET_MS: f64 = 8.0;

/// 加载阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLoadStage {
    FirstPaint,
    FetchingStylesheets,
    StyledPaint,
    FetchingImages,
    Complete,
}

/// 分阶段加载协调器。
pub struct RendererPageLoad {
    url: String,
    html: String,
    css: String,
    css_pending: Vec<String>,
    img_pending: Vec<(String, u64)>,
    stage: PageLoadStage,
    session: Option<BudgetedRenderSession>,
    budget_pending: bool,
    error: Option<String>,
}

impl RendererPageLoad {
    /// 从已获取的 HTML 开始。
    pub fn from_html(url: String, html: String) -> Self {
        Self {
            url,
            html,
            css: String::new(),
            css_pending: Vec::new(),
            img_pending: Vec::new(),
            stage: PageLoadStage::FirstPaint,
            session: None,
            budget_pending: true,
            error: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != PageLoadStage::Complete
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.error.take()
    }

    /// 推进加载；`host` 提供 fetch 与 publish。
    pub fn tick<H: PageLoadHost>(&mut self, pipeline: &mut RenderPipeline, host: &mut H) -> Result<bool, String> {
        let mut changed = false;

        if self.budget_pending {
            changed |= self.advance_render(pipeline, host)?;
        }

        if self.stage == PageLoadStage::FirstPaint && !self.budget_pending && self.session.is_none() {
            self.begin_stylesheet_list();
            changed = true;
        }

        if self.stage == PageLoadStage::FetchingStylesheets {
            changed |= self.fetch_stylesheets(host)?;
        }

        if self.stage == PageLoadStage::FetchingImages {
            changed |= self.fetch_images(pipeline, host)?;
        }

        Ok(changed)
    }

    fn advance_render<H: PageLoadHost>(&mut self, pipeline: &mut RenderPipeline, host: &mut H) -> Result<bool, String> {
        if self.session.is_none() {
            pipeline.set_document_url(Some(&self.url));
            self.session = Some(BudgetedRenderSession::new(self.html.clone(), self.css.clone()));
        }
        let session = self.session.as_mut().expect("session");
        match pipeline.advance_budgeted_render(session, FRAME_BUDGET_MS) {
            BudgetAdvance::InProgress => Ok(false),
            BudgetAdvance::Complete => {
                if let Some(result) = session.take_result() {
                    let title = extract_title(&self.html);
                    let is_final = self.stage == PageLoadStage::FetchingImages
                        || (self.stage == PageLoadStage::StyledPaint && self.img_pending.is_empty());
                    host.publish(&result, title, is_final)?;
                }
                self.session = None;
                self.budget_pending = false;
                match self.stage {
                    // 留在 FirstPaint，由 tick() 调用 begin_stylesheet_list（勿提前切到 FetchingStylesheets）。
                    PageLoadStage::FirstPaint => {}
                    PageLoadStage::StyledPaint | PageLoadStage::FetchingImages
                        if self.css_pending.is_empty() && self.img_pending.is_empty() =>
                    {
                        self.stage = PageLoadStage::Complete;
                    }
                    _ => {}
                }
                Ok(true)
            }
        }
    }

    fn begin_stylesheet_list(&mut self) {
        for href in extract_stylesheet_hrefs(&self.html) {
            let resolved = resolve_document_url(&self.url, &href);
            self.css_pending.push(resolved);
        }
        if self.css_pending.is_empty() {
            self.begin_image_list();
        } else {
            self.stage = PageLoadStage::FetchingStylesheets;
        }
    }

    fn fetch_stylesheets<H: PageLoadHost>(&mut self, host: &mut H) -> Result<bool, String> {
        if self.css_pending.is_empty() {
            if self.stage == PageLoadStage::FetchingStylesheets {
                self.stage = PageLoadStage::StyledPaint;
                self.budget_pending = true;
                self.begin_image_list();
                return Ok(true);
            }
            return Ok(false);
        }
        let urls: Vec<String> = self.css_pending.drain(..).collect();
        for url in urls {
            match host.fetch_bytes(&url) {
                Ok(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes) {
                        self.css.push_str(&text);
                        self.css.push('\n');
                    }
                }
                Err(e) => tracing::warn!("stylesheet {url} fetch failed: {e}"),
            }
        }
        self.stage = PageLoadStage::StyledPaint;
        self.budget_pending = true;
        self.begin_image_list();
        Ok(true)
    }

    fn begin_image_list(&mut self) {
        self.img_pending.clear();
        for src in extract_img_srcs(&self.html) {
            if src.starts_with("data:") {
                continue;
            }
            let resolved = resolve_document_url(&self.url, &src);
            let key = image_resource_key(&resolved, None);
            self.img_pending.push((resolved, key));
        }
        if self.img_pending.is_empty() {
            self.stage = PageLoadStage::Complete;
        } else {
            self.stage = PageLoadStage::FetchingImages;
        }
    }

    fn fetch_images<H: PageLoadHost>(&mut self, pipeline: &mut RenderPipeline, host: &mut H) -> Result<bool, String> {
        if self.img_pending.is_empty() {
            return Ok(false);
        }
        let pending: Vec<(String, u64)> = self.img_pending.drain(..).collect();
        let mut sizes: HashMap<u64, (f32, f32)> = HashMap::new();
        for (url, key) in pending {
            if let Ok(bytes) = host.fetch_bytes(&url)
                && let Ok(data) = decode_image_bytes(&bytes)
            {
                sizes.insert(key, (data.width as f32, data.height as f32));
            }
        }
        if !sizes.is_empty() {
            pipeline.set_image_sizes(sizes);
        }
        self.stage = PageLoadStage::FetchingImages;
        self.budget_pending = true;
        self.img_pending.clear();
        self.advance_render(pipeline, host)
    }
}

fn extract_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html.find("</title>")?;
    if end > start {
        Some(html[start..end].trim().to_string())
    } else {
        None
    }
}

/// 运行完整分阶段加载直到完成。
pub fn run_page_load<H: PageLoadHost>(
    pipeline: &mut RenderPipeline,
    page_url: &str,
    html: &str,
    host: &mut H,
) -> Result<(), String> {
    let mut load = RendererPageLoad::from_html(page_url.to_string(), html.to_string());
    while load.is_active() {
        load.tick(pipeline, host)?;
    }
    if let Some(err) = load.take_error() {
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_engine::RenderResult;

    struct MockHost {
        publishes: u32,
    }

    impl PageLoadHost for MockHost {
        fn fetch_bytes(&mut self, _url: &str) -> Result<Vec<u8>, String> {
            Err("no network in test".into())
        }

        fn publish(&mut self, _result: &RenderResult, _title: Option<String>, _is_final: bool) -> Result<(), String> {
            self.publishes += 1;
            Ok(())
        }
    }

    /// 无外链 CSS/图片时，分阶段加载须能结束（回归：FirstPaint 后勿卡在 FetchingStylesheets）。
    #[test]
    fn run_page_load_completes_without_external_resources() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><head><title>Test</title></head><body><p>hi</p></body></html>";
        let mut host = MockHost { publishes: 0 };
        run_page_load(&mut pipeline, "zero://newtab", html, &mut host).expect("load should complete");
        assert!(host.publishes > 0, "expected at least one publish");
    }
}
