//! T5/P10 页面 history 遍历宿主桥——`history.back()/forward()/go(delta)` 在 shim 本地栈
//! 越界（跨文档 entry 不在每文档本地栈）时经 `__zw_request_history_traverse` 回调把 delta
//! 投递到共享队列，宿主（renderer runtime / tab_manager）drain 后经既有返回路径提交遍历。
//!
//! 桥本身**不导航**（遍历提交在宿主线程，复用既有返回路径——多进程走
//! `IpcMessageKind::HistoryTraverse` 到主进程 shell 栈，单进程直接调 app 返回路径）——
//! worker 线程回调仅 push delta（同 mutations / navigation 队列模式），保持 page-load
//! 状态机单线程归属不变。delta 越界由宿主 shell 判界，不可达时静默 no-op（R3004）。
//!
//! 规范根据：https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-history-interface
//! ——`back()`/`forward()` 定义为 delta traverse，遍历目标是 traversable navigable 的
//! 会话历史，不是当前文档本地栈。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

/// `__zw_request_history_traverse` 回调 → 共享队列桥。worker 线程 push，宿主线程 drain。
pub struct HistoryBridge {
    queue: Arc<Mutex<Vec<i32>>>,
}

impl HistoryBridge {
    /// 构造（空队列）。
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 共享队列句柄（宿主经此 drain——`mem::take` 取出全部 delta）。
    pub fn queue(&self) -> Arc<Mutex<Vec<i32>>> {
        Arc::clone(&self.queue)
    }

    /// 注册 `__zw_request_history_traverse(delta)` 回调。worker 线程调，仅 push delta 后返 ""。
    /// 实际遍历提交由宿主 drain 时完成。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let queue = Arc::clone(&self.queue);
        sandbox.register_callback(
            "__zw_request_history_traverse",
            Box::new(move |args: &[String]| -> String {
                let delta = args.first().and_then(|s| s.parse::<i32>().ok());
                if let Some(delta) = delta
                    && let Ok(mut q) = queue.lock()
                {
                    q.push(delta);
                }
                String::new()
            }),
        );
    }
}

impl Default for HistoryBridge {
    fn default() -> Self {
        Self::new()
    }
}
