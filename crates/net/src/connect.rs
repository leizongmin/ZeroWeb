//! 连接层辅助 — reqwest 客户端构建与 IPv4 优先 DNS（Windows 上 IPv6 不可达时常见）。

use std::error::Error;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::sync::mpsc;

use reqwest::Method;
use reqwest::blocking::{Client, Response};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::header::HeaderMap;

use crate::NetError;

/// 仅返回 IPv4 地址的 DNS 解析器。
#[derive(Debug, Clone, Copy, Default)]
struct Ipv4OnlyResolver;

impl Resolve for Ipv4OnlyResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let (tx, rx) = mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let result = (host.as_str(), 0)
                    .to_socket_addrs()
                    .map(|iter| iter.filter(|a| a.is_ipv4()).collect::<Vec<_>>());
                let _ = tx.send(result);
            });
            let resolved = rx
                .recv()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

            if resolved.is_empty() {
                return Err("no IPv4 address resolved".into());
            }
            Ok(Box::new(resolved.into_iter()) as Addrs)
        })
    }
}

/// 构建 blocking HTTP 客户端（与 `HttpClient` 配置一致）。
pub(crate) fn build_blocking_client(user_agent: &str, timeout_secs: u64) -> Client {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(user_agent)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .dns_resolver(Arc::new(Ipv4OnlyResolver));
    if !http2_enabled() {
        builder = builder.http1_only();
        tracing::info!("HTTP/1.1 only (ZERO_HTTP2=0)");
    }
    builder.build().expect("failed to build HTTP client")
}

/// 默认启用 HTTP/2；设 `ZERO_HTTP2=0` 可退回 HTTP/1.1。
fn http2_enabled() -> bool {
    std::env::var("ZERO_HTTP2")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

/// 将 reqwest 错误映射为 `NetError`；代理相关失败单独归类以便 UI/日志识别。
pub(crate) fn map_reqwest_error(e: reqwest::Error) -> NetError {
    if e.is_timeout() {
        return NetError::Timeout;
    }
    if is_proxy_connect_error(&e) {
        let detail = deepest_error_message(&e);
        return NetError::Proxy(format!("cannot connect via {}{detail}", proxy_source_hint(),));
    }
    NetError::Network(e.to_string())
}

fn is_proxy_connect_error(e: &reqwest::Error) -> bool {
    if !e.is_connect() {
        return false;
    }
    error_chain_matches(e, message_indicates_proxy_failure)
}

fn message_indicates_proxy_failure(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("tunnel") || m.contains("proxy")
}

fn error_chain_matches(e: &reqwest::Error, pred: impl Fn(&str) -> bool) -> bool {
    if pred(&e.to_string()) {
        return true;
    }
    let mut src = e.source();
    while let Some(s) = src {
        if pred(&s.to_string()) {
            return true;
        }
        src = s.source();
    }
    false
}

fn deepest_error_message(e: &reqwest::Error) -> String {
    let mut last = e.to_string();
    let mut src = e.source();
    while let Some(s) = src {
        last = s.to_string();
        src = s.source();
    }
    last
}

fn proxy_source_hint() -> String {
    for key in [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        if let Ok(val) = std::env::var(key)
            && !val.is_empty()
        {
            return format!("proxy ({key}={val}) — ");
        }
    }
    "system proxy — ".to_string()
}

/// 发送 HTTP 请求（经 IPv4 优先解析器）。
pub(crate) fn send_with_ipv4_fallback(
    client: &Client,
    method: Method,
    url: &str,
    headers: &HeaderMap,
    body: Option<&Vec<u8>>,
) -> Result<Response, reqwest::Error> {
    let mut builder = client.request(method, url);
    builder = builder.headers(headers.clone());
    if let Some(b) = body {
        builder = builder.body(b.clone());
    }
    builder.send()
}

#[cfg(test)]
mod tests {
    use super::message_indicates_proxy_failure;

    #[test]
    fn proxy_tunnel_message_is_recognized() {
        assert!(message_indicates_proxy_failure(
            "tunnel error: failed to create underlying connection"
        ));
        assert!(message_indicates_proxy_failure("Proxy Authentication Required"));
        assert!(!message_indicates_proxy_failure("connection refused"));
        assert!(!message_indicates_proxy_failure(
            "error sending request for url (https://example.com/)"
        ));
    }
}
