//! CSP（内容安全策略）模块。
//!
//! 提供 CSP 策略解析和资源加载检查功能。

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

    /// 检查资源加载是否允许。
    ///
    /// `resource_type` 如 "script", "style", "img", "connect", "font", "media"。
    /// `url` 为资源 URL。
    pub fn is_resource_allowed(&self, resource_type: &str, url: &str) -> bool {
        let directive_name = format!("{resource_type}-src");

        // 查找对应指令，如果没有则使用 default-src
        let directive = self
            .directives
            .iter()
            .find(|d| d.name == directive_name)
            .or_else(|| {
                self.directives
                    .iter()
                    .find(|d| d.name == "default-src")
            });

        let Some(directive) = directive else {
            // 没有 default-src 也没有对应指令，默认允许
            return true;
        };

        if directive.values.is_empty() {
            // 指令无值表示不限制
            return true;
        }

        // 检查 'none'
        if directive
            .values
            .iter()
            .any(|v| v == "'none'")
        {
            return false;
        }

        // 检查 '*'
        if directive.values.iter().any(|v| v == "*") {
            return true;
        }

        // 检查 'self'
        if directive
            .values
            .iter()
            .any(|v| v == "'self'")
        {
            // 简化：如果 URL 不以 http 开头（相对路径），视为同源
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return true;
            }
        }

        // 检查精确 URL 匹配
        if directive.values.iter().any(|v| v == url) {
            return true;
        }

        // 检查通配符域名匹配
        for value in &directive.values {
            if let Some(domain) = value.strip_prefix("*.") {
                // 简化匹配：检查 URL 是否包含该域名
                if url.contains(domain) {
                    return true;
                }
            } else if url.starts_with(value) {
                return true;
            }
        }

        // 默认拒绝（有指令但没有匹配）
        false
    }

    /// 检查内联脚本是否允许。
    pub fn is_inline_script_allowed(&self) -> bool {
        let directive = self
            .directives
            .iter()
            .find(|d| d.name == "script-src")
            .or_else(|| {
                self.directives
                    .iter()
                    .find(|d| d.name == "default-src")
            });

        let Some(directive) = directive else {
            return true;
        };

        directive
            .values
            .iter()
            .any(|v| v == "'unsafe-inline'" || v == "*")
    }

    /// 检查内联样式是否允许。
    pub fn is_inline_style_allowed(&self) -> bool {
        let directive = self
            .directives
            .iter()
            .find(|d| d.name == "style-src")
            .or_else(|| {
                self.directives
                    .iter()
                    .find(|d| d.name == "default-src")
            });

        let Some(directive) = directive else {
            return true;
        };

        directive
            .values
            .iter()
            .any(|v| v == "'unsafe-inline'" || v == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_csp_is_resource_allowed_self() {
        let csp = ContentSecurityPolicy::parse("default-src 'self'");
        // Relative URL should be allowed with 'self'
        assert!(csp.is_resource_allowed("script", "app.js"));
        // Absolute external URL should be blocked
        assert!(!csp.is_resource_allowed("script", "https://evil.com/script.js"));
    }

    #[test]
    fn test_csp_is_resource_allowed_none() {
        let csp = ContentSecurityPolicy::parse("script-src 'none'");
        assert!(!csp.is_resource_allowed("script", "app.js"));
        assert!(!csp.is_resource_allowed("script", "https://cdn.com/app.js"));
    }

    #[test]
    fn test_csp_inline_script_blocked() {
        let csp = ContentSecurityPolicy::parse("script-src 'self'");
        assert!(!csp.is_inline_script_allowed());
    }

    #[test]
    fn test_csp_inline_script_allowed_unsafe() {
        let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
        assert!(csp.is_inline_script_allowed());
    }

    #[test]
    fn test_csp_empty_policy() {
        let csp = ContentSecurityPolicy::parse("");
        assert!(csp.directives.is_empty());
        // Empty policy allows everything
        assert!(csp.is_resource_allowed("script", "https://any.com/script.js"));
        assert!(csp.is_inline_script_allowed());
        assert!(csp.is_inline_style_allowed());
    }
}
