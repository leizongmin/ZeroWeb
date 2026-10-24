//! T5/P10 宿主历史桥集成测试——页面 history.back/forward 接通宿主会话历史栈。
//!
//! 覆盖链路两半：shim 移交半（TabJsWorkerHandle 真实 shim + HistoryBridge，跨文档
//! `__zw_reset_history` 重置后越界遍历投递 delta）与宿主提交半（BrowserShell 会话栈
//! go_back/go_forward 后 can_go_forward 装载面）。两半的衔接契约是桥队列 delta。
//! spec：https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-history-interface

use zero_browser::tab_js_worker::TabJsWorkerHandle;
use zero_browser_shell::BrowserShell;
use zero_webview::{WebView, WebViewConfig};

const PAGE_HTML: &str = "<html><body><p>t5 history traverse</p></body></html>";

fn drain_traverses(worker: &TabJsWorkerHandle) -> Vec<i32> {
    worker
        .pending_history_traverses()
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default()
}

/// 等待 shim 遍历任务落地：back/forward/go 入队走 `setTimeout(...,0)`（tab 路径经
/// TimerBridge 真实异步延迟），execute_script_direct 回执后任务可能尚未跑——轮询
/// 桥队列直至移交到达（上限 2s，超时返空供断言给出发信号）。
fn wait_for_handoff(worker: &TabJsWorkerHandle) -> Vec<i32> {
    for _ in 0..200 {
        let drained = drain_traverses(worker);
        if !drained.is_empty() {
            return drained;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Vec::new()
}

/// A→B 跨文档装载的 shim 侧模拟：SetDomSnapshot url 变化 → `__zw_reset_history`
/// 重置为单 entry（R3059 契约，宿主 tab_js_worker/renderer js_worker 真实路径同款）。
fn load_cross_document(worker: &TabJsWorkerHandle, from: &str, to: &str) {
    worker.set_dom_snapshot(PAGE_HTML, from);
    worker.set_dom_snapshot(PAGE_HTML, to);
}

/// ① A→B 跨文档后执行 history.back()：shim 移交 delta=-1，宿主提交装载回 A 且
/// 活跃 tab can_go_forward()=true（宿主 shell 栈与页面遍历共享同一 tab 级栈）。
#[test]
fn history_back_after_cross_document_loads_previous_entry_t5() {
    let mut worker = TabJsWorkerHandle::spawn(zero_browser_shell::TabId(7001));
    let mut shell = BrowserShell::new();

    // A→B 跨文档：宿主 shell 栈 [A, B]，shim 每文档栈随重置归一。
    shell.navigate("https://zero.test/a");
    shell.on_page_loaded("A");
    shell.navigate("https://zero.test/b");
    shell.on_page_loaded("B");
    load_cross_document(&worker, "https://zero.test/a", "https://zero.test/b");

    // 页面发起返回：shim 本地栈越界 → 移交宿主。
    worker.execute_script_direct("history.back();").unwrap();
    assert_eq!(
        wait_for_handoff(&worker),
        vec![-1],
        "跨文档后 history.back() → 移交 delta=-1"
    );

    // 宿主提交（app.go_back 同路径：shell 栈移动 + 装载目标 URL）。
    assert!(shell.go_back(), "宿主 shell 栈后退成功");
    assert_eq!(
        shell.active_tab().and_then(|t| t.url()),
        Some("https://zero.test/a"),
        "装载回 A"
    );
    assert!(
        shell.active_tab().is_some_and(|t| t.can_go_forward()),
        "前进可达（can_go_forward=true）"
    );
    worker.shutdown();
}

/// ② forward() 回 B：移交 delta=+1，宿主提交后 active url 回 B、前进不可达。
#[test]
fn history_forward_after_back_returns_to_next_entry_t5() {
    let mut worker = TabJsWorkerHandle::spawn(zero_browser_shell::TabId(7002));
    let mut shell = BrowserShell::new();

    shell.navigate("https://zero.test/a");
    shell.on_page_loaded("A");
    shell.navigate("https://zero.test/b");
    shell.on_page_loaded("B");
    load_cross_document(&worker, "https://zero.test/a", "https://zero.test/b");

    // back 半（①）。
    worker.execute_script_direct("history.back();").unwrap();
    assert_eq!(wait_for_handoff(&worker), vec![-1]);
    assert!(shell.go_back());

    // forward 半：移交 +1 → 宿主提交回 B。
    worker.execute_script_direct("history.forward();").unwrap();
    assert_eq!(wait_for_handoff(&worker), vec![1], "history.forward() → 移交 delta=+1");
    assert!(shell.go_forward(), "宿主 shell 栈前进成功");
    assert_eq!(
        shell.active_tab().and_then(|t| t.url()),
        Some("https://zero.test/b"),
        "forward 装载回 B"
    );
    assert!(
        !shell.active_tab().is_some_and(|t| t.can_go_forward()),
        "已到栈尾，前进不可达"
    );
    worker.shutdown();
}

/// ③ 无桥环境越界 back()/go() 仍 no-op（零漂移守卫）——WebView 路径不注册
/// `__zw_request_history_traverse`（zero-webview 稳定边界不新增会话历史 API），
/// shim feature-detect 保底回落 R3004 静默 no-op：location 与 history.length 不变。
#[test]
fn history_back_without_bridge_stays_noop_t5() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html(PAGE_HTML, None);
    let href0 = wv.execute_script("location.href").unwrap();
    wv.execute_script(
        "history.back();\
         history.go(-2);",
    )
    .expect("execute no-bridge probe");
    let href1 = wv.execute_script("location.href").unwrap();
    let len1 = wv.execute_script("String(history.length)").unwrap();
    assert_eq!(href1, href0, "无桥越界 back/go 后 location 不变");
    assert_eq!(
        len1, "1",
        "无桥越界 back/go 后 history.length 保持单 entry（R3004 no-op）"
    );
}

/// ④ go(-2) 越界：shim 移交原始 delta（判界在宿主），宿主 shell 判界不可达时 no-op——
/// 栈、索引、按钮态全部不变（go(delta) 绝对目标 = jump_history(index+delta) 共享路径，
/// 越界 no-op 语义由 shell jump_history 单测钉定；此处断言移交面与 shell 零漂移）。
#[test]
fn history_go_beyond_bounds_hands_off_and_shell_stays_put_t5() {
    let mut worker = TabJsWorkerHandle::spawn(zero_browser_shell::TabId(7003));
    let mut shell = BrowserShell::new();

    shell.navigate("https://zero.test/a");
    shell.on_page_loaded("A");
    shell.navigate("https://zero.test/b");
    shell.on_page_loaded("B");
    load_cross_document(&worker, "https://zero.test/a", "https://zero.test/b");

    worker.execute_script_direct("history.go(-2);").unwrap();
    assert_eq!(
        wait_for_handoff(&worker),
        vec![-2],
        "go(-2) 越界 → 移交原始 delta=-2（宿主判界）"
    );

    // 宿主判界（app.history_traverse：index+delta < 0 → 不触栈）后 shell 零变化。
    let tab = shell.active_tab().unwrap();
    let (len, index) = (tab.history_len(), tab.history_index());
    assert_eq!((len, index), (2, 1), "前置：shell 栈 [A,B] 在 index=1");
    assert!(!shell.jump_history(5), "绝对目标越界 no-op（jump_history 共享路径）");
    let tab = shell.active_tab().unwrap();
    assert_eq!(tab.url(), Some("https://zero.test/b"), "越界后 url 不变");
    assert_eq!(
        (tab.history_len(), tab.history_index()),
        (len, index),
        "越界后栈与索引不变"
    );
    assert!(tab.can_go_back() && !tab.can_go_forward(), "越界后遍历按钮态不变");
    worker.shutdown();
}
