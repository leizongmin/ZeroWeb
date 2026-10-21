// part64 — M3-S1a 宿主 worker 分流（goal compositor-animation-continuity，RFC
// `docs/goal/compositor-animation-continuity/m3-slice-worker-canvas.md` §2.1）：
// ZW_THREADED_WORKERS flag 关零行为（`__zw_worker_host_op`/`__zw_worker_host_dispatch`
// 不定义，worker 走 same-realm——workers-compat 钉住）；flag 开 + 桥槽装填时 spawn
// 分流（prologue + 脚本体）、post/terminate 路由、`__zw_worker_host_dispatch` 三类
// 事件派发与 `__zwUndef`/`__zwStr` 类型标记解包；spawn 失败回落 same-realm。
// 沙盒沿用 part61 worker_m2_sandbox 样式。flag/桥槽进程全局：本文件测试经
// WORKER_HOST_FLAG_LOCK 串行 + 假桥 marker 过滤（非本测试 spawn 一律 err: 回落
// same-realm）——并行窗口内其他 worker 测试（part61 等）不受 set_var 影响。

use std::sync::Mutex;

/// flag/槽窗口互斥（本文件 flag 关/开测试串行；进程全局面见文件头注记）。
static WORKER_HOST_FLAG_LOCK: Mutex<()> = Mutex::new(());

/// 本测试 worker 脚本体 marker：假桥只对含 marker 的 spawn 回 id，其余 err:。
const WORKER_HOST_E2E_MARKER: &str = "/*ZW_HOST_E2E*/";

/// ZW_THREADED_WORKERS=1 作用域守卫（Drop 还原——进程全局，panic 也还原）。
struct WorkerHostFlagGuard;

impl WorkerHostFlagGuard {
    fn set() -> Self {
        // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）——basic.rs:455 先例。
        unsafe { std::env::set_var("ZW_THREADED_WORKERS", "1") };
        WorkerHostFlagGuard
    }
}

impl Drop for WorkerHostFlagGuard {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("ZW_THREADED_WORKERS") };
    }
}

/// 假桥槽记录：`op\u{1f}arg1\u{1f}arg2…` 逐条 push；spawn 含 marker 回 "7"，其余 err:。
/// marker 过滤保证并行窗口内其他测试的 Worker spawn 回落 same-realm（零影响）。
fn install_worker_host_fake_bridge(log: &std::sync::Arc<Mutex<Vec<String>>>) {
    let log = log.clone();
    crate::worker_bridge::set_worker_host_op(Some(std::sync::Arc::new(
        move |args: &[String]| -> String {
            let op = args.first().map(String::as_str).unwrap_or("");
            if op == "spawn" && !args.get(1).is_some_and(|s| s.contains(WORKER_HOST_E2E_MARKER)) {
                return "err:not-under-test".into();
            }
            log.lock().unwrap().push(args.join("\u{1f}"));
            match op {
                "spawn" => "7".into(),
                _ => "ok".into(),
            }
        },
    )));
}

/// flag 关（默认）：`__zw_worker_host_op`/`__zw_worker_host_dispatch` 不定义、
/// Worker 照常 same-realm 回显——分流面零行为（workers-compat 钉住）。
#[test]
fn test_worker_host_flag_off_zero_surface_and_same_realm() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let mut sandbox = worker_m2_sandbox("https://wpt.test/hostE2E/flag-off.html", |_page, src| {
        if src == "flag-off-worker.js" {
            "onmessage = function (e) { postMessage(e.data); };".to_string()
        } else {
            String::new()
        }
    });
    assert_eq!(
        sandbox.execute("typeof __zw_worker_host_op").unwrap().value,
        "undefined",
        "flag 关不注册宿主 op 回调"
    );
    assert_eq!(
        sandbox.execute("typeof __zw_worker_host_dispatch").unwrap().value,
        "undefined",
        "flag 关不定义宿主派发函数"
    );
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('flag-off-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = event.data; };",
        )
        .unwrap();
    sandbox.execute("worker.postMessage('same-realm');").unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "same-realm",
        "flag 关 worker 走 same-realm（零行为）"
    );
    assert_eq!(
        sandbox.execute("String(worker._zwHostId)").unwrap().value,
        "undefined",
        "flag 关不设宿主 id（same-realm 路径无该属性）"
    );
}

/// flag 开 + 桥槽装填：spawn 收 prologue + 脚本体并回 id（`_zwHostId` 置位）；
/// postMessage 路由 `op('post', id, JSON)`（`__zwStr` 标记保字符串类型）；
/// `__zw_worker_host_dispatch` message 派发经 setTimeout(0) task 到 onmessage，
/// `__zwStr`/`__zwUndef` 解包还原类型；terminate 路由并摘注册表（后续派发无效）。
#[test]
fn test_worker_host_flag_on_routes_spawn_post_dispatch_terminate() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let _flag = WorkerHostFlagGuard::set();
    let log: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::new(Mutex::new(vec![]));
    install_worker_host_fake_bridge(&log);
    let mut sandbox = worker_m2_sandbox("https://wpt.test/hostE2E/route.html", |_page, src| {
        if src == "host-e2e-worker.js" {
            format!(
                "{WORKER_HOST_E2E_MARKER} onmessage = function (e) {{ postMessage(e.data); }};"
            )
        } else {
            String::new()
        }
    });
    assert_eq!(
        sandbox.execute("typeof __zw_worker_host_op").unwrap().value,
        "function",
        "flag 开 + 槽装填注册宿主 op 回调"
    );
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             globalThis.__undefR = 'pending';\
             var worker = new Worker('host-e2e-worker.js');\
             worker.onmessage = function (event) {\
               globalThis.__r = String(event.data) + ':' + typeof event.data;\
             };",
        )
        .unwrap();

    // spawn：prologue（__zwUndef/__zwStr 标记补丁 + _dispatchMessage JSON.parse）+ 脚本体。
    let spawn_op = log
        .lock()
        .unwrap()
        .iter()
        .find(|entry| entry.starts_with("spawn\u{1f}"))
        .expect("构造器应向桥发 spawn")
        .clone();
    assert!(
        spawn_op.contains("__zwUndef") && spawn_op.contains("__zwStr"),
        "spawn 载荷应含 prologue 类型标记补丁"
    );
    assert!(
        spawn_op.contains(WORKER_HOST_E2E_MARKER),
        "spawn 载荷应含脚本体"
    );
    assert_eq!(
        sandbox.execute("String(worker._zwHostId)").unwrap().value,
        "7",
        "spawn 回 id 置 _zwHostId"
    );
    assert_eq!(
        sandbox.execute("typeof __zw_worker_host_dispatch").unwrap().value,
        "function",
        "首个 spawn 成功时惰性定义宿主派发函数"
    );

    // post 路由：字符串包 `__zwStr` 标记 JSON。
    sandbox.execute("worker.postMessage('hi');").unwrap();
    assert!(
        log.lock()
            .unwrap()
            .iter()
            .any(|entry| entry.starts_with("post\u{1f}7\u{1f}"))
            && log
                .lock()
                .unwrap()
                .iter()
                .any(|entry| entry.contains("\"__zwStr\":1") && entry.contains("\"v\":\"hi\"")),
        "postMessage 应路由 op('post', id, JSON)，got: {:?}",
        log.lock().unwrap()
    );

    // message 派发：__zwStr 解包 → onmessage 收到字符串 'hi'（类型保真）。
    sandbox
        .execute("__zw_worker_host_dispatch(7, 'message', { __zwStr: 1, v: 'hi' });")
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "hi:string",
        "message 派发应解包 __zwStr 还原字符串"
    );
    // __zwUndef 解包 → undefined。
    sandbox
        .execute(
            "worker.onmessage = function (event) {\
               globalThis.__undefR = String(event.data) + ':' + typeof event.data;\
             };",
        )
        .unwrap();
    sandbox
        .execute("__zw_worker_host_dispatch(7, 'message', { __zwUndef: 1 });")
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__undefR").unwrap().value,
        "undefined:undefined",
        "message 派发应解包 __zwUndef 还原 undefined"
    );

    // terminate：路由 op('terminate', id)、摘注册表——后续派发无效。
    sandbox.execute("worker.terminate();").unwrap();
    assert!(
        log.lock()
            .unwrap()
            .iter()
            .any(|entry| entry.starts_with("terminate\u{1f}7")),
        "terminate 应路由 op('terminate', id)，got: {:?}",
        log.lock().unwrap()
    );
    assert_eq!(
        sandbox.execute("String(worker._zwHostId)").unwrap().value,
        "null",
        "terminate 应清 _zwHostId"
    );
    sandbox
        .execute(
            "worker.onmessage = function () { globalThis.__r = 'SHOULD-NOT-FIRE'; };\
             __zw_worker_host_dispatch(7, 'message', { __zwStr: 1, v: 'late' });",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "hi:string",
        "terminate 后注册表已摘，迟派发不触发 onmessage"
    );
    crate::worker_bridge::set_worker_host_op(None);
}

/// flag 开 + spawn 失败（桥回 err:）：回落 same-realm worker（逐字节不变路径），
/// postMessage 走 same-realm 队列不落桥。
#[test]
fn test_worker_host_spawn_failure_falls_back_to_same_realm() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let _flag = WorkerHostFlagGuard::set();
    let log: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::new(Mutex::new(vec![]));
    let log2 = log.clone();
    crate::worker_bridge::set_worker_host_op(Some(std::sync::Arc::new(move |args: &[String]| -> String {
        log2.lock().unwrap().push(args.join("\u{1f}"));
        "err:off".into()
    })));
    let mut sandbox = worker_m2_sandbox("https://wpt.test/hostE2E/fallback.html", |_page, src| {
        if src == "fallback-worker.js" {
            "onmessage = function (e) { postMessage(e.data); };".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('fallback-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = event.data; };",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(worker._zwHostId)").unwrap().value,
        "undefined",
        "spawn 失败不设宿主 id（回落 same-realm 路径无该属性）"
    );
    sandbox.execute("worker.postMessage('fallback');").unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "fallback",
        "spawn 失败回落 same-realm 回显"
    );
    assert!(
        !log.lock().unwrap().iter().any(|entry| entry.starts_with("post\u{1f}")),
        "回落后 postMessage 不落桥，got: {:?}",
        log.lock().unwrap()
    );
    crate::worker_bridge::set_worker_host_op(None);
}
