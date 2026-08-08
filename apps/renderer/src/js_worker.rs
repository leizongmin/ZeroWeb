//! 渲染进程 JS 线程 — V8 与页面渲染分离。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

const TAB_JS_EXEC_TIMEOUT_MS: u64 = 15_000;
const TAB_JS_CHANNEL_TIMEOUT: Duration = Duration::from_millis(TAB_JS_EXEC_TIMEOUT_MS + 5_000);

/// P1a gBCR kill-switch：默认 on；`ZW_REAL_RECT=0` 关闭 RectBridge（`__zw_getBoundingClientRect`
/// 不注册 → shim 回落零 rect = 当前行为，零回归）。snapshot 为空 / identity 未命中同样回落零 rect。
/// P1a Slice 2b：亦用作 observer host-tick 的 kill-switch（gBCR 关 → rect 恒零 → tick 无意义）。
pub(crate) fn real_rect_enabled() -> bool {
    !matches!(std::env::var("ZW_REAL_RECT").as_deref(), Ok("0"))
}

/// P1a 事件循环 slice 1（R2713b）：帧驱动 rAF kill-switch。默认 off——shim rAF 走同步 stub
///（reftest 兼容，零默认行为变更）；`ZW_RAF_FRAME_DRIVEN=1` 开启：shim rAF 注册队列，render 后
/// `tick_observers` 调 `__zw_raf_tick` 派发。详见 p1a-event-loop-raf-slice-design-2026-08-05.md。
pub(crate) fn raf_frame_driven_enabled() -> bool {
    matches!(std::env::var("ZW_RAF_FRAME_DRIVEN").as_deref(), Ok("1"))
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
    /// P1a gBCR path A：持久 handle→唯一选择器映射——生产 apply 路径（`apply_recorded_mutations`）
    /// merge 进此 map，js_worker 的 RectBridge handler 读它解析 handle-identity（`__n{n}`）。
    handle_selector_map: HandleSelectorMap,
    /// P1a elementFromPoint：共享 hit-test 缓存槽——renderer 主循环 render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    element_from_point_cache: ElementFromPointCache,
    /// R2949 FontFace.load() 请求队列——`__zw_load_font` 回调（worker 线程）push，renderer 主循环
    /// drain 后 fetch_get 字节 + load_font/register/set_resolver + async_resolver.resolve 解析 Promise。
    font_loads: Arc<std::sync::Mutex<Vec<zero_engine::FontLoadRequest>>>,
}

impl RendererJsWorker {
    /// 启动 JS 专用线程。
    pub fn spawn(renderer_id: u64) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let rect_snapshot = new_layout_rect_snapshot();
        let handle_selector_map = new_handle_selector_map();
        let element_from_point_cache = new_element_from_point_cache();
        // R2949 FontFace.load() 桥——queue 与 runtime 共享，bridge 移入 worker 线程注册 __zw_load_font。
        let font_bridge = zero_engine::FontLoadBridge::new();
        let font_loads = font_bridge.queue();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let cmd_for_worker = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);
        let rect_snapshot_for_worker = Arc::clone(&rect_snapshot);
        let handle_selector_map_for_worker = Arc::clone(&handle_selector_map);
        let element_from_point_cache_for_worker = Arc::clone(&element_from_point_cache);

        let join = thread::Builder::new()
            .name(format!("renderer-js-{}", renderer_id))
            .spawn(move || {
                js_worker_main(
                    cmd_rx,
                    cmd_for_worker,
                    mutations_for_worker,
                    rect_snapshot_for_worker,
                    handle_selector_map_for_worker,
                    element_from_point_cache_for_worker,
                    font_bridge,
                )
            })
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
            handle_selector_map,
            element_from_point_cache,
            font_loads,
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

    /// R2949 FontFace.load() 请求队列句柄——`__zw_load_font` 回调（worker 线程）push，renderer 主循环
    /// drain 后处理（fetch_get 字节 + load_font/register/set_resolver + async_resolver.resolve）。
    pub fn pending_font_loads(&self) -> Arc<std::sync::Mutex<Vec<zero_engine::FontLoadRequest>>> {
        Arc::clone(&self.font_loads)
    }

    /// P1a gBCR：共享 layout-rect snapshot 句柄——renderer 主循环 render 后经
    /// `fill_layout_rect_snapshot` 填充，js_worker 的 RectBridge handler 读取。
    pub fn rect_snapshot(&self) -> LayoutRectSnapshot {
        Arc::clone(&self.rect_snapshot)
    }

    /// P1a gBCR path A：持久 handle→唯一选择器映射句柄——生产 apply 路径
    /// （`page_scripts::apply_recorded_mutations`）merge 进此 map，js_worker 的 RectBridge
    /// handler 读它解析 handle-identity（`__n{n}`，createElement 元素）。
    pub fn handle_selector_map(&self) -> HandleSelectorMap {
        Arc::clone(&self.handle_selector_map)
    }

    /// P1a elementFromPoint：共享 hit-test 缓存槽句柄——renderer 主循环 render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    pub fn element_from_point_cache(&self) -> ElementFromPointCache {
        Arc::clone(&self.element_from_point_cache)
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
    handle_selector_map: HandleSelectorMap,
    element_from_point_cache: ElementFromPointCache,
    font_bridge: zero_engine::FontLoadBridge,
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
            Arc::clone(&handle_selector_map),
        ));
    }
    // P1a elementFromPoint：`ElementFromPointBridge` 注 `__zw_elementFromPoint(x, y)` 同步回调。
    // 回调锁内 clone `Arc<HitTestCache>`（renderer render 后 swap 进共享槽）→ `hit_test_element`
    // + `selector_from_element_hit` → 稳定选择器。未注入 cache / 无命中 → 空串（shim 返 null）。
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
    // R2949 FontFace.load() 桥——__zw_load_font 回调 push 请求到共享队列（runtime drain 后 fetch+register+resolve）。
    font_bridge.register(&mut *sandbox);
    // P1b S5：TimerBridge 注 __zw_setTimeout——shim setTimeout/setInterval 真实延迟
    // （子线程 sleep + resolver.resolve → __zwResolveCallback 调用 JS 回调）。
    let timer_bridge = TimerBridge::new(resolver);
    timer_bridge.register(&mut *sandbox);
    // R2713b：帧驱动 rAF kill-switch——execute shim 前注入 globalThis.__ZW_RAF_FRAME_DRIVEN
    //（shim 据此分支 rAF 同步 stub / 帧驱动）。默认 OFF 不注入 → shim `|| false` → 同步 stub。
    if raf_frame_driven_enabled() {
        let _ = sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;");
    }
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
                    // P1a gBCR path A：导航 → 旧页 handle 在新页无效，清 handle→selector map
                    // （apply 路径会在新页 createElement 时重新 merge）。
                    if let Ok(mut map) = handle_selector_map.lock() {
                        map.clear();
                    }
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

/// R2923 fetch 完整化：生产 fetch handler——经 `zero_net::HttpClient::send` 发起真实 HTTP 请求，
/// 支持全方法（GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS）、请求头、请求体，返 [`FetchResponse`]（status/
/// status_text/headers/body）。renderer 进程直接联网（与 browser `tab_js_worker::default_fetch_handler`
/// 同实现）。GET 行为零回归（method 默认 GET、body=None）。
///
/// `FetchBridge::register` 在**子线程**调本 handler（`send()` 阻塞子线程，非 JS worker），故 JS worker
/// 不在 fetch 期间冻结。Response 对象 spec-compliance 由 shim `_makeResponseFromWire` 在 JS 侧包装。
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
    fn renderer_js_worker_element_from_point_r2924() {
        // R2924 elementFromPoint：`document.elementFromPoint(x, y)` → 视口 (x,y) 命中的最深元素。
        // 注入合成 HitTestCache（root div + 子 p#inner），shim 经 `__zw_elementFromPoint` 求命中选择器
        // → `_wrapSelector` → `.tagName`（`_realTag` 经 `__zw_get_tag` 查 dom_html 真实 tag）。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = RendererJsWorker::spawn(31);
        // dom_html 须含 #inner（`_realTag("#inner")` 经 `__zw_get_tag` 查它返 "p" → tagName "P"）。
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 合成 HitTestCache：root div(0,0,800,600) + 子 p#inner(10,20,100,50)（坐标相对父内容区）。
        let id0 = node_id_from_u64(0); // Document 节点（非元素，落空时 hit_test_element 返 None）
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
    fn renderer_js_worker_elements_from_point_r2925() {
        // R2925 elementsFromPoint（镜像 browser）：`document.elementsFromPoint(x, y)` → 视口 (x,y)
        // 处全部元素（绘制序，最前在前）。注入合成 HitTestCache（root div + 子 p#inner），断言命中栈。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = RendererJsWorker::spawn(32);
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 合成 HitTestCache：root div(0,0,800,600) + 子 p#inner(10,20,100,50)（坐标相对父内容区）。
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
    fn renderer_js_worker_attach_shadow_r2926() {
        // R2926 attachShadow / shadowRoot（Tier 2 Web Components 地基，镜像 browser）：
        // `element.attachShadow({mode})` 返 ShadowRoot（nodeType 11 / nodeName '#shadow-root' /
        // mode / host）；shadowRoot getter（open 返 root / closed 返 null）；已挂载→抛 NotSupportedError；
        // 非法 mode→抛 TypeError。shadow root 复用 DocumentFragment handle（不渲染，fidelity defer）。
        let mut worker = RendererJsWorker::spawn(33);
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
    fn renderer_js_worker_handle_children_registry_r2927() {
        // R2927 handle-children registry：容器 handle（shadow root / fragment）经 appendChild 记录子节点
        // → childNodes/firstChild/firstElementChild/childElementCount 可观察（旧实现 handle-only 恒返 []）。
        // shadow 构建模式（imperative custom element）自测解锁。removeChild 同步更新；fragment flatten
        // 进非容器父后清空（spec）。
        let mut worker = RendererJsWorker::spawn(34);
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
    fn renderer_js_worker_handle_subtree_query_selector_r2928() {
        // R2928 handle 子树 querySelector/querySelectorAll——JS 端 registry 树搜索 + 客户端选择器匹配。
        // handle 元素（shadow root / createElement）无 sel，host `__zw_query_*_sub` 不可用 → registry DFS
        // + compound（tag/id/class/attr）/ 后代组合器 / 逗号列表 匹配。覆盖 Lit `sr.querySelector('#x')`
        // shadow 构建模式自测。querySelector 不穿透 shadow 边界（host.querySelector 查 light-DOM 子树）。
        let mut worker = RendererJsWorker::spawn(35);
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
                 globalThis.__compound2 = (sr.querySelector('button.primary#go') === btn);\
                 globalThis.__desc = (sr.querySelector('div button') === btn);\
                 globalThis.__desc2 = (sr.querySelector('div span') === span);\
                 globalThis.__descClass = (sr.querySelector('div .primary') === btn);\
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
            (
                "__compound2",
                "true",
                "shadow querySelector('button.primary#go')（多 class + id）",
            ),
            ("__desc", "true", "shadow querySelector('div button')（后代）"),
            ("__desc2", "true", "shadow querySelector('div span')（后代，另支）"),
            (
                "__descClass",
                "true",
                "shadow querySelector('div .primary')（后代 + class）",
            ),
            ("__attr", "true", "shadow querySelector('[type=submit]')（属性 =）"),
            (
                "__comma",
                "true",
                "shadow querySelector('button, span')（逗号列表，文档序首匹配）",
            ),
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
            (
                "__wildcard",
                "true",
                "shadow querySelector('*') === wrap（通配，DFS 首元素）",
            ),
            ("__nomatch", "true", "shadow querySelector('input') === null（无匹配）"),
            (
                "__nomatchAll",
                "0",
                "shadow querySelectorAll('input').length === 0（无匹配）",
            ),
            (
                "__boundary",
                "true",
                "host.querySelector('button') === null（不穿透 shadow 边界）",
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
    fn renderer_js_worker_range_mutation_ops_r2929() {
        // R2929 Range 变更操作：deleteContents/extractContents/insertNode/真实 cloneContents（既有 _makeRange
        // 这几项原为 defer / 仅文本）。经既有 mutation-emitting proxy（child.remove() → __zw_remove、
        // insertBefore/appendChild、cloneNode deep）真实变更，emit DomMutation。精确覆盖 start==end 元素容器的
        // offset 区间（selectNode/selectNodeContents 后），sel/handle 子均支持。
        // 验证：① fragment 内容（in-script 同步）；② apply_mutations_to_html → re-snapshot → 结构反映变更。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(36);
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
                 var ins = document.createElement('b');\
                 globalThis.__insRet = (r4.insertNode(ins) === ins);",
            )
            .unwrap();
        // ① fragment 内容（同步）
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccN)").unwrap(),
            "3",
            "cloneContents fragment 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccT)").unwrap(),
            "SPAN",
            "cloneContents [0].tagName SPAN（真实克隆非文本）"
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
            "insertNode 返回插入的节点（spec）"
        );
        // ② apply mutations → re-snapshot → 结构反映变更（用 .children.length 测元素子数；`> *` 选择器
        // 在 host querySelectorAll 路径不稳定，避免）
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply range mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__dcN = document.getElementById('dc').children.length;\
                 globalThis.__ecSrc = document.getElementById('ec').children.length;\
                 globalThis.__ccSrc = document.getElementById('cc').children.length;\
                 globalThis.__icN = document.getElementById('ic').children.length;\
                 globalThis.__icFirst = document.getElementById('ic').children[0].tagName;",
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
            "extractContents 后 #ec 0 子（内容移走）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccSrc)").unwrap(),
            "3",
            "cloneContents 不改源 → #cc 仍 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__icN)").unwrap(),
            "2",
            "insertNode 后 #ic 2 子（b 插在 p 前）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__icFirst)").unwrap(),
            "B",
            "insertNode 插在首位 → #ic 首子为 B"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_range_surround_contents_r2930() {
        // R2930 Range surroundContents：selectNodeContents 包整元素内容（覆盖块延伸到容器末尾）→ clone 内容进
        // newParent + 逆序删原件 + appendChild newParent。headline 用法（rich-text wrap）。验证 apply 后结构：
        // 容器仅含 newParent（1 子），newParent 含克隆内容。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(37);
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
            "surroundContents 返回 undefined（spec）"
        );
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply surround mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__scN = document.getElementById('sc').children.length;\
                 globalThis.__scChildTag = document.getElementById('sc').children[0].tagName;\
                 globalThis.__scChildId = document.getElementById('sc').children[0].id;\
                 globalThis.__wN = document.getElementById('w').children.length;\
                 globalThis.__wFirst = document.getElementById('w').children[0].tagName;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scN)").unwrap(),
            "1",
            "surroundContents 后 #sc 仅 1 子（wrap）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scChildTag)").unwrap(),
            "DIV",
            "#sc 唯一子为 wrap（DIV）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scChildId)").unwrap(),
            "w",
            "#sc 子 id=w"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wN)").unwrap(),
            "2",
            "wrap 含 2 个克隆子（原 span A/B）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wFirst)").unwrap(),
            "SPAN",
            "wrap 首子为 SPAN"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_page_lifecycle_r2931() {
        // R2931 页面生命周期/分析簇：navigator.sendBeacon（卸载 beacon，accept-and-return-true）+
        // PageTransitionEvent 构造器 + pageshow 首次注册 _defer 派发（window + document 路径）。
        let mut worker = RendererJsWorker::spawn(38);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__beacon1 = navigator.sendBeacon('/analytics', { x: 1 });\
                 globalThis.__beacon2 = navigator.sendBeacon();\
                 globalThis.__pt1 = new PageTransitionEvent('pageshow', { persisted: true }).persisted;\
                 globalThis.__pt2 = new PageTransitionEvent('pageshow').persisted;\
                 globalThis.__ev = (document.createEvent('PageTransitionEvent').constructor === globalThis.PageTransitionEvent);\
                 globalThis.__ps = 'no'; globalThis.__ps2 = 'no';\
                 window.addEventListener('pageshow', function (e) {\
                   globalThis.__ps = e.type + ':' + String(e.persisted);\
                 });\
                 document.addEventListener('pageshow', function (e) {\
                   globalThis.__ps2 = e.type;\
                 });",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon1)").unwrap(),
            "true",
            "sendBeacon(url, data) → true（accept）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon2)").unwrap(),
            "false",
            "sendBeacon() 缺 url → false"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt1)").unwrap(),
            "true",
            "PageTransitionEvent persisted:true → persisted true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt2)").unwrap(),
            "false",
            "PageTransitionEvent 默认 persisted false"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ev)").unwrap(),
            "true",
            "createEvent('PageTransitionEvent') 构造器匹配"
        );
        // pageshow 经首次注册 _defer 派发（execute 末 drain）；window + document 两路径 listener 均收。
        let ps = wait_eq(&worker, "__ps", "pageshow:false", 2000);
        assert_eq!(
            ps, "pageshow:false",
            "window pageshow listener 触发（type + persisted:false）"
        );
        let ps2 = wait_eq(&worker, "__ps2", "pageshow", 2000);
        assert_eq!(ps2, "pageshow", "document pageshow listener 亦触发（同一次 dispatch）");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_window_on_handlers_r2932() {
        // R2932 window IDL on-event handler：on* setter 经 _globalAddEventListener 注册为 listener（移除旧），
        // getter 返存储 fn；=null 移除。window.dispatchEvent 合成派发可触 handler。onpageshow 触发 R2931 派发。
        let mut worker = RendererJsWorker::spawn(39);
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
            "window.onload = h1 → getter 返同一 fn（identity）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c1)").unwrap(),
            "1",
            "dispatch load → h1 触发一次"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c2)").unwrap(),
            "1",
            "重赋 onload=h2 → h2 触发一次（h1 已移除不再触发，c1 仍 1）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__null)").unwrap(),
            "true",
            "window.onload = null → getter 返 null（移除）"
        );
        // onpageshow 经 setter→_globalAddEventListener 触发 R2931 首次注册 _defer 派发。
        let ps = wait_eq(&worker, "__ps", "pageshow:false", 2000);
        assert_eq!(ps, "pageshow:false", "onpageshow setter 触发 pageshow 派发");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_element_on_handlers_r2933() {
        // R2933 element 级 IDL on-event handler：onclick/oninput setter 路由到 per-element listener store
        //（先于 set trap 末尾属性 fallthrough，否则 fn 被当字符串属性写）；getter 返存储 fn；=null 移除；
        // dispatchEvent 触发。parsed 元素（sel-based）+ created 元素（handle-based）均覆盖。
        let mut worker = RendererJsWorker::spawn(40);
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
                 d.dispatchEvent(new Event('click'));\
                 var btn = document.createElement('button');\
                 globalThis.__bc = 0;\
                 function bh() { globalThis.__bc++; }\
                 btn.oninput = bh;\
                 globalThis.__bid = (btn.oninput === bh);\
                 btn.dispatchEvent(new Event('input'));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__did)").unwrap(),
            "true",
            "parsed 元素 d.onclick = dh → getter 返同一 fn（identity）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dc)").unwrap(),
            "1",
            "dispatchEvent click → onclick 触发一次（=null 后不再触发）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dnull)").unwrap(),
            "true",
            "d.onclick = null → getter 返 null（移除）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__bid)").unwrap(),
            "true",
            "created 元素 btn.oninput = bh → getter 返同一 fn"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__bc)").unwrap(),
            "1",
            "created 元素 dispatchEvent input → oninput 触发"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_inline_html_handlers_r2934() {
        // R2934 inline HTML event handler：`<button onclick="...">` on* 属性编译为函数（new Function + with(this)
        // scope），on* getter 返编译 fn；dispatchEvent/click 触发；JS 设值覆盖 inline；无 inline 元素 onclick===null。
        let mut worker = RendererJsWorker::spawn(41);
        let html = "<html><body>\
                    <button id='b' onclick=\"globalThis.__inline='yes'\">\
                      <span id='s' onclick=\"globalThis.__tag=this.tagName\"></span>\
                    </button>\
                    <button id='b2' onclick=\"globalThis.__cfired='yes'\"></button>\
                    <div id='d'></div>\
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
                 s.dispatchEvent(new Event('click'));\
                 b.onclick = function () { globalThis.__js = 'yes'; };\
                 globalThis.__js = 'no';\
                 b.dispatchEvent(new Event('click'));\
                 globalThis.__cfired = 'no';\
                 document.getElementById('b2').click();\
                 var d = document.getElementById('d');\
                 globalThis.__disnull = (d.onclick === null);",
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
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__js)").unwrap(),
            "yes",
            "JS 覆盖 inline → JS handler 触发"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cfired)").unwrap(),
            "yes",
            "click() 方法触发 inline handler"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__disnull)").unwrap(),
            "true",
            "无 inline 无 JS → onclick === null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_inline_handler_ancestor_bubble_r2935() {
        // R2935 祖先 inline handler 冒泡触发：R2934 仅 target 阶段 ensure；补 capture/bubble 祖先阶段 →
        // <div onclick><button> 点 button 冒泡到 div 触发其 inline handler（this=祖先 currentTarget）。
        // 非 bubbles 事件不触发祖先 inline。
        let mut worker = RendererJsWorker::spawn(42);
        let html = "<html><body>\
                    <div id='outer' onclick=\"globalThis.__outer=this.id\">\
                      <div id='inner' onclick=\"globalThis.__inner=this.id\">\
                        <button id='btn'>x</button>\
                      </div>\
                    </div>\
                    <div id='o2' onclick=\"globalThis.__o2='yes'\"><button id='b2'>x</button></div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__outer = 'no'; globalThis.__inner = 'no';\
                 document.getElementById('btn').dispatchEvent(new Event('click', { bubbles: true }));\
                 globalThis.__o2 = 'no';\
                 document.getElementById('b2').dispatchEvent(new Event('click', { bubbles: false }));",
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
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__o2)").unwrap(),
            "no",
            "非 bubbles 事件不触发祖先 inline handler"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_clipboard_events_r2936() {
        // R2936 剪贴板事件：ClipboardEvent 构造器 + document.execCommand('copy') 派发 ClipboardEvent 到
        // document.activeElement（焦点元素，bubbles+cancelable）→ copy listener + oncopy handler + 冒泡到 window。
        let mut worker = RendererJsWorker::spawn(43);
        let html = "<html><body><input id='inp'></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__cd = new ClipboardEvent('copy', { clipboardData: 'dt' }).clipboardData;\
                 globalThis.__evc = (document.createEvent('ClipboardEvent').constructor === globalThis.ClipboardEvent);\
                 var inp = document.getElementById('inp');\
                 inp.focus();\
                 globalThis.__copy = 'no';\
                 inp.addEventListener('copy', function (e) {\
                   globalThis.__copy = e.type + ':' + (e.constructor === globalThis.ClipboardEvent);\
                 });\
                 globalThis.__oncopy = 'no';\
                 inp.oncopy = function (e) { globalThis.__oncopy = e.type; };\
                 globalThis.__wcopy = 'no';\
                 window.addEventListener('copy', function (e) { globalThis.__wcopy = e.type; });\
                 document.execCommand('copy');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cd)").unwrap(),
            "dt",
            "ClipboardEvent clipboardData:'dt' → clipboardData 字段"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__evc)").unwrap(),
            "true",
            "createEvent('ClipboardEvent') 构造器匹配"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__copy)").unwrap(),
            "copy:true",
            "execCommand('copy') → 焦点元素 copy listener 触发（type + ClipboardEvent）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__oncopy)").unwrap(),
            "copy",
            "execCommand('copy') → oncopy handler 触发（R2933 on* 路由）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wcopy)").unwrap(),
            "copy",
            "copy 事件冒泡到 window listener"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_drag_and_drop_r2937() {
        // R2937 Drag & Drop API：DataTransfer（setData/getData/types）+ DragEvent（extends MouseEvent +
        // dataTransfer）+ createEvent 注册。drag 事件类型经 generic addEventListener/ondrop（R2933）+ dispatchEvent
        // 触发。headless 无真拖拽源，但库 / drop handler 经合成 DragEvent + dataTransfer 读写 payload。
        let mut worker = RendererJsWorker::spawn(44);
        let html = "<html><body><div id='dz'></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var dt = new DataTransfer();\
                 dt.setData('text/plain', 'hello');\
                 dt.setData('text/html', '<b>hi</b>');\
                 globalThis.__dt1 = dt.getData('text/plain');\
                 globalThis.__dt2 = dt.getData('text/html');\
                 globalThis.__dt3 = dt.getData('text/missing');\
                 globalThis.__types = dt.types.join(',');\
                 globalThis.__evc = (document.createEvent('DragEvent').constructor === globalThis.DragEvent);\
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
                 dz.dispatchEvent(new DragEvent('drop', { dataTransfer: dt2, bubbles: true, cancelable: true }));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt1)").unwrap(),
            "hello",
            "DataTransfer.setData/getData text/plain"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt2)").unwrap(),
            "<b>hi</b>",
            "DataTransfer.setData/getData text/html"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt3)").unwrap(),
            "",
            "getData 未设格式 → ''"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__types)").unwrap(),
            "text/plain,text/html",
            "DataTransfer.types（插入序）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__evc)").unwrap(),
            "true",
            "createEvent('DragEvent') 构造器匹配"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__drop)").unwrap(),
            "drop:payload:true",
            "dispatchEvent DragEvent('drop') → drop listener 触发（dataTransfer.getData + 构造器）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ondrop)").unwrap(),
            "drop",
            "ondrop handler 触发（R2933 on* 路由）"
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
    fn renderer_js_worker_get_bounding_client_rect_handle_identity_create_element() {
        // P1a gBCR path A：createElement 元素（JS 持 handle `__n{n}`，sel 空）的 getBoundingClientRect
        // 返真实 rect。流程模拟生产 apply 路径：脚本1 createElement+setId+append（记录 mutations）
        // → apply_mutations_to_html_with_handles 产出 handle→selector map → merge 进 worker 持久 map
        // → set_dom_snapshot 新 html（含已 append 元素）→ 填 snapshot → 脚本2 经 handle 测量返真实 rect。
        use zero_dom::parse_html;
        use zero_engine::{apply_mutations_to_html_with_handles, find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(22);
        let html0 = "<html><body id='b'></body></html>";
        worker.set_dom_snapshot(html0, "about:blank");
        // 脚本1：创建 div、设 id、append（handle 持于 globalThis.__el）。
        worker
            .execute_script_direct(
                "globalThis.__el = document.createElement('div');\
                 globalThis.__el.id = 'dyn';\
                 document.body.appendChild(globalThis.__el);",
            )
            .unwrap();
        // 模拟 apply_recorded_mutations：取记录的 mutations 应用到 html0，得新 html + handle→selector map。
        let recorded = worker.mutations().lock().unwrap().clone();
        let (html1, handle_map) = apply_mutations_to_html_with_handles(html0, &recorded).unwrap();
        assert!(
            html1.contains("<div id=\"dyn\">"),
            "createElement+setId+append 应产出 <div id=\"dyn\">，got: {html1}"
        );
        assert_eq!(handle_map.len(), 1, "唯一选择器映射应只含一个 createElement handle");
        assert_eq!(
            handle_map.values().next(),
            Some(&"#dyn".to_string()),
            "handle → #dyn（id 唯一）"
        );
        // merge map 进 worker 持久 map（= page_scripts::apply_recorded_mutations 的行为）。
        worker.handle_selector_map().lock().unwrap().extend(handle_map);
        // 更新 dom_html 为含已 append 元素的新 html（= 下一次 set_dom_snapshot）。
        worker.set_dom_snapshot(&html1, "about:blank");
        // 填 snapshot：fresh-parse html1 取 #dyn NodeId（= 渲染管线会用同一 NodeId）。
        let doc = parse_html(&html1);
        let id_dyn = find_by_selector(&doc, "#dyn").expect("#dyn in html1");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_dyn), (30.0, 40.0, 100.0, 50.0));
        // 脚本2：经 handle（sel 空）测量 → path A 解析 handle→#dyn→NodeId→snapshot rect。
        worker
            .execute_script_direct(
                "var r = globalThis.__el.getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__l = r.left;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__w)").unwrap(),
            "100",
            "handle-identity gBCR width 应反映 snapshot"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__l)").unwrap(),
            "30",
            "handle-identity gBCR left 应反映 snapshot"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_handle_identity_ambiguous_tag() {
        // P1a gBCR path A + nth-child 结构路径：无 id/class 的 createElement 元素，文档已有同 tag
        // 元素（歧义）→ stable_selector 不唯一 → 回落 nth-child 结构路径 → 仍返真实 rect
        // （path A 限制①「tag-only 歧义→零 rect」的收尾）。
        use zero_dom::parse_html;
        use zero_engine::{apply_mutations_to_html_with_handles, find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(23);
        // body 已有一个 div（使新 div 的 "div" 选择器歧义）。
        let html0 = "<html><body id='b'><div>existing</div></body></html>";
        worker.set_dom_snapshot(html0, "about:blank");
        // 脚本1：创建无 id/class 的 div 并 append（歧义 tag）。
        worker
            .execute_script_direct(
                "globalThis.__el = document.createElement('div');\
                 document.body.appendChild(globalThis.__el);",
            )
            .unwrap();
        let recorded = worker.mutations().lock().unwrap().clone();
        let (html1, handle_map) = apply_mutations_to_html_with_handles(html0, &recorded).unwrap();
        assert_eq!(handle_map.len(), 1, "一个 createElement handle");
        let (handle, sel) = handle_map.iter().next().unwrap();
        let sel = sel.clone();
        assert!(
            sel.contains("nth-child"),
            "歧义 tag 应回落 nth-child 结构路径，got handle={handle} sel={sel}"
        );
        // merge + 更新 dom_html。
        worker.handle_selector_map().lock().unwrap().extend(handle_map);
        worker.set_dom_snapshot(&html1, "about:blank");
        // 用结构路径解析出该 handle 的 NodeId（= 渲染管线会用同一 NodeId），填 snapshot。
        let doc = parse_html(&html1);
        let id_el = find_by_selector(&doc, &sel).expect("结构路径须可解析");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_el), (5.0, 6.0, 80.0, 40.0));
        // 脚本2：经 handle 测量歧义元素 → 结构路径解析 → 真实 rect（非零）。
        worker
            .execute_script_direct(
                "var r = globalThis.__el.getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__h = r.height;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__w)").unwrap(),
            "80",
            "歧义 tag 经结构路径应返真实 width"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__h)").unwrap(),
            "40",
            "歧义 tag 经结构路径应返真实 height"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_select_value_read_and_setter() {
        // P1a select：<select>.value 读（选中 option 的 value）+ selectedIndex + option.selected，
        // + 编程设 select.value=x（SelectOption mutation，apply 后反映）。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(24);
        // option b 默认 selected。
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 读：value='b'、selectedIndex=1、option b selected=true / a selected=false。
        worker
            .execute_script_direct(
                "var s = document.querySelector('#s');\
                 globalThis.__v = s.value;\
                 globalThis.__i = s.selectedIndex;\
                 globalThis.__sb = document.querySelector('#s > option:nth-of-type(2)').selected;\
                 globalThis.__sa = document.querySelector('#s > option:nth-of-type(1)').selected;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__v)").unwrap(), "b");
        assert_eq!(worker.execute_script_direct("String(globalThis.__i)").unwrap(), "1");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sb)").unwrap(),
            "true",
            "option b 应 selected"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sa)").unwrap(),
            "false",
            "option a 应未 selected"
        );
        // 编程设 select.value='c'（记录 SelectOption mutation）→ apply → 反映。
        worker
            .execute_script_direct("document.querySelector('#s').value = 'c';")
            .unwrap();
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).unwrap();
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct("globalThis.__v2 = document.querySelector('#s').value;")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__v2)").unwrap(),
            "c",
            "setter 后 select.value 应为 c"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_query_selector_all_unique_identity() {
        // P1a querySelectorAll 唯一选择器：`querySelectorAll('option')` 每元素返唯一身份（nth-child
        // 结构路径），各 `.value`/`.selected` 读对（此前全返 "option"→全指向首个 option，读全错）。
        let mut worker = RendererJsWorker::spawn(25);
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var opts = document.querySelectorAll('#s option');\
                 globalThis.__n = opts.length;\
                 globalThis.__vals = opts.map(function(o){ return o.value; }).join(',');\
                 globalThis.__sels = opts.map(function(o){ return o.selected ? '1':'0'; }).join(',');",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__n)").unwrap(), "3");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__vals)").unwrap(),
            "a,b,c",
            "各 option.value 应读对（唯一身份）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sels)").unwrap(),
            "0,1,0",
            "各 option.selected 应读对（b 选中）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_select_options_collection() {
        // P1a select：`select.options` 集合（length/索引/value/selectedIndex）+
        // `select.selectedOptions`（选中 option 数组）。
        let mut worker = RendererJsWorker::spawn(26);
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var s = document.querySelector('#s');\
                 globalThis.__len = s.options.length;\
                 globalThis.__v0 = s.options[0].value;\
                 globalThis.__v2 = s.options[2].value;\
                 globalThis.__ov = s.options.value;\
                 globalThis.__oi = s.options.selectedIndex;\
                 globalThis.__item = s.options.item(1).value;\
                 globalThis.__selN = s.selectedOptions.length;\
                 globalThis.__selV = s.selectedOptions[0].value;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__len)").unwrap(), "3");
        assert_eq!(worker.execute_script_direct("String(globalThis.__v0)").unwrap(), "a");
        assert_eq!(worker.execute_script_direct("String(globalThis.__v2)").unwrap(), "c");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ov)").unwrap(),
            "b",
            "options.value 应 = select.value（选中 b）"
        );
        assert_eq!(worker.execute_script_direct("String(globalThis.__oi)").unwrap(), "1");
        assert_eq!(worker.execute_script_direct("String(globalThis.__item)").unwrap(), "b");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__selN)").unwrap(),
            "1",
            "selectedOptions 应含 1 个（b）"
        );
        assert_eq!(worker.execute_script_direct("String(globalThis.__selV)").unwrap(), "b");
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
    fn renderer_js_worker_raf_tick_fires_frame_driven_callbacks() {
        // R2713b：renderer worker 帧驱动 rAF——__ZW_RAF_FRAME_DRIVEN=true 时 requestAnimationFrame
        // 注册延后到 __zw_raf_tick（renderer `tick_observers` 在 post-render 调 `__zw_raf_tick`；
        // 本测试 JS 侧直调验证 shim 在 renderer worker 上下文正确，env set_var 在并行测试下有竞态
        // 故 JS 侧注入 flag）。tick 前不 fire，tick 后按序 fire。
        let mut worker = RendererJsWorker::spawn(28);
        worker.set_dom_snapshot("<html><body><div id='t'>hi</div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__ZW_RAF_FRAME_DRIVEN = true;\
                 globalThis.__count = 0;\
                 requestAnimationFrame(function(){ globalThis.__count++; });\
                 requestAnimationFrame(function(){ globalThis.__count++; });",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__count)").unwrap(),
            "0",
            "帧驱动：tick 前回调不应 fire"
        );
        worker
            .execute_script_direct("if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(0);")
            .unwrap();
        assert_eq!(
            wait_eq(&worker, "__count", "2", 1000),
            "2",
            "tick 后按注册序 fire 两个回调"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_performance_now_available() {
        // R2769：renderer worker 上下文 performance.now() 可用（register_dom_callbacks 注册
        // __zw_performance_now，R2768 land）——证明 tick_observers 的
        // `__zw_raf_tick(performance.now())`（page_scripts.rs）真 ts 参数在 renderer 路径有效。
        let mut worker = RendererJsWorker::spawn(33);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        assert_eq!(
            worker.execute_script_direct("typeof performance.now").unwrap(),
            "function",
            "performance.now 应为 function"
        );
        assert_eq!(
            worker.execute_script_direct("String(performance.now() >= 0)").unwrap(),
            "true",
            "performance.now() 非负（renderer 上下文真单调时钟）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_refires_on_threshold_cross() {
        // R2714：IO 持续跟踪（Slice 2b 已就绪——post-render `__zw_observers_tick` → IO `_schedule`
        // → `_crossed` threshold 越界 → 再派发）。observe（initial：target 在 root 外 ratio 0）→
        // 更新 snapshot（target 移入 root，ratio 跨 threshold 0.5）→ tick → 再派发（isIntersecting
        // false→true，__calls 1→2）。显式 root + threshold 0.5 使几何确定（不受 viewport 影响）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(29);
        let html = "<html><body><div id='root'><div id='t'>hi</div></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_root = find_by_selector(&doc, "#root").expect("#root");
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        // v1：root 200x200，target 在 root 外（1000,1000）→ ratio 0、isIntersecting false。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_root), (0.0, 0.0, 200.0, 200.0));
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (1000.0, 1000.0, 100.0, 100.0));
        worker
            .execute_script_direct(
                "globalThis.__calls = 0;\
                 globalThis.__intersecting = null;\
                 var obs = new IntersectionObserver(function(entries){\
                   globalThis.__calls = (globalThis.__calls | 0) + 1;\
                   globalThis.__intersecting = String(entries[0].isIntersecting);\
                 }, { root: document.querySelector('#root'), threshold: 0.5 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        // initial 派发（ratio 0，isIntersecting false）。
        assert_eq!(wait_eq(&worker, "__calls", "1", 1000), "1");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__intersecting)")
                .unwrap(),
            "false",
            "initial：target 在 root 外 → isIntersecting false"
        );
        // v2：target 移入 root（10,10）→ ratio 1.0 跨 threshold 0.5 → tick 再派发。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 10.0, 100.0, 100.0));
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        assert_eq!(wait_eq(&worker, "__calls", "2", 1000), "2");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__intersecting)")
                .unwrap(),
            "true",
            "tick 后 target 移入 root → isIntersecting true"
        );
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
    fn renderer_js_worker_form_textarea_newline_via_text_input() {
        // P1a form input：textarea 的 Enter 经 host 路由为 `__zw_text_input('#ta', '\n')`
        //（main.rs handle_keyboard_event：textarea Enter → 换行，非 submit）。验证 '\n' append 到
        // textarea value + 派发 'input'。修复前 textarea Enter 为 no-op（多行输入断裂）。
        let mut worker = RendererJsWorker::spawn(31);
        worker.set_dom_snapshot(
            "<html><body><textarea id='ta'>ab</textarea></body></html>",
            "about:blank",
        );
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#ta');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_input('#ta', '\\n');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab\n"
        );
        // 再加 'c' → "ab\nc"（缓存跨 execute 存活，多行 typing 成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_input('#ta', 'c');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab\nc"
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
    fn renderer_js_worker_checkbox_checked_reflection_and_change_dispatch() {
        // P1a checkbox：`el.checked` getter 经 `__zw_has_attr` 反映 boolean 属性存在性；
        // change 事件经 shim 派发命中 listener（toggle 由 host `apply_toggle_checkbox` 覆盖，
        // engine 单测覆盖 RemoveAttr/has_attribute/is_checkbox）。
        let mut worker = RendererJsWorker::spawn(30);
        worker.set_dom_snapshot(
            "<html><body><input id='on' type='checkbox' checked><input id='off' type='checkbox'></body></html>",
            "about:blank",
        );
        // el.checked 反映存在性。
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#on').checked)")
                .unwrap(),
            "true"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#off').checked)")
                .unwrap(),
            "false"
        );
        // change 派发命中 listener（e.target.checked 读当前状态）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 document.querySelector('#off').addEventListener('change', function(e){\
                   globalThis.__seen = 'change:' + String(e.target.checked);\
                 });\
                 __zw_dispatch_event('#off', 'change', null);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "change:false"
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
