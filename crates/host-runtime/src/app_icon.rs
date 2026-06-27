//! ZeroBrowser 应用图标（运行时窗口图标）。
//!
//! 内嵌 256×256 RGBA 数据，供 Windows / Linux 在任务栏、窗口标题栏、
//! Alt+Tab 中显示。macOS 不使用窗口图标（由 .app bundle 的 .icns 提供）。
//!
//! 源 SVG：apps/browser/assets/app-icon.svg
//! 由 tools/icon-gen 生成：window-icon-256.rgba（256×256×4 = 262144 字节）。

/// 内嵌的 256×256 RGBA 窗口图标数据。
const WINDOW_ICON_RGBA: &[u8] = include_bytes!("../assets/window-icon-256.rgba");
const WINDOW_ICON_SIDE: u32 = 256;

/// 构造 winit 窗口图标；解码失败时返回 None（不阻塞启动）。
pub fn window_icon() -> Option<winit::window::Icon> {
    winit::window::Icon::from_rgba(WINDOW_ICON_RGBA.to_vec(), WINDOW_ICON_SIDE, WINDOW_ICON_SIDE).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_has_expected_size() {
        // 256×256×4 = 262144
        assert_eq!(WINDOW_ICON_RGBA.len(), 262_144, "内嵌窗口图标数据大小应为 262144 字节");
    }

    #[test]
    fn window_icon_decodes_successfully() {
        let icon = window_icon();
        assert!(icon.is_some(), "窗口图标应能成功解码");
    }
}
