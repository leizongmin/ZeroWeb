//! 渲染进程 JS 线程 — V8 与页面渲染分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_engine::{
    AsyncResolver, DomMutation, FetchBridge, FetchHandler, LayoutRectSnapshot, RectBridge, TimerBridge,
    generate_js_dom_shim, make_dom_html_rect_handler, new_layout_rect_snapshot, register_dom_callbacks,
};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, build_module_runtime_prelude, compile_dependency_iife, compile_module_script,
    extract_module_import_specifiers,
};
use zero_webview::fetch_text_async;

const TAB_JS_EXEC_TIMEOUT_MS: u64 = 15_000;
const TAB_JS_CHANNEL_TIMEOUT: Duration = Duration::from_millis(TAB_JS_EXEC_TIMEOUT_MS + 5_000);

/// P1a gBCR kill-switch：默认 on；`ZW_REAL_RECT=0` 关闭 RectBridge（`__zw_getBoundingClientRect`
/// 不注册 → shim 回落零 rect = 当前行为，零回归）。snapshot 为空 / identity 未命中同样回落零 rect。
/// P1a Slice 2b：亦用作 observer host-tick 的 kill-switch（gBCR 关 → rect 恒零 → tick 无意义）。
pub(crate) fn real_rect_enabled() -> bool {
    !matches!(std::env::var("ZW_REAL_RECT").as_deref(), Ok("0"))
}

type ScriptFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;
type ModuleFn = Arc<dyn Fn(&str, &str, &[(String, String)]) -> Result<String, String> + Send + Sync>;

enum JsWorkerCommand {
    Execute {
        script: String,
        reply: Sender<Result<String, String>>,
    },
    ExecuteModule {
        source: String,
        url: String,
        deps: Vec<(String, String)>,
        reply: Sender<Result<String, String>>,
    },
    SetDomSnapshot {
        html: String,
        url: String,
    },
    /// P1b S1：跨线程异步回调 resolve（marshal channel）。任意线程经
    /// [`RendererJsWorker::async_resolver`] 投递 (id, result)，JS worker 收到后调
    /// `sandbox.resolve_async_callback`（执行 shim 的 `__zwResolveCallback` resolve Promise）。
    ResolveAsyncCallback {
        id: String,
        result: String,
    },
    /// P1b S3：注入 fetch handler（renderer 在 WebView 初始化后发送；测试用合成 handler）。
    SetFetchHandler {
        handler: FetchHandler,
    },
    Shutdown,
}

/// 渲染进程 JS worker 句柄。
pub struct RendererJsWorker {
    cmd_tx: Sender<JsWorkerCommand>,
    join: Option<JoinHandle<()>>,
    executor: ScriptFn,
    module_executor: ModuleFn,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    /// P1a gBCR：共享 layout-rect snapshot——renderer 主循环 render 后填充，
    /// js_worker 的 RectBridge handler 读取（经 identity→NodeId 解析后查 rect）。
    rect_snapshot: LayoutRectSnapshot,
}

impl RendererJsWorker {
    /// 启动 JS 专用线程。
    pub fn spawn(renderer_id: u64) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let rect_snapshot = new_layout_rect_snapshot();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let cmd_for_worker = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);
        let rect_snapshot_for_worker = Arc::clone(&rect_snapshot);

        let join = thread::Builder::new()
            .name(format!("renderer-js-{}", renderer_id))
            .spawn(move || js_worker_main(cmd_rx, cmd_for_worker, mutations_for_worker, rect_snapshot_for_worker))
            .expect("spawn renderer js worker");

        let executor: ScriptFn = Arc::new(move |script: &str| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_exec
                .send(JsWorkerCommand::Execute {
                    script: script.to_string(),
                    reply: reply_tx,
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
                .map_err(|e| e.to_string())?
        });

        let module_executor: ModuleFn = Arc::new(move |source: &str, url: &str, deps: &[(String, String)]| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_module
                .send(JsWorkerCommand::ExecuteModule {
                    source: source.to_string(),
                    url: url.to_string(),
                    deps: deps.to_vec(),
                    reply: reply_tx,
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
                .map_err(|e| e.to_string())?
        });

        Self {
            cmd_tx,
            join: Some(join),
            executor,
            module_executor,
            mutations,
            rect_snapshot,
        }
    }

    /// 供 WebView 注入的外部脚本执行器（T4 脚本桥接统一后评估是否保留）。
    #[allow(dead_code)]
    pub fn executor(&self) -> ScriptFn {
        Arc::clone(&self.executor)
    }

    /// 执行 ES module（含依赖注册表）。
    pub fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        (self.module_executor)(source, url, deps)
    }

    /// 在 JS 线程执行脚本（不经 WebView 包装）。
    pub fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        (self.executor)(script)
    }

    /// 脚本执行前更新 DOM HTML 快照与页面 URL。
    pub fn set_dom_snapshot(&self, html: &str, url: &str) {
        let _ = self.cmd_tx.send(JsWorkerCommand::SetDomSnapshot {
            html: html.to_string(),
            url: url.to_string(),
        });
    }

    /// 脚本执行期间记录的 DOM 变更（由 `__zw_*` 回调写入）。
    pub fn mutations(&self) -> Arc<std::sync::Mutex<Vec<DomMutation>>> {
        Arc::clone(&self.mutations)
    }

    /// P1a gBCR：共享 layout-rect snapshot 句柄——renderer 主循环 render 后经
    /// `fill_layout_rect_snapshot` 填充，js_worker 的 RectBridge handler 读取。
    pub fn rect_snapshot(&self) -> LayoutRectSnapshot {
        Arc::clone(&self.rect_snapshot)
    }

    /// 返回异步回调 resolver（P1b S1）。克隆供跨线程异步完成方（fetch host / 定时器）持有，
    /// `resolver.resolve(id, result)` 经 cmd channel marshal 回 JS worker 线程，由 worker 调
    /// `sandbox.resolve_async_callback` resolve 对应 Promise。
    #[allow(dead_code)] // 未来 setTimeout / MutationObserver 跨线程完成方消费（S5/S2）。
    pub fn async_resolver(&self) -> AsyncResolver {
        let tx = Arc::new(std::sync::Mutex::new(self.cmd_tx.clone()));
        AsyncResolver::new(move |id, result| {
            let _ = tx.lock().unwrap().send(JsWorkerCommand::ResolveAsyncCallback {
                id: id.to_string(),
                result: result.to_string(),
            });
        })
    }

    /// P1b S3：注入 fetch handler（renderer 在 WebView 初始化后调用；测试用合成实现）。
    /// `__zw_fetch` 回调读此 handler 抓取后 resolve Promise。
    pub fn set_fetch_handler(&self, handler: FetchHandler) {
        let _ = self.cmd_tx.send(JsWorkerCommand::SetFetchHandler { handler });
    }

    /// 关闭 JS 线程。
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(JsWorkerCommand::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for RendererJsWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn js_worker_main(
    cmd_rx: Receiver<JsWorkerCommand>,
    cmd_tx: Sender<JsWorkerCommand>,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    rect_snapshot: LayoutRectSnapshot,
) {
    let js_config = SandboxConfig {
        persistent_context: true,
        timeout_ms: TAB_JS_EXEC_TIMEOUT_MS,
        ..Default::default()
    };
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::V8Sandbox::with_config(js_config).expect("V8 sandbox init"));
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::QuickJSSandbox::with_config(js_config).expect("QuickJS sandbox init"));
    let dom_html: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::new()));
    let page_url: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::from("about:blank")));
    register_dom_callbacks(&mut *sandbox, &mutations, &dom_html, &page_url);
    register_module_compile_callback(&mut *sandbox);
    // P1a gBCR（Slice 1）：RectBridge 注 `__zw_getBoundingClientRect(identity)` 同步回调。
    // handler 解析 identity(selector) → NodeId（fresh-parse dom_html，与渲染管线确定性一致）
    // → 查 rect_snapshot。kill-switch `ZW_REAL_RECT=0` 关闭（回落零 rect = 当前行为，零回归）。
    if real_rect_enabled() {
        let rect_bridge = RectBridge::new();
        rect_bridge.register(&mut *sandbox);
        rect_bridge.set_handler(make_dom_html_rect_handler(
            Arc::clone(&dom_html),
            Arc::clone(&rect_snapshot),
        ));
    }
    // P1b S1/S3：AsyncResolver（跨线程 resolve Promise）+ FetchBridge（__zw_fetch 注册 +
    // handler cell）。fetch_bridge 经 SetFetchHandler 命令在 WebView 初始化后注入生产 handler
    // （chicken-and-egg——js_worker spawn 早于 WebView）；未注入时 __zw_fetch resolve 错误 Promise。
    let resolver = AsyncResolver::new({
        let tx = Arc::new(std::sync::Mutex::new(cmd_tx));
        move |id, result| {
            let _ = tx.lock().unwrap().send(JsWorkerCommand::ResolveAsyncCallback {
                id: id.to_string(),
                result: result.to_string(),
            });
        }
    });
    let fetch_bridge = FetchBridge::new(resolver.clone());
    fetch_bridge.register(&mut *sandbox);
    // P1b S5：TimerBridge 注 __zw_setTimeout——shim setTimeout/setInterval 真实延迟
    // （子线程 sleep + resolver.resolve → __zwResolveCallback 调用 JS 回调）。
    let timer_bridge = TimerBridge::new(resolver);
    timer_bridge.register(&mut *sandbox);
    let shim = generate_js_dom_shim();
    if let Err(e) = sandbox.execute(shim) {
        tracing::error!("JS DOM shim init failed: {e}");
    }

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            JsWorkerCommand::Execute { script, reply } => {
                let full = format!("__zw_begin_script && __zw_begin_script();\n{script}");
                let result = sandbox.execute(&full).map(|r| r.value).map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
            JsWorkerCommand::ExecuteModule {
                source,
                url,
                deps,
                reply,
            } => {
                let result = execute_module_in_sandbox(&mut *sandbox, &source, &url, &deps);
                let _ = reply.send(result);
            }
            JsWorkerCommand::SetDomSnapshot { html, url } => {
                // P1a form input：URL 变化（导航）→ 清 shim value 缓存，防跨页同选择器 stale value。
                let url_changed = page_url.lock().map(|u| *u != url).unwrap_or(true);
                if let Ok(mut snap) = dom_html.lock() {
                    *snap = html;
                }
                if let Ok(mut u) = page_url.lock() {
                    *u = url;
                }
                if url_changed {
                    let _ = sandbox.execute("__zw_reset_form_state && __zw_reset_form_state();");
                }
            }
            JsWorkerCommand::ResolveAsyncCallback { id, result } => {
                // P1b S1：跨线程 marshal 到此——在 JS worker 线程调 resolve_async_callback
                // （执行 shim 的 __zwResolveCallback resolve Promise）。
                sandbox.resolve_async_callback(&id, &result);
            }
            JsWorkerCommand::SetFetchHandler { handler } => {
                // P1b S3：注入 fetch handler（renderer 在 WebView 初始化后发送）。
                fetch_bridge.set_handler(handler);
            }
            JsWorkerCommand::Shutdown => break,
        }
    }
}

/// P1b S3：生产 fetch handler——经 `zero_webview::fetch_text_async`（net pool 自动 OnceLock
/// 初始化）发起 HTTP GET，`recv()` 等 response body。renderer 进程直接联网（与 browser
/// `tab_js_worker::default_fetch_handler` 同实现；net pool 共享）。
///
/// `FetchBridge::register` 在**子线程**调本 handler（`recv()` 阻塞子线程，非 JS worker），
/// 故 JS worker 不在 fetch 期间冻结。response 为 body 字符串（Response 对象 spec-compliance
/// 由 shim `_makeResponse` 在 JS 侧包装）。
pub fn default_fetch_handler() -> FetchHandler {
    Arc::new(|url: &str| {
        fetch_text_async(url)
            .recv()
            .map_err(|e| format!("fetch recv: {e}"))
            .and_then(|r| r)
    })
}

fn execute_module_in_sandbox(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    source: &str,
    url: &str,
    deps: &[(String, String)],
) -> Result<String, String> {
    let mut registry = ModuleRegistry::new();
    for (spec, src) in deps {
        registry.register(spec, src);
    }
    let prelude = build_module_runtime_prelude(&registry).map_err(|e| e.to_string())?;
    let transformed = compile_module_script(source, url, &registry).map_err(|e| e.to_string())?;
    let full = format!("{prelude}\n{transformed}");
    sandbox.execute(&full).map(|r| r.value).map_err(|e| e.to_string())
}

fn register_module_compile_callback(sandbox: &mut dyn zero_script_sandbox::Sandbox) {
    // 动态 `import()` 仍直连网络；静态模块依赖由主线程 prefetch + collect_module_deps 经 IPC 加载。
    let http = zero_net::client::HttpClient::new();
    let runtime_iifes: Arc<std::sync::Mutex<HashMap<String, String>>> = Arc::new(std::sync::Mutex::new(HashMap::new()));

    sandbox.register_callback(
        "__zw_compile_module",
        Box::new(move |args| {
            if args.is_empty() {
                return String::new();
            }
            let spec = &args[0];
            let parent = args.get(1).map(String::as_str).unwrap_or("about:blank");
            let url = zero_engine::resolve_document_url(parent, spec);

            if let Ok(cache) = runtime_iifes.lock() {
                if let Some(iife) = cache.get(&url) {
                    return iife.clone();
                }
                if let Some(iife) = cache.get(spec) {
                    return iife.clone();
                }
            }

            let fetch = |u: &str| -> Result<String, String> {
                http.get(u)
                    .map(|r| String::from_utf8_lossy(&r.body).into_owned())
                    .map_err(|e| e.to_string())
            };

            let mut registry = HashMap::new();
            let src = match fetch(&url) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("fetch module {url}: {e}");
                    return String::new();
                }
            };
            if let Err(e) = collect_module_deps(&fetch, &url, &src, &mut registry) {
                tracing::warn!("module deps {url}: {e}");
                return String::new();
            }
            let mut reg = ModuleRegistry::new();
            for (spec, body) in &registry {
                reg.register(spec, body);
            }
            let iife = match compile_dependency_iife(&url, &reg) {
                Ok(i) => i,
                Err(e) => {
                    tracing::warn!("compile module {url}: {e}");
                    return String::new();
                }
            };
            if let Ok(mut cache) = runtime_iifes.lock() {
                cache.insert(url, iife.clone());
            }
            iife
        }),
    );
}

/// 递归抓取模块依赖图（specifier URL → 源码）。
pub fn collect_module_deps(
    fetch: &dyn Fn(&str) -> Result<String, String>,
    entry_url: &str,
    source: &str,
    registry: &mut HashMap<String, String>,
) -> Result<(), String> {
    if registry.contains_key(entry_url) {
        return Ok(());
    }
    registry.insert(entry_url.to_string(), source.to_string());
    for spec in extract_module_import_specifiers(source) {
        let dep_url = zero_engine::resolve_document_url(entry_url, &spec);
        if !registry.contains_key(&dep_url) {
            let dep_src = fetch(&dep_url)?;
            collect_module_deps(fetch, &dep_url, &dep_src, registry)?;
        }
    }
    Ok(())
}

/// renderer 的 JS worker 实现统一脚本执行器契约（T4）。
impl zero_page_runtime::JsExecutor for RendererJsWorker {
    fn set_dom_snapshot(&self, html: &str, url: &str) {
        self.set_dom_snapshot(html, url)
    }
    fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        self.execute_script_direct(script)
    }
    fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        self.execute_module(source, url, deps)
    }
    fn mutations(&self) -> Arc<std::sync::Mutex<Vec<zero_engine::DomMutation>>> {
        self.mutations()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P1b S3 incr-d（镜像 browser tab_js_worker）：非阻塞 fetch 的 resolve 时机异步——
    /// 轮询 `globalThis.{key}` 直到非 undefined（或超时返当前值）。子线程抓取 → generous
    /// 超时下可靠（非 flaky）。
    fn wait_for_global(worker: &RendererJsWorker, key: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe)
                && v != "undefined"
            {
                return v;
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    // P1a Slice 2b：轮询直到 `globalThis.{key}` === want。observer tick 回调经 `_defer`
    // （queueMicrotask）在 execute 末尾 checkpoint drain；probe 本身触发 drain，故即便 drain
    // 跨 execute 也能在下一轮 probe 捕获。带超时兜底。
    fn wait_eq(worker: &RendererJsWorker, key: &str, want: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe)
                && v == want
            {
                return v;
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn renderer_js_worker_async_resolver_delivers_cross_command() {
        // P1b S1（镜像 browser）：跨命令 marshal 验证。JS 建 pending Promise → 主线程经
        // async_resolver().resolve() 投递 ResolveAsyncCallback → worker FIFO 后于该命令的
        // 下一条 Execute 读到已 resolve 的 __result。
        let mut worker = RendererJsWorker::spawn(1);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        let init = worker.execute_script_direct(
            "new Promise(function(resolve){ globalThis.__zw_pending['r1'] = resolve; })
                 .then(function(v){ globalThis.__result = v; });",
        );
        assert!(init.is_ok(), "init script should succeed: {:?}", init.err());
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__result").unwrap(),
            "undefined"
        );
        let resolver = worker.async_resolver();
        resolver.resolve("r1", "delivered!");
        // FIFO：resolve 命令先于下一条 Execute 入队 → worker 先 resolve 后读。
        assert_eq!(
            worker.execute_script_direct("globalThis.__result").unwrap(),
            "delivered!"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_async_resolver_safe_for_unknown_id() {
        // 未知 id（无 pending resolver）经 shim 防御分支静默 no-op，不报错/不崩溃。
        let mut worker = RendererJsWorker::spawn(2);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.async_resolver().resolve("nonexistent-id", "v");
        assert_eq!(worker.execute_script_direct("1 + 2").unwrap(), "3");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_async_resolver_usable_from_other_thread() {
        // AsyncResolver: Send（可 move 到子线程）+ Arc<Mutex> clone 跨线程工作
        // （仿真实 fetch host / 定时器跨线程完成）。
        let mut worker = RendererJsWorker::spawn(3);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "new Promise(function(r){ globalThis.__zw_pending['t1'] = r; })
                 .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        let resolver = worker.async_resolver();
        let handle = std::thread::spawn(move || {
            resolver.resolve("t1", "from-thread!");
        });
        handle.join().unwrap();
        assert_eq!(
            worker.execute_script_direct("globalThis.__result").unwrap(),
            "from-thread!"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_resolves_via_handler() {
        // P1b S3（镜像 browser）：fetch 经 __zw_fetch 回调 + handler 抓取 + resolver.resolve
        // 端到端。合成 handler 返 body:<url>；resolve Response 对象（r.text() 取 body）。
        let mut worker = RendererJsWorker::spawn(4);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|url: &str| Ok(format!("body:{url}"))));
        worker
            .execute_script_direct(
                "fetch('/hello').then(function(r){ return r.text(); })
                 .then(function(t){ globalThis.__result = t; });",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "body:/hello");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_without_handler_resolves_error() {
        // 未注入 handler 时 __zw_fetch resolve 错误标记 → Response.ok=false（不悬挂）。
        let mut worker = RendererJsWorker::spawn(5);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct("fetch('/x').then(function(r){ globalThis.__result = r.ok ? 'OK' : 'ERR'; });")
            .unwrap();
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "ERR");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_response_object_shape_and_json() {
        // P1b S3 incr-c（镜像 browser）：Response 对象 spec-compliance（ok/status/text()/json()）。
        let mut worker = RendererJsWorker::spawn(7);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|_url: &str| Ok("{\"key\":\"value\",\"n\":42}".to_string())));
        worker
            .execute_script_direct(
                "fetch('/j').then(function(r){
                   globalThis.__shape = r.ok + ':' + r.status;
                   return r.json();
                 }).then(function(o){ globalThis.__result = o.key + '/' + o.n; });",
            )
            .unwrap();
        assert_eq!(wait_for_global(&worker, "__shape", 1000), "true:200");
        assert_eq!(wait_for_global(&worker, "__result", 1000), "value/42");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_default_fetch_handler_real_http() {
        // P1b S3（镜像 browser）：生产 default_fetch_handler 经 net pool 真实 HTTP GET。
        // 本地 HTTP server（127.0.0.1）服务固定 body——不依赖外部网络。非阻塞：子线程 recv
        // （不冻结 JS worker），测试 polling（3s 超时，本地 fetch ~ms）。
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local server");
        let port = listener.local_addr().expect("local addr").port();
        let server = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf); // 丢弃请求行
                let body = "hello-from-renderer";
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        let mut worker = RendererJsWorker::spawn(6);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(default_fetch_handler());
        let url = format!("http://127.0.0.1:{port}/data");
        worker
            .execute_script_direct(&format!(
                "fetch({:?}).then(function(r){{ return r.text(); }})
                 .then(function(t){{ globalThis.__result = t; }});",
                url
            ))
            .unwrap();
        let r = wait_for_global(&worker, "__result", 3000);
        assert_eq!(r, "hello-from-renderer");
        worker.shutdown();
        let _ = server.join();
    }

    #[test]
    fn renderer_js_worker_settimeout_fires_after_real_delay() {
        // P1b S5（镜像 browser）：setTimeout 真实延迟。host __zw_setTimeout → 子线程 sleep
        // 后 resolve → __zwResolveCallback 调用回调。
        let mut worker = RendererJsWorker::spawn(8);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct("setTimeout(function(){ globalThis.__fired = 'yes'; }, 50);")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        let r = wait_for_global(&worker, "__fired", 1000);
        assert_eq!(r, "yes");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_cleartimeout_cancels_callback() {
        // P1b S5（镜像 browser）：clearTimeout 删 pending → 回调永不触发。
        let mut worker = RendererJsWorker::spawn(9);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var h = setTimeout(function(){ globalThis.__fired = 'yes'; }, 30);
                 clearTimeout(h);",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_setinterval_repeats_then_clear() {
        // P1b S5（镜像 browser）：setInterval re-arm 重复触发；clearInterval 断链。
        let mut worker = RendererJsWorker::spawn(10);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__n = 0;
                 globalThis.__iv = setInterval(function(){ globalThis.__n++; }, 20);",
            )
            .unwrap();
        // 轮询等待 setInterval 至少触发 2 次（R2149：原固定 sleep 120ms 在 `make test` 全
        // workspace 并行负载下偶发 worker 线程饿死 → n1<2 false-fail；改条件式轮询
        // robust-to-starvation，1000ms 充分覆盖调度延迟）。镜像 browser 侧同名测试。
        let mut n1: u32 = 0;
        let poll_start = std::time::Instant::now();
        while n1 < 2 && poll_start.elapsed().as_millis() < 1000 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            n1 = worker
                .execute_script_direct("String(globalThis.__n)")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0);
        }
        assert!(n1 >= 2, "setInterval should repeat at least twice, got {n1}");
        worker.execute_script_direct("clearInterval(globalThis.__iv);").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(120));
        let n2 = worker
            .execute_script_direct("String(globalThis.__n)")
            .unwrap()
            .parse::<u32>()
            .unwrap_or(n1);
        assert_eq!(n2, n1, "clearInterval should stop the interval (n stayed {n1})");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_childlist() {
        // P1b S2 incr1（镜像 browser）：MutationObserver handle-based，JS 创建子树。
        let mut worker = RendererJsWorker::spawn(11);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].addedNodes.length;
                 });
                 var root = document.createElement('div');
                 obs.observe(root, { childList: true });
                 root.appendChild(document.createElement('span'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "childList:1");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_attributes() {
        // P1b S2 incr1（镜像 browser）：attributes 观测。
        let mut worker = RendererJsWorker::spawn(12);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.createElement('div');
                 obs.observe(el, { attributes: true });
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:data-x");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_disconnect() {
        // P1b S2 incr1（镜像 browser）：disconnect 后不再派发。
        let mut worker = RendererJsWorker::spawn(13);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){ globalThis.__seen = 'fired'; });
                 var el = document.createElement('div');
                 obs.observe(el, { attributes: true });
                 obs.disconnect();
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_existing_dom_attributes() {
        // P1b S2 incr2（镜像 browser）：观测现有 DOM attributes（selector 身份）。
        let mut worker = RendererJsWorker::spawn(14);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.querySelector('#t');
                 obs.observe(el, { attributes: true });
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:data-x");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_existing_dom_childlist() {
        // P1b S2 incr2（镜像 browser）：观测现有 DOM childList。
        let mut worker = RendererJsWorker::spawn(15);
        worker.set_dom_snapshot("<html><body><ul id='list'></ul></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].addedNodes.length;
                 });
                 var list = document.querySelector('#list');
                 obs.observe(list, { childList: true });
                 list.appendChild(document.createElement('li'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "childList:1");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_element_identity_stable_proxy() {
        // P1b S2 incr3（镜像 browser）：=== node identity——Proxy 缓存。
        let mut worker = RendererJsWorker::spawn(16);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var a = document.querySelector('#t');
                 var b = document.querySelector('#t');
                 globalThis.__same = (a === b);
                 var c1 = document.createElement('div');
                 var c2 = document.createElement('div');
                 globalThis.__diff = (c1 !== c2);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__same)").unwrap(),
            "true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__diff)").unwrap(),
            "true"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_real_rect() {
        // P1a gBCR path C：selector-identity 元素的 getBoundingClientRect 返真实 DOMRect。
        // shim `__zw_getBoundingClientRect(sel)` → handler fresh-parse dom_html → find_by_selector
        // → NodeId → 查 rect_snapshot。本测试用「同一 html fresh-parse」的 NodeId 填 snapshot
        // （模拟 renderer 主循环 render 后填充；NodeId 确定性由 engine 的
        // `test_node_id_determinism_across_fresh_parses` 保证 = 渲染管线会用同一 NodeId）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(18);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 填 snapshot：解析同一 html 取 #t 的 NodeId（= 渲染管线会用的 NodeId），插入其 rect。
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        // 读 gBCR：width/left/top 应反映 snapshot（rect 反映「上次 render」，此处 snapshot 即该 render）。
        worker
            .execute_script_direct(
                "var r = document.querySelector('#t').getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__l = r.left; globalThis.__t = r.top;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__w)").unwrap(), "100");
        assert_eq!(worker.execute_script_direct("String(globalThis.__l)").unwrap(), "10");
        assert_eq!(worker.execute_script_direct("String(globalThis.__t)").unwrap(), "20");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_empty_snapshot_zero() {
        // 零回归：snapshot 未填（无 render / 未命中）→ handler 返 None → shim 回落零 rect
        // （= 旧行为；作 reflow 触发器语义仍正确，返回值多被丢弃）。
        let mut worker = RendererJsWorker::spawn(19);
        worker.set_dom_snapshot("<html><body><div id='t'>hi</div></body></html>", "about:blank");
        worker
            .execute_script_direct("globalThis.__w = document.querySelector('#t').getBoundingClientRect().width;")
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__w)").unwrap(), "0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_intersecting() {
        // P1a Slice 2a：observe 视口内元素 → spec initial notification 派发，isIntersecting=true、
        // ratio≈1（target 完全在 viewport 内）。复用 gBCR：snapshot 填 #t rect，IO 经
        // `__zw_getBoundingClientRect(sel)` 算与 viewport 重叠（sel = `__zw_query_match('#t')` 返回值，
        // 与本测试 `find_by_selector(&doc, "#t")` 同 NodeId，见 gBCR test 既有验证）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(20);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0)); // 视口内（innerWidth/Height=1280/800）
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#t');\
                 var obs = new IntersectionObserver(function(entries){\
                   var e = entries[0];\
                   globalThis.__seen = String(e.isIntersecting) + ':' + String(e.target === el)\
                     + ':' + (e.intersectionRatio > 0.99 ? 'full' : 'partial');\
                 });\
                 obs.observe(el);",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "true:true:full");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_not_intersecting_initial() {
        // P1a Slice 2a：observe 视口外元素 → spec 仍派发 initial notification，isIntersecting=false、ratio=0。
        // （旧 shim 无 IO → `new IntersectionObserver` 抛 ReferenceError 中断脚本；本切片消除之。）
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(21);
        let html = "<html><body><div id='t' style='width:10px;height:10px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (2000.0, 2000.0, 10.0, 10.0)); // 视口外（>1280/800）
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new IntersectionObserver(function(entries){\
                   globalThis.__seen = String(entries[0].isIntersecting) + ':' + String(entries[0].intersectionRatio);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "false:0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_disconnect() {
        // P1a Slice 2a：observe 后 disconnect（microtask 派发前）→ _targets 清空 → callback 不派发。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(22);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new IntersectionObserver(function(_entries){ globalThis.__seen = 'fired'; });\
                 obs.observe(document.querySelector('#t'));\
                 obs.disconnect();",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_initial() {
        // P1a Slice 3：observe → spec initial notification 派发，contentRect.width/height 匹配
        // snapshot 尺寸（复用 gBCR：snapshot 填 #t rect，RO 经 `__zw_getBoundingClientRect(sel)` 读取）。
        // sel = `__zw_query_match('#t')` 返回值，与本测试 `find_by_selector(&doc, "#t")` 同 NodeId
        // （见 gBCR test 既有确定性验证）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(23);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#t');\
                 var obs = new ResizeObserver(function(entries){\
                   var e = entries[0];\
                   globalThis.__seen = String(e.target === el) + ':' + String(e.contentRect.width)\
                     + 'x' + String(e.contentRect.height) + ':' + String(e.borderBoxSize[0].inlineSize);\
                 });\
                 obs.observe(el);",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "true:100x50:100");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_zero_fallback() {
        // P1a Slice 3：gBCR 未命中（snapshot 空）→ contentRect 为零，仍派发 initial notification（no-throw）。
        // （旧 shim 无 RO → `new ResizeObserver` 抛 ReferenceError 中断脚本；本切片消除之。）
        let mut worker = RendererJsWorker::spawn(24);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new ResizeObserver(function(entries){\
                   globalThis.__seen = String(entries[0].contentRect.width) + 'x'\
                     + String(entries[0].contentRect.height);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "0x0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_disconnect() {
        // P1a Slice 3：observe 后 disconnect（microtask 派发前）→ _targets 清空 → callback 不派发。
        let mut worker = RendererJsWorker::spawn(25);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new ResizeObserver(function(_entries){ globalThis.__seen = 'fired'; });\
                 obs.observe(document.querySelector('#t'));\
                 obs.disconnect();",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_observer_tick_refires_on_size_change() {
        // P1a Slice 2b：observe（initial 派发，snapshot v1 100x50）→ 更新 snapshot（size 变化 200x80）
        // → `__zw_observers_tick` → RO size-diff 再次派发回调（__calls 1→2，__last 200x80）。
        // size 未变再 tick → 不派发（_lastSize 守，__calls 仍 2）。证明 host render-loop tick 机制。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(26);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (0.0, 0.0, 100.0, 50.0)); // v1
        worker
            .execute_script_direct(
                "globalThis.__calls = 0;\
                 globalThis.__last = '';\
                 var obs = new ResizeObserver(function(entries){\
                   globalThis.__calls = (globalThis.__calls | 0) + 1;\
                   globalThis.__last = String(entries[0].contentRect.width) + 'x' + String(entries[0].contentRect.height);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        // initial 派发（microtask 在 execute 末尾 checkpoint drain；wait_eq probe 兜底）。
        assert_eq!(wait_eq(&worker, "__calls", "1", 1000), "1");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__last)").unwrap(),
            "100x50"
        );
        // 更新 snapshot → size 变化 → tick 再次派发。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (0.0, 0.0, 200.0, 80.0));
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        assert_eq!(wait_eq(&worker, "__calls", "2", 1000), "2");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__last)").unwrap(),
            "200x80"
        );
        // size 未变再 tick → 不派发（_lastSize 守，__calls 仍 2）。
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(80));
        assert_eq!(worker.execute_script_direct("String(globalThis.__calls)").unwrap(), "2");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_text_input_updates_value_and_fires_input() {
        // P1a form input：`__zw_text_input(sel, ch)` 对 input 元素 append char 到 `.value`（缓存，
        // listener 立即可见新值）+ 派发 'input' 事件。`.value` lazy-init 自 value 属性（"ab"），
        // 注入 "c" → "abc"，input listener 读 `el.value` 见 "abc"（不滞后 mutation-apply）。
        // 非 input/textarea 目标 → no-op（不派发 input）。
        let mut worker = RendererJsWorker::spawn(27);
        worker.set_dom_snapshot("<html><body><input id='i' value='ab'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#i');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_input('#i', 'c');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abc"
        );
        // 第二次注入 "d" → "abcd"（缓存跨 execute 存活，多键 typing 成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_input('#i', 'd');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abcd"
        );
        // 非 input 目标（body）→ no-op，无 input 派发。
        worker
            .execute_script_direct(
                "globalThis.__seen2 = 'unchanged';\
                 __zw_text_input('body', 'x');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen2)").unwrap(),
            "unchanged"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_text_delete_removes_last_char_and_fires_input() {
        // P1a form input 编辑互补：`__zw_text_delete(sel)` 删 value 末字符 + 派发 'input'。
        // "abcd" → backspace → "abc"，listener 见新值。空值 backspace → 无变化不派发（同 real browser）。
        let mut worker = RendererJsWorker::spawn(28);
        worker.set_dom_snapshot("<html><body><input id='i' value='abcd'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#i');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_delete('#i');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abc"
        );
        // 再删 → "ab"（多键成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_delete('#i');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab"
        );
        // 删到空后再删 → 无 input 派发（__seen 保持 null）。
        worker.execute_script_direct("__zw_text_delete('#i');").unwrap(); // "a"
        worker.execute_script_direct("__zw_text_delete('#i');").unwrap(); // ""
        worker
            .execute_script_direct("globalThis.__seen = 'sentinel'; __zw_text_delete('#i');")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "sentinel"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_submit_dispatches_submit_event() {
        // P1a form submit：apply_submit_on_enter 经 script_dispatch_dom_event(form_sel,"submit")
        // → 即 `__zw_dispatch_event(form_sel, 'submit', null)`。本 driving test 验证 submit 事件
        // 经 shim 派发命中 form 的 submit listener（form-resolution 由 engine 单测覆盖）。
        let mut worker = RendererJsWorker::spawn(29);
        worker.set_dom_snapshot(
            "<html><body><form id='f'><input id='i'></form></body></html>",
            "about:blank",
        );
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 document.querySelector('#f').addEventListener('submit', function(_e){\
                   globalThis.__seen = 'submit-fired';\
                 });\
                 __zw_dispatch_event('#f', 'submit', null);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "submit-fired"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_property_set() {
        // P1b S2 incr3（镜像 browser）：property set（el.className='x'）触发 attributes 记录。
        let mut worker = RendererJsWorker::spawn(17);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.querySelector('#t');
                 obs.observe(el, { attributes: true });
                 el.className = 'active';",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:class");
        worker.shutdown();
    }
}
