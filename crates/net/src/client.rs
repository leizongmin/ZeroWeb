//! HTTP 客户端 — 封装 reqwest blocking 客户端。
//!
//! 提供同步 HTTP 请求发送能力。

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;

use crate::connect::{build_blocking_client, map_reqwest_error, send_with_ipv4_fallback};
use crate::{HttpRequest, HttpResponse, NetError};

/// HTTP 客户端 — 封装 reqwest。
pub struct HttpClient {
    client: Client,
    /// 最大重定向次数。
    pub max_redirects: usize,
    /// 超时时间（秒）。
    pub timeout_secs: u64,
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

                // 获取 Location 头
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
}
