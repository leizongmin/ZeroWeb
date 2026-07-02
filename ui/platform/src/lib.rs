//! # zero-ui-platform
//!
//! 平台服务（spec §8.4.1 `zero-ui-platform` / FR-016 / §8.4.1B 拖拽/剪贴板/file picker 走 platform service）。
//!
//! 全部为 trait（可 mock），不向 widgets 暴露具体后端；浏览器拖拽/打开文件通过这些服务执行。

use std::cell::RefCell;

/// 剪贴板服务。
pub trait Clipboard {
    fn get_text(&self) -> Option<String>;
    fn set_text(&self, text: &str);
}

/// 文件选择结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedFile {
    pub path: String,
    pub display_name: String,
}

/// 文件选择器。
pub trait FilePicker {
    fn pick_open(&self) -> Option<PickedFile>;
    fn pick_save(&self, suggested: &str) -> Option<PickedFile>;
}

/// 内存剪贴板（测试用 + headless）。trait 方法为 `&self`（与 arboard 后端一致），内部用 RefCell。
#[derive(Debug, Default)]
pub struct InMemoryClipboard {
    content: RefCell<Option<String>>,
}

impl InMemoryClipboard {
    pub fn new() -> InMemoryClipboard {
        InMemoryClipboard::default()
    }
}

impl Clipboard for InMemoryClipboard {
    fn get_text(&self) -> Option<String> {
        self.content.borrow().clone()
    }
    fn set_text(&self, text: &str) {
        *self.content.borrow_mut() = Some(text.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_round_trip() {
        let cb: Box<dyn Clipboard> = Box::new(InMemoryClipboard::new());
        assert!(cb.get_text().is_none());
        cb.set_text("https://zero.example");
        assert_eq!(cb.get_text().as_deref(), Some("https://zero.example"));
    }
}
