//! P1b S1 异步回调 resolver——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 克隆供跨线程异步完成方（fetch host / 定时器）持有，`resolve(id, result)` 经闭包
//! 投递到 app 的 cmd channel，JS worker 收到后调 `sandbox.resolve_async_callback`
//! （执行 shim 的 `__zwResolveCallback` resolve pending Promise）。
//!
//! 内部 `Arc<dyn Fn>`：`mpsc::Sender` 是 `Send` 但**非 `Sync`**，无法直接被
//! `register_callback` 要求的 `Send + Sync` 闭包捕获（S3 `__zw_fetch` 等回调所需）。
//! app 在闭包内用 `Arc<Mutex<Sender>>` 包裹自身 sender，故 `AsyncResolver: Send + Sync + Clone`。

use std::sync::Arc;

/// 异步回调 resolver。app 构造时传入包装自身 cmd channel 的闭包 `(id, result) -> ()`：
/// 闭包内 send 一条 `ResolveAsyncCallback { id, result }` 命令到 JS worker 线程。
#[derive(Clone)]
pub struct AsyncResolver(Arc<dyn Fn(&str, &str) + Send + Sync>);

impl AsyncResolver {
    /// 构造——`resolve_fn` 把 `(id, result)` 投递到 app 的 cmd channel（app 包装自身
    /// `resolver.resolve` / cmd channel send）。
    pub fn new<F>(resolve_fn: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        Self(Arc::new(resolve_fn))
    }

    /// 投递一次异步 resolve（fire-and-forget）。JS worker 收到后执行
    /// `__zwResolveCallback(id, result)` resolve pending Promise。
    pub fn resolve(&self, id: &str, result: &str) {
        (self.0)(id, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S3 prep：register_callback 闭包要求 Send + Sync（mpsc::Sender 非 Sync，Arc<Mutex<>>
    /// 修复）。编译期 trait 断言——非运行时行为。
    #[test]
    fn async_resolver_traits_send_sync_clone() {
        fn assert_bounds<T: Send + Sync + Clone>() {}
        assert_bounds::<AsyncResolver>();
    }

    /// resolve 经闭包投递；克隆可移到子线程 resolve（仿真实 fetch host / 定时器跨线程完成）。
    #[test]
    fn async_resolver_delivers_via_closure_from_other_thread() {
        use std::sync::Mutex;
        let got: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
        let g = Arc::clone(&got);
        let resolver = AsyncResolver::new(move |id, result| {
            *g.lock().unwrap() = Some((id.to_string(), result.to_string()));
        });
        let moved = resolver.clone();
        let handle = std::thread::spawn(move || moved.resolve("id1", "v1"));
        handle.join().unwrap();
        assert_eq!(
            got.lock().unwrap().clone().unwrap(),
            ("id1".to_string(), "v1".to_string())
        );
    }
}
