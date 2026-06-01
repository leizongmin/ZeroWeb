//! 右键上下文菜单 — 页面右键菜单的数据模型。
//!
//! 提供不同场景下的菜单项定义，由 UI 层消费渲染。

/// 上下文菜单项。
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    /// 菜单项 ID。
    id: String,
    /// 显示文本。
    label: String,
    /// 菜单项类型。
    item_type: MenuItemType,
}

/// 菜单项类型。
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemType {
    /// 普通可点击项。
    Action,
    /// 分隔线。
    Separator,
    /// 子菜单（包含子项）。
    SubMenu(Vec<MenuItem>),
}

impl MenuItem {
    /// 创建可点击菜单项。
    pub fn action(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type: MenuItemType::Action,
        }
    }

    /// 创建分隔线。
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            item_type: MenuItemType::Separator,
        }
    }

    /// 创建子菜单。
    pub fn sub_menu(id: &str, label: &str, children: Vec<MenuItem>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type: MenuItemType::SubMenu(children),
        }
    }

    /// 获取菜单项 ID。
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取显示文本。
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 是否为分隔线。
    pub fn is_separator(&self) -> bool {
        self.item_type == MenuItemType::Separator
    }

    /// 是否为子菜单。
    pub fn is_sub_menu(&self) -> bool {
        matches!(self.item_type, MenuItemType::SubMenu(_))
    }

    /// 获取子菜单项（如果不是子菜单返回 None）。
    pub fn children(&self) -> Option<&[MenuItem]> {
        match &self.item_type {
            MenuItemType::SubMenu(children) => Some(children),
            _ => None,
        }
    }
}

/// 上下文场景类型 — 决定显示哪些菜单项。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// 页面空白区域。
    Page,
    /// 链接。
    Link,
    /// 图片。
    Image,
    /// 文本选区。
    Selection,
    /// 输入框 / 可编辑区域。
    Editable,
}

/// 右键上下文菜单。
#[derive(Debug, Clone)]
pub struct ContextMenu {
    /// 触发场景。
    context_type: ContextType,
    /// 菜单项列表。
    items: Vec<MenuItem>,
    /// 关联的 URL（链接/图片场景）。
    source_url: Option<String>,
}

impl ContextMenu {
    /// 为指定场景创建默认上下文菜单。
    pub fn new(context_type: ContextType) -> Self {
        let items = default_items_for_context(context_type);
        Self {
            context_type,
            items,
            source_url: None,
        }
    }

    /// 创建带关联 URL 的上下文菜单。
    pub fn with_url(context_type: ContextType, url: &str) -> Self {
        let items = default_items_for_context(context_type);
        Self {
            context_type,
            items,
            source_url: Some(url.to_string()),
        }
    }

    /// 创建使用自定义菜单项的上下文菜单。
    pub fn with_items(context_type: ContextType, items: Vec<MenuItem>) -> Self {
        Self {
            context_type,
            items,
            source_url: None,
        }
    }

    /// 获取触发场景。
    pub fn context_type(&self) -> ContextType {
        self.context_type
    }

    /// 获取菜单项列表。
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// 获取关联 URL。
    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    /// 当前菜单项数量。
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空菜单。
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 查找指定 ID 的菜单项。
    pub fn find_item(&self, id: &str) -> Option<&MenuItem> {
        find_item_recursive(&self.items, id)
    }
}

/// 递归查找菜单项。
fn find_item_recursive<'a>(items: &'a [MenuItem], id: &str) -> Option<&'a MenuItem> {
    for item in items {
        if item.id == id {
            return Some(item);
        }
        if let Some(children) = item.children()
            && let Some(found) = find_item_recursive(children, id)
        {
            return Some(found);
        }
    }
    None
}

/// 根据场景生成默认菜单项。
fn default_items_for_context(context_type: ContextType) -> Vec<MenuItem> {
    match context_type {
        ContextType::Page => vec![
            MenuItem::action("back", "后退"),
            MenuItem::action("forward", "前进"),
            MenuItem::action("reload", "重新加载"),
            MenuItem::separator(),
            MenuItem::action("save_as", "另存为..."),
            MenuItem::action("print", "打印..."),
            MenuItem::separator(),
            MenuItem::action("view_source", "查看源代码"),
            MenuItem::action("inspect", "检查元素"),
        ],
        ContextType::Link => vec![
            MenuItem::action("open_link", "在新标签页中打开链接"),
            MenuItem::action("copy_link", "复制链接地址"),
            MenuItem::separator(),
            MenuItem::action("save_link", "将链接另存为..."),
            MenuItem::action("bookmark_link", "将链接添加为书签"),
        ],
        ContextType::Image => vec![
            MenuItem::action("open_image", "在新标签页中打开图片"),
            MenuItem::action("copy_image_url", "复制图片地址"),
            MenuItem::action("save_image", "将图片另存为..."),
            MenuItem::separator(),
            MenuItem::action("copy_image", "复制图片"),
        ],
        ContextType::Selection => vec![
            MenuItem::action("copy", "复制"),
            MenuItem::action("search_selection", "使用搜索引擎搜索"),
            MenuItem::separator(),
            MenuItem::action("print", "打印..."),
        ],
        ContextType::Editable => vec![
            MenuItem::action("cut", "剪切"),
            MenuItem::action("copy", "复制"),
            MenuItem::action("paste", "粘贴"),
            MenuItem::action("select_all", "全选"),
            MenuItem::separator(),
            MenuItem::action("undo", "撤销"),
            MenuItem::action("redo", "重做"),
        ],
    }
}
