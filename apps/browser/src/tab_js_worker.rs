//! 标签页专用 JS 线程 — V8 与布局/绘制 worker 分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_engine::{DomMutation, generate_js_dom_shim, register_dom_callbacks};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, build_module_runtime_prelude, compile_dependency_iife, compile_module_script,
    extract_module_import_specifiers,
};

/// 页面 `<script>` 执行超时（毫秒）— 短于事件派发，避免死循环拖死 tab worker。
pub const PAGE_SCRIPT_TIMEOUT_MS: u64 = 2_500;
/// 宿主事件 / `execute_script_direct` 超时。
pub const TAB_JS_EVENT_TIMEOUT_MS: u64 = 5_000;

fn channel_timeout_for(exec_timeout_ms: u64) -> Duration {
    Duration::from_millis(exec_timeout_ms + 2_000)
}

type ScriptFn = Arc<dyn Fn(&str, u64) -> Result<String, String> + Send + Sync>;
type ModuleFn = Arc<dyn Fn(&str, &str, &[(String, String)]) -> Result<String, String> + Send + Sync>;
type ExecutorFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

enum JsWorkerCommand {
    Execute {
        script: String,
        timeout_ms: u64,
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
    /// P1b S1 incr3：跨线程异步回调 resolve（marshal channel）。任意线程经
    /// [`TabJsWorkerHandle::async_resolver`] 投递 (id, result)，JS worker 收到后调
    /// `sandbox.resolve_async_callback`（执行 shim 的 `__zwResolveCallback` resolve Promise）。
    /// 复用现有 cmd mpsc（FIFO 保序），无需独立 channel / select。
    ResolveAsyncCallback {
        id: String,
        result: String,
    },
    Shutdown,
}

/// 异步回调 resolver（P1b S1 incr3）——克隆供跨线程异步完成方（fetch host / 定时器）
/// 持有，把 `(id, result)` 投递回 JS worker 线程 resolve 对应 Promise。
#[derive(Clone)]
pub struct AsyncResolver {
    cmd_tx: Sender<JsWorkerCommand>,
}

impl AsyncResolver {
    /// 投递一次异步 resolve（fire-and-forget）。JS worker 收到后执行
    /// `__zwResolveCallback(id, result)` resolve pending Promise。
    pub fn resolve(&self, id: &str, result: &str) {
        let _ = self.cmd_tx.send(JsWorkerCommand::ResolveAsyncCallback {
            id: id.to_string(),
            result: result.to_string(),
        });
    }
}

/// 专用 JS worker 句柄（每 Tab 一个）。
pub struct TabJsWorkerHandle {
    cmd_tx: Sender<JsWorkerCommand>,
    join: Option<JoinHandle<()>>,
    executor: ScriptFn,
    module_executor: ModuleFn,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
}

impl TabJsWorkerHandle {
    /// 启动 JS 专用线程。
    pub fn spawn(tab_id: TabId) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);

        let join = thread::Builder::new()
            .name(format!("tab-js-{}", tab_id.0))
            .spawn(move || js_worker_main(cmd_rx, mutations_for_worker))
            .expect("spawn tab js worker");

        let executor: ScriptFn = Arc::new(move |script: &str, timeout_ms: u64| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_exec
                .send(JsWorkerCommand::Execute {
                    script: script.to_string(),
                    timeout_ms,
                    reply: reply_tx,
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(channel_timeout_for(timeout_ms))
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
                .recv_timeout(channel_timeout_for(TAB_JS_EVENT_TIMEOUT_MS))
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

    /// 供 WebView 注入的外部脚本执行器（事件派发等，较长超时）。
    pub fn executor(&self) -> ExecutorFn {
        let exec = Arc::clone(&self.executor);
        Arc::new(move |script: &str| exec(script, TAB_JS_EVENT_TIMEOUT_MS))
    }

    /// 执行页面 `<script>`（较短超时，减轻死循环对界面的影响）。
    pub fn execute_page_script(&self, script: &str) -> Result<String, String> {
        (self.executor)(script, PAGE_SCRIPT_TIMEOUT_MS)
    }

    /// 执行 ES module（含依赖注册表）。
    pub fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        (self.module_executor)(source, url, deps)
    }

    /// 在 JS 线程执行脚本（不经 WebView 包装）。
    pub fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        (self.executor)(script, TAB_JS_EVENT_TIMEOUT_MS)
    }

    /// 脚本执行前更新 DOM HTML 快照与页面 URL（供 querySelector、location 等只读回调）。
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

    /// 返回异步回调 resolver（P1b S1 incr3）。克隆供跨线程异步完成方（fetch host /
    /// 定时器）持有，`resolver.resolve(id, result)` 经 cmd channel marshal 回 JS worker
    /// 线程，由 worker 调 `sandbox.resolve_async_callback` resolve 对应 Promise。
    pub fn async_resolver(&self) -> AsyncResolver {
        AsyncResolver {
            cmd_tx: self.cmd_tx.clone(),
        }
    }

    /// 关闭 JS 线程。
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(JsWorkerCommand::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for TabJsWorkerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn js_worker_main(cmd_rx: Receiver<JsWorkerCommand>, mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>) {
    let js_config = SandboxConfig {
        persistent_context: true,
        timeout_ms: TAB_JS_EVENT_TIMEOUT_MS,
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
    let shim = generate_js_dom_shim();
    if let Err(e) = sandbox.execute(shim) {
        tracing::error!("JS DOM shim init failed: {e}");
    }

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            JsWorkerCommand::Execute {
                script,
                timeout_ms,
                reply,
            } => {
                sandbox.set_timeout_ms(timeout_ms);
                let full = format!("__zw_begin_script && __zw_begin_script();\n{script}");
                let result = sandbox.execute(&full).map(|r| r.value).map_err(|e| e.to_string());
                sandbox.set_timeout_ms(TAB_JS_EVENT_TIMEOUT_MS);
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
                // P1b S1 incr3：跨线程 marshal 到此——在 JS worker 线程调
                // resolve_async_callback（执行 shim 的 __zwResolveCallback resolve Promise）。
                sandbox.resolve_async_callback(&id, &result);
            }
            JsWorkerCommand::Shutdown => break,
        }
    }
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

/// tabworker 的 JS worker 实现统一脚本执行器契约（T4）——与 renderer 的 RendererJsWorker 同契约。
impl zero_page_runtime::JsExecutor for TabJsWorkerHandle {
    fn set_dom_snapshot(&self, html: &str, url: &str) {
        self.set_dom_snapshot(html, url)
    }
    fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        self.execute_script_direct(script)
    }
    fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        self.execute_module(source, url, deps)
    }
    fn mutations(&self) -> std::sync::Arc<std::sync::Mutex<Vec<zero_engine::DomMutation>>> {
        self.mutations()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_browser_shell::TabId;

    #[test]
    fn tab_js_worker_async_resolver_delivers_cross_command() {
        // P1b S1 incr3：跨命令 marshal 验证。JS 建 pending Promise → 主线程经
        // async_resolver().resolve() 投递 ResolveAsyncCallback → worker FIFO 后于该命令的
        // 下一条 Execute 读到已 resolve 的 __result（证跨线程/跨命令 resolve 通路工作）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(1));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        // 建 pending Promise on id "r1"，then 写全局 __result。
        let init = worker.execute_script_direct(
            "new Promise(function(resolve){ globalThis.__zw_pending['r1'] = resolve; })
                 .then(function(v){ globalThis.__result = v; });",
        );
        assert!(init.is_ok(), "init script should succeed: {:?}", init.err());
        // resolve 前：__result 未设（Promise pending）。
        let before = worker.execute_script_direct("typeof globalThis.__result").unwrap();
        assert_eq!(before, "undefined");
        // 经 async_resolver 投递 resolve（cmd channel → worker → sandbox.resolve_async_callback）。
        let resolver = worker.async_resolver();
        resolver.resolve("r1", "delivered!");
        // FIFO：resolve 命令先于下一条 Execute 入队 → worker 先 resolve 后读。
        let after = worker.execute_script_direct("globalThis.__result").unwrap();
        assert_eq!(after, "delivered!");
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_async_resolver_safe_for_unknown_id() {
        // 未知 id（无 pending resolver）经 shim 防御分支静默 no-op，不报错/不崩溃。
        let mut worker = TabJsWorkerHandle::spawn(TabId(2));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        let resolver = worker.async_resolver();
        resolver.resolve("nonexistent-id", "v");
        // worker 仍正常服务。
        let r = worker.execute_script_direct("1 + 2").unwrap();
        assert_eq!(r, "3");
        worker.shutdown();
    }
}
