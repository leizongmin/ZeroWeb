//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zero_browser_shell::{BrowserShell, ContextMenu, ContextType, SuggestionSource, TabId};
use zero_engine::PrefersColorSchemeValue;
use zero_engine::set_char_measure_fn;
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_full_scene;
#[cfg(test)]
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::image_cache::ImageCache;
use zero_render_foundation::primitive::{FillPrimitive, GlyphPrimitive, GradientKind, RenderPrimitives};

use crate::colors;
use crate::input_keys::key_matches;
use crate::layout;
use crate::page_selection::{GlyphSelection, hit_test_glyph};
use crate::pages;
use crate::tab_manager::TabManager;
use crate::text_input::TextInput;
use crate::text_metrics;

const TAB_BAR_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(450);

/// 页面内容区指针拖拽（鼠标左键；RDP/远程桌面触摸常模拟为此路径）
struct ContentPointerDrag {
    start_x: f64,
    start_y: f64,
    last_y: f64,
    scrolling: bool,
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
    Close,
    StartDrag,
}

/// 自动补全建议缓存
struct AutocompleteState {
    /// 当前显示的建议列表
    suggestions: Vec<zero_browser_shell::Suggestion>,
    /// 鼠标悬停的索引
    hovered_index: Option<usize>,
}

impl AutocompleteState {
    fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            hovered_index: None,
        }
    }

    fn clear(&mut self) {
        self.suggestions.clear();
        self.hovered_index = None;
    }
}

/// 右键上下文菜单状态
pub struct ContextMenuState {
    /// 是否显示
    pub visible: bool,
    /// 菜单类型（预留用于区分不同场景的菜单行为）
    #[allow(dead_code)]
    pub context_type: ContextType,
    /// 菜单项标签列表
    pub items: Vec<String>,
    /// 悬停索引
    pub hovered_index: Option<usize>,
    /// 菜单左上角物理像素坐标
    pub x: f32,
    pub y: f32,
}

impl ContextMenuState {
    fn new() -> Self {
        Self {
            visible: false,
            context_type: ContextType::Page,
            items: Vec::new(),
            hovered_index: None,
            x: 0.0,
            y: 0.0,
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.items.clear();
        self.hovered_index = None;
    }
}

/// 浏览器应用状态
pub struct BrowserApp {
    /// 浏览器 Shell（标签页、书签、历史）
    pub shell: BrowserShell,
    /// 标签页运行时（每 Tab 独立 worker 或渲染进程）
    tabs: TabManager,
    /// GPU 渲染器
    gpu_renderer: Option<GpuRenderer>,
    /// 渲染模式
    render_mode: RenderMode,
    /// 字体加载器
    font_loader: FontLoader,
    /// Glyph 缓存
    glyph_cache: GlyphCache,
    /// 已加载的系统字体 ID
    font_id: Option<u32>,
    /// 是否已初始化 GPU 表面
    pub surface_configured: bool,
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
    /// 自动补全状态
    autocomplete: AutocompleteState,
    /// 查找栏输入文本
    find_input: String,
    /// 标签页布局缓存：每个标签页的 (x, width) 位置信息
    tab_layout: Vec<(TabId, f32, f32)>,
    /// 右键上下文菜单状态
    context_menu: ContextMenuState,
    /// 页面滚动偏移（物理像素）
    scroll_offset: HashMap<TabId, f32>,
    /// 页面文本选区（glyph 索引）
    page_selection: HashMap<TabId, GlyphSelection>,
    /// 页面选区拖拽中
    page_selection_drag: bool,
    /// 左键是否按下
    left_button_down: bool,
    /// 待执行的窗口控制动作
    pending_window_chrome_action: Option<WindowChromeAction>,
    /// 窗口控制按钮悬停索引（0=最小化, 1=最大化, 2=关闭）
    window_control_hover: Option<usize>,
    /// 窗口是否最大化（用于绘制还原图标）
    window_is_maximized: bool,
    /// 标签栏空白处上次点击（用于双击检测）
    last_tab_bar_blank_click: Option<(f64, f64, Instant)>,
    /// 标签栏空白处按下位置（移动超过阈值后触发拖动）
    tab_bar_drag_press: Option<(f64, f64)>,
    /// 标签栏 chrome 动画起始时间（loading 旋转环）
    chrome_anim_start: Instant,
    /// 系统颜色方案偏好
    color_scheme: PrefersColorSchemeValue,
    /// 浏览器外壳配色
    chrome_palette: colors::ChromePalette,
    /// 是否已用 winit 窗口主题同步过颜色方案
    color_scheme_window_synced: bool,
    /// 标签页 URL 加载（延迟绘制 loading / 后台 HTTP）
    tab_fetch: TabFetchState,
    /// 鼠标悬停链接时在浮动状态栏中显示的 URL
    hovered_link_url: Option<String>,
    /// 单指触摸滚动：`(touch_id, last_y 物理像素)`
    touch_scroll: Option<(u64, f64)>,
    /// 鼠标左键在页面内容区的拖拽（远程桌面触摸模拟）
    content_pointer_drag: Option<ContentPointerDrag>,
}

impl BrowserApp {
    /// 创建新的浏览器应用
    pub fn new(render_mode: RenderMode) -> Self {
        let mut font_loader = FontLoader::new();
        let font_id = load_system_fonts(&mut font_loader);

        if font_id.is_some() {
            tracing::info!("System font loaded");
        } else {
            tracing::warn!("No system font found, text rendering will be limited");
        }

        set_char_measure_fn(text_metrics::measure_char);

        let color_scheme = detect_system_color_scheme();

        Self {
            shell: BrowserShell::new(),
            tabs: TabManager::new((800, 600), color_scheme),
            gpu_renderer: None,
            render_mode,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            gpu_surface_stale: false,
            window_focused: true,
            address_bar: TextInput::new(),
            address_bar_focused: false,
            address_bar_ime_preedit: String::new(),
            address_bar_drag: false,
            address_bar_last_click: None,
            physical_size: (1024, 768),
            scale_factor: 1.0,
            needs_redraw: true,
            mouse_pos: (0.0, 0.0),
            ctrl_pressed: false,
            cmd_pressed: false,
            shift_pressed: false,
            autocomplete: AutocompleteState::new(),
            find_input: String::new(),
            tab_layout: Vec::new(),
            context_menu: ContextMenuState::new(),
            scroll_offset: HashMap::new(),
            page_selection: HashMap::new(),
            page_selection_drag: false,
            left_button_down: false,
            pending_window_chrome_action: None,
            window_control_hover: None,
            window_is_maximized: false,
            last_tab_bar_blank_click: None,
            tab_bar_drag_press: None,
            chrome_anim_start: Instant::now(),
            color_scheme,
            chrome_palette: colors::ChromePalette::for_scheme(color_scheme),
            color_scheme_window_synced: false,
            tab_fetch: TabFetchState::None,
            hovered_link_url: None,
            touch_scroll: None,
            content_pointer_drag: None,
        }
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
        let href = if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32) {
            self.tabs.hit_test_link(tab_id, doc_x, doc_y)
        } else {
            None
        };
        self.set_hovered_link_url(href);
    }

    /// 是否有进行中的标签页 fetch（含等待首帧绘制）。
    pub fn tab_fetch_active(&self) -> bool {
        !matches!(self.tab_fetch, TabFetchState::None)
    }

    /// 轮询 Tab worker / IPC 并处理页面事件。
    pub fn poll_tab_fetch(&mut self) {
        if self.tabs.poll(self.shell.active_tab_id()) {
            self.needs_redraw = true;
        }
        for (tab_id, title, url) in self.tabs.take_page_loaded_events() {
            self.shell.on_page_loaded(&title);
            self.refresh_tab_favicon(tab_id, &url);
            if self.shell.active_tab_id() == Some(tab_id)
                && let Some(tab) = self.shell.active_tab_mut()
            {
                tab.set_loading(false);
            }
        }
        for (tab_id, error) in self.tabs.take_page_error_events() {
            self.shell.on_page_error(&error);
            if self.shell.active_tab_id() == Some(tab_id)
                && let Some(tab) = self.shell.active_tab_mut()
            {
                tab.set_loading(false);
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

    /// 使用 winit 窗口主题更新颜色方案（`ZERO_BROWSER_COLOR_SCHEME` 已设置时跳过）。
    pub fn sync_color_scheme_from_window(&mut self, window: &winit::window::Window) {
        if self.color_scheme_window_synced || color_scheme_from_env().is_some() {
            return;
        }
        self.color_scheme_window_synced = true;

        let Some(theme) = window.theme() else {
            tracing::debug!("Window theme unavailable, keeping startup color scheme");
            return;
        };

        self.apply_color_scheme(match theme {
            winit::window::Theme::Dark => PrefersColorSchemeValue::Dark,
            winit::window::Theme::Light => PrefersColorSchemeValue::Light,
        });
    }

    /// Wayland 无系统装饰时需自绘窗口控制按钮
    pub fn uses_custom_window_controls(&self) -> bool {
        is_wayland()
    }

    /// macOS 一体化标题栏（系统 traffic lights 与标签栏同排）
    pub fn tab_bar_leading_inset(&self) -> f32 {
        if uses_unified_titlebar() {
            layout::MACOS_TRAFFIC_LIGHT_INSET
        } else {
            0.0
        }
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

    /// 最大化/全屏时视口底部额外留白（物理像素）；普通窗口仅保留 [`PAGE_FRAME_INSET_BOTTOM`]。
    fn page_frame_bottom_reserves(&self, scale: f32) -> (f32, f32) {
        if self.window_is_maximized {
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
        self.tab_layout.last().map(|&(_, x, w)| x + w).unwrap_or(0.0)
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
                WindowChromeAction::StartDrag => unreachable!(),
            });
        if hover != self.window_control_hover {
            self.window_control_hover = hover;
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
        self.tabs.resize_all(w, h);
    }

    /// 测试用：获取标签 WebView 的逻辑视口尺寸
    #[cfg(test)]
    pub fn webview_logical_size_for_tab(&self, _tab_id: zero_browser_shell::TabId) -> Option<(u32, u32)> {
        Some(self.tabs.logical_viewport())
    }

    /// 测试用：构建场景（暴露私有方法给测试模块）
    #[cfg(test)]
    pub fn build_scene_for_test(
        &mut self,
        width: u32,
        height: u32,
    ) -> (Vec<FillPrimitive>, Vec<GlyphDraw>, Vec<FillPrimitive>, Vec<GlyphDraw>) {
        self.build_scene(width, height)
    }

    /// 测试用：构建场景并 CPU 渲染为帧缓冲。
    #[cfg(test)]
    pub fn render_scene_for_test(&mut self, width: u32, height: u32) -> zero_render_foundation::surface::FrameBuffer {
        let (fills, glyphs, overlay_fills, overlay_glyphs) = self.build_scene(width, height);
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
        )
    }

    /// 测试用：当前 Chrome 配色
    #[cfg(test)]
    pub fn chrome_palette(&self) -> colors::ChromePalette {
        self.chrome_palette
    }

    /// 测试用：Tab 是否已有可滚动/可交互的页面内容。
    #[cfg(test)]
    fn is_tab_content_ready(&self, tab_id: TabId) -> bool {
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
        let _guard = crate::test_sync::tab_runtime_test_guard();
        for _ in 0..500 {
            self.tabs.poll(Some(tab_id));
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

    /// 测试用：获取标签页滚动偏移（物理像素）
    #[cfg(test)]
    pub fn scroll_offset_for_tab(&self, tab_id: TabId) -> f32 {
        self.scroll_offset.get(&tab_id).copied().unwrap_or(0.0)
    }

    /// 计算网页内容区域物理像素尺寸（用于滚动、合成区域）
    pub fn content_physical_size(&self) -> (u32, u32) {
        let (_, _, w, h) = self.page_content_rect();
        (w.max(0.0) as u32, h.max(0.0) as u32)
    }

    /// WebView 布局视口（CSS 逻辑像素，与 devicePixelRatio 对应）。
    ///
    /// 高度用 `floor` 而非 `round`，保证 `logical_h * scale_factor` 不超过内容区物理高度，
    /// 避免页面背景在底部溢出并盖住圆角。
    pub fn content_logical_size(&self) -> (u32, u32) {
        let s = self.scale_factor.max(f32::EPSILON);
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

    /// 按指定窗口物理尺寸计算内容区（边框内侧）。
    pub fn page_content_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let (x, y, w, h) = self.page_frame_rect_for(width, height);
        let border = layout::PAGE_FRAME_BORDER * self.scale_factor;
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
        if url.starts_with("zero://") {
            if url != "zero://settings" {
                self.load_welcome_page(tab_id);
            }
            return;
        }
        self.tabs.navigate(tab_id, url.to_string());
    }

    fn finish_tab_load(&mut self, tab_id: TabId, url: &str, title: &str) {
        self.shell.on_page_loaded(title);
        self.refresh_tab_favicon(tab_id, url);
    }

    fn schedule_tab_fetch(&mut self, tab_id: TabId, url: String) {
        if self.shell.active_tab_id() == Some(tab_id)
            && let Some(tab) = self.shell.active_tab_mut()
        {
            tab.set_loading(true);
        }
        self.tab_fetch = TabFetchState::WaitingPaint(tab_id, url);
        self.needs_redraw = true;
    }

    /// 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) {
        let url = normalize_url(&resolve_path_relative_url(url, &self.shell), &self.shell);
        tracing::info!("Navigating to: {url}");

        self.shell.navigate(&url);
        self.address_bar.set_text(url.clone());
        self.autocomplete.clear();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);

        // 重置滚动偏移与页面选区
        self.scroll_offset.insert(tab_id, 0.0);
        self.page_selection.remove(&tab_id);
        self.clear_tab_favicon(tab_id);

        self.schedule_tab_fetch(tab_id, url);
    }

    fn load_welcome_page(&mut self, tab_id: TabId) {
        self.tabs.ensure_tab(tab_id);
        self.tabs
            .load_html(tab_id, pages::WELCOME_HTML, None, Some("zero://newtab"));
        self.finish_tab_load(tab_id, "zero://newtab", "ZeroBrowser");
    }

    fn clear_tab_favicon(&mut self, tab_id: TabId) {
        let size = layout::TAB_ICON_SIZE * self.scale_factor;
        crate::tab_favicon::clear_tab_favicon(&mut self.font_loader, tab_id, size);
    }

    fn refresh_tab_favicon(&mut self, tab_id: TabId, page_url: &str) {
        let size = layout::TAB_ICON_SIZE * self.scale_factor;
        let html = Self::tab_html_hint(Some(page_url));
        crate::tab_favicon::ensure_tab_favicon(&mut self.font_loader, tab_id, Some(page_url), html, size);
    }

    pub fn any_tab_loading(&self) -> bool {
        self.shell.tabs().any(|tab| tab.is_loading()) || self.tabs.any_loading()
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
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        if self.tabs.has_tab(tab_id) {
            return;
        }
        self.init_default_tab();
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
        self.scroll_offset.insert(tab_id, 0.0);
        self.needs_redraw = true;
    }

    /// 创建新标签页
    pub fn new_tab(&mut self, url: Option<&str>) {
        let tab_id = self.shell.new_tab(url);
        self.tabs.ensure_tab(tab_id);

        if let Some(url) = url {
            self.address_bar.set_text(url.to_string());
        } else {
            self.address_bar.clear();
            self.load_welcome_page(tab_id);
        }

        self.scroll_offset.insert(tab_id, 0.0);
        self.needs_redraw = true;
    }

    /// 关闭活跃标签页
    pub fn close_active_tab(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.tabs.remove_tab(tab_id);
            self.scroll_offset.remove(&tab_id);
            self.shell.close_tab(tab_id);

            if self.shell.is_empty() {
                self.new_tab(None);
            }

            self.update_address_bar_from_active_tab();
            self.needs_redraw = true;
        }
    }

    /// 关闭指定 ID 的标签页
    fn close_tab_by_id(&mut self, id: TabId) {
        self.tabs.remove_tab(id);
        self.scroll_offset.remove(&id);
        self.shell.close_tab(id);

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

        self.schedule_tab_fetch(tab_id, url);
    }

    /// 执行后退导航
    pub fn go_back(&mut self) {
        if !self.shell.go_back() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar.set_text(url.clone());
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        self.schedule_tab_fetch(tab_id, url);
    }

    /// 执行前进导航
    pub fn go_forward(&mut self) {
        if !self.shell.go_forward() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar.set_text(url.clone());
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        self.schedule_tab_fetch(tab_id, url);
    }

    /// 打开设置页面（about:preferences）
    pub fn open_settings_page(&mut self) {
        let html = pages::generate_settings_html(self.shell.settings());
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.tabs.ensure_tab(tab_id);
        self.tabs.load_html(tab_id, &html, None, Some("zero://settings"));
        self.shell.on_page_loaded("设置");
        self.address_bar.set_text("zero://settings".to_string());
        self.needs_redraw = true;
    }
}

// 输入处理方法（键盘、鼠标、IME、自动补全、上下文菜单）
// 拆分到独立文件以控制 app.rs 体积
include!("app_input.rs");

// 渲染方法（build_scene 及所有 render_*）
// 拆分到独立文件以控制 app.rs 体积
include!("app_render.rs");

// 渲染工具函数（圆角矩形/圆形/几何裁剪等图元构造）
// 从 app_render.rs 进一步拆分以控制单文件体积
include!("app_render_geometry.rs");

// 平台相关独立函数（is_wayland、字体加载、颜色方案检测等）
// 拆分到独立文件以控制 app.rs 体积
include!("app_platform.rs");
