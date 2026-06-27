//! 测试专用同步 — Tab worker 与全局 engine 测量回调在并行测试中互斥。

#[cfg(test)]
use std::cell::Cell;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(test)]
thread_local! {
    static TAB_RUNTIME_LOCK_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// 串行化 Tab worker / WebView 相关测试（可重入，同一线程 nested 安全）。
#[cfg(test)]
pub struct TabRuntimeTestGuard {
    #[allow(dead_code)]
    lock: Option<MutexGuard<'static, ()>>,
}

#[cfg(test)]
impl TabRuntimeTestGuard {
    fn acquire() -> Self {
        let depth = TAB_RUNTIME_LOCK_DEPTH.with(|d| d.get());
        if depth > 0 {
            TAB_RUNTIME_LOCK_DEPTH.with(|d| d.set(depth + 1));
            return Self { lock: None };
        }
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        TAB_RUNTIME_LOCK_DEPTH.with(|d| d.set(1));
        Self { lock: Some(lock) }
    }
}

#[cfg(test)]
impl Drop for TabRuntimeTestGuard {
    fn drop(&mut self) {
        TAB_RUNTIME_LOCK_DEPTH.with(|d| {
            let depth = d.get();
            if depth <= 1 {
                d.set(0);
            } else {
                d.set(depth - 1);
            }
        });
    }
}

/// 串行化 Tab worker / WebView 加载相关测试，避免并行套件中的全局状态竞争。
#[cfg(test)]
pub fn tab_runtime_test_guard() -> TabRuntimeTestGuard {
    TabRuntimeTestGuard::acquire()
}
