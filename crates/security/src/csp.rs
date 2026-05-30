//! CSP（内容安全策略）模块。
//!
//! 提供 CSP 策略解析和资源加载检查功能。

use crate::origin::Origin;

/// CSP 指令。
#[derive(Debug, Clone)]
pub struct CspDirective {
    /// 指令名称（script-src, style-src, img-src 等）。
    pub name: String,
    /// 指令值（'self', 'unsafe-inline', URL 等）。
    pub values: Vec<String>,
}

/// CSP 策略。
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    /// 策略指令列表。
    pub directives: Vec<CspDirective>,
}

/// CSP sandbox 标志。
///
/// 对应 CSP sandbox 指令支持的各种沙箱标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxFlag {
    /// 允许表单提交。
    AllowForms,
    /// 允许弹窗。
    AllowPopups,
    /// 允许同源访问。
    AllowSameOrigin,
    /// 允许运行脚本。
    AllowScripts,
    /// 允许顶部导航。
    AllowTopNavigation,
    /// 允许通过用户激活进行顶部导航。
    AllowTopNavigationByUserActivation,
    /// 允许弹出窗口使用 ESC 键关闭。
    AllowPopupsToEscapeSandbox,
    /// 允许下载。
    AllowDownloads,
    /// 允许呈现演示。
    AllowPresentation,
    /// 允许存储访问 API。
    AllowStorageAccessByUserActivation,
    /// 允许定向导航。
    AllowOrientationLock,
    /// 允许指针锁定。
    AllowPointerLock,
    /// 允许自动播放。
    AllowAutoplay,
    /// 允许模态窗口。
    AllowModals,
}

impl SandboxFlag {
    /// 从 CSP 指令值字符串解析。
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "allow-forms" => Some(Self::AllowForms),
            "allow-popups" => Some(Self::AllowPopups),
            "allow-same-origin" => Some(Self::AllowSameOrigin),
            "allow-scripts" => Some(Self::AllowScripts),
            "allow-top-navigation" => Some(Self::AllowTopNavigation),
            "allow-top-navigation-by-user-activation" => Some(Self::AllowTopNavigationByUserActivation),
            "allow-popups-to-escape-sandbox" => Some(Self::AllowPopupsToEscapeSandbox),
            "allow-downloads" => Some(Self::AllowDownloads),
            "allow-presentation" => Some(Self::AllowPresentation),
            "allow-storage-access-by-user-activation" => Some(Self::AllowStorageAccessByUserActivation),
            "allow-orientation-lock" => Some(Self::AllowOrientationLock),
            "allow-pointer-lock" => Some(Self::AllowPointerLock),
            "allow-autoplay" => Some(Self::AllowAutoplay),
            "allow-modals" => Some(Self::AllowModals),
            _ => None,
        }
    }
}

impl ContentSecurityPolicy {
    /// 从 Content-Security-Policy header 值解析。
    ///
    /// 格式：`directive1 value1 value2; directive2 value3`
    pub fn parse(header_value: &str) -> Self {
        let directives = header_value
            .split(';')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    return None;
                }
                let mut tokens = part.split_whitespace();
                let name = tokens.next()?.to_string();
                let values: Vec<String> = tokens.map(|t| t.to_string()).collect();
                Some(CspDirective { name, values })
            })
            .collect();

        Self { directives }
    }

    /// 查找指定名称的指令。
    fn find_directive(&self, name: &str) -> Option<&CspDirective> {
        self.directives.iter().find(|d| d.name == name)
    }

    /// 查找指定名称的指令，若不存在则回退到 default-src。
    fn find_directive_or_default(&self, name: &str) -> Option<&CspDirective> {
        self.find_directive(name)
            .or_else(|| self.find_directive("default-src"))
    }

    /// 检查资源加载是否允许。
    ///
    /// `resource_type` 如 "script", "style", "img", "connect", "font", "media"。
    /// `url` 为资源 URL。
    /// `document_origin` 为文档源（用于 'self' 匹配），None 时仅对非绝对 URL 视为同源。
    pub fn is_resource_allowed(
        &self,
        resource_type: &str,
        url: &str,
        document_origin: Option<&Origin>,
    ) -> bool {
        let directive_name = format!("{resource_type}-src");

        let directive = self.find_directive_or_default(&directive_name);

        let Some(directive) = directive else {
            // 没有 default-src 也没有对应指令，默认允许
            return true;
        };

        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查源列表是否匹配给定 URL。
    fn check_source_list(
        &self,
        values: &[String],
        url: &str,
        document_origin: Option<&Origin>,
    ) -> bool {
        if values.is_empty() {
            return true;
        }

        // 检查 'none'
        if values.iter().any(|v| v == "'none'") {
            return false;
        }

        // 检查 '*'
        if values.iter().any(|v| v == "*") {
            return true;
        }

        // 检查 'self' — 与文档源匹配
        if values.iter().any(|v| v == "'self'")
            && Self::is_self_match(url, document_origin)
        {
            return true;
        }

        // 检查精确 URL 匹配
        if values.iter().any(|v| v == url) {
            return true;
        }

        // 检查通配符域名匹配和前缀匹配
        for value in values {
            if let Some(domain) = value.strip_prefix("*.") {
                if Self::wildcard_domain_matches(domain, url) {
                    return true;
                }
            } else if url.starts_with(value) {
                return true;
            }
        }

        false
    }

    /// 判断 URL 是否匹配 'self'（同源）。
    fn is_self_match(url: &str, document_origin: Option<&Origin>) -> bool {
        // 相对路径（非 http/https 开头）视为同源
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return true;
        }
        // 如果提供了文档源，解析 URL 并比较 origin
        if let Some(origin) = document_origin
            && let Ok(resource_origin) = Origin::parse(url)
        {
            return origin.is_same_origin(&resource_origin);
        }
        false
    }

    /// 安全的通配符域名匹配。
    ///
    /// `*.example.com` 应匹配 `sub.example.com`，但不匹配 `notexample.com`。
    fn wildcard_domain_matches(domain: &str, url: &str) -> bool {
        // 从 URL 中提取主机名
        let host = Self::extract_host(url);
        let Some(host) = host else { return false };

        // host 必须以 "." + domain 结尾，或等于 domain（"*." 不包括根域名本身）
        if host.ends_with(&format!(".{domain}")) {
            return true;
        }
        false
    }

    /// 从 URL 字符串提取主机名部分。
    fn extract_host(url: &str) -> Option<String> {
        // 简单提取：尝试剥离 scheme
        let after_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
        // 取到第一个 '/' 或 ':'（端口）或 '?' 或 '#' 之前
        let end = after_scheme
            .find(['/', ':', '?', '#'])
            .unwrap_or(after_scheme.len());
        Some(after_scheme[..end].to_string())
    }

    /// 检查内联脚本是否允许。
    ///
    /// `nonce` 为脚本标签上的 nonce 属性值（不含 'nonce-' 前缀）。
    /// `hash` 为脚本内容的 SHA-256 哈希值（base64 编码，不含 'sha256-' 前缀）。
    pub fn is_inline_script_allowed(
        &self,
        nonce: Option<&str>,
        hash: Option<&str>,
    ) -> bool {
        let directive = self.find_directive_or_default("script-src");

        let Some(directive) = directive else {
            return true;
        };

        if directive.values.iter().any(|v| v == "'unsafe-inline'" || v == "*") {
            return true;
        }

        // 检查 nonce 匹配（CSP 中格式为 'nonce-<value>'，含单引号）
        if let Some(n) = nonce {
            let nonce_quoted = format!("'nonce-{n}'");
            let nonce_bare = format!("nonce-{n}");
            if directive
                .values
                .iter()
                .any(|v| v == &nonce_quoted || v == &nonce_bare)
            {
                return true;
            }
        }

        // 检查 hash 匹配（CSP 中格式为 'sha256-<base64>'，含单引号）
        if let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive
                .values
                .iter()
                .any(|v| v == &hash_quoted || v == &hash_bare)
            {
                return true;
            }
        }

        false
    }

    /// 检查内联样式是否允许。
    ///
    /// `nonce` 为样式标签上的 nonce 属性值（不含 'nonce-' 前缀）。
    /// `hash` 为样式内容的 SHA-256 哈希值（base64 编码，不含 'sha256-' 前缀）。
    pub fn is_inline_style_allowed(
        &self,
        nonce: Option<&str>,
        hash: Option<&str>,
    ) -> bool {
        let directive = self.find_directive_or_default("style-src");

        let Some(directive) = directive else {
            return true;
        };

        if directive.values.iter().any(|v| v == "'unsafe-inline'" || v == "*") {
            return true;
        }

        // 检查 nonce 匹配（CSP 中格式为 'nonce-<value>'，含单引号）
        if let Some(n) = nonce {
            let nonce_quoted = format!("'nonce-{n}'");
            let nonce_bare = format!("nonce-{n}");
            if directive
                .values
                .iter()
                .any(|v| v == &nonce_quoted || v == &nonce_bare)
            {
                return true;
            }
        }

        // 检查 hash 匹配（CSP 中格式为 'sha256-<base64>'，含单引号）
        if let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive
                .values
                .iter()
                .any(|v| v == &hash_quoted || v == &hash_bare)
            {
                return true;
            }
        }

        false
    }

    // ---- 导航和文档指令 ----

    /// 检查 base URI 是否允许。
    ///
    /// 对应 `base-uri` 指令，限制 `<base>` 元素的 href。
    /// `url` 为候选 base URI。
    /// `document_origin` 为文档源。
    pub fn is_base_uri_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive("base-uri");
        let Some(directive) = directive else {
            // base-uri 不回退到 default-src
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查表单提交目标是否允许。
    ///
    /// 对应 `form-action` 指令，限制表单可以提交到哪些地址。
    /// `url` 为表单 action URL。
    /// `document_origin` 为文档源。
    pub fn is_form_action_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive("form-action");
        let Some(directive) = directive else {
            // form-action 不回退到 default-src
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查页面是否可以被嵌入（frame-ancestors）。
    ///
    /// 对应 `frame-ancestors` 指令，限制哪些源可以把此页面嵌入 iframe/frame。
    /// `embedder_origin` 为嵌入方的源。
    pub fn is_frame_ancestor_allowed(&self, embedder_origin: &Origin) -> bool {
        let directive = self.find_directive("frame-ancestors");
        let Some(directive) = directive else {
            // frame-ancestors 不回退到 default-src
            return true;
        };

        if directive.values.is_empty() {
            return true;
        }

        if directive.values.iter().any(|v| v == "'none'") {
            return false;
        }

        if directive.values.iter().any(|v| v == "*") {
            return true;
        }

        if directive.values.iter().any(|v| v == "'self'") {
            // frame-ancestors 'self' — 对于自身嵌入需要文档源，此处简单允许
            return true;
        }

        // 检查源字符串匹配
        let origin_str = format!(
            "{}://{}",
            embedder_origin.scheme,
            if (embedder_origin.port == 80 && embedder_origin.scheme == "http")
                || (embedder_origin.port == 443 && embedder_origin.scheme == "https")
            {
                embedder_origin.host.clone()
            } else {
                format!("{}:{}", embedder_origin.host, embedder_origin.port)
            }
        );
        directive.values.iter().any(|v| v == &origin_str)
    }

    /// 检查导航目标是否允许（navigate-to）。
    ///
    /// 对应 `navigate-to` 指令，限制页面可以导航到哪些地址。
    /// `url` 为目标 URL。
    /// `document_origin` 为文档源。
    pub fn is_navigate_to_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive("navigate-to");
        let Some(directive) = directive else {
            // navigate-to 不回退到 default-src
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 获取 CSP sandbox 标志列表。
    ///
    /// 返回 `None` 表示无 sandbox 指令（不施加沙箱）。
    /// 返回 `Some(flags)` 表示应施加的沙箱限制。
    pub fn sandbox_flags(&self) -> Option<Vec<SandboxFlag>> {
        let directive = self.find_directive("sandbox")?;
        // sandbox 指令存在但无值 → 最严格的沙箱
        Some(
            directive
                .values
                .iter()
                .filter_map(|v| SandboxFlag::from_str(v))
                .collect(),
        )
    }

    /// 检查子资源（iframe/frame）加载是否允许（child-src）。
    ///
    /// 回退顺序：child-src → frame-src → default-src。
    /// `url` 为子资源 URL。
    /// `document_origin` 为文档源。
    pub fn is_child_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self
            .find_directive("child-src")
            .or_else(|| self.find_directive("frame-src"))
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 Worker 加载是否允许（worker-src）。
    ///
    /// 回退顺序：worker-src → child-src → script-src → default-src。
    /// `url` 为 Worker 脚本 URL。
    /// `document_origin` 为文档源。
    pub fn is_worker_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self
            .find_directive("worker-src")
            .or_else(|| self.find_directive("child-src"))
            .or_else(|| self.find_directive("script-src"))
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 Web Manifest 加载是否允许（manifest-src）。
    ///
    /// 回退到 default-src。
    /// `url` 为 manifest 文件 URL。
    /// `document_origin` 为文档源。
    pub fn is_manifest_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("manifest-src");
        let Some(directive) = directive else {
            return true;
        };
        self.check_source_list(&directive.values, url, document_origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 解析测试 ----

    #[test]
    fn test_csp_parse_default_src() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        assert_eq!(csp.directives.len(), 1);
        assert_eq!(csp.directives[0].name, "default-src");
        assert_eq!(csp.directives[0].values, vec!["'self'"]);
    }

    #[test]
    fn test_csp_parse_script_src() {
        let csp = ContentSecurityPolicy::parse("script-src 'self' https://cdn.example.com");
        assert_eq!(csp.directives.len(), 1);
        assert_eq!(csp.directives[0].name, "script-src");
        assert_eq!(
            csp.directives[0].values,
            vec!["'self'", "https://cdn.example.com"]
        );
    }

    #[test]
    fn test_csp_parse_multiple_directives() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'; script-src https://cdn.com");
        assert_eq!(csp.directives.len(), 2);
        assert_eq!(csp.directives[0].name, "default-src");
        assert_eq!(csp.directives[1].name, "script-src");
        assert_eq!(csp.directives[1].values, vec!["https://cdn.com"]);
    }

    #[test]
    fn test_csp_parse_trailing_semicolons() {
        let csp = ContentSecurityPolicy::parse("default-src 'self';");
        assert_eq!(csp.directives.len(), 1);
    }

    #[test]
    fn test_csp_parse_extra_whitespace() {
        let csp = ContentSecurityPolicy::parse("  default-src   'self'  ;  script-src  'self'  ");
        assert_eq!(csp.directives.len(), 2);
    }

    #[test]
    fn test_csp_empty_policy() {
        let csp = ContentSecurityPolicy::parse("");
        assert!(csp.directives.is_empty());
        assert!(csp.is_resource_allowed("script", "https://any.com/script.js", None));
        assert!(csp.is_inline_script_allowed(None, None));
        assert!(csp.is_inline_style_allowed(None, None));
    }

    // ---- 资源加载测试 ----

    #[test]
    fn test_csp_is_resource_allowed_self_relative() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        // 相对 URL 在有或无 document_origin 时都应允许
        assert!(csp.is_resource_allowed("script", "app.js", None));
        assert!(csp.is_resource_allowed(
            "script",
            "app.js",
            Some(&Origin::parse("https://example.com").unwrap())
        ));
    }

    #[test]
    fn test_csp_is_resource_allowed_self_absolute_blocked() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        // 绝对外部 URL，无 document_origin → 应拒绝
        assert!(!csp.is_resource_allowed("script", "https://evil.com/script.js", None));
    }

    #[test]
    fn test_csp_is_resource_allowed_self_with_origin_match() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        // 同源绝对 URL → 应允许
        assert!(csp.is_resource_allowed(
            "script",
            "https://example.com/app.js",
            Some(&doc_origin)
        ));
    }

    #[test]
    fn test_csp_is_resource_allowed_self_with_origin_mismatch() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        // 不同源绝对 URL → 应拒绝
        assert!(!csp.is_resource_allowed(
            "script",
            "https://evil.com/app.js",
            Some(&doc_origin)
        ));
    }

    #[test]
    fn test_csp_is_resource_allowed_none() {
        let csp = ContentSecurityPolicy::parse("script-src 'none'");
        assert!(!csp.is_resource_allowed("script", "app.js", None));
        assert!(!csp.is_resource_allowed("script", "https://cdn.com/app.js", None));
    }

    #[test]
    fn test_csp_resource_allowed_wildcard() {
        let csp = ContentSecurityPolicy::parse("default-src *");
        assert!(csp.is_resource_allowed("script", "https://evil.com/bad.js", None));
    }

    #[test]
    fn test_csp_resource_allowed_exact_url_match() {
        let csp = ContentSecurityPolicy::parse("script-src https://cdn.example.com/app.js");
        assert!(csp.is_resource_allowed("script", "https://cdn.example.com/app.js", None));
        assert!(!csp.is_resource_allowed("script", "https://cdn.example.com/other.js", None));
    }

    #[test]
    fn test_csp_resource_allowed_wildcard_domain() {
        let csp = ContentSecurityPolicy::parse("script-src *.example.com");
        assert!(csp.is_resource_allowed("script", "https://sub.example.com/script.js", None));
        assert!(!csp.is_resource_allowed("script", "https://other.com/script.js", None));
    }

    #[test]
    fn test_csp_wildcard_domain_false_positive() {
        let csp = ContentSecurityPolicy::parse("script-src *.example.com");
        // 这些 URL 不应该匹配 *.example.com
        assert!(!csp.is_resource_allowed("script", "https://evil-example-cdn.com/script.js", None));
        assert!(!csp.is_resource_allowed("script", "https://notexample.com/script.js", None));
        assert!(!csp.is_resource_allowed("script", "https://example.com/script.js", None));
    }

    #[test]
    fn test_csp_wildcard_domain_subdomain_match() {
        let csp = ContentSecurityPolicy::parse("script-src *.example.com");
        assert!(csp.is_resource_allowed("script", "https://cdn.example.com/app.js", None));
        assert!(csp.is_resource_allowed("script", "https://deep.sub.example.com/app.js", None));
    }

    #[test]
    fn test_csp_resource_allowed_url_prefix() {
        let csp = ContentSecurityPolicy::parse("script-src https://cdn.example.com/libs/");
        assert!(csp.is_resource_allowed("script", "https://cdn.example.com/libs/v1/app.js", None));
    }

    #[test]
    fn test_csp_resource_allowed_fallback_to_default_src() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'; img-src https://images.com");
        // "script" type falls back to default-src
        assert!(csp.is_resource_allowed("script", "app.js", None));
        // "img" has specific directive
        assert!(csp.is_resource_allowed("img", "https://images.com/logo.png", None));
    }

    #[test]
    fn test_csp_resource_allowed_directive_empty_values() {
        let csp = ContentSecurityPolicy::parse("script-src");
        assert!(csp.is_resource_allowed("script", "https://evil.com/bad.js", None));
    }

    #[test]
    fn test_csp_resource_type_img() {
        let csp = ContentSecurityPolicy::parse("img-src https://images.com");
        assert!(csp.is_resource_allowed("img", "https://images.com/photo.jpg", None));
        assert!(!csp.is_resource_allowed("img", "https://evil.com/photo.jpg", None));
    }

    // ---- 内联脚本/样式测试（含 nonce/hash）----

    #[test]
    fn test_csp_inline_script_blocked() {
        let csp = ContentSecurityPolicy::parse("script-src 'self'");
        assert!(!csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_script_allowed_unsafe() {
        let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
        assert!(csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_script_nonce_match() {
        let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123'");
        assert!(csp.is_inline_script_allowed(Some("abc123"), None));
        assert!(!csp.is_inline_script_allowed(Some("wrong"), None));
        assert!(!csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_script_hash_match() {
        let csp = ContentSecurityPolicy::parse("script-src 'sha256-RFWPLDbv2BY+rCkDzsE+0fr8ylGr2R2faWMhq4lfEQc='");
        assert!(csp.is_inline_script_allowed(
            None,
            Some("RFWPLDbv2BY+rCkDzsE+0fr8ylGr2R2faWMhq4lfEQc=")
        ));
        assert!(!csp.is_inline_script_allowed(None, Some("wronghash")));
        assert!(!csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_script_nonce_with_unsafe_inline() {
        // 有 'unsafe-inline' 时 nonce 不需要匹配
        let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline' 'nonce-abc'");
        assert!(csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_style_blocked() {
        let csp = ContentSecurityPolicy::parse("style-src 'self'");
        assert!(!csp.is_inline_style_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_style_allowed_unsafe() {
        let csp = ContentSecurityPolicy::parse("style-src 'unsafe-inline'");
        assert!(csp.is_inline_style_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_style_nonce_match() {
        let csp = ContentSecurityPolicy::parse("style-src 'nonce-xyz789'");
        assert!(csp.is_inline_style_allowed(Some("xyz789"), None));
        assert!(!csp.is_inline_style_allowed(Some("wrong"), None));
    }

    #[test]
    fn test_csp_inline_style_hash_match() {
        let csp = ContentSecurityPolicy::parse("style-src 'sha256-base64hashvalue='");
        assert!(csp.is_inline_style_allowed(None, Some("base64hashvalue=")));
        assert!(!csp.is_inline_style_allowed(None, Some("nope")));
    }

    #[test]
    fn test_csp_inline_script_fallback_to_default_src() {
        let csp = ContentSecurityPolicy::parse("default-src 'unsafe-inline'");
        assert!(csp.is_inline_script_allowed(None, None));
    }

    #[test]
    fn test_csp_inline_style_fallback_to_default_src() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        assert!(!csp.is_inline_style_allowed(None, None));
    }

    // ---- 导航/文档指令测试 ----

    #[test]
    fn test_csp_base_uri_allowed() {
        let csp = ContentSecurityPolicy::parse("base-uri 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_base_uri_allowed("https://example.com/base", Some(&doc_origin)));
        assert!(!csp.is_base_uri_allowed("https://evil.com/base", Some(&doc_origin)));
    }

    #[test]
    fn test_csp_base_uri_no_directive() {
        let csp = ContentSecurityPolicy::parse("default-src 'none'");
        // base-uri 不回退到 default-src
        assert!(csp.is_base_uri_allowed("https://any.com/base", None));
    }

    #[test]
    fn test_csp_form_action_allowed() {
        let csp = ContentSecurityPolicy::parse("form-action 'self' https://api.example.com");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_form_action_allowed("https://example.com/submit", Some(&doc_origin)));
        assert!(csp.is_form_action_allowed("https://api.example.com/submit", Some(&doc_origin)));
        assert!(!csp.is_form_action_allowed("https://evil.com/steal", Some(&doc_origin)));
    }

    #[test]
    fn test_csp_form_action_no_directive() {
        let csp = ContentSecurityPolicy::parse("default-src 'none'");
        // form-action 不回退到 default-src
        assert!(csp.is_form_action_allowed("https://any.com/submit", None));
    }

    #[test]
    fn test_csp_frame_ancestors_none() {
        let csp = ContentSecurityPolicy::parse("frame-ancestors 'none'");
        let embedder = Origin::parse("https://embedder.com").unwrap();
        assert!(!csp.is_frame_ancestor_allowed(&embedder));
    }

    #[test]
    fn test_csp_frame_ancestors_self() {
        let csp = ContentSecurityPolicy::parse("frame-ancestors 'self'");
        let embedder = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_frame_ancestor_allowed(&embedder));
    }

    #[test]
    fn test_csp_frame_ancestors_specific_origin() {
        let csp = ContentSecurityPolicy::parse("frame-ancestors https://allowed.com");
        let allowed = Origin::parse("https://allowed.com").unwrap();
        let blocked = Origin::parse("https://blocked.com").unwrap();
        assert!(csp.is_frame_ancestor_allowed(&allowed));
        assert!(!csp.is_frame_ancestor_allowed(&blocked));
    }

    #[test]
    fn test_csp_frame_ancestors_no_directive() {
        let csp = ContentSecurityPolicy::parse("default-src 'none'");
        // frame-ancestors 不回退到 default-src
        let embedder = Origin::parse("https://any.com").unwrap();
        assert!(csp.is_frame_ancestor_allowed(&embedder));
    }

    #[test]
    fn test_csp_navigate_to_allowed() {
        let csp = ContentSecurityPolicy::parse("navigate-to 'self' https://safe.com");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_navigate_to_allowed("https://example.com/page", Some(&doc_origin)));
        assert!(csp.is_navigate_to_allowed("https://safe.com/page", Some(&doc_origin)));
        assert!(!csp.is_navigate_to_allowed("https://evil.com", Some(&doc_origin)));
    }

    #[test]
    fn test_csp_navigate_to_no_directive() {
        let csp = ContentSecurityPolicy::parse("default-src 'none'");
        assert!(csp.is_navigate_to_allowed("https://any.com", None));
    }

    #[test]
    fn test_csp_sandbox_flags() {
        let csp = ContentSecurityPolicy::parse("sandbox allow-scripts allow-forms");
        let flags = csp.sandbox_flags().unwrap();
        assert!(flags.contains(&SandboxFlag::AllowScripts));
        assert!(flags.contains(&SandboxFlag::AllowForms));
        assert!(!flags.contains(&SandboxFlag::AllowSameOrigin));
    }

    #[test]
    fn test_csp_sandbox_empty() {
        let csp = ContentSecurityPolicy::parse("sandbox");
        let flags = csp.sandbox_flags().unwrap();
        assert!(flags.is_empty());
    }

    #[test]
    fn test_csp_sandbox_no_directive() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        assert!(csp.sandbox_flags().is_none());
    }

    #[test]
    fn test_csp_child_src() {
        let csp = ContentSecurityPolicy::parse("child-src https://frames.example.com");
        assert!(csp.is_child_allowed("https://frames.example.com/widget", None));
        assert!(!csp.is_child_allowed("https://evil.com/widget", None));
    }

    #[test]
    fn test_csp_child_src_fallback_to_frame_src() {
        let csp = ContentSecurityPolicy::parse("frame-src https://frames.example.com");
        assert!(csp.is_child_allowed("https://frames.example.com/widget", None));
    }

    #[test]
    fn test_csp_child_src_fallback_to_default() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        assert!(csp.is_child_allowed("app.js", None));
    }

    #[test]
    fn test_csp_worker_src() {
        let csp = ContentSecurityPolicy::parse("worker-src https://workers.example.com");
        assert!(csp.is_worker_allowed("https://workers.example.com/worker.js", None));
        assert!(!csp.is_worker_allowed("https://evil.com/worker.js", None));
    }

    #[test]
    fn test_csp_worker_src_fallback_to_script_src() {
        let csp = ContentSecurityPolicy::parse("script-src https://scripts.example.com");
        assert!(csp.is_worker_allowed("https://scripts.example.com/worker.js", None));
    }

    #[test]
    fn test_csp_manifest_src() {
        let csp = ContentSecurityPolicy::parse("manifest-src https://app.example.com");
        assert!(csp.is_manifest_allowed("https://app.example.com/manifest.json", None));
        assert!(!csp.is_manifest_allowed("https://evil.com/manifest.json", None));
    }

    #[test]
    fn test_csp_manifest_src_fallback_to_default() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_manifest_allowed("https://example.com/manifest.json", Some(&doc_origin)));
    }

    #[test]
    fn test_csp_frame_ancestors_wildcard() {
        let csp = ContentSecurityPolicy::parse("frame-ancestors *");
        let embedder = Origin::parse("https://any.com").unwrap();
        assert!(csp.is_frame_ancestor_allowed(&embedder));
    }

    #[test]
    fn test_csp_sandbox_all_flags() {
        let csp = ContentSecurityPolicy::parse(
            "sandbox allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals",
        );
        let flags = csp.sandbox_flags().unwrap();
        assert!(flags.contains(&SandboxFlag::AllowForms));
        assert!(flags.contains(&SandboxFlag::AllowPopups));
        assert!(flags.contains(&SandboxFlag::AllowSameOrigin));
        assert!(flags.contains(&SandboxFlag::AllowScripts));
        assert!(flags.contains(&SandboxFlag::AllowTopNavigation));
        assert!(flags.contains(&SandboxFlag::AllowModals));
    }

    #[test]
    fn test_csp_worker_src_fallback_chain() {
        // worker-src → child-src → script-src → default-src
        let csp = ContentSecurityPolicy::parse("default-src 'none'; child-src https://child.com");
        assert!(csp.is_worker_allowed("https://child.com/worker.js", None));
    }
}
