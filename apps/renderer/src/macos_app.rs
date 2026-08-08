//! macOS Helper app 生命周期。

use std::path::Path;
use std::sync::mpsc;
use std::thread;

use dispatch::Queue;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSImage, NSImageNameApplicationIcon};
use objc2_foundation::MainThreadMarker;

use super::RendererRuntime;

fn is_app_bundle_executable(path: &Path) -> bool {
    path.parent().is_some_and(|dir| dir.ends_with("Contents/MacOS"))
        && path
            .ancestors()
            .nth(3)
            .and_then(Path::extension)
            .is_some_and(|extension| extension == "app")
}

/// 判断当前 renderer 是否作为 macOS app bundle 的主可执行文件运行。
pub(super) fn is_bundled_app_executable() -> bool {
    std::env::current_exe()
        .ok()
        .is_some_and(|path| is_app_bundle_executable(&path))
}

/// 在 AppKit 主事件循环旁运行 renderer runtime。
///
/// AppKit 必须占用主线程，才能把直接 spawn 的 Helper 注册为 UIElement application。
pub(super) fn run_renderer(renderer_id: u64) -> Result<(), String> {
    let main_thread = MainThreadMarker::new().ok_or("AppKit must run on the main thread")?;
    let application = NSApplication::sharedApplication(main_thread);
    application.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    if let Some(icon) = unsafe { NSImage::imageNamed(NSImageNameApplicationIcon) } {
        unsafe { application.setApplicationIconImage(Some(&icon)) };
    }

    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let worker = thread::Builder::new()
        .name(format!("renderer-{renderer_id}-runtime"))
        .spawn(move || {
            let mut runtime = RendererRuntime::new(renderer_id);
            let result = runtime.run();
            let _ = result_tx.send(result);
            Queue::main().exec_async(|| {
                let main_thread = MainThreadMarker::new().expect("main dispatch queue must run on the main thread");
                unsafe { NSApplication::sharedApplication(main_thread).terminate(None) };
            });
        })
        .map_err(|error| format!("Failed to start renderer runtime thread: {error}"))?;

    unsafe { application.run() };
    worker
        .join()
        .map_err(|_| "Renderer runtime thread panicked".to_string())?;
    result_rx
        .recv()
        .map_err(|error| format!("Failed to receive renderer runtime result: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::is_app_bundle_executable;
    use std::path::Path;

    #[test]
    fn recognizes_only_app_bundle_main_executables() {
        assert!(is_app_bundle_executable(Path::new(
            "/Applications/ZeroBrowser.app/Contents/MacOS/ZeroBrowser"
        )));
        assert!(is_app_bundle_executable(Path::new(
            "/Applications/ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Renderer).app/Contents/MacOS/ZeroBrowser Helper (Renderer)"
        )));
        assert!(!is_app_bundle_executable(Path::new(
            "/workspace/target/release/zero-renderer"
        )));
    }
}
