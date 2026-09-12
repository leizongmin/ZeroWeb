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
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置认证令牌。
    #[allow(dead_code)]
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

    /// 验证 WebSocket 请求的 Origin 头。
    pub fn verify_origin(&self, origin: Option<&str>) -> bool {
        if self.allowed_origins.is_empty() {
            return true;
        }
        origin.is_some_and(|o| self.allowed_origins.iter().any(|a| a == o))
    }
}
