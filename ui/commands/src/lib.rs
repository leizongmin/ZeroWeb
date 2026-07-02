//! # zero-ui-commands
//!
//! 命令系统（spec §8.4.1 `zero-ui-commands` / FR-016 / §8.4.1B 菜单/快捷键/command palette 同 command）。
//!
//! 所有命令注册为 ActionId；菜单项、快捷键、命令面板只触发 command，再映射到应用 Action。

use compact_str::CompactString;
use zero_ui_core::action::{ActionId, ActionResult};
use zero_ui_core::event::{KeyCode, Modifiers};

/// 键盘快捷键。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

impl Shortcut {
    pub fn new(key: &str, modifiers: Modifiers) -> Shortcut {
        Shortcut {
            key: KeyCode::new(key),
            modifiers,
        }
    }

    /// 是否匹配给定键 + 修饰。
    pub fn matches(&self, key: &KeyCode, modifiers: Modifiers) -> bool {
        &self.key == key && self.modifiers.contains(modifiers) && modifiers.contains(self.modifiers)
    }
}

/// 命令模型。
#[derive(Debug, Clone)]
pub struct Command {
    pub action: ActionId,
    pub label_msg: CompactString,
    pub shortcut: Option<Shortcut>,
}

impl Command {
    pub fn new(action: &str, label_msg: &str) -> Command {
        Command {
            action: ActionId::new(action),
            label_msg: CompactString::new(label_msg),
            shortcut: None,
        }
    }
}

/// 命令派发器：把快捷键映射到 command → 应用 reducer（spec §8.4.1B）。
#[derive(Debug, Default)]
pub struct CommandDispatcher {
    by_shortcut: Vec<(Shortcut, ActionId)>,
}

impl CommandDispatcher {
    pub fn new() -> CommandDispatcher {
        CommandDispatcher::default()
    }
    pub fn register(&mut self, shortcut: Shortcut, action: ActionId) -> &mut CommandDispatcher {
        self.by_shortcut.push((shortcut, action));
        self
    }
    /// 查找匹配的 command。
    pub fn resolve(&self, key: &KeyCode, modifiers: Modifiers) -> Option<&ActionId> {
        self.by_shortcut
            .iter()
            .find(|(s, _)| s.matches(key, modifiers))
            .map(|(_, a)| a)
    }
    /// 派发（此处只返回是否找到；真实 reducer 由应用 ActionRegistry 执行）。
    pub fn dispatch(&self, key: &KeyCode, modifiers: Modifiers) -> ActionResult {
        match self.resolve(key, modifiers) {
            Some(_) => ActionResult::Handled,
            None => ActionResult::UnknownAction(ActionId::new("__shortcut_miss__")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_matches() {
        let s = Shortcut::new("R", Modifiers::CONTROL);
        assert!(s.matches(&KeyCode::new("R"), Modifiers::CONTROL));
        assert!(!s.matches(&KeyCode::new("R"), Modifiers::NONE));
    }

    #[test]
    fn dispatcher_resolves_command() {
        let mut d = CommandDispatcher::new();
        d.register(Shortcut::new("R", Modifiers::CONTROL), ActionId::new("browser.reload"));
        assert_eq!(
            d.resolve(&KeyCode::new("R"), Modifiers::CONTROL),
            Some(&ActionId::new("browser.reload"))
        );
        assert!(d.resolve(&KeyCode::new("X"), Modifiers::CONTROL).is_none());
    }
}
