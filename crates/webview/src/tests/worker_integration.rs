//! Web Worker 集成测试。
//!
//! 测试 WebView 与 WorkerRuntime 的集成：创建 Worker、
//! 消息传递、脚本执行、多 Worker 隔离、生命周期管理。

use crate::{WebView, WebViewConfig};
use std::time::Duration;
use zero_script_sandbox::WorkerEvent;

/// 测试创建 Worker 并立即终止。
#[test]
fn test_worker_create_and_terminate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.create_worker("var x = 1;").unwrap();
    assert!(wv.is_worker_running(id));
    assert_eq!(wv.worker_count(), 1);

    assert!(wv.terminate_worker(id));
    assert!(!wv.is_worker_running(id));
    assert_eq!(wv.worker_count(), 0);
}

/// 测试终止不存在的 Worker。
#[test]
fn test_terminate_nonexistent_worker() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.terminate_worker(999));
}

/// 测试 Worker 发送初始化消息。
#[test]
fn test_worker_init_message() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.create_worker("postMessage('hello from worker');").unwrap();

    // 等待 Worker 初始化并发送消息
    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    assert!(!events.is_empty(), "Should have at least one event");

    let (wid, event) = &events[0];
    assert_eq!(*wid, id);
    match event {
        WorkerEvent::Message(msg) => assert_eq!(msg, "hello from worker"),
        other => panic!("Expected Message, got: {other:?}"),
    }

    wv.terminate_worker(id);
}

/// 测试 Worker echo 消息。
#[test]
fn test_worker_echo_message() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv
        .create_worker("onmessage = function(e) { postMessage('echo: ' + e.data); };")
        .unwrap();

    // 等待 Worker 就绪
    std::thread::sleep(Duration::from_millis(300));

    wv.post_message_to_worker(id, "test").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msg_events: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msg_events.is_empty(), "Should have echo message");

    if let WorkerEvent::Message(msg) = &msg_events[0].1 {
        assert_eq!(msg, "echo: test");
    }

    wv.terminate_worker(id);
}

/// 测试 Worker 状态保持（有状态计数器）。
#[test]
fn test_worker_stateful_counter() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv
        .create_worker("var count = 0; onmessage = function(e) { count++; postMessage('count: ' + count); };")
        .unwrap();

    std::thread::sleep(Duration::from_millis(300));

    for i in 1..=3 {
        wv.post_message_to_worker(id, "inc").unwrap();
        std::thread::sleep(Duration::from_millis(200));

        let events = wv.poll_worker_events();
        let msgs: Vec<_> = events
            .iter()
            .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
            .collect();
        assert!(!msgs.is_empty(), "Iteration {i}: should have message");
        if let WorkerEvent::Message(msg) = &msgs[0].1 {
            assert_eq!(msg, &format!("count: {i}"), "Iteration {i}");
        }
    }

    wv.terminate_worker(id);
}

/// 测试 Worker 执行额外脚本。
#[test]
fn test_worker_execute_script() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.create_worker("var result = '';").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    wv.execute_worker_script(id, "result = 'computed'; postMessage(result);")
        .unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msgs.is_empty(), "Should have computed result");
    if let WorkerEvent::Message(msg) = &msgs[0].1 {
        assert_eq!(msg, "computed");
    }

    wv.terminate_worker(id);
}

/// 测试多个 Worker 独立隔离。
#[test]
fn test_multiple_workers_isolated() {
    let mut wv = WebView::new(WebViewConfig::default());

    let id1 = wv
        .create_worker("var id = 'w1'; onmessage = function() { postMessage(id); };")
        .unwrap();
    let id2 = wv
        .create_worker("var id = 'w2'; onmessage = function() { postMessage(id); };")
        .unwrap();

    assert_ne!(id1, id2, "Worker IDs should be unique");
    assert_eq!(wv.worker_count(), 2);

    std::thread::sleep(Duration::from_millis(300));

    wv.post_message_to_worker(id1, "ping").unwrap();
    wv.post_message_to_worker(id2, "ping").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();

    assert!(msgs.len() >= 2, "Should have messages from both workers");

    let w1_msg = msgs.iter().find(|(wid, _)| *wid == id1);
    let w2_msg = msgs.iter().find(|(wid, _)| *wid == id2);

    assert!(w1_msg.is_some(), "Should have message from worker 1");
    assert!(w2_msg.is_some(), "Should have message from worker 2");

    if let WorkerEvent::Message(msg) = &w1_msg.unwrap().1 {
        assert_eq!(msg, "w1");
    }
    if let WorkerEvent::Message(msg) = &w2_msg.unwrap().1 {
        assert_eq!(msg, "w2");
    }

    wv.terminate_worker(id1);
    wv.terminate_worker(id2);
}

/// 测试 JSON 消息传递。
#[test]
fn test_worker_json_message() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv
        .create_worker("onmessage = function(e) { postMessage(JSON.stringify({echo: e.data})); };")
        .unwrap();

    std::thread::sleep(Duration::from_millis(300));

    wv.post_message_to_worker(id, "hello").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msgs.is_empty());
    if let WorkerEvent::Message(msg) = &msgs[0].1 {
        assert!(msg.contains("echo"), "Message: {msg}");
        assert!(msg.contains("hello"), "Message: {msg}");
    }

    wv.terminate_worker(id);
}

/// 测试向不存在的 Worker 发送消息。
#[test]
fn test_post_message_to_nonexistent_worker() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.post_message_to_worker(999, "test");
    assert!(result.is_err());
}

/// 测试向不存在的 Worker 执行脚本。
#[test]
fn test_execute_script_on_nonexistent_worker() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_worker_script(999, "1+1");
    assert!(result.is_err());
}

/// 测试 terminate_all_workers。
#[test]
fn test_terminate_all_workers() {
    let mut wv = WebView::new(WebViewConfig::default());

    let id1 = wv.create_worker("var x = 1;").unwrap();
    let id2 = wv.create_worker("var y = 2;").unwrap();
    let id3 = wv.create_worker("var z = 3;").unwrap();

    assert_eq!(wv.worker_count(), 3);

    wv.terminate_all_workers();

    assert_eq!(wv.worker_count(), 0);
    assert!(!wv.is_worker_running(id1));
    assert!(!wv.is_worker_running(id2));
    assert!(!wv.is_worker_running(id3));
}

/// 测试 Worker 复杂计算。
#[test]
fn test_worker_complex_computation() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv
        .create_worker(
            "onmessage = function(e) { var n = parseInt(e.data); var sum = 0; for (var i = 0; i <= n; i++) sum += i; postMessage(String(sum)); };",
        )
        .unwrap();

    std::thread::sleep(Duration::from_millis(300));

    wv.post_message_to_worker(id, "100").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msgs.is_empty());
    if let WorkerEvent::Message(msg) = &msgs[0].1 {
        assert_eq!(msg, "5050");
    }

    wv.terminate_worker(id);
}

/// 测试 Worker 与 WebView 渲染可以并行工作。
#[test]
fn test_worker_parallel_with_render() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载 HTML 并渲染
    let _render = wv.load_html("<html><body><h1>Test</h1></body></html>", None);

    // 创建 Worker
    let id = wv
        .create_worker("onmessage = function(e) { postMessage('processed: ' + e.data); };")
        .unwrap();

    std::thread::sleep(Duration::from_millis(300));

    // 发送消息
    wv.post_message_to_worker(id, "data").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    // 检查事件
    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msgs.is_empty());
    if let WorkerEvent::Message(msg) = &msgs[0].1 {
        assert_eq!(msg, "processed: data");
    }

    // WebView 仍然可以正常渲染
    let render2 = wv.render();
    assert!(!render2.primitives.is_empty());

    wv.terminate_worker(id);
}

/// 测试 Worker 创建使用自定义配置。
#[test]
fn test_worker_with_custom_config() {
    use zero_script_sandbox::SandboxConfig;

    let mut wv = WebView::new(WebViewConfig::default());
    let config = SandboxConfig {
        heap_limit: 8 * 1024 * 1024,
        timeout_ms: 10000,
        persistent_context: false,
        ..Default::default()
    };
    let id = wv.create_worker_with_config("postMessage('ok');", config).unwrap();

    std::thread::sleep(Duration::from_millis(300));

    let events = wv.poll_worker_events();
    let msgs: Vec<_> = events
        .iter()
        .filter(|(_, e)| matches!(e, WorkerEvent::Message(_)))
        .collect();
    assert!(!msgs.is_empty());
    if let WorkerEvent::Message(msg) = &msgs[0].1 {
        assert_eq!(msg, "ok");
    }

    wv.terminate_worker(id);
}

/// 测试 Worker 无 onmessage 处理器时不崩溃。
#[test]
fn test_worker_no_handler_no_crash() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.create_worker("var x = 42;").unwrap();

    std::thread::sleep(Duration::from_millis(300));

    // 没有 onmessage 处理器，不应崩溃
    wv.post_message_to_worker(id, "test").unwrap();

    std::thread::sleep(Duration::from_millis(100));

    assert!(wv.is_worker_running(id));
    wv.terminate_worker(id);
}

/// 测试 Worker 终止后再操作失败。
#[test]
fn test_worker_operations_after_terminate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.create_worker("var x = 1;").unwrap();

    wv.terminate_worker(id);

    assert!(wv.post_message_to_worker(id, "test").is_err());
    assert!(wv.execute_worker_script(id, "1+1").is_err());
    assert!(!wv.is_worker_running(id));
}

/// 测试 Worker ID 单调递增。
#[test]
fn test_worker_ids_monotonic() {
    let mut wv = WebView::new(WebViewConfig::default());

    let id1 = wv.create_worker("var x = 1;").unwrap();
    let id2 = wv.create_worker("var y = 2;").unwrap();
    let id3 = wv.create_worker("var z = 3;").unwrap();

    assert!(id1 < id2);
    assert!(id2 < id3);

    // 终止中间的，新建的 ID 仍然递增
    wv.terminate_worker(id2);
    let id4 = wv.create_worker("var w = 4;").unwrap();
    assert!(id3 < id4);

    wv.terminate_all_workers();
}
