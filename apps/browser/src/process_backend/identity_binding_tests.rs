//! M1 消息权限矩阵「已由服务端权威绑定」各行的接收方常驻回归（renderer-trust-boundary
//! M2 切片 6；矩阵见 goal evidence 2026-11-01-m1-matrix-storageop-fix.md）。
//!
//! 矩阵各行在 M1 已按生产接收方代码核查定级为安全；本文件把代码核查升级为常驻回归，
//! 防接收方后续重构回退。全部用例不 spawn 真实 renderer：`with_renderer_bin` 惰性构造
//! （cfg(test) 下存储走内存桩），经子模块可见的私有方法直接驱动生产处理路径；与
//! owner tests 的真进程用例（SW/IndexedDB 生命周期、warm spare）互补。

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;

use super::*;
use zero_protocol::{
    AutomationResult, AutomationValue, ServiceWorkerErrorCode, ServiceWorkerScriptTypeWire,
    ServiceWorkerUpdateViaCacheWire,
};

fn test_backend() -> ProcessTabBackend {
    ProcessTabBackend::with_renderer_bin(std::path::PathBuf::from("unused-renderer"))
}

/// DC-3（IPC-03）：renderer 发 `AutomationRequest` 在接收端不可自授自动化角色——
/// poll 分发无该臂，落 `apply_inbound_message` 末臂 `_ => {}` 丢弃；自动化只能由
/// browser 侧 `send_automation_request` 发起。钉住：sink 不产生任何状态变化与回执。
#[test]
fn renderer_automation_request_falls_to_receiver_sink_without_execution() {
    let backend = test_backend();
    let mut snap = TabSnapshot::default();
    let mut snapshot_seq = HashMap::new();
    let mut pending_loaded = Vec::new();
    let mut pending_errors = Vec::new();
    let mut pending_tab_navigates = Vec::new();

    ProcessTabBackend::apply_inbound_message(
        TabId(701),
        &mut snap,
        IpcMessageKind::AutomationRequest(AutomationRequest {
            operation: AutomationOperation::ExecuteScript {
                script: "return navigator.webdriver".into(),
                arguments: Vec::new(),
            },
        }),
        &mut snapshot_seq,
        &mut pending_loaded,
        &mut pending_errors,
        &mut pending_tab_navigates,
    );

    assert!(pending_loaded.is_empty(), "丢弃臂不得产生 loaded 信号");
    assert!(pending_errors.is_empty(), "丢弃臂不得产生错误信号");
    assert!(pending_tab_navigates.is_empty(), "丢弃臂不得产生导航信号");
    assert!(!snap.loading && snap.url.is_none(), "丢弃臂不得改写快照");
    assert!(
        backend.pending_automation_responses.is_empty(),
        "丢弃臂不得伪造自动化回执"
    );
}

/// DC-3：`AutomationResponse` 交付前必须通过 tab 绑定校验——被攻破 renderer 伪造的
/// 回执不得计入其他 tab 的在途自动化请求（矩阵 AutomationResponse 行）。
#[test]
fn automation_response_never_delivered_to_other_tab() {
    let mut backend = test_backend();
    let request_id = 7001_u64;
    backend.pending_automation_responses.insert(
        request_id,
        (
            TabId(1),
            AutomationResponse {
                navigation_epoch: 1,
                document_generation: 1,
                result: Ok(AutomationResult::Value(AutomationValue::String("poisoned".into()))),
            },
        ),
    );

    assert!(
        backend.take_automation_response(TabId(2), request_id).is_none(),
        "跨 tab 回执必须拒绝交付"
    );
    assert!(
        backend.take_automation_response(TabId(1), request_id).is_some(),
        "同 tab 合法回执照常交付（对照臂）"
    );
}

/// DC-1（IPC-01）：SW Register 消息自称的 document_url 与服务端已提交权威不符时必须
/// 拒绝——注册只对已提交导航的文档成立，不采信 renderer 字段。
/// https://w3c.github.io/ServiceWorker/#navigator-service-worker-register
#[test]
fn service_worker_register_rejects_document_url_diverging_from_committed_authority() {
    let mut backend = test_backend();
    let tab_id = TabId(703);
    let renderer_id = 73_u64;
    let committed = "https://trusted.test/page";
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, committed, 1);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: committed.into(),
            navigation_epoch: 1,
        },
    );

    // 权威与 client_id 均取服务端状态（镜像 handle_service_worker_request 的取值来源）。
    let authority = backend
        .committed_document_urls
        .get(&renderer_id)
        .cloned()
        .expect("commit 后应有已提交权威");
    let client_id = backend.service_worker_client_id(renderer_id);

    let register = |document_url: &str| ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::Register {
            script_url: "https://trusted.test/sw.js".into(),
            scope: None,
            document_url: document_url.into(),
            update_via_cache: ServiceWorkerUpdateViaCacheWire::All,
            script_type: ServiceWorkerScriptTypeWire::Classic,
        },
    };

    let forged = backend.service_worker_owner.begin_request_for_client(
        tab_id,
        false,
        7101,
        Some(&authority),
        &client_id,
        register("https://evil.test/page"),
    );
    match forged {
        ServiceWorkerRequestDisposition::Respond(response) => {
            let Err(error) = response.params.result else {
                panic!("伪造 document_url 必须被拒绝，不得注册成功");
            };
            assert_eq!(error.code, ServiceWorkerErrorCode::InvalidArgument);
            assert!(
                error.message.contains("does not match the committed navigation"),
                "拒绝语义应指向权威比对: {error:?}"
            );
        }
        ServiceWorkerRequestDisposition::Fetch(_) => {
            panic!("伪造 document_url 不得进入脚本拉取");
        }
    }

    let honest = backend.service_worker_owner.begin_request_for_client(
        tab_id,
        false,
        7102,
        Some(&authority),
        &client_id,
        register(&authority),
    );
    assert!(
        matches!(honest, ServiceWorkerRequestDisposition::Fetch(_)),
        "document_url 与权威一致的诚实注册应进入正常注册管线（对照臂），不得直接回错误应答"
    );
}

/// DC-1：NavigationStarted 陈旧 epoch（既非快照世代 +1、也非当前世代）必须拒收——
/// 不进 begin_navigation、不 stage pending（矩阵 NavigationStarted 行 epoch+URL 双匹配）。
#[test]
fn navigation_start_with_stale_epoch_is_rejected() {
    let mut backend = test_backend();
    let tab_id = TabId(704);
    let renderer_id = 74_u64;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    let mut snap = TabSnapshot {
        navigation_epoch: 1,
        ..Default::default()
    };

    backend.handle_navigation_started(
        tab_id,
        renderer_id,
        &mut snap,
        NavigationStartedParams {
            url: "https://attacker.test/".into(),
            navigation_epoch: 5,
        },
    );

    assert_eq!(snap.navigation_epoch, 1, "陈旧 epoch 不得推进快照世代");
    assert!(
        !snap.loading && snap.url.is_none(),
        "陈旧 epoch 不得触发 begin_navigation"
    );
    assert!(
        !backend.pending_indexed_db_navigations.contains_key(&renderer_id),
        "陈旧 epoch 不得 stage pending navigation"
    );
    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
}

/// DC-1：NavigationStarted 与已 stage 的 pending 导航（epoch+URL 任一不匹配）必须
/// 拒收且不得覆盖 staged pending——renderer 不能把导航重定向到未授权 URL。
#[test]
fn navigation_start_diverging_from_staged_pending_is_rejected() {
    let mut backend = test_backend();
    let tab_id = TabId(705);
    let renderer_id = 75_u64;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, "https://trusted.test/a", 2);
    let mut snap = TabSnapshot::default();

    backend.handle_navigation_started(
        tab_id,
        renderer_id,
        &mut snap,
        NavigationStartedParams {
            url: "https://attacker.test/b".into(),
            navigation_epoch: 2,
        },
    );

    let pending = backend
        .pending_indexed_db_navigations
        .get(&renderer_id)
        .expect("staged pending 必须保留");
    assert_eq!(pending.url, "https://trusted.test/a", "staged pending 不得被改写");
    assert_eq!(pending.navigation_epoch, 2);
}

/// 对照臂：合法 NavigationStarted（epoch = 快照世代 +1）进入 begin_navigation 并
/// stage pending——证明上述两条拒绝臂不是无条件拒绝。
#[test]
fn navigation_start_with_next_epoch_begins_navigation_and_stages_pending() {
    let mut backend = test_backend();
    let tab_id = TabId(706);
    let renderer_id = 76_u64;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    let mut snap = TabSnapshot::default();

    backend.handle_navigation_started(
        tab_id,
        renderer_id,
        &mut snap,
        NavigationStartedParams {
            url: "https://trusted.test/next".into(),
            navigation_epoch: 1,
        },
    );

    assert!(snap.loading, "合法导航开始应进入 loading");
    assert_eq!(snap.url.as_deref(), Some("https://trusted.test/next"));
    let pending = backend
        .pending_indexed_db_navigations
        .get(&renderer_id)
        .expect("合法导航应 stage pending");
    assert_eq!(pending.url, "https://trusted.test/next");
    assert_eq!(pending.navigation_epoch, 1);
}

/// DC-1/DC-2：renderer `FetchRequest` 自带的 `X-Zero-*` 内部命名空间头是浏览器路由
/// 元数据，剥离后不得出现在出 origin 的请求上；普通头照常转发（矩阵 FetchRequest 行
/// 防护「仅剥 X-Zero-* 内部头」的请求侧）。
#[test]
fn fetch_request_internal_headers_never_reach_origin() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut buf = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buf).unwrap();
            request.extend_from_slice(&buf[..read]);
            if read == 0 || request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let text = String::from_utf8_lossy(&request).to_ascii_lowercase();
        assert!(
            !text.contains("x-zero-"),
            "renderer 内部命名空间头不得出 origin: {text}"
        );
        assert!(text.contains("accept: text/html"), "普通头应照常转发: {text}");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nok")
            .unwrap();
    });

    let mut proxy = TabFetchProxy::new();
    proxy.on_navigate(TabId(1), &base_url);
    proxy.enqueue(
        TabId(1),
        &FetchParams {
            request_id: 1,
            url: format!("{base_url}/probe"),
            method: "GET".into(),
            headers: vec![
                ("X-Zero-Resource-Type".into(), "script".into()),
                ("Accept".into(), "text/html".into()),
            ],
            body: None,
        },
    );

    let completed = wait_for_completed_fetch(&mut proxy);
    assert_eq!(completed.request_id, 1);
    assert_eq!(completed.status, 200, "剥离不得影响请求本身");
    server.join().unwrap();
}

fn wait_for_completed_fetch(proxy: &mut TabFetchProxy) -> crate::fetch_proxy::CompletedFetch {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(item) = proxy.drain().pop() {
            return item;
        }
        assert!(Instant::now() < deadline, "fetch did not complete in time");
        std::thread::sleep(Duration::from_millis(5));
    }
}
