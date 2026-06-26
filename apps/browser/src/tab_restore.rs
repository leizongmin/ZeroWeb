//! Tab 解冻恢复载荷 — LRU 冻结后重建 worker / 渲染进程。

/// 冻结 Tab 解冻时应执行的加载方式。
#[derive(Debug, Clone)]
pub enum TabRestorePayload {
    /// 普通 URL 导航。
    Navigate(String),
    /// 内联 HTML（zero:// 页、设置页等）。
    LoadHtml {
        /// HTML 文档。
        html: String,
        /// 可选 CSS。
        css: Option<String>,
        /// 逻辑 URL。
        url: Option<String>,
    },
}

impl TabRestorePayload {
    /// 从 URL 推断恢复方式（已知 zero:// 内联页）。
    pub fn from_url(url: &str) -> Self {
        if url == "zero://newtab" {
            Self::LoadHtml {
                html: crate::pages::WELCOME_HTML.to_string(),
                css: None,
                url: Some(url.to_string()),
            }
        } else {
            Self::Navigate(url.to_string())
        }
    }
}
