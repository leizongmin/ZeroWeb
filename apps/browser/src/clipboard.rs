//! 系统剪贴板读写（地址栏与页面选区复制）。
//!
//! X11 下剪贴板所有权随 `arboard::Clipboard` 实例存活——实例即丢即弃会让刚写入的
//! 内容立刻失主（其他应用与本进程都读不回）。故进程内保活单实例，读写共用；
//! 初始化失败（无 display 等）缓存为不可用，不逐次重试。

use std::sync::{Mutex, MutexGuard, OnceLock};

fn clipboard_instance() -> Option<MutexGuard<'static, arboard::Clipboard>> {
    static CLIPBOARD: OnceLock<Option<Mutex<arboard::Clipboard>>> = OnceLock::new();
    CLIPBOARD
        .get_or_init(|| arboard::Clipboard::new().ok().map(Mutex::new))
        .as_ref()
        .and_then(|clipboard| clipboard.lock().ok())
}

/// 读取剪贴板纯文本。
pub fn read_text() -> Option<String> {
    clipboard_instance()?.get_text().ok()
}

/// 写入剪贴板纯文本。
pub fn write_text(text: &str) -> bool {
    let Some(mut clipboard) = clipboard_instance() else {
        return false;
    };
    clipboard.set_text(text.to_owned()).is_ok()
}
