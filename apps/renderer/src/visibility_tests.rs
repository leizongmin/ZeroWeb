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
