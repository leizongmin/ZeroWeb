//! 无头协议安全配置（Phase 5）— token 认证与 Origin 校验。

// ── 安全配置（Phase 5）──

/// 无头协议安全配置。
#[derive(Default)]
pub struct HeadlessSecurityConfig {
    /// 认证令牌。如果设置，客户端必须在第一个请求中包含 `token` 字段。
    pub auth_token: Option<String>,
    /// 允许的 Origin 列表。如果为空，允许所有来源。
    pub allowed_origins: Vec<String>,
}

impl HeadlessSecurityConfig {
    /// 创建空安全配置（无限制）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成 128-bit 随机认证令牌（32 hex 字符；OS CSPRNG）。
    ///
    /// renderer-trust-boundary ①a：CDP 默认强制 token——未显式指定
    /// `--remote-debugging-token` 时令牌随机生成，经 stderr 交付操作员、发现端点
    /// `webSocketDebuggerUrl` 交付标准 CDP 客户端（Chromium 同款能力模型，浏览器级
    /// ws URL 内嵌不可猜 UUID）。CSPRNG 不可用即 panic（fail-closed：拒绝以可预测
    /// 令牌暴露自动化面）。
    pub fn generate_auth_token() -> String {
        let mut bytes = [0u8; 16];
        if getrandom::fill(&mut bytes).is_err() {
            panic!("CDP auth token generation failed: OS CSPRNG unavailable");
        }
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// 设置认证令牌。
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// 添加允许的 Origin。
    #[allow(dead_code)]
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// 验证认证令牌。
    pub fn verify_token(&self, provided: Option<&str>) -> bool {
        match &self.auth_token {
            None => true,
            Some(expected) => provided.is_some_and(|p| p == expected),
        }
    }

    /// 验证 WS 升级 URL query 携带的令牌（`?auth_token=<t>`）。
    ///
    /// 标准客户端（playwright `connectOverCDP`、Chrome DevTools frontend）无法在
    /// 首消息注入 `params.token`，能力经发现端点发布的 `webSocketDebuggerUrl`
    /// query 交付（Chromium 同款）；首消息 `params.token` 通路保留（显式 token
    /// 的带外交付形态）。
    pub fn verify_upgrade_query(&self, query: Option<&str>) -> bool {
        let Some(expected) = &self.auth_token else {
            return true;
        };
        query.is_some_and(|query| {
            query.split('&').any(|pair| {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                key == "auth_token" && value == expected
            })
        })
    }

    /// 发现端点 `webSocketDebuggerUrl` 的认证后缀（无 token 时为空串）。
    pub fn discovery_ws_suffix(&self) -> String {
        self.auth_token
            .as_deref()
            .map(|token| format!("?auth_token={token}"))
            .unwrap_or_default()
    }

    /// 验证 WebSocket 请求的 Origin 头。
    pub fn verify_origin(&self, origin: Option<&str>) -> bool {
        if self.allowed_origins.is_empty() {
            return true;
        }
        origin.is_some_and(|o| self.allowed_origins.iter().any(|a| a == o))
    }

    /// 验证 Host 头是否指向本机（IP 字面量或 `localhost`），防 DNS rebinding。
    ///
    /// 浏览器页面发起的跨源连接总携带攻击者域名的 Host（即使其解析到
    /// 127.0.0.1）；CDP 端口若放行，页面即可借本机自动化通道驱动浏览器。
    /// 采 Chromium 同款策略：仅接受 IP 字面量与 `localhost`（可带端口；
    /// IPv6 按 Host 头规范要求带方括号）。Host 缺失时放行——裸 HTTP/1.0
    /// 客户端不发 Host，而 rebinding 攻击必然携带。
    /// https://chromium.googlesource.com/chromium/src/+/refs/heads/main/net/server/http_server/
    pub fn verify_host(&self, host: Option<&str>) -> bool {
        let Some(host) = host else {
            return true;
        };
        let trimmed = host.trim();
        // 整体先按 IP 字面量尝试（含裸 IPv6 如 `::1`——其冒号会干扰端口剥离）。
        if trimmed.parse::<std::net::IpAddr>().is_ok() {
            return true;
        }
        // 去端口：IPv6 字面量形如 `[::1]:9222`；其余取最后一个 `:` 之后的纯数字端口。
        let hostname = if let Some(rest) = trimmed.strip_prefix('[') {
            match rest.split_once(']') {
                Some((ip, _)) => ip,
                None => return false,
            }
        } else {
            match trimmed.rsplit_once(':') {
                Some((ip, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => ip,
                _ => trimmed,
            }
        };
        hostname.parse::<std::net::IpAddr>().is_ok() || hostname.eq_ignore_ascii_case("localhost")
    }
}
