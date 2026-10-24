//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use zero_browser_shell::{
    BrowserMenuLabel, BrowserSettings, BrowserShell, ColorThemePreference, ContextMenu, ContextType, FindWrapHint,
    MenuItem, PrintOrientationPreference, SearchEngine, SessionState, SuggestionSource, TabId, TabMenuLabel,
    UiLanguage, browser_menu_label, tab_menu_label,
};
use zero_engine::PrefersColorSchemeValue;
use zero_engine::{set_char_measure_fn, set_fallback_line_metrics_fn, set_text_shape_fn};
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::rasterize_full_scene;
#[cfg(test)]
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::image_cache::ImageCache;
use zero_render_foundation::primitive::{
    FillPrimitive, GlyphPrimitive, GradientKind, ImagePrimitive, RenderPrimitives, RoundedRectPrimitive,
    ShadowPrimitive,
};

use crate::colors;
use crate::favicon_fetch::FaviconFetchState;
use crate::image_save_fetch::ImageSaveFetchState;
use crate::input_keys::key_matches;
use crate::layout;
use crate::page_scroll::{self, TabScrollState};
use crate::page_selection::{GlyphSelection, hit_test_caret};
use crate::pages;
use crate::tab_manager::TabManager;
use crate::text_input::TextInput;
use crate::text_metrics;
use crate::view_source_fetch::ViewSourceFetchState;

const TAB_BAR_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(450);

/// ZW_RESIZE_TRACE 置位时启用窗口 resize 链路诊断（t11，值可为 "1" 或 trace 文件
/// 路径——renderer 子进程走文件）。进程级快照：入口读一次，热路径只做 bool 判断
/// （不做逐事件 getenv）。
pub(crate) static RESIZE_TRACE: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_RESIZE_TRACE").is_ok());

/// ZW_RESIZE_TRACE 诊断输出。时间戳用 epoch 毫秒——browser 与 renderer（子进程，
/// stderr 同流）跨进程对齐时序用。
pub(crate) fn resize_trace(component: &str, msg: impl std::fmt::Display) {
    if *RESIZE_TRACE {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();
        eprintln!("[ZW_RESIZE_TRACE] t={now} {component} {msg}");
    }
}

/// 浏览器 UI 场景图元包：`(fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows)`。
/// `overlay_*` 在所有 fills/glyphs 之后绘制；`chrome_shadows` 是壳层阴影（如页面视口）。
pub(crate) type ChromeScene = (
    Vec<FillPrimitive>,
    Vec<GlyphDraw>,
    Vec<FillPrimitive>,
    Vec<GlyphDraw>,
    Vec<ShadowPrimitive>,
    Vec<RoundedRectPrimitive>,
);

/// 地址栏页面类型（由 URL 推导）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddressBarPageKind {
    Secure,
    Insecure,
    Internal,
    Local,
    Unknown,
}

/// 页面内查找单条匹配缓存：glyph 区间 + 文档坐标包围盒 `(x, top, w, h)`。
pub(crate) type FindMatchRange = (std::ops::Range<usize>, (f32, f32, f32, f32));

include!("app_types.rs");

/// 浏览器应用状态
pub struct BrowserApp {
    /// 浏览器 Shell（标签页、书签、历史）
    pub shell: BrowserShell,
    /// 标签页运行时（每 Tab 独立 worker 或渲染进程）
    tabs: TabManager,
    /// GPU 渲染器
    gpu_renderer: Option<GpuRenderer>,
    /// 上次渲染时活跃标签页（R3254-M4：标签切换时清 GPU 图片纹理缓存，防跨标签滞留）。
    last_rendered_tab: Option<TabId>,
    /// 已真实呈现的帧计数（GPU/CPU present 路径各计一次；ui_driver 滚动帧率基准用）。
    presented_frames: u64,
    /// 渲染模式
    render_mode: RenderMode,
    /// 字体加载器
    font_loader: FontLoader,
    /// renderer-local → browser-local 下载字体 ID，仅用于页面 glyph。
    page_font_imports: HashMap<u32, u32>,
    /// Glyph 缓存
    glyph_cache: GlyphCache,
    /// 已加载的系统字体 ID
    font_id: Option<u32>,
    /// 是否已初始化 GPU 表面
    pub surface_configured: bool,
    /// GPU 初始化时的窗口引用（R3254-G5：设备丢失后重建 renderer 用）。
    gpu_window: Option<std::sync::Arc<winit::window::Window>>,
    /// GPU 表面在失焦后需重新配置（Wayland surface 挂起）
    pub gpu_surface_stale: bool,
    /// 窗口是否获得焦点（Wayland 下失焦时 surface 可能挂起）
    pub window_focused: bool,
    /// 地址栏文本编辑状态
    address_bar: TextInput,
    /// 地址栏是否获得焦点
    pub address_bar_focused: bool,
    /// 地址栏 IME 预编辑文本
    address_bar_ime_preedit: String,
    /// 地址栏左键拖选
    address_bar_drag: bool,
    /// 地址栏双击检测
    address_bar_last_click: Option<(Instant, f64, f64)>,
    /// 地址栏编辑态水平滚动偏移（全文像素坐标；T37 光标跟随滚动，渲染帧钳制，
    /// 见 `clamp_address_bar_scroll`）
    address_bar_scroll_x: f32,
    /// 地址栏光标闪烁相位基准（编辑/移动光标时重置——输入后短暂常显再闪烁，
    /// Chrome 同款；此前复用 `chrome_anim_start` 全局相位，输入后可能立即隐没）
    address_bar_cursor_phase: Instant,
    /// 窗口物理像素尺寸
    pub physical_size: (u32, u32),
    /// 窗口缩放因子
    pub scale_factor: f32,
    /// 是否需要重绘
    pub needs_redraw: bool,
    /// 鼠标位置（用于悬停检测）
    pub mouse_pos: (f64, f64),
    /// Ctrl 键是否按住
    ctrl_pressed: bool,
    /// Cmd (macOS) / Meta 键是否按住
    cmd_pressed: bool,
    /// Shift 键是否按住
    shift_pressed: bool,
    /// Alt 键是否按住（用于 Alt+Left/Right 后退/前进）
    alt_pressed: bool,
    /// 自动补全状态
    autocomplete: AutocompleteState,
    /// 查找栏输入文本
    find_input: String,
    /// 上次查找栏使用的查询（关闭后保留，供 F3 重复查找）。
    last_find_query: String,
    /// 标签页布局缓存：每个标签页的 (x, width) 位置信息
    tab_layout: Vec<(TabId, f32, f32)>,
    /// 标签溢出 ▾ 按钮的 x（物理像素）；None=无溢出，按钮未渲染。渲染帧刷新。
    tab_overflow_btn_x: Option<f32>,
    /// 标签溢出下拉是否展开（▾ 点击切换）。
    tab_overflow_open: bool,
    /// 压缩标签悬停 tooltip：`(标签 id, 悬停起始时刻)`。悬停进入压缩标签时记录，
    /// 移开/点击/标签消失时清除；延迟到期后由渲染层显示。
    tab_tooltip: Option<(TabId, Instant)>,
    /// 溢出隐藏的标签 id（渲染帧刷新，与 tab_layout 同生命周期）。
    tab_overflow_ids: Vec<TabId>,
    /// 右键上下文菜单状态
    context_menu: ContextMenuState,
    /// 打开菜单的同一次左键按下后，忽略紧随其后的左键释放（避免按下 `...` 打开菜单时被立即关闭）。
    context_menu_suppress_left_up: bool,
    /// 页面滚动偏移（物理像素）
    scroll: HashMap<TabId, TabScrollState>,
    /// T26：进行中的滚轮平滑滚动动画（每帧由 `tick_smooth_scroll` 推进）。
    scroll_anim: HashMap<TabId, page_scroll::ScrollAnimation>,
    /// M2-S3c：env 门控点击注入器（`ZW_INJECT_CLICKS`，DC-3 命中面触发器；
    /// env 未设 = 禁用零行为）。
    click_injector: crate::app_click_injector::ClickInjector,
    /// 保留帧缓冲 + 滚动 blit 缓存（性能门禁优化 S1，2026-08-08）：
    /// 纯滚动帧平移上一帧内容像素 + 只重绘新露条带，避免全量光栅。
    /// `fb_cache_epoch` = (快照序号, 宽, 高, 缩放)——任一变化即失效走全量渲染。
    retained_fb: Option<zero_render_foundation::surface::FrameBuffer>,
    fb_cache_scroll: (f32, f32),
    fb_cache_epoch: (u64, u32, u32, f32),
    /// 页面文本选区（glyph 索引）
    page_selection: HashMap<TabId, GlyphSelection>,
    /// 页面选区拖拽中
    page_selection_drag: bool,
    /// 左键是否按下
    left_button_down: bool,
    /// 站点权限管理器（按 origin 隔离的 Web API 授权状态）。
    permissions: zero_security::permission::PermissionManager,
    /// 待执行的窗口控制动作
    pending_window_chrome_action: Option<WindowChromeAction>,
    /// 窗口控制按钮悬停索引（0=最小化, 1=最大化, 2=关闭）
    window_control_hover: Option<usize>,
    /// 窗口是否最大化（用于绘制还原图标）
    window_is_maximized: bool,
    /// 窗口是否处于全屏（macOS 全屏时 traffic lights 移走，标签栏左侧留白应消失）
    window_is_fullscreen: bool,
    /// 标签栏空白处上次点击（用于双击检测）
    last_tab_bar_blank_click: Option<(f64, f64, Instant)>,
    /// 标签栏空白处按下位置（移动超过阈值后触发拖动）
    tab_bar_drag_press: Option<(f64, f64)>,
    /// 标签栏 chrome 动画起始时间（loading 旋转环）
    chrome_anim_start: Instant,
    /// 当前加载动画起始时刻（is_loading 从 false→true 时记录）。
    /// 用于模拟 Chrome 风格的加载进度条动画。
    loading_anim_start: Option<Instant>,
    /// 菜单打开动画起始时刻（context_menu.visible false→true 时记录）。
    /// 用于打开时的透明度渐入（150ms 内，Chrome 同款；关闭立即消失）。
    menu_anim_start: Option<Instant>,
    /// 缩放百分比浮层显示起始时刻。None 表示不显示。
    /// zoom_in/out/reset 触发时记录，3 秒后由渲染层清除。
    zoom_indicator_start: Option<Instant>,
    /// T61：打印预览设置面板各行的命中区（物理像素 `(x, y, w, h)`；渲染层写入，输入层消费）。
    /// 行 0 = 方向（整行切换），行 1 = 份数、行 2 = 边距（左半 − / 右半 +）。全零 = 未显示。
    print_setup_panel_rows: [(f32, f32, f32, f32); 3],
    /// T61-D4：份数/边距行的 −/+ 字形起点 x（渲染层按文本前缀实测写入；行 0 为 (0,0)）。
    print_setup_panel_steppers: [(f32, f32); 3],
    /// T62-D2：一次性放行文件导入动作 URL（仅 UI 入口设置；内容锚点/渲染端表单
    /// 各自经来源门禁后亦可置位）。try_apply_bookmarks_url 消费后即复位。
    allow_file_import_once: bool,
    /// 下载完成 toast：`(文件名, 显示起始时刻)`。下载完成时记录，3 秒后由渲染层清除。
    download_toast: Option<(String, Instant)>,
    /// 关闭标签撤销 toast 显示起始时刻。关闭标签时记录，5 秒后由渲染层清除；
    /// 撤销动作走 [`BrowserApp::reopen_last_closed_tab`]（弹出 shell 的 recently_closed 栈）。
    closed_tab_toast: Option<Instant>,
    /// 每标签导航历史条目的滚动偏移（物理像素），键为 `(标签 id, 历史索引)`。
    /// 离开条目前保存、back/forward 回到时恢复（Chrome 同款会话内滚动位置保持）。
    history_scroll: HashMap<(TabId, usize), f32>,
    /// back/forward 待应用的滚动恢复 `(标签 id, 目标偏移)`。目标文档装载完成
    /// （page_loaded）后才落地——立即写入会在空白文档期被 clamp_tab_scroll 压回 0。
    pending_scroll_restore: Option<(TabId, f32)>,
    /// 最近提交给 compositor 的 UI surface 位图（T36 滚动帧 IPC 跳过用）。
    ui_forward_cache: Option<UiForwardCache>,
    /// 上次设置的窗口标题缓存，用于检测变化避免重复 set_title。
    last_window_title: String,
    /// 上次左键点击标签的时间，用于判定双击关闭。
    last_tab_click_time: Option<Instant>,
    /// 上次左键点击的标签 id。
    last_tab_click_id: Option<TabId>,
    /// 后台标签的最近 title 快照，用于检测 title 变化触发 attention。
    /// 仅缓存非活跃标签的 title；活跃标签的 title 变化不需要提醒。
    background_tab_titles: HashMap<TabId, String>,
    /// 当前标签拖拽状态。None 表示未拖拽。
    tab_drag: Option<TabDragState>,
    /// 标签拖拽让位动画：各标签当前视觉位移（物理像素，0 = 原槽位）。
    /// 渲染帧朝目标位移指数平滑，拖拽结束/取消后首次渲染清零。
    tab_drag_paint_offsets: HashMap<TabId, f32>,
    /// 让位动画上一帧时间戳（帧率无关平滑）。
    tab_drag_anim_last: Option<Instant>,
    /// 触摸 tap 候选：Started 时记录，Ended 时若移动 <阈值则合成为左键 click。
    /// 仅 chrome UI 区（非页面内容）走此路径；页面内容区用 touch_scroll。
    touch_tap_candidate: Option<(u64, f64, f64)>,
    /// 触摸长按候选：页面内容区 Started 时记录，超时未移动/释放则合成为右键菜单。
    touch_long_press: Option<(u64, f64, f64, Instant)>,
    /// 系统颜色方案偏好
    color_scheme: PrefersColorSchemeValue,
    /// 浏览器外壳配色
    chrome_palette: colors::ChromePalette,
    /// 最近一次已知的 winit 窗口主题（Auto 模式解析用）。
    cached_window_theme: Option<winit::window::Theme>,
    /// 标签页 URL 加载（延迟绘制 loading / 后台 HTTP）
    tab_fetch: TabFetchState,
    /// CLI 位置参数启动 URL：窗口就绪后由 `ensure_startup_tab` 消费一次。
    startup_url: Option<String>,
    /// 鼠标悬停链接时在浮动状态栏中显示的 URL
    hovered_link_url: Option<String>,
    /// 单指触摸滚动：`(touch_id, last_y 物理像素)`
    touch_scroll: Option<(u64, f64)>,
    /// 鼠标左键在页面内容区的拖拽（远程桌面触摸模拟）
    content_pointer_drag: Option<ContentPointerDrag>,
    /// 滚动条滑块拖拽。
    scrollbar_drag: Option<ScrollbarDrag>,
    /// 滚动条 hover 命中。
    scrollbar_hover: Option<crate::page_scroll::ScrollbarHit>,
    /// Overlay 滚动条因最近滚动保持可见的截止时间。
    scrollbar_visible_until: Option<Instant>,
    /// 进行中的 favicon 异步拉取。
    favicon_fetch: FaviconFetchState,
    /// 进行中的 view-source 源码异步抓取（后台线程 + 主循环 poll）。
    view_source_fetch: ViewSourceFetchState,
    /// 进行中的图片另存为异步抓取（后台线程 + 主循环 poll，走下载管线落盘）。
    image_save_fetch: ImageSaveFetchState,
    /// 进行中的页面另存为异步抓取（T51；与图片另存为同一抓取基础设施的独立
    /// 队列，完成项按页面命名规则走下载管线）。
    page_save_fetch: ImageSaveFetchState,
    /// 是否显示下载浮动面板（也可因活动下载自动展开）。
    download_panel_open: bool,
    /// 最近一次「在文件夹中显示」动作打开的目录（演示流断言面）。
    last_opened_download_dir: Option<String>,
    /// 页面内查找（Ctrl+F）匹配缓存：glyph 区间 + 文档坐标包围盒，与
    /// `find_matches_query` 同步失效（导航/关栏/换词）。
    find_match_ranges: Vec<FindMatchRange>,
    /// 匹配缓存对应的查询词。
    find_matches_query: String,
    /// T20：崩溃恢复候选会话（启动读到 `clean_exit=false` 时保留，
    /// 气泡「恢复标签」/「关闭」后清空）。
    pending_session_restore: Option<SessionState>,
    /// T20：崩溃恢复气泡（"Restore pages?"）是否可见。
    session_restore_prompt_visible: bool,
    /// T20：启动会话恢复检查只执行一次（`ensure_startup_tab` 可能多次调用）。
    session_startup_handled: bool,
    /// FR-6 崩溃循环守卫：观察窗到期时刻（tab 存活且无新崩溃即清零计数）。
    crash_reset_deadlines: Vec<(TabId, Instant)>,
    #[cfg(test)]
    compositor_status_override: Option<crate::compositor_client::CompositorStatus>,
}

/// 屏幕矩形 `(x, y, w, h)`（物理像素）；崩溃恢复气泡按钮几何签名用别名。
type ScreenRect = (f32, f32, f32, f32);

/// Chrome UI 转发缓存（T36）：最近提交给 compositor 的 UI surface 位图，
/// 用于逐字节比对跳过无变化帧的整帧 IPC。
#[derive(Debug)]
struct UiForwardCache {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl UiForwardCache {
    fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rgba: Vec::new(),
        }
    }
}

impl BrowserApp {
    /// 测试路径的系统字体加载：进程级缓存 + `duplicate`（Arc 共享已解析字体）。
    ///
    /// 77 个 app 测试每个都 `BrowserApp::new`，生产路径每次全量解析（含 19MB CJK
    /// fallback，~2s/测）；缓存后全进程只解析一次，每测 ~10ms。duplicate 保持
    /// font_id 序号与字体顺序一致 → 各测试的 loader/font_id 内容与独立加载等价。
    /// `&self` 只读 + fontdue 无内部可变性 → 并发（cargo test 多线程）安全。
    fn cached_system_fonts() -> (FontLoader, Option<u32>) {
        // 进程级共享：BrowserApp 与 TabWorker 线程共用一次解析（见
        // shared_system_fonts——worker 不再重复解析 19MB CJK 字体）
        shared_system_fonts()
    }

    /// 创建新的浏览器应用
    pub fn new(render_mode: RenderMode) -> Self {
        // 进程级共享系统字体（生产与测试同路径）：BrowserApp 与各 TabWorker 共用
        // 一次解析（19MB CJK fallback ~0.5s + ~40-60MB/份），避免 N 标签页 N 份
        // 重复解析（startup 475ms 的主要嫌疑之一）。duplicate 保持 font_id 序号
        // 一致，内容与独立加载等价；进程生命周期内系统字体文件视为稳定。
        let (font_loader, font_id) = Self::cached_system_fonts();

        if font_id.is_some() {
            tracing::info!("System font loaded");
        } else {
            tracing::warn!("No system font found, text rendering will be limited");
        }

        set_char_measure_fn(text_metrics::measure_char);
        set_text_shape_fn(text_metrics::shape_text);
        zero_engine::set_hmtx_measure_fn(text_metrics::measure_text_hmtx);
        // R4376：回退链垂直度量回调（消费门禁 ZW_FALLBACK_LINE_METRICS，默认 on）。
        set_fallback_line_metrics_fn(text_metrics::fallback_line_metrics);

        let shell = BrowserShell::new_with_persisted_settings();
        let detected = detect_system_color_scheme();
        let color_scheme = resolve_effective_color_scheme(shell.settings().color_theme, None, detected);

        let mut app = Self {
            shell,
            tabs: TabManager::new((800, 600), color_scheme),
            gpu_renderer: None,
            last_rendered_tab: None,
            presented_frames: 0,
            render_mode,
            font_loader,
            page_font_imports: HashMap::new(),
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            gpu_window: None,
            gpu_surface_stale: false,
            window_focused: true,
            address_bar: TextInput::new(),
            address_bar_focused: false,
            address_bar_ime_preedit: String::new(),
            address_bar_drag: false,
            address_bar_last_click: None,
            address_bar_scroll_x: 0.0,
            address_bar_cursor_phase: Instant::now(),
            physical_size: (1024, 768),
            scale_factor: 1.0,
            needs_redraw: true,
            mouse_pos: (0.0, 0.0),
            ctrl_pressed: false,
            cmd_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            autocomplete: AutocompleteState::new(),
            find_input: String::new(),
            last_find_query: String::new(),
            tab_layout: Vec::new(),
            tab_overflow_btn_x: None,
            tab_overflow_open: false,
            tab_tooltip: None,
            tab_overflow_ids: Vec::new(),
            context_menu: ContextMenuState::new(),
            context_menu_suppress_left_up: false,
            scroll: HashMap::new(),
            scroll_anim: HashMap::new(),
            click_injector: crate::app_click_injector::ClickInjector::from_env(),
            retained_fb: None,
            fb_cache_scroll: (0.0, 0.0),
            fb_cache_epoch: (0, 0, 0, 0.0),
            page_selection: HashMap::new(),
            page_selection_drag: false,
            left_button_down: false,
            pending_window_chrome_action: None,
            window_control_hover: None,
            window_is_maximized: false,
            window_is_fullscreen: false,
            last_tab_bar_blank_click: None,
            tab_bar_drag_press: None,
            chrome_anim_start: Instant::now(),
            loading_anim_start: None,
            menu_anim_start: None,
            zoom_indicator_start: None,
            print_setup_panel_rows: [(0.0, 0.0, 0.0, 0.0); 3],
            print_setup_panel_steppers: [(0.0, 0.0); 3],
            allow_file_import_once: false,
            download_toast: None,
            closed_tab_toast: None,
            history_scroll: HashMap::new(),
            pending_scroll_restore: None,
            ui_forward_cache: None,
            last_window_title: String::new(),
            last_tab_click_time: None,
            last_tab_click_id: None,
            background_tab_titles: HashMap::new(),
            tab_drag: None,
            tab_drag_paint_offsets: HashMap::new(),
            tab_drag_anim_last: None,
            touch_tap_candidate: None,
            touch_long_press: None,
            color_scheme,
            chrome_palette: colors::ChromePalette::for_scheme(color_scheme),
            cached_window_theme: None,
            tab_fetch: TabFetchState::None,
            startup_url: None,
            hovered_link_url: None,
            touch_scroll: None,
            content_pointer_drag: None,
            scrollbar_drag: None,
            scrollbar_hover: None,
            scrollbar_visible_until: None,
            favicon_fetch: FaviconFetchState::new(),
            view_source_fetch: ViewSourceFetchState::new(),
            image_save_fetch: ImageSaveFetchState::new(),
            page_save_fetch: ImageSaveFetchState::new(),
            download_panel_open: false,
            last_opened_download_dir: None,
            find_match_ranges: Vec::new(),
            find_matches_query: String::new(),
            pending_session_restore: None,
            session_restore_prompt_visible: false,
            session_startup_handled: false,
            crash_reset_deadlines: Vec::new(),
            permissions: zero_security::permission::PermissionManager::new(),
            #[cfg(test)]
            compositor_status_override: None,
        };
        app.tabs.set_javascript_enabled(app.shell.settings().javascript_enabled);
        if crate::compositor_client::enabled() {
            crate::compositor_client::register_ui_surface(zero_protocol::CompositorUiSurfaceInfo {
                surface_id: crate::compositor_client::CHROME_UI_SURFACE_ID,
                width: app.physical_size.0,
                height: app.physical_size.1,
            });
            crate::compositor_client::register_window_surface(zero_protocol::CompositorWindowSurfaceInfo {
                surface_id: crate::compositor_client::CHROME_WINDOW_SURFACE_ID,
                width: app.physical_size.0,
                height: app.physical_size.1,
            });
        }
        app
    }

    /// 当前悬停链接 URL（浮动状态栏内容；无悬停时为 `None`）。
    #[cfg(test)]
    pub fn hovered_link_url(&self) -> Option<&str> {
        self.hovered_link_url.as_deref()
    }

    fn set_hovered_link_url(&mut self, url: Option<String>) {
        if self.hovered_link_url != url {
            self.hovered_link_url = url;
            self.needs_redraw = true;
        }
    }

    fn update_hovered_link_at(&mut self, x: f64, y: f64) {
        // 菜单覆盖在页面之上时状态栏不跟随（菜单打开即已清除，不因指针扫过
        // 菜单下的页面区域而重新出现，Chrome 同语义）。
        if self.context_menu.visible {
            return;
        }
        let href = self
            .page_doc_point(x as f32, y as f32)
            .and_then(|(tab_id, doc_x, doc_y)| {
                let href = self.tabs.hit_test_link(tab_id, doc_x, doc_y)?;
                // T6-I7：状态栏显示解析后的绝对 URL——与链接点击导航同一 join
                // 逻辑（resolve_clicked_link_url），保证"所显即所往"。
                let document_url = self.shell.tab(tab_id).and_then(|tab| tab.url());
                Some(resolve_clicked_link_url(&href, document_url))
            });
        self.set_hovered_link_url(href);
    }

    /// 是否有进行中的标签页 fetch（含等待首帧绘制）。
    pub fn tab_fetch_active(&self) -> bool {
        !matches!(self.tab_fetch, TabFetchState::None)
    }

    /// 轮询 Tab worker / IPC 并处理页面事件。
    pub fn poll_tab_fetch(&mut self) {
        // M2-S1e：窗口轮询 tick 心跳（每 64 tick ~1s 一条 debug）——负载窗帧空洞
        // 归因：心跳在 = 主循环活着，心跳停 = tick 被卡（renderer/mux 同步耦合）。
        static POLL_TICK: AtomicU64 = AtomicU64::new(0);
        let tick = POLL_TICK.fetch_add(1, Ordering::Relaxed);
        if tick.is_multiple_of(64) {
            tracing::debug!("browser: poll tick #{} 心跳", tick);
        }
        if self.favicon_fetch.poll(&mut self.font_loader) {
            self.needs_redraw = true;
        }
        if self.favicon_fetch.poll_bookmarks(&mut self.font_loader) {
            self.needs_redraw = true;
        }
        // view-source 后台抓取完成：取结果并开源码/错误内页（异步，不阻塞事件循环）。
        for completed in self.view_source_fetch.take_completed() {
            let page = match completed.result {
                Ok(html) => pages::generate_view_source_page(&completed.target, &html),
                Err(error) => pages::generate_error_page(&completed.view_url, &error),
            };
            self.open_internal_document_tab(page, &completed.view_url, "查看源代码");
        }
        if self.tabs.poll(self.shell.active_tab_id(), self.gpu_renderer_is_some()) {
            self.needs_redraw = true;
        }
        // 消费异步 DOM 事件派发产生的延迟动作（链接导航等）。
        // 必须在 `tabs.poll` 之后调用 —— pending actions 由 poll 收集。
        for (tab_id, action) in self.tabs.take_pending_actions() {
            use crate::tab_manager::PendingTabAction;
            match action {
                PendingTabAction::NavigateActiveTab(href) => {
                    let href = resolve_clicked_link_url(
                        &href,
                        self.shell
                            .tabs()
                            .find(|tab| tab.id() == tab_id)
                            .and_then(|tab| tab.url()),
                    );
                    if self.shell.active_tab_id() == Some(tab_id) {
                        self.navigate_to(&href);
                    } else {
                        // 用户在等待结果期间切走了；在新标签打开更符合预期。
                        self.new_tab_background(&href);
                    }
                }
                PendingTabAction::OpenBackgroundTab(href) => {
                    let href = resolve_clicked_link_url(
                        &href,
                        self.shell
                            .tabs()
                            .find(|tab| tab.id() == tab_id)
                            .and_then(|tab| tab.url()),
                    );
                    self.new_tab_background(&href);
                }
                PendingTabAction::RequestRedraw => {
                    self.needs_redraw = true;
                }
                // R3254-M9：页面 keydown 未 preventDefault 时执行滚动（焦点在文本控件的
                // Tab 已在 take_pending_actions 被守卫过滤）。
                PendingTabAction::ScrollViewport { delta } => {
                    self.scroll_active_page_by_px(delta);
                }
            }
        }
        // R3254-M10：单进程表单提交导航（worker 回执；GET / POST）。
        for (tab_id, url, method, body) in self.tabs.take_pending_navigations() {
            if self.shell.active_tab_id() == Some(tab_id) {
                self.navigate_to_request(&url, method, body);
            }
        }
        // T62-D1：多进程渲染端 zero:// 导航装载请求（与上loop同语义）。
        for (tab_id, request) in self.tabs.take_renderer_navigations() {
            self.handle_renderer_tab_navigate(tab_id, request);
        }
        // T5/P10：页面 history 遍历请求（单进程 worker 回执 + 多进程 HistoryTraverse，
        // tab_manager 已合流）——仅活跃 tab 提交（遍历作用于当前会话，后台 tab 丢弃，
        // 同上两 loop 守卫）。
        for (tab_id, delta) in self.tabs.take_history_traverses() {
            if self.shell.active_tab_id() == Some(tab_id) {
                self.history_traverse(delta);
            }
        }
        if let Some(id) = self.shell.active_tab_id() {
            self.clamp_tab_scroll(id);
        }
        for (tab_id, title, url) in self.tabs.take_page_loaded_events() {
            if self.shell.active_tab_id() == Some(tab_id) {
                self.shell.on_page_loaded(&title);
            }
            self.refresh_tab_favicon(tab_id, &url);
            self.shell.set_tab_crashed(tab_id, false);
            // back/forward 滚动恢复：目标文档就绪后落地（Chrome 同款时机）。
            self.apply_pending_scroll_restore(tab_id);
            if self.shell.active_tab_id() == Some(tab_id) {
                if let Some(tab) = self.shell.active_tab_mut() {
                    tab.set_loading(false);
                }
                self.shell.set_tab_needs_attention(tab_id, false);
                // M2-S3c：页面 load 完成即武装注入时钟（禁用态零行为）。
                self.click_injector.arm();
            } else {
                self.shell.set_tab_needs_attention(tab_id, true);
            }
        }
        if self.shell.find_state().is_active() && !self.find_input.is_empty() && self.find_match_ranges.is_empty() {
            // 查找栏开着时页面（重）加载完成：对新内容重算匹配。
            self.refresh_find_matches();
        }
        for (tab_id, error) in self.tabs.take_page_error_events() {
            self.shell.on_page_error(&error);
            self.shell.set_tab_crashed(tab_id, true);
            if self.shell.active_tab_id() == Some(tab_id)
                && let Some(tab) = self.shell.active_tab_mut()
            {
                tab.set_loading(false);
            }
        }
        // FR-6 崩溃循环守卫：崩溃 tab 计数递增并即时落盘（浏览器可能随后即死），
        // 并重排观察窗到期时刻——存活一个观察窗且无新崩溃即清零。
        for tab_id in self.tabs.take_renderer_crash_events() {
            if let Some(count) = self.shell.increment_tab_crash_count(tab_id) {
                tracing::info!(tab_id = tab_id.0, count, "tab renderer crash counted");
                self.persist_running_session();
            }
            self.schedule_crash_reset(tab_id);
        }
        self.sweep_crash_reset_deadlines();
        for download in self.tabs.take_pending_downloads() {
            self.save_attachment_download(download);
        }
        for (tab_id, url, result) in self.image_save_fetch.take_completed() {
            self.save_image_fetch_result(tab_id, url, result);
        }
        // T51：页面另存为抓取完成（与图片另存为同一抓取基础设施，独立队列）。
        for (tab_id, url, result) in self.page_save_fetch.take_completed() {
            self.save_page_fetch_result(tab_id, url, result);
        }

        // 检测后台标签 title 变化（如聊天应用收到消息改 title 加 "(3)"），
        // 变化时触发 needs_attention 提醒用户。
        let active_id = self.shell.active_tab_id();
        let mut title_changes: Vec<(TabId, String)> = Vec::new();
        for tab in self.shell.tabs() {
            let id = tab.id();
            if Some(id) == active_id {
                // 活跃标签不需要提醒，从缓存移除避免残留。
                self.background_tab_titles.remove(&id);
                continue;
            }
            let current_title = tab.title().unwrap_or("").to_string();
            match self.background_tab_titles.get(&id) {
                Some(prev) if *prev == current_title => {}
                _ => title_changes.push((id, current_title)),
            }
        }
        for (id, title) in title_changes {
            let is_new = !self.background_tab_titles.contains_key(&id);
            self.background_tab_titles.insert(id, title.clone());
            // 仅在 title 从一个非空值变为另一个非空值时触发提醒，
            // 首次记录（加载初期的 title）不算。
            if !is_new && !title.is_empty() {
                self.shell.set_tab_needs_attention(id, true);
            }
        }

        // 触摸长按检测：页面内容区按住 ~500ms 未移动 → 合成右键菜单。
        if let Some((id, x, y, start)) = self.touch_long_press {
            if start.elapsed() >= std::time::Duration::from_millis(500) {
                self.touch_long_press = None;
                // 合成右键 click（press + release）触发 show_context_menu。
                self.handle_mouse_click(x, y, true, "Right");
                self.handle_mouse_click(x, y, false, "Right");
                // 标记已处理，避免后续 Ended 重复。清空 touch_scroll 防止滚动。
                if self.touch_scroll.is_some_and(|(sid, _)| sid == id) {
                    self.touch_scroll = None;
                }
            }
        }
    }

    /// 在绘制 loading 帧之后启动 Tab worker 加载。
    pub fn begin_tab_fetch_after_paint(&mut self) {
        let state = std::mem::replace(&mut self.tab_fetch, TabFetchState::None);
        let TabFetchState::WaitingPaint(tab_id, url) = state else {
            self.tab_fetch = state;
            return;
        };
        self.tabs.ensure_tab(tab_id);
        if url == "zero://settings" {
            self.open_settings_page();
        } else if url.starts_with("http://") || url.starts_with("https://") {
            self.tabs.navigate(tab_id, url);
        } else {
            self.load_local_tab_url(tab_id, &url);
        }
        self.needs_redraw = true;
    }

    fn apply_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        if self.color_scheme == scheme {
            return;
        }
        self.color_scheme = scheme;
        self.chrome_palette = colors::ChromePalette::for_scheme(scheme);
        self.tabs.set_color_scheme(scheme);
        self.needs_redraw = true;
    }

    fn apply_resolved_color_scheme(&mut self, window_theme: Option<winit::window::Theme>) {
        let detected = detect_system_color_scheme();
        let scheme = resolve_effective_color_scheme(self.shell.settings().color_theme, window_theme, detected);
        self.apply_color_scheme(scheme);
    }

    /// 轮换主题偏好（Auto → Light → Dark → Auto）并立即应用。
    pub fn cycle_color_theme(&mut self) {
        let next = self.shell.settings().color_theme.cycle();
        self.shell.apply_settings(|settings| settings.color_theme = next);
        self.apply_resolved_color_scheme(self.cached_window_theme);
    }

    /// 操作系统主题变更（仅在 Auto 模式下生效）。
    pub fn handle_system_theme_changed(&mut self, dark: bool) {
        self.cached_window_theme = Some(if dark {
            winit::window::Theme::Dark
        } else {
            winit::window::Theme::Light
        });
        if color_scheme_from_env().is_some() {
            return;
        }
        if self.shell.settings().color_theme != ColorThemePreference::Auto {
            return;
        }
        self.apply_color_scheme(if dark {
            PrefersColorSchemeValue::Dark
        } else {
            PrefersColorSchemeValue::Light
        });
    }

    fn theme_button_icon(&self) -> crate::ui_icons::Icon {
        match self.shell.settings().color_theme {
            ColorThemePreference::Auto => crate::ui_icons::Icon::SunMoon,
            ColorThemePreference::Light => crate::ui_icons::Icon::Sun,
            ColorThemePreference::Dark => crate::ui_icons::Icon::Moon,
        }
    }

    /// 使用 winit 窗口主题更新颜色方案（`ZERO_BROWSER_COLOR_SCHEME` 已设置时跳过）。
    pub fn sync_color_scheme_from_window(&mut self, window: &winit::window::Window) {
        if let Some(theme) = window.theme() {
            self.cached_window_theme = Some(theme);
        }
        self.apply_resolved_color_scheme(self.cached_window_theme);
    }

    /// 是否需要自绘窗口控制按钮（最小化/最大化/关闭）。
    ///
    /// - Wayland：无系统装饰
    /// - Windows：Chrome 风格无边框标题栏，改用浏览器自绘按钮
    /// - macOS：使用系统 traffic lights，无需自绘
    pub fn uses_custom_window_controls(&self) -> bool {
        is_wayland() || cfg!(target_os = "windows")
    }

    /// macOS 一体化标题栏（系统 traffic lights 与标签栏同排）。
    /// 全屏时 traffic lights 移走，无需为它们预留左侧留白。
    pub fn tab_bar_leading_inset(&self) -> f32 {
        if uses_unified_titlebar() && !self.window_is_fullscreen {
            layout::MACOS_TRAFFIC_LIGHT_INSET
        } else {
            0.0
        }
    }

    /// 标签栏背景色（Windows / macOS 一体化标题栏下与工具栏融合）。
    pub fn chrome_tab_strip_bg(&self) -> zero_render_foundation::color::Color {
        if !self.window_focused {
            return self.chrome_palette.chrome_inactive_bg;
        }
        self.chrome_palette.tab_bar_bg
    }

    /// 标签栏空白区可拖动窗口
    pub fn supports_tab_bar_window_drag(&self) -> bool {
        self.uses_custom_window_controls() || uses_unified_titlebar()
    }

    /// 取出并清除待执行的窗口控制动作
    pub fn take_window_chrome_action(&mut self) -> Option<WindowChromeAction> {
        self.pending_window_chrome_action.take()
    }

    /// 同步窗口最大化/全屏状态（用于控制按钮图标与视口底部留白）
    pub fn set_window_maximized(&mut self, maximized: bool) {
        if self.window_is_maximized != maximized {
            self.window_is_maximized = maximized;
            self.sync_webview_viewport();
            self.needs_redraw = true;
        }
    }

    /// 同步窗口全屏状态（macOS 全屏时去除 traffic lights 留白）。
    pub fn set_window_fullscreen(&mut self, fullscreen: bool) {
        if self.window_is_fullscreen != fullscreen {
            self.window_is_fullscreen = fullscreen;
            self.needs_redraw = true;
        }
    }

    /// 最大化/全屏时视口底部额外留白（物理像素）；普通窗口仅保留 [`PAGE_FRAME_INSET_BOTTOM`]。
    ///
    /// 仅在 Linux（含 WSLg）启用：这些窗口管理器会裁切最大化窗口的圆角，
    /// 需要 clip guard 避免内容被切；UI guard 名义上给浮动状态栏预留，
    /// 但浮动 UI 实际是 content rect 内的 overlay，并不依赖外部预留。
    /// Windows/macOS 最大化窗口为纯矩形，无需任何 guard。
    fn page_frame_bottom_reserves(&self, scale: f32) -> (f32, f32) {
        if self.window_is_maximized && cfg!(target_os = "linux") {
            (
                layout::PAGE_FRAME_BOTTOM_CLIP_GUARD * scale,
                layout::PAGE_FRAME_BOTTOM_UI_GUARD * scale,
            )
        } else {
            (0.0, 0.0)
        }
    }

    /// 是否显示书签栏（设置开启且至少有一个根书签）。
    pub fn bookmarks_bar_visible(&self) -> bool {
        self.shell.settings().show_bookmarks_bar && !self.shell.bookmarks().list_root().is_empty()
    }

    /// 书签栏占用高度（物理像素）；不可见时为 0。
    pub fn bookmarks_bar_height_for(&self, scale: f32) -> f32 {
        if self.bookmarks_bar_visible() {
            layout::BOOKMARKS_BAR_HEIGHT * scale
        } else {
            0.0
        }
    }

    /// 页面内容区上沿 Y（工具栏 + 可选书签栏），物理像素。
    pub fn chrome_top_y_for(&self, scale: f32) -> f32 {
        layout::TOOLBAR_HEIGHT * scale + self.bookmarks_bar_height_for(scale)
    }

    /// 窗口 surface 尺寸可能变化（全屏/最大化切换后需重新配置）
    pub fn mark_surface_stale(&mut self) {
        self.gpu_surface_stale = true;
        self.surface_configured = false;
        self.needs_redraw = true;
    }

    fn new_tab_button_x(&self) -> f32 {
        // 溢出时「+」按钮位于 ▾ 溢出按钮右侧（render_tabs 同一布局）。
        if let Some(btn_x) = self.tab_overflow_btn_x {
            return btn_x + layout::NEW_TAB_BTN_WIDTH * self.scale_factor;
        }
        self.tab_layout.last().map(|&(_, x, w)| x + w).unwrap_or(0.0)
    }

    /// 标签溢出 ▾ 按钮矩形 `(x, y, w, h)`（物理像素）；无溢出时 None。
    pub(crate) fn tab_overflow_button_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let s = self.scale_factor;
        self.tab_overflow_btn_x.map(|x| {
            (
                x,
                layout::TAB_BAR_TOP_INSET * s,
                layout::NEW_TAB_BTN_WIDTH * s,
                layout::TAB_BAR_HEIGHT * s,
            )
        })
    }

    /// 溢出下拉最多可见行数（隐藏标签数与上限取小）。
    fn tab_overflow_visible_rows(&self) -> usize {
        self.tab_overflow_ids.len().min(layout::TAB_OVERFLOW_MAX_ROWS)
    }

    /// 溢出下拉面板矩形 `(x, y, w, h)`（物理像素），渲染与命中测试同源；
    /// 面板按 ▾ 按钮右对齐并夹到窗口内。无溢出/未渲染按钮时 None。
    fn tab_overflow_panel_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let btn_x = self.tab_overflow_btn_x?;
        let s = self.scale_factor;
        let width = self.physical_size.0 as f32;
        let panel_w = layout::TAB_OVERFLOW_PANEL_WIDTH * s;
        let panel_h = self.tab_overflow_visible_rows() as f32 * layout::TAB_OVERFLOW_ROW_HEIGHT * s;
        let x = (btn_x + layout::NEW_TAB_BTN_WIDTH * s - panel_w).clamp(0.0, (width - panel_w).max(0.0));
        Some((x, layout::TAB_STRIP_HEIGHT * s, panel_w, panel_h))
    }

    /// 溢出下拉行命中测试：返回命中的隐藏标签下标（仅展开时生效）。
    pub(crate) fn tab_overflow_row_at(&self, x: f64, y: f64) -> Option<usize> {
        if !self.tab_overflow_open {
            return None;
        }
        let (px, py, pw, _) = self.tab_overflow_panel_rect()?;
        let row_h = layout::TAB_OVERFLOW_ROW_HEIGHT * self.scale_factor;
        let (x_f, y_f) = (x as f32, y as f32);
        if x_f < px || x_f >= px + pw || y_f < py {
            return None;
        }
        let row = ((y_f - py) / row_h) as usize;
        (row < self.tab_overflow_visible_rows()).then_some(row)
    }

    /// 压缩标签悬停 tooltip 内容 `(标题, URL)`。悬停延迟未到期或来源标签已关闭时
    /// 返回 None（隐藏）；无痕标签标题加前缀。渲染与 smoke 断言面同源。
    fn tab_tooltip_content(&self) -> Option<(String, String)> {
        let (tab_id, since) = self.tab_tooltip.as_ref()?;
        if since.elapsed().as_millis() < u128::from(layout::TAB_TOOLTIP_DELAY_MS) {
            return None;
        }
        let tab = self.shell.tabs().find(|t| t.id() == *tab_id)?;
        let mut title = tab.title().or(tab.url()).unwrap_or("New Tab").to_string();
        if tab.is_private() {
            title = format!("无痕 · {title}");
        }
        Some((title, tab.url().unwrap_or_default().to_string()))
    }

    /// 压缩标签 tooltip 外框 `(x, y, w, h)`（物理像素）：悬停标签槽位下方、
    /// 水平对齐标签并夹到窗口内。渲染与 smoke 断言面同源；隐藏 = None。
    fn tab_tooltip_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        let (title, url) = self.tab_tooltip_content()?;
        let (tab_id, _) = self.tab_tooltip.as_ref()?;
        let (_, tx, tw) = self.tab_layout.iter().find(|(id, _, _)| id == tab_id)?;
        self.font_id?;
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let pad_h = 12.0 * s;
        let max_text_w = layout::TAB_TOOLTIP_MAX_WIDTH * s;
        let text_w = self
            .measure_ui_text_width(&title, font_size)
            .max(self.measure_ui_text_width(&url, font_size))
            .min(max_text_w);
        let box_w = text_w + pad_h * 2.0;
        let line_gap = 4.0 * s;
        let box_h = font_size * 2.0 + line_gap + 12.0 * s;
        let x = (tx + tw * 0.5 - box_w * 0.5).clamp(0.0, (width as f32 - box_w).max(0.0));
        let y = (layout::TAB_STRIP_HEIGHT * s + 2.0 * s).min((height as f32 - box_h).max(0.0));
        Some((x, y, box_w, box_h))
    }

    /// 测试 helper：暴露「+」按钮 x 坐标。
    #[cfg(test)]
    pub fn new_tab_button_x_for_test(&self) -> f32 {
        self.new_tab_button_x()
    }

    /// 测试 helper：溢出下拉面板矩形（T18 溢出下拉命中测试用）。
    #[cfg(test)]
    pub fn tab_overflow_panel_rect_for_test(&self) -> Option<(f32, f32, f32, f32)> {
        self.tab_overflow_panel_rect()
    }

    /// 测试 helper：显式回退单进程 worker 路径（R3254——测试默认单进程，
    /// 断言 worker 路径行为的测试可显式禁用）。
    #[cfg(test)]
    pub fn disable_multiprocess_for_test(&mut self) {
        self.tabs.disable_multiprocess_for_test();
    }

    /// R3254 测试 helper：显式启用多进程 renderer 后端（断言真实多进程链路的 GUI 测试用）。
    #[cfg(test)]
    pub fn enable_multiprocess_for_test(&mut self) {
        self.tabs.enable_multiprocess_for_test();
    }

    /// R3254 测试 helper：强制 renderer legacy 帧发布（本地合成像素测试需要 last_render）。
    #[cfg(test)]
    pub fn set_compositor_status_for_test(&mut self, status: crate::compositor_client::CompositorStatus) {
        self.compositor_status_override = Some(status);
    }

    /// R3254-KP1（keyboard-page-scrolling goal M1）测试 helper：设置 Shift 修饰键状态
    /// 并读取滚动键 delta 映射（分发表语义断言——键位 → 滚动量，不经渲染）。
    #[cfg(test)]
    pub fn scroll_delta_for_key_for_test(&mut self, key: &str, shift: bool) -> Option<f32> {
        self.shift_pressed = shift;
        self.scroll_delta_for_key(key)
    }

    /// R3254-KP1 测试 helper：页面渲染缩放（scroll_delta_for_key 方向键步进的
    /// 40px×scale 断言用）。
    #[cfg(test)]
    pub fn page_render_scale_for_test(&self) -> f32 {
        self.page_render_scale()
    }

    /// R3254 测试 helper：last_render 的 glyph 数量（诊断合成帧内容）。
    #[cfg(test)]
    pub fn last_render_glyphs_for_test(&self, tab_id: TabId) -> Option<usize> {
        self.tabs.last_render(tab_id).map(|r| r.primitives.glyphs.len())
    }

    #[cfg(test)]
    pub fn last_render_primitives_for_test(
        &self,
        tab_id: TabId,
    ) -> Option<&zero_render_foundation::primitive::RenderPrimitives> {
        self.tabs.last_render(tab_id).map(|render| &render.primitives)
    }

    #[cfg(test)]
    pub fn last_render_text_for_test(&self, tab_id: TabId) -> Option<String> {
        self.tabs.last_render(tab_id).map(|render| {
            render
                .primitives
                .glyphs
                .iter()
                .filter_map(|glyph| glyph.code_point())
                .collect()
        })
    }

    fn window_controls_origin_x(&self, width: f32, s: f32) -> f32 {
        width - layout::WINDOW_CONTROLS_WIDTH * s
    }

    fn window_control_hit_test(&self, x: f32, y: f32, width: f32, s: f32) -> Option<WindowChromeAction> {
        if !self.uses_custom_window_controls() || y >= layout::TAB_STRIP_HEIGHT * s {
            return None;
        }
        let x0 = self.window_controls_origin_x(width, s);
        let btn_w = layout::WINDOW_CONTROL_BTN_WIDTH * s;
        if x < x0 || x >= width {
            return None;
        }
        let idx = ((x - x0) / btn_w) as i32;
        match idx {
            0 => Some(WindowChromeAction::Minimize),
            1 => Some(WindowChromeAction::ToggleMaximize),
            2 => Some(WindowChromeAction::Close),
            _ => None,
        }
    }

    /// 是否点击在标签栏空白区域（可拖动 / 双击最大化）
    fn is_tab_bar_blank_hit(&self, x: f32, y: f32, width: f32, s: f32) -> bool {
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        if y >= tab_strip_h {
            return false;
        }
        let leading = self.tab_bar_leading_inset() * s;
        if x < leading {
            return false;
        }
        if self.window_control_hit_test(x, y, width, s).is_some() {
            return false;
        }
        let new_tab_x = self.new_tab_button_x();
        if x >= new_tab_x && x < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
            return false;
        }
        !self
            .tab_layout
            .iter()
            .any(|&(_, tab_x, tab_w)| x >= tab_x && x < tab_x + tab_w)
    }

    fn handle_tab_bar_blank_press(&mut self, x: f64, y: f64) {
        let now = Instant::now();
        let slop = 12.0 * self.scale_factor as f64;
        if let Some((lx, ly, t)) = self.last_tab_bar_blank_click
            && now.duration_since(t) <= TAB_BAR_DOUBLE_CLICK_INTERVAL
            && (x - lx).abs() <= slop
            && (y - ly).abs() <= slop
        {
            self.last_tab_bar_blank_click = None;
            self.tab_bar_drag_press = None;
            self.pending_window_chrome_action = Some(WindowChromeAction::ToggleMaximize);
            return;
        }
        self.last_tab_bar_blank_click = Some((x, y, now));
        self.tab_bar_drag_press = Some((x, y));
    }

    fn update_tab_bar_drag(&mut self, x: f64, y: f64) {
        let Some((ox, oy)) = self.tab_bar_drag_press else {
            return;
        };
        let threshold = 4.0 * self.scale_factor as f64;
        if (x - ox).hypot(y - oy) >= threshold {
            self.tab_bar_drag_press = None;
            self.pending_window_chrome_action = Some(WindowChromeAction::StartDrag);
        }
    }

    fn update_window_control_hover(&mut self, x: f64, y: f64) {
        if !self.uses_custom_window_controls() {
            return;
        }
        let s = self.scale_factor;
        let width = self.physical_size.0 as f32;
        let hover = self
            .window_control_hit_test(x as f32, y as f32, width, s)
            .map(|action| match action {
                WindowChromeAction::Minimize => 0,
                WindowChromeAction::ToggleMaximize => 1,
                WindowChromeAction::Close => 2,
                WindowChromeAction::ToggleFullscreen | WindowChromeAction::StartDrag => {
                    unreachable!()
                }
            });
        if hover != self.window_control_hover {
            self.window_control_hover = hover;
            self.needs_redraw = true;
        }
    }

    /// 将 Windows 非客户区的最大化按钮悬停状态同步到自绘控制按钮。
    #[cfg(target_os = "windows")]
    pub fn sync_windows_caption_hover(&mut self) {
        let maximize_hovered = crate::windows_titlebar::maximize_hovered();
        if maximize_hovered && self.window_control_hover != Some(1) {
            self.window_control_hover = Some(1);
            self.needs_redraw = true;
        } else if !maximize_hovered && self.window_control_hover == Some(1) {
            self.window_control_hover = None;
            self.needs_redraw = true;
        }
    }

    /// 设置窗口逻辑尺寸
    pub fn set_window_size(&mut self, size: (u32, u32)) {
        // 保留兼容：仅记录但不用于关键路径
        let _ = size;
    }

    /// GPU 渲染器是否存在
    pub fn gpu_renderer_is_some(&self) -> bool {
        self.gpu_renderer.is_some()
    }

    /// GPU 渲染器是否不存在
    pub fn gpu_renderer_is_none(&self) -> bool {
        self.gpu_renderer.is_none()
    }

    /// 获取 GPU 渲染器可变引用
    pub fn gpu_renderer_as_mut(&mut self) -> Option<&mut GpuRenderer> {
        self.gpu_renderer.as_mut()
    }

    /// 获取当前渲染模式
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// 调整所有 Tab 视口尺寸
    pub fn resize_all_webviews(&mut self, w: u32, h: u32) {
        self.tabs.set_viewport(w, h);
        // dsf 携带页面 zoom（CSS 视口已按 render_scale 缩小，光栅物理尺寸不变）。
        self.tabs.resize_all(w, h, self.page_render_scale());
    }

    /// 测试用：获取标签 WebView 的逻辑视口尺寸
    #[cfg(test)]
    pub fn webview_logical_size_for_tab(&self, _tab_id: zero_browser_shell::TabId) -> Option<(u32, u32)> {
        Some(self.tabs.logical_viewport())
    }

    /// 测试用：在 tab 的渲染 worker WebView 上执行 JS 并同步回读结果（单进程路径）。
    /// 供集成测试读回页面 JS 状态（如 R3294 用户滚动 listener 触发计数）。委托
    /// `TabManager::test_execute_script`（经 worker 线程 ExecuteScriptForTest 命令 + reply channel）。
    #[cfg(test)]
    pub fn test_execute_script(&self, tab_id: zero_browser_shell::TabId, script: &str) -> Result<String, String> {
        self.tabs.test_execute_script(tab_id, script)
    }

    /// 测试用：构建场景（暴露私有方法给测试模块）
    #[cfg(test)]
    pub fn build_scene_for_test(&mut self, width: u32, height: u32) -> ChromeScene {
        self.build_scene(width, height)
    }

    /// 测试用：构建场景并 CPU 渲染为帧缓冲。
    #[cfg(test)]
    pub fn render_scene_for_test(&mut self, width: u32, height: u32) -> zero_render_foundation::surface::FrameBuffer {
        let (fills, glyphs, overlay_fills, overlay_glyphs, _chrome_shadows, overlay_rounded_rects) =
            self.build_scene(width, height);
        render_scene_to_framebuffer(
            width,
            height,
            1.0,
            &fills,
            &[],
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
        )
    }

    /// 测试用：在页面内容区坐标打开右键菜单（与生产路径 `show_context_menu` 一致）。
    #[cfg(test)]
    pub fn show_context_menu_for_test(&mut self, x: f32, y: f32) {
        self.show_context_menu(x as f64, y as f64);
    }

    /// 测试用：当前是否显示菜单。
    #[cfg(test)]
    pub fn is_context_menu_visible_for_test(&self) -> bool {
        self.context_menu.visible
    }

    /// 测试用：当前展开的子菜单父项索引。
    #[cfg(test)]
    pub fn open_sub_menu_for_test(&self) -> Option<usize> {
        self.context_menu.open_sub_menu
    }

    /// 测试用：查找指定 id 的菜单项索引。
    #[cfg(test)]
    pub fn context_menu_item_index_for_test(&self, id: &str) -> Option<usize> {
        self.context_menu.items.iter().position(|i| i.id() == id)
    }

    /// 测试用：分段行（如 Zoom 行）的段 `(id, label)` 序列；非分段行返回 None。
    #[cfg(test)]
    pub fn context_menu_segments_for_test(&self, id: &str) -> Option<Vec<(String, String)>> {
        let item = self.context_menu.items.iter().find(|i| i.id() == id)?;
        Some(
            item.segments()?
                .iter()
                .map(|s| (s.id().to_string(), s.label().to_string()))
                .collect(),
        )
    }

    /// 测试用：菜单原点 x。
    #[cfg(test)]
    pub fn context_menu_x_for_test(&self) -> f32 {
        self.context_menu.x
    }

    /// 测试用：菜单原点 y。
    #[cfg(test)]
    pub fn context_menu_y_for_test(&self) -> f32 {
        self.context_menu.y
    }

    /// 测试用：标签在标签栏中的 `(x, width)` 布局（需先 `build_scene_for_test`）。
    #[cfg(test)]
    pub fn tab_layout_rect_for_test(&self, tab_id: TabId) -> Option<(f32, f32)> {
        self.tab_layout
            .iter()
            .find(|(id, _, _)| *id == tab_id)
            .map(|&(_, x, w)| (x, w))
    }

    /// 测试用：标签条可视槽位 id 序（tab_layout 原序；溢出时不含隐藏标签，
    /// 需先 `build_scene_for_test` 刷新）。
    #[cfg(test)]
    pub fn tab_layout_ids_for_test(&self) -> Vec<TabId> {
        self.tab_layout.iter().map(|&(id, _, _)| id).collect()
    }

    /// 测试用：溢出 ▾ 按钮 x（物理像素；与渲染同源，None = 无溢出）。
    #[cfg(test)]
    pub fn tab_overflow_btn_x_for_test(&self) -> Option<f32> {
        self.tab_overflow_btn_x
    }

    /// 测试用：标签拖拽是否已激活（越过拖拽阈值）。
    #[cfg(test)]
    pub fn tab_drag_active_for_test(&self) -> bool {
        self.tab_drag.is_some_and(|drag| drag.active)
    }

    /// 测试用：注入标签页渲染快照（不经过 worker，避免异步覆盖）。
    #[cfg(test)]
    pub fn inject_tab_render_for_test(
        &mut self,
        tab_id: TabId,
        render: crate::tab_snapshot::PageRenderResult,
        document_height: f32,
    ) {
        self.tabs.ensure_snapshot_for_test(tab_id);
        if let Some(snap) = self.tabs.snapshot_mut(tab_id) {
            snap.last_render = Some(render);
            snap.document_height = Some(document_height);
        }
    }

    /// 测试用：注入命中测试快照（fragment 锚点定位等主线程快照面的合成数据源）。
    #[cfg(test)]
    pub fn inject_hit_test_for_test(&mut self, tab_id: TabId, hit_test: zero_engine::HitTestCache) {
        self.tabs.ensure_snapshot_for_test(tab_id);
        if let Some(snap) = self.tabs.snapshot_mut(tab_id) {
            snap.hit_test = Some(hit_test);
        }
    }

    /// 测试用：向指定标签注入已完成的 compositor RGBA 位图。
    #[cfg(test)]
    pub fn inject_compositor_frame_for_test(
        &mut self,
        tab_id: TabId,
        surface_id: u64,
        navigation_epoch: u64,
        frame_id: u64,
        size: (u32, u32),
        rgba: Vec<u8>,
    ) {
        self.tabs.ensure_snapshot_for_test(tab_id);
        if let Some(snap) = self.tabs.snapshot_mut(tab_id) {
            snap.navigation_epoch = navigation_epoch;
            let submission = crate::tab_snapshot::CompositorSubmission {
                surface_id,
                navigation_epoch,
                frame_id,
            };
            assert!(snap.record_compositor_submission(submission));
            assert!(snap.commit_compositor_frame(submission, size.0, size.1, rgba, 0.0, 0.0));
            snap.document_width = Some(size.0 as f32);
            snap.document_height = Some(size.1 as f32);
            snap.loading = false;
        }
    }

    /// 测试用：读取指定标签当前 compositor surface。
    #[cfg(test)]
    pub fn compositor_surface_for_test(&self, tab_id: TabId) -> Option<u64> {
        self.tabs
            .snapshot(tab_id)?
            .compositor_frame
            .as_ref()
            .map(|frame| frame.surface_id)
    }

    /// 测试用：当前 Chrome 配色
    #[cfg(test)]
    pub fn chrome_palette(&self) -> colors::ChromePalette {
        self.chrome_palette
    }

    /// 测试用：当前生效的颜色方案。
    #[cfg(test)]
    pub fn color_scheme_for_test(&self) -> PrefersColorSchemeValue {
        self.color_scheme
    }

    /// 测试用：当前渲染媒体类型（DC-12 @media print；R1993）。
    #[cfg(test)]
    pub fn media_type_for_test(&self) -> zero_engine::MediaType {
        self.tabs.media_type()
    }

    /// 测试用：当前下发的打印页面设置（T61）。
    #[cfg(test)]
    pub fn print_page_setup_for_test(&self) -> Option<zero_engine::PrintPageSetup> {
        self.tabs.print_page_setup()
    }

    /// ui_driver：打印预览设置面板第 `row` 行命中矩形（渲染层写入；未渲染返回 None）。
    pub(crate) fn print_setup_panel_rect(&self, row: usize) -> Option<(f32, f32, f32, f32)> {
        self.print_setup_panel_rows.get(row).copied()
    }

    /// ui_driver：第 `row` 行 −/+ 字形起点 x（D4；未渲染/无步进行返回 None）。
    pub(crate) fn print_setup_panel_stepper_x(&self, row: usize) -> Option<(f32, f32)> {
        let (minus_x, plus_x) = self.print_setup_panel_steppers.get(row).copied()?;
        (plus_x > minus_x).then_some((minus_x, plus_x))
    }

    /// 测试用：窗口是否处于全屏。
    #[cfg(test)]
    pub fn window_is_fullscreen_for_test(&self) -> bool {
        self.window_is_fullscreen
    }

    /// 测试用：Tab 是否已有可滚动/可交互的页面内容。
    #[cfg(test)]
    fn is_tab_content_ready(&self, tab_id: TabId) -> bool {
        // compositor 发布模式下页面经 compositor 进程位图显示；浏览器侧同时
        // 解码全文档图元到 last_render（滚动回落路径用，见 process_backend）
        // ——以已提交的 compositor 帧作为内容就绪判据。
        // R3254-C11：叠加 document_height>0 门槛（legacy 判据同款）——渐进式 publish 的
        // 过渡空白帧（加载中、脚本执行前）不应提前判就绪。
        if self.tabs.compositor_frame(tab_id).is_some() && self.tabs.document_height(tab_id).is_some_and(|h| h > 0.0) {
            return true;
        }
        let has_primitives = self
            .tabs
            .last_render(tab_id)
            .is_some_and(|r| !r.primitives.fills.is_empty() || !r.primitives.glyphs.is_empty());
        let has_height = self.tabs.document_height(tab_id).is_some_and(|h| h > 0.0);
        has_primitives && has_height
    }

    /// 测试用：轮询 worker 直至页面布局与首帧就绪。
    #[cfg(test)]
    pub fn wait_for_tab_content_ready(&mut self, tab_id: TabId) {
        // R2414：上限 30s（3000×10ms）。in-process tab_worker 是独立 OS 线程，在高并行测试
        // 负载下（多 tab_worker + 测试线程争 CPU）首帧可能 >5s；旧上限 5s 致 wait 超时后
        // 测试用空/未就绪快照继续 → hover hit-test 返回 None → floating_link flake。
        // 早返（is_tab_content_ready 即 return）保证正常（<1s 完成）测试零额外开销。
        for _ in 0..3000 {
            self.tabs.poll(Some(tab_id), self.gpu_renderer_is_some());
            if self.is_tab_content_ready(tab_id) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// 测试用：向指定标签的 WebView 加载 HTML（不加锁；由 `load_webview_html` 调用）。
    #[cfg(test)]
    pub fn load_webview_html_unlocked(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        self.tabs.ensure_tab(tab_id);
        self.sync_webview_viewport();
        self.tabs.load_html(tab_id, html, css, None);
        self.wait_for_tab_content_ready(tab_id);
    }

    /// 测试用：向指定标签的 WebView 加载 HTML
    #[cfg(test)]
    pub fn load_webview_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        self.load_webview_html_unlocked(tab_id, html, css);
    }

    #[cfg(test)]
    pub fn load_webview_html_without_wait_for_test(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        self.tabs.ensure_tab(tab_id);
        self.sync_webview_viewport();
        self.tabs.load_html(tab_id, html, css, None);
    }

    #[cfg(test)]
    pub fn load_webview_html_with_url_without_wait_for_test(&mut self, tab_id: TabId, html: &str, url: &str) {
        self.tabs.ensure_tab(tab_id);
        self.sync_webview_viewport();
        self.tabs.load_html(tab_id, html, None, Some(url));
    }

    #[cfg(test)]
    pub fn set_javascript_enabled_for_test(&mut self, enabled: bool) {
        self.tabs.set_javascript_enabled(enabled);
    }

    /// 测试用：读取标签页最近一次渲染快照的序号。
    #[cfg(test)]
    pub fn snapshot_seq_for_test(&self, tab_id: TabId) -> u64 {
        self.tabs.snapshot_seq(tab_id)
    }

    /// 测试用：读取 renderer 最新提交和 Browser 最新采用的 compositor 帧序号。
    #[cfg(test)]
    pub fn compositor_frame_ids_for_test(&self, tab_id: TabId) -> (u64, u64) {
        (
            self.tabs.compositor_submission_frame_id(tab_id),
            self.tabs.compositor_frame_id(tab_id),
        )
    }

    /// 测试用：当前 live renderer 数量。
    #[cfg(test)]
    pub fn live_renderer_count_for_test(&self) -> usize {
        self.tabs.live_renderer_count_for_test()
    }

    /// 测试用：读取标签页最近快照中的 HTML。
    #[cfg(test)]
    pub fn page_html_for_test(&self, tab_id: TabId) -> Option<String> {
        self.tabs.page_html(tab_id)
    }

    /// 测试用：读取标签页最近快照中的 URL。
    #[cfg(test)]
    pub fn page_url_for_test(&self, tab_id: TabId) -> Option<String> {
        self.tabs.page_url(tab_id)
    }

    /// 测试用：读取标签页最近快照中的标题。
    #[cfg(test)]
    pub fn page_title_for_test(&self, tab_id: TabId) -> Option<String> {
        self.tabs.page_title(tab_id)
    }

    /// 测试用：读取标签页最近快照中的导航 epoch。
    #[cfg(test)]
    pub fn navigation_epoch_for_test(&self, tab_id: TabId) -> u64 {
        self.tabs.navigation_epoch_for_test(tab_id)
    }

    /// 测试用：以 GPU/合成器路径轮询渲染进程。
    #[cfg(test)]
    pub fn poll_tab_fetch_with_gpu_present_for_test(&mut self) {
        if self.tabs.poll(self.shell.active_tab_id(), true) {
            self.needs_redraw = true;
        }
    }

    /// 测试用：读取页面元素命中结果。
    #[cfg(test)]
    pub fn hit_test_page_element_for_test(&mut self, tab_id: TabId, x: f32, y: f32) -> Option<zero_engine::ElementHit> {
        self.tabs.hit_test_element(tab_id, x, y)
    }

    /// 测试用：模拟合成器帧到达前浏览器侧命中缓存缺失。
    #[cfg(test)]
    pub fn clear_page_hit_test_for_test(&mut self, tab_id: TabId) {
        self.tabs.clear_hit_test_for_test(tab_id);
    }

    #[cfg(test)]
    pub fn page_event_target_for_test(&self, tab_id: TabId) -> Option<&str> {
        self.tabs.event_target_for_test(tab_id)
    }

    /// 测试用：同步视口并等待 worker 快照更新。
    #[cfg(test)]
    pub fn sync_webview_viewport_and_poll(&mut self, tab_id: TabId) {
        self.sync_webview_viewport();
        self.wait_for_tab_content_ready(tab_id);
    }

    /// 平台感知的修饰键（macOS 用 Cmd，其他平台用 Ctrl）
    fn is_modifier_pressed(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.cmd_pressed
        } else {
            self.ctrl_pressed
        }
    }

    /// 测试用：获取修饰键状态（平台感知：macOS 用 Cmd，其他用 Ctrl）
    #[cfg(test)]
    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_modifier_pressed()
    }

    /// 测试用：全部修饰键按下态 `(ctrl, cmd, shift, alt)`——T53 失焦复位断言面。
    #[cfg(test)]
    pub fn modifier_states_for_test(&self) -> (bool, bool, bool, bool) {
        (
            self.ctrl_pressed,
            self.cmd_pressed,
            self.shift_pressed,
            self.alt_pressed,
        )
    }

    /// 测试用：设置平台修饰键（macOS 用 Meta，其他用 Control）
    #[cfg(test)]
    pub fn test_modifier_key_name() -> &'static str {
        if cfg!(target_os = "macos") { "Meta" } else { "Control" }
    }

    /// 测试用：获取地址栏文本
    #[cfg(test)]
    pub fn address_bar_text(&self) -> &str {
        self.address_bar.text()
    }

    /// 测试用：获取地址栏光标位置
    #[cfg(test)]
    pub fn address_bar_cursor(&self) -> usize {
        self.address_bar.cursor()
    }

    /// 测试用：地址栏编辑态水平滚动偏移（全文像素坐标，T37 光标跟随滚动）
    #[cfg(test)]
    pub fn address_bar_scroll_x_for_test(&self) -> f32 {
        self.address_bar_scroll_x
    }

    /// 测试用：地址栏文本绘制起点 x（物理像素）
    #[cfg(test)]
    pub fn address_bar_text_origin_x_for_test(&self) -> f32 {
        self.address_bar_text_origin_x()
    }

    /// 测试用：地址栏光标闪烁相位基准（T37 编辑/聚焦重置）
    #[cfg(test)]
    pub fn address_bar_cursor_phase_for_test(&self) -> std::time::Instant {
        self.address_bar_cursor_phase
    }

    /// 测试用：地址栏 IME 预编辑文本
    #[cfg(test)]
    pub fn address_bar_ime_preedit_for_test(&self) -> &str {
        &self.address_bar_ime_preedit
    }

    /// 测试用：地址栏选区字符范围（None = 无选区）
    #[cfg(test)]
    pub fn address_bar_selection_for_test(&self) -> Option<(usize, usize)> {
        self.address_bar
            .has_selection()
            .then(|| self.address_bar.selection_char_range())
    }

    /// 测试用：获取标签页垂直滚动偏移（物理像素）
    #[cfg(test)]
    pub fn scroll_offset_for_tab(&self, tab_id: TabId) -> f32 {
        self.scroll.get(&tab_id).map(|s| s.y).unwrap_or(0.0)
    }

    /// 测试用：设置标签页的本地滚动状态。
    #[cfg(test)]
    pub fn set_tab_scroll_for_test(&mut self, tab_id: TabId, scroll: TabScrollState) {
        self.scroll.insert(tab_id, scroll);
    }

    /// 测试用：标记当前 compositor 位图已烘焙的文档滚动偏移。
    #[cfg(test)]
    pub fn set_compositor_scroll_for_test(&mut self, tab_id: TabId, scroll: TabScrollState) {
        if let Some(snapshot) = self.tabs.snapshot_mut(tab_id) {
            snapshot.compositor_scroll = Some((scroll.x, scroll.y));
        }
    }

    /// 当前标签页滚动状态。
    pub fn tab_scroll_state(&self, tab_id: TabId) -> TabScrollState {
        self.scroll.get(&tab_id).copied().unwrap_or_default()
    }

    /// 文档内容尺寸（物理像素）。
    /// 活动标签页的页面缩放系数（Chrome page zoom 语义：影响 CSS 布局视口与
    /// 所有 CSS↔物理换算，不影响 chrome UI）。值域由 shell `set_zoom` 钳制。
    fn page_zoom(&self) -> f32 {
        self.shell.zoom().max(f32::EPSILON)
    }

    /// 页面渲染合成缩放——物理像素 ↔ CSS 像素的统一换算因子（device scale × 页面 zoom）。
    ///
    /// Chrome 模型：CSS 布局视口 = 内容物理尺寸 ÷ 本值；renderer 光栅 dsf = 本值
    /// （位图物理尺寸恒等于内容区，合成 1:1，内容随 zoom 相对变大/变小）。
    fn page_render_scale(&self) -> f32 {
        self.scale_factor * self.page_zoom()
    }

    /// 页面 zoom 变化后的统一入口：钳制滚动 + 重推 CSS 视口（renderer 随新视口
    /// 重布局出新帧）+ 重绘。zoom 键与 Ctrl+滚轮缩放共用。
    /// T24：缩放经此生效的按当前页 origin 记入 `zoom_by_origin` 并持久化。
    fn apply_page_zoom_change(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.clamp_tab_scroll(tab_id);
        }
        self.sync_webview_viewport();
        self.record_zoom_override_for_active_page();
        self.needs_redraw = true;
    }

    /// T24：把当前活动页的缩放按 origin 记入设置（Chrome per-site zoom）。
    /// 非 http(s)/file 页面（zero:// 内部页等）无站点语义，不记录。
    fn record_zoom_override_for_active_page(&mut self) {
        let Some(url) = self.shell.active_tab().and_then(|tab| tab.url().map(str::to_string)) else {
            return;
        };
        let Some(origin) = zoom_origin_key(&url) else {
            return;
        };
        let zoom = self.shell.zoom();
        self.shell.apply_settings(|settings| {
            settings.zoom_by_origin.insert(origin, zoom);
        });
    }

    /// T24：导航时恢复目标 origin 的持久化缩放；无记录的 http(s)/file 页面回
    /// 设置默认缩放（Chrome 行为：切到别的站点不带上一站的缩放）。zero:// 内部
    /// 页等无站点语义的 URL 不干预。
    fn restore_zoom_for_url(&mut self, url: &str) {
        let Some(origin) = zoom_origin_key(url) else {
            return;
        };
        let want = self
            .shell
            .settings()
            .zoom_by_origin
            .get(&origin)
            .copied()
            .unwrap_or(self.shell.settings().default_zoom);
        if (self.shell.zoom() - want).abs() <= f32::EPSILON {
            return;
        }
        self.shell.set_zoom(want);
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.clamp_tab_scroll(tab_id);
        }
        self.sync_webview_viewport();
        self.needs_redraw = true;
    }

    fn document_size_physical(&self, tab_id: TabId) -> (f32, f32) {
        let s = self.page_render_scale();
        let layout_h = self.tabs.document_height(tab_id).filter(|height| *height > 0.0);
        // Root scrolling must stop at the last drawable document primitive. A
        // layout-only extent can include unsupported positioned descendants;
        // letting it win creates a scrollbar into blank compositor frames.
        // https://drafts.csswg.org/css-overflow-3/#scrollable
        let painted_h = self.tabs.painted_content_height(tab_id).or_else(|| {
            let (_, _, content_w, content_h) = self.page_content_rect();
            self.tabs.last_render(tab_id).map(|r| {
                page_scroll::primitives_scrollable_content_height(
                    &r.primitives,
                    content_w / s.max(f32::EPSILON),
                    content_h / s.max(f32::EPSILON),
                )
            })
        });
        let logical_h = match (layout_h, painted_h) {
            (Some(layout), Some(painted)) => layout.min(painted),
            (Some(layout), None) => layout,
            (None, Some(painted)) => painted,
            (None, None) => 0.0,
        };
        // 性能门禁优化 S3（2026-08-08）：宽度已随快照缓存（每快照一次 O(P) 扫描），
        // 不再在每次 mousemove/wheel 上扫全部图元；缓存缺失（旧快照/异常路径）回退扫描
        let logical_w = self
            .tabs
            .document_width(tab_id)
            .or_else(|| {
                self.tabs
                    .last_render(tab_id)
                    .map(|r| page_scroll::primitives_content_width(&r.primitives))
            })
            .unwrap_or(0.0);
        (logical_w * s, logical_h * s)
    }

    /// 按窗口尺寸计算标签页视口与滚动条布局。
    pub fn page_scroll_layout_for(&self, tab_id: TabId, width: u32, height: u32) -> page_scroll::PageScrollLayout {
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let (doc_w, doc_h) = self.document_size_physical(tab_id);
        page_scroll::compute_page_scroll_layout(cx, cy, cw, ch, doc_w, doc_h, self.scale_factor)
    }

    /// 当前窗口下活跃标签页的视口与滚动条布局。
    pub fn page_scroll_layout(&self, tab_id: TabId) -> page_scroll::PageScrollLayout {
        self.page_scroll_layout_for(tab_id, self.physical_size.0, self.physical_size.1)
    }

    fn clamp_tab_scroll(&mut self, tab_id: TabId) {
        let layout = self.page_scroll_layout(tab_id);
        let entry = self.scroll.entry(tab_id).or_default();
        *entry = page_scroll::clamp_scroll(*entry, &layout);
    }

    /// T26：帧循环推进滚轮平滑滚动动画——各动画标签的当前偏移设为 eased 值，
    /// 完成时落到目标并钳制；合成器路径逐帧推送异步滚动。仍有标签在动画时保持
    /// 重绘请求。主循环每次事件迭代调用（动画期间经 request_redraw 连续成帧）。
    pub fn tick_smooth_scroll(&mut self) {
        if self.scroll_anim.is_empty() {
            return;
        }
        let now = Instant::now();
        let mut finished: Vec<TabId> = Vec::new();
        for (tab_id, anim) in &self.scroll_anim {
            let done = anim.is_finished(now);
            let value = if done { anim.target() } else { anim.value_at(now) };
            // T36：中间帧取整到整数物理像素。滚动呈现是位图平移，亚像素偏移会让
            // translate-blit（can_blit 的 same_fraction 守卫）逐帧失效，滚动动画期
            // 退化为全量重光栅（长页面单帧 ~40ms）；取整后动画帧与既有整数滚轮
            // 路径共享同一 blit 机制，终值仍精确落在整数目标上。
            let value = page_scroll::TabScrollState {
                x: value.x.round(),
                y: value.y.round(),
            };
            // 滚轮可能早于任何直接滚动：写入时补建条目（与增量路径 or_default 同语义）。
            *self.scroll.entry(*tab_id).or_default() = value;
            if crate::compositor_client::async_scroll_enabled()
                && let Some(submission) = self.tabs.snapshot(*tab_id).and_then(|snap| snap.compositor_submission)
            {
                crate::compositor_client::set_scroll(
                    submission.surface_id,
                    submission.navigation_epoch,
                    submission.frame_id,
                    value.x,
                    value.y,
                );
            }
            if done {
                finished.push(*tab_id);
            }
        }
        for tab_id in finished {
            self.scroll_anim.remove(&tab_id);
            self.clamp_tab_scroll(tab_id);
        }
        self.needs_redraw = true;
    }

    /// 测试用：是否有进行中的平滑滚动动画。
    #[cfg(test)]
    pub fn tick_smooth_scroll_has_animation(&self) -> bool {
        !self.scroll_anim.is_empty()
    }

    /// 计算网页内容区域物理像素尺寸（用于滚动、合成区域）
    pub fn content_physical_size(&self) -> (u32, u32) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            let layout = self.page_scroll_layout(tab_id);
            return (layout.viewport_w.max(0.0) as u32, layout.viewport_h.max(0.0) as u32);
        }
        let (_, _, w, h) = self.page_content_rect();
        (w.max(0.0) as u32, h.max(0.0) as u32)
    }

    /// WebView 布局视口（CSS 逻辑像素，与 devicePixelRatio 对应）。
    ///
    /// 高度用 `floor` 而非 `round`，保证 `logical_h * scale_factor` 不超过内容区物理高度，
    /// 避免页面背景在底部溢出并盖住圆角。
    pub fn content_logical_size(&self) -> (u32, u32) {
        let s = self.page_render_scale().max(f32::EPSILON);
        if let Some(tab_id) = self.shell.active_tab_id() {
            let layout = self.page_scroll_layout(tab_id);
            let logical_w = (layout.viewport_w / s).floor().max(1.0) as u32;
            let logical_h = if layout.viewport_h <= f32::EPSILON {
                0
            } else {
                (layout.viewport_h / s).floor().max(1.0) as u32
            };
            return (logical_w, logical_h);
        }
        let (_, _, w, h) = self.page_content_rect();
        let logical_w = (w / s).floor().max(1.0) as u32;
        let logical_h = if h <= f32::EPSILON {
            0
        } else {
            (h / s).floor().max(1.0) as u32
        };
        (logical_w, logical_h)
    }

    /// 页面视口外框（物理像素）：含圆角与边框的 `(x, y, w, h)`。
    pub fn page_frame_rect(&self) -> (f32, f32, f32, f32) {
        self.page_frame_rect_for(self.physical_size.0, self.physical_size.1)
    }

    /// 按指定窗口物理尺寸计算视口外框（渲染与布局应使用同一组 `(width, height)`）。
    pub fn page_frame_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let chrome_top = self.chrome_top_y_for(s);
        let inset_h = layout::PAGE_FRAME_INSET_H * s;
        let inset_top = layout::PAGE_FRAME_INSET_TOP * s;
        let inset_bottom = layout::PAGE_FRAME_INSET_BOTTOM * s;
        let (clip_guard, ui_guard) = self.page_frame_bottom_reserves(s);
        let x = inset_h;
        let y = chrome_top + inset_top;
        let w = width as f32 - 2.0 * inset_h;
        let h = (height as f32 - y - inset_bottom - clip_guard - ui_guard).max(0.0);
        (x, y, w, h)
    }

    /// 视口外框底边 + 下间距（页面 gutter 底边；浮动 UI 锚点）。
    pub fn page_frame_bottom_y_for(&self, width: u32, height: u32) -> f32 {
        let (_, fy, _, fh) = self.page_frame_rect_for(width, height);
        let _ = height;
        fy + fh + layout::PAGE_FRAME_INSET_BOTTOM * self.scale_factor
    }

    /// 运行时有效的视口圆角（物理像素）。最大化/全屏时归零（避免内容被任务栏/边遮挡）。
    pub(crate) fn effective_page_frame_radius(&self) -> f32 {
        if self.window_is_maximized || self.window_is_fullscreen {
            0.0
        } else {
            layout::PAGE_FRAME_RADIUS * self.scale_factor
        }
    }

    /// 运行时有效的视口边框宽度（物理像素）。最大化/全屏时归零。
    pub(crate) fn effective_page_frame_border(&self) -> f32 {
        if self.window_is_maximized || self.window_is_fullscreen {
            0.0
        } else {
            layout::PAGE_FRAME_BORDER * self.scale_factor
        }
    }

    /// 上下文菜单中第 `idx` 项的 y 偏移（物理像素，相对菜单顶部）。
    ///
    /// separator 项使用紧凑行高 `CONTEXT_MENU_SEPARATOR_HEIGHT`，
    /// 普通项使用 `CONTEXT_MENU_ROW_HEIGHT`，以避免分隔线撑大菜单。
    pub(crate) fn context_menu_row_y(&self, idx: usize) -> f32 {
        let s = self.scale_factor;
        let normal = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        self.context_menu
            .items
            .iter()
            .take(idx)
            .map(|it| if it.is_separator() { sep } else { normal })
            .sum()
    }

    /// 上下文菜单总高度（物理像素），separator 项计入紧凑高度。
    pub(crate) fn context_menu_total_height(&self) -> f32 {
        Self::menu_items_total_height(&self.context_menu.items, self.scale_factor)
    }

    /// 子菜单面板中第 `child_idx` 项的 y 偏移（物理像素，相对子面板顶部）。
    /// 子菜单内 separator 同样使用紧凑高度。
    pub(crate) fn sub_menu_row_y(&self, parent_idx: usize, child_idx: usize) -> f32 {
        let s = self.scale_factor;
        let normal = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        let Some(parent) = self.context_menu.items.get(parent_idx) else {
            return 0.0;
        };
        let Some(children) = parent.children() else {
            return 0.0;
        };
        children
            .iter()
            .take(child_idx)
            .map(|it| if it.is_separator() { sep } else { normal })
            .sum()
    }

    /// 子菜单面板总高度（物理像素）。
    pub(crate) fn sub_menu_total_height(&self, parent_idx: usize) -> f32 {
        let s = self.scale_factor;
        let normal = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        let Some(parent) = self.context_menu.items.get(parent_idx) else {
            return 0.0;
        };
        let Some(children) = parent.children() else {
            return 0.0;
        };
        children
            .iter()
            .map(|it| if it.is_separator() { sep } else { normal })
            .sum()
    }

    /// 子菜单面板的矩形 `(x, y, w, h)`（物理像素）。
    ///
    /// 默认紧贴主菜单右侧；当右侧空间不足以容纳子菜单宽度时，
    /// 改为紧贴主菜单左侧显示，避免溢出屏幕（如全局菜单按钮位于地址栏右侧时）。
    pub(crate) fn sub_menu_panel_rect(&self, parent_idx: usize) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let sub_h = self.sub_menu_total_height(parent_idx);
        let sub_y = menu_y + self.context_menu_row_y(parent_idx);

        let screen_w = self.physical_size.0 as f32;
        let right_gap = screen_w - (menu_x + menu_w);
        let sub_x = if right_gap >= menu_w + 1.0 * s {
            menu_x + menu_w + 1.0 * s
        } else {
            menu_x - menu_w - 1.0 * s
        };
        (sub_x, sub_y, menu_w, sub_h)
    }

    /// 按指定窗口物理尺寸计算内容区（边框内侧）。
    pub fn page_content_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let (x, y, w, h) = self.page_frame_rect_for(width, height);
        let border = self.effective_page_frame_border();
        (
            x + border,
            y + border,
            (w - 2.0 * border).max(0.0),
            (h - 2.0 * border).max(0.0),
        )
    }

    /// 页面内容区（物理像素，边框内侧）：WebView 绘制与命中区域。
    pub fn page_content_rect(&self) -> (f32, f32, f32, f32) {
        self.page_content_rect_for(self.physical_size.0, self.physical_size.1)
    }

    /// 产品一致性验收：当前活动标签页。
    pub fn parity_active_tab_id(&self) -> Option<TabId> {
        self.shell.active_tab_id()
    }

    /// 首帧不能代表 webfont 等初始资源已加载；采集器须等同一导航的完成事件。
    pub fn parity_resources_settled(&self) -> bool {
        self.shell
            .active_tab_id()
            .and_then(|id| self.tabs.snapshot(id))
            .is_some_and(|snapshot| snapshot.resources_settled)
    }

    /// 产品一致性验收：最新可显示 compositor 页面帧序号。
    pub fn parity_compositor_frame_id(&self, tab_id: TabId) -> u64 {
        self.tabs.compositor_frame_id(tab_id)
    }

    pub(crate) fn parity_set_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.apply_color_scheme(scheme);
    }

    /// 产品一致性验收：在 live renderer 的页面上下文执行观察脚本。
    pub fn parity_execute_script(
        &mut self,
        tab_id: TabId,
        script: String,
        timeout: Duration,
    ) -> Result<zero_protocol::message::AutomationValue, String> {
        self.tabs.execute_script_for_parity(tab_id, script, timeout)
    }

    /// 浮动查找栏外框（物理像素）。
    pub(crate) fn find_bar_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (cx, cy, cw, _ch) = self.page_content_rect_for(width, height);
        let bar_w = layout::FIND_BAR_WIDTH * s;
        let bar_h = layout::FIND_BAR_HEIGHT * s;
        let margin = layout::FIND_BAR_FLOAT_MARGIN * s;
        (cx + cw - bar_w - margin, cy + margin, bar_w, bar_h)
    }

    /// 缩放百分比浮层外框（物理像素）。未显示（无 zoom 操作或已超 3 秒）返回 None。
    /// 几何与 render_zoom_indicator 同源（渲染与点击命中共用，避免两处漂移）。
    pub(crate) fn zoom_indicator_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        let start = self.zoom_indicator_start?;
        if start.elapsed().as_secs_f32() > 3.0 || self.font_id.is_none() {
            return None;
        }
        let s = self.scale_factor;
        let zoom = self.shell.zoom();
        // 100% 时不显示（zoom_reset 到 100% 仍短暂提示，便于用户确认复位）。
        let label = if (zoom - 1.0).abs() < 0.001 {
            "100%".to_string()
        } else {
            format!("{:.0}%", (zoom * 100.0).round())
        };
        let font_size = layout::CHROME_FONT_SIZE * s;
        let label_w = self.measure_ui_text_width(&label, font_size);
        let pad_h = 12.0 * s;
        let pad_v = 8.0 * s;
        let box_w = label_w + pad_h * 2.0;
        let box_h = font_size + pad_v * 2.0;
        let margin = 16.0 * s;
        // 右下角，避开滚动条。
        let scrollbar_w = layout::SCROLLBAR_THICKNESS * s;
        let box_x = width as f32 - box_w - margin - scrollbar_w;
        let box_y = height as f32 - box_h - margin;
        Some((box_x, box_y, box_w, box_h))
    }

    /// 下载完成 toast 外框 `(x, y, w, h)`（物理像素）。未显示（无 toast、已超 3 秒
    /// 或字体未就绪）返回 None。几何与 render_download_toast / smoke 断言面同源。
    pub(crate) fn download_toast_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        let (filename, start) = self.download_toast.as_ref()?;
        if start.elapsed().as_secs_f32() > layout::DOWNLOAD_TOAST_DURATION_SECS || self.font_id.is_none() {
            return None;
        }
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let status = Self::download_toast_status_label();
        let pad_h = 14.0 * s;
        let text_w = self
            .measure_ui_text_width(filename, font_size)
            .max(self.measure_ui_text_width(&status, font_size))
            .min(layout::DOWNLOAD_TOAST_MAX_WIDTH * s);
        let box_w = (text_w + pad_h * 2.0).min(layout::DOWNLOAD_TOAST_MAX_WIDTH * s + pad_h * 2.0);
        let line_gap = 4.0 * s;
        let box_h = font_size * 2.0 + line_gap + 12.0 * s;
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        // 页面内容区底部居中（Chrome snackbar 位）；右下角留给下载面板，左下角留给状态栏。
        let box_x = cx + (cw - box_w) * 0.5;
        let box_y = cy + ch - box_h - 16.0 * s;
        Some((box_x, box_y, box_w, box_h))
    }

    /// 下载完成 toast 的状态行文案（渲染与几何同源）。
    fn download_toast_status_label() -> String {
        match UiLanguage::detect_from_env() {
            UiLanguage::ZhCn => "下载完成".to_string(),
            UiLanguage::EnUs => "Download complete".to_string(),
        }
    }

    /// 关闭标签撤销 toast 外框 `(x, y, w, h)`（物理像素）。未显示（无 toast、已超
    /// 5 秒或字体未就绪）返回 None。几何与 render_closed_tab_toast / smoke 断言面同源。
    pub(crate) fn closed_tab_toast_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        let start = self.closed_tab_toast?;
        if start.elapsed().as_secs_f32() > layout::CLOSED_TAB_TOAST_DURATION_SECS || self.font_id.is_none() {
            return None;
        }
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let pad_h = 14.0 * s;
        let gap = 18.0 * s;
        let text_w = self.measure_ui_text_width(&Self::closed_tab_toast_label(), font_size)
            + gap
            + self.measure_ui_text_width(&Self::closed_tab_toast_undo_label(), font_size);
        let box_w = text_w + pad_h * 2.0;
        let box_h = font_size + 12.0 * s;
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        // 页面内容区底部居中（下载完成 toast 同位；两者同时出现的概率低，
        // 且撤销 toast 只持续 5 秒）。
        let box_x = cx + (cw - box_w) * 0.5;
        let box_y = cy + ch - box_h - 16.0 * s;
        Some((box_x, box_y, box_w, box_h))
    }

    /// 撤销 toast 的「撤销」动作命中区（toast 右段，物理像素；渲染与点击、
    /// ui_driver 点击定位同源）。未显示返回 None。
    pub(crate) fn closed_tab_toast_undo_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        let (box_x, box_y, box_w, box_h) = self.closed_tab_toast_rect_for(width, height)?;
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let undo_w = self.measure_ui_text_width(&Self::closed_tab_toast_undo_label(), font_size);
        let pad_h = 14.0 * s;
        Some((box_x + box_w - pad_h - undo_w, box_y, undo_w + pad_h * 0.5, box_h))
    }

    /// 关闭标签撤销 toast 的主文案（渲染与几何同源）。
    fn closed_tab_toast_label() -> String {
        match UiLanguage::detect_from_env() {
            UiLanguage::ZhCn => "标签已关闭".to_string(),
            UiLanguage::EnUs => "Tab closed".to_string(),
        }
    }

    /// 关闭标签撤销 toast 的动作文案（渲染与几何同源）。
    fn closed_tab_toast_undo_label() -> String {
        match UiLanguage::detect_from_env() {
            UiLanguage::ZhCn => "撤销".to_string(),
            UiLanguage::EnUs => "Undo".to_string(),
        }
    }

    /// 浮动下载面板外框（物理像素）。有活动下载时高度随行数增长（T8），
    /// 完成视图保持固定高度。
    pub(crate) fn download_panel_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let panel_w = layout::DOWNLOAD_PANEL_WIDTH * s;
        let panel_h = self.download_panel_height_for(s);
        let margin = layout::DOWNLOAD_PANEL_FLOAT_MARGIN * s;
        (cx + cw - panel_w - margin, cy + ch - panel_h - margin, panel_w, panel_h)
    }

    /// 下载面板高度：活动视图 = 头部 + 可见行数 × 行高（超出折叠为计数行）+ 底部留白。
    fn download_panel_height_for(&self, s: f32) -> f32 {
        let active = self.active_download_count();
        if active == 0 {
            return layout::DOWNLOAD_PANEL_HEIGHT * s;
        }
        let rows = active.min(layout::DOWNLOAD_PANEL_MAX_VISIBLE_ROWS) as f32;
        let mut h = layout::DOWNLOAD_PANEL_HEADER_HEIGHT
            + rows * layout::DOWNLOAD_PANEL_ROW_HEIGHT
            + layout::DOWNLOAD_PANEL_BOTTOM_PAD;
        if active > layout::DOWNLOAD_PANEL_MAX_VISIBLE_ROWS {
            h += layout::DOWNLOAD_PANEL_OVERFLOW_HEIGHT;
        }
        h * s
    }

    /// 活动下载条数。
    fn active_download_count(&self) -> usize {
        self.shell.downloads().active_count()
    }

    /// 下载面板第 `index` 行的顶边 y（物理像素；渲染与命中测试同源）。
    pub(crate) fn download_panel_row_y(panel_y: f32, index: usize, s: f32) -> f32 {
        panel_y + (layout::DOWNLOAD_PANEL_HEADER_HEIGHT + index as f32 * layout::DOWNLOAD_PANEL_ROW_HEIGHT) * s
    }

    /// 下载面板行内暂停/取消按钮矩形 `(pause_x, cancel_x, btn_y, btn_size)`（物理像素；
    /// 渲染与命中测试同源）。
    pub(crate) fn download_panel_button_rects(panel_x: f32, panel_w: f32, row_y: f32, s: f32) -> (f32, f32, f32, f32) {
        let size = layout::DOWNLOAD_PANEL_BUTTON_SIZE * s;
        let cancel_x = panel_x + panel_w - layout::DOWNLOAD_PANEL_BUTTON_MARGIN * s - size;
        let pause_x = cancel_x - layout::DOWNLOAD_PANEL_BUTTON_GAP * s - size;
        let btn_y = row_y + (layout::DOWNLOAD_PANEL_ROW_HEIGHT * s - size) * 0.5;
        (pause_x, cancel_x, btn_y, size)
    }

    /// 下载面板行按钮命中测试：返回 `(下载 id, 是否取消)`。矩形与渲染同源。
    pub(crate) fn download_panel_button_hit_at(
        &self,
        x_f: f32,
        y_f: f32,
    ) -> Option<(zero_browser_shell::DownloadId, bool)> {
        if !self.should_show_download_panel() || self.active_download_count() == 0 {
            return None;
        }
        let (panel_x, panel_y, panel_w, _) = self.download_panel_rect_for(self.physical_size.0, self.physical_size.1);
        let s = self.scale_factor;
        let visible = self
            .shell
            .downloads()
            .iter()
            .filter(|d| d.is_active())
            .take(layout::DOWNLOAD_PANEL_MAX_VISIBLE_ROWS);
        for (index, dl) in visible.enumerate() {
            let row_y = Self::download_panel_row_y(panel_y, index, s);
            let (pause_x, cancel_x, btn_y, size) = Self::download_panel_button_rects(panel_x, panel_w, row_y, s);
            let in_box = |bx: f32| x_f >= bx && x_f < bx + size && y_f >= btn_y && y_f < btn_y + size;
            if in_box(cancel_x) {
                return Some((dl.id(), true));
            }
            if in_box(pause_x) {
                return Some((dl.id(), false));
            }
        }
        None
    }

    /// 下载面板「Show in folder」命中区（与渲染同源：面板可见且最近完成项存在时
    /// 返回面板底部右侧条带矩形，物理像素）。None = 无可命中按钮。
    pub(crate) fn download_panel_action_rect_for(&self, width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
        if !self.should_show_download_panel() {
            return None;
        }
        let has_completed = self.shell.downloads().iter().any(|d| d.is_completed());
        if !has_completed {
            return None;
        }
        let (px, py, pw, ph) = self.download_panel_rect_for(width, height);
        let s = self.scale_factor;
        Some((px, py + ph - 28.0 * s, pw, 28.0 * s))
    }

    /// 地址栏页面类型（由 URL 推导，UI-agnostic 规则）。
    pub(crate) fn address_bar_page_kind(url: Option<&str>) -> AddressBarPageKind {
        match url {
            None => AddressBarPageKind::Internal,
            Some(u) if u.starts_with("https://") => AddressBarPageKind::Secure,
            Some(u) if u.starts_with("http://") => AddressBarPageKind::Insecure,
            Some(u) if u.starts_with("zero://") => AddressBarPageKind::Internal,
            Some(u) if u.starts_with("file://") => AddressBarPageKind::Local,
            _ => AddressBarPageKind::Unknown,
        }
    }

    /// 按当前窗口尺寸同步所有 Tab 的逻辑视口
    pub fn sync_webview_viewport(&mut self) {
        let (w, h) = self.content_logical_size();
        self.resize_all_webviews(w, h);
    }

    /// 获取或创建活跃标签页 runtime
    pub fn ensure_webview(&mut self, tab_id: TabId) {
        self.tabs.ensure_tab(tab_id);
    }

    fn load_local_tab_url(&mut self, tab_id: TabId, url: &str) {
        match url {
            "zero://settings" => self.open_settings_page(),
            // 历史页（含 `?q=` 过滤形态，F5 刷新走这里而非 try_apply_internal_url）。
            "zero://history" => self.open_history_page(),
            u if u.starts_with("zero://history?q=") => {
                let query = Self::percent_decode(u.strip_prefix("zero://history?q=").unwrap_or(u));
                self.open_history_page_filtered(Some(&query));
            }
            "zero://downloads" => self.open_downloads_page(),
            "zero://bookmarks" => self.open_bookmarks_page(),
            // 导入页（设置页「导入书签」入口；F5 刷新走这里而非 try_apply_internal_url，
            // 同历史页模式）。apply 为一次性动作 URL，不进刷新分派。
            "zero://bookmarks/import" => self.open_bookmarks_import_page(None),
            // 书签页过滤/编辑/删除/保存路由与地址栏入口共用同一分派（F5 刷新与
            // 渲染端表单 GET 提交走这里而非 try_apply_internal_url，同历史页模式）。
            u if u.starts_with("zero://bookmarks?")
                || u.starts_with("zero://bookmarks/edit?id=")
                || u.starts_with("zero://bookmarks/delete?id=")
                || u.starts_with("zero://bookmarks/save?") =>
            {
                self.try_apply_bookmarks_url(u);
            }
            // T62-D2：文件导入动作 URL 为一次性语义——F5/worker GET 刷新分派一律
            // 回导入页（不重复执行导入；表单提交的实际执行走渲染端
            // TabNavigateRequest → navigate_to_request → try_apply_internal_url）。
            u if u.starts_with("zero://bookmarks/import/file?") => {
                self.open_bookmarks_import_page(None);
            }
            "zero://shortcuts" => self.open_shortcuts_page(),
            // 反馈页（F5 刷新走这里而非 try_apply_internal_url，同历史页模式）；
            // remove/export 为一次性动作 URL，执行后立即回列表页。
            "zero://feedback" => self.open_feedback_page(),
            u if u.starts_with("zero://") => self.load_welcome_page(tab_id),
            _ => self.tabs.navigate(tab_id, url.to_string()),
        }
    }

    fn finish_tab_load(&mut self, tab_id: TabId, url: &str, title: &str) {
        self.shell.on_page_loaded(title);
        self.refresh_tab_favicon(tab_id, url);
    }

    fn schedule_tab_fetch(&mut self, tab_id: TabId, url: String) {
        self.shell.set_tab_crashed(tab_id, false);
        if self.shell.active_tab_id() == Some(tab_id)
            && let Some(tab) = self.shell.active_tab_mut()
        {
            tab.set_loading(true);
        }
        self.tab_fetch = TabFetchState::WaitingPaint(tab_id, url);
        self.needs_redraw = true;
    }

    /// 启动标签页加载（多进程下立即发 IPC，不等到下一帧 paint）。
    fn start_tab_load(&mut self, tab_id: TabId, url: String) {
        self.start_tab_request(tab_id, url, "GET".to_string(), None);
    }

    fn start_tab_request(&mut self, tab_id: TabId, url: String, method: String, body: Option<String>) {
        // T24：导航换页先按目标 origin 恢复持久化缩放（在推视口前生效，
        // 首帧即用恢复后的 zoom 渲染）。
        self.restore_zoom_for_url(&url);
        if self.shell.active_tab_id() == Some(tab_id)
            && let Some(tab) = self.shell.active_tab_mut()
        {
            tab.set_loading(true);
        }
        self.ensure_webview(tab_id);
        self.sync_webview_viewport();

        if url == "zero://settings" {
            if method == "GET" && body.is_none() {
                self.open_settings_page();
            }
        } else if url.starts_with("http://") || url.starts_with("https://") {
            tracing::info!("Tab {} navigate IPC: {method} {url}", tab_id.0);
            self.tabs.navigate_request(tab_id, url, method, body);
        } else if method == "GET" && body.is_none() {
            self.load_local_tab_url(tab_id, &url);
        }
        self.needs_redraw = true;
    }

    /// 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) {
        self.navigate_to_request(url, "GET".to_string(), None);
    }

    /// UI 入口导航（地址栏/自动补全/主页/书签栏/启动 URL）：用户可信路径，为
    /// T62-D2 文件导入动作 URL 置一次性放行位（内容路径 navigate_to 不置位）。
    pub fn navigate_from_ui(&mut self, url: &str) {
        if url.starts_with("zero://bookmarks/import/file?") {
            self.allow_file_import_once = true;
        }
        self.navigate_to(url);
    }

    fn navigate_to_request(&mut self, url: &str, method: String, body: Option<String>) {
        // 同文档 fragment 导航（链接点击路径已按文档 base 解析为绝对 URL）先于
        // 内部页分派与 normalize_url：仅差 fragment 的目标不重载文档、不落搜索兜底。
        if method == "GET"
            && body.is_none()
            && (self.navigate_to_fragment_anchor(url) || self.try_apply_internal_url(url))
        {
            return;
        }
        if !matches!(method.as_str(), "GET" | "POST")
            || (method == "GET" && body.is_some())
            || (method == "POST" && body.is_none())
        {
            tracing::warn!("ignoring invalid document navigation request: {method} {url}");
            return;
        }

        let url = normalize_url(&resolve_path_relative_url(url, &self.shell), &self.shell);

        // view-source:<url>：拆出内层 URL 抓取源码并以内部页展示（Chrome 地址栏行为；
        // 该 scheme 未被 WHATWG 标准化，无规范条目可引）。
        if method == "GET"
            && body.is_none()
            && let Some(inner) = url.strip_prefix("view-source:")
        {
            self.open_view_source_page(inner);
            return;
        }

        tracing::info!("Navigating to: {method} {url}");

        // 离开当前历史条目前保存滚动位置（back/forward 回到时恢复，Chrome 同款）。
        // 新导航使任何挂起的 back/forward 恢复失效（该条目将按新访问从顶部开始）。
        self.pending_scroll_restore = None;
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.save_history_scroll(tab_id);
        }
        self.shell.navigate(&url);
        self.address_bar.set_text(url.clone());
        self.autocomplete.clear();
        // 导航后页面内容更替，查找匹配缓存失效（查找栏若仍开着，加载完成后重算）。
        self.find_match_ranges.clear();
        self.find_matches_query.clear();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);

        // 重置滚动偏移与页面选区。新条目从顶部开始：清除该索引位可能残留的
        // 旧偏移（前进历史被截断后索引会被新条目复用）。
        self.scroll.insert(tab_id, TabScrollState::default());
        if let Some(index) = self.shell.tab(tab_id).map(|t| t.history_index()) {
            self.history_scroll.remove(&(tab_id, index));
        }
        self.scroll_anim.remove(&tab_id);
        self.page_selection.remove(&tab_id);
        self.clear_tab_favicon(tab_id);

        self.start_tab_request(tab_id, url, method, body);
    }

    /// 同文档 fragment 导航（HTML navigate-to-a-fragment）：目标 URL 除 fragment 外
    /// 与当前文档一致时不重新加载文档——push 历史条目并滚动/跳转到锚点。
    /// 锚点不存在仅更新 URL 不滚动（Chrome 同款）；空 fragment 与 `top`（ASCII 大小写
    /// 不敏感且无匹配元素）回到顶部。返回 `true` 表示已按同文档导航处理。
    /// https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate-to-a-fragment
    fn navigate_to_fragment_anchor(&mut self, url: &str) -> bool {
        let Some((target_base, raw_fragment)) = url.split_once('#') else {
            return false;
        };
        if target_base.is_empty() {
            return false;
        }
        let Some(tab_id) = self.shell.active_tab_id() else {
            return false;
        };
        let Some(current_base) = self
            .shell
            .tab(tab_id)
            .and_then(|tab| tab.url())
            .map(|current| current.split('#').next().unwrap_or(current).to_string())
        else {
            return false;
        };
        if current_base != target_base {
            return false;
        }

        // 离开当前条目前保存滚动位置；fragment 导航同样 push 历史（back/forward 在
        // 同文档锚点条目间移动），文档本体保持原状（标题/加载态不变）。
        self.pending_scroll_restore = None;
        self.save_history_scroll(tab_id);
        self.shell.navigate_fragment(url);
        self.address_bar.set_text(url.to_string());
        self.autocomplete.clear();

        // 锚点定位：命中测试快照按元素 `id` 查文档坐标 Y（`<a name>` 传统锚未建模 FIXME）。
        let fragment = Self::percent_decode(raw_fragment);
        let anchor_y = if fragment.is_empty() {
            Some(0.0)
        } else {
            self.tabs
                .anchor_scroll_y(tab_id, &fragment)
                .or_else(|| fragment.eq_ignore_ascii_case("top").then_some(0.0))
        };
        if let Some(y) = anchor_y {
            self.apply_fragment_scroll(tab_id, y);
        }
        self.needs_redraw = true;
        true
    }

    /// fragment 同文档跳转的滚动落地：按当前布局钳制后立即写入——文档已布局
    ///（无装载窗口），不存在 pending 恢复期被逐帧 clamp 压回的问题。
    fn apply_fragment_scroll(&mut self, tab_id: TabId, y: f32) {
        self.scroll_anim.remove(&tab_id);
        let max_scroll_y = self.page_scroll_layout(tab_id).max_scroll_y;
        let state = self.tab_scroll_state(tab_id);
        self.scroll.insert(
            tab_id,
            TabScrollState {
                x: state.x,
                y: y.min(max_scroll_y),
            },
        );
        self.needs_redraw = true;
    }

    /// 历史移动（back/forward）落点与出发为同一文档（URL 除 fragment 外一致，含
    /// 落点无 fragment 的基础条目）时同文档跳转：不重载文档，滚动按目标条目记录
    /// 恢复；无记录则定位到该 fragment 锚点（无锚点/空/无 fragment 回顶部）。
    /// 返回 `true` 表示已处理。
    fn fragment_jump_after_history_move(&mut self, tab_id: TabId, from_url: Option<&str>, to_url: &str) -> bool {
        let Some(from_base) = from_url.map(|u| u.split('#').next().unwrap_or(u)) else {
            return false;
        };
        let to_base = to_url.split('#').next().unwrap_or(to_url);
        if to_base.is_empty() || to_base != from_base {
            return false;
        }
        let raw_fragment = to_url.split_once('#').map(|(_, fragment)| fragment);
        let recorded = self
            .shell
            .tab(tab_id)
            .map(|tab| tab.history_index())
            .and_then(|index| self.history_scroll.get(&(tab_id, index)).copied());
        let y = match recorded {
            Some(y) => y,
            None => raw_fragment
                .map(|fragment| {
                    self.tabs
                        .anchor_scroll_y(tab_id, &Self::percent_decode(fragment))
                        .unwrap_or(0.0)
                })
                .unwrap_or(0.0),
        };
        self.apply_fragment_scroll(tab_id, y);
        true
    }

    fn load_welcome_page(&mut self, tab_id: TabId) {
        // T4-I6：欢迎页此前只进页面运行时，shell 历史无条目——启动/新标签首次导航后
        // 返回箭头永远禁用，按钮态与可见历史差一。此处补记初始导航（仅当该标签
        // 尚无 URL；地址栏路径已先 shell.navigate，不会重复入账）。
        self.shell.record_initial_navigation(tab_id, "zero://newtab");
        self.tabs.ensure_tab(tab_id);
        // 每次装载都按当前历史与搜索引擎重新生成（t12-t14：新标签页展示最近访问
        // 站点；t31-t33：中央搜索框以当前搜索引擎为表单 action）。
        let html = pages::generate_welcome_html(self.shell.history(), self.shell.settings().search_engine);
        self.tabs.load_html(tab_id, &html, None, Some("zero://newtab"));
        self.finish_tab_load(tab_id, "zero://newtab", "ZeroBrowser");
    }

    fn clear_tab_favicon(&mut self, tab_id: TabId) {
        let size = layout::TAB_ICON_SIZE * self.scale_factor;
        crate::tab_favicon::clear_tab_favicon(&mut self.font_loader, tab_id, size);
        self.favicon_fetch.cancel_tab(tab_id);
    }

    fn refresh_tab_favicon(&mut self, tab_id: TabId, page_url: &str) {
        let size = layout::TAB_ICON_SIZE * self.scale_factor;
        crate::tab_favicon::ensure_tab_favicon_placeholder(&mut self.font_loader, tab_id, size);
        let html_owned = self.tabs.page_html(tab_id);
        let html = html_owned.as_deref().or_else(|| Self::tab_html_hint(Some(page_url)));
        self.favicon_fetch.request(tab_id, page_url, html, size);
    }

    /// 为书签栏中尚未缓存 favicon 的书签发起后台抓取。
    fn refresh_bookmark_favicons(&mut self) {
        if !self.bookmarks_bar_visible() {
            return;
        }
        let size = layout::BOOKMARKS_BAR_ICON_SIZE * self.scale_factor;
        let urls: Vec<String> = self
            .shell
            .bookmarks()
            .list_root()
            .iter()
            .map(|bm| bm.url().to_string())
            .collect();
        for url in urls {
            self.favicon_fetch.request_bookmark(&url, size);
        }
    }

    pub fn any_tab_loading(&self) -> bool {
        self.shell.tabs().any(|tab| tab.is_loading()) || self.tabs.any_loading()
    }

    pub(crate) fn show_scrollbar_overlay(&mut self) {
        self.scrollbar_visible_until = Some(Instant::now() + Duration::from_millis(750));
    }

    pub(crate) fn scrollbar_overlay_visible(&self) -> bool {
        self.scrollbar_drag.is_some()
            || self.scrollbar_hover.is_some()
            || self.touch_scroll.is_some()
            || self
                .scrollbar_visible_until
                .is_some_and(|deadline| Instant::now() < deadline)
    }

    pub(crate) fn expire_scrollbar_overlay(&mut self) {
        if self
            .scrollbar_visible_until
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.scrollbar_visible_until = None;
            self.needs_redraw = true;
        }
    }

    /// 返回当前 Tab 后端实际可用的 compositor 状态。
    pub(crate) fn compositor_status(&self) -> crate::compositor_client::CompositorStatus {
        #[cfg(test)]
        if let Some(status) = self.compositor_status_override {
            return status;
        }
        if self.tabs.is_multiprocess() {
            crate::compositor_client::status()
        } else {
            crate::compositor_client::CompositorStatus::Disabled
        }
    }

    /// 返回真实窗口产品 smoke 当前可验收的页面像素来源。
    ///
    /// 只有 renderer 页面帧已经进入 Browser 最终场景后才返回，避免捕获启动空白帧。
    pub fn product_smoke_frame_source(&self) -> Option<&'static str> {
        let tab_id = self.shell.active_tab_id()?;
        let snapshot = self.tabs.snapshot(tab_id)?;
        if snapshot.navigation_epoch == 0 {
            return None;
        }
        match self.compositor_status() {
            crate::compositor_client::CompositorStatus::Healthy if snapshot.compositor_frame.is_some() => {
                Some("compositor_bitmap")
            }
            crate::compositor_client::CompositorStatus::Disabled
            | crate::compositor_client::CompositorStatus::Disconnected
                if snapshot.last_render.is_some() =>
            {
                Some("legacy_view_painted")
            }
            _ => None,
        }
    }

    fn tab_html_hint(page_url: Option<&str>) -> Option<&'static str> {
        match page_url {
            None => Some(pages::WELCOME_HTML),
            Some(url) if url.starts_with("zero://") && url != "zero://settings" => Some(pages::WELCOME_HTML),
            _ => None,
        }
    }

    /// 在窗口物理尺寸已知后创建默认标签页的 WebView（避免以 1024 默认视口渲染）
    pub fn ensure_startup_tab(&mut self) {
        // T20：会话恢复只在首次启动检查（干净退出 → 直接恢复；崩溃残留 → 气泡）。
        if !self.session_startup_handled {
            self.session_startup_handled = true;
            self.restore_session_at_startup();
        }
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        if self.tabs.has_tab(tab_id) {
            return;
        }
        self.init_default_tab();
        // CLI 位置参数作为启动 URL（对齐 Chrome `zero-browser <url>` 行为）：走
        // 地址栏同一 navigate_to 路径，normalize_url 的 scheme 启发式对其同样生效。
        if let Some(url) = self.startup_url.take() {
            self.navigate_from_ui(&url);
        }
    }

    /// T20：启动时会话恢复检查。
    ///
    /// 会话文件存在且 `clean_exit=true`（上次干净退出）→ 直接恢复全部标签；
    /// `clean_exit=false`（上次异常退出）→ 保留快照并显示「恢复标签？」气泡
    /// （Chrome "Restore pages?" 同款），由用户决定是否恢复。检查完成后立即写回
    /// `clean_exit=false` 运行中标记：本次运行若再崩溃，下次启动仍能检测到。
    fn restore_session_at_startup(&mut self) {
        let path = SessionState::default_session_path();
        let Some(session) = SessionState::load(&path) else {
            return;
        };
        if !session.is_empty() {
            if session.clean_exit {
                self.apply_restored_session(&session);
            } else {
                tracing::info!("previous session did not exit cleanly, offering restore");
                self.pending_session_restore = Some(session);
                self.session_restore_prompt_visible = true;
                self.needs_redraw = true;
            }
        }
        self.persist_running_session();
    }

    /// 将当前会话以运行中标记（`clean_exit=false`）落到默认路径。
    ///
    /// 启动检查、崩溃计数递增与观察窗清零共用：崩溃随时可能到来，计数变更
    /// 必须即时持久化，不能等退出路径。
    fn persist_running_session(&self) {
        let path = SessionState::default_session_path();
        let mut running = self.shell.save_session();
        running.clean_exit = false;
        if let Err(err) = running.save(&path) {
            tracing::warn!(%err, "failed to write session running marker");
        }
    }

    /// FR-6：为 tab 安排崩溃计数观察窗（到期清零；再次崩溃则重排）。
    fn schedule_crash_reset(&mut self, tab_id: TabId) {
        let deadline = Instant::now() + Duration::from_secs(zero_browser_shell::CRASH_LOOP_OBSERVATION_WINDOW_SECS);
        if let Some(entry) = self.crash_reset_deadlines.iter_mut().find(|(id, _)| *id == tab_id) {
            entry.1 = deadline;
        } else {
            self.crash_reset_deadlines.push((tab_id, deadline));
        }
    }

    /// FR-6：观察窗到期——tab 存活且无新崩溃，崩溃计数清零并落盘。
    fn sweep_crash_reset_deadlines(&mut self) {
        if self.crash_reset_deadlines.is_empty() {
            return;
        }
        let now = Instant::now();
        let expired: Vec<TabId> = self
            .crash_reset_deadlines
            .iter()
            .filter(|(_, deadline)| *deadline <= now)
            .map(|(tab_id, _)| *tab_id)
            .collect();
        if expired.is_empty() {
            return;
        }
        self.crash_reset_deadlines.retain(|(_, deadline)| *deadline > now);
        let mut changed = false;
        for tab_id in expired {
            if self.shell.reset_tab_crash_count(tab_id) {
                tracing::info!(tab_id = tab_id.0, "tab crash count reset after observation window");
                changed = true;
            }
        }
        if changed {
            self.persist_running_session();
        }
    }

    /// 将会话快照落到 shell 标签并逐标签装载（T20）。
    ///
    /// `restore_session` 只重建 shell 标签（URL/标题/历史，替换现有标签），
    /// 渲染端 runtime 与页面加载由这里补齐：http(s) 走 `tabs.navigate`；
    /// 活跃标签的 `zero://` 内部页按 URL 分发；非活跃标签的内部页快照按欢迎页
    /// 装载（内部页生成器只作用于活跃标签；FIXME：内部页快照恢复后续切片），
    /// 此时绕过 `finish_tab_load`——`on_page_loaded` 只改活跃标签状态，
    /// 快照里已带标题。
    fn apply_restored_session(&mut self, session: &SessionState) -> usize {
        let count = self.shell.restore_session(session);
        let active_id = self.shell.active_tab_id();
        let restored: Vec<(TabId, Option<String>)> = self
            .shell
            .tabs()
            .map(|tab| (tab.id(), tab.url().map(str::to_string)))
            .collect();
        for (tab_id, url) in restored {
            self.tabs.ensure_tab(tab_id);
            self.scroll.insert(tab_id, TabScrollState::default());
            // FR-6 崩溃循环守卫：恢复的 tab 带非零崩溃计数即进入观察窗，存活
            // 一个窗口即清零（跳过加载的 tab 同样适用，跳过只此一次）。
            if self.shell.tab_crash_count(tab_id) > 0 {
                self.schedule_crash_reset(tab_id);
            }
            let Some(url) = url else { continue };
            // FR-6：达到崩溃循环阈值的 tab 不再自动加载崩溃页，置空 about:blank。
            if self.shell.tab_crash_count(tab_id) >= zero_browser_shell::CRASH_LOOP_SKIPPED_THRESHOLD {
                tracing::info!(tab_id = tab_id.0, %url, "skipping auto-load of crash-loop tab");
                self.tabs.load_html(tab_id, "", None, Some("about:blank"));
                continue;
            }
            // T24：仅活动标签恢复按站点缩放（set_zoom 作用于活动标签）。
            if Some(tab_id) == active_id {
                self.restore_zoom_for_url(&url);
            }
            if url.starts_with("http://") || url.starts_with("https://") {
                self.tabs.navigate(tab_id, url);
            } else if Some(tab_id) == active_id && url.starts_with("zero://") {
                self.load_local_tab_url(tab_id, &url);
            } else {
                let html = pages::generate_welcome_html(self.shell.history(), self.shell.settings().search_engine);
                self.tabs.load_html(tab_id, &html, None, Some("zero://newtab"));
            }
        }
        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        self.sync_webview_viewport();
        self.needs_redraw = true;
        count
    }

    /// T20：恢复崩溃前会话（气泡「恢复标签」按钮）。替换当前标签。
    fn restore_pending_session(&mut self) {
        if let Some(session) = self.pending_session_restore.take() {
            self.apply_restored_session(&session);
        }
        self.session_restore_prompt_visible = false;
        self.needs_redraw = true;
    }

    /// T20：关闭崩溃恢复气泡（不恢复）。
    pub fn dismiss_session_restore(&mut self) {
        self.pending_session_restore = None;
        self.session_restore_prompt_visible = false;
        self.needs_redraw = true;
    }

    /// FR-6 测试钩子：注入崩溃恢复候选会话并执行恢复（等价气泡「恢复标签」）。
    #[cfg(test)]
    pub(crate) fn restore_session_for_test(&mut self, session: SessionState) {
        self.pending_session_restore = Some(session);
        self.restore_pending_session();
    }

    /// T20：崩溃恢复气泡矩形（内容区左上角浮层；渲染与命中测试同源）。
    pub(crate) fn session_restore_bubble_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (content_x, content_y, _content_w, _content_h) = self.page_content_rect_for(width, height);
        (content_x + 12.0 * s, content_y + 12.0 * s, 380.0 * s, 118.0 * s)
    }

    /// T20：气泡按钮矩形 `([恢复标签], [关闭])`，与渲染同源。
    pub(crate) fn session_restore_button_rects(bubble: ScreenRect, s: f32) -> (ScreenRect, ScreenRect) {
        let (bx, by, _bw, bh) = bubble;
        let btn_w = 112.0 * s;
        let btn_h = 28.0 * s;
        let pad = 12.0 * s;
        let y = by + bh - btn_h - pad;
        (
            (bx + pad, y, btn_w, btn_h),
            (bx + pad + btn_w + 10.0 * s, y, btn_w, btn_h),
        )
    }

    /// 设置 CLI 位置参数给出的启动 URL（窗口就绪后由 `ensure_startup_tab` 消费）。
    pub fn set_startup_url(&mut self, url: Option<String>) {
        self.startup_url = url;
    }

    /// 初始化 Shell 默认标签页（仅创建 WebView，不额外开 tab）
    fn init_default_tab(&mut self) {
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        self.ensure_webview(tab_id);
        if self.shell.active_tab().and_then(|t| t.url()).is_none() {
            self.load_welcome_page(tab_id);
        }
        self.scroll.insert(tab_id, TabScrollState::default());
        self.needs_redraw = true;
    }

    /// 创建新标签页
    pub fn new_tab(&mut self, url: Option<&str>) {
        let tab_id = self.shell.new_tab(url);
        self.tabs.ensure_tab(tab_id);

        if let Some(url) = url {
            self.address_bar.set_text(url.to_string());
            // `Tab::new` 只把 URL 记进 shell 模型，不向 renderer 派发导航命令，
            // 标签会停在"Loading…"。走地址栏同款标签级装载路径（normalize 已在
            // 入口完成，避免 shell.navigate 重复入账历史）。
            self.start_tab_request(tab_id, url.to_string(), "GET".to_string(), None);
        } else {
            self.address_bar.clear();
            self.load_welcome_page(tab_id);
        }

        self.scroll.insert(tab_id, TabScrollState::default());
        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        self.sync_webview_viewport();
        self.needs_redraw = true;
    }

    /// 后台打开新标签页（不切换活跃、不聚焦地址栏）。
    /// 用于 Ctrl+点击链接、中键点击链接等"在新标签打开但不离开当前页"场景。
    pub fn new_tab_background(&mut self, url: &str) {
        let tab_id = self.shell.new_tab_background(Some(url));
        self.tabs.ensure_tab(tab_id);
        // 同 [`Self::new_tab`]：shell 记 URL 不等于 renderer 开始装载，须显式派发。
        self.start_tab_request(tab_id, url.to_string(), "GET".to_string(), None);
        self.scroll.insert(tab_id, TabScrollState::default());
        self.needs_redraw = true;
    }

    /// 创建无痕标签页（不写磁盘 HTTP 缓存、不保存到会话）。
    pub fn new_private_tab(&mut self, url: Option<&str>) {
        let tab_id = self.shell.new_private_tab(url);
        self.tabs.ensure_tab(tab_id);
        self.tabs.set_tab_private(tab_id, true);

        if let Some(url) = url {
            self.address_bar.set_text(url.to_string());
        } else {
            self.address_bar.clear();
            self.load_welcome_page(tab_id);
        }

        self.scroll.insert(tab_id, TabScrollState::default());
        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        self.sync_webview_viewport();
        self.needs_redraw = true;
    }

    /// 关闭活跃标签页
    pub fn close_active_tab(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.tabs.remove_tab(tab_id);
            self.scroll.remove(&tab_id);
            self.scroll_anim.remove(&tab_id);
            self.shell.close_tab(tab_id);
            self.on_tab_closed_show_undo_toast(tab_id);
            // Chrome 行为：关闭标签后焦点落到相邻标签的页面内容，地址栏失焦——
            // 否则编辑中的地址栏文本残留，键入继续进地址栏而非新活跃标签。
            self.address_bar_focused = false;
            self.address_bar_ime_preedit.clear();
            self.autocomplete.clear();

            if self.shell.is_empty() {
                self.new_tab(None);
            } else {
                self.tabs.on_active_tab_changed(self.shell.active_tab_id());
            }

            self.update_address_bar_from_active_tab();
            self.needs_redraw = true;
        }
    }

    /// 关闭指定 ID 的标签页
    fn close_tab_by_id(&mut self, id: TabId) {
        self.tabs.remove_tab(id);
        self.scroll.remove(&id);
        self.scroll_anim.remove(&id);
        self.background_tab_titles.remove(&id);
        self.shell.close_tab(id);
        self.on_tab_closed_show_undo_toast(id);
        // 与 close_active_tab 同款地址栏失焦（Chrome 行为，见该处注释）。
        self.address_bar_focused = false;
        self.address_bar_ime_preedit.clear();
        self.autocomplete.clear();

        if self.shell.is_empty() {
            self.new_tab(None);
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 关闭标签后的收尾：启动撤销 toast、清理该标签的按历史条目滚动缓存。
    /// 多标签批量关闭（关闭其他/右侧）不弹 toast——撤销一次只恢复一个，
    /// 提示会误导（栈里已入账，Ctrl+Shift+T 仍可逐个恢复）。
    fn on_tab_closed_show_undo_toast(&mut self, id: TabId) {
        self.history_scroll.retain(|(tab_id, _), _| *tab_id != id);
        if matches!(self.pending_scroll_restore, Some((closed, _)) if closed == id) {
            self.pending_scroll_restore = None;
        }
        self.closed_tab_toast = Some(Instant::now());
    }

    /// 恢复最近关闭的标签（Ctrl+Shift+T / 撤销 toast 共用）。返回是否恢复成功。
    ///
    /// shell 侧建标签并入账导航；app 侧须补齐 renderer 装载与 UI 同步——
    /// 与 [`Self::new_tab`] 同理：shell 记 URL 不等于 renderer 开始装载。
    pub fn reopen_last_closed_tab(&mut self) -> bool {
        let Some(new_id) = self.shell.reopen_last_closed_tab() else {
            return false;
        };
        self.finish_reopen_closed_tab(new_id);
        true
    }

    /// 恢复菜单「最近关闭的标签」点选的指定 URL 标签。返回是否恢复成功。
    pub fn reopen_closed_tab_by_url(&mut self, url: &str) -> bool {
        let Some(new_id) = self.shell.reopen_closed_by_url(url) else {
            return false;
        };
        self.finish_reopen_closed_tab(new_id);
        true
    }

    /// 补齐恢复标签的 app 侧装载与 UI 同步（两类恢复入口的共享路径）。
    fn finish_reopen_closed_tab(&mut self, new_id: TabId) {
        self.closed_tab_toast = None;
        self.tabs.ensure_tab(new_id);
        self.scroll.insert(new_id, TabScrollState::default());
        if let Some(url) = self.shell.tab(new_id).and_then(|t| t.url().map(str::to_string)) {
            // 地址栏同款标签级装载路径（zero:// 内部页由 load_local_tab_url 分派）。
            self.start_tab_request(new_id, url, "GET".to_string(), None);
        } else {
            self.load_welcome_page(new_id);
        }
        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        self.sync_webview_viewport();
        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 复制指定标签页，副本插入其后并设为活跃。
    fn duplicate_tab_by_id(&mut self, id: TabId) {
        if let Some(new_id) = self.shell.duplicate_tab(id) {
            self.tabs.ensure_tab(new_id);
            // 同 [`Self::new_tab`] / [`Self::new_tab_background`]：`Tab::duplicate` 只把
            // URL 记进 shell 模型，不向 renderer 派发导航命令，副本会停在"Loading…"。
            // 按当前 URL 走地址栏同款标签级装载路径（内部页由 load_local_tab_url 分派）。
            if let Some(url) = self.shell.tab(new_id).and_then(|t| t.url().map(str::to_string)) {
                self.start_tab_request(new_id, url, "GET".to_string(), None);
            }
            self.scroll.insert(new_id, TabScrollState::default());
            self.tabs.on_active_tab_changed(self.shell.active_tab_id());
            self.sync_webview_viewport();
            self.update_address_bar_from_active_tab();
            self.needs_redraw = true;
        }
    }

    /// 关闭除指定标签页外的所有标签页。
    fn close_other_tabs_by_id(&mut self, id: TabId) {
        let to_close: Vec<TabId> = self.shell.tabs().map(|t| t.id()).filter(|&tid| tid != id).collect();
        for tid in to_close {
            self.tabs.remove_tab(tid);
            self.scroll.remove(&tid);
            self.scroll_anim.remove(&tid);
        }
        self.shell.close_other_tabs(id);
        self.history_scroll
            .retain(|(tab_id, _), _| self.shell.tab(*tab_id).is_some());
        if matches!(self.pending_scroll_restore, Some((closed, _)) if self.shell.tab(closed).is_none()) {
            self.pending_scroll_restore = None;
        }

        if self.shell.is_empty() {
            self.new_tab(None);
        } else {
            self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 关闭指定标签页右侧的所有标签页。
    fn close_tabs_to_right_by_id(&mut self, id: TabId) {
        let to_close: Vec<TabId> = {
            let mut found = false;
            self.shell
                .tabs()
                .filter(|t| {
                    if t.id() == id {
                        found = true;
                        false
                    } else {
                        found
                    }
                })
                .map(|t| t.id())
                .collect()
        };
        for tid in to_close {
            self.tabs.remove_tab(tid);
            self.scroll.remove(&tid);
            self.scroll_anim.remove(&tid);
        }
        self.shell.close_tabs_to_right(id);
        self.history_scroll
            .retain(|(tab_id, _), _| self.shell.tab(*tab_id).is_some());
        if matches!(self.pending_scroll_restore, Some((closed, _)) if self.shell.tab(closed).is_none()) {
            self.pending_scroll_restore = None;
        }

        if self.shell.is_empty() {
            self.new_tab(None);
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 刷新当前页面
    pub fn refresh_page(&mut self) {
        self.shell.refresh();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.start_tab_load(tab_id, url);
    }

    /// 切换打印预览（Ctrl+P）：Screen ↔ Print，重渲染使 `@media print` 规则即时生效/失效
    ///（DC-12 打印预览；R1993）。minimal preview——显示打印媒体样式（@page/page-break
    /// 真 print-layout 分页为后续 feature）。
    pub fn toggle_print_preview(&mut self) {
        let next = match self.tabs.media_type() {
            zero_engine::MediaType::Print => zero_engine::MediaType::Screen,
            _ => zero_engine::MediaType::Print,
        };
        self.tabs.set_media_type(next);
        // T61：预览态按当前设置下发打印页面设置（方向/边距）。Screen 态下发无害——
        // Print 页几何仅 Print 媒体消费。
        self.sync_print_page_setup_from_settings();
        self.needs_redraw = true;
    }

    /// T61：把设置中的打印页面设置换算成 [`zero_engine::PrintPageSetup`] 下发所有 Tab。
    /// 方向/边距参与 Print 布局；份数仅打印作业语义（Chrome 同），不经渲染链。
    fn sync_print_page_setup_from_settings(&mut self) {
        let settings = self.shell.settings();
        let orientation = match settings.print_orientation {
            PrintOrientationPreference::Portrait => zero_engine::PrintOrientation::Portrait,
            PrintOrientationPreference::Landscape => zero_engine::PrintOrientation::Landscape,
        };
        self.tabs.set_print_page_setup(Some(zero_engine::PrintPageSetup {
            orientation,
            margin_mm: settings.print_margin_mm,
        }));
    }

    /// T61：切换打印页面方向（预览面板方向行点击；纵向 ↔ 横向）。持久化并即时下发。
    pub fn cycle_print_orientation(&mut self) {
        self.shell.apply_settings(|settings| {
            settings.print_orientation = match settings.print_orientation {
                PrintOrientationPreference::Portrait => PrintOrientationPreference::Landscape,
                PrintOrientationPreference::Landscape => PrintOrientationPreference::Portrait,
            };
        });
        self.sync_print_page_setup_from_settings();
        self.needs_redraw = true;
    }

    /// T61：按步进调整打印份数（预览面板 −/+；clamp 1–99），持久化。
    /// 份数不影响预览渲染（Chrome print dialog 同为作业参数）。
    pub fn adjust_print_copies(&mut self, delta: i32) {
        self.shell.apply_settings(|settings| {
            let next = i64::from(settings.print_copies) + i64::from(delta);
            settings.print_copies = next.clamp(
                i64::from(BrowserSettings::PRINT_COPIES_MIN),
                i64::from(BrowserSettings::PRINT_COPIES_MAX),
            ) as u32;
        });
        self.needs_redraw = true;
    }

    /// T61：按步进调整打印页边距（预览面板 −/+；clamp 0–50mm），持久化并即时下发。
    /// `@page { margin }` 显式声明时页面值优先（引擎 `extract_print_page_geometry` 裁决）。
    pub fn adjust_print_margin(&mut self, delta_mm: f32) {
        self.shell.apply_settings(|settings| {
            settings.print_margin_mm = (settings.print_margin_mm + delta_mm).clamp(
                BrowserSettings::PRINT_MARGIN_MIN_MM,
                BrowserSettings::PRINT_MARGIN_MAX_MM,
            );
        });
        self.sync_print_page_setup_from_settings();
        self.needs_redraw = true;
    }

    /// T61：打印预览设置面板行命中测试（渲染层写入的物理像素矩形）。
    /// 返回 `(行索引, 动作)`：行 0 = `Some(false)`（方向切换）；行 1/2 命中区按
    /// 渲染实测的 −/+ 字形起点划定（D4：行宽二分会把可见 + 判进 − 区），字形区
    /// 之外（标签/数值）为 `Some(row, None)`——点击被面板消费但不触发动作
    /// （面板悬浮于页面区之上，穿透会误点下层页面内容）；面板外返回 None。
    fn print_setup_panel_hit(&self, x: f32, y: f32) -> Option<(usize, Option<bool>)> {
        for (row, &(rx, ry, rw, rh)) in self.print_setup_panel_rows.iter().enumerate() {
            if rw <= 0.0 {
                continue;
            }
            if x < rx || x >= rx + rw || y < ry || y >= ry + rh {
                continue;
            }
            if row == 0 {
                return Some((row, Some(false)));
            }
            let (minus_x, plus_x) = self.print_setup_panel_steppers[row];
            if plus_x <= minus_x {
                // 渲染层未写入（陈旧帧）：退化为行宽二分，保持可用。
                return Some((row, Some(x >= rx + rw / 2.0)));
            }
            const GLYPH_TOUCH_PAD: f32 = 6.0;
            let plus = x >= plus_x - GLYPH_TOUCH_PAD;
            let minus = x >= minus_x - GLYPH_TOUCH_PAD;
            return Some((
                row,
                match (minus, plus) {
                    (_, true) => Some(true),
                    (true, false) => Some(false),
                    (false, false) => None,
                },
            ));
        }
        None
    }

    /// T61：当前打印设置 `(方向, 份数, 边距 mm)`（ui_driver 断言与诊断读回）。
    pub fn print_settings(&self) -> (PrintOrientationPreference, u32, f32) {
        let s = self.shell.settings();
        (s.print_orientation, s.print_copies, s.print_margin_mm)
    }

    /// 停止当前页加载（Esc / loading 时点击刷新按钮）。
    pub fn stop_loading_page(&mut self) {
        self.shell.stop_loading();
        self.needs_redraw = true;
    }

    /// 重新加载所有标签（菜单项「重新加载所有标签」）。
    /// 收集所有标签的 (id, url) 快照，再逐个 start_tab_load，避免借用冲突。
    pub fn reload_all_tabs(&mut self) {
        let targets: Vec<(TabId, String)> = self
            .shell
            .tabs()
            .filter_map(|t| t.url().map(|u| (t.id(), u.to_string())))
            .collect();
        for (tab_id, url) in targets {
            self.start_tab_load(tab_id, url);
        }
    }

    /// 强制刷新当前页（Ctrl+F5 / Ctrl+Shift+R）：清除该 URL 缓存后重新加载。
    pub fn refresh_page_bypass_cache(&mut self) {
        self.shell.refresh();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        if url.starts_with("http://") || url.starts_with("https://") {
            if self.shell.active_tab_id() == Some(tab_id)
                && let Some(tab) = self.shell.active_tab_mut()
            {
                tab.set_loading(true);
            }
            self.ensure_webview(tab_id);
            self.sync_webview_viewport();
            tracing::info!("Tab {} hard reload (bypass cache): {url}", tab_id.0);
            self.tabs.navigate_bypass_cache(tab_id, url);
            self.needs_redraw = true;
        } else {
            // 非 http(s) 页面（本地、zero://）无 HTTP 缓存，走普通刷新。
            self.start_tab_load(tab_id, url);
        }
    }

    /// 在新标签页打开内部 HTML 文档（查看源代码、检查元素等）。
    pub fn open_internal_document_tab(&mut self, html: String, url: &str, title: &str) {
        let tab_id = self.shell.new_tab(Some(url));
        self.tabs.ensure_tab(tab_id);
        self.tabs.load_html(tab_id, &html, None, Some(url));
        if let Some(tab) = self.shell.active_tab_mut() {
            tab.set_title(title);
        }
        self.address_bar.set_text(url.to_string());
        self.scroll.insert(tab_id, TabScrollState::default());
        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
        self.sync_webview_viewport();
        self.needs_redraw = true;
    }

    /// 地址栏 `view-source:<url>` 入口：后台线程抓取目标内容，主循环 poll 到结果后
    /// 以内页展示源码（参照 favicon_fetch 异步模式；抓取线程自带 30s 超时，事件
    /// 循环不再被慢/黑洞目标阻塞）。右键菜单「查看源代码」经同一入口。
    ///
    /// 内层 URL 与地址栏普通输入走同一 normalize 路径，因此
    /// `view-source:127.0.0.1:8080/x` 等无 scheme 简写同样生效。
    fn open_view_source_page(&mut self, inner: &str) {
        let inner = inner.trim();
        if inner.is_empty() {
            // 空内层（含仅空白）明确出错误内页而非静默无动作（对齐 Chrome 空内层错误视图）
            let page = pages::generate_error_page("view-source:", "view-source 缺少内层 URL");
            self.open_internal_document_tab(page, "view-source:", "查看源代码");
            return;
        }
        let target = normalize_url(inner, &self.shell);
        self.view_source_fetch.request(target);
    }

    /// 审查右键点击位置的元素。
    pub fn inspect_element_at(&mut self, tab_id: TabId, doc_x: f32, doc_y: f32) {
        let hit = self.tabs.hit_test_element(tab_id, doc_x, doc_y);
        let source_url = self.tabs.page_url(tab_id).unwrap_or_else(|| "about:blank".to_string());
        let page = pages::generate_inspect_element_page(&source_url, doc_x, doc_y, hit.as_ref());
        let inspect_url = format!("zero://inspect?url={source_url}");
        self.open_internal_document_tab(page, &inspect_url, "检查元素");
    }

    /// 执行后退导航
    /// 触发缩放百分比浮层显示（3 秒内有效），并请求重绘。
    fn show_zoom_indicator(&mut self) {
        self.zoom_indicator_start = Some(Instant::now());
        self.needs_redraw = true;
    }

    /// 计算当前应显示的窗口标题。
    /// 有页面标题时为 "<title> - ZeroBrowser"，否则为 "ZeroBrowser"。
    pub fn current_window_title(&self) -> String {
        match self.shell.active_tab().and_then(|t| t.title()) {
            Some(t) if !t.is_empty() => format!("{t} - ZeroBrowser"),
            _ => "ZeroBrowser".to_string(),
        }
    }

    /// 若窗口标题相比缓存发生变化，返回新标题；否则返回 None。
    /// 调用方在成功 set_title 后应调用 confirm_window_title 同步缓存。
    pub fn window_title_if_changed(&mut self) -> Option<String> {
        let new = self.current_window_title();
        if new == self.last_window_title { None } else { Some(new) }
    }

    /// 确认窗口标题已应用，更新内部缓存。
    pub fn confirm_window_title(&mut self, title: &str) {
        self.last_window_title = title.to_string();
    }

    /// 新建空白标签页并聚焦地址栏（用户主动开新标签场景：Ctrl+T / 点 + 按钮）。
    /// 聚焦后全选地址栏内容，便于用户直接输入覆盖。
    fn new_blank_tab_focused(&mut self, private: bool) {
        if private {
            self.new_private_tab(None);
        } else {
            self.new_tab(None);
        }
        self.address_bar_focused = true;
        // T37：聚焦重置光标闪烁相位，光标立即可见。
        self.address_bar_cursor_phase = Instant::now();
        self.address_bar.select_all();
        self.needs_redraw = true;
    }

    pub fn go_back(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.save_history_scroll(tab_id);
        }
        // 出发 URL 先记录：落点为同一文档（除 fragment 外一致）时走同文档跳转（不重载）。
        let from_url = self.shell.active_tab().and_then(|tab| tab.url().map(str::to_string));
        if !self.shell.go_back() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar.set_text(url.clone());
        let tab_id = self.shell.active_tab_id().unwrap();
        if self.fragment_jump_after_history_move(tab_id, from_url.as_deref(), &url) {
            return;
        }
        self.restore_history_scroll(tab_id);

        self.start_tab_load(tab_id, url);
    }

    /// T5/P10：页面 history 遍历提交（shim 本地栈越界移交 → 宿主 shell 栈）。
    ///
    /// delta ∓1 复用 go_back/go_forward 既有路径（shell 栈移动、地址栏、scroll 保存
    /// 恢复、start_tab_load）；其余步长经 jump_history(index+delta) 落同一流程，越界
    /// （index+delta 不可达）保持静默 no-op（R3004）。
    /// https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-history-interface
    pub fn history_traverse(&mut self, delta: i32) {
        match delta {
            -1 => return self.go_back(),
            1 => return self.go_forward(),
            0 => return,
            _ => {}
        }
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        // 越界判界先于一切副作用（R3004 静默 no-op = 零状态变更）。
        let (index, len) = {
            let tab = self.shell.active_tab().unwrap();
            (tab.history_index(), tab.history_len())
        };
        let target = index as i64 + delta as i64;
        if target < 0 || target >= len as i64 {
            return; // 越界：宿主判界静默 no-op（R3004）
        }
        self.save_history_scroll(tab_id);
        // 出发 URL 先记录：落点为同一文档（除 fragment 外一致）时走同文档跳转（不重载）。
        let from_url = self.shell.active_tab().and_then(|tab| tab.url().map(str::to_string));
        if !self.shell.jump_history(target as usize) {
            return;
        }
        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };
        self.address_bar.set_text(url.clone());
        if self.fragment_jump_after_history_move(tab_id, from_url.as_deref(), &url) {
            return;
        }
        self.restore_history_scroll(tab_id);
        self.start_tab_load(tab_id, url);
    }

    /// 执行前进导航
    pub fn go_forward(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.save_history_scroll(tab_id);
        }
        // 出发 URL 先记录：落点为同一文档（除 fragment 外一致）时走同文档跳转（不重载）。
        let from_url = self.shell.active_tab().and_then(|tab| tab.url().map(str::to_string));
        if !self.shell.go_forward() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar.set_text(url.clone());
        let tab_id = self.shell.active_tab_id().unwrap();
        if self.fragment_jump_after_history_move(tab_id, from_url.as_deref(), &url) {
            return;
        }
        self.restore_history_scroll(tab_id);

        self.start_tab_load(tab_id, url);
    }

    /// 离开历史条目前保存当前滚动偏移（按 `(标签 id, 历史索引)` 记账）。
    fn save_history_scroll(&mut self, tab_id: TabId) {
        let Some(index) = self.shell.tab(tab_id).map(|t| t.history_index()) else {
            return;
        };
        if let Some(state) = self.scroll.get(&tab_id) {
            self.history_scroll.insert((tab_id, index), state.y);
        }
    }

    /// 回到历史条目（back/forward）时恢复该条目的滚动偏移；
    /// 无记录（首次访问）则从顶部开始。有记录时挂起待页面装载完成后落地——
    /// 立即写入会在新文档布局就绪前被每帧的 clamp_tab_scroll 压回 0。
    fn restore_history_scroll(&mut self, tab_id: TabId) {
        let y = self
            .shell
            .tab(tab_id)
            .map(|t| t.history_index())
            .and_then(|index| self.history_scroll.get(&(tab_id, index)).copied());
        self.scroll_anim.remove(&tab_id);
        match y {
            Some(y) if y > 0.0 => {
                self.pending_scroll_restore = Some((tab_id, y));
                self.scroll.insert(tab_id, TabScrollState::default());
            }
            _ => {
                self.pending_scroll_restore = None;
                self.scroll.insert(tab_id, TabScrollState::default());
            }
        }
        self.needs_redraw = true;
    }

    /// 落地待应用的滚动恢复（页面装载完成时调用）：按当前文档布局钳制后写入。
    fn apply_pending_scroll_restore(&mut self, tab_id: TabId) {
        let Some((pending_tab, y)) = self.pending_scroll_restore else {
            return;
        };
        if pending_tab != tab_id {
            return;
        }
        self.pending_scroll_restore = None;
        let max_scroll_y = self.page_scroll_layout(tab_id).max_scroll_y;
        let state = self.tab_scroll_state(tab_id);
        self.scroll.insert(
            tab_id,
            TabScrollState {
                x: state.x,
                y: y.min(max_scroll_y),
            },
        );
        self.needs_redraw = true;
    }

    /// 单元测试面：back/forward 后的待应用滚动恢复（None = 无挂起恢复）。
    #[cfg(test)]
    pub fn pending_scroll_restore_for_test(&self) -> Option<(TabId, f32)> {
        self.pending_scroll_restore
    }

    /// 单元测试面：手动落地待应用恢复（stub worker 无 page_loaded 事件，
    /// 真实事件驱动路径由 ui_driver e2e 覆盖）。
    #[cfg(test)]
    pub fn apply_pending_scroll_restore_for_test(&mut self, tab_id: TabId) {
        self.apply_pending_scroll_restore(tab_id);
    }

    /// 打开设置页面（about:preferences）
    pub fn open_settings_page(&mut self) {
        self.open_internal_list_page(
            "zero://settings",
            pages::generate_settings_html(self.shell.settings()),
            "设置",
        );
    }

    /// 打开键盘快捷键帮助页（`zero://shortcuts`，Ctrl+/ 快捷键）。
    pub fn open_shortcuts_page(&mut self) {
        self.open_internal_list_page("zero://shortcuts", pages::generate_shortcuts_html(), "快捷键");
    }

    /// 打开崩溃报告与反馈页（`zero://feedback`，FR-5 GUI 导出入口）。
    ///
    /// 报告目录不存在（从未崩溃）时 `list_reports` 返回空，页面走空态。
    pub fn open_feedback_page(&mut self) {
        let reports = zero_browser_shell::list_reports(&zero_browser_shell::crash_reports_dir());
        self.open_internal_list_page("zero://feedback", pages::generate_feedback_html(&reports), "崩溃报告");
    }

    /// 打开历史记录页面。
    pub fn open_history_page(&mut self) {
        self.open_history_page_filtered(None);
    }

    /// 打开带过滤词的历史记录页面（地址栏 URL 为 `zero://history?q=…`）。
    fn open_history_page_filtered(&mut self, query: Option<&str>) {
        let html = pages::generate_history_html(self.shell.history(), query);
        let url = match query.map(str::trim).filter(|q| !q.is_empty()) {
            Some(q) => format!("zero://history?q={}", pages::percent_encode_query_value(q)),
            None => "zero://history".to_string(),
        };
        self.open_internal_list_page(&url, html, "History");
    }

    /// 打开下载管理页面。
    /// 下载落盘 + 下载管理器记账（backend 拦截 Content-Disposition: attachment 或
    /// 顶层文档导航的不可内联 Content-Type 后经 TabManager 转交）。整包到达 →
    /// 进度两跳（0/N → N/N），状态链 Pending → Downloading → Completed。
    /// 附件拦截 / 图片另存为 / 页面另存为共用此路径（app_image_save.rs 等）。
    pub(crate) fn save_attachment_download(&mut self, download: crate::process_backend::PendingAttachmentDownload) {
        let total = download.body.len() as u64;
        // D1：同名文件不静默覆盖——落盘前在扩展名前追加 " (n)"（Chrome 惯例），
        // 下载管理器与完成提示都记实际落盘名。
        let filename =
            crate::process_backend::dedupe_download_filename(&self.download_target_dir(), &download.filename);
        let id = self.shell.downloads_mut().start_download(&download.url, &filename);
        self.shell.downloads_mut().update_progress(id, 0, Some(total));
        let path = self.download_target_dir().join(&filename);
        let write_result = std::fs::write(&path, &download.body);
        match write_result {
            Ok(()) => {
                self.shell.downloads_mut().mark_completed(id);
                // 下载完成 toast（3 秒自动消失；渲染层清除）。
                self.download_toast = Some((filename.clone(), Instant::now()));
                tracing::info!(
                    "SMOKE_EVENT component=browser event=download_completed tab={} file={} bytes={} path={}",
                    download.tab_id.0,
                    filename,
                    total,
                    path.display()
                );
            }
            Err(ref err) => {
                self.shell.downloads_mut().mark_failed(id);
                tracing::warn!("download write failed file={} err={err}", path.display());
            }
        }
        // Chrome 语义：下载开始即弹出下载气泡。
        self.download_panel_open = true;
        self.needs_redraw = true;
    }

    /// 下载目标目录：设置优先，空值回退系统下载目录（不存在则创建）。
    pub(crate) fn download_target_dir(&self) -> std::path::PathBuf {
        let configured = self.shell.settings().download_directory.clone();
        let dir = if configured.is_empty() {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join("Downloads")
        } else {
            std::path::PathBuf::from(configured)
        };
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// 重新计算页面内查找匹配（查找栏键入/切换选项/页面加载后调用）。
    /// 匹配数写回 find_state（计数 UI 数据源），glyph 区间缓存供高亮与滚动定位。
    pub(crate) fn refresh_find_matches(&mut self) {
        if !self.shell.find_state().is_active() {
            self.find_match_ranges.clear();
            self.find_matches_query.clear();
            return;
        }
        let query = self.find_input.clone();
        if query.is_empty() {
            self.find_match_ranges.clear();
            self.find_matches_query.clear();
            self.shell.find_set_matches(0);
            self.needs_redraw = true;
            return;
        }
        let case_sensitive = self.shell.find_state().case_sensitive();
        let whole_word = self.shell.find_state().whole_word();
        let Some(tab_id) = self.shell.active_tab_id() else {
            self.shell.find_set_matches(0);
            return;
        };
        let Some(glyphs) = self.page_glyphs(tab_id) else {
            self.shell.find_set_matches(0);
            return;
        };
        let matches = crate::page_find::find_matches(&glyphs, &query, case_sensitive, whole_word);
        self.shell.find_set_matches(matches.len());
        self.find_matches_query = query;
        self.find_match_ranges = matches.into_iter().map(|m| (m.glyph_range, m.rect)).collect();
        self.needs_redraw = true;
    }

    /// 滚动当前匹配进入视口（find_next/previous 后调用）。
    pub(crate) fn scroll_to_current_match(&mut self) {
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        let current = self.shell.find_state().current_match();
        if current == 0 {
            return;
        }
        let Some((_, (x, top, w, h))) = self.find_match_ranges.get(current - 1) else {
            return;
        };
        let (_x, top, _w, h) = (*x, *top, *w, *h);
        let (width, height) = self.physical_size;
        let layout = self.page_scroll_layout_for(tab_id, width, height);
        let mut state = self.tab_scroll_state(tab_id);
        // 匹配在视口上/下方时才滚动：滚到视口 25%/75% 参考线，尽量保留上下文。
        let visible = state.y <= top && top + h <= state.y + layout.viewport_h;
        if !visible {
            let target = if state.y > top {
                (top - layout.viewport_h * 0.25).max(0.0)
            } else {
                (top + h - layout.viewport_h * 0.75).max(0.0)
            };
            state.y = target.min(layout.max_scroll_y);
            self.scroll.insert(tab_id, state);
            self.needs_redraw = true;
        }
    }

    /// 「在文件夹中显示」：打开下载目录（面板按钮触发；文件管理器随平台）。
    pub(crate) fn show_download_in_folder(&mut self) {
        let dir = self.download_target_dir();
        #[cfg(target_os = "macos")]
        let program = "open";
        #[cfg(target_os = "windows")]
        let program = "explorer";
        #[cfg(all(unix, not(target_os = "macos")))]
        let program = "xdg-open";
        let spawned = std::process::Command::new(program).arg(&dir).spawn().is_ok();
        self.last_opened_download_dir = Some(dir.display().to_string());
        tracing::info!(
            "SMOKE_EVENT component=browser event=download_show_in_folder dir={} spawned={spawned}",
            dir.display()
        );
        self.needs_redraw = true;
    }

    pub fn open_downloads_page(&mut self) {
        let html = pages::generate_downloads_html(self.shell.downloads());
        self.open_internal_list_page("zero://downloads", html, "Downloads");
    }

    /// 打开书签管理页面。
    pub fn open_bookmarks_page(&mut self) {
        self.open_bookmarks_page_filtered(None);
    }

    /// 打开带过滤词的书签管理页面（地址栏 URL 为 `zero://bookmarks?q=…`）。
    fn open_bookmarks_page_filtered(&mut self, query: Option<&str>) {
        let html = pages::generate_bookmarks_html(self.shell.bookmarks(), query);
        let url = match query.map(str::trim).filter(|q| !q.is_empty()) {
            Some(q) => format!("zero://bookmarks?q={}", pages::percent_encode_query_value(q)),
            None => "zero://bookmarks".to_string(),
        };
        self.open_internal_list_page(&url, html, "Bookmarks");
    }

    /// 打开书签编辑表单页（`zero://bookmarks/edit?id=…`）；ID 无效时回书签列表。
    fn open_bookmarks_edit_page(&mut self, id: zero_browser_shell::BookmarkId, query: Option<&str>) {
        let Some(bookmark) = self.shell.bookmarks().get(id) else {
            self.open_bookmarks_page_filtered(query);
            return;
        };
        let html = pages::generate_bookmarks_edit_html(bookmark, query);
        let query_suffix = match query.map(str::trim).filter(|q| !q.is_empty()) {
            Some(q) => format!("&q={}", pages::percent_encode_query_value(q)),
            None => String::new(),
        };
        let url = format!("zero://bookmarks/edit?id={}{}", id.0, query_suffix);
        self.open_internal_list_page(&url, html, "Bookmarks");
    }

    /// 打开书签导入页（`zero://bookmarks/import`；设置页「导入书签」入口）。
    /// `outcome` 非空时展示上一次导入结果。
    fn open_bookmarks_import_page(&mut self, outcome: Option<pages::BookmarkImportOutcome>) {
        let html = pages::generate_bookmarks_import_html(outcome.as_ref());
        self.open_internal_list_page("zero://bookmarks/import", html, "Bookmarks");
    }

    /// 从剪贴板导入书签（`zero://bookmarks/import/apply`）。
    ///
    /// 剪贴板文本按 Netscape bookmark 格式解析（[`pages::parse_bookmarks_import_html`]），
    /// 解析出的条目经 shell 合并（URL 去重、文件夹同名复用、快照上限内截断），
    /// 完成后回导入页展示结果；书签栏随 shell 状态刷新。
    fn apply_bookmarks_import(&mut self) {
        let outcome = match crate::clipboard::read_text() {
            Some(text) => {
                let entries = pages::parse_bookmarks_import_html(&text);
                if entries.is_empty() {
                    tracing::info!("bookmarks import: no entries parsed from clipboard");
                    pages::BookmarkImportOutcome::NoEntries
                } else {
                    let (added, skipped) = self.shell.import_bookmarks(&entries);
                    tracing::info!("bookmarks import: added={added} skipped={skipped}");
                    pages::BookmarkImportOutcome::Imported { added, skipped }
                }
            }
            None => {
                tracing::warn!("bookmarks import: clipboard unavailable");
                pages::BookmarkImportOutcome::ClipboardUnavailable
            }
        };
        self.open_bookmarks_import_page(Some(outcome));
    }

    /// T62-D1：处理渲染端 zero:// 导航装载请求。
    ///
    /// D2 来源门禁：文件导入动作 URL 仅认导入页发起的表单提交（initiator 为渲染端
    /// 权威当前页——内部页的 shell 侧 tab URL 因 try_apply 早退不更新，不可用）。
    /// 非导入页来源（远端页面锚点/表单直链动作 URL）或路径为空：不执行本地文件
    /// 读取，落导入页。其余 zero:// GET 走 navigate_to_request（内部分派与 F5 共用）。
    pub(crate) fn handle_renderer_tab_navigate(
        &mut self,
        tab_id: TabId,
        request: zero_protocol::message::TabNavigateRequestParams,
    ) {
        if self.shell.active_tab_id() != Some(tab_id) {
            return;
        }
        if request.url.starts_with("zero://bookmarks/import/file?") {
            // D2 来源门禁：仅导入页发起的表单提交放行（initiator 为渲染端权威当前
            // 页——内部页的 shell 侧 tab URL 因 try_apply 早退不更新，不可用）；
            // 执行点统一在 try_apply_bookmarks_url（一次性放行位）。
            if request.initiator.as_deref() == Some("zero://bookmarks/import") {
                self.allow_file_import_once = true;
            }
            self.navigate_to_request(&request.url, "GET".to_string(), None);
            return;
        }
        self.navigate_to_request(&request.url, request.method, request.body);
    }

    /// 从文件导入书签（`zero://bookmarks/import/file?path=...`；T62 文件源）。
    ///
    /// 与剪贴板导入共用 Netscape 解析器与 shell 合并逻辑；读取失败与「未解析出
    /// 条目」都给用户可见结果（防数据丢失：不静默丢弃，失败回显路径供核对）。
    fn apply_bookmarks_import_from_file(&mut self, path: String) {
        let outcome = match std::fs::read_to_string(&path) {
            Ok(text) => {
                let entries = pages::parse_bookmarks_import_html(&text);
                if entries.is_empty() {
                    tracing::info!("bookmarks import(file): no entries parsed");
                    pages::BookmarkImportOutcome::FileNoEntries
                } else {
                    let (added, skipped) = self.shell.import_bookmarks(&entries);
                    tracing::info!("bookmarks import(file): added={added} skipped={skipped}");
                    pages::BookmarkImportOutcome::Imported { added, skipped }
                }
            }
            Err(err) => {
                tracing::warn!("bookmarks import(file): cannot read {path}: {err}");
                pages::BookmarkImportOutcome::FileUnavailable { path }
            }
        };
        self.open_bookmarks_import_page(Some(outcome));
    }

    /// 处理书签管理页内部动作 URL（过滤 / 编辑表单 / 删除 / 保存提交 / 导入）。
    ///
    /// `zero://bookmarks` 精确路径不在此处理（列表页经 [`Self::load_local_tab_url`] 装载）。
    pub(crate) fn try_apply_bookmarks_url(&mut self, url: &str) -> bool {
        if url == "zero://bookmarks/import" {
            self.open_bookmarks_import_page(None);
            return true;
        }
        if url == "zero://bookmarks/import/apply" {
            self.apply_bookmarks_import();
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://bookmarks/import/file?") {
            // T62-D2：文件导入动作 URL 的唯一执行点。放行位仅由两类可信入口置位：
            // UI 直输（地址栏/自动补全/书签栏，navigate_from_ui）与导入页自身表单
            // 提交（渲染端 TabNavigateRequest 携带 initiator，来源门禁见
            // handle_renderer_tab_navigate）。内容锚点/远端表单/刷新分派无放行位
            // ——不执行本地文件读取，落导入页（Chrome 对 web→browser-scheme 同类
            // 阻断语义）。path 由 parse_form_query 单次解码（D5：无二次解码）。
            let allow = std::mem::take(&mut self.allow_file_import_once);
            let path = if allow {
                Self::parse_form_query(rest)
                    .into_iter()
                    .find(|(key, _)| key == "path")
                    .map(|(_, value)| value)
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            } else {
                String::new()
            };
            if path.is_empty() {
                self.open_bookmarks_import_page(None);
            } else {
                self.apply_bookmarks_import_from_file(path);
            }
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://bookmarks?q=") {
            // 搜索框 GET 提交 / 地址栏直输：按过滤词重开书签页。
            let query = pages::percent_decode_form_value(rest);
            self.open_bookmarks_page_filtered(Some(&query));
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://bookmarks/delete?id=") {
            // 单条删除：可选 `&q=` 保留过滤词（值已 percent-encode，先切参再解码，顺序不可反）。
            let (encoded_id, query) = match rest.split_once("&q=") {
                Some((value, q)) => (value, Some(pages::percent_decode_form_value(q))),
                None => (rest, None),
            };
            if let Ok(id) = encoded_id.parse::<u64>() {
                self.shell.remove_bookmark_by_id(zero_browser_shell::BookmarkId(id));
            }
            self.open_bookmarks_page_filtered(query.as_deref());
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://bookmarks/edit?id=") {
            let (encoded_id, query) = match rest.split_once("&q=") {
                Some((value, q)) => (value, Some(pages::percent_decode_form_value(q))),
                None => (rest, None),
            };
            match encoded_id.parse::<u64>() {
                Ok(id) => self.open_bookmarks_edit_page(zero_browser_shell::BookmarkId(id), query.as_deref()),
                Err(_) => self.open_bookmarks_page_filtered(query.as_deref()),
            }
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://bookmarks/save?") {
            // 编辑表单 GET 提交：id/title/url + 隐藏 q（过滤词）。
            let params = Self::parse_form_query(rest);
            let param = |name: &str| -> Option<String> {
                params
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.trim().to_string())
            };
            if let (Some(id), Some(title), Some(target)) = (
                param("id").and_then(|v| v.parse::<u64>().ok()),
                param("title").filter(|v| !v.is_empty()),
                param("url").filter(|v| !v.is_empty()),
            ) {
                self.shell
                    .update_bookmark(zero_browser_shell::BookmarkId(id), &title, &target);
            }
            self.open_bookmarks_page_filtered(param("q").as_deref());
            return true;
        }
        false
    }

    /// 解析渲染端表单 GET 提交的 query（`k=v&k=v`，值经 form-urlencoded 编码）。
    fn parse_form_query(query: &str) -> Vec<(String, String)> {
        query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| match pair.split_once('=') {
                Some((key, value)) => (key.to_string(), pages::percent_decode_form_value(value)),
                None => (pair.to_string(), String::new()),
            })
            .collect()
    }

    fn open_internal_list_page(&mut self, url: &str, html: String, title: &str) {
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.tabs.ensure_tab(tab_id);
        self.tabs.load_html(tab_id, &html, None, Some(url));
        if let Some(tab) = self.shell.active_tab_mut() {
            tab.set_loading(false);
            tab.set_title(title);
        }
        self.address_bar.set_text(url.to_string());
        self.needs_redraw = true;
    }

    pub(crate) fn should_show_download_panel(&self) -> bool {
        self.download_panel_open || self.shell.downloads().active_count() > 0
    }

    pub(crate) fn looks_like_search_query(text: &str) -> bool {
        let trimmed = text.trim();
        let authority = split_authority(trimmed).0;
        !trimmed.is_empty()
            && !trimmed.contains("://")
            && !trimmed.starts_with("localhost")
            // host:port（数字端口，如 myserver:8080）是 URL 而非搜索词的强信号；
            // authority 含空白时不适用（`meeting at 10:30` 的尾部 `:30` 不是端口），
            // 与 normalize_url 的空白守卫保持导航层一致
            && (authority_has_whitespace(authority) || !authority_has_numeric_port(authority))
            && (!trimmed.contains('.') || trimmed.contains(' '))
    }

    /// 处理 `zero://` 内部动作链接，成功时返回 `true`。
    fn try_apply_internal_url(&mut self, url: &str) -> bool {
        if self.try_apply_settings_url(url) {
            return true;
        }
        if self.try_apply_feedback_url(url) {
            return true;
        }
        if self.try_apply_bookmarks_url(url) {
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://history?q=") {
            // 历史页搜索框 GET 提交 / 地址栏直输：按过滤词重开历史页。
            let query = Self::percent_decode(rest);
            self.open_history_page_filtered(Some(&query));
            return true;
        }
        if let Some(rest) = url.strip_prefix("zero://history/remove?url=") {
            // 单条删除：`url=` 值经 percent-encode_query_value 编码（原 URL 内的
            // `&` 已转义为 %26），先按字面分隔符切参、再解码，顺序不可反。
            let (encoded_url, query) = match rest.split_once("&q=") {
                Some((value, q)) => (value, Some(Self::percent_decode(q))),
                None => (rest, None),
            };
            let target = Self::percent_decode(encoded_url);
            self.shell.history_mut().remove(&target);
            self.open_history_page_filtered(query.as_deref());
            return true;
        }
        match url {
            "zero://history/clear" => {
                self.shell.history_mut().clear();
                self.open_history_page();
                true
            }
            _ => false,
        }
    }

    /// 处理反馈页内部链接（列表回跳 / 单条删除 / 导出），成功时返回 `true`。
    ///
    /// 删除走 `delete_report`（文件名信任校验在其中）；导出与 CLI zip 形态同一
    /// 管线（`build_feedback_zip_bytes`），经附件下载落盘到下载目录（同名去重
    /// 归下载层，对齐书签导出先例）。GUI 面不提供会话摘要与 URL 导出（NFR-1，
    /// 页面注明走 CLI）。
    pub(crate) fn try_apply_feedback_url(&mut self, url: &str) -> bool {
        if url == "zero://feedback" {
            self.open_feedback_page();
            return true;
        }
        if let Some(file_name) = url.strip_prefix("zero://feedback/remove/") {
            let dir = zero_browser_shell::crash_reports_dir();
            if let Err(error) = zero_browser_shell::delete_report(&dir, file_name) {
                tracing::debug!("feedback report delete rejected: {error}");
            }
            self.open_feedback_page();
            return true;
        }
        if url == "zero://feedback/export" {
            self.apply_feedback_export();
            return true;
        }
        false
    }

    /// 导出反馈包动作（`zero://feedback/export`）：全部现存报告 → zip 字节 →
    /// 下载管线落盘。无报告时不产生下载，仅回列表页（页面无导出按钮的兜底）。
    fn apply_feedback_export(&mut self) {
        let reports_dir = zero_browser_shell::crash_reports_dir();
        let reports = zero_browser_shell::list_reports(&reports_dir);
        if reports.is_empty() {
            self.open_feedback_page();
            return;
        }
        let selected: Vec<String> = reports.iter().map(|entry| entry.file_name.clone()).collect();
        let payload = zero_browser_shell::build_feedback_zip_bytes(
            &reports_dir,
            zero_product_version::VERSION,
            zero_product_version::BUILD_COMMIT,
            &selected,
            &zero_browser_shell::FeedbackExportOptions { session_summary: None },
        );
        let payload = match payload {
            Ok(payload) => payload,
            Err(error) => {
                tracing::warn!("feedback export failed: {error}");
                return;
            }
        };
        let Some(tab_id) = self.shell.active_tab_id() else {
            tracing::debug!("feedback export ignored: no active tab");
            return;
        };
        self.save_attachment_download(crate::process_backend::PendingAttachmentDownload {
            tab_id,
            url: "zero://feedback/export".to_string(),
            filename: payload.file_name,
            body: payload.bytes,
        });
    }

    /// 处理设置页内部链接（toggle / cycle / set），成功时返回 `true`。
    fn try_apply_settings_url(&mut self, url: &str) -> bool {
        if let Some(key) = url.strip_prefix("zero://settings/toggle/") {
            self.apply_settings_toggle(key);
            return true;
        }
        if url == "zero://settings/clear/history" {
            self.apply_settings_clear_history();
            return true;
        }
        if url == "zero://settings/clear/cookies" {
            self.apply_settings_clear_cookies();
            return true;
        }
        if url == "zero://settings/cycle/search_engine" {
            self.apply_settings_cycle_search_engine();
            return true;
        }
        if let Some(encoded) = url.strip_prefix("zero://settings/set/home_url/") {
            self.apply_settings_home_url(encoded);
            return true;
        }
        if url == "zero://settings/edit/home_url" {
            self.apply_settings_edit_home_url();
            return true;
        }
        if let Some(direction) = url.strip_prefix("zero://settings/adjust/default_zoom/") {
            self.apply_settings_adjust_default_zoom(direction);
            return true;
        }
        if let Some(value) = url.strip_prefix("zero://settings/set/default_zoom/") {
            self.apply_settings_default_zoom(value);
            return true;
        }
        if let Some(engine) = url.strip_prefix("zero://settings/set/search_engine/") {
            self.apply_settings_search_engine(engine);
            return true;
        }
        if url == "zero://settings/edit/download_directory" {
            self.apply_settings_edit_download_directory();
            return true;
        }
        if let Some(encoded) = url.strip_prefix("zero://settings/set/download_directory/") {
            self.apply_settings_download_directory(encoded);
            return true;
        }
        if url == "zero://settings/cycle/color_theme" {
            self.apply_settings_cycle_color_theme();
            return true;
        }
        if url == "zero://settings/export_bookmarks" {
            self.apply_settings_export_bookmarks();
            return true;
        }
        if let Some(name) = url.strip_prefix("zero://settings/set/color_theme/") {
            self.apply_settings_color_theme(name);
            return true;
        }
        false
    }

    /// 清除浏览历史（`zero://settings/clear/history`；设置页「清除浏览数据」行）。
    /// 与历史页 `zero://history/clear` 同一存储路径；磁盘落盘仍走退出时 persist_user_data。
    fn apply_settings_clear_history(&mut self) {
        self.shell.history_mut().clear();
        self.open_settings_page();
    }

    /// 导出书签为 Netscape bookmark 格式 HTML（`zero://settings/export_bookmarks`；
    /// 设置页「导出书签」）。生成整包后复用 attachment 下载管线落盘
    ///（与图片另存为同一路径，写入下载目录 bookmarks.html 并弹下载气泡）。
    fn apply_settings_export_bookmarks(&mut self) {
        let Some(tab_id) = self.shell.active_tab_id() else {
            tracing::debug!("export_bookmarks ignored: no active tab");
            return;
        };
        let body = pages::generate_bookmarks_export_html(self.shell.bookmarks()).into_bytes();
        self.save_attachment_download(crate::process_backend::PendingAttachmentDownload {
            tab_id,
            url: "zero://settings/export_bookmarks".to_string(),
            filename: "bookmarks.html".to_string(),
            body,
        });
    }

    /// 清除共享 Cookie jar（`zero://settings/clear/cookies`）。
    /// 覆盖 browser 进程权威 jar（顶层导航与 fetch 代理路径）；renderer 子进程本地
    /// jar 的跨进程清理需要新增 IPC 消息，属后续切片。
    fn apply_settings_clear_cookies(&mut self) {
        zero_net::shared_cookie_store()
            .lock()
            .expect("shared cookie store lock")
            .clear();
        self.open_settings_page();
    }

    fn percent_decode(input: &str) -> String {
        // 先按 %XX 还原为字节序列，再整体按 UTF-8 解码：搜索词等参数含中文时
        // 逐字节 `char::from(u8)` 会把 UTF-8 序列拆成 Latin-1 乱码。
        let bytes = input.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hex = &input[i + 1..i + 3];
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    decoded.push(value);
                    i += 3;
                    continue;
                }
            }
            decoded.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&decoded).into_owned()
    }

    /// 轮换默认搜索引擎。
    fn apply_settings_cycle_search_engine(&mut self) {
        let next = self.shell.settings().search_engine.cycle();
        self.shell.apply_settings(|settings| settings.search_engine = next);
        self.open_settings_page();
    }

    /// 轮换外壳主题偏好（Auto → Light → Dark → Auto）。
    fn apply_settings_cycle_color_theme(&mut self) {
        self.cycle_color_theme();
        self.open_settings_page();
    }

    /// 设置外壳主题偏好（`zero://settings/set/color_theme/auto|light|dark`）。
    fn apply_settings_color_theme(&mut self, name: &str) {
        let Some(theme) = ColorThemePreference::from_name(name) else {
            tracing::debug!(%name, "unknown color_theme setting");
            return;
        };
        self.shell.apply_settings(|settings| settings.color_theme = theme);
        self.apply_resolved_color_scheme(self.cached_window_theme);
        self.open_settings_page();
    }

    /// 设置主页 URL（`zero://settings/set/home_url/<percent-encoded>`）。
    fn apply_settings_home_url(&mut self, encoded: &str) {
        let home_url = Self::percent_decode(encoded).trim().to_string();
        if home_url.is_empty() {
            tracing::debug!("rejecting empty home_url setting");
            return;
        }
        self.shell.apply_settings(|settings| settings.home_url = home_url);
        self.open_settings_page();
    }

    /// 聚焦地址栏以输入自定义主页 URL。
    fn apply_settings_edit_home_url(&mut self) {
        self.address_bar.set_text("zero://settings/set/home_url/".to_string());
        self.address_bar_focused = true;
        // T37：聚焦重置光标闪烁相位，光标立即可见。
        self.address_bar_cursor_phase = Instant::now();
        self.autocomplete.clear();
        self.needs_redraw = true;
    }

    /// 调整默认缩放（`zero://settings/adjust/default_zoom/up|down`）。
    fn apply_settings_adjust_default_zoom(&mut self, direction: &str) {
        let delta = match direction {
            "up" => BrowserSettings::DEFAULT_ZOOM_STEP,
            "down" => -BrowserSettings::DEFAULT_ZOOM_STEP,
            _ => {
                tracing::debug!(%direction, "unknown default_zoom adjust direction");
                return;
            }
        };
        let zoom = self.shell.settings().adjust_default_zoom_by(delta);
        self.shell.apply_settings(|settings| settings.default_zoom = zoom);
        self.shell.set_zoom(zoom);
        self.open_settings_page();
    }

    /// 设置默认缩放（`zero://settings/set/default_zoom/<value>`）。
    fn apply_settings_default_zoom(&mut self, value: &str) {
        let Ok(parsed) = value.parse::<f32>() else {
            tracing::debug!(%value, "invalid default_zoom value");
            return;
        };
        let zoom = parsed.clamp(BrowserSettings::DEFAULT_ZOOM_MIN, BrowserSettings::DEFAULT_ZOOM_MAX);
        self.shell.apply_settings(|settings| settings.default_zoom = zoom);
        self.shell.set_zoom(zoom);
        self.open_settings_page();
    }

    fn apply_settings_search_engine(&mut self, name: &str) {
        let Some(engine) = SearchEngine::from_name(name) else {
            tracing::debug!(%name, "unknown search engine");
            return;
        };
        self.shell.apply_settings(|settings| settings.search_engine = engine);
        self.open_settings_page();
    }

    fn apply_settings_edit_download_directory(&mut self) {
        self.address_bar
            .set_text("zero://settings/set/download_directory/".to_string());
        self.address_bar_focused = true;
        // T37：聚焦重置光标闪烁相位，光标立即可见。
        self.address_bar_cursor_phase = Instant::now();
        self.autocomplete.clear();
        self.needs_redraw = true;
    }

    fn apply_settings_download_directory(&mut self, encoded: &str) {
        let path = Self::percent_decode(encoded).trim().to_string();
        self.shell.apply_settings(|settings| settings.download_directory = path);
        self.open_settings_page();
    }

    /// 应用设置页开关（`zero://settings/toggle/<key>`）。
    fn apply_settings_toggle(&mut self, key: &str) {
        let was_visible = self.bookmarks_bar_visible();
        match key {
            "show_bookmarks_bar" => {
                let show = !self.shell.settings().show_bookmarks_bar;
                self.shell.apply_settings(|settings| settings.show_bookmarks_bar = show);
            }
            "javascript_enabled" => {
                let enabled = !self.shell.settings().javascript_enabled;
                self.shell
                    .apply_settings(|settings| settings.javascript_enabled = enabled);
                self.tabs.set_javascript_enabled(enabled);
            }
            "cookies_enabled" => {
                let enabled = !self.shell.settings().cookies_enabled;
                self.shell.apply_settings(|settings| settings.cookies_enabled = enabled);
            }
            "block_third_party_cookies" => {
                let block = !self.shell.settings().block_third_party_cookies;
                self.shell
                    .apply_settings(|settings| settings.block_third_party_cookies = block);
            }
            "do_not_track" => {
                let dnt = !self.shell.settings().do_not_track;
                self.shell.apply_settings(|settings| settings.do_not_track = dnt);
            }
            _ => {
                tracing::debug!(%key, "unknown settings toggle key");
                return;
            }
        }

        self.open_settings_page();
        if self.bookmarks_bar_visible() != was_visible {
            self.sync_webview_viewport();
        }
    }

    /// 将用户数据（设置、书签、会话）写入默认配置文件。
    ///
    /// T20：会话（全部标签 + `clean_exit=true`）在干净退出时持久化，
    /// 下次启动按 `restore_session_at_startup` 恢复。
    pub fn persist_user_data(&self) {
        if let Err(err) = self.shell.save_settings() {
            tracing::warn!(%err, "failed to save settings");
        }
        if let Err(err) = self.shell.save_bookmarks() {
            tracing::warn!(%err, "failed to save bookmarks");
        }
        if let Err(err) = self.shell.save_session_to(&SessionState::default_session_path()) {
            tracing::warn!(%err, "failed to save session");
        }
    }

    /// 显式终止所有 `zero-renderer` 子进程。
    ///
    /// 必须在 `std::process::exit` 之前调用 —— `process::exit` 跳过 `Drop`，
    /// 会让 `zero-renderer.exe` 子进程成为孤儿，继续锁定自身可执行文件，
    /// 导致下次构建覆盖二进制时 `os error 5 拒绝访问`。
    pub fn shutdown_child_processes(&mut self) {
        self.tabs.shutdown_child_processes();
    }
}

// 输入处理方法（键盘、鼠标、IME、自动补全、上下文菜单）
// 拆分到独立文件以控制 app.rs 体积
include!("app_input.rs");

// 键盘输入处理（handle_key 及 find/address_bar/global 等键处理）
// 从 app_input.rs 进一步拆分以控制单文件体积
include!("app_input_keys.rs");

// 右键上下文菜单动作（show_context_menu / activate_context_menu_item 等）
// 从 app_input.rs 进一步拆分以控制单文件体积
include!("app_input_context_menus.rs");

// 渲染方法（build_scene 及所有 render_*）
// 拆分到独立文件以控制 app.rs 体积
include!("app_render.rs");
include!("app_render_ui.rs");

// 渲染工具函数（圆角矩形/圆形/几何裁剪等图元构造）
// 从 app_render.rs 进一步拆分以控制单文件体积
include!("app_render_geometry.rs");

// WebView 图元消费层（DC-10）：append_webview_primitives + ViewportClip + 视口/圆角裁剪 + 变换
// 从 app_render.rs 进一步拆分以控制单文件体积
include!("app_render_primitives.rs");

// 地址栏 UI 渲染（render_address_bar）
// 从 app_render.rs 进一步拆分以控制单文件体积
include!("app_render_address.rs");

// 平台相关独立函数（is_wayland、字体加载、颜色方案检测等）
// 拆分到独立文件以控制 app.rs 体积
include!("app_platform.rs");
include!("app_image_save.rs");
include!("app_page_fonts.rs");
include!("app_smoke_state.rs");
