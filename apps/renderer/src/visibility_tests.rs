//! SetVisibility IPC 宿主驱动面（后台资源生命周期 M3，HTML Page Visibility）。
//!
//! 锁定：browser 前后台 tab 差分 → `SetVisibility` → renderer `handle_set_visibility`
//! → page_hidden 宿主态 + worker 内 `document.hidden`/`visibilityState` 翻转（赋值+
//! visibilitychange 派发原子脚本）。JS 关闭时宿主态仍记录、worker 不执行。
//! 差分发送方与事件派发契约分别在 browser `poll_diffs_set_visibility_on_active_tab_change`
//! 与 engine `page_visibility_state_and_visibilitychange_dispatch_s72` 钉住。

use super::*;
use zero_protocol::message::SetVisibilityParams;

fn runtime_with_sink(renderer_id: u64) -> RendererRuntime {
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime
}

fn vis_state(runtime: &RendererRuntime) -> String {
    runtime
        .js_worker
        .execute_script_direct("[String(document.hidden), document.visibilityState].join(',')")
        .unwrap()
}

#[test]
fn set_visibility_ipc_flips_document_hidden_and_host_state() {
    let mut runtime = runtime_with_sink(9401);
    runtime.stub_network = true;
    // 装载 shim（worker 初始文档），初态可见（headless 单 tab 零回归面）。
    runtime
        .js_worker
        .set_dom_snapshot("<html><body></body></html>", "about:blank");
    assert!(!runtime.page_hidden, "构造初态宿主可见");
    assert_eq!(vis_state(&runtime), "false,visible", "初态可见");

    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetVisibility(SetVisibilityParams { hidden: true }),
        })
        .unwrap();
    assert!(runtime.page_hidden, "宿主态记录隐藏（rAF 泵门控数据源）");
    assert_eq!(vis_state(&runtime), "true,hidden", "IPC 链后 worker 内 hidden 翻转");

    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetVisibility(SetVisibilityParams { hidden: false }),
        })
        .unwrap();
    assert!(!runtime.page_hidden, "恢复可见");
    assert_eq!(vis_state(&runtime), "false,visible", "恢复可见翻转回 visible");
}

#[test]
fn set_visibility_records_host_state_with_js_disabled() {
    let mut runtime = runtime_with_sink(9402);
    runtime.javascript_enabled = false;
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetVisibility(SetVisibilityParams { hidden: true }),
        })
        .unwrap();
    assert!(runtime.page_hidden, "JS 关闭仍记录宿主态（泵门控不依赖 JS 开关）");
}

/// DC-1 后台策略面端到端（真 SetVisibility IPC 臂 + 真 shim interval 链）：隐藏页
/// interval 钳到 1s 节奏（恢复自动回原延迟——re-arm 每代重读 `__ZW_HIDDEN`，无需显式
/// 恢复动作）。钳制面在 shim 公共 timer API 入口（`_zwClampDelay`），本测经 IPC 注入
/// 隐藏态走完整链。窗口余量：可见 ~4ms/拍（shim nesting 钳制下限）→ 500ms 窗 ≥10 拍；
/// 隐藏 1s/拍 → 2200ms 窗 ≤3 拍；恢复首拍 ≤1s 到位后回 4ms 节奏 → 2200ms 窗 ≥10 拍。
/// 翻转后先静置 100ms 再清零——翻转前 arm 的 4ms 残留 entry（≤2 拍）排空后才起测窗，
/// 防计入残留拍。
#[test]
fn background_timer_policy_throttles_interval_and_recovers() {
    let mut runtime = runtime_with_sink(9403);
    runtime.stub_network = true;
    runtime
        .js_worker
        .set_dom_snapshot("<html><body></body></html>", "about:blank");
    let tick = |runtime: &RendererRuntime| {
        runtime
            .js_worker
            .execute_script_direct("globalThis.__n = (globalThis.__n || 0) + 1; String(globalThis.__n)")
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    runtime
        .js_worker
        .execute_script_direct(
            "globalThis.__n = 0; setInterval(function () { globalThis.__n = globalThis.__n + 1; }, 0);",
        )
        .unwrap();

    // 可见基线：~4ms/拍。
    std::thread::sleep(std::time::Duration::from_millis(500));
    let visible_ticks = tick(&runtime);
    assert!(
        visible_ticks >= 10,
        "可见页 interval 应按 shim 钳制节奏（~4ms/拍）触发，500ms 窗实得 {visible_ticks} 拍"
    );

    // 隐藏：钳到 1s/拍（先静置排空翻转前 arm 的 4ms 残留）。
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetVisibility(SetVisibilityParams { hidden: true }),
        })
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    runtime.js_worker.execute_script_direct("globalThis.__n = 0;").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2200));
    let hidden_ticks = tick(&runtime);
    assert!(
        hidden_ticks <= 3,
        "隐藏页 interval 应钳到 1s/拍（2200ms 窗 ≤3 拍），实得 {hidden_ticks} 拍——后台节流失效"
    );

    // 恢复可见：re-arm 时点判定，新 arm 回 ~4ms/拍（首拍 ≤1s 后节奏恢复）。
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::SetVisibility(SetVisibilityParams { hidden: false }),
        })
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    runtime.js_worker.execute_script_direct("globalThis.__n = 0;").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2200));
    let restored_ticks = tick(&runtime);
    assert!(
        restored_ticks >= 10,
        "恢复可见后新 arm 应回原节奏（2200ms 窗 ≥10 拍），实得 {restored_ticks} 拍——恢复语义失效"
    );
}
