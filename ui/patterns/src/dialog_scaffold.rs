//! DialogScaffold — 对话框脚手架（spec FR-009；权限提示/确认弹窗等通用结构）。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogScaffold {
    pub title_msg: String,
    pub open: bool,
    pub modal: bool,
}

impl DialogScaffold {
    pub fn new(title_msg: &str) -> DialogScaffold {
        DialogScaffold {
            title_msg: title_msg.to_string(),
            open: false,
            modal: true,
        }
    }
    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_close_modal_default() {
        let mut d = DialogScaffold::new("perm.geolocation.title");
        assert!(d.modal);
        d.open();
        assert!(d.open);
        d.close();
        assert!(!d.open);
    }
}
