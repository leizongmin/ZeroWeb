//! 组件画廊 GPU 演示 — 开窗运行 Gallery 示例（`cargo run -p zero-ui-examples --example gallery`）。
//!
//! Gallery 经 WinitRuntime 驱动完整 winit 事件循环 + GPU 渲染管线：
//! 1. WinitRuntime::run() 创建 winit EventLoop + Window
//! 2. 经 WinitDriver 驱动 UI SDK retained 闭环
//! 3. 经 RenderFoundationBackend 把 Scene 转换为 RenderPrimitives
//! 4. 经 GpuRenderer 渲染到窗口 surface

use zero_ui_adapter_winit::{FontAsset, FontContainer};
use zero_ui_runtime::platform::PlatformRuntime;

fn main() {
    let mut rt = zero_ui_adapter_winit::WinitRuntime::new();
    // 注册字体（DC-17 FontConfig API）：调用方负责自己 example 的字体资源，
    // adapter 不再硬编码依赖 wpt-runner 测试目录。注册顺序 = fallback 顺序。
    rt.add_font(FontAsset::static_bytes(
        "UI",
        include_bytes!("../../../tests/wpt-runner/wpt-data/fonts/noto/noto-sans-v8-latin-regular.woff"),
        FontContainer::Woff,
    ));
    rt.add_font(FontAsset::static_bytes(
        "CJK",
        include_bytes!("../../../tests/wpt-runner/wpt-data/fonts/mplus-1p-regular.woff"),
        FontContainer::Woff,
    ));
    // P1-2：系统字体回退。wpt 自带的 mplus-1p 是日文字体，中文覆盖不全
    // （"你" U+4F60 不在其中），导致输入中文时 draw_text 找不到字形只推进
    // pen_x 不绘制，字符显示空白。系统字体补齐 CJK 覆盖（与浏览器
    // app_platform.rs 同口径：macOS=PingFang / Windows=YaHei / Linux=NotoCJK）。
    // 系统字体可能不存在（CI 无头环境等），缺失时 tracing 提示并跳过。
    for (family, path) in system_cjk_font_paths() {
        if let Ok(data) = std::fs::read(path) {
            eprintln!("[gallery] Loaded system CJK font {family} from {path}");
            rt.add_font(FontAsset::owned(family, data, FontContainer::Ttf));
        } else {
            eprintln!("[gallery] System CJK font {family} not present at {path}, skipping");
        }
    }
    rt.set_register(|host| {
        zero_ui_examples::gallery::register_gallery_factories(host);
    });
    let mut app = zero_ui_examples::gallery::GalleryApp::new();
    rt.run(&mut app).unwrap();
}

/// 按平台返回系统 CJK 字体候选路径（注册顺序 = fallback 顺序）。
/// 与 `apps/browser/src/app_platform.rs` 同口径。
fn system_cjk_font_paths() -> Vec<(&'static str, &'static str)> {
    let mut paths = Vec::new();
    #[cfg(target_os = "macos")]
    {
        paths.push(("CJK-System", "/System/Library/Fonts/PingFang.ttc"));
        paths.push(("CJK-System2", "/System/Library/Fonts/STHeiti Light.ttc"));
    }
    #[cfg(target_os = "windows")]
    {
        paths.push(("CJK-System", "C:\\Windows\\Fonts\\msyh.ttc"));
        paths.push(("CJK-System2", "C:\\Windows\\Fonts\\msyh.ttf"));
        paths.push(("CJK-System3", "C:\\Windows\\Fonts\\simsun.ttc"));
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        paths.push(("CJK-System", "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"));
        paths.push(("CJK-System2", "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf"));
        paths.push(("CJK-System3", "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"));
        paths.push(("CJK-System4", "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"));
    }
    paths
}
