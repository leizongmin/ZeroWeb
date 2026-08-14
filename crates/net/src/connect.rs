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
    if no_proxy_enabled() {
        // 绕过所有代理（系统注册表代理 + HTTP_PROXY/HTTPS_PROXY 等环境变量），
        // 用于诊断代理本身故障或强制直连的场景。
        builder = builder.no_proxy();
        tracing::info!("proxy disabled (ZERO_NOPROXY=1)");
    }
    builder.build().expect("failed to build HTTP client")
}

/// 默认启用 HTTP/2；设 `ZERO_HTTP2=0` 可退回 HTTP/1.1。
///
/// 供浏览器进程的流式 fetch 路径决定是否附加 RFC 9218 `Priority` 请求头。
pub fn http2_enabled() -> bool {
    std::env::var("ZERO_HTTP2")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

/// 默认尊重系统/环境代理；设 `ZERO_NOPROXY=1`（或 `true`，大小写不敏感）可完全绕过代理直连。
pub(crate) fn no_proxy_enabled() -> bool {
    match std::env::var("ZERO_NOPROXY").ok().as_deref() {
        Some("1") => true,
        Some(v) => v.eq_ignore_ascii_case("true"),
        None => false,
    }
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
    // 普通网络错误：若环境代理变量已设置，附在消息里便于排查
    // （reqwest 默认会透明走系统代理，但握手失败常不带 proxy 关键字）。
    let msg = e.to_string();
    if let Some(key) = env_proxy_var() {
        return NetError::Network(format!("{msg} [env {key} set]"));
    }
    NetError::Network(msg)
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

/// 若 HTTP_PROXY / HTTPS_PROXY / ALL_PROXY 任一已设置，返回其变量名（用于错误消息提示）。
fn env_proxy_var() -> Option<&'static str> {
    [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ]
    .into_iter()
    .find(|key| std::env::var(key).map(|v| !v.is_empty()).unwrap_or(false))
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
            // 代理 URL 可能内嵌凭据（如 `http://user:pass@host:port`），错误消息会经
            // NetError::Proxy → tracing 日志 / UI load-failed 透传，直接写入原始值会
            // 把密码泄漏到日志与界面。这里只保留 scheme + host(:port)，剥离 userinfo。
            return format!("proxy ({key}={}) — ", redact_proxy_userinfo(&val));
        }
    }
    "system proxy — ".to_string()
}

/// 剥离代理 URL 中的 userinfo（`user[:pass]@`），保留 scheme 与 host(:port) 用于诊断。
///
/// 例如 `http://bob:s3cr3t@proxy.corp:8080` → `http://***@proxy.corp:8080`，
/// `socks5://host:1080`（无凭据）原样返回。无法解析的值原样返回（保守，不吞诊断信息）。
fn redact_proxy_userinfo(val: &str) -> String {
    // 分离可选的 scheme 前缀（如 `http://`、`socks5://`）。
    let (scheme, rest) = match val.split_once("://") {
        Some((s, r)) => (format!("{s}://"), r),
        None => (String::new(), val),
    };
    // 以首个 `@` 分割 userinfo 与 host（host 部分不应再含 `@`）。
    match rest.split_once('@') {
        // 存在 userinfo → 用占位符替换，保留 host(:port)。
        Some((_, host)) => format!("{scheme}***@{host}"),
        None => val.to_string(),
    }
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
    use super::{env_proxy_var, message_indicates_proxy_failure, no_proxy_enabled, redact_proxy_userinfo};

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

    /// 覆盖 `env_proxy_var`：未设时返回 None，设置后返回变量名。
    /// 注意：该测试通过 `set_var/remove_var` 修改进程级环境变量，
    /// 因此依赖 `RUST_TEST_THREADS=1` 或同模块测试不会并发读这些变量。
    /// 在本模块内，无其他测试会读取代理相关环境变量，故安全。
    #[test]
    fn env_proxy_var_unset_and_set() {
        for key in [
            "HTTPS_PROXY",
            "https_proxy",
            "HTTP_PROXY",
            "http_proxy",
            "ALL_PROXY",
            "all_proxy",
        ] {
            // edition 2024: 修改进程环境变量需要 unsafe。
            unsafe { std::env::remove_var(key) };
        }
        assert!(env_proxy_var().is_none());

        unsafe { std::env::set_var("HTTPS_PROXY", "http://127.0.0.1:7078") };
        assert_eq!(env_proxy_var(), Some("HTTPS_PROXY"));

        unsafe { std::env::remove_var("HTTPS_PROXY") };
    }

    /// 覆盖 `no_proxy_enabled`：未设/乱设时为 false，仅 `1`/`true` 时为 true。
    /// 读写的 `ZERO_NOPROXY` 与其他测试不重叠，无并发风险。
    #[test]
    fn no_proxy_enabled_truthy_values() {
        unsafe { std::env::remove_var("ZERO_NOPROXY") };
        assert!(!no_proxy_enabled());

        for bad in ["0", "false", "no", "yes", ""] {
            unsafe { std::env::set_var("ZERO_NOPROXY", bad) };
            assert!(!no_proxy_enabled(), "ZERO_NOPROXY={bad:?} should be false");
        }

        for good in ["1", "true", "TRUE", "True"] {
            unsafe { std::env::set_var("ZERO_NOPROXY", good) };
            assert!(no_proxy_enabled(), "ZERO_NOPROXY={good:?} should be true");
        }

        unsafe { std::env::remove_var("ZERO_NOPROXY") };
    }

    // ── R3368：代理凭据脱敏（防日志/UI 泄漏）──

    #[test]
    /// R3368：`redact_proxy_userinfo` 剥离 `user:pass@`，保留 scheme + host(:port)。
    ///
    /// `proxy_source_hint` 直接对 env 值调用此函数（connect.rs:162），故此纯函数测试
    /// 即锁定「代理密码不泄漏到 NetError::Proxy → 日志/UI」的安全属性，无需触碰进程级
    /// env 变量（避免与 `env_proxy_var_unset_and_set` 在多线程下并发读 `HTTPS_PROXY` 的竞态）。
    fn redact_proxy_userinfo_strips_credentials_r3368() {
        // user:pass@
        assert_eq!(
            redact_proxy_userinfo("http://bob:s3cr3t@proxy.corp:8080"),
            "http://***@proxy.corp:8080",
        );
        // 仅 user@（无密码，username 仍属敏感）
        assert_eq!(
            redact_proxy_userinfo("http://alice@proxy.corp:8080"),
            "http://***@proxy.corp:8080",
        );
        // socks5 + 凭据
        assert_eq!(
            redact_proxy_userinfo("socks5://u:p@127.0.0.1:1080"),
            "socks5://***@127.0.0.1:1080",
        );
        // 无凭据 → 原样返回
        assert_eq!(
            redact_proxy_userinfo("http://proxy.corp:8080"),
            "http://proxy.corp:8080"
        );
        assert_eq!(redact_proxy_userinfo("socks5://host:1080"), "socks5://host:1080");
        // 无 scheme 但含 userinfo
        assert_eq!(redact_proxy_userinfo("bob:s3cr3t@host:8080"), "***@host:8080");
        // 无 scheme 无凭据
        assert_eq!(redact_proxy_userinfo("host:8080"), "host:8080");
        // 空值原样返回
        assert_eq!(redact_proxy_userinfo(""), "");
    }
}
