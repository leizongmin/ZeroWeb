//! 浏览器 UI 布局常量

/// Chrome 工具栏/标签统一字号（逻辑像素）
pub const CHROME_FONT_SIZE: f32 = 13.0;
/// 标签栏高度
pub const TAB_BAR_HEIGHT: f32 = 32.0;
/// 标签栏顶部距窗口顶端的间距（标签圆角与窗口顶边留白）
pub const TAB_BAR_TOP_INSET: f32 = 4.0;
/// 标签栏区域总高度（含顶部间距）
pub const TAB_STRIP_HEIGHT: f32 = TAB_BAR_TOP_INSET + TAB_BAR_HEIGHT;
/// 地址栏高度
pub const ADDRESS_BAR_HEIGHT: f32 = 40.0;
/// 地址栏内边距
pub const ADDRESS_BAR_PADDING: f32 = 8.0;
/// 地址栏行内胶囊输入框上下留白（相对地址栏行）
pub const ADDRESS_BAR_INPUT_V_INSET: f32 = 4.0;
/// 地址栏胶囊内文字上下留白
pub const ADDRESS_BAR_TEXT_V_PAD: f32 = 2.0;
/// 工具栏总高度（标签栏含顶部间距 + 地址栏）
pub const TOOLBAR_HEIGHT: f32 = TAB_STRIP_HEIGHT + ADDRESS_BAR_HEIGHT;
/// 新建标签按钮宽度
pub const NEW_TAB_BTN_WIDTH: f32 = 32.0;
/// 窗口控制按钮宽度（最小化/最大化/关闭）
pub const WINDOW_CONTROL_BTN_WIDTH: f32 = 46.0;
/// 窗口控制按钮区域总宽度
pub const WINDOW_CONTROLS_WIDTH: f32 = WINDOW_CONTROL_BTN_WIDTH * 3.0;
/// 导航按钮宽度
pub const NAV_BUTTON_WIDTH: f32 = 32.0;
/// 单个标签最小宽度
pub const TAB_MIN_WIDTH: f32 = 100.0;
/// 单个标签最大宽度
pub const TAB_MAX_WIDTH: f32 = 240.0;
/// 标签关闭按钮大小
pub const TAB_CLOSE_SIZE: f32 = 16.0;
/// 标签页顶部圆角半径
pub const TAB_TOP_RADIUS: f32 = 7.0;
/// 激活标签底部曲线半径（Chrome 风格二次贝塞尔「脚」）
pub const TAB_FOOT_RADIUS: f32 = 7.0;
/// 标签页内图标边长
pub const TAB_ICON_SIZE: f32 = 14.0;
/// 相邻非激活标签之间的竖线分隔，距标签栏上下边的内边距
pub const TAB_SEPARATOR_INSET: f32 = 8.0;
/// 自动补全下拉最大显示条数
pub const AUTOCOMPLETE_MAX_VISIBLE: usize = 6;
/// 自动补全下拉行高
pub const AUTOCOMPLETE_ROW_HEIGHT: f32 = 28.0;
/// 书签栏高度
pub const BOOKMARKS_BAR_HEIGHT: f32 = 26.0;
/// macOS 一体化标题栏：为系统 traffic lights 预留的左侧间距
pub const MACOS_TRAFFIC_LIGHT_INSET: f32 = 78.0;
/// 查找栏高度
pub const FIND_BAR_HEIGHT: f32 = 36.0;
/// 链接悬停浮动状态栏高度
pub const STATUS_BAR_HEIGHT: f32 = 22.0;
/// 浮动状态栏距页面内容区边缘的外边距
pub const STATUS_BAR_FLOAT_MARGIN: f32 = 8.0;
/// 浮动状态栏文本左右内边距
pub const STATUS_BAR_FLOAT_PAD_H: f32 = 8.0;
/// 浮动状态栏圆角半径
pub const STATUS_BAR_FLOAT_RADIUS: f32 = 3.0;
/// 下载栏高度（有活跃下载时显示）
pub const DOWNLOAD_BAR_HEIGHT: f32 = 28.0;
/// 页面视口相对 chrome 的水平内边距（与 [`PAGE_FRAME_INSET_TOP`] 一致）
pub const PAGE_FRAME_INSET_H: f32 = 4.0;
/// 页面视口距书签栏的下间距
pub const PAGE_FRAME_INSET_TOP: f32 = 4.0;
/// 页面视口距窗口底 clip guard 的下间距（与顶部 gutter 对齐）
pub const PAGE_FRAME_INSET_BOTTOM: f32 = 4.0;
/// 视口底部额外留白：Windows/WSLg **最大化**时 `inner_size` 常大于可见客户区，
/// 若不预留此空间，视口下缘圆角与底边会被裁到屏幕外（侧边框看似直通窗口底）。
/// 非最大化窗口不使用此值。
pub const PAGE_FRAME_BOTTOM_CLIP_GUARD: f32 = 24.0;
/// 最大化时额外 UI 预留（原常驻状态栏高度）：与 [`PAGE_FRAME_BOTTOM_CLIP_GUARD`] 一并启用，
/// 保证浮动链接状态栏时代的外框底边位置与圆角可见性；非最大化时不使用。
pub const PAGE_FRAME_BOTTOM_UI_GUARD: f32 = STATUS_BAR_HEIGHT;
/// 页面视口边框宽度
pub const PAGE_FRAME_BORDER: f32 = 1.0;
/// 页面视口圆角半径
pub const PAGE_FRAME_RADIUS: f32 = 8.0;
/// 页面滚动条厚度（逻辑像素，classic 滚动条占位）
pub const SCROLLBAR_THICKNESS: f32 = 12.0;
/// 滚动条滑块最小长度（逻辑像素）
pub const SCROLLBAR_MIN_THUMB: f32 = 24.0;
/// Wayland 无系统装饰时，非最大化窗口外框描边宽度
pub const WINDOW_FRAME_BORDER: f32 = 1.0;
