//! HTTP 客户端 — 封装 reqwest blocking 客户端。
//!
//! 提供同步 HTTP 请求发送能力。

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Mutex, OnceLock};

use crate::connect::{build_blocking_client, map_reqwest_error, send_with_ipv4_fallback};
use crate::{HttpRequest, HttpResponse, NetError};

const ASYNC_NETWORK_WORKERS: usize = 4;
const MAX_BLOCKING_NETWORK_TASKS: usize = 32;
type AsyncClientCache = HashMap<(u64, bool, bool), reqwest::Client>;

/// 共享异步网络 runtime，避免资源调度器为每个请求创建线程或 runtime。
pub(crate) fn async_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(ASYNC_NETWORK_WORKERS)
            .max_blocking_threads(MAX_BLOCKING_NETWORK_TASKS)
            .build()
            .expect("create async network runtime")
    })
}

/// 在共享且有上限的网络 blocking pool 中运行短暂的接收/转换任务。
///
/// 仅用于等待同步 `mpsc` receiver；网络传输本身必须继续使用异步 API。
pub fn spawn_network_bridge<F>(task: F)
where
    F: FnOnce() + Send + 'static,
{
    async_runtime().spawn_blocking(task);
}

fn async_client(timeout_secs: u64) -> Result<reqwest::Client, NetError> {
    static CLIENTS: OnceLock<Mutex<AsyncClientCache>> = OnceLock::new();
    let no_proxy = crate::connect::no_proxy_enabled();
    let http2 = crate::connect::http2_enabled();
    let clients = CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut clients = clients.lock().expect("async HTTP client cache lock");
    if let Some(client) = clients.get(&(timeout_secs, no_proxy, http2)) {
        return Ok(client.clone());
    }
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(HttpClient::default_user_agent());
    if no_proxy {
        builder = builder.no_proxy();
    }
    if !http2 {
        builder = builder.http1_only();
    }
    let client = builder.build().map_err(map_reqwest_error)?;
    clients.insert((timeout_secs, no_proxy, http2), client.clone());
    Ok(client)
}

/// HTTP 客户端 — 封装 reqwest。
pub struct HttpClient {
    client: Client,
    /// 最大重定向次数。
    pub max_redirects: usize,
    /// 超时时间（秒）。
    pub timeout_secs: u64,
}

/// 流式 HTTP 响应的元数据；响应体通过回调逐块交付。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponseHead {
    /// HTTP 状态码。
    pub status_code: u16,
    /// 响应头。
    pub headers: Vec<(String, String)>,
    /// 最终响应 URL。
    pub url: String,
    /// 已跟随的重定向数。
    pub redirect_count: usize,
    /// 协商的 HTTP 协议版本。
    pub protocol: String,
}

struct AsyncHttpResponse {
    response: HttpResponse,
    protocol: String,
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            max_redirects: self.max_redirects,
            timeout_secs: self.timeout_secs,
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// 创建新的 HTTP 客户端，使用默认配置。
    pub fn new() -> Self {
        Self::with_config(30, 10)
    }

    /// 异步传输入口。
    ///
    /// P2 迁移的接缝：调用方在 Tokio runtime 中可避免为每个请求创建阻塞 worker。
    pub async fn send_async(&self, request: HttpRequest) -> Result<HttpResponse, NetError> {
        Ok(
            Self::send_async_with_config(self.timeout_secs, self.max_redirects, request)
                .await?
                .response,
        )
    }

    /// 异步流式发送请求，不在客户端中聚合响应体。
    ///
    /// 回调在 async runtime 线程执行，应快速处理或自行转交数据。重定向、方法转换与敏感
    /// 头剥离语义与 [`Self::send_async`] 保持一致。
    pub async fn send_async_stream<F>(
        &self,
        request: HttpRequest,
        mut on_chunk: F,
    ) -> Result<HttpResponseHead, NetError>
    where
        F: FnMut(&[u8]),
    {
        let client = async_client(self.timeout_secs)?;
        let mut current_url = request.url.clone();
        let mut method = request.method.clone();
        let mut body = request.body.clone();
        let mut redirect_count = 0;
        let mut active_headers = request.headers.clone();

        loop {
            let mut builder = client.request(method.to_reqwest(), &current_url);
            for (name, value) in &active_headers {
                builder = builder.header(name, value);
            }
            if let Some(body) = body.clone() {
                builder = builder.body(body);
            }
            let mut response = builder.send().await.map_err(map_reqwest_error)?;
            let status_code = response.status().as_u16();
            if matches!(status_code, 301 | 302 | 303 | 307 | 308) {
                redirect_count += 1;
                if redirect_count > self.max_redirects {
                    return Err(NetError::TooManyRedirects);
                }
                let Some(location) = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                else {
                    return Err(NetError::Http(format!(
                        "{status_code} redirect without Location header"
                    )));
                };
                current_url = url::Url::parse(&current_url)
                    .and_then(|base| base.join(location))
                    .map(|url| url.to_string())
                    .map_err(|error| NetError::Http(format!("invalid redirect URL: {error}")))?;
                if status_code == 303
                    || ((status_code == 301 || status_code == 302) && method == crate::HttpMethod::Post)
                {
                    method = crate::HttpMethod::Get;
                    body = None;
                }
                if body.is_none() {
                    active_headers.retain(|(name, _)| {
                        !matches!(name.to_ascii_lowercase().as_str(), "content-type" | "content-length")
                    });
                }
                if !same_origin(&current_url, &request.url) {
                    active_headers.retain(|(name, _)| {
                        !Self::SENSITIVE_HEADERS
                            .iter()
                            .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
                    });
                }
                continue;
            }

            let head = HttpResponseHead {
                status_code,
                headers: response
                    .headers()
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                    .collect(),
                url: response.url().to_string(),
                redirect_count,
                protocol: protocol_name(response.version()).to_string(),
            };
            while let Some(chunk) = response.chunk().await.map_err(map_reqwest_error)? {
                on_chunk(&chunk);
            }
            return Ok(head);
        }
    }

    /// 异步预热 HTTP/1.1/2 连接池。
    ///
    /// reqwest 未公开可复用的裸 TCP/TLS 预连接接口，因此以同一 async client 的无凭据 `HEAD`
    /// 请求预热连接。该请求不会经过 HTTP 缓存，且调用方应将失败视为非致命。
    pub async fn preconnect_async(&self, origin: &str) -> Result<HttpResponseHead, NetError> {
        let parsed = url::Url::parse(origin).map_err(|error| NetError::UrlParse(error.to_string()))?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(NetError::UrlParse(format!(
                "preconnect requires an HTTP(S) origin: {origin}"
            )));
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(NetError::UrlParse(
                "preconnect origin must not contain credentials".to_string(),
            ));
        }
        let authority = match parsed.port() {
            Some(port) => format!("{}:{port}", parsed.host_str().expect("validated host")),
            None => parsed.host_str().expect("validated host").to_string(),
        };
        let request = HttpRequest {
            method: crate::HttpMethod::Head,
            url: format!("{}://{authority}/", parsed.scheme()),
            headers: Vec::new(),
            body: None,
        };
        self.send_async_stream(request, |_| {}).await
    }

    /// 非阻塞提交连接预热；接收端获得成功的响应头或网络错误。
    pub fn preconnect(&self, origin: impl Into<String>) -> Receiver<Result<HttpResponseHead, NetError>> {
        let client = self.clone();
        let origin = origin.into();
        let (tx, rx) = channel();
        async_runtime().spawn(async move {
            let _ = tx.send(client.preconnect_async(&origin).await);
        });
        rx
    }

    /// 异步预解析 HTTP(S) origin 的 DNS，不建立 HTTP 连接。
    pub async fn dns_prefetch_async(&self, origin: &str) -> Result<(), NetError> {
        let url = url::Url::parse(origin)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(NetError::UrlParse(format!(
                "dns-prefetch requires an HTTP(S) origin: {origin}"
            )));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(NetError::UrlParse(
                "dns-prefetch origin must not contain credentials".to_string(),
            ));
        }
        let host = url.host_str().expect("validated host").to_string();
        let port = url.port_or_known_default().expect("HTTP(S) has a default port");
        tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|error| NetError::Network(format!("DNS lookup failed: {error}")))?
            .next()
            .ok_or_else(|| NetError::Network("DNS lookup returned no addresses".to_string()))?;
        Ok(())
    }

    /// 非阻塞提交 DNS 预取；接收端获得成功或网络错误。
    pub fn dns_prefetch(&self, origin: impl Into<String>) -> Receiver<Result<(), NetError>> {
        let client = self.clone();
        let origin = origin.into();
        let (tx, rx) = channel();
        async_runtime().spawn(async move {
            let _ = tx.send(client.dns_prefetch_async(&origin).await);
        });
        rx
    }

    /// 不依赖 blocking client 状态的异步请求实现，可安全在 Tokio task 内调用。
    pub async fn send_async_with_timeout(timeout_secs: u64, request: HttpRequest) -> Result<HttpResponse, NetError> {
        Ok(Self::send_async_with_config(timeout_secs, 10, request).await?.response)
    }

    /// 供调度器写入匿名实际协议 telemetry 的异步请求入口。
    pub(crate) async fn send_async_with_timeout_and_protocol(
        timeout_secs: u64,
        request: HttpRequest,
    ) -> Result<(HttpResponse, String), NetError> {
        let response = Self::send_async_with_config(timeout_secs, 10, request).await?;
        Ok((response.response, response.protocol))
    }

    async fn send_async_with_config(
        timeout_secs: u64,
        max_redirects: usize,
        request: HttpRequest,
    ) -> Result<AsyncHttpResponse, NetError> {
        let client = async_client(timeout_secs)?;
        let mut current_url = request.url.clone();
        let mut method = request.method.clone();
        let mut body = request.body.clone();
        let mut redirect_count = 0;
        let mut active_headers = request.headers.clone();

        loop {
            let mut builder = client.request(method.to_reqwest(), &current_url);
            for (name, value) in &active_headers {
                builder = builder.header(name, value);
            }
            if let Some(body) = body.clone() {
                builder = builder.body(body);
            }
            let response = builder.send().await.map_err(map_reqwest_error)?;
            let status_code = response.status().as_u16();

            if matches!(status_code, 301 | 302 | 303 | 307 | 308) {
                redirect_count += 1;
                if redirect_count > max_redirects {
                    return Err(NetError::TooManyRedirects);
                }
                let Some(location) = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                else {
                    return Err(NetError::Http(format!(
                        "{status_code} redirect without Location header"
                    )));
                };
                current_url = url::Url::parse(&current_url)
                    .and_then(|base| base.join(location))
                    .map(|url| url.to_string())
                    .map_err(|error| NetError::Http(format!("invalid redirect URL: {error}")))?;
                if status_code == 303
                    || ((status_code == 301 || status_code == 302) && method == crate::HttpMethod::Post)
                {
                    method = crate::HttpMethod::Get;
                    body = None;
                }
                if body.is_none() {
                    active_headers.retain(|(name, _)| {
                        !matches!(name.to_ascii_lowercase().as_str(), "content-type" | "content-length")
                    });
                }
                if !same_origin(&current_url, &request.url) {
                    active_headers.retain(|(name, _)| {
                        !Self::SENSITIVE_HEADERS
                            .iter()
                            .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
                    });
                }
                continue;
            }

            let url = response.url().to_string();
            let protocol = protocol_name(response.version()).to_string();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response.bytes().await.map_err(map_reqwest_error)?.to_vec();
            return Ok(AsyncHttpResponse {
                response: HttpResponse {
                    status_code,
                    headers,
                    body,
                    url,
                    redirect_count,
                },
                protocol,
            });
        }
    }

    /// 创建指定超时时间的 HTTP 客户端。
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self::with_config(timeout_secs, 10)
    }

    /// 创建指定最大重定向次数的 HTTP 客户端。
    pub fn with_max_redirects(max: usize) -> Self {
        Self::with_config(30, max)
    }

    /// HTTP 客户端默认 User-Agent 平台前缀。
    #[cfg(target_os = "macos")]
    const DEFAULT_USER_AGENT_PREFIX: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";
    /// HTTP 客户端默认 User-Agent 平台前缀。
    #[cfg(target_os = "windows")]
    const DEFAULT_USER_AGENT_PREFIX: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";
    /// HTTP 客户端默认 User-Agent 平台前缀。
    #[cfg(target_os = "linux")]
    const DEFAULT_USER_AGENT_PREFIX: &str =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";
    /// HTTP 客户端默认 User-Agent 平台前缀。
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    const DEFAULT_USER_AGENT_PREFIX: &str =
        "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";

    /// 跨域重定向时应剥离的敏感请求头。
    const SENSITIVE_HEADERS: &[&str] = &[
        "authorization",
        "cookie",
        "cookie2",
        "www-authenticate",
        "proxy-authorization",
    ];

    /// 使用完整配置创建 HTTP 客户端。
    fn with_config(timeout_secs: u64, max_redirects: usize) -> Self {
        let user_agent = Self::default_user_agent();
        let client = build_blocking_client(&user_agent, timeout_secs);

        Self {
            client,
            max_redirects,
            timeout_secs,
        }
    }

    /// 构造包含产品构建日期版本的默认 User-Agent。
    pub fn default_user_agent() -> String {
        format!(
            "{} ZeroWeb/{}",
            Self::DEFAULT_USER_AGENT_PREFIX,
            zero_product_version::VERSION
        )
    }

    /// 发送 HTTP 请求，自动处理重定向。
    ///
    /// 支持 301/302/303/307/308 重定向，跟踪重定向次数并在响应中记录。
    /// 超过最大重定向次数时返回 `NetError::TooManyRedirects`。
    pub fn send(&self, request: HttpRequest) -> Result<HttpResponse, NetError> {
        let mut current_url = request.url.clone();
        let mut method = request.method.clone();
        let mut body = request.body.clone();
        let mut redirect_count: usize = 0;
        let mut active_headers: Vec<(String, String)> = request.headers.clone();

        loop {
            let reqwest_method = method.to_reqwest();

            // 添加请求头
            let mut header_map = HeaderMap::new();
            for (name, value) in &active_headers {
                let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| NetError::Http(format!("invalid header name: {e}")))?;
                let header_value = reqwest::header::HeaderValue::from_bytes(value.as_bytes())
                    .map_err(|e| NetError::Http(format!("invalid header value: {e}")))?;
                header_map.append(header_name, header_value);
            }

            let response =
                send_with_ipv4_fallback(&self.client, reqwest_method, &current_url, &header_map, body.as_ref())
                    .map_err(map_reqwest_error)?;

            let status_code = response.status().as_u16();

            // 检查是否为重定向状态码
            if matches!(status_code, 301 | 302 | 303 | 307 | 308) {
                redirect_count += 1;
                if redirect_count > self.max_redirects {
                    return Err(NetError::TooManyRedirects);
                }

                // 获取 Location 头（httparse 已剥离前后 OWS，无需额外 trim）
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let Some(location) = location else {
                    return Err(NetError::Http(format!(
                        "{status_code} redirect without Location header"
                    )));
                };

                // 解析 Location（可能是相对 URL）
                current_url = url::Url::parse(&current_url)
                    .and_then(|base| base.join(&location))
                    .map(|u| u.to_string())
                    .map_err(|e| NetError::Http(format!("invalid redirect URL: {e}")))?;

                // 303 将方法改为 GET 并清除 body
                if status_code == 303 {
                    method = crate::HttpMethod::Get;
                    body = None;
                }
                // 301/302 对非 POST 请求保持原方法；POST 改为 GET（浏览器行为）
                if (status_code == 301 || status_code == 302) && method == crate::HttpMethod::Post {
                    method = crate::HttpMethod::Get;
                    body = None;
                }
                // 307/308 保持原方法和 body
                // R3339：body 被清除（303 / 301+302 POST→GET）时，同步剥离 Content-Type / Content-Length
                // 头——带 Content-Type 但无 body 的 GET 对接收方 malformed（Fetch 标准重定向：清 body 须清
                // body 相关头）。此前仅清 body 不清头 → 重定向 GET 残留 POST 的 content-type（真浏览器会剥）。
                if body.is_none() {
                    active_headers.retain(|(name, _)| {
                        !matches!(name.to_ascii_lowercase().as_str(), "content-type" | "content-length")
                    });
                }

                // SEC-03: 跨域重定向时剥离敏感头（Authorization、Cookie 等）
                if !same_origin(&current_url, &request.url) {
                    active_headers
                        .retain(|(name, _)| !Self::SENSITIVE_HEADERS.iter().any(|h| name.eq_ignore_ascii_case(h)));
                }

                continue;
            }

            // 非重定向响应 — 转换并返回
            let url = response.url().to_string();
            let headers: Vec<(String, String)> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let resp_body = response.bytes().map_err(|e| NetError::Network(e.to_string()))?;
            let resp_body = resp_body.to_vec();

            return Ok(HttpResponse {
                status_code,
                headers,
                body: resp_body,
                url,
                redirect_count,
            });
        }
    }

    /// GET 请求。
    pub fn get(&self, url: &str) -> Result<HttpResponse, NetError> {
        self.get_with_headers(url, &[])
    }

    /// 带额外请求头的 GET（用于条件再验证等）。
    pub fn get_with_headers(&self, url: &str, headers: &[(String, String)]) -> Result<HttpResponse, NetError> {
        if crate::is_file_url(url) {
            return crate::read_file_url(url);
        }
        let mut req = HttpRequest::get(url);
        for (name, value) in headers {
            req = req.header(name, value);
        }
        self.send(req)
    }

    /// POST 请求。
    pub fn post(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse, NetError> {
        self.send(HttpRequest::post(url, body))
    }
}

fn protocol_name(version: reqwest::Version) -> &'static str {
    match version {
        reqwest::Version::HTTP_09 => "http/0.9",
        reqwest::Version::HTTP_10 => "http/1.0",
        reqwest::Version::HTTP_11 => "http/1.1",
        reqwest::Version::HTTP_2 => "h2",
        reqwest::Version::HTTP_3 => "h3",
        _ => "unknown",
    }
}

/// 比较两个 URL 是否同源（scheme + host + port）。
fn same_origin(url_a: &str, url_b: &str) -> bool {
    let parse = |u: &str| -> Option<(String, String, u16)> {
        let parsed = url::Url::parse(u).ok()?;
        let scheme = parsed.scheme().to_string();
        let host = parsed.host_str()?.to_string();
        let port = parsed.port_or_known_default()?;
        Some((scheme, host, port))
    };
    parse(url_a) == parse(url_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert_eq!(client.timeout_secs, 30);
        assert_eq!(client.max_redirects, 10);
    }

    #[test]
    fn test_http_client_with_timeout() {
        let client = HttpClient::with_timeout(60);
        assert_eq!(client.timeout_secs, 60);
        assert_eq!(client.max_redirects, 10);
    }

    #[test]
    fn test_http_client_config() {
        let client = HttpClient::with_max_redirects(5);
        assert_eq!(client.max_redirects, 5);
        assert_eq!(client.timeout_secs, 30);

        let client2 = HttpClient::with_timeout(120);
        assert_eq!(client2.timeout_secs, 120);
        assert_eq!(client2.max_redirects, 10);
    }

    #[test]
    fn test_http_client_default() {
        let client = HttpClient::default();
        assert_eq!(client.timeout_secs, 30);
        assert_eq!(client.max_redirects, 10);
    }

    // ── Integration tests using wiremock ──

    /// Helper: build a tokio runtime and run a wiremock server for one test.
    /// Since HttpClient uses reqwest::blocking, we can't share the same tokio
    /// runtime easily. Instead we use a standalone integration test file.
    /// These tests verify error path mapping without network access.

    #[test]
    fn test_send_invalid_url_returns_network_error() {
        let client = HttpClient::new();
        let req = HttpRequest::get("http://0.0.0.0:1");
        let result = client.send(req);
        assert!(result.is_err());
        // Should be a Network error (connection refused)
        match result.unwrap_err() {
            NetError::Network(msg) => {
                assert!(msg.contains("connection refused") || msg.contains("error"));
            }
            NetError::Timeout => {
                // Also acceptable: could timeout trying to connect
            }
            other => panic!("expected Network or Timeout error, got: {other:?}"),
        }
    }

    #[test]
    fn test_send_invalid_header_name() {
        let client = HttpClient::new();
        let req = HttpRequest::get("http://example.com/").header("Bad Header", "value");
        // This will fail because "Bad Header" is not a valid header name
        let result = client.send(req);
        assert!(result.is_err());
        match result.unwrap_err() {
            NetError::Http(msg) => {
                assert!(msg.contains("invalid header name"));
            }
            other => panic!("expected Http error for bad header name, got: {other:?}"),
        }
    }
}

/// Integration tests for HttpClient::send() using raw TCP mock servers.
///
/// These tests use minimal HTTP/1.1 TCP servers to avoid tokio runtime conflicts
/// with reqwest::blocking.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::{Read, Write};

    /// 返回状态码对应的原因短语。
    fn reason_phrase(status: u16) -> &'static str {
        match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        }
    }

    /// Accept one connection, drain the request, and send a canned response.
    fn respond_once(listener: &std::net::TcpListener, status: u16, extra_headers: &str, body: &str) {
        let mut stream = listener.incoming().next().unwrap().unwrap();
        let mut buf = [0u8; 8192];
        let _ = stream.read(&mut buf);

        let reason = reason_phrase(status);
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\n{extra_headers}Content-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }

    /// Bind a random port and return (listener, base URL).
    fn bind_server() -> (std::net::TcpListener, String) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        (listener, format!("http://127.0.0.1:{port}"))
    }

    /// 本地 mock 服务端测试用：在 `NetError::Network`（连接级瞬态错误）上重试整个请求，
    /// 吸收 nextest/cargo-test 高并发下 `bind_server()` TcpListener accept 竞态导致的偶发 connect 失败
    ///（本地隔离跑 5/5 全绿；并发负载下偶发 `Network("error sending request for url ...")`，非真回归）。
    /// 仅重试 Network（连接级）——Timeout/Proxy/TooManyRedirects/Http 等立即返回，不掩盖断言失败或真实超时。
    /// 调用方需保证 mock 服务端**路径幂等**（按请求路径而非连接序号响应），使整请求重试语义安全。
    fn send_with_local_retry<F>(mut send: F) -> Result<HttpResponse, NetError>
    where
        F: FnMut() -> Result<HttpResponse, NetError>,
    {
        const MAX_ATTEMPTS: u32 = 5;
        let mut last = None;
        for attempt in 0..MAX_ATTEMPTS {
            match send() {
                Err(e @ NetError::Network(_)) => {
                    last = Some(e);
                    std::thread::sleep(std::time::Duration::from_millis(20 * (attempt as u64 + 1)));
                }
                other => return other,
            }
        }
        Err(last.expect("retry loop executes the body at least once"))
    }

    /// GET 请求成功返回 200，验证状态码和响应体。
    #[test]
    fn test_send_get_200() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 200, "Content-Type: text/plain\r\n", "hello world");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 200);
        assert!(resp.is_success());
        assert_eq!(resp.text().unwrap(), "hello world");
        assert!(resp.content_type().is_some());
        assert_eq!(resp.redirect_count, 0);
    }

    /// 默认请求应发送网站兼容的 Chromium User-Agent。
    #[test]
    fn test_send_uses_chromium_compatible_user_agent() {
        let user_agent = HttpClient::default_user_agent();
        assert!(user_agent.contains("Chrome/151.0.0.0"));
        assert!(
            user_agent.ends_with(&format!("ZeroWeb/{}", zero_product_version::VERSION)),
            "User-Agent should expose the product build version"
        );

        let (listener, url) = bind_server();

        let server = std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            let expected = format!("user-agent: {}", HttpClient::default_user_agent());
            assert!(
                request.contains(&expected),
                "request should contain Chromium-compatible User-Agent, got: {request}"
            );

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let response = HttpClient::new().get(&url).unwrap();
        assert_eq!(response.status_code, 200);
        server.join().unwrap();
    }

    /// POST 请求发送 body 并验证响应。
    #[test]
    fn test_send_post_with_body() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 201, "", "created");
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&url, b"request body data".to_vec());
        let resp = client.send(req).unwrap();

        assert_eq!(resp.status_code, 201);
        assert!(resp.is_success());
        assert_eq!(resp.redirect_count, 0);
    }

    /// R3339：POST→GET 重定向（301/302/303）须剥离 body 相关头（Content-Type / Content-Length）。
    /// Spec：Fetch 标准重定向——POST 改 GET 且清 body 时，Content-Type / Content-Length 不应保留
    ///（带 Content-Type 但无 body 的 GET 对接收方是 malformed）。此前 `client.rs` 153-162 行
    /// 仅清 `body=None`，未清 `active_headers` 的 Content-Type/Content-Length → 重定向 GET 仍带 POST 的
    /// content-type（真浏览器会剥）。本测试复现：POST with Content-Type → 302 → GET 不应带 Content-Type。
    #[test]
    fn test_post_to_get_redirect_strips_body_headers_r3339() {
        let (listener, url) = bind_server();

        let server = std::thread::spawn(move || {
            // 读完整请求头（Windows TCP 分片下单次 read 常只读到部分请求，
            // 服务器提前响应会致客户端 send 失败重试整个 POST，污染捕获断言）。
            let read_full_headers = |stream: &mut std::net::TcpStream| -> Vec<u8> {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 1024];
                while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = stream.read(&mut chunk).unwrap();
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                buf
            };
            // 第 1 个连接：POST → 302 重定向到 /dest（同源）。
            {
                let mut stream = listener.incoming().next().unwrap().unwrap();
                let _ = read_full_headers(&mut stream);
                let resp = "HTTP/1.1 302 Found\r\nLocation: /dest\r\nContent-Length: 0\r\n\r\n";
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
            // 第 2 个连接：重定向后的 GET → 捕获请求，断言 Content-Type 不应出现。
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let captured = read_full_headers(&mut stream);
            let captured_get = String::from_utf8_lossy(&captured).to_string();
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
            captured_get
        });

        let client = HttpClient::with_max_redirects(5);
        let req = HttpRequest::post(&url, br#"{"k":"v"}"#.to_vec()).header("content-type", "application/json");
        let resp = send_with_local_retry(|| client.send(req.clone()));
        let _ = resp.expect("POST→GET 重定向应成功");
        let captured = server.join().expect("server thread");

        // 第 2 请求须是 GET（非 POST）——方法已转。
        assert!(
            captured.starts_with("GET /dest"),
            "重定向后应为 GET /dest，got: {}",
            captured.lines().next().unwrap_or("")
        );
        // R3339 核心：Content-Type 不应泄漏到重定向 GET（body 已清，content-type 无意义且 malformed）。
        let lower = captured.to_ascii_lowercase();
        assert!(
            !lower.contains("content-type:"),
            "POST→GET 重定向 GET 不应带 Content-Type（body 已清），captured:\n{captured}"
        );
    }

    /// 验证 404 响应正确解析（非成功状态码）。
    #[test]
    fn test_send_404() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 404, "", "not found");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 404);
        assert!(!resp.is_success());
        assert_eq!(resp.text().unwrap(), "not found");
    }

    /// 验证 500 响应正确解析。
    #[test]
    fn test_send_500() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 500, "", "internal error");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 500);
        assert!(!resp.is_success());
    }

    /// 验证 401/403 状态码正确解析。
    #[test]
    fn test_send_401_403() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();

        let h1 = std::thread::spawn(move || {
            respond_once(&l1, 401, "WWW-Authenticate: Basic\r\n", "unauthorized");
        });
        let h2 = std::thread::spawn(move || {
            respond_once(&l2, 403, "", "forbidden");
        });

        let client = HttpClient::new();
        let r1 = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(r1.status_code, 401);
        assert!(!r1.is_success());

        let r2 = client.send(HttpRequest::get(&url2)).unwrap();
        assert_eq!(r2.status_code, 403);
        assert!(!r2.is_success());

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证响应头正确解析。
    #[test]
    fn test_send_response_headers() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 200, "X-Response-Header: header-value\r\n", "ok");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        let header_val = resp
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-response-header"));
        assert!(header_val.is_some());
        assert_eq!(header_val.unwrap().1, "header-value");
    }

    /// 验证 POST 请求不带 body 也能成功发送。
    #[test]
    fn test_send_post_no_body() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 200, "", "ok");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Post,
            url,
            headers: Vec::new(),
            body: None,
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 PUT 请求方法正确映射。
    #[test]
    fn test_send_put_method() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 200, "", "ok");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Put,
            url,
            headers: Vec::new(),
            body: Some(b"put data".to_vec()),
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 DELETE 请求方法正确映射。
    #[test]
    fn test_send_delete_method() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(&listener, 204, "", "");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Delete,
            url,
            headers: Vec::new(),
            body: None,
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 204);
    }

    /// 验证自定义请求头正确发送到服务端。
    #[test]
    fn test_send_custom_headers_received() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            // reqwest lowercases header names
            assert!(
                request_str.contains("x-custom: test-value"),
                "request should contain x-custom header, got: {request_str}"
            );

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::get(&url).header("X-Custom", "test-value");
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 POST body 正确发送到服务端。
    #[test]
    fn test_send_post_body_received() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.contains("hello from test"),
                "request should contain body, got: {request_str}"
            );
            assert!(
                request_str.starts_with("POST"),
                "request should be POST, got: {request_str}"
            );

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&url, b"hello from test".to_vec());
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 302 重定向后 URL 更新，redirect_count 记录正确。
    #[test]
    fn test_send_redirect_302_updates_url() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/final");

        let target_clone = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = format!("HTTP/1.1 302 Found\r\nLocation: {target_clone}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = "final page";
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}", body.len());
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::with_max_redirects(5);
        let resp = client.send(HttpRequest::get(&url1)).unwrap();

        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/final"));
        assert_eq!(resp.text().unwrap(), "final page");
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 301 永久重定向正确跟随。
    #[test]
    fn test_send_redirect_301() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/moved");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ndone";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/moved"));
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 303 See Other 将 POST 改为 GET。
    #[test]
    fn test_send_redirect_303_changes_post_to_get() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/see-other");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 303 See Other\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            // 303 后方法应变为 GET
            assert!(
                request_str.starts_with("GET"),
                "expected GET after 303, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&url1, b"data".to_vec());
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 307 Temporary Redirect 保持 POST 方法和 body。
    #[test]
    fn test_send_redirect_307_preserves_post() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/temp");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            // 307 应保持 POST
            assert!(
                request_str.starts_with("POST"),
                "expected POST preserved after 307, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&url1, b"payload".to_vec());
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 308 Permanent Redirect 保持原方法。
    #[test]
    fn test_send_redirect_308_preserves_method() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/perm");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 308 Permanent Redirect\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            // 308 应保持 PUT
            assert!(
                request_str.starts_with("PUT"),
                "expected PUT preserved after 308, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Put,
            url: url1,
            headers: Vec::new(),
            body: Some(b"data".to_vec()),
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证多跳重定向（3 次）正常工作。
    #[test]
    fn test_send_redirect_chain() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let (l3, url3) = bind_server();
        let (l4, url4) = bind_server();

        let target2 = format!("{url2}/hop2");
        let target3 = format!("{url3}/hop3");
        let target4 = format!("{url4}/final");

        let t2 = target2.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {t2}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let t3 = target3.clone();
        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {t3}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let t4 = target4.clone();
        let h3 = std::thread::spawn(move || {
            let mut stream = l3.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {t4}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h4 = std::thread::spawn(move || {
            let mut stream = l4.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ndone";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::with_max_redirects(10);
        let resp = client.send(HttpRequest::get(&url1)).unwrap();

        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/final"));
        assert_eq!(resp.redirect_count, 3);

        let _ = h1.join();
        let _ = h2.join();
        let _ = h3.join();
        let _ = h4.join();
    }

    /// 验证超出最大重定向次数时返回 TooManyRedirects 错误。
    #[test]
    fn test_send_redirect_exceeds_max() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/loop");

        // 设置 max_redirects = 0，任何重定向都应失败
        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        // 不需要 h2 因为不会到达
        let _ = std::thread::spawn(move || {
            // 可能有请求到达，也可能不会
            if let Ok(mut stream) = l2.incoming().next().unwrap() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });

        let client = HttpClient::with_max_redirects(0);
        let result = client.send(HttpRequest::get(&url1));
        assert!(result.is_err());
        match result.unwrap_err() {
            NetError::TooManyRedirects => {}
            other => panic!("expected TooManyRedirects, got: {other:?}"),
        }

        let _ = h1.join();
    }

    /// 验证重定向缺少 Location 头时返回错误。
    #[test]
    fn test_send_redirect_without_location() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            // 302 但没有 Location 头
            let resp = "HTTP/1.1 302 Found\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let result = client.send(HttpRequest::get(&url));
        assert!(result.is_err());
        match result.unwrap_err() {
            NetError::Http(msg) => {
                assert!(msg.contains("redirect without Location"), "got: {msg}");
            }
            other => panic!("expected Http error, got: {other:?}"),
        }
    }

    /// 验证相对 URL 重定向正确解析。
    #[test]
    fn test_send_redirect_relative_url() {
        let (listener, url) = bind_server();

        // 路径幂等服务：/original → 302 /other，/other → 200 "relative"。终端响应（/other）后线程退出，
        // 使 send_with_local_retry 在瞬态 connect 失败上重试整个请求时安全（每次从 /original 重新开始，
        // 服务端按路径响应，不依赖连接序号）。吸收并发负载下 bind_server accept 竞态。
        let h = std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut buf = [0u8; 4096];
                if stream.read(&mut buf).is_err() {
                    continue;
                }
                let req = String::from_utf8_lossy(&buf);
                if req.contains("/other") {
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nrelative");
                    let _ = stream.flush();
                    return; // 终端响应后退出 → h.join() 不挂
                } else {
                    let _ = stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /other\r\nContent-Length: 0\r\n\r\n");
                    let _ = stream.flush();
                }
            }
        });

        let client = HttpClient::new();
        let base = url.clone();
        let resp = send_with_local_retry(|| client.send(HttpRequest::get(&format!("{base}/original")))).unwrap();
        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/other"));
        assert_eq!(resp.text().unwrap(), "relative");
        assert_eq!(resp.redirect_count, 1);

        let _ = h.join();
    }

    /// 验证多个响应头正确解析，包括 Content-Type 和自定义头。
    #[test]
    fn test_send_multiple_response_headers() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(
                &listener,
                200,
                "Content-Type: application/json\r\nX-Request-Id: abc123\r\nCache-Control: no-cache\r\n",
                r#"{"status":"ok"}"#,
            );
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.content_type(), Some("application/json"));
        assert_eq!(resp.content_type_mime(), Some("application/json"));
        assert_eq!(resp.header("x-request-id"), Some("abc123"));
        assert_eq!(resp.header("cache-control"), Some("no-cache"));
    }

    /// 验证 Content-Type 带字符集参数的解析。
    #[test]
    fn test_send_content_type_with_charset() {
        let (listener, url) = bind_server();

        std::thread::spawn(move || {
            respond_once(
                &listener,
                200,
                "Content-Type: text/html; charset=utf-8\r\n",
                "<html></html>",
            );
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.content_type(), Some("text/html; charset=utf-8"));
        assert_eq!(resp.content_type_mime(), Some("text/html"));
    }

    /// 验证 POST 请求在 301/302 后变为 GET（浏览器标准行为）。
    #[test]
    fn test_send_post_302_changes_to_get() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/destination");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.starts_with("GET"),
                "POST + 302 should become GET, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::post(&url1, b"body".to_vec())).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 GET 请求在 301 后保持 GET。
    #[test]
    fn test_send_get_301_stays_get() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/new-location");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.starts_with("GET"),
                "GET + 301 should stay GET, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    // ── Additional redirect handling tests ──

    /// 验证 302 POST 变 GET 后 body 被清除。
    #[test]
    fn test_send_post_301_changes_to_get_no_body() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/moved");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.starts_with("GET"),
                "POST + 301 should become GET, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client
            .send(HttpRequest::post(&url1, b"original-body".to_vec()))
            .unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 max_redirects = 1 时，恰好 1 次重定向成功，2 次则失败。
    #[test]
    fn test_send_redirect_max_limit_boundary() {
        // 1 次重定向在 max=1 时应成功
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/ok");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::with_max_redirects(1);
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证重定向到不同 host 正常跟随。
    #[test]
    fn test_send_redirect_to_different_host() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        // 目标是完全不同的地址
        let target = format!("{url2}/cross-origin");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 302 Found\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = "cross-origin";
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}", body.len());
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/cross-origin"));
        assert_eq!(resp.text().unwrap(), "cross-origin");

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 self-referencing 重定向（Location 指向自身）在超限时报错。
    #[test]
    fn test_send_self_referencing_redirect_loop() {
        let (listener, url) = bind_server();
        let self_url = format!("{url}/loop");

        // 路径幂等服务：始终 302 → /loop（自引用）。send_with_local_retry 在瞬态 connect 失败上重试整个
        // 请求（每次 3 跳后 TooManyRedirects）；TooManyRedirects 非 Network 立即返回不重试。服务端 generous
        // 上限（take(24)）覆盖重试最坏请求量；detach（不 join）——成功路径客户端仅发 3 请求，服务端阻塞在
        // 后续 accept，由测试进程退出回收（吸收并发负载下 bind_server accept 竞态；原 join+0..3 与重试不兼容）。
        let su = self_url.clone();
        let _server = std::thread::spawn(move || {
            for stream in listener.incoming().take(24) {
                let Ok(mut stream) = stream else { break };
                let mut buf = [0u8; 4096];
                if stream.read(&mut buf).is_err() {
                    continue;
                }
                let resp = format!("HTTP/1.1 302 Found\r\nLocation: {su}\r\nContent-Length: 0\r\n\r\n");
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });

        let client = HttpClient::with_max_redirects(2);
        let result = send_with_local_retry(|| client.send(HttpRequest::get(&self_url)));
        assert!(result.is_err());
        match result.unwrap_err() {
            NetError::TooManyRedirects => {}
            other => panic!("expected TooManyRedirects for self-loop, got: {other:?}"),
        }
    }

    /// 验证 307 保持 GET 方法不变。
    #[test]
    fn test_send_redirect_307_preserves_get() {
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let target = format!("{url2}/temp-get");

        let tc = target.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: {tc}\r\nContent-Length: 0\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.starts_with("GET"),
                "GET + 307 should stay GET, got: {request_str}"
            );

            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.redirect_count, 1);

        let _ = h1.join();
        let _ = h2.join();
    }

    /// 验证 timeout=0 时请求极快完成或立即超时（边界值测试）。
    #[test]
    fn test_http_client_timeout_zero() {
        let (listener, url) = bind_server();

        let h = std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        });

        // timeout_secs=1：reqwest 的 Duration::from_secs(0) 在部分平台（macos）
        // 当作"无 timeout"导致挂起，用 1s 保证确定行为；mock 快速响应应 1s 内返回 200。
        let client = HttpClient::with_config(1, 5);
        let result = client.send(HttpRequest::get(&url));
        match result {
            Ok(resp) => {
                assert_eq!(resp.status_code, 200, "timeout=0 快速响应时应返回 200");
            }
            Err(NetError::Timeout) => {
                // 零超时立即触发超时也是合理行为
            }
            Err(other) => panic!("expected 200 or Timeout, got: {other:?}"),
        }

        let _ = h.join();
    }

    // ── 高优先级重定向链深度边界测试 ──

    /// 验证恰好 N 次重定向在 max_redirects=N 时成功，N+1 次则失败。
    /// 使用两阶段：先验证 3 次重定向在 max=3 时成功，再验证 2 次在 max=1 时失败。
    #[test]
    fn test_send_redirect_depth_exact_boundary() {
        // ── 阶段 1: 3 次重定向，max=3，应成功 ──
        let (l1, url1) = bind_server();
        let (l2, url2) = bind_server();
        let (l3, url3) = bind_server();
        let (l4, url4) = bind_server();

        let t2 = format!("{url2}/hop2");
        let t3 = format!("{url3}/hop3");
        let t4 = format!("{url4}/final");

        let t2c = t2.clone();
        let h1 = std::thread::spawn(move || {
            let mut s = l1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ =
                s.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {t2c}\r\nContent-Length: 0\r\n\r\n").as_bytes());
            let _ = s.flush();
        });

        let t3c = t3.clone();
        let h2 = std::thread::spawn(move || {
            let mut s = l2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ =
                s.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {t3c}\r\nContent-Length: 0\r\n\r\n").as_bytes());
            let _ = s.flush();
        });

        let t4c = t4.clone();
        let h3 = std::thread::spawn(move || {
            let mut s = l3.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ =
                s.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {t4c}\r\nContent-Length: 0\r\n\r\n").as_bytes());
            let _ = s.flush();
        });

        let h4 = std::thread::spawn(move || {
            let mut s = l4.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ = s.write_all("HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ndone".as_bytes());
            let _ = s.flush();
        });

        let client = HttpClient::with_max_redirects(3);
        let resp = client.send(HttpRequest::get(&url1)).unwrap();
        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/final"));
        assert_eq!(resp.redirect_count, 3, "3 次重定向在 max=3 时应成功");

        let _ = h1.join();
        let _ = h2.join();
        let _ = h3.join();
        let _ = h4.join();

        // ── 阶段 2: 2 次重定向，max=1，应失败 ──
        // l5 返回 302 → l6，l6 返回 302 → /final
        // 客户端收到 l5 的 302（count=1, 1<=1 继续）→ 请求 l6
        // 收到 l6 的 302（count=2, 2>1 失败）→ TooManyRedirects
        // l6 必须能响应（客户端确实会连接 l6），不需要第三个服务器。
        let (l5, url5) = bind_server();
        let (l6, url6) = bind_server();

        let t6 = format!("{url6}/hop");

        let t6c = t6.clone();
        let h5 = std::thread::spawn(move || {
            let mut s = l5.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ =
                s.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {t6c}\r\nContent-Length: 0\r\n\r\n").as_bytes());
            let _ = s.flush();
        });

        let h6 = std::thread::spawn(move || {
            let mut s = l6.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ = s.write_all("HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\n\r\n".as_bytes());
            let _ = s.flush();
        });

        let client = HttpClient::with_max_redirects(1);
        let result = client.send(HttpRequest::get(&url5));
        assert!(result.is_err(), "2 次重定向在 max=1 时应失败");
        match result.unwrap_err() {
            NetError::TooManyRedirects => {}
            other => panic!("expected TooManyRedirects, got: {other:?}"),
        }

        let _ = h5.join();
        let _ = h6.join();
    }

    #[test]
    #[ignore = "uses external network; run manually when validating live HTTPS fetching"]
    fn fetch_https_from_spawned_thread() {
        let handle = std::thread::spawn(|| {
            let client = HttpClient::new();
            client.get("https://example.com").expect("fetch")
        });
        let resp = handle.join().expect("join");
        assert_eq!(resp.status_code, 200);
        assert!(!resp.body.is_empty());
    }

    #[test]
    fn send_async_fetches_local_response() {
        let (listener, url) = bind_server();
        let server = std::thread::spawn(move || {
            respond_once(&listener, 200, "Content-Type: text/plain\r\n", "async hello");
        });
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        let response = runtime
            .block_on(client.send_async(HttpRequest::get(&url)))
            .expect("async fetch");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"async hello");
        server.join().expect("join server");
    }

    #[test]
    fn send_async_stream_delivers_body_without_response_buffer() {
        let (listener, url) = bind_server();
        let server = std::thread::spawn(move || {
            respond_once(&listener, 200, "Content-Type: text/plain\r\n", "streamed body");
        });
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        let mut body = Vec::new();
        let head = runtime
            .block_on(client.send_async_stream(HttpRequest::get(&url), |chunk| body.extend_from_slice(chunk)))
            .expect("stream fetch");
        assert_eq!(head.status_code, 200);
        assert_eq!(head.protocol, "http/1.1");
        assert_eq!(body, b"streamed body");
        server.join().expect("join server");
    }

    #[test]
    fn preconnect_uses_credential_free_head_request() {
        use std::io::{Read, Write};

        let (listener, url) = bind_server();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept preconnect");
            let mut request = [0_u8; 4096];
            let count = stream.read(&mut request).expect("read preconnect request");
            let request = String::from_utf8_lossy(&request[..count]).to_ascii_lowercase();
            assert!(request.starts_with("head / http/1.1\r\n"));
            assert!(!request.contains("authorization:"));
            assert!(!request.contains("cookie:"));
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .expect("write preconnect response");
        });
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        let response = runtime
            .block_on(client.preconnect_async(&url))
            .expect("preconnect response");
        assert_eq!(response.status_code, 204);
        assert_eq!(response.protocol, "http/1.1");
        server.join().expect("join preconnect server");
    }

    #[test]
    fn preconnect_rejects_non_http_or_credentialed_origins() {
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        assert!(matches!(
            runtime.block_on(client.preconnect_async("file:///tmp/nope")),
            Err(NetError::UrlParse(_))
        ));
        assert!(matches!(
            runtime.block_on(client.preconnect_async("https://user:pass@example.test")),
            Err(NetError::UrlParse(_))
        ));
    }

    #[test]
    fn dns_prefetch_resolves_without_an_http_request() {
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        runtime
            .block_on(client.dns_prefetch_async("http://localhost:9"))
            .expect("resolve localhost without connecting to port 9");
    }

    #[test]
    fn dns_prefetch_rejects_non_http_or_credentialed_origins() {
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        assert!(matches!(
            runtime.block_on(client.dns_prefetch_async("file:///tmp/nope")),
            Err(NetError::UrlParse(_))
        ));
        assert!(matches!(
            runtime.block_on(client.dns_prefetch_async("https://user:pass@example.test")),
            Err(NetError::UrlParse(_))
        ));
    }

    #[test]
    fn send_async_follows_relative_redirect() {
        let (listener, url) = bind_server();
        let server = std::thread::spawn(move || {
            respond_once(&listener, 302, "Location: /final\r\n", "");
            respond_once(&listener, 200, "Content-Type: text/plain\r\n", "redirected");
        });
        let client = HttpClient::new();
        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        let response = runtime
            .block_on(client.send_async(HttpRequest::get(&url)))
            .expect("async redirect");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"redirected");
        assert_eq!(response.redirect_count, 1);
        assert!(response.url.ends_with("/final"));
        server.join().expect("join server");
    }
}
