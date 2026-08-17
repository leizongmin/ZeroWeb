//! WebView 构建器 — Builder pattern 创建 WebView。

use crate::IndexedDbOwner;
use crate::webview::{WebView, WebViewConfig};

/// WebView 构建器。
pub struct WebViewBuilder {
    config: WebViewConfig,
    indexed_db_owner: Option<IndexedDbOwner>,
}

impl WebViewBuilder {
    /// 创建新的 WebView 构建器。
    pub fn new() -> Self {
        Self {
            config: WebViewConfig::default(),
            indexed_db_owner: None,
        }
    }

    /// 设置视口宽度。
    pub fn width(mut self, w: u32) -> Self {
        self.config.width = w;
        self
    }

    /// 设置视口高度。
    pub fn height(mut self, h: u32) -> Self {
        self.config.height = h;
        self
    }

    /// 设置是否透明背景。
    pub fn transparent(mut self, t: bool) -> Self {
        self.config.transparent = t;
        self
    }

    /// 设置用户代理字符串。
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.config.user_agent = Some(ua.to_string());
        self
    }

    /// 设置初始 URL。
    pub fn url(mut self, url: &str) -> Self {
        self.config.url = Some(url.to_string());
        self
    }

    /// 设置是否启用开发者工具。
    pub fn devtools(mut self, enable: bool) -> Self {
        self.config.devtools = enable;
        self
    }

    /// 设置 HTTP 请求超时（秒）；`None` 使用默认 30s。
    pub fn http_timeout(mut self, secs: Option<u64>) -> Self {
        self.config.http_timeout_secs = secs;
        self
    }

    /// 使用外部 JS 执行器（专用 JS 线程），不在 WebView 内初始化 V8。
    pub fn external_script(mut self, executor: crate::ExternalScriptExecutor) -> Self {
        self.config.external_script = Some(executor);
        self
    }

    /// 外链脚本源获取器（进程内/headless 路径）：fetch 外链 `<script src>` / `<script type=module src>`
    /// 源后于进程内 sandbox 执行。入参 `(page_url, script_src)`，返回脚本源（或错误 → 该脚本跳过）。
    /// 与 `external_script`（多进程执行委托）互斥独立。为 None 时外链脚本跳过（离线语义）。
    pub fn script_source_fetcher(mut self, fetcher: crate::ScriptSourceFetcher) -> Self {
        self.config.script_source_fetcher = Some(fetcher);
        self
    }

    /// P1b S2：启用原生 DOM 绑定（`engine::dom_bindings`，read-only 快照）。
    ///
    /// 开启时 `run_page_scripts` 在 polyfill 桥之上额外安装原生 `nodeType`/`tagName` 等
    /// getter，从 re-parsed `Document` 直读（不经 shim 字符串桥）。默认关 → 零回归。
    /// 详见 [`WebViewConfig::native_dom`]。
    pub fn native_dom(mut self, enabled: bool) -> Self {
        self.config.native_dom = enabled;
        self
    }

    /// 使用宿主提供的 IndexedDB owner。
    pub fn indexed_db_owner(mut self, owner: IndexedDbOwner) -> Self {
        self.indexed_db_owner = Some(owner);
        self
    }

    /// 构建 WebView 实例。
    ///
    /// 如果 `config.url` 已设置，会自动调用 `load_url` 将 WebView
    /// 置为加载状态（但不会同步发起网络请求）。
    pub fn build(self) -> WebView {
        let mut wv = match self.indexed_db_owner {
            Some(owner) => WebView::new_with_indexed_db_owner(self.config, owner),
            None => WebView::new(self.config),
        };
        if let Some(ref url) = wv.config().url {
            let url = url.clone();
            wv.load_url(&url);
        }
        wv
    }
}

impl Default for WebViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}
