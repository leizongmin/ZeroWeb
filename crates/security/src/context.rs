//! 安全上下文 — 统一的资源加载安全决策门面。
//!
//! 在资源加载前统一执行 HSTS 升级、混合内容阻止和 CSP 检查。
//! WebView 和引擎通过 `SecurityContext` 做出安全的资源加载决策。

use crate::hsts::HstsStore;
use crate::mixed_content::{MixedContentStatus, check_mixed_content, is_mixed_content, upgrade_to_https};
use crate::origin::Origin;

/// 资源加载安全检查结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCheckResult {
    /// 允许加载，使用原始 URL。
    Allow,
    /// 允许加载，但 URL 已被安全升级（如 HSTS HTTP→HTTPS）。
    Upgraded(String),
    /// 阻止加载（混合内容、CSP 违规等）。
    Blocked(String),
}

/// 安全上下文 — 在资源加载前统一执行安全检查。
///
/// 组合了：
/// - **HSTS 存储**：自动将 HTTP 请求升级为 HTTPS
/// - **混合内容检测**：阻止/升级 HTTPS 页面上的 HTTP 资源
/// - **页面源跟踪**：跟踪当前页面源用于安全决策
///
/// 使用方式：
/// ```ignore
/// let mut ctx = SecurityContext::new();
/// ctx.set_page_origin(&page_url);
/// match ctx.check_resource_url(&resource_url, "script") {
///     ResourceCheckResult::Allow => { /* 加载 */ }
///     ResourceCheckResult::Upgraded(https_url) => { /* 使用 https_url 加载 */ }
///     ResourceCheckResult::Blocked(reason) => { /* 拒绝加载 */ }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// HSTS 策略存储。
    hsts_store: HstsStore,
    /// 当前页面源。
    page_origin: Option<Origin>,
}

impl SecurityContext {
    /// 创建新的安全上下文。
    pub fn new() -> Self {
        let mut ctx = Self {
            hsts_store: HstsStore::new(),
            page_origin: None,
        };
        ctx.load_preload_list();
        ctx
    }

    /// 设置当前页面源（用于混合内容检测和 CSP）。
    pub fn set_page_origin(&mut self, url: &str) {
        self.page_origin = Origin::parse(url).ok();
    }

    /// 获取当前页面源的引用。
    pub fn page_origin(&self) -> Option<&Origin> {
        self.page_origin.as_ref()
    }

    /// 清除页面源（导航到新页面时调用）。
    pub fn clear_page_origin(&mut self) {
        self.page_origin = None;
    }

    /// 从响应头注册 HSTS 策略。
    ///
    /// 在收到 HTTPS 响应时调用，解析 `Strict-Transport-Security` 头并注册。
    /// 返回 `true` 表示成功解析并注册。
    pub fn register_hsts(&mut self, host: &str, header_value: &str) -> bool {
        self.hsts_store.register_from_header(host, header_value)
    }

    /// 清理过期的 HSTS 记录。
    pub fn cleanup_hsts(&mut self) -> usize {
        self.hsts_store.cleanup_expired()
    }

    /// 检查资源 URL 是否可以安全加载。
    ///
    /// 执行以下检查（按顺序）：
    /// 1. HSTS 升级 — 如果目标域名在 HSTS 存储中，HTTP→HTTPS
    /// 2. 混合内容检测 — 如果页面源是 HTTPS，检查资源是否为 HTTP
    ///    - Blockable 类型（script/style/connect/font 等）→ 阻止
    ///    - OptionallyBlockable 类型（img/audio/video 等）→ 自动升级
    ///
    /// `url` 为资源 URL。
    /// `resource_type` 为资源类型（如 "script", "img", "style", "connect", "font", "media"）。
    pub fn check_resource_url(&mut self, url: &str, resource_type: &str) -> ResourceCheckResult {
        // 阶段 1: HSTS 升级
        if let Some(upgraded) = self.hsts_store.should_upgrade(url) {
            // HSTS 升级：HTTP → HTTPS
            // 升级后继续检查混合内容（使用升级后的 URL）
            return self.check_mixed_content_stage(&upgraded, resource_type, true);
        }

        // 阶段 2: 混合内容检测
        self.check_mixed_content_stage(url, resource_type, false)
    }

    /// 混合内容检查阶段。
    fn check_mixed_content_stage(&self, url: &str, resource_type: &str, already_upgraded: bool) -> ResourceCheckResult {
        let Some(ref page_origin) = self.page_origin else {
            // 无页面源，无法判断混合内容，允许加载
            if already_upgraded {
                return ResourceCheckResult::Upgraded(url.to_string());
            }
            return ResourceCheckResult::Allow;
        };

        // 如果页面不是 HTTPS，无需检查混合内容
        if !page_origin.is_secure() {
            if already_upgraded {
                return ResourceCheckResult::Upgraded(url.to_string());
            }
            return ResourceCheckResult::Allow;
        }

        // 检查是否为混合内容
        if !is_mixed_content(page_origin, url) {
            if already_upgraded {
                return ResourceCheckResult::Upgraded(url.to_string());
            }
            return ResourceCheckResult::Allow;
        }

        // 是混合内容 — 根据类型决定处理方式
        let status = check_mixed_content(page_origin, url, resource_type);
        match status {
            MixedContentStatus::Blockable => ResourceCheckResult::Blocked(format!(
                "Mixed content blocked: {resource_type} resource {url} on secure page"
            )),
            MixedContentStatus::OptionallyBlockable => {
                // 可升级类型 — 自动升级为 HTTPS
                if let Some(upgraded) = upgrade_to_https(url) {
                    // 混合内容自动升级
                    ResourceCheckResult::Upgraded(upgraded)
                } else {
                    // 无法升级（不应发生，因为已经确认是 http:// 开头）
                    ResourceCheckResult::Blocked(format!("Mixed content: cannot upgrade {url}"))
                }
            }
            MixedContentStatus::NotMixedContent => {
                // is_mixed_content 返回 true 但 check_mixed_content 返回 NotMixedContent
                // 不应发生，但安全起见允许
                if already_upgraded {
                    return ResourceCheckResult::Upgraded(url.to_string());
                }
                ResourceCheckResult::Allow
            }
        }
    }

    /// 加载 HSTS 预加载列表（常见域名）。
    ///
    /// 这些域名的 HSTS 策略被内置到浏览器中，
    /// 不需要等待首次 HTTPS 响应头即可自动升级。
    fn load_preload_list(&mut self) {
        let preload_entries: &[(&str, u64, bool)] = &[
            // 主要 CDN 和云服务
            ("cloudflare.com", 31536000, true),
            ("amazonaws.com", 31536000, false),
            ("googleapis.com", 31536000, true),
            ("github.com", 31536000, true),
            ("githubusercontent.com", 31536000, false),
            // 主要社交媒体和通信
            ("facebook.com", 31536000, true),
            ("twitter.com", 31536000, true),
            ("x.com", 31536000, true),
            ("linkedin.com", 31536000, true),
            ("reddit.com", 31536000, true),
            ("discord.com", 31536000, true),
            ("slack.com", 31536000, true),
            ("telegram.org", 31536000, true),
            ("whatsapp.com", 31536000, true),
            // 主要搜索引擎和工具
            ("google.com", 31536000, true),
            ("google.co.jp", 31536000, true),
            ("google.co.uk", 31536000, true),
            ("bing.com", 31536000, true),
            ("duckduckgo.com", 31536000, true),
            ("wikipedia.org", 31536000, true),
            ("mozilla.org", 31536000, true),
            // 主要开发平台
            ("npmjs.com", 31536000, true),
            ("pypi.org", 31536000, true),
            ("crates.io", 31536000, true),
            ("stackoverflow.com", 31536000, true),
            ("gitlab.com", 31536000, true),
            ("bitbucket.org", 31536000, true),
            // 主要云和 SaaS
            ("microsoft.com", 31536000, true),
            ("apple.com", 31536000, true),
            ("amazon.com", 31536000, true),
            ("azure.com", 31536000, true),
            ("digitalocean.com", 31536000, true),
            ("heroku.com", 31536000, true),
            ("vercel.com", 31536000, true),
            ("netlify.com", 31536000, true),
            ("cloudfront.net", 31536000, true),
            ("fastly.net", 31536000, true),
            // 安全和身份
            ("letsencrypt.org", 31536000, true),
            ("github.io", 31536000, true),
            ("pages.dev", 31536000, true),
        ];

        for &(host, max_age, include_subdomains) in preload_entries {
            self.hsts_store.register(
                host,
                crate::hsts::HstsDirective {
                    max_age,
                    include_subdomains,
                    // 预加载策略：使用 u64::MAX 使其永不过期
                    // （与运行时注册的 HSTS 策略不同，预加载策略由浏览器厂商管理）
                    registered_at: u64::MAX - max_age,
                },
            );
        }
    }

    /// 返回 HSTS 存储的策略数量（含预加载）。
    pub fn hsts_count(&self) -> usize {
        self.hsts_store.len()
    }

    /// 返回 HSTS 存储的可变引用（用于高级操作）。
    pub fn hsts_store_mut(&mut self) -> &mut HstsStore {
        &mut self.hsts_store
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_http_page_http_resource() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("http://example.com");
        let result = ctx.check_resource_url("http://cdn.com/script.js", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_allow_https_page_https_resource() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("https://cdn.com/script.js", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_block_mixed_content_script() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://evil.com/script.js", "script");
        assert!(matches!(result, ResourceCheckResult::Blocked(_)));
    }

    #[test]
    fn test_upgrade_mixed_content_img() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://cdn.com/photo.jpg", "img");
        assert_eq!(
            result,
            ResourceCheckResult::Upgraded("https://cdn.com/photo.jpg".to_string())
        );
    }

    #[test]
    fn test_hsts_preload_upgrade() {
        let mut ctx = SecurityContext::new();
        // github.com 在预加载列表中
        let result = ctx.check_resource_url("http://github.com/user/repo", "script");
        // HSTS 升级 + HTTPS 页面源缺失 = Upgraded（无混合内容检查）
        assert!(matches!(result, ResourceCheckResult::Upgraded(_)));
        if let ResourceCheckResult::Upgraded(url) = result {
            assert!(url.starts_with("https://"));
        }
    }

    #[test]
    fn test_hsts_preload_with_mixed_content() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        // github.com 在预加载列表中，HTTP URL 先被 HSTS 升级
        let result = ctx.check_resource_url("http://github.com/file.js", "script");
        // HSTS 升级为 HTTPS → 不再是混合内容
        assert!(matches!(result, ResourceCheckResult::Upgraded(_)));
    }

    #[test]
    fn test_no_page_origin_allows_all() {
        let mut ctx = SecurityContext::new();
        let result = ctx.check_resource_url("http://anything.com/resource", "script");
        // 无页面源时，非 HSTS 域名允许加载
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_block_mixed_content_style() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://cdn.com/style.css", "style");
        assert!(matches!(result, ResourceCheckResult::Blocked(_)));
    }

    #[test]
    fn test_block_mixed_content_connect() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://api.com/data", "connect");
        assert!(matches!(result, ResourceCheckResult::Blocked(_)));
    }

    #[test]
    fn test_upgrade_mixed_content_audio() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://cdn.com/audio.mp3", "audio");
        assert_eq!(
            result,
            ResourceCheckResult::Upgraded("https://cdn.com/audio.mp3".to_string())
        );
    }

    #[test]
    fn test_upgrade_mixed_content_video() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://cdn.com/video.mp4", "video");
        assert_eq!(
            result,
            ResourceCheckResult::Upgraded("https://cdn.com/video.mp4".to_string())
        );
    }

    #[test]
    fn test_register_hsts_from_header() {
        let mut ctx = SecurityContext::new();
        assert!(ctx.register_hsts("custom.com", "max-age=31536000"));
        // 现在 HTTP URL 应该被升级
        let result = ctx.check_resource_url("http://custom.com/page", "script");
        assert!(matches!(result, ResourceCheckResult::Upgraded(_)));
    }

    #[test]
    fn test_preload_list_loaded() {
        let ctx = SecurityContext::new();
        // 预加载列表包含 40+ 条目
        assert!(ctx.hsts_count() > 30);
    }

    #[test]
    fn test_clear_page_origin() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        assert!(ctx.page_origin().is_some());
        ctx.clear_page_origin();
        assert!(ctx.page_origin().is_none());
    }

    #[test]
    fn test_data_uri_not_blocked() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("data:text/html,<h1>Hi</h1>", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_blob_uri_not_blocked() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("blob:https://example.com/abc", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_relative_url_not_blocked() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("scripts/app.js", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }

    #[test]
    fn test_cleanup_hsts() {
        let mut ctx = SecurityContext::new();
        let cleaned = ctx.cleanup_hsts();
        // 预加载策略的 registered_at=0，不过期（max-age 很大）
        assert_eq!(cleaned, 0);
    }

    #[test]
    fn test_mixed_content_blocked_reason_message() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://secure.com");
        let result = ctx.check_resource_url("http://evil.com/steal.js", "script");
        if let ResourceCheckResult::Blocked(reason) = result {
            assert!(reason.contains("script"));
            assert!(reason.contains("http://evil.com/steal.js"));
            assert!(reason.contains("secure page"));
        } else {
            panic!("Expected Blocked, got {result:?}");
        }
    }

    #[test]
    fn test_hsts_subdomain_upgrade() {
        let mut ctx = SecurityContext::new();
        // cloudflare.com 在预加载列表中且 includeSubDomains
        let result = ctx.check_resource_url("http://cdn.cloudflare.com/resource", "script");
        assert!(matches!(result, ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")));
    }

    #[test]
    fn test_mixed_content_font_blocked() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://cdn.com/font.woff2", "font");
        assert!(matches!(result, ResourceCheckResult::Blocked(_)));
    }

    #[test]
    fn test_mixed_content_iframe_blocked() {
        let mut ctx = SecurityContext::new();
        ctx.set_page_origin("https://example.com");
        let result = ctx.check_resource_url("http://other.com/embed", "iframe");
        assert!(matches!(result, ResourceCheckResult::Blocked(_)));
    }

    #[test]
    fn test_default_impl() {
        let ctx = SecurityContext::default();
        assert!(ctx.page_origin.is_none());
        assert!(ctx.hsts_count() > 0);
    }

    #[test]
    fn test_non_preloaded_http_allowed_without_origin() {
        let mut ctx = SecurityContext::new();
        let result = ctx.check_resource_url("http://unknown-site.com/page", "script");
        assert_eq!(result, ResourceCheckResult::Allow);
    }
}
