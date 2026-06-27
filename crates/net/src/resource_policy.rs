//! 资源加载策略 — 对齐主流浏览器 HTTP/1.1 每 host 并发连接上限（通常 6）。

/// 环境变量：每 origin 最大并发 HTTP 连接数（正整数）。
pub const ENV_MAX_CONNECTIONS_PER_ORIGIN: &str = "ZERO_MAX_CONNECTIONS_PER_ORIGIN";

/// 主流浏览器对同一 origin 的默认并发连接数（HTTP/1.1）。
pub const DEFAULT_MAX_CONNECTIONS_PER_ORIGIN: usize = 6;

/// 每 origin 最大并发连接数；可通过 [`ENV_MAX_CONNECTIONS_PER_ORIGIN`] 覆盖。
pub fn max_connections_per_origin() -> usize {
    std::env::var(ENV_MAX_CONNECTIONS_PER_ORIGIN)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_MAX_CONNECTIONS_PER_ORIGIN)
}

/// 从 URL 提取 origin（scheme + host + port），用于 per-origin 限流。
pub fn origin_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .map(|u| u.origin().ascii_serialization())
        .unwrap_or_else(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_max_connections_is_six() {
        assert_eq!(max_connections_per_origin(), DEFAULT_MAX_CONNECTIONS_PER_ORIGIN);
    }

    #[test]
    fn origin_from_https_url() {
        assert_eq!(origin_from_url("https://example.com/path?q=1"), "https://example.com");
    }

    #[test]
    fn origin_includes_non_default_port() {
        assert_eq!(
            origin_from_url("http://example.com:8080/foo"),
            "http://example.com:8080"
        );
    }
}
