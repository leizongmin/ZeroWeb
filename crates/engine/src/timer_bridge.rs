//! P1b S5 定时器 bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持 [`AsyncResolver`]；`register` 在 sandbox 注 `__zw_setTimeout(id, delayMs)` 回调。
//! JS `setTimeout/setInterval`（shim 侧）把回调存 `__zw_pending[id]` 后调本回调；本回调
//! **非阻塞**——子线程 `sleep(delayMs)` 后 `resolver.resolve(id, "")` 投递回 JS worker，
//! worker 调 `sandbox.resolve_async_callback` → shim `__zwResolveCallback` 取出并调用回调。
//!
//! `setInterval` 由 shim 在回调内 re-arm（再次调 `__zw_setTimeout`），故 host 仅需本回调。
//! `clearTimeout/clearInterval` 由 shim 删 `__zw_pending[id]` 实现——即便子线程后到，
//! `__zwResolveCallback` 见无 pending 项即 no-op（host 线程浪费一次 sleep，JS 回调不触发）。

use std::time::Duration;

use zero_script_sandbox::Sandbox;

use crate::async_resolver::AsyncResolver;

/// P1b S5 定时器 bridge——`setTimeout`/`setInterval` 真实延迟（子线程 sleep + 异步 resolve）。
pub struct TimerBridge {
    resolver: AsyncResolver,
}

impl TimerBridge {
    /// 构造——`resolver` 用于定时器到期后 resolve（触发 shim 调用 JS 回调）。
    pub fn new(resolver: AsyncResolver) -> Self {
        Self { resolver }
    }

    /// 注册 `__zw_setTimeout(id, delayMs)` 回调——shim 的 `setTimeout`/`setInterval` 调此。
    /// **非阻塞**：子线程 `sleep(delayMs)` 后 `resolver.resolve(id, "")`——JS worker 不在
    /// 等待期间冻结。`delayMs=0` 仍走子线程（yield 让出 worker，微秒级）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let resolver = self.resolver.clone();
        sandbox.register_callback(
            "__zw_setTimeout",
            Box::new(move |args: &[String]| -> String {
                let id = args.first().cloned().unwrap_or_default();
                let delay_ms: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let resolver = resolver.clone();
                std::thread::spawn(move || {
                    if delay_ms > 0 {
                        std::thread::sleep(Duration::from_millis(delay_ms));
                    }
                    resolver.resolve(&id, "");
                });
                String::new()
            }),
        );
    }
}
