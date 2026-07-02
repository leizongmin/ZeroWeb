// BrowserApp 相关状态类型定义（ContentPointerDrag / ScrollbarDrag / TabFetchState /
// WindowChromeAction / TabDragState / AutocompleteState / ContextMenuState）。
// 从 app.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs 等同模式（pub/私有字段、impl 直接可达，无可见性变化）。

/// 页面内容区指针拖拽（鼠标左键；RDP/远程桌面触摸常模拟为此路径）
struct ContentPointerDrag {
    start_x: f64,
    start_y: f64,
    last_y: f64,
    scrolling: bool,
}

/// 滚动条滑块拖拽。
#[derive(Clone, Copy)]
struct ScrollbarDrag {
    tab_id: TabId,
    axis: page_scroll::ScrollbarAxis,
    grab_offset: f32,
}

/// 标签页 URL 加载状态（先绘制 loading，再发起 worker 加载）。
enum TabFetchState {
    None,
    WaitingPaint(TabId, String),
}

/// 自定义窗口控制按钮动作（Wayland 无系统装饰时使用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowChromeAction {
    Minimize,
    ToggleMaximize,
    ToggleFullscreen,
    Close,
    StartDrag,
}

/// 标签拖拽状态。鼠标按下标签后，移动超过阈值即进入拖拽，
/// 释放时按位置重排序。
#[derive(Debug, Clone, Copy)]
pub struct TabDragState {
    pub tab_id: TabId,
    /// 鼠标按下时的物理 x 坐标。
    pub press_x: f32,
    /// 标签在按下时左边缘的物理 x 坐标。
    pub tab_origin_x: f32,
    /// 标签宽度（按下时记录）。
    pub tab_w: f32,
    /// 当前鼠标物理 x 坐标。
    pub current_x: f32,
    /// 是否已越过拖拽阈值（进入实际拖拽）。
    pub active: bool,
}

/// 自动补全建议缓存
struct AutocompleteState {
    /// 当前显示的建议列表
    suggestions: Vec<zero_browser_shell::Suggestion>,
    /// 鼠标悬停的索引
    hovered_index: Option<usize>,
    /// 键盘选中的索引
    selected_index: Option<usize>,
}

impl AutocompleteState {
    fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            hovered_index: None,
            selected_index: None,
        }
    }

    fn clear(&mut self) {
        self.suggestions.clear();
        self.hovered_index = None;
        self.selected_index = None;
    }

    fn highlight_index(&self) -> Option<usize> {
        self.hovered_index.or(self.selected_index)
    }
}

/// 右键上下文菜单状态
pub struct ContextMenuState {
    /// 是否显示
    pub visible: bool,
    /// 菜单类型（预留用于区分不同场景的菜单行为）
    #[allow(dead_code)]
    pub context_type: ContextType,
    /// 菜单项（消费 browser-shell 的 MenuItem 模型，含 separator/disabled/icon/submenu）。
    pub items: Vec<zero_browser_shell::MenuItem>,
    /// 悬停索引
    pub hovered_index: Option<usize>,
    /// 当前展开的子菜单父项索引（None 表示无子菜单展开）。
    pub open_sub_menu: Option<usize>,
    /// 子菜单内悬停的子项索引。
    pub sub_menu_hovered: Option<usize>,
    /// 菜单左上角物理像素坐标
    pub x: f32,
    pub y: f32,
    /// 打开菜单时的源标签页。
    pub source_tab_id: Option<TabId>,
    /// 页面内容区文档坐标（审查元素用）。
    pub page_doc_x: f32,
    pub page_doc_y: f32,
    /// 书签栏右键菜单的目标书签 URL（仅书签上下文菜单使用）。
    pub bookmark_url: Option<String>,
    /// 书签栏右键菜单的目标书签标题（仅书签上下文菜单使用）。
    pub bookmark_title: Option<String>,
    /// 图片右键菜单的目标图片 URL（绝对化后的 src）。
    pub image_url: Option<String>,
    /// 链接右键菜单的目标链接 URL（绝对化后的 href）。
    pub link_url: Option<String>,
}

impl ContextMenuState {
    fn new() -> Self {
        Self {
            visible: false,
            context_type: ContextType::Page,
            items: Vec::new(),
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: 0.0,
            y: 0.0,
            source_tab_id: None,
            page_doc_x: 0.0,
            page_doc_y: 0.0,
            bookmark_url: None,
            bookmark_title: None,
            image_url: None,
            link_url: None,
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.items.clear();
        self.hovered_index = None;
        self.open_sub_menu = None;
        self.sub_menu_hovered = None;
        self.source_tab_id = None;
        self.bookmark_url = None;
        self.bookmark_title = None;
        self.image_url = None;
        self.link_url = None;
    }
}
