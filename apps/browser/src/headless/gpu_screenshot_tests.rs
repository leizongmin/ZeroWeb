//! GPU 截图开关一致性测试（R3282 #4）。

use super::*;

use serde_json::Value;

/// R3282（#4）：GPU 截图开关下 PNG 输出与 CPU 截图一致（同 pipeline primitives，
/// GPU 支持子集逐像素一致——parity/reftest 已验证）。
#[test]
fn gpu_screenshot_matches_cpu_for_supported_scene() {
    let _gpu_lock = super::GPU_SCREENSHOT_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let server = HeadlessServer::new(0, 64.0, 64.0);
    let mut session = HeadlessSession::new(64.0, 64.0);
    // 先渲染一帧（页面内容进入 webview）
    session.webview.load_html(
        r#"<html><body style="margin:0"><div style="width:40px;height:40px;background:#f00;"></div></body></html>"#,
        None,
    );
    // CPU 截图
    let cpu_result = server
        .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
        .unwrap();
    let cpu_png = cpu_result["data"]["png"].as_str().unwrap().to_string();
    // GPU 截图（env 开关）
    unsafe {
        std::env::set_var("ZW_HEADLESS_GPU_SCREENSHOT", "1");
    }
    let gpu_result = server
        .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
        .unwrap();
    unsafe {
        std::env::remove_var("ZW_HEADLESS_GPU_SCREENSHOT");
    }
    let gpu_png = gpu_result["data"]["png"].as_str().unwrap().to_string();
    // GPU 环境不可用（无适配器）时可能回退 CPU——两者仍应一致
    assert_eq!(
        cpu_png, gpu_png,
        "GPU 截图应与 CPU 截图逐字节一致（支持子集；GPU 不可用自动回退）"
    );
}
