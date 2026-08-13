//! CSP（内容安全策略）模块。
//!
//! 提供 CSP 策略解析和资源加载检查功能。

use crate::origin::Origin;

/// CSP 违规报告回调函数类型。
/// 参数：(url, directive, blocked_uri)
type CspReportCallback = dyn Fn(&str, &str, &str);

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
    ///
    /// R3389：CSP sandbox 指令 token 须按 **ASCII 大小写不敏感**匹配（CSP §6.3.2 sandbox
    /// directive，token 语义继承 HTML iframe sandbox，与 R3388 sandbox.rs 同根）。指令值
    /// 由 parse() 从 header 原样保留大小写喂入（CSP source-list token 保留大小写），故
    /// `sandbox Allow-Scripts` 会真实到达，旧精确 match 静默丢弃 token → 整个沙箱退化为
    /// 最严格，丢失作者意图启用的能力。
    /// 规范引用：https://www.w3.org/TR/CSP3/#directive-sandbox
    fn from_str(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("allow-forms") {
            Some(Self::AllowForms)
        } else if s.eq_ignore_ascii_case("allow-popups") {
            Some(Self::AllowPopups)
        } else if s.eq_ignore_ascii_case("allow-same-origin") {
            Some(Self::AllowSameOrigin)
        } else if s.eq_ignore_ascii_case("allow-scripts") {
            Some(Self::AllowScripts)
        } else if s.eq_ignore_ascii_case("allow-top-navigation") {
            Some(Self::AllowTopNavigation)
        } else if s.eq_ignore_ascii_case("allow-top-navigation-by-user-activation") {
            Some(Self::AllowTopNavigationByUserActivation)
        } else if s.eq_ignore_ascii_case("allow-popups-to-escape-sandbox") {
            Some(Self::AllowPopupsToEscapeSandbox)
        } else if s.eq_ignore_ascii_case("allow-downloads") {
            Some(Self::AllowDownloads)
        } else if s.eq_ignore_ascii_case("allow-presentation") {
            Some(Self::AllowPresentation)
        } else if s.eq_ignore_ascii_case("allow-storage-access-by-user-activation") {
            Some(Self::AllowStorageAccessByUserActivation)
        } else if s.eq_ignore_ascii_case("allow-orientation-lock") {
            Some(Self::AllowOrientationLock)
        } else if s.eq_ignore_ascii_case("allow-pointer-lock") {
            Some(Self::AllowPointerLock)
        } else if s.eq_ignore_ascii_case("allow-autoplay") {
            Some(Self::AllowAutoplay)
        } else if s.eq_ignore_ascii_case("allow-modals") {
            Some(Self::AllowModals)
        } else {
            None
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
                // R3389：CSP 规范「解析序列化 CSP」§2.2.1 第 4 步要求对 directive name 做
                // ASCII 小写化。旧实现保留原大小写，致 `Script-Src 'none'` 这类 mixed-case
                // 指令名在 find_directive 精确比较时被当未知指令丢弃 → 回退 default-src 或
                // 放行 = CSP 绕过（应阻断的脚本被允许）。指令值（source list）保留大小写
                // （'self'/'unsafe-inline' 等 token 自带单引号区分，host 由 origin_expr_matches
                // 已做 eq_ignore_ascii_case）。
                // 规范引用：https://www.w3.org/TR/CSP3/#parse-serialized-csp
                let name = tokens.next()?.to_ascii_lowercase();
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
        self.find_directive(name).or_else(|| self.find_directive("default-src"))
    }

    /// 检查资源加载是否允许。
    ///
    /// `resource_type` 如 "script", "style", "img", "connect", "font", "media"。
    /// `url` 为资源 URL。
    /// `document_origin` 为文档源（用于 'self' 匹配），None 时仅对非绝对 URL 视为同源。
    pub fn is_resource_allowed(&self, resource_type: &str, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive_name = format!("{resource_type}-src");

        let directive = self.find_directive_or_default(&directive_name);

        let Some(directive) = directive else {
            // 没有 default-src 也没有对应指令，默认允许
            return true;
        };

        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查源列表是否匹配给定 URL。
    fn check_source_list(&self, values: &[String], url: &str, document_origin: Option<&Origin>) -> bool {
        // R3389：CSP §6.7.2.7「Matches the source list」——空源列表等价于只含 'none'
        // 的列表，须阻断全部资源（fetch 指令无值即「全禁」）。旧实现空列表返回 true
        // （放行全部）= CSP 绕过：`script-src`（无值）应阻断所有脚本，旧实现却放行
        // 任意脚本。`directive.values` 空 = 指令存在但无源，区别于「无该指令」（由
        // find_directive_or_default 的 None 分支走 default-src/默认放行）。
        // 规范引用：https://www.w3.org/TR/CSP3/#match-source-list
        if values.is_empty() {
            return false;
        }

        // R3389：'none' 仅当**独占**源列表时表示「阻断全部」；与其它源共存时须被忽略
        // （CSP §6.7.2.7 注：'none' is ignored if any other source is present）。旧实现
        // `script-src 'none' 'self'` 会因 'none' 在场而阻断 self——过度阻断（安全方向但
        // 非 spec 语义）。现仅当列表除 'none' 外无其它源时才短路返 false。
        let has_other_source = values.iter().any(|v| v != "'none'");
        if !has_other_source {
            return false;
        }

        // 检查 '*'
        if values.iter().any(|v| v == "*") {
            return true;
        }

        // 检查 'self' — 与文档源匹配
        if values.iter().any(|v| v == "'self'") && Self::is_self_match(url, document_origin) {
            return true;
        }

        // 检查精确 URL 匹配
        if values.iter().any(|v| v == url) {
            return true;
        }

        // 检查 scheme-source（如 "https:"、"data:"、"blob:"）
        for value in values {
            if value.ends_with(':') && !value.starts_with('\'') {
                let scheme = &value[..value.len() - 1];
                if url.starts_with(&format!("{scheme}:")) {
                    return true;
                }
            }
        }

        // 检查通配符域名匹配和源表达式匹配。
        // R3342：旧实现用 `url.starts_with(value)` 做纯字符串前缀匹配——攻击者注册
        // `example.com.evil.com`，`script-src https://example.com` 下其脚本
        // `https://example.com.evil.com/x.js` 因 starts_with 误判被允许（CSP 绕过）。
        // 改为按源表达式（[scheme://]host[:port]）解析后按 host 精确匹配。
        for value in values {
            if let Some(domain) = value.strip_prefix("*.") {
                if Self::wildcard_domain_matches(domain, url) {
                    return true;
                }
            } else if Self::origin_expr_matches(value, url) {
                return true;
            }
        }

        false
    }

    /// R3342：CSP 源表达式匹配——解析 `[scheme://]host[:port]` 形式的源表达式，
    /// 按主机名（及可选的 scheme/port）匹配资源 URL。
    ///
    /// 规范（CSP §8.3 source expression）：源表达式 host 部分须精确匹配资源 URL 的 host，
    /// 绝不前缀匹配。`https://example.com` 只匹配 host 恰为 example.com 的资源；
    /// scheme 缺省时任意 scheme 都接受（host 匹配即可）；port 缺省时任意 port 都接受。
    /// 非 host 形式（如相对路径、含 `/` 的路径表达式）回退到精确整串匹配。
    fn origin_expr_matches(expr: &str, url: &str) -> bool {
        // 取 expr 的 authority 部分（去掉可选 scheme://）。
        let (expr_scheme, authority) = match expr.find("://") {
            Some(pos) => (Some(&expr[..pos]), &expr[pos + 3..]),
            None => (None, expr),
        };
        // authority = host[:port]；若含路径分隔符则不是纯源表达式，回退精确匹配。
        if authority.contains('/') || authority.contains('?') || authority.contains('#') {
            return expr == url;
        }
        let (expr_host, expr_port) = match authority.rfind(':') {
            Some(pos) => (&authority[..pos], Some(&authority[pos + 1..])),
            None => (authority, None),
        };
        if expr_host.is_empty() {
            return expr == url;
        }
        // 解析资源 URL 的 scheme/host/port。
        let Some((url_scheme, url_host, url_port)) = Self::split_url_origin(url) else {
            return expr == url;
        };
        // host 须精确相等（大小写不敏感，规范 host 不区分大小写）。
        if !url_host.eq_ignore_ascii_case(expr_host) {
            return false;
        }
        // scheme 约束（expr 显式给出时须匹配；缺省则任意 scheme 接受）。
        if let Some(es) = expr_scheme
            && !url_scheme.eq_ignore_ascii_case(es)
        {
            return false;
        }
        // port 约束（expr 显式给出时须匹配；缺省则任意 port 接受）。
        if let Some(ep) = expr_port
            && Some(ep) != url_port
        {
            return false;
        }
        true
    }

    /// 从 URL 提取 `(scheme, host, port)`；非 http/https URL 返回 None。
    fn split_url_origin(url: &str) -> Option<(&str, &str, Option<&str>)> {
        let after_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
        let end = after_scheme.find(['/', '?', '#']).unwrap_or(after_scheme.len());
        let authority = &after_scheme[..end];
        let (host, port) = match authority.find(':') {
            // 注意 IPv6 字面量 [::1]:8080 罕见于此 crate 语境，按首个 ':' 分 host/port。
            Some(pos) => (&authority[..pos], Some(&authority[pos + 1..])),
            None => (authority, None),
        };
        let scheme = if url.starts_with("https://") { "https" } else { "http" };
        Some((scheme, host, port))
    }

    /// 判断 URL 是否匹配 'self'（同源）。
    fn is_self_match(url: &str, document_origin: Option<&Origin>) -> bool {
        // data: 和 blob: URI 不匹配 'self'
        if url.starts_with("data:") || url.starts_with("blob:") {
            return false;
        }
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
        let end = after_scheme.find(['/', ':', '?', '#']).unwrap_or(after_scheme.len());
        Some(after_scheme[..end].to_string())
    }

    /// 检查内联脚本是否允许。
    ///
    /// `nonce` 为脚本标签上的 nonce 属性值（不含 'nonce-' 前缀）。
    /// `hash` 为脚本内容的 SHA-256 哈希值（base64 编码，不含 'sha256-' 前缀）。
    pub fn is_inline_script_allowed(&self, nonce: Option<&str>, hash: Option<&str>) -> bool {
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
            if directive.values.iter().any(|v| v == &nonce_quoted || v == &nonce_bare) {
                return true;
            }
        }

        // 检查 hash 匹配（CSP 中格式为 'sha256-<base64>'，含单引号）
        if let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive.values.iter().any(|v| v == &hash_quoted || v == &hash_bare) {
                return true;
            }
        }

        false
    }

    /// 检查内联样式是否允许。
    ///
    /// `nonce` 为样式标签上的 nonce 属性值（不含 'nonce-' 前缀）。
    /// `hash` 为样式内容的 SHA-256 哈希值（base64 编码，不含 'sha256-' 前缀）。
    pub fn is_inline_style_allowed(&self, nonce: Option<&str>, hash: Option<&str>) -> bool {
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
            if directive.values.iter().any(|v| v == &nonce_quoted || v == &nonce_bare) {
                return true;
            }
        }

        // 检查 hash 匹配（CSP 中格式为 'sha256-<base64>'，含单引号）
        if let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive.values.iter().any(|v| v == &hash_quoted || v == &hash_bare) {
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

    // ---- 资源类型便捷方法 ----

    /// 检查 connect-src（Fetch、XHR、WebSocket、EventSource）。
    ///
    /// 回退到 default-src。
    pub fn is_connect_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("connect-src");
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 font-src（@font-face、CSS Font Loading API）。
    ///
    /// 回退到 default-src。
    pub fn is_font_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("font-src");
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 media-src（<audio>、<video>、<track>）。
    ///
    /// 回退到 default-src。
    pub fn is_media_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("media-src");
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 object-src（<object>、<embed>、<applet>）。
    ///
    /// 回退到 default-src。注意：object-src 不匹配 nonce/hash，
    /// 只匹配源表达式。
    pub fn is_object_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("object-src");
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 frame-src（<frame>、<iframe>）。
    ///
    /// 回退顺序：frame-src → child-src → default-src。
    pub fn is_frame_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self
            .find_directive("frame-src")
            .or_else(|| self.find_directive("child-src"))
            .or_else(|| self.find_directive("default-src"));
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 img-src（<img>、<picture>、CSS background-image 等）。
    ///
    /// 回退到 default-src。
    pub fn is_image_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self.find_directive_or_default("img-src");
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 script-src-elem（<script> 元素，不含内联事件处理器）。
    ///
    /// 回退顺序：script-src-elem → script-src → default-src。
    pub fn is_script_element_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self
            .find_directive("script-src-elem")
            .or_else(|| self.find_directive("script-src"))
            .or_else(|| self.find_directive("default-src"));
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查 style-src-elem（<style> 元素、<link rel="stylesheet">）。
    ///
    /// 回退顺序：style-src-elem → style-src → default-src。
    pub fn is_style_element_allowed(&self, url: &str, document_origin: Option<&Origin>) -> bool {
        let directive = self
            .find_directive("style-src-elem")
            .or_else(|| self.find_directive("style-src"))
            .or_else(|| self.find_directive("default-src"));
        let Some(directive) = directive else { return true };
        self.check_source_list(&directive.values, url, document_origin)
    }

    /// 检查是否启用了 upgrade-insecure-requests 指令。
    ///
    /// 启用时，浏览器自动将 HTTP 请求升级为 HTTPS。
    pub fn has_upgrade_insecure_requests(&self) -> bool {
        self.directives.iter().any(|d| d.name == "upgrade-insecure-requests")
    }

    /// 获取 report-uri 指令值（CSP 违规报告地址）。
    ///
    /// 返回报告 URI，如果未设置返回 `None`。
    pub fn report_uri(&self) -> Option<&str> {
        self.find_directive("report-uri")
            .and_then(|d| d.values.first().map(|s| s.as_str()))
    }

    /// 检查 script-src-attr（内联事件处理器如 onclick）。
    ///
    /// 回退顺序：script-src-attr → script-src → default-src。
    pub fn is_script_attr_allowed(&self, nonce: Option<&str>, hash: Option<&str>) -> bool {
        let directive = self
            .find_directive("script-src-attr")
            .or_else(|| self.find_directive("script-src"))
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };

        if directive.values.iter().any(|v| v == "'unsafe-inline'" || v == "*") {
            return true;
        }

        // unsafe-hashes 允许内联事件处理器通过 hash 验证
        let allow_hashes = directive.values.iter().any(|v| v == "'unsafe-hashes'");

        if let Some(n) = nonce {
            let nonce_quoted = format!("'nonce-{n}'");
            let nonce_bare = format!("nonce-{n}");
            if directive.values.iter().any(|v| v == &nonce_quoted || v == &nonce_bare) {
                return true;
            }
        }

        if allow_hashes && let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive.values.iter().any(|v| v == &hash_quoted || v == &hash_bare) {
                return true;
            }
        }

        false
    }

    /// 检查 style-src-attr（内联 style 属性）。
    ///
    /// 回退顺序：style-src-attr → style-src → default-src。
    pub fn is_style_attr_allowed(&self, nonce: Option<&str>, hash: Option<&str>) -> bool {
        let directive = self
            .find_directive("style-src-attr")
            .or_else(|| self.find_directive("style-src"))
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };

        if directive.values.iter().any(|v| v == "'unsafe-inline'" || v == "*") {
            return true;
        }

        let allow_hashes = directive.values.iter().any(|v| v == "'unsafe-hashes'");

        if let Some(n) = nonce {
            let nonce_quoted = format!("'nonce-{n}'");
            let nonce_bare = format!("nonce-{n}");
            if directive.values.iter().any(|v| v == &nonce_quoted || v == &nonce_bare) {
                return true;
            }
        }

        if allow_hashes && let Some(h) = hash {
            let hash_quoted = format!("'sha256-{h}'");
            let hash_bare = format!("sha256-{h}");
            if directive.values.iter().any(|v| v == &hash_quoted || v == &hash_bare) {
                return true;
            }
        }

        false
    }

    /// 检查是否允许 eval()/new Function()（`unsafe-eval`）。
    ///
    /// 回退到 script-src → default-src。
    pub fn is_eval_allowed(&self) -> bool {
        let directive = self
            .find_directive("script-src")
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };

        directive.values.iter().any(|v| v == "'unsafe-eval'" || v == "*")
    }

    /// 检查是否允许 WASM 编译（`wasm-unsafe-eval` 或 `unsafe-eval`）。
    ///
    /// `wasm-unsafe-eval` 单独允许 WebAssembly.compile/instantiate，
    /// 不允许 eval() 和 new Function()。
    /// 回退到 script-src → default-src。
    pub fn is_wasm_eval_allowed(&self) -> bool {
        let directive = self
            .find_directive("script-src")
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return true;
        };

        directive
            .values
            .iter()
            .any(|v| v == "'wasm-unsafe-eval'" || v == "'unsafe-eval'" || v == "*")
    }

    /// 检查是否启用了 `strict-dynamic`（信任传播）。
    ///
    /// 当 strict-dynamic 出现在 script-src 中时，通过 nonce 或 hash
    /// 信任的脚本可以动态加载更多脚本，这些新脚本无需出现在源列表中。
    /// 回退到 script-src → default-src。
    pub fn has_strict_dynamic(&self) -> bool {
        let directive = self
            .find_directive("script-src")
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return false;
        };

        directive.values.iter().any(|v| v == "'strict-dynamic'")
    }

    /// 检查是否启用了 `report-sample`。
    ///
    /// `report-sample` 请求浏览器在违规报告中包含违规资源的一小段样本
    /// （通常 40 个字符），方便调试。适用于 script-src 和 style-src。
    /// 回退到 script-src → default-src。
    pub fn has_report_sample(&self) -> bool {
        let directive = self
            .find_directive("script-src")
            .or_else(|| self.find_directive("style-src"))
            .or_else(|| self.find_directive("default-src"));

        let Some(directive) = directive else {
            return false;
        };

        directive.values.iter().any(|v| v == "'report-sample'")
    }

    /// 获取 report-to 指令值（CSP Level 3 报告组名）。
    ///
    /// 返回报告组名，如果未设置返回 `None`。
    pub fn report_to(&self) -> Option<&str> {
        self.find_directive("report-to")
            .and_then(|d| d.values.first().map(|s| s.as_str()))
    }
}

/// Report-Only CSP — 仅报告不阻止。
///
/// 从 `Content-Security-Policy-Report-Only` 头解析。
/// 所有违规仅发送报告而不阻止资源加载。
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicyReportOnly {
    /// 内部 CSP 策略。
    policy: ContentSecurityPolicy,
}

impl ContentSecurityPolicyReportOnly {
    /// 从 Content-Security-Policy-Report-Only 头值解析。
    pub fn parse(header_value: &str) -> Self {
        Self {
            policy: ContentSecurityPolicy::parse(header_value),
        }
    }

    /// 检查资源加载是否会被阻止（Report-Only 模式始终返回 true）。
    ///
    /// 如果资源违反策略，记录违规报告但不阻止。
    /// `report_callback` 在违规时被调用（url, directive, blocked_uri）。
    pub fn check_resource(
        &self,
        resource_type: &str,
        url: &str,
        document_origin: Option<&Origin>,
        report_callback: Option<&CspReportCallback>,
    ) -> bool {
        let allowed = self.policy.is_resource_allowed(resource_type, url, document_origin);
        if !allowed && let Some(cb) = report_callback {
            let directive_name = format!("{resource_type}-src");
            cb(url, &directive_name, url);
        }
        // Report-Only 永远不阻止
        true
    }

    /// 检查内联脚本是否会被阻止（Report-Only 始终允许）。
    pub fn check_inline_script(
        &self,
        nonce: Option<&str>,
        hash: Option<&str>,
        report_callback: Option<&CspReportCallback>,
    ) -> bool {
        let allowed = self.policy.is_inline_script_allowed(nonce, hash);
        if !allowed && let Some(cb) = report_callback {
            cb("inline", "script-src", "inline-script");
        }
        true
    }

    /// 获取底层策略引用（用于报告生成）。
    pub fn policy(&self) -> &ContentSecurityPolicy {
        &self.policy
    }
}
