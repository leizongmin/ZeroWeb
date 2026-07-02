//! 菜单模型（spec §8.4.1B 主菜单/上下文菜单 — 由注册命令的 `menu_path` 归组构建）。
//!
//! 命令的 `menu_path` 是归组路径（如 `"File"`、`"View/Zoom"`），叶子显示文案取命令的
//! `label_msg`（message id）。`MenuModel::build` 把带 `menu_path` 的命令组装成树。

use compact_str::CompactString;
use zero_ui_core::action::ActionId;

use crate::Command;

/// 菜单树节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuNode {
    /// 子菜单（归组），按注册顺序保留子节点。
    Group {
        label: CompactString,
        children: Vec<MenuNode>,
    },
    /// 叶子命令项；`label` 为显示文案 message id，`command` 为要派发的 ActionId。
    Item {
        label: CompactString,
        command: ActionId,
        enabled: bool,
    },
}

impl MenuNode {
    /// 叶子项的便捷构造（测试/宿主手动构建菜单用）。
    pub fn item(label: &str, command: &str) -> MenuNode {
        MenuNode::Item {
            label: CompactString::new(label),
            command: ActionId::new(command),
            enabled: true,
        }
    }
}

/// 完整菜单模型（顶层 = 各主菜单 Group）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenuModel {
    pub roots: Vec<MenuNode>,
}

impl MenuModel {
    pub fn new() -> MenuModel {
        MenuModel::default()
    }

    /// 从命令列表构建菜单树（仅 `menu_path` 为 Some 的命令入菜单）。
    pub fn build(commands: &[Command]) -> MenuModel {
        let mut model = MenuModel::new();
        for cmd in commands {
            let Some(path) = cmd.menu_path.as_deref() else {
                continue;
            };
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if segments.is_empty() {
                continue;
            }
            // 前 n-1 段为归组路径，最后一段也是归组（叶子 = 命令本身，label 用 label_msg）。
            // 即 "View/Zoom" 表示叶子在 View > Zoom 下；叶子 label = 命令 label_msg。
            let leaf = MenuNode::Item {
                label: cmd.label_msg.clone(),
                command: cmd.id.clone(),
                enabled: cmd.enabled,
            };
            insert_path(&mut model.roots, &segments, leaf);
        }
        model
    }

    /// 顶层各菜单的 label（按首次出现顺序）。
    pub fn top_level_labels(&self) -> Vec<&str> {
        self.roots
            .iter()
            .filter_map(|n| match n {
                MenuNode::Group { label, .. } => Some(label.as_str()),
                _ => None,
            })
            .collect()
    }

    /// 取指定顶层菜单下的直接叶子项 `(label, command)`（不含嵌套子组）。
    pub fn items_in(&self, top: &str) -> Vec<(CompactString, ActionId)> {
        let Some(MenuNode::Group { children, .. }) = self.group_in(top) else {
            return Vec::new();
        };
        children
            .iter()
            .filter_map(|n| match n {
                MenuNode::Item { label, command, .. } => Some((label.clone(), command.clone())),
                _ => None,
            })
            .collect()
    }

    /// 取指定顶层菜单 Group 节点（只读）。
    pub fn group_in(&self, top: &str) -> Option<&MenuNode> {
        self.roots.iter().find(|n| match n {
            MenuNode::Group { label, .. } => label.as_str() == top,
            _ => false,
        })
    }
}

/// 把叶子沿 `segments` 归组路径插入 `roots`（已有同路径归组则复用，保持注册顺序）。
fn insert_path(roots: &mut Vec<MenuNode>, segments: &[&str], leaf: MenuNode) {
    let mut current = roots;
    for seg in segments {
        let idx = current
            .iter()
            .position(|n| matches!(n, MenuNode::Group { label, .. } if label.as_str() == *seg));
        match idx {
            Some(i) => match &mut current[i] {
                MenuNode::Group { children, .. } => current = children,
                _ => return,
            },
            None => {
                current.push(MenuNode::Group {
                    label: CompactString::new(seg),
                    children: Vec::new(),
                });
                let last = current.last_mut().expect("just pushed");
                match last {
                    MenuNode::Group { children, .. } => current = children,
                    _ => return,
                }
            }
        }
    }
    current.push(leaf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_groups_leaves_under_paths() {
        let cmds = vec![
            Command::new("a.new", "msg.new").menu("File"),
            Command::new("a.open", "msg.open").menu("File"),
            Command::new("a.zoom_in", "msg.zoom_in").menu("View/Zoom"),
            Command::new("a.no_menu", "msg.x"), // 不入菜单
        ];
        let m = MenuModel::build(&cmds);
        assert_eq!(m.top_level_labels(), vec!["File", "View"]);
        assert_eq!(m.items_in("File").len(), 2);
        // View 顶层直接叶子为空（只有 Zoom 子组）。
        assert!(m.items_in("View").is_empty());
        let view = m.group_in("View").expect("View");
        let view_children = match view {
            MenuNode::Group { children, .. } => children,
            _ => panic!("View is a Group"),
        };
        let zoom_children = view_children
            .iter()
            .find_map(|n| match n {
                MenuNode::Group { label, children } if label.as_str() == "Zoom" => Some(children),
                _ => None,
            })
            .expect("Zoom subgroup");
        assert_eq!(zoom_children.len(), 1);
    }

    #[test]
    fn leaf_carries_enabled_flag() {
        let cmds = vec![
            Command::new("a.disabled", "msg.d").menu("File").disabled(),
            Command::new("a.enabled", "msg.e").menu("File"),
        ];
        let m = MenuModel::build(&cmds);
        let file = m.group_in("File").expect("File");
        let file_children = match file {
            MenuNode::Group { children, .. } => children,
            _ => panic!("File is a Group"),
        };
        let disabled_item = file_children
            .iter()
            .find_map(|n| match n {
                MenuNode::Item { command, enabled, .. } if command == &ActionId::new("a.disabled") => Some(*enabled),
                _ => None,
            })
            .unwrap();
        assert!(!disabled_item, "disabled command leaf carries enabled=false");
    }

    #[test]
    fn item_constructor() {
        let n = MenuNode::item("msg.label", "browser.x");
        match n {
            MenuNode::Item {
                label,
                command,
                enabled,
            } => {
                assert_eq!(label.as_str(), "msg.label");
                assert_eq!(command, ActionId::new("browser.x"));
                assert!(enabled);
            }
            _ => panic!("expected Item"),
        }
    }

    #[test]
    fn empty_segments_skipped() {
        // menu_path "//" 全空段 → 不入菜单。
        let mut c = Command::new("a.x", "msg.x");
        c.menu_path = Some(CompactString::new("//"));
        let m = MenuModel::build(&[c]);
        assert!(m.roots.is_empty());
    }
}
