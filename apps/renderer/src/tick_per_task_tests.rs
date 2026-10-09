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

// t8 返修（defect-r1 D1）：导航 reset 竞态窗——主线程 reset_document_state 的队列清空之后、
// worker 处理 ResetDocumentState 之前，旧页脚本/timer 回调仍可写入（worker 串行保证 arm
// 执行时旧页已结束）。滞留写入会被新文档的 tick/drain apply（跨文档污染）。本测让旧页
// 脚本横跨 reset 窗口落两笔写：a 在主清队前（被主清队消费）、b 在主清队后 arm 前（竞态窗），
// 断言 reset 返回后队列无残留。变异判别：移除 arm 侧清队 → b 滞留 → 本测转红。
// 时序 margin（testval-r2 F1）：120ms spawn 前置 + 600ms 忙等——全模块 6 路并行 V8 负载下
// spawn 延迟远小于前置量；极端 CI 延迟的退化方向是 reset 先于脚本入队（execute 报错）或
// 双写均落主清队前被兜底消费（假通过），无假红方向。
#[test]
fn reset_clears_writes_landed_after_main_thread_clear_d1() {
    let runtime = runtime_with_observer_page(9114);
    let worker = &runtime.js_worker;
    std::thread::scope(|s| {
        let script = s.spawn(|| {
            worker.execute_script_direct(
                "document.getElementById('t1').setAttribute('data-d1','a'); var __s = Date.now(); while (Date.now() - __s < 600); document.getElementById('t1').setAttribute('data-d1','b');",
            )
        });
        // 旧页脚本进入忙等（a 已落队）后 reset：主清队消费 a；b 在 600ms 忙等结束时落队，
        // 早于 reset arm（排在此后的脚本命令完成之后），恰落在竞态窗内。
        std::thread::sleep(std::time::Duration::from_millis(120));
        worker.reset_document_state();
        script.join().expect("script thread").expect("execute ok");
    });
    let queue = worker.mutations();
    let pending = queue.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        pending.is_empty(),
        "reset 返回后队列不得残留竞态窗内落队的旧页写入（跨导航污染源），实际 {} 条",
        pending.len()
    );
}

// t8n：真导航 reset_context 丢弃一次性 execute 的 `__zwVideoBridge` 门面全局
//（native 回调随 sandbox 回调表自动重挂，门面不会）——SetVideoPlayers 武装后须随
// shim 重装补挂，否则新文档页面侧 feature-detect（play 真值路径 / MSE
// isTypeSupported）恒回落 headless 近似（活体探针证据：bridge=undefined →
// isTypeSupported false → blob: 短路失联，loadedmetadata 600s fallback）。
// 变异判别：移除 ResetDocumentState 臂的门面补挂 → rearmed 落 -1 → 本测转红。
#[test]
fn video_bridge_facade_rearmed_after_document_reset_t8n() {
    let runtime = runtime_with_observer_page(9121);
    let worker = &runtime.js_worker;
    let count = "typeof __zwVideoBridge === 'object' ? Object.keys(__zwVideoBridge).length : -1";
    let initial: i64 = worker.execute_script_direct(count).unwrap().trim().parse().unwrap();
    assert!(initial > 0, "构造期 SetVideoPlayers 已注入门面，实际 {initial}");
    worker.reset_document_state();
    let rearmed: i64 = worker.execute_script_direct(count).unwrap().trim().parse().unwrap();
    assert_eq!(rearmed, initial, "reset 后门面须补挂（方法数一致），实际 {rearmed}");
    // t8n 返修 N5：reset 后实调门面方法——键数一致不证方法体可调（natives 回调表
    // 若改为 context 级，门面方法将 ReferenceError；实调击穿该静默面）。isPlaying
    // 走 `__zw_video_1` native 往返（未登记键 → falsy 返回），返回值本身不重于
    // 「可调用且不抛」。
    let callable = worker
        .execute_script_direct(
            "typeof __zwVideoBridge.isPlaying === 'function' ? String(__zwVideoBridge.isPlaying('x')) : 'MISSING'",
        )
        .unwrap();
    assert_ne!(callable.trim(), "MISSING", "reset 后门面方法缺失");
    assert_eq!(callable.trim(), "false", "isPlaying 未登记键回落 falsy");
}

// slice18 评审收尾 S1：loadmatrix 最小判别案（同源基本型 + 真跨域）renderer 真管线
// 端到端钉——页面脚本 createElement('script')+src+appendChild → 宿主
// PendingDynamicScripts 取回执行 → onload/onerror 派发。单执行者归属契约
//（`__zwHostOwnsDynamicScripts`，SetDomSnapshot 置位 → shim cors 语义页面 fetch 通道
// 整体跳过）下断言：取回源码执行恰一次 + onload 恰一次 + 零误派 error。修前
//（base f88350219/85479210a）通道与宿主并存：无 ACAO 跨域脚本在 cors 语义下恒败误派
// error（err==1 红）。run_page_scripts 每 chunk 前 set_dom_snapshot（page_scripts.rs
// 快照换代语义）在修复后先行置位——本钉的判别力即来自该真实时点。
// 活体对应形态：loadmatrix v1（同源）/ v9（跨域）——
// diag/evidence/slice18/s18-netcount-{base-truebin-double,fixed-truebin-single}.json。
// 注：本文件属 zero-renderer lib target，不在 make test 口径（workspace 排除
// zero-renderer 后仅跑其 bin target）；与既有 R-baidu8 runtime 钉同层，CI 覆盖由
// CI 矩阵现状决定，常驻判别钉（make test 口径）见 engine r387c（loadmatrix fixture）。
// https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
#[test]
fn dynamic_sameorigin_script_single_execution_end_to_end() {
    let html = r#"<html><head><script>
      var s = document.createElement('script');
      s.src = '/s18-dyn-a.js';
      s.onload = function () { globalThis.__zwALoad = (globalThis.__zwALoad | 0) + 1; };
      s.onerror = function () { globalThis.__zwErr = (globalThis.__zwErr | 0) + 1; };
      document.head.appendChild(s);
    </script></head><body></body></html>"#;
    let url = "https://zero.test/s18-loadmatrix-a";
    let mut runtime = RendererRuntime::new(9121);
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
        "https://zero.test/s18-dyn-a.js".to_string(),
        Ok("globalThis.__zwExec = (globalThis.__zwExec | 0) + 1;".to_string()),
    );
    runtime.execute_new_dynamic_scripts();
    runtime.tick_dynamic_scripts().expect("tick dynamic scripts");
    let verdict = runtime
        .js_worker
        .execute_script_direct(
            "JSON.stringify({exec: (globalThis.__zwExec | 0), load: (globalThis.__zwALoad | 0), err: (globalThis.__zwErr | 0)})",
        )
        .unwrap();
    assert_eq!(
        verdict.trim(),
        r#"{"exec":1,"load":1,"err":0}"#,
        "同源动态脚本单执行语义：宿主取回执行恰一次、onload 恰一次、零误派 error（修前 err==1）"
    );
}

#[test]
fn dynamic_crossorigin_script_no_false_error_end_to_end() {
    // 真跨域案（loadmatrix v9 镜像）：无 ACAO 的跨域 CDN 脚本在 cors 语义页面 fetch
    // 通道下恒败——修前误派元素 error（活体 base error+load 双派）；宿主 no-cors
    // 取回实际可执行。断言取回执行恰一次、onload 恰一次、error 恒零。
    let html = r#"<html><head><script>
      var s = document.createElement('script');
      s.src = 'https://cdn.zero.test/s18-dyn-x.js';
      s.onload = function () { globalThis.__zwXLoad = (globalThis.__zwXLoad | 0) + 1; };
      s.onerror = function () { globalThis.__zwErr = (globalThis.__zwErr | 0) + 1; };
      document.head.appendChild(s);
    </script></head><body></body></html>"#;
    let url = "https://zero.test/s18-loadmatrix-x";
    let mut runtime = RendererRuntime::new(9122);
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
        "https://cdn.zero.test/s18-dyn-x.js".to_string(),
        Ok("globalThis.__zwXExec = (globalThis.__zwXExec | 0) + 1;".to_string()),
    );
    runtime.execute_new_dynamic_scripts();
    runtime.tick_dynamic_scripts().expect("tick dynamic scripts");
    let verdict = runtime
        .js_worker
        .execute_script_direct(
            "JSON.stringify({exec: (globalThis.__zwXExec | 0), load: (globalThis.__zwXLoad | 0), err: (globalThis.__zwErr | 0)})",
        )
        .unwrap();
    assert_eq!(
        verdict.trim(),
        r#"{"exec":1,"load":1,"err":0}"#,
        "跨域动态脚本单执行语义：宿主 no-cors 取回执行恰一次、onload 恰一次、无 ACAO 不误派 error（修前 err==1）"
    );
}

fn runtime_with_sink(renderer_id: u64) -> RendererRuntime {
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime
}

// t8o G1：脚本阶段收尾先于 load 完成时（无外链脚本页面 prefetch 秒完），其后完成的
// 媒体资源事件在 load 完成 drain 点立即派发——不再滞留 stash 永不派发。修复前活体
// 证据 out-t8o-attr-diag3（merged main 构建）：fetch 200/94294B 落定、自然态零事件
//（stash 唯一消费者 finish 已过场）；手动 commit 全链可达证明 shim 侧无缺口。
// 判别：媒体 fetch 走 IPC 挂起（outbound=sink 吞请求），脚本阶段收尾完成后经
// inflight 注入 FetchResponse 复现迟到序；settle 状态（currentSrc）同步写，不依赖
// 定时器泵。修前 cur==''（事件滞留 stash）转红。
// https://html.spec.whatwg.org/multipage/media.html#concept-media-load-algorithm
#[test]
fn late_media_resource_events_dispatched_after_script_finish_t8o() {
    let html = r#"<html><head><script>
      globalThis.__ev = [];
      var v = document.getElementById('v');
      ['loadstart', 'loadedmetadata', 'error'].forEach(function (e) {
        v.addEventListener(e, function () { globalThis.__ev.push(e); });
      });
    </script></head>
    <body><video id="v" src="/media/t8o.mp4" width="320" height="240"></video></body></html>"#;
    let url = "https://zero.test/t8o-late-media";
    let mut runtime = runtime_with_sink(9141);
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: None,
                url: Some(url.to_string()),
                navigation_epoch: 1,
            }),
        })
        .unwrap();
    // 媒体 fetch 走 IPC 挂起（outbound 吞请求）→ pending_load 保持活跃、prefetch
    // （无外链脚本）先收口——复现「finish 先于 load 完成」迟到序。
    let mut ticks = 0;
    loop {
        runtime.tick_pending_load().unwrap();
        runtime.tick_script_prefetch().unwrap();
        if runtime.pending_load.is_some() && runtime.pending_script_prefetch.is_none() {
            break;
        }
        ticks += 1;
        assert!(ticks < 500, "prefetch 未先于挂起媒体收口");
    }
    let cur_before = runtime
        .js_worker
        .execute_script_direct("document.getElementById('v').currentSrc")
        .unwrap();
    assert_eq!(
        cur_before.trim(),
        "",
        "收尾时媒体未 settle，currentSrc 恒空（此断言在误先 settle 的实现下转红）"
    );
    // 注入迟到 FetchResponse（媒体字节垃圾 → 探针 None → Available w0h0 合成序，
    // 与 decode feature 无关）并推 load 至完成 drain。settle 状态同步写——
    // currentSrc 非空即迟到派发达成，不定时器面。
    let request_id = *runtime
        .inflight_fetches
        .pending_request_ids()
        .first()
        .expect("媒体 IPC fetch 应在飞");
    let (inject_tx, inject_rx) = mpsc::channel();
    inject_tx
        .send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id,
                status_code: 200,
                headers: Vec::new(),
                body: b"t8o-not-a-container".to_vec(),
            }),
        })
        .unwrap();
    drop(inject_tx);
    runtime.inbound_rx = inject_rx;
    runtime.drain_inflight_fetch_responses();
    runtime.tick_pending_load().unwrap();
    runtime.tick_pending_load().unwrap();
    let cur_after = runtime
        .js_worker
        .execute_script_direct("document.getElementById('v').currentSrc")
        .unwrap();
    assert_eq!(
        runtime.pending_resource_element_events.len(),
        0,
        "迟到事件应在 drain 点派发清空，不得滞留 stash"
    );
    assert_ne!(
        cur_after.trim(),
        "",
        "收尾后完成的媒体事件须迟到派发（settle 状态同步可达）；修前滞留 stash 恒空"
    );
}
