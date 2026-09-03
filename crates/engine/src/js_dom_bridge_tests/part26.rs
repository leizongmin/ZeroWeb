// js_dom_bridge 测试切片 26。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

#[test]
fn test_canvas_capture_stream_track_marks_service_worker_messageerror() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='canvas' width='5' height='5'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var stream = canvas.captureStream();\
             var track = stream.getVideoTracks()[0];\
             globalThis.__canvasNamed = String(canvas === document.getElementById('canvas'));\
             globalThis.__trackKind = track.kind;\
             globalThis.__trackState = track.readyState;\
             globalThis.__tracks = String(stream.getTracks().length + ':' + stream.getAudioTracks().length);\
             globalThis.__marker = String(track.__zwServiceWorkerMessageErrorTransfer === true);\
             track.stop();\
             globalThis.__trackStopped = track.readyState;",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__canvasNamed").unwrap().value,
        "true",
        "static canvas id should be exposed through window named access"
    );
    assert_eq!(
        sandbox.execute("globalThis.__trackKind").unwrap().value,
        "video",
        "captureStream() returns one video track"
    );
    assert_eq!(
        sandbox.execute("globalThis.__trackState").unwrap().value,
        "live",
        "captureStream() track starts live"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tracks").unwrap().value,
        "1:0",
        "captureStream() exposes video tracks only"
    );
    assert_eq!(
        sandbox.execute("globalThis.__marker").unwrap().value,
        "true",
        "canvas track carries the Service Worker messageerror marker"
    );
    assert_eq!(
        sandbox.execute("globalThis.__trackStopped").unwrap().value,
        "ended",
        "track.stop() transitions the synthetic track to ended"
    );
}

#[test]
fn test_service_worker_post_message_routes_canvas_track_to_messageerror() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let posted = Arc::new(Mutex::new(Vec::new()));
    let posted_for_callback = posted.clone();
    sandbox.register_callback(
        "__zw_sw_post_message",
        Box::new(move |args| {
            posted_for_callback.lock().unwrap().push(format!(
                "{}|{}|{}|{}|{}",
                args.first().cloned().unwrap_or_default(),
                args.get(1).cloned().unwrap_or_default(),
                args.get(2).cloned().unwrap_or_default(),
                args.get(3).cloned().unwrap_or_default(),
                args.get(4).cloned().unwrap_or_default()
            ));
            r#"{"ok":true}"#.to_string()
        }),
    );

    sandbox
        .execute(
            "var worker = new ServiceWorker('https://example.test/sw.js', 'activated');\
             worker._id = 'r1';\
             __zwInitServiceWorkerMessageBridge(worker, { id: 'client-1', url: 'https://example.test/page' });\
             var track = { kind: 'video' };\
             Object.defineProperty(track, '__zwServiceWorkerMessageErrorTransfer', { value: true });\
             globalThis.__posted = 'no';\
             try {\
               worker.postMessage({ track: track }, [track]);\
               globalThis.__posted = 'yes';\
             } catch (e) {\
               globalThis.__posted = e.name;\
             }",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__posted").unwrap().value,
        "yes",
        "marked canvas capture tracks should not synchronously throw DataCloneError"
    );
    assert_eq!(
        posted.lock().unwrap().as_slice(),
        &[r#"r1|{"__zwServiceWorkerMessageError":true}|[]||"#.to_string()]
    );
}
