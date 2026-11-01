//! renderer-trust-boundary：`ClearHttpCookies` 清空 renderer 本地 cookie jar 回归。
//!
//! jar 由 `ipc_fetch` 的 FetchResponse Set-Cookie 写入、`ws_handshake_cookie` 消费；
//! browser 权威 jar 清理时经本消息同步，否则已清除 cookie 继续附着 WS 握手。

use super::*;

/// jar 为进程全局共享，测试间互斥（其他测试可能经 fetch/WS 路径写入）。
static JAR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn clear_http_cookies_empties_renderer_local_jar() {
    let _guard = JAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut runtime = RendererRuntime::new(9101);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));

    // 预置本地 jar 条目（同 ipc_fetch try_complete 的 Set-Cookie 写入面）。
    {
        let store = zero_net::shared_cookie_store();
        let mut jar = store.lock().unwrap();
        zero_net::store_set_cookie_headers(
            &mut jar,
            "https://zero.test/page",
            &[("Set-Cookie".to_string(), "sid=abc; Path=/".to_string())],
        );
    }
    assert!(!zero_net::shared_cookie_store().lock().unwrap().is_empty());

    runtime.handle_clear_http_cookies().expect("clear 应成功");

    assert!(zero_net::shared_cookie_store().lock().unwrap().is_empty());
}
