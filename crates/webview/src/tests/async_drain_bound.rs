//! execute_script 的异步回调排水预算上界回归测试。
//!
//! 根因背景（service_worker_runtime 套件 leg 上下文 livelock）：`webview` 的
//! `drain_next_async_callback_if_pending` 原先仅在通道空档（`try_recv` 返回
//! Empty）检查 20ms 预算。SW 轮询链（pollRegistration / pollClientMessages）
//! 以 `setTimeout(fn, 0)` 自续且每跳含宿主调用——解析耗时大于定时器线程
//! spawn——多链并行时供给压倒排水，通道永无空档，排水循环永不返回：测试
//! 线程在单次 `execute_script` 内 100% 单核空转（实测 101.6s），自身 60s
//! deadline 断言不可达，serial 锁被占，整个测试二进制挂死。本测试以 4 条
//! 永续慢跳自续链复现该形态，要求每次 `execute_script` 在有界时间内返回；
//! 回归时断言快速失败，修复后确定性通过。

use std::time::{Duration, Instant};

use crate::{WebView, WebViewConfig};

#[test]
fn execute_script_returns_with_eternal_zero_timeout_chain() {
    let mut wv = WebView::new(WebViewConfig::default());
    let setup = wv
        .execute_script("typeof setTimeout")
        .expect("setup execute_script should succeed");
    assert_eq!(setup, "function", "setTimeout shim must exist in plain execute_script");

    // 4 条永续慢跳链：每跳 ~3ms 同步 JS 计算后 setTimeout(f, 0)。单条链在每次
    // 回调解析期间无新回调到达，通道必现空档、deadline 可兜住；多条慢链下，
    // 任一回调解析期间其余链已积累新回调（spawn ~50µs ≪ 解析 ~3ms），队列
    // 永不空档——模拟 pollRegistration / pollClientMessages 每跳含宿主调用
    // （解析慢于线程 spawn）且多注册并行轮询的形态。
    wv.execute_script(
        "globalThis.__drainProbeBusy = function () {
           var s = 0;
           for (var i = 0; i < 200000; i++) s += Math.sqrt(i);
           return s;
         };
         for (var i = 0; i < 4; i++) {
           (function f() { globalThis.__drainProbeBusy(); setTimeout(f, 0); })();
         } undefined;",
    )
    .expect("chain setup should succeed");

    let overall = Instant::now() + Duration::from_secs(30);
    for i in 0..50 {
        let call_start = Instant::now();
        let result = wv.execute_script("1+1");
        assert!(result.is_ok(), "execute_script call {i} failed: {result:?}");
        let elapsed = call_start.elapsed();
        assert!(
            elapsed < Duration::from_secs(2),
            "execute_script call {i} took {elapsed:?} — async-callback drain exceeded its budget"
        );
        assert!(Instant::now() < overall, "probe overall budget exceeded");
        std::thread::sleep(Duration::from_millis(5));
    }
}
