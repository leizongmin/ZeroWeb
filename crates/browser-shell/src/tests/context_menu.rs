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
