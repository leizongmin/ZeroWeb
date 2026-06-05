//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use zero_browser_shell::{BrowserShell, ContextMenu, ContextType, SuggestionSource, TabId};
use zero_engine::PrefersColorSchemeValue;
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
use zero_webview::WebViewBuilder;

use crate::colors;
use crate::input_keys::key_matches;
use crate::layout;
use crate::page_selection::{GlyphSelection, hit_test_glyph};
use crate::pages;
use crate::text_input::TextInput;

const TAB_BAR_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(450);

/// 标签页 URL 加载状态（先绘制 loading，再发起请求）。
enum TabFetchState {
    None,
    WaitingPaint(TabId, String),
    HttpInFlight {
        tab_id: TabId,
        url: String,
        rx: mpsc::Receiver<Result<String, String>>,
    },
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
    /// 每个标签页对应的 WebView
    webviews: HashMap<TabId, zero_webview::WebView>,
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

        let color_scheme = detect_system_color_scheme();

        Self {
            shell: BrowserShell::new(),
            webviews: HashMap::new(),
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
            self.webviews.get(&tab_id).and_then(|wv| wv.hit_test_link(doc_x, doc_y))
        } else {
            None
        };
        self.set_hovered_link_url(href);
    }

    /// 是否有进行中的标签页 fetch（含等待首帧绘制）。
    pub fn tab_fetch_active(&self) -> bool {
        !matches!(self.tab_fetch, TabFetchState::None)
    }

    /// 轮询后台 HTTP fetch 结果。
    pub fn poll_tab_fetch(&mut self) {
        let TabFetchState::HttpInFlight { tab_id, url, rx } = &self.tab_fetch else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        let tab_id = *tab_id;
        let url = url.clone();
        self.tab_fetch = TabFetchState::None;
        match result {
            Ok(html) => self.apply_fetched_html(tab_id, &url, &html),
            Err(error) => self.apply_fetch_error(tab_id, &url, &error),
        }
        self.needs_redraw = true;
    }

    /// 在绘制 loading 帧之后启动 fetch。
    pub fn begin_tab_fetch_after_paint(&mut self) {
        let state = std::mem::replace(&mut self.tab_fetch, TabFetchState::None);
        let TabFetchState::WaitingPaint(tab_id, url) = state else {
            self.tab_fetch = state;
            return;
        };
        if url.starts_with("http://") || url.starts_with("https://") {
            let (tx, rx) = mpsc::channel();
            let fetch_url = url.clone();
            std::thread::spawn(move || {
                let result = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .user_agent("ZeroBrowser/1.0")
                    .build()
                    .and_then(|client| client.get(&fetch_url).send())
                    .and_then(|response| response.text())
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
            self.tab_fetch = TabFetchState::HttpInFlight { tab_id, url, rx };
        } else {
            self.fetch_tab_url_sync(tab_id, &url);
        }
        self.needs_redraw = true;
    }

    fn apply_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        if self.color_scheme == scheme {
            return;
        }
        self.color_scheme = scheme;
        self.chrome_palette = colors::ChromePalette::for_scheme(scheme);
        for wv in self.webviews.values_mut() {
            wv.set_prefers_color_scheme(scheme);
        }
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

    /// 调整所有 WebView 视口尺寸，并在已有页面内容时按新尺寸重新布局
    pub fn resize_all_webviews(&mut self, w: u32, h: u32) {
        for wv in self.webviews.values_mut() {
            wv.resize(w, h);
            if wv.last_render().is_some() {
                wv.render();
            }
        }
    }

    /// 测试用：获取标签 WebView 的逻辑视口尺寸
    #[cfg(test)]
    pub fn webview_logical_size_for_tab(&self, tab_id: zero_browser_shell::TabId) -> Option<(u32, u32)> {
        self.webviews
            .get(&tab_id)
            .map(|wv| (wv.config().width, wv.config().height))
    }

    /// 测试用：构建场景（暴露私有方法给测试模块）
    #[cfg(test)]
    pub fn build_scene_for_test(
        &mut self,
        width: u32,
        height: u32,
    ) -> (Vec<FillPrimitive>, Vec<GlyphDraw>, Vec<FillPrimitive>) {
        self.build_scene(width, height)
    }

    /// 测试用：构建场景并 CPU 渲染为帧缓冲。
    #[cfg(test)]
    pub fn render_scene_for_test(&mut self, width: u32, height: u32) -> zero_render_foundation::surface::FrameBuffer {
        let (fills, glyphs, overlay_fills) = self.build_scene(width, height);
        render_scene_to_framebuffer(
            width,
            height,
            1.0,
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
            &overlay_fills,
        )
    }

    /// 测试用：当前 Chrome 配色
    #[cfg(test)]
    pub fn chrome_palette(&self) -> colors::ChromePalette {
        self.chrome_palette
    }

    /// 测试用：向指定标签的 WebView 加载 HTML
    #[cfg(test)]
    pub fn load_webview_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.load_html(html, css);
        }
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

    /// 创建指定视口尺寸的 WebView
    fn create_webview(&self) -> zero_webview::WebView {
        let (w, h) = self.content_logical_size();
        let mut wv = WebViewBuilder::new().width(w).height(h).build();
        wv.set_prefers_color_scheme(self.color_scheme);
        wv
    }

    /// 按当前窗口尺寸同步所有 WebView 的逻辑视口
    pub fn sync_webview_viewport(&mut self) {
        let (w, h) = self.content_logical_size();
        self.resize_all_webviews(w, h);
    }

    /// 获取或创建活跃标签页的 WebView
    pub fn ensure_webview(&mut self, tab_id: TabId) {
        if !self.webviews.contains_key(&tab_id) {
            let wv = self.create_webview();
            self.webviews.insert(tab_id, wv);
        }
    }

    /// 通过 WebView 加载指定标签页 URL（同步，用于 zero:// 等）
    fn fetch_tab_url_sync(&mut self, tab_id: TabId, url: &str) {
        if url == "zero://settings" {
            self.open_settings_page();
            return;
        }

        let result = match self.webviews.get_mut(&tab_id) {
            Some(wv) => wv.fetch_url(url),
            None => return,
        };

        match result {
            Ok(_) => self.finish_tab_load(tab_id, url),
            Err(e) => {
                tracing::warn!("Failed to fetch URL: {e}, loading error page");
                let error = e.to_string();
                let error_page = pages::generate_error_page(url, &error);
                if let Some(wv) = self.webviews.get_mut(&tab_id) {
                    wv.load_html(&error_page, None);
                }
                self.shell.on_page_error(&error);
            }
        }
    }

    fn finish_tab_load(&mut self, tab_id: TabId, url: &str) {
        let title = self
            .webviews
            .get(&tab_id)
            .and_then(|wv| pages::extract_html_title(wv.html_content()))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| url.to_string());
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.set_title(&title);
        }
        self.shell.on_page_loaded(&title);
        self.refresh_tab_favicon(tab_id, url);
    }

    fn apply_fetched_html(&mut self, tab_id: TabId, url: &str, html: &str) {
        let title = pages::extract_html_title(html)
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| url.to_string());
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.set_prefers_color_scheme(self.color_scheme);
            wv.complete_load(html, None);
            wv.set_title(&title);
        }
        self.shell.on_page_loaded(&title);
        self.refresh_tab_favicon(tab_id, url);
    }

    fn apply_fetch_error(&mut self, tab_id: TabId, url: &str, error: &str) {
        tracing::warn!("Failed to fetch URL: {error}, loading error page");
        let error_page = pages::generate_error_page(url, error);
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.set_prefers_color_scheme(self.color_scheme);
            wv.load_html(&error_page, None);
        }
        self.shell.on_page_error(error);
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
        let url = normalize_url(url, &self.shell);
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
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.set_prefers_color_scheme(self.color_scheme);
            wv.load_html(pages::WELCOME_HTML, None);
        }
        self.refresh_tab_favicon(tab_id, "zero://newtab");
        self.shell.on_page_loaded("ZeroBrowser");
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
        self.shell.tabs().any(|tab| tab.is_loading())
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
        if self.webviews.contains_key(&tab_id) {
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
        let webview = self.create_webview();
        self.webviews.insert(tab_id, webview);

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
            self.webviews.remove(&tab_id);
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
        self.webviews.remove(&id);
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

        let url = match self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.url().map(|s| s.to_string()))
        {
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
        self.ensure_webview(tab_id);
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.set_prefers_color_scheme(self.color_scheme);
            wv.load_html(&html, None);
        }
        self.shell.on_page_loaded("设置");
        self.address_bar.set_text("zero://settings".to_string());
        self.needs_redraw = true;
    }

    /// 处理鼠标滚轮滚动
    pub fn handle_scroll(&mut self, delta: zero_host_runtime::event::MouseScrollDelta) {
        // 上下文菜单打开时不滚动
        if self.context_menu.visible {
            return;
        }

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        let s = self.scale_factor;
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let content_bottom = content_y + content_h;
        let mouse_x = self.mouse_pos.0 as f32;
        let mouse_y = self.mouse_pos.1 as f32;

        // 仅在 WebView 内容区响应滚轮；mouse_pos 初始为 (0,0)，未移动过时不拦截
        if mouse_y > 0.0
            && (mouse_y < content_y
                || mouse_y >= content_bottom
                || mouse_x < content_x
                || mouse_x >= content_x + content_w)
        {
            return;
        }

        // 提取 Y 方向滚动量（滚轮向下增大 scroll offset，与 Linux/winit 符号相反故取反）
        let delta_y = match delta {
            zero_host_runtime::event::MouseScrollDelta::PixelDelta(_, y) => -(y as f32),
            zero_host_runtime::event::MouseScrollDelta::LineDelta(_, y) => -(y * 40.0),
        };

        self.ensure_webview(tab_id);

        let content_h = self.content_physical_size().1 as f32;

        // 文档高度：优先布局树，回退到图元包围盒
        let page_height_logical = self
            .webviews
            .get(&tab_id)
            .and_then(|wv| {
                wv.document_height()
                    .or_else(|| wv.last_render().map(|r| primitives_content_height(&r.primitives)))
            })
            .unwrap_or(0.0);

        let page_height = page_height_logical * s;
        let max_scroll = (page_height - content_h).max(0.0);
        let offset = self.scroll_offset.entry(tab_id).or_insert(0.0);
        *offset = (*offset + delta_y).clamp(0.0, max_scroll);

        self.needs_redraw = true;
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: &str, pressed: bool) {
        // 追踪修饰键状态
        match key {
            "Control" => {
                self.ctrl_pressed = pressed;
                return;
            }
            "Meta" | "MetaLeft" | "MetaRight" | "Super" | "SuperLeft" | "SuperRight" => {
                self.cmd_pressed = pressed;
                return;
            }
            "Shift" => {
                self.shift_pressed = pressed;
                return;
            }
            _ => {}
        }

        // 只处理按键按下事件
        if !pressed {
            return;
        }

        // 上下文菜单打开时，Escape 关闭菜单，其他按键忽略
        if self.context_menu.visible {
            match key {
                "Escape" => {
                    self.context_menu.close();
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Up") && !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| {
                            if i > 0 {
                                i - 1
                            } else {
                                self.context_menu.items.len() - 1
                            }
                        })
                        .unwrap_or(self.context_menu.items.len() - 1);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Down") && !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| (i + 1) % self.context_menu.items.len())
                        .unwrap_or(0);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                "Enter" => {
                    self.activate_context_menu_item();
                }
                _ => {}
            }
            return;
        }

        if self.shell.find_state().is_active() {
            self.handle_find_key(key);
        } else if self.address_bar_focused {
            self.handle_address_bar_key(key);
        } else {
            self.handle_global_key(key);
        }
    }

    fn handle_find_key(&mut self, key: &str) {
        match key {
            "Enter" => {
                if self.find_input.is_empty() {
                    self.shell.find_close();
                } else if self.shell.find_state().total_matches() == 0 {
                    self.shell.find_start(&self.find_input.clone());
                } else {
                    self.shell.find_next();
                }
                self.needs_redraw = true;
            }
            "Escape" => {
                self.shell.find_close();
                self.find_input.clear();
                self.needs_redraw = true;
            }
            "Backspace" => {
                self.find_input.pop();
                if self.find_input.is_empty() {
                    self.shell.find_close();
                } else {
                    self.shell.find_start(&self.find_input);
                }
                self.needs_redraw = true;
            }
            _ => {
                if key.len() == 1 {
                    self.find_input.push_str(key);
                    self.shell.find_start(&self.find_input);
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_address_bar_key(&mut self, key: &str) {
        let extend = self.shift_pressed;
        if self.is_modifier_pressed() {
            match key {
                "a" | "A" => {
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                    return;
                }
                "c" | "C" => {
                    let _ = self.address_bar.copy_selection();
                    return;
                }
                "x" | "X" => {
                    if self.address_bar.cut_selection() {
                        self.update_autocomplete();
                        self.needs_redraw = true;
                    }
                    return;
                }
                "v" | "V" => {
                    if self.address_bar.paste_from_clipboard() {
                        self.update_autocomplete();
                        self.needs_redraw = true;
                    }
                    return;
                }
                _ => {}
            }
        }

        match key {
            "Enter" => {
                let url = self.address_bar.text().trim().to_string();
                if !url.is_empty() {
                    let nav_url = if let Some(idx) = self.autocomplete.hovered_index {
                        self.autocomplete
                            .suggestions
                            .get(idx)
                            .map(|s| s.url().to_string())
                            .unwrap_or(url)
                    } else {
                        url
                    };
                    self.navigate_to(&nav_url);
                }
                self.address_bar_focused = false;
                self.address_bar_ime_preedit.clear();
                self.autocomplete.clear();
            }
            "Escape" => {
                self.address_bar_focused = false;
                self.address_bar_ime_preedit.clear();
                self.autocomplete.clear();
                self.update_address_bar_from_active_tab();
            }
            "Backspace" => {
                self.address_bar.delete_backward();
                self.update_autocomplete();
                self.needs_redraw = true;
            }
            "Delete" => {
                self.address_bar.delete_forward();
                self.update_autocomplete();
                self.needs_redraw = true;
            }
            k if key_matches(k, "Left") => {
                self.address_bar.move_left(extend);
                self.needs_redraw = true;
            }
            k if key_matches(k, "Right") => {
                self.address_bar.move_right(extend);
                self.needs_redraw = true;
            }
            "Home" => {
                self.address_bar.move_home(extend);
                self.needs_redraw = true;
            }
            "End" => {
                self.address_bar.move_end(extend);
                self.needs_redraw = true;
            }
            k if key_matches(k, "Down") => {
                if !self.autocomplete.suggestions.is_empty() {
                    let next = self
                        .autocomplete
                        .hovered_index
                        .map(|i| (i + 1).min(self.autocomplete.suggestions.len() - 1))
                        .unwrap_or(0);
                    self.autocomplete.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
            }
            k if key_matches(k, "Up") => {
                if let Some(i) = self.autocomplete.hovered_index {
                    if i > 0 {
                        self.autocomplete.hovered_index = Some(i - 1);
                    } else {
                        self.autocomplete.hovered_index = None;
                    }
                    self.needs_redraw = true;
                }
            }
            "Tab" => {
                if let Some(sug) = self.autocomplete.suggestions.first() {
                    self.address_bar.set_text(sug.url().to_string());
                    self.autocomplete.clear();
                    self.needs_redraw = true;
                }
            }
            _ => {
                if key.len() == 1 {
                    self.address_bar.insert_str(key);
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_global_key(&mut self, key: &str) {
        // Ctrl 修饰键快捷键
        if self.is_modifier_pressed() {
            match key {
                "l" | "L" => {
                    self.address_bar_focused = true;
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                }
                "c" | "C" => {
                    if self.address_bar_focused {
                        let _ = self.address_bar.copy_selection();
                    } else if self.copy_page_selection() {
                        self.needs_redraw = true;
                    }
                }
                "v" | "V" if self.address_bar_focused && self.address_bar.paste_from_clipboard() => {
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
                "x" | "X" if self.address_bar_focused && self.address_bar.cut_selection() => {
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
                "a" | "A" if self.address_bar_focused => {
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                }
                "t" | "T" => {
                    self.new_tab(None);
                }
                "w" | "W" => {
                    self.close_active_tab();
                }
                "r" | "R" => {
                    self.refresh_page();
                }
                "f" | "F" => {
                    self.find_input.clear();
                    self.shell.find_close();
                    self.needs_redraw = true;
                }
                "d" | "D" => {
                    let was_visible = self.bookmarks_bar_visible();
                    self.shell.add_bookmark();
                    if self.bookmarks_bar_visible() != was_visible {
                        self.sync_webview_viewport();
                    }
                    self.needs_redraw = true;
                }
                "+" | "=" => {
                    self.shell.zoom_in();
                    self.needs_redraw = true;
                }
                "-" => {
                    self.shell.zoom_out();
                    self.needs_redraw = true;
                }
                "0" => {
                    self.shell.zoom_reset();
                    self.needs_redraw = true;
                }
                "," => {
                    // Ctrl+, 打开设置页面
                    self.open_settings_page();
                }
                _ => {}
            }
            return;
        }

        // 无修饰键的全局快捷键（保留兼容无 Ctrl 的单键模式）
        match key {
            "l" => {
                self.address_bar_focused = true;
                self.needs_redraw = true;
            }
            "t" => {
                self.new_tab(None);
            }
            "w" => {
                self.close_active_tab();
            }
            "r" => {
                self.refresh_page();
            }
            k if key_matches(k, "Left") => {
                self.go_back();
            }
            k if key_matches(k, "Right") => {
                self.go_forward();
            }
            "Home" => {
                self.navigate_to("https://example.com");
            }
            "f" => {
                self.find_input.clear();
                self.shell.find_close();
                self.needs_redraw = true;
            }
            "+" | "=" => {
                self.shell.zoom_in();
                self.needs_redraw = true;
            }
            "-" => {
                self.shell.zoom_out();
                self.needs_redraw = true;
            }
            "0" => {
                self.shell.zoom_reset();
                self.needs_redraw = true;
            }
            "n" => {
                self.shell.find_next();
                self.find_input = self.shell.find_state().query().to_string();
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    /// 更新自动补全建议
    fn update_autocomplete(&mut self) {
        let query = self.address_bar.text().trim();
        if query.is_empty() {
            self.autocomplete.clear();
            return;
        }
        self.autocomplete.suggestions = self.shell.suggest(query);
        self.autocomplete.hovered_index = None;
    }

    /// 处理鼠标移动
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = (x, y);

        // 上下文菜单悬停检测
        if self.context_menu.visible {
            let hovered = self.context_menu_hit_test(x, y);
            if hovered != self.context_menu.hovered_index {
                self.context_menu.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        // 自动补全悬停
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            let hovered = self.autocomplete_hit_test(x, y);
            if hovered != self.autocomplete.hovered_index {
                self.autocomplete.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        let s = self.scale_factor;
        let y_f = y as f32;
        let chrome_bottom = self.chrome_top_y_for(s);
        if y_f < chrome_bottom {
            self.needs_redraw = true;
        }

        if (old_pos.0 - x).abs() > 1.0 || (old_pos.1 - y).abs() > 1.0 {
            if self.address_bar_drag && self.address_bar_focused && self.left_button_down {
                let s = self.scale_factor;
                let font_size = layout::CHROME_FONT_SIZE * s;
                let (bar_x, _, _, _) = self.address_bar_layout();
                let rel_x = (x as f32 - bar_x - 10.0 * s).max(0.0);
                let idx = self
                    .address_bar
                    .x_to_cursor(rel_x, |t| self.measure_ui_text_width(t, font_size));
                self.address_bar.set_cursor(idx, true);
                self.needs_redraw = true;
            }

            let toolbar_h = self.chrome_top_y_for(self.scale_factor);
            if (y as f32) < toolbar_h {
                self.needs_redraw = true;
            }
            if (y as f32) < layout::TAB_STRIP_HEIGHT * self.scale_factor {
                self.update_window_control_hover(x, y);
            }
            self.update_tab_bar_drag(x, y);
        }

        if self.page_selection_drag
            && self.left_button_down
            && let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32)
            && let Some(glyphs) = self.page_glyphs(tab_id)
            && let Some(idx) = hit_test_glyph(&glyphs, doc_x, doc_y)
            && let Some(sel) = self.page_selection.get_mut(&tab_id)
        {
            sel.focus = idx;
            self.needs_redraw = true;
        }

        self.update_hovered_link_at(x, y);
    }

    /// 处理鼠标点击（物理像素坐标）
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool, button: &str) {
        if button == "Left" {
            if pressed {
                self.left_button_down = true;
            } else {
                self.left_button_down = false;
                self.tab_bar_drag_press = None;
                self.address_bar_drag = false;
                if self.page_selection_drag {
                    self.page_selection_drag = false;
                    if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32) {
                        let collapsed = self.page_selection.get(&tab_id).is_none_or(|s| s.is_collapsed());
                        if collapsed
                            && let Some(href) = self.webviews.get(&tab_id).and_then(|wv| wv.hit_test_link(doc_x, doc_y))
                        {
                            self.navigate_to(&href);
                        }
                    }
                }
                return;
            }
        } else if !pressed {
            return;
        }

        // 右键 → 上下文菜单
        if button == "Right" {
            self.show_context_menu(x, y);
            return;
        }

        // 左键点击时关闭上下文菜单
        if self.context_menu.visible {
            if let Some(idx) = self.context_menu_hit_test(x, y) {
                // 点击菜单项
                self.context_menu.hovered_index = Some(idx);
                self.activate_context_menu_item();
                return;
            }
            // 点击菜单外关闭
            self.context_menu.close();
            self.needs_redraw = true;
            return;
        }

        let s = self.scale_factor;
        let y_f = y as f32;
        let x_f = x as f32;
        let width = self.physical_size.0 as f32;

        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        let chrome_top = self.chrome_top_y_for(s);
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let nav_btn_w = layout::NAV_BUTTON_WIDTH * s;
        let addr_padding = layout::ADDRESS_BAR_PADDING * s;
        let tab_close_size = layout::TAB_CLOSE_SIZE * s;
        let autocomplete_row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;

        // 1. 自动补全下拉区域点击
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            if let Some(idx) = self.autocomplete_hit_test(x, y) {
                let url = self.autocomplete.suggestions.get(idx).map(|s| s.url().to_string());
                if let Some(url) = url {
                    self.navigate_to(&url);
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                    return;
                }
            }
            let autocomplete_top = toolbar_h;
            let autocomplete_height = self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE) as f32
                * autocomplete_row_h;
            if y_f >= autocomplete_top && y_f < autocomplete_top + autocomplete_height {
                return;
            }
            self.autocomplete.clear();
        }

        // 2. 标签栏区域点击
        if y_f < tab_strip_h {
            if y_f >= tab_y
                && let Some(action) = self.window_control_hit_test(x_f, y_f, width, s)
            {
                self.pending_window_chrome_action = Some(action);
                self.needs_redraw = true;
                return;
            }

            if y_f >= tab_y {
                let new_tab_x = self.new_tab_button_x();
                if x_f >= new_tab_x && x_f < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
                    self.new_tab(None);
                    return;
                }

                for &(id, tab_x, tab_w) in &self.tab_layout {
                    if x_f >= tab_x && x_f < tab_x + tab_w {
                        let close_x = tab_x + tab_w - 24.0 * s;
                        let close_y_center = tab_y + tab_bar_h / 2.0;
                        if x_f >= close_x
                            && x_f <= close_x + tab_close_size
                            && (y_f - close_y_center).abs() <= tab_close_size / 2.0
                        {
                            self.close_tab_by_id(id);
                            return;
                        }
                        if Some(id) != self.shell.active_tab_id() {
                            self.shell.switch_tab(id);
                            self.set_hovered_link_url(None);
                            self.update_address_bar_from_active_tab();
                            self.needs_redraw = true;
                        }
                        return;
                    }
                }
            }

            if self.supports_tab_bar_window_drag() && self.is_tab_bar_blank_hit(x_f, y_f, width, s) {
                self.handle_tab_bar_blank_press(x, y);
            }
            return;
        }

        // 3. 地址栏区域点击
        if y_f < toolbar_h {
            let addr_bar_x = nav_w + addr_padding;

            if x_f < nav_w {
                let button_index = ((x_f - 8.0 * s) / nav_btn_w) as i32;
                match button_index {
                    0 => self.go_back(),
                    1 => self.go_forward(),
                    2 => self.refresh_page(),
                    3 => {
                        let home = self.shell.settings().home_url.clone();
                        self.navigate_to(&home);
                    }
                    _ => {}
                }
                return;
            }

            if x_f >= addr_bar_x && x_f <= width - addr_padding {
                self.handle_address_bar_press(x, y);
                return;
            }
        }

        // 4. 书签栏区域点击
        if y_f >= toolbar_h && y_f < chrome_top {
            self.handle_bookmark_bar_click(x_f, y_f, toolbar_h, width, s);
            return;
        }

        // 5. 查找栏区域点击
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        if self.shell.find_state().is_active() && y_f >= content_y && y_f < content_y + layout::FIND_BAR_HEIGHT * s {
            let bar_w = 320.0 * s;
            let bar_x = width - bar_w - 10.0 * s;
            if x_f >= bar_x && x_f <= bar_x + bar_w {
                let close_x = bar_x + bar_w - 40.0 * s;
                if x_f >= close_x {
                    self.shell.find_close();
                    self.find_input.clear();
                    self.needs_redraw = true;
                    return;
                }
                let prev_x = bar_x + bar_w - 100.0 * s;
                let next_x = bar_x + bar_w - 70.0 * s;
                if x_f >= prev_x && x_f < prev_x + 28.0 * s {
                    self.shell.find_previous();
                    self.needs_redraw = true;
                    return;
                }
                if x_f >= next_x && x_f < next_x + 28.0 * s {
                    self.shell.find_next();
                    self.needs_redraw = true;
                    return;
                }
                return;
            }
        }

        // 6. 页面内容区域 — 链接点击 / 取消地址栏焦点
        let find_bar_h = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };
        let page_top = content_y + find_bar_h;

        if y_f >= content_y
            && y_f < content_y + content_h
            && x_f >= content_x
            && x_f < content_x + content_w
            && y_f >= page_top
        {
            if button == "Left"
                && let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f)
                && let Some(glyphs) = self.page_glyphs(tab_id)
            {
                let idx = hit_test_glyph(&glyphs, doc_x, doc_y).unwrap_or(0);
                if self.shift_pressed {
                    if let Some(sel) = self.page_selection.get_mut(&tab_id) {
                        sel.focus = idx;
                    } else {
                        self.page_selection.insert(tab_id, GlyphSelection::collapsed(idx));
                    }
                } else {
                    self.page_selection.insert(tab_id, GlyphSelection::collapsed(idx));
                }
                self.page_selection_drag = true;
                self.needs_redraw = true;
            }

            if self.address_bar_focused {
                self.address_bar_focused = false;
                self.autocomplete.clear();
                self.needs_redraw = true;
            }
        }
    }

    /// 处理书签栏点击
    fn handle_bookmark_bar_click(&mut self, x: f32, _y: f32, _bar_y: f32, _width: f32, s: f32) {
        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let mut target_url: Option<String> = None;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;
            if x >= bx && x < bx + item_w {
                target_url = Some(bm.url().to_string());
                break;
            }
            bx += item_w + 8.0 * s;
        }

        if let Some(url) = target_url {
            self.navigate_to(&url);
        }
    }

    /// 处理 IME 输入（地址栏）
    pub fn handle_ime(&mut self, event: zero_host_runtime::event::ImeEvent) {
        if !self.address_bar_focused {
            return;
        }
        match event {
            zero_host_runtime::event::ImeEvent::Preedit { text, .. } => {
                self.address_bar_ime_preedit = text;
                self.needs_redraw = true;
            }
            zero_host_runtime::event::ImeEvent::Commit(text) => {
                self.address_bar_ime_preedit.clear();
                if !text.is_empty() {
                    self.address_bar.insert_str(&text);
                    self.update_autocomplete();
                }
                self.needs_redraw = true;
            }
            zero_host_runtime::event::ImeEvent::Enabled | zero_host_runtime::event::ImeEvent::Disabled => {}
        }
    }

    fn copy_page_selection(&self) -> bool {
        let Some(tab_id) = self.shell.active_tab_id() else {
            return false;
        };
        let Some(sel) = self.page_selection.get(&tab_id) else {
            return false;
        };
        if sel.is_collapsed() {
            return false;
        }
        let Some(glyphs) = self.page_glyphs(tab_id) else {
            return false;
        };
        let text = GlyphSelection::selected_text(&glyphs, sel);
        if text.is_empty() {
            return false;
        }
        crate::clipboard::write_text(&text)
    }

    fn page_doc_point(&self, x_f: f32, y_f: f32) -> Option<(TabId, f32, f32)> {
        let s = self.scale_factor;
        let tab_id = self.shell.active_tab_id()?;
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let find_bar_h = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };
        let page_top = content_y + find_bar_h;
        let content_bottom = content_y + content_h;
        if x_f < content_x || x_f >= content_x + content_w || y_f < page_top || y_f >= content_bottom {
            return None;
        }
        let scroll_y = self.scroll_offset.get(&tab_id).copied().unwrap_or(0.0);
        Some((tab_id, (x_f - content_x) / s, (y_f - page_top + scroll_y) / s))
    }

    /// 与渲染一致的页面 glyph 列表（含字体 reflow）。
    fn page_glyphs(&self, tab_id: TabId) -> Option<Vec<zero_render_foundation::primitive::GlyphPrimitive>> {
        let wv = self.webviews.get(&tab_id)?;
        let mut glyphs = wv.last_render()?.primitives.glyphs.clone();
        if let Some(primary) = self.font_id {
            reflow_webview_glyphs(&mut glyphs, &self.font_loader, primary);
        }
        Some(glyphs)
    }

    fn address_bar_layout(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = self.physical_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let inset = layout::ADDRESS_BAR_INPUT_V_INSET * s;
        let bar_y = layout::TAB_STRIP_HEIGHT * s + inset;
        let bar_h = layout::ADDRESS_BAR_HEIGHT * s - 2.0 * inset;
        (bar_x, bar_y, bar_w, bar_h)
    }

    fn address_bar_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        let s = self.scale_factor;
        if y_f >= layout::TOOLBAR_HEIGHT * s {
            return false;
        }
        let (bar_x, _, bar_w, _) = self.address_bar_layout();
        x_f >= bar_x && x_f <= bar_x + bar_w
    }

    fn handle_address_bar_press(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let (bar_x, _, _, _) = self.address_bar_layout();
        let rel_x = (x as f32 - bar_x - 10.0 * s).max(0.0);
        let measure = |t: &str| self.measure_ui_text_width(t, font_size);
        let idx = self.address_bar.x_to_cursor(rel_x, measure);
        let extend = self.shift_pressed;
        if let Some((last_t, last_x, last_y)) = self.address_bar_last_click
            && last_t.elapsed() < TAB_BAR_DOUBLE_CLICK_INTERVAL
            && (x - last_x).abs() < 5.0
            && (y - last_y).abs() < 5.0
        {
            self.address_bar.select_word_at(idx);
            self.address_bar_last_click = None;
            self.address_bar_focused = true;
            self.address_bar_drag = false;
            self.needs_redraw = true;
            return;
        }
        self.address_bar_last_click = Some((Instant::now(), x, y));
        self.address_bar.set_cursor(idx, extend);
        self.address_bar_focused = true;
        self.address_bar_drag = true;
        self.autocomplete.clear();
        self.needs_redraw = true;
    }

    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let y_f = y as f32;
        let x_f = x as f32;
        let chrome_top = self.chrome_top_y_for(s);

        let context_type = if self.address_bar_hit_test(x_f, y_f) {
            ContextType::Editable
        } else if y_f < chrome_top {
            return;
        } else if let Some(tab_id) = self.shell.active_tab_id()
            && self.page_selection.get(&tab_id).is_some_and(|sel| !sel.is_collapsed())
        {
            ContextType::Selection
        } else if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f)
            && self
                .webviews
                .get(&tab_id)
                .and_then(|wv| wv.hit_test_link(doc_x, doc_y))
                .is_some()
        {
            ContextType::Link
        } else {
            ContextType::Page
        };

        let menu = ContextMenu::new(context_type);
        let items: Vec<String> = menu
            .items()
            .iter()
            .map(|mi| {
                if mi.is_separator() {
                    "---".to_string()
                } else {
                    mi.label().to_string()
                }
            })
            .collect();

        self.context_menu = ContextMenuState {
            visible: true,
            context_type,
            items,
            hovered_index: None,
            x: x as f32,
            y: y as f32,
        };
        self.needs_redraw = true;
    }

    /// 激活上下文菜单中选中的项
    fn activate_context_menu_item(&mut self) {
        let idx = match self.context_menu.hovered_index {
            Some(i) => i,
            None => return,
        };

        let label = match self.context_menu.items.get(idx) {
            Some(l) => l.clone(),
            None => return,
        };

        self.context_menu.close();
        self.needs_redraw = true;

        match label.as_str() {
            "后退" => self.go_back(),
            "前进" => self.go_forward(),
            "重新加载" => self.refresh_page(),
            "复制" => {
                if self.context_menu.context_type == ContextType::Editable {
                    let _ = self.address_bar.copy_selection();
                } else {
                    let _ = self.copy_page_selection();
                }
            }
            "剪切" if self.address_bar.cut_selection() => {
                self.update_autocomplete();
            }
            "粘贴" if self.address_bar.paste_from_clipboard() => {
                self.update_autocomplete();
            }
            "全选" => {
                self.address_bar.select_all();
            }
            _ => {}
        }
        self.needs_redraw = true;
    }

    /// 上下文菜单命中检测
    fn context_menu_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        if !self.context_menu.visible {
            return None;
        }

        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = 28.0 * s;
        let menu_w = 200.0 * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;

        let x_f = x as f32;
        let y_f = y as f32;

        if x_f < menu_x || x_f > menu_x + menu_w || y_f < menu_y || y_f > menu_y + menu_h {
            return None;
        }

        let idx = ((y_f - menu_y) / row_h) as usize;
        if idx < self.context_menu.items.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// 自动补全下拉命中检测（物理像素坐标）
    fn autocomplete_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let s = self.scale_factor;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = self.physical_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;

        let autocomplete_top = layout::TOOLBAR_HEIGHT * s;
        let y_f = y as f32;
        let x_f = x as f32;

        if x_f < bar_x || x_f > bar_x + bar_w || y_f < autocomplete_top {
            return None;
        }

        let row_offset = y_f - autocomplete_top;
        if row_offset < 0.0 {
            return None;
        }

        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let index = (row_offset / row_h) as usize;
        if index
            < self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE)
        {
            Some(index)
        } else {
            None
        }
    }

    /// 从活跃标签更新地址栏文本
    fn update_address_bar_from_active_tab(&mut self) {
        if let Some(tab) = self.shell.active_tab() {
            self.address_bar.set_text(tab.url().unwrap_or("").to_string());
        }
    }
}

// 渲染方法（build_scene 及所有 render_*）和渲染工具函数
// 拆分到独立文件以控制 app.rs 体积
include!("app_render.rs");

// 平台相关独立函数（is_wayland、字体加载、颜色方案检测等）
// 拆分到独立文件以控制 app.rs 体积
include!("app_platform.rs");
