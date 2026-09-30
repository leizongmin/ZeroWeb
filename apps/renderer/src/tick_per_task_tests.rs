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

#[test]
fn dynamic_external_scripts_host_fetch_fires_element_error_event() {
    // R-baidu8：动态外链脚本走宿主取回（PendingDynamicScripts + ResourceFetchMeta::SCRIPT，
    // no-cors 脚本语义），不再用页面 fetch()（cors 语义）+ eval——无 ACAO 脚本源在页面 fetch
    // 下永远失败。stub network 取回失败 → 元素 error 事件（R2944 镜像，shim 按 src 绝对 URL
    // 匹配派发）+ 队列排空。
    let html = r#"<html><head><script>
      var s = document.createElement('script');
      s.src = '/dyn.js';
      s.onerror = function () { globalThis.__dynErr = (globalThis.__dynErr | 0) + 1; };
      document.head.appendChild(s);
    </script></head><body></body></html>"#;
    let url = "https://zero.test/dynamic-scripts";
    let mut runtime = RendererRuntime::new(9111);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.stub_network = true;
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
        // 页面脚本 appendChild 的 <script src="/dyn.js"> 落定（真实流程由主循环每轮
        // drain_pending_script_mutations 驱动；run_page_scripts chunk 内已即时 apply 时此调用 no-op）。
        let _ = page_scripts::drain_pending_dom_mutations(&mut ctx);
    }
    assert!(
        runtime.cached_html.contains("/dyn.js"),
        "appendChild mutation 落定到 cached_html（动态脚本提取源）"
    );
    runtime.execute_new_dynamic_scripts();
    assert!(runtime.pending_dynamic_scripts.is_some(), "动态外链脚本入队");
    runtime.tick_dynamic_scripts().expect("tick dynamic scripts");
    assert!(runtime.pending_dynamic_scripts.is_none(), "stub 即刻 Err → 队列排空");
    let fired = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__dynErr | 0)")
        .unwrap();
    assert_eq!(
        fired.trim(),
        "1",
        "取回失败 → 元素 error 事件一次（R2944 镜像按 src 绝对 URL 匹配；createElement 产物经 __zw_handle_for_selector 反查命中 handle 监听）"
    );
}

#[test]
fn dynamic_external_scripts_host_fetch_success_fires_load_event_and_executes() {
    // R-baidu8 review m1：成功路径（取回 Ok → execute_script_direct → 元素 load 事件）
    // 的 runtime 级回归钉。stub 注入 Ok 应答；onload 计数与脚本全局副作用双断言。
    let html = r#"<html><head><script>
      var s = document.createElement('script');
      s.src = '/ok.js';
      s.onload = function () { globalThis.__okLoad = (globalThis.__okLoad | 0) + 1; };
      s.onerror = function () { globalThis.__okErr = (globalThis.__okErr | 0) + 1; };
      document.head.appendChild(s);
    </script></head><body></body></html>"#;
    let url = "https://zero.test/dynamic-scripts-ok";
    let mut runtime = RendererRuntime::new(9112);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.stub_network = true;
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
        let _ = page_scripts::drain_pending_dom_mutations(&mut ctx);
    }
    runtime.stub_fetch_responses.insert(
        "https://zero.test/ok.js".to_string(),
        Ok("globalThis.__okExec = 1;".to_string()),
    );
    runtime.execute_new_dynamic_scripts();
    assert!(runtime.pending_dynamic_scripts.is_some(), "动态外链脚本入队");
    runtime.tick_dynamic_scripts().expect("tick dynamic scripts");
    assert!(runtime.pending_dynamic_scripts.is_none(), "stub 即刻 Ok → 队列排空");
    let load = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__okLoad | 0)")
        .unwrap();
    assert_eq!(load.trim(), "1", "取回成功 → 元素 load 事件一次（R2944 镜像）");
    let exec = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__okExec | 0)")
        .unwrap();
    assert_eq!(exec.trim(), "1", "取回的源码已执行（execute_script_direct）");
    let err = runtime
        .js_worker
        .execute_script_direct("String(globalThis.__okErr | 0)")
        .unwrap();
    assert_eq!(err.trim(), "0", "成功路径不派元素 error 事件");
}

// t8/P16：observer tick 不得清空异步 turn 的 pending mutation——tick_observers_with 在每次
// 帧发布末尾运行，与定时器 resolve 等异步 turn 共享 worker mutation 队列；旧实现的
// set_dom_snapshot+clear 把「已入队尚未 drain」的写入（html5test.co 完成回调的
// contents/loading 样式写入）在下一帧渲染前整批销毁。修复后 pending 与 observer 写入一并落 host。
// 变异判别：恢复 pre-clear → pending 被清空 → host 无 data-t8，本测转红。
#[test]
fn observer_tick_preserves_pending_async_mutations_t8() {
    let mut runtime = runtime_with_observer_page(9113);
    // 模拟异步 turn（定时器回调）已写队列、尚未 drain。
    runtime
        .js_worker
        .execute_script_direct("document.getElementById('t1').setAttribute('data-t8','pending-async');")
        .unwrap();
    let mut ctx = PageScriptContext {
        html: &mut runtime.cached_html,
        url: "https://zero.test/tick-per-task",
        js_worker: &runtime.js_worker,
        webview: Some(runtime.webview.as_mut().unwrap()),
    };
    page_scripts::tick_observers_with(&mut ctx, false);
    assert!(
        runtime.cached_html.contains("data-t8"),
        "异步 turn 的 pending mutation 应在 observer tick 后落 host，而非被 pre-clear 销毁"
    );
}
