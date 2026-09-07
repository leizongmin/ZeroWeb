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

// R3254-E2 切片 13（editing goal，2026-09-08）：stacked same-sel innerHTML 融合视图
// 塌缩修复——apply 代际作废 removed 补偿。driving: WPT selection/onselectionchange-
// on-document.html 第 3 subtest（'task to fire selectionchange event gets queued each
// time selection is mutated'）——同 sel 连续 innerHTML 后 spin 边界 childNodes 塌缩为 0
// → setPosition IndexSizeError。根因：pa2b apply 代际只清 parse 补偿 added，removed[]
// 条目残留（sel 域快照节点的 identity 补偿）；换代后新基底 rebuild 经 _proxyCache
// 复用同 sel proxy 对象，overlay 的 removed 剔除 identity 命中把快照真实子剔空。
#[test]
fn r3254_e2_slice13_apply_generation_invalidates_removed_compensation() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='container'><br><br></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r3254e2s13".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));

    // 宿主 apply 语义（镜像 apply_pending_shared_mutations：apply → bump，代际边界）。
    fn host_apply(
        sandbox: &mut V8Sandbox,
        mutations: &Arc<Mutex<Vec<DomMutation>>>,
        dom_html: &Arc<Mutex<String>>,
    ) {
        let tail: Vec<DomMutation> = mutations.lock().unwrap().clone();
        if tail.is_empty() {
            return;
        }
        let new_html = crate::js_dom_bridge::apply_mutations_to_html(
            &dom_html.lock().unwrap().clone(),
            &tail,
        )
        .unwrap();
        *dom_html.lock().unwrap() = new_html;
        mutations.lock().unwrap().clear();
        sandbox
            .execute("if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();")
            .unwrap();
    }

    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // turn 1：首次 innerHTML（旧子 br,br 入 removed 补偿）。
    sandbox
        .execute(
            "var container = document.getElementById('container');\n\
             container.innerHTML = '<span>a</span><span>b</span>';\n\
             globalThis.__t1 = container.childNodes.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t1").unwrap().value,
        "2",
        "首次 innerHTML 后同步视图 2 子"
    );
    host_apply(&mut sandbox, &mutations, &dom_html);

    // turn 2：re-register（换代）+ 第二次同 sel innerHTML（前子 span,span 入 removed 补偿）。
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
        .execute(
            "container.innerHTML = '<span>c</span><span>d</span>';\n\
             globalThis.__t2 = container.childNodes.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t2").unwrap().value,
        "2",
        "第二次 innerHTML 后同步视图 2 子"
    );
    host_apply(&mut sandbox, &mutations, &dom_html);

    // turn 3：re-register + 读——修复前 removed 补偿残留使 overlay 把 fresh base 剔空。
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
        .execute("globalThis.__t3 = container.childNodes.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t3").unwrap().value,
        "2",
        "apply 代际后 fresh 基底保持 2 子（removed 补偿已作废，不再剔除真实子）"
    );

    // turn 4：跨 apply 后内容正确性（不仅长度——子内容须为第二次写入的值）。
    sandbox
        .execute(
            "globalThis.__t4a = container.childNodes[0].textContent;\n\
             globalThis.__t4b = container.childNodes[1].textContent;"
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t4a").unwrap().value,
        "c",
        "fresh 基底首子内容 = 第二次 innerHTML 首段"
    );
    assert_eq!(
        sandbox.execute("globalThis.__t4b").unwrap().value,
        "d",
        "fresh 基本次子内容 = 第二次 innerHTML 次段"
    );
}
