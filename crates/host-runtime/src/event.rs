//! 事件类型 — 窗口事件和输入事件

/// 应用生命周期事件
#[derive(Debug)]
pub enum AppEvent {
    /// 窗口需要重绘
    RedrawRequested,
    /// 窗口大小变更
    Resized {
        /// 新宽度
        width: u32,
        /// 新高度
        height: u32,
    },
    /// 窗口关闭请求
    CloseRequested,
    /// 窗口获得焦点
    Focused,
    /// 窗口失去焦点
    Unfocused,
    /// 键盘输入
    KeyboardInput {
        /// 按键
        key: String,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
    },
}
