//! V8引擎运行时实现。
//!
//! 封装 v8 crate，提供安全的JavaScript脚本执行沙箱。

use std::cell::RefCell;
use std::sync::{Arc, Once};

use crate::{SandboxConfig, ScriptError, ScriptResult};

/// 从 TryCatch 提取异常消息与堆栈（须在调用点展开以匹配具体 scope 类型）。
macro_rules! v8_try_catch_message {
    ($try_catch:expr) => {{
        let try_catch = $try_catch;
        if try_catch.has_caught() {
            if let Some(exception) = try_catch.exception() {
                let mut out = exception
                    .to_string(try_catch)
                    .map(|s| s.to_rust_string_lossy(try_catch))
                    .unwrap_or_else(|| "exception".to_string());
                // t5（qq-20261016-r1）execute 错误栈捕获：栈文本 = TryCatch::stack_trace()
                //（Message 面）优先；无帧行时回落异常对象 `.stack` 属性（Error.stack，
                // promise-reject 报告同径）——部分抛出形态 Message 面无帧而对象属性仍在。
                // 栈文本首行为消息本体，只拼帧行：无帧不拼（消除 t2 qq.com 'set'
                // pageerror 实测的 msg\nmsg 重复形态——页面设 Error.stackTraceLimit=0
                // 或栈捕获关闭时 stack 文本退化为消息单行）。
                let mut stack_text = try_catch
                    .stack_trace()
                    .and_then(|stack| stack.to_string(try_catch))
                    .map(|s| s.to_rust_string_lossy(try_catch))
                    .filter(|text| text.contains('\n'));
                if stack_text.is_none() {
                    stack_text = exception
                        .try_cast::<v8::Object>()
                        .ok()
                        .and_then(|obj| {
                            v8::String::new(try_catch, "stack").and_then(|key| obj.get(try_catch, key.into()))
                        })
                        .filter(|value| value.is_string())
                        .and_then(|value| value.to_string(try_catch))
                        .map(|s| s.to_rust_string_lossy(try_catch))
                        .filter(|text| text.contains('\n'));
                }
                if let Some(text) = stack_text
                    && let Some((_head, frames)) = text.split_once('\n')
                {
                    out.push('\n');
                    out.push_str(frames);
                }
                out
            } else {
                "unknown error".to_string()
            }
        } else {
            "unknown error".to_string()
        }
    }};
}
// worker.rs 的 Error 事件提取复用同一宏（macro_rules 文本作用域，须显式 re-export）。
pub(crate) use v8_try_catch_message;

/// 宿主注入的扁平字符串回调类型（JS-DOM-bridge：JS shim 经 __zw_* 回调操作宿主状态）。
///
/// 回调接收 JS 传入参数的字符串数组，返回字符串结果（写入 V8 ReturnValue）。
/// `Send + Sync + 'static` 以便能跨 V8 调用边界存于线程局部注册表。
pub type HostCallback = Arc<dyn Fn(&[String]) -> String + Send + Sync + 'static>;

/// s88（html-semantics-compat dynamic-import classic 面）：V8 原生动态 `import()`
/// 宿主解析器——服务 classic script / eval 字符串 / Function 构造器 / 事件处理器
/// 上下文的 `import()`（这些面源码重写不可达，须 V8 isolate 回调）。
///
/// 契约与 `__zw_compile_module` 宿主回调完全一致（es_module `__zw_load_module`
/// 三态）：返回 `resolved\u{1f}code`（IIFE，求值为 module namespace）、
/// `resolved\u{1f}`（依赖图编译失败）、空串（取回失败）。spec/referrer/nonce 均为
/// 宿主解析语义（referrer/nonce 由 [`Sandbox::set_dynamic_import_referrer`] /
/// [`Sandbox::set_dynamic_import_nonce`] 每脚本前设定；nonce 空串 = 无 nonce，
/// s92 string-compilation CSP nonce 面——宿主据此对外链 module fetch 做 script-src
/// 检查，阻止返空串 → reject TypeError）。
#[allow(clippy::type_complexity)] // 镜像 HostCallback 形态（Arc 闭包跨 V8 回调边界）
pub type DynamicImportResolver = Arc<dyn Fn(&str, &str, &str) -> String + Send + Sync + 'static>;

/// 动态 import() 宿主状态（isolate slot 持有——同线程多 sandbox 状态隔离）。
#[derive(Default)]
struct DynamicImportState {
    /// 当前执行脚本的 referrer（classic 路径 ScriptOrigin 缺席，宿主显式供给）。
    referrer: String,
    /// s92（string-compilation CSP nonce 面）：当前执行脚本的 nonce 属性值
    /// （None = 无 nonce——字符串编译产物在事件处理器求值窗被 shim 换入 None，
    /// spec 事件处理器不继承触发脚本 nonce）。
    nonce: Option<String>,
    /// 宿主解析器；None = 维持 V8 默认语义（reject 普通 `Error("Not supported")`，
    /// 未接线宿主 renderer/tab 行为不劣化）。
    resolver: Option<DynamicImportResolver>,
}

/// 动态 import() V8 isolate 回调。失败三态与 es_module `__zw_load_module` 对齐：
/// 取回失败 → 新 TypeError（classic 上下无语料身份断言，`__moduleFailures` 记账
/// 不接线——FIXME: 与模块面共享 module map 失败记账）；IIFE parse/求值异常 →
/// 原异常对象 reject。嵌套动态 import()（IIFE 内重写体）依赖 `__zw_dynamic_import`
/// 全局 helper——classic 上下文缺席时按需装模块 prelude（空 registry，无急切求值），
/// helper 自身又依赖宿主注册的 `__zw_compile_module` 全局回调（webview 同块注册）。
#[allow(unused_variables)]
fn dynamic_import_callback<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    host_defined_options: v8::Local<'s, v8::Data>,
    resource_name: v8::Local<'s, v8::Value>,
    specifier: v8::Local<'s, v8::String>,
    import_attributes: v8::Local<'s, v8::FixedArray>,
) -> Option<v8::Local<'s, v8::Promise>> {
    let spec = specifier.to_rust_string_lossy(scope);
    let state = scope.get_slot::<Arc<std::sync::Mutex<DynamicImportState>>>().cloned()?;
    let promise_resolver = v8::PromiseResolver::new(scope)?;
    let contract = match state.lock() {
        Ok(st) => match st.resolver.as_ref() {
            Some(resolver) => resolver(&spec, &st.referrer, st.nonce.as_deref().unwrap_or("")),
            // 未注册宿主：V8 默认行为即 reject 普通 Error("Not supported")（原生效语义）。
            None => {
                if let Some(msg) = v8::String::new(scope, "Not supported")
                    && promise_resolver
                        .reject(scope, v8::Exception::error(scope, msg))
                        .is_none()
                {
                    return None;
                }
                return Some(promise_resolver.get_promise(scope));
            }
        },
        Err(_) => return None, // 毒化锁（前置 panic）——V8 按 promise 创建失败抛出。
    };
    // 三态解析（同 __zw_load_module）。
    if contract.is_empty() {
        let msg = format!("Failed to fetch module: '{spec}'");
        if let Some(m) = v8::String::new(scope, &msg) {
            promise_resolver.reject(scope, v8::Exception::type_error(scope, m));
        }
        return Some(promise_resolver.get_promise(scope));
    }
    let Some((resolved, code)) = contract.split_once('\u{1f}') else {
        let msg = format!("Failed to fetch module: '{spec}'");
        if let Some(m) = v8::String::new(scope, &msg) {
            promise_resolver.reject(scope, v8::Exception::type_error(scope, m));
        }
        return Some(promise_resolver.get_promise(scope));
    };
    if code.is_empty() {
        let msg = format!("Failed to fetch module graph: '{resolved}'");
        if let Some(m) = v8::String::new(scope, &msg) {
            promise_resolver.reject(scope, v8::Exception::type_error(scope, m));
        }
        return Some(promise_resolver.get_promise(scope));
    }
    // classic 上下文 helper 缺席面：嵌套动态 import() 走 prelude helper（幂等——
    // 模块脚本先于此执行时已装，presence 检查免重复求值）。
    let context = scope.get_current_context();
    // s124（modulepreload 面）：成功条目 module map 记忆化——classic 上下文 import()
    // 原生回调此前每次调用都重取回+重求值（module map 语义缺口）。命中
    // `__moduleCache[resolved]`（宿主解析键，与 `__zw_load_module` helper 面同表——
    // link rel=modulepreload shim 取回面的播种条目同键复用）→ 直接以缓存值 resolve，
    // 模块体恰好一次求值（HTML fetch-a-single-module-script 命中即复用）。失败三态
    // 不入缓存（取回失败逐次新 TypeError——s87 原口径，spec 同）。best-effort：缓存
    // 读取失败按未命中续行。
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    let cached_entry = v8::String::new(scope, "__moduleCache")
        .and_then(|key| context.global(scope).get(scope, key.into()))
        .filter(|v| v.is_object())
        .and_then(|cache| {
            v8::String::new(scope, resolved).and_then(|k| cache.to_object(scope).and_then(|o| o.get(scope, k.into())))
        })
        .filter(|v| !v.is_undefined());
    if let Some(entry) = cached_entry {
        promise_resolver.resolve(scope, entry);
        return Some(promise_resolver.get_promise(scope));
    }
    let has_helpers = v8::String::new(scope, "__zw_dynamic_import")
        .and_then(|key| context.global(scope).get(scope, key.into()))
        .map(|v| !v.is_undefined())
        .unwrap_or(false);
    let mut script_src = String::with_capacity(code.len() + 256);
    if !has_helpers && let Ok(prelude) = crate::build_module_runtime_prelude(&crate::ModuleRegistry::new(), None) {
        script_src.push_str(&prelude);
    }
    script_src.push_str(code);
    // compile/run 须在自有 TryCatch 内——宿主回调 scope 非 TryCatch，裸 run 的
    // 异常会按 uncaught 上抛（永不达 promise rejection）。
    v8::tc_scope!(let tc, scope);
    let result = v8::String::new(tc, &script_src)
        .and_then(|src| v8::Script::compile(tc, src, None))
        .and_then(|script| script.run(tc));
    match result {
        Some(value) => {
            // s124：成功条目写入 module map（与读取同表同键；表缺席则建——classic
            // 上下文 prelude 按需装时已含 init，此处兜底自建覆盖无 prelude 面）。
            // best-effort：写入失败不影响 resolve（下次调用重取回重求值，原行为）。
            if let Some(cache_key) = v8::String::new(tc, "__moduleCache") {
                let global = tc.get_current_context().global(tc);
                let cache_obj = global
                    .get(tc, cache_key.into())
                    .filter(|v| v.is_object())
                    .and_then(|v| v.to_object(tc))
                    .unwrap_or_else(|| {
                        let obj = v8::Object::new(tc);
                        let _ = global.set(tc, cache_key.into(), obj.into());
                        obj
                    });
                if let Some(entry_key) = v8::String::new(tc, resolved) {
                    let _ = cache_obj.set(tc, entry_key.into(), value);
                }
            }
            promise_resolver.resolve(tc, value);
        }
        None => {
            // parse（SyntaxError）/求值异常 → 原异常对象 reject（spec module record
            // evaluation error 语义；classic 上下文 identity 记账不接线，见上 FIXME）。
            if let Some(exception) = tc.exception() {
                promise_resolver.reject(tc, exception);
            }
        }
    }
    Some(promise_resolver.get_promise(tc))
}

// 宿主回调注册表（线程局部）。FunctionTemplate 回调须为 `Copy`
//（MapFnTo<FunctionCallback>），无法捕获 Arc 状态；故回调闭包存于此注册表，
// FunctionTemplate 经 builder().data(idx) 携带索引，fn 回调按 idx 查表调用。
//
// 槽位为 `Option<HostCallback>`（所有权语义）：sandbox 自有槽位由其 `Drop`
// 置 None（own-slot 清空），他 sandbox 的槽位不受影响——t8p 审查 D1/D2：
// 旧实现整表 `Vec<HostCallback>` + Drop 无条件整表清空，同线程多 sandbox
// 存活期重叠时，幸存者记录的 idx 失效（同名重注册越界 panic）或撞写他人
// 活槽位（跨实例回调被静默替换）。置 None 的洞不再复用（无 free-list），
// 表长以线程历史上**不同名字的注册总数**为上界。
thread_local! {
    static HOST_CALLBACKS: RefCell<Vec<Option<HostCallback>>> = RefCell::new(Vec::new());
    // P105 诊断：与 HOST_CALLBACKS 平行的名字表（同槽位同步置 None）——
    // ZW_HOST_CALL_TRACE 慢宿主调用归因用，invoke 处按 idx 取名。仅诊断读取，
    // 不参与分派语义。
    static HOST_CALLBACK_NAMES: RefCell<Vec<Option<String>>> = const { RefCell::new(Vec::new()) };
}

/// 清空线程局部宿主回调注册表（整表）。
///
/// 2026-08-10 windows tab_js_worker 全量挂起修复（根因）：回调闭包捕获的资源
/// 可能拥有后台线程（如 `__zw_compile_module` 捕获的 `HttpClient` = reqwest
/// blocking Client，构造即 spawn dispatcher 线程）。若闭包留到**线程退出**才由
/// Rust TLS 析构 drop——Windows 上 TLS 析构在 loader lock 下运行，闭包内资源的
/// drop（join 后台线程）与后台线程自身退出（同样需要 loader lock）互锁死锁；
/// Linux 无 loader lock 故不复现。清空后旧 idx 查询返 None（调用方
/// `unwrap_or_default` 兜底），语义安全。
///
/// **注意（t8p 审查 D1/D2）**：本函数清**整表**，会同时杀死同线程所有存活
/// sandbox 的宿主回调。`V8Sandbox::drop` 自 t8p 起改为 own-slot 清空（只置
/// None 自己的槽位），不再调用本函数；仅限「确认整线程无存活 sandbox」的
/// 整线程拆卸场景使用。
pub fn clear_host_callbacks() {
    HOST_CALLBACKS.with(|cbs| cbs.borrow_mut().clear());
    HOST_CALLBACK_NAMES.with(|names| names.borrow_mut().clear());
}

/// V8 FunctionTemplate 回调：按 args.data() 的索引查 HOST_CALLBACKS 调用。
fn host_callback_invoke(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let idx = args.data().integer_value(scope).unwrap_or(-1);
    if idx < 0 {
        return;
    }
    let n = args.length();
    let strs: Vec<String> = (0..n)
        .filter_map(|i| args.get(i).to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .collect();
    // P105 诊断：慢宿主调用归因（crate::HOST_CALL_TRACE 注释）。
    let _trace = if *crate::HOST_CALL_TRACE {
        let nm = HOST_CALLBACK_NAMES.with(|names| names.borrow().get(idx as usize).and_then(|slot| slot.clone()));
        crate::HostCallTraceGuard::arm(nm.as_deref().unwrap_or("__zw_idx_unknown"), &strs)
    } else {
        None
    };
    let result = HOST_CALLBACKS.with(|cbs| {
        cbs.borrow()
            .get(idx as usize)
            .and_then(|slot| slot.as_ref())
            .map(|cb| cb(&strs))
            .unwrap_or_default()
    });
    if let Some(s) = v8::String::new(scope, &result) {
        rv.set(s.into());
    }
}

/// 将 Rust 字符串转为 JS 字符串字面量（含两端双引号），转义特殊字符防注入。
/// 供 [`V8Sandbox::resolve_async_callback`] 把 `id`/`result` 安全嵌入执行脚本。
fn js_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// V8平台初始化守卫（全局只初始化一次）。
static V8_INIT: Once = Once::new();

struct IsolateEnterGuard {
    isolate: *const v8::OwnedIsolate,
}

impl Drop for IsolateEnterGuard {
    fn drop(&mut self) {
        // SAFETY: The guard is created only after entering this isolate, and it
        // is dropped before the owned isolate is disposed.
        unsafe {
            (*self.isolate).exit();
        }
    }
}

/// 确保V8平台已初始化。
///
/// P1b S1：engine 的原生 DOM 绑定（`zero_engine::dom_bindings`）现直接依赖 v8 crate，
/// 需在自建 Isolate 前确保平台初始化。本函数经 `pub use v8_runtime::*` 公开（feature-gated v8），
/// 进程级 `Once` 防重复初始化（V8 平台初始化须全局一次）。
pub fn ensure_v8_initialized() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        // SAFETY: V8平台初始化在进程生命周期内只调用一次，
        // 且在所有Isolate创建之前完成。
        #[allow(unused_unsafe)]
        unsafe {
            v8::V8::initialize_platform(platform);
        }
        v8::V8::initialize();
    });
}

/// R-baidu2/P3 slice-2：未捕获异常报告收集（promise-reject 回调写入
/// [`UNCAUGHT_REPORTS`] thread_local；`take_uncaught_reports` 在 JS 执行线程排空）。
pub type UncaughtReport = (String, u32, u32);

thread_local! {
    // 报告以 `Option` 存储：retraction（见下）置 None，排空时过滤。
    static UNCAUGHT_REPORTS: std::cell::RefCell<Vec<Option<UncaughtReport>>> = const { std::cell::RefCell::new(Vec::new()) };
    // 待定 rejection 的 promise 身份 → UNCAUGHT_REPORTS 下标。同 execute 内
    // 后挂 handler 时按 promise 身份撤回报告；`take_uncaught_reports` 排空即清，
    // 故跨 execute 才挂 handler 的报告照常上报（Chrome 语义：微任务检查点末
    // 仍无 handler 才上报）。
    static PENDING_REJECTS: std::cell::RefCell<Vec<(v8::Global<v8::Promise>, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// promise-reject 回调入口（值传参，V8 要求）。
unsafe extern "C" fn promise_reject_callback_entry(msg: v8::PromiseRejectMessage) {
    promise_reject_callback(&msg);
}

/// promise-reject 回调：`PromiseRejectWithNoHandler`（未处理 rejection）与
/// `PromiseHandlerAddedAfterReject`（撤回，见下）。
///
/// 报告文本对齐 Chrome console 形态：`Uncaught (in promise) <原因>`（原因经
/// `Value::to_string`）+ 原因 Error 对象的创建时符号化栈（读 `.stack` 属性——
/// `Object::get` 沿原型链查找，继承 accessor 值亦得；去掉原因头行后拼接）。
/// 行/列保持 0：上报点 JS 栈已空（rejection 自已完成的微任务浮出），
/// `Exception::create_message` 的 current-stack 路径取不到帧，创建时栈仅以
/// `.stack` 文本形态可得——结构化 stackTrace 挂后续切片。
///
/// 撤回语义（对齐 Chrome 同 execute 已处理 rejection 报 0）：
/// https://html.spec.whatwg.org/multipage/webappapis.html#about-to-be-notified-promises
/// —— rejection 只在微任务检查点末仍无 handler 才上报。V8 的 reject 回调在
/// reject 时刻即发 `PromiseRejectWithNoHandler`（此时 `.then`/`.catch` 尚未挂），
/// 立即上报会把随后同 execute 内挂上 handler 的 rejection 误报为 uncaught
/// （baidu live：`new Image()` 解码管线 reject 后同步 `.then(…, fail)` 仍被报）。
/// 故 reject 时先暂存 promise 身份；V8 随后发 `PromiseHandlerAddedAfterReject`
/// 时按身份撤回。跨 execute 才挂 handler 的报告照常上报（排空即清暂存）。
fn promise_reject_callback(msg: &v8::PromiseRejectMessage) {
    // 非「未处理 reject / 追加 handler」事件（已 settle 后的 reject/resolve）不涉报告。
    match msg.get_event() {
        v8::PromiseRejectEvent::PromiseRejectWithNoHandler => {}
        v8::PromiseRejectEvent::PromiseHandlerAddedAfterReject => {
            // SAFETY: 同下 NoHandler 分支——promise-reject 回调在 V8 持锁 isolate
            // 内被调用，`&PromiseRejectMessage` 是 CallbackScope 的文档化引导参数。
            v8::callback_scope!(unsafe scope, msg);
            let promise = v8::Global::new(scope, msg.get_promise());
            PENDING_REJECTS.with(|p| {
                let mut pending = p.borrow_mut();
                if let Some(pos) = pending.iter().position(|(id, _)| *id == promise) {
                    let (_, idx) = pending.remove(pos);
                    UNCAUGHT_REPORTS.with(|q| {
                        if let Some(slot) = q.borrow_mut().get_mut(idx) {
                            *slot = None;
                        }
                    });
                }
            });
            return;
        }
        _ => return,
    }
    // SAFETY: promise-reject 回调在 V8 持锁 isolate 内被调用，`&PromiseRejectMessage`
    // 是 CallbackScope 的文档化引导参数（v8 scope.rs `NewCallbackScope` impl）。
    v8::callback_scope!(unsafe scope, msg);
    // 原因 to_string / .stack 读取均可触发用户 JS（toString/valueOf/getter），
    // 统一入 TryCatch：异常就地吞掉（报告回退事件级文本），不泄漏进外层 execute。
    v8::tc_scope!(let tc, scope);
    let mut text = "unhandled promise rejection".to_string();
    if let Some(value) = msg.get_value() {
        if let Some(reason) = value.to_string(tc)
            && !tc.has_caught()
        {
            text = format!("Uncaught (in promise) {}", reason.to_rust_string_lossy(tc));
        }
        if let Ok(obj) = value.try_cast::<v8::Object>()
            && let Some(key) = v8::String::new(tc, "stack")
            && !tc.has_caught()
            && let Some(stack) = obj.get(tc, key.into())
            && stack.is_string()
        {
            let stack_s = stack
                .to_string(tc)
                .map(|s| s.to_rust_string_lossy(tc))
                .unwrap_or_default();
            if let Some((_head, frames)) = stack_s.split_once('\n') {
                text.push('\n');
                text.push_str(frames);
            }
        }
    }
    UNCAUGHT_REPORTS.with(|q| {
        let mut reports = q.borrow_mut();
        reports.push(Some((text, 0, 0)));
        PENDING_REJECTS.with(|p| {
            p.borrow_mut()
                .push((v8::Global::new(tc, msg.get_promise()), reports.len() - 1));
        });
    });
}

enum WatchdogMsg {
    Arm {
        seq: u64,
        timeout_ms: u64,
        handle: v8::IsolateHandle,
    },
    Disarm {
        seq: u64,
    },
    Stop,
}

/// 持久超时看门狗线程主循环：阻塞等待直到（a）收到消息或（b）已装载的截止时刻
/// 到期 → `terminate_execution`。无装载时无限等待（`recv_timeout(Duration::MAX)`）。
///
/// 2026-08-10 重构：旧实现每次 execute spawn+join 一个 OS 线程——高频执行
/// （wait_for_global 5ms 轮询等）下线程 churn 可观（Windows 线程创建尤贵）；
/// 改为每 sandbox 一个常驻线程 + seq 协议；timeout_ms 由 Arm 携带支持
/// set_timeout_ms 动态变化。
fn watchdog_main(rx: std::sync::mpsc::Receiver<WatchdogMsg>) {
    let mut armed: Option<(u64, std::time::Instant, v8::IsolateHandle)> = None;
    loop {
        let wait = match &armed {
            Some((_, deadline, _)) => deadline.saturating_duration_since(std::time::Instant::now()),
            None => std::time::Duration::MAX,
        };
        match rx.recv_timeout(wait) {
            Ok(WatchdogMsg::Arm {
                seq,
                timeout_ms,
                handle,
            }) => {
                armed = Some((
                    seq,
                    std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms),
                    handle,
                ));
            }
            Ok(WatchdogMsg::Disarm { seq }) => {
                if armed.as_ref().is_some_and(|(s, _, _)| *s == seq) {
                    armed = None;
                }
            }
            Ok(WatchdogMsg::Stop) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // 截止到期：终止执行（execute 侧 guard Drop 会按 seq 撤除/清理）。
                if let Some((_, _, handle)) = armed.take() {
                    handle.terminate_execution();
                }
            }
        }
    }
}

/// execute 超时装载守卫：Drop 时按 seq 撤除看门狗。成功与错误路径统一走 Drop
/// （旧实现脚本编译/运行出错提前返回时看门狗线程残留，5s 后误伤下一次执行）。
struct WatchdogGuard {
    tx: std::sync::mpsc::Sender<WatchdogMsg>,
    seq: u64,
}

impl Drop for WatchdogGuard {
    fn drop(&mut self) {
        let _ = self.tx.send(WatchdogMsg::Disarm { seq: self.seq });
    }
}

/// V8脚本沙箱 — 封装一个V8 Isolate和Context，提供安全的脚本执行环境。
///
/// # 生命周期
///
/// 1. 调用 [`V8Sandbox::new()`] 创建沙箱（内部初始化V8平台）
/// 2. 调用 [`V8Sandbox::execute()`] 执行脚本
/// 3. 沙箱释放时自动清理V8资源
///
/// # 性能优化
///
/// 当 `SandboxConfig::persistent_context` 为 `true` 时，首次 execute 创建的
/// V8 Context 会被缓存复用，避免每次执行都重新引导 JS 内置对象，显著降低执行开销。
/// 适用于 WebView 等需要频繁执行脚本的场景。
///
/// # 线程安全
///
/// V8 Isolate不是线程安全的。每个线程应创建独立的沙箱实例。
pub struct V8Sandbox {
    /// 缓存的 V8 Context（当 persistent_context 启用时复用）。
    cached_context: Option<v8::Global<v8::Context>>,
    /// V8 Isolate（拥有所有权）。
    isolate: Option<v8::OwnedIsolate>,
    /// 沙箱配置。
    config: SandboxConfig,
    /// 宿主注入的回调名 + 线程局部注册表索引（register_callback 注册），execute 时挂到全局对象。
    callbacks: Vec<(String, usize)>,
    /// SEC-13 持久超时看门狗（2026-08-10 重构）：常驻线程 + Arm/Disarm seq 协议，
    /// 替代每次 execute 创建/销毁 OS 线程（Windows 并行负载线程 churn 楔死修复）。
    watchdog_tx: Option<std::sync::mpsc::Sender<WatchdogMsg>>,
    watchdog_seq: u64,
    watchdog_join: Option<std::thread::JoinHandle<()>>,
    /// performance-api-compat M2-S1：long task 观察器（None = 未订阅零开销）。
    long_task_observer: Option<crate::LongTaskObserver>,
    /// s88：动态 import() 宿主状态（与 isolate slot 同享 Arc——setter 经此写，
    /// V8 回调经 slot 读）。
    dynamic_import: Arc<std::sync::Mutex<DynamicImportState>>,
}

impl V8Sandbox {
    /// 创建新的V8脚本沙箱。
    ///
    /// 首次调用时会初始化V8平台（全局一次性操作）。
    /// 使用默认配置。
    pub fn new() -> Result<Self, ScriptError> {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用自定义配置创建V8脚本沙箱。
    pub fn with_config(config: SandboxConfig) -> Result<Self, ScriptError> {
        ensure_v8_initialized();

        let mut create_params = v8::Isolate::create_params();
        // 初始堆大小可配置（默认 0 = V8 按系统内存推导）；堆按需增长。上限与
        // 初始堆的合法组合见 v8_heap_limits（V8 要求 initial <= max）。
        if let Some((initial, max)) = crate::v8_heap_limits(&config) {
            create_params = create_params.heap_limits(initial, max);
        }

        let mut isolate = v8::Isolate::new(create_params);
        // R-baidu2/P3 slice-2：未捕获 promise rejection 收集（thread_local，
        // 回调与排空同在 JS 执行线程）。
        isolate.set_promise_reject_callback(promise_reject_callback_entry);

        // s88：动态 import() 宿主回调（classic 上下文）。回调状态经 isolate slot
        //（同线程多 sandbox 隔离）；未 set_dynamic_import_resolver 时回调按 V8
        // 默认语义 reject "Not supported"（行为与今恒等）。
        let dynamic_import = Arc::new(std::sync::Mutex::new(DynamicImportState::default()));
        isolate.set_slot(dynamic_import.clone());
        isolate.set_host_import_module_dynamically_callback(dynamic_import_callback);

        // SEC-13 持久看门狗（2026-08-10）：每 sandbox 一个常驻线程（execute 侧按
        // 当前 timeout_ms Arm），避免每次 execute spawn+join 的线程 churn。
        let (watchdog_tx, watchdog_rx) = std::sync::mpsc::channel();
        let watchdog_join = std::thread::Builder::new()
            .name("v8-sandbox-watchdog".to_string())
            .spawn(move || watchdog_main(watchdog_rx))
            .expect("spawn v8 sandbox watchdog");

        Ok(Self {
            isolate: Some(isolate),
            config,
            cached_context: None,
            callbacks: Vec::new(),
            watchdog_tx: Some(watchdog_tx),
            watchdog_seq: 0,
            watchdog_join: Some(watchdog_join),
            long_task_observer: None,
            dynamic_import,
        })
    }

    /// s88：注册 V8 原生动态 `import()` 宿主解析器（classic 上下文——源码重写仅
    /// 覆盖模块管线；eval 字符串/Function 构造器/事件处理器上下文重写不可达）。
    ///
    /// 契约：`resolver(spec, referrer)` 返回 `resolved\u{1f}iife` / `resolved\u{1f}`
    /// （依赖图编译失败）/ 空串（取回失败）——与 `__zw_compile_module` 宿主回调
    /// 完全一致（webview 侧复用同一闭包实现）。须在 `execute` 之前调用；未注册时
    /// 动态 import() 维持 V8 默认 reject（普通 `Error("Not supported")`）。
    pub fn set_dynamic_import_resolver(&mut self, resolver: DynamicImportResolver) {
        if let Ok(mut st) = self.dynamic_import.lock() {
            st.resolver = Some(resolver);
        }
    }

    /// s88：设定当前执行脚本的 referrer（动态 import() 的 base URL——classic 路径
    /// `v8::Script::compile` 不设 ScriptOrigin，resource_name 缺席，宿主显式供给）。
    /// 每脚本执行前更新（webview 脚本循环取 `current_url`）。
    pub fn set_dynamic_import_referrer(&mut self, url: &str) {
        if let Ok(mut st) = self.dynamic_import.lock() {
            st.referrer = url.to_string();
        }
    }

    /// s90（string-compilation base URL——inline 事件处理器臂）：注册全局函数
    /// `__zwSwapImportReferrer(url)`——JS shim 于 inline/反射事件处理器求值窗调用，
    /// 返回旧 referrer 并换入新值（处理器体内动态 import() 用求值时刻文档基 URL，
    /// finally 换回零外溢）。回调捕获 [`DynamicImportState`] 的 Arc，不经
    /// `&mut self`（JS 执行中沙箱不可变借用不可得——与 resolver 同理由回调面承载）。
    /// 未注册（QuickJS 后端）时 shim 按函数缺席降级为直接执行，零变更。
    pub fn register_dynamic_import_referrer_swapper(&mut self) {
        let dynamic_import = self.dynamic_import.clone();
        self.register_callback(
            "__zwSwapImportReferrer",
            Box::new(move |args| {
                if let Ok(mut st) = dynamic_import.lock() {
                    // 无参调用 = 纯 getter（返回当前值不换）——shim 字符串 timer 注册
                    // 期捕获脚本基（fire 在求值窗外，届时槽位已被后续脚本改写）。
                    return match args.first() {
                        Some(url) => std::mem::replace(&mut st.referrer, url.clone()),
                        None => st.referrer.clone(),
                    };
                }
                String::new()
            }),
        );
    }

    /// s92（string-compilation CSP nonce 面）：设定当前执行脚本的 nonce（动态
    /// `import()` 的 CSP script-src 检查输入——spec 动态 import 继承触发脚本
    /// nonce）。`None` = 无 nonce。每脚本执行前更新（webview 脚本循环取
    /// script_csp_info）。
    pub fn set_dynamic_import_nonce(&mut self, nonce: Option<&str>) {
        if let Ok(mut st) = self.dynamic_import.lock() {
            st.nonce = nonce.map(str::to_string);
        }
    }

    /// s92（string-compilation CSP nonce 面）：注册全局函数
    /// `__zwSwapImportNonce(v)`——与 [`Self::register_dynamic_import_referrer_swapper`]
    /// 同形：有参换入新 nonce 返旧值、无参纯 getter。shim 侧事件处理器求值窗换入
    /// 空串（None——spec 事件处理器不继承触发脚本 nonce，体内 import() 的 module
    /// fetch 无 nonce → 宿主 CSP 检查阻止）、字符串 timer fire 窗换入注册期捕获值
    /// （spec 字符串编译继承触发脚本 nonce）。nonce 空串 ↔ `None` 双射（extract
    /// 侧 nonce 空值视作缺失，合法 nonce 恒非空）。未注册（QuickJS 后端）时 shim
    /// typeof 降级直执行，零变更。
    pub fn register_dynamic_import_nonce_swapper(&mut self) {
        let dynamic_import = self.dynamic_import.clone();
        self.register_callback(
            "__zwSwapImportNonce",
            Box::new(move |args| {
                if let Ok(mut st) = dynamic_import.lock() {
                    return match args.first() {
                        Some(v) => std::mem::replace(&mut st.nonce, if v.is_empty() { None } else { Some(v.clone()) })
                            .unwrap_or_default(),
                        None => st.nonce.clone().unwrap_or_default(),
                    };
                }
                String::new()
            }),
        );
    }

    /// 注册宿主回调，挂为全局函数 `name`（JS-DOM-bridge：JS shim 调 `name(...)` 触发）。
    ///
    /// 回调闭包存入线程局部注册表（返回索引），`execute` 时按索引挂到当前 Context 的
    /// 全局对象。参数按字符串数组传入，返回字符串写入 JS 调用结果。
    /// 无 `register_callback` 调用时行为完全同今（零回归）。须在 `execute` 之前调用。
    #[allow(clippy::type_complexity)]
    pub fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        // 同名重注册 = 替换（最后注册者胜——execute 全量重装时后安装覆盖前者，此处
        // 把该既有可见语义显式化）。宿主每次脚本执行都会全量重注册回调面（webview
        // `execute_dom_script` 每脚本两遍 `register_dom_callbacks`），不去重时
        // `self.callbacks` 与 `HOST_CALLBACKS` 随脚本数无界增长，且 `execute` 每次按
        // 注册史全量重装 O(n)、整页脚本累计 O(n²)（t8p deterministic replay 二次方
        // 退化根因：445 脚本后单脚本 ~1s）。复用**自有**槽位（旧闭包就地 drop；
        // 槽位所有权语义见 HOST_CALLBACKS 注释——他 sandbox 的槽位不可达）；槽位
        // 失效（None 洞/越界，仅防御面——own-slot 清空下活 sandbox 不可达）时退化
        // 为新注册占新槽，不越界不撞写。
        let cb: HostCallback = Arc::from(callback);
        let name_for_trace = name.to_string();
        if let Some(entry) = self.callbacks.iter_mut().find(|(n, _)| n == name) {
            HOST_CALLBACKS.with(|cbs| {
                let mut cbs = cbs.borrow_mut();
                if entry.1 < cbs.len() && cbs[entry.1].is_some() {
                    cbs[entry.1] = Some(cb);
                } else {
                    let idx = cbs.len();
                    cbs.push(Some(cb));
                    entry.1 = idx;
                }
                HOST_CALLBACK_NAMES.with(|names| {
                    let mut names = names.borrow_mut();
                    while names.len() < cbs.len() {
                        names.push(None);
                    }
                    names[entry.1] = Some(name_for_trace);
                });
            });
            return;
        }
        let idx = HOST_CALLBACKS.with(|cbs| {
            let mut cbs = cbs.borrow_mut();
            let idx = cbs.len();
            cbs.push(Some(cb));
            HOST_CALLBACK_NAMES.with(|names| {
                let mut names = names.borrow_mut();
                // P105 定向复核（DEF-1）：pad 到 idx（cb 槽位前一位）再落名，名字对齐
                // 槽位 idx；此前 pad 到 cbs.len()（push 后= idx+1）再 push 落在 idx+1。
                while names.len() < idx {
                    names.push(None);
                }
                names.push(Some(name_for_trace));
            });
            idx
        });
        self.callbacks.push((name.to_string(), idx));
    }

    /// P1b S1 异步回调 resolve（方案 A）。Rust 异步完成后调此方法：执行 JS 全局
    /// `__zwResolveCallback(id, result)`，由 JS 侧 pending 表 resolve 对应 Promise。
    /// `execute` 末尾的 `perform_microtask_checkpoint` 会 drain 随后触发的 `.then`
    /// 微任务，故返回时 Promise 已 resolve。
    ///
    /// **防御**：JS 侧未注入 `__zwResolveCallback`（dom_bridge 未接通）时，外层
    /// `if(globalThis.__zwResolveCallback)` 守卫令本次 execute 的脚本结果为
    /// `undefined`，不报错（零回归）。`id`/`result` 经 [`js_string_literal`] 转义防注入。
    pub fn resolve_async_callback(&mut self, id: &str, result: &str) {
        let id_lit = js_string_literal(id);
        let result_lit = js_string_literal(result);
        let js = format!("if(globalThis.__zwResolveCallback){{__zwResolveCallback({id_lit},{result_lit});}}");
        let _ = self.execute(&js);
    }

    /// t-p103 D2-1 批决议：原生按序逐条 call `__zwResolveCallback`，每条后立即
    /// `perform_microtask_checkpoint`（= 逐次 execute 的逐跳语义，仅省每跳脚本
    /// 编译）；逐条 try/catch 隔离 + 每跳独立看门狗装载。实现注释含 spec 依据与
    /// 返修记录（字符串批方案的 checkpoint 时机缺陷）。
    pub fn resolve_async_callbacks_batch(&mut self, items: &[(String, String)]) {
        if items.is_empty() {
            return;
        }
        // t-p103 D2-1（返修）：原生逐条 call + 每条后 microtask checkpoint——字符串批
        // （单脚本连调 N 次 + execute 末统一 drain）会让后一条 macrotask 级 resolve
        // 抢在前一条的 .then 微任务前执行，违反 HTML 事件循环「task 完成后执行
        // microtask checkpoint 整链排空」语义（event_loop_mixed_async_order_r2953
        // 实测红）。逐条 call + checkpoint 与逐次 execute 逐跳语义精确一致，
        // 仅省每跳脚本编译（风暴 ~2 万跳的收益本体）。
        // https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint
        let persistent = self.config.persistent_context;
        let watchdog_tx = self.watchdog_tx.clone();
        let Some(isolate) = self.isolate.as_mut() else {
            return;
        };
        // SAFETY: 与 execute 一致——OwnedIsolate 创建时已 enter，此处配对
        // IsolateEnterGuard 维持栈式 enter/exit 纪律。
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };
        // thread_safe_handle 须在 scope!（可变借用 isolate）之前取——批内每跳看门狗
        // Arm 都要用，而 scope 存续期无法再借 isolate。
        let thread_safe_handle = isolate.thread_safe_handle();
        let cached_ptr: *mut _ = &mut self.cached_context;
        v8::scope!(let hs, isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 self.isolate 不重叠。
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let try_catch, &mut ctx_scope);
        // P103 返修 DEF-2：首次属性查找也须在看门狗保护内——页面可把 shim 全局
        // 重定义为自旋 getter，无保护的 global.get 会执行任意 JS 且不受终止约束
        // （与逐次 execute 的 arm/disarm 语义对齐）。
        let _lookup_guard: Option<WatchdogGuard> = if self.config.timeout_ms > 0 {
            let seq = self.watchdog_seq;
            self.watchdog_seq += 1;
            watchdog_tx.as_ref().and_then(|tx| {
                tx.send(WatchdogMsg::Arm {
                    seq,
                    timeout_ms: self.config.timeout_ms,
                    handle: thread_safe_handle.clone(),
                })
                .ok()
                .map(|_| WatchdogGuard { tx: tx.clone(), seq })
            })
        } else {
            None
        };
        // shim 未注入 = 整批 no-op（逐次 execute 的外层 if 守卫等价）。
        let Some(key) = v8::String::new(try_catch, "__zwResolveCallback") else {
            return;
        };
        let Some(cb_val) = context.global(try_catch).get(try_catch, key.into()) else {
            // 查找被终止/抛错（DEF-2 getter 场景）：取走异常不留滞留态。
            if try_catch.has_caught() {
                try_catch.reset();
            }
            return;
        };
        drop(_lookup_guard);
        let Ok(cb) = v8::Local::<v8::Function>::try_from(cb_val) else {
            return;
        };
        for (id, result) in items {
            // 每跳独立看门狗装载（与逐次 execute 的 arm/disarm 语义一致；
            // Drop 统一撤除，早退无残留）。
            let _timeout_guard: Option<WatchdogGuard> = if self.config.timeout_ms > 0 {
                let seq = self.watchdog_seq;
                self.watchdog_seq += 1;
                if let Some(tx) = &watchdog_tx {
                    let _ = tx.send(WatchdogMsg::Arm {
                        seq,
                        timeout_ms: self.config.timeout_ms,
                        handle: thread_safe_handle.clone(),
                    });
                    Some(WatchdogGuard { tx: tx.clone(), seq })
                } else {
                    None
                }
            } else {
                None
            };
            let (Some(id_s), Some(r_s)) = (v8::String::new(try_catch, id), v8::String::new(try_catch, result)) else {
                continue;
            };
            let undef = v8::undefined(try_catch);
            // 逐条隔离：单条 call 抛错（含超时终止异常）不阻断同批其余条目。
            let _ = cb.call(try_catch, undef.into(), &[id_s.into(), r_s.into()]);
            // 本跳的微任务整链排空后才处理下一跳（spec checkpoint 语义）。
            try_catch.perform_microtask_checkpoint();
            if try_catch.has_caught() {
                try_catch.reset();
            }
        }
    }

    /// 设置脚本执行超时（毫秒）；0 表示无超时。
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// performance-api-compat M2-S1：注册/撤销 long task 观察器（trait 委托实现）。
    pub fn set_long_task_observer(&mut self, observer: Option<crate::LongTaskObserver>) {
        self.long_task_observer = observer;
    }

    /// execute 成功返回前的 long task 通知（≥ [`crate::LONG_TASK_THRESHOLD_MS`]）。
    /// 观察器在 execute 返回前同步调用——不得重入沙箱（webview 侧排空模式）。
    /// 关联函数形态：`hs` scope 存续期持有 `self` 的可变借用（isolate.as_mut），
    /// 观察器须以局部 clone 传入（绕开字段借用冲突）。
    fn fire_long_task(observer: &Option<crate::LongTaskObserver>, execution_time_ms: f64) {
        if execution_time_ms >= crate::LONG_TASK_THRESHOLD_MS
            && let Some(observer) = observer
        {
            observer(execution_time_ms);
        }
    }

    /// 在沙箱中执行JavaScript脚本。
    ///
    /// # 参数
    ///
    /// - `code` — JavaScript源代码
    ///
    /// # 返回
    ///
    /// 返回脚本执行结果，包含返回值的字符串表示和执行耗时。
    pub fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        if code.trim().is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        // long task 观察器以局部 clone 持有（execute 体内 self 被 hs scope 可变借用）。
        let long_task_observer = self.long_task_observer.clone();
        let persistent = self.config.persistent_context;
        // SAFETY: cached_context 和 isolate 是不同的字段。
        // HandleScope 持有 isolate 的借用，不会触及 cached_context。
        // 原始指针允许我们在 isolate 被借用时访问 cached_context。
        let cached_ptr: *mut _ = &mut self.cached_context;

        let isolate = self.isolate.as_mut().ok_or(ScriptError::NotInitialized)?;
        // SAFETY: OwnedIsolate instances are entered on creation. Re-entering
        // around each execution makes multiple sandbox instances usable on the
        // same thread while preserving V8's stack-like enter/exit discipline.
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };

        let start = std::time::Instant::now();

        // SEC-13: 强制执行 timeout——持久看门狗 Arm/Disarm（2026-08-10 重构：
        // 旧实现每次 execute spawn+join 一个 OS 线程，Windows 并行负载下线程
        // churn 楔死；guard Drop 统一撤除，顺带消除脚本出错提前返回时看门狗
        // 线程残留 5s 后 terminate_execution 误伤下一次执行的 latent bug）。
        let _timeout_guard: Option<WatchdogGuard> = if self.config.timeout_ms > 0 {
            let seq = self.watchdog_seq;
            self.watchdog_seq += 1;
            if let Some(tx) = &self.watchdog_tx {
                let _ = tx.send(WatchdogMsg::Arm {
                    seq,
                    timeout_ms: self.config.timeout_ms,
                    handle: isolate.thread_safe_handle(),
                });
                Some(WatchdogGuard { tx: tx.clone(), seq })
            } else {
                None
            }
        } else {
            None
        };

        v8::scope!(let hs, isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 self.isolate 不重叠。
        // HandleScope 的借用仅涉及 isolate，不会修改 cached_context。
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let try_catch, &mut ctx_scope);

        // 把宿主回调（register_callback 注册）挂到全局对象。无注册时为 no-op（零回归）。
        if !self.callbacks.is_empty() {
            let global = context.global(try_catch);
            for (name, idx) in &self.callbacks {
                let data = v8::Integer::new(try_catch, *idx as i32);
                let tmpl = v8::FunctionTemplate::builder(host_callback_invoke)
                    .data(data.into())
                    .build(try_catch);
                let Some(function) = tmpl.get_function(try_catch) else {
                    continue;
                };
                if let Some(key) = v8::String::new(try_catch, name) {
                    let _ = global.set(try_catch, key.into(), function.into());
                }
            }
        }

        // 编译脚本
        let code_str = v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = v8::Script::compile(try_catch, code_str, None);
        if try_catch.has_caught() || script.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::CompileError(msg));
        }
        let script = script.unwrap();

        // 执行脚本
        let result = script.run(try_catch);
        if try_catch.has_caught() || result.is_none() {
            // SEC-13：超时终止（看门狗 terminate_execution）——V8 终止异常在
            // TryCatch 表现为 caught 的 "null" 异常，须先于 RuntimeError 判定，
            // 报告为 Timeout（2026-08-10 持久看门狗重构时校正）。
            if try_catch.has_terminated() {
                try_catch.cancel_terminate_execution();
                return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
            }
            if result.is_none() && !try_catch.has_caught() {
                try_catch.cancel_terminate_execution();
                return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
            }
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }
        let result = result.unwrap();

        try_catch.perform_microtask_checkpoint();
        if try_catch.has_caught() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }

        // 转换结果为字符串
        let result_str = result
            .to_string(try_catch)
            .map(|s| s.to_rust_string_lossy(try_catch))
            .unwrap_or_default();

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // 清理 timeout guard（Drop → Disarm 撤除看门狗；成功与错误路径统一）
        drop(_timeout_guard);
        Self::fire_long_task(&long_task_observer, execution_time_ms);

        Ok(ScriptResult {
            value: result_str,
            execution_time_ms,
        })
    }

    /// 在沙箱中执行JavaScript脚本并返回JSON字符串。
    pub fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        if code.trim().is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        // long task 观察器以局部 clone 持有（同 [`Self::execute`] 借用注记）。
        let long_task_observer = self.long_task_observer.clone();

        let persistent = self.config.persistent_context;
        // SAFETY: 同 execute() 中的说明
        let cached_ptr: *mut _ = &mut self.cached_context;

        let isolate = self.isolate.as_mut().ok_or(ScriptError::NotInitialized)?;
        // SAFETY: See execute().
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };

        let start = std::time::Instant::now();

        v8::scope!(let hs, isolate);
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let try_catch, &mut ctx_scope);

        let code_str = v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = v8::Script::compile(try_catch, code_str, None);
        if try_catch.has_caught() || script.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::CompileError(msg));
        }
        let script = script.unwrap();

        let result = script.run(try_catch);
        if try_catch.has_caught() || result.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }
        let result = result.unwrap();

        try_catch.perform_microtask_checkpoint();
        if try_catch.has_caught() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }

        // 尝试JSON.stringify
        let json_str = Self::value_to_json_string(try_catch, result);

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        Self::fire_long_task(&long_task_observer, execution_time_ms);

        Ok(ScriptResult {
            value: json_str,
            execution_time_ms,
        })
    }
}

/// 获取或创建 V8 Context。
///
/// 当 `persistent` 为 true 时，首次创建的 Context 会被缓存到 `cached` 中复用。
/// 否则每次调用都创建新的 Context（保证执行间状态隔离）。
///
/// # Safety
///
/// `cached_ptr` 必须指向一个有效的、不与 `scope` 所借用的 isolate 重叠的
/// `Option<Global<Context>>`。调用方需确保两个引用不冲突。
unsafe fn resolve_context<'s>(
    persistent: bool,
    cached_ptr: *mut Option<v8::Global<v8::Context>>,
    scope: &mut v8::PinScope<'s, '_, ()>,
) -> v8::Local<'s, v8::Context> {
    let cached = unsafe { &mut *cached_ptr };
    if !persistent {
        return v8::Context::new(scope, Default::default());
    }

    // 尝试复用缓存的 Context
    if let Some(ref cached_ctx) = *cached {
        return v8::Local::new(scope, cached_ctx);
    }

    // 首次执行：创建并缓存 Context
    let context = v8::Context::new(scope, Default::default());
    *cached = Some(v8::Global::new(scope, context));
    context
}

impl V8Sandbox {
    /// 重置缓存的 V8 Context。
    ///
    /// 下次 execute 会创建新的 Context。仅在 `persistent_context` 模式下有意义。
    ///
    /// T13 R1：S3 的 `low_memory_notification` 移至 [`Self::low_memory_reclaim`]
    /// （延迟到点，isolate 线程命令边界执行）——复位臂自身只做 context 丢弃，成本
    /// 移出首载测窗（处置工单主控 M1；窗口证据见 bench/results/t13-rework-frozen.md）。
    pub fn reset_context(&mut self) {
        self.cached_context = None;
    }

    /// T13 S3/R1：对 isolate 发 `low_memory_notification`——`cached_context = None`
    /// 只 drop Context 引用，旧 Context 成 isolate 内垃圾，而持久 isolate 永不销毁、
    /// 复位处此前无任何 GC 触发点，垃圾滞留 V8 堆 + glibc arena（C1→C2 增长主嫌疑，
    /// 设计卡 §1-#3）。宿主 GC hint 无需 `--expose-gc` 全局 flag（同
    /// [`crate::dom_bindings`] weak-handle 先例），只对已 drop 的旧 Context 垃圾生效，
    /// 新文档语义零接触。须在 isolate 所属线程调用（js worker 命令边界满足）。
    pub fn low_memory_reclaim(&mut self) {
        let Some(isolate) = self.isolate.as_mut() else {
            return;
        };
        // SAFETY: 与 execute 一致——OwnedIsolate 创建时已 enter，此处配对
        // IsolateEnterGuard 维持栈式 enter/exit 纪律。
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };
        isolate.low_memory_notification();
    }

    /// T13 S7 探针：isolate 已用堆字节（`ZEROWEB_MEM_PROBE=1` 观测腿，不进判据样本）。
    pub fn heap_used_bytes(&mut self) -> Option<usize> {
        let isolate = self.isolate.as_mut()?;
        // SAFETY: 同 reset_context——栈式 enter/exit 配对。
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };
        Some(isolate.get_heap_statistics().used_heap_size())
    }

    /// P1b S2 escape-hatch：进入持久 V8 Context，在 raw scope + context 内执行 `f`。
    ///
    /// 供 [`Sandbox::install_native_bindings`] 安装原生 DOM 绑定（`ObjectTemplate`/
    /// `FunctionTemplate`/accessor）——与 `execute` 共享同一持久 Context + scope setup
    /// （isolate.enter + `scope!` + `resolve_context` + `ContextScope`），故 `f` 安装的
    /// 全局对象/模板对后续 `execute` 可见。无 isolate / 未初始化时返 `None`。
    ///
    /// 镜像 [`V8Sandbox::execute`] 的 scope 进入（含 `IsolateEnterGuard` 的 enter/exit 配对）。
    fn with_context<R>(&mut self, f: impl FnOnce(&mut v8::PinScope, v8::Local<v8::Context>) -> R) -> Option<R> {
        let persistent = self.config.persistent_context;
        // SAFETY: 同 execute()——cached_context 与 isolate 为不同字段；raw ptr 借用
        // cached_context 不与 HandleScope 的 isolate 借用重叠。
        let cached_ptr: *mut _ = &mut self.cached_context;
        let isolate = self.isolate.as_mut()?;
        // SAFETY: enter/exit 配对（IsolateEnterGuard drop 时 exit），维持 V8 栈式纪律。
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };
        v8::scope!(let hs, isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 isolate 不重叠。
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let scope, &mut ctx_scope);
        Some(f(scope, context))
    }

    /// 获取V8引擎版本号。
    pub fn v8_version() -> &'static str {
        ensure_v8_initialized();
        v8::V8::get_version()
    }

    /// 将V8值转换为JSON字符串。
    fn value_to_json_string(
        scope: &mut v8::PinnedRef<v8::TryCatch<v8::HandleScope>>,
        value: v8::Local<v8::Value>,
    ) -> String {
        let context = scope.get_current_context();
        let global = context.global(scope);

        let json_key = v8::String::new(scope, "JSON").unwrap();
        let json_val = global.get(scope, json_key.into());

        let Some(json_val) = json_val else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        if json_val.is_undefined() {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        }

        let Some(json_obj) = json_val.to_object(scope) else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        let stringify_key = v8::String::new(scope, "stringify").unwrap();
        let stringify_val = json_obj.get(scope, stringify_key.into());

        let Some(stringify_val) = stringify_val else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        if !stringify_val.is_function() {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        }

        let Ok(stringify_fn) = v8::Local::<v8::Function>::try_from(stringify_val) else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        let args = [value];
        let result = stringify_fn.call(scope, json_obj.into(), &args);

        result
            .and_then(|v| v.to_string(scope))
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| {
                value
                    .to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_default()
            })
    }
}

impl Drop for V8Sandbox {
    fn drop(&mut self) {
        // 2026-08-10 windows tab_js_worker 挂起修复：提前在本线程 drop 线程局部
        // 注册表中**本 sandbox 自有**的回调（含 reqwest HttpClient 等带后台线程的
        // 资源——避免其留到线程退出时在 Windows loader lock 下 drop 死锁，见
        // [`clear_host_callbacks`]）。
        //
        // t8p 审查 D1/D2 起改 own-slot 清空（此前整表 `clear_host_callbacks()`）：
        // 同线程多 sandbox 并存时整表清空会 (1) 悬空幸存 sandbox 记录的槽位索引
        // （重注册越界 panic 或撞写他人活槽位），(2) 杀死幸存者回调使其 invoke
        // 静默退回 ""。own-slot 清空后幸存者不受影响；置 None 的洞不复用（无
        // free-list），表长上界 = 线程历史不同名字注册总数。整线程拆卸场景仍可
        // 显式调用 [`clear_host_callbacks`]。
        let slots: Vec<usize> = self.callbacks.iter().map(|(_, slot)| *slot).collect();
        HOST_CALLBACKS.with(|cbs| {
            let mut cbs = cbs.borrow_mut();
            for slot in &slots {
                if *slot < cbs.len() {
                    cbs[*slot] = None;
                }
            }
            HOST_CALLBACK_NAMES.with(|names| {
                let mut names = names.borrow_mut();
                for slot in &slots {
                    if *slot < names.len() {
                        names[*slot] = None;
                    }
                }
            });
        });
        // 未处理 rejection 撤回暂存持有本 isolate 的 `Global<Promise>` 句柄；
        // 若 drop 前未排空（异常路径/测试直落），残留句柄会在同线程下一个
        // sandbox 的事件比较中触碰已销毁 isolate（"Handle hosted by disposed
        // Isolate" panic，extern "C" 回调内不可 unwind → abort）。排空即清 +
        // drop 即清双保险。
        PENDING_REJECTS.with(|p| p.borrow_mut().clear());
        if let Some(tx) = &self.watchdog_tx {
            let _ = tx.send(WatchdogMsg::Stop);
        }
        if let Some(join) = self.watchdog_join.take() {
            let _ = join.join();
        }
        self.cached_context = None;
        self.isolate = None;
    }
}

impl crate::Sandbox for V8Sandbox {
    fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        V8Sandbox::execute(self, code)
    }
    fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        V8Sandbox::execute_json(self, code)
    }
    fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        V8Sandbox::register_callback(self, name, callback)
    }
    fn resolve_async_callback(&mut self, id: &str, result: &str) {
        V8Sandbox::resolve_async_callback(self, id, result);
    }
    fn resolve_async_callbacks_batch(&mut self, items: &[(String, String)]) {
        V8Sandbox::resolve_async_callbacks_batch(self, items);
    }
    fn set_timeout_ms(&mut self, timeout_ms: u64) {
        V8Sandbox::set_timeout_ms(self, timeout_ms)
    }
    fn set_long_task_observer(&mut self, observer: Option<crate::LongTaskObserver>) {
        V8Sandbox::set_long_task_observer(self, observer)
    }
    fn set_dynamic_import_resolver(&mut self, resolver: DynamicImportResolver) {
        V8Sandbox::set_dynamic_import_resolver(self, resolver)
    }
    fn set_dynamic_import_referrer(&mut self, url: &str) {
        V8Sandbox::set_dynamic_import_referrer(self, url)
    }
    fn register_dynamic_import_referrer_swapper(&mut self) {
        V8Sandbox::register_dynamic_import_referrer_swapper(self)
    }
    fn set_dynamic_import_nonce(&mut self, nonce: Option<&str>) {
        V8Sandbox::set_dynamic_import_nonce(self, nonce)
    }
    fn register_dynamic_import_nonce_swapper(&mut self) {
        V8Sandbox::register_dynamic_import_nonce_swapper(self)
    }
    fn reset_context(&mut self) {
        V8Sandbox::reset_context(self)
    }
    fn low_memory_reclaim(&mut self) {
        V8Sandbox::low_memory_reclaim(self)
    }
    fn heap_used_bytes(&mut self) -> Option<usize> {
        V8Sandbox::heap_used_bytes(self)
    }
    /// R-baidu2/P3 slice-2：取走未捕获异常报告（message listener + promise-reject
    /// 回调经 isolate slot 收集）。
    fn take_uncaught_reports(&mut self) -> Vec<(String, u32, u32)> {
        // 排空即过检查点：撤回暂存一并清（此后才挂 handler 的 rejection 照常上报）。
        PENDING_REJECTS.with(|p| p.borrow_mut().clear());
        UNCAUGHT_REPORTS.with(|q| std::mem::take(&mut *q.borrow_mut()).into_iter().flatten().collect())
    }

    fn config(&self) -> &SandboxConfig {
        &self.config
    }
    #[allow(clippy::type_complexity)] // escape-hatch 闭包类型（镜像 register_callback 模式）
    fn install_native_bindings(
        &mut self,
        installer: Box<dyn FnOnce(&mut v8::PinScope, v8::Local<v8::Context>)>,
    ) -> bool {
        self.with_context(|scope, ctx| installer(scope, ctx)).is_some()
    }
}

impl Default for V8Sandbox {
    fn default() -> Self {
        Self::new().expect("V8 sandbox creation should not fail")
    }
}

impl std::fmt::Debug for V8Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V8Sandbox")
            .field("config", &self.config)
            .field("initialized", &self.isolate.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 创建与初始化 ──

    /// T13 S3：reset_context 尾 `low_memory_notification` 后 context 重建语义不破——
    /// 持久 isolate 下旧全局随 Context 丢弃，新 execute 拿到全新全局对象（导航复位
    /// 语义），通知调用不悬 isolate、不污染后续执行。
    #[test]
    fn t13_reset_context_low_memory_notification_keeps_fresh_global() {
        let mut sandbox = V8Sandbox::with_config(crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox
            .execute("globalThis.__doc = 'old'; 1")
            .expect("first document execute");
        sandbox.reset_context();
        let r = sandbox
            .execute("typeof globalThis.__doc")
            .expect("execute after reset must recreate context");
        assert_eq!(r.value, "undefined", "旧全局随 Context 丢弃: {:?}", r.value);
        // 复位后再次写入 + 读回（新 Context 可用性）。
        let r = sandbox
            .execute("globalThis.__doc = 'new'; globalThis.__doc")
            .expect("write after reset");
        assert_eq!(r.value, "new", "新全局可写: {:?}", r.value);
    }

    #[test]
    fn test_unhandled_promise_rejection_collected() {
        // R-baidu2/P3 slice-2：未处理 rejection 经 promise-reject 回调收集。
        // 诊断保真切片（2026-09-29）：原因文本（Chrome console 形态）+ `.stack`
        // 创建时符号化栈帧；行/列保持 0（上报点 JS 栈已空，见回调注释）。
        use crate::Sandbox;
        let mut sandbox = V8Sandbox::new().unwrap();
        let r = sandbox
            .execute("Promise.reject(new Error('boom')); 42")
            .expect("execute should succeed (rejection is async-surface)");
        assert!(r.value.contains("42"), "tail expression: {:?}", r.value);
        let reports = sandbox.take_uncaught_reports();
        assert_eq!(reports.len(), 1, "one unhandled rejection: {:?}", reports);
        assert!(
            reports[0].0.starts_with("Uncaught (in promise) Error: boom"),
            "reason text (Chrome console form): {:?}",
            reports[0].0
        );
        assert!(
            reports[0].0.contains("\n    at "),
            "symbolized creation stack frames: {:?}",
            reports[0].0
        );
        assert_eq!((reports[0].1, reports[0].2), (0, 0));
        // 排空后为空（幂等）。
        assert!(sandbox.take_uncaught_reports().is_empty());
    }

    #[test]
    fn test_unhandled_promise_rejection_non_error_reason() {
        // 非 Error 原因（字符串）无 `.stack` → 原因文本照报、行/列保持 0
        //（事件级信息下限，不因原因类型降级为丢报）。
        use crate::Sandbox;
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox
            .execute("Promise.reject('str-bang'); 7")
            .expect("execute should succeed (rejection is async-surface)");
        let reports = sandbox.take_uncaught_reports();
        assert_eq!(reports.len(), 1, "one unhandled rejection: {:?}", reports);
        assert_eq!(reports[0].0, "Uncaught (in promise) str-bang");
        assert_eq!((reports[0].1, reports[0].2), (0, 0));
    }

    #[test]
    fn test_promise_reject_event_filter_and_ordering() {
        // 事件过滤 + 多 rejection 顺序。V8 时序：NoHandler 在 reject 即时触发
        // （reject 点 has_handler() 已定），先于同脚本内 .catch 挂接。撤回机制
        // 覆盖同 execute 场景（见 retraction 测试）；本测试钉的是跨 execute 边界：
        // ① HandlerAdded 事件不得产生第二条报告（防双报守卫核心面）；
        // ② 跨 execute 才挂 handler → 报告已在上一轮排空，不重复不追报；
        // ③ 同一 execute 内多个未处理 rejection 按创建顺序各报一条。
        use crate::Sandbox;
        let mut sandbox = V8Sandbox::with_config(crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox
            .execute("globalThis.__p = Promise.reject('r'); 0")
            .expect("execute should succeed");
        assert_eq!(
            sandbox.take_uncaught_reports().len(),
            1,
            "reject 即时上报（V8 NoHandler 时序，先于 .catch）"
        );
        sandbox
            .execute("globalThis.__p.catch(function () {}); 0")
            .expect("execute should succeed");
        let reports = sandbox.take_uncaught_reports();
        assert!(reports.is_empty(), "HandlerAdded 不得追加上报: {:?}", reports);
        sandbox
            .execute("Promise.reject('first'); Promise.reject('second'); 0")
            .expect("execute should succeed (rejections are async-surface)");
        let reports = sandbox.take_uncaught_reports();
        assert_eq!(reports.len(), 2, "two unhandled rejections: {:?}", reports);
        assert_eq!(reports[0].0, "Uncaught (in promise) first");
        assert_eq!(reports[1].0, "Uncaught (in promise) second");
    }

    #[test]
    fn test_promise_reject_same_execute_handler_retracts_report() {
        // 撤回语义（about-to-be-notified retraction，R-baidu7）：同 execute 内
        // reject 后挂上 handler（.catch / .then 任一 reaction）→ 撤回不上报
        //（Chrome 报 0）。baidu live 根因：R56h 运行时 img 解码管线
        // createImageBitmap(blob) reject 后同步 .then(onF, onR) 派 error 事件——
        // 撤回机制前被误报 3 条 uncaught rejection。
        use crate::Sandbox;
        let mut sandbox = V8Sandbox::new().unwrap();
        // 同步挂 .catch 具名 handler。
        sandbox
            .execute("Promise.reject('handled-catch').catch(function () {}); 0")
            .expect("execute should succeed");
        assert!(
            sandbox.take_uncaught_reports().is_empty(),
            "同 execute 挂 .catch → 撤回，不报"
        );
        // .then 仅 onFulfilled（无 rejection reaction）→ 不撤回。V8 粒度：仅
        // rejection reaction 挂接才发 HandlerAdded——与 Chrome 可观察行为一致：
        // rejected promise 上仅挂 fulfillment reaction 时派生 promise 采用
        // rejection，其自身 NoHandler 照报（同因文本）。
        sandbox
            .execute("Promise.reject('handled-then').then(function () {}); 0")
            .expect("execute should succeed");
        let reports = sandbox.take_uncaught_reports();
        assert_eq!(
            reports.len(),
            1,
            "仅 fulfillment reaction → 派生 promise rejection 照报: {:?}",
            reports
        );
        assert_eq!(reports[0].0, "Uncaught (in promise) handled-then");
        // 微任务内才挂 handler（execute 末 perform_microtask_checkpoint 先于
        // 排空）→ 撤回。
        sandbox
            .execute(
                "var p = Promise.reject('handled-late'); \
                 Promise.resolve().then(function () { p.catch(function () {}); }); 0",
            )
            .expect("execute should succeed");
        assert!(
            sandbox.take_uncaught_reports().is_empty(),
            "同 execute 微任务内挂 handler → 撤回，不报"
        );
        // 对照组：未挂 handler 照报（防撤回过宽）。
        sandbox
            .execute("Promise.reject('still-unhandled'); 0")
            .expect("execute should succeed");
        let reports = sandbox.take_uncaught_reports();
        assert_eq!(reports.len(), 1, "未挂 handler 照报: {:?}", reports);
        assert_eq!(reports[0].0, "Uncaught (in promise) still-unhandled");
    }

    #[test]
    fn test_promise_reject_pending_cleared_on_sandbox_drop() {
        // R-baidu7 回归（wpt-runner storage_cases_execute_scripts_r3081 abort）：
        // sandbox 1 未排空即 drop，其撤回暂存持有指向已销毁 isolate 的
        // `Global<Promise>` 句柄；同线程 sandbox 2 的 HandlerAdded 事件比较触碰
        // 死句柄 → "Handle hosted by disposed Isolate" panic（extern "C" 回调
        // 内不可 unwind → abort）。sandbox Drop 即清暂存必须兜住，且 sandbox 2
        // 的同 execute 撤回照常生效。
        use crate::Sandbox;
        {
            let mut s1 = V8Sandbox::new().unwrap();
            s1.execute("Promise.reject('dangling'); 0")
                .expect("execute should succeed");
            // 故意不排空报告——模拟异常路径（报告队列经 thread_local 存续，
            // 字符串无 isolate 句柄，泄漏属既有行为，此处不断言）。
        }
        let mut s2 = V8Sandbox::new().unwrap();
        s2.execute("Promise.reject('fresh').catch(function () {}); 0")
            .expect("execute should succeed (no disposed-isolate panic)");
        let reports = s2.take_uncaught_reports();
        assert!(
            !reports.iter().any(|r| r.0.contains("fresh")),
            "sandbox 2 同 execute 撤回照常生效: {:?}",
            reports
        );
    }

    #[test]
    fn test_sandbox_new() {
        let sandbox = V8Sandbox::new();
        assert!(sandbox.is_ok(), "V8 sandbox creation should succeed");
    }

    #[test]
    fn test_sandbox_default() {
        let sandbox = V8Sandbox::default();
        let _ = &sandbox;
    }

    #[test]
    fn test_sandbox_debug_format() {
        let sandbox = V8Sandbox::new().unwrap();
        let debug = format!("{sandbox:?}");
        assert!(debug.contains("V8Sandbox"));
        assert!(debug.contains("initialized"));
    }

    #[test]
    fn test_sandbox_with_config() {
        let config = SandboxConfig {
            heap_limit: 32 * 1024 * 1024, // 32MB
            timeout_ms: 5000,
            persistent_context: false,
            ..Default::default()
        };
        let sandbox = V8Sandbox::with_config(config);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.heap_limit, 0);
        assert_eq!(config.timeout_ms, 0);
        assert!(!config.persistent_context, "默认应使用每次创建新 Context");
    }

    #[test]
    fn test_v8_version() {
        let version = V8Sandbox::v8_version();
        assert!(!version.is_empty(), "V8 version should not be empty");
        assert!(version.contains('.'), "Version should contain dots");
    }

    // ── 脚本执行：正常路径 ──

    #[test]
    fn test_execute_simple_expression() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("1 + 1").unwrap();
        assert_eq!(result.value, "2");
        assert!(result.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_string_concat() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'Hello' + ' ' + 'World'").unwrap();
        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn test_execute_variable_declaration() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("var x = 42; x").unwrap();
        assert_eq!(result.value, "42");
    }

    // ── 宿主回调（register_callback，JS-DOM-bridge Phase 0）──

    #[test]
    fn test_register_callback_invoked_from_js() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.register_callback(
            "__zw_test",
            Box::new(|args| format!("echo:{}:{}", args.len(), args.join("|"))),
        );
        // JS 调用宿主回调，返回值成为脚本结果。
        let result = sandbox.execute("__zw_test('a', 'bb')").unwrap();
        assert_eq!(result.value, "echo:2:a|bb");
    }

    #[test]
    fn test_register_callback_persistent_context() {
        // persistent_context=true 时回调仍须在缓存的 Context 上生效。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox.register_callback("__zw_greet", Box::new(|args| format!("hi {}", args[0])));
        let r1 = sandbox.execute("__zw_greet('world')").unwrap();
        assert_eq!(r1.value, "hi world");
        // 第二次 execute 复用缓存 Context，回调仍可用。
        let r2 = sandbox.execute("__zw_greet('again')").unwrap();
        assert_eq!(r2.value, "hi again");
    }

    // ── t8p（webview deterministic replay 二次方退化根因）：同名重注册须替换不累积 ──
    // 宿主每次脚本执行都全量重注册回调面（webview execute_dom_script 每脚本两遍
    // register_dom_callbacks）；不去重时 `callbacks` 与 `HOST_CALLBACKS` 随脚本数无界
    // 增长，`execute` 每次按注册史全量重装 O(n)、整页脚本累计 O(n²)（445 脚本后单
    // 脚本 ~1s，replay 测试挂死）。修前两表长度随注册次数线性增长，本钉红态可判别。
    // HOST_CALLBACKS 为 thread-local（同线程跨测试共享），断言用注册前后差值。

    #[test]
    fn test_register_callback_same_name_replaces_not_accumulates() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let host_before = HOST_CALLBACKS.with(|cbs| cbs.borrow().len());
        sandbox.register_callback("__zw_pin", Box::new(|_| "first".to_string()));
        // 同名重注册 50 次 + 一个不同名回调：两表都不得累积。
        for i in 0..50 {
            let tag = format!("v{i}");
            sandbox.register_callback("__zw_pin", Box::new(move |_| tag.clone()));
        }
        sandbox.register_callback("__zw_other", Box::new(|_| "other".to_string()));
        assert_eq!(sandbox.callbacks.len(), 2, "同名重注册去重（修前 51+）");
        let host_after = HOST_CALLBACKS.with(|cbs| cbs.borrow().len());
        assert_eq!(host_after - host_before, 2, "HOST_CALLBACKS 槽位复用（修前 51+）");
        // 最后注册者胜 + 其他回调不受影响。
        let latest = sandbox.execute("__zw_pin()").unwrap();
        assert_eq!(latest.value, "v49");
        let other = sandbox.execute("__zw_other()").unwrap();
        assert_eq!(other.value, "other");
    }

    // ── t8p 审查 D1/D2（own-slot 清空与槽位所有权）：同线程多 sandbox 场景 ──
    // HOST_CALLBACKS 为 thread-local 且 sandbox Drop 会清理自有槽位；同线程并存、
    // 生存期重叠的 sandbox 组合下，旧实现（Drop 整表清空 + 重注册盲写槽位）有
    // panic（D1）与跨实例静默替换（D2）两面。红态各钉一处。

    #[test]
    fn test_register_callback_survives_foreign_sandbox_drop_t8p() {
        // D1：a 注册后，b 注册同名（占新槽），b drop（旧实现整表清空 → a 的槽位
        // 记录失效），a 重注册同名——旧实现盲写已清空的槽位越界 panic。
        let mut a = V8Sandbox::new().unwrap();
        a.register_callback("__zw_d1", Box::new(|_| "a1".to_string()));
        let mut b = V8Sandbox::new().unwrap();
        b.register_callback("__zw_d1", Box::new(|_| "b1".to_string()));
        drop(b);
        // 修前红态：此处 panic（index out of bounds）。
        a.register_callback("__zw_d1", Box::new(|_| "a2".to_string()));
        let r = a.execute("__zw_d1()").unwrap();
        assert_eq!(r.value, "a2");
    }

    #[test]
    fn test_drop_does_not_clobber_survivor_slots_t8p() {
        // D2：幸存者 b 的槽位记录在「整表清空 + 他人重填」后指向他人活槽位，
        // b 重注册同名会静默替换他人回调。（V8 isolate 须按创建逆序 drop，
        // 故 b 先创建——它才是持陈旧槽位索引的幸存者。）
        let mut b = V8Sandbox::new().unwrap();
        b.register_callback("__zw_b", Box::new(|_| "b-old".to_string()));
        let mut a = V8Sandbox::new().unwrap();
        a.register_callback("__zw_a", Box::new(|_| "a".to_string()));
        drop(a); // 修前整表清空 → b 的槽位记录（0）失效；t8p 修后仅清 a 自有槽位
        let mut c = V8Sandbox::new().unwrap();
        c.register_callback("__zw_c", Box::new(|_| "c".to_string()));
        // 修前红态：c 重填槽位 0（b 的陈旧索引），此调用盲写 cbs[0] 静默替换
        // c 的回调。
        b.register_callback("__zw_b", Box::new(|_| "b-new".to_string()));
        let c_r = c.execute("__zw_c()").unwrap();
        assert_eq!(c_r.value, "c", "他实例重注册不得撞写本实例回调（D2）");
        let b_r = b.execute("__zw_b()").unwrap();
        assert_eq!(b_r.value, "b-new");
    }

    #[test]
    fn test_survivor_callback_invokable_after_foreign_drop_t8p() {
        // D2 伴随面：空 sandbox drop（旧实现也整表清空）不得杀死幸存者的回调。
        let mut a = V8Sandbox::new().unwrap();
        a.register_callback("__zw_alive", Box::new(|_| "alive".to_string()));
        let b = V8Sandbox::new().unwrap();
        drop(b);
        // 修前红态：整表清空后 invoke 查表落空，静默退回 ""。
        let r = a.execute("__zw_alive()").unwrap();
        assert_eq!(r.value, "alive");
    }

    #[test]
    fn test_reregister_does_not_disturb_other_name_slot_t8p() {
        // 审查① G2 加固：替换实现若写错槽（如写 last 而非按名命中的槽），先注册
        // 的异名回调会被串扰——B 先注册占槽，重注册 A ×N 后 B 须仍返 B 值。
        let mut s = V8Sandbox::new().unwrap();
        s.register_callback("__zw_g2a", Box::new(|_| "A1".to_string()));
        s.register_callback("__zw_g2b", Box::new(|_| "B".to_string()));
        for i in 0..10 {
            let tag = format!("A2-{i}");
            s.register_callback("__zw_g2a", Box::new(move |_| tag.clone()));
        }
        let b = s.execute("__zw_g2b()").unwrap();
        assert_eq!(b.value, "B", "同名重注册不得串扰异名槽位");
        let a = s.execute("__zw_g2a()").unwrap();
        assert_eq!(a.value, "A2-9");
    }

    #[test]
    fn test_persistent_captured_callback_reference_latest_t8p() {
        // 审查① G3 语义显式化钉（persistent context）：页面捕获的旧函数对象跨
        // 重注册解析到**当前**注册（FunctionTemplate 携带槽位索引，invoke 查表
        // 命中替换后的闭包）。注意引擎分歧（审查② N1）：QuickJS 旧函数对象持
        // 旧闭包 Arc 恒调旧值，本钉仅钉 V8 共享表语义，不作跨引擎一致断言。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut s = V8Sandbox::with_config(config).unwrap();
        s.register_callback("__zw_g3", Box::new(|_| "v1".to_string()));
        s.execute("globalThis.captured = __zw_g3;").unwrap();
        s.register_callback("__zw_g3", Box::new(|_| "v2".to_string()));
        let r = s.execute("captured()").unwrap();
        assert_eq!(r.value, "v2", "捕获的旧函数引用跨重注册调当前注册（最后注册者胜）");
    }

    // ── P1b S1 异步回调 resolve（方案 A）──

    #[test]
    fn test_resolve_async_callback_resolves_pending_promise() {
        // 方案 A 端到端：宿主回调同步返「回调 ID」，JS 端建 pending Promise；
        // Rust 异步完成后 resolve_async_callback 触发 __zwResolveCallback resolve。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        // JS 侧 __zwResolveCallback + pending 表（生产由 dom_bridge 注入）。
        sandbox
            .execute(
                "globalThis.__zw_pending = {};
                 globalThis.__zwResolveCallback = function(id, result) {
                     var r = globalThis.__zw_pending[id];
                     if (r) { r(result); delete globalThis.__zw_pending[id]; }
                 };",
            )
            .unwrap();
        // 宿主回调返回调 ID（同步，方案 A）。
        sandbox.register_callback("__zw_start", Box::new(|args| format!("id:{}", args[0])));
        // JS：调宿主回调拿 ID，建 Promise 存 pending[id]，then 写全局 __result。
        sandbox
            .execute(
                "var id = __zw_start('7');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        // resolve 前：Promise pending，__result 未设。
        let before = sandbox.execute("typeof globalThis.__result").unwrap();
        assert_eq!(before.value, "undefined");
        // Rust 异步完成 → resolve（execute 内 perform_microtask_checkpoint drain then）。
        sandbox.resolve_async_callback("id:7", "done-value");
        // resolve 后：__result 已被 .then 写入。
        let after = sandbox.execute("globalThis.__result").unwrap();
        assert_eq!(after.value, "done-value");
    }

    #[test]
    fn test_resolve_async_callback_safe_when_resolver_not_injected() {
        // 防御：JS 侧未注入 __zwResolveCallback（dom_bridge 未接通）时不报错（零回归）。
        let mut sandbox = V8Sandbox::new().unwrap();
        // 不注入 __zwResolveCallback，直接调 resolve_async_callback。
        sandbox.resolve_async_callback("id:x", "v");
        // 沙箱仍可用。
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2");
    }

    #[test]
    fn test_resolve_async_callback_escapes_injection() {
        // id/result 含双引号/反斜杠/代码片段时，js_string_literal 须转义，不得逃逸为 JS 代码。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute(
                "globalThis.__zw_pending = {};
                 globalThis.__zw_evil_called = false;
                 globalThis.__zwResolveCallback = function(id, result) {
                     globalThis.__zw_received = result;
                 };",
            )
            .unwrap();
        // 恶意 payload：试图闭合字符串注入 evil()。js_string_literal 须使其被当作纯字符串。
        sandbox.resolve_async_callback("a\", globalThis.__zw_evil_called = true, \"b", "x\"); evil");
        // evil 未执行（evil_called 仍 false），payload 作为字符串原样送达。
        let evil = sandbox.execute("globalThis.__zw_evil_called").unwrap();
        assert_eq!(evil.value, "false");
        let received = sandbox.execute("globalThis.__zw_received").unwrap();
        assert_eq!(received.value, "x\"); evil");
    }

    #[test]
    fn test_resolve_async_callbacks_batch_order_and_isolation() {
        // t-p103 D2-1：批决议保序（= FIFO 提交序）+ 逐条 try/catch 隔离——单条
        // resolve 抛错不阻断同批其余条目；空批 no-op。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute(
                "globalThis.__resolved = [];
                 globalThis.__zwResolveCallback = function(id, result) {
                     if (id === 'boom') { throw new Error('sentinel'); }
                     globalThis.__resolved.push(id + '=' + result);
                 };",
            )
            .unwrap();
        sandbox.resolve_async_callbacks_batch(&[]);
        sandbox.resolve_async_callbacks_batch(&[
            ("a".to_string(), "1".to_string()),
            ("boom".to_string(), "x".to_string()),
            ("c".to_string(), "3".to_string()),
        ]);
        let r = sandbox.execute("globalThis.__resolved.join(',')").unwrap();
        assert_eq!(r.value, "a=1,c=3");
        // 沙箱仍可用（批内异常被隔离，无遗留异常态）。
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2");
    }

    #[test]
    fn test_resolve_batch_lookup_under_watchdog_p103() {
        // t-p103 返修 DEF-2：批路径首次 `__zwResolveCallback` 属性查找也须在看门狗
        // 保护内——页面可把 shim 全局重定义为自旋 getter，无保护的 global.get 会在
        // 零超时窗执行任意 JS（改前实测 40s 自旋完整越过 30s 沙箱超时）。修复后
        // 查找前装载 Arm，getter 被终止、批整体返回，沙箱可复用。
        let config = SandboxConfig {
            persistent_context: true,
            timeout_ms: 300,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute("globalThis.__zwResolveCallback = function(id, result) {};")
            .unwrap();
        sandbox
            .execute(
                "Object.defineProperty(globalThis, '__zwResolveCallback', {
                     get() { const t = Date.now(); while (Date.now() - t < 5000) {} },
                     configurable: true,
                 });",
            )
            .unwrap();
        let start = std::time::Instant::now();
        sandbox.resolve_async_callbacks_batch(&[("x".to_string(), "v".to_string())]);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 3_000,
            "自旋 getter 须被看门狗终止而非跑满 5s: {elapsed:?}"
        );
        // 沙箱可复用（终止异常已被取走，无残留 terminate 误伤）。
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2");
    }

    #[test]
    fn test_resolve_batch_per_item_watchdog_isolation_p103() {
        // t-p103 返修 TV-2：批内每跳独立装载看门狗（Arm 每跳重设超时窗）——批
        // [正常, 自旋, 正常] 须整体在超时窗内返回，自旋条目被终止，其余条目照常
        // 执行、沙箱不受污染。与 DEF-2 测试互补：DEF-2 钉首查保护，本条钉循环内
        // 每跳保护。
        let config = SandboxConfig {
            persistent_context: true,
            timeout_ms: 300,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute(
                "globalThis.__resolved = [];
                 globalThis.__zwResolveCallback = function(id) {
                     if (id === 'spin') { const t = Date.now(); while (Date.now() - t < 5000) {} }
                     globalThis.__resolved.push(id);
                 };",
            )
            .unwrap();
        let start = std::time::Instant::now();
        sandbox.resolve_async_callbacks_batch(&[
            ("a".to_string(), "1".to_string()),
            ("spin".to_string(), "x".to_string()),
            ("c".to_string(), "3".to_string()),
        ]);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 3_000,
            "自旋条目须被终止而非整批卡满 5s: {elapsed:?}"
        );
        // 自旋条目终止（push 未执行），同批其余条目不受阻断。
        let r = sandbox.execute("globalThis.__resolved.join(',')").unwrap();
        assert_eq!(r.value, "a,c");
        // 沙箱可复用（终止异常已被取走，无残留 terminate 误伤）。
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2");
    }

    #[test]
    fn test_resolve_batch_per_item_checkpoint_order_p103() {
        // t-p103 返修 TV-4：逐条 checkpoint 的 sandbox 层确定性断言——每条决议后
        // 本跳注册的 .then 微任务整链排空才处理下一跳（HTML 事件循环 checkpoint
        // 语义，event_loop_mixed_async_order_r2953 同型回归下沉到 sandbox 层）。
        // 被否决的批末统一 drain 形态会得到 id1,id2,id1m1,id2m1（item2 的 resolve
        // 抢在 item1 的 .then 前，判别）。
        // https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute(
                "globalThis.__ords = [];
                 globalThis.__zwResolveCallback = function(id) {
                     globalThis.__ords.push(id);
                     Promise.resolve().then(function() { globalThis.__ords.push(id + 'm1'); });
                 };",
            )
            .unwrap();
        sandbox.resolve_async_callbacks_batch(&[
            ("id1".to_string(), "v".to_string()),
            ("id2".to_string(), "v".to_string()),
        ]);
        let r = sandbox.execute("globalThis.__ords.join(',')").unwrap();
        assert_eq!(r.value, "id1,id1m1,id2,id2m1");
    }

    #[test]
    fn test_js_string_literal_escapes_special_chars() {
        // 直接单测转义器：常见注入向量与控制字符。
        assert_eq!(js_string_literal("plain"), "\"plain\"");
        assert_eq!(js_string_literal("a\"b"), "\"a\\\"b\"");
        assert_eq!(js_string_literal("a\\b"), "\"a\\\\b\"");
        assert_eq!(js_string_literal("a\nb"), "\"a\\nb\"");
        assert_eq!(js_string_literal("a\tb"), "\"a\\tb\"");
        // 控制字符（U+0001）转 。
        assert_eq!(js_string_literal("a\u{0001}b"), "\"a\\u0001b\"");
    }

    #[test]
    fn test_no_callback_zero_impact() {
        // 无 register_callback 时 execute 行为完全同今（零回归）。
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("6 * 7").unwrap();
        assert_eq!(result.value, "42");
    }

    #[test]
    fn test_execute_function_call() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(function() { return 'success'; })()").unwrap();
        assert_eq!(result.value, "success");
    }

    #[test]
    fn test_execute_object_creation() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("({ name: 'test', value: 123 })").unwrap();
        // V8的object toString返回"[object Object]"
        assert!(
            result.value.contains("test") || result.value.contains("[object Object]"),
            "Expected object representation, got: {}",
            result.value
        );
    }

    #[test]
    fn test_execute_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("[1, 2, 3]").unwrap();
        assert_eq!(result.value, "1,2,3");
    }

    #[test]
    fn test_execute_boolean() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("true").unwrap();
        assert_eq!(result.value, "true");
    }

    #[test]
    fn test_execute_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("undefined").unwrap();
        assert_eq!(result.value, "undefined");
    }

    #[test]
    fn test_execute_null() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("null").unwrap();
        assert_eq!(result.value, "null");
    }

    #[test]
    fn test_execute_math() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("Math.PI").unwrap();
        let pi: f64 = result.value.parse().unwrap();
        assert!((pi - std::f64::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_execute_json_parse() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("JSON.parse('{\"a\":1}').a").unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_execute_json_stringify() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("JSON.stringify({x: 1})").unwrap();
        assert_eq!(result.value, "{\"x\":1}");
    }

    #[test]
    fn test_execute_template_literal() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("`hello ${42}`").unwrap();
        assert_eq!(result.value, "hello 42");
    }

    #[test]
    fn test_execute_arrow_function() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("((x) => x * 2)(21)").unwrap();
        assert_eq!(result.value, "42");
    }

    #[test]
    fn test_execute_destructuring() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const {a, b} = {a: 1, b: 2}; a + b").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_execute_spread_operator() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const arr = [1, 2, 3]; [...arr, 4].length").unwrap();
        assert_eq!(result.value, "4");
    }

    #[test]
    fn test_execute_promise() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("Promise.resolve(42)").unwrap();
        assert!(result.value.contains("Promise") || result.value == "42");
    }

    #[test]
    fn test_execution_time_positive() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("var sum = 0; for (var i = 0; i < 1000; i++) sum += i; sum")
            .unwrap();
        assert_eq!(result.value, "499500");
        assert!(result.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_unicode() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'こんにちは世界 🌍'").unwrap();
        assert!(result.value.contains("こんにちは"));
    }

    #[test]
    fn test_execute_class() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("class Point { constructor(x, y) { this.x = x; this.y = y; } } new Point(1, 2).x")
            .unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_execute_let_const() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("let a = 10; const b = 20; a + b").unwrap();
        assert_eq!(result.value, "30");
    }

    #[test]
    fn test_execute_typeof() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof 42").unwrap();
        assert_eq!(result.value, "number");
    }

    #[test]
    fn test_execute_typeof_string() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof 'hello'").unwrap();
        assert_eq!(result.value, "string");
    }

    #[test]
    fn test_execute_typeof_function() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof function(){}").unwrap();
        assert_eq!(result.value, "function");
    }

    #[test]
    fn test_execute_typeof_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof {}").unwrap();
        assert_eq!(result.value, "object");
    }

    #[test]
    fn test_execute_array_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("[1, 2, 3].map(x => x * 2).join(',')").unwrap();
        assert_eq!(result.value, "2,4,6");
    }

    #[test]
    fn test_execute_string_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'hello world'.split(' ').join('-')").unwrap();
        assert_eq!(result.value, "hello-world");
    }

    // ── 脚本执行：错误路径 ──

    #[test]
    fn test_execute_empty_script() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    fn test_execute_whitespace_only() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("   ");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    fn test_execute_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("function(");
        assert!(matches!(result, Err(ScriptError::CompileError(_))));
    }

    #[test]
    fn test_execute_undefined_variable() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("nonExistentVariable");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_execute_type_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("null.toString()");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    /// t5（qq-20261016-r1）execute 错误栈捕获：必抛脚本的 Err 文本须带栈帧行
    ///（`    at `）——此前 Message 面无帧时拼出 msg\nmsg 重复形态、可归因性为零
    ///（t2 P3 'set' pageerror 证据）。帧行来源：TryCatch::stack_trace()，无帧回落
    /// 异常对象 `.stack`。
    #[test]
    fn test_execute_runtime_error_carries_stack_frames() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let err = sandbox
            .execute("function boom(){ null.toString(); } boom();")
            .expect_err("必抛脚本");
        let text = err.to_string();
        assert!(text.contains("Runtime error"), "前缀保持：{text}");
        assert!(text.contains("    at "), "错误文本须含栈帧行：{text}");
        assert!(text.contains("boom"), "帧行须指向抛出函数：{text}");
    }

    #[test]
    fn test_execute_reference_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("x");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_execute_range_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("new Array(-1)");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_compile_error_has_message() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("function(");
        if let Err(ScriptError::CompileError(msg)) = result {
            assert!(!msg.is_empty(), "Compile error message should not be empty");
        } else {
            panic!("Expected CompileError");
        }
    }

    // ── execute_json ──

    #[test]
    fn test_execute_json_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({a: 1, b: 2})").unwrap();
        assert!(result.value.contains("\"a\""));
        assert!(result.value.contains("\"b\""));
    }

    #[test]
    fn test_execute_json_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("[1, 2, 3]").unwrap();
        assert_eq!(result.value, "[1,2,3]");
    }

    #[test]
    fn test_execute_json_string() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("'hello'").unwrap();
        assert_eq!(result.value, "\"hello\"");
    }

    #[test]
    fn test_execute_json_number() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("42").unwrap();
        assert_eq!(result.value, "42");
    }

    // ── 多次执行 ──

    #[test]
    fn test_multiple_executions() {
        let mut sandbox = V8Sandbox::new().unwrap();

        let r1 = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r1.value, "2");

        let r2 = sandbox.execute("'hello'").unwrap();
        assert_eq!(r2.value, "hello");

        let r3 = sandbox.execute("Math.sqrt(144)").unwrap();
        assert_eq!(r3.value, "12");
    }

    #[test]
    fn test_multiple_sandboxes() {
        let mut sandbox1 = V8Sandbox::new().unwrap();
        let mut sandbox2 = V8Sandbox::new().unwrap();

        let r1 = sandbox1.execute("42").unwrap();
        assert_eq!(r1.value, "42");

        let r2 = sandbox2.execute("'hello'").unwrap();
        assert_eq!(r2.value, "hello");
    }

    // ── 错误类型 ──

    #[test]
    fn test_script_error_display() {
        let err = ScriptError::CompileError("unexpected token".into());
        assert!(err.to_string().contains("Compile error"));
        assert!(err.to_string().contains("unexpected token"));

        let err = ScriptError::RuntimeError("type error".into());
        assert!(err.to_string().contains("Runtime error"));

        let err = ScriptError::Timeout("5s".into());
        assert!(err.to_string().contains("timeout"));

        let err = ScriptError::NotInitialized;
        assert!(err.to_string().contains("not initialized"));

        let err = ScriptError::InvalidInput("empty".into());
        assert!(err.to_string().contains("Invalid input"));

        let err = ScriptError::EngineUnavailable("v8".into());
        assert!(err.to_string().contains("Engine unavailable"));
    }

    #[test]
    fn test_script_result_clone() {
        let result = ScriptResult {
            value: "42".to_string(),
            execution_time_ms: 1.5,
        };
        let cloned = result.clone();
        assert_eq!(cloned.value, "42");
        assert_eq!(cloned.execution_time_ms, 1.5);
    }

    #[test]
    fn test_script_result_debug() {
        let result = ScriptResult {
            value: "hello".to_string(),
            execution_time_ms: 0.1,
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("hello"));
    }

    #[test]
    fn test_sandbox_config_clone() {
        let config = SandboxConfig {
            heap_limit: 1024,
            timeout_ms: 100,
            persistent_context: false,
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.heap_limit, 1024);
        assert_eq!(cloned.timeout_ms, 100);
    }

    #[test]
    fn test_sandbox_config_debug() {
        let config = SandboxConfig {
            heap_limit: 2048,
            timeout_ms: 200,
            persistent_context: false,
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(debug.contains("2048"));
        assert!(debug.contains("200"));
    }

    // ── 状态隔离测试 ──

    #[test]
    /// 测试多次 execute 调用之间变量不泄漏（每次创建新 Context）。
    fn test_state_isolation_between_executions() {
        let mut sandbox = V8Sandbox::new().unwrap();
        // 第一次执行定义变量 x
        let r1 = sandbox.execute("var x = 42; x").unwrap();
        assert_eq!(r1.value, "42");
        // 第二次执行应无法访问 x（新 Context）
        let r2 = sandbox.execute("typeof x === 'undefined'");
        match r2 {
            Ok(result) => assert_eq!(
                result.value, "true",
                "variable from previous execution should not be visible"
            ),
            Err(ScriptError::RuntimeError(_)) => {} // 访问未定义变量抛出 ReferenceError 也是合法
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试多个独立沙箱之间状态完全隔离。
    fn test_state_isolation_between_sandboxes() {
        let mut sandbox1 = V8Sandbox::new().unwrap();
        let mut sandbox2 = V8Sandbox::new().unwrap();
        sandbox1.execute("var secret = 123").unwrap();
        let r2 = sandbox2.execute("typeof secret === 'undefined'");
        assert_eq!(r2.unwrap().value, "true");
    }

    // ── execute_json 边界测试 ──

    #[test]
    /// 测试 execute_json 处理嵌套对象。
    fn test_execute_json_nested_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({a: {b: {c: 1}}})").unwrap();
        assert!(result.value.contains("\"a\""));
        assert!(result.value.contains("\"b\""));
        assert!(result.value.contains("\"c\""));
        assert!(result.value.contains("1"));
    }

    #[test]
    /// 测试 execute_json 处理空对象。
    fn test_execute_json_empty_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({})").unwrap();
        assert_eq!(result.value, "{}");
    }

    #[test]
    /// 测试 execute_json 处理空数组。
    fn test_execute_json_empty_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("([])").unwrap();
        assert_eq!(result.value, "[]");
    }

    #[test]
    /// 测试 execute_json 处理 null。
    fn test_execute_json_null() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("null").unwrap();
        assert_eq!(result.value, "null");
    }

    #[test]
    /// 测试 execute_json 处理 boolean。
    fn test_execute_json_boolean() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute_json("true").unwrap();
        assert_eq!(r1.value, "true");
        let r2 = sandbox.execute_json("false").unwrap();
        assert_eq!(r2.value, "false");
    }

    #[test]
    /// 测试 execute_json 处理 undefined（JSON 序列化为 undefined 通常是 undefined）。
    fn test_execute_json_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("undefined").unwrap();
        // JSON.stringify(undefined) 返回 undefined（非字符串），value_to_json_string 应处理
        assert!(result.value == "null" || result.value == "undefined" || result.value.is_empty());
    }

    #[test]
    /// 测试 execute_json 对语法错误返回 CompileError。
    fn test_execute_json_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("function(");
        assert!(matches!(result, Err(ScriptError::CompileError(_))));
    }

    #[test]
    /// 测试 execute_json 对空脚本返回 InvalidInput。
    fn test_execute_json_empty_script() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    /// 测试 execute_json 处理含特殊字符的字符串。
    fn test_execute_json_special_chars() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("'hello\\nworld\\t!'").unwrap();
        assert!(result.value.contains("hello"));
    }

    // ── 大脚本与性能边界测试 ──

    #[test]
    /// 测试执行较大脚本（10000 次循环）正常返回。
    fn test_execute_large_loop() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let code = "var sum = 0; for (var i = 0; i < 10000; i++) { sum += i; } sum";
        let result = sandbox.execute(code).unwrap();
        assert_eq!(result.value, "49995000");
    }

    #[test]
    /// 测试执行生成大字符串的脚本。
    fn test_execute_large_string_concat() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let code = "var s = ''; for (var i = 0; i < 1000; i++) { s += 'a'; } s.length";
        let result = sandbox.execute(code).unwrap();
        assert_eq!(result.value, "1000");
    }

    // ── 更多 ES6+ 特性测试 ──

    #[test]
    /// 测试 Map 数据结构。
    fn test_execute_map() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("const m = new Map([[1,'a'],[2,'b']]); m.get(1)")
            .unwrap();
        assert_eq!(result.value, "a");
    }

    #[test]
    /// 测试 Set 数据结构。
    fn test_execute_set() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const s = new Set([1,2,2,3]); s.size").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    /// 测试 Symbol。
    fn test_execute_symbol() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof Symbol('test')").unwrap();
        assert_eq!(result.value, "symbol");
    }

    #[test]
    /// 测试 Proxy。
    fn test_execute_proxy() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("const p = new Proxy({}, { get: (t, k) => k === 'name' ? 'zero' : undefined }); p.name")
            .unwrap();
        assert_eq!(result.value, "zero");
    }

    #[test]
    /// 测试 async/await 语法（返回 Promise）。
    fn test_execute_async_await() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(async () => 42)()").unwrap();
        // V8 可能输出 Promise 对象或 42
        assert!(result.value.contains("42") || result.value.contains("Promise"));
    }

    #[test]
    /// 测试默认参数。
    fn test_execute_default_parameters() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(function(x = 10) { return x; })()").unwrap();
        assert_eq!(result.value, "10");
    }

    #[test]
    /// 测试 rest 参数。
    fn test_execute_rest_parameters() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("(function(...args) { return args.length; })(1,2,3)")
            .unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    /// 测试 for...of 循环。
    fn test_execute_for_of() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("let sum = 0; for (const x of [1,2,3]) sum += x; sum")
            .unwrap();
        assert_eq!(result.value, "6");
    }

    #[test]
    /// 测试 Object.entries / Object.values / Object.keys。
    fn test_execute_object_static_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Object.keys({a:1,b:2}).length").unwrap();
        assert_eq!(r1.value, "2");
        let r2 = sandbox
            .execute("Object.values({a:1,b:2}).reduce((s,v)=>s+v,0)")
            .unwrap();
        assert_eq!(r2.value, "3");
        let r3 = sandbox.execute("Object.entries({x:10}).length").unwrap();
        assert_eq!(r3.value, "1");
    }

    #[test]
    /// 测试 Array.isArray / Array.from / Array.of。
    fn test_execute_array_static_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Array.isArray([1,2])").unwrap();
        assert_eq!(r1.value, "true");
        let r2 = sandbox.execute("Array.from('abc').length").unwrap();
        assert_eq!(r2.value, "3");
        let r3 = sandbox.execute("Array.of(1,2,3).length").unwrap();
        assert_eq!(r3.value, "3");
    }

    #[test]
    /// 测试 JSON.parse / JSON.stringify 可用。
    fn test_execute_json_builtins() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("JSON.parse('{\"a\":1}').a").unwrap();
        assert_eq!(r1.value, "1");
        let r2 = sandbox.execute("JSON.stringify({b:2})").unwrap();
        assert!(r2.value.contains("\"b\""));
    }

    #[test]
    /// 测试 Math 常用方法。
    fn test_execute_math_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Math.max(1, 5, 3)").unwrap();
        assert_eq!(r1.value, "5");
        let r2 = sandbox.execute("Math.min(1, 5, 3)").unwrap();
        assert_eq!(r2.value, "1");
        let r3 = sandbox.execute("Math.round(2.7)").unwrap();
        assert_eq!(r3.value, "3");
        let r4 = sandbox.execute("Math.abs(-42)").unwrap();
        assert_eq!(r4.value, "42");
    }

    /// 验证：V8Sandbox::with_config() with custom heap limit and timeout
    #[test]
    fn test_v8_sandbox_with_config_custom() {
        let custom_config = SandboxConfig {
            heap_limit: 64 * 1024 * 1024, // 64MB
            timeout_ms: 10000,
            persistent_context: false,
            ..Default::default()
        };
        let sandbox = V8Sandbox::with_config(custom_config);
        assert!(sandbox.is_ok(), "Custom config should create sandbox successfully");

        let mut sandbox = sandbox.unwrap();

        // 测试能够执行脚本
        let result = sandbox.execute("1 + 1");
        assert!(result.is_ok(), "Sandbox with custom config should execute scripts");
        assert_eq!(result.unwrap().value, "2");
    }

    /// 验证：ScriptError variants: timeout errors, memory limit errors
    #[test]
    fn test_script_error_variants() {
        // 测试各种 ScriptError 变体
        let errors = [
            ScriptError::CompileError("syntax error".to_string()),
            ScriptError::RuntimeError("reference error".to_string()),
            ScriptError::Timeout("5s".to_string()),
            ScriptError::NotInitialized,
            ScriptError::InvalidInput("empty".to_string()),
            ScriptError::EngineUnavailable("v8".to_string()),
        ];

        for error in errors {
            // 测试每个错误的 Display 实现
            let error_str = error.to_string();
            assert!(!error_str.is_empty(), "Error string should not be empty");
        }

        // 验证错误类型的区分
        let compile_error = ScriptError::CompileError("syntax".to_string());
        let runtime_error = ScriptError::RuntimeError("runtime".to_string());
        let timeout_error = ScriptError::Timeout("5s".to_string());

        assert_ne!(compile_error.to_string(), runtime_error.to_string());
        assert_ne!(compile_error.to_string(), timeout_error.to_string());
    }

    /// 验证：execute_json with arrays, nested objects, null
    #[test]
    fn test_execute_json_complex_values() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试数组
        let array_result = sandbox.execute_json("[1, 2, 3, 'hello', null]").unwrap();
        assert!(array_result.value.contains("["));
        assert!(array_result.value.contains("]"));
        assert!(array_result.value.contains("1"));
        assert!(array_result.value.contains("hello"));

        // 测试嵌套对象
        let nested_result = sandbox
            .execute_json("({person: {name: 'John', age: 30, hobbies: ['reading', 'coding']}})")
            .unwrap();
        assert!(nested_result.value.contains("John"));
        assert!(nested_result.value.contains("30"));
        assert!(nested_result.value.contains("reading"));

        // 测试 null
        let null_result = sandbox.execute_json("null").unwrap();
        assert_eq!(null_result.value, "null");
    }

    /// 验证：v8_version() returns non-empty string
    #[test]
    fn test_v8_version_non_empty() {
        let version = V8Sandbox::v8_version();
        assert!(!version.is_empty(), "V8 version should not be empty");
        assert!(version.contains('.'), "Version should contain dots");

        // 版本号应该符合语义化版本格式 (major.minor.patch)
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major and minor");

        // 主要版本号应该是数字
        if let Ok(major) = parts[0].parse::<u32>() {
            assert!(major > 0, "Major version should be positive");
        }
    }

    /// 验证：Empty script execution (should return InvalidInput)
    #[test]
    fn test_empty_script_execution() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试空字符串
        let result = sandbox.execute("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));

        // 测试只有空白字符
        let result = sandbox.execute("   ");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));

        // 测试只有换行符
        let result = sandbox.execute("\n\t\r");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    /// 验证：Script returning undefined
    #[test]
    fn test_script_returning_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试直接返回 undefined
        let result = sandbox.execute("undefined");
        assert!(result.is_ok(), "Should execute undefined without error");
        assert_eq!(result.unwrap().value, "undefined");

        // 测试没有显式返回的函数
        let result = sandbox.execute("(function() {})()");
        assert!(result.is_ok(), "Should execute function without return");
        assert_eq!(result.unwrap().value, "undefined");

        // 测试变量声明但没有赋值
        let result = sandbox.execute("var x; x");
        assert!(result.is_ok(), "Should declare variable without value");
        assert_eq!(result.unwrap().value, "undefined");
    }

    /// 验证：Script with syntax error (verify error type)
    #[test]
    fn test_script_with_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试各种语法错误
        let syntax_error_scripts = [
            "function(",        // 函数定义不完整
            "var = 1",          // 缺少变量名
            "if {",             // if 语句不完整
            "for ()",           // for 循环缺少条件
            "42)",              // 多余的括号
            "'unclosed string", // 未闭合的字符串
        ];

        for script in syntax_error_scripts {
            let result = sandbox.execute(script);
            assert!(
                matches!(result, Err(ScriptError::CompileError(_))),
                "Script '{}' should return CompileError",
                script
            );
        }

        // 验证编译错误消息不为空
        let result = sandbox.execute("function(");
        if let Err(ScriptError::CompileError(msg)) = result {
            assert!(!msg.is_empty(), "Compile error message should not be empty");
            // V8 错误类型名首字母大写（如 "SyntaxError: Unexpected..."），按大小写不敏感匹配
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("syntax") || lower.contains("error") || lower.contains("unexpected"),
                "Error message should indicate syntax issue, got: {msg}"
            );
        } else {
            panic!("Expected CompileError");
        }
    }

    // ── 持久化 Context（V8 快照优化）测试 ──

    #[test]
    /// 测试 persistent_context 模式下变量在多次 execute 间保持。
    fn test_persistent_context_state_persists() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        // 第一次执行：定义变量
        let r1 = sandbox.execute("var persistentVar = 42; persistentVar").unwrap();
        assert_eq!(r1.value, "42");

        // 第二次执行：变量应该仍然存在
        let r2 = sandbox.execute("persistentVar + 8").unwrap();
        assert_eq!(r2.value, "50");
    }

    #[test]
    /// 测试 persistent_context 模式下函数在多次 execute 间保持。
    fn test_persistent_context_function_persists() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("function add(a, b) { return a + b; }").unwrap();
        let r = sandbox.execute("add(3, 4)").unwrap();
        assert_eq!(r.value, "7");
    }

    #[test]
    /// 测试 persistent_context=false（默认）时状态隔离仍然有效。
    fn test_fresh_context_state_isolated() {
        let mut sandbox = V8Sandbox::new().unwrap();

        sandbox.execute("var x = 42; x").unwrap();
        let r = sandbox.execute("typeof x === 'undefined'");
        match r {
            Ok(result) => assert_eq!(result.value, "true", "默认模式下变量不应泄漏"),
            Err(ScriptError::RuntimeError(_)) => {} // ReferenceError 也合法
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试 reset_context 清除持久化上下文。
    fn test_reset_context_clears_state() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("var beforeReset = 99; beforeReset").unwrap();
        sandbox.reset_context();
        // reset 后应该在新 Context 中，变量不存在
        let r = sandbox.execute("typeof beforeReset === 'undefined'");
        match r {
            Ok(result) => assert_eq!(result.value, "true", "reset 后变量应消失"),
            Err(ScriptError::RuntimeError(_)) => {}
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试 persistent_context 模式下 execute_json 也复用上下文。
    fn test_persistent_context_execute_json() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("var data = {x: 1};").unwrap();
        let r = sandbox.execute_json("data").unwrap();
        assert!(r.value.contains("\"x\""), "execute_json 应看到之前定义的变量");
    }

    #[test]
    /// 测试 persistent_context 模式下多次执行不会累积内存问题。
    fn test_persistent_context_many_executions() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        for i in 0..50 {
            let code = format!("var v{i} = {i}; v{i}");
            let r = sandbox.execute(&code).unwrap();
            assert_eq!(r.value, i.to_string());
        }
    }

    #[test]
    /// SEC-13 超时看门狗（2026-08-10 持久化重构）：死循环脚本在 timeout_ms 后被
    /// terminate_execution 终止（Timeout 错误，非长期挂起）；且看门狗可恢复——
    /// 后续 execute 正常（旧 per-execute 线程实现下，脚本出错提前返回会残留
    /// 看门狗线程于 timeout 后误伤下一次执行；新 Arm/Disarm 协议经 guard Drop
    /// 统一撤除，无残留）。多次死循环→恢复循环验证 seq 协议无累积。
    fn test_execute_timeout_terminates_dead_loop_and_reuse() {
        let config = SandboxConfig {
            persistent_context: true,
            timeout_ms: 200,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        let start = std::time::Instant::now();
        let r = sandbox.execute("while(true){}");
        assert!(
            matches!(r, Err(ScriptError::Timeout(_))),
            "死循环应在 timeout_ms 后被终止: {:?}",
            r.err()
        );
        assert!(
            start.elapsed().as_millis() < 5_000,
            "terminate 应在 ~timeout 内到达而非长期等待: {}ms",
            start.elapsed().as_millis()
        );
        // 看门狗撤除后 sandbox 可复用（无残留 terminate 误伤）。
        let r2 = sandbox.execute("1 + 1");
        assert!(r2.is_ok(), "terminate 后 sandbox 应可复用: {:?}", r2.err());
        assert_eq!(r2.unwrap().value, "2");
        // 连续多次死循环→恢复：seq 协议无累积残留。
        for _ in 0..3 {
            let r3 = sandbox.execute("while(true){}");
            assert!(matches!(r3, Err(ScriptError::Timeout(_))), "死循环应持续被终止: {r3:?}");
            assert_eq!(sandbox.execute("2 * 3").unwrap().value, "6");
        }
    }

    // ── s88（html-semantics-compat）：classic 上下文原生动态 import ──
    // classic 脚本（内联/eval 字符串/setTimeout 字符串/Function 构造器——
    // string-compilation 家族）中的 `import()` 无法经源码重写触达，由 V8
    // host-import-module-dynamically 回调承载；三态契约与 `__zw_compile_module`
    // 一致（取回失败返空 / 依赖图断 `resolved\x1f` 空码 / 成功 `resolved\x1fiife`）。

    /// classic 上下文 `import()` 成功路径：resolver 返回 `resolved\x1fiife`，
    /// promise 以模块命名空间落定且模块体已执行；resolver 收到 (spec, referrer)。
    #[test]
    fn test_native_dynamic_import_resolves_in_classic_context() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let calls: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let calls_cb = calls.clone();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(move |spec, referrer, _nonce| {
            calls_cb.lock().unwrap().push((spec.to_string(), referrer.to_string()));
            "https://zero.test/a.js\u{1f}(function() { return (function() { \
             globalThis.__s88_eval = true; return { from: 'a.js' }; })(); })()"
                .to_string()
        }));
        sandbox.set_dynamic_import_referrer("https://zero.test/page.html");
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        // classic 脚本（无 module prelude 预装）：微任务检查点在 execute 内跑 →
        // .then 落定发生在返回前。
        sandbox
            .execute(
                "import('./a.js').then(\n\
                   function (m) { __zw_report('fulfilled:' + JSON.stringify(m) + ':' + globalThis.__s88_eval); },\n\
                   function (e) { __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); });\n",
            )
            .expect("classic 上下文 import() 不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("fulfilled:{\"from\":\"a.js\"}:true"),
            "resolver 返回的 IIFE 求值结果作为命名空间落定且模块体已执行"
        );
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            [("./a.js".to_string(), "https://zero.test/page.html".to_string())],
            "resolver 收到 (spec, referrer)"
        );
    }

    /// s124（modulepreload 面）：classic 上下文原生 import() 的 module map 记忆化——
    /// 成功条目写 `__moduleCache[resolved]`，二次 import() 命中以缓存值落定且模块体
    /// 恰好一次求值（fetch-a-single-module-script 命中复用）；预播种条目（link
    /// rel=modulepreload shim 取回面的同表直写）同样命中不重求值。失败三态不入缓存
    /// （取回失败逐次新 TypeError——s87 口径保持）。
    #[test]
    fn test_native_dynamic_import_module_map_memoization_s124() {
        // module map 记忆化是跨 execute 的全局表面——须 persistent context
        //（非持久模式每 execute 新 context，全局不保留，钉不到命中面）。
        let mut sandbox = V8Sandbox::with_config(crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        let calls: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let calls_cb = calls.clone();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(move |spec, _referrer, _nonce| {
            calls_cb.lock().unwrap().push(spec.to_string());
            if spec.ends_with("fail.js") {
                return String::new(); // 取回失败三态
            }
            let name = spec.trim_start_matches("./").trim_end_matches(".js");
            format!(
                "https://zero.test/{name}.js\u{1f}(function() {{ return (function() {{ \
                 globalThis.__evals = (globalThis.__evals || 0) + 1; return {{ from: '{name}' }}; }})(); }})()"
            )
        }));
        sandbox.set_dynamic_import_referrer("https://zero.test/page.html");
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox.execute("globalThis.__evals = 0;").unwrap();
        // ① 首次 import()：取回 + 求值 + 写表。
        sandbox
            .execute(
                "import('./a.js').then(\n\
                   function (m) { __zw_report('r1:' + m.from + ':' + globalThis.__evals); },\n\
                   function (e) { __zw_report('r1-rejected:' + (e && e.message)); });\n",
            )
            .unwrap();
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("r1:a:1"),
            "首次 import() 取回求值，体执行一次"
        );
        // ② 预播种条目命中（modulepreload shim 面同表直写）——体不重求值。
        sandbox
            .execute(
                "globalThis.__moduleCache = globalThis.__moduleCache || {};\n\
                 globalThis.__moduleCache['https://zero.test/preloaded.js'] = { from: 'preloaded' };",
            )
            .unwrap();
        sandbox
            .execute(
                "import('./preloaded.js').then(\n\
                   function (m) { __zw_report('r2:' + m.from + ':' + (globalThis.__evals || 0)); },\n\
                   function (e) { __zw_report('r2-rejected:' + (e && e.message)); });\n",
            )
            .unwrap();
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("r2:preloaded:1"),
            "播种条目命中即复用，模块体不重求值"
        );
        // ③ 二次 import() 同 spec：命中缓存，体恰好一次求值。
        sandbox
            .execute(
                "import('./a.js').then(\n\
                   function (m) { __zw_report('r3:' + m.from + ':' + globalThis.__evals); },\n\
                   function (e) { __zw_report('r3-rejected:' + (e && e.message)); });\n",
            )
            .unwrap();
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("r3:a:1"),
            "二次 import() 命中缓存，__evals 仍为 1（恰好一次求值）"
        );
        // ④ 失败三态不入缓存：同 spec 两次取回失败各自 reject 新 TypeError，表无条目。
        sandbox
            .execute(
                "import('./fail.js').then(function () { __zw_report('r4-fulfilled'); },\n\
                   function (e) { __zw_report('r4:' + (e && e.constructor.name)); });\n\
                 import('./fail.js').then(function () { __zw_report('r5-fulfilled'); },\n\
                   function (e) { __zw_report('r5:' + (e && e.constructor.name)); });\n",
            )
            .unwrap();
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("r5:TypeError"),
            "取回失败逐次 reject TypeError（不落缓存）"
        );
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            [
                "./a.js".to_string(),
                "./preloaded.js".to_string(),
                "./a.js".to_string(),
                "./fail.js".to_string(),
                "./fail.js".to_string(),
            ],
            "resolver 逐调用取键（取回去重在宿主面），失败臂不缓存短路"
        );
        assert_eq!(
            sandbox
                .execute(
                    "globalThis.__evals + ':' + (globalThis.__moduleCache['https://zero.test/fail.js'] === undefined)"
                )
                .unwrap()
                .value,
            "1:true",
            "模块体恰好一次求值 + 失败条目不入缓存"
        );
    }

    /// s92（string-compilation CSP nonce 面）：nonce 槽位随 resolver 第三参透传——
    /// `set_dynamic_import_nonce(Some)` → resolver 收到该值；`None` 编码空串。
    #[test]
    fn test_dynamic_import_nonce_reaches_resolver() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let nonces: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let nonces_cb = nonces.clone();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(move |_spec, _referrer, nonce| {
            nonces_cb.lock().unwrap().push(nonce.to_string());
            String::new() // 空契约 → reject TypeError（本测只看透传，不关心落定）
        }));
        sandbox.set_dynamic_import_referrer("https://zero.test/page.html");
        sandbox.set_dynamic_import_nonce(Some("correct"));
        sandbox
            .execute("import('./a.js').then(function () {}, function () {});")
            .expect("classic 上下文 import() 不得同步 throw");
        sandbox.set_dynamic_import_nonce(None);
        sandbox
            .execute("import('./b.js').then(function () {}, function () {});")
            .expect("classic 上下文 import() 不得同步 throw");
        assert_eq!(
            nonces.lock().unwrap().as_slice(),
            ["correct".to_string(), String::new()],
            "nonce 槽位随 resolver 第三参透传（None 编码空串）"
        );
    }

    /// s92：`__zwSwapImportNonce` swapper 契约——有参换入返旧值、无参纯 getter；
    /// nonce 空串 ↔ `None` 双射。
    #[test]
    fn test_dynamic_import_nonce_swapper_contract() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.register_dynamic_import_nonce_swapper();
        sandbox.set_dynamic_import_nonce(Some("correct"));
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox
            .execute(
                "__zw_report([\n\
                   __zwSwapImportNonce(),\n\
                   __zwSwapImportNonce(''),\n\
                   __zwSwapImportNonce(),\n\
                   __zwSwapImportNonce('rotated'),\n\
                   __zwSwapImportNonce()\n\
                 ].join('|'));",
            )
            .expect("swapper 序列不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("correct|correct|||rotated"),
            "getter 读当前值、swap 返旧换新、空串 = None"
        );
    }

    /// string-compilation 家族根因钉：eval 字符串内的 `import()`（classic 上下文
    /// 运行期编译，源码重写不可达）同样经原生回调走三态契约。红态：V8 默认
    /// 回调 reject `Error: Not supported`。
    #[test]
    fn test_native_dynamic_import_reaches_eval_compiled_string() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(|_spec, _referrer, _nonce| {
            "https://zero.test/a.js\u{1f}(function() { return (function() { \
             globalThis.__s88_eval = true; return { from: 'a.js' }; })(); })()"
                .to_string()
        }));
        sandbox.set_dynamic_import_referrer("https://zero.test/page.html");
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox
            .execute(
                "eval(\"import('./a.js').then(\\\n\
                   function (m) { __zw_report('fulfilled:' + JSON.stringify(m) + ':' + globalThis.__s88_eval); },\\\n\
                   function (e) { __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); });\")",
            )
            .expect("eval 字符串内 import() 不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("fulfilled:{\"from\":\"a.js\"}:true"),
            "运行期编译字符串内的 import() 经原生回调解析"
        );
    }

    /// 取回失败三态（返空）：以 TypeError `Failed to fetch module: '<spec>'`
    /// reject——与 `__zw_compile_module` 契约同判据（s87 钉死），不再 V8 默认
    /// `Not supported`。
    #[test]
    fn test_native_dynamic_import_fetch_failure_rejects_type_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(|_spec, _referrer, _nonce| String::new()));
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox
            .execute(
                "import('./missing.js').then(\n\
                   function () { __zw_report('fulfilled'); },\n\
                   function (e) { __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); });\n",
            )
            .expect("取回失败不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("rejected:TypeError:Failed to fetch module: './missing.js'"),
            "取回失败以携带 spec 的 TypeError rejection 呈现"
        );
    }

    /// 依赖图断三态（`resolved\x1f` 空码）：以 TypeError `Failed to fetch module
    /// graph: '<resolved>'` reject（WPT dynamic-imports-fetch-error 判据族）。
    #[test]
    fn test_native_dynamic_import_graph_cut_rejects_type_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.set_dynamic_import_resolver(std::sync::Arc::new(|_spec, _referrer, _nonce| {
            "https://zero.test/graphcut.js\u{1f}".to_string()
        }));
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox
            .execute(
                "import('./graphcut.js').then(\n\
                   function () { __zw_report('fulfilled'); },\n\
                   function (e) { __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); });\n",
            )
            .expect("依赖图断不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("rejected:TypeError:Failed to fetch module graph: 'https://zero.test/graphcut.js'"),
            "依赖图断以携带已解析 URL 的 TypeError rejection 呈现"
        );
    }

    /// 未注册 resolver 的嵌入方保真：保持 V8 默认行为——`Error: Not supported`
    /// rejection（不注入 helper、不改判据，零行为外溢）。
    #[test]
    fn test_native_dynamic_import_without_resolver_keeps_v8_default() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        sandbox
            .execute(
                "import('./x.js').then(\n\
                   function () { __zw_report('fulfilled'); },\n\
                   function (e) { __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); });\n",
            )
            .expect("未注册 resolver 时不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("rejected:Error:Not supported"),
            "无宿主 resolver 保持 V8 默认 Error: Not supported"
        );
    }
}
