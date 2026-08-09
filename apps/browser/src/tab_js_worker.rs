//! 标签页专用 JS 线程 — V8 与布局/绘制 worker 分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_engine::{
    AsyncResolver, DomMutation, ElementFromPointBridge, ElementFromPointCache, FetchBridge, FetchHandler, FetchRequest,
    FetchResponse, HandleSelectorMap, LayoutRectSnapshot, RectBridge, TimerBridge, generate_js_dom_shim,
    make_dom_html_rect_handler, new_element_from_point_cache, new_handle_selector_map, new_layout_rect_snapshot,
    register_dom_callbacks,
};
use zero_net::{HttpClient, HttpMethod, HttpRequest};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, build_module_runtime_prelude, compile_dependency_iife, compile_module_script,
    extract_module_import_specifiers,
};

/// 页面 `<script>` 执行超时（毫秒）— 短于事件派发，避免死循环拖死 tab worker。
pub const PAGE_SCRIPT_TIMEOUT_MS: u64 = 2_500;
/// 宿主事件 / `execute_script_direct` 超时。
pub const TAB_JS_EVENT_TIMEOUT_MS: u64 = 5_000;

/// P1a gBCR kill-switch：默认 on；`ZW_REAL_RECT=0` 关闭 RectBridge（`__zw_getBoundingClientRect`
/// 不注册 → shim 回落零 rect = 当前行为，零回归）。与 renderer `js_worker::real_rect_enabled` 同实现。
fn real_rect_enabled() -> bool {
    !matches!(std::env::var("ZW_REAL_RECT").as_deref(), Ok("0"))
}

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
    /// P1b S3 incr-a：注入 fetch handler（tab_worker 在 WebView 初始化后发送；
    /// 测试用合成 handler）。js_worker_main 存入 `Arc<Mutex<Option<FetchHandler>>>`
    /// 供 `__zw_fetch` 回调读取。chicken-and-egg 解：js_worker spawn 时 WebView 未就绪。
    SetFetchHandler {
        handler: FetchHandler,
    },
    Shutdown,
}

/// 专用 JS worker 句柄（每 Tab 一个）。
pub struct TabJsWorkerHandle {
    cmd_tx: Sender<JsWorkerCommand>,
    join: Option<JoinHandle<()>>,
    executor: ScriptFn,
    module_executor: ModuleFn,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    /// P1a gBCR：共享 layout-rect snapshot——tab_worker render 后填充，
    /// js_worker 的 RectBridge handler 读取（经 identity→NodeId 解析后查 rect）。
    rect_snapshot: LayoutRectSnapshot,
    /// P1a gBCR path A：持久 handle→唯一选择器映射——`tab_scripts::apply_recorded_mutations`
    /// merge 进此 map，js_worker 的 RectBridge handler 读它解析 handle-identity（`__n{n}`）。
    handle_selector_map: HandleSelectorMap,
    /// P1a elementFromPoint：共享 hit-test 缓存槽——tab_worker render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    element_from_point_cache: ElementFromPointCache,
}

impl TabJsWorkerHandle {
    /// 启动 JS 专用线程。
    pub fn spawn(tab_id: TabId) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let rect_snapshot = new_layout_rect_snapshot();
        let handle_selector_map = new_handle_selector_map();
        let element_from_point_cache = new_element_from_point_cache();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let cmd_for_worker = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);
        let rect_snapshot_for_worker = Arc::clone(&rect_snapshot);
        let handle_selector_map_for_worker = Arc::clone(&handle_selector_map);
        let element_from_point_cache_for_worker = Arc::clone(&element_from_point_cache);

        let join = thread::Builder::new()
            .name(format!("tab-js-{}", tab_id.0))
            .spawn(move || {
                js_worker_main(
                    cmd_rx,
                    cmd_for_worker,
                    mutations_for_worker,
                    rect_snapshot_for_worker,
                    handle_selector_map_for_worker,
                    element_from_point_cache_for_worker,
                )
            })
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
            rect_snapshot,
            handle_selector_map,
            element_from_point_cache,
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

    /// P1a gBCR：共享 layout-rect snapshot 句柄——tab_worker render 后经
    /// `HitTestCache::fill_layout_rect_snapshot` 填充，js_worker 的 RectBridge handler 读取。
    pub fn rect_snapshot(&self) -> LayoutRectSnapshot {
        Arc::clone(&self.rect_snapshot)
    }

    /// P1a gBCR path A：持久 handle→唯一选择器映射句柄——`tab_scripts::apply_recorded_mutations`
    /// merge 进此 map，js_worker 的 RectBridge handler 读它解析 handle-identity（`__n{n}`）。
    pub fn handle_selector_map(&self) -> HandleSelectorMap {
        Arc::clone(&self.handle_selector_map)
    }

    /// P1a elementFromPoint：共享 hit-test 缓存槽句柄——tab_worker render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    pub fn element_from_point_cache(&self) -> ElementFromPointCache {
        Arc::clone(&self.element_from_point_cache)
    }

    /// 返回异步回调 resolver（P1b S1 incr3）。克隆供跨线程异步完成方（fetch host /
    /// 定时器）持有，`resolver.resolve(id, result)` 经 cmd channel marshal 回 JS worker
    /// 线程，由 worker 调 `sandbox.resolve_async_callback` resolve 对应 Promise。
    pub fn async_resolver(&self) -> AsyncResolver {
        let tx = Arc::new(std::sync::Mutex::new(self.cmd_tx.clone()));
        AsyncResolver::new(move |id, result| {
            let _ = tx.lock().unwrap().send(JsWorkerCommand::ResolveAsyncCallback {
                id: id.to_string(),
                result: result.to_string(),
            });
        })
    }

    /// P1b S3 incr-a：注入 fetch handler（解 chicken-and-egg——tab_worker 在 WebView
    /// 初始化后调用，handler 经 WebView/fetch_host 抓取；测试用合成实现）。
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

impl Drop for TabJsWorkerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn js_worker_main(
    cmd_rx: Receiver<JsWorkerCommand>,
    cmd_tx: Sender<JsWorkerCommand>,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    rect_snapshot: LayoutRectSnapshot,
    handle_selector_map: HandleSelectorMap,
    element_from_point_cache: ElementFromPointCache,
) {
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
    // P1a gBCR（镜像 renderer js_worker）：RectBridge 注 `__zw_getBoundingClientRect(identity)`
    // 同步回调。handler 解析 identity(selector) → NodeId（fresh-parse dom_html，与渲染管线确定性一致）
    // → 查 rect_snapshot。kill-switch `ZW_REAL_RECT=0` 关闭（回落零 rect = 当前行为，零回归）。
    if real_rect_enabled() {
        let rect_bridge = RectBridge::new();
        rect_bridge.register(&mut *sandbox);
        rect_bridge.set_handler(make_dom_html_rect_handler(
            Arc::clone(&dom_html),
            Arc::clone(&rect_snapshot),
            Arc::clone(&handle_selector_map),
        ));
    }
    // P1a elementFromPoint（镜像 renderer js_worker）：`ElementFromPointBridge` 注
    // `__zw_elementFromPoint(x, y)` 同步回调。回调锁内 clone `Arc<HitTestCache>`（tab_worker
    // render 后 swap 进共享槽）→ `hit_test_element` + `selector_from_element_hit` → 稳定选择器。
    // 未注入 cache / 无命中 → 空串（shim 返 null）。
    let element_from_point_bridge = ElementFromPointBridge::new(element_from_point_cache);
    element_from_point_bridge.register(&mut *sandbox);
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
                // 导航（URL 变化）→ 旧页 handle 在新页无效，清 handle→selector map（path A）。
                let url_changed = page_url.lock().map(|u| *u != url).unwrap_or(true);
                if let Ok(mut snap) = dom_html.lock() {
                    *snap = html;
                }
                if let Ok(mut u) = page_url.lock() {
                    *u = url;
                }
                if url_changed {
                    if let Ok(mut map) = handle_selector_map.lock() {
                        map.clear();
                    }
                    // R3059：导航 → 清旧页 _hist_entries（pushState/hash-setter 残留），新页 location.href
                    // 读 page_url fallback（= 新文档 url）。与 renderer 路径一致（闭合 SPA-then-redirect stale）。
                    let _ = sandbox.execute("__zw_reset_history && __zw_reset_history();");
                }
            }
            JsWorkerCommand::ResolveAsyncCallback { id, result } => {
                // P1b S1 incr3：跨线程 marshal 到此——在 JS worker 线程调
                // resolve_async_callback（执行 shim 的 __zwResolveCallback resolve Promise）。
                sandbox.resolve_async_callback(&id, &result);
            }
            JsWorkerCommand::SetFetchHandler { handler } => {
                // P1b S3 incr-a：注入 fetch handler（tab_worker 在 WebView 初始化后发送）。
                fetch_bridge.set_handler(handler);
            }
            JsWorkerCommand::Shutdown => break,
        }
    }
}

/// R2923 fetch 完整化：生产 fetch handler——经 `zero_net::HttpClient::send` 发起真实 HTTP 请求，
/// 支持全方法（GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS）、请求头、请求体，返 [`FetchResponse`]（status/
/// status_text/headers/body）。GET 行为零回归（method 默认 GET、body=None）。
///
/// `register_fetch_callback` 在**子线程**调本 handler（`send()` 阻塞子线程，非 JS worker），故 JS worker
/// 不在 fetch 期间冻结。Response 对象 spec-compliance（ok/status/statusText/headers/text/json）由 shim
/// `_makeResponseFromWire` 在 JS 侧包装。
pub fn default_fetch_handler() -> FetchHandler {
    Arc::new(|req: &FetchRequest| {
        let method = match req.method.to_ascii_uppercase().as_str() {
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => HttpMethod::Get,
        };
        let http_req = HttpRequest {
            method,
            url: req.url.clone(),
            headers: req.headers.clone(),
            // R3020：二进制 body 优先（Blob/FormData multipart 字节保真）；否则文本 body → 字节。
            body: req
                .body_bytes
                .clone()
                .or_else(|| req.body.as_ref().map(|b| b.as_bytes().to_vec())),
        };
        // per-request client：reqwest client 构造（含 TLS）有成本但可接受；net pool 连接复用属后续优化。
        let resp = HttpClient::new()
            .send(http_req)
            .map_err(|e| format!("fetch send: {e}"))?;
        Ok(FetchResponse {
            status: resp.status_code,
            status_text: status_reason(resp.status_code).to_string(),
            headers: resp.headers,
            body: String::from_utf8_lossy(&resp.body).to_string(),
            // R3021：携原始字节——bridge 对非 UTF-8 body 经 byte-wire 传 JS（response.blob()/arrayBuffer() 保真）。
            body_bytes: Some(resp.body),
        })
    })
}

/// 常见 HTTP 状态码 → 标准原因短语（供 `response.statusText`）；未知 → "OK"。
fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
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

    /// P1b S3 incr-d：非阻塞 fetch 的 resolve 时机异步——轮询 `globalThis.{key}` 直到
    /// 非 undefined（或超时返当前值）。子线程抓取（synthetic ~ms / 本地 server ~ms）→
    /// generous 超时下可靠（非 flaky）。
    fn wait_for_global(worker: &TabJsWorkerHandle, key: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe) {
                if v != "undefined" {
                    return v;
                }
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

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

    #[test]
    fn tab_js_worker_async_resolver_usable_from_other_thread() {
        // S3 prep：resolver 可移到子线程 resolve（仿真实 fetch host / 定时器跨线程完成）。
        // 证 AsyncResolver: Send（可 move 到子线程）+ Arc<Mutex> clone 跨线程工作。
        let mut worker = TabJsWorkerHandle::spawn(TabId(3));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "new Promise(function(r){ globalThis.__zw_pending['t1'] = r; })
                 .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        let resolver = worker.async_resolver();
        let handle = std::thread::spawn(move || {
            // 仿 fetch host / 定时器在子线程完成 → resolve。
            resolver.resolve("t1", "from-thread!");
        });
        handle.join().unwrap();
        // join 后 resolve 已入 cmd channel（FIFO 先于下一条 Execute）。
        let r = worker.execute_script_direct("globalThis.__result").unwrap();
        assert_eq!(r, "from-thread!");
        worker.shutdown();
    }

    #[test]
    fn async_resolver_traits_send_sync_clone() {
        // S3 prep：register_callback 闭包要求 Send + Sync（mpsc::Sender 非 Sync，
        // Arc<Mutex<>> 修复）。编译期 trait 断言——非运行时行为。
        fn assert_bounds<T: Send + Sync + Clone>() {}
        assert_bounds::<AsyncResolver>();
    }

    #[test]
    fn tab_js_worker_fetch_resolves_via_handler() {
        // P1b S3 incr-a/c/d：fetch 经 __zw_fetch 回调 + handler 抓取 + resolver.resolve
        // 端到端。合成 handler（无网络）返 body:<url>。incr-c 起 resolve Response 对象（r.text()
        // 取 body）；incr-d 起非阻塞（子线程抓取 → 异步 resolve → 测试 polling）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(4));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|req: &FetchRequest| {
            Ok(FetchResponse::ok(format!("body:{}", req.url)))
        }));
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
    fn tab_js_worker_fetch_without_handler_resolves_error() {
        // 未注入 handler 时 __zw_fetch resolve 错误标记 → Response.ok=false（不悬挂）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(5));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        // 不调 set_fetch_handler。
        worker
            .execute_script_direct("fetch('/x').then(function(r){ globalThis.__result = r.ok ? 'OK' : 'ERR'; });")
            .unwrap();
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "ERR");
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_fetch_response_object_shape_and_json() {
        // P1b S3 incr-c：Response 对象 spec-compliance（ok/status/text()/json()）。
        // 合成 handler 返 JSON body → r.ok/status 正确，r.json() 解析出对象。
        let mut worker = TabJsWorkerHandle::spawn(TabId(7));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|_req: &FetchRequest| {
            Ok(FetchResponse::ok("{\"key\":\"value\",\"n\":42}".to_string()))
        }));
        worker
            .execute_script_direct(
                "fetch('/j').then(function(r){
                   globalThis.__shape = r.ok + ':' + r.status;
                   return r.json();
                 }).then(function(o){ globalThis.__result = o.key + '/' + o.n; });",
            )
            .unwrap();
        let shape = wait_for_global(&worker, "__shape", 1000);
        assert_eq!(shape, "true:200");
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "value/42");
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_default_fetch_handler_real_http() {
        // P1b S3 incr-b/d：生产 default_fetch_handler 经 net pool 真实 HTTP GET。
        // 本地 HTTP server（127.0.0.1）服务固定 body——不依赖外部网络。incr-d 非阻塞：
        // 子线程 recv（不冻结 JS worker），测试 polling（3s 超时，本地 fetch ~ms）。
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local server");
        let port = listener.local_addr().expect("local addr").port();
        let server = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf); // 丢弃请求行
                let body = "hello-from-server";
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        let mut worker = TabJsWorkerHandle::spawn(TabId(6));
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
        assert_eq!(r, "hello-from-server");
        worker.shutdown();
        let _ = server.join();
    }

    #[test]
    fn tab_js_worker_fetch_post_method_headers_body_r2923() {
        // R2923 fetch 完整化：fetch(input, init) 透传 method/headers/body → handler 收 FetchRequest，
        // 返 FetchResponse(status/statusText/headers/body) → shim 解析为 Response。合成 handler 把请求
        // 契约 echo 进响应 body（避免 handler 子线程 assert 丢失），JS 侧断言完整往返契约。
        let mut worker = TabJsWorkerHandle::spawn(TabId(9));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|req: &FetchRequest| {
            let has_ct = req
                .headers
                .iter()
                .any(|(k, v)| k.eq_ignore_ascii_case("content-type") && v == "application/json");
            Ok(FetchResponse {
                status: 201,
                status_text: "Created".to_string(),
                headers: vec![("X-Test".to_string(), "r2923".to_string())],
                body: format!(
                    "{}|{}|{}|{}",
                    req.method,
                    req.url,
                    req.body.as_deref().unwrap_or(""),
                    has_ct
                ),
                body_bytes: None,
            })
        }));
        worker
            .execute_script_direct(
                "fetch('/api/echo', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{\"x\":1}' })\
                 .then(function(r){\
                   globalThis.__status = r.status + ':' + r.statusText + ':' + (r.ok ? 'ok' : 'no');\
                   globalThis.__hdr = r.headers.get('X-Test');\
                   return r.text();\
                 })\
                 .then(function(t){ globalThis.__result = t; });",
            )
            .unwrap();
        let status = wait_for_global(&worker, "__status", 3000);
        assert_eq!(status, "201:Created:ok", "Response.status/statusText/ok 从 wire 解析");
        let hdr = wait_for_global(&worker, "__hdr", 2000);
        assert_eq!(hdr, "r2923", "Response.headers 从 wire 解析（X-Test 头）");
        let r = wait_for_global(&worker, "__result", 2000);
        assert_eq!(
            r, "POST|/api/echo|{\"x\":1}|true",
            "请求 method/url/body/headers 全透传 + Response.text() 返响应 body"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_settimeout_fires_after_real_delay() {
        // P1b S5：setTimeout 真实延迟。host 注册 __zw_setTimeout → 子线程 sleep(delay) 后
        // resolve → __zwResolveCallback 调用回调。注册前（delay 未到）未触发；之后触发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(8));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");

        worker
            .execute_script_direct("setTimeout(function(){ globalThis.__fired = 'yes'; }, 50);")
            .unwrap();
        // delay（50ms）未到：回调尚未触发（host 子线程 sleep 中）。
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        let r = wait_for_global(&worker, "__fired", 1000);
        assert_eq!(r, "yes");
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_cleartimeout_cancels_callback() {
        // P1b S5：clearTimeout 删 pending 项 → 即便 host 子线程后到 resolve，
        // __zwResolveCallback 见无 pending 即 no-op，回调永不触发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(9));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var h = setTimeout(function(){ globalThis.__fired = 'yes'; }, 30);
                 clearTimeout(h);",
            )
            .unwrap();
        // 等待远超 30ms（clearTimeout 应已阻止触发）。
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_setinterval_repeats_then_clear() {
        // P1b S5：setInterval 回调内 re-arm（再次 __zw_setTimeout）实现重复触发；
        // clearInterval 删 pending 断开 re-arm 链。
        let mut worker = TabJsWorkerHandle::spawn(TabId(10));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__n = 0;
                 globalThis.__iv = setInterval(function(){ globalThis.__n++; }, 20);",
            )
            .unwrap();
        // 轮询等待 setInterval 至少触发 2 次（R2149：原固定 sleep 120ms 在 `make test` 全
        // workspace 并行负载下偶发 worker 线程饿死 → n1<2 false-fail；改条件式轮询
        // robust-to-starvation，1000ms 充分覆盖调度延迟，只要 interval 能 fire 即必达 ≥2）。
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
        // clearInterval 后再等，计数应不再增长。
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
    fn tab_js_worker_mutation_observer_childlist() {
        // P1b S2 incr1：MutationObserver（handle-based，JS 创建子树）。observe(createElement'd
        // root, {childList:true}) → appendChild → microtask 派发回调（records[0].type=childList）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(11));
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
    fn tab_js_worker_mutation_observer_attributes() {
        // P1b S2 incr1：attributes 观测——setAttribute → records[0].attributeName。
        let mut worker = TabJsWorkerHandle::spawn(TabId(12));
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
    fn tab_js_worker_mutation_observer_disconnect() {
        // P1b S2 incr1：disconnect 后不再派发——回调永不触发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(13));
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
        // 等过任何 microtask flush 窗口。
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_mutation_observer_existing_dom_attributes() {
        // P1b S2 incr2：观测现有 DOM——querySelector 返 selector-based proxy（handle=null），
        // observe 用 selector 身份注册；setAttribute 经 sel 匹配派发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(14));
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
    fn tab_js_worker_mutation_observer_existing_dom_childlist() {
        // P1b S2 incr2：观测现有 DOM childList——appendChild 到既有容器。
        let mut worker = TabJsWorkerHandle::spawn(TabId(15));
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
    fn tab_js_worker_element_identity_stable_proxy() {
        // P1b S2 incr3：=== node identity——Proxy 缓存使 querySelector 同元素返同一对象；
        // createElement 每次返新节点（不同 handle → 不同 proxy）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(16));
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
    fn tab_js_worker_mutation_observer_property_set() {
        // P1b S2 incr3：property set 路径（el.className='x'，set trap）也触发 MutationObserver
        // attributes 记录（incr1/2 仅 hook setAttribute；incr3 补 set trap）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(17));
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

    #[test]
    fn tab_js_worker_element_from_point_r2924() {
        // R2924 elementFromPoint（镜像 renderer js_worker）：`document.elementFromPoint(x, y)` →
        // 视口 (x,y) 命中的最深元素。注入合成 HitTestCache（root div + 子 p#inner），shim 经
        // `__zw_elementFromPoint` 求命中选择器 → `_wrapSelector` → `.tagName`（`_realTag` 查 dom_html）。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = TabJsWorkerHandle::spawn(TabId(11));
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 合成 HitTestCache：root div(0,0,800,600) + 子 p#inner(10,20,100,50)（坐标相对父内容区）。
        let id0 = node_id_from_u64(0); // Document 节点（非元素）
        let id1 = node_id_from_u64(1); // div
        let id2 = node_id_from_u64(2); // p#inner
        let cache = HitTestCache::from_snapshot(HitTestCacheSnapshot {
            doc_root: id0,
            layout_root: HitTestLayoutSnapshot {
                node_id: Some(id1),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                children: vec![HitTestLayoutSnapshot {
                    node_id: Some(id2),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    children: vec![],
                }],
            },
            nodes: vec![
                (
                    id1,
                    HitTestNodeSnapshot {
                        tag_name: "div".into(),
                        id: None,
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
                (
                    id2,
                    HitTestNodeSnapshot {
                        tag_name: "p".into(),
                        id: Some("inner".into()),
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
            ],
            parents: vec![(id2, id1)],
        });
        *worker.element_from_point_cache().lock().unwrap() = Some(Arc::new(cache));
        worker
            .execute_script_direct(
                "var hit = document.elementFromPoint(50, 40);\
                 globalThis.__t1 = hit ? hit.tagName : 'null';\
                 var root = document.elementFromPoint(5, 5);\
                 globalThis.__t2 = root ? root.tagName : 'null';\
                 var miss = document.elementFromPoint(900, 900);\
                 globalThis.__t3 = miss ? miss.tagName : 'null';",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t1)").unwrap(),
            "P",
            "(50,40) 命中最深子元素 p#inner"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t2)").unwrap(),
            "DIV",
            "(5,5) 仅落在 root div 内（子外）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t3)").unwrap(),
            "null",
            "(900,900) 落在所有元素外 → null（spec）"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_elements_from_point_r2925() {
        // R2925 elementsFromPoint（镜像 renderer）：`document.elementsFromPoint(x, y)` → 视口 (x,y)
        // 处全部元素（绘制序，最前在前）。注入合成 HitTestCache（root div + 子 p#inner），断言命中栈。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = TabJsWorkerHandle::spawn(TabId(12));
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let id0 = node_id_from_u64(0);
        let id1 = node_id_from_u64(1);
        let id2 = node_id_from_u64(2);
        let cache = HitTestCache::from_snapshot(HitTestCacheSnapshot {
            doc_root: id0,
            layout_root: HitTestLayoutSnapshot {
                node_id: Some(id1),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                children: vec![HitTestLayoutSnapshot {
                    node_id: Some(id2),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    children: vec![],
                }],
            },
            nodes: vec![
                (
                    id1,
                    HitTestNodeSnapshot {
                        tag_name: "div".into(),
                        id: None,
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
                (
                    id2,
                    HitTestNodeSnapshot {
                        tag_name: "p".into(),
                        id: Some("inner".into()),
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
            ],
            parents: vec![(id2, id1)],
        });
        *worker.element_from_point_cache().lock().unwrap() = Some(Arc::new(cache));
        worker
            .execute_script_direct(
                "var list = document.elementsFromPoint(50, 40);\
                 globalThis.__n1 = list.length;\
                 globalThis.__f1 = list.length ? list[0].tagName : 'null';\
                 globalThis.__s1 = list.length > 1 ? list[1].tagName : 'null';\
                 globalThis.__n2 = document.elementsFromPoint(5, 5).length;\
                 globalThis.__n3 = document.elementsFromPoint(900, 900).length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n1)").unwrap(),
            "2",
            "(50,40) 命中栈 = 2 元素（p#inner + root div）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__f1)").unwrap(),
            "P",
            "首元素 = p#inner（最前/最深）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__s1)").unwrap(),
            "DIV",
            "次元素 = root div（最后/最浅）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n2)").unwrap(),
            "1",
            "(5,5) 仅 root div → 1 元素"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n3)").unwrap(),
            "0",
            "(900,900) 落空 → 空数组"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_attach_shadow_r2926() {
        // R2926 attachShadow / shadowRoot（Tier 2 Web Components 地基，镜像 renderer）：
        // `element.attachShadow({mode})` 返 ShadowRoot（nodeType 11 / nodeName '#shadow-root' /
        // mode / host）；shadowRoot getter（open 返 root / closed 返 null）；已挂载→抛 NotSupportedError；
        // 非法 mode→抛 TypeError。shadow root 复用 DocumentFragment handle（不渲染，fidelity defer）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(13));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 globalThis.__nt = sr.nodeType;\
                 globalThis.__nn = sr.nodeName;\
                 globalThis.__mode = sr.mode;\
                 globalThis.__host = (sr.host === host);\
                 globalThis.__sr = (host.shadowRoot === sr);\
                 var host2 = document.createElement('div');\
                 host2.attachShadow({ mode: 'closed' });\
                 globalThis.__closed = (host2.shadowRoot === null);\
                 var host3 = document.createElement('div');\
                 host3.attachShadow({ mode: 'open' });\
                 var threw = false;\
                 try { host3.attachShadow({ mode: 'open' }); } catch (e) { threw = true; }\
                 globalThis.__threw = threw;\
                 var host4 = document.createElement('div');\
                 var threwMode = false;\
                 try { host4.attachShadow({ mode: 'bad' }); } catch (e) { threwMode = true; }\
                 globalThis.__threwMode = threwMode;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__nt)").unwrap(),
            "11",
            "ShadowRoot nodeType = 11"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__nn)").unwrap(),
            "#shadow-root",
            "ShadowRoot nodeName = '#shadow-root'"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__mode)").unwrap(),
            "open",
            "ShadowRoot.mode 反映 init.mode"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__host)").unwrap(),
            "true",
            "shadowRoot.host === 宿主元素（同一 proxy）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sr)").unwrap(),
            "true",
            "element.shadowRoot（open）=== attachShadow 返回的 root"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__closed)").unwrap(),
            "true",
            "closed mode → element.shadowRoot === null（spec）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__threw)").unwrap(),
            "true",
            "已挂 shadow 的 host 再次 attachShadow → 抛异常（spec NotSupportedError）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__threwMode)").unwrap(),
            "true",
            "非法 mode → 抛异常（spec TypeError）"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_handle_children_registry_r2927() {
        // R2927 handle-children registry（镜像 renderer）：容器 handle（shadow root / fragment）经
        // appendChild 记录子节点 → childNodes/firstChild/firstElementChild/childElementCount 可观察
        //（旧实现 handle-only 恒返 []）。removeChild 同步更新；fragment flatten 进非容器父后清空（spec）。
        let mut worker = TabJsWorkerHandle::spawn(TabId(14));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 var span1 = document.createElement('span');\
                 sr.appendChild(span1);\
                 globalThis.__cn1 = sr.childNodes.length;\
                 globalThis.__ff1 = (sr.firstChild === span1);\
                 globalThis.__fe1 = (sr.firstElementChild === span1);\
                 globalThis.__ec1 = sr.childElementCount;\
                 var tn = document.createTextNode('hi');\
                 sr.appendChild(tn);\
                 globalThis.__cn2 = sr.childNodes.length;\
                 globalThis.__ec2 = sr.childElementCount;\
                 sr.removeChild(span1);\
                 globalThis.__cn3 = sr.childNodes.length;\
                 var frag = document.createDocumentFragment();\
                 frag.appendChild(document.createElement('b'));\
                 globalThis.__fc = frag.childNodes.length;\
                 document.body.appendChild(frag);\
                 globalThis.__fc2 = frag.childNodes.length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn1)").unwrap(),
            "1",
            "appendChild 1 子 → childNodes 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ff1)").unwrap(),
            "true",
            "firstChild === span1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fe1)").unwrap(),
            "true",
            "firstElementChild === span1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ec1)").unwrap(),
            "1",
            "childElementCount 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn2)").unwrap(),
            "2",
            "append textNode → childNodes 2"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ec2)").unwrap(),
            "1",
            "childElementCount 仍 1（text 过滤）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn3)").unwrap(),
            "1",
            "removeChild span1 → childNodes 1（剩 textNode）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fc)").unwrap(),
            "1",
            "fragment appendChild → fragment.childNodes 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fc2)").unwrap(),
            "0",
            "fragment flatten 进 body 后清空 → childNodes 0（spec）"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_handle_subtree_query_selector_r2928() {
        // R2928 handle 子树 querySelector/querySelectorAll（镜像 renderer js_worker）：JS 端 registry 树搜索
        // + 客户端 compound / 后代 / 逗号列表匹配。shadow root + created element 句柄子树查询。querySelector
        // 不穿透 shadow 边界。
        let mut worker = TabJsWorkerHandle::spawn(TabId(15));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 var wrap = document.createElement('div'); wrap.id = 'wrap'; wrap.className = 'outer';\
                 var btn = document.createElement('button'); btn.id = 'go';\
                 btn.className = 'btn primary'; btn.setAttribute('type', 'submit');\
                 var span = document.createElement('span'); span.className = 'label';\
                 wrap.appendChild(btn); wrap.appendChild(span); sr.appendChild(wrap);\
                 globalThis.__byTag = (sr.querySelector('button') === btn);\
                 globalThis.__byId = (sr.querySelector('#go') === btn);\
                 globalThis.__byClass = (sr.querySelector('.btn') === btn);\
                 globalThis.__compound = (sr.querySelector('button.btn') === btn);\
                 globalThis.__desc = (sr.querySelector('div button') === btn);\
                 globalThis.__desc2 = (sr.querySelector('div span') === span);\
                 globalThis.__attr = (sr.querySelector('[type=submit]') === btn);\
                 globalThis.__comma = (sr.querySelector('button, span') === btn);\
                 globalThis.__allBtnSpan = sr.querySelectorAll('button, span').length;\
                 globalThis.__allClass = sr.querySelectorAll('.btn, .label').length;\
                 globalThis.__wildcard = (sr.querySelector('*') === wrap);\
                 globalThis.__nomatch = (sr.querySelector('input') === null);\
                 globalThis.__nomatchAll = sr.querySelectorAll('input').length;\
                 globalThis.__boundary = (host.querySelector('button') === null);\
                 var sec = document.createElement('section');\
                 var p = document.createElement('p'); p.id = 'p1';\
                 sec.appendChild(p);\
                 globalThis.__elQs = (sec.querySelector('#p1') === p);\
                 globalThis.__elQsTag = (sec.querySelector('p') === p);",
            )
            .unwrap();
        let cases = [
            ("__byTag", "true", "shadow querySelector('button') === btn（tag）"),
            ("__byId", "true", "shadow querySelector('#go') === btn（id）"),
            ("__byClass", "true", "shadow querySelector('.btn') === btn（class）"),
            ("__compound", "true", "shadow querySelector('button.btn')（复合）"),
            ("__desc", "true", "shadow querySelector('div button')（后代）"),
            ("__desc2", "true", "shadow querySelector('div span')（后代，另支）"),
            ("__attr", "true", "shadow querySelector('[type=submit]')（属性 =）"),
            ("__comma", "true", "shadow querySelector('button, span')（逗号列表）"),
            (
                "__allBtnSpan",
                "2",
                "shadow querySelectorAll('button, span').length === 2",
            ),
            (
                "__allClass",
                "2",
                "shadow querySelectorAll('.btn, .label').length === 2",
            ),
            ("__wildcard", "true", "shadow querySelector('*') === wrap（通配）"),
            ("__nomatch", "true", "shadow querySelector('input') === null（无匹配）"),
            ("__nomatchAll", "0", "shadow querySelectorAll('input').length === 0"),
            (
                "__boundary",
                "true",
                "host.querySelector('button') === null（不穿透 shadow）",
            ),
            (
                "__elQs",
                "true",
                "created element querySelector('#p1') === p（非容器 handle）",
            ),
            ("__elQsTag", "true", "created element querySelector('p') === p（tag）"),
        ];
        for (key, expect, msg) in cases {
            assert_eq!(
                worker
                    .execute_script_direct(&format!("String(globalThis.{key})"))
                    .unwrap(),
                expect,
                "{msg}"
            );
        }
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_range_mutation_ops_r2929() {
        // R2929 Range 变更操作（镜像 renderer js_worker）：deleteContents/extractContents/insertNode/真实
        // cloneContents。经既有 mutation-emitting proxy 真实变更。验证 fragment 内容（同步）+ apply 后结构。
        use zero_engine::apply_mutations_to_html;
        let mut worker = TabJsWorkerHandle::spawn(TabId(16));
        let html = "<html><body>\
                    <div id='cc'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='ec'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='dc'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='ic'><p>0</p></div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var r1 = document.createRange(); r1.selectNodeContents(document.getElementById('cc'));\
                 var cc = r1.cloneContents();\
                 globalThis.__ccN = cc.childNodes.length;\
                 globalThis.__ccT = cc.childNodes[0].tagName;\
                 var r2 = document.createRange(); r2.selectNodeContents(document.getElementById('ec'));\
                 var ec = r2.extractContents();\
                 globalThis.__ecN = ec.childNodes.length;\
                 globalThis.__ecT = ec.childNodes[1].tagName;\
                 var r3 = document.createRange(); r3.selectNodeContents(document.getElementById('dc'));\
                 r3.deleteContents();\
                 var r4 = document.createRange(); r4.selectNodeContents(document.getElementById('ic'));\
                 r4.collapse(true);\
                 globalThis.__insRet = (r4.insertNode(document.createElement('b')) !== undefined);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccN)").unwrap(),
            "3",
            "cloneContents fragment 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccT)").unwrap(),
            "SPAN",
            "cloneContents [0].tagName SPAN（真实克隆）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecN)").unwrap(),
            "3",
            "extractContents fragment 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecT)").unwrap(),
            "SPAN",
            "extractContents [1].tagName SPAN"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__insRet)").unwrap(),
            "true",
            "insertNode 返回插入的节点"
        );
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply range mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__dcN = document.getElementById('dc').children.length;\
                 globalThis.__ecSrc = document.getElementById('ec').children.length;\
                 globalThis.__ccSrc = document.getElementById('cc').children.length;\
                 globalThis.__icN = document.getElementById('ic').children.length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dcN)").unwrap(),
            "0",
            "deleteContents 后 #dc 0 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecSrc)").unwrap(),
            "0",
            "extractContents 后 #ec 0 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccSrc)").unwrap(),
            "3",
            "cloneContents 不改源 → #cc 仍 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__icN)").unwrap(),
            "2",
            "insertNode 后 #ic 2 子"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_range_surround_contents_r2930() {
        // R2930 Range surroundContents（镜像 renderer）：selectNodeContents 包整元素内容 → wrap。验证 apply 后结构。
        use zero_engine::apply_mutations_to_html;
        let mut worker = TabJsWorkerHandle::spawn(TabId(17));
        let html = "<html><body><div id='sc'><span>A</span><span>B</span></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var r = document.createRange();\
                 r.selectNodeContents(document.getElementById('sc'));\
                 var wrap = document.createElement('div'); wrap.id = 'w';\
                 globalThis.__ret = r.surroundContents(wrap);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ret)").unwrap(),
            "undefined",
            "surroundContents 返回 undefined"
        );
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply surround mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__scN = document.getElementById('sc').children.length;\
                 globalThis.__scChildId = document.getElementById('sc').children[0].id;\
                 globalThis.__wN = document.getElementById('w').children.length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scN)").unwrap(),
            "1",
            "surroundContents 后 #sc 仅 1 子（wrap）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scChildId)").unwrap(),
            "w",
            "#sc 子 id=w"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wN)").unwrap(),
            "2",
            "wrap 含 2 个克隆子"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_page_lifecycle_r2931() {
        // R2931 页面生命周期/分析簇（镜像 renderer）：sendBeacon + PageTransitionEvent + pageshow 派发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(18));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__beacon1 = navigator.sendBeacon('/analytics', { x: 1 });\
                 globalThis.__beacon2 = navigator.sendBeacon();\
                 globalThis.__pt1 = new PageTransitionEvent('pageshow', { persisted: true }).persisted;\
                 globalThis.__pt2 = new PageTransitionEvent('pageshow').persisted;\
                 globalThis.__ps = 'no';\
                 window.addEventListener('pageshow', function (e) {\
                   globalThis.__ps = e.type + ':' + String(e.persisted);\
                 });",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon1)").unwrap(),
            "true",
            "sendBeacon(url, data) → true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon2)").unwrap(),
            "false",
            "sendBeacon() 缺 url → false"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt1)").unwrap(),
            "true",
            "PageTransitionEvent persisted:true → true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt2)").unwrap(),
            "false",
            "PageTransitionEvent 默认 persisted false"
        );
        // pageshow 经首次注册 _defer 派发（execute 末 drain）→ 轮询读。
        let mut ps = String::new();
        for _ in 0..400 {
            ps = worker
                .execute_script_direct("String(globalThis.__ps)")
                .unwrap_or_default();
            if ps == "pageshow:false" {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            ps, "pageshow:false",
            "window pageshow listener 触发（type + persisted:false）"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_window_on_handlers_r2932() {
        // R2932 window IDL on-event handler（镜像 renderer）：on* setter 注册 listener + 移除语义 + dispatch 触发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(19));
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__c1 = 0; globalThis.__c2 = 0; globalThis.__id = 'no';\
                 globalThis.__null = 'no'; globalThis.__ps = 'no';\
                 function h1() { globalThis.__c1++; }\
                 function h2() { globalThis.__c2++; }\
                 window.onload = h1;\
                 globalThis.__id = (window.onload === h1);\
                 window.dispatchEvent(new Event('load'));\
                 window.onload = h2;\
                 window.dispatchEvent(new Event('load'));\
                 window.onload = null;\
                 globalThis.__null = (window.onload === null);\
                 window.dispatchEvent(new Event('load'));\
                 window.onpageshow = function (e) {\
                   globalThis.__ps = e.type + ':' + String(e.persisted);\
                 };",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__id)").unwrap(),
            "true",
            "window.onload = h1 → getter 返同一 fn"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c1)").unwrap(),
            "1",
            "dispatch load → h1 触发一次"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c2)").unwrap(),
            "1",
            "重赋 onload=h2 → h2 触发一次（h1 已移除）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__null)").unwrap(),
            "true",
            "window.onload = null → 移除"
        );
        // onpageshow setter 触发 R2931 派发（轮询读）。
        let mut ps = String::new();
        for _ in 0..400 {
            ps = worker
                .execute_script_direct("String(globalThis.__ps)")
                .unwrap_or_default();
            if ps == "pageshow:false" {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(ps, "pageshow:false", "onpageshow setter 触发 pageshow 派发");
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_element_on_handlers_r2933() {
        // R2933 element 级 IDL on-event handler（镜像 renderer）：onclick/oninput setter 路由 + getter + 移除 + dispatch 触发。
        let mut worker = TabJsWorkerHandle::spawn(TabId(20));
        worker.set_dom_snapshot("<html><body><div id='d'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var d = document.getElementById('d');\
                 globalThis.__dc = 0; globalThis.__did = 'no'; globalThis.__dnull = 'no';\
                 function dh(e) { if (e && e.type === 'click') globalThis.__dc++; }\
                 d.onclick = dh;\
                 globalThis.__did = (d.onclick === dh);\
                 d.dispatchEvent(new Event('click'));\
                 d.onclick = null;\
                 globalThis.__dnull = (d.onclick === null);\
                 d.dispatchEvent(new Event('click'));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__did)").unwrap(),
            "true",
            "parsed 元素 d.onclick = dh → getter 返同一 fn"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dc)").unwrap(),
            "1",
            "dispatchEvent click → onclick 触发一次（=null 后不再触发）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dnull)").unwrap(),
            "true",
            "d.onclick = null → getter 返 null"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_inline_html_handlers_r2934() {
        // R2934 inline HTML event handler（镜像 renderer）：on* 属性编译 + dispatch/click 触发 + scope + JS 覆盖。
        let mut worker = TabJsWorkerHandle::spawn(TabId(21));
        let html = "<html><body>\
                    <button id='b' onclick=\"globalThis.__inline='yes'\"></button>\
                    <span id='s' onclick=\"globalThis.__tag=this.tagName\"></span>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var b = document.getElementById('b');\
                 globalThis.__typeof = typeof b.onclick;\
                 globalThis.__inline = 'no';\
                 b.dispatchEvent(new Event('click'));\
                 var s = document.getElementById('s');\
                 globalThis.__tag = 'no';\
                 s.dispatchEvent(new Event('click'));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__typeof)").unwrap(),
            "function",
            "inline onclick → getter 返编译的 function"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__inline)").unwrap(),
            "yes",
            "dispatchEvent click → inline handler 触发"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__tag)").unwrap(),
            "SPAN",
            "inline handler with(this) scope → this.tagName === 'SPAN'"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_inline_handler_ancestor_bubble_r2935() {
        // R2935 祖先 inline handler 冒泡触发（镜像 renderer）：capture/bubble 祖先阶段 ensure inline handler。
        let mut worker = TabJsWorkerHandle::spawn(TabId(22));
        let html = "<html><body>\
                    <div id='outer' onclick=\"globalThis.__outer=this.id\">\
                      <div id='inner' onclick=\"globalThis.__inner=this.id\">\
                        <button id='btn'>x</button>\
                      </div>\
                    </div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__outer = 'no'; globalThis.__inner = 'no';\
                 document.getElementById('btn').dispatchEvent(new Event('click', { bubbles: true }));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__inner)").unwrap(),
            "inner",
            "祖先 inner inline handler 冒泡触发（this=inner）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__outer)").unwrap(),
            "outer",
            "祖父 outer inline handler 冒泡触发（this=outer）"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_clipboard_events_r2936() {
        // R2936 剪贴板事件（镜像 renderer）：ClipboardEvent + execCommand('copy') 派发 + listener/oncopy/冒泡。
        let mut worker = TabJsWorkerHandle::spawn(TabId(23));
        let html = "<html><body><input id='inp'></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__cd = new ClipboardEvent('copy', { clipboardData: 'dt' }).clipboardData;\
                 var inp = document.getElementById('inp');\
                 inp.focus();\
                 globalThis.__copy = 'no';\
                 inp.addEventListener('copy', function (e) {\
                   globalThis.__copy = e.type + ':' + (e.constructor === globalThis.ClipboardEvent);\
                 });\
                 globalThis.__oncopy = 'no';\
                 inp.oncopy = function (e) { globalThis.__oncopy = e.type; };\
                 document.execCommand('copy');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cd)").unwrap(),
            "dt",
            "ClipboardEvent clipboardData 字段"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__copy)").unwrap(),
            "copy:true",
            "execCommand('copy') → copy listener 触发（type + ClipboardEvent）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__oncopy)").unwrap(),
            "copy",
            "execCommand('copy') → oncopy handler 触发"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_drag_and_drop_r2937() {
        // R2937 Drag & Drop API（镜像 renderer）：DataTransfer + DragEvent + dispatch + ondrop。
        let mut worker = TabJsWorkerHandle::spawn(TabId(24));
        let html = "<html><body><div id='dz'></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var dt = new DataTransfer();\
                 dt.setData('text/plain', 'hello');\
                 globalThis.__dt1 = dt.getData('text/plain');\
                 globalThis.__types = dt.types.join(',');\
                 var dz = document.getElementById('dz');\
                 globalThis.__drop = 'no';\
                 dz.addEventListener('drop', function (e) {\
                   globalThis.__drop = e.type + ':' + e.dataTransfer.getData('text/plain') + ':'\
                     + (e.constructor === globalThis.DragEvent);\
                 });\
                 globalThis.__ondrop = 'no';\
                 dz.ondrop = function (e) { globalThis.__ondrop = e.type; };\
                 var dt2 = new DataTransfer();\
                 dt2.setData('text/plain', 'payload');\
                 dz.dispatchEvent(new DragEvent('drop', { dataTransfer: dt2, bubbles: true }));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt1)").unwrap(),
            "hello",
            "DataTransfer.setData/getData"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__types)").unwrap(),
            "text/plain",
            "DataTransfer.types"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__drop)").unwrap(),
            "drop:payload:true",
            "dispatchEvent DragEvent('drop') → drop listener 触发（dataTransfer + 构造器）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ondrop)").unwrap(),
            "drop",
            "ondrop handler 触发"
        );
        worker.shutdown();
    }

    #[test]
    fn tab_js_worker_get_bounding_client_rect_real_rect() {
        // P1a gBCR path C（镜像 renderer js_worker）：selector-identity 元素的 getBoundingClientRect
        // 返真实 DOMRect。shim `__zw_getBoundingClientRect(sel)` → handler fresh-parse dom_html
        // → find_by_selector → NodeId → 查 rect_snapshot。用「同一 html fresh-parse」的 NodeId 填
        // snapshot（= tab_worker render 后会用的 NodeId；确定性由 engine 守护测试保证）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = TabJsWorkerHandle::spawn(TabId(18));
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
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
    fn tab_js_worker_get_bounding_client_rect_empty_snapshot_zero() {
        // 零回归：snapshot 未填（无 render / 未命中）→ handler 返 None → shim 回落零 rect。
        let mut worker = TabJsWorkerHandle::spawn(TabId(19));
        worker.set_dom_snapshot("<html><body><div id='t'>hi</div></body></html>", "about:blank");
        worker
            .execute_script_direct("globalThis.__w = document.querySelector('#t').getBoundingClientRect().width;")
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__w)").unwrap(), "0");
        worker.shutdown();
    }
}
