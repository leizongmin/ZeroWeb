// part65 — M3-S1b worker canvas 桥（goal compositor-animation-continuity，RFC
// `docs/goal/compositor-animation-continuity/m3-slice-worker-canvas.md` §2.2）：
// `transferControlToOffscreen` flag 关保持 R3313 平面对象逐字节不变（workers-compat
// 钉住）；flag 开返回占位 OffscreenCanvas + surface 键（getContext/transferToImageBitmap
// 抛 InvalidStateError——HTML spec §transferred-canvas，control 已移交 worker；二次
// transfer 抛 TypeError）；postMessage 宿主分支把列内 OffscreenCanvas 序列化为
// `__zwXferCanvas` 标记（worker 侧 bootstrap 复活），未列/已 detach 抛 DataCloneError。
// flag/桥槽进程全局：沿用 part64 WORKER_HOST_FLAG_LOCK 串行 + 假桥 marker 过滤。
// 沙盒沿用 part14 R3313 先例：预解析 `<canvas id>`（createElement 形态无该方法面）。

/// S1b 公共沙盒：worker_m2_sandbox 同构，body 预置 `<canvas id='cv'>`。
fn s1b_sandbox<F>(page_url: &str, fetch: F) -> zero_script_sandbox::V8Sandbox
where
    F: Fn(&str, &str) -> String + Send + Sync + 'static,
{
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let page = args.first().map(String::as_str).unwrap_or("");
            let src = args.get(1).map(String::as_str).unwrap_or("");
            fetch(page, src)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><canvas id='cv' width='64' height='32'></canvas><canvas id='cv2' width='8' height='8'></canvas></body></html>".to_string()));
    let page: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url.to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page, &canvas_registry, None);
    sandbox
}

/// flag 关（默认）：`transferControlToOffscreen()` 保持 R3313 平面对象（非
/// `instanceof OffscreenCanvas`、无 `_zwXferKey`、getContext 复用 DOM handle 可用）
/// ——宿主分支不可达，workers-compat 零行为。
#[test]
fn test_transfer_control_to_offscreen_flag_off_plain_object_unchanged() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let mut sandbox = s1b_sandbox("https://wpt.test/s1b/flag-off.html", |_page, _src| String::new());
    sandbox
        .execute(
            "var c = document.getElementById('cv');\
             var oc = c.transferControlToOffscreen();\
             globalThis.__r = [oc instanceof OffscreenCanvas, typeof oc._zwXferKey, typeof oc.getContext].join('|');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "false|undefined|function",
        "flag off 保持 R3313 平面对象（共享 handle 路径逐字节不变）"
    );
}

/// flag 开：占位 OffscreenCanvas（真类型 + surface 键 + 主 canvas 尺寸）；
/// getContext/transferToImageBitmap 抛 InvalidStateError；二次 transfer 抛 TypeError。
#[test]
fn test_transfer_control_to_offscreen_flag_on_placeholder_semantics() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let _flag = WorkerHostFlagGuard::set();
    let log: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::new(Mutex::new(vec![]));
    install_worker_host_fake_bridge(&log);
    let mut sandbox = s1b_sandbox("https://wpt.test/s1b/placeholder.html", |_page, _src| String::new());
    sandbox
        .execute(
            "var c = document.getElementById('cv');\
             var oc = c.transferControlToOffscreen();\
             globalThis.__r = [oc instanceof OffscreenCanvas, typeof oc._zwXferKey, oc.width, oc.height].join('|');\
             globalThis.__eCtx = '';\
             try { oc.getContext('2d'); globalThis.__eCtx = 'no-throw'; } catch (e) { globalThis.__eCtx = e.name; }\
             globalThis.__eTib = '';\
             try { oc.transferToImageBitmap(); globalThis.__eTib = 'no-throw'; } catch (e) { globalThis.__eTib = e.name; }\
             globalThis.__eRe = '';\
             try { c.transferControlToOffscreen(); globalThis.__eRe = 'no-throw'; } catch (e) { globalThis.__eRe = e.name; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "true|string|64|32",
        "flag 开返回占位 OffscreenCanvas（真类型 + surface 键 + 尺寸保真）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__eCtx").unwrap().value,
        "InvalidStateError",
        "占位 getContext 抛 InvalidStateError（control 已移交 worker）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__eTib").unwrap().value,
        "InvalidStateError",
        "占位 transferToImageBitmap 抛 InvalidStateError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__eRe").unwrap().value,
        "TypeError",
        "二次 transfer 抛 TypeError"
    );
    crate::worker_bridge::set_worker_host_op(None);
}

/// postMessage 宿主分支：列内占位序列化为 `__zwXferCanvas` 标记（键 + 尺寸随迁）；
/// 未列入转移列表抛 DataCloneError；已 detach（首次转移后重发）抛 DataCloneError。
#[test]
fn test_worker_post_message_transfer_offscreen_marker_and_errors() {
    use zero_script_sandbox::Sandbox;
    let _flag_window = WORKER_HOST_FLAG_LOCK.lock().unwrap();
    let _flag = WorkerHostFlagGuard::set();
    let log: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::new(Mutex::new(vec![]));
    install_worker_host_fake_bridge(&log);
    let mut sandbox = s1b_sandbox("https://wpt.test/s1b/xfer.html", |_page, src| {
        if src == "s1b-worker.js" {
            format!("{WORKER_HOST_E2E_MARKER} onmessage = function (e) {{ postMessage('got'); }};")
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "var worker = new Worker('s1b-worker.js');\
             var c = document.getElementById('cv');\
             var oc = c.transferControlToOffscreen();\
             globalThis.__err = '';\
             try { worker.postMessage({ bmp: oc }, [oc]); } catch (e) { globalThis.__err = e.name; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err").unwrap().value,
        "",
        "列内占位转移不抛"
    );
    let post = log
        .lock()
        .unwrap()
        .iter()
        .find(|entry| entry.starts_with("post\u{1f}7\u{1f}"))
        .expect("转移 postMessage 应路由 op('post', id, JSON)")
        .clone();
    let payload = post.split('\u{1f}').nth(2).expect("post 载荷三段式");
    assert!(
        payload.contains(r#"{"bmp":{"__zwXferCanvas":{"key":"#) && payload.contains(r#""w":64,"h":32}}"#),
        "post 载荷应把占位序列化为 __zwXferCanvas 标记（键 + 尺寸），got: {payload}"
    );

    // 未列入转移列表 → DataCloneError（spec：OffscreenCanvas transferable 非
    // Serializable——载荷出现必须在转移列表）。
    sandbox
        .execute(
            "var c2 = document.getElementById('cv2');\
             var oc2 = c2.transferControlToOffscreen();\
             globalThis.__err2 = '';\
             try { worker.postMessage({ bmp: oc2 }); } catch (e) { globalThis.__err2 = e.name; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err2").unwrap().value,
        "DataCloneError",
        "未列入转移列表的 OffscreenCanvas 抛 DataCloneError"
    );

    // 已 detach（首次转移置 _detached）再转移 → DataCloneError（spec transfer 步
    // is-detached check——_zwValidateTransferList 先于 detach 校验）。
    sandbox
        .execute(
            "globalThis.__err3 = '';\
             try { worker.postMessage({ bmp: oc }, [oc]); } catch (e) { globalThis.__err3 = e.name; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err3").unwrap().value,
        "DataCloneError",
        "已 detach 的 OffscreenCanvas 再转移抛 DataCloneError"
    );
    crate::worker_bridge::set_worker_host_op(None);
}
