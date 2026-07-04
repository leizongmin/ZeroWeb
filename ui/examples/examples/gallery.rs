//! 组件画廊 GPU 演示 — 开窗运行 Gallery 示例（`cargo run -p zero-ui-examples --example gallery`）。
//!
//! Gallery 经 WinitRuntime 驱动完整 winit 事件循环 + GPU 渲染管线：
//! 1. WinitRuntime::run() 创建 winit EventLoop + Window
//! 2. 经 WinitDriver 驱动 UI SDK retained 闭环
//! 3. 经 RenderFoundationBackend 把 Scene 转换为 RenderPrimitives
//! 4. 经 GpuRenderer 渲染到窗口 surface

use zero_ui_runtime::platform::PlatformRuntime;

fn main() {
    let mut rt = zero_ui_adapter_winit::WinitRuntime::new();
    rt.set_register(|host| {
        zero_ui_examples::gallery::register_gallery_factories(host);
    });
    let mut app = zero_ui_examples::gallery::GalleryApp::new();
    rt.run(&mut app).unwrap();
}
