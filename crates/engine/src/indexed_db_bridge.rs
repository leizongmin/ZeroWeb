//! IndexedDB 同步宿主桥。
//!
//! `zero-engine` 只定义可信 origin 推导与 wire 契约，不依赖具体存储实现。
//! `zero-page-runtime` 提供基于 `zero-storage` 的 handler。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

/// IndexedDB handler。
///
/// 参数依次为宿主从当前页面 URL 推导的 origin、页面请求 JSON；返回响应 JSON 或错误。
pub type IndexedDbHandler = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

const OK_PREFIX: &str = "__zw_idb_ok:";
const ERROR_PREFIX: &str = "__zw_idb_error:";
const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;

/// IndexedDB 同步 callback bridge。
pub struct IndexedDbBridge {
    handler: IndexedDbHandler,
}

impl IndexedDbBridge {
    /// 使用业务 handler 构造 bridge。
    pub fn new(handler: IndexedDbHandler) -> Self {
        Self { handler }
    }

    /// 注册 `__zw_idb(requestJson)`。
    ///
    /// origin 始终由宿主维护的 `page_url` 推导，页面不能通过 request JSON 伪造。
    pub fn register(&self, sandbox: &mut dyn Sandbox, page_url: &Arc<Mutex<String>>) {
        let handler = Arc::clone(&self.handler);
        let page_url = Arc::clone(page_url);
        sandbox.register_callback(
            "__zw_idb",
            Box::new(move |args: &[String]| -> String {
                let request = args.first().map(String::as_str).unwrap_or("");
                invoke_handler(&handler, &page_url, request)
            }),
        );
    }
}

fn invoke_handler(handler: &IndexedDbHandler, page_url: &Mutex<String>, request: &str) -> String {
    if request.len() > MAX_REQUEST_BYTES {
        return format!("{ERROR_PREFIX}request exceeds 8 MiB");
    }
    let origin = page_url
        .lock()
        .map(|url| indexed_db_origin(&url))
        .unwrap_or_else(|_| "null".to_string());
    serialize_result(handler(&origin, request))
}

/// 从宿主管理的页面 URL 推导 IndexedDB origin。
pub fn indexed_db_origin(page_url: &str) -> String {
    url::Url::parse(page_url)
        .map(|url| url.origin().ascii_serialization())
        .unwrap_or_else(|_| "null".to_string())
}

fn serialize_result(result: Result<String, String>) -> String {
    match result {
        Ok(response) => format!("{OK_PREFIX}{response}"),
        Err(error) => format!("{ERROR_PREFIX}{error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_derived_from_page_url() {
        assert_eq!(indexed_db_origin("https://example.com/path"), "https://example.com");
        assert_eq!(
            indexed_db_origin("https://example.com:8443/path"),
            "https://example.com:8443"
        );
        assert_eq!(indexed_db_origin("about:blank"), "null");
        assert_eq!(indexed_db_origin("not a url"), "null");
    }

    #[test]
    fn result_wire_distinguishes_success_and_error() {
        assert_eq!(
            serialize_result(Ok("{\"version\":1}".to_string())),
            "__zw_idb_ok:{\"version\":1}"
        );
        assert_eq!(
            serialize_result(Err("VersionError".to_string())),
            "__zw_idb_error:VersionError"
        );
    }

    #[test]
    fn callback_uses_host_origin_and_rejects_oversized_requests() {
        let handler: IndexedDbHandler = Arc::new(|origin, request| Ok(format!("{origin}|{request}")));
        let page_url = Mutex::new("https://trusted.example/path".to_string());
        assert_eq!(
            invoke_handler(&handler, &page_url, r#"{"origin":"https://attacker.example"}"#),
            r#"__zw_idb_ok:https://trusted.example|{"origin":"https://attacker.example"}"#
        );

        let oversized = "x".repeat(MAX_REQUEST_BYTES + 1);
        assert_eq!(
            invoke_handler(&handler, &page_url, &oversized),
            "__zw_idb_error:request exceeds 8 MiB"
        );
    }
}
