//! 系统剪贴板读写（地址栏与页面选区复制）。

/// 读取剪贴板纯文本。
pub fn read_text() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

/// 写入剪贴板纯文本。
pub fn write_text(text: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .is_ok()
}
