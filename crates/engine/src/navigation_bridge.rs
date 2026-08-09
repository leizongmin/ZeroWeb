//! R3058 JS 发起跨文档导航宿主桥——`location.href = url` / `location.assign(url)` /
//! `location.replace(url)` 经 `__zw_request_navigate` 回调把目标 URL 投递到共享队列，
//! runtime（renderer/browser 主线程）drain 后 `handle_navigate`（fetch 新文档 + 重载）。
//!
//! 旧 shim 仅更新内存 history（`_pushHistNav`/`_replaceHistNav`）——headless 无真文档重载，
//! JS 重定向（登录后跳转 / meta-refresh JS 等价 / 导航菜单）失效。本桥补「真导航」信号通路。
//!
//! 桥本身**不导航**（fetch + 文档重载在 runtime 线程，复用既有 handle_navigate）——worker 线程
//! 回调仅 push URL（同 mutations / font_loads 队列模式），保持 page-load 状态机单线程归属不变。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

/// `__zw_request_navigate` 回调 → 共享队列桥。worker 线程 push，runtime 线程 drain。
pub struct NavigationBridge {
    queue: Arc<Mutex<Vec<String>>>,
}

impl NavigationBridge {
    /// 构造（空队列）。
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 共享队列句柄（runtime 经此 drain——`mem::take` 取出全部 URL）。
    pub fn queue(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.queue)
    }

    /// 注册 `__zw_request_navigate(url)` 回调。worker 线程调，仅 push URL 后返 ""。
    /// 实际 handle_navigate 由 runtime drain 时完成。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let queue = Arc::clone(&self.queue);
        sandbox.register_callback(
            "__zw_request_navigate",
            Box::new(move |args: &[String]| -> String {
                let url = args.first().cloned().unwrap_or_default();
                if !url.is_empty()
                    && let Ok(mut q) = queue.lock()
                {
                    q.push(url);
                }
                String::new()
            }),
        );
    }
}

impl Default for NavigationBridge {
    fn default() -> Self {
        Self::new()
    }
}
