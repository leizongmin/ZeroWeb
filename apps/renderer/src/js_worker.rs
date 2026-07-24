//! 渲染进程 JS 线程 — V8 与页面渲染分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_engine::{
    AsyncResolver, DomMutation, FetchBridge, FetchHandler, TimerBridge, generate_js_dom_shim, register_dom_callbacks,
};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, build_module_runtime_prelude, compile_dependency_iife, compile_module_script,
    extract_module_import_specifiers,
};
use zero_webview::fetch_text_async;

const TAB_JS_EXEC_TIMEOUT_MS: u64 = 15_000;
const TAB_JS_CHANNEL_TIMEOUT: Duration = Duration::from_millis(TAB_JS_EXEC_TIMEOUT_MS + 5_000);

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
}

impl RendererJsWorker {
    /// 启动 JS 专用线程。
    pub fn spawn(renderer_id: u64) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let cmd_for_worker = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);

        let join = thread::Builder::new()
            .name(format!("renderer-js-{}", renderer_id))
            .spawn(move || js_worker_main(cmd_rx, cmd_for_worker, mutations_for_worker))
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
                if let Ok(mut snap) = dom_html.lock() {
                    *snap = html;
                }
                if let Ok(mut u) = page_url.lock() {
                    *u = url;
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
        std::thread::sleep(std::time::Duration::from_millis(120));
        let n1 = wait_for_global(&worker, "__n", 500).parse::<u32>().unwrap_or(0);
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
}
