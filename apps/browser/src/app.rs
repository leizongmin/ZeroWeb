//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zero_browser_shell::{
    BrowserMenuLabel, BrowserSettings, BrowserShell, ColorThemePreference, ContextMenu, ContextType, FindWrapHint,
    MenuItem, SearchEngine, SuggestionSource, TabId, TabMenuLabel, UiLanguage, browser_menu_label, tab_menu_label,
};
use zero_engine::PrefersColorSchemeValue;
use zero_engine::{set_char_measure_fn, set_text_shape_fn};
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
use crate::input_keys::key_matches;
use crate::layout;
use crate::page_scroll::{self, TabScrollState};
use crate::page_selection::{GlyphSelection, hit_test_caret};
use crate::pages;
use crate::tab_manager::TabManager;
use crate::text_input::TextInput;
use crate::text_metrics;

const TAB_BAR_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(450);

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
    /// 右键上下文菜单状态
    context_menu: ContextMenuState,
    /// 打开菜单的同一次左键按下后，忽略紧随其后的左键释放（避免按下 `...` 打开菜单时被立即关闭）。
    context_menu_suppress_left_up: bool,
    /// 页面滚动偏移（物理像素）
    scroll: HashMap<TabId, TabScrollState>,
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
    /// 缩放百分比浮层显示起始时刻。None 表示不显示。
    /// zoom_in/out/reset 触发时记录，3 秒后由渲染层清除。
    zoom_indicator_start: Option<Instant>,
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
    /// 是否显示下载浮动面板（也可因活动下载自动展开）。
    download_panel_open: bool,
    #[cfg(test)]
    compositor_status_override: Option<crate::compositor_client::CompositorStatus>,
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

        let shell = BrowserShell::new_with_persisted_settings();
        let detected = detect_system_color_scheme();
        let color_scheme = resolve_effective_color_scheme(shell.settings().color_theme, None, detected);

        let mut app = Self {
            shell,
            tabs: TabManager::new((800, 600), color_scheme),
            gpu_renderer: None,
            last_rendered_tab: None,
            render_mode,
            font_loader,
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
            context_menu: ContextMenuState::new(),
            context_menu_suppress_left_up: false,
            scroll: HashMap::new(),
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
            zoom_indicator_start: None,
            last_window_title: String::new(),
            last_tab_click_time: None,
            last_tab_click_id: None,
            background_tab_titles: HashMap::new(),
            tab_drag: None,
            touch_tap_candidate: None,
            touch_long_press: None,
            color_scheme,
            chrome_palette: colors::ChromePalette::for_scheme(color_scheme),
            cached_window_theme: None,
            tab_fetch: TabFetchState::None,
            hovered_link_url: None,
            touch_scroll: None,
            content_pointer_drag: None,
            scrollbar_drag: None,
            scrollbar_hover: None,
            scrollbar_visible_until: None,
            favicon_fetch: FaviconFetchState::new(),
            download_panel_open: false,
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
        if self.favicon_fetch.poll(&mut self.font_loader) {
            self.needs_redraw = true;
        }
        if self.favicon_fetch.poll_bookmarks(&mut self.font_loader) {
            self.needs_redraw = true;
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
        if let Some(id) = self.shell.active_tab_id() {
            self.clamp_tab_scroll(id);
        }
        for (tab_id, title, url) in self.tabs.take_page_loaded_events() {
            if self.shell.active_tab_id() == Some(tab_id) {
                self.shell.on_page_loaded(&title);
            }
            self.refresh_tab_favicon(tab_id, &url);
            self.shell.set_tab_crashed(tab_id, false);
            if self.shell.active_tab_id() == Some(tab_id) {
                if let Some(tab) = self.shell.active_tab_mut() {
                    tab.set_loading(false);
                }
                self.shell.set_tab_needs_attention(tab_id, false);
            } else {
                self.shell.set_tab_needs_attention(tab_id, true);
            }
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
    /// - Windows：禁用了系统标题栏，改用自绘
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
        self.tab_layout.last().map(|&(_, x, w)| x + w).unwrap_or(0.0)
    }

    /// 测试 helper：暴露「+」按钮 x 坐标。
    #[cfg(test)]
    pub fn new_tab_button_x_for_test(&self) -> f32 {
        self.new_tab_button_x()
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
    pub fn set_legacy_frame_publish_for_test(&mut self, tab_id: TabId) {
        self.tabs.set_legacy_frame_publish_for_test(tab_id);
    }

    #[cfg(test)]
    pub fn set_compositor_status_for_test(&mut self, status: crate::compositor_client::CompositorStatus) {
        self.compositor_status_override = Some(status);
    }

    /// R3254 测试 helper：last_render 的 glyph 数量（诊断合成帧内容）。
    #[cfg(test)]
    pub fn last_render_glyphs_for_test(&self, tab_id: TabId) -> Option<usize> {
        self.tabs.last_render(tab_id).map(|r| r.primitives.glyphs.len())
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

    /// 测试用：注入标签页渲染快照（不经过 worker，避免异步覆盖）。
    #[cfg(test)]
    pub fn inject_tab_render_for_test(
        &mut self,
        tab_id: TabId,
        render: zero_webview::WebViewRenderResult,
        document_height: f32,
    ) {
        self.tabs.ensure_snapshot_for_test(tab_id);
        if let Some(snap) = self.tabs.snapshot_mut(tab_id) {
            snap.last_render = Some(render);
            snap.document_height = Some(document_height);
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

    /// 测试用：获取标签页垂直滚动偏移（物理像素）
    #[cfg(test)]
    pub fn scroll_offset_for_tab(&self, tab_id: TabId) -> f32 {
        self.scroll.get(&tab_id).map(|s| s.y).unwrap_or(0.0)
    }

    /// 当前标签页滚动状态。
    pub fn tab_scroll_state(&self, tab_id: TabId) -> TabScrollState {
        self.scroll.get(&tab_id).copied().unwrap_or_default()
    }

    /// 文档内容尺寸（物理像素）。
    fn document_size_physical(&self, tab_id: TabId) -> (f32, f32) {
        let s = self.scale_factor;
        let logical_h = self
            .tabs
            .document_height(tab_id)
            .or_else(|| {
                self.tabs
                    .last_render(tab_id)
                    .map(|r| page_scroll::primitives_content_height(&r.primitives))
            })
            .unwrap_or(0.0);
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
        let s = self.scale_factor.max(f32::EPSILON);
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
        let s = self.scale_factor;
        let normal = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        self.context_menu
            .items
            .iter()
            .map(|it| if it.is_separator() { sep } else { normal })
            .sum()
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

    /// 浮动下载面板外框（物理像素）。
    pub(crate) fn download_panel_rect_for(&self, width: u32, height: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let panel_w = layout::DOWNLOAD_PANEL_WIDTH * s;
        let panel_h = layout::DOWNLOAD_PANEL_HEIGHT * s;
        let margin = layout::DOWNLOAD_PANEL_FLOAT_MARGIN * s;
        (cx + cw - panel_w - margin, cy + ch - panel_h - margin, panel_w, panel_h)
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
            "zero://history" => self.open_history_page(),
            "zero://downloads" => self.open_downloads_page(),
            "zero://bookmarks" => self.open_bookmarks_page(),
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

    fn navigate_to_request(&mut self, url: &str, method: String, body: Option<String>) {
        if method == "GET" && body.is_none() && self.try_apply_internal_url(url) {
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
        tracing::info!("Navigating to: {method} {url}");

        self.shell.navigate(&url);
        self.address_bar.set_text(url.clone());
        self.autocomplete.clear();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);

        // 重置滚动偏移与页面选区
        self.scroll.insert(tab_id, TabScrollState::default());
        self.page_selection.remove(&tab_id);
        self.clear_tab_favicon(tab_id);

        self.start_tab_request(tab_id, url, method, body);
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
        self.scroll.insert(tab_id, TabScrollState::default());
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
            self.shell.close_tab(tab_id);

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
        self.background_tab_titles.remove(&id);
        self.shell.close_tab(id);

        if self.shell.is_empty() {
            self.new_tab(None);
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 复制指定标签页，副本插入其后并设为活跃。
    fn duplicate_tab_by_id(&mut self, id: TabId) {
        if let Some(new_id) = self.shell.duplicate_tab(id) {
            self.tabs.ensure_tab(new_id);
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
        }
        self.shell.close_other_tabs(id);

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
        }
        self.shell.close_tabs_to_right(id);

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
        self.needs_redraw = true;
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

    /// 查看当前标签页 HTML 源代码。
    pub fn view_page_source(&mut self, tab_id: TabId) {
        let Some(html) = self.tabs.page_html(tab_id) else {
            return;
        };
        let source_url = self.tabs.page_url(tab_id).unwrap_or_else(|| "about:blank".to_string());
        let view_url = format!("view-source:{source_url}");
        let page = pages::generate_view_source_page(&source_url, &html);
        self.open_internal_document_tab(page, &view_url, "查看源代码");
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
        self.address_bar.select_all();
        self.needs_redraw = true;
    }

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

        self.start_tab_load(tab_id, url);
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

        self.start_tab_load(tab_id, url);
    }

    /// 打开设置页面（about:preferences）
    pub fn open_settings_page(&mut self) {
        self.open_internal_list_page(
            "zero://settings",
            pages::generate_settings_html(self.shell.settings()),
            "设置",
        );
    }

    /// 打开历史记录页面。
    pub fn open_history_page(&mut self) {
        let html = pages::generate_history_html(self.shell.history());
        self.open_internal_list_page("zero://history", html, "History");
    }

    /// 打开下载管理页面。
    pub fn open_downloads_page(&mut self) {
        let html = pages::generate_downloads_html(self.shell.downloads());
        self.open_internal_list_page("zero://downloads", html, "Downloads");
    }

    /// 打开书签管理页面。
    pub fn open_bookmarks_page(&mut self) {
        let html = pages::generate_bookmarks_html(self.shell.bookmarks());
        self.open_internal_list_page("zero://bookmarks", html, "Bookmarks");
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
        !trimmed.is_empty()
            && !trimmed.contains("://")
            && !trimmed.starts_with("localhost")
            && (!trimmed.contains('.') || trimmed.contains(' '))
    }

    /// 处理 `zero://` 内部动作链接，成功时返回 `true`。
    fn try_apply_internal_url(&mut self, url: &str) -> bool {
        if self.try_apply_settings_url(url) {
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

    /// 处理设置页内部链接（toggle / cycle / set），成功时返回 `true`。
    fn try_apply_settings_url(&mut self, url: &str) -> bool {
        if let Some(key) = url.strip_prefix("zero://settings/toggle/") {
            self.apply_settings_toggle(key);
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
        if let Some(name) = url.strip_prefix("zero://settings/set/color_theme/") {
            self.apply_settings_color_theme(name);
            return true;
        }
        false
    }

    fn percent_decode(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hex = &input[i + 1..i + 3];
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(char::from(value));
                    i += 3;
                    continue;
                }
            }
            out.push(char::from(bytes[i]));
            i += 1;
        }
        out
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

    /// 将用户数据（设置、书签等）写入默认配置文件。
    pub fn persist_user_data(&self) {
        if let Err(err) = self.shell.save_settings() {
            tracing::warn!(%err, "failed to save settings");
        }
        if let Err(err) = self.shell.save_bookmarks() {
            tracing::warn!(%err, "failed to save bookmarks");
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
