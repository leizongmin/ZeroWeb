//! CommandPalette — 命令面板（spec FR-009 / §8.4.1B 同 command 多入口）。
//!
//! 输入过滤 command model（`ui/commands`）中的命令；选中触发 command。

use zero_ui_core::action::ActionId;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandEntry {
    pub label: String,
    pub action: ActionId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandPalette {
    pub open: bool,
    pub query: String,
    pub entries: Vec<CommandEntry>,
    pub highlighted: Option<usize>,
}

impl CommandPalette {
    pub fn new(entries: Vec<CommandEntry>) -> CommandPalette {
        CommandPalette {
            open: false,
            query: String::new(),
            entries,
            highlighted: None,
        }
    }

    /// 按 query 过滤（label 大小写不敏感包含）。
    pub fn filtered(&self) -> Vec<&CommandEntry> {
        let q = self.query.to_ascii_lowercase();
        self.entries
            .iter()
            .filter(|e| e.label.to_ascii_lowercase().contains(&q))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_query() {
        let p = CommandPalette::new(vec![
            CommandEntry {
                label: "Reload".into(),
                action: ActionId::new("browser.reload"),
            },
            CommandEntry {
                label: "Open File".into(),
                action: ActionId::new("file.open"),
            },
        ]);
        let mut f = p.clone();
        f.query = "rel".into();
        assert_eq!(f.filtered().len(), 1);
        assert_eq!(f.filtered()[0].action, ActionId::new("browser.reload"));
    }
}
