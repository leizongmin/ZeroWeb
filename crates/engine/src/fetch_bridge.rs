//! P1b S3 fetch bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持 fetch handler cell + [`AsyncResolver`]；`register` 在 sandbox 注 `__zw_fetch` 回调。
//! 各 app 在 `js_worker_main` 构造 `FetchBridge`（传入自身 resolver），调 `register(sandbox)`
//! 注 `__zw_fetch` 回调；`SetFetchHandler` 命令 arm 调 `set_handler` 注入生产 handler。
//! `__zw_fetch` 回调非阻塞——子线程抓取 + `resolver.resolve` 回投（不冻结 JS worker）。
//!
//! `default_fetch_handler`（生产 HTTP GET 经 `zero_webview::fetch_text_async`）由各 app 提供：
//! `zero-engine` 不依赖 `zero-webview`（避免循环依赖），故生产 handler 留在 app 层。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

use crate::async_resolver::AsyncResolver;

/// JS `fetch(url)` 的抓取函数类型（同步返 response body 文本或 error）。
/// 生产由各 app 提供 `default_fetch_handler`（经 net pool 真实 HTTP GET）；测试用合成实现。
pub type FetchHandler = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// P1b S3 fetch bridge——共享 fetch 机制（handler cell + `__zw_fetch` 注册 + 非阻塞抓取）。
///
/// 各 app 在 `js_worker_main` 构造（传入包装自身 resolver 的 `AsyncResolver`），调
/// [`FetchBridge::register`] 注 `__zw_fetch` 回调；app 的 `SetFetchHandler` 命令 arm 调
/// [`FetchBridge::set_handler`] 注入生产 handler。`__zw_fetch` 回调非阻塞——子线程抓取
/// + `resolver.resolve` 回投（不冻结 JS worker）。handler 未注入时子线程 resolve 错误标记。
pub struct FetchBridge {
    handler_cell: Arc<Mutex<Option<FetchHandler>>>,
    resolver: AsyncResolver,
}

impl FetchBridge {
    /// 构造——`resolver` 用于 `__zw_fetch` 抓取完成后 resolve Promise（复用 S1 通路）。
    pub fn new(resolver: AsyncResolver) -> Self {
        Self {
            handler_cell: Arc::new(Mutex::new(None)),
            resolver,
        }
    }

    /// 注入 fetch handler（各 app 的 `SetFetchHandler` 命令 arm 调用）。
    /// chicken-and-egg 解：app 在 js_worker spawn 后（WebView/net pool 就绪后）注入。
    pub fn set_handler(&self, handler: FetchHandler) {
        if let Ok(mut cell) = self.handler_cell.lock() {
            *cell = Some(handler);
        }
    }

    /// 注册 `__zw_fetch(id, url)` 回调——JS `fetch(url)` 经 shim 调此。
    /// **非阻塞**：回调锁内仅克隆 handler Option（`FetchHandler=Arc` 廉价）后立即返，
    /// 子线程 `std::thread::spawn` 抓取（`h(url)` / `fetch_text_async.recv`）+ `resolver.resolve`
    /// 回投——JS worker 不在 fetch 期间冻结。handler 未注入时子线程 resolve 错误标记。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let handler_cell = Arc::clone(&self.handler_cell);
        let resolver = self.resolver.clone();
        sandbox.register_callback(
            "__zw_fetch",
            Box::new(move |args: &[String]| -> String {
                let id = args.first().cloned().unwrap_or_default();
                let url = args.get(1).cloned().unwrap_or_default();
                let handler_opt: Option<FetchHandler> = handler_cell.lock().ok().and_then(|c| c.as_ref().cloned());
                let resolver = resolver.clone();
                std::thread::spawn(move || {
                    let result = match handler_opt {
                        Some(h) => h(&url).unwrap_or_else(|e| format!("__zw_fetch_error:{e}")),
                        None => "__zw_fetch_error:no-handler".to_string(),
                    };
                    resolver.resolve(&id, &result);
                });
                String::new()
            }),
        );
    }
}
