//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zero_browser_shell::{BrowserShell, ContextMenu, ContextType, SuggestionSource, TabId};
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
use zero_webview::WebViewBuilder;

use crate::colors;
use crate::layout;
use crate::pages;

const TAB_BAR_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(450);

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
    /// 地址栏当前文本
    address_bar_text: String,
    /// 地址栏是否获得焦点
    pub address_bar_focused: bool,
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
            address_bar_text: String::new(),
            address_bar_focused: false,
            physical_size: (1024, 768),
            scale_factor: 1.0,
            needs_redraw: true,
            mouse_pos: (0.0, 0.0),
            ctrl_pressed: false,
            shift_pressed: false,
            autocomplete: AutocompleteState::new(),
            find_input: String::new(),
            tab_layout: Vec::new(),
            context_menu: ContextMenuState::new(),
            scroll_offset: HashMap::new(),
            pending_window_chrome_action: None,
            window_control_hover: None,
            window_is_maximized: false,
            last_tab_bar_blank_click: None,
            tab_bar_drag_press: None,
        }
    }

    /// Wayland 无系统装饰时需自绘窗口控制按钮
    pub fn uses_custom_window_controls(&self) -> bool {
        is_wayland()
    }

    /// 取出并清除待执行的窗口控制动作
    pub fn take_window_chrome_action(&mut self) -> Option<WindowChromeAction> {
        self.pending_window_chrome_action.take()
    }

    /// 同步窗口最大化/全屏状态（用于控制按钮图标）
    pub fn set_window_maximized(&mut self, maximized: bool) {
        if self.window_is_maximized != maximized {
            self.window_is_maximized = maximized;
            self.needs_redraw = true;
        }
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
        if !self.uses_custom_window_controls() || y >= layout::TAB_BAR_HEIGHT * s {
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
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        if y >= tab_bar_h {
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

    /// 测试用：构建场景（暴露私有方法给测试模块）
    #[cfg(test)]
    pub fn build_scene_for_test(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
        self.build_scene(width, height)
    }

    /// 测试用：向指定标签的 WebView 加载 HTML
    #[cfg(test)]
    pub fn load_webview_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.load_html(html, css);
        }
    }

    /// 测试用：获取 Ctrl 修饰键状态
    #[cfg(test)]
    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    /// 测试用：获取地址栏文本
    #[cfg(test)]
    pub fn address_bar_text(&self) -> &str {
        &self.address_bar_text
    }

    /// 计算网页内容区域物理像素尺寸
    pub fn content_physical_size(&self) -> (u32, u32) {
        let s = self.scale_factor;
        let chrome_h = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT + layout::STATUS_BAR_HEIGHT) * s;
        let content_w = self.physical_size.0;
        let content_h = (self.physical_size.1 as f32 - chrome_h).max(0.0) as u32;
        (content_w, content_h)
    }

    /// 创建指定视口尺寸的 WebView
    fn create_webview(&self) -> zero_webview::WebView {
        let (w, h) = self.content_physical_size();
        WebViewBuilder::new().width(w).height(h).build()
    }

    /// 获取或创建活跃标签页的 WebView
    pub fn ensure_webview(&mut self, tab_id: TabId) {
        if !self.webviews.contains_key(&tab_id) {
            let wv = self.create_webview();
            self.webviews.insert(tab_id, wv);
        }
    }

    /// 通过 WebView 加载指定标签页 URL
    fn fetch_tab_url(&mut self, tab_id: TabId, url: &str) {
        let result = match self.webviews.get_mut(&tab_id) {
            Some(wv) => wv.fetch_url(url),
            None => return,
        };

        match result {
            Ok(_) => {
                let title = self
                    .webviews
                    .get(&tab_id)
                    .and_then(|wv| wv.title().map(str::to_string))
                    .unwrap_or_else(|| url.to_string());
                self.shell.on_page_loaded(&title);
            }
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

    /// 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) {
        let url = normalize_url(url, &self.shell);
        tracing::info!("Navigating to: {url}");

        self.shell.navigate(&url);
        self.address_bar_text = url.clone();
        self.autocomplete.clear();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);

        // 重置滚动偏移
        self.scroll_offset.insert(tab_id, 0.0);

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
    }

    fn load_welcome_page(&mut self, tab_id: TabId) {
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.load_html(pages::WELCOME_HTML, None);
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
            self.address_bar_text = url.to_string();
        } else {
            self.address_bar_text.clear();
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

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
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

        self.address_bar_text = url.clone();
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
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

        self.address_bar_text = url.clone();
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
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
            wv.load_html(&html, None);
        }
        self.address_bar_text = "zero://settings".to_string();
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

        // 提取 Y 方向滚动量
        let delta_y = match delta {
            zero_host_runtime::event::MouseScrollDelta::PixelDelta(_, y) => y as f32,
            zero_host_runtime::event::MouseScrollDelta::LineDelta(_, y) => y * 40.0, // 每行约 40 像素
        };

        let content_h = self.content_physical_size().1 as f32;

        // 获取页面实际高度（如果有的话）
        let page_height = self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.last_render())
            .map(|r| {
                r.primitives
                    .fills
                    .iter()
                    .map(|f| f.rect.origin.y + f.rect.size.height)
                    .fold(0.0f32, f32::max)
            })
            .unwrap_or(content_h);

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
                "Up" if !self.context_menu.items.is_empty() => {
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
                "Down" if !self.context_menu.items.is_empty() => {
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
        match key {
            "Enter" => {
                let url = self.address_bar_text.trim().to_string();
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
                self.autocomplete.clear();
            }
            "Escape" => {
                self.address_bar_focused = false;
                self.autocomplete.clear();
                self.update_address_bar_from_active_tab();
            }
            "Backspace" => {
                self.address_bar_text.pop();
                self.update_autocomplete();
                self.needs_redraw = true;
            }
            "Down" => {
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
            "Up" => {
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
                    self.address_bar_text = sug.url().to_string();
                    self.autocomplete.clear();
                    self.needs_redraw = true;
                }
            }
            _ => {
                if key.len() == 1 {
                    self.address_bar_text.push_str(key);
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_global_key(&mut self, key: &str) {
        // Ctrl 修饰键快捷键
        if self.ctrl_pressed {
            match key {
                "l" | "L" => {
                    self.address_bar_focused = true;
                    self.address_bar_text.clear();
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
                    // 收藏当前页面
                    self.shell.add_bookmark();
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
            "Left" => {
                self.go_back();
            }
            "Right" => {
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
        let query = self.address_bar_text.trim();
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

        if (old_pos.0 - x).abs() > 1.0 || (old_pos.1 - y).abs() > 1.0 {
            let toolbar_h = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT) * self.scale_factor;
            if (y as f32) < toolbar_h {
                self.needs_redraw = true;
            }
            if (y as f32) < layout::TAB_BAR_HEIGHT * self.scale_factor {
                self.update_window_control_hover(x, y);
            }
            self.update_tab_bar_drag(x, y);
        }
    }

    /// 处理鼠标点击（物理像素坐标）
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool, button: &str) {
        if !pressed {
            if button == "Left" {
                self.tab_bar_drag_press = None;
            }
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

        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let addr_bar_h = layout::ADDRESS_BAR_HEIGHT * s;
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        let bookmarks_bar_h = layout::BOOKMARKS_BAR_HEIGHT * s;
        let chrome_top = toolbar_h + bookmarks_bar_h;
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
            let autocomplete_top = tab_bar_h + addr_bar_h;
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
        if y_f < tab_bar_h {
            if let Some(action) = self.window_control_hit_test(x_f, y_f, width, s) {
                self.pending_window_chrome_action = Some(action);
                self.needs_redraw = true;
                return;
            }

            let new_tab_x = self.new_tab_button_x();
            if x_f >= new_tab_x && x_f < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
                self.new_tab(None);
                return;
            }

            for &(id, tab_x, tab_w) in &self.tab_layout {
                if x_f >= tab_x && x_f < tab_x + tab_w {
                    let close_x = tab_x + tab_w - 24.0 * s;
                    let close_y_center = tab_bar_h / 2.0;
                    if x_f >= close_x
                        && x_f <= close_x + tab_close_size
                        && (y_f - close_y_center).abs() <= tab_close_size / 2.0
                    {
                        self.close_tab_by_id(id);
                        return;
                    }
                    if Some(id) != self.shell.active_tab_id() {
                        self.shell.switch_tab(id);
                        self.update_address_bar_from_active_tab();
                        self.needs_redraw = true;
                    }
                    return;
                }
            }

            if self.uses_custom_window_controls() && self.is_tab_bar_blank_hit(x_f, y_f, width, s) {
                self.handle_tab_bar_blank_press(x, y);
            }
            return;
        }

        // 3. 地址栏区域点击
        if y_f < tab_bar_h + addr_bar_h {
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
                if !self.address_bar_focused {
                    self.address_bar_focused = true;
                    self.needs_redraw = true;
                }
                return;
            }
        }

        // 4. 书签栏区域点击
        if y_f >= toolbar_h && y_f < chrome_top {
            self.handle_bookmark_bar_click(x_f, y_f, toolbar_h, width, s);
            return;
        }

        // 5. 查找栏区域点击
        if self.shell.find_state().is_active() && y_f >= chrome_top && y_f < chrome_top + layout::FIND_BAR_HEIGHT * s {
            let close_x = width - 40.0 * s;
            if x_f >= close_x {
                self.shell.find_close();
                self.find_input.clear();
                self.needs_redraw = true;
                return;
            }
            let prev_x = width - 100.0 * s;
            let next_x = width - 70.0 * s;
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

        // 6. 页面内容区域 — 取消地址栏焦点
        if y_f >= chrome_top && self.address_bar_focused {
            self.address_bar_focused = false;
            self.autocomplete.clear();
            self.needs_redraw = true;
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

    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let y_f = y as f32;
        let chrome_top = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT) * s;

        // 工具栏区域不显示上下文菜单
        if y_f < chrome_top {
            return;
        }

        let context_type = ContextType::Page;
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
            _ => {}
        }
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

        let autocomplete_top = (layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT) * s;
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

    /// Wayland 上是否强制使用 CPU softbuffer present（规避 wgpu swapchain 与 winit CSD 冲突）
    pub fn wayland_forces_cpu_present(&self) -> bool {
        is_wayland() && matches!(self.render_mode, RenderMode::Gpu | RenderMode::Auto)
    }

    /// 初始化 GPU 渲染器（Wayland 上跳过 wgpu 窗口 surface，改走 CPU present）
    pub fn init_gpu(&mut self, window: &std::sync::Arc<winit::window::Window>) {
        if matches!(self.render_mode, RenderMode::Cpu) || self.wayland_forces_cpu_present() {
            if self.wayland_forces_cpu_present() {
                tracing::warn!(
                    "Wayland: wgpu window surface disabled (focus-switch crash); using CPU softbuffer present"
                );
            }
            return;
        }

        match GpuRenderer::new_for_window(std::sync::Arc::clone(window)) {
            Ok(renderer) => {
                tracing::info!("GPU renderer initialized (format: {:?})", renderer.surface_format());
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                if matches!(self.render_mode, RenderMode::Gpu) {
                    tracing::error!("GPU renderer init failed: {e}");
                } else {
                    tracing::warn!("GPU renderer init failed: {e}; using CPU renderer");
                }
            }
        }
    }

    /// 窗口失焦：Wayland 上销毁 GPU 渲染器，避免 swapchain 在失焦后 commit
    pub fn on_window_unfocused(&mut self) {
        if is_wayland() {
            if self.gpu_renderer.is_some() {
                tracing::debug!("Wayland unfocus: releasing GPU renderer");
                self.gpu_renderer = None;
                self.surface_configured = false;
            }
        } else {
            self.suspend_gpu_present();
        }
    }

    /// 初始化 CPU 软件渲染 surface
    pub fn init_cpu_surface(
        &mut self,
        window: &std::sync::Arc<winit::window::Window>,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
    ) {
        if cpu_surface.is_some() {
            return;
        }

        match softbuffer::Context::new(std::sync::Arc::clone(window))
            .and_then(|context| softbuffer::Surface::new(&context, std::sync::Arc::clone(window)))
        {
            Ok(surface) => {
                tracing::info!("CPU renderer initialized");
                *cpu_surface = Some(surface);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(err) => {
                tracing::error!("CPU renderer init failed: {err}");
            }
        }
    }

    /// 同步 IME 状态（Wayland 失焦时必须关闭，否则 subsurface commit 会导致 compositor 断开）
    pub fn sync_ime_state(&self, window: &winit::window::Window) {
        use winit::dpi::{LogicalPosition, LogicalSize};

        let needs_ime = self.window_focused && (self.address_bar_focused || self.shell.find_state().is_active());
        window.set_ime_allowed(needs_ime);

        if !needs_ime {
            return;
        }

        if self.address_bar_focused {
            let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
            let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
            let bar_y = layout::TAB_BAR_HEIGHT + 4.0;
            window.set_ime_cursor_area(
                LogicalPosition::new(bar_x, bar_y),
                LogicalSize::new(480.0, layout::ADDRESS_BAR_HEIGHT),
            );
        } else if self.shell.find_state().is_active() {
            window.set_ime_cursor_area(
                LogicalPosition::new(8.0, layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT + 4.0),
                LogicalSize::new(240.0, layout::FIND_BAR_HEIGHT),
            );
        }
    }

    /// 失焦时暂停 GPU swapchain present（非 Wayland，Wayland 直接销毁 renderer）
    pub fn suspend_gpu_present(&mut self) {
        if is_wayland() {
            return;
        }
        if let Some(gpu) = self.gpu_renderer_as_mut() {
            gpu.suspend_present();
        }
    }

    /// 获焦后恢复 GPU swapchain present（非 Wayland）
    pub fn resume_gpu_present(&mut self) {
        if is_wayland() {
            return;
        }
        if let Some(gpu) = self.gpu_renderer_as_mut() {
            gpu.resume_present();
        }
    }

    /// GPU 渲染一帧
    pub fn render_frame(&mut self, width: u32, height: u32, present: bool) {
        if !present || !self.window_focused {
            return;
        }
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            if renderer.is_present_suspended() {
                self.gpu_renderer = gpu;
                return;
            }
            let (fills, glyphs) = self.build_scene(width, height);
            renderer.render_scene(&fills, &self.font_loader, &mut self.glyph_cache, &glyphs);
        }
        self.gpu_renderer = gpu;
    }

    /// CPU 软件渲染一帧（`present` 为 false 时跳过）
    pub fn render_cpu(
        &mut self,
        width: u32,
        height: u32,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
        present: bool,
    ) {
        if !present {
            return;
        }

        let (fills, glyphs) = self.build_scene(width, height);
        let fb = render_scene_to_framebuffer(
            width,
            height,
            1.0,
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
        );
        present_rgba_to_softbuffer(cpu_surface, fb.width, fb.height, &fb.data);
    }

    /// 从活跃标签更新地址栏文本
    fn update_address_bar_from_active_tab(&mut self) {
        if let Some(tab) = self.shell.active_tab() {
            self.address_bar_text = tab.url().unwrap_or("").to_string();
        }
    }
}

/// 当前进程是否运行在 Wayland 上
pub fn is_wayland() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("WINIT_UNIX_BACKEND")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// 将 RGBA 像素提交到 softbuffer 表面
fn present_rgba_to_softbuffer(
    cpu_surface: &mut Option<
        softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
    >,
    width: u32,
    height: u32,
    rgba: &[u8],
) {
    use std::num::NonZeroU32;

    let Some(surface) = cpu_surface.as_mut() else {
        return;
    };

    let sw = match NonZeroU32::new(width.max(1)) {
        Some(w) => w,
        None => return,
    };
    let sh = match NonZeroU32::new(height.max(1)) {
        Some(h) => h,
        None => return,
    };

    if let Err(err) = surface.resize(sw, sh) {
        tracing::error!("CPU surface resize failed: {err}");
        return;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(err) => {
            tracing::error!("CPU surface buffer failed: {err}");
            return;
        }
    };

    for (dst, chunk) in buffer.iter_mut().zip(rgba.chunks_exact(4)) {
        *dst = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
    }

    if let Err(err) = buffer.present() {
        tracing::error!("CPU surface present failed: {err}");
    }
}

// 渲染方法（build_scene 及所有 render_*）和渲染工具函数
// 拆分到独立文件以控制 app.rs 体积
include!("app_render.rs");

/// URL 规范化 — 支持 URL 和搜索引擎回退
pub fn normalize_url(input: &str, shell: &BrowserShell) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.starts_with("ftp://") || input.starts_with("file://") || input.starts_with("data:") {
        return input.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{input}");
    }
    shell.settings().search(input)
}

/// 加载系统字体（主字体 + CJK/Emoji 回退链）
pub fn load_system_fonts(font_loader: &mut FontLoader) -> Option<u32> {
    let primary_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    let primary = primary_paths.iter().find_map(|path| {
        std::fs::read(path)
            .ok()
            .and_then(|data| font_loader.load_font(&data).ok())
    })?;

    let fallback_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\seguiemj.ttf",
    ];

    let mut fallbacks = Vec::new();
    for path in fallback_paths {
        if let Ok(data) = std::fs::read(path)
            && let Ok(id) = font_loader.load_font(&data)
            && id != primary
        {
            fallbacks.push(id);
        }
    }
    font_loader.set_fallback_chain(fallbacks);

    Some(primary)
}
