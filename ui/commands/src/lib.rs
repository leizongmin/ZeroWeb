//! # zero-ui-commands
//!
//! 命令系统（spec §8.4.1 `zero-ui-commands` / FR-016 / IF-010 `CommandRegistry` / §8.4.1B
//! 主菜单·上下文菜单·快捷键·command palette 同一 command）。
//!
//! 所有命令注册一次为 [`Command`]（携带 `ActionId` + 显示 message id + 快捷键 + 菜单归组 +
//! palette 可见性）；菜单项、快捷键、命令面板只是不同的触发入口，最终都解析到同一个
//! command，再映射到应用 [`ActionId`](zero_ui_core::action::ActionId) 由 reducer 执行。
//!
//! 错误处理（spec §8.4.1B / IF-010）：未注册 command 返回 [`CommandResult::Unknown`]，
//! 已注册但禁用返回 [`CommandResult::Disabled`]——不静默忽略。

mod menu;

use compact_str::CompactString;
use zero_ui_core::action::ActionId;
use zero_ui_core::event::{KeyCode, Modifiers};

pub use menu::{MenuModel, MenuNode};

/// 命令标识（spec IF-010 `CommandId`）。复用 `ActionId`，使命令与单向数据流的 action 同构。
pub type CommandId = ActionId;

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

    /// 是否精确匹配给定键 + 修饰（修饰集必须完全相等，多按一个修饰键不算命中）。
    pub fn matches(&self, key: &KeyCode, modifiers: Modifiers) -> bool {
        &self.key == key && self.modifiers.contains(modifiers) && modifiers.contains(self.modifiers)
    }
}

/// 命令模型（spec IF-010 `CommandSpec`）。
///
/// 一个命令 = 唯一 [`ActionId`] + 显示文案 message id + 可选快捷键 + 菜单归组 + palette 可见性。
/// `menu_path` 是归组路径（如 `"File"`、`"View/Zoom"`），**不含**叶子显示文案——叶子文案用
/// `label_msg`（遵循「可见字符串走 message id」铁律，spec DC-10）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: ActionId,
    pub label_msg: CompactString,
    pub description_msg: Option<CompactString>,
    pub shortcut: Option<Shortcut>,
    /// 菜单归组路径（如 `"File"`）；`None` 表示不进菜单。
    pub menu_path: Option<CompactString>,
    /// 是否出现在命令面板。
    pub in_palette: bool,
    /// 当前是否启用（禁用命令仍注册，但执行返回 `Disabled`，菜单/面板灰显）。
    pub enabled: bool,
}

impl Command {
    /// 新建命令（默认无快捷键、不进菜单、进 palette、启用）。
    pub fn new(id: &str, label_msg: &str) -> Command {
        Command {
            id: ActionId::new(id),
            label_msg: CompactString::new(label_msg),
            description_msg: None,
            shortcut: None,
            menu_path: None,
            in_palette: true,
            enabled: true,
        }
    }

    pub fn shortcut(mut self, shortcut: Shortcut) -> Command {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn menu(mut self, path: &str) -> Command {
        self.menu_path = Some(CompactString::new(path));
        self
    }

    pub fn description(mut self, msg: &str) -> Command {
        self.description_msg = Some(CompactString::new(msg));
        self
    }

    /// 不进命令面板。
    pub fn hide_from_palette(mut self) -> Command {
        self.in_palette = false;
        self
    }

    pub fn disabled(mut self) -> Command {
        self.enabled = false;
        self
    }
}

/// 命令执行结果（spec IF-010 `CommandResult` / §8.4.1B 错误处理）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// 命令已注册且启用——应用 reducer 应据此 `ActionId` 派发。
    Executed(ActionId),
    /// 命令未注册（diagnostic，不静默忽略）。
    Unknown(ActionId),
    /// 命令已注册但当前禁用（graceful 降级，调用方灰显）。
    Disabled(ActionId),
}

/// 命令面板条目（§8.4.1B command palette）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteEntry {
    pub command: ActionId,
    pub label_msg: CompactString,
    pub description_msg: Option<CompactString>,
    pub shortcut: Option<Shortcut>,
}

/// 命令注册表（spec IF-010 `CommandRegistry`）。
///
/// 持有全部已注册命令；提供按 id 执行、按快捷键解析、构建菜单模型、构建/检索命令面板。
/// 实际副作用由应用 reducer 执行（`execute` 只做查找 + 启用判断，返回要派发的 `ActionId`）。
#[derive(Debug, Default)]
pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> CommandRegistry {
        CommandRegistry::default()
    }

    /// 注册一个命令。同 id 重复注册会**替换**旧定义（以最后一次为准，匹配「注册一次」语义）。
    pub fn register(&mut self, command: Command) -> &mut CommandRegistry {
        if let Some(existing) = self.commands.iter_mut().find(|c| c.id == command.id) {
            *existing = command;
        } else {
            self.commands.push(command);
        }
        self
    }

    /// 已注册命令数。
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 按 id 查命令定义。
    pub fn get(&self, id: &ActionId) -> Option<&Command> {
        self.commands.iter().find(|c| &c.id == id)
    }

    /// 执行命令：查找 + 启用判断（spec IF-010 `execute` / §8.4.1B 错误处理）。
    /// 真实副作用由应用 reducer 据 [`CommandResult::Executed`] 的 `ActionId` 派发。
    pub fn execute(&self, id: &ActionId) -> CommandResult {
        match self.get(id) {
            None => CommandResult::Unknown(id.clone()),
            Some(cmd) if !cmd.enabled => CommandResult::Disabled(id.clone()),
            Some(_) => CommandResult::Executed(id.clone()),
        }
    }

    /// 按快捷键解析到 command id（spec §8.4.1B 快捷键入口）。
    /// 多个命令绑定同一快捷键时返回第一个（注册顺序）；调用方应避免歧义绑定。
    pub fn resolve_shortcut(&self, key: &KeyCode, modifiers: Modifiers) -> Option<&ActionId> {
        self.commands
            .iter()
            .find(|c| c.shortcut.as_ref().is_some_and(|s| s.matches(key, modifiers)) && c.enabled)
            .map(|c| &c.id)
    }

    /// 构建菜单模型：把带 `menu_path` 的命令按归组路径组装成菜单树（§8.4.1B 主菜单/上下文菜单）。
    /// 顶层为各菜单（如 File/Edit/View），按首次注册顺序排列。
    pub fn menu_model(&self) -> MenuModel {
        MenuModel::build(&self.commands)
    }

    /// 命令面板条目（`in_palette=true` 的命令，按注册顺序）。
    pub fn palette_entries(&self) -> Vec<PaletteEntry> {
        self.commands
            .iter()
            .filter(|c| c.in_palette)
            .map(|c| PaletteEntry {
                command: c.id.clone(),
                label_msg: c.label_msg.clone(),
                description_msg: c.description_msg.clone(),
                shortcut: c.shortcut.clone(),
            })
            .collect()
    }

    /// 命令面板检索：label_msg / id / description 任一包含 `query`（大小写不敏感）。
    /// 空 query 返回全部 palette 条目。
    pub fn palette_search(&self, query: &str) -> Vec<PaletteEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.palette_entries();
        }
        self.palette_entries()
            .into_iter()
            .filter(|e| {
                e.label_msg.to_lowercase().contains(&q)
                    || e.command.0.to_lowercase().contains(&q)
                    || e.description_msg
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&q))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reload() -> Command {
        Command::new("browser.reload", "msg.reload")
            .shortcut(Shortcut::new("R", Modifiers::CONTROL))
            .menu("View")
            .description("msg.reload_desc")
    }

    #[test]
    fn shortcut_exact_match_only() {
        let s = Shortcut::new("R", Modifiers::CONTROL);
        assert!(s.matches(&KeyCode::new("R"), Modifiers::CONTROL));
        // 缺修饰 → 不命中。
        assert!(!s.matches(&KeyCode::new("R"), Modifiers::NONE));
        // 多余修饰 → 不命中（精确匹配）。
        assert!(!s.matches(&KeyCode::new("R"), Modifiers::CONTROL | Modifiers::SHIFT));
        // 键不同 → 不命中。
        assert!(!s.matches(&KeyCode::new("T"), Modifiers::CONTROL));
    }

    #[test]
    fn command_builder_defaults() {
        let c = Command::new("browser.reload", "msg.reload");
        assert!(c.in_palette, "default visible in palette");
        assert!(c.enabled, "default enabled");
        assert!(c.shortcut.is_none());
        assert!(c.menu_path.is_none());
        assert!(c.description_msg.is_none());

        let c2 = reload().hide_from_palette().disabled();
        assert!(!c2.in_palette);
        assert!(!c2.enabled);
        assert_eq!(c2.menu_path.as_deref(), Some("View"));
    }

    #[test]
    fn register_and_execute_outcomes() {
        let mut reg = CommandRegistry::new();
        reg.register(reload());
        reg.register(
            Command::new("browser.new_tab", "msg.new_tab")
                .shortcut(Shortcut::new("T", Modifiers::CONTROL))
                .disabled(),
        );

        // 已注册且启用 → Executed。
        assert_eq!(
            reg.execute(&ActionId::new("browser.reload")),
            CommandResult::Executed(ActionId::new("browser.reload"))
        );
        // 已注册但禁用 → Disabled（不静默，不执行）。
        assert_eq!(
            reg.execute(&ActionId::new("browser.new_tab")),
            CommandResult::Disabled(ActionId::new("browser.new_tab"))
        );
        // 未注册 → Unknown（diagnostic）。
        assert_eq!(
            reg.execute(&ActionId::new("browser.nope")),
            CommandResult::Unknown(ActionId::new("browser.nope"))
        );
    }

    #[test]
    fn duplicate_register_replaces() {
        let mut reg = CommandRegistry::new();
        reg.register(Command::new("browser.reload", "msg.reload_old"));
        reg.register(Command::new("browser.reload", "msg.reload_new"));
        assert_eq!(reg.len(), 1, "duplicate id replaces, does not append");
        assert_eq!(
            reg.get(&ActionId::new("browser.reload")).unwrap().label_msg,
            CompactString::new("msg.reload_new")
        );
    }

    #[test]
    fn resolve_shortcut_skips_disabled() {
        let mut reg = CommandRegistry::new();
        reg.register(reload()); // Ctrl+R enabled
        reg.register(
            Command::new("browser.new_tab", "msg.new_tab")
                .shortcut(Shortcut::new("T", Modifiers::CONTROL))
                .disabled(),
        );
        // Ctrl+R → reload。
        assert_eq!(
            reg.resolve_shortcut(&KeyCode::new("R"), Modifiers::CONTROL),
            Some(&ActionId::new("browser.reload"))
        );
        // Ctrl+T 命中的命令被禁用 → 不解析（快捷键不应触发禁用命令）。
        assert!(reg.resolve_shortcut(&KeyCode::new("T"), Modifiers::CONTROL).is_none());
        // 未绑定 → None。
        assert!(reg.resolve_shortcut(&KeyCode::new("X"), Modifiers::CONTROL).is_none());
    }

    #[test]
    fn menu_model_groups_by_path() {
        let mut reg = CommandRegistry::new();
        reg.register(
            Command::new("browser.new_tab", "msg.new_tab")
                .menu("File")
                .shortcut(Shortcut::new("T", Modifiers::CONTROL)),
        );
        reg.register(Command::new("browser.close_tab", "msg.close_tab").menu("File"));
        reg.register(reload().menu("View")); // browser.reload → View
        reg.register(Command::new("browser.find", "msg.find")); // 无 menu_path → 不进菜单

        let m = reg.menu_model();
        // 顶层菜单：File、View（按首次注册顺序）。
        let top_labels: Vec<&str> = m.top_level_labels();
        assert_eq!(top_labels, vec!["File", "View"]);

        // File 下两条叶子（new_tab / close_tab），按注册顺序。
        let file_items = m.items_in("File");
        assert_eq!(file_items.len(), 2);
        assert_eq!(file_items[0].0.as_str(), "msg.new_tab");
        assert_eq!(file_items[0].1, ActionId::new("browser.new_tab"));
        assert_eq!(file_items[1].0.as_str(), "msg.close_tab");

        // View 下一条（reload）。
        let view_items = m.items_in("View");
        assert_eq!(view_items.len(), 1);
        assert_eq!(view_items[0].1, ActionId::new("browser.reload"));

        // find 不在任何菜单。
        assert!(m.items_in("Edit").is_empty());
    }

    #[test]
    fn menu_model_supports_nested_groups() {
        // menu_path "View/Zoom" → View > Zoom > leaf。
        let mut reg = CommandRegistry::new();
        reg.register(
            Command::new("browser.zoom_in", "msg.zoom_in")
                .menu("View/Zoom")
                .shortcut(Shortcut::new("Equal", Modifiers::CONTROL)),
        );
        reg.register(Command::new("browser.zoom_out", "msg.zoom_out").menu("View/Zoom"));
        reg.register(Command::new("browser.reload", "msg.reload").menu("View"));

        let m = reg.menu_model();
        let view = m.group_in("View").expect("View group");
        let view_children = match view {
            MenuNode::Group { children, .. } => children,
            _ => panic!("View is a Group"),
        };
        // View 直接子项：Zoom 子组 + reload 叶子。
        let zoom = view_children
            .iter()
            .find(|n| matches!(n, MenuNode::Group { label, .. } if label.as_str() == "Zoom"))
            .expect("nested Zoom subgroup present");
        let has_reload = view_children.iter().any(|n| {
            if let MenuNode::Item { command, .. } = n {
                command == &ActionId::new("browser.reload")
            } else {
                false
            }
        });
        assert!(has_reload, "reload leaf under View");

        // Zoom 子组含两条叶子。
        let zoom_children = match zoom {
            MenuNode::Group { children, .. } => children,
            _ => panic!("Zoom is a Group"),
        };
        assert_eq!(zoom_children.len(), 2);
    }

    #[test]
    fn palette_entries_and_search() {
        let mut reg = CommandRegistry::new();
        reg.register(reload()); // label "msg.reload", in_palette default true
        reg.register(
            Command::new("browser.new_tab", "msg.new_tab")
                .description("msg.new_tab_desc")
                .hide_from_palette(),
        );
        reg.register(Command::new("browser.open_history", "msg.history").description("msg.history_desc"));

        // palette 只含 in_palette=true：reload + history（new_tab 被隐藏）。
        let entries = reg.palette_entries();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.command == ActionId::new("browser.reload")));
        assert!(
            entries
                .iter()
                .any(|e| e.command == ActionId::new("browser.open_history"))
        );

        // search "history" 命中 history（label msg.history / desc）。
        let hist = reg.palette_search("history");
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].command, ActionId::new("browser.open_history"));

        // search "reload" 命中 reload（id browser.reload 含 "reload"）。
        let rel = reg.palette_search("reload");
        assert!(rel.iter().any(|e| e.command == ActionId::new("browser.reload")));

        // 大小写不敏感。
        assert_eq!(reg.palette_search("HISTORY").len(), 1);
        // 空 query → 全部 palette 条目。
        assert_eq!(reg.palette_search("").len(), 2);
        // 无命中。
        assert!(reg.palette_search("nonexistent_xyz").is_empty());
    }

    #[test]
    fn same_command_triggered_from_menu_shortcut_and_palette() {
        // §8.4.1B 验收：同一 reload command 可由菜单/快捷键/command palette 触发。
        let mut reg = CommandRegistry::new();
        reg.register(reload()); // browser.reload, Ctrl+R, menu View, in_palette

        let from_shortcut = reg
            .resolve_shortcut(&KeyCode::new("R"), Modifiers::CONTROL)
            .expect("shortcut resolves to reload");
        let from_menu = reg
            .menu_model()
            .items_in("View")
            .iter()
            .find(|(_, cmd)| cmd == &ActionId::new("browser.reload"))
            .map(|(_, cmd)| cmd.clone())
            .expect("menu has reload item");
        let from_palette = reg
            .palette_entries()
            .iter()
            .find(|e| e.command == ActionId::new("browser.reload"))
            .map(|e| e.command.clone())
            .expect("palette has reload entry");

        // 三处入口都指向同一 command id。
        assert_eq!(from_shortcut, &ActionId::new("browser.reload"));
        assert_eq!(from_menu, ActionId::new("browser.reload"));
        assert_eq!(from_palette, ActionId::new("browser.reload"));

        // 任一入口 execute 都得到 Executed。
        assert_eq!(
            reg.execute(&ActionId::new("browser.reload")),
            CommandResult::Executed(ActionId::new("browser.reload"))
        );
    }
}
