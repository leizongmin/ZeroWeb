//! event-loop-spec M3-S2：renderer `tick_observers` per-task 模式结构验证。
//!
//! 锁定机制面：per-task 模式下 tick 循环对 N 个活跃 observer 调 N 次
//! `__zw_observers_tick_once`（每 observer 一 execute 一 checkpoint）；默认合并模式不调
//! tick_once（`__zw_observers_tick` 整批 schedule）。observer 回调间的 apply 时序差异
//! （per-task 每 observer 后 apply）依赖 host 渲染循环几何刷新，unit 面不可达——行为级
//! 验收走 bench-gate + product-smoke + reftest A/B（默认 OFF）。

use super::*;
use crate::page_scripts;

fn runtime_with_observer_page(renderer_id: u64) -> RendererRuntime {
    let html = r#"<html><body>
        <div id="t1" style="width:50px;height:50px;"></div>
        <div id="t2" style="width:50px;height:50px;"></div>
        <script>
          globalThis.__obs1 = [];
          globalThis.__obs2 = [];
          globalThis.__ioA = new IntersectionObserver(function (rs) {
            for (var i = 0; i < rs.length; i++) globalThis.__obs1.push(rs[i].target.id);
          });
          globalThis.__ioB = new IntersectionObserver(function (rs) {
            for (var i = 0; i < rs.length; i++) globalThis.__obs2.push(rs[i].target.id);
          });
          globalThis.__ioA.observe(document.getElementById('t1'));
          globalThis.__ioB.observe(document.getElementById('t2'));
        </script>
        </body></html>"#;
    let url = "https://zero.test/tick-per-task";
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().load_html(html, None);
    {
        let mut ctx = PageScriptContext {
            html: &mut runtime.cached_html,
            url,
            js_worker: &runtime.js_worker,
            webview: runtime.webview.as_mut(),
        };
        page_scripts::run_page_scripts(&mut ctx, true, |_url| Err::<String, String>("no fetch".into()));
    }
    runtime
}

#[test]
fn tick_per_task_mode_iterates_one_observer_per_execute() {
    let mut runtime = runtime_with_observer_page(9101);
    // 包裹 tick_once 计数器（放 probe 后——run_page_scripts 已执行初通知派发）。
    runtime
        .js_worker
        .execute_script_direct(
            "globalThis.__tickOnceCalls = 0;\
             var orig = globalThis.__zw_observers_tick_once;\
             globalThis.__zw_observers_tick_once = function () {\
               globalThis.__tickOnceCalls++;\
               return orig ? orig.apply(globalThis, arguments) : false;\
             };",
        )
        .unwrap();
    let mut ctx = PageScriptContext {
        html: &mut runtime.cached_html,
        url: "https://zero.test/tick-per-task",
        js_worker: &runtime.js_worker,
        webview: Some(runtime.webview.as_mut().unwrap()),
    };
    page_scripts::tick_observers_with(&mut ctx, true);
    let calls = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__tickOnceCalls)")
        .unwrap();
    assert_eq!(
        calls.trim(),
        "3",
        "per-task：2 活跃 observer → tick_once 调 3 次（2 次 schedule + 1 次终止探测返 -1）——每 observer 一 execute"
    );
    // 游标耗尽：cursor=2 起无活跃 observer → tick_once 返 -1（host 循环终止依据）。
    let exhausted = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__zw_observers_tick_once(2))")
        .unwrap();
    assert_eq!(exhausted.trim(), "-1", "游标越界 → -1（host 循环终止依据）");
}

#[test]
fn tick_combined_mode_default_does_not_call_tick_once() {
    let mut runtime = runtime_with_observer_page(9102);
    runtime
        .js_worker
        .execute_script_direct(
            "globalThis.__tickOnceCalls = 0;\
             var orig = globalThis.__zw_observers_tick_once;\
             globalThis.__zw_observers_tick_once = function () {\
               globalThis.__tickOnceCalls++;\
               return orig ? orig.apply(globalThis, arguments) : false;\
             };",
        )
        .unwrap();
    let mut ctx = PageScriptContext {
        html: &mut runtime.cached_html,
        url: "https://zero.test/tick-per-task",
        js_worker: &runtime.js_worker,
        webview: Some(runtime.webview.as_mut().unwrap()),
    };
    page_scripts::tick_observers_with(&mut ctx, false);
    let calls = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__tickOnceCalls)")
        .unwrap();
    assert_eq!(
        calls.trim(),
        "0",
        "默认合并模式走 __zw_observers_tick 整批，不经 tick_once"
    );
    // 合并模式交付面不变：初通知已在 run_page_scripts 派发（双数组各 1 条）。
    let delivered = runtime
        .js_worker
        .execute_script_direct("globalThis.__obs1.length + '/' + globalThis.__obs2.length")
        .unwrap();
    assert_eq!(delivered.trim(), "1/1", "双 observer 初通知均可达");
}
