// js_dom_bridge 测试切片 31（t6 验收：P10 IDB 连接队列楔死 + P11 blob: worker 脚本）。
// 本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，与 part01-30 共享模块作用域。
// 新切片先例：part16（part14/15 超 2000 行后新工作落新文件）。

// ── P10：host 不可达时 deleteDatabase 须以 error 事件结算，且连接队列不得楔死 ──
//
// 修复前：headless 下 IndexedDbRequest 快速错误应答（PR #40）使 `_zwIDBHostCall` 同步抛
// DOMException；deleteDatabase 回调首个调用 `_zwIDBUsesHostConnections()`（part02）位于
// try/catch 之外 → 异常逃逸 `_zwIDBRunConnectionQueue` → queue.running 恒 true，
// 同名后续 open/deleteDatabase 永不结算。
// 修复后：删除前置步骤失败 → error 事件送达 request（spec deleting-a-database）；
// 队列兜底 finish 保证后续请求照常推进（spec connection-queues）。

#[test]
fn test_indexeddb_delete_database_host_error_settles_request_and_keeps_queue_alive() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 模拟 headless host 桥：每个 host 调用都以 PR #40 的统一错误串应答。
    sandbox
        .execute(
            "globalThis.__zw_idb = function () {\
               return '__zw_idb_error:UnknownError: IndexedDB is unavailable in headless mode';\
             };\
             globalThis.__p10 = [];\
             var dq = indexedDB.deleteDatabase('p10');\
             dq.onerror = function (e) { globalThis.__p10.push('delete-error:' + e.target.error.name); };\
             dq.onsuccess = function () { globalThis.__p10.push('delete-success'); };\
             var op = indexedDB.open('p10');\
             op.onerror = function (e) { globalThis.__p10.push('open-error:' + e.target.error.name); };\
             op.onsuccess = function () { globalThis.__p10.push('open-success'); };\
             var dq2 = indexedDB.deleteDatabase('p10');\
             dq2.onerror = function (e) { globalThis.__p10.push('delete2-error:' + e.target.error.name); };\
             dq2.onsuccess = function () { globalThis.__p10.push('delete2-success'); };",
        )
        .unwrap();
    sandbox.execute("1;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__p10.join('|')").unwrap().value,
        "delete-error:UnknownError|open-error:UnknownError|delete2-error:UnknownError",
        "deleteDatabase host 错误以 error 事件结算；同名 open 与第二次 delete 不被队列楔死"
    );
}

// ── P11：blob: URL worker 脚本须从 _zwBlobStore 取源执行；查无此 blob → error 事件 ──
//
// 修复前：blob: worker URL 落 __zw_fetch_script（net 不解析 blob:）→ 静默取 null →
// worker 不执行、无 onmessage/onerror。修复后：createObjectURL 注册的 Blob 同步取
// _parts 拼源执行；store 未命中（revoke 后使用）按 spec worker-processing-model
// fetch 失败语义派发 error 事件。

#[test]
fn test_worker_blob_url_script_executes_and_missing_blob_fires_error() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "var source = 'self.onmessage=function(e){postMessage(e.data+1)}';\
             var url = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));\
             var w = new Worker(url);\
             globalThis.__p11 = 'none';\
             w.onmessage = function (ev) { globalThis.__p11 = 'onmessage:' + ev.data; };\
             w.onerror = function (ev) { globalThis.__p11 = 'onerror:' + (ev.message || ''); };\
             w.postMessage(41);\
             var revoked = URL.createObjectURL(new Blob(['self.onmessage=null'], { type: 'text/javascript' }));\
             URL.revokeObjectURL(revoked);\
             var w2 = new Worker(revoked);\
             globalThis.__p11revoked = 'none';\
             w2.onerror = function () { globalThis.__p11revoked = 'onerror'; };\
             w2.onmessage = function () { globalThis.__p11revoked = 'onmessage'; };\
             w2.postMessage(1);\
             var src3 = 'var T=\"\u{2713}\";self.onmessage=function(e){postMessage(e.data+T)};';\
             var bytes3 = new TextEncoder().encode(src3);\
             var w3 = new Worker(URL.createObjectURL(new Blob([bytes3], { type: 'text/javascript' })));\
             globalThis.__p11bytes = 'none';\
             w3.onmessage = function (ev) { globalThis.__p11bytes = ev.data; };\
             w3.postMessage('done');\
             var w4 = new Worker(URL.createObjectURL(new Blob([new Blob([src3])])));\
             globalThis.__p11nested = 'none';\
             w4.onmessage = function (ev) { globalThis.__p11nested = ev.data; };\
             w4.postMessage('done');",
        )
        .unwrap();
    sandbox.execute("1;").unwrap();

    assert_eq!(
        sandbox.execute("String(globalThis.__p11)").unwrap().value,
        "onmessage:42",
        "blob: worker 脚本从 _zwBlobStore 取源执行：postMessage(41) → worker +1 → main onmessage(42)"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p11revoked)").unwrap().value,
        "onerror",
        "revoke 后的 blob URL：fetch 失败语义 → worker error 事件（非静默无回调）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p11bytes)").unwrap().value,
        "done\u{2713}",
        "TypedArray part（含多字节 UTF-8 ✓）按 UTF-8 解码执行，非 Latin-1 乱码"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p11nested)").unwrap().value,
        "done\u{2713}",
        "嵌套 Blob part 递归物化，不静默丢弃"
    );
}
