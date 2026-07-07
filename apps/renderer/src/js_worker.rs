//! 渲染进程 JS 线程 — V8 与页面渲染分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_engine::{DomMutation, generate_js_dom_shim, register_dom_callbacks};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, V8Sandbox, build_module_runtime_prelude, compile_dependency_iife,
    compile_module_script, extract_module_import_specifiers,
};

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
        let mutations_for_worker = Arc::clone(&mutations);

        let join = thread::Builder::new()
            .name(format!("renderer-js-{}", renderer_id))
            .spawn(move || js_worker_main(cmd_rx, mutations_for_worker))
            .expect("spawn tab js worker");

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

fn js_worker_main(cmd_rx: Receiver<JsWorkerCommand>, mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>) {
    let js_config = SandboxConfig {
        persistent_context: true,
        timeout_ms: TAB_JS_EXEC_TIMEOUT_MS,
        ..Default::default()
    };
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(V8Sandbox::with_config(js_config).expect("V8 sandbox init"));
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::QuickJSSandbox::with_config(js_config).expect("QuickJS sandbox init"));
    let dom_html: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::new()));
    let page_url: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::from("about:blank")));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    register_module_compile_callback(&mut sandbox);
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
                let result = execute_module_in_sandbox(&mut sandbox, &source, &url, &deps);
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
    // 动态 `import()` 仍直连网络；静态模块依赖由主线程 prefetch + collect_module_deps 经 IPC 加载。
    let http = zero_net::client::HttpClient::new();
    let runtime_iifes: Arc<std::sync::Mutex<HashMap<String, String>>> = Arc::new(std::sync::Mutex::new(HashMap::new()));

    sandbox.register_callback("__zw_compile_module", move |args| {
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
    });
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
