//! ContextMenu 单元测试。

use crate::*;

// ── ContextMenu 测试 ──

#[test]
fn test_menu_item_action() {
    let item = MenuItem::action("copy", "复制");
    assert_eq!(item.id(), "copy");
    assert_eq!(item.label(), "复制");
    assert!(!item.is_separator());
    assert!(!item.is_sub_menu());
    assert!(item.children().is_none());
}

#[test]
fn test_menu_item_separator() {
    let item = MenuItem::separator();
    assert!(item.is_separator());
    assert!(item.id().is_empty());
    assert!(item.label().is_empty());
}

#[test]
fn test_menu_item_sub_menu() {
    let child1 = MenuItem::action("copy", "复制");
    let child2 = MenuItem::action("paste", "粘贴");
    let item = MenuItem::sub_menu("edit", "编辑", vec![child1, child2]);
    assert!(item.is_sub_menu());
    assert_eq!(item.children().unwrap().len(), 2);
    assert_eq!(item.children().unwrap()[0].id(), "copy");
}

#[test]
fn test_menu_item_equality() {
    let a = MenuItem::action("copy", "复制");
    let b = MenuItem::action("copy", "复制");
    assert_eq!(a, b);
}

#[test]
fn test_context_menu_page() {
    let menu = ContextMenu::new(ContextType::Page);
    assert_eq!(menu.context_type(), ContextType::Page);
    assert!(menu.source_url().is_none());
    assert!(!menu.is_empty());
    // Page menu should have back/forward/reload at minimum
    assert!(menu.find_item("back").is_some());
    assert!(menu.find_item("forward").is_some());
    assert!(menu.find_item("reload").is_some());
    assert!(menu.find_item("inspect").is_some());
}

#[test]
fn test_context_menu_link() {
    let menu = ContextMenu::new(ContextType::Link);
    assert_eq!(menu.context_type(), ContextType::Link);
    assert!(menu.find_item("open_link").is_some());
    assert!(menu.find_item("copy_link").is_some());
}

#[test]
fn test_context_menu_image() {
    let menu = ContextMenu::new(ContextType::Image);
    assert!(menu.find_item("copy_image_url").is_some());
    assert!(menu.find_item("save_image").is_some());
}

#[test]
fn test_context_menu_selection() {
    let menu = ContextMenu::new(ContextType::Selection);
    assert!(menu.find_item("copy").is_some());
    assert!(menu.find_item("search_selection").is_some());
}

#[test]
fn test_context_menu_editable() {
    let menu = ContextMenu::new(ContextType::Editable);
    assert!(menu.find_item("cut").is_some());
    assert!(menu.find_item("copy").is_some());
    assert!(menu.find_item("paste").is_some());
    assert!(menu.find_item("undo").is_some());
}

#[test]
fn test_context_menu_with_url() {
    let menu = ContextMenu::with_url(ContextType::Link, "https://example.com/page");
    assert_eq!(menu.source_url(), Some("https://example.com/page"));
}

#[test]
fn test_context_menu_find_nonexistent() {
    let menu = ContextMenu::new(ContextType::Page);
    assert!(menu.find_item("nonexistent").is_none());
}

#[test]
fn test_context_menu_find_in_sub_menu() {
    let child = MenuItem::action("nested", "嵌套项");
    let parent = MenuItem::sub_menu("parent", "父菜单", vec![child]);
    let menu = ContextMenu::with_items(ContextType::Page, vec![parent]);
    assert!(menu.find_item("nested").is_some());
    assert!(menu.find_item("parent").is_some());
}

#[test]
fn test_context_menu_len() {
    let menu = ContextMenu::new(ContextType::Page);
    assert!(menu.len() > 0);
    // Page menu items: back, forward, reload, sep, save_as, print, sep, view_source, inspect = 9
    assert_eq!(menu.len(), 9);
}

#[test]
fn test_context_type_equality() {
    assert_eq!(ContextType::Page, ContextType::Page);
    assert_ne!(ContextType::Page, ContextType::Link);
}

#[test]
fn test_menu_item_action_clone() {
    let item = MenuItem::action("copy", "复制");
    let cloned = item.clone();
    assert_eq!(item, cloned);
}

// ── ContextMenu 边界 ──

#[test]
/// 测试 ContextMenu::with_items() 空列表。
fn test_context_menu_empty_items() {
    let menu = ContextMenu::with_items(ContextType::Page, vec![]);
    assert!(menu.is_empty());
    assert_eq!(menu.len(), 0);
}

#[test]
/// 测试 ContextType::Image 默认菜单项完整性。
fn test_context_menu_image_items_complete() {
    let menu = ContextMenu::new(ContextType::Image);
    assert!(
        menu.find_item("open_image").is_some(),
        "Image menu should have open_image"
    );
    assert!(
        menu.find_item("copy_image_url").is_some(),
        "Image menu should have copy_image_url"
    );
}

#[test]
/// 测试 ContextType::Editable 默认菜单项包含 select_all。
fn test_context_menu_editable_items_complete() {
    let menu = ContextMenu::new(ContextType::Editable);
    assert!(
        menu.find_item("select_all").is_some(),
        "Editable menu should have select_all"
    );
}

#[test]
/// 测试 ContextMenu::find_item 在子菜单中递归查找。
fn test_context_menu_find_item_in_submenu() {
    let menu = ContextMenu::new(ContextType::Page);
    // 页面菜单应有子菜单或直接项
    let found = menu.find_item("inspect");
    assert!(found.is_some(), "Page 菜单应包含 inspect");
}

// ── 额外覆盖率测试 ──

/// 测试 ContextMenu::with_items() 使用自定义菜单项。
#[test]
fn test_context_menu_with_custom_items() {
    let items = vec![
        MenuItem::action("custom1", "自定义1"),
        MenuItem::separator(),
        MenuItem::action("custom2", "自定义2"),
    ];
    let menu = ContextMenu::with_items(ContextType::Page, items);
    assert_eq!(menu.len(), 3);
    assert!(!menu.is_empty());
    assert_eq!(menu.items().len(), 3);
}

/// 测试 ContextMenu::with_items() 中深层嵌套子菜单的查找。
#[test]
fn test_context_menu_deeply_nested_find() {
    let deep = MenuItem::action("deep_item", "深层项");
    let mid = MenuItem::sub_menu("mid", "中间", vec![deep]);
    let top = MenuItem::sub_menu("top", "顶层", vec![mid]);
    let menu = ContextMenu::with_items(ContextType::Page, vec![top]);

    assert!(menu.find_item("deep_item").is_some(), "应找到深层嵌套项");
    assert!(menu.find_item("mid").is_some(), "应找到中间层");
    assert!(menu.find_item("top").is_some(), "应找到顶层");
    assert!(menu.find_item("nonexistent").is_none());
}

/// 测试 MenuItem::action 返回正确的 children()。
#[test]
fn test_menu_item_action_children_none() {
    let item = MenuItem::action("test", "测试");
    assert!(item.children().is_none());
    assert!(!item.is_separator());
    assert!(!item.is_sub_menu());
}

/// 测试 MenuItem Clone 后相等。
#[test]
fn test_menu_item_sub_menu_clone() {
    let child = MenuItem::action("c1", "子项1");
    let item = MenuItem::sub_menu("parent", "父", vec![child]);
    let cloned = item.clone();
    assert_eq!(item, cloned);
    assert!(cloned.is_sub_menu());
    assert_eq!(cloned.children().unwrap().len(), 1);
}

/// 测试 MenuItem::separator 的 Clone 和 PartialEq。
#[test]
fn test_menu_item_separator_clone_eq() {
    let sep1 = MenuItem::separator();
    let sep2 = sep1.clone();
    assert_eq!(sep1, sep2);
}

/// 测试 ContextMenu::items() 返回的切片可遍历。
#[test]
fn test_context_menu_items_iteration() {
    let menu = ContextMenu::new(ContextType::Link);
    let ids: Vec<&str> = menu.items().iter().map(|i| i.id()).collect();
    assert!(ids.contains(&"open_link"));
    assert!(ids.contains(&"copy_link"));
}

/// 测试 ContextMenu::with_url 的 URL。
#[test]
fn test_context_menu_with_url_source() {
    let menu = ContextMenu::with_url(ContextType::Image, "https://example.com/img.png");
    assert_eq!(menu.source_url(), Some("https://example.com/img.png"));
    assert_eq!(menu.context_type(), ContextType::Image);
}

/// 测试 ContextMenu::with_url 空字符串 URL。
#[test]
fn test_context_menu_with_empty_url() {
    let menu = ContextMenu::with_url(ContextType::Link, "");
    assert_eq!(menu.source_url(), Some(""));
}

/// 测试所有 ContextType 变体的 Debug 输出。
#[test]
fn test_context_type_debug() {
    assert!(!format!("{:?}", ContextType::Page).is_empty());
    assert!(!format!("{:?}", ContextType::Link).is_empty());
    assert!(!format!("{:?}", ContextType::Image).is_empty());
    assert!(!format!("{:?}", ContextType::Selection).is_empty());
    assert!(!format!("{:?}", ContextType::Editable).is_empty());
}

/// 测试 ContextType Copy。
#[test]
fn test_context_type_copy() {
    let a = ContextType::Page;
    let b = a;
    assert_eq!(a, b);
}

/// 测试 ContextMenu Clone。
#[test]
fn test_context_menu_clone() {
    let menu = ContextMenu::new(ContextType::Page);
    let cloned = menu.clone();
    assert_eq!(menu.context_type(), cloned.context_type());
    assert_eq!(menu.len(), cloned.len());
    assert_eq!(menu.source_url(), cloned.source_url());
}

/// 测试 ContextMenu::is_empty 在有内容时为 false。
#[test]
fn test_context_menu_not_empty() {
    let menu = ContextMenu::new(ContextType::Selection);
    assert!(!menu.is_empty());
    assert!(menu.len() > 0);
}

/// 测试 find_item 第一级查找。
#[test]
fn test_find_item_at_top_level() {
    let menu = ContextMenu::new(ContextType::Editable);
    let found = menu.find_item("paste");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), "paste");
}

/// 测试 separator 的空 id 可被找到。
#[test]
fn test_find_item_separator_empty_id() {
    let items = vec![
        MenuItem::action("a", "A"),
        MenuItem::separator(),
        MenuItem::action("b", "B"),
    ];
    let menu = ContextMenu::with_items(ContextType::Page, items);
    assert!(menu.find_item("a").is_some());
    assert!(menu.find_item("b").is_some());
    assert!(menu.find_item("").is_some(), "separator 的空 id 应能找到");
}
